//! `MapHelper` colour selection: which packed `colorLookup` entry a map pixel
//! resolves to.
//!
//! Ports the per-tile half of `MapHelper.CreateMapTile`
//! (`decompiled/Terraria/Map/MapHelper.cs:1900-1923`): `GetTileType`
//! (`:1925-1985`), `GetTileBaseOption` (`:2083-3005`), `GetWallType`
//! (`:1987-2025`) and `GetBackgroundType` (`:2027-2076`).

use crate::mapdata::WALL_COUNT;
use crate::tile::{FLAG_FULLBRIGHT, Tile};
use crate::world::World;

use super::colors::{calc_sky_gradient, color_table};
use super::{liquid_index, MapTile};

/// `Main.Map[x, y].Type` read from our row-major map slice (`World::index`).
fn map_type_at(world: &World, map: &[MapTile], x: i32, y: i32) -> u16 {
    if !world.in_bounds(x, y) {
        return 0;
    }
    map.get(world.index(x, y)).map_or(0, |t| t.ty)
}

/// `MapHelper.GetTileType` (`MapHelper.cs:1925-1985`).
///
/// Our `Tile` has a single 8-bit `flags` byte: `invisibleBlock()`
/// (`bTileHeader3 & 0x20`) is not representable (ruling 1), `fullbrightBlock()`
/// (`bTileHeader3 & 0x80`) maps to [`FLAG_FULLBRIGHT`] (ruling 2) and
/// `DebugOptions.ShowUnbreakableWall` is a debug toggle treated as `false`.
pub(super) fn get_tile_type(
    i: i32,
    j: i32,
    tile: &Tile,
    new_color: &mut i32,
    new_light: &mut i32,
    base_type: &mut i32,
    base_option: &mut i32,
    world: &World,
) {
    if !tile.active() {
        return;
    }
    let mut num = tile.kind as i32;
    // Vanilla indexes `tileLookup[num]` directly; guard malformed ids.
    *base_type = color_table()
        .tile_lookup
        .get(tile.kind as usize)
        .copied()
        .unwrap_or(0) as i32;
    let invisible = false; // tile.invisibleBlock() — ruling 1.
    if tile.flags & FLAG_FULLBRIGHT != 0 && !invisible {
        *new_light = 255;
    }
    if invisible {
        *base_type = 0;
        return;
    }
    match num {
        5 => {
            // WorldGen.IsThisAMushroomTree is not ported (ruling 5); only this
            // tile's option (1) is affected, so it stays 0.
            *new_color = tile.paint() as i32; // tile.color() — ruling 6.
            return;
        }
        51 | 697 => {
            if (i + j) % 2 == 0 {
                *base_type = 0;
            }
        }
        19 => {
            if tile.frame_y as i32 == 864 {
                *base_type = 0;
            }
        }
        184 => {
            if tile.frame_x / 22 == 10 {
                num = 627;
                *base_type = color_table()
                    .tile_lookup
                    .get(num as usize)
                    .copied()
                    .unwrap_or(0) as i32;
            }
        }
        _ => {}
    }
    if *base_type != 0 {
        // `MapHelper.GetTileBaseOption` (MapHelper.cs:1975).
        *base_option = get_tile_base_option(i, j, num as u16, tile, world) as i32;
        if num == 160 {
            *new_color = 0;
        } else {
            *new_color = tile.paint() as i32; // tile.color() — ruling 6.
        }
    }
}

