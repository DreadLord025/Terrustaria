//! The floating islands — the verbatim port of the `FloatingIslands` pass
//! (`WorldGen.cs:12927-13122`) + the `FloatingIslandHouses` pass
//! (`:17859-17874`) → `IslandHouse` (`:79203-79484`), W4-g44 (the census +
//! residuals list: ANALYSIS "Sky lakes / floating-island residuals").
//!
//! The disc BODIES stay the compact M1.4-WGEN-4 stand-ins (`CloudIsland`
//! `:78132-78538` / `CloudLake` `:78539-79202`'s shrinking-ring walks are
//! the recorded shape-level deviation): the elliptical cloud 189 disc with
//! the ragged rim, the grass cap, the rain-cloud 196 lake bowl. The one
//! CloudIsland slice the house pass consumes — the wall-73 interior flood
//! (`:78421-78442`) — lands verbatim here (`flood_cloud_walls`).
//!
//! The pass mechanics are verbatim: the count `(int)(maxTilesX * 0.0008)`
//! with the errorWorld triple, the `skyLakes` budget (Reset,
//! `:11182-11190`) appended as forced-lake tail islands (`num2 >= num`),
//! the extraFloatingIslands multipliers, the spawn-exclusion re-draw loop
//! (150 → 100), the house-anchor spacing (180 → 100, strict bounds), the
//! column-scan y-band with the `worldSurfaceLow - 50` clamp, the
//! noSurface/errorWorld y-arms, the width attempt budget, the GenVars
//! island ledger and the post-loop `num3 * 2` house cap. The houses are
//! the separate `FloatingIslandHouses` step: sunplate 202 / disc wall 82,
//! the door-side flip, the ground scan, the style-9 door, the chest cycle
//! (S1-A-4), the wall-21 windows, the table/chairs/banners.
//!
//! Deferred (ANALYSIS): the drunk snow/desert island variants + their
//! fountain/palm house arms, the remix/getGood style tables, the skyblock
//! worldgen terrain (the SKYBLOCK bit gates the clear window + the deny
//! gate only), `SquareTileFrame` after the house writes (the chain's
//! framing passes run later — the temple precedent).

use super::pipeline::GenVars;
use super::structures::place_chest;
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, Tile};
use crate::world::World;

pub(crate) const TILE_CLOUD: u16 = 189;
pub(crate) const TILE_RAIN_CLOUD: u16 = 196;
pub(crate) const WALL_CLOUD: u16 = 73;
/// `TileID.Sunplate` (`TileID.cs:1033`) — the island house's frame.
const TILE_SUNPLATE: u16 = 202;
/// `WallID.DiscWall` (`WallID.cs:269`) — the house interior.
const WALL_DISC: u16 = 82;
/// `WallID.Glass` (`WallID.cs:147`) — the house window strips.
const WALL_GLASS: u16 = 21;

fn put_cloud(world: &mut World, x: i32, y: i32, kind: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = kind;
    t.flags |= FLAG_ACTIVE;
    t.liquid = 0;
    t.set_slope(0);
}

/// The `CloudIsland` wall-73 flood (`WorldGen.cs:78421-78442`), verbatim
/// over the port's disc bounds: a cell takes the cloud wall when its 3×3
/// neighbourhood is all active with no foreign wall (`:78430`). This is
/// the slice `IslandHouse`'s ground scan (`:79241`'s `wall > 0`) consumes.
/// `SquareWallFrame` is the chain-framing deviation (the module doc).
fn flood_cloud_walls(world: &mut World, cx: i32, cy: i32, radius: i32) {
    let (x0, x1) = (cx - radius - 20, cx + radius + 20);
    let (y0, y1) = (cy - radius / 2 - 20, cy + radius / 2 + 20);
    for x in x0..=x1 {
        for y in y0..=y1 {
            if x < 1 || y < 1 || x >= world.width - 1 || y >= world.height - 1 {
                continue;
            }
            let mut clear = true;
            for nx in (x - 1)..=(x + 1) {
                for ny in (y - 1)..=(y + 1) {
                    let ok = world.tile(nx, ny).is_some_and(|t| {
                        t.active() && (t.wall == 0 || t.wall == WALL_CLOUD)
                    });
                    if !ok {
                        clear = false;
                    }
                }
            }
            if clear {
                let i = world.index(x, y);
                world.tiles[i].wall = WALL_CLOUD;
            }
        }
    }
}

