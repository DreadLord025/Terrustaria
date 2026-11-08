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
use crate::app::{deterministic_pointer_park, mouse_pin, App};
use crate::item::HOTBAR;
use crate::map;
use crate::menu;
use crate::npc::{Npc, NpcKind};

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
        if let Ok(cam) = std::env::var("TR_CAM") {
            let mut it = cam.split(',').map(|v| v.trim().parse::<f32>());
            if let (Some(Ok(x)), Some(Ok(y))) = (it.next(), it.next()) {
                self.camera = Vec2::new(x * TILE_PX, y * TILE_PX);
            }
        }
        self.zoom = self.settings.zoom;
        if let Ok(z) = std::env::var("TR_ZOOM") {
            if let Ok(z) = z.parse::<f32>() {
                self.zoom = z.clamp(0.25, 8.0);
                self.zoom_pinned = true;
            }
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
        if std::env::var_os("TR_POST").is_some() {
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
            WindowEvent::RedrawRequested => {
                self.frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.keys.insert(code);
                        // `DoUpdate_HandleChat` (`Main.cs:18318-18372`): the
                        // open chat eats the keyboard before the world
                        // routes — Enter sends + closes (`inputTextEnter`),
                        // Escape closes (`inputTextEscape`), Backspace
                        // trims, printable text appends (`GetInputText`).
                        // The world controls stay live like the source.
                        if self.chat.open {
                            match code {
                                KeyCode::Enter | KeyCode::NumpadEnter => {
                                    // `ChatHelper.SendChatMessageFromClient`
                                    // (`ChatHelper.cs:72-79`): a joining client
                                    // sends the line to the host, which runs
                                    // the command processor and relays the
                                    // result back as `82` text modules.
                                    let text = self.chat.text.clone();
                                    let is_client = self.is_client();
                                    let before = self.chat.messages.len();
                                    if is_client {
                                        self.send_chat_message("", &text);
                                    } else {
                                        let name = self.player_name.clone();
                                        let mut rng = self.item_rng;
                                        // The chat commands read the player
                                        // state (`DeathCommand`'s counters).
                                        self.chat.send_with(
                                            &text,
                                            &name,
                                            &mut rng,
                                            &mut self.emotes,
                                            self.player.as_ref(),
                                        );
                                        self.item_rng = rng;
                                    }
                                    // The host relays the lines its own submit
                                    // produced (slot 0 is the host's author).
                                    // The borrow dance: the server handle
                                    // borrows `self.net`, so the lines are
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
                                                        color: crate::net::Rgb(
                                                            color[0], color[1], color[2],
                                                        ),
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
                                }
                                KeyCode::Escape => {
                                    self.chat.text.clear();
                                    self.chat.open = false;
                                }
                                KeyCode::Backspace => {
                                    self.chat.text.pop();
                                }
                                _ => {}
                            }
                            if let Some(text) = event.text.as_ref() {
                                for ch in text.chars() {
                                    if !ch.is_control() {
                                        self.chat.text.push(ch);
                                    }
                                }
                            }
                            return;
                        }
                        // `Main.chatRelease && !drawingPlayerChat` →
                        // `OpenPlayerChat` (`Main.cs:18268-18287`): Enter
                        // opens the chat.
                        if matches!(code, KeyCode::Enter | KeyCode::NumpadEnter) {
                            self.chat.open = true;
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
                        match code {
                            KeyCode::Escape => {
                                // The bestiary eats the Escape first
                                // (`IngameFancyUI`'s GoBack), then the original
                                // opens the inventory on Escape; the world is
                                // saved on window close and with F9.
                                if self.bestiary_open {
                                    self.bestiary_open = false;
                                    self.bestiary_selected = None;
                                } else if self.emotes.menu_open {
                                    self.emotes.menu_open = false;
                                } else {
                                    self.inventory_open = !self.inventory_open;
                                }
                            }
                            KeyCode::F1 => self.debug = !self.debug,
                            KeyCode::F3 => {
                                crate::debug_panel::handle_key(self, KeyCode::F3);
                            }
                            KeyCode::F2 => {
                                self.light_mode = self.light_mode.next();
                                self.world_key = None;
                                log::info!("light mode: {}", self.light_mode.label());
                            }
                            KeyCode::F9 => self.save_world(),
                            KeyCode::KeyM => {
                                // `Main.TryOpeningFullscreenMap` (`Player.cs:29105`):
                                // `M` only toggles the map when `Main.mapEnabled`.
                                if self.settings.map_enabled {
                                    if let Some(m) = self.map.as_mut() {
                                        let open = !m.open;
                                        m.set_open(open);
                                    }
                                }
                            }
                            _ if crate::debug_panel::handle_key(self, code) => {}
                            KeyCode::Digit1 => {
                                // The shop open: the 1 key buys the first
                                // stock slot (the click-buy lands with the
                                // click-routing work).
                                if !self.buy_shop_item(0) {
                                    self.selected = 0;
                                }
                            }
                            KeyCode::Digit2 => self.selected = 1,
                            KeyCode::Digit3 => self.selected = 2,
                            KeyCode::Digit4 => self.selected = 3,
                            KeyCode::Digit5 => self.selected = 4,
                            KeyCode::Digit6 => self.selected = 5,
                            KeyCode::Digit7 => self.selected = 6,
                            KeyCode::Digit8 => self.selected = 7,
                            KeyCode::Digit9 => self.selected = 8,
                            KeyCode::Digit0 => self.selected = 9,
                            _ => {}
                        }
                    } else {
                        self.keys.remove(&code);
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
                    // `Main.DoScrollingInInventory`'s chest arm
                    // (`Main.cs:17435-17452`): the wheel scrolls the chest
                    // slot window while the inventory is open and the pointer
                    // is inside the grid (the plan-amendment insertion #10).
                    if step != 0
                        && self.menu.is_none()
                        && self.front.is_none()
                        && self.inventory_open
                    {
                        crate::ui::chest::scroll(self, step, self.mouse_pos);
                    }
                    if step != 0
                        && self.menu.is_none()
                        && self.front.is_none()
                        && !self.inventory_open
                    {
                        self.selected =
                            (self.selected as i32 + step).rem_euclid(HOTBAR as i32) as usize;
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
                                m.update_hover(self.mouse_pos);
                                let action = m.click();
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
                            if pressed {
                                self.handle_front_click();
                            }
                            return;
                        }
                        if !pressed {
                            crate::debug_panel::end_drag(self);
                        }
                        self.mine_down = pressed;
                        if pressed {
                            self.ui_left_press = true;
                            // The debug panel's click (the self-contained
                            // cheat-sheet — the panel consumes the clicks
                            // inside its rect).
                            crate::debug_panel::handle_click(self, self.mouse_pos);
                        }
                    }
                    MouseButton::Right => {
                        self.place_down = pressed;
                        if pressed {
                            self.ui_right_press = true;
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = Vec2::new(position.x as f32, position.y as f32);
                // The debug panel's drag (the title-bar drag moves the
                // panel).
                if crate::debug_panel::handle_drag(self, p) {
                    // The drag consumed the move.
                }
                // A `TR_SHOT` capture without `TR_MOUSE` parks the pointer
                // offscreen and ignores real motion (`deterministic_pointer_park`).
                if mouse_pin().is_none() && !deterministic_pointer_park() {
                    self.mouse_pos = p;
                }
                // The menu reads the pinned pointer when `TR_MOUSE` is set, so
                // its hover and value-bar drags stay deterministic.
                let pointer = self.mouse_pos;
                if let Some(m) = self.menu.as_mut() {
                    m.update_hover(pointer);
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
