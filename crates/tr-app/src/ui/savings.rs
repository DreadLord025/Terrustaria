//! The shop/reforge savings display — `ItemSlot.DrawSavings`
//! (`ItemSlot.cs:3704-3741`) + `ItemSlot.DrawMoney` (`:3652-3702`), the
//! `Main.cs:41637` reforge-menu call (`num51 + 130, invBottom`, horizontal)
//! and the `Main.cs:41791` shop-panel call (`504, invBottom`, vertical).
//! W4-d122 fifth round.
//!
//! The C# counts the four banks' coin stacks (`Utils.CoinsCount`,
//! `Utils.cs:1019-1046`), combines them capped at 9,999,999,999
//! (`CoinsCombineStacks`, `:1062-1076`), and on a non-zero total draws the
//! bank-furniture pile (one icon per bank that holds coins) + the "Savings"
//! label + the four-denomination coin row (`CoinsSplit`, `:1048-1060`).
//!
//! Deviations (recorded, ANALYSIS "the savings draw" +
//! research/142-savings-keep-one.md): the reforge call site draws only
//! while the reforge slot holds an item (the C#'s `reforgeItem.type > 0`
//! block, `Main.cs:41584`); the shop-panel call (`:41791`) rides
//! `ui::shop_happiness::draw`. The third C# caller — `DrawHairWindow`
//! (`Main.cs:43862`, horizontal, price-gated) — has its price + position
//! core in `ui::hair_window` (the final phase); the window itself is a
//! task-seed (the 213-texture hair grid is unported). The
//! `_customCurrencyForSavings` latch (`ItemSlot.cs:308/:363/:520/:3177`) is
//! threaded in as `currency` and dispatches to the medal arm
//! (`CustomCurrencyManager.DrawSavings` `:28-62` +
//! `CustomCurrencySingleCoin.DrawSavingsMoney` `:76-101`).

use tr_render::SpriteRenderer;

use crate::assets;
use crate::item::ItemSlot;

use super::coin_slot::{draw_item_coin, CoinDrawState, CoinSlotState};
use super::text::{draw_border_string, hud_text_width, BorderFont};

/// `Utils.CoinsCount` (`Utils.cs:1019-1046`): the copper total of one
/// container's coin stacks (71/72/73/74 → ×1/×100/×10000/×1000000). No
/// ignore slots on the savings calls (`DrawSavings` passes none).
fn coins_count(slots: &[ItemSlot]) -> i64 {
    let mut num = 0i64;
    for s in slots {
        let mult: i64 = match s.kind {
            71 => 1,
            72 => 100,
            73 => 10_000,
            74 => 1_000_000,
            _ => continue,
        };
        num += s.count as i64 * mult;
    }
    num
}

/// `Utils.CoinsCombineStacks` (`Utils.cs:1062-1076`): the running sum,
/// clamped at 9,999,999,999 the moment it reaches the cap (`>=`).
fn coins_combine(counts: &[i64]) -> i64 {
    let mut num = 0i64;
    for c in counts {
        num += c;
        if num >= 9_999_999_999 {
            return 9_999_999_999;
        }
    }
    num
}

/// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): the four-denomination split,
/// platinum first (`num2 = 1000000`, `num2 /= 100` per step) — index 0
/// copper .. index 3 platinum.
pub(crate) fn coins_split(count: i64) -> [i32; 4] {
    let mut array = [0i32; 4];
    let mut num = 0i64;
    let mut num2 = 1_000_000i64;
    for i in (0..4).rev() {
        array[i] = ((count - num) / num2) as i32;
        num += array[i] as i64 * num2;
        num2 /= 100;
    }
    array
}

/// `ItemSlot.DrawSavings`'s bank counts (`ItemSlot.cs:3712-3716`): the four
/// banks in bank/bank2/bank3/bank4 order (the port's `Player.banks[0..4]`,
/// `bank.rs`'s `bank_for_chest` resolver), each counted then combined.
/// Returns `(per_bank, combined)`.
pub(crate) fn savings_counts(banks: &[&[ItemSlot]; 4]) -> ([i64; 4], i64) {
    let counts = [
        coins_count(banks[0]),
        coins_count(banks[1]),
        coins_count(banks[2]),
        coins_count(banks[3]),
    ];
    (counts, coins_combine(&counts))
}

