//! The `SceneMetrics` tile-window scan (`SceneMetrics.Scan` / `ScanTiles`).
//!
//! Lives in `tr-app` because the scan is centered on the player (a `tr-app`
//! type); everything it fills is `tr_core::scene::SceneMetrics` data.
//!
//! Source: `decompiled/Terraria/SceneMetrics.cs` (`Scan` `:336-351`,
//! `ScanTiles` `:353-379`, `CalculateZones` `:662-707`), `Main.cs:62541`
//! (the scan center is the local player's `Center`).

use glam::Vec2;

use tr_core::scene::{is_desert_biome_sand, ocean_depths, SceneMetrics, ZONE_SCAN_SIZE};
use tr_core::scene::{Sandstorm, SceneState};
use tr_core::World;

use crate::app::App;
use crate::npc::{Npc, NpcKind};
use crate::player::{self, Player};

/// `SceneMetrics.TownNPCRectSize` (`SceneMetrics.cs:920`):
/// `AssumedConstantScreenSize.ToVector2() * 2f` = (3840, 2400) px.
const TOWN_NPC_RECT_SIZE: (f32, f32) = (1920.0 * 2.0, 1200.0 * 2.0);

/// `Main.SceneMetrics.Scan(new SceneMetricsScanSettings { BiomeScanCenterPositionInWorld =
/// player.Center … })` (`Main.cs:62533-62546`, `SceneMetrics.cs:336-351`): reset,
/// compute the tile center, run the window pass, derive the counts and the zones.
///
/// `raining` is `Main.raining` (our `Rain::is_raining()`), `sandstorm_happening`
/// is `Sandstorm::happening` — both are game-state inputs the zone derivation
/// reads (`SceneMetrics.cs:701,703`).
#[cfg(test)]
pub(crate) fn scan_scene(
    world: &World,
    player: &Player,
    raining: bool,
    sandstorm_happening: bool,
) -> SceneMetrics {
    // `metrics.Center` = the local player's center in world pixels
    // (`Main.cs:62541`; the player position is the top-left corner).
    let center = [
        player.x + player::WIDTH * 0.5,
        player.y + player::HEIGHT * 0.5,
    ];
    scan_scene_at(
        world,
        center,
        raining,
        sandstorm_happening,
        player.shimmer_wet,
    )
}

/// The injectable center variant of `scan_scene`: the headless TR_NOPLAYER
/// shots stand in the camera centre for the absent player (vanilla has no
/// such path — a test hook, not a behaviour). No visual area and no NPCs: the
/// `ScanOnScreenTiles`/`ScanNPCPositions` halves need both (see
/// [`scan_scene_full`]).
pub(crate) fn scan_scene_at(
    world: &World,
    center: [f32; 2],
    raining: bool,
    sandstorm_happening: bool,
    player_shimmering: bool,
) -> SceneMetrics {
    scan_scene_full(
        world,
        center,
        raining,
        sandstorm_happening,
        player_shimmering,
        None,
        &[],
    )
}

/// [`scan_scene_at`] plus the two settings the live driver passes and the
/// unit tests do not need: `visual_area` is the C#'s
/// `SceneMetricsScanSettings.VisualScanArea` as `(left, top, right, bottom)`
/// in tiles (`Main.cs:62536-62543`, `Main.GetAreaToLight()` inflated by 28)
/// and `npcs` is `Main.npc` for `ScanNPCPositions` (`:727-745`).
pub(crate) fn scan_scene_full(
    world: &World,
    center: [f32; 2],
    raining: bool,
    sandstorm_happening: bool,
    player_shimmering: bool,
    visual_area: Option<(i32, i32, i32, i32)>,
    npcs: &[Npc],
) -> SceneMetrics {
    // `this.TileCenter = Center.ToTileCoordinates().ClampedInWorld()`
    // (`SceneMetrics.cs:333`); `ToTileCoordinates` is `(int)v >> 4`
    // (`Utils.cs:1807-1810`). `ClampedInWorld` clamps both coordinates to
    // `maxTilesX - 1` — the vanilla Y clamp uses `maxTilesX` too
    // (`Utils.cs:1855-1860`, latent bug: width > height on every vanilla world
    // size, so it never binds there either).
    let tx = (center[0] as i32) >> 4;
    let ty = (center[1] as i32) >> 4;
    let tile_center = (tx.clamp(0, world.width - 1), ty.clamp(0, world.width - 1));

    let mut metrics = SceneMetrics::new();
    metrics.center = center;
    metrics.tile_center = tile_center;
    scan_tiles(world, &mut metrics);
    // `SceneMetrics.cs:338-341`: the visual-area pass runs only when the
    // settings carry the rect (`Main.cs:62540` always does).
    if let Some(area) = visual_area {
        metrics.scan_on_screen_tiles(world, area);
    }
    // `SceneMetrics.cs:342-345`: `ScanNPCPositions` when the settings ask
    // (`Main.cs:62541` sets `ScanNPCPositions = true`).
    scan_npc_positions(&mut metrics, npcs);
    // `CalculateZones`'s `BehindBackwall` probe (`SceneMetrics.cs:671-672`):
    // `Framing.GetTileSafely(TileCenter).wall > 0`.
    metrics.behind_backwall = world
        .tile(tile_center.0, tile_center.1)
        .is_some_and(|t| t.wall > 0);
    metrics.aggregate_tile_counts();
    // `AddPlayerEffects` (`SceneMetrics.cs:747`): the local player's
    // shimmering flag rides the metrics (`shimmerWet` = the center tile).
    metrics.player_shimmering = player_shimmering;
    // `CalculateZones`' `ZoneUnderworldHeight`/`ZoneBeach` terms
    // (`SceneMetrics.cs:674,691`): `Main.UnderworldLayer` is `maxTilesY - 200`
    // (`Main.cs:2790`, `spawner::UNDERWORLD_LAYER`).
    // `SceneMetrics.cs:682-684`: ZoneDungeon — the brick count, below the
    // world surface, behind a dungeon wall at the scan centre.
    if metrics.dungeon_tile_count >= tr_core::scene::thresholds::DUNGEON_TILE
        && (metrics.tile_center.1 as f64) > world.world_surface
    {
        metrics.zone_dungeon = world
            .tile(metrics.tile_center.0, metrics.tile_center.1)
            .is_some_and(|t| tr_core::scene::is_dungeon_wall(t.wall));
    }
    metrics.calculate_zones(
        world.world_surface,
        world.height - 200,
        world.rock_layer,
        world.width,
        world.height,
        raining,
        sandstorm_happening,
    );
    // `CalculateZones`' wall terms (`SceneMetrics.cs:694-699`): the centre
    // tile's wall — the temple/granite/marble/hive/gem-cave zones and the
    // underground desert.
    metrics.calculate_wall_zones(world);
    // `Main.maxTilesX`/`maxTilesY`: the `MoonlordDeathDrama` `Dead` walls
    // (`MoonlordDeathDrama.cs:29-31`, `:105`).
    metrics.world_tiles = (world.width, world.height);
    metrics
}

