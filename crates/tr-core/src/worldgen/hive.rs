//! The faithful hive rewrite (W4-c31): `HiveBiome` + the `Beehives` and
//! `BeeLarvaInBeehives` passes, replacing the shape-level dome in
//! `structures::pass_beehives` (retired, kept for its tests' history).
//!
//! * `HiveBiome.Place` (`HiveBiome.cs:11-102`) — the 100×100 `CanPlace`
//!   gate, the `TooCloseToImportantLocations` 150/10-step scan (`:258-285`),
//!   the 15-disc `IsSolid` → mud/hive 75 % + hive-60 count ≥ 2 scan
//!   (`:24-28`), the `Next(2,5)` tunnel chains (`:29-54`), the
//!   `FrameOutAllHiveContents` 225/86 sweep (`:55`, `:104-125`), the
//!   honey-fall walk with its blocked cubes and dents (`:56-84`),
//!   `CreateStandForLarva` at the chain end (`:85`) and the drunk seed's
//!   1000-try extra stand (`:86-99`), then the 100×100 protected rect
//!   inflated 5 (`:100`).
//! * `CreateHiveTunnel` (`HiveBiome.cs:127-256`) — the radius-12..20 ×
//!   budget-10..20 elliptical walk with the ±20 % radius jitter, the
//!   wall-87 proximity veto (±10 probes), the surface wall-0 veto, the
//!   0.4/0.75/0.6 dithered bands (the honey core / the 225 shell / the 86
//!   wall) and the drifting ±10·0.2 velocity with its ±10·0.05 walk.
//! * `Beehives` (`WorldGen.cs:15927-15976`) — the width-scaled
//!   `1 + Next(5·s, 8·s)` count, the 10000-attempt budget, the
//!   `RandomWorldPoint((surface+rock)>>1, 20, 300, 20)` origins, and each
//!   placed hive's `Next(5)` honey-patch ring (the existing
//!   `mini_biomes::place_honey_patches_around`, `:15954-15972`).
//! * `BeeLarvaInBeehives` (`WorldGen.cs:20868-20900`) — the stand re-clear
//!   and `PlaceTile(num, num2, 231, mute: true)` per `GenVars.larvaX/Y`
//!   slot; the 231 rides `Place3x3` (`:59967-59970` → `:53534-53634`,
//!   the `num = -2` group) with its 3×3 clear + floor gates.
//!
//! Deviations (ANALYSIS "Hive rewrite (W4-c31)"):
//! * The tunnel's `IsSolid` scan is `Modifiers.IsSolid` = active +
//!   `SolidOrSlopedTile` (`Modifiers.cs:561-571`), not the honey patch's
//!   raw `Conditions.IsSolid` — the shared `mini_biomes::solid_or_sloped`
//!   carries it. The `Shapes.Circle(15)` membership is the C# row-walk
//!   (`Shapes.cs:33-49`: `sqrt((r+1)² − dy²)` truncated), not `dx²+dy² ≤
//!   r²` — [`circle_rows`] reproduces it.
//! * `FrameOutAllHiveContents`' `SquareTileFrame`/`SquareWallFrame` ride
//!   the post-pipeline `frame_world`/`frame_walls` seam (the port's
//!   framing model — the same deferral the mahogany/temple passes
//!   document); the sweep itself draws no RNG and writes no state, so it
//!   is a documented no-op here.
//! * `CreateDentForHoneyFall`'s `PoundTile` rides the gated
//!   `edit::pound_tile(generating: true)` (no chest store in worldgen —
//!   `|_, _| false`); the non-generation effects never run.
//! * The `tileSolid[229/232/162]` pass-head pokes (`WorldGen.cs:20872`,
//!   `:20897-20898`) are the port's boot table (the documented
//!   `solid_tile2` deferral); the `progress.Set` calls draw nothing.
//!   The `tileSolid[225] = false` WINDOW is modelled where it matters:
//!   vanilla's weight order runs Hives (weight-slot 50) BEFORE Life
//!   Crystals (60), whose tail flips 225 non-solid (`:16850`) until the
//!   surface-spread pass restores it (`:18426`) — so `HiveBiome.Place`'s
//!   dent walk reads the boot table's SOLID 225 (`Main.cs:10236`), while
//!   the Larva pass (slot 99, after the Piles restore `:19454`) reads
//!   solid 225 again. The port's EXECUTED_PIPELINE runs `beehives` after
//!   its Life-Crystals-era steps, so the window is carried explicitly:
//!   [`place_hive`] takes the boot-table `solid_tile` (the vanilla Hives
//!   slot's state) — no adjustment needed, the C# state at the C# slot
//!   is what the port reproduces.
//! * The drunk-seed branches are ported but dead in the port's classic
//!   fixtures (`world.seeds` carries the flags); the remix arms ride the
//!   same `SeedFlags::REMIX` the other passes read.

