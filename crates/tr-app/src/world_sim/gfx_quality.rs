//! The gfx-quality gates (S1-B-6) — `Main.gfxQuality`'s adaptive loop
//! (`Main.cs:16960-17001`), the `qaStyle` override arms (`:17056-17065`) and
//! the per-update budgets they feed: `Main.maxDustToDraw` (`:17066-17073`),
//! `Gore.goreTime` (`:17075`), `Liquid.cycles`/`curMaxLiquid` (`:17077-17091`
//! — the port's `LiquidSystem::cycles`/`cur_max_liquid`, S1-A-8's runtime
//! pair), `LegacyLighting.RenderPhases` (`:17085-17101` — the field rides
//! for S1-E-19's legacy engine) and `Main.mapTimeMax` (`:16995-17000`). The
//! snow-dust budget gate (`:13057`) lives on the rain rig's spawn cap
//! ([`snow_cap`]).

use crate::app::App;

/// `Main.gfxQuality` + its adaptive-loop state (`Main.cs:953`/`:955`).
#[derive(Debug)]
pub(crate) struct GfxQuality {
    /// `Main.gfxQuality` — starts at the C#'s 0 and adapts to the frame rate.
    pub quality: f32,
    /// `Main.gfxRate` — the climb step, self-reinforcing on good fps.
    pub rate: f32,
    /// `Main.fpsCount`/`fpsTimer.ElapsedMilliseconds` (`:16962-16964`).
    fps_count: u32,
    fps_timer: f32,
    /// `Main.maxDustToDraw` (`:17066-17073`).
    pub max_dust_to_draw: i32,
    /// `Gore.goreTime` (`:17075`) — the gore system is S1-E-1's; the value
    /// rides here until then.
    pub gore_time: i32,
    /// `LegacyLighting.RenderPhases` (`:17085-17101`) — the legacy engine's
    /// consumer is S1-E-19's.
    pub render_phases: i32,
    /// `Main.mapTimeMax` (`:16995-17000`).
    pub map_time_max: i32,
}

impl Default for GfxQuality {
    fn default() -> Self {
        Self {
            quality: 0.0,
            rate: 0.01,
            fps_count: 0,
            fps_timer: 0.0,
            max_dust_to_draw: 0,
            gore_time: 0,
            render_phases: 8,
            map_time_max: 0,
        }
    }
}

/// The snow-dust budget the spawn loop caps against —
/// `Main.cs:13057`'s `num2 * (gfxQuality / 2f + 0.5f) + num2 * 0.1f` over
/// the port's per-screen snow budget (`rain.rs`'s `MAX_SNOW`).
pub(crate) fn snow_cap(max_snow: usize, quality: f32) -> usize {
    let num2 = max_snow as f32;
    (num2 * (quality / 2.0 + 0.5) + num2 * 0.1) as usize
}

impl GfxQuality {
    /// The per-frame half of the adaptive loop
    /// (`Main.cs:16961-16964`'s counter, the per-second block
    /// `:16965-17001`): count, and once a second's worth of frames
    /// accumulated, run the fps arms.
    pub fn count_frame(&mut self, dt: f32, fps_counted: u32) {
        self.fps_count += fps_counted;
        self.fps_timer += dt;
        if self.fps_timer < 1.0 {
            return;
        }
        self.fps_timer -= 1.0;
        let fps = self.fps_count as f32;
        self.fps_count = 0;
        // `:16965-16974`: the climb arm and the drop arm.
        if fps >= 30.0 + 30.0 * self.quality {
            self.quality += self.rate;
            self.rate += 0.005;
        } else if fps < 29.0 + 30.0 * self.quality {
            self.rate = 0.01;
            self.quality -= 0.1;
        }
        // `:16975-16981` — the [0, 1] clamp.
        self.quality = self.quality.clamp(0.0, 1.0);
        // `:16995-17000` — the map refresh budget.
        self.map_time_max = if self.quality < 0.8 {
            ((1.0 - self.quality) * 60.0) as i32
        } else {
            0
        };
    }

