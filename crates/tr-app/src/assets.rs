//! Terraria asset loading: real game textures extracted from `.xnb` to PNG.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use rayon::prelude::*;
use tr_core::World;
use tr_render::{AtlasImage, SheetLayout};

pub fn usage(world: &World) -> (BTreeSet<u16>, BTreeSet<u16>) {
    let mut tiles = BTreeSet::new();
    let mut walls = BTreeSet::new();
    for t in &world.tiles {
        // Tile id 0 is dirt, not air; only the active flag distinguishes air.
        if t.active() {
            tiles.insert(t.kind);
        }
        if t.wall != 0 {
            walls.insert(t.wall);
        }
    }
    (tiles, walls)
}

pub fn image_name_for_tile(kind: u16) -> String {
    format!("Tiles_{kind}")
}

pub fn image_name_for_wall(wall: u16) -> String {
    format!("Wall_{wall}")
}

/// Loads only the sheets referenced by the world (plus a dirt fallback).
pub fn load_world_images(
    dir: &Path,
    tiles: &BTreeSet<u16>,
    walls: &BTreeSet<u16>,
) -> Result<Vec<AtlasImage>> {
    let mut images = Vec::new();
    let mut missing = Vec::new();

    // C5: decode sheets on Rayon's pool. `par_iter().collect()` keeps iterator
    // order, so the atlas layout (and every UV) stays bit-identical to the
    // sequential loader.
    let tile_images: Vec<(u16, Result<AtlasImage>)> = tiles
        .par_iter()
        .map(|&kind| {
            (
                kind,
                load_png(dir, &image_name_for_tile(kind), SheetLayout::Tile),
            )
        })
        .collect();
    for (kind, result) in tile_images {
        match result {
            Ok(img) => images.push(img),
            Err(_) => missing.push(image_name_for_tile(kind)),
        }
    }

    let wall_images: Vec<(u16, Result<AtlasImage>)> = walls
        .par_iter()
        .map(|&wall| {
            (
                wall,
                load_png(dir, &image_name_for_wall(wall), SheetLayout::Wall),
            )
        })
        .collect();
    for (wall, result) in wall_images {
        match result {
            Ok(img) => images.push(img),
            Err(_) => missing.push(image_name_for_wall(wall)),
        }
    }

    // Fallback sheet so unknown ids still render something instead of vanishing.
    if !images.iter().any(|i| i.name == "Tiles_0") {
        if let Ok(img) = load_png(dir, "Tiles_0", SheetLayout::Tile) {
            images.push(img);
        }
    }

    // Whole-image sheets used by the sky/background pass.
    for name in [
        "Background_0",
        // Underground: `Background_1` is the surface transition row, `Background_2`
        // the tiled dirt backdrop (`_drawBackground_backTexture[0]/[1]`).
        "Background_1",
        "Background_2",
        // Surface-background layer sets: forest mountains 7/8, snow mountains 35/36,
        // forest tree lines 50..52 and snow tree lines 37..39.
        "Background_7",
        "Background_8",
        "Background_35",
        "Background_36",
        "Background_37",
        "Background_38",
        "Background_39",
        "Background_50",
        "Background_51",
        "Background_52",
        "Background_28",
        "Background_21",
        "Background_20",
        "Background_112",
        "Background_113",
        "Sun",
        "Moon_0",
        "Star_0",
        "Star_1",
        "Star_2",
        "Star_3",
        "Player_0_0",
        "Player_0_1",
        "Player_0_2",
        "Player_0_3",
        "Player_0_6",
        "Player_0_7",
        "Player_0_10",
        "Player_0_11",
        "Player_0_12",
        "Player_Hair_1",
        "NPC_1",
        "NPC_2",
        "NPC_3",
        "NPC_50",
        "Rain",
        "Dust",
        "Projectile_3",
        "Item_71",
        "Item_72",
        "Item_73",
        "Item_74",
        "Item_2",
        "Item_3",
        "Item_8",
        "Item_9",
        "Item_11",
        "Item_12",
        "Item_13",
        "Item_14",
        "Item_26",
        "Item_30",
        "Item_42",
        "Item_169",
        "Item_172",
        "Item_174",
        "Item_276",
        "Item_593",
        "Logo",
        "Inventory_Back",
        "Inventory_Back2",
        "Inventory_Back9",
        "Inventory_Back14",
        "Heart",
        "Bubble",
        "NPC_Head_Boss_7",
        "UI/UI_BossBar",
    ] {
        match load_png(dir, name, SheetLayout::Raw) {
            Ok(img) => images.push(img),
            Err(_) => log::warn!("missing background texture {name}"),
        }
    }

    // Tree canopies and branches are separate free-form textures (`Tree_Tops_*`,
    // `Tree_Branches_*`); the trunk itself lives in `Tiles_5.png`.
    for style in 0..32 {
        for prefix in ["Tree_Tops_", "Tree_Branches_"] {
            let name = format!("{prefix}{style}");
            if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
                images.push(img);
            }
        }
    }

    // Sky clouds (`Cloud_0`..`Cloud_41`), drawn as a parallax layer.
    for i in 0..42 {
        let name = format!("Cloud_{i}");
        if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
            images.push(img);
        }
    }

    // Liquid surfaces: `Images/Misc/water_0..14` (water styles, lava, honey,
    // shimmer). 48x1360 each: 3 columns of 16 px and 17 animation blocks of 80 px.
    for i in 0..15 {
        let name = format!("water_{i}");
        if let Ok(img) = load_misc_png(dir, &name) {
            images.push(img);
        }
    }

    // `DrawTile_LiquidBehindTile`: `Liquid_*` is the 16 px partial strip and
    // `LiquidSlope_*` the four diagonal masks drawn behind blocks that stand in
    // liquid.
    for i in 0..15 {
        for prefix in ["Liquid_", "LiquidSlope_"] {
            let name = format!("{prefix}{i}");
            if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
                images.push(img);
            }
        }
    }

    // `WaterfallManager.waterfallTexture[0..27]`: the common sheet is 512x40
    // (sixteen 32x40 frames), the rain/dust variants are 144x18 (eight 18x18
    // frames) and the shimmer/disco companion sheets are 512x124.
    for i in 0..tr_core::waterfall::MAX_TYPES {
        let name = format!("Waterfall_{i}");
        if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
            images.push(img);
        }
    }

    if !missing.is_empty() {
        log::warn!(
            "{} world sheets have no matching texture (first 10: {:?})",
            missing.len(),
            &missing[..missing.len().min(10)]
        );
    }

    Ok(images)
}

