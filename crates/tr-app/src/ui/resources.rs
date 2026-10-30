//! Life / mana / breath resources (`ClassicPlayerResourcesDisplaySet`, `Main.cs:43406`).
//! Ported by Task 3 of the Whole-UI P1 plan.
//!
//! Faithful port of `ClassicPlayerResourcesDisplaySet.DrawLife` (lines 33-123) and
//! `DrawMana` (lines 125-180): the classic hearts anchored to the right edge, the
//! mana-star column, the breath bubbles above the player and the source-exact
//! "Life x/y" / "Mana" labels at y = 6.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::UiPulse;
use super::text::{hud_text, hud_text_width};
use crate::assets;
use crate::player::{BREATH_MAX, Player, WIDTH as PLAYER_WIDTH};

/// `ClassicPlayerResourcesDisplaySet.UIDisplay_ManaPerStar` (line 10): one star per 20 mana.
const MANA_PER_STAR: i32 = 20;

/// Screen centre (the XNA `position` passed to `SpriteBatch.Draw`, whose origin is the
/// sprite centre) of heart `i` (1-based).
///
/// Mirrors lines 102-119: `500 + 26*(i-1) + num7 + UI_ScreenAnchorX + Heart.Width()/2`
/// and `32 + (Heart.Height() - Heart.Height()*num5)/2 + num8 + Heart.Height()/2`, with
/// `num7 = -260` / `num8 = +26` once `i > 10` and `UI_ScreenAnchorX = screenWidth-800`.
pub fn life_heart_center(
    i: usize,
    screen_w: f32,
    heart_w: f32,
    heart_h: f32,
    scale: f32,
) -> [f32; 2] {
    let wrap_x = if i > 10 { -260.0 } else { 0.0 };
    let wrap_y = if i > 10 { 26.0 } else { 0.0 };
    [
        500.0 + 26.0 * (i as f32 - 1.0) + wrap_x + (screen_w - 800.0) + heart_w / 2.0,
        32.0 + (heart_h - heart_h * scale) / 2.0 + wrap_y + heart_h / 2.0,
    ]
}

/// Screen centre of mana star `i` (1-based).
///
/// Mirrors line 178: `775 + UI_ScreenAnchorX` for x (origin `Mana.Width()/2`) and
/// `30 + Mana.Height()/2 + (Mana.Height() - Mana.Height()*num3)/2 + 28*(i-1)` for y.
pub fn mana_star_center(
    i: usize,
    screen_w: f32,
    mana_w: f32,
    mana_h: f32,
    scale: f32,
) -> [f32; 2] {
    let _ = mana_w;
    [
        775.0 + (screen_w - 800.0),
        30.0 + mana_h / 2.0 + (mana_h - mana_h * scale) / 2.0 + 28.0 * (i as f32 - 1.0),
    ]
}

/// Label origin (left-aligned) and value anchor (right-aligned, XNA origin =
/// `MeasureString(value).X`) for the life text at y = 6 (lines 60-65):
/// `500 + 13*num3 ∓ MeasureString("Life max/max").X*0.5 + (screenWidth-800)`.
pub fn life_text_positions(num3: usize, total_text_w: f32, screen_w: f32) -> ([f32; 2], [f32; 2]) {
    let cx = 500.0 + 13.0 * num3 as f32 + (screen_w - 800.0);
    (
        [cx - total_text_w * 0.5, 6.0],
        [cx + total_text_w * 0.5, 6.0],
    )
}

/// Number of mana stars: `statManaMax2 / UIDisplay_ManaPerStar` (line 143), clamped at 0.
pub fn mana_star_count(mana_max: i32) -> usize {
    (mana_max / MANA_PER_STAR).max(0) as usize
}

/// x of the "Mana" label (line 137-142): `800 - num + UI_ScreenAnchorX`, where `num`
/// is 50 unless `MeasureString("Mana").X >= 45` (then `num = width + 5`).
pub fn mana_label_x(label_w: f32, screen_w: f32) -> f32 {
    screen_w - (if label_w >= 45.0 { label_w + 5.0 } else { 50.0 })
}

/// `Player.statMana` / `Player.statManaMax` (`Player.cs:1930-1932`); the classic
/// display reads them directly.
fn player_mana(p: &Player) -> (i32, i32) {
    (p.mana, p.mana_max)
}

