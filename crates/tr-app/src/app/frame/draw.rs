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
use crate::render::sky::draw_sky;
use crate::render::tiles::draw_tiles;
use crate::render::tiles::TileDrawStats;
use crate::ui;
use crate::ui::text::{border_string, hud_text, hud_text_width};

use super::draw_phase;

/// `Main.DoUpdate_AnimateCursorColors`' masterColor / `DoUpdate_AnimateTileGlows`'
/// martianLight + demonTorch bounce (`Main.cs:18225-18235`/`:18240-18261`):
/// `value += dir * step`, clamped to `[0, 1]` with the direction flipping at
/// each rail. W4-e7 lane 23 — the shared oscillator the three global-flicker
/// pulses ride (the per-tile torch flicker itself is the scanner's, d56/e8).
/// RNG-free. Returns the new `(value, dir)`.
fn bounce_oscillator(value: f32, dir: f32, step: f32) -> (f32, f32) {
    let mut value = value + dir * step;
    let mut dir = dir;
    if value > 1.0 {
        value = 1.0;
        dir = -1.0;
    }
    if value < 0.0 {
        value = 0.0;
        dir = 1.0;
    }
    (value, dir)
}

impl App {
    /// The buff bar's cancel gate — `Main.mouseRight && Main.mouseRightRelease`
    /// (`Main.cs:43333`) with the vanilla's end-of-draw latch recompute
    /// (`:62458-62465`: `mouseRightRelease = !mouseRight`). Called once per
    /// interface draw; the conjunction is true only on the press frame
    /// after a released frame, never while held.
    pub(crate) fn buff_cancel_gate(&mut self) -> bool {
        let gate = self.place_down && self.ui_right_release_latch;
        self.ui_right_release_latch = !self.place_down; // `:62458-62465`
        gate
    }