/// `SceneMetrics.ScanTiles` (`SceneMetrics.cs:353-379`): count every active
/// tile in `ClampToWorld(CenteredRectangle(TileCenter, ZoneScanSize))`.
fn scan_tiles(world: &World, metrics: &mut SceneMetrics) {
    // `Utils.CenteredRectangle(Point, Point)` (`Utils.cs:952-956`).
    let left = metrics.tile_center.0 - ZONE_SCAN_SIZE.0 / 2;
    let top = metrics.tile_center.1 - ZONE_SCAN_SIZE.1 / 2;
    let right = left + ZONE_SCAN_SIZE.0;
    let bottom = top + ZONE_SCAN_SIZE.1;
    // `WorldUtils.ClampToWorld` (`WorldBuilding/WorldUtils.cs`, fluff 0): all
    // four edges clamp independently, so a window hanging off the world edge
    // shrinks instead of sliding back in.
    let left = left.clamp(0, world.width);
    let top = top.clamp(0, world.height);
    let right = right.clamp(0, world.width);
    let bottom = bottom.clamp(0, world.height);
    for x in left..right {
        for y in top..bottom {
            let Some(tile) = world.tile(x, y) else {
                // `SceneMetrics.cs:361-363`: absent tiles are skipped.
                continue;
            };
            if !tile.active() {
                // `SceneMetrics.cs:365-371`: inactive tiles only feed the
                // liquid counts (`_liquidCounts`) — the shimmer count is the
                // M1 reader (`ShimmerTileCount` = `_liquidCounts[3]`, `:594`).
                if tile.liquid > 0 && tile.liquid_kind() == tr_core::tile::LiquidKind::Shimmer {
                    metrics.shimmer_tile_count += 1;
                }
                continue;
            }
            // `SceneMetrics.cs:373`: `_tileCounts[tile.type]++`.
            if let Some(count) = metrics.tile_counts.get_mut(tile.kind as usize) {
                *count += 1;
            }
            // `SceneMetrics.cs:377-379`: ocean-edge desert sand.
            if is_desert_biome_sand(tile.kind)
                && ocean_depths(world.world_surface, world.rock_layer, x, world.width, y)
            {
                metrics.ocean_sand_tile_count += 1;
            }
            // The M2-BUFF-8 banner arm (`SceneMetrics.cs:416-445`): tile 91
            // decodes to the banner id; the strength-enabled ones raise the
            // buff (the port's family banners all carry the 1.5/0.75 pair).
            if tile.kind == 91 {
                if let Some(banner) =
                    tr_core::bestiary::banner_num_from_frame(tile.frame_x, tile.frame_y)
                {
                    if let Some(buff) = metrics.npc_banner_buff.get_mut(banner as usize) {
                        *buff = true;
                        metrics.has_banner = true;
                    }
                }
            }
            // The M2-BUFF-5 auras (`SceneMetrics.cs:378-409`): the lit-frame
            // gates ride the stored frames — the object system (DW-1.6)
            // writes real frames, so a computed `-1` only slips through the
            // `<` gates (a legacy -1 lantern matches neither range).
            match tile.kind {
                215 => {
                    if tile.frame_y < 36 {
                        metrics.has_campfire = true;
                    }
                }
                405 => {
                    if tile.frame_x < 54 {
                        metrics.has_campfire = true;
                    }
                }
                49 => {
                    if tile.frame_x < 18 {
                        metrics.water_candle_count += 1;
                    }
                }
                372 => {
                    if tile.frame_x < 18 {
                        metrics.peace_candle_count += 1;
                    }
                }
                646 => {
                    if tile.frame_x < 18 {
                        metrics.shadow_candle_count += 1;
                    }
                }
                506 => {
                    if tile.frame_x < 72 {
                        metrics.has_cat_bast = true;
                    }
                }
                42 => {
                    if (324..=358).contains(&tile.frame_y) {
                        metrics.has_heart_lantern = true;
                    }
                    if (252..=286).contains(&tile.frame_y) {
                        metrics.has_star_in_bottle = true;
                    }
                }
                _ => {}
            }
            // `SceneMetrics.cs:445`: `UpdateOreFinder(new Point(i, j), tile)`
            // closes the per-tile body of `ScanTiles`.
            metrics.update_ore_finder((x, y), tile);
        }
    }
}

/// `SceneMetrics.ScanNPCPositions` (`SceneMetrics.cs:727-745`): the nearest
/// active NPC of each type (for `WithinRangeOfNPC`) plus `TownNPCCount`.
///
/// `TownNPCRectSize` is `AssumedConstantScreenSize.ToVector2() * 2f`
/// (`:920`) = (3840, 2400) px, so the town rect is the player-centred
/// ±1920 x ±1200 box; `Utils.CenteredRectangle(Vector2, Vector2)`
/// (`Utils.cs:947-950`) truncates the origin toward zero.
fn scan_npc_positions(metrics: &mut SceneMetrics, npcs: &[Npc]) {
    // `Utils.CenteredRectangle(Vector2, Vector2)` (`Utils.cs:947-950`): the
    // origin truncates, the size is the vector cast to int.
    let rect_x = (metrics.center[0] - TOWN_NPC_RECT_SIZE.0 * 0.5) as i32;
    let rect_y = (metrics.center[1] - TOWN_NPC_RECT_SIZE.1 * 0.5) as i32;
    let rect_w = TOWN_NPC_RECT_SIZE.0 as i32;
    let rect_h = TOWN_NPC_RECT_SIZE.1 as i32;
    for npc in npcs {
        // `:732`: `nPC.active`. The port drops dead NPCs at the end of the
        // NPC tick (`npcs.rs`'s `retain(|n| n.hp > 0)`), so `hp > 0` is the
        // standing-active proxy.
        if npc.hp <= 0 {
            continue;
        }
        let kind = npc.kind.net_id();
        let Some(slot) = metrics.closest_npc_position.get_mut(kind as usize) else {
            continue;
        };
        let (cx, cy) = npc.center();
        // `:734-738`: first sighting or strictly closer wins.
        if (*slot == [0.0, 0.0])
            || dist_sq(metrics.center, [cx, cy]) < dist_sq(metrics.center, *slot)
        {
            *slot = [cx, cy];
        }
        // `:739-742`: `nPC.townNPC && CenteredRectangle(...).Contains(
        // nPC.Center.ToPoint())` — `ToPoint` truncates, `Contains` is
        // half-open on the far edge.
        let px = cx as i32;
        let py = cy as i32;
        if npc.kind.is_town_npc()
            && px >= rect_x
            && px < rect_x + rect_w
            && py >= rect_y
            && py < rect_y + rect_h
        {
            metrics.town_npc_count += 1;
        }
    }
}

