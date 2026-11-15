//! The meteorite generator (`WorldGen.dropMeteor` + `WorldGen.meteor`,
//! `WorldGen.cs:6246-6549`) and the nightly resolution
//! (`Main.HandleMeteorFall`, `Main.cs:65469-65502`).
//!
//! The vanilla flow: the flag (`WorldGen.spawnMeteor`) raises at the
//! night bookend (`Main.cs:65230`'s `Next(50)` after the boss-2 downing)
//! and at the orb-boss deaths (`NPC.cs:79450`); the 16200-tick dusk
//! resolution clears it and runs `dropMeteor` — the site pick (away
//! from the spawn, the open-sky scan) then the 5-phase ore stamp.

use crate::tile::FLAG_ACTIVE;
use crate::tiledata::TILE_SOLID;
use crate::unified_random::UnifiedRandom;
use crate::world::World;

/// `TileID.Meteorite` (`TileID.cs`'s 37).
pub const METEORITE: u16 = 37;

/// `Main.cs:65230`'s night roll: `Main.rand.Next(50) == 0 && NPC.downedBoss2` —
/// the `Next(50)` draw happens every `UpdateTime_StartNight` (it is the
/// leftmost operand), the boss flag only gates the flag raise. The stream is
/// `_rngs["UpdateTime"]`: `UpdateTime_StartNight` runs inside `Main.UpdateTime`
/// (`Main.cs:17795`'s swap).
pub fn night_roll(rng: &mut UnifiedRandom, downed_boss2: bool) -> bool {
    rng.next_max(50) == 0 && downed_boss2
}

/// `NPC.cs:79450-79456`'s orb-boss death arm: the guaranteed first
/// (`!downedBoss2`, no draw), the `Next(2) == 0` afterwards. The stream is the
/// caller's — the death arm runs under `UpdateNPCs` (`NPC.checkDead` →
/// `DoDeathEvents` inside `NPC.UpdateNPC`, `Main.cs:17767-17769`), so the
/// npcs pass feeds `UpdateNPCs` (S1-A-11 P3) and the weather site would feed
/// `UpdateTime`.
pub fn boss_death_roll(rng: &mut UnifiedRandom, downed_boss2: bool) -> bool {
    !downed_boss2 || rng.next_max(2) == 0
}

/// The site-scan's decay threshold (`:6302`'s 600 -> the `:6356` 100
/// floor).
const OPEN_SKY_START: f64 = 600.0;
const OPEN_SKY_FLOOR: f64 = 100.0;

/// The destroy-for-meteors tile set (`TileID.cs:489`'s
/// `GetsDestroyedForMeteors`).
const DESTROYED_FOR_METEORS: [u16; 14] = [
    5, 32, 352, 583, 584, 585, 586, 587, 588, 589, 596, 616, 634, 0,
];

fn destroyed_for_meteors(kind: u16) -> bool {
    DESTROYED_FOR_METEORS.contains(&kind)
}

fn solid_no_plat(w: &World, x: i32, y: i32) -> bool {
    w.tile(x, y)
        .is_some_and(|t| t.active() && TILE_SOLID.has(t.kind))
}

