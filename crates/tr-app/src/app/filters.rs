//! The screen-filter driver — `SceneState.ApplyVisuals`'s port
//! (`SceneState.cs:64-253`, M1 subset). Per frame it steps the filter board's
//! fades (the `Filters.Scene.Update` half, `Main.cs:17041`), evaluates the
//! `ManageSpecialBiomeVisuals` conditions, refreshes the storm shader state
//! and assembles the winning filter's uniform for the post pass. The pure
//! state machines (board, storm shaders, parameter formulas) live in
//! `tr_core::scene`; this module owns the uniform assembly because
//! `FilterParams` is a render type.
//!
//! The C# routine reads `Main.*` statics; the port passes them per frame
//! through [`SceneVisualInputs`]. The `Noir` parameter block is ported and
//! unit-tested but has no M1 activation path (the monolith is M4 content);
//! HeatDistortion/WaterDistortion/Aurora are phase-4/sky-manager work.

use tr_core::scene::{
    FilterBoard, FilterId, Sandstorm, SceneMetrics, SceneState, blizzard_foreground_params,
    blood_moon_opacity, graveyard_progress, is_desert_bg_style, noir_image_offset, noir_intensity,
    sandstorm_background_opacity, sandstorm_foreground_params,
};
use tr_render::FilterParams;

/// The `Main.*` statics and per-frame values `ApplyVisuals` reads
/// (`SceneState.cs:64-253`).
pub(crate) struct SceneVisualInputs<'a> {
    /// `Sandstorm` (the event state; `Severity` feeds the storm parameters).
    pub(crate) storm: &'a Sandstorm,
    /// `Main.windSpeedCurrent` (`SandstormShaderData.cs:12`).
    pub(crate) wind_speed_current: f32,
    /// `Main.cloudAlpha` — the Blizzard block's `num8` (`SceneState.cs:169`).
    pub(crate) cloud_alpha: f32,
    /// Frame elapsed — the board's 1/s fades and the storm texture scroll.
    pub(crate) dt: f32,
    /// The `SceneState.skipTransitions` value captured before
    /// `SceneState::update` (vanilla reads it inside `Update` before the
    /// `:62` flip) — the first-frame snap.
    pub(crate) skip_transitions: bool,
    /// `Main.screenPosition.Y + Main.screenHeight / 2` — the view centre the
    /// BloodMoon depth probe reads (`BloodMoonScreenShaderData.cs:15`), in
    /// world pixels. Also the uniform's view anchor.
    pub(crate) view_center: [f32; 2],
    /// The view size in screen pixels and the render zoom — the uniform's
    /// `uScreenResolution = UnscaledScreenSize` (`ScreenShaderData.cs:96`).
    pub(crate) screen: [f32; 2],
    pub(crate) zoom: f32,
    /// `Main.worldSurface` / `Main.rockLayer` / `Main.maxTilesY`.
    pub(crate) world_surface: f64,
    pub(crate) rock_layer: f64,
    pub(crate) max_tiles_y: i32,
    /// `Main.tileColor` — the frame's tile tint; only the Noir formula reads
    /// its brightness (`SceneState.cs:117`).
    pub(crate) tile_color: [f32; 3],
    /// `Main.bgStyle` / `Main.bgDelay` — `Sandstorm.ShowSandstormVisuals`'s
    /// desert-variant gate (`Sandstorm.cs:131-138`).
    pub(crate) bg_style: i32,
    pub(crate) bg_delay: i32,
    /// `Main.GlobalTimeWrappedHourly` (`ScreenShaderData.cs:146`).
    pub(crate) global_time: f32,
}