/// The `CloudIsland` disc (`:78132+`): the elliptical cloud body with the
/// grass cap on the top surface. `lake` islands carve the rain-cloud bowl
/// with the water instead of the grass cap's centre. The compact body is
/// the M1.4-WGEN-4 shape deviation; the wall flood is verbatim.
fn build_island(world: &mut World, cx: i32, cy: i32, radius: i32, lake: bool, rng: &mut super::Rng) {
    for dx in -radius..=radius {
        for dy in -radius / 2..=radius / 2 {
            let px = cx + dx;
            let py = cy + dy;
            if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                continue;
            }
            // The ellipse: the horizontal radius full, the vertical halved.
            let ex = dx as f32 / radius as f32;
            let ey = dy as f32 / (radius as f32 * 0.5);
            let d = (ex * ex + ey * ey).sqrt();
            if d > 1.0 {
                continue;
            }
            // The wobble on the boundary (`Next(80, 120) * 0.01`'s ragged
            // rim — the cells at 0.82..1.0 roll 50/50).
            if d > 0.82 && rng.range(0, 2) == 0 {
                continue;
            }
            put_cloud(world, px, py, TILE_CLOUD);
        }
    }
    // The grass cap on the top surface (the compact 2-row cap — the C#'s
    // per-column ragged grass line is the shape deviation).
    for dx in -(radius - 2)..=(radius - 2) {
        for dy in 0..2 {
            let px = cx + dx;
            let py = cy - radius / 2 + dy;
            if px < 2 || py < 2 || px >= world.width - 2 {
                continue;
            }
            if world
                .tile(px, py)
                .is_some_and(|t| t.active() && t.kind == TILE_CLOUD)
            {
                let i = world.index(px, py);
                world.tiles[i].kind = if dy == 0 { 2 } else { 0 };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
    }
    if lake {
        // The rain-cloud bowl with the water (`CloudLake`'s compact shape).
        let half = (radius / 3).max(4);
        for dx in -half..=half {
            for dy in 0..4 {
                let px = cx + dx;
                let py = cy - radius / 2 + 2 + dy;
                if px < 2 || py < 2 || px >= world.width - 2 {
                    continue;
                }
                let i = world.index(px, py);
                if dy == 3 || dx.abs() == half {
                    let t = &mut world.tiles[i];
                    t.kind = TILE_RAIN_CLOUD;
                    t.flags |= FLAG_ACTIVE;
                    t.set_slope(0);
                } else {
                    let t = &mut world.tiles[i];
                    *t = Tile::default();
                    t.liquid = 255;
                    t.set_liquid_kind(crate::tile::LiquidKind::Water);
                }
            }
        }
    }
    // The wall-73 interior flood (`:78421-78442`) — the house ground scan
    // keys on it. Runs for both bodies (the C# lake carries its own flood
    // at `:78800-78824`; the port's single flood covers both).
    flood_cloud_walls(world, cx, cy, radius);
}

/// `SecretSeed.Variations.errorWorldAdjustment` (`WorldGen.cs:329-336`):
/// `< 1` active seeds → 4, else `(int)(value * ((count + 3) / 4))` — the
/// C#'s INTEGER division inside the cast.
pub(crate) fn error_world_adjustment(value: f64, world: &World) -> i32 {
    let count = crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds);
    if count < 1 {
        return 4;
    }
    (value * ((count + 3) / 4) as f64) as i32
}

/// `Skyblock.denyFloatingIslands` (`WorldGen.cs:3087-3096`): a skyblock
/// world without extraFloatingIslands denies both island passes. The
/// port's SKYBLOCK bit stands in for `WorldGen.skyblockWorldGen` (the
/// skyblock terrain itself is M5-SEED-1's recorded deviation).
fn deny_floating_islands(world: &World) -> bool {
    world.seeds.contains(SeedFlags::SKYBLOCK)
        && !world
            .micro_seeds
            .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
}

/// `SecretSeed.Variations.noSurfaceNoFloatingIslands`
/// (`WorldGen.cs:169-178`): noSurface && !errorWorld → !extra.
fn no_surface_no_floating_islands(world: &World) -> bool {
    if world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE)
        && !world.micro_seeds.contains(MicroSeedFlags::ERROR_WORLD)
    {
        return !world
            .micro_seeds
            .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS);
    }
    false
}

/// `SecretSeed.Variations.extraFloatingIslandsNormalAmount`
/// (`WorldGen.cs:233-243`): an extraFloatingIslands world that is ALSO a
/// skyblock world keeps the normal (non-multiplied) island count.
fn extra_floating_islands_normal_amount(world: &World) -> bool {
    world
        .micro_seeds
        .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
        && world.seeds.contains(SeedFlags::SKYBLOCK)
}

/// `SecretSeed.Variations.extraFloatingIslandsReducedAmount`
/// (`WorldGen.cs:245-255`): the halved multiplier when the seed list is
/// crowded — `activeSecretSeedCount < 6` reads `noSurface.Enabled`,
/// otherwise `true`.
fn extra_floating_islands_reduced_amount(world: &World) -> bool {
    if !world
        .micro_seeds
        .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
        || crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds) < 6
    {
        world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE)
    } else {
        true
    }
}

