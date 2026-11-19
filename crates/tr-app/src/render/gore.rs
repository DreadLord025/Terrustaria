//! The gore draw pass — `Main.DrawGore` (`Main.cs:21873-21907`) over the
//! S1-E-4 pool (`tr_core::gore::GorePool`), with `Gore.GetAlpha`
//! (`Gore.cs:1464-1505`) and `SpriteFrame.GetSourceRectangle`
//! (`SpriteFrame.cs:72-77`).
//!
//! Draw order: the C# runs `DrawItems` (`Main.cs:62278`) → `DrawRain`
//! (`:62279`) → `DrawGore` (`:62282`) → `DrawDust` (`:62293`), so the seam
//! sits between the rain and the sand/wheel dust draws in the frame pass,
//! in the same world-effect (fx) range the dust and leaf gores use.
//!
//! Documented drops:
//! - `DrawGoreBehind` (`Main.cs:21842-21871`, called at `:56937`/`:63424`
//!   behind the tiles, gated on `Main.drawBackGore`) is not ported — the
//!   drip rows outside frames 7..=9 are skipped here exactly like
//!   `DrawGore`'s own gate (`:21883-21887`), but nothing draws them behind
//!   the tiles yet (the back pass needs the tile-batch seam).
//! - `LoadGore` (`Main.cs:59100-59106`) is a lazy texture request; the port
//!   packs every `Gore_<id>` sheet at startup instead
//!   (`AssetInitializer.cs:135-137` names them `Images/Gore_<i>`), so an
//!   absent file simply leaves that type undrawn.
//!
//! The pool/update behaviour (S1-E-4) is untouched — this module only reads
//! the slots; `DrawGore` has no kill-at-draw arm (unlike `DrawDust`).

use glam::Vec2;
use tr_core::gore::{GORE_SLOTS, Gore, GorePool, SpriteFrame};
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::particles::Color;
use tr_render::sprite::SpriteRenderer;

use crate::assets::{self, Lookups};

/// `SpriteFrame.GetSourceRectangle` (`SpriteFrame.cs:72-77`): the cell is
/// `texture / count` (integer division), the cursor picks the cell, and the
/// 2 px padding (`SpriteFrame.cs:7-12`) is subtracted only on an axis with
/// more than one cell. Returns `(x, y, w, h)` in sheet pixels. The `max(1)`
/// guards only fire on a zero-count frame the pool never produces
/// (`Gore.Frame` starts at `new SpriteFrame(1, 1)`, `Gore.cs:42`).
pub(crate) fn source_rect(frame: &SpriteFrame, tex_w: u32, tex_h: u32) -> [i32; 4] {
    let num = (tex_w / frame.column_count.max(1) as u32) as i32;
    let num2 = (tex_h / frame.row_count.max(1) as u32) as i32;
    [
        frame.current_column as i32 * num,
        frame.current_row as i32 * num2,
        num - if frame.column_count != 1 { 2 } else { 0 },
        num2 - if frame.row_count != 1 { 2 } else { 0 },
    ]
}

/// `(byte)` of an XNA `new Color(int, …)` argument: truncate, wrap past 255
/// (the `dust_table.rs` convention).
const fn byte(v: i32) -> u8 {
    v as u8
}

/// `Gore.GetAlpha(Color newColor)` (`Gore.cs:1464-1505`): the fade tint over
/// the light colour sampled at the gore's centre. Every arm is the C#'s
/// literal; the default path's channel products truncate through the XNA
/// byte ctor, and the alpha byte clamps into `[0, 255]` first (`:1491-1499`).
pub(crate) fn get_alpha(gore_type: i32, alpha: i32, new_color: Color) -> Color {
    // `:1466`.
    let num = (255 - alpha) as f32 / 255.0;
    if gore_type == 16 || gore_type == 17 {
        // `:1467-1470`.
        return Color::new(255, 255, 255, 0);
    }
    if gore_type == 716 || gore_type == 1383 {
        // `:1471-1474`.
        return Color::new(255, 255, 255, 200);
    }
    if (570..=572).contains(&gore_type) {
        // `:1475-1479`: `b / 2` is the byte's integer division.
        let b = byte(255 - alpha);
        return Color::new(b, b, b, b / 2);
    }
    if gore_type == 331 {
        // `:1480-1483`.
        return Color::new(255, 255, 255, 50);
    }
    if gore_type == 1225 {
        // `:1484-1487`: `new Color(float, …)` scales by 255 then byte-casts.
        let c = byte((num * 255.0) as i32);
        return Color::new(c, c, c, c);
    }
    // `:1488-1490`: `(int)((float)newColor.R * num)`, then the byte ctor.
    let r = byte((new_color.r as f32 * num) as i32);
    let g = byte((new_color.g as f32 * num) as i32);
    let b = byte((new_color.b as f32 * num) as i32);
    // `:1491-1499`.
    let num2 = (new_color.a as i32 - alpha).clamp(0, 255);
    if (1202..=1204).contains(&gore_type) {
        // `:1500-1503`.
        return Color::new(r, g, b, if num2 < 20 { num2 as u8 } else { 20 });
    }
    // `:1504`.
    Color::new(r, g, b, num2 as u8)
}

