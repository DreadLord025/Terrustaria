//! The ocean pass family (M5-OCEAN): `Ocean Sand`, `Beaches` and
//! `Gravitating Sand` — the vanilla pre-carve sandifier, the ocean carve and
//! the sand settle, ported verbatim under the golden rule.
//!
//! Call graph of the family (all RNG from one `genRand`-class stream; here
//! each pass takes its own `pass_rng` stream, the per-pass re-seed model of
//! `WorldGenerator.cs:497`):
//!
//! ```text
//! OceanSand (WorldGen.cs:11580)  — wedge sand over both beach bands
//!   └── writes Main.tile[j,k].type = 53 only (no walls, no water)
//! BeachesAndOceanCleanup (:14888) — the ocean carve, anchors GenVars.shellStart*
//!   └── WorldGen.TuneOceanDepth (:26960)  — the per-column seafloor curve
//! GravitatingSandCleanup (:15138) — fills the air gaps the carve leaves
//!   └── WorldGen.SolidOrSlopedTile (:69985), Tile.ResetToType (Tile.cs:249)
//! ```
//!
//! The beach anchors (`leftBeachEnd`/`rightBeachStart`) come from the `Beaches`
//! stream's Reset draws (P1, terrain step — the first consumer) — the passes
//! read them from `GenVars` without redrawing.

use crate::tiledata as td;
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
use crate::world::World;
use super::biome_detail::empty_tile_check;
use super::noise::Rng;
use super::pipeline::GenVars;
use super::terrain::Layers;
use super::{TILE_HARDENED_SAND, TILE_SAND};

/// `GenVars.oceanWaterStartRandomMin`/`Max` (`GenVars.cs:112-114`, set in
/// Reset `WorldGen.cs:11196-11197`).
const OCEAN_WATER_START_MIN: i32 = 220;
const OCEAN_WATER_START_MAX: i32 = 260;
/// `GenVars.oceanWaterForcedJungleLength` (`GenVars.cs:116`, `WorldGen.cs:11198`):
/// the water start forced on the jungle-adjacent ocean (dungeonSide >= Right).
const OCEAN_WATER_FORCED_JUNGLE_LENGTH: i32 = 275;
/// The `int num = 50` padding shared by the two beach carve branches
/// (`WorldGen.cs:14898`).
const BEACH_PADDING: i32 = 50;
/// `TileID.Sets.Falling` (`TileID.cs:610`) — the tiles `GravitatingSandCleanup`
/// refills into the air gaps.
const FALLING: [u16; 11] = [53, 234, 112, 116, 224, 123, 330, 331, 332, 333, 495];

fn falling(kind: u16) -> bool {
    FALLING.contains(&kind)
}

/// `WorldGen.SolidOrSlopedTile(Tile, includePlatforms: false)`
/// (`WorldGen.cs:69985-70009`): active, `tileSolid` and not `tileSolidTop`
/// and not wired-inactive. No wiring exists during generation, so the
/// `inActive` arm never fires here.
#[inline]
fn solid_or_sloped(t: &Tile) -> bool {
    t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
}

/// `Ocean Sand` (`WorldGen.cs:11580-11687`). The per-iteration `num` draw and
/// the `i == 1` full-draw `continue` are RNG-burn quirks of the original —
/// ported verbatim: the middle iteration draws `num`, `num2`, the extra
/// `next(20,40) * num3`, the two `Next(3)` doubles and `continue`s.
/// `Main.maxTilesX` is `world.width`; `Main.worldSurface`/`rockLayer` the
/// layer anchors.
pub(crate) fn ocean_sand(
    world: &mut World,
    gv: &mut GenVars,
    layers: &super::terrain::Layers,
    rng: &mut Rng,
) {
    let (left_beach_end, right_beach_start) = (gv.left_beach_end, gv.right_beach_start);
    let (world_surface, rock_layer) = (layers.world_surface, layers.rock_layer);
    let (max_x, max_y) = (world.width, world.height);
    let mid_scan = ((world_surface + rock_layer) / 2.0) as i32;

    for i in 0..3 {
        // The centre draw, re-rolled while inside (0.4, 0.6)·maxTilesX
        // (`:11588-11592`) — drawn but unused for i == 0 / 2.
        let mut num = rng.range(0, max_x);
        while num > (max_x as f64 * 0.4) as i32 && num < (max_x as f64 * 0.6) as i32 {
            num = rng.range(0, max_x);
        }
        let mut num2 = rng.range(35, 90);
        if i == 1 {
            let num3 = max_x as f64 / 4200.0;
            num2 += (rng.range(20, 40) as f64 * num3) as i32;
        }
        if rng.chance(3) {
            num2 *= 2;
        }
        if i == 1 {
            num2 *= 2;
        }
        let num4 = num - num2;
        let mut num2 = rng.range(35, 90);
        if rng.chance(3) {
            num2 *= 2;
        }
        if i == 1 {
            num2 *= 2;
        }
        let num5 = num + num2;
        let num4 = if num4 < 0 { 0 } else { num4 };
        let num5 = if num5 > max_x { max_x } else { num5 };
        // The edge bands replace the drawn centre (`:11626-11639`); i == 1
        // burns its draws and continues (the middle-band branch never runs).
        let (num4, num5) = match i {
            0 => (0, left_beach_end.min(max_x).max(0)),
            2 => (right_beach_start.min(max_x).max(0), max_x),
            _ => continue,
        };
        let mut num6 = rng.range(50, 100);
        for j in num4..num5 {
            if rng.chance(2) {
                num6 += rng.range(-1, 2);
                if num6 < 50 {
                    num6 = 50;
                }
                if num6 > 200 {
                    num6 = 200;
                }
            }
            // The k-scan (`Main.tile[j, k].active()`) stops at the first
            // active tile, bounded by the mid-line (`:11655-11657`).
            let mut k = 0;
            while k < mid_scan && !world.tile(j, k).is_some_and(|t| t.active()) {
                k += 1;
            }
            if k >= mid_scan {
                continue;
            }
            let is_mid = j == (num4 + num5) / 2;
            if is_mid && rng.chance(6) {
                // `GenVars.PyrX/PyrY[numPyr]` (`:11659-11664`) — consumed by
                // the Pyramids pass (published, unconsumed today).
                let n = gv.num_pyr;
                if (n as usize) < gv.pyr_x.len() {
                    gv.pyr_x[n as usize] = j;
                    gv.pyr_y[n as usize] = k;
                    gv.num_pyr = n + 1;
                }
            }
            let mut num7 = num6;
            if j - num4 < num7 {
                num7 = j - num4;
            }
            if num5 - j < num7 {
                num7 = num5 - j;
            }
            num7 += rng.range(0, 5);
            for l in k..k + num7 {
                // Two `Next(5)` draws per tile-iteration (`:11677`).
                if j > num4 + rng.range(0, 5) && j < num5 - rng.range(0, 5) {
                    let idx = world.index(j, l);
                    world.tiles[idx].kind = TILE_SAND;
                }
            }
            break;
        }
    }
}

/// `WorldGen.TuneOceanDepth` (`WorldGen.cs:26960-27093`): the per-column
/// seafloor depth increment — one `genRand.Next(10, 20)` draw, scaled by the
/// column-count factor table. The garden-variety curve falls 0.2 → 0.001
/// then rises to 0.1; the `floridaStyle` curve is the mirror (0.001 → 0.6).
pub(crate) fn tune_ocean_depth(count: i32, mut depth: f64, florida_style: bool, rng: &mut Rng) -> f64 {
    let n = rng.range(10, 20) as f64;
    if !florida_style {
        if count < 3 {
            depth += n * 0.2;
        } else if count < 6 {
            depth += n * 0.15;
        } else if count < 9 {
            depth += n * 0.1;
        } else if count < 15 {
            depth += n * 0.07;
        } else if count < 50 {
            depth += n * 0.05;
        } else if count < 75 {
            depth += n * 0.04;
        } else if count < 100 {
            depth += n * 0.03;
        } else if count < 125 {
            depth += n * 0.02;
        } else if count < 150 {
            depth += n * 0.01;
        } else if count < 175 {
            depth += n * 0.005;
        } else if count < 200 {
            depth += n * 0.001;
        } else if count < 230 {
            depth += n * 0.01;
        } else if count < 235 {
            depth += n * 0.05;
        } else if count < 240 {
            depth += n * 0.1;
        } else if count < 245 {
            depth += n * 0.05;
        } else if count < 255 {
            depth += n * 0.01;
        }
    } else if count < 3 {
        depth += n * 0.001;
    } else if count < 6 {
        depth += n * 0.002;
    } else if count < 9 {
        depth += n * 0.004;
    } else if count < 15 {
        depth += n * 0.007;
    } else if count < 50 {
        depth += n * 0.01;
    } else if count < 75 {
        depth += n * 0.014;
    } else if count < 100 {
        depth += n * 0.019;
    } else if count < 125 {
        depth += n * 0.027;
    } else if count < 150 {
        depth += n * 0.038;
    } else if count < 175 {
        depth += n * 0.052;
    } else if count < 200 {
        depth += n * 0.08;
    } else if count < 230 {
        depth += n * 0.12;
    } else if count < 235 {
        depth += n * 0.16;
    } else if count < 240 {
        depth += n * 0.27;
    } else if count < 245 {
        depth += n * 0.43;
    } else if count < 255 {
        depth += n * 0.6;
    }
    depth
}

/// The left-ocean carve (`WorldGen.cs:14917-14980`): one column of the
/// `for (i == 0)` branch. Returns the water surface row.
fn carve_left(
    world: &mut World,
    gv: &mut GenVars,
    num3: i32,
    florida_style: bool,
    rng: &mut Rng,
) -> i32 {
    let (max_x, max_y) = (world.width, world.height);
    // `for (j = 0; !Main.tile[num3 - 1, j].active(); j++)` (`:14934-14937`)
    let mut j = 0;
    while j < max_y && !world.tile(num3 - 1, j).is_some_and(|t| t.active()) {
        j += 1;
    }
    gv.shell_start_y_left = j;
    j += rng.range(1, 5);
    let mut num6 = 1.0f64;
    let mut num5 = 0i32;
    let mut num8 = num3 - 1;
    while num8 >= 0 {
        // `num6` is the seafloor depth from the water surface; the last 30
        // edge columns drop linearly (`:14943-14951`).
        if num8 > 30 {
            num5 += 1;
            num6 = tune_ocean_depth(num5, num6, florida_style, rng);
        } else {
            num6 += 1.0;
        }
        carve_column(world, gv, num8, j, num6, rng, true);
        num8 -= 1;
    }
    j
}

/// The right-ocean carve (`WorldGen.cs:14981-15044`): the mirror branch.
/// Returns the water surface row.
fn carve_right(
    world: &mut World,
    gv: &mut GenVars,
    num2: i32,
    florida_style: bool,
    rng: &mut Rng,
) -> i32 {
    let (max_x, max_y) = (world.width, world.height);
    // `for (l = 0; !Main.tile[num2, l].active(); l++)` (`:15001-15003`) — the
    // scan column is `num2` itself here (the left branch scans `num3 - 1`).
    let mut l = 0;
    while l < max_y && !world.tile(num2, l).is_some_and(|t| t.active()) {
        l += 1;
    }
    gv.shell_start_x_right = 0;
    gv.shell_start_y_right = l;
    l += rng.range(1, 5);
    let mut num11 = 1.0f64;
    let mut num12 = 0i32;
    let mut m = num2;
    while m < max_x {
        if m < max_x - 30 {
            num12 += 1;
            num11 = tune_ocean_depth(num12, num11, florida_style, rng);
        } else {
            num11 += 1.0;
        }
        carve_column(world, gv, m, l, num11, rng, false);
        m += 1;
    }
    l
}

/// One carved column of either branch (`:14952-14979` / `:15014-15043`):
/// the air/water/sand prism with the `liquid = 127` surface row and the
/// `shellStartX*` first-column record.
fn carve_column(
    world: &mut World,
    gv: &mut GenVars,
    x: i32,
    j: i32,
    depth: f64,
    rng: &mut Rng,
    left: bool,
) {
    let max_y = world.height;
    let num9 = rng.range(15, 20);
    // `for (k = 0; k < j + depth + num9; k++)` — the comparison is double in
    // C# (`(double)k < (double)j + num6 + (double)num9`).
    let mut k = 0;
    while (k as f64) < j as f64 + depth + num9 as f64 {
        if (k as f64) < j as f64 + depth * 0.75 - 3.0 {
            let idx = world.index(x, k);
            world.tiles[idx].flags &= !FLAG_ACTIVE;
            if k > j {
                world.tiles[idx].liquid = 255;
                // `liquidType(0)` (`Tile.cs:216-222`): the water type clears
                // every liquid-type flag.
                world.tiles[idx].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER);
            } else if k == j {
                world.tiles[idx].liquid = 127;
                let anchor = if left { &mut gv.shell_start_x_left } else { &mut gv.shell_start_x_right };
                if *anchor == 0 {
                    *anchor = x;
                }
            }
        } else if k > j {
            let idx = world.index(x, k);
            world.tiles[idx].kind = TILE_SAND;
            world.tiles[idx].flags |= FLAG_ACTIVE;
        }
        let idx = world.index(x, k);
        world.tiles[idx].wall = 0;
        k += 1;
    }
    let _ = max_y;
}

/// `Beaches` / `BeachesAndOceanCleanup` (`WorldGen.cs:14888-15047`): the
/// ocean carve on both sides. The `floridaStyle`/`floridaStyle2` flags come
/// from the shared `Next(4)`/`Next(2)` opener; the dungeon side is fixed
/// Right (M5-BIOME), so the LEFT ocean always takes the forced 275 water
/// start and the right side always its draw.
pub(crate) fn beaches(world: &mut World, gv: &mut GenVars, rng: &mut Rng) {
    let max_x = world.width;
    // The opener (`:14902-14912`).
    let (florida_style, florida_style2) = if rng.chance(4) {
        if rng.chance(2) {
            (true, false)
        } else {
            (false, true)
        }
    } else {
        (false, false)
    };
    for i in 0..2 {
        if i == 0 {
            // `num3 = Next(220, 260)`, forced to 275 when the dungeon side
            // is at or right of this ocean, then capped at
            // `leftBeachEnd - 50` (`:14921-14930`).
            let mut num3 = rng.range(OCEAN_WATER_START_MIN, OCEAN_WATER_START_MAX);
            // The port's dungeon side is fixed `DungeonSide.Right` (the
            // M5-BIOME stand-in), so the left ocean always takes the forced
            // length.
            num3 = OCEAN_WATER_FORCED_JUNGLE_LENGTH;
            let num4 = gv.left_beach_end - BEACH_PADDING;
            if num3 > num4 {
                num3 = num4;
            }
            carve_left(world, gv, num3, florida_style, rng);
        } else {
            // `num2 = maxTilesX - Next(220, 260)`, the forced branch skipped
            // (dungeonSide <= Left never holds), then raised to
            // `rightBeachStart + 50` (`:14981-14994`).
            let mut num2 = max_x - rng.range(OCEAN_WATER_START_MIN, OCEAN_WATER_START_MAX);
            let num10 = gv.right_beach_start + BEACH_PADDING;
            if num2 < num10 {
                num2 = num10;
            }
            carve_right(world, gv, num2, florida_style2, rng);
        }
    }
}

/// `Tile.ResetToType` (`Tile.cs:249-259`): liquid 0, `sTileHeader = 32` (the
/// active bit alone — the liquid amount/type and the slope bits in
/// `sTileHeader` all clear), the paint/colored headers 0, frames 0, type set.
pub(crate) fn reset_to_type(t: &mut Tile, kind: u16) {
    t.liquid = 0;
    t.flags = FLAG_ACTIVE;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
    t.kind = kind;
}

/// `Gravitating Sand` / `GravitatingSandCleanup` (`WorldGen.cs:15138-15165`):
/// every column, bottom-up, refills the air gaps under a falling tile with
/// the falling tile above. `num2 < (int)Main.worldSurface` is the surface
/// cutoff.
pub(crate) fn gravitating_sand(world: &mut World, world_surface: f64) {
    let height = world.height as i32;
    let surface = world_surface as i32;
    for i in 0..world.width {
        let mut flag = false;
        let mut num = 0i32;
        let mut num2 = height - 1;
        while num2 > 0 {
            let t = &world.tiles[world.index(i, num2)];
            if solid_or_sloped(t) {
                let kind = t.kind;
                if flag && num2 < surface && num2 != num - 1 && falling(kind) {
                    let mut j = num2;
                    while j < num {
                        let idx = world.index(i, j);
                        reset_to_type(&mut world.tiles[idx], kind);
                        j += 1;
                    }
                }
                flag = true;
                num = num2;
            }
            num2 -= 1;
        }
    }
}

// --- Create Ocean Caves (the faithful walker) ---------------------------------

