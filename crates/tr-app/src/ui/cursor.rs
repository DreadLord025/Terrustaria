//! Vanilla mouse cursor (`Main.DrawCursor`, `Main.cs:47982`). In the non-gamepad
//! branch the cursor is `Images/UI/Cursor_0` (`Cursor_1` when smart-selecting),
//! drawn as a shadow plus the sprite itself in `Main.cursorColor`.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;

/// `Main.cursorAlpha` / `Main.cursorColorDirection` (`Main.cs:52479-52491`).
pub struct CursorState {
    /// `Main.mouseColor` (`Main.cs:67199`): the stored cursor colour, edited by
    /// the cursor-colour screen (menuMode 25, `Main.cs:50324`). `CursorColor`
    /// reads only R/G/B (`Main.cs:52490`), so the stored alpha byte is unused
    /// here.
    pub mouse_color: [u8; 4],
    /// Oscillates in `0.6..=1.0` (starts at `0.6`; `Main.cs:67201` sets the
    /// direction to `+1`).
    pub alpha: f32,
    /// `+1` while fading in, `-1` while fading out.
    pub direction: f32,
}

impl Default for CursorState {
    fn default() -> Self {
        // `Main.mouseColor`'s static default (`Main.cs:67199`) — the persisted
        // value arrives through [`CursorState::with_mouse_color`], because
        // `config.json` is read once at startup (`Settings::load` in `main`),
        // never per consumer.
        Self::with_mouse_color([255, 50, 95, 255])
    }
}

