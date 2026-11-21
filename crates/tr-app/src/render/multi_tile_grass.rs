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
//! Scope: the MultiTileGrass 2×2 / 3×2 / 2×3 / 1×2 family — the bulb family
//! 236/238/702 (the shared `TileDrawing.cs:606-613` / `:8760-8764` arms), the
//! 2×2 default-arm tiles 485/490/652, the frozen dragonfly jars 521-527
//! (`:8940-8949`), the 3×2 oasis/echo plants 530/651/705 (`:8769-8774`), the
//! 2×3 pinwheel 489 (`:8788-8791`) and the 1×2 golf flag 493 with its
//! wind-sign flip + ∓6 px shift (`:8792-8795`, `:8983-8997`). Only 238
//! animates a row among the bulb family; 236/702 have no `GetTileDrawData`
//! case (`tileTop = 0`). The remaining roster (27 sunflower — landed
//! per-cell; 233 detritus, 519 cattail) is the general-pass follow-up.
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

/// The bulb-family (236/238/702) specialization of [`grass_sway_2x2`] —
/// the default 0.15 factor, no freeze. Kept as the witness-facing entry
/// point for the original bulb tests; the dispatch calls the general
/// function directly.
#[cfg(test)]
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
    grass_sway_2x2(
        wind, world, tx, ty, frame_x, frame_y, tile_top,
        sunflower_wind_counter, 0.15, false,
    )
}

/// The general `DrawMultiTileGrassInWind` per-cell transform for the 2×2
/// default-arm family (`TileDrawing.cs:8916-9032`): the bulb family
/// 236/238/702, the antlion larva 485, the weather vane 490, the echo
/// detritus 652 and the dragonfly jars 521-527. The object's top-left is
/// reconstructed from the `Style2x2` frame grid (`frameX = 36*style +
/// 18*col`, `frameY = 18*row`); `num` is the type's sway factor (the
/// default arm's 0.15, `:8937-8939`) and `freeze` the 521-527 arm
/// (`num = 0`, `flag = false`, `:8940-8949`) that pins the jars static.
/// The 493 golf flag's wind-sign flip + ±6 px shift (`:8983-8997`) is NOT
/// here — it lives in the dedicated 1×2 arm.
pub(crate) fn grass_sway_2x2(
    wind: &Wind,
    world: &World,
    tx: i32,
    ty: i32,
    frame_x: i16,
    frame_y: i16,
    tile_top: i16,
    sunflower_wind_counter: f32,
    num: f32,
    freeze: bool,
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
    // with the 2×2 window; the 521-527 arm forces it false (`:8948`).
    let in_wind =
        !freeze && wind.in_a_place_with_wind(world, top_left_x, top_left_y, SIZE, SIZE);
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
        rot: cycle * num * num3,
        pivot: [
            (top_left_x * 16 + SIZE * 8 - tx * 16) as f32,
            (top_left_y * 16 + 16 * SIZE - ty * 16 - tile_top as i32) as f32,
        ],
    }
}

/// The 3×2 MultiTileGrass transform (`TileDrawing.cs:8769-8774`: 530 oasis
/// plants, 651 echo detritus, 705 echo oasis — `sizeX = 3, sizeY = 2`). The
/// frame grid is `Style3x2` (`frameX = 54*style + 18*col`, `frameY = 18*row`,
/// `WorldGen.PlaceJunglePlant` `:47908-47932`); the bottom-centre anchor is
/// `(topLeftX*16 + 24, topLeftY*16 + 32)`. All three take the default 0.15
/// factor. The 530 biome-row shift rides `GetTileDrawData` (already landed).
pub(crate) fn grass_sway_3x2(
    wind: &Wind,
    world: &World,
    tx: i32,
    ty: i32,
    frame_x: i16,
    frame_y: i16,
    tile_top: i16,
    sunflower_wind_counter: f32,
) -> BulbSway {
    let col = (frame_x.rem_euclid(54) / 18) as i32;
    let row = (frame_y / 18) as i32;
    let top_left_x = tx - col;
    let top_left_y = ty - row;
    let cycle = wind.cycle(top_left_x, top_left_y, sunflower_wind_counter);
    let in_wind = wind.in_a_place_with_wind(world, top_left_x, top_left_y, 3, SIZE);
    let mut num3 = 1.0 - (row + 1) as f32 / SIZE as f32;
    if num3 == 0.0 {
        num3 = 0.1;
    }
    if !in_wind {
        num3 = 0.0;
    }
    BulbSway {
        dy: cycle.abs() * 2.0 * num3,
        rot: cycle * 0.15 * num3,
        pivot: [
            (top_left_x * 16 + 24 - tx * 16) as f32,
            (top_left_y * 16 + 16 * SIZE - ty * 16 - tile_top as i32) as f32,
        ],
    }
}

