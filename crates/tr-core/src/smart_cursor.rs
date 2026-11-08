//! The smart-cursor first pass — `SmartCursorHelper.SmartCursorLookup`
//! (`GameContent/SmartCursorHelper.cs:61-165`) with the **torch step**
//! (`Step_Torch` :2795-2866 + `IsValidSpotForTorch` :2867-2936).
//!
//! The vanilla scans the reachable rect for a match and re-points
//! `Main.SmartCursorX/Y` (a copy of `Player.tileTargetX/Y`, `:158-162`).
//! The port models the lookup over its tile grid: the caller passes the
//! reachable rect and the held item kind; the module re-points
//! `App.smart_cursor: Option<(i32, i32)>` when a better spot exists.
//!
//! The torch step's full logic is ported verbatim: the water-torch
//! `flag` arm, the cell scan skipping liquid/occupied non-cut cells, the
//! 17x17 no-torch-around sweep, `IsValidSpotForTorch`'s wall / side /
//! floor anchors, and the closest-to-mouse pick among the candidates
//! (`Collision.InTileBounds`' inclusive rect gate at the end).
//!
//! Out of scope (documented in the ROADMAP row): the other 30 steps (the
//! pickaxe/axe/hammer/platform/wall/grass-seed/moss/clay-pot/actuation
//! families), `UsingSmartCursorWithContinuity`'s lock state, the grapple
//! target arms, and the paint-lookup arm.

use crate::tile::Tile;

/// `Main.SmartCursorIsUsed` is a per-player setting the port reads off the
/// `TR_SMARTCURSOR` pin (the input model lands with the UI parity pass);
/// the engine's gate lives here so the caller has one condition.
pub const SMART_CURSOR_IS_USED: bool = true;

/// `ItemID.Sets.Torches` (`TileID.cs:615`) — the vanilla set is the single
/// torch tile 4 (`Sets.Torches = Factory.CreateBoolSet(4)`), extended by the
/// modded variants this decompile does not carry.
pub const TORCH_TILE: u16 = 4;
/// `ItemID.Sets.WaterTorches` (`ItemID.cs:1372`): the item ids that place
/// underwater torches.
pub const WATER_TORCH_ITEMS: [u16; 3] = [523, 1333, 4384];
/// `TileID.Sets.BreakableWhenPlacing` (`TileID.cs:604`): the tiles an
/// item placement may replace (the trap family 185/187, the saplings, the
/// minecart track 165, portal 530…).
pub const BREAKABLE_WHEN_PLACING: [u16; 11] =
    [324, 186, 187, 185, 165, 530, 233, 227, 485, 81, 624];

fn is_torch(kind: u16) -> bool {
    kind == TORCH_TILE
}

/// `Main.tileCut[type]`'s herb/shrub row — the vanilla `tileCut` set the
/// step's skip arm consults (`tile.active() && !BreakableWhenPlacing &&
/// (!tileCut || type == 82 || type == 83)`). The port's set is the growing
/// herbs 82-84 + the sapling 20 (the `TileID.Sets.CommonSapling` row) —
/// `growth.rs`' `TILE_ALCH` already carries the ids; the sapling 20 sits
/// in the C# set via the tree-vine family.
fn is_cut(kind: u16) -> bool {
    matches!(kind, 20 | 82 | 83 | 84)
}

/// `TileID.Sets.NotReallySolid`'s no-attach family — the solids a torch may
/// not hang on. The decompile's set is the closed doors/tall gates; the
/// port keeps the closed-door row 10/11 (its door kinds).
fn is_not_really_solid(kind: u16) -> bool {
    matches!(kind, 10 | 11)
}