impl CursorState {
    /// The idle pulse seeded from a stored `MouseColor` ("MouseColor" config
    /// key, `Main.cs:4164-4176`; the value's default is `Main.cs:67199`).
    pub fn with_mouse_color(mouse_color: [u8; 4]) -> Self {
        Self {
            mouse_color,
            alpha: 0.6,
            direction: 1.0,
        }
    }

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
            self.mouse_color[0] as f32 / 255.0 * self.alpha,
            self.mouse_color[1] as f32 / 255.0 * self.alpha,
            self.mouse_color[2] as f32 / 255.0 * self.alpha,
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

/// `Main.DrawThickCursor`'s non-gamepad half (`Main.cs:48064-48134`): while
/// `Main.ThickMouse` (`MouseBorderColor != Color.Transparent`, `Main.cs:2599`)
/// four `Cursors[11]` ring sprites sit at the cursor plus the offsets
/// `(0,1), (1,0), (0,-1), (-1,0)` each `+ Vector2.One * 2` (`:48094-48107`),
/// origin `(2, 2)` (`:48108`), scale `cursorScale * 1.1` (`:48110`), tinted
/// `MouseBorderColor`. Returns the bonus `(2, 2)` the caller adds to the
/// cursor sprite (`:48131`); a transparent border colour returns zero and
/// draws nothing (`:48133`). The gamepad branches (`ShowGamepadCursor`,
/// `UILinkPointNavigator`) have no port counterpart — keyboard/mouse only.
pub fn draw_thick(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &CursorState,
    mouse: Vec2,
    border_color: [u8; 4],
) -> Vec2 {
    let bonus = thick_cursor_bonus(border_color);
    if bonus == Vec2::ZERO {
        return bonus;
    }
    let Some(rect) = lookups.cursor_thick else {
        // The ring sheet is missing but the bonus still applies — the C#
        // returns `(2, 2)` regardless of the asset's load state.
        return bonus;
    };
    let Some(uv) = lookups.hud_uv(Some(&rect)) else {
        return bonus;
    };
    let size = [rect.width as f32, rect.height as f32];
    let scale = state.scale() * 1.1;
    let tint = [
        border_color[0] as f32 / 255.0,
        border_color[1] as f32 / 255.0,
        border_color[2] as f32 / 255.0,
        border_color[3] as f32 / 255.0,
    ];
    // The origin `(2, 2)` shifts the top-left by `-2 * scale` per axis.
    let origin = [2.0 * scale, 2.0 * scale];
    for off in [[0.0, 1.0], [1.0, 0.0], [0.0, -1.0], [-1.0, 0.0]] {
        // `vector += Vector2.One * 2f` (`:48107`).
        let pos = [
            mouse.x + off[0] + 2.0 - origin[0],
            mouse.y + off[1] + 2.0 - origin[1],
        ];
        r.push_ui_sprite(pos, [size[0] * scale, size[1] * scale], uv, tint);
    }
    bonus
}

/// `Main.DrawThickCursor`'s return value (`Main.cs:48066,48131,48133`): the
/// `Main.ThickMouse` gate (`MouseBorderColor != Color.Transparent`,
/// `Main.cs:2599` — XNA `Color !=` compares the packed RGBA, so any non-zero
/// byte keeps it on) picks the `(2, 2)` bonus or zero.
pub fn thick_cursor_bonus(border_color: [u8; 4]) -> Vec2 {
    if border_color == [0, 0, 0, 0] {
        Vec2::ZERO
    } else {
        Vec2::new(2.0, 2.0)
    }
}

/// `Main.DrawCursor`'s non-gamepad branch with the thick-cursor bonus
/// (`Main.cs:48046-48048` + the `DrawThickCursor` call at `:56039`): the same
/// shadow + sprite as [`draw`], both shifted by the bonus the border pass
/// returned. The fullscreen map draws through this (`Main.cs:56037-56040`).
pub fn draw_with_bonus(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &CursorState,
    mouse: Vec2,
    bonus: Vec2,
) {
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
    // `Main.cs:48047`: `mouse + bonus + Vector2.One`.
    r.push_ui_sprite(
        [mouse.x + bonus.x + 1.0, mouse.y + bonus.y + 1.0],
        [size[0] * scale * 1.1, size[1] * scale * 1.1],
        uv,
        [
            color[0] * 0.2,
            color[1] * 0.2,
            color[2] * 0.2,
            color[3] * 0.5,
        ],
    );
    // `Main.cs:48048`: `mouse + bonus`.
    r.push_ui_sprite(
        [mouse.x + bonus.x, mouse.y + bonus.y],
        [size[0] * scale, size[1] * scale],
        uv,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values derived from `Main.cs:52477-52491` / `:67199` — never
    // from this code. The states are built explicitly so no test touches the
    // `Default` config.json read.

    #[test]
    fn the_stored_mouse_color_seeds_the_idle_pulse() {
        // `Main.cs:67199`: `Main.mouseColor = new Color(255, 50, 95)` and
        // `cursorColorDirection = 1` with `cursorAlpha` at its 0.6 floor — the
        // constructor seeds the tint, `Default` carries the vanilla default and
        // neither reads `config.json` (the app passes the startup-read value).
        let seeded = CursorState::with_mouse_color([10, 20, 30, 255]);
        assert_eq!(seeded.mouse_color, [10, 20, 30, 255]);
        assert_eq!(seeded.alpha, 0.6);
        assert_eq!(seeded.direction, 1.0);
        let default = CursorState::default();
        assert_eq!(default.mouse_color, [255, 50, 95, 255]);
        assert_eq!(default.alpha, 0.6);
        assert_eq!(default.direction, 1.0);
    }

    #[test]
    fn cursor_oscillates_between_0_6_and_1() {
        let mut c = CursorState {
            mouse_color: [255, 50, 95, 255],
            alpha: 0.6,
            direction: 1.0,
        };
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
            mouse_color: [255, 50, 95, 255],
            alpha: 1.0,
            direction: 1.0,
        };
        assert!((c.scale() - 1.1).abs() < 1e-6); // 1.0*0.3+0.7+0.1
    }

    #[test]
    fn cursor_color_uses_default_mouse_color_at_full_alpha() {
        // `Main.mouseColor = new Color(255, 50, 95)` (`Main.cs:67199`).
        let c = CursorState {
            mouse_color: [255, 50, 95, 255],
            alpha: 1.0,
            direction: -1.0,
        };
        assert_eq!(c.color(), [1.0, 50.0 / 255.0, 95.0 / 255.0, 1.0]);
    }

    #[test]
    fn cursor_color_tints_from_the_stored_mouse_color() {
        // `Main.cs:52490`: every channel is `(int)mouseColor.* * cursorAlpha`
        // (rendered as the unit-float ratio here); the stored alpha byte is
        // never read, and the pulse alpha drives the output alpha.
        let c = CursorState {
            mouse_color: [10, 20, 30, 255],
            alpha: 0.8,
            direction: 1.0,
        };
        let a = 0.8f32;
        assert_eq!(
            c.color(),
            [
                10.0 / 255.0 * a,
                20.0 / 255.0 * a,
                30.0 / 255.0 * a,
                a * 0.3 + 0.7
            ]
        );
    }

    /// W4-e18 residual census (written, not run - code-only): the thick
    /// cursor's gate and bonus (`Main.cs:2599`, `:48066-48133`). A fully
    /// transparent `MouseBorderColor` draws nothing and returns zero; any
    /// other colour returns the `(2, 2)` bonus — the C# returns it regardless
    /// of the ring asset's load state. The vanilla default border is
    /// `Color(64, 64, 64, 64)` (`Main.cs:66514`), so the border is ON.
    #[test]
    fn w4_e18_thick_cursor_gate_and_bonus() {
        // Transparent: no bonus, no sprites.
        assert_eq!(thick_cursor_bonus([0, 0, 0, 0]), Vec2::ZERO);
        // The vanilla default border (`Main.cs:66514`).
        assert_eq!(thick_cursor_bonus([64, 64, 64, 64]), Vec2::new(2.0, 2.0));
        // A single non-zero byte keeps ThickMouse true (XNA packed compare).
        assert_eq!(thick_cursor_bonus([0, 0, 0, 1]), Vec2::new(2.0, 2.0));
        assert_eq!(thick_cursor_bonus([1, 0, 0, 0]), Vec2::new(2.0, 2.0));
    }
}
