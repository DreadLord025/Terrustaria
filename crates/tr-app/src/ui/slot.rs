//! Inventory and hotbar slots: a faithful port of `ItemSlot.Draw`
//! (`decompiled/Terraria/UI/ItemSlot.cs:2105-2776`).
//!
//! Covered contexts (the ones the P2 inventory panels use): 0 (main grid/hotbar),
//! 1 (coins), 2 (ammo), 3 (the world-chest panel, `Inventory_Back5`), 5 (reforge),
//! 6 (trash), 7 (guide), 8-12 (armour/vanity/dye), 13 (the closed hotbar,
//! `GUIHotbarDrawInner`), 21 (the mouse-carried item, no background —
//! `Main.cs:45194`), 16-20 (misc equips) and 33 (misc dyes).

use tr_render::{SheetRect, SpriteRenderer};

use super::text::hud_item_text;
use crate::assets;
use crate::item::ItemSlot;

/// `ItemSlot.cs:2148-2304`: which `Inventory_Back*` sheet a slot uses.
///
/// The numbers are indices into this port's sheet set: `0` `Inventory_Back`,
/// `1` `Inventory_Back9`, `2` `Inventory_Back4`, `3` `Inventory_Back7`,
/// `4` `Inventory_Back13`, `5` `Inventory_Back3`, `6` `Inventory_Back12`,
/// `7` `Inventory_Back18`, `8` `Inventory_Back5`.
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

/// The favorited slot's draw tint — the fallback for `ItemSlot.cs:2149-2156`'s
/// gold `Inventory_Back10` background (32 → `Inventory_Back19`). The port's
/// atlas packs neither sheet (`assets.rs`'s sheet list has Back 2/3/4/5/6/7/9/
/// 12/13/14/18 only), so the arm tints the slot's own background with
/// `Main.OurFavoriteColor` = `Color(255, 231, 69)` (`Main.cs:66748`) — the
/// plan-authorized fallback, recorded in Phase 5's ANALYSIS entry.
pub(crate) fn favorited_slot_tint(inv_back: [f32; 4]) -> [f32; 4] {
    // `Main.OurFavoriteColor` (`Main.cs:66748`), XNA byte-multiplied into the
    // caller's `Main.inventoryBack` tint (the C# keeps `color2 =
    // Main.inventoryBack` on the favorited arm and swaps the sheet instead).
    let favorite = [255.0 / 255.0, 231.0 / 255.0, 69.0 / 255.0, 1.0];
    [
        favorite[0] * inv_back[0],
        favorite[1] * inv_back[1],
        favorite[2] * inv_back[2],
        favorite[3] * inv_back[3],
    ]
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

/// `ItemSlot.cs:2699-2706`: the stack text is positioned at
/// `position + (10, 26 + LineSpacing) * inventoryScale` with origin
/// `(0, LineSpacing) * inventoryScale`, so its net top-left is `(10, 26) * scale`.
fn stack_number_pos(x: f32, y: f32, scale: f32, line_spacing: f32) -> (f32, f32) {
    let pos = (x + 10.0 * scale, y + (26.0 + line_spacing) * scale);
    (pos.0, pos.1 - line_spacing * scale)
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
    r.push_ui_sprite([center.0 - w * 0.5, center.1 - h * 0.5], [w, h], uv, tint);
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
    } else {
        (background_for(lookups, context, index), tint)
    };
    // `ItemSlot.cs:2499`: cell = `value.Size()`, the chosen sheet's native size
    // (the retail `Inventory_Back*` sheets are 52x52; the 56 px is the layout
    // pitch, `Main.cs:41026`). Contexts 14/21/37 (`flag3`, `ItemSlot.cs:2236-2240`)
    // draw no background at all — the mouse item rides on them — but still centre
    // on `InventoryBack.Size()`.
    let cell = back.map(|b| b.width as f32).unwrap_or(52.0);
    let no_background = matches!(context, 14 | 21 | 37);
    if !no_background {
        if let Some(uv) = lookups.hud_uv(back.as_ref()) {
            r.push_ui_sprite([x, y], [cell * scale, cell * scale], uv, tint);
        }
    }

    // `ItemSlot.cs:2499`: `vector = value.Size() * inventoryScale`; both the icon
    // and the empty glyph are centred on `position + vector / 2`.
    let center = (x + cell * 0.5 * scale, y + cell * 0.5 * scale);

    if item.count == 0 {
        // `ItemSlot.cs:2490-2498`: glyphs are only drawn on an empty slot.
        draw_equip_glyph(r, lookups, context, index, center, scale);
    } else {
        // `ItemSlot.cs:2513-2515`: the item icon (coins/ammo too), tinted by
        // `item.GetAlpha(lightColor)` (`ItemSlot.cs:2876,2970`).
        draw_item_icon(r, lookups, item, center, scale, light_color);
        // `ItemSlot.cs:2703-2712`: the stack count, context != 37, tinted by
        // `color` (= `lightColor`; `drawState.stackTextScale` is 1 outside the
        // coin-jump animation, `CoinSlot.cs:316-322`).
        if item.count > 1 {
            let line_spacing = lookups
                .item_font
                .as_ref()
                .map(|f| f.line_spacing)
                .unwrap_or(0.0);
            let (tx, ty) = stack_number_pos(x, y, scale, line_spacing);
            let text = item.count.to_string();
            hud_item_text_shadowed(r, lookups, tx, ty, scale, light_color, scale, &text);
        }
    }

    // `ItemSlot.cs:2693-2701`: the closed-hotbar digits are drawn inside the
    // item block (`flag4`), so an empty slot shows no number.
    if context == 13 && item.count > 0 {
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
        // `ItemSlot.cs:2699-2706`: net top-left is `(10, 26) * scale`, independent
        // of `LineSpacing` (it cancels with the origin).
        approx(stack_number_pos(0.0, 0.0, 0.85, 17.0), (8.5, 22.1));
        approx(stack_number_pos(100.0, 200.0, 0.6, 30.0), (106.0, 215.6));
        approx(stack_number_pos(0.0, 0.0, 1.0, 5.0), (10.0, 26.0));
        approx(stack_number_pos(0.0, 0.0, 1.0, 40.0), (10.0, 26.0));
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
    fn test_DW_4_5_favorited_tint_is_our_favorite_color() {
        // `Main.OurFavoriteColor = Color(255, 231, 69)` (`Main.cs:66748`)
        // composed over `Main.inventoryBack` (220, 220, 220, 220) — the
        // fallback for the `Inventory_Back10` sheet the atlas lacks.
        let tint = favorited_slot_tint([220.0 / 255.0; 4]);
        assert_eq!(tint[0], 220.0 / 255.0);
        assert_eq!(tint[1], (231.0 / 255.0) * (220.0 / 255.0));
        assert_eq!(tint[2], (69.0 / 255.0) * (220.0 / 255.0));
        assert_eq!(tint[3], 220.0 / 255.0);
    }
}