/// `Content` directory bundled with the project (the retail `.xnb` files copied into
/// `assets/Content`), set once at startup by `main`. The game reads them at runtime with
/// the original XNB readers; there is no pre-extracted fallback.
pub fn set_content_dir(dir: PathBuf) {
    let _ = IMAGES_DIR.set(dir.join("Images"));
    let _ = CONTENT_DIR.set(dir);
}

static CONTENT_DIR: OnceLock<PathBuf> = OnceLock::new();
static IMAGES_DIR: OnceLock<PathBuf> = OnceLock::new();

/// The bundled `Content` directory, if it was found at startup.
pub fn content_dir() -> Option<&'static Path> {
    CONTENT_DIR.get().map(|p| p.as_path())
}

fn content_images_dir() -> Option<&'static Path> {
    IMAGES_DIR.get().map(|p| p.as_path()).filter(|p| p.is_dir())
}

/// Decodes one sheet straight from the game's `.xnb` (the original pipeline).
fn load_sheet(name: &str, layout: SheetLayout) -> Result<AtlasImage> {
    let dir = content_images_dir().context("Content/Images not found")?;
    let path = dir.join(format!("{name}.xnb"));
    let img = tr_content::load_texture(&path)
        .with_context(|| format!("decoding {}", path.display()))?;
    Ok(AtlasImage {
        name: name.to_string(),
        width: img.width,
        height: img.height,
        rgba: img.pixels,
        layout,
    })
}

pub fn load_png(_dir: &Path, name: &str, layout: SheetLayout) -> Result<AtlasImage> {
    load_sheet(name, layout)
}

fn load_misc_png(_dir: &Path, name: &str) -> Result<AtlasImage> {
    load_sheet(&format!("Misc/{name}"), SheetLayout::Raw)
}

/// One glyph of a ReLogic `DynamicSpriteFont`, in screen pixels.
#[derive(Clone, Copy, Debug)]
pub struct FontGlyph {
    pub uv: [f32; 4],
    pub w: f32,
    pub h: f32,
    /// `SpriteCharacterData.Padding`: draw offset inside the line box.
    pub pad: [f32; 2],
    /// `Kerning`: (left bearing, advance - z, z).
    pub kerning: [f32; 3],
}

