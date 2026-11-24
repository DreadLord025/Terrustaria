//! Inventory and hotbar slots: a faithful port of `ItemSlot.Draw`
//! (`decompiled/Terraria/UI/ItemSlot.cs:2105-2776`).
//!
//! Covered contexts (the ones the P2 inventory panels use): 0 (main grid/hotbar),
//! 1 (coins), 2 (ammo), 3 (the world-chest panel, `Inventory_Back5`), 5 (reforge),
//! 6 (trash), 7 (guide), 8-12 (armour/vanity/dye), 13 (the closed hotbar,
//! `GUIHotbarDrawInner`), 21 (the mouse-carried item, no background —
//! `Main.cs:45194`), 16-20 (misc equips) and 33 (misc dyes).

use tr_render::{SheetRect, SpriteRenderer};

use super::coin_slot::{draw_item_coin, CoinDrawState, CoinSlotState};
use super::text::hud_item_text;
use crate::assets;
use crate::item::ItemSlot;

/// `ItemSlot.cs:2148-2304`: which `Inventory_Back*` sheet a slot uses.
///
/// The numbers are indices into this port's sheet set: `0` `Inventory_Back`,
/// `1` `Inventory_Back9`, `2` `Inventory_Back4`, `3` `Inventory_Back7`,
/// `4` `Inventory_Back13`, `5` `Inventory_Back3`, `6` `Inventory_Back12`,
/// `7` `Inventory_Back18`, `8` `Inventory_Back5`, `9` `Inventory_Back10`,
/// `10` `Inventory_Back6`.
/// Tests assert on this because `SheetRect` has no `PartialEq`.
fn context_texture_index(context: i32, index: i32) -> u8 {
    match context {
        // `ItemSlot.cs:2172-2175`: context 0, slot < 10 is `Inventory_Back9`.
        0 if index < 10 => 1,
        // `ItemSlot.cs:2148-2149`: context 0 slot >= 10, coins (1) and ammo (2).
        0 | 1 | 2 => 0,
        // `ItemSlot.cs:2215-2218`: the world-chest slots (3) are
        // `Inventory_Back5`.
        3 => 8,
        // W4-g56 — `ItemSlot.cs:2218-2221`: the bank (4) AND void-vault (32)
        // slots are `Inventory_Back2`.
        4 | 32 => 11,
        // `ItemSlot.cs:2222-2225`: reforge (5) and guide (7) are `Inventory_Back4`.
        5 | 7 => 2,
        // `ItemSlot.cs:2226-2228`: trash (6) is `Inventory_Back7`.
        6 => 3,
        // `ItemSlot.cs:2189-2213`: armour (8/10), vanity (9/11) and dye (12) are
        // `Inventory_Back13`.
        8..=12 => 4,
        // `ItemSlot.cs:2181-2184`: misc equips (16-20) are `Inventory_Back3`.
        16..=20 => 5,
        // `ItemSlot.cs:2206-2209`: misc dyes (33) are `Inventory_Back12`.
        33 => 6,
        // `ItemSlot.cs:2264-2277` / `:2298-2302`: crafting contexts 41/42/43 end up
        // on `Inventory_Back18` (42/43 start on `_4` and are then overridden).
        41 | 42 | 43 => 7,
        // `ItemSlot.Draw`'s context-15 arm (`ItemSlot.cs:2250-2252`): the
        // shop cells ride `Inventory_Back6`.
        15 => 10,
        // Contexts outside the ported set keep the default `Inventory_Back`.
        _ => 0,
    }
}

/// `ItemSlot.Draw`: the background sheet for a slot (`ItemSlot.cs:2148-2304`).
pub(crate) fn background_for(
    lookups: &assets::Lookups,
    context: i32,
    index: i32,
) -> Option<SheetRect> {
    match context_texture_index(context, index) {
        1 => lookups.ui_back_hotbar, // Inventory_Back9
        2 => lookups.ui_back4,       // Inventory_Back4
        3 => lookups.ui_back7,       // Inventory_Back7
        4 => lookups.ui_inv_bg13,    // Inventory_Back13
        5 => lookups.ui_back3,       // Inventory_Back3
        6 => lookups.ui_back12,      // Inventory_Back12
        7 => lookups.ui_back18,      // Inventory_Back18
        8 => lookups.ui_back5,       // Inventory_Back5
        9 => lookups.ui_back10,      // Inventory_Back10
        10 => lookups.ui_back6,      // Inventory_Back6
        11 => lookups.ui_back2,      // Inventory_Back2 (banks/vault, W4-g56)
        _ => lookups.ui_back,        // Inventory_Back
    }
}

/// `ItemSlot.cs:2270-2271` / `:2299-2300`: the `Inventory_Back18` tint for the
/// crafting-recipe slots (contexts 41/42/43) =
/// `ShiftBlueToCyanTheme(new Color(16, 36, 56, 180) * 0.9f)`.
///
/// `ItemSlot.cs:2340-2343` then multiplies it by `lightColor`; the crafting panel
/// passes `Color.White` (`NewCraftingUI.cs:563`), so this is applied as-is.
pub fn craft_slot_tint() -> [f32; 4] {
    super::color::tint(super::color::shift_blue_to_cyan(super::color::mul(
        [16, 36, 56, 180],
        0.9,
    )))
}

/// `ItemSlot.cs:2149-2156`'s favorited-background arm gate:
/// `!item.IsAir && item.favorited && ItemSlot.retainFavoriteOnMoveAt[context]`
/// (the contexts whose moves keep the flag — `chest::retain_favorite_on_move_at`).
/// Non-air is `count > 0` (the port's implicit-air shape).
pub(crate) fn favorited_background_active(context: i32, favorited: bool, item: &ItemSlot) -> bool {
    favorited && item.count > 0 && crate::chest::retain_favorite_on_move_at(context)
}

/// `ItemSlot.cs:2154-2158`: the sheet the favorited arm swaps in — the gold
/// `Inventory_Back10`, or `Inventory_Back19` for context 32 (the void
/// vault's purple frame, W4-g56). The tint stays `Main.inventoryBack`;
/// `Main.OurFavoriteColor` (`Main.cs:66748`) colours the tooltip name, not
/// the frame.
pub(crate) fn favorited_background_for(
    lookups: &assets::Lookups,
    context: i32,
) -> Option<SheetRect> {
    if context == 32 {
        // `:2155-2158` — the vault favorite frame; the packed sheet may be
        // absent (a content-less build), the gold frame stands in then.
        lookups.ui_back19.or(lookups.ui_back10)
    } else {
        lookups.ui_back10
    }
}

/// `ItemSlot.cs:2403-2470`: the `Extra_54` frame index for an empty equipment
/// slot (`num15`), or `-1` when the context has no glyph.
fn equip_glyph_index(context: i32, index: i32) -> i32 {
    match context {
        // `case 8:` slot 0/1/2 -> 0/6/12.
        8 => match index {
            0 => 0,
            1 => 6,
            2 => 12,
            _ => -1,
        },
        // `case 9:` slot 10/11/12 -> 3/9/15.
        9 => match index {
            10 => 3,
            11 => 9,
            12 => 15,
            _ => -1,
        },
        // `case 10, 24:` -> 11.
        10 => 11,
        // `case 11:` -> 2.
        11 => 2,
        // `case 12, 25, 27, 33:` -> 1.
        12 | 33 => 1,
        // `case 16:` -> 4.
        16 => 4,
        // `case 17, 39:` -> 13.
        17 => 13,
        // `case 18:` -> 7.
        18 => 7,
        // `case 19:` -> 10.
        19 => 10,
        // `case 20:` -> 17.
        20 => 17,
        // `case 7:` -> 18.
        7 => 18,
        _ => -1,
    }
}