/// The bank-furniture pile's centres + item ids — `DrawSavings`'s four
/// `sb.Draw` sites (`ItemSlot.cs:3723-3738`), in draw order: the Void Vault
/// 4076 (bank4) at (+70, +45), the Defenders Forge 3813 (bank3) at
/// (+92, +45), the Safe 346 (bank2) at (+80, +50), the Piggy Bank 87 (bank)
/// at (+70, +60). Each draws only when its bank holds coins.
pub(crate) fn pile_icons(counts: &[i64; 4], shopx: f32, shopy: f32) -> Vec<(u16, f32, f32)> {
    let mut out = Vec::new();
    if counts[3] > 0 {
        out.push((4076, shopx + 70.0, shopy + 45.0));
    }
    if counts[2] > 0 {
        out.push((3813, shopx + 92.0, shopy + 45.0));
    }
    if counts[1] > 0 {
        out.push((346, shopx + 80.0, shopy + 50.0));
    }
    if counts[0] > 0 {
        out.push((87, shopx + 70.0, shopy + 60.0));
    }
    out
}

/// One coin cell's draw position — `DrawMoney`'s horizontal arm
/// (`ItemSlot.cs:3680`): `(shopx + MeasureString(text).X + 24*i + 45,
/// shopy + 50)`; the vertical arm (`:3699`): `(shopx + 11 + 24*j,
/// shopy + 75)`. The savings jump offset subtracts from Y at the draw.
pub(crate) fn money_coin_pos(
    horizontal: bool,
    i: usize,
    shopx: f32,
    shopy: f32,
    text_w: f32,
) -> (f32, f32) {
    if horizontal {
        (shopx + text_w + 24.0 * i as f32 + 45.0, shopy + 50.0)
    } else {
        (shopx + 11.0 + 24.0 * i as f32, shopy + 75.0)
    }
}

/// The stack-count text's top-left — `DrawMoney`'s horizontal arm
/// (`ItemSlot.cs:3682`): `(vector.X - 11, vector.Y + LineSpacing*0.75)`
/// with origin `(0, LineSpacing)` and scale `0.75 * stackTextScale`; the
/// vertical arm (`:3700`): `(shopx + 24*j + num2, shopy + 75 + ls*0.75)`,
/// same origin/scale, `num2 = -6` when the platinum cell's RAW count
/// exceeds 99 (`:3689`). `DrawString` SCALES the origin
/// (`DynamicSpriteFont.InternalDrawFast`, `DynamicSpriteFont.cs:194`:
/// `position -= origin * scale`), so the net top-left is
/// `y + ls*0.75 - ls*scale` — at rest (scale 0.75) the ls terms cancel;
/// during the text pulse the count lifts (research/142, the g58b arm).
#[allow(clippy::too_many_arguments)]
pub(crate) fn money_text_pos(
    horizontal: bool,
    i: usize,
    count: i32,
    shopx: f32,
    shopy: f32,
    coin_x: f32,
    coin_y: f32,
    line_spacing: f32,
    scale: f32,
) -> (f32, f32) {
    if horizontal {
        (
            coin_x - 11.0,
            coin_y + line_spacing * 0.75 - line_spacing * scale,
        )
    } else {
        let num2 = if i == 0 && count > 99 { -6.0 } else { 0.0 };
        (
            shopx + 24.0 * i as f32 + num2,
            shopy + 75.0 + line_spacing * 0.75 - line_spacing * scale,
        )
    }
}

