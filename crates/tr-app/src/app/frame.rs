//! The frame orchestrator — `Main.Draw`'s port. One `frame()` per
//! `RedrawRequested`: the sim-facing timers, the animated colour cycles,
//! the light parameters, the world-instance cache (G6) and the world plus
//! interface draw. Plus the benchmark epilogue and the two RNG helpers the
//! world simulation shares. (A7: extracted from `main.rs` whole.)

use std::time::Instant;

use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::lighting::{LightParams, LightStyle};
use wgpu::util::DeviceExt;

use crate::app::flow::{FrontDraw, FrontScreenBehaviours};
use crate::app::{App, RngKey, WorldFrameKey};
use crate::assets;
use crate::audio;
use crate::craft;
use crate::item::{self};
use crate::map;
use crate::npc::{self, NpcKind};
use crate::player;
use crate::render::bg::cave_parallax;
use crate::render::debug::{capture_frame, draw_debug};
use crate::render::entities::draw_player_map_icon;
use crate::render::sky::{color_of_the_skies, draw_sky, tile_color};
use crate::render::tiles::draw_tiles;
use crate::sound;
use crate::ui;
use crate::ui::text::{border_string, hud_text, hud_text_width};
use crate::world_sim::clock::advance_time_with_events;

impl App {
    /// Records one world-branch phase for the order-pinning test below
    /// (`S1-B-1`). A no-op outside `cfg(test)` — the phase names exist to make
    /// the `Main.DoUpdateInWorld_Inner` order (`Main.cs:17745-17822`) assertable
    /// instead of only readable.
    #[inline]
    fn phase(&self, #[cfg_attr(not(test), allow(unused_variables))] name: &'static str) {
        #[cfg(test)]
        phase_trace::record(name);
    }

    /// `Main.Draw`'s stack-split accelerator (`Main.cs:61781-61807`), ticked
    /// per drawn frame before the interface handlers read it: the idle reset,
    /// then the `stackDelay` decay ladder (30/25/20/15/10/5 frames keyed on
    /// the current delay 7..3) that bottoms the delay at 2 and grows
    /// `superFastStack` once the tries pass 10 (~2 s of holding).
    pub(crate) fn tick_stack_split(&mut self) {
        if self.stack_split == 0 {
            self.times_tried_to_fast_stack = 0;
            self.stack_counter = 0;
            self.stack_delay = 7;
            self.super_fast_stack = 0;
            return;
        }
        self.stack_counter += 1;
        let step = match self.stack_delay {
            7 => 30,
            6 => 25,
            5 => 20,
            4 => 15,
            3 => 10,
            _ => 5,
        };
        if self.stack_counter >= step {
            self.stack_delay -= 1;
            if self.stack_delay < 2 {
                self.stack_delay = 2;
                self.super_fast_stack += 1;
            }
            if self.times_tried_to_fast_stack < 10 {
                self.super_fast_stack = 0;
            }
            self.times_tried_to_fast_stack += 1;
            self.stack_counter = 0;
        }
    }

    /// `Main.cs:62466-62473` — the stack-split decay after the interface's
    /// MouseUI pass: no mouse button held resets the split, then one tick
    /// comes off.
    pub(crate) fn decay_stack_split(&mut self) {
        if !self.mine_down && !self.place_down {
            self.stack_split = 0;
        }
        if self.stack_split > 0 {
            self.stack_split -= 1;
        }
    }

