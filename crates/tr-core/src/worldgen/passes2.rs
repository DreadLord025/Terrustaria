//! W4-d102 — the second slice of unexecuted worldgen passes, in C# anchor
//! order (the lane's range declaration: `docs/research/12a-d102-passes-2.md`):
//! Silt (`WorldGen.cs:13143`), Webs (`:13607`), Slush (`:14697`), Mountain
//! Caves (`:14873`), Gems (`:15050`), Clean Up Dirt (`:15246`), Dirt Rock
//! Wall Runner (`:15461`) and Wood Tree Walls (`:15717`). The Shimmer pass
//! (`:15195`) lives in its own module (`worldgen/shimmer.rs`).
//!
//! Every pass draws from its own `pass_rng("<vanilla name>")` stream (the
//! port's per-pass re-seed model, `pipeline.rs`'s `Streams::pass_rng`), so
//! inserting a step shifts no existing step's draw ledger. The secret-seed
//! arms are ported where they are pure multipliers/band swaps behind
//! `World::seeds` / `World::micro_seeds`; a classic world reads no flag.
//! The `!Skyblock.denyAllGeneration` outer gate (`WorldGen.cs:3131` —
//! `skyblockWorldGen`) reads the port's `SeedFlags::SKYBLOCK` (the
//! `islands.rs` precedent): a skyblock world skips the pass body whole.

use super::noise::Rng;
use super::tile_runner::{tile_runner, TileRunnerOpts};
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::FLAG_ACTIVE;
use crate::tiledata::SET_CAN_BE_CLEARED_DURING_GENERATION;
use crate::world::World;

/// `TileID.SiltBlock` (`TileID.cs`) — the Silt pass's runner kind.
const TILE_SILT: i32 = 123;
/// `WallID.Sandstone` = 187 / `WallID.HardenedSand` = 216 — the desert walls
/// the Silt candidates skip (`WorldGen.cs:13159`, `:13174`).
const WALL_SANDSTONE: u16 = 187;
const WALL_HARDENED_SAND: u16 = 216;

/// The Silt pass (`WorldGen.cs:13143-13181`): two TileRunner loops over the
/// rock layer, kind 123. The first runs `(int)((float)area * 0.0001f)` big
/// blobs (`Next(5, 12)` strength, `Next(15, 50)` steps), the second
/// `(int)((float)area * 0.0005f)` small ones (`Next(2, 5)` × `Next(2, 5)`).
/// Each candidate draws x then y; the remix band swap draws y AGAIN from
/// `worldSurface..rockLayer` (the C# overwrites `num3`/`num5` after the
/// first draw — both draws ride the stream). Candidates on the desert
/// walls 187/216 are skipped without the strength/steps draws.
pub(crate) fn pass_silt(
    world: &mut World,
    rock_layer_high: f64,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
) {
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let opts = TileRunnerOpts::for_world(world);
    // `:13149` — the float multiply is the C#'s `(float)` cast, kept in f32.
    let area = (world.width * world.height) as f32;
    let num = (area * 0.0001f32) as i32;
    for _ in 0..num {
        let x = rng.range(0, world.width);
        let mut y = rng.range(rock_layer_high as i32, world.height);
        if remix {
            y = rng.range(world_surface as i32, rock_layer as i32);
        }
        let wall = world.tile(x, y).map_or(0, |t| t.wall);
        if wall != WALL_SANDSTONE && wall != WALL_HARDENED_SAND {
            let strength = rng.range(5, 12);
            let steps = rng.range(15, 50);
            tile_runner(
                world,
                rng,
                x,
                y,
                strength as f64,
                steps,
                TILE_SILT,
                world_surface,
                &opts,
            );
        }
    }
    // `:13164` — the small-blob loop.
    let num = (area * 0.0005f32) as i32;
    for _ in 0..num {
        let x = rng.range(0, world.width);
        let mut y = rng.range(rock_layer_high as i32, world.height);
        if remix {
            y = rng.range(world_surface as i32, rock_layer as i32);
        }
        let wall = world.tile(x, y).map_or(0, |t| t.wall);
        if wall != WALL_SANDSTONE && wall != WALL_HARDENED_SAND {
            let strength = rng.range(2, 5);
            let steps = rng.range(2, 5);
            tile_runner(
                world,
                rng,
                x,
                y,
                strength as f64,
                steps,
                TILE_SILT,
                world_surface,
                &opts,
            );
        }
    }
}

/// `TileID.Cobweb` = 51 — the Webs pass's runner kind.
const TILE_COBWEB: i32 = 51;

/// The walls the port's terrain pre-lays under every solid row
/// (`terrain.rs:343-351`) — the deviation counterpart of the C#'s `wall > 0`
/// (the `spider_gem_moss.rs::BASELINE_WALLS` precedent): vanilla leaves the
/// underground wall 0 until `Cave Walls` runs, so its surface-gate
/// `wall > 0` only counts walls a pass painted; the port reads its baseline
/// walls as unwalled.
const BASELINE_WALLS: [u16; 4] = [1, 2, 187, 216];

/// `tile.wall > 0` with the port's baseline-wall adaptation (see
/// [`BASELINE_WALLS`]).
fn walled(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.wall > 0 && !BASELINE_WALLS.contains(&t.wall))
}