/// `MapHelper.GetTileBaseOption` (`MapHelper.cs:2083-3005`): the colour variant
/// within a tile's option list, added to `baseType` to index the packed
/// `colorLookup`.
///
/// The switch is transcribed case-for-case in source order. Four cases need data
/// we do not have yet and are substituted with `0` (flagged in place, and listed
/// in `docs/ANALYSIS.md` by Task 9):
/// - `461`  — `Main.SceneMetrics.ZoneCorrupt/ZoneCrimson/ZoneHallow`
/// - `80`   — `WorldGen.GetCactusType`
/// - `529`  — `WorldGen.GetBiomeInfluence`
/// - `530`  — `WorldGen.GetBiomeInfluence`
///
/// `world` is only read by those substituted cases, hence `_world`.
#[allow(unused_assignments)] // `case 4` mirrors the source's assign-then-overwrite.
pub fn get_tile_base_option(
    x: i32,
    y: i32,
    tile_type: u16,
    tile: &Tile,
    _world: &World,
) -> u8 {
    let frame_x = tile.frame_x as i32;
    let frame_y = tile.frame_y as i32;
    let mut base_option = 0i32;
    match tile_type {
        89 => {
            base_option = match frame_x / 54 {
                0 | 21 | 23 => 0,
                43 => 2,
                _ => 1,
            };
        }
        160 | 627 | 628 | 692 => {
            base_option = (x + y) % 9;
        }
        461 => {
            // `Main.SceneMetrics.ZoneCorrupt/ZoneCrimson/ZoneHallow` — not ported.
            base_option = 0;
        }
        80 => {
            // `WorldGen.GetCactusType(x, y, frameX, frameY, ...)` — not ported.
            base_option = 0;
        }
        529 => {
            // `WorldGen.GetBiomeInfluence(x, x, y+1, y+1, ...)` — not ported.
            base_option = 0;
        }
        530 => {
            // `WorldGen.GetBiomeInfluence(...)` — not ported.
            base_option = 0;
        }
        19 => {
            // MapHelper.cs:2179-2188.
            let num = frame_y / 18;
            base_option = 0;
            if num == 48 {
                base_option = 1;
            }
        }
        15 => {
            // MapHelper.cs:2189-2198.
            let num = frame_y / 40;
            base_option = 0;
            if num == 1 || num == 20 {
                base_option = 1;
            }
        }
        518 | 519 => {
            // MapHelper.cs:2199-2202.
            base_option = frame_y / 18;
        }
        4 => {
            // MapHelper.cs:2203-2209: assigns 1 then unconditionally 0.
            if frame_x < 66 {
                base_option = 1;
            }
            base_option = 0;
        }
        572 => {
            // MapHelper.cs:2210-2212.
            base_option = frame_y / 36;
        }
        21 | 441 => {
            // MapHelper.cs:2213-2239.
            base_option = match frame_x / 36 {
                1 | 2 | 10 | 13 | 15 => 1,
                3 | 4 => 2,
                6 => 3,
                11 | 17 => 4,
                _ => 0,
            };
        }
        467 | 468 => {
            // MapHelper.cs:2240-2278.
            let num = frame_x / 36;
            base_option = match num {
                0..=11 => num,
                12 | 13 | 15 | 18 => 10,
                14 | 17 => 7,
                16 => 3,
                _ => 12,
            };
        }
        560 => {
            // MapHelper.cs:2279-2291.
            let num = frame_x / 36;
            if (0..=2).contains(&num) {
                base_option = num;
            } else {
                base_option = 0;
            }
        }
        28 | 653 => {
            // MapHelper.cs:2292-2342.
            base_option = if frame_y < 144 {
                0
            } else if frame_y < 252 {
                1
            } else if frame_y < 360 || (frame_y >= 900 && frame_y < 1008) {
                2
            } else if frame_y < 468 {
                3
            } else if frame_y < 576 {
                4
            } else if frame_y < 684 {
                5
            } else if frame_y < 792 {
                6
            } else if frame_y < 900 {
                8
            } else if frame_y < 1008 {
                7
            } else if frame_y < 1116 {
                0
            } else if frame_y < 1224 {
                3
            } else {
                7
            };
        }
        27 => {
            // MapHelper.cs:2343-2352.
            base_option = if frame_y < 34 { 1 } else { 0 };
        }
        31 | 696 => {
            // MapHelper.cs:2353-2363.
            base_option = if frame_x >= 36 { 1 } else { 0 };
        }
        26 | 695 => {
            // MapHelper.cs:2364-2374.
            base_option = if frame_x >= 54 { 1 } else { 0 };
        }
        137 => {
            // MapHelper.cs:2375-2391.
            base_option = match frame_y / 18 {
                1 | 2 | 3 | 4 => 1,
                5 => 2,
                _ => 0,
            };
        }
        82 | 83 | 84 => {
            // MapHelper.cs:2392-2423.
            base_option = if frame_x < 18 {
                0
            } else if frame_x < 36 {
                1
            } else if frame_x < 54 {
                2
            } else if frame_x < 72 {
                3
            } else if frame_x < 90 {
                4
            } else if frame_x < 108 {
                5
            } else {
                6
            };
        }
        591 => {
            // MapHelper.cs:2424-2426.
            base_option = frame_x / 36;
        }
        105 => {
            // MapHelper.cs:2427-2440.
            base_option = if frame_x >= 1548 && frame_x <= 1654 {
                1
            } else if frame_x >= 1656 && frame_x <= 1798 {
                2
            } else {
                0
            };
        }
        133 => {
            // MapHelper.cs:2441-2450.
            base_option = if frame_x < 52 { 0 } else { 1 };
        }
        134 => {
            // MapHelper.cs:2451-2460.
            base_option = if frame_x < 28 { 0 } else { 1 };
        }
        149 => {
            // MapHelper.cs:2461-2463.
            base_option = y % 3;
        }
        165 | 693 | 694 => {
            // MapHelper.cs:2464-2487.
            base_option = if frame_x < 54 {
                0
            } else if frame_x < 106 {
                1
            } else if frame_x >= 216 {
                1
            } else if frame_x < 162 {
                2
            } else {
                3
            };
        }
        178 => {
            // MapHelper.cs:2488-2517.
            base_option = if frame_x < 18 {
                0
            } else if frame_x < 36 {
                1
            } else if frame_x < 54 {
                2
            } else if frame_x < 72 {
                3
            } else if frame_x < 90 {
                4
            } else if frame_x < 108 {
                5
            } else {
                6
            };
        }
        184 => {
            // MapHelper.cs:2518-2563 (no final else: >= 242 leaves it untouched).
            if frame_x < 22 {
                base_option = 0;
            } else if frame_x < 44 {
                base_option = 1;
            } else if frame_x < 66 {
                base_option = 2;
            } else if frame_x < 88 {
                base_option = 3;
            } else if frame_x < 110 {
                base_option = 4;
            } else if frame_x < 132 {
                base_option = 5;
            } else if frame_x < 154 {
                base_option = 6;
            } else if frame_x < 176 {
                base_option = 7;
            } else if frame_x < 198 {
                base_option = 8;
            } else if frame_x < 220 {
                base_option = 9;
            } else if frame_x < 242 {
                base_option = 10;
            }
        }
        650 => {
            // MapHelper.cs:2564-2600 (no final else).
            let num = frame_x / 18;
            if num < 6 || num == 28 || num == 29 || num == 30 || num == 31 || num == 32 {
                base_option = 0;
            } else if num < 12 || num == 33 || num == 34 || num == 35 {
                base_option = 1;
            } else if num < 28 {
                base_option = 2;
            } else if num < 48 {
                base_option = 3;
            } else if num < 54 {
                base_option = 4;
            } else if num < 72 {
                base_option = 0;
            } else if num == 72 {
                base_option = 1;
            } else if num < 78 {
                base_option = 11;
            }
        }
        649 => {
            // MapHelper.cs:2601-2637 (no final else).
            let num = frame_x / 36 + (frame_y / 18 - 1) * 18;
            if num < 6
                || num == 19
                || num == 20
                || num == 21
                || num == 22
                || num == 23
                || num == 24
                || num == 33
                || num == 38
                || num == 39
                || num == 40
            {
                base_option = 0;
            } else if num < 16 {
                base_option = 2;
            } else if num < 19 || num == 31 || num == 32 {
                base_option = 1;
            } else if num < 31 {
                base_option = 3;
            } else if num < 38 {
                base_option = 4;
            } else if num < 59 {
                base_option = 0;
            } else if num < 62 {
                base_option = 1;
            } else if num < 65 {
                base_option = 11;
            }
        }
        185 => {
            // MapHelper.cs:2638-2711: frame Y selects the 650 or 649 layout.
            if frame_y < 18 {
                let num = frame_x / 18;
                if num < 6 || num == 28 || num == 29 || num == 30 || num == 31 || num == 32 {
                    base_option = 0;
                } else if num < 12 || num == 33 || num == 34 || num == 35 {
                    base_option = 1;
                } else if num < 28 {
                    base_option = 2;
                } else if num < 48 {
                    base_option = 3;
                } else if num < 54 {
                    base_option = 4;
                } else if num < 72 {
                    base_option = 0;
                } else if num == 72 {
                    base_option = 1;
                } else if num < 78 {
                    base_option = 11;
                }
            } else {
                let num = frame_x / 36 + (frame_y / 18 - 1) * 18;
                if num < 6
                    || num == 19
                    || num == 20
                    || num == 21
                    || num == 22
                    || num == 23
                    || num == 24
                    || num == 33
                    || num == 38
                    || num == 39
                    || num == 40
                {
                    base_option = 0;
                } else if num < 16 {
                    base_option = 2;
                } else if num < 19 || num == 31 || num == 32 {
                    base_option = 1;
                } else if num < 31 {
                    base_option = 3;
                } else if num < 38 {
                    base_option = 4;
                } else if num < 59 {
                    base_option = 0;
                } else if num < 62 {
                    base_option = 1;
                } else if num < 65 {
                    base_option = 11;
                }
            }
        }
        186 | 647 => {
            // MapHelper.cs:2712-2737 (no final else).
            let num = frame_x / 54;
            if num < 7 {
                base_option = 2;
            } else if num < 22 || num == 33 || num == 34 || num == 35 {
                base_option = 0;
            } else if num < 25 {
                base_option = 1;
            } else if num == 25 {
                base_option = 5;
            } else if num < 32 {
                base_option = 3;
            }
        }
        187 | 648 => {
            // MapHelper.cs:2738-2791 (no final else).
            let num = frame_x / 54 + frame_y / 36 * 36;
            if num < 3 || num == 14 || num == 15 || num == 16 {
                base_option = 0;
            } else if num < 6 {
                base_option = 6;
            } else if num < 9 {
                base_option = 7;
            } else if num < 14 {
                base_option = 4;
            } else if num < 18 {
                base_option = 4;
            } else if num < 23 {
                base_option = 8;
            } else if num < 25 {
                base_option = 0;
            } else if num < 29 {
                base_option = 1;
            } else if num < 47 {
                base_option = 0;
            } else if num < 50 {
                base_option = 1;
            } else if num < 52 {
                base_option = 10;
            } else if num < 55 {
                base_option = 2;
            }
        }
        227 => {
            // MapHelper.cs:2792-2794.
            base_option = frame_x / 34;
        }
        129 => {
            // MapHelper.cs:2795-2804.
            base_option = if frame_x >= 324 { 1 } else { 0 };
        }
        240 => {
            // MapHelper.cs:2805-2868.
            let num = frame_x / 54 + frame_y / 54 * 36;
            if (0..=11).contains(&num)
                || (47..=53).contains(&num)
                || num == 72
                || num == 73
                || num == 75
            {
                base_option = 0;
            } else if (num < 12 || num > 15)
                && (num < 18 || num > 35)
                && (num < 63 || num > 71)
            {
                // The source's `return`s mean every arm here terminates the
                // function; only the 74/76..100 arms set 1, the rest keep 0.
                match num {
                    16 | 17 => base_option = 2,
                    41..=45 => base_option = 3,
                    46 => base_option = 4,
                    74 | 76..=100 => base_option = 1,
                    _ => {}
                }
            } else {
                base_option = 1;
            }
        }
        242 => {
            // MapHelper.cs:2869-2881.
            let num = frame_y / 72;
            base_option = if frame_x / 106 == 0 && (22..=24).contains(&num) {
                1
            } else {
                0
            };
        }
        440 => {
            // MapHelper.cs:2882-2891.
            let mut num = frame_x / 54;
            if num > 6 {
                num = 6;
            }
            base_option = num;
        }
        457 => {
            // MapHelper.cs:2892-2901.
            let mut num = frame_x / 36;
            if num > 4 {
                num = 4;
            }
            base_option = num;
        }
        453 => {
            // MapHelper.cs:2902-2911.
            let mut num = frame_x / 36;
            if num > 2 {
                num = 2;
            }
            base_option = num;
        }
        419 => {
            // MapHelper.cs:2912-2921.
            let mut num = frame_x / 18;
            if num > 2 {
                num = 2;
            }
            base_option = num;
        }
        428 => {
            // MapHelper.cs:2922-2931.
            let mut num = frame_y / 18;
            if num > 3 {
                num = 3;
            }
            base_option = num;
        }
        420 => {
            // MapHelper.cs:2932-2941.
            let mut num = frame_y / 18;
            if num > 5 {
                num = 5;
            }
            base_option = num;
        }
        423 => {
            // MapHelper.cs:2942-2951.
            let mut num = frame_y / 18;
            if num > 6 {
                num = 6;
            }
            base_option = num;
        }
        493 => {
            // MapHelper.cs:2952-2977.
            base_option = if frame_x < 18 {
                0
            } else if frame_x < 36 {
                1
            } else if frame_x < 54 {
                2
            } else if frame_x < 72 {
                3
            } else if frame_x < 90 {
                4
            } else {
                5
            };
        }
        548 => {
            // MapHelper.cs:2978-2987.
            base_option = if frame_x / 54 < 7 { 0 } else { 1 };
        }
        597 => {
            // MapHelper.cs:2988-3000.
            let num = frame_x / 54;
            if (0..=10).contains(&num) {
                base_option = num;
            } else {
                base_option = 0;
            }
        }
        _ => {
            // MapHelper.cs:3001-3003.
            base_option = 0;
        }
    }
    base_option as u8
}

