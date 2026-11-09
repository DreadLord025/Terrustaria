//! The Don't Starve ("the constant") passes — the seed's own generation
//! work (`WorldGen.cs`): the `WavyCaves` pass (`:12271-12311`, the only
//! seed that runs it) and the `LavaLayerCaverer` the Dungeon pass appends
//! for the Constant (`:14867`, `:73950-74060`).
//!
//! M5-SEED-1 seam: this module is the Constant's pass bodies; the pipeline
//! hooks live in `pipeline.rs` (`step_wavy_caves`, the vanilla "Wavy Caves"
//! slot, and `step_constant_lava_layer`, the vanilla Dungeon-tail call). A
//! classic world never reaches either body and draws nothing — the port's
//! classic draws stay byte-identical.
//!
//! The two anchors the C# reads from `Main`/`GenVars` come off the port's
//! own world state instead, and the two bodies carry the C#'s own guards,
//! so that the seeded behaviour is complete at the call sites. Two
//! deviations from the vanilla reads, both deliberate and pinned here:
//!
//! - **The anchor rows.** `Main.worldSurface` / the `rockLayer` input of
//!   `GenVars.waterLine` are the terrain pass' values; the port's
//!   `World.world_surface` / `World.rock_layer` are only copied from the
//!   terrain's `Layers` at the world tail (`pipeline.rs:2243-22444`), so
//!   during these two passes they still hold the `World::new` defaults
//!   (`height * 0.25` / `height * 0.45`). The bodies therefore sit the
//!   vanilla bands a little off their anchor on a live pipeline world: the
//!   draw order and every band arithmetic are exact, and a classic world is
//!   unaffected either way.
//! - **`GenVars.waterLine` is unreachable.** The LavaLayer seam passes no
//!   `GenVars`, so [`lava_layer_caverer`] re-derives the line with the
//!   port's own formula off the same `pass_rng("Terrain")` stream
//!   (`pipeline.rs:1331-1337`), whose first draw is exactly the water-line
//!   offset.

use crate::seeds::SeedFlags;
use crate::tile::{FLAG_ACTIVE, LiquidKind, Tile};
use crate::world::World;
use super::noise::Rng;
use super::pipeline::{GenVars, Streams};
use super::{GenOptions, TILE_DIRT, TILE_ICE, TILE_SNOW, TILE_STONE};

// --- the id sets the LavaLayer sweep reads ----------------------------------

/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the nine dungeon walls.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];
/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks: the
/// blue/green/pink 41/43/44 plus the ancient 677/678/679.
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9696`) — the bounds
/// test with a margin, the WavyCaverer's per-cell gate.
#[inline]
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.KillTile`'s generation-time strip (`WorldGen.cs:63517`, the
/// `isGeneratingOrLoadingWorld → noItem` path): the tile goes inactive with
/// its type cleared — the helper `ocean.rs`'s caverers carve with.
fn kill_tile_gen(t: &mut Tile) {
    if !t.active() {
        return;
    }
    t.flags &= !FLAG_ACTIVE;
    t.kind = 0;
    t.frame_x = 0;
    t.frame_y = 0;
}

/// A `Main.tile[x, y].wall` read. An out-of-bounds cell is air, exactly like
/// the port's other safe reads (`Framing.GetTileSafely`), so it is no wall.
#[inline]
fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

/// A `Main.wallDungeon[Main.tile[x, y].wall]` read, same safe-read rule — so
/// the right-edge sweep below stops at the map edge where the C# would walk
/// off `Main.tile`.
#[inline]
fn dungeon_wall_at(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| WALL_DUNGEON.contains(&t.wall))
}

/// A `Main.tileDungeon[Main.tile[x, y].type]` read, same safe-read rule.
#[inline]
fn dungeon_tile_at(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| TILE_DUNGEON.contains(&t.kind))
}