/// `ItemSlot.DrawMoney` (`ItemSlot.cs:3652-3702`): the "Savings" label at
/// `(shopx, shopy + 40)` (`DrawBorderStringFourWay`, MouseText, white ×
/// `mouseTextColor/255`), then the four coin cells (platinum first —
/// `coinsArray[3 - i]`, kind `74 - i`), each clamped at 999 for display
/// (`:3672-3675`/`:3691-3694`), the savings animation state feeding the
/// jump offset + the text scale (`fromSavings: true`, `:3676-3679`).
#[allow(clippy::too_many_arguments)]
fn draw_money(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    coin_slot: &mut CoinSlotState,
    text: &str,
    shopx: f32,
    shopy: f32,
    coins: &[i32; 4],
    horizontal: bool,
    text_alpha: f32,
) {
    // `:3654`: the label — four ±2 px black passes then the colour pass.
    draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        shopx,
        shopy + 40.0,
        1.0,
        &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
        [text_alpha; 4],
        text,
    );
    let text_w = hud_text_width(r, lookups, text, 1.0);
    let ls = lookups.item_font.as_ref().map(|f| f.line_spacing).unwrap_or(8.0);
    for i in 0..4 {
        // `:3671`/`:3690`: the cell reads the REVERSED array (platinum
        // first); the display count clamps at 999.
        let raw = coins[3 - i];
        let num = raw.min(999);
        // `:3676-3679`/`:3695-3698`: `UpdateSavings(i, num, out drawState)`
        // — the state machine reads the CLAMPED count.
        let d: CoinDrawState = coin_slot.update_savings(i, num as u32);
        let (cx, cy) = money_coin_pos(horizontal, i, shopx, shopy, text_w);
        // `:3681`/`:3699`: the coin draws at the position minus the jump
        // offset; kind `74 - i` → strip index `3 - i`.
        draw_item_coin(
            r,
            lookups,
            (cx, cy - d.coin_y_offset),
            3 - i,
            d.coin_anim_frame,
            1.0,
            1024.0,
            [1.0; 4],
            1.0,
        );
        let scale = 0.75 * d.stack_text_scale;
        let (tx, ty) = money_text_pos(horizontal, i, raw, shopx, shopy, cx, cy, ls, scale);
        // `:3682`/`:3700`: the count text, ItemStack, scale
        // `0.75 * stackTextScale`, four-way black border.
        draw_border_string(
            r,
            lookups,
            BorderFont::Item,
            tx,
            ty,
            scale,
            &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
            [1.0; 4],
            &num.to_string(),
        );
    }
}

/// `ItemSlot.DrawSavings` (`ItemSlot.cs:3704-3741`): the bank-furniture
/// pile + the money row, only when the combined bank total is positive
/// (`:3717`). `text_alpha` is `Main.mouseTextColor / 255` (the label's
/// `Color.White *` multiplier, `:3654`). `currency` is
/// `ItemSlot._customCurrencyForSavings` — `!= -1` routes to
/// `CustomCurrencyManager.DrawSavings` and RETURNS (`:3707-3711`); the
/// only registered currency is the Defender Medals (index 0,
/// `CustomCurrencyManager.cs:17`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_savings(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    coin_slot: &mut CoinSlotState,
    banks: &[&[ItemSlot]; 4],
    shopx: f32,
    shopy: f32,
    horizontal: bool,
    text_alpha: f32,
    currency: i32,
) {
    // `:3707-3711` — the custom-currency branch runs INSTEAD of the coin
    // arm. The single registration means any non-(-1) latch is the medal
    // wallet (`CustomCurrencyID.DefenderMedals = 0`).
    if currency != crate::custom_currency::CURRENCY_NONE {
        draw_savings_medals(r, lookups, coin_slot, banks, shopx, shopy, horizontal, text_alpha);
        return;
    }
    let (counts, total) = savings_counts(banks);
    if total <= 0 {
        return; // `:3717`'s gate — an empty savings draws nothing
    }
    // `:3719-3738`: the pile — `GetItemDrawFrame`'s full frame (none of
    // 4076/3813/346/87 registers an item animation, `Main.cs:3641-3679`),
    // `Utils.CenteredRectangle(center, frame.Size() * 0.65)` (`Utils.cs:
    // 947-950` — the (int) truncations), `Color.White`.
    for (id, cx, cy) in pile_icons(&counts, shopx, shopy) {
        if let Some((uv, w, h)) = lookups.item_icon(id) {
            let (sw, sh) = (w * 0.65, h * 0.65);
            let x = (cx - sw * 0.5) as i32 as f32;
            let y = (cy - sh * 0.5) as i32 as f32;
            r.push_ui_sprite([x, y], [sw as i32 as f32, sh as i32 as f32], uv, [1.0; 4]);
        }
    }
    // `:3739`: `DrawMoney(sb, Lang.inter[66], shopx, shopy,
    // CoinsSplit(total), horizontal, fromSavings: true)`.
    draw_money(
        r,
        lookups,
        coin_slot,
        &crate::lang::inter(66),
        shopx,
        shopy,
        &coins_split(total),
        horizontal,
        text_alpha,
    );
}

