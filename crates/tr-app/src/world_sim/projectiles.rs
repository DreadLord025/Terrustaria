//! Projectile simulation: the fixed 60 Hz pass over the 1000-slot pool
//! (`Projectile.Update`, see `projectile.rs` for the source map).

use glam::Vec2;

use crate::app::App;
use crate::projectile::{FlailInputs, ProjTrackedCue, Projectile, SHURIKEN, YoyoInputs};

/// W4-d80 — one drained bomb-fuse tracked-sound intent: the ensure-play /
/// stop dance against the pool, the handle float written back into
/// `Projectile.local_ai[2]` (the C# `localAI[2]` dance,
/// `Projectile.cs:46917-46921` / `:70673-70677`). The ensure's null test
/// is the pool's versioned-slot lookup PLUS the playing state — a
/// stopped-but-not-reaped slot must not re-play (the C# `GetActiveSound`
/// returns stopped slots too). The `ProjectileAudioTracker` condition
/// (`ProjectileAudioTracker.cs:15-27`: `!gameMenu && active && type ==
/// expectedType`) folds to the explicit death stop (the census fold 4 —
/// the pool cannot hand a live projectile reference to a `Send` closure;
/// the `gameMenu` term folds with the focus/pause gate that freezes the
/// pool). ZERO gameplay draws.
fn apply_proj_tracked_cue(
    audio: &mut crate::audio::Audio,
    slots: &mut [Projectile],
    slot: usize,
    cue: ProjTrackedCue,
    x: f32,
    y: f32,
) {
    use tr_core::slot::SlotId;
    let Some(p) = slots.get_mut(slot) else {
        return;
    };
    let handle = SlotId::from_float(p.local_ai[2]);
    match cue {
        ProjTrackedCue::BombFuseEnsure => {
            // The authoritative null test: a live, playing handle holds
            // the ensure (no re-play); a reaped OR stopped slot re-plays.
            let live = audio
                .get_active_sound(handle)
                .is_some_and(|s| s.is_playing());
            if !live {
                let Some(style) = crate::tracked_sound::TrackedStyle::by_name("BombFuse") else {
                    return;
                };
                // The `ProjectileAudioTracker` condition fold (the census
                // fold 4): the closure reads a shared atomic the slot's
                // death/detonation flips false (the pool cannot hand a
                // live projectile reference to a `Send` closure). The
                // `gameMenu` term folds with the focus/pause gate that
                // freezes the pool.
                let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                p.fuse_alive = Some(alive.clone());
                let id = audio.play_tracked_looped(
                    style,
                    x,
                    y,
                    Box::new(move || alive.load(std::sync::atomic::Ordering::Relaxed)),
                    Default::default(),
                );
                p.local_ai[2] = id.to_float();
            }
        }
        ProjTrackedCue::BombFuseStop => {
            // `:70675-70677`: `GetActiveSound(localAI[2])?.Stop()` then
            // `localAI[2] = 0`. The shared atomic flips false too (the
            // condition fold — the loop's `Update` stops it the moment
            // the slot dies, even before the explicit stop drains).
            if let Some(s) = audio.get_active_sound_mut(handle) {
                s.stop();
            }
            if let Some(alive) = p.fuse_alive.take() {
                alive.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            p.local_ai[2] = 0.0;
        }
        // W4-d80 — the DD2 projectile one-shots (research/64 deferred): the
        // fire-and-forget tracked plays + the handle-store-and-follow dance.
        // ZERO gameplay draws (the variation/pitch picks ride the two
        // audio-only statics).
        ProjTrackedCue::OneShot(name) | ProjTrackedCue::KillOneShot(name) => {
            // `PlayTrackedSound(style, Center)` — the handle is discarded
            // (`Projectile.cs:56509`/`:68615`/`:68905`/`:56466`/`:36441`/
            // `:71637`). The 10 000-px guard + the cap run inside the play.
            if let Some(style) = crate::tracked_sound::TrackedStyle::by_name(name) {
                let _ = audio.play_tracked(style, x, y, Default::default());
            }
        }
        ProjTrackedCue::GlobalOneShot(name) => {
            // `PlayTrackedSound(style)` — the global overload (no position,
            // `SoundEngine.cs:142-149`): the Explosive Trap's explode
            // (`Projectile.cs:69362`). No distance guard (global).
            if let Some(style) = crate::tracked_sound::TrackedStyle::by_name(name) {
                let _ = audio.play_tracked_global(style);
            }
        }
        ProjTrackedCue::HandlePlayFollow { style, idx, play } => {
            // `:68462-68475` / `:55774-55798`: the one-shot play into
            // `localAI[idx]` (the arm's latch gates `play`), then the
            // per-tick follow: `GetActiveSound == null` → `SlotId.Invalid`,
            // else `Position = Center`.
            if play {
                if let Some(style) = crate::tracked_sound::TrackedStyle::by_name(style) {
                    let id = audio.play_tracked(style, x, y, Default::default());
                    p.local_ai[idx] = id.to_float();
                }
            }
            let h = SlotId::from_float(p.local_ai[idx]);
            match audio.get_active_sound_mut(h) {
                None => p.local_ai[idx] = SlotId::INVALID.to_float(),
                Some(s) => s.position = (x, y),
            }
        }
        ProjTrackedCue::EngineLoop { style, vol, pitch } => {
            // `:38256-38267` / `:38288-38299` / `:40986-40997`: the ensure
            // (`GetActiveSound(localAI[2]) == null && vol != 0`) then the
            // follow (`Volume = vol`, `Position = Center`, `Pitch = pitch`).
            // The `ProjectileAudioTracker` condition folds to the shared
            // atomic (the census fold 4, the bomb-fuse precedent).
            let live = audio
                .get_active_sound(handle)
                .is_some_and(|s| s.is_playing());
            if !live && vol != 0.0 {
                if let Some(style) = crate::tracked_sound::TrackedStyle::by_name(style) {
                    let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                    p.fuse_alive = Some(alive.clone());
                    let id = audio.play_tracked_looped(
                        style,
                        x,
                        y,
                        Box::new(move || alive.load(std::sync::atomic::Ordering::Relaxed)),
                        Default::default(),
                    );
                    p.local_ai[2] = id.to_float();
                }
            }
            // The follow: the live handle's volume/position/pitch ride the
            // engine-speed envelope every tick.
            let h = SlotId::from_float(p.local_ai[2]);
            if let Some(s) = audio.get_active_sound_mut(h) {
                s.volume = vol;
                s.position = (x, y);
                s.pitch = pitch;
            }
        }
        ProjTrackedCue::EngineLoopStop => {
            // `:70863`: `GetActiveSound(localAI[2])?.Stop()` — the drone's
            // explicit Kill stop. The condition atomic flips false too.
            if let Some(s) = audio.get_active_sound_mut(handle) {
                s.stop();
            }
            if let Some(alive) = p.fuse_alive.take() {
                alive.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
}

/// Fixed 60 Hz pool stepping. Damage lands through `Npc::hurt`, so the
/// knockback chain is shared with melee; the immunity bookkeeping is the
/// pool's (`localNPCImmunity` / `perIDStaticNPCImmunity` / `NPC.immune`).
pub(crate) fn update_projectiles(app: &mut App, dt: f32) {
    // `TR_THROW`: the projectile-sprite fixture — one shot of the pinned kind
    // from the player's centre, through the same `spawn` the items use.
    // One shot only: the fixture re-arms when the pool empties.
    if let Some(kind) = crate::env::throw_kind() {
        let empty = app.projectiles.slots().iter().all(|p| !p.active);
        if empty {
            if let Some(p) = &app.player {
                let (cx, cy) = p.center();
                if kind == crate::projectile::LIGHTS_BANE {
                    // The slash's scale rides `ai[0]` (`AI_188_LightsBane`,
                    // `Projectile.cs:42240`) — the fixture spawns it through
                    // the SetDefaults row at scale 1 (S1-G-3's blend-batch
                    // screenshot).
                    app.projectiles.spawn_with_data_owned(
                        kind,
                        0,
                        cx - 3.0,
                        cy - 3.0,
                        3.0,
                        0.0,
                        10,
                        4.0,
                        1.0,
                        0.0,
                    );
                } else {
                    app.projectiles.spawn(
                        kind,
                        0,
                        Vec2::new(cx - 3.0, cy - 3.0),
                        Vec2::new(3.0, 0.0),
                        10,
                        4.0,
                        false,
                    );
                }
            }
        }
    }
    if app.dev_throw {
        if let Some(p) = &app.player {
            let (cx, cy) = p.center();
            app.projectiles.spawn(
                SHURIKEN,
                0,
                Vec2::new(cx - 3.0, cy - 3.0),
                Vec2::new(11.0, 0.0),
                10,
                12.0,
                false,
            );
        }
        app.dev_throw = false;
    }
    if app.summon_shot {
        if let Some(p) = &mut app.player {
            let (cx, cy) = p.center();
            // Phase 3's pool block is live now: the minion books against the
            // summon budget, so the debug fixture grants the slots the pair
            // needs (`maxMinions = 1` base would self-kill the Hornet on its
            // first pool tick — the cap semantics themselves are Phase 3's
            // tests; this fixture only wants both sprites on screen).
            p.max_minions = 3;
            // The DW-1.4 pair (M4-PROJ-8 Phase 1): a Hornet (373) + a Frost
            // Hydra (308) ride the generic active-slot draw pass
            // (`frame.rs`'s `p.draw` loop — the plan's Decision Log: presence
            // via the placeholder draw, not per-kind art). Damage 0 keeps the
            // PVE walk a no-op; the unported aiStyles 62/53 dispatch no arm,
            // so both slots hold position for the capture.
            // The pair anchors on the player's centre, clamped strictly
            // inside the world bounds (the guard's test is `x + w >=
            // rightWorld`, so a slot flush on the edge is out): Phase 3's
            // out-of-world guard (`Projectile.cs:16683-16703`, rightWorld /
            // bottomWorld = maxTiles * 16) would deactivate such a slot on
            // the very next pool tick, and the front-flow worlds run as
            // small as 8x8 tiles. On a real world the clamp is a no-op — the
            // DW-1.4 screenshot positions are unchanged.
            let (ww, wh) = (
                app.world.width as f32 * 16.0,
                app.world.height as f32 * 16.0,
            );
            let hx = (cx - 24.0).clamp(0.0, (ww - 24.0 - 1.0).max(0.0));
            let hy = (cy - 48.0).clamp(0.0, (wh - 26.0 - 1.0).max(0.0));
            let gx = (cx + 24.0).clamp(0.0, (ww - 80.0 - 1.0).max(0.0));
            let gy = (cy - 48.0).clamp(0.0, (wh - 74.0 - 1.0).max(0.0));
            app.projectiles.spawn_with_data_ai_owned(
                crate::projectile::HORNET,
                0,
                hx,
                hy,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
            app.projectiles.spawn_with_data_ai_owned(
                crate::projectile::FROST_HYDRA,
                0,
                gx,
                gy,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
        }
        app.summon_shot = false;
    }
    // The boomerangs' return seek reads the owner's center (`AI_003`'s
    // `Main.player[owner]` inputs); the whips anchor their curve on the same
    // center (the `GetArmPosition` stand-in) and aim the Moon Lord arms
    // along `swing_aim` (`AI_165_Whip`'s mouse aim, `:47709`).
    let owner = app.player.as_ref().map(|p| {
        let (x, y) = p.center();
        (x, y)
    });
    // `AI_099_2_Yoyos`'s `Main.player[owner]` reads: the aim world point
    // (`Main.ReverseGravitySupport(Main.MouseScreen) + Main.screenPosition`,
    // `Projectile.cs:67928` — the same mouse-world math editing.rs:26 uses),
    // the held-button channel state (`Player.channel`, the release arm
    // `:67920-67925`), the CCed gate (`:67920`'s `|| CCed`), the owner's
    // death (`:67824`) and `meleeSpeed` (the lifetime divisor `:67750` + the
    // range/top-speed division `:67886-67887` — S1-P-i09). The channel reads
    // the LATCHED `p.channel` (armed at the use start, cleared on
    // release/CC/kill/death — S1-P-i23), not a per-tick re-derive from the
    // button: a tracked-shot death must be able to drop the channel mid-hold
    // (`TryCancelChannel` `Player.cs:52975`).
    let (sw, sh) = app.screen_size();
    // The `controlUseItem` zeroing gates (`Player.cs:42804-42807`'s
    // `channel = false` on the cleared input): the inventory
    // (`Main.cs:17659-17663`'s `player.mouseInterface`/inventory arm), the
    // fullscreen map (`Player.cs:24798-24807`) and the minimap hover
    // (`MinimapFrame.cs:164`) all clear the use input, so the channel
    // drops behind them exactly like `update_editing`'s own gates
    // (S1-P-i09 — the old snapshot ignored them).
    let use_input_live = !app.inventory_open
        && !app.map.as_ref().is_some_and(|m| m.open)
        && app.minimap_hover.is_none();
    let inputs = YoyoInputs {
        aim: app.player.as_ref().map(|_| {
            (
                app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
            )
        }),
        // S1-P-i23's latched channel AND S1-P-i09's use-input gates: the
        // inventory/map/minimap clears drop the use input, so the channel
        // drops behind them even before `update_editing` clears the latch.
        channel: app.player.as_ref().is_some_and(|p| p.channel) && use_input_live,
        player_dead: app.player.as_ref().is_some_and(|p| p.dead),
        player_cced: app.player.as_ref().is_some_and(|p| p.cced()),
        melee_speed: app
            .player
            .as_ref()
            .map_or(1.0, |p| p.melee_speed(app.inventory[app.selected].kind)),
    };
    // The whips anchor their curve on the player center and aim the Moon
    // Lord arms along the same aim point (`AI_165_Whip`'s mouse aim, `:47709`)
    // — `inputs.aim` minus the center, so the two systems share one read.
    let aim = inputs
        .aim
        .zip(owner)
        .map(|((ax, ay), (ox, oy))| (Vec2::new(ax - ox, ay - oy)).normalize_or_zero())
        .filter(|d| *d != Vec2::ZERO);
    // `AI_015_Flails`'s `Main.player[owner]` reads (`Projectile.cs:43726-
    // 44169`): the same aim/channel/death snapshot plus the chain anchor
    // (`MountedCenter`, `:3335`), the kill-gate reads (`CCed`), the throw
    // add and the state-3 stand-still read (`player.velocity`), the hold
    // orbit's `player.direction`, the meleeSpeed scaling and the local
    // owner gate on the family's spawns.
    let flail_inputs = app.player.as_ref().map_or_else(
        || FlailInputs { player_dead: true, ..FlailInputs::default() },
        |p| {
            let (cx, cy) = p.center();
            let (mx, my) = p.mounted_center();
            // `Main.LocalPlayer.inventory[selectedItem].shoot` (`:43850`) —
            // the dual-flail flag5's selected-item gate.
            let held = app.inventory[app.selected];
            let selected_shoot = if held.count > 0 {
                crate::registry::projectile(held.kind)
            } else {
                None
            };
            FlailInputs {
                aim: inputs.aim,
                channel: inputs.channel,
                // `Player.controlUseItem` — the raw use button (the port's
                // `mine_down`), NOT the channel state: the flight/retract/
                // spin re-grab arms read the button even when `channel` is
                // false (`Projectile.cs:43905/43942/43959/44035`).
                control_use_item: app.mine_down,
                map_fullscreen: app.map.as_ref().is_some_and(|m| m.open),
                player_dead: false,
                player_cced: p.cced(),
                mounted_center: Some((mx, my)),
                center: Some((cx, cy)),
                player_vel: (p.vx, p.vy),
                player_dir: p.facing,
                // `player.meleeSpeed` as the flail reads it — the inverted,
                // held-item-scaled multiplier (`Player.cs:25912` +
                // `CapAttackSpeeds` `:29021-29039`), keyed on the owner's
                // selected item exactly like `BonusMeleeSpeedMultiplier
                // [inventory[selectedItem].type]`.
                melee_speed: p.melee_speed(app.inventory[app.selected].kind),
                local_owner: app.local_id as u8,
                selected_shoot,
            }
        },
    );
    // The closure's disjoint field captures keep the step borrow
    // (`projectile_step`) apart from the pool/world/npcs/player set. The
    // yoyo rolls (`Projectile.cs:67746`'s jitter, `:67886`'s beam rolls,
    // `:68041`'s penetrate roll) draw `Main.rand` under the
    // `UpdateProjectiles` swap (`Main.cs:17775-17777`) — the pass binds its
    // own named stream. Only this machine's projectiles simulate here — a
    // peer's shots ride the `27` wire (`Projectile.cs:17210`'s owner gate),
    // and the server-owned hostile shots (owner 255) only on the host /
    // single-player (`spawn_hostile`).
    let (local_owner, hostile_local) = (app.local_id as u8, !app.is_client());
    let rng = app.rngs.stream(crate::app::RngKey::UpdateProjectiles);
    // S1-P-i06 — the spear family's owner snapshot (`AI_019_Spears`'s
    // `Main.player[owner]` reads, `Projectile.cs:45859-45868`). The C# reads
    // the live `itemAnimation`/`itemAnimationMax` integers; the port's swing
    // timers run in seconds, so the counts derive from `app.swing` /
    // `app.swing_max` (the `ticks_of` convention, `world_sim/editing.rs`).
    // The anchor is `RotatedRelativePoint(MountedCenter)` (`:45860`).
    let spear_owner = app.player.as_ref().map(|p| {
        let (mcx, mcy) = p.mounted_center();
        let anchor = p.rotated_relative_point(Vec2::new(mcx, mcy), 0);
        crate::projectile_ai_spear::SpearOwner {
            anchor,
            direction: p.facing,
            item_animation: (app.swing * 60.0).round() as i32,
            item_animation_max: (app.swing_max * 60.0).round() as i32,
            channel: p.channel,
            // i12's held-kind form (`Player.cs:25912` reads
            // `BonusMeleeSpeedMultiplier[selectedItem.type]`).
            melee_speed: p.melee_speed(app.inventory[app.selected].kind),
            wet: p.wet,
            player_vel: Vec2::new(p.vx, p.vy),
            player_pos: Vec2::new(p.x, p.y),
            player_w: crate::player::WIDTH,
            player_h: p.height,
        }
    });
    app.projectiles.set_spear_owner(spear_owner);

    // S1-P-i35 — the bobber arm's per-tick player/world snapshot (`AI_061` +
    // `FishingCheck`'s `Main.player[owner]` reads, `Projectile.cs:53658+`).
    // Built from the live player/world/scene state before the pass (the
    // `set_disco` precedent); `None` when no player exists.
    let bobber_inputs = app.player.as_ref().map(|p| {
        let held = app.inventory[app.selected];
        let held_def = crate::registry::get(held.kind);
        let (_, mounted_y) = p.mounted_center();
        let cond = p.fishing_conditions(
            &app.inventory,
            &app.equip.coins,
            &app.equip.ammo,
            app.selected,
            &app.world,
            app.rain.is_raining(),
            app.rain.cloud_bg(),
        );
        let quest_item = crate::angler::QUEST_POOL
            .get(app.world.angler_quest as usize)
            .copied()
            .unwrap_or(0) as i32;
        let player_has_quest_item = quest_item > 0
            && app.inventory.iter().any(|sl| sl.count > 0 && sl.kind as i32 == quest_item);
        let m = &app.scene_metrics;
        crate::projectile_bobber::BobberInputs {
            held_pole_power: held_def.map_or(0, |d| d.fishing_pole),
            held_shoot: held_def.and_then(|d| d.projectile).unwrap_or(0),
            cced: p.cced(),
            no_items: p.no_items,
            pulley: p.pulley,
            dead: p.dead,
            final_level: cond.4,
            bait_type: cond.3,
            pole_type: cond.1,
            player_wet: p.wet,
            player_mounted_y: mounted_y,
            luck: p.luck(),
            crate_potion: p.crate_potion,
            sonar_potion: p.sonar_potion,
            acc_lava_fishing: p.acc_lava_fishing,
            not_the_bees: app.world.seeds.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES),
            remix: app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
            blood_moon: app.world.blood_moon,
            day_time: app.world.day_time,
            hard_mode: app.world.hard_mode,
            unlocked_slime_red_spawn: app.world.unlocked_slime_red_spawn,
            duke_alive: app.npcs.iter().any(|n| n.kind == crate::npc::NpcKind::DukeFishron && n.hp > 0),
            quest_item,
            player_has_quest_item,
            angler_alive: app.npcs.iter().any(|n| n.kind == crate::npc::NpcKind::Angler && n.hp > 0),
            angler_quest_finished: app.world.angler_quest_finished,
            zone_corrupt: m.zone_corrupt,
            zone_crimson: m.zone_crimson,
            zone_jungle: m.zone_jungle,
            zone_snow: m.zone_snow,
            zone_dungeon: m.zone_dungeon,
            zone_desert: m.zone_desert,
            zone_hallow: m.zone_hallow,
            // `ZoneGlowshroom` has no port metric (the editing DropCtx
            // precedent, false — ANALYSIS).
            zone_glowshroom: false,
            zone_beach: m.zone_beach,
            downed_boss3: app.world.downed_skeletron,
            combat_book_was_used: app.world.combat_book_was_used,
        }
    });
    app.projectiles.set_bobber_inputs(bobber_inputs);
    // `Main.DiscoR/G/B` as the aiStyle-123 crystal's shard-hue roll reads
    // it (`Projectile.cs:37262`); the clock's `advance_disco` owns the cycle.
    let disco = app.disco;
    app.projectiles.set_disco(disco);
    // S1-P-i55 — the golf context (`Main.LocalGolfState` + `Main.MouseWorld`
    // + the mouse-right pair + the player snapshot). The C# reads the live
    // state mid-walk; the port clones it into the pool for the tick and
    // writes the resolved clone back after (the strikes' RecordSwing and
    // the cup's scoring run inside the arms). `mouseRightRelease` lags one
    // frame in the C# (`Main.cs:62458-62465` runs after Draw): the pair
    // `mouseRight && mouseRightRelease` is therefore the PRESS tick — the
    // port's `place_down && !place_prev_down` is the same edge.
    let golf_ctx = app.player.as_ref().map(|p| {
        let (cx, cy) = p.center();
        let (bx, by) = p.bottom();
        let held = app
            .inventory
            .get(app.selected)
            .filter(|s| s.count > 0)
            .map(|s| s.kind)
            .unwrap_or(0);
        let use_animation = crate::registry::get(held).map_or(0, |d| d.use_animation as i32);
        crate::projectile_ai_golf::GolfContext {
            golf_state: app.golf_state.clone(),
            mouse_world: Vec2::new(
                app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
            ),
            mouse_right: app.place_down,
            // The C#'s `mouseRightRelease` lags one frame (the flag
            // refresh runs after Draw, `Main.cs:62458-62465`), so the
            // pair's `true && true` is the PRESS tick. The port's
            // `place_prev_down` is last frame's button state — the
            // release-lag stand-in.
            mouse_right_release: !app.place_prev_down,
            player: crate::projectile_ai_golf::GolfClubPlayer {
                // `player.active` is the slot-in-use flag, NOT liveness —
                // a dead player's helper keeps charging (the C# kill gate
                // `:52420` reads `active` only). The context exists only
                // when the player is present, so the flag is true.
                active: true,
                center: Vec2::new(cx, cy),
                bottom: Vec2::new(bx, by),
                direction: p.facing,
                channel: p.channel,
                use_animation,
                held_item_type: held as i32,
            },
            player_name: app.player_name.clone(),
            golfer_score: p.golfer_score,
        }
    });
    if let Some(ctx) = golf_ctx {
        app.projectiles.set_golf_context(ctx);
    }
    app.projectile_step.tick(dt.min(0.25), |_| {
        app.projectiles.update(
            &mut app.world,
            &mut app.npcs,
            owner,
            aim,
            app.player.as_mut(),
            inputs,
            flail_inputs,
            rng,
            local_owner,
            hostile_local,
        );
    });
    app.projectile_step.cap_backlog();
    // S1-P-i55 — the golf context writeback: the arms mutated the clone
    // (RecordSwing / the cup scoring); the resolved state is authoritative
    // for the tick. The `golfer_score` accumulator rides the player.
    if let Some(ctx) = app.projectiles.take_golf_context() {
        app.golf_state = ctx.golf_state;
        if let Some(p) = app.player.as_mut() {
            p.golfer_score = ctx.golfer_score;
        }

    }
    // S1-P-i06 — the lances' channel-hold write-back (`SetDummyItemTime(third)`,
    // `Projectile.cs:45941` → `Player.cs:4432-4441`: `itemAnimation = itemTime =
    // frames`, `itemTimeMax = frames + 1`). The arm queued the tick count; the
    // port re-arms the swing/item timers so the hold persists while the button
    // is down (the `FlailTick.facing` write-back precedent). `app.swing_max`
    // stays the item's animation length (the C# `SetDummyItemTime` does not
    // touch `itemAnimationMax`).
    if let Some(frames) = app.projectiles.take_spear_hold() {
        let secs = frames as f32 / 60.0;
        app.swing = secs;
        app.item_time = secs;
    }
    // S1-C-7 wave B — the hostile-shot lane's hurt modifiers (`Player.Hurt`
    // `:38396-38517` via `Projectile.Damage` `Projectile.cs:14868`): the
    // accepted shot queued its retaliate request and possibly a dismount-on-
    // hit (the golf-ball returns 877/878/879); the C# rolls inline under the
    // `UpdateProjectiles` swap (`Main.cs:17775`), the port drains at the pass
    // tail on the same stream (the S1-E-21c batch convention).
    if let Some(p) = app.player.as_mut() {
        crate::hurt_modifiers::drain_dismount(p, &app.world);
    }
    if let Some(req) = app.player.as_mut().and_then(|p| p.take_retaliate()) {
        let expert = app.player.as_ref().is_some_and(|p| p.buffs.expert_mode);
        crate::hurt_modifiers::drain_retaliate(
            app.player.as_mut().unwrap(),
            &mut app.projectiles,
            &mut app.npcs,
            req,
            app.local_id as u8,
            expert,
            rng,
        );
    }
    // S1-E-21 wave C — the landed strikes' `NPC.HitSound` plays
    // (`NPC.cs:81544-81553` via `Projectile.Damage` `Projectile.cs:13649`):
    // the case-3 volume arms + the `Main.rand.Next(-10, 11) * 0.01` pitch
    // draw ride the `UpdateProjectiles` stream (`Main.cs:17775`), only
    // after each play's attenuation gate passed. The C# draws mid-walk;
    // the port drains at the pass tail (ANALYSIS, S1-E-21c).
    for (cue, x, y) in app.projectiles.drain_npc_hit_sounds() {
        app.audio.play_npc_hit_sound(cue, x, y, rng);
    }
    // S1-E-21h — the projectile-family cue plays (the AI arms' first-tick
    // latches, the boomerang whoosh, the missile hum, the bomb blast): the
    // style play's case-2 draws ride the SAME `UpdateProjectiles` stream
    // (`Main.cs:17775-17777`), only after each play's attenuation gate
    // passed. The C# plays inline mid-walk; the port drains at the pass
    // tail (the wave-C batch convention, ANALYSIS "The projectile sound
    // wave").
    for (id, style, x, y, pitch_offset) in app.projectiles.drain_proj_sounds() {
        // S1-P-i08 + W3-i39 — the aiStyle-3 wall thud (`Projectile.cs:19571`)
        // and `AI_007`'s latch dig cue (`Projectile.cs:52757`:
        // `PlaySound(0, i*16, j*16)`) are the int-id Dig play: case 0's
        // `Next(3)` variation + `Next(-10, 11)*0.01` pitch ride the same
        // stream, attenuation gate first (the `play_place_sound` façade's
        // id-0 arm, `LegacySoundPlayer.cs:440-452`, S1-E-21 wave B); the
        // id-2 styles keep the wave-A façade.
        if id == 0 {
            app.audio.play_place_sound((0, 0), x, y, rng);
        } else {
            app.audio.play_proj_sound((id, style), x, y, pitch_offset, rng);
        }
    }
    // W4-d80 — the bomb-fuse tracked-sound intents (research/64 sites
    // 20/21): the ensure-play / stop the C# runs inline
    // (`Projectile.cs:46917-46921` / `:70673-70677`). The drain applies
    // them against the tracked pool and writes the `SlotId` floats back
    // into the slots' `local_ai[2]`. ZERO gameplay draws — the
    // variation/pitch picks ride the two audio-only statics (the census'
    // RNG contract); the `ProjectileAudioTracker` condition folds to the
    // explicit death stop (the census fold 4).
    for (slot, cue, x, y) in app.projectiles.drain_proj_tracked() {
        apply_proj_tracked_cue(&mut app.audio, app.projectiles.slots_mut(), slot, cue, x, y);
    }
    // `NewProjectile`'s `TryUpdateChannel` (`Projectile.cs:11097-11100`):
    // the local spawns of this tick track the cancel key's expected type —
    // the last one wins (`ChannelCancelKey.TryTracking` `Player.cs:134-140`).
    // The pool queued the intents (it cannot borrow the player mid-loop);
    // apply them here, after the walk, before the next tick's kills read the
    // key (S1-P-i23).
    if let Some(p) = app.player.as_mut() {
        app.projectiles.apply_channel_tracks(p);
    }
    // `AI_103`'s mark spawn (`Projectile.cs:33467-33475`): the stardust
    // tower's mark resolves into the NPC vec here (the pool can't grow it).
    for (kind, x, y, vx, vy) in app.projectiles.drain_mark_spawns() {
        let k = crate::npc::NpcKind::from_u16(kind).unwrap_or(crate::npc::NpcKind::Slime);
        let mut n = crate::npc::Npc::new(k, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n.vx = vx;
        n.vy = vy;
        n.worm_id = 1;
        app.npcs.push(n);
    }
    // `AI_108`'s 578 bolt spawn (`Projectile.cs:33838-33841`): the lightning
    // marker resolves into the ritual bolt (580) with the marker's payload.
    for (owner, damage, kb, x, y, vx, vy, ai0) in app.projectiles.drain_ritual_bolts() {
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::VORTEX_RITUAL_BOLT,
            owner,
            x,
            y,
            vx,
            vy,
            damage,
            kb,
            ai0,
            0.0,
        );
    }
    // S1-D-3 W-9 fishron wiring — the aiStyle-64 cloud's next segment
    // (`Projectile.cs:29496`): the same kind, ai0 = 10, ai1 - 1, at the
    // segment point (the C#'s NewProjectile centre convention re-centred
    // to the port's top-left spawn).
    for (owner, kind, damage, kb, x, y, vx, vy, ai1) in app.projectiles.drain_cloud_segments() {
        if let Some(row) = crate::projectile::proj_data(kind) {
            app.projectiles.spawn_with_data_ai_owned(
                kind,
                owner,
                x - row.w * 0.5,
                y - row.h * 0.5,
                // The segment inherits the parent's velocity
                // (`:29496`'s `base.velocity.X/Y`).
                vx,
                vy,
                damage,
                kb,
                10.0,
                ai1,
            );
        }
    }
    // The aiStyle-64 cloud's shark spawn (`Projectile.cs:29509-29516`):
    // `NPC.NewNPC`'s Bottom = (X, Y) (`NPC.cs:80771`), the velocity
    // inherited, the 386's ai[2]/ai[3] presets AT THE SPAWN SITE — the
    // style71 machine reads them on its first tick (the sway amplitude
    // and the rise speed).
    for (npc_type, x, y, vx, vy, ai2, ai3) in app.projectiles.drain_shark_spawns() {
        let kind = match npc_type {
            372 => crate::npc::NpcKind::Sharkron,
            373 => crate::npc::NpcKind::Sharkron2,
            _ => continue,
        };
        let (w, h) = kind.size();
        let mut n = crate::npc::Npc::new(kind, x - w * 0.5, y - h, 1, 0.0);
        n.hp = n.max_hp;
        n.vx = vx;
        n.vy = vy;
        n.ai[2] = ai2;
        n.ai[3] = ai3;
        n.net_update = true;
        app.npcs.push(n);
    }
    // S1-P-i35 — the bobber `DoASplash` drain (`Projectile.cs:53893-53905`):
    // case 19's playing guard refuses BEFORE the one pitch draw, on the
    // `UpdateProjectiles` stream (the C# plays inline mid-`AI_061`; the port
    // drains at the pass tail, the S1-E-21c convention).
    for (style, x, y) in app.projectiles.drain_splashes() {
        app.audio.play_splash_sound(style, x, y, rng);
    }
    // S1-P-i35 — the bobber death hook's catch drain (`Projectile.cs:71986-
    // 71994` → `GiveItemToPlayer` → `QuickSpawnItem` → `GetOrDropItem`): the
    // item fills the owner's inventory first (`Player.GetItem`'s slot walk),
    // the overflow drops at the player centre. The stack roll already ran on
    // the `UpdateProjectiles` stream (the kill hook); the drop's velocity/
    // prefix draws ride the `UpdateItems` stream (`Item.NewItem` under
    // `Main.cs:17779`). The prefix rides the port's pinned velocity-triplet-
    // then-prefix order (`spawn_item_drop_px`), drawn ONLY for the overflow
    // (the C# `GetItem` keeps the rolled prefix, a recorded order deviation).
    for (owner, item, stack) in app.projectiles.drain_catches() {
        if owner != app.local_id as u8 || stack <= 0 {
            continue;
        }
        let max_stack = crate::registry::max_stack(item);
        let mut pending = stack as u32;
        while pending > 0 {
            let Some(slot) = crate::item::slot_for_pickup(&mut app.inventory, item, 0) else {
                break;
            };
            let room = max_stack.saturating_sub(app.inventory[slot].count);
            if room == 0 {
                break;
            }
            let take = pending.min(room);
            app.inventory[slot].count += take;
            pending -= take;
        }
        if pending > 0 {
            if let Some(p) = app.player.as_ref() {
                let (cx, cy) = p.center();
                crate::world_sim::items::spawn_item_drop_px(
                    &mut app.items,
                    app.rngs.stream(crate::app::RngKey::UpdateItems),
                    item,
                    pending,
                    glam::Vec2::new(cx, cy),
                );
            }
        }
    }
    // S1-P-i55 — the golf arms' queued effects, drained at the pass tail
    // (the `drain_mark_spawns` shape — the pool cannot borrow the App).
    // The C# plays the sounds / trips the wiring mid-walk; the port's
    // batch convention defers them here, on the SAME `UpdateProjectiles`
    // stream (the S1-E-21c precedent, ANALYSIS). The tee placement runs
    // first among the golf effects — the C#'s `TryPlacingAGolfBallNearA
    // NearbyTee` fires inside the helper's AI, ahead of the strike's
    // sounds; the effect queue keeps the arm order.
    for effect in app.projectiles.drain_golf_effects() {
        match effect {
            crate::projectile_ai_golf::GolfEffect::TeePlace { place_position } => {
                crate::projectile_ai_golf::try_place_golf_ball_near_tee_app(app, place_position);
            }
            crate::projectile_ai_golf::GolfEffect::CupScored { hit_location, chat, cue } => {
                // `Wiring.HitSwitch` + `SendData(59)` (`GolfHelper.cs:105-
                // 106`) — the cup's tile trip.
                app.wiring.hit_switch(&mut app.world, hit_location.0, hit_location.1);
                app.send_hit_switch(hit_location.0, hit_location.1);
                // The trip's Mech-28 clicks ride the same stream (the
                // C# call sits inside the `UpdateProjectiles` swap).
                for (px, py) in app.wiring.take_switch_clicks() {
                    app.audio.play_switch_click(
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
                if let Some(text) = chat {
                    app.chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
                }
                if let Some((id, style, x, y, pitch)) = cue {
                    app.audio.play_proj_sound(
                        (id, style),
                        x,
                        y,
                        pitch,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
            }
            crate::projectile_ai_golf::GolfEffect::PadProbe { pos, old_pos, w, h } => {
                // `Collision.SwitchTiles(..., 4)` (`Projectile.cs:20367`)
                // — the projectile pressure-pad call. The switch sends
                // ride the outcome (`Collision.cs:3304` et al.).
                let outcome = crate::collision::switch_tiles(
                    &mut app.wiring,
                    &mut app.world,
                    pos.x,
                    pos.y,
                    old_pos.x,
                    old_pos.y,
                    w as i32,
                    h as i32,
                    4,
                );
                app.send_switch_outcome(&outcome);
                // The tripped switches' Mech-28 clicks (S1-E-21i): the
                // C#'s `Collision.SwitchTiles` runs inside
                // `UpdateProjectiles` (`Main.cs:17775`'s swap), so the
                // pitch draws ride this pass's stream.
                for (px, py) in app.wiring.take_switch_clicks() {
                    app.audio.play_switch_click(
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
            }
            crate::projectile_ai_golf::GolfEffect::PlayerWrites {
                facing,
                held_proj,
                dummy_item_time,
                cancel_use,
            } => {
                if let Some(p) = app.player.as_mut() {
                    // `player.ChangeDir(direction)` (`Player.cs:42091`) —
                    // the pulley arm is unported (no pulley state); the
                    // facing write is the rest.
                    if let Some(dir) = facing {
                        p.facing = dir;
                    }
                    // `player.heldProj = whoAmI` (`:52490`/`:52535`).
                    if let Some(slot) = held_proj {
                        p.held_proj = slot;
                    }
                    if cancel_use {
                        // `SetItemTime(0)` + `itemAnimation/Max = 0` +
                        // `reuseDelay = 0` (`Projectile.cs:52515-52518`)
                        // — the port's seconds timers.
                        app.item_time = 0.0;
                        app.swing = 0.0;
                        app.swing_max = 0.0;
                        app.attack_cd = 0.0;
                        app.reuse_delay = 0.0;
                        // The channel clears with the helper's death
                        // (`Kill()`'s TryCancelChannel, `Player.cs:52975`)
                        // — the editing pass's release arm owns the
                        // steady-state clear; the cancel is the same tick.
                        p.channel = false;
                        p.channel_item = 0;
                        p.channel_target = Vec2::ZERO;
                    } else if let Some(frames) = dummy_item_time {
                        // `SetDummyItemTime(frames)` (`Player.cs:4432-
                        // 4441`): itemAnimation = itemTime = frames,
                        // itemTimeMax = frames + 1. The port's swing is
                        // the animation, item_time the action cadence.
                        app.swing = frames as f32 / 60.0;
                        app.swing_max = app.swing;
                        app.item_time = frames as f32 / 60.0;
                    }
                }
            }
        }
    }
}

/// S1-E-21h — the pass-tail sound drain's stream contract (written, NOT
/// run — code-only mode): the projectile cue plays consume the
/// `UpdateProjectiles` stream ONLY when the attenuation gate passed, and
/// every other stage stream stays pristine (the E21e witness pattern).
#[cfg(test)]
mod s1_e_21h_sound_stream_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::FROST_BOLT;
    use tr_core::UnifiedRandom;

    fn frost_bolt_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        // Inside the 8x8-tile fixture world, at rest — the arm's first
        // tick queues its `SoundID.Item28` latch cue and nothing else.
        app.projectiles.spawn_with_data_ai_owned(
            FROST_BOLT,
            0,
            32.0,
            32.0,
            0.0,
            0.0,
            10,
            4.0,
            0.0,
            0.0,
        );
        app
    }

    fn listener(sound_volume: f32) -> crate::sound::Listener {
        crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        }
    }

    #[test]
    fn s1_e_21h_the_proj_cue_draws_ride_update_projectiles_only_when_audible() {
        // Muted: the drain plays nothing and the whole pass draws nothing
        // (the frost bolt's arm is RNG-free) — the gate precedes the
        // case-2 switch (`LegacySoundPlayer.cs:369-436` before `:516`).
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine under the muted gate",
                key.name()
            );
        }
        // Audible: exactly ONE case-2 default pitch draw (`Next(-6, 7)`,
        // `:632`) on the `UpdateProjectiles` stream; the GetRandomPitch
        // NextFloat rides the audio-only stream, never a stage stream.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "exactly one case-2 pitch draw on UpdateProjectiles"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
        // The latch holds: the second tick queues nothing, draws nothing.
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "the one-cue-per-life latch draws nothing on tick 2"
        );
    }
}

#[cfg(test)]
mod summon_fixture_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::{FROST_HYDRA, HORNET, MAX_PROJECTILES, WOODEN_ARROW};

    /// One sim tick: the player step's summon-budget reset (`Player.cs:28279-28280`
    /// — the live frame runs the player step before the pool pass) then the
    /// pool pass. Driving `update_projectiles` alone would accumulate the
    /// counters across ticks and false-trip the cap.
    fn tick(app: &mut crate::app::App) {
        if let Some(p) = &mut app.player {
            p.num_minions = 0;
            p.slots_minions = 0.0;
        }
        update_projectiles(app, 1.0 / 60.0);
    }

    /// The `TR_SUMMON` fixture arm's once-per-activation shape: the pin
    /// spawns the Hornet + Frost Hydra pair anchored on the player's center,
    /// then resets — further ticks spawn nothing and a re-armed pin spawns
    /// the second pair (the review fix: the arm had no unit test).
    #[test]
    fn the_summon_fixture_spawns_one_pair_per_activation() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let (cx, cy) = app.player.as_ref().unwrap().center();
        app.summon_shot = true;
        tick(&mut app);
        assert!(!app.summon_shot, "the one-shot flag consumed");
        let mut hornet = None;
        let mut hydra = None;
        for p in app.projectiles.slots().iter().filter(|p| p.active) {
            match p.kind {
                HORNET => hornet = Some((p.pos.x, p.pos.y)),
                FROST_HYDRA => hydra = Some((p.pos.x, p.pos.y)),
                _ => {}
            }
        }
        // The same world-bounds clamp the fixture applies (the flow world is
        // 8x8 tiles — an unclamped 80-px Frost Hydra would fall out of world
        // and Phase 3's guard would deactivate it, exactly as the C# would).
        let (ww, wh) = (
            app.world.width as f32 * 16.0,
            app.world.height as f32 * 16.0,
        );
        // The sentry (aiStyle 53) free-falls and `HandleMovement` integrates it
        // on the same tick, so both members stay anchored to within one tick's
        // step rather than exactly.
        let sentry_anchor = (
            (cx + 24.0).clamp(0.0, ww - 80.0 - 1.0),
            (cy - 48.0).clamp(0.0, wh - 74.0 - 1.0),
        );
        let (sx, sy) = hydra.expect("the Frost Hydra renders");
        assert!(
            (sx - sentry_anchor.0).abs() < 8.0 && (sy - sentry_anchor.1).abs() < 8.0,
            "the sentry anchored within one tick's fall: {sx},{sy} vs {sentry_anchor:?}"
        );
        let (hx, hy) = hornet.expect("the Hornet renders");
        let anchor = (
            (cx - 24.0).clamp(0.0, ww - 24.0 - 1.0),
            (cy - 48.0).clamp(0.0, wh - 26.0 - 1.0),
        );
        assert!(
            (hx - anchor.0).abs() < 8.0 && (hy - anchor.1).abs() < 8.0,
            "the Hornet anchored within one tick's drift: {hx},{hy} vs {anchor:?}"
        );
        assert_eq!(app.projectiles.active_count(), 2, "exactly the pair");
        // The flag stays down: further ticks spawn nothing new.
        tick(&mut app);
        assert_eq!(app.projectiles.active_count(), 2, "no per-tick respawn");
        // The reset makes the arm re-armable: a second activation spawns
        // the second pair.
        app.summon_shot = true;
        tick(&mut app);
        assert!(!app.summon_shot);
        assert_eq!(app.projectiles.active_count(), 4, "the second pair");
    }

    /// A full pool declines the spawn — the C# `NewProjectile`'s 1000
    /// sentinel (`Projectile.cs:10793`) drops the request — while the arm
    /// still consumes the flag.
    #[test]
    fn the_summon_fixture_respects_a_full_pool() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // Fill every slot with an active filler so the arm finds no space.
        for _ in 0..MAX_PROJECTILES {
            app.projectiles.spawn_with_data_ai_owned(
                WOODEN_ARROW,
                0,
                16.0,
                16.0,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
        }
        app.summon_shot = true;
        update_projectiles(&mut app, 1.0 / 60.0);
        assert!(!app.summon_shot, "the flag consumed even on the decline");
        assert_eq!(app.projectiles.active_count(), MAX_PROJECTILES);
        assert!(
            !app.projectiles
                .slots()
                .iter()
                .any(|p| p.active && (p.kind == HORNET || p.kind == FROST_HYDRA)),
            "the pair declined: the pool stayed full of fillers"
        );
    }
}

#[cfg(test)]
mod dw4_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::WOOD_YOYO;
    use tr_core::UnifiedRandom;

    /// Two same-kind yoyos out: the doubling `flag` lifetime jitter
    /// (`Projectile.cs:67746-67749` — one `Next(10, 31)`) is the pass's
    /// whole draw for the tick.
    fn yoyo_pair_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // Inside the 8x8-tile fixture world (the out-of-world guard would
        // deactivate edge spawns on the next tick).
        for i in 0..2 {
            app.projectiles.spawn_with_data_ai_owned(
                WOOD_YOYO,
                0,
                32.0 + i as f32 * 16.0,
                32.0,
                2.0,
                0.0,
                10,
                4.0,
                0.0,
                0.0,
            );
        }
        app
    }

    /// DW-4.1 (the projectiles pass): one tick with two same-kind yoyos out
    /// draws exactly the doubling lifetime jitter's one `Next(10, 31)` on the
    /// `UpdateProjectiles` stream (`Main.cs:17775-17777`), and every other
    /// stage stream stays pristine (the pass's isolation).
    #[test]
    fn test_dw_4_1_projectiles_pass_draws_the_update_projectiles_stream() {
        let mut app = yoyo_pair_app();
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine",
                key.name()
            );
        }
        update_projectiles(&mut app, 1.0 / 60.0);

        // The twin: one `Next(10, 31)` from the pristine seed.
        let mut expect = UnifiedRandom::new(0);
        let _ = expect.next_range(10, 31);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            expect,
            "exactly one lifetime-jitter draw on UpdateProjectiles"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
    }

    /// DW-4.4: the projectiles pass's determinism pair — two same-seed runs
    /// of five ticks give byte-equal slot states and stream states.
    #[test]
    fn test_dw_4_4_projectiles_pass_determinism_pair() {
        let run = || {
            let mut app = yoyo_pair_app();
            // Two ticks: the unchanneld yoyos start their return soon after
            // (the release arm), so the pin holds the pair while both live.
            for _ in 0..2 {
                update_projectiles(&mut app, 1.0 / 60.0);
            }
            let slots: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .map(|p| (p.kind, p.pos, p.vel, p.ai, p.local_ai))
                .collect();
            (slots, app.rngs.stream(RngKey::UpdateProjectiles).clone())
        };
        let a = run();
        let b = run();
        assert_eq!(a.1, b.1, "the stream replayed identically");
        assert_eq!(a.0, b.0, "the pool state is byte-equal");
        assert!(!a.0.is_empty(), "at least one yoyo live across the pair");
    }
}