/// `ClassicPlayerResourcesDisplaySet.Draw` (line 26): life, then mana.
///
/// `camera`/`zoom` are only used by the breath bubbles, which follow the player rather
/// than the fixed screen anchor.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    player: Option<&Player>,
    camera: Vec2,
    zoom: f32,
    pulse: UiPulse,
) {
    let Some(p) = player else { return };
    // `Player.ghost` is not modelled by `tr_app::Player`; there is no ghost state to
    // early-return on yet (see report).
    draw_life(r, lookups, screen, p, pulse);
    draw_breath(r, lookups, screen, p, camera, zoom);
    draw_mana(r, lookups, screen, p, pulse);
}

/// `ClassicPlayerResourcesDisplaySet.DrawLife` (lines 33-123).
fn draw_life(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    p: &Player,
    pulse: UiPulse,
) {
    let Some(heart_rect) = lookups.ui_heart.as_ref() else {
        return;
    };
    let Some(heart_uv) = lookups.hud_uv(lookups.ui_heart.as_ref()) else {
        return;
    };
    let heart_w = heart_rect.width as f32;
    let heart_h = heart_rect.height as f32;
    // `new Color(Main.mouseTextColor, ..., Main.mouseTextColor)` (line 37).
    let t = pulse.text;
    let color = [t, t, t, t];

    // `statLifeMax` and `statLifeMax2` both map to `p.max_hp` (the port has no separate
    // base/bonus max). Lines 43-59:
    let life_max = p.max_hp;
    let mut num2 = (life_max - 400) / 5;
    if num2 < 0 {
        num2 = 0;
    }
    let mut life_per_heart = 20.0f32; // UIDisplay_LifePerHeart (line 38)
    if num2 > 0 {
        // `num` (line 51) only feeds the `(statLifeMax2-statLifeMax)/num` term, which is
        // 0 while both maxima are `p.max_hp`; only the per-heart value matters here.
        life_per_heart = life_max as f32 / 20.0;
    }
    // `+= (statLifeMax2 - statLifeMax)/num` is 0 here (both are `max_hp`).
    let hearts = (life_max as f32 / life_per_heart) as i32;
    let num3 = hearts.min(10) as usize;

    // Lines 60-65: the text is measured as "Life <max>/<max>" regardless of current hp.
    let label = "Life"; // Lang.inter[0]
    let value = format!("{}/{}", p.hp, life_max);
    let measured = format!("{label} {life_max}/{life_max}");
    let total_text_w = hud_text_width(r, lookups, &measured, 1.0);
    let (label_pos, value_pos) = life_text_positions(num3, total_text_w, screen.x);
    hud_text(
        r,
        lookups,
        label_pos[0],
        label_pos[1],
        1.0,
        color,
        label,
    );
    let value_w = hud_text_width(r, lookups, &value, 1.0);
    hud_text(
        r,
        lookups,
        value_pos[0] - value_w,
        value_pos[1],
        1.0,
        color,
        &value,
    );

    // Lines 67-122.
    for i in 1..=hearts {
        let mut num4 = 255i32;
        let mut num5 = 1.0f32;
        let mut flag = false;
        let threshold = i as f32 * life_per_heart;
        if p.hp as f32 >= threshold {
            if p.hp as f32 == threshold {
                flag = true;
            }
        } else {
            let num6 = (p.hp as f32 - (i - 1) as f32 * life_per_heart) / life_per_heart;
            num4 = (30.0 + 225.0 * num6) as i32;
            if num4 < 30 {
                num4 = 30;
            }
            num5 = num6 / 4.0 + 0.75;
            if num5 < 0.75 {
                num5 = 0.75;
            }
            if num6 > 0.0 {
                flag = true;
            }
        }
        if flag {
            // `num5 += Main.cursorScale - 1f` (line 100).
            num5 += pulse.cursor_scale - 1.0;
        }
        let center = life_heart_center(i as usize, screen.x, heart_w, heart_h, num5);
        let size = [heart_w * num5, heart_h * num5];
        let pos = [center[0] - size[0] / 2.0, center[1] - size[1] / 2.0];
        let t = num4 as f32 / 255.0;
        // `a = (int)(num4 * 0.9)` (line 109).
        r.push_ui_sprite(pos, size, heart_uv, [t, t, t, t * 0.9]);
    }
}

