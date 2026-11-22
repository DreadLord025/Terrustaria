//! The actuation/wire tool family of the editing loop (S1-K-4d): the wrench
//! family, the wire cutter, the Actuator item and the Actuation Rod
//! (`Player.ItemCheck_UseWiringTools`, `Player.cs:46749-46920`), moved verbatim
//! out of `world_sim/editing.rs`; `use_wiring_tool` is re-exported from
//! `editing`, so the `update_editing` caller seam is unchanged.

use tr_core::{TILE_PX, edit};

use crate::app::App;

use super::super::items::spawn_item_drop;

/// `ItemID.Wire` (530) (`ID/ItemID.cs:2743`) — the resource the wiring
/// tools consume: the wrenches place wires and burn a Wire item from the
/// inventory, never the tool itself (`Player.cs:46790-46799`).
pub(crate) const WIRE_ITEM: u16 = 530;

/// `ItemID.Actuator` (849) (`ID/ItemID.cs`) — the item the wire cutter drops
/// when it removes an actuator (`WorldGen.KillActuator`, `WorldGen.cs:58030`)
/// and the resource the Actuator tool consumes on placement.
pub(crate) const ACTUATOR_ITEM: u16 = 849;

/// The wiring tools of `Player.ItemCheck_UseWiringTools`
/// (`Player.cs:46749-46920`) — the wrench family, the cutter and the
/// actuator.
///
/// The Multicolor Wrench (3625) is not in this set: the multi-tool needs
/// the `WiresUI` tool modes (`Player.cs:46903-46918`) and the mass wire
/// operation (`Wiring.MassWireOperation`, `Wiring.cs:419-453`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WiringTool {
    /// The wrench family: 509 red, 850 blue, 851 green, 3612 yellow.
    Place(tr_core::wiring::WireColor),
    /// 510 — the wire cutter.
    Cut,
    /// 849 — the actuator.
    Actuator,
    /// 3620 — the Actuation Rod.
    ActuationRod,
}

/// The tool kind → its action (`ItemID`, `ID/ItemID.cs:2701-2743/:3381-3385/
/// :8907`).
fn wiring_tool(kind: u16) -> Option<WiringTool> {
    use tr_core::wiring::WireColor;
    Some(match kind {
        509 => WiringTool::Place(WireColor::Red),
        850 => WiringTool::Place(WireColor::Blue),
        851 => WiringTool::Place(WireColor::Green),
        3612 => WiringTool::Place(WireColor::Yellow),
        510 => WiringTool::Cut,
        849 => WiringTool::Actuator,
        3620 => WiringTool::ActuationRod,
        _ => return None,
    })
}

/// `Player.CanDoWireStuffHere` (`Player.cs:46920-46943`): the wire may go on
/// any tile in the world except the unbreakable wall (350) and — while the
/// Golem still lives — the temple's own wall (87).
fn can_do_wire_stuff_here(app: &App, tx: i32, ty: i32) -> bool {
    if !app.world.in_bounds(tx, ty) {
        return false;
    }
    let Some(t) = app.world.tile(tx, ty) else {
        return false;
    };
    if t.wall == 350 {
        return false;
    }
    // `!NPC.downedGolemBoss` (`Player.cs:46932-46942`): the temple wall
    // refuses the wires until the Golem is downed. S1-D-3: the flag DOES
    // exist (`world.downed_golem`, the body's death chain writes it) -
    // the v1 comment claimed it did not and kept the arm closed forever.
    // The dual-dungeons temple check of the same gate
    // (`Main.dualDungeonsSeed`) has no ported seed.
    if !app.world.downed_golem && t.wall == 87 {
        return false;
    }
    true
}

