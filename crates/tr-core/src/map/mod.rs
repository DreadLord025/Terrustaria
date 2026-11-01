//! The world map: `Terraria.Map.WorldMap` + the `MapHelper` colour state.
//!
//! - [`colors`] — the packed `colorLookup` table and the biome gradients
//!   (`MapHelper.Initialize`).
//! - [`option`] — tile/wall/background colour selection (`GetTileType`,
//!   `GetTileBaseOption`, `GetWallType`, `GetBackgroundType`).
//! - [`file`] — `.map` v326 persistence (`InternalSaveMap`, `LoadMapVersion2`).
//! - this module — the `MapTile` bitfield, paint application (`MapColor`),
//!   `CreateMapTile` and the reveal/lighting-update entry points.

mod colors;
mod file;
mod option;

pub use colors::{
    color_table, dirt_gradient, rock_gradient, sky_gradient, LIQUID_COLORS, MapColorTable,
};
pub use file::{
    decode_tiles, encode_tiles, read_map_prefix, write_map_header, write_option_tables, MapHeader,
    MapPrefix, MAP_FORMAT_VERSION,
};
pub use option::get_tile_base_option;
use option::{get_background_type, get_tile_type, get_wall_type};

use crate::lighting::LightMap;
use crate::tile::LiquidKind;
use crate::tile_light::paint_color;
use crate::world::World;

/// One world-map pixel (`Terraria.Map.MapTile`).
///
/// `extra` mirrors the C# `_extraData` byte: bit 7 `IsChanged`, bit 6
/// `UpdateQueued`, bits 0-4 `Color` (`MapTile.cs:9-59`).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct MapTile {
    /// Packed `colorLookup` index: `baseType + baseOption`.
    pub ty: u16,
    /// Map light 0..=255; `0` means the tile is not revealed.
    pub light: u8,
    pub extra: u8,
}

impl MapTile {
    const CHANGED: u8 = 0x80;
    const QUEUED: u8 = 0x40;
    const COLOR_MASK: u8 = 0x1f;

    /// `MapTile.Color` (`MapTile.cs:49-59`).
    #[inline]
    pub fn color(&self) -> u8 {
        self.extra & Self::COLOR_MASK
    }

    #[inline]
    pub fn set_color(&mut self, c: u8) {
        self.extra = (self.extra & !Self::COLOR_MASK) | (c & Self::COLOR_MASK);
    }

    /// `MapTile.IsChanged` (`MapTile.cs:11-28`).
    #[inline]
    pub fn is_changed(&self) -> bool {
        self.extra & Self::CHANGED != 0
    }

    #[inline]
    pub fn set_changed(&mut self, v: bool) {
        if v {
            self.extra |= Self::CHANGED;
        } else {
            self.extra &= !Self::CHANGED;
        }
    }

    /// `MapTile.UpdateQueued` (`MapTile.cs:30-47`).
    #[inline]
    pub fn update_queued(&self) -> bool {
        self.extra & Self::QUEUED != 0
    }

    #[inline]
    pub fn set_update_queued(&mut self, v: bool) {
        if v {
            self.extra |= Self::QUEUED;
        } else {
            self.extra &= !Self::QUEUED;
        }
    }

    /// `MapTile.Create` (`MapTile.cs:98-101`): the changed bit is always set.
    #[inline]
    pub fn create(ty: u16, light: u8, color: u8) -> Self {
        Self {
            ty,
            light,
            extra: color | Self::CHANGED,
        }
    }

    /// `MapTile.Clear` (`MapTile.cs:86-91`).
    #[inline]
    pub fn clear(&mut self) {
        self.ty = 0;
        self.light = 0;
        self.extra = 0;
    }

    /// `MapTile.WithLight` (`MapTile.cs:93-96`): sets the changed bit.
    #[inline]
    pub fn with_light(&self, light: u8) -> Self {
        Self {
            ty: self.ty,
            light,
            extra: self.extra | Self::CHANGED,
        }
    }

    /// `MapTile.Equals` (`MapTile.cs:68-75`).
    #[inline]
    pub fn equals_all(&self, o: &Self) -> bool {
        self.light == o.light && self.ty == o.ty && self.color() == o.color()
    }

    /// `MapTile.EqualsWithoutLight` (`MapTile.cs:77-84`).
    #[inline]
    pub fn equals_without_light(&self, o: &Self) -> bool {
        self.ty == o.ty && self.color() == o.color()
    }
}

/// `WorldMap.IsRevealed` (`WorldMap.cs:44-47`): `Light > 0`.
#[inline]
pub fn is_revealed(t: MapTile) -> bool {
    t.light > 0
}

