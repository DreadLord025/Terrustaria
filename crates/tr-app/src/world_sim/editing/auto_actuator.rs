//! The autoPaint + autoActuator build pass — the Actuation Accessory (3624)
//! auto-places an actuator on every newly placed solid block, the Paint
//! Sprayer (2216) / Architect Gizmo Pack (3061) / Hand Of Creation (5126)
//! auto-paint every newly placed tile. The C# pair:
//! `Player.PlaceThing_Tiles_GetAutoAccessoryCache` (`Player.cs:40460-40488`,
//! the pre-placement 11×11 snapshot) +
//! `PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate` (`:40594-40669`, BOTH
//! halves), called at the tail of `PlaceThing_Tiles_PlaceIt` (`:40163`).
//! The flags are granted by `grant_mechanical_accs` (`world_sim/player.rs`,
//! `Player.cs:12813-12820` + the held-item arms `:15271-15298`). The paint
//! writes/effects ride `auto_paint.rs` (`TryPainting`, `:41946-42024`).
//!
//! The block-swap consumer (`:39892/:39909`,
//! `PlaceThing_TryReplacingTiles`) is ported in `block_swap.rs` — it calls
//! THIS pass with the same cache (`:39909`).

use tr_core::tiledata as td;
use tr_core::{World, edit};

use crate::app::App;

use super::wiring::{ACTUATOR_ITEM, wire_placed_feedback};

/// One pre-placement cell — the C# `Vector3(tile.type, tile.frameX,
/// tile.frameY)` (`Player.cs:40477`). An in-bounds INACTIVE cell snapshots
/// `(-1, -1, -1)` (`:40481`); a null (out-of-world) cell keeps the array's
/// default `(0, 0, 0)` (`:40473`'s `tile != null` guard leaves it untouched).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CacheCell {
    kind: i32,
    frame_x: i32,
    frame_y: i32,
}

/// `Player.PlaceThing_Tiles_GetAutoAccessoryCache` (`Player.cs:40460-40488`):
/// the 11×11 snapshot centred on the placement target, taken BEFORE the
/// dispatcher writes (the C# calls it at `:40115`, ahead of
/// `TileObject.Place`/`WorldGen.PlaceTile`; the block-swap route takes its
/// own at `:39892`). The C# gates the allocation on
/// `autoPaint || autoActuator` (`:40463`); the caller gates the same way.
pub(crate) fn auto_accessory_cache(world: &World, tx: i32, ty: i32) -> [[CacheCell; 11]; 11] {
    let mut cache = [[CacheCell::default(); 11]; 11];
    for i in 0..11 {
        for j in 0..11 {
            let x = tx - 5 + i as i32;
            let y = ty - 5 + j as i32;
            // `:40473-40483`: null keeps the default; active snapshots the
            // type + frame; inactive snapshots (-1, -1, -1).
            if let Some(t) = world.tile(x, y) {
                cache[i][j] = if t.active() {
                    CacheCell {
                        kind: t.kind as i32,
                        frame_x: t.frame_x as i32,
                        frame_y: t.frame_y as i32,
                    }
                } else {
                    CacheCell {
                        kind: -1,
                        frame_x: -1,
                        frame_y: -1,
                    }
                };
            }
        }
    }
    cache
}

