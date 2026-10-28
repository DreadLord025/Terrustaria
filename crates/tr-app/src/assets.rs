//! Terraria asset loading: real game textures extracted from `.xnb` to PNG.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

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
        "Background_7",
        "Sun",
        "Moon_0",
        "Star_0",
        "Star_1",
        "Star_2",
        "Star_3",
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

    if !missing.is_empty() {
        log::warn!(
            "{} world sheets have no matching texture (first 10: {:?})",
            missing.len(),
            &missing[..missing.len().min(10)]
        );
    }

    Ok(images)
}

pub fn load_png(dir: &Path, name: &str, layout: SheetLayout) -> Result<AtlasImage> {
    let path = dir.join(format!("{name}.png"));
    let img = image::open(&path)
        .with_context(|| format!("loading {}", path.display()))?
        .to_rgba8();
    Ok(AtlasImage {
        name: name.to_string(),
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
        layout,
    })
}

fn load_misc_png(dir: &Path, name: &str) -> Result<AtlasImage> {
    let path = dir.join("Misc").join(format!("{name}.png"));
    let img = image::open(&path)
        .with_context(|| format!("loading {}", path.display()))?
        .to_rgba8();
    Ok(AtlasImage {
        name: name.to_string(),
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
        layout: SheetLayout::Raw,
    })
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
    /// `Framing.wallFrameLookup` cells: `row * 13 + col` (36 px cells, 8 px inset).
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
    ) -> Self {
        let mut me = Self {
            tiles: vec![None; tr_core::tiledata::TILE_COUNT],
            walls: vec![None; tr_core::tiledata::WALL_COUNT],
            foliage: vec![None; 32],
            clouds: vec![None; 42],
            liquids: vec![None; 15],
            frame_cache: HashMap::new(),
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
                    let fx = col as i16 * 36 + 8;
                    let fy = row as i16 * 36 + 8;
                    if fx as u32 + 16 > rect.width || fy as u32 + 16 > rect.height {
                        frames.push(None);
                    } else {
                        frames.push(Some(FrameTex {
                            uv: uv_of(atlas, rect, &name, fx, fy),
                            opaque: atlas.frame_is_opaque(rect, fx, fy),
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
            if let Some(rect) = atlas.sheet(&format!("water_{i}")) {
                me.liquids[i] = Some(*rect);
            }
        }
        me
    }

    /// UV of a 16x16 source cell inside `Misc/water_{index}`. The sheet is a
    /// vertical strip of 80 px animation blocks, 3 cells wide.
    pub fn liquid_uv(&self, index: usize, sx: i32, sy: i32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.liquids.get(index)?.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
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
    let x0 = rect.x as f32 + fx.max(0) as f32;
    let y0 = rect.y as f32 + fy.max(0) as f32;
    if fx.max(0) as u32 + 16 > rect.width || fy.max(0) as u32 + 16 > rect.height {
        log::warn!(
            "sheet {name}: frame ({fx},{fy}) outside rect {}x{}",
            rect.width,
            rect.height
        );
    }
    [
        x0 / atlas.width as f32,
        y0 / atlas.height as f32,
        (x0 + 16.0) / atlas.width as f32,
        (y0 + 16.0) / atlas.height as f32,
    ]
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