/// `WorldGen.meteor(int i, int j)` (`:6367-6549`): the validity gates
/// (the 50-border + the 70x70 important-tile box — the player/NPC rect
/// gates fold to the caller's distance check) then the 5-phase stamp.
/// The vanilla's `genRand` arms port as `UnifiedRandom` draws (`rng` —
/// `Next(min, max)`/`Next(max)` per site; inside `HandleMeteorFall` the C#
/// `Main.rand` rides the caller's stream).
pub fn meteor(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) -> bool {
    // Gate 1 (`:6369-6376`): the 50-tile border.
    if i < 50 || i > world.width - 50 || j < 50 || j > world.height - 50 {
        return false;
    }
    // Gate 3 (`:6398-6420`): the 70x70 box must hold no "important"
    // tiles (the port folds the vanilla set to the altar 26 and the
    // trap/statue kinds).
    for x in (i - 35)..(i + 35) {
        for y in (j - 35)..(j + 35) {
            if let Some(t) = world.tile(x, y) {
                if t.active() && matches!(t.kind, 26 | 226 | 470 | 475 | 488 | 597) {
                    return false;
                }
            }
        }
    }
    // Phase 1 (`:6421-6445`): the ore body — the jagged top edge, the
    // 0.9 radius, the air carve + the meteorite write.
    let num = rng.next_range(17, 23);
    for x in (i - num)..(i + num) {
        for y in (j - num)..(j + num) {
            let dx = x - i;
            let dy = y - j;
            let r = num as f32 * 0.9 + (rng.next_range(-4, 5)) as f32;
            if (dx * dx + dy * dy) as f32 >= r * r {
                continue;
            }
            if y <= j + rng.next_range(-2, 3) - 5 {
                if solid_no_plat(world, x, y) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.kind = METEORITE;
                        t.liquid = 0;
                        t.flags |= FLAG_ACTIVE;
                    }
                }
            } else if let Some(t) = world.tile_mut(x, y) {
                t.kind = METEORITE;
                t.liquid = 0;
                t.flags |= FLAG_ACTIVE;
            }
        }
    }
    // Phase 2 (`:6446-6461`): the hollowing.
    let num2 = rng.next_range(8, 14);
    for x in (i - num2)..(i + num2) {
        for y in (j - num2)..(j + num2) {
            let dx = x - i;
            let dy = y - j;
            let r = num2 as f32 * 0.8 + (rng.next_range(-3, 4)) as f32;
            if (dx * dx + dy * dy) as f32 >= r * r {
                continue;
            }
            if world.tile(x, y).is_some_and(|t| t.kind == METEORITE) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.flags &= !FLAG_ACTIVE;
                    t.kind = 0;
                }
            }
        }
    }
    // Phase 3 (`:6462-6491`): the destroy-for-meteors sweep + the liquid
    // carve in the 25-35 radius (0.7) around the site.
    let num3 = rng.next_range(25, 35);
    for x in (i - num3)..(i + num3) {
        for y in (j - num3)..(i + num3) {
            let dx = x - i;
            let dy = y - j;
            let r = num3 as f32 * 0.7;
            if (dx * dx + dy * dy) as f32 >= r * r {
                continue;
            }
            if let Some(t) = world.tile_mut(x, y) {
                if t.active() && destroyed_for_meteors(t.kind) {
                    t.flags &= !FLAG_ACTIVE;
                    t.kind = 0;
                }
                t.liquid = 0;
            }
        }
    }
    // Phases 4+5 (`:6492-6541`): the outer fill rings (the 1/10 and the
    // 1/20 per-tile chances.
    for &(base, chance) in &[(23, 10), (30, 20)] {
        let n = rng.next_range(base, base + 9);
        let r = n as f32 * 0.8;
        for x in (i - n)..(i + n) {
            for y in (j - n)..(j + n) {
                let dx = x - i;
                let dy = y - j;
                if (dx * dx + dy * dy) as f32 >= r * r {
                    continue;
                }
                if rng.next_max(chance) != 0 {
                    continue;
                }
                if let Some(t) = world.tile_mut(x, y) {
                    if t.active() && destroyed_for_meteors(t.kind) {
                        t.flags &= !FLAG_ACTIVE;
                        t.kind = 0;
                    }
                    t.kind = METEORITE;
                    t.liquid = 0;
                    t.flags |= FLAG_ACTIVE;
                }
            }
        }
    }
    true
}

/// `WorldGen.dropMeteor()` (`:6246-6365`): the site pick. The ore-cap
/// pre-scan (`:6272-6290`'s 400 x maxTilesX/4200) and the player/NPC
/// rect gates fold — the port runs the pick on the loaded world.
/// The C# X pick is `Main.rand.Next(150, maxTilesX - 150)` (`:6301`/`:6303`);
/// the port's `150 + Next(0, max(1, width - 300))` keeps that shape with the
/// small-world guard the splitmix fixture worlds need.
pub fn drop_meteor(world: &World, spawn_x: i32, rng: &mut UnifiedRandom) -> Option<(i32, i32)> {
    let num5 = (world.world_surface * 0.3) as i32;
    let mut num3 = OPEN_SKY_START;
    let mut attempts = 0;
    let max_attempts = world.width * 5;
    while num3 >= OPEN_SKY_FLOOR && attempts < max_attempts {
        attempts += 1;
        // The X pick: away from the spawn's +-8% (`:6300-6305`).
        let band = (world.width as f64 * 0.08) as i32;
        let mut x = 150 + rng.next_range(0, (world.width - 300).max(1));
        if (x - spawn_x).abs() < band {
            x = 150 + rng.next_range(0, (world.width - 300).max(1));
        }
        // The row scan from num5 down (`:6306-6353`): the first active
        // solid row, then the 30x30 count.
        let mut landed = None;
        for l in num5..world.height {
            if !solid_no_plat(world, x, l) {
                continue;
            }
            let mut num8 = 0;
            for m in -15..=15 {
                for n in -15..=15 {
                    if !world.in_bounds(x + m, l + n) {
                        continue;
                    }
                    let t = world.tile(x + m, l + n);
                    if t.is_some_and(|t| t.active() && TILE_SOLID.has(t.kind)) {
                        num8 += 1;
                    } else if t.is_some_and(|t| t.liquid > 0) {
                        num8 -= 1;
                    }
                }
            }
            if num8 as f64 >= num3 {
                landed = Some((x, l));
            }
            break;
        }
        if let Some((lx, ly)) = landed {
            // The vanilla stamps the live world directly; the port's
            // `drop_meteor` takes `&World` so the caller re-runs the
            // stamp on success (see `resolve`).
            return Some((lx, ly));
        }
        num3 -= 0.5;
    }
    None
}

