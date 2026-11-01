//! NPC simulation (plan stage A4): the fixed 60 Hz `Npc::update` step, the
//! death coin drops, the boss minion requests, the player respawn and the
//! `Spawner.SpawnNPC` ambient spawn attempt.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::item::ItemDrop;
use crate::npc::{Npc, NpcKind};
use crate::registry;
use crate::spawner;
use crate::app::{rand01, App};

pub(crate) fn update_npcs(app: &mut App, dt: f32) {
    app.attack_cd = (app.attack_cd - dt).max(0.0);
    app.npc_timer -= dt;
    // Fixed 60 Hz NPC simulation, like NPC.cs. Updating per rendered frame made
    // enemies move faster on high-refresh displays.
    let world = &app.world;
    let player = &mut app.player;
    let npcs = &mut app.npcs;
    app.npc_step.tick(dt.min(0.25), |step| {
        if let Some(player) = player {
            for n in npcs.iter_mut() {
                n.update(world, player, step);
            }
        }
    });
    app.npc_step.cap_backlog();
    let dead: Vec<(NpcKind, f32, f32)> = app
        .npcs
        .iter()
        .filter(|n| n.hp <= 0)
        .map(|n| {
            let (cx, cy) = n.center();
            (n.kind, cx, cy)
        })
        .collect();
    app.npcs.retain(|n| n.hp > 0);
    for (kind, x, y) in dead {
        spawn_coins(app, kind, x, y);
    }
    // `AI_015_KingSlime`: landing bosses request a blue slime minion.
    let mut minions = app
        .npcs
        .iter()
        .filter(|n| n.kind == NpcKind::Slime)
        .count();
    let mut spawned = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.wants_minion {
            n.wants_minion = false;
            if minions < 4 {
                let xt = ((n.x / TILE_PX) as i32 + n.facing * 2).clamp(2, app.world.width - 3);
                if let Some(m) = Npc::spawn_slime(&app.world, xt) {
                    spawned.push(m);
                    minions += 1;
                }
            }
        }
    }
    app.npcs.extend(spawned);
    if let Some(player) = app.player.as_mut() {
        if player.hp <= 0 {
            log::info!("player died; respawning at the world spawn");
            player.respawn();
            app.npcs.clear();
            app.npc_timer = 4.0;
            if app.follow_player {
                let (cx, cy) = player.center();
                app.camera = Vec2::new(cx, cy);
            }
        }
    }
    // `Spawner.SpawnNPC()`: one ambient spawn attempt per 60 Hz tick
    // (NPC.cs:289-306). The rate, caps, area scan and on-screen check are
    // the ported pipeline in `spawner.rs`.
    if app.npcs.len() < 200 && app.npc_timer <= 0.0 {
        let spawned = if let Some(player) = &app.player {
            let (pcx, pcy) = player.center();
            // `player.nearbyActiveNPCs` (NPC.cs:77902-77922): enemy NPCs
            // within the screen-size rect around the player, counted as
            // one slot each (`npcSlots == 1` for the implemented kinds).
            let nearby = app
                .npcs
                .iter()
                .filter(|n| {
                    let (cx, cy) = n.center();
                    (cx - pcx).abs() < spawner::S_WIDTH as f32 * 0.5
                        && (cy - pcy).abs() < spawner::S_HEIGHT as f32 * 0.5
                })
                .count() as f32;
            let ctx = spawner::SpawnContext {
                world: &app.world,
                player_center: (pcx, pcy),
                player_pos: (player.x, player.y),
                nearby_active_npcs: nearby,
                town_npcs: 0, // no town NPCs yet (M4-TOWN)
                day_time: app.world.day_time,
            };
            app.npc_rng = app
                .npc_rng
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let mut rng = spawner::FastRandom::new(app.npc_rng);
            spawner::try_spawn_an_npc(&ctx, &mut rng)
        } else {
            None
        };
        if let Some(n) = spawned {
            app.npcs.push(n);
        }
        app.npc_timer = 1.0 / 60.0;
    }
}

/// `Item.NewItem` coin drops: a few copper from small enemies, gold from the boss.
pub(crate) fn spawn_coins(app: &mut App, kind: NpcKind, x: f32, y: f32) {
    let roll = rand01(&mut app.item_rng);
    let (denom, count) = match kind {
        NpcKind::KingSlime => (2u16, 3 + (roll * 4.0) as u32),
        NpcKind::Zombie => {
            if roll < 0.2 {
                (1, 1)
            } else {
                (0, 2 + (roll * 30.0) as u32 % 4)
            }
        }
        NpcKind::Slime => {
            if roll < 0.05 {
                (1, 1)
            } else {
                (0, 1 + (roll * 20.0) as u32 % 3)
            }
        }
        NpcKind::DemonEye => (0, 1 + (roll * 10.0) as u32 % 2),
    };
    let vx = (rand01(&mut app.item_rng) * 2.0 - 1.0) * 1.2;
    let vy = -1.5 - rand01(&mut app.item_rng);
    let phase = rand01(&mut app.item_rng) * std::f32::consts::TAU;
    app.items.push(ItemDrop::new(
        registry::coin_id(denom as usize),
        count,
        x - 7.0,
        y - 7.0,
        vx,
        vy,
        phase,
    ));
}