/// `CustomCurrencyManager.DrawSavings`' bank-icon pile
/// (`CustomCurrencyManager.cs:43-58`): the SAME four bank items but
/// DIFFERENT centres from the coin arm — the Void Vault 4076 (bank4) at
/// (+96, +50), the Defenders Forge 3813 (bank3) at (+80, +50), the Safe
/// 346 (bank2) ALSO at (+80, +50) (the C# draws the two overlapped,
/// `:49/:53` — kept as-is), the Piggy Bank 87 (bank) at (+70, +60).
pub(crate) fn medal_pile_icons(
    counts: &[i64; 4],
    shopx: f32,
    shopy: f32,
) -> Vec<(u16, f32, f32)> {
    let mut out = Vec::new();
    if counts[3] > 0 {
        out.push((4076, shopx + 96.0, shopy + 50.0));
    }
    if counts[2] > 0 {
        out.push((3813, shopx + 80.0, shopy + 50.0));
    }
    if counts[1] > 0 {
        out.push((346, shopx + 80.0, shopy + 50.0));
    }
    if counts[0] > 0 {
        out.push((87, shopx + 70.0, shopy + 60.0));
    }
    out
}

/// `CustomCurrencySingleCoin.DrawSavingsMoney`'s icon centre + count-text
/// net top-left (`CustomCurrencySingleCoin.cs:88-100`). The icon is the
/// STATIC `Item_3817` at `CurrencyDrawScale = 0.8` (`:14`), centred; the
/// horizontal centre is `(shopx + labelWidth + 45, shopy + 50 - jump)`
/// (`:91`) and its text `(pos.X - 11, pos.Y)` (`:93`) — the text RIDES
/// THE JUMP in this arm; the vertical centre is `(shopx + 11,
/// shopy + 75 - jump)` (`:98`) with the text at `(shopx + num2,
/// shopy + 75)` (`:99`, NO jump), `num2 = -6` iff `total > 99` (`:97`).
/// Both texts draw with origin `(0.3, 0.3)` scaled by `0.75 *
/// stackTextScale` (`DrawString` scales the origin,
/// `DynamicSpriteFont.cs:194`) → net `pos - 0.3*scale` per axis.
/// Returns `(icon_center, text_top_left)`.
pub(crate) fn medal_money_positions(
    horizontal: bool,
    total: i64,
    shopx: f32,
    shopy: f32,
    text_w: f32,
    jump: f32,
    scale: f32,
) -> ((f32, f32), (f32, f32)) {
    if horizontal {
        let pos = (shopx + text_w + 45.0, shopy + 50.0 - jump);
        (pos, (pos.0 - 11.0 - 0.3 * scale, pos.1 - 0.3 * scale))
    } else {
        let num2 = if total > 99 { -6.0 } else { 0.0 };
        let center = (shopx + 11.0, shopy + 75.0 - jump);
        let text = (shopx + num2, shopy + 75.0);
        (center, (text.0 - 0.3 * scale, text.1 - 0.3 * scale))
    }
}