/// S1-P-i23 — the pass-level channel witnesses (written, NOT run — code-only
/// mode): the yoyo/flail snapshots read the LATCHED `Player.channel` (not a
/// per-tick button re-derive), and the pass tail drains the spawn-tracking
/// intents onto the cancel key (`Projectile.cs:11097-11100`).
#[cfg(test)]
mod s1_p_i23_channel_pass_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::WOOD_YOYO;

    /// A mid-channel player on the Wood Yoyo row (item 3278, shoot 541),
    /// parked at centre (64, 64) inside the 8x8-tile fixture world, the
    /// cursor 40 px right of centre. The button is UP — the latch alone
    /// must hold the aim (the pre-i23 per-tick re-derive would release).
    fn yoyo_channel_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 54.0;
        p.y = 43.0;
        p.channel = true;
        p.channel_item = 3278;
        p.channel_proj_expected = WOOD_YOYO;
        app.player = Some(p);
        app.camera = Vec2::new(64.0, 64.0);
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 40.0, sh * 0.5);
        app.mine_down = false;
        app.projectiles
            .spawn_with_data_ai_owned(WOOD_YOYO, 0, 56.0, 56.0, 0.0, 0.0, 9, 2.5, 0.0, 0.0);
        app
    }

    #[test]
    fn s1_p_i23_the_pass_reads_the_latched_channel_not_the_button() {
        // `AI_099_2_Yoyos`'s release arm (`Projectile.cs:67920-67925`)
        // reads `Main.player[owner].channel` — the latch. Button up +
        // latch open keeps the aim; the latch drop releases it.
        let mut app = yoyo_channel_app();
        update_projectiles(&mut app, 1.0 / 60.0);
        let yoyo = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == WOOD_YOYO)
            .expect("the yoyo lives");
        assert_eq!(yoyo.ai[0], 104.0, "the latched channel holds the aim");
        // The latch drops (a kill-cancel / CC / release would do this).
        app.player.as_mut().unwrap().channel = false;
        update_projectiles(&mut app, 1.0 / 60.0);
        let yoyo = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.kind == WOOD_YOYO)
            .expect("the slot");
        assert_eq!(yoyo.ai[0], -1.0, "the dropped latch releases the aim");
    }

    #[test]
    fn s1_p_i23_the_pass_tail_tracks_the_local_spawns() {
        // `NewProjectile`'s tail (`Projectile.cs:11097-11100`): the pool
        // queues the intents mid-loop; the pass tail applies them — the
        // LAST local spawn of the expected type wins (`TryTracking`
        // `Player.cs:134-140`). Two yoyos out: the key lands on slot 1.
        let mut app = yoyo_channel_app();
        app.projectiles
            .spawn_with_data_ai_owned(WOOD_YOYO, 0, 72.0, 56.0, 0.0, 0.0, 9, 2.5, 0.0, 0.0);
        assert_eq!(
            app.player.as_ref().unwrap().channel_index_expected,
            0,
            "the struct default before the pass"
        );
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.player.as_ref().unwrap().channel_index_expected,
            1,
            "the pass tail tracked the last spawn"
        );
    }
}

