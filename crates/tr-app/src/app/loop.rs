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
        if let Ok(img) = image::load_from_memory(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/icon.png"
        ))) {
            let rgba = img.to_rgba8();
            if let Ok(icon) = winit::window::Icon::from_rgba(
                rgba.clone().into_raw(),
                rgba.width(),
                rgba.height(),
            ) {
                attrs = attrs.with_window_icon(Some(icon));
            }
        }
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));

        let gpu = pollster::block_on(Gpu::new(window.clone())).expect("gpu init");
        let atlas = self.atlas.as_ref().expect("atlas");
        let renderer = SpriteRenderer::new(&gpu, atlas, 8);

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
                                    let name = self.player_name.clone();
                                    let mut rng = self.item_rng;
                                    self.chat.send(
                                        &self.chat.text.clone(),
                                        &name,
                                        &mut rng,
                                        &mut self.emotes,
                                    );
                                    self.item_rng = rng;
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
                            KeyCode::F2 => {
                                self.light_mode = self.light_mode.next();
                                self.world_key = None;
                                log::info!("light mode: {}", self.light_mode.label());
                            }
                            KeyCode::F5 => {
                                if let Some(p) = self.player.as_mut() {
                                    p.respawn();
                                }
                                if self.follow_player {
                                    if let Some(p) = &self.player {
                                        let (cx, cy) = p.center();
                                        self.camera = Vec2::new(cx, cy);
                                    }
                                } else {
                                    self.camera = Vec2::new(
                                        self.world.width as f32 * TILE_PX * 0.5,
                                        self.world.spawn_y as f32 * TILE_PX,
                                    );
                                }
                                self.zoom = self.settings.zoom;
                                self.world_key = None;
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
                            KeyCode::KeyE => {
                                // The EoC debug spawn (the vanilla's
                                // Suspicious Looking Eye + the night roll
                                // land with the boss-item work).
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::EyeOfCthulhu && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 15)
                                            .clamp(2, self.world.width - 3);
                                        let (w2, h2) = NpcKind::EyeOfCthulhu.size();
                                        let mut boss = Npc::new(
                                            NpcKind::EyeOfCthulhu,
                                            xt as f32 * TILE_PX,
                                            ((p.y / TILE_PX) as i32 - 20) as f32 * TILE_PX,
                                            1,
                                            0.0,
                                        );
                                        boss.hp = boss.max_hp;
                                        boss.caster_counter = 0;
                                        boss.cast_windup = 0;
                                        let _ = (w2, h2);
                                        log::info!("Eye of Cthulhu spawned");
                                        self.npcs.push(boss);
                                    }
                                }
                            }
                            KeyCode::KeyH => {
                                // The Brain of Cthulhu debug spawn (the
                                // Bloody Spine item = the follow-up).
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::BrainOfCthulhu && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 15)
                                            .clamp(2, self.world.width - 3);
                                        let mut boss = Npc::new(
                                            NpcKind::BrainOfCthulhu,
                                            xt as f32 * TILE_PX,
                                            ((p.y / TILE_PX) as i32 - 15) as f32 * TILE_PX,
                                            1,
                                            0.0,
                                        );
                                        boss.hp = boss.max_hp;
                                        for i in 0..20 {
                                            let ang = i as f32 / 20.0
                                                * std::f32::consts::TAU;
                                            let mx = boss.x + 160.0
                                                + ang.cos() * 200.0;
                                            let my = boss.y + 55.0
                                                + ang.sin() * 200.0;
                                            let mut cr = Npc::new(
                                                NpcKind::Creeper,
                                                mx,
                                                my,
                                                1,
                                                0.0,
                                            );
                                            cr.hp = cr.max_hp;
                                            self.npcs.push(cr);
                                        }
                                        log::info!("Brain of Cthulhu spawned");
                                        self.npcs.push(boss);
                                    }
                                }
                            }
                            KeyCode::KeyG => {
                                // The Plantera debug spawn.
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::Plantera && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 12)
                                            .clamp(2, self.world.width - 3);
                                        let mut boss = Npc::new(
                                            NpcKind::Plantera,
                                            xt as f32 * TILE_PX,
                                            ((p.y / TILE_PX) as i32 - 15) as f32 * TILE_PX,
                                            1,
                                            0.0,
                                        );
                                        boss.hp = boss.max_hp;
                                        log::info!("Plantera spawned");
                                        self.npcs.push(boss);
                                    }
                                }
                            }
                            KeyCode::KeyT => {
                                // The Twins debug spawn (the Mechanical
                                // Eyes item = the follow-up): the pair of
                                // eyes at the offset points.
                                let twins_alive = self.npcs.iter().any(|n| {
                                    matches!(n.kind, NpcKind::Retinazer | NpcKind::Spazmatism)
                                        && n.hp > 0
                                });
                                if !twins_alive {
                                    if let Some(p) = &self.player {
                                        for (kind, dx) in [
                                            (NpcKind::Retinazer, -14),
                                            (NpcKind::Spazmatism, 14),
                                        ] {
                                            let xt = ((p.x / TILE_PX) as i32 + dx)
                                                .clamp(2, self.world.width - 3);
                                            let mut boss = Npc::new(
                                                kind,
                                                xt as f32 * TILE_PX,
                                                ((p.y / TILE_PX) as i32 - 18) as f32 * TILE_PX,
                                                1,
                                                0.0,
                                            );
                                            boss.hp = boss.max_hp;
                                            self.npcs.push(boss);
                                        }
                                        log::info!("The Twins spawned");
                                    }
                                }
                            }
                            KeyCode::KeyP => {
                                // The Prime debug spawn (the Mechanical
                                // Skull item = the follow-up).
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::SkeletronPrime && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 16)
                                            .clamp(2, self.world.width - 3);
                                        let mut boss = Npc::new(
                                            NpcKind::SkeletronPrime,
                                            xt as f32 * TILE_PX,
                                            ((p.y / TILE_PX) as i32 - 18) as f32 * TILE_PX,
                                            1,
                                            0.0,
                                        );
                                        boss.hp = boss.max_hp;
                                        log::info!("Skeletron Prime spawned");
                                        self.npcs.push(boss);
                                    }
                                }
                            }
                            KeyCode::KeyD => {
                                // The Destroyer debug spawn (the Mechanical
                                // Worm item = the follow-up): the head —
                                // the 80-segment same-kind chain assembles
                                // through the worm pass.
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::TheDestroyer && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 18)
                                            .clamp(2, self.world.width - 3);
                                        let mut head = Npc::spawn_worm_head(
                                            &self.world,
                                            xt,
                                            (p.y / TILE_PX) as i32 - 6,
                                            NpcKind::TheDestroyer,
                                        );
                                        head.hp = head.max_hp;
                                        head.worm_left = 80;
                                        log::info!("The Destroyer spawned");
                                        self.npcs.push(head);
                                    }
                                }
                            }
                            KeyCode::KeyF => {
                                // The WoF debug spawn (the Voodoo Doll
                                // sacrifice = the follow-up). The wall
                                // spawns at the underworld depth.
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::WallOfFlesh && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 - 25)
                                            .clamp(2, self.world.width - 3);
                                        let mut boss = Npc::new(
                                            NpcKind::WallOfFlesh,
                                            xt as f32 * TILE_PX,
                                            ((p.y / TILE_PX) as i32 + 5) as f32 * TILE_PX,
                                            1,
                                            0.0,
                                        );
                                        boss.hp = boss.max_hp;
                                        log::info!("Wall of Flesh spawned");
                                        self.npcs.push(boss);
                                    }
                                }
                            }
                            KeyCode::KeyS => {
                                // The Skeletron debug spawn (the Old Man's
                                // curse dialog = the follow-up).
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::SkeletronHead && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 14)
                                            .clamp(2, self.world.width - 3);
                                        let mut boss = Npc::new(
                                            NpcKind::SkeletronHead,
                                            xt as f32 * TILE_PX,
                                            ((p.y / TILE_PX) as i32 - 18) as f32 * TILE_PX,
                                            1,
                                            0.0,
                                        );
                                        boss.hp = boss.max_hp;
                                        log::info!("Skeletron spawned");
                                        self.npcs.push(boss);
                                    }
                                }
                            }
                            KeyCode::KeyQ => {
                                // The Queen Bee debug spawn (the Abeemination
                                // item = the follow-up).
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::QueenBee && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 10)
                                            .clamp(2, self.world.width - 3);
                                        let mut boss = Npc::new(
                                            NpcKind::QueenBee,
                                            xt as f32 * TILE_PX,
                                            ((p.y / TILE_PX) as i32 - 15) as f32 * TILE_PX,
                                            1,
                                            0.0,
                                        );
                                        boss.hp = boss.max_hp;
                                        log::info!("Queen Bee spawned");
                                        self.npcs.push(boss);
                                    }
                                }
                            }
                            KeyCode::KeyW => {
                                // The EoW debug spawn (the Worm Food item +
                                // the orb breaks = the follow-ups): the
                                // head in the air — the chain assembles
                                // through the worm pass.
                                if !app_has_eow(self) {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 12)
                                            .clamp(2, self.world.width - 3);
                                        let mut head = Npc::spawn_worm_head(
                                            &self.world,
                                            xt,
                                            (p.y / TILE_PX) as i32 - 8,
                                            NpcKind::EaterOfWorldsHead,
                                        );
                                        head.hp = head.max_hp;
                                        log::info!("Eater of Worlds spawned");
                                        self.npcs.push(head);
                                    }
                                }
                            }
                            KeyCode::KeyB => {
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 12)
                                            .clamp(2, self.world.width - 3);
                                        if let Some(n) = Npc::spawn_king_slime(&self.world, xt) {
                                            log::info!("King Slime spawned");
                                            self.npcs.push(n);
                                        }
                                    }
                                }
                            }
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
                        self.mine_down = pressed;
                        if pressed {
                            self.ui_left_press = true;
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
