//! The in-world branch of the frame orchestrator — `DoUpdateInWorld`'s
//! port (S1-B-8 phase 2): one fn per world phase, in the C#'s order
//! (`Main.cs:17745-17822`), plus the draw-half pre-snapshot spine. The
//! `phase("…")` trace calls stay at their exact sites so the S1-B-1 and
//! S1-B-3 gate pins keep detecting a reordering.

use glam::Vec2;

use crate::app::App;
use crate::audio;
use crate::npc::NpcKind;
use crate::player;
use tr_core::TILE_PX;
use crate::world_sim::clock::advance_time_with_events;

impl App {
    /// The pre-gate interface cluster (`Main.cs:61781`/`:62466` + the
    /// minimap/bestiary/chest/crafting/inventory handlers and the hotbar
    /// scales): the UI stays live while paused, so it runs before the
    /// pause gate. The statements are the branch's own, in order.
    fn in_world_click_cluster(&mut self) {
    // `Main.Draw`'s stack-split accelerator (`Main.cs:61781-61807`),
    // ticked before the interface handlers read the state.
    self.tick_stack_split();
    // `MinimapFrame.Update` runs before the inventory's click handling
    // (`MinimapFrameManager.cs:45`, `Main.cs:41037`'s `ItemSlot.Handle`).
    self.handle_minimap_clicks();
    self.handle_fullmap_ping();
    self.handle_bestiary_clicks();
    // The town-chatter panel's Close button + hover tick
    // (`NPCChatPanel.DrawButtons`, `NPCChatPanel.cs:88-162`) — the press
    // is consumed before the chest/crafting/inventory handlers see it.
    self.handle_npc_chat_clicks();
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
    // `Main.DrawInventory`'s shop-close rule (`Main.cs:41784-41787`) — the
    // interface pass closes the shop when the inventory closes.
    self.update_shop_close();
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
    }

    /// S1-B-3 — the in-game pause gate (`Main.cs:17194-17201`). Yields the
    /// frame's `game_paused`.
    fn in_world_pause_gates(&mut self) -> bool {
                let paused = self.can_pause_game();
                if paused {
                    self.do_update_while_paused();
                }
                self.game_paused = paused;
        paused
    }

    /// `DoUpdateInWorld`'s gate (`Main.cs:17278`,
    /// `ShouldUpdateEntities() && WorldUpdateStepper.ShouldUpdateWorld()`).
    fn in_world_step_gate(&mut self, paused: bool) -> bool {
        let step_allowed = !paused
            && self.should_update_entities()
            && self
                .stepper
                .should_update_world(self.step_input_down, self.step_input_was);
        step_allowed
    }

    /// The step body's head (`Main.cs:17747`/`:17753` + the crafting
    /// filter's `RecipeFilterHack` gate).
    fn world_step_head(&mut self) {
    self.game_update_count = self.game_update_count.wrapping_add(1);
    // `WorldUpdateStepper.DrawnThisUpdate = false` at the step
    // head (`Main.cs:17747`, right after `_gameUpdateCount++`).
    self.stepper.drawn_this_update = false;
    // `Main.tileSolid[379] = false` (`Main.cs:17753`): the bubble
    // is not solid for the entity half of the tick — the liquid
    // pass re-arms it (`Liquid.cs:451`, S1-L-14).
    self.liquids.clear_bubble_for_entities();
    // `CraftingUI.RecipeFilterHack`'s gate (`CraftingUI.cs:112-121`)
    // plus `NewCraftingUI.OnDeactivate`'s clear (`:912-914`): the
    // station filter only lives while the crafting page is up.
    if !self.crafting_open {
        self.craft_filter = None;
    }
    // `Main.UpdateWorld_Players` (`Main.cs:17825-17859`): the player
    // pass. The sleeping counts it exports (`:17844-17859`) gate the
    // clock's rate, so they land right after it.
    }