/// The 489 pinwheel's 2×3 MultiTileGrass transform
/// (`TileDrawing.cs:8788-8791`: `sizeX = 2, sizeY = 3`). The frame grid is
/// `Style2xX` (`frameX = 36*style + 18*col`, `frameY = 18*row`); the
/// bottom-centre anchor is `(topLeftX*16 + 16, topLeftY*16 + 48)`. The
/// default 0.15 factor applies; the spin animation rides the landed
/// `drawdata_cases_b` case 489 (`InAPlaceWithWind` + counter, `:4731-4741`).
pub(crate) fn grass_sway_2x3(
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
    let cycle = wind.cycle(top_left_x, top_left_y, sunflower_wind_counter);
    let in_wind = wind.in_a_place_with_wind(world, top_left_x, top_left_y, SIZE, 3);
    let mut num3 = 1.0 - (row + 1) as f32 / 3.0;
    if num3 == 0.0 {
        num3 = 0.1;
    }
    if !in_wind {
        num3 = 0.0;
    }
    BulbSway {
        dy: cycle.abs() * 2.0 * num3,
        rot: cycle * 0.15 * num3,
        pivot: [
            (top_left_x * 16 + SIZE * 8 - tx * 16) as f32,
            (top_left_y * 16 + 16 * 3 - ty * 16 - tile_top as i32) as f32,
        ],
    }
}