    pub(crate) fn frame(&mut self) {
        // `Lang.InitGlobalSubstitutions` (S1-L-12b): refresh live `{Name}` values
        // before any UI/text resolves them this frame (`Lang.cs:100-167`).
        crate::lang::refresh_global_substitutions(self);

        let now = Instant::now();
        // `TR_SHOT` pins the animation clock: the sky's twinkle/drift phases read
        // `anim_time`, and wall-clock jitter between runs would make two shots of
        // the same scene differ in a few pixels. A fixed 1/60 step (what vsync
        // delivers anyway) keeps `anim_time = frames / 60` exact.
        let dt = if self.shot_path.is_some() {
            1.0 / 60.0
        } else {
            now.duration_since(self.last_frame)
                .as_secs_f32()
                .clamp(0.0, 0.1)
        };
        self.last_frame = now;
        self.frame_count += 1;
        // Main.cs:16801-16815,18376 — view zoom keys (S1-B-5).
        self.update_view_zoom_keys();
        // The adaptive fps loop's counter (`Main.cs:16961-16964`, S1-B-6):
        // the per-second arms run off the measured frames, dt-pinned under
        // `TR_SHOT` like the rest of the clock.
        self.gfx_quality
            .count_frame(dt, if self.shot_path.is_some() { 60 } else { 1 });
        // S1-B-3 — the step keys' snapshot pair (`WorldUpdateStepper.cs:39-43`):
        // `down` reads live (the frame's events have landed), then the
        // previous-frame pin shifts — the `Main.oldKeyState = Main.keyState`
        // analog (`Main.cs:18381-18382`, the same per-Update region the port
        // already runs unconditionally at the frame head). Every frame shifts
        // it — menus, pauses included — exactly like the C# snapshot.
        self.step_input_was = std::mem::replace(
            &mut self.step_input_down,
            crate::world_stepper::WorldUpdateStepper::get_step_key_state(
                &self.keys,
                self.mouse_back_held,
            ),
        );
        self.anim_time += dt;
        // `UserInterface.Update`'s lockout countdown
        // (`UserInterface.cs:203-204`): the pointer machine's 200 ms
        // state-change gate decays with the frame step, deterministic under
        // `TR_SHOT` (dt pinned to 1/60 above). `Main.UpdateUIStates`
        // (`Main.cs:17487-17491`) runs this whichever UI side is live, so it
        // must not sit inside the in-game branch below — see
        // [`App::tick_pointer_clock`].
        self.tick_pointer_clock(dt);
        // `Main.cs:17048-17051`: the PVP/team-change lockout counts down once per
        // update, beside the UI-state tick (`UpdateAudio`/`ItemSlot.UpdateInterface`
        // neighbours) — not on the fixed world step.
        if self.team_cooldown > 0 {
            self.team_cooldown -= 1;
        }
        // `IngameOptions.Draw`'s per-frame hover/animation state and its
        // `SoundEngine.PlaySound(12)`/`(10)` cues (`IngameOptions.cs:157-199`,
        // `:421-436`) — plus the sticky bar drag (`:424`). Runs on every frame
        // like the vanilla's, and before the HUD borrows the state.
        if let Some(cue) = crate::ui::ingame_options::update(self, self.pointer.left_held()) {
            self.audio.play(cue);
        }
        // `Main.DrawInterface_29_SettingsButton` (`Main.cs:45513-45527`): the
        // Settings button's position, its `exitScale` hover animation and the
        // `_settingsButtonIsPushedToSide` latch. Runs every frame the button is
        // visible (`playerInventory || ingameOptionsWindow || dead`) because the
        // inventory's accessory column reads the latch.
        if self.inventory_open || self.ingame_options.open {
            let (sw, sh) = self.screen_size();
            let (m_h, mouse) = (self.main_m_h(), self.mouse_pos);
            let dead = self.player.as_ref().is_some_and(|p| p.dead);
            crate::ui::ingame_options::refresh_button(
                &mut self.ingame_options,
                &self.lookups,
                Vec2::new(sw, sh),
                m_h,
                dead,
                mouse,
                crate::ui::inventory::extra_accessory_slots(),
                false,
            );
        }
        // `Main.DrawPageIcons`' tail (`Main.cs:42864-42867`): the drawn page is
        // whatever the held item forces (`ItemSlot.SelectEquipPage`), else the
        // page the player selected with the tabs. Recomputed every frame so the
        // auto-jump is undone the moment the cursor stack empties.
        self.resolve_equip_page();
        // `Main.DoUpdate_AnimateDiscoRGB`: the disco-ball colour cycle.
        self.advance_disco();
        let anim_time = self.anim_time;
        if !self.follow_player {
            self.pan_camera(dt);
        }
        // `Main.UpdateWorld_Time` (`Main.cs:18083-18097`, the `Main.UpdateTime`
        // call) is the last phase of the world branch's entity half, and the C#
        // never reaches it while `Main.gameMenu` is set (`Main.cs:17166-17175`
        // returns before `DoUpdateInWorld`). It therefore runs *inside* the
        // in-world branch below, after the dust pass — see the phase list there
        // (S1-B-1) and `docs/ANALYSIS.md` "World update order".
        // `Main.cs:19751-19765`: with Autosave on, `WorldGen.saveAndPlay` runs
        // every 600000 ms (10 minutes); the player/map save rides along.
        if self.menu.is_none() && self.front.is_none() && self.settings.auto_save {
            self.autosave += dt;
            if self.autosave >= 600.0 {
                self.autosave = 0.0;
                self.save_world();
            }
        }

        let screen = self
            .gpu
            .as_ref()
            .map(|g| Vec2::new(g.config.width as f32, g.config.height as f32))
            .unwrap_or(Vec2::new(1600.0, 900.0));

        // `Main.DrawInterface_16_MapOrMinimap` (`Main.cs:45771-45783`): `mH =
        // 256` while the minimap HUD is live — `mapEnabled && !mapFullscreen
        // && mapStyle == 1` (the fullscreen map draws no HUD at all,
        // `Main.cs:62147-62181`); the `+ 600` push-up clamp sits in `main_m_h`.
        let minimap_live = self.settings.map_enabled
            && self.map.as_ref().is_some_and(|m| !m.open)
            && self.map_style == 1;
        let m_h = ui::minimap::main_m_h(minimap_live, screen.y);

        // `FocusHelper.cs:160-163`: while the focused window draws the game's own
        // cursor the OS pointer is hidden (`Main.instance.IsMouseVisible = false`).
        // We only draw the vanilla cursor together with the in-world HUD so far, so
        // the OS pointer stays in menus and on the fullscreen map.
        let vanilla_cursor = self.menu.is_none()
            && self.front.is_none()
            && !crate::env::no_hud()
            && !(self.settings.map_enabled && self.map.as_ref().is_some_and(|m| m.open));
        if self.os_cursor_hidden != Some(vanilla_cursor) {
            if let Some(w) = &self.window {
                w.set_cursor_visible(!vanilla_cursor);
            }
            self.os_cursor_hidden = Some(vanilla_cursor);
        }

        // `Main.Camera.Center` + the volume sliders for `LegacySoundPlayer.PlaySound`.
        self.audio.set_listener(sound::Listener {
            camera_center: (self.camera.x, self.camera.y),
            sound_volume: self.settings.sound_volume,
            ambient_volume: self.settings.ambient_volume,
            quiet_ambient_sounds: false,
            game_menu: self.menu.is_some() || self.front.is_some(),
            // The worldgen worker never plays (the port's generation runs to
            // completion before the frame loop starts); `netMode == 2` is the
            // `NetRole::Host` server role (`LegacySoundPlayer.cs:382-388`,
            // S1-E-24).
            generating_or_loading: false,
            server: matches!(self.net, crate::app::NetRole::Host(_)),
        });

        // `SoundEngine.Update` (`SoundEngine.cs:48-68`, S1-E-23): the pause
        // edge (`FocusHelper.PauseSounds` = `!GameplayActive && netMode == 0`,
        // where `GameplayActive = IsSelectedApplication && !gamePaused` —
        // `FocusHelper.cs:11-20`/`:97-105`) rides the window-focus and the
        // S1-B-3 pause gate, then the pool's per-frame update re-volumes
        // every tracked `ActiveSound` against the fresh listener and reaps
        // the dead.
        let pause_sounds = (!self.window_focused || self.game_paused)
            && matches!(self.net, crate::app::NetRole::Off);
        self.audio.update_tracked(pause_sounds);

        // `Main.Ambience` (`Main.cs:13466-13515`): every 15 ticks (¼ s) the
        // waterfall/lavafall loops re-volume from the strength scans; the
        // volume math is the LegacySoundPlayer case 34/35 (the distance
        // attenuation + strength/50 + the 0.2/0.65 multipliers). The strength
        // scans ran in the waterfall pass this frame.
        self.ambient_counter += dt;
        if self.ambient_counter >= 0.25 {
            self.ambient_counter = 0.0;
            let ambient = self.settings.ambient_volume;
            let quiet = self.menu.is_some() || self.front.is_some();
            let sources = [
                (
                    self.ambient_waterfall.0,
                    self.ambient_waterfall.1,
                    self.ambient_waterfall.2,
                    0.2,
                ),
                (
                    self.ambient_lavafall.0,
                    self.ambient_lavafall.1,
                    self.ambient_lavafall.2,
                    0.65,
                ),
            ];
            for (channel, (strength, x, y, mult)) in sources.into_iter().enumerate() {
                // The positional attenuation (`LegacySoundPlayer.cs:383-397`).
                let dist = ((x - self.camera.x).powi(2) + (y - self.camera.y).powi(2)).sqrt();
                let (atten, pan) = if dist < sound::SOUND_ATTENUATION_DISTANCE {
                    (
                        1.0 - dist / sound::SOUND_ATTENUATION_DISTANCE,
                        ((x - self.camera.x) / (screen.x * 0.5)).clamp(-1.0, 1.0),
                    )
                } else {
                    (0.0, 0.0)
                };
                // `:1249-1279`: strength/50 (clamped 1) x the 0.2/0.65
                // multiplier x (1 - shimmerAlpha).
                let strength_factor = (strength / 50.0).min(1.0);
                let volume = if quiet {
                    0.0
                } else {
                    atten
                        * strength_factor
                        * mult
                        * (1.0 - self.scene_state.shimmer_alpha)
                        * ambient
                };
                self.audio.set_liquid_loop(channel, volume, pan);
            }
        }

        // `Main.UpdateAudio` runs on every frame, in the menus as well as in
        // the world (`Main.cs:17816`); the `Main.gameMenu` arm of
        // `UpdateAudio_DecideOnNewMusic` (`:12455`) picks the title track.
        // The port's audio clock was wired to the world step only, so the
        // front-end screens were silent (BUG-01).
        if self.menu.is_some() || self.front.is_some() {
            self.update_menu_music();
        }

        // The front end's Play tail (`UIWorldListItem.PlayGame` →
        // `Main.LoadWorld`, BUG-04): the pending world is swapped at a frame
        // boundary — after the front screens' clicks parked it and before the
        // world branch runs, so the swap never interleaves a half-ticked world.
        if let Some(path) = self.play_world.take() {
            match self.swap_world(&path) {
                Ok(()) => {
                    // `Main.LoadWorld` leaves `Main.gameMenu` false, so nothing
                    // may draw over the world the swap just installed (BUG-21:
                    // the lobby used to come back and the Play looked like it
                    // "dropped back" to the menu). `swap_world` ends on
                    // `enter_loaded_world`; this is the tripwire for any path
                    // that reinstates an overlay after it.
                    debug_assert!(
                        self.menu.is_none() && self.front.is_none(),
                        "the world swap must close both front-end sides"
                    );
                    log::info!("world: {}", path.display())
                }
                Err(e) => {
                    // A failed load keeps the current world: the menu stays
                    // alive with its own world rather than a black screen (the
                    // vanilla's `LoadWorld` failure keeps `Main.gameMenu`).
                    log::warn!("world load failed: {e:#}");
                    self.front_hint = Some(format!("world load failed: {e:#}"));
                }
            }
        }

        // S1-B-3 — the focus pause (`FocusHelper.UpdateFocus`'s `wantsToPause`
        // arm, `Main.cs:17128-17141`): `wantsToPause = (!IsActive &&
        // netMode == 0) && !SettingPlayWhenUnfocused`. The C# clears the mouse
        // release edges, runs `UpdateMenu` when `gameMenu` (the menu/front path
        // keeps flowing) and returns from *Update* — Draw keeps running, so the
        // port's fused `frame()` gates the world branch instead of returning.
        let focus_paused = !self.window_focused && !self.settings.play_when_unfocused;
        if focus_paused {
            // `Main.cs:17129-17134`: the mouse release edges clear every
            // focus-paused frame, so no edge accumulated while unfocused fires
            // on refocus. The port's frame-consumed click edges are the
            // press-edge pair the click cluster reads (`mouseLeftRelease` has
            // no stored per-frame flag here — releases are window events); the
            // held flags stay untouched exactly like the C# leaves `mouseLeft`
            // alone.
            self.ui_left_press = false;
            self.ui_right_press = false;
            // `Main.cs:17138`: `Main.gamePaused = true` (the `UpdateMenu` arm
            // above it keeps the menu/front path alive — nothing to call here,
            // its per-frame work is draw/event-driven).
            self.game_paused = true;
        }
        if !focus_paused && self.menu.is_none() && self.front.is_none() {
            // `Main.Draw`'s stack-split accelerator (`Main.cs:61781-61807`),
            // ticked before the interface handlers read the state.
            self.tick_stack_split();
            // `MinimapFrame.Update` runs before the inventory's click handling
            // (`MinimapFrameManager.cs:45`, `Main.cs:41037`'s `ItemSlot.Handle`).
            self.handle_minimap_clicks();
            self.handle_bestiary_clicks();
            // The chest panel's slot dispatch (`ChestUI.DrawSlots`' per-cell
            // `ItemSlot.Handle`, `ChestUI.cs:385`) runs before the crafting
            // and inventory handlers and consumes the press when a chest
            // slot, the shift transfer or a chest tile took it.
            if !self.handle_chest_clicks() && !self.handle_crafting_clicks() {
                self.handle_inventory_clicks();
            }
            // `Main.cs:62466-62473` — the stack-split decay, after the
            // interface's MouseUI pass.
            self.decay_stack_split();
            for (i, scale) in self.hotbar_scales.iter_mut().enumerate() {
                let target = if i == self.selected { 1.0 } else { 0.75 };
                if *scale < target {
                    *scale = (*scale + 0.05).min(target);
                } else if *scale > target {
                    *scale = (*scale - 0.05).max(target);
                }
            }
            // S1-B-3 — the in-game pause gate (`Main.cs:17194-17201`):
            // `if (CanPauseGame()) { DoUpdate_WhilePaused(); gamePaused = true;
            // return; } gamePaused = false;`. The click cluster above it is the
            // pre-gate interface pass (DW-1.3: the UI stays live while paused).
            let paused = self.can_pause_game();
            if paused {
                self.do_update_while_paused();
            }
            self.game_paused = paused;
            // `DoUpdateInWorld`'s gate (`Main.cs:17278`,
            // `ShouldUpdateEntities() && WorldUpdateStepper.ShouldUpdateWorld()`
            // — the stepper half is Phase 2's `world_stepper.rs`, fed the
            // frame-head snapshot pair). The short-circuit `!paused` mirrors
            // the C#'s early `return` (`:17197-17199`): a paused frame never
            // reaches the gate, so the pacing count freezes with it.
            let step_allowed = !paused
                && self.should_update_entities()
                && self
                    .stepper
                    .should_update_world(self.step_input_down, self.step_input_was);
            if step_allowed {
                self.game_update_count = self.game_update_count.wrapping_add(1);
                // `WorldUpdateStepper.DrawnThisUpdate = false` at the step
                // head (`Main.cs:17747`, right after `_gameUpdateCount++`).
                self.stepper.drawn_this_update = false;
                // `Main.UpdateWorld_Players` (`Main.cs:17825-17859`): the player
                // pass. The sleeping counts it exports (`:17844-17859`) gate the
                // clock's rate, so they land right after it.
                self.phase("players");
                self.update_player(dt);
                // `UpdateTimeRate` (`Main.cs:6199-6218`): the rate is ×5 while
                // every active player is fully asleep
                // (`PlayerSleepingHelper.FullyFallenAsleep`, 120 ticks).
                self.world.all_players_asleep = self
                    .player
                    .as_ref()
                    .is_some_and(|p| p.sleeping && p.sleep_time >= 120);
                let hp = self.player.as_ref().map_or(0, |p| p.hp);
                if hp < self.last_hp {
                    self.audio.play("Player_Hit_0");
                }
                self.last_hp = hp;
                // `Main.UpdateAudio_DecideOnNewMusic` reads the scene centre, not the
                // player: underground once the view centre passes the world surface
                // (`Main.cs:12815`).
                // The boss scan (`Main.cs:12224-12430`): the screen rect expanded
                // 5000 px around each active NPC's centre; the type→slot switch
                // feeds the flag chain. The port's bosses: King Slime (50 → slot
                // 19 → track 93, outranking the others) and the generic `boss`
                // slot 1 → 5 (no other ported kind carries `NPC.boss`).
                let (sw, sh) = self.screen_size();
                let screen_rect = [self.camera.x - sw * 0.5, self.camera.y - sh * 0.5, sw, sh];
                let mut king_near = false;
                let mut eoc_near = false;
                let mut generic_boss_near = false;
                for n in &self.npcs {
                    if n.hp <= 0 {
                        continue;
                    }
                    let (cx, cy) = n.center();
                    let near = cx + 5000.0 >= screen_rect[0]
                        && cx - 5000.0 <= screen_rect[0] + screen_rect[2]
                        && cy + 5000.0 >= screen_rect[1]
                        && cy - 5000.0 <= screen_rect[1] + screen_rect[3];
                    if !near {
                        continue;
                    }
                    if n.kind == NpcKind::KingSlime {
                        king_near = true;
                    }
                    if n.kind == NpcKind::EyeOfCthulhu {
                        eoc_near = true;
                    }
                    // `if (num3 == 0 && Main.npc[j].boss) num3 = 1;` (`:12366`).
                }
                let boss_music = if king_near {
                    Some(93) // MusicID.KingSlime (flag19, `:12596`)
                } else if eoc_near {
                    Some(5) // MusicID.Boss1 (the generic boss slot, `:12611`)
                } else {
                    // No other ported kind sets `boss` — the generic slot stays
                    // out until a third boss lands.
                    None
                };
                let underworld_px = (self.world.height - 200) as f32 * TILE_PX;
                self.music.update(audio::MusicEnv {
                    music_volume: self.settings.music_volume,
                    day: self.world.day_time,
                    underground: self.camera.y
                        >= self.world.world_surface as f32 * TILE_PX + screen.y * 0.5,
                    rain: self.rain.is_raining(),
                    boss_music,
                    clock: self.world.time,
                    eclipse: self.world.eclipse
                        && self.camera.y
                            < self.world.world_surface as f32 * TILE_PX + screen.y * 0.5,
                    shimmer: self.scene_metrics.zone_shimmer,
                    town_npcs: 0, // no town NPCs yet (M4-TOWN)
                    // — the M1.4-BG-20 ambience-track inputs (`Main.cs:11527-11600`) —
                    cloud_alpha: self.rain.visual_alpha(),
                    zone_rain: self.scene_metrics.zone_rain,
                    zone_snow: self.scene_metrics.zone_snow,
                    surface_atmospherics: self.scene_metrics.surface_atmospherics,
                    wind_speed: self.wind.speed_current,
                    shimmer_alpha: self.scene_state.shimmer_alpha,
                    atmo: crate::render::sky::atmosphere_factor(
                        self.camera.y,
                        self.world.world_surface,
                        self.world.width,
                    ),
                    ambient_volume: self.settings.ambient_volume,
                    shadow_candle: self.scene_metrics.shadow_candle_count > 0,
                    sandstorm: self.sandstorm.happening,
                    underworld: self.camera.y > underworld_px,
                    blood_moon: self.world.blood_moon,
                    windy_music: self.wind.happy_windy_day,
                    // The biome chain (`:12686-12920`): the space band with
                    // `num4 = maxTilesX / 4200`, the temple wall behind the scene
                    // centre, the metrics zones, the mushroom bg style and the
                    // storm flag.
                    space_band: {
                        let num4 = self.world.width as f32 / 4200.0;
                        let center_tile_y = self.camera.y / 16.0;
                        (center_tile_y - (65.0 + 10.0 * num4 * num4))
                            / (self.world.world_surface as f32 / 5.0)
                            < 1.0
                    },
                    temple_wall: self
                        .world
                        .tile((self.camera.x / 16.0) as i32, (self.camera.y / 16.0) as i32)
                        .is_some_and(|t| t.wall == 87),
                    dungeon: self.scene_metrics.zone_dungeon,
                    mushroom_bg: self.bg_style == 9
                        && self.camera.y
                            < self.world.world_surface as f32 * TILE_PX + screen.y * 0.5,
                    corrupt: self.scene_metrics.zone_corrupt,
                    crimson: self.scene_metrics.zone_crimson,
                    blood_leads_evil: self.scene_metrics.blood_tile_count
                        > self.scene_metrics.evil_tile_count,
                    meteor: self.scene_metrics.zone_meteor,
                    graveyard: self.scene_metrics.zone_graveyard,
                    desert: self.scene_metrics.zone_desert,
                    jungle: self.scene_metrics.zone_jungle,
                    snow: self.scene_metrics.zone_snow,
                    hallow: self.scene_metrics.zone_hallow,
                    storm: self.wind.storming,
                    ocean: self.scene_metrics.zone_beach,
                    center_y: self.camera.y,
                    surface_band_px: self.world.world_surface as f32 * TILE_PX + screen.y * 0.5,
                    rock_band_px: self.world.rock_layer as f32 * TILE_PX + screen.y * 0.5,
                    jungle_deep_band_px: (self.world.rock_layer as f32 - 50.0) * TILE_PX
                        + screen.y * 0.5,
                    quiet_sky: !self.rain.is_raining() && !self.wind.happy_windy_day,
                    menu: false,
                    menu_flags: Default::default(),
                    music_box: self.scene_metrics.active_music_box,
                    music_box_silence: self.scene_metrics.music_box_silence,
                });
                // `Main.UpdateWorld_SpawnNPCs` (`Main.cs:17878`) + `UpdateWorld_NPCs`
                // (`:17898`) — the port runs spawn and update in one pass.
                if !self.is_client() {
                    self.phase("npcs");
                    self.update_npcs(dt);
                }
                // `Main.UpdateWorld_Projectiles` (`Main.cs:17976`) runs **before**
                // `UpdateWorld_Items` (`:18004`).
                self.phase("projectiles");
                self.update_projectiles(dt);
                self.phase("items");
                self.update_items(dt);
                // `Main.UpdateWorld_Dust` (`Main.cs:18047`): the minecart sparks
                // ride this per-frame slot (the same slot the rain/sand dust get in
                // `world_sim::scene_scan::update_scene`).
                self.phase("dust");
                let (sw, sh) = self.screen_size();
                self.minecart_sparks
                    .tick(self.camera, Vec2::new(sw, sh), self.zoom);
                // `Main.UpdateWorld_Time` (`Main.cs:18083`) — `Main.UpdateTime`
                // runs after the entity half, never before it.
                self.phase("time");
                if !crate::env::day_pinned() {
                    // `FreezeTime`/`ModifyTimeRate` (`CreativePowers.cs:861-1000`/
                    // `:1441`): the powers' multiplier folds into the clock tick
                    // (the freeze stops the advance entirely).
                    let rate = self.creative_powers.time_rate();
                    if rate > 0.0 {
                        // The dawn/dusk event rolls ride the clock's own named
                        // stream (`Main.SwapRandom("UpdateTime")`, `Main.cs:17795`)
                        // inside `advance_time_with_events`, which also fires the
                        // phase-boundary achievements and flag clears
                        // (`Main.cs:65242-65246`, `:65383-65387`, `:65415`).
                        let any_over_120 = self.player.as_ref().is_some_and(|p| p.max_hp > 120);
                        let (blood_moon, eclipse) =
                            advance_time_with_events(self, dt * rate, any_over_120);
                        if blood_moon {
                            crate::chat::broadcast_death_text(
                                &mut self.chat,
                                &crate::lang::text("LegacyMisc.8"),
                                crate::chat::COLOR_WORLD,
                            );
                        }
                        if eclipse {
                            crate::chat::broadcast_death_text(
                                &mut self.chat,
                                &crate::lang::text("LegacyMisc.20"),
                                crate::chat::COLOR_WORLD,
                            );
                        }
                    }
                }
                // `WorldGen.UpdateWorld` (`WorldGen.cs:71495-71630`): the
                // `UpdateWorld_WorldGenAndInvasion` wrapper (`Main.cs:18099`) runs
                // *after* the time pass and walks `Wiring.UpdateMech` →
                // `TileEntity.PerformUpdates` → `Liquid.UpdateLiquid` → the growth
                // loops, in that order.
                self.phase("wiring");
                self.update_wiring(dt);
                self.phase("tile_entities");
                self.update_tile_entities(dt);
                self.phase("liquids");
                self.update_liquids(dt);
                self.phase("growth");
                self.update_growth(dt);
            }
            // `DoUpdate_AnimateWaterfalls` (`Main.cs:19545-19566`): the
            // `WaterfallManager.UpdateFrame`/`FindWaterfalls` pass — the
            // pre-gate Animate cluster (`:17146`), so it ticks while paused.
            self.phase("waterfalls");
            self.update_waterfalls(dt, screen);
            // S1-B-3: the post-gate weather cluster (`Main.cs:17222`'s
            // `Rain.MakeRain`, `:17228-17231`'s dayRate walks) — frozen while
            // the pause gate holds, live on stepper-blocked frames (the
            // `Main.cs:17278` gate sits *after* it in the C#; Phase 2's stepper
            // must not swallow it).
            if !paused {
                // M1.4-BG-22: the shared 60 Hz world tick — `Main.Update`'s
                // dayRate cluster (`Main.cs:17228-17231`): the `UpdateWeather`
                // thunder arm once, the wind walk ×`Main.dayRate`, then
                // `Sandstorm.UpdateTime` (`Main.cs:64966`).
                self.phase("weather");
                crate::world_sim::weather::update_world_tick(self, dt);
                self.phase("rain");
                self.update_rain(dt, screen);
                // S1-B-3 Phase 2 (decision #5): `scene_state` moved out of
                // this pause-gated block — `Main.SceneState.Update` sits in
                // `DoUpdateInWorld_Inner`'s step-gated tail (`Main.cs:17816`),
                // so a stepper-blocked frame freezes it too. It now runs in
                // the `step_allowed` tail below, after the camera walks.
            }
            // The SceneState blizzard loops' sound emitter rides inside
            // `update_scene` (`scene_scan.rs`'s `SceneState::update` call —
            // the C# keeps the block inside `ApplyVisuals`, `SceneState.cs:189`).
            // `Main.UpdateWindyDayState` (`Main.cs:12990-13027`, called from
            // the per-tick audio update): the sticky windy-day/storm flags the
            // ambience gates read; the hysteresis bands make the per-frame
            // evaluation equivalent. The pre-gate cluster (`:17182`) — it ticks
            // while paused, reading the frozen clock.
            self.phase("windy_day");
            self.wind.update_windy_day_state(
                self.rain.visual_alpha(),
                self.world.time,
                self.world.day_time,
            );
            // S1-B-3: the post-gate tail — `Main.AmbienceServer.Update`'s own
            // `!gamePaused` arm (`Main.cs:17214`). The tile-edit and net
            // passes move under the step gate below (batch review fix 1); the
            // map reveal keeps the pause gate only (see its block).
            if !paused {
                // `Main.AmbienceServer.Update` (`Main.cs:17214-17216`) +
                // `SkyManager.Update` (`Main.cs:17139`): the ambient sky scheduler
                // and its entities, stepped on the 60 Hz accumulator.
                self.phase("ambience");
                {
                    let ambience = &mut self.ambience;
                    let ambience_rng = &mut self.ambience_rng;
                    let ctx = crate::ambience::AmbienceCtx {
                        world: &self.world,
                        day_time: self.world.day_time,
                        eclipse: self.world.eclipse,
                        blood_moon: self.world.blood_moon,
                        pumpkin_moon: self.world.pumpkin_moon,
                        snow_moon: self.world.snow_moon,
                        hard_mode: self.world.hard_mode,
                        is_raining: self.rain.is_raining(),
                        is_storming: self.wind.storming,
                        happy_windy_day: self.wind.happy_windy_day,
                        too_windy_for_butterflies: self.wind.too_windy_for_butterflies(),
                        butterfly_chance: self.world.butterfly_chance,
                        wind: self.wind.speed_current,
                        zone_desert: self.scene_metrics.zone_desert,
                        zone_hallow: self.scene_metrics.zone_hallow,
                        zone_beach: self.scene_metrics.zone_beach,
                        zone_corrupt: self.scene_metrics.zone_corrupt,
                        zone_crimson: self.scene_metrics.zone_crimson,
                        zone_jungle: self.scene_metrics.zone_jungle,
                        player_center: self.player.as_ref().map_or(self.camera, |p| {
                            Vec2::new(p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5)
                        }),
                        // `plr.position.Y` is the hitbox TOP (`AmbienceServer.cs:191`).
                        player_y: self.player.as_ref().map_or(self.camera.y, |p| p.y),
                        // `SkyManager.Update`'s `max(1, dayRate)` loop
                        // (`SkyManager.cs:24-46`) rides `Main.dayRate`
                        // (`Main.cs:6199-6206`): 60 while a dial fast-forwards.
                        day_rate: self.world.day_rate(),
                    };
                    ambience.update(dt, &ctx, ambience_rng);
                }
            }
            // S1-B-3 (batch review fix 1) — the `DoUpdateInWorld` body's
            // tile-edit and net halves move under the step gate: the C#'s
            // `LookForTileInteractions`/`ItemCheck` (the mining and placing)
            // ride `Player.Update` inside `DoUpdateInWorld`
            // (`Player.cs:27430`), and `UpdateWorld_ServerOrClient` sits in
            // its tail (`Main.cs:17808`) — a stepper-blocked frame cannot
            // mine, place or sync, exactly like the vanilla. The pass order
            // is unchanged (ambience → editing → net → map → the
            // camera/SceneState tail), so no RNG stream reorders: the gate
            // only removes draws.
            if step_allowed {
                self.phase("editing");
                self.update_editing(dt, screen);
                self.phase("net");
                self.net_tick(dt);
            }
            // The map reveal keeps the pause gate only — its C# source,
            // `LightingEngine.ExportToMiniMap` (`LightingEngine.cs:246-270`),
            // is the draw-side lighting export: Draw keeps running on
            // stepper-blocked frames, so vanilla keeps revealing the map
            // while the world steps are frozen. No update-side `UpdateMap`
            // exists in the 1.4.5.8 decompile to gate it on. The edit-side
            // refresh drain (`Main.cs:61862`) is itself `!gamePaused`-gated
            // in the C# (`Main.updateMap.HasValue && !Main.gamePaused`), so
            // it rides the same gate here.
            if !paused {
                self.phase("map");
                if let Some(m) = self.map.as_mut() {
                    m.update(&self.world, self.player.as_ref(), &self.light, dt);
                    // `MapUpdateQueue.Update()` (`Main.cs:61862`): the edit-side
                    // refresh drains through `WorldMap.UpdateType`; the changed
                    // tiles re-upload (`MapRenderer.QueueChange`'s analog).
                    let changed = self
                        .map_queue
                        .update(&self.world, m);
                    m.push_dirty(changed);
                }
            }
            // The world-branch tail's camera walks (`Main.cs:17814-17815`):
            // `UpdateCameraPan` (the tracking/pan target's MoveTowards) and
            // `UpdateCameraLerp` (the teleport transition's timer) — S1-B-4.
            // They sit inside `DoUpdateInWorld_Inner`'s tail block
            // (`:17795-17822`), so they freeze with the world step; the
            // draw-time camera (`update_camera_at_draw`, S1-B-4) keeps
            // computing from the frozen walk state.
            if step_allowed {
                let mut camera_state = std::mem::take(&mut self.camera_state);
                crate::world_sim::camera::update_camera_pan(self, &mut camera_state);
                crate::world_sim::camera::update_camera_lerp(self, &mut camera_state);
                self.camera_state = camera_state;
                // S1-B-3 Phase 2 (decision #5): `Main.SceneState.Update` rides
                // the same gated tail — `DoUpdateInWorld_Inner`'s order is
                // `UpdateCameraPan`, `UpdateCameraLerp`, `SceneState.Update`
                // (`Main.cs:17814-17816`), so the fades freeze with the world
                // step. M1.4-BG-18 phase 3: the fused
                // `Main.UpdateSceneMetrics` + `Main.SceneState.Update` — the
                // scan, the fades and the `ApplyVisuals` driver; the post
                // branch consumes `filter_driver.output`.
                self.phase("scene_state");
                crate::world_sim::scene_scan::update_scene(self, dt, screen);
            }
            // The gfx-quality budgets (`Main.cs:17056-17101`, S1-B-6): the
            // `qaStyle` overrides, `maxDustToDraw`, `Gore.goreTime`, the
            // liquid pass pair and the legacy render phases.
            let mut gfx = std::mem::take(&mut self.gfx_quality);
            gfx.apply(self);
            self.gfx_quality = gfx;
        }
        // S1-B-3 — `WorldUpdateStepper.DrawnThisUpdate = true`
        // (`Main.cs:62478`, Draw's stack-split block `:62470-62477`). The
        // fused frame orders the two flag sites as the C#'s Update→Draw
        // sequence does: a step head resets it (`:17747`), then the draw half
        // marks the frame — so a stepping frame still ends `true`, the C#'s
        // post-Draw state. Runs on every in-world frame (Draw-side: a
        // focus-paused frame draws too); menu/front frames never reach it
        // (the C# block sits in the in-game Draw branch).
        if self.menu.is_none() && self.front.is_none() {
            self.stepper.drawn_this_update = true;
        }
        // `TR_NET_EDIT=tx,ty`: one client edit request, used to verify the round trip.
        if self.is_client() && self.frame_count == 90 {
            if let Some((x, y)) = crate::env::net_edit() {
                self.request_edit(x, y, 0, 0);
                log::info!("net edit requested at {x},{y}");
            }
        }
        self.poll_net();
        // The `netOffset` decay (`UpdateNetOffset`, `Player.cs:28721-28749`) —
        // client-only, and after the incoming snapshots are applied.
        self.net_interp_tick(dt);
        // `Main.DrawBG`: advance the surface-background cross-fade once per drawn
        // frame, exactly where the original runs it (inside the draw pass).
        self.update_bg_visibility();
        // Render between fixed steps so the jump/fall is not a 60 Hz staircase on
        // high-refresh displays.
        let player_alpha = (self.player_step.residual() / (1.0 / 60.0)).clamp(0.0, 1.0);
        // The dead player renders nothing until the respawn (vanilla draws
        // no corpse; the ghost is master-mode only).
        let mut render_player = self
            .player
            .as_ref()
            .filter(|p| !p.dead)
            .map(|p| p.interpolated(player_alpha));
        // The body rows — the item-use rows included — are all computed by
        // `Player::update_frames` (`PlayerFrame()`) inside the fixed step.
        let player_key = render_player.as_ref().map_or(0, |p| p.render_key());
        let npc_key = npc::render_key(&self.npcs);
        let mut stats = tr_render::sprite::RenderStats::default();
        let mut tile_stats = self.tile_stats;

        // `Main.DoDraw_UpdateCameraPosition` (`Main.cs:62590-62649`, S1-B-4):
        // the camera snaps/lerps/clamps at DRAW time, ahead of every read
        // this frame's render makes of it — the update half above kept the
        // previous frame's position (the C#'s update reads it too).
        let mut camera_state = std::mem::take(&mut self.camera_state);
        crate::world_sim::camera::update_camera_at_draw(self, &mut camera_state);
        self.camera_state = camera_state;

        // The chest-rename editor's per-frame reads — `Main.editChest`
        // (`Main.cs:1747`), `Main.npcChatText`'s editor copy
        // (`UIVirtualKeyboard.cs:1061-1067`) and the stored `Chest.name`
        // (`ChestUI.cs:115-122`, the `WorldChest` entity's own field since the
        // M6-SAVE-1 phase-3 fold) — parked before the draw site's field
        // borrows (the keyboard session owns the editor state; the name
        // reads the entity directly so the borrow stays field-scoped).
        // `Main.DoDraw_UpdateCameraPosition` (`Main.cs:62590-62649`, S1-B-4):
        // the camera snaps/lerps/clamps at DRAW time, ahead of every read
        // this frame's render makes of it.
        let mut camera_state = std::mem::take(&mut self.camera_state);
        crate::world_sim::camera::update_camera_at_draw(self, &mut camera_state);
        self.camera_state = camera_state;
        let chest_editing = self.chest_editing();
        let chest_edit_text = self.chest_edit_text().to_string();
        let chest_open = self.open_container();
        let chest_stored_name = crate::chest::stored_name(&self.world_chests, chest_open);
        // `Chest.item.Length` (`ChestUI.cs:376-377`) — the open chest's own
        // per-chest slot count from the file (`Chest.maxItems`), the default
        // 40 for every port-created chest.
        let chest_max_items = crate::chest::chest_max_items(&self.world_chests, chest_open);
        // `Main.DrawPVPIcons`'s state block, snapshotted before the disjoint
        // borrow below (it reads `self.player` / `self.net`).
        let pvp_icon_state = self.pvp_icon_state();
        // The team-based-spawns seed gate (`TeamBasedSpawnMapLayer.cs:10`)
        // and the vampire-seed — snapshotted before the disjoint borrows.
        let team_based_spawns = self
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::TEAM_BASED_SPAWNS);
        // The buff bar's inputs (`Main.DrawInterface_Resources_Buffs`,
        // `Main.cs:43203`): the right-release edge (`Main.mouseRightRelease`)
        // and the vampire-seed gate (`Main.vampireSeed`, the duration hides
        // `:43298`). The removals drain in the tick.
        let right_release = std::mem::take(&mut self.ui_right_release);
        let vampire_seed = self
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM);
        // `Player.hideVisibleAccessory` (`Player.cs:1708`) — snapshotted for the
        // same reason (the accessory ticks, `Main.cs:41425-41460`).
        let hide_visible_accessory = self
            .player
            .as_ref()
            .map(|p| p.hide_visible_accessory)
            .unwrap_or([false; 10]);
        // `Main.ActivePlayerFileData.Player.difficulty` for the world-select
        // grey-out (`UIWorldSelect.cs:179`) — snapshotted before the disjoint
        // borrow below (it reads `self.session_draft`).
        let active_difficulty = self.active_player_difficulty();

        {
            let App {
                gpu,
                renderer,
                post,
                noise_tex,
                water_mask,
                filter_driver,
                world,
                atlas,
                lookups,
                clouds,
                sky,
                bg_alpha_front,
                bg_alpha_far,
                stars,
                player_assets,
                player,
                npcs,
                npc_assets,
                inventory,
                selected,
                edit_target,
                camera,
                zoom,
                debug,
                frame_history,
                last_stats,
                frame_count,
                shot_path,
                shot_at,
                atlas_stats,
                light,
                scratch,
                debug_cache,
                light_mode,
                has_liquids,
                liquids,
                waterfall,
                rain,
                sand,
                minecart_sparks,
                liquid_alpha,
                wind,
                tree_wind_counter,
                grass_wind_counter,
                sunflower_wind_counter,
                sunflower_wind_step,
                leaves,
                items,
                swing,
                projectiles,
                world_key,
                menu,
                front,
                front_hint,
                settings,
                remote_players,
                map,
                hell_black_box_bottom,
                ..
            } = self;

            if let (Some(gpu), Some(renderer)) = (gpu.as_ref(), renderer.as_mut()) {
                // `TR_DAY` overrides the time of day (0 = midnight, 1 = noon) so a
                // screenshot can be compared against the original at a fixed time.
                // The override only sets the clock — the sky/tile colour comes from
                // the same `SetBackColor` path as the natural clock (the old
                // night→day lerp bypassed the ramps and the moon-phase floor).
                let override_day = crate::env::day_override();
                let (is_day, natural_time) = match override_day {
                    Some(d) => {
                        let is_day = d >= 0.5;
                        // Pin safely INSIDE the half: the old endpoints sat on the
                        // day/night boundaries (54000/32400), and one tick of the
                        // live clock flipped the freshly-ported `SetBackColor`
                        // ramps into the other half mid-shot.
                        let clock = if is_day {
                            // [13500 dawn .. 27000 noon] — `TR_DAY=1.0` = noon.
                            13500.0 + d as f64 * 13500.0
                        } else {
                            // [16215 midnight .. 30800 late night].
                            16200.0 + (0.5 - d) as f64 * 29200.0
                        };
                        (is_day, clock)
                    }
                    None => (world.day_time, world.time),
                };
                // `Main.SetBackColor` + `ApplyColorOfTheSkiesToTiles`. Quantise the
                // daylight/sky colour to one-second steps: the sun still moves
                // smoothly on the draw pass, but the light map and world range can
                // be reused for a second.
                let quantised = (natural_time / 60.0).floor() * 60.0;
                let sky_tint = color_of_the_skies(quantised, is_day, world.moon_phase);
                let clock_time = natural_time;
                // `SetBackColor`: rain multiplies the sky (and therefore the surface
                // sunlight) by `1 - cloudAlpha * 0.9`.
                let rain_alpha = rain.visual_alpha();
                let rain_dim = (1.0 - rain_alpha * 0.9).max(0.1);
                let mut sky_tint = [
                    sky_tint[0] * rain_dim,
                    sky_tint[1] * rain_dim,
                    sky_tint[2] * rain_dim,
                ];
                // `Main.instance.HorizonRenderer.ModifyHorizonLight`
                // (`Main.cs:63921`): the sunrise/sunset gradient entries blend
                // into the sky colour before it becomes the tile light.
                let atmo = crate::render::sky::atmosphere_factor(
                    camera.y,
                    world.world_surface,
                    world.width,
                );
                crate::render::sky::modify_horizon_light(
                    &mut sky_tint,
                    world.moon_phase,
                    is_day,
                    quantised,
                    atmo,
                    rain_alpha,
                );
                let sky_light = tile_color(sky_tint);
                // `Main.DoUpdate_AnimateCursorColors`: mouseTextColor oscillates
                // 190..=255, changing by 1 per frame, driving light flicker.
                self.mouse_text_color = (self.mouse_text_color as i32
                    + self.mouse_text_color_dir as i32)
                    .clamp(190, 255) as u8;
                if self.mouse_text_color >= 255 {
                    self.mouse_text_color_dir = -1;
                }
                if self.mouse_text_color <= 190 {
                    self.mouse_text_color_dir = 1;
                }
                // `Main.mouseColor = Main.selColor` runs every frame the colour
                // screen is open (`Main.cs:50324`), and `CursorColor`
                // (`Main.cs:52490`) reads that static, so the tint follows the
                // bars live during a drag and keeps the stored colour once the
                // menu closes. The menu owns the live copy while it is open.
                self.cursor.mouse_color = match menu {
                    Some(m) => m.settings.mouse_color,
                    None => settings.mouse_color,
                };
                // `Main.CursorColor()` (`Main.cs:52477`): the cursor fade pulse.
                self.cursor.update();
                // `Main.DoUpdate_AnimateTileGlows`: the demon torch colour ramp.
                self.demon_torch += self.demon_torch_dir * 0.01;
                if self.demon_torch > 1.0 {
                    self.demon_torch = 1.0;
                    self.demon_torch_dir = -1.0;
                }
                if self.demon_torch < 0.0 {
                    self.demon_torch = 0.0;
                    self.demon_torch_dir = 1.0;
                }
                // `TileLightScanner.Update()`: one `NextSeed()` per frame, then
                // `WithModifier(x, y)` per tile (done inside the scanner).
                let mut light_rng = tr_core::tile_light::FastRandom::new(self.light_rng_seed);
                light_rng.next_seed_in_place();
                self.light_rng_seed = light_rng.seed;
                let params = LightParams {
                    sky: sky_light,
                    hell_glow: 0.55,
                    mouse_text_color: self.mouse_text_color,
                    demon_torch: self.demon_torch,
                    disco: self.disco,
                    // `Main.GlobalTimeWrappedHourly = TotalGameTime.TotalSeconds % 3600`.
                    global_time: anim_time % 3600.0,
                    // `Main.timeForVisualEffects`: ticks since start, wrapped at 216000.
                    time_for_visuals: (anim_time * 60.0) % 216000.0,
                    time: clock_time as f32,
                    day_time: is_day,
                    raining: rain.is_raining(),
                    // `Main.shimmerDarken` / `Main.liquidAlpha[13]`: shimmer is not
                    // simulated yet, so both stay at their neutral value.
                    // `Main.shimmerDarken` (SceneState.UpdateShimmer ramps).
                    shimmer_darken: self.scene_state.shimmer_darken,
                    shimmer_alpha: 0.0,
                    // `Main.sundialCooldown`/`Main.moondialCooldown`
                    // (`Main.cs:2161`/`:2165`): the scanner and the glow pass
                    // mute the dial light while one ticks (0..=8, u8 on World).
                    sundial_cooldown: i32::from(world.sundial_cooldown),
                    moondial_cooldown: i32::from(world.moondial_cooldown),
                    // `Main.jellyfishCageMode`: `Main.AnimateTiles` is not ported, so
                    // every cage stays in mode 0.
                    cage_mode: [[0u8; tr_core::tile_light::CAGE_FRAMES]; 3],
                    rng_seed: self.light_rng_seed,
                    // `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023`):
                    // false until the EchoMonolith / 4409-accessory systems land;
                    // `TR_SHOW_INVISIBLE=1` pins it on for A/B shots.
                    draw_invisible_walls: crate::env::show_invisible(),
                };
                let sky_uv = atlas
                    .as_ref()
                    .and_then(|a| assets::Lookups::sheet_uv(a, "Background_0"));
                let mut style = LightStyle::for_mode(*light_mode);
                // `Lighting.UpdateGlobalBrightness` (`Lighting.cs:108-116`):
                // the Darkness debuff's `blind` flag drops GlobalBrightness
                // 1.2 → 1.0 while it lasts.
                if player
                    .as_ref()
                    .is_some_and(|p: &crate::player::Player| p.blind)
                {
                    style.brightness = 1.0;
                }
                // `Main.ForcedMinimumZoom` (`Main.cs:61989-61994`, computed per
                // draw) splits the two view matrices: the background chain
                // zooms to the bare floor (`BackgroundViewMatrix.Zoom`,
                // `:61998`) and the world chain multiplies it by the clamped
                // `GameZoomTarget` (`GameViewMatrix.Zoom`, `:61999`) — the
                // port's `zoom` is that clamp (S1-B-5, S1-G-7).
                let bg_zoom = App::forced_minimum_zoom((screen.x, screen.y));
                let world_zoom = bg_zoom * *zoom;
                // Snap the render camera so integer world coordinates land on whole
                // screen pixels at any zoom. With a fractional camera, nearest-neighbour
                // sampling makes every tile texture crawl sub-pixel as the camera moves
                // while the player (drawn with `snap_to_screen`) stayed crisp, so the
                // world appeared to slide under the character. Rounding `cam * zoom`
                // keeps the step at one screen pixel even at 1.5x / 2x.
                let draw_cam = (*camera * world_zoom).round() / world_zoom;
                let surface_px = world.world_surface as f32 * TILE_PX;
                // `Main.DrawWaters` (`Main.cs:57473-57489`): the `liquidAlpha`
                // cross-fade ramps per frame while the current water style is a
                // plain water style — lava (1), honey (11) and shimmer (14) sit
                // outside the ramp.
                let water_style_now = crate::biome::calculate_water_style(world, draw_cam);
                if crate::render::liquids::is_liquid_style_water(water_style_now) {
                    for (i, a) in liquid_alpha.iter_mut().enumerate() {
                        *a = if i == water_style_now {
                            (*a + 0.2).min(1.0)
                        } else {
                            (*a - 0.2).max(0.0)
                        };
                    }
                }
                // `Main.DoDraw_Waterfalls` + `WaterfallManager.AddLight`: the
                // waterfall walk runs once per frame, before the light map and
                // outside the G6-cached world pass — its quads are buffered for
                // the tile pass and its `AddLight` list feeds the light compute.
                // Running it unconditionally (not only on world-range rebuilds)
                // keeps the light feed a pure function of the frame index.
                let mut leaf_spawns = Vec::new();
                let mut leaf_rng = tr_core::tile_light::FastRandom::new(
                    (*frame_count as u64).wrapping_mul(0x6A09_E667_F3BC_C909) ^ world.seed,
                );
                if !crate::env::no_waterfall() {
                    let half_v = screen * 0.5 / world_zoom;
                    let vx0 = (((camera.x - half_v.x) / TILE_PX).floor() as i32 - 1)
                        .clamp(0, world.width - 1);
                    let vx1 = (((camera.x + half_v.x) / TILE_PX).ceil() as i32 + 1)
                        .clamp(0, world.width - 1);
                    let vy0 = (((camera.y - half_v.y) / TILE_PX).floor() as i32 - 1)
                        .clamp(0, world.height - 1);
                    let vy1 = (((camera.y + half_v.y) / TILE_PX).ceil() as i32 + 1)
                        .clamp(0, world.height - 1);
                    let lights = waterfall.draw(
                        world,
                        light,
                        &style,
                        anim_time,
                        liquid_alpha,
                        params.mouse_text_color,
                        (*frame_count as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ world.seed,
                        [vx0, vy0, vx1, vy1],
                    );
                    scratch.waterfall_quads = waterfall.take_quads();
                    // `WaterfallManager.DrawWaterfall`'s accumulators
                    // (`WaterfallManager.cs:710-733` + the store `:960-965`):
                    // per nearby fall segment, `1 - dist/(screenWidth*0.75)`
                    // summed into the waterfall (any kind except the lava and
                    // the rain/shimmer falls) or the lavafall (kind 1)
                    // strength; the nearest segment's world position rides
                    // along for the loop pan.
                    let mut wfs: [f32; 3] = [0.0, 0.0, 0.0];
                    let mut lfs: [f32; 3] = [0.0, 0.0, 0.0];
                    let cam_c = Vec2::new(camera.x + screen.x * 0.5 - screen.x * 0.5, camera.y);
                    let _ = cam_c;
                    for q in &scratch.waterfall_quads {
                        let (fx, fy) = (q.pos[0], q.pos[1]);
                        let dx = (fx - camera.x).abs();
                        let dy = (fy - camera.y).abs();
                        if dx >= screen.x * 2.0 || dy >= screen.y * 2.0 {
                            continue;
                        }
                        let strength = 1.0 - (dx * dx + dy * dy).sqrt() / (screen.x * 0.75);
                        if strength <= 0.0 {
                            continue;
                        }
                        let (lava, slot) = if q.texture == 1 {
                            (true, &mut lfs)
                        } else {
                            (false, &mut wfs)
                        };
                        let _ = lava;
                        slot[0] += strength;
                        if dx < slot[1].abs() || slot[1] == 0.0 {
                            slot[1] = dx;
                        }
                        if dy < slot[2].abs() || slot[2] == 0.0 {
                            slot[2] = dy;
                        }
                    }
                    self.ambient_waterfall = (wfs[0], wfs[1], wfs[2]);
                    self.ambient_lavafall = (lfs[0], lfs[1], lfs[2]);
                    scratch.waterfall_lights = lights;
                    // `WaterfallManager.TrySparkling`: drain the walk's spawns
                    // into the rain module's dust particles.
                    rain.spawn_sparkles(waterfall.take_sparkles());
                } else {
                    scratch.waterfall_quads.clear();
                    scratch.waterfall_lights.clear();
                }

                // `TileDrawing.Update` (`TileDrawing.cs:367-375`): the wind WALK
                // itself lives on the shared world tick now
                // (`world_sim::weather::update_world_tick`, ×`Main.dayRate`,
                // `Main.cs:17228-17231`); the sway counters stay per frame,
                // faster with stronger wind.
                wind.sway_enabled = settings.tiles_sway && !crate::env::no_wind();
                // `TR_WIND_FORCE=<speed>`: pin `windSpeedCurrent` so the sway and
                // the leaf frequency are screenshotable at a chosen strength.
                if let Some(f) = crate::env::wind_force() {
                    wind.speed_current = f.clamp(-1.24, 1.24);
                }
                tr_core::wind::advance_tree_counter(tree_wind_counter, wind.speed_current);
                tr_core::wind::advance_counter(grass_wind_counter, wind.speed_current);
                // `TileDrawing.Update`'s sunflower counter (`TileDrawing.cs:374`)
                // runs per world-update tick in the original
                // (`TilesRenderer.Update()` in `Main.DoUpdateInWorld_Inner`,
                // `Main.cs:17812`), so it steps on the 1/60 accumulator instead of
                // the redraw cadence above — the flower's cadence stays
                // tick-quantised (and screenshotable) at any refresh rate.
                sunflower_wind_step.tick(dt, |_| {
                    tr_core::wind::advance_sunflower_counter(
                        sunflower_wind_counter,
                        wind.speed_current,
                    );
                });

                // Liquids animate the texture at 8 fps; include the frame in the G6
                // key only when the world has any liquid.
                let liquid_anim = if *has_liquids {
                    ((anim_time * 8.0) as u32) & 15
                } else {
                    0
                };

                // G6: the whole world range (sky strips, tiles, walls, dark rects)
                // depends on exactly these values. A hit reuses last frame's uploaded
                // instance range and only rebuilds the UI.
                let key = WorldFrameKey {
                    cam_tile: [
                        (camera.x / TILE_PX).floor() as i32,
                        (camera.y / TILE_PX).floor() as i32,
                    ],
                    // The effective GameView zoom — the floor times the
                    // clamped target (`Main.cs:61999`) — is what the cached
                    // world range's pixels depend on.
                    zoom: world_zoom.to_bits(),
                    screen: [screen.x.to_bits(), screen.y.to_bits()],
                    sky_light: [
                        params.sky[0].to_bits(),
                        params.sky[1].to_bits(),
                        params.sky[2].to_bits(),
                    ],
                    hell_glow: params.hell_glow.to_bits(),
                    light_params: params.identity(),
                    light_mode: *light_mode as u32,
                    anim: if style.perturb > 0.0 {
                        anim_time.to_bits()
                    } else {
                        // `TileDrawing.GetWindCycle`: the sway is zero below
                        // |wind| 0.08 (`Utils.GetLerpValue(0.08, 0.18, ...)`),
                        // so a calm world keeps the world range cacheable; a
                        // windy one animates the sway tiles every frame.
                        liquid_anim
                            ^ (if wind.sway_enabled && wind.speed_current.abs() > 0.08 {
                                (grass_wind_counter.to_bits() as u32)
                                    ^ (tree_wind_counter.to_bits() as u32)
                                    ^ (sunflower_wind_counter.to_bits() as u32)
                            } else {
                                0
                            })
                    },
                    player_key,
                    npc_key,
                    items_key: item::render_key(items, ((anim_time * 8.0) as u32) & 15),
                    hand_key: {
                        let held = inventory
                            .get(*selected)
                            .filter(|s| s.count > 0)
                            .map_or(0, |s| s.kind as u32);
                        let phase = if *swing > 0.0 {
                            (((1.0 - *swing / self.swing_max.max(f32::EPSILON)) * 40.0) as u32)
                                .min(40)
                        } else {
                            0
                        };
                        held | (phase << 16)
                    },
                    light_generation: light.generation(),
                    waterfall: waterfall.anim_key(),
                    net_key: {
                        // FNV-1a over the peers' smoothed state and the NPC
                        // smoothing accumulators (`npc_key` covers the base
                        // positions; the offsets decay every frame without
                        // moving the base).
                        let mut h: u32 = 0xcbf2_9ce4;
                        let mut mix = |h: &mut u32, v: u32| {
                            for b in v.to_le_bytes() {
                                *h = (*h ^ b as u32).wrapping_mul(0x0100_0193);
                            }
                        };
                        for rp in remote_players.values() {
                            mix(&mut h, rp.id);
                            mix(&mut h, (rp.x + rp.off_x).to_bits());
                            mix(&mut h, (rp.y + rp.off_y).to_bits());
                            mix(&mut h, rp.leg_frame);
                            mix(&mut h, rp.body_frame);
                            mix(&mut h, rp.hp as u32);
                        }
                        for n in npcs.iter() {
                            mix(&mut h, n.net_offset.x.to_bits());
                            mix(&mut h, n.net_offset.y.to_bits());
                        }
                        h
                    },
                    world_revision: world.revision,
                };
                let reuse_world = !crate::env::no_cache() && *world_key == Some(key);
                if reuse_world {
                    renderer.begin_frame_keep_world();
                    tile_stats.draw_ms = 0.0;
                    tile_stats.light_ms = 0.0;
                    tile_stats.world_reused = true;
                } else {
                    renderer.begin_frame();
                }
                // The parallax sky follows the camera continuously; it lives in the
                // always-rebuilt back range so the world range can stay cached.
                // `Main.UpdateAtmosphereTransparencyToSkyColor` runs per draw with
                // `screenPosition.Y + screenHeight / 2` (`Main.cs:61983`) — the
                // backbuffer-mapped view's top edge plus half the backbuffer. The
                // port's `draw_cam` is that view's centre and it spans
                // `screen / zoom` world pixels, so the equivalent input is
                // `cam.y - screen.y / (2 * zoom) + screen.y / 2`.
                self.atmo = tr_core::sky::atmosphere_transparency(
                    draw_cam.y - screen.y / (2.0 * world_zoom) + screen.y / 2.0,
                    world.world_surface,
                    world.width,
                );
                draw_sky(
                    renderer,
                    draw_cam,
                    bg_zoom,
                    screen,
                    world,
                    clouds,
                    sky,
                    stars,
                    surface_px,
                    is_day,
                    clock_time,
                    anim_time,
                    sky_uv,
                    sky_tint,
                    rain.cloud_bg(),
                    settings.background,
                    cave_parallax(settings.parallax_scroll),
                    bg_alpha_front,
                    bg_alpha_far,
                    light,
                    hell_black_box_bottom,
                    &self.ambience.entities,
                    lookups,
                    sky_tint,
                    self.atmo,
                );
                if !reuse_world {
                    tile_stats = draw_tiles(
                        renderer,
                        world,
                        lookups,
                        atlas.as_ref(),
                        draw_cam,
                        world_zoom,
                        screen,
                        &params,
                        light,
                        scratch,
                        &style,
                        anim_time,
                        waterfall,
                        render_player.as_ref(),
                        player_assets,
                        // `DrawPlayer_27_HeldItem`'s gate
                        // (`PlayerDrawLayers.cs:3084-3086`): the item rides the
                        // player layers while it is being used (`flag =
                        // itemAnimation > 0 && useStyle != 0`) OR when its
                        // `holdStyle` keeps it in the hand at rest (`flag2 =
                        // holdStyle != 0`) — torches.
                        match render_player.as_ref() {
                            Some(_)
                                if *swing > 0.0
                                    || inventory.get(*selected).is_some_and(|s| {
                                        s.count > 0 && item::hold_style(s.kind) != 0
                                    }) =>
                            {
                                inventory
                                    .get(*selected)
                                    .filter(|slot| slot.count > 0)
                                    .map(|slot| {
                                        // The builders consume the remaining
                                        // use fraction (`itemAnimation /
                                        // itemAnimationMax`).
                                        (slot.kind, *swing / self.swing_max.max(f32::EPSILON))
                                    })
                            }
                            _ => None,
                        },
                        // `Player.armor[0..3]`'s item kinds — the armor draw
                        // input (head/body/legs, 0 = the empty slot).
                        [
                            self.equip
                                .armor
                                .first()
                                .map_or(0, |s| if s.count > 0 { s.kind } else { 0 }),
                            self.equip
                                .armor
                                .get(1)
                                .map_or(0, |s| if s.count > 0 { s.kind } else { 0 }),
                            self.equip
                                .armor
                                .get(2)
                                .map_or(0, |s| if s.count > 0 { s.kind } else { 0 }),
                        ],
                        npcs,
                        npc_assets,
                        items,
                        &remote_players.values().cloned().collect::<Vec<_>>(),
                        *edit_target,
                        liquid_alpha,
                        // `TrySparkling`'s draw thread: the frame count pins it.
                        (*frame_count as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ world.seed,
                        wind,
                        *grass_wind_counter,
                        *tree_wind_counter,
                        *sunflower_wind_counter,
                        &mut leaf_spawns,
                    );
                    *world_key = Some(key);
                }

                // `TileDrawing.EmitTreeLeaves`: the foliage pass's spawns become
                // leaf gores; `Gore_UpdateLeaf` ticks them and the draw rides the
                // world-effect range like the rain (world content — under the map
                // batches and the interface). `TR_NO_WIND` is the A/B switch for
                // the whole wind feature, spawns included.
                let wind_ab_on = !crate::env::no_wind();
                let drained: Vec<_> = leaf_spawns.drain(..).collect();
                for s in drained.into_iter().take_while(|_| wind_ab_on) {
                    if leaves.len() >= 400 {
                        break;
                    }
                    leaves.push(crate::render::foliage::LeafGore::from_spawn(
                        s,
                        &mut leaf_rng,
                    ));
                }
                crate::render::foliage::tick_leaves(leaves, world, wind.speed_current);

                rain.draw(renderer, lookups, light, draw_cam, world_zoom, screen);
                // `Main.DrawDust`'s 268 entries: the flying sand, the same
                // world-effect range as the rain (`Main.DrawRain`'s draw call
                // site — world content, under the map batches and the UI).
                sand.draw(renderer, lookups, light, draw_cam, world_zoom, screen);
                // The type-213 wheel sparks — `Main.DrawDust`'s generic
                // 213-draw (self-lit gray, `Dust.cs:2498-2501`), the same
                // world-content range as the rain/sand dust.
                minecart_sparks.draw(renderer, lookups, draw_cam, world_zoom, screen);
                crate::render::foliage::draw_leaves(
                    renderer, lookups, light, draw_cam, world_zoom, screen, leaves,
                );
                // `Main.DrawWires` (`Main.cs:57028-57030`): the wire overlay
                // rides the tile batch after the liquids, under the
                // over-wire entities (the port draws it before the
                // projectiles, matching `DrawCachedProjsOverWiresUI`).
                crate::render::wires::draw_wires(
                    renderer, world, light, &style, draw_cam, world_zoom, screen, anim_time,
                );
                for p in projectiles.slots().iter().filter(|p| p.active) {
                    // The yoyo family's string anchors on the player's
                    // centre (`Main.cs:27943-27945`'s mountedCenter +
                    // gfxOffY) — the frame's rendered player carries both.
                    let hand = render_player.as_ref().map(|pl| {
                        let (cx, cy) = pl.center();
                        (cx, cy + pl.gfx_off_y)
                    });
                    // `Main.GlobalTimeWrappedHourly` (`anim_time % 3600`, the
                    // same wrapped value the world pass keys on) — the Rainbow
                    // Rod's trail hue scrolls by it (`RainbowRodDrawer.cs:25`).
                    p.draw_with_hand(
                        renderer,
                        lookups,
                        light,
                        draw_cam,
                        world_zoom,
                        screen,
                        hand,
                        anim_time % 3600.0,
                    );
                }

                let boss = npcs
                    .iter()
                    .find(|n| {
                        (n.kind == NpcKind::KingSlime || n.kind == NpcKind::EyeOfCthulhu)
                            && n.hp > 0
                    })
                    .map(|n| (n.hp, n.max_hp));
                // `Main.cs:62147-62181`: while the fullscreen map is open `Main.Draw`
                // returns early after `DrawMap`, before `DrawInterface` (`62432`), so
                // the HUD (hotbar, life, inventory) is not drawn over the map.
                let fullscreen_map = settings.map_enabled && map.as_ref().is_some_and(|m| m.open);
                // The minimap branch draws **before** the HUD: the vanilla
                // interface passes are ordered 16 (map/minimap,
                // `Main.cs:44796`) < 25/27/30 (resources/inventory/hotbar,
                // `:44831/:44841/:44861`), so the hearts and the open inventory
                // paint over the frame. `Main.DrawMap`'s `mapStyle == 1`
                // branch (`Main.cs:55503-55547`): the three vanilla passes in
                // draw order — the `DrawTo` black backdrop (`:55546`, via
                // `map_back` so it lands under the tiles), the tile pass
                // (`MapRenderer.DrawMap`, `:55609`) and the frame foreground
                // batch (`:55768-55769`). `mapStyle == 2` (the overlay) draws
                // nothing (out of P4 scope, the whole-UI P4 plan's
                // Uncertainty 7).
                let view_center = [camera.x / TILE_PX, camera.y / TILE_PX];
                // `Main.MapScale` (`Main.cs:67297`, clamp `Main.cs:4494`): the
                // minimap anchor and the whole panel's render scale, through the
                // `Matrix.CreateScale(Main.MapScale)` batch transform
                // (`Main.cs:55309-55329`). The full map keeps
                // `Main.mapFullscreenScale` (`Main.cs:55390`, default 4f
                // `:66756`) and the zoom axis `Main.mapMinimapScale` is a
                // separate multiplier (`Main.cs:55515-55522`).
                let map_scale = settings.map_scale;
                if settings.map_enabled
                    && menu.is_none()
                    && front.is_none()
                    && !crate::env::no_hud()
                    && !fullscreen_map
                    // `Main.inFancyUI` (`IngameFancyUI.cs:64-72`): the
                    // bestiary state covers the interface — the minimap HUD
                    // does not draw under it.
                    && !self.bestiary_open
                    && self.map_style == 1
                {
                    if let Some(m) = map.as_ref() {
                        let pos = ui::minimap::draw_to_position(ui::minimap::minimap_origin_raw(
                            screen, map_scale,
                        ));
                        // `Main.mapMinimapAlpha = 1` (`Main.cs:66753`).
                        ui::minimap::draw_background(renderer, lookups, pos, 1.0, map_scale);
                        m.draw_minimap(renderer, world, screen, view_center, map_scale);
                        // The hover highlight is `MinimapFrame.Update`'s
                        // `buttonUnderMouse.IsHighlighted`
                        // (`MinimapFrame.cs:163`), resolved per frame in
                        // `handle_minimap_clicks` — like the source, where
                        // `DrawTo` runs `Update` before the foreground batch
                        // (`MinimapFrameManager.cs:45`, `Main.cs:55769`).
                        let s = ui::minimap::style_index(&settings.minimap_frame);
                        ui::minimap::draw_foreground(
                            renderer,
                            lookups,
                            s,
                            pos,
                            self.minimap_hover,
                            map_scale,
                        );
                    }
                }
                if let Some(front) = front.as_mut() {
                    // `Main.menuMode == 888` (`Main.cs:51059-51064`): the
                    // front-end draws over the world backdrop the frame
                    // already rendered — the draw site picks `front` first,
                    // then `menu` (Review Focus 1).
                    let mut d = FrontDraw {
                        renderer: &mut *renderer,
                        lookups: &*lookups,
                        screen,
                        players: &self.players,
                        worlds: &self.worlds,
                        active_difficulty,
                        player_assets: &*player_assets,
                        anim_time,
                        mouse: self.mouse_pos,
                        front_mouse_down: self.front_mouse_down,
                        vk_grid: self.vk_grid,
                        frame_count: *frame_count,
                    };
                    front.draw(&mut d);
                    // The port's not-ported notice for the front screens
                    // (ours, like `Menu.hint` — not in the original).
                    if let Some(hint) = front_hint {
                        let w = hud_text_width(renderer, lookups, hint, 0.8);
                        hud_text(
                            renderer,
                            lookups,
                            (screen.x - w) * 0.5,
                            screen.y - 64.0,
                            0.8,
                            [1.0, 0.8, 0.45, 1.0],
                            hint,
                        );
                    }
                } else if menu.is_none() && !crate::env::no_hud() && !fullscreen_map {
                    let craft_available = if self.crafting_open {
                        // `Recipe.UpdateRecipeList` (`Recipe.cs:230-259`): the
                        // owned counts fold the inventory and the in-range
                        // chests; the environment reads the adjacency scan
                        // and the zone flags.
                        let (adj, in_range) = player
                            .as_ref()
                            .map(|p| {
                                (
                                    craft::scan_adjacent(world, p),
                                    craft::chests_in_range(
                                        world,
                                        p,
                                        &self.world_chests,
                                        settings.craft_from_nearby_chests,
                                    ),
                                )
                            })
                            .unwrap_or_else(|| {
                                (
                                    craft::Adjacent {
                                        tiles: Vec::new(),
                                        water: false,
                                        honey: false,
                                        lava: false,
                                        alchemy_table: false,
                                    },
                                    Vec::new(),
                                )
                            });
                        let zones = craft::Zones {
                            snow: self.scene_metrics.zone_snow,
                            graveyard: self.scene_metrics.zone_graveyard,
                        };
                        craft::available(inventory, &self.world_chests, &in_range, &adj, &zones)
                    } else {
                        Vec::new()
                    };
                    // The unified open-container state (`Player.chest`,
                    // `Player.cs:2876`/`:2878`/:2880) — the panel reads the
                    // player's fields now that one field covers every kind.
                    let panel_chest = player
                        .as_ref()
                        .map_or(crate::bank::CHEST_CLOSED, |p| p.chest);
                    let (panel_chest_x, panel_chest_y) =
                        player.as_ref().map_or((0, 0), |p| (p.chest_x, p.chest_y));
                    ui::draw_hud(
                        renderer,
                        *camera,
                        *zoom,
                        screen,
                        lookups,
                        inventory,
                        &self.favorite_slots,
                        &self.equip,
                        self.equip_page,
                        *selected,
                        &self.hotbar_scales,
                        &npcs[..],
                        player.as_ref(),
                        boss,
                        // `BigProgressBarSystem.ShowText` (`ShowBossBarHealthText`,
                        // `BigProgressBarSystem.cs:231`/`:236`): the gate on the
                        // boss bar's numbers (`BigProgressBarHelper.cs:53-55`).
                        settings.boss_bar_text,
                        self.inventory_open,
                        self.crafting_open,
                        &self.mouse_item,
                        self.mouse_pos,
                        &self.cursor,
                        &craft_available,
                        ui::UiPulse {
                            // `Main.mouseTextColor / 255` (`Main.cs:20355`).
                            text: self.mouse_text_color as f32 / 255.0,
                            cursor_scale: self.cursor.scale(),
                        },
                        m_h,
                        self.map_style,
                        self.bestiary_open,
                        self.bestiary_selected,
                        &self.bestiary.kills,
                        &mut self.chat,
                        &self.emotes,
                        // BUG-11: the in-game options window's state
                        // (`Main.ingameOptionsWindow`) — the Settings button
                        // and the panel draw over the open inventory.
                        Some((settings, &self.ingame_options)),
                        self.shop_npc.map(|_| {
                            (
                                self.shop_stock.as_slice(),
                                self.shop_mood,
                                self.discount_available,
                            )
                        }),
                        // `ChestUI.Draw`'s state (`Main.player[myPlayer].chest`
                        // + the statics) — the plan-amendment insertion
                        // (#10): the disjoint `App` fields the panel reads
                        // and animates per frame. The unified open state is
                        // `Player.chest` (M3-CHEST-INT) — `panel_chest`
                        // carries the full index space; the panel itself is
                        // the world-chest surface (the `>= 0` gate in
                        // `ui::chest::draw`, the STORE-1 two-panel shape).
                        ui::chest::PanelState {
                            chest: panel_chest,
                            origin: (panel_chest_x, panel_chest_y),
                            crafting_open: self.crafting_open,
                            tile: world
                                .tile(panel_chest_x, panel_chest_y)
                                .filter(|t| t.active())
                                .map(|t| (t.kind, t.frame_x)),
                            slots: (panel_chest > -1)
                                .then(|| {
                                    crate::chest::chest_slots(
                                        &self.world_chests,
                                        panel_chest as usize,
                                    )
                                })
                                .flatten(),
                            editing: chest_editing,
                            name: chest_stored_name,
                            max_items: chest_max_items,
                            edit_text: &chest_edit_text,
                            blinker: &mut self.text_blinker,
                            scroll_row: &mut self.chest_scroll_row,
                            button_scales: &mut self.chest_button_scales,
                            button_hovered: &mut self.chest_button_hovered,
                            audio: &mut self.audio,
                        },
                        self.in_reforge_menu.then(|| {
                            let cost = self
                                .reforge_slot
                                .kind
                                .gt(&0)
                                .then(|| {
                                    crate::shop::reforge_cost(
                                        crate::prefix::prefixed_stats(
                                            self.reforge_slot.kind,
                                            self.reforge_slot.prefix,
                                        )
                                        .map(|s| s.value)
                                        .unwrap_or(0),
                                        self.reforge_slot.count.max(1),
                                        self.shop_mood,
                                    )
                                })
                                .unwrap_or(0);
                            (&self.reforge_slot, cost, self.reforge_cooldown)
                        }),
                        player.as_ref().and_then(|p| {
                            crate::bank::bank_for_chest(p.chest)
                                .map(|bi| (&p.banks[bi][..], p.chest))
                        }),
                        // `Main.DrawPVPIcons`'s state (`Main.cs:42654`): the
                        // net-mode / team-based-spawns gates, the local
                        // player's `hostile` + `team` and `Main.teamCooldown`.
                        &pvp_icon_state,
                        // `Player.hideVisibleAccessory` (`Player.cs:1708`): the
                        // accessory visible/hidden ticks in `DrawInventory`
                        // (`Main.cs:41425-41460`).
                        &hide_visible_accessory,
                        // The buff bar (`Main.DrawInterface_Resources_Buffs`).
                        &mut self.buff_alpha,
                        &mut self.buff_removals,
                        right_release,
                        vampire_seed,
                    );
                    // `CoinSlot.UpdateSlotAnims()` (`Main.cs:44936`, S1-H-9):
                    // the coin cells' animation clocks step per frame after
                    // the interface layers draw.
                    self.coin_slot.update_slot_anims();
                    // `Main.DrawDeathText`'s overlay (`Main.cs:45324-45341`):
                    // the death broadcast + the softcore "Dropped coins"
                    // line while the respawn timer runs.
                    if let Some(p) = player.as_ref() {
                        if p.hp <= 0 {
                            // `Main._framesWithRespawnSkipAvailable`
                            // (`Main.cs:45348-45376`): the counter the C#
                            // ticks inside `DrawDeathText` — up while the skip
                            // is allowed, zeroed otherwise.
                            if self.respawn_skip_available {
                                self.respawn_skip_frames =
                                    self.respawn_skip_frames.saturating_add(1);
                            } else {
                                self.respawn_skip_frames = 0;
                            }
                            ui::draw_death_screen(
                                renderer,
                                lookups,
                                screen,
                                &self.death_text,
                                self.lost_coins,
                                &self.lost_coin_string,
                                p.respawn_timer,
                                self.respawn_skip_frames,
                            );
                        }
                    }
                    // `Main.DrawInvasionProgress` (`Main.cs:47696-47840`):
                    // the bar draws while the fade alpha holds (the
                    // display-timer decay rides the world tick).
                    ui::draw_invasion_progress(
                        renderer,
                        lookups,
                        screen,
                        self.invasion_progress,
                        self.invasion_progress_max,
                        self.invasion_progress_icon,
                        self.invasion_progress_wave,
                        self.invasion_progress_alpha,
                    );
                } else if let Some(menu) = menu.as_mut() {
                    menu.draw(renderer, lookups, screen, dt);
                }
                // `UIAchievementsMenu` (`Main.cs:48931`'s menuMode 888 +
                // `MenuUI.SetState(Main.AchievementsMenu)`): the state draws
                // over whatever the frame rendered — the menu's world
                // backdrop here, the world itself in the fancy-UI arm. Its
                // `OnActivate` picks the container's Top by `Main.gameMenu`.
                if self.achievements_open {
                    ui::achievements::draw_screen(
                        renderer,
                        lookups,
                        screen,
                        &self.achievements,
                        self.achievements_scroll,
                        &self.achievements_categories,
                        self.mouse_pos,
                        menu.is_some(),
                    );
                }
                // Task 6b: upload the revealed tiles into the world-sized map texture
                // and clamp/recentre the fullscreen map. Vanilla centres the recentre
                // and the minimap on the view centre (`Main.cs:55420-55421`,
                // `55537-55538`): `(screenPosition + screenSize/2)/16` — `view_center`
                // is computed above, where the minimap branch consumes it.
                if settings.map_enabled {
                    if let Some(m) = map.as_mut() {
                        m.prepare(renderer, gpu, world, screen, view_center);
                    }
                }
                if let Some(map) = map.as_ref() {
                    if !crate::env::no_hud()
                        && menu.is_none()
                        && front.is_none()
                        && settings.map_enabled
                    {
                        if map.open {
                            // `Main.cs:55483`: the biome `MapBG` backdrop is
                            // drawn before the wooden frame, so it goes into
                            // `map_back` first (`draw_full` pushes the frame).
                            // `Main.player[myPlayer]` always exists in vanilla;
                            // with no player we fall back to the view centre.
                            let player_tile = player
                                .as_ref()
                                .map(|p| {
                                    let (cx, cy) = p.center();
                                    ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32)
                                })
                                .unwrap_or((
                                    (camera.x / TILE_PX) as i32,
                                    (camera.y / TILE_PX) as i32,
                                ));
                            if let Some(bg) = map::map_bg_index(world, *camera, screen, player_tile)
                            {
                                let sky = [sky_tint[0], sky_tint[1], sky_tint[2], 1.0];
                                map::draw_map_background(renderer, bg, screen, sky);
                            }
                            // `Main.cs:55996`: the icon overlay layers ride the
                            // fullscreen map's offset/scale — the layer walk in
                            // registration order, the hover text collected.
                            let (map_offset, map_draw_scale) =
                                map.draw_full(renderer, lookups, world, screen);
                            let digtoise: Vec<[f32; 2]> = Vec::new();
                            let hover = crate::map_layers::map_layers_draw(
                                renderer,
                                lookups,
                                &crate::map_layers::MapIconInputs {
                                    world,
                                    player: player.as_ref(),
                                    item_drops: &self.items,
                                    digtoise: &digtoise,
                                    pings: crate::map_layers::map_pings(map),
                                    pylons: 0,
                                    team_based_spawns,
                                    mouse: self.mouse_pos,
                                },
                                [0.0, 0.0],
                                map_offset,
                                None,
                                map.scale,
                                map_draw_scale,
                                1.0,
                            );
                            if let Some(t) = hover {
                                crate::ui::text::hud_text(
                                    renderer,
                                    lookups,
                                    self.mouse_pos.x + 12.0,
                                    self.mouse_pos.y + 12.0,
                                    0.85,
                                    [1.0; 4],
                                    &t,
                                );
                            }
                            // `Main.cs:55931-55954`: the local player's head icon,
                            // drawn after the map tiles (`Main.cs:55609`).
                            if let Some(p) = player.as_ref() {
                                let (cx, cy) = p.center();
                                // `Main.cs:55938`: `position.Y + gfxOffY + height/2`.
                                let center_px = [cx, cy + p.gfx_off_y];
                                let icon_pos =
                                    map::head_icon_pos(center_px, map.pos, map.scale, screen);
                                // `Main.GetPlayerHeadBordersColor` (`Main.cs:56616-56618`)
                                // → `Main.teamColor[0]` = White (singleplayer).
                                draw_player_map_icon(
                                    renderer,
                                    player_assets,
                                    p,
                                    icon_pos,
                                    self.settings.applied_ui_scale(screen.x, screen.y),
                                    [1.0; 4],
                                );
                            }
                        }
                        // The `else if self.map_style == 1` minimap branch
                        // (`Main.cs:55503-55547`) draws above `ui::draw_hud`
                        // now — vanilla renders interface layer 16 (the
                        // minimap, `Main.cs:44796`) before layers 25/27/30
                        // (`:44831/:44841/:44861`), so the hearts and the
                        // inventory paint over the frame.
                    }
                }

                if *debug {
                    draw_debug(
                        renderer,
                        screen,
                        tile_stats,
                        frame_history,
                        *camera,
                        *zoom,
                        *last_stats,
                        world,
                        *atlas_stats,
                        gpu,
                        dt,
                        params.sky,
                        debug_cache,
                        style.mode.label(),
                        player.as_ref(),
                        npcs.len(),
                        liquids.active_len(),
                        rain.visual_alpha(),
                        rain.drop_count(),
                        rain.snow_count(),
                    );
                }

                // The debug cheat-sheet panel (the self-contained
                // debug_panel.rs) — independent of F1.
                if self.debug_panel.open {
                    crate::debug_panel::draw(
                        renderer,
                        lookups,
                        screen,
                        &self.debug_panel,
                        npcs,
                        world,
                        self.stepper.paused,
                    );
                }

                // `WorldUpdateStepper.DrawHUD` (`Main.cs:62409`, the
                // UIScale batch after DrawFPS; `WorldUpdateStepper.cs:51-57`):
                // the `Updates Since Paused` border string, bottom-right at
                // `ScreenSize - (5, 22)` with anchorx 1 — the right edge sits
                // at `screen.x - 5`. The guard is the stepper's own `Paused`
                // (`:53-54`), so it draws while stepper-paused regardless of
                // `game_paused`; `Color.White` at scale 1.
                if let Some(text) = self.stepper.hud_text(self.game_update_count) {
                    let w = hud_text_width(renderer, lookups, &text, 1.0);
                    border_string(
                        renderer,
                        lookups,
                        screen.x - 5.0 - w,
                        screen.y - 22.0,
                        [1.0, 1.0, 1.0, 1.0],
                        &text,
                    );
                }

                let frame = match gpu.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(f)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                    other => {
                        log::debug!("surface acquire skipped: {other:?}");
                        return;
                    }
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                renderer.aberration = style.aberration_px;
                // M1.4-BG-18 phase 3 (`TR_POST=1`): the scene renders into the
                // offscreen post target, then the `ApplyVisuals` driver's
                // winner uniform (`filter_driver.output`, computed in
                // `update_scene`) selects the filter pass. `None` = no filter
                // — the NEUTRAL/passthrough pair is the pixel-identity pass
                // (DW-2.1).
                if let Some(post) = post.as_mut() {
                    let scene = post.scene_view(gpu);
                    // The vanilla order puts the filters between the world draw
                    // and `DrawMap`/`DrawInterface` (`Main.DoDraw`): the world
                    // and the world-effect ranges render into the post target,
                    // the filter chain lands on the screen, then the map
                    // batches and the interface draw over the filtered frame.
                    let now_secs = now.elapsed().as_secs_f32();
                    stats = renderer.render_world_fx(
                        gpu,
                        scene,
                        draw_cam.to_array(),
                        world_zoom,
                        now_secs,
                        wgpu::Color {
                            r: 0.045,
                            g: 0.055,
                            b: 0.085,
                            a: 1.0,
                        },
                    );
                    let noise = noise_tex.as_ref();
                    // Phase 4: the water mask exists only while the
                    // WaterDistortion filter is visible AND the liquid pass
                    // drew water this frame (plan: "the second target only
                    // while WaterDistortion is active"; with no water the pass
                    // is the identity, so it and the target are skipped).
                    let water_in_view =
                        filter_driver.water_wanted() && !scratch.water_rects.is_empty();
                    if let Some(mask) = water_mask.as_mut() {
                        if water_in_view {
                            mask.render(
                                &gpu.device,
                                &gpu.queue,
                                (gpu.config.width, gpu.config.height),
                                draw_cam.to_array(),
                                world_zoom,
                                &scratch.water_rects,
                            );
                        } else {
                            mask.release();
                        }
                    }
                    let mut chain = filter_driver.chain(water_in_view);
                    if chain.is_empty() {
                        // No filter: the NEUTRAL passthrough (DW-2.1 identity).
                        chain.push(tr_render::FilterPass {
                            params: tr_render::FilterParams::NEUTRAL,
                            pass: tr_render::PassId::PASSTHROUGH,
                        });
                    }
                    post.apply_filters(
                        gpu,
                        &view,
                        &chain,
                        tr_render::FilterImages {
                            noise,
                            water_mask: water_mask.as_ref().and_then(|m| m.texture()),
                        },
                    );
                    let ui_stats =
                        renderer.render_map_ui(gpu, &view, draw_cam.to_array(), *zoom, now_secs);
                    stats.instances += ui_stats.instances;
                    stats.draw_calls += ui_stats.draw_calls;
                    stats.uploaded_bytes += ui_stats.uploaded_bytes;
                } else {
                    stats = renderer.render(
                        gpu,
                        &view,
                        draw_cam.to_array(),
                        world_zoom,
                        now.elapsed().as_secs_f32(),
                        wgpu::Color {
                            r: 0.045,
                            g: 0.055,
                            b: 0.085,
                            a: 1.0,
                        },
                    );
                }
                if let Some(path) = shot_path.as_ref() {
                    if *frame_count >= *shot_at {
                        log::info!(
                            "shot camera px {:.2},{:.2} (tile {:.3},{:.3}) zoom {:.3}",
                            camera.x,
                            camera.y,
                            camera.x / TILE_PX,
                            camera.y / TILE_PX,
                            *zoom
                        );
                        capture_frame(gpu, &frame.texture, path);
                        gpu.queue.present(frame);
                        std::process::exit(0);
                    }
                }
                gpu.queue.present(frame);
            }
        }

        self.tile_stats = tile_stats;
        if self.stats_on {
            self.stats_acc.add(&tile_stats);
            if self.stats_acc.frames >= 120 {
                self.stats_acc.log_and_reset();
            }
        }
        self.last_stats = stats;
        let ms = dt * 1000.0;
        self.frame_history.push_back(ms);
        if self.frame_history.len() > 240 {
            self.frame_history.pop_front();
        }

        if let Some(target) = self.bench_frames {
            self.bench_samples.push(ms);
            if self.bench_samples.len() as u32 >= target {
                self.finish_bench();
            }
        }
    }
}