/// `Vector2.DistanceSquared`.
fn dist_sq(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// Reads the TR_ determinism forces from the process environment and applies
/// them: `TR_SANDSTORM=<0..1>`, `TR_GRAVEYARD=<0..1>`, `TR_BLOODMOON=1`
/// (parsing is [`tr_core::scene::parse_env_force`]). Called once per fixed
/// step by the phase-3 driver, so a screenshot run pins the whole state.
///
/// While `TR_SANDSTORM` is set the driver should skip `Sandstorm::update_time`
/// entirely (the pin holds `severity == intended_severity`, but the periodic
/// intention roll would still fight it); `forced_severity` marks that.
pub(crate) fn apply_scene_forces_from_env(state: &mut SceneState, sandstorm: &mut Sandstorm) {
    apply_scene_forces(state, sandstorm, crate::env::lookup());
}

/// The injectable body of [`apply_scene_forces_from_env`]: `lookup` stands in
/// for the environment read so the wiring is unit-testable.
fn apply_scene_forces(
    state: &mut SceneState,
    sandstorm: &mut Sandstorm,
    lookup: impl Fn(&str) -> Option<String>,
) {
    for var in ["TR_SANDSTORM", "TR_GRAVEYARD", "TR_BLOODMOON"] {
        let Some(raw_value) = lookup(var) else {
            continue;
        };
        let entry = format!("{var}={raw_value}");
        let Some((name, value)) = tr_core::scene::parse_env_force(&entry) else {
            continue;
        };
        if name == "SANDSTORM" {
            sandstorm.force_severity(value);
        }
        state.force(name, value);
    }
}

/// The per-frame scene update (`Main.Update`'s tail, in the source order):
/// the TR_ pins, the `Sandstorm.EmitDust` (`Main.cs:17247`), the
/// `SceneMetrics.Scan` (`Main.cs:62560`), `SceneState.Update`
/// (`Main.cs:17816`) and the `ApplyVisuals` driver. Runs while a world is
/// live, regardless of `TR_POST` — the C# update runs unconditionally and the
/// driver output is simply unconsumed without the post pipeline.
///
/// `Sandstorm.UpdateTime` (`Main.cs:64966`) and the wind walk moved onto the
/// shared 60 Hz world tick (M1.4-BG-22, `world_sim::weather::update_world_tick`,
/// which `frame()` runs before `update_rain`/this scan): the wind and the
/// sandstorm state read here are same-frame fresh, no longer the one-draw-frame
/// lag the old draw-block wind tick produced.
pub(crate) fn update_scene(app: &mut App, dt: f32, screen: Vec2) {
    let App {
        world,
        player,
        rain,
        wind,
        camera,
        zoom,
        bg_style,
        bg_delay,
        anim_time,
        scene_state,
        sandstorm,
        filter_driver,
        settings,
        npcs,
        ..
    } = app;
    // The TR_ pins re-read per frame (idempotent; the parse glue of phase 1).
    // The world tick earlier this frame gates its sandstorm arm on
    // `forced_severity` too, so the pin holds from the first tick.
    apply_scene_forces_from_env(scene_state, sandstorm);
    // `Sandstorm.EmitDust` (`Main.cs:17247`): the flying sand spawns off the
    // stored `SceneMetrics` (vanilla's last-scan persistence) and ticks the
    // `Dust` 268 AI, ahead of this frame's rescan.
    let show = sandstorm.show_sandstorm_visuals(
        app.scene_metrics.zone_sandstorm,
        tr_core::scene::is_desert_bg_style(*bg_style),
        *bg_delay,
    );
    let player_vy = player.as_ref().map(|p| p.vy).unwrap_or(0.0);
    // The spawn ctx rides the disjoint field borrows (the whole-App
    // `spawn_ctx` would clash with the destructure's `&mut` fields).
    let ctx = crate::app::spawn_ctx_from(
        app.menu.is_some() || app.front.is_some(),
        app.game_paused,
        *camera,
        screen,
        *zoom,
        app.gfx_quality.max_dust_to_draw,
    );
    app.sand.emit_dust(
        &app.scene_metrics,
        sandstorm.severity,
        show,
        wind.speed_current,
        &ctx,
        world,
        &mut app.dust,
    );
    // `Main.UpdateSceneMetrics` (`Main.cs:62560`): the scan center is the
    // local player's center; headless shots (TR_NOPLAYER) stand in the camera.
    let center = player
        .as_ref()
        .map(|p| [p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5])
        .unwrap_or([camera.x, camera.y]);
    // `Main.GetAreaToLight()` (`Main.cs:63926-63936`) then `.Inflate(28, 28)`
    // (`:62537`): the view rect in tiles, one tile of slack on the leading
    // edges and two on the trailing ones. `ScaledPosition`/`ScaledSize` are
    // the unscaled screen rect (the port draws at `zoom` inside the same
    // pixels), so `screen / zoom` is the world-space view.
    let half = screen * 0.5 / *zoom;
    let view_min = *camera - half;
    let view_max = *camera + half;
    let visual_area = (
        (view_min.x / 16.0).floor() as i32 - 1 - 28,
        (view_min.y / 16.0).floor() as i32 - 1 - 28,
        (view_max.x / 16.0).floor() as i32 + 2 + 28,
        (view_max.y / 16.0).floor() as i32 + 2 + 28,
    );
    // `Main.UpdateSceneMetrics`'s capture gate (`Main.cs:62518-62521`):
    // `if (CaptureManager.Instance.IsCapturing) return;` — the metrics scan
    // freezes while a capture renders (the chunk draws read the parked
    // `Main.SceneMetrics`, not a live rescan). This is the DRAW-SIDE guard
    // (the lighting engines' ExportMetrics callers, LightingEngine.cs:93 /
    // LegacyLighting.cs:722). The UPDATE-SIDE freeze is the DoUpdate gate
    // (`Main.cs:16886-16890`, ported in frame.rs): while capturing the whole
    // world update — Sandstorm.EmitDust (:17247, dust RNG), SceneState.Update
    // (:17816), the sim — skips, so update_scene is not even reached from the
    // world branch. This guard keeps the metrics parked for any draw-side
    // caller that still runs. S1-E-17.
    let capturing = app.capture.is_capturing();
    let mut metrics = if capturing {
        app.scene_metrics.clone()
    } else {
        scan_scene_full(
            world,
            center,
            rain.is_raining(),
            sandstorm.happening,
            player.as_ref().is_some_and(|p| p.shimmer_wet),
            Some(visual_area),
            npcs,
        )
    };
    // `SceneMetrics.AddPlayerEffects` (`SceneMetrics.cs:747+`) plus the
    // `ScreenObstruction`/`ScreenDarkness` inputs. Vanilla reads them off
    // `PerspectivePlayer` / `Main.npc[]`; the port has neither field, so the
    // driver fills the metrics directly. These fills sit inside
    // `UpdateSceneMetrics` past the capture gate — frozen with the scan.
    if !capturing {
        if let Some(p) = player.as_ref() {
            // `Player.headcovered` (`Player.cs:11826-11829`): buff 163.
            metrics.headcovered = p.buffs.find_buff_index(163).is_some();
            // `Player.DoUnbreakableWallScan` (`Player.cs:28606-28619`) behind
            // the `Main.dualDungeonsSeed` gate (`:28608`).
            metrics.inside_unbreakable_walls = world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::DUAL_DUNGEONS)
                && tr_core::scene::inside_unbreakable_walls(
                    world,
                    (metrics.center[0] as i32) >> 4,
                    (metrics.center[1] as i32) >> 4,
                );
            // `DangerousDungeonCurse` (`DangerousDungeonCurse.cs:5-55`).
            metrics.curse_progress_needed = tr_core::scene::curse_progress_needed(
                metrics.zone_lihzhard_temple,
                metrics.zone_hallow,
                metrics.zone_dungeon,
                metrics.zone_jungle,
                metrics.zone_crimson || metrics.zone_corrupt,
            );
            metrics.curse_progress_safely = tr_core::scene::curse_progress_safely(
                world.downed_mech_boss_any() || world.downed_queen_slime,
                world.downed_skeletron || world.hard_mode,
                world.downed_queen_bee,
                // `NPC.downedBoss2` — the Eater of Worlds / Brain.
                world.downed_eow,
                // `NPC.downedSlimeKing || NPC.downedBoss1` (the Eye).
                world.downed_slime_king || world.downed_eoc,
            );
        }
        // `ScreenDarkness.Update`'s `Main.npc[]` walk (`:17-32`): only the two
        // kinds with an arm matter, and only while active. Duke Fishron (370)
        // is not a ported `NpcKind` yet, so only the Wall of Flesh row exists.
        metrics.screen_darkness_npcs = npcs
            .iter()
            .filter(|n| n.hp > 0 && matches!(n.kind, NpcKind::WallOfFlesh))
            .map(|n| {
                let (cx, cy) = n.center();
                tr_core::scene::ScreenDarknessNpc {
                    npc_type: 113,
                    center: [cx, cy],
                    // The C# fields this arm reads are `ai[0]`/`ai[2]` on the
                    // 370 row and `localAI[1]` on this one; the port folds the
                    // WoF abandonment ramp into `ai[0]` (`npc.rs:9160`).
                    ai0: n.ai[0],
                    ai2: n.ai[2],
                    local_ai1: n.ai[0],
                }
            })
            .collect();
    }
    // `Main.SceneState.Update` (`Main.cs:17816`); the `skipTransitions` the
    // manage calls read is the pre-update value — vanilla flips it at `:62`,
    // after `ApplyVisuals` inside `Update`.
    let skip = scene_state.skip_transitions;
    scene_state.update(&metrics);
    // M6-AUDIO-1: the SceneState blizzard loops' sound half
    // (`SceneState.cs:189-253`, inside `ApplyVisuals` — hence inside the
    // port's `SceneState::update` call). The gate is the raw
    // `metrics.ZoneRain && metrics.ZoneSnow` (`:80`); the cloud-alpha /
    // outside-weather split and the ±0.01 ramp are
    // `ambient_sound::blizzard_loop_volumes`. The whole call chain runs only
    // in game (the C# `SceneState.Update` is under `!gameMenu`, `:17802`), so
    // the loops freeze at their last volume in menus.
    //
    // S1-E-23: the loops ride the tracked-sound layer verbatim
    // (`SceneState.cs:197-247`) — the ensure-plays behind the dead
    // `SlotId`s, the split landing on the `ActiveSound.Volume` override,
    // and the `SlotId.Invalid` drops when the sound died at volume 0.
    let blizzard_on = metrics.zone_rain && metrics.zone_snow;
    let (strong_volume, inside_volume, blizzard_volume) =
        crate::ambient_sound::blizzard_loop_volumes(
            blizzard_on,
            rain.visual_alpha(),
            scene_state.outside_weather_effect_intensity,
            scene_state.blizzard_sound_volume,
        );
    scene_state.blizzard_sound_volume = blizzard_volume;
    // `:197-206`: while blizzard, both sounds exist (a dead slot re-plays).
    if blizzard_on {
        if app
            .audio
            .get_active_sound(scene_state.strong_blizzard_sound)
            .is_none()
        {
            scene_state.strong_blizzard_sound = app
                .audio
                .play_tracked_global(crate::ambient_sound::blizzard_strong_loop_style());
        }
        if app
            .audio
            .get_active_sound(scene_state.inside_blizzard_sound)
            .is_none()
        {
            scene_state.inside_blizzard_sound = app
                .audio
                .play_tracked_global(crate::ambient_sound::blizzard_inside_loop_style());
        }
    }
    // `:222-247`: the split as the volume override; a dead sound and a zero
    // volume drop the slot.
    if blizzard_volume > 0.0 {
        if app
            .audio
            .get_active_sound(scene_state.strong_blizzard_sound)
            .is_none()
        {
            scene_state.strong_blizzard_sound = app
                .audio
                .play_tracked_global(crate::ambient_sound::blizzard_strong_loop_style());
        }
        if let Some(strong) = app
            .audio
            .get_active_sound_mut(scene_state.strong_blizzard_sound)
        {
            strong.volume = strong_volume;
        }
        if app
            .audio
            .get_active_sound(scene_state.inside_blizzard_sound)
            .is_none()
        {
            scene_state.inside_blizzard_sound = app
                .audio
                .play_tracked_global(crate::ambient_sound::blizzard_inside_loop_style());
        }
        if let Some(inside) = app
            .audio
            .get_active_sound_mut(scene_state.inside_blizzard_sound)
        {
            inside.volume = inside_volume;
        }
    } else {
        // `:236-245`: mute the live instances, drop the dead slots.
        match app
            .audio
            .get_active_sound_mut(scene_state.strong_blizzard_sound)
        {
            Some(strong) => strong.volume = 0.0,
            None => scene_state.strong_blizzard_sound = tr_core::slot::SlotId::INVALID,
        }
        match app
            .audio
            .get_active_sound_mut(scene_state.inside_blizzard_sound)
        {
            Some(inside) => inside.volume = 0.0,
            None => scene_state.inside_blizzard_sound = tr_core::slot::SlotId::INVALID,
        }
    }
    // `Main.tileColor` = the sky tint applied to tiles
    // (`Main.SetBackColor` + `ApplyColorOfTheSkiesToTiles`), with the rain dim
    // the frame applies (`frame.rs`'s `rain_dim`). Only the Noir formula
    // reads it (no M1 activation path).
    let quantised = (world.time / 60.0).floor() * 60.0;
    let c = crate::render::sky::color_of_the_skies(quantised, world.day_time, world.moon_phase);
    let rain_dim = (1.0 - rain.visual_alpha() * 0.9).max(0.1);
    let mut c = [c[0] * rain_dim, c[1] * rain_dim, c[2] * rain_dim];
    // `ModifyHorizonLight` (`Main.cs:63921`) — the same blend the frame's
    // light-map sky colour takes, so the filters see the tinted tileColor.
    let atmo = crate::render::sky::atmosphere_factor(camera.y, world.world_surface, world.width);
    crate::render::sky::modify_horizon_light(
        &mut c,
        world.moon_phase,
        world.day_time,
        (world.time / 60.0).floor() * 60.0,
        atmo,
        rain.visual_alpha(),
    );
    let tile_color = crate::render::sky::tile_color(c);
    let inputs = crate::app::filters::SceneVisualInputs {
        storm: &*sandstorm,
        wind_speed_current: wind.speed_current,
        cloud_alpha: rain.visual_alpha(),
        dt,
        skip_transitions: skip,
        view_center: [camera.x, camera.y],
        screen: [screen.x, screen.y],
        zoom: *zoom,
        world_surface: world.world_surface,
        rock_layer: world.rock_layer,
        max_tiles_x: world.width,
        max_tiles_y: world.height,
        max_raining: rain.max_raining(),
        tile_color,
        bg_style: *bg_style,
        bg_delay: *bg_delay,
        global_time: *anim_time % 3600.0,
    };
    // The graphics gates follow the live menu toggles (`TR_*` pins override).
    filter_driver.sync_settings(settings);
    filter_driver.apply_visuals(&metrics, scene_state, &inputs);
    // W4-c40: the SkyManager half of `ManageSpecialBiomeVisuals("Sandstorm",
    // ...)` (`SceneState.cs:313-321`): `inZone != IsActive()` flips the sky.
    // The condition is the filter driver's `sandstorm_visuals_on` (the same
    // `:97` boolean the Sandstorm filter manage read).
    if filter_driver.sandstorm_visuals_on != app.sandstorm_sky.is_active() {
        if filter_driver.sandstorm_visuals_on {
            app.sandstorm_sky.activate();
        } else {
            app.sandstorm_sky.deactivate();
        }
    }
    // `Main.SceneMetrics` persists per frame (`Main.cs:1789`); the ambience
    // gates read its zone flags from the stored copy.
    app.scene_metrics = metrics;
}