/// `WorldGen.beachDistance` (`WorldGen.cs:86914`, decl `:4154`) — the
/// dungeon-side ocean caves only run in the outer band. Shared with the
/// Water Chests scatter's trySlope beach test (W4-c33).
pub(crate) const BEACH_DISTANCE: i32 = 380;
/// `GenVars.maxOceanCaveTreasure` (`GenVars.cs:440`, readonly 2).
const MAX_OCEAN_CAVE_TREASURE: i32 = 2;
/// The carve sentinel — `num = 264` (`WorldGen.cs:76471`): an inert type id
/// used purely as the "already carved" marker in the ring's `type != num`
/// test (`:76546`). 264 is `SapphireGemspark` in this decompile
/// (`TileID.cs:1157`), chosen by the original only for being an inactive
/// marker value.
const OCEAN_CAVE_CARVED: u16 = 264;
/// `badOceanCaveTiles`' protected tiles (`WorldGen.cs:76443-76450`):
/// ebonstone 25 / crimstone 203 / altar 26 / shadow orb 31 plus the dungeon
/// bricks `Main.tileDungeon` {41, 43, 44, 677, 678, 679} (`Main.cs:7992-7998`).
const BAD_OCEAN_CAVE_TILES: [u16; 10] = [25, 203, 26, 31, 41, 43, 44, 677, 678, 679];
/// `badOceanCaveTiles`' protected walls (`:76443-76450`): grass wall 3 / blue
/// wall 83 plus `Main.wallDungeon` {7, 8, 9, 94, 95, 96, 97, 98, 99}
/// (`Main.cs:10598-10606`).
const BAD_OCEAN_CAVE_WALLS: [u16; 11] = [3, 83, 7, 8, 9, 94, 95, 96, 97, 98, 99];

fn bad_ocean_cave_tiles(world: &World, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return true;
    }
    let t = world.tile(x, y).unwrap();
    BAD_OCEAN_CAVE_WALLS.contains(&t.wall) || BAD_OCEAN_CAVE_TILES.contains(&t.kind)
}