/// A decoded `DynamicSpriteFont` bound to the atlas (`DynamicSpriteFont.DrawString`
/// walks these metrics exactly).
#[derive(Clone, Debug, Default)]
pub struct UiFont {
    pub glyphs: HashMap<char, FontGlyph>,
    pub line_spacing: f32,
    pub spacing: f32,
    pub fallback: char,
}

impl UiFont {
    pub fn glyph(&self, c: char) -> Option<&FontGlyph> {
        self.glyphs
            .get(&c)
            .or_else(|| self.glyphs.get(&self.fallback))
    }
}

/// Decodes `Fonts/Mouse_Text`, `Fonts/Item_Stack` and `Fonts/Death_Text` from the retail
/// content and returns their page textures as atlas images plus the raw metrics.
pub fn load_fonts() -> (
    Vec<AtlasImage>,
    Option<(String, tr_content::DynamicSpriteFont)>,
    Option<(String, tr_content::DynamicSpriteFont)>,
    Option<(String, tr_content::DynamicSpriteFont)>,
) {
    let mut images = Vec::new();
    let Some(images_dir) = content_images_dir() else {
        return (images, None, None, None);
    };
    let fonts_dir = images_dir.parent().map(|p| p.join("Fonts"));
    let mut ui = None;
    let mut item = None;
    let mut death = None;
    if let Some(dir) = fonts_dir {
        for (name, slot) in [("Mouse_Text", 0), ("Item_Stack", 1), ("Death_Text", 2)] {
            let path = dir.join(format!("{name}.xnb"));
            match tr_content::load_font(&path) {
                Ok(font) => {
                    let sheet = format!("Font_{name}");
                    for (page, pg) in font.pages.iter().enumerate() {
                        images.push(AtlasImage {
                            name: format!("{sheet}_{page}"),
                            width: pg.texture.width,
                            height: pg.texture.height,
                            rgba: pg.texture.pixels.clone(),
                            layout: SheetLayout::Raw,
                        });
                    }
                    log::info!(
                        "font {name}: {} glyphs, line spacing {}",
                        font.pages.first().map_or(0, |p| p.glyphs.len()),
                        font.line_spacing
                    );
                    match slot {
                        0 => ui = Some((sheet, font)),
                        1 => item = Some((sheet, font)),
                        _ => death = Some((sheet, font)),
                    }
                }
                Err(e) => log::warn!("font {name}: {e:#}"),
            }
        }
    }
    (images, ui, item, death)
}

/// Binds a decoded font to its atlas page UVs.
pub fn build_ui_font(
    atlas: &tr_render::Atlas,
    sheet: &str,
    font: &tr_content::DynamicSpriteFont,
) -> Option<UiFont> {
    let mut out = UiFont {
        line_spacing: font.line_spacing as f32,
        spacing: font.spacing,
        fallback: font.default_char,
        glyphs: HashMap::new(),
    };
    for (page, pg) in font.pages.iter().enumerate() {
        let rect = atlas.sheet(&format!("{sheet}_{page}"))?;
        for g in &pg.glyphs {
            let [sx, sy, w, h] = g.src;
            out.glyphs.insert(
                g.ch,
                FontGlyph {
                    uv: uv_rect(rect, (atlas.width, atlas.height), sx, sy, w as u32, h as u32),
                    w: w as f32,
                    h: h as f32,
                    pad: [g.pad[0] as f32, g.pad[1] as f32],
                    kerning: g.kerning,
                },
            );
        }
    }
    Some(out)
}

/// One resolved frame: atlas UV plus whether it fully covers its 16x16 cell.
#[derive(Clone, Copy, Debug)]
pub struct FrameTex {
    pub uv: [f32; 4],
    pub opaque: bool,
}

/// Texture information for one tile type present in the world.
#[derive(Clone, Debug)]
pub struct TypeTex {
    pub sheet: String,
    /// UV rectangle of the "plain" 16x16 frame (fallback / no frame stored).
    pub plain_uv: [f32; 4],
    pub plain_opaque: bool,
    /// Dense 18 px frame grid: index `(fy / 18) * cols + fx / 18`.
    pub cols: u32,
    pub rows: u32,
    pub frames: Vec<FrameTex>,
}

/// Texture information for one wall type present in the world.
#[derive(Clone, Debug)]
pub struct WallTex {
    pub sheet: String,
    /// `Framing.wallFrameLookup` cells: `row * 13 + col`. Each entry is the 32x32
    /// sprite `WallDrawing.DrawWalls` draws 8 px up/left of the cell.
    /// `None` when the sheet has no such cell.
    pub frames: Vec<Option<FrameTex>>,
}