/// The Webs pass (`WorldGen.cs:13607-13644`): `(int)(area * 0.0006)`
/// ceiling-hugging cobweb runners. Each candidate draws x (`Next(20,
/// maxTilesX - 20)`) then y (`Next(worldSurfaceHigh, maxTilesY - 20)`);
/// the first `GenVars.numMCaves` iterations take the mountain-cave anchors
/// instead (the draws still happen — the C# overwrites after drawing).
/// An inactive candidate above the surface line needs a real wall; the
/// walker climbs to the first active tile above `worldSurfaceLow`, steps
/// back down one, rolls the horizontal direction (`Next(2)`), slides along
/// the ceiling to the first active tile (or the ±10 world margin), steps
/// back one, and runs `TileRunner(j, num, Next(4, 11), Next(2, 4), 51,
/// addTile: true, speedX: num2, speedY: -1)` when the landing cell is
/// below the surface or walled.
pub(crate) fn pass_webs(
    world: &mut World,
    world_surface_high: f64,
    world_surface_low: f64,
    world_surface: f64,
    num_m_caves: i32,
    m_cave_x: &[i32; 30],
    m_cave_y: &[i32; 30],
    rng: &mut Rng,
) {
    let opts = TileRunnerOpts {
        add_tile: true,
        speed_y: -1.0,
        over_ride: false,
        ..TileRunnerOpts::for_world(world)
    };
    // `:13612` — the double multiply, truncated.
    let count = ((world.width * world.height) as f64 * 0.0006) as i32;
    for i in 0..count {
        let mut j = rng.range(20, world.width - 20);
        let mut num = rng.range(world_surface_high as i32, world.height - 20);
        if i < num_m_caves {
            j = m_cave_x[i as usize];
            num = m_cave_y[i as usize];
        }
        let inactive = !world.tile(j, num).is_some_and(|t| t.active());
        // `:13621` — only an INACTIVE candidate enters, and above the
        // surface line it needs a real wall.
        if inactive && ((num as f64) > world_surface || walled(world, j, num)) {
            // The climb (`:13623-13626`): up while inactive, stopping at
            // the worldSurfaceLow floor; then one step back down.
            while !world.tile(j, num).is_some_and(|t| t.active())
                && num > world_surface_low as i32
            {
                num -= 1;
            }
            num += 1;
            let mut num2 = 1;
            if rng.range(0, 2) == 0 {
                num2 = -1;
            }
            // The ceiling slide (`:13633-13636`).
            while !world.tile(j, num).is_some_and(|t| t.active())
                && j > 10
                && j < world.width - 10
            {
                j += num2;
            }
            j -= num2;
            if (num as f64) > world_surface || walled(world, j, num) {
                let strength = rng.range(4, 11);
                let steps = rng.range(2, 4);
                let mut opts = opts;
                opts.speed_x = num2 as f64;
                tile_runner(
                    world,
                    rng,
                    j,
                    num,
                    strength as f64,
                    steps,
                    TILE_COBWEB,
                    world_surface,
                    &opts,
                );
            }
        }
    }
}

/// The Slush pass (`StoneToIceAndSiltPlusMudIntoSlush`,
/// `WorldGen.cs:14697-14752`): the snow-rect retype sweep — silt 123 →
/// slush 224, mud 59 → slush 224 gated on the 7×7 jungle-grass scan (the
/// C#'s inner `break` leaves only the inner loop — the scan keeps walking
/// the outer columns, and a later hit still clears the flag), stone 1 →
/// ice 161. The `surfaceIsDesert && noSurface` micro-seed pair retypes to
/// sand 53 instead of slush (`:14708-14715`, `:14734-14741`). The C#
/// switches on `tile.type` without an active gate for 123/1 — kept
/// verbatim; the mud arm's scan checks `active()` on the neighbours.
/// Draws nothing.
pub(crate) fn pass_slush(
    world: &mut World,
    snow_top: i32,
    snow_bottom: i32,
    snow_min_x: &[i32],
    snow_max_x: &[i32],
) {
    let desert_no_surface = world.micro_seeds.contains(MicroSeedFlags::SURFACE_IS_DESERT)
        && world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE);
    let slush_or_sand: u16 = if desert_no_surface { 53 } else { 224 };
    for i in snow_top..snow_bottom {
        for j in snow_min_x[i as usize]..snow_max_x[i as usize] {
            let Some(t) = world.tile(j, i) else { continue };
            match t.kind {
                123 => {
                    world.tile_mut(j, i).unwrap().kind = slush_or_sand;
                }
                59 => {
                    // The 7×7 jungle-grass scan (`:14719-14731`): 60/70/71/72.
                    let mut flag = true;
                    let num = 3;
                    for k in (j - num)..=(j + num) {
                        for l in (i - num)..=(i + num) {
                            if world.tile(k, l).is_some_and(|t| {
                                t.active() && matches!(t.kind, 60 | 70 | 71 | 72)
                            }) {
                                flag = false;
                                break;
                            }
                        }
                    }
                    if flag {
                        world.tile_mut(j, i).unwrap().kind = slush_or_sand;
                    }
                }
                1 => {
                    world.tile_mut(j, i).unwrap().kind = 161;
                }
                _ => {}
            }
        }
    }
}

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the dungeon bricks the
/// Cavinator refuses to carve through (`WorldGen.cs:80038`).
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];
/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the dungeon walls.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// The Mountain Caves pass (`MountainCaveOpenings`,
/// `WorldGen.cs:14873-14887`): per `GenVars.mCaveX/mCaveY` anchor, a
/// `CaveOpenater` surface-opening walk then a recursive `Cavinator` with
/// `Next(40, 50)` recursion budget (the argument draws BEFORE the
/// Cavinator's own draws). A no-op until d101's Mount Caves producer
/// writes the anchors (`numMCaves == 0`).
pub(crate) fn pass_mountain_caves(
    world: &mut World,
    rock_layer: f64,
    num_m_caves: i32,
    m_cave_x: &[i32; 30],
    m_cave_y: &[i32; 30],
    rng: &mut Rng,
) {
    for i in 0..num_m_caves {
        let x = m_cave_x[i as usize];
        let y = m_cave_y[i as usize];
        cave_openater(world, rng, x, y);
        let steps = rng.range(40, 50);
        cavinator(world, rng, x, y, steps, rock_layer);
    }
}