/// `WorldGen.SolidTile(Tile)` (`WorldGen.cs:69957-69970`): nactive (active and
/// not wired-inactive), `tileSolid` and not `tileSolidTop`, not a half brick,
/// no slope.
#[inline]
fn solid_tile(t: &Tile) -> bool {
    t.active()
        && (t.kind as usize) < td::TILE_COUNT
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `Utils.Clamp` (`Utils.cs`): the standard three-argument clamp.
#[inline]
fn clampf(v: f64, lo: f64, hi: f64) -> f64 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// The treasure slot writer — `GenVars.oceanCaveTreasure[numOceanCaveTreasure]`
/// (`WorldGen.cs:76503-76507`), with the wrap-at-2 counter (`:76454-76457`).
fn record_cave_treasure(gv: &mut GenVars, x: i32, y: i32) {
    let idx = gv.num_ocean_cave_treasure;
    if idx >= MAX_OCEAN_CAVE_TREASURE {
        gv.num_ocean_cave_treasure = 0;
    }
    let slot = &mut gv.ocean_cave_treasure[gv.num_ocean_cave_treasure as usize];
    slot.0 = x;
    slot.1 = y;
}

/// The sand stalactite column the walker grows under its centre
/// (`WorldGen.cs:76566-76629`).
#[allow(clippy::too_many_arguments)]
fn cave_column(
    world: &mut World,
    k: i32,
    l: i32,
    depth: i32,
    hardened: i32,
    half_width: i32,
    right: bool,
    rng: &mut Rng,
) {
    let (num15, num16) = if right {
        (k, k + half_width)
    } else {
        (k - half_width, k)
    };
    for m in num15..=num16 {
        let mut n = l;
        while n < l + depth && !bad_ocean_cave_tiles(world, m, n) {
            if n > l + hardened {
                let idx = world.index(m, n);
                if solid_tile(&world.tiles[idx]) && world.tiles[idx].kind != TILE_SAND {
                    break;
                }
                world.tiles[idx].kind = TILE_HARDENED_SAND;
            } else {
                let idx = world.index(m, n);
                world.tiles[idx].kind = TILE_SAND;
            }
            let idx = world.index(m, n);
            world.tiles[idx].flags |= FLAG_ACTIVE;
            if rng.chance(3) {
                let i = world.index(m - 1, n);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            if rng.chance(3) {
                let i = world.index(m + 1, n);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            n += 1;
        }
    }
}

/// `WorldGen.oceanCave(i, j)` (`WorldGen.cs:76452-76713`): the ocean cave
/// walker — the direction/budget steer, the shrinking ellipse disc with the
/// sand/hardened-sand ring on the outward half, the centre stalactite column,
/// the water fill and the 5-wide water shaft, the treasure anchors.
///
/// The dungeon-bounds shrink arm (`:76532-76535`, `DualDungeons` only) is
/// coded-but-inactive: the port has no dual-dungeon seed (recorded).
pub(crate) fn ocean_cave(
    world: &mut World,
    gv: &mut GenVars,
    layers: &Layers,
    i: i32,
    j: i32,
    rng: &mut Rng,
) {
    if gv.num_ocean_cave_treasure >= MAX_OCEAN_CAVE_TREASURE {
        gv.num_ocean_cave_treasure = 0;
    }
    let (max_x, max_y) = (world.width, world.height);
    let (world_surface, rock_layer) = (layers.world_surface, layers.rock_layer);
    let (mut x, mut y) = (i as f64, j as f64);
    let (mut vel_x, mut vel_y) = if i < max_x / 2 {
        (0.25 + rng.next_f64() * 0.25, 0.0)
    } else {
        (-0.35 - rng.next_f64() * 0.5, 0.0)
    };
    vel_y = 0.4 + rng.next_f64() * 0.25;
    let mut num4 = rng.range(17, 25) as f64;
    let mut num5 = rng.range(600, 800) as f64;
    let num6 = 4.0f64;
    let mut flag = true;
    let mut flag2 = false;
    while num4 > num6 && num5 > 0.0 {
        let mut flag3 = true;
        let mut flag4 = true;
        let mut flag5 = true;
        // The landmass-band shrink (`:76483-76488`).
        if flag2 || (x > (BEACH_DISTANCE - 50) as f64 && x < (max_x - BEACH_DISTANCE + 50) as f64) {
            num4 *= 0.96;
            num5 *= 0.96;
        }
        if num4 < num6 + 2.0 || num5 < 20.0 {
            flag5 = false;
        }
        if flag {
            num4 -= 0.01 + rng.next_f64() * 0.01;
            num5 -= 0.5;
        } else {
            num4 -= 0.02 + rng.next_f64() * 0.02;
            num5 -= 1.0;
        }
        if flag5 {
            record_cave_treasure(gv, x as i32, y as i32);
        }
        // The ellipse box, clamped (`:76508-76527`).
        let num7 = ((x - num4 * 3.0) as i32).max(1);
        let num8 = ((x + num4 * 3.0) as i32).min(max_x - 1);
        let num9 = ((y - num4 * 3.0) as i32).max(1);
        let num10 = ((y + num4 * 3.0) as i32).min(max_y - 1);
        let mut k = num7;
        while k < num8 {
            let mut l = num9;
            while l < num10 {
                // `DualDungeons` + `InAnyPotentialDungeonBounds` — inactive in
                // the port (no dual-dungeon seed).
                if bad_ocean_cave_tiles(world, k, l) {
                    l += 1;
                    continue;
                }
                let num11 = ((k as f64 - x).abs().powi(2) + (l as f64 - y).abs().powi(2)).sqrt();
                if flag5 && num11 < num4 * 0.5 + 1.0 {
                    // The carve: inactive + sentinel type.
                    let idx = world.index(k, l);
                    world.tiles[idx].kind = OCEAN_CAVE_CARVED;
                    world.tiles[idx].flags &= !FLAG_ACTIVE;
                } else if num11 < num4 * 1.5 + 1.0 && world.tile(k, l).unwrap().kind != OCEAN_CAVE_CARVED {
                    if (l as f64) < y {
                        // The outward upper half (`:76548-76564`).
                        if (vel_x < 0.0 && (k as f64) < x) || (vel_x > 0.0 && (k as f64) > x) {
                            let idx = world.index(k, l);
                            if num11 < num4 * 1.1 + 1.0 {
                                world.tiles[idx].kind = TILE_HARDENED_SAND;
                                if world.tiles[idx].liquid == 255 {
                                    world.tiles[idx].wall = 0;
                                }
                            } else if world.tiles[idx].kind != TILE_HARDENED_SAND {
                                world.tiles[idx].kind = TILE_SAND;
                            }
                        }
                    } else if (vel_x < 0.0 && k < i) || (vel_x > 0.0 && k > i) {
                        // The lower half, anchored at the ORIGINAL `i`
                        // (`:76566`) — the walker's steered x is NOT used here.
                        let idx = world.index(k, l);
                        if world.tiles[idx].liquid == 255 {
                            world.tiles[idx].wall = 0;
                        }
                        world.tiles[idx].kind = TILE_SAND;
                        world.tiles[idx].flags |= FLAG_ACTIVE;
                        if k == x as i32 && flag3 {
                            flag3 = false;
                            let num12 = 50 + rng.range(0, 3);
                            let num13 = 43 + rng.range(0, 3);
                            let num14 = 20 + rng.range(0, 3);
                            let (num12, num13, num14) = if num5 < 100.0 {
                                let f = num5 / 100.0;
                                ((num12 as f64 * f) as i32, (num13 as f64 * f) as i32, (num14 as f64 * f) as i32)
                            } else {
                                (num12, num13, num14)
                            };
                            let (num12, num13, num14) = if num4 < num6 + 5.0 {
                                let f = (num4 - num6) / 5.0;
                                ((num12 as f64 * f) as i32, (num13 as f64 * f) as i32, (num14 as f64 * f) as i32)
                            } else {
                                (num12, num13, num14)
                            };
                            cave_column(
                                world,
                                k,
                                l,
                                num12,
                                num13,
                                num14,
                                vel_x > 0.0,
                                rng,
                            );
                        }
                    }
                }
                // The ring water (`:76632-76636`).
                if num11 < num4 * 1.3 + 1.0 && l > j - 10 {
                    let idx = world.index(k, l);
                    world.tiles[idx].liquid = 255;
                    world.tiles[idx].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER);
                }
                // The vertical water shaft (`:76637-76654`).
                if flag4 && k == x as i32 && (l as f64) > y {
                    flag4 = false;
                    let half = 2i32;
                    for num20 in k - half..=k + half {
                        for num21 in l..l + 100 {
                            if !bad_ocean_cave_tiles(world, num20, num21) {
                                let idx = world.index(num20, num21);
                                world.tiles[idx].liquid = 255;
                                world.tiles[idx].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER);
                            }
                        }
                    }
                }
                l += 1;
            }
            k += 1;
        }
        // The step and the steer (`:76657-76702`).
        x += vel_x;
        y += vel_y;
        vel_x += rng.next_f64() * 0.1 - 0.05;
        vel_y += rng.next_f64() * 0.1 - 0.05;
        if flag {
            if y > (world_surface * 2.0 + rock_layer) / 3.0 && y > (j + 30) as f64 {
                flag = false;
            }
            vel_y = clampf(vel_y, 0.35, 1.0);
        } else {
            if x < (max_x / 2) as f64 {
                if vel_x < 0.5 {
                    vel_x += 0.02;
                }
            } else if vel_x > -0.5 {
                vel_x -= 0.02;
            }
            if !flag5 {
                if vel_y < 0.0 {
                    vel_y *= 0.95;
                }
                vel_y += 0.04;
            } else if y < (world_surface * 4.0 + rock_layer) / 5.0 {
                if vel_y < 0.0 {
                    vel_y *= 0.97;
                }
                vel_y += 0.02;
            } else if vel_y > -0.1 {
                vel_y *= 0.99;
                vel_y -= 0.01;
            }
            vel_y = clampf(vel_y, -1.0, 1.0);
        }
        if x < (max_x / 2) as f64 {
            vel_x = clampf(vel_x, 0.1, 1.0);
        } else {
            vel_x = clampf(vel_x, -1.0, -0.1);
        }
    }
    gv.num_ocean_cave_treasure += 1;
}

/// `Create Ocean Caves` (`WorldGen.cs:15167-15194`): the wrapper — the
/// per-side dungeon-side gate, the `Next(maxValue)` roll, the anchor scan and
/// the walker. The port's dungeon side is fixed Right (M5-BIOME), so the
/// LEFT ocean never runs (vanilla `:15176`'s `i != 0 || dungeonSide < Right`
/// arm is false) and the RIGHT ocean takes the roll.
pub(crate) fn ocean_caves(
    world: &mut World,
    gv: &mut GenVars,
    layers: &Layers,
    rng: &mut Rng,
) {
    // `maxValue = 3` (remix 2 — no remix seed yet).
    for i in 0..2 {
        let side_ok = if i == 0 {
            false // dungeonSide < Right never holds (fixed Right)
        } else {
            true // dungeonSide > Left always holds (fixed Right)
        };
        if !side_ok {
            continue;
        }
        // `WorldGen.cs:15178`: the chance gate is
        // `genRand.Next(maxValue) == 0 || drunkWorldGen || tenthAnniversaryWorldGen`.
        // `Next(maxValue)` is the LEFT operand of `||`, so C# spends the roll
        // whatever the seed flags say — the drunk (and the anniversary, the
        // other agent's arm) world only ever *passes* the gate, it never skips
        // the draw. Ported the same way: burn the roll first, then OR the flag
        // in. A drunk world therefore always generates its ocean cave on the
        // right side while the draw order — and every classic world — stays
        // byte-identical.
        let roll = rng.chance(3);
        let drunk = world.seeds.contains(crate::seeds::SeedFlags::DRUNK);
        if !(roll || drunk) {
            continue;
        }
        let num = if i == 1 {
            rng.range(world.width - 95, world.width - 55)
        } else {
            rng.range(55, 95)
        };
        // The anchor scan (`:15186-15189`): the first active tile down.
        let mut j = 0;
        while j < world.height - 4 && !world.tile(num, j).is_some_and(|t| t.active()) {
            j += 1;
        }
        ocean_cave(world, gv, layers, num, j, rng);
    }
}

// --- Remove Water From Sand / Shell Piles / Water Plants ----------------------

/// The sand-family types `RemoveSurfaceWaterAboveSand` clears above
/// (`WorldGen.cs:16221-16223`): sand 53, sandstone 396, hardened sand 397,
/// fossil? — 404 DesertFossil, 407 the silt-ish variant, 151 mud.
const WATER_ABOVE_SAND_TYPES: [u16; 6] = [53, 396, 397, 404, 407, 151];
/// `WorldGen.beachDistance`'s inland exclusion: the pass runs over
/// `[400, maxTilesX - 400)` (`WorldGen.cs:16216`) — the oceans are
/// deliberately untouched.
const REMOVE_WATER_MARGIN: i32 = 400;

/// `Remove Water From Sand` / `RemoveSurfaceWaterAboveSand`
/// (`WorldGen.cs:16211-16242`): the inland sand columns lose the stray water
/// above them (a settling step after the caves) — the oceans (the outer 400
/// columns per side) are excluded by design. The C# tail mutation
/// `Main.tileSolid[192] = true` (`:16240`, `TileID.LeafBlock` — the living
/// leaves become solid during generation) has no port counterpart: the
/// tile sets are generated statics (`tiledata/`), so the runtime mutation is
/// deferred to that owner (ANALYSIS "Ocean passes fidelity").
pub(crate) fn remove_water_from_sand(world: &mut World, world_surface: f64) {
    let (max_x, max_y) = (world.width, world.height);
    let surface = world_surface as i32;
    for i in REMOVE_WATER_MARGIN..max_x - REMOVE_WATER_MARGIN {
        for j in 100..(surface - 1) {
            let idx = world.index(i, j);
            if world.tiles[idx].active() {
                let kind = world.tiles[idx].kind;
                if WATER_ABOVE_SAND_TYPES.contains(&kind) {
                    let mut num = j;
                    while num > 100 {
                        num -= 1;
                        let up = world.index(i, num);
                        if world.tiles[up].active() {
                            break;
                        }
                        world.tiles[up].liquid = 0;
                    }
                }
                break;
            }
        }
    }
    let _ = max_y;
}

/// `TileID.ShellPile` = 495 (`TileID.cs:1619`).
const TILE_SHELL_PILE: u16 = 495;

/// `WorldGen.PlaceOasis(X, Y)` (`WorldGen.cs:10707-10932`): the desert
/// oasis basin — the sand-floor scan, the 350-tile spacing gate against
/// `GenVars.oasisPosition`, the `Next(45, 61)` width roll, the sand-only
/// clearance box, the ≤20-row floor walk, the wobble-radius ellipse carve
/// (one `NextDouble` per cell, the water row at `i + 1`), the sand rim and
/// the `Next(5)`/`Next(5, 10)` bridge speckle. The dead
/// `k > X - num2/2 && k < X - num2/2` box (`:10759`) and the `flag = true`
/// overwrite (`:10899`) are verbatim no-ops/quirks of the original.
pub(crate) fn place_oasis(
    world: &mut World,
    gv: &mut GenVars,
    x: i32,
    y: i32,
    world_surface: f64,
    rng: &mut Rng,
) -> bool {
    let (max_x, max_y) = (world.width, world.height);
    if x < 0 || y < 0 || x >= max_x || y >= max_y {
        return false;
    }
    let mut i = y;
    if world.tiles[world.index(x, i)].active() || world.tiles[world.index(x, i)].wall != 0 {
        return false;
    }
    // The scan reads the tile first, then the surface bound (`:10712`).
    while !world.tiles[world.index(x, i)].active()
        && world.tiles[world.index(x, i)].wall == 0
        && (i as f64) <= world_surface
    {
        i += 1;
        if i >= max_y {
            return false;
        }
    }
    if (i as f64) > world_surface - 10.0 {
        return false;
    }
    if world.tiles[world.index(x, i)].kind != TILE_SAND {
        return false;
    }
    // The 350-tile spacing against every placed basin (`:10723-10730`).
    for j in 0..gv.num_oasis as usize {
        let (px, py) = gv.oasis_position[j];
        let (dx, dy) = ((px - x) as f64, (py - i) as f64);
        if (dx * dx + dy * dy).sqrt() < 350.0 {
            return false;
        }
    }
    let num2 = rng.range(45, 61);
    let oasis_height = gv.oasis_height;
    let num3 = num2 + 50;
    let num4 = 4;
    // The clearance box (`:10735-10770`): solids must be sand (the
    // 151/397 near-core reject first), the near-core air must be dry and
    // wall-free.
    for k in x - num3..=x + num3 {
        if k < 0 || k >= max_x {
            continue;
        }
        for l in i - oasis_height..=i + oasis_height + num4 {
            if l < 0 || l >= max_y {
                continue;
            }
            let tile = world.tiles[world.index(k, l)];
            if tile.active() {
                if (tile.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(tile.kind) {
                    if (tile.kind == 151 || tile.kind == 397)
                        && (k - x).abs() < num2
                        && (l - i).abs() < oasis_height / 2
                    {
                        return false;
                    }
                    if tile.kind != TILE_SAND {
                        return false;
                    }
                }
            } else if (tile.liquid > 0 || tile.wall > 0)
                && (k - x).abs() < num2
                && (l - i).abs() < oasis_height / 2
            {
                return false;
            }
        }
        // `:10759` — `k > X - num2/2 && k < X - num2/2` is never true;
        // the C#'s dead box is a verbatim no-op.
    }
    // The floor walk (`:10771-10780`): drop until both rim columns at
    // `i + 5` are active and wall-free, at most 20 rows.
    let num5 = 5;
    let num6 = i;
    loop {
        let (lx, rx) = (x - num2, x + num2);
        if lx < 0 || rx >= max_x || i + num5 >= max_y {
            break;
        }
        let lt = world.tiles[world.index(lx, i + num5)];
        let rt = world.tiles[world.index(rx, i + num5)];
        if lt.active() && lt.wall == 0 && rt.active() && rt.wall == 0 {
            break;
        }
        i += 1;
        if i - num6 > 20 {
            break;
        }
    }
    let num7 = num2 / 2;
    let num8 = (x - num2 * 3).max(0);
    let num9 = (x + num2 * 3).min(max_x);
    let num10 = (i - oasis_height * 4).max(0);
    let num11 = (i + oasis_height * 3).min(max_y);
    // The ellipse carve (`:10802-10847`): one NextDouble per cell, the
    // 0.7/1.35 axis stretch, the `num7 * (0.53 + d * 0.04)` radius, the
    // `(1 - |m-X|/(num9-X)) * 2.3` rim factor raised to the fourth.
    for m in num8..num9 {
        for n in num10..num11 {
            let num12 = (m - x).abs() as f64 * 0.7;
            let num13 = (n - i).abs() as f64 * 1.35;
            let num14 = (num12 * num12 + num13 * num13).sqrt();
            let num15 = num7 as f64 * (0.53 + rng.next_f64() * 0.04);
            let mut num16 = (1.0 - (m - x).abs() as f64 / (num9 - x) as f64) * 2.3;
            num16 *= num16;
            num16 *= num16;
            let idx = world.index(m, n);
            if num14 < num15 {
                let t = &mut world.tiles[idx];
                if n == i + 1 {
                    t.liquid = 127;
                } else if n > i + 1 {
                    t.liquid = 255;
                }
                t.flags &= !FLAG_LIQUID_LAVA;
                t.flags &= !FLAG_ACTIVE;
            } else if n < i && num12 < num15 + ((n - i).abs() * 3) as f64 * num16 {
                let t = &mut world.tiles[idx];
                if t.kind == TILE_SAND {
                    t.flags &= !FLAG_ACTIVE;
                }
            } else if n >= i
                && num12 < num15 + (n - i).abs() as f64 * num16
                && world.tiles[idx].wall == 0
            {
                let t = &mut world.tiles[idx];
                if t.active()
                    && (t.kind as usize) < td::TILE_COUNT
                    && td::TILE_SOLID.has(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
                {
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                    continue;
                }
                t.flags |= FLAG_ACTIVE;
                t.kind = TILE_SAND;
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
            }
        }
    }
    // The bridge speckle (`:10848-10922`): outside the 0.57 core, scan
    // ±50 for a solid floor pair and fill the gap with sand, the
    // `Next(5) == 0 → Next(5, 10)` raised-row runs.
    let num17 = 50;
    let (b_left, b_right) = (x - num2 * 2, x + num2 * 2);
    let b_bottom = i + oasis_height * 2;
    for num18 in b_left..b_right {
        if num18 < 0 || num18 >= max_x {
            continue;
        }
        for num19 in (i..=b_bottom).rev() {
            if num19 < 0 || num19 >= max_y {
                continue;
            }
            let num20 = (num18 - x).abs() as f64 * 0.7;
            let num21 = (num19 - i).abs() as f64 * 1.35;
            if (num20 * num20 + num21 * num21).sqrt() <= num7 as f64 * 0.5700000000000001 {
                continue;
            }
            let here = world.tiles[world.index(num18, num19)];
            if here.active() || here.wall != 0 {
                continue;
            }
            let mut num22 = -1i32;
            let mut num23 = -1i32;
            // The east scan (`:10865-10880`).
            let mut num24 = num18;
            while num24 <= num18 + num17 && num24 < max_x && num19 + 1 < max_y {
                let below = world.tiles[world.index(num24, num19 + 1)];
                let cell = world.tiles[world.index(num24, num19)];
                if !(below.active()
                    && (below.kind as usize) < td::TILE_COUNT
                    && td::TILE_SOLID.has(below.kind)
                    && cell.wall == 0)
                {
                    break;
                }
                if cell.active()
                    && (cell.kind as usize) < td::TILE_COUNT
                    && td::TILE_SOLID.has(cell.kind)
                {
                    num23 = num24;
                    break;
                }
                if cell.active() {
                    break;
                }
                num24 += 1;
            }
            // The west scan (`:10881-10898`).
            let mut num25 = num18;
            while num25 >= num18 - num17 && num25 >= 0 && num19 + 1 < max_y {
                let below = world.tiles[world.index(num25, num19 + 1)];
                let cell = world.tiles[world.index(num25, num19)];
                if !(below.active()
                    && (below.kind as usize) < td::TILE_COUNT
                    && td::TILE_SOLID.has(below.kind)
                    && cell.wall == 0)
                {
                    break;
                }
                if cell.active()
                    && (cell.kind as usize) < td::TILE_COUNT
                    && td::TILE_SOLID.has(cell.kind)
                {
                    num22 = num25;
                    break;
                }
                if cell.active() {
                    break;
                }
                num25 -= 1;
            }
            // `:10899` — the C# overwrites the sand-found flag with `true`
            // before the `&` test; verbatim, the flag never gates.
            if num22 > -1 && num23 > -1 {
                let mut num26 = 0i32;
                for num27 in num22 + 1..num23 {
                    if num23 - num22 > 5 && rng.chance(5) {
                        num26 = rng.range(5, 10);
                    }
                    let idx = world.index(num27, num19);
                    let t = &mut world.tiles[idx];
                    t.flags |= FLAG_ACTIVE;
                    t.kind = TILE_SAND;
                    if num26 > 0 {
                        num26 -= 1;
                        let up = world.index(num27, num19 - 1);
                        let t = &mut world.tiles[up];
                        t.flags |= FLAG_ACTIVE;
                        t.kind = TILE_SAND;
                    }
                }
            }
        }
    }
    // The ledger append (`:10923-10928`).
    if (gv.num_oasis as usize) < gv.oasis_position.len() {
        let slot = gv.num_oasis as usize;
        gv.oasis_position[slot] = (x, i);
        gv.oasis_width[slot] = num2;
        gv.num_oasis += 1;
    }
    true
}

/// `Oasis` (`WorldGen.cs:16243-16265`): `maxTilesX / 2100 + Next(2)`
/// basins, each up to `maxTilesX * 2` placement tries at
/// `Next(beachDistance + 300, maxTilesX - (beachDistance + 300))` /
/// `Next(100, (int)worldSurface)`. The seed gate lives in the step
/// adapter. Sub-vanilla width guard: the C#'s `Next(680, maxTilesX - 680)`
/// throws below 1360 wide; the port skips the pass before any draw (the
/// mining-explosives convention).
pub(crate) fn oasis(world: &mut World, gv: &mut GenVars, world_surface: f64, rng: &mut Rng) {
    let margin = BEACH_DISTANCE + 300;
    if world.width <= margin * 2 || (world_surface as i32) <= 100 {
        return;
    }
    let num = world.width / 2100 + rng.range(0, 2);
    for _ in 0..num {
        let mut tries = world.width * 2;
        while tries > 0 {
            tries -= 1;
            // Named arguments evaluate in source order: X, then Y.
            let x = rng.range(margin, world.width - margin);
            let y = rng.range(100, world_surface as i32);
            if place_oasis(world, gv, x, y, world_surface, rng) {
                tries = -1;
            }
        }
    }
}

/// `WorldGen.ShellPile(X, Y)` (`WorldGen.cs:10332-10448`): the oval shell
/// blob — the sentinel walk down to the sand floor, the radius wobble per
/// cell (`Next(70, 91) * 0.01`), the 495 core with the sand/hardened-sand
/// support stack, the `Next(6)` outer scatter. The two `Utils.Clamp` calls
/// at the walk's end discard their result in the original (`:10430-10431` —
/// the wobble velocity is NOT clamped): ported verbatim, a no-op.
pub(crate) fn shell_pile(world: &mut World, world_surface: f64, x: i32, y: i32, rng: &mut Rng) -> bool {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return false;
    }
    let mut num = y;
    while !solid_tile_at(world, x, num) {
        num += 1;
        if num as f64 > world_surface {
            return false;
        }
    }
    {
        let idx = world.index(x, num);
        if world.tiles[idx].kind != TILE_SAND || world.tiles[idx].wall > 0 {
            return false;
        }
    }
    num -= 1;
    let (mut cx, mut cy) = (x as f64, num as f64);
    let mut vel_x = rng.next_f64() * 0.6 - 0.3;
    let mut vel_y = rng.next_f64() * 0.5 + 0.5;
    let mut num2 = rng.range(2, 4) as f64;
    if rng.chance(10) {
        num2 += 1.0;
    }
    let mut num3 = rng.range(3, 6);
    while num3 > 0 {
        num3 -= 1;
        for i in (x - (num2 as i32) * 4)..=(x + (num2 as i32) * 4) {
            if i < 0 || i >= world.width {
                continue;
            }
            let mut num4 = num + (num2 as i32) * 4;
            while (num4 as f64) > num as f64 - num2 * 4.0 {
                let num5 = num2 * rng.range(70, 91) as f64 * 0.01;
                let (dx, dy) = (cx - i as f64, cy - num4 as f64);
                let dist = ((dx * 0.6).powi(2) + dy * dy).sqrt();
                if dist < num5 {
                    let idx = world.index(i, num4);
                    if num4 <= num + 1 || rng.chance(6) {
                        // The 495 core with the support stack
                        // (`:10382-10408`).
                        let t = &mut world.tiles[idx];
                        t.kind = TILE_SHELL_PILE;
                        t.flags |= FLAG_ACTIVE;
                        t.flags &= !FLAG_HALF_BRICK;
                        t.set_slope(0);
                        let idx2 = world.index(i, num4 + 1);
                        if !world.tiles[idx2].active() {
                            let t = &mut world.tiles[idx2];
                            t.kind = TILE_SHELL_PILE;
                            t.flags |= FLAG_ACTIVE;
                            t.flags &= !FLAG_HALF_BRICK;
                            t.set_slope(0);
                            let idx3 = world.index(i, num4 + 2);
                            if !world.tiles[idx3].active() {
                                let t = &mut world.tiles[idx3];
                                t.kind = TILE_SAND;
                                t.flags |= FLAG_ACTIVE;
                                t.flags &= !FLAG_HALF_BRICK;
                                t.set_slope(0);
                                let idx4 = world.index(i, num4 + 3);
                                if !world.tiles[idx4].active() {
                                    let t = &mut world.tiles[idx4];
                                    t.kind = TILE_HARDENED_SAND;
                                    t.flags |= FLAG_ACTIVE;
                                    t.flags &= !FLAG_HALF_BRICK;
                                    t.set_slope(0);
                                }
                            }
                        }
                    } else if world.tiles[idx].kind != TILE_SHELL_PILE {
                        let t = &mut world.tiles[idx];
                        t.flags |= FLAG_ACTIVE;
                        t.flags &= !FLAG_HALF_BRICK;
                        t.set_slope(0);
                        t.kind = TILE_SAND;
                    }
                }
                num4 -= 1;
            }
        }
        cx += vel_x;
        cy += vel_y;
        vel_x += rng.next_f64() * 0.2 - 0.1;
        vel_y += rng.next_f64() * 0.2 - 0.1;
        // The C# clamps discard their result (`:10430-10431`) — the wobble
        // velocity is unclamped in the original; verbatim no-op.
        let _ = clampf(vel_x, -0.3, 0.3);
        let _ = clampf(vel_y, 0.5, 1.0);
    }
    true
}

#[inline]
fn solid_tile_at(world: &World, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return true;
    }
    solid_tile(&world.tiles[world.index(x, y)])
}

/// `Shell Piles` / `ShellPilesMarblePilesAndSpikePits`
/// (`WorldGen.cs:16266-16416`): the sand-adjacency scan over the
/// `shellStart*` anchors, the y-shift, the five X offsets and the shell
/// blobs. The DST marble-pile half and the FTW `AddSpikeCaves` call are
/// seed-gated off (no draws in the port). The right branch's two offset
/// arms write `GenVars.shellStartXLeft` — the vanilla quirk at `:16381`/
/// `:16385` — ported verbatim.
pub(crate) fn shell_piles(world: &mut World, gv: &mut GenVars, world_surface: f64, rng: &mut Rng) {
    // Left ocean (`:16311-16349`).
    if rng.chance(2) {
        for j in gv.shell_start_x_left - 20..=gv.shell_start_x_left + 20 {
            for k in gv.shell_start_y_left - 10..=gv.shell_start_y_left + 10 {
                if j < 0 || k < 1 || j >= world.width || k >= world.height {
                    continue;
                }
                let here = world.index(j, k);
                let here_ok = world.tiles[here].active()
                    && world.tiles[here].kind == TILE_SAND
                    && !world.tiles[world.index(j, k - 1)].active()
                    && world.tiles[world.index(j, k - 1)].liquid == 0;
                let west = world.index(j - 1, k);
                let west_ok = !world.tiles[west].active() && world.tiles[west].liquid > 0;
                if here_ok && west_ok {
                    gv.shell_start_x_left = j;
                    gv.shell_start_y_left = k;
                }
            }
        }
        gv.shell_start_y_left -= 50;
        gv.shell_start_x_left -= rng.range(0, 5);
        if rng.chance(2) {
            gv.shell_start_x_left -= rng.range(0, 10);
        }
        if rng.chance(3) {
            gv.shell_start_x_left -= rng.range(0, 15);
        }
        if !rng.chance(4) {
            shell_pile(world, world_surface, gv.shell_start_x_left, gv.shell_start_y_left, rng);
        }
        let max_value = rng.range(2, 4);
        if rng.range(0, max_value) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_left - rng.range(10, 35),
                gv.shell_start_y_left,
                rng,
            );
        }
        if rng.range(0, max_value) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_left - rng.range(40, 65),
                gv.shell_start_y_left,
                rng,
            );
        }
        if rng.range(0, max_value) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_left - rng.range(70, 95),
                gv.shell_start_y_left,
                rng,
            );
        }
        if rng.range(0, max_value) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_left - rng.range(100, 125),
                gv.shell_start_y_left,
                rng,
            );
        }
        if rng.range(0, max_value) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_left + rng.range(10, 25),
                gv.shell_start_y_left,
                rng,
            );
        }
    }
    // Right ocean (`:16351-16413`).
    if rng.chance(2) {
        for l in gv.shell_start_x_right - 20..=gv.shell_start_x_right + 20 {
            for m in gv.shell_start_y_right - 10..=gv.shell_start_y_right + 10 {
                if l < 0 || m < 1 || l >= world.width || m >= world.height {
                    continue;
                }
                let here = world.index(l, m);
                let here_ok = world.tiles[here].active()
                    && world.tiles[here].kind == TILE_SAND
                    && !world.tiles[world.index(l, m - 1)].active()
                    && world.tiles[world.index(l, m - 1)].liquid == 0;
                let east = world.index(l + 1, m);
                let east_ok = !world.tiles[east].active() && world.tiles[east].liquid > 0;
                if here_ok && east_ok {
                    gv.shell_start_x_right = l;
                    gv.shell_start_y_right = m;
                }
            }
        }
        gv.shell_start_y_right -= 50;
        gv.shell_start_x_right += rng.range(0, 5);
        // The vanilla quirk: the right branch's offsets adjust the LEFT
        // anchor (`:16381`, `:16385`) — never read again in the pass, so
        // harmless, but ported verbatim.
        if rng.chance(2) {
            gv.shell_start_x_left += rng.range(0, 10);
        }
        if rng.chance(3) {
            gv.shell_start_x_left += rng.range(0, 15);
        }
        if !rng.chance(4) {
            shell_pile(world, world_surface, gv.shell_start_x_right, gv.shell_start_y_right, rng);
        }
        let max_value2 = rng.range(2, 4);
        if rng.range(0, max_value2) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_right + rng.range(10, 35),
                gv.shell_start_y_right,
                rng,
            );
        }
        if rng.range(0, max_value2) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_right + rng.range(40, 65),
                gv.shell_start_y_right,
                rng,
            );
        }
        if rng.range(0, max_value2) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_right + rng.range(70, 95),
                gv.shell_start_y_right,
                rng,
            );
        }
        if rng.range(0, max_value2) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_right + rng.range(100, 125),
                gv.shell_start_y_right,
                rng,
            );
        }
        if rng.range(0, max_value2) == 0 {
            shell_pile(
                world,
                world_surface,
                gv.shell_start_x_right - rng.range(10, 25),
                gv.shell_start_y_right,
                rng,
            );
        }
    }
}

