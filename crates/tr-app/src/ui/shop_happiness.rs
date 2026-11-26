//! The shop happiness chrome (W4-g58's slice, ported onto this branch —
//! research/142-savings-keep-one.md) — `Main.DrawInventory`'s shop block
//! (`Main.cs:41788-41797`): the `Lang.inter[28]` "Shop" label at
//! `(504, invBottom)`, the `ItemSlot.DrawSavings` coin pile (`:41791`,
//! vertical), the `NPCHappiness` icon (the 4-frame strip whose frame rides
//! the price multiplier's bands) and the multiplier's `P0` percent label.
//! The offsets are the statics `Main.cs:67266-67269`
//! (`shopHappinessTextOffset = (26, 98)`, `shopHappinessIconOffset =
//! (12, 108)`); `invBottom = 258` (`Main.cs:6608`).
//!
//! The pile rides THIS branch's `ui::savings::draw_savings` (both arms +
//! the `_customCurrencyForSavings` medal dispatch); g58b's
//! `happiness_button`/`npc_chat` report gate is NOT part of this port
//! (it belongs to g58b's report lane — merge note in research/142).

use tr_render::SpriteRenderer;

use super::{text, UiPulse};
use crate::assets::Lookups;

/// `Main.invBottom` (`Main.cs:1093`/`:6608` — the inventory-open value).
pub(crate) const INV_BOTTOM: f32 = 258.0;
/// `Main.shopHappinessTextOffsetX/Y` (`Main.cs:67266-67267`).
pub(crate) const TEXT_OFFSET: [f32; 2] = [26.0, 98.0];
/// `Main.shopHappinessIconOffsetX/Y` (`Main.cs:67268-67269`).
pub(crate) const ICON_OFFSET: [f32; 2] = [12.0, 108.0];

/// The icon's frame index (`Main.cs:41794`): `Frame(4, 1, ...)` picks
/// `0` at ≤0.82 (`MaxHappinessAchievementPriceMultiplier`,
/// `ShopHelper.cs:14`), `1` at ≤1, `2` at ≤1.1, `3` above — the nested
/// ternary's exact band order.
pub(crate) fn icon_frame(price_adjustment: f32) -> i32 {
    if price_adjustment <= 0.82 {
        0
    } else if price_adjustment <= 1.0 {
        1
    } else if price_adjustment <= 1.1 {
        2
    } else {
        3
    }
}

/// `priceAdjustment.ToString("P0")` (`Main.cs:41797`) — .NET's percent
/// format at 0 decimals: `value × 100` rounded to the nearest integer,
/// the en-US `" %"` group separator (`NumberFormatInfo.
/// PercentPositivePattern = 0` — "n %", a space before the sign). The
/// multiplier is already cents-rounded (`ShopHelper.
/// LimitAndRoundMultiplier` `:176-181`), so the ×100 is integral in
/// practice and the midpoint convention never fires.
pub(crate) fn price_percent_text(price_adjustment: f32) -> String {
    let pct = (price_adjustment * 100.0).round() as i64;
    format!("{pct} %")
}

/// The percent label's top-left (`Main.cs:41797`):
/// `(504 + shopHappinessTextOffsetX, invBottom + shopHappinessTextOffsetY)`.
pub(crate) fn text_pos() -> [f32; 2] {
    [504.0 + TEXT_OFFSET[0], INV_BOTTOM + TEXT_OFFSET[1]]
}

/// The icon's draw anchor (`Main.cs:41795`):
/// `(504 + shopHappinessIconOffsetX, invBottom + shopHappinessIconOffsetY)`
/// — the sprite draws with `origin = rectangle2.Size() / 2f` (`:41796`),
/// so the anchor is the icon's CENTRE.
pub(crate) fn icon_pos() -> [f32; 2] {
    [504.0 + ICON_OFFSET[0], INV_BOTTOM + ICON_OFFSET[1]]
}

