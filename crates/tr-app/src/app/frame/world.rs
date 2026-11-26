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
use crate::ui;
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
    // The fullscreen map's close button (`Main.cs:56024-56033`) — after the
    // ping detector, like the C#'s order inside `DrawMap` (`:55486` < `:56024`).
    self.handle_fullmap_close();
    self.handle_bestiary_clicks();
    // The debug commands overlay (`UIDebugCommandsList`, W4-d85) — an
    // `IngameFancyUI` state like the bestiary; its clicks consume the press
    // before the chest/crafting/inventory handlers see it.
    self.handle_debug_commands_clicks();
    self.handle_debug_loadouts_clicks();
    // The bestiary info-page scrollbar's drag frame (`UIScrollbar.DrawSelf`,
    // `UIScrollbar.cs:109-112`): while the thumb is grabbed and the left
    // button is held, the view follows the pointer. W4-e23 remainder.
    if self.bestiary_open && self.mine_down && self.bestiary_ui.info_scroll_drag.is_some() {
        let (sw, sh) = self.screen_size();
        self.bestiary_info_scrollbar_drag(Vec2::new(sw, sh), self.mouse_pos.y);
    }
    // The bestiary's hover-sound ticks (W4-e23 remainder 4): the
    // `GroupOptionButton.DrawSelf:212-223` / `UIImageButton.MouseOver:61`
    // per-hover `PlaySound(12)` edges, once per open frame.
    if self.bestiary_open {
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let options = self.bestiary_ui.options;
        let at = self.bestiary_ui.at;
        let fired = crate::ui::bestiary_hover::tick(
            &mut self.bestiary_ui.hover_sounds,
            screen,
            self.mouse_pos,
            options,
            &self.bestiary,
            &self.bestiary_ui.working,
            at,
        );
        if fired {
            self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
        }
    } else {
        self.bestiary_ui.hover_sounds.reset();
    }
    // The achievements overlay's clicks (`UIAchievementsMenu` rides
    // `IngameFancyUI`): the scrollbar/Back/toggles consume the press
    // before the chest/crafting/inventory handlers see it (W4-g53).
    self.handle_achievements_clicks();
    // The town-chatter panel's Close button + hover tick
    // (`NPCChatPanel.DrawButtons`, `NPCChatPanel.cs:88-162`) — the press
    // is consumed before the chest/crafting/inventory handlers see it.
    self.handle_npc_chat_clicks();
    // The world housing banners' right-click kick-out (`DrawNPCHousesInWorld`,
    // `Main.cs:40824-40829`, W4-g21 batch 4 host 53) — the banner hover box
    // consumes the right press before the chest/crafting/inventory handlers.
    self.handle_house_banner_clicks();
    // W4-f30 — the Dryad's Stardew text override (`NPCChatPanel.
    // OverrideChatTextWithShenanigans`, `NPCChatPanel.cs:195-213`): the
    // holding-cola reminder + the post-gift reroll state machine. The C#
    // runs it in the panel's draw (`PrepareText`); the port folds it into
    // the per-frame update (the text is state, the draw reads it).
    self.update_dryad_chat_override();
    // The chest panel's slot dispatch (`ChestUI.DrawSlots`' per-cell
    // `ItemSlot.Handle`, `ChestUI.cs:385`) runs before the crafting
    // and inventory handlers and consumes the press when a chest
    // slot, the shift transfer or a chest tile took it.
    if !self.handle_chest_clicks() && !self.handle_crafting_clicks() && !self.handle_journey_clicks() {
        self.handle_inventory_clicks();
    }
    // The Journey menu's per-frame tail (W4-c24): the animation tick, the
    // hover cues and the close rules (`CreativeUI.Update`/`Draw`,
    // `Main.cs:17497`/`:45643-45664`).
    self.journey_tick();
    // `Main.cs:62466-62473` — the stack-split decay, after the
    // interface's MouseUI pass.
    self.decay_stack_split();
    // `Main.DrawInventory`'s shop-close rule (`Main.cs:41784-41787`) — the
    // interface pass closes the shop when the inventory closes.
    self.update_shop_close();
    // W4-c46 final — the hotbar scale lerp lives INSIDE `GUIHotbarDrawInner`
    // (`Main.cs:43604-43611`), which early-returns for the ghost
    // (`:43588`): the scales freeze while the hardcore ghost flies.
    if !self.player.as_ref().is_some_and(|p| p.ghost) {
        for (i, scale) in self.hotbar_scales.iter_mut().enumerate() {
            let target = if i == self.selected { 1.0 } else { 0.75 };
            if *scale < target {
                *scale = (*scale + 0.05).min(target);
            } else if *scale > target {
                *scale = (*scale - 0.05).max(target);
            }
        }
    }
    // S1-B-3 — the in-game pause gate (`Main.cs:17194-17201`):
    // `if (CanPauseGame()) { DoUpdate_WhilePaused(); gamePaused = true;
    // return; } gamePaused = false;`. The click cluster above it is the
    // pre-gate interface pass (DW-1.3: the UI stays live while paused).
    }

    /// W4-f30 — `NPCChatPanel.OverrideChatTextWithShenanigans`
    /// (`NPCChatPanel.cs:195-213`): the Dryad's cola reminder + the post-gift
    /// reroll state machine. The C# runs it in the panel's `PrepareText`
    /// (per draw); the port folds it into the per-frame update. The reroll
    /// arm (`:206`) re-rolls `GetChat()` — the full chat context is rebuilt
    /// here (the `talk_to_npc` open arm's shape).
    fn update_dryad_chat_override(&mut self) {
        if self.talk_npc.is_none() {
            return;
        }
        let held = self.inventory.get(self.selected).map_or(0, |s| s.kind);
        match crate::dryad_stardew::override_dryad_chat_text(self, held) {
            Some(text) if text.is_empty() => {
                // The reroll sentinel: re-roll GetChat() for the talk NPC.
                if let Some(i) = self.talk_npc {
                    if let Some(npc) = self.npcs.get(i) {
                        let has_cola = self.inventory.iter().any(|s| s.kind == 5275);
                        let rng = self.rngs.global();
                        let mut draw = |n: i32| rng.next_max(n);
                        let mut ctx = crate::npc_chat::ChatContext {
                            npcs: &self.npcs,
                            rng: &mut draw,
                            day_time: self.world.day_time,
                            time: self.world.time,
                            blood_moon: self.world.blood_moon,
                            eclipse: self.world.eclipse,
                            slime_rain: self.world.slime_rain,
                            hard_mode: self.world.hard_mode,
                            moon_phase: self.world.moon_phase,
                            remix_world: false,
                            raining: self.world.raining,
                            storming: false,
                            windy_day: false,
                            zone_graveyard: false,
                            party_is_up: false,
                            crimson: false,
                            downed_boss1: self.world.downed_eoc,
                            downed_boss2: self.world.downed_eow,
                            downed_boss3: self.world.downed_skeletron,
                            downed_moonlord: false,
                            dd2_downed_t1: self.world.dd2_downed_t1,
                            dd2_downed_t2: self.world.dd2_downed_t2,
                            dd2_downed_t3: self.world.dd2_downed_t3,
                            dd2_ongoing: self.world.dd2_event_ongoing,
                            lanterns_up: false,
                            player_life: self.player.as_ref().map_or(0, |p| p.hp),
                            player_life_max: self.player.as_ref().map_or(0, |p| p.max_hp),
                            player_base_life_max: self.player.as_ref().map_or(0, |p| p.max_hp),
                            player_defense: 0,
                            player_male: true,
                            player_head: 0,
                            player_deaths_pve: self.player.as_ref().map_or(0, |p| p.deaths_pve as i32),
                            player_luck: 0.0,
                            player_has_cola: has_cola,
                            angler_quest_finished: false,
                            angler_quests_finished: self.player.as_ref().map_or(0, |p| p.angler_quests_finished),
                            golfer_score: self.player.as_ref().map_or(0, |p| p.golfer_score),
                        };
                        let text = crate::npc_chat::get_chat(npc, &mut ctx);
                        self.npc_chat_text = text;
                    }
                }
            }
            Some(text) => {
                // The holding-cola reminder (`:211`).
                self.npc_chat_text = text;
            }
            None => {}
        }
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
    // `Main.UpdateCreativeGameModeOverride` (`Main.cs:16847` → `:17294-
    // 17305`): once per game update, the Journey difficulty slider's
    // strength becomes `Main._gameModeDifficultyOverride`; outside a
    // Journey world it clears (the world's own GameMode answers). The
    // port's Journey stand-in is the session difficulty (the same one
    // `journey_available` reads — a Journey character only joins a
    // Journey world, `UIWorldSelect.cs:177-180`).
    self.game_mode_difficulty_override = self
        .creative_powers
        .game_mode_difficulty_override(self.active_player_difficulty() == 3);
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
    // The `dropItemCheck` right-click edge (`Player.cs:5147`'s
    // `Main.mouseRight && !mouseInterface && Main.mouseRightRelease`): the
    // press edge outside every slot hit, captured BEFORE the player pass —
    // `update_editing` refreshes `place_prev_down` only after it. The
    // `mouseInterface` stand-in covers every drawn slot surface: the grid/
    // closed hotbar, the coin/ammo rows, the equipment columns and the
    // trash cell (`Main.DrawInventory`'s `ItemSlot.Handle` sites).
    let over_ui_slot = self.slot_at_mouse().is_some()
        || self.coin_ammo_at_mouse().is_some()
        // S1-W4-g81 residuals — the wiring radial's petal hover writes
        // `player.mouseInterface = true` (`WiresUI.cs:499`); the port's
        // draw-side state folds into the same suppression stand-in.
        || self.wires_radial.hovering
        || (self.inventory_open && {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            let m_h = self.main_m_h();
            let cell = self.lookups.ui_back.map_or(52.0, |b| b.width as f32);
            let chest_or_shop = self.open_container() >= 0 || self.shop_npc.is_some();
            ui::inventory::equip_cell_at(
                self.mouse_pos,
                screen,
                m_h,
                self.equip_page,
                cell,
                ui::inventory::ExtraAccessoryShow::of(self.player.as_ref()),
            )
            .is_some()
                || ui::inventory::trash_slot_at(self.mouse_pos, chest_or_shop)
        });
    self.drop_right_edge =
        self.place_down && !self.place_prev_down && !over_ui_slot;
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
        // g67b IsItRaining fix: the music rain branches read
        // `Main.cloudAlpha > 0` (`Main.cs:12859/:12880/:12932/:12973`),
        // NOT the `Main.raining` event flag — the post-StopRain cloud ramp
        // keeps the rain track until the clouds clear.
        rain: self.rain.is_it_raining(),
        boss_music,
        clock: self.world.time,
        eclipse: self.world.eclipse
            && self.camera.y
                < self.world.world_surface as f32 * TILE_PX + screen.y * 0.5,
        // W4-f33 — the Torch God minigame track (`Main.cs:12542`).
        in_torch_god_minigame: self.scene_metrics.in_torch_god_minigame,
        // `flag24` (`Main.cs:12200`): the Slime Rain event track 48
        // (`:12654-12657`) — the raw flag; the ladder arm applies the
        // graveyard/blood-moon/surface gates (W4-f39 co-lane compose).
        slime_rain: self.world.slime_rain,
        shimmer: self.scene_metrics.zone_shimmer,
        town_npcs: self.scene_metrics.town_npc_count,
        // — the M1.4-BG-20 ambience-track inputs (`Main.cs:11527-11600`) —
        // The RAW `Main.cloudAlpha` (`:11544`'s `num11` reads the unclamped
        // static; the sky-tint `visual_alpha` clamp was a deviation —
        // observable above 1 under a heavy storm / TR_RAIN pin, W4-d130).
        cloud_alpha: self.rain.cloud_alpha(),
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
        // — the W4-f39 closeout inputs (`Main.cs:12200`/`:12654`/
        // `:12982-12989`) —
        pumpkin_moon: self.world.pumpkin_moon,
        snow_moon: self.world.snow_moon,
        remix_world: self.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
        // `screenPosition.Y / 16 < worldSurface + 10`: the camera is the
        // view centre, the C# screenPosition the top-left (`camera.rs`).
        moon_tail_band: (self.camera.y - screen.y * 0.5) / TILE_PX
            < self.world.world_surface as f32 + 10.0,
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
        // `Main.cs:12748-12751` — the UG-desert 61 arm ahead of the desert
        // 21 (W4-c33: the S1-L-1 wall zone was never fed into the ladder).
        underground_desert: self.scene_metrics.zone_underground_desert,
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
        // g67b IsItRaining fix: the town quiet-sky gate reads
        // `Main.cloudAlpha == 0` (`Main.cs:12670/:12674`), not the event flag.
        quiet_sky: !self.rain.is_it_raining() && !self.wind.happy_windy_day,
        menu: false,
        menu_flags: Default::default(),
        menu_micro_flags: Default::default(),
        menu_credits: false,
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
            // The spawn context is owned (`SpawnCtx`), read before the
            // `rngs`/`dust`/`gore` mutable borrows below.
            let ctx = self.spawn_ctx();
            let rng = self.rngs.stream(crate::app::RngKey::UpdateGores);
            let wind_for_visuals = self.wind.speed_current;
            // W4-d79 PITCH SEAL — the case-39 Drip seam: the pool draws
            // the `Next(2)` style argument and the façade's pitch draw on
            // the SAME `UpdateGores` stream, at the C# statement position
            // (`Gore.cs:953`/`:965` — the `Main.cs:17771` swap wraps both).
            let mut sound = crate::gore_seam::AppGoreSound(&mut self.audio);
            // W4-g68 dust seam: the lava leaf's five-dust puff
            // (`Gore.cs:1119-1122`) spawns into `Main.dust` on the same
            // stage stream — the C# runs both inside
            // `SwapRandom("UpdateGores")` (`Main.cs:17771-17774`).
            let dust = &mut self.dust;
            self.gore.update(
                rng,
                &self.world,
                &crate::gore_seam::AppGoreCollision,
                wind_for_visuals,
                &mut sound,
                Some((dust, &ctx)),
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

    /// `Main.UpdateWorld_Dust` (`Main.cs:18047`): the whole dust pool rides
    /// this per-frame slot — the W4-d50 wiring dispatches the per-type
    /// physics chain (`tr_core::dust_physics::type_chain`) as the shared
    /// loop's hook (`minecart::tick`).
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
    // The chain's `Main.player[Main.myPlayer]` snapshot (27/45/76/268/235):
    // velocity, wet and Center; no player reads the default-constructed
    // fallback (`DustPlayerSnap::default`, the dust_physics module docs).
    let player = self.player.as_ref().map_or_else(
        crate::minecart::DustPlayerSnap::default,
        |p| {
            let (cx, cy) = p.center();
            crate::minecart::DustPlayerSnap {
                vx: p.vx,
                vy: p.vy,
                wet: p.wet,
                center: Vec2::new(cx, cy),
            }
        },
    );
    // The child-drain `NewDust` ctx (`Dust.cs:77-98` statics).
    let spawn_ctx = self.spawn_ctx();
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
            player,
            show_sand,
            &spawn_ctx,
        ),
        None => crate::minecart::tick(
            dust,
            self.rngs.global(),
            max_dust,
            self.camera,
            Vec2::new(sw, sh),
            self.zoom,
            &self.world,
            player,
            show_sand,
            &spawn_ctx,
        ),
    }
    }

    /// `Main.UpdateWorld_Time` (`Main.cs:18083`) — the clock's own named
    /// stream carries the dawn/dusk event rolls.
    fn time_phase(&mut self, dt: f32) {
    // `Main.UpdateWorld_Time` (`Main.cs:18083`) — `Main.UpdateTime`
    // runs after the entity half, never before it.
    self.phase("time");
    // `CreditsRollEvent.UpdateTime` (`Main.cs:64968` →
    // `CreditsRollEvent.cs:35-38`): one decrement per UpdateTime call,
    // independent of the day rate — the C# runs UpdateTime even while
    // the creative freeze holds `Main.time` (dayRate 0), so the tick
    // sits OUTSIDE the port's rate gate (W4-g51 id-140).
    self.world.tick_credits_roll();
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
        // W4-g34: `AnimateTiles`' trailing `AnimateTiles_CritterCages()`
        // (`Main.cs:19303`) — same 60 Hz tick, same `Main.rand` stream, gated
        // on the `Main.critterCage` proximity flag the tile draw raised last
        // frame (`TileDrawing.cs:474` clears it at the non-solid pass head).
        let cage_anim = &mut self.cage_anim;
        let critter_cage = self.critter_cage;
        // W4-f14: the table-driven cage animator (`Main.*CageFrame`) rides the
        // SAME tail call and stream; the draw clears/sets `critter_cage` and
        // reads the tables (`TileDrawing.cs:474`/`:5672-5907`).
        let cages = &mut self.cages;
        step.tick(dt, |_| {
            animator.animate_tiles(wind, rng, false);
            cage_anim.animate(critter_cage, rng);
            cages.animate_critter_cages(rng);
        });
        step.cap_backlog();
    }

    /// S1-B-3: the post-gate weather cluster (`Main.cs:17222`'s
    /// `Rain.MakeRain`, `:17228-17231`'s dayRate walks) — frozen while the
    /// pause gate holds, live on stepper-blocked frames.
    fn weather_cluster(&mut self, dt: f32, screen: Vec2) {
                    // `WorldGen.BackgroundsCache.UpdateFlashValues`
                    // (`Main.cs:17218`): the style-change flash decay, ahead
                    // of the `Rain.MakeRain` slot (`:17220`).
                    self.update_bg_flash_values();
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
        // g67b IsItRaining fix: the scheduler's sunny/calm gates read
        // `Main.IsItRaining` = `cloudAlpha > 0` (`Main.cs:2597`,
        // `AmbienceServer.cs:32/:50`), NOT the event flag.
        is_raining: self.rain.is_it_raining(),
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
    // W4-f39 closeout: `SkyManager.Update` (`Main.cs:17139`) also walks
    // the SlimeSky — the event's activation edges ride `World.slime_rain`
    // (the port's stand-in for the `Activate`/`Deactivate` calls inside
    // `StartSlimeRain`/`StopSlimeRain`), the update ×`max(1, dayRate)`.
    // `Main.screenPosition.Y` is the unscaled view top (`camera.y -
    // screenHeight/2`, the sky pass's convention).
    {
        let (sw, sh) = self.screen_size();
        let sky_ctx = crate::slime_sky::SlimeSkyCtx {
            screen_position_y: self.camera.y - sh * 0.5,
            screen_height: sh,
            max_tiles_x: self.world.width,
            max_tiles_y: self.world.height,
            world_surface: self.world.world_surface,
        };
        let _ = sw;
        self.slime_sky.update(
            dt,
            self.world.slime_rain,
            self.world.day_rate(),
            &sky_ctx,
        );
    }
    }

    /// W4-c39 — `SkyManager.Update` (`Main.cs:17139`, `SkyManager.cs:24-46`):
    /// the two event skies step `max(1, dayRate)` times per 60 Hz tick. The
    /// lantern sky reads `LanternNight.LanternsUp`; the party sky's opacity
    /// ramps toward the town-NPC presence (`SceneState.cs:106`). The
    /// activation edges ride the schedulers' UpdateTime (weather.rs).
    fn event_skies_phase(&mut self, dt: f32) {
        self.phase("event_skies");
        let lanterns_up = crate::world_events::lanterns_up(&self.world_events);
        let wind = self.wind.speed_current;
        let day_rate = self.world.day_rate();
        let town_present = self.scene_metrics.town_npc_count > 0;
        let sh = self.screen_size().1;
        let ctx = crate::event_skies::SkyCtx {
            screen_position_y: self.camera.y - sh * 0.5 / self.zoom,
            screen_height: sh,
            max_tiles_x: self.world.width,
            max_tiles_y: self.world.height,
            world_surface: self.world.world_surface,
        };
        self.event_skies.ramp_party_opacity(town_present);
        // The 60 Hz accumulator (the ambience precedent): the sky update is
        // a per-tick walk, not a per-frame one.
        self.event_skies_accumulator += dt;
        const STEP: f32 = 1.0 / 60.0;
        while self.event_skies_accumulator >= STEP {
            self.event_skies_accumulator -= STEP;
            for _ in 0..day_rate.max(1) {
                self.event_skies.update_lanterns(lanterns_up, wind, &ctx);
                self.event_skies.update_party(day_rate, wind, &ctx);
            }
        }
    // W4-c40: `SkyManager.Update` (`Main.cs:17139`, `SkyManager.cs:24-46`)
    // runs the active skies `max(1, dayRate)` times per update — the
    // SandstormSky tint fade (`SandstormSky.cs:24-47`) rides the same loop
    // the ambience entities use (`AmbienceCtx::day_rate`). The manager's
    // `!IsActive()` removal (`:39-42`) is the sky's own below-zero
    // deactivate; the `FocusHelper.PauseSkies` head is this phase's
    // `!paused` gate (the f40 lantern-sky fold).
    let sky_iters = self.world.day_rate().max(1);
    for _ in 0..sky_iters {
        self.sandstorm_sky.update(dt);
    }
    // W4-f40: `SkyManager.Update`'s `LanternSky` iteration (`SkyManager.cs:
    // 24-46`, `max(1, dayRate)` per 60 Hz tick). The sky's private RNG draws
    // on no named stream; the opacity walk + lantern drift ride the same
    // step gate as the ambience (the `FocusHelper.PauseSkies` fold — module
    // docs). The dedicated accumulator keeps the cadence frame-rate
    // independent (the ambience precedent).
    {
        let lanterns_up = crate::world_events::lanterns_up(&self.world_events);
        let wind = self.wind.speed_current;
        let (max_x, surface) = (self.world.width, self.world.world_surface);
        let day_rate = self.world.day_rate().max(1);
        let sky = &mut self.lantern_sky;
        self.lantern_sky_step.tick(dt, |_| {
            for _ in 0..day_rate {
                sky.update(lanterns_up, wind, max_x, surface);
            }
        });
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
                    // `Main.cs:18078` — `PopupText.UpdateItemText()` right after
                    // the combat-text walk (`UpdateWorld_FloatingText`, W4-g57):
                    // the reforge popups' rise/fade clock. Visual only — no RNG.
                    self.update_popup_texts(screen);
                    self.phase("net");
                    self.net_tick(dt);
    }

    /// `PopupText.UpdateItemText` (`Main.cs:18078`, W4-g57): the pool's
    /// per-tick walk beside the combat text. `TargetScale = UIScale /
    /// RenderZoom.X` (`PopupText.cs:88`) — the port's applied UI scale over
    /// the same `bg_zoom * zoom` product the draw half uses; the per-slot
    /// sizes are `MeasureString(displayText)` (`:672`), the renderer-free
    /// `text::ui_font_*` twins with the bitmap estimate fallback. No RNG.
    fn update_popup_texts(&mut self, screen: Vec2) {
        // `UpdateItemText` walks all 20 slots and refreshes `numActive` even
        // when none is active (`PopupText.cs:752-764`) — no early return.
        let ui_scale = self.settings.applied_ui_scale(screen.x, screen.y);
        let render_zoom = App::forced_minimum_zoom((screen.x, screen.y)) * self.zoom;
        let mut target_scale = if render_zoom > 0.0 { ui_scale / render_zoom } else { 0.0 };
        if target_scale == 0.0 {
            target_scale = 1.0; // `Main.cs:62398-62400`'s zero guard
        }
        let env = crate::popup_text::Env {
            disco: self.disco,
            master_color: self.master_color,
            mouse_text_color: self.mouse_text_color,
            global_time_wrapped_hourly: self.anim_time % 3600.0,
            grav_dir: 1.0, // the port's player gravDir is unported (always 1)
        };
        let mut sizes = [[0.0f32; 2]; crate::popup_text::MAX_POPUPS];
        for (i, slot) in sizes.iter_mut().enumerate() {
            if let Some(t) = self.popup_texts.texts.get(i).filter(|t| t.active) {
                *slot = [
                    crate::ui::text::ui_font_width(&self.lookups, &t.display_text, 1.0)
                        .unwrap_or(t.display_text.chars().count() as f32 * 8.0),
                    crate::ui::text::ui_font_height(&self.lookups, 1.0).unwrap_or(8.0),
                ];
            }
        }
        crate::popup_text::update_all(&mut self.popup_texts, target_scale, env, &sizes);
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
        // `Main.mapEnabled` — the export's first gate
        // (`LightingEngine.cs:248`).
        m.update(
            &self.world,
            self.player.as_ref(),
            &self.light,
            self.settings.map_enabled,
            dt,
        );
    }
    // `PingMapLayer.Draw`'s expiry (`PingMapLayer.cs:36-39`):
    // the markers older than 15 s drop while the map is up.
    m.age_pings(dt);
    // `MapUpdateQueue.Update()` (`Main.cs:61862`): the edit-side
    // refresh drains through `WorldMap.UpdateType`; the changed
    // tiles re-upload (`MapRenderer.QueueChange`'s analog).
    let changed = self.map_queue.update(&self.world, m);
    // `MapUpdateQueue.UpdateTiles`/`UpdateAreas` (`MapUpdateQueue.cs:82`,
    // `:97`): every changed tile queues through `MapRenderer.QueueChange`
    // — the 2048-per-target threshold arms `refreshMap` (`:295-298`).
    for (x, y) in &changed {
        m.queue_change(*x, *y);
    }
    m.push_dirty(changed);
                    }
    }

    /// `Main.cs:61863-61899` — the reveal queue's driver, on the DRAW half
    /// (`DoDraw`, after `MapUpdateQueue.Update()` `:61862` — the call site
    /// is `frame/draw.rs`, W4-f46 salvage per research/84 §3): the sweep
    /// (`:61865`) gates only on `!gameMenu && !generatingWorld`, so it
    /// keeps walking while paused; only the `updateMap` consume arm
    /// (`:61886`) checks `!gamePaused`. The draw site's in-world gate
    /// implies the menu/gen gates; the staged `.map` upload (`load_map`)
    /// rides the `!Main.loadMap` gate (`:61863`). The field-borrow shape:
    /// the draw region's 55-field destructure holds `map`/`world`/
    /// `settings` out of `&mut self` reach, so the call site passes the
    /// parts; `section_manager`/`debug_unlock_map` stay disjoint `self`
    /// fields.
    pub(crate) fn map_reveal_draw(
        map: &mut Option<crate::map::WorldMap>,
        section_manager: &mut crate::map_sections::MapSections,
        debug_unlock_map: &mut i32,
        world: &tr_core::World,
        player_pos: [f32; 2],
        map_enabled: bool,
        paused: bool,
        r: &mut tr_render::SpriteRenderer,
        gpu: &tr_render::Gpu,
    ) {
        if !Self::map_reveal_load_gate(map) {
            return; // `:61863`'s `!Main.loadMap` — blocks the WHOLE block
        }
        // `MapRenderer.checkMap` (`MapRenderer.cs:322`, inside
        // `DrawToMap_Section`): the target exists before the first paint —
        // `ensure_map_texture` is idempotent, so it arms ahead of the walk
        // even when `mapEnabled` is off (the C# sweep gates on menu/gen
        // only, `Main.cs:61865`).
        r.ensure_map_texture(gpu, world.width as u32, world.height as u32);
        let drawn = Self::map_reveal_sweep_fields(
            map,
            section_manager,
            debug_unlock_map,
            world,
            player_pos,
            5.0, // `:61871`'s 5 ms `Stopwatch`
        );
        if let Some(m) = map.as_mut() {
            for (x, y) in drawn {
                // `:61877`: `MapRenderer.DrawToMap_Section(x, y)` — the
                // section's row-major colour cache blits into the map
                // texture (`MapRenderer.cs:306-355`, W4-f46 salvage).
                m.draw_to_map_section(r, gpu, world, x, y);
            }
        }
        Self::map_reveal_consume_fields(map, section_manager, map_enabled, paused);
    }

    /// `Main.cs:61863`: the driver's `!Main.loadMap` head gate — a staged
    /// `.map` upload blocks the WHOLE block (sweep + consume). Shared by
    /// the live path and the GPU-free witness.
    pub(crate) fn map_reveal_load_gate(map: &Option<crate::map::WorldMap>) -> bool {
        map.as_ref().is_some_and(|m| !m.load_map)
    }

    /// The `updateMap` / `ChangesQueued` consumer (`Main.cs:61886-61898`),
    /// GPU-free — the driver's flag-lifecycle half; the witnesses exercise
    /// it without a renderer (the live path calls the field form directly).
    #[cfg(test)]
    pub(crate) fn map_reveal_consume(&mut self, paused: bool) {
        Self::map_reveal_consume_fields(
            &mut self.map,
            &mut self.section_manager,
            self.settings.map_enabled,
            paused,
        );
    }

    /// The field-borrow form of [`Self::map_reveal_consume`].
    pub(crate) fn map_reveal_consume_fields(
        map: &mut Option<crate::map::WorldMap>,
        section_manager: &mut crate::map_sections::MapSections,
        map_enabled: bool,
        paused: bool,
    ) {
        // `MapRenderer.DrawToMap`'s first gate (`MapRenderer.cs:130-133`):
        // `!Main.mapEnabled` returns before any queue work — the
        // `updateMap` slot still nulls (`Main.cs:61894` runs after the
        // call), but the change lists keep their entries.
        // W4-f46 salvage (research/84 §3): the C# `else if (ChangesQueued)`
        // attaches to the WHOLE `HasValue && !gamePaused` condition — a
        // paused frame with an armed `updateMap` still falls through to the
        // empty-rect draw. The old `if paused { return }` skipped it.
        let consume = map.as_ref().is_some_and(|m| m.update_map.is_some()) && !paused;
        if consume {
            // `:61888-61892`: the `refreshMap` consume rides INSIDE the
            // `updateMap.HasValue && !gamePaused` arm, BEFORE the DrawToMap
            // call — clear the flag, reset the section sweep, then draw.
            // The C# runs these regardless of `mapEnabled` (the gate is
            // inside DrawToMap).
            if map.as_ref().is_some_and(|m| m.refresh_map) {
                if let Some(m) = map.as_mut() {
                    m.refresh_map = false; // `:61890`
                }
                section_manager.clear_map_draw(); // `:61891`
            }
            if let Some(m) = map.as_mut() {
                // `MapRenderer.DrawToMap(updateMap.Value)` (`:61893`):
                // the port's texture sync rides the dirty list in
                // `prepare`; the queue half clears the change lists and
                // completes the staged upload (`MapRenderer.cs:148-160`,
                // `:196-199`).
                if map_enabled {
                    m.draw_to_map_queues();
                }
                m.update_map = None; // `:61894`
            }
        } else if map.as_ref().is_some_and(|m| m.changes_queued()) {
            // `:61896-61898`: `else if (MapRenderer.ChangesQueued)` →
            // `DrawToMap(Rectangle.Empty)` — runs while paused too (the
            // pause blocked the first arm) and whenever no rect is armed;
            // same queue drain, no area walk.
            if map_enabled {
                if let Some(m) = map.as_mut() {
                    m.draw_to_map_queues();
                }
            }
        }
    }

    /// The GPU-free STATE half of the sweep (`Main.cs:61867-61883`): mark
    /// sections drawn, unlock each while `DebugOptions.unlockMap != 0 &&
    /// SectionLoaded`, zero the option on an empty pass without a pending
    /// refresh. Returns the drawn sections for the paint wrapper; the
    /// witnesses exercise this core (no renderer needed — the live path
    /// calls the field form directly). `budget_ms`
    /// stands in for the C# `Stopwatch`'s 5 ms (`:61871`) — the driver
    /// passes 5.0; a test passes a large budget to drain the whole spiral
    /// in one call (W4-f46 salvage, research/84 §3).
    #[cfg(test)]
    pub(crate) fn map_reveal_sweep_core(&mut self, budget_ms: f64) -> Vec<(i32, i32)> {
        let pos = self
            .player
            .as_ref()
            .map(|p| [p.x, p.y])
            .unwrap_or([0.0, 0.0]);
        Self::map_reveal_sweep_fields(
            &mut self.map,
            &mut self.section_manager,
            &mut self.debug_unlock_map,
            &self.world,
            pos,
            budget_ms,
        )
    }

    /// The field-borrow form of [`Self::map_reveal_sweep_core`] — the draw
    /// region's destructure holds `map`/`world` out of `&mut self` reach.
    pub(crate) fn map_reveal_sweep_fields(
        map: &mut Option<crate::map::WorldMap>,
        section_manager: &mut crate::map_sections::MapSections,
        debug_unlock_map: &mut i32,
        world: &tr_core::World,
        player_pos: [f32; 2],
        budget_ms: f64,
    ) -> Vec<(i32, i32)> {
        let start = std::time::Instant::now();
        let mut num3 = 0;
        let mut drawn = Vec::new();
        while start.elapsed().as_secs_f64() * 1000.0 < budget_ms {
            let Some((x, y)) = section_manager.get_next_map_draw(player_pos) else {
                break;
            };
            // `:61873-61876`: `DebugOptions.unlockMap != 0 &&
            // Main.sectionManager.SectionLoaded(x, y)`.
            if *debug_unlock_map != 0 && section_manager.section_loaded(x, y) {
                if let Some(m) = map.as_mut() {
                    m.unlock_section(world, x, y, *debug_unlock_map == 2);
                }
            }
            drawn.push((x, y));
            num3 += 1;
        }
        // `:61880-61883`: the exhausted sweep disarms the debug option
        // unless a refresh is still pending.
        let refresh = map.as_ref().is_some_and(|m| m.refresh_map);
        if num3 == 0 && !refresh {
            *debug_unlock_map = 0;
        }
        drawn
    }

    /// The world-branch tail's camera walks (`Main.cs:17814-17815`, S1-B-4)
    /// and `SceneState.Update` (`:17816`) — they freeze with the world step.
    fn camera_scene_tail(&mut self, dt: f32, screen: Vec2) {
                    // `Chest.UpdateChestFrames()` (`Main.cs:17810`): the lid
                    // animation ticks after `UpdateWorld_ServerOrClient` and
                    // before the camera walks — the port's slot is the head
                    // of this step-gated tail (S1-P-i43). The eat chime
                    // (`Chest.cs:3628-3631`, `PlaySound(7, chest centre)`)
                    // plays for every cue the walk reports — the int overload
                    // consumes one case-7 `Next(-10, 11)` pitch draw on the
                    // BARE stream (the `:17810` call site sits outside every
                    // SwapRandom block, the Lead's faithful-cue decision).
                    // Dormant until the eat timer's producer lands.
                    let open = self.open_container();
                    let eat_cues =
                        crate::chest::update_chest_frames(&mut self.world_chests, open);
                    for j in eat_cues {
                        let (cx, cy) = self.world_chests[j].tile;
                        self.audio.play_chest_eat_sound(
                            cx as f32 * 16.0 + 16.0,
                            cy as f32 * 16.0 + 16.0,
                            self.rngs.global(),
                        );
                    }
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

    /// `Main.DoUpdate_AnimateBackgrounds` (`Main.cs:17052`): the four
    /// forest-set hill animators, right before the gfx-quality budgets
    /// (`:17056`) — outside the pause gate, like the C#.
    fn animate_backgrounds_phase(&mut self) {
    self.phase("animate_backgrounds");
    self.update_animate_backgrounds();
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
        self.animate_backgrounds_phase();
        if !paused {
            self.weather_cluster(dt, screen);
        }
        self.windy_day_phase();
        if !paused {
            self.ambience_phase(dt);
            self.event_skies_phase(dt);
        }
        if step_allowed {
            self.editing_net_phase(dt, screen);
        }
        if !paused {
            self.map_phase(dt);
        }
        // W4-f46 salvage (research/84 §3): the reveal driver
        // (`Main.cs:61863-61899`) sits in `DoDraw`, after
        // `MapUpdateQueue.Update()` (`:61862`) — the call moved to the
        // draw half (`frame/draw.rs`, ahead of `WorldMap::prepare`).
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
    // `WorldUpdateStepper.DrawnThisUpdate = true` (`Main.cs:62478`) sits at
    // the END of `DoDraw`, AFTER the draw passes — not at the draw-half head.
    // The W4-g13 dust emitters read `FocusHelper.Allow*` (= `!DrawnThisUpdate`)
    // DURING the draw, so the mark must stay clear until the passes finish;
    // `frame()` sets it right after `draw_region` returns (the C# position).
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

#[cfg(test)]
mod w4_e18_reveal_driver_tests {
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::App;
    use crate::map::WorldMap;
    use crate::map_sections::MapSections;

    /// A 400x300 world → a 2x2 section grid (`WorldGen.cs:7150`), with the
    /// map live and a player parked in section (0,0).
    fn rig() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = tr_core::World::new("reveal".into(), 400, 300);
        app.section_manager = MapSections::new(400, 300);
        app.map = Some(WorldMap::new(&app.world));
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 50.0 * 16.0;
        p.y = 50.0 * 16.0;
        app.player = Some(p);
        app
    }

    /// W4-e18 first half (written, not run — code-only): the sweep
    /// (`Main.cs:61871-61879`) walks `GetNextMapDraw` and marks each section
    /// drawn; with `DebugOptions.unlockMap != 0` it reveals every loaded
    /// section (`:61873-61876`). The 2x2 grid exhausts in one 5 ms pass —
    /// the option zeroes only on the NEXT pass, where `num3 == 0`
    /// (`:61880-61883`): the frame that draws the last sections keeps it.
    #[test]
    fn w4_e18_the_sweep_walks_and_reveals_every_loaded_section() {
        let mut app = rig();
        app.debug_unlock_map = 1; // the flat-255 mode
        let drawn = app.map_reveal_sweep_core(10_000.0);
        assert_eq!(drawn.len(), 4, "the 2x2 grid drains in one call");
        assert_eq!(drawn[0], (0, 0), "the player's section draws first");
        assert_eq!(app.section_manager.sections_left(), 0);
        // All four sections drawn + revealed.
        for y in 0..2 {
            for x in 0..2 {
                assert!(app.section_manager.map_section_drawn(x, y));
            }
        }
        let m = app.map.as_ref().unwrap();
        assert!(m.tiles.iter().any(|t| t.light > 0), "the unlock arm revealed");
        // The pass that drew the last sections had num3 > 0 — the option
        // stays armed until an empty pass (`:61880`).
        assert_eq!(app.debug_unlock_map, 1);
        assert!(app.map_reveal_sweep_core(10_000.0).is_empty());
        assert_eq!(app.debug_unlock_map, 0, "the empty pass zeroes it");
    }

    /// W4-e18 first half (written, not run — code-only): an EMPTY sweep pass
    /// does NOT zero `DebugOptions.unlockMap` while a refresh is pending
    /// (`:61880`'s `&& !Main.refreshMap`); clearing the refresh lets the
    /// next empty pass zero it.
    #[test]
    fn w4_e18_a_pending_refresh_keeps_the_debug_option_armed() {
        let mut app = rig();
        app.debug_unlock_map = 2;
        app.map_reveal_sweep_core(10_000.0); // exhaust the 2x2 grid
        app.map.as_mut().unwrap().refresh_map = true;
        app.map_reveal_sweep_core(10_000.0); // num3 == 0, but the refresh holds
        assert_eq!(app.debug_unlock_map, 2, "the refresh holds the option");
        app.map.as_mut().unwrap().refresh_map = false;
        app.map_reveal_sweep_core(10_000.0);
        assert_eq!(app.debug_unlock_map, 0, "no refresh → the empty pass zeroes");
    }

    /// W4-e18 first half (written, not run — code-only): the `updateMap`
    /// consume (`Main.cs:61886-61894`) — with `refreshMap` armed it clears
    /// the flag and resets the section sweep (`ClearMapDraw`, `:61890-61891`)
    /// BEFORE the draw, then nulls `updateMap` (`:61894`). The reset refills
    /// the budget so the next sweep re-walks from the player's section.
    #[test]
    fn w4_e18_the_refresh_consume_resets_the_sweep_and_nulls_update_map() {
        let mut app = rig();
        // Exhaust the sweep first so the reset is observable.
        app.map_reveal_sweep_core(10_000.0);
        assert!(app.section_manager.map_section_drawn(0, 0));
        let m = app.map.as_mut().unwrap();
        m.refresh_map = true;
        m.update_map = Some([40, 40, 100, 60]);
        app.map_reveal_consume(false);
        let m = app.map.as_ref().unwrap();
        assert!(!m.refresh_map, "`:61890`");
        assert!(m.update_map.is_none(), "`:61894`");
        // ClearMapDraw refilled the budget + cleared the drawn bits.
        assert!(!app.section_manager.map_section_drawn(0, 0));
        assert_eq!(app.section_manager.sections_left(), 4, "the budget refilled");
    }

    /// W4-e18 first half, EXTENDED by the W4-f46 union (written, not run —
    /// code-only, research/84 §7): while paused the sweep still runs
    /// (`:61865` gates only menu/gen) but the `updateMap` consume is
    /// skipped (`:61886`'s `!Main.gamePaused`) — the slot stays armed and
    /// `refreshMap` is not consumed. The `else if (ChangesQueued)` arm
    /// (`:61896-61898`) attaches to the WHOLE condition: a paused frame
    /// with queued changes STILL drains them (the f46 fall-through).
    #[test]
    fn w4_e18_paused_skips_the_consume_but_not_the_sweep() {
        let mut app = rig();
        let m = app.map.as_mut().unwrap();
        // Queue FIRST: `QueueChange` is a no-op while `refreshMap` is
        // armed (`MapRenderer.cs:290`), so the entry must land before it.
        m.queue_change(50, 50); // a pending change in target (0,0)
        m.refresh_map = true;
        m.update_map = Some([40, 40, 100, 60]);
        app.map_reveal_sweep_core(10_000.0);
        app.map_reveal_consume(true);
        // The sweep walked (sections drawn) ...
        assert!(app.section_manager.map_section_drawn(0, 0));
        // ... but the consume arm did not run.
        let m = app.map.as_ref().unwrap();
        assert!(m.refresh_map, "the paused consume is skipped");
        assert!(m.update_map.is_some(), "the slot stays armed");
        // ... and the ChangesQueued else-arm DID run while paused: the
        // queue drained, the draw tail completed (`MapRenderer.cs:196`).
        assert!(!m.changes_queued(), "the paused else-arm drains the queues");
        assert!(m.map_ready, "the else-arm's DrawToMap tail ran");
    }

    /// W4-e18 first half (written, not run — code-only): the staged `.map`
    /// upload gate (`Main.cs:61863`'s `!Main.loadMap`) blocks the whole
    /// driver while `load_map` is armed — no sweep, no consume.
    #[test]
    fn w4_e18_the_load_map_gate_blocks_the_driver() {
        let mut app = rig();
        let m = app.map.as_mut().unwrap();
        m.load_map = true;
        m.update_map = Some([40, 40, 100, 60]);
        // The gate predicate refuses — the driver returns before BOTH
        // halves (the GPU wrapper's first `if` reads this same fn).
        assert!(!App::map_reveal_load_gate(&app.map), "`:61863`'s !Main.loadMap");
        assert!(!app.section_manager.map_section_drawn(0, 0), "no sweep");
        assert!(app.map.as_ref().unwrap().update_map.is_some(), "no consume");
        // The tail's `loadMap = false` (`MapRenderer.cs:198`) lifts it.
        app.map.as_mut().unwrap().draw_to_map_queues();
        assert!(App::map_reveal_load_gate(&app.map), "the draw tail lifts the gate");
    }

    /// W4-e18 first half (written, not run — code-only): `DrawToMap`'s first
    /// gate is `Main.mapEnabled` (`MapRenderer.cs:130-133`). With the map
    /// disabled the `refreshMap` consume still runs (`Main.cs:61888-61892`
    /// sits OUTSIDE DrawToMap) and `updateMap` still nulls (`:61894` runs
    /// after the call), but the change lists keep their entries — the
    /// queue drain is skipped.
    #[test]
    fn w4_e18_map_disabled_skips_the_queue_drain_but_not_the_refresh_consume() {
        let mut app = rig();
        app.settings.map_enabled = false;
        let m = app.map.as_mut().unwrap();
        // Queue before arming the refresh: `QueueChange` is a no-op while
        // `refreshMap` is armed (`MapRenderer.cs:290`).
        m.queue_change(50, 50); // a pending change in target (0,0)
        m.refresh_map = true;
        m.update_map = Some([40, 40, 100, 60]);
        assert!(m.changes_queued());
        app.map_reveal_consume(false);
        let m = app.map.as_ref().unwrap();
        assert!(!m.refresh_map, "the refresh consume runs regardless");
        assert!(m.update_map.is_none(), "`:61894` nulls after the call");
        assert!(m.changes_queued(), "the disabled DrawToMap keeps the lists");
        // Enabling lets the next consume drain them.
        app.settings.map_enabled = true;
        app.map.as_mut().unwrap().update_map = Some([40, 40, 100, 60]);
        app.map_reveal_consume(false);
        assert!(!app.map.as_ref().unwrap().changes_queued());
    }
}
