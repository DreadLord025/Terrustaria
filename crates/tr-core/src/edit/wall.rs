//! The background-wall edit family (S1-K-4 round 2): `WorldGen.KillWall` /
//! `WorldGen.PlaceWall` and the wall re-frame hook. Moved verbatim from
//! `edit.rs`; behaviour frozen (same revision bump, same `reframe_area` call).

use super::{touch, SoundCue, WALL_UNBREAKABLE};
use crate::framing;
use crate::world::World;

/// `WorldGen.KillWall_PlaySounds` (`WorldGen.cs:60423-60441`, S1-E-21
/// world-sounds wave): the four-arm wall-break cue table. The C# plays it
/// from `KillWall` (`:60385`) BEFORE the dust and the drop — and on the
/// `fail` path too (`KillWall_CheckFailure` `:60410-60421`: the dungeon
/// walls before Skeletron, wall 87 before Golem); the port's `break_wall`
/// has no fail path, so the cue rides the successful break only (ANALYSIS,
/// S1-E-21i).
///
/// The arms: the glass/stone family → the legacy id 13 (Shatter, no pitch
/// draw, `LegacySoundPlayer.cs:783-792`); the grass/jungle family → id 6
/// (Grass, one `Next(-30, 31) * 0.01`, `:706-716`); wall 357 →
/// `SoundID.NPCHit25` (`SoundID.cs:1281`, the id-3 style-25 half-volume
/// hit); everything else → id 0 (the Dig variation + pitch, `:440-452`).
pub fn kill_wall_sound(wall: u16) -> SoundCue {
    // `:60425-60428` — the Shatter arm.
    if wall == 241
        || (88..=93).contains(&wall)
        || matches!(
            wall,
            21 | 186 | 136 | 137 | 168 | 169 | 172 | 226 | 227 | 242 | 243 | 347 | 365
        )
    {
        return (13, 1);
    }
    // `:60429-60432` — the Grass arm.
    if (63..=70).contains(&wall) || matches!(wall, 264 | 268 | 265) {
        return (6, 1);
    }
    // `:60433-60436` — the NPCHit25 arm.
    if wall == 357 {
        return (3, 25);
    }
    // `:60437-60440` — the default Dig arm.
    (0, 1)
}

/// Removes the background wall at `(x, y)`. Returns the wall id.
pub fn break_wall(world: &mut World, x: i32, y: i32) -> Option<u16> {
    let t = *world.tile(x, y)?;
    if t.wall == 0 || t.wall == WALL_UNBREAKABLE {
        return None;
    }
    let wall = t.wall;
    let i = world.index(x, y);
    world.tiles[i].wall = 0;
    world.tiles[i].set_wall_frame(0, 0);
    after_wall_edit(world, x, y);
    Some(wall)
}

/// Places a background wall into a cell that has none. The caller checks reach and
/// support; a wall may sit behind an active tile.
pub fn place_wall(world: &mut World, x: i32, y: i32, wall: u16) -> bool {
    let Some(t) = world.tile(x, y).copied() else {
        return false;
    };
    if t.wall != 0 || wall == 0 {
        return false;
    }
    let i = world.index(x, y);
    world.tiles[i].wall = wall;
    world.tiles[i].set_wall_frame(0, 0);
    after_wall_edit(world, x, y);
    true
}

fn after_wall_edit(world: &mut World, x: i32, y: i32) {
    touch(world, x, y);
    framing::reframe_area(world, x, y, 2, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::tests::ground_world;

    #[test]
    fn walls_place_and_break_and_reframe_neighbours() {
        let mut w = ground_world();
        assert!(place_wall(&mut w, 10, 15, 2));
        assert_eq!(w.tile(10, 15).unwrap().wall, 2);
        assert!(!place_wall(&mut w, 10, 15, 1), "already walled");
        assert!(place_wall(&mut w, 11, 15, 2));
        // The shared edge reframes to a connecting cell.
        assert_ne!(w.tile(10, 15).unwrap().wall_col(), 0);
        assert_eq!(break_wall(&mut w, 10, 15), Some(2));
        assert_eq!(w.tile(10, 15).unwrap().wall, 0);
        assert_eq!(break_wall(&mut w, 10, 15), None);
    }

    /// S1-E-21i — the `KillWall_PlaySounds` table
    /// (`WorldGen.cs:60423-60441`): every arm's membership, verbatim.
    #[test]
    fn s1_e_21i_kill_wall_sound_table_arms() {
        // Arm 1 (`:60425`): Shatter (13) — 241, 88..93, and the 13 singles.
        for wall in [
            241, 88, 89, 90, 91, 92, 93, 21, 186, 136, 137, 168, 169, 172, 226, 227, 242,
            243, 347, 365,
        ] {
            assert_eq!(kill_wall_sound(wall), (13, 1), "wall {wall} shatters");
        }
        // Arm 2 (`:60429`): Grass (6) — 63..70 plus 264/268/265.
        for wall in [63, 64, 65, 66, 67, 68, 69, 70, 264, 268, 265] {
            assert_eq!(kill_wall_sound(wall), (6, 1), "wall {wall} is grass");
        }
        // Arm 3 (`:60433`): wall 357 → SoundID.NPCHit25 (`SoundID.cs:1281`).
        assert_eq!(kill_wall_sound(357), (3, 25));
        // Arm 4 (`:60437`): everything else → the Dig 0.
        for wall in [1, 2, 4, 20, 22, 62, 71, 87, 94, 185, 240, 244, 263, 266, 356, 358] {
            assert_eq!(kill_wall_sound(wall), (0, 1), "wall {wall} digs");
        }
        // The arm boundaries: 87 and 94 sit OUTSIDE the 88..93 window,
        // 62/71 outside 63..70, 240/244 outside the singles.
        assert_eq!(kill_wall_sound(87), (0, 1));
        assert_eq!(kill_wall_sound(94), (0, 1));
    }
}
