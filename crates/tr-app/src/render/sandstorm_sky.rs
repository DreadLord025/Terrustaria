//! W4-c40 — the sandstorm sky tint (`SandstormSky : CustomSky`,
//! `GameContent/Skies/SandstormSky.cs:10-79`). The `SkyManager`'s "Sandstorm"
//! sky activates/deactivates on the `SceneState.cs:97` edge
//! (`ManageSpecialBiomeVisuals("Sandstorm", Main.UseStormEffects &&
//! Sandstorm.ShowSandstormVisuals())` → `:313-321`'s `inZone != IsActive()`
//! flip) — the same edge the port's filter board already evaluates
//! (`app/filters.rs`'s `sandstorm_on`), so the sky rides that boolean.
//!
//! C# sources read WHOLE (the golden rule):
//! - `Update` (`:24-47`): the `FocusHelper.PauseSkies` early return, then
//!   `_opacity -= dt` while leaving (below 0 → `_isActive = false`,
//!   `_opacity = 0`) else `_opacity += dt` capped at 1.
//! - `Draw` (`:49-56`): the `minDepth < 1f || maxDepth == float.MaxValue`
//!   band gate, then ONE full-screen `MagicPixel` quad at
//!   `new Color(new Vector4(0.85f, 0.66f, 0.33f, 1f) * 0.8f *
//!   Main.ColorOfTheSkies.ToVector4()) * _opacity * min(1, Severity * 1.5)`.
//! - `Activate` (`:58-62`): `_isActive = true`, `_isLeaving = false` (the
//!   opacity is NOT reset — a re-activation mid-fade keeps the fade).
//! - `Deactivate` (`:64-67`): `_isLeaving = true` only.
//! - `Reset` (`:69-73`): `_opacity = 0`, `_isActive = false` (the leaving
//!   latch is NOT cleared — verbatim).
//! - `IsActive` (`:75-78`): the flag the manager's update loop removes the
//!   sky on (`SkyManager.cs:39-42`).
//!
//! The manager halves ride the port's existing seams: the update is one
//! `SkyManager.Update` iteration (`SkyManager.cs:24-46`'s `max(1, dayRate)`
//! loop — the caller loops, `ambience_phase`), the draw is one
//! `DrawDepthRange` member (`SkyManager.cs:62-68`) at the sky-depth ladder's
//! `DrawToDepth(1f)` boundary (`Main.cs:60186`, the port's
//! `draw_range_min(..., 1.0, ...)` slot in `render/sky.rs`). The quad's
//! `destinationRectangle` is `(0, 0, screenWidth, screenHeight)` — screen
//! space, NOT the ±2 pad the `ScreenDarkness` family uses.
//!
//! The `_random` field (`:12`) is never drawn from in 1.4.5.8 — no port
//! stream. ZERO RNG draws anywhere in this sky (witnessed).
//!
//! Deliberate folds (ANALYSIS "The sandstorm sky tint (W4-c40)"):
//! - `FocusHelper.PauseSkies` (`:26`) — the port's update rides the same
//!   `!paused` ambience phase gate (the f40 lantern-sky fold).
//! - the `new Color(Vector4)` byte round-trip — the C# `Color` ctor clamps
//!   each channel to 0..255 bytes; the port keeps the float product (the
//!   renderer's tint is float; the ≤1/255 quantisation is the house
//!   float-tint convention, `render/screen_fx.rs`).

use tr_render::SpriteRenderer;

/// `SandstormSky : CustomSky` (`SandstormSky.cs:10-79`).
pub(crate) struct SandstormSky {
    /// `_isActive` (`:14`).
    active: bool,
    /// `_isLeaving` (`:16`).
    leaving: bool,
    /// `_opacity` (`:18`).
    opacity: f32,
}

impl Default for SandstormSky {
    fn default() -> Self {
        Self { active: false, leaving: false, opacity: 0.0 }
    }
}

impl SandstormSky {
    /// `Activate` (`:58-62`): the flags only — the opacity survives a
    /// re-activation (the C# never touches it here).
    pub(crate) fn activate(&mut self) {
        self.active = true;
        self.leaving = false;
    }