/// `Player.ItemCheck_UseWiringTools` (`Player.cs:46749-46920`): the wrench
/// family places the wires, the cutter takes them off and the Actuator item
/// turns the tile into an actuator. The tools consume a Wire item (530) from
/// the inventory — never the tool itself.
///
/// Returns whether the click was consumed.
pub(crate) fn use_wiring_tool(app: &mut App, tx: i32, ty: i32) -> bool {
    let kind = app.inventory[app.selected].kind;
    let Some(tool) = wiring_tool(kind) else {
        return false;
    };
    if !can_do_wire_stuff_here(app, tx, ty) {
        return false;
    }
    match tool {
        WiringTool::Place(color) => {
            // The C#'s inventory scan for the Wire item (530)
            // (`Player.cs:46791-46798`) — the first slot that holds one.
            let Some(slot) = app
                .inventory
                .iter()
                .position(|s| s.kind == WIRE_ITEM && s.count > 0)
            else {
                return false;
            };
            if !edit::place_wire(&mut app.world, tx, ty, color) {
                return false;
            }
            app.inventory[slot].count -= 1;
            if app.inventory[slot].count == 0 {
                app.inventory[slot] = crate::item::EMPTY_SLOT;
            }
            wire_placed_feedback(app, tx, ty);
            true
        }
        WiringTool::Cut => {
            // The C#'s cutter order (`Player.cs:46855-46881`): the ACTUATOR
            // first (`KillActuator`, :46857), then the yellow wire, the green,
            // the blue and the red. Each successful kill is its own arm.
            if edit::kill_actuator(&mut app.world, tx, ty) {
                // `WorldGen.KillActuator`'s drop (`Item.NewItem(..., 849)`,
                // `WorldGen.cs:58030`) — the cutter runs in `Player.ItemCheck`
                // (`Player.cs:46857`), so the drop rides `UpdatePlayers`. The
                // five dust 50 (`:58032-58035`) are the visual seam (no dust
                // engine on this path — ANALYSIS "The wiring tools").
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                spawn_item_drop(&mut app.items, rng, ACTUATOR_ITEM, 1, tx, ty);
                wire_placed_feedback(app, tx, ty);
                return true;
            }
            use tr_core::wiring::WireColor;
            for color in [
                WireColor::Yellow,
                WireColor::Green,
                WireColor::Blue,
                WireColor::Red,
            ] {
                if edit::kill_wire(&mut app.world, tx, ty, color) {
                    // `WorldGen.KillWire`'s drop (`Item.NewItem(..., 530)`,
                    // `WorldGen.cs:58130`) — the cutter's tile interaction runs
                    // in `Player.ItemCheck` (`Player.cs:46862-46880`), so the
                    // drop rides the `UpdatePlayers` stream.
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    spawn_item_drop(&mut app.items, rng, WIRE_ITEM, 1, tx, ty);
                    wire_placed_feedback(app, tx, ty);
                    return true;
                }
            }
            false
        }
        WiringTool::ActuationRod => {
            // `Player.cs:46888-46901`: the rod flips the tile's actuator
            // state — the tile must carry the actuator bit, and the lock
            // keeps the rod to one actuation per HOLD (the lock clears only
            // on `releaseUseItem`, `Player.cs:18802-18804`, run at the
            // `update_player` seam — not per tick).
            let Some(inactive) = app.world.tile(tx, ty).map(|t| t.inactive()) else {
                return false;
            };
            let Some(p) = app.player.as_mut() else {
                return false;
            };
            // The C#'s gate (`Player.cs:46894`): the rod acts when the lock
            // is free or when the lock's setting matches the tile's current
            // state — the lock keeps one tile from flipping twice.
            if p.actuation_rod_lock && p.actuation_rod_lock_setting != inactive {
                return false;
            }
            if !app.wiring.actuate(&mut app.world, tx, ty) {
                return false;
            }
            let now_inactive = app.world.tile(tx, ty).map_or(inactive, |t| t.inactive());
            if now_inactive == inactive {
                return false;
            }
            p.actuation_rod_lock = true;
            // `Player.cs:46897`: the lock remembers the state the tile had
            // before the actuation.
            p.actuation_rod_lock_setting = inactive;
            true
        }
        WiringTool::Actuator => {
            // `WorldGen.PlaceActuator` (`Player.cs:46883-46886`). The branch
            // itself decrements nothing, but the Actuator IS `consumable`
            // (`Item.cs:11331`) and `ApplyItemTime` (:46885) sets
            // `itemTime == itemTimeMax`, so the generic consumable block
            // (`Player.cs:43280-43325`, same tick, after `ItemCheck_OwnerOnlyCode`)
            // burns one — not ranged/coin, `CanConsumeConsumableItem` true
            // (:5496), no `ForceConsumption` override (:413). The v1 port kept
            // the stack whole; that was WRONG.
            if !edit::place_actuator(&mut app.world, tx, ty) {
                return false;
            }
            let slot = &mut app.inventory[app.selected];
            slot.count = slot.count.saturating_sub(1);
            if slot.count == 0 {
                *slot = crate::item::EMPTY_SLOT;
            }
            wire_placed_feedback(app, tx, ty);
            true
        }
    }
}

