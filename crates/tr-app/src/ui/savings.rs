//! The savings coin pile — `ItemSlot.DrawSavings` (`ItemSlot.cs:3704-3741`)
//! + `ItemSlot.DrawMoney` (`:3652-3702`) + the `Utils.Coins*` helpers
//! (`Utils.cs:1019-1076`), W4-g12 doc-sweep. The g58 shop-happiness lane
//! deferred the pile here ("rides the coin-slot lane"); the g57 reforge
//! lane recorded the same skip. Three C# call sites:
//!
//! * the shop block: `DrawSavings(sb, 504f, invBottom)` (`Main.cs:41791`,
//!   vertical) — WIRED in `ui::mod`'s shop arm;
//! * the reforge block: `DrawSavings(sb, num51 + 130, invBottom,
//!   horizontal: true)` (`Main.cs:41637`, num51 = 50 `:41581`) — the g57
//!   reforge-anim lane owns that block's merge;
//! * the buy button: `DrawSavings(sb, num18, num19 - 70, horizontal: true)`
//!   (`Main.cs:43862`) — the NPC-chat buy surface, same follow-up.
//!
//! Deviations (recorded in ANALYSIS "The savings coin pile (W4-g12)"):
//! * `_customCurrencyForSavings != -1` (`:3707-3711`) routes to
//!   `CustomCurrencyManager.DrawSavings` — the custom-currency display is
//!   d122's UI follow-up; the port keeps the vanilla coin path.
//! * `CoinSlot.DrawItemCoin`'s tint is `ContentSamples.ItemsByType[coin]
//!   .GetAlpha(itemColor)` (`CoinSlot.cs:252`); a fresh sample item has
//!   `alpha = 0` and coins hit no special `GetAlpha` arm (`Item.cs:49170-
//!   49341`), so the tint is `Color.White` — folded to the constant.
//! * The pile icons' `GetItemDrawFrame` (`Main.cs:26898-26910`) picks an
//!   animated frame when `itemAnimations[id]` is registered; none of
//!   4076/3813/346/87 registers one, so the whole-sheet frame is exact.

use tr_render::SpriteRenderer;

use super::coin_slot::CoinSlotState;
use super::text::{self, BorderFont};
use crate::assets::{self, Lookups};
use crate::item::ItemSlot;

/// `Utils.CoinsCount` (`Utils.cs:1019-1046`): the copper value of the
/// coin stacks in one container. `ignoreSlots` is empty at every
/// `DrawSavings` call (`:3712-3715` pass only the array), so the port
/// takes no ignore list. `overFlowing` is always false (`:1044`) — the
/// out-param is dead and dropped.
pub(crate) fn coins_count(inv: &[ItemSlot]) -> i64 {
    let mut num = 0i64;
    for slot in inv {
        match slot.kind {
            71 => num += slot.count as i64,
            72 => num += slot.count as i64 * 100,
            73 => num += slot.count as i64 * 10000,
            74 => num += slot.count as i64 * 1000000,
            _ => {}
        }
    }
    num
}

/// `Utils.CoinsCombineStacks` (`Utils.cs:1062-1076`): the running sum
/// with the 9999999999 cap — the C# checks AFTER each add and returns
/// the cap immediately (later counts never read). `overFlowing` has no
/// consumer at the `DrawSavings` site (`:3716` discards it) — dropped.
pub(crate) fn coins_combine_stacks(coin_counts: &[i64]) -> i64 {
    let mut num = 0i64;
    for &c in coin_counts {
        num += c;
        if num >= 9999999999 {
            return 9999999999;
        }
    }
    num
}

/// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): `[copper, silver, gold,
/// platinum]` — the descending walk divides the REMAINDER by the
/// denomination, then subtracts it back (the `num` accumulator).
pub(crate) fn coins_split(count: i64) -> [i32; 4] {
    let mut array = [0i32; 4];
    let mut num = 0i64;
    let mut num2 = 1000000i64;
    for i in (0..4).rev() {
        array[i] = ((count - num) / num2) as i32;
        num += array[i] as i64 * num2;
        num2 /= 100;
    }
    array
}

/// `Utils.CenteredRectangle(Vector2, Vector2)` (`Utils.cs:947-950`):
/// `(int)(center - size/2)` position, `(int)size` extent — all four
/// casts truncate toward zero. Returns `(x, y, w, h)`.
fn centered_rect(cx: f32, cy: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
    (
        (cx - w / 2.0) as i32 as f32,
        (cy - h / 2.0) as i32 as f32,
        w as i32 as f32,
        h as i32 as f32,
    )
}

