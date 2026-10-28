//! Terraria asset loading: real game textures extracted from `.xnb` to PNG.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Context, Result};
use tr_core::World;
use tr_render::{AtlasImage, SheetLayout};

pub fn usage(world: &World) -> (BTreeSet<u16>, BTreeSet<u16>) {
    let mut tiles = BTreeSet::new();
    let mut walls = BTreeSet::new();
    for t in &world.tiles {
        if t.active() && t.kind != 0 {
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

    for &kind in tiles {
        let name = image_name_for_tile(kind);
        match load_png(dir, &name, SheetLayout::Tile) {
            Ok(img) => images.push(img),
            Err(_) => missing.push(name),
        }
    }
    for &wall in walls {
        let name = image_name_for_wall(wall);
        match load_png(dir, &name, SheetLayout::Wall) {
            Ok(img) => images.push(img),
            Err(_) => missing.push(name),
        }
    }

    // Fallback sheet so unknown ids still render something instead of vanishing.
    if !images.iter().any(|i| i.name == "Tiles_0") {
        if let Ok(img) = load_png(dir, "Tiles_0", SheetLayout::Tile) {
            images.push(img);
        }
    }

    // Whole-image sheets used by the sky/background pass.
    for name in ["Background_0"] {
        match load_png(dir, name, SheetLayout::Raw) {
            Ok(img) => images.push(img),
            Err(_) => log::warn!("missing background texture {name}"),
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

/// Precomputed texture information for every type present in the world.
#[derive(Default)]
pub struct Lookups {
    /// Dense per-type tables: one slot per `TileID`, `None` if the sheet is absent.
    pub tiles: Vec<Option<TypeTex>>,
    pub walls: Vec<Option<WallTex>>,
    /// Frames that do not sit on the 18 px grid (furniture and other important
    /// tiles). Populated lazily; bounded by the number of distinct visible frames.
    pub frame_cache: HashMap<(u16, i16, i16), FrameTex>,
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
            frame_cache: HashMap::new(),
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
        me
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
    let aw = atlas.width as f32;
    let ah = atlas.height as f32;
    let x0 = rect.x as f32 + fx.max(0) as f32;
    let y0 = rect.y as f32 + fy.max(0) as f32;
    if fx.max(0) as u32 + 16 > rect.width || fy.max(0) as u32 + 16 > rect.height {
        log::warn!(
            "sheet {name}: frame ({fx},{fy}) outside rect {}x{}",
            rect.width,
            rect.height
        );
    }
    [x0 / aw, y0 / ah, (x0 + 16.0) / aw, (y0 + 16.0) / ah]
}