/// Wall autotile table dimensions (columns 0..=12, rows 0..=4).
pub const WALL_COLS: usize = 13;
pub const WALL_ROWS: usize = 5;

/// Tree canopy + branch sheets for one foliage style.
#[derive(Clone, Copy, Debug)]
pub struct FoliageTex {
    pub top: tr_render::SheetRect,
    pub branch: tr_render::SheetRect,
}

/// One cloud texture, already reduced to what the sky pass needs.
#[derive(Clone, Copy, Debug)]
pub struct CloudTex {
    pub uv: [f32; 4],
    pub w: f32,
    pub h: f32,
}

/// Precomputed texture information for every type present in the world.
#[derive(Default)]
pub struct Lookups {
    /// Dense per-type tables: one slot per `TileID`, `None` if the sheet is absent.
    pub tiles: Vec<Option<TypeTex>>,
    pub walls: Vec<Option<WallTex>>,
    /// `Tree_Tops_*` / `Tree_Branches_*` per foliage style.
    pub foliage: Vec<Option<FoliageTex>>,
    /// `Cloud_0`..`Cloud_41`.
    pub clouds: Vec<Option<CloudTex>>,
    /// `Misc/water_0..14` sheets (water styles, lava, honey, shimmer).
    pub liquids: Vec<Option<tr_render::SheetRect>>,
    /// `LiquidSlope_0..14`: 4 diagonal masks (18 px stride) drawn behind slopes.
    pub liquid_slopes: Vec<Option<tr_render::SheetRect>>,
    /// `Liquid_0..14`: 16 px partial strips (18 px stride) drawn behind blocks.
    pub liquid_parts: Vec<Option<tr_render::SheetRect>>,
    /// `Waterfall_0..27` (`WaterfallManager.waterfallTexture`).
    pub waterfalls: [Option<tr_render::SheetRect>; tr_core::waterfall::MAX_TYPES],
    /// `Rain.png`: 2x40 drop frames at 4 px strides.
    pub rain: Option<tr_render::SheetRect>,
    /// `Dust.png`: 10 px cells, 8 px frames.
    pub dust: Option<tr_render::SheetRect>,
    /// `Projectile_3.png`: the 22x22 shuriken frame.
    pub shuriken: Option<tr_render::SheetRect>,
    /// `Item_71..74`: copper/silver/gold/platinum coin sprites.
    pub coins: [Option<tr_render::SheetRect>; 4],
    /// Vanilla HUD textures (`Images/Inventory_Back`, `Heart`, ...).
    pub ui_back: Option<tr_render::SheetRect>,
    /// `Inventory_Back14`: the yellow frame of the selected hotbar slot.
    pub ui_back_sel: Option<tr_render::SheetRect>,
    /// `Inventory_Back9`: the hotbar row frame inside the open inventory.
    pub ui_back_hotbar: Option<tr_render::SheetRect>,
    /// `Item_<id>` sprites for the HUD/inventory icons, keyed by `ItemID`.
    pub item_icons: HashMap<u16, tr_render::SheetRect>,
    /// `Fonts/Mouse_Text` for UI labels and item names.
    pub ui_font: Option<UiFont>,
    /// `Fonts/Item_Stack` for stack counts and slot numbers.
    pub item_font: Option<UiFont>,
    /// `Fonts/Death_Text`: the menu items (`FontAssets.DeathText`).
    pub death_font: Option<UiFont>,
    /// `Logo` (menu title) as UV plus native size.
    pub logo: Option<([f32; 4], f32, f32)>,
    pub ui_heart: Option<tr_render::SheetRect>,
    pub ui_bubble: Option<tr_render::SheetRect>,
    pub ui_boss_bar: Option<tr_render::SheetRect>,
    pub ui_boss_head: Option<tr_render::SheetRect>,
    /// Frames that do not sit on the 18 px grid (furniture and other important
    /// tiles). Populated lazily; bounded by the number of distinct visible frames.
    pub frame_cache: HashMap<(u16, i16, i16), FrameTex>,
    pub atlas_size: (u32, u32),
}

