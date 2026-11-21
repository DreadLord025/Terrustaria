//! The vine lifecycle checks that feed the draw frames (W4-g83): the C#
//! `WorldGen.TileFrame` non-solid arm runs `CheckVines` on every `IsVine`
//! cell (`WorldGen.cs:81375-81378`), and `TileFrameImportant`'s case 549 runs
//! `CheckUnderwaterPlant` (`:84972-84974`). Both decide the cell's TYPE and
//! FRAME — the inputs the vine draw passes (`render::vines`) read — so they
//! are the draw residuals' framing half.
//!
//! * [`check_vines`] (`WorldGen.cs:84858-84960`): the type-conversion ladder
//!   — a vine hanging under a foreign grass converts to that grass's vine
//!   (the glowing-mushroom 70/528 → 528 arm included), and a vine with no
//!   valid support dies. Returns `false` when the cell changed (converted or
//!   killed), exactly like the C# (`TileFrame` then skips the cosmetic
//!   framing, `:81375-81378`).
//! * [`can_underwater_plant_grow_here`] (`WorldGen.cs:58542-58576`) +
//!   [`check_underwater_plant`] (`:58578-58627`): the seaweed (549) validity
//!   + the TIP-FRAME variants — `frameX` column 0 (lone), 1-7 (a 549 above)
//!   or 7-12 (a 549 below), re-rolled with `genRand.Next(1,8)` / `Next(7,13)`
//!   when the column disagrees with the neighbours. These columns are the
//!   rising strip's per-segment source frames (`DrawRisingVineStrip`
//!   `TileDrawing.cs:9173`).
//!
//! UNWIRED (code-only): the natural call sites are contested — `CheckVines`
//! belongs in `framing.rs`'s `TileFrame` non-solid arm (d109/d110/g48) and
//! `CheckUnderwaterPlant` in `TileFrameImportant`'s case 549 / the seaweed
//! growth arm (`growth/decorations.rs`, d102/d106/g42). The pure checks land
//! here; the wiring is the family's merge step. No RNG stream, frame-cache
//! key or fixed-step order changes until wired.
//!
//! Deviations: the `Main.netMode == 2` `NetMessage.SendData`/`SendTileSquare`
//! sides are MP scope (the port's single-player growth never sends); the
//! `SquareTileFrame` re-frame rides [`crate::edit::square_tile_frame_default`]
//! (the C# `WorldGen.SquareTileFrame(i, j)` default `resetFrame: true`).

