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
    // `!NPC.downedGolemBoss` (`:46927`): the temple wall refuses the wires
    // until the Golem is downed. The port has the Golem's parts but no
    // world's downed-Golem flag, so the arm stays closed — the vanilla's
    // pre-Golem state. The dual-dungeons temple check of the same gate
    // (`Main.dualDungeonsSeed`) has no ported seed.
    if t.wall == 87 {
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
            // The C#'s cutter order (`Player.cs:46851-46880`): the yellow
            // wire first, then the green, the blue and the red.
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
                    // in `Player.ItemCheck` (`Player.cs:46851-46880`), so the
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
            // `Player.cs:46887-46900`: the rod flips the tile's actuator
            // state — the tile must carry the actuator bit, and the lock
            // keeps the rod to one actuation per tick.
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
            // `WorldGen.PlaceActuator` (`Player.cs:46878-46883`) — the C#'s
            // branch consumes no stack.
            if !edit::place_actuator(&mut app.world, tx, ty) {
                return false;
            }
            wire_placed_feedback(app, tx, ty);
            true
        }
    }
}

/// The wiring tools' feedback: the C#'s `SoundEngine.PlaySound(0, i * 16,
/// j * 16)` (sound id 0 = "Dig_1") at the tile, and the net message 17 the
/// port has no wire flavour for (the local wire state is the truth).
fn wire_placed_feedback(app: &mut App, tx: i32, ty: i32) {
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

    /// The Actuator item turns the tile into an actuator (`WorldGen.
    /// PlaceActuator`, `WorldGen.cs:58011-58020`) and consumes no stack
    /// (`Player.cs:46878-46883`).
    #[test]
    fn the_actuator_item_marks_the_tile() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.inventory[app.selected] = slot(849, 1);
        assert!(use_wiring_tool(&mut app, 10, 19));
        let t = app.world.tile(10, 19).unwrap();
        assert!(t.flags & tr_core::tile::FLAG_ACTUATOR != 0, "the actuator bit");
        assert_eq!(app.inventory[app.selected].count, 1, "no stack consumed");
    }

    /// `Player.CanDoWireStuffHere` (`Player.cs:46920-46943`): the temple wall
    /// (87) refuses the wires while the Golem lives.
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
    }
}