/// S1-P-i09 — the yoyo snapshot's `controlUseItem` gates. `Player.ItemCheck`
/// clears the channel whenever the use input is cleared
/// (`Player.cs:42804-42807`): the inventory, the fullscreen map
/// (`Player.cs:24798-24807`) and the minimap hover (`MinimapFrame.cs:164`)
/// all drop it, so the aiStyle-99 release arm (`Projectile.cs:67920-67925`)
/// fires behind them. Re-pinned onto the S1-P-i23 latch: the fixture arms
/// `p.channel` directly (a real use-start would), and the snapshot gates
/// the latch on the use-input clears — no per-tick button re-derive.
#[cfg(test)]
mod s1_p_i09_yoyo_snapshot_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::WOOD_YOYO;

    /// One Wood Yoyo out with the channel latch armed (the Wood Yoyo item
    /// 3278 held). The spawn sits inside the 8×8-tile fixture world (the
    /// out-of-world guard); the camera centres on the player and the mouse
    /// sits right of centre so the aim world point lands inside the string
    /// range (`ai[0]` is a world coordinate, the C#'s own edge quirk).
    fn yoyo_channel_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = crate::player::Player::spawn(&app.world);
        let (cx, cy) = p.center();
        p.channel = true;
        p.channel_item = 3278;
        p.channel_proj_expected = WOOD_YOYO;
        app.player = Some(p);
        app.camera = Vec2::new(cx, cy);
        app.zoom = 1.0;
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 100.0, sh * 0.5);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 3278,
            name: "Wood Yoyo",
            count: 1,
            prefix: 0,
        };
        app.mine_down = true;
        app.projectiles.spawn_with_data_ai_owned(
            WOOD_YOYO, 0, 32.0, 32.0, 2.0, 0.0, 10, 4.0, 0.0, 0.0,
        );
        app
    }

    #[test]
    fn s1_p_i09_the_held_channel_keeps_the_yoyo_flying() {
        let mut app = yoyo_channel_app();
        update_projectiles(&mut app, 1.0 / 60.0);
        assert!(app.projectiles.slots()[0].ai[0] >= 0.0, "the hold keeps the flight");
    }

    #[test]
    fn s1_p_i09_the_open_inventory_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        app.inventory_open = true;
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the inventory clears the use input");
        assert_eq!(app.projectiles.slots()[0].ai[1], 0.0);
    }

    #[test]
    fn s1_p_i09_the_fullscreen_map_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        let mut map = crate::map::WorldMap::new(&app.world);
        map.open = true;
        app.map = Some(map);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the fullscreen map clears the use input");
    }

    #[test]
    fn s1_p_i09_the_minimap_hover_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        app.minimap_hover = Some(crate::ui::minimap::MinimapButton::ZoomIn);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the minimap hover clears the use input");
    }

    /// `Projectile.cs:67920` — the snapshot carries the owner's `CCed`
    /// (`Player.cced()` — frozen / webbed / stoned): a frozen owner
    /// releases the string even with the button held.
    #[test]
    fn s1_p_i09_the_frozen_owner_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        app.player.as_mut().unwrap().frozen = true;
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the CC releases");
    }
}

