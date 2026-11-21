//! `TileDrawing.DrawMultiTileGrass` / `DrawMultiTileGrassInWind` — the
//! Plantera-bulb (238) slice (W4-g84). C# never draws a bulb cell through
//! `DrawSingleTile`: the draw-area switch registers the object's top-left
//! cell as a `TileCounterType.MultiTileGrass` special point
//! (`TileDrawing.cs:606-613` — `236/238/702`, `frameX % 36 == 0 &&
//! frameY == 0`), and `PostDrawTiles` draws the 2×2 object
//! (`DrawMultiTileGrass` `:8738-8805`, size arm `:8760-8764`;
//! `DrawMultiTileGrassInWind` `:8916-9032`) about its bottom-centre anchor.
//!
//! The port keeps the fused row-major tile pass and draws each bulb cell
//! inline with the special pass's per-cell transform — the sunflower
//! precedent (`render/tiles.rs::sunflower_sway`, the documented z-order
//! approximation, ANALYSIS "The Plantera bulb wind draw"). The animator row
//! (`Main.tileFrame[238]`, `Main.cs:18709-18718`, period 21 ticks / 4 rows)
//! and the case-238 metrics (`drawdata.rs` — top 2, `addFrY = row * 36`,
//! `TileDrawing.cs:5441-5444`) are already landed and feed the source rect
//! through the normal `resolve_tile_frame` path; this module adds the wind
//! transform that was missing.
//!
//! Scope: tile 238 only. The sibling MultiTileGrass kinds 236/702 (jungle
//! roses / Life Fruit, same 2×2 shape) and the rest of the pass's roster
//! (27 sunflower — landed per-cell; 233, 530, 651, 485…, 519 cattail) keep
//! their existing draws; porting them is the general-pass follow-up.
//!
//! Deviations (ANALYSIS "The Plantera bulb wind draw (W4-g84)"):
//! - the deferred special-point ordering is approximated inline (the
//!   sunflower precedent);
//! - the `DrawAnimatedTile_AdjustForVisionChangers` /
//!   `DrawTiles_GetLightOverride` chain (`:8976-8977`) is the d56 unwired
//!   lane — for 238 both are identity outside the unported vision potions
//!   and the Glow coating (the vine strip's recorded deviation);
//! - the `_rand.Next(4)` flag3 roll + the 238 dust-168 sparkle
//!   (`DrawTiles_EmitParticles` `:6956-6961`) are the d60 unwired lane —
//!   adding the roll here would change the `_rand` stream order;
//! - the `SideFlags` seams (`:9007-9023`) feed only the unported LitNature
//!   shader (the vine strip's deviation).

use tr_core::wind::Wind;
use tr_core::World;

/// The Plantera bulb's 2×2 MultiTileGrass size (`TileDrawing.cs:8760-8764`:
/// `case 236: case 238: case 702: sizeX = (num4 = 2)`).
const SIZE: i32 = 2;

/// One bulb cell's wind transform — the `DrawNature` call's rotation and
/// vertical offset (`TileDrawing.cs:8998-9024`). Pure world/wind state, so
/// the maths is unit-testable without a GPU (the `VineSegment` shape).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BulbSway {
    /// `vector3.Y = |windCycle| * 2 * num3` (`:8998`) — the draw position's
    /// vertical offset; the horizontal lean comes from the rotation about
    /// the object's bottom-centre anchor.
    pub dy: f32,
    /// `rotation = windCycle * num * num3` with the default-arm factor
    /// `num = 0.15` (`:8937-8939`, `:9024`).
    pub rot: f32,
    /// `origin = vector - vector2` (`:8999`, `:8982`) — the rotation pivot
    /// relative to the cell sprite's top-left: the object's bottom-centre
    /// `(topLeftX*16 + sizeX*8, topLeftY*16 + 16*sizeY)` minus the cell's
    /// `(i*16, j*16 + tileTop)` (case 238's `tileTop = 2`).
    pub pivot: [f32; 2],
}