/// `ItemSlot.cs:2494-2496`: the source sub-rect of the `Extra_54` sheet for glyph
/// `num15` — `Frame(3, 7, num15 % 3, num15 / 3)` with `Width`/`Height` shrunk by
/// 2 px. Returns `(offset_x, offset_y, width, height)` inside the sheet.
fn glyph_frame(sheet_w: u32, sheet_h: u32, num15: i32) -> Option<(u32, u32, u32, u32)> {
    if num15 < 0 || sheet_w < 3 || sheet_h < 7 {
        return None;
    }
    let fw = sheet_w / 3;
    let fh = sheet_h / 7;
    if fw < 3 || fh < 3 {
        return None;
    }
    Some((
        (num15 % 3) as u32 * fw,
        (num15 / 3) as u32 * fh,
        fw - 2,
        fh - 2,
    ))
}

/// `ItemSlot.cs:2755-2760`: the hotbar label is `slot + 1`, except slot 10 which
/// is drawn as `"0"`.
fn hotbar_digit(index: i32) -> String {
    let text = format!("{}", index + 1);
    if text == "10" {
        "0".to_string()
    } else {
        text
    }
}

/// `ItemSlot.cs:2762-2770`: the hotbar digit sits at
/// `position + (6, 4 + num29) * inventoryScale` with `num29 = selected ? -2 : 0`
/// and origin zero.
fn hotbar_digit_offset(x: f32, y: f32, selected: bool, scale: f32) -> (f32, f32) {
    let num29 = if selected { -2.0f32 } else { 0.0 };
    (x + 6.0 * scale, y + (4.0 + num29) * scale)
}

/// `ItemSlot.cs:2693-2701`: the closed-hotbar (context 13) digit sits at
/// `position + (8, 4) * inventoryScale` — unlike the context-0 row it never
/// lifts 2 px and is tinted by the caller's `lightColor`, not `Main.inventoryBack`.
fn closed_hotbar_digit_offset(x: f32, y: f32, scale: f32) -> (f32, f32) {
    (x + 8.0 * scale, y + 4.0 * scale)
}

/// `GUIHotbarDrawInner` (`Main.cs:43615`): the `lightColor` every context-13
/// element is tinted by — `new Color(255, 255, 255, (int)(75f + 150f * scale))`,
/// so the item, its stack text and the digit fade with the slot's animation scale.
fn closed_hotbar_light_color(scale: f32) -> [f32; 4] {
    [1.0, 1.0, 1.0, (75.0 + 150.0 * scale) / 255.0]
}

/// `GUIHotbarDrawInner` (`Main.cs:43614,43639`): the closed hotbar's per-slot
/// metrics at `scale` — the draw row `y` `(int)(20 + 22 * (1 - scale))`, the x
/// advance `(int)(InventoryBack.Width() * scale) + 4` (the retail sheet is 52 px
/// wide) and the untruncated `Width * scale` hover size the C# hit test uses
/// (`Main.cs:43616`). Returns `(y, x_advance, hover_size)`.
pub(crate) fn closed_hotbar_metrics(scale: f32) -> (f32, f32, f32) {
    let y = (20.0 + 22.0 * (1.0 - scale)) as i32 as f32;
    let advance = (52.0 * scale) as i32 as f32 + 4.0;
    (y, advance, 52.0 * scale)
}

/// `ItemSlot.cs:2232-2242`: the closed-hotbar (context 13) background — the
/// selected slot is the yellow `Inventory_Back14` at full white and every other
/// slot stays on the default `Inventory_Back` at `new Color(200, 200, 200, 200)`.
/// (`Inventory_Back20`, the white variant for
/// `slot == player.selectedItemState.Hotbar`, is only reachable while a
/// `SelectedItemState` override is active (`ItemSlot.cs:2234-2237`); the port has
/// no override system, so `Hotbar` is always the selected slot and that branch
/// merges into the `flag` one.)
pub(crate) fn closed_hotbar_background(
    lookups: &assets::Lookups,
    selected: bool,
) -> (Option<SheetRect>, [f32; 4]) {
    if selected {
        (lookups.ui_back_sel, [1.0; 4])
    } else {
        (lookups.ui_back, [200.0 / 255.0; 4])
    }
}

/// `ItemSlot.cs:2699-2706`: the stack count's net top-left. The C# draws at
/// `position + (10, 26 + LineSpacing) * inventoryScale` with origin
/// `(0, LineSpacing)` and scale `num23 = inventoryScale * stackTextScale`;
/// `DrawString` scales the origin (`DynamicSpriteFont.cs:194`), so the net y
/// is `y + (26 + ls) * scale - ls * num23`. At rest (`stackTextScale = 1`)
/// the ls terms cancel and the count sits at `(10, 26) * scale`; during the
/// coin-jump text pulse the count lifts. Context 43 shifts `(-5, +7)` and
/// scales `num23` by 1.2 (`:2701-2704`).
fn stack_number_pos(
    x: f32,
    y: f32,
    scale: f32,
    line_spacing: f32,
    text_scale: f32,
    context: i32,
) -> (f32, f32) {
    let (mut ox, mut oy) = (10.0, 26.0 + line_spacing);
    let mut num23 = scale * text_scale;
    if context == 43 {
        ox += -5.0;
        oy += 7.0;
        num23 *= 1.2;
    }
    (x + ox * scale, y + oy * scale - line_spacing * num23)
}

/// `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333-353`): four
/// copies offset by `spread` along `±x`/`±y` (`ShadowDirections`,
/// `ChatManager.cs:376-382`), then the coloured text on top. The shadow colour is
/// `baseColor.MultiplyRGBA(Color.Black)` — black with the base colour's own alpha
/// (`ChatManager.cs:334`), so a slot faded by `lightColor` fades its shadow too.
fn hud_item_text_shadowed(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    spread: f32,
    text: &str,
) {
    let shadow = [0.0, 0.0, 0.0, color[3]];
    for (dx, dy) in [(-1.0f32, 0.0f32), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        hud_item_text(
            r,
            lookups,
            x + dx * spread,
            y + dy * spread,
            scale,
            shadow,
            text,
        );
    }
    hud_item_text(r, lookups, x, y, scale, color, text);
}

/// `ItemSlot.DrawItemIcon` (`ItemSlot.cs:2848-2881`) with `sizeLimit = 32`
/// (`ItemSlot.cs:2515`): the item's own texture centred on `center`, its longest
/// side clamped to 32 px before the inventory scale
/// (`DrawItem_GetColorAndScale`, `ItemSlot.cs:2966-2975`), tinted by
/// `item.GetAlpha(environmentColor)` (`ItemSlot.cs:2876`).
fn draw_item_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    center: (f32, f32),
    scale: f32,
    tint: [f32; 4],
    item_fade: f32,
) {
    let (uv, iw, ih) = match lookups.item_icon(item.kind) {
        Some(v) => v,
        None => match lookups.plain_tex(item.kind) {
            Some(t) => (t.uv, 16.0, 16.0),
            None => return,
        },
    };
    let longest = iw.max(ih);
    let limit = if longest > 32.0 { 32.0 / longest } else { 1.0 };
    let s = scale * limit;
    let (w, h) = (iw * s, ih * s);
    let c = [tint[0] * item_fade, tint[1] * item_fade, tint[2] * item_fade, tint[3] * item_fade];
    r.push_ui_sprite([center.0 - w * 0.5, center.1 - h * 0.5], [w, h], uv, c);
}

