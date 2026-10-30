//! Minimap and full map (`Main.DrawMap`, `Terraria.Map.WorldMap`): reveals
//! `MapTile`s from the light map (`WorldMap.UpdateLighting`,
//! `LightingEngine.ExportToMiniMap`). Drawing the tiles is Task 6; for now the
//! panels are placeholders.

use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::map::{MapTile, reveal_rect};
use tr_core::{TILE_PX, World};
use tr_render::SpriteRenderer;

use crate::player::Player;

pub struct WorldMap {
    /// Row-major `y * width + x`, matching `World::index`.
    pub tiles: Vec<MapTile>,
    pub open: bool,
    timer: f32,
    /// Tiles revealed/changed since the last `take_dirty` (Task 6 uploads these).
    dirty: Vec<(i32, i32)>,
}

impl WorldMap {
    pub fn new(world: &World) -> Self {
        Self {
            tiles: vec![MapTile::default(); (world.width as usize) * (world.height as usize)],
            open: false,
            timer: 1.0,
            dirty: Vec::new(),
        }
    }

    /// Reveals a `75x47` window around the player 4x/second from `light`.
    pub fn update(&mut self, world: &World, player: Option<&Player>, light: &LightMap, dt: f32) {
        let Some(p) = player else { return };
        self.timer += dt;
        if self.timer < 0.25 {
            return;
        }
        self.timer = 0.0;
        let px = (p.x / TILE_PX) as i32;
        let py = (p.y / TILE_PX) as i32;
        let changed = reveal_rect(
            world,
            &mut self.tiles,
            light,
            px - 37,
            py - 23,
            px + 37,
            py + 23,
        );
        self.dirty.extend(changed);
    }

    /// Tiles changed since the previous call (Task 6 uploads these).
    // Wired into the texture upload by Task 6; unused until then.
    #[allow(dead_code)]
    pub fn take_dirty(&mut self) -> Vec<(i32, i32)> {
        std::mem::take(&mut self.dirty)
    }

    /// Placeholder panel + border; Task 6 replaces the body with the tile texture.
    pub fn draw_minimap(&self, r: &mut SpriteRenderer, screen: Vec2) {
        let size = Vec2::new(90.0 * 2.0, 60.0 * 2.0);
        let origin = Vec2::new(screen.x - size.x - 20.0, 92.0);
        r.push_ui_rect(
            [origin.x - 2.0, origin.y - 2.0],
            [size.x + 4.0, size.y + 4.0],
            [0.25, 0.3, 0.4, 0.9],
        );
        r.push_ui_rect(
            [origin.x, origin.y],
            [size.x, size.y],
            [0.05, 0.06, 0.1, 0.9],
        );
    }

    /// Placeholder panel + border; Task 6 replaces the body with the tile texture.
    pub fn draw_full(&self, r: &mut SpriteRenderer, _world: &World, screen: Vec2) {
        r.push_ui_rect([0.0, 0.0], [screen.x, screen.y], [0.02, 0.03, 0.05, 0.93]);
        let size = Vec2::new((screen.x - 80.0).max(0.0), (screen.y - 120.0).max(0.0));
        let origin = Vec2::new((screen.x - size.x) * 0.5, (screen.y - size.y) * 0.5);
        r.push_ui_rect(
            [origin.x - 2.0, origin.y - 2.0],
            [size.x + 4.0, size.y + 4.0],
            [0.25, 0.3, 0.4, 0.9],
        );
        r.push_ui_rect(
            [origin.x, origin.y],
            [size.x, size.y],
            [0.05, 0.06, 0.1, 0.95],
        );
    }
}

// ---------------------------------------------------------------------------
// `.map` persistence (`MapHelper.InternalSaveMap` / `LoadMapVersion2`). The
// byte layout lives in `tr-core`; this crate owns the raw-deflate wrapping and
// the file IO, since `flate2` is a `tr-app`-only dependency.
// ---------------------------------------------------------------------------