/// One pile icon (`ItemSlot.cs:3723-3738`): the item's whole-sheet frame
/// scaled 0.65, centred via `CenteredRectangle` — the destination rect
/// is the draw box (`sb.Draw(texture, rectangle, null, Color.White)`).
fn pile_icon(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    item: u16,
    cx: f32,
    cy: f32,
) {
    let Some((uv, iw, ih)) = lookups.item_icon(item) else {
        return;
    };
    let (x, y, w, h) = centered_rect(cx, cy, iw * 0.65, ih * 0.65);
    r.push_ui_sprite([x, y], [w, h], uv, [1.0; 4]);
}

/// `CoinSlot.DrawItemCoin` (`CoinSlot.cs:240-254`) at the `DrawMoney`
/// call shape (`scale 1, sizeLimit 1024, Color.White`): the `Coin_<i>`
/// strip's `Frame(1, 8, 0, coinFrame)` cell, centred on the anchor,
/// origin `rect.Size()/2` — at rotation 0 the top-left is
/// `center - cell/2 * scale`. The 1024 size limit never binds (the
/// cells are ~20 px), kept as the guard anyway.
fn draw_item_coin(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    center: [f32; 2],
    coin_type: usize,
    coin_frame: i32,
) {
    let Some(rect) = lookups.coin_strips.get(coin_type).and_then(|s| s.as_ref()) else {
        return;
    };
    // `Utils.Frame` (`Utils.cs:1344-1349`): integer cell divide, NO
    // padding subtraction (the coin strips are packed edge-to-edge).
    let cw = rect.width;
    let ch = rect.height / 8;
    let row = coin_frame.clamp(0, 7) as u32;
    let uv = assets::uv_rect(rect, lookups.atlas_size, 0, (row * ch) as i32, cw, ch);
    // `:247-250`: the sizeLimit guard (never fires at 1024).
    let limit = if cw > 1024 || ch > 1024 {
        if cw <= ch { 1024.0 / ch as f32 } else { 1024.0 / cw as f32 }
    } else {
        1.0
    };
    let (w, h) = (cw as f32 * limit, ch as f32 * limit);
    r.push_ui_sprite(
        [center[0] - w / 2.0, center[1] - h / 2.0],
        [w, h],
        uv,
        [1.0; 4],
    );
}

/// `Utils.DrawBorderStringFourWay` (`Utils.cs:2464-2495`) with an
/// explicit origin: the four ±2 px black side passes then the colour
/// pass, each `DrawString` shifting by `-origin * scale` (the port's
/// `text::draw_border_string` has no origin parameter — the shift is
/// applied to the anchor here).
#[allow(clippy::too_many_arguments)]
fn border_four_way(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    font: BorderFont,
    x: f32,
    y: f32,
    color: [f32; 4],
    origin: [f32; 2],
    scale: f32,
    text_str: &str,
) {
    let (bx, by) = (x - origin[0] * scale, y - origin[1] * scale);
    text::draw_border_string(
        r,
        lookups,
        font,
        bx,
        by,
        scale,
        &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
        color,
        text_str,
    );
}