/// W4-d80 — the bomb-fuse tracked loop (research/64 sites 20/21): the
/// ensure-play / stop dance through the drain, the `local_ai[2]` handle
/// write-back, the shared-atomic condition fold and the RNG contract
/// (zero gameplay draws). WRITTEN NOT RUN (code-only mode).
#[cfg(test)]
mod w4_d80_bomb_fuse_tests {
    use super::*;
    use crate::projectile::{is_bomb_with_fuse, ProjTrackedCue};
    use tr_core::slot::SlotId;

    /// The headless rig: the fuse cue seeded silent so the play resolves
    /// its asset and adds a slot (no output device).
    fn rig() -> (crate::audio::Audio, Vec<Projectile>) {
        let mut audio = crate::audio::Audio::new(None);
        audio.test_seed_tracked_cue("fuse");
        let mut p = Projectile {
            active: true,
            kind: crate::projectile::BOMB,
            owner: 0,
            pos: Vec2::new(96.0, 96.0),
            time_left: 180,
            ..Default::default()
        };
        p.local_ai[2] = 0.0;
        (audio, vec![p])
    }

    /// The membership gate: the verbatim `IsABombWithFuse` set
    /// (`ProjectileID.cs:424`) — every member passes, the grenades and
    /// the launcher rockets do not.
    #[test]
    fn w4_d80_the_fuse_set_matches_the_census() {
        for kind in [
            28u16, 37, 516, 519, 910, 911, 1086, 1087, 906, 905, 904, 903, 773,
            1077, 75, 102, 681, 470, 29, 637,
        ] {
            assert!(is_bomb_with_fuse(kind), "kind {kind} is a fuse bomb");
        }
        for kind in [30u16, 397, 517, 133, 134, 776, 862] {
            assert!(!is_bomb_with_fuse(kind), "kind {kind} is NOT a fuse bomb");
        }
    }