impl App {
    pub(crate) fn finish_bench(&mut self) {
        let mut s = self.bench_samples.clone();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = s.len();
        let avg = s.iter().sum::<f32>() / n as f32;
        let pct = |p: f32| s[((n as f32 - 1.0) * p) as usize];
        println!(
            "BENCH frames={n} avg={avg:.3}ms ({:.1} fps) p50={:.3} p95={:.3} p99={:.3} max={:.3}",
            if avg > 0.0 { 1000.0 / avg } else { 0.0 },
            pct(0.50),
            pct(0.95),
            pct(0.99),
            s[n - 1]
        );
        println!(
            "BENCH draws={} instances={} visible_world={} zoom={:.2}",
            self.last_stats.draw_calls,
            self.last_stats.instances,
            self.world.tiles.len(),
            self.zoom
        );
        self.bench_frames = None;
        std::process::exit(0);
    }
}

/// `S1-B-1` order pin: the world branch records each phase name it runs, so a
/// headless `App::frame` can *prove* the `Main.DoUpdateInWorld_Inner`
/// (`Main.cs:17745-17822`) sequence instead of a reviewer only reading it.
///
/// Thread-local because the test binary runs cases on parallel threads, and
/// `cfg(test)`-only so the release build pays nothing.
#[cfg(test)]
pub(crate) mod phase_trace {
    use std::cell::RefCell;