/// `CustomCurrencyManager.DrawSavings` (`CustomCurrencyManager.cs:28-62`)
/// + `CustomCurrencySingleCoin.DrawSavingsMoney` (`:76-101`): the medal
/// savings display — the four banks' 3817 counts (each capped 999, the
/// combine capped 999, `count_savings`), the `> 0` gate (`:37`), the
/// medal pile (`:43-58`), the SAME Savings label (`:59`), then ONE static
/// medal icon + the total's count text driven by `CoinSlot.UpdateCustom`
/// (`:87` — no 999 display clamp; the wallet caps at 999 anyway).
#[allow(clippy::too_many_arguments)]
fn draw_savings_medals(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    coin_slot: &mut CoinSlotState,
    banks: &[&[ItemSlot]; 4],
    shopx: f32,
    shopy: f32,
    horizontal: bool,
    text_alpha: f32,
) {
    let (counts, total) =
        crate::custom_currency::count_savings([banks[0], banks[1], banks[2], banks[3]]);
    if total <= 0 {
        return; // `:37`'s gate
    }
    // `:43-58` — the pile, the same CenteredRectangle 0.65 truncations.
    for (id, cx, cy) in medal_pile_icons(&counts, shopx, shopy) {
        if let Some((uv, w, h)) = lookups.item_icon(id) {
            let (sw, sh) = (w * 0.65, h * 0.65);
            let x = (cx - sw * 0.5) as i32 as f32;
            let y = (cy - sh * 0.5) as i32 as f32;
            r.push_ui_sprite([x, y], [sw as i32 as f32, sh as i32 as f32], uv, [1.0; 4]);
        }
    }
    // `:59` — the Savings label, identical to the coin arm's `:3654`.
    let label = crate::lang::inter(66);
    draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        shopx,
        shopy + 40.0,
        1.0,
        &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
        [text_alpha; 4],
        &label,
    );
    let text_w = hud_text_width(r, lookups, &label, 1.0);
    // `DrawSavingsMoney:87` — UpdateCustom(3817, (int)total).
    let d = coin_slot.update_custom(crate::custom_currency::DEFENDER_MEDAL_ITEM, total as u32);
    let scale = 0.75 * d.stack_text_scale;
    let (center, text_pos) = medal_money_positions(
        horizontal,
        total,
        shopx,
        shopy,
        text_w,
        d.coin_y_offset,
        scale,
    );
    // `:92`/`:98` — the STATIC Item_3817 texture, centred origin, scale
    // `CurrencyDrawScale = 0.8`, Color.White, NO size clamp.
    if let Some((uv, w, h)) = lookups.item_icon(crate::custom_currency::DEFENDER_MEDAL_ITEM) {
        let (dw, dh) = (w * 0.8, h * 0.8);
        r.push_ui_sprite(
            [center.0 - dw * 0.5, center.1 - dh * 0.5],
            [dw, dh],
            uv,
            [1.0; 4],
        );
    }
    // `:93`/`:99` — the total's count text, ItemStack, four-way border.
    draw_border_string(
        r,
        lookups,
        BorderFont::Item,
        text_pos.0,
        text_pos.1,
        scale,
        &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
        [1.0; 4],
        &total.to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bank::BANK_SLOTS;
    use crate::item::EMPTY_SLOT;

    fn coin(kind: u16, count: u32) -> ItemSlot {
        ItemSlot { kind, name: "", count, prefix: 0 }
    }

    fn banks_with(coins: [(u16, u32); 4]) -> [Vec<ItemSlot>; 4] {
        std::array::from_fn(|i| {
            let mut v = vec![EMPTY_SLOT; BANK_SLOTS];
            let (k, c) = coins[i];
            if c > 0 {
                v[0] = coin(k, c);
            }
            v
        })
    }

    fn views(b: &[Vec<ItemSlot>; 4]) -> [&[ItemSlot]; 4] {
        std::array::from_fn(|i| b[i].as_slice())
    }

    /// `CoinsCount`/`CoinsCombineStacks`/`CoinsSplit` — the denomination
    /// multipliers, the 9,999,999,999 running clamp, the platinum-first
    /// split (`Utils.cs:1019-1076`).
    #[test]
    fn the_savings_arithmetic_matches_utils() {
        let b = banks_with([(71, 5), (72, 3), (73, 2), (74, 1)]);
        let (counts, total) = savings_counts(&views(&b));
        assert_eq!(counts, [5, 300, 20_000, 1_000_000]);
        assert_eq!(total, 1_020_305);
        assert_eq!(coins_split(total), [5, 3, 2, 1]);
        // The combine clamp: two near-cap banks clamp at the cap.
        assert_eq!(coins_combine(&[9_000_000_000, 2_000_000_000]), 9_999_999_999);
        assert_eq!(coins_combine(&[9_999_999_998, 1]), 9_999_999_999, "the >= trips at the cap");
        assert_eq!(coins_combine(&[1, 2, 3]), 6);
        // The split of the cap: 9999 platinum, 99 gold, 99 silver, 99 copper.
        assert_eq!(coins_split(9_999_999_999), [99, 99, 99, 9999]);
        assert_eq!(coins_split(0), [0, 0, 0, 0]);
        assert_eq!(coins_split(100), [0, 1, 0, 0]);
    }

    /// The pile's draw order + per-bank gates (`ItemSlot.cs:3723-3738`):
    /// vault → forge → safe → piggy, each only when its bank holds coins.
    #[test]
    fn the_pile_draws_one_icon_per_funded_bank_in_order() {
        let all = [1i64, 1, 1, 1];
        let icons = pile_icons(&all, 180.0, 258.0);
        assert_eq!(
            icons,
            vec![
                (4076, 250.0, 303.0),
                (3813, 272.0, 303.0),
                (346, 260.0, 308.0),
                (87, 250.0, 318.0),
            ]
        );
        // Only the piggy bank funded: one icon at (+70, +60).
        assert_eq!(pile_icons(&[5, 0, 0, 0], 0.0, 0.0), vec![(87, 70.0, 60.0)]);
        // Only the vault: the 4076 at (+70, +45).
        assert_eq!(pile_icons(&[0, 0, 0, 9], 0.0, 0.0), vec![(4076, 70.0, 45.0)]);
        assert!(pile_icons(&[0, 0, 0, 0], 0.0, 0.0).is_empty());
    }

    /// `DrawMoney`'s cell + text positions — the horizontal arm rides the
    /// label width (`Main.cs:41637`'s reforge call), the vertical arm the
    /// fixed (+11, +75) grid with the >99 platinum text nudge. The text's
    /// origin `(0, LineSpacing)` is SCALED by the draw scale
    /// (`DynamicSpriteFont.cs:194`), so at rest (scale 0.75) the ls terms
    /// cancel and the pulse lifts the text (research/142).
    #[test]
    fn the_money_positions_match_the_two_arms() {
        // Horizontal: cell i at shopx + text_w + 24i + 45, shopy + 50.
        assert_eq!(money_coin_pos(true, 0, 180.0, 258.0, 60.0), (285.0, 308.0));
        assert_eq!(money_coin_pos(true, 3, 180.0, 258.0, 60.0), (357.0, 308.0));
        // The text at rest: (cell.X - 11, cell.Y + ls*0.75 - ls*0.75) =
        // (cell.X - 11, cell.Y) — the ls terms cancel at scale 0.75.
        assert_eq!(
            money_text_pos(true, 0, 5, 180.0, 258.0, 285.0, 308.0, 20.0, 0.75),
            (274.0, 308.0)
        );
        // The pulse (stackTextScale 1.25 → scale 0.9375): the text lifts
        // ls*(0.9375 - 0.75) = 3.75 px.
        assert_eq!(
            money_text_pos(true, 0, 5, 180.0, 258.0, 285.0, 308.0, 20.0, 0.9375),
            (274.0, 304.25)
        );
        // Vertical: cell j at shopx + 11 + 24j, shopy + 75.
        assert_eq!(money_coin_pos(false, 0, 504.0, 258.0, 0.0), (515.0, 333.0));
        assert_eq!(money_coin_pos(false, 3, 504.0, 258.0, 0.0), (587.0, 333.0));
        // The platinum (>99) text nudge: -6 on the first cell only; at
        // rest the net y is shopy + 75 exactly.
        assert_eq!(
            money_text_pos(false, 0, 100, 504.0, 258.0, 515.0, 333.0, 20.0, 0.75),
            (498.0, 333.0)
        );
        assert_eq!(
            money_text_pos(false, 0, 99, 504.0, 258.0, 515.0, 333.0, 20.0, 0.75),
            (504.0, 333.0)
        );
        assert_eq!(
            money_text_pos(false, 1, 100, 504.0, 258.0, 539.0, 333.0, 20.0, 0.75),
            (528.0, 333.0),
            "the nudge is platinum-cell-only"
        );
        // The vertical pulse lifts the same way.
        assert_eq!(
            money_text_pos(false, 0, 100, 504.0, 258.0, 515.0, 333.0, 20.0, 0.9375),
            (498.0, 329.25)
        );
    }

    /// `CustomCurrencyManager.DrawSavings`' pile (`:43-58`): the medal
    /// centres DIFFER from the coin arm — the vault moves to (+96,+50),
    /// the forge to (+80,+50), and the Safe SHARES the forge's centre
    /// (the C# overlap, kept 1:1); the piggy stays (+70,+60).
    #[test]
    fn the_medal_pile_centres_differ_and_the_safe_overlaps_the_forge() {
        let all = [1i64, 1, 1, 1];
        let icons = medal_pile_icons(&all, 504.0, 258.0);
        assert_eq!(
            icons,
            vec![
                (4076, 600.0, 308.0),
                (3813, 584.0, 308.0),
                (346, 584.0, 308.0), // the overlap — C# :49/:53 as-is
                (87, 574.0, 318.0),
            ]
        );
        // The per-bank gates match the coin arm's shape.
        assert_eq!(medal_pile_icons(&[5, 0, 0, 0], 0.0, 0.0), vec![(87, 70.0, 60.0)]);
        assert_eq!(medal_pile_icons(&[0, 0, 0, 9], 0.0, 0.0), vec![(4076, 96.0, 50.0)]);
        assert!(medal_pile_icons(&[0, 0, 0, 0], 0.0, 0.0).is_empty());
    }

    /// `DrawSavingsMoney`'s positions (`CustomCurrencySingleCoin.cs:88-100`):
    /// the horizontal text RIDES the jump (the C# passes `position.Y`), the
    /// vertical text does NOT; the (0.3, 0.3) origin scales with the draw;
    /// the >99 nudge reads the TOTAL (not a split cell).
    #[test]
    fn the_medal_money_positions_match_the_two_arms() {
        // Horizontal at rest: icon centre (shopx + labelW + 45, shopy+50),
        // text net (centre.X - 11 - 0.3*scale, centre.Y - 0.3*scale).
        let (c, t) = medal_money_positions(true, 50, 180.0, 258.0, 60.0, 0.0, 0.75);
        assert_eq!(c, (285.0, 308.0));
        assert_eq!(t, (285.0 - 11.0 - 0.3 * 0.75, 308.0 - 0.3 * 0.75));
        // The jump lifts BOTH the icon and the horizontal text.
        let (c, t) = medal_money_positions(true, 50, 180.0, 258.0, 60.0, 10.0, 0.75);
        assert_eq!(c, (285.0, 298.0));
        assert_eq!(t, (285.0 - 11.0 - 0.3 * 0.75, 298.0 - 0.3 * 0.75));
        // Vertical at rest: icon (shopx+11, shopy+75-jump); the text stays
        // at shopy+75 even mid-jump; the >99 nudge is -6 on the total.
        let (c, t) = medal_money_positions(false, 100, 504.0, 258.0, 0.0, 10.0, 0.75);
        assert_eq!(c, (515.0, 323.0));
        assert_eq!(t, (504.0 - 6.0 - 0.3 * 0.75, 333.0 - 0.3 * 0.75));
        let (_, t) = medal_money_positions(false, 99, 504.0, 258.0, 0.0, 0.0, 0.75);
        assert_eq!(t, (504.0 - 0.3 * 0.75, 333.0 - 0.3 * 0.75), "99 does not nudge");
        // The pulse scale grows the origin offset (0.3 * scale).
        let (_, t) = medal_money_positions(false, 99, 504.0, 258.0, 0.0, 0.0, 0.9375);
        assert_eq!(t, (504.0 - 0.3 * 0.9375, 333.0 - 0.3 * 0.9375));
    }

    /// The savings machine feeds the draw: a stack change arms the text
    /// pulse, the jump fires in the trigger window, the clamped count is
    /// what the machine reads (`DrawMoney`'s `:3672-3679`).
    #[test]
    fn the_savings_machine_tracks_the_clamped_count() {
        let mut st = CoinSlotState::new();
        // A fresh cell absorbs its stack (DrawActive == 0) — no pulse.
        let d = st.update_savings(0, 5);
        assert_eq!(d.stack_text_scale, 1.0);
        assert_eq!(d.coin_y_offset, 0.0);
        // The display clamp: 1500 copper shows as 999 — the machine reads
        // the CLAMPED count. The change arms the pulse at the tail key
        // (TextAnimKeys[16] = 1.0); it grows as UpdateAnim drains the frame.
        let d = st.update_savings(0, 999);
        assert_eq!(d.stack_text_scale, 1.0, "the pulse starts at the tail key");
        st.update_slot_anims();
        let d2 = st.update_savings(0, 999);
        assert!(d2.stack_text_scale > 1.0, "the pulse grows after the anim step");
        // A later draw at the same clamped count re-arms nothing.
        let d3 = st.update_savings(0, 999);
        assert_eq!(d3.stack_text_scale, d2.stack_text_scale, "no re-pulse at the same clamped count");
        // The four cells are independent ranks (71 + slot).
        let _ = st.update_savings(3, 1);
        st.update_slot_anims();
        let d3 = st.update_savings(3, 1);
        assert_eq!(d3.coin_anim_frame, 0, "the platinum cell keeps its own clock");
    }
}