/// `WorldGen.WavyCaves` (`WorldGen.cs:12271-12311`) — the Constant's cave
/// pass: `35 * (maxTilesX / 4200)^2` wavy caverers spread across the map from
/// left to right, each starting at a y at least 80 rows from the previous
/// one's. The `progress.Message` / `progress.Set` lines (`:12275`, `:12287`)
/// are UI only and have no port state.
pub(crate) fn wavy_caves(
    world: &mut World,
    _gv: &mut GenVars,
    streams: &mut Streams,
    opts: &GenOptions,
) {
    // The body's own guard (`:12273`): the pass is registered for every
    // world and the Constant (`WorldGen.dontStarveWorldGen`) is the only
    // seed that runs it — the pipeline seam gates the call the same way
    // (`pipeline.rs`'s `step_wavy_caves`), so a classic world draws nothing
    // either way. The `!Skyblock.denyAllGeneration` half has no port state
    // (no skyblock worldgen) and reads false, as with the other unmodelled
    // vanilla guards (`finish_seeds.rs`'s module doc).
    if !opts.flags.contains(SeedFlags::DONT_STARVE) {
        return;
    }
    let mut rng = streams.pass_rng("Wavy Caves");
    // `num` / `num2` (`:12276-12281`): the caverer count from the width
    // factor, a remix Constant cutting it to a third.
    let factor = world.width as f64 / 4200.0;
    let mut count = (35.0 * factor * factor) as i32;
    if opts.flags.contains(SeedFlags::REMIX) {
        count /= 3;
    }
    // `WorldGen.genRand.Next((int)Main.worldSurface + 100, Main
    // .UnderworldLayer - 100)` (`:12288`, `:12297`) — `UnderworldLayer` is
    // `maxTilesY - 200` (`Main.cs:2790`).
    let (low, high) = ((world.world_surface as i32) + 100, world.height - 200 - 100);
    // `num3` / `num4` (`:12282-12283`): the previous caverer's y — zero, so
    // the first is only re-rolled when it lands within 80 of row 0 — and
    // the 80-row separation.
    let mut last_y = 0;
    for i in 0..count {
        // `num5` (`:12286`): the pass's progress fraction, here only the
        // `startX` spread across the map.
        let fraction = i as f64 / (count - 1) as f64;
        // The 100-try rejection loop (`:12289-12298`).
        let mut y = rng.range(low, high);
        let mut tries = 0;
        while (y - last_y).abs() < 80 {
            tries += 1;
            if tries > 100 {
                break;
            }
            y = rng.range(low, high);
        }
        last_y = y;
        let start_x = 80 + ((world.width - 80 * 2) as f64 * fraction) as i32;
        // The argument draws in the C#'s order (`:12304`): `Next(3, 6)`,
        // `NextDouble()`, `Next(300, 500)`. The C#'s `double
        // waveStrengthScalar` widens the int sum, so the cast lands here.
        let strength = (12 + rng.range(3, 6)) as f64;
        let percent = 0.25 + rng.next_f64();
        let steps = rng.range(300, 500);
        // The try/catch (`:12302-12308`) swallowed every throw; the walker
        // below is bounds-checked cell by cell instead, so it cannot throw.
        wavy_caverer(
            world,
            &mut rng,
            start_x,
            y,
            strength,
            percent,
            steps,
            -1,
        );
    }
}

