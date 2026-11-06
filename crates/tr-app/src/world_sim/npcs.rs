//! NPC simulation (plan stage A4): the fixed 60 Hz `Npc::update` step, the
//! death coin drops, the boss minion requests, the player respawn and the
//! `Spawner.SpawnNPC` ambient spawn attempt.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::achievements;
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

/// `AI_006_Worms` (`NPC.cs:50963-52090`) — the chain pass. The vanilla runs
/// it inside each NPC's `Update` against the `Main.npc` slots; the port runs
/// it here against the `Vec` (the slot order = the Vec order = the spawn
/// order, so the head moves before its followers exactly like the original).
/// The head digs (`flag3`'s solid-overlap scan switches the direct aim), the
/// followers ride the rigid trail (`:51867-51922`), a missing leader kills
/// the link (the chain death, `:51425-51433`). The EoW middle-segment split
/// (`:51511-51518`'s `Transform`) is deferred: a middle kill takes the tail
/// with it (the chain-death rule), the split lands with the boss-summon
/// work.
/// The town shop open (`NPC.cs`'s right-click dialog → `OpenShop`): the
/// press-edge cursor on a town NPC's rect opens the MERCHANT's shop (the
/// Guide offers none — M4-TOWN-3's finding). The stock and the happiness
/// multiplier capture at the open (`Main.OpenShop`'s `SetupShop` +
/// `ShopHelper.OpenShop` pair).
pub(crate) fn try_open_shop_at_cursor(app: &mut App, world_pos: Vec2) -> bool {
    // The town dispatch: the Merchant opens the shop, the Nurse heals (the
    // vanilla's dialog-button pair — the port collapses them onto the
    // right-click).
    let Some(nurse_idx) = app.npcs.iter().position(|n| {
        n.hp > 0
            && n.kind == NpcKind::Nurse
            && {
                let (w, h) = n.size();
                world_pos.x >= n.x
                    && world_pos.x < n.x + w
                    && world_pos.y >= n.y
                    && world_pos.y < n.y + h
            }
    }) else {
        return try_open_merchant_shop_at_cursor(app, world_pos);
    };
    app.nurse_heal(nurse_idx)
}

fn try_open_merchant_shop_at_cursor(app: &mut App, world_pos: Vec2) -> bool {
    let Some(idx) = app.npcs.iter().position(|n| {
        n.hp > 0
            && n.kind == NpcKind::Merchant
            && {
                let (w, h) = n.size();
                world_pos.x >= n.x
                    && world_pos.x < n.x + w
                    && world_pos.y >= n.y
                    && world_pos.y < n.y + h
            }
    }) else {
        return false;
    };
    let (pcx, pcy) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((0.0, 0.0));
    let mood = crate::happiness::process_mood(
        &app.world,
        &app.npcs,
        &app.npcs[idx],
        (pcx, pcy),
        false,
    );
    let flags = crate::shop::ShopFlags {
        hard_mode: false,
        night: !app.world.day_time,
        blood_moon: app.world.blood_moon,
        zone_snow: app.scene_metrics.zone_snow,
        zone_jungle: app.scene_metrics.zone_jungle,
        downed_boss2: false,
        downed_boss3: false,
    };
    let inv_ids: Vec<u16> = app
        .inventory
        .iter()
        .filter(|sl| sl.kind != 0 && sl.count > 0)
        .map(|sl| sl.kind)
        .collect();
    app.shop_stock = crate::shop::resolve_stock(1, &flags, &inv_ids);
    app.shop_mood = mood.multiplier;
    app.shop_npc = Some(idx);
    true
}

pub(crate) fn update_worms(app: &mut App) {
    let mut rng = app.item_rng;
    let world = &app.world;
    let player = app.player.as_ref().map(|p| (p.center().0, p.center().1));
    update_worms_in(world, player, &mut app.npcs, &mut rng, &mut app.next_worm_id);
}

