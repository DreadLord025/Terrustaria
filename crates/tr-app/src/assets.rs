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

    if !missing.is_empty() {
        log::warn!("{} world sheets have no matching texture (first 10: {:?})", missing.len(), &missing[..missing.len().min(10)]);
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

/// UV lookup for one tile/wall type.
#[derive(Clone, Debug)]
pub struct TypeTex {
    pub sheet: String,
    /// UV rectangle of the "plain" 16x16 frame.
    pub plain_uv: [f32; 4],
}

/// Precomputed texture information for every type present in the world.
#[derive(Default)]
pub struct Lookups {
    pub tiles: HashMap<u16, TypeTex>,
    pub walls: HashMap<u16, TypeTex>,
    /// Cache for important-tile frames: (type, frameX, frameY) -> UV.
    pub frame_cache: HashMap<(u16, i16, i16), [f32; 4]>,
}

impl Lookups {
    pub fn build(
        atlas: &tr_render::Atlas,
        tile_names: impl Iterator<Item = (u16, String)>,
        wall_names: impl Iterator<Item = (u16, String)>,
    ) -> Self {
        let mut me = Self::default();
        for (kind, name) in tile_names {
            if let Some(rect) = atlas.sheet(&name) {
                let plain = rect.plain_frame;
                me.tiles.insert(
                    kind,
                    TypeTex {
                        sheet: name.clone(),
                        plain_uv: uv_of(atlas, rect, &name, plain.0, plain.1),
                    },
                );
            }
        }
        for (wall, name) in wall_names {
            if let Some(rect) = atlas.sheet(&name) {
                let plain = rect.plain_frame;
                me.walls.insert(
                    wall,
                    TypeTex {
                        sheet: name.clone(),
                        plain_uv: uv_of(atlas, rect, &name, plain.0, plain.1),
                    },
                );
            }
        }
        me
    }

    /// UV for an arbitrary frame of a tile sheet (important tiles store pixels).
    pub fn frame_uv(
        &mut self,
        atlas: &tr_render::Atlas,
        kind: u16,
        fx: i16,
        fy: i16,
    ) -> Option<[f32; 4]> {
        if let Some(uv) = self.frame_cache.get(&(kind, fx, fy)) {
            return Some(*uv);
        }
        let tt = self.tiles.get(&kind)?;
        let rect = atlas.sheet(&tt.sheet)?;
        let uv = uv_of(atlas, rect, &tt.sheet, fx, fy);
        self.frame_cache.insert((kind, fx, fy), uv);
        Some(uv)
    }
}

fn uv_of(atlas: &tr_render::Atlas, rect: &tr_render::SheetRect, name: &str, fx: i16, fy: i16) -> [f32; 4] {
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