    thread_local! {
        static PHASES: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    }

    pub(crate) fn record(name: &'static str) {
        PHASES.with(|p| p.borrow_mut().push(name));
    }

    /// Drains the current thread's trace (one frame's worth).
    pub(crate) fn take() -> Vec<&'static str> {
        PHASES.with(|p| std::mem::take(&mut *p.borrow_mut()))
    }
}

#[cfg(test)]
mod tests {
    use super::phase_trace;
    use crate::app::App;
    use crate::app::flow::FrontScreen;
    use glam::Vec2;
    use winit::keyboard::KeyCode;

    /// The world-branch sequence, transcribed from the C#:
    ///
    /// - players → spawn NPCs → NPCs (`Main.cs:17757`, `:17878`, `:17898`)
    /// - gores (unported) → **projectiles** → **items** → dust → leashed
    ///   (unported) → floating text (unported) → **time**
    ///   (`:17971`, `:17976`, `:18004`, `:18047`, `:18083`)
    /// - `WorldGen.UpdateWorld` (`WorldGen.cs:71495-71630`): wiring → tile
    ///   entities → liquid → growth (`:18105`)
    /// - then the port's environment cluster (weather / rain / scene state /
    ///   windy day / ambience), the tile-edit pass, the net pass and the map.
    ///
    /// Every phase name is emitted once per in-world frame.
    const WORLD_PHASES: [&str; 19] = [
        "players",
        "npcs",
        "projectiles",
        "items",
        "dust",
        "time",
        "wiring",
        "tile_entities",
        "liquids",
        "growth",
        "waterfalls",
        "weather",
        "rain",
        "windy_day",
        "ambience",
        "editing",
        "net",
        "map",
        // S1-B-3 Phase 2 (decision #5): `SceneState.Update` sits in
        // `DoUpdateInWorld_Inner`'s step-gated tail (`Main.cs:17816`), after
        // the camera walks — it stepped from the post-rain slot down here.
        "scene_state",
    ];