    /// `Main.UpdateWorld_Players` (`Main.cs:17825-17859`) + the fused
    /// `DecideOnNewMusic`/boss-scan block (`Main.cs:12224-12430`) the port
    /// runs in the players→NPCs gap.
    fn players_phase(&mut self, dt: f32, screen: Vec2) {
    self.phase("players");
    self.update_player(dt);
    // `UpdateTimeRate` (`Main.cs:6199-6218`): the rate is ×5 while
    // every active player is fully asleep
    // (`PlayerSleepingHelper.FullyFallenAsleep`, 120 ticks).
    self.world.all_players_asleep = self
        .player
        .as_ref()
        .is_some_and(|p| p.sleeping && p.sleep_time >= 120);
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
        town_npcs: self.scene_metrics.town_npc_count,
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
    }

    /// `Main.UpdateWorld_SpawnNPCs` (`Main.cs:17878`) + `UpdateWorld_NPCs`
    /// (`:17898`) — the port runs spawn and update in one pass.
    fn npcs_phase(&mut self, dt: f32) {
    if !self.is_client() {
        self.phase("npcs");
        self.update_npcs(dt);
    }
    }

    /// `Main.UpdateWorld_Gores` (`Main.cs:17971-17996`, called at
    /// `:17771-17774`): the 600-slot walk, between the NPCs and the
    /// projectiles. Its rolls ride the stage's own named stream
    /// (`Main.SwapRandom("UpdateGores")`).
    fn gores_phase(&mut self) {
        self.phase("gores");
        {
            let rng = self.rngs.stream(crate::app::RngKey::UpdateGores);
            let wind_for_visuals = self.wind.speed_current;
            self.gore.update(
                rng,
                &self.world,
                &crate::gore_seam::AppGoreCollision,
                wind_for_visuals,
            );
        }
    }

    /// `Main.UpdateWorld_Projectiles` (`Main.cs:17976`) — **before** items.
    fn projectiles_phase(&mut self, dt: f32) {
    // `Main.UpdateWorld_Projectiles` (`Main.cs:17976`) runs **before**
    // `UpdateWorld_Items` (`:18004`).
    self.phase("projectiles");
    self.update_projectiles(dt);
    }

    /// `Main.UpdateWorld_Items` (`Main.cs:18004`).
    fn items_phase(&mut self, dt: f32) {
    self.phase("items");
    self.update_items(dt);
    }

    /// `Main.UpdateWorld_Dust` (`Main.cs:18047`): the minecart sparks ride
    /// this per-frame slot.
    fn dust_phase(&mut self, dt: f32) {
    // `Main.UpdateWorld_Dust` (`Main.cs:18047`): the minecart sparks
    // ride this per-frame slot (the same slot the rain/sand dust get in
    // `world_sim::scene_scan::update_scene`).
    self.phase("dust");
    let (sw, sh) = self.screen_size();
    // The tick rides a client stream: the riding player's
    // `spark_rng` (the cart's own) when a player exists, else the
    // NamedRngs' global stream (the C#'s single `Main.rand`
    // equivalent - `self.rngs.global()`, `app/rng.rs`); no new
    // stream instances either (S1-E-3).
    let max_dust = self.gfx_quality.max_dust_to_draw;
    let show_sand = self.sandstorm.show_sandstorm_visuals(
        self.scene_metrics.zone_sandstorm,
        tr_core::scene::is_desert_bg_style(self.bg_style),
        self.bg_delay,
    );
    let player_vy = self.player.as_ref().map_or(0.0, |p| p.vy);
    let dust = &mut self.dust;
    match self.player.as_mut() {
        Some(p) => crate::minecart::tick(
            dust,
            &mut crate::app::SplitmixRand(&mut p.spark_rng),
            max_dust,
            self.camera,
            Vec2::new(sw, sh),
            self.zoom,
            &self.world,
            player_vy,
            show_sand,
        ),
        None => crate::minecart::tick(
            dust,
            self.rngs.global(),
            max_dust,
            self.camera,
            Vec2::new(sw, sh),
            self.zoom,
            &self.world,
            player_vy,
            show_sand,
        ),
    }
    }