use super::mini_biomes::{
    can_place_general, in_world_fluff, place_honey_patches_around, solid_or_sloped, solid_tile,
};
use super::Rng;
use crate::seeds::SeedFlags;
use crate::tile::{LiquidKind, FLAG_ACTIVE, FLAG_HALF_BRICK};
use crate::world::World;

/// `GenVars.larvaX/larvaY` (`GenVars.cs:150-154`, `new int[100]` at
/// `:446-447`) + `numLarva` (reset 0, `WorldGen.cs:11245`). The stand
/// counter wraps at `Length - 1` (`HiveBiome.cs:362-365`) — the last slot
/// is overwritten, never grown.
pub(crate) const LARVA_SLOTS: usize = 100;

/// The larva anchors the `Beehives` pass publishes and the `Larva` pass
/// consumes — the port's `GenVars.numLarva/larvaX/larvaY` slice.
#[derive(Clone)]
pub(crate) struct LarvaAnchors {
    pub(crate) x: [i32; LARVA_SLOTS],
    pub(crate) y: [i32; LARVA_SLOTS],
    pub(crate) num: usize,
}

impl Default for LarvaAnchors {
    fn default() -> Self {
        Self {
            x: [0; LARVA_SLOTS],
            y: [0; LARVA_SLOTS],
            num: 0,
        }
    }
}

/// `Shapes.Circle(radius)`'s row membership (`Shapes.cs:33-49`): row `dy`
/// spans `±min(r, (int)sqrt((r+1)² − dy²))` — the truncated square root,
/// so the disc is one row wider than the naive `dx²+dy² ≤ r²` test.
fn circle_half_width(radius: i32, dy: i32) -> i32 {
    let num = (radius + 1) * (radius + 1);
    let scaled = dy; // horizontal == vertical radius (Circle(r))
    radius.min(((num - scaled * scaled) as f64).sqrt() as i32)
}

/// `HiveBiome.TooCloseToImportantLocations` (`HiveBiome.cs:258-285`): the
/// 150-tile 10-step scan for the larva tile 226 and the walls 83/3/87.
/// (The honey patch's variant adds the underworld band; this one has
/// none — the two C# bodies differ, both ported 1:1.)
fn too_close_to_important_locations(world: &World, ox: i32, oy: i32) -> bool {
    for i in (ox - 150..ox + 150).step_by(10) {
        if i <= 0 || i > world.width - 1 {
            continue;
        }
        for j in (oy - 150..oy + 150).step_by(10) {
            if j > 0 && j <= world.height - 1 {
                let Some(t) = world.tile(i, j) else { continue };
                if t.active() && t.kind == 226 {
                    return true;
                }
                if matches!(t.wall, 83 | 3 | 87) {
                    return true;
                }
            }
        }
    }
    false
}

/// `HiveBiome.CreateStandForLarva` (`HiveBiome.cs:357-383`): publish the
/// clamped anchor (the wrap-at-99 counter), then the 3×4 stand — the
/// three rows above the anchor cleared, the row below set to hive 225
/// with slope/half-brick wiped.
pub(crate) fn create_stand_for_larva(
    world: &mut World,
    anchors: &mut LarvaAnchors,
    px: f64,
    py: f64,
) {
    anchors.x[anchors.num] = (px as i32).clamp(5, world.width - 5);
    anchors.y[anchors.num] = (py as i32).clamp(5, world.height - 5);
    anchors.num += 1;
    if anchors.num >= LARVA_SLOTS {
        anchors.num = LARVA_SLOTS - 1;
    }
    let (num, num2) = (px as i32, py as i32);
    // `:368-370`: the loop guards ride the ITERATION (`i > 0 && i <
    // maxTilesX` in the for-condition — the walk stops at the edge).
    let mut i = num - 1;
    while i <= num + 1 && i > 0 && i < world.width {
        let mut j = num2 - 2;
        while j <= num2 + 1 && j > 0 && j < world.height {
            let idx = world.index(i, j);
            let t = &mut world.tiles[idx];
            if j != num2 + 1 {
                t.flags &= !FLAG_ACTIVE;
            } else {
                t.flags |= FLAG_ACTIVE;
                t.kind = 225;
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
            }
            j += 1;
        }
        i += 1;
    }
}