    /// An in-world headless `App`: the flow fixture plus a spawned player, so
    /// `frame()` takes the `menu.is_none() && front.is_none()` branch.
    fn in_world_app() -> App {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app
    }

    #[test]
    fn the_world_branch_runs_in_the_csharp_update_order() {
        let mut app = in_world_app();
        app.frame();
        assert_eq!(
            phase_trace::take(),
            WORLD_PHASES,
            "the world branch must follow Main.cs:17745-17822 (projectiles before \
             items, time after dust, WorldGen after time)"
        );
        // A second frame records the same sequence: the trace is per frame.
        app.frame();
        assert_eq!(phase_trace::take(), WORLD_PHASES);
    }

    /// S1-B-2, landed with the clock move: the C# returns on `Main.gameMenu`
    /// before `DoUpdateInWorld` (`Main.cs:17166-17175`), so `UpdateTime` never
    /// runs while a front-end screen owns the frame.
    #[test]
    fn the_clock_only_advances_inside_the_world_branch() {
        let mut app = in_world_app();
        let before = app.world.time;
        app.frame();
        assert!(
            app.world.time > before,
            "the in-world clock advances ({before} -> {})",
            app.world.time
        );

        // A front-end frame: no world branch, no clock.
        phase_trace::take(); // drain the in-world frame's trace
        app.front = Some(FrontScreen::CharSelect);
        let frozen = app.world.time;
        for _ in 0..3 {
            app.frame();
        }
        assert_eq!(
            app.world.time, frozen,
            "a front-end screen must not age the world clock"
        );
        assert!(
            phase_trace::take().is_empty(),
            "the world branch did not run on the front end"
        );
    }