/// The `ApplyVisuals` driver: filter board + storm shader state + the last
/// winner uniform. Lives on `App`; `world_sim::scene_scan::update_scene`
/// calls [`FilterDriver::apply_visuals`] once per drawn frame, and the post
/// branch consumes [`FilterDriver::output`].
pub(crate) struct FilterDriver {
    /// The `Filters.Scene`/`Overlays.Scene` M1 entries (actives + fades).
    pub(crate) board: FilterBoard,
    /// `SandstormShaderData` state (direction + scrolling texture position).
    sandstorm_fx: tr_core::scene::StormShader,
    /// `BlizzardShaderData` state (smoothed wind + scroll).
    blizzard_fx: tr_core::scene::StormShader,
    /// `Main.UseStormEffects` (`Main.cs:429`): the graphics toggle the
    /// Sandstorm/Blizzard gates read (`SceneState.cs:96-97`). Retail default
    /// is **false** — the filters show only with the option enabled;
    /// `TR_STORM=1` pins it for deterministic shots.
    pub(crate) use_storm_effects: bool,
    /// The winning filter's uniform this frame (`None` = no filter runs: the
    /// post pass applies the passthrough, DW-2.1's identity).
    pub(crate) output: Option<FilterParams>,
}

impl FilterDriver {
    /// A fresh driver. `TR_STORM=1` pins `use_storm_effects` for screenshots
    /// (the vanilla graphics toggle lives in the settings UI this build
    /// does not have yet).
    pub(crate) fn new() -> Self {
        Self {
            board: FilterBoard::new(),
            sandstorm_fx: tr_core::scene::StormShader::new(),
            blizzard_fx: tr_core::scene::StormShader::new(),
            use_storm_effects: std::env::var_os("TR_STORM").is_some(),
            output: None,
        }
    }

    /// `SceneState.ApplyVisuals` (`SceneState.cs:64-253`), phase subset.
    /// Contract (plan `**Produces:**`): returns the active filter's
    /// parameters, `None` when nothing runs. The `pass_index` field carries
    /// the WGSL pass — the post branch forwards it to `apply_filter`.
    pub(crate) fn apply_visuals(
        &mut self,
        metrics: &SceneMetrics,
        state: &mut SceneState,
        inputs: &SceneVisualInputs<'_>,
    ) -> Option<FilterParams> {
        // `Main.Update` runs `Filters.Scene.Update(gameTime)` (`Main.cs:17041`)
        // before `UpdateWorld`'s `SceneState.Update` (`:17816`): the fades step
        // first, then the conditions and parameters.
        self.board.update(inputs.dt);
        // The storm shaders' `Update` runs every frame their filter sits in
        // the active list — `FilterManager.Update` → `Filter.Update` →
        // `_shader.Update` (`FilterManager.cs:105`, `Filter.cs:20-24`) —
        // including during fade-out.
        if self.board.in_list(FilterId::Sandstorm) {
            self.sandstorm_fx
                .update_sandstorm(inputs.wind_speed_current, inputs.dt);
        }
        if self.board.in_list(FilterId::Blizzard) {
            self.blizzard_fx
                .update_blizzard(inputs.wind_speed_current, inputs.dt);
        }

        // The `ManageSpecialBiomeVisuals` subset, in the source's call order
        // (the activation order is the same-priority tie-break). The tower/
        // monolith/MoonLord rows (`:81-91`), Aurora (`:95`), HeatDistortion
        // (`:99`), Sepia/Noir/CRT/Test2 (`:101-104`) and WaterDistortion
        // (`:105`) have no M1 activation reader.
        // `:92`: `Main.bloodMoon || BloodMoonMonolith || bloodMoonMonolithShader`.
        self.board.manage(
            FilterId::BloodMoon,
            state.blood_moon,
            inputs.skip_transitions,
        );
        // `:96`: `Main.UseStormEffects & (ZoneRain && ZoneSnow)` — the Filter
        // and its overlay twin are one manage call in vanilla.
        let blizzard_zone = metrics.zone_rain && metrics.zone_snow;
        let blizzard_on = self.use_storm_effects && blizzard_zone;
        self.board
            .manage(FilterId::Blizzard, blizzard_on, inputs.skip_transitions);
        self.board.manage(
            FilterId::BlizzardOverlay,
            blizzard_on,
            inputs.skip_transitions,
        );
        // `:97`: `Main.UseStormEffects && Sandstorm.ShowSandstormVisuals()`.
        let sandstorm_on = self.use_storm_effects
            && inputs.storm.show_sandstorm_visuals(
                metrics.zone_sandstorm,
                is_desert_bg_style(inputs.bg_style),
                inputs.bg_delay,
            );
        self.board
            .manage(FilterId::Sandstorm, sandstorm_on, inputs.skip_transitions);
        self.board.manage(
            FilterId::SandstormOverlay,
            sandstorm_on,
            inputs.skip_transitions,
        );
        // `:100`: `Main.GraveyardVisualIntensity > 0`.
        self.board.manage(
            FilterId::Graveyard,
            state.graveyard_visual_intensity > 0.0,
            inputs.skip_transitions,
        );

        // The parameter updates (`:107-188`) apply to shader objects that keep
        // their values; only the winner's uniform is rendered, so the driver
        // rebuilds it fresh each frame. The one observable freeze — storm
        // `Use*` persisting after their condition dies — reproduces
        // implicitly: the inputs move slowly relative to the 1 s fade (ANALYSIS).
        let winner = self.board.winner();
        self.output = winner.map(|id| self.build_params(id, metrics, state, inputs));
        self.output
    }