/// The `FloatingIslands` pass (`WorldGen.cs:12927-13122`), verbatim: the
/// count + multipliers, the width attempt budget, the spawn-exclusion
/// re-draw, the house-anchor spacing, the column-scan y-band, the
/// `num2 >= num` lake tail, the GenVars ledger writes and the post-loop
/// house cap. Returns the placed-island count (`GenVars.numIslandHouses`
/// after the cap). The disc bodies ride the compact `build_island`.
///
/// Sub-vanilla valves (deterministic skips where vanilla's 4200×1200
/// minimum never degenerates): a column with no active tile below row
/// 200 fails the site (`flag2` stays false — the C# rolls
/// `Next(90, -100)` and throws); an empty y-band skips the island before
/// the draw (the C# `Next(90, num6 - 100)` throws there too).
pub(crate) fn pass_floating_islands(
    world: &mut World,
    gv: &mut GenVars,
    rng: &mut super::Rng,
    surface_low: f64,
) -> i32 {
    // `:12929`: the deny gates.
    if deny_floating_islands(world) || no_surface_no_floating_islands(world) {
        return 0;
    }
    // `:12931-12932`: the pass zeroes both counters.
    gv.num_island_houses = 0;
    gv.sky_island_house_count = 0;
    gv.island_house_x.clear();
    gv.island_house_y.clear();
    gv.island_style.clear();
    gv.sky_lake.clear();
    let extra = world
        .micro_seeds
        .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS);
    let error = world.micro_seeds.contains(MicroSeedFlags::ERROR_WORLD);
    let no_surface = world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE);
    let skyblock = world.seeds.contains(SeedFlags::SKYBLOCK);
    // `:12934-12939`: `num = (int)(maxTilesX * 0.0008)`, the errorWorld
    // triple BEFORE `num3 = num`.
    let mut num = (world.width as f64 * 0.0008) as i32;
    let mut num2 = 0;
    if error && error_world_adjustment(1.0, world) < 3 {
        num *= 3;
    }
    // `:12940-12954`: `num3` (the house cap's base) + the multipliers —
    // `skyLakes` takes the same multiplier as `num`.
    let num3 = num;
    let mut flag = false;
    if extra && !extra_floating_islands_normal_amount(world) {
        if extra_floating_islands_reduced_amount(world) {
            num *= 2;
            gv.sky_lakes *= 2;
        } else {
            num *= 10;
            gv.sky_lakes *= 10;
        }
    }
    // `:12955-12956`: `num4 = num + GenVars.skyLakes`.
    let num4 = num + gv.sky_lakes;
    let underworld_layer = world.height - 200; // `Main.cs:2790`
    for i in 0..num4 {
        // `:12959-12964`: the attempt budget — the width, `/42` skyblock.
        let mut num5 = world.width;
        if skyblock {
            num5 /= 42;
        }
        while {
            num5 -= 1;
            num5 > 0
        } {
            // `:12968-12979`: the x draw + the spawn-exclusion RE-DRAW
            // loop (150 → 100 under extra; errorWorld skips it entirely).
            let mut num7 =
                rng.range((world.width as f64 * 0.1) as i32, (world.width as f64 * 0.9) as i32);
            let num8 = if extra { 100 } else { 150 };
            if !error {
                while num7 > world.width / 2 - num8 && num7 < world.width / 2 + num8 {
                    num7 = rng.range(
                        (world.width as f64 * 0.1) as i32,
                        (world.width as f64 * 0.9) as i32,
                    );
                }
            }
            // `:12981-12993`: the spacing against the placed-island
            // anchors — strict bounds, 180 → 100 under multiplied extra.
            let num9 = if extra && !extra_floating_islands_normal_amount(world) {
                100
            } else {
                180
            };
            let mut flag2 = true;
            for j in 0..gv.num_island_houses as usize {
                let hx = gv.island_house_x[j];
                if num7 > hx - num9 && num7 < hx + num9 {
                    flag2 = false;
                    break;
                }
            }
            if !flag2 {
                continue;
            }
            // `:12996-13019`: the y anchor — noSurface's deep roll,
            // skyblock's fixed line, else the column scan from row 200.
            flag2 = false;
            let num6;
            if no_surface {
                // The sub-vanilla valve: `Next(200, UnderworldLayer - 200)`
                // throws on an inverted band (the C# never generates one).
                if underworld_layer - 200 < 200 {
                    continue;
                }
                num6 = rng.range(200, underworld_layer - 200);
                flag2 = true;
            } else if skyblock {
                num6 = (world.world_surface / 2.0 + 100.0) as i32;
                flag2 = true;
            } else {
                let mut found = None;
                for k in 200..(world.world_surface as i32) {
                    if world.tile(num7, k).is_some_and(|t| t.active()) {
                        found = Some(k);
                        break;
                    }
                }
                match found {
                    Some(k) => {
                        num6 = k;
                        flag2 = true;
                    }
                    None => continue, // the sub-vanilla valve (the C# throws)
                }
            }
            if !flag2 {
                continue;
            }
            // `:13022-13023`: `num11 = Next(90, num6 - 100)` — the empty
            // band skips (the sub-vanilla valve; `Next(90, 90)` returns 90
            // without a draw, the port's `range` mirrors it).
            if num6 - 100 < 90 {
                continue;
            }
            let mut num11 = rng.range(90, num6 - 100);
            // `:13024-13040`: the ±150/±75 clear window — SKYBLOCK only.
            if skyblock {
                let mut clear = true;
                for l in (num7 - 150)..=(num7 + 150) {
                    for m in (num11 - 75)..=(num11 + 75) {
                        if world.tile(l, m).is_some_and(|t| t.active()) {
                            clear = false;
                        }
                    }
                }
                if !clear {
                    continue;
                }
            }
            // `:13042-13049`: the surface-low clamp + the errorWorld
            // deep re-roll (`Next(4) != 0`).
            if !no_surface {
                num11 = num11.min(surface_low as i32 - 50);
                if error && rng.range(0, 4) != 0 {
                    // The sub-vanilla valve: the deep re-roll band
                    // `Next(worldSurface + 50, UnderworldLayer - 50)`
                    // throws inverted (the C# never generates one) — the
                    // port keeps the clamped y instead.
                    if underworld_layer - 50 > world.world_surface as i32 + 50 {
                        num11 =
                            rng.range(world.world_surface as i32 + 50, underworld_layer - 50);
                    }
                }
            }
            // `:13050-13065`: the lake decision — the `num2 >= num` tail
            // force, then the extra `i > 4` force-then-1-in-4 arm.
            let mut flag3 = num2 >= num;
            if extra && i > 4 {
                if !flag {
                    flag3 = true;
                } else if rng.range(0, 4) == 0 {
                    flag3 = true;
                }
            }
            // `:13066-13106`: the body. The drunk snow/desert variants and
            // the remix/getGood num10 style table are deferred (the module
            // doc) — the classic arm is style 0 + the compact disc.
            let num10 = 0;
            // The compact disc's radius draw (the M1.4-WGEN-4 stand-in for
            // `CloudIsland`/`CloudLake`'s `Next(100,150)` ring budget).
            let radius = rng.range(40, 61);
            if flag3 {
                flag = true;
                build_island(world, num7, num11, radius, true, rng);
            } else {
                build_island(world, num7, num11, radius, false, rng);
            }
            // `:13069`/`:13074` + `:13108-13112`: the ledger writes. The C#
            // stores `num11` — the disc TOP (CloudIsland hangs down from j,
            // `:78224`'s `m > num13 ≈ j`). The port's compact disc is
            // CENTERED at num11 (the M1.4-WGEN-4 shape), so its top sits at
            // `num11 - radius/2`; the ledger stores that row so the verbatim
            // IslandHouse ground scan (`j-15`, `:79238`) reaches the grass
            // cap exactly as the C#'s reaches its disc top.
            let disc_top = num11 - radius / 2;
            gv.sky_lake.push(flag3);
            gv.island_house_x.push(num7);
            gv.island_house_y.push(disc_top);
            gv.island_style.push(num10);
            gv.num_island_houses += 1;
            num2 += 1;
            // `:13041`: `num5 = -1` — the site succeeded, leave the budget
            // loop (the `while (--num5 > 0)` test fails on the next turn).
            num5 = -1;
        }
    }
    // `:13117-13120`: the house cap on the PRE-multiplier count — the
    // discs stay, the ledger truncates (the houses pass reads it).
    if extra && gv.num_island_houses > num3 * 2 {
        gv.num_island_houses = num3 * 2;
    }
    gv.num_island_houses
}