    /// The per-update gates (`Main.cs:17056-17101` — the `Update` block after
    /// `DoUpdate_AnimateBackgrounds`; `WorldGen.isGeneratingOrLoadingWorld`'s
    /// liquid gate is the generation-side exclusion the port's sim never
    /// runs under). The `maxQ` arm (`:16984-16987`) has no port writer.
    pub fn apply(&mut self, app: &mut App) {
        // `:17056-17065` — the qaStyle overrides.
        match app.settings.quality {
            1 => self.quality = 1.0,
            2 => self.quality = 0.5,
            3 => self.quality = 0.0,
            _ => {}
        }
        // `:17066-17073` — `Main.maxDustToDraw`.
        let mut max_dust = (6000.0 * (self.quality * 0.7 + 0.3)) as i32;
        if (self.quality as f64) < 0.9 {
            max_dust = (max_dust as f32 * self.quality) as i32;
        }
        if max_dust < 1000 {
            max_dust = 1000;
        }
        self.max_dust_to_draw = max_dust;
        // `:17075` — `Gore.goreTime`.
        self.gore_time = (600.0 * self.quality) as i32;
        // `:17077-17091` — the liquid budgets (the runtime pair S1-A-8 walks).
        // `Liquid.quickSettle`'s full-window arm (`:17098-17104`) rides
        // R-11-7's flag — the port's settlement pass keeps it `false`, so the
        // quality values hold.
        app.liquids.set_cycles((17.0 - 10.0 * self.quality) as usize);
        let max_liquid = app.liquids.max_active();
        app.liquids
            .set_cur_max_liquid((max_liquid as f64 * 0.25 + max_liquid as f64 * 0.75 * self.quality as f64) as usize);
        // `:17085-17101` — the legacy light scanner's render phases.
        self.render_phases = if (self.quality as f64) < 0.2 {
            8
        } else if (self.quality as f64) < 0.4 {
            7
        } else if (self.quality as f64) < 0.6 {
            6
        } else if (self.quality as f64) < 0.8 {
            5
        } else {
            4
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S1-B-6 — the per-update budgets at the qaStyle pins
    /// (`Main.cs:17056-17101`): the dust floor 1000, the liquid pair
    /// (`17 - 10q` / `maxLiquid * 0.25 + maxLiquid * 0.75q`), the render
    /// phases' thresholds and `Gore.goreTime`'s 600 scale.
    #[test]
    fn s1_b_6_the_quality_budgets_follow_the_csharp_formulas() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut gfx = GfxQuality::default();
        // The `qaStyle` overrides (`:17056-17065`).
        app.settings.quality = 1;
        gfx.apply(&mut app);
        assert_eq!(gfx.quality, 1.0);
        assert_eq!(gfx.max_dust_to_draw, 6000, "the full-quality dust cap");
        assert_eq!(app.liquids.cycles(), 7, "`17 - 10 * 1` (`:17078`)");
        assert_eq!(
            app.liquids.cur_max_liquid(),
            app.liquids.max_active(),
            "the full-quality liquid window"
        );
        assert_eq!(gfx.render_phases, 4, "the >= 0.8 phases (`:17085-17101`)");
        assert_eq!(gfx.gore_time, 600);
        assert_eq!(gfx.map_time_max, 0, "the map refresh lives above 0.8");

        app.settings.quality = 2;
        gfx.apply(&mut app);
        assert_eq!(gfx.quality, 0.5);
        // `6000 * (0.5 * 0.7 + 0.3)` truncates to 3899 in the C#'s f32 math,
        // then `* 0.5` (q < 0.9) truncates again to 1949.
        assert_eq!(gfx.max_dust_to_draw, 1949);
        assert_eq!(app.liquids.cycles(), 12, "`17 - 10 * 0.5`");
        assert_eq!(
            app.liquids.cur_max_liquid(),
            (app.liquids.max_active() as f64 * 0.25
                + app.liquids.max_active() as f64 * 0.75 * 0.5) as usize,
            "the half-quality liquid window"
        );
        assert_eq!(gfx.render_phases, 6, "the 0.4..0.6 phases");
        assert_eq!(gfx.gore_time, 300);
        // `mapTimeMax` is the per-second block's own product (`:16995-17000`)
        // — the qaStyle arms do not touch it; the adaptive test below pins
        // the formula through `count_frame`.

        app.settings.quality = 3;
        gfx.apply(&mut app);
        assert_eq!(gfx.quality, 0.0);
        // `6000 * 0.3 = 1800`, then `* 0` — floored at 1000 (`:17071-17073`).
        assert_eq!(gfx.max_dust_to_draw, 1000, "the dust floor");
        assert_eq!(app.liquids.cycles(), 17, "`17 - 0`");
        assert_eq!(
            app.liquids.cur_max_liquid(),
            app.liquids.max_active() / 4,
            "the quarter liquid window"
        );
        assert_eq!(gfx.render_phases, 8, "the < 0.2 phases");
        assert_eq!(gfx.gore_time, 0);
    }

    /// S1-B-6 — the adaptive loop (`Main.cs:16965-17001`): 60 fps climbs
    /// `gfxQuality` by the self-reinforcing rate (clamped at 1), a low fps
    /// drops it by 0.1 and resets the rate.
    #[test]
    fn s1_b_6_the_adaptive_loop_climbs_and_drops() {
        let mut gfx = GfxQuality::default();
        // Sixty 60 fps seconds: the climb arm fires per second.
        for _ in 0..60 {
            gfx.count_frame(1.0, 60);
        }
        assert!(
            (gfx.quality - 1.0).abs() < 1e-6,
            "60 fps saturates the quality ({})",
            gfx.quality
        );
        // A 10 fps second drops 0.1 and resets the rate (`:16970-16974`).
        gfx.quality = 1.0;
        gfx.rate = 0.1;
        gfx.count_frame(1.0, 10);
        assert!((gfx.quality - 0.9).abs() < 1e-6, "{}", gfx.quality);
        assert_eq!(gfx.rate, 0.01, "the drop resets the rate");
        // The map budget rides the same per-second block (`:16995-17000`):
        // 0 at the saturation, `(1 - q) * 60` after the drop.
        assert_eq!(gfx.map_time_max, 0, "the map refresh lives above 0.8");
        gfx.count_frame(1.0, 10);
        assert!((gfx.quality - 0.8).abs() < 1e-6, "{}", gfx.quality);
        assert_eq!(gfx.map_time_max, (int_ceil((1.0 - gfx.quality) * 60.0)), "the (1 - q) * 60 budget");
    }

/// `as i32` truncation the C# casts the budget with.
fn int_ceil(v: f32) -> i32 {
    v as i32
    }

    /// S1-B-6 — the snow budget (`Main.cs:13057`): the cap scales down with
    /// the quality and only bites below ~0.8 on the port's 600 budget.
    #[test]
    fn s1_b_6_the_snow_cap_scales_with_quality() {
        let full = snow_cap(600, 1.0);
        let half = snow_cap(600, 0.5);
        let zero = snow_cap(600, 0.0);
        assert_eq!(full, 600 + 60, "`600 * 1.0 + 60` at q=1");
        assert_eq!(half, 510, "`600 * 0.75 + 60` at q=0.5");
        assert_eq!(zero, 360, "`600 * 0.5 + 60` at q=0");
        assert!(half < full && zero < half, "low quality flakes fewer");
    }
}