/// `ItemSlot.cs:2891-2929` (W4-g35): the item's glowMask overlay, drawn at
/// the icon's own centre/scale with `origin = rectangle.Size() / 2`. The
/// rect is the icon frame (`Main.itemAnimations` is unported — the whole
/// sheet, the `draw_item_icon` stand-in), EXCEPT the Monk Staff T3 3858's
/// `Glow_233.Frame(1, 3, 0, miscCounter % 15 / 5)` with `Height -= 2`
/// (`:2913-2914`). The 5670/5671 pair draws TWICE (`:2924-2928`): the phase
/// colour, then the `drawColor: true` copy (A 127). `itemFade`/context-37
/// `num` stay 1 (no port fade/overdraw system). Returns the quad count.
fn draw_item_glow(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    center: (f32, f32),
    scale: f32,
    pulse: super::UiPulse,
) -> u32 {
    if !item_glow_active(item.kind) {
        return 0;
    }
    let def = crate::registry::get(item.kind).expect("the gate read the row");
    // The icon's own frame + clamp (`draw_item_icon`'s resolution).
    let (iw, ih) = match lookups.item_icon(item.kind) {
        Some((_, w, h)) => (w, h),
        None => match lookups.plain_tex(item.kind) {
            Some(_) => (16.0, 16.0),
            None => return 0,
        },
    };
    let longest = iw.max(ih);
    let limit = if longest > 32.0 { 32.0 / longest } else { 1.0 };
    let s = scale * limit;
    let name = format!("Glow_{}", def.glow_mask);
    let shoot = def.projectile.unwrap_or(0);
    let tint = item_glow_tint(item.kind, shoot, pulse);
    let f = |c: [u8; 4]| {
        [
            c[0] as f32 / 255.0,
            c[1] as f32 / 255.0,
            c[2] as f32 / 255.0,
            c[3] as f32 / 255.0,
        ]
    };
    let mut quads = 0;
    // The 3858 rect override: the Glow_233 3-row strip, the miscCounter row.
    if item.kind == 3858 {
        let Some(rect) = lookups.glow_sheets.get("Glow_233") else {
            return 0;
        };
        let row_h = rect.height / 3;
        let row = (pulse.misc_counter % 15 / 5) as i32;
        let h = row_h.saturating_sub(2);
        let Some(uv) = lookups.glow_uv("Glow_233", 0, row * row_h as i32, rect.width, h) else {
            return 0;
        };
        let (w, h) = (rect.width as f32 * s, h as f32 * s);
        r.push_ui_sprite([center.0 - w * 0.5, center.1 - h * 0.5], [w, h], uv, f(tint));
        return 1;
    }
    let Some(uv) = lookups.glow_uv(&name, 0, 0, iw as u32, ih as u32) else {
        return 0;
    };
    let (w, h) = (iw * s, ih * s);
    let tl = [center.0 - w * 0.5, center.1 - h * 0.5];
    if matches!(item.kind, 5670 | 5671) {
        // `:2926`: the first copy at the phase colour, then the A-127 copy.
        r.push_ui_sprite(tl, [w, h], uv, f(tint));
        let second = phase_color(shoot, pulse.misc_counter, true);
        r.push_ui_sprite(tl, [w, h], uv, f(second));
        quads += 2;
    } else {
        r.push_ui_sprite(tl, [w, h], uv, f(tint));
        quads += 1;
    }
    quads
}

/// `ItemSlot.cs:2891`'s gate (W4-g35): the glow overlay draws for every
/// `item.glowMask != -1` EXCEPT the Spirit Flame 3779, the Light's Bane 46
/// and the Dead Cells Flint 5462 (their slashes draw their own glows).
pub(crate) fn item_glow_active(kind: u16) -> bool {
    if matches!(kind, 3779 | 46 | 5462) {
        return false;
    }
    crate::registry::get(kind).is_some_and(|d| d.glow_mask != -1)
}

/// `Item.GetPhaseColorDirect` (`Item.cs:49111-49168`): the nine-entry
/// `_phaseColors` ramp (`:49057-49068` — 1059-1064, 1071, 1075, 1059 again).
/// `Color(float, float, float)` scales by 255 and clamps to the byte range
/// (the 1.2/1.05 channels saturate at 255), A = 255.
const PHASE_COLORS: [[u8; 4]; 9] = [
    [25, 127, 255, 255],  // 1059: (0.1, 0.5, 1.2)
    [255, 51, 25, 255],   // 1060: (1, 0.2, 0.1)
    [25, 255, 51, 255],   // 1061: (0.1, 1, 0.2)
    [204, 25, 255, 255],  // 1062: (0.8, 0.1, 1)
    [204, 229, 255, 255], // 1063: (0.8, 0.9, 1)
    [204, 204, 0, 255],   // 1064: (0.8, 0.8, 0)
    [229, 127, 0, 255],   // 1071: (0.9, 0.5, 0)
    [255, 25, 102, 255],  // 1075: (1.05, 0.1, 0.4)
    [25, 127, 255, 255],  // 1059 again
];

/// `Utils.MultiLerp` (`Utils.cs:292-303`) over [`PHASE_COLORS`]: the segment
/// walk stops at index 7, so a `percent > 1` EXTRAPOLATES past the last pair
/// (XNA's `Color.Lerp` does not clamp the amount; the byte write truncates
/// like the C# `(byte)` cast — `miscCounterNormalized = miscCounter / 300f`
/// grows past 1, `Player.cs:3389`).
fn phase_multi_lerp(percent: f32) -> [u8; 4] {
    let num = 1.0 / (PHASE_COLORS.len() as f32 - 1.0);
    let mut num2 = num;
    let mut num3 = 0usize;
    while percent / num2 > 1.0 && num3 < PHASE_COLORS.len() - 2 {
        num2 += num;
        num3 += 1;
    }
    let amount = (percent - num * num3 as f32) / num;
    let (a, b) = (PHASE_COLORS[num3], PHASE_COLORS[num3 + 1]);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * amount) as i32 as u8;
    [l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2]), l(a[3], b[3])]
}

/// `Item.GetPhaseColor` (`Item.cs:49053-49109`): the rainbow phaseblade /
/// phasersaber pair (1101/1102 — the item's `shoot`) reads the MultiLerp;
/// `drawColor: true` forces A = 127 (`:49102`). Every other id keeps White.
pub(crate) fn phase_color(shoot: u16, misc_counter: u32, draw_color: bool) -> [u8; 4] {
    if !matches!(shoot, 1101 | 1102) {
        return [255, 255, 255, 255];
    }
    let mut c = phase_multi_lerp(misc_counter as f32 / 300.0);
    if draw_color {
        c[3] = 127;
    }
    c
}

/// `Utils.WrappedLerp` (`Utils.cs:305-313`): the percent folds at 0.5.
fn wrapped_lerp(v1: f32, v2: f32, percent: f32) -> f32 {
    let mut num = percent * 2.0;
    if num > 1.0 {
        num = 2.0 - num;
    }
    v1 + (v2 - v1) * num
}

/// `ItemSlot.GetTrustyPalGlowColor` (`ItemSlot.cs:2954-2959`): the pal
/// accessories' (6148-6151/6174) hue ramp — `hslToRgb((0.04 + 0.07·num) % 1,
/// 1, 0.5, 127) * 0.5 * num2`, `num = WrappedLerp(0, 1, (t/3) % 1)`,
/// `num2 = Remap(num, 0.2, 1, 0, 1)` clamped. The two float scales ride the
/// XNA per-byte truncation (`xna_scale_color`'s convention).
pub(crate) fn trusty_pal_glow_color(global_time: f32) -> [u8; 4] {
    let num = wrapped_lerp(0.0, 1.0, (global_time / 3.0) % 1.0);
    let num2 = ((num - 0.2) / (1.0 - 0.2)).clamp(0.0, 1.0);
    let rgb = tr_core::tile_light::hsl_to_rgb((0.04 + 0.07 * num) % 1.0, 1.0, 0.5);
    let scale = 0.5 * num2;
    let s = |x: f32| (x * 255.0 * scale) as i32 as u8;
    [s(rgb[0]), s(rgb[1]), s(rgb[2]), (127.0 * scale) as i32 as u8]
}