/// `ItemSlot.DrawMoney` (`ItemSlot.cs:3652-3702`) with `fromSavings:
/// true`: the `Lang.inter[66]` label at `(shopx, shopy + 40)` in
/// `White * (mouseTextColor / 255)`, then the four coin cells
/// (platinum first — `coinsArray[3 - i]`, type `74 - i`), each count
/// capped at 999 and animated through `CoinSlot.UpdateSavings`.
/// `pulse_text` is `Main.mouseTextColor / 255` (`UiPulse.text`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_money(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    coin_state: &mut CoinSlotState,
    label: &str,
    shopx: f32,
    shopy: f32,
    coins_array: [i32; 4],
    horizontal: bool,
    pulse_text: f32,
) {
    // `:3654`: the label, origin zero, scale 1.
    border_four_way(
        r,
        lookups,
        BorderFont::Ui,
        shopx,
        shopy + 40.0,
        [pulse_text, pulse_text, pulse_text, pulse_text],
        [0.0, 0.0],
        1.0,
        label,
    );
    let item_line = lookups
        .item_font
        .as_ref()
        .map(|f| f.line_spacing)
        .unwrap_or(8.0);
    if horizontal {
        // `:3661-3684`: the label-width offset rides the FULL string
        // measure at scale 1; the `i == 0` discard reads (`:3665-3669`)
        // are dead C# (`_ =`), dropped.
        let label_w = text::ui_font_width(lookups, label, 1.0).unwrap_or(0.0);
        for i in 0..4usize {
            let num = coins_array[3 - i].min(999);
            let st = coin_state.update_savings(i, num.max(0) as u32);
            let v = [
                shopx + label_w + (24 * i) as f32 + 45.0,
                shopy + 50.0,
            ];
            draw_item_coin(
                r,
                lookups,
                [v[0], v[1] - st.coin_y_offset],
                3 - i,
                st.coin_anim_frame,
            );
            // `:3682`: the count at `(v.X - 11, v.Y + LineSpacing*0.75)`,
            // origin `(0, LineSpacing)`, scale `0.75 * stackTextScale`.
            border_four_way(
                r,
                lookups,
                BorderFont::Item,
                v[0] - 11.0,
                v[1] + item_line * 0.75,
                [1.0; 4],
                [0.0, item_line],
                0.75 * st.stack_text_scale,
                &num.to_string(),
            );
        }
        return;
    }
    // `:3685-3701`: the vertical row — the `j == 0 && count > 99` -6 px
    // text shift (`num2`), the coin at `(shopx + 11 + 24j, shopy + 75)`.
    for j in 0..4usize {
        let num2 = if j == 0 && coins_array[3 - j] > 99 { -6.0 } else { 0.0 };
        let num3 = coins_array[3 - j].min(999);
        let st = coin_state.update_savings(j, num3.max(0) as u32);
        draw_item_coin(
            r,
            lookups,
            [shopx + 11.0 + (24 * j) as f32, shopy + 75.0 - st.coin_y_offset],
            3 - j,
            st.coin_anim_frame,
        );
        border_four_way(
            r,
            lookups,
            BorderFont::Item,
            shopx + (24 * j) as f32 + num2,
            shopy + 75.0 + item_line * 0.75,
            [1.0; 4],
            [0.0, item_line],
            0.75 * st.stack_text_scale,
            &num3.to_string(),
        );
    }
}

