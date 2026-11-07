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
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
use crate::world::World;
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
fn reset_to_type(t: &mut Tile, kind: u16) {
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
/// dungeon-side ocean caves only run in the outer band.
const BEACH_DISTANCE: i32 = 380;
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
        if !side_ok || !rng.chance(3) {
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

}