/// `MapHelper.GetWallType` (`MapHelper.cs:1987-2025`).
///
/// `invisibleWall()` (`bTileHeader3 & 0x40`, ruling 1) and `fullbrightWall()`
/// (`sTileHeader & 0x8000`, ruling 3 — a *separate* wall flag we do not store)
/// are both absent, so the `newLight = 255` branch is unreachable.
pub(super) fn get_wall_type(
    i: i32,
    _j: i32,
    tile: &Tile,
    new_color: &mut i32,
    _new_light: &mut i32,
    base_type: &mut i32,
    base_option: &mut i32,
) {
    let invisible = false; // tile.invisibleWall() — ruling 1.
    if tile.liquid > 32 {
        // Tile.liquidType() -> LIQUID_COLORS slot (MapHelper.cs:1996-1998).
        *base_type = color_table().liquid_position as i32 + liquid_index(tile.liquid_kind()) as i32;
    } else if !invisible && tile.wall > 0 && (tile.wall as usize) < WALL_COUNT {
        let wall = tile.wall as usize;
        *base_type = color_table().wall_lookup[wall] as i32;
        *new_color = tile.paint() as i32; // tile.wallColor() — ruling 6.
        match tile.wall {
            21 | 88 | 89 | 90 | 91 | 92 | 93 | 168 | 241 => *new_color = 0,
            27 => *base_option = i % 2,
            _ => *base_option = 0,
        }
    }
}