    /// `Main.UpdateWorld_Time` (`Main.cs:18083`) — the clock's own named
    /// stream carries the dawn/dusk event rolls.
    fn time_phase(&mut self, dt: f32) {
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
    }

    /// `WorldGen.UpdateWorld`'s walk (`WorldGen.cs:71495-71630`):
    /// `Wiring.UpdateMech` → `TileEntity.PerformUpdates` →
    /// `Liquid.UpdateLiquid` → the growth loops, in that order.
    fn wiring_phase(&mut self, dt: f32) {
    // `WorldGen.UpdateWorld` (`WorldGen.cs:71495-71630`): the
    // `UpdateWorld_WorldGenAndInvasion` wrapper (`Main.cs:18099`) runs
    // *after* the time pass and walks `Wiring.UpdateMech` →
    // `TileEntity.PerformUpdates` → `Liquid.UpdateLiquid` → the growth
    // loops, in that order.
    self.phase("wiring");
    self.update_wiring(dt);
    }
    fn tile_entities_phase(&mut self, dt: f32) {
    self.phase("tile_entities");
    self.update_tile_entities(dt);
    }
    fn liquids_phase(&mut self, dt: f32) {
    self.phase("liquids");
    self.update_liquids(dt);
    }
    fn growth_phase(&mut self, dt: f32) {
    self.phase("growth");
    self.update_growth(dt);
    }

    /// `DoUpdate_AnimateWaterfalls` (`Main.cs:19545-19566`): the pre-gate
    /// Animate cluster — it ticks while paused.
    fn waterfalls_phase(&mut self, dt: f32, screen: Vec2) {
    // `DoUpdate_AnimateWaterfalls` (`Main.cs:19545-19566`): the
    // `WaterfallManager.UpdateFrame`/`FindWaterfalls` pass — the
    // pre-gate Animate cluster (`:17146`), so it ticks while paused.
    self.phase("waterfalls");
    self.update_waterfalls(dt, screen);
    }

    /// `Main.AnimateTiles` (`Main.cs:17150`, body `:18516-19304`): the
    /// pre-gate Animate cluster's tile animator, right after
    /// `DoUpdate_AnimateWalls` (`:17149`) — like the waterfalls it ticks
    /// while paused, on the shared 60 Hz accumulator (S1-G-1f wiring).
    /// The two `Main.rand` drift draws (453/456) and the weather-vane bob
    /// roll ride the bare-`Main.rand` stand-in (`NamedRngs::global()`),
    /// the same stream the ambience scheduler draws; the
    /// `WorldGen.isGeneratingOrLoadingWorld` gate is false in the frame
    /// loop (the port's worldgen completes before it starts).
    fn animate_tiles_phase(&mut self, dt: f32) {
    self.phase("animate_tiles");
    let wind = self.wind.speed_current;
    let rng = self.rngs.global();
    let step = &mut self.tile_anim_step;
    let animator = &mut self.tile_anim;
    step.tick(dt, |_| {
        animator.animate_tiles(wind, rng, false);
    });
    step.cap_backlog();
    }

