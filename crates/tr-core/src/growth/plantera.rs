//! The Plantera bulb event placement — `WorldGen.
//! GeneratePlanteraBulbOnAllMechsDefeated` (`WorldGen.cs:74127-74186`) +
//! `AttemptToGeneratePlanteraBulbAt` (`:74188-74267`). W4-c31 jungle
//! residuals (the placement / worldgen side; the draw side is W4-g84).
//!
//! The C# fires it from `NPC.OnGameEventClearedForTheFirstTime` case
//! 16/17/18 (`NPC.cs:78780-78788`) when the third mech boss falls
//! (`Main.netMode != 1 && Main.hardMode && downedMechBoss1/2/3`). It scans
//! the jungle half of the underground for a jungle-grass (60) — or, in the
//! last 500 attempts, a mud (59) — ceiling and forces a 238 bulb onto it,
//! carving a 4×4 grass ring when the spot is too crowded.
//!
//! Deviations: `Main.dungeonX` reads the port's `World.dungeon_x`;
//! `Main.UnderworldLayer` = `maxTilesY - 200` (`Main.cs:2790`). The
//! `NetMessage.SendTileSquare` sides are MP scope (the host path). The
//! `SquareTileFrame` re-frames ride the port's `square_tile_frame_gen`.

use crate::tile::FLAG_ACTIVE;
use crate::world::World;
use crate::UnifiedRandom;

use super::jungle_plants::place_jungle_plant;

/// `WorldGen.IsAContainer` (`WorldGen.cs:48822-48829`): the basic dresser /
/// chest / fake-chest sets plus the two hardcoded kinds 470/475.
fn is_a_container(t: &crate::tile::Tile) -> bool {
    crate::tiledata::SET_BASIC_DRESSER.has(t.kind)
        || t.kind == 470
        || t.kind == 475
        || crate::tiledata::SET_BASIC_CHEST.has(t.kind)
        || crate::tiledata::SET_BASIC_CHEST_FAKE.has(t.kind)
}

/// The force-bulb veto set (`:74206`): a container, or one of the listed
/// furniture / painting / boulder kinds, drops the score to −1.
fn bulb_veto(t: &crate::tile::Tile) -> bool {
    is_a_container(t)
        || matches!(
            t.kind,
            226 | 237 | 10 | 11 | 26 | 16 | 15 | 14 | 104 | 158 | 105 | 531 | 349 | 137
                | 443 | 12 | 314
        )
        || crate::tiledata::SET_PAINTINGS.has(t.kind)
        || crate::tiledata::SET_BOULDERS.has(t.kind)
}

