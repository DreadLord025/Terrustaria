//! The shop savings pile (W4-g58 D5) — `ItemSlot.DrawSavings`
//! (`UI/ItemSlot.cs:3704-3741`) + `ItemSlot.DrawMoney`'s vertical arm
//! (`:3686-3701`) + `CoinSlot.DrawItemCoin` (`UI/CoinSlot.cs:240-254`) +
//! the `Utils.CoinsCount`/`CoinsSplit`/`CoinsCombineStacks` coin math
//! (`Utils.cs:1019-1076`). The shop block calls it at `(504, invBottom)`
//! (`Main.cs:41791`); the pile shows the player's four banks' combined
//! coins as a bank-icon stack + the four coin spin strips with their
//! counts.
//!
//! The custom-currency branch (`_customCurrencyForSavings != -1`,
//! `:3707-3711`) is inert — the port has no custom currency (the DD2
//! defender-medals shop rides its own lane), so the plain coin path runs.

use tr_render::SpriteRenderer;

use super::text::{self, BorderFont};
use crate::assets::Lookups;
use crate::item::ItemSlot;
use crate::ui::coin_slot::{CoinDrawState, CoinSlotState};

/// `Utils.CoinsCount` (`Utils.cs:1019-1046`): the copper total of one
/// bank's coin stacks (71 copper ×1, 72 silver ×100, 73 gold ×10000,
/// 74 platinum ×1000000). The `ignoreSlots` params are empty at every
/// `DrawSavings` call site, so the port omits them.
pub(crate) fn coins_count(bank: &[ItemSlot]) -> i64 {
    let mut total: i64 = 0;
    for sl in bank {
        let mult: i64 = match sl.kind {
            71 => 1,
            72 => 100,
            73 => 10_000,
            74 => 1_000_000,
            _ => continue,
        };
        total += sl.count as i64 * mult;
    }
    total
}

/// `Utils.CoinsCombineStacks` (`Utils.cs:1062-1076`): the running sum of
/// the four banks' totals, clamped at 9,999,999,999 (the overflow cap —
/// the C# returns the cap the moment the running sum reaches it).
pub(crate) fn coins_combine_stacks(counts: [i64; 4]) -> i64 {
    let mut total: i64 = 0;
    for c in counts {
        total += c;
        if total >= 9_999_999_999 {
            return 9_999_999_999;
        }
    }
    total
}

/// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): the copper total split into
/// `[copper, silver, gold, platinum]` counts — the loop walks platinum
/// down to copper, each `(count - accumulated) / denomination`.
pub(crate) fn coins_split(count: i64) -> [i32; 4] {
    let mut array = [0i32; 4];
    let mut accumulated: i64 = 0;
    let mut denomination: i64 = 1_000_000;
    for slot in (0..4).rev() {
        array[slot] = ((count - accumulated) / denomination) as i32;
        accumulated += array[slot] as i64 * denomination;
        denomination /= 100;
    }
    array
}

/// `Utils.CenteredRectangle(Vector2 center, Vector2 size)`
/// (`Utils.cs:947-950`): the int-truncated top-left + size of a centred
/// rect — `(int)(center - size/2)` per axis, `(int)size` per axis. The
/// sprite draws at that top-left with the truncated size.
fn centered_rect(center: [f32; 2], size: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    (
        [
            (center[0] - size[0] / 2.0) as i32 as f32,
            (center[1] - size[1] / 2.0) as i32 as f32,
        ],
        [size[0] as i32 as f32, size[1] as i32 as f32],
    )
}

/// One bank-icon sprite (`ItemSlot.DrawSavings`, `:3725`/`:3729`/`:3733`/
/// `:3737`): `sb.Draw(texture, CenteredRectangle(center, frame.Size() *
/// 0.65f), null, Color.White)`. The four bank items (4076 Void Vault,
/// 3813 Defender's Forge, 346 Safe, 87 Piggy Bank) carry no
/// `Main.itemAnimations` row, so `GetItemDrawFrame` (`Main.cs:26898-
/// 26910`) returns the full texture frame — `frame.Size()` == the
/// texture size for all four.
fn draw_bank_icon(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    item: u16,
    center: [f32; 2],
) {
    let Some((uv, iw, ih)) = lookups.item_icon(item) else {
        return;
    };
    let (pos, size) = centered_rect(center, [iw * 0.65, ih * 0.65]);
    r.push_ui_sprite(pos, size, uv, [1.0; 4]);
}