use crate::tiledata as td;
use crate::unified_random::UnifiedRandom;
use crate::world::World;

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs`, the `int` overload): the
/// `fluff`-tile border guard.
#[inline]
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `Framing.GetTileSafely(x, y)` (`Framing.cs:426-439`): an out-of-world or
/// null cell reads as a fresh inactive `Tile` (kind 0, no liquid). The port's
/// bounded world returns `None` for those, so the readers below default to
/// the same empty shape.
#[inline]
fn safe_kind(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map(|t| t.kind).unwrap_or(0)
}

/// `WorldGen.CheckVines` (`WorldGen.cs:84858-84960`): the vine type-conversion
/// ladder + the support kill. `i, j` is the vine cell, `kind` its type.
/// Returns `true` when the cell is unchanged (the C# `TileFrame` then runs
/// the cosmetic framing), `false` when it converted or died (`:81375-81378`
/// skips the framing on `false`).
///
/// The ladder reads the cell ABOVE (`j - 1`): its `nactive && !bottomSlope`
/// type (`num`, `-1` when absent). The eight family flags map that type to
/// the family vine (`num6`), in the C#'s LAST-write-wins order (`:84883-84914`
/// — ash, mushroom, hallow, jungle, corrupt, crimson, plain, flower). A
/// mismatch converts the cell (`tileCache.type = num6` + `SquareTileFrame`,
/// `:84915-84920`); otherwise the per-type support gates (`:84921-84953`)
/// kill a vine whose support is wrong (`KillTile`, `:84954-84958`).
pub fn check_vines(world: &mut World, i: i32, j: i32, kind: u16) -> bool {
    // `Tile tile = Main.tile[i, j - 1]; if (tile == null) return true;`
    // (`:84860-84864`). The port's out-of-world read is `None`.
    let Some(above) = world.tile(i, j - 1) else {
        return true;
    };
    // `num = -1; if (tile.nactive() && !tile.bottomSlope()) num = tile.type;`
    // (`:84865-84869`). `nactive` = active && !inactive.
    let num: i32 = if above.active() && !above.inactive() && !above.bottom_slope() {
        above.kind as i32
    } else {
        -1
    };
    // `if (type == num) return true;` (`:84870-84873`).
    if kind as i32 == num {
        return true;
    }
    // The eight family flags (`:84874-84881`).
    let jungle = num == 60 || num == 226 || num == 62;
    let hallow = num == 109 || num == 115 || num == 492;
    let corrupt = num == 23 || num == 636 || num == 661;
    let crimson = num == 199 || num == 205 || num == 662;
    let plain = num == 2 || num == 52 || num == 477;
    let flower = num == 382;
    let mushroom = num == 70 || num == 528;
    let ash = num == 633 || num == 638;
    // `num6` — the LAST-write-wins ladder (`:84882-84914`), in source order.
    let mut num6: u16 = 0;
    if ash {
        num6 = 638;
    }
    if mushroom {
        num6 = 528;
    }
    if hallow {
        num6 = 115;
    }
    if jungle {
        num6 = 62;
    }
    if corrupt {
        num6 = 636;
    }
    if crimson {
        num6 = 205;
    }
    if plain && kind != 382 {
        num6 = 52;
    }
    if flower {
        num6 = 382;
    }
    // `if (num6 != 0 && num6 != type) { tileCache.type = num6;
    // SquareTileFrame(i, j); return false; }` (`:84915-84920`).
    if num6 != 0 && num6 != kind {
        if let Some(t) = world.tile_mut(i, j) {
            t.kind = num6;
        }
        crate::edit::square_tile_frame_default(world, i, j);
        return false;
    }
    // The support kill gates (`:84921-84953`). `flag5` starts false; each
    // arm ORs in (the C# writes `flag5 = true` unconditionally per arm).
    let mut kill = num == -1;
    if (kind == 52 || kind == 382) && num != 2 && num != 477 && num != 192 {
        kill = true;
    }
    if kind == 62 && num != 60 && num != 384 && num != 226 {
        kill = true;
    }
    if kind == 115 && num != 109 && num != 492 {
        kill = true;
    }
    if kind == 528 && num != 70 {
        kill = true;
    }
    if kind == 636 && num != 23 && num != 661 {
        kill = true;
    }
    if kind == 205 && num != 199 && num != 662 {
        kill = true;
    }
    if kind == 638 && num != 633 {
        kill = true;
    }
    // `if (flag5) { KillTile(i, j); return false; }` (`:84954-84958`).
    if kill {
        crate::edit::kill_tile(world, i, j, false);
        return false;
    }
    true
}