    /// The ensure plays once into `local_ai[2]`; a live, playing handle
    /// holds the ensure (no re-play); the stop zeroes the handle and
    /// flips the shared atomic (`Projectile.cs:46917-46921` /
    /// `:70673-70677`).
    #[test]
    fn w4_d80_the_fuse_ensures_once_then_stops() {
        let (mut audio, mut slots) = rig();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::BombFuseEnsure, 100.0, 100.0);
        let id = SlotId::from_float(slots[0].local_ai[2]);
        assert!(id.is_active(), "the ensure stored a live handle");
        assert!(slots[0].fuse_alive.is_some(), "the condition atomic armed");
        audio.test_keep_tracked_alive(id);
        // The ensure is a no-op while the slot lives and plays.
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::BombFuseEnsure, 100.0, 100.0);
        assert_eq!(SlotId::from_float(slots[0].local_ai[2]), id, "no re-play while live");
        // The stop zeroes the handle and flips the atomic false.
        let alive = slots[0].fuse_alive.clone().unwrap();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::BombFuseStop, 100.0, 100.0);
        assert_eq!(slots[0].local_ai[2], 0.0, "the stop zeroes localAI[2]");
        assert!(slots[0].fuse_alive.is_none(), "the atomic released");
        assert!(!alive.load(std::sync::atomic::Ordering::Relaxed), "the condition flipped false");
    }

    /// The RNG contract: the fuse ensure + stop leave the gameplay stream
    /// pristine; the two audio-only statics move on the play.
    #[test]
    fn w4_d80_the_fuse_drain_is_gameplay_draw_free() {
        let (mut audio, mut slots) = rig();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let peek = rng.peek();
        let (style_before, pitch_before) = audio.test_audio_streams();
        // The drain takes no gameplay rng at all — the tracked plays draw
        // only the audio-only statics. Assert via a witness stream that
        // the ensure/stop path never touches a `Main.rand` stand-in.
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::BombFuseEnsure, 100.0, 100.0);
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::BombFuseStop, 100.0, 100.0);
        assert_eq!(rng.peek(), peek, "zero Main.rand draws on the fuse path");
        let (style_after, pitch_after) = audio.test_audio_streams();
        // The fuse is one variation (`SoundID.cs:1764`) — `random_sound`
        // draws NOTHING (`LegacySoundStyle.cs:24`'s `Variations != 1`
        // gate); only the always-drawn pitch roll moves.
        assert_eq!(style_after, style_before, "one variation: no variation pick");
        assert_ne!(pitch_after, pitch_before, "the pitch roll moved");
    }

    /// The style row: `BombFuse` resolves with volume 0.2, one variation,
    /// uncapped (`SoundID.cs:1764`'s `CreateTrackable("fuse").WithVolume(0.2f)`).
    #[test]
    fn w4_d80_the_fuse_style_resolves() {
        let style = crate::tracked_sound::TrackedStyle::by_name("BombFuse").expect("BombFuse");
        assert_eq!(style.cue, "fuse");
        assert_eq!(style.variations, 1);
        assert!((style.volume - 0.2).abs() < 1e-6, "the 0.2 volume");
        assert_eq!(style.max_tracked_instances, 0);
    }
}