/// `WorldGen.CaveOpenater` (`WorldGen.cs:80085-80166`): the horizontal
/// opening walk. The direction rolls `Next(2)` then the 9-in-10
/// world-half override (`Next(10) != 0` → away from the centre... toward
/// the nearer edge: `i < maxTilesX/2 ? 1 : -1`). The 100-step budget
/// decrements every iteration; the walk stops when the anchor cell has
/// no wall or is an active non-clearable tile — but the CURRENT
/// iteration's disc still carves (the C# has no inner budget break).
/// The port reads `wall == 0` literally: the mountain cells carry wall 2
/// in both engines (vanilla's Dirt Wall Backgrounds, the port's baseline
/// + the same pass), so the sky boundary stops the walk identically.
fn cave_openater(world: &mut World, rng: &mut Rng, i: i32, j: i32) {
    let num = rng.range(7, 12) as f64;
    let mut num3 = 1;
    if rng.range(0, 2) == 0 {
        num3 = -1;
    }
    if rng.range(0, 10) != 0 {
        num3 = if i < world.width / 2 { 1 } else { -1 };
    }
    let (mut cx, mut cy) = (i as f64, j as f64);
    let mut budget = 100;
    let (mut vx, mut vy) = (num3 as f64, 0.0);
    while budget > 0 {
        let t = world.tile(cx as i32, cy as i32);
        let stop = t.is_some_and(|t| {
            t.wall == 0
                || (t.active() && !SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind))
        });
        if stop {
            budget = 0;
        }
        budget -= 1;
        let mut left = (cx - num * 0.5) as i32;
        let mut right = (cx + num * 0.5) as i32;
        let mut top = (cy - num * 0.5) as i32;
        let mut bottom = (cy + num * 0.5) as i32;
        left = left.max(0);
        right = right.min(world.width);
        top = top.max(0);
        bottom = bottom.min(world.height);
        let num2 = num * rng.range(80, 120) as f64 * 0.01;
        for k in left..right {
            for l in top..bottom {
                let dx = (k as f64 - cx).abs();
                let dy = (l as f64 - cy).abs();
                let clearable = world
                    .tile(k, l)
                    .is_some_and(|t| SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind));
                if (dx * dx + dy * dy).sqrt() < num2 * 0.4 && clearable {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
            }
        }
        cx += vx;
        cy += vy;
        vx += rng.range(-10, 11) as f64 * 0.05;
        vy += rng.range(-10, 11) as f64 * 0.05;
        vx = vx.clamp(num3 as f64 - 0.5, num3 as f64 + 0.5);
        vy = vy.clamp(-0.5, 0.0);
    }
}

/// `WorldGen.Cavinator` (`WorldGen.cs:79984-80083`): the recursive cave
/// blob walker. Per segment: `Next(7, 15)` radius, `Next(2)` direction,
/// `Next(20, 40)` step budget, `Next(10, 20) * 0.01` downward velocity;
/// per iteration the `Next(80, 120)` radius wobble, the disc carve with
/// the dungeon veto (a dungeon brick/wall inside the disc zeroes the
/// budget and breaks BOTH loops mid-disc) and the sand/non-clearable
/// skip; the velocity drifts `Next(-10, 11) * 0.05` with the direction ±
/// 0.5 X clamp and the [0, 2] Y clamp. The recursion fires while
/// `steps > 0` and the walker is above `rockLayer + 50`. The
/// dualDungeons bounds veto (`:80033`) is not modelled (no dual-dungeon
/// bounds in the port — the `ocean.rs` precedent).
fn cavinator(world: &mut World, rng: &mut Rng, i: i32, j: i32, steps: i32, rock_layer: f64) {
    let num = rng.range(7, 15) as f64;
    let mut num3 = 1;
    if rng.range(0, 2) == 0 {
        num3 = -1;
    }
    let (mut cx, mut cy) = (i as f64, j as f64);
    let mut budget = rng.range(20, 40);
    let mut vy = rng.range(10, 20) as f64 * 0.01;
    let mut vx = num3 as f64;
    while budget > 0 {
        budget -= 1;
        let mut left = (cx - num * 0.5) as i32;
        let mut right = (cx + num * 0.5) as i32;
        let mut top = (cy - num * 0.5) as i32;
        let mut bottom = (cy + num * 0.5) as i32;
        left = left.max(0);
        right = right.min(world.width);
        top = top.max(0);
        bottom = bottom.min(world.height);
        let num2 = num * rng.range(80, 120) as f64 * 0.01;
        'disc: for k in left..right {
            for l in top..bottom {
                let dx = (k as f64 - cx).abs();
                let dy = (l as f64 - cy).abs();
                if (dx * dx + dy * dy).sqrt() < num2 * 0.4 {
                    let Some(t) = world.tile(k, l) else { continue };
                    // The dungeon veto (`:80038-80042`).
                    if (t.active() && TILE_DUNGEON.contains(&t.kind))
                        || WALL_DUNGEON.contains(&t.wall)
                    {
                        budget = 0;
                        break 'disc;
                    }
                    if t.active()
                        && (!SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind) || t.kind == 53)
                    {
                        continue;
                    }
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
                if budget <= 0 {
                    break;
                }
            }
            if budget <= 0 {
                break;
            }
        }
        cx += vx;
        cy += vy;
        vx += rng.range(-10, 11) as f64 * 0.05;
        vy += rng.range(-10, 11) as f64 * 0.05;
        vx = vx.clamp(num3 as f64 - 0.5, num3 as f64 + 0.5);
        vy = vy.clamp(0.0, 2.0);
    }
    if steps > 0 && (cy as i32) as f64 < rock_layer + 50.0 {
        cavinator(world, rng, cx as i32, cy as i32, steps - 1, rock_layer);
    }
}

/// `Main.tileSand` (`Main.cs:7228-7231`) — the four sand kinds the Gems
/// pass's edge slide moves.
const TILE_SAND_SET: [u16; 4] = [53, 112, 116, 234];

