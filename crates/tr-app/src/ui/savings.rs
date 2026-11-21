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
//! Deviations (recorded, ANALYSIS "the savings draw"): the
//! `_customCurrencyForSavings` hover latch (`ItemSlot.cs:3177`'s
//! `MouseHover` write, `:3707-3711`'s branch) has no port consumer — the
//! Tavernkeep medal-savings arm (`CustomCurrencyManager.DrawSavings`,
//! `:28-62`) stays the chrome follow-up, so this port always draws the
//! coin arm. The reforge call site draws only while the reforge slot holds
//! an item (the C#'s `reforgeItem.type > 0` block, `Main.cs:41584`); the
//! shop-panel call (`:41791`) and its "Shop" label / happiness icon stay
//! with the shop-chrome pass.

use tr_render::SpriteRenderer;

use crate::assets;
use crate::item::ItemSlot;

use super::coin_slot::{CoinDrawState, CoinSlotState};
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
fn coins_split(count: i64) -> [i32; 4] {
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
/// (`ItemSlot.cs:3682`): `(vector.X - 11, vector.Y + LineSpacing*0.75)` with
/// origin `(0, LineSpacing)` → net `(vector.X - 11, vector.Y - 0.25*ls)`;
/// the vertical arm (`:3700`): `(shopx + 24*j + num2, shopy + 75 + ls*0.75)`
/// with the same origin → net `(shopx + 24*j + num2, shopy + 75 - 0.25*ls)`,
/// `num2 = -6` when the platinum cell's count exceeds 99 (`:3689`).
pub(crate) fn money_text_pos(
    horizontal: bool,
    i: usize,
    count: i32,
    shopx: f32,
    shopy: f32,
    coin_x: f32,
    coin_y: f32,
    line_spacing: f32,
) -> (f32, f32) {
    if horizontal {
        (coin_x - 11.0, coin_y - 0.25 * line_spacing)
    } else {
        let num2 = if i == 0 && count > 99 { -6.0 } else { 0.0 };
        (shopx + 24.0 * i as f32 + num2, shopy + 75.0 - 0.25 * line_spacing)
    }
}

/// `CoinSlot.DrawItemCoin` (`CoinSlot.cs:240-254`): the coin strip's frame
/// centred on `center`, the longest side clamped to `size_limit` before the
/// scale, tinted by `GetAlpha(itemColor)` (coins 71-74 hit the default arm
/// with `alpha = 0` — the tint passes through, `Item.cs:49324-49340`).
fn draw_item_coin(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    center: (f32, f32),
    coin_kind: usize,
    frame: i32,
    scale: f32,
    size_limit: f32,
) {
    let Some((uv, w, h)) = lookups.coin_anim_uv(coin_kind, frame) else {
        return;
    };
    // `:247-250`: the clamp picks the LONGER side's ratio.
    let num = if w > size_limit || h > size_limit {
        if w <= h { size_limit / h } else { size_limit / w }
    } else {
        1.0
    };
    let s = scale * num;
    let (dw, dh) = (w * s, h * s);
    r.push_ui_sprite(
        [center.0 - dw * 0.5, center.1 - dh * 0.5],
        [dw, dh],
        uv,
        [1.0; 4],
    );
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
        );
        let (tx, ty) = money_text_pos(horizontal, i, raw, shopx, shopy, cx, cy, ls);
        // `:3682`/`:3700`: the count text, ItemStack, scale
        // `0.75 * stackTextScale`, four-way black border.
        draw_border_string(
            r,
            lookups,
            BorderFont::Item,
            tx,
            ty,
            0.75 * d.stack_text_scale,
            &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
            [1.0; 4],
            &num.to_string(),
        );
    }
}

/// `ItemSlot.DrawSavings` (`ItemSlot.cs:3704-3741`): the bank-furniture
/// pile + the money row, only when the combined bank total is positive
/// (`:3717`). `text_alpha` is `Main.mouseTextColor / 255` (the label's
/// `Color.White *` multiplier, `:3654`).
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
) {
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
    /// fixed (+11, +75) grid with the >99 platinum text nudge.
    #[test]
    fn the_money_positions_match_the_two_arms() {
        // Horizontal: cell i at shopx + text_w + 24i + 45, shopy + 50.
        assert_eq!(money_coin_pos(true, 0, 180.0, 258.0, 60.0), (285.0, 308.0));
        assert_eq!(money_coin_pos(true, 3, 180.0, 258.0, 60.0), (357.0, 308.0));
        // The text: (cell.X - 11, cell.Y - 0.25*ls).
        assert_eq!(
            money_text_pos(true, 0, 5, 180.0, 258.0, 285.0, 308.0, 20.0),
            (274.0, 303.0)
        );
        // Vertical: cell j at shopx + 11 + 24j, shopy + 75.
        assert_eq!(money_coin_pos(false, 0, 504.0, 258.0, 0.0), (515.0, 333.0));
        assert_eq!(money_coin_pos(false, 3, 504.0, 258.0, 0.0), (587.0, 333.0));
        // The platinum (>99) text nudge: -6 on the first cell only.
        assert_eq!(
            money_text_pos(false, 0, 100, 504.0, 258.0, 515.0, 333.0, 20.0),
            (498.0, 328.0)
        );
        assert_eq!(
            money_text_pos(false, 0, 99, 504.0, 258.0, 515.0, 333.0, 20.0),
            (504.0, 328.0)
        );
        assert_eq!(
            money_text_pos(false, 1, 100, 504.0, 258.0, 539.0, 333.0, 20.0),
            (528.0, 328.0),
            "the nudge is platinum-cell-only"
        );
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
