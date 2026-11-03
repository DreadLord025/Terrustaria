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
    EffectPriority, FilterBoard, FilterId, Sandstorm, SceneMetrics, SceneState, WaterShader,
    blizzard_foreground_params, blood_moon_opacity, graveyard_progress, heat_distortion_active,
    heat_distortion_intensity, heat_zone_flags, is_desert_bg_style, noir_image_offset,
    noir_intensity, sandstorm_background_opacity, sandstorm_foreground_params,
    water_distortion_active, water_distortion_intensity,
};
use tr_render::{FilterParams, FilterPass, PassId};

/// `Main.WaveQuality` default (`Main.cs:66571`, the static constructor).
const DEFAULT_WAVE_QUALITY: i32 = 3;

/// `TR_WAVES=<0..3>`: pins `Main.WaveQuality` (clamped to its load range,
/// `Main.cs:4439-4446`); unset/garbage keeps the retail default 3.
fn parse_wave_quality(raw: Option<&str>) -> i32 {
    raw.and_then(|v| v.trim().parse::<i32>().ok())
        .map_or(DEFAULT_WAVE_QUALITY, |q| q.clamp(0, 3))
}

/// `TR_HEAT=0|1`: pins `Main.UseHeatDistortion` (default true,
/// `Main.cs:66573`); only an explicit `0` turns it off.
fn parse_use_heat(raw: Option<&str>) -> bool {
    raw.map_or(true, |v| v.trim() != "0")
}

