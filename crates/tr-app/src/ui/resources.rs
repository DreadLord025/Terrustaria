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

/// `Main.DrawInterface_Resources_Buffs`'s slot layout (`Main.cs:43215-43224`):
/// 11 icons per row, 38 px pitch, the rows 50 px apart from the 76 px first
/// row — the occupied slots pack (the empty ones never advance the count).
pub fn buff_slot_pos(num3: usize) -> [f32; 2] {
    let mut x = 32.0 + num3 as f32 * 38.0;
    let mut y = 76.0;
    let mut num5 = num3 as i32;
    while num5 >= 11 {
        num5 -= 11;
        x = 32.0 + num5 as f32 * 38.0;
        y += 50.0;
    }
    [x, y]
}

/// `Lang.LocalizedDuration` (`Lang.cs:1158-1207`) with the call's
/// `(abbreviated: true, showAllAvailableUnits: false)`: the biggest unit
/// wins and the running remainder ceilings into it (`time > 1 day` →
/// `days + 1`). The `Misc.Short*` keys ride the localized json.
pub(crate) fn localized_duration(seconds: i32) -> String {
    let t = seconds as i64;
    if t / 86400 > 0 {
        let n = if t > 86400 { t / 86400 + 1 } else { t / 86400 };
        return format!("{n} {}", crate::lang::text("Misc.ShortDays"));
    }
    if t / 3600 > 0 {
        let n = if t > 3600 { t / 3600 + 1 } else { t / 3600 };
        return format!("{n} {}", crate::lang::text("Misc.ShortHours"));
    }
    if t / 60 > 0 {
        let n = if t > 60 { t / 60 + 1 } else { t / 60 };
        return format!("{n} {}", crate::lang::text("Misc.ShortMinutes"));
    }
    format!("{t} {}", crate::lang::text("Misc.ShortSeconds"))
}

/// `Main.TryGetBuffTime` (`Main.cs:43290-43305`): the duration-text gate —
/// the vanity/light pets and `Main.buffNoTimeDisplay` hide it, the honey-48
/// wet gate, the expert wet-46 gate, and the vampire seed's sub-10 gates for
/// 23/24/32. Returns the tick count.
pub(crate) fn try_get_buff_time(p: &Player, slot: usize, vampire_seed: bool) -> Option<i32> {
    let b = p.buffs.buff_type[slot];
    if crate::buffs::is_vanity_pet(b)
        || crate::buffs::is_light_pet(b)
        || crate::buffs::is_no_time_display(b)
        || (p.honey_wet && b == 48)
        || (p.wet && p.buffs.expert_mode && b == 46)
    {
        return None;
    }
    let t = p.buffs.buff_time[slot];
    if vampire_seed && t < 10 && (b == 24 || b == 23 || b == 32) {
        return None;
    }
    Some(t)
}

/// `Main.DrawBuffIcon` (`Main.cs:43307-43368`) + the grid walk of
/// `DrawInterface_Resources_Buffs` (`:43203-43250`). The icons tint by the
/// per-slot alpha (`Main.buffAlpha` — the port's `buff_alpha`), the hover
/// raises it 0.1 and the rest decays 0.05 clamped to [0.4, 1]; the
/// right-click-release pushes the slot onto `removals` (the C# removes in
/// the draw — the port drains the request in the tick). The hover's
/// `drawBuffText` slot feeds the name/tooltip block (`:43250-43270`), which
/// stays out of this pass (the tooltip text rides the localization work).
/// The fullscreen-map second call site (`:41335`) rides the map tasks.
#[allow(clippy::too_many_arguments)]
pub fn draw_buffs(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    player: &Player,
    buff_alpha: &mut [f32; crate::buffs::MAX_BUFFS],
    mouse: Vec2,
    right_release: bool,
    removals: &mut Vec<usize>,
    vampire_seed: bool,
) {
    let mut num3 = 0usize;
    for i in 0..crate::buffs::MAX_BUFFS {
        let bt = player.buffs.buff_type[i];
        // The empty-slot reset (`:43231-43233`).
        if bt == 0 {
            buff_alpha[i] = 0.4;
            continue;
        }
        let [x, y] = buff_slot_pos(num3);
        num3 += 1;
        let Some(rect) = lookups.buff_icons.get(&(bt as u16)) else {
            continue;
        };
        let (w, h) = (rect.width as f32, rect.height as f32);
        let Some(uv) = lookups.hud_uv(Some(rect)) else {
            continue;
        };
        // The icon (`:43311`): the full sheet tinted by the slot alpha.
        let a = buff_alpha[i];
        r.push_ui_sprite([x, y], [w, h], uv, [a, a, a, a]);
        // The duration text (`:43317-43327`): `FontAssets.ItemStack` at 0.8
        // under the icon.
        if let Some(t) = try_get_buff_time(player, i, vampire_seed) {
            if t > 2 {
                let text = localized_duration(t / 60);
                crate::ui::text::hud_text(r, lookups, x, y + h, 0.85, [1.0; 4], &text);
            }
        }
        // The hover + the right-click removal (`:43328-43349`).
        if mouse.x < x + w && mouse.y < y + h && mouse.x > x && mouse.y > y {
            buff_alpha[i] += 0.1;
            if right_release {
                removals.push(i);
            }
        } else {
            buff_alpha[i] -= 0.05;
        }
        // The clamp (`:43355-43361`).
        if buff_alpha[i] > 1.0 {
            buff_alpha[i] = 1.0;
        } else if (buff_alpha[i] as f64) < 0.4 {
            buff_alpha[i] = 0.4;
        }
    }
}

#[cfg(test)]
mod s1_e_6_tests {
    use super::*;

    #[test]
    fn buff_slot_grid_wraps_at_11_with_the_38_50_pitch() {
        // `Main.cs:43215-43224`: the first row `32 + i*38` at y 76, the wrap
        // resets x and steps y by 50.
        assert_eq!(buff_slot_pos(0), [32.0, 76.0]);
        assert_eq!(buff_slot_pos(10), [32.0 + 10.0 * 38.0, 76.0]);
        assert_eq!(buff_slot_pos(11), [32.0, 126.0]);
        assert_eq!(buff_slot_pos(12), [70.0, 126.0]);
    }

    #[test]
    fn localized_duration_ceilings_the_running_remainder() {
        // `Lang.cs:1158-1207` with (abbreviated, showAll=false): the biggest
        // unit wins, `time > 1 unit` bumps the count.
        assert!(localized_duration(30).starts_with("30 "), "s: {}", localized_duration(30));
        assert!(localized_duration(90).starts_with("2 "), "m+1: {}", localized_duration(90));
        assert!(localized_duration(3600).starts_with("1 "), "h: {}", localized_duration(3600));
        assert!(localized_duration(5400).starts_with("2 "), "h+1: {}", localized_duration(5400));
        assert!(localized_duration(86_400 * 2 + 1).starts_with("3 "), "d+1: {}", localized_duration(86_400 * 2 + 1));
    }
}