/// `Tile.liquidType()` -> the `LIQUID_COLORS` slot (`MapHelper.cs:1996-1998`).
pub fn liquid_index(kind: LiquidKind) -> u16 {
    match kind {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

/// `MapHelper.IsBackground` (`MapHelper.cs:1895-1898`).
#[inline]
pub fn is_background(map_type: u16) -> bool {
    map_type >= color_table().sky_position
}

/// `MapHelper.MapColor` (`MapHelper.cs:1823-1874`).
///
/// `paintColor(c)` is reused from [`crate::tile_light::paint_color`] (a faithful
/// `WorldGen.paintColor` port); its normalised floats are converted back to the
/// original bytes (divergence ruling 7).
fn map_color(ty: u16, old: &mut [u8; 3], color_type: u8) {
    let pc = paint_color(color_type);
    let color = [
        (pc[0] * 255.0).round() as u8,
        (pc[1] * 255.0).round() as u8,
        (pc[2] * 255.0).round() as u8,
    ];
    // Sort so `num` is the largest channel (MapHelper.cs:1826-1840).
    let mut num = old[0] as f32 / 255.0;
    let mut num2 = old[1] as f32 / 255.0;
    let mut num3 = old[2] as f32 / 255.0;
    if num2 > num {
        std::mem::swap(&mut num, &mut num2);
    }
    if num3 > num {
        std::mem::swap(&mut num, &mut num3);
    }
    let table = color_table();
    match color_type {
        29 => {
            // Shadow paint (MapHelper.cs:1843-1850).
            let num7 = num3 * 0.3;
            old[0] = (color[0] as f32 * num7) as u8;
            old[1] = (color[1] as f32 * num7) as u8;
            old[2] = (color[2] as f32 * num7) as u8;
        }
        30 => {
            // Negative paint inverts, or halves a wall (MapHelper.cs:1851-1864).
            if ty >= table.wall_range_start && ty <= table.wall_range_end {
                old[0] = ((255 - old[0]) as f32 * 0.5) as u8;
                old[1] = ((255 - old[1]) as f32 * 0.5) as u8;
                old[2] = ((255 - old[2]) as f32 * 0.5) as u8;
            } else {
                old[0] = 255 - old[0];
                old[1] = 255 - old[1];
                old[2] = 255 - old[2];
            }
        }
        _ => {
            // Default paint multiplies by the max channel (MapHelper.cs:1865-1872).
            old[0] = (color[0] as f32 * num) as u8;
            old[1] = (color[1] as f32 * num) as u8;
            old[2] = (color[2] as f32 * num) as u8;
        }
    }
}

/// `MapHelper.GetMapTileXnaColor` (`MapHelper.cs:1876-1893`).
///
/// `i`/`j` are unused in the C# too; kept for signature parity (ruling 10).
pub fn get_map_tile_color(t: MapTile, _i: i32, _j: i32) -> [u8; 3] {
    let mut old = color_table().color(t.ty);
    let color = t.color();
    if color > 0 {
        map_color(t.ty, &mut old, color);
    }
    if t.light == 255 {
        return old;
    }
    let num = t.light as f32 / 255.0;
    old[0] = (old[0] as f32 * num) as u8;
    old[1] = (old[1] as f32 * num) as u8;
    old[2] = (old[2] as f32 * num) as u8;
    old
}

/// `MapHelper.CreateMapTile` (`MapHelper.cs:1900-1923`).
pub fn create_map_tile(
    world: &World,
    map: &[MapTile],
    i: i32,
    j: i32,
    light: u8,
    background_override: u16,
) -> MapTile {
    let Some(tile) = world.tile(i, j) else {
        return MapTile::default();
    };
    let mut new_color = 0i32;
    let mut new_light = light as i32;
    let mut base_type = 0i32;
    let mut base_option = 0i32;
    get_tile_type(
        i,
        j,
        tile,
        &mut new_color,
        &mut new_light,
        &mut base_type,
        &mut base_option,
        world,
    );
    if base_type == 0 {
        get_wall_type(
            i,
            j,
            tile,
            &mut new_color,
            &mut new_light,
            &mut base_type,
            &mut base_option,
        );
    }
    if base_type == 0 {
        new_color = 0;
        new_light = light as i32;
        base_type = if background_override == 0 {
            get_background_type(world, map, i, j, &mut new_light) as i32
        } else {
            background_override as i32
        };
    }
    MapTile::create(
        (base_type + base_option) as u16,
        new_light as u8,
        new_color as u8,
    )
}

/// `WorldMap.UpdateLighting` (`WorldMap.cs:49-63`): keep the brighter of the old
/// and new light, rebuild the tile with `CreateMapTile`, and return true when it
/// changed (`MapTile.Equals`, which ignores the changed bit). Out of bounds →
/// false.
pub fn update_map_lighting(
    world: &World,
    map: &mut [MapTile],
    x: i32,
    y: i32,
    light: u8,
) -> bool {
    if !world.in_bounds(x, y) {
        return false;
    }
    let idx = world.index(x, y);
    let other = map[idx];
    if light == 0 && other.light == 0 {
        return false; // WorldMap.cs:52-55
    }
    let new = create_map_tile(world, map, x, y, other.light.max(light), 0); // :56
    if new.equals_all(&other) {
        return false; // :57-60
    }
    map[idx] = new;
    true
}

/// `LightingEngine.ExportToMiniMap` (`LightingEngine.cs:246-270`): sample the
/// light map per tile with `light = min(255, max(r, g, b) * 255)` and reveal only
/// the tiles whose pixels changed (Task 6 uploads exactly these). Tiles outside
/// the computed light rect read `[0, 0, 0]` and stay dark. The rect is inclusive.
pub fn reveal_rect(
    world: &World,
    map: &mut [MapTile],
    light: &LightMap,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> Vec<(i32, i32)> {
    let mut changed = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let [r, g, b] = light.get(x, y);
            // LightingEngine.cs:264: `(byte)Math.Min(255, (int)(max * 255f))`.
            let l = ((r.max(g).max(b) * 255.0) as i32).clamp(0, 255) as u8;
            if update_map_lighting(world, map, x, y, l) {
                changed.push((x, y));
            }
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Task 3: MapTile + CreateMapTile + colour ----

    use crate::tile::{Tile, FLAG_ACTIVE};
    use crate::world::World;

    fn blank_map(world: &World) -> Vec<MapTile> {
        vec![MapTile::default(); (world.width * world.height) as usize]
    }

    fn active_stone(world: &mut World, x: i32, y: i32) -> &mut Tile {
        let t = world.tile_mut(x, y).unwrap();
        t.kind = 1; // TileID.Stone, map colour [128, 128, 128]
        t.flags |= FLAG_ACTIVE;
        t
    }

    #[test]
    fn stone_tile_maps_to_its_lookup() {
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.ty, color_table().tile_lookup[1]);
        assert_ne!(mt.ty, 0);
    }

    #[test]
    fn unlit_tile_is_not_revealed() {
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 0, 0);
        assert_eq!(mt.light, 0);
        assert!(!is_revealed(mt));
    }

    #[test]
    fn wall_only_tile_uses_a_wall_colour() {
        let mut world = World::new("test".into(), 20, 20);
        world.tile_mut(5, 5).unwrap().wall = 1; // WallID.Stone
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        let table = color_table();
        assert!(
            mt.ty >= table.wall_range_start && mt.ty <= table.wall_range_end,
            "ty {} not in wall range {}..={}",
            mt.ty,
            table.wall_range_start,
            table.wall_range_end
        );
    }

    #[test]
    fn water_maps_to_a_liquid_colour() {
        let mut world = World::new("test".into(), 20, 20);
        world.tile_mut(5, 5).unwrap().liquid = 200;
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.ty, color_table().liquid_position);
    }

    #[test]
    fn surface_air_uses_the_sky_gradient() {
        let world = World::new("test".into(), 20, 1000); // world_surface == 250
        let map = blank_map(&world);
        let j = 100;
        let mt = create_map_tile(&world, &map, 5, j, 0, 0);
        // CalcSkyGradient (MapHelper.cs:2078): (byte)(255 * (j / worldSurface)).
        let expected =
            color_table().sky_position + (255.0 * (j as f64 / world.world_surface)) as u8 as u16;
        assert_eq!(mt.ty, expected);
        // The surface branch sets newLight = 255 (MapHelper.cs:2036).
        assert_eq!(mt.light, 255);
    }

    #[test]
    fn deep_air_uses_dirt_then_rock_then_hell() {
        let world = World::new("test".into(), 20, 2000); // surface 500, rock 900
        let map = blank_map(&world);
        let table = color_table();

        let dirt = create_map_tile(&world, &map, 5, 600, 255, 0);
        assert!(
            dirt.ty >= table.dirt_position && dirt.ty < table.dirt_position + 256,
            "dirt ty {}",
            dirt.ty
        );

        let rock = create_map_tile(&world, &map, 5, 1000, 255, 0);
        assert!(
            rock.ty >= table.rock_position && rock.ty < table.rock_position + 256,
            "rock ty {}",
            rock.ty
        );

        // UnderworldLayer == height - 200 (Main.cs:2790), so j == 1900 >= 1800.
        let hell = create_map_tile(&world, &map, 5, 1900, 255, 0);
        assert_eq!(hell.ty, table.hell_position);
    }

    #[test]
    fn negative_paint_inverts_the_colour() {
        // Grey stone [128,128,128] with paint 30 → MapHelper.cs:1851-1863 inverts.
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5).set_paint(30);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.color(), 30);
        assert_eq!(get_map_tile_color(mt, 5, 5), [127, 127, 127]);
    }

    #[test]
    fn paint_multiplies_by_max_channel() {
        // Paint 1 is red; only the red channel survives (MapHelper.cs:1865-1872).
        let mut world = World::new("test".into(), 20, 20);
        active_stone(&mut world, 5, 5).set_paint(1);
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.color(), 1);
        assert_eq!(get_map_tile_color(mt, 5, 5), [128, 0, 0]);
    }

    #[test]
    fn painted_wall_half_inverts() {
        // A wall-range type with paint 30 → (255 - c) / 2 (MapHelper.cs:1852-1856).
        let table = color_table();
        let ty = table.wall_lookup[1]; // WallID.Stone == [52, 52, 52]
        assert_eq!(table.color(ty), [52, 52, 52]);
        let mt = MapTile::create(ty, 255, 30);
        assert_eq!(get_map_tile_color(mt, 5, 5), [101, 101, 101]);
    }

    #[test]
    fn fully_lit_returns_the_raw_colour() {
        let ty = color_table().tile_lookup[1];
        let mt = MapTile::create(ty, 255, 0);
        assert_eq!(get_map_tile_color(mt, 0, 0), color_table().color(ty));
        assert_eq!(get_map_tile_color(mt, 0, 0), [128, 128, 128]);
    }

    #[test]
    fn changed_bit_is_set_by_create_and_with_light() {
        let created = MapTile::create(1, 2, 3);
        assert!(created.is_changed());
        assert_eq!(created.color(), 3);
        assert!(MapTile::default().with_light(7).is_changed());
    }

    // ---- Task 5: reveal the map from the light map ----

    use crate::lighting::{LightMap, LightParams};

    /// A 20x400 world whose whole column is below the surface (so
    /// `create_map_tile` never forces light 255, `MapHelper.cs:1064`) but still
    /// above the underworld (`height - 200`).
    fn reveal_world() -> World {
        let mut w = World::new("t".into(), 20, 400);
        w.world_surface = 0.0;
        w.rock_layer = 200.0;
        w
    }

    #[test]
    fn light_zero_tile_is_never_revealed() {
        let world = reveal_world();
        let mut map = blank_map(&world);
        assert!(!update_map_lighting(&world, &mut map, 1, 1, 0));
        assert_eq!(map[world.index(1, 1)].light, 0);
    }

    #[test]
    fn brighter_light_wins() {
        let world = reveal_world();
        let mut map = blank_map(&world);
        let i = world.index(1, 1);
        assert!(update_map_lighting(&world, &mut map, 1, 1, 50));
        assert!(update_map_lighting(&world, &mut map, 1, 1, 120));
        assert_eq!(map[i].light, 120);
        assert!(update_map_lighting(&world, &mut map, 1, 1, 200));
        assert!(!update_map_lighting(&world, &mut map, 1, 1, 100));
        assert_eq!(map[i].light, 200);
    }

    #[test]
    fn out_of_bounds_is_ignored() {
        let world = reveal_world();
        let mut map = blank_map(&world);
        assert!(!update_map_lighting(&world, &mut map, -1, 0, 255));
        assert!(!update_map_lighting(&world, &mut map, 0, 400, 255));
    }

    #[test]
    fn reveal_rect_only_touches_lit_tiles() {
        let world = reveal_world();
        let mut map = blank_map(&world);
        let light = LightMap::compute(&world, 5, 5, 10, 10, &LightParams::default());
        let changed = reveal_rect(&world, &mut map, &light, 0, 0, 15, 15);
        let mut revealed = 0;
        for y in 0..=15 {
            for x in 0..=15 {
                if light.get(x, y) == [0.0; 3] {
                    assert_eq!(
                        map[world.index(x, y)].light,
                        0,
                        "dark tile ({x},{y}) must stay dark"
                    );
                } else if map[world.index(x, y)].light > 0 {
                    revealed += 1;
                }
            }
        }
        assert!(revealed > 0, "the computed light rect must reveal a tile");
        // `changed` is exactly the set of newly revealed tiles.
        assert_eq!(changed.len(), revealed);
    }
}
