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
/// than the fixed screen anchor. `inventory_open`/`options_open`/`fancy_ui` feed the
/// breath meter's anchor walk (`Main.cs:43422-43438`).
#[allow(clippy::too_many_arguments)]
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    player: Option<&Player>,
    camera: Vec2,
    zoom: f32,
    pulse: UiPulse,
    inventory_open: bool,
    options_open: bool,
    fancy_ui: bool,
) {
    let Some(p) = player else { return };
    // `Player.ghost` gates the breath/lava meters only (`Main.cs:43440`/
    // `:43476`); the hearts/mana draw for the ghost too (the C# resource set
    // has no dead/ghost early-return — `ClassicPlayerResourcesDisplaySet`).
    draw_life(r, lookups, screen, p, pulse);
    draw_breath(
        r, lookups, screen, p, camera, zoom, inventory_open, options_open, fancy_ui,
    );
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

/// `Main.DrawInterface_Resources_Breath`'s per-bubble visual
/// (`Main.cs:43446-43465`): a full bubble (value >= i*per) is grey 255 /
/// scale 1; a partial one ramps `num4 = (value - (i-1)*per)/per`, grey
/// `(int)(30 + 225*num4)` floored at 30, scale `num4/4 + 0.75` floored at
/// 0.75. `i` is 1-based. Shared by the breath bubbles (per 20) and the lava
/// flames (per `lavaMax/10`).
pub fn breath_bubble_visual(value: i32, i: i32, per: i32) -> (i32, f32) {
    if value >= i * per {
        return (255, 1.0);
    }
    let num4 = (value - (i - 1) * per) as f32 / per as f32;
    let grey = ((30.0 + 225.0 * num4) as i32).max(30);
    let scale = (num4 / 4.0 + 0.75).max(0.75);
    (grey, scale)
}

/// `Main.DrawInterface_Resources_Breath`'s anchor walk (`Main.cs:43421-43438`):
/// `player.Top + (0, gfxOffY)`, the open-inventory drop (`height - 20` when
/// `screenHeight < 1000`), the world→screen zoom transform, the `-100` lift
/// when the inventory is closed (or the screen is tall), then the
/// options/fancy-UI re-anchor. Returns the screen-space anchor the bubble
/// row hangs from. `UIScale`'s `/ Main.UIScale` (`:43431`) cancels the
/// `UIScaleMatrix` batch the C# draws in — the port's whole HUD is unscaled
/// (the S1-H-3 deviation, ANALYSIS "UI scale"), so the anchor stays raw.
pub fn breath_meter_anchor(
    p: &Player,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    inventory_open: bool,
    options_open: bool,
    fancy_ui: bool,
) -> Vec2 {
    let mut v = Vec2::new(p.x + PLAYER_WIDTH * 0.5, p.y + p.gfx_off_y);
    if inventory_open && screen.y < 1000.0 {
        v.y += p.height - 20.0;
    }
    v = (v - camera) * zoom + screen * 0.5;
    if !inventory_open || screen.y >= 1000.0 {
        v.y -= 100.0;
    }
    if options_open || fancy_ui {
        v = Vec2::new(screen.x * 0.5, screen.y * 0.5 + 236.0);
        if fancy_ui {
            v.y = screen.y - 64.0;
        }
    }
    v
}

/// `Main.DrawInterface_Resources_Breath` (`Main.cs:43406-43513`): the breath
/// bubbles above the player's head while submerged, and the lava flames
/// while the lava budget drains. The dead player draws nothing (`:43409`);
/// the `flag` suppression (`:43413-43420`) hides the bubbles while the lava
/// budget is below max AND (in lava OR breath full); the ghost gate
/// (`:43440`/`:43476`) rides the hardcore-ghost lane's `Player.ghost`.
fn draw_breath(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    p: &Player,
    camera: Vec2,
    zoom: f32,
    inventory_open: bool,
    options_open: bool,
    fancy_ui: bool,
) {
    // `:43409-43412`: the dead early-return.
    if p.dead {
        return;
    }
    // `:43413-43420`: the lava-suppression flag.
    let lava_draining = p.lava_time < p.lava_max;
    let flag = lava_draining && (p.lava_wet || p.breath == BREATH_MAX);
    let anchor = breath_meter_anchor(
        p, camera, zoom, screen, inventory_open, options_open, fancy_ui,
    );
    // `:43440-43475`: the breath bubbles — `breath < breathMax && !ghost &&
    // !flag`, one per 20 breath, the `i > 10` wrap (`-260 x, +26 y`).
    if p.breath < BREATH_MAX && !p.ghost && !flag {
        if let Some(bubble) = lookups.hud_uv(lookups.ui_bubble.as_ref()) {
            let (bw, bh) = sheet_size(lookups.ui_bubble.as_ref());
            let per = 20;
            let count = BREATH_MAX / per;
            for i in 1..=count {
                let (grey, scale) = breath_bubble_visual(p.breath, i, per);
                let (wrap_x, wrap_y) = if i > 10 { (-260.0, 26.0) } else { (0.0, 0.0) };
                let x = anchor.x + 26.0 * (i - 1) as f32 + wrap_x - 125.0;
                let y = anchor.y + 32.0 + (bh - bh * scale) * 0.5 + wrap_y;
                let t = grey as f32 / 255.0;
                // `Color(num2, num2, num2, num2)` (`:43473`) — alpha rides grey.
                r.push_ui_sprite([x, y], [bw * scale, bh * scale], bubble, [t, t, t, t]);
            }
        }
    }
    // `:43476-43512`: the lava flames — `(lavaTime < lavaMax && !ghost) & flag`.
    if !((lava_draining && !p.ghost) & flag) {
        return;
    }
    let Some(flame) = lookups.hud_uv(lookups.ui_flame.as_ref()) else {
        return;
    };
    let (fw, fh) = sheet_size(lookups.ui_flame.as_ref());
    // `num7 = lavaMax / 10` (`:43480`); the gate above guarantees lavaMax > 0.
    let per = p.lava_max / 10;
    if per <= 0 {
        return;
    }
    let count = p.lava_max / per;
    for j in 1..=count {
        let (grey, scale) = breath_bubble_visual(p.lava_time, j, per);
        let (wrap_x, wrap_y) = if j > 10 { (-260.0, 26.0) } else { (0.0, 0.0) };
        let x = anchor.x + 26.0 * (j - 1) as f32 + wrap_x - 125.0;
        let y = anchor.y + 32.0 + (fh - fh * scale) * 0.5 + wrap_y;
        let t = grey as f32 / 255.0;
        // The C# source rect is `Rectangle(0, 0, Bubble.Width(), Bubble.Height())`
        // (`:43511`) — Flame and Bubble are both 22x22, so the whole flame sheet.
        r.push_ui_sprite([x, y], [fw * scale, fh * scale], flame, [t, t, t, t]);
    }
}

/// The native pixel size of a HUD sheet (0x0 when absent).
fn sheet_size(rect: Option<&tr_render::SheetRect>) -> (f32, f32) {
    rect.map_or((0.0, 0.0), |s| (s.width as f32, s.height as f32))
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

/// `BuffID.Sets.BuffTextHandlers` (`BuffID.cs:54-168`): the 28 buff ids whose
/// icon text is a live projectile counter instead of the duration. Every entry
/// is a `CachedProjectileCounterBuffTextHandler(projectiles...)`
/// (`DataStructures/CachedProjectileCounterBuffTextHandler.cs:3-30`), which
/// sums the local player's `ownedProjectileCounts` over the listed types.
const BUFF_TEXT_HANDLERS: [(i32, &[i32]); 28] = [
    (64, &[266]),
    (125, &[373]),
    (49, &[191, 192, 193, 194]),
    (83, &[317]),
    (126, &[375]),
    (134, &[387]),
    (133, &[390, 391, 392]),
    (135, &[393, 394, 395]),
    (139, &[407]),
    (140, &[423]),
    (161, &[533]),
    (182, &[613]),
    (188, &[626]),
    (213, &[755]),
    (214, &[758]),
    (216, &[759]),
    (263, &[831]),
    (271, &[864]),
    (322, &[946]),
    (325, &[951]),
    (335, &[970]),
    (355, &[1022]),
    (385, &[1093]),
    (386, &[1094]),
    (389, &[1112]),
    (390, &[1113]),
    (393, &[1118]),
    (394, &[1119]),
];

/// `CachedProjectileCounterBuffTextHandler.HandleBuffText`
/// (`DataStructures/CachedProjectileCounterBuffTextHandler.cs:12-30`): the
/// summed `ownedProjectileCounts` of the handler's types as `"xN"`, or `None`
/// when the sum is zero (the caller then falls back to the duration).
pub(crate) fn buff_text(p: &Player, slot: usize) -> Option<String> {
    let b = p.buffs.buff_type[slot];
    let (_, kinds) = BUFF_TEXT_HANDLERS.iter().find(|(id, _)| *id == b)?;
    let n: i32 = kinds
        .iter()
        .map(|t| p.owned_projectile_counts[*t as usize])
        .sum();
    (n > 0).then(|| format!("x{n}"))
}

/// `Main.GetBuffTooltip` (`Main.cs:43254-43289`): the buff description with
/// its per-buff overrides. The three WellFed tiers swap to their `_Expert`
/// rows under `Main.expertMode`; the `ManaSickness` (94) and `ManaHeat` (396)
/// `switch` arms append a player-computed percentage and are documented drops
/// — the port has no `Player.manaSickReduction` / `Player.manaHeat`.
pub(crate) fn buff_tooltip(p: &Player, buff: i32) -> String {
    let expert = p.buffs.expert_mode;
    match buff {
        26 if expert => crate::lang::text("BuffDescription.WellFed_Expert"),
        206 if expert => crate::lang::text("BuffDescription.WellFed2_Expert"),
        207 if expert => crate::lang::text("BuffDescription.WellFed3_Expert"),
        _ => crate::lang::buff_description(buff),
    }
}

/// `MouseText_DrawBuffTooltip`'s banner list (`Main.cs:21360-21388`): when the
/// hovered buff is the Monster Banner (147), `Main.bannerMouseOver` is set and
/// every banner type the player owns prints its NPC name under the tooltip.
/// `HasNPCBannerBuff` is the port's `SceneMetrics::npc_banner_buff` scan
/// (`Player.cs:44197`), and `BannerSystem.BannerToNPC` is
/// `tr_core::bestiary::banner_to_npc`.
pub(crate) fn banner_hover_names(npc_banner_buff: &[bool]) -> Vec<String> {
    let mut out = Vec::new();
    for (banner, owned) in npc_banner_buff.iter().enumerate() {
        if !owned {
            continue;
        }
        let npc = tr_core::bestiary::banner_to_npc(banner as i32);
        if npc != 0 {
            out.push(crate::lang::npc_name(npc));
        }
    }
    out
}

/// `MouseTextInner`'s rarity colour table (`Main.cs:20245-20310`) — the
/// `MouseTextHackZoom` flavour the buff hover passes its `rare` through. It
/// matches the item-name table (`tooltip::item_name_color`) except for `-10`,
/// the melee-buff green `(65, 255, 110)` (`:20259-20262`), which the item
/// table never uses.
fn mouse_text_color(rare: i32) -> [f32; 4] {
    match rare {
        -10 => [65.0 / 255.0, 255.0 / 255.0, 110.0 / 255.0, 1.0],
        // `-13` (master) and `-12` (disco) read `Main.masterColor` /
        // `Main.Disco*` — documented drops, no such port global.
        _ => crate::ui::tooltip::item_name_color(rare),
    }
}

/// `MouseText_DrawBuffTooltip` (`Main.cs:21345-21489`) + the `MouseTextInner`
/// plain-text tail (`:20218-20319`), via the shared block printer. The C# draws
/// the tooltip string below the name (`:20221` before the name at `:20319`),
/// both at `Main.mouseTextColor`, and appends the banner names in the green
/// `(80, 255, 120)` tint (`:21478`).
pub(crate) fn draw_buff_hover(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mouse: Vec2,
    name: &str,
    tooltip: &str,
    rarity: i32,
    screen: Vec2,
    pulse: UiPulse,
    banner_names: Vec<String>,
) {
    let mut lines = vec![(name.to_string(), mouse_text_color(rarity))];
    if !tooltip.is_empty() {
        lines.push((tooltip.to_string(), [1.0, 1.0, 1.0, 1.0]));
    }
    for npc in banner_names {
        lines.push((npc, [80.0 / 255.0, 255.0 / 255.0, 120.0 / 255.0, 1.0]));
    }
    crate::ui::tooltip::draw_mouse_text_block(r, lookups, mouse, &lines, screen, pulse);
}

/// `Main.DrawBuffIcon` (`Main.cs:43307-43368`) + the grid walk of
/// `DrawInterface_Resources_Buffs` (`:43203-43250`). The icons tint by the
/// per-slot alpha (`Main.buffAlpha` — the port's `buff_alpha`), the hover
/// raises it 0.1 and the rest decays 0.05 clamped to [0.4, 1]; the
/// right-click (`Main.mouseRight && Main.mouseRightRelease`, `:43333`)
/// pushes the slot onto `removals` (the C# removes in
/// the draw — the port drains the request in the tick). The icon text is the
/// `BuffID.Sets.BuffTextHandlers` counter when the buff has one
/// (`:43317-43320`), else the duration when over 2 ticks (`:43321-43324`).
/// Returns the hovered slot — the C#'s `drawBuffText` — which the caller turns
/// into the name/tooltip block (`:43234-43251`).
/// The fullscreen-map second call site (`:41335`) rides the map tasks.
#[allow(clippy::too_many_arguments)]
pub fn draw_buffs(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    player: &Player,
    buff_alpha: &mut [f32; crate::buffs::MAX_BUFFS],
    mouse: Vec2,
    // `Main.mouseRight && Main.mouseRightRelease` (`Main.cs:43333`) — the
    // cancel needs BOTH the held state and the release latch; the port used
    // to pass the bare edge, which fired on release-while-held frames too.
    right_click: bool,
    removals: &mut Vec<usize>,
    vampire_seed: bool,
) -> Option<usize> {
    let mut num3 = 0usize;
    let mut hovered = None;
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
        // The icon text (`:43316-43327`): the `BuffTextHandlers` counter wins
        // when the handler yields one, else the duration over 2 ticks —
        // `FontAssets.ItemStack` at 0.8 under the icon.
        let text = buff_text(player, i).or_else(|| {
            try_get_buff_time(player, i, vampire_seed)
                .filter(|t| *t > 2)
                .map(|t| localized_duration(t / 60))
        });
        if let Some(text) = text {
            crate::ui::text::hud_text(r, lookups, x, y + h, 0.85, [1.0; 4], &text);
        }
        // The hover + the right-click removal (`:43329-43349`). The C#'s
        // `!PlayerInput.IgnoreMouseInterface` gate has no port counterpart.
        if mouse.x < x + w && mouse.y < y + h && mouse.x > x && mouse.y > y {
            hovered = Some(i);
            buff_alpha[i] += 0.1;
            // `:43333` — `Main.mouseRight && Main.mouseRightRelease`.
            if right_click {
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
    hovered
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

    #[test]
    fn the_buff_text_handler_table_is_the_csharp_set() {
        // `BuffID.cs:54-168`: 28 ids, each a `CachedProjectileCounterBuffTextHandler`.
        // Every entry is present with its exact projectile list, and no extra id
        // sneaks in.
        assert_eq!(BUFF_TEXT_HANDLERS.len(), 28);
        let ids: Vec<i32> = BUFF_TEXT_HANDLERS.iter().map(|(id, _)| *id).collect();
        let expected = [
            64, 125, 49, 83, 126, 134, 133, 135, 139, 140, 161, 182, 188, 213, 214, 216, 263,
            271, 322, 325, 335, 355, 385, 386, 389, 390, 393, 394,
        ];
        assert_eq!(ids, expected, "the handler ids in source order");
        // Spot-check the multi-type handlers (`BuffID.cs:66`, `:82`, `:86`).
        let get = |id: i32| BUFF_TEXT_HANDLERS.iter().find(|(k, _)| *k == id).unwrap().1.to_vec();
        assert_eq!(get(49), vec![191, 192, 193, 194]);
        assert_eq!(get(133), vec![390, 391, 392]);
        assert_eq!(get(135), vec![393, 394, 395]);
        assert_eq!(get(64), vec![266]);
    }

    #[test]
    fn the_buff_text_counter_is_xn_and_gated_on_a_positive_sum() {
        // `CachedProjectileCounterBuffTextHandler.HandleBuffText`
        // (`CachedProjectileCounterBuffTextHandler.cs:12-30`): the summed
        // counts as `"xN"`, nothing at zero.
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        // The buff 64's handler watches the projectile 266 (`BuffID.cs:58`).
        p.buffs.buff_type[0] = 64;
        assert_eq!(buff_text(&p, 0), None, "no owned projectiles yet");
        p.owned_projectile_counts[266] = 3;
        assert_eq!(buff_text(&p, 0).as_deref(), Some("x3"));
        // The buff 49 sums four types (`BuffID.cs:66`): 1 + 2 = 3.
        p.buffs.buff_type[0] = 49;
        assert_eq!(buff_text(&p, 0), None);
        p.owned_projectile_counts[191] = 1;
        p.owned_projectile_counts[192] = 2;
        assert_eq!(buff_text(&p, 0).as_deref(), Some("x3"));
        // A buff with no handler yields nothing — the duration path takes over.
        p.buffs.buff_type[0] = 1;
        assert_eq!(buff_text(&p, 0), None);
    }

    #[test]
    fn get_buff_tooltip_swaps_the_well_fed_rows_under_expert() {
        // `test_register_source` reloads the process-global language manager;
        // hold the guard for the registration and the reads alike.
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        // `Main.GetBuffTooltip` (`Main.cs:43254-43289`): the three fed tiers
        // (26 / 206 / 207) use their `_Expert` rows in expert mode, everything
        // else is the plain description.
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{
                "BuffName": { "WellFed": "Well Fed" },
                "BuffDescription": {
                    "WellFed": "Minor improvements to all stats",
                    "WellFed_Expert": "Medium improvements to all stats",
                    "WellFed2": "Major improvements to all stats",
                    "WellFed2_Expert": "Major improvements to all stats and a bit more",
                    "WellFed3": "Exquisite improvements to all stats",
                    "WellFed3_Expert": "Exquisite improvements to all stats and more"
                }
            }"#,
        );
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        p.buffs.expert_mode = false;
        assert_eq!(buff_tooltip(&p, 26), "Minor improvements to all stats");
        p.buffs.expert_mode = true;
        assert_eq!(buff_tooltip(&p, 26), "Medium improvements to all stats");
        assert_eq!(buff_tooltip(&p, 206), "Major improvements to all stats and a bit more");
        assert_eq!(buff_tooltip(&p, 207), "Exquisite improvements to all stats and more");
        // A non-fed buff ignores the flag.
        p.buffs.expert_mode = false;
        assert_eq!(buff_tooltip(&p, 26), "Minor improvements to all stats");
        // The `ManaSickness` (94) / `ManaHeat` (396) percentage arms are
        // documented drops: they fall back to the plain description row (no
        // `Player.manaSickReduction` / `manaHeat` in the port).
        assert_eq!(buff_tooltip(&p, 396), crate::lang::buff_description(396));
    }

    #[test]
    fn the_melee_buff_hover_colour_is_the_minus_ten_green() {
        // `MouseTextInner` (`Main.cs:20259-20262`): `rare == -10` is
        // `(65, 255, 110)`; the buff hover passes it for the melee buffs
        // (`Main.cs:43243-43246`).
        assert_eq!(
            mouse_text_color(-10),
            [65.0 / 255.0, 255.0 / 255.0, 110.0 / 255.0, 1.0]
        );
        // The shared arms fall through to the item-name table.
        assert_eq!(mouse_text_color(0), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(
            mouse_text_color(1),
            [150.0 / 255.0, 150.0 / 255.0, 1.0, 1.0]
        );
    }

    #[test]
    fn the_banner_hover_lists_only_owned_banners_with_a_mapped_npc() {
        // `MouseText_DrawBuffTooltip` (`Main.cs:21363-21387`): every
        // `BannerSystem.BannerToNPC(i) != 0 && HasNPCBannerBuff(i)` row.
        let mut owned = vec![false; 320];
        assert!(banner_hover_names(&owned).is_empty());
        // Banner 69 maps to a live NPC (`bestiary::banner_to_npc`).
        owned[69] = true;
        let names = banner_hover_names(&owned);
        assert_eq!(names.len(), 1, "{names:?}");
    }

    /// W4-c46 — `Main.DrawInterface_Resources_Breath`'s per-bubble visual
    /// (`Main.cs:43446-43465`): full bubbles are 255/1.0, the partial one
    /// ramps grey `30 + 225*num4` and scale `num4/4 + 0.75`, both floored.
    /// Written NOT run (code-only mode).
    #[test]
    fn w4_c46_the_breath_bubble_visual_ramps_grey_and_scale() {
        // A full bubble (breath >= i*20).
        assert_eq!(breath_bubble_visual(200, 10, 20), (255, 1.0));
        assert_eq!(breath_bubble_visual(200, 1, 20), (255, 1.0));
        // The boundary: breath == i*20 is still full.
        assert_eq!(breath_bubble_visual(20, 1, 20), (255, 1.0));
        // A half bubble: num4 = 0.5 → grey (int)(30+112.5) = 142, scale 0.875.
        assert_eq!(breath_bubble_visual(10, 1, 20), (142, 0.875));
        // An empty bubble: num4 = 0 → grey 30, scale 0.75 (the floors).
        assert_eq!(breath_bubble_visual(0, 1, 20), (30, 0.75));
        // Below the floor (negative num4): both clamp.
        assert_eq!(breath_bubble_visual(0, 2, 20), (30, 0.75));
        // The lava flames share the helper with per = lavaMax/10 (420 → 42):
        // a half-drained budget's 5th flame is full, the 6th half.
        assert_eq!(breath_bubble_visual(210, 5, 42), (255, 1.0));
        assert_eq!(breath_bubble_visual(210, 6, 42), (142, 0.875));
    }

    /// W4-c46 — the breath meter's anchor walk (`Main.cs:43421-43438`): the
    /// closed-inventory `-100` lift, the open-inventory `< 1000` drop
    /// (`height - 20`), and the options/fancy-UI re-anchors. Written NOT run.
    #[test]
    fn w4_c46_the_breath_anchor_walks_the_inventory_and_options_arms() {
        let w = tr_core::World::new("breath".into(), 200, 200);
        let mut p = Player::spawn(&w);
        p.x = 100.0;
        p.y = 200.0;
        p.gfx_off_y = 3.0;
        let camera = Vec2::new(0.0, 0.0);
        let screen = Vec2::new(800.0, 600.0);
        // Closed inventory: Top + gfxOffY, zoomed, then -100.
        let a = breath_meter_anchor(&p, camera, 1.0, screen, false, false, false);
        assert_eq!(a, Vec2::new(100.0 + PLAYER_WIDTH * 0.5 + 400.0, 203.0 + 300.0 - 100.0));
        // Open inventory on a short screen: + (height - 20), no -100 lift.
        let a = breath_meter_anchor(&p, camera, 1.0, screen, true, false, false);
        assert_eq!(a.y, 203.0 + p.height - 20.0 + 300.0);
        // Open inventory on a TALL screen (>= 1000): the lift returns, no drop.
        let tall = Vec2::new(800.0, 1000.0);
        let a = breath_meter_anchor(&p, camera, 1.0, tall, true, false, false);
        assert_eq!(a.y, 203.0 + 500.0 - 100.0);
        // The options window re-anchors to the screen centre + 236.
        let a = breath_meter_anchor(&p, camera, 1.0, screen, false, true, false);
        assert_eq!(a, Vec2::new(400.0, 300.0 + 236.0));
        // A fancy-UI state (the bestiary) anchors to screenHeight - 64.
        let a = breath_meter_anchor(&p, camera, 1.0, screen, false, true, true);
        assert_eq!(a, Vec2::new(400.0, 600.0 - 64.0));
    }
}