    /// S1-B-3: the post-gate weather cluster (`Main.cs:17222`'s
    /// `Rain.MakeRain`, `:17228-17231`'s dayRate walks) — frozen while the
    /// pause gate holds, live on stepper-blocked frames.
    fn weather_cluster(&mut self, dt: f32, screen: Vec2) {
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

    /// `Main.UpdateWindyDayState` (`Main.cs:12990-13027`): the sticky
    /// windy-day/storm flags, the pre-gate cluster (`:17182`).
    fn windy_day_phase(&mut self) {
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
    }

    /// S1-B-3: the post-gate tail — `Main.AmbienceServer.Update`'s own
    /// `!gamePaused` arm (`Main.cs:17214`) + `SkyManager.Update` (`:17139`).
    fn ambience_phase(&mut self, dt: f32) {
                    self.phase("ambience");
                    {
    let ambience = &mut self.ambience;
    // The scheduler's draws ride the bare-`Main.rand` stand-in
    // (`Main.cs:17214-17216` runs `AmbienceServer.Update` outside
    // every swap) — the shared `NamedRngs::global()` stream, which
    // chat commands and menu rolls also draw between frames.
    let ambience_rng = self.rngs.global();
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

    /// S1-B-3 (batch review fix 1): the tile-edit and net halves ride the
    /// step gate — a stepper-blocked frame cannot mine, place or sync.
    fn editing_net_phase(&mut self, dt: f32, screen: Vec2) {
                    self.phase("editing");
                    self.update_editing(dt, screen);
                    // `Main.cs:18077` — `CombatText.UpdateCombatText()` inside the
                    // world update, on the same step gate as the strike that
                    // spawns the numbers.
                    crate::combat_text::update_all(&mut self.combat_texts);
                    self.phase("net");
                    self.net_tick(dt);
    }

    /// The map reveal keeps the pause gate only (its C# source is the
    /// draw-side lighting export, `LightingEngine.cs:246-270`).
    fn map_phase(&mut self, dt: f32) {
                    self.phase("map");
                    if let Some(m) = self.map.as_mut() {
    // `Main.mapTimeMax` (`Main.cs:801`) — the gfx-quality
    // budget the countdown re-arms from (`:61818-61821`).
    m.map_time_max = self.gfx_quality.map_time_max;
    if self.light_mode != tr_core::lighting::LightMode::Color {
        // S1-E-19 wave B: the legacy engine exports from its own
        // `ProcessArea` (`LegacyLighting.cs:341-358`, wired in
        // `frame/draw.rs`); the update side only runs the per-frame
        // `Main.mapTime` countdown (`Main.cs:61818-61821`). The modern
        // `ExportToMiniMap` countdown lives inside `WorldMap::update`.
        m.tick_map_time();
    } else {
        m.update(&self.world, self.player.as_ref(), &self.light, dt);
    }
    // `PingMapLayer.Draw`'s expiry (`PingMapLayer.cs:36-39`):
    // the markers older than 15 s drop while the map is up.
    m.age_pings(dt);
    // `MapUpdateQueue.Update()` (`Main.cs:61862`): the edit-side
    // refresh drains through `WorldMap.UpdateType`; the changed
    // tiles re-upload (`MapRenderer.QueueChange`'s analog).
    let changed = self.map_queue.update(&self.world, m);
    m.push_dirty(changed);
                    }
    }

    /// The world-branch tail's camera walks (`Main.cs:17814-17815`, S1-B-4)
    /// and `SceneState.Update` (`:17816`) — they freeze with the world step.
    fn camera_scene_tail(&mut self, dt: f32, screen: Vec2) {
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

    /// `Main.chatMonitor.Update()` (`Main.cs:17282`): the chat containers'
    /// `_timeLeft` decay — one tick per game update, **outside** the
    /// `DoUpdateInWorld` gate (`:17278` wraps only the world body), so it
    /// runs on stepper-blocked frames too. The `CanPauseGame` early return
    /// (`:17194-17199`) sits before it, so the in-game pause freezes the
    /// timers — the `!paused` arm. The front-end/focus returns
    /// (`:17166-17175`, `:17127-17138`) skip the whole branch already.
    fn chat_phase(&mut self) {
    self.phase("chat");
    // `RemadeChatMonitor.Update` (`:115-129`): every container's
    // `ChatMessageContainer.Update` (`ChatMessageContainer.cs:45-52`) —
    // the 600-tick corner-visibility countdown.
    self.chat.update();
    }

    /// The gfx-quality budgets (`Main.cs:17056-17101`, S1-B-6).
    fn gfx_quality_apply(&mut self) {
    // The gfx-quality budgets (`Main.cs:17056-17101`, S1-B-6): the
    // `qaStyle` overrides, `maxDustToDraw`, `Gore.goreTime`, the
    // liquid pass pair and the legacy render phases.
    let mut gfx = std::mem::take(&mut self.gfx_quality);
    gfx.apply(self);
    self.gfx_quality = gfx;
    }

    /// The in-world branch, phase by phase in the C#'s order — the
    /// orchestrator the S1-B-1 order pin walks. Every gate arm is the
    /// branch's own, unchanged.
    pub(super) fn update_world_frame(&mut self, dt: f32, screen: Vec2) {
        self.in_world_click_cluster();
        let paused = self.in_world_pause_gates();
        let step_allowed = self.in_world_step_gate(paused);
        if step_allowed {
            self.world_step_head();
            self.players_phase(dt, screen);
            self.npcs_phase(dt);
            self.gores_phase();
            self.projectiles_phase(dt);
            self.items_phase(dt);
            self.dust_phase(dt);
            self.time_phase(dt);
            self.wiring_phase(dt);
            self.tile_entities_phase(dt);
            self.liquids_phase(dt);
            self.growth_phase(dt);
        }
        self.waterfalls_phase(dt, screen);
        // `Main.AnimateTiles` (`Main.cs:17150`) — the pre-gate Animate
        // cluster, right after `DoUpdate_AnimateWalls` (`:17149`); the
        // waterfall pass above stands in for the walls animator's slot.
        self.animate_tiles_phase(dt);
        if !paused {
            self.weather_cluster(dt, screen);
        }
        self.windy_day_phase();
        if !paused {
            self.ambience_phase(dt);
        }
        if step_allowed {
            self.editing_net_phase(dt, screen);
        }
        if !paused {
            self.map_phase(dt);
        }
        if step_allowed {
            self.camera_scene_tail(dt, screen);
        }
        if !paused {
            self.chat_phase();
        }
        self.gfx_quality_apply();
    }

    /// The draw-half pre-snapshot spine (S1-B-8 phase 2): the drawn-flag,
    /// the net cluster and the two `update_camera_at_draw` walks (the C#'s
    /// named steps between the world branch and the draw region). The pure
    /// field snapshots between them stay inline in the orchestrator — they
    /// are the draw region's borrow scaffolding, consumed below; the spine's
    /// statements are state-disjoint from them, so the consolidation is a
    /// no-op (the screenshot pairs prove it).
    pub(super) fn draw_half_pre_snapshots(&mut self, dt: f32) {
    if self.menu.is_none() && self.front.is_none() {
        self.stepper.drawn_this_update = true;
    }
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
            // `Main.DoDraw_UpdateCameraPosition` (`Main.cs:62590-62649`, S1-B-4):
            // the camera snaps/lerps/clamps at DRAW time, ahead of every read
            // this frame's render makes of it — the update half above kept the
            // previous frame's position (the C#'s update reads it too).
            let mut camera_state = std::mem::take(&mut self.camera_state);
            crate::world_sim::camera::update_camera_at_draw(self, &mut camera_state);
            self.camera_state = camera_state;
            // `Main.DoDraw_UpdateCameraPosition` (`Main.cs:62590-62649`, S1-B-4):
            // the camera snaps/lerps/clamps at DRAW time, ahead of every read
            // this frame's render makes of it.
            let mut camera_state = std::mem::take(&mut self.camera_state);
            crate::world_sim::camera::update_camera_at_draw(self, &mut camera_state);
            self.camera_state = camera_state;
            // `ChildSafety.Disabled = GoreVisualsAllowed` (`Main.cs:4101`,
            // `:4349`): the gate follows the same live settings copy the cursor
            // tint reads — synced before the chest reads below pin `&mut self`
            // out of reach.
            let menu_blood = self.menu.as_ref().map(|m| m.settings.blood);
            self.sync_child_safety(menu_blood);
    }
}