    // ---- S1-B-3: the pause gates (Main.cs:17128-17407, 17717-17746) ----

    /// A world tick helper for the freeze tests: `frame()` derives `dt` from
    /// the wall clock, and back-to-back test frames would give the fixed
    /// steppers nothing to accumulate — parking `last_frame` 100 ms back makes
    /// each frame run ~6 fixed steps (the clamp at the top of `frame()`).
    fn step_frame(app: &mut App) {
        app.last_frame = std::time::Instant::now() - std::time::Duration::from_millis(100);
        app.frame();
    }

    /// DW-1.1 — `CanPauseGame`'s truth table (`Main.cs:17315-17327`), the
    /// port-expressible arms: the options window and the achievements/bestiary
    /// overlays pause unconditionally; the `autoPause` arm adds the inventory
    /// (and re-adds the overlays); nothing open never pauses, even with
    /// autoPause on (the boundary). netMode is always 0 in the port, so the
    /// C#'s `netMode == 0` wrapper is the whole table.
    #[test]
    fn s1_b_3_can_pause_game_matches_the_csharp_table() {
        let mut app = in_world_app();
        // Nothing open → false, with and without autoPause.
        assert!(!app.can_pause_game(), "nothing open never pauses");
        app.settings.auto_pause = true;
        assert!(
            !app.can_pause_game(),
            "autoPause with every window closed stays unpaused (the boundary)"
        );
        app.settings.auto_pause = false;
        // The unconditional arms (`flag |= ingameOptionsWindow | (InGameUI…)`).
        app.ingame_options.open = true;
        assert!(app.can_pause_game(), "the in-game options window pauses");
        app.ingame_options.open = false;
        app.achievements_open = true;
        assert!(app.can_pause_game(), "the achievements overlay pauses");
        app.achievements_open = false;
        app.bestiary_open = true;
        // The bestiary is an `InGameUI` state (`Main.cs:42459`) but its
        // `CurrentState` is neither `ManageControlsMenu` nor `AchievementsMenu`
        // (`Main.cs:17320`), so without autoPause it never pauses — only the
        // autoPause arm's bare `InGameUI.IsVisible` catches it (`:17323`).
        assert!(
            !app.can_pause_game(),
            "the bestiary overlay alone never pauses without autoPause"
        );
        app.bestiary_open = false;
        // The autoPause arm (`flag |= playerInventory | … | InGameUI.IsVisible`).
        app.settings.auto_pause = true;
        app.inventory_open = true;
        assert!(app.can_pause_game(), "autoPause + the inventory pauses");
        app.inventory_open = false;
        app.achievements_open = true;
        assert!(
            app.can_pause_game(),
            "autoPause + the achievements overlay pauses"
        );
        app.achievements_open = false;
        app.bestiary_open = true;
        assert!(
            app.can_pause_game(),
            "autoPause + the bestiary overlay pauses"
        );
        // autoPause off with only the inventory open stays unpaused.
        app.settings.auto_pause = false;
        app.bestiary_open = false;
        app.inventory_open = true;
        assert!(
            !app.can_pause_game(),
            "the inventory alone never pauses without autoPause"
        );
    }