/// The Gems pass (`WorldGen.cs:15050-15137`): the six gem-kind TileRunner
/// bands (63..68, densities 0.3/0.1/0.25/0.45/0.5/0.05 × width × 0.2) with
/// the 3-attempt stone-site do-while, then the two-direction sand-edge
/// slide (the desert rect's columns are exempt). The C#'s
/// `Main.tileSolid[484] = false` poke (`:15055`) is a load-order detail
/// the port's tile data does not model (the `desert.rs` precedent).
pub(crate) fn pass_gems(
    world: &mut World,
    world_surface: f64,
    underground_desert: Option<super::desert::Rect>,
    rng: &mut Rng,
) {
    let opts = TileRunnerOpts::for_world(world);
    for i in 63..=68 {
        // The density table (`:15060-15080`), then `num *= 0.2`.
        let mut num = match i {
            67 => world.width as f64 * 0.5,
            66 => world.width as f64 * 0.45,
            63 => world.width as f64 * 0.3,
            65 => world.width as f64 * 0.25,
            64 => world.width as f64 * 0.1,
            68 => world.width as f64 * 0.05,
            _ => 0.0,
        };
        num *= 0.2;
        let mut j = 0;
        while (j as f64) < num {
            j += 1;
            // The do-while (`:15084-15092`): up to 3 draws, the decrement
            // only on a non-stone site (the C#'s `&&` short-circuit).
            let mut num2 = 3;
            let (mut x, mut y) = (0, 0);
            loop {
                x = rng.range(0, world.width);
                y = rng.range(world_surface as i32, world.height);
                let stone = world.tile(x, y).is_some_and(|t| t.active() && t.kind == 1);
                if stone {
                    break;
                }
                num2 -= 1;
                if num2 <= 0 {
                    break;
                }
            }
            if num2 != 0 {
                let strength = rng.range(2, 6);
                let steps = rng.range(3, 7);
                tile_runner(
                    world,
                    rng,
                    x,
                    y,
                    strength as f64,
                    steps,
                    i,
                    world_surface,
                    &opts,
                );
            }
        }
    }
    // The sand-edge slide (`:15099-15135`).
    gems_sand_edge_slide(world, underground_desert);
}

/// The Gems pass's two-direction sand-edge slide
/// (`WorldGen.cs:15099-15135`): left-to-right then right-to-left, the
/// desert rect's columns exempt (the C#'s `Rectangle.Empty` — the port's
/// `None` — exempts nothing: `l <= 0 || l >= 0` is always true). Draws
/// nothing.
fn gems_sand_edge_slide(world: &mut World, underground_desert: Option<super::desert::Rect>) {
    let (desert_left, desert_right) = match underground_desert {
        Some(r) => (r.left(), r.right()),
        None => (0, 0),
    };
    for k in 0..2 {
        let (num5, num6, num7) = if k == 1 {
            (-1, world.width - 5, 5)
        } else {
            (1, 5, world.width - 5)
        };
        let mut l = num6;
        while l != num7 {
            if l <= desert_left || l >= desert_right {
                for m in 10..(world.height - 10) {
                    let top = world.tile(l, m);
                    let below = world.tile(l, m + 1);
                    let sand_pair = top.is_some_and(|t| t.active() && TILE_SAND_SET.contains(&t.kind))
                        && below.is_some_and(|t| t.active() && TILE_SAND_SET.contains(&t.kind));
                    if sand_pair {
                        let kind = top.unwrap().kind;
                        let num8 = l + num5;
                        let mut n = m + 1;
                        if !world.tile(num8, m).is_some_and(|t| t.active())
                            && !world.tile(num8, n).is_some_and(|t| t.active())
                        {
                            // The fall (`:15123-15126`): down while
                            // inactive, inside the 10-tile world margin.
                            while !world.tile(num8, n).is_some_and(|t| t.active())
                                && in_world(world, num8, n, 10)
                            {
                                n += 1;
                            }
                            n -= 1;
                            if let Some(t) = world.tile_mut(l, m) {
                                t.flags &= !FLAG_ACTIVE;
                            }
                            if let Some(t) = world.tile_mut(num8, n) {
                                t.flags |= FLAG_ACTIVE;
                                t.kind = kind;
                            }
                        }
                    }
                }
            }
            l += num5;
        }
    }
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// The Clean Up Dirt pass (`DirtWallCleanup`, `WorldGen.cs:15246-15359`):
/// the two surface wall-scrub sweeps — left-to-right (`i` in `3..width-3`)
/// then right-to-left (`width-5..=5`), each column walking `j` from 0 while
/// `(double)j < worldSurface`. While `flag` is set the centre wall
/// (2/40/64/86 first sweep, 2/40/64 second) zeroes; unless the cell is an
/// active sand (53/112/234 first sweep, 53 only second) the six neighbour
/// walls scrub too — ±1 unconditional, ±2/±3 gated on `Next(2) == 0` (the
/// draw only fires when the neighbour wall test passes — the C# `&&`).
/// An active cell clears `flag`; the else arm re-sets it once a 5-row
/// zero-wall air pocket opens (`:15301`/`:15351`). The C#'s duplicated
/// `== 40` terms in every neighbour test (`:15271` etc. — 64/86 absent
/// there) are kept verbatim: the neighbour set is {2, 40}.
///
/// The `noSurface` micro-seed arm (`:15250-15253`) runs
/// `DoNoSurfaceFillTheTop` instead. The port's baseline wall 2 under the
/// solid rows IS the vanilla Dirt Wall Backgrounds wall the sweep scrubs
/// (the equality tests read identically — no `BASELINE_WALLS` adaptation
/// needed, unlike the `wall > 0` gates).
pub(crate) fn pass_clean_up_dirt(world: &mut World, world_surface: f64, rng: &mut Rng) {
    // `:15248` — the skyblock outer gate (module doc).
    if world.seeds.contains(SeedFlags::SKYBLOCK) {
        return;
    }
    if world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE) {
        do_no_surface_fill_the_top(world, rng);
        return;
    }
    // The first sweep (`:15257-15306`).
    for i in 3..world.width - 3 {
        let mut flag = true;
        let mut j = 0;
        while (j as f64) < world_surface {
            if flag {
                let wall = world.tile(i, j).map_or(0, |t| t.wall);
                if matches!(wall, 2 | 40 | 64 | 86) {
                    world.tile_mut(i, j).unwrap().wall = 0;
                }
                let t = world.tile(i, j).unwrap();
                if !t.active() || !matches!(t.kind, 53 | 112 | 234) {
                    scrub_neighbours(world, rng, i, j);
                    if world.tile(i, j).unwrap().active() {
                        flag = false;
                    }
                }
            } else if scrub_resume(world, i, j) {
                flag = true;
            }
            j += 1;
        }
    }
    // The second sweep (`:15307-15356`): right-to-left, the centre set
    // loses 86 and the sand gate narrows to 53.
    let mut num = world.width - 5;
    while num >= 5 {
        let mut flag2 = true;
        let mut k = 0;
        while (k as f64) < world_surface {
            if flag2 {
                let wall = world.tile(num, k).map_or(0, |t| t.wall);
                if matches!(wall, 2 | 40 | 64) {
                    world.tile_mut(num, k).unwrap().wall = 0;
                }
                let t = world.tile(num, k).unwrap();
                if !t.active() || t.kind != 53 {
                    scrub_neighbours(world, rng, num, k);
                    if world.tile(num, k).unwrap().active() {
                        flag2 = false;
                    }
                }
            } else if scrub_resume(world, num, k) {
                flag2 = true;
            }
            k += 1;
        }
        num -= 1;
    }
}

