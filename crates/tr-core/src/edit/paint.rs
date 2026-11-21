//! The paint/coating tile writes — `WorldGen.paintTile` / `paintWall` /
//! `paintCoatTile` / `paintCoatWall` (`WorldGen.cs:44110-44252`), the world
//! halves only. The dust effects (`paintEffect` :44064-44085 /
//! `paintCoatEffect` :44087-44108) and the net 63/64 broadcasts are app-side
//! (`world_sim/editing/auto_paint.rs`); the core returns the OLD colour so
//! the caller can spawn the effect (`paintEffect` reads no tile state, so
//! the write-then-effect order is observation-identical to the C#'s
//! effect-then-write for tiles; walls keep the C# write→net→effect order
//! app-side).

use crate::tile::FLAG_FULLBRIGHT;
use crate::world::World;

/// `WorldGen.paintTile` (`WorldGen.cs:44110-44132`): null/inactive refuses,
/// the same colour refuses, else the paint byte is written. `Some(old)` =
/// the write landed (the caller spawns `paintEffect(x, y, color, old)` and
/// the net 63 record); `None` = refused.
pub fn paint_tile(world: &mut World, x: i32, y: i32, color: u8) -> Option<u8> {
    let t = world.tile(x, y)?;
    if !t.active() {
        return None;
    }
    let old = t.paint();
    if old == color {
        return None;
    }
    world.tile_mut(x, y)?.set_paint(color);
    Some(old)
}

/// `WorldGen.paintWall` (`WorldGen.cs:44230-44252`): a wall-less cell
/// refuses, the same colour refuses, else the wall paint byte is written.
/// The C# broadcasts BEFORE the effect here (the tile twin effects first) —
/// the caller keeps that order.
pub fn paint_wall(world: &mut World, x: i32, y: i32, color: u8) -> Option<u8> {
    let t = world.tile(x, y)?;
    if t.wall == 0 {
        return None;
    }
    let old = world.wall_paint(x, y);
    if old == color {
        return None;
    }
    world.set_wall_paint(x, y, color);
    Some(old)
}

/// `WorldGen.paintCoatTile` (`WorldGen.cs:44134-44180`): the Glow (1) /
/// Echo (2) coating bits, the scraper clear (0). The 0/2 arms re-frame
/// through `SquareTileFrame(x, y, resetFrame: false)` (`:44153`/:44168) —
/// the port's `actuation_square_frame` (the wiring callers' seam); the
/// object drops the support checks produce are discarded like the wiring
/// callers' (the coating flip pops no furniture in practice — the tile
/// stays active and solid).
pub fn paint_coat_tile(world: &mut World, x: i32, y: i32, id: u8) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    match id {
        0 => {
            if !t.fullbright_block() && !world.invisible_block(x, y) {
                return false;
            }
            let i = world.index(x, y);
            world.tiles[i].flags &= !FLAG_FULLBRIGHT;
            world.set_invisible_block(x, y, false);
            let _ = super::actuation_square_frame(world, x, y);
        }
        1 => {
            if t.fullbright_block() {
                return false;
            }
            let i = world.index(x, y);
            world.tiles[i].flags |= FLAG_FULLBRIGHT;
        }
        2 => {
            if world.invisible_block(x, y) {
                return false;
            }
            world.set_invisible_block(x, y, true);
            let _ = super::actuation_square_frame(world, x, y);
        }
        _ => return false,
    }
    true
}