// --- the FloatingIslandHouses pass (`:17859-17874`) → IslandHouse ---------

/// The `IslandHouse` furniture writes — the `PlaceTile` arms the classic
/// house uses, each with its C# gate. `SquareTileFrame` is the
/// chain-framing deviation (the module doc).
mod furniture {
    use super::*;
    use crate::tiledata as td;

    /// `WorldGen.SolidTile` (`WorldGen.cs:70111-70133`) — the house-scan
    /// predicate; the port's `edit::solid_tile` twin (OOB → false).
    pub(super) fn solid(world: &World, x: i32, y: i32) -> bool {
        crate::edit::solid_tile(world, x, y)
    }

    /// `WorldGen.SolidTile2` (`:70134-70150`) — the furniture floor gate.
    pub(super) fn solid2(world: &World, x: i32, y: i32) -> bool {
        crate::edit::solid_tile2(world, x, y)
    }

    fn put(world: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        if x < 0 || y < 0 || x >= world.width || y >= world.height {
            return;
        }
        let i = world.index(x, y);
        let t = &mut world.tiles[i];
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = fx;
        t.frame_y = fy;
        // The frame-important boot bits (`Main.cs:7637`/`:7641`/`:7649` —
        // the door 10, the table 14, the chair 15): the framer keeps the
        // stored frames (the temple/dungeon `mark_important` precedent).
        if world.frame_important.len() > kind as usize {
            world.frame_important[kind as usize] = true;
        }
    }

    /// `PlaceDoor` (`WorldGen.cs:31773-31803`): the ceiling/floor gate,
    /// then three rows with one `Next(3)` frame-variant roll EACH
    /// (`:31786`/`:31790`/`:31794`).
    pub(super) fn place_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut super::super::Rng) -> bool {
        let num3 = 54 * (style / 36);
        let num4 = 54 * (style % 36);
        let ceiling = world
            .tile(x, y - 2)
            .is_some_and(|t| t.active() && !t.inactive() && td::TILE_SOLID.has(t.kind));
        if !ceiling || !solid(world, x, y + 2) {
            return false;
        }
        for row in -1..=1 {
            let fx = (num3 + rng.range(0, 3) * 18) as i16;
            put(world, x, y + row, 10, fx, (num4 + (row + 1) * 18) as i16);
        }
        true
    }

    /// `PlaceTile`'s door arm (`WorldGen.cs:59843-59883`): the up-hang
    /// (two free rows under a solid ceiling) else the down-hang (two free
    /// rows over a solid floor), else the refusal.
    pub(super) fn place_tile_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut super::super::Rng) -> bool {
        let free = |w: &World, dy: i32| w.tile(x, y + dy).is_some_and(|t| !t.active());
        let solid_at = |w: &World, dy: i32| {
            w.tile(x, y + dy)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
        };
        // The C# branch conditions, evaluated before the mutable place.
        let up = free(world, -1) && free(world, -2) && solid_at(world, -3);
        let down = free(world, 1) && free(world, 2) && solid_at(world, 3);
        if up {
            place_door(world, x, y - 1, style, rng)
        } else if !down {
            false
        } else {
            place_door(world, x, y + 1, style, rng)
        }
    }

    /// `Place3x2`'s default path (`WorldGen.cs:52066-52195`) — the table
    /// 14: the 3×2 body empty, the floor `SolidTile2` under all three
    /// columns, `frameX = 54 * style + col * 18`, `frameY = row * 18`.
    pub(super) fn place_table(world: &mut World, x: i32, y: i32, style: i32) -> bool {
        if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
            return false;
        }
        let mut ok = true;
        for col in (x - 1)..(x + 2) {
            for row in (y - 1)..(y + 1) {
                if world.tile(col, row).is_some_and(|t| t.active()) {
                    ok = false;
                }
            }
            if !solid2(world, col, y + 1) {
                ok = false;
            }
        }
        if !ok {
            return false;
        }
        for col in 0..3 {
            for row in 0..2 {
                put(
                    world,
                    x - 1 + col,
                    y - 1 + row,
                    14,
                    (54 * style + col * 18) as i16,
                    (row * 18) as i16,
                );
            }
        }
        true
    }

    /// `Place1x2` (`WorldGen.cs:46652-46679`) — the chair 15: the floor
    /// `SolidTile2`, the top cell free, `frameY = 40 * style + row * 18`,
    /// `frameX = 0` (the sapling roll is kind-20 only).
    pub(super) fn place_chair(world: &mut World, x: i32, y: i32, style: i32) -> bool {
        if !solid2(world, x, y + 1) || world.tile(x, y - 1).is_some_and(|t| t.active()) {
            return false;
        }
        for row in 0..2 {
            put(world, x, y - 1 + row, 15, 0, (40 * style + row * 18) as i16);
        }
        true
    }
}

