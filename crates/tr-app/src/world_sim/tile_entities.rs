//! The tile-entity simulation pass (`TileEntity.PerformUpdates`, driven from
//! `WorldGen.cs:71507`'s `UpdateWorld`): the logic sensors are the first
//! `RequiresUpdates` half (M7-TE-2). The training dummy's state machine lives
//! in `tr_core::tile_entities` too — its NPC half waits for the roster's
//! `NPCID.TargetDummy` (488) landing with the M4-NPC families.

use crate::app::App;

use crate::player;

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
        let world = &mut app.world;
        let wiring = &mut app.wiring;
        tr_core::tile_entities::update_logic_sensors(world, wiring, day, &boxes);
    });
    app.te_step.cap_backlog();
}