/// The shop block's happiness chrome (`Main.cs:41790-41797`): the "Shop"
/// label + the savings pile (`:41791`) + the icon + the percent, all
/// tinted `Color.White * (mouseTextColor / 255f)` — the border passes
/// stay `Color.Black` (`Utils.DrawBorderStringFourWay`, `Utils.cs:
/// 2464-2496`). `savings` carries the coin machine + the player's four
/// banks + the `_customCurrencyForSavings` latch; `None` skips the pile
/// (the headless/no-player draws).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    price_adjustment: f32,
    pulse: UiPulse,
    savings: Option<(
        &mut crate::ui::coin_slot::CoinSlotState,
        &[&[crate::item::ItemSlot]; 4],
        i32,
    )>,
) {
    let tint = [pulse.text; 4];
    const FOUR_WAY: [[f32; 2]; 4] = [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]];
    // `:41790` — `Lang.inter[28]` at (504, invBottom).
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Ui,
        504.0,
        INV_BOTTOM,
        1.0,
        &FOUR_WAY,
        tint,
        &crate::lang::inter(28),
    );
    // `:41791` — `ItemSlot.DrawSavings(Main.spriteBatch, 504f, invBottom)`:
    // the vertical arm (the default `horizontal: false`).
    if let Some((coin_slot, banks, currency)) = savings {
        crate::ui::savings::draw_savings(
            r,
            lookups,
            coin_slot,
            banks,
            504.0,
            INV_BOTTOM,
            false,
            pulse.text,
            currency,
        );
    }
    // `:41792-41796` — the NPCHappiness strip: `Frame(4, 1, band)`
    // (`Utils.cs:1344-1349`: `width/4` per frame), drawn at scale 1 with
    // the frame-centre origin — the top-left is `anchor − size/2`.
    if let Some(sheet) = lookups.ui_npc_happiness {
        let fw = sheet.width as f32 / 4.0;
        let fh = sheet.height as f32;
        let frame = icon_frame(price_adjustment) as f32;
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let uv = [
            (sheet.x as f32 + fw * frame) / aw,
            sheet.y as f32 / ah,
            (sheet.x as f32 + fw * (frame + 1.0)) / aw,
            (sheet.y as f32 + fh) / ah,
        ];
        let [cx, cy] = icon_pos();
        r.push_ui_sprite([cx - fw * 0.5, cy - fh * 0.5], [fw, fh], uv, [1.0; 4]);
    }
    // `:41797` — the P0 percent at the text offset.
    let [tx, ty] = text_pos();
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Ui,
        tx,
        ty,
        1.0,
        &FOUR_WAY,
        tint,
        &price_percent_text(price_adjustment),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Main.cs:41794` — the nested ternary's four bands, boundaries
    /// inclusive on the ≤ arms.
    #[test]
    fn the_icon_bands_match_the_ternary() {
        assert_eq!(icon_frame(0.75), 0);
        assert_eq!(icon_frame(0.82), 0, "≤0.82 is the love band");
        assert_eq!(icon_frame(0.83), 1);
        assert_eq!(icon_frame(0.89), 1);
        assert_eq!(icon_frame(1.0), 1, "≤1 stays band 1");
        assert_eq!(icon_frame(1.06), 2);
        assert_eq!(icon_frame(1.1), 2, "≤1.1 stays band 2");
        assert_eq!(icon_frame(1.12), 3);
        assert_eq!(icon_frame(1.5), 3, "the ceiling");
    }

    /// `ToString("P0")` — ×100, round, the en-US " %" separator.
    #[test]
    fn the_percent_text_is_p0() {
        assert_eq!(price_percent_text(0.89), "89 %");
        assert_eq!(price_percent_text(1.0), "100 %");
        assert_eq!(price_percent_text(1.5), "150 %");
        assert_eq!(price_percent_text(0.75), "75 %");
        // The multiplier is cents-rounded upstream, so ×100 is integral
        // and the .NET midpoint convention (ToEven) never fires; the
        // port's `round` matches on every reachable value.
        assert_eq!(price_percent_text(1.06), "106 %");
    }

    /// `Main.cs:67266-67269` + `:41795-41797` — the literal offsets off
    /// (504, invBottom=258).
    #[test]
    fn the_chrome_positions() {
        assert_eq!(text_pos(), [530.0, 356.0]);
        assert_eq!(icon_pos(), [516.0, 366.0]);
    }
}