/// `Lighting.GetColor(int x, int y)` (`Lighting.cs:216-242`): the light
/// sample as XNA bytes — `(int)(c * GlobalBrightness * 255)` clamped at 255,
/// alpha 255 (`:240`'s `-16777216`). The style's `shade` folds the mode's
/// transforms and `GlobalBrightness` and clamps to `[0, 1]` first (the
/// wires-pass convention, `render/wires.rs`), so the truncation lands on
/// the same byte. The `Main.gameMenu` white branch (`:218-221`) never runs
/// here — the pass draws in-world only.
fn light_color(style: &LightStyle, light: &LightMap, x: i32, y: i32, time: f32) -> Color {
    let c = style.shade(light.get(x, y), x, y, time);
    Color::new(
        byte(((c[0] * 255.0) as i32).min(255)),
        byte(((c[1] * 255.0) as i32).min(255)),
        byte(((c[2] * 255.0) as i32).min(255)),
        255,
    )
}

/// The light-sample tile of one gore draw (`Main.cs:21897`/`:21902`), in
/// C# `double` precision. The two casts differ in order and both are kept:
/// X is `(int)(pos.X + w*0.5) / 16` — the cast precedes the INTEGER division
/// — while Y is `(int)((pos.Y + h*0.5) / 16.0)` — the division precedes the
/// cast. (For a positive divisor the two truncations agree, but the port
/// keeps the source's shape.)
pub(crate) fn light_tile(pos: Vec2, w: i32, h: i32) -> (i32, i32) {
    let x = (pos.x as f64 + w as f64 * 0.5) as i32 / 16;
    let y = ((pos.y as f64 + h as f64 * 0.5) / 16.0) as i32;
    (x, y)
}

/// `Main.cs:21879-21887`: whether a slot draws in `DrawGore` (the front
/// pass). Inactive/type-0 slots never draw; the drip rows outside the fall
/// frames 7..=9 belong to `DrawGoreBehind` (`GoreID.Sets.IsDrip`,
/// `GoreID.cs:5-25`) and would latch `Main.drawBackGore` (`:21885`).
pub(crate) fn draws_in_front(g: &Gore) -> bool {
    // `:21879-21882`.
    if !g.active || g.gore_type <= 0 {
        return false;
    }
    // `:21883-21887`.
    !((g.gore_type as usize) < tr_core::gore_id::GORE_COUNT
        && tr_core::gore_id::is_drip(g.gore_type as usize)
        && (g.frame.current_row < 7 || g.frame.current_row > 9))
}

/// One planned gore quad: the rotated top-left in screen px, the size, the
/// atlas UV, the byte tint normalised to `[0, 1]` and the rotation. Split
/// out of [`draw`] so the maths tests run without a GPU.
pub(crate) struct GoreQuad {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv: [f32; 4],
    pub color: [f32; 4],
    pub rot: f32,
}