/// `WorldGen.AttemptToGeneratePlanteraBulbAt(i, j, forceBulb)`
/// (`WorldGen.cs:74188-74267`). The force arm scores the 4×4 box (mud/grass
/// +1, other active −1, a veto kind → −1 abort), needs ≥ 12, then clears the
/// box and rings it with grass before placing the bulb; the plain arm just
/// places. Returns whether a 238 landed at the anchor.
fn attempt_bulb_at(world: &mut World, i: i32, j: i32, force_bulb: bool) -> bool {
    if force_bulb {
        // The 4×4 score (`:74192-74220`): k/l in [-2, 2).
        let mut num = 0;
        'outer: for k in -2..2 {
            for l in -2..2 {
                let Some(t) = world.tile(i + k, j + l) else {
                    continue;
                };
                if t.kind == 59 || t.kind == 60 {
                    num += 1;
                } else if t.active() {
                    num -= 1;
                }
                if bulb_veto(t) {
                    num = -1;
                    break;
                }
            }
            if num <= -1 {
                break 'outer;
            }
        }
        if num < 12 {
            return false;
        }
        // The clear + grass ring (`:74221-74237`): the border cells (m ==
        // -2/1 or n == -2/1) become active grass 60, the inner 2×2 clears.
        for m in -2..2 {
            for n in -2..2 {
                let (cx, cy) = (i + m, j + n);
                let border = m == -2 || m == 1 || n == -2 || n == 1;
                if let Some(t) = world.tile_mut(cx, cy) {
                    // `Tile.ClearTile` (`Tile.cs:123-128`): slope/half off,
                    // active + inactive off (the kind/wall/frames survive).
                    t.set_slope(0);
                    t.flags &= !(FLAG_ACTIVE | crate::tile::FLAG_INACTIVE | crate::tile::FLAG_HALF_BRICK);
                    if border {
                        t.kind = 60;
                        t.flags |= FLAG_ACTIVE;
                    }
                }
                crate::edit::square_tile_frame_gen(world, cx, cy);
            }
        }
        place_jungle_plant(world, i, j, 238, 0, 0);
        if world.tile(i, j).is_some_and(|t| t.kind == 238) {
            crate::edit::square_tile_frame_gen(world, i, j);
            crate::edit::square_tile_frame_gen(world, i + 2, j);
            crate::edit::square_tile_frame_gen(world, i - 1, j);
            return true;
        }
    } else {
        place_jungle_plant(world, i, j, 238, 0, 0);
        if world.tile(i, j).is_some_and(|t| t.kind == 238) {
            crate::edit::square_tile_frame_gen(world, i, j);
            crate::edit::square_tile_frame_gen(world, i + 2, j);
            crate::edit::square_tile_frame_gen(world, i - 1, j);
            return true;
        }
    }
    false
}