    /// The plan's dirty check: pause with the achievements overlay open, close
    /// it (the `UIAchievementsMenu` Escape arm, `app/loop.rs`), and the world
    /// unpauses on the next frame — the C#'s `CanPauseGame` re-evaluation
    /// (`Main.cs:17194-17201`) picks the flag change up with no extra state.
    #[test]
    fn s1_b_3_closing_the_pausing_overlay_unpauses_the_next_frame() {
        let mut app = in_world_app();
        app.achievements_open = true;
        app.frame();
        assert!(app.game_paused, "the achievements overlay pauses");
        assert_eq!(phase_trace::take(), ["waterfalls", "windy_day"]);
        // The overlay's close route (`loop.rs`'s Escape arm).
        app.achievements_open = false;
        app.frame();
        assert!(!app.game_paused, "the next frame unpaused");
        assert_eq!(phase_trace::take(), WORLD_PHASES);
    }

    /// DW-1.2 (trace half) — a paused frame runs only the pre-gate cluster:
    /// `DoUpdate_AnimateWaterfalls` (`Main.cs:17146`) and
    /// `UpdateWindyDayState` (`:17182`); every gated phase is skipped and
    /// `gamePaused` reads back true. Closing the pause source resumes the
    /// whole pinned branch on the next frame (the C# `:17197-17201` else arm).
    #[test]
    fn s1_b_3_a_paused_frame_runs_only_the_pregate_phases() {
        let mut app = in_world_app();
        assert!(
            app.should_update_entities(),
            "the sync load is always Ready (Main.cs:17717-17726)"
        );
        app.ingame_options.open = true;
        app.frame();
        assert!(app.game_paused, "the gate wrote gamePaused (Main.cs:17197)");
        assert_eq!(
            phase_trace::take(),
            ["waterfalls", "windy_day"],
            "a paused frame keeps only the pre-gate Animate cluster alive"
        );
        app.ingame_options.open = false;
        app.frame();
        assert!(!app.game_paused, "the gate's else arm clears gamePaused");
        assert_eq!(
            phase_trace::take(),
            WORLD_PHASES,
            "unpausing resumes the full world branch, order unchanged"
        );
    }

    /// DW-1.2 (state half) — 20 paused frames (≈120 fixed world steps) freeze
    /// the NPC, the clock, the rain countdown, the falling water and the step
    /// counter; the same frames unpaused move all of them.
    #[test]
    fn s1_b_3_a_paused_world_freezes_entities_clock_liquids_and_weather() {
        let mut app = in_world_app();
        // A 220-tall ground world: `world_surface = height * 0.25` = 55 opens
        // the rain machine's `worldSurface > 50` gate (`Main.cs:2788`'s
        // `isThereAWorldSurface`).
        let mut w = tr_core::World::new("t".into(), 220, 220);
        for x in 0..220 {
            for y in 120..220 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        // `spawn_slime` places on `surface_of` — the cached column scan.
        w.compute_column_surface();
        app.world = w;
        app.npcs = vec![crate::npc::Npc::spawn_slime(&app.world, 20).expect("the slime spawns")];
        // Free-falling water in open air (the dirt starts at y=120): the tile
        // carries the volume, `add_water` wakes the cell into the active list
        // (`Liquid.cs`'s `LiquidBuffer` shape) — the sim is rebuilt for the
        // swapped world's dimensions first.
        let cell = app.world.index(30, 100);
        app.world.tiles[cell].liquid = 255;
        app.liquids = tr_core::liquid::LiquidSystem::new(&app.world);
        app.liquids.add_water(&app.world, 30, 100);
        // The rain event state (`Rain::restore_saved` — `SetOngoingToTemps`'s
        // triple): raining with a live countdown.
        app.rain.restore_saved(true, 4000, 0.8);
        let (npc_x, npc_y) = (app.npcs[0].x, app.npcs[0].y);
        let time0 = app.world.time;
        let rain_time0 = app.rain.rain_time();
        let liquid0 = app.world.tile(30, 100).map(|t| t.liquid).unwrap_or(0);
        let count0 = app.game_update_count;
        // CanPauseGame through the options window.
        app.ingame_options.open = true;
        for _ in 0..20 {
            step_frame(&mut app);
        }
        assert!(app.game_paused);
        assert_eq!(app.npcs[0].x, npc_x, "the NPC never moved while paused");
        assert_eq!(app.npcs[0].y, npc_y);
        assert_eq!(app.world.time, time0, "the clock froze");
        assert_eq!(
            app.rain.rain_time(),
            rain_time0,
            "the rain countdown froze (UpdateWeather/UpdateTime are post-gate)"
        );
        assert_eq!(
            app.world.tile(30, 100).map(|t| t.liquid).unwrap_or(0),
            liquid0,
            "the falling water did not settle while paused"
        );
        assert!(
            app.liquids.active_len() > 0,
            "the water cell stayed woken, untouched"
        );
        assert_eq!(
            app.game_update_count, count0,
            "GameUpdateCount advanced per executed step, so it froze (Main.cs:17746)"
        );
        // One more paused frame, drained alone, for the per-frame trace check.
        phase_trace::take();
        app.frame();
        assert_eq!(phase_trace::take(), ["waterfalls", "windy_day"]);
        // The positive control: the same frames unpaused move everything.
        app.ingame_options.open = false;
        for _ in 0..20 {
            step_frame(&mut app);
        }
        assert!(app.world.time > time0, "the clock ran again");
        assert!(
            app.rain.rain_time() < rain_time0,
            "the rain countdown drained again"
        );
        assert_ne!(
            app.world.tile(30, 100).map(|t| t.liquid).unwrap_or(0),
            liquid0,
            "the falling water settled"
        );
        assert!(
            app.game_update_count > count0,
            "one increment per executed world step"
        );
        assert!(
            app.npcs[0].x != npc_x || app.npcs[0].y != npc_y,
            "the slime moved again"
        );
        phase_trace::take();
        app.frame();
        assert_eq!(phase_trace::take(), WORLD_PHASES);
    }

    /// DW-1.3 — the UI cluster stays live while paused. The C# toggle runs
    /// inside `DoUpdate_WhilePaused` (the `controlInv` edge, `Main.cs:17335`-
    /// `:17362`); the port's toggle fires on the key event (`world_key`,
    /// `Main.cInv`'s route) and nothing gates it on the pause — the autoPause
    /// UX (items movable while the world is frozen) holds.
    #[test]
    fn s1_b_3_the_inventory_toggles_while_paused() {
        let mut app = in_world_app();
        app.ingame_options.open = true;
        assert!(app.can_pause_game(), "the fixture starts paused");
        app.settings.keybinds[crate::input_map::Action::Inventory.index()] = "I".into();
        // The live bind table mirrors `settings` only through `apply_settings`.
        app.apply_settings();
        assert!(!app.inventory_open);
        app.world_key(KeyCode::KeyI);
        assert!(app.inventory_open, "the inventory opens while paused");
        app.world_key(KeyCode::KeyI);
        assert!(!app.inventory_open, "and closes again while paused");
    }

    /// DW-1.3 — the click cluster (the pre-gate interface pass,
    /// `Main.cs:17179`'s `DoUpdate_HandleInput` neighbour) runs on paused
    /// frames: the stack-split decay (`Main.cs:62466-62473`) and the hotbar
    /// scale animation both advance with the gate holding.
    #[test]
    fn s1_b_3_the_click_cluster_still_runs_while_paused() {
        let mut app = in_world_app();
        app.ingame_options.open = true;
        app.stack_split = 5;
        app.mine_down = false;
        app.place_down = false;
        app.frame();
        assert!(app.game_paused, "the fixture paused");
        assert_eq!(
            app.stack_split, 0,
            "the interface pass decayed the stack split while paused"
        );
        app.selected = 0;
        app.hotbar_scales[1] = 0.0;
        app.frame();
        assert!(
            (app.hotbar_scales[1] - 0.05).abs() < 1e-6,
            "the hotbar scales animated while paused"
        );
    }

    /// DW-1.4 — the focus pause (`Main.cs:17128-17141`): an unfocused window
    /// freezes the world unless `SettingPlayWhenUnfocused`
    /// (`FocusHelper.cs:137-138`) holds.
    #[test]
    fn s1_b_3_focus_loss_freezes_unless_play_when_unfocused() {
        let mut app = in_world_app();
        app.window_focused = false;
        let time0 = app.world.time;
        let count0 = app.game_update_count;
        app.frame();
        assert!(
            app.game_paused,
            "an unfocused window pauses the world (Main.cs:17138)"
        );
        assert!(
            phase_trace::take().is_empty(),
            "the world branch is skipped while focus-paused"
        );
        assert_eq!(app.world.time, time0, "the clock froze");
        assert_eq!(app.game_update_count, count0, "no world step ran");
        // The setting releases the pause and the branch resumes.
        app.settings.play_when_unfocused = true;
        app.frame();
        assert!(
            !app.game_paused,
            "play_when_unfocused keeps the world running unfocused"
        );
        assert!(app.world.time > time0, "the clock ran again");
        assert_eq!(phase_trace::take(), WORLD_PHASES);
    }

    /// DW-1.4 — the menu/front path keeps flowing while unfocused (the C#'s
    /// `UpdateMenu` arm, `Main.cs:17135-17137`): the port's front screens are
    /// event/draw-driven, so the focus arm must not early-return the frame.
    #[test]
    fn s1_b_3_the_front_path_keeps_flowing_while_unfocused() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(
            Some(FrontScreen::CharSelect),
            vec![],
            vec![],
        );
        app.window_focused = false;
        let frames0 = app.frame_count;
        let anim0 = app.anim_time;
        app.frame();
        assert_eq!(app.frame_count, frames0 + 1, "the front frame still ran");
        assert!(app.anim_time > anim0, "the frame clock advanced");
        assert!(app.front.is_some(), "the front screen stayed live");
        assert!(
            phase_trace::take().is_empty(),
            "and the world branch still never ran on a front frame"
        );
    }

    /// DW-1.4 — the focus arm clears the frame-consumed click edges every
    /// unfocused frame (`Main.cs:17129-17134` clears `mouseLeftRelease`/
    /// `mouseRightRelease`; the port's press-edge pair is what the click
    /// cluster consumes — releases are window events here, no stored flag).
    #[test]
    fn s1_b_3_the_focus_pause_clears_the_pointer_edges() {
        let mut app = in_world_app();
        app.ui_left_press = true;
        app.ui_right_press = true;
        app.window_focused = false;
        app.frame();
        assert!(app.game_paused);
        assert!(
            !app.ui_left_press && !app.ui_right_press,
            "no edge accumulated while unfocused fires on refocus"
        );
    }