/// The worm pass's core — the pieces the tests drive directly (the world,
/// the player centre, the Vec, the RNG stream, the id counter).
pub(crate) fn update_worms_in(
    world: &tr_core::World,
    player: Option<(f32, f32)>,
    npcs: &mut Vec<Npc>,
    mut rng: &mut u64,
    next_worm_id: &mut u32,
) {
    // The init loop: the links spawn their children (the chain assembles
    // front to back — the Vec order visits the head, then each child as it
    // lands).
    // The `while` (not the `for 0..len`) — the pushes extend the Vec and
    // the new links init in the same pass.
    let mut i = 0usize;
    while i < npcs.len() {
        if npcs[i].worm_inited || npcs[i].hp <= 0 || !npcs[i].kind.is_worm() {
            i += 1;
            continue;
        }
        // The chain identity: an id-less head takes the next id (the
        // segments share it — the vanilla's `ai[1]`/`ai[3]` slot links).
        if npcs[i].worm_id == 0 {
            npcs[i].worm_id = *next_worm_id;
            *next_worm_id += 1;
        }
        npcs[i].worm_inited = true;
        let kind = npcs[i].kind;
        // The tails (9/12/15) sit outside the chain-spawn lists
        // (`:51393`/`:51436`) — they end the chain.
        if !kind.is_worm_head() && !kind.is_worm_mid() {
            i += 1;
            continue;
        }
        let total = if kind.is_worm_head() {
            match kind {
                // `:51310`'s `Next(8, 13)` / `:51313`'s `Next(4, 7)` — the
                // inclusive-low exclusive-high C# ranges.
                NpcKind::DevourerHead => {
                    8 + (crate::app::frame::rand01(&mut rng) * 5.0) as u32
                }
                NpcKind::GiantWormHead => {
                    4 + (crate::app::frame::rand01(&mut rng) * 3.0) as u32
                }
                // `GetEaterOfWorldsSegmentsCount` (65; the expert 70 rides
                // the difficulty work).
                _ => 65,
            }
        } else {
            npcs[i].worm_left
        };
        if kind.is_worm_head() {
            npcs[i].worm_left = total;
        }
        // The child kind: the head spawns `type + 1`; the mid body spawns
        // itself while the budget holds, else the tail (`:51347-51358`).
        let child_kind = if kind.is_worm_head() {
            kind.worm_next()
        } else if npcs[i].worm_left > 0 {
            kind
        } else {
            kind.worm_next()
        };
        let (w, h) = npcs[i].size();
        let mut child = Npc::new(
            child_kind,
            npcs[i].x + w * 0.5 - child_kind.size().0 * 0.5,
            npcs[i].y + h,
            1,
            0.0,
        );
        child.worm_id = npcs[i].worm_id;
        child.seg_index = npcs[i].seg_index + 1;
        child.worm_left = npcs[i].worm_left.saturating_sub(1);
        child.hp = child.max_hp;
        npcs.push(child);
        i += 1;
    }
    // The follower trail (`:51867-51922`) + the chain death
    // (`:51425-51433`).
    for i in 0..npcs.len() {
        if !npcs[i].kind.is_worm() || npcs[i].seg_index == 0 || npcs[i].hp <= 0 {
            continue;
        }
        let (wid, seg) = (npcs[i].worm_id, npcs[i].seg_index);
        let Some(leader) = npcs
            .iter()
            .find(|n| n.worm_id == wid && n.seg_index == seg - 1 && n.hp > 0)
        else {
            npcs[i].hp = 0;
            continue;
        };
        let (lw, lh) = npcs[i].size();
        let dx = leader.x + leader.size().0 * 0.5 - (npcs[i].x + lw * 0.5);
        let dy = leader.y + leader.size().1 * 0.5 - (npcs[i].y + lh * 0.5);
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let num62 = (len - lw) / len;
        npcs[i].vx = 0.0;
        npcs[i].vy = 0.0;
        npcs[i].x += dx * num62;
        npcs[i].y += dy * num62;
        npcs[i].rotation = dy.atan2(dx) + 1.57;
    }
    // The heads: the dig/air machine (`:51923-52051`).
    let Some((pcx, pcy)) = player else { return };
    for i in 0..npcs.len() {
        if !npcs[i].kind.is_worm_head() || npcs[i].hp <= 0 {
            continue;
        }
        worm_head_step(&mut npcs[i], world, pcx, pcy, rng);
    }
}