    /// `Deactivate` (`:64-67`): the leaving latch only.
    pub(crate) fn deactivate(&mut self) {
        self.leaving = true;
    }

    /// `Reset` (`:69-73`): opacity 0 + inactive; the leaving latch stays
    /// (verbatim — the C# does not clear it).
    pub(crate) fn reset(&mut self) {
        self.opacity = 0.0;
        self.active = false;
    }

    /// `IsActive` (`:75-78`) — the manager's removal test
    /// (`SkyManager.cs:39-42`) and the `ManageSpecialBiomeVisuals` edge
    /// comparison (`SceneState.cs:313`).
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    /// `Update` (`:24-47`), ONE manager iteration: `dt` is
    /// `gameTime.ElapsedGameTime.TotalSeconds`. The leaving arm fades out
    /// and deactivates below zero; the entering arm fades in capped at 1.
    /// The `FocusHelper.PauseSkies` head is the phase gate (module doc).
    pub(crate) fn update(&mut self, dt: f32) {
        if self.leaving {
            self.opacity -= dt;
            if self.opacity < 0.0 {
                self.active = false;
                self.opacity = 0.0;
            }
        } else {
            self.opacity += dt;
            if self.opacity > 1.0 {
                self.opacity = 1.0;
            }
        }
    }

    /// `Draw` (`:49-56`), one `DrawDepthRange(minDepth, maxDepth)` call:
    /// the band gate `minDepth < 1f || maxDepth == float.MaxValue`, then
    /// the full-screen tint quad. `severity` is the live
    /// `Sandstorm.Severity`; `sky_color` is `Main.ColorOfTheSkies`
    /// (the frame's rain-dimmed sky tint). `camera`/`zoom` map the C#'s
    /// screen-space `destinationRectangle` into the port's world-space
    /// back range (the `push_screen` convention — the fx range is world
    /// space, the sky pass draws before the tiles).
    pub(crate) fn draw(
        &self,
        r: &mut SpriteRenderer,
        camera: glam::Vec2,
        zoom: f32,
        screen: [f32; 2],
        sky_color: [f32; 3],
        severity: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        // `:51`: the quad only draws in a band that reaches below depth 1
        // (or the manager's whole-range `Draw`, maxDepth == float.MaxValue).
        if !(min_depth < 1.0 || max_depth == f32::MAX) {
            return;
        }
        // `:53`: `num = Math.Min(1f, Sandstorm.Severity * 1.5f)`.
        let num = (severity * 1.5).min(1.0);
        // `:54`: `(0.85, 0.66, 0.33, 1) * 0.8 * ColorOfTheSkies.ToVector4()`
        // then `Color * (_opacity * num)` — every channel (alpha included)
        // scales by the same product; the sky colour's alpha is 1.
        let a = self.opacity * num;
        let color = [
            0.85 * 0.8 * sky_color[0] * a,
            0.66 * 0.8 * sky_color[1] * a,
            0.33 * 0.8 * sky_color[2] * a,
            0.8 * a,
        ];
        if a <= 0.0 {
            return;
        }
        // `destinationRectangle = (0, 0, Main.screenWidth, Main.screenHeight)`
        // — the exact screen rect (no ±2 pad). The sky pass draws into the
        // world-space back range, so the screen rect maps through the
        // camera/zoom exactly like `push_screen` (`render/bg.rs:1064-1066`):
        // top-left = camera - screen/2/zoom, size = screen/zoom.
        let white = r.white_uv();
        let tl = camera - glam::Vec2::new(screen[0], screen[1]) * 0.5 / zoom;
        r.push_back_sprite(
            [tl.x, tl.y],
            [screen[0] / zoom, screen[1] / zoom],
            white,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Activate`/`Deactivate`/`Reset` (`:58-73`): the flag machine —
    /// activate keeps the opacity, deactivate only latches leaving, reset
    /// zeroes the opacity and the active flag but NOT the leaving latch.
    #[test]
    fn w4_c40_the_flag_machine_matches_the_csharp_verbatim() {
        let mut sky = SandstormSky::default();
        assert!(!sky.is_active());
        sky.activate();
        assert!(sky.is_active() && !sky.leaving);
        // Fade partway in, then deactivate + re-activate: the opacity stays.
        sky.update(0.5);
        sky.deactivate();
        assert!(sky.leaving && sky.is_active());
        sky.activate();
        assert!(!sky.leaving);
        assert!((sky.opacity - 0.5).abs() < 1e-6, "re-activate keeps the fade");
        // Reset clears opacity + active, keeps the leaving latch (C# :69-73).
        sky.deactivate();
        sky.reset();
        assert!(!sky.is_active() && sky.opacity == 0.0);
        assert!(sky.leaving, "Reset does not clear _isLeaving (verbatim)");
    }

    /// `Update` (`:30-46`): the leaving arm fades at dt/second and
    /// deactivates strictly below zero; the entering arm caps at 1.
    #[test]
    fn w4_c40_the_opacity_walk_fades_and_caps() {
        let mut sky = SandstormSky::default();
        sky.activate();
        // 60 ticks of 1/60 s reach exactly 1 and cap there.
        for _ in 0..120 {
            sky.update(1.0 / 60.0);
        }
        assert!((sky.opacity - 1.0).abs() < 1e-4);
        assert!(sky.is_active());
        // Leaving: 1 → 0 over a second, then the below-zero deactivate.
        sky.deactivate();
        for _ in 0..60 {
            sky.update(1.0 / 60.0);
        }
        assert!(sky.opacity.abs() < 1e-4);
        assert!(sky.is_active(), "opacity == 0 is still active (the < 0 test)");
        sky.update(1.0 / 60.0);
        assert!(!sky.is_active() && sky.opacity == 0.0);
    }

    /// `Draw` (`:51`): the band gate — a band wholly above depth 1 draws
    /// nothing; the whole-range `Draw` (maxDepth == float.MaxValue) and any
    /// band reaching below 1 draw. Pure gate pin (no GPU).
    #[test]
    fn w4_c40_the_depth_band_gate_is_the_csharp_disjunction() {
        let gate = |min: f32, max: f32| min < 1.0 || max == f32::MAX;
        assert!(gate(f32::MIN, f32::MAX), "the manager's whole-range Draw");
        assert!(gate(0.5, 1.0), "a band reaching below 1");
        assert!(gate(f32::NEG_INFINITY, 6.667), "the first DrawToDepth band");
        assert!(!gate(1.0, 6.667), "min == 1 is NOT < 1");
        assert!(!gate(5.0, 6.667), "a band wholly above 1 skips the quad");
    }

    /// The tint product (`:53-54`): `(0.85,0.66,0.33)*0.8*skyColor *
    /// opacity * min(1, Severity*1.5)` — the severity saturates at 2/3,
    /// the alpha channel rides the same 0.8 factor.
    #[test]
    fn w4_c40_the_tint_math_saturates_severity_at_two_thirds() {
        let sky_color = [0.5, 0.6, 0.7];
        let tint = |opacity: f32, severity: f32| -> [f32; 4] {
            let num = (severity * 1.5).min(1.0);
            let a = opacity * num;
            [
                0.85 * 0.8 * sky_color[0] * a,
                0.66 * 0.8 * sky_color[1] * a,
                0.33 * 0.8 * sky_color[2] * a,
                0.8 * a,
            ]
        };
        let half = tint(1.0, 1.0 / 3.0); // severity 1/3 → num 0.5
        assert!((half[0] - 0.85 * 0.8 * 0.5 * 0.5).abs() < 1e-6);
        assert!((half[3] - 0.8 * 0.5).abs() < 1e-6);
        // Severity 2/3 and 1 both saturate num at 1.
        assert_eq!(tint(1.0, 2.0 / 3.0), tint(1.0, 1.0));
        // Opacity 0 → every channel 0 (the quad is skipped).
        assert_eq!(tint(0.0, 1.0), [0.0; 4]);
    }
}