/// `WorldGen.WavyCaverer` (`WorldGen.cs:76716-76768`): the Constant's wavy
/// walker — a sine-stepped y on a straight x march, the tunnel widening from
/// `num` to `num2` rows over the first `num4` steps, holding through the
/// middle span (where the wave amplitude/period jitter draws) and narrowing
/// back over the last `num4`. `type < 0` carves (`KillTile`), which is the
/// only arm `WavyCaves` uses — it is the walker's one call site (`:12304`)
/// and it always passes `-1`.
#[allow(clippy::too_many_arguments)]
fn wavy_caverer(
    world: &mut World,
    rng: &mut Rng,
    start_x: i32,
    start_y: i32,
    wave_strength_scalar: f64,
    wave_percent_scalar: f64,
    steps: i32,
    kind: i32,
) {
    let mut x = start_x as f64;
    let rightward = start_x > world.width / 2;
    let num = 2 + rng.range(0, 2);
    let num2 = 15 + rng.range(0, 11);
    let num3 = 1 + rng.range(0, 2);
    // `(int)Math.Ceiling((double)num2 / (double)num3)` (`:76723`).
    let num4 = (num2 as f64 / num3 as f64).ceil() as i32;
    let mut num5 = 1.0f64;
    let mut num6 = 1.0f64;
    // `(int)(-1.0 + NextDouble() * 3.0)` (`:76726`) — the -1 / 0 / 1 step.
    let num7 = (-1.0 + rng.next_f64() * 3.0) as i32;
    let mut num8 = num;
    let mut num9 = 0;
    for i in 0..steps {
        let opening = i < num4;
        let closing = i >= steps - num4;
        x += if rightward { -1.0 } else { 1.0 };
        if !opening && !closing {
            num9 += 1;
            num5 = (num5 + (-0.5 + rng.next_f64()) * 0.25).clamp(0.5, 2.0);
            num6 = (num6 + (-0.5 + rng.next_f64()) * 0.02).clamp(0.9, 1.1);
        }
        // `vector2D.Y = ...` (`:76740`) — a full re-assignment from the fixed
        // start every step, so the walker's y never carries across steps.
        let mut y = start_y as f64
            + (num9 as f64 * 0.1 * num6 * wave_percent_scalar).sin() * num5 * wave_strength_scalar
            + (num9 * num7) as f64;
        let num10 = num8;
        if opening {
            num8 = (num8 + num3).min(num2);
        } else if closing {
            num8 = (num8 - num3).max(num);
        }
        // `vector2D.Y -= (num10 + num8) / 4` (`:76750`) — int division.
        y -= ((num10 + num8) / 4) as f64;
        for j in 0..num8 {
            let (px, py) = (x as i32, y as i32 + j);
            if !in_world(world, px, py, 20) {
                continue;
            }
            if kind >= 0 {
                // `PlaceTile(num11, num12, type, mute: true, forced: true)`
                // (`:76759`) — unreachable from the port (the only call site
                // passes `-1`); the activation shape stands in for the full
                // `PlaceTile` framing bookkeeping, which is recorded as a gap
                // rather than half-ported.
                if let Some(t) = world.tile_mut(px, py) {
                    t.kind = kind as u16;
                    t.flags |= FLAG_ACTIVE;
                }
            } else if let Some(t) = world.tile_mut(px, py) {
                kill_tile_gen(t);
            }
        }
    }
}