/// Writes header + option tables + deflated body (`MapHelper.InternalSaveMap`).
// Wired into the save/autosave path by Task 9; unused until then.
#[allow(dead_code)]
pub fn save_map_file(
    world: &World,
    map: &[MapTile],
    path: &std::path::Path,
) -> std::io::Result<()> {
    use std::io::Write;
    let header = tr_core::map::write_map_header(world);
    let tables = tr_core::map::write_option_tables();
    let body = tr_core::map::encode_tiles(map, world.width as usize, world.height as usize);
    // The C# `DeflateStream` is raw deflate (no zlib/gzip wrapper).
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&body)?;
    let deflated = encoder.finish()?;

    let mut out = Vec::with_capacity(header.len() + tables.len() + deflated.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(&tables);
    out.extend_from_slice(&deflated);
    std::fs::write(path, out)
}

/// Loads a `.map` sidecar into `map`.
///
/// `Ok(false)` when the file is absent, unreadable, truncated, has a foreign
/// version, or its world seed/size do not match `world` (`MapHelper.cs:3763`).
// Wired into the load path by Task 9; unused until then.
#[allow(dead_code)]
pub fn load_map_file(
    world: &World,
    map: &mut [MapTile],
    path: &std::path::Path,
) -> std::io::Result<bool> {
    use std::io::Read;
    let Ok(bytes) = std::fs::read(path) else {
        return Ok(false);
    };
    let Some(prefix) = tr_core::map::read_map_prefix(&bytes)? else {
        return Ok(false);
    };
    let header = &prefix.header;
    if header.seed != world.seed as i32
        || header.width != world.width
        || header.height != world.height
    {
        return Ok(false);
    }
    let Some(deflated) = bytes.get(prefix.body_offset..) else {
        return Ok(false);
    };
    let mut decoder = flate2::read::DeflateDecoder::new(deflated);
    let mut body = Vec::new();
    if decoder.read_to_end(&mut body).is_err() {
        return Ok(false);
    }
    let tiles = match tr_core::map::decode_tiles(
        &body,
        header,
        &prefix.tile_options,
        &prefix.wall_options,
        world.width as usize,
        world.height as usize,
        world.world_surface,
        world.rock_layer,
    ) {
        Ok(tiles) => tiles,
        Err(_) => return Ok(false),
    };
    if map.len() != tiles.len() {
        return Ok(false);
    }
    map.copy_from_slice(&tiles);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::map::{color_table, read_map_prefix, MapTile};

    fn test_world() -> World {
        let mut w = World::new("Terrustaria".into(), 40, 40);
        w.seed = 0x1234_5678_9abc_def0;
        w.world_surface = 10.0;
        w.rock_layer = 18.0;
        w
    }

    fn put(map: &mut [MapTile], width: usize, x: usize, y: usize, tile: MapTile) {
        map[y * width + x] = tile;
    }

    /// The same 40x40 map as the `tr-core` round-trip test.
    fn test_tiles(width: usize, height: usize) -> Vec<MapTile> {
        let t = color_table();
        let mut map = vec![MapTile::default(); width * height];
        for x in 4..=6 {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 2, 255, 0));
        }
        for (x, light) in [(7usize, 30u8), (8, 60), (9, 90)] {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 3, light, 0));
        }
        put(&mut map, width, 10, 0, MapTile::create(t.tile_position + 4, 100, 7));
        put(&mut map, width, 11, 0, MapTile::create(t.tile_position + 300, 255, 0));
        put(&mut map, width, 12, 0, MapTile::create(t.wall_position, 200, 0));
        put(&mut map, width, 13, 0, MapTile::create(t.liquid_position, 200, 0));
        put(&mut map, width, 14, 0, MapTile::create(t.liquid_position + 3, 200, 0));
        map
    }

    fn unique_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("tr-map-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn map_round_trips_through_a_file() {
        let world = test_world();
        let original = test_tiles(40, 40);
        let dir = unique_dir("roundtrip");
        std::fs::create_dir_all(&dir).unwrap();
        let path_a = dir.join("a.map");
        let path_b = dir.join("b.map");
        save_map_file(&world, &original, &path_a).unwrap();

        let mut loaded = vec![MapTile::default(); original.len()];
        assert!(load_map_file(&world, &mut loaded, &path_a).unwrap());
        assert_eq!(loaded, original);

        // Re-saving the loaded map must be byte-identical (deterministic header +
        // deflate).
        save_map_file(&world, &loaded, &path_b).unwrap();
        assert_eq!(
            std::fs::read(&path_a).unwrap(),
            std::fs::read(&path_b).unwrap()
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn truncated_file_is_rejected() {
        let world = test_world();
        let original = test_tiles(40, 40);
        let dir = unique_dir("truncated");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cut.map");
        save_map_file(&world, &original, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        std::fs::write(&path, &bytes[..10]).unwrap();

        let mut loaded = vec![MapTile::default(); original.len()];
        assert!(!load_map_file(&world, &mut loaded, &path).unwrap());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn mismatched_world_is_rejected() {
        let world = test_world();
        let original = test_tiles(40, 40);
        let dir = unique_dir("mismatch");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("other.map");
        save_map_file(&world, &original, &path).unwrap();

        let mut other = World::new("Other".into(), 40, 40);
        other.seed = world.seed ^ 0xff;
        let mut loaded = vec![MapTile::default(); original.len()];
        assert!(!load_map_file(&other, &mut loaded, &path).unwrap());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[ignore]
    fn loads_a_real_vanilla_map() {
        let Ok(path) = std::env::var("TR_VANILLA_MAP") else {
            return;
        };
        let bytes = std::fs::read(&path).expect("read map");
        let prefix = read_map_prefix(&bytes)
            .expect("no error")
            .expect("vanilla prefix must parse");
        let mut world = World::new(
            prefix.world_name.clone(),
            prefix.header.width,
            prefix.header.height,
        );
        world.seed = prefix.header.seed as u32 as u64;
        // Only used for the sky/dirt/rock bands; irrelevant to the revealed count.
        world.world_surface = 300.0;
        world.rock_layer = 600.0;
        let mut map = vec![MapTile::default(); (world.width * world.height) as usize];
        assert!(load_map_file(&world, &mut map, std::path::Path::new(&path)).unwrap());
        assert!(
            map.iter().filter(|t| t.light > 0).count() > 0,
            "vanilla map decoded to no revealed tiles"
        );
    }

    // ---- Task 5: reveal from the light map ----

    use crate::player::Player;
    use tr_core::lighting::{LightMap, LightParams};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The `ground_world` from `player.rs`: a 64x128 world with a solid band
    /// `y in 60..128`, spawn at `(10, 40)`.
    fn ground_world() -> World {
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
            for y in 60..128 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 10;
        w.spawn_y = 40;
        w
    }

    #[test]
    fn reveal_window_marks_tiles_around_the_player() {
        let world = ground_world();
        let p = Player::spawn(&world);
        let px = (p.x / TILE_PX) as i32;
        let py = (p.y / TILE_PX) as i32;
        let light = LightMap::compute(
            &world,
            px - 40,
            py - 30,
            px + 40,
            py + 30,
            &LightParams::default(),
        );
        let mut m = WorldMap::new(&world);
        m.update(&world, Some(&p), &light, 0.3);
        assert!(m.tiles[world.index(px, py)].light > 0);
    }

    #[test]
    fn update_without_a_player_is_a_no_op() {
        let world = ground_world();
        let light = LightMap::compute(&world, 0, 0, 63, 127, &LightParams::default());
        let mut m = WorldMap::new(&world);
        m.update(&world, None, &light, 1.0);
        assert!(m.tiles.iter().all(|t| t.light == 0));
    }
}