/// `IslandHouse` (`WorldGen.cs:79203-79484`), the classic arm: the
/// dungeon-wall abort, the door-side flip + the size rolls, the ground
/// scan, the sunplate shell + the disc-wall interior, the door carve +
/// the style-9 door, the opposite wall column, the chest (the S1-A-4
/// cycle), the wall-21 windows, the table/chairs/banners. The drunk
/// fountain/palm arms and the remix furniture variants are deferred
/// (the module doc). The chest placement rides the port's
/// `place_chest` + `stock_placed_chest` seam (the `AddBuriedChest`
/// floor-scan is the compact stand-in — the house floor is already
/// carved at `num13`, the C# scan's first solid row).
#[allow(clippy::too_many_arguments)]
fn island_house(
    world: &mut World,
    i: i32,
    j: i32,
    _island_style: i32,
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
    get_good: bool,
    sky_count: &mut u32,
) {
    // `:79206-79216`: the ±30 dungeon-wall abort (`Main.wallDungeon` +
    // the temple wall 87 + the evil walls 3/83).
    for k in (i - 30)..=(i + 30) {
        for l in (j - 30)..=(j + 30) {
            if let Some(t) = world.tile(k, l) {
                if super::dungeon_features::is_dungeon_wall(t.wall)
                    || t.wall == 87
                    || t.wall == 3
                    || t.wall == 83
                {
                    return;
                }
            }
        }
    }
    // `:79224-79230`: the door-side flip + the size rolls (draw order).
    let num2 = if rng.range(0, 2) == 0 { -1 } else { 1 };
    let num3 = rng.range(7, 12);
    let num4 = rng.range(5, 7);
    // `:79231-79247`: the ground scan at the door-side column — the first
    // row in [j-15, j+30) with a wall, active and solid (the leaf/wood
    // kinds excluded). Below the surface line the C# takes j-5 flat; the
    // port's islands sit above it (the y clamp), so the scan is the arm
    // that runs — the j-5 branch is kept for the errorWorld deep islands.
    let scan_x = i + (num3 + 2) * num2;
    let mut ground_y = None;
    if j as f64 > world.world_surface {
        ground_y = Some(j - 5);
    } else {
        for m in (j - 15)..(j + 30) {
            let kind_ok = world.tile(scan_x, m).is_some_and(|t| {
                t.wall > 0
                    && t.active()
                    && t.kind != 192
                    && t.kind != 191
                    && t.kind != 384
            });
            if kind_ok && furniture::solid(world, scan_x, m) {
                ground_y = Some(m - 1);
                break;
            }
        }
    }
    // The C# leaves `vector2D.Y` at 0 when the scan finds nothing (the
    // house builds at the world top — a degenerate case the port's
    // discs never produce: the wall flood guarantees a hit). The port
    // skips instead of building at row 0 (the sub-vanilla valve).
    let Some(vy) = ground_y else {
        return;
    };
    // `:79249-79283`: the outer sunplate shell (the top corners skipped).
    let (num5, num6) = (i - num3 - 1, i + num3 + 1);
    let (num7, num8) = (vy - num4 - 1, vy + 2);
    for n in num5..=num6 {
        for y in (num7 - 1)..(num8 + 1) {
            if y == num7 - 1 && (n == num5 || n == num6) {
                continue;
            }
            if n < 0 || y < 0 || n >= world.width || y >= world.height {
                continue;
            }
            let idx = world.index(n, y);
            let t = &mut world.tiles[idx];
            t.kind = TILE_SUNPLATE;
            t.flags |= FLAG_ACTIVE;
            t.liquid = 0;
            t.wall = 0;
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
    // `:79284-79314`: the interior carve — the disc wall where the cell
    // carried no wall (the top corners skipped).
    let (ix0, ix1) = (i - num3, i + num3);
    let (iy0, iy1) = (vy - num4, vy + 1);
    for n in ix0..=ix1 {
        for y in iy0..iy1 {
            if y == iy0 && (n == ix0 || n == ix1) {
                continue;
            }
            if n < 0 || y < 0 || n >= world.width || y >= world.height {
                continue;
            }
            let idx = world.index(n, y);
            if world.tiles[idx].wall == 0 {
                let t = &mut world.tiles[idx];
                t.flags &= !FLAG_ACTIVE;
                t.wall = WALL_DISC;
            }
        }
    }
    // `:79315-79337`: the door carve (5 wide, 3 tall) + the style-9 door
    // (the remix variants deferred).
    let mut num12 = i + (num3 + 1) * num2;
    let num13 = vy;
    for n in (num12 - 2)..=(num12 + 2) {
        for dy in 0..3 {
            if n < 0 || num13 - dy < 0 || n >= world.width || num13 - dy >= world.height {
                continue;
            }
            let idx = world.index(n, num13 - dy);
            world.tiles[idx].flags &= !FLAG_ACTIVE;
        }
    }
    furniture::place_tile_door(world, num12, num13, 9, rng);
    // `:79338-79347`: the opposite wall column.
    num12 = i + (num3 + 1) * -num2 - num2;
    for y in iy0..=(iy1 + 1) {
        if num12 < 0 || y < 0 || num12 >= world.width || y >= world.height {
            continue;
        }
        let idx = world.index(num12, y);
        let t = &mut world.tiles[idx];
        t.kind = TILE_SUNPLATE;
        t.flags |= FLAG_ACTIVE;
        t.liquid = 0;
        t.wall = 0;
        t.flags &= !FLAG_HALF_BRICK;
        t.set_slope(0);
    }
    // `:79348-79376`: the chest — the `skyIslandHouseCount` cycle (the
    // `> 3 → Next(4)` re-roll), the style 13 / getGood 2, the anchor
    // `(i, num13 - 3)` through the port's AddBuriedChest seam.
    let mut num16 = *sky_count;
    if num16 > 3 {
        num16 = rng.range(0, 4) as u32;
    }
    let main = match num16 {
        0 => 159,
        1 => 65,
        2 => 158,
        _ => 2219,
    };
    let chest_style = if get_good { 2 } else { 13 };
    // `AddBuriedChest(i, num13 - 3, …)` (`:79371`/`:79375` → `:35889-35963`
    // + `:36206`): the downward scan for the first solid row, the chest
    // one row above it at column `i - 1`. The house floor is the sunplate
    // row `num13 + 1`, so the scan lands there and the 2×2 occupies rows
    // `num13 - 1`/`num13` — the port's `place_chest` takes the top-left
    // anchor `(i - 1, k - 2)`.
    let mut chest_anchor = None;
    for k in (num13 - 3)..(world.height - 10) {
        if furniture::solid(world, i, k) {
            chest_anchor = Some((i - 1, k - 2));
            break;
        }
    }
    if let Some((cx, cy)) = chest_anchor {
        let style = super::structures::resolved_chest_style(
            chest_style,
            main,
            cy,
            world.world_surface,
        );
        if place_chest(world, cx, cy, style) {
            super::chest_stock::stock_placed_chest(
                world,
                stock.as_deref_mut(),
                rng,
                cx,
                cy,
                style,
                21,
                main,
                false,
            );
        }
    }
    // `:79377-79392`: the drunk fountain arm — deferred (island_style is
    // always 0 on the classic path; the module doc).
    // `:79394`: the counter increments once per house, after the place.
    *sky_count += 1;
    // `:79395-79419`: the wall-21 window strips (the classic arm — the
    // remix gate reads false).
    let num20 = i - num3 / 2 + 1;
    let num21 = i + num3 / 2 - 1;
    let num22 = if num3 > 10 { 2 } else { 1 };
    let num23 = (iy0 + iy1) / 2 - 1;
    for n in (num20 - num22)..=(num20 + num22) {
        for y in (num23 - 1)..=(num23 + 1) {
            if n >= 0 && y >= 0 && n < world.width && y < world.height {
                let idx = world.index(n, y);
                world.tiles[idx].wall = WALL_GLASS;
            }
        }
    }
    for n in (num21 - num22)..=(num21 + num22) {
        for y in (num23 - 1)..=(num23 + 1) {
            if n >= 0 && y >= 0 && n < world.width && y < world.height {
                let idx = world.index(n, y);
                world.tiles[idx].wall = WALL_GLASS;
            }
        }
    }
    // `:79420-79443`: the table + the two chairs (the classic arm), the
    // left chair's frameX flip (`:79442-79443`).
    let num28 = i + (num3 / 2 + 1) * -num2;
    furniture::place_table(world, num28, iy1 - 1, 7);
    furniture::place_chair(world, num28 - 2, iy1 - 1, 10);
    furniture::place_chair(world, num28 + 2, iy1 - 1, 10);
    // `:79442-79443`: the left chair's frameX flip — UNCONDITIONAL in the
    // C# (it adds 18 to whatever sits at the two cells, even a failed
    // chair place).
    for dy in 0..2 {
        let (x, y) = (num28 - 2, iy1 - 1 - dy);
        if x >= 0 && y >= 0 && x < world.width && y < world.height {
            let idx = world.index(x, y);
            world.tiles[idx].frame_x += 18;
        }
    }
    // `:79444-79461`: the three banners (`Next(7,10)` styles) — the two
    // ceiling corners + the door-side top corner.
    super::dungeon_features::place_banner(world, ix0 + 1, iy0, 91, rng.range(7, 10));
    super::dungeon_features::place_banner(world, ix1 - 1, iy0, 91, rng.range(7, 10));
    let (bx, by) = if num2 > 0 { (ix0, iy0 + 1) } else { (ix1, iy0 + 1) };
    super::dungeon_features::place_banner(world, bx, by, 91, rng.range(7, 10));
    // `:79462-79483`: the drunk palm arm — deferred (island_style 1).
}

/// The `FloatingIslandHouses` pass (`WorldGen.cs:17859-17874`): a house
/// on every non-lake island of the ledger, in placement order. The
/// `Skyblock.denyFloatingIslands` gate (`:17861`) matches. The ledger
/// rides a snapshot (the C# reads the GenVars statics while `IslandHouse`
/// mutates only `skyIslandHouseCount` — the port's `&mut GenVars` carries
/// that counter back the same way).
pub(crate) fn pass_floating_island_houses(
    world: &mut World,
    gv: &mut GenVars,
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
    get_good: bool,
) {
    if deny_floating_islands(world) {
        return;
    }
    let count = gv.num_island_houses as usize;
    let ledger: Vec<(i32, i32, i32, bool)> = (0..count)
        .map(|idx| {
            (
                gv.island_house_x[idx],
                gv.island_house_y[idx],
                gv.island_style[idx],
                gv.sky_lake[idx],
            )
        })
        .collect();
    for (x, y, style, lake) in ledger {
        // `:17868-17871`: the lake skip.
        if lake {
            continue;
        }
        island_house(
            world,
            x,
            y,
            style,
            rng,
            stock.as_deref_mut(),
            get_good,
            &mut gv.sky_island_house_count,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::noise::Rng;

    /// A wide sky world with the ground far below the island band: the
    /// column scan (`:13010-13018`) finds the ground row 520, the y roll
    /// `Next(90, 420)` stays well above it.
    fn sky_world(width: i32, height: i32, ground: i32) -> World {
        let mut w = World::new("fi".into(), width, height);
        for x in 0..width {
            for y in ground..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = (ground + 25) as f64;
        w
    }

    fn gv_with_lakes(width: i32) -> GenVars {
        let mut gv = GenVars::default();
        // The Reset skyLakes block (`WorldGen.cs:11182-11190`).
        gv.sky_lakes = 1;
        if width > 8000 {
            gv.sky_lakes += 1;
        }
        if width > 6000 {
            gv.sky_lakes += 1;
        }
        gv
    }

    fn count_kind(world: &World, kind: u16) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == kind)
            .count()
    }

    /// A guaranteed house ground: a wide solid dirt slab carrying the
    /// cloud wall, so `IslandHouse`'s ground scan (`:79238-79247`) always
    /// finds `wall > 0 && active && SolidTile` regardless of the compact
    /// disc's exact cap geometry. The slab top row is `top`; the scan
    /// lands on it and the house floor sits at `top`.
    fn house_ground(w: &mut World, cx: i32, top: i32) {
        for x in (cx - 40)..=(cx + 40) {
            for y in top..=(top + 6) {
                let i = w.index(x, y);
                w.tiles[i].kind = 0; // dirt — solid, non-solid-top
                w.tiles[i].flags |= FLAG_ACTIVE;
                w.tiles[i].wall = WALL_CLOUD;
            }
        }
    }

    /// The count formula + the skyLakes budget (`:12934`, `:11182-11190`,
    /// `:12955`): 4200 → 3+1, 6400 → 5+2, 8400 → 6+3 islands, the tail
    /// `skyLakes` of them lakes (`num2 >= num`, `:13051`).
    #[test]
    fn the_count_formula_and_the_sky_lakes_tail() {
        for (width, dry, lakes) in [(4200i32, 3usize, 1usize), (6400, 5, 2), (8400, 6, 3)] {
            let mut w = sky_world(width, 1200, 520);
            let mut gv = gv_with_lakes(width);
            let placed =
                pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0);
            assert_eq!(placed as usize, dry + lakes, "width {width}");
            assert_eq!(gv.sky_lake.len(), dry + lakes);
            // The FIRST `num` islands are dry, the tail `skyLakes` lakes.
            for (idx, lake) in gv.sky_lake.iter().enumerate() {
                assert_eq!(*lake, idx >= dry, "width {width} island {idx}");
            }
        }
    }

    /// The errorWorld triple (`:12936-12939`): a lone errorWorld seed has
    /// `errorWorldAdjustment(1.0) = 1 < 3` → `num *= 3` BEFORE `num3`, so
    /// the 4200 world attempts 9+1 islands and the house cap base triples.
    #[test]
    fn the_error_world_triple_carries_into_the_cap_base() {
        let mut w = sky_world(4200, 1200, 520);
        w.micro_seeds |= MicroSeedFlags::ERROR_WORLD;
        assert_eq!(error_world_adjustment(1.0, &w), 1);
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(7), 520.0);
        // 9 dry + 1 lake attempted; the deep y re-roll may bury some, but
        // every placed island past index 8 is a lake (the num2 >= 9 tail).
        assert!(gv.sky_lake.len() >= 2, "the tripled count places islands");
        let dry = gv.sky_lake.iter().filter(|l| !**l).count();
        assert!(dry <= 9, "at most the tripled num dry islands");
        // The errorWorld spawn-exclusion skip (`:12974`): an island MAY
        // sit over spawn — the classic control never does (below).
    }

    /// The spawn exclusion (`:12969-12979`): the classic 150 half-width
    /// re-draw keeps every anchor out of the spawn band; the extra flag
    /// narrows it to 100.
    #[test]
    fn the_spawn_exclusion_keeps_the_anchors_out() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0);
        assert!(!gv.island_house_x.is_empty());
        for x in &gv.island_house_x {
            assert!(
                *x <= 2100 - 150 || *x >= 2100 + 150,
                "anchor {x} inside the classic spawn band"
            );
        }
    }

    /// The spacing (`:12981-12993`): strict bounds — two anchors exactly
    /// 180 apart are legal, 179 are not. The classic run never places a
    /// pair closer than 180.
    #[test]
    fn the_anchor_spacing_is_strict() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(99), 520.0);
        for a in 0..gv.island_house_x.len() {
            for b in (a + 1)..gv.island_house_x.len() {
                let d = (gv.island_house_x[a] - gv.island_house_x[b]).abs();
                assert!(d >= 180, "anchors {a}/{b} only {d} apart");
            }
        }
    }

    /// The extraFloatingIslands multipliers (`:12942-12954`) + the house
    /// cap (`:13117-13120`): the 10x arm attempts 30+10 islands, the
    /// ledger truncates to `num3 * 2 = 6`; the crowded 2x arm attempts
    /// 6+2 and caps at 6 too; the skyblock passthrough keeps 3+1.
    #[test]
    fn the_multipliers_and_the_post_loop_house_cap() {
        let mut w = sky_world(4200, 1200, 520);
        w.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0);
        assert_eq!(gv.sky_lakes, 10, "the 10x arm multiplies skyLakes");
        assert!(gv.island_house_x.len() > 6, "the discs keep placing");
        assert_eq!(gv.num_island_houses, 6, "the cap is num3 * 2");

        let mut crowded = sky_world(4200, 1200, 520);
        crowded.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS
            | MicroSeedFlags::PAINT_EVERYTHING_GRAY
            | MicroSeedFlags::ERROR_WORLD
            | MicroSeedFlags::RAINS_FOR_A_YEAR
            | MicroSeedFlags::RANDOM_SPAWN
            | MicroSeedFlags::ADD_TELEPORTERS;
        assert_eq!(
            crate::seeds::active_secret_seed_count(crowded.micro_seeds, crowded.seeds),
            6
        );
        let mut gv2 = gv_with_lakes(4200);
        pass_floating_islands(&mut crowded, &mut gv2, &mut Rng(42), 520.0);
        assert_eq!(gv2.sky_lakes, 2, "the reduced 2x arm");

        let mut skyblock = sky_world(4200, 1200, 520);
        skyblock.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        skyblock.seeds |= SeedFlags::SKYBLOCK;
        let mut gv3 = gv_with_lakes(4200);
        pass_floating_islands(&mut skyblock, &mut gv3, &mut Rng(42), 520.0);
        assert_eq!(gv3.sky_lakes, 1, "the skyblock passthrough keeps 1");
        // The unmultiplied 3+1 attempt budget — the ±150/±75 clear window
        // may reject sites, so the placed count is at most 4.
        assert!(gv3.island_house_x.len() <= 4, "3 + 1, unmultiplied");
    }

    /// The pass gates (`:12929`): noSurface alone skips the whole pass
    /// without a draw; skyblock without extra denies it; noSurface WITH
    /// extra runs (the deep y arm, `:12998-13001`).
    #[test]
    fn the_deny_gates_and_the_no_surface_arm() {
        let mut w = sky_world(4200, 1200, 520);
        w.micro_seeds |= MicroSeedFlags::NO_SURFACE;
        let mut gv = gv_with_lakes(4200);
        let mut rng = Rng(42);
        let before = rng.0;
        assert_eq!(pass_floating_islands(&mut w, &mut gv, &mut rng, 520.0), 0);
        assert_eq!(rng.0, before, "the gated pass draws nothing");

        let mut sb = sky_world(4200, 1200, 520);
        sb.seeds |= SeedFlags::SKYBLOCK;
        let mut gv2 = gv_with_lakes(4200);
        assert_eq!(
            pass_floating_islands(&mut sb, &mut gv2, &mut Rng(42), 520.0),
            0,
            "skyblock without extra denies the islands"
        );

        let mut deep = sky_world(4200, 1200, 520);
        deep.micro_seeds |= MicroSeedFlags::NO_SURFACE | MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        let mut gv3 = gv_with_lakes(4200);
        pass_floating_islands(&mut deep, &mut gv3, &mut Rng(42), 520.0);
        assert!(!gv3.island_house_y.is_empty(), "noSurface+extra runs");
        // The deep roll `Next(200, UnderworldLayer - 200)` = Next(200, 800).
        for y in &gv3.island_house_y {
            assert!((200..800).contains(y), "the noSurface y {y} is deep");
        }
    }

    /// The houses pass (`:17859-17874`): a house on every non-lake island,
    /// the sunplate 202 frame + the disc wall 82 interior, the chest cycle
    /// walking 159/65/158/2219 with the `> 3 → Next(4)` re-roll, the
    /// counter advancing once per house.
    #[test]
    fn the_houses_pass_builds_sunplate_and_walks_the_chest_cycle() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        // Five dry islands + one lake, hand-ledgered over built discs.
        let xs = [600i32, 1000, 1400, 1800, 2600, 3000];
        for (idx, x) in xs.iter().enumerate() {
            build_island(&mut w, *x, 200, 50, idx == 5, &mut Rng(5));
            house_ground(&mut w, *x, 205);
            gv.island_house_x.push(*x);
            gv.island_house_y.push(200);
            gv.island_style.push(0);
            gv.sky_lake.push(idx == 5);
        }
        gv.num_island_houses = 6;
        let mut mains: Vec<u16> = Vec::new();
        let mut styles: Vec<i32> = Vec::new();
        {
            let mut hook = |ev: &super::super::chest_stock::ChestSiteEvent,
                            _rng: &mut dyn super::super::chest_stock::WorldGenRand|
             -> Vec<crate::save::ChestItem> {
                mains.push(ev.main_item);
                styles.push(ev.chest_style);
                Vec::new()
            };
            pass_floating_island_houses(
                &mut w,
                &mut gv,
                &mut Rng(11),
                Some(&mut hook),
                false,
            );
        }
        // Five houses (the lake island carries none), the cycle in order;
        // the fifth re-rolls Next(4) — deterministic on the fixed seed.
        assert_eq!(mains.len(), 5, "the lake island has no house");
        assert_eq!(&mains[..4], &[159, 65, 158, 2219]);
        assert!([159, 65, 158, 2219].contains(&mains[4]));
        assert!(styles.iter().all(|s| *s == 13), "the classic style 13");
        assert_eq!(gv.sky_island_house_count, 5, "once per house");
        assert!(count_kind(&w, TILE_SUNPLATE) > 100, "the sunplate shells");
        assert!(count_kind(&w, 9) == 0, "no planks — the old box is gone");
        let disc_walls = w.tiles.iter().filter(|t| t.wall == WALL_DISC).count();
        assert!(disc_walls > 50, "the disc-wall interiors");
        // The lake island's disc carries no sunplate above its centre.
        let lake_house = (2950..=3050)
            .any(|x| (150..=210).any(|y| w.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_SUNPLATE)));
        assert!(!lake_house, "the lake island carries no house");
    }

    /// The getGood chest style (`:79369-79372`): style 2 under
    /// FOR_THE_WORTHY, 13 otherwise.
    #[test]
    fn the_get_good_world_takes_chest_style_2() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        let mut styles: Vec<i32> = Vec::new();
        {
            let mut hook = |ev: &super::super::chest_stock::ChestSiteEvent,
                            _rng: &mut dyn super::super::chest_stock::WorldGenRand|
             -> Vec<crate::save::ChestItem> {
                styles.push(ev.chest_style);
                Vec::new()
            };
            pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), Some(&mut hook), true);
        }
        assert_eq!(styles, vec![2]);
    }

    /// The dungeon-wall abort (`:79207-79216`): a dungeon wall within ±30
    /// of the anchor refuses the whole house.
    #[test]
    fn the_dungeon_wall_aborts_the_house() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        let i = w.index(1020, 210);
        w.tiles[i].wall = 7; // a base dungeon wall inside the ±30 box
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), None, false);
        assert_eq!(count_kind(&w, TILE_SUNPLATE), 0, "the abort builds nothing");
        assert_eq!(gv.sky_island_house_count, 0, "the counter never advances");
    }

    /// The `a == b` determinism witness: the two passes over the same seed
    /// produce byte-identical worlds (the world-hash fold), classic and
    /// extraFloatingIslands alike — the RNG world-pass slots add no
    /// entropy source.
    #[test]
    fn the_island_passes_are_reproducible_for_a_seed() {
        for extra in [false, true] {
            let run = || {
                let mut w = sky_world(4200, 1200, 520);
                if extra {
                    w.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
                }
                let mut gv = gv_with_lakes(4200);
                pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0);
                pass_floating_island_houses(&mut w, &mut gv, &mut Rng(42), None, false);
                super::super::world_hash::world_hash(&w)
            };
            let a = run();
            let b = run();
            assert_eq!(a, b, "extra={extra}: the island passes must be deterministic");
        }
    }
}