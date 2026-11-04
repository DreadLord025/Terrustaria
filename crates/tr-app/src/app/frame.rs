//! The frame orchestrator — `Main.Draw`'s port. One `frame()` per
//! `RedrawRequested`: the sim-facing timers, the animated colour cycles,
//! the light parameters, the world-instance cache (G6) and the world plus
//! interface draw. Plus the benchmark epilogue and the two RNG helpers the
//! world simulation shares. (A7: extracted from `main.rs` whole.)

use std::sync::OnceLock;
use std::time::Instant;

use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::lighting::{LightParams, LightStyle};
use wgpu::util::DeviceExt;

use crate::app::flow::{FrontDraw, FrontScreenBehaviours};
use crate::app::{App, WorldFrameKey, no_hud};
use crate::assets;
use crate::audio;
use crate::craft;
use crate::item::{self};
use crate::map;
use crate::npc::{self, NpcKind};
use crate::render::bg::cave_parallax;
use crate::render::debug::{capture_frame, draw_debug};
use crate::render::entities::draw_player_map_icon;
use crate::render::sky::{color_of_the_skies, draw_sky, tile_color};
use crate::render::tiles::draw_tiles;
use crate::player;
use crate::sound;
use crate::ui;
use crate::ui::text::{hud_text, hud_text_width};
use crate::world_sim::clock::{advance_time, time_frozen};

/// `TR_NOCACHE=1` disables D2 and G6, so benchmarks measure the full per-frame
/// work instead of a cached scene. Also used to A/B the optimisations.
fn no_cache() -> bool {
    static NO_CACHE: OnceLock<bool> = OnceLock::new();
    *NO_CACHE.get_or_init(|| std::env::var_os("TR_NOCACHE").is_some())
}

