//! The Stylist hair window's PRICE core — `Main.DrawHairWindow`
//! (`Main.cs:43786-43862`): the hairstyle-change price arithmetic, the
//! four-denomination coin text and the savings-draw position. W4-d122 final
//! phase (the third savings caller, research/142 §caller-3).
//!
//! The full hair window is NOT self-contained — the hair-style grid needs
//! all 213 `PlayerHair` textures (the port decodes only `Player_Hair_1`, a
//! recorded char-creation deviation), `Main.LoadHair` (`Main.cs:59146`),
//! `Player.GetHairDrawOffset` (`Player.cs:4549`), the in-game
//! `HairstyleUnlocksHelper.UpdateUnlocks` list (`:19-113`) and the Stylist
//! chat trigger (`NPCInteractions.cs:425-441`). Those ride the hair-system
//! task (the task-seed in research/142). This module ports the SELF-CONTAINED
//! price + savings-apply core the window consumes, so the third
//! `ItemSlot.DrawSavings` caller (`Main.cs:43862`, horizontal, price-gated)
//! has its arithmetic + position ready when the window lands.

use super::savings::coins_split;
use crate::lang;
use crate::shop::round_half_to_even;

/// `Main.DrawHairWindow`'s price (`Main.cs:43786-43799`): a hairstyle change
/// costs 100000 copper (10 gold, `:43787-43790`), a colour change 20000 (2
/// gold, `:43791-43794`); the Discount Card takes 20% off (`:43795-43798`,
/// `(int)(num11 * 0.8f)` truncation); then the happiness adjustment scales
/// and the result rounds to whole gold (`:43799`,
/// `(int)Math.Round((float)(int)((float)num11 * PriceAdjustment) / 10000f) *
/// 10000` — the inner `(int)` truncates the scaled copper, the `/10000`
/// float-divides, `Math.Round` is banker's, the outer `*10000` restores
/// copper). `price_adjustment` is the TALK-OPEN latch
/// (`Player.currentShoppingSettings.PriceAdjustment`, g58's census — the
/// hair price reads the latch like the reforge/nurse, NOT a fresh mood).
pub(crate) fn hair_price(
    style_changed: bool,
    color_changed: bool,
    discount_available: bool,
    price_adjustment: f32,
) -> i64 {
    let mut num11: i64 = 0;
    if style_changed {
        num11 += 100000; // `:43789`
    }
    if color_changed {
        num11 += 20000; // `:43793`
    }
    if discount_available {
        num11 = (num11 as f32 * 0.8) as i64; // `:43797` — the (int) truncation
    }
    // `:43799`: the inner (int) truncates the adjusted copper, /10000f
    // float-divides, Math.Round bankers, *10000 restores copper.
    let scaled = (num11 as f32 * price_adjustment) as i32;
    (round_half_to_even(scaled as f32 / 10000.0) as i64) * 10000
}

/// `Main.DrawHairWindow`'s coin text (`Main.cs:43800-43852`): the price
/// split into platinum/gold/silver/copper (`CoinsSplit`), each non-zero
/// denomination appended as `"<count> <name> "` (`Lang.inter[15..18]`,
/// `:43832-43847`), then `Language.GetTextValue("GameUI.BuyWithValue", text2)`
/// (`:43848`); a zero price reads `GameUI.Buy` (`:43849-43852`). The price
/// is clamped at 0 first (`:43808-43812`).
pub(crate) fn hair_price_text(price: i64) -> String {
    let num11 = price.max(0);
    if num11 == 0 {
        return lang::text("GameUI.Buy");
    }
    let split = coins_split(num11); // [copper, silver, gold, platinum]
    let mut text2 = String::new();
    // The C# walks platinum → copper (`num12`/`num13`/`num14`/`num15`).
    for (idx, inter) in [(3usize, 15usize), (2, 16), (1, 17), (0, 18)] {
        if split[idx] > 0 {
            text2.push_str(&format!("{} {} ", split[idx], lang::inter(inter)));
        }
    }
    lang::text_args("GameUI.BuyWithValue", &[text2])
}

