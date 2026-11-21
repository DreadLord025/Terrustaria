//! `WorldGen.PlaceJunglePlant` (`WorldGen.cs:47779-47944`) — the full port,
//! shared by the jungle-plants worldgen passes and the hardmode growth tick
//! (the life fruit 236, the Plantera bulb 238, the detritus 233, the replica
//! 702). W4-c31 jungle residuals.
//!
//! The C# has two shapes: the 2×2 branch (`styleY > 0 || type == 236/238/
//! 702`, `:47781-47864`) and the 3×2 branch (`:47865-47943`). Both scan the
//! overlay box for clear cells (the plant set {61, 703, 62, 655, 69, 74}
//! plus pile 185 at frameY 0 is tolerated; the 236/702/238 types also
//! tolerate a 233 detritus occupant), require solid jungle grass (60) under
//! every column, `KillTile` the occupants, then write the frames.
//!
//! Deviations (the `biome_detail.rs::place_jungle_plant_detritus` precedent):
//! `KillTile` collapses into the overwrite (the box is fully rewritten on
//! success — the C#'s `noItem: true` kill drops nothing and the cascade's
//! support checks cannot pop a neighbour the box does not touch);
//! `CopyPaintAndCoating` is a no-op during generation (paint is 0) and the
//! growth tick carries no paint source here. The `type == 98` liquid veto
//! and the 95/126 row shift are dead for the kinds this port places
//! (233/236/238/702) — carried for the shape.

use crate::tile::FLAG_ACTIVE;
use crate::world::World;

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — the growth module's
/// read: out-of-bounds reads solid (the C# `null → true`), else active +
/// `tileSolid` + not `tileSolidTop` + flat + not actuated.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// The overlay-cell tolerance (`:47802`/`:47880`): an active cell refuses
/// unless it is in the plant set, a pile 185 at frameY 0, or (for the
/// 236/702/238 types) a 233 detritus.
fn overlay_clear(t: &crate::tile::Tile, kind: u16) -> bool {
    if !t.active() {
        return true;
    }
    if matches!(t.kind, 61 | 703 | 62 | 655 | 69 | 74) {
        return true;
    }
    if matches!(kind, 236 | 702 | 238) && t.kind == 233 {
        return true;
    }
    if t.kind == 185 && t.frame_y == 0 {
        return true;
    }
    false
}

/// The frame write over one box cell — active + kind + frames, everything
/// else preserved (the C# mutates the same fields).
fn write(world: &mut World, cx: i32, cy: i32, kind: u16, fx: i16, fy: i16) {
    if let Some(t) = world.tile_mut(cx, cy) {
        t.kind = kind;
        t.frame_x = fx;
        t.frame_y = fy;
        t.flags |= FLAG_ACTIVE;
    }
}