/// `Collision.InTileBounds` (`Collision.cs:1501-1507`): the inclusive rect
/// test the step's final gate runs (`Step_Torch` :2859).
pub fn in_tile_bounds(x: i32, y: i32, start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> bool {
    x >= start_x && x <= end_x && y >= start_y && y <= end_y
}

/// `SmartCursorHelper.IsValidSpotForTorch` (`SmartCursorHelper.cs:2867-2936`):
/// the tile accepts a torch — a wall behind it, a valid side anchor (left or
/// right, the beam/three-tree-column arm included), or a floor anchor
/// (`tileSolid && !tileSolidTop`, the platform row, no halfBrick).
pub fn is_valid_spot_for_torch(w: &crate::World, x: i32, y: i32, cache: Option<&Tile>) -> bool {
    if let Some(t) = cache {
        if t.wall > 0 {
            return true;
        }
        if is_torch(t.kind) {
            return false;
        }
    } else {
        let Some(t) = w.tile(x, y) else {
            return false;
        };
        if t.wall > 0 {
            return true;
        }
        if is_torch(t.kind) {
            return false;
        }
    }
    let attachable = |t: &Tile, allow_slopes: bool| -> bool {
        if !t.active() {
            return false;
        }
        let slope_ok = allow_slopes || t.slope() == 0;
        if !slope_ok {
            return false;
        }
        (crate::tiledata::TILE_SOLID.has(t.kind)
            && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
            && !is_not_really_solid(t.kind))
            || t.kind == 5 // the beam family tile
    };
    // The side anchors: the C#'s `slope() == 0 || slope() % 2 != X` split
    // (left: even, right: odd) — the slope-0 default covers the port's
    // flat tiles; the tree column's `IsTreeType` triple arm is dropped
    // (worldgen shrinks trees to trunks in this slice).
    if let Some(t) = w.tile(x - 1, y) {
        if attachable(t, false) {
            return true;
        }
    }
    if let Some(t) = w.tile(x + 1, y) {
        if attachable(t, false) {
            return true;
        }
    }
    if let Some(t) = w.tile(x, y + 1) {
        if t.active()
            && t.slope() == 0
            && !t.half_brick()
            && (crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                || crate::tiledata::SET_PLATFORMS.has(t.kind))
            && !is_not_really_solid(t.kind)
        {
            return true;
        }
    }
    false
}

/// `SmartCursorHelper.Step_Torch` (`:2795-2866`): the held item is a
/// torch → the reachable rect's closest valid empty spot. Returns the
/// re-pointed `(x, y)` or `None`. `mouse` is the mouse world position (the
/// closeness vote, `:2848`) and `(fx, fy)` the current focus (−1 when
/// empty — an earlier step's target wins).
pub fn step_torch(
    w: &crate::World,
    item_kind: u16,
    fx: i32,
    fy: i32,
    mouse: (f32, f32),
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) -> Option<(i32, i32)> {
    if fx != -1 || fy != -1 {
        return None;
    }
    // The item id gate: the port's torch item is 8 (`registry::name(8)`
    // carries the torch; `itemdata.rs`' tile placement), the water torches
    // are 523/1333/4384 — the vanilla reads `ItemID.Sets.Torches[type]`
    // on the ITEM id, which is true for the item kinds whose
    // `SetDefaults` places a torch tile.
    let is_water = WATER_TORCH_ITEMS.contains(&item_kind);
    if !is_water && item_kind != 8 {
        return None;
    }
    let water_flag = !is_water;
    let mut best: Option<(i32, i32)> = None;
    let mut best_dist = f32::INFINITY;
    for i in start_x..=end_x {
        for j in start_y..=end_y {
            let Some(t) = w.tile(i, j) else {
                continue;
            };
            // `:2832-2835`: the water-torch gate / the occupied non-cut skip.
            if (water_flag && t.liquid > 0)
                || (t.active()
                    && !BREAKABLE_WHEN_PLACING.contains(&t.kind)
                    && (!is_cut(t.kind)))
            {
                continue;
            }
            // `:2837-2848`: no torch within the ±8 box (`17x17`'s `k/l`
            // sweep over the whole width including the row just outside).
            let mut torch_near = false;
            for k in (i - 8)..=(i + 8) {
                for l in (j - 8)..=(j + 8) {
                    if let Some(t2) = w.tile(k, l) {
                        if is_torch(t2.kind) {
                            torch_near = true;
                            break;
                        }
                    }
                }
                if torch_near {
                    break;
                }
            }
            if torch_near {
                continue;
            }
            if !is_valid_spot_for_torch(w, i, j, Some(t)) {
                continue;
            }
            // `:2849-2854`: the closest-to-mouse pick.
            let dist = ((i as f32 * 16.0 + 8.0) - mouse.0)
                .hypot((j as f32 * 16.0 + 8.0) - mouse.1);
            if dist < best_dist {
                best_dist = dist;
                best = Some((i, j));
            }
        }
    }
    best
}

/// `SmartCursorHelper.SmartCursorLookup`'s first-pass shell
/// (`:61-165` minus the 30 unported steps): the rect gate — an
/// interactable blocking the cursor short-circuits to `None`
// (`IsHoveringOverAnInteractableTileThatBlocksSmartCursor`, :193 — the
/// port has no sign/chest hover routing yet, so the gate is a no-op that
/// always passes) — then the torch step over the clamped reachable rect.
/// Returns the re-pointed tile or `None`.
pub fn smart_cursor_lookup(
    w: &crate::World,
    item_kind: u16,
    fx: i32,
    fy: i32,
    mouse: (f32, f32),
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) -> Option<(i32, i32)> {
    step_torch(w, item_kind, fx, fy, mouse, start_x, start_y, end_x, end_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small world with a floor row at y = 20 and solid walls on the
    /// sides (the `object_world`-style fixture).
    fn w() -> crate::World {
        let mut w = crate::World::new("t".into(), 60, 40);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for x in 0..60 {
            let i = w.index(x, 20);
            w.tiles[i] = crate::tile::Tile {
                kind: 1,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w
    }

    fn put(w: &mut crate::World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = crate::tile::Tile {
            kind,
            flags: crate::tile::FLAG_ACTIVE,
            ..Default::default()
        };
    }

    #[test]
    fn test_smart_1_a_torch_item_finds_the_floor_spot() {
        // `Step_Torch`: the held torch (item 8) re-points at the closest
        // empty cell above the floor row (`tileSolid && !tileSolidTop`
        // below).
        let mut world = w();
        put(&mut world, 30, 19, 1); // a solid block: not a torch spot
        let found = step_torch(&world, 8, -1, -1, (30.0 * 16.0, 19.0 * 16.0), 20, 10, 40, 19);
        let (fx, fy) = found.expect("the torch finds a spot");
        assert!(in_tile_bounds(fx, fy, 20, 10, 40, 19));
        // The spot must be empty and anchored to the floor row.
        let spot = world.tile(fx, fy).unwrap();
        assert!(!spot.active());
        assert!(world.tile(fx, fy + 1).is_some_and(|t| t.active()));
    }

    #[test]
    fn test_smart_1_a_non_torch_item_skips_the_step() {
        let world = w();
        assert_eq!(
            smart_cursor_lookup(&world, 1, -1, -1, (0.0, 0.0), 20, 10, 40, 19),
            None,
            "the pickaxe's lookup rides M3-ITEM's step"
        );
    }

    #[test]
    fn test_smart_1_an_existing_torch_blocks_the_box() {
        // The ±8 sweep: an existing torch within 8 tiles refuses the whole
        // box.
        let mut world = w();
        put(&mut world, 30, 19, TORCH_TILE);
        let found = step_torch(&world, 8, -1, -1, (30.0 * 16.0, 19.0 * 16.0), 25, 15, 35, 19);
        assert_eq!(
            found, None,
            "the no-torch-around sweep covers the reachable rect"
        );
    }

    #[test]
    fn test_smart_1_a_filled_cell_is_skipped() {
        // The occupied non-cut cell: solid stones in the rect leave only
        // the free row.
        let mut world = w();
        for x in 20..=40 {
            for y in 15..=19 {
                put(&mut world, x, y, 1);
            }
        }
        // The rect 20..=40 x 15..=19 is solid, but the reachable window
        // extends above it (y = 10..19): the empty rows 10-14 still offer
        // spots — the closest valid one is picked (the vanilla scans the
        // whole reachable rect, not just the filled pocket).
        let found = step_torch(&world, 8, -1, -1, (30.0 * 16.0, 19.0 * 16.0), 20, 10, 40, 19);
        let (fx, fy) = found.expect("the empty rows above the pocket are spots");
        assert!(
            fy < 15,
            "the picked spot sits above the filled block: {fy}"
        );
    }

    #[test]
    fn test_smart_1_the_wall_behind_the_cell_anchors_a_torch() {
        // `IsValidSpotForTorch`'s first arm: a wall behind the cell accepts
        // the torch even with no side/floor anchor.
        let mut world = crate::World::new("w".into(), 20, 20);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        let i = world.index(10, 10);
        world.tiles[i].wall = 1; // the dirt wall
        assert!(is_valid_spot_for_torch(&world, 10, 10, None));
        assert!(!is_valid_spot_for_torch(&world, 10, 11, None));
    }
}
