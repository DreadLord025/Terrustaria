//! The wiring tick (plan stage A4): the `Wiring.UpdateMech` cooldown queue and
//! the teleport events' entity half (`Wiring.Teleport`, `Wiring.cs:3120-3178`).

use glam::Vec2;

use crate::app::App;
use crate::npc::Npc;
use crate::npc::NpcKind;

/// Fixed 60 Hz wiring stepping: the mech queue's timers/traps plus the
/// teleporter events computed by the last `TripWire` (`Wiring.cs:651-661`).
/// The engine half (`Wiring::update` + the pad pairs) lives in
/// `tr-core/src/wiring.rs`.
pub(crate) fn update_wiring(app: &mut App, dt: f32) {
    app.wiring_step.tick(dt.min(0.25), |_| {
        app.wiring.update(&mut app.world);
        // `Wiring.Teleport()` per registered pad pair (`Wiring.cs:3120-3178`):
        // every living player/NPC standing in a pad's 48×48 box moves to the
        // other pad. The `teleporting` re-entry flag and the MP section sync
        // are single-player no-ops.
        // `HitWireSingle`'s statue case (105): the engine parses the style,
        // runs `CheckMech(30/600)` and emits the spawn request; the app runs
        // the `MechSpawn` caps against its entity lists and constructs the
        // ported kinds (`NPC.MechSpawn`, `NPC.cs:7420`; `Item.MechSpawn`,
        // `Item.cs:49385`).
        for spawn in app.wiring.take_statue_spawns() {
            match spawn {
                tr_core::wiring::StatueSpawn::Npc { x, y, npc_id } => {
                    // The C# NPC id → the ported kind (the Blue Slime 1 and
                    // the Demon Eye 2; the rest arrive with M4-NPC-1's
                    // families).
                    // `NPC.MechSpawn`'s caps: 3 within 200 px, 6 within
                    // 600 px, 10 global (`NPC.cs:7420-7505`) — the C# ids of
                    // the ported kinds (Blue Slime 1, Demon Eye 2).
                    if !crate::wiring_stub::npc_mech_spawn(
                        app.npcs.iter().map(|n| (n.x, n.y, npc_id as u16)),
                        x as f32,
                        y as f32,
                    ) {
                        continue;
                    }
                    let spawned = match npc_id {
                        1 => Npc::spawn_slime_variant(
                            &app.world,
                            (x / 16) as i32,
                            crate::npc::SlimeVariant::Blue,
                        ),
                        2 => Npc::spawn_eye(&app.world, (x / 16) as i32).map(|mut n| {
                            n.x = x as f32;
                            n.y = y as f32;
                            n
                        }),
                        _ => None,
                    };
                    if let Some(mut npc) = spawned {
                        npc.spawned_from_statue = true;
                        app.npcs.push(npc);
                    }
                }
                tr_core::wiring::StatueSpawn::Item { x, y, item_id } => {
                    // `Item.MechSpawn` (`Item.cs:49385-49405`): the same caps
                    // over the dropped items of the same C# item id.
                    if !crate::wiring_stub::item_mech_spawn(
                        app.items.iter().map(|d| (d.pos.x, d.pos.y, d.kind)),
                        x as f32,
                        y as f32,
                    ) {
                        continue;
                    }
                    app.items.push(crate::item::ItemDrop {
                        kind: item_id as u16,
                        count: 1,
                        pos: Vec2::new(x as f32, y as f32),
                        vel: Vec2::ZERO,
                        age: 0.0,
                        phase: 0.0,
                    });
                }
            }
        }
        for (from, to) in app.wiring.take_teleport_events() {
            let ax = (from.0 * 16.0) as i32;
            let ay = (from.1 * 16.0) as i32 - 48;
            let bx = (to.0 * 16.0) as i32;
            let by = (to.1 * 16.0) as i32 - 48;
            let dx = (bx - ax) as f32;
            let dy = (by - ay) as f32;
            // `TeleporterHitboxIntersects` (`Wiring.cs:3181-3189`): the union
            // of the 48×48 box and the entity box fits both spans.
            let hits = |ex: f32, ey: f32, ew: f32, eh: f32| {
                let ux0 = ax.max(ex as i32);
                let uy0 = ay.max(ey as i32);
                let ux1 = (ax + 48).min((ex as f32 + ew) as i32);
                let uy1 = (ay + 48).min((ey as f32 + eh) as i32);
                ux1 >= ux0 && uy1 >= uy0
            };
            if !app.wiring.block_player_teleportation() {
                if let Some(p) = &mut app.player {
                    let px = p.x;
                    let py = p.y;
                    if !p.dead && hits(px, py, crate::player::WIDTH, crate::player::HEIGHT) {
                        p.teleport(px + dx, py + dy);
                    }
                }
            }
            // `Wiring.cs:3147-3157`: the NPC gates — `lifeMax > 5`, no bosses,
            // `noTileCollide` and the `TeleportationImmune` set (none of the
            // implemented kinds is immune; bosses = King Slime).
            for n in app.npcs.iter_mut() {
                if n.max_hp > 5 && n.hp > 0 && n.kind != NpcKind::KingSlime {
                    let nx = n.x;
                    let ny = n.y;
                    let (w, h) = n.size();
                    if hits(nx, ny, w, h) {
                        n.x = nx + dx;
                        n.y = ny + dy;
                    }
                }
            }
        }
    });
    app.wiring_step.cap_backlog();
}