impl Lookups {
    pub fn build(
        atlas: &tr_render::Atlas,
        tile_names: impl Iterator<Item = (u16, String)>,
        wall_names: impl Iterator<Item = (u16, String)>,
        ui_font: Option<UiFont>,
        item_font: Option<UiFont>,
        death_font: Option<UiFont>,
    ) -> Self {
        let mut me = Self {
            tiles: vec![None; tr_core::tiledata::TILE_COUNT],
            walls: vec![None; tr_core::tiledata::WALL_COUNT],
            foliage: vec![None; 32],
            clouds: vec![None; 42],
            liquids: vec![None; 15],
            liquid_slopes: vec![None; 15],
            liquid_parts: vec![None; 15],
            waterfalls: [None; tr_core::waterfall::MAX_TYPES],
            frame_cache: HashMap::new(),
            rain: atlas.sheet("Rain").copied(),
            dust: atlas.sheet("Dust").copied(),
            shuriken: atlas.sheet("Projectile_3").copied(),
            coins: [71, 72, 73, 74].map(|i| atlas.sheet(&format!("Item_{i}")).copied()),
            ui_back: atlas.sheet("Inventory_Back").copied(),
            ui_back_sel: atlas.sheet("Inventory_Back14").copied(),
            ui_back_hotbar: atlas.sheet("Inventory_Back9").copied(),
            ui_font,
            item_font,
            death_font,
            logo: atlas.sheet("Logo").map(|r| {
                (
                    uv_rect(r, (atlas.width, atlas.height), 0, 0, r.width, r.height),
                    r.width as f32,
                    r.height as f32,
                )
            }),
            item_icons: [
                2, 3, 8, 9, 11, 12, 13, 14, 26, 30, 42, 169, 172, 174, 276, 593,
            ]
            .into_iter()
            .filter_map(|i| atlas.sheet(&format!("Item_{i}")).map(|r| (i, *r)))
            .collect(),
            ui_heart: atlas.sheet("Heart").copied(),
            ui_bubble: atlas.sheet("Bubble").copied(),
            ui_boss_bar: atlas.sheet("UI/UI_BossBar").copied(),
            ui_boss_head: atlas.sheet("NPC_Head_Boss_7").copied(),
            atlas_size: (atlas.width, atlas.height),
        };
        for (kind, name) in tile_names {
            let Some(rect) = atlas.sheet(&name) else {
                continue;
            };
            let plain = rect.plain_frame;
            let plain_uv = uv_of(atlas, rect, &name, plain.0, plain.1);
            let plain_opaque = atlas.frame_is_opaque(rect, plain.0, plain.1);
            let cols = rect.width / 18;
            let rows = rect.height / 18;
            let mut frames = Vec::with_capacity((cols * rows) as usize);
            for fy in 0..rows {
                for fx in 0..cols {
                    let px = (fx * 18) as i16;
                    let py = (fy * 18) as i16;
                    frames.push(FrameTex {
                        uv: uv_of(atlas, rect, &name, px, py),
                        opaque: atlas.frame_is_opaque(rect, px, py),
                    });
                }
            }
            if let Some(slot) = me.tiles.get_mut(kind as usize) {
                *slot = Some(TypeTex {
                    sheet: name,
                    plain_uv,
                    plain_opaque,
                    cols,
                    rows,
                    frames,
                });
            }
        }
        for (wall, name) in wall_names {
            let Some(rect) = atlas.sheet(&name) else {
                continue;
            };
            let mut frames = Vec::with_capacity(WALL_COLS * WALL_ROWS);
            for row in 0..WALL_ROWS {
                for col in 0..WALL_COLS {
                    // `WallDrawing.DrawWalls`: a 32x32 source at the 36 px cell origin,
                    // drawn 8 px up/left of the cell so neighbouring walls join.
                    let fx = col as i16 * 36;
                    let fy = row as i16 * 36;
                    if fx as u32 + 32 > rect.width || fy as u32 + 32 > rect.height {
                        frames.push(None);
                    } else {
                        frames.push(Some(FrameTex {
                            uv: uv_of_size(atlas, rect, &name, fx, fy, 32, 32),
                            opaque: atlas.rect_is_opaque(
                                rect,
                                fx as u32,
                                fy as u32,
                                32,
                                32,
                            ),
                        }));
                    }
                }
            }
            if let Some(slot) = me.walls.get_mut(wall as usize) {
                *slot = Some(WallTex {
                    sheet: name,
                    frames,
                });
            }
        }
        for style in 0..me.foliage.len() {
            let top = atlas.sheet(&format!("Tree_Tops_{style}"));
            let branch = atlas.sheet(&format!("Tree_Branches_{style}"));
            if let (Some(top), Some(branch)) = (top, branch) {
                me.foliage[style] = Some(FoliageTex {
                    top: *top,
                    branch: *branch,
                });
            }
        }
        for i in 0..me.clouds.len() {
            if let Some(rect) = atlas.sheet(&format!("Cloud_{i}")) {
                me.clouds[i] = Some(CloudTex {
                    uv: uv_rect(rect, me.atlas_size, 0, 0, rect.width, rect.height),
                    w: rect.width as f32,
                    h: rect.height as f32,
                });
            }
        }
        for i in 0..me.liquids.len() {
            if let Some(rect) = atlas.sheet(&format!("LiquidSlope_{i}")) {
                me.liquid_slopes[i] = Some(*rect);
            }
            if let Some(rect) = atlas.sheet(&format!("Liquid_{i}")) {
                me.liquid_parts[i] = Some(*rect);
            }
            if let Some(rect) = atlas.sheet(&format!("Misc/water_{i}")) {
                me.liquids[i] = Some(*rect);
            }
        }
        for i in 0..me.waterfalls.len() {
            if let Some(rect) = atlas.sheet(&format!("Waterfall_{i}")) {
                me.waterfalls[i] = Some(*rect);
            }
        }
        me
    }