/// `ItemSlot.cs:2894-2923`'s per-type glow tint (W4-g35). `shoot` is the
/// item's `Item.shoot` (the 5670/5671 phase pair reads it); the 5462 arm is
/// dead code in the C# (the `:2891` gate excludes 5462) — kept verbatim.
/// The 5669 arm's `color.A = item.alpha` reads the slot's fade — the port's
/// `ItemSlot` carries no alpha field (stock 0, `Item.cs:49006`), recorded.
pub(crate) fn item_glow_tint(
    kind: u16,
    shoot: u16,
    pulse: super::UiPulse,
) -> [u8; 4] {
    match kind {
        5670 | 5671 => phase_color(shoot, pulse.misc_counter, false),
        5146 => [pulse.disco[0], pulse.disco[1], pulse.disco[2], 255],
        5462 => [255, 140, 0, 5],
        5669 => {
            // `Color.Lerp(White, (180,85,30), WrappedLerp(0.5, 1,
            // (miscCounter % 100) / 100))`, A = item.alpha (stock 0).
            let amount = wrapped_lerp(
                0.5,
                1.0,
                (pulse.misc_counter % 100) as f32 / 100.0,
            );
            let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * amount) as u8;
            [l(255, 180), l(255, 85), l(255, 30), 0]
        }
        // `(255,255,255,63) * 0.75` — the per-byte truncating scale.
        3858 => [191, 191, 191, 47],
        6148 | 6149 | 6150 | 6151 | 6174 => trusty_pal_glow_color(pulse.global_time),
        _ => [255, 255, 255, 255],
    }
}


/// `ItemSlot.cs:2490-2498`: the empty-equipment glyph — `Extra_54`,
/// `Frame(3, 7, num15 % 3, num15 / 3)`, shrunk 2 px, drawn centred on
/// `position + value.Size() / 2 * inventoryScale` in `White * 0.35`.
fn draw_equip_glyph(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    context: i32,
    index: i32,
    center: (f32, f32),
    scale: f32,
) {
    let num15 = equip_glyph_index(context, index);
    if num15 < 0 {
        return;
    }
    let Some(rect) = lookups.ui_equip_glyph else {
        return;
    };
    let Some((ox, oy, w, h)) = glyph_frame(rect.width, rect.height, num15) else {
        return;
    };
    let u0 = rect.x as f32 + ox as f32;
    let v0 = rect.y as f32 + oy as f32;
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let uv = [
        u0 / aw,
        v0 / ah,
        (u0 + w as f32) / aw,
        (v0 + h as f32) / ah,
    ];
    let sw = w as f32 * scale;
    let sh = h as f32 * scale;
    r.push_ui_sprite(
        [center.0 - sw * 0.5, center.1 - sh * 0.5],
        [sw, sh],
        uv,
        [0.35; 4], // `Color.White * 0.35f` (`ItemSlot.cs:2492,2497`)
    );
}

/// `ItemSlot.Draw`: one inventory/equipment slot — background, empty glyph or the
/// item icon, the stack count and (for the hotbar) the index digit. `tint` is the
/// C# `color2` (`Main.inventoryBack` and its per-context overrides). The C#
/// `lightColor` parameter is derived here: every caller omits it (`Color.White`,
/// `ItemSlot.cs:2109-2112`) except `GUIHotbarDrawInner`, whose context-13
/// `lightColor` is `Color(255, 255, 255, (int)(75 + 150 * inventoryScale))`
/// (`Main.cs:43615,43637`) — a pure function of the `inventoryScale` this port
/// already receives, with no other context-13 call site (`Main.cs:43648` is the
/// override slot, not ported). The item, its stack text and the closed-hotbar
/// digit are tinted by it.
///
/// `coin_slot` is the `CoinSlot` machine (`ui/coin_slot.rs`): the C# calls
/// `CoinSlot.UpdateDrawState(slot, context, item)` on EVERY slot draw
/// (`ItemSlot.cs:2349`), so the port threads `Some(&mut state)` plus the
/// cell's VANILLA slot index (`vanilla_slot` — the coin/ammo rows draw at
/// `Player.inventory[50..57]`, `Main.cs:41757/:41775`, while the glyph keeps
/// the display index). Contexts outside 0/1/2/3/4 get the neutral state and
/// may pass `None` (the machine is untouched). `mouse_air` is
/// `Main.mouseItem.IsAir` for the mouse-item dim (`GetDimSlotForMouseItem`,
/// `ItemSlot.cs:1361`); the port dims only the dye contexts (12/33) whose
/// gate needs no equip-validation system (the rest stay 1 — a recorded
/// deviation, the dim needs `CanEquipInArmorSlot`/mount/pet tables).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_slot(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    x: f32,
    y: f32,
    context: i32,
    index: i32,
    selected: bool,
    scale: f32,
    tint: [f32; 4],
    pulse: super::UiPulse,
    coin_slot: Option<&mut CoinSlotState>,
    vanilla_slot: i32,
    mouse_air: bool,
    mouse_is_dye: bool,
) {
    draw_slot_flagged(
        r, lookups, item, x, y, context, index, selected, scale, tint, false,
        pulse, None, coin_slot, vanilla_slot, mouse_air, mouse_is_dye,
    );
}

/// [`draw_slot`] with the favorite flag the C# reads from the item
/// (`item.favorited`): the port keeps favorites in a side set (`App`'s
/// `favorites`), so the caller passes the bit in.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_slot_favorited(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    x: f32,
    y: f32,
    context: i32,
    index: i32,
    selected: bool,
    scale: f32,
    tint: [f32; 4],
    pulse: super::UiPulse,
    coin_slot: Option<&mut CoinSlotState>,
    vanilla_slot: i32,
    mouse_air: bool,
    mouse_is_dye: bool,
) {
    draw_slot_flagged(
        r, lookups, item, x, y, context, index, selected, scale, tint, true,
        pulse, None, coin_slot, vanilla_slot, mouse_air, mouse_is_dye,
    );
}