/// The head's per-tick machine (`AI_006`'s `ai[1] == 0` branch). `flag3`
/// (the dig mode) = any tile in the body rect is solid or liquid-heavy
/// (`:51580`'s count rule). In the open air the head stumbles: the gravity
/// `+0.11` (the cap `num45`), the horizontal steering (`:51940-51972`); in
/// the ground it flies straight at the tile-snapped player (`:51997-52002`).
/// `noTileCollide` is family-wide — no collision blocks the head.
fn worm_head_step(
    npc: &mut Npc,
    world: &tr_core::World,
    pcx: f32,
    pcy: f32,
    _rng: &mut u64,
) {
    let (w, h) = npc.size();
    // `flag3`: the dig-mode scan (`:51575-51597`).
    let mut flag3 = false;
    let x0 = (npc.x / TILE_PX) as i32;
    let x1 = ((npc.x + w) / TILE_PX) as i32 + 1;
    let y0 = (npc.y / TILE_PX) as i32;
    let y1 = ((npc.y + h) / TILE_PX) as i32 + 1;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if world.tile(tx, ty).is_some_and(|t| {
                t.active() && (tr_core::tiledata::TILE_SOLID.has(t.kind) || t.liquid > 64)
            }) {
                flag3 = true;
            }
        }
    }
    let (num45, num46) = npc.kind.worm_tunables();
    if !flag3 {
        // The air stumble (`:51927-51973`).
        npc.vy = (npc.vy + 0.11).min(num45);
        if (npc.vx.abs() + npc.vy.abs()) < num45 * 0.4 {
            npc.vx += npc.facing as f32 * num46 * 1.1;
        } else if npc.vy == num45 {
            let target_vx = pcx - (npc.x + w * 0.5);
            if npc.vx < target_vx {
                npc.vx += num46;
            } else if npc.vx > target_vx {
                npc.vx -= num46;
            }
        } else if npc.vy > 4.0 {
            npc.vx -= npc.vx.signum() * num46 * 0.9;
        }
        npc.facing = if pcx >= npc.x + w * 0.5 { 1 } else { -1 };
    } else {
        // The dig (`:51974-52002`): the tile-snapped aim at the max speed.
        let mut dx = (((pcx / TILE_PX) as i32 - ((npc.x + w * 0.5) / TILE_PX) as i32) * 16) as f32;
        let mut dy = (((pcy / TILE_PX) as i32 - ((npc.y + h * 0.5) / TILE_PX) as i32) * 16) as f32;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let num67 = num45 / len;
        dx *= num67;
        dy *= num67;
        npc.vx = dx;
        npc.vy = dy;
    }
    // `noTileCollide` — the direct integration (`NPC.cs:90495-90498`).
    npc.x += npc.vx;
    npc.y += npc.vy;
    // The off-world guard (the framework shape).
    if npc.y > world.height as f32 * TILE_PX + 200.0 {
        npc.hp = 0;
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
                n.update_rng(world, player, step, &mut app.item_rng);
                black_slime_darkness_touch(n, player, &mut app.item_rng);
            }
        }
    });
    app.npc_step.cap_backlog();
    // The worm pass (`AI_006`): the chain movement after the per-NPC step
    // (the vanilla interleaves it in the slot order; the port's pass reads
    // the same spawn order).
    update_worms(app);
    // `EmoteBubble.UpdateAll` (`EmoteBubble.cs:46-63`): the bubble lifetimes
    // tick on the world update.
    app.emotes.update_all();
    // `NPC.UpdateHomeTileState`'s look-for-home shape (`NPC.cs:5739`'s
    // `QuickFindHome` rides): the homeless town NPCs scan every 2 s.
    app.town_look_timer += dt;
    if app.town_look_timer >= 2.0 {
        app.town_look_timer = 0.0;
        let world = &app.world;
        let npcs = &mut app.npcs;
        for n in npcs.iter_mut() {
            if n.kind != NpcKind::Guide || n.home.is_some() {
                continue;
            }
            let tx = (n.x / TILE_PX) as i32;
            let ty = (n.y / TILE_PX) as i32;
            if let Some((hx, hy)) = crate::town::quick_find_home(world, tx, ty) {
                // `MoveTownNPC`'s success arm (`WorldGen.cs:4848-4854`):
                // the home assigns, the homeless flag clears.
                n.home = Some((hx, hy));
            }
        }
    }
    // The town-NPC arrival (`NPC.SpawnAllowed_Merchant`, `NPC.cs:7175`):
    // the money condition + the absence check — the port's cadence rides
    // the 2-second town pass (the vanilla checks through UpdateWorld; the
    // dawn-gating lands with the clock events).
    if app.town_look_timer == 0.0 {
        if let Some(player) = app.player.as_ref() {
            if player.max_hp > 100
                && !app.npcs.iter().any(|n| n.kind == NpcKind::Nurse)
                && !app.npcs.iter().any(|n| n.kind == NpcKind::Merchant)
            {
                // The Nurse arrives when the life maximum grew past 100
                // (`SpawnAllowed_Nurse`) — the port gates her behind the
                // Merchant's absence (the vanilla arrival picks one town
                // NPC per pass).
                let xt = ((player.x / TILE_PX) as i32 - 6).clamp(2, app.world.width - 3);
                if let Some(mut n) = Npc::spawn_town(&app.world, xt, NpcKind::Nurse) {
                    n.home = crate::town::quick_find_home(
                        &app.world,
                        xt,
                        (n.y / TILE_PX) as i32,
                    );
                    app.npcs.push(n);
                }
            }
        }
    }
    if app.town_look_timer == 0.0 {
        if !app.npcs.iter().any(|n| n.kind == NpcKind::Merchant) {
            let inv: Vec<(u16, u32)> = app
                .inventory
                .iter()
                .filter(|s| s.kind != 0 && s.count > 0)
                .map(|s| (s.kind, s.count))
                .collect();
            if crate::spawner::spawn_allowed_merchant(&inv) {
                if let Some(player) = app.player.as_ref() {
                    let xt = ((player.x / TILE_PX) as i32 + 6).clamp(2, app.world.width - 3);
                    if let Some(mut m) = Npc::spawn_town(&app.world, xt, NpcKind::Merchant) {
                        m.home = crate::town::quick_find_home(
                            &app.world,
                            xt,
                            (m.y / TILE_PX) as i32,
                        );
                        app.npcs.push(m);
                    }
                }
            }
        }
    }
    let dead: Vec<Npc> = app.npcs.iter().filter(|n| n.hp <= 0).cloned().collect();
    app.npcs.retain(|n| n.hp > 0);
    for npc in &dead {
        // `Main.BestiaryTracker.Kills.RegisterKill` at the death's NPCLoot
        // (`NPC.cs:78835-78847`); the `GetWereThereAnyInteractions` gate is
        // constants-on — every death the port produces today is a player
        // kill, and no ported kind fails `IsNPCValidForBestiaryKillCredit`.
        let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);
        app.kill_tracker.register_kill(entry.credit_id);
        // `AchievementsHelper.NotifyNPCKilled` (`GameContent/Achievements/`):
        // the kill feeds the NPCKilledCondition rows (the M6-ACH-1 hook —
        // the original wire pass lost it to a mid-script assert).
        let _ = app
            .achievements
            .notify(achievements::CondKind::NpcKilled(entry.net_id));
        // `NPC.CountKillForBannersAndDropThem` (`NPC.cs:79818-79827`) →
        // `BannerSystem.AddNPCKillBy` (M2-BUFF-8): the milestone spawns the
        // banner item at the death spot and announces the tally in the
        // Command color.
        if let Some((banner, text)) =
            app.banners
                .add_npc_kill_by(entry.net_id, &app.player_name)
        {
            crate::chat::broadcast_death_text(
                &mut app.chat,
                &text,
                crate::chat::COLOR_COMMAND,
            );
            let item = tr_core::bestiary::banner_to_item(banner);
            crate::world_sim::items::spawn_item_drop(
                app,
                item as u16,
                1,
                (npc.x / 16.0) as i32,
                (npc.y / 16.0) as i32,
            );
        }
        // `NPC.checkDead`'s boss arms (`NPC.cs:79416`'s
        // `SetEventFlagCleared`): the boss deaths set the world's downed
        // flags. The player-kill gate rides the same kill path as the
        // bestiary credit above.
        match npc.kind {
            NpcKind::KingSlime => app.world.downed_slime_king = true,
            _ => {}
        }
        spawn_coins(app, &npc);
        spawn_loot(app, &npc);
    }
    // `AI_015_KingSlime`: landing bosses request a blue slime minion.
    // The casters' sphere spawn requests (`AI_008`'s `NPC.NewNPC` at the
    // shoot tick, `NPC.cs:21238`+): the update pass set `wants_sphere`,
    // this pass resolves it — the same borrow-safe shape as the minion
    // block below.
    let mut spheres = Vec::new();
    for n in app.npcs.iter_mut() {
        if let Some((kind, sx, sy)) = n.wants_sphere.take() {
            // `NPC.NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80771`): the payload
            // is the sphere's bottom-centre.
            let (w, h) = kind.size();
            let mut ball = Npc::new(kind, sx - w * 0.5, sy - h, 1, 0.0);
            ball.hp = ball.max_hp;
            spheres.push(ball);
        }
    }
    app.npcs.append(&mut spheres);

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
                underworld_for_bats: (app.camera.y / 16.0) as i32
                    > app.world.height - 200,
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

