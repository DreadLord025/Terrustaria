//! The draw region of the frame orchestrator — `Main.DoDraw`'s body
//! (S1-B-8 phase 3): the pre-draw state (the sky tint, the animators, the
//! light parameters, the zooms, the waterfall walk, the wind sway, the G6
//! `WorldFrameKey` and the atmosphere) and the marker-ordered passes (sky →
//! world → rain → wires → projectiles) plus the interface, the map, the
//! debug overlays, the post/filters and the present path. The `draw_phase`
//! markers stay at their exact sites so the S1-B-7 order pin keeps
//! detecting a reordering; the marker-level fn split is the follow-up
//! refinement.

use std::time::Instant;

use glam::Vec2;
use tr_core::TILE_PX;
use tr_core::lighting::{LightParams, LightStyle};

use crate::app::flow::{FrontDraw, FrontScreenBehaviours};
use crate::app::{App, WorldFrameKey};
use crate::assets;
use crate::craft;
use crate::item::{self};
use crate::map;
use crate::npc::{self, NpcKind};
use crate::render::bg::cave_parallax;
use crate::render::debug::{
    capture_frame, cursor_tile, draw_debug, draw_debug_window, DebugPlayer, DebugTile,
    DebugWindowInput,
};
use crate::render::entities::draw_player_map_icon;
use crate::render::sky::{color_of_the_skies, draw_sky, tile_color};
use crate::render::tiles::draw_tiles;
use crate::render::tiles::TileDrawStats;
use crate::ui;
use crate::ui::text::{border_string, hud_text, hud_text_width};

use super::draw_phase;

