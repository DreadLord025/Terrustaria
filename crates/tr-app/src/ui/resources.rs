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
/// rows under `Main.expertMode`. The `ManaSickness` (94) arm appends
/// `(int)(manaSickReduction * 100f) + 1` percent (`:43282`) and `ManaHeat`
/// (396) appends `(int)(Math.Ceiling(manaHeat * 100f) - 100.0)` percent
/// (`:43285`) — both fields now exist on the port's Player (i19/i52), so
/// the S1-E-6 documented drops are closed. `DebugOptions.ManaPotionDelay`
/// (`:43278-43281`, `Testing/DebugOptions.cs` — the developer test toggle)
/// has no port counterpart and stays false, so the plain arm always runs.
pub(crate) fn buff_tooltip(p: &Player, buff: i32) -> String {
    let expert = p.buffs.expert_mode;
    let text = match buff {
        26 if expert => crate::lang::text("BuffDescription.WellFed_Expert"),
        206 if expert => crate::lang::text("BuffDescription.WellFed2_Expert"),
        207 if expert => crate::lang::text("BuffDescription.WellFed3_Expert"),
        _ => crate::lang::buff_description(buff),
    };
    match buff {
        // `:43282` — the C# string-concats the int then a literal "%"; the
        // truncating cast is `as i32`, the `+ 1` rides AFTER it.
        94 => format!("{text}{}%", (p.mana_sick_reduction * 100.0) as i32 + 1),
        // `:43285` — `manaHeat * 100f` multiplies in f32, `Math.Ceiling`
        // promotes the product to double, minus the double 100.0, then the
        // `(int)` truncation (the ceil of an integral double is exact).
        396 => format!(
            "{text}{}%",
            ((p.mana_heat * 100.0f32) as f64).ceil() as i32 - 100
        ),
        _ => text,
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

/// `Main.DrawBuffIcon`'s icon-text selection (`Main.cs:43316-43324`): the
/// `BuffTextHandlers` counter wins when the handler yields one, else the
/// duration when `TryGetBuffTime` passes and the tick count is over 2,
/// formatted at second resolution (`buffTimeValue / 60`). Split out of the
/// draw so the gates witness without a renderer.
pub(crate) fn buff_icon_text(p: &Player, slot: usize, vampire_seed: bool) -> Option<String> {
    buff_text(p, slot).or_else(|| {
        try_get_buff_time(p, slot, vampire_seed)
            .filter(|t| *t > 2)
            .map(|t| localized_duration(t / 60))
    })
}

/// `Main.DrawBuffIcon` (`Main.cs:43307-43368`) + the grid walk of
/// `DrawInterface_Resources_Buffs` (`:43203-43250`). The icons tint by the
/// per-slot alpha (`Main.buffAlpha` — the port's `buff_alpha`), the hover
/// raises it 0.1 and the rest decays 0.05 clamped to [0.4, 1]; the
/// right-click (`Main.mouseRight && Main.mouseRightRelease`, `:43333`)
/// pushes the slot onto `removals` (the C# removes in
/// the draw — the port drains the request in the tick). The icon text is
/// [`buff_icon_text`] (`:43316-43327`), drawn on `FontAssets.ItemStack` at
/// 0.8 under the icon in the same slot-alpha colour (`:43327`).
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
        // The icon (`:43311`): the full sheet tinted by the slot alpha. A
        // buff with no extracted sheet draws a grey 32×32 placeholder box
        // on the white UV (the crafting-slot convention, `ui/crafting.rs`
        // — documented in docs/ANALYSIS.md) so the slot still fades, texts,
        // hover-tooltips and right-click-cancels instead of silently
        // vanishing. The C# never misses (`TextureAssets.Buff[num]` loads
        // eagerly, `AssetInitializer.cs`); the port's atlas is enumeration-
        // only, hence the stand-in.
        let a = buff_alpha[i];
        let rect = lookups.buff_icons.get(&(bt as u16));
        let (w, h, tint) = match rect {
            Some(rect) => (
                rect.width as f32,
                rect.height as f32,
                [a, a, a, a],
            ),
            // The grey box at the crafting convention's 0.5, still riding
            // the slot alpha so the fade/hover reads the same.
            None => (32.0, 32.0, [0.5 * a, 0.5 * a, 0.5 * a, 0.5 * a]),
        };
        let uv = lookups.hud_uv(rect).unwrap_or_else(|| r.white_uv());
        r.push_ui_sprite([x, y], [w, h], uv, tint);
        // The icon text (`:43316-43327`): `buff_icon_text` picks the counter
        // or the duration — `FontAssets.ItemStack` at 0.8 under the icon,
        // tinted by the SAME slot-alpha colour as the icon (`:43314`'s
        // `color`, reused at `:43327`). The S1-E-6 pass drew it white at
        // 0.85 on the Mouse_Text font — three verbatim deltas, fixed here.
        if let Some(text) = buff_icon_text(player, i, vampire_seed) {
            crate::ui::text::hud_item_text(r, lookups, x, y + h, 0.8, [a, a, a, a], &text);
        }
        // The hover + the right-click removal (`:43329-43349`). The C#'s
        // `!PlayerInput.IgnoreMouseInterface` gate has no port counterpart.
        // `ui::in_rect_strict` is the same open-on-every-edge comparison
        // (`:43329`'s strict `<`/`>` on all four sides) — the shared seam,
        // not a private copy.
        if crate::ui::in_rect_strict(mouse, [x, y, w, h]) {
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
        // The `ManaHeat` (396) arm now appends the player-computed percent
        // (`Main.cs:43285`): the default `manaHeat == 1.0` ceils to exactly
        // 100, so the appended text is "0%" — not a bare description.
        assert_eq!(
            buff_tooltip(&p, 396),
            format!("{}0%", crate::lang::buff_description(396))
        );
    }

    /// W4-c26 — `Main.GetBuffTooltip`'s two player-computed percent arms
    /// (`Main.cs:43282`/`:43285`), the S1-E-6 drops closed now that i19/i52
    /// landed `Player.manaSickReduction`/`manaHeat`.
    #[test]
    fn w4_c26_the_mana_percent_arms_append_the_player_computed_text() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{
                "BuffDescription": {
                    "ManaSickness": "Cannot use magic items ",
                    "ManaHeat": "Magic damage reduced by "
                }
            }"#,
        );
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        // `:43282` — `(int)(manaSickReduction * 100f) + 1`, the +1 AFTER
        // the truncating cast: 0.25 → 25 + 1 = 26.
        p.mana_sick_reduction = 0.25;
        assert_eq!(buff_tooltip(&p, 94), "Cannot use magic items 26%");
        // The truncation: 0.259 → 25 (not 26) + 1 = 26.
        p.mana_sick_reduction = 0.259;
        assert_eq!(buff_tooltip(&p, 94), "Cannot use magic items 26%");
        // Zero reduction still appends 1% (the vanilla's floor).
        p.mana_sick_reduction = 0.0;
        assert_eq!(buff_tooltip(&p, 94), "Cannot use magic items 1%");
        // `:43285` — `(int)(Math.Ceiling(manaHeat * 100f) - 100.0)`:
        // 1.123 → ceil(112.3…) = 113 − 100 = 13.
        p.mana_heat = 1.123;
        assert_eq!(buff_tooltip(&p, 396), "Magic damage reduced by 13%");
        // Exactly 1.0 ceils to 100 → 0%.
        p.mana_heat = 1.0;
        assert_eq!(buff_tooltip(&p, 396), "Magic damage reduced by 0%");
    }

    /// W4-c26 — `Main.DrawBuffIcon`'s icon-text selection
    /// (`Main.cs:43316-43324`): the counter wins, else the over-2-tick
    /// duration at second resolution (`buffTimeValue / 60`), else nothing.
    #[test]
    fn w4_c26_the_icon_text_prefers_the_counter_then_the_duration() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Game.json",
            r#"{ "Misc": { "ShortMinutes": "m", "ShortSeconds": "s" } }"#,
        );
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        // The handler buff (64 → projectile 266) with a live counter wins
        // (`:43317-43320` before `:43321`) — 64 is also in the
        // no-time-display set, so the counter is its ONLY text.
        p.buffs.buff_type[0] = 64;
        p.buffs.buff_time[0] = 6000;
        p.owned_projectile_counts[266] = 2;
        assert_eq!(buff_icon_text(&p, 0, false).as_deref(), Some("x2"));
        // Counter back to zero: 64's duration stays hidden (the
        // `buffNoTimeDisplay` gate, `:43295`).
        p.owned_projectile_counts[266] = 0;
        assert_eq!(buff_icon_text(&p, 0, false), None);
        // A timed buff (1, ObsidianSkin) falls to the duration: 6000 ticks
        // / 60 = 100 s → "2 m" (the ceiling bump, `Lang.cs:1183-1189`).
        p.buffs.buff_type[0] = 1;
        assert_eq!(buff_icon_text(&p, 0, false).as_deref(), Some("2 m"));
        // 2 ticks or fewer: no text (`:43321`'s `> 2`).
        p.buffs.buff_time[0] = 2;
        assert_eq!(buff_icon_text(&p, 0, false), None);
        // 3 ticks: 3/60 = 0 s → "0 s" — the vanilla draws the zero.
        p.buffs.buff_time[0] = 3;
        assert_eq!(buff_icon_text(&p, 0, false).as_deref(), Some("0 s"));
        // The TryGetBuffTime gates hide it: OnFire (24) sub-10 under the
        // vampire seed (`:43297-43300`).
        p.buffs.buff_type[0] = 24;
        p.buffs.buff_time[0] = 9;
        assert_eq!(buff_icon_text(&p, 0, true), None);
        assert!(buff_icon_text(&p, 0, false).is_some(), "no seed, no hide");
    }

    /// W4-c26 — the hover block's name read (`Main.cs:43237`):
    /// `Lang.GetBuffName(num6)` is the `BuffName.{const}` row, direct — the
    /// c21 accessor lane's `lang::buff_name`, no intermediate table.
    #[test]
    fn w4_c26_the_hover_name_reads_the_buff_name_row_directly() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{ "BuffName": { "ObsidianSkin": "Obsidian Skin" } }"#,
        );
        assert_eq!(crate::lang::buff_name(1), "Obsidian Skin");
        // An id with no const answers empty (`Lang.cs:241-244`'s cache miss).
        assert_eq!(crate::lang::buff_name(99999), "");
    }

    /// W4-c26 — the cancel hit-test (`Main.cs:43329`) rides the shared
    /// `ui::in_rect_strict` seam: open on EVERY edge, at the grid's slot-0
    /// rect. The gate conjunction itself is witnessed at
    /// `app/frame/draw.rs::s1_p_i53_the_cancel_gate_fires_on_press_not_on_release`;
    /// this pins the rect half the draw feeds it.
    #[test]
    fn w4_c26_the_cancel_hit_test_is_strict_on_every_edge() {
        let [x, y] = buff_slot_pos(0);
        let rect = [x, y, 32.0, 32.0];
        let hit = |mx: f32, my: f32| crate::ui::in_rect_strict(Vec2::new(mx, my), rect);
        assert!(hit(x + 1.0, y + 1.0), "inside");
        assert!(!hit(x, y + 1.0), "the left edge is open (`mouseX > x`)");
        assert!(!hit(x + 1.0, y), "the top edge is open (`mouseY > y`)");
        assert!(!hit(x + 32.0, y + 1.0), "the right edge is open (`<`)");
        assert!(!hit(x + 1.0, y + 32.0), "the bottom edge is open (`<`)");
        // The packed grid: slot 1 sits at +38, so the 6-px gap between the
        // 32-wide icons never hover-cancels either slot.
        assert!(!hit(x + 34.0, y + 1.0), "the inter-icon gap");
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
}