/// The per-slot draw maths of `Main.DrawGore`'s body (`Main.cs:21889-21904`)
/// for a gore whose sheet is packed. `sheet` is the whole `Gore_<type>`
/// texture rect on the atlas.
pub(crate) fn plan_quad(
    g: &Gore,
    sheet: &tr_render::SheetRect,
    atlas_size: (u32, u32),
    tint: Color,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> GoreQuad {
    // `:21889-21891` vs `:21900-21903`: the framed cell or the whole sheet.
    let framed = g.frame.column_count > 1 || g.frame.row_count > 1;
    let src = if framed {
        source_rect(&g.frame, sheet.width, sheet.height)
    } else {
        [0, 0, sheet.width as i32, sheet.height as i32]
    };
    let (w, h) = (src[2], src[3]);
    // `:21892-21896`: the type-1217 nudge (framed path only).
    let nudge_y = if framed && g.gore_type == 1217 {
        4.0
    } else {
        0.0
    };
    // `:21898`/`:21903`: the draw position — `position - screenPosition +
    // (w/2, h/2)` with the framed path's extra `- 2f`; the halves are the
    // C#'s INTEGER divisions (`sourceRectangle.Width / 2`). The view
    // transform is the rain/sand `to_screen` (zoom about the screen centre).
    let half = [(w / 2) as f32, (h / 2) as f32];
    let lift = if framed { 2.0 } else { 0.0 };
    let anchor = [
        (g.position.x - camera.x + half[0]) * zoom + screen.x * 0.5,
        (g.position.y - camera.y + half[1] - lift + nudge_y) * zoom + screen.y * 0.5,
    ];
    // XNA `SpriteBatch.Draw` rotates about `position` with `origin` scaled
    // by `scale` (and the view zoom): the rotated top-left is
    // `anchor − R(rot)·(origin·scale·zoom)` (the `push_sprite_origin_rot`
    // emulation, `sprite.rs:780-801`, in the fx range). `origin` is the same
    // integer half (`:21898`/`:21903`).
    let origin = [half[0] * g.scale * zoom, half[1] * g.scale * zoom];
    let (s, c) = g.rotation.sin_cos();
    let pos = [
        anchor[0] - (origin[0] * c - origin[1] * s),
        anchor[1] - (origin[0] * s + origin[1] * c),
    ];
    GoreQuad {
        pos,
        size: [w as f32 * g.scale * zoom, h as f32 * g.scale * zoom],
        uv: assets::uv_rect(sheet, atlas_size, src[0], src[1], w as u32, h as u32),
        color: [
            tint.r as f32 / 255.0,
            tint.g as f32 / 255.0,
            tint.b as f32 / 255.0,
            tint.a as f32 / 255.0,
        ],
        rot: g.rotation,
    }
}

/// `Main.DrawGore` (`Main.cs:21873-21907`): the 600-slot walk, the drip
/// gate, the light sample at the source-rect centre and the tinted,
/// rotated, scaled quad. Read-only over the pool.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    pool: &GorePool,
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    for i in 0..GORE_SLOTS {
        let g = pool.slot(i);
        // `:21879-21887`.
        if !draws_in_front(g) {
            continue;
        }
        // `:21888` `LoadGore` — the sheet is packed at startup; an absent
        // file leaves the type undrawn (module docs).
        let Some(sheet) = lookups.gore_sheets.get(&(g.gore_type as u16)) else {
            continue;
        };
        // The source rect the light sample reads (`:21891`/`:21902`).
        let framed = g.frame.column_count > 1 || g.frame.row_count > 1;
        let (w, h) = if framed {
            let src = source_rect(&g.frame, sheet.width, sheet.height);
            (src[2], src[3])
        } else {
            (sheet.width as i32, sheet.height as i32)
        };
        let (tx, ty) = light_tile(g.position, w, h);
        let tint = get_alpha(
            g.gore_type,
            g.alpha,
            light_color(style, light, tx, ty, anim_time),
        );
        let q = plan_quad(g, sheet, lookups.atlas_size, tint, camera, zoom, screen);
        r.push_fx_sprite_rot(q.pos, q.size, q.uv, q.color, q.rot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SpriteFrame.cs:72-77`: the cell grid, the cursor and the 2 px
    /// padding on multi-cell axes only.
    #[test]
    fn s1_e_4b_the_source_rect_matches_sprite_frame() {
        // A 15-row drip sheet, 20x330: cell 20x22, padding on the row axis.
        let f = SpriteFrame {
            current_column: 0,
            current_row: 8,
            column_count: 1,
            row_count: 15,
        };
        assert_eq!(source_rect(&f, 20, 330), [0, 176, 20, 20]);
        // Single cell: no padding, the whole texture.
        let f = SpriteFrame::new(1, 1);
        assert_eq!(source_rect(&f, 26, 26), [0, 0, 26, 26]);
        // The 32x8 leaf strip: padding on BOTH axes. Sheet 320x80: cell
        // 10x10 → 8x8 after the padding.
        let f = SpriteFrame {
            current_column: 3,
            current_row: 5,
            column_count: 32,
            row_count: 8,
        };
        assert_eq!(source_rect(&f, 320, 80), [30, 50, 8, 8]);
        // Integer division floors the cell (`texture.Width / ColumnCount`).
        let f = SpriteFrame::new(3, 1);
        assert_eq!(source_rect(&f, 25, 20), [0, 0, 6, 20]);
    }

    /// `Gore.cs:1467-1487`: the constant arms.
    #[test]
    fn s1_e_4b_get_alpha_constant_arms() {
        let white = Color::WHITE;
        assert_eq!(get_alpha(16, 100, white), Color::new(255, 255, 255, 0));
        assert_eq!(get_alpha(17, 0, white), Color::new(255, 255, 255, 0));
        assert_eq!(get_alpha(716, 100, white), Color::new(255, 255, 255, 200));
        assert_eq!(get_alpha(1383, 0, white), Color::new(255, 255, 255, 200));
        assert_eq!(get_alpha(331, 250, white), Color::new(255, 255, 255, 50));
        // `:1475-1479`: b = 255 - alpha, alpha byte b/2 (integer).
        assert_eq!(get_alpha(570, 100, white), Color::new(155, 155, 155, 77));
        assert_eq!(get_alpha(572, 254, white), Color::new(1, 1, 1, 0));
        // `:1484-1487`: the float ctor — (255-alpha)/255 * 255 truncated.
        assert_eq!(get_alpha(1225, 100, white), Color::new(155, 155, 155, 155));
        assert_eq!(get_alpha(1225, 0, white), Color::new(255, 255, 255, 255));
    }

    /// `Gore.cs:1488-1504`: the default byte multiplies, the alpha clamp
    /// and the 1202-1204 cap.
    #[test]
    fn s1_e_4b_get_alpha_default_byte_maths() {
        // alpha 0 over white: the light passes through untouched.
        assert_eq!(get_alpha(1, 0, Color::WHITE), Color::WHITE);
        // alpha 128: num = 127/255; 255 * num = 127.0 → 127 per channel,
        // alpha 255 - 128 = 127.
        assert_eq!(
            get_alpha(1, 128, Color::WHITE),
            Color::new(127, 127, 127, 127)
        );
        // A coloured light truncates per channel: 200 * (155/255) =
        // 121.568… → 121; 100 * 0.60784… = 60.784… → 60; 50 → 30.392… → 30.
        let c = Color::new(200, 100, 50, 255);
        assert_eq!(get_alpha(1, 100, c), Color::new(121, 60, 30, 155));
        // The alpha clamp (`:1491-1499`): a light A below the fade floors
        // at 0, never wraps.
        let dark = Color::new(10, 10, 10, 40);
        assert_eq!(get_alpha(1, 100, dark).a, 0);
        // `:1500-1503`: the fog-cloud rows cap the alpha at 20.
        assert_eq!(get_alpha(1202, 0, Color::WHITE).a, 20);
        assert_eq!(get_alpha(1204, 250, Color::WHITE).a, 5);
        assert_eq!(
            get_alpha(1203, 100, Color::WHITE),
            Color::new(155, 155, 155, 20)
        );
    }

    /// `Main.cs:21897`: the light-sample tile — X casts before the integer
    /// division, Y divides before the cast (both kept verbatim).
    #[test]
    fn s1_e_4b_the_light_sample_tile() {
        // (int)(100 + 10) = 110 → 110/16 = 6; 110/16.0 = 6.875 → 6.
        assert_eq!(light_tile(Vec2::new(100.0, 100.0), 20, 20), (6, 6));
        // Negative coords truncate toward zero on both paths:
        // (int)(-25) = -25 → -25/16 = -1; -25/16.0 = -1.5625 → -1.
        assert_eq!(light_tile(Vec2::new(-30.0, -30.0), 10, 10), (-1, -1));
        // The half-pixel sum crosses a tile boundary.
        assert_eq!(light_tile(Vec2::new(15.9, 15.9), 0, 0), (0, 0));
        assert_eq!(light_tile(Vec2::new(16.1, 16.1), 0, 0), (1, 1));
        // Fractional sums: X truncates the sum first ((int)15.9 = 15 → 0).
        assert_eq!(light_tile(Vec2::new(15.4, 15.4), 1, 1), (0, 0));
    }

    /// `Main.cs:21898`/`:21903`: the quad geometry — the integer halves,
    /// the framed path's −2 lift, the 1217 nudge and the scaled origin.
    #[test]
    fn s1_e_4b_the_quad_geometry_matches_the_draw_call() {
        let sheet = tr_render::SheetRect {
            x: 0,
            y: 0,
            width: 26,
            height: 26,
            plain_frame: (0, 0),
        };
        let mut g = Gore {
            active: true,
            gore_type: 1,
            position: Vec2::new(100.0, 200.0),
            scale: 1.0,
            rotation: 0.0,
            frame: SpriteFrame::new(1, 1),
            ..Gore::default()
        };
        let cam = Vec2::ZERO;
        let screen = Vec2::new(800.0, 600.0);
        // Unframed, rot 0: top-left = position + screen/2 (the half
        // cancels the origin), full 26x26.
        let q = plan_quad(&g, &sheet, (8192, 8192), Color::WHITE, cam, 1.0, screen);
        assert_eq!(q.pos, [500.0, 500.0]);
        assert_eq!(q.size, [26.0, 26.0]);
        assert_eq!(q.rot, 0.0);
        // Framed 1x4 sheet (26x104): cell 26x26 − 2 padding = 26x24, the
        // −2 lift moves the anchor up 2 px.
        let sheet4 = tr_render::SheetRect {
            height: 104,
            ..sheet
        };
        g.frame = SpriteFrame::new(1, 4);
        let q = plan_quad(&g, &sheet4, (8192, 8192), Color::WHITE, cam, 1.0, screen);
        assert_eq!(q.size, [26.0, 24.0]);
        // anchor.y = 200 + 12 − 2 + 300 = 510; rot 0 → tl = anchor − origin.
        assert_eq!(q.pos, [500.0, 498.0]);
        // The 1217 nudge adds 4 to the framed anchor (`:21893-21896`).
        g.gore_type = 1217;
        let q = plan_quad(&g, &sheet4, (8192, 8192), Color::WHITE, cam, 1.0, screen);
        assert_eq!(q.pos, [500.0, 502.0]);
        // Scale 0.5, rot π/2: origin (13, 12)·0.5 = (6.5, 6);
        // R(π/2)·origin = (−6, 6.5); tl = anchor − that.
        g.gore_type = 1;
        g.scale = 0.5;
        g.rotation = std::f32::consts::FRAC_PI_2;
        let q = plan_quad(&g, &sheet4, (8192, 8192), Color::WHITE, cam, 1.0, screen);
        assert_eq!(q.size, [13.0, 12.0]);
        let ax = 500.0 + 13.0;
        let ay = 500.0 + 12.0 - 2.0;
        assert!((q.pos[0] - (ax + 6.0)).abs() < 1e-3);
        assert!((q.pos[1] - (ay - 6.5)).abs() < 1e-3);
        // The tint normalises the XNA bytes.
        let q = plan_quad(
            &g,
            &sheet4,
            (8192, 8192),
            Color::new(127, 60, 30, 155),
            cam,
            1.0,
            screen,
        );
        assert_eq!(
            q.color,
            [127.0 / 255.0, 60.0 / 255.0, 30.0 / 255.0, 155.0 / 255.0]
        );
    }

    /// `Main.cs:21879-21887`: the walk's gates — inactive/type-0 slots and
    /// the drip rows outside frames 7..=9 draw nothing in the front pass,
    /// while a fall-frame drip does draw.
    #[test]
    fn s1_e_4b_the_drip_gate_matches_draw_gore() {
        let mut g = Gore {
            active: true,
            gore_type: 706,
            frame: SpriteFrame::new(1, 15),
            ..Gore::default()
        };
        g.frame.current_row = 3;
        assert!(
            !draws_in_front(&g),
            "a hanging drip belongs to DrawGoreBehind"
        );
        g.frame.current_row = 7;
        assert!(draws_in_front(&g), "the fall frames draw in DrawGore");
        g.frame.current_row = 9;
        assert!(draws_in_front(&g));
        g.frame.current_row = 10;
        assert!(!draws_in_front(&g), "the rest frames draw behind the tiles");
        g.gore_type = 1;
        g.frame.current_row = 0;
        assert!(draws_in_front(&g), "non-drip types always draw");
        g.active = false;
        assert!(!draws_in_front(&g));
        g.active = true;
        g.gore_type = 0;
        assert!(!draws_in_front(&g));
    }
}
