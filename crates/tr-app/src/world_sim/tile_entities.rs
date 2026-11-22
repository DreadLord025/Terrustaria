//! The tile-entity simulation pass (`TileEntity.PerformUpdates`, driven from
//! `WorldGen.cs:71507`'s `UpdateWorld`): the logic sensors are the first
//! `RequiresUpdates` half (M7-TE-2). The training dummy's state machine lives
//! in `tr_core::tile_entities` too — its NPC half waits for the roster's
//! `NPCID.TargetDummy` (488) landing with the M4-NPC families.

use crate::app::App;

use crate::player;
use crate::world_sim::wiring::stamp_wiring_entity_boxes;

/// Fixed 60 Hz TE stepping. The sensors scan their state; the trips fire
/// through `HitSwitch` after the scan (`UpdateEndInternal`'s order), the
/// PlayerAbove trips blocking the teleporter iteration for their hit.
pub(crate) fn update_tile_entities(app: &mut App, dt: f32) {
    // The player hitboxes the sensors and dummies scan (`FillPlayerHitboxes`
    // — the port has the single local player).
    let player_boxes: Vec<(i32, i32, i32, i32)> = app
        .player
        .as_ref()
        .filter(|p| !p.dead)
        .map(|p| {
            let (w, h) = (player::WIDTH, player::HEIGHT);
            vec![(p.x as i32, p.y as i32, w as i32, h as i32)]
        })
        .unwrap_or_default();
    app.te_step.tick(dt.min(0.25), |_| {
        let day = app.world.day_time;
        let boxes = player_boxes.clone();
        // The §8.1 stamp: the sensor trips fire `HitSwitch` after the scan
        // — a wired trapdoor/gate on the line reads the live entity boxes.
        stamp_wiring_entity_boxes(app);
        let world = &mut app.world;
        let wiring = &mut app.wiring;
        tr_core::tile_entities::update_logic_sensors(world, wiring, day, &boxes);
        // S1-E-21i — the sensors' Mech-28 clicks (the logic sensor's
        // `UpdateEndInternal` fires `HitSwitch`, `TileEntity.cs` via
        // `tr_core::tile_entities`): `TileEntity.PerformUpdates` runs from
        // `WorldGen.UpdateWorld` (`WorldGen.cs:71510`) under the
        // `UpdateWorldGen` swap (`Main.cs:17802-17805`), so the pitch draws
        // ride that stream — drained right after the scan, the trips'
        // play order matching the scan order.
        for (px, py) in app.wiring.take_switch_clicks() {
            app.audio.play_switch_click(
                px as f32,
                py as f32,
                app.rngs.stream(crate::app::RngKey::UpdateWorldGen),
            );
        }
        // W4-c42 final (§8.4 drain-union finish): the sensor trips' wired
        // door/trapdoor/gate cues ride the SAME UpdateWorldGen stream —
        // this site drained only the switch clicks, so a sensor-tripped
        // door cue lingered until the next stage's drain (wrong stream).
        for (open, px, py) in app.wiring.take_door_cues() {
            app.audio.play_door_sound(
                open,
                px as f32,
                py as f32,
                app.rngs.stream(crate::app::RngKey::UpdateWorldGen),
            );
        }
        // The §8.2 echoes: SP discards (the C# `SendData` netMode-0
        // return); the queue must still drain so it never lingers.
        let _ = app.wiring.take_door_echoes();
    });
    app.te_step.cap_backlog();
}

#[cfg(test)]
mod w4_c42_final_tests {
    //! W4-c42 FINAL — the §8.4 drain-union finish: the sensor trips'
    //! wired door cues play on the UpdateWorldGen stream HERE (the site
    //! drained only the switch clicks before — a sensor-tripped door cue
    //! lingered into the next stage's drain). WRITTEN NOT RUN (code-only).

    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};
    use tr_core::tile_entities::TeKind;
    use tr_core::wiring::WireColor;

    /// A night sensor (423, frameY 18) wired to an open door (11) at
    /// column 14. The world is DAY (the sensor Off); the pass flips the
    /// day flag to night → the sensor trips → the door closes → the close
    /// cue must drain in THIS pass (the queue is empty after), not linger.
    #[test]
    fn c42_the_sensor_trip_drains_its_door_cues_on_the_worldgen_stream() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut w = tr_core::world::World::new("t".into(), 64, 64);
        // The sensor at (10, 20), check = night (frameY 18). The entity's
        // check rides `figure_sensor_state` (the core rig's shape —
        // `place_unchecked` alone stores check 0).
        let i = w.index(10, 20);
        w.tiles[i] = Tile { kind: 423, frame_y: 18, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tile_entities.place_unchecked(TeKind::LogicSensor, 10, 20);
        let mut tes = std::mem::take(&mut w.tile_entities);
        let initial = tr_core::tile_entities::figure_sensor_state(&mut tes, &w, 10, 20, &[], true);
        w.tile_entities = tes;
        assert!(!initial, "day: the night sensor starts Off");
        // The open door pair at columns 14/15, rows 17-19.
        for row in 0..3i32 {
            for (col, fx) in [(14i32, 0i16), (15, 18)] {
                let k = w.index(col, 17 + row);
                w.tiles[k] = Tile { kind: 11, frame_x: fx, frame_y: (row * 18) as i16, ..Default::default() };
                w.tiles[k].flags |= FLAG_ACTIVE;
            }
        }
        // The red wire line from the sensor along row 20, then UP column
        // 15 into the door's leaf cell (15, 18) — the flood fires
        // `HitWireSingle` on every wired cell it reaches.
        for x in 10..=15 {
            w.wires.set(x, 20, WireColor::Red, true);
        }
        w.wires.set(15, 19, WireColor::Red, true);
        w.wires.set(15, 18, WireColor::Red, true);
        w.day_time = true; // day — the night sensor starts Off
        app.world = w;
        // Night falls: the sensor's state flips, the trip closes the door.
        app.world.day_time = false;
        update_tile_entities(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.tile(14, 18).unwrap().kind, 10, "the door closed");
        // The cue drained HERE (the §8.4 finish) — nothing lingers.
        assert!(app.wiring.take_door_cues().is_empty(), "the close cue drained in-pass");
        // The forced close's echo drained too (the §8.2 SP discard).
        assert!(app.wiring.take_door_echoes().is_empty());
    }
}