/// `WorldGen.GeneratePlanteraBulbOnAllMechsDefeated`
/// (`WorldGen.cs:74127-74186`): 2500 attempts over the jungle-half
/// underground rectangle; each drops a column to the first jungle-grass
/// ceiling (mud in the last 500, any tile in the last 200), then tries the
/// bulb at the cell above and its two neighbours. Returns whether a bulb
/// placed.
///
/// `dungeon_x` is `Main.dungeonX`; the rectangle's x band is the
/// jungle-opposite half (`flag = dungeonX > maxTilesX*0.5` → `[0.15, 0.35)`
/// else `[0.65, 0.85)` of the width), y from `worldSurface` to
/// `UnderworldLayer` (`maxTilesY - 200`).
pub fn generate_plantera_bulb_on_all_mechs_defeated(
    world: &mut World,
    dungeon_x: i32,
    rng: &mut UnifiedRandom,
) -> bool {
    let width = world.width;
    let height = world.height;
    let flag = dungeon_x as f64 > width as f64 * 0.5;
    let x0 = (if flag { width as f64 * 0.15 } else { width as f64 * 0.65 }) as i32;
    let x1 = (if flag { width as f64 * 0.35 } else { width as f64 * 0.85 }) as i32;
    let y0 = world.world_surface as i32;
    let underworld = height - 200; // `Main.UnderworldLayer` (`Main.cs:2790`)
    let rect_w = x1 - x0;
    let rect_h = underworld - y0;
    if rect_w <= 0 || rect_h <= 0 {
        return false;
    }
    let mut num4 = 2500;
    while num4 > 0 {
        let force_bulb = num4 < 500;
        let any_tile = num4 < 200;
        num4 -= 1;
        let mut px = x0 + rng.next_max(rect_w);
        let mut py = y0 + rng.next_max(rect_h);
        // The ceiling walk (`:74142-74168`): up to 500 steps upward, the
        // first jungle-grass (60) — or mud (59) when force_bulb — ends it.
        // `tile` starts at the drawn cell (`:74141`) — if the walk breaks on
        // the first bounds check, the post-walk gate reads that cell.
        let mut num5 = 500;
        let mut done = false;
        let (mut tile_kind, mut tile_active) = world
            .tile(px, py)
            .map(|t| (t.kind, t.active()))
            .unwrap_or((0, false));
        while !done {
            num5 -= 1;
            if num5 <= 0 {
                done = true;
            }
            py -= 1;
            if py < y0 {
                break;
            }
            if let Some(t) = world.tile(px, py) {
                tile_kind = t.kind;
                tile_active = t.active();
                if any_tile || t.active() {
                    if t.kind == 60 {
                        done = true;
                    } else if force_bulb && t.kind == 59 {
                        done = true;
                    }
                }
            }
        }
        // The post-walk gate (`:74169`): `(active | any_tile) & (kind == 60
        // || (force_bulb && kind == 59))`.
        if (tile_active | any_tile) & (tile_kind == 60 || (force_bulb && tile_kind == 59)) {
            if attempt_bulb_at(world, px, py - 1, force_bulb) {
                return true;
            }
            if attempt_bulb_at(world, px - 1, py - 1, force_bulb) {
                return true;
            }
            if attempt_bulb_at(world, px + 1, py - 1, force_bulb) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    /// A world with a jungle-grass ceiling row and a mud body below, in the
    /// bulb rectangle's x band.
    fn jungle_ceiling(width: i32, height: i32, ceiling_y: i32) -> World {
        let mut w = World::new("bulb".into(), width, height);
        w.world_surface = (ceiling_y - 50) as f64;
        for x in 0..width {
            for y in ceiling_y..height - 200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 59, flags: FLAG_ACTIVE, ..Default::default() };
            }
            let i = w.index(x, ceiling_y);
            w.tiles[i].kind = 60;
        }
        w
    }

    /// The event places a 238 bulb somewhere in the jungle half.
    #[test]
    fn the_event_places_a_bulb() {
        let mut w = jungle_ceiling(1200, 1000, 400);
        // dungeon on the right → the bulb band is the left [0.15, 0.35).
        let placed = generate_plantera_bulb_on_all_mechs_defeated(&mut w, 1000, &mut UnifiedRandom::new(42));
        assert!(placed, "the event places a bulb");
        assert!(w.tiles.iter().any(|t| t.active() && t.kind == 238), "a 238 exists");
    }

    /// The a == b determinism witness: two runs over identical worlds draw
    /// identical streams and write identical tiles.
    #[test]
    fn the_event_is_reproducible_for_a_seed() {
        let run = || {
            let mut w = jungle_ceiling(1200, 1000, 400);
            let placed = generate_plantera_bulb_on_all_mechs_defeated(&mut w, 1000, &mut UnifiedRandom::new(7));
            (w, placed)
        };
        let (a, pa) = run();
        let (b, pb) = run();
        assert_eq!(pa, pb);
        assert_eq!(a.tiles, b.tiles, "a == b — the full tile state");
    }

    /// The force-bulb arm carves a grass ring when the spot is crowded: a
    /// mud pocket with a grass ceiling forces the 4×4 ring.
    #[test]
    fn the_force_arm_rings_the_bulb_with_grass() {
        let mut w = jungle_ceiling(1200, 1000, 400);
        // Drive the force arm directly at a mud cell below the ceiling.
        let ok = attempt_bulb_at(&mut w, 300, 410, true);
        assert!(ok, "the force arm places on the mud body");
        // The border of the 4×4 box is grass 60.
        let t = w.tile(298, 408).unwrap();
        assert!(t.active() && t.kind == 60, "the ring border is grass");
        assert!(w.tile(300, 410).is_some_and(|t| t.kind == 238), "the bulb anchor");
    }

    /// A veto kind (a chest 21) inside the box aborts the force arm.
    #[test]
    fn a_container_vetoes_the_force_arm() {
        let mut w = jungle_ceiling(1200, 1000, 400);
        let i = w.index(300, 410);
        w.tiles[i] = Tile { kind: 21, flags: FLAG_ACTIVE, ..Default::default() };
        assert!(!attempt_bulb_at(&mut w, 300, 410, true), "the chest vetoes");
    }
}