/// `HiveBiome.CreateHiveTunnel` (`HiveBiome.cs:127-256`): one elliptical
/// tunnel step — returns the drifted centre. `drunk`/`remix` mirror
/// `WorldGen.drunkWorldGen`/`remixWorldGen`; `surface` is the pass's
/// `Layers.world_surface` anchor (`Main.worldSurface` — the port only
/// publishes `world.world_surface` at the world tail, so the anchor
/// rides in as a parameter, the cave-house pass's idiom).
#[allow(clippy::too_many_arguments)]
fn create_hive_tunnel(
    world: &mut World,
    i: i32,
    j: i32,
    rng: &mut Rng,
    drunk: bool,
    remix: bool,
    surface: f64,
) -> (f64, f64) {
    // `:129-142`: the radius/budget rolls and the seed branches.
    let mut num = rng.range(12, 21) as f64;
    let mut num2 = rng.range(10, 21) as f64;
    if drunk {
        num = rng.range(8, 26) as f64;
        num2 = rng.range(10, 41) as f64;
        let num3 = (world.width as f64 / 4200.0 + 1.0) / 2.0;
        num *= num3;
        num2 *= num3;
    } else if remix {
        num += rng.range(0, 3) as f64;
    }
    let num4 = num;
    let (mut rx, mut ry) = (i as f64, j as f64);
    // `:147-149`: the drift velocity, ±10·0.2 per axis.
    let (mut vx, mut vy) = (
        rng.range(-10, 11) as f64 * 0.2,
        rng.range(-10, 11) as f64 * 0.2,
    );
    while num > 0.0 && num2 > 0.0 {
        // `:152-155`: the deep-world stop.
        if ry > (world.height - 250) as f64 {
            num2 = 0.0;
        }
        // `:156-157`: the ±20 % radius jitter and the first budget burn.
        num = num4 * (1.0 + rng.range(-20, 20) as f64 * 0.01);
        num2 -= 1.0;
        // `:158-177`: the clamped square bounds (C# truncation casts).
        let num5 = ((rx - num) as i32).max(1);
        let num6 = ((rx + num) as i32).min(world.width - 1);
        let num7 = ((ry - num) as i32).max(1);
        let num8 = ((ry + num) as i32).min(world.height - 1);
        for k in num5..num6 {
            for l in num7..num8 {
                // `:182-204`: the InWorld(50) gate and the four ±10
                // wall-87 probes — any hit zeroes the budget (the walk
                // keeps writing this row, then stops).
                if !in_world_fluff(world, k, l, 50) {
                    num2 = 0.0;
                } else {
                    for (px, py) in [(k - 10, l), (k + 10, l), (k, l - 10), (k, l + 10)] {
                        if world.tile(px, py).is_some_and(|t| t.wall == 87) {
                            num2 = 0.0;
                        }
                    }
                }
                // `:205-208`: above the surface, a wall-less cell five up
                // vetoes (the hive must stay underground).
                if (l as f64) < surface
                    && world.tile(k, l - 5).is_some_and(|t| t.wall == 0)
                {
                    num2 = 0.0;
                }
                let num9 = (k as f64 - rx).abs();
                let num10 = (l as f64 - ry).abs();
                let num11 = (num9 * num9 + num10 * num10).sqrt();
                // `:212-227`: the 0.4 honey core — the 1-in-3 honey fill
                // (drunk always), the 86 wall, the inactive clear.
                if num11 < num4 * 0.4 * (1.0 + rng.range(-10, 11) as f64 * 0.005) {
                    let idx = world.index(k, l);
                    let t = &mut world.tiles[idx];
                    if rng.range(0, 3) == 0 {
                        t.liquid = u8::MAX;
                    }
                    if drunk {
                        t.liquid = u8::MAX;
                    }
                    t.set_liquid_kind(LiquidKind::Honey);
                    t.wall = 86;
                    t.flags &= !FLAG_ACTIVE;
                    t.flags &= !FLAG_HALF_BRICK;
                    t.set_slope(0);
                } else if num11 < num4 * 0.75 * (1.0 + rng.range(-10, 11) as f64 * 0.005) {
                    // `:228-238`: the 0.75 shell — the 225 block unless
                    // the cell already carries the hive/shimmer wall.
                    let idx = world.index(k, l);
                    let t = &mut world.tiles[idx];
                    t.liquid = 0;
                    if t.wall != 86 && t.wall != 244 {
                        t.flags |= FLAG_ACTIVE;
                        t.flags &= !FLAG_HALF_BRICK;
                        t.set_slope(0);
                        t.kind = 225;
                    }
                }
                // `:239-247`: the 0.6 wall band (NOT else-if — it runs
                // over both arms above), the drunk 1-in-2 honey fill.
                if num11 < num4 * 0.6 * (1.0 + rng.range(-10, 11) as f64 * 0.005) {
                    let idx = world.index(k, l);
                    let t = &mut world.tiles[idx];
                    t.wall = 86;
                    if drunk && rng.range(0, 2) == 0 {
                        t.liquid = u8::MAX;
                        t.set_liquid_kind(LiquidKind::Honey);
                    }
                }
            }
        }
        // `:250-253`: the drift, the second budget burn, the velocity walk.
        rx += vx;
        ry += vy;
        num2 -= 1.0;
        vy += rng.range(-10, 11) as f64 * 0.05;
        vx += rng.range(-10, 11) as f64 * 0.05;
    }
    (rx, ry)
}

/// `HiveBiome.BadSpotForHoneyFall` (`HiveBiome.cs:348-355`): the 2×2
/// probe — three active corners with the fourth open is the good spot.
fn bad_spot_for_honey_fall(world: &World, x: i32, y: i32) -> bool {
    let a = |px: i32, py: i32| world.tile(px, py).is_some_and(|t| t.active());
    if a(x, y) && a(x, y + 1) && a(x + 1, y) {
        return !a(x + 1, y + 1);
    }
    true
}