impl App {
    /// The draw region, verbatim from the pre-snapshot locals through the
    /// present: one borrow region as landed (the 55-field destructure and
    /// the gpu/renderer gate move with it unchanged). Returns the frame's
    /// render/tile stats for the orchestrator's epilogue.
    pub(super) fn draw_region(
        &mut self,
        now: Instant,
        dt: f32,
        anim_time: f32,
        screen: Vec2,
        m_h: f32,
    ) -> Option<(tr_render::sprite::RenderStats, TileDrawStats)> {
            // S1-G-13: `Main.Draw`'s `EnsureRenderTargetContent` head call
            // (`Main.cs:61644`, `:58152-58158`) — allocate/refresh the
            // render-target family (`Main.InitTargets`, `:58160-58216`)
            // when absent or the backbuffer resized. The port runs HiDef
            // (the non-XNA default, `Main.cs:67047`) with
            // `SupportWideScreen = false` (`:67046`). Allocation only: no
            // draw pass consumes the layers yet — the frame path below is
            // unchanged (ANALYSIS "The render-target family (S1-G-13)").
            if let Some(gpu) = self.gpu.as_ref() {
                tr_render::RenderTargetFamily::ensure(
                    &mut self.targets,
                    &gpu.device,
                    gpu.config.format,
                    (gpu.config.width, gpu.config.height),
                    true,
                    false,
                    tr_render::RENDER_TARGET_MAX_HIDEF,
                );
            }
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

            // The chest-rename editor's per-frame reads — `Main.editChest`
            // (`Main.cs:1747`), `Main.npcChatText`'s editor copy
            // (`UIVirtualKeyboard.cs:1061-1067`) and the stored `Chest.name`
            // (`ChestUI.cs:115-122`, the `WorldChest` entity's own field since the
            // M6-SAVE-1 phase-3 fold) — parked before the draw site's field
            // borrows (the keyboard session owns the editor state; the name
            // reads the entity directly so the borrow stays field-scoped).
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
            // `Main.PylonSystem.Pylons` (`TeleportPylonsMapLayer.cs:32`): the map
            // layers read the live network. The scan walks the TE book, so it stays
            // off the per-frame path unless the map or the minimap is drawing.
            let map_layer_pylons = if (self.map.as_ref().is_some_and(|m| m.open)
                || (self.settings.map_enabled && self.map_style == 1))
                && !crate::env::no_hud()
            {
                tr_core::pylons::scan_pylons(&self.world)
            } else {
                Vec::new()
            };
            // The buff bar's inputs (`Main.DrawInterface_Resources_Buffs`,
            // `Main.cs:43203`): the right-release edge (`Main.mouseRightRelease`)
            // and the vampire-seed gate (`Main.vampireSeed`, the duration hides
            // `:43298`). The removals drain in the tick.
            let right_release = std::mem::take(&mut self.ui_right_release);
            // `Main.mouseLeftRelease` — the achievement popup's click edge
            // (`InGamePopups.cs:124`), consumed per frame like the right one.
            let left_release = std::mem::take(&mut self.ui_left_release);
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
            // The F4 debug window's container-free inputs, snapshotted before
            // the field destructuring (its rows read the pools only by count).
            let debug_overlay = self.debug_overlay;
            let debug_mouse = self.mouse_pos;
            let debug_items = self.items.len();
            let debug_projectiles = self.projectiles.active_count();
            let debug_gore = self.gore.slots().iter().filter(|g| g.active).count();
            // S1-G-1 wave D: the void lens's `Update` gate
            // (`TileDrawing.cs:8717`, `!Main.gamePaused`) — snapshotted before
            // the field destructuring (the pass borrows `entity_lights` mutably).
            let game_paused_for_specials = self.game_paused;
            // The achievement popup's click target (`InGamePopups.cs:138`'s
            // `OpenAchievementsAndGoto`) — parked here because the goto
            // mutates `self` while the draw destructure holds its fields;
            // applied after the gpu block (S1-E-26 wave A).
            let mut popup_goto: Option<&'static str> = None;

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
                    combat_texts,
                    npc_assets,
                    inventory,
                    selected,
                    edit_target,
                    mine_target,
                    mine_timer,
                    mine_hardness,
                    mine_crack_style,
                    mine_crack_target,
                    mine_crack_rng,
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
                    legacy_light,
                    has_liquids,
                    liquids,
                    waterfall,
                    rain,
                    sandstorm,
                    sand,
                    dust,
                    liquid_alpha,
                    wind,
                    tree_wind_counter,
                    grass_wind_counter,
                    sunflower_wind_counter,
                    sunflower_wind_step,
                    vine_wind_counter,
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
                    scene_state,
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
                        shimmer_darken: scene_state.shimmer_darken,
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
                    // `WorldItem.Update`'s `UpdateItem_VisualEffects` lights (S1-E-5)
                    // (`WorldItem.cs:1204-1649`). The C# runs them once per live
                    // drop per tick, in `Main.item` slot order, at the end of
                    // `WorldItem.Update` (`:436`); `items` here is that same slot
                    // order after the step. No arm draws `Main.rand`, so the
                    // placement relative to the sim leaves the stream untouched.
                    for drop in items.iter() {
                        let Some(def) = crate::registry::get(drop.kind) else {
                            continue;
                        };
                        let wet = drop.box_is_wet(world);
                        if let Some((x, y, rgb)) = crate::entity_lights::item_light(
                            def,
                            drop.pos,
                            crate::item::SIZE,
                            wet,
                            &params,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                    }
                    // The NPC `AI` lights (S1-E-5): the aiStyle-8 tail
                    // (`NPC.cs:21280-21373`) runs per live NPC per update in
                    // `Main.npc` slot order; the port collects the same
                    // sources here after the step, in the same slot order.
                    // The C# adds `netOffset` around the chain (`:21279`/
                    // `:21374`) — the feed reads the offset position too.
                    // No arm draws `Main.rand` (the dust halves are skipped,
                    // `npc_lights.rs`), so the placement relative to the sim
                    // leaves the stream untouched.
                    for n in npcs.iter() {
                        if n.hp <= 0 {
                            continue;
                        }
                        let (w, h) = n.size();
                        if let Some((x, y, rgb)) = crate::npc_lights::npc_light(
                            n.kind,
                            Vec2::new(n.x, n.y) + n.net_offset,
                            w,
                            h,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                    }
                    // `Player.UpdateBuffs`' Shine arm (`Player.cs:9993-9996`,
                    // S1-E-7 wave A): the 0.8/0.95/1.0 light at the player's
                    // center tile while buff 11 is held. No RNG — the feed
                    // leaves the streams untouched.
                    if let Some(p) = player.as_ref() {
                        if let Some((x, y, rgb)) = crate::entity_lights::shine_light(p) {
                            self.entity_lights.add(x, y, rgb);
                        }
                    }
                    let sky_uv = atlas
                        .as_ref()
                        .and_then(|a| assets::Lookups::sheet_uv(a, "Background_0"));
                    let mut style = LightStyle::for_mode(*light_mode);
                    // S1-E-19: White/Retro/Trippy select the LEGACY engine
                    // (`Lighting.Mode` setter, `Lighting.cs:45-56`); the style
                    // transforms become pass-throughs — the engine itself
                    // produces the mode's light (White's luminance collapse,
                    // Retro's subtractive falloff, Trippy's per-channel
                    // falloff). Color keeps the modern engine and its style.
                    let legacy_active = *light_mode != tr_core::lighting::LightMode::Color;
                    if legacy_active {
                        style.grayscale = false;
                        style.levels = 0;
                        style.perturb = 0.0;
                    }
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
                        // `LightingEngine.ApplyPerFrameLights`'s swap
                        // (`LightingEngine.cs:228-233`): the list just filled becomes
                        // the one the next frame's light pass reads. The waterfall
                        // sources above are swapped by their own manager, so only the
                        // entity list is swapped here.
                        self.entity_lights.end_frame();
                        // `WaterfallManager.TrySparkling`: drain the walk's spawns
                        // into the rain module's dust particles.
                        rain.spawn_sparkles(waterfall.take_sparkles());
                    } else {
                        scratch.waterfall_quads.clear();
                        scratch.waterfall_lights.clear();
                        self.entity_lights.end_frame();
                    }

                    // S1-E-19: `Lighting.LightTiles` (`Main.cs:62145`, after
                    // the waterfall walk at `:61841`) — the legacy engine's
                    // `ProcessArea` for White/Retro/Trippy. The camera inputs
                    // are `Camera.UnscaledPosition/Size` (`Camera.cs:8-10`):
                    // the port's `draw_cam` is the view centre and `screen`
                    // the backbuffer, so the unscaled top-left is
                    // `draw_cam - screen/2`. The `AddLight` list is the
                    // one-frame-delayed union the modern compute reads.
                    if legacy_active {
                        scratch.all_lights.clear();
                        scratch.all_lights.extend_from_slice(&scratch.waterfall_lights);
                        scratch.all_lights.extend_from_slice(self.entity_lights.visible());
                        let mut legacy_water_style =
                            crate::biome::calculate_water_style(world, *camera);
                        if let Some(n) = crate::env::water_style() {
                            legacy_water_style = n.min(14);
                        }
                        let legacy_inputs = tr_core::lighting::legacy::LegacyInputs {
                            render_phases: self.gfx_quality.render_phases,
                            water_style: legacy_water_style as u8,
                            // NightOwl (buff 12) arms `nightVision` per tick
                            // (`Player.cs:9999`, S1-E-7 wave A); blackout /
                            // headcovered have no ported sources yet.
                            night_vision: player.as_ref().is_some_and(|p| p.night_vision),
                            blind: player.as_ref().is_some_and(|p| p.blind),
                            blackout: false,
                            headcovered: false,
                            game_paused: self.game_paused,
                            // `GameViewMatrix.Translation`
                            // (`SpriteViewMatrix.cs:149-153`): the zoom
                            // frame's offset — `Round(screen/2 - (screen/2)
                            // / Round(zoom/(1/128))*(1/128))`, banker's
                            // rounding (`Utils.Round`, `Utils.cs:112-115`).
                            // The minimap export reads it (`:375-382`).
                            view_translation: {
                                let q = [
                                    (world_zoom / (1.0 / 128.0)).round_ties_even()
                                        * (1.0 / 128.0),
                                    (world_zoom / (1.0 / 128.0)).round_ties_even()
                                        * (1.0 / 128.0),
                                ];
                                [
                                    (screen.x * 0.5 - screen.x * 0.5 / q[0]).round_ties_even(),
                                    (screen.y * 0.5 - screen.y * 0.5 / q[1]).round_ties_even(),
                                ]
                            },
                            // `Netplay.Connection.StatusMax > 0` (`:341`).
                            net_connecting: self.net_status_max > 0,
                        };
                        let engine = legacy_light.get_or_insert_with(|| {
                            tr_core::lighting::legacy::LegacyLighting::new(*light_mode)
                        });
                        let want_mode = match *light_mode {
                            tr_core::lighting::LightMode::White => 1,
                            tr_core::lighting::LightMode::Retro => 2,
                            tr_core::lighting::LightMode::Trippy => 3,
                            _ => 0,
                        };
                        if engine.mode() != want_mode {
                            engine.set_mode(*light_mode);
                        }
                        // S1-E-19 wave B: the minimap light export
                        // (`TryUpdatingMapWithLight`, `LegacyLighting.cs:371-433`)
                        // rides the engine's `ProcessArea` gate block
                        // (`:341-358`). The `Main.Map` seam borrows the
                        // WorldMap's tiles + `mapTime`; `map_enabled` is
                        // `Main.mapEnabled` (`Main.cs:813`). The changed tiles
                        // come back through the seam and queue for re-upload
                        // (`MapRenderer.QueueChange`'s analog).
                        let mut export = map
                            .as_mut()
                            .map(|m| m.legacy_light_export(settings.map_enabled));
                        engine.process_area(
                            world,
                            [draw_cam.x - screen.x * 0.5, draw_cam.y - screen.y * 0.5],
                            [screen.x, screen.y],
                            &params,
                            &legacy_inputs,
                            &scratch.all_lights,
                            export.as_mut(),
                        );
                        // Consume the seam (ending its tile/counter borrows)
                        // before re-borrowing the map for the dirty push.
                        let changed = export.map(|e| e.changed);
                        if let Some(changed) = changed {
                            if let Some(m) = map.as_mut() {
                                m.push_dirty(changed);
                            }
                        }
                        engine.present_into(light);
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
                        // `_vineWindCounter` advances in the same
                        // `TileDrawing.Update` (`TileDrawing.cs:375`), so it
                        // rides the same 60 Hz step (S1-G-1).
                        tr_core::wind::advance_vine_counter(
                            vine_wind_counter,
                            wind.speed_current,
                        );
                    });

                    // `TileDrawing.Update`'s WindGrid chain (`TileDrawing.cs:377-378`,
                    // S1-G-8): the grid covers the screen's draw area
                    // (`EnsureWindGridSize`, `:7773-7778` — the port's view rect is
                    // the waterfall scan's camera±half shape) and ticks every
                    // update; then the moving player stamps the tiles their
                    // hitbox overlaps (`WindGrid.ScanPlayer`, `WindGrid.cs:73-101`
                    // — the single-player arm; the port's roster is the one local
                    // player). The push family's draw-pass consumers are the
                    // S1-G-1 tile passes (the grass/vine sway reads the grid).
                    {
                        let half = screen * 0.5 / *zoom;
                        let vx0 = ((camera.x - half.x) / TILE_PX).floor() as i32;
                        let vx1 = ((camera.x + half.x) / TILE_PX).ceil() as i32;
                        let vy0 = ((camera.y - half.y) / TILE_PX).floor() as i32;
                        let vy1 = ((camera.y + half.y) / TILE_PX).ceil() as i32;
                        wind.wind_grid.set_size(vx1 - vx0, vy1 - vy0);
                        wind.wind_grid.update();
                        if wind.sway_enabled {
                            if let Some(p) = player.as_ref() {
                                wind.scan_player(
                                    world,
                                    (p.x, p.y),
                                    (p.x + crate::player::WIDTH, p.y + p.height),
                                    (p.vx, p.vy),
                                    (camera.x, camera.y),
                                    (screen.x, screen.y),
                                );
                            }
                        }
                    }

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
                        anim: if style.perturb > 0.0 || legacy_active {
                            // S1-E-19: the legacy engine advances its blue
                            // wave / swipe RNG every frame and re-publishes
                            // its presented values, so the world range must
                            // rebuild per frame in the legacy modes — the
                            // G6 key's contents are unchanged, only this
                            // legacy-mode input rides the existing `anim`
                            // slot.
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
                                        ^ (vine_wind_counter.to_bits() as u32)
                                } else {
                                    0
                                })
                                // S1-G-1: the vine strips' `GetWindGridPush`
                                // (`TileDrawing.cs:9076-9078`) decays on the
                                // grid clock even in calm air — a player's
                                // stamp keeps moving pixels for 20 ticks with
                                // no other key input changing. The clock
                                // rides the key only when a vine thread is
                                // in the view rect (the pass's only grid
                                // consumer), so vine-free worlds keep the
                                // old cache behaviour bit-for-bit.
                                ^ (if wind.sway_enabled && {
                                    let vh = screen * 0.5 / *zoom;
                                    let vx0 = ((camera.x - vh.x) / TILE_PX).floor() as i32 - 1;
                                    let vx1 = ((camera.x + vh.x) / TILE_PX).ceil() as i32 + 1;
                                    let vy0 = ((camera.y - vh.y) / TILE_PX).floor() as i32 - 1;
                                    let vy1 = ((camera.y + vh.y) / TILE_PX).ceil() as i32 + 1;
                                    (vy0..=vy1).any(|ty| {
                                        (vx0..=vx1).any(|tx| {
                                            world.tile(tx, ty).is_some_and(|t| {
                                                t.active()
                                                    && crate::render::vines::is_vine_thread(t.kind)
                                            })
                                        })
                                    })
                                } {
                                    wind.wind_grid.key()
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
                        // S1-H-15: `shade` multiplies every tile tint by the style's
                        // brightness, and the Darkness debuff's `blind` flag drops it
                        // 1.2 → 1.0 (`Lighting.cs:108-116`) — the input must ride the
                        // key or a blind gain/loss keeps the previous frame's tints.
                        brightness: style.brightness.to_bits(),
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
                    draw_phase("sky");
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
                        self.game_update_count,
                        self.mouse_text_color,
                    );
                    // `Main.cs:62217`: `ScreenDarkness.DrawBack` sits right after
                    // `Overlays.Scene.Draw(Background)`.
                    crate::render::screen_fx::draw_darkness_back(renderer, screen, scene_state);
                    draw_phase("world");
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
                            &self.entity_lights,
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
                            *vine_wind_counter,
                            // S1-F-9 slice B: the engine-driven DrawData cases'
                            // inputs (plates 428, party 455, echo sight 541).
                            &self.wiring,
                            crate::world_events::party_is_up(&self.world_events),
                            crate::env::show_invisible(),
                        );
                        *world_key = Some(key);
                    }

                    // S1-G-1 wave C: the tile-crack overlay
                    // (`Main.DrawTileCracks`, `Main.cs:61533-61626`; the
                    // solid-layer sites `TileDrawing.cs:732-733`). It rides
                    // the per-frame world range right after the cached tile
                    // pass — the damage timer advances every frame, so the
                    // crack cannot live inside the G6-cached range (the
                    // `WorldFrameKey` is untouched). The port's stand-in for
                    // the solid-layer site; ANALYSIS "The tile-crack
                    // overlay (S1-G-1 wave C)".
                    crate::render::tile_cracks::draw_mining_cracks(
                        renderer,
                        world,
                        light,
                        &style,
                        draw_cam,
                        anim_time,
                        *mine_target,
                        *mine_timer,
                        *mine_hardness,
                        mine_crack_rng,
                        mine_crack_style,
                        mine_crack_target,
                    );
                    // S1-G-1 wave D: `TileDrawing.PostDrawTiles(solidLayer:
                    // false)`'s pylon / trophy / void-lens passes
                    // (`TileDrawing.cs:423-425`). The sprites ride the
                    // always-rebuilt world-effect range (screen space, like
                    // the leaf gores), so the G6-cached world range and its
                    // key stay untouched; the C# draws them between the tile
                    // layers — the port's fused pass keeps the documented
                    // z-order approximation (ANALYSIS "The pylon/trophy/
                    // void-lens passes"). The `tileFrameCounter[597]/[491]`
                    // stand-ins are the update-count wraps (`Main.cs:19284`,
                    // `:19227` — the animator is unported).
                    {
                        let special_env = crate::render::pylons::SpecialEnv {
                            global_time: anim_time % 3600.0,
                            pylon_counter: self.game_update_count % 64,
                            lens_counter: self.game_update_count % 40,
                            show_invisible: crate::env::show_invisible(),
                        };
                        crate::render::pylons::draw_pylons_trophies_void(
                            renderer,
                            world,
                            light,
                            &style,
                            draw_cam,
                            world_zoom,
                            screen,
                            anim_time,
                            &special_env,
                            game_paused_for_specials,
                            &mut self.entity_lights,
                        );
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

                    draw_phase("rain");
                    rain.draw(renderer, lookups, light, draw_cam, world_zoom, screen);
                    // `Main.DrawDust`'s 268 entries: the flying sand, the same
                    // world-effect range as the rain (`Main.DrawRain`'s draw call
                    // site — world content, under the map batches and the UI).
                    crate::sand::draw(dust, renderer, lookups, light, draw_cam, world_zoom, screen);
                    // The type-213 wheel sparks — `Main.DrawDust`'s generic
                    // 213-draw (self-lit gray, `Dust.cs:2498-2501`), the same
                    // world-content range as the rain/sand dust.
                    crate::minecart::draw(dust, renderer, lookups, draw_cam, world_zoom, screen);
                    crate::render::foliage::draw_leaves(
                        renderer, lookups, light, draw_cam, world_zoom, screen, leaves,
                    );
                    // `Main.DrawWires` (`Main.cs:57028-57030`): the wire overlay
                    // rides the tile batch after the liquids, under the
                    // over-wire entities (the port draws it before the
                    // projectiles, matching `DrawCachedProjsOverWiresUI`).
                    draw_phase("wires");
                    crate::render::wires::draw_wires(
                        renderer, world, light, &style, draw_cam, world_zoom, screen, anim_time,
                    );
                    // `Main.cs:63276-63277` (`DoDraw_WallsTilesNPCs`): the Moon
                    // Lord's death drama pieces + blasts, after the walls and the
                    // WoF/back gore, under the non-solid tiles.
                    crate::render::screen_fx::draw_moonlord_pieces(
                        renderer,
                        lookups,
                        light,
                        draw_cam,
                        world_zoom,
                        screen,
                        scene_state,
                    );
                    crate::render::screen_fx::draw_moonlord_explosions(
                        renderer,
                        lookups,
                        draw_cam,
                        world_zoom,
                        screen,
                        scene_state,
                    );
                    draw_phase("projectiles");
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

                    // `Main.cs:62322-62324`: `ScreenDarkness.DrawFront`,
                    // `MoonlordDeathDrama.DrawWhite` and `ScreenObstruction.Draw`
                    // close the world pass, after the projectiles (`:62251`) and
                    // the over-wire cached projectiles (`:62315`).
                    crate::render::screen_fx::draw_darkness_front(renderer, screen, scene_state);
                    crate::render::screen_fx::draw_white(renderer, screen, scene_state);
                    // `:55` — `PerspectivePlayer.getRect()` in framebuffer pixels.
                    let player_rect = render_player
                        .as_ref()
                        .or(player.as_ref())
                        .map(|p| {
                            let sx = (p.x - draw_cam.x) * world_zoom + screen.x * 0.5;
                            let sy = (p.y - draw_cam.y) * world_zoom + screen.y * 0.5;
                            [
                                sx,
                                sy + p.gfx_off_y * world_zoom,
                                crate::player::WIDTH * world_zoom,
                                crate::player::HEIGHT * world_zoom,
                            ]
                        })
                        .unwrap_or([0.0, 0.0, 0.0, 0.0]);
                    crate::render::screen_fx::draw_obstruction(
                        renderer,
                        lookups,
                        screen,
                        player_rect,
                        scene_state.screen_obstruction,
                    );
                    // `Main.cs:62340-62396`: the combat-text pass closes `DoDraw`,
                    // under the interface. Then `DrawInterface_14_EntityHealthBars`
                    // (`Main.cs:45810`) paints the ordinary mob bars.
                    draw_phase("combat_text");
                    crate::combat_text::draw(
                        renderer,
                        lookups,
                        combat_texts,
                        draw_cam,
                        world_zoom,
                        screen,
                    );
                    draw_phase("entity_health_bars");
                    crate::render::entities::draw_npc_health_bars(
                        renderer,
                        lookups,
                        npcs,
                        light,
                        &style,
                        anim_time,
                        draw_cam,
                        world_zoom,
                        screen,
                    );

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
                            // `Main.cs:55696-55703`: the same layer walk inside the
                            // `mapStyle == 1` branch, clipped to the 240x240 minimap
                            // rect and drawn at
                            // `num46 = ((mapMinimapScale * 0.25 * 2 + 1) / 3)`
                            // capped at 1 (`:55698-55702`). The context's
                            // `mapPosition`/`mapOffset` are the transform-space
                            // `(num12, num13)` and `(num3 + num10, num4 + num11)`
                            // (`:55539-55544` + `:55703`): the raw panel origin plus
                            // the sub-tile centres, which `map_minimap_quad` already
                            // computes for the tile pass, so the clip is its rect.
                            let q = map::map_minimap_quad(
                                view_center,
                                m.minimap_scale,
                                map_scale,
                                screen,
                                world,
                            );
                            let num46 = ((m.minimap_scale * 0.25 * 2.0 + 1.0) / 3.0).min(1.0);
                            let digtoise =
                                crate::map_layers::digtoise_marker(player.as_ref(), &self.projectiles);
                            let near_pylon = player.as_ref().is_some_and(|p| {
                                let (cx, cy) = p.center();
                                tr_core::pylons::is_player_near_a_pylon(
                                    world,
                                    ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32),
                                )
                            });
                            let _ = crate::map_layers::map_layers_draw(
                                renderer,
                                lookups,
                                &crate::map_layers::MapIconInputs {
                                    world,
                                    player: player.as_ref(),
                                    item_drops: &self.items,
                                    digtoise,
                                    pings: crate::map_layers::map_pings(m),
                                    pylons: &map_layer_pylons,
                                    near_pylon,
                                    map: Some(m),
                                    team_based_spawns,
                                    mouse: self.mouse_pos,
                                },
                                [0.0, 0.0],
                                ui::minimap::minimap_origin_raw(screen, map_scale),
                                Some([
                                    q.dest[0],
                                    q.dest[1],
                                    q.dest[0] + q.dest[2],
                                    q.dest[1] + q.dest[3],
                                ]),
                                m.minimap_scale,
                                num46,
                                1.0,
                            );
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
                            scroll: self.front_scroll,
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
                            // The reach (`Player.tileRangeX/Y` +
                            // `TileReachCheckSettings.Simple`): the Extendo Grip /
                            // the Architect's Gizmo Pack, the held Toolbox, the
                            // Journey mode's Far Placement Range power and the
                            // artisan loaf.
                            let reach = player.as_ref().map_or((5, 3), |p| {
                                craft::tile_reach(
                                    p.equipped[..10].contains(&2215)
                                        || p.equipped[..10].contains(&3061),
                                    self.mouse_item.kind == 1923,
                                    self.creative_powers.far_placement_range(),
                                    p.ate_artisan_bread,
                                )
                            });
                            let (adj, in_range) = player
                                .as_ref()
                                .map(|p| {
                                    (
                                        craft::scan_adjacent(world, p, reach),
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
                            // `Recipe.CanCraft`'s two gates (`Recipe.cs:349-356`).
                            let gates = craft::Gates {
                                unlocked_biome_torches: player
                                    .as_ref()
                                    .map_or(false, |p| p.unlocked_biome_torches),
                                // `SpecialSeedFeatures.Mechdusa`
                                // (`SpecialSeedFeatures.cs:119-127`).
                                mechdusa: world.seeds.contains(tr_core::seeds::SeedFlags::REMIX)
                                    && world
                                        .seeds
                                        .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
                            };
                            // `NewCraftingUI.cs:596-611`: the grid shows only the
                            // recipes whose product passes the filterer.
                            self.craft_filters.shown(craft::available_with_gates(
                                inventory,
                                &self.world_chests,
                                &in_range,
                                &adj,
                                &zones,
                                self.craft_filter.as_ref(),
                                None,
                                &gates,
                            ))
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
                            self.craft_filters.active_flags(),
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
                            // `Player.HasNPCBannerBuff`'s table — the Monster
                            // Banner's hover list (`Main.cs:21360-21388`).
                            &self.scene_metrics.npc_banner_buff,
                            // `Main.mouseNPCIndex/mouseNPCType` — the carried
                            // housing head (`Main.cs:45200-45216`).
                            ui::housing::MouseNpc {
                                index: self.mouse_npc_index,
                                ty: self.mouse_npc_type,
                            },
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
                    // `DrawInterface_13_AchievementCompletePopups`
                    // (`Main.cs:44779-44783`, `:46143-46146`): the unlock
                    // toasts draw over the interface — after the fancy UI
                    // (layer 12) and the achievements overlay, before the
                    // entity health bars (layer 14). The menu twin
                    // (`Main.cs:48457`) draws them over the legacy menu too;
                    // the port's popups only ever exist in-world (the drain
                    // runs on the world branch and `SaveAndQuit` clears the
                    // tracker, `WorldGen.cs:6714`). S1-E-26 wave A.
                    if menu.is_none() && front.is_none() && !crate::env::no_hud() {
                        if let Some(name) = ui::notifications::draw(
                            renderer,
                            lookups,
                            screen,
                            &mut self.notifications,
                            self.mouse_pos,
                            self.mouse_text_color,
                            left_release,
                        ) {
                            // `OnMouseOver` → `IngameFancyUI.
                            // OpenAchievementsAndGoto` (`InGamePopups.cs:138`,
                            // `UI/IngameFancyUI.cs`): open the overlay and
                            // scroll the list to the row (`UIList.Goto`'s
                            // `ViewPosition = item.Top.Pixels`) — applied
                            // after the gpu block releases the field borrows.
                            popup_goto = Some(name);
                        }
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
                                let digtoise = crate::map_layers::digtoise_marker(
                                    player.as_ref(),
                                    &self.projectiles,
                                );
                                let near_pylon = player.as_ref().is_some_and(|p| {
                                    let (cx, cy) = p.center();
                                    tr_core::pylons::is_player_near_a_pylon(
                                        world,
                                        ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32),
                                    )
                                });
                                let hover = crate::map_layers::map_layers_draw(
                                    renderer,
                                    lookups,
                                    &crate::map_layers::MapIconInputs {
                                        world,
                                        player: player.as_ref(),
                                        item_drops: &self.items,
                                        digtoise,
                                        pings: crate::map_layers::map_pings(map),
                                        pylons: &map_layer_pylons,
                                        near_pylon,
                                        map: Some(map),
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

                    // The debug window (`M2-debug-window`, F4): the static
                    // Phase-1 rows only — frame timing, the player, the tile
                    // under the cursor, the entity counts.
                    if debug_overlay {
                        let (tile_x, tile_y) = cursor_tile(debug_mouse, screen, *camera, *zoom);
                        let tile = world.tile(tile_x, tile_y).map(|t| DebugTile {
                            x: tile_x,
                            y: tile_y,
                            active: t.active(),
                            kind: t.kind,
                            wall: t.wall,
                            frame_x: world.tile_frame_x(tile_x, tile_y),
                            frame_y: world.tile_frame_y(tile_x, tile_y),
                            slope: t.slope(),
                            liquid: t.liquid,
                        });
                        let input = DebugWindowInput {
                            screen,
                            camera: *camera,
                            zoom: *zoom,
                            mouse: debug_mouse,
                            dt,
                            history: frame_history.iter().copied().collect(),
                            draw_calls: last_stats.draw_calls,
                            instances: last_stats.instances,
                            player: player.as_ref().map(DebugPlayer::from_player),
                            tile,
                            npcs: npcs.len(),
                            projectiles: debug_projectiles,
                            items: debug_items,
                            dust: dust.slots().iter().filter(|d| d.active).count(),
                            gore: debug_gore,
                        };
                        draw_debug_window(renderer, &input);
                    }

                    // The debug cheat-sheet panel (`debug_panel.rs`) — independent
                    // of F1. Compiled only with the `debug` feature.
                    #[cfg(feature = "debug")]
                    if self.debug_panel.open {
                        crate::debug_panel::draw(
                            renderer,
                            lookups,
                            screen,
                            &self.debug_panel,
                            npcs,
                            world,
                            self.stepper.paused,
                            self.player.as_ref().is_some_and(|p| p.debug_god_mode),
                            rain.is_raining(),
                            sandstorm.happening,
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
                            return None;
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
        // The popup click's `OpenAchievementsAndGoto` (S1-E-26 wave A) —
        // the field borrows are released, so the overlay open + scroll lands
        // for the next frame's draw.
        if let Some(name) = popup_goto {
            self.open_achievements_and_goto(name);
        }
        Some((stats, tile_stats))
    }
}