    /// UV of one 2x40 raindrop frame (`DrawRain`: `frame.X = type * 4`).
    pub fn rain_uv(&self, kind: u8) -> Option<[f32; 4]> {
        let rect = self.rain.as_ref()?;
        let x = kind as i32 * 4;
        if x + 2 > rect.width as i32 {
            return None;
        }
        Some(uv_rect(rect, self.atlas_size, x, 0, 2, 40))
    }

    /// UV of a coin item sprite (0 copper .. 3 platinum).
    pub fn coin_uv(&self, kind: usize) -> Option<[f32; 4]> {
        let rect = self.coins.get(kind)?.as_ref()?;
        Some(uv_rect(
            rect,
            self.atlas_size,
            0,
            0,
            rect.width,
            rect.height,
        ))
    }

    /// UV of the shuriken projectile/item sprite.
    pub fn shuriken_uv(&self) -> Option<[f32; 4]> {
        let rect = self.shuriken.as_ref()?;
        Some(uv_rect(
            rect,
            self.atlas_size,
            0,
            0,
            rect.width,
            rect.height,
        ))
    }

    /// UV of a representative wall cell, used as the inventory/drop icon.
    pub fn wall_icon_uv(&self, wall: u16) -> Option<[f32; 4]> {
        let tex = self.walls.get(wall as usize)?.as_ref()?;
        // Row 2, column 6 is an interior cell without exposed edges. Frames hold the
        // full 32x32 wall sprite; the icon is its central 16x16.
        let idx = 2 * WALL_COLS + 6;
        let uv = tex.frames.get(idx).and_then(|f| f.as_ref())?.uv;
        let du = (uv[2] - uv[0]) * 0.25;
        let dv = (uv[3] - uv[1]) * 0.25;
        Some([uv[0] + du, uv[1] + dv, uv[2] - du, uv[3] - dv])
    }