/// W4-g56 — [`draw_slot_favorited`] with the chest-glow entries
/// (`ItemSlot.Draw:2314-2326`): a glowing (`time > 0`) non-air non-favorited
/// slot in context 3/4/32 swaps to `Inventory_Back13` and lerps the base tint
/// toward `hslToRgb(hue, 1, 0.5)` by `t²/2`. `glow` is `Some((time, hue))` for
/// the open-container slots (the caller reads `App.slot_glow`'s chest pair);
/// `None` keeps the plain background. The favorite arm still wins (the C#
/// glow gate reads `!inv[slot].favorited`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_slot_glow(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    x: f32,
    y: f32,
    context: i32,
    index: i32,
    selected: bool,
    scale: f32,
    tint: [f32; 4],
    pulse: super::UiPulse,
    coin_slot: Option<&mut CoinSlotState>,
    vanilla_slot: i32,
    mouse_air: bool,
    mouse_is_dye: bool,
    favorite: bool,
    glow: Option<(i32, f32)>,
) {


/// The neutral `CoinDrawState` for contexts outside the machine's arrays
/// (`CoinSlot.UpdateDrawState`'s tail, `CoinSlot.cs:306-311`) — used when a
/// caller passes no machine (`None`).
fn neutral_draw_state() -> CoinDrawState {
    CoinDrawState {
        coin_anim_frame: 0,
        coin_y_offset: 0.0,
        stack_text_scale: 1.0,
        stack_text_draw_fade_overload: -1.0,
        fade_item: 0,
        fade_scale: 1.0,
    }
}

/// `ItemSlot.GetDimSlotForMouseItem` (`ItemSlot.cs:1361-1420`) for the ported
/// contexts: with a non-air mouse item the dye slots (12/33) dim to 0.6 when
/// the carried item is not a dye (`checkItem.dye <= 0`). The armour/mount/pet
/// arms need the equip-validation tables (unported) and stay 1.
fn dim_slot_item_fade(context: i32, mouse_air: bool, mouse_is_dye: bool) -> f32 {
    if !mouse_air && matches!(context, 12 | 33) && !mouse_is_dye {
        0.6
    } else {
        1.0
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_slot_flagged(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    x: f32,
    y: f32,
    context: i32,
    index: i32,
    selected: bool,
    scale: f32,
    tint: [f32; 4],
    favorite: bool,
    pulse: super::UiPulse,
    glow: Option<(i32, f32)>,
    coin_slot: Option<&mut CoinSlotState>,
    vanilla_slot: i32,
    mouse_air: bool,
    mouse_is_dye: bool,
) {
    // `ItemSlot.cs:2109-2112` + `Main.cs:43615`: the derived `lightColor`.
    let light_color = if context == 13 {
        closed_hotbar_light_color(scale)
    } else {
        [1.0; 4]
    };
    // `ItemSlot.cs:2232-2242`: the closed hotbar (context 13) picks its own
    // background and hardcodes the tint, ignoring `Main.inventoryBack`.
    let (back, tint) = if context == 13 {
        closed_hotbar_background(lookups, selected)
    } else if favorited_background_active(context, favorite, item) {
        // `ItemSlot.cs:2149-2156`: the favorited arm sits at the head of the
        // C# sheet chain — it beats the hotbar row frame (`context == 0 && slot
        // < 10` → `Inventory_Back9`) too.
        (favorited_background_for(lookups, context), tint)
    } else {
        (background_for(lookups, context, index), tint)
    };
    // W4-g56 — `ItemSlot.cs:2314-2326`: the chest-glow arm runs AFTER the
    // context sheet chain and overrides it (a glowing non-air non-favorited
    // slot in context 3/4/32 swaps to `Inventory_Back13` + the lerped tint).
    // The favorite gate (`!inv[slot].favorited`) and the air gate
    // (`!inv[slot].IsAir`) match the C#; the favorite arm above already won
    // for a favorited slot, so this only fires for the plain ones.
    let (back, tint) = if !favorite && item.count > 0 {
        match glow.and_then(|(time, hue)| super::glow::chest_glow_tint(context, time, hue)) {
            Some(glow_tint) => (lookups.ui_inv_bg13, glow_tint),
            None => (back, tint),
        }
    } else {
        (back, tint)
    };
    // `ItemSlot.cs:2499`: cell = `value.Size()`, the chosen sheet's native size
    // (the retail `Inventory_Back*` sheets are 52x52; the 56 px is the layout
    // pitch, `Main.cs:41026`). Contexts 14/21/37 (`flag3`, `ItemSlot.cs:2236-2240`)
    // draw no background at all — the mouse item rides on them — but still centre
    // on `InventoryBack.Size()`.
    let cell = back.map(|b| b.width as f32).unwrap_or(52.0);
    let no_background = matches!(context, 14 | 21 | 37);

    // `ItemSlot.cs:2349`: `CoinSlot.UpdateDrawState(slot, context, item, out
    // drawState)` runs on EVERY slot draw, BEFORE the background. Contexts
    // 0/1/2 feed the inventory array, 3/4 the chest array; the rest get the
    // neutral state (`CoinSlot.cs:292-312`).
    let draw_state = match coin_slot {
        Some(cs) => cs.update_draw_state(vanilla_slot, context, item.kind, item.count),
        None => neutral_draw_state(),
    };
    // `ItemSlot.cs:2350-2352`: `GetDimSlotForMouseItem` → `itemFade`, and the
    // BACKGROUND tint `color2 *= itemFade`.
    let item_fade = dim_slot_item_fade(context, mouse_air, mouse_is_dye);
    let bg_tint = [tint[0] * item_fade, tint[1] * item_fade, tint[2] * item_fade, tint[3] * item_fade];
    if !no_background {
        if let Some(uv) = lookups.hud_uv(back.as_ref()) {
            r.push_ui_sprite([x, y], [cell * scale, cell * scale], uv, bg_tint);
        }
    }

    // `ItemSlot.cs:2499`: `vector = value.Size() * inventoryScale`; both the icon
    // and the empty glyph are centred on `position + vector / 2`.
    let center = (x + cell * 0.5 * scale, y + cell * 0.5 * scale);

    // `ItemSlot.cs:2490-2498`: glyphs are only drawn on an empty slot.
    if item.count == 0 {
        draw_equip_glyph(r, lookups, context, index, center, scale);
    }

    // `ItemSlot.cs:2500`: `flag4 = (item.type > 0 && item.stack > 0) ||
    // drawState.fadeItem > 0` — a fading-out coin keeps drawing its icon and
    // stack text after the slot went air. (The `_nextTickDrawAvailable`
    // throttle, `:2503-2511`, is unported.)
    let has_item = (item.kind > 0 && item.count > 0) || draw_state.fade_item > 0;
    if has_item {
        // `ItemSlot.cs:2515`: the icon. A coin (or a fading coin) rides the
        // SPIN strip via `CoinSlot.DrawItemCoin` at `sizeLimit = 32`, lifted
        // by `coinYOffset * inventoryScale` and faded by `itemFade *
        // fadeScale`; the medal 3817 keeps its STATIC icon but still rides
        // the jump (the third arm); every other item is the plain icon.
        let is_coin = (71..=74).contains(&item.kind);
        if is_coin || draw_state.fade_item > 0 {
            let kind = if draw_state.fade_item > 0 { draw_state.fade_item } else { item.kind };
            let coin_center = (center.0, center.1 - draw_state.coin_y_offset * scale);
            draw_item_coin(
                r, lookups, coin_center, (kind - 71) as usize, draw_state.coin_anim_frame,
                scale, 32.0, light_color, item_fade * draw_state.fade_scale,
            );
        } else if item.kind == 3817 {
            let medal_center = (center.0, center.1 - draw_state.coin_y_offset * scale);
            draw_item_icon(r, lookups, item, medal_center, scale, light_color, item_fade);
        } else {
            draw_item_icon(r, lookups, item, center, scale, light_color, item_fade);
        }
        // `ItemSlot.cs:2891-2929` (W4-g35): the glowMask overlay, drawn
        // right after the icon (the tinted-item second pass is unported).
        draw_item_glow(r, lookups, item, center, scale, pulse);
        // `ItemSlot.cs:2697-2706`: the stack count, context != 37, drawn when
        // `stack > 1 || stackTextDrawFadeOverload > 0`; scale
        // `inventoryScale * stackTextScale`, tint `color * (overload > 0 ?
        // overload : 1)`. The port has no ammo/paint/wand count override
        // (`num22`), so the `num22 == -1` gate is always true.
        if context != 37 && (item.count > 1 || draw_state.stack_text_draw_fade_overload > 0.0) {
            let line_spacing = lookups
                .item_font
                .as_ref()
                .map(|f| f.line_spacing)
                .unwrap_or(0.0);
            let text_scale = scale * draw_state.stack_text_scale;
            let (tx, ty) = stack_number_pos(x, y, scale, line_spacing, draw_state.stack_text_scale, context);
            let overload = if draw_state.stack_text_draw_fade_overload > 0.0 {
                draw_state.stack_text_draw_fade_overload
            } else {
                1.0
            };
            let text_color = [
                light_color[0] * overload, light_color[1] * overload,
                light_color[2] * overload, light_color[3] * overload,
            ];
            let text = item.count.to_string();
            hud_item_text_shadowed(r, lookups, tx, ty, text_scale, text_color, text_scale, &text);
        }
    }

    // `ItemSlot.cs:2708-2716`: the closed-hotbar digits are drawn inside the
    // item block (`flag4`), so an empty slot shows no number.
    if context == 13 && has_item {
        let (dx, dy) = closed_hotbar_digit_offset(x, y, scale);
        let text = hotbar_digit(index);
        hud_item_text_shadowed(r, lookups, dx, dy, scale, light_color, scale, &text);
    }

    // `ItemSlot.cs:2753-2771`: the open-inventory hotbar digits (context 0,
    // slot < 10 only) — always drawn, empty slot or not.
    if context == 0 && (0..10).contains(&index) {
        // `Color baseColor = Main.inventoryBack;` unless selected (`Color.White`,
        // `A = 200`). `tint` mirrors `Main.inventoryBack`.
        let color = if selected {
            [1.0, 1.0, 1.0, 200.0 / 255.0]
        } else {
            tint
        };
        let (dx, dy) = hotbar_digit_offset(x, y, selected, scale);
        let text = hotbar_digit(index);
        // NOTE: `ItemSlot.cs:2755,2768` computes `num28 = inventoryScale * 1.4f`
        // for the selected slot but the draw call passes `new Vector2(inventoryScale)`,
        // so the digit is not actually enlarged.
        hud_item_text_shadowed(r, lookups, dx, dy, scale, color, scale, &text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: u32) -> SheetRect {
        SheetRect {
            x,
            y: 0,
            width: 52,
            height: 52,
            plain_frame: (0, 0),
        }
    }

    fn approx(a: (f32, f32), b: (f32, f32)) {
        assert!(
            (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4,
            "{a:?} != {b:?}"
        );
    }

    #[test]
    fn background_texture_per_context() {
        // `ItemSlot.cs:2172-2175`: context 0 is `Inventory_Back9` for slot < 10.
        assert_eq!(context_texture_index(0, 0), 1);
        assert_eq!(context_texture_index(0, 9), 1);
        assert_eq!(context_texture_index(0, 10), 0);
        assert_eq!(context_texture_index(0, 49), 0);
        // coins (1) and ammo (2) use the plain grid background.
        assert_eq!(context_texture_index(1, 0), 0);
        assert_eq!(context_texture_index(2, 3), 0);
        // reforge (5) and guide (7) share `Inventory_Back4`.
        assert_eq!(context_texture_index(5, 0), 2);
        assert_eq!(context_texture_index(7, 0), 2);
        // trash (6) is `Inventory_Back7`.
        assert_eq!(context_texture_index(6, 0), 3);
        // armour / vanity / dye (8-12) are `Inventory_Back13`.
        for c in 8..=12 {
            assert_eq!(context_texture_index(c, 0), 4, "context {c}");
        }
        // misc equips (16-20) are `Inventory_Back3`.
        for c in 16..=20 {
            assert_eq!(context_texture_index(c, 0), 5, "context {c}");
        }
        // misc dye (33) is `Inventory_Back12`.
        assert_eq!(context_texture_index(33, 0), 6);
        // the world-chest panel (3) is `Inventory_Back5` (`ItemSlot.cs:2215-2218`).
        assert_eq!(context_texture_index(3, 0), 8);
        assert_eq!(context_texture_index(3, 39), 8);
        // The bank contexts (4/32) map to `Inventory_Back2` in the C#
        // (`ItemSlot.cs:2219-2221`) — M3-STORE-1's; deliberately left on the
        // default like contexts 22/34 below.
        assert_eq!(context_texture_index(4, 0), 0);
        assert_eq!(context_texture_index(32, 0), 0);
    }

    #[test]
    fn background_for_selects_the_sheet_per_context() {
        // `Lookups::default()` has every sheet `None`, so set discriminating
        // sheets and compare the chosen rect's `x`.
        let mut lk = assets::Lookups::default();
        lk.ui_back = Some(rect(10));
        lk.ui_back_hotbar = Some(rect(11));
        lk.ui_back4 = Some(rect(12));
        lk.ui_back7 = Some(rect(13));
        lk.ui_inv_bg13 = Some(rect(14));
        lk.ui_back3 = Some(rect(15));
        lk.ui_back12 = Some(rect(16));
        lk.ui_back5 = Some(rect(17));

        let pick = |c, i| background_for(&lk, c, i).map(|r| r.x);
        assert_eq!(pick(0, 5), Some(11)); // hotbar row
        assert_eq!(pick(0, 15), Some(10)); // main grid
        assert_eq!(pick(1, 0), Some(10)); // coins
        assert_eq!(pick(2, 0), Some(10)); // ammo
        assert_eq!(pick(3, 0), Some(17)); // the chest panel
        assert_eq!(pick(5, 0), Some(12)); // reforge
        assert_eq!(pick(7, 0), Some(12)); // guide
        assert_eq!(pick(6, 0), Some(13)); // trash
        assert_eq!(pick(10, 0), Some(14)); // accessory
        assert_eq!(pick(20, 0), Some(15)); // misc equip
        assert_eq!(pick(33, 0), Some(16)); // misc dye
    }

    #[test]
    fn background_for_is_none_when_the_sheet_is_absent() {
        let lk = assets::Lookups::default();
        assert!(background_for(&lk, 0, 0).is_none());
        assert!(background_for(&lk, 33, 0).is_none());
        assert!(background_for(&lk, 3, 0).is_none());
    }

    #[test]
    fn stack_number_offset_uses_the_source_formula() {
        // `ItemSlot.cs:2699-2706`: at rest (stackTextScale 1) the net top-left
        // is `(10, 26) * scale`, independent of `LineSpacing` (it cancels with
        // the scaled origin).
        approx(stack_number_pos(0.0, 0.0, 0.85, 17.0, 1.0, 0), (8.5, 22.1));
        approx(stack_number_pos(100.0, 200.0, 0.6, 30.0, 1.0, 0), (106.0, 215.6));
        approx(stack_number_pos(0.0, 0.0, 1.0, 5.0, 1.0, 0), (10.0, 26.0));
        approx(stack_number_pos(0.0, 0.0, 1.0, 40.0, 1.0, 0), (10.0, 26.0));
        // The text pulse (stackTextScale 1.25): num23 = scale*1.25, so the net
        // y lifts by ls*(num23 - scale) = ls*0.25*scale. At scale 1, ls 20:
        // y = (26+20) - 20*1.25 = 21 (5 px up from the rest 26).
        approx(stack_number_pos(0.0, 0.0, 1.0, 20.0, 1.25, 0), (10.0, 21.0));
        // Context 43 shifts (-5, +7) and scales num23 by 1.2: at rest
        // num23 = 1.2, y = (26+7+20) - 20*1.2 = 29, x = (10-5) = 5.
        approx(stack_number_pos(0.0, 0.0, 1.0, 20.0, 1.0, 43), (5.0, 29.0));
    }

    /// `ItemSlot.GetDimSlotForMouseItem` (`ItemSlot.cs:1361`): the dye contexts
    /// dim to 0.6 with a non-dye mouse item; an air cursor never dims.
    #[test]
    fn the_dye_slots_dim_for_a_non_dye_mouse_item() {
        assert_eq!(dim_slot_item_fade(12, false, false), 0.6);
        assert_eq!(dim_slot_item_fade(33, false, false), 0.6);
        assert_eq!(dim_slot_item_fade(12, false, true), 1.0, "a dye does not dim");
        assert_eq!(dim_slot_item_fade(12, true, false), 1.0, "an air cursor never dims");
        assert_eq!(dim_slot_item_fade(0, false, false), 1.0, "the grid is unported-dim");
    }

    /// The neutral state matches `CoinSlot.UpdateDrawState`'s tail
    /// (`CoinSlot.cs:306-311`): no jump, no pulse, no fade, scale 1.
    #[test]
    fn the_neutral_draw_state_matches_the_csharp_tail() {
        let n = neutral_draw_state();
        assert_eq!(n.coin_anim_frame, 0);
        assert_eq!(n.coin_y_offset, 0.0);
        assert_eq!(n.stack_text_scale, 1.0);
        assert_eq!(n.stack_text_draw_fade_overload, -1.0);
        assert_eq!(n.fade_item, 0);
        assert_eq!(n.fade_scale, 1.0);
    }

    #[test]
    fn hotbar_digit_renders_ten_as_zero() {
        assert_eq!(hotbar_digit(0), "1");
        assert_eq!(hotbar_digit(8), "9");
        assert_eq!(hotbar_digit(9), "0");
    }

    #[test]
    fn hotbar_digit_offset_moves_up_when_selected() {
        approx(hotbar_digit_offset(0.0, 0.0, false, 0.85), (5.1, 3.4));
        approx(hotbar_digit_offset(0.0, 0.0, true, 0.85), (5.1, 1.7));
    }

    #[test]
    fn closed_hotbar_digit_offset_is_eight_four() {
        // `ItemSlot.cs:2693-2701`: context 13 digits sit at `(8, 4) * scale` and
        // never lift on the selected slot (that is the context-0 row's rule).
        approx(closed_hotbar_digit_offset(0.0, 0.0, 0.85), (6.8, 3.4));
        approx(closed_hotbar_digit_offset(100.0, 200.0, 1.0), (108.0, 204.0));
    }

    #[test]
    fn closed_hotbar_light_color_fades_with_the_scale() {
        // `Main.cs:43615`: `new Color(255, 255, 255, (int)(75f + 150f * num2))` —
        // alpha 187.5 at the 0.75 idle scale, 225 when selected.
        let idle = closed_hotbar_light_color(0.75);
        assert!((idle[3] - 187.5 / 255.0).abs() < 1e-6);
        assert_eq!(idle[0], 1.0);
        assert_eq!(closed_hotbar_light_color(1.0)[3], 225.0 / 255.0);
    }

    #[test]
    fn closed_hotbar_metrics_match_guihotbardrawinner() {
        // `Main.cs:43614,43639`: y `(int)(20 + 22 * (1 - scale))`, advance
        // `(int)(52 * scale) + 4`, hover size the untruncated `52 * scale`.
        // Steady states: the selected slot at 1.0 and the rest at 0.75.
        assert_eq!(closed_hotbar_metrics(1.0), (20.0, 56.0, 52.0));
        assert_eq!(closed_hotbar_metrics(0.75), (25.0, 43.0, 39.0));
        // Mid-lerp the ints truncate: 52 * 0.8 = 41.6 -> 41.
        let (y, advance, size) = closed_hotbar_metrics(0.8);
        assert_eq!((y, advance), (24.0, 45.0));
        assert!((size - 41.6).abs() < 1e-4);
    }

    #[test]
    fn closed_hotbar_background_is_back14_when_selected_back_at_200_otherwise() {
        // `ItemSlot.cs:2232-2242`: `b = 200`; selected swaps to `Inventory_Back14`
        // and `b = 255`.
        let mut lk = assets::Lookups::default();
        lk.ui_back = Some(rect(10));
        lk.ui_back_sel = Some(rect(14));
        let (sheet, tint) = closed_hotbar_background(&lk, true);
        assert_eq!(sheet.map(|s| s.x), Some(14));
        assert_eq!(tint, [1.0; 4]);
        let (sheet, tint) = closed_hotbar_background(&lk, false);
        assert_eq!(sheet.map(|s| s.x), Some(10));
        assert_eq!(tint, [200.0 / 255.0; 4]);
    }

    #[test]
    fn equip_glyph_index_map() {
        // `ItemSlot.cs:2406-2420` context 8: slot 0/1/2 -> 0/6/12.
        assert_eq!(equip_glyph_index(8, 0), 0);
        assert_eq!(equip_glyph_index(8, 1), 6);
        assert_eq!(equip_glyph_index(8, 2), 12);
        assert_eq!(equip_glyph_index(8, 3), -1);
        // `ItemSlot.cs:2424-2437` context 9: slot 10/11/12 -> 3/9/15.
        assert_eq!(equip_glyph_index(9, 10), 3);
        assert_eq!(equip_glyph_index(9, 11), 9);
        assert_eq!(equip_glyph_index(9, 12), 15);
        assert_eq!(equip_glyph_index(9, 13), -1);
        // The single-value contexts.
        assert_eq!(equip_glyph_index(10, 0), 11);
        assert_eq!(equip_glyph_index(11, 0), 2);
        assert_eq!(equip_glyph_index(12, 0), 1);
        assert_eq!(equip_glyph_index(16, 0), 4);
        assert_eq!(equip_glyph_index(17, 0), 13);
        assert_eq!(equip_glyph_index(18, 0), 7);
        assert_eq!(equip_glyph_index(19, 0), 10);
        assert_eq!(equip_glyph_index(20, 0), 17);
        assert_eq!(equip_glyph_index(33, 0), 1);
        assert_eq!(equip_glyph_index(7, 0), 18);
        // No glyph for contexts outside the table.
        assert_eq!(equip_glyph_index(0, 0), -1);
    }

    #[test]
    fn glyph_frame_is_frame_3_7_minus_two() {
        // A 168x392 sheet is 3x7 cells of 56; each frame loses 2 px on each side.
        assert_eq!(glyph_frame(168, 392, 0), Some((0, 0, 54, 54)));
        assert_eq!(glyph_frame(168, 392, 12), Some((0, 224, 54, 54)));
        assert_eq!(glyph_frame(168, 392, 17), Some((112, 280, 54, 54)));
        assert_eq!(glyph_frame(168, 392, 18), Some((0, 336, 54, 54)));
        // Out-of-range / degenerate sheets yield nothing.
        assert_eq!(glyph_frame(168, 392, -1), None);
        assert_eq!(glyph_frame(2, 392, 0), None);
    }

    #[test]
    fn crafting_contexts_use_inventory_back18() {
        // `ItemSlot.cs:2264-2277`: 42/43 start on `Inventory_Back4` then override to
        // `Inventory_Back18`; `:2298-2302`: 41 is `Inventory_Back18` outright.
        for c in [41, 42, 43] {
            assert_eq!(context_texture_index(c, 0), 7, "context {c}");
            assert_eq!(context_texture_index(c, 5), 7, "context {c} slot 5");
        }
        // Contexts 22/34 are out of P3 scope -- deliberately left on the default,
        // even though the C# maps 22 -> `Inventory_Back4` (`ItemSlot.cs:2264-2267`)
        // and 34 -> `Inventory_Back18` (`:2257-2259`).
        assert_eq!(context_texture_index(22, 0), 0);
        assert_eq!(context_texture_index(34, 0), 0);
    }

    #[test]
    fn crafting_contexts_pick_the_back18_sheet() {
        let mut lk = assets::Lookups::default();
        lk.ui_back = Some(rect(10));
        lk.ui_back4 = Some(rect(12));
        lk.ui_back18 = Some(rect(18));
        let pick = |c| background_for(&lk, c, 0).map(|r| r.x);
        assert_eq!(pick(41), Some(18));
        assert_eq!(pick(42), Some(18));
        assert_eq!(pick(43), Some(18));
    }

    #[test]
    fn craft_slot_tint_is_the_shifted_recolour() {
        // `ItemSlot.cs:2270-2271` / `:2299-2300`:
        // `ShiftBlueToCyanTheme((16,36,56,180) * 0.9f)` = `[10,58,74,162]`.
        assert_eq!(craft_slot_tint(), [
            10.0 / 255.0,
            58.0 / 255.0,
            74.0 / 255.0,
            162.0 / 255.0,
        ]);
    }

    // M3-CHEST-INT phase 4 — the favorited slot's draw args (DW-4.5).

    #[test]
    fn test_DW_4_5_favorited_background_gate_matches_the_csharp_arm() {
        // `ItemSlot.cs:2149-2156`: `!item.IsAir && item.favorited &&
        // ItemSlot.retainFavoriteOnMoveAt[context]`.
        let item = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 3,
            prefix: 0,
        };
        let air = crate::item::EMPTY_SLOT;
        assert!(favorited_background_active(0, true, &item));
        assert!(favorited_background_active(32, true, &item));
        assert!(favorited_background_active(5, true, &item), "retain[5] = true");
        assert!(
            !favorited_background_active(0, true, &air),
            "IsAir returns before the arm"
        );
        assert!(!favorited_background_active(0, false, &item));
        assert!(
            !favorited_background_active(3, true, &item),
            "ctx 3 is not in the retain table"
        );
        assert!(
            !favorited_background_active(6, true, &item),
            "the trash slot is not retained either"
        );
        assert!(!favorited_background_active(13, true, &item));
    }

    #[test]
    fn test_DW_4_5_favorited_slot_uses_the_gold_sheet() {
        // `ItemSlot.cs:2149-2158`: the favorited arm swaps the sheet to the
        // gold `Inventory_Back10` and keeps `Main.inventoryBack` as the tint.
        let mut lk = assets::Lookups::default();
        lk.ui_back = Some(rect(0));
        lk.ui_back_hotbar = Some(rect(9));
        lk.ui_back10 = Some(rect(10));
        let favorited = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 3,
            prefix: 0,
        };
        let pick = |c, f| {
            if favorited_background_active(c, f, &favorited) {
                favorited_background_for(&lk, c)
            } else {
                background_for(&lk, c, 0)
            }
        };
        assert_eq!(pick(0, true).map(|r| r.x), Some(10), "gold, not the hotbar row");
        assert_eq!(pick(1, true).map(|r| r.x), Some(10));
        assert_eq!(pick(5, true).map(|r| r.x), Some(10));
        assert_eq!(pick(0, false).map(|r| r.x), Some(9), "unfavorited hotbar row");
    }

    // W4-g35 — the item-slot glow overlay (`ItemSlot.cs:2891-2929`).

    #[test]
    fn w4_g35_the_glow_gate_excludes_the_three_slash_items() {
        // `:2891`: `item.glowMask != -1 && type != 3779 && type != 46 &&
        // type != 5462`. All three carry a non-stock glowMask in the table
        // (3779/218, 46/328, 5462/355) — the gate, not the data, excludes them.
        assert!(!item_glow_active(3779));
        assert!(!item_glow_active(46));
        assert!(!item_glow_active(5462));
        // A non-stock row draws; the stock -1 stays dark.
        assert!(item_glow_active(2757)); // Vortex Helmet, glow 26
        assert!(item_glow_active(5146)); // the rainbow bobber, glow 324
        assert!(!item_glow_active(1)); // Iron Pickaxe, stock -1
        assert!(!item_glow_active(0)); // air
    }

    #[test]
    fn w4_g35_the_phase_colour_ramp_matches_get_phase_color_direct() {
        // `Item.cs:49111-49168` — the nine `Color(num, num2, num3)` rows
        // (×255, clamped): the 1.2/1.05 channels saturate at 255.
        assert_eq!(PHASE_COLORS[0], [25, 127, 255, 255]);
        assert_eq!(PHASE_COLORS[3], [204, 25, 255, 255]);
        assert_eq!(PHASE_COLORS[6], [229, 127, 0, 255]);
        assert_eq!(PHASE_COLORS[7], [255, 25, 102, 255]);
        assert_eq!(PHASE_COLORS[8], PHASE_COLORS[0], "the 1059 repeat");
        // `MultiLerp(0)` = the first colour; `MultiLerp(1)` = the last.
        assert_eq!(phase_multi_lerp(0.0), PHASE_COLORS[0]);
        assert_eq!(phase_multi_lerp(1.0), PHASE_COLORS[8]);
        // The segment walk stops at index 7 — a percent > 1 extrapolates
        // (`Utils.cs:297`'s `num3 < colors.Length - 2` guard): 1.25 lands
        // amount 3 on the 1075→1059 pair, the byte cast wrapping negative.
        let over = phase_multi_lerp(1.25);
        assert_eq!(over, [77, 75, 49, 255], "amount 3 past the 1075→1059 pair");
        // `GetPhaseColor`: only the 1101/1102 shoot pair reads the ramp;
        // `drawColor: true` forces A = 127 (`Item.cs:49102`).
        assert_eq!(phase_color(1101, 0, false), PHASE_COLORS[0]);
        assert_eq!(phase_color(1102, 0, true)[3], 127);
        assert_eq!(phase_color(1, 0, false), [255, 255, 255, 255]);
    }

    #[test]
    fn w4_g35_the_glow_tints_match_the_itemslot_switch() {
        let pulse = super::UiPulse {
            misc_counter: 0,
            disco: [10, 20, 30],
            global_time: 0.0,
            ..Default::default()
        };
        // `:2902`: the rainbow bobber reads Main.DiscoR/G/B, A 255.
        assert_eq!(item_glow_tint(5146, 0, pulse), [10, 20, 30, 255]);
        // `:2905`: the dead 5462 arm (the gate excludes it) — verbatim.
        assert_eq!(item_glow_tint(5462, 0, pulse), [255, 140, 0, 5]);
        // `:2912`: `(255,255,255,63) * 0.75` truncates per byte.
        assert_eq!(item_glow_tint(3858, 0, pulse), [191, 191, 191, 47]);
        // `:2908`: miscCounter 0 → WrappedLerp(0.5, 1, 0) = 0.5 → the
        // White→(180,85,30) midpoint, A = item.alpha (stock 0).
        assert_eq!(item_glow_tint(5669, 0, pulse), [217, 170, 142, 0]);
        // miscCounter 50 → percent 0.5 → WrappedLerp folds to 1 → the end.
        let p50 = super::UiPulse { misc_counter: 50, ..pulse };
        assert_eq!(item_glow_tint(5669, 0, p50), [180, 85, 30, 0]);
        // The default arm keeps White (`:2894`).
        assert_eq!(item_glow_tint(2757, 0, pulse), [255, 255, 255, 255]);
        // `:2899`: the phase pair reads the item's shoot through the ramp.
        assert_eq!(item_glow_tint(5670, 1101, pulse), PHASE_COLORS[0]);
    }

    #[test]
    fn w4_g35_the_trusty_pal_glow_ramp_matches_the_source() {
        // `ItemSlot.cs:2954-2959`: t = 0 → num = WrappedLerp(0,1,0) = 0,
        // num2 = Remap(0, 0.2, 1, 0, 1) clamps to 0 → the whole colour is 0.
        assert_eq!(trusty_pal_glow_color(0.0), [0, 0, 0, 0]);
        // t = 1.5 → (1.5/3) % 1 = 0.5 → WrappedLerp folds to 1 → num = 1,
        // num2 = 1 → hslToRgb(0.11, 1, 0.5) * 0.5, A = 127 * 0.5 = 63.
        let c = trusty_pal_glow_color(1.5);
        assert_eq!(c[3], 63);
        let rgb = tr_core::tile_light::hsl_to_rgb(0.11, 1.0, 0.5);
        assert_eq!(c[0], (rgb[0] * 255.0 * 0.5) as u8);
        assert_eq!(c[1], (rgb[1] * 255.0 * 0.5) as u8);
        assert_eq!(c[2], (rgb[2] * 255.0 * 0.5) as u8);
        // The hue stays in the 0.04..0.11 band (the % 1 never wraps there).
        let mid = trusty_pal_glow_color(0.75); // num = 0.5, num2 = 0.375
        assert!(mid[0] > 0 && mid[3] > 0, "the ramp is live mid-cycle");
    }

    #[test]
    fn w4_g35_the_monk_staff_rect_is_the_three_row_strip() {
        // `:2913-2914`: `Glow_233.Frame(1, 3, 0, miscCounter % 15 / 5)`,
        // `Height -= 2`. The row pick cycles 0/1/2 every 5 ticks.
        let row = |c: u32| (c % 15 / 5) as i32;
        assert_eq!(row(0), 0);
        assert_eq!(row(4), 0);
        assert_eq!(row(5), 1);
        assert_eq!(row(10), 2);
        assert_eq!(row(14), 2);
        assert_eq!(row(15), 0);
    }

    /// W4-g56 — `ItemSlot.cs:2155-2158`: the context-32 favorite swaps to
    /// `Inventory_Back19` (the vault's purple frame), not the gold one; the
    /// unpacked sheet falls back to the gold frame.
    #[test]
    fn w4_g56_the_vault_favorite_uses_inventory_back19() {
        let mut lk = assets::Lookups::default();
        lk.ui_back10 = Some(rect(10));
        assert_eq!(
            favorited_background_for(&lk, 32).map(|r| r.x),
            Some(10),
            "the unpacked _19 falls back to the gold frame"
        );
        lk.ui_back19 = Some(rect(19));
        assert_eq!(favorited_background_for(&lk, 32).map(|r| r.x), Some(19));
        assert_eq!(
            favorited_background_for(&lk, 0).map(|r| r.x),
            Some(10),
            "the grid keeps the gold frame"
        );
        // `ItemSlot.cs:2218-2221`: contexts 4 AND 32 draw Inventory_Back2.
        lk.ui_back2 = Some(rect(2));
        assert_eq!(background_for(&lk, 4, 0).map(|r| r.x), Some(2));
        assert_eq!(background_for(&lk, 32, 7).map(|r| r.x), Some(2));
    }
}