/// `HiveBiome.SpotActuallyNotInHive` (`HiveBiome.cs:329-346`): the 4×4
/// probe — an edge cell or any active non-225 tile means "not in hive".
fn spot_actually_not_in_hive(world: &World, x: i32, y: i32) -> bool {
    for i in x - 1..=x + 2 {
        for j in y - 1..=y + 2 {
            if i < 10 || i > world.width - 10 {
                return true;
            }
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind != 225) {
                return true;
            }
        }
    }
    false
}

/// `HiveBiome.CreateBlockedHoneyCube` (`HiveBiome.cs:308-327`): the 4×4
/// cube — the inner 2×2 honey pocket, the 12-cell 225 rim.
fn create_blocked_honey_cube(world: &mut World, x: i32, y: i32) {
    for i in x - 1..=x + 2 {
        for j in y - 1..=y + 2 {
            let Some(_) = world.tile(i, j) else { continue };
            let idx = world.index(i, j);
            let t = &mut world.tiles[idx];
            if i >= x && i <= x + 1 && j >= y && j <= y + 1 {
                t.flags &= !FLAG_ACTIVE;
                t.liquid = u8::MAX;
                t.set_liquid_kind(LiquidKind::Honey);
            } else {
                t.flags |= FLAG_ACTIVE;
                t.kind = 225;
            }
        }
    }
}

/// `HiveBiome.CreateDentForHoneyFall` (`HiveBiome.cs:287-306`): the
/// reversed-direction walk pounding the floor into half-bricks and
/// patching the open cell below each with 225. The walk runs while
/// `num < 4 || SolidTile` and stays inside the ±10 border.
fn create_dent_for_honey_fall(world: &mut World, x: i32, y: i32, dir: i32) {
    let dir = -dir;
    let y = y + 1;
    let mut x = x;
    let mut num = 0;
    while (num < 4 || solid_tile(world, x, y)) && x > 10 && x < world.width - 10 {
        num += 1;
        x += dir;
        if solid_tile(world, x, y) {
            // `:298` — PoundTile, generation-time (the gated i31 port;
            // worldgen has no chest store, the closure answers false).
            crate::edit::pound_tile(world, x, y, true, |_, _| false);
            if world.tile(x, y + 1).is_some_and(|t| !t.active()) {
                let idx = world.index(x, y + 1);
                world.tiles[idx].flags |= FLAG_ACTIVE;
                world.tiles[idx].kind = 225;
            }
        }
    }
}