    /// Assembles the winner's uniform: the shared `Apply` parameters
    /// (`ScreenShaderData.cs:139-187`) plus the per-filter block.
    fn build_params(
        &mut self,
        id: FilterId,
        metrics: &SceneMetrics,
        state: &mut SceneState,
        inputs: &SceneVisualInputs<'_>,
    ) -> FilterParams {
        let mut params = FilterParams::NEUTRAL;
        // `uTime = GlobalTimeWrappedHourly` (`:146`).
        params.time = inputs.global_time;
        // `uScreenResolution = UnscaledScreenSize` (`:96`) and
        // `uScreenPosition = UnscaledScreenPosition - offScreenRange` (`:148`);
        // the gameplay draw zeroes `offScreenRange` (`Main.cs:56687`) and the
        // zoom-matrix delta it also subtracts has no counterpart here.
        params.screen_resolution = [
            inputs.screen[0] / inputs.zoom,
            inputs.screen[1] / inputs.zoom,
        ];
        params.screen_position = [
            inputs.view_center[0] - params.screen_resolution[0] * 0.5,
            inputs.view_center[1] - params.screen_resolution[1] * 0.5,
        ];
        // `CombinedOpacity = uOpacity × global opacity` (`:84`): the board's
        // fade multiplies every filter's own `UseOpacity` value.
        params.opacity = self.board.opacity(id);

        match id {
            FilterId::Graveyard => {
                // `SceneState.cs:107-114` (`:113`'s UseIntensity(1.2) is a
                // constant; `:109`'s progress ramps with the intensity).
                params.pass_index = tr_render::PassId::GRAVEYARD.0;
                params.target_pos = [metrics.center[0], metrics.center[1]];
                params.progress = graveyard_progress(state.graveyard_visual_intensity);
                params.intensity = 1.2;
            }
            FilterId::Noir => {
                // `SceneState.cs:115-125`. Unreachable in M1 (no monolith) —
                // kept complete so the pass stays faithful once the reader
                // lands.
                params.pass_index = tr_render::PassId::NOIR.0;
                params.target_pos = [metrics.center[0], metrics.center[1]];
                params.intensity = noir_intensity(
                    inputs.tile_color,
                    inputs.world_surface as i32,
                    metrics.tile_center.1,
                    inputs.max_tiles_y - 200, // `Main.UnderworldLayer`, Main.cs:2790
                );
                params.image_offset = noir_image_offset(inputs.global_time);
            }
            FilterId::BloodMoon => {
                // `ScreenEffectInitializer.cs:27`'s UseColor(2, -0.8, -0.6) +
                // `BloodMoonScreenShaderData.Update`'s depth opacity (`:15,24`).
                params.pass_index = tr_render::PassId::BLOOD_MOON.0;
                params.color = [2.0, -0.8, -0.6, 0.0];
                let depth = inputs.view_center[1] / 16.0;
                params.opacity *=
                    blood_moon_opacity(depth, inputs.world_surface, inputs.rock_layer);
            }
            FilterId::Sandstorm | FilterId::SandstormOverlay => {
                // Initializer colors (`ScreenEffectInitializer.cs:30-33`).
                params.color = [1.1, 1.0, 0.5, 0.0];
                params.secondary_color = [0.7, 0.5, 0.3, 0.0];
                let severity = inputs.storm.severity;
                let weather = state.outside_weather_effect_intensity;
                let (fg_intensity, fg_shader) = sandstorm_foreground_params(severity, weather);
                let bg_shader = sandstorm_background_opacity(severity, weather);
                // The post pass cannot draw behind the scene and the plan pins
                // one filter: run whichever side of the vanilla cross-fade
                // currently dominates (foreground in the open, background
                // behind backwalls). The overlay's intensity stays at the
                // initializer's 0.4 — `ApplyVisuals` never re-sets it.
                let fg = fg_shader * self.board.opacity(FilterId::Sandstorm);
                let bg = bg_shader * self.board.opacity(FilterId::SandstormOverlay);
                if fg >= bg {
                    params.pass_index = tr_render::PassId::SANDSTORM_FOREGROUND.0;
                    params.intensity = fg_intensity;
                    params.opacity = fg;
                } else {
                    params.pass_index = tr_render::PassId::SANDSTORM_BACKGROUND.0;
                    params.intensity = 0.4;
                    params.opacity = bg;
                }
                // Both storm shaders force `UseTargetPosition(_texturePosition)`
                // at Apply time (`SandstormShaderData.cs:25-29`).
                params.target_pos = self.sandstorm_fx.texture_pos();
                params.direction = self.sandstorm_fx.direction();
                params.image_scale = [1.0, 1.0];
            }
            FilterId::Blizzard | FilterId::BlizzardOverlay => {
                // Initializer colors + image scale (`:34-39`).
                params.color = [1.0, 1.0, 1.0, 0.0];
                params.secondary_color = [0.7, 0.7, 1.0, 0.0];
                params.image_scale = [3.0, 0.75];
                let weather = state.outside_weather_effect_intensity;
                let (fg_intensity, fg_shader, bg_shader) =
                    blizzard_foreground_params(inputs.cloud_alpha, weather, 0.0);
                let fg = fg_shader * self.board.opacity(FilterId::Blizzard);
                let bg = bg_shader * self.board.opacity(FilterId::BlizzardOverlay);
                if fg >= bg {
                    params.pass_index = tr_render::PassId::BLIZZARD_FOREGROUND.0;
                    params.intensity = fg_intensity;
                    params.opacity = fg;
                } else {
                    params.pass_index = tr_render::PassId::BLIZZARD_BACKGROUND.0;
                    params.intensity = 0.4;
                    params.opacity = bg;
                }
                params.target_pos = self.blizzard_fx.texture_pos();
                params.direction = self.blizzard_fx.direction();
            }
        }
        // `uImageSize[1] = image dims × scale` (`ScreenShaderData.cs:183`);
        // the noise image is 256×256 (`Images/Misc/noise`).
        params.image_size = [256.0 * params.image_scale[0], 256.0 * params.image_scale[1]];
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::scene::SceneMetrics;

    /// Default inputs: the view centre sits at (1600, 400) world px, a
    /// 1280×720 view at zoom 1, surface 100 / rock 180, a 400-tall world.
    fn inputs_with(storm: &Sandstorm) -> SceneVisualInputs<'_> {
        SceneVisualInputs {
            storm,
            wind_speed_current: 0.0,
            cloud_alpha: 0.0,
            dt: 1.0 / 60.0,
            skip_transitions: false,
            view_center: [1600.0, 400.0],
            screen: [1280.0, 720.0],
            zoom: 1.0,
            world_surface: 100.0,
            rock_layer: 180.0,
            max_tiles_y: 400,
            tile_color: [1.0, 1.0, 1.0],
            bg_style: 0,
            bg_delay: 0,
            global_time: 12.5,
        }
    }