/// `CoinSlot.DrawItemCoin` (`CoinSlot.cs:240-254`): the coin spin strip
/// `TextureAssets.Coin[coinType-71].Frame(1, 8, 0, coinFrame)` drawn
/// centred on `screenPositionForItemCenter` at `scale` (the 1024 size
/// limit never binds a 16-px coin, so `num = 1`). The tint is
/// `ContentSamples.ItemsByType[coinType].GetAlpha(Color.White)` — the
/// coin items carry no `color`/`alpha` override, so it is plain white.
fn draw_item_coin(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    center: [f32; 2],
    coin_type: u16,
    coin_frame: i32,
    scale: f32,
) {
    let kind = (coin_type - 71) as usize;
    let Some((uv, fw, fh)) = lookups.coin_strip_frame(kind, coin_frame) else {
        return;
    };
    let sw = fw * scale;
    let sh = fh * scale;
    r.push_ui_sprite([center[0] - sw * 0.5, center[1] - sh * 0.5], [sw, sh], uv, [1.0; 4]);
}

/// `ItemSlot.DrawMoney`'s vertical arm (`ItemSlot.cs:3686-3701`) with
/// `fromSavings: true`: the `Lang.inter[66]` "Savings" label at
/// `(shopx, shopy + 40)`, then the four coin cells (platinum down to
/// copper) — each `CoinSlot.UpdateSavings(j, count)` drives the spin
/// frame + the stack-text scale, the coin draws at
/// `(shopx + 11 + 24j, shopy + 75 - coinYOffset)` and the count text at
/// `(shopx + 24j + num2, shopy + 75 + LineSpacing * 0.75)` with origin
/// `(0, LineSpacing)` and scale `0.75 * stackTextScale` (`num2 = -6` when
/// the platinum count > 99, `:3689`).
fn draw_money_vertical(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    coin_slot: &mut CoinSlotState,
    label: &str,
    shopx: f32,
    shopy: f32,
    coins: [i32; 4],
    text_tint: [f32; 4],
) {
    const FOUR_WAY: [[f32; 2]; 4] = [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]];
    // `:3654` — the Savings label, white × mouseTextColor/255.
    text::draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        shopx,
        shopy + 40.0,
        1.0,
        &FOUR_WAY,
        text_tint,
        label,
    );
    let line_spacing = lookups.item_font.as_ref().map_or(8.0, |f| f.line_spacing);
    for j in 0..4usize {
        // `:3689` — the platinum count > 99 shifts its text left 6 px.
        let num2 = if j == 0 && coins[3] > 99 { -6.0 } else { 0.0 };
        // `:3690-3694` — the count clamps at 999.
        let count = coins[3 - j].min(999);
        // `:3695-3698` — the savings cell's draw state.
        let draw_state: CoinDrawState = coin_slot.update_savings(j, count as u32);
        // `:3699` — the coin spin strip.
        draw_item_coin(
            r,
            lookups,
            [shopx + 11.0 + 24.0 * j as f32, shopy + 75.0 - draw_state.coin_y_offset],
            74 - j as u16,
            draw_state.coin_anim_frame,
            1.0,
        );
        // `:3700` — the count text, origin (0, LineSpacing): the net
        // top-left is y + LineSpacing*0.75 - LineSpacing*scale.
        let scale = 0.75 * draw_state.stack_text_scale;
        let tx = shopx + 24.0 * j as f32 + num2;
        let ty = shopy + 75.0 + line_spacing * 0.75 - line_spacing * scale;
        text::draw_border_string(
            r,
            lookups,
            BorderFont::Item,
            tx,
            ty,
            scale,
            &FOUR_WAY,
            [1.0; 4],
            &count.to_string(),
        );
    }
}

/// `ItemSlot.DrawSavings` (`ItemSlot.cs:3704-3741`): the four banks'
/// combined coins → the bank-icon pile + `DrawMoney`. `banks` is
/// `Player.bank/bank2/bank3/bank4` in order; `text_tint` is
/// `Color.White * (mouseTextColor/255)` (the Savings label's tint).
pub(crate) fn draw_savings(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    coin_slot: &mut CoinSlotState,
    banks: &[&[ItemSlot]; 4],
    shopx: f32,
    shopy: f32,
    text_tint: [f32; 4],
) {
    // `:3712-3716` — the per-bank totals + the combined clamp.
    let num = coins_count(banks[0]);
    let num2 = coins_count(banks[1]);
    let num3 = coins_count(banks[2]);
    let num4 = coins_count(banks[3]);
    let total = coins_combine_stacks([num, num2, num3, num4]);
    if total <= 0 {
        return;
    }
    // `:3723-3738` — the bank-icon pile: the Void Vault (4076) when bank4
    // has coins, the Defender's Forge (3813) when bank3 does, the Safe
    // (346) when bank2 does, the Piggy Bank (87) when bank does.
    if num4 > 0 {
        draw_bank_icon(r, lookups, 4076, [shopx + 70.0, shopy + 45.0]);
    }
    if num3 > 0 {
        draw_bank_icon(r, lookups, 3813, [shopx + 92.0, shopy + 45.0]);
    }
    if num2 > 0 {
        draw_bank_icon(r, lookups, 346, [shopx + 80.0, shopy + 50.0]);
    }
    if num > 0 {
        draw_bank_icon(r, lookups, 87, [shopx + 70.0, shopy + 60.0]);
    }
    // `:3739` — DrawMoney(Lang.inter[66], …, CoinsSplit(total), fromSavings).
    draw_money_vertical(
        r,
        lookups,
        coin_slot,
        &crate::lang::inter(66),
        shopx,
        shopy,
        coins_split(total),
        text_tint,
    );
}