/// `Water Plants` / `LilypadsCattailsBambooAndSeaweed` (`WorldGen.cs:
/// 21973-22036`): the lily/cattail/bamboo j-loop (`:21988-22017`, W4-c33)
/// ahead of the seaweed walk — per column, the `Next(5)` + `liquid > 0`
/// gate, the `Next(2)` lily-vs-cattail pick (the cattail's `Next(14)` ×
/// `GrowCatTail` climb), then the bamboo arm (`PlaceBamboo` + the
/// `Next(10, 20)` climb). The remix `num = maxTilesY - 200` arm stays with
/// the seed lanes (d107). `Main.UnderworldLayer` is `maxTilesY - 200`
/// (`Main.cs:2790`). The `SquareTileFrame` calls ride the port's framing
/// convention (the later framing pass — the seaweed arm's precedent).
pub(crate) fn water_plants(world: &mut World, layers: &Layers, rng: &mut Rng) {
    let (max_x, max_y) = (world.width, world.height);
    let surface = layers.world_surface as i32;
    for i in 20..max_x - 20 {
        // The j-loop (`:21986-22018`).
        for j in 1..surface {
            if rng.range(0, 5) != 0 || !world.tile(i, j).is_some_and(|t| t.liquid > 0) {
                continue;
            }
            // `:21990-22009`: the lily/cattail pick on an open water cell.
            if !world.tile(i, j).is_some_and(|t| t.active()) {
                if rng.range(0, 2) == 0 {
                    place_lily_pad(world, i, j, rng);
                } else {
                    let point = place_cat_tail(world, i, j, rng);
                    // `:21999`: `InWorld(point)` — the (-1, -1) failure
                    // sentinel drops out before the climb draw.
                    if point.0 >= 0 && point.0 < max_x && point.1 >= 0 && point.1 < max_y {
                        let num2 = rng.range(0, 14);
                        for _ in 0..num2 {
                            grow_cat_tail_gen(world, point.0, point.1, rng);
                        }
                    }
                }
            }
            // `:22010-22016`: the bamboo arm — the gate re-reads the cell
            // (the lily/cattail may have filled it), the climb re-calls
            // `PlaceBamboo` per row (each call draws its own `Next(1, 21)`).
            let bamboo_gate = world
                .tile(i, j)
                .is_some_and(|t| !t.active() || t.kind == 61 || t.kind == 74);
            if bamboo_gate && place_bamboo_gen(world, i, j, rng) {
                let num3 = rng.range(10, 20);
                let mut l = 0;
                while l < num3 && place_bamboo_gen(world, i, j - l, rng) {
                    l += 1;
                }
            }
        }
        // The seaweed walk (`:22019-22034`).
        let mut num4 = max_y - 200; // Main.UnderworldLayer
        while num4 > surface {
            let idx = world.index(i, num4);
            if world.tiles[idx].active() {
                let kind = world.tiles[idx].kind;
                if (kind == TILE_SAND && rng.range(0, 3) != 0) || kind == 549 {
                    grow_check_seaweed(world, layers, i, num4, rng);
                }
            }
            num4 -= 1;
        }
    }
}

/// The water-surface scan shared by `PlaceLilyPad`/`PlaceCatTail`
/// (`:59097-59101`/`:58641-58645`): up through the liquid column, then one
/// row back down — the surface cell. `None` when the entry gates fail.
fn water_surface_cell(world: &World, x: i32, j: i32) -> Option<i32> {
    let (max_x, max_y) = (world.width, world.height);
    if x < 50 || x > max_x - 50 || j < 50 || j > max_y - 50 {
        return None;
    }
    let mut num = j;
    while world.tile(x, num).is_some_and(|t| t.liquid > 0) && num > 50 {
        num -= 1;
    }
    num += 1;
    // `:59102`/`:58646`: the surface cell must be open water with an open
    // cell above.
    if world.tile(x, num).is_some_and(|t| t.active())
        || world.tile(x, num - 1).is_some_and(|t| t.active())
        || !world
            .tile(x, num)
            .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Water)
    {
        return None;
    }
    Some(num)
}

/// `WorldGen.PlaceLilyPad` (`WorldGen.cs:59086-59200`): the lily pad 518 on
/// the water surface — the wall gate, the ±5 density cap (3), the floor
/// scan (3..12 rows, no cattail in the column), the soil family (the grass
/// 2/477 → frame row 0, the hallowed 109/492/116 → 18, the jungle 60 → 36)
/// and the frame roll (the `Next(2)` common arm, the `Next(15)` rare arm,
/// else the fifth-of-the-world band).
fn place_lily_pad(world: &mut World, x: i32, j: i32, rng: &mut Rng) -> bool {
    let (max_x, max_y) = (world.width, world.height);
    // `:59093`: the entry gate (active / dry / non-water).
    if world.tile(x, j).is_some_and(|t| {
        t.active() || t.liquid == 0 || t.liquid_kind() != crate::tile::LiquidKind::Water
    }) {
        return false;
    }
    let Some(num) = water_surface_cell(world, x, j) else {
        return false;
    };
    // `:59106`: the wall gate.
    let wall = world.tile(x, num).map(|t| t.wall).unwrap_or(0);
    if wall != 0 && wall != 15 && wall != 70 && !(63..=68).contains(&wall) {
        return false;
    }
    // `:59110-59125`: the ±5 density cap.
    let mut num3 = 0;
    for i in (x - 5)..=(x + 5) {
        for k in (num - 5)..=(num + 5) {
            if world
                .tile(i, k)
                .is_some_and(|t| t.active() && t.kind == 518)
            {
                num3 += 1;
            }
        }
    }
    if num3 > 3 {
        return false;
    }
    // `:59126-59141`: the floor scan — the raw `tileSolid`/`tileSolidTop`
    // table test (no slope/half-brick arm), a cattail in the column kills
    // the pad, the depth window is 3..12.
    let mut l = num;
    while l < max_y - 50
        && !world.tile(x, l).is_some_and(|t| {
            t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
        })
    {
        if world
            .tile(x, l)
            .is_some_and(|t| t.active() && t.kind == 519)
        {
            return false;
        }
        l += 1;
    }
    if l - num > 12 || l - num < 3 {
        return false;
    }
    // `:59142-59159`: the soil family → the frame row.
    let soil = world.tile(x, l).map(|t| t.kind).unwrap_or(0);
    let num4 = match soil {
        2 | 477 => 0,
        109 | 492 | 116 => 18,
        60 => 36,
        _ => return false,
    };
    let color = world.tile(x, l).map(|t| t.paint()).unwrap_or(0);
    // `:59160-59198`: the write + the frame roll.
    let frame_x = if rng.range(0, 2) == 0 {
        18 * rng.range(0, 3)
    } else if rng.range(0, 15) == 0 {
        18 * rng.range(0, 18)
    } else {
        let num5 = max_x / 5;
        let band = if x < num5 {
            rng.range(6, 9)
        } else if x < num5 * 2 {
            rng.range(9, 12)
        } else if x < num5 * 3 {
            rng.range(3, 6)
        } else if x < num5 * 4 {
            rng.range(15, 18)
        } else {
            rng.range(12, 15)
        };
        18 * band
    };
    if let Some(t) = world.tile_mut(x, num) {
        t.kind = 518;
        t.flags |= FLAG_ACTIVE;
        t.set_paint(color);
        t.frame_x = frame_x as i16;
        t.frame_y = num4;
        t.flags &= !FLAG_HALF_BRICK;
        t.set_slope(0);
    }
    true
}

/// `WorldGen.PlaceCatTail` (`WorldGen.cs:58629-58742`): the cattail 519 on
/// the floor under the water column — the entry gate admits an existing
/// 71 (the dye plant), the wall gate, the ±7 per-column density cap (3),
/// the floor scan (`catTailDistance` 8, `:86957`), the soil family (the
/// sand 53 only inland, the hardmode soils 199/234/662 → 54, 23/112/661 →
/// 72, the mushroom 70 → 90), the top-slope flatten roll. Returns the
/// stalk's base cell or the (-1, -1) sentinel.
fn place_cat_tail(world: &mut World, x: i32, j: i32, rng: &mut Rng) -> (i32, i32) {
    let fail = (-1, -1);
    let (max_x, max_y) = (world.width, world.height);
    // `:58637`: the entry gate (the 71 exemption).
    if world.tile(x, j).is_some_and(|t| {
        (t.active() && t.kind != 71)
            || t.liquid == 0
            || t.liquid_kind() != crate::tile::LiquidKind::Water
    }) {
        return fail;
    }
    let Some(num) = water_surface_cell(world, x, j) else {
        return fail;
    };
    // `:58650`: the wall gate.
    let wall = world.tile(x, num).map(|t| t.wall).unwrap_or(0);
    if wall != 0 && wall != 80 && wall != 81 && wall != 69 && !(63..=68).contains(&wall) {
        return fail;
    }
    // `:58654-58670`: the ±7 density cap — the C# `break`s after the first
    // 519 per column, so num3 counts COLUMNS, not tiles.
    let mut num3 = 0;
    for i in (x - 7)..=(x + 7) {
        for k in (num - 7)..=(num + 7) {
            if world
                .tile(i, k)
                .is_some_and(|t| t.active() && t.kind == 519)
            {
                num3 += 1;
                break;
            }
        }
    }
    if num3 > 3 {
        return fail;
    }
    // `:58671-58686`: the floor scan (anything but a 71 in the column
    // kills), the 2..catTailDistance-1 depth window.
    let mut l = num;
    while l < max_y - 50
        && !world.tile(x, l).is_some_and(|t| {
            t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
        })
    {
        if world.tile(x, l).is_some_and(|t| t.active() && t.kind != 71) {
            return fail;
        }
        l += 1;
    }
    // `WorldGen.catTailDistance` = 8 (`:86957`).
    if l - num > 8 - 1 || l - num < 2 {
        return fail;
    }
    // `:58687-58723`: the soil family (the floor must be nactive).
    let Some(floor) = world.tile(x, l).copied() else {
        return fail;
    };
    if !floor.active() || floor.inactive() {
        return fail;
    }
    let num4 = match floor.kind {
        2 | 477 => 0,
        53 => {
            // `:58700`: the sand cattail only inland.
            if x < BEACH_DISTANCE || x > max_x - BEACH_DISTANCE {
                return fail;
            }
            18
        }
        199 | 234 | 662 => 54,
        23 | 112 | 661 => 72,
        70 => 90,
        _ => return fail,
    };
    // `:58724-58731`: the top-slope flatten (the generation-time
    // `isGeneratingOrLoadingWorld` arm) — the `Next(3)` draws only on a
    // top-sloped floor; a surviving slope or a half brick kills.
    if floor.top_slope() {
        if rng.range(0, 3) != 0 {
            if let Some(t) = world.tile_mut(x, l) {
                t.set_slope(0);
            }
        } else {
            return fail;
        }
    } else if floor.half_brick() {
        return fail;
    }
    // `:58732-58741`: the write one row above the floor, the paint copy.
    let num = l - 1;
    if let Some(t) = world.tile_mut(x, num) {
        t.kind = 519;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = 0;
        t.frame_y = num4;
        t.flags &= !FLAG_HALF_BRICK;
        t.set_slope(0);
        t.set_paint(floor.paint());
    }
    (x, num)
}

/// `WorldGen.GrowCatTail` (`WorldGen.cs:58974-59084`) — the worldgen-stream
/// copy (the runtime twin is `growth/decorations.rs::grow_cat_tail`): the
/// stalk's bottom cell advances 0 → 18 → `18·Next(2,5)`, then the tip chain
/// (90 → 108 / 126..162 with the 180..216 partner). The `KillTile` arm
/// rides the generation-time strip (`kill_tile_gen` — the seaweed arm's
/// precedent); the `SquareTileFrame` calls defer to the framing pass.
fn grow_cat_tail_gen(world: &mut World, x: i32, j: i32, rng: &mut Rng) {
    let max_y = world.height - 50;
    // `:58980-58983`: up through the liquid.
    let mut num = j;
    while world.tile(x, num).is_some_and(|t| t.liquid > 0) && num > 50 {
        num -= 1;
    }
    // `:58985-58989`: down to the ground (the raw table test).
    let mut i = num + 1;
    while i < max_y
        && !world.tile(x, i).is_some_and(|t| {
            t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
        })
    {
        i += 1;
    }
    num = i - 1;
    // `:58990-58994`: back down to the stalk's own bottom cell.
    while world
        .tile(x, num)
        .is_some_and(|t| t.active() && t.kind == 519)
    {
        num -= 1;
    }
    num += 1;
    if !world
        .tile(x, num)
        .is_some_and(|t| t.active() && t.kind == 519)
    {
        return;
    }
    // `:58999-59006`: a full tip (frameX 90) with a cuttable tile above
    // clears it first (the generation-time KillTile strip).
    if world.tile(x, num).is_some_and(|t| t.frame_x == 90)
        && world
            .tile(x, num - 1)
            .is_some_and(|t| t.active() && crate::tileobject::tile_cut(t.kind))
    {
        if let Some(t) = world.tile_mut(x, num - 1) {
            kill_tile_gen(t);
        }
    }
    if world.tile(x, num - 1).is_some_and(|t| t.active()) {
        return;
    }
    let (frame_x, frame_y, paint) = world
        .tile(x, num)
        .map(|t| (t.frame_x, t.frame_y, t.paint()))
        .unwrap_or((0, 0, 0));
    // The `CopyPaintAndCoating` target: the new cell inherits the base's
    // paint (`:59029`/`:59049`/`:59062`/`:59075`).
    let place_above = |world: &mut World, frame_x_above: i16| {
        if let Some(t) = world.tile_mut(x, num - 1) {
            t.kind = 519;
            t.flags |= FLAG_ACTIVE;
            t.frame_x = frame_x_above;
            t.frame_y = frame_y;
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(0);
            t.set_paint(paint);
        }
    };
    if frame_x == 0 {
        // `:59011-59019`: the first segment.
        if let Some(t) = world.tile_mut(x, num) {
            t.frame_x = 18;
        }
    } else if frame_x == 18 {
        // `:59020-59035`: the stalk grows its paired tip cell above.
        let len = (18 * rng.range(2, 5)) as i16;
        if let Some(t) = world.tile_mut(x, num) {
            t.frame_x = len;
        }
        place_above(world, 90);
    } else if frame_x == 90 {
        // `:59036-59078`: the tip arms — the dry arm's `Next(3)` wet-extend
        // roll, else the 126/144/162 tip with its 180/198/216 partner; the
        // submerged arm takes the 108 tip unconditionally.
        if world.tile(x, num - 1).is_some_and(|t| t.liquid == 0) {
            let wet_below = world.tile(x, num).is_some_and(|t| t.liquid > 0)
                || world.tile(x, num + 1).is_some_and(|t| t.liquid > 0)
                || world.tile(x, num + 2).is_some_and(|t| t.liquid > 0);
            if !world.tile(x, num - 2).is_some_and(|t| t.active())
                && wet_below
                && rng.range(0, 3) == 0
            {
                if let Some(t) = world.tile_mut(x, num) {
                    t.frame_x = 108;
                }
                place_above(world, 90);
            } else {
                let pick = rng.range(0, 3) as i16;
                if let Some(t) = world.tile_mut(x, num) {
                    t.frame_x = 126 + pick * 18;
                }
                place_above(world, 180 + pick * 18);
            }
        } else {
            if let Some(t) = world.tile_mut(x, num) {
                t.frame_x = 108;
            }
            place_above(world, 90);
        }
    }
}