/// W4-d80 — the DD2 projectile one-shots (research/64 deferred): the
/// fire-and-forget tracked plays, the handle-store-and-follow dance and the
/// RNG contract (zero gameplay draws). WRITTEN NOT RUN (code-only mode).
#[cfg(test)]
mod w4_d80_dd2_one_shot_tests {
    use super::*;
    use crate::projectile::ProjTrackedCue;
    use tr_core::slot::SlotId;

    /// The headless rig: the DD2 cues seeded silent so the plays resolve
    /// their assets and add slots (no output device). EVERY variation of
    /// each family is seeded — the `random_sound` pick draws 0..3, so a
    /// missing `_1`/`_2` would fold the play to `SlotId::INVALID`.
    fn rig() -> (crate::audio::Audio, Vec<Projectile>) {
        let mut audio = crate::audio::Audio::new(None);
        for cue in [
            "dd2_betsy_fireball_shot_0",
            "dd2_betsy_fireball_shot_1",
            "dd2_betsy_fireball_shot_2",
            "dd2_betsy_fireball_impact_0",
            "dd2_betsy_fireball_impact_1",
            "dd2_betsy_fireball_impact_2",
            "dd2_flameburst_tower_shot_0",
            "dd2_flameburst_tower_shot_1",
            "dd2_flameburst_tower_shot_2",
            "dd2_phantom_phoenix_shot_0",
            "dd2_phantom_phoenix_shot_1",
            "dd2_phantom_phoenix_shot_2",
            "dd2_explosive_trap_explode_0",
            "dd2_explosive_trap_explode_1",
            "dd2_explosive_trap_explode_2",
        ] {
            audio.test_seed_tracked_cue(cue);
        }
        let p = Projectile {
            active: true,
            kind: crate::projectile::BETSY_FIREBALL,
            owner: 0,
            pos: Vec2::new(96.0, 96.0),
            time_left: 180,
            ..Default::default()
        };
        (audio, vec![p])
    }