/// `Player.PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`
/// (`Player.cs:40594-40669`): after a successful placement, scan the window
/// around the target and auto-paint / auto-actuate every cell that NOW holds
/// the placed kind but did NOT hold it (or its exact frame) before. The
/// paint half runs FIRST per cell (`:40632-40635`), the actuator half after
/// (`:40636-40666`) — a cell can take both.
pub(crate) fn auto_actuate_after_place(
    app: &mut App,
    tx: i32,
    ty: i32,
    tile_to_create: u16,
    cache: &[[CacheCell; 11]; 11],
) {
    // `:40596-40599` — the flag gate: the C# returns when BOTH autoPaint and
    // autoActuator are off.
    let Some(p) = app.player.as_ref() else {
        return;
    };
    if !p.auto_paint && !p.auto_actuator {
        return;
    }
    // `:40632`/:40636-40639 — the builder toggles
    // (`BuilderAccToggleIDs.AutoPaint = 3`, `AutoActuate = 2`,
    // `Player.cs:53-55`). The C# checks them per cell; nothing in the pass
    // mutates them, so the port hoists each to one gate.
    let paint_on = p.auto_paint && p.builder_acc_status[3] == 0;
    let actuate_on = p.auto_actuator && p.builder_acc_status[2] == 0;
    // `:40600-40612` — the TARGET cell (read AFTER the placement) decides the
    // scan window and the same-kind compare mode.
    let Some(target) = app.world.tile(tx, ty).copied() else {
        return;
    };
    let platform = td::SET_PLATFORMS.has(target.kind);
    // `flag2` (`:40606`) — the frame-exact same-kind families. Every member
    // (chests, dressers, campfires, torches) is non-solid, so the frame
    // compare only ever gates the autoPaint half; kept verbatim.
    let same_frame_kind = (td::SET_BASIC_CHEST.has(target.kind)
        && td::SET_BASIC_CHEST.has(tile_to_create))
        || (td::SET_BASIC_CHEST_FAKE.has(target.kind)
            && td::SET_BASIC_CHEST_FAKE.has(tile_to_create))
        || (td::SET_BASIC_DRESSER.has(target.kind)
            && td::SET_BASIC_DRESSER.has(tile_to_create))
        || (td::SET_CAMPFIRES.has(target.kind) && td::SET_CAMPFIRES.has(tile_to_create))
        || (td::SET_TORCHES.has(target.kind) && td::SET_TORCHES.has(tile_to_create));
    // `flag3` (`:40607`) — flag2 OR the platform pair. NOTE the C# quirk at
    // `:40628`: the skip reads `(!flag3 && type == cached) || (flag2 && type
    // == cached && frames == cached)` — a platform pair sets flag3 but NOT
    // flag2, so NEITHER disjunct fires and an unchanged platform cell is
    // never skipped (the auto pass re-acts it every placement).
    let platform_pair = same_frame_kind || (platform && td::SET_PLATFORMS.has(tile_to_create));
    // `:40608-40612` — a non-frame-important target (or a platform) shrinks
    // the 11×11 scan to the single target cell (i = j = 5).
    let (lo, hi) = if !tr_core::tileobject::frame_important(target.kind) || platform {
        (5, 6)
    } else {
        (0, 11)
    };
    for i in lo..hi {
        for j in lo..hi {
            let x = tx - 5 + i as i32;
            let y = ty - 5 + j as i32;
            // `:40619-40623` — only active cells of the placed kind.
            let Some(t) = app.world.tile(x, y).copied() else {
                continue;
            };
            if !t.active() || t.kind != tile_to_create {
                continue;
            }
            // `:40624-40631` — the unchanged-cell skip (the quirk above).
            let c = cache[i][j];
            if (!platform_pair && t.kind as i32 == c.kind)
                || (same_frame_kind
                    && t.kind as i32 == c.kind
                    && t.frame_x as i32 == c.frame_x
                    && t.frame_y as i32 == c.frame_y)
            {
                continue;
            }
            // `:40632-40635` — the autoPaint half: `TryPainting(num5, num6,
            // paintingAWall: false, applyItemAnimation: false)`. The paint
            // source scan runs per cell (the C# re-finds after each burn).
            if paint_on {
                super::auto_paint::try_painting(app, x, y, false);
            }
            // `:40636-40639` — the actuator gate (hoisted above).
            if !actuate_on {
                continue;
            }
            // `:40640-40652` — the `Wiring.DeActive` solid gate
            // (`Wiring.cs:3201-3213`) WITHOUT the above-cell/`CanKillTile`
            // rule (`:3214` has no twin here): active + tileSolid +
            // !NotReallySolid, minus the {314, 379, 386-389, 476} exceptions.
            // The C# RE-READS `Main.tile[num5, num6]` here (`:40640`) — the
            // coating's SquareTileFrame cascade may have changed the cell.
            let Some(t) = app.world.tile(x, y).copied() else {
                continue;
            };
            let mut solid = t.active()
                && td::TILE_SOLID.has(t.kind)
                && !td::SET_NOT_REALLY_SOLID.has(t.kind);
            if matches!(t.kind, 314 | 379 | 386 | 387 | 388 | 389 | 476) {
                solid = false;
            }
            if !solid {
                continue;
            }
            // `:40657-40666` — `FindItem(849)` (`Player.cs:55972-55982`: the
            // first inventory slot holding one; the port's 50-slot grid —
            // the coin/ammo rows never hold actuators), `PlaceActuator`,
            // net 17 number 8, then burn one (empty → SetDefaults(0)).
            let Some(slot) = app
                .inventory
                .iter()
                .position(|s| s.kind == ACTUATOR_ITEM && s.count > 0)
            else {
                continue;
            };
            if !edit::place_actuator(&mut app.world, x, y) {
                continue;
            }
            // `WorldGen.PlaceActuator`'s sound 0 (`WorldGen.cs:58015`) — the
            // shared wiring feedback seam. The net 17/8 (`:40660`) has no
            // wire flavour in the port's edit kinds (the S1-P-i34 Actuator
            // tool arm's convention — the local tile state is the truth).
            wire_placed_feedback(app, x, y);
            app.inventory[slot].count -= 1;
            if app.inventory[slot].count == 0 {
                app.inventory[slot] = crate::item::EMPTY_SLOT;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{object_world, slot};
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::FLAG_ACTIVE;

    /// A player with the Actuation Accessory flag and `count` actuators in
    /// slot 5; the held item is a Stone Block stack (item 3 → tile 1).
    fn builder_app(actuators: u32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        let mut p = crate::player::Player::spawn(&app.world);
        p.auto_actuator = true;
        app.player = Some(p);
        app.inventory[app.selected] = slot(3, 10);
        if actuators > 0 {
            app.inventory[5] = slot(ACTUATOR_ITEM, actuators);
        }
        app
    }

    /// The build pass marks the freshly placed block and burns one actuator
    /// (`Player.cs:40657-40665`), through the real `place_tile` route.
    #[test]
    fn the_auto_actuator_pass_marks_the_placed_block_and_burns_one() {
        let mut app = builder_app(3);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        let t = app.world.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 1);
        assert!(
            t.flags & tr_core::tile::FLAG_ACTUATOR != 0,
            "the placed block carries the actuator bit"
        );
        assert_eq!(app.inventory[5].count, 2, "one actuator burned");
        assert_eq!(app.inventory[app.selected].count, 9, "the block burned too");
    }

    /// An unchanged neighbour keeps its actuator-free state: the snapshot
    /// kind equals the placed kind and flag3 is false (`:40628`'s first
    /// disjunct), so the cell is skipped.
    #[test]
    fn the_unchanged_neighbour_is_skipped() {
        let mut app = builder_app(3);
        // A stone block already at (11, 19) BEFORE the placement.
        let i = app.world.index(11, 19);
        app.world.tiles[i].kind = 1;
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert!(
            app.world.tile(11, 19).unwrap().flags & tr_core::tile::FLAG_ACTUATOR == 0,
            "the pre-existing block is unchanged — no actuator"
        );
        assert_eq!(app.inventory[5].count, 2, "only the target burned one");
    }

    /// The AutoActuate builder toggle off (`builderAccStatus[2] != 0`,
    /// `:40636`) skips the whole pass.
    #[test]
    fn the_toggle_off_skips_the_pass() {
        let mut app = builder_app(3);
        app.player.as_mut().unwrap().builder_acc_status[2] = 1;
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert!(app.world.tile(10, 19).unwrap().flags & tr_core::tile::FLAG_ACTUATOR == 0);
        assert_eq!(app.inventory[5].count, 3, "nothing burned");
    }

    /// No actuator in the inventory → the block places, the bit stays off
    /// (`FindItem(849) == -1`, `:40657-40658`).
    #[test]
    fn the_empty_inventory_places_no_actuator() {
        let mut app = builder_app(0);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert!(app.world.tile(10, 19).unwrap().flags & tr_core::tile::FLAG_ACTUATOR == 0);
    }

    /// The last actuator empties the slot (`SetDefaults(0)`, `:40662-40665`).
    #[test]
    fn the_last_actuator_empties_the_slot() {
        let mut app = builder_app(1);
        assert!(super::super::place_tile(&mut app, 10, 19, 1));
        assert_eq!(app.inventory[5], crate::item::EMPTY_SLOT);
    }

    /// The snapshot keeps the pre-placement state: active cells carry
    /// (kind, frameX, frameY), inactive cells (-1, -1, -1)
    /// (`Player.cs:40475-40482`).
    #[test]
    fn the_cache_snapshots_the_pre_placement_cell() {
        let mut app = builder_app(0);
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 1;
        // The C# snapshot tests `Tile.active()` (`Player.cs:40475`) — the RAW
        // active flag, NOT `nactive()`: an ACTIVE|INACTIVE cell still
        // snapshots its kind. The witness's empty cell therefore carries no
        // ACTIVE bit at all.
        let cache = auto_accessory_cache(&app.world, 10, 19);
        assert_eq!(cache[5][5], CacheCell { kind: -1, frame_x: -1, frame_y: -1 });
        // The floor below is active stone.
        assert_eq!(cache[5][6].kind, 1);
        // A later write never leaks into the snapshot.
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        assert_eq!(cache[5][5].kind, -1);
        // An ACTIVE|INACTIVE cell keeps its kind: `active()` is raw
        // (`Player.cs:40475`), so the framed-off flag does not blank it.
        app.world.tiles[i].flags |= tr_core::tile::FLAG_INACTIVE;
        assert_eq!(auto_accessory_cache(&app.world, 10, 19)[5][5].kind, 1);
    }

    /// The platform-pair quirk (`:40607/:40628`): an UNCHANGED platform cell
    /// is never skipped — flag3 true / flag2 false fires neither disjunct —
    /// so the auto pass re-acts it (platforms are tileSolid, not in the
    /// exception set). The platform target also shrinks the window to the
    /// single cell (`:40608-40612`).
    #[test]
    fn the_platform_pair_never_skips_the_unchanged_cell() {
        let mut app = builder_app(2);
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 19; // a platform already there
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        let cache = auto_accessory_cache(&app.world, 10, 19);
        auto_actuate_after_place(&mut app, 10, 19, 19, &cache);
        assert!(
            app.world.tile(10, 19).unwrap().flags & tr_core::tile::FLAG_ACTUATOR != 0,
            "the unchanged platform is re-acted (the flag3 quirk)"
        );
        assert_eq!(app.inventory[5].count, 1);
        // A second pass refuses: PlaceActuator returns false on the bit
        // already set (`WorldGen.cs:58013`), so nothing burns.
        auto_actuate_after_place(&mut app, 10, 19, 19, &cache);
        assert_eq!(app.inventory[5].count, 1, "the set bit blocks a second burn");
    }

    /// The DeActive exception kinds never auto-actuate (`:40641-40652`):
    /// a minecart track (314) is tileSolid but in the forced-false set.
    #[test]
    fn the_exception_kinds_stay_actuator_free() {
        let mut app = builder_app(2);
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 314;
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        // The snapshot saw air there (the track "just placed").
        let mut cache = auto_accessory_cache(&app.world, 10, 19);
        cache[5][5] = CacheCell { kind: -1, frame_x: -1, frame_y: -1 };
        auto_actuate_after_place(&mut app, 10, 19, 314, &cache);
        assert!(app.world.tile(10, 19).unwrap().flags & tr_core::tile::FLAG_ACTUATOR == 0);
        assert_eq!(app.inventory[5].count, 2, "nothing burned");
    }
}