/// `HiveBiome.Place` (`HiveBiome.cs:11-102`). `protected` is the port's
/// structure-map stand-in (the shared mini-biomes rect list); the larva
/// anchors ride [`LarvaAnchors`] (the `GenVars` slice).
pub(crate) fn place_hive(
    world: &mut World,
    ox: i32,
    oy: i32,
    protected: &mut Vec<(i32, i32, i32, i32)>,
    anchors: &mut LarvaAnchors,
    rng: &mut Rng,
    surface: f64,
) -> bool {
    // `:13-16`: the 100×100 CanPlace gate (padding 0).
    if !can_place_general(world, (ox - 50, oy - 50, 100, 100), protected) {
        return false;
    }
    // `:17-20`: the important-location scan.
    if too_close_to_important_locations(world, ox, oy) {
        return false;
    }
    // `:21-28`: the 15-disc scan — IsSolid (active + SolidOrSlopedTile),
    // the mud/hive (59/60) count and the hive (60) count; the gates are
    // 75 % and 2+. `Shapes.Circle(15)`'s row membership, not dx²+dy².
    let (mut solid, mut mud_hive, mut hive) = (0i32, 0i32, 0i32);
    for dy in -15..=15 {
        let hw = circle_half_width(15, dy);
        for dx in -hw..=hw {
            let (x, y) = (ox + dx, oy + dy);
            if !solid_or_sloped(world, x, y) {
                continue;
            }
            solid += 1;
            match world.tile(x, y).map(|t| t.kind) {
                Some(59) | Some(60) => mud_hive += 1,
                _ => {}
            }
            if world.tile(x, y).map(|t| t.kind) == Some(60) {
                hive += 1;
            }
        }
    }
    if solid == 0 || (mud_hive as f64) / (solid as f64) < 0.75 || hive < 2 {
        return false;
    }
    let drunk = world.seeds.contains(SeedFlags::DRUNK);
    let remix = world.seeds.contains(SeedFlags::REMIX);
    // `:29-54`: the tunnel chains — Next(2,5) chains (drunk +Next(7,10),
    // remix +Next(2,5)), each Next(2,5) linked tunnel steps.
    let mut num2 = rng.range(2, 5);
    if drunk {
        num2 += rng.range(7, 10);
    } else if remix {
        num2 += rng.range(2, 5);
    }
    let (mut cx, mut cy) = (ox as f64, oy as f64);
    let mut ends: Vec<(i32, i32)> = Vec::new();
    for _ in 0..num2 {
        let num3 = rng.range(2, 5);
        for _ in 0..num3 {
            (cx, cy) = create_hive_tunnel(world, cx as i32, cy as i32, rng, drunk, remix, surface);
        }
        ends.push((cx as i32, cy as i32));
    }
    // `:55`: FrameOutAllHiveContents — the 225/86 framing sweep; the
    // post-pipeline frame_world/frame_walls seam carries it (module doc).
    // `:56-84`: the honey falls — one per chain end.
    for (end_x, end_y) in ends {
        let mut num4 = end_x;
        let y = end_y;
        let num5 = if rng.range(0, 2) == 0 { -1 } else { 1 };
        let mut flag = false;
        while in_world_fluff(world, num4, y, 10) && bad_spot_for_honey_fall(world, num4, y) {
            num4 += num5;
            if (num4 - end_x).abs() > 50 {
                flag = true;
                break;
            }
        }
        if !flag {
            num4 += num5;
            if !spot_actually_not_in_hive(world, num4, y) {
                create_blocked_honey_cube(world, num4, y);
                create_dent_for_honey_fall(world, num4, y, num5);
            }
        }
    }
    // `:85`: the stand at the LAST chain's end (the C# `vector2D`).
    create_stand_for_larva(world, anchors, cx, cy);
    // `:86-99`: the drunk seed's extra stand — 1000 tries, >10 away,
    // inactive with the hive wall 86.
    if drunk {
        for _ in 0..1000 {
            let x3 = cx + rng.range(-50, 51) as f64;
            let y3 = cy + rng.range(-50, 51) as f64;
            let (ix, iy) = (x3 as i32, y3 as i32);
            let dist = ((cx - x3) * (cx - x3) + (cy - y3) * (cy - y3)).sqrt();
            if in_world_fluff(world, ix, iy, 0)
                && dist > 10.0
                && world
                    .tile(ix, iy)
                    .is_some_and(|t| !t.active() && t.wall == 86)
            {
                create_stand_for_larva(world, anchors, x3, y3);
                break;
            }
        }
    }
    // `:100`: AddProtectedStructure(rect, 5) — the inflate-5 rect.
    protected.push((ox - 55, oy - 55, 110, 110));
    true
}

/// The `Beehives` pass (`WorldGen.cs:15927-15976`): the width-scaled
/// count, the 10000-attempt budget, the RandomWorldPoint origins and the
/// per-hive honey-patch ring. Returns the placed-hive count.
pub(crate) fn pass_beehives(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    anchors: &mut LarvaAnchors,
    rng: &mut Rng,
) -> u32 {
    // `:15932-15937`: 1 + Next(5·s, 8·s), drunk ×0.667.
    let s = world.width as f64 / 4200.0;
    let drunk = world.seeds.contains(SeedFlags::DRUNK);
    let mut num2 = 1.0 + rng.range((5.0 * s) as i32, (8.0 * s) as i32) as f64;
    if drunk {
        num2 *= 0.667;
    }
    let mut num3 = 10000;
    let mut protected: Vec<(i32, i32, i32, i32)> = Vec::new();
    let mut placed = 0u32;
    // `:15942-15974`: the attempt loop.
    while num2 > 0.0 && num3 > 0 {
        num3 -= 1;
        // `:15946`: RandomWorldPoint((surface+rock)>>1, 20, 300, 20) —
        // Next(left, maxX-right) × Next(top, maxY-bottom).
        let top = (world_surface + rock_layer) as i32 >> 1;
        // The sub-vanilla guard: the C#'s Next(top, maxY-300) throws on a
        // collapsed band; the port's small test worlds skip the attempt
        // (the same idiom the mahogany/thin-ice passes carry).
        if world.width <= 40 || world.height - 300 <= top {
            break;
        }
        let ox = rng.range(20, world.width - 20);
        let oy = rng.range(top, world.height - 300);
        // `:15947-15950`: the drunk seed draws a SECOND point and
        // discards it (the C# calls RandomWorldPoint without using the
        // result — the stream burn is real).
        if drunk {
            let _ = rng.range(20, world.width - 20);
            let _ = rng.range(world_surface as i32, world.height - 300);
        }
        if place_hive(world, ox, oy, &mut protected, anchors, rng, world_surface) {
            num2 -= 1.0;
            placed += 1;
            // `:15954-15972`: the Next(5) honey-patch ring (the existing
            // HoneyPatchBiome port; the extraLiquid seed zeroes it).
            place_honey_patches_around(world, ox, oy, &mut protected, rng);
        }
    }
    placed
}

