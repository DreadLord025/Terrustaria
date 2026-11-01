//! The frame orchestrator — `Main.Draw`'s port. One `frame()` per
//! `RedrawRequested`: the sim-facing timers, the animated colour cycles,
//! the light parameters, the world-instance cache (G6) and the world plus
//! interface draw. Plus the benchmark epilogue and the two RNG helpers the
//! world simulation shares. (A7: extracted from `main.rs` whole.)

use std::sync::OnceLock;
use std::time::Instant;

use glam::Vec2;
use tr_core::lighting::{LightParams, LightStyle};
use tr_core::TILE_PX;

use crate::app::flow::{FrontDraw, FrontScreenBehaviours};
use crate::app::{no_hud, App, WorldFrameKey};
use crate::assets;
use crate::audio;
use crate::item::{self, SWING_TIME};
use crate::map;
use crate::npc::{self, NpcKind};
use crate::render::bg::cave_parallax;
use crate::render::debug::{capture_frame, draw_debug};
use crate::render::entities::{draw_held_item, draw_player_map_icon};
use crate::render::sky::{color_of_the_skies, draw_sky, tile_color};
use crate::render::tiles::draw_tiles;
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
            self.handle_inventory_clicks();
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
            self.update_rain(dt, screen);
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
        let mut render_player = self.player.as_ref().map(|p| p.interpolated(player_alpha));
        // `PlayerFrame` item-use branch: while a swing is active the body shows the use
        // rows instead of the idle/air pose. `swing` counts down from `SWING_TIME`,
        // which is `itemAnimation / itemAnimationMax` in the original.
        if let Some(p) = render_player.as_mut() {
            if self.swing > 0.0 && self.inventory[self.selected].count > 0 {
                p.apply_item_use_frame(self.swing / SWING_TIME);
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
                let override_day = std::env::var("TR_DAY")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                    .map(|v| v.clamp(0.0, 1.0));
                let (sky_tint, _sky_light, is_day, clock_time) = match override_day {
                    Some(d) => {
                        let night = [0.075f32, 0.08, 0.10];
                        let day = [1.0f32, 1.0, 1.0];
                        let tint = [
                            night[0] + (day[0] - night[0]) * d,
                            night[1] + (day[1] - night[1]) * d,
                            night[2] + (day[2] - night[2]) * d,
                        ];
                        let is_day = d >= 0.5;
                        let clock = if is_day {
                            d as f64 * 54000.0
                        } else {
                            (1.0 - d as f64) * 32400.0
                        };
                        (tint, tile_color(tint), is_day, clock)
                    }
                    // `Main.SetBackColor` + `ApplyColorOfTheSkiesToTiles`.
                    None => {
                        // Quantise the daylight/sky colour to one-second steps: the sun
                        // still moves smoothly on the draw pass, but the light map and
                        // world range can be reused for a second.
                        let quantised = (world.time / 60.0).floor() * 60.0;
                        let c = color_of_the_skies(quantised, world.day_time);
                        (c, tile_color(c), world.day_time, world.time)
                    }
                };
                // `SetBackColor`: rain multiplies the sky (and therefore the surface
                // sunlight) by `1 - cloudAlpha * 0.9`.
                let rain_alpha = rain.visual_alpha();
                let rain_dim = (1.0 - rain_alpha * 0.9).max(0.1);
                let sky_tint = [
                    sky_tint[0] * rain_dim,
                    sky_tint[1] * rain_dim,
                    sky_tint[2] * rain_dim,
                ];
                let sky_light = tile_color(sky_tint);
                // `Main.DoUpdate_AnimateCursorColors`: mouseTextColor oscillates
                // 190..=255, changing by 1 per frame, driving light flicker.
                self.mouse_text_color = (self.mouse_text_color as i32 + self.mouse_text_color_dir as i32).clamp(190, 255) as u8;
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
                    shimmer_darken: 0.0,
                    shimmer_alpha: 0.0,
                    sundial_cooldown: 0,
                    moondial_cooldown: 0,
                    // `Main.jellyfishCageMode`: `Main.AnimateTiles` is not ported, so
                    // every cage stays in mode 0.
                    cage_mode: [[0u8; tr_core::tile_light::CAGE_FRAMES]; 3],
                    rng_seed: self.light_rng_seed,
                };
                let sky_uv = atlas
                    .as_ref()
                    .and_then(|a| assets::Lookups::sheet_uv(a, "Background_0"));
                let style = LightStyle::for_mode(*light_mode);
                // Snap the render camera so integer world coordinates land on whole
                // screen pixels at any zoom. With a fractional camera, nearest-neighbour
                // sampling makes every tile texture crawl sub-pixel as the camera moves
                // while the player (drawn with `snap_to_screen`) stayed crisp, so the
                // world appeared to slide under the character. Rounding `cam * zoom`
                // keeps the step at one screen pixel even at 1.5x / 2x.
                let draw_cam = (*camera * *zoom).round() / *zoom;
                let surface_px = world.world_surface as f32 * TILE_PX;
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
                        liquid_anim
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
                            (((1.0 - *swing / SWING_TIME) * 40.0) as u32).min(40)
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
                        npcs,
                        npc_assets,
                        items,
                        &remote_players.values().cloned().collect::<Vec<_>>(),
                        *edit_target,
                    );
                    // `PlayerDrawLayers.DrawPlayer_27_HeldItem`: the item is only drawn
                    // while it is being used (`itemAnimation > 0 && useStyle != 0`), so
                    // the hand is empty at rest. Hold-style items (torches etc.) are not
                    // modelled yet.
                    if *swing > 0.0 {
                        if let Some(p) = render_player.as_ref() {
                            if let Some(slot) = inventory.get(*selected) {
                                if slot.count > 0 {
                                    tile_stats.quads += draw_held_item(
                                        renderer, lookups, light, &style, p, slot.kind,
                                        *swing, anim_time,
                                    );
                                }
                            }
                        }
                    }
                    *world_key = Some(key);
                }

                rain.draw(renderer, lookups, light, draw_cam, *zoom, screen);
                for p in projectiles.iter() {
                    p.draw(renderer, lookups, light, draw_cam, *zoom, screen);
                }

                let boss = npcs
                    .iter()
                    .find(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
                    .map(|n| (n.hp, n.max_hp));
                // `Main.cs:62147-62181`: while the fullscreen map is open `Main.Draw`
                // returns early after `DrawMap`, before `DrawInterface` (`62432`), so
                // the HUD (hotbar, life, inventory) is not drawn over the map.
                let fullscreen_map =
                    settings.map_enabled && map.as_ref().is_some_and(|m| m.open);
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
                    && self.map_style == 1
                {
                    if let Some(m) = map.as_ref() {
                        let pos = ui::minimap::draw_to_position(ui::minimap::minimap_origin(
                            screen,
                        ));
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
                        ui::minimap::draw_foreground(
                            renderer,
                            lookups,
                            s,
                            pos,
                            self.minimap_hover,
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
                        ui::UiPulse {
                            // `Main.mouseTextColor / 255` (`Main.cs:20355`).
                            text: self.mouse_text_color as f32 / 255.0,
                            cursor_scale: self.cursor.scale(),
                        },
                        m_h,
                        self.map_style,
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
                            if let Some(bg) =
                                map::map_bg_index(world, *camera, screen, player_tile)
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
                                    renderer, player_assets, p, icon_pos, 1.0, [1.0; 4],
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