    /// UV of an 8x8 `Dust.png` cell.
    pub fn dust_uv(&self, sx: i32, sy: i32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.dust.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a 16x16 source cell inside `Misc/water_{index}`. The sheet is a
    /// vertical strip of 80 px animation blocks, 3 cells wide.
    pub fn liquid_uv(&self, index: usize, sx: i32, sy: i32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.liquids.get(index)?.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a `WaterfallManager` source rectangle inside `Waterfall_{texture}`.
    pub fn waterfall_uv(&self, texture: usize, source: [i32; 4]) -> Option<[f32; 4]> {
        let rect = self.waterfalls.get(texture)?.as_ref()?;
        if source[2] <= 0 || source[3] <= 0 {
            return None;
        }
        Some(uv_rect(
            rect,
            self.atlas_size,
            source[0],
            source[1],
            source[2] as u32,
            source[3] as u32,
        ))
    }

    /// UV of a partial `Liquid_*` strip (16 px tall, 18 px frame stride).
    pub fn liquid_part_uv(
        &self,
        index: usize,
        sx: i32,
        sy: i32,
        w: u32,
        h: u32,
    ) -> Option<[f32; 4]> {
        let rect = self.liquid_parts.get(index)?.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a `LiquidSlope_*` mask: 18 px stride, `slope` in 1..=4.
    pub fn liquid_slope_uv(
        &self,
        index: usize,
        slope: u8,
        sy: i32,
        h: u32,
    ) -> Option<[f32; 4]> {
        let rect = self.liquid_slopes.get(index)?.as_ref()?;
        let sx = 18 * (slope.clamp(1, 4) as i32 - 1);
        Some(uv_rect(rect, self.atlas_size, sx, sy, 16, h))
    }

    /// Average colour of a tile type's plain 16x16 frame (map colours,
    /// `MapHelper.GetTileColor` samples the same pixels).
    pub fn tile_color_avg(&self, atlas: &tr_render::Atlas, kind: u16) -> Option<[f32; 3]> {
        let tt = self.tiles.get(kind as usize)?.as_ref()?;
        let rect = atlas.sheet(&tt.sheet)?;
        sample_avg(atlas, rect, 0, 0, 16, 16)
    }

    /// Average colour of a wall type's inner 16x16 cell.
    pub fn wall_color_avg(&self, atlas: &tr_render::Atlas, wall: u16) -> Option<[f32; 3]> {
        let wt = self.walls.get(wall as usize)?.as_ref()?;
        let rect = atlas.sheet(&wt.sheet)?;
        let x0 = (rect.width.saturating_sub(16)) / 2;
        let y0 = (rect.height.saturating_sub(16)) / 2;
        sample_avg(atlas, rect, x0, y0, 16, 16)
    }

    /// UV of a whole HUD sheet.
    pub fn hud_uv(&self, rect: Option<&tr_render::SheetRect>) -> Option<[f32; 4]> {
        let rect = rect?;
        Some(uv_rect(rect, self.atlas_size, 0, 0, rect.width, rect.height))
    }

    /// UV and native size of the original `Item_<id>` sprite for a slot kind
    /// (`ItemSlot.Draw` uses the item's own texture at inventory scale).
    pub fn item_icon(&self, id: u16) -> Option<([f32; 4], f32, f32)> {
        let rect = self.item_icons.get(&id)?;
        Some((
            uv_rect(rect, self.atlas_size, 0, 0, rect.width, rect.height),
            rect.width as f32,
            rect.height as f32,
        ))
    }

    /// UV of one `UI_BossBar` row (516x58, six vertical frames).
    pub fn boss_bar_uv(&self, row: u32, sx: u32, sy: u32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.ui_boss_bar.as_ref()?;
        Some(uv_rect(
            rect,
            self.atlas_size,
            sx as i32,
            (row * 58 + sy) as i32,
            w,
            h,
        ))
    }

    /// UV rectangle covering a whole raw (non-tiled) sheet, e.g. the sky gradient.
    pub fn sheet_uv(atlas: &tr_render::Atlas, name: &str) -> Option<[f32; 4]> {
        let rect = atlas.sheet(name)?;
        Some([
            rect.x as f32 / atlas.width as f32,
            rect.y as f32 / atlas.height as f32,
            (rect.x + rect.width) as f32 / atlas.width as f32,
            (rect.y + rect.height) as f32 / atlas.height as f32,
        ])
    }

    /// Texture for a tile frame. Non-important tiles almost always sit on the 18 px
    /// grid and hit the dense table; free-form important frames use a small cache.
    pub fn frame_tex(
        &mut self,
        atlas: &tr_render::Atlas,
        kind: u16,
        fx: i16,
        fy: i16,
    ) -> Option<FrameTex> {
        let tt = self.tiles.get(kind as usize)?.as_ref()?;
        if fx >= 0 && fy >= 0 && fx % 18 == 0 && fy % 18 == 0 {
            let cx = (fx / 18) as u32;
            let cy = (fy / 18) as u32;
            if cx < tt.cols && cy < tt.rows {
                return tt.frames.get((cy * tt.cols + cx) as usize).copied();
            }
        }
        if let Some(f) = self.frame_cache.get(&(kind, fx, fy)) {
            return Some(*f);
        }
        let rect = atlas.sheet(&tt.sheet)?;
        let frame = FrameTex {
            uv: uv_of(atlas, rect, &tt.sheet, fx, fy),
            opaque: atlas.frame_is_opaque(rect, fx, fy),
        };
        self.frame_cache.insert((kind, fx, fy), frame);
        Some(frame)
    }

    /// Fallback texture for a tile type (plain interior frame).
    pub fn plain_tex(&self, kind: u16) -> Option<FrameTex> {
        let tt = self.tiles.get(kind as usize)?.as_ref()?;
        Some(FrameTex {
            uv: tt.plain_uv,
            opaque: tt.plain_opaque,
        })
    }

    /// UV of any source rectangle inside a tile type's sheet. Tree trunks are 20x20
    /// sprites on a 22 px grid, so they cannot use the 16x16 block-frame table.
    pub fn rect_uv(
        &self,
        atlas: &tr_render::Atlas,
        kind: u16,
        fx: i16,
        fy: i16,
        w: u32,
        h: u32,
    ) -> Option<FrameTex> {
        let tt = self.tiles.get(kind as usize)?.as_ref()?;
        let rect = atlas.sheet(&tt.sheet)?;
        let uv = uv_rect(rect, self.atlas_size, fx as i32, fy as i32, w, h);
        let opaque = atlas.rect_is_opaque(rect, fx.max(0) as u32, fy.max(0) as u32, w, h);
        Some(FrameTex { uv, opaque })
    }

    /// UV rectangle of a canopy (`top = true`) or branch sprite for a foliage style.
    pub fn foliage_uv(
        &self,
        style: usize,
        top: bool,
        sx: i32,
        sy: i32,
        w: u32,
        h: u32,
    ) -> Option<[f32; 4]> {
        let f = self.foliage.get(style)?.as_ref()?;
        let rect = if top { &f.top } else { &f.branch };
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// Texture for a wall autotile cell.
    ///
    /// Wall sheets use 36 px cells with the 16x16 art inset by 8 px, and cells are
    /// addressed by column/row rather than by pixel offset (`Framing.wallFrameLookup`).
    pub fn wall_tex(&self, wall: u16, col: u8, row: u8) -> Option<FrameTex> {
        let wt = self.walls.get(wall as usize)?.as_ref()?;
        if col as usize >= WALL_COLS || row as usize >= WALL_ROWS {
            return None;
        }
        wt.frames[row as usize * WALL_COLS + col as usize]
    }
}

fn uv_of(
    atlas: &tr_render::Atlas,
    rect: &tr_render::SheetRect,
    name: &str,
    fx: i16,
    fy: i16,
) -> [f32; 4] {
    uv_of_size(atlas, rect, name, fx, fy, 16, 16)
}

fn uv_of_size(
    atlas: &tr_render::Atlas,
    rect: &tr_render::SheetRect,
    name: &str,
    fx: i16,
    fy: i16,
    w: u32,
    h: u32,
) -> [f32; 4] {
    let x0 = rect.x as f32 + fx.max(0) as f32;
    let y0 = rect.y as f32 + fy.max(0) as f32;
    if fx.max(0) as u32 + w > rect.width || fy.max(0) as u32 + h > rect.height {
        log::warn!(
            "sheet {name}: frame ({fx},{fy}) outside rect {}x{}",
            rect.width,
            rect.height
        );
    }
    [
        x0 / atlas.width as f32,
        y0 / atlas.height as f32,
        (x0 + w as f32) / atlas.width as f32,
        (y0 + h as f32) / atlas.height as f32,
    ]
}

/// Alpha-weighted average of an atlas region, used for map colours.
fn sample_avg(
    atlas: &tr_render::Atlas,
    rect: &tr_render::SheetRect,
    ox: u32,
    oy: u32,
    w: u32,
    h: u32,
) -> Option<[f32; 3]> {
    let mut acc = [0f32; 3];
    let mut n = 0u32;
    for y in oy..oy + h {
        for x in ox..ox + w {
            let px = rect.x + x;
            let py = rect.y + y;
            if px >= atlas.width || py >= atlas.height {
                continue;
            }
            let i = ((py * atlas.width + px) * 4) as usize;
            if atlas.pixels[i + 3] < 90 {
                continue;
            }
            acc[0] += atlas.pixels[i] as f32 / 255.0;
            acc[1] += atlas.pixels[i + 1] as f32 / 255.0;
            acc[2] += atlas.pixels[i + 2] as f32 / 255.0;
            n += 1;
        }
    }
    (n > 0).then(|| [acc[0] / n as f32, acc[1] / n as f32, acc[2] / n as f32])
}

fn uv_rect(
    rect: &tr_render::SheetRect,
    atlas_size: (u32, u32),
    sx: i32,
    sy: i32,
    w: u32,
    h: u32,
) -> [f32; 4] {
    let (aw, ah) = (atlas_size.0 as f32, atlas_size.1 as f32);
    let x0 = rect.x as f32 + sx.max(0) as f32;
    let y0 = rect.y as f32 + sy.max(0) as f32;
    [x0 / aw, y0 / ah, (x0 + w as f32) / aw, (y0 + h as f32) / ah]
}