/// `WorldGen.paintCoatWall` (`WorldGen.cs:44182-44228`): the wall twin.
/// The 0/2 arms' `SquareWallFrame(x, y, resetFrame: false)`
/// (`:44201`/:44216) rides the port's `reframe_area` radius-1 reset-false
/// superset (the `spread.rs:427` SquareWallFrame convention — the tile
/// frames it also touches re-frame to their stored variants, a no-op on
/// unchanged cells).
pub fn paint_coat_wall(world: &mut World, x: i32, y: i32, id: u8) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.wall == 0 {
        return false;
    }
    match id {
        0 => {
            if !world.fullbright_wall(x, y) && !world.invisible_wall(x, y) {
                return false;
            }
            world.set_fullbright_wall(x, y, false);
            world.set_invisible_wall(x, y, false);
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        1 => {
            if world.fullbright_wall(x, y) {
                return false;
            }
            world.set_fullbright_wall(x, y, true);
        }
        2 => {
            if world.invisible_wall(x, y) {
                return false;
            }
            world.set_invisible_wall(x, y, true);
            crate::framing::reframe_area(world, x, y, 1, false);
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    fn world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 1,
            wall: 4,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    /// `paintTile` refuses the same colour and the inactive cell, writes
    /// otherwise and reports the old byte (`WorldGen.cs:44113-44126`).
    #[test]
    fn w4_g80_paint_tile_writes_and_refuses_the_same_colour() {
        let mut w = world();
        assert_eq!(paint_tile(&mut w, 10, 19, 1), Some(0));
        assert_eq!(w.tile(10, 19).unwrap().paint(), 1);
        assert_eq!(paint_tile(&mut w, 10, 19, 1), None, "same colour refuses");
        assert_eq!(paint_tile(&mut w, 10, 19, 2), Some(1));
        let i = w.index(10, 19);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert_eq!(paint_tile(&mut w, 10, 19, 3), None, "inactive refuses");
    }

    /// `paintWall` needs a wall, refuses the same colour
    /// (`WorldGen.cs:44233-44242`).
    #[test]
    fn w4_g80_paint_wall_needs_a_wall_and_refuses_the_same_colour() {
        let mut w = world();
        assert_eq!(paint_wall(&mut w, 10, 19, 7), Some(0));
        assert_eq!(w.wall_paint(10, 19), 7);
        assert_eq!(paint_wall(&mut w, 10, 19, 7), None);
        assert_eq!(paint_wall(&mut w, 11, 19, 7), None, "wall-less refuses");
    }

    /// The coating bits: Glow sets fullbright, Echo sets invisible, the
    /// second apply refuses, the scraper (0) clears both
    /// (`WorldGen.cs:44146-44169`).
    #[test]
    fn w4_g80_the_coating_bits_set_refuse_and_clear() {
        let mut w = world();
        assert!(paint_coat_tile(&mut w, 10, 19, 1));
        assert!(w.tile(10, 19).unwrap().fullbright_block());
        assert!(!paint_coat_tile(&mut w, 10, 19, 1), "already coated");
        assert!(paint_coat_tile(&mut w, 10, 19, 2));
        assert!(w.invisible_block(10, 19));
        assert!(!paint_coat_tile(&mut w, 10, 19, 2));
        assert!(paint_coat_tile(&mut w, 10, 19, 0), "the scraper clears");
        assert!(!w.tile(10, 19).unwrap().fullbright_block());
        assert!(!w.invisible_block(10, 19));
        assert!(!paint_coat_tile(&mut w, 10, 19, 0), "nothing left to clear");
        assert!(!paint_coat_tile(&mut w, 10, 19, 3), "unknown id refuses");
    }

    /// The wall coating twin (`WorldGen.cs:44194-44217`).
    #[test]
    fn w4_g80_the_wall_coating_bits_set_refuse_and_clear() {
        let mut w = world();
        assert!(paint_coat_wall(&mut w, 10, 19, 1));
        assert!(w.fullbright_wall(10, 19));
        assert!(!paint_coat_wall(&mut w, 10, 19, 1));
        assert!(paint_coat_wall(&mut w, 10, 19, 2));
        assert!(w.invisible_wall(10, 19));
        assert!(paint_coat_wall(&mut w, 10, 19, 0));
        assert!(!w.fullbright_wall(10, 19) && !w.invisible_wall(10, 19));
        assert!(!paint_coat_wall(&mut w, 11, 19, 1), "wall-less refuses");
    }
}