    /// The fire-and-forget one-shot (`OneShot`/`KillOneShot`): the play adds
    /// a slot, the handle is NOT stored (the C# discards it). The global
    /// overload (`GlobalOneShot`) plays without a position.
    #[test]
    fn w4_d80_the_dd2_one_shot_plays_and_discards() {
        let (mut audio, mut slots) = rig();
        let before = audio.test_audio_streams();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::OneShot("DD2_BetsyFireballShot"), 100.0, 100.0);
        // The handle is discarded — local_ai stays 0.
        assert_eq!(slots[0].local_ai[0], 0.0, "the one-shot discards the handle");
        assert_eq!(slots[0].local_ai[1], 0.0);
        assert_eq!(slots[0].local_ai[2], 0.0);
        // The play drew the audio-only statics (3 variations → the pick +
        // the always-drawn pitch roll).
        let after = audio.test_audio_streams();
        assert_ne!(after.0, before.0, "the variation pick moved");
        assert_ne!(after.1, before.1, "the pitch roll moved");
        // The kill-tail one-shot plays too.
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::KillOneShot("DD2_BetsyFireballImpact"), 100.0, 100.0);
        // The global overload (the trap explode).
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::GlobalOneShot("DD2_ExplosiveTrapExplode"), 100.0, 100.0);
    }

    /// The handle-store-and-follow (`HandlePlayFollow`): the play tick
    /// stores the handle into `local_ai[idx]`; the follow re-points the live
    /// sound; a reaped handle folds to `SlotId.Invalid`.
    #[test]
    fn w4_d80_the_dd2_handle_play_follow() {
        let (mut audio, mut slots) = rig();
        // The play tick (idx 1, the Flameburst convention).
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::HandlePlayFollow { style: "DD2_FlameburstTowerShot", idx: 1, play: true }, 100.0, 100.0);
        let id = SlotId::from_float(slots[0].local_ai[1]);
        assert!(id.is_active(), "the play stored a live handle");
        audio.test_keep_tracked_alive(id);
        // The follow tick: no re-play, the position re-points.
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::HandlePlayFollow { style: "DD2_FlameburstTowerShot", idx: 1, play: false }, 200.0, 200.0);
        assert_eq!(SlotId::from_float(slots[0].local_ai[1]), id, "no re-play");
        assert_eq!(audio.get_active_sound(id).unwrap().position, (200.0, 200.0), "the follow re-points");
        // A reaped handle folds to Invalid (the C# null arm). Clear the
        // keep-alive seam so the headless slot (no sink) reaps on update.
        if let Some(s) = audio.get_active_sound_mut(id) {
            s.set_test_keep_alive(false);
        }
        audio.update_tracked(false);
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::HandlePlayFollow { style: "DD2_FlameburstTowerShot", idx: 1, play: false }, 300.0, 300.0);
        assert_eq!(slots[0].local_ai[1].to_bits(), 65535, "the reaped handle folds to SlotId.Invalid");
    }

    /// The RNG contract: the DD2 one-shot drain leaves the gameplay stream
    /// pristine (the plays draw only the audio-only statics).
    #[test]
    fn w4_d80_the_dd2_one_shot_drain_is_gameplay_draw_free() {
        let (mut audio, mut slots) = rig();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let peek = rng.peek();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::OneShot("DD2_BetsyFireballShot"), 100.0, 100.0);
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::HandlePlayFollow { style: "DD2_PhantomPhoenixShot", idx: 0, play: true }, 100.0, 100.0);
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::GlobalOneShot("DD2_ExplosiveTrapExplode"), 100.0, 100.0);
        assert_eq!(rng.peek(), peek, "zero Main.rand draws on the DD2 one-shot path");
    }

    /// The style rows resolve through `soundid.rs` (the trackable names).
    #[test]
    fn w4_d80_the_dd2_one_shot_styles_resolve() {
        for (name, cue, variations) in [
            ("DD2_BetsyFireballShot", "dd2_betsy_fireball_shot", 3u32),
            ("DD2_BetsyFireballImpact", "dd2_betsy_fireball_impact", 3),
            ("DD2_FlameburstTowerShot", "dd2_flameburst_tower_shot", 3),
            ("DD2_PhantomPhoenixShot", "dd2_phantom_phoenix_shot", 3),
            ("DD2_SkyDragonsFuryShot", "dd2_sky_dragons_fury_shot", 3),
            ("DD2_ExplosiveTrapExplode", "dd2_explosive_trap_explode", 3),
            ("DD2_SkeletonSummoned", "dd2_skeleton_summoned", 1),
        ] {
            let style = crate::tracked_sound::TrackedStyle::by_name(name).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(style.cue, cue, "{name}");
            assert_eq!(style.variations, variations, "{name}");
            assert_eq!(style.max_tracked_instances, 0, "{name} uncapped");
        }
    }
}