/// Breath bubbles above the player's head while submerged (moved from `ui::mod`).
fn draw_breath(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    p: &Player,
    camera: Vec2,
    zoom: f32,
) {
    if p.breath >= BREATH_MAX {
        return;
    }
    let Some(bubble) = lookups.hud_uv(lookups.ui_bubble.as_ref()) else {
        return;
    };
    let psx = (p.x + PLAYER_WIDTH * 0.5 - camera.x) * zoom + screen.x * 0.5;
    let psy = (p.y - camera.y) * zoom + screen.y * 0.5;
    let bx = psx - 125.0;
    let by = psy - 100.0;
    for i in 1..=10 {
        let ratio = ((p.breath as f32 - (i - 1) as f32 * 20.0) / 20.0).clamp(0.0, 1.0);
        let (grey, scale) = if p.breath >= i * 20 {
            (255.0f32, 1.0f32)
        } else {
            (
                (30.0 + 225.0 * ratio).max(30.0),
                (ratio / 4.0 + 0.75).max(0.75),
            )
        };
        let size = 22.0 * scale;
        let x = bx + 26.0 * (i - 1) as f32;
        let y = by + (22.0 - size) * 0.5;
        let t = grey / 255.0;
        r.push_ui_sprite([x, y], [size, size], bubble, [t, t, t, 1.0]);
    }
}

/// `ClassicPlayerResourcesDisplaySet.DrawMana` (lines 125-180).
fn draw_mana(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    p: &Player,
    pulse: UiPulse,
) {
    let (mana, mana_max) = player_mana(p);
    if mana_max <= 0 {
        return;
    }
    let Some(mana_rect) = lookups.ui_mana.as_ref() else {
        return;
    };
    let Some(mana_uv) = lookups.hud_uv(lookups.ui_mana.as_ref()) else {
        return;
    };
    let mana_w = mana_rect.width as f32;
    let mana_h = mana_rect.height as f32;
    // `new Color(Main.mouseTextColor, ...)` (line 129).
    let t = pulse.text;
    let color = [t, t, t, t];

    // Lines 136-142.
    let label = "Mana"; // Lang.inter[2]
    let label_w = hud_text_width(r, lookups, label, 1.0);
    hud_text(r, lookups, mana_label_x(label_w, screen.x), 6.0, 1.0, color, label);

    // Lines 143-179.
    let stars = mana_star_count(mana_max);
    for i in 1..=stars {
        let mut num2 = 255i32;
        let mut num3 = 1.0f32;
        let mut flag = false;
        if mana >= i as i32 * MANA_PER_STAR {
            if mana == i as i32 * MANA_PER_STAR {
                flag = true;
            }
        } else {
            let num4 = (mana - (i as i32 - 1) * MANA_PER_STAR) as f32 / MANA_PER_STAR as f32;
            num2 = (30.0 + 225.0 * num4) as i32;
            if num2 < 30 {
                num2 = 30;
            }
            num3 = num4 / 4.0 + 0.75;
            if num3 < 0.75 {
                num3 = 0.75;
            }
            if num4 > 0.0 {
                flag = true;
            }
        }
        if flag {
            // `num3 += Main.cursorScale - 1f` (line 176).
            num3 += pulse.cursor_scale - 1.0;
        }
        let center = mana_star_center(i, screen.x, mana_w, mana_h, num3);
        let size = [mana_w * num3, mana_h * num3];
        let pos = [center[0] - size[0] / 2.0, center[1] - size[1] / 2.0];
        let t = num2 as f32 / 255.0;
        r.push_ui_sprite(pos, size, mana_uv, [t, t, t, t * 0.9]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hearts_start_at_screen_width_minus_300() {
        assert_eq!(
            life_heart_center(1, 1600.0, 22.0, 22.0, 1.0),
            [1600.0 - 300.0 + 11.0, 32.0 + 11.0]
        );
    }
    #[test]
    fn hearts_wrap_after_ten() {
        let a = life_heart_center(10, 1600.0, 22.0, 22.0, 1.0);
        let b = life_heart_center(11, 1600.0, 22.0, 22.0, 1.0);
        assert_eq!([b[0] - a[0], b[1] - a[1]], [-260.0 + 26.0, 26.0]);
    }
    #[test]
    fn mana_stars_sit_at_screen_width_minus_25() {
        assert_eq!(mana_star_center(1, 1600.0, 22.0, 22.0, 1.0)[0], 1575.0);
        assert_eq!(
            mana_star_center(2, 1600.0, 22.0, 22.0, 1.0)[1]
                - mana_star_center(1, 1600.0, 22.0, 22.0, 1.0)[1],
            28.0
        );
    }
    #[test]
    fn mana_label_uses_fifty_or_width_plus_five() {
        assert_eq!(mana_label_x(30.0, 1600.0), 1550.0);
        assert_eq!(mana_label_x(60.0, 1600.0), 1535.0);
    }
    #[test]
    fn mana_with_zero_max_draws_no_stars() {
        assert_eq!(mana_star_count(0), 0);
        assert_eq!(mana_star_count(-10), 0);
        assert_eq!(mana_star_count(40), 2);
    }
}