    /// `Main.cs:62150-62153`: the fullscreen map's draw head —
    /// `talkNPC >= 0 || sign >= 0 || (playerInventory &&
    /// !CaptureManager.Instance.Active)` → `Player.ToggleInv()`. The port's
    /// open paths already clear the dialog/inventory
    /// (`try_open_fullscreen_map`), but a dialog or inventory opened WHILE
    /// the map is up must close on the next drawn frame — the C# runs this
    /// every draw, ahead of `DrawMap`. With the map already open the call
    /// falls through `ToggleInv`'s first arm (`Player.cs:4972-4978`, the
    /// map-close, reached only when the map is SHUT) to the talkNPC arm
    /// (`:5005-5016`: `SetTalkNPC(-1)` + text clear + `playerInventory =
    /// false`) or the inventory-close arm (`:5033-5056`) — both leave the
    /// map open. The capture gate's `!Active` arm keeps the inventory while
    /// a capture renders the interface; the sign editor has no port state.
    pub(crate) fn close_dialog_behind_fullmap(&mut self) {
        if !self.settings.map_enabled || !self.map.as_ref().is_some_and(|m| m.open) {
            return;
        }
        let dialog_open = self.talk_npc.is_some();
        let inv_open = self.inventory_open && !self.capture.interface.active;
        if dialog_open || inv_open {
            self.talk_npc = None; // `SetTalkNPC(-1)` (`:5007`)
            self.npc_chat_text.clear(); // `:5009`
            self.inventory_open = false; // `:5015`/`:5035`
        }
    }

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
            // fix/player-motion-blur: the draw reads the RAW fixed-step
            // position, exactly like the C#. `Main.DrawPlayers` hands
            // `player.position` to the renderer (`Main.cs:22298` →
            // `LegacyPlayerRenderer.DrawPlayerFull`, the final draw at
            // `LegacyPlayerRenderer.cs:468-475` is `position + netOffset +
            // (0, gfxOffY)` — no blend with `oldPosition`), and the layers
            // truncate the screen position to whole pixels per component
            // (`(int)(drawinfo.Position.X - Main.screenPosition.X + ...)` —
            // `PlayerDrawLayers.cs:1907/1939`). `Main.screenPosition` is
            // itself `(int)`-truncated at draw time
            // (`DoDraw_UpdateCameraPosition`, `Main.cs:62646-62647`, ported in
            // `world_sim::camera::update_camera_at_draw`). The port's old
            // `Player::interpolated(residual)` blend was invented: it moved
            // the sprite to sub-pixel positions BETWEEN the snapped integer
            // camera steps, so every rendered frame re-rasterised the
            // nearest-sampled art on a shifted grid — the "character smears
            // during any movement" bug. C# motion is a 60 Hz whole-pixel
            // step; the render is now the same. `gfxOffY` stays a separate
            // draw offset (`draw_player`'s `gfx_off_y`), not baked into `y`.
            // The dead player renders nothing until the respawn (vanilla draws
            // no corpse); the hardcore ghost draws its own sheet after the
            // cached world pass (W4-c46, `draw_ghost` below).
            // W4-d115 wiring round — `LegacyPlayerRenderer.cs:433-448`:
            // `shadowDodgeCount` steps ±1 per DRAW frame (cap 30/floor 0,
            // keyed on the buff-scan `shadowDodge` flag). The C# mutates
            // the live Player inside the renderer; the port steps the
            // live field here, before the render clone below, so the
            // clone, the G6 key and the ghost draw all read the stepped
            // value. Dead players render nothing here (the hardcore ghost
            // is the W4-c46 sheet below), so their counter holds — the sim
            // tick and the respawn never touch it (writer census: the
            // renderer is the field's ONLY writer).
            if let Some(p) = self.player.as_mut().filter(|p| !p.dead) {
                p.step_shadow_dodge_draw_counter();
            }
            let render_player = self.player.as_ref().filter(|p| !p.dead).cloned();
            // The body rows — the item-use rows included — are all computed by
            // `Player::update_frames` (`PlayerFrame()`) inside the fixed step.
            let player_key = render_player.as_ref().map_or(0, |p| {
                let mut k = p.render_key();
                // W4-d115 wiring round: while the shadow-dodge ghosts
                // draw, their two `Next(-10,11)` alpha rolls are pinned
                // per frame (the sparkle-seed convention), so the frame
                // count changes the cached range's pixels — it rides the
                // key only while the counter is live (the vine-grid gate
                // convention: a ghost-free frame keeps the pre-wiring
                // key bit-for-bit).
                if p.shadow_dodge_count > 0.0 {
                    k ^= (*frame_count as u32).wrapping_mul(0x2545_F491);
                }
                k
            });
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
            let chest_stored_name = crate::chest::stored_name(&self.world_chests, chest_open).to_string();
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
            // `Main.cs:62150-62153`: the fullscreen map's draw head closes
            // the inventory / NPC dialog / sign via `ToggleInv` when any is
            // open — every drawn frame, BEFORE `DrawMap` (`:62155`), so the
            // pylon scan below reads the post-close state like the C#.
            self.close_dialog_behind_fullmap();
            // `Main.PylonSystem.Pylons` (`TeleportPylonsMapLayer.cs:32`): the map
            // layers read the live network. The scan walks the TE book, so it stays
            // off the per-frame path unless the map or the minimap or the style-2
            // overlay (`Main.cs:55625`'s same `MapIcons.Draw`) is drawing.
            let map_layer_pylons = if (self.map.as_ref().is_some_and(|m| m.open)
                || (self.settings.map_enabled && (self.map_style == 1 || self.map_style == 2)))
                && !crate::env::no_hud()
            {
                tr_core::pylons::scan_pylons(&self.world)
            } else {
                Vec::new()
            };
            // The buff bar's inputs (`Main.DrawInterface_Resources_Buffs`,
            // `Main.cs:43203`): the cancel gate is `Main.mouseRight &&
            // Main.mouseRightRelease` (`:43333`). The vanilla latch is
            // recomputed at the END of every interface draw (`:62458-62465`:
            // false while held, true otherwise), so the conjunction fires on
            // the PRESS frame after a released frame and never while held.
            // The port mirrors the latch (`ui_right_release_latch`, updated
            // per draw below — the button state cannot change mid-draw, so
            // the head-of-draw update equals the C#'s tail-of-draw one); the
            // bare release edge the port used fired one frame late, on the
            // RELEASE frame. The vampire-seed gate (`Main.vampireSeed`)
            // drives the duration hides (`:43298`). The removals drain in
            // the tick.
            let _right_release_edge = std::mem::take(&mut self.ui_right_release);
            let buff_right_click = self.buff_cancel_gate();
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
            // The misc-page tick inputs (`Main.cs:41226-41283`, W4-g21 batch 2).
            let hide_misc = self.player.as_ref().map(|p| p.hide_misc).unwrap_or([false; 2]);
            let unlocked_super_cart = self.player.as_ref().is_some_and(|p| p.unlocked_super_cart);
            let enabled_super_cart = self.player.as_ref().is_some_and(|p| p.enabled_super_cart);
            // The hotbar-lock + builder-accessory icon inputs (`Main.cs:42479-42596`,
            // W4-g21 batch 5 hosts 71/72/73).
            let hb_locked = self.player.as_ref().is_some_and(|p| p.hb_locked);
            let unlocked_biome_torches =
                self.player.as_ref().is_some_and(|p| p.unlocked_biome_torches);
            let using_biome_torches =
                self.player.as_ref().is_some_and(|p| p.using_biome_torches());
            let tile_replacement_enabled =
                self.player.as_ref().is_some_and(|p| p.tile_replacement_enabled());
            let info_acc_mech_show_wires =
                self.player.as_ref().is_some_and(|p| p.info_acc_mech_show_wires);
            let ruler_line = self.player.as_ref().is_some_and(|p| p.ruler_line);
            let ruler_grid = self.player.as_ref().is_some_and(|p| p.ruler_grid);
            let auto_actuator = self.player.as_ref().is_some_and(|p| p.auto_actuator);
            let auto_paint = self.player.as_ref().is_some_and(|p| p.auto_paint);
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
            // W4-f6 wiring: the dust spawn ctx for the draw-side emitters
            // (lava bubbles + pylon/void-lens) — snapshotted before the
            // destructure (it reads camera/zoom, which the block borrows).
            let draw_spawn_ctx = self.spawn_ctx();
            // W4-g56 — `Main.DrawProjDirect`'s per-lens
            // `TryInteractingWithVoidLens` call (`Main.cs:27924`): the
            // draw-time interaction pass runs here (the right-press toggle
            // fires inside it, exactly like the C#), the selection modes
            // land on `void_lens_selection` for the projectile draw loop.
            if !self.game_paused && self.menu.is_none() && self.front.is_none() {
                crate::void_lens::update_selection(self);
            } else {
                self.void_lens_selection.clear();
            }
            // The achievement popup's click target (`InGamePopups.cs:138`'s
            // `OpenAchievementsAndGoto`) — parked here because the goto
            // mutates `self` while the draw destructure holds its fields;
            // applied after the gpu block (S1-E-26 wave A).
            let mut popup_goto: Option<&'static str> = None;
            // S1-G-13 consumers: park the render-target family + compositor
            // before the field destructure (the gated draw path borrows them
            // alongside the destructured `&mut` fields; `self.targets` would
            // conflict). Put back after the gpu block.
            let mut targets_parked = self.targets.take();
            let mut compositor_parked = self.target_compositor.take();
            // The fullscreen pylon marker click (`TeleportPylonsMapLayer.cs:
            // 90-109`): resolved during the map draw, drained after the gpu
            // block releases the field borrows (the popup_goto convention).
            let mut pylon_click: Option<crate::map_layers::PylonClick> = None;
            // The craft RAW-list rebuild (`Recipe.UpdateRecipeList`) and the
            // worldgen-debug feed — computed BEFORE the field destructure
            // below: both read many `self` fields disjointly, and a whole-`self`
            // method call inside the destructure block would conflict with the
            // live `&mut` field bindings (E0502). The owned results are used
            // inside the block (W4-f23 closeout).
            let craft_raw_pre = if self.inventory_open
                && self.menu.is_none()
                && self.front.is_none()
                && !crate::env::no_hud()
                && !(self.settings.map_enabled && self.map.as_ref().is_some_and(|m| m.open))
            {
                self.craft_raw_list()
            } else {
                Vec::new()
            };
            let worldgen_debug_pre = self.worldgen_debug_feed();

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
                    cloud_field,
                    sky,
                    bg_alpha_front,
                    bg_alpha_far,
                    forest_hills,
                    bg_flash,
                    ug_back,
                    stars,
                    player_assets,
                    player,
                    npcs,
                    combat_texts,
                    npc_assets,
                    inventory,
                    selected,
                    edit_target,
                    hit_tile,
                    mine_crack_styles,
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
                    gore,
                    liquid_alpha,
                    // W4-g11 union-exec (research/116 gap 4): the
                    // `Main.waterStyle` static write (`DrawWaters` :57475).
                    water_style,
                    wind,
                    tree_wind_counter,
                    grass_wind_counter,
                    sunflower_wind_counter,
                    sunflower_wind_step,
                    vine_wind_counter,
                    items,
                    swing,
                    projectiles,
                    world_key,
                    menu,
                    front,
                    front_hint,
                    issue_indicator,
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
                    let sky_tint = color_of_the_skies(
                        quantised,
                        is_day,
                        world.moon_phase,
                        world.blood_moon,
                    );
                    let clock_time = natural_time;
                    // `SetBackColor`: rain multiplies the sky (and therefore
                    // the surface sunlight) by `1 - cloudAlpha * 0.9 * atmo`
                    // (`Main.cs:63546-63552`, byte-truncated per channel —
                    // `rain_dim_sky`). W4-g69 doc-sweep: the RAW cloudAlpha,
                    // the atmo factor and the remix exclusion replaced the
                    // clamped-visual/no-atmo/0.1-floor stand-in. The atmo
                    // here is the early atmosphere_factor estimate (the C#
                    // reads last frame's Main.atmo at SetBackColor; the
                    // refined self.atmo below re-derives the same value at
                    // the surface).
                    let rain_alpha = rain.raw_alpha();
                    // `Main.UpdateAtmosphereTransparencyToSkyColor`'s input
                    // (`Main.cs:61983`): `screenPosition.Y + screenHeight / 2`
                    // — the view top edge plus half the backbuffer. The port's
                    // `camera` is the view centre spanning `screen / zoom`
                    // world pixels, so the input is
                    // `camera.y - screen.y / (2 * zoom) + screen.y / 2` (the
                    // same formula the `self.atmo` recompute below uses; the
                    // zoom is the world chain's `bg_zoom * zoom`, computed
                    // inline here because `world_zoom` is bound further down).
                    // The atmo feeds SetBackColor's rain dim BEFORE the tile
                    // colour is taken (`Main.cs:61971-61983` order).
                    let atmo_zoom = App::forced_minimum_zoom((screen.x, screen.y)) * *zoom;
                    let atmo = crate::render::sky::atmosphere_factor(
                        camera.y - screen.y / (2.0 * atmo_zoom) + screen.y / 2.0,
                        world.world_surface,
                        world.width,
                    );
                    let mut sky_tint = crate::render::sky::rain_dim_sky(
                        sky_tint,
                        rain_alpha,
                        atmo,
                        world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
                    );
                    // `Main.instance.HorizonRenderer.ModifyHorizonLight`
                    // (`Main.cs:63921`): the sunrise/sunset gradient entries blend
                    // into the sky colour before it becomes the tile light.
                    crate::render::sky::modify_horizon_light(
                        &mut sky_tint,
                        world.moon_phase,
                        is_day,
                        quantised,
                        atmo,
                        rain_alpha,
                    // W4-g70 lane 8: the whole `SetBackColor` chain
                    // (`Main.cs:63452-63924`) in byte space — the blood-moon
                    // night, the eclipse tint, the biome/graveyard dims, the
                    // lightning flash and the remix override all ride it.
                    let rain_alpha = rain.visual_alpha();
                    // `Main.SmoothedMushroomLightInfluence`'s per-draw walk
                    // (`Main.cs:61945-61970`), ahead of SetBackColor.
                    self.mushroom_light_influence = tr_core::sky_color::smooth_mushroom_influence(
                        self.mushroom_light_influence,
                        self.scene_metrics.mushroom_tile_count,
                    );
                    // `Lighting.NotRetro` (`Lighting.cs:63-73`): the port's
                    // internal mode order is Color 0 / White 1 / Retro 2 /
                    // Trippy 3 (`lighting/params.rs`).
                    let not_retro = !matches!(
                        *light_mode,
                        tr_core::lighting::LightMode::Retro | tr_core::lighting::LightMode::Trippy
                    );
                    let back = tr_core::sky_color::set_back_color(
                        &tr_core::sky_color::inputs_from(
                            world,
                            &self.scene_metrics,
                            scene_state.graveyard_visual_intensity,
                            self.mushroom_light_influence,
                            self.eclipse_light,
                            rain_alpha,
                            atmo,
                            rain.flash(),
                            quantised,
                            is_day,
                            not_retro,
                        ),
                    );
                    self.eclipse_light = back.eclipse_light;
                    // `Main.instance.HorizonRenderer.ModifyHorizonLight`
                    // (`Main.cs:63921`, the LAST step of SetBackColor): the
                    // sunrise/sunset gradient entries blend into the sky colour
                    // before it becomes the tile light. The
                    // `ShouldDrawSurfaceBackground` gate
                    // (`NextHorizonRenderer.cs:156`, `Main.cs:59630-59642`) —
                    // false on a remix world in-game (the map-fullscreen case
                    // has no sky pass in the port). NOTE: the gate does NOT
                    // read `BackgroundEnabled` — the tint survives the
                    // backgrounds-off toggle in vanilla.
                    let mut sky_bytes = back.bg;
                    if !world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) {
                        crate::render::sky::modify_horizon_light(
                            &mut sky_bytes,
                            world.moon_phase,
                            is_day,
                            quantised,
                            atmo,
                            rain_alpha,
                            self.mushroom_light_influence,
                            world.eclipse,
                        );
                    }
                    // `Main.ApplyColorOfTheSkiesToTiles` (`Main.cs:61982`) — the
                    // integer 7/10 blend BEFORE the atmo dim (`:61983`).
                    let tile_bytes = tr_core::sky_color::apply_sky_to_tiles(sky_bytes);
                    let sky_light = tile_bytes.map(|c| c as f32 / 255.0);
                    // `UpdateAtmosphereTransparencyToSkyColor`'s colour half
                    // (`Main.cs:63201-63209`): the SKY colour fades with depth
                    // (black at atmo <= 0.01) while the tile light keeps the
                    // undimmed bytes.
                    let sky_dim = tr_core::sky_color::atmo_dim_sky(sky_bytes, atmo);
                    let sky_tint = sky_dim.map(|c| c as f32 / 255.0);
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
                    // `Main.DoUpdate_AnimateCursorColors`' masterColor pulse
                    // (`Main.cs:18225-18235`): 0..1 by 0.05 per frame, the
                    // master-mode text colour. W4-e7 lane 23 — the oscillator
                    // lands (the demon_torch precedent); its readers (rarity
                    // -13) stay unported (research/102). RNG-free.
                    (self.master_color, self.master_color_dir) =
                        bounce_oscillator(self.master_color, self.master_color_dir, 0.05);
                    // `Main.DoUpdate_AnimateCursorColors`' master-colour half
                    // (`Main.cs:18225-18235`): ±0.05 per frame, the direction
                    // flips at the [0, 1] bounds — the −13 master-rarity
                    // tooltip name pulse (W4-f28).
                    (self.master_color, self.master_color_dir) =
                        ui::tooltip_lines2::advance_master_color(
                            self.master_color,
                            self.master_color_dir,
                        );
                    // `Main.DrawInterface_24_InterfaceLogic2`
                    // (`Main.cs:45697-45708`): the invAlpha pulse runs once
                    // per draw-interface pass — the crafting selection
                    // overlay's `Main.inventoryBack` tint.
                    self.inv_pulse.tick();
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
                    // `Main.masterColor`'s ±0.05 cycle (`Main.cs:18225-18235`,
                    // beside the cursor-colour clock above) — the master-rarity
                    // pulse the reforge popup recolours by (`PopupText.cs:667`).
                    self.master_color += self.master_color_dir as f32 * 0.05;
                    if self.master_color > 1.0 {
                        self.master_color = 1.0;
                        self.master_color_dir = -1;
                    }
                    if self.master_color < 0.0 {
                        self.master_color = 0.0;
                        self.master_color_dir = 1;
                    }
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
                    // `Main.DoUpdate_AnimateTileGlows`' martianLight pulse
                    // (`Main.cs:18251-18261`): 0..1 by 0.015 per frame, the
                    // Martian Madness glow. W4-e7 lane 23 — the oscillator
                    // lands; its reader (`TileDrawing.cs:498`'s `_martianGlow`)
                    // stays fed 0 until the glow pass + event land
                    // (research/102). RNG-free.
                    (self.martian_light, self.martian_light_dir) =
                        bounce_oscillator(self.martian_light, self.martian_light_dir, 0.015);
                    // The same fn's martianLight bounce (`Main.cs:18251-
                    // 18261`): ±0.015 per frame, clamped at 1/0 with the
                    // direction flip. Unconditional in C# (`:17145`) —
                    // W4-g12 doc-sweep; feeds `_martianGlow` below.
                    tick_martian_light(&mut self.martian_light, &mut self.martian_light_dir);
                    // `TileLightScanner.Update()`: one `NextSeed()` per frame, then
                    // `WithModifier(x, y)` per tile (done inside the scanner).
                    let mut light_rng = tr_core::tile_light::FastRandom::new(self.light_rng_seed);
                    light_rng.next_seed_in_place();
                    self.light_rng_seed = light_rng.seed;
                    // W4-g78 — `Main.ShouldShowInvisibleBlocksAndWalls`
                    // (`Main.cs:58023-58030`) via the landed scene scan
                    // (`SceneMetrics.EchoMonolith`, tile 657 `frameY >= 54`)
                    // and the player's `CanSeeInvisibleBlocks` (the 4409
                    // Spectre Goggles / vanity-5345 grants). ONE per-frame
                    // value feeds the light options (`LightingEngine.cs:124`),
                    // the tile pass's `IsVisible`/case-541 row
                    // (`TileDrawing.Update` `:379`) and the pylon/trophy
                    // passes (`WallDrawing.Update` `:45`'s wall twin rides
                    // the light params). `TR_SHOW_INVISIBLE=1` keeps pinning
                    // it on for A/B shots. The perspective player is
                    // `Main.LocalPlayer` (`SceneMetrics.AddPlayerEffects`
                    // `:749`) — the sim player, dead or not.
                    let show_invisible = crate::env::show_invisible()
                        || tr_core::scene::should_show_invisible_blocks_and_walls(
                            self.scene_metrics.echo_monolith,
                            player
                                .as_ref()
                                .is_some_and(|p| p.can_see_invisible_blocks),
                        );
                    let params = LightParams {
                        sky: sky_light,
                        // `TileLightScanner.cs:3245`:
                        // `0.55f + (float)Math.Sin(Main.GlobalTimeWrappedHourly * 2f) * 0.08f`
                        // — the sin runs in double, then narrows to f32 (W4-d66:
                        // the driver had pinned the static 0.55 base).
                        hell_glow: 0.55
                            + ((anim_time % 3600.0 * 2.0) as f64).sin() as f32 * 0.08,
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
                        // `Main.shimmerDarken` (SceneState.UpdateShimmer ramps).
                        shimmer_darken: scene_state.shimmer_darken,
                        // `Main.liquidAlpha[13]` (TileLightScanner.cs:3160-3166):
                        // the shimmer blue boost. NOTE: in the decompile NO writer
                        // ever raises liquidAlpha[13] (the DrawWaters ramp skips
                        // style 14, the capture loop only writes 0..=10) — the
                        // C# arm is dead code; the port keeps the 0 feed (the
                        // census deviation, ANALYSIS W4-e19 aether).
                        shimmer_alpha: 0.0,
                        // `Main.sundialCooldown`/`Main.moondialCooldown`
                        // (`Main.cs:2161`/`:2165`): the scanner and the glow pass
                        // mute the dial light while one ticks (0..=8, u8 on World).
                        sundial_cooldown: i32::from(world.sundial_cooldown),
                        moondial_cooldown: i32::from(world.moondial_cooldown),
                        // `Main.jellyfishCageMode` (`Main.cs:66948`): the wired
                        // critter-cage animator's mode bytes (W4-g34 union). The
                        // scanner's jellyfish arm (`TileLightScanner.cs:714-768`)
                        // brightens the cage while mode == 2; feeding the live
                        // table keeps that pulse in step with the draw frames.
                        cage_mode: self.cage_anim.jellyfish_mode,
                        rng_seed: self.light_rng_seed,
                        // `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023`):
                        // W4-g78 wired — the EchoMonolith scan + the 4409/5345
                        // grants above; `TR_SHOW_INVISIBLE=1` still pins it on.
                        draw_invisible_walls: show_invisible,
                    };
                    // S1-G-13 consumers: park the frame's ambient light
                    // parameters for the capture chunk render
                    // (`Main.DrawCapture`'s `Lighting.LightTiles`,
                    // `Main.cs:56727-56732` — the capture tick runs at the
                    // next frame's head, before this region recomputes them).
                    self.capture_light_params = Some(params);
                    // `Main.essScale`'s drift (`Main.cs:62264-62277`), which
                    // the C# runs once per draw pass just before `DrawItems`
                    // (`:62278`), behind `!Main.gamePaused`. The step-time
                    // flicker arms above read the value the last draw left.
                    if !self.game_paused {
                        crate::item_lights::update_ess_scale(
                            &mut self.ess_scale,
                            &mut self.ess_dir,
                        );
                    }
                    // `WorldItem.Update`'s `UpdateItem_VisualEffects` lights (S1-E-5)
                    // (`WorldItem.cs:1295-1648`). The C# runs them once per live
                    // drop per tick, in `Main.item` slot order, at the end of
                    // `WorldItem.Update` (`:436`); `items` here is that same slot
                    // order after the step. No arm in THIS feed draws `Main.rand`
                    // (the rand/essScale head `:1206-1293` feeds from the fixed
                    // step instead, `item_lights.rs`), so the placement relative
                    // to the sim leaves the stream untouched.
                    for drop in items.iter() {
                        // `WorldItem.UpdateShimmer`'s shimmered glow
                        // (`WorldItem.cs:482`, W4-g20) — RNG-free (the
                        // `Next(30)` dust branch `:459-481` stays skipped
                        // wholesale), fed ahead of the visual-effects light
                        // like the C#'s `:403` before `:436` call order.
                        if let Some((x, y, rgb)) = crate::entity_lights::shimmer_light(
                            drop.shimmered,
                            drop.shimmer_time,
                            drop.pos,
                            crate::item::SIZE,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
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
                    // W4-c20: the probe light's `FindClosestPlayer` gate
                    // (`NPC.cs:36572-36576`) — no live player reads as
                    // -1/dead in the C#'s single-player slot.
                    let player_dead = player.as_ref().map_or(true, |p| p.dead);
                    for n in npcs.iter() {
                        if n.hp <= 0 {
                            continue;
                        }
                        let (w, h) = n.size();
                        let pos = Vec2::new(n.x, n.y) + n.net_offset;
                        if let Some((x, y, rgb)) = crate::npc_lights::npc_light(
                            n.kind,
                            pos,
                            w,
                            h,
                            n.ai[0],
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                        // W4-f47 keep-one 144 — the ShimmerSlime 676's
                        // `AddLight(base.Center, 23)` (`NPC.cs:61130`):
                        // the shimmer-torch pulse, RNG-free (the
                        // npc_lights seam; the dust half rides the sim's
                        // slot-turn drain).
                        if let Some((x, y, rgb)) = crate::npc_lights::shimmer_slime_light(
                            n.kind,
                            pos,
                            w,
                            h,
                            &params,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                        // W4-c20: the FIRST-half NPC AddLight sites
                        // (`NPC.cs:21307-:48641`, `npc_lights_a.rs`).
                        // The kind sets are disjoint from the aiStyle-8
                        // tail above, so no site double-feeds. The
                        // BurningSphere's else arm sits inside the C#'s
                        // 2-iteration dust loop (`:21517`) — the count
                        // is part of the port (the list pair's key
                        // hashes every entry). No arm draws RNG.
                        if let Some((x, y, rgb)) =
                            crate::npc_lights_a::npc_light_a(n, world, player_dead)
                        {
                            for _ in 0..crate::npc_lights_a::npc_light_a_emits(n.kind) {
                                self.entity_lights.add(x, y, rgb);
                            }
                        }
                        // The NPC lights SECOND half (W4-e7, S1-E-5):
                        // `NPC.cs:49929-92412`'s 40 sites — the destroyer
                        // airborne / probe / town / Medusa / torch-zombie /
                        // fighter-tail / slime AI arms, the buff-VFX fire
                        // lights, the trail mode-4 duplicates and the
                        // cast-light arms (`npc_lights_b.rs`). Same slot
                        // order, same one-list-pair feed; no arm draws
                        // `Main.rand` (the rand halves are cited skips),
                        // so the placement leaves the streams untouched.
                        // The Horseman `:62420` is fed below (W-10) — the
                        // module records it as a census-only no-double-feed.
                        for (x, y, rgb) in crate::npc_lights_b::npc_lights_b(n, world, &params) {
                            self.entity_lights.add(x, y, rgb);
                        }
                        // S1-D-4 W-10: the Horseman's unconditional
                        // `Lighting.AddLight(Center, 0.4, 0.36, 0.2)`
                        // (`NPC.cs:62420`) — slice B's pure fn, no RNG
                        // (the npc_lights precedent above).
                        if let Some((x, y, rgb)) = crate::npc_ai_unicorn_b::horseman_light(
                            n.kind,
                            Vec2::new(n.x, n.y) + n.net_offset,
                            w,
                            h,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                        // W4-f8 (S1-E-5 third NPC batch): the DD2 body
                        // casts — `Utils.PlotTileLine(Top, Bottom, width,
                        // CastLightOpen)` for the etherian skeletons
                        // 566/567 (`NPC.cs:63542-63543`, AI_107's case
                        // head) and the Dark Mages 564/565 (`:65828-
                        // 65829`, AI_109's per-kind block). RAW position
                        // (neither arm wraps netOffset), RNG-free — the
                        // same draw-side feed precedent as above.
                        for (x, y, rgb) in crate::npc_lights_c::dd2_body_cast_light(n, world) {
                            self.entity_lights.add(x, y, rgb);
                        }
                        // The small-kinds wiring wave (S1-D-3-wire-smallkinds):
                        // the BlazingWheel's centre light (`NPC.cs:24664` —
                        // the green term reads the pulsing `Main.mouseTextColor`)
                        // and the ForceBubble's constant (0.1, 0.5, 0.7) at the
                        // truncated centre tile (`NPC.cs:35092`). Both are
                        // RNG-free; the bubble's is fed only while attached
                        // (the pop tick zeroes hp, the `hp <= 0` gate above).
                        match n.kind {
                            NpcKind::BlazingWheel => {
                                let (x, y, rgb) = crate::npc_ai_style21::blazing_wheel_light(
                                    pos.x,
                                    pos.y,
                                    (w, h),
                                    self.mouse_text_color,
                                );
                                self.entity_lights.add(x, y, rgb);
                            }
                            NpcKind::ForceBubble => {
                                let x = crate::entity_lights::tile_x(
                                    pos,
                                    w,
                                    crate::entity_lights::XAnchor::CenterTruncated,
                                );
                                let y = crate::entity_lights::tile_y(
                                    pos,
                                    h,
                                    crate::entity_lights::XAnchor::CenterTruncated,
                                );
                                self.entity_lights.add(x, y, [0.1, 0.5, 0.7]);
                            }
                            // W4-c9 — the snail family's aiStyle-67
                            // lights (`NPC.cs:34387`/`:34391`): the
                            // 360 blue / 655 orange at the truncated
                            // centre tile. RNG-free (the npc_lights
                            // convention).
                            NpcKind::GlowingSnail | NpcKind::MagmaSnail => {
                                if let Some((x, y, rgb)) = crate::npc_lights::snail_light(
                                    n.kind,
                                    pos,
                                    w,
                                    h,
                                ) {
                                    self.entity_lights.add(x, y, rgb);
                                }
                            }
                            // W4-g8 lane 1 batch 1 — the Pixie's hallow glow
                            // (`NPC.cs:24801`, the aiStyle-22 arm's 75 branch):
                            // (0.9, 0.3+(270-mouseTextColor)/400, 0.2) at the
                            // truncated centre — the BlazingWheel's pulsing
                            // green-term shape. RNG-free.
                            NpcKind::Pixie => {
                                let x = crate::entity_lights::tile_x(
                                    pos,
                                    w,
                                    crate::entity_lights::XAnchor::CenterTruncated,
                                );
                                let y = crate::entity_lights::tile_y(
                                    pos,
                                    h,
                                    crate::entity_lights::XAnchor::CenterTruncated,
                                );
                                let g = 0.3 + (270 - self.mouse_text_color as i32) as f32 / 400.0;
                                self.entity_lights.add(x, y, [0.9, g, 0.2]);
                            }
                            _ => {}
                        }
                        // W4-g8 lane 1 batch 1 — the AI machine's per-tick
                        // light (the jelly's shield-scaled glow, the ghost
                        // family's 122/169 glows): the machine wrote it on
                        // `ai_light`; feed it here (the npc_lights precedent).
                        if let Some((x, y, rgb)) = n.ai_light {
                            self.entity_lights.add(x, y, rgb);

        }                            // W4-c11: the Tesla Turret's blue light — the
                            // ai[1]/60-scaled fade (`NPC.cs:35167`) then
                            // the constant (0.1, 0.5, 0.7) (`:35175`), at
                            // the truncated centre tile 10 px up. RNG-free.
                            NpcKind::MartianTurret => {
                                let (x, y, rgb) = crate::npc_ai_style73::turret_light(
                                    n.ai[1],
                                    (pos.x + w / 2.0, pos.y + h / 2.0),
                                self.entity_lights.add(x, y, rgb);
                        // W4-d31 style83 wiring — the cultist tablet's
                        // constant light (`NPC.cs:36921-36924`:
                        // `AddLight(Center, 0.8, 0.75, 0.55)`). Keyed on the
                        // vanilla `cs_id` (437), NOT a kind variant — the
                        // 437 kind rides the roster wave (W4-d40 `7fec14aa`),
                        // so this arm is dormant until it lands (the
                        // kind-agnostic gate keeps the wiring compilable
                        // pre-merge, the style83 axis-arm precedent). The
                        // `Lighting.AddLight(Vector2, ...)` overload
                        // truncates the quotient per axis (`Lighting.cs:
                        // 140-143` — `(int)(v/16f)`, the `XAnchor::Center`
                        // form). RNG-free; fed only while the tablet lives
                        // (the `hp <= 0` gate above — the C#'s early-return
                        // paths skip the tail too).
                        if n.kind.cs_id() == 437 {
                            let x = crate::entity_lights::tile_x(
                                pos,
                                w,
                                crate::entity_lights::XAnchor::Center,
                            );
                            let y = crate::entity_lights::tile_y(
                                h,
                            self.entity_lights.add(x, y, [0.8, 0.75, 0.55]);
                        }
                    }
                    // `Player.UpdateBuffs`' Shine arm (`Player.cs:9993-9996`,
                    // S1-E-7 wave A): the 0.8/0.95/1.0 light at the player's
                    // center tile while buff 11 is held. No RNG — the feed
                    // leaves the streams untouched. This stays the LIVE
                    // Shine feed: `player_lights.rs` below deliberately
                    // skips buff 11 so the family never double-feeds it.
                    // (Feed order follows `Main.Update`'s family order:
                    // Players, NPCs, Projectiles, Items, Dust —
                    // `Main.cs:17754-17783`.)
                    if let Some(p) = player.as_ref() {
                        if let Some((x, y, rgb)) = crate::entity_lights::shine_light(p) {
                            self.entity_lights.add(x, y, rgb);
                        }
                    }
                    // The Player `AddLight` family (S1-E-5,
                    // `player_lights.rs`): the `UpdateBuffs` buff arms
                    // (`Player.cs:10215`/`:10316`/`:10466` — Shine `:9995`
                    // is fed above by S1-E-7a), the `UpdateEquips`
                    // accessory walk (`:15224-15262`) and
                    // `ItemCheck_EmitHeldItemLight` (`:49810-50194`) —
                    // the C# runs them in that order inside
                    // `Player.Update` (`:25566`/`:25633`/`:31827`); the
                    // port collects them here after the step, like the
                    // item/NPC feeds above, feeding the SAME
                    // `EntityLights` pair. No ported arm draws
                    // `Main.rand` — every rand-bearing dust branch is
                    // skipped wholesale — so the placement leaves the
                    // UpdatePlayers stream untouched. The dead player
                    // is skipped like the dead NPCs (the port's death
                    // path clears the buffs; vanilla's residual light
                    // while dead needs the unported death visuals).
                    if let Some(p) = player.as_ref().filter(|p| !p.dead) {
                        let held_kind = inventory
                            .get(*selected)
                            .filter(|s| s.count > 0)
                            .map_or(0, |s| s.kind);
                        // The rest anchor's `heldItemFrame.Width` — the
                        // same icon width the draw layer reads
                        // (`render/entities.rs::held_item_quad`).
                        let held_frame_w = lookups
                            .item_icon(held_kind)
                            .map_or(0.0, |(_, w, _)| w);
                        for (x, y, rgb) in crate::player_lights::player_lights(
                            p,
                            held_kind,
                            held_frame_w,
                            p.zones,
                            world,
                            &params,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                        // The Player GAP batch (S1-E-5 W4-e7 census close,
                        // `player_lights_b.rs`): the 10 dropped sites whose
                        // mount/armor/wing/melee/phase-color dependencies have
                        // since landed — Rudolph's trail, the three armor-set
                        // lights, the Chillet dash + melee-swing lights, the
                        // Kazzymodus wing and the phase-blade held light. Same
                        // ONE `EntityLights` pair, no RNG (every rand-bearing
                        // dust branch skipped wholesale). The held frame's
                        // height rides the swing lights' hitbox.
                        let held_frame_h = lookups
                            .item_icon(held_kind)
                            .map_or(0.0, |(_, _, h)| h);
                        for (x, y, rgb) in crate::player_lights_b::player_lights_b(
                            p,
                            held_kind,
                            held_frame_w,
                            held_frame_h,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                    }
                    // `Projectile.ProjLight` (S1-E-5, `Projectile.cs:
                    // 16153-16661`): `Update` calls it once per movement
                    // sub-step (`:17031`), after `HandleMovement` and the
                    // active re-check (`:17027-17030`) — the port collects
                    // the same sources here after the step, in
                    // `Main.projectile` slot order, reading each slot's
                    // current `light` amount (the aiStyle-24 arm rewrites
                    // its shard's per tick, `:25953`). No ported arm draws
                    // `Main.rand` (the rand-bearing 867/1000 arms have no
                    // rows, `proj_lights.rs`), so the placement relative to
                    // the sim leaves the stream untouched.
                    let good_world = world
                        .seeds
                        .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
                    for p in projectiles.slots().iter().filter(|p| p.active) {
                        if let Some((x, y, rgb)) = crate::proj_lights::proj_light(
                            p.kind, p.light, p.pos, p.w, p.h, good_world, &params,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                        // W4-g5 — the 1092 book's per-tick `Lighting.AddLight
                        // (base.Center, 0.3f, 0.25f, 0.1f)` (`Projectile.cs:
                        // 25394`): a constant AI-head light, NOT the
                        // SetDefaults `light` amount (the 1092 row carries
                        // none). The Centre anchor truncates per axis.
                        if p.kind == crate::projectile::LIBRARIAN_BOOK {
                            let cx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
                            let cy = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
                            self.entity_lights.add(cx, cy, [0.3, 0.25, 0.1]);
                        // W4-g56 — `AI_154_VoidLens`'s `VoidLensHelper.Update`
                        // (`Projectile.cs:52195` → `VoidLensHelper.cs:34-38`):
                        // `AddLight(Center, 0.4, 0.2, 0.9)` — the Vector2
                        // overload tiles by `(int)(v / 16f)`
                        // (`Lighting.cs:140-143`). The lens row's `light` is
                        // 0, so the ProjLight chain above contributes
                        // nothing; this is the AI-side source, collected
                        // here like the tile lens's (one frame late, the
                        // EntityLights convention). The dust half stays
                        // skipped (the S1-G-1 wave-D RNG-order rule).
                        if p.kind == crate::projectile::VOID_LENS && !self.game_paused {
                            let cx = p.pos.x + p.w * 0.5;
                            let cy = p.pos.y + p.h * 0.5;
                            self.entity_lights.add(
                                (cx / TILE_PX) as i32,
                                (cy / TILE_PX) as i32,
                                [0.4, 0.2, 0.9],
                            );
                        }
                    }
                    // The projectile AI-arm lights, second wave (S1-E-5,
                    // W4-e6, `proj_ai_lights.rs`): the `AddLight` sites
                    // OUTSIDE `ProjLight` — `AI_001`'s per-type arms
                    // (`Projectile.cs:54173`…`:55171`), `AI_026`'s foxsparks
                    // pair (`:58213`/`:58226`), `AI_062`'s StardustCell
                    // (`:65120`), `AI_075`'s LaserDrill (`:66272`),
                    // `AI_130`'s FlameBurst tower pairs (`:68124-68225`) and
                    // `AI_138`'s ExplosiveTrap tail (`:69332`). Like the
                    // `ProjLight` feed above, the port collects the sources
                    // here after the step in `Main.projectile` slot order;
                    // an arm whose kind has no `proj_data` row is absent
                    // wholesale. No ported arm draws `Main.rand`, so the
                    // placement leaves the UpdateProjectiles stream
                    // untouched.
                    for p in projectiles.slots().iter().filter(|p| p.active) {
                        // The 445 LaserDrill's distance gate reads the
                        // owner's `RotatedRelativePoint(MountedCenter)`
                        // (`AI_075`'s `vector`, `Projectile.cs:66135`).
                        // The port is single-player: `owner == 0` resolves
                        // the local player (the `map_layers.rs` /
                        // `ai_drill` precedent); every other owner has no
                        // player slot, so the hand stays `None` and the
                        // arm skips (the C# reads `Main.player[owner]`
                        // unconditionally — the no-owner kill precedent).
                        let hand = player.as_ref().filter(|_| p.owner == 0).map(|pl| {
                            let (mx, my) = pl.mounted_center();
                            pl.rotated_relative_point(Vec2::new(mx, my), 0)
                        });
                        for (x, y, rgb) in crate::proj_ai_lights::proj_ai_lights(
                            p.kind,
                            p.pos,
                            p.w,
                            p.h,
                            p.alpha,
                            p.local_ai[0],
                            hand,
                        ) {
                            self.entity_lights.add(x, y, rgb);
                        }
                    }
                    // The projectile AI-arm lights, FIRST wave (S1-E-5,
                    // W4-c19, `proj_lights_a.rs`): sites 1-67 of the 107
                    // (`Projectile.cs:16655`…`:54071`) — sites 1-2 are the
                    // `ProjLight` tail above (audit-landed), the rest are
                    // the AI-arm lights outside it. Like the `ProjLight`
                    // feed above and the e6 second wave, the port collects
                    // the sources here after the step in `Main.projectile`
                    // slot order; an arm whose kind has no `proj_data` row
                    // is absent wholesale, and a rand-gated light is absent
                    // wholesale (the e6 RNG rule). No ported arm draws
                    // `Main.rand`, so the placement leaves the
                    // UpdateProjectiles stream untouched.
                    for p in projectiles.slots().iter().filter(|p| p.active) {
                        // The hooks' anchor is the owner's `MountedCenter`
                        // (`AI_007`, `Projectile.cs:52572`). Single-player
                        // fold: `owner == 0` resolves the local player (the
                        // e6 `ai_drill` hand precedent); every other owner
                        // has no player slot, so the anchor stays `None`
                        // and the arm skips.
                        let mounted_center =
                            player.as_ref().filter(|_| p.owner == 0).map(|pl| {
                                let (mx, my) = pl.mounted_center();
                                Vec2::new(mx, my)
                            });
                        // Site 3 (`:17343`) — the Chillet-mount enchantment
                        // light runs in `Update` BEFORE the AI arms
                        // (`:16702`), so it feeds first. The owner's active
                        // mount type resolves for `owner == 0` (the same
                        // single-player fold); `None` otherwise.
                        let owner_mount = player
                            .as_ref()
                            .filter(|_| p.owner == 0)
                            .filter(|pl| pl.mount.is_active())
                            .map(|pl| pl.mount.kind());
                        for (x, y, rgb) in
                            crate::proj_lights_a::chillet_mount_light(p, owner_mount)
                        {
                            self.entity_lights.add(x, y, rgb);
                        }
                        for (x, y, rgb) in
                            crate::proj_lights_a::proj_lights_a(p, mounted_center, self.disco)
                        {
                            self.entity_lights.add(x, y, rgb);
                        // W4-c39 (the surviving graft, research/114 §1c) —
                        // `AI_166_Dove`'s lantern light cast
                        // (`Projectile.cs:47568-47569`): `(1, 0.6, 0.1) *
                        // (1 - alpha/255)` at the lantern's centre. The AI
                        // writes it per tick (not the SetDefaults `light`
                        // slice), so it rides the feed directly. d17's
                        // in-file `ai_dove` machine skips the light (no pool
                        // layer); this frame-side seam is c39's unique
                        // contribution, grafted beside the canonical machine.
                        if p.kind == crate::projectile::RELEASE_LANTERN {
                            let num2 = 1.0 - p.alpha as f32 / 255.0;
                            let cx = (p.pos.x + p.w * 0.5) / TILE_PX;
                            let cy = (p.pos.y + p.h * 0.5) / TILE_PX;
                            self.entity_lights.add(
                                cx as i32,
                                cy as i32,
                                [num2, 0.6 * num2, 0.1 * num2],
                            );
                        }
                    }
                    // The Dust `UpdateDust` lights (S1-E-5, dust family):
                    // the RNG-free AddLight arms of the FIRST ladder
                    // (`Dust.cs:483-1047`, `dust_lights.rs`) run per active
                    // dust per update in `Main.dust` slot order; the port
                    // collects the same sources here after the dust pass,
                    // in the same slot order. No arm draws `Main.rand` and
                    // none mutates the pool (the witness test in
                    // `dust_lights.rs` pins the stream), so the placement
                    // relative to the sim leaves the fixed-step order exact.
                    crate::dust_lights::feed_dust_lights(dust, &params, &mut self.entity_lights);
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
                    // The Dust `UpdateDust` SECOND-chain lights (S1-E-5,
                    // W4-g13 — the research/202 flag-a orphan closure): the 52
                    // RNG-free AddLight arms `Dust.cs:1061-2312`
                    // (`dust_lights_b.rs`), fed immediately after the FIRST
                    // ladder in the same `Main.dust` slot order, reading the
                    // POST-move pool (d50 owns the move). The 244-247/43
                    // ambient sample is `Lighting.GetColor` (the light map ×
                    // GlobalBrightness, `main_dust::light_color_bytes`); the
                    // 27/45 player push and the 35 wet probe ride the same
                    // seams d50's `DustPhysicsEnv` uses. No arm draws
                    // `Main.rand` (the witness in `dust_lights_b.rs` pins the
                    // stream), so the placement leaves the fixed-step order
                    // exact. Placed after the brightness block so the ambient
                    // sample reads the Darkness-adjusted GlobalBrightness.
                    {
                        let (pvx, pvy, pwet) = player
                            .as_ref()
                            .map_or((0.0, 0.0, false), |p| (p.vx, p.vy, p.wet));
                        let brightness = style.brightness;
                        let lc = |x: i32, y: i32| {
                            crate::main_dust::light_color_bytes(light, brightness, x, y)
                        };
                        let wet = |pos: Vec2, w: f32, h: f32| {
                            crate::player::wet_collision(world, pos.x, pos.y, w, h)
                        };
                        let chain_env = crate::dust_lights_b::ChainLightEnv {
                            player_wet: pwet,
                            player_velocity: Vec2::new(pvx, pvy),
                            light_color: &lc,
                            wet_collision: &wet,
                        };
                        crate::dust_lights_b::feed_dust_chain_lights(
                            dust,
                            &params,
                            &chain_env,
                            &mut self.entity_lights,
                        );
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
                    // outside the ramp. The style carries the fountain head arm
                    // (`CalculateWaterStyle`, `:57390-57393`) like `Main.waterStyle`.
                    let water_style_now = crate::biome::calculate_water_style_with_fountain(
                        world,
                        draw_cam,
                        self.scene_metrics.active_fountain_color,
                    );
                    // `Main.waterStyle = Main.CalculateWaterStyle()`
                    // (`Main.cs:57475`): the STORED static the
                    // `TeleportEffect` style-5 arm reads (research/116
                    // gap 4) — written here at the C# position.
                    *water_style = water_style_now as i32;
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
                    // W4-g68: the leaf spawns go straight into `App.gore`
                    // (`Main.gore`) during the tile/foliage passes — no
                    // intermediate Vec. The `Main.rand` stand-in for the
                    // draw-side `NewGore*` internals is the bare global
                    // stream (the C# `DrawTiles_EmitParticles` runs outside
                    // every SwapRandom block; the per-frame pinning of the
                    // gate stream stays below in `draw_tiles`).
                    if !crate::env::no_waterfall() {
                        // `Main.screenPosition` / `screenWidth` / `screenHeight`
                        // for both ambience scans and the rain-arm view cull
                        // (`WaterfallManager.cs:381`/`:687-691`/`:710-714`): the
                        // unscaled world-pixel view, top-left = draw_cam -
                        // screen/2 (the S1-E-19 `Camera.UnscaledPosition`
                        // convention).
                        let screen_info = tr_core::waterfall::ScreenInfo {
                            position: [draw_cam.x - screen.x * 0.5, draw_cam.y - screen.y * 0.5],
                            width: screen.x,
                            height: screen.y,
                        };
                        // `Main.drewLava` at the `DoDraw_Waterfalls` site
                        // (`Main.cs:63281`): the port's liquid passes do not
                        // port `oldDrawWater`'s true-store (`:57938`), so the
                        // flag rides false; `WaterfallManager.Draw` clears it
                        // at `:1125` either way.
                        // `WaterfallManager.Draw`'s head (`WaterfallManager.cs:1120`):
                        // the `_shouldShowInvisibleBlocksAndWalls` latch GetAlpha's
                        // default arm reads (`:1037`) — the same value the light
                        // params carry (`Main.cs:58023-58030`).
                        waterfall.set_show_invisible(params.draw_invisible_walls);
                        // `Main.DiscoR/G/B` (`Main.cs:959-963`): the disco fall
                        // (tile 160 → type 2) reads the SHARED statics the
                        // lighting scanner reads (`StylizeColor` case 2,
                        // `WaterfallManager.cs:1010-1012`; `AddLight` case 2,
                        // `:1084`). The cycle is `DoUpdate_AnimateDiscoRGB`
                        // (`Main.cs:19591-19648`), stepped by `advance_disco`;
                        // latch the same values the `LightParams` carry.
                        // (W4-g75 final sweep `d118f7ba`, folded.)
                        waterfall.set_disco_rgb(params.disco);
                        let (lights, waterfall_scan, lavafall_scan) = waterfall.draw(
                            world,
                            light,
                            &style,
                            anim_time,
                            liquid_alpha,
                            params.mouse_text_color,
                            (*frame_count as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ world.seed,
                            screen_info,
                            false,
                        );
                        scratch.waterfall_quads = waterfall.take_quads();
                        // `Main.ambientWaterfallX/Y/Strength` +
                        // `Main.ambientLavafallX/Y/Strength`
                        // (`WaterfallManager.cs:685-729` scans, `:960-965`
                        // stores): the near-screen strength sums and the
                        // nearest segments' cell-centre world pixels — the
                        // waterfall/lavafall loops' volume/pan inputs
                        // (`Main.Ambience`, `Main.cs:13475-13508`). Honey
                        // falls feed neither triple (`:708`'s kind-14 skip).
                        self.ambient_waterfall = (
                            waterfall_scan.strength,
                            waterfall_scan.x,
                            waterfall_scan.y,
                        );
                        self.ambient_lavafall = (
                            lavafall_scan.strength,
                            lavafall_scan.x,
                            lavafall_scan.y,
                        );
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
                            crate::biome::calculate_water_style_with_fountain(
                                world,
                                *camera,
                                self.scene_metrics.active_fountain_color,
                            );
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
                            // `Main.wofNPCIndex >= 0 &&
                            // Main.player[Main.myPlayer].gross`
                            // (`LegacyLighting.cs:678`, S1-E-19d): the
                            // Horrified + living-WoF gate carries the indexed
                            // WoF's position/direction into the gross light
                            // floor. An out-of-range index is the C# try's
                            // abort before any write (`:684`) — `None` here.
                            wof_gross: if world.wof_npc_index >= 0
                                && player.as_ref().is_some_and(|p| p.gross)
                            {
                                npcs.get(world.wof_npc_index as usize).map(|n| {
                                    tr_core::lighting::legacy::WofGrossLight {
                                        position_x: n.x,
                                        direction: n.facing,
                                    }
                                })
                            } else {
                                None
                            },
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
                        let consumed = export.map(|e| (e.changed, e.update_map));
                        if let Some((changed, update_map)) = consumed {
                            if let Some(m) = map.as_mut() {
                                // The legacy export's changed tiles upload
                                // directly (`LegacyLighting.cs:429` calls
                                // `UpdateLighting`, NOT `QueueChange` — only
                                // `MapUpdateQueue` queues, `:82`/`:97`); the
                                // IsChanged bit rides the next DrawToMap walk.
                                m.push_dirty(changed);
                                // `LegacyLighting.cs:432`: `Main.updateMap =
                                // rect` — the consume arm (`Main.cs:61886-61894`)
                                // reads it next frame.
                                if let Some(rect) = update_map {
                                    m.update_map = Some(rect);
                                }
                            }
                        }
                        engine.present_into(light);
                        // `Main.DrawLiquid`'s retro branch (`Main.cs:57519-
                        // 57522`): `!Lighting.NotRetro` (Retro/Trippy,
                        // `Lighting.cs:63-73`) routes the liquid draw to
                        // `oldDrawWater`, whose lava arm writes the
                        // `Main.ambientLava*` triple (`:57615-57634` scan,
                        // `:57932-57936` store). The port keeps the
                        // LiquidRenderer draw in every mode but ports the
                        // scan (W4-g73 closeout): the foreground
                        // `DrawWaters` (`:62300`) resets `drewLava` false
                        // (`:57472`) and its FIRST `DrawLiquid` pass stores
                        // the triple (later passes are gated by the
                        // `:57938` true-store); the background call's
                        // (`:62192`) store is overwritten by it — so one
                        // `bg = false, drew_lava = false` scan IS the final
                        // Main state. Non-retro modes never write (the
                        // triple keeps its last value, exactly like the
                        // original — the load reset until a retro frame).
                        if matches!(
                            *light_mode,
                            tr_core::lighting::LightMode::Retro
                                | tr_core::lighting::LightMode::Trippy
                        ) {
                            let scan = tr_core::retro_water::ambient_lava_scan(
                                world,
                                light,
                                tr_core::waterfall::ScreenInfo {
                                    position: [
                                        draw_cam.x - screen.x * 0.5,
                                        draw_cam.y - screen.y * 0.5,
                                    ],
                                    width: screen.x,
                                    height: screen.y,
                                },
                                false,
                                false,
                            );
                            self.ambient_lava = (scan.strength, scan.x, scan.y);
                        }
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
                            // `WindGrid.cs:91`: the `!player.active || player.dead`
                            // head gates — a dead player's corpse drift must not
                            // stamp gusts (W4-g68 final-phase census fix; the
                            // active half is the `Some(p)` itself).
                            if let Some(p) = player.as_ref().filter(|p| !p.dead) {
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

                    // `LiquidRenderer.Update` (`LiquidRenderer.cs:831-847`,
                    // called once per client update at `Main.cs:17043`): the
                    // liquid texture clock is WIND-driven — `wind*25 ± 6`
                    // frames/s for the body/edge rows, a wind-independent
                    // 0.5 frames/s for the interior waterfall rows. Both
                    // integer frames ride the G6 key (packed) only when the
                    // world has any liquid; a calm world advances at 6 fps
                    // (the old wind-independent `anim_time*8` approximation
                    // is retired — ANALYSIS "the liquid render-gaps census").
                    let (liquid_anim_frame, liquid_waterfall_frame) =
                        self.liquid_clock.update(dt, wind.speed_current);
                    // The body frame can be NEGATIVE (a west wind below
                    // -0.24 — the C# single +16 wrap keeps the remainder's
                    // sign); the low byte of the two's-complement cast keeps
                    // -15..15 injective, the waterfall frame rides byte 1.
                    let liquid_anim = if *has_liquids {
                        (liquid_anim_frame as u32 & 0xFF)
                            | ((liquid_waterfall_frame as u32 & 0xFF) << 8)
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
                            // W4-f41: the EFFECTIVE head kind rides the G6
                            // key — equipping/unequipping the vanity Party
                            // Hat (or a real helmet) changes the drawn head
                            // with no player/world motion (the brightness
                            // key's S1-H-15 convention). Folded into
                            // `player_key`'s slot, gated on any head being
                            // worn so bare-headed frames keep the pre-wiring
                            // cache behaviour bit-for-bit.
                            player_key: {
                                let kind_of = |i: usize| {
                                    self.equip
                                        .armor
                                        .get(i)
                                        .filter(|s| s.count > 0)
                                        .map_or(0, |s| s.kind)
                                };
                                let real = kind_of(0);
                                let vanity = kind_of(10);
                                let effective = if crate::registry::get(vanity)
                                    .is_some_and(|d| d.head_slot >= 0)
                                {
                                    vanity
                                } else {
                                    real
                                };
                                if effective != 0 {
                                    player_key ^ (effective as u32).wrapping_mul(0x9E37_79B1)
                                } else {
                                    player_key
                                }
                            },
                        npc_key,
                        items_key: item::render_key(items, ((anim_time * 8.0) as u32) & 15, anim_time),
                        hand_key: {
                            let slot = inventory.get(*selected).filter(|s| s.count > 0);
                            let held = slot.map_or(0, |s| s.kind as u32);
                            let phase = if *swing > 0.0 {
                                (((1.0 - *swing / self.swing_max.max(f32::EPSILON)) * 40.0) as u32)
                                    .min(40)
                            } else {
                                0
                            };
                            // S1-P-i05: the prefix rides the key — the
                            // thrust's `GetAdjustedItemScale` reads the
                            // PREFIXED scale (`Player.cs:42566`), so a
                            // reforge changes the held item's pixels.
                            let prefix = slot.map_or(0, |s| s.prefix as u32);
                            held | (phase << 16) | (prefix << 24)
                        },
                        light_generation: light.generation(),
                        // S1-H-15: `shade` multiplies every tile tint by the style's
                        // brightness, and the Darkness debuff's `blind` flag drops it
                        // 1.2 → 1.0 (`Lighting.cs:108-116`) — the input must ride the
                        // key or a blind gain/loss keeps the previous frame's tints.
                        brightness: style.brightness.to_bits(),
                        waterfall: waterfall.anim_key(),
                        // S1-G-1f: the wired tile animator feeds
                        // `Main.tileFrame`/`tileFrameCounter` rows into the
                        // tile draw (`draw_tiles` → `draw_data` /
                        // `case_draw_data`) — clock-dependent pixels inside
                        // the cached range, so the state must ride the key.
                        // Gated on an animated kind being active in the
                        // view rect (the vine-grid key's convention): an
                        // animator-free view keeps the pre-wiring cache
                        // behaviour bit-for-bit.
                        tile_anim: {
                            let vh = screen * 0.5 / *zoom;
                            let vx0 = ((camera.x - vh.x) / TILE_PX).floor() as i32 - 1;
                            let vx1 = ((camera.x + vh.x) / TILE_PX).ceil() as i32 + 1;
                            let vy0 = ((camera.y - vh.y) / TILE_PX).floor() as i32 - 1;
                            let vy1 = ((camera.y + vh.y) / TILE_PX).ceil() as i32 + 1;
                            let any_animated = (vy0..=vy1).any(|ty| {
                                (vx0..=vx1).any(|tx| {
                                    world.tile(tx, ty).is_some_and(|t| {
                                        t.active()
                                            && tr_core::drawdata_anim::TileAnimator::is_animated_kind(
                                                t.kind,
                                            )
                                    })
                                })
                            });
                            let base = if any_animated {
                                self.tile_anim.anim_key()
                            } else {
                                0
                            };
                            // W4-g34 union (research/197 §2.4): ALL 92
                            // critter-cage ids animate through the SEPARATE
                            // cage tables (`AnimateTiles_CritterCages`), not
                            // `tileFrame`, and none is in `is_animated_kind`'s
                            // gate set — so they need their own contribution.
                            // Gated on ANY cage kind being active in the view
                            // rect (the same convention); a cage-free view
                            // keeps the pre-wiring key bit-for-bit. A
                            // lane-2-only gate would leave the g33-half cages
                            // (bunny 275, penguin, owl, ...) animating with a
                            // stale key under the union wiring.
                            let any_cage = (vy0..=vy1).any(|ty| {
                                (vx0..=vx1).any(|tx| {
                                    world.tile(tx, ty).is_some_and(|t| {
                                        t.active()
                                            && tr_core::drawdata_furniture::is_cage_kind(
                                                t.kind,
                                            )
                                    })
                                })
                            });
                            if any_cage {
                                // Mix the cage key into the tile-animator key
                                // with one more FNV round so the two inputs
                                // cannot collide with a tile-only state.
                                let mut h = base;
                                for b in self.cage_anim.anim_key().to_le_bytes() {
                                    h ^= b as u64;
                                    h = h.wrapping_mul(0x0000_0100_0000_01b3);
                                }
                                h
                            } else {
                                base
                            }
                        },
                        // W4-d58: the chest lid frames feed the chest
                        // rewrite's source rects inside the cached range
                        // (`CacheSpecialDraws_Part2`,
                        // `TileDrawing.cs:4309-4320`). Gated on a
                        // BasicChest tile (21/467) being active in the
                        // view rect (the tile_anim convention): a
                        // chest-free view keeps the pre-wiring cache
                        // behaviour bit-for-bit. The hash covers every
                        // live chest's anchor + frame — the lid animator
                        // ticks on the world step, so an opening chest
                        // rebuilds the range with the camera still.
                        chest_frames: {
                            let vh = screen * 0.5 / *zoom;
                            let vx0 = ((camera.x - vh.x) / TILE_PX).floor() as i32 - 1;
                            let vx1 = ((camera.x + vh.x) / TILE_PX).ceil() as i32 + 1;
                            let vy0 = ((camera.y - vh.y) / TILE_PX).floor() as i32 - 1;
                            let vy1 = ((camera.y + vh.y) / TILE_PX).ceil() as i32 + 1;
                            let any_chest = (vy0..=vy1).any(|ty| {
                                (vx0..=vx1).any(|tx| {
                                    world.tile(tx, ty).is_some_and(|t| {
                                        t.active()
                                            && tr_core::drawdata_furniture::is_basic_chest(
                                                t.kind,
                                            )
                                    })
                                })
                            });
                            if any_chest {
                                crate::chest::chest_frame_key(&self.world_chests)
                            } else {
                                0
                            }
                        },
                        // W4-f14: the cage animator's frame tables feed the
                        // cage arms' addFrY rewrites inside the cached range
                        // — the state must ride the key, gated on a cage
                        // kind being active in the view rect (the tile_anim
                        // key's convention): a cage-free view keeps the
                        // pre-wiring cache behaviour bit-for-bit.
                        cage_anim: {
                            let vh = screen * 0.5 / *zoom;
                            let vx0 = ((camera.x - vh.x) / TILE_PX).floor() as i32 - 1;
                            let vx1 = ((camera.x + vh.x) / TILE_PX).ceil() as i32 + 1;
                            let vy0 = ((camera.y - vh.y) / TILE_PX).floor() as i32 - 1;
                            let vy1 = ((camera.y + vh.y) / TILE_PX).ceil() as i32 + 1;
                            let any_cage = (vy0..=vy1).any(|ty| {
                                (vx0..=vx1).any(|tx| {
                                    world.tile(tx, ty).is_some_and(|t| {
                                        t.active()
                                            && tr_core::drawdata_furniture::is_cage_kind(t.kind)
                                    })
                                })
                            });
                            if any_cage {
                                self.cage_anim.anim_key()
                            } else {
                                0
                            }
                        },
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
                        herb_weather: {
                            // W4-g80: the 83 herb-maturity sheet swap
                            // (`TileDrawing.cs:1450-1455` → `WorldGen.
                            // IsAlchemyPlantHarvestable`, `WorldGen.cs:
                            // 66148-66175`) reads weather the light key does
                            // not carry: cloudAlpha's > 0 gate, the deathweed
                            // moon gate and the fireblossom time gate. Gated
                            // on an herb 83 in the view rect (the vine-grid
                            // key's convention) — herb-free views keep the
                            // pre-wiring cache behaviour bit-for-bit.
                            let vh = screen * 0.5 / *zoom;
                            let vx0 = ((camera.x - vh.x) / TILE_PX).floor() as i32 - 1;
                            let vx1 = ((camera.x + vh.x) / TILE_PX).ceil() as i32 + 1;
                            let vy0 = ((camera.y - vh.y) / TILE_PX).floor() as i32 - 1;
                            let vy1 = ((camera.y + vh.y) / TILE_PX).ceil() as i32 + 1;
                            let any_herb = (vy0..=vy1).any(|ty| {
                                (vx0..=vx1).any(|tx| {
                                    world
                                        .tile(tx, ty)
                                        .is_some_and(|t| t.active() && t.kind == 83)
                                })
                            });
                            if any_herb {
                                // The fireblossom gate reads `world.time` (the
                                // harvest predicate's own input), not the
                                // pinned render clock.
                                (rain.visual_alpha() > 0.0) as u32
                                    | (((world.blood_moon || world.moon_phase == 0) as u32) << 1)
                                    | (((world.time > 40500.0) as u32) << 2)
                            } else {
                                0
                            }
                        },
                        // W4-g12: `DrawGoreBehind`'s drip quads draw inside
                        // the cached range — 0 while no drip slot draws
                        // (the waterfall convention).
                        back_gore: WorldFrameKey::back_gore_key(gore),
                        // W4-d70: the Moon Lord parts' `GetMoonLordColoring`
                        // tint rides the cached world range; the strength only
                        // rides the key while a part is present (the tile_anim
                        // key's convention — Moon-Lord-free worlds keep the
                        // pre-wiring cache behaviour bit-for-bit).
                        moon_lord_strength: if npcs.iter().any(|n| {
                            matches!(
                                n.kind,
                                crate::npc::NpcKind::MoonLordHead
                                    | crate::npc::NpcKind::MoonLordHand
                                    | crate::npc::NpcKind::MoonLordCore
                                    | crate::npc::NpcKind::MoonLordFreeEye
                            )
                        }) {
                            filter_driver.latest_filter_strength.to_bits()
                        } else {
                            0
                        },
                        world_revision: world.revision,
                        // W4-d127 — the WoF draw inputs inside the cached
                        // world range (`Main.DrawWoF`, `Main.cs:38377-38525`):
                        // the mouth clock, the band statics and the tongue
                        // gate. Non-zero only while a wall lives, so wall-free
                        // worlds keep the pre-wiring cache behaviour. The
                        // clock is the NEXT value `draw_tiles` will tick to
                        // (it advances inside the pass, `:38427`), so the key
                        // matches the frame the pass actually draws.
                        wof_key: if world.wof_present {
                            let next_idx = if game_paused_for_specials {
                                scratch.wof_draw_frame_index
                            } else {
                                (scratch.wof_draw_frame_index + 1) % 18
                            };
                            let tongue = player.as_ref().is_some_and(|p| p.tongued && !p.dead);
                            // W4-d127 re-verify (by inspection): the four fields
                            // occupy disjoint bit ranges — clock [0,8), band top
                            // [8,24), band bottom [24,40), tongue [40,41). The
                            // band statics are i32 but ALWAYS valid while a wall
                            // lives: the first style27_step initialises them
                            // (ease_band's -1 -> target branch) in the update
                            // half, which runs ahead of this draw half, and the
                            // clamps hold them in [underworld+10, +80]*16 (max
                            // 2400*16 = 38400 < 2^16). So the sign-extended
                            // `as i64 as u64` shift never sees a negative and the
                            // fields never overlap — the pack is injective over
                            // reachable states (the wof_key_field_layout witness
                            // pins it). The clock is the NEXT value draw_tiles
                            // ticks to, so the key matches the drawn frame.
                            (next_idx as u64)
                                | ((world.wof_draw_top as i64 as u64) << 8)
                                | ((world.wof_draw_bottom as i64 as u64) << 24)
                                | ((tongue as u64) << 40)
                        } else {
                            0
                        },
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
                    // `Main.cs:61832-61833`: `Cloud.UpdateCloudParallax` runs
                    // per draw, right after the camera update — each active
                    // cloud shifts against the camera-centre X delta by its
                    // own `GetParallax` fraction (W4-g69).
                    cloud_field.update_parallax(draw_cam);
                    // `Main.DrawBG`'s head (`Main.cs:63980-63984`): the
                    // `resetClouds` latch — `windSpeedCurrent = windSpeedTarget`
                    // then the full respawn (`Cloud.cs:39-58`).
                    if cloud_field.reset_pending {
                        wind.speed_current = wind.speed_target;
                        let (sw, sh) = (screen.x, screen.y);
                        let env = crate::cloud::CloudEnv::gather(
                            world,
                            Vec2::new(sw, sh),
                            wind.speed_current,
                            player.as_ref().map(|p| p.vx).unwrap_or(0.0),
                            menu.is_some() || front.is_some(),
                            world.day_rate(),
                            rain.raw_alpha(),
                            bg_alpha_front[4],
                        );
                        cloud_field.reset_clouds(&env, lookups);
                        cloud_field.reset_pending = false;
                    }
                    // `DrawSurfaceBG`'s `cloudBGAlpha` ramp
                    // (`Main.cs:59358-59373`) — per draw, gated on
                    // `!drunkWorldGen && ShouldDrawSurfaceBackground() &&
                    // screenPosition.Y/16 <= worldSurface + 10` (`:59354`)
                    // AND `BackgroundEnabled` (`:59356`); the same gate
                    // chain `draw_sky`'s storm-bank block folds (W4-g69
                    // deviation fix — the retired 10×-per-tick intensity
                    // chase is gone). The C# runs it between the distant
                    // and closer cloud passes; the port runs it at the draw
                    // head — no pass before the storm banks reads the value.
                    {
                        let ramp_view_top = draw_cam.y - screen.y * 0.5;
                        let ramp_should_draw = !world
                            .seeds
                            .contains(tr_core::seeds::SeedFlags::REMIX)
                            && !(settings.map_enabled
                                && map.as_ref().is_some_and(|m| m.open));
                        if !world.seeds.contains(tr_core::seeds::SeedFlags::DRUNK)
                            && ramp_should_draw
                            && ramp_view_top <= surface_px + 10.0 * TILE_PX
                            && settings.background
                        {
                            rain.cloud_bg_draw_step(
                                cloud_field.cloud_bg_active,
                                world.day_rate(),
                            );
                        }
                    }
                    // S1-G-13 consumers: the gated layer path renders the sky
                    // into the skyTarget at the present site (after the surface
                    // is acquired) and the underground backdrop into the
                    // backgroundTarget via the `RenderBackground` callback;
                    // the fused default keeps both in this pass.
                    let layer_targets_on = crate::env::layer_targets();
                    draw_sky(
                        renderer,
                        draw_cam,
                        bg_zoom,
                        screen,
                        world,
                        cloud_field,
                        sky,
                        &stars.stars,
                        surface_px,
                        is_day,
                        clock_time,
                        anim_time,
                        sky_uv,
                        sky_tint,
                        rain.cloud_bg(),
                        settings.background,
                        settings.map_enabled && map.as_ref().is_some_and(|m| m.open),
                        cave_parallax(settings.parallax_scroll),
                        bg_alpha_front,
                        bg_alpha_far,
                        light,
                        hell_black_box_bottom,
                        // `Main.shimmerAlpha` (W4-e19 aether arm): the live
                        // SceneState ramp — 1 skips the underground backdrop
                        // + the natural stars, >0 dims them.
                        scene_state.shimmer_alpha,
                        &self.ambience.entities,
                        lookups,
                        // W4-f39: the SlimeSky draws only while active
                        // (the SkyManager's active list).
                        self.slime_sky.is_active().then_some(&self.slime_sky),
                        sky_tint,
                        self.atmo,
                        self.game_update_count,
                        self.mouse_text_color,
                        // W4-c39 exec: the PartySky balloons ride the sky
                        // pass's depth boundaries (research/114 §1e).
                        &self.event_skies,
                        // `DrawStarsInBackground`'s head (W4-g69 retire sweep):
                        // the graveyard/shimmer statics + the RAW cloudAlpha
                        // for the `:63051` star gate.
                        scene_state.graveyard_visual_intensity,
                        scene_state.shimmer_alpha,
                        rain.raw_alpha(),
                        &*forest_hills,
                        std::array::from_fn(|n| bg_flash.get_flash_power(n)),
                        back.sun,
                        back.moon,
                        rain_alpha,
                        world.eclipse,
                        self.mushroom_light_influence,
                        // `Main.player[Main.myPlayer].head` — the effective
                        // head armour slot (`Player.cs`'s armor walk); the
                        // Sunglasses (237) resolve to 12.
                        self.equip
                            .armor
                            .first()
                            .map_or(-1, |s| {
                                if s.count > 0 {
                                    crate::registry::get(s.kind).map_or(-1, |d| d.head_slot)
                                } else {
                                    -1
                                }
                            }),
                        scene_state.graveyard_visual_intensity,
                        // W4-g72: the live style pick's inputs — the scene
                        // metrics + the underground style latch.
                        &self.scene_metrics,
                        ug_back,
                        scene_state.shimmer_alpha,
                        // W4-c40: the SandstormSky tint — only the ACTIVE
                        // sky draws (`SkyManager` walks `_activeSkies`).
                        self.sandstorm_sky
                            .is_active()
                            .then_some(&self.sandstorm_sky),
                        sandstorm.severity,
                        // S1-G-13 consumers: the gated path moves the
                        // underground backdrop to the backgroundTarget.
                        layer_targets_on,
                        &self.lantern_sky,
                        anim_time % 3600.0,
                    );
                    // `Main.cs:62217`: `ScreenDarkness.DrawBack` sits right after
                    // `Overlays.Scene.Draw(Background)`.
                    crate::render::screen_fx::draw_darkness_back(renderer, screen, scene_state);
                    // W4-c39 — the LanternSky draws in the
                    // `SkyManager.DrawRemainingDepth` slot (after the surface
                    // layers, before the underground backdrop — the
                    // depth-sandwich interleave is the ambience pass's own
                    // machinery, a recorded layering deviation; f40's
                    // depth-band module is the surviving form at the merge,
                    // research/114 §1b). The PartySky balloons moved INTO
                    // draw_sky's depth boundaries (research/114 §1e).
                    crate::event_skies::draw_lanterns(
                        renderer,
                        &self.event_skies,
                        lookups,
                        draw_cam,
                        world_zoom,
                        screen,
                        anim_time % 3600.0,
                        surface_px,
                    );
                    draw_phase("world");
                    // S1-G-13 consumers: the gated path renders the world
                    // through the layer family at the present site (the
                    // executor + composites need the surface view); the
                    // fused default keeps the single cached world range.
                    if !reuse_world && !layer_targets_on {
                        // W4-g85: the NewDust internals' `Main.rand`
                        // stand-in — the `frame_rng` clone snapshot (the
                        // d54 break-seam convention: the draw holds
                        // `&World`, so the stream rides a copy written
                        // back after the pass).
                        let mut orb_rand = world.frame_rng.clone();
                        let orb_ctx = crate::app::spawn_ctx_from(
                            menu.is_some() || front.is_some(),
                            game_paused_for_specials,
                            *camera,
                            screen,
                            *zoom,
                            self.gfx_quality.max_dust_to_draw,
                        );
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
                            (liquid_anim_frame, liquid_waterfall_frame),
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
                                            // itemAnimationMax`); the prefix
                                            // rides along for the thrust's
                                            // `GetAdjustedItemScale` (S1-P-i05).
                                            (
                                                slot.kind,
                                                *swing / self.swing_max.max(f32::EPSILON),
                                                slot.prefix,
                                            )
                                        })
                                }
                                _ => None,
                            },
                            // `Player.armor[0..3]`'s item kinds — the armor draw
                            // input (head/body/legs, 0 = the empty slot).
                            // W4-f41: `PlayerFrame`'s head resolution
                            // (`Player.cs:35765`): the VANITY head (armor[10])
                            // wins when its headSlot >= 0 — the Party Hat 3732
                            // (headSlot 195) draws from the social row.
                            [
                                {
                                    let real = self
                                        .equip
                                        .armor
                                        .first()
                                        .map_or(0, |s| if s.count > 0 { s.kind } else { 0 });
                                    let vanity = self
                                        .equip
                                        .armor
                                        .get(10)
                                        .map_or(0, |s| if s.count > 0 { s.kind } else { 0 });
                                    if crate::registry::get(vanity)
                                        .is_some_and(|d| d.head_slot >= 0)
                                    {
                                        vanity
                                    } else {
                                        real
                                    }
                                },
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
                            // The `CalculateWaterStyle` head arm's input
                            // (`Main.cs:57390-57393`): the scene scan's
                            // flowing-fountain colour (-1 = none).
                            self.scene_metrics.active_fountain_color,
                            // `TrySparkling`'s draw thread: the frame count pins it.
                            (*frame_count as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ world.seed,
                            wind,
                            *grass_wind_counter,
                            *tree_wind_counter,
                            *sunflower_wind_counter,
                            // W4-g68: the leaf spawns land in the gore pool
                            // directly; `Main.rand` is the bare global stream
                            // (the tile-draw emission runs outside every
                            // SwapRandom block in the C#).
                            gore,
                            self.rngs.global(),
                            !game_paused_for_specials && !crate::env::no_wind(),
                            *vine_wind_counter,
                            // S1-F-9 slice B: the engine-driven DrawData cases'
                            // inputs (plates 428, party 455, echo sight 541).
                            &self.wiring,
                            crate::world_events::party_is_up(&self.world_events),
                            // W4-g78: `_shouldShowInvisibleBlocks` — the real
                            // sight value (echo monolith / goggles), not the
                            // bare env pin.
                            show_invisible,
                            // S1-G-1f: the wired tile animator's rows/counters
                            // (`Main.tileFrame[kind]`, `TileDrawing.cs:4533`).
                            &self.tile_anim,
                            // W4-g34 union: the critter-cage animator's tables
                            // + the `Main.critterCage` proximity flag the pass
                            // raises (`TileDrawing.cs:474`/`:5672`).
                            &self.cage_anim,
                            &mut self.critter_cage,
                            crate::env::show_invisible(),
                            // W4-g80: `Main.cloudAlpha` for the 83 herb-maturity
                            // sheet swap's waterleaf gate
                            // (`WorldGen.IsAlchemyPlantHarvestable`,
                            // `WorldGen.cs:66148-66175`) — the same live value
                            // the smart-cursor harvest arm reads
                            // (`world_sim/smart_cursor.rs`).
                            rain.visual_alpha(),
                            // S1-G-1f: the wired tile animator's rows/counters
                            // (`Main.tileFrame[kind]`, `TileDrawing.cs:4533`).
                            &self.tile_anim,
                            // W4-d58: `Main.chest[]` — the chest rewrite's
                            // lid-frame input (`TileDrawing.cs:4311`).
                            &self.world_chests,
                            // S1-G-13 consumers: the fused default draws every
                            // cell (the frozen path); the layer callbacks pass
                            // the `solidLayer` halves (`Main.cs:57329-57348`).
                            crate::render::tiles::TileFilter::All,
                            // W4-f14: the critter-cage animator — the draw
                            // clears/sets `Main.critterCage` and reads the
                            // `Main.*CageFrame` tables (TileDrawing.cs:474/
                            // :5672-5907).
                            &mut self.cage_anim,
                            // W4-f6 wiring: the lava-bubble private stream +
                            // the pool/ctx (`LiquidRenderer._random`, :132).
                            &mut self.liquid_rand,
                            dust,
                            &draw_spawn_ctx,
                            // W4-g12: the live `Main.martianLight` bounce
                            // (`_martianGlow`, `TileDrawing.cs:498-499`).
                            self.martian_light,
                            // W4-g12: `DrawGoreBehind`'s pool (the
                            // behind-tiles drip rows, `Main.cs:21842`).
                            gore,
                            // W4-g85: the orb-family emit slice
                            // (`DrawTiles_EmitParticles`' :7335/:7413 arms).
                            // The pool is `Main.dust`; the NewDust internals
                            // ride the `orb_rand` snapshot (Main.rand, S1-E-1).
                            dust,
                            &mut orb_rand,
                            &orb_ctx,
                            game_paused_for_specials,
                            // W4-d127: the WoF mouth clock's pause gate
                            // (`Main.cs:38427`).
                            game_paused_for_specials,
                            // W4-d70: the Moon Lord parts' `GetMoonLordColoring`
                            // input (`Main.cs:26325`).
                            filter_driver.latest_filter_strength,
                        );
                        world.frame_rng = orb_rand;
                        *world_key = Some(key);
                    }

                    // W4-c46 — the hardcore ghost (`LegacyPlayerRenderer.
                    // DrawPlayerFull`'s ghost arm, `:274-281`): the dead
                    // player renders no body, but the ghost flies and draws
                    // every frame — OUTSIDE the cached world pass (the cache
                    // key carries no ghost, so a cached frame still animates
                    // it). The C# draws it in the player phase; the port's
                    // post-tiles slot keeps it above the terrain like the
                    // living body (the item-drop z-order rides the cache).
                    if let (Some(p), Some(a)) =
                        (player.as_ref().filter(|p| p.ghost), atlas.as_ref())
                    {
                        crate::render::entities::draw_ghost(
                            renderer,
                            a,
                            p,
                            light,
                            &style,
                            anim_time,
                            draw_cam,
                            world_zoom,
                            screen,
                            self.mouse_text_color,
                        );
                    // W4-g13 — the Main.cs dust-site draw-call feeds (batch 3,
                    // sites 17-23). They roll the bare `Main.rand` slot
                    // (`rngs.global()`) and spawn into the dust pool, so they
                    // ride the per-frame world-effect range OUTSIDE the G6
                    // cached world pass (the rain/sand/gore dust precedent) —
                    // the C# runs them inside DrawNPCDirect / DrawNPCExtras /
                    // DrawItem / oldDrawWater, but the port fuses those into
                    // the cached `draw_tiles`, so the spawns are lifted here in
                    // the C# relative order. `FocusHelper.Allow*` is
                    // `!DrawnThisUpdate` (the mark now sets AFTER draw_region,
                    // so it reads clear through the draw on a stepping frame).
                    {
                        let allow_effects = !self.stepper.drawn_this_update;
                        let dust_ctx = crate::app::spawn_ctx_from(
                            menu.is_some() || front.is_some(),
                            self.game_paused,
                            draw_cam,
                            screen,
                            world_zoom,
                            self.gfx_quality.max_dust_to_draw,
                        );
                        let rng = self.rngs.global();
                        // Sites 21-23 (bg pass) — `DrawWaters(isBackground:true)`
                        // precedes the tile/NPC/item draws (`Main.cs:62192`).
                        // Retro/Trippy only (`!Lighting.NotRetro`, `:57519`).
                        let retro = matches!(
                            *light_mode,
                            tr_core::lighting::LightMode::Retro
                                | tr_core::lighting::LightMode::Trippy
                        );
                        let mut drew_lava = false; // `Main.drewLava` reset per DrawWaters (`:57472`)
                        if retro {
                            crate::main_dust::retro_liquid_dust_feed(
                                world, light, style.brightness, draw_cam, screen, world_zoom,
                                true, allow_effects, &mut drew_lava, rng, dust, &dust_ctx,
                            );
                        }
                        // Site 17 — the Moon Lord core tentacle-tip puff
                        // (`DrawNPCDirect`'s type-398 arm, the dead flag8 latch).
                        crate::main_dust::moon_lord_tentacle_feed(npcs, rng, dust, &dust_ctx, world);
                        // Sites 18/19 — the Tavernkeep ale-toss sparkle
                        // (`DrawNPCExtras`' 550 arm, `!beforeDraw`).
                        crate::main_dust::tavernkeep_ale_feed(
                            npcs, player.as_ref(), rng, dust, &dust_ctx, world,
                        );
                        // Site 20 — the world-item coin/heart sparkle
                        // (`DrawItems` → `DrawItem`, `Main.cs:62278`).
                        crate::main_dust::item_sparkle_feed(
                            items, allow_effects, style.brightness, light, rng, dust, &dust_ctx,
                            world,
                        );
                        // Sites 21-23 (fg pass) — `DrawWaters()` after the
                        // entities (`Main.cs:62300`); a fresh drewLava reset.
                        if retro {
                            let mut drew_lava_fg = false;
                            crate::main_dust::retro_liquid_dust_feed(
                                world, light, style.brightness, draw_cam, screen, world_zoom,
                                false, allow_effects, &mut drew_lava_fg, rng, dust, &dust_ctx,
                            );
                        }
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
                    // S1-G-1 wave C / W4-g77: the tile-crack overlay
                    // (`Main.DrawTileCracks`, `Main.cs:61533-61626`) over
                    // the local player's real i29 `HitTile` book — all
                    // four sites in the C# layer order: the wall pair
                    // (`WallDrawing.cs:203-204`, crack type 2) then the
                    // solid-tile pair (`TileDrawing.cs:732-733`, crack
                    // type 1), `hitReplace` before `hitTile` (both
                    // `hitReplace` sites empty — no replacement-mining
                    // book yet). Every damaged slot draws its crack
                    // stage, TTL decay included. It rides the per-frame
                    // world range right after the cached tile pass — the
                    // book decays every tick, so the crack cannot live
                    // inside the G6-cached range (the `WorldFrameKey` is
                    // untouched). The port's stand-in for the two layer
                    // epilogues; the deviations live in the
                    // `render/tile_cracks.rs` module doc (ANALYSIS
                    // "The tile-crack overlay (S1-G-1 wave C)" — the
                    // W4-g77 re-arm supersedes its projection paragraph).
                    crate::render::tile_cracks::draw_mining_cracks(
                        renderer,
                        world,
                        light,
                        &style,
                        draw_cam,
                        anim_time,
                        hit_tile,
                        mine_crack_styles,
                        mine_crack_rng,
                        crate::env::show_invisible(),
                    );
                    // W4-g86: the smart-cursor tile-outline overlay
                    // (`TileDrawing.GetTileOutlineInfo` :4478-4489 + the
                    // draw tail :1404-1408). Like the crack overlay it
                    // rides the per-frame world-effect range right after
                    // the cached tile pass — the highlight lists track the
                    // mouse every tick, so the outline cannot live inside
                    // the G6-cached range (the `WorldFrameKey` is
                    // untouched). The lists come from the
                    // `SmartInteractLookup` feed (`world_sim/
                    // smart_interact.rs`, the `Player.cs:25447` seam).
                    crate::render::tile_outlines::draw_tile_outlines(
                        renderer,
                        world,
                        light,
                        draw_cam,
                        world_zoom,
                        screen,
                        &self.smart_interact,
                        &self.tile_anim,
                        &self.wind,
                        &self.wiring,
                        crate::world_events::party_is_up(&self.world_events),
                        crate::env::show_invisible(),
                        player.as_ref().map(|p| p.center().0),
                        anim_time % 3600.0,
                        self.tile_anim.weather_vane_bobframe,
                    );
                    // W4-g68 wiring: `TileDrawing.EmitLiquidDrops`
                    // (`TileDrawing.cs:7594-7674`, the `Draw` switch arm
                    // `:688-694`) — the vine-tip/stalactite/shimmer-candle
                    // drips. Like the crack overlay it rides the per-frame
                    // world range OUTSIDE the G6-cached tile pass (the
                    // `WorldFrameKey` is untouched): the C# rolls its
                    // `Next(num*2)` gate per DRAW, so a cached rebuild
                    // cadence would starve the drips in a calm world.
                    // The gate stream is the App's persistent
                    // `TileDrawing._rand` stand-in (`tile_draw_rand`); the
                    // spawn internals ride the bare global `Main.rand`
                    // (outside every SwapRandom block). The g30
                    // tile-pass-attach contract (UNION-CONTRACT-tilepass
                    // §5) feeds the same cells once the g-family lands —
                    // until then the driver rescans the draw area.
                    // Gate: the `emit_effects` stand-in the leaf arm uses
                    // (`!game_paused && !TR_NO_WIND`, the A/B switch).
                    // The exact C# `_isActiveAndNotPaused` is the
                    // DrawnThisUpdate flag (`FocusHelper.cs:27`), NOT
                    // `!gamePaused` — the port's stepper already carries
                    // the flag (`world.rs:90/:688`), but its set-site is
                    // the draw half's HEAD, so reading it here would be
                    // vacuously true; the flag-gate reconciliation is the
                    // d60 wiring wave's contract (ANALYSIS "The wind leaf
                    // particles", deferred-2).
                    if !game_paused_for_specials && !crate::env::no_wind() {
                        let mut water_style =
                            crate::biome::calculate_water_style(world, *camera) as i32;
                        if let Some(n) = crate::env::water_style() {
                            water_style = n.min(14) as i32;
                        }
                        crate::render::foliage::emit_liquid_drops_area(
                            world,
                            draw_cam,
                            world_zoom,
                            screen,
                            water_style,
                            self.scene_metrics.zone_corrupt,
                            self.scene_metrics.zone_crimson,
                            self.scene_metrics.zone_hallow,
                            &mut self.tile_draw_rand,
                            self.rngs.global(),
                            gore,
                        );
                    }
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
                            show_invisible,
                        };
                        // W4-f6 wiring: the special-tile dust rolls — the
                        // pylon gates on the private `tile_draw_rand`, the
                        // void lens + the pylon spawn on the bare global.
                        // W4-g86: the pylon/void-lens selection glow reads the
                        // same smart-interact highlight area the outline pass
                        // uses (`TileDrawing.cs:8687-8695`/`:8721-8729`).
                        let highlight_area = crate::render::tile_outlines::HighlightArea {
                            coords: &self.smart_interact.coords,
                            selected: &self.smart_interact.selected,
                            hide_ui: false,
                            lx: self.smart_interact.lx,
                            ly: self.smart_interact.ly,
                            hx: self.smart_interact.hx,
                            hy: self.smart_interact.hy,
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
                            &mut self.tile_draw_rand,
                            self.rngs.global(),
                            dust,
                            &draw_spawn_ctx,
                            &highlight_area,
                        );
                    }

                    // W4-g68: the leaf gores live in `App.gore` now — the
                    // foliage/tile passes spawned them during the world
                    // pass, `Gore_UpdateLeaf` ticks them in the `gores`
                    // phase (`Main.cs:17971`), and `Main.DrawGore` below
                    // draws them in the world-effect range like the rain.
                    // The old private 400-cap `LeafGore` Vec and its
                    // draw-time tick are retired.

                    draw_phase("rain");
                    rain.draw(
                        renderer,
                        lookups,
                        light,
                        draw_cam,
                        world_zoom,
                        screen,
                        scene_state.shimmer_alpha,
                    );
                    // `Main.DrawGore` (`Main.cs:62282`, body `:21873-21907`):
                    // the gore pool's front pass between the rain and the
                    // dust, in the same world-effect range (S1-E-4b).
                    crate::render::gore::draw(
                        gore, renderer, lookups, light, &style, anim_time, draw_cam, world_zoom,
                        screen,
                    );
                    // `Main.DrawDust`'s 268 entries: the flying sand, the same
                    // world-effect range as the rain (`Main.DrawRain`'s draw call
                    // site — world content, under the map batches and the UI).
                    crate::sand::draw(dust, renderer, lookups, light, draw_cam, world_zoom, screen);
                    // The type-213 wheel sparks — `Main.DrawDust`'s generic
                    // 213-draw (self-lit gray, `Dust.cs:2498-2501`), the same
                    // world-content range as the rain/sand dust.
                    crate::minecart::draw(dust, renderer, lookups, draw_cam, world_zoom, screen);
                    // g67 (transplanted): the rain collision splashes —
                    // `Main.DrawDust`'s generic path over the `dustWater()`
                    // family slots (`Rain.cs:122`'s spawns), same
                    // world-effect range.
                    crate::rain::draw_splashes(
                        dust, renderer, lookups, light, draw_cam, world_zoom, screen,
                    // The cart wheel sparks (213/260/211/107) — `Main.DrawDust`'s
                    // self-lit 213|260 gray + the light-tinted 211/107 generic
                    // path, the same world-content range as the rain/sand dust.
                    crate::minecart::draw(dust, renderer, lookups, light, draw_cam, world_zoom, screen);
                    crate::render::foliage::draw_leaves(
                        renderer, lookups, light, draw_cam, world_zoom, screen, leaves,
                    );
                    // The type-76 snowflakes — `Main.DrawDust`'s generic path
                    // (`Main.cs:38968-39018`), the same world-content range
                    // (W4-d130).
                    crate::snow::draw(dust, renderer, lookups, light, draw_cam, world_zoom, screen);
                    // `Main.DrawWires` (`Main.cs:57028-57030`): the wire overlay
                    // rides the tile batch after the liquids, under the
                    // over-wire entities (the port draws it before the
                    // projectiles, matching `DrawCachedProjsOverWiresUI`).
                    // `WiresUI.Settings.DrawWires` gates the whole pass
                    // (`CaptureInterface.cs:1088`/`:1170` set `CaptureMech`):
                    // a held mech tool or the lens/kite draws it, else nothing.
                    draw_phase("wires");
                    let held = inventory.get(*selected).copied().unwrap_or(crate::item::EMPTY_SLOT);
                    let held_mech = crate::registry::get(held.kind).is_some_and(|d| d.mech);
                    let wire_view = player.as_ref().and_then(|p| {
                        crate::render::wires::wire_view(
                            held.kind,
                            held.count,
                            held_mech,
                            p.info_acc_mech_show_wires,
                            &p.builder_acc_status,
                            world.seeds.contains(tr_core::seeds::SeedFlags::NO_TRAPS),
                            world.downed_skeletron,
                        )
                    });
                    if let Some(view) = wire_view {
                        crate::render::wires::draw_wires(
                            renderer, world, light, &style, draw_cam, world_zoom, screen, &view,
                        );
                    }
                    // S1-W4-g81 residuals — the Wire Kite's placement ghost
                    // (`Main.cs:31620-31907`, drawLayer 5): the over-wire
                    // cached projectile draws right after `DrawWires`
                    // (`Main.cs:62303` → `:62315`), before the generic
                    // projectile pass. The generic pass below skips 651
                    // (the C#'s CacheProjDraws routes it out of the main
                    // walk).
                    if let Some(pl) = player.as_ref() {
                        for p in projectiles.slots().iter().filter(|p| p.active) {
                            if p.kind == crate::projectile::WIRE_KITE {
                                crate::render::wire_kite::draw_wire_kite(
                                    renderer,
                                    lookups,
                                    p,
                                    self.wire_tool_mode,
                                    pl.facing == 1,
                                    draw_cam,
                                    world_zoom,
                                    screen,
                                    world.width,
                                    world.height,
                                );
                            }
                        }
                    }
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
                    for (pi, p) in projectiles.slots().iter().enumerate().filter(|(_, p)| p.active) {
                        // W4-g56 — `Main.DrawProjDirect`'s type-734 arm
                        // (`Main.cs:27920-27931`): the Void Lens draws via
                        // `VoidLensHelper.DrawToDrawData` (the projectile
                        // ctor — Center / Opacity / frame), never the
                        // generic emit. `selectionMode` is the per-frame
                        // `TryInteractingWithVoidLens` answer
                        // (`App.void_lens_selection`, the draw pass runs it
                        // per lens slot); the smart-cursor arm is unported
                        // (mode 0/1 only).
                        if p.kind == crate::projectile::VOID_LENS {
                            let selection = self
                                .void_lens_selection
                                .get(pi)
                                .copied()
                                .unwrap_or(0);
                            crate::render::pylons::draw_void_lens_projectile(
                                renderer,
                                light,
                                &style,
                                draw_cam,
                                world_zoom,
                                screen,
                                anim_time,
                                anim_time % 3600.0,
                                [p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5],
                                p.frame,
                                1.0 - p.alpha as f32 / 255.0,
                                selection,
                            );
                            continue;
                        }
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
                        // W4-d16: the kite draw's two scene reads —
                        // `Main.WindForVisuals` (`Main.cs:28056`) and the
                        // `timeForVisualEffects` tick clock (`:37467`, the
                        // world pass's `anim_time * 60 % 216000` form).
                        p.draw_with_hand(
                            renderer,
                            lookups,
                            light,
                            draw_cam,
                            world_zoom,
                            screen,
                            hand,
                            anim_time % 3600.0,
                            wind.speed_current,
                            (anim_time as f64 * 60.0) % 216000.0,
                            // W4-g35: the 993 rainbow bobber's glow tint.
                            self.disco,
                        );
                    }

                    // `Main.cs:62316-62317`: `DrawInfernoRings` then
                    // `DrawPaladinsShield`, after the over-wire cached
                    // projectiles (`:62315`) and before `DrawLensFlare`.
                    // `FocusHelper.AllowPlayerToEmitEffects` (`FocusHelper.cs:29`)
                    // is `!WorldUpdateStepper.DrawnThisUpdate` — the ring
                    // animation advances only on a frame that did not step.
                    if let Some(p) = player.as_mut() {
                        crate::render::screen_fx::draw_inferno_rings(
                            renderer,
                            lookups,
                            p,
                            draw_cam,
                            world_zoom,
                            screen,
                            !self.stepper.drawn_this_update,
                        );
                    }
                    if let Some(p) = player.as_ref() {
                        // The C# walks `Main.player[0..255]` skipping
                        // `Main.myPlayer` (`Main.cs:57205-57231`); the port's
                        // peers are the `remote_players` snapshots, whose
                        // team/dead ride the `13`/`12`/`45` wires and whose
                        // shield flag re-derives from the synced inventory
                        // (`PlayerNet::paladin_shield`). Slot order, like the
                        // C#'s array walk.
                        let mut peers: Vec<_> = remote_players.values().collect();
                        peers.sort_by_key(|rp| rp.id);
                        let peers: Vec<_> = peers
                            .into_iter()
                            .map(|rp| crate::render::screen_fx::PaladinPeer {
                                mounted_center: [
                                    rp.x + crate::player::WIDTH * 0.5,
                                    rp.y + crate::player::HEIGHT * 0.5,
                                ],
                                team: rp.team,
                                has_paladin_shield: rp.paladin_shield(),
                                dead: rp.dead,
                                hp: rp.hp,
                                max_hp: rp.max_hp,
                            })
                            .collect();
                        crate::render::screen_fx::draw_paladins_shield(
                            renderer,
                            lookups,
                            p,
                            &peers,
                            draw_cam,
                            world_zoom,
                            screen,
                        );
                    }
                    // `Main.cs:62320`: `DrawLensFlare`, under the background
                    // zoom (`PlayerInput.SetZoom_Background`). The
                    // `ShouldDrawSurfaceBackground` gate (`Main.cs:59630-59642`)
                    // and `HorizonHelper.SunVisibilityEnabled` (the sun-occlusion
                    // pixel target, unported → assumed visible) gate the chain.
                    {
                        let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
                        let drunk = world.seeds.contains(tr_core::seeds::SeedFlags::DRUNK);
                        let map_fullscreen =
                            settings.map_enabled && map.as_ref().is_some_and(|m| m.open);
                        let should_draw_surface_bg = !remix && !(remix && drunk) && !map_fullscreen;
                        crate::render::screen_fx::draw_lens_flare(
                            renderer,
                            lookups,
                            world,
                            // W4-d70 DOC-SWEEP: the advanced-shadow offset +
                            // the sun-scorch override read the local player.
                            player.as_ref(),
                            draw_cam,
                            bg_zoom,
                            screen,
                            self.atmo,
                            rain.visual_alpha(),
                            should_draw_surface_bg,
                            true,
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
                    // `Main.cs:62334`: the `!Main.hideUI` block — the combat
                    // text (and the player chat bubbles, unported) skip while
                    // F11 hides the interface.
                    draw_phase("combat_text");
                    if !self.hide_ui {
                        crate::combat_text::draw(
                            renderer,
                            lookups,
                            combat_texts,
                            draw_cam,
                            world_zoom,
                            screen,
                        );
                    }
                    // `Main.cs:62397-62402` — `PopupText.DrawItemTextPopups`
                    // closes the floating-text pair right after the combat
                    // text (W4-g57): the reforge reveal's rise/fade + the
                    // best-tier rainbow. Same `TargetScale` the update tick
                    // used (`PopupText.cs:88` + the `:62398-62400` guard).
                    {
                        let ui_scale = settings.applied_ui_scale(screen.x, screen.y);
                        let mut target_scale = if world_zoom > 0.0 {
                            ui_scale / world_zoom
                        } else {
                            0.0
                        };
                        if target_scale == 0.0 {
                            target_scale = 1.0;
                        }
                        crate::popup_text::draw(
                            renderer,
                            lookups,
                            &self.popup_texts,
                            draw_cam,
                            world_zoom,
                            screen,
                            target_scale,
                            crate::popup_text::Env {
                                disco: self.disco,
                                master_color: self.master_color,
                                mouse_text_color: self.mouse_text_color,
                                global_time_wrapped_hourly: anim_time % 3600.0,
                                grav_dir: 1.0,
                            },
                        );
                    }
                    draw_phase("entity_health_bars");
                    // Layer 14 (`Main.cs:44784-44788`) sits after the layer-8
                    // hide check.
                    if !self.hide_ui {
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
                    }

                    // `DrawInterface_7_TownNPCHouseBanners` (`Main.cs:46214-46222`,
                    // `DrawNPCHousesInWorld` `:40711-40832`): the world housing
                    // banners draw while `EquipPage == 1` (the housing tab). The
                    // hover boxes park on the App for the right-click kick-out
                    // (`Main.cs:40824-40829`, W4-g21 batch 4 host 53).
                    if self.equip_page == 1 && !self.hide_ui {
                        self.house_banners = crate::house_banners::draw_house_banners(
                            renderer,
                            lookups,
                            world,
                            npcs,
                            &self.town_rooms,
                            light,
                            &style,
                            anim_time,
                            draw_cam,
                            world_zoom,
                            screen,
                        );
                    } else {
                        self.house_banners.clear();
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
                    // `DrawInterface_16_MapOrMinimap` (`Main.cs:44794-44798`) sits
                    // AFTER the F11 hide toggle in the layer list, so `hideUI`
                    // stands the minimap down too (W4-g21 batch 2).
                    let hide_ui = self.hide_ui;
                    // The minimap branch draws **before** the HUD: the vanilla
                    // interface passes are ordered 16 (map/minimap,
                    // `Main.cs:44796`) < 25/27/30 (resources/inventory/hotbar,
                    // `:44831/:44841/:44861`), so the hearts and the open inventory
                    // paint over the frame. `Main.DrawMap`'s `mapStyle == 1`
                    // branch (`Main.cs:55503-55547`): the three vanilla passes in
                    // draw order — the `DrawTo` black backdrop (`:55546`, via
                    // `map_back` so it lands under the tiles), the tile pass
                    // (`MapRenderer.DrawMap`, `:55609`) and the frame foreground
                    // batch (`:55768-55769`). `mapStyle == 2` (the overlay)
                    // draws its tile pass + layer walk below (D18's draw arm,
                    // `Main.cs:55548-55578`/`:55609`/`:55615-55627`).
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
                        && !hide_ui
                        // `Main.hideUI` stops the layer loop before layer 16
                        // (Map/Minimap, `Main.cs:44794-44798`).
                        && !self.hide_ui
                        // `Main.inFancyUI` (`IngameFancyUI.cs:64-72`): the
                        // bestiary state covers the interface — the minimap HUD
                        // does not draw under it.
                        && !self.bestiary_open
                        && self.map_style == 1
                    {
                        if let Some(m) = map.as_ref().filter(|m| m.map_ready) {
                            // `Main.DrawMap`'s second gate (`Main.cs:55283`):
                            // `!Main.mapEnabled || !Main.mapReady` — no map
                            // pixels until the first `DrawToMap` completes
                            // (`MapRenderer.cs:196`).
                            let pos = ui::minimap::draw_to_position(ui::minimap::minimap_origin_raw(
                                screen, map_scale,
                            ));
                            // `Main.mapMinimapAlpha` (`Main.cs:66753` seed 1;
                            // the MapAlphaUp/Down keys step it — W4-f46 dup
                            // fold of g53 `a9774d2a`/e18 `249a0c1b`).
                            // `Main.mapMinimapAlpha` (`Main.cs:66753`, the
                            // MapAlphaUp/Down keys step it — W4-g53).
                            ui::minimap::draw_background(renderer, lookups, pos, m.minimap_alpha, map_scale);
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
                            let (layer_hover, _, _) = crate::map_layers::map_layers_draw(
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
                                    map_fullscreen: false,
                                    pylon_tile: m.pylon_tile,
                                    screen,
                                    left_press: self.map_left_press,
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
                                m.minimap_alpha, // `b` (`Main.cs:55532` → `:55703`)
                            );
                            // `Main.cs:55705-55706`: the revenge markers +
                            // the misc icons (return portals / golf ball),
                            // between the icon layers and the NPC heads.
                            // The C# passes the transform-space
                            // `(num12, num13)` / `(num3+num10, num4+num11)`
                            // pair; the port folds `MapScale` into the
                            // positions (the `minimap_origin_raw`
                            // convention), so the pair rides screen space
                            // and the clip is the scaled 240x240 panel.
                            let mm_origin_raw = ui::minimap::minimap_origin_raw(screen, map_scale);
                            let mm_scale_raw = m.minimap_scale;
                            let mm_win = 240.0 / mm_scale_raw.clamp(0.2, 3.0);
                            let mm_left = view_center[0].floor() - mm_win * 0.5;
                            let mm_top = view_center[1].floor() - mm_win * 0.5;
                            let mm_sub = [
                                -(view_center[0] - view_center[0].floor()) * mm_scale_raw,
                                -(view_center[1] - view_center[1].floor()) * mm_scale_raw,
                            ];
                            let mm_panel = ui::minimap::scale_rect(
                                [mm_origin_raw[0], mm_origin_raw[1], 240.0, 240.0],
                                map_scale,
                            );
                            let misc_offset = [
                                (mm_origin_raw[0] + mm_sub[0]) * map_scale,
                                (mm_origin_raw[1] + mm_sub[1]) * map_scale,
                            ];
                            let misc_top_left = [mm_left, mm_top];
                            let misc_scale = mm_scale_raw * map_scale;
                            let revenge_hover = crate::map_player_icons::draw_revenge_map_icons(
                                renderer,
                                lookups,
                                &self.revenge_markers,
                                misc_top_left,
                                misc_offset,
                                Some(mm_panel),
                                misc_scale,
                                num46 * map_scale,
                                self.mouse_pos,
                            );
                            let golf_ball = self.golf_state.last_hit_golf_ball().and_then(|stored| {
                                let live = self.projectiles.slots().get(stored.who_am_i).map(|q| {
                                    let (cx, cy) = q.center();
                                    tr_core::golf_state::GolfBall {
                                        who_am_i: stored.who_am_i,
                                        kind: q.kind as i32,
                                        owner: q.owner as i32,
                                        active: q.active,
                                        position: q.pos,
                                        center: glam::Vec2::new(cx, cy),
                                        ai1: q.ai[1],
                                        local_ai1: q.local_ai[1],
                                    }
                                });
                                self.golf_state
                                    .last_hit_ball_for_draw(live.as_ref(), self.local_id as i32)
                            });
                            let misc_hover = crate::map_player_icons::draw_misc_map_icons(
                                renderer,
                                lookups,
                                &crate::map_player_icons::MiscIconInputs {
                                    player: player.as_ref(),
                                    golf_ball,
                                    mouse: self.mouse_pos,
                                },
                                misc_top_left,
                                misc_offset,
                                Some(mm_panel),
                                misc_scale,
                                num46 * map_scale,
                            );
                            // `Main.cs:55709`: `DrawNPCMapIcons2` — the
                            // minimap's boss/town head pass, between the
                            // icon layers and the player heads. The C#
                            // runs it in the `MapScale` batch transform
                            // (`transformMatrix2`, `:55708`), so the panel
                            // origin / left / top / sub-offsets ride the
                            // raw transform space and the clip is the raw
                            // 240x240 rect; the port folds the transform
                            // into the positions (the `minimap_origin_raw`
                            // convention). The hover text rides the same
                            // `ref text` chain — the layers' write wins
                            // over the heads' (`:55703` runs first).
                            let mm_origin = ui::minimap::minimap_origin_raw(screen, map_scale);
                            let mm_scale = m.minimap_scale;
                            let win = 240.0 / mm_scale.clamp(0.2, 3.0);
                            let left = view_center[0].floor() - win * 0.5;
                            let top = view_center[1].floor() - win * 0.5;
                            let sub = [
                                -(view_center[0] - view_center[0].floor()) * mm_scale,
                                -(view_center[1] - view_center[1].floor()) * mm_scale,
                            ];
                            let head_hover = crate::map_npc_icons::draw_npc_map_icons_minimap(
                                renderer,
                                lookups,
                                npcs,
                                [mm_origin[0] * map_scale, mm_origin[1] * map_scale],
                                &crate::map_npc_icons::NpcIconView {
                                    map_position: [left, top],
                                    sub_offset: [sub[0] * map_scale, sub[1] * map_scale],
                                    minimap_rect: Some(ui::minimap::scale_rect(
                                        [mm_origin[0], mm_origin[1], 240.0, 240.0],
                                        map_scale,
                                    )),
                                    map_scale: mm_scale * map_scale,
                                    // The head draws at `num46` inside the
                                    // `MapScale` transform — the screen-space
                                    // size/hover rect is `num46 * MapScale`
                                    // (the C# compares in transform space
                                    // against the `num16`-divided mouse,
                                    // `Main.cs:55512-55513`).
                                    head_scale: num46 * map_scale,
                                    // `b/255` (`Main.cs:55709` → the live
                                    // minimap alpha, D17 — research/140 §8.3).
                                    alpha: m.minimap_alpha,
                                    alpha: 1.0,
                                    mouse: self.mouse_pos,
                                    global_time: anim_time % 3600.0,
                                },
                            );
                            // `Main.cs:55712-55766`: the 255-player head loop
                            // — the local head plus the remote peers' pins,
                            // then the death markers. The C# runs it in the
                            // `MapScale` transform with the `num16`-divided
                            // mouse; the port folds the transform into the
                            // positions/sizes and compares the screen mouse
                            // against the scaled rects (the same convention
                            // as the NPC head pass above).
                            let net_mode = match self.net {
                                crate::app::NetRole::Host(_) => 2,
                                crate::app::NetRole::Client(_) => 1,
                                crate::app::NetRole::Off => 0,
                            };
                            let local_team = player.as_ref().map_or(0, |p| p.team);
                            let local_hostile = player.as_ref().is_some_and(|p| p.hostile);
                            let mm_hs = num46 * map_scale;
                            let mut pin_hover: Option<String> = None;
                            let mut death_hover: Option<String> = None;
                            let mut death_hovered = false;
                            // The local player first (`Main.myPlayer` is
                            // slot 0 in the port).
                            if let Some(p) = player.as_ref() {
                                let center = [
                                    p.x + p.width * 0.5,
                                    p.y + p.gfx_off_y + p.height * 0.5,
                                ];
                                let pin = crate::map_player_icons::minimap_pin_pos(
                                    center,
                                    [left, top],
                                    [mm_origin[0] * map_scale, mm_origin[1] * map_scale],
                                    [sub[0] * map_scale, sub[1] * map_scale],
                                    mm_scale * map_scale,
                                );
                                if !p.dead
                                    && crate::map_player_icons::minimap_head_gate(pin, mm_panel)
                                {
                                    let border = crate::map_player_icons::player_head_borders_color(
                                        p.dead, p.team, p.hostile, true, local_team, net_mode,
                                    );
                                    // `b/255` (`Main.cs:55729`, D17).
                                    draw_player_map_icon(renderer, player_assets, p, pin, mm_hs, border, m.minimap_alpha);
                                    draw_player_map_icon(renderer, player_assets, p, pin, mm_hs, border);
                                }
                                if p.show_last_death {
                                    if let Some(d) = p.last_death_position {
                                        let dp = crate::map_player_icons::death_marker_pos(
                                            [d.0, d.1],
                                            [left, top],
                                            [mm_origin[0] * map_scale, mm_origin[1] * map_scale],
                                            mm_scale * map_scale,
                                            [-sub[0] * map_scale, -sub[1] * map_scale],
                                        );
                                        // `:55746-55751`: the minimap death
                                        // position adds `num10/num11` (the
                                        // sub-offset) — the inset pair is
                                        // the negated sub, not 10*scale.
                                        if crate::map_player_icons::minimap_death_gate(dp, mm_panel)
                                            && crate::map_player_icons::draw_death_marker(
                                                // `White * b/255` (`:55754`, D17).
                                                renderer, lookups, dp, mm_hs, m.minimap_alpha, self.mouse_pos,
                                                // `:55755-55758`: the minimap
                                                // inline copy's extra -4/-4
                                                // hover shift (W4-f29 F5).
                                                true,
                                                renderer, lookups, dp, mm_hs, 1.0, self.mouse_pos,
                                            )
                                        {
                                            death_hovered = true;
                                            if let Some(t) = p.last_death_time {
                                                death_hover = Some(
                                                    crate::map_player_icons::death_hover_text(
                                                        &self.player_name, t,
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            // The remote peers (`Main.player[j]`, j !=
                            // myPlayer): the snapshot players, team 0 /
                            // not hostile (unsynced), hover = the name.
                            for rp in remote_players.values() {
                                if !crate::map_player_icons::player_map_icon_can_be_seen(
                                    false, false, 0, local_hostile, local_team,
                                ) {
                                    continue;
                                }
                                let pp = crate::map_player_icons::remote_pin_player(world, rp);
                                let center = [
                                    pp.x + pp.width * 0.5,
                                    pp.y + pp.gfx_off_y + pp.height * 0.5,
                                ];
                                let pin = crate::map_player_icons::minimap_pin_pos(
                                    center,
                                    [left, top],
                                    [mm_origin[0] * map_scale, mm_origin[1] * map_scale],
                                    [sub[0] * map_scale, sub[1] * map_scale],
                                    mm_scale * map_scale,
                                );
                                if crate::map_player_icons::minimap_head_gate(pin, mm_panel) {
                                    let border = crate::map_player_icons::player_head_borders_color(
                                        false, 0, false, false, local_team, net_mode,
                                    );
                                    // `b/255` (`Main.cs:55729`, D17).
                                    draw_player_map_icon(renderer, player_assets, &pp, pin, mm_hs, border, m.minimap_alpha);
                                    draw_player_map_icon(renderer, player_assets, &pp, pin, mm_hs, border);
                                    // `:55730-55740`: the peer hover rect.
                                    let hr = crate::map_player_icons::head_hover_rect(pin, mm_hs);
                                    if crate::map_player_icons::hover_inclusive(hr, self.mouse_pos) {
                                        pin_hover = Some(rp.name.clone());
                                    }
                                }
                            }
                            // The hover precedence is the C#'s `ref text`
                            // write order: layers (`:55703`) → revenge/misc
                            // (`:55705-55706`) → NPC heads (`:55709`) →
                            // peer names (`:55738`) → the death duration
                            // (`:55772-55774`, num19 wins over the revenge
                            // UseMouseOver).
                            let mm_text = death_hover
                                .or(pin_hover)
                                .or(head_hover)
                                .or(misc_hover)
                                .or(layer_hover);
                            if let Some(t) = mm_text {
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
                            // `:55778`: no death hover → the revenge
                            // marker's UseMouseOver money row.
                            if !death_hovered {
                                if let Some(i) = revenge_hover {
                                    crate::map_player_icons::draw_revenge_mouse_over(
                                        renderer,
                                        lookups,
                                        self.revenge_markers[i].coins_value,
                                        self.mouse_pos,
                                        num46 * map_scale,
                                    );
                                }
                            }
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
                    // `Main.cs:62231-62235`: the style-2 overlay's call site
                    // is the in-world draw (`DoDraw_WallsTilesNPCs`' tail),
                    // NOT interface pass 16 — the gate is only
                    // `!mapFullscreen && mapStyle == 2` plus `DrawMap`'s own
                    // `mapEnabled && mapReady` head (`:55283`). The port's
                    // map batches sit under the UI batch, so the overlay
                    // draws under the HUD/bestiary like the source; the
                    // menu/front/no_hud gates mirror the minimap branch
                    // (the in-world path does not run on the front screens).
                    if settings.map_enabled
                        && menu.is_none()
                        && front.is_none()
                        && !crate::env::no_hud()
                        && !fullscreen_map
                        && self.map_style == 2
                    {
                        if let Some(m) = map.as_ref().filter(|m| m.map_ready) {
                            // `Main.cs:55548-55578` + `:55609`: the clamps ran
                            // in the prepare block above; the tile pass tints
                            // `(byte)(255*mapOverlayAlpha)` (`:55568`).
                            let (overlay_offset, num5, alpha) =
                                m.draw_overlay(renderer, world, screen, view_center);
                            // `Main.cs:55619-55625`: the icon draw scale
                            // `num37 = (num5*0.2*2+1)/3` capped at 1, then
                            // `*= Main.UIScale`. The layer walk is the SAME
                            // `MapIcons.Draw` as the fullscreen/minimap arms
                            // at the overlay transform: `mapPosition =
                            // Vector2.Zero`, no clip (`null`), `b` opacity.
                            // The `:55626-55694` icon block below is the f29
                            // tip's overlay pick inserted INTO this re-folded
                            // branch (research/84 §10.3 step 2 / research/140
                            // §11.1) — never a second tile pass, num37 walk
                            // or hover tail.
                            let num37 = ((num5 * 0.2 * 2.0 + 1.0) / 3.0).min(1.0)
                                * settings.applied_ui_scale(screen.x, screen.y);
                            let digtoise =
                                crate::map_layers::digtoise_marker(player.as_ref(), &self.projectiles);
                            let near_pylon = player.as_ref().is_some_and(|p| {
                                let (cx, cy) = p.center();
                                tr_core::pylons::is_player_near_a_pylon(
                                    world,
                                    ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32),
                                )
                            });
                            // The third `MapIconInputs` site gains the f29
                            // tip's 4 fields (research/140 §8.5/§11.1):
                            // `map_fullscreen: false` + `pylon_tile: (-1,-1)`
                            // (no network lines — `TeleportPylonsMapLayer.cs:45`
                            // gates fullscreen), `left_press: false` (the
                            // marker click is fullscreen-only, `:83`/`:90`);
                            // the PylonClick/cancel_wormhole return halves are
                            // inert at that gate — dropped here.
                            let (layer_hover, _, _) = crate::map_layers::map_layers_draw(
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
                                    map_fullscreen: false,
                                    pylon_tile: (-1, -1),
                                    screen,
                                    left_press: false,
                                },
                                [0.0, 0.0],
                                overlay_offset,
                                None,
                                num5,
                                num37,
                                alpha,
                            );
                            // `Main.cs:55626-55627`: the revenge markers +
                            // the misc icons at the overlay transform —
                            // `Vector2.Zero` / `(num - 10*num5, num2 -
                            // 10*num5)` (the inset-free origin draw_overlay
                            // returned), no clip. Like the fullscreen pair
                            // these take NO `b` in C#.
                            let ov_revenge_hover = crate::map_player_icons::draw_revenge_map_icons(
                                renderer,
                                lookups,
                                &self.revenge_markers,
                                [0.0, 0.0],
                                overlay_offset,
                                None,
                                num5,
                                num37,
                                self.mouse_pos,
                            );
                            let ov_golf_ball = self.golf_state.last_hit_golf_ball().and_then(|stored| {
                                let live = self.projectiles.slots().get(stored.who_am_i).map(|q| {
                                    let (cx, cy) = q.center();
                                    tr_core::golf_state::GolfBall {
                                        who_am_i: stored.who_am_i,
                                        kind: q.kind as i32,
                                        owner: q.owner as i32,
                                        active: q.active,
                                        position: q.pos,
                                        center: glam::Vec2::new(cx, cy),
                                        ai1: q.ai[1],
                                        local_ai1: q.local_ai[1],
                                    }
                                });
                                self.golf_state
                                    .last_hit_ball_for_draw(live.as_ref(), self.local_id as i32)
                            });
                            let ov_misc_hover = crate::map_player_icons::draw_misc_map_icons(
                                renderer,
                                lookups,
                                &crate::map_player_icons::MiscIconInputs {
                                    player: player.as_ref(),
                                    golf_ball: ov_golf_ball,
                                    mouse: self.mouse_pos,
                                },
                                [0.0, 0.0],
                                overlay_offset,
                                None,
                                num5,
                                num37,
                            );
                            // `Main.cs:55629-55641`: the revenge marker's
                            // UseMouseOver runs BETWEEN the misc icons and
                            // the NPC heads in this arm (no death gate —
                            // the loop's num19 hover only feeds the shared
                            // MouseText tail).
                            if let Some(i) = ov_revenge_hover {
                                crate::map_player_icons::draw_revenge_mouse_over(
                                    renderer,
                                    lookups,
                                    self.revenge_markers[i].coins_value,
                                    self.mouse_pos,
                                    num37,
                                );
                            }
                            // `Main.cs:55645`: `DrawNPCMapIcons3` — the
                            // overlay head pass (fullscreen geometry, no
                            // hover text), now WIRED (research/140 §11.1).
                            crate::map_npc_icons::draw_npc_map_icons_overlay(
                                renderer,
                                lookups,
                                npcs,
                                map::map_origin(m.pos, num5, screen),
                                &crate::map_npc_icons::NpcIconView {
                                    map_position: [0.0, 0.0],
                                    sub_offset: [0.0, 0.0],
                                    minimap_rect: None,
                                    map_scale: num5,
                                    head_scale: num37,
                                    alpha,
                                    mouse: self.mouse_pos,
                                    global_time: anim_time % 3600.0,
                                },
                            );
                            // `Main.cs:55653-55692`: the 255-player loop —
                            // the overlay arm SKIPS the local head
                            // (`i != Main.myPlayer`, `:55659`) but draws
                            // EVERY player's death marker (`:55672`), at
                            // the `:55661-55668` offsets (position.Y only,
                            // no gfxOffY/height half on the head). The
                            // remote peers stand in for the 255 slots.
                            let ov_net_mode = match self.net {
                                crate::app::NetRole::Host(_) => 2,
                                crate::app::NetRole::Client(_) => 1,
                                crate::app::NetRole::Off => 0,
                            };
                            let ov_local_team = player.as_ref().map_or(0, |p| p.team);
                            let ov_local_hostile = player.as_ref().is_some_and(|p| p.hostile);
                            let ov_origin = map::map_origin(m.pos, num5, screen);
                            let mut ov_death_hover: Option<String> = None;
                            for rp in remote_players.values() {
                                if !crate::map_player_icons::player_map_icon_can_be_seen(
                                    false, false, 0, ov_local_hostile, ov_local_team,
                                ) {
                                    continue;
                                }
                                let pp = crate::map_player_icons::remote_pin_player(world, rp);
                                // `:55661-55668`: `(position + (width/2, 0))
                                // /16*num5 + origin - (6, 2 + (2 - num5/5*2))
                                // - 10*num5` — the head Y has NO gfxOffY /
                                // height-half (unlike fullscreen `:55938`).
                                let ov_pin = [
                                    (pp.x + pp.width * 0.5) / 16.0 * num5 + ov_origin[0]
                                        - 6.0 - 10.0 * num5,
                                    pp.y / 16.0 * num5 + ov_origin[1]
                                        - 2.0 - (2.0 - num5 / 5.0 * 2.0) - 10.0 * num5,
                                ];
                                let border = crate::map_player_icons::player_head_borders_color(
                                    false, 0, false, false, ov_local_team, ov_net_mode,
                                );
                                draw_player_map_icon(renderer, player_assets, &pp, ov_pin, num37, border, alpha);
                                // The peers' `showLastDeath`/`lastDeath`
                                // have no net sync (the fullscreen loop's
                                // recorded deviation) — only the local
                                // player's death marker draws below.
                            }
                            // The local player's death marker (`i ==
                            // myPlayer` still runs `:55672`).
                            if let Some(p) = player.as_ref() {
                                if p.show_last_death {
                                    if let Some(d) = p.last_death_position {
                                        let dp = crate::map_player_icons::death_marker_pos(
                                            [d.0, d.1],
                                            [0.0, 0.0],
                                            ov_origin,
                                            num5,
                                            [10.0 * num5, 10.0 * num5],
                                        );
                                        if crate::map_player_icons::draw_death_marker(
                                            renderer, lookups, dp, num37, alpha, self.mouse_pos,
                                            true,
                                        ) {
                                            if let Some(t) = p.last_death_time {
                                                ov_death_hover = Some(
                                                    crate::map_player_icons::death_hover_text(
                                                        &self.player_name, t,
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            // `Main.cs:56046-56049`: the shared `MouseText`
                            // tail — the hover line draws over the overlay.
                            // The `ref text` write order: layers (`:55625`)
                            // → revenge/misc (`:55626-55627` + the `:55634`
                            // UseMouseOver) → the death duration (num19).
                            let ov_text = ov_death_hover
                                .or(ov_misc_hover)
                                .or(layer_hover);
                            if let Some(t) = ov_text {
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
                            sign_cancel_cache: &self.sign_cancel_cache,
                            everything_glitch: self.world_advanced_glitch,
                            audio: &mut self.audio,
                            rng_global: self.rngs.global(),
                            worldgen_debug: worldgen_debug_pre.clone(),
                            worldgen_debug_open: self.worldgen_debug_open,
                        };
                        front.draw(&mut d);
                        // `IngameFancyUI.Draw`'s sign-dialog arm
                        // (`IngameFancyUI.cs:212-215`): while the ctx-1
                        // keyboard is up, `GUIChatDraw` still draws — the
                        // sign text edits in the NPCChatPanel dialog (the
                        // keyboard's own textbox is parked off-screen,
                        // `UIVirtualKeyboard.cs:176`). W4-f22.
                        if self.sign_editing() {
                            let sign_open = self.player.as_ref().is_some_and(|p| p.sign >= 0);
                            ui::npc_chat::draw_panel(
                                renderer,
                                lookups,
                                screen,
                                &self.npc_chat_text,
                                ui::UiPulse {
                                    text: self.mouse_text_color as f32 / 255.0,
                                    cursor_scale: 1.0,
                                },
                                self.mouse_pos,
                                sign_open,
                                true,
                                &mut self.sign_text_blinker,
                            );
                        }
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
                    } else if menu.is_none() && !crate::env::no_hud() && !fullscreen_map
                        // `DrawInterface_8_CheckF11UIHideToggle` (`Main.cs:46175-46183`):
                        // `Main.hideUI` returns false, so every interface layer
                        // after the toggle stands down (W4-g21 batch 2). The
                        // pre-toggle layers (emote bubbles 1.1, entity markers)
                        // are fused into this call — the recorded deviation.
                        && !self.hide_ui
                    {
                    } else if menu.is_none() && !crate::env::no_hud() && !fullscreen_map {
                        // S1-W4-g81 residuals — `Main.DrawInterface_9_WireSelection`
                        // (`Main.cs:46196-46204` → `WiresUI.HandleWiresUI`,
                        // `WiresUI.cs:698-702`): the tool-mode radial's Update+Draw,
                        // the interface layer between the town-house banners (7)
                        // and the capture check (10) — under the inventory/hotbar/
                        // cursor layers the `draw_hud` below carries. The petal
                        // clicks XOR `wire_tool_mode`; the hover write folds into
                        // the mine-suppression stand-in through `wires_radial.hovering`.
                        {
                            let held = inventory.get(*selected).copied().unwrap_or(crate::item::EMPTY_SLOT);
                            let tool_mode_ui =
                                ui::wires_ui::draw_tool_mode_ui(held.kind, held.count);
                            let allow_actuators = ui::wires_ui::draw_tool_allow_actuators(
                                held.kind,
                                held.count,
                                &mut self.wires_radial.last_actuator_enabled,
                            );
                            // The cursor-icon latch (`Player.cs:46756-46760`)
                            // — the sim's per-update in-reach write (the
                            // PLAYER field, the batch-H keep-one — g79's
                            // paint gates share it; the icon ID stays 0 for
                            // the wiring tools, so the `:130` ID gate never
                            // trips — the g86 research/244 override system
                            // owns the ID).
                            let icon_enabled =
                                player.as_ref().is_some_and(|p| p.cursor_item_icon_enabled);
                            // The slot-surface hit tests over disjoint fields
                            // (the destructure holds `renderer`/`lookups` —
                            // a whole-`self` method call would conflict).
                            let over_ui_slot = crate::app::clicks::slot_at_mouse_fields(
                                self.mouse_pos,
                                self.inventory_open,
                                &self.hotbar_scales,
                            )
                            .is_some()
                                || crate::app::clicks::coin_ammo_at_mouse_fields(
                                    self.mouse_pos,
                                    self.inventory_open,
                                    self.bestiary_open,
                                    lookups.ui_back.map_or(52.0, |b| b.width as f32),
                                )
                                .is_some()
                                || self.wires_radial.hovering;
                            let inp = ui::wires_ui::RadialInputs {
                                mouse: self.mouse_pos,
                                screen,
                                held_kind: held.kind,
                                held_count: held.count,
                                dead: player.as_ref().is_some_and(|p| p.dead),
                                mouse_item: self.mouse_item.count > 0,
                                icon_enabled,
                                chat_open: self.chat.open,
                                mouse_interface: over_ui_slot,
                                options_or_fancy: self.ingame_options.open
                                    || self.bestiary_open
                                    || self.achievements_open,
                                right_press: self.place_down && !self.place_prev_down,
                                left_press: self.mine_down && !self.mine_prev_down,
                                no_throw: self.no_throw != 0,
                                talking: self.talk_npc.is_some(),
                            };
                            self.wires_radial.update(tool_mode_ui, allow_actuators, &inp);
                            self.wires_radial.draw(
                                renderer,
                                lookups,
                                &mut self.wire_tool_mode,
                                allow_actuators,
                                &inp,
                            );
                        }
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
                                &self.equip,
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
                        // `Recipe.UpdateRecipeList` (`Recipe.cs:230-259`) +
                        // its refocus tail (`:232-233`, `:257`,
                        // `TryRefocusingRecipe` `:477-495`): the RAW
                        // availableRecipe list rebuilds and the focus
                        // follows the recipe it pointed at. The C# runs it
                        // from TWO draw sites — the modern grid
                        // (`NewCraftingUI.Draw` → `UpdateContents` `:585`,
                        // while `NewCraftingUI.Visible`) AND the classic
                        // pip-list arm (`Main.cs:41712`, while the inventory
                        // is open and the grid is NOT visible). The port
                        // collapses both onto the one grid, so the rebuild
                        // runs whenever the inventory is open — the
                        // closed-panel deviation is retired: the snapshot
                        // now tracks every inventory change while the panel
                        // is closed-but-inventory-open, exactly the C#'s
                        // `:41712` arm (research/126 closeout). The classic
                        // pip-list DRAW (`CraftingUI.DrawRecipesList`,
                        // `Main.cs:41715`) stays unported — the modern grid
                        // replaces it (ANALYSIS "Crafting panel").
                        let craft_available = craft_raw_pre;
                        if self.inventory_open {
                            let old_recipe = self
                                .craft_last_available
                                .get(self.focus_recipe.max(0) as usize)
                                .copied()
                                .unwrap_or(0);
                            self.focus_recipe = craft::try_refocusing_recipe(
                                &craft_available,
                                old_recipe,
                                self.focus_recipe,
                            );
                            self.craft_last_available = craft_available.clone();
                        }
                        // `NewCraftingUI.UpdateContents` (`:596-611`): the
                        // grid shows only the recipes whose product passes
                        // the filterer (`_filteredRecipes`); the raw list
                        // above stays the focus/available index space.
                        let craft_shown = self.craft_filters.shown(craft_available.clone());
                        // The unified open-container state (`Player.chest`,
                        // `Player.cs:2876`/`:2878`/:2880) — the panel reads the
                        // player's fields now that one field covers every kind.
                        let panel_chest = player
                            .as_ref()
                            .map_or(crate::bank::CHEST_CLOSED, |p| p.chest);
                        let (panel_chest_x, panel_chest_y) =
                            player.as_ref().map_or((0, 0), |p| (p.chest_x, p.chest_y));
                        // S1-W4-g81 residuals — `DrawInterface_31_BuilderAccToggles`
                        // (`Main.cs:45477-45480` → `DrawBuilderAccToggles(10, 77)`,
                        // `:47353-47590`): the wire-channel cycle column + the
                        // ruler/actuator/paint on/off rows + the block-replacer/
                        // torch-swap side icons. The layer rides between the
                        // hotbar (30) and the mouse text (33) — the port draws it
                        // just before `draw_hud` (which carries 25/27/30/33+);
                        // the column's own `playerInventory` gate is inside.
                        // The click's sound-12 (`Main.cs:47413` etc.) plays here.
                        if let Some(p) = player.as_mut() {
                            let toggled = ui::builder_toggles::draw_builder_acc_toggles(
                                renderer,
                                lookups,
                                p,
                                &ui::builder_toggles::ToggleInputs {
                                    mouse: self.mouse_pos,
                                    screen,
                                    left_press: self.mine_down && !self.mine_prev_down,
                                    inventory_open: self.inventory_open,
                                    pulse: ui::UiPulse {
                                        text: self.mouse_text_color as f32 / 255.0,
                                        cursor_scale: self.cursor.scale(),
                                    },
                                },
                            );
                            if toggled == Some(true) {
                                self.audio.play("Menu_Tick");
                            }
                        // `NewCraftingUI.OnActivate` (`:904`):
                        // `_selectedRecipeIndex = Main.numAvailableRecipes < 0
                        // ? null : Main.availableRecipe[Main.focusRecipe]` —
                        // the raw recipe index the grid's selection overlay
                        // marks (`UpdateContents` `:612-622` clears it once
                        // the focused recipe leaves the available list).
                        // `focus_recipe` is a RAW availableRecipe position
                        // (W4-f23 deviation fix — the click path stores the
                        // position, `HandleCraftSlot` `:510`).
                        let craft_selected = (self.focus_recipe >= 0)
                            .then(|| craft_available.get(self.focus_recipe as usize).copied())
                            .flatten();
                        // The craft grid's scrollbar frame, in
                        // `UIScrollbar.DrawSelf`'s order (`:103-124`):
                        // `SetView`'s clamp rides every rebuild
                        // (`UIList.UpdateScrollbar` `:130-137`), the drag
                        // arm re-derives the position from the pointer
                        // (`:109-112`), then the handle hover latch with
                        // its enter-edge sound 12 (`:115-120`). The latch
                        // and the drag only run while the bar can scroll
                        // (`AutoHide`'s `CanScroll` gate, `:105`).
                        if self.crafting_open {
                            let cl = ui::crafting::layout(
                                screen,
                                ui::crafting::title_width_for(lookups),
                                false,
                            );
                            let content_h =
                                ui::crafting::grid_content_height(&cl, craft_shown.len());
                            self.craft_scroll = ui::char_select::clamp_scroll(
                                self.craft_scroll,
                                cl.list[3],
                                content_h,
                            );
                            if ui::char_select::scrollbar_attaches(cl.list[3], content_h) {
                                if let Some(offset) = self.craft_scroll_drag {
                                    self.craft_scroll = ui::crafting::scrollbar_drag(
                                        &cl,
                                        craft_shown.len(),
                                        offset,
                                        self.mouse_pos.y,
                                    );
                                }
                                let handle = ui::panel::scrollbar_handle_at(
                                    cl.scrollbar,
                                    cl.list[3],
                                    content_h,
                                    self.craft_scroll,
                                );
                                let hovering = ui::in_rect(self.mouse_pos, handle);
                                if hovering && !self.craft_scroll_hover {
                                    self.audio.play("Menu_Tick"); // `:119`
                                }
                                self.craft_scroll_hover = hovering || self.craft_scroll_drag.is_some();
                            } else {
                                self.craft_scroll_hover = false;
                            }
                        }
                        // The nearby-chests toggle's hover cue
                        // (`CraftingUI.DrawCraftFromNearbyChestsToggle`,
                        // `:354`'s `DoStatefulTickSound(ref
                        // Main.nearbyCraftingMouseOver, flag)`): the latch
                        // follows the ±15 px hover square, the enter edge
                        // plays sound 12 (`Main.cs:3244`). Runs only while
                        // the panel draws — the C# runs it inside the toggle
                        // draw; with the panel closed the latch holds its
                        // last value until `Main.ResetInventoryState`
                        // (`:3225`) clears it on the inventory open/close
                        // (the port clears it in the draw's else arm below,
                        // the same net effect: no stale enter-edge cue on
                        // reopen). The toggle is hidden in the guide menu
                        // (`NewCraftingUI.cs:492-494`), which the port never
                        // enters (`guide_mode` stays false).
                        if self.crafting_open {
                            let nearby_hover = ui::crafting::nearby_toggle_hit(
                                screen,
                                self.mouse_pos,
                            );
                            if ui::crafting::stateful_tick_sound(
                                &mut self.nearby_crafting_mouse_over,
                                nearby_hover,
                            ) {
                                self.audio.play("Menu_Tick");
                            }
                        } else {
                            self.nearby_crafting_mouse_over = false;
                        }
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
                            &craft_shown,
                            craft_selected,
                            // `Main.inventoryBack` when the crafting grid
                            // draws: Interface Logic 2 pulses `invAlpha`
                            // 180..240 by 0.2/tick (`Main.cs:45697-45708`,
                            // ticked in the per-frame animation block above)
                            // — the byte colour, no more midpoint stand-in.
                            self.inv_pulse.inventory_back(),
                            self.craft_scroll,
                            self.craft_scroll_hover,
                            settings.craft_from_nearby_chests,
                            ui::UiPulse {
                                // `Main.mouseTextColor / 255` (`Main.cs:20355`).
                                text: self.mouse_text_color as f32 / 255.0,
                                cursor_scale: self.cursor.scale(),
                                // W4-g35: the item-slot glow clocks
                                // (`ItemSlot.cs:2891-2929`).
                                misc_counter: player
                                    .as_ref()
                                    .map_or(0, |p| p.misc_counter),
                                disco: self.disco,
                                global_time: anim_time % 3600.0,
                            },
                            m_h,
                            self.map_style,
                            self.bestiary_open,
                            self.bestiary_selected,
                            &self.bestiary_ui,
                            &self.bestiary,
                            &mut self.chat,
                            &self.emotes,
                            // BUG-11: the in-game options window's state
                            // (`Main.ingameOptionsWindow`) — the Settings button
                            // and the panel draw over the open inventory.
                            Some((settings, &self.ingame_options)),
                            self.shop_npc.map(|_| {
                                (
                                    self.shop_stock.as_slice(),
                                    // `Main.cs:41793` — the chrome reads the
                                    // LATCHED `currentShoppingSettings.
                                    // PriceAdjustment`, not the shop-open
                                    // `shop_mood` capture (the buy-lane
                                    // repoint, research/240).
                                    self.shopping_settings.price_adjustment,
                                    self.discount_available,
                                )
                            }),
                            // W4-f28 — the shop price row's session inputs:
                            // `ItemSlot.ShiftInUse` (the pressed-codes set,
                            // the clicks.rs:41 shape) + the sellback memo.
                            self.shop_npc.map(|_| ui::ShopTooltipState {
                                shift_in_use: self.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
                                    || self.keys.contains(&winit::keyboard::KeyCode::ShiftRight),
                                sellback: &self.shop_sellback,
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
                                name: &chest_stored_name,
                                max_items: chest_max_items,
                                edit_text: &chest_edit_text,
                                blinker: &mut self.text_blinker,
                                scroll_row: &mut self.chest_scroll_row,
                                button_scales: &mut self.chest_button_scales,
                                button_hovered: &mut self.chest_button_hovered,
                                // W4-g56 — the world-chest glow (context 3).
                                glow: Some(&self.slot_glow),
                                audio: &mut self.audio,
                                coin_slot: &mut self.coin_slot,
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
                                            // `Main.cs:41591` — the LATCHED
                                            // `currentShoppingSettings.
                                            // PriceAdjustment`, the same
                                            // value the charge reads
                                            // (`reforge_click`).
                                            self.shopping_settings.price_adjustment,
                                            self.discount_available,
                                        )
                                    })
                                    .unwrap_or(0);
                                (
                                    &self.reforge_slot,
                                    cost,
                                    self.reforge_scale,
                                    self.mouse_reforge,
                                )
                            }),
                            // W4-d122: the savings draws' coin machine
                            // (`CoinSlot.Savings[4]` + the Custom dict).
                            &mut self.coin_slot,
                            // `ItemSlot._customCurrencyForSavings` — the
                            // medal-savings latch (`ItemSlot.cs:308`).
                            &mut self.custom_currency_for_savings,
                            player.as_ref().and_then(|p| {
                                crate::bank::bank_for_chest(p.chest)
                                    .map(|bi| (&p.banks[bi][..], p.chest))
                            }),
                            // W4-g56 — the void vault's favorite bits ride
                            // the draw (context 32's `Inventory_Back19`).
                            player.as_ref().and_then(|p| {
                                (p.chest == -5).then_some(&p.vault_favorites)
                            }),
                            // W4-g56 — the chest-glow state (contexts 3/4/32).
                            Some(&self.slot_glow),
                            // `Main.DrawPVPIcons`'s state (`Main.cs:42654`): the
                            // net-mode / team-based-spawns gates, the local
                            // player's `hostile` + `team` and `Main.teamCooldown`.
                            &pvp_icon_state,
                            // `Player.hideVisibleAccessory` (`Player.cs:1708`): the
                            // accessory visible/hidden ticks in `DrawInventory`
                            // (`Main.cs:41425-41460`).
                            &hide_visible_accessory,
                            // The misc-page ticks (`Main.cs:41226-41283`).
                            &hide_misc,
                            unlocked_super_cart,
                            enabled_super_cart,
                            // The quick-stack/sort buttons (`Main.cs:41833-41897`).
                            self.settings.stack_to_chests_mode,
                            &mut self.all_chest_stack_hover,
                            &mut self.inventory_sort_hover,
                            // The hotbar-lock + builder-accessory icons
                            // (`Main.cs:42479-42596`, W4-g21 batch 5).
                            hb_locked,
                            unlocked_biome_torches,
                            using_biome_torches,
                            tile_replacement_enabled,
                            info_acc_mech_show_wires,
                            ruler_line,
                            ruler_grid,
                            auto_actuator,
                            auto_paint,
                            &mut self.hotbar_lock_hover,
                            &mut self.toggle_acc_hover,
                            // The buff bar (`Main.DrawInterface_Resources_Buffs`).
                            &mut self.buff_alpha,
                            &mut self.buff_removals,
                            buff_right_click,
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
                            // The town-chatter UI (S1-E-10b) — the panel
                            // draws while the talk latch holds; the sign
                            // half (W4-f22) rides the same panel.
                            self.talk_npc,
                            &self.npc_chat_text,
                            // `Main.cursorOverride` — the shift-hover sell/
                            // trash/transfer cursor (W4-e19).
                            self.cursor_override,
                            // W4-g79 — the cursor interact-icon pass's world
                            // (the biome hold-style conversions).
                            world,
                            // S1-W4-g81 — the 10-frame cache linger pair
                            // (the batch-H keep-one compose).
                            (self.item_icon_cache_kind, self.item_icon_cache_time),
                            // `GUIChatDraw`'s sign gate + the panel's own
                            // blinker (`Main.cs:46713`, `NPCChatPanel.cs:
                            // 254-261`).
                            self.player.as_ref().is_some_and(|p| p.sign >= 0),
                            self.sign_editing(),
                            chest_editing,
                            &mut self.sign_text_blinker,
                            // `Main.signBubble`/`signX`/`signY` — the
                            // placed-sign bubble anchor (W4-f22).
                            self.sign_bubble.then_some((self.sign_x, self.sign_y)),
                            // `Main.signHover`'s record text — the hover
                            // tooltip's preview (W4-f22). Reads the
                            // destructured `world` binding (the `self.world`
                            // field is already borrowed by the draw site).
                            (self.sign_hover >= 0)
                                .then(|| {
                                    world
                                        .signs
                                        .get(self.sign_hover as usize)
                                        .map(|s| s.text.as_str())
                                })
                                .flatten(),
                            // The Journey powers menu (W4-c24) —
                            // `CreativeUI.Draw`'s gate (`CreativeUI.cs:98-102`):
                            // the difficulty-3 player, unblocked.
                            (self.active_player_difficulty() == 3
                                && !ui::journey::blocked(
                                    self.talk_npc.is_some(),
                                    panel_chest >= 0,
                                    self.crafting_open,
                                ))
                                .then(|| {
                                    (
                                        &self.journey,
                                        &self.research,
                                        &self.creative_powers,
                                        (
                                            self.wind.speed_target,
                                            self.rain.cloud_alpha(),
                                            self.rain.max_raining(),
                                        ),
                                    )
                                }),
                            // `Player.CurrentLoadoutIndex` (`Player.cs:3207`) +
                            // the loadout-change flash (`ItemSlot.cs:2786-2791`)
                            // + the hovered-button latch (`Main.cs:41990`).
                            player.as_ref().map_or(0, |p| p.current_loadout_index),
                            ui::inventory::loadout_flash(
                                self.loadout_change_time,
                                (self.anim_time * 60.0) % 216000.0,
                                settings.flashy_effects_interface,
                            ),
                            &mut self.loadout_mouse_over,
                            self.equip_share_flags,
                            // W4-f28 — the tooltip's world/session gates.
                            ui::tooltip_lines2::TooltipWorld {
                                expert_mode: player
                                    .as_ref()
                                    .map_or(false, |p| p.buffs.expert_mode),
                                downed_plant_boss: world.downed_plant_boss,
                                // `DD2Event.DownedInvasionAnyDifficulty`
                                // (`DD2Event.cs:89-99`) = T1 ‖ T2 ‖ T3 —
                                // `Player.cs:24450-24453` latches it into
                                // the .plr flag; NOT the `dd2_won` sentinel
                                // (research/128 §4.2: a T1 win already
                                // satisfies the getter).
                                downed_dd2_any: world.downed_dd2_any(),
                                tenth_anniversary: world
                                    .seeds
                                    .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY),
                                get_good_world: world
                                    .seeds
                                    .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
                                // `SpecialSeedFeatures.Mechdusa`
                                // (`SpecialSeedFeatures.cs:119-129`).
                                mechdusa: world.seeds.contains(tr_core::seeds::SeedFlags::REMIX)
                                    && world
                                        .seeds
                                        .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
                                // `Main.npcShop > 0` — filled at the draw site
                                // from the shop panel's presence.
                                npc_shop: false,
                                master_color: self.master_color,
                                disco: self.disco,
                            },
                            // `ReportHappiness.Condition`
                            // (`NPCInteractions.cs:291-298`).
                            !crate::happiness::can_show_homeless_text(
                                self.talk_npc.and_then(|i| self.npcs.get(i)),
                            ) && !self.shopping_settings.happiness_report.is_empty(),
                            // `CoinSlot.Savings` — the shop savings pile's
                            // coin cells (`ItemSlot.DrawSavings`,
                            // `Main.cs:41791`, W4-g58 D5).
                            &mut self.coin_slot,
                            // The chat portrait (W4-g58 retire): the hop
                            // clock's pre-increment value (`Main.cs:40335`
                            // reads THEN increments — the tick below), the
                            // `DialoguePortraitPreference` setting and the
                            // Zoologist lycanthrope predicate.
                            self.npc_chat_portrait_frame_counter,
                            self.settings.portrait_preference,
                            crate::happiness::should_bestiary_girl_be_lycantrope(
                                self.world.blood_moon,
                                self.world.day_time,
                                self.world.moon_phase,
                            ),
                        );
                        // S1-W4-g81 residuals — `Main.ItemIconCacheVerification`
                        // (`Main.cs:45062` inside `DrawInterface_40`, the C#'s
                        // only call site): the linger's per-draw decay, after
                        // the icon pass read it. Inlined over disjoint fields —
                        // the destructure holds `renderer`, a whole-`self`
                        // method call would conflict (the `:46522-46540` body
                        // verbatim).
                            // W4-g12: the savings pile's coin machine.
                            &mut self.coin_slot,
                            // W4-f30 — the Dryad's Purify button gate.
                            crate::dryad_stardew::can_dryad_play_stardew_animation(
                                self.inventory.get(self.selected).map_or(0, |s| s.kind),
                                self.talk_npc.and_then(|i| self.npcs.get(i)),
                            ),
                            // `Main.item[]` — the world-hover item walk.
                            &self.items,
                            // `Main.hideUI` — the F11 layer-8 stop.
                            self.hide_ui,
                            self.cursor_item_icon_enabled,
                            (self.item_icon_cache_kind, self.item_icon_cache_time),
                            // W4-g86 research/244 phase 1 — the layer-40 ID
                            // override state (`Player.cs:1852-1858`).
                            (
                                self.cursor_item_icon_id,
                                self.cursor_item_icon_push,
                                self.cursor_item_icon_reversed,
                                self.cursor_item_icon_text.as_str(),
                            ),
                            // The layer-40 head gate (`Main.cs:45039`): the
                            // NPC-hover bubble pick (`Main.HoveringOverAnNPC`
                            // via the port's `npc_chat::bubble_npc` stand-in)
                            // OR the over-UI state — the minimap button
                            // (`MinimapFrame.cs:164`), the closed hotbar
                            // hover (`ItemSlot.Handle`'s `mouseInterface`,
                            // the same read `update_editing`'s HUD gate
                            // uses). The open-panel slot hover has no
                            // pre-draw resolution on the port (the panel
                            // hit-tests inside `inventory::draw`); recorded
                            // deviation — the C# suppresses the icon there
                            // too.
                            {
                                let mouse_world = Vec2::new(
                                    (self.mouse_pos.x - screen.x * 0.5) / *zoom + camera.x,
                                    (self.mouse_pos.y - screen.y * 0.5) / *zoom + camera.y,
                                );
                                ui::npc_chat::bubble_npc(&npcs[..], player.as_ref(), mouse_world)
                                    .is_some()
                                    || self.minimap_hover.is_some()
                                    || (!self.inventory_open
                                        && ui::tooltip::hovered_slot(
                                            self.mouse_pos,
                                            false,
                                            *selected,
                                            &self.hotbar_scales,
                                            inventory,
                                        )
                                        .is_some())
                            },
                        );
                        // S1-W4-g81 residuals — `Main.ItemIconCacheVerification`
                        // (`Main.cs:45062` inside `DrawInterface_40`): the
                        // linger's per-draw decay, after the icon pass read it.
                        // W4-g86 carry compile-fix: INLINED from
                        // `App::item_icon_cache_verification` — a whole-`self`
                        // method call cannot run inside the `let App {…} = self`
                        // destructure's live field borrows (renderer/player/
                        // inventory stay live across the interface pass); the
                        // same maths rides the disjoint fields directly. The
                        // method stays for the cache witnesses.
                        if self.item_icon_cache_time > 0 {
                            let d = (self.item_icon_cache_pos - self.mouse_pos).length();
                            if d != 0.0 {
                                self.item_icon_cache_time -= 1;
                            }
                            if d > 4.0 {
                                self.item_icon_cache_time = 0;
                            }
                            if self.item_icon_cache_kind != inventory[*selected].kind {
                                self.item_icon_cache_time = 0;
                            }
                        }
                        // `CoinSlot.UpdateSlotAnims()` (`Main.cs:44936`, S1-H-9):
                        // the coin cells' animation clocks step per frame after
                        // the interface layers draw.
                        self.coin_slot.update_slot_anims();
                        // `Main.DrawNPCPortrait` (`Main.cs:40335`): while the
                        // chat panel draws, the hop counter reads THEN
                        // increments (`npcChatPortraitFrameCounter++`). The
                        // Detailed-arm DRAW now consumes the pre-increment
                        // value inside `draw_hud` (`ui::chat_portrait`,
                        // W4-g58 retire); the tick stays on the same gate —
                        // the panel draws iff `talk_npc` holds, and every
                        // ported town kind has an `NPCPortraits` row
                        // (`NPCID.cs:5894+`), so the provider/texture
                        // null-gates never bind (the Disabled/CloseUp/
                        // FullBodyRetro preferences still tick — the C#
                        // increments BEFORE the mode branch).
                        if self.talk_npc.is_some() {
                            self.npc_chat_portrait_frame_counter =
                                self.npc_chat_portrait_frame_counter.wrapping_add(1);
                        }
                        // `Main.DrawDeathText`'s overlay (`Main.cs:45324-45341`):
                        // the death broadcast + the softcore "Dropped coins"
                        // line while the respawn timer runs.
                        if let Some(p) = player.as_ref() {
                            // Layer 35 (Death Text) sits after the layer-8
                            // hide check — F11 hides it too.
                            if p.hp <= 0 && !self.hide_ui {
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
                                    // `GetDeathAlpha(Color.Transparent)`
                                    // (`Main.cs:45332`): the death fade rides
                                    // `Player.immuneAlpha` (UpdateDead's +2/tick).
                                    p.immune_alpha,
                                );
                            }
                        }
                        // `Main.DrawInvasionProgress` (`Main.cs:47696-47840`):
                        // the bar draws while the fade alpha holds (the
                        // display-timer decay rides the world tick). Layer 15
                        // sits after the layer-8 hide check.
                        if !self.hide_ui {
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
                        }
                    } else if let Some(menu) = menu.as_mut() {
                        menu.draw(renderer, lookups, screen, dt);
                        // W4-g23 — the chrome-pass cue drain: `DrawMenu`'s
                        // draw-time slider-block `PlaySound`s (the colour-
                        // focus tick `Main.cs:51276`, the volume-bar hover
                        // tick `:51575`) queue on the menu because the draw
                        // half cannot borrow `Audio`; they play here, in
                        // C# statement order, right after the draw. Union
                        // contract: the ACTIVATION cues (row clicks/keys —
                        // `:52059`/`:51952` included) ride g22's
                        // `pending_cue` and drain in the event loop
                        // (`app/loop.rs`) BEFORE this draw; this queue is
                        // chrome-only, so the two never double-play.
                        for id in menu.take_pending_ticks() {
                            self.audio.play(match id {
                                10 => "Menu_Open",
                                11 => "Menu_Close",
                                _ => "Menu_Tick", // 12
                            });
                        }
                    }
                    // `UIAchievementsMenu` (`Main.cs:48931`'s menuMode 888 +
                    // `MenuUI.SetState(Main.AchievementsMenu)`): the state draws
                    // over whatever the frame rendered — the menu's world
                    // backdrop here, the world itself in the fancy-UI arm. Its
                    // `OnActivate` picks the container's Top by `Main.gameMenu`.
                    // Layer 12 (Fancy UI) sits after the layer-8 hide check.
                    if self.achievements_open && !self.hide_ui {
                        ui::achievements::draw_screen(
                            renderer,
                            lookups,
                            screen,
                            &self.achievements,
                            self.achievements_scroll,
                            &self.achievements_categories,
                            self.mouse_pos,
                            menu.is_some(),
                            self.mouse_text_color,
                            self.achievements_scroll_drag.is_some(),
                        );
                    }
                    // `UIDebugCommandsList` (W4-d85 row 7): the same overlay
                    // treatment — the state draws over whatever the frame
                    // rendered (`IngameFancyUI.OpenUIState`).
                    if self.debug_commands_open {
                        let is_client = self.is_client();
                        ui::debug_commands::draw_screen(
                            renderer,
                            lookups,
                            screen,
                            &self.debug_commands_search,
                            self.debug_commands_scroll,
                            self.mouse_pos,
                            is_client,
                        );
                    }
                    // `UIDebugLoadoutsList` (W4-d85 row 8) — the twin overlay.
                    if self.debug_loadouts_open {
                        ui::debug_loadouts::draw_screen(
                            renderer,
                            lookups,
                            screen,
                            &self.debug_loadouts,
                            &self.debug_loadouts_search,
                            self.debug_loadouts_scroll,
                            self.mouse_pos,
                        );
                    }
                    // `Main.IssueReporterIndicator.Draw` (`Main.cs:51886`,
                    // `DrawMenu`'s tail right before `DrawCursor` — AFTER the
                    // 888 `MenuUI.Draw` at `:51063`, the legacy rows and the
                    // fancy-UI overlays, so it covers both menu regimes):
                    // the issue-report button slides in from the right edge
                    // while a report needs attention (`IssueReportsIndicator.
                    // cs:45-79`). The percent ticks once per DRAWN frame here
                    // — the C# advances it inside Draw, not Update. Never
                    // shows in the port (no producer), the exact vanilla
                    // state on a pack-free, no-Steam install.
                    if (menu.is_some() || front.is_some()) && issue_indicator.tick_draw() {
                        ui::reports::draw_indicator(
                            renderer,
                            lookups,
                            screen,
                            issue_indicator.display_up_percent,
                            self.mouse_pos,
                            ui::UiPulse {
                                text: self.mouse_text_color as f32 / 255.0,
                                cursor_scale: self.cursor.scale(),
                            },
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
                    if menu.is_none() && front.is_none() && !crate::env::no_hud() && !self.hide_ui {
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
                    // W4-f46 salvage (research/84 §3): the section-draw
                    // driver (`Main.cs:61863-61899`, the `DoDraw` tail
                    // after `MapUpdateQueue.Update` `:61862`): the 5
                    // ms-budgeted spiral walk paints the map texture one
                    // 200x150 section at a time (`GetNextMapDraw` →
                    // `DrawToMap_Section`), unlocking each while
                    // `DebugOptions.unlockMap != 0`, then the `updateMap`
                    // / `refreshMap` flag lifecycle. The `!gameMenu &&
                    // !generatingWorld` gate (`:61865`) is the in-world
                    // gate here; the `!Main.loadMap` gate (`:61863`) rides
                    // inside `map_reveal_draw`. `player_pos` is
                    // `Main.player[myPlayer].position` in world PIXELS
                    // (`:61871`); `[0, 0]` without a player
                    // (`TR_NOPLAYER` — the C# always has one).
                    if menu.is_none() && front.is_none() {
                        let player_pos =
                            player.as_ref().map_or([0.0, 0.0], |p| [p.x, p.y]);
                        Self::map_reveal_draw(
                            map,
                            &mut self.section_manager,
                            &mut self.debug_unlock_map,
                            world,
                            player_pos,
                            settings.map_enabled,
                            self.game_paused,
                            renderer,
                            gpu,
                        );
                    }
                    // Task 6b: upload the revealed tiles into the world-sized map texture
                    // and clamp/recentre the fullscreen map. Vanilla centres the recentre
                    // and the minimap on the view centre (`Main.cs:55420-55421`,
                    // `55537-55538`): `(screenPosition + screenSize/2)/16` — `view_center`
                    // is computed above, where the minimap branch consumes it.
                    if settings.map_enabled {
                        if let Some(m) = map.as_mut() {
                            // `Main.cs:55363`: a held left press cancels the
                            // offscreen-pylon pan (sampled before `prepare`
                            // runs the lerp).
                            m.pan_cancel_press = self.mine_down && self.map_left_press;
                            // `Main.cs:55550-55566`: the style-2 overlay's
                            // draw-side clamps ride the same draw half as
                            // the minimap's — `world_zoom` IS `RenderZoom`
                            // (`:61999`, the `ForcedMinimumZoom *
                            // Clamp(GameZoomTarget)` product computed above).
                            m.clamp_overlay_axes(screen.x, world.width, world_zoom);
                            m.prepare(renderer, gpu, world, screen, view_center, settings.map_enabled);
                            m.prepare(renderer, gpu, world, screen, view_center);
                        }
                    }
                    if let Some(map) = map.as_ref() {
                        if !crate::env::no_hud()
                            && menu.is_none()
                            && front.is_none()
                            && settings.map_enabled
                            // `Main.DrawMap`'s second gate (`Main.cs:55283`):
                            // `!Main.mapReady` returns before the fullscreen
                            // branch too — no backdrop/frame until the first
                            // `DrawToMap` completes (`MapRenderer.cs:196`).
                            && map.map_ready
                            // Layer 16 is after the layer-8 hide check; the
                            // fullscreen map draws on its own branch above
                            // (F11 never reaches it — `Main.cs:62147-62181`).
                            && !self.hide_ui
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
                                // `Main.cs:55782-55913`: the hovered map
                                // cell's name (chests/dressers/legend) is
                                // the hover text's base; the icon layers'
                                // `ref text` writes (`:55922-55923`) win.
                                let tile_hover = crate::map::fullscreen_hover_name(
                                    world,
                                    map,
                                    &self.world_chests,
                                    self.mouse_pos,
                                    screen,
                                );
                                // `Main.cs:55915-55921`: `num77` — the
                                // `(scale*0.25*2+1)/3` formula is dead code
                                // (the `:55920-55921` unconditional writes
                                // overwrite it); the head scale is
                                // `Main.UIScale`.
                                let num77 = self.settings.applied_ui_scale(screen.x, screen.y);
                                // `Main.cs:55922-55923`: the revenge markers
                                // + the misc icons, BEFORE the NPC heads.
                                // The fullscreen pair is `Vector2.Zero` /
                                // `(num - 10*scale, num2 - 10*scale)` — the
                                // same offset `draw_full` returned.
                                let revenge_hover = crate::map_player_icons::draw_revenge_map_icons(
                                    renderer,
                                    lookups,
                                    &self.revenge_markers,
                                    [0.0, 0.0],
                                    map_offset,
                                    None,
                                    map.scale,
                                    num77,
                                    self.mouse_pos,
                                );
                                let golf_ball = self.golf_state.last_hit_golf_ball().and_then(|stored| {
                                    let live = self.projectiles.slots().get(stored.who_am_i).map(|q| {
                                        let (cx, cy) = q.center();
                                        tr_core::golf_state::GolfBall {
                                            who_am_i: stored.who_am_i,
                                            kind: q.kind as i32,
                                            owner: q.owner as i32,
                                            active: q.active,
                                            position: q.pos,
                                            center: glam::Vec2::new(cx, cy),
                                            ai1: q.ai[1],
                                            local_ai1: q.local_ai[1],
                                        }
                                    });
                                    self.golf_state
                                        .last_hit_ball_for_draw(live.as_ref(), self.local_id as i32)
                                });
                                let misc_hover = crate::map_player_icons::draw_misc_map_icons(
                                    renderer,
                                    lookups,
                                    &crate::map_player_icons::MiscIconInputs {
                                        player: player.as_ref(),
                                        golf_ball,
                                        mouse: self.mouse_pos,
                                    },
                                    [0.0, 0.0],
                                    map_offset,
                                    None,
                                    map.scale,
                                    num77,
                                );
                                // `Main.cs:55926`: `DrawNPCMapIcons` — the
                                // boss/town head pass, AFTER the revenge/
                                // misc icons and BEFORE the player heads;
                                // its hover text overrides the tile hover.
                                let head_hover = crate::map_npc_icons::draw_npc_map_icons_fullscreen(
                                    renderer,
                                    lookups,
                                    npcs,
                                    map::map_origin(map.pos, map.scale, screen),
                                    &crate::map_npc_icons::NpcIconView {
                                        map_position: [0.0, 0.0],
                                        sub_offset: [0.0, 0.0],
                                        minimap_rect: None,
                                        map_scale: map.scale,
                                        head_scale: num77,
                                        alpha: 1.0,
                                        mouse: self.mouse_pos,
                                        global_time: anim_time % 3600.0,
                                    },
                                );
                                let (hover, click, pylon_hovered) = crate::map_layers::map_layers_draw(
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
                                        map_fullscreen: true,
                                        pylon_tile: map.pylon_tile,
                                        screen,
                                        left_press: self.map_left_press,
                                    },
                                    [0.0, 0.0],
                                    map_offset,
                                    None,
                                    map.scale,
                                    map_draw_scale,
                                    1.0,
                                );
                                pylon_click = click;
                                // `TeleportPylonsMapLayer.cs:81`: a pylon
                                // marker hover claims `Main.cancelWormHole`.
                                if pylon_hovered {
                                    self.map.as_mut().unwrap().cancel_wormhole = true;
                                }
                                // `Main.cs:55931-55976`: the 255-player head
                                // loop — the local head plus the remote
                                // peers, each with the name hover; then the
                                // death markers (`:55977-55983`).
                                let net_mode = match self.net {
                                    crate::app::NetRole::Host(_) => 2,
                                    crate::app::NetRole::Client(_) => 1,
                                    crate::app::NetRole::Off => 0,
                                };
                                let local_team = player.as_ref().map_or(0, |p| p.team);
                                let local_hostile = player.as_ref().is_some_and(|p| p.hostile);
                                let fs_origin = map::map_origin(map.pos, map.scale, screen);
                                let mut pin_hover: Option<String> = None;
                                if let Some(p) = player.as_ref() {
                                    if !p.dead {
                                        let (cx, cy) = p.center();
                                        // `Main.cs:55938`: `position.Y + gfxOffY + height/2`.
                                        let center_px = [cx, cy + p.gfx_off_y];
                                        let icon_pos = map::head_icon_pos(
                                            center_px, map.pos, map.scale, screen,
                                        );
                                        let border = crate::map_player_icons::player_head_borders_color(
                                            p.dead, p.team, p.hostile, true, local_team, net_mode,
                                        );
                                        draw_player_map_icon(
                                            // Fullscreen `b = 255` (`Main.cs:55291`).
                                            renderer, player_assets, p, icon_pos, num77, border, 1.0,
                                            renderer, player_assets, p, icon_pos, num77, border,
                                        );
                                        // `:55945-55959`: the local head's
                                        // hover writes the name too (the
                                        // wormhole branch is MP-only and
                                        // unported — HasUnityPotion).
                                        let hr = crate::map_player_icons::head_hover_rect(icon_pos, num77);
                                        if crate::map_player_icons::hover_inclusive(hr, self.mouse_pos) {
                                            pin_hover = Some(self.player_name.clone());
                                        }
                                    }
                                }
                                for rp in remote_players.values() {
                                    if !crate::map_player_icons::player_map_icon_can_be_seen(
                                        false, false, 0, local_hostile, local_team,
                                    ) {
                                        continue;
                                    }
                                    let pp = crate::map_player_icons::remote_pin_player(world, rp);
                                    let center_px = [
                                        pp.x + pp.width * 0.5,
                                        pp.y + pp.gfx_off_y + pp.height * 0.5,
                                    ];
                                    let pin = crate::map_player_icons::fullscreen_pin_pos(
                                        center_px, fs_origin, map.scale,
                                    );
                                    let border = crate::map_player_icons::player_head_borders_color(
                                        false, 0, false, false, local_team, net_mode,
                                    );
                                    draw_player_map_icon(renderer, player_assets, &pp, pin, num77, border, 1.0);
                                    draw_player_map_icon(renderer, player_assets, &pp, pin, num77, border);
                                    let hr = crate::map_player_icons::head_hover_rect(pin, num77);
                                    if crate::map_player_icons::hover_inclusive(hr, self.mouse_pos) {
                                        pin_hover = Some(rp.name.clone());
                                    }
                                }
                                // `:55977-55983`: the death markers — the
                                // local player's (the peers carry no
                                // lastDeath sync).
                                let mut death_hover: Option<String> = None;
                                let mut death_hovered = false;
                                if let Some(p) = player.as_ref() {
                                    if p.show_last_death {
                                        if let Some(d) = p.last_death_position {
                                            let dp = crate::map_player_icons::death_marker_pos(
                                                [d.0, d.1],
                                                [0.0, 0.0],
                                                fs_origin,
                                                map.scale,
                                                [10.0 * map.scale, 10.0 * map.scale],
                                            );
                                            if crate::map_player_icons::draw_death_marker(
                                                renderer, lookups, dp, num77, 1.0, self.mouse_pos,
                                                // `DrawPlayerDeathMarker`
                                                // (`:56455-56462`): NO -4
                                                // shift (W4-f29 F5).
                                                false,
                                            ) {
                                                death_hovered = true;
                                                if let Some(t) = p.last_death_time {
                                                    death_hover = Some(
                                                        crate::map_player_icons::death_hover_text(
                                                            &self.player_name, t,
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                // The hover precedence is the C#'s write
                                // order: the tile name (`:55784-55913`),
                                // the revenge/misc (`:55922-55923`), the NPC
                                // heads (`:55926`), the player names
                                // (`:55959`), the icon layers (`:55996`),
                                // then the death duration (`:55984-55986`,
                                // num19 wins over the revenge UseMouseOver).
                                let fs_text = death_hover
                                    .or(hover)
                                    .or(pin_hover)
                                    .or(head_hover)
                                    .or(misc_hover)
                                    .or(tile_hover);
                                if let Some(t) = fs_text {
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
                                    // `Main.GetPlayerHeadBordersColor` (`Main.cs:56610-56618`):
                                    // ghost/dead → `Color.Transparent` (`:56612-56615`),
                                    // else `Main.teamColor[0]` = White (singleplayer,
                                    // `:56616-56618`). W4-c46 final: the ghost/dead arm.
                                    let border = if p.ghost || p.dead {
                                        [0.0; 4]
                                    } else {
                                        [1.0; 4]
                                    };
                                    draw_player_map_icon(
                                        renderer,
                                        player_assets,
                                        p,
                                        icon_pos,
                                        self.settings.applied_ui_scale(screen.x, screen.y),
                                        border,
                                    );
                                // `:55988-55995`: no death hover → the
                                // revenge marker's UseMouseOver money row.
                                if !death_hovered {
                                    if let Some(i) = revenge_hover {
                                        crate::map_player_icons::draw_revenge_mouse_over(
                                            renderer,
                                            lookups,
                                            self.revenge_markers[i].coins_value,
                                            self.mouse_pos,
                                            num77,
                                        );
                                    }
                                }
                                // `Main.cs:56014-56036`: the close button draws
                                // AFTER the hover text (`:56046-56049`'s
                                // `MouseText` rides the batch the icon calls
                                // run in; the button's `:56012` batch is the
                                // UI one) — the bottom-left `Map_0`/`Map_4`
                                // icon, hover resolved against the same box
                                // `handle_fullmap_close` clicks.
                                let close_pos =
                                    ui::chrome::fullmap_close_button_pos(screen);
                                let close_hovered = self.mouse_pos.x >= close_pos[0]
                                    && self.mouse_pos.x
                                        <= close_pos[0] + ui::chrome::MAP_ICON_HIT[0]
                                    && self.mouse_pos.y >= close_pos[1]
                                    && self.mouse_pos.y
                                        <= close_pos[1] + ui::chrome::MAP_ICON_HIT[1];
                                ui::chrome::draw_fullmap_close_button(
                                    renderer,
                                    lookups,
                                    screen,
                                    close_hovered,
                                );
                                // `Main.cs:56037-56040`: the vanilla cursor
                                // over the fullscreen map — `DrawCursor(
                                // DrawThickCursor())` with the thick border
                                // ring (`Main.cs:48064-48134`) and the
                                // `(2, 2)` bonus. `!Main.gameMenu &&
                                // !Main.InGameUI.IsVisible` — the port draws
                                // no in-game UI over the map (the options
                                // window rides the inventory, which is shut).
                                let bonus = crate::ui::cursor::draw_thick(
                                    renderer,
                                    lookups,
                                    &self.cursor,
                                    self.mouse_pos,
                                    settings.mouse_border_color,
                                );
                                crate::ui::cursor::draw_with_bonus(
                                    renderer,
                                    lookups,
                                    &self.cursor,
                                    self.mouse_pos,
                                    bonus,
                                );
                                // The union's single local-head block DROPS
                                // here (research/140 §8.2 hunk 5): the f29
                                // 255-player head loop (`Main.cs:55931-55976`,
                                // above) replaces it — local + remote pins
                                // with the team borders + name hovers.
                                // `Main.cs:62160`: the fullscreen map's draw
                                // tail runs `DrawPlayerChat` after `DrawMap`
                                // — the chat log + input strip draw over the
                                // map (the HUD is suppressed, the chat is
                                // not). `DrawFPS` (`:62159`) rides
                                // `showFrameRate` (F10, unported);
                                // `DrawPendingMouseText` (`:62161`) is the
                                // hover-text pass the map's own `MouseText`
                                // (`:56046-56049`) already covers.
                                ui::draw_chat(renderer, lookups, screen, &mut self.chat);
                            }
                            // The `else if self.map_style == 1` minimap branch
                            // (`Main.cs:55503-55547`) draws above `ui::draw_hud`
                            // now — vanilla renders interface layer 16 (the
                            // minimap, `Main.cs:44796`) before layers 25/27/30
                            // (`:44831/:44841/:44861`), so the hearts and the
                            // inventory paint over the frame.
                        }
                    }

                    // `Main.DrawFPS` (`Main.cs:21491-21526`, called at `:62407`):
                    // the fps/ups line at `(4, screenHeight - 24)` while
                    // `showFrameRate` holds. `DoUpdate_F7`'s `drawDiag > 0` forces
                    // `showFrameRate = true` (`:62479-62481`); the DetailedFPS
                    // graph / TimeLogger draws (`:62485/:62494`) are unported
                    // (W4-g21 batch 2). The port's fps rides `frame_history`.
                    let show_fps = self.show_frame_rate || self.draw_diag > 0;
                    if show_fps && !crate::env::no_hud() {
                        let avg = if frame_history.is_empty() {
                            0.0
                        } else {
                            frame_history.iter().sum::<f32>() / frame_history.len() as f32
                        };
                        let fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
                        // `Main.frameRate + "fps " + Main.updateRate + "ups"`
                        // (`:21495`) — the port's update rate is the 60 Hz step.
                        let text = format!("{fps:.0}fps 60ups");
                        let c = self.mouse_text_color as f32 / 255.0;
                        hud_text(
                            renderer,
                            lookups,
                            4.0,
                            screen.y - 24.0,
                            1.0,
                            [c, c, c, c],
                            &text,
                        );
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
                            // W4-g11: the flakes are the shared pool's live
                            // type-76 population (`Main.snowing`'s spawns).
                            crate::minecart::live_type(dust, 76),
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
                    // S1-G-13 consumers (`TR_LAYER_TARGETS=1`): the world
                    // renders through the render-target family — the
                    // `RenderToTargets` executor drives the per-layer
                    // callbacks, the skyTarget bind captures the sky range,
                    // and the six `DoDraw` composites land on the screen in
                    // order with the direct-screen entity blocks interleaved
                    // (`Main.cs:57277-57327`, `:62021-62027`, `:62196-63447`).
                    // The fused default below is untouched.
                    if layer_targets_on {
                        let now_secs = now.elapsed().as_secs_f32();
                        if let Some(family) = targets_parked.as_mut() {
                            let compositor = compositor_parked
                                .get_or_insert_with(|| tr_render::TargetCompositor::new(&gpu.device));
                            let remote_vec: Vec<_> =
                                remote_players.values().cloned().collect();
                            let held = match render_player.as_ref() {
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
                                            (
                                                slot.kind,
                                                *swing / self.swing_max.max(f32::EPSILON),
                                                slot.prefix,
                                            )
                                        })
                                }
                                _ => None,
                            };
                            let armor = [
                                self.equip.armor.first().map_or(0, |s| if s.count > 0 { s.kind } else { 0 }),
                                self.equip.armor.get(1).map_or(0, |s| if s.count > 0 { s.kind } else { 0 }),
                                self.equip.armor.get(2).map_or(0, |s| if s.count > 0 { s.kind } else { 0 }),
                            ];
                            let mut lf = crate::app::layer_targets::LayerFrame {
                                world,
                                lookups,
                                atlas: atlas.as_ref(),
                                sky,
                                light,
                                scratch,
                                params: &params,
                                style: &style,
                                entity_lights: &self.entity_lights,
                                anim_time,
                                waterfall,
                                liquid_alpha,
                                sparkle_seed: (*frame_count as u64)
                                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                                    ^ world.seed,
                                wind,
                                grass_wind_counter: *grass_wind_counter,
                                tree_wind_counter: *tree_wind_counter,
                                sunflower_wind_counter: *sunflower_wind_counter,
                                vine_wind_counter: *vine_wind_counter,
                                wiring: &self.wiring,
                                party_is_up: crate::world_events::party_is_up(&self.world_events),
                                show_invisible_blocks: crate::env::show_invisible(),
                                tile_anim: &self.tile_anim,
                                camera: draw_cam,
                                zoom: world_zoom,
                                screen,
                                sky_zoom: bg_zoom,
                                cave_parallax: cave_parallax(settings.parallax_scroll),
                                player: render_player.as_ref(),
                                player_assets,
                                held,
                                armor,
                                npcs,
                                npc_assets,
                                items,
                                remote: &remote_vec,
                                edit_target: *edit_target,
                                leaf_spawns: &mut leaf_spawns,
                                // W4-g72 closeout delta 64d §2: the frame's
                                // scene metrics + the App's cavern-style
                                // latch for the 14-arg draw_background.
                                metrics: &self.scene_metrics,
                                ug_back: &mut self.ug_back,
                            };
                            // The skyTarget gate (`Main.cs:62021-62027`):
                            // `CanCapture || ForegroundSunlightEffects` (the
                            // retail default true, `:66574`) AND the target
                            // path / not-fullscreen-map / not-fancy-UI /
                            // not-Retro. The port's `onlyDrawFancyUI` is false.
                            let can_capture = tr_core::scene::FilterId::ALL
                                .iter()
                                .any(|&id| filter_driver.board.in_list(id));
                            let not_retro = !matches!(
                                *light_mode,
                                tr_core::lighting::LightMode::Retro
                                    | tr_core::lighting::LightMode::Trippy
                            );
                            let map_fullscreen =
                                settings.map_enabled && map.as_ref().is_some_and(|m| m.open);
                            let sky_active = tr_render::sky_target_active(
                                can_capture,
                                true,
                                false,
                                map_fullscreen,
                                false,
                                not_retro,
                            );
                            crate::app::layer_targets::render_sky_target(
                                gpu, renderer, family, &lf, compositor, &view, sky_active, now_secs,
                            );
                            tile_stats = crate::app::layer_targets::render_layer_targets(
                                gpu, renderer, family, &mut lf, compositor,
                                &mut self.layer_render_count, &view, now_secs,
                            );
                            // The fx-range passes the frame pushed above
                            // (rain/gore/dust/leaves/wires/projectiles, the
                            // screen-fx tail) ride the world range; render
                            // them over the tile composite, under the water
                            // (`Main.cs:62279-62315`'s entity section).
                            let fx_stats = renderer.render_fx_only(
                                gpu, &view, draw_cam.to_array(), world_zoom, now_secs,
                            );
                            stats.instances += fx_stats.instances;
                            stats.draw_calls += fx_stats.draw_calls;
                            stats.uploaded_bytes += fx_stats.uploaded_bytes;
                            // The waterTarget composite after the entity
                            // passes (`Main.cs:62308`).
                            crate::app::layer_targets::composite_water(
                                gpu, renderer, family, &lf, compositor, &view, now_secs,
                            );
                            // The map batches + the interface over the
                            // composited world (the fx range holds the
                            // rain/gore/projectile passes pushed above).
                            let ui_stats = renderer.render_map_ui(
                                gpu, &view, draw_cam.to_array(), *zoom, now_secs,
                            );
                            stats.instances += ui_stats.instances;
                            stats.draw_calls += ui_stats.draw_calls;
                            stats.uploaded_bytes += ui_stats.uploaded_bytes;
                        }
                    } else if let Some(post) = post.as_mut() {
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
            // S1-G-13 consumers: put the parked family + compositor back.
            self.targets = targets_parked;
            self.target_compositor = compositor_parked;
        // The popup click's `OpenAchievementsAndGoto` (S1-E-26 wave A) —
        // the field borrows are released, so the overlay open + scroll lands
        // for the next frame's draw.
        if let Some(name) = popup_goto {
            self.open_achievements_and_goto(name);
        }
        // The fullscreen pylon marker click (`TeleportPylonsMapLayer.cs:
        // 90-109`) — drained here, the field borrows released.
        if let Some(click) = pylon_click {
            self.handle_pylon_map_click(click);
        }
        // `Main.cs:56006`: the fullscreen tail clears `cancelWormHole` every
        // frame (the pylon hover re-arms it on the next draw).
        if self.map.as_ref().is_some_and(|m| m.open) {
            if let Some(m) = self.map.as_mut() {
                m.cancel_wormhole = false;
            }
        }
        // W4-g56 — `Main.cs:61811-61812`: `lastMouseInterface =
        // mouseInterface; mouseInterface = false` at the update tail. The
        // port recomputes the hover surfaces here (the draw tail — the
        // button state cannot change mid-frame, the buff-latch precedent);
        // the next draw's `TryInteractingWithVoidLens` reads it.
        self.last_mouse_interface = self.mouse_interface_now();
        Some((stats, tile_stats))
    }

    /// `Recipe.UpdateRecipeList`'s raw availableRecipe list
    /// (`Recipe.cs:230-259`): the owned counts fold the inventory and the
    /// in-range chests; the environment reads the adjacency scan and the
    /// zone flags. `Main.focusRecipe` indexes THIS list
    /// (`NewCraftingUI.cs:610`, `:622`, `:904`), not the filter-narrowed
    /// grid. Extracted from the draw region (W4-f23 closeout item 2) so the
    /// rebuild reads the same inputs from either C# draw site — the modern
    /// grid (`:585`) and the classic pip-list arm (`Main.cs:41712`).
    /// The `UIWorldGenDebug` driver's per-frame feed (W4-f23 closeout item 1):
    /// the live job's per-step snapshot + the executed pipeline's descriptors.
    /// `None` when no generation job runs (the `TR_UI=worldload` preset parks a
    /// fixed snapshot with no worker — the debug overlay has nothing to feed).
    pub(crate) fn worldgen_debug_feed(&self) -> Option<ui::worldgen_debug::DebugFeed> {
        let job = self.world_gen_job.as_ref()?;
        let snap = job.snapshot();
        Some(ui::worldgen_debug::DebugFeed {
            steps: tr_core::worldgen::pipeline::executed_steps(job.micro_flags),
            current_step: snap.current_step,
            completed_steps: snap.completed_steps,
        })
    }

    pub(super) fn craft_raw_list(&self) -> Vec<usize> {
        // The reach (`Player.tileRangeX/Y` + `TileReachCheckSettings.Simple`):
        // the Extendo Grip / the Architect's Gizmo Pack, the held Toolbox,
        // the Journey mode's Far Placement Range power and the artisan loaf.
        let reach = self.player.as_ref().map_or((5, 3), |p| {
            craft::tile_reach(
                p.equipped[..10].contains(&2215) || p.equipped[..10].contains(&3061),
                self.mouse_item.kind == 1923,
                self.creative_powers.far_placement_range(),
                p.ate_artisan_bread,
            )
        });
        let (adj, in_range) = self
            .player
            .as_ref()
            .map(|p| {
                (
                    craft::scan_adjacent(&self.world, p, reach),
                    craft::chests_in_range(
                        &self.world,
                        p,
                        &self.world_chests,
                        self.settings.craft_from_nearby_chests,
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
            unlocked_biome_torches: self
                .player
                .as_ref()
                .map_or(false, |p| p.unlocked_biome_torches),
            // `SpecialSeedFeatures.Mechdusa` (`SpecialSeedFeatures.cs:119-127`).
            mechdusa: self.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX)
                && self
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
        };
        craft::available_with_gates(
            &self.inventory,
            &self.equip,
            &self.world_chests,
            &in_range,
            &adj,
            &zones,
            self.craft_filter.as_ref(),
            None,
            &gates,
        )
    }
}

/// `Main.DoUpdate_AnimateTileGlows`' martianLight half (`Main.cs:18251-
/// 18261`): `martianLight += martianLightDir * 0.015f`, then the two
/// clamps — `> 1` snaps to 1 and flips the dir to -1, `< 0` snaps to 0
/// and flips to +1. Both clamps run every call (no else), exactly like
/// the demon-torch half above it. Called per draw frame at the C#
/// `:17145` site — UNCONDITIONALLY, no Martian-event gate (the
/// `drawdata_cases_a.rs` "0 outside the event" note was wrong).
fn tick_martian_light(light: &mut f32, dir: &mut f32) {
    *light += *dir * 0.015;
    if *light > 1.0 {
        *light = 1.0;
        *dir = -1.0;
    }
    if *light < 0.0 {
        *light = 0.0;
        *dir = 1.0;
    }
}

#[cfg(test)]
mod w4_g12_martian_light_tests {
    use super::tick_martian_light;

    /// The bounce starts at (1, +1) (`Main.cs:66831-66832`): the first
    /// tick overshoots to 1.015, clamps back to 1 and flips the dir —
    /// the same first-tick shape as the demon torch.
    #[test]
    fn w4_g12_the_first_tick_clamps_and_flips_like_demon_torch() {
        let (mut l, mut d) = (1.0f32, 1.0f32);
        tick_martian_light(&mut l, &mut d);
        assert_eq!((l, d), (1.0, -1.0));
        tick_martian_light(&mut l, &mut d);
        assert!((l - 0.985).abs() < 1e-6, "the second tick descends: {l}");
        assert_eq!(d, -1.0);
    }

    /// The bottom clamp mirrors the top: below 0 snaps to 0 and flips
    /// to +1; both clamps are independent `if`s (no else).
    #[test]
    fn w4_g12_the_bottom_clamp_snaps_to_zero_and_flips_up() {
        let (mut l, mut d) = (0.01f32, -1.0f32);
        tick_martian_light(&mut l, &mut d);
        assert_eq!((l, d), (0.0, 1.0));
        tick_martian_light(&mut l, &mut d);
        assert!((l - 0.015).abs() < 1e-6);
    }

    /// The glow byte the value feeds (`TileDrawing.cs:498`):
    /// `b = (byte)(100 + 150 * martianLight)` truncates — 1 → 250,
    /// 0 → 100, 0.5 → 175. The full bounce spans [100, 250], never 0:
    /// the old constant-0 feed pinned the glow at 100 forever.
    #[test]
    fn w4_g12_the_glow_byte_spans_100_to_250_over_the_bounce() {
        let b = crate::drawdata_cases_a::martian_glow;
        assert_eq!(b(1.0), [250, 250, 250, 0]);
        assert_eq!(b(0.0), [100, 100, 100, 0]);
        assert_eq!(b(0.5), [175, 175, 175, 0]);
        // A full bounce cycle visits both ends.
        let (mut l, mut d) = (1.0f32, 1.0f32);
        let (mut saw_min, mut saw_max) = (false, false);
        for _ in 0..300 {
            tick_martian_light(&mut l, &mut d);
            saw_min |= l == 0.0;
            saw_max |= l == 1.0;
        }
        assert!(saw_min && saw_max, "the bounce reaches both clamps");
    }
}

#[cfg(test)]
mod s1_p_i53_tests {
    use crate::app::flow::front_flow_tests::flow_app;

    /// S1-P-i53 — the buff cancel gate is `Main.mouseRight &&
    /// Main.mouseRightRelease` (`Main.cs:43333`) with the end-of-draw latch
    /// (`:62458-62465`): it fires on the PRESS frame after a released
    /// frame, never on held frames, and never on the release frame (the
    /// bare-edge port fired one frame late).
    #[test]
    fn s1_p_i53_the_cancel_gate_fires_on_press_not_on_release() {
        let mut app = flow_app(None, vec![], vec![]);
        // Released frames: no gate, latch stays true.
        app.place_down = false;
        assert!(!app.buff_cancel_gate());
        assert!(!app.buff_cancel_gate());
        // The press frame: mouseRight true + the latch still true → fire.
        app.place_down = true;
        assert!(app.buff_cancel_gate(), "the press-after-release frame");
        // Held frames: the latch went false — no repeat.
        assert!(!app.buff_cancel_gate());
        assert!(!app.buff_cancel_gate());
        // The release frame: mouseRight false → no gate (the old bare-edge
        // port fired HERE), and the latch re-arms.
        app.place_down = false;
        assert!(!app.buff_cancel_gate(), "the release frame never cancels");
        // The next press fires again.
        app.place_down = true;
        assert!(app.buff_cancel_gate());
    }

    /// W4-g68 final-phase census: the WindGrid scan call site carries the
    /// `WindGrid.cs:91` dead-player gate — `!player.active || player.dead`
    /// refuses the stamp (a drifting corpse must not push grass/vines).
    /// The `active` half is the `Option` itself; the `dead` half is the
    /// `.filter(|p| !p.dead)` on the scan's player borrow. Source pin (the
    /// `tiles.rs` include_str convention — the draw pass needs a GPU).
    #[test]
    fn w4_g68_the_wind_grid_scan_gates_the_dead_player() {
        let src = include_str!("draw.rs");
        let scan = &src[src.find("wind.scan_player(").unwrap()..];
        let head = &src[..src.find("wind.scan_player(").unwrap()];
        let borrow = &head[head.rfind("if let Some(p) = player.as_ref()").unwrap()..];
        assert!(
            borrow.contains(".filter(|p| !p.dead)"),
            "the scan's player borrow filters the dead (WindGrid.cs:91)"
        );
        // The gate sits INSIDE the sway-enabled arm (the C# ScanPlayers
        // runs under SettingsEnabled_TilesSwayInWind, WindGrid.cs:35-38).
        let sway = head.rfind("if wind.sway_enabled").unwrap();
        let filt = head.rfind(".filter(|p| !p.dead)").unwrap();
        assert!(sway < filt && filt < head.len(), "the gate order");
        assert!(scan.starts_with("wind.scan_player("));
    }
}

#[cfg(test)]
mod w4_f23_closeout_craft_list_tests {
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::craft;

    /// W4-f23 closeout item 2 — the classic pip-list arm's
    /// `Recipe.UpdateRecipeList` (`Main.cs:41712`) runs while the inventory
    /// is open and the modern grid is NOT visible. The port collapses both
    /// C# draw sites onto the one grid, so `craft_raw_list` (the extracted
    /// rebuild) must answer the SAME raw availableRecipe list whether the
    /// panel is open or closed-but-inventory-open: the closed-panel
    /// deviation is retired — the snapshot tracks every inventory change.
    #[test]
    fn the_raw_list_rebuilds_while_the_panel_is_closed_but_inventory_open() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // Stock the torch ingredients (1 wood + 1 gel per craft, row 0).
        app.inventory[0] = crate::item::ItemSlot { kind: 9, name: "Wood", count: 30, prefix: 0 };
        app.inventory[1] = crate::item::ItemSlot { kind: 23, name: "Gel", count: 30, prefix: 0 };
        app.inventory_open = true;
        // The panel CLOSED (the classic pip-list arm's gate, `:41710`'s
        // `else if (!NewCraftingUI.Visible ...)`): the rebuild still runs.
        app.crafting_open = false;
        let closed = app.craft_raw_list();
        assert!(
            closed.iter().any(|&r| craft::RECIPES[r].create.0 == 8),
            "the torch recipe is available with the panel closed"
        );
        // The panel OPEN (the modern grid's `UpdateContents` `:585` site):
        // the same inputs, the same list.
        app.crafting_open = true;
        let open = app.craft_raw_list();
        assert_eq!(closed, open, "both C# draw sites rebuild the same raw list");
    }

    /// The refocus tail (`Recipe.cs:232-233`, `:257`) follows the recipe
    /// across the rebuild even while the panel is closed: after the
    /// ingredients drain, the focused recipe leaves the list and the focus
    /// pins to the last row — the `:41712` arm keeps `Main.focusRecipe`
    /// live, not frozen at the last open frame.
    #[test]
    fn the_focus_follows_the_recipe_across_a_closed_panel_rebuild() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.inventory[0] = crate::item::ItemSlot { kind: 9, name: "Wood", count: 30, prefix: 0 };
        app.inventory[1] = crate::item::ItemSlot { kind: 23, name: "Gel", count: 30, prefix: 0 };
        app.inventory_open = true;
        app.crafting_open = false;
        // First rebuild: the torch is row 0; focus it.
        let first = app.craft_raw_list();
        let torch_pos = first.iter().position(|&r| craft::RECIPES[r].create.0 == 8).unwrap();
        app.focus_recipe = torch_pos as i32;
        app.craft_last_available = first.clone();
        // Drain the gel — the torch leaves the available list.
        app.inventory[1].count = 0;
        let second = app.craft_raw_list();
        assert!(
            !second.iter().any(|&r| craft::RECIPES[r].create.0 == 8),
            "no gel, no torch"
        );
        // The refocus tail the draw region runs on the rebuilt list.
        let old_recipe = app.craft_last_available[torch_pos];
        let new_focus = craft::try_refocusing_recipe(&second, old_recipe, app.focus_recipe);
        assert!(
            (new_focus as usize) < second.len().max(1),
            "the focus pins into the rebuilt list, not the stale position"
        );
    }
}

#[cfg(test)]
mod w4_g75_final_sweep_tests {
    /// W4-g75 final sweep (`d118f7ba`, folded) - the waterfall block feeds
    /// the manager the SAME `Main.DiscoR/G/B` the `LightParams` carry:
    /// the disco fall (tile 160 -> type 2) reads the shared statics
    /// (`StylizeColor` case 2, `WaterfallManager.cs:1010-1012`;
    /// `AddLight` case 2, `:1084`), and the cycle is
    /// `Main.DoUpdate_AnimateDiscoRGB` (`Main.cs:19591-19648`), stepped
    /// by `advance_disco` - the manager has no cycle of its own. Without
    /// this feed the fall tinted from a stale private copy.
    #[test]
    fn the_waterfall_block_latches_the_shared_disco_statics() {
        let src = include_str!("draw.rs");
        assert!(
            src.contains("waterfall.set_disco_rgb(params.disco)"),
            "the waterfall block must latch the shared disco statics"
        );
        assert!(
            src.contains("waterfall.set_show_invisible(params.draw_invisible_walls)"),
            "the invisible latch stays fed too"
        );

        }
        }mod w4_e7_global_flicker_tests {
    use super::bounce_oscillator;

    /// W4-e7 lane 23 — the masterColor pulse (`Main.cs:18225-18235`): +0.05
    /// per frame from the 1f init, flipping at the rails. WRITTEN NOT RUN.
    fn w4_e7_master_color_bounces_at_the_rails() {
        // Init 1f / dir 1 (Main.cs:66833-66834): the first step clamps to 1
        // and flips dir to -1.
        let (v, d) = bounce_oscillator(1.0, 1.0, 0.05);
        assert_eq!((v, d), (1.0, -1.0), "the top rail flips");
        // Then it descends by 0.05.
        let (v, d) = bounce_oscillator(v, d, 0.05);
        assert!((v - 0.95).abs() < 1e-6 && d == -1.0);
        // The bottom rail: 0.02 - 0.05 < 0 → clamp 0, flip +1.
        let (v, d) = bounce_oscillator(0.02, -1.0, 0.05);
        assert_eq!((v, d), (0.0, 1.0), "the bottom rail flips");
    }
    /// The martianLight pulse (`Main.cs:18251-18261`): +0.015 per frame.
    fn w4_e7_martian_light_bounces_at_the_rails() {
        let (v, d) = bounce_oscillator(1.0, 1.0, 0.015);
        assert_eq!((v, d), (1.0, -1.0));
        let (v, d) = bounce_oscillator(v, d, 0.015);
        assert!((v - 0.985).abs() < 1e-6 && d == -1.0);
        let (v, d) = bounce_oscillator(0.01, -1.0, 0.015);
        assert_eq!((v, d), (0.0, 1.0));
    /// The oscillator stays bounded in [0, 1] and reverses at the rails (the
    /// demon_torch 0.01 shape): deterministic and RNG-free, so the flicker
    /// phase is a pure function of the frame count. The C# clamp holds one
    /// frame at each rail before descending (the `> 1` / `< 0` flip fires on
    /// the overshoot step), so the value never leaves the unit interval.
    fn w4_e7_the_oscillator_stays_bounded_and_rng_free() {
        let (mut v, mut d) = (1.0f32, 1.0f32);
        let mut saw_bottom = false;
        for _ in 0..200 {
            (v, d) = bounce_oscillator(v, d, 0.05);
            assert!((0.0..=1.0).contains(&v), "bounded: {v}");
            if v == 0.0 {
                saw_bottom = true;
            }
        }
        assert!(saw_bottom, "the pulse reaches the bottom rail");
        // After reaching 0 it climbs back toward 1 (the dir flipped +1).
        assert!(d == 1.0 || v > 0.0, "the reversal climbs back");
    }

// balance-close (union shared delimiter)
}}))
}

#[cfg(test)]
mod w4_e18_fullmap_draw_head_tests {
    use crate::app::flow::front_flow_tests::flow_app;

    /// W4-e18 final sweep (written, not run - code-only): the fullscreen
    /// map's draw head (`Main.cs:62150-62153`) closes a dialog/inventory
    /// opened while the map is up — `talkNPC >= 0 || (playerInventory &&
    /// !CaptureManager.Active)` → `ToggleInv`'s talk/inventory arms
    /// (`Player.cs:5005-5016`/`:5033-5056`), leaving the map open. The
    /// capture-active arm keeps the inventory.
    #[test]
    fn w4_e18_the_fullmap_draw_head_closes_the_dialog_and_inventory() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = tr_core::World::new("drawhead".into(), 260, 200);
        app.map = Some(crate::map::WorldMap::new(&app.world));
        app.map.as_mut().unwrap().set_open(true);
        // A dialog + inventory opened while the map is up.
        app.talk_npc = Some(0);
        app.npc_chat_text = "hi".into();
        app.inventory_open = true;
        app.close_dialog_behind_fullmap();
        assert!(app.map.as_ref().unwrap().open, "the map stays open");
        assert_eq!(app.talk_npc, None, "SetTalkNPC(-1)");
        assert!(app.npc_chat_text.is_empty(), "the text clears");
        assert!(!app.inventory_open, "the inventory closes");
        // Capture active: the inventory arm is gated off (`!Active`), and
        // with no dialog the whole block is a no-op.
        app.inventory_open = true;
        app.capture.interface.active = true;
        app.close_dialog_behind_fullmap();
        assert!(app.inventory_open, "the capture gate keeps the inventory");
        // The map shut: nothing closes.
        app.capture.interface.active = false;
        app.map.as_mut().unwrap().set_open(false);
        app.talk_npc = Some(0);
        app.close_dialog_behind_fullmap();
        assert_eq!(app.talk_npc, Some(0), "the map-shut head is a no-op");
        assert!(app.inventory_open);
    }
}