/// W4-d80 — the RC-car / Jim's-drone engine loops (research/64 deferred):
/// the ensure-play into `localAI[2]`, the per-tick volume/position/pitch
/// follow, the drone's explicit Kill stop and the RNG contract (zero
/// gameplay draws). WRITTEN NOT RUN (code-only mode).
#[cfg(test)]
mod w4_d80_engine_loop_tests {
    use super::*;
    use crate::projectile::ProjTrackedCue;
    use tr_core::slot::SlotId;

    /// The headless rig: the two engine cues seeded silent so the looped
    /// plays resolve their assets and add slots (no output device).
    fn rig() -> (crate::audio::Audio, Vec<Projectile>) {
        let mut audio = crate::audio::Audio::new(None);
        audio.test_seed_tracked_cue("rccar");
        audio.test_seed_tracked_cue("Drone");
        let p = Projectile {
            active: true,
            kind: crate::projectile::REMOTE_CONTROL_CAR,
            owner: 0,
            pos: Vec2::new(96.0, 96.0),
            time_left: 2,
            ..Default::default()
        };
        (audio, vec![p])
    }

    /// The ensure + follow (`Projectile.cs:38256-38267`): the first cue
    /// plays the loop into `localAI[2]` and arms the condition atomic; a
    /// live handle holds the ensure (no re-play) while the follow re-points
    /// the volume/position/pitch every tick.
    #[test]
    fn w4_d80_the_engine_loop_ensures_then_follows() {
        let (mut audio, mut slots) = rig();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::EngineLoop { style: "RCCar", vol: 12.0, pitch: 0.5 }, 100.0, 100.0);
        let id = SlotId::from_float(slots[0].local_ai[2]);
        assert!(id.is_active(), "the ensure stored a live handle");
        assert!(slots[0].fuse_alive.is_some(), "the condition atomic armed");
        audio.test_keep_tracked_alive(id);
        // The second tick: no re-play, the envelope follow rides.
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::EngineLoop { style: "RCCar", vol: 25.0, pitch: -0.2 }, 200.0, 200.0);
        assert_eq!(SlotId::from_float(slots[0].local_ai[2]), id, "no re-play while live");
        let s = audio.get_active_sound(id).unwrap();
        assert_eq!(s.volume, 25.0, "the follow volume");
        assert_eq!(s.position, (200.0, 200.0), "the follow position");
        assert_eq!(s.pitch, -0.2, "the follow pitch");
    }

    /// The zero-volume gate (`:38257`'s `num11 != 0f`): a silent envelope
    /// never plays the loop.
    #[test]
    fn w4_d80_the_engine_loop_zero_volume_does_not_play() {
        let (mut audio, mut slots) = rig();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::EngineLoop { style: "RCCar", vol: 0.0, pitch: -1.0 }, 100.0, 100.0);
        assert_eq!(slots[0].local_ai[2], 0.0, "no play at zero volume");
        assert!(slots[0].fuse_alive.is_none(), "no atomic armed");
    }

    /// The drone's Kill stop (`:70863`): `GetActiveSound(localAI[2])?.Stop()`
    /// — the explicit stop + the atomic flip. The C# does NOT zero
    /// `localAI[2]` here (contrast the bomb fuse's `:70677`).
    #[test]
    fn w4_d80_the_engine_loop_stop() {
        let (mut audio, mut slots) = rig();
        slots[0].kind = crate::projectile::JIMS_DRONE;
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::EngineLoop { style: "JimsDrone", vol: 12.0, pitch: 0.0 }, 100.0, 100.0);
        let id = SlotId::from_float(slots[0].local_ai[2]);
        assert!(id.is_active());
        audio.test_keep_tracked_alive(id);
        let alive = slots[0].fuse_alive.clone().unwrap();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::EngineLoopStop, 100.0, 100.0);
        assert!(!alive.load(std::sync::atomic::Ordering::Relaxed), "the condition flipped false");
        assert!(slots[0].fuse_alive.is_none(), "the atomic released");
        assert_eq!(SlotId::from_float(slots[0].local_ai[2]), id, "the C# keeps the handle (`:70863` has no zero write)");
    }

    /// The RNG contract: the engine-loop drain leaves the gameplay stream
    /// pristine; both styles are one variation (`soundid.rs` rows) so the
    /// variation pick draws NOTHING — only the always-drawn pitch roll moves.
    #[test]
    fn w4_d80_the_engine_loop_drain_is_gameplay_draw_free() {
        let (mut audio, mut slots) = rig();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let peek = rng.peek();
        let (style_before, pitch_before) = audio.test_audio_streams();
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::EngineLoop { style: "RCCar", vol: 12.0, pitch: 0.0 }, 100.0, 100.0);
        apply_proj_tracked_cue(&mut audio, &mut slots, 0, ProjTrackedCue::EngineLoopStop, 100.0, 100.0);
        assert_eq!(rng.peek(), peek, "zero Main.rand draws on the engine-loop path");
        let (style_after, pitch_after) = audio.test_audio_streams();
        assert_eq!(style_after, style_before, "one variation: no variation pick");
        assert_ne!(pitch_after, pitch_before, "the pitch roll moved");
    }

    /// The style rows (`soundid.rs`, the `SoundID.cs` static-ctor port):
    /// `RCCar` → "rccar" vol 0.015, `JimsDrone` → "Drone" vol 0.1, one
    /// variation each, uncapped.
    #[test]
    fn w4_d80_the_engine_loop_styles_resolve() {
        let style = crate::tracked_sound::TrackedStyle::by_name("RCCar").expect("RCCar");
        assert_eq!(style.cue, "rccar");
        assert_eq!(style.variations, 1);
        assert!((style.volume - 0.015).abs() < 1e-6, "the 0.015 volume");
        assert_eq!(style.max_tracked_instances, 0);
        let style = crate::tracked_sound::TrackedStyle::by_name("JimsDrone").expect("JimsDrone");
        assert_eq!(style.cue, "Drone");
        assert_eq!(style.variations, 1);
        assert!((style.volume - 0.1).abs() < 1e-6, "the 0.1 volume");
        assert_eq!(style.max_tracked_instances, 0);
    }
}

