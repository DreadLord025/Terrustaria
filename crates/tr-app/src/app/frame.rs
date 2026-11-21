//! The frame orchestrator — `Main.Draw`'s port. One `frame()` per
//! `RedrawRequested`: the sim-facing timers, the animated colour cycles,
//! the light parameters, the world-instance cache (G6) and the world plus
//! interface draw. Plus the benchmark epilogue and the two RNG helpers the
//! world simulation shares. (A7: extracted from `main.rs` whole.)

mod world;
mod draw;
use std::time::Instant;

use glam::Vec2;
use wgpu::util::DeviceExt;

use crate::app::App;
use crate::sound;
use crate::ui;

/// Records one **draw**-branch phase for the draw-order pinning test
/// (`S1-B-7`). A no-op outside `cfg(test)` — the names make the
/// `Main.DoDraw` order (`Main.cs:62249-62326`) assertable instead of only
/// readable. A free function (not an `App` method): the draw branch holds
/// a mutable borrow of the GPU handle across the passes.
#[inline]
fn draw_phase(#[cfg_attr(not(test), allow(unused_variables))] name: &'static str) {
    #[cfg(test)]
    draw_phase_trace::record(name);
}

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

    /// The per-frame head — `Main.Draw`'s top region, named by S1-B-8: the
    /// live language substitutions, the `TR_SHOT`-pinned frame step, the
    /// per-frame counters the rest of the frame reads (the zoom keys, the
    /// adaptive fps counter, the step-key snapshot pair, the animation clock),
    /// the always-run UI/animation ticks and the autosave budget. Runs on
    /// **every** frame — menus and focus pauses included — exactly like the
    /// vanilla's Update/Draw head, so it must stay out of the in-world branch.
    ///
    /// `now` is the frame's `Instant`, taken by the caller because the draw half
    /// timestamps its filter/render passes off the same stamp. Yields
    /// `(dt, anim_time)`: the frame step (the `TR_SHOT` pin) and the animation
    /// clock snapshot the draw passes key on.
    fn frame_head(&mut self, now: Instant) -> (f32, f32) {
        // `Lang.InitGlobalSubstitutions` (S1-L-12b): refresh live `{Name}` values
        // before any UI/text resolves them this frame (`Lang.cs:100-167`).
        crate::lang::refresh_global_substitutions(self);

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
        // `Main.DrawInventory`'s reforge head (`Main.cs:41557-41578`): the
        // button's hover-scale animation, the unhovered cooldown clear and
        // the force-close gate. Runs every frame the menu is open (the C#
        // block sits in `DrawInventory`, which runs whenever `playerInventory`).
        self.update_reforge_menu();
        // `Main.DrawInterface_26_InterfaceLogic3` (`Main.cs:45676-45683`):
        // with the inventory closed, `InReforgeMenu = false` and the slot
        // item returns through `dropItemCheck` (`Player.cs:5123-5127`).
        if !self.inventory_open && self.in_reforge_menu {
            self.in_reforge_menu = false;
            self.mouse_reforge = false;
            self.return_reforge_slot();
        }
        // `IngameOptions.Draw`'s per-frame hover/animation state and its
        // `SoundEngine.PlaySound(12)`/`(10)` cues (`IngameOptions.cs:157-199`,
        // `:421-436`) — plus the sticky bar drag (`:424`). Runs on every frame
        // like the vanilla's, and before the HUD borrows the state. The drag
        // reads `Main.mouseLeft` — the physical left button (`mine_down`), not
        // the front-end pointer machine's `left_held`, which only tracks the
        // 888 screens' presses.
        if let Some(cue) = crate::ui::ingame_options::update(self, self.mine_down) {
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
        // `DrawInterface_38_MouseCarriedObject`'s per-frame head lifecycle
        // (`Main.cs:45185-45211`) + the closed-inventory clear (`:45666`).
        self.refresh_mouse_npc();
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

        (dt, anim_time)
    }

    /// The shared per-frame setup — the region between the head and the two
    /// frame branches: the GPU (or the inline 1600×900) screen size the whole
    /// frame draws into, the minimap's `m_h`, the OS cursor visibility, the
    /// audio listener and pool update, the ambience waterfall/lavafall loops,
    /// the menu music, and the front end's pending **world swap** at a frame
    /// boundary (after the swap, before the world branch, so a half-ticked world
    /// never shows).
    ///
    /// Yields `(screen, m_h)` — both are consumed deep in the draw half.
    fn frame_shared_setup(&mut self, dt: f32) -> (Vec2, f32) {
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

        // `Main.Ambience` (`Main.cs:13466-13509`): every 15 ticks (¼ s) the
        // waterfall/lavafall loops re-volume from the strength scans; the
        // volume math is the LegacySoundPlayer case 34/35 (the distance
        // attenuation + strength/50 + the 0.2/0.65 multipliers). The strength
        // scans ran in the waterfall pass this frame. The decision half is
        // `ambient_sound::liquid_loop_update` — the (int) PlaySound casts,
        // the lavafall arm's ambientLava Manhattan pick + strength sum
        // (`:13488-13497`) and the out-of-range keep-last-volume arm
        // (`LegacySoundPlayer.cs:383-397` — `flag` false skips the block;
        // the case bodies are `:1249-1279` + `:1279-1307`).
        self.ambient_counter += dt;
        if self.ambient_counter >= 0.25 {
            self.ambient_counter = 0.0;
            // `Main.screenPosition + (screenWidth/2, screenHeight/2)` with
            // the int halving (`Main.cs:13488-13489`); the view's top-left
            // is the same unscaled camera - screen/2 the draw pass feeds
            // `ScreenInfo`.
            let centre = (
                self.camera.x - screen.x * 0.5 + (screen.x as i32 / 2) as f32,
                self.camera.y - screen.y * 0.5 + (screen.y as i32 / 2) as f32,
            );
            for channel in 0..2 {
                let triple = if channel == 0 {
                    self.ambient_waterfall
                } else {
                    self.ambient_lavafall
                };
                let update = crate::ambient_sound::liquid_loop_update(
                    channel,
                    triple,
                    self.ambient_lava,
                    centre,
                    (self.camera.x, self.camera.y),
                    self.settings.ambient_volume,
                    self.menu.is_some() || self.front.is_some(),
                    self.audio.liquid_loop_playing(channel),
                    self.scene_state.shimmer_alpha,
                );
                if let Some((volume, pan)) = update.volume_pan {
                    self.audio.set_liquid_loop(channel, volume, pan, update.pitch);
                } else if !update.playing {
                    // The stop arm (`:1257-1262`/`:1287-1292`) — volume 0
                    // stops the slot; the out-of-range arm (playing kept)
                    // leaves the live loop untouched.
                    self.audio.set_liquid_loop(channel, 0.0, 0.0, None);
                }
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

        // S1-E-17: the loading screen's per-frame update (`UIWorldLoad.Update`
        // + the `worldGenCallback` completion tail) — the progress latch and
        // the result poll run before the draw, exactly as the C# `UpdateMenu`
        // drives `MenuUI.Update` ahead of `DoDraw`.
        self.update_world_load();

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

        (screen, m_h)
    }

    /// The focus pause arm (S1-B-3), named by S1-B-8. Returns whether this frame
    /// is focus-paused; the caller gates the in-world branch on the flag, which
    /// is how the port's fused frame reproduces the C#'s early `return` from
    /// *Update* while Draw keeps running (the details at the site below).
    fn frame_focus_pause(&mut self) -> bool {
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

        focus_paused
    }

    pub(crate) fn frame(&mut self) {
        // The frame's wall-clock stamp: the head derives `dt` from it (the
        // `TR_SHOT` pin aside) and the draw half's filter/render passes
        // timestamp off it, so the orchestrator takes it and hands it down.
        let now = Instant::now();
        let (dt, anim_time) = self.frame_head(now);
        let (screen, m_h) = self.frame_shared_setup(dt);
        let focus_paused = self.frame_focus_pause();

        // `InGameNotificationsTracker.Update()` (`Main.cs:17045`): the popup
        // timers tick on every update — menus included (the call sits before
        // the `gameMenu` return at `:17166`), and the capture gate
        // (`:16886-16890`) freezes it like the rest of the update. S1-E-26.
        if !self.capture.is_capturing() {
            self.notifications.update();
        }

        // S1-B-8 — the in-world branch (`DoUpdateInWorld`'s port): one fn
        // per phase, in the C#'s order — see `frame/world.rs`.
        if !focus_paused && self.menu.is_none() && self.front.is_none() {
            // S1-E-17: `DoUpdate`'s capture gate (`Main.cs:16886-16890`):
            // `if (CaptureManager.Instance.IsCapturing) { FocusHelper.UpdateFocus;
            // return; }` — while a capture renders the ENTIRE game update
            // freezes (Sandstorm.EmitDust at :17247, SceneState.Update at
            // :17816, the world sim, all sit AFTER this return). The capture
            // interface itself still updates (it rides the interface pass,
            // not DoUpdate's world branch), so update_capture runs regardless.
            if !self.capture.is_capturing() {
                self.update_world_frame(dt, screen);
            }
            // S1-E-17: `CaptureManager.Instance.Update` at the interface-pass
            // call site (`Main.cs:46187` world / `:62166` map) — the capture
            // selection UI is an in-game interface, so it rides the world
            // branch (the front end never draws it).
            self.update_capture(screen);
        }
        // The `OnAchievementCompleted` subscribers (`AchievementManager.cs:
        // 180-187` → `InGameNotificationsTracker.AddCompleted` +
        // `AchievementInitializer.OnAchievementCompleted`): the completions
        // the update branch queued this frame become popups + the chat line
        // + the cue. S1-E-26 wave A.
        self.drain_achievement_completions();
        // S1-B-8 — the draw-half pre-snapshot spine (see `frame/world.rs`);
        // the pure field snapshots below stay inline: the draw region's
        // borrow scaffolding, consumed by the destructure.
        self.draw_half_pre_snapshots(dt);
        // S1-E-17: `CaptureManager.Instance.DrawTick()` at the `DoDraw` head
        // (`Main.cs:61729`) — the countdown advances and a started capture
        // drains one chunk (the GPU render is the S1-G-13 seam).
        self.capture_draw_tick();
        // S1-B-8 — the draw region (`Main.DoDraw`'s body): the pre-snapshot
        // locals, the destructure and the gpu block live in
        // `frame/draw.rs::draw_region`, verbatim; it yields the frame's stats.
        let (stats, tile_stats) = match self.draw_region(
            now, dt, anim_time, screen, m_h,
        ) {
            Some(pair) => pair,
            None => return,
        };
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

/// The draw branch's phase trace (`S1-B-7`): the `Main.DoDraw` order
/// (`Main.cs:62249-62326`) made assertable. Separate from the update
/// trace — the two run in different branches of `frame()`.
pub(crate) mod draw_phase_trace {
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
    use super::draw_phase_trace;
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
    const WORLD_PHASES: [&str; 22] = [
        "players",
        "npcs",
        // `Main.UpdateWorld_Gores` (`Main.cs:17971`, called at `:17771`) sits
        // between the NPCs and the projectiles.
        "gores",
        "projectiles",
        "items",
        "dust",
        "time",
        "wiring",
        "tile_entities",
        "liquids",
        "growth",
        "waterfalls",
        // `Main.AnimateTiles` (`Main.cs:17150`): the pre-gate Animate
        // cluster's tile animator, right after `DoUpdate_AnimateWalls`
        // (`:17149`) — the S1-G-1f wiring ticks it beside the waterfalls.
        "animate_tiles",
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
        // `Main.chatMonitor.Update()` (`Main.cs:17282`): after the gated
        // `DoUpdateInWorld` (`:17278-17281`), before the `!paused` gate's
        // siblings end — the chat corner-visibility timers decay on every
        // unpaused frame, stepper-blocked included.
        "chat",
    ];

    /// An in-world headless `App`: the flow fixture plus a spawned player, so
    /// `frame()` takes the `menu.is_none() && front.is_none()` branch.
    fn in_world_app() -> App {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app
    }

    /// The draw-phase call names of a source file, in order.
    fn draw_phase_calls(src: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut rest = src;
        // The needle is assembled at run time — the literal must not appear
        // in this helper's own source.
        let needle = ["draw_phase", "(\""].concat();
        while let Some(at) = rest.find(needle.as_str()) {
            rest = &rest[at + needle.len()..];
            let end = rest.find('"').expect("a closed phase name");
            out.push(&rest[..end]);
            rest = &rest[end..];
        }
        out
    }

    #[test]
    fn the_draw_passes_run_in_the_csharp_order() {
        let tiles = include_str!("../render/tiles.rs");
        // The draw passes moved to `frame/draw.rs` with the draw region
        // (S1-B-8 phase 3); the pin scans that module's text.
        let frame = include_str!("frame/draw.rs");
        // The world pass's entity phases: the NPCs behind the tiles
        // (`:63293`) → the NPCs over the tiles (`:63301`) → the players
        // (`:62256`) → the dropped items (`:62278`).
        assert_eq!(
            draw_phase_calls(tiles),
            ["npcs_behind_tiles", "npcs", "player", "items"],
            "the entities must follow Main.cs:62249-62326"
        );
        // The frame's draw passes: the sky (`:62080`) → the world pass
        // (`:62249`) → the rain/dust (`:62279`/`:62293`) → the wires
        // (`:62303`) → the projectiles over the wires UI (`:62315`) → the
        // combat text (`:62340-62396`, the `Main.combatText` walk) → the
        // ordinary NPC health bars (`DrawInterface_14_EntityHealthBars`,
        // `:45810`).
        assert_eq!(
            draw_phase_calls(frame),
            [
                "sky",
                "world",
                "rain",
                "wires",
                "projectiles",
                "combat_text",
                "entity_health_bars"
            ],
            "the draw passes must follow Main.cs:62249-62326"
        );
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

    /// S1-E-17: the `DoUpdate` capture gate (`Main.cs:16886-16890`):
    /// `if (CaptureManager.Instance.IsCapturing) { FocusHelper.UpdateFocus;
    /// return; }` — while a capture renders the whole world update freezes
    /// (the sim, Sandstorm.EmitDust's dust RNG, SceneState.Update all sit
    /// after the return). The world clock must not age and no world phase
    /// may run while capturing; the capture interface itself still updates.
    #[test]
    fn capturing_freezes_the_world_update() {
        let mut app = in_world_app();
        app.frame();
        phase_trace::take(); // drain the warm-up frame
        let settings = crate::capture::CaptureSettings {
            // 300x300 tiles -> a 3x3 grid of 126-stride chunks (9 chunks), so
            // the capture spans multiple frames (draw_tick drains one chunk
            // per frame); a 1-chunk capture would finish on frame 1 and the
            // gate would clear before the later frames.
            area: crate::capture::CaptureRect::new(0, 0, 300, 300),
            ..Default::default()
        };
        app.capture.capture(settings).unwrap();
        assert!(app.capture.is_capturing());
        let frozen_time = app.world.time;
        for _ in 0..3 {
            app.frame();
            assert!(app.capture.is_capturing(), "the 9-chunk capture must still be active");
        }
        assert_eq!(app.world.time, frozen_time, "the world clock must not age while capturing");
        assert!(
            phase_trace::take().is_empty(),
            "no world phase may run while capturing (DoUpdate returns early)"
        );
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
        assert_eq!(phase_trace::take(), ["waterfalls", "animate_tiles", "windy_day"]);
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
            ["waterfalls", "animate_tiles", "windy_day"],
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
        assert_eq!(phase_trace::take(), ["waterfalls", "animate_tiles", "windy_day"]);
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
        assert!(app.world.time > time0, "the stepped world ticked the clock");
        step_one_frame(&mut app);
        assert_eq!(
            app.game_update_count,
            count0 + 1,
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
            app.camera = Vec2::new(
                30.0 * tr_core::TILE_PX + 8.0,
                260.0 * tr_core::TILE_PX + 8.0,
            );
            step_one_frame(&mut app);
            phase_trace::take();
        }
        app.camera = Vec2::new(
            30.0 * tr_core::TILE_PX + 8.0,
            260.0 * tr_core::TILE_PX + 8.0,
        );
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
            [
                "waterfalls",
                "animate_tiles",
                "weather",
                "rain",
                "windy_day",
                "ambience",
                "map",
                // `Main.chatMonitor.Update()` (`Main.cs:17282`) sits after
                // the `DoUpdateInWorld` gate — it ticks on blocked frames.
                "chat",
            ],
            "a stepper-blocked frame runs no editing or net pass"
        );

        // The positive control: the identical rig, gate open — the tile mines.
        let mut app = mining_rig();
        for _ in 0..90 {
            app.camera = Vec2::new(
                30.0 * tr_core::TILE_PX + 8.0,
                260.0 * tr_core::TILE_PX + 8.0,
            );
            step_one_frame(&mut app);
        }
        assert_ne!(app.game_update_count, 65536, "the unblocked rig stepped");
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
        app.camera = Vec2::new(
            30.0 * tr_core::TILE_PX + 8.0,
            260.0 * tr_core::TILE_PX + 8.0,
        );
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

    // ---- S1-B-8 phase 1: the frame's named head / setup / pause arms ----

    /// The source slice of one `impl App` fn — from its signature to the next
    /// top-level item in the impl block. The S1-B-8 marker scans read it, so a
    /// citation that slipped out of an extracted body (or a body that lost its
    /// split) fails the scan instead of passing silently against the whole
    /// file. The needle is the full signature, so a param-set rewrite of the
    /// head also drops the slice's start and the test that reads it.
    fn fn_body<'a>(src: &'a str, sig: &str) -> &'a str {
        let start = src
            .find(sig)
            .unwrap_or_else(|| panic!("frame.rs no longer defines `{sig}`"));
        let rest = &src[start + sig.len()..];
        let end = ["\n    fn ", "\n    pub(crate) fn "]
            .iter()
            .filter_map(|needle| rest.find(needle))
            .min()
            .unwrap_or(rest.len());
        &rest[..end]
    }

    /// S1-B-8 DW-1.1 — the three named fns exist with their documented yields,
    /// the orchestrator is the four-line head that hands the stamp and the
    /// yields down, and each extracted body still carries its A/B/C citation
    /// markers: the per-line grep-diff of the phase, made repeatable.
    #[test]
    fn s1_b_8_phase1_the_frame_head_is_extracted() {
        let src = include_str!("frame.rs");
        let head_sig = "fn frame_head(&mut self, now: Instant) -> (f32, f32)";
        let setup_sig = "fn frame_shared_setup(&mut self, dt: f32) -> (Vec2, f32)";
        let pause_sig = "fn frame_focus_pause(&mut self) -> bool";
        assert!(src.contains(head_sig), "frame_head must keep its shape");
        assert!(src.contains(setup_sig), "frame_shared_setup must keep its shape");
        assert!(src.contains(pause_sig), "frame_focus_pause must keep its shape");
        // The orchestrator takes the stamp at its head and passes it on.
        assert!(
            src.contains("let now = Instant::now();"),
            "frame() takes the frame's Instant at its head"
        );
        assert!(
            src.contains("let (dt, anim_time) = self.frame_head(now);"),
            "frame() calls frame_head with the stamp"
        );
        assert!(
            src.contains("let (screen, m_h) = self.frame_shared_setup(dt);"),
            "frame() hands dt to frame_shared_setup and takes both yields"
        );
        assert!(
            src.contains("let focus_paused = self.frame_focus_pause();"),
            "frame() takes the pause flag from frame_focus_pause"
        );

        // A — the head: the TR_SHOT dt pin and the per-frame counters.
        let head = fn_body(src, head_sig);
        for marker in [
            "`TR_SHOT` pins the animation clock",
            "Main.cs:16801-16815,18376",
            "Main.cs:16961-16964",
            "WorldUpdateStepper.cs:39-43",
            "UserInterface.cs:203-204",
            "Main.cs:17048-17051",
            "IngameOptions.cs:157-199",
            "Main.cs:45513-45527",
            "Main.cs:42864-42867",
            "Main.cs:19751-19765",
        ] {
            assert!(head.contains(marker), "frame_head lost `{marker}`");
        }
        // B — the setup: the inline screen, mH, cursor, audio, ambience,
        // menu music and the world swap.
        let setup = fn_body(src, setup_sig);
        for marker in [
            "Main.cs:45771-45783",
            "FocusHelper.cs:160-163",
            "SoundEngine.cs:48-68",
            "Main.cs:13466-13515",
            "LegacySoundPlayer.cs:383-397",
            // The strength/50 multiplier arm (the class name rides the line
            // above the `:1249-1279` marker).
            "`:1249-1279`",
            ":12455",
            "Main.LoadWorld",
        ] {
            assert!(setup.contains(marker), "frame_shared_setup lost `{marker}`");
        }
        // C — the focus pause: the wantsToPause arm and its two side effects.
        let pause = fn_body(src, pause_sig);
        for marker in [
            "Main.cs:17128-17141",
            "Main.cs:17129-17134",
            "Main.cs:17138",
        ] {
            assert!(pause.contains(marker), "frame_focus_pause lost `{marker}`");
        }
    }

    /// S1-B-8 DW-1.1 (behavioural half) — `frame_head` yields the pinned frame
    /// step and the animation clock: dt is the clamped wall step, `anim_time`
    /// advances by exactly dt, the step-key snapshot shifts, and the `TR_SHOT`
    /// pin overrides the wall clock with the fixed 1/60 step — all as the
    /// single `frame()` did before the split.
    #[test]
    fn s1_b_8_phase1_the_frame_head_yields_the_pinned_dt_and_clock() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        // A frame 100 ms late: the clamp at the head yields exactly 0.1 s.
        let now = std::time::Instant::now();
        app.last_frame = now - std::time::Duration::from_millis(100);
        let frames0 = app.frame_count;
        let anim0 = app.anim_time;
        let down0 = app.step_input_down;
        app.keys.insert(KeyCode::End);
        let (dt, anim_time) = app.frame_head(now);
        assert!((dt - 0.1).abs() < 1e-6, "dt is the clamped frame step ({dt})");
        assert!(app.last_frame == now, "the head stores the caller's stamp");
        assert_eq!(app.frame_count, frames0 + 1, "one head run per frame");
        assert!(
            (app.anim_time - (anim0 + dt)).abs() < 1e-6,
            "anim_time advanced by exactly dt"
        );
        assert!(
            anim_time == app.anim_time,
            "the yield is the post-step animation clock"
        );
        // The step-key snapshot pair (`WorldUpdateStepper.cs:39-43`): `down`
        // reads live, the previous frame's value shifts into `was`.
        assert!(app.step_input_down, "End pinned the live step key");
        assert!(
            app.step_input_was == down0,
            "the previous frame's pin shifted into step_input_was"
        );
        // The TR_SHOT pin (`Main.cs`'s deterministic-frame contract): whatever
        // the wall clock says, the step is the fixed 1/60.
        app.shot_path = Some(std::path::PathBuf::from("s1-b-8-shot.png"));
        app.last_frame = now - std::time::Duration::from_millis(100);
        let (pinned, _) = app.frame_head(now);
        assert!(
            (pinned - 1.0 / 60.0).abs() < 1e-9,
            "TR_SHOT pins dt to 1/60 ({pinned})"
        );
    }

    /// S1-B-8 DW-1.3 — the setup's inline 1600x900 fallback (not
    /// `App::screen_size()`'s 1280x720) is what the whole frame draws into,
    /// and `m_h` comes from that fallback: with a live minimap the setup
    /// yields 256 while `App::main_m_h()` — the un-unified twin the plan's
    /// do-not-unify note covers — clamps to 120 (`Main.cs:45795-45798`).
    #[test]
    fn s1_b_8_phase1_the_setup_returns_the_inline_screen_fallback() {
        let mut app = in_world_app();
        // A live, non-fullscreen minimap: `mapEnabled && !mapFullscreen &&
        // mapStyle == 1` (`Main.cs:45771-45783`).
        app.map = Some(crate::map::WorldMap::new(&app.world));
        let (screen, m_h) = app.frame_shared_setup(1.0 / 60.0);
        assert_eq!(
            screen,
            Vec2::new(1600.0, 900.0),
            "the headless frame draws into the inline 1600x900 fallback"
        );
        assert!(
            (m_h - 256.0).abs() < 1e-6,
            "m_h is computed from the 900-tall fallback ({m_h})"
        );
        assert!(
            (app.main_m_h() - 120.0).abs() < 1e-6,
            "App::main_m_h() clamps against screen_size()'s 720 — the twin \
             that stays separate"
        );
    }

    /// S1-B-8 DW-1.3 — the front end's pending world swap is taken inside the
    /// setup (a frame boundary: after the front screens' clicks parked it and
    /// before the world branch), and its failure arm keeps the current world
    /// with the `front_hint` the menus render (`UIWorldListItem.PlayGame` →
    /// `Main.LoadWorld`, BUG-04).
    #[test]
    fn s1_b_8_phase1_the_world_swap_yields_inside_the_setup() {
        let mut app = in_world_app();
        let world0 = app.world.name.clone();
        app.play_world = Some(std::path::PathBuf::from("nowhere-s1-b-8.wld"));
        let (screen, _m_h) = app.frame_shared_setup(1.0 / 60.0);
        assert!(
            app.play_world.is_none(),
            "the pending world is taken at the frame boundary"
        );
        let hint = app.front_hint.as_deref().unwrap_or_default();
        assert!(
            hint.starts_with("world load failed"),
            "the Err arm keeps the menu with its front_hint ({hint})"
        );
        assert_eq!(
            app.world.name, world0,
            "a failed load keeps the current world (Main.LoadWorld's failure)"
        );
        assert_eq!(
            screen,
            Vec2::new(1600.0, 900.0),
            "the frame still draws at the fallback after the failed swap"
        );
    }

    /// S1-B-8 DW-1.4 — the Phase-3 re-borrow contract, pinned: the three
    /// extracted fns are `&mut self` re-borrows with the documented param sets,
    /// so a param-set rewrite of the frame head (the shape-A spike the
    /// discovery rejected) fails this test instead of compiling. The sequence
    /// below also drives the three sequential re-borrows in the orchestrator's
    /// own order — the shape the draw phases of Phase 3 follow.
    #[test]
    fn s1_b_8_phase1_the_re_borrow_strategy_is_the_frame_contract() {
        // The signatures as function items: a coercion failure is the pin.
        let _head: fn(&mut App, std::time::Instant) -> (f32, f32) = App::frame_head;
        let _setup: fn(&mut App, f32) -> (Vec2, f32) = App::frame_shared_setup;
        let _pause: fn(&mut App) -> bool = App::frame_focus_pause;
        // The orchestrator's head, driven straight: one stamp, then the three
        // sequential `&mut self` re-borrows (each borrow ends at its call).
        let mut app = in_world_app();
        let now = std::time::Instant::now();
        let (dt, anim_time) = app.frame_head(now);
        let (screen, m_h) = app.frame_shared_setup(dt);
        let focus_paused = app.frame_focus_pause();
        assert!(dt > 0.0, "the head yielded a frame step");
        assert!(anim_time >= dt, "the clock advanced with the step");
        assert_eq!(screen, Vec2::new(1600.0, 900.0), "the setup yielded the fallback");
        assert!(m_h >= 0.0, "the setup yielded the minimap height");
        assert!(
            !focus_paused,
            "a focused window is not focus-paused (Main.cs:17128-17141)"
        );
    }

    /// S1-B-8 DW-1.3 — the `TR_*` reads stay with the blocks that own them:
    /// the `TR_SHOT` pin reads `shot_path` inside the head (twice: the dt pin
    /// and the adaptive fps counter) and the cursor gate reads
    /// `env::no_hud()` inside the setup; neither read crossed the split.
    #[test]
    fn s1_b_8_phase1_the_env_reads_stay_inside_the_extracted_bodies() {
        let src = include_str!("frame.rs");
        let head = fn_body(src, "fn frame_head(&mut self, now: Instant) -> (f32, f32)");
        let setup = fn_body(src, "fn frame_shared_setup(&mut self, dt: f32) -> (Vec2, f32)");
        let pins = head.matches("self.shot_path.is_some()").count();
        assert_eq!(
            pins, 2,
            "the TR_SHOT pin's two shot_path reads stay in frame_head"
        );
        assert!(
            head.contains("self.last_frame = now;"),
            "frame_head owns the frame stamp store"
        );
        let cursors = setup.matches("crate::env::no_hud()").count();
        assert_eq!(
            cursors, 1,
            "the cursor gate's no_hud read stays in frame_shared_setup"
        );
        // The reads did not migrate across the A/B boundary.
        assert!(
            !head.contains("crate::env::no_hud"),
            "no_hud belongs to the setup, not the head"
        );
        assert!(
            !setup.contains("shot_path"),
            "shot_path belongs to the head, not the setup"
        );
    }

    /// S1-B-8 phase 2: the in-world branch lives in `frame::world` as one fn
    /// per `phase("…")` site, and the orchestrator (`update_world_frame`)
    /// calls them in the frozen S1-B-1 order. The include scan pins the fn
    /// set; the behavioural order is `the_world_branch_runs_in_the_csharp_
    /// update_order`'s, which walks the trace through the new module.
    #[test]
    fn s1_b_8_phase2_the_world_branch_calls_the_named_phase_fns() {
        let world = include_str!("frame/world.rs");
        for name in [
            "in_world_click_cluster",
            "in_world_pause_gates",
            "in_world_step_gate",
            "world_step_head",
            "players_phase",
            "npcs_phase",
            "gores_phase",
            "projectiles_phase",
            "items_phase",
            "dust_phase",
            "time_phase",
            "wiring_phase",
            "tile_entities_phase",
            "liquids_phase",
            "growth_phase",
            "waterfalls_phase",
            "animate_tiles_phase",
            "weather_cluster",
            "windy_day_phase",
            "ambience_phase",
            "editing_net_phase",
            "map_phase",
            "camera_scene_tail",
            "chat_phase",
            "gfx_quality_apply",
        ] {
            assert!(
                world.contains(&format!("fn {name}(")),
                "the extracted fn {name} exists in frame/world.rs"
            );
        }
        // The orchestrator's frozen call order — the same sequence the
        // `phase()` trace walks (the C#'s `DoUpdateInWorld_Inner`, with the
        // port's documented fusions). The slice is bounded at the next
        // `pub(super) fn` so the spine's statements don't bleed in.
        let orch = world
            .split("pub(super) fn update_world_frame")
            .nth(1)
            .expect("the world-branch orchestrator")
            .split("pub(super) fn draw_half_pre_snapshots")
            .next()
            .expect("the spine follows the orchestrator")
            .to_string();
        let calls: Vec<&str> = orch
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                let idx = l.find("self.")?;
                let rest = &l[idx + 5..];
                let end = rest.find('(')?;
                Some(&rest[..end])
            })
            .collect();
        assert_eq!(
            calls,
            vec![
                "in_world_click_cluster",
                "in_world_pause_gates",
                "in_world_step_gate",
                "world_step_head",
                "players_phase",
                "npcs_phase",
                // `Main.UpdateWorld_Gores` (`Main.cs:17971`, called at
                // `:17771-17774`): between the NPCs and the projectiles
                // — landed on main with the S1-E-2 gores port; the pin
                // gains the phase at the C#'s position.
                "gores_phase",
                "projectiles_phase",
                "items_phase",
                "dust_phase",
                "time_phase",
                "wiring_phase",
                "tile_entities_phase",
                "liquids_phase",
                "growth_phase",
                "waterfalls_phase",
                // `Main.AnimateTiles` (`Main.cs:17150`, after
                // `DoUpdate_AnimateWalls` `:17149`): the pre-gate Animate
                // cluster's tile animator — S1-G-1f wiring adds the phase
                // at the C#'s position (the waterfall pass stands in for
                // the walls animator's slot).
                "animate_tiles_phase",
                "weather_cluster",
                "windy_day_phase",
                "ambience_phase",
                "editing_net_phase",
                "map_phase",
                "camera_scene_tail",
                // `Main.chatMonitor.Update()` (`Main.cs:17282`) — after
                // the gated `DoUpdateInWorld`, under the `!paused` arm.
                "chat_phase",
                "gfx_quality_apply",
            ],
            "the orchestrator's call order is the frozen phase order"
        );
        // The orchestrator is the branch's only body: `frame()` delegates.
        let src = include_str!("frame.rs");
        assert!(
            src.contains("self.update_world_frame(dt, screen);"),
            "frame() hands the in-world branch to frame::world"
        );
    }

    /// S1-B-8 phase 2: the gate arms ride the extracted fns unchanged — the
    /// pause/step gates wrap exactly the phases they wrapped before the split
    /// (S1-B-3's per-gate traces assert the behaviour; this pins the text).
    #[test]
    fn s1_b_8_phase2_the_gate_arms_are_unchanged() {
        let orch = include_str!("frame/world.rs")
            .replace("\r\n", "\n")
            .split("pub(super) fn update_world_frame")
            .nth(1)
            .expect("the world-branch orchestrator")
            .split("pub(super) fn draw_half_pre_snapshots")
            .next()
            .expect("the spine follows the orchestrator")
            .to_string();
        let arm = |gate: &str, call: &str| {
            format!("{} {{\n            self.{};\n        }}", gate, call)
        };
        assert!(
            orch.contains(&arm("if !paused", "weather_cluster(dt, screen)")),
            "weather under !paused"
        );
        assert!(
            orch.contains(&arm("if !paused", "ambience_phase(dt)")),
            "ambience under !paused"
        );
        assert!(
            orch.contains(&arm("if !paused", "map_phase(dt)")),
            "map under !paused"
        );
        assert!(
            orch.contains(&arm("if step_allowed", "editing_net_phase(dt, screen)")),
            "editing/net under the step gate"
        );
        assert!(
            orch.contains(&arm("if step_allowed", "camera_scene_tail(dt, screen)")),
            "the camera walks + scene_state under the step gate"
        );
        assert!(
            orch.contains(&arm("if !paused", "chat_phase()")),
            "the chat timers decay under the pause gate only (Main.cs:17282 is post-step-gate)"
        );
    }

    /// S1-B-8 phase 2: the duplicated `update_camera_at_draw` pair moved into
    /// `draw_half_pre_snapshots` verbatim — both calls, no dedupe (the plan's
    /// constraint; the dedupe needs its own pixel-identity proof).
    #[test]
    fn s1_b_8_phase2_the_camera_at_draw_pair_is_preserved() {
        let spine = include_str!("frame/world.rs")
            .split("pub(super) fn draw_half_pre_snapshots")
            .nth(1)
            .expect("the pre-snapshot spine fn");
        let calls = spine
            .matches("crate::world_sim::camera::update_camera_at_draw")
            .count();
        assert_eq!(
            calls, 2,
            "the duplicated camera-at-draw pair is preserved verbatim in the spine"
        );
        // The orchestrator's own body carries no camera walk (they all live in
        // the spine now).
        let src = include_str!("frame.rs");
        let frame_body = src
            .split("pub(crate) fn frame(&mut self)")
            .nth(1)
            .expect("the orchestrator")
            .split("pub(crate) fn finish_bench")
            .next()
            .expect("the epilogue follows the orchestrator");
        assert_eq!(
            frame_body.matches("update_camera_at_draw").count(),
            0,
            "frame() itself no longer carries the camera walks"
        );
    }

    /// S1-B-8 phase 3: the draw region (the pre-snapshot locals, the
    /// destructure and the gpu block) lives in `frame/draw.rs::draw_region`,
    /// moved verbatim - the `draw_phase` markers and the S1-B-7 order pin
    /// scan it (repointed), and `frame()` consumes the returned stats.
    #[test]
    fn s1_b_8_phase3_the_draw_region_lives_in_frame_draw() {
        let draw = include_str!("frame/draw.rs");
        assert!(
            draw.contains("pub(super) fn draw_region("),
            "the draw region fn exists in frame/draw.rs"
        );
        for marker in ["sky", "world", "rain", "wires", "projectiles"] {
            assert!(
                draw.contains(&format!("draw_phase(\"{marker}\")")),
                "the {marker} marker moved with the draw region"
            );
        }
        assert!(
            draw.contains("let App {"),
            "the destructure moved with the region (no name rewriting)"
        );
        assert!(
            draw.contains("if let (Some(gpu), Some(renderer)) ="),
            "the gpu/renderer gate moved with the region"
        );
        assert!(
            draw.contains("return None;"),
            "the early return yields None (the epilogue skips, as before)"
        );
        let src = include_str!("frame.rs");
        assert!(
            src.contains("let (stats, tile_stats) = match self.draw_region("),
            "frame() consumes draw_region's stats for the epilogue"
        );
    }

    /// S1-B-8 phase 3: the orchestrator is thin - its body is the phase-list
    /// calls plus the epilogue, and the draw region's heavy body lives in the
    /// module (the file's centre of gravity moved with it).
    #[test]
    fn s1_b_8_phase3_the_orchestrator_is_a_phase_list() {
        let src = include_str!("frame.rs");
        let frame_body = src
            .split("pub(crate) fn frame(&mut self)")
            .nth(1)
            .expect("the orchestrator")
            .split("pub(crate) fn finish_bench")
            .next()
            .expect("the epilogue follows the orchestrator");
        for call in [
            "self.frame_head(now);",
            "self.frame_shared_setup(dt);",
            "self.frame_focus_pause();",
            "self.update_world_frame(dt, screen);",
            "self.draw_half_pre_snapshots(dt);",
            "self.draw_region(",
        ] {
            assert!(
                frame_body.contains(call),
                "the orchestrator calls {call}"
            );
        }
        assert!(
            !frame_body.contains("let App {"),
            "the destructure lives in frame/draw.rs"
        );
        assert!(
            !frame_body.contains("if let (Some(gpu), Some(renderer)) ="),
            "the gpu gate lives in frame/draw.rs"
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

/// The dust pool's draw seam over the port's splitmix u64 stream state
/// (S1-E-1 phase 2): the same state the client rolls — no new streams or
/// instances — fed straight to `DustPool::new_dust`. One splitmix step per
/// draw, the C# `Next` conversions (`UnifiedRandom.cs:120-141`) over the
/// 53-bit sample.
pub(crate) struct SplitmixRand<'a>(pub(crate) &'a mut u64);

impl SplitmixRand<'_> {
    /// One draw's `[0,1)` sample: one `next_rand` step, 53 fraction bits.
    fn sample(&mut self) -> f64 {
        (next_rand(self.0) >> 11) as f64 / (1u64 << 53) as f64
    }
}

impl tr_core::particles::DustRng for SplitmixRand<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        (self.sample() * max as f64) as i32
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        (self.sample() * (max - min) as f64) as i32 + min
    }
    fn next_float(&mut self) -> f32 {
        self.sample() as f32
    }
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