/// The full NPC loot through the rule engine
/// (`ItemDropResolver.TryDropping` over `ItemDropDatabase.Populate()`, ported
/// into `loot_db_gen`): the dead NPC's vanilla net id resolves its registered
/// rules; the executor skips requests for items the registry does not model.
fn spawn_loot(app: &mut App, npc: &Npc) {
    let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);
    let net_id = entry.net_id;
    let x = npc.x;
    let y = npc.y;
    // The drop rolls ride the app's item stream (`rand01`), so the gel keeps
    // its slot in the deterministic order.
    let mut rng_state = app.item_rng;
    let mut loot_rng = LootRng(&mut rng_state);
    let zones = &app.scene_metrics;
    // The drop rules read the killer's luck (`info.player.RollLuck`,
    // `CommonDrop.cs:36` — the port's single player).
    let mut info = tr_core::loot::DropAttemptInfo {
        rng: &mut loot_rng,
        luck: app.player.as_ref().map(|p| p.luck()).unwrap_or(0.0),
        is_expert: false,
        is_master: false,
        active_players: 1,
        extra_gel: false,
        flags: tr_core::loot::LootFlags {
            hard_mode: app.world.hard_mode,
            day_time: app.world.day_time,
            zone_snow: zones.zone_snow,
            zone_corrupt: zones.zone_corrupt,
            zone_crimson: zones.zone_crimson,
            ..Default::default()
        },
        npc: tr_core::loot::LootNpc {
            kind: if net_id < 0 { 1 } else { net_id },
            value: 1.0,
            life_max: npc.max_hp,
            friendly: false,
            spawned_from_statue: npc.spawned_from_statue,
            is_boss: npc.kind == NpcKind::KingSlime,
            has_player_target: true,
            ..Default::default()
        },
    };
    let mut out = Vec::new();
    tr_core::loot::try_dropping(crate::loot_db::database(), net_id, &mut info, &mut out);
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
        // `NPCLoot_DropMoney` reads `closestPlayer.luck` (`NPC.cs:79622`) —
        // the port's single player; the `|luck|` double-roll gate lives in
        // `drop_money` (`loot.rs:316-318`).
        app.player.as_ref().map(|p| p.luck()).unwrap_or(0.0),
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

    #[test]
    fn spawn_gel_and_coins_ride_the_player_luck() {
        // DW-3.1 — the drop rules read the killer's luck (`CommonDrop.cs:36`
        // through `info.luck`) and `NPCLoot_DropMoney` reads
        // `closestPlayer.luck` (`NPC.cs:79622`): at luck 1.0 both hit
        // branches always fire, so the item stream lands elsewhere than at
        // luck 0, while luck 0 stays byte-identical to the pre-wiring plain
        // rolls (the double-roll math itself is loot.rs-tested).
        use crate::app::flow::front_flow_tests::flow_app;
        let mut a = flow_app(None, vec![], vec![]);
        // A floor row gives the spawner a surface (flow_app's world is bare;
        // y 5 keeps top < height - 2 on the 8-tall world).
        for x in 0..a.world.width {
            let i = a.world.index(x, 5);
            a.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            a.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        a.world.compute_column_surface();
        a.player = Some(crate::player::Player::spawn(&a.world));
        let npc = Npc::spawn_slime_variant(
            &a.world,
            (a.player.as_ref().unwrap().x / 16.0) as i32,
            SlimeVariant::Blue,
        )
        .expect("the surface spawns a blue slime");

        let start = a.item_rng;
        spawn_coins(&mut a, &npc);
        spawn_loot(&mut a, &npc);
        let state_0 = a.item_rng;

        a.item_rng = start;
        a.player.as_mut().unwrap().luck_state.luck = 1.0;
        spawn_coins(&mut a, &npc);
        spawn_loot(&mut a, &npc);
        assert_ne!(a.item_rng, state_0, "luck 1.0 takes the hit branches");
    }
}