/// The six neighbour scrubs (`:15271-15294` / `:15321-15344`), in the C#'s
/// order: i-1 unconditional, i-2/i-3 `Next(2)`-gated, i+1 unconditional,
/// i+2/i+3 gated. The wall test is {2, 40} — the C#'s `== 2 || == 40 ||
/// == 40` duplication kept as the set it computes.
fn scrub_neighbours(world: &mut World, rng: &mut Rng, i: i32, j: i32) {
    for (dx, gated) in [(-1i32, false), (-2, true), (-3, true), (1, false), (2, true), (3, true)] {
        let wall = world.tile(i + dx, j).map_or(0, |t| t.wall);
        if matches!(wall, 2 | 40) && (!gated || rng.range(0, 2) == 0) {
            world.tile_mut(i + dx, j).unwrap().wall = 0;
        }
    }
}

/// The flag-resume pocket test (`:15301` / `:15351`): the centre column's
/// five walls (j..j+4) and the four side walls (±1/±2 at j) all zero, and
/// the centre four tiles (j..j+3) inactive.
fn scrub_resume(world: &World, i: i32, j: i32) -> bool {
    let wall_zero = |x: i32, y: i32| world.tile(x, y).is_some_and(|t| t.wall == 0);
    let inactive = |y: i32| world.tile(i, y).is_some_and(|t| !t.active());
    wall_zero(i, j)
        && wall_zero(i, j + 1)
        && wall_zero(i, j + 2)
        && wall_zero(i, j + 3)
        && wall_zero(i, j + 4)
        && wall_zero(i - 1, j)
        && wall_zero(i + 1, j)
        && wall_zero(i - 2, j)
        && wall_zero(i + 2, j)
        && inactive(j)
        && inactive(j + 1)
        && inactive(j + 2)
        && inactive(j + 3)
}

/// `SecretSeed.DoNoSurfaceFillTheTop` (`WorldGen.cs:1055-1082`): per
/// column, fill the top rows active — the loop bound `100 + Next(2)` is
/// re-evaluated EVERY condition check (the C# `for` draws on each
/// iteration test, so the column runs 100 or 101 rows and the final
/// failing check still draws). Dungeon walls/tiles are exempt; jungle
/// grass 60 and mushroom grass 70 retype to mud 59. The inner
/// `skyblockWorldGen` guard (`:1057`) is redundant behind the pass's own
/// `!denyAllGeneration` gate — the adapter never calls this on skyblock.
fn do_no_surface_fill_the_top(world: &mut World, rng: &mut Rng) {
    for i in 0..world.width {
        let mut j = 0;
        while j < 100 + rng.range(0, 2) {
            let t = world.tile(i, j).unwrap();
            if !WALL_DUNGEON.contains(&t.wall) {
                if !TILE_DUNGEON.contains(&t.kind) {
                    world.tile_mut(i, j).unwrap().flags |= FLAG_ACTIVE;
                }
                let kind = world.tile(i, j).unwrap().kind;
                if kind == 60 || kind == 70 {
                    world.tile_mut(i, j).unwrap().kind = 59;
                }
            }
            j += 1;
        }
    }
}

/// The Dirt Rock Wall Runner pass (`WorldGen.cs:15461-15475`): one
/// candidate per world column — x `Next(10, width-10)` then y
/// `Next(10, (int)worldSurface)`, both drawn BEFORE the wall test; only a
/// wall-2 cell fires `DirtyRockRunner`. The pass sets no progress message
/// (the C# body has none). The port's baseline wall 2 under the surface
/// rows IS the vanilla Dirt Wall Backgrounds wall the runner retypes (the
/// equality test reads identically — no `BASELINE_WALLS` adaptation).
///
/// The sub-vanilla valve: the C#'s `Next(10, width-10)` /
/// `Next(10, (int)worldSurface)` bands throw when `width < 20` /
/// `worldSurface < 10`; the port skips the pass before any draw (the
/// `mining explosives` width-guard precedent).
pub(crate) fn pass_dirt_rock_wall_runner(world: &mut World, world_surface: f64, rng: &mut Rng) {
    // `:15463` — the skyblock outer gate (module doc).
    if world.seeds.contains(SeedFlags::SKYBLOCK) {
        return;
    }
    if world.width < 20 || (world_surface as i32) < 10 {
        return;
    }
    for _ in 0..world.width {
        let x = rng.range(10, world.width - 10);
        let y = rng.range(10, world_surface as i32);
        if world.tile(x, y).is_some_and(|t| t.wall == 2) {
            dirty_rock_runner(world, rng, x, y);
        }
    }
}