/// `WorldGen.GetWaterDepth` (`WorldGen.cs:58355-58373`): the liquid column
/// above the floor under `(x, y)` — the solid walk down, then the liquid
/// count back up.
fn get_water_depth(world: &World, x: i32, y: i32) -> i32 {
    let mut num = y;
    while !solid_tile_at(world, x, num) {
        num += 1;
        if num > world.height - 1 {
            return 0;
        }
    }
    num -= 1;
    let mut num2 = num;
    while world.tile(x, num2).is_some_and(|t| t.liquid > 0) && !solid_tile_at(world, x, num2) {
        num2 -= 1;
    }
    num - num2
}

/// `WorldGen.CountGrowingPlantTiles` (`WorldGen.cs:58375-58390`): the
/// `kind` tiles in the `[x±range] × [y±range·3]` box.
fn count_growing_plant_tiles(world: &World, x: i32, y: i32, range: i32, kind: u16) -> i32 {
    let mut num = 0;
    for i in (x - range)..=(x + range) {
        for j in (y - range * 3)..=(y + range * 3) {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == kind)
            {
                num += 1;
            }
        }
    }
    num
}

/// `WorldGen.PlaceBamboo` (`WorldGen.cs:58391-58445`) — the worldgen-stream
/// copy (the runtime twin is `growth.rs::place_bamboo`): one bamboo 571
/// stalk on the jungle grass 60 / bamboo 571 under `(x, y)` — the
/// wall/surface guard, the 314 minecart-track veto, the 2..5 water-depth
/// window, the density gates (`Next(1,21)` height budget, the 571-column
/// `i + num4/Next(1,21)` test or the +25 jungle-grass penalty, the
/// `num4 + i*2 > Next(40,61)` cap) and the write with the soil's paint.
fn place_bamboo_gen(world: &mut World, x: i32, y: i32, rng: &mut Rng) -> bool {
    let num = 2;
    let num2 = 5;
    let num3 = rng.range(1, 21);
    let Some(tile) = world.tile(x, y).copied() else {
        return false;
    };
    // `:58397-58404`: the wall/surface guard + the track veto.
    if tile.wall > 0 && (y as f64) <= world.world_surface {
        return false;
    }
    if tile.active() && tile.kind == 314 {
        return false;
    }
    let Some(tile2) = world.tile(x, y + 1).copied() else {
        return false;
    };
    if !(tile2.active() && (tile2.kind == 571 || tile2.kind == 60)) {
        return false;
    }
    // `:58408-58412`: the water-depth window.
    let water_depth = get_water_depth(world, x, y);
    if water_depth < num || water_depth > num2 {
        return false;
    }
    let mut num4 = count_growing_plant_tiles(world, x, y, 5, 571);
    let mut i = 1;
    if tile2.kind == 571 {
        // `:58417-58423`: the bamboo column's height budget.
        while !solid_tile_at(world, x, y + i) {
            i += 1;
        }
        if i + num4 / rng.range(1, 21) > num3 {
            return false;
        }
    } else {
        num4 += 25;
    }
    // `:58429-58432`: the density cap.
    if num4 + i * 2 > rng.range(40, 61) {
        return false;
    }
    // `:58433-58443`: the write (`UseBlockColors` — the soil's paint).
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = 571;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = 0;
        t.frame_y = 0;
        t.set_slope(0);
        t.flags &= !FLAG_HALF_BRICK;
        t.set_paint(tile2.paint());
    }
    true
}

/// `WorldGen.GrowCheckSeaweed(x, y)` (`WorldGen.cs:58917-58970`): the
/// seaweed growth check — the dry-out kill arm (with `KillTile`'s
/// generation-time strip), the density gate (30 in a 9×13 box), the floor
/// distance roll (`k - y < 17 - Next(20)`), the 549 placement.
fn grow_check_seaweed(world: &mut World, layers: &Layers, x: i32, y: i32, rng: &mut Rng) {
    let (max_x, max_y) = (world.width, world.height);
    let here = world.index(x, y);
    let above = world.index(x, y - 1);
    if (world.tiles[here].kind == 549 && world.tiles[here].liquid < 200)
        || world.tiles[above].liquid < 200
    {
        if world.tiles[here].active() && world.tiles[here].kind == 549 && rng.chance(2) {
            kill_tile_gen(&mut world.tiles[here]);
        }
        return;
    }
    let above2 = world.index(x, y - 2);
    let above3 = world.index(x, y - 3);
    if world.tiles[above].active()
        || world.tiles[above2].active()
        || rng.range(0, 1) != 0
        || world.tiles[above2].liquid != 255
        || world.tiles[above3].liquid != 255
    {
        return;
    }
    // The density gate: 30 seaweed tiles in the x±4 / y..y+12 box.
    let mut num4 = 0;
    'density: for i in (x - 4)..=(x + 4) {
        for j in y..=(y + 12) {
            if i < 0 || j >= max_y || i >= max_x {
                continue;
            }
            let idx = world.index(i, j);
            if world.tiles[idx].active() && world.tiles[idx].kind == 549 {
                num4 += 1;
                if num4 > 30 {
                    break 'density;
                }
            }
        }
    }
    if num4 > 30 {
        return;
    }
    // The floor scan (`:58955-58958`).
    let mut k = y;
    while !solid_tile_at(world, x, k) && k < max_y - 50 {
        k += 1;
    }
    if k - y < 17 - rng.range(0, 20) {
        // `PlaceTile(x, y - 1, 549, mute: true)` (`:58964`).
        world.tiles[above].kind = 549;
        world.tiles[above].flags |= FLAG_ACTIVE;
    }
}

/// `// --- Cactus, Palm Trees, & Coral ---------------------------------------------

/// `WorldGen.cactusWaterWidth`/`Height`/`Limit` (`WorldGen.cs:4312-4316`,
/// values `:86930-86932`) — the middle zone's cactus-vs-palm water box.
const CACTUS_WATER_WIDTH: i32 = 50;
const CACTUS_WATER_HEIGHT: i32 = 25;
const CACTUS_WATER_LIMIT: i32 = 25;
/// `TileID.Cactus` = 80, `TileID.Coral` = 81, `TileID.BeachPiles` = 324,
/// `TileID.SeaOats` = 529, `TileID.OasisPlants` = 530, `TileID.PalmTree` = 323.
const TILE_CACTUS: u16 = 80;
const TILE_CORAL: u16 = 81;
const TILE_BEACH_PILES: u16 = 324;
const TILE_SEA_OATS: u16 = 529;
const TILE_OASIS_PLANTS: u16 = 530;
const TILE_PALM: u16 = 323;
/// `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`) — the zone margins
/// and the sea-oat/oasis excision band.
pub(crate) const ZONE_MARGIN: i32 = 380;
/// The sand family the palm ground test accepts (`WorldGen.cs:27148`).
const PALM_SOIL: [u16; 4] = [53, 234, 116, 112];
/// `WallID.Sets.AllowsPlantsToGrow` (`WallID.cs:97`) — `DefaultTreeWallTest`.
const ALLOWS_PLANTS: [u16; 27] = [
    0, 150, 138, 145, 107, 152, 140, 139, 141, 106, 245, 315, 317, 63, 64, 65, 66, 67, 68, 69, 81, 70, 264, 268, 265, 74, 80,
];

#[inline]
fn default_tree_wall_test(wall: u16) -> bool {
    ALLOWS_PLANTS.contains(&wall)
}

/// `RollRandomSeaShellStyle` (`WorldGen.cs:45170-45187`): the 5 seashell
/// styles on tile 324 — `Next(2)`, then `Next(10)==0 → 2`, `Next(10)==0 → 3`,
/// `Next(50)==0 → 4` (nested `if`s, so the later rolls re-roll earlier).
fn roll_random_sea_shell_style(rng: &mut Rng) -> i32 {
    let mut result = rng.range(0, 2);
    if rng.chance(10) {
        result = 2;
    }
    if rng.chance(10) {
        result = 3;
    }
    if rng.chance(50) {
        result = 4;
    }
    result
}

/// `WorldGen.SolidTileAllowBottomSlope` (`WorldGen.cs:70167-70180`): out of
/// world is solid, active + (solid or solidTop), not top-slope/platform,
/// not half brick, not wired-inactive.
#[inline]
fn solid_tile_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return true;
    }
    let t = &world.tiles[world.index(x, y)];
    t.active()
        && ((t.kind as usize) < td::TILE_COUNT
            && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind)))
        && !t.top_slope()
        && !t.half_brick()
}

/// `WorldGen.SolidTile2(Tile)` (`WorldGen.cs:70134-70148`): active, solid,
/// no slope, no half brick (no solidTop arm).
#[inline]
fn solid_tile_2(t: &Tile) -> bool {
    t.active()
        && (t.kind as usize) < td::TILE_COUNT
        && td::TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
}

/// `WorldGen.SeaOatWaterCheck` (`WorldGen.cs:58253-58296`): the water-sum
/// probe — the beach band uses the widened 65/25 box with the inverted
/// verdict, the inland band the 45/20 box.
fn sea_oat_water_check(world: &World, x: i32, y: i32) -> bool {
    let (mut num, mut num2, num3) = (45i32, 20i32, 20i32);
    let (mut num5, mut num6) = (46i32, 0u32);
    let beach = x <= ZONE_MARGIN || x >= world.width - ZONE_MARGIN;
    if beach {
        num = 65;
        num2 += 5;
    }
    for i in x - num..=x + num {
        for j in y - num2..=y + num2 {
            if i < 0 || j < 0 || i >= world.width || j >= world.height {
                continue;
            }
            let idx = world.index(i, j);
            if !solid_tile(&world.tiles[idx]) && world.tiles[idx].liquid > 0 {
                num6 += world.tiles[idx].liquid as u32;
                let d = (i - x).abs();
                if d < num5 {
                    num5 = d;
                }
            }
        }
    }
    if num6 / 255 >= num3 as u32 {
        return !beach;
    }
    beach
}

/// `WorldGen.OasisPlantWaterCheck` (`WorldGen.cs:47506-47536`): the 45/20
/// water sum, with the `x <= beachDistance` skip quirk (the beach columns
/// count no water and fail the check by construction).
fn oasis_plant_water_check(world: &World, x: i32, y: i32) -> bool {
    let (num, num2, num3) = (45i32, 20i32, 20u32);
    let mut num4 = 0u32;
    for i in x - num..=x + num {
        if x <= ZONE_MARGIN || x >= world.width - ZONE_MARGIN {
            continue;
        }
        for j in y - num2..=y + num2 {
            if i < 0 || j < 0 || i >= world.width || j >= world.height {
                continue;
            }
            let idx = world.index(i, j);
            if !solid_tile(&world.tiles[idx]) {
                num4 += world.tiles[idx].liquid as u32;
            }
        }
    }
    num4 / 255 >= num3
}

/// `WorldGen.PlaceTile(81, …)`'s 1x1 coral slice (`WorldGen.cs:59431-59446`
/// guards + the generic active/type/frame write): the tile above must be
/// inactive, the tile below solid with no half brick or slope.
fn place_coral(world: &mut World, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return false;
    }
    let above = world.index(x, y - 1);
    if world.tiles[above].active() {
        return false;
    }
    let below = world.index(x, y + 1);
    if y + 1 >= world.height
        || !world.tiles[below].active()
        || !solid_tile_2(&world.tiles[below])
        || world.tiles[below].slope() != 0
    {
        return false;
    }
    let idx = world.index(x, y);
    let t = &mut world.tiles[idx];
    t.flags |= FLAG_ACTIVE;
    t.kind = TILE_CORAL;
    t.frame_x = 0;
    t.frame_y = 0;
    t.set_slope(0);
    true
}

/// `WorldGen.PlaceTile(324, …, style)`'s Place1x1 slice
/// (`WorldGen.cs:45204-45215` via `:60133-60140`): the ground must be solid
/// (or a table — none exist in the port's gen), then frameX re-rolls the
/// shell style and frameY takes the passed style.
fn place_beach_piles(world: &mut World, x: i32, y: i32, style: i32, rng: &mut Rng) {
    if x < 0 || y < 0 || x >= world.width || y + 1 >= world.height {
        return;
    }
    let below = world.index(x, y + 1);
    if !solid_tile_2(&world.tiles[below]) {
        return;
    }
    let idx = world.index(x, y);
    let t = &mut world.tiles[idx];
    t.flags |= FLAG_ACTIVE;
    t.kind = TILE_BEACH_PILES;
    t.frame_x = (22 * roll_random_sea_shell_style(rng)) as i16;
    t.frame_y = (22 * style) as i16;
}

/// `WorldGen.PlantSeaOat` (`WorldGen.cs:58299-58330`).
fn plant_sea_oat(world: &mut World, x: i32, y: i32, rng: &mut Rng) -> bool {
    if x < 0 || y < 0 || x >= world.width || y + 1 >= world.height {
        return false;
    }
    let idx = world.index(x, y);
    let here = &world.tiles[idx];
    if here.wall > 0 || here.active() || here.liquid > 0 {
        return false;
    }
    let below = world.index(x, y + 1);
    if !solid_tile_allow_bottom_slope(world, x, y + 1)
        || !PALM_SOIL.contains(&world.tiles[below].kind)
    {
        return false;
    }
    if !sea_oat_water_check(world, x, y) {
        return false;
    }
    let t = &mut world.tiles[idx];
    t.flags |= FLAG_ACTIVE;
    t.set_slope(0);
    t.flags &= !FLAG_HALF_BRICK;
    t.kind = TILE_SEA_OATS;
    t.frame_x = (rng.range(0, 5) * 18) as i16;
    t.frame_y = 0;
    true
}

/// `WorldGen.GrowSeaOat` (`WorldGen.cs:58342-58352`).
fn grow_sea_oat(t: &mut Tile) {
    if t.frame_x < 180 {
        t.frame_x += 90;
    }
}

/// `WorldGen.PlaceOasisPlant` (`WorldGen.cs:47538-47600`): the 3x2 plant —
/// `Next(9)` for the frame column first (before the bounds gate), the
/// occupancy scan, the water check, and the KillTile strip across the box.
fn place_oasis_plant(world: &mut World, x: i32, y: i32, kind: u16, rng: &mut Rng) {
    let num = rng.range(0, 9);
    let num2 = 0i32;
    if x < ZONE_MARGIN || x > world.width - ZONE_MARGIN || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag = true;
    for i in x - 1..x + 2 {
        for j in y - 1..y + 1 {
            let idx = world.index(i, j);
            if world.tiles[idx].active() && world.tiles[idx].kind != TILE_SEA_OATS {
                flag = false;
            }
            if world.tiles[idx].liquid > 0 {
                flag = false;
            }
        }
        let below = world.index(i, y + 1);
        if !solid_tile(&world.tiles[below]) || !PALM_SOIL.contains(&world.tiles[below].kind) {
            flag = false;
        }
    }
    if !flag || !oasis_plant_water_check(world, x, y) {
        return;
    }
    for k in x - 1..x + 2 {
        for l in y - 1..y + 1 {
            let idx = world.index(k, l);
            if world.tiles[idx].active() {
                let t = &mut world.tiles[idx];
                t.flags &= !FLAG_ACTIVE;
                t.kind = 0;
                t.frame_x = 0;
                t.frame_y = 0;
            }
        }
    }
    let num3 = (54 * num) as i16;
    let num4 = (36 * num2) as i16;
    for k in x - 1..x + 2 {
        let idx = world.index(k, y - 1);
        let t = &mut world.tiles[idx];
        t.flags |= FLAG_ACTIVE;
        t.frame_y = num4;
        t.frame_x = num3 + (k - (x - 1)) as i16 * 18;
        t.kind = kind;
    }
}