/// `WorldGen.Place3x3`'s 231 arm (`WorldGen.cs:53534-53634`, the
/// `num = -2` group at `:53538-53567`): the 3×3 clear above the floor
/// row, the `SolidTile2` floor gate, then the nine style-0 frames.
/// `PlaceTile(x, y, 231, mute: true)` dispatches here (`:59967-59970`);
/// the outer PlaceTile gate passes on `!tileSolid[231]` (`:59373` — the
/// larva is not a solid kind), and at the Larva slot the boot table's
/// solid 225 floor stands (the Piles pass restored it, `:19454`).
fn place_larva_3x3(world: &mut World, x: i32, y: i32) -> bool {
    // `:53541-53554`: the 3×3 body must be clear (x-1..x+1 × y-2..y).
    for i in x - 1..x + 2 {
        for j in y - 2..y + 1 {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                return false;
            }
        }
    }
    // `:53555-53567`: the floor row must be SolidTile2 (the boot-table
    // fold — `edit::solid_tile2`, the documented 229/232/162 deferral).
    for k in x - 1..x + 2 {
        if !crate::edit::solid_tile2(world, k, y + 1) {
            return false;
        }
    }
    // `:53594-53633`: the nine writes — style 0 (`num2 = 0`), the
    // `num = -2` offset: rows y-2/y-1/y carry frameY 0/18/36.
    for (row, fy) in [(y - 2, 0i16), (y - 1, 18), (y, 36)] {
        for (col, fx) in [(x - 1, 0i16), (x, 18), (x + 1, 36)] {
            let Some(_) = world.tile(col, row) else { continue };
            let idx = world.index(col, row);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.frame_y = fy;
            t.frame_x = fx;
            t.kind = 231;
        }
    }
    true
}