/// `WorldGen.CanUnderwaterPlantGrowHere` (`WorldGen.cs:58542-58576`): the
/// seaweed (549) placement validity. `ignore_self` skips the "cell already
/// active" gate (the `CheckUnderwaterPlant` re-validation passes `true`).
///
/// The gates: the 50-tile border (`InWorld(x, y, 50)`), the cell free (unless
/// `ignore_self`), three full WATER cells at/above (`y, y-1, y-2`,
/// `liquid != 0 && liquidType() == 0`), a solid-below support that is either
/// conversion sand with an attachable top edge OR the same plant type, and
/// the wall whitelist (0, 63-69, 80-81).
pub fn can_underwater_plant_grow_here(
    world: &World,
    kind: u16,
    x: i32,
    y: i32,
    ignore_self: bool,
) -> bool {
    // `if (!InWorld(x, y, 50)) return false;` (`:58544-58547`).
    if !in_world(world, x, y, 50) {
        return false;
    }
    // `if (!ignoreSelf && tileSafely.active()) return false;` (`:58548-58552`).
    if !ignore_self && world.tile(x, y).is_some_and(|t| t.active()) {
        return false;
    }
    // `for (i = 0; i < 3; i++) { tile = GetTileSafely(x, y - i);
    // if (liquid == 0 || liquidType() != 0) return false; }` (`:58553-58560`).
    // `liquidType() == 0` is water — the port's `liquid_kind() == Water`
    // (no lava/honey/shimmer flag).
    for dy in 0..3 {
        let ok = world
            .tile(x, y - dy)
            .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Water);
        if !ok {
            return false;
        }
    }
    // `tile2 = GetTileSafely(x, y + 1); if (!tile2.nactive()) return false;`
    // (`:58561-58565`).
    let below_ok = world
        .tile(x, y + 1)
        .is_some_and(|t| t.active() && !t.inactive());
    if !below_ok {
        return false;
    }
    // `if ((!Conversion.Sand[tile2.type] || !TopEdgeCanBeAttachedTo(x, y+1))
    // && type != tile2.type) return false;` (`:58566-58569`).
    let below_kind = safe_kind(world, x, y + 1);
    let sand_attach =
        td::SET_CONVERSION_SAND.has(below_kind) && crate::torch::top_edge_can_be_attached_to(world, x, y + 1);
    if !sand_attach && kind != below_kind {
        return false;
    }
    // `wall = GetTileSafely(x, y).wall; if (wall != 0 && (uint)(wall-63) > 6
    // && (uint)(wall-80) > 1) return false;` (`:58570-58574`) — the whitelist
    // is 0, 63..=69, 80..=81.
    let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
    if wall != 0 && !(63..=69).contains(&wall) && !(80..=81).contains(&wall) {
        return false;
    }
    true
}