/// The wiring tools' feedback: the C#'s `SoundEngine.PlaySound(0, i * 16,
/// j * 16)` (sound id 0 = "Dig_1") at the tile, and the net message 17 the
/// port has no wire flavour for (the local wire state is the truth). Shared
/// with the autoActuator build pass (`auto_actuator.rs`).
pub(super) fn wire_placed_feedback(app: &mut App, tx: i32, ty: i32) {
    app.audio
        .play_at("Dig_1", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
}

#[cfg(test)]
mod tests {
    use super::super::tests::{object_world, slot};
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::FLAG_ACTIVE;

    /// The Actuation Rod (3620) flips the tile's actuator state
    /// (`Player.cs:46887-46900` → `Wiring.Actuate`, `Wiring.cs:390-405`),
    /// and the rod's lock keeps it from flipping the same tile twice
    /// (`Player.cs:46894-46897`).
    #[test]
    fn the_actuation_rod_flips_the_actuator_tile() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.player = Some(crate::player::Player::spawn(&app.world));
        // A stone block carrying the actuator bit at (10, 19).
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 1;
        app.world.tiles[i].flags |= FLAG_ACTIVE | tr_core::tile::FLAG_ACTUATOR;
        app.inventory[app.selected] = slot(3620, 1);
        assert!(use_wiring_tool(&mut app, 10, 19), "the rod flips the tile");
        assert!(
            app.world.tile(10, 19).unwrap().inactive(),
            "the tile went off"
        );
        assert!(
            !use_wiring_tool(&mut app, 10, 19),
            "the rod's lock refuses the second flip"
        );
    }

    /// A tile without the actuator bit refuses the rod (`Wiring.cs:391-393`).
    #[test]
    fn the_actuation_rod_needs_an_actuator_tile() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.player = Some(crate::player::Player::spawn(&app.world));
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 1;
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        app.inventory[app.selected] = slot(3620, 1);
        assert!(!use_wiring_tool(&mut app, 10, 19));
        assert!(!app.world.tile(10, 19).unwrap().inactive());
    }

    /// The rod's lock is per-HOLD, not per-tick: `ResetEffects` clears it only
    /// while `releaseUseItem` rides (`Player.cs:18802-18804`). A held button
    /// keeps the lock across ticks; the release re-arms it. The v1 port cleared
    /// the lock every tick, letting a held rod re-flip each tick.
    #[test]
    fn the_rod_lock_survives_a_held_button_and_clears_on_release() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.player = Some(crate::player::Player::spawn(&app.world));
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 1;
        app.world.tiles[i].flags |= FLAG_ACTIVE | tr_core::tile::FLAG_ACTUATOR;
        app.inventory[app.selected] = slot(3620, 1);
        assert!(use_wiring_tool(&mut app, 10, 19), "first flip");
        assert!(app.player.as_ref().unwrap().actuation_rod_lock, "locked");
        // A tick with the button STILL held (`mine_released = false`): the lock
        // survives — the rod cannot flip again this hold.
        app.mine_down = true;
        app.mine_released = false;
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
        assert!(
            app.player.as_ref().unwrap().actuation_rod_lock,
            "the held button keeps the lock"
        );
        assert!(!use_wiring_tool(&mut app, 10, 19), "no second flip while held");
        // The button releases (`mine_released = true`): the next ResetEffects
        // clears the lock, re-arming the rod.
        app.mine_down = false;
        app.mine_released = true;
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
        assert!(
            !app.player.as_ref().unwrap().actuation_rod_lock,
            "the release cleared the lock"
        );
        assert!(use_wiring_tool(&mut app, 10, 19), "the rod flips again");
    }

    // --- the wiring tools (`ItemCheck_UseWiringTools`) -------------------

    /// The wrench places the red wire and burns one Wire item
    /// (`Player.cs:46789-46787` — the inventory scan and `PlaceWire`).
    #[test]
    fn the_wrench_places_a_wire_and_consumes_one_wire_item() {
        use tr_core::wiring::WireColor;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.inventory[app.selected] = slot(509, 1); // the Wrench
        app.inventory[10] = slot(WIRE_ITEM, 3); // three Wire items
        assert!(use_wiring_tool(&mut app, 10, 19));
        assert!(
            app.world.wires.get(10, 19, WireColor::Red),
            "the red wire is on the tile"
        );
        assert_eq!(app.inventory[10].count, 2, "one wire consumed");
        assert_eq!(app.inventory[app.selected].count, 1, "the tool stays");
    }

    /// Without a Wire item the wrench has nothing to burn (`Player.cs:46791`).
    #[test]
    fn the_wrench_without_a_wire_item_places_nothing() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.inventory[app.selected] = slot(509, 1);
        assert!(!use_wiring_tool(&mut app, 10, 19));
        assert!(!app.world.wires.get(10, 19, tr_core::wiring::WireColor::Red));
    }

    /// The four wrenches place their own colour (`ItemID`, `ItemID.cs:2701-
    /// 2743/:3381-3385/:8907`).
    #[test]
    fn each_wrench_places_its_own_colour() {
        use tr_core::wiring::WireColor;
        for (kind, color) in [
            (509u16, WireColor::Red),
            (850, WireColor::Blue),
            (851, WireColor::Green),
            (3612, WireColor::Yellow),
        ] {
            let mut app = flow_app(None, vec![], vec![]);
            app.world = object_world();
            app.inventory[app.selected] = slot(kind, 1);
            app.inventory[10] = slot(WIRE_ITEM, 1);
            assert!(use_wiring_tool(&mut app, 10, 19), "tool {kind}");
            assert!(app.world.wires.get(10, 19, color), "tool {kind}");
        }
    }

    /// The cutter takes the wire off and drops the Wire item
    /// (`WorldGen.KillWire`, `WorldGen.cs:58122-58140`).
    #[test]
    fn the_cutter_removes_the_wire_and_drops_it() {
        use tr_core::wiring::WireColor;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.world.wires.set(10, 19, WireColor::Red, true);
        app.inventory[app.selected] = slot(510, 1); // the Wire Cutter
        assert!(use_wiring_tool(&mut app, 10, 19));
        assert!(!app.world.wires.get(10, 19, WireColor::Red));
        assert!(
            app.items.iter().any(|d| d.kind == WIRE_ITEM),
            "the wire drops as an item"
        );
    }

    /// The cutter's FIRST arm is `KillActuator` (`Player.cs:46857`): a tile
    /// carrying both an actuator and a wire loses the ACTUATOR (dropping item
    /// 849, `WorldGen.cs:58030`), not the wire — the v1 port skipped this arm
    /// and cut the wire instead.
    #[test]
    fn the_cutter_removes_the_actuator_before_any_wire() {
        use tr_core::wiring::WireColor;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 1;
        app.world.tiles[i].flags |= FLAG_ACTIVE | tr_core::tile::FLAG_ACTUATOR;
        app.world.wires.set(10, 19, WireColor::Red, true);
        app.inventory[app.selected] = slot(510, 1);
        assert!(use_wiring_tool(&mut app, 10, 19));
        assert!(
            app.world.tile(10, 19).unwrap().flags & tr_core::tile::FLAG_ACTUATOR == 0,
            "the actuator came off"
        );
        assert!(
            app.world.wires.get(10, 19, WireColor::Red),
            "the wire stays — the actuator arm won"
        );
        assert!(
            app.items.iter().any(|d| d.kind == ACTUATOR_ITEM),
            "the actuator drops as item 849"
        );
        assert!(
            !app.items.iter().any(|d| d.kind == WIRE_ITEM),
            "no wire dropped"
        );
    }

    /// The cutter's wire order is yellow → green → blue → red
    /// (`Player.cs:46862-46880`): with several colours on one tile the yellow
    /// goes first.
    #[test]
    fn the_cutter_takes_the_yellow_wire_first() {
        use tr_core::wiring::WireColor;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        for c in [WireColor::Red, WireColor::Blue, WireColor::Green, WireColor::Yellow] {
            app.world.wires.set(10, 19, c, true);
        }
        app.inventory[app.selected] = slot(510, 1);
        assert!(use_wiring_tool(&mut app, 10, 19));
        assert!(!app.world.wires.get(10, 19, WireColor::Yellow), "yellow gone");
        assert!(app.world.wires.get(10, 19, WireColor::Red), "red stays");
        assert!(app.world.wires.get(10, 19, WireColor::Green), "green stays");
        assert!(app.world.wires.get(10, 19, WireColor::Blue), "blue stays");
    }

    /// The Actuator item turns the tile into an actuator (`WorldGen.
    /// PlaceActuator`, `WorldGen.cs:58011-58020`) and burns ONE from the stack:
    /// the branch (`Player.cs:46883-46886`) decrements nothing itself, but the
    /// Actuator is `consumable` (`Item.cs:11331`) and `ApplyItemTime` sets
    /// `itemTime == itemTimeMax`, so the generic consumable block
    /// (`Player.cs:43280-43325`) spends one the same tick. The v1 port kept the
    /// stack whole — that was WRONG.
    #[test]
    fn the_actuator_item_marks_the_tile_and_consumes_one() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.inventory[app.selected] = slot(849, 3);
        assert!(use_wiring_tool(&mut app, 10, 19));
        let t = app.world.tile(10, 19).unwrap();
        assert!(t.flags & tr_core::tile::FLAG_ACTUATOR != 0, "the actuator bit");
        assert_eq!(app.inventory[app.selected].count, 2, "one actuator consumed");
        // The last one empties the slot to air (`SetDefaults(0)`, `:43321-43324`).
        app.inventory[app.selected] = slot(849, 1);
        let j = app.world.index(11, 19);
        app.world.tiles[j].kind = 1;
        app.world.tiles[j].flags |= FLAG_ACTIVE;
        assert!(use_wiring_tool(&mut app, 11, 19));
        assert_eq!(app.inventory[app.selected].kind, 0, "the slot went to air");
    }

    /// `Player.CanDoWireStuffHere` (`Player.cs:46920-46943`): the temple wall
    /// (87) refuses the wires while the Golem lives - and ACCEPTS them once
    /// `downedGolemBoss` rides (S1-D-3: the flag exists; the v1 kept the arm
    /// closed forever).
    #[test]
    fn the_temple_wall_refuses_the_wires() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.inventory[app.selected] = slot(509, 1);
        app.inventory[10] = slot(WIRE_ITEM, 1);
        let i = app.world.index(10, 19);
        app.world.tiles[i].wall = 87;
        assert!(!use_wiring_tool(&mut app, 10, 19));
        assert!(!app.world.wires.get(10, 19, tr_core::wiring::WireColor::Red));
        // The Golem downed: the same wall now wires (`:46932`'s gate opens).
        app.world.downed_golem = true;
        assert!(use_wiring_tool(&mut app, 10, 19));
        assert!(app.world.wires.get(10, 19, tr_core::wiring::WireColor::Red));
    }
}
