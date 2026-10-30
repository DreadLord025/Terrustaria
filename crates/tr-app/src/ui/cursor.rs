//! Vanilla mouse cursor (`Main.DrawCursor`, `Main.cs:47982`). In the non-gamepad
//! branch the cursor is `Images/UI/Cursor_0` (`Cursor_1` when smart-selecting),
//! drawn as a shadow plus the sprite itself in `Main.cursorColor`.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;

/// `Main.mouseColor` (`Main.cs:67199`): the default `(255, 50, 95)`.
const MOUSE_COLOR: [f32; 3] = [255.0, 50.0, 95.0];

/// `Main.cursorAlpha` / `Main.cursorColorDirection` (`Main.cs:52479-52491`).
pub struct CursorState {
    /// Oscillates in `0.6..=1.0` (starts at `0.6`; `Main.cs:67201` sets the
    /// direction to `+1`).
    pub alpha: f32,
    /// `+1` while fading in, `-1` while fading out.
    pub direction: f32,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            alpha: 0.6,
            direction: 1.0,
        }
    }
}

impl CursorState {
    /// `Main.CursorColor` (`Main.cs:52477`): advance the pulse by `0.015` per
    /// frame, clamping at `1.0` / `0.6` and flipping the direction there.
    pub fn update(&mut self) {
        self.alpha += self.direction * 0.015;
        if self.alpha >= 1.0 {
            self.alpha = 1.0;
            self.direction = -1.0;
        }
        if self.alpha <= 0.6 {
            self.alpha = 0.6;
            self.direction = 1.0;
        }
    }

    /// `Main.cursorScale` (`Main.cs:52491`): `cursorAlpha * 0.3 + 0.7 + 0.1`.
    pub fn scale(&self) -> f32 {
        self.alpha * 0.3 + 0.7 + 0.1
    }

    /// `Main.cursorColor` (`Main.cs:52490`), built from `mouseColor * alpha` with
    /// the pulse-written alpha `cursorAlpha * 0.3 + 0.7`.
    pub fn color(&self) -> [f32; 4] {
        [
            MOUSE_COLOR[0] / 255.0 * self.alpha,
            MOUSE_COLOR[1] / 255.0 * self.alpha,
            MOUSE_COLOR[2] / 255.0 * self.alpha,
            self.alpha * 0.3 + 0.7,
        ]
    }
}

/// `Main.DrawCursor` non-gamepad branch (`Main.cs:48046-48048`): a shadow at
/// `mouse + (1, 1)` scaled by `cursorScale * 1.1` with `rgb * 0.2` / `alpha * 0.5`,
/// then the cursor itself at `cursorScale` in `cursorColor`.
///
/// `lookups.cursor[0]` is `Cursor_0`; `[1]` (`Cursor_1`, smart select) is unused
/// this phase.
pub fn draw(r: &mut SpriteRenderer, lookups: &assets::Lookups, state: &CursorState, mouse: Vec2) {
    let rect = match lookups.cursor[0] {
        Some(rect) => rect,
        None => return,
    };
    let uv = match lookups.hud_uv(Some(&rect)) {
        Some(uv) => uv,
        None => return,
    };
    let size = [rect.width as f32, rect.height as f32];
    let scale = state.scale();
    let color = state.color();
    // `Main.cs:48047`
    r.push_ui_sprite(
        [mouse.x + 1.0, mouse.y + 1.0],
        [size[0] * scale * 1.1, size[1] * scale * 1.1],
        uv,
        [
            color[0] * 0.2,
            color[1] * 0.2,
            color[2] * 0.2,
            color[3] * 0.5,
        ],
    );
    // `Main.cs:48048`
    r.push_ui_sprite(mouse.into(), [size[0] * scale, size[1] * scale], uv, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_oscillates_between_0_6_and_1() {
        let mut c = CursorState::default();
        // 108 = 2 full periods: 27 frames up (0.6 -> 1.0) + 27 down (1.0 -> 0.6)
        // = 54, so 108 lands exactly on the 0.6 extreme.
        for _ in 0..108 {
            c.update();
            assert!((0.6..=1.0).contains(&c.alpha));
        }
        assert!((c.alpha - 1.0).abs() < 1e-3 || (c.alpha - 0.6).abs() < 1e-3);
    }

    #[test]
    fn cursor_scale_matches_cursor_alpha_formula() {
        let c = CursorState {
            alpha: 1.0,
            direction: 1.0,
        };
        assert!((c.scale() - 1.1).abs() < 1e-6); // 1.0*0.3+0.7+0.1
    }

    #[test]
    fn cursor_color_uses_default_mouse_color_at_full_alpha() {
        let c = CursorState {
            alpha: 1.0,
            direction: -1.0,
        };
        assert_eq!(c.color(), [1.0, 50.0 / 255.0, 95.0 / 255.0, 1.0]);
    }
}