#[cfg(test)]
mod w4_g58_tests {
    use super::*;

    /// `Utils.CoinsCount` (`Utils.cs:1019-1046`): the copper total of the
    /// coin stacks, non-coin kinds ignored.
    #[test]
    fn the_coins_count_sums_the_denominations() {
        let slot = |kind: u16, count: u32| ItemSlot {
            kind,
            name: "",
            count,
            prefix: 0,
        };
        let bank = [slot(71, 5), slot(72, 3), slot(73, 2), slot(74, 1), slot(8, 99)];
        // 5 + 300 + 20000 + 1000000 = 1020305; the torch 8 is ignored.
        assert_eq!(coins_count(&bank), 1_020_305);
        assert_eq!(coins_count(&[]), 0);
    }

    /// `Utils.CoinsCombineStacks` (`Utils.cs:1062-1076`): the running sum
    /// clamps at 9,999,999,999 the moment it reaches the cap.
    #[test]
    fn the_coins_combine_clamps_at_the_cap() {
        assert_eq!(coins_combine_stacks([1, 2, 3, 4]), 10);
        // The cap binds mid-sum: the first two banks already exceed it.
        assert_eq!(
            coins_combine_stacks([9_999_999_999, 5, 0, 0]),
            9_999_999_999
        );
        assert_eq!(
            coins_combine_stacks([5_000_000_000, 5_000_000_000, 0, 0]),
            9_999_999_999
        );
    }

    /// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): the copper total split
    /// into [copper, silver, gold, platinum].
    #[test]
    fn the_coins_split_walks_the_denominations() {
        // 1 platinum, 2 gold, 3 silver, 4 copper = 1020304.
        assert_eq!(coins_split(1_020_304), [4, 3, 2, 1]);
        assert_eq!(coins_split(0), [0, 0, 0, 0]);
        assert_eq!(coins_split(99), [99, 0, 0, 0]);
        assert_eq!(coins_split(100), [0, 1, 0, 0]);
        // The 999-platinum cap: 999 platinum + 99 gold + 99 silver + 99 copper.
        let max = 999 * 1_000_000 + 99 * 10_000 + 99 * 100 + 99;
        assert_eq!(coins_split(max), [99, 99, 99, 999]);
    }

    /// `Utils.CenteredRectangle` (`Utils.cs:947-950`): the int-truncated
    /// top-left + size.
    #[test]
    fn the_centered_rect_truncates() {
        // A 20x20 icon at 0.65 = 13x13, centred on (574, 303):
        // top-left = (574 - 6.5, 303 - 6.5) = (567.5, 296.5) → (567, 296).
        let (pos, size) = centered_rect([574.0, 303.0], [13.0, 13.0]);
        assert_eq!(pos, [567.0, 296.0]);
        assert_eq!(size, [13.0, 13.0]);
    }

    /// `CoinSlot.UpdateSavings` (`CoinSlot.cs:225-228`): the savings cell
    /// rides the 71+slot coin type and the savings jump scale; a fresh
    /// cell absorbs its stack (the neutral spin frame 0).
    #[test]
    fn the_savings_cell_draw_state() {
        let mut st = CoinSlotState::new();
        let d = st.update_savings(0, 5);
        assert_eq!(d.coin_anim_frame, 0, "the fresh cell's spin frame");
        assert!((d.stack_text_scale - 1.0).abs() < f32::EPSILON);
        // A stack change re-arms the text pulse (the CoinEntry machine).
        let _ = st.update_savings(0, 5); // drawn
        let d2 = st.update_savings(0, 6);
        assert!(d2.stack_text_scale >= 1.0, "the pulse keys");
    }
}