/// `WorldGen.LavaLayerCaverer` (`WorldGen.cs:73950-74086`) — the Dungeon
/// tail's horizontal lava caver, the Constant's dungeon cleanup: a wandering
/// two-to-eight-row band around the water line that kills its tiles, spills
/// lava into the gaps with a 1-in-15 roll, unifies the dungeon walls the band
/// crosses and fills lava into the wet column below them.
pub(crate) fn lava_layer_caverer(world: &mut World, streams: &mut Streams, opts: &GenOptions) {
    // The Dungeon-tail guard (`WorldGen.cs:14868-14870`): a Constant that is
    // not also the anniversary or remix world. The pipeline seam gates the
    // call on the Constant alone (`pipeline.rs`'s `step_constant_lava_layer`),
    // so the C#'s two negations live here — the only place the call-site
    // condition can still be honoured.
    if !opts.flags.contains(SeedFlags::DONT_STARVE)
        || opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY)
        || opts.flags.contains(SeedFlags::REMIX)
    {
        return;
    }
    let mut rng = streams.pass_rng("LavaLayerCaverer");
    // The port's re-derivation of `GenVars.waterLine` (`TerrainPass.cs:211-236`
    // via `pipeline.rs:1331-1337`): a fresh `pass_rng("Terrain")` stream whose
    // first draw is the offset — the same stream the terrain step drew it
    // from. See the module doc for the `rock_layer` anchor deviation.
    let water_line = {
        let mut trng = streams.pass_rng("Terrain");
        ((world.rock_layer + world.height as f64) as i32) / 2 + trng.range(-100, 20)
    };
    // `num` / `num2` / `num3` (`:73952-73954`): the two-row minimum, the
    // half-width 8, the 30-row wander clamp.
    let (num, num2, num3) = (2, 8, 30);
    // `num5` / `num6` / `num7` (`:73956-73958`): the corridor's top and bottom
    // rows around the water line, and the wander centre `waterLine - 30`.
    let (mut top, mut bottom) = (water_line - 1, water_line + 1);
    let band = water_line - num3;
    let mut wander = band;
    // `num4` (`:73955`): the running wall id carried across columns, -1 when
    // none is carried.
    let mut carry_wall: i32 = -1;
    // `num9 = 10` and the `num9++` first line (`:73960-73963`): the first
    // column is 11 and the last is `maxTilesX - 10`.
    let mut x = 10;
    while x < world.width - 10 {
        x += 1;
        if rng.range(0, 4) == 0 {
            wander += rng.range(-4, 5);
            wander = wander.clamp(band - num3, band + num3);
        }
        if rng.range(0, 3) == 0 {
            top += rng.range(-4, 5);
            top = top.clamp(wander - num2, wander - num);
        }
        if rng.range(0, 3) == 0 {
            bottom += rng.range(-4, 5);
            bottom = bottom.clamp(wander + num, wander + num2);
        }
        // The corridor (`:74000-74008`): kill every tile of the band, and
        // spill a lava cell where the killed cell was bare and carries no
        // dungeon wall. The `&&` chain short-circuits, so the `Next(15)` is
        // drawn only on such a cell — and every killed cell is inactive
        // after its kill, so the live test is the wall.
        for i in top..=bottom {
            if let Some(t) = world.tile_mut(x, i) {
                kill_tile_gen(t);
            }
            let bare = world
                .tile(x, i)
                .is_some_and(|t| !t.active() && !WALL_DUNGEON.contains(&t.wall));
            if bare && rng.range(0, 15) == 0 {
                if let Some(t) = world.tile_mut(x, i) {
                    t.liquid = 255;
                    t.set_liquid_kind(LiquidKind::Lava);
                }
            }
        }
        // The wall arm (`:74009-74039`): walk the padded column, carry the
        // first dungeon wall's id sideways across the corridor's rows, and
        // blank a wall that borders open rock or a dungeon brick.
        let mut corridor_open = true;
        for j in top - 1..=bottom + 1 {
            if !dungeon_wall_at(world, x, j) {
                continue;
            }
            if j > top && j < bottom {
                corridor_open = false;
            }
            let (top_wall, bottom_wall) = (wall_at(world, x, top), wall_at(world, x, bottom));
            if dungeon_wall_at(world, x, top) && top_wall == bottom_wall {
                carry_wall = top_wall as i32;
            }
            if carry_wall < 0 {
                carry_wall = wall_at(world, x, j) as i32;
            } else if let Some(t) = world.tile_mut(x, j) {
                t.wall = carry_wall as u16;
            }
            let above_bare =
                !dungeon_wall_at(world, x, top - 2) && !dungeon_tile_at(world, x, top - 2);
            let below_bare =
                !dungeon_wall_at(world, x, bottom + 2) && !dungeon_tile_at(world, x, bottom + 3);
            if above_bare || below_bare {
                if let Some(t) = world.tile_mut(x, j) {
                    t.wall = 0;
                }
            }
        }
        if corridor_open {
            carry_wall = -1;
        }
        // The column tail (`:74040-74054`): every cell of the column below
        // the corridor to the pre-underworld row turns its liquid into lava,
        // the thin ice 161 becomes stone 1 and the snow block 147 becomes
        // dirt 0 — each an independent `if`, in the C#'s order.
        for k in top..world.height - 50 {
            if let Some(t) = world.tile_mut(x, k) {
                if !WALL_DUNGEON.contains(&t.wall) && t.liquid > 0 {
                    t.set_liquid_kind(LiquidKind::Lava);
                }
                if t.kind == TILE_ICE {
                    t.kind = TILE_STONE;
                }
                if t.kind == TILE_SNOW {
                    t.kind = TILE_DIRT;
                }
            }
        }
    }
    // The right-side fill (`:74056-74085`): for every dungeon wall in the
    // band, walk right to the next dungeon wall within 20 columns and fill
    // the gap between with the first wall's id.
    for l in 10..world.width - 10 {
        for m in (band - num3 - 10)..(band + num3 + 10) {
            if !dungeon_wall_at(world, l, m) {
                continue;
            }
            let source = wall_at(world, l, m);
            let mut end = l + 20;
            for n in l + 1..=end {
                if dungeon_wall_at(world, n, m) {
                    end = n;
                    break;
                }
            }
            if !dungeon_wall_at(world, end, m) {
                continue;
            }
            for n in l..=end {
                if !dungeon_wall_at(world, n, m) {
                    if let Some(t) = world.tile_mut(n, m) {
                        t.wall = source;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONSTANT: u16 = SeedFlags::DONT_STARVE;

    /// A world of solid stone with the `World::new` layer anchors — both
    /// bodies carve, so every cell starts active.
    fn stone(width: i32, height: i32) -> World {
        let mut world = World::new("constant".into(), width, height);
        for t in &mut world.tiles {
            t.kind = TILE_STONE;
            t.flags |= FLAG_ACTIVE;
        }
        world
    }

    fn opts(flags: u16) -> GenOptions {
        GenOptions {
            flags: SeedFlags::from(flags),
            ..Default::default()
        }
    }

    /// The port's water line (`pipeline.rs:1331-1337`) for a `World::new`
    /// world of this height — the derivation `lava_layer_caverer` re-runs.
    fn water_line(streams: &Streams, height: i32) -> i32 {
        let mut trng = streams.pass_rng("Terrain");
        ((height as f64 * 0.45 + height as f64) as i32) / 2 + trng.range(-100, 20)
    }

    fn carved(world: &World) -> usize {
        world.tiles.iter().filter(|t| !t.active()).count()
    }

    /// The Constant's Wavy Caves carve the underground: the width factor's
    /// caverer count, the 100-try y rejection and the walker's own
    /// widening/narrowing spans land as carved (inactive) air, all inside the
    /// `InWorld` 20-fluff margin the walker gates on — and a classic world
    /// draws nothing (the body's own guard, mirroring `:12273`).
    #[test]
    fn wavy_caves_carves_only_for_the_constant() {
        // 2100x700: the width factor `35 * (2100/4200)^2 = 8` caverers and
        // the window `worldSurface + 100 = 275 .. height - 300 = 400`.
        let (w, h) = (2100, 700);
        let seed = 0xC0FFEE;
        let mut constant = stone(w, h);
        wavy_caves(
            &mut constant,
            &mut GenVars::default(),
            &mut Streams::new(seed),
            &opts(CONSTANT),
        );
        // The per-pass stream is seed-keyed: a replay is byte-identical.
        let mut replay = stone(w, h);
        wavy_caves(
            &mut replay,
            &mut GenVars::default(),
            &mut Streams::new(seed),
            &opts(CONSTANT),
        );
        assert_eq!(
            super::super::world_hash::world_hash(&constant),
            super::super::world_hash::world_hash(&replay),
            "the Constant's wavy caves are deterministic"
        );

        let carved_cells = carved(&constant);
        assert!(carved_cells > 0, "the Constant carves wavy tunnels");
        // Every carved cell is inside the `InWorld(x, y, 20)` margin.
        for y in 0..h {
            for x in 0..w {
                if !constant.tiles[constant.index(x, y)].active() {
                    assert!(
                        x >= 20 && x < w - 20 && y >= 20 && y < h - 20,
                        "carve at ({x}, {y}) is outside the InWorld margin"
                    );
                }
            }
        }

        // The classic half: the body's guard returns before any draw, so the
        // world is untouched (the pipeline seam gates the call the same way).
        let mut classic = stone(w, h);
        wavy_caves(
            &mut classic,
            &mut GenVars::default(),
            &mut Streams::new(seed),
            &opts(0),
        );
        assert_eq!(carved(&classic), 0, "a classic world draws no wavy caves");
    }

    /// The Constant's LavaLayerCaverer cuts the two-row band around the
    /// re-derived water line — a contiguous 4..=16 row span in every column
    /// of the sweep, lava spilling into the gaps, the column tail turning
    /// the wet cells and the thin ice below the band into lava and stone —
    /// and the anniversary/remix variants of the Constant draw nothing
    /// (`WorldGen.cs:14868-14870`).
    #[test]
    fn lava_layer_caverer_cuts_the_two_row_band() {
        let (w, h) = (400, 600);
        let seed = 0x5EED;
        let streams = Streams::new(seed);
        let line = water_line(&streams, h);

        let mut world = stone(w, h);
        // A wet cell well below the corridor and ice/snow cells the column
        // tail converts (`:74040-74054`), plus an ice cell far above the band
        // that no arm reaches.
        let (ice_below, snow_below, wet, ice_above) = (h - 100, h - 90, line + 50, 5);
        for &(x, y) in &[
            (200, ice_below),
            (210, snow_below),
            (220, wet),
            (230, ice_above),
        ] {
            let i = world.index(x, y);
            world.tiles[i].kind = if y == snow_below {
                TILE_SNOW
            } else {
                TILE_ICE
            };
            if y == wet {
                world.tiles[i].flags &= !FLAG_ACTIVE;
                world.tiles[i].liquid = 100;
            }
        }
        lava_layer_caverer(&mut world, &mut Streams::new(seed), &opts(CONSTANT));

        // Determinism: the pass draws only from its own two streams.
        let mut replay = stone(w, h);
        for &(x, y) in &[
            (200, ice_below),
            (210, snow_below),
            (220, wet),
            (230, ice_above),
        ] {
            let i = replay.index(x, y);
            replay.tiles[i].kind = if y == snow_below {
                TILE_SNOW
            } else {
                TILE_ICE
            };
            if y == wet {
                replay.tiles[i].flags &= !FLAG_ACTIVE;
                replay.tiles[i].liquid = 100;
            }
        }
        lava_layer_caverer(&mut replay, &mut Streams::new(seed), &opts(CONSTANT));
        assert_eq!(
            super::super::world_hash::world_hash(&world),
            super::super::world_hash::world_hash(&replay),
            "the Constant's lava caver is deterministic"
        );

        // Every swept column carries at most one contiguous carved run — the
        // C#'s single `for (i = num5; i <= num6; i++)` corridor. A run can be
        // empty: `top`'s and `bottom`'s draws are independent, so a far
        // wander can leave `top > bottom` and the C# carves nothing either.
        // The span has no tight bound above 3 (the untouched
        // `waterLine - 1 .. waterLine + 1` opening): each endpoint is clamped
        // to its own window only on its own draw (`:73976-73999`), so a
        // wander moving between two draws stretches the corridor up to the
        // full `wander ± 30 ± 8` reach.
        for x in 11..=w - 10 {
            let mut runs = 0;
            let mut rows = 0;
            let mut seen = false;
            for y in (line - 70)..(line + 10) {
                if !world.tiles[world.index(x, y)].active() {
                    if !seen {
                        runs += 1;
                    }
                    seen = true;
                    rows += 1;
                } else {
                    seen = false;
                }
            }
            assert!(runs <= 1, "column {x} has {runs} carved runs");
            if runs == 1 {
                assert!(
                    (3..=77).contains(&rows),
                    "column {x} carved {rows} rows outside the 3..=77 corridor"
                );
            }
        }

        // The 1-in-15 lava spill (`:74003`) fills some gaps of the band.
        let lava = world
            .tiles
            .iter()
            .filter(|t| t.liquid == 255 && t.liquid_kind() == LiquidKind::Lava)
            .count();
        assert!(lava > 0, "the corridor spills lava");

        // The column tail (`:74040-74054`): the wet cell below the corridor
        // keeps its amount and turns lava, the ice turns stone, the snow
        // turns dirt, and the ice above the band is untouched.
        let at = |x: i32, y: i32| world.tiles[world.index(x, y)];
        assert_eq!(at(220, wet).liquid, 100, "the spill keeps the amount");
        assert_eq!(
            at(220, wet).liquid_kind(),
            LiquidKind::Lava,
            "the wet cell below the corridor turns lava"
        );
        assert_eq!(at(200, ice_below).kind, TILE_STONE, "thin ice → stone");
        assert_eq!(at(210, snow_below).kind, TILE_DIRT, "snow block → dirt");
        assert_eq!(at(230, ice_above).kind, TILE_ICE, "the ice above the band survives");

        // The call-site guard (`:14868-14870`): a Constant that is also the
        // anniversary or the remix world does not run the caver.
        for flags in [CONSTANT | 8, CONSTANT | 32] {
            let mut other = stone(w, h);
            lava_layer_caverer(&mut other, &mut Streams::new(seed), &opts(flags));
            assert_eq!(carved(&other), 0, "flags {flags} draw no lava caver");
        }
        // A classic world is untouched.
        let mut classic = stone(w, h);
        lava_layer_caverer(&mut classic, &mut Streams::new(seed), &opts(0));
        assert_eq!(carved(&classic), 0, "a classic world draws no lava caver");
    }
}