    /// DW-1.5 — a world never enters paused (`Main.cs:19856`'s
    /// `Main.gamePaused = false` in `LoadWorld`'s tail): the shared
    /// world-install tail resets the flag, and the constructor carries the
    /// same reset for the `swap_world`/`client_install_world` path (the C#'s
    /// `OnWorldLoad` → `ResetGameCounter` hook, `Main.cs:6605`).
    #[test]
    fn s1_b_3_entering_a_loaded_world_resets_the_pause() {
        let mut app = in_world_app();
        app.game_paused = true;
        app.enter_loaded_world();
        assert!(!app.game_paused, "enter_loaded_world cleared the pause");
        let fresh = in_world_app();
        assert!(!fresh.game_paused, "a fresh App boots unpaused");
        assert_eq!(
            fresh.game_update_count, 65536,
            "ResetGameCounter's initial value (Main.cs:6444)"
        );
    }

    /// The Phase-2 stepper tests' fixture: the in-world app with a slime on a
    /// ground world — the DW-1.2 shape, minus the water/rain instruments the
    /// pacing doesn't read. The liquid sim is rebuilt for the swapped world's
    /// dimensions like DW-1.2 does.
    fn stepper_app() -> App {
        let mut app = in_world_app();
        let mut w = tr_core::World::new("t".into(), 220, 220);
        for x in 0..220 {
            for y in 120..220 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        app.world = w;
        app.npcs = vec![crate::npc::Npc::spawn_slime(&app.world, 20).expect("the slime spawns")];
        app.liquids = tr_core::liquid::LiquidSystem::new(&app.world);
        app
    }

    /// One frame at ~1/60 s (a single world step's dt), the pacing tests'
    /// driver.
    fn step_one_frame(app: &mut App) {
        app.last_frame = std::time::Instant::now() - std::time::Duration::from_millis(17);
        app.frame();
    }

    /// DW-2.3 — the stepper-paused world steps exactly once per allowed
    /// frame: held frames 1-11 advance nothing (counter and clock static),
    /// the 12th held frame advances the counter by exactly one and ticks the
    /// clock, held 13 nothing, held 14 exactly one more — the `Main.cs:17278`
    /// gate wraps the whole `DoUpdateInWorld` body, one counter increment per
    /// executed step (`:17746`).
    #[test]
    fn s1_b_3_stepper_paused_world_steps_once_per_allowed_frame() {
        let mut app = stepper_app();
        // `pauseworld` — the stepper's own pause, not the in-game gate.
        app.stepper.toggle_paused(app.game_update_count);
        app.keys.insert(KeyCode::End);
        let count0 = app.game_update_count;
        let time0 = app.world.time;
        for held in 1..=11 {
            step_one_frame(&mut app);
            assert_eq!(
                app.game_update_count, count0,
                "held frame {held} steps nothing"
            );
            assert_eq!(
                app.world.time, time0,
                "the frozen world clock ticks nothing on held frame {held}"
            );
        }
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count,
            count0 + 1,
            "the 12th held frame steps exactly once"
        );
        assert!(
            app.world.time > time0,
            "the stepped world ticked the clock"
        );
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count, count0 + 1,
            "held frame 13 (odd) steps nothing"
        );
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count,
            count0 + 2,
            "held frame 14 (even) steps exactly one more"
        );
    }

    /// DW-2.3 — `DrawnThisUpdate` (`WorldUpdateStepper.cs:9`): the draw half
    /// marks every in-world frame (`Main.cs:62478`) — blocked, paused and
    /// stepping frames alike end the frame marked, because the fused frame
    /// keeps the C#'s Update→Draw order (a step head resets at `:17747`, the
    /// draw half's set follows it). A stepping frame ending unmarked would
    /// mean the set-site drifted ahead of the reset.
    #[test]
    fn s1_b_3_stepper_drawn_this_update_flips_per_step_and_draw() {
        let mut app = stepper_app();
        assert!(
            !app.stepper.drawn_this_update,
            "the constructor starts unmarked (the C# static's default)"
        );
        app.frame();
        assert!(
            app.stepper.drawn_this_update,
            "an unpaused in-world frame ends marked"
        );
        // Stepper-paused blocked frames keep the mark (no reset runs, the
        // draw half still marks).
        app.stepper.toggle_paused(app.game_update_count);
        app.keys.insert(KeyCode::End);
        for held in 1..=11 {
            step_one_frame(&mut app);
            assert!(
                app.stepper.drawn_this_update,
                "blocked frame {held} keeps the draw mark"
            );
        }
        // The stepping frame: the step head reset (`:17747`) runs, then the
        // draw half marks — the end value is `true`, the C#'s post-Draw state.
        step_one_frame(&mut app);
        assert!(
            app.stepper.drawn_this_update,
            "a stepping frame ends marked (the reset precedes the draw set)"
        );
        // The in-game pause (a different pause source) also keeps drawing —
        // the overlay guard is the stepper's own `Paused`.
        app.keys.remove(&KeyCode::End);
        app.ingame_options.open = true;
        app.frame();
        assert!(
            app.stepper.drawn_this_update,
            "an in-game-paused frame still draws, still marked"
        );
    }

    /// The plan's Phase-2 dirty check — `pauseworld` toggled mid-hold
    /// re-pins the counter mid-pacing (`TogglePaused`'s unconditional
    /// `StartUpdateCount` write, `WorldUpdateStepper.cs:45-49`) and the next
    /// allowed frame still steps exactly once (the pacing count itself
    /// freezes across the toggles — only `should_update_world` moves it).
    #[test]
    fn s1_b_3_stepper_pauseworld_mid_hold_repins_and_next_step_is_single() {
        let mut app = stepper_app();
        app.stepper.toggle_paused(app.game_update_count);
        app.keys.insert(KeyCode::End);
        for _ in 0..12 {
            step_one_frame(&mut app);
        }
        assert_eq!(app.game_update_count, 65537, "held 12 stepped once");
        // Toggle OFF then ON mid-hold — each write re-pins to the current
        // count.
        app.stepper.toggle_paused(app.game_update_count);
        app.stepper.toggle_paused(app.game_update_count);
        assert!(app.stepper.paused, "back to paused");
        assert_eq!(
            app.stepper.start_update_count, 65537,
            "the mid-hold toggle re-pinned"
        );
        // Held 13 (odd) steps nothing; held 14 steps exactly once — the
        // pacing count survived the toggles untouched.
        step_one_frame(&mut app);
        assert_eq!(app.game_update_count, 65537, "held 13 no step");
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count, 65538,
            "held 14 steps exactly once after the mid-hold re-pin"
        );
        // The overlay diffs against the re-pin: exactly one update since.
        assert_eq!(
            app.stepper.hud_text(app.game_update_count).as_deref(),
            Some("Updates Since Paused: 1")
        );
    }

    /// Decision #5 resolved — `SceneState.Update` rides the step gate:
    /// `Main.cs:17816` sits inside `DoUpdateInWorld_Inner`'s tail (after the
    /// camera walks `:17814-17815`), so a stepper-blocked frame freezes it
    /// while the pre-gate weather cluster stays live (decision #6 — the
    /// C# `:17278` gate sits after that cluster). A stepping frame runs the
    /// full pinned branch with `scene_state` at its tail.
    #[test]
    fn s1_b_3_scene_state_is_step_gated() {
        let mut app = in_world_app();
        app.stepper.toggle_paused(app.game_update_count);
        app.frame();
        let trace = phase_trace::take();
        assert!(
            !trace.contains(&"scene_state"),
            "a stepper-blocked frame freezes SceneState.Update (Main.cs:17816)"
        );
        assert!(
            trace.contains(&"weather") && trace.contains(&"rain"),
            "the weather cluster stays live on stepper-blocked frames (decision #6)"
        );
        // The 12th held frame steps — the full branch, `scene_state` last.
        app.keys.insert(KeyCode::End);
        for _ in 1..=11 {
            app.frame();
            phase_trace::take();
        }
        app.frame();
        assert_eq!(
            phase_trace::take(),
            WORLD_PHASES,
            "the stepping frame runs the full pinned branch, scene_state at the tail"
        );
    }

    /// Batch review fix 1 — the step gate now encloses the whole C#
    /// `DoUpdateInWorld` body: a stepper-blocked frame cannot mine. The C#'s
    /// `LookForTileInteractions`/`ItemCheck` ride `Player.Update` inside
    /// `DoUpdateInWorld` (`Player.cs:27430`), so vanilla cannot mine while
    /// the stepper has the world frozen — the port's `update_editing` used to
    /// run under the plain pause gate and mined anyway (the reviewer's
    /// probe). The positive control runs the identical rig unblocked, proving
    /// the fixture mines at all.
    #[test]
    fn s1_b_3_a_stepper_blocked_frame_cannot_mine() {
        let mut app = mining_rig();
        // `pauseworld` — the stepper's own pause, no step key held.
        app.stepper.toggle_paused(app.game_update_count);
        for _ in 0..89 {
            // The camera re-pins each frame so the screen-centre cursor stays
            // on the target tile (the camera walks are step-gated and would
            // drift toward the player otherwise).
            app.camera = Vec2::new(30.0 * tr_core::TILE_PX + 8.0, 260.0 * tr_core::TILE_PX + 8.0);
            step_one_frame(&mut app);
            phase_trace::take();
        }
        app.camera = Vec2::new(30.0 * tr_core::TILE_PX + 8.0, 260.0 * tr_core::TILE_PX + 8.0);
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count, 65536,
            "90 blocked frames stepped nothing"
        );
        assert_eq!(
            app.world.tile(30, 260).map(|t| t.kind),
            Some(1),
            "a stepper-blocked frame never mines the dirt"
        );
        // The blocked frame's trace pins the gate composition: the pre-gate
        // cluster + ambience + the map reveal (draw-side in the C#) — the
        // tile-edit and net halves are inside the gate now.
        assert_eq!(
            phase_trace::take(),
            ["waterfalls", "weather", "rain", "windy_day", "ambience", "map"],
            "a stepper-blocked frame runs no editing or net pass"
        );

        // The positive control: the identical rig, gate open — the tile mines.
        let mut app = mining_rig();
        for _ in 0..90 {
            app.camera = Vec2::new(30.0 * tr_core::TILE_PX + 8.0, 260.0 * tr_core::TILE_PX + 8.0);
            step_one_frame(&mut app);
        }
        assert_ne!(
            app.game_update_count, 65536,
            "the unblocked rig stepped"
        );
        assert_eq!(
            app.world.tile(30, 260).map(|t| t.kind),
            Some(0),
            "the unblocked rig mines the dirt"
        );
    }

    /// The mining rig — the `mining_app` fixture shape (`world_sim/editing.rs`'s
    /// tests) driven through `frame()` instead of `update_editing` directly:
    /// a dirt row, the player beside the target, the Iron Pickaxe (pick 40)
    /// held, LMB (`mine_down`) held, the cursor at the screen centre (the
    /// headless screen is 1280x720, so the centre maps onto the camera).
    fn mining_rig() -> App {
        let mut app = in_world_app();
        let mut w = tr_core::World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 260);
            w.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        app.world = w;
        app.liquids = tr_core::liquid::LiquidSystem::new(&app.world);
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 33.0 * tr_core::TILE_PX;
        // The hitbox top sits one player-height above the row top, so the
        // player STANDS on the dirt row (spawning inside it depenetrates
        // downward into the air below the one-tile row and falls out of
        // reach mid-mine).
        p.y = 260.0 * tr_core::TILE_PX - crate::player::HEIGHT as f32;
        app.player = Some(p);
        app.inventory[app.selected].kind = 1; // the Iron Pickaxe (pick 40)
        app.inventory[app.selected].count = 1;
        app.camera = Vec2::new(30.0 * tr_core::TILE_PX + 8.0, 260.0 * tr_core::TILE_PX + 8.0);
        // The screen centre of frame()'s headless screen (1600x900 —
        // `frame()`'s `screen` differs from the editing tests' MINE_SCREEN),
        // so the cursor's world point is exactly the camera = the tile.
        app.mouse_pos = Vec2::new(800.0, 450.0);
        app.mine_down = true;
        app
    }

    /// Batch review fix 2's App-level half — the NumpadAdd arm rides the same
    /// `keys` set End does (the frame-head snapshot pins it into
    /// `step_input_down`), and the XButton1 arm steps through
    /// `mouse_back_held` alone. Each arm alone paces to a step at the 12th
    /// held frame, through the real `Main.cs:17278` gate.
    #[test]
    fn s1_b_3_step_key_arms_drive_the_gate_numpadadd_and_mouse_back() {
        // The NumpadAdd arm (`Keys.Add`).
        let mut app = stepper_app();
        app.stepper.toggle_paused(app.game_update_count);
        app.keys.insert(KeyCode::NumpadAdd);
        let count0 = app.game_update_count;
        for _ in 0..11 {
            step_one_frame(&mut app);
        }
        assert_eq!(
            app.game_update_count, count0,
            "NumpadAdd-held frames 1-11 step nothing"
        );
        assert!(
            app.step_input_down,
            "the frame head pinned the NumpadAdd arm into step_input_down"
        );
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count,
            count0 + 1,
            "the 12th NumpadAdd-held frame steps"
        );

        // The XButton1 arm (`PlayerInput.MouseInfo.XButton1` — the port's
        // `mouse_back_held`, fed by loop.rs's `MouseButton::Back`), no key held.
        let mut app = stepper_app();
        app.stepper.toggle_paused(app.game_update_count);
        app.mouse_back_held = true;
        let count0 = app.game_update_count;
        for _ in 0..11 {
            step_one_frame(&mut app);
        }
        assert_eq!(
            app.game_update_count, count0,
            "mouse-back-held frames 1-11 step nothing"
        );
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count,
            count0 + 1,
            "the 12th mouse-back-held frame steps"
        );
    }
}

/// M1.4-BG-18 phase 2 harness: uploads `Images/Misc/noise` — the noise image every
/// M1 screen filter binds (`ScreenEffectInitializer.cs:30-40`) — for the post pass
/// bindings. Missing content returns None and `PostPipeline::apply_filter` binds
/// its checkerboard fallback instead (plan edge case).
pub(crate) fn load_noise_texture(gpu: &tr_render::Gpu) -> Option<wgpu::Texture> {
    let path = crate::assets::content_dir()?.join("Images/Misc/noise.xnb");
    let img = tr_content::load_texture(&path).ok()?;
    let tex = gpu.device.create_texture_with_data(
        &gpu.queue,
        &wgpu::TextureDescriptor {
            label: Some("post-noise"),
            size: wgpu::Extent3d {
                width: img.width,
                height: img.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &img.pixels,
    );
    Some(tex)
}

pub(crate) fn next_rand(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub(crate) fn rand01(state: &mut u64) -> f32 {
    (next_rand(state) >> 40) as f32 / (1u32 << 24) as f32
}

/// The u64 streams' `Main.rand.Next(max)`-shaped roll — `next_rand` modulo,
/// the same shape `Rain::range` uses (uniform over 0..hi).
pub(crate) fn rand_range(state: &mut u64, hi: u32) -> u32 {
    (next_rand(state) % hi.max(1) as u64) as u32
}

/// The u64 streams' signed `Main.rand.Next(min, max)` — the critter
/// family's pitch draws (`Next(-70, 1)` and friends,
/// `LegacySoundPlayer.cs:450`-family) ride it.
pub(crate) fn rand_range_signed(state: &mut u64, lo: i32, hi: i32) -> i32 {
    lo + (next_rand(state) % (hi.saturating_sub(lo).max(1)) as u64) as i32
}