/// `ItemSlot.DrawSavings` (`ItemSlot.cs:3704-3741`): the four banks'
/// coin totals (`Utils.CoinsCount` each, combined with the 9999999999
/// cap), the pile icons for every NON-EMPTY bank (void vault 4076 at
/// `+70,+45`, defender's forge 3813 at `+92,+45`, safe 346 at `+80,+50`,
/// piggy 87 at `+70,+60` — each `CenteredRectangle` at 0.65 scale), then
/// `DrawMoney(Lang.inter[66], …, CoinsSplit(total), fromSavings: true)`.
/// Nothing draws when the combined total is 0 (`:3717`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_savings(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    coin_state: &mut CoinSlotState,
    banks: &[&[ItemSlot]; 4],
    shopx: f32,
    shopy: f32,
    horizontal: bool,
    pulse_text: f32,
) {
    // `:3712-3716`: bank/bank2/bank3/bank4 in order.
    let counts = [
        coins_count(banks[0]),
        coins_count(banks[1]),
        coins_count(banks[2]),
        coins_count(banks[3]),
    ];
    let total = coins_combine_stacks(&counts);
    if total <= 0 {
        return;
    }
    // `:3723-3738`: the pile icons, void vault first.
    if counts[3] > 0 {
        pile_icon(r, lookups, 4076, shopx + 70.0, shopy + 45.0);
    }
    if counts[2] > 0 {
        pile_icon(r, lookups, 3813, shopx + 92.0, shopy + 45.0);
    }
    if counts[1] > 0 {
        pile_icon(r, lookups, 346, shopx + 80.0, shopy + 50.0);
    }
    if counts[0] > 0 {
        pile_icon(r, lookups, 87, shopx + 70.0, shopy + 60.0);
    }
    // `:3739`.
    draw_money(
        r,
        lookups,
        coin_state,
        &crate::lang::inter(66),
        shopx,
        shopy,
        coins_split(total),
        horizontal,
        pulse_text,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::EMPTY_SLOT;

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot { kind, count, ..EMPTY_SLOT }
    }

    /// `Utils.cs:1027-1041`: the denomination multipliers, non-coin
    /// kinds ignored.
    #[test]
    fn w4_g12_coins_count_sums_the_four_denominations() {
        let inv = [
            slot(71, 5),
            slot(72, 3),
            slot(73, 2),
            slot(74, 1),
            slot(87, 99), // the piggy item itself is not a coin
            EMPTY_SLOT,
        ];
        assert_eq!(coins_count(&inv), 5 + 300 + 20000 + 1000000);
        assert_eq!(coins_count(&[EMPTY_SLOT; 4]), 0);
    }

    /// `Utils.cs:1062-1076`: the cap fires AFTER an add and returns
    /// immediately — later counts are never read.
    #[test]
    fn w4_g12_coins_combine_caps_at_9999999999_mid_walk() {
        assert_eq!(coins_combine_stacks(&[1, 2, 3]), 6);
        assert_eq!(
            coins_combine_stacks(&[9999999999, 5]),
            9999999999,
            "the cap returns before the second count"
        );
        assert_eq!(coins_combine_stacks(&[9999999998, 1]), 9999999999);
        assert_eq!(coins_combine_stacks(&[9999999998]), 9999999998);
    }

    /// `Utils.cs:1048-1060`: the remainder walk, copper last.
    #[test]
    fn w4_g12_coins_split_walks_the_remainder_descending() {
        assert_eq!(coins_split(0), [0, 0, 0, 0]);
        assert_eq!(coins_split(1234567), [67, 45, 23, 1]);
        assert_eq!(coins_split(9999999999), [99, 99, 99, 9999]);
        // 100 copper is 1 silver, 0 copper.
        assert_eq!(coins_split(100), [0, 1, 0, 0]);
    }

    /// `Utils.cs:947-950`: all four casts truncate toward zero.
    #[test]
    fn w4_g12_centered_rect_truncates_all_four_casts() {
        // A 30x30 icon at 0.65 = 19.5: extent 19, pos (70-9.75)=60.25→60.
        assert_eq!(centered_rect(70.0, 45.0, 19.5, 19.5), (60.0, 35.0, 19.0, 19.0));
        // Negative coordinates truncate toward zero, not floor.
        assert_eq!(centered_rect(-1.2, -1.2, 1.9, 1.9), (-2.0, -2.0, 1.0, 1.0));
    }

    /// `ItemSlot.cs:3712-3739`: the bank order (bank..bank4), the icon
    /// gates per bank, the vertical DrawMoney tail with inter[66].
    #[test]
    fn w4_g12_the_savings_walk_matches_the_call_shape() {
        let src = include_str!("savings.rs");
        let body = &src[src.find("pub(crate) fn draw_savings(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        // The icon order: 4076, 3813, 346, 87 (void vault first).
        let i4076 = body.find("4076").unwrap();
        let i3813 = body.find("3813").unwrap();
        let i346 = body.find("346,").unwrap();
        let i87 = body.find("87,").unwrap();
        assert!(i4076 < i3813 && i3813 < i346 && i346 < i87);
        assert!(body.contains("counts[3] > 0"), "bank4 gates the vault icon");
        assert!(body.contains("counts[0] > 0"), "bank gates the piggy icon");
        assert!(body.contains("inter(66)"), "the Lang.inter[66] label");
        assert!(body.contains("total <= 0"), "the :3717 empty gate");
    }

    /// `CoinSlot.cs:225-228` + `:341`: UpdateSavings drives a 4-entry
    /// array with type 71+slot; the jump/text anims ride the same
    /// machine as the grid cells.
    #[test]
    fn w4_g12_update_savings_animates_the_four_rows() {
        let mut st = CoinSlotState::new();
        // First sight of 5 copper: the state latches, no jump yet.
        let d0 = st.update_savings(0, 5);
        assert_eq!(d0.coin_y_offset, 0.0);
        // A stack change arms the text anim; stepping the clocks drives
        // the jump through the trigger window.
        let _ = st.update_savings(0, 6);
        let mut jumped = false;
        for _ in 0..30 {
            st.update_slot_anims();
            let d = st.update_savings(0, 6);
            jumped |= d.coin_y_offset > 0.0;
        }
        assert!(jumped, "the savings row bounces on a stack change");
        // Row 3 is platinum (71+3 = 74).
        let d3 = st.update_savings(3, 2);
        assert_eq!(d3.coin_anim_frame, 0);
    }
}