impl App {
    pub(crate) fn frame(&mut self) {
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
        self.anim_time += dt;
        // `Main.DoUpdate_AnimateDiscoRGB`: the disco-ball colour cycle.
        self.advance_disco();
        let anim_time = self.anim_time;
        if !self.follow_player {
            self.pan_camera(dt);
        }
        if !time_frozen() {
            advance_time(&mut self.world, dt);
        }
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
            && !no_hud()
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
        });

        if self.menu.is_none() && self.front.is_none() {
            // `MinimapFrame.Update` runs before the inventory's click handling
            // (`MinimapFrameManager.cs:45`, `Main.cs:41037`'s `ItemSlot.Handle`).
            self.handle_minimap_clicks();
            self.handle_bestiary_clicks();
            if !self.handle_crafting_clicks() {
                self.handle_inventory_clicks();
            }
            for (i, scale) in self.hotbar_scales.iter_mut().enumerate() {
                let target = if i == self.selected { 1.0 } else { 0.75 };
                if *scale < target {
                    *scale = (*scale + 0.05).min(target);
                } else if *scale > target {
                    *scale = (*scale - 0.05).max(target);
                }
            }
            self.update_player(dt);
            let hp = self.player.as_ref().map_or(0, |p| p.hp);
            if hp < self.last_hp {
                self.audio.play("Player_Hit_0");
            }
            self.last_hp = hp;
            // `Main.UpdateAudio_DecideOnNewMusic` reads the scene centre, not the
            // player: underground once the view centre passes the world surface
            // (`Main.cs:12815`).
            let boss_music = self
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
                .then_some(93); // MusicID.KingSlime
            self.music.update(audio::MusicEnv {
                music_volume: self.settings.music_volume,
                day: self.world.day_time,
                underground: self.camera.y
                    >= self.world.world_surface as f32 * TILE_PX + screen.y * 0.5,
                rain: self.rain.is_raining(),
                boss_music,
                clock: self.world.time,
            });
            if !self.is_client() {
                self.update_npcs(dt);
            }
            self.update_liquids(dt);
            self.update_waterfalls(dt, screen);
            self.update_wiring(dt);
            self.update_rain(dt, screen);
            // M1.4-BG-18 phase 3: `Main.UpdateSceneMetrics` + `Main.SceneState.
            // Update` (`Main.cs:62560,17816`) — the scan, the fades and the
            // `ApplyVisuals` driver; the post branch consumes
            // `filter_driver.output`.
            crate::world_sim::scene_scan::update_scene(self, dt, screen);
            // `Main.UpdateWindyDayState` (`Main.cs:12990-13027`, called from
            // the per-tick audio update): the sticky windy-day/storm flags the
            // ambience gates read; the hysteresis bands make the per-frame
            // evaluation equivalent.
            self.wind.update_windy_day_state(
                self.rain.visual_alpha(),
                self.world.time,
                self.world.day_time,
            );
            // `Main.AmbienceServer.Update` (`Main.cs:17214-17216`) +
            // `SkyManager.Update` (`Main.cs:17139`): the ambient sky scheduler
            // and its entities, stepped on the 60 Hz accumulator.
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
                        Vec2::new(
                            p.x + player::WIDTH * 0.5,
                            p.y + player::HEIGHT * 0.5,
                        )
                    }),
                    // `plr.position.Y` is the hitbox TOP (`AmbienceServer.cs:191`).
                    player_y: self.player.as_ref().map_or(self.camera.y, |p| p.y),
                    day_rate: 1,
                };
                ambience.update(dt, &ctx, ambience_rng);
            }
            self.update_items(dt);
            self.update_projectiles(dt);
            self.update_editing(dt, screen);
            self.net_tick(dt);
            if let Some(m) = self.map.as_mut() {
                m.update(&self.world, self.player.as_ref(), &self.light, dt);
            }
        }
        // `TR_NET_EDIT=tx,ty`: one client edit request, used to verify the round trip.
        if self.is_client() && self.frame_count == 90 {
            if let Ok(v) = std::env::var("TR_NET_EDIT") {
                if let Some((sx, sy)) = v.split_once(',') {
                    if let (Ok(x), Ok(y)) = (sx.trim().parse::<i32>(), sy.trim().parse::<i32>()) {
                        self.request_edit(x, y, 0, 0);
                        log::info!("net edit requested at {x},{y}");
                    }
                }
            }
        }
        self.poll_net();
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
        // `PlayerFrame` item-use branch: while a swing is active the body shows the use
        // rows instead of the idle/air pose; `swing / swing_max` is
        // `itemAnimation / itemAnimationMax`. At rest a `holdStyle == 1` item
        // (the torch) pins the body to row 3 (`Player.cs:36455-36459`).
        let held_hold_style = self
            .inventory
            .get(self.selected)
            .filter(|s| s.count > 0)
            .map_or(0, |s| item::hold_style(s.kind));
        if let Some(p) = render_player.as_mut() {
            if self.swing > 0.0 && self.inventory[self.selected].count > 0 {
                p.apply_item_use_frame(self.swing / self.swing_max.max(f32::EPSILON));
            } else if held_hold_style == 1 {
                p.body_frame = 3;
            }
        }
        let player_key = render_player.as_ref().map_or(0, |p| p.render_key());
        let npc_key = npc::render_key(&self.npcs);
        let mut stats = tr_render::sprite::RenderStats::default();
        let mut tile_stats = self.tile_stats;

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
                liquid_alpha,
                wind,
                tree_wind_counter,
                grass_wind_counter,
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
                let override_day = std::env::var("TR_DAY")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                    .map(|v| v.clamp(0.0, 1.0));
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
                    sundial_cooldown: 0,
                    moondial_cooldown: 0,
                    // `Main.jellyfishCageMode`: `Main.AnimateTiles` is not ported, so
                    // every cage stays in mode 0.
                    cage_mode: [[0u8; tr_core::tile_light::CAGE_FRAMES]; 3],
                    rng_seed: self.light_rng_seed,
                    // `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023`):
                    // false until the EchoMonolith / 4409-accessory systems land;
                    // `TR_SHOW_INVISIBLE=1` pins it on for A/B shots.
                    draw_invisible_walls: std::env::var_os("TR_SHOW_INVISIBLE").is_some(),
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
                // Snap the render camera so integer world coordinates land on whole
                // screen pixels at any zoom. With a fractional camera, nearest-neighbour
                // sampling makes every tile texture crawl sub-pixel as the camera moves
                // while the player (drawn with `snap_to_screen`) stayed crisp, so the
                // world appeared to slide under the character. Rounding `cam * zoom`
                // keeps the step at one screen pixel even at 1.5x / 2x.
                let draw_cam = (*camera * *zoom).round() / *zoom;
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
                if std::env::var_os("TR_NO_WATERFALL").is_none() {
                    let half_v = screen * 0.5 / *zoom;
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
                    scratch.waterfall_lights = lights;
                    // `WaterfallManager.TrySparkling`: drain the walk's spawns
                    // into the rain module's dust particles.
                    rain.spawn_sparkles(waterfall.take_sparkles());
                } else {
                    scratch.waterfall_quads.clear();
                    scratch.waterfall_lights.clear();
                }

                // `TileDrawing.Update` (`TileDrawing.cs:367-375`): the wind walk
                // runs in `UpdateWeather`; the sway counters advance per frame,
                // faster with stronger wind.
                wind.sway_enabled = settings.tiles_sway && std::env::var_os("TR_NO_WIND").is_none();
                wind.update(rain.is_raining());
                // `TR_WIND_FORCE=<speed>`: pin `windSpeedCurrent` so the sway and
                // the leaf frequency are screenshotable at a chosen strength.
                if let Ok(v) = std::env::var("TR_WIND_FORCE") {
                    if let Ok(f) = v.parse::<f32>() {
                        wind.speed_current = f.clamp(-1.24, 1.24);
                    }
                }
                tr_core::wind::advance_tree_counter(tree_wind_counter, wind.speed_current);
                tr_core::wind::advance_counter(grass_wind_counter, wind.speed_current);

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
                    zoom: zoom.to_bits(),
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
                    world_revision: world.revision,
                };
                let reuse_world = !no_cache() && *world_key == Some(key);
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
                    draw_cam.y - screen.y / (2.0 * *zoom) + screen.y / 2.0,
                    world.world_surface,
                    world.width,
                );
                draw_sky(
                    renderer,
                    draw_cam,
                    *zoom,
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
                        *zoom,
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
                        &mut leaf_spawns,
                    );
                    *world_key = Some(key);
                }

                // `TileDrawing.EmitTreeLeaves`: the foliage pass's spawns become
                // leaf gores; `Gore_UpdateLeaf` ticks them and the draw rides the
                // UI range like the rain. `TR_NO_WIND` is the A/B switch for the
                // whole wind feature, spawns included.
                let wind_ab_on = std::env::var_os("TR_NO_WIND").is_none();
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

                rain.draw(renderer, lookups, light, draw_cam, *zoom, screen);
                crate::render::foliage::draw_leaves(
                    renderer, lookups, light, draw_cam, *zoom, screen, leaves,
                );
                for p in projectiles.slots().iter().filter(|p| p.active) {
                    p.draw(renderer, lookups, light, draw_cam, *zoom, screen);
                }

                let boss = npcs
                    .iter()
                    .find(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
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
                if settings.map_enabled
                    && menu.is_none()
                    && front.is_none()
                    && !no_hud()
                    && !fullscreen_map
                    // `Main.inFancyUI` (`IngameFancyUI.cs:64-72`): the
                    // bestiary state covers the interface — the minimap HUD
                    // does not draw under it.
                    && !self.bestiary_open
                    && self.map_style == 1
                {
                    if let Some(m) = map.as_ref() {
                        let pos =
                            ui::minimap::draw_to_position(ui::minimap::minimap_origin(screen));
                        // `Main.mapMinimapAlpha = 1` (`Main.cs:66753`).
                        ui::minimap::draw_background(renderer, lookups, pos, 1.0);
                        m.draw_minimap(renderer, world, screen, view_center);
                        // The hover highlight is `MinimapFrame.Update`'s
                        // `buttonUnderMouse.IsHighlighted`
                        // (`MinimapFrame.cs:163`), resolved per frame in
                        // `handle_minimap_clicks` — like the source, where
                        // `DrawTo` runs `Update` before the foreground batch
                        // (`MinimapFrameManager.cs:45`, `Main.cs:55769`).
                        let s = ui::minimap::style_index(&settings.minimap_frame);
                        ui::minimap::draw_foreground(renderer, lookups, s, pos, self.minimap_hover);
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
                } else if menu.is_none() && !no_hud() && !fullscreen_map {
                    let craft_available = if self.crafting_open {
                        let adj = player
                            .as_ref()
                            .map(|p| craft::adjacent_tiles(world, p))
                            .unwrap_or_default();
                        craft::available(inventory, &adj)
                    } else {
                        Vec::new()
                    };
                    ui::draw_hud(
                        renderer,
                        *camera,
                        *zoom,
                        screen,
                        lookups,
                        inventory,
                        &self.equip,
                        self.equip_page,
                        *selected,
                        &self.hotbar_scales,
                        player.as_ref(),
                        boss,
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
                        &self.kill_tracker,
                    );
                } else if let Some(menu) = menu.as_mut() {
                    menu.draw(renderer, lookups, screen, dt);
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
                    if !no_hud() && menu.is_none() && front.is_none() && settings.map_enabled {
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
                            map.draw_full(renderer, lookups, world, screen);
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
                                    1.0,
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
                    stats = renderer.render(
                        gpu,
                        scene,
                        draw_cam.to_array(),
                        *zoom,
                        now.elapsed().as_secs_f32(),
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
                                *zoom,
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
                } else {
                    stats = renderer.render(
                        gpu,
                        &view,
                        draw_cam.to_array(),
                        *zoom,
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