/// The 493 golf flag's wind transform PLUS its sprite flip / horizontal
/// shift (`TileDrawing.cs:8983-8997`). The 1×2 object (`:8792-8795`) sways
/// like the default family, but on frame row 0 the flag flips horizontally
/// when the wind blows right (`Main.WindForVisuals >= 0`) and shifts ∓6 px
/// so the pole stays put. Returns `(sway, flip_xor, dx)`: `flip_xor` toggles
/// the cell's `dd.flip`, `dx` shifts the draw X. Only frame row 0 carries
/// the flip/shift; the bottom row keeps the plain transform.
pub(crate) fn golf_flag_sway(
    wind: &Wind,
    world: &World,
    tx: i32,
    ty: i32,
    frame_x: i16,
    frame_y: i16,
    tile_top: i16,
    sunflower_wind_counter: f32,
    base_flip: bool,
) -> (BulbSway, bool, f32) {
    let col = (frame_x.rem_euclid(18) / 18) as i32; // 1-wide: col always 0
    let row = (frame_y / 18) as i32;
    let top_left_x = tx - col;
    let top_left_y = ty - row;
    let cycle = wind.cycle(top_left_x, top_left_y, sunflower_wind_counter);
    let in_wind = wind.in_a_place_with_wind(world, top_left_x, top_left_y, 1, SIZE);
    let mut num3 = 1.0 - (row + 1) as f32 / SIZE as f32;
    if num3 == 0.0 {
        num3 = 0.1;
    }
    if !in_wind {
        num3 = 0.0;
    }
    let sway = BulbSway {
        dy: cycle.abs() * 2.0 * num3,
        rot: cycle * 0.15 * num3,
        pivot: [
            (top_left_x * 16 + 8 - tx * 16) as f32,
            (top_left_y * 16 + 16 * SIZE - ty * 16 - tile_top as i32) as f32,
        ],
    };
    // `if (tile.type == 493 && tile.frameY == 0)` (`:8983`).
    if frame_y != 0 {
        return (sway, false, 0.0);
    }
    // `tileSpriteEffect ^= FlipHorizontally` when `WindForVisuals >= 0`
    // (`:8985-8988`); the shift keys off the RESULTING flip (`:8989-8996`).
    let mut flip = base_flip;
    if wind.speed_current >= 0.0 {
        flip = !flip;
    }
    let dx = if !flip { -6.0 } else { 6.0 };
    (sway, flip != base_flip, dx)
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

    // W4-g84 follow-up witness 5 — the Life Fruit siblings 236/702 share the
    // bulb's 2×2 grid and 0.15 factor but carry NO `GetTileDrawData` case, so
    // `tileTop = 0` (vs 238's 2): the bottom-centre anchor is the same
    // `(tlx*16+16, tly*16+32)` but the pivot's Y is 2 px LARGER (the cell's
    // `vector2` has no +2). The shared registration `:606-613` and size arm
    // `:8760-8764` cover all three; only 238 animates (`:5441-5444`).
    #[test]
    fn w4_g84_life_fruit_siblings_use_top_zero() {
        let wind = windy();
        let world = bulb_world(10, 20, 0);
        // 238 (tileTop 2) vs 236/702 (tileTop 0), same cell, same object.
        let bulb = bulb_sway(&wind, &world, 10, 20, 0, 0, 2, 0.0);
        let fruit = bulb_sway(&wind, &world, 10, 20, 0, 0, 0, 0.0);
        assert_eq!(bulb.pivot, [16.0, 30.0]);
        assert_eq!(fruit.pivot, [16.0, 32.0]);
        // dy/rot are tileTop-independent (they ride num3 + the cycle only).
        assert_eq!(bulb.dy, fruit.dy);
        assert_eq!(bulb.rot, fruit.rot);
        // The bottom row's pivot: 238 → 14, fruit → 16.
        let bulb_b = bulb_sway(&wind, &world, 10, 21, 0, 18, 2, 0.0);
        let fruit_b = bulb_sway(&wind, &world, 10, 21, 0, 18, 0, 0.0);
        assert_eq!(bulb_b.pivot, [16.0, 14.0]);
        assert_eq!(fruit_b.pivot, [16.0, 16.0]);
    }

    // W4-g84 follow-up witness 6 — the 2×2 default-arm family (485 antlion
    // larva, 490 weather vane, 652 echo detritus) shares the bulb's grid and
    // 0.15 factor; all carry `tileTop = 2` (the GetTileDrawData cases
    // `:4727-4818`, `:4693-4695`), so the pivot matches the 238 bulb exactly.
    #[test]
    fn w4_g84_two_by_two_family_matches_the_bulb_pivot() {
        let wind = windy();
        let world = bulb_world(10, 20, 0);
        let bulb = grass_sway_2x2(&wind, &world, 10, 20, 0, 0, 2, 0.0, 0.15, false);
        let larva = grass_sway_2x2(&wind, &world, 10, 20, 0, 0, 2, 0.0, 0.15, false);
        assert_eq!(bulb.pivot, [16.0, 30.0]);
        assert_eq!(larva.pivot, bulb.pivot);
        assert_eq!(larva.rot, bulb.rot);
        assert_eq!(larva.dy, bulb.dy);
    }

    // W4-g84 follow-up witness 7 — the dragonfly jars 521-527 freeze: the
    // `num = 0`, `flag = false` arm (`:8940-8949`) zeroes num3 regardless of
    // wind, so dy AND rot are 0 even in a live wind; the pivot survives (the
    // static draw keeps the anchor geometry).
    #[test]
    fn w4_g84_dragonfly_jars_freeze() {
        let wind = windy();
        let world = bulb_world(10, 20, 0);
        let cycle = wind.cycle(10, 20, 0.0);
        assert!(cycle != 0.0, "the fixture wind must produce a live cycle");
        let frozen = grass_sway_2x2(&wind, &world, 10, 20, 0, 0, 2, 0.0, 0.0, true);
        assert_eq!((frozen.dy, frozen.rot), (0.0, 0.0));
        assert_eq!(frozen.pivot, [16.0, 30.0]);
        // Even with the 0.15 factor passed, `freeze` still forces num3 = 0.
        let frozen2 = grass_sway_2x2(&wind, &world, 10, 20, 0, 0, 2, 0.0, 0.15, true);
        assert_eq!((frozen2.dy, frozen2.rot), (0.0, 0.0));
    }

    // W4-g84 follow-up witness 8 — the 3×2 oasis/echo plants (530/651/705):
    // the Style3x2 grid (`frameX = 54*style + 18*col`) reconstructs the same
    // top-left from all three columns, and the bottom-centre anchor is
    // `(tlx*16 + 24, tly*16 + 32)` — 8 px wider than the 2×2 family. The
    // per-row num3 ladder (0.5 top, 0.1 bottom) and 0.15 factor match.
    #[test]
    fn w4_g84_three_by_two_anchor_is_wider() {
        let wind = windy();
        let mut w = empty_world();
        for row in 0..2 {
            for col in 0..3 {
                let i = w.index(10 + col, 20 + row);
                w.tiles[i].kind = 530;
                w.tiles[i].flags = FLAG_ACTIVE;
                w.tiles[i].frame_x = (18 * col) as i16;
                w.tiles[i].frame_y = (18 * row) as i16;
            }
        }
        w.compute_chunks();
        let cycle = wind.cycle(10, 20, 0.0);
        assert!(cycle != 0.0);
        for col in 0..3_i32 {
            let sw = grass_sway_3x2(&wind, &w, 10 + col, 20, (18 * col) as i16, 0, 2, 0.0);
            assert_eq!(sw.pivot, [(24 - 16 * col) as f32, 30.0], "col {col}");
            assert_eq!(sw.dy, cycle.abs() * 2.0 * 0.5);
            assert_eq!(sw.rot, cycle * 0.15 * 0.5);
        }
        // A style-1 plant (frameX 54/72/90) still reconstructs the top-left.
        let a = grass_sway_3x2(&wind, &w, 10, 20, 54, 0, 2, 0.0);
        assert_eq!(a.pivot, [24.0, 30.0]);
    }

    // W4-g84 follow-up witness 9 — the 489 pinwheel's 2×3 anchor: the
    // bottom-centre is `(tlx*16+16, tly*16+48)` (sizeY 3, `:8788-8791`), the
    // num3 ladder is 2/3, 1/3, 0.1 across the three rows, and the 0.15
    // factor matches the default family.
    #[test]
    fn w4_g84_pinwheel_two_by_three_anchor() {
        let wind = windy();
        let mut w = empty_world();
        for row in 0..3 {
            for col in 0..2 {
                let i = w.index(10 + col, 20 + row);
                w.tiles[i].kind = 489;
                w.tiles[i].flags = FLAG_ACTIVE;
                w.tiles[i].frame_x = (18 * col) as i16;
                w.tiles[i].frame_y = (18 * row) as i16;
            }
        }
        w.compute_chunks();
        let cycle = wind.cycle(10, 20, 0.0);
        assert!(cycle != 0.0);
        let top = grass_sway_2x3(&wind, &w, 10, 20, 0, 0, 2, 0.0);
        let mid = grass_sway_2x3(&wind, &w, 10, 21, 0, 18, 2, 0.0);
        let bot = grass_sway_2x3(&wind, &w, 10, 22, 0, 36, 2, 0.0);
        assert_eq!(top.pivot, [16.0, 46.0]);
        assert_eq!(mid.pivot, [16.0, 30.0]);
        assert_eq!(bot.pivot, [16.0, 14.0]);
        assert_eq!(top.rot, cycle * 0.15 * (2.0 / 3.0));
        assert_eq!(mid.rot, cycle * 0.15 * (1.0 / 3.0));
        assert_eq!(bot.rot, cycle * 0.15 * 0.1);
    }

    // W4-g84 follow-up witness 10 — the 493 golf flag's wind-sign flip +
    // ∓6 px shift (`:8983-8997`): on frame row 0 a right-blowing wind
    // (speed >= 0) flips the sprite and shifts +6; a left wind keeps the
    // base flip and shifts -6. The bottom row (frameY 18) never flips or
    // shifts. The 1×2 anchor is `(tlx*16+8, tly*16+32)`.
    #[test]
    fn w4_g84_golf_flag_flip_and_shift() {
        let mut w = empty_world();
        for row in 0..2 {
            let i = w.index(10, 20 + row);
            w.tiles[i].kind = 493;
            w.tiles[i].flags = FLAG_ACTIVE;
            w.tiles[i].frame_x = 0;
            w.tiles[i].frame_y = (18 * row) as i16;
        }
        w.compute_chunks();
        let mut right = windy();
        right.speed_current = 0.5;
        let (sw, xor, dx) = golf_flag_sway(&right, &w, 10, 20, 0, 0, 2, 0.0, false);
        assert_eq!(sw.pivot, [8.0, 30.0]);
        assert!(xor, "right wind flips the flag");
        assert_eq!(dx, 6.0);
        let mut left = windy();
        left.speed_current = -0.5;
        let (_, xor_l, dx_l) = golf_flag_sway(&left, &w, 10, 20, 0, 0, 2, 0.0, false);
        assert!(!xor_l, "left wind keeps the base flip");
        assert_eq!(dx_l, -6.0);
        // The bottom row never flips/shifts regardless of wind.
        let (_, xor_b, dx_b) = golf_flag_sway(&right, &w, 10, 21, 0, 18, 2, 0.0, false);
        assert!(!xor_b);
        assert_eq!(dx_b, 0.0);
    }
}