/// `Player.sunScorchCounter` — set by the sun-scorch debuff, which has no M1
/// source; the HeatDistortion branches that read it (`SceneState.cs:98,146`)
/// take 0.
const SUN_SCORCH_COUNTER: i32 = 0;

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
    /// `Main.worldSurface` / `Main.rockLayer` / `Main.maxTilesX` / `Main.maxTilesY`.
    pub(crate) world_surface: f64,
    pub(crate) rock_layer: f64,
    pub(crate) max_tiles_x: i32,
    pub(crate) max_tiles_y: i32,
    /// `Main.maxRaining` — the WaterDistortion rain term (`SceneState.cs:129`).
    pub(crate) max_raining: f32,
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
    /// `Main.WaveQuality` (`Main.cs:427`, default 3): `> 0` activates
    /// WaterDistortion (`SceneState.cs:105`). `TR_WAVES=<0..3>` pins it (no
    /// settings UI yet).
    pub(crate) wave_quality: i32,
    /// `Main.UseHeatDistortion` (`Main.cs:431`, default true); `TR_HEAT=0`
    /// turns it off.
    pub(crate) use_heat_distortion: bool,
    /// `WaterShaderData` state: the progress clock + last `UseIntensity`.
    water_fx: WaterShader,
    /// `Filters.Scene["HeatDistortion"].GetShader().Intensity` — the
    /// initializer's `UseIntensity(4f)` (`ScreenEffectInitializer.cs:41`)
    /// until `ApplyVisuals` sets it, then it persists while inactive.
    heat_intensity: f32,
    /// The winning tint filter's uniform this frame (`None` = no tint runs).
    /// Distortions do not compete for it — see [`FilterDriver::chain`].
    pub(crate) output: Option<FilterParams>,
    /// The board id behind `output` (its priority orders the chain).
    winner: Option<FilterId>,
    /// HeatDistortion's uniform while its filter is visible.
    pub(crate) heat: Option<FilterParams>,
    /// WaterDistortion's uniform while its filter is visible.
    pub(crate) water: Option<FilterParams>,
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
            wave_quality: parse_wave_quality(std::env::var("TR_WAVES").ok().as_deref()),
            use_heat_distortion: parse_use_heat(std::env::var("TR_HEAT").ok().as_deref()),
            water_fx: WaterShader::new(),
            heat_intensity: 4.0,
            output: None,
            winner: None,
            heat: None,
            water: None,
        }
    }

    /// The pass chain for this frame, in vanilla stack order
    /// (`FilterManager.EndCapture`, `FilterManager.cs:154-200`: ascending
    /// `EffectPriority`): HeatDistortion (Low), the tint winner (Medium/High),
    /// WaterDistortion (VeryHigh). Empty = no filter (the caller runs the
    /// passthrough). `water_in_view` = the liquid pass drew at least one rect;
    /// without water the distortion is the identity by construction, so its
    /// pass (and the mask target) is skipped.
    pub(crate) fn chain(&self, water_in_view: bool) -> Vec<FilterPass> {
        let mut items: Vec<(EffectPriority, FilterPass)> = Vec::with_capacity(3);
        let mut push = |priority, params: FilterParams, pass: PassId| {
            items.push((priority, FilterPass { params, pass }));
        };
        if let Some(p) = self.heat {
            push(
                FilterId::HeatDistortion.priority(),
                p,
                PassId::HEAT_DISTORTION,
            );
        }
        if let (Some(p), Some(id)) = (self.output, self.winner) {
            push(id.priority(), p, PassId(p.pass_index));
        }
        if let (Some(p), true) = (self.water, water_in_view) {
            push(
                FilterId::WaterDistortion.priority(),
                p,
                PassId::WATER_DISTORTION,
            );
        }
        // Stable: equal priorities keep insertion order.
        items.sort_by_key(|(priority, _)| *priority);
        items.into_iter().map(|(_, pass)| pass).collect()
    }

    /// Whether the WaterDistortion filter is visible this frame (active or
    /// fading, not hidden) — the frame renders the water mask only then.
    pub(crate) fn water_wanted(&self) -> bool {
        self.water.is_some()
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
        // `WaterShaderData.Update` (`WaterShaderData.cs:117-134`) rides the
        // same list walk; its clock uses the previous frame's UseIntensity.
        if self.board.in_list(FilterId::WaterDistortion) {
            self.water_fx.update(inputs.dt);
        }
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
        // monolith/MoonLord rows (`:81-91`), Aurora (`:95`) and
        // Sepia/Noir/CRT/Test2 (`:101-104`) have no M1 activation reader.
        let (heat_underworld, heat_desert) = heat_zone_flags(
            metrics.tile_center.1,
            inputs.max_tiles_y,
            inputs.world_surface,
            metrics.zone_desert,
            metrics.zone_rain,
            metrics.zone_sandstorm,
        );
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
        // `:98-99`: `UseHeatDistortion & ((flag2 | flag3) || sunScorch > 0)`.
        self.board.manage(
            FilterId::HeatDistortion,
            heat_distortion_active(
                self.use_heat_distortion,
                heat_underworld,
                heat_desert,
                SUN_SCORCH_COUNTER,
            ),
            inputs.skip_transitions,
        );
        // `:100`: `Main.GraveyardVisualIntensity > 0`.
        self.board.manage(
            FilterId::Graveyard,
            state.graveyard_visual_intensity > 0.0,
            inputs.skip_transitions,
        );
        // `:105`: `Main.WaveQuality > 0`.
        self.board.manage(
            FilterId::WaterDistortion,
            water_distortion_active(self.wave_quality),
            inputs.skip_transitions,
        );
        // `:126-133`: the WaterDistortion `UseIntensity` (the shader keeps it
        // after deactivation, so the fade-out reuses the last value).
        if self.board.is_active(FilterId::WaterDistortion) {
            self.water_fx.intensity = water_distortion_intensity(
                metrics.tile_center,
                inputs.max_tiles_x,
                inputs.max_tiles_y,
                inputs.wind_speed_current,
                inputs.max_raining,
                inputs.world_surface,
                inputs.rock_layer,
            );
        }
        // `:142-162`: HeatDistortion's intensity and `IsHidden = intensity <= 0`.
        if self.board.is_active(FilterId::HeatDistortion) {
            self.heat_intensity = heat_distortion_intensity(
                SUN_SCORCH_COUNTER,
                heat_underworld,
                heat_desert,
                metrics.tile_center.1,
                inputs.max_tiles_y,
                state.outside_weather_effect_intensity,
                inputs.cloud_alpha,
                inputs.tile_color,
            );
            self.board
                .set_hidden(FilterId::HeatDistortion, self.heat_intensity <= 0.0);
        }

        // The parameter updates (`:107-188`) apply to shader objects that keep
        // their values; only the winner's uniform is rendered, so the driver
        // rebuilds it fresh each frame. The one observable freeze — storm
        // `Use*` persisting after their condition dies — reproduces
        // implicitly: the inputs move slowly relative to the 1 s fade (ANALYSIS).
        self.winner = self.board.winner();
        self.output = self
            .winner
            .map(|id| self.build_params(id, metrics, state, inputs));
        // The distortions chain on top of the tint winner (vanilla stacks every
        // visible filter, `FilterManager.EndCapture`); see `chain`.
        self.heat = self.board.is_visible(FilterId::HeatDistortion).then(|| {
            // `UseImage("Images/Misc/noise")`, `UseIntensity(4f)` base
            // (`ScreenEffectInitializer.cs:41`); the intensity is the
            // `:160` value.
            let mut p = Self::base_params(inputs);
            p.intensity = self.heat_intensity;
            p.opacity = self.board.opacity(FilterId::HeatDistortion);
            p
        });
        self.water = self.board.is_visible(FilterId::WaterDistortion).then(|| {
            // `WaterShaderData.Apply` (`WaterShaderData.cs:361-376`): progress
            // (`UseProgress(_progress)`) and the intensity above. The
            // target-position/image-offset terms align `Main.waterTarget` and
            // the ripple buffer with the screen; the mask is view-aligned and
            // the ripple buffer is the neutral `_noDistortionTexture`, so both
            // stay zero (ANALYSIS: the entity-wave buffer is not ported).
            let mut p = Self::base_params(inputs);
            p.intensity = self.water_fx.intensity;
            p.progress = self.water_fx.progress;
            p.opacity = self.board.opacity(FilterId::WaterDistortion);
            p.target_pos = [0.0, 0.0];
            p
        });
        self.output
    }

    /// The `ScreenShaderData.Apply` parameters every pass receives
    /// (`ScreenShaderData.cs:139-187`): time, view size/position and the
    /// noise image size at scale 1.
    fn base_params(inputs: &SceneVisualInputs<'_>) -> FilterParams {
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
        // `uImageSize[1] = image dims × scale` (`:183`); `Images/Misc/noise`
        // is 256x256.
        params.image_size = [256.0, 256.0];
        params
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
        let mut params = Self::base_params(inputs);
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
            // Never tint winners (`FilterBoard::winner` pools `FILTERS`):
            // their uniforms are built in `apply_visuals`.
            FilterId::HeatDistortion | FilterId::WaterDistortion => {}
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
            max_tiles_x: 1000,
            max_tiles_y: 400,
            max_raining: 0.0,
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
    fn test_DW_4_3_driver_water_params_follow_apply_visuals() {
        // `SceneState.cs:126-133` + `WaterShaderData.cs:117-134,361-376`.
        let mut metrics = SceneMetrics::new();
        metrics.tile_center = (500, 100);
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm);
        inputs.wind_speed_current = 0.4;
        inputs.max_raining = 0.5;
        let mut driver = FilterDriver::new();
        driver.wave_quality = 3;
        inputs.skip_transitions = true;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        let expected = water_distortion_intensity((500, 100), 1000, 400, 0.4, 0.5, 100.0, 180.0);
        let water = driver
            .water
            .expect("WaveQuality 3 activates WaterDistortion");
        assert!((water.intensity - expected).abs() < 1e-6, "UseIntensity");
        assert_eq!(water.opacity, 1.0, "skip frame snaps the fade");
        assert_eq!(water.progress, 0.0, "not in the active list before frame 1");
        assert_eq!(water.target_pos, [0.0, 0.0], "mask is view-aligned");
        assert_eq!(water.image_size, [256.0, 256.0]);
        assert_eq!(water.screen_resolution, [1280.0, 720.0]);
        // Next frame: `_progress += dt * Intensity * 0.75`.
        inputs.skip_transitions = false;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        let water = driver.water.expect("still active");
        let want = (1.0 / 60.0) * expected * 0.75;
        assert!(
            (water.progress - want).abs() < 1e-7,
            "{} vs {want}",
            water.progress
        );
        // WaterDistortion never claims the tint slot.
        assert!(driver.output.is_none());
    }

    #[test]
    fn test_DW_4_4_wave_quality_zero_drops_the_water_pass() {
        let metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm);
        inputs.skip_transitions = true;
        let mut driver = FilterDriver::new();
        driver.wave_quality = 0;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        assert!(
            driver.water.is_none(),
            "WaveQuality 0 deactivates the filter"
        );
        assert!(!driver.water_wanted());
        assert!(
            driver.chain(true).is_empty(),
            "no pass even with water in view"
        );

        // With the default quality, water out of view still runs no pass
        // (identity by construction), and water in view runs exactly one.
        let mut driver = FilterDriver::new();
        driver.wave_quality = 3;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        assert!(driver.water_wanted());
        assert!(
            driver.chain(false).is_empty(),
            "no water in view -> no pass"
        );
        let chain = driver.chain(true);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].pass, PassId::WATER_DISTORTION);
        assert_eq!(
            chain[0].params.pass_index, 0,
            "apply_filters stamps the index"
        );
    }

    #[test]
    fn test_DW_4_1_driver_heat_underworld_and_hidden() {
        // `SceneState.cs:98-99,142-162`.
        let mut metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm); // 400 tall world: underworld Y > 80
        inputs.skip_transitions = true;
        let mut driver = FilterDriver::new();
        driver.wave_quality = 0;
        metrics.tile_center = (500, 140); // (140 - 80) / 120 = 0.5 -> 1.0
        driver.apply_visuals(&metrics, &mut state, &inputs);
        let heat = driver.heat.expect("underworld activates HeatDistortion");
        assert!((heat.intensity - 1.0).abs() < 1e-6);
        assert_eq!(heat.opacity, 1.0);
        assert_eq!(driver.chain(false).len(), 1);
        assert_eq!(driver.chain(false)[0].pass, PassId::HEAT_DISTORTION);

        // Deeper: the ramp tops out at 2.
        inputs.skip_transitions = false;
        metrics.tile_center = (500, 300);
        driver.apply_visuals(&metrics, &mut state, &inputs);
        assert!((driver.heat.expect("active").intensity - 2.0).abs() < 1e-6);

        // The desert-surface branch under a cloudy sky has intensity 0:
        // `IsHidden = true` removes the pass although the filter is active.
        let mut driver = FilterDriver::new();
        driver.wave_quality = 0;
        metrics.zone_desert = true;
        metrics.tile_center = (500, 50);
        inputs.cloud_alpha = 0.6;
        inputs.skip_transitions = true;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        assert!(driver.board.is_active(FilterId::HeatDistortion));
        assert!(driver.heat.is_none(), "hidden at intensity 0");
        // Clear sky: 1 * 4 * 0.5 * brightness(1) = 2.
        inputs.cloud_alpha = 0.0;
        inputs.skip_transitions = false;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        assert!((driver.heat.expect("visible again").intensity - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_DW_4_1_heat_toggle_and_env_pins() {
        let mut metrics = SceneMetrics::new();
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm);
        inputs.skip_transitions = true;
        metrics.tile_center = (500, 300);
        let mut driver = FilterDriver::new();
        driver.use_heat_distortion = false; // TR_HEAT=0 / the graphics toggle
        driver.apply_visuals(&metrics, &mut state, &inputs);
        assert!(driver.heat.is_none());
        // Pins: `Main.WaveQuality` defaults to 3 and clamps to 0..=3; the heat
        // toggle defaults on.
        assert_eq!(parse_wave_quality(None), 3);
        assert_eq!(parse_wave_quality(Some("0")), 0);
        assert_eq!(parse_wave_quality(Some(" 2 ")), 2);
        assert_eq!(parse_wave_quality(Some("9")), 3);
        assert_eq!(parse_wave_quality(Some("-4")), 0);
        assert_eq!(parse_wave_quality(Some("x")), 3);
        assert!(parse_use_heat(None));
        assert!(parse_use_heat(Some("1")));
        assert!(!parse_use_heat(Some("0")));
    }

    #[test]
    fn test_DW_4_1_chain_order_heat_tint_water() {
        // Vanilla stack order = ascending priority: Low heat, Medium tint,
        // VeryHigh water (`ScreenEffectInitializer.cs:41-42`,
        // `FilterManager.EndCapture`).
        let mut metrics = SceneMetrics::new();
        metrics.tile_center = (500, 300);
        let mut state = SceneState::new();
        let storm = Sandstorm::new(1);
        let mut inputs = inputs_with(&storm);
        inputs.skip_transitions = true;
        state.force("BLOODMOON", 1.0);
        let mut driver = FilterDriver::new();
        driver.wave_quality = 3;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        let order: Vec<u32> = driver.chain(true).iter().map(|p| p.pass.0).collect();
        assert_eq!(
            order,
            [8, PassId::BLOOD_MOON.0, 9],
            "heat, blood moon, water"
        );
        // Without water in view the tint and heat remain.
        let order: Vec<u32> = driver.chain(false).iter().map(|p| p.pass.0).collect();
        assert_eq!(order, [8, PassId::BLOOD_MOON.0]);
        // A High-priority storm still sorts below the water pass.
        let mut storm = Sandstorm::new(1);
        storm.force_severity(0.8);
        let mut inputs = inputs_with(&storm);
        inputs.skip_transitions = true;
        inputs.bg_style = 2;
        metrics.zone_desert = true;
        metrics.zone_sandstorm = true;
        metrics.surface_atmospherics = true;
        metrics.tile_center = (500, 50);
        let mut driver = FilterDriver::new();
        driver.use_storm_effects = true;
        driver.wave_quality = 3;
        driver.apply_visuals(&metrics, &mut state, &inputs);
        let order: Vec<u32> = driver.chain(true).iter().map(|p| p.pass.0).collect();
        assert_eq!(*order.last().unwrap(), 9, "water is last");
        assert!(order.contains(&PassId::SANDSTORM_FOREGROUND.0));
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