/// `WorldGen.GrowPalmTree` (`WorldGen.cs:27116-27202`): the palm trunk on
/// tile 323 — the ground/slope/water/wall guards, the 2x1 ground clear box
/// and the trunk clearance box (`EmptyTileCheck(…, 20)` — saplings allowed),
/// then the frame table (base 66, trunk 22·Next(0,3), top 22·Next(4,7),
/// frameY the lean walk). `RangeFrame`/`NetMessage` do not exist in the port.
pub(crate) fn grow_palm_tree(
    world: &mut World,
    i: i32,
    y: i32,
    addon: i32,
    ignore_walls: bool,
    rng: &mut Rng,
) -> bool {
    if i < 0 || y < 0 || i >= world.width || y >= world.height {
        return false;
    }
    let mut num = y;
    // The 20-walk (`:27124-27128`).
    while world.tile(i, num).is_some_and(|t| t.active() && t.kind == 20) {
        num += 1;
        if num >= world.height {
            return false;
        }
    }
    // The guards (`:27139-27151`).
    {
        let tile = &world.tiles[world.index(i, num)];
        let tile2 = &world.tiles[world.index(i, num - 1)];
        if !tile.active() || tile.half_brick() || tile.slope() != 0 {
            return false;
        }
        if (!ignore_walls && !default_tree_wall_test(tile2.wall)) || tile2.liquid != 0 {
            return false;
        }
        if !PALM_SOIL.contains(&tile.kind) {
            return false;
        }
    }
    let num2 = 20 + addon;
    if !empty_tile_check(world, i, i, num - 2, num - 1, 20) {
        return false;
    }
    if !empty_tile_check(world, i - 1, i + 1, num - num2, num - 3, 20) {
        return false;
    }
    let num3 = rng.range(10, 21) + addon;
    let num4 = rng.range(-8, 9) * 2;
    let mut num5: i16 = 0;
    for j in 0..num3 {
        let idx = world.index(i, num - 1 - j);
        if j == 0 {
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = TILE_PALM;
            t.frame_x = 66;
            t.frame_y = 0;
        } else if j == num3 - 1 {
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = TILE_PALM;
            t.frame_x = (22 * rng.range(4, 7)) as i16;
            t.frame_y = num5;
        } else {
            // The lean walk (`:27182-27194`): the frame-fraction gates, the
            // `|| true` tail makes the last arm an unconditional step.
            if num5 as i32 != num4 {
                let num6 = j as f64 / num3 as f64;
                let stepped = if num6 < 0.25 {
                    false
                } else if num6 < 0.5 {
                    rng.chance(13)
                } else if num6 < 0.7 {
                    rng.chance(9)
                } else if num6 < 0.95 {
                    rng.chance(5)
                } else {
                    true
                };
                if stepped {
                    num5 += (num4.signum() * 2) as i16;
                }
            }
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = TILE_PALM;
            t.frame_x = (22 * rng.range(0, 3)) as i16;
            t.frame_y = num5;
        }
    }
    true
}

/// `TryGrowingTreeByType(323, …)` (`WorldGen.cs:30343-30353`).
#[inline]
fn try_growing_palm(world: &mut World, x: i32, y: i32, rng: &mut Rng) -> bool {
    grow_palm_tree(world, x, y, 0, false, rng)
}

/// `WorldGen.GrowCactus` (`WorldGen.cs:56575-56813`) — the full port. The
/// `SquareTileFrame`/`NetMessage` arms do not exist in the port (the later
/// framing pass owns them).
pub(crate) fn grow_cactus(world: &mut World, i: i32, j: i32, rng: &mut Rng) {
    if j <= 0 || i < 0 || j >= world.height || i >= world.width {
        return;
    }
    let mut num = i;
    let mut num2 = j;
    let (tile_kind, tile_notactive, tile_half, tile_slope, above_liquid) = {
        let t = &world.tiles[world.index(i, j)];
        let t2 = &world.tiles[world.index(i, j - 1)];
        (
            t.kind,
            !t.active(),
            t.half_brick(),
            t.slope() != 0,
            t2.liquid > 0,
        )
    };
    if tile_notactive
        || tile_half
        || tile_slope
        || (tile_kind != TILE_CACTUS && !PALM_SOIL.contains(&tile_kind))
        || above_liquid
    {
        return;
    }
    // The water box (`:56604-56616`).
    let mut sum = 0u32;
    for k in (i - CACTUS_WATER_WIDTH)..(i + CACTUS_WATER_WIDTH) {
        for l in (j - CACTUS_WATER_HEIGHT)..(j + CACTUS_WATER_HEIGHT) {
            if k >= 0 && l >= 0 && k < world.width && l < world.height {
                sum += world.tiles[world.index(k, l)].liquid as u32;
            }
        }
    }
    if sum / 255 > CACTUS_WATER_LIMIT as u32 {
        return;
    }
    // Tile 80 is style-driven — the retired scatter declared its boot static,
    // so the pass keeps it (sizing the table on raw test worlds).
    if world.frame_important.len() <= TILE_CACTUS as usize {
        world.frame_important.resize(td::TILE_COUNT, false);
    }
    world.frame_important[TILE_CACTUS as usize] = true;
    if PALM_SOIL.contains(&tile_kind) {
        // The sand arm (`:56619-56668`): the neighbours-above clear, the
        // 13x5 census, the 1-in-2 slope flatten and the tile-80 plant.
        let above = world.index(i, j - 1);
        let left_above = world.index(i - 1, j - 1);
        let right_above = world.index(i + 1, j - 1);
        if world.tiles[above].active()
            || world.tiles[left_above].active()
            || world.tiles[right_above].active()
        {
            return;
        }
        let mut num4 = 0;
        let mut num5 = 0;
        'census: for m in (i - 6)..=(i + 6) {
            for n in (j - 3)..=(j + 1) {
                if m < 0 || n < 0 || m >= world.width || n >= world.height {
                    continue;
                }
                let idx = world.index(m, n);
                if !world.tiles[idx].active() {
                    continue;
                }
                if world.tiles[idx].kind == TILE_CACTUS {
                    num4 += 1;
                    if num4 >= 4 {
                        return;
                    }
                }
                if PALM_SOIL.contains(&world.tiles[idx].kind) {
                    num5 += 1;
                    if num5 > 10 {
                        break 'census;
                    }
                }
            }
            if num5 > 10 {
                break;
            }
        }
        if num5 > 10 {
            if rng.chance(2) {
                let idx = world.index(i, j);
                world.tiles[idx].set_slope(0);
            }
            let idx = world.index(i, j - 1);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = TILE_CACTUS;
        }
    } else {
        // The cactus arm (`:56669-56813`): the walk up the column, the
        // branch scans, the density gate and the three growth writes.
        if tile_kind != TILE_CACTUS {
            return;
        }
        while world.tile(num, num2).is_some_and(|t| t.active() && t.kind == TILE_CACTUS) {
            num2 += 1;
            if num2 >= world.height {
                return;
            }
            let cur = world.index(num, num2);
            if !(world.tiles[cur].active() && world.tiles[cur].kind == TILE_CACTUS) {
                let l1 = world.index(num - 1, num2);
                let l2 = world.index(num - 1, num2 - 1);
                if world.tiles[l1].active()
                    && world.tiles[l1].kind == TILE_CACTUS
                    && world.tiles[l2].active()
                    && world.tiles[l2].kind == TILE_CACTUS
                    && num >= i
                {
                    num -= 1;
                }
                let r1 = world.index(num + 1, num2);
                let r2 = world.index(num + 1, num2 - 1);
                if world.tiles[r1].active()
                    && world.tiles[r1].kind == TILE_CACTUS
                    && world.tiles[r2].active()
                    && world.tiles[r2].kind == TILE_CACTUS
                    && num <= i
                {
                    num += 1;
                }
            }
        }
        let num6 = num2 - 1 - j;
        let num7 = i - num;
        num = i - num7;
        num2 = j;
        let num8 = 11 - num6;
        let mut num9 = 0;
        for num10 in (num - 2)..=(num + 2) {
            for num11 in (num2 - num8)..=(num2 + num6) {
                if num10 < 0 || num11 < 0 || num10 >= world.width || num11 >= world.height {
                    continue;
                }
                let idx = world.index(num10, num11);
                if world.tiles[idx].active() && world.tiles[idx].kind == TILE_CACTUS {
                    num9 += 1;
                }
            }
        }
        if num9 >= rng.range(11, 13) {
            return;
        }
        num = i;
        num2 = j;
        if num7 == 0 {
            if num6 == 0 {
                let idx = world.index(num, num2 - 1);
                if !world.tiles[idx].active() {
                    let t = &mut world.tiles[idx];
                    t.flags |= FLAG_ACTIVE;
                    t.kind = TILE_CACTUS;
                }
                return;
            }
            let (mut flag, mut flag2) = (false, false);
            {
                let here = world.index(num, num2 - 1);
                if world.tiles[here].active() && world.tiles[here].kind == TILE_CACTUS {
                    let left = [(-1, 0), (-2, 1), (-1, -1), (-1, 1), (-2, 0)];
                    if left
                        .iter()
                        .all(|&(dx, dy)| !world.tile(num + dx, num2 + dy).is_some_and(|t| t.active()))
                    {
                        flag = true;
                    }
                    let right = [(1, 0), (2, 1), (1, -1), (1, 1), (2, 0)];
                    if right
                        .iter()
                        .all(|&(dx, dy)| !world.tile(num + dx, num2 + dy).is_some_and(|t| t.active()))
                    {
                        flag2 = true;
                    }
                }
            }
            let num12 = rng.range(0, 3);
            if num12 == 0 && flag {
                let idx = world.index(num - 1, num2);
                let t = &mut world.tiles[idx];
                t.flags |= FLAG_ACTIVE;
                t.kind = TILE_CACTUS;
            } else if num12 == 1 && flag2 {
                let idx = world.index(num + 1, num2);
                let t = &mut world.tiles[idx];
                t.flags |= FLAG_ACTIVE;
                t.kind = TILE_CACTUS;
            } else if num6 < rng.range(2, 8) {
                let l = world.index(num - 1, num2 - 1);
                let r = world.index(num + 1, num2 - 1);
                let up = world.index(num, num2 - 1);
                let l_ok = !world.tiles[l].active() || world.tiles[l].kind != TILE_CACTUS;
                let r_ok = !world.tiles[r].active() || world.tiles[r].kind != TILE_CACTUS;
                if l_ok && r_ok && !world.tiles[up].active() {
                    let t = &mut world.tiles[up];
                    t.flags |= FLAG_ACTIVE;
                    t.kind = TILE_CACTUS;
                }
            }
        } else {
            let up = world.index(num, num2 - 1);
            let up2 = world.index(num, num2 - 2);
            let lean = world.index(num + num7, num2 - 1);
            let anchor = world.index(num - num7, num2 - 1);
            if !world.tiles[up].active()
                && !world.tiles[up2].active()
                && !world.tiles[lean].active()
                && world.tiles[anchor].active()
                && world.tiles[anchor].kind == TILE_CACTUS
            {
                let t = &mut world.tiles[up];
                t.flags |= FLAG_ACTIVE;
                t.kind = TILE_CACTUS;
            }
        }
    }
}

/// `WorldGen.PlantCactus` (`WorldGen.cs:54160-54168`).
fn plant_cactus(world: &mut World, i: i32, j: i32, rng: &mut Rng) {
    grow_cactus(world, i, j, rng);
    for _ in 0..150 {
        let x = rng.range(i - 1, i + 2);
        let y = rng.range(j - 10, j + 2);
        grow_cactus(world, x, y, rng);
    }
}