/// The `BeeLarvaInBeehives` pass (`WorldGen.cs:20868-20900`): per larva
/// anchor, the stand re-clear (the same 3×4 shape `CreateStandForLarva`
/// wrote) and the 231 placement. The `tileSolid[229/232/162]` pokes are
/// the boot-table deferral (module doc); `progress.Set` draws nothing.
pub(crate) fn pass_larva(world: &mut World, anchors: &LarvaAnchors) {
    for i in 0..anchors.num {
        let (num, num2) = (anchors.x[i], anchors.y[i]);
        // `:20878-20894`: the stand re-clear — NO iteration guards here
        // (the C# loop is unguarded; the anchors are clamped 5..max-5).
        for j in num - 1..=num + 1 {
            for k in num2 - 2..=num2 + 1 {
                let Some(_) = world.tile(j, k) else { continue };
                let idx = world.index(j, k);
                let t = &mut world.tiles[idx];
                if k != num2 + 1 {
                    t.flags &= !FLAG_ACTIVE;
                } else {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 225;
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                }
            }
        }
        // `:20895`: PlaceTile(num, num2, 231, mute: true) → Place3x3.
        place_larva_3x3(world, num, num2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    /// A solid-mud world with a jungle-grass (60) core — the disc the
    /// Place gate's 75 %/2+ scan passes on. `surface` sits above the
    /// hive band so the tunnel's wall-0 veto never fires (the cells
    /// carry the natural mud wall 15 below it — the fixture sets 86
    /// nowhere, the tunnels write it).
    fn mud_world(width: i32, height: i32) -> World {
        let mut w = World::new("hive".into(), width, height);
        for x in 0..width {
            for y in 0..height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 59,
                    wall: 15,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    #[test]
    fn the_circle_membership_is_the_csharp_row_walk() {
        // `Shapes.Circle(15)` (`Shapes.cs:33-49`): row dy spans
        // ±min(15, (int)sqrt(16² − dy²)) — the dy=0 row is the FULL
        // ±15 (the naive dx²+dy² ≤ 225 test agrees), but dy=±15 spans
        // ±(int)sqrt(256−225) = ±5, NOT 0 — the C# disc is wider at the
        // poles than the Euclidean disc.
        assert_eq!(circle_half_width(15, 0), 15);
        assert_eq!(circle_half_width(15, 15), 5);
        assert_eq!(circle_half_width(15, -15), 5);
        // dy=±8: sqrt(256−64) = sqrt(192) ≈ 13.85 → 13 (the Euclidean
        // test would give sqrt(225−64) ≈ 12.68 → 12).
        assert_eq!(circle_half_width(15, 8), 13);
    }

    #[test]
    fn the_stand_publishes_the_clamped_anchor_and_clears_the_3x4() {
        // `CreateStandForLarva` (`HiveBiome.cs:357-383`): the anchor
        // clamps 5..max-5, the counter advances, the three rows above
        // clear and the row below sets 225 slopeless.
        let mut w = mud_world(60, 60);
        let mut anchors = LarvaAnchors::default();
        create_stand_for_larva(&mut w, &mut anchors, 30.7, 20.2);
        assert_eq!((anchors.x[0], anchors.y[0], anchors.num), (30, 20, 1));
        for dx in -1..=1 {
            for dy in -2..=0 {
                assert!(
                    !w.tile(30 + dx, 20 + dy).unwrap().active(),
                    "the stand clears ({dx}, {dy})"
                );
            }
            let floor = w.tile(30 + dx, 21).unwrap();
            assert!(floor.active() && floor.kind == 225, "the 225 floor row");
            assert_eq!(floor.slope(), 0);
            assert!(!floor.half_brick());
        }
        // The clamp: an anchor near the edge lands at 5 / max-5.
        create_stand_for_larva(&mut w, &mut anchors, 1.0, 59.0);
        assert_eq!((anchors.x[1], anchors.y[1]), (5, 55));
    }

    #[test]
    fn the_larva_counter_wraps_at_the_last_slot() {
        // `:362-365`: numLarva >= 100 → 99 — the last slot overwrites.
        let mut w = mud_world(60, 60);
        let mut anchors = LarvaAnchors::default();
        for k in 0..105 {
            create_stand_for_larva(&mut w, &mut anchors, 30.0, 20.0 + (k % 5) as f64);
        }
        assert_eq!(anchors.num, 99);
        // The 105th stand (k=104, y=24) overwrote slot 99.
        assert_eq!(anchors.y[99], 24);
    }

    #[test]
    fn the_place_gate_refuses_the_sparse_disc_and_the_larva_tile() {
        // `:21-28`: an empty world fails the IsSolid scan (0 solids);
        // `:17-20`: a 226 tile inside the 150/10 scan vetoes.
        let mut w = World::new("hive2".into(), 400, 600);
        let mut protected = Vec::new();
        let mut anchors = LarvaAnchors::default();
        let mut rng = Rng::from_seed(5);
        assert!(!place_hive(&mut w, 200, 300, &mut protected, &mut anchors, &mut rng, 100.0));
        // The mud disc passes the scan but a 226 at a scanned cell vetoes.
        let mut w2 = mud_world(400, 600);
        let i = w2.index(200, 300);
        w2.tiles[i].kind = 226;
        assert!(!place_hive(&mut w2, 200, 300, &mut protected, &mut anchors, &mut rng, 100.0));
        assert_eq!(anchors.num, 0, "no stand published on a refusal");
    }

    #[test]
    fn the_place_gate_refuses_the_non_jungle_disc() {
        // The 75 % mud/hive ratio: a stone disc with a 2-cell 60 core
        // fails the ratio even though the hive count clears 2.
        let mut w = mud_world(400, 600);
        for x in 0..400 {
            for y in 0..600 {
                let i = w.index(x, y);
                w.tiles[i].kind = 1; // stone
            }
        }
        for (x, y) in [(200, 300), (201, 300)] {
            let i = w.index(x, y);
            w.tiles[i].kind = 60;
        }
        let mut protected = Vec::new();
        let mut anchors = LarvaAnchors::default();
        let mut rng = Rng::from_seed(5);
        assert!(!place_hive(&mut w, 200, 300, &mut protected, &mut anchors, &mut rng, 100.0));
    }

    #[test]
    fn the_hive_carves_the_tunnels_and_publishes_the_stand() {
        // A full mud disc with a 60 core: the Place gates pass, the
        // tunnel chains carve the 86-walled honey pockets and the 225
        // shells, and the stand publishes one anchor per chain.
        let mut w = mud_world(400, 600);
        for x in 150..250 {
            for y in 250..350 {
                let i = w.index(x, y);
                w.tiles[i].kind = 60;
            }
        }
        let mut protected = Vec::new();
        let mut anchors = LarvaAnchors::default();
        let mut rng = Rng::from_seed(11);
        assert!(place_hive(&mut w, 200, 300, &mut protected, &mut anchors, &mut rng, 100.0));
        // The 110×110 inflate-5 rect registered (`:100`).
        assert_eq!(protected, vec![(145, 245, 110, 110)]);
        // 2-4 chains, one stand each (`:29-54`, `:85` publishes the LAST
        // chain's end only — the C# CreateStandForLarva runs once).
        assert_eq!(anchors.num, 1, "one stand per hive (the last chain end)");
        // The tunnels wrote the hive wall 86 and the honey liquid.
        let wall86 = w.tiles.iter().filter(|t| t.wall == 86).count();
        assert!(wall86 > 50, "the tunnel walls: {wall86}");
        let honey = w
            .tiles
            .iter()
            .filter(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Honey)
            .count();
        assert!(honey > 20, "the honey pockets: {honey}");
        let hive_blocks = w.tiles.iter().filter(|t| t.active() && t.kind == 225).count();
        assert!(hive_blocks > 50, "the 225 shells: {hive_blocks}");
        // The stand ran at the last chain's end (the anchor publishes;
        // the 3×4 floor shape is pinned by the dedicated stand witness —
        // the chain end can drift, so the anchor may clamp away from the
        // written cells).
        assert!(anchors.x[0] >= 5 && anchors.x[0] <= w.width - 5);
        assert!(anchors.y[0] >= 5 && anchors.y[0] <= w.height - 5);
    }

    #[test]
    fn the_honey_cube_and_the_dent_write_their_shapes() {
        // `CreateBlockedHoneyCube` (`:308-327`): the inner 2×2 honey,
        // the 12-cell 225 rim. `CreateDentForHoneyFall` (`:287-306`):
        // the reversed walk pounds the solid floor into half-bricks.
        let mut w = mud_world(80, 80);
        create_blocked_honey_cube(&mut w, 40, 40);
        for i in 40..=41 {
            for j in 40..=41 {
                let t = w.tile(i, j).unwrap();
                assert!(!t.active() && t.liquid == u8::MAX);
                assert_eq!(t.liquid_kind(), LiquidKind::Honey);
            }
        }
        for i in 39..=42 {
            for j in 39..=42 {
                if (40..=41).contains(&i) && (40..=41).contains(&j) {
                    continue;
                }
                let t = w.tile(i, j).unwrap();
                assert!(t.active() && t.kind == 225, "the rim ({i}, {j})");
            }
        }
        // The dent: dir=1 → the walk goes LEFT (dir *= -1), y+1. The
        // floor is solid mud, so the first 4+ cells pound half-brick.
        create_dent_for_honey_fall(&mut w, 40, 40, 1);
        let pounded = (30..40)
            .filter(|&x| w.tile(x, 41).unwrap().half_brick())
            .count();
        assert!(pounded >= 4, "the dent pounds the floor: {pounded}");
    }

    #[test]
    fn the_larva_pass_places_the_3x3_over_the_stand() {
        // `BeeLarvaInBeehives` (`:20868-20900`): the stand re-clear +
        // the 231 Place3x3 — nine cells, style-0 frames, the num=-2
        // offset (rows y-2..y carry frameY 0/18/36).
        let mut w = mud_world(80, 80);
        let mut anchors = LarvaAnchors::default();
        create_stand_for_larva(&mut w, &mut anchors, 40.0, 40.0);
        pass_larva(&mut w, &anchors);
        for (row, fy) in [(38, 0i16), (39, 18), (40, 36)] {
            for (col, fx) in [(39, 0i16), (40, 18), (41, 36)] {
                let t = w.tile(col, row).unwrap();
                assert!(t.active() && t.kind == 231, "the larva cell ({col}, {row})");
                assert_eq!((t.frame_x, t.frame_y), (fx, fy), "the style-0 frames");
            }
        }
        // The floor row stays 225 (the Place3x3 gate read it).
        for col in 39..=41 {
            let t = w.tile(col, 41).unwrap();
            assert!(t.active() && t.kind == 225);
        }
    }

    #[test]
    fn the_larva_placement_refuses_a_blocked_body() {
        // `Place3x3`'s clear gate (`:53541-53554`): an active cell in
        // the 3×3 body refuses the placement. (The pass's stand re-clear
        // runs first and would wipe a foreign body cell, so the gate is
        // pinned on the placement fn directly.)
        let mut w = mud_world(80, 80);
        // The floor row: 225 at y=41 under the body.
        for col in 39..=41 {
            let i = w.index(col, 41);
            w.tiles[i].kind = 225;
        }
        // A foreign active cell inside the body (x-1..x+1 × y-2..y).
        let i = w.index(40, 39);
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].kind = 1;
        assert!(!place_larva_3x3(&mut w, 40, 40));
        assert!(
            !w.tiles.iter().any(|t| t.active() && t.kind == 231),
            "the blocked body refuses the 231"
        );
        // The floor gate: a non-solid floor row refuses too.
        let mut w2 = mud_world(80, 80);
        for col in 39..=41 {
            let i = w2.index(col, 41);
            w2.tiles[i].flags &= !FLAG_ACTIVE; // the floor is air
        }
        // The body above is solid mud — clear it first (the gate order:
        // body clear, then floor).
        for row in 38..=40 {
            for col in 39..=41 {
                let i = w2.index(col, row);
                w2.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        assert!(!place_larva_3x3(&mut w2, 40, 40));
    }

    #[test]
    fn the_beehives_pass_is_reproducible_for_a_seed() {
        // The a == b determinism witness: two runs of the faithful pass
        // over identical worlds draw identical streams and write
        // identical tiles (the full Tile, all fields).
        let run = || {
            let mut w = mud_world(400, 600);
            for x in 100..300 {
                for y in 200..400 {
                    let i = w.index(x, y);
                    w.tiles[i].kind = 60;
                }
            }
            let mut anchors = LarvaAnchors::default();
            let mut rng = Rng::from_seed(42);
            let placed = pass_beehives(&mut w, 100.0, 180.0, &mut anchors, &mut rng);
            pass_larva(&mut w, &anchors);
            (w, anchors, placed)
        };
        let (a, aa, pa) = run();
        let (b, ab, pb) = run();
        assert_eq!(pa, pb);
        assert_eq!(aa.num, ab.num);
        assert_eq!(aa.x, ab.x);
        assert_eq!(aa.y, ab.y);
        assert_eq!(a.tiles, b.tiles, "a == b — the full tile state");
    }
}