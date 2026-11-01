//! The boss health bar: the `BigProgressBarHelper.DrawFancyBar` port — the six
//! 516x58 rows of the `UI_BossBar` sheet around a 456x22 bar at the bottom
//! centre of the screen, the boss head icon and the `current / max` text in
//! `Fonts/Item_Stack`.
//!
//! King Slime reaches only the 4-arg overload, through
//! `CommonBossBigProgressBar` (`CommonBossBigProgressBar.cs:32-36`), which feeds
//! `BigProgressBarCache.LifeCurrent/LifeMax` (`BigProgressBarCache.cs:13`) and
//! the whole `NPC_Head_Boss_7` sheet as `barIconFrame` (`value.Frame()` =
//! the full sheet, `Utils.cs:1344-1349`; King Slime is head index 7,
//! `NPCID.cs:10479`). The shielded 7-arg overload (`:72-126`) and
//! `DrawBareBonesBar` (`:11-23`) have no reachable caller in this port and are
//! not ported.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::color;
use super::text::{draw_border_string, hud_item_text_width, BorderFont};
use crate::assets;

/// The four `Utils.DrawBorderStringFourWay` border-pass offsets, in order
/// (`Utils.cs:2472-2492`): left, right, up, down — each ±2 px around the draw
/// point. The centre colour pass is the shared `text::draw_border_string`'s
/// own (`Utils.cs:2490-2492`).
const FOUR_WAY_RING: [[f32; 2]; 4] = [
    [-2.0, 0.0],
    [2.0, 0.0],
    [0.0, -2.0],
    [0.0, 2.0],
];

/// `Color.White` — the centre text pass (`Utils.cs:2490-2492`) and the white
/// sprite tints (`BigProgressBarHelper.cs:49-52`).
const TEXT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// `BigProgressBarHelper.DrawFancyBar` (`BigProgressBarHelper.cs:25-57`) with
/// the `CommonBossBigProgressBar` inputs (`CommonBossBigProgressBar.cs:32-36`):
/// `current`/`max` are `BigProgressBarCache.LifeCurrent/LifeMax`
/// (`BigProgressBarCache.cs:13`), the icon is `Lookups::ui_boss_head`
/// (`NPC_Head_Boss_7` — King Slime, `NPCID.cs:10479`) drawn with
/// `value.Frame()` = the whole sheet (`Utils.cs:1344-1349`). Draws nothing when
/// `ui_boss_bar` is missing (`Lookups::default()`).
pub fn draw_fancy_bar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    current: i32,
    max: i32,
) {
    // Missing `UI_BossBar` (PNG-only run, `Lookups::default()`) draws nothing.
    if lookups.ui_boss_bar.is_none() {
        return;
    }
    // The one UV path: the `bar_srcs()` rects fed through
    // `Lookups::boss_bar_uv` with `row = 0` and `sy` = the rect's absolute
    // sheet `y` (the helper adds `row * 58`).
    let uv = |src: [f32; 4]| {
        lookups.boss_bar_uv(
            0,
            src[0] as u32,
            src[1] as u32,
            src[2] as u32,
            src[3] as u32,
        )
    };
    let srcs = bar_srcs();
    let bar = bar_rect(screen);
    let vector = frame_origin(screen); // `rectangle.TopLeft() - p2.ToVector2()` (`:47`)

    // 1. backdrop (`:48`): `Color.White * 0.2f` (`:32`) dims every channel —
    //    XNA byte truncation gives (51, 51, 51, 51), not white at alpha 0.2.
    if let Some(backdrop_uv) = uv(srcs[0]) {
        let dim = color::tint(color::mul([255, 255, 255, 255], 0.2));
        r.push_ui_sprite(vector, [srcs[0][2], srcs[0][3]], backdrop_uv, dim);
    }
    // 2. body (`:49`): the 2x22 fill strip stretched to `num2` px wide (the
    //    original scales the source by `num2 / value3.Width = num2 / 2`; the
    //    port stretches the destination instead).
    let num2 = fill_width(current as f32, max as f32);
    if let Some(body_uv) = uv(srcs[1]) {
        r.push_ui_sprite(
            [bar[0], bar[1]],
            [num2 as f32, srcs[1][3]],
            body_uv,
            TEXT_COLOR,
        );
    }
    // 3. cap (`:50`): 2 px wide, at `rectangle.TopLeft() + (num2 - 2, 0)`.
    if let Some(cap_uv) = uv(srcs[2]) {
        r.push_ui_sprite(
            [bar[0] + num2 as f32 - 2.0, bar[1]],
            [srcs[2][2], srcs[2][3]],
            cap_uv,
            TEXT_COLOR,
        );
    }
    // 4. frame (`:51`): `value.Frame(1, verticalFrames)` = the row-0 whole.
    if let Some(frame_uv) = uv(srcs[3]) {
        r.push_ui_sprite(vector, [srcs[3][2], srcs[3][3]], frame_uv, TEXT_COLOR);
    }
    // 5. icon (`:52`): the whole `NPC_Head_Boss_7` sheet centred on
    //    `vector + (4, 20) + (26, 28) / 2` — the C# literal, NOT
    //    `barIconFrame.Size()/2` (plan Uncertainty 9) — with the runtime
    //    `Size()` (`Utils.cs:1364-1367`) as the draw origin.
    if let Some(head) = lookups.ui_boss_head {
        let size = [head.width as f32, head.height as f32];
        let center = icon_center(screen);
        let top_left = [center[0] - size[0] / 2.0, center[1] - size[1] / 2.0];
        if let Some(icon_uv) = lookups.hud_uv(Some(&head)) {
            r.push_ui_sprite(top_left, size, icon_uv, TEXT_COLOR);
        }
    }
    // 6. `BigProgressBarSystem.ShowText` is `true` (`BigProgressBarSystem.cs:259`)
    //    and the port has no toggle, so the health text always draws (`:53-55`).
    let slash = "/";
    let current_text = format!("{current}"); // `((int)current).ToString()` (`:66`)
    let max_text = format!("{max}"); // `((int)max).ToString()` (`:68`)
    let line = item_line_height(lookups); // `MeasureString(...).Y`
    let (slash_at, current_at, max_at) = health_text_positions(
        // `area.Center + (0, 1)` (`:62-63`).
        [bar[0] + bar[2] / 2.0, bar[1] + bar[3] / 2.0 + 1.0],
        [hud_item_text_width(r, lookups, slash, 1.0), line],
        [hud_item_text_width(r, lookups, &current_text, 1.0), line],
        [hud_item_text_width(r, lookups, &max_text, 1.0), line],
    );
    draw_border_string_four_way(r, lookups, slash_at, slash);
    draw_border_string_four_way(r, lookups, current_at, &current_text);
    draw_border_string_four_way(r, lookups, max_at, &max_text);
}

/// `BigProgressBarHelper.cs:46` + `Utils.cs:947-950`:
/// `CenteredRectangle(screen * (0.5, 1) + (0, -50), (456, 22))` — all four
/// components `(int)`-truncated. == (572, 839, 456, 22) at 1600x900.
pub fn bar_rect(screen: Vec2) -> [f32; 4] {
    // `Main.ScreenSize.ToVector2() * new Vector2(0.5f, 1f) + new Vector2(0f, -50f)`
    // (`:46`); `p = (456, 22)` (`:28`).
    let center = [screen.x * 0.5, screen.y - 50.0];
    // `(int)(center.X - size.X / 2f)`, `(int)(center.Y - size.Y / 2f)`,
    // `(int)size.X`, `(int)size.Y` (`Utils.cs:947-950`).
    [
        (center[0] - 456.0 / 2.0) as i32 as f32,
        (center[1] - 22.0 / 2.0) as i32 as f32,
        456.0,
        22.0,
    ]
}

/// `BigProgressBarHelper.cs:33-35`: `num2 = (int)(456f * current/max)`,
/// `num2 -= num2 % 2` — an even 0..=456 for `0 <= current <= max`.
pub fn fill_width(current: f32, max: f32) -> i32 {
    let num = current / max; // `float num = lifeAmount / lifeMax;` (`:33`)
    let mut num2 = (456.0f32 * num) as i32; // `(int)((float)p.X * num)` (`:34`)
    num2 -= num2 % 2; // (`:35`)
    num2
}

/// The four source rects on the 516x348 `UI_BossBar` (six 516x58 rows,
/// `Frame(1, 6, 0, row)` = `Utils.cs:1344-1349`), indexed
/// (backdrop, body, cap, frame): (0,174,516,58) `:31` | (32,140,2,22) `:36-40` |
/// (32,82,2,22) `:41-45` | (0,0,516,58) `:51`.
pub fn bar_srcs() -> [[f32; 4]; 4] {
    // `p2 = (32, 24)` (`:29`): the body/cap offsets are `X += p2.X, Y += p2.Y`,
    // `Width = 2`, `Height = p.Y` (`:37-40`, `:42-45`).
    [
        [0.0, 174.0, 516.0, 58.0], // backdrop: `value.Frame(1, 6, 0, 3)` (`:31`)
        [32.0, 140.0, 2.0, 22.0],  // body: `Frame(1, 6, 0, 2)` + `p2` (`:36-40`)
        [32.0, 82.0, 2.0, 22.0],   // cap: `Frame(1, 6, 0, 1)` + `p2` (`:41-45`)
        [0.0, 0.0, 516.0, 58.0],   // frame: `value.Frame(1, verticalFrames)` (`:51`)
    ]
}

/// `BigProgressBarHelper.cs:47` + `:52`: the frame origin
/// `bar_rect.TopLeft() - (32, 24)` and the icon centre
/// `vector + (4, 20) + (26, 28) / 2` — `(26,28)` is the C# literal, NOT
/// `barIconFrame.Size()` (plan Uncertainty 9). At 1600x900: (540, 815) and
/// (557, 849).
pub fn frame_origin(screen: Vec2) -> [f32; 2] {
    let bar = bar_rect(screen);
    [bar[0] - 32.0, bar[1] - 24.0] // `rectangle.TopLeft() - p2.ToVector2()` (`:47`)
}

pub fn icon_center(screen: Vec2) -> [f32; 2] {
    let vector = frame_origin(screen);
    // `vector + (new Vector2(4f, 20f) + new Vector2(26f, 28f) / 2f)` (`:52`).
    [
        vector[0] + 4.0 + 26.0 / 2.0,
        vector[1] + 20.0 + 28.0 / 2.0,
    ]
}

/// `BigProgressBarHelper.DrawHealthText` (`:59-70`): the three draw
/// top-lefts `(slash, current, max)`. `anchor` is
/// `area.Center + (0, 1)` (`:62-63`); each pass draws at
/// `point - origin` (`Utils.cs:2494`), with origins `(ws/2, hs/2)`,
/// `(wc, hc/2)` at `anchor.x - 5`, `(0, hm/2)` at `anchor.x + 5`
/// (`:64-69`). `hs/hc/hm` are `MeasureString(...).Y` — the port passes the
/// item font's `line_spacing` for all three (the established convention,
/// `ui/tooltip.rs:151-156`).
pub fn health_text_positions(
    anchor: [f32; 2],
    slash_wh: [f32; 2],
    cur_wh: [f32; 2],
    max_wh: [f32; 2],
) -> ([f32; 2], [f32; 2], [f32; 2]) {
    // `origin: value.MeasureString(text) * 0.5f` (`:65`).
    let slash = [
        anchor[0] - slash_wh[0] / 2.0,
        anchor[1] - slash_wh[1] / 2.0,
    ];
    // `x: vector.X - 5f`, `origin: MeasureString(text) * new Vector2(1f, 0.5f)` (`:67`).
    let current = [anchor[0] - 5.0 - cur_wh[0], anchor[1] - cur_wh[1] / 2.0];
    // `x: vector.X + 5f`, `origin: MeasureString(text) * new Vector2(0f, 0.5f)`
    // (`:69`) — `origin.x == 0`, so the draw left is `anchor.x + 5`.
    let max = [anchor[0] + 5.0, anchor[1] - max_wh[1] / 2.0];
    (slash, current, max)
}

/// `MeasureString(...).Y` for `Fonts/Item_Stack` (`:61`): the port passes the
/// item font's `line_spacing` (the established convention,
/// `ui/tooltip.rs:151-156`; the built-in bitmap fallback draws 8 px glyphs).
fn item_line_height(lookups: &assets::Lookups) -> f32 {
    match &lookups.item_font {
        Some(f) => f.line_spacing,
        None => 8.0,
    }
}

/// `Utils.DrawBorderStringFourWay` (`Utils.cs:2464-2495`) with the fixed inputs
/// the boss bar uses: scale 1, `borderColor` black, `textColor` white
/// (`:65-69`), through `ui/text.rs`'s shared pass loop. `top_left` is the
/// `point - origin` of `health_text_positions`; the four `hud_item_text` border
/// passes sit at `top_left + FOUR_WAY_RING[..]` and the colour pass on top.
fn draw_border_string_four_way(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    top_left: [f32; 2],
    text: &str,
) {
    draw_border_string(
        r,
        lookups,
        BorderFont::Item,
        top_left[0],
        top_left[1],
        1.0,
        &FOUR_WAY_RING,
        TEXT_COLOR,
        text,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values derived from `BigProgressBarHelper.cs:25-70` /
    // `Utils.cs:947-950, 2464-2495` — never from this code.

    #[test]
    fn bar_rect_matches_the_source_geometry() {
        // `800 - 456/2`, `850 - 22/2` (`Utils.cs:947-950` truncation).
        assert_eq!(
            bar_rect(Vec2::new(1600.0, 900.0)),
            [572.0, 839.0, 456.0, 22.0]
        );
        // Centre `(640, 720 - 50)`.
        assert_eq!(
            bar_rect(Vec2::new(1280.0, 720.0)),
            [412.0, 659.0, 456.0, 22.0]
        );
    }

    #[test]
    fn frame_origin_and_icon_center_match_the_source() {
        // `:47` and `:52` — the icon centre reads the `(26, 28) / 2` literal
        // (Uncertainty 9), giving `+ (17, 34)` over the frame origin.
        assert_eq!(frame_origin(Vec2::new(1600.0, 900.0)), [540.0, 815.0]);
        assert_eq!(icon_center(Vec2::new(1600.0, 900.0)), [557.0, 849.0]);
    }

    #[test]
    fn fill_width_is_an_even_fraction_of_456() {
        assert_eq!(fill_width(1000.0, 1000.0), 456); // full bar
        assert_eq!(fill_width(0.0, 1000.0), 0); // empty bar
        assert_eq!(fill_width(1.0, 3.0), 152); // `(int)(456/3) = 152`, even
        assert_eq!(fill_width(1.0, 8.0), 56); // 57 -> 56 via `% 2` (`:35`)
    }

    #[test]
    fn bar_srcs_are_the_four_ui_boss_bar_rects() {
        assert_eq!(
            bar_srcs(),
            [
                [0.0, 174.0, 516.0, 58.0], // backdrop (`:31`)
                [32.0, 140.0, 2.0, 22.0],  // body: `116 + 24` (`:36-40`)
                [32.0, 82.0, 2.0, 22.0],   // cap: `58 + 24` (`:41-45`)
                [0.0, 0.0, 516.0, 58.0],   // frame (`:51`)
            ]
        );
    }

    #[test]
    fn health_text_positions_match_the_source_origins() {
        // `:64-69`: `800 - 10/2`, `851 - 20/2`; `795 - 30`; `805 - 0`.
        let (slash, current, max) = health_text_positions(
            [800.0, 851.0],
            [10.0, 20.0],
            [30.0, 20.0],
            [30.0, 20.0],
        );
        assert_eq!(slash, [795.0, 841.0]);
        assert_eq!(current, [765.0, 841.0]);
        assert_eq!(max, [805.0, 841.0]);
    }

    #[test]
    fn border_passes_are_the_four_four_way_offsets() {
        // `Utils.cs:2472-2492` — the centre colour pass is the shared
        // `text::draw_border_string`'s own, so the ring carries the four sides.
        assert_eq!(
            FOUR_WAY_RING,
            [
                [-2.0, 0.0],
                [2.0, 0.0],
                [0.0, -2.0],
                [0.0, 2.0],
            ]
        );
    }
}