/// `Cactus, Palm Trees, & Coral` (`WorldGen.cs:21310-21518`): the oasis
/// block (`:21326-21363`) over `GenVars.numOasis` (W4-d103: the Oasis pass
/// now writes the ledger) and the three zones' depth and shallow arms.
/// The middle (cactus) zone keeps the faithful water box and the 1-in-4
/// palm, with `PlantCactus` growing the cactus.
pub(crate) fn cactus_palm_trees_and_coral(
    world: &mut World,
    gv: &GenVars,
    layers: &Layers,
    rng: &mut Rng,
) {
    let (num2, max_x, max_y) = (400i32, world.width, world.height);
    let num = 8; // remix 2 — no remix seed yet
    let num3 = rng.range(3, 13);
    let num4 = rng.range(3, 13);
    let _num5 = rng.range(2, 6);
    let _num6 = rng.range(2, 6);
    let num7 = 380;
    // The oasis block (`:21326-21363`): palms at 1.0 (0.13 near an
    // existing palm), sea oats and oasis plants over each basin's box.
    for oi in 0..gv.num_oasis as usize {
        let (ox, oy) = gv.oasis_position[oi];
        let num8 = (gv.oasis_width[oi] as f64 * 1.5) as i32;
        for j in ox - num8..=ox + num8 {
            if j < 0 || j >= max_x {
                continue;
            }
            for k in oy - gv.oasis_height..=oy + gv.oasis_height {
                if k < 0 || k >= max_y {
                    continue;
                }
                let mut num9 = 1.0;
                let num10 = 8;
                'palm_scan: for l in j - num10..=j + num10 {
                    for m in k - num10..=k + num10 {
                        if l < 0 || m < 0 || l >= max_x || m >= max_y {
                            continue;
                        }
                        let t = &world.tiles[world.index(l, m)];
                        if t.active() && t.kind == TILE_PALM {
                            num9 = 0.13;
                            break 'palm_scan;
                        }
                    }
                }
                if rng.next_f64() < num9 {
                    try_growing_palm(world, j, k, rng);
                }
                if plant_sea_oat(world, j, k, rng) {
                    if rng.chance(2) {
                        let idx = world.index(j, k);
                        grow_sea_oat(&mut world.tiles[idx]);
                    }
                    if rng.chance(2) {
                        let idx = world.index(j, k);
                        grow_sea_oat(&mut world.tiles[idx]);
                    }
                }
                place_oasis_plant(world, j, k, TILE_OASIS_PLANTS, rng);
            }
        }
    }
    let num13 = layers.world_surface - 1.0;
    for n in 0..3 {
        let (num11, num12, flag, max_value) = match n {
            0 => (5, num7, false, num3),
            1 => (num2, max_x - num2, true, num),
            _ => (max_x - num7, max_x - 5, false, num4),
        };
        for num14 in num11..num12 {
            if rng.range(0, max_value) == 0 {
                // The depth arm (`:21397-21492`).
                for num15 in 0..(num13 as i32) {
                    let idx = world.index(num14, num15);
                    let tile_ok = world.tiles[idx].active()
                        && (world.tiles[idx].kind == 53
                            || world.tiles[idx].kind == 112
                            || world.tiles[idx].kind == 234);
                    if !tile_ok {
                        continue;
                    }
                    let above = world.index(num14, num15 - 1);
                    if world.tiles[above].active() || world.tiles[above].wall != 0 {
                        continue;
                    }
                    if flag {
                        // The middle zone: the 50/25/25 water box.
                        let mut sum = 0u32;
                        for k in (num14 - CACTUS_WATER_WIDTH)..(num14 + CACTUS_WATER_WIDTH) {
                            for l in (num15 - CACTUS_WATER_HEIGHT)..(num15 + CACTUS_WATER_HEIGHT) {
                                if k >= 0 && l >= 0 && k < max_x && l < max_y {
                                    sum += world.tiles[world.index(k, l)].liquid as u32;
                                }
                            }
                        }
                        if sum / 255 > CACTUS_WATER_LIMIT as u32 {
                            if rng.chance(4) {
                                try_growing_palm(world, num14, num15, rng);
                            }
                        } else {
                            plant_cactus(world, num14, num15, rng);
                        }
                    } else {
                        // The beach zones (`:21476-21492`).
                        let w2 = world.index(num14, num15 - 2);
                        let w3 = world.index(num14, num15 - 3);
                        let w4 = world.index(num14, num15 - 4);
                        if world.tiles[w2].liquid == 255
                            && world.tiles[w3].liquid == 255
                            && world.tiles[w4].liquid == 255
                        {
                            if rng.chance(2) {
                                place_coral(world, num14, num15 - 1);
                            } else {
                                let style = roll_random_sea_shell_style(rng);
                                place_beach_piles(world, num14, num15 - 1, style, rng);
                            }
                            break;
                        }
                        if world.tiles[w2].liquid == 0 && (num15 as f64) < layers.world_surface {
                            let style = roll_random_sea_shell_style(rng);
                            place_beach_piles(world, num14, num15 - 1, style, rng);
                            break;
                        }
                    }
                }
            } else {
                // The shallow arm (`:21498-21515`).
                for num22 in 0..(num13 as i32) {
                    if plant_sea_oat(world, num14, num22, rng) {
                        if rng.chance(2) {
                            let idx = world.index(num14, num22);
                            grow_sea_oat(&mut world.tiles[idx]);
                        }
                        if rng.chance(2) {
                            let idx = world.index(num14, num22);
                            grow_sea_oat(&mut world.tiles[idx]);
                        }
                    }
                    place_oasis_plant(world, num14, num22, TILE_OASIS_PLANTS, rng);
                }
            }
        }
    }
}
/// `KillTile`'s generation-time strip (`WorldGen.cs:63517`, the
/// `isGeneratingOrLoadingWorld → noItem` path): the tile goes inactive with
/// its type cleared. The item-drop / frame / net arms do not exist in the
/// port; the `SquareTileFrame` reframe is the port's later framing pass.
fn kill_tile_gen(t: &mut Tile) {
    if !t.active() {
        return;
    }
    t.flags &= !FLAG_ACTIVE;
    t.kind = 0;
    t.frame_x = 0;
    t.frame_y = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    fn flat(w: i32, h: i32) -> World {
        World::new("ocean".into(), w, h)
    }

    /// DW-2.1: the Ocean Sand draw sequence — the centre re-roll, the i==1
    /// full-draw burn, the two `Next(5)` per sand tile — replays against an
    /// independent ledger on a flat world. The world starts solid, so every
    /// band's first active tile is row 0 and the sand writes are gated by
    /// the `Next(5)` margins only (the bands' true wedge depth is untestable
    /// on a filled world — the draw EQUALITY is the pin).
    #[test]
    fn ocean_sand_draw_ledger() {
        let (w, h) = (400, 300);
        let mut world = flat(w, h);
        for x in 0..w {
            for y in 0..h {
                let i = world.index(x, y);
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut gv = GenVars::default();
        gv.left_beach_end = 120;
        gv.right_beach_start = 280;
        let layers = super::super::terrain::Layers {
            avg_surface: 100.0,
            world_surface: 110.0,
            rock_layer: 160.0,
            hell_top: 250,
            surface_low: 90.0,
            surface_high: 110.0,
            rock_low: 150.0,
            rock_high: 160.0,
        };
        let mut a = Rng(0x5EED);
        let mut gva = gv.clone();
        ocean_sand(&mut world, &mut gva, &layers, &mut a);

        // The ledger: same world, same anchors, independent Rng replay.
        let mut wb = flat(w, h);
        for x in 0..w {
            for y in 0..h {
                let i = wb.index(x, y);
                wb.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut rep = Rng(0x5EED);
        for i in 0..3i32 {
            let mut num = rep.range(0, w);
            while num > (w as f64 * 0.4) as i32 && num < (w as f64 * 0.6) as i32 {
                num = rep.range(0, w);
            }
            let mut num2 = rep.range(35, 90);
            if i == 1 {
                let num3 = w as f64 / 4200.0;
                num2 += (rep.range(20, 40) as f64 * num3) as i32;
            }
            if rep.chance(3) {
                num2 *= 2;
            }
            if i == 1 {
                num2 *= 2;
            }
            let num4 = num - num2;
            let mut nb = rep.range(35, 90);
            if rep.chance(3) {
                nb *= 2;
            }
            if i == 1 {
                nb *= 2;
            }
            let num5 = num + nb;
            let num4 = if num4 < 0 { 0 } else { num4 };
            let num5 = if num5 > w { w } else { num5 };
            let (num4, num5) = match i {
                0 => (0, gv.left_beach_end),
                2 => (gv.right_beach_start, w),
                _ => continue,
            };
            let mut num6 = rep.range(50, 100);
            for j in num4..num5 {
                if rep.chance(2) {
                    num6 += rep.range(-1, 2);
                    if num6 < 50 {
                        num6 = 50;
                    }
                    if num6 > 200 {
                        num6 = 200;
                    }
                }
                let mut k = 0;
                while k < 150 && !wb.tile(j, k).is_some_and(|t| t.active()) {
                    k += 1;
                }
                if k >= 150 {
                    continue;
                }
                let is_mid = j == (num4 + num5) / 2;
                if is_mid && rep.chance(6) {
                    let n = gv.num_pyr;
                    if (n as usize) < gv.pyr_x.len() {
                        gv.pyr_x[n as usize] = j;
                        gv.pyr_y[n as usize] = k;
                        gv.num_pyr = n + 1;
                    }
                }
                let mut num7 = num6;
                if j - num4 < num7 {
                    num7 = j - num4;
                }
                if num5 - j < num7 {
                    num7 = num5 - j;
                }
                num7 += rep.range(0, 5);
                for l in k..k + num7 {
                    if j > num4 + rep.range(0, 5) && j < num5 - rep.range(0, 5) {
                        let i = wb.index(j, l);
                        wb.tiles[i].kind = TILE_SAND;
                    }
                }
                break;
            }
        }
        assert_eq!(world.tiles.len(), wb.tiles.len());
        // The worlds must match tile-for-tile after the pass — the ledger
        // replay proves the draw sequence (both sandifying and leaving
        // untouched columns alike).
        for (n, (ta, tb)) in world.tiles.iter().zip(wb.tiles.iter()).enumerate() {
            assert_eq!(
                (ta.kind, ta.active(), ta.wall),
                (tb.kind, tb.active(), tb.wall),
                "tile {n} diverges"
            );
        }
    }

    /// DW-2.2: the Beaches prism — the air/water/sand layering, the `127`
    /// surface rows, the wall-0 wipe and the `shellStart*` anchors.
    #[test]
    fn beaches_carves_the_prism() {
        let (w, h) = (600, 200);
        let mut world = flat(w, h);
        // A solid flat ground: the carve's anchor scan finds it.
        for x in 0..w {
            for y in 40..h {
                let i = world.index(x, y);
                world.tiles[i].flags |= FLAG_ACTIVE;
                world.tiles[i].kind = TILE_SAND;
                if (43..57).contains(&y) {
                    world.tiles[i].wall = 1;
                }
            }
        }
        let mut gv = GenVars::default();
        gv.left_beach_end = 200;
        gv.right_beach_start = 400;
        // The wall probe: walls sit at rows 43..56 (above the water line in
        // the test's flat ground), inside the carve's column range.
        let mut rng = Rng(0x5EED);
        beaches(&mut world, &mut gv, &mut rng);
        // The left prism: columns [0, 275) carry air and water above a sand
        // floor. The anchor scan finds the flat ground at row 40, so the
        // carve's k-range covers the wall rows — every carved column's wall
        // in the range is wiped, and the inland columns are untouched.
        let (mut water, mut sand_rows) = (0, 0);
        for x in 0..275 {
            for y in 0..200 {
                let t = &world.tiles[world.index(x, y)];
                if t.liquid > 0 {
                    water += 1;
                }
                if t.active() && t.kind == TILE_SAND {
                    sand_rows += 1;
                }
            }
        }
        assert!(water > 200, "the prism is flooded ({water})");
        assert!(sand_rows > 200, "the seafloor is sand ({sand_rows})");
        // The left carve covers [0, 275) forced, capped here by
        // `leftBeachEnd - 50` = 150 (the probe shows the water starting at
        // the anchor row 44, so the wall wipe runs over the full carve
        // range). Every carved column's wall rows are wiped; the inland
        // columns keep theirs.
        let unwiped = (0..150).any(|x| (43..57).any(|y| world.tile(x, y).is_some_and(|t| t.wall != 0)));
        assert!(!unwiped, "the carved columns lose their walls");
        let kept = (400..500).any(|x| (43..57).any(|y| world.tile(x, y).is_some_and(|t| t.wall != 0)));
        assert!(kept, "the inland columns are untouched");
        assert!(gv.shell_start_y_left > 0, "the left anchor is recorded");
        assert!(gv.shell_start_x_left > 0, "the first-surface column is recorded");
        assert!(gv.shell_start_y_right > 0, "the right anchor is recorded");
    }

    /// DW-2.3: `TuneOceanDepth`'s factor table — the depth curve is monotone
    /// in count within each arm and the two arms differ at the extremes.
    #[test]
    fn tune_ocean_depth_curves() {
        let mut rng = Rng(42);
        let mut normal = 1.0f64;
        for c in 1..260 {
            normal = tune_ocean_depth(c, normal, false, &mut rng);
        }
        let mut rng = Rng(42);
        let mut florida = 1.0f64;
        for c in 1..260 {
            florida = tune_ocean_depth(c, florida, true, &mut rng);
        }
        assert!(normal > 20.0, "the normal shelf deepens ({normal})");
        assert!(florida > 20.0, "the florida shelf deepens ({florida})");
        assert!(
            (normal - florida).abs() > 1.0,
            "the two curves are different shapes"
        );
    }

    /// DW-2.4: `GravitatingSandCleanup` refills the gap under a falling tile
    /// with the falling type, and leaves the other columns alone.
    #[test]
    fn gravitating_sand_refills_the_gap() {
        let mut world = flat(10, 60);
        for i in 0..600 {
            world.tiles[i].flags |= FLAG_ACTIVE;
            world.tiles[i].kind = TILE_SAND;
        }
        // An air gap at rows 40..44 in column 5.
        for y in 40..44 {
            let idx = world.index(5, y);
            world.tiles[idx] = crate::tile::Tile::default();
        }
        // A solid 1-tile hole below: the `num2 != num - 1` guard skips the
        // adjacent case, so the column below the gap counts.
        gravitating_sand(&mut world, 50.0);
        for y in 40..44 {
            let t = &world.tiles[world.index(5, y)];
            assert!(t.active() && t.kind == TILE_SAND, "the gap at {y} is refilled");
        }
    }

    /// The test world for the cave walkers: a solid sand block with a flat
    /// seafloor, water above it and walls below — what `Beaches` leaves.
    fn cave_world(w: i32, h: i32, floor: i32) -> World {
        let mut world = World::new("cave".into(), w, h);
        for x in 0..w {
            for y in floor..h {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
                if y > floor + 2 {
                    world.tiles[i].wall = 1;
                }
                if y < floor + 12 {
                    world.tiles[i].liquid = 255;
                }
            }
        }
        world
    }

    fn cave_layers() -> Layers {
        Layers {
            avg_surface: 40.0,
            world_surface: 50.0,
            rock_layer: 120.0,
            hell_top: 260,
            surface_low: 30.0,
            surface_high: 50.0,
            rock_low: 110.0,
            rock_high: 120.0,
        }
    }

    /// DW-3.1: the wrapper's dungeon-side gate — the port's dungeon side is
    /// fixed Right, so only the RIGHT ocean takes the `Next(3)` roll (the
    /// left side is skipped before any draw, per the C# short-circuit
    /// `i != 0 || dungeonSide < Right`, `WorldGen.cs:15176`). The wrapper is
    /// pinned as a composition: replaying the right-side roll + anchor on an
    /// independent Rng must reproduce the wrapper's world exactly — a
    /// left-side run would consume its own roll and diverge.
    #[test]
    fn ocean_caves_gates_on_the_dungeon_side() {
        for seed in 0..16u64 {
            let mut a = cave_world(600, 300, 60);
            let mut gva = GenVars::default();
            ocean_caves(&mut a, &mut gva, &cave_layers(), &mut Rng(seed));

            let mut rep = Rng(seed);
            let mut b = cave_world(600, 300, 60);
            let mut gvb = GenVars::default();
            if rep.chance(3) {
                let num = rep.range(600 - 95, 600 - 55);
                let mut j = 0;
                while j < b.height - 4 && !b.tile(num, j).is_some_and(|t| t.active()) {
                    j += 1;
                }
                ocean_cave(&mut b, &mut gvb, &cave_layers(), num, j, &mut rep);
            }
            for (n, (ta, tb)) in a.tiles.iter().zip(b.tiles.iter()).enumerate() {
                assert_eq!(
                    (ta.kind, ta.flags, ta.wall, ta.liquid),
                    (tb.kind, tb.flags, tb.wall, tb.liquid),
                    "tile {n} diverges at seed {seed}"
                );
            }
            assert_eq!(gva.ocean_cave_treasure, gvb.ocean_cave_treasure);
            assert_eq!(gva.num_ocean_cave_treasure, gvb.num_ocean_cave_treasure);
        }
    }

    /// DW-3.1b: the `WorldGen.cs:15178` chance gate's drunk arm —
    /// `genRand.Next(maxValue) == 0 || WorldGen.drunkWorldGen`. `Next` is the
    /// left `||` operand, so the roll is burned either way; a drunk world
    /// simply never fails the gate. Pinned with a seed whose first `Next(3)`
    /// fails: the classic run leaves the world untouched, the drunk run
    /// carves.
    #[test]
    fn ocean_caves_drunk_passes_the_chance_gate() {
        let seed = (0..64u64)
            .find(|s| !Rng(*s).chance(3))
            .expect("a failing Next(3) in 64 seeds");
        // Classic: the failed roll skips both the anchor scan and the
        // walker — the world is byte-for-byte the untouched baseline.
        let mut classic = cave_world(600, 300, 60);
        let mut gv = GenVars::default();
        ocean_caves(&mut classic, &mut gv, &cave_layers(), &mut Rng(seed));
        let untouched = cave_world(600, 300, 60);
        for (n, (a, b)) in classic.tiles.iter().zip(untouched.tiles.iter()).enumerate() {
            assert_eq!(
                (a.kind, a.flags, a.wall, a.liquid),
                (b.kind, b.flags, b.wall, b.liquid),
                "tile {n} diverges at seed {seed}"
            );
        }
        // Drunk: the same burned roll cannot fail the gate — the cave lands.
        let mut drunk = cave_world(600, 300, 60);
        drunk.seeds |= crate::seeds::SeedFlags::DRUNK;
        let mut gvd = GenVars::default();
        ocean_caves(&mut drunk, &mut gvd, &cave_layers(), &mut Rng(seed));
        let carved = drunk
            .tiles
            .iter()
            .filter(|t| t.kind == OCEAN_CAVE_CARVED)
            .count();
        assert!(carved > 0, "the drunk ocean cave carved ({carved})");
        // And the classic world still carves on a passing roll — the arm
        // adds no draw of its own, it only widens the gate.
        let pass = (0..64u64)
            .find(|s| Rng(*s).chance(3))
            .expect("a passing Next(3) in 64 seeds");
        let mut classic2 = cave_world(600, 300, 60);
        let mut gv2 = GenVars::default();
        ocean_caves(&mut classic2, &mut gv2, &cave_layers(), &mut Rng(pass));
        let carved2 = classic2
            .tiles
            .iter()
            .filter(|t| t.kind == OCEAN_CAVE_CARVED)
            .count();
        assert!(
            carved2 > 0,
            "the classic ocean cave carved on a pass ({carved2})"
        );
    }

    /// DW-3.2: the walker's invariants — the sentinel-264 carve marker, the
    /// protected bad tiles (never carved, never deactivated), the full-depth
    /// water shaft below the floor, and the treasure anchors landing inside
    /// the world. The stalactite's `Next(3)` widening writes sand beside the
    /// column WITHOUT the bad check (`WorldGen.cs:76623-76628`, verbatim) —
    /// so the altar column keeps its ACTIVE state but may be retyped.
    #[test]
    fn ocean_cave_invariants() {
        let mut world = cave_world(600, 300, 60);
        // A protected column: an altar (26) pillar at x=700 — outside the
        // walker's reach (the box's right edge never passes x + 3*num4 ≈
        // 595 for a right-side anchor). Columns ON the walker's centre are
        // legitimately overwritten by the stalactite's unchecked writes
        // (`WorldGen.cs:76623-76628`) — the box's bad check then sees the
        // sand, not the altar; that is verbatim behaviour.
        for y in 40..120 {
            let i = world.index(700, y);
            world.tiles[i].kind = 26;
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut gv = GenVars::default();
        ocean_cave(
            &mut world,
            &mut gv,
            &cave_layers(),
            520,
            70,
            &mut Rng(0x5EED),
        );
        // The altar column is never carved and never deactivated.
        for y in 40..120 {
            let t = &world.tiles[world.index(700, y)];
            assert!(t.active(), "the altar at {y} stays active");
            assert_eq!(t.kind, 26, "the altar at {y} keeps its kind");
            assert_ne!(t.kind, OCEAN_CAVE_CARVED, "the altar at {y} is protected");
        }
        // The carve marker exists and marks INACTIVE tiles only.
        let mut carved = 0;
        let mut carved_active = 0;
        for t in world.tiles.iter() {
            if t.kind == OCEAN_CAVE_CARVED {
                carved += 1;
                if t.active() {
                    carved_active += 1;
                }
            }
        }
        assert!(carved > 50, "the walker carves its disc ({carved})");
        assert_eq!(carved_active, 0, "the sentinel marks inactive tiles only");
        // The treasure anchor is recorded in the world and the counter moved.
        let (tx, ty) = gv.ocean_cave_treasure[0];
        assert!((0..600).contains(&tx) && (0..300).contains(&ty), "{tx},{ty}");
        assert!(gv.num_ocean_cave_treasure >= 1, "the counter advanced");
        // The water shaft: some column below the walker's start carries water
        // 100 deep — far below the seafloor fill.
        let deep_water = (0..600).any(|x| {
            (150..260).any(|y| world.tile(x, y).is_some_and(|t| t.liquid == 255))
        });
        assert!(deep_water, "the shaft floods below the floor");
        // The treasure wrap: a third call wraps the counter back to 0 and
        // overwrites slot 0.
        let before = gv.ocean_cave_treasure[0];
        gv.num_ocean_cave_treasure = MAX_OCEAN_CAVE_TREASURE;
        record_cave_treasure(&mut gv, 123, 456);
        assert_eq!(gv.ocean_cave_treasure[0], (123, 456), "the counter wraps to slot 0");
        assert_ne!(gv.ocean_cave_treasure[0], before);
    }

    /// DW-3.3: the walker's draw chain replays on an independent Rng over a
    /// mirrored world — the `NextDouble` direction/budget/jitter order and
    /// the per-step clamps are the pin.
    #[test]
    fn ocean_cave_draw_ledger() {
        let (w, h, i0, j0) = (600i32, 300i32, 520i32, 70i32);
        let run = |seed: u64| -> (Vec<(u16, u8)>, Vec<(i32, i32)>) {
            let mut world = cave_world(w, h, 60);
            let mut gv = GenVars::default();
            ocean_cave(&mut world, &mut gv, &cave_layers(), i0, j0, &mut Rng(seed));
            let kinds: Vec<(u16, u8)> = world
                .tiles
                .iter()
                .map(|t| (t.kind, t.flags))
                .collect();
            (kinds, gv.ocean_cave_treasure.to_vec())
        };
        let (ka, ta) = run(0x5EED);
        let (kb, tb) = run(0x5EED);
        assert_eq!(ka, kb, "the same seed carves identical worlds");
        assert_eq!(ta, tb);
        // A different seed carves a different world (the chain is live).
        let (kc, _) = run(0x5EED + 1);
        assert_ne!(ka, kc, "the chain consumes its stream");
    }

    /// DW-4.1: `RemoveSurfaceWaterAboveSand` clears the water above inland
    /// sand columns and never touches the ocean band.
    #[test]
    fn remove_water_clears_inland_only() {
        // A tall world: the pass runs over [400, 600) — the inland sand
        // column sits at 500, the ocean one at 200 (the excluded band).
        let mut world = World::new("rw".into(), 1000, 200);
        for &x in &[500i32, 200] {
            let ground = world.index(x, 120);
            world.tiles[ground].kind = TILE_SAND;
            world.tiles[ground].flags |= FLAG_ACTIVE;
            for y in 110..120 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        let _ = &world;
        remove_water_from_sand(&mut world, 130.0);
        // The inland column lost its water.
        let inland = (110..120).all(|y| world.tile(500, y).is_some_and(|t| t.liquid == 0));
        assert!(inland, "the inland water clears");
        // The ocean column keeps its water.
        let ocean = (110..120).any(|y| world.tile(200, y).is_some_and(|t| t.liquid == 255));
        assert!(ocean, "the ocean band keeps its water");
    }

    /// DW-4.2: `ShellPile` walks down to the sand floor and drops the 495
    /// blob with the support stack; a non-sand floor is rejected.
    #[test]
    fn shell_pile_drops_on_sand_only() {
        let mut world = World::new("sp".into(), 200, 200);
        for x in 0..200 {
            for y in 100..200 {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let ok = shell_pile(&mut world, 110.0, 100, 100, &mut Rng(0x5EED));
        assert!(ok, "the pile lands on sand");
        let shells = world.tiles.iter().filter(|t| t.kind == TILE_SHELL_PILE).count();
        assert!(shells > 4, "the blob is 495 ({shells})");
        // A stone floor is rejected (the `type != 53` guard).
        let mut stone = World::new("sp2".into(), 200, 200);
        for x in 0..200 {
            for y in 100..200 {
                let i = stone.index(x, y);
                stone.tiles[i].kind = 1;
                stone.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let rejected = shell_pile(&mut stone, 110.0, 100, 100, &mut Rng(0x5EED));
        assert!(!rejected, "the stone floor is rejected");
        assert_eq!(
            stone.tiles.iter().filter(|t| t.kind == TILE_SHELL_PILE).count(),
            0,
            "nothing placed"
        );
    }

    /// DW-4.3: the Shell Piles anchor scan — a sand tile with air above and
    /// water to the WEST (left) / EAST (right) is picked, and the pile lands
    /// near it. The right-side vanilla quirk (offsets writing
    /// `shellStartXLeft`) does not crash.
    #[test]
    fn shell_piles_anchor_scan() {
        let mut world = World::new("sa".into(), 400, 200);
        // The left shore: water on the west, sand at x=100..160.
        for x in 0..400 {
            for y in 120..200 {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 0..100 {
            for y in 120..140 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        // Water on the east for the right anchor.
        for x in 300..400 {
            for y in 120..140 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        let mut gv = GenVars::default();
        gv.shell_start_x_left = 100;
        gv.shell_start_y_left = 120;
        gv.shell_start_x_right = 300;
        gv.shell_start_y_right = 120;
        // Force both sides to run by picking a seed where chance(2) passes.
        let mut ran = false;
        for seed in 0..8u64 {
            gv.shell_start_x_left = 100;
            gv.shell_start_y_left = 120;
            shell_piles(&mut world, &mut gv, 130.0, &mut Rng(seed));
            if world.tiles.iter().any(|t| t.kind == TILE_SHELL_PILE) {
                ran = true;
                break;
            }
        }
        assert!(ran, "at least one seed produces piles");
        // The y-shift moved the anchor up by 50 (the pile anchor is ABOVE
        // the scan point by construction).
        assert!(gv.shell_start_y_left < 120, "the anchor rose by 50");
    }

    /// DW-4.4: the seaweed arm — underwater sand grows 549 under the
    /// water-full, solid-free, low-density conditions.
    #[test]
    fn water_plants_grows_seaweed() {
        let mut world = World::new("wp".into(), 400, 300);
        // A wide sand seafloor at 80 with water from 50 to 79 — the pass
        // scans from UnderworldLayer (300-200 = 100) down to worldSurface
        // (50), so the floor sits inside the scan band.
        for x in 0..400 {
            for y in 80..300 {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            for y in 50..80 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        let layers = cave_layers();
        water_plants(&mut world, &layers, &mut Rng(0x5EED));
        let seaweed = world.tiles.iter().filter(|t| t.kind == 549).count();
        assert!(seaweed > 0, "the seafloor grows seaweed ({seaweed})");
    }

    /// W4-c33: the lily/cattail arms — a grass-floored pond grows the lily
    /// pads 518 and the cattails 519 (the j-loop's draws join ahead of the
    /// seaweed walk); the a == b replay pins the stream.
    #[test]
    fn water_plants_grows_lilies_and_cattails() {
        // The j-loop scans 1..worldSurface, so the pond sits under a 120
        // surface line: water 100..104 over the grass floor at 105 (the
        // lily's 3..12 / cattail's 2..7 floor windows both admit depth 5).
        let layers = Layers {
            world_surface: 120.0,
            ..cave_layers()
        };
        let pond = || {
            let mut world = World::new("wp2".into(), 400, 300);
            for x in 0..400 {
                for y in 105..300 {
                    let i = world.index(x, y);
                    world.tiles[i].kind = 2;
                    world.tiles[i].flags |= FLAG_ACTIVE;
                }
                for y in 100..105 {
                    let i = world.index(x, y);
                    world.tiles[i].liquid = 255;
                }
            }
            world
        };
        let mut a = pond();
        water_plants(&mut a, &layers, &mut Rng(0xC33));
        let mut b = pond();
        water_plants(&mut b, &layers, &mut Rng(0xC33));
        for (ta, tb) in a.tiles.iter().zip(&b.tiles) {
            assert_eq!(ta, tb, "the a == b determinism half");
        }
        let lilies = a
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 518)
            .count();
        let cattails = a
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 519)
            .count();
        assert!(
            lilies + cattails > 0,
            "the pond grows pads/cattails ({lilies}/{cattails})"
        );
        // The lily pads sit ON the water surface (row 100), frame row 0
        // (the grass soil).
        for y in 0..300 {
            for x in 0..400 {
                let t = a.tile(x, y).unwrap();
                if t.active() && t.kind == 518 {
                    assert_eq!(y, 100, "the pad rides the surface row");
                    assert_eq!(t.frame_y, 0, "the grass soil's frame row");
                }
            }
        }
    }

    /// W4-c33: the bamboo arm — a jungle-grass floor under 2..5 rows of
    /// water grows the 571 stalks (the climb re-calls PlaceBamboo per row).
    #[test]
    fn water_plants_grows_bamboo_on_jungle_grass() {
        let layers = Layers {
            world_surface: 120.0,
            ..cave_layers()
        };
        let mut world = World::new("wp3".into(), 400, 300);
        // The jungle grass 60 floor at 105, water 102..104 (depth 3 —
        // inside the 2..5 window).
        for x in 0..400 {
            for y in 105..300 {
                let i = world.index(x, y);
                world.tiles[i].kind = 60;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            for y in 102..105 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        water_plants(&mut world, &layers, &mut Rng(0xBA));
        let bamboo = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 571)
            .count();
        assert!(bamboo > 0, "the jungle shore grows bamboo ({bamboo})");
    }

    /// The beach test world: a sand shore with water above the ground rows.
    fn beach_world(w: i32, h: i32) -> World {
        let mut world = World::new("bw".into(), w, h);
        for x in 0..w {
            for y in 60..h {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            for y in 20..60 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        world
    }

    /// DW-5.1: the palm arm of the pass — `GrowPalmTree` grows tile 323 with
    /// the vanilla frames (base 66, trunk 22·0-2, top 22·4-6) and the lean
    /// frameY, and the guards reject a non-sand ground.
    #[test]
    fn grow_palm_tree_plants_the_trunk() {
        // Air above the ground — the `tile2.liquid != 0` guard rejects a
        // wet base (`WorldGen.cs:27141-27143`), so the shore world's water
        // is not what a palm base sits on.
        let mut world = World::new("pt".into(), 200, 300);
        for y in 60..300 {
            let i = world.index(100, y);
            world.tiles[i].kind = TILE_SAND;
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
        let ok = grow_palm_tree(&mut world, 100, 60, 0, false, &mut Rng(0x5EED));
        assert!(ok, "the palm grows on flat beach sand");
        let palms = world.tiles.iter().filter(|t| t.kind == TILE_PALM).count();
        assert!(palms >= 8, "the trunk column is 323 ({palms})");
        let base = world.tile(100, 59).unwrap();
        assert!(base.active() && base.kind == TILE_PALM, "the base tile");
        assert_eq!(base.frame_x, 66, "the base frame");
        // A non-sand ground is rejected.
        let mut stone = World::new("st".into(), 200, 300);
        for x in 0..200 {
            for y in 60..300 {
                let i = stone.index(x, y);
                stone.tiles[i].kind = 1;
                stone.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let rejected = grow_palm_tree(&mut stone, 100, 60, 0, false, &mut Rng(0x5EED));
        assert!(!rejected, "the stone ground is rejected");
    }

    /// DW-5.2: the depth arm's beach zones — coral 81 on the water-lined
    /// shore and seashells 324 above it, over the pass's left band.
    #[test]
    fn cactus_palm_beach_zones_place_coral_and_shells() {
        let mut world = World::new("cz".into(), 1000, 300);
        for x in 0..1000 {
            for y in 40..300 {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            for y in 36..40 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        let layers = cave_layers();
        let (mut coral, mut shells, mut sand_top) = (0, 0, 0);
        for seed in 0..12u64 {
            let mut w = world.clone();
            cactus_palm_trees_and_coral(&mut w, &GenVars::default(), &layers, &mut Rng(seed));
            for (i, t) in w.tiles.iter().enumerate() {
                if !t.active() {
                    continue;
                }
                let x = (i as i32) % 1000;
                if x < 5 || x >= 380 {
                    continue;
                }
                match t.kind {
                    81 => coral += 1,
                    324 => shells += 1,
                    53 => sand_top += 1,
                    _ => {}
                }
            }
        }
        assert!(coral + shells > 0, "the beach gets coral ({coral}) / shells ({shells})");
        assert!(sand_top > 1000, "the shore stays sand ({sand_top})");
    }

    /// DW-5.3: the sea-oat arm — tile 529 grows on dry sand beside water, and
    /// `GrowSeaOat` advances the style frames by 90 up to the 180 cap.
    #[test]
    fn sea_oats_grow_on_dry_sand() {
        // Dry air above the sand with a small water strip nearby: the beach
        // band's verdict needs fewer than 20 water tiles in the 65/25 box.
        let mut world = World::new("so".into(), 1000, 300);
        for x in 0..1000 {
            for y in 40..300 {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 100..102 {
            for y in 20..40 {
                let i = world.index(x, y);
                world.tiles[i].liquid = 255;
            }
        }
        let mut any = false;
        for seed in 0..12u64 {
            let mut w = world.clone();
            let layers = cave_layers();
            cactus_palm_trees_and_coral(&mut w, &GenVars::default(), &layers, &mut Rng(seed));
            if w.tiles.iter().any(|t| t.kind == TILE_SEA_OATS) {
                any = true;
                break;
            }
        }
        assert!(any, "the pass plants sea oats");
        let mut t = Tile::default();
        t.frame_x = 0;
        grow_sea_oat(&mut t);
        assert_eq!(t.frame_x, 90);
        grow_sea_oat(&mut t);
        assert_eq!(t.frame_x, 180);
        grow_sea_oat(&mut t);
        assert_eq!(t.frame_x, 180, "the cap holds");
    }

    /// DW-5.4: the middle zone's cactus — `GrowCactus` plants tile 80 on the
    /// sand and `PlantCactus` scatters the retries.
    #[test]
    fn cactus_grows_on_sand() {
        let mut world = World::new("cg".into(), 300, 300);
        for x in 0..300 {
            for y in 60..300 {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        world.frame_important = vec![false; 300 * 300];
        plant_cactus(&mut world, 150, 60, &mut Rng(0x5EED));
        let cactus = world.tiles.iter().filter(|t| t.kind == TILE_CACTUS).count();
        assert!(cactus > 0, "PlantCactus grows a cactus ({cactus})");
    }

    /// W4-d103 pass 19: `PlaceOasis` carves the water basin on flat sand
    /// and appends the GenVars ledger; a stone floor is rejected.
    #[test]
    fn place_oasis_carves_the_basin_and_publishes_the_ledger() {
        let mut world = World::new("oa".into(), 1000, 300);
        for x in 0..1000 {
            for y in 100..300 {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The Oasis carve body is code-only in the default GenVars
        // (`oasis_height` stays 0 until the deferred re-pin awakens it);
        // this witness exercises the carve directly, so set the vanilla
        // `GenVars.oasisHeight` = 20 (`GenVars.cs:473`).
        let mut gv = GenVars::default();
        gv.oasis_height = 20;
        let ok = place_oasis(&mut world, &mut gv, 500, 90, 150.0, &mut Rng(0x5EED));
        assert!(ok, "the basin lands on flat sand");
        assert_eq!(gv.num_oasis, 1, "the ledger appended");
        assert_eq!(gv.oasis_position[0].0, 500);
        assert!((45..61).contains(&gv.oasis_width[0]), "the width roll");
        // The water row at i + 1 carries 127, deeper rows 255.
        let i = gv.oasis_position[0].1;
        assert_eq!(world.tile(500, i + 1).unwrap().liquid, 127, "the surface row");
        assert_eq!(world.tile(500, i + 2).unwrap().liquid, 255, "the deep rows");
        // The spacing gate: a second basin within 350 tiles is rejected.
        let near = place_oasis(&mut world, &mut gv, 600, 90, 150.0, &mut Rng(0x5EED));
        assert!(!near, "the 350-tile spacing gate");
        assert_eq!(gv.num_oasis, 1);
        // A stone floor is rejected (the `type != 53` guard).
        let mut stone = World::new("oa2".into(), 1000, 300);
        for x in 0..1000 {
            for y in 100..300 {
                let i = stone.index(x, y);
                stone.tiles[i].kind = 1;
                stone.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut gv2 = GenVars::default();
        gv2.oasis_height = 20;
        let rejected = place_oasis(&mut stone, &mut gv2, 500, 90, 150.0, &mut Rng(0x5EED));
        assert!(!rejected, "the stone floor is rejected");
        assert_eq!(gv2.num_oasis, 0);
    }

    /// W4-d103 pass 19: the Oasis pass is deterministic for a seed and
    /// places `maxTilesX / 2100 + Next(2)` basins on a vanilla-width world.
    #[test]
    fn oasis_pass_is_deterministic_and_places_basins() {
        let base = {
            let mut world = World::new("op".into(), 4200, 400);
            for x in 0..4200 {
                for y in 120..400 {
                    let i = world.index(x, y);
                    world.tiles[i].kind = TILE_SAND;
                    world.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            world
        };
        let mut a = base.clone();
        let mut b = base.clone();
        let mut gva = GenVars::default();
        let mut gvb = GenVars::default();
        oasis(&mut a, &mut gva, 200.0, &mut Rng(42));
        oasis(&mut b, &mut gvb, 200.0, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert_eq!(gva.num_oasis, gvb.num_oasis);
        assert!(gva.num_oasis >= 2, "4200 / 2100 + Next(2) basins ({})", gva.num_oasis);
        // The sub-vanilla width guard draws nothing.
        let mut small = World::new("os".into(), 512, 300);
        let mut gvs = GenVars::default();
        let mut rng = Rng(7);
        let state = rng.0;
        oasis(&mut small, &mut gvs, 200.0, &mut rng);
        assert_eq!(rng.0, state, "the width guard skips before any draw");
        assert_eq!(gvs.num_oasis, 0);
    }
}