/// `Main.DrawHairWindow`'s savings-draw position (`Main.cs:43857-43862`):
/// `num18 = num2 + 18`, `num19 = num + 156`, and `DrawSavings(num18,
/// num19 - 70, horizontal: true)` gated on `num11 > 0`. With
/// `num = screenHeight/2 + 60` (`:43718`) and `num2 = screenWidth/2 -
/// HairStyleBack.Width()/2` (`:43719`), the call is
/// `(screenWidth/2 - backW/2 + 18, screenHeight/2 + 146)`. Returns the
/// `(x, y)` the horizontal `draw_savings` call reads (research/142's
/// caller-3 row).
pub(crate) fn hair_savings_pos(screen_w: f32, screen_h: f32, back_w: f32) -> (f32, f32) {
    let num = screen_h / 2.0 + 60.0;
    let num2 = screen_w / 2.0 - back_w / 2.0;
    let num18 = num2 + 18.0;
    let num19 = num + 156.0;
    (num18, num19 - 70.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The price ladder (`Main.cs:43786-43799`): style 10g, colour 2g, both
    /// 12g; the Discount Card ×0.8; the adjustment rounds to whole gold.
    #[test]
    fn the_hair_price_matches_the_csharp_ladder() {
        // No change → 0 (the "Buy" free button).
        assert_eq!(hair_price(false, false, false, 1.0), 0);
        // Style only → 100000 (10 gold); colour only → 20000 (2 gold).
        assert_eq!(hair_price(true, false, false, 1.0), 100000);
        assert_eq!(hair_price(false, true, false, 1.0), 20000);
        // Both → 120000 (12 gold).
        assert_eq!(hair_price(true, true, false, 1.0), 120000);
        // The Discount Card: 120000 * 0.8 = 96000 → rounds to 100000 (10g).
        assert_eq!(hair_price(true, true, true, 1.0), 100000);
        // Style + discount: 100000 * 0.8 = 80000 → 80000 (8g, exact).
        assert_eq!(hair_price(true, false, true, 1.0), 80000);
        // The happiness latch: 120000 * 1.5 = 180000 → 180000 (18g).
        assert_eq!(hair_price(true, true, false, 1.5), 180000);
        // A fractional adjustment rounds to whole gold: 100000 * 1.234 =
        // 123400 → /10000 = 12.34 → round 12 → 120000.
        assert_eq!(hair_price(true, false, false, 1.234), 120000);
        // Banker's rounding at the half-gold boundary: 100000 * 1.25 =
        // 125000 → 12.5 → round-half-to-even 12 → 120000 (not 130000).
        assert_eq!(hair_price(true, false, false, 1.25), 120000);
        // 100000 * 1.35 = 135000 → 13.5 → round-half-to-even 14 → 140000.
        assert_eq!(hair_price(true, false, false, 1.35), 140000);
    }

    /// The savings position (`Main.cs:43857-43862`): at 1600x900 with a
    /// 300-wide HairStyleBack, num2 = 800 - 150 = 650, num18 = 668; num =
    /// 450 + 60 = 510, num19 = 666, the call y = 666 - 70 = 596.
    #[test]
    fn the_hair_savings_position_matches_the_call_site() {
        let (x, y) = hair_savings_pos(1600.0, 900.0, 300.0);
        assert_eq!((x, y), (668.0, 596.0));
        // The position rides the screen centre + the back width.
        let (x2, y2) = hair_savings_pos(1280.0, 720.0, 300.0);
        assert_eq!((x2, y2), (508.0, 506.0));
    }

    /// The coin text (`Main.cs:43800-43852`): the platinum-first
    /// denomination walk + the BuyWithValue/Buy keys. Needs the language
    /// manager (the process-global test lock + init, the chat.rs pattern).
    #[test]
    fn the_hair_price_text_walks_the_denominations() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        // A zero price reads the free "Buy" button.
        assert_eq!(hair_price_text(0), "Buy");
        // 120000 = 12 gold → "Buy (12 gold )" (the C#'s trailing space).
        assert_eq!(hair_price_text(120000), "Buy (12 gold )");
        // 100000 = 10 gold.
        assert_eq!(hair_price_text(100000), "Buy (10 gold )");
        // 1234567 = 1 platinum + 23 gold + 45 silver + 67 copper.
        assert_eq!(
            hair_price_text(1234567),
            "Buy (1 platinum 23 gold 45 silver 67 copper )"
        );
        // 20000 = 2 gold (the colour-only price).
        assert_eq!(hair_price_text(20000), "Buy (2 gold )");
    }
}