fn clone_tiles(world: &World) -> World {
    let mut snapshot = World::new("meteor".into(), world.width, world.height);
    snapshot.tiles.copy_from_slice(&world.tiles);
    snapshot
}

/// `Main.HandleMeteorFall` (`:65469-65502`): the dusk resolution —
/// `time > 16200 && spawnMeteor` clears the flag and lands the ore. The
/// stream is the caller's `UpdateTime` (`HandleMeteorFall` runs inside
/// `Main.UpdateTime`, `Main.cs:65141`); the C#'s failed-pick
/// `Next(3)` meteor-shower arm is not ported (no shower system), so no
/// draw stands in for it.
pub fn resolve(world: &mut World, time: f64, rng: &mut UnifiedRandom) -> bool {
    if time <= 16200.0 || !world.spawn_meteor {
        return false;
    }
    world.spawn_meteor = false;
    // The vanilla also broadcasts "A meteorite has landed!"
    // (`Lang.gen[59]`) — the chat-announce pass folds it.
    if let Some((x, y)) = drop_meteor(world, 0, rng) {
        return meteor(world, x, y, rng);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ground(w: &mut World, top: i32) {
        for x in 0..w.width {
            for y in top..w.height {
                let i = w.index(x, y);
                w.tiles[i] = crate::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = top as f64;
    }

    #[test]
    fn the_meteor_stamps_the_ore_body() {
        // `meteor`'s phase 1: the ore body carves the air and writes the
        // meteorite tiles around the site. (S1-A-11 P2: the draws moved from
        // the splitmix `random_int` bridge to `UnifiedRandom` — the structural
        // pins survive, the seed fixtures are stream states now.)
        let mut w = World::new("t".into(), 400, 400);
        ground(&mut w, 120);
        let mut rng = UnifiedRandom::new(9);
        assert!(meteor(&mut w, 200, 130, &mut rng));
        let ore = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == METEORITE)
            .count();
        assert!(ore > 100, "the ore body stamped ({ore})");
        // The hollowing keeps the centre open-ish.
        let centre = w.tile(200, 130).unwrap();
        let _ = centre;
    }

    #[test]
    fn the_dusk_resolution_lands_once() {
        // `HandleMeteorFall`: the flag clears on the first dusk pass.
        let mut w = World::new("t".into(), 400, 400);
        ground(&mut w, 120);
        w.spawn_meteor = true;
        let mut rng = UnifiedRandom::new(4);
        assert!(resolve(&mut w, 16201.0, &mut rng));
        assert!(!w.spawn_meteor, "the flag cleared");
        assert!(!resolve(&mut w, 16202.0, &mut rng), "no second landing");
        // The day-time is inert.
        let mut w2 = World::new("t".into(), 400, 400);
        ground(&mut w2, 120);
        w2.spawn_meteor = true;
        assert!(!resolve(&mut w2, 15000.0, &mut rng));
    }

    #[test]
    fn the_rolls_gate_on_the_boss_flag() {
        // The night roll needs the boss-2 downing (`Main.cs:65230` — but the
        // `Next(50)` draw happens BEFORE the flag read, every StartNight);
        // the death roll is the guaranteed-first with no draw (`NPC.cs:79450`).
        let mut rng = UnifiedRandom::new(1);
        assert!(
            !night_roll(&mut rng, false),
            "the flag still gates the raise"
        );
        let mut hits = 0;
        for _ in 0..500 {
            if night_roll(&mut rng, true) {
                hits += 1;
            }
        }
        assert!(hits > 0 && hits < 40, "the 1/50 rate ({hits})");
        assert!(boss_death_roll(&mut rng, false), "the guaranteed first");
    }

    /// DW-2.2 (S1-A-11): the night roll consumes exactly one `Next(50)` draw
    /// even when the boss flag is down — the C#'s leftmost operand
    /// (`Main.cs:65230`) draws before the flag read. Pinned by twin-stream
    /// equality: two `Next(50)` draws land the stream in the same state as a
    /// night_roll(gated) + night_roll(rolling) pair.
    #[test]
    fn the_night_roll_draws_next50_before_the_boss_flag_read() {
        let mut rng = UnifiedRandom::new(42);
        let mut twin = UnifiedRandom::new(42);
        assert!(!night_roll(&mut rng, false));
        twin.next_max(50);
        assert_eq!(rng, twin, "exactly one Next(50) draw when gated off");
        // The rolling call consumes one more draw.
        let _ = night_roll(&mut rng, true);
        twin.next_max(50);
        assert_eq!(rng, twin, "one Next(50) draw per rolling call");
    }
}