/// `WorldGen.CheckUnderwaterPlant` (`WorldGen.cs:58578-58627`): the seaweed
/// validity sweep + the TIP-FRAME variants. An invalid cell dies
/// (`KillTile` + `SquareTileFrame`, `:58580-58589`); a valid one gets
/// `frameY = 0` and its `frameX` column re-rolled to match the neighbours:
/// * a 549 ABOVE (`num`) → column 1-7 (`Next(1,8)*18`) when the current
///   column is outside 1..=7 (`:58597-58607`);
/// * else a 549 BELOW (`flag`) → column 7-12 (`Next(7,13)*18`) when outside
///   7..=12 (`:58608-58618`);
/// * else (lone) → column 0 when not already 0 (`:58619-58626`).
///
/// `rng` is the C# `WorldGen.genRand` (the growth stream). The frame columns
/// are the rising strip's per-segment source frames
/// (`DrawRisingVineStrip` `TileDrawing.cs:9173`).
pub fn check_underwater_plant(world: &mut World, kind: u16, x: i32, y: i32, rng: &mut UnifiedRandom) {
    // `if (!CanUnderwaterPlantGrowHere(type, x, y, ignoreSelf: true))
    // { KillTile(x, y); SquareTileFrame(x, y); return; }` (`:58580-58589`).
    if !can_underwater_plant_grow_here(world, kind, x, y, true) {
        crate::edit::kill_tile(world, x, y, false);
        crate::edit::square_tile_frame_default(world, x, y);
        return;
    }
    // `num = above.active() && above.type == type; flag = below.active() &&
    // below.type == type; num2 = frameX / 18; frameY = 0;` (`:58590-58596`).
    let above_same = world
        .tile(x, y - 1)
        .is_some_and(|t| t.active() && t.kind == kind);
    let below_same = world
        .tile(x, y + 1)
        .is_some_and(|t| t.active() && t.kind == kind);
    let col = world.tile(x, y).map(|t| t.frame_x / 18).unwrap_or(0);
    if let Some(t) = world.tile_mut(x, y) {
        t.frame_y = 0;
    }
    if above_same {
        // `if (1 > num2 || num2 > 7) frameX = Next(1,8)*18;` (`:58599-58606`).
        if !(1..=7).contains(&col) {
            let fx = (rng.next_range(1, 8) * 18) as i16;
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_x = fx;
            }
        }
    } else if below_same {
        // `if (7 > num2 || num2 > 12) frameX = Next(7,13)*18;` (`:58610-58617`).
        if !(7..=12).contains(&col) {
            let fx = (rng.next_range(7, 13) * 18) as i16;
            if let Some(t) = world.tile_mut(x, y) {
                t.frame_x = fx;
            }
        }
    } else if col != 0 {
        // `else if (num2 != 0) frameX = 0;` (`:58619-58626`).
        if let Some(t) = world.tile_mut(x, y) {
            t.frame_x = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    /// A 120x120 world (the 50-tile border guard needs the interior).
    fn world() -> World {
        World::new("t".into(), 120, 120)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// Plant a tile keeping the cell's liquid (the seaweed column's water
    /// must survive — `put` replaces the whole Tile and would dry it).
    fn plant(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        let liquid = w.tiles[i].liquid;
        w.tiles[i] = Tile {
            kind,
            liquid,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    // ---- CheckVines: the type-conversion ladder ----

    // A jungle vine 62 under jungle grass 60 is unchanged (`type == num`
    // early-return, `:84870`); under dirt 2 it converts to the plain vine 52
    // (the `flag3` arm, `:84907-84910`) and returns false.
    #[test]
    fn w4_g83_check_vines_converts_to_the_support_family() {
        let mut w = world();
        put(&mut w, 60, 60, 60); // jungle grass above
        put(&mut w, 60, 61, 62); // jungle vine
        assert!(check_vines(&mut w, 60, 61, 62), "same family: unchanged");
        assert_eq!(w.tile(60, 61).unwrap().kind, 62);
        // Replace the support with dirt: the vine converts to 52.
        put(&mut w, 60, 60, 2);
        assert!(!check_vines(&mut w, 60, 61, 62), "foreign support: converts");
        assert_eq!(w.tile(60, 61).unwrap().kind, 52);
    }

    // The glowing-mushroom arm: a 528 vine under mushroom grass 70 is
    // unchanged; a 62 vine under 70 converts to 528 (the `num4` arm wins over
    // the jungle arm in the last-write order, `:84887-84898`).
    #[test]
    fn w4_g83_check_vines_the_glowing_mushroom_arm() {
        let mut w = world();
        put(&mut w, 60, 60, 70); // mushroom grass
        put(&mut w, 60, 61, 528);
        assert!(check_vines(&mut w, 60, 61, 528), "528 under 70: unchanged");
        // A jungle vine under mushroom grass converts to 528.
        put(&mut w, 60, 61, 62);
        assert!(!check_vines(&mut w, 60, 61, 62));
        assert_eq!(w.tile(60, 61).unwrap().kind, 528);
    }

    // The last-write-wins ladder: ash 633 sets 638 first, but a support that
    // is BOTH ash and (say) mushroom cannot happen — instead pin that the
    // flower arm (382) is LAST and overrides the plain arm: a 52 vine under
    // a 382 flower converts to 382, not 52.
    #[test]
    fn w4_g83_check_vines_the_flower_arm_is_last() {
        let mut w = world();
        put(&mut w, 60, 60, 382); // vine flower above
        put(&mut w, 60, 61, 52);
        assert!(!check_vines(&mut w, 60, 61, 52));
        assert_eq!(w.tile(60, 61).unwrap().kind, 382);
    }

    // The plain arm excludes the flower: a 382 vine under dirt 2 does NOT
    // convert to 52 (`flag3 && type != 382`, `:84907`); the 382 support gate
    // accepts num 2/477/192 (`:84926`), so dirt 2 is valid and it survives.
    // Pin: 382 under 2 stays 382.
    #[test]
    fn w4_g83_check_vines_the_flower_survives_dirt() {
        let mut w = world();
        put(&mut w, 60, 60, 2);
        put(&mut w, 60, 61, 382);
        assert!(check_vines(&mut w, 60, 61, 382), "382 under dirt 2: valid support");
        assert_eq!(w.tile(60, 61).unwrap().kind, 382);
    }

    // The support kill: a 528 vine under stone 1 (no family flag → num6 = 0)
    // dies (`:84938`). NOTE: under dirt 2 it would CONVERT to 52 first (the
    // plain arm `:84907` fires before the kill gates) — the kill only fires
    // when no conversion applies.
    #[test]
    fn w4_g83_check_vines_kills_the_unsupported_mushroom_vine() {
        let mut w = world();
        put(&mut w, 60, 60, 1); // stone — no family flag
        put(&mut w, 60, 61, 528);
        assert!(!check_vines(&mut w, 60, 61, 528), "528 under stone: dies");
        assert!(!w.tile(60, 61).unwrap().active(), "the cell is killed");
        // Under dirt the 528 CONVERTS to the plain vine 52 instead (the
        // ladder runs before the kill gates, `:84915-84920`).
        let mut w2 = world();
        put(&mut w2, 60, 60, 2);
        put(&mut w2, 60, 61, 528);
        assert!(!check_vines(&mut w2, 60, 61, 528));
        assert_eq!(w2.tile(60, 61).unwrap().kind, 52, "dirt converts 528 → 52");
    }

    // A vine with no support above (inactive) dies (`num == -1`, `:84922`).
    #[test]
    fn w4_g83_check_vines_kills_the_floating_vine() {
        let mut w = world();
        put(&mut w, 60, 61, 62); // no cell above
        assert!(!check_vines(&mut w, 60, 61, 62));
        assert!(!w.tile(60, 61).unwrap().active());
    }

    // A bottom-sloped support reads as absent (`!bottomSlope` gate, `:84866`)
    // → the vine dies.
    #[test]
    fn w4_g83_check_vines_a_bottom_slope_support_is_absent() {
        let mut w = world();
        put(&mut w, 60, 60, 60);
        w.tiles[w.index(60, 60)].set_slope(3); // bottom slope
        put(&mut w, 60, 61, 62);
        assert!(!check_vines(&mut w, 60, 61, 62));
        assert!(!w.tile(60, 61).unwrap().active());
    }

    // An actuated (inactive) support reads as absent (`nactive`, `:84866`).
    #[test]
    fn w4_g83_check_vines_an_actuated_support_is_absent() {
        let mut w = world();
        put(&mut w, 60, 60, 60);
        w.tiles[w.index(60, 60)].flags |= crate::tile::FLAG_INACTIVE;
        put(&mut w, 60, 61, 62);
        assert!(!check_vines(&mut w, 60, 61, 62));
    }

    // ---- CheckUnderwaterPlant: the tip-frame variants ----

    /// A valid seaweed column: water at y/y-1/y-2, sand floor at y+1, the
    /// 549 cell at (60, 60). The 50-border needs x,y in 50..70.
    fn seaweed_world() -> World {
        let mut w = world();
        for dy in 0..3 {
            let i = w.index(60, 60 - dy);
            w.tiles[i].liquid = 255; // full water
        }
        put(&mut w, 60, 61, 53); // sand floor (conversion sand)
        put(&mut w, 60, 60, 549);
        w
    }

    // The validity gate: full water above + sand floor passes; dry water
    // fails.
    #[test]
    fn w4_g83_the_seaweed_validity_gates() {
        let w = seaweed_world();
        assert!(can_underwater_plant_grow_here(&w, 549, 60, 60, true));
        let mut dry = seaweed_world();
        dry.tiles[dry.index(60, 59)].liquid = 0; // y-1 dry
        assert!(!can_underwater_plant_grow_here(&dry, 549, 60, 60, true));
        // ignore_self=false refuses an already-active cell.
        assert!(!can_underwater_plant_grow_here(&w, 549, 60, 60, false));
    }

    // The wall whitelist: wall 0 passes, wall 63-69 / 80-81 pass, others fail.
    #[test]
    fn w4_g83_the_seaweed_wall_whitelist() {
        let mut w = seaweed_world();
        for wall in [0u16, 63, 69, 80, 81] {
            w.tiles[w.index(60, 60)].wall = wall;
            assert!(
                can_underwater_plant_grow_here(&w, 549, 60, 60, true),
                "wall {wall} allowed"
            );
        }
        for wall in [1u16, 62, 70, 79, 82] {
            w.tiles[w.index(60, 60)].wall = wall;
            assert!(
                !can_underwater_plant_grow_here(&w, 549, 60, 60, true),
                "wall {wall} refused"
            );
        }
    }

    // A lone seaweed (no 549 above/below) frames to column 0 and frameY 0.
    #[test]
    fn w4_g83_the_lone_seaweed_frames_to_column_zero() {
        let mut w = seaweed_world();
        w.tiles[w.index(60, 60)].frame_x = 90; // a stale column 5
        w.tiles[w.index(60, 60)].frame_y = 18;
        let mut rng = UnifiedRandom::new(1);
        check_underwater_plant(&mut w, 549, 60, 60, &mut rng);
        let t = w.tile(60, 60).unwrap();
        assert_eq!(t.frame_x, 0, "lone → column 0");
        assert_eq!(t.frame_y, 0);
    }

    // A 549 ABOVE forces column 1-7; a stale column 0 re-rolls into 18..=126.
    #[test]
    fn w4_g83_a_seaweed_above_forces_columns_one_to_seven() {
        let mut w = seaweed_world();
        plant(&mut w, 60, 59, 549); // a 549 above (keeps the water)
        w.tiles[w.index(60, 60)].frame_x = 0; // stale lone column
        let mut rng = UnifiedRandom::new(7);
        check_underwater_plant(&mut w, 549, 60, 60, &mut rng);
        let fx = w.tile(60, 60).unwrap().frame_x;
        assert!((18..=126).contains(&fx) && fx % 18 == 0, "column 1-7, got {fx}");
        // A column already in 1-7 is kept (no re-roll).
        w.tiles[w.index(60, 60)].frame_x = 54; // column 3
        check_underwater_plant(&mut w, 549, 60, 60, &mut rng);
        assert_eq!(w.tile(60, 60).unwrap().frame_x, 54);
    }

    // A 549 BELOW (and none above) forces column 7-12; a stale column 0
    // re-rolls into 126..=216.
    #[test]
    fn w4_g83_a_seaweed_below_forces_columns_seven_to_twelve() {
        let mut w = seaweed_world();
        // Move the floor down and put a 549 below the cell.
        put(&mut w, 60, 61, 549);
        put(&mut w, 60, 62, 53); // new sand floor
        w.tiles[w.index(60, 60)].frame_x = 0;
        let mut rng = UnifiedRandom::new(11);
        check_underwater_plant(&mut w, 549, 60, 60, &mut rng);
        let fx = w.tile(60, 60).unwrap().frame_x;
        assert!((126..=216).contains(&fx) && fx % 18 == 0, "column 7-12, got {fx}");
    }

    // The above-arm wins over the below-arm (the C# else-if, `:58597/58608`):
    // with 549 both above and below, the column lands in 1-7, not 7-12.
    #[test]
    fn w4_g83_the_above_arm_wins_over_the_below_arm() {
        let mut w = seaweed_world();
        plant(&mut w, 60, 59, 549); // above (keeps the water)
        put(&mut w, 60, 61, 549); // below (replaces the sand)
        put(&mut w, 60, 62, 53);
        w.tiles[w.index(60, 60)].frame_x = 0;
        let mut rng = UnifiedRandom::new(3);
        check_underwater_plant(&mut w, 549, 60, 60, &mut rng);
        let fx = w.tile(60, 60).unwrap().frame_x;
        assert!((18..=126).contains(&fx), "above wins → 1-7, got {fx}");
    }

    // An invalid seaweed (dry water) dies and reframes.
    #[test]
    fn w4_g83_an_invalid_seaweed_dies() {
        let mut w = seaweed_world();
        w.tiles[w.index(60, 59)].liquid = 0;
        let mut rng = UnifiedRandom::new(1);
        check_underwater_plant(&mut w, 549, 60, 60, &mut rng);
        assert!(!w.tile(60, 60).unwrap().active(), "the dry seaweed is killed");
    }

    /// W4-g83 ROPE-interplay census pin: natural vines are NOT climbable —
    /// `TileID.Sets.IsVine` (`TileID.cs:531`, the 8 kinds) and
    /// `Main.tileRope` (`Main.cs:9313-9321`, the 9 rope kinds incl. the
    /// crafted VineRope 353) are DISJOINT, and so are `ReverseVineThreads`
    /// (549) and `tileRope`. `Player.cs`'s only `IsVine` read is the
    /// placement-kill gate (`:41309`); the climbing movement
    /// (`FindPulley` `:19644-19749` + `Update`'s rope block
    /// `:26116-26470`) reads `WorldGen.IsRope`/`Main.tileRope` only —
    /// see `docs/research/123-vine-wire-contract.md` §7 (NOT ported:
    /// ~430 lines entangled with `Player.Update`, not self-contained).
    #[test]
    fn w4_g83_vines_and_ropes_are_disjoint_sets() {
        for id in [52u16, 382, 62, 115, 205, 528, 636, 638, 549] {
            assert!(
                !td::TILE_ROPE.has(id),
                "vine/seaweed {id} is not a rope — no pulley attach"
            );
        }
        for id in [213u16, 214, 353, 365, 366, 504, 449, 450, 451] {
            assert!(td::TILE_ROPE.has(id), "rope {id} climbs");
            assert!(
                !crate::tileobject::is_vine(id),
                "rope {id} is not a natural vine — CheckVines never touches it"
            );
            assert!(!td::SET_REVERSE_VINE_THREADS.has(id));
        }
    }

    /// W4-g83 census pin: the frame-important asymmetry that routes the two
    /// checks — the seaweed 549 IS frame-important (`Main.cs:7373`, so
    /// `TileFrame` dispatches `TileFrameImportant` case 549 →
    /// `CheckUnderwaterPlant`, `WorldGen.cs:84972-84974`), while NONE of the
    /// 8 vine kinds is (no `tileFrameImportant[52/62/…]` write exists in
    /// `Main.cs` — they ride `TileFrame`'s non-solid `IsVine` arm,
    /// `:81375-81378`). `vine_growth::plant_vine` sets the flag anyway
    /// (documented deviation, contract §3).
    #[test]
    fn w4_g83_only_the_seaweed_is_frame_important() {
        // The port's generated table has no frame-important set; the C#
        // census is pinned here as data (Main.cs:7373 vs the absent writes).
        const FRAME_IMPORTANT_549: bool = true; // Main.cs:7373
        assert!(FRAME_IMPORTANT_549);
        for id in [52u16, 382, 62, 115, 205, 528, 636, 638] {
            assert!(
                crate::tileobject::is_vine(id),
                "vine {id} rides the non-solid TileFrame arm, not case dispatch"
            );
        }
    }
}