/// `WorldGen.PlaceJunglePlant(X2, Y2, type, styleX, styleY, inheritPaint)`
/// (`WorldGen.cs:47779-47944`). Returns whether the anchor cell ended up
/// carrying `kind` (the C# callers read `Main.tile[i, j].type == num`).
pub fn place_jungle_plant(
    world: &mut World,
    x: i32,
    y: i32,
    kind: u16,
    style_x: i32,
    style_y: i32,
) -> bool {
    if style_y > 0 || kind == 236 || kind == 238 || kind == 702 {
        // The 2×2 branch (`:47781-47864`); `num = Y2` (the 95/126 shift is
        // dead for the kinds this port places).
        let num = y;
        if x < 5 || x > world.width - 5 || num < 5 || num > world.height - 5 {
            return false;
        }
        // The overlay + ground scan (`:47793-47819`): 2 columns.
        for cx in (x - 1)..(x + 1) {
            for cy in (num - 1)..(num + 1) {
                if let Some(t) = world.tile(cx, cy) {
                    if !overlay_clear(t, kind) {
                        return false;
                    }
                }
            }
            if !solid_tile(world, cx, num + 1)
                || !world.tile(cx, num + 1).is_some_and(|t| t.kind == 60)
            {
                return false;
            }
        }
        // The KillTile collapse (`:47824-47833`) — the box is rewritten.
        let fx = (36 * style_x) as i16;
        // `num3 = 0` for 236/238/702 (`:47836-47839`), else 36.
        let fy = if kind == 236 || kind == 238 || kind == 702 { 0 } else { 36 };
        write(world, x - 1, num - 1, kind, fx, fy);
        write(world, x, num - 1, kind, fx + 18, fy);
        write(world, x - 1, num, kind, fx, fy + 18);
        write(world, x, num, kind, fx + 18, fy + 18);
    } else {
        // The 3×2 branch (`:47865-47943`).
        if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
            return false;
        }
        for cx in (x - 1)..(x + 2) {
            for cy in (y - 1)..(y + 1) {
                if let Some(t) = world.tile(cx, cy) {
                    if !overlay_clear(t, kind) {
                        return false;
                    }
                }
            }
            if !solid_tile(world, cx, y + 1)
                || !world.tile(cx, y + 1).is_some_and(|t| t.kind == 60)
            {
                return false;
            }
        }
        let fx = (54 * style_x) as i16;
        write(world, x - 1, y - 1, kind, fx, 0);
        write(world, x, y - 1, kind, fx + 18, 0);
        write(world, x + 1, y - 1, kind, fx + 36, 0);
        write(world, x - 1, y, kind, fx, 18);
        write(world, x, y, kind, fx + 18, 18);
        write(world, x + 1, y, kind, fx + 36, 18);
    }
    world.tile(x, y).is_some_and(|t| t.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    /// A jungle-grass floor row at `y+1` with air above — the placement
    /// ground the scan requires.
    fn grass_floor(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("jp".into(), width, height);
        for x in 0..width {
            let i = w.index(x, floor_y);
            w.tiles[i] = Tile { kind: 60, flags: FLAG_ACTIVE, ..Default::default() };
        }
        w
    }

    /// The 2×2 life-fruit write: four cells at `36*styleX`, frameY 0/18.
    #[test]
    fn the_life_fruit_places_the_2x2_frames() {
        let mut w = grass_floor(64, 64, 31);
        assert!(place_jungle_plant(&mut w, 30, 30, 236, 2, 0));
        for (dx, dy, fx, fy) in [(-1i32, -1i32, 72i16, 0i16), (0, -1, 90, 0), (-1, 0, 72, 18), (0, 0, 90, 18)] {
            let t = w.tile(30 + dx, 30 + dy).unwrap();
            assert!(t.active() && t.kind == 236, "the fruit cell ({dx},{dy})");
            assert_eq!((t.frame_x, t.frame_y), (fx, fy));
        }
    }

    /// The bulb 238 places at styleX 0 (frameX 0/18, frameY 0/18).
    #[test]
    fn the_bulb_places_at_style_zero() {
        let mut w = grass_floor(64, 64, 31);
        assert!(place_jungle_plant(&mut w, 30, 30, 238, 0, 0));
        let t = w.tile(29, 29).unwrap();
        assert!(t.active() && t.kind == 238);
        assert_eq!((t.frame_x, t.frame_y), (0, 0));
    }

    /// The 3×2 detritus write at styleY 0: six cells at `54*styleX`.
    #[test]
    fn the_detritus_places_the_3x2_frames() {
        let mut w = grass_floor(64, 64, 31);
        assert!(place_jungle_plant(&mut w, 30, 30, 233, 1, 0));
        let t = w.tile(31, 30).unwrap();
        assert!(t.active() && t.kind == 233);
        assert_eq!((t.frame_x, t.frame_y), (54 + 36, 18));
    }

    /// A non-grass floor refuses the placement.
    #[test]
    fn a_non_grass_floor_refuses() {
        let mut w = World::new("jp".into(), 64, 64);
        for x in 0..64 {
            let i = w.index(x, 31);
            w.tiles[i] = Tile { kind: 59, flags: FLAG_ACTIVE, ..Default::default() };
        }
        assert!(!place_jungle_plant(&mut w, 30, 30, 236, 0, 0));
        assert!(w.tiles.iter().all(|t| t.kind != 236));
    }

    /// A solid occupant in the overlay box refuses (the plant-set tolerance
    /// does not cover stone).
    #[test]
    fn a_solid_occupant_refuses() {
        let mut w = grass_floor(64, 64, 31);
        let i = w.index(30, 30);
        w.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        assert!(!place_jungle_plant(&mut w, 30, 30, 236, 0, 0));
    }

    /// The a == b determinism witness: two identical placements write
    /// identical tiles (the fn is RNG-free).
    #[test]
    fn the_placement_is_reproducible() {
        let run = || {
            let mut w = grass_floor(64, 64, 31);
            place_jungle_plant(&mut w, 30, 30, 236, 1, 0);
            w
        };
        assert_eq!(run().tiles, run().tiles, "a == b — the full tile state");
    }
}