/// The wind transform of the bulb cell at `(tx, ty)` whose object's
/// top-left is reconstructed from the frame grid: `frameX = 36*style +
/// 18*col`, `frameY = 18*row` (`WorldGen.PlaceJunglePlant`,
/// `WorldGen.cs:47834-47855` — the 236/238/702 arm writes `num3 = 0`).
pub(crate) fn bulb_sway(
    wind: &Wind,
    world: &World,
    tx: i32,
    ty: i32,
    frame_x: i16,
    frame_y: i16,
    tile_top: i16,
    sunflower_wind_counter: f32,
) -> BulbSway {
    let col = (frame_x.rem_euclid(36) / 18) as i32;
    let row = (frame_y / 18) as i32;
    let top_left_x = tx - col;
    let top_left_y = ty - row;
    // `GetWindCycle(topLeftX, topLeftY, _sunflowerWindCounter)` (`:8918`) —
    // the MultiTileGrass pass rides the sunflower counter; the surface-band
    // and |wind| 0.08..0.18 gates live inside `Wind::cycle`.
    let cycle = wind.cycle(top_left_x, top_left_y, sunflower_wind_counter);
    // `flag = InAPlaceWithWind(topLeftX, topLeftY, sizeX, sizeY)` (`:8924`)
    // with the 2×2 window.
    let in_wind = wind.in_a_place_with_wind(world, top_left_x, top_left_y, SIZE, SIZE);
    // `num3 = 1 - (j - topLeftY + 1) / sizeY` (`:8964`), the 0-row guard
    // `num3 == 0 -> 0.1` (`:8965-8968`), and `!flag -> 0` (`:8969-8972`).
    let mut num3 = 1.0 - (row + 1) as f32 / SIZE as f32;
    if num3 == 0.0 {
        num3 = 0.1;
    }
    if !in_wind {
        num3 = 0.0;
    }
    BulbSway {
        dy: cycle.abs() * 2.0 * num3,
        // The default switch arm's `num = 0.15f` (`:8937-8939`) — 238 is
        // neither the sunflower's 0.07 nor the 521-527 zero.
        rot: cycle * 0.15 * num3,
        pivot: [
            (top_left_x * 16 + SIZE * 8 - tx * 16) as f32,
            (top_left_y * 16 + 16 * SIZE - ty * 16 - tile_top as i32) as f32,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn empty_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// A wind above the 0.08..0.18 gate under an infinite surface (the
    /// sunflower test's `windy` shape).
    fn windy() -> Wind {
        let mut wind = Wind::new(7);
        wind.speed_current = 0.5;
        wind.world_surface = f64::INFINITY;
        wind
    }

    /// A bulb object with its top-left at `(tlx, tly)`: the four cells carry
    /// frameX 0/18 (+36 per style) and frameY 0/18
    /// (`WorldGen.cs:47840-47855`).
    fn bulb_world(tlx: i32, tly: i32, style: i16) -> World {
        let mut w = empty_world();
        for row in 0..2 {
            for col in 0..2 {
                let i = w.index(tlx + col, tly + row);
                w.tiles[i].kind = 238;
                w.tiles[i].flags = FLAG_ACTIVE;
                w.tiles[i].frame_x = 36 * style + 18 * col as i16;
                w.tiles[i].frame_y = 18 * row as i16;
            }
        }
        w.compute_chunks();
        w
    }

    // W4-g84 witness 1 — the per-cell transform of `DrawMultiTileGrassInWind`
    // for the 2×2 bulb: the pivot reconstructs the SAME bottom-centre anchor
    // `(tlx*16 + 16, tly*16 + 32 - 2)` from every cell (the case-238
    // `tileTop = 2` rides `vector2`, `:8982`), the top row's num3 is 0.5 and
    // the bottom row's is the 0-guard 0.1 (`:8964-8968`), so dy/rot scale
    // exactly 5:1 between the rows, and rot = cycle * 0.15 * num3.
    #[test]
    fn w4_g84_bulb_sway_transform_per_cell() {
        let wind = windy();
        let world = bulb_world(10, 20, 0);
        let cycle = wind.cycle(10, 20, 0.0);
        assert!(cycle != 0.0, "the fixture wind must produce a live cycle");
        let mut top = None;
        let mut bottom = None;
        for row in 0..2_i32 {
            for col in 0..2_i32 {
                let (tx, ty) = (10 + col, 20 + row);
                let sw = bulb_sway(
                    &wind,
                    &world,
                    tx,
                    ty,
                    (18 * col) as i16,
                    (18 * row) as i16,
                    2,
                    0.0,
                );
                assert_eq!(
                    sw.pivot,
                    [(16 - 16 * col) as f32, (30 - 16 * row) as f32],
                    "pivot of cell ({tx}, {ty})"
                );
                if row == 0 {
                    assert_eq!(sw.dy, cycle.abs() * 2.0 * 0.5);
                    assert_eq!(sw.rot, cycle * 0.15 * 0.5);
                    top = Some(sw);
                } else {
                    // The bottom row's num3 hits the 0-guard 0.1.
                    assert_eq!(sw.dy, cycle.abs() * 2.0 * 0.1);
                    assert_eq!(sw.rot, cycle * 0.15 * 0.1);
                    bottom = Some(sw);
                }
            }
        }
        let (top, bottom) = (top.unwrap(), bottom.unwrap());
        // f32: the 0.1 factor is not a binary fraction, so the 5:1 row
        // ratio is checked with an ulp-scale tolerance, not bit-equality.
        assert!((top.dy - bottom.dy * 5.0).abs() < 1e-6);
        assert!((top.rot - bottom.rot * 5.0).abs() < 1e-6);
    }

    // W4-g84 witness 2 — the style column: a style-2 bulb's frameX 72/90
    // still reconstructs the same top-left (the `% 36` grid), and the two
    // columns of one object share the cycle (the anchor is the object's,
    // not the cell's).
    #[test]
    fn w4_g84_bulb_sway_reconstructs_the_style_grid() {
        let wind = windy();
        let world = bulb_world(12, 8, 2);
        let a = bulb_sway(&wind, &world, 12, 8, 72, 0, 2, 0.0);
        let b = bulb_sway(&wind, &world, 13, 9, 90, 18, 2, 0.0);
        assert_eq!(a.pivot, [16.0, 30.0]);
        assert_eq!(b.pivot, [0.0, 14.0]);
        // Same object → same cycle → the row ratio holds across columns
        // (ulp-scale tolerance: the 0.1 factor is not a binary fraction).
        assert!((a.rot - b.rot * 5.0).abs() < 1e-6);
        assert!((a.dy - b.dy * 5.0).abs() < 1e-6);
    }

    // W4-g84 witness 3 — the gates freeze the bulb: `!InAPlaceWithWind`
    // (a blocking wall inside the 2×2 window) zeroes num3 (`:8969-8972`),
    // `sway_enabled = false` zeroes the cycle (`GetWindCycle :7678-7681`),
    // and a calm wind (|wind| < 0.08) zeroes the lerp (`:7699`). The pivot
    // survives — the static draw keeps the anchor geometry.
    #[test]
    fn w4_g84_bulb_sway_gates_freeze_the_object() {
        let mut world = bulb_world(10, 20, 0);
        // Wall 1 (stone) does not allow wind (`WallID.Sets.AllowsWind`).
        let i = world.index(11, 21);
        world.tiles[i].wall = 1;
        let wind = windy();
        let sw = bulb_sway(&wind, &world, 10, 20, 0, 0, 2, 0.0);
        assert_eq!((sw.dy, sw.rot), (0.0, 0.0));
        assert_eq!(sw.pivot, [16.0, 30.0]);

        let world = bulb_world(10, 20, 0);
        let mut off = windy();
        off.sway_enabled = false;
        let sw = bulb_sway(&off, &world, 10, 20, 0, 0, 2, 0.0);
        assert_eq!((sw.dy, sw.rot), (0.0, 0.0));

        let mut calm = windy();
        calm.speed_current = 0.05;
        let sw = bulb_sway(&calm, &world, 10, 20, 0, 0, 2, 0.0);
        assert_eq!((sw.dy, sw.rot), (0.0, 0.0));
    }

    // W4-g84 witness 4 — the surface band: a bulb BELOW `Main.worldSurface`
    // (the jungle underground, where bulbs actually grow) gets cycle 0 from
    // `GetWindCycle`'s `y < worldSurface` gate (`:7693-7696`) — the object
    // draws static there, exactly like the C# special pass. The remix
    // world flips the band (`:7683-7690`).
    #[test]
    fn w4_g84_bulb_sway_surface_band() {
        let world = bulb_world(10, 20, 0);
        let mut wind = windy();
        wind.world_surface = 15.0; // the object at y=20 is underground
        let sw = bulb_sway(&wind, &world, 10, 20, 0, 0, 2, 0.0);
        assert_eq!((sw.dy, sw.rot), (0.0, 0.0));
        wind.remix_world = true; // remix: below-surface sways
        let sw = bulb_sway(&wind, &world, 10, 20, 0, 0, 2, 0.0);
        assert!(sw.rot != 0.0);
    }
}
