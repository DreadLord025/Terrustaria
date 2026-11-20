//! The winit event loop — `impl ApplicationHandler for App`: the window
//! and renderer creation (`resumed`), the input routes (`window_event`)
//! and the continuous redraw pump (`about_to_wait`). (A7: extracted from
//! `main.rs` whole.)

use std::sync::Arc;
use std::time::Instant;

use glam::Vec2;
use tr_core::TILE_PX;
use tr_render::{Gpu, SpriteRenderer};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::app::flow::FrontScreenBehaviours;
use crate::app::App;
use crate::map;
use crate::menu;
use crate::npc::{Npc, NpcKind};
use crate::ui::pointer::PointerEvent;

/// The port's stand-in for the `UIElement` reference `UserInterface.Update`
/// dispatches the wheel to (`UserInterface.cs:221`): the immediate-mode scroll
/// surfaces are hit-tested by the callers, so the machine only needs to know
/// **that** an element is under the pointer, not which one.
const SCROLL_UI: u32 = 0;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut attrs = Window::default_attributes()
            .with_title("Terraria-Rust")
            .with_inner_size(PhysicalSize::new(1600, 900));
        // Window/taskbar icon, generated from the game logo by `build.rs`.
        if let Ok(img) =
            image::load_from_memory(include_bytes!(concat!(env!("OUT_DIR"), "/icon.png")))
        {
            let rgba = img.to_rgba8();
            if let Ok(icon) =
                winit::window::Icon::from_rgba(rgba.clone().into_raw(), rgba.width(), rgba.height())
            {
                attrs = attrs.with_window_icon(Some(icon));
            }
        }
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));

        let gpu = pollster::block_on(Gpu::new(window.clone())).expect("gpu init");
        let atlas = self.atlas.as_ref().expect("atlas");
        // The BC2 font pages upload only when the adapter has the BC feature;
        // without it the built-in bitmap font takes over (GL debug path).
        let font_atlas = if gpu
            .features
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
        {
            self.font_atlas.as_ref()
        } else {
            if self.font_atlas.is_some() {
                log::warn!(
                    "adapter lacks TEXTURE_COMPRESSION_BC; text falls back to the bitmap font"
                );
            }
            None
        };
        let mut renderer = SpriteRenderer::new(&gpu, atlas, 8, font_atlas);
        // The dye pass table is static vanilla data (`DyeInitializer`'s
        // binding constants, deduped in `dye::dye_params`) — one upload at
        // startup, like the C# `Load()` bakes the shader parameters once.
        renderer.set_dye_params(
            &gpu,
            &crate::dye::dye_params()
                .iter()
                .map(|p| tr_render::sprite::DyeParam {
                    mode: p.mode,
                    _pad: p._pad,
                    color: p.color,
                    secondary: p.secondary,
                })
                .collect::<Vec<_>>(),
        );

        let fmt = gpu.config.format;
        let info = gpu.adapter.get_info();
        log::info!("renderer: {} [{:?}] | {:?}", info.name, info.backend, fmt);

        self.camera = Vec2::new(
            self.world.width as f32 * TILE_PX * 0.5,
            self.world.spawn_y as f32 * TILE_PX,
        );
        if let Some((x, y)) = crate::env::camera_pin() {
            self.camera = Vec2::new(x * TILE_PX, y * TILE_PX);
        }
        self.zoom = self.settings.zoom.clamp(1.0, 2.0);
        if let Some(z) = crate::env::zoom_pin() {
            // Main.cs:16801-16815 — vanilla GameZoomTarget clamps to 1.0..2.0;
            // TR_ZOOM must not widen it (S1-B-5).
            self.zoom = z.clamp(1.0, 2.0);
            self.zoom_pinned = true;
        }
        if self.follow_player {
            if let Some(p) = &self.player {
                let (cx, cy) = p.center();
                self.camera = Vec2::new(cx, cy);
            }
        }
        self.last_frame = Instant::now();

        self.gpu = Some(gpu);
        self.renderer = Some(renderer);
        // M1.4-BG-18 phase 2 harness: `TR_POST=1` routes frames through the post
        // pipeline (passthrough until phase 3 ports filter passes). The noise
        // image rides along so filter bindings hold the real content texture.
        if crate::env::post_capture() {
            let gpu = self.gpu.as_ref().expect("gpu");
            let mut post = tr_render::PostPipeline::new(gpu);
            post.resize(gpu, gpu.config.width, gpu.config.height);
            self.noise_tex = crate::app::frame::load_noise_texture(gpu);
            self.water_mask = Some(tr_render::WaterMask::new(&gpu.device));
            self.post = Some(post);
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.save_world();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size);
                }
            }
            // `FocusHelper.IsSelectedApplication = Main.instance.IsActive`
            // (`FocusHelper.cs:135-136`, the Windows form refinement aside):
            // the window's focus is the pause source's input. `frame()`'s
            // focus-pause arm (`app/frame.rs`) consumes it with
            // `SettingPlayWhenUnfocused` (`FocusHelper.cs:137-138`).
            WindowEvent::Focused(focused) => {
                self.window_focused = focused;
            }
            WindowEvent::RedrawRequested => {
                self.frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.keys.insert(code);
                        // `DoUpdate_Enter_ToggleChat` + `DoUpdate_HandleChat`
                        // (`Main.cs:18264-18372`): the chat open/close/edit
                        // chain eats the keyboard before the world routes.
                        // Extracted like `world_key` so the chain (and its
                        // sound cues) is reachable headlessly.
                        if self.chat_key(code, event.text.as_deref()) {
                            return;
                        }
                        // `UIAchievementsMenu`'s Back (`GoBackClick` sets
                        // `Main.menuMode = 0`): Escape closes the overlay
                        // before the legacy menu sees the key (M6-ACH-2).
                        if self.achievements_open && matches!(code, KeyCode::Escape) {
                            self.achievements_open = false;
                            self.achievements_categories = [true; 4];
                            self.achievements_scroll = 0.0;
                            return;
                        }
                        if self.menu.is_some() {
                            if let Some(text) = event.text.as_ref() {
                                if let Some(m) = self.menu.as_mut() {
                                    for ch in text.chars() {
                                        if !ch.is_control() {
                                            m.character(ch);
                                        }
                                    }
                                }
                            }
                            let action = self
                                .menu
                                .as_mut()
                                .map(|m| m.key(code))
                                .unwrap_or(menu::Action::None);
                            self.apply_menu_action(action, event_loop);
                            return;
                        }
                        if let Some(front) = self.front.clone() {
                            // menuMode 888 takes no world keyboard routes
                            // (the screens are mouse-driven; the gamepad snap
                            // points are out of scope, §J.3) — the world
                            // controls stay parked while a front screen is
                            // live. The route goes through the screen
                            // behaviours; the keyboard session is the one
                            // text box.
                            front.key(self, &event);
                            return;
                        }
                        self.world_key(code);
                    } else {
                        self.keys.remove(&code);
                        // The smart-cursor hold mode (`Player.cs:28835-28839`):
                        // releasing the key turns the wanted state off when the
                        // toggle mode (`Main.cSmartCursorModeIsToggleAndNotHold`,
                        // `Main.cs:67178`, the port's persisted
                        // `settings.smart_cursor_toggle`) is off.
                        if self.is_bound(crate::input_map::Action::Smart, code)
                            && !self.settings.smart_cursor_toggle
                        {
                            self.smart_cursor_used = false;
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Vanilla: the wheel scrolls the hotbar (`HotbarPlus`/`HotbarMinus`);
                // zoom is only changed by the View Zoom keys or the settings screen.
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 60.0,
                };
                // The F3 workbench's list scroll (the panel consumes the wheel
                // while it is open and the pointer is over it).
                #[cfg(feature = "debug")]
                if tr_debug::in_rect(self.mouse_pos, self.debug_panel.layout(Vec2::ZERO).panel)
                    && crate::debug_panel::handle_scroll(self, d)
                {
                    return;
                }
                // `UIAchievementsMenu`'s list scroll (M6-ACH-2): the wheel
                // moves the overlay's rows while it is open, before the
                // hotbar's `HotbarPlus`/`HotbarMinus` cycle claims it.
                if self.achievements_open {
                    // `UserInterface.Update`'s scroll arm (`:219-223`): the
                    // machine owns `PlayerInput.ScrollWheelDeltaForUI`
                    // (accumulated, then consumed once). The overlay's `UIList`
                    // is the element under the pointer.
                    self.pointer.queue_scroll(d.round() as i32);
                    if matches!(
                        self.pointer.scroll_wheel(Some(SCROLL_UI)),
                        Some(PointerEvent::ScrollWheel { .. })
                    ) {
                        self.scroll_achievements(d);
                    }
                    return;
                }
                // `UIList.ScrollWheel` through `UserInterface.Update`'s wheel
                // arm (`:219-222`): while a list screen is live the `UIList`
                // under the pointer owns the notch — `ViewPosition -=
                // evt.ScrollWheelValue` (`UIList.cs:107-114`). The C# consumes
                // the accumulator whatever the hit, so the hotbar arm below
                // never sees a front-screen wheel.
                if self.front_wheel(d.round() as i32) {
                    return;
                }
                if self.map.as_ref().is_some_and(|m| m.open) {
                    // `Player.cs:25190-25197`: while the map is fullscreen the wheel
                    // zooms it instead of cycling the hotbar.
                    let (sw, _) = self.screen_size();
                    if let Some(m) = self.map.as_mut() {
                        m.scale = map::map_scale_clamp(
                            sw,
                            self.world.width,
                            map::map_wheel_scale(m.scale, d * 120.0),
                        );
                    }
                } else {
                    let step = d.round() as i32;
                    // The UI-element scroll surfaces are the elements the
                    // pointer is over: `UIList.ScrollWheel` (`UIList.cs:107-114`)
                    // for the emotes wheel and `Main.DoScrollingInInventory`'s
                    // chest arm (`Main.cs:17435-17452`) for the slot window. The
                    // machine consumes the delta once (`:219-223`) and hands the
                    // consumed value back here.
                    let over_ui = step != 0
                        && self.menu.is_none()
                        && self.front.is_none()
                        && (self.emotes.menu_open || self.inventory_open);
                    self.pointer.queue_scroll(step);
                    let ui_delta = match self.pointer.scroll_wheel(over_ui.then_some(SCROLL_UI)) {
                        Some(PointerEvent::ScrollWheel { delta }) => delta,
                        _ => 0,
                    };
                    // `UIList.ScrollWheel`: the emotes wheel scrolls while it is
                    // open, clamped at the roster's end (`UIScrollbar.cs:37-47`).
                    if ui_delta != 0
                        && self.menu.is_none()
                        && self.front.is_none()
                        && self.emotes.menu_open
                    {
                        let (sw, sh) = self.screen_size();
                        let grid = crate::ui::emotes::grid_rect(Vec2::new(sw, sh));
                        self.emotes
                            .scroll_by(ui_delta, crate::ui::emotes::visible_rows(&grid));
                    }
                    // `Main.DoScrollingInInventory`'s chest arm
                    // (`Main.cs:17435-17452`): the wheel scrolls the chest
                    // slot window while the inventory is open and the pointer
                    // is inside the grid (the plan-amendment insertion #10).
                    if ui_delta != 0
                        && self.menu.is_none()
                        && self.front.is_none()
                        && self.inventory_open
                    {
                        crate::ui::chest::scroll(self, ui_delta, self.mouse_pos);
                    }
                    if step != 0
                        && self.menu.is_none()
                        && self.front.is_none()
                        && !self.inventory_open
                    {
                        // S1-P-i54 — `HandleHotbarControls` (`Player.cs:
                        // 31766-31796`): `num += ScrollWheelDelta / -120`
                        // (`:31790`) — wheel-UP (+120) steps BACKWARD — and
                        // the offset applies to the HOTBAR latch
                        // (`selectedItemState.Hotbar`, :31794), not the live
                        // slot — an overridden selection (smart-select)
                        // does not shift the cycle base. The request
                        // buffers (`Select`); the apply runs on the player
                        // tick (`selectedItemState.Update`, `:25218`). The
                        // 12 cue plays at the request (`Select`, `:506`).
                        let target = crate::hotbar::clamp_hotbar_offset(
                            self.selection.hotbar as i32 - step,
                        );
                        if self.selection.select(self.selected, target, false) {
                            self.audio.play("Menu_Tick");
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Middle => self.mouse_down = pressed,
                    MouseButton::Left => {
                        // The flag belongs to whichever side owns the input;
                        // while a legacy page is live the 888 side is closed,
                        // so a release here must not leak a stale `true` into
                        // a restored front screen.
                        if self.menu.is_some() {
                            self.front_mouse_down = false;
                        }
                        if let Some(m) = self.menu.as_mut() {
                            // `Main.mouseLeft` (`Main.cs:51101`): the value
                            // bars drag off this flag (`Main.cs:51533-51576`),
                            // held between the press and release events.
                            m.mouse_down = pressed;
                            if pressed {
                                // `UIAchievementsMenu` rides over the legacy menu
                                // (`Main.MenuUI.SetState`), so its Back and filter
                                // toggles take the press first (M6-ACH-2).
                                if pressed && self.achievements_open {
                                    self.ui_left_press = true;
                                    self.handle_achievements_clicks();
                                    return;
                                }
                                m.update_hover(self.mouse_pos);
                                let action = m.click();
                                // Bug 17: the legacy menu's click tick —
                                // `Main.DoStatefulTickSound` (`Main.cs:3244`,
                                // `:3256`) / the settings rows' activation.
                                if !matches!(action, menu::Action::None) {
                                    self.audio.play("Menu_Tick");
                                }
                                self.apply_menu_action(action, event_loop);
                            }
                            return;
                        }
                        if self.front.is_some() {
                            // The 888 screens are click-driven: the press
                            // routes through the card-hit table (`Main.
                            // MouseUI`'s click while `MenuUI` is live). The
                            // held flag rides along for the creation screen's
                            // HSL drags (`Main.mouseLeft`, `Main.cs:51101`).
                            self.front_mouse_down = pressed;
                            if !pressed {
                                // `UIScrollbar.LeftMouseUp` (`UIScrollbar.cs:145-149`):
                                // the drag latch clears on release.
                                self.front_scroll_drag = None;
                            }
                            // `UIScrollbar.LeftMouseDown` (`UIScrollbar.cs:126-143`):
                            // a press on the attached bar grabs the handle or
                            // jumps the track — `evt.Target == this` keeps it
                            // off the cards, so no click table runs.
                            if pressed
                                && !self.pointer.clicks_disabled()
                                && self.front_scrollbar_press()
                            {
                                return;
                            }
                            // `UserInterface.HandleClick` (`:230-255`)'s front
                            // path: the machine owns the press/release →
                            // click/`DoubleClick` edges. The existing single-click
                            // dispatch keeps firing on the press, exactly as
                            // before; Phase 2 feeds it the framework's hit id so
                            // the double-click a card produces is real.
                            let now = self.anim_time as f64 * 1000.0;
                            // The element under the pointer — the framework's
                            // `GetElementAt` (`UIElement.cs:324-352`), the id
                            // the machine's `LastClicked` compares.
                            let hit = self.front_hit();
                            if pressed {
                                if !self.pointer.clicks_disabled() {
                                    self.pointer_event = self.pointer.press(
                                        now,
                                        crate::ui::pointer::PointerButton::Left,
                                        hit,
                                    );
                                }
                                self.handle_front_click();
                            } else {
                                self.pointer_event = self
                                    .pointer
                                    .release(now, crate::ui::pointer::PointerButton::Left, hit);
                            }
                            return;
                        }
                        if !pressed {
                            #[cfg(feature = "debug")]
                            crate::debug_panel::end_drag(self);
                            // S1-P-i54 — `TriggersSet.CopyInto`
                            // (`TriggersSet.cs:844-847`): the left release
                            // clears `Main.blockMouse`, so the next press
                            // arms `controlUseItem` again.
                            self.block_mouse = false;
                        }
                        self.mine_down = pressed;
                        if !pressed {
                            // `Main.mouseLeftRelease` — the release edge the
                            // achievement popup's click reads
                            // (`InGamePopups.cs:124`), consumed per frame.
                            self.ui_left_release = true;
                        }
                        if pressed {
                            self.ui_left_press = true;
                            // The debug panel's click (the self-contained
                            // cheat-sheet — the panel consumes the clicks
                            // inside its rect).
                            #[cfg(feature = "debug")]
                            crate::debug_panel::handle_click(self, self.mouse_pos);
                        }
                    }
                    MouseButton::Right => {
                        self.place_down = pressed;
                        if pressed {
                            self.ui_right_press = true;
                        } else {
                            // `Main.mouseRightRelease` — the release edge the
                            // buff bar's removal reads (`Main.cs:43342`).
                            self.ui_right_release = true;
                        }
                    }
                    // `PlayerInput.MouseInfo.XButton1 == ButtonState.Pressed`'s
                    // held state (`WorldUpdateStepper.cs:48` — the step key's
                    // mouse arm; `GetStepKeyState` ORs it with End/NumpadAdd).
                    // Nothing reads it until the Phase-2 stepper lands; the
                    // held flag tracks the physical button either way.
                    MouseButton::Back => {
                        self.mouse_back_held = pressed;
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = Vec2::new(position.x as f32, position.y as f32);
                // The debug panel's drag (the title-bar drag moves the
                // panel).
                #[cfg(feature = "debug")]
                if crate::debug_panel::handle_drag(self, p) {
                    // The drag consumed the move.
                }
                // A `TR_SHOT` capture without `TR_MOUSE` parks the pointer
                // offscreen and ignores real motion (`deterministic_pointer_park`).
                if crate::env::mouse_pin().is_none()
                    && !crate::env::deterministic_pointer_park()
                {
                    self.mouse_pos = p;
                }
                // The menu reads the pinned pointer when `TR_MOUSE` is set, so
                // its hover and value-bar drags stay deterministic.
                let pointer = self.mouse_pos;
                if let Some(m) = self.menu.as_mut() {
                    let before = m.hover;
                    m.update_hover(pointer);
                    // Bug 17: the legacy menu's hover tick (`DoStatefulTickSound`).
                    if m.hover != before && m.hover.is_some() {
                        self.audio.play("Menu_Tick");
                    }
                }
                // `UIScrollbar.DrawSelf`'s drag (`UIScrollbar.cs:109-112`):
                // while the handle is held the view follows the pointer.
                if self.front_scroll_drag.is_some() {
                    self.front_scrollbar_drag();
                }
                // Bug 17: the front-end hover edge — `FadedMouseOver`'s
                // menu tick (`UICharacterSelect.cs:124`, `UIWorldSelect.cs:126`)
                // fires once when the pointer enters an element.
                if self.front.is_some() {
                    let hit = self.front_hit();
                    if hit != self.front_hover_hit {
                        if hit.is_some() {
                            self.audio.play("Menu_Tick");
                        }
                        self.front_hover_hit = hit;
                    }
                } else {
                    self.front_hover_hit = None;
                }
                // `EmoteButton.MouseOver`/`MouseOut` (`EmoteButton.cs:77-88`):
                // sound 12 on the cell-enter edge — the same Bug-17 hover
                // cue the front screens fire. The highlight itself is
                // draw-derived (`ui::emotes::draw`'s `card_at`).
                if self.emotes.menu_open && self.menu.is_none() && self.front.is_none() {
                    let (sw, sh) = self.screen_size();
                    let hit = crate::ui::emotes::card_at(
                        Vec2::new(sw, sh),
                        self.mouse_pos,
                        self.emotes.scroll,
                    );
                    if hit != self.emotes.hover {
                        if hit.is_some() {
                            self.audio.play("Menu_Tick");
                        }
                        self.emotes.hover = hit;
                    }
                } else {
                    self.emotes.hover = None;
                }
                // The middle-button world-camera drag is suppressed while the map is
                // fullscreen, like the other world controls (`Player.cs:24798`).
                if self.mouse_down && !self.map.as_ref().is_some_and(|m| m.open) {
                    let delta = p - self.last_mouse;
                    self.camera -= delta / self.zoom;
                }
                self.last_mouse = p;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

impl App {
    /// The world-side key routes — `Main.UpdateInput`'s keyboard pass as the
    /// port splits it: the Escape close chain, the map key (`Main.cMapFull`,
    /// `M`), the hotbar digits, the debug panel and the smart-cursor toggle.
    ///
    /// Extracted from `window_event`'s `KeyboardInput` arm (the chat/menu/front
    /// overlays eat the keys first and stay in the event handler, as in
    /// `Main.DoUpdate_HandleChat`) so the chain is reachable headlessly: the
    /// winit `KeyEvent` cannot be built without a live window, but every arm
    /// below is pure `App` state. The method runs on the press only — winit
    /// delivers key repeats as separate `Pressed` events, which is the port's
    /// edge convention (the vanilla `release*` machine,
    /// `Player.cs:25075-25086`).
    /// Is `code` the key bound to `action` (`Main.cUp`..`Main.cMount`,
    /// `Main.cs:67163-67176`)?
    pub(crate) fn is_bound(&self, action: crate::input_map::Action, code: KeyCode) -> bool {
        crate::input_map::is_bound(&self.keybinds, action, code)
    }

    /// Is `code` the key bound to a fullscreen-map row (`Main.cMapStyle`..
    /// `Main.cMapAlphaDown`, `Main.cs:67190-67195`)?
    pub(crate) fn is_map_bound(&self, action: crate::input_map::MapAction, code: KeyCode) -> bool {
        self.map_binds[action.index()].as_str() == crate::input_map::key_name(code)
    }

    /// S1-P-i54 — one hotbar digit press (`Player.cs:25095-25134` +
    /// `:25174-25177`): the digit latches `num7` and `Select` buffers the
    /// request; the apply + use-timer reset run on the player tick
    /// (`selectedItemState.Update`, `:25218`). The 12 cue plays at the
    /// request (`Select`, `:506`). Vanilla has NO shop-buy on the digit
    /// keys — the shop buys ride `ItemSlot.HandleShopSlot`'s mouse arms
    /// (`ItemSlot.cs:1220`/`:1650`); the port's old "1 buys stock slot 0"
    /// was an invention and is gone (the click-buy stays the shop's
    /// surface). The chat/sign/chest-edit gate (`:25093`) is the caller's:
    /// those overlays eat the key before `world_key` runs.
    fn hotbar_digit(&mut self, slot: usize) {
        if self.selection.select(self.selected, slot, false) {
            self.audio.play("Menu_Tick");
        }
    }

    /// The chat keyboard chain — `DoUpdate_Enter_ToggleChat` +
    /// `DoUpdate_HandleChat` (`Main.cs:18264-18372`). Extracted from
    /// `window_event`'s `KeyboardInput` arm like [`Self::world_key`] so the
    /// chain (and its sound cues) is reachable headlessly. Returns `true`
    /// when the key was consumed (the open chat eats every key; the closed
    /// chat eats only Enter). The world controls stay live like the source.
    ///
    /// Sound sites (W4-g21): `:18270` the open plays 10 (`Menu_Open`) BEFORE
    /// `OpenPlayerChat`; `:18349` any text change plays 12 (`Menu_Tick`);
    /// `:18370` the submit tail plays 11 (`Menu_Close`) AFTER
    /// `ClosePlayerChat`. The Escape close (`:18332-18335`) plays nothing.
    pub(crate) fn chat_key(&mut self, code: KeyCode, text: Option<&str>) -> bool {
        if self.chat.open {
            match code {
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    // `ChatHelper.SendChatMessageFromClient`
                    // (`ChatHelper.cs:72-79`): a joining client sends the
                    // line to the host, which runs the command processor
                    // and relays the result back as `82` text modules.
                    let text = self.chat.text.clone();
                    let is_client = self.is_client();
                    let before = self.chat.messages.len();
                    if is_client {
                        self.send_chat_message("", &text);
                    } else {
                        let name = self.player_name.clone();
                        // The chat commands draw the bare `Main.rand`
                        // stand-in (the C# chat path runs outside every
                        // swap); they also read the player state
                        // (`DeathCommand`'s counters).
                        self.chat.send_with(
                            &text,
                            &name,
                            self.rngs.global(),
                            &mut self.emotes,
                            self.player.as_ref(),
                        );
                    }
                    // The host relays the lines its own submit produced
                    // (slot 0 is the host's author). The borrow dance: the
                    // server handle borrows `self.net`, so the lines are
                    // collected first.
                    if !is_client {
                        let produced = self
                            .chat
                            .messages
                            .iter()
                            .take(self.chat.messages.len().saturating_sub(before))
                            .map(|l| (l.text.clone(), l.color))
                            .collect::<Vec<_>>();
                        if !produced.is_empty() {
                            if let crate::app::NetRole::Host(server) = &self.net {
                                use crate::app::netmodules::{
                                    module_id, ModuleBody, ServerMessage,
                                };
                                for (text, color) in produced {
                                    let sm = ServerMessage {
                                        author: 0,
                                        text,
                                        color: crate::net::Rgb(color[0], color[1], color[2]),
                                    };
                                    server.broadcast(&crate::net::S2c::NetModule {
                                        module: module_id::TEXT,
                                        body: sm.encode(),
                                    });
                                }
                            }
                        }
                    }
                    self.chat.text.clear();
                    self.chat.open = false;
                    // `Main.cs:18370` — the send tail plays sound 11 AFTER
                    // `ClosePlayerChat` + `chatRelease = false`.
                    self.audio.play("Menu_Close");
                }
                KeyCode::Escape => {
                    self.chat.text.clear();
                    self.chat.open = false;
                }
                KeyCode::Backspace => {
                    // The pre-edit line — the change tick
                    // (`Main.cs:18336-18350`) reads it before
                    // `GetInputText`'s trim/append.
                    let before = self.chat.text.clone();
                    self.chat.text.pop();
                    // `:18347-18350` — `text != Main.chatText` plays the
                    // MenuTick (sound 12).
                    if self.chat.text != before {
                        self.audio.play("Menu_Tick");
                    }
                }
                _ => {}
            }
            if let Some(text) = text {
                // The same change tick for the append half
                // (`GetInputText`, `Main.cs:18337`).
                let before = self.chat.text.clone();
                for ch in text.chars() {
                    if !ch.is_control() {
                        self.chat.text.push(ch);
                    }
                }
                // `:18347-18350` — sound 12 on any change.
                if self.chat.text != before {
                    self.audio.play("Menu_Tick");
                }
            }
            return true;
        }
        // `Main.chatRelease && !drawingPlayerChat` → `OpenPlayerChat`
        // (`Main.cs:18268-18287`): Enter opens the chat.
        if matches!(code, KeyCode::Enter | KeyCode::NumpadEnter) {
            // `DoUpdate_Enter_ToggleChat` (`Main.cs:18270`): sound 10 plays
            // BEFORE `OpenPlayerChat` + the `chatText = ""` reset.
            self.audio.play("Menu_Open");
            self.chat.open = true;
            self.chat.text.clear(); // `:18272`
            return true;
        }
        false
    }

    pub(crate) fn world_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Escape => {
                // The bestiary eats the Escape first (`IngameFancyUI`'s
                // GoBack), the emotes wheel next, then the fullscreen map —
                // `Player.ToggleInv` (`Player.cs:4973-4978`) closes
                // `Main.mapFullscreen` and returns before the inventory
                // toggle, so the vanilla close order is map → inventory →
                // pause. The world is saved on window close and with F9.
                if self.ingame_options.open {
                    // BUG-11: `IngameOptions.Close` (`IngameOptions.cs:84`):
                    // the window shuts and persists its settings
                    // (`Main.SaveSettings`), the vanilla's first close arm.
                    self.ingame_options.close();
                    let _ = self.settings.save();
                } else if self.bestiary_open {
                    self.bestiary_open = false;
                    self.bestiary_selected = None;
                } else if self.emotes.menu_open {
                    self.emotes.close();
                } else if self.map.as_ref().is_some_and(|m| m.open) {
                    // `Player.ToggleInv`'s first arm: Escape closes the
                    // fullscreen map and consumes the press — no inventory
                    // toggle this frame, like the source's `return`.
                    if let Some(m) = self.map.as_mut() {
                        m.set_open(false);
                    }
                } else {
                    self.inventory_open = !self.inventory_open;
                }
            }
            KeyCode::F1 => self.debug = !self.debug,
            KeyCode::F4 => self.debug_overlay = !self.debug_overlay,
            KeyCode::F3 => {
                #[cfg(feature = "debug")]
                crate::debug_panel::handle_key(self, KeyCode::F3);
            }
            KeyCode::F2 => {
                // `DoUpdate_F9_ToggleLighting` (`Main.cs:18481`): sound 12
                // plays BEFORE `Lighting.NextLightMode()` (`:18482`). The
                // port's key map binds the cycle to F2 (the help overlay's
                // row, `render/debug.rs`; the vanilla Shift+F9 chord is not
                // modelled — F9 is the port's save key).
                self.audio.play("Menu_Tick");
                self.light_mode = self.light_mode.next();
                // `Lighting.NextLightMode` (`Lighting.cs:165-173`): the Mode
                // setter resets `renderCount` and `Lighting.Clear()` zeroes
                // the active engine's presented values — no state leaks
                // across the switch (S1-E-19).
                if let Some(engine) = self.legacy_light.as_mut() {
                    engine.set_mode(self.light_mode);
                    engine.clear();
                }
                self.world_key = None;
                log::info!("light mode: {}", self.light_mode.label());
            }
            KeyCode::F9 => self.save_world(),
            _ if self.is_map_bound(crate::input_map::MapAction::Full, code) => {
                // `Main.TryOpeningFullscreenMap` (`Player.cs:29105`) on
                // `Main.cMapFull` (`Main.cs:67192`, default `M`): the key only
                // toggles the map when `Main.mapEnabled`.
                if self.settings.map_enabled {
                    if let Some(m) = self.map.as_mut() {
                        let open = !m.open;
                        m.set_open(open);
                    }
                }
            }
            _ if self.is_bound(crate::input_map::Action::Smart, code) => {
                // The smart-cursor toggle (`Player.TryToToggleSmartCursor`,
                // `Player.cs:24867`/`:28814-28820`): the press edge flips
                // `Main.SmartCursorWanted_Mouse` while
                // `Main.cSmartCursorModeIsToggleAndNotHold` (`Main.cs:67178`,
                // the port's persisted `settings.smart_cursor_toggle`) selects
                // the toggle mode — its vanilla default. The runtime flag is
                // `App.smart_cursor_used` (`Main.cs:2011`, default false); the
                // persisted setting is never read as the state.
                //
                // `SoundEngine.PlaySound(12)` (`:28818`), the port's 12 cue.
                self.audio.play("Menu_Tick");
                if self.settings.smart_cursor_toggle {
                    self.smart_cursor_used = !self.smart_cursor_used;
                } else {
                    // The hold mode (`:28835-28839`): the state tracks the key
                    // (the release arm in `window_event` clears it).
                    self.smart_cursor_used = true;
                }
            }
            #[cfg(feature = "debug")]
            _ if crate::debug_panel::handle_key(self, code) => {}
            // S1-P-i54 — `Main.KeyStatus`' digit arms (`Player.cs:25095-25134`):
            // Hotbar1 ALWAYS selects (`Main.cs:31790`'s digit loop buffers the
            // selection; the shop buys are mouse-only, `ItemSlot.cs:1220`/
            // `:1650` — the i44 Digit1-buy invention retired).
            KeyCode::Digit1 => self.hotbar_digit(0),
            KeyCode::Digit2 => self.hotbar_digit(1),
            KeyCode::Digit3 => self.hotbar_digit(2),
            KeyCode::Digit4 => self.hotbar_digit(3),
            KeyCode::Digit5 => self.hotbar_digit(4),
            KeyCode::Digit6 => self.hotbar_digit(5),
            KeyCode::Digit7 => self.hotbar_digit(6),
            KeyCode::Digit8 => self.hotbar_digit(7),
            KeyCode::Digit9 => self.hotbar_digit(8),
            KeyCode::Digit0 => self.hotbar_digit(9),
            // `Main.cInv` (`Main.cs:67175`, default `Escape`): the rebind row
            // for `Player.controlInv`. Escape keeps the close chain above
            // (the vanilla's menu/map close order runs before `ToggleInv`),
            // so only a *different* key takes this arm.
            _ if self.is_bound(crate::input_map::Action::Inventory, code) => {
                self.inventory_open = !self.inventory_open;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::map::WorldMap;

    /// A headless App with the map live and open (`M` while `mapEnabled`),
    /// ready for the world-key chain.
    fn map_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.map = Some(WorldMap::new(&app.world));
        if let Some(m) = app.map.as_mut() {
            m.set_open(true);
        }
        app
    }

    #[test]
    fn escape_closes_the_ingame_options_window_first() {
        // BUG-11: `IngameOptions.Close` (`IngameOptions.cs:84`) is the
        // vanilla close chain's first arm — the options window shuts and the
        // inventory does NOT toggle open on the same press.
        let mut app = flow_app(None, vec![], vec![]);
        app.ingame_options.open = true;
        app.inventory_open = false;
        app.world_key(KeyCode::Escape);
        assert!(!app.ingame_options.open, "Escape closes the options window");
        assert!(!app.inventory_open, "the press is consumed");
    }

    #[test]
    fn escape_falls_through_when_the_options_window_is_shut() {
        // The window shut: Escape keeps its old behaviour (the inventory
        // toggle), so the new arm does not eat the key.
        let mut app = flow_app(None, vec![], vec![]);
        app.ingame_options.open = false;
        app.world_key(KeyCode::Escape);
        assert!(app.inventory_open, "the inventory still toggles open");
    }

    #[test]
    fn escape_closes_the_open_map_before_the_inventory() {
        // `Player.ToggleInv` (`Player.cs:4973-4978`): with the fullscreen map
        // up, the inventory key (`cInv`, Escape) closes the map and returns —
        // the press must not also toggle the inventory open.
        let mut app = map_app();
        assert!(!app.inventory_open);
        app.world_key(KeyCode::Escape);
        assert!(
            !app.map.as_ref().is_some_and(|m| m.open),
            "Escape closes the map"
        );
        assert!(
            !app.inventory_open,
            "the press is consumed: the inventory stays shut"
        );
    }

    #[test]
    fn escape_falls_through_to_the_inventory_when_the_map_is_shut() {
        // The vanilla close order (map → inventory → pause): with the map
        // closed the same key keeps its old behaviour — the inventory toggle.
        let mut app = map_app();
        if let Some(m) = app.map.as_mut() {
            m.set_open(false);
        }
        app.world_key(KeyCode::Escape);
        assert!(app.inventory_open, "the inventory opens with the map shut");
        app.world_key(KeyCode::Escape);
        assert!(!app.inventory_open, "and closes again");
    }

    #[test]
    fn the_map_key_still_toggles_the_map() {
        // `Player.cs:24968-24982` (`cMapFull`, `M`): the map key opens the
        // closed map — the extraction kept the arm.
        let mut app = map_app();
        if let Some(m) = app.map.as_mut() {
            m.set_open(false);
        }
        app.world_key(KeyCode::KeyM);
        assert!(app.map.as_ref().is_some_and(|m| m.open), "M opens the map");
        app.world_key(KeyCode::KeyM);
        assert!(!app.map.as_ref().is_some_and(|m| m.open), "M closes it");
    }

    #[test]
    fn the_smart_cursor_toggle_flips_the_runtime_flag() {
        // `Player.TryToToggleSmartCursor` (`Player.cs:28814-28820`) reads
        // `Player.controlSmart`, i.e. `Main.cSmart` — **LeftControl**
        // (`Main.cs:67175`), not `cTorch`'s LeftShift (`:67173`). The runtime
        // `Main.SmartCursorWanted_Mouse` analogue starts off (the vanilla
        // default, `Main.cs:67179`) and every press edge flips it.
        let mut app = flow_app(None, vec![], vec![]);
        assert!(!app.smart_cursor_used, "the default is off");
        app.world_key(KeyCode::ControlLeft);
        assert!(app.smart_cursor_used, "the press turns it on");
        app.world_key(KeyCode::ControlLeft);
        assert!(!app.smart_cursor_used, "the second press turns it off");
        // The old port bound it to LeftShift, which is `cTorch` — the
        // auto-select row, unported.
        app.world_key(KeyCode::ShiftLeft);
        assert!(!app.smart_cursor_used, "LeftShift is not the smart-cursor key");
    }

    #[test]
    fn m7_ui_5_a_rebound_key_drives_the_action_and_frees_the_old_one() {
        // `Main.cMapFull` (`Main.cs:67192`, default `M`): the row the
        // settings screen edits is the row the input pass reads, so a rebind
        // moves the action instead of adding a second key.
        let mut app = map_app();
        if let Some(m) = app.map.as_mut() {
            m.set_open(false);
        }
        app.settings.map_binds[crate::input_map::MapAction::Full.index()] = "F".into();
        app.apply_settings();
        app.world_key(KeyCode::KeyM);
        assert!(!app.map.as_ref().is_some_and(|m| m.open), "M is no longer the map key");
        app.world_key(KeyCode::KeyF);
        assert!(app.map.as_ref().is_some_and(|m| m.open), "F opens the map now");
        // The inventory row (`Main.cInv`, `:67175`) works the same way.
        let mut app = flow_app(None, vec![], vec![]);
        app.settings.keybinds[crate::input_map::Action::Inventory.index()] = "I".into();
        app.apply_settings();
        app.world_key(KeyCode::KeyI);
        assert!(app.inventory_open, "I toggles the inventory now");
        app.world_key(KeyCode::KeyI);
        assert!(!app.inventory_open);
    }

    #[test]
    fn m7_ui_5_a_movement_rebind_steers_the_player() {
        // `Main.cRight` (`Main.cs:67166`, default `D`): the movement rows are
        // the same table, so rebinding Right to `L` walks the player right and
        // leaves `D` free.
        let mut app = flow_app(None, vec![], vec![]);
        app.settings.keybinds[crate::input_map::Action::Right.index()] = "L".into();
        app.apply_settings();
        let binds = app.keybinds.clone();
        app.keys.insert(KeyCode::KeyL);
        let input = crate::world_sim::player::player_input(&app, &binds, false, false, false);
        assert!(input.right, "L is the right row now");
        app.keys.clear();
        app.keys.insert(KeyCode::KeyD);
        let input = crate::world_sim::player::player_input(&app, &binds, false, false, false);
        assert!(!input.right, "D is free after the rebind");
    }

    // -- W4-g21: the chat chain's sound cues (`Main.cs:18270/:18349/:18370`) --

    /// `DoUpdate_Enter_ToggleChat` (`Main.cs:18270`): the closed chat's Enter
    /// plays sound 10 (`Menu_Open`) BEFORE `OpenPlayerChat`, and resets
    /// `chatText` (`:18272`).
    #[test]
    fn w4_g21_the_chat_open_plays_menu_open() {
        let mut app = flow_app(None, vec![], vec![]);
        app.chat.text = "stale".into();
        let before = app.audio.played_count();
        assert!(app.chat_key(KeyCode::Enter, None), "Enter opens the chat");
        assert!(app.chat.open);
        assert!(app.chat.text.is_empty(), "`chatText = \"\"` (`:18272`)");
        assert_eq!(app.audio.played_count(), before + 1, "one Menu_Open cue");
    }

    /// `DoUpdate_HandleChat` (`Main.cs:18347-18350`): a text change (append
    /// or trim) plays sound 12; an unchanged line plays nothing.
    #[test]
    fn w4_g21_the_chat_edit_ticks_only_on_a_change() {
        let mut app = flow_app(None, vec![], vec![]);
        app.chat.open = true;
        let before = app.audio.played_count();
        // The append half (`GetInputText`).
        assert!(app.chat_key(KeyCode::KeyA, Some("a")));
        assert_eq!(app.chat.text, "a");
        assert_eq!(app.audio.played_count(), before + 1, "the append tick");
        // A control-only input changes nothing — no cue.
        assert!(app.chat_key(KeyCode::KeyB, Some("\u{1}")));
        assert_eq!(app.audio.played_count(), before + 1, "no change, no cue");
        // The trim half (Backspace).
        assert!(app.chat_key(KeyCode::Backspace, None));
        assert!(app.chat.text.is_empty());
        assert_eq!(app.audio.played_count(), before + 2, "the trim tick");
        // An empty-line Backspace changes nothing.
        assert!(app.chat_key(KeyCode::Backspace, None));
        assert_eq!(app.audio.played_count(), before + 2, "no change, no cue");
    }

    /// `DoUpdate_HandleChat` (`Main.cs:18370`): the submit tail plays sound
    /// 11 (`Menu_Close`) AFTER the close; the Escape close (`:18332-18335`)
    /// plays nothing.
    #[test]
    fn w4_g21_the_chat_submit_plays_menu_close_but_escape_is_silent() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.chat.open = true;
        app.chat.text = "hello".into();
        let before = app.audio.played_count();
        assert!(app.chat_key(KeyCode::Enter, None), "the submit consumes");
        assert!(!app.chat.open);
        assert!(app.chat.text.is_empty());
        assert_eq!(app.audio.played_count(), before + 1, "one Menu_Close cue");
        // The Escape close is silent.
        app.chat.open = true;
        let before = app.audio.played_count();
        assert!(app.chat_key(KeyCode::Escape, None));
        assert!(!app.chat.open);
        assert_eq!(app.audio.played_count(), before, "Escape plays nothing");
    }

    /// `DoUpdate_F9_ToggleLighting` (`Main.cs:18481`): the lighting cycle
    /// plays sound 12 BEFORE `Lighting.NextLightMode()`. The port's key map
    /// binds the cycle to F2 (the help overlay's row).
    #[test]
    fn w4_g21_the_lighting_cycle_ticks_before_the_mode_switch() {
        let mut app = flow_app(None, vec![], vec![]);
        let mode_before = app.light_mode;
        let before = app.audio.played_count();
        app.world_key(KeyCode::F2);
        assert_eq!(app.light_mode, mode_before.next(), "the mode cycled");
        assert_eq!(app.audio.played_count(), before + 1, "one Menu_Tick cue");
    }
}

/// The EoW presence check for the debug spawn (any active segment).
fn app_has_eow(app: &App) -> bool {
    app.npcs
        .iter()
        .any(|n| n.kind.is_worm() && n.worm_id != 0 && n.hp > 0)
        && app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::EaterOfWorldsHead && n.hp > 0)
}