#[cfg(test)]
mod tests {
    // W4-d129 census witnesses (written-not-run, code-only lane): the scan
    // window census, the RNG-free property, the aura frame edges, the banner
    // decode convention pin and the dungeon-zone gate.
    #[path = "scene_scan_w4_d129.rs"]
    mod scene_scan_w4_d129;

    /// A ground world with one aura tile set at `(x, y)`.
    fn aura_world(kind: u16, frame_x: i16, frame_y: i16) -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 100..200 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        let i = w.index(100, 95);
        w.tiles[i] = tr_core::Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_aura_scan_reads_the_frame_gates() {
        // `SceneMetrics.ScanTiles` (`:378-409`): the lit campfire, the three
        // candles and the lantern/star ranges ride their frame windows; the
        // wrong-frame tile does not count.
        // The lit campfire: 215 with frameY < 36.
        let m = scan_scene_at(
            &aura_world(215, 0, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(m.has_campfire);
        // An unlit campfire frame (>= 36) does not count.
        let m = scan_scene_at(
            &aura_world(215, 0, 54),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(!m.has_campfire);
        // The water candle: 49 with frameX < 18.
        let m = scan_scene_at(
            &aura_world(49, 0, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert_eq!(m.water_candle_count, 1);
        let m = scan_scene_at(
            &aura_world(49, 18, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert_eq!(m.water_candle_count, 0);
        // The peace/shadow candles count separately.
        let m = scan_scene_at(
            &aura_world(372, 0, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert_eq!(m.peace_candle_count, 1);
        let m = scan_scene_at(
            &aura_world(646, 0, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert_eq!(m.shadow_candle_count, 1);
        // The heart lantern range 324-358 vs the star range 252-286 on 42.
        let m = scan_scene_at(
            &aura_world(42, 0, 324),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(m.has_heart_lantern && !m.has_star_in_bottle);
        let m = scan_scene_at(
            &aura_world(42, 0, 252),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(m.has_star_in_bottle && !m.has_heart_lantern);
        // The snapshot the Player reads.
        let m = scan_scene_at(
            &aura_world(215, 0, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(m.auras().campfire);
    }

    #[test]
    fn w4_e28_the_bast_and_torch_campfire_scan_arms() {
        // W4-e28 census witnesses — `SceneMetrics.cs:394-401`: the torch
        // campfire (405, `frameX < 54`) and the Bast statue (506,
        // `frameX < 72`) raise the SAME aura flags the grants read.
        let m = scan_scene_at(
            &aura_world(506, 0, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(m.has_cat_bast);
        assert!(m.auras().cat_bast, "the snapshot the Player reads");
        // The unlit/extinguished frame (>= 72) does not count.
        let m = scan_scene_at(
            &aura_world(506, 72, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(!m.has_cat_bast);
        // The torch campfire 405 counts on frameX < 54.
        let m = scan_scene_at(
            &aura_world(405, 36, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(m.has_campfire && m.auras().campfire);
        let m = scan_scene_at(
            &aura_world(405, 54, 0),
            [100.0 * 16.0, 95.0 * 16.0],
            false,
            false,
            false,
        );
        assert!(!m.has_campfire);
    }

    #[test]
    fn w4_e28_the_aura_proximity_window_gates_the_scan() {
        // `ZoneScanSize` (`SceneMetrics.cs:917-919`): 169x124 tiles centred
        // on the player's tile — a lit campfire INSIDE the window raises the
        // aura, one OUTSIDE it does not.
        let mut w = aura_world(215, 0, 0);
        // Move the campfire out of the ±84-tile half-width (x 100 → 195).
        let i = w.index(100, 95);
        w.tiles[i] = tr_core::Tile::default();
        let i = w.index(195, 95);
        w.tiles[i] = tr_core::Tile {
            kind: 215,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let m = scan_scene_at(&w, [100.0 * 16.0, 95.0 * 16.0], false, false, false);
        assert!(!m.has_campfire, "95 tiles away is outside the 84-tile half");
        // Pull it back inside the window (x 150 — 50 tiles away).
        let i = w.index(195, 95);
        w.tiles[i] = tr_core::Tile::default();
        let i = w.index(150, 95);
        w.tiles[i] = tr_core::Tile {
            kind: 215,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let m = scan_scene_at(&w, [100.0 * 16.0, 95.0 * 16.0], false, false, false);
        assert!(m.has_campfire, "50 tiles away is inside the window");
    }

    #[test]
    fn the_sunflower_count_and_the_graveyard_cancel() {
        // `AggregateTileCounts` (`:617-624`): a sunflower (27) in the window
        // raises HasSunflower; a graveyard above GraveyardTileMin cancels it.
        let mut w = aura_world(27, 0, 0);
        let m = scan_scene_at(&w, [100.0 * 16.0, 95.0 * 16.0], false, false, false);
        assert!(m.has_sunflower);
        // Tombstones (85) past the min cancel the flower.
        for x in 90..120 {
            for y in 60..75 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 85,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        let m = scan_scene_at(&w, [100.0 * 16.0, 95.0 * 16.0], false, false, false);
        assert!(m.graveyard_tile_count > tr_core::scene::thresholds::GRAVEYARD_TILE_MIN);
        assert!(!m.has_sunflower);
    }

    use super::*;
    use tr_core::scene::{thresholds, Sandstorm, SceneState};
    use tr_core::tile::FLAG_ACTIVE;

    use crate::app::flow::front_flow_tests::flow_app;
    use crate::npc::NpcKind;

    /// A small test world with vanilla-shaped surface lines.
    fn fixture_world() -> World {
        let mut world = World::new("scene-scan-test".into(), 1000, 400);
        world.world_surface = 100.0;
        world.rock_layer = 180.0;
        world
    }

    fn set_tile(world: &mut World, x: i32, y: i32, kind: u16) {
        let tile = world.tile_mut(x, y).unwrap();
        tile.kind = kind;
        tile.flags |= FLAG_ACTIVE;
    }

    fn fill(world: &mut World, x0: i32, y0: i32, x1: i32, y1: i32, kind: u16) {
        for y in y0..y1 {
            for x in x0..x1 {
                set_tile(world, x, y, kind);
            }
        }
    }

    /// The player standing with its hitbox center exactly on tile `(tx, ty)`
    /// (the scan center is `x + WIDTH/2, y + HEIGHT/2`).
    fn player_at(world: &World, tx: i32, ty: i32) -> Player {
        let mut player = Player::spawn(world);
        player.x = tx as f32 * 16.0 - player::WIDTH * 0.5;
        player.y = ty as f32 * 16.0 - player::HEIGHT * 0.5;
        player
    }

    #[test]
    fn test_DW_1_1_snow_zone_counts() {
        let mut world = fixture_world();
        // 80x20 = 1600 snow tiles (id 147) fully inside the 169x124 window at
        // tile center (300, 50): window x 216..385, y 0..113.
        fill(&mut world, 260, 30, 340, 50, 147);
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_center.0, 300);
        assert_eq!(metrics.tile_center.1, 50);
        assert_eq!(metrics.tile_counts[147], 1600);
        assert_eq!(metrics.snow_tile_count, 1600);
        assert!(metrics.enough_tiles_for_snow());
        assert!(metrics.zone_snow);
        // Not raining, no sandstorm, no desert.
        assert!(!metrics.zone_rain);
        assert!(!metrics.zone_desert);
        assert!(!metrics.zone_sandstorm);
        // The block is a solid rectangle: every window tile of another kind
        // must be untouched (sanity: total active == placed).
        let total: u32 = metrics.tile_counts.iter().sum();
        assert_eq!(total, 1600);
    }

    #[test]
    fn test_DW_1_1_desert_zone_counts() {
        let mut world = fixture_world();
        // Sand far from both beach bands (380 left / 620 right on a 1000-wide
        // world) around the player at tile (500, 50): window x 416..585.
        fill(&mut world, 460, 30, 540, 50, 53);
        let player = player_at(&world, 500, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_counts[53], 1600);
        assert_eq!(metrics.sand_tile_count, 1600);
        assert_eq!(metrics.ocean_sand_tile_count, 0);
        assert_eq!(metrics.desert_sand_tile_count, 1600);
        assert!(metrics.zone_desert);
        assert!(metrics.surface_atmospherics);
    }

    #[test]
    fn beach_sand_is_subtracted_from_the_desert_count() {
        // Sand in the left beach band (x < 380, above oceanLevel=180) counts
        // as ocean sand: `DesertSandTileCount = Sand - OceanSand`
        // (`SceneMetrics.cs:377-379,658`) — a full beach window yields no
        // desert zone at all.
        let mut world = fixture_world();
        fill(&mut world, 60, 30, 140, 50, 53);
        let player = player_at(&world, 100, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.sand_tile_count, 1600);
        assert_eq!(metrics.ocean_sand_tile_count, 1600);
        assert_eq!(metrics.desert_sand_tile_count, 0);
        assert!(!metrics.zone_desert);
    }

    #[test]
    fn test_DW_1_1_rain_and_snow_both_true() {
        // `SceneState.cs:120` gates the blizzard on `ZoneRain && ZoneSnow` —
        // the flags must coexist.
        let mut world = fixture_world();
        fill(&mut world, 260, 30, 340, 50, 147);
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, true, false);
        assert!(metrics.zone_rain, "raining + on the surface");
        assert!(metrics.zone_snow);
    }

    #[test]
    fn sandstorm_zone_requires_desert_surface_and_happening() {
        let mut world = fixture_world();
        fill(&mut world, 460, 30, 540, 50, 53);
        let player = player_at(&world, 500, 50);
        let metrics = scan_scene(&world, &player, true, true);
        // `SceneMetrics.cs:703-707`: the storm zone wins over the rain zone.
        assert!(metrics.zone_sandstorm);
        assert!(!metrics.zone_rain);
        let metrics = scan_scene(&world, &player, true, false);
        assert!(!metrics.zone_sandstorm);
        assert!(metrics.zone_rain);
    }

    #[test]
    fn test_DW_1_1_tile_center_outside_world_empty_scan() {
        let world = fixture_world();
        // Far above-left of the world: the center clamps to (0, 0) and the
        // window is clipped to the world corner.
        let mut player = Player::spawn(&world);
        player.x = -5000.0;
        player.y = -5000.0;
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_center, (0, 0));
        let total: u32 = metrics.tile_counts.iter().sum();
        assert_eq!(total, 0);
        assert!(!metrics.zone_snow && !metrics.zone_desert && !metrics.zone_rain);
        assert!(metrics.surface_atmospherics); // clamped y=0 <= surface

        // Far below the world: the vanilla Y clamp uses maxTilesX (`Utils.cs:1859`,
        // latent bug ported 1:1), so the center lands at (999, 999) — past the
        // 400-tall world — and the whole window clamps away to nothing.
        let mut player = Player::spawn(&world);
        player.x = 50000.0;
        player.y = 50000.0;
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_center, (999, 999));
        let total: u32 = metrics.tile_counts.iter().sum();
        assert_eq!(total, 0);
        assert!(!metrics.surface_atmospherics);
        assert!(!metrics.behind_backwall);
    }

    #[test]
    fn test_DW_1_1_window_clamped_at_world_edge() {
        let mut world = fixture_world();
        // A wall of snow spanning the world's left edge; the player stands at
        // the first column, so the window's left half is cut off.
        fill(&mut world, 0, 30, 200, 50, 147);
        let player = player_at(&world, 0, 50);
        let metrics = scan_scene(&world, &player, false, false);
        // Window at tile center (0, 50): x clamps to 0..85 (85 columns), y to
        // 0..112 — the block's rows 30..50 are fully inside the y range, its
        // columns beyond 85 fall outside the window.
        assert_eq!(metrics.tile_counts[147], 85 * 20);
        assert_eq!(metrics.snow_tile_count, 85 * 20);
        assert!(metrics.zone_snow);
    }

    #[test]
    fn scan_counts_graveyard_minus_sunflowers() {
        // Tombstones (85) minus half the sunflowers (27)
        // (`SceneMetrics.cs:615-624`).
        let mut world = fixture_world();
        fill(&mut world, 260, 30, 280, 50, 85); // 400 tombstones
        fill(&mut world, 300, 30, 304, 50, 27); // 80 sunflowers
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert_eq!(metrics.tile_counts[85], 400);
        assert_eq!(metrics.tile_counts[27], 80);
        assert_eq!(metrics.graveyard_tile_count, 400 - 40);
        assert!(metrics.enough_tiles_for_graveyard());
    }

    #[test]
    fn behind_backwall_reads_the_center_tile_wall() {
        // `SceneMetrics.cs:671-672`: `GetTileSafely(TileCenter).wall > 0`.
        let mut world = fixture_world();
        let player = player_at(&world, 300, 50);
        let metrics = scan_scene(&world, &player, false, false);
        assert!(!metrics.behind_backwall);
        // Give the center tile a backwall without activating it.
        world.tile_mut(300, 50).unwrap().wall = 1;
        let metrics = scan_scene(&world, &player, false, false);
        assert!(metrics.behind_backwall);
    }

    #[test]
    fn test_DW_1_4_scene_forces_wire_through_the_glue() {
        // The env glue parses and applies each force; the lookup closure
        // stands in for the process environment.
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        let vars: Vec<(String, String)> = vec![
            ("TR_SANDSTORM".into(), "0.75".into()),
            ("TR_GRAVEYARD".into(), "0.4".into()),
            ("TR_BLOODMOON".into(), String::new()),
        ];
        apply_scene_forces(&mut state, &mut storm, |var| {
            vars.iter().find(|(k, _)| k == var).map(|(_, v)| v.clone())
        });
        assert_eq!(state.forced_severity, Some(0.75));
        assert_eq!(state.forced_graveyard, Some(0.4));
        assert!(state.blood_moon);
        assert!(storm.happening);
        assert_eq!(storm.severity, 0.75);

        // Junk values are ignored entirely.
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        apply_scene_forces(&mut state, &mut storm, |var| {
            (var == "TR_SANDSTORM").then(|| "garbage".to_string())
        });
        assert_eq!(state.forced_severity, None);
        assert!(!storm.happening);

        // No vars set: nothing changes.
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        apply_scene_forces(&mut state, &mut storm, |_| None);
        assert_eq!(state.forced_severity, None);
        assert_eq!(state.forced_graveyard, None);
        assert!(!state.blood_moon);
        assert!(!storm.happening);
    }

    #[test]
    fn thresholds_match_scene_metrics_statics() {
        // `SceneMetrics.cs:930,934,939-941`.
        assert_eq!(thresholds::SNOW_TILE_NORMAL, 1500);
        assert_eq!(thresholds::DESERT_TILE_NORMAL, 1500);
        assert_eq!(thresholds::GRAVEYARD_TILE_MIN, 16);
        assert_eq!(thresholds::GRAVEYARD_TILE_MAX, 36);
        assert_eq!(thresholds::GRAVEYARD_TILE, 28);
        // `SceneMetrics.cs:917-919`.
        assert_eq!(ZONE_SCAN_SIZE, (169, 124));
    }

    /// A headless App on a 1000x400 world with the camera (the TR_NOPLAYER
    /// scan center) on tile (300, 50) and 400 tombstones in the window.
    fn graveyard_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = fixture_world();
        fill(&mut app.world, 260, 30, 280, 50, 85); // 400 tombstones
        app.camera = Vec2::new(300.0 * 16.0, 50.0 * 16.0);
        app
    }

    const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    #[test]
    fn update_scene_scans_updates_state_and_drives_the_filter() {
        let mut app = graveyard_app();
        assert!(app.scene_state.skip_transitions, "fresh state = load frame");
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        // Scan -> SceneState::update: the tombstones drove the graveyard
        // intensity to its target and the skip flag flipped (`:62`).
        assert_eq!(app.scene_state.graveyard_visual_intensity, 1.0);
        assert!(!app.scene_state.skip_transitions);
        // -> driver: the graveyard filter won, centred on the scan centre.
        let out = app.filter_driver.output.expect("graveyard filter runs");
        assert_eq!(out.pass_index, tr_render::PassId::GRAVEYARD.0);
        assert_eq!(out.target_pos, [4800.0, 800.0], "metrics.center = camera");
        assert_eq!(out.screen_resolution, [1280.0, 720.0]);
    }

    /// The capture gate has TWO halves (`Main.cs:16886-16890` DoUpdate return
    /// + `:62518-62521` UpdateSceneMetrics return). While capturing, the whole
    /// world update freezes (EmitDust/SceneState.Update/sim skip — the DoUpdate
    /// gate, ported in frame.rs), AND any draw-side update_scene caller reads
    /// the parked metrics (this guard). This test pins the draw-side half:
    /// update_scene called directly while capturing keeps the parked metrics
    /// (no rescan) and still runs SceneState.Update + the driver against them.
    #[test]
    fn s1_e_17_capturing_freezes_the_scan_but_not_the_scene_update() {
        let mut app = graveyard_app();
        // Park a live graveyard scan first.
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        assert_eq!(app.scene_state.graveyard_visual_intensity, 1.0);
        let parked = app.scene_metrics.clone();
        // Start a capture (the queue machine's active flag is the gate).
        let settings = crate::capture::CaptureSettings {
            area: crate::capture::CaptureRect::new(0, 0, 10, 10),
            ..Default::default()
        };
        app.capture.capture(settings).unwrap();
        assert!(app.capture.is_capturing());
        // Remove the tombstones: a live rescan would drop the graveyard
        // zone; the frozen scan must keep the parked metrics.
        fill(&mut app.world, 260, 30, 280, 50, 0);
        app.scene_state.skip_transitions = true; // the load-frame snap flag
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        // The scan froze: the parked metrics survived the tile removal.
        assert_eq!(app.scene_metrics, parked, "no rescan while capturing");
        // SceneState.Update + the driver still ran (skip flipped, output fed).
        assert!(!app.scene_state.skip_transitions, "SceneState.Update ran");
        assert_eq!(
            app.filter_driver.output.expect("the driver ran").pass_index,
            tr_render::PassId::GRAVEYARD.0,
        );
    }

    #[test]
    fn update_scene_hands_the_pre_update_skip_flag_to_the_driver() {
        // The load frame snaps the fade (skip is read BEFORE `update` flips
        // it): opacity is already 1. A later activation fades in from 0.
        let mut app = graveyard_app();
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        assert_eq!(app.filter_driver.output.unwrap().opacity, 1.0, "snap");

        let mut app = graveyard_app();
        app.scene_state.skip_transitions = false; // not the load frame
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        let fading = app.filter_driver.output.map_or(0.0, |p| p.opacity);
        assert!(fading < 1.0, "no snap without skipTransitions: {fading}");
    }

    #[test]
    fn update_scene_ramps_the_blizzard_loop_emitter() {
        // M6-AUDIO-1: `SceneState.cs:189-253`'s sound half — a snowy, raining
        // surface scene ramps `_blizzardSoundVolume` toward 1 and splits it by
        // `min(1, cloudAlpha*2) * outsideWeatherEffectIntensity`; an ordinary
        // forest scene leaves the pair muted.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = fixture_world();
        // 169x124 window of snow (tile 147) around the camera, well over the
        // 1500-tile threshold (`SceneMetrics.cs:917-919`), on the surface.
        // The y span keeps clear of the world's top edge (the fixture's
        // `world_surface` is 100, the camera at 50).
        let (cx, cy) = (300, 50);
        fill(&mut app.world, cx - 90, cy - 40, cx + 90, cy + 40, 147);
        app.camera = Vec2::new(cx as f32 * 16.0, cy as f32 * 16.0);
        app.rain.set_raining_for_test(true);
        let mut blizzard = 0.0f32;
        for _ in 0..5 {
            update_scene(&mut app, 1.0 / 60.0, SCREEN);
            blizzard = app.scene_state.blizzard_sound_volume;
        }
        assert!(blizzard > 0.04 && blizzard < 0.06, "ramped {blizzard}");
        // And the pair mutes when the zone passes.
        app.rain.set_raining_for_test(false);
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        let off = app.scene_state.blizzard_sound_volume;
        assert!(off < blizzard, "the fade reverses: {off} < {blizzard}");
    }

    #[test]
    fn update_scene_without_conditions_runs_no_tint_filter() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = fixture_world();
        app.camera = Vec2::new(300.0 * 16.0, 50.0 * 16.0);
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        assert!(app.filter_driver.output.is_none());
        assert!(!app.scene_state.skip_transitions);
    }

    #[test]
    fn update_scene_feeds_the_driver_gates_from_settings() {
        let mut app = graveyard_app();
        app.settings.use_storm_effects = false;
        app.settings.use_heat_distortion = false;
        app.settings.wave_quality = 0;
        update_scene(&mut app, 1.0 / 60.0, SCREEN);
        // (A TR_* pin set in the test environment would override; unset here.)
        if !crate::env::present_raw("TR_STORM") {
            assert!(!app.filter_driver.use_storm_effects);
        }
        if !crate::env::present_raw("TR_HEAT") {
            assert!(!app.filter_driver.use_heat_distortion);
        }
        if !crate::env::present_raw("TR_WAVES") {
            assert_eq!(app.filter_driver.wave_quality, 0);
        }
    }

    #[test]
    fn scan_npc_positions_counts_the_town_rect_and_keeps_the_closest() {
        // `SceneMetrics.cs:727-745`: `TownNPCRectSize` is the ±1920x1200 box
        // around `Center`; a resident outside it does not count, and per-type
        // slots keep the nearest `Center`.
        let world = fixture_world();
        let center = [500.0 * 16.0, 50.0 * 16.0];
        let mut near = Npc::new(NpcKind::Merchant, center[0] + 10.0, center[1], 1, 0.0);
        near.hp = near.max_hp;
        // The half-extent is 1920 px: +2000 px is outside, +1900 inside.
        let mut far = Npc::new(NpcKind::Nurse, center[0] + 2000.0, center[1], 1, 0.0);
        far.hp = far.max_hp;
        let mut edge = Npc::new(NpcKind::Dryad, center[0] + 1900.0, center[1], 1, 0.0);
        edge.hp = edge.max_hp;
        // A non-town NPC (zombie) rides the closest table but not the count.
        let mut mob = Npc::new(NpcKind::Zombie, center[0] - 100.0, center[1], 1, 0.0);
        mob.hp = mob.max_hp;
        let metrics = scan_scene_full(
            &world,
            center,
            false,
            false,
            false,
            None,
            &[near.clone(), far.clone(), edge.clone(), mob.clone()],
        );
        assert_eq!(metrics.town_npc_count, 2, "the merchant and the dryad");
        let merchant = metrics.closest_npc_position[NpcKind::Merchant.net_id() as usize];
        assert_eq!(merchant[0], near.center().0);
        let nurse = metrics.closest_npc_position[NpcKind::Nurse.net_id() as usize];
        assert_eq!(
            nurse[0],
            far.center().0,
            "still recorded outside the town rect"
        );
        // A dead NPC contributes nothing.
        let mut dead = Npc::new(NpcKind::Guide, center[0], center[1], 1, 0.0);
        dead.hp = 0;
        let metrics = scan_scene_full(&world, center, false, false, false, None, &[dead]);
        assert_eq!(metrics.town_npc_count, 0);
        assert_eq!(
            metrics.closest_npc_position[NpcKind::Guide.net_id() as usize],
            [0.0, 0.0]
        );
    }

    #[test]
    fn scan_on_screen_tiles_reads_the_clock_fountain_and_monoliths() {
        // `SceneMetrics.cs:450-577`: the visual-area pass.
        let mut world = fixture_world();
        // The clock (104) and the monoliths live outside the 169x124 zone
        // window but inside the visual rect drawn by the caller: the pass is
        // driven by `area`, not by the scan window.
        set_tile(&mut world, 300, 50, 104);
        set_tile(&mut world, 301, 50, 207);
        world.tile_mut(301, 50).unwrap().frame_y = 72; // flowing
        world.tile_mut(301, 50).unwrap().frame_x = 36 * 1; // fountain colour 12
        set_tile(&mut world, 302, 50, 410);
        world.tile_mut(302, 50).unwrap().frame_y = 56;
        world.tile_mut(302, 50).unwrap().frame_x = 36 * 2; // monolith type 2
        set_tile(&mut world, 303, 50, 480);
        world.tile_mut(303, 50).unwrap().frame_y = 54;
        set_tile(&mut world, 304, 50, 658);
        world.tile_mut(304, 50).unwrap().frame_y = 54; // shimmer state 1
        set_tile(&mut world, 305, 50, 139);
        world.tile_mut(305, 50).unwrap().frame_x = 36;
        world.tile_mut(305, 50).unwrap().frame_y = 36 * 7; // music box 7
        let center = [300.0 * 16.0, 50.0 * 16.0];
        let metrics = scan_scene_full(
            &world,
            center,
            false,
            false,
            false,
            Some((290, 40, 320, 60)),
            &[],
        );
        assert!(metrics.has_clock);
        assert_eq!(metrics.active_fountain_color, 12);
        assert_eq!(metrics.active_monolith_type, 2);
        assert!(metrics.blood_moon_monolith);
        assert_eq!(metrics.shimmer_monolith_state, 1);
        assert_eq!(metrics.active_music_box, 7);
        // No visual area = the pass does not run (the C#'s
        // `settings.VisualScanArea` is `HasValue`-gated, `:338`).
        let metrics = scan_scene_full(&world, center, false, false, false, None, &[]);
        assert!(!metrics.has_clock);
        assert_eq!(metrics.active_fountain_color, -1);
        // A fountain with a non-flowing frame leaves the colour alone.
        let mut world = fixture_world();
        set_tile(&mut world, 300, 50, 207);
        world.tile_mut(300, 50).unwrap().frame_y = 0;
        let metrics = scan_scene_full(
            &world,
            center,
            false,
            false,
            false,
            Some((290, 40, 320, 60)),
            &[],
        );
        assert_eq!(metrics.active_fountain_color, -1);
    }

    #[test]
    fn the_ore_finder_keeps_the_highest_priority_nearest_ore() {
        // `SceneMetrics.cs:867-903`: `UpdateOreFinder` from `ScanTiles`; the
        // priority table is `Main.tileOreFinderPriority` (`Main.cs:6404-6440`)
        // — copper 7 is 200, iron 6 is 220, gold 8 is 260.
        let mut world = fixture_world();
        let center = [300.0 * 16.0, 50.0 * 16.0];
        // Far gold vs near copper: the priority wins over the distance.
        set_tile(&mut world, 300, 50, 7);
        set_tile(&mut world, 320, 50, 8);
        let metrics = scan_scene_full(&world, center, false, false, false, None, &[]);
        assert_eq!(metrics.best_ore_type, 8);
        assert_eq!(metrics.best_ore_position, (320, 50));
        // Two equal-priority ores: the nearer one wins.
        let mut world = fixture_world();
        set_tile(&mut world, 300, 50, 7);
        set_tile(&mut world, 310, 50, 7);
        let metrics = scan_scene_full(&world, center, false, false, false, None, &[]);
        assert_eq!(metrics.best_ore_position, (300, 50));
        // `IsValidForOreFinder`: tile 227 outside frames 272..=374 is skipped.
        let mut world = fixture_world();
        set_tile(&mut world, 300, 50, 227);
        world.tile_mut(300, 50).unwrap().frame_x = 100;
        let metrics = scan_scene_full(&world, center, false, false, false, None, &[]);
        assert_eq!(metrics.best_ore_type, -1);
        world.tile_mut(300, 50).unwrap().frame_x = 300;
        let metrics = scan_scene_full(&world, center, false, false, false, None, &[]);
        assert_eq!(metrics.best_ore_type, 227);
    }
}
