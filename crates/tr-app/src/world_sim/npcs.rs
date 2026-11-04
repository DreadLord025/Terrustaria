//! NPC simulation (plan stage A4): the fixed 60 Hz `Npc::update` step, the
//! death coin drops, the boss minion requests, the player respawn and the
//! `Spawner.SpawnNPC` ambient spawn attempt.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::app::{rand01, App};
use crate::item::ItemDrop;
use crate::npc::{Npc, NpcKind, SlimeVariant};
use crate::regen::DotDeath;
use crate::registry;
use crate::spawner;

/// The player's epitaph: a DoT drain death uses its attribution's table text
/// (`KillMe(PlayerDeathReason.ByOther(N))` — `Player.cs:19321-19331`,
/// `:19362-19385`); every other death keeps the M2-DEATH-1 stub.
fn dot_death_text(pending: Option<DotDeath>) -> &'static str {
    pending.map_or("was slain...", crate::death::death_text_for)
}

/// `Player.StatusFromNPC` (`Player.cs:22333`), the black-slime row `:22591`:
/// `npc.type == 1 && npc.netID == -6` rolls `Next(4) == 0` for 900 ticks of
/// Darkness on touch. The roll rides the item stream — the port's
/// `Main.rand` stand-in (the M4-LOOT convention).
pub(crate) fn black_slime_darkness_touch(
    slime: &Npc,
    player: &mut crate::player::Player,
    item_rng: &mut u64,
) {
    if slime.kind == NpcKind::Slime
        && slime.slime == SlimeVariant::Black
        && slime.touches(player)
        && rand01(item_rng) < 0.25
    {
        player.buffs.add_buff(22, 900);
    }
}

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
                black_slime_darkness_touch(n, player, &mut app.item_rng);
            }
        }
    });
    app.npc_step.cap_backlog();
    // `EmoteBubble.UpdateAll` (`EmoteBubble.cs:46-63`): the bubble lifetimes
    // tick on the world update.
    app.emotes.update_all();
    let dead: Vec<Npc> = app.npcs.iter().filter(|n| n.hp <= 0).cloned().collect();
    app.npcs.retain(|n| n.hp > 0);
    for npc in &dead {
        // `Main.BestiaryTracker.Kills.RegisterKill` at the death's NPCLoot
        // (`NPC.cs:78835-78847`); the `GetWereThereAnyInteractions` gate is
        // constants-on — every death the port produces today is a player
        // kill, and no ported kind fails `IsNPCValidForBestiaryKillCredit`.
        let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);
        app.kill_tracker.register_kill(entry.credit_id);
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
            let death_text = dot_death_text(player.dot_death).to_string();
            player.dot_death = None;
            drop(player);
            // `Player.KillMe`'s chat broadcast: `{name} {text}` in the Death
            // color (`ChatColors.cs`); the ported monitor rides the App.
            crate::chat::broadcast_death(
                &mut app.chat,
                &app.player_name,
                &death_text,
            );
            crate::death::on_player_death(app, dx, dy, &death_text);
            if let Some(player) = app.player.as_mut() {
                player.start_death(boss_near, false);
            }
            log::info!("player died; the respawn timer is running");
        }
    }
    // `NPC.SpawnNPC` (`NPC.cs:80170-80176`): one ambient spawn call per 60 Hz
    // world update. The named `Main.SwapRandom("SpawnNPC")` stream
    // (`Main.cs:17886`) advances once per call regardless of the outcome, the
    // `noSpawnCycle` latch (`:80171-80174`) consumes a whole call, and the
    // `Spawner` pipeline (rate, caps, area scan, on-screen check) lives in
    // `spawner.rs`.
    app.npc_rng = app
        .npc_rng
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let mut spawned = None;
    if app.npcs.len() < 200 && app.npc_timer <= 0.0 {
        // `Spawner.CanSpawnEnemiesNear` (`NPC.cs:349-351`): a dead player
        // blocks the attempt (the journey and Moon Lord gates have no
        // sources yet).
        let ctx = app.player.as_ref().and_then(|player| {
            if player.dead {
                return None;
            }
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
            Some(spawner::SpawnContext {
                // The scene flags arrive as the systems land (hardmode
                // M5-HARDFORCE, blood moon M2-WEATHER, zones M5-BIOME); the
                // candle zones ride the M2-BUFF-5 scene scan
                // (`SceneMetrics.CalculateZones`, `:706-708`).
                flags: spawner::SpawnFlags {
                    zone_water_candle: app.scene_metrics.water_candle_count > 0,
                    zone_peace_candle: app.scene_metrics.peace_candle_count > 0,
                    ..spawner::SpawnFlags::default()
                },
                world: &app.world,
                player_center: (pcx, pcy),
                player_pos: (player.x, player.y),
                nearby_active_npcs: nearby,
                town_npcs: 0, // no town NPCs yet (M4-TOWN)
                day_time: app.world.day_time,
                player_life_max: app.player.as_ref().map_or(100, |p| p.max_hp),
            })
        });
        let mut rng = spawner::FastRandom::new(app.npc_rng);
        let (spawn, suppressed) = spawner::spawn_npc_call(ctx, &mut rng, app.spawn_no_cycle);
        app.spawn_no_cycle = false; // the latch is consumed by one call
        spawned = spawn;
        let _ = suppressed;
    }
    if let Some(n) = spawned {
        app.npcs.push(n);
    }
    app.npc_timer = 1.0 / 60.0;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::World;

    #[test]
    fn black_slime_darkness_touch_rolls_the_item_stream() {
        // `Next(4) == 0` ≈ `rand01 < 0.25`: a crafted seed pins both
        // outcomes; a non-black slime never rolls (the stream state proves
        // it — the deny seed under the blue slime leaves the rng untouched).
        let mut world = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 40..800 {
                let i = world.index(x, y);
                world.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        world.compute_column_surface();
        let mut player = crate::player::Player::spawn(&world);
        let px = player.x;
        let py = player.y;
        let mut black =
            Npc::spawn_slime_variant(&world, (px / 16.0) as i32, SlimeVariant::Black)
                .expect("the surface spawns a black slime");
        black.x = px;
        black.y = py;
        // The first rand01 of seed 1 — under the 0.25 gate?
        let mut rng = 1u64;
        let first = rand01(&mut rng);
        black_slime_darkness_touch(&black, &mut player, &mut 1u64);
        assert_eq!(
            player.buffs.active_types().contains(&22),
            first < 0.25,
            "seed 1 first roll {first}"
        );
        // A seed whose first roll is >= 0.25 denies the grant.
        let mut deny_seed = 2u64;
        while rand01(&mut deny_seed) < 0.25 {
            deny_seed += 1;
        }
        let mut player2 = crate::player::Player::spawn(&world);
        black_slime_darkness_touch(&black, &mut player2, &mut deny_seed);
        assert!(!player2.buffs.active_types().contains(&22));
        // The stream advances only when the touch rolls (a blue slime skips
        // the roll entirely — the same seed stays put).
        let blue = Npc::spawn_slime_variant(&world, (px / 16.0) as i32, SlimeVariant::Blue)
            .unwrap();
        let mut untouched = 7u64;
        black_slime_darkness_touch(&blue, &mut player, &mut untouched);
        assert_eq!(untouched, 7);
    }

    #[test]
    fn dot_death_text_selects_table_over_stub() {
        // DW-1.3 — the death collection resolves the DoT table text when the
        // drain left a reason (`Player.cs:19321-19331,19362-19385`) and keeps
        // the M2-DEATH-1 stub for any other death.
        assert_eq!(dot_death_text(None), "was slain...");
        assert_eq!(dot_death_text(Some(DotDeath::Poison)), "was poisoned.");
        assert_eq!(dot_death_text(Some(DotDeath::Burned)), "was burned.");
    }
}