/// `MapHelper.GetBackgroundType` (`MapHelper.cs:2027-2076`).
pub(super) fn get_background_type(world: &World, map: &[MapTile], i: i32, j: i32, new_light: &mut i32) -> u16 {
    if (j as f64) < world.world_surface {
        // Main.remixWorld == false (ruling 4).
        *new_light = 255;
        return calc_sky_gradient(j, world.world_surface);
    }
    // Main.UnderworldLayer == Main.maxTilesY - 200 (Main.cs:2790).
    let underworld_layer = world.height - 200;
    if j < underworld_layer {
        let mut b = 0u8;
        // `sceneArea`/`sceneSnowiness` are not ported (ruling 8): the neighbour
        // scan always runs, bounds-checked against the world by `map_type_at`.
        'scan: for k in (i - 36..=i + 30).step_by(10) {
            for l in (j - 36..=j + 30).step_by(10) {
                let ty = map_type_at(world, map, k, l);
                for snow in color_table().snow_types {
                    if snow == ty {
                        b = 255;
                        break 'scan;
                    }
                }
            }
        }
        if (j as f64) < world.rock_layer {
            return color_table().dirt_position + b as u16;
        }
        return color_table().rock_position + b as u16;
    }
    color_table().hell_position
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{color_table, create_map_tile, MapTile};
    use crate::tile::FLAG_ACTIVE;

    // ---- Task 3b: GetTileBaseOption ----

    fn option_world() -> World {
        World::new("test".into(), 20, 20)
    }

    fn framed(kind: u16, frame_x: i16, frame_y: i16) -> Tile {
        Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        }
    }

    /// `(x, y)` fixed at `(5, 5)`; only the `(x + y) % 9` cases read them.
    fn base_option_of(kind: u16, frame_x: i16, frame_y: i16) -> u8 {
        let world = option_world();
        let tile = framed(kind, frame_x, frame_y);
        get_tile_base_option(5, 5, kind, &tile, &world)
    }

    fn blank_map(world: &World) -> Vec<MapTile> {
        vec![MapTile::default(); (world.width * world.height) as usize]
    }

    #[test]
    fn tile_518_options_use_frame_row() {
        // MapHelper.cs:2199-2202: `baseOption = frameY / 18`. (The task brief
        // wrote `frameY = 36 -> 1`; `36 / 18 == 2`, so the source-true values
        // are used here.)
        assert_eq!(base_option_of(518, 0, 0), 0);
        assert_eq!(base_option_of(518, 0, 18), 1);
        assert_eq!(base_option_of(518, 0, 36), 2);
    }

    #[test]
    fn tile_19_options_use_the_waterfall_row() {
        // MapHelper.cs:2179-2188: `frameY / 18 == 48`.
        assert_eq!(base_option_of(19, 0, 864), 1);
        assert_eq!(base_option_of(19, 0, 0), 0);
    }

    #[test]
    fn tile_21_options_use_frame_column() {
        // MapHelper.cs:2213-2239: `switch (frameX / 36)`.
        assert_eq!(base_option_of(21, 0, 0), 0);
        assert_eq!(base_option_of(21, 36, 0), 1);
        assert_eq!(base_option_of(21, 108, 0), 2); // 3
        assert_eq!(base_option_of(21, 216, 0), 3); // 6
        assert_eq!(base_option_of(21, 396, 0), 4); // 11
    }

    #[test]
    fn tile_89_options_special_case() {
        // MapHelper.cs:2087-2102: `switch (frameX / 54)`.
        assert_eq!(base_option_of(89, 0, 0), 0);
        assert_eq!(base_option_of(89, 54, 0), 1); // default
        assert_eq!(base_option_of(89, 43 * 54, 0), 2); // case 43
    }

    #[test]
    fn tile_160_repeats_every_nine() {
        // MapHelper.cs:2103-2108: `(x + y) % 9`.
        let world = option_world();
        let tile = framed(160, 0, 0);
        assert_eq!(get_tile_base_option(4, 5, 160, &tile, &world), 0); // 9 % 9
        assert_eq!(get_tile_base_option(4, 6, 160, &tile, &world), 1); // 10 % 9
        assert_eq!(get_tile_base_option(2, 1, 160, &tile, &world), 3); // 3 % 9
    }

    #[test]
    fn tile_4_always_zero() {
        // MapHelper.cs:2203-2209: assigns 1 then unconditionally 0.
        assert_eq!(base_option_of(4, 0, 0), 0);
        assert_eq!(base_option_of(4, 999, 0), 0);
    }

    #[test]
    fn unknown_tile_is_zero() {
        // MapHelper.cs:3001-3003: default.
        assert_eq!(base_option_of(12345, 0, 0), 0);
    }

    #[test]
    fn zone_dependent_cases_default_to_zero() {
        // Substituted in the port: 461 (SceneMetrics zones), 80
        // (WorldGen.GetCactusType), 529/530 (WorldGen.GetBiomeInfluence).
        for kind in [461u16, 529, 530, 80] {
            assert_eq!(base_option_of(kind, 1234, 567), 0, "type {kind}");
        }
    }

    #[test]
    fn get_tile_type_uses_the_option() {
        let mut world = World::new("test".into(), 20, 20);
        {
            let t = world.tile_mut(5, 5).unwrap();
            t.kind = 518;
            t.frame_y = 18; // frameY / 18 == 1
            t.flags |= FLAG_ACTIVE;
        }
        let map = blank_map(&world);
        let mt = create_map_tile(&world, &map, 5, 5, 255, 0);
        assert_eq!(mt.ty, color_table().tile_lookup[518] + 1);
    }
}