    #[test]
    fn test_driver_uniform_fill_matches_apply() {
        // `ScreenShaderData.Apply` (`:139-187`): the shared parameters every
        // pass receives, filled from the inputs.
        let mut metrics = SceneMetrics::new();
        metrics.center = [1600.0, 400.0];
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm);
        let mut driver = FilterDriver::new();
        // Activate the graveyard by pinning the intensity; the first call is
        // the world-load (skip) frame, so the fade snaps to 1.
        state.graveyard_visual_intensity = 1.0;
        inputs.skip_transitions = true;
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("graveyard active");
        assert!((params.time - 12.5).abs() < 1e-6, "uTime");
        assert_eq!(
            params.screen_resolution,
            [1280.0, 720.0],
            "uScreenResolution = screen / zoom"
        );
        assert_eq!(
            params.screen_position,
            [1600.0 - 640.0, 400.0 - 360.0],
            "uScreenPosition = view top-left"
        );
        assert_eq!(params.image_size, [256.0, 256.0], "noise dims × scale");
        // The graveyard block (`SceneState.cs:107-114`).
        assert_eq!(params.pass_index, tr_render::PassId::GRAVEYARD.0);
        assert_eq!(
            params.target_pos,
            [1600.0, 400.0],
            "UseTargetPosition(Center)"
        );
        assert!((params.progress - 0.75).abs() < 1e-6, "Lerp(0, 0.75, 1.0)");
        assert_eq!(params.intensity, 1.2, "UseIntensity(1.2)");
        assert_eq!(params.opacity, 1.0, "no UseOpacity call, fade snapped");
    }

    #[test]
    fn test_DW_3_1_blood_moon_params_matrix_and_depth() {
        let metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm);
        let mut driver = FilterDriver::new();
        state.force("BLOODMOON", 1.0);
        inputs.skip_transitions = true; // the world-load frame snaps the fade
        // The view centre tile y = 400/16 = 25 — above worldSurface + 50 →
        // full opacity 0.75.
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("blood moon active");
        assert_eq!(params.pass_index, tr_render::PassId::BLOOD_MOON.0);
        assert_eq!(params.color, [2.0, -0.8, -0.6, 0.0], "initializer :27");
        assert!((params.opacity - 0.75).abs() < 1e-6);

        // Descend past rockLayer + 100: the depth term drains the opacity.
        inputs.view_center[1] = 300.0 * 16.0;
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("still the winner while fading");
        assert!(params.opacity < 0.01, "deep underground the tint fades");
    }

    #[test]
    fn test_DW_3_3_sandstorm_foreground_background_split() {
        let mut metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        storm.force_severity(0.8);
        let mut inputs = inputs_with(&storm);
        let mut driver = FilterDriver::new();
        driver.use_storm_effects = true;
        state.outside_weather_effect_intensity = 1.0;
        inputs.bg_style = 2; // a desert bg variant
        inputs.skip_transitions = true; // the world-load frame snaps the fade
        inputs.bg_delay = 0;
        // The zone needs a desert-surface scan with the storm happening.
        metrics.zone_desert = true;
        metrics.zone_sandstorm = true;
        metrics.surface_atmospherics = true;

        // In the open the FOREGROUND filter dominates.
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("storm active");
        assert_eq!(params.pass_index, tr_render::PassId::SANDSTORM_FOREGROUND.0);
        assert_eq!(params.color, [1.1, 1.0, 0.5, 0.0]);
        assert_eq!(params.secondary_color, [0.7, 0.5, 0.3, 0.0]);
        let (intensity, opacity) = sandstorm_foreground_params(0.8, 1.0);
        assert!((params.intensity - intensity).abs() < 1e-6);
        assert!((params.opacity - opacity).abs() < 1e-6);
        // The scroll rides the shader state (uTargetPosition).
        assert_eq!(params.target_pos, driver.sandstorm_fx.texture_pos());

        // Behind a backwall the weather intensity drains → the BACKGROUND
        // overlay pass takes over (its opacity is the complement).
        state.outside_weather_effect_intensity = 0.0;
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("storm still active");
        assert_eq!(params.pass_index, tr_render::PassId::SANDSTORM_BACKGROUND.0);
        let bg = sandstorm_background_opacity(0.8, 0.0);
        assert!((params.opacity - bg).abs() < 1e-6);
        assert!(
            (params.intensity - 0.4).abs() < 1e-6,
            "initializer intensity"
        );

        // Without the graphics toggle (retail default) nothing activates.
        let mut driver = FilterDriver::new();
        driver.use_storm_effects = false;
        state.outside_weather_effect_intensity = 1.0;
        assert!(
            driver
                .apply_visuals(&metrics, &mut state, &inputs)
                .is_none()
        );
    }

    #[test]
    fn test_DW_3_3_blizzard_params_and_split() {
        let mut metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm);
        let mut driver = FilterDriver::new();
        driver.use_storm_effects = true;
        state.outside_weather_effect_intensity = 1.0;
        inputs.skip_transitions = true; // the world-load frame snaps the fade
        inputs.cloud_alpha = 0.5;
        metrics.zone_rain = true;
        metrics.zone_snow = true;

        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("blizzard active");
        assert_eq!(params.pass_index, tr_render::PassId::BLIZZARD_FOREGROUND.0);
        assert_eq!(params.color, [1.0, 1.0, 1.0, 0.0]);
        assert_eq!(params.secondary_color, [0.7, 0.7, 1.0, 0.0]);
        assert_eq!(params.image_scale, [3.0, 0.75], "initializer :36");
        assert_eq!(params.image_size, [768.0, 192.0], "dims × scale (`:183`)");
        let (intensity, fg, _bg) = blizzard_foreground_params(0.5, 1.0, 0.0);
        assert!((params.intensity - intensity).abs() < 1e-6);
        assert!((params.opacity - fg).abs() < 1e-6);
        assert_eq!(params.target_pos, driver.blizzard_fx.texture_pos());

        // Behind a backwall the background overlay wins.
        state.outside_weather_effect_intensity = 0.0;
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("blizzard still active");
        assert_eq!(params.pass_index, tr_render::PassId::BLIZZARD_BACKGROUND.0);
        assert!((params.opacity - 1.0).abs() < 1e-6, "1 - num9 at ow=0");
    }

    #[test]
    fn test_DW_3_4_driver_switch_crossfade() {
        // A storm dying while the graveyard persists: the winner swaps from
        // the storm pass to the graveyard pass, each at its own fade opacity.
        let mut metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let mut storm = Sandstorm::new(1);
        storm.force_severity(0.8);
        let mut inputs = inputs_with(&storm);
        let mut driver = FilterDriver::new();
        driver.use_storm_effects = true;
        metrics.zone_desert = true;
        metrics.zone_sandstorm = true;
        metrics.surface_atmospherics = true;
        inputs.bg_style = 2;
        state.graveyard_visual_intensity = 1.0;

        // Both activate on the first (skip) frame — the world-load frame
        // vanilla snaps via `skipTransitions` (`SceneState.cs:355-365`); the
        // storm (High) wins at once.
        inputs.skip_transitions = true;
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("storm active");
        assert_eq!(params.pass_index, tr_render::PassId::SANDSTORM_FOREGROUND.0);
        assert!((params.opacity - 1.0).abs() < 1e-6, "skip frame snaps");
        inputs.skip_transitions = false;

        // The storm ends: the zone clears, the storm slot fades out over 1 s
        // while the graveyard (still active) waits; the storm stays the winner
        // until it drains (the f32 fade tail can linger a frame past 60 —
        // vanilla accumulates the same way).
        metrics.zone_sandstorm = false;
        metrics.zone_desert = false;
        // The deactivation frame itself still renders at full opacity (the
        // board's fade update runs before the manage call, vanilla order).
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("storm still the winner");
        assert_eq!(params.pass_index, tr_render::PassId::SANDSTORM_FOREGROUND.0);
        let params = driver
            .apply_visuals(&metrics, &mut state, &inputs)
            .expect("storm fading");
        assert!(params.opacity < 1.0, "fading out");

        // Once the storm's opacity drains, the graveyard pass takes over at
        // full strength.
        let mut params = None;
        for _ in 0..70 {
            params = driver.apply_visuals(&metrics, &mut state, &inputs);
            if params.map_or(false, |p| p.pass_index == tr_render::PassId::GRAVEYARD.0) {
                break;
            }
        }
        let params = params.expect("graveyard takes over");
        assert_eq!(params.pass_index, tr_render::PassId::GRAVEYARD.0);
        assert!((params.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn driver_without_conditions_is_identity() {
        // No forces, no zones, no graveyard: nothing activates and the post
        // branch falls back to the passthrough (DW-2.1's identity holds).
        let metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let inputs = inputs_with(&storm);
        let mut driver = FilterDriver::new();
        assert!(
            driver
                .apply_visuals(&metrics, &mut state, &inputs)
                .is_none()
        );
        assert!(driver.output.is_none());
    }
}
