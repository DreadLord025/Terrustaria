//! NPC simulation (plan stage A4): the fixed 60 Hz `Npc::update` step, the
//! death coin drops, the boss minion requests, the player respawn and the
//! `Spawner.SpawnNPC` ambient spawn attempt.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::app::{App, rand01};
use crate::item::ItemDrop;
use crate::npc::{Npc, NpcKind, SlimeVariant};
use crate::registry;
use crate::spawner;

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
    let dead: Vec<Npc> = app.npcs.iter().filter(|n| n.hp <= 0).cloned().collect();
    app.npcs.retain(|n| n.hp > 0);
    for npc in &dead {
        spawn_coins(app, &npc);
        spawn_gel(app, npc.kind, npc.slime, npc.x, npc.y);
    }
    // `AI_015_KingSlime`: landing bosses request a blue slime minion.
    let mut minions = app.npcs.iter().filter(|n| n.kind == NpcKind::Slime).count();
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
        // `Player.KillMe`'s drop half (`Player.cs:38863`) fires once at the
        // death moment; the respawn TIMER then runs in `update_player`
        // (M2-DEATH-2's `UpdateDead` port) and revives on expiry.
        if player.hp <= 0 && !player.dead {
            let (dx, dy) = player.center();
            let boss_near = player.boss_hinders(&app.npcs);
            let death_text = "was slain...".to_string();
            drop(player);
            crate::death::on_player_death(app, dx, dy, &death_text);
            if let Some(player) = app.player.as_mut() {
                player.start_death(boss_near, false);
            }
            log::info!("player died; the respawn timer is running");
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
                // The scene flags arrive as the systems land (hardmode
                // M5-HARDFORCE, blood moon M2-WEATHER, zones M5-BIOME); the
                // chain runs with them false today.
                flags: spawner::SpawnFlags::default(),
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

/// The slime gel loot through the rule engine
/// (`ItemDropResolver.TryDropping` + the `ItemDropDatabase.cs:1072-1082`
/// registrations): the dead slime's vanilla net id resolves its registered
/// rules; the executor skips requests for items the registry does not model
/// (the Bladed Glove until its item data lands).
fn spawn_gel(app: &mut App, kind: NpcKind, slime: SlimeVariant, x: f32, y: f32) {
    if kind != NpcKind::Slime {
        return;
    }
    let net_id = match slime {
        SlimeVariant::Blue => crate::loot_db::BLUE_SLIME,
        SlimeVariant::Green => crate::loot_db::GREEN_SLIME,
        SlimeVariant::Pinky => crate::loot_db::PINKY,
        SlimeVariant::Baby => crate::loot_db::BABY_SLIME,
        SlimeVariant::Black => crate::loot_db::BLACK_SLIME,
        SlimeVariant::Purple => crate::loot_db::PURPLE_SLIME,
        SlimeVariant::Red => crate::loot_db::RED_SLIME,
        SlimeVariant::Yellow => crate::loot_db::YELLOW_SLIME,
    };
    // The drop rolls ride the app's item stream (`rand01`), so the gel keeps
    // its slot in the deterministic order.
    let mut rng_state = app.item_rng;
    let mut loot_rng = LootRng(&mut rng_state);
    let mut info = tr_core::loot::DropAttemptInfo {
        rng: &mut loot_rng,
        luck: 0.0,
        is_expert: false,
        is_master: false,
        active_players: 1,
        extra_gel: false,
    };
    let mut out = Vec::new();
    tr_core::loot::try_dropping(
        crate::loot_db::slime_database(),
        net_id,
        &mut info,
        &mut out,
    );
    app.item_rng = rng_state;
    for req in out {
        if req.item > 0 && crate::registry::get(req.item as u16).is_some() {
            let vx = (rand01(&mut app.item_rng) * 2.0 - 1.0) * 1.2;
            let vy = -1.5 - rand01(&mut app.item_rng);
            let phase = rand01(&mut app.item_rng) * std::f32::consts::TAU;
            app.items.push(ItemDrop::new(
                req.item as u16,
                req.stack as u32,
                x,
                y,
                vx,
                vy,
                phase,
            ));
        }
    }
}

/// The engine's RNG adapter over the app's deterministic item stream.
pub(crate) struct LootRng<'a>(pub(crate) &'a mut u64);

impl tr_core::loot::DropRng for LootRng<'_> {
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (crate::app::rand01(self.0) * (max - min) as f32) as i32
    }
    fn next_float(&mut self) -> f32 {
        crate::app::rand01(self.0)
    }
}

/// `NPC.NPCLoot_DropMoney` through the ported chain
/// (`tr_core::loot::drop_money`): the per-kind `NPC.value` base, the
/// ±20-75 % spread, the six bonus chains and the platinum/gold/silver/copper
/// split. Coin velocities use the app's item stream like the C# scatter.
fn spawn_coins(app: &mut App, npc: &Npc) {
    let (x, y) = npc.center();
    let mut rng_state = app.item_rng;
    let mut loot_rng = LootRng(&mut rng_state);
    let mut out = Vec::new();
    tr_core::loot::drop_money(
        &mut loot_rng,
        npc.value(),
        0.0,   // Player.luck: the buff engine lands with M2-BUFF-2
        false, // Midas
        false, // Main.bloodMoon
        0.0,   // NPC.extraValue (master-mode bump, not modelled)
        &mut out,
    );
    app.item_rng = rng_state;
    for req in out {
        let vx = (rand01(&mut app.item_rng) * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - rand01(&mut app.item_rng);
        let phase = rand01(&mut app.item_rng) * std::f32::consts::TAU;
        app.items.push(ItemDrop::new(
            req.item as u16,
            req.stack as u32,
            x,
            y,
            vx,
            vy,
            phase,
        ));
    }
}
