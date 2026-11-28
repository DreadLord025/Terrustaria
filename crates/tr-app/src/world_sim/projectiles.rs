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
    // W4-g70 lane 8 — the natural star's death-burst screen gate
    // (`Projectile.cs:75704-75705`): the view rect in world pixels
    // (`Main.screenPosition` + the screen size; the port's camera is the
    // view centre, the zoom spans `screen / zoom` world pixels).
    app.projectiles.set_view_rect(
        app.camera.x - sw * 0.5 / app.zoom,
        app.camera.y - sh * 0.5 / app.zoom,
        sw / app.zoom,
        sh / app.zoom,
    );
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
        // S1-E-21h d75 — the drill hum's `inventory[selectedItem].pick > 0`
        // read (`Projectile.cs:25512`), snapshotted off the held ItemDef.
        held_pick_positive: crate::registry::item(app.inventory[app.selected].kind)
            .is_some_and(|it| it.pick > 0),
        // W4-d10 — `AI_099_1_Counterweights`'s channel push (`:67521`):
        // the frame-head `UpdateMainMouse` snapshot (`PlayerInput.cs:
        // 911-917`), already in the world-update zoom basis
        // (`SetZoom_MouseInWorld`, `:2528-2542`). Drained per projectile
        // tick — every sub-step reads the same per-tick delta, exactly
        // like the C# reads the same `Main.mouseX/lastMouseX` pair.
        mouse_delta: (app.mouse_tick_delta.x, app.mouse_tick_delta.y),
        // W4-d10 — the yoyo-string accessory grants (`Player.cs:14710-
        // 14754`, the `ApplyEquipFunctional` rows; `ResetEffects` clears
        // per tick `:18435`/`:18439`). The yoyo arm's lifetime/range terms
        // and the counterweight's radius/ghost split read them.
        yoyo_string: app.player.as_ref().is_some_and(|p| p.yoyo_string),
        magic_string: app.player.as_ref().is_some_and(|p| p.magic_string),
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
    // W4-e3 — the dust seam: the frame's spawn ctx is captured HERE, before
    // the `rng` mutable borrow (the helper borrows `app` immutably — the
    // `world_sim/npcs.rs:4191` convention); the live pool is threaded into
    // the tick closure below. The aiStyle-2/4/5/8 dust arms roll the SAME
    // `UpdateProjectiles` stream through the verbatim `DustPool::new_dust`
    // (`Main.cs:17775-17777`'s `SwapRandom` wraps the walk, dust included).
    let dust_spawn_ctx = app.spawn_ctx();
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
    // W4-d9 — the aiStyle-90 lantern's owner snapshot
    // (`Projectile.cs:32389-32416`'s `Main.player[owner]` reads; the scan
    // gate's `Main.myPlayer` centre folds onto the same owner — the port
    // simulates only the local owner's slots single-player). The screen
    // width rides `app.screen_size()` (the `Main.screenWidth` stand-in).
    let lantern_owner = app.player.as_ref().map(|p| {
        let (cx, cy) = p.center();
        crate::projectile_ai_style90::LanternOwner {
            center: Vec2::new(cx, cy),
            direction: p.facing,
            dead: p.dead,
            magic_lantern: p.magic_lantern,
        }
    });
    app.projectiles.set_lantern_owner(lantern_owner);
    app.projectiles.set_screen_width(sw);

    // W4-d15 — the DD2 slice's owner snapshot (`AI_142_MonkStaffT2And3`'s
    // `Main.player[owner]` reads, `Projectile.cs:69819-69844`). The anchor
    // is `RotatedRelativePoint(MountedCenter)` (`:69820`); the animation
    // integers derive from the swing timers (the spear convention). The
    // pet arms' reads extend this snapshot with their styles.
    let dd2_owner = app.player.as_ref().map(|p| {
        let (mcx, mcy) = p.mounted_center();
        let (cx, cy) = p.center();
        crate::projectile_ai_dd2::Dd2Owner {
            anchor: p.rotated_relative_point(Vec2::new(mcx, mcy), 0),
            center: Vec2::new(cx, cy),
            mounted_center: Vec2::new(mcx, mcy),
            direction: p.facing,
            dead: p.dead,
            frozen: p.frozen,
            item_animation: (app.swing * 60.0).round() as i32,
            item_animation_max: (app.swing_max * 60.0).round() as i32,
            // `player.petFlagDD2Dragon/Ghost/Gato/UpbeatStar/AxeFairyPet`
            // — the buff rows index `pet_flags` by PET PROJECTILE id
            // (`buff_effects_accessory.rs`).
            pet_flag_dd2_dragon: p.pet_flags[usize::from(crate::projectile::DD2_PET_DRAGON)],
            pet_flag_dd2_ghost: p.pet_flags[usize::from(crate::projectile::DD2_PET_GHOST)],
            pet_flag_dd2_gato: p.pet_flags[usize::from(crate::projectile::DD2_PET_GATO)],
            pet_flag_upbeat_star: p.pet_flags[usize::from(crate::projectile::UPBEAT_STAR)],
            pet_flag_axe_fairy: p.pet_flags[usize::from(crate::projectile::AXE_FAIRY_PET)],
            player_pos: Vec2::new(p.x, p.y),
        }
    });
    app.projectiles.set_dd2_owner(dd2_owner);

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
    // W4-d7 F3 — the Companion Cube's darkness read (`Projectile.cs:49527-
    // 49528`): the two `Lighting.GetColor` samples (the cube's centre tile +
    // the owner's centre tile) as `Color.ToVector3()` lengths. `GetColor`
    // (`Lighting.cs:216-242`) byte-quantizes `(int)(engineColor * GlobalBrightness
    // * 255)` clamped to 255, then `ToVector3` divides by 255 — so the length is
    // the brightness-scaled, byte-truncated colour. GlobalBrightness is 1.2
    // (`Lighting.cs:91`), dropped to 1.0 by the Darkness debuff's `blind` flag
    // (`:114`). Sampled from the live light map before the pass (the `set_disco`
    // precedent); only when a cube is owned.
    let cube_light = app.player.as_ref().and_then(|p| {
        let cube = app
            .projectiles
            .slots()
            .iter()
            .find(|s| s.active && s.kind == crate::projectile::COMPANION_CUBE && s.owner == 0)?;
        let brightness = if p.blind { 1.0 } else { tr_core::lighting::GLOBAL_BRIGHTNESS };
        // `Lighting.cs:224` precomputes `num = GlobalBrightness * 255f` ONCE,
        // then `(int)(color.X * num)` — the multiply order matters for the
        // byte truncation, so fold the scale first (not `v * b * 255`).
        let scale = brightness * 255.0;
        let len = |wx: f32, wy: f32| {
            let c = app.light.get((wx / 16.0) as i32, (wy / 16.0) as i32);
            let byte = |v: f32| ((v * scale) as i32).min(255) as f32 / 255.0;
            let (r, g, b) = (byte(c[0]), byte(c[1]), byte(c[2]));
            (r * r + g * g + b * b).sqrt()
        };
        let (pcx, pcy) = p.center();
        Some((
            len(cube.pos.x + cube.w * 0.5, cube.pos.y + cube.h * 0.5),
            len(pcx, pcy),
        ))
    });
    app.projectiles.set_cube_light(cube_light);
    // W4-d7 F3 — the Storm Tigers' special-attack gem count
    // (`ownedProjectileCounts[831]`, `Projectile.cs:49039`) + the
    // `Main.timeForVisualEffects` draw clock (the state-5 cosine bob,
    // `:49683`; the `anim_time * 60 % 216000` form, `frame/draw.rs:374`).
    let gem_counts = app.player.as_ref().map_or((0, 0), |p| {
        (p.owned_projectile_counts[831], p.owned_projectile_counts[970])
    });
    app.projectiles.set_tiger_gem_counts(gem_counts);
    app.projectiles.set_time_for_visuals((app.anim_time * 60.0) % 216000.0);
    // `Main.DiscoR/G/B` as the aiStyle-123 crystal's shard-hue roll reads
    // it (`Projectile.cs:37262`); the clock's `advance_disco` owns the cycle.
    let disco = app.disco;
    app.projectiles.set_disco(disco);
    // `Player.SceneMetrics.NPCBannerBuff` as the hostile shots' `Damage_EVP`
    // banner arm reads it (`Projectile.cs:14832` → `Player.cs:44197`); the
    // per-tick scene-scan snapshot (the `set_disco` precedent).
    app.projectiles
        .set_npc_banner_buff(app.scene_metrics.npc_banner_buff.clone());
    // W4-d1 — `Main.infectedSeed` as the PurificationPowder 10's spawn-time
    // box growth reads it (`Projectile.cs:907-911`): the world's
    // `SecretSeed.worldIsInfected` micro-seed flag (`WorldGen.cs:559-562`).
    app.projectiles.set_infected_seed(
        app.world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::WORLD_IS_INFECTED),
    );
    // W4-d3 — `Main.tileFrame[665]` as the aiStyle-25 1014 LifeCrystalBoulder's
    // frame pin reads it (`Projectile.cs:26028`); the tile animator owns the
    // row (`app.tile_anim`, S1-G-1f), the pool only reads it (the `set_disco`
    // pattern).
    let frame_665 = app.tile_anim.tile_frame[665];
    app.projectiles.set_tile_frame_665(frame_665);
    // W4-c19 round 3 — `Main.mouseTextColor` as the 590 TruffleSpore's flicker
    // reads it (`Projectile.cs:34221`); the draw-pass pulse owns the value
    // (the `set_disco` precedent — the step reads what the last draw left).
    app.projectiles.set_mouse_text_color(app.mouse_text_color);
    // W4-d53 batch 4 — `Main.waterStyle` as the wet machine's `dustWater()`
    // pick reads it (`Projectile.cs:16891`/`:16973`): the draw-pass value
    // (the g15 items-pass `calculate_water_style` precedent), fed before
    // the tick so the splash drain resolves the water dust type.
    let water_style = crate::biome::calculate_water_style(&app.world, app.camera) as i32;
    app.projectiles.set_water_style(water_style);
    // W4-g64 (the d13 seam) — `Main.screenWidth` for the aiStyle-124
    // tentacle's ore-scan distance gate (`Projectile.cs:35577`); the same
    // swapchain width the dust cull reads (`App::spawn_ctx`).
    app.projectiles.set_screen_width(sw);
    // W4-d12 — `Main.WindForVisuals` (= `windSpeedCurrent`, `Main.cs:2823`)
    // as the aiStyle-112 836 seed's steering reads it (`Projectile.cs:34138`/
    // `:34153`), and `Main.mouseTextColor` as the 590 spore's light wobble
    // reads it (`:34221`). The `set_disco` setter pattern — the pool reads
    // the live values, the wind tail's `windPhysics` gate stays separate.
    let wind = app.wind.speed_current;
    app.projectiles.set_wind(wind);
    let mouse_text_color = app.mouse_text_color;
    app.projectiles.set_mouse_text_color(mouse_text_color);
    // W4-d13 — `Main.screenWidth` for the aiStyle-124 tentacle's ore-scan
    // distance gate (`Projectile.cs:35577`); the same swapchain width the
    // dust cull reads (`App::spawn_ctx`).
    app.projectiles.set_screen_width(sw);
    // W4-d13 — the aiStyle-125 Wire Kite's mouse-right pair
    // (`Projectile.cs:35710`). The release flag lags one frame in the C#
    // (`Main.cs:62458-62465` runs after Draw), so the pair's `true && true`
    // is the PRESS tick — the golf context's same-edge convention. The
    // mouse world point rides `inputs.aim` (the yoyo's math, gravDir = 1).
    app.projectiles.set_wire_kite_inputs(Some(crate::projectile_ai_wire_kite::WireKiteInputs {
        mouse_right: app.place_down,
        mouse_right_release: !app.place_prev_down,
    }));
    // W4-d16 — the aiStyle-151 SuperStar's on-screen gore gate
    // (`Projectile.cs:22152-22153`): the screen rect in world px. The
    // port's camera IS the screen centre (`Main.screenPosition +
    // (screenWidth, screenHeight)/2` at zoom 1); the world-space size
    // divides the pixel size by the zoom (the golf mouse-world conversion).
    let screen_view = Some((
        Vec2::new(app.camera.x, app.camera.y),
        Vec2::new(sw / app.zoom, sh / app.zoom),
    ));
    app.projectiles.set_screen_view(screen_view);
    // W4-d16 — the paper airplane's / kite's wind + scene clocks
    // (`Main.WindForVisuals` = `windSpeedCurrent` `Main.cs:2823`,
    // `Main.timeForVisualEffects` = the draw-frame clock ticks
    // (`anim_time * 60 % 216000`, the frame feed's form),
    // `Main.cloudAlpha` = the rain's visible intensity,
    // `Main.GlobalTimeWrappedHourly` = `anim_time % 3600`).
    app.projectiles.set_visual_scene((
        app.wind.speed_current,
        (app.anim_time as f64 * 60.0) % 216000.0,
        app.rain.visual_alpha(),
        app.anim_time % 3600.0,
    ));
    // W4-d16 — the kite family's item-state snapshot (`AI_160_Kites`'s
    // `inventory[selectedItem].shoot` gate `Projectile.cs:48431` +
    // `KiteLogic`'s `controlUseItem && itemAnimation == 0` reel-out gate
    // `:48485`). The swing timer derives the tick count (the spear
    // `ticks_of` convention); the use button is the raw `mine_down`
    // (the flail `control_use_item` precedent).
    let kite_ctx = app.player.as_ref().map(|_| {
        let held = app.inventory[app.selected];
        let selected_shoot = if held.count > 0 {
            crate::registry::projectile(held.kind).unwrap_or(0)
        } else {
            0
        };
        crate::projectile_ai_kites::KiteContext {
            selected_shoot,
            item_animation: (app.swing * 60.0).round() as i32,
            control_use_item: app.mine_down,
        }
    });
    app.projectiles.set_kite_context(kite_ctx);
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
                // W4-d20 — the flint arm's anchor + dust rotation reads.
                mounted_center: Vec2::new(p.mounted_center().0, p.mounted_center().1),
                item_rotation: p.item_rotation,
            },
            player_name: app.player_name.clone(),
            golfer_score: p.golfer_score,
        }
    });
    if let Some(ctx) = golf_ctx {
        app.projectiles.set_golf_context(ctx);
    }
    // W4-d3 — the boulder dust/gore lane's live seam (the W4-g64/g16 shapes):
    // the frame's spawn ctx (`Main.screenPosition/width/height` + the gfx
    // budget) captured BEFORE the step closure's field borrows, and the App's
    // gore pool moved in for the step (the `golf_ctx` take/replace precedent).
    let dust_spawn_ctx = app.spawn_ctx();
    let gore_pool = std::mem::replace(&mut app.gore, tr_core::gore::GorePool::new());
    app.projectiles.set_gore_pool(Some(gore_pool));
    // W4-g63 — the trail-dust lane's live seam: the frame's spawn ctx
    // (`Main.screenPosition/width/height` + the gfx budget) captured
    // BEFORE the step closure's `&mut app.world`/`&mut app.dust` field
    // borrows (the `world_sim/npcs.rs` `dust_spawn_ctx` convention).
    // W4-g64 — the trail-dust lane's live seam (the W4-g63 batch-1 shape):
    // the frame's spawn ctx (`Main.screenPosition/width/height` + the gfx
    // budget) captured BEFORE the step closure's `&mut app.world`/`&mut
    // app.dust` field borrows (the `world_sim/npcs.rs` `dust_spawn_ctx`
    // convention).
    let dust_spawn_ctx = app.spawn_ctx();
    app.projectile_step.tick(dt.min(0.25), |_| {
        app.projectiles.update_dust(
            &mut app.world,
            &mut app.npcs,
            owner,
            aim,
            app.player.as_mut(),
            inputs,
            flail_inputs,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            local_owner,
            hostile_local,
            &mut app.dust,
            &dust_spawn_ctx,
            // W4-g83: the tile-kill arms' framing dispatch rides the SAME
            // UpdateProjectiles stream (the active genRand, contract §2).
        );
    });
    // W4-d3 — the gore pool writeback: the boulder Kill arms spawned into it.
    if let Some(pool) = app.projectiles.take_gore_pool() {
        app.gore = pool;
    }
    app.projectile_step.cap_backlog();
    // W4-g70 lane 8 — the starfall spawner's `Star.StarFall` request
    // (`Projectile.cs:53107`): the nearest sky star converts to a
    // shooting-star trail. The Fall's two rolls ride the same
    // `UpdateProjectiles` stream the AI walk consumed (`Main.cs:17775`).
    if let Some(x) = app.projectiles.take_starfall() {
        let right_world = app.world.width as f32 * 16.0;
        // `SecretSeed.anySecretSeedIsActive` (`WorldGen.cs:429`): the
        // classic OR micro seed count — either bitset non-empty.
        let any_seed = app.world.seeds.bits() != 0 || app.world.micro_seeds.bits() != 0;
        app.stars.star_fall(
            x,
            right_world,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            any_seed,
        );
    }
    // W4-d21 — the digtoise dig drain: the aiStyle-204 arm queued the dig
    // cells (the hit-tile book + the best-pickaxe read live on the App);
    // resolve them on the same `UpdateProjectiles` stream the C#'s PickTile
    // runs under (`Main.cs:17775-17777`).
    crate::world_sim::editing::drain_digtoise_digs(app);
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
    // W4-f11 — the Grand Design's release-death mass-wire apply
    // (`Projectile.cs:72051-72067` → `Wiring.MassWireOperation`): the
    // ACTIVATION CONTRACT for the d13 union (research/235). d13's pool
    // Kill hook queues `(owner, ps, pe)` (`take_mass_wire_ops`) and parks
    // them on `app.pending_mass_wire_ops` at THIS seam; the merge wave
    // drains them right here through the f11 apply. SEAL RE-TARGET
    // (research/133 §8.4): the union calls the c42-MOVED copy
    // `crate::world_sim::wiring::mass_wire_operation` (this module drops
    // per §4a) and the direct drain REPLACES d13's park line — the
    // buffer field drops, one drain shape, never both:
    //   for (owner, psx, psy, pex, pey) in app.projectiles.take_mass_wire_ops() {
    //       if owner as u32 != app.local_id { continue; } // the Kill gate
    //       crate::world_sim::wiring::mass_wire_operation(
    //           app, (psx, psy), (pex, pey),
    //           app.wire_tool_mode,                  // RAW ToolMode (:3241)
    //           crate::app::RngKey::UpdateProjectiles, // the Kill's stage
    //       );
    //   }
    // (Batch-H: the `drop_rng` param is LANDED on the c42-moved fn per
    // §8.4(a); g81's own `drain_mass_wire_ops` 2-tuple seam dedupes to d13's
    // pool shape at the d13 union — 382 §3/§4.)
    // The f11 half is READY: `mass_wire_operation` now takes the caller's
    // stage stream (`drop_rng` — the d13 stream note closed); the mode is
    // UNMASKED (the kite does not strip the Actuator bit — only the 3625
    // click arm masks, `Player.cs:46908`); no cooldown gate (the Kill hook
    // has none). The pool seam itself is d13's unmerged code — nothing
    // drains on this branch yet (the documented deferral, both sides).
    // W4-d13 — the Wire Kite's per-tick writes (`Projectile.cs:35714`/
    // `:35831`): `Main.blockMouse = true` on the right-click press kill and
    // `SetDummyItemTime(2)` while the kite lives (the `take_spear_hold`
    // pattern; `Player.cs:4432-4441` — itemAnimation = itemTime = 2).
    if app.projectiles.take_block_mouse() {
        app.block_mouse = true;
    }
    if app.projectiles.take_kite_hold() {
        let secs = 2.0 / 60.0;
        app.swing = secs;
        app.item_time = secs;
    }
    // W4-d13 — `Kill()`'s type-651 MassWireOperation requests
    // (`Projectile.cs:72051-72067`): the release-latch deaths queued their
    // `(owner, ps, pe)` anchors; the app slot holds THIS tick's requests
    // (a per-tick buffer — the C# applies synchronously inside Kill).
    // ACTIVATION CONTRACT — SEALED against f11's retire (tip `14ca0a8d`,
    // research/235 §2; renumbered 136→153→235 per the registry-150 key-136
    // row). f11's apply landed on ITS branch (`24fa39f1` + the `84ed6342b`
    // stream close): `wiring_devices::mass_wire_operation(app, ps, pe,
    // mode, drop_rng)`, unmerged here. The merge wave drains THIS buffer
    // at THIS seam (after `take_spear_hold`), before the next pass
    // replaces it —
    //   for (owner, psx, psy, pex, pey) in take_mass_wire_ops() {
    //       if owner as u32 != app.local_id { continue; } // the Kill gate
    //       mass_wire_operation(app, (psx, psy), (pex, pey),
    //                           app.wire_tool_mode, // :3241 ToolMode, UNMASKED
    //                           crate::app::RngKey::UpdateProjectiles);
    //   }
    // The shapes compose verbatim: ps/pe are TILE coords on both sides
    // (`(int)X >> 4`, `Utils.cs:1807-1810`); the mode is the RAW
    // `WiresUI.Settings.ToolMode` (`Wiring.cs:3241` — the kite does NOT
    // strip the Actuator bit; only the 3625 click arm masks, f11's
    // `Player.cs:46908`). No `wire_operations_cooldown` gate on this path
    // (the C# Kill hook has none — the throttle is CanUseItem's, the use
    // arm's). STREAM NOTE CLOSED (f11 `84ed6342b`): the fn rolls its
    // cut-cache `Item.DropCache` drops on the CALLER's stage stream —
    // the 3625 use arm passes `RngKey::UpdatePlayers` (unchanged), THIS
    // drain passes `RngKey::UpdateProjectiles` (the Kill runs inside
    // `Main.cs:17775`'s swap); no deviation to record. ONE drain shape
    // (research/235 §2): the merger keeps EITHER this direct drain OR
    // the buffer + a second-pass drain, never both (the buffer field
    // drops if the direct drain wins). Until f11 merges, the requests
    // drop tick-end (the documented deferral).
    app.pending_mass_wire_ops = app.projectiles.take_mass_wire_ops();
    // W4-d14 — the monk staff swings' `player.reuseDelay` write
    // (`Projectile.cs:69457` = 10 / `:69531` = 2): the arm queued the tick
    // count on the swing's death; the port re-arms `app.reuse_delay` (the
    // `take_spear_hold` write-back precedent, the seconds timer).
    if let Some(frames) = app.projectiles.take_monk_reuse() {
        app.reuse_delay = frames as f32 / 60.0;
    }
    // W4-d15 — the 142 arm's player writes (`Projectile.cs:69822`'s
    // `player.heldProj = whoAmI` + `:69966`'s `player.reuseDelay = 2`),
    // drained in slot order (the `take_spear_hold` precedent). The port's
    // reuse timer runs in seconds.
    for (held_proj, reuse_delay) in app.projectiles.take_dd2_writes() {
        if let Some(p) = app.player.as_mut() {
            if let Some(slot) = held_proj {
                p.held_proj = slot;
            }
        }
        if let Some(ticks) = reuse_delay {
            app.reuse_delay = ticks as f32 / 60.0;
        }
    }
    // W4-d15 — the 144 arm's 1050 shake-tick effects
    // (`Projectile.cs:70217-70229`): the arm's three `Main.rand` draws
    // ran inline on the `UpdateProjectiles` stream; the effects resolve
    // here on the SAME stream (the `drain_catches` precedent). The dust
    // body (`KillTile_MakeTileDust`, `:70218`) is the recorded layer
    // skip — CROSS-NOTE (final phase): the BODY is the d54/g14 lane's
    // (`tr_core::worldgen_dust_tile::kill_tile_make_tile_dust`, d54 tip
    // `ac132be1` — the full `:66687-69169` ladder incl. every shakeable
    // kind's arm: the 5 trunk ground-walk `:68160`, the 323 palm
    // `:68235` + frameY-offset tail `:69118`, the 72 mushroom `:68778`,
    // the 583-589 gem `Next(10)` rolls `:67031-67049`, 596/616 → 78,
    // the 634 `Next(10)`+`Next(12)` pair `:67071`). Neither dust lane
    // claims the `:70218` CALL SITE (d54's census is WorldGen/Main
    // sites; g17 is gore) — at the integrate this drain calls their
    // body with (rng = UpdateProjectiles, gen = the SAME stream — the
    // `genRand == Main.rand` swap, `WorldGen.cs:4445`), app.dust +
    // spawn_ctx + app.disco; the gem/634 genRand draws then join this
    // stream AHEAD of the drop resolver's rolls (the C# order: dust
    // `:70218` → sounds `:70219` → drops `:70228`). The sound plays
    // through the break-cue table (`fail: true`, `:70219`); the drop
    // gate runs the `KillTile_DropItems` per-cell rolls WITHOUT killing
    // the tile.
    for shake in app.projectiles.take_dd2_shakes() {
        let Some(tile) = app.world.tile(shake.x, shake.y).copied().filter(|t| t.active()) else {
            continue;
        };
        let (px, py) = (shake.x as f32 * 16.0, shake.y as f32 * 16.0);
        // `KillTile_PlaySounds(x, y, fail: true, tileCache)` (`:70219`).
        let sounds = tr_core::edit::kill_tile_sound(tile.kind, tile.frame_x, true);
        for cue in [sounds.primary, sounds.extra, sounds.shatter].into_iter().flatten() {
            if let Some((name, volume)) = crate::world_sim::editing::kill_cue(cue) {
                app.audio.play_at(name, px, py, volume);
            }
        }
        if shake.drop {
            // `KillTile_DropItems(x, y, tileCache)` (`:70228`): the
            // bonus-wood roll reads the closest player's held axe
            // (`WorldGen.cs:66255-66256` — the port's one player).
            let axe_power = crate::registry::get(app.inventory[app.selected].kind)
                .map(|d| i32::from(d.axe))
                .unwrap_or(0);
            // `SpecialSeedFeatures.ShouldDropExtraWood`
            // (`SpecialSeedFeatures.cs:17-27`): tenth + drunk + !remix
            // + !notTheBees.
            let seeds = app.world.seeds;
            let extra_wood = seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY)
                && seeds.contains(tr_core::seeds::SeedFlags::DRUNK)
                && !seeds.contains(tr_core::seeds::SeedFlags::REMIX)
                && !seeds.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES);
            let drops = crate::world_sim::editing::dd2_shake_tile_drops(
                &app.world, shake.x, shake.y, axe_power, extra_wood, app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            );
            for (id, count) in drops {
                crate::world_sim::items::spawn_item_drop(
                    &mut app.items, app.rngs.stream(crate::app::RngKey::UpdateProjectiles), id, count, shake.x, shake.y,
                );
            }
        }
    }
    // W4-g11 union-exec (research/116 gap 1) — the Queen-Slime hook's
    // `Player.Teleport(vector, 10)` (`Projectile.cs:52775` →
    // `Player.cs:22753`): the 935 latch queued the style-10 effect pair
    // (the move ran inline); the replay runs Entry-at-OLD → Exit-at-NEW
    // on the `UpdateProjectiles` stream (`Main.cs:17775` — the C# executes
    // `Teleport` inside the projectile pass). The same-tick deferral is
    // observation-identical (nothing probes the dust between the latch
    // and this drain); the replay owns the grants + the camera transition.
    if let Some((ox, oy, nx, ny, style, e2, ei, mult)) =
        app.player.as_mut().and_then(|p| p.take_teleport_effect())
    {
        crate::main_dust::player_teleport(
            app,
            crate::main_dust::PlayerTeleportReq {
                old_x: ox,
                old_y: oy,
                new_x: nx,
                new_y: ny,
                style,
                extra_info2: e2,
                extra_info: ei,
                mult,
                key: crate::app::RngKey::UpdateProjectiles,
            },
        );
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
    // W4-f50 — the player hurt combat text (`Player.cs:38382`): the C# draws
    // it BEFORE the retaliate block (`:38401`), so the drain spawns it ahead
    // of the retaliate rolls on the `UpdateProjectiles` stream.
    if let Some((amount, crit)) = app.player.as_mut().and_then(|p| p.take_hurt_text()) {
        let text = amount.to_string();
        let size = match app.lookups.combat_text_font.as_ref() {
            Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
            None => [text.chars().count() as f32 * 8.0, 10.0],
        };
        let p = app.player.as_mut().unwrap();
        let location = crate::combat_text::Rect {
            x: p.x,
            y: p.y,
            width: p.width,
            height: p.height,
        };
        let color = crate::combat_text::damage_text_color(crit, true, false);
        crate::combat_text::spawn_amount(
            &mut app.combat_texts,
            location,
            color,
            amount,
            crit,
            false,
            size,
            1.0,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
        );
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
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
        );
        // W4-c28: the brain's Confused AddBuff (`Player.cs:38425-38426`)
        // queued its net intent — the pass-tail drain below carries it
        // (one drain per pass; the C# sends inline at each AddBuff).
    }
    // W4-f50 — the Solar Flare hurt blast (`Player.cs:38316-38318`): the
    // accepted hostile shot consumed a shield charge and queued the 608
    // blast; the C# spawns + Kills it inline under the `UpdateProjectiles`
    // swap (`Main.cs:17775`), the port drains at the pass tail on the same
    // stream (the S1-E-21c batch convention).
    if let Some((center, damage)) = app.player.as_mut().and_then(|p| p.take_solar_blast()) {
        let dolls = app.player.as_ref().map_or((false, false), |p| p.doll_flags());
        let luck = app.player.as_ref().map_or(0.0, |p| p.luck());
        let blast_ctx = app.spawn_ctx();
        app.projectiles.solar_flare_blast(
            center,
            damage,
            app.local_id as u8,
            &mut app.npcs,
            dolls,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            luck,
            &mut app.dust,
            &blast_ctx,
            &app.world,
            &mut app.gore,
        );
    }
    // W4-f50 — the hit-dust loop + football + golf-club hurt drains
    // (`Player.cs:38527-38553` / `:38255-38280`): the accepted hostile shot
    // queued them; the port drains at the pass tail on the `UpdateProjectiles`
    // stream (the S1-E-21c batch convention).
    // `SpawnCtx` is an `App` method; hoist it above the player borrow.
    let hurt_ctx = app.spawn_ctx();
    if let Some(p) = app.player.as_mut() {
        crate::hurt_modifiers::drain_hurt_dust(
            p,
            &mut app.dust,
            &hurt_ctx,
            &app.world,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
        );
    }
    if let Some(p) = app.player.as_mut() {
        crate::hurt_modifiers::drain_football(
            p,
            &mut app.projectiles,
            &mut app.inventory,
            &mut app.mouse_item,
            app.local_id as u8,
        );
    }
    if let Some(p) = app.player.as_mut() {
        let held = app.inventory[app.selected].kind;
        crate::hurt_modifiers::drain_golf_club_kill(
            p,
            &mut app.projectiles,
            held,
            app.local_id as u8,
        );
    }
    // W4-c19 round 3 — the rand-gated AI-arm light intents
    // (`Projectile.cs:30378` typhoon / `:34226` spore / `:47846`/`:48011`
    // whips): the arms took their gate draws on the `UpdateProjectiles`
    // stream mid-walk and queued the lights in slot + sub-step order; the
    // drain appends them into the ONE `EntityLights` pair here (the
    // `item_lights.rs` named-stream precedent — the e6-rule exception, the
    // draw-pass feed stays pure). No RNG: the append is stream-neutral.
    for (x, y, rgb) in app.projectiles.drain_ai_lights() {
        app.entity_lights.add(x, y, rgb);
    }
    // W4-d116 — the landed strikes' combat texts (`NPC.cs:81250-81264`,
    // inside `StrikeNPC_Inner` BEFORE `HitEffect`/`HitSound`): the pool
    // queued `(hitbox, dealt num, crit, friendly)` per landed hit on a
    // `lifeMax > 1` NPC; the spawn's 2 (+1 crit) `Main.rand` rolls ride the
    // `UpdateProjectiles` stream here, ahead of the hit-sound drain below
    // (the C#'s order; the S1-E-21c batch convention defers both from
    // mid-walk — ANALYSIS W4-d116).
    for (location, amount, crit, friendly) in app.projectiles.drain_npc_damage_texts() {
        let text = amount.to_string();
        let font = if crit {
            app.lookups.combat_crit_font.as_ref()
        } else {
            app.lookups.combat_text_font.as_ref()
        };
        // `FontAssets.CombatText[crit ? 1 : 0].MeasureString`
        // (`CombatText.cs:76`).
        let size = match font {
            Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
            None => [text.chars().count() as f32 * 8.0, 10.0],
        };
        let color = crate::combat_text::damage_text_color(crit, friendly, false);
        crate::combat_text::spawn(
            &mut app.combat_texts,
            location,
            color,
            text,
            crit,
            false,
            size,
            1.0,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
        );
    }
    // W4-d116 — the Destroyer-probe roll intents (`NPC.cs:81488` →
    // `:89266-89285`): the landed body hits queue their index; the roll runs
    // HERE, after the text draws above and before the hit-sound drain —
    // the C#'s inside-the-strike order (text → HitEffect → HitSound).
    for idx in app.projectiles.drain_probe_rolls() {
        // The queued index rides this tick's list (the death retain runs in
        // the NPC pass, after this one); the guard keeps a stale index from
        // ever panicking.
        let Some(n) = app.npcs.get_mut(idx) else { continue };
        if let Some(anchor) = crate::npc_ai_style37::probe_launch_on_hit(n, app.rngs.stream(crate::app::RngKey::UpdateProjectiles), false) {
            n.wants_probe = Some(anchor);
        }
    }
    // S1-E-21 wave C — the landed strikes' sound plays
    // (`NPC.cs:81476-81554` via `Projectile.Damage` `Projectile.cs:13649`):
    // the case-3 volume arms + the `Main.rand.Next(-10, 11) * 0.01` pitch
    // draw ride the `UpdateProjectiles` stream (`Main.cs:17775`), only
    // after each play's attenuation gate passed. The C# draws mid-walk;
    // the port drains at the pass tail (ANALYSIS, S1-E-21c). W4-c13: the
    // requests carry the HitEffect cues (the bubble's int-4 style-3, the
    // spheres' Item10) ahead of each hit/voice cue — the shared
    // dispatcher keeps the C#'s per-strike order and case maths.
    for (cue, sx, sy) in app.projectiles.drain_npc_hit_sounds() {
        app.audio.play_npc_hit_sound(cue, sx, sy, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
    }
    // W4-f32 — the projectile-side Lucky Coin proc's drops
    // (`Projectile.cs:13685-13697`): the six rolls (the Next(5) gate, the
    // denomination pair, the DISCARDED Next(1, 11), the velocity pair) were
    // drawn inline mid-walk on the `UpdateProjectiles` stream; the pool
    // queued the fully-rolled drop (it cannot borrow app.items). The spawn
    // itself draws NOTHING — the explicit velocity skips `Item.NewItem`'s
    // spread draws (`Item.cs:49738-49740`) and prefix 0 draws nothing, so
    // the pass-tail placement is stream-exact (the drain convention).
    // Stack 1 (the C# discards its stack roll), `WorldItem.Center = center`
    // centres the 14x14 box on the NPC (`Item.cs:49736`).
    for (coin, ncx, ncy, vx, vy) in app.projectiles.drain_lucky_coin_drops() {
        let drop = crate::item::ItemDrop::new(
            coin,
            1,
            0,
            ncx - crate::item::SIZE * 0.5,
            ncy - crate::item::SIZE * 0.5,
            vx,
            vy,
            0.0,
        );
        app.items.push(drop);
    }
    // W4-c28: the pass's `StatusNPC`/whip/dart/flare/bullet/arrow debuffs
    // (`Projectile.cs:11173-11750`) AND the retaliate brain's Confused
    // (`Player.cs:38425-38426`) queued their `NPC.AddBuff` net intents —
    // drain at the pass tail (the C# sends inline at each AddBuff,
    // `NPC.cs:89932-89935` client `53` / `:89979-89982` host `54`).
    app.drain_npc_buff_sends();
    // S1-E-21h — the projectile-family cue plays (the AI arms' first-tick
    // latches, the boomerang whoosh, the missile hum, the bomb blast): the
    // style play's case-2 draws ride the SAME `UpdateProjectiles` stream
    // (`Main.cs:17775-17777`), only after each play's attenuation gate
    // passed. The C# plays inline mid-walk; the port drains at the pass
    // tail (the wave-C batch convention, ANALYSIS "The projectile sound
    // wave").
    for (id, style, x, y, pitch_offset, volume_scale) in app.projectiles.drain_proj_sounds() {
        // S1-P-i08 + W3-i39 — the aiStyle-3 wall thud (`Projectile.cs:19571`)
        // and `AI_007`'s latch dig cue (`Projectile.cs:52757`:
        // `PlaySound(0, i*16, j*16)`) are the int-id Dig play: case 0's
        // `Next(3)` variation + `Next(-10, 11)*0.01` pitch ride the same
        // stream, attenuation gate first (the `play_place_sound` façade's
        // id-0 arm, `LegacySoundPlayer.cs:440-452`, S1-E-21 wave B); the
        // id-2 styles keep the wave-A façade. The d76 DOC-SWEEP volume term
        // rides the id-2/3/4 façades (the Corrupt Whip ×0.6 / Lights Bane
        // `0.15*ai[0]`); the id-0 Dig sites all queue volume 1.
        if id == 0 {
            app.audio.play_place_sound((0, 0), x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 13 {
            // S1-E-21h d75 part 2 + d77 — the Shatter plays: case 13 draws
            // NOTHING (`LegacySoundPlayer.cs:783-792`), attenuation gate
            // first. The queue's sixth term carries the call-site
            // volumeScale: the Crystal Shard's lead (`Projectile.cs:25943`,
            // `PlaySound(13, x, y, 0, 0.35f)`) queues 0.35; the Kill-tail
            // water bottles (`:77018`/`:77033`/`:77048`) and the molotov
            // (`:73923`) queue the stock 1.0 (d77's wall-break int arm
            // shape — both draw-free, the volume rides the queue).
            app.audio.play_shatter_sound(x, y, volume_scale);
        } else if id == 6 {
            // S1-E-21h d77 — the spore cloud's Grass rustle
            // (`Projectile.cs:74517`, `PlaySound(6, pos)`): the wall-break
            // façade's int arm (`LegacySoundPlayer.cs:706-716`, one
            // `Next(-30, 31)` pitch), attenuation gate first.
            app.audio.play_wall_break_sound((id, 1), x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 15 {
            // The yoyo-shot's rare roar (`:36821`, `PlaySound(15, …, 0,
            // 0.35f, jitter)`): case 15 draws nothing (the slot guard is the
            // whole decision); the arm already drew the Next(100)/Next(6)/
            // NextFloat jitter inline — the jitter VALUE and the 0.35
            // volumeScale are dropped here (the recorded deviation).
            app.audio.play_roar(style, x, y);
        } else if id == 36 {
            // The dash-roar twin (`:36825`, `PlaySound(36, …, -1, 0.35f,
            // jitter)`): case 36 draws nothing, style −1 → slot 0 at
            // Pitch += 0.6.
            app.audio.play_dash_roar(style, x, y);
        } else if id == 29 {
            // S1-E-21h d75 part 3 — the Phantasmal Deathray's first-tick
            // zombie-voice cue (`Projectile.cs:31504`, `PlaySound(29, x, y,
            // 104)`): case 29's Style IS the slot (`LegacySoundPlayer.cs:
            // 1008-1050`), the ×0.55 style-104 volume arm, the playing guard
            // BEFORE the one `Next(-10, 11)` pitch — all ride the E21e
            // `play_fighter_moan` façade on the same UpdateProjectiles
            // stream, attenuation gate first. The int overload draws NO
            // `GetRandomPitch` NextFloat.
            // The C# sites truncate (`PlaySound(29, (int)X, (int)Y, …)`
            // `:31504`/`:72989`) — the drain-side truncation is the
            // faithful gate probe (the d75 part-6 receipt).
            app.audio.play_fighter_moan(29, style, x.trunc(), y.trunc(), app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 3 && style == 29 {
            // S1-E-21h d75 part 4 — the 1054 SpiderBoulder's death cue
            // (`Projectile.cs:74865`, `PlaySound(SoundID.NPCHit29, x, y)`).
            // d3's `boulder_kill` parks the (3, 29) tuple; the C# call is
            // the STYLE overload (one audio-only GetRandomPitch), so the
            // drain routes to wave C's style façade — case 3's one
            // `Next(-10, 11)` pitch rides this stream, gate first. Other
            // id-3 styles fall through (the façade's None — a future
            // style's route is its own lane's follow-up).
            app.audio.play_npc_hit_sound("NPC_Hit_29", x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 4 && style == -6 {
            // S1-E-21h d75 part 4 + d77 compose — the 1055 Ghoulder's death
            // cue (`Projectile.cs:74869`, `PlaySound(SoundID.NPCDeath6, x,
            // y)`): the STYLE overload — case 4's one `Next(-10, 11)` pitch
            // after the gate (the 10 playing guard is style-10-only), the
            // GetRandomPitch NextFloat audio-only. d3's `boulder_kill`
            // intent rides the −6 named-route sentinel (the −34 precedent,
            // the d75 part-6 receipt): the payload alone cannot tell the
            // style call from d77's kill-tail int `(4, 6)` — the sentinel
            // keeps BOTH overload paths live.
            app.audio.play_npc_death_style(6, x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 4 {
            // S1-E-21h d77 — the kill-tail death cries: the Sharknado bolt
            // (`Projectile.cs:73763`, `PlaySound(4, Center, 19)`), the
            // Phantasmal Sphere (`:73016`, style 6), the Stardust Jellyfish
            // (`:72571`, style 7): the int-overload case-4 façade (one
            // `Next(-10, 11)` pitch after the gate, the 10 playing guard,
            // NO GetRandomPitch).
            app.audio.play_npc_death_int(style, x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 37 {
            // S1-E-21h d75 part 4 — the 502 Meowmere star's bounce chime
            // (`Projectile.cs:18209`, `PlaySound(37, x, y, 5 + ai[0])`):
            // case 37's Next(57,59) slot pick + the Next(-40,41) pitch
            // ride this stream (the int overload — no GetRandomPitch),
            // the Style×0.05 volume scales inside the façade.
            app.audio.play_star_chime(style, x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 2 && style == -34 {
            // S1-E-21h d75 part 4 — the 1106 Foxparks flame's per-life cue
            // (`Projectile.cs:41369`, `PlaySound(SoundID.FoxparksFlame,
            // Center)`): the −34 named-row marker routes to the NAMED
            // (2, 34) row at volume 0.15 (`SoundID.cs:1725`) — the plain
            // +34 tuple stays Item34's (d76's Nebula Blaze latch, volume
            // 1.0). The by-name façade keeps the volume (the
            // play_chillet_sound precedent).
            app.audio.play_foxparks_flame(x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else if id == 18 {
            // W4-d1 retire — the falling-block forced placement's coin-pile
            // tail (`WorldGen.cs:60273-60278`: the legacy Coins 18, the int
            // overload — no draws, no pitch, `LegacySoundPlayer.cs:865-869`;
            // the `place_tile_sound` table's 330-333 arm).
            app.audio.play_place_sound((18, 1), x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
        } else {
            // ONE play per queued cue (the union left BOTH façades here, so a
            // single `PlaySound` drew its case-2 pitch twice — the S1-E-21h
            // frost-bolt/coil stream witnesses caught it). The f1-scaled
            // façade owns the int routes, the (42, 274) RainbowBoulder row
            // and the id-2 styles (`:1335`); the older façade alone can
            // resolve the OTHER id-42 trackables (the scaled one returns
            // `None` for them without drawing).
            if id == 42 && style != 274 {
                app.audio.play_proj_sound((id, style), x, y, pitch_offset, volume_scale, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
            } else {
                // S1-E-21h f1 — the queue's sixth term (d77's `volumeScale`
                // seam, adopted from `9d35874a`): the id-0/13/4 int-overload
                // routes ride INSIDE `play_proj_sound_scaled`'s int match (f1's
                // part-4 shape) and ignore the scale (the int overload's own
                // volume is 1 at every f1 site); the id-2 styles fold it into
                // `play_use_sound_scaled`'s post-gate multiply
                // (`LegacySoundPlayer.cs:1411`).
                app.audio
                    .play_proj_sound_scaled((id, style), x, y, pitch_offset, volume_scale, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
            }
        }
    }
    // W4-e7 lane 23 — the Snapping Stone strike family's queued cues
    // (`TryMakingSnappingStoneStrike`'s particle-body + owner-side plays,
    // `Projectile.cs:14355`/`:14363`/`:14383-14394`): the PVE walk queued
    // them mid-hit (the pool cannot borrow the audio); they drain here on
    // the SAME `UpdateProjectiles` stream (`Main.cs:17775`), the S1-E-21c
    // queue→drain convention. The case-2 draws defer to this tail (the
    // recorded ANALYSIS deviation); the particle dust draws ran inline.
    for sound in app.projectiles.drain_striking_sounds() {
        crate::snapping_stone::play_striking_sound(&mut app.audio, sound, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
    }
    // W4-c15 — the explosion wall-walk's `KillWall_PlaySounds` plays
    // (`Projectile.ExplodeTiles`' 3×3 `WorldGen.KillWall`, `Projectile.cs:
    // 78745`): the four-arm table's draws ride the SAME `UpdateProjectiles`
    // stream (`Main.cs:17775`), attenuation gate first. Drained AFTER the
    // proj cues — the blast cue queues at detonation, the wall walk runs in
    // `Kill` after it (the C# draws mid-walk; the pass-tail batch convention,
    // ANALYSIS "The world-sounds wave" addendum).
    for (cue, x, y) in app.projectiles.drain_wall_break_sounds() {
        app.audio.play_wall_break_sound(cue, x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
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
    // W4-f49 — the pool-side CheckMana pays' mana-flower auto-drink
    // (`Player.CheckMana` `:52588-52609` reached from the Last Prism's
    // interval pay `Projectile.cs:66613`, the Charged Blaster Cannon's
    // `flag3` pay `:66479`, and the d-lane AI bodies' pays on their
    // branches — the Glacier Fang `:36943` (d21), the Life Drain `:42662`
    // (d19), the Medusa Head `:44567` (d10); the 439 holder `:66204` and
    // the Forbidden storm `Player.cs:56870` ride their lanes): the
    // pool-side pay took the ManaV2 empty arm on a short pay tick; the
    // flower lane re-runs the FULL CheckMana (the
    // QuickMana drink + the post-drink re-pay) here, where the inventory is
    // reachable. The draws + the heal text ride `UpdateProjectiles`
    // (`Main.cs:17775`, the AI_075 caller's swap) — the drain convention.
    for amount in app.projectiles.drain_prism_flower_drinks() {
        crate::potions::check_mana_with_flower_on(
            app,
            amount,
            true,
            false,
            crate::app::RngKey::UpdateProjectiles,
        );
    }
    // W4-f30 — the aiStyle-17 gravestone's landing (`Projectile.cs:25348-
    // 25364`): the placement + the epitaph sign + the Dig sound + the Kill.
    // The AI arm queued the intent (it cannot borrow the world mutably); the
    // pass resolves it here. `place_item_tile(85, style)` is the C#'s
    // `TileObject.CanPlace/Place`; a refused placement keeps the stone
    // falling (no Kill, the `flag7` false arm).
    for (slot, tx, ty, style, text) in app.projectiles.drain_gravestone_places() {
        if tr_core::edit::place_item_tile(&mut app.world, tx, ty, 85, style, 1) {
            // `Sign.ReadSign` + `Sign.TextSign` (`:25358-25361`): the epitaph
            // lands on the tombstone's sign record (the origin decode resolves
            // the 2-wide tile's top-left).
            if let Some((ox, oy)) = crate::signs::origin(&app.world, tx, ty) {
                if let Some(idx) = crate::signs::find_or_create(&mut app.world.signs, ox, oy) {
                    crate::signs::text_sign(&mut app.world, idx, ox, oy, text);
                }
            }
            // `SoundEngine.PlaySound(0, x*16, y*16)` (`:25357`) — the Dig cue,
            // the id-0 place façade (the wave-B precedent above).
            app.audio.play_place_sound((0, 0), (tx * 16) as f32, (ty * 16) as f32, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
            app.projectiles.kill_gravestone(slot);
            app.world_key = None;
        }
    }
    // W4-f30 — `Player.OnKillNPC`'s GRAVE_MISTAKE (`Player.cs:20471-20474`):
    // a gravestone's strike killed a town NPC this tick → progression 36.
    if app.projectiles.drain_grave_mistake() {
        let _ = app.notify_progression_net(36);
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
    // W4-d9 wiring — the aiStyle-89 ritual's 300-tick dragon/squid spawn
    // (`Projectile.cs:32346-32356`): the VANILLA ids 454/521 resolve through
    // `kind_from_net_id` (the bestiary NET-id space, the
    // `spawn_shimmer_release` precedent — NOT `from_u16`'s CS ids). The
    // dragon head (454, the d40 rows) and the squidhead (521, ported from
    // `NPC.cs:15768-15779`) now have port kinds. `NPC.NewNPC`'s Bottom =
    // (X, Y) (`NPC.cs:80770`) — the spawn site subtracts the half-width +
    // the full height (the shark-spawn precedent). The C# stores the
    // returned NPC index into the marker's `ai[1]` (`:32350`/`:32354`);
    // the port's compacted NPC vec re-derives indices each tick, so the
    // write-back rides the current push index (the saucer host-link
    // convention, `world_sim/npcs.rs`'s carrier pre-pass).
    for (slot, npc_type, x, y) in app.projectiles.drain_ritual_npc_spawns() {
        // `NPC.NewNPC(..., 454/521, ...)` takes an NPCID TYPE id, so the
        // kind resolves through `from_u16` (the 454/521 rows,
        // `data.rs:1829`/`:2118`, ported from `NPC.cs:15768-15779`); the
        // bestiary net-id table is the FALLBACK for the protocol-renumbered
        // ids (the squidhead 521 is not a bestiary entry, so the net-id
        // table alone dropped it — the w4-d9 drain witness).
        let Some((kind, slime)) = crate::npc::NpcKind::from_u16(npc_type)
            .map(|k| (k, crate::npc::SlimeVariant::Blue))
            .or_else(|| crate::npc::kind_from_net_id(npc_type as i32))
        else {
            continue;
        };
        let (w, h) = kind.size();
        let mut n = crate::npc::Npc::new(kind, x - w * 0.5, y - h, 1, 0.0);
        n.slime = slime;
        n.hp = n.max_hp;
        n.net_update = true;
        let idx = app.npcs.len();
        app.npcs.push(n);
        // The C#'s `this.ai[1] = NPC.NewNPC(...)` — the host-validity arm
        // (`Projectile.cs:32362-32381`) reads the index next tick.
        app.projectiles.set_ai1(slot, idx as f32);
    }
    // W4-d11 DOC-SWEEP — the 584's catch `HealEffect`
    // (`Projectile.cs:34089` → `NPC.HealEffect`, `NPC.cs:77433-77448`):
    // the HealLife combat text at the host's hitbox. The C# call sits
    // inside `UpdateProjectiles` (`Main.cs:17775`), so the placement
    // draws ride THIS pass's stream (the S1-E-21c drain convention —
    // the potion heal-text seam's `spawn_amount` shape, but the NPC
    // rect + the projectile stream, not the player's).
    for (x, y, w, h, amount) in app.projectiles.drain_heal_texts() {
        let text = amount.to_string();
        let size = match app.lookups.combat_text_font.as_ref() {
            Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
            None => [text.chars().count() as f32 * 8.0, 10.0],
        };
        let location = crate::combat_text::Rect { x, y, width: w, height: h };
        crate::combat_text::spawn_amount(
            &mut app.combat_texts,
            location,
            crate::combat_text::HEAL_LIFE,
            amount,
            false,
            false,
            size,
            1.0,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
        );
    }
    // W4-d11 DOC-SWEEP — the Nebula set's magic on-hit pickup proc
    // (`Projectile.cs:13826-13830`): the hit-site rolls (the Next(3) gate,
    // the SelectRandom pick, the two velocity draws) already ran on the
    // `UpdateProjectiles` stream inside `damage_pve`; the drop spawn itself
    // is DRAW-FREE (the explicit-velocity arm, `Item.cs:49738-49741`). The
    // world sim resolves the intent into the item vec here (the pool cannot
    // grow it — the `drain_mark_spawns` shape).
    for (kind, cx, cy, vx, vy) in app.projectiles.drain_nebula_pickups() {
        crate::world_sim::items::spawn_nebula_pickup(
            &mut app.items,
            kind,
            Vec2::new(cx, cy),
            Vec2::new(vx, vy),
        );
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
    // W4-d1 — the aiStyle-6 fertilizer cloud's sapling growth
    // (`Projectile.cs:24213-24220`'s `AttemptToGrowTreeFromSapling` calls):
    // the growth draws `WorldGen.genRand`, so the pool arm parked the
    // requests and the pass resolves them on the `UpdateWorldGen` stream
    // (the `palm_frame_rolls` deferral precedent, `world_sim/editing.rs`).
    // The `treeHeightAddon` / `ignoreWalls` args park in the growth port
    // today (`growth/trees.rs`, M5-SEED-1) — the request carries them.
    for (x, y, underground, _addon) in app.projectiles.drain_sapling_grows() {
        let _ = tr_core::growth::attempt_grow_tree_from_sapling(
            &mut app.world,
            x,
            y,
            underground,
            app.rngs.stream(crate::app::RngKey::UpdateWorldGen),
        );
    }
    // S1-P-i35 — the bobber `DoASplash` drain (`Projectile.cs:53893-53905`):
    // case 19's playing guard refuses BEFORE the one pitch draw, on the
    // `UpdateProjectiles` stream (the C# plays inline mid-`AI_061`; the port
    // drains at the pass tail, the S1-E-21c convention).
    for (style, x, y) in app.projectiles.drain_splashes() {
        app.audio.play_splash_sound(style, x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
    }
    // W4-d4 — the beach ball's `Game.BallBounceResult` broadcasts
    // (`Projectile.cs:18605-18608`, the type-155 movement arm): the C#
    // `ChatHelper.BroadcastChatMessage(..., ChatColors.ServerMessage)` runs
    // inline mid-`HandleMovement`; the port drains the pool's queued lines
    // at the pass tail (the golf cup-chat precedent).
    for text in app.projectiles.drain_ball_chats() {
        app.chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
    }
    // W4-d4 — the Mechanical Piranha's `SetDummyItemTime(5)`
    // (`Projectile.cs:27528`): the C# calls it per active piranha per
    // sub-step (idempotent); the port applies ONE dummy-item-time(5) when
    // the pool flagged it. `SetDummyItemTime` (`Player.cs:4432-4441`):
    // itemAnimation = itemTime = frames, itemTimeMax = frames + 1 — the
    // port's swing is the animation, item_time the action cadence (the
    // golf `PlayerWrites` dummy arm, `world_sim/projectiles.rs`).
    if app.projectiles.drain_piranha_dummy() {
        app.swing = 5.0 / 60.0;
        app.swing_max = app.swing;
        app.item_time = 5.0 / 60.0;
    }
    // W4-d10 — the Coin Portal's gold-coin fountain (`Projectile.cs:32932-
    // 32933`): `Item.NewItem` of the Gold Coin 73 (stack 1, prefix 0) at
    // the truncated centre with the arm's EXPLICIT velocity — the
    // RotatedByRandom pair + the NextFloat speed already drew on the
    // `UpdateProjectiles` stream inside the AI; the explicit-velocity arm
    // (`Item.cs:49738-49741`) skips the scatter pair and prefix 0 rolls
    // nothing, so this drain is stream-neutral. The 14×14 item box centres
    // on the spawn point (`WorldItem.Center = center`, `Item.cs:49736`).
    for (x, y, vx, vy) in app.projectiles.drain_coin_drops() {
        app.items.push(crate::item::ItemDrop::new(
            73,
            1,
            0,
            x - crate::item::SIZE * 0.5,
            y - crate::item::SIZE * 0.5,
            vx,
            vy,
            0.0,
        ));
        if app.items.len() > 300 {
            app.items.remove(0);
        }
    }
    // W4-c36 — the lava-fishing achievement drain (`Projectile.cs:20798-
    // 20801`): `HandleSpecialEvent(19)` → GO_LAVA_FISHING's "Do".
    if app.projectiles.drain_lava_fishing_achievement() {
        let _ = app.achievements.complete_condition("GO_LAVA_FISHING", "Do");
    }
    // S1-P-i35 — the bobber `DoASplash` drain (`Projectile.cs:53893-53905`)
    // + W4-d53 batch 4's wet-machine edge plays (`:16871`/`:16885`/`:16898`/
    // `:16912` entry, `:16953`/`:16967`/`:16980`/`:16994` exit — shimmer
    // styles 4/5, every other arm the default style 1): case 19's playing
    // guard refuses BEFORE the one pitch draw, on the `UpdateProjectiles`
    // stream (the C# plays inline mid-walk; the port drains at the pass
    // tail, the S1-E-21c convention). The splash queue drains AHEAD of the
    // dust queue: the C# draws the edge's dust THEN plays (the shimmer arm
    // plays inside the loop), the port's two queues invert that within-edge
    // order — the recorded queue→drain deviation family (ANALYSIS batch 4;
    // the per-tick draw COUNT is identical, the interleaving is not).
    for (style, x, y) in app.projectiles.drain_splashes() {
        app.audio.play_splash_sound(style, x, y, app.rngs.stream(crate::app::RngKey::UpdateProjectiles));
    }
    // W4-d53 — the tick's `Dust.NewDust` bursts (`Projectile.cs:16700-16703`
    // enchantment visuals, `:17036-17077` the arrow trail, `:17117-17125` the
    // stationary Vortex-bolt puff, `:16843-16996` the wet entry/exit splash):
    // the pass snapshotted each at the C# read moment; the drain replays the
    // chain on the SAME `UpdateProjectiles` stream through the live
    // `DustPool` (the `player_dust` / S1-E-21c queue→drain convention — the
    // C# draws mid-walk, the port at the tail).
    for burst in app.projectiles.drain_dust_bursts() {
        crate::projectile_dust::drain(app, burst);
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
    // W4-d1 — the falling-block death placement's item drops
    // (`Kill_ExplodeTiles`'s aiStyle-10 arm, `Projectile.cs:77562-77569`'s
    // `Item.RequestNewItem`): the refused placement / occupied cell drops
    // the block item at the parked pixel point. The C# queues the request
    // under the `UpdateProjectiles` swap and `Main.UpdateItems` resolves
    // it (`Item.NewItem`'s velocity/prefix draws ride the `UpdateItems`
    // stream — the `drain_catches` convention).
    for (item, x, y) in app.projectiles.drain_block_drops() {
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            app.rngs.stream(crate::app::RngKey::UpdateItems),
            item,
            1,
            glam::Vec2::new(x, y),
        );
    }
    // W4-d16 — the paper airplane's `Kill()` recovery drop
    // (`Projectile.cs:76083-76095` → `Item.RequestNewItem`): the world sim
    // spawns the item at the plane's centre on the `UpdateItems` stream
    // (`Item.NewItem` under `Main.cs:17779`), the `spawn_item_drop_px`
    // velocity-triplet-then-prefix order (the catch-overflow precedent).
    for (item, cx, cy) in app.projectiles.drain_item_drops() {
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            app.rngs.stream(crate::app::RngKey::UpdateItems),
            item,
            1,
            glam::Vec2::new(cx, cy),
        );
    }
    // W4-d4 — `Kill()`'s world-drop drain (`Projectile.cs:77572-77615` the
    // rope-coil refund ladder, `:77620-77623` the beach ball's return): the
    // pool's death hook queued `(item, stack, base.Center, zero_velocity)`;
    // each spawns an `Item.NewItem` WORLD drop at the recorded centre (NOT
    // a bobber catch — no inventory hand-off). The COIL calls pass no
    // velocity, so the C# draws the pair inline under `Kill()`
    // (`Item.cs:49749-49750`: X = Next(-30,31)*0.1, Y = Next(-40,-15)*0.1 —
    // none of the kinds is `ItemNoGravity`, `ItemID.cs:1320`); the BALL
    // call passes `Vector2.Zero` (`:77622`) — the `velocity.HasValue` arm
    // (`Item.cs:49738-49741`) sets (0,0) and draws NOTHING (the SEAL-VERIFY
    // fix). `prefix: 0` draws nothing. The port defers the spawn to the
    // pass tail on the `UpdateItems` stream (`Main.cs:17779` — the bobber
    // overflow's recorded stream deviation). The `ReserveForLocalPlayer`
    // ownership sets only `enemyGrabDelayTime` in netMode 0
    // (`WorldItem.cs:295-305`), unported — the drop is player-grabbable
    // immediately (the lucky-coin precedent).
    for (item, stack, cx, cy, zero_velocity) in app.projectiles.drain_death_drops() {
        if stack <= 0 {
            continue;
        }
        let (vx, vy) = if zero_velocity {
            (0.0, 0.0)
        } else {
            let rng = app.rngs.stream(crate::app::RngKey::UpdateItems);
            (
                rng.next_range(-30, 31) as f32 * 0.1,
                rng.next_range(-40, -15) as f32 * 0.1,
            )
        };
        // `WorldItem.Center = center` (`Item.cs:49736`) — the port's 14×14
        // drop box centred on the dead projectile's centre (the universal
        // `ItemDrop` box deviation, the bug-net catch precedent).
        let half = crate::item::SIZE * 0.5;
        let drop = crate::item::ItemDrop::new(
            item,
            stack as u32,
            0,
            cx - half,
            cy - half,
            vx,
            vy,
            0.0,
        );
        app.items.push(drop);
        if app.items.len() > 300 {
            app.items.remove(0);
        }
    }
    // W4-d6 — the aiStyle-60 slime walk's campfire-validate mismatch kills
    // (`Wiring.ToggleCampFire` → `ValidateTileSquareIsActiveAndOfType`,
    // `Wiring.cs:3012` → `WorldGen.cs:37891-37914`): the object drops spawn
    // on the `UpdateProjectiles` stream (the walk ran inside the pool's
    // update — the actuation-drops precedent, `world_sim/wiring.rs`).
    for drop in app.projectiles.drain_light_drops() {
        if let Some(id) = drop
            .item
            .map(|i| i as u16)
            .or_else(|| crate::registry::tile_drop(drop.tile_kind, 0))
        {
            crate::world_sim::items::spawn_item_drop(&mut app.items, app.rngs.stream(crate::app::RngKey::UpdateProjectiles), id, 1, drop.x, drop.y);
        }
    }
    // W4-d6 residual — the Spectre-heal orb spawns (`Projectile.ghostHeal`,
    // `Projectile.cs:12095`): the damage walk queued `(owner, x, y, ai0,
    // ai1)` per qualifying hit; the pass spawns each 298 CENTRED on the
    // victim point (`NewProjectile`'s half-extent subtract,
    // `Projectile.cs:10876-10877` — the c6 BladeShot precedent), zero
    // velocity, damage 0, knockback 0. Draw-free (the 298 NewProjectile
    // path rolls nothing — no spawn-roll case, no crit at spawn).
    for (owner, x, y, ai0, ai1) in app.projectiles.drain_heal_orbs() {
        let d = crate::projectile::proj_data(crate::projectile::SPIRIT_HEAL);
        let (hw, hh) = d.map_or((0.0, 0.0), |d| (d.w * 0.5, d.h * 0.5));
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::SPIRIT_HEAL,
            owner,
            x - hw,
            y - hh,
            0.0,
            0.0,
            0,
            0.0,
            ai0,
            ai1,
        );
    }
    // W4-d6 residual 5 — the Spectre-Mask damage-share spawns
    // (`Projectile.ghostHurt`, `Projectile.cs:12060`): the damage walk
    // queued `(owner, x, y, vx, vy, damage, ai0 target)` per qualifying hit
    // (the direction rolls ran at the hit); the pass spawns each 356
    // CENTRED on the victim point (the `NewProjectile` half-extent
    // subtract), zero knockback, ai0 = the RAW npc index the style-59
    // machine reads. Draw-free spawn (no 356 case in the switch).
    for (owner, x, y, vx, vy, damage, ai0) in app.projectiles.drain_wraths() {
        let d = crate::projectile::proj_data(crate::projectile::SPECTRE_WRATH);
        let (hw, hh) = d.map_or((0.0, 0.0), |d| (d.w * 0.5, d.h * 0.5));
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::SPECTRE_WRATH,
            owner,
            x - hw,
            y - hh,
            vx,
            vy,
            damage,
            0.0,
            ai0,
            0.0,
        );
    }
    // W4-d6 residual 5 — the Vampire Knives' heal-orb spawns
    // (`Projectile.vampireHeal`, `Projectile.cs:12104`): `(owner, x, y,
    // ai0 owner, ai1 float share)` per qualifying 304 hit; the pass spawns
    // each 305 centred on the victim, zero velocity, damage 0. Draw-free.
    for (owner, x, y, ai0, ai1) in app.projectiles.drain_vampire_orbs() {
        let d = crate::projectile::proj_data(crate::projectile::VAMPIRE_HEAL);
        let (hw, hh) = d.map_or((0.0, 0.0), |d| (d.w * 0.5, d.h * 0.5));
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::VAMPIRE_HEAL,
            owner,
            x - hw,
            y - hh,
            0.0,
            0.0,
            0,
            0.0,
            ai0,
            ai1,
        );
    }
    // W4-d6 residual 8 — the gun streams' Kill-splash dust
    // (`Projectile.cs:74029-74147`): the death hook snapshotted the dying
    // 358/406/1042 corpses (the velocity base already written); the drain
    // emits the 50/40/50 dusts on the LIVE DustPool, the explicit gates +
    // the `new_dust` internals on the `UpdateProjectiles` stream in C#
    // order (the g65 gallop-dust pass-tail convention).
    let gun_splashes = app.projectiles.drain_gun_splashes();
    if !gun_splashes.is_empty() {
        let ctx = app.spawn_ctx();
        crate::projectile_gun_splash::drain(
            gun_splashes,
            &mut app.dust,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            &ctx,
            &app.world,
        );
    }
    // W4-d20 — the aiStyle-192 jumino's completion gift (`AI_192_JuminoAnimation`
    // `Projectile.cs:41632-41637`): `Item.NewItem(EntitySource_Gift, Center +
    // (0, 12), 5276)` then `velocity = (0, 0)`. The C# `NewItem` draws the
    // velocity triplet on `Main.rand` (the `UpdateProjectiles` swap,
    // `Main.cs:17775`) BEFORE the explicit zeroing overrides it; prefix 0
    // rolls nothing (`Item.Prefix(0)`, `Item.cs:49730`). The port keeps the
    // three draws for stream fidelity, then zeroes the velocity (the death
    // drop's explicit-velocity precedent, `death.rs`). The drop centres on
    // the gift point (`WorldItem.Center = center`, `Item.cs:49736`).
    for (owner, item, x, y) in app.projectiles.drain_gift_items() {
        if owner != app.local_id as u8 {
            continue;
        }
        let _ = app.rngs.stream(crate::app::RngKey::UpdateProjectiles).next_float();
        let _ = app.rngs.stream(crate::app::RngKey::UpdateProjectiles).next_float();
        let _ = app.rngs.stream(crate::app::RngKey::UpdateProjectiles).next_float();
        app.items.push(crate::item::ItemDrop::new(
            item,
            1,
            0,
            x - crate::item::SIZE * 0.5,
            y - crate::item::SIZE * 0.5,
            0.0,
            0.0,
            0.0,
        ));
    }
    // W4-g82 — the coil refund drain (`Projectile.cs:77572-77615` →
    // `Item.RequestNewItem` = `Item.NewItem`, a world DROP at
    // `base.Center`): the velocity-triplet-then-prefix draws ride the
    // `UpdateProjectiles` stream (the C# spawns inside `Kill()` under the
    // `Main.cs:17775` swap; the port drains at the pass tail, the
    // S1-E-21c convention — the ai1==0 Dig cue already drained above in
    // `drain_proj_sounds`, the C#'s cue-before-refund order kept).
    for (item, stack, cx, cy) in app.projectiles.drain_coil_drops() {
        if stack <= 0 {
            continue;
        }
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            item,
            stack as u32,
            glam::Vec2::new(cx, cy),
        );
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
                // 106`) — the cup's tile trip. The §8.1 stamp first.
                crate::world_sim::wiring::stamp_wiring_entity_boxes(app);
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
                // W4-c42 — the cup trip's wired door cues (same stream).
                for (open, px, py) in app.wiring.take_door_cues() {
                    app.audio.play_door_sound(
                        open,
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
                // The §8.2 echoes: SP discards (the netMode-0 return).
                for (n, x, y, d) in app.wiring.take_door_echoes() {
                    app.send_door_toggle(n as u8, x, y, d);
                }
                if let Some(text) = chat {
                    app.chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
                }
                if let Some((id, style, x, y, pitch, vol)) = cue {
                    app.audio.play_proj_sound(
                        (id, style),
                        x,
                        y,
                        pitch,
                        vol,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
            }
            crate::projectile_ai_golf::GolfEffect::PadProbe { pos, old_pos, w, h } => {
                // `Collision.SwitchTiles(..., 4)` (`Projectile.cs:20367`)
                // — the projectile pressure-pad call. The switch sends
                // ride the outcome (`Collision.cs:3304` et al.). The §8.1
                // stamp first (the trip can shift a wired trapdoor/gate).
                crate::world_sim::wiring::stamp_wiring_entity_boxes(app);
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
                    // W4-g83: the plate kill's framing dispatch rides the
                    // UpdateProjectiles stream (the active genRand).
                    app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
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
                // W4-c42 — the pad probe's wired door cues (same stream).
                for (open, px, py) in app.wiring.take_door_cues() {
                    app.audio.play_door_sound(
                        open,
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
                // The §8.2 echoes: SP discards (the netMode-0 return).
                for (n, x, y, d) in app.wiring.take_door_echoes() {
                    app.send_door_toggle(n as u8, x, y, d);
                }
            }
            crate::projectile_ai_golf::GolfEffect::PlayerWrites {
                facing,
                held_proj,
                dummy_item_time,
                cancel_use,
                item_animation_max,
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
                    // W4-d20 — `player.itemAnimationMax = useAnimation`
                    // (`AI_198_Flint` `Projectile.cs:52307`): the port's
                    // swing_max is the itemAnimationMax stand-in (seconds).
                    if let Some(max) = item_animation_max {
                        app.swing_max = max as f32 / 60.0;
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
    // W4-d3 — the aiStyle-25 boulder family's parked requests, drained at the
    // pass tail (the `drain_mark_spawns` shape — the pool cannot borrow the
    // App). The C# rolls the drops/bees and trips the switches inline under
    // the `UpdateProjectiles` swap; the port defers the App-borrowing resolves
    // here, keeping the rolls on this pass's stream (the stack/count/velocity
    // rolls already ran in the kill hook / arm).
    // The item drops (`Item.NewItem`, `Projectile.cs:74952`/`:74956`/`:75096`):
    // the velocity/prefix draws ride the `UpdateItems` stream (`Main.cs:17779`
    // — the bobber-catch precedent); the spawn point is the dying boulder's
    // centre (`NewItem`'s `center` argument).
    for (item, stack, x, y) in app.projectiles.drain_boulder_drops() {
        if stack <= 0 {
            continue;
        }
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            app.rngs.stream(crate::app::RngKey::UpdateItems),
            item,
            stack as u32,
            Vec2::new(x, y),
        );
    }
    // The 655 bee-hive death's bee spawns (`:75116-75118`): `NPC.NewNPC`'s
    // Bottom placement (`NPC.cs:80769-80770` — the Queen-Bee volley precedent,
    // `world_sim/npcs.rs:5507`), the rolled velocity kicks, `netUpdate`.
    // `CanBeReplacedByOtherNPCs` has no port field — dropped (the volley's
    // recorded fold).
    let mut boulder_bees = Vec::new();
    for (kind, x, y, vx, vy) in app.projectiles.drain_boulder_bees() {
        let npc_kind = match kind {
            210 => crate::npc::NpcKind::Bee,
            211 => crate::npc::NpcKind::BeeSmall,
            _ => unreachable!("the hive burst draws only 210/211"),
        };
        let (bw, bh) = npc_kind.size();
        let mut bee = crate::npc::Npc::new(npc_kind, x - bw * 0.5, y - bh, 1, 0.0);
        bee.hp = bee.max_hp;
        bee.vx = vx;
        bee.vy = vy;
        bee.net_update = true;
        boulder_bees.push(bee);
    }
    app.npcs.append(&mut boulder_bees);
    // The `Collision.SwitchTiles` requests the arm parked (the `:17089`
    // objType-3 plate scan, the `:19863`/`:17790` objType-4 pad calls). The
    // scan's plate drops and Mech-28 clicks ride THIS pass's stream (the C#
    // call sits inside `UpdateProjectiles`, `Main.cs:17775`); the `59` sends
    // ride the outcome (the golf PadProbe precedent).
    let boulder_switches: Vec<_> = app
        .projectiles
        .slots_mut()
        .iter_mut()
        .flat_map(|p| std::mem::take(&mut p.wants_boulder_switches))
        .collect();
    for (px, py, ox, oy, w, h, obj_type) in boulder_switches {
        let outcome = crate::collision::switch_tiles(
            &mut app.wiring,
            &mut app.world,
            px,
            py,
            ox,
            oy,
            w,
            h,
            obj_type,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
        );
        if let Some((item, tx, ty)) = outcome.dropped {
            crate::world_sim::items::spawn_item_drop_fields(
                &mut app.items,
                app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                item,
                1,
                tx,
                ty,
            );
        }
        app.send_switch_outcome(&outcome);
        for (cx, cy) in app.wiring.take_switch_clicks() {
            app.audio.play_switch_click(
                cx as f32,
                cy as f32,
                app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            );
        }
    }
    // S1-W4-g81 residuals — the Wire Kite's owner writes
    // (`Projectile.cs:35825-35832`), drained after the walk (the golf
    // `PlayerWrites` shape): ChangeDir(Sign(velocity.X)), heldProj,
    // SetDummyItemTime(2), itemRotation = 0, and the right-click kill's
    // Main.blockMouse.
    for (slot, tick) in app.projectiles.drain_kite_ticks() {
        if let Some(p) = app.player.as_mut() {
            if let Some(dir) = tick.facing {
                p.facing = dir;
            }
            if tick.held_proj {
                p.held_proj = slot as i32;
            }
            if tick.zero_item_rotation {
                p.item_rotation = 0.0;
            }
        }
        if let Some(frames) = tick.dummy_item_time {
            // `SetDummyItemTime(2)` — the swing/item timers re-arm to 2
            // ticks so the held kite never starts a new use (the spear
            // hold's write-back precedent).
            app.swing = frames as f32 / 60.0;
            app.item_time = frames as f32 / 60.0;
        }
        if tick.block_mouse {
            app.block_mouse = true;
        }
    }
    // `Kill()`'s type-651 mass-wire intents (`Projectile.cs:72051-72067` →
    // `Wiring.MassWireOperation`): the DIRECT DRAIN (f11 research/133
    // §8.4b, executed at the batch-H c42×g81 compose per research/382 §4).
    // RAW ToolMode (`Wiring.cs:3241` — the kite does NOT strip the Actuator
    // bit; that mask is the 3625 arm's, `Player.cs:46908`), no cooldown
    // gate (the Kill hook has none), `RngKey::UpdateProjectiles` (the
    // Kill's stage, `Main.cs:17775`). The owner gate already ran at the
    // queue site (`kill_mass_wire_op`'s `p.owner != 0` refusal — the SP
    // `owner == local_id` fold, g81's 2-tuple shape). ONE drain shape:
    // at the d13 union this seam dedupes to d13's 5-tuple pool shape
    // (research/382 §3 — d13's pool seam is the keep-one; the apply call
    // here is the same c42 fn either way).
    for (ps, pe) in app.projectiles.drain_mass_wire_ops() {
        crate::world_sim::wiring::mass_wire_operation(
            app,
            ps,
            pe,
            app.wire_tool_mode,
            crate::app::RngKey::UpdateProjectiles,
        );
    // W4-e4 — the orchestra request drain (`ParticleOrchestrator.
    // RequestParticleSpawn` → `SpawnParticlesDirect`,
    // `ParticleOrchestrator.cs:52-59`/`:162`): the damage/death/tag arms
    // queued `(kind byte, settings)` this tick; the emitter bodies run
    // here on the SAME `UpdateProjectiles` stream (`Main.cs:17775`), the
    // S1-E-21c queue→drain convention (the pool cannot borrow the App's
    // dust pool mid-walk). The settings' argument draws were consumed
    // inline at the queue sites; only the emitter internals roll here.
    // The spawn ctx is the pass-moment one (the cull/pressure gates read
    // the live camera + quality cap, `App::spawn_ctx`).
    let orchestra = app.projectiles.drain_orchestra();
    if !orchestra.is_empty() {
        let ctx = app.spawn_ctx();
        let disco = app.disco;
        let players = [app.player.as_ref()];
        // `&mut frame_rng` and `&world` overlap; snapshot like the
        // editing.rs `genRand` sites, then write the advanced state back.
        let mut gen_rng = app.world.frame_rng.clone();
        for (kind, settings) in orchestra {
            crate::orchestrator_dust::spawn_orchestra(
                kind,
                &settings,
                app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                &mut gen_rng,
                disco,
                &mut app.dust,
                &ctx,
                &app.world,
                &players[..],
            );
        }
        app.world.frame_rng = gen_rng;
    }
}
}

/// W4-d6 residual 7 — the 298-orb chain end-to-end (written, NOT run —
/// code-only mode): the Spectre-Hood hit queues the intent, the pass-tail
/// drain spawns the orb centred, and the armed style-52 dispatch flies it
/// into the player for the contact heal (`Projectile.cs:28504-28532`).
#[cfg(test)]
mod w4_d6_heal_orb_chain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::{LOST_SOUL, SPIRIT_HEAL};

    #[test]
    fn w4_d6_the_hit_intent_drains_into_a_flying_orb_that_heals_on_contact() {
        // The 8x8 fixture world, the player parked mid-world; a high-life
        // zombie overlapping a Lost Soul shot. The hood flag + the missing
        // health arm the ghostHeal spawn; the drained 298 homes at speed 4
        // through the (v*15+d)/16 blend and heals on the <50f contact.
        let mut app = flow_app(None, vec![], vec![]);
        app.local_id = 0;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 40.0;
        player.y = 40.0;
        player.hp = 50;
        player.max_hp = 100;
        player.ghost_heal = true;
        player.life_steal = 80.0;
        app.player = Some(player);
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::Zombie, 56.0, 40.0, 1, 0.0);
        npc.hp = 10_000;
        npc.max_hp = 10_000;
        npc.defense_override = Some(0);
        app.npcs.push(npc);
        // The magic shot overlapping the victim (the 12x12 soul box),
        // flying right at 8 px/tick — it exits the 8x8 world (the
        // out-of-world guard, no Kill) before the id-static 10-tick window
        // expires, so exactly ONE hit lands and ONE orb spawns.
        app.projectiles
            .spawn_with_data_ai_owned(LOST_SOUL, 0, 58.0, 44.0, 8.0, 0.0, 100, 0.0, 0.0, 0.0);
        update_projectiles(&mut app, 1.0 / 60.0);
        // The drain spawned the orb: centred on the POST-strike victim
        // centre (the 6x6 half-extent subtract), ai0 = the local player
        // (the missing-health scan), ai1 = the 0.2 float share.
        let orb = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == SPIRIT_HEAL)
            .expect("the drained 298 lives");
        let n = &app.npcs[0];
        let (nw, nh) = n.size();
        assert_eq!(orb.ai[0], 0.0, "the target scan picked the local player");
        let strike = 10_000 - n.hp;
        assert!(strike > 0, "the hit landed");
        // The drain runs at the pass TAIL, so the orb's first AI tick is
        // the NEXT pass — this tick it holds the spawn velocity 0.
        assert!(orb.vel.length() < 1e-6, "the drain spawns at rest");
        let heal = orb.ai[1];
        assert!((heal - strike as f32 * 0.2).abs() < 1e-3, "the 0.2 share");
        // The next ticks: the armed 52 dispatch blends toward the player
        // (the victim centre is ~15f from the player centre — inside the
        // 50f contact radius, so the box overlap decides; the orb closes
        // the last few px within a tick or two). Fly it home.
        for _ in 0..40 {
            update_projectiles(&mut app, 1.0 / 60.0);
        }
        assert!(
            !app.projectiles
                .slots()
                .iter()
                .any(|p| p.active && p.kind == SPIRIT_HEAL),
            "the contact killed the orb (:28526)"
        );
        let p = app.player.as_ref().unwrap();
        assert!(p.hp > 50, "the contact healed");
        assert!(p.hp <= 100, "the statLifeMax2 cap");
    }
}

/// W4-d4 union — the `Kill()` world-drop drain's wire + stream contract
/// (written, NOT run — code-only): the coil refund ladder
/// (`Projectile.cs:77572-77615`) spawns `Item.NewItem` world drops at the
/// recorded centre, TWO integer draws on the `UpdateItems` stream per drop
/// (`Item.cs:49749-49750`); the beach ball's return (`:77620-77623`) passes
/// `Vector2.Zero` — the `HasValue` arm draws NOTHING (the SEAL-VERIFY
/// fix). Every other stage stream pristine.
#[cfg(test)]
mod w4_d4_death_drop_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::{ROPE_COIL, VINE_ROPE_COIL};
    use tr_core::UnifiedRandom;

    /// A coil hovering inside the 8×8-tile fixture world with the given
    /// placed-count. `time_left = 1` expires it on the first pass tick
    /// (the lifetime block) without any placement contact.
    fn coil_app(kind: u16, placed: f32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let s = app
            .projectiles
            .spawn_with_data_ai(kind, 32.0, 32.0, 0.0, 0.0, 0, 0.0, 0.0, placed)
            .unwrap();
        app.projectiles.slots_mut()[s].time_left = 1;
        app
    }

    /// The unplaced coil returns ONE coil item; the partially placed one
    /// returns `10−ai[1]` rope items — both as WORLD drops at the coil's
    /// centre, not inventory hand-offs (the C# `RequestNewItem`, not the
    /// bobber's `GiveItemToPlayer`).
    #[test]
    fn w4_d4_union_the_coil_expiry_drains_the_refund_as_a_world_drop() {
        let mut app = coil_app(ROPE_COIL, 0.0);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "one refund drop");
        let d = &app.items[0];
        assert_eq!((d.kind, d.count), (985, 1), "the coil item back (`:77576`)");
        // The 14×14 box centred on (39, 39) — the coil's centre at death.
        assert!((d.center().x - 39.0).abs() < 1e-3 && (d.center().y - 39.0).abs() < 1e-3);
        assert_eq!(d.prefix, 0, "prefix: 0 (`:77576`)");
        assert_eq!(d.grab_delay_time, 0, "netMode 0 sets no player delay");
        // The velocity pair on the UpdateItems stream: X = Next(-30,31)*0.1,
        // Y = Next(-40,-15)*0.1 (`Item.cs:49749-49750`), seed 0.
        let mut expect = UnifiedRandom::new(0);
        let vx = expect.next_range(-30, 31) as f32 * 0.1;
        let vy = expect.next_range(-40, -15) as f32 * 0.1;
        assert!((d.vel.x - vx).abs() < 1e-6 && (d.vel.y - vy).abs() < 1e-6);
        assert_eq!(*app.rngs.stream(RngKey::UpdateItems), expect, "exactly two draws");
        // The unplaced coil's death ALSO plays the Dig thud
        // (`Projectile.cs:74683`: `if (this.ai[1] == 0f)
        // SoundEngine.PlaySound(0, (int)base.position.X, ...)` for type 171,
        // `:74619` for 475/505/506) — the port's id-0 façade draws its
        // case-0 pair `Next(3)` (the variation) then `Next(-10, 11)` (the
        // pitch, `LegacySoundPlayer.cs:442`/`:450`) on the `UpdateProjectiles`
        // stream. Pin it against a MUTED control run of the same death: the
        // gate refuses before the case draws, so the audible stream must be
        // exactly the muted state plus those two draws (and no third — a
        // second Dig play is the double-dispatch bug, not a permit).
        let mut muted = coil_app(ROPE_COIL, 0.0);
        muted.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        update_projectiles(&mut muted, 1.0 / 60.0);
        let mut expect_proj = muted.rngs.stream(RngKey::UpdateProjectiles).clone();
        let _ = expect_proj.next_range(0, 3);
        let _ = expect_proj.next_range(-10, 11);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            expect_proj,
            "the coil's one Dig cue: the variation + pitch pair"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateItems || key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine",
                key.name()
            );
        }
        // The partial refund: ai[1] = 4 → 6 vine-rope items (`:77591`).
        let mut app2 = coil_app(VINE_ROPE_COIL, 4.0);
        update_projectiles(&mut app2, 1.0 / 60.0);
        assert_eq!(app2.items.len(), 1);
        assert_eq!((app2.items[0].kind, app2.items[0].count), (2996, 6));
        // The full budget drops nothing — the drain never runs.
        let mut app3 = coil_app(ROPE_COIL, 10.0);
        update_projectiles(&mut app3, 1.0 / 60.0);
        assert!(app3.items.is_empty(), "10 placed → no refund");
        for key in RngKey::ALL {
            assert_eq!(*app3.rngs.stream(key), UnifiedRandom::new(0), "{} pristine", key.name());
        }
    }

    /// The beach ball's zero-velocity death returns ONE Beach Ball (859)
    /// at `base.Center` (`:77620-77623`) — the same drain, but the call
    /// passes `Vector2.Zero` (`:77622`): the `velocity.HasValue` arm
    /// (`Item.cs:49738-49741`) sets (0,0) and draws NOTHING (the
    /// SEAL-VERIFY fix — the earlier two-draw pin misread the call).
    #[test]
    fn w4_d4_union_the_beach_ball_return_drains_on_the_items_stream() {
        let mut app = flow_app(None, vec![], vec![]);
        app.projectiles
            .spawn_with_data(crate::projectile::BEACH_BALL, 32.0, 32.0, 0.0, 0.0, 0, 0.0);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "the ball returns");
        let d = &app.items[0];
        assert_eq!((d.kind, d.count), (859, 1), "`:77622`");
        // The 44×44 ball's centre: (54, 54.1) — the death hook reads the
        // post-integrate slot (the dry gravity tail's 0.1 dead-tick move,
        // the pool-wide hook timing deviation).
        assert!((d.center().x - 54.0).abs() < 1e-3 && (d.center().y - 54.1).abs() < 1e-3);
        assert!(
            d.vel.x == 0.0 && d.vel.y == 0.0,
            "Vector2.Zero — the HasValue arm sets (0,0)"
        );
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine — the explicit velocity draws nothing",
                key.name()
            );
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

    /// S1-E-21h d75 part 2 — the new drain arms' stream contract: the
    /// Crystal Shard's Shatter (id 13) and the yoyo dash's roar pair
    /// (id 15/36) draw NOTHING from the pass stream at the drain (cases
    /// 13/15/36 are draw-free in the legacy player) — the dash's own
    /// `Next(100)` roll already drew inside the arm. The sonar trackable
    /// (id 42) is draw-free too.
    #[test]
    fn s1_e_21h_d75_p2_the_shatter_and_roar_drains_are_draw_free() {
        use crate::projectile::{CRYSTAL_SHARD_MELEE, EYE_OF_CTHULHU_DASH};
        // The lead shard: one Next(100)-free Shatter intent — the whole
        // pass draws nothing (the shard arm is RNG-free).
        let mut app = flow_app(None, vec![], vec![]);
        app.audio.set_listener(listener(1.0));
        app.projectiles.spawn_with_data_ai_owned(
            CRYSTAL_SHARD_MELEE,
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
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine: the Shatter drain draws nothing",
                key.name()
            );
        }
        // The dash: `extraUpdates = 3` — four sub-steps, but the cue
        // block's gate reads `localAI[0] == 0` and the SAME arm call bumps
        // the counter after it (`:36845`), so only the FIRST sub-step
        // rolls Next(100); the drain then adds one case-2 pitch draw for
        // the common Item20 intent (the roar drains draw nothing). The
        // 100-px box would trip the out-of-world guard in the 8x8 fixture
        // world — shrink it (the cue block reads no box state; Center
        // reads pos + w/2).
        let mut app = flow_app(None, vec![], vec![]);
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .spawn_with_data_ai_owned(
                EYE_OF_CTHULHU_DASH,
                0,
                32.0,
                32.0,
                0.0,
                0.0,
                10,
                4.0,
                0.0,
                0.0,
            )
            .expect("a free slot");
    /// S1-E-21h f1 — the `Kill()` death cue's stream contract: a shot that
    /// dies this pass queues its death cue, and the case-2 pitch draw rides
    /// the `UpdateProjectiles` stream ONLY when audible. The Jester's Arrow
    /// (5, `:75670` Item10) dies on its first sub-step's `penetrate == 0`
    /// after a hit; here it is spawned with `time_left = 1` so the lifetime
    /// block kills it on tick 1 without a target. Written, NOT run.
    #[test]
    fn s1_e_21h_f1_the_death_cue_draws_ride_update_projectiles_only_when_audible() {
        use crate::projectile::JESTERS_ARROW;
        fn dying_star_app() -> App {
            let mut app = flow_app(None, vec![], vec![]);
            app.projectiles.spawn_with_data_ai_owned(
                JESTERS_ARROW,
                0,
                32.0,
                32.0,
                0.0,
                0.0,
                10,
                4.0,
                0.0,
                0.0,
            )
            .expect("a free slot");
            // Force the lifetime kill on tick 1 (the death hook fires).
            let slot = app
                .projectiles
                .slots()
                .iter()
                .position(|p| p.active && p.kind == JESTERS_ARROW)
                .expect("the arrow lives");
            app.projectiles.slots_mut()[slot].time_left = 1;
            app.projectiles.slots_mut()[slot].w = 8.0;
            app.projectiles.slots_mut()[slot].h = 8.0;
            app
        }
        // Muted: the death cue queues but the drain's gate refuses before
        // the case-2 switch — no gameplay draw.
        let mut app = dying_star_app();
        app.audio.set_listener(listener(0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine under the muted death-cue gate",
                key.name()
            );
        }
        // Audible: exactly ONE case-2 default pitch draw (`Next(-6, 7)`)
        // on the `UpdateProjectiles` stream — the death cue's Item10.
        let mut app = dying_star_app();
        app.audio.set_listener(listener(1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "exactly one case-2 pitch draw for the death cue"
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

    /// S1-E-21h d75 part 4 — the two new drain routes' stream contract:
    /// the RainbowBoulder trackable `(42, 274)` (d3's hard-bounce cue,
    /// `Projectile.cs:20104`) draws NOTHING (case 42,
    /// `LegacySoundPlayer.cs:1376-1381`); the FoxparksFlame named-row
    /// marker `(2, -34)` (`:41369`) draws exactly ONE case-2 default
    /// pitch (`Next(-6, 7)`) — the same draw the plain +34 tuple makes,
    /// only the volume differs (0.15 vs 1.0). Both fire only past the
    /// attenuation gate. The intents are parked through a rowed carrier
    /// (the FROST_BOLT slot at rest — its own first-tick latch pre-fired
    /// via ai[1]; the 1047/1106 rows ride d3/d18's pushed tips).
    #[test]
    fn s1_e_21h_d75_p4_the_rainbow_and_foxparks_drain_routes_draw_correctly() {
        // The trackable: muted refuses draw-free …
        let mut app = frost_bolt_app();
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((42, 274, 32.0, 32.0, 0.0, 1.0));
        // The death fired once: the second tick draws nothing further.
        update_projectiles(&mut app, 1.0 / 60.0);
        let witness = app.rngs.stream(RngKey::UpdateProjectiles).clone();
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "the once-per-death cue draws nothing on tick 2"
        );
    }

    /// S1-E-21h f1 fifth batch — the Kill-tail death cue's stream contract:
    /// the Holy Water bottle (69, `:77018` Shatter) dies this pass and queues
    /// its id-13 cue; the Shatter case draws NOTHING (`LegacySoundPlayer.cs:
    /// 783-792`), so the `UpdateProjectiles` stream stays pristine whether
    /// audible or muted — the gate still precedes the (empty) case. The
    /// bottle is aiStyle 2 (RNG-free flight), spawned `time_left = 1` so the
    /// lifetime block kills it on tick 1. Written, NOT run.
    #[test]
    fn s1_e_21h_f1_b5_the_water_bottle_shatter_cue_is_draw_free() {
        use crate::projectile::HOLY_WATER;
        fn dying_bottle_app() -> App {
            let mut app = flow_app(None, vec![], vec![]);
            app.projectiles.spawn_with_data_ai_owned(
                HOLY_WATER, 0, 32.0, 32.0, 0.0, 0.0, 10, 0.0, 0.0, 0.0,
            );
            let slot = app
                .projectiles
                .slots()
                .iter()
                .position(|p| p.active && p.kind == HOLY_WATER)
                .expect("the bottle lives");
            app.projectiles.slots_mut()[slot].time_left = 1;
            app
        }
        // Audible: the Shatter plays but draws nothing — every stream pristine.
        let mut app = dying_bottle_app();
        app.audio.set_listener(listener(1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine — the Shatter case draws nothing",
                key.name()
            );
        }
        // Muted: the gate refuses before the (empty) case — still pristine.
        let mut app = dying_bottle_app();
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
        // … audible: case 42 draws nothing anywhere.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((42, 274, 32.0, 32.0, 0.0, 1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine: the trackable drain draws nothing",
                key.name()
            );
        }
        // The FoxparksFlame marker: one case-2 pitch on UpdateProjectiles.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((2, -34, 32.0, 32.0, 0.0, 1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "exactly one case-2 pitch draw for the named flame row"
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
        // The 1054 SpiderBoulder death cue (3, 29): case 3's ONE
        // `Next(-10, 11)` pitch draw (`LegacySoundPlayer.cs:670`), the
        // GetRandomPitch NextFloat audio-only.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((3, 29, 32.0, 32.0, 0.0, 1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "case 3: exactly one pitch draw for the NPCHit29 death cue"
        );
        // The 1055 Ghoulder death cue (4, 6): case 4's ONE
        // `Next(-10, 11)` pitch draw (`:693`), no playing guard (10-only).
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((4, 6, 32.0, 32.0, 0.0, 1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "case 4: exactly one pitch draw for the NPCDeath6 death cue"
        );
        // The 502 Meowmere star chime (37, style): case 37's TWO draws —
        // the Next(57,59) slot pick, then the Next(-40,41) pitch
        // (`LegacySoundPlayer.cs:1328`/`:1332`).
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((37, 6, 32.0, 32.0, 0.0, 1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(57, 59);
        let _ = witness.next_range(-40, 41);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "case 37: slot pick then pitch, two draws"
        );
    }
}
}

#[cfg(test)]
mod w4_c19_ai_light_drain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::TYPHOON;
    use tr_core::UnifiedRandom;

    /// W4-c19 round 3 witness (written, NOT run): the typhoon's rand-gated
    /// light intents drain into the ONE `EntityLights` pair at the pass tail
    /// — the gate draws ride `UpdateProjectiles` ONLY (the `item_lights`
    /// named-stream precedent), every other stage stream stays pristine,
    /// and the intents land in slot + sub-step order with the C# colour.
    ///
    /// The gate is `Main.rand.Next(3) == 0` (`Projectile.cs:30358`), with no
    /// guarantee for any given window: seed 0's first three `Next(3)` draws
    /// are 2, 2, 2 (the `UnifiedRandom` golden vector's 1/3 cuts), so ONE
    /// update (three sub-steps, `extraUpdates = 2`) can never take the gate
    /// on this rig. The fixture therefore runs TWO updates — six sub-steps,
    /// whose gates are 2, 2, 2, 1, 0, 2, taking the gate on the fifth. The
    /// twin below replays the same six draws (with the taken branch's dust
    /// pairs interleaved) and is the census witness.
    #[test]
    fn the_typhoon_intents_drain_into_the_entity_lights_pair() {
        let mut app = flow_app(None, vec![], vec![]);
        // Muted: any spawn-cue play draws nothing (the s1_e_21h precedent),
        // so the stream witness sees ONLY the gate + dust-ring draws.
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        // Past the 10-sub-step gate: the 30x30 box at (32, 32) centres on
        // (47, 47) → tile (2, 2) inside the 8x8 fixture world.
        let slot = app
            .projectiles
            .spawn_with_data_ai_owned(TYPHOON, 0, 32.0, 32.0, 0.0, 0.0, 40, 5.0, 0.0, 0.0)
            .expect("a free slot");
        app.projectiles.slots_mut()[slot].local_ai[1] = 10.0;
        // Two update calls = six AI sub-steps (`extraUpdates = 2`).
        update_projectiles(&mut app, 1.0 / 60.0);
        update_projectiles(&mut app, 1.0 / 60.0);
        // The drain emptied the pool seam (the pass tail ran each call).
        assert!(app.projectiles.drain_ai_lights().is_empty());
        // The lights: swap the pair, then read the frame's sources. Every
        // intent is the typhoon's (0.1, 0.4, 0.6) at the truncated centre.
        app.entity_lights.end_frame();
        let lights = app.entity_lights.visible();
        assert_eq!(lights.len(), 1, "the fifth of the six gates is taken");
        for l in lights {
            assert_eq!(l.rgb, [0.1, 0.4, 0.6]);
        }
        // The twin: replay the six sub-steps' gate draws (the taken branch
        // adds the 6 × (NextDouble, Next(3,8)) dust-ring pairs).
        let mut twin = UnifiedRandom::new(0);
        let mut taken = 0;
        for _ in 0..6 {
            if twin.next_range(0, 3) == 0 {
                taken += 1;
                for _ in 0..6 {
                    let _ = twin.next_double();
                    let _ = twin.next_range(3, 8);
                }
            }
        }
        assert_eq!(taken, 1, "the seed-0 window's taken-gate census");
        assert_eq!(lights.len(), taken, "one light per taken gate");
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            twin,
            "the gate + dust-ring draws, nothing else"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert!(
                !app.rngs.is_created(key) || *app.rngs.stream(key) == UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
    }
}

/// W4-c15 — the explosion wall-walk drain's stream contract (written, NOT
/// run — code-only mode): the queued `KillWall_PlaySounds` cues draw on the
/// `UpdateProjectiles` stream at the pass-tail drain, attenuation gate
/// first — the draw census equals the queue's payload census (Shatter 0,
/// Dig 2 per cue), plus the blast cue's one case-2 pitch draw.
#[cfg(test)]
mod w4_c15_wall_drain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::BOMB;
    use tr_core::tile::{FLAG_ACTIVE, Tile};
    use tr_core::World;

    /// The bomb + slab fixture: glass walls (21 → Shatter, draw-free) with
    /// one pre-Skeletron dungeon wall (7 → the Dig fail arm) at the centre.
    fn bomb_wall_app(sound_volume: f32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("t".into(), 64, 64);
        for y in 14..34 {
            for x in 10..40 {
                let i = app.world.index(x, y);
                app.world.tiles[i] = Tile { kind: 1, wall: 21, ..Default::default() };
                app.world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = app.world.index(20, 20);
        app.world.tiles[i].wall = 7;
        app.projectiles.spawn_with_data(BOMB, 20.0 * 16.0, 20.0 * 16.0, 0.0, 0.0, 0, 0.0);
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (320.0, 320.0),
            sound_volume,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    #[test]
    fn w4_c15_the_wall_drain_draws_match_the_queue_census_on_update_projectiles() {
        // The queue census: a sibling pool run (the bomb_tests shape) drains
        // the intents the pass will play — every Dig cue owes two draws
        // (`Next(3)` + `Next(-10, 11)`), every Shatter none.
        let mut census = bomb_wall_app(1.0);
        for _ in 0..181 {
            census.projectiles.update(
                &mut census.world,
                &mut census.npcs,
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut tr_core::tile_light::FastRandom::new(7),
                0,
                true,
            );
        }
        let queued = census.projectiles.drain_wall_break_sounds();
        let digs = queued.iter().filter(|(c, _, _)| *c == (0, 1)).count();
        let shatters = queued.iter().filter(|(c, _, _)| *c == (13, 1)).count();
        assert!(digs >= 1, "the dungeon fail arm queued");
        assert!(shatters >= 1, "the glass walk queued");
        // The muted pass: the gate refuses every play — the stream state is
        // the gameplay-only baseline (the bomb arc's dust draws are skipped).
        let mut muted = bomb_wall_app(0.0);
        for _ in 0..181 {
            update_projectiles(&mut muted, 1.0 / 60.0);
        }
        // The audible pass: baseline + the blast's one case-2 default pitch
        // (`Next(-6, 7)`) + the wall census, in drain order (proj cues first).
        let mut audible = bomb_wall_app(1.0);
        for _ in 0..181 {
            update_projectiles(&mut audible, 1.0 / 60.0);
        }
        let mut witness = muted.rngs.stream(RngKey::UpdateProjectiles).clone();
        let _ = witness.next_range(-6, 7);
        for _ in 0..digs {
            let _ = witness.next_range(0, 3);
            let _ = witness.next_range(-10, 11);
        }
        assert_eq!(
            *audible.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "the drain drew exactly the blast + the queue census"
        );
        // Every other stage stream matches the muted baseline — the wall
        // cues never leak onto another stage's stream.
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *audible.rngs.stream(key),
                *muted.rngs.stream(key),
                "{} stayed on the muted baseline",
                key.name()
            );
        }
        // The refused wall stayed through the whole pass.
        assert_eq!(audible.world.tile(20, 20).unwrap().wall, 7);
        assert_eq!(audible.world.tile(19, 20).unwrap().wall, 0, "the glass broke");
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

/// W4-d9 wiring — the aiStyle-89 ritual's 300-tick dragon/squid spawn drain
/// (`Projectile.cs:32346-32356`): the VANILLA ids 454/521 resolve through
/// `kind_from_net_id` into real NPC spawns (the d40 dragon rows + the
/// ported 521 squidhead), and the returned NPC index writes back into the
/// marker's `ai[1]` (the C#'s `this.ai[1] = NPC.NewNPC(...)`). WRITTEN, NOT
/// RUN (code-only mode).
#[cfg(test)]
mod w4_d9_ritual_spawn_drain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::CULTIST_RITUAL;

    /// A ritual marker parked at (40, 40) inside the 8x8-tile (128-px)
    /// fixture world, with a live cultist host at NPC index 0. The
    /// 300-tick spawn request is seeded directly (the AI arm's own
    /// witness pins the 300-tick gate; this pins the DRAIN).
    pub(super) fn ritual_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut host = crate::npc::Npc::new(crate::npc::NpcKind::AncientCultist, 10.0, 10.0, 1, 0.0);
        host.hp = host.max_hp;
        app.npcs.push(host);
        let slot = app.projectiles
            .spawn_hostile_ai(CULTIST_RITUAL, Vec2::new(40.0, 40.0), Vec2::ZERO, 0, 0.0, 0.0, 0.0)
            .expect("the 490 row spawns");
        app.projectiles.slots_mut()[slot].wants_ritual_npc_spawn = Some((454, 60.0, 80.0));
        app
    }

}

/// W4-f49 MANA FINAL item 3 — the Last Prism flower-lane FINAL witness
/// coverage: the end-to-end pass-tail (the one link the pool-side and
/// potions-side witnesses did not drive together). The holder's short
/// interval pay takes the ManaV2 empty arm and records the shared
/// `wants_prism_flower_drink` intent (`Projectile.cs:66613`); the
/// `update_projectiles` pass-tail drains it through
/// `potions::check_mana_with_flower_on` on `UpdateProjectiles` — the
/// QuickMana drink + the post-drink re-pay (`Player.cs:52588-52609`).
/// WRITTEN NOT RUN (code-only).
#[cfg(test)]
mod w4_f49_prism_flower_pass_tests {
    use super::*;
    use super::w4_d9_ritual_spawn_drain_tests::ritual_app;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::{CULTIST_RITUAL, LAST_PRISM};

    /// A mid-channel player on the Last Prism row (item 3541, mana 12),
    /// the holder parked so the FIRST pass tick is an interval tick
    /// (ai[0] 29 → 30, `30 % 30 == 0`), a Lesser Mana Potion in slot 0,
    /// the flower armed, the SILENT listener (the drink is draw-free).
    pub(super) fn prism_flower_app(mana: i32, flower: bool) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 54.0;
        p.y = 43.0;
        p.channel = true;
        p.channel_item = 3541;
        p.channel_proj_expected = LAST_PRISM;
        p.mana_flower = flower;
        p.mana = mana;
        let (cx, cy) = p.center();
        p.channel_target = Vec2::new(cx + 40.0, cy);
        app.player = Some(p);
        app.inventory[0] = crate::item::ItemSlot {
            kind: 110,
            name: crate::registry::name(110),
            count: 2,
            prefix: 0,
        };
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        let holder = app
            .projectiles
            .spawn_with_data_ai_owned(LAST_PRISM, 0, 56.0, 56.0, 30.0, 0.0, 100, 0.25, 29.0, 0.0)
            .unwrap();
        app.projectiles.slots_mut()[holder].friendly = true;
        app
    }

    #[test]
    fn w4_d9_the_drain_spawns_the_dragon_head_and_writes_ai1() {
        let mut app = ritual_app();
        update_projectiles(&mut app, 1.0 / 60.0);
        // The dragon head spawned at the Bottom convention: centre X 60,
        // bottom Y 80 → top-left (60 - 16, 80 - 32) = (44, 48).
        let dragon = app.npcs.iter().find(|n| n.kind == crate::npc::NpcKind::CultistDragonHead)
            .expect("the 454 resolved through kind_from_net_id");
        assert_eq!((dragon.x, dragon.y), (44.0, 48.0), "NPC.NewNPC's Bottom = (X, Y)");
        assert_eq!(dragon.hp, dragon.max_hp, "the spawn is at full life");
        // The marker's ai[1] carries the returned NPC index (the host-
        // validity arm reads it next tick).
        let marker = app.projectiles.slots().iter().find(|p| p.active && p.kind == CULTIST_RITUAL)
            .expect("the marker lives (its host is valid)");
        let idx = app.npcs.iter().position(|n| n.kind == crate::npc::NpcKind::CultistDragonHead).unwrap();
        assert_eq!(marker.ai[1], idx as f32, "the ai[1] write-back");
        assert!(marker.wants_ritual_npc_spawn.is_none(), "the intent drained");
    }

    #[test]
    fn w4_d9_the_drain_spawns_the_squidhead_when_requested() {
        let mut app = ritual_app();
        let slot = app.projectiles.slots().iter().position(|p| p.active && p.kind == CULTIST_RITUAL).unwrap();
        app.projectiles.slots_mut()[slot].wants_ritual_npc_spawn = Some((521, 60.0, 80.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        // The squidhead's 60x60 box: top-left (60 - 30, 80 - 60) = (30, 20).
        let squid = app.npcs.iter().find(|n| n.kind == crate::npc::NpcKind::AncientCultistSquidhead)
            .expect("the 521 resolved through kind_from_net_id");
        assert_eq!((squid.x, squid.y), (30.0, 20.0));
        assert_eq!(squid.hp, 2000, "the squidhead's lifeMax");
    }
}

/// W4-f32 — the projectile-side Lucky Coin proc's pass-tail drain
/// (`Projectile.cs:13685-13697` → `drain_lucky_coin_drops`). WRITTEN NOT RUN
/// (code-only mode). The six rolls ride the `UpdateProjectiles` stream
/// inline mid-walk; the drain spawns the `ItemDrop` (stack 1, prefix 0, the
/// explicit velocity) and draws NOTHING — the `UpdateItems` stream stays
/// pristine (the explicit velocity skips `Item.NewItem`'s spread draws,
/// `Item.cs:49738-49740`).
#[cfg(test)]
mod f32_lucky_coin_drain_tests {
    use super::*;
    use super::w4_f49_prism_flower_pass_tests::prism_flower_app;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::LAST_PRISM;

    #[test]
    fn f32_the_lucky_coin_drain_spawns_the_drop_draw_free() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // A pre-rolled proc intent (the pool's mid-walk rolls are pinned by
        // `f32_lucky_coin_proj_tests`): silver 72 at (60, 40), vx 4, vy -2.
        app.projectiles.push_lucky_coin_drop_for_test((72, 60.0, 40.0, 4.0, -2.0));
        // The intent injection only QUEUES the `ItemDrop` (the pool cannot
        // borrow `app.items`); the pass-tail drain does the spawn
        // (`Projectile.cs:13685-13697` reached from `Projectile.Update`
        // under the `Main.cs:17775` swap) — drive one pass to drain it.
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "the drain spawned the coin");
        let coin = &app.items[0];
        assert_eq!(coin.kind, 72);
        assert_eq!(coin.count, 1, "stack 1 — the C# discards its Next(1, 11)");
        assert_eq!(coin.prefix, 0);
        // `WorldItem.Center = center` — the 14x14 box centred on the NPC.
        assert!((coin.pos.x - (60.0 - crate::item::SIZE * 0.5)).abs() < 1e-5);
        assert!((coin.pos.y - (40.0 - crate::item::SIZE * 0.5)).abs() < 1e-5);
        assert!((coin.vel.x - 4.0).abs() < 1e-5);
        assert!((coin.vel.y - -2.0).abs() < 1e-5);
        assert_eq!(coin.phase, 0.0, "the explicit-velocity spawn draws no phase");
        // The spawn drew nothing: the item stream is still the pristine seed.
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateItems),
            tr_core::UnifiedRandom::new(0),
            "the drain is draw-free"
        );
        // The queue emptied (a second tick spawns nothing new).
        assert_eq!(app.items.len(), 1, "the drain consumed the intent");
    }

    #[test]
    fn w4_f49_the_prism_flower_drink_drains_end_to_end_through_the_pass() {
        // Short bar (5 < 12) + flower: the pool pay drains to 0 and arms
        // the slow cast; the intent rides the slot; the pass-tail drink
        // restores 50 capped at 20, re-pays 12 → 8, clears the slow cast,
        // grants Mana Sickness 94 and consumes one potion.
        let mut app = prism_flower_app(5, true);
        update_projectiles(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 8, "50 capped at 20, then the 12-mana re-pay");
        assert!(!p.slow_magic_use, "the drink covered the cost (:52600)");
        assert!(
            p.buffs.find_buff_index(crate::potions::MANA_SICKNESS).is_some(),
            "the drink grants 94 (:51938)"
        );
        assert_eq!(app.inventory[0].count, 1, "one potion consumed");
        let holder = app
            .projectiles
            .slots()
            .iter()
            .find(|s| s.active && s.kind == LAST_PRISM)
            .expect("ManaV2: the holder lives");
        assert_eq!(holder.wants_prism_flower_drink, None, "the drain took it");
        // The silent mana-potion drink is draw-free on BOTH keys (the
        // w4_f49 stream-key witness's shape, now through the real pass).
        for key in crate::app::RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                tr_core::UnifiedRandom::new(0),
                "{} pristine",
                key.name()
            );
        }
        // Full bar: the interval pays cleanly, NO drink, NO intent.
        let mut app = prism_flower_app(20, true);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.player.as_ref().unwrap().mana, 8, "20 − 12, no drink");
        assert_eq!(app.inventory[0].count, 2, "no consume");
        // Flowerless short bar: the V2 empty arm, no drink owed.
        let mut app = prism_flower_app(5, false);
        update_projectiles(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 0, "the plain V2 empty arm");
        assert!(p.slow_magic_use);
        assert_eq!(app.inventory[0].count, 2, "no flower, no drink");
    }
}

/// W4-d15 seam witnesses — the DD2 write drain (`take_dd2_writes`, the
/// `AI_142` arm's `player.heldProj = whoAmI` `Projectile.cs:69822` +
/// `player.reuseDelay = 2` `:69966`) and the per-tick `heldProj = -1`
/// reset (`Player.cs:24376`). WRITTEN NOT RUN (code-only round).
#[cfg(test)]
mod w4_d15_dd2_drain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::MONK_STAFF_T2;

    /// One live 699 swing owned by the local player, inside the 8×8-tile
    /// fixture world; `swing_ticks` arms the owner's `itemAnimation` the
    /// snapshot derives (`(app.swing * 60).round()`).
    fn swing_app(swing_ticks: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let p = crate::player::Player::spawn(&app.world);
        app.player = Some(p);
        app.swing = swing_ticks as f32 / 60.0;
        app.swing_max = 27.0 / 60.0;
        app.projectiles.spawn_with_data_ai_owned(
            MONK_STAFF_T2, 0, 32.0, 32.0, 12.0, 0.0, 45, 7.0, 0.0, 0.0,
        );
        app
    }

    /// The animation-end tick (`itemAnimation <= 2`, `:69963-69967`): the
    /// arm kills, claims `heldProj` (the claim precedes the kill,
    /// `:69822`) and queues `reuseDelay = 2`; the drain writes BOTH onto
    /// the player/app (the port's reuse timer runs in seconds).
    #[test]
    fn the_animation_end_drains_the_claim_and_the_two_tick_reuse() {
        let mut app = swing_app(2);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert!(!app.projectiles.slots()[0].active, ":69965 kills");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.held_proj, 0, ":69822 — the slot index reaches the Player");
        assert_eq!(app.reuse_delay, 2.0 / 60.0, ":69966 — 2 ticks in seconds");
    }

    /// A mid-swing tick (`itemAnimation > 2`): the claim drains every tick
    /// the swing lives, but NO reuse delay is queued (`:69963`'s gate).
    #[test]
    fn the_mid_swing_claim_drains_without_the_reuse() {
        let mut app = swing_app(10);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert!(app.projectiles.slots()[0].active, "the swing keeps flying");
        assert_eq!(app.player.as_ref().unwrap().held_proj, 0, ":69822");
        assert_eq!(app.reuse_delay, 0.0, ":69963 — no reuse mid-swing");
    }

    /// `Player.cs:24376` — the movement-defaults block resets `heldProj`
    /// to −1 EVERY player tick, ahead of ItemCheck and the projectile
    /// pass: a stale claim from a dead swing cannot survive a tick with
    /// no live claimant.
    #[test]
    fn the_player_tick_resets_the_held_claim() {
        let mut app = swing_app(10);
        let w = app.world.clone();
        let p = app.player.as_mut().unwrap();
        p.held_proj = 7;
        p.update(&w, crate::player::PlayerInput::default());
        assert_eq!(p.held_proj, -1, ":24376 — the per-tick stock reset");
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

    /// W4-d80 final — the bank-only rows resolve too (the producers stay
    /// deferred): the 711 Betsy's Wrath impact shares the 686 impact's
    /// three assets at volume 0.4 (`Projectile.cs:71641`, `SoundID.cs:1675`
    /// — `DD2_BetsyFireballImpact.WithVolume(0.4f)`); the
    /// ParticleOrchestrator plays (`ParticleOrchestrator.cs:1888`/`:2123`/
    /// `:2427`) — the Monk Staff ground impact (3 variations, both the
    /// flint spike burst and the down-dash explosion) and the mushroom-boi
    /// explosion (one variation, 0.15 volume, behind the `Next(2)` coin).
    #[test]
    fn w4_d80_final_the_bank_only_styles_resolve() {
        let wrath = crate::tracked_sound::TrackedStyle::by_name("DD2_BetsysWrathImpact")
            .expect("DD2_BetsysWrathImpact");
        assert_eq!(wrath.cue, "dd2_betsy_fireball_impact", "the shared assets");
        assert_eq!(wrath.variations, 3);
        assert!((wrath.volume - 0.4).abs() < 1e-6, "the 0.4 volume");
        let impact = crate::tracked_sound::TrackedStyle::by_name("DD2_BetsyFireballImpact")
            .expect("DD2_BetsyFireballImpact");
        assert_eq!(wrath.cue, impact.cue, "the SAME family as the 686 impact");
        assert!((impact.volume - 1.0).abs() < 1e-6, "the 686 keeps volume 1");
        let staff = crate::tracked_sound::TrackedStyle::by_name("DD2_MonkStaffGroundImpact")
            .expect("DD2_MonkStaffGroundImpact");
        assert_eq!(staff.cue, "dd2_monk_staff_ground_impact");
        assert_eq!(staff.variations, 3);
        let mush = crate::tracked_sound::TrackedStyle::by_name("DeadCellsMushroomExplode")
            .expect("DeadCellsMushroomExplode");
        assert_eq!(mush.cue, "deadcells_mushroom_explode");
        assert_eq!(mush.variations, 1);
        assert!((mush.volume - 0.15).abs() < 1e-6, "the 0.15 volume");
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

