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
use super::TILE_SAND;

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

}