/// `WorldGen.DirtyRockRunner` (`WorldGen.cs:77145-77210`): the wall-2→59
/// blob walk. Strength `Next(2, 6)` and steps `Next(5, 50)` draw first,
/// then the two `Next(-10, 11) * 0.1` velocity components. Each of the
/// `steps` iterations sizes the disc `strength * (budget / steps)` (the
/// budget decrements AFTER the size), clamps the `(int)`-truncated rect to
/// the world, and retypes every wall-2 cell inside the Manhattan-distance
/// gate — the per-cell `Next(-10, 11) * 0.015` jitter draw fires for EVERY
/// rect cell (the C# evaluates the comparison's right side before the
/// `&& wall == 2` test). The walker then advances and jitters both
/// velocity components (`* 0.05`, clamped to ±1 — the C#'s two ifs).
fn dirty_rock_runner(world: &mut World, rng: &mut Rng, i: i32, j: i32) {
    let strength = rng.range(2, 6) as f64;
    let steps = rng.range(5, 50) as f64;
    let mut budget = steps;
    let (mut cx, mut cy) = (i as f64, j as f64);
    let (mut vx, mut vy) = (
        rng.range(-10, 11) as f64 * 0.1,
        rng.range(-10, 11) as f64 * 0.1,
    );
    while strength > 0.0 && budget > 0.0 {
        let size = strength * (budget / steps);
        budget -= 1.0;
        let mut left = (cx - size * 0.5) as i32;
        let mut right = (cx + size * 0.5) as i32;
        let mut top = (cy - size * 0.5) as i32;
        let mut bottom = (cy + size * 0.5) as i32;
        left = left.max(0);
        right = right.min(world.width);
        top = top.max(0);
        bottom = bottom.min(world.height);
        for k in left..right {
            for l in top..bottom {
                let jitter = 1.0 + rng.range(-10, 11) as f64 * 0.015;
                let dist = (k as f64 - cx).abs() + (l as f64 - cy).abs();
                if dist < strength * 0.5 * jitter
                    && world.tile(k, l).is_some_and(|t| t.wall == 2)
                {
                    world.tile_mut(k, l).unwrap().wall = 59;
                }
            }
        }
        cx += vx;
        cy += vy;
        vx += rng.range(-10, 11) as f64 * 0.05;
        vx = vx.clamp(-1.0, 1.0);
        vy += rng.range(-10, 11) as f64 * 0.05;
        vy = vy.clamp(-1.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone world below row 100, walls 0 — the Silt band's canvas.
    fn stone_world(width: i32, height: i32) -> World {
        let mut w = World::new("silt".into(), width, height);
        for x in 0..width {
            for y in 100..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 100.0;
        w.rock_layer = 120.0;
        w
    }

    fn silt_count(w: &World) -> usize {
        w.tiles.iter().filter(|t| t.kind == 123).count()
    }

    #[test]
    fn silt_blobs_land_below_the_rock_line_and_replay() {
        let mut a = stone_world(600, 600);
        let mut b = stone_world(600, 600);
        pass_silt(&mut a, 120.0, 100.0, 120.0, &mut Rng(0x5117));
        pass_silt(&mut b, 120.0, 100.0, 120.0, &mut Rng(0x5117));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert!(silt_count(&a) > 0, "the pass places silt");
        // Every silt cell sits at or below the rock-line band's floor.
        for y in 0..100 {
            for x in 0..a.width {
                assert_ne!(a.tile(x, y).unwrap().kind, 123, "silt above the band");
            }
        }
    }

    #[test]
    fn the_desert_walls_skip_every_candidate() {
        // `:13159`/`:13174`: wall 187/216 candidates skip the runner — a
        // fully sandstone-walled band places nothing and draws only the
        // x/y pairs.
        let mut w = stone_world(600, 600);
        for x in 0..w.width {
            for y in 100..w.height {
                let i = w.index(x, y);
                w.tiles[i].wall = if x % 2 == 0 { 187 } else { 216 };
            }
        }
        pass_silt(&mut w, 120.0, 100.0, 120.0, &mut Rng(0x5117));
        assert_eq!(silt_count(&w), 0);
    }

    /// A stone world with an air pocket at rows 120..140 — the webs'
    /// ceiling canvas.
    fn cave_world() -> World {
        let mut w = stone_world(600, 600);
        for x in 0..w.width {
            for y in 120..140 {
                let i = w.index(x, y);
                w.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        w
    }

    fn web_count(w: &World) -> usize {
        w.tiles
            .iter()
            .filter(|t| t.active() && t.kind == 51)
            .count()
    }

    #[test]
    fn webs_hug_the_pocket_ceiling_and_replay() {
        let mut a = cave_world();
        let mut b = cave_world();
        let anchors = [0i32; 30];
        pass_webs(&mut a, 100.0, 90.0, 100.0, 0, &anchors, &anchors, &mut Rng(0xEB));
        pass_webs(&mut b, 100.0, 90.0, 100.0, 0, &anchors, &anchors, &mut Rng(0xEB));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert!(web_count(&a) > 0, "the pass places cobwebs");
        // Every cobweb sits inside the pocket band (the ceiling walk never
        // leaves the carved rows).
        for y in 0..a.height {
            for x in 0..a.width {
                if a.tile(x, y).is_some_and(|t| t.active() && t.kind == 51) {
                    assert!((120..140).contains(&y), "web outside the pocket");
                }
            }
        }
    }

    #[test]
    fn a_solid_world_grows_no_webs() {
        // Every candidate below worldSurfaceHigh is active — the entry
        // gate (`:13621`) rejects them all without a direction draw.
        let mut w = stone_world(600, 600);
        let anchors = [0i32; 30];
        pass_webs(&mut w, 100.0, 90.0, 100.0, 0, &anchors, &anchors, &mut Rng(0xEB));
        assert_eq!(web_count(&w), 0);
    }

    #[test]
    fn slush_retypes_the_snow_rect_and_respects_the_jungle_scan() {
        let mut w = World::new("slush".into(), 100, 100);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let i = w.index(x, y);
            w.tiles[i].kind = kind;
            w.tiles[i].flags |= FLAG_ACTIVE;
        };
        put(&mut w, 10, 10, 123); // silt → slush
        put(&mut w, 11, 10, 1); // stone → ice
        put(&mut w, 12, 10, 59); // mud near jungle grass → stays
        put(&mut w, 14, 12, 60); // the jungle grass (within 3 of 12,10)
        put(&mut w, 30, 10, 59); // mud far from grass → slush
        put(&mut w, 60, 10, 123); // outside the rect → stays silt
        let min_x = vec![5i32; 100];
        let max_x = vec![50i32; 100];
        pass_slush(&mut w, 8, 20, &min_x, &max_x);
        assert_eq!(w.tile(10, 10).unwrap().kind, 224);
        assert_eq!(w.tile(11, 10).unwrap().kind, 161);
        assert_eq!(w.tile(12, 10).unwrap().kind, 59, "the jungle scan vetoes");
        assert_eq!(w.tile(30, 10).unwrap().kind, 224);
        assert_eq!(w.tile(60, 10).unwrap().kind, 123, "outside the rect");
    }

    #[test]
    fn the_desert_no_surface_pair_retypes_to_sand() {
        let mut w = World::new("slush".into(), 100, 100);
        w.micro_seeds |= MicroSeedFlags::SURFACE_IS_DESERT | MicroSeedFlags::NO_SURFACE;
        let i = w.index(10, 10);
        w.tiles[i].kind = 123;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let min_x = vec![5i32; 100];
        let max_x = vec![50i32; 100];
        pass_slush(&mut w, 8, 20, &min_x, &max_x);
        assert_eq!(w.tile(10, 10).unwrap().kind, 53);
    }

    #[test]
    fn the_empty_snow_rect_is_a_no_op() {
        // The blocked-producer state: snowTop == snowBottom == 0 until
        // Generate Ice Biome (d101) lands — the sweep touches nothing.
        let mut w = stone_world(100, 100);
        let before = w.tiles.clone();
        pass_slush(&mut w, 0, 0, &[], &[]);
        assert_eq!(w.tiles, before);
    }

    #[test]
    fn mountain_caves_carve_from_the_anchors_and_replay() {
        let mut a = stone_world(600, 600);
        let mut b = stone_world(600, 600);
        // Wall 2 everywhere below the surface — the Dirt Wall Backgrounds
        // state the CaveOpenater walk needs.
        for w in [&mut a, &mut b] {
            for x in 0..w.width {
                for y in 100..w.height {
                    let i = w.index(x, y);
                    w.tiles[i].wall = 2;
                }
            }
        }
        let mut xs = [0i32; 30];
        let mut ys = [0i32; 30];
        xs[0] = 300;
        ys[0] = 150;
        pass_mountain_caves(&mut a, 120.0, 1, &xs, &ys, &mut Rng(0xCAFE));
        pass_mountain_caves(&mut b, 120.0, 1, &xs, &ys, &mut Rng(0xCAFE));
        assert_eq!(a.tiles, b.tiles, "a == b");
        let carved = a
            .tiles
            .iter()
            .filter(|t| !t.active() && t.kind == 1)
            .count();
        assert!(carved > 50, "the walkers carve a cave: {carved}");
    }

    #[test]
    fn zero_anchors_is_a_no_op() {
        let mut w = stone_world(600, 600);
        let before = w.tiles.clone();
        pass_mountain_caves(&mut w, 120.0, 0, &[0; 30], &[0; 30], &mut Rng(0xCAFE));
        assert_eq!(w.tiles, before);
    }

    #[test]
    fn the_cavinator_stops_at_a_dungeon_wall() {
        // A dungeon-brick column across the walk's path vetoes the disc
        // (`:80038-80042`) — nothing beyond it carves.
        let mut w = stone_world(600, 600);
        for x in 0..w.width {
            for y in 100..w.height {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        for y in 100..w.height {
            let i = w.index(320, y);
            w.tiles[i].kind = 41;
            w.tiles[i].wall = 7;
        }
        let mut xs = [0i32; 30];
        let mut ys = [0i32; 30];
        xs[0] = 300;
        ys[0] = 150;
        pass_mountain_caves(&mut w, 120.0, 1, &xs, &ys, &mut Rng(0xCAFE));
        // The dungeon column itself never carves.
        for y in 100..w.height {
            assert!(w.tile(320, y).unwrap().active(), "dungeon brick carved");
        }
    }

    #[test]
    fn gems_vein_the_stone_and_replay() {
        let mut a = stone_world(600, 600);
        let mut b = stone_world(600, 600);
        pass_gems(&mut a, 100.0, None, &mut Rng(0x6E35));
        pass_gems(&mut b, 100.0, None, &mut Rng(0x6E35));
        assert_eq!(a.tiles, b.tiles, "a == b");
        let gems = a
            .tiles
            .iter()
            .filter(|t| t.active() && (63..=68).contains(&t.kind))
            .count();
        assert!(gems > 0, "the pass veins gems");
    }

    #[test]
    fn the_sand_edge_slide_conserves_sand_and_replays() {
        // A sand column at x=100 beside an air gap at x=101 with a floor:
        // the two-direction slide shuffles the paired cells into the gap
        // and back, conserving the sand count (each move deactivates one
        // cell and activates one). No RNG — deterministic by construction.
        let mut w = World::new("gems".into(), 200, 200);
        for y in 10..150 {
            let i = w.index(100, y);
            w.tiles[i].kind = 53;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let i = w.index(101, 150);
        w.tiles[i].kind = 1;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let sand_before = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 53)
            .count();
        let mut twin = w.clone();
        gems_sand_edge_slide(&mut w, None);
        gems_sand_edge_slide(&mut twin, None);
        assert_eq!(w.tiles, twin.tiles, "a == b");
        let sand_after = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 53)
            .count();
        assert_eq!(sand_before, sand_after, "the slide conserves sand");
        // Some sand reached the gap column.
        assert!(
            (0..w.height).any(|y| w.tile(101, y).is_some_and(|t| t.active() && t.kind == 53)),
            "sand slid into the gap"
        );
    }

    /// A sky band with dirt walls over an active dirt floor at row 40 —
    /// the Clean Up Dirt canvas.
    fn walled_sky_world() -> World {
        // 120 tall: the noSurface fill arm reaches row 100/101
        // (`100 + Next(2)`), so the canvas must clear that band.
        let mut w = World::new("cleanup".into(), 100, 120);
        for x in 0..100 {
            for y in 0..40 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
            for y in 40..120 {
                let i = w.index(x, y);
                w.tiles[i].kind = 0;
                w.tiles[i].flags |= FLAG_ACTIVE;
                w.tiles[i].wall = 2;
            }
        }
        w.world_surface = 60.0;
        w
    }

    #[test]
    fn the_scrub_clears_the_sky_walls_and_replays() {
        let mut a = walled_sky_world();
        let mut b = walled_sky_world();
        pass_clean_up_dirt(&mut a, 60.0, &mut Rng(0xC1EA));
        pass_clean_up_dirt(&mut b, 60.0, &mut Rng(0xC1EA));
        assert_eq!(a.tiles, b.tiles, "a == b");
        // The sky band above the floor scrubs clean (the ±1 unconditional
        // arms reach every column from both sweeps).
        for x in 3..97 {
            for y in 0..39 {
                assert_eq!(a.tile(x, y).unwrap().wall, 0, "sky wall at {x},{y}");
            }
        }
        // The first active row's OWN wall zeroes too (the centre scrub at
        // `:15265` runs before the flag clears); the rows BELOW the floor
        // keep their walls — the flag stays false and the pocket never
        // re-opens (the floor rows are solid).
        assert_eq!(a.tile(50, 40).unwrap().wall, 0);
        assert_eq!(a.tile(50, 41).unwrap().wall, 2);
    }

    #[test]
    fn active_sand_keeps_the_flag_and_its_neighbours() {
        // An active sand 53 cell does NOT enter the neighbour-scrub branch
        // (`:15269`) — the flag stays true and the column keeps scrubbing
        // below it; the sand cell's own wall still zeroes.
        let mut w = walled_sky_world();
        for y in 0..40 {
            let i = w.index(50, y);
            w.tiles[i].kind = 53;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        pass_clean_up_dirt(&mut w, 60.0, &mut Rng(0xC1EA));
        assert_eq!(w.tile(50, 10).unwrap().wall, 0, "the centre wall zeroes");
        assert_eq!(w.tile(50, 10).unwrap().kind, 53, "the sand stays");
    }

    #[test]
    fn the_no_surface_arm_fills_the_top_and_replays() {
        let mut a = walled_sky_world();
        let mut b = walled_sky_world();
        a.micro_seeds |= MicroSeedFlags::NO_SURFACE;
        b.micro_seeds |= MicroSeedFlags::NO_SURFACE;
        // A jungle-grass cell inside the fill band retypes to mud.
        for w in [&mut a, &mut b] {
            let i = w.index(10, 10);
            w.tiles[i].kind = 60;
        }
        pass_clean_up_dirt(&mut a, 60.0, &mut Rng(0xC1EA));
        pass_clean_up_dirt(&mut b, 60.0, &mut Rng(0xC1EA));
        assert_eq!(a.tiles, b.tiles, "a == b");
        // The top 100 rows are active everywhere (the band is 100/101).
        for x in 0..100 {
            for y in 0..100 {
                assert!(a.tile(x, y).unwrap().active(), "fill at {x},{y}");
            }
        }
        assert_eq!(a.tile(10, 10).unwrap().kind, 59, "jungle grass → mud");
        // The walls are untouched by the fill arm — even below the surface
        // line, where the scrub arm would have cleared them.
        assert_eq!(a.tile(10, 10).unwrap().wall, 2);
        assert_eq!(a.tile(10, 45).unwrap().wall, 2);
    }

    #[test]
    fn the_no_surface_fill_exempts_dungeon_tiles() {
        let mut w = walled_sky_world();
        w.micro_seeds |= MicroSeedFlags::NO_SURFACE;
        let i = w.index(10, 10);
        w.tiles[i].kind = 41; // a dungeon brick
        w.tiles[i].flags &= !FLAG_ACTIVE;
        w.tiles[i].wall = 7; // a dungeon wall
        pass_clean_up_dirt(&mut w, 60.0, &mut Rng(0xC1EA));
        assert!(!w.tile(10, 10).unwrap().active(), "the dungeon wall exempts");
        assert_eq!(w.tile(10, 10).unwrap().kind, 41);
    }

    /// A dirt-wall canvas: wall 2 over the whole world — the Dirt Rock
    /// Wall Runner's retype substrate.
    fn dirt_wall_world() -> World {
        let mut w = World::new("dirtyrock".into(), 200, 200);
        for x in 0..200 {
            for y in 0..200 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        w.world_surface = 60.0;
        w
    }

    #[test]
    fn the_dirty_rock_blobs_retype_wall_two_and_replay() {
        let mut a = dirt_wall_world();
        let mut b = dirt_wall_world();
        pass_dirt_rock_wall_runner(&mut a, 60.0, &mut Rng(0xD1B7));
        pass_dirt_rock_wall_runner(&mut b, 60.0, &mut Rng(0xD1B7));
        assert_eq!(a.tiles, b.tiles, "a == b");
        let retyped = a.tiles.iter().filter(|t| t.wall == 59).count();
        assert!(retyped > 0, "the runners retype wall 2 -> 59");
        // The blobs stay inside the reachable band: candidates draw
        // y < 60, the walker drifts at most +1/step over ≤49 steps and the
        // disc radius is ≤2.5 — no write can reach row 110.
        for x in 0..200 {
            for y in 110..200 {
                assert_eq!(a.tile(x, y).unwrap().wall, 2, "deep wall at {x},{y}");
            }
        }
    }

    #[test]
    fn a_non_dirt_wall_candidate_draws_but_does_not_run() {
        // Every cell carries wall 1 (stone): both candidate draws still
        // fire per column, but no runner starts — the world is untouched.
        let mut w = dirt_wall_world();
        for t in &mut w.tiles {
            t.wall = 1;
        }
        let before = w.tiles.clone();
        pass_dirt_rock_wall_runner(&mut w, 60.0, &mut Rng(0xD127));
        assert_eq!(w.tiles, before, "no wall-2 cell, no retype");
    }

    #[test]
    fn the_sub_vanilla_valve_skips_before_any_draw() {
        let mut w = World::new("tiny".into(), 16, 16);
        for t in &mut w.tiles {
            t.wall = 2;
        }
        w.world_surface = 12.0;
        let before = w.tiles.clone();
        let mut rng = Rng(0xD127);
        pass_dirt_rock_wall_runner(&mut w, 12.0, &mut rng);
        assert_eq!(w.tiles, before, "the width valve skips the pass");
        assert_eq!(rng.next_u32(), Rng(0xD127).next_u32(), "no draw fired");
    }
}
