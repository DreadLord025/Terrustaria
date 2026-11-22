//! W4-f37 doc-sweep — the Stylist's hair-restyle window STATE machine
//! (`Main.OpenHairWindow`/`CancelHairWindow`/`BuyHairWindow`/`DrawHairWindow`,
//! `Main.cs:43653-44138`). The window's DRAW (the HairStyleBack panel, the
//! 15-cell style grid, the HSL slider rows, the Buy/Cancel labels) is the
//! recorded M4-TOWN-17 deviation — this module ports the state transitions,
//! the price read and the force-close gate, the `npc_ai_town` precedent
//! (the machine lands unwired behind witnesses until the UI wave).
//!
//! Lane split: the scroll step (`hair_window_scroll`, `Main.cs:43726-43785`)
//! and the cell pick (`hair_cell_pick`, `:44110-44119`) are W4-g22's pure
//! fns (`main_sounds.rs`, its batch-2 census) — this module does NOT
//! duplicate them; the grid/wheel wiring calls them when the UI lands. The
//! open/cancel/buy cues (10/11/18) ride the existing `play_menu_sound` /
//! `Audio::play("Coins")` seams. `Main.UnlockedMaxHair` (`:43695-43707`) is
//! DEAD CODE in the decompilation (definition only, zero callers — the
//! window reads `AvailableHairstyles.Count`), so it stays unported.
//!
//! RNG: every transition is draw-free (the menu ids 10/11/18 draw nothing,
//! `LegacySoundPlayer.cs:750-782`); the price math is the f32-bits
//! `shop::hair_restyle_cost` (W4-f37 phase 2, the banker's /10000 round).

use crate::app::App;
use crate::npc::NpcKind;

/// `Main.Hairstyles.AvailableHairstyles` at the Stylist —
/// `HairstyleUnlocksHelper.RebuildList`'s `_isAtStylist` branch
/// (`HairstyleUnlocksHelper.cs:51-111`): the character-creation base list
/// (the 213 ids, `ui::char_creation::HAIRSTYLES`) PLUS the Plantera four
/// (162/164/163/145), the Martian ten (132..=123 descending) and the
/// Moon Lord one (133). The `UpdateUnlocks` remake gate (`:27-45`) keys on
/// the downed flags + `hairWindow`; the port rebuilds on every read (the
/// list is small and the flags change rarely — the C# caches, the port's
/// read is the same content).
pub fn stylist_hairstyles(
    downed_plant_boss: bool,
    downed_martians: bool,
    downed_moon_lord: bool,
) -> Vec<i32> {
    let mut list = crate::ui::char_creation::HAIRSTYLES.to_vec();
    // `:96-102` — the Plantera four, in the C#'s Add order.
    if downed_plant_boss {
        list.extend_from_slice(&[162, 164, 163, 145]);
    }
    // `:103-106` — the Martian ten, descending 132..=123.
    if downed_martians {
        list.extend_from_slice(&[132, 131, 130, 129, 128, 127, 126, 125, 124, 123]);
    }
    // `:107-110` — the Moon Lord one (gated on BOTH martians and moonlord).
    if downed_martians && downed_moon_lord {
        list.push(133);
    }
    list
}

/// `Main.OpenHairWindow` (`Main.cs:43653-43664`): the HSL bars reset to the
/// −1 "seed on first draw" sentinel, the inventory closes, the chat text
/// clears, the current hair style/colour latch into `oldHairStyle`/
/// `oldHairColor`, the window opens and sound 10 plays. The port's
/// `inventory_open = false` is the `Main.playerInventory = false` write;
/// the shop stays open (the C# does not close it here — the force-close
/// gate reads `npcShop != 0` and cancels, so the Stylist's shop and the
/// window are mutually exclusive by the gate, not by the open).
pub fn open(app: &mut App) {
    app.hair_h_bar = -1.0; // `:43655`
    app.hair_l_bar = -1.0; // `:43656`
    app.hair_s_bar = -1.0; // `:43657`
    app.inventory_open = false; // `:43658`
    app.npc_chat_text.clear(); // `:43659`
    if let Some(p) = app.player.as_ref() {
        app.old_hair_style = p.hair; // `:43660`
        app.old_hair_color = p.hair_color; // `:43661`
    }
    app.hair_window = true; // `:43662`
    app.hair_start = 0; // the grid resets at the open (the C# hairStart static)
    app.hair_buy_hover = false;
    app.hair_cancel_hover = false;
    app.hair_grab_slider = 0;
    crate::chest::play_menu_sound(app, 10); // `:43663`
}

/// `Main.CancelHairWindow` (`Main.cs:43666-43683`): restore the latched
/// style/colour, close the window, `SetTalkNPC(-1)` ONLY when the talked
/// NPC is the Stylist (353), and sound 11 unless `quiet`. The port's
/// `SetTalkNPC(-1)` is the `talk_npc = None` + `shop_mood = 1.0` pair (the
/// `world_sim/npcs.rs:196-201` close convention — the mood latch resets).
/// `ContentSamples.FixItemsUsingPlayerColours` (`:43681`) is the dye-item
/// colour refresh, unported (no dye-item colour cache).
pub fn cancel(app: &mut App, quiet: bool) {
    if !app.hair_window {
        return;
    }
    if let Some(p) = app.player.as_mut() {
        p.hair = app.old_hair_style; // `:43670`
        p.hair_color = app.old_hair_color; // `:43671`
    }
    app.hair_window = false; // `:43672`
    // `:43673-43676` — the Stylist-only talk close.
    if let Some(i) = app.talk_npc {
        if app.npcs.get(i).is_some_and(|n| n.kind == NpcKind::Stylist) {
            app.talk_npc = None;
            app.shop_mood = 1.0; // `SetTalkNPC(-1)` → NotInShop
        }
    }
    if !quiet {
        crate::chest::play_menu_sound(app, 11); // `:43679`
    }
}

/// `Main.BuyHairWindow` (`Main.cs:43685-43693`): sound 18, close the
/// window, `SetTalkNPC(-1)` (unconditional — the buy always ends the talk),
/// `npcChatCornerItem = 0`. The bought style/colour STAY on the player (the
/// cancel's restore is the only revert). The `NetMessage.SendData(4)`
/// (`:43691`) is the MP player-sync, unported; `FixItemsUsingPlayerColours`
/// (`:43692`) as above.
pub fn buy(app: &mut App) {
    app.audio.play("Coins"); // `:43687` — sound 18
    app.hair_window = false; // `:43688`
    app.talk_npc = None; // `:43689` — SetTalkNPC(-1)
    app.shop_mood = 1.0; // NotInShop
    // `:43690` — npcChatCornerItem = 0 (the port carries no corner item).
}

/// `Main.DrawHairWindow`'s force-close gate (`Main.cs:43711-43715`): the
/// window cancels when the chat text is non-empty, the inventory opens, a
/// chest opens, the shop opens (`npcShop != 0`), the talk ends
/// (`talkNPC == -1`) or the Guide craft menu opens. The port's collapsed
/// shop entry (`world_sim/npcs.rs`'s `try_open_town_shop_at_cursor`) opens
/// the Stylist's shop with `talk_npc = None`, so the `talkNPC == -1` and
/// `npcShop != 0` pair maps to the port's `shop_npc.is_some()` (the shop
/// open) OR `talk_npc.is_none()` (the talk ended) — the window lives only
/// while the Stylist talk is live and no shop/chest/inventory/craft is up.
/// Returns true when the gate fired (the caller skips the draw).
pub fn force_close_gate(app: &mut App) -> bool {
    let gate = !app.npc_chat_text.is_empty() // `Main.npcChatText != ""`
        || app.inventory_open // `Main.playerInventory`
        || app.open_container() != crate::bank::CHEST_CLOSED // `chest != -1`
        || app.shop_npc.is_some() // `Main.npcShop != 0`
        || app.talk_npc.is_none() // `talkNPC == -1`
        || app.crafting_open; // `Main.InGuideCraftMenu`
    if gate {
        cancel(app, false); // `:43713` — the loud cancel
    }
    gate
}

/// `Main.DrawHairWindow`'s price block (`Main.cs:43786-43799`): the
/// style-change +100000 / colour-change +20000 adds, the Discount Card
/// ×0.8 truncation, the mood multiply (truncates to int first) and the
/// banker's /10000 round ×10000. The change flags read the LATCHED
/// `oldHairStyle`/`oldHairColor` against the live player hair — the window
/// previews the change, the price is what the Buy button charges. The
/// math is `shop::hair_restyle_cost` (W4-f37 phase 2, the f32-bits
/// witnesses); this fn resolves the change flags and the mood latch.
pub fn restyle_price(app: &App) -> i64 {
    let Some(p) = app.player.as_ref() else {
        return 0;
    };
    let style_changed = p.hair != app.old_hair_style; // `:43787`
    let color_changed = p.hair_color != app.old_hair_color; // `:43791`
    crate::shop::hair_restyle_cost(
        style_changed,
        color_changed,
        app.shop_mood,
        app.discount_available,
    )
}

/// `Main.DrawHairWindow`'s Buy-label text (`Main.cs:43800-43852`): the
/// price's coin split into the `Lang.inter[15..18]` platinum/gold/silver/
/// copper names (each non-zero denomination appends `"{n} {name} "`), then
/// `GameUI.BuyWithValue` formats `"Buy ({0})"`; a zero price answers the
/// bare `GameUI.Buy`. The `< 0 → 0` clamp (`:43808-43812`) is display-only
/// (the charge at `:43978` is the unclamped `num11`). The trailing space
/// inside the parens is the C#'s `text2` build (each append ends `" "`).
pub fn buy_label(app: &App) -> String {
    let price = restyle_price(app).max(0); // `:43808-43812` display clamp
    if price == 0 {
        return crate::lang::game_ui("Buy"); // `:43851`
    }
    let mut num = price;
    let mut text2 = String::new();
    // `:43813-43847` — the four denominations, each `>=` gate then subtract.
    let platinum = num / 1_000_000;
    num -= platinum * 1_000_000;
    let gold = num / 10_000;
    num -= gold * 10_000;
    let silver = num / 100;
    num -= silver * 100;
    let copper = num;
    if platinum > 0 {
        text2 += &format!("{} {} ", platinum, crate::lang::inter(15));
    }
    if gold > 0 {
        text2 += &format!("{} {} ", gold, crate::lang::inter(16));
    }
    if silver > 0 {
        text2 += &format!("{} {} ", silver, crate::lang::inter(17));
    }
    if copper > 0 {
        text2 += &format!("{} {} ", copper, crate::lang::inter(18));
    }
    // `:43848` — `Language.GetTextValue("GameUI.BuyWithValue", text2)`.
    crate::lang::text_args("GameUI.BuyWithValue", &[text2])
}

/// `Main.DrawHairWindow`'s HSL seeding + recompose (`Main.cs:43996-44092`):
/// on the first slider frame the −1 bars seed from `rgbToHsl(hairColor)`;
/// afterwards the bars drive `hairColor = hslToRgb(h, s, l)`. The luminosity
/// bar floors at 0.15 (`num28 = 0.15f`, `:44066`): a grab stores
/// `l = t * (1 - 0.15) + 0.15` (`:44089`). The port reuses the menu's
/// `seed_hsl_bars`/`recompose_sel_color` shape (`menu/mod.rs:1026-1048`)
/// through the `ui::color` fns; the slider DRAG itself is the UI wave's
/// (the `ui::sliders::color_sliders_update` seam, `Main.cs:51102-51284`'s
/// twin — the hair window's rows are the same grab semantics).
pub fn seed_and_recompose(app: &mut App) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let sel = [p.hair_color[0], p.hair_color[1], p.hair_color[2], 255];
    // `:44001-44007` — the −1 sentinel seeds from the current colour.
    if app.hair_h_bar == -1.0 || app.hair_s_bar == -1.0 || app.hair_l_bar == -1.0 {
        let (h, s, l) = crate::ui::color::rgb_to_hsl(sel);
        app.hair_h_bar = h as f32;
        app.hair_s_bar = s as f32;
        app.hair_l_bar = l as f32;
    }
    // `:44091-44092` — `selColor = hslToRgb(hBar, sBar, lBar)`, then
    // `player.hairColor = selColor`.
    let c = crate::ui::color::hsl_to_rgb(
        app.hair_h_bar as f64,
        app.hair_s_bar as f64,
        app.hair_l_bar as f64,
        255,
    );
    if let Some(p) = app.player.as_mut() {
        p.hair_color = [c[0], c[1], c[2]];
    }
}

/// `Main.DrawHairWindow`'s Buy/Cancel click tail (`Main.cs:43974-43989`):
/// on the left press edge, the Buy hover charges `BuyItem(num11)` and a
/// successful buy routes to `BuyHairWindow`; the Cancel hover routes to
/// `CancelHairWindow`. The port's `BuyItem` is the aggregate-purse gate
/// (`self.coins >= price`, the recorded coin model) — the charge deducts
/// and the buy closes the window. Returns true when a click fired.
pub fn buy_cancel_click(app: &mut App, left_edge: bool) -> bool {
    if !left_edge {
        return false;
    }
    if app.hair_buy_hover {
        // `:43978` — `BuyItem(num11)`: the purse gate then the charge.
        let price = restyle_price(app);
        if price >= 0 && app.coins >= price as u64 {
            app.coins -= price as u64;
            buy(app); // `:43980`
            return true;
        }
        return false; // the purse refused — the window stays open
    }
    if app.hair_cancel_hover {
        cancel(app, false); // `:43986`
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn stylist_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = Player::spawn(&app.world);
        p.hair = 5;
        p.hair_color = [10, 20, 30];
        app.player = Some(p);
        let mut s = crate::npc::Npc::new(NpcKind::Stylist, 32.0, 100.0, 1, 0.0);
        s.hp = s.max_hp;
        app.npcs.push(s);
        app.talk_npc = Some(0);
        app
    }

    /// `OpenHairWindow` (`:43653-43664`): the bars reset to −1, the
    /// inventory closes, the chat clears, the style/colour latch, the
    /// window opens, sound 10.
    #[test]
    fn the_open_latches_the_hair_and_resets_the_bars() {
        let mut app = stylist_app();
        app.inventory_open = true;
        app.npc_chat_text = "hello".into();
        app.hair_h_bar = 0.5;
        let before = app.audio.played_count();
        open(&mut app);
        assert!(app.hair_window);
        assert!(!app.inventory_open);
        assert!(app.npc_chat_text.is_empty());
        assert_eq!(app.old_hair_style, 5);
        assert_eq!(app.old_hair_color, [10, 20, 30]);
        assert_eq!(app.hair_h_bar, -1.0);
        assert_eq!(app.hair_s_bar, -1.0);
        assert_eq!(app.hair_l_bar, -1.0);
        assert_eq!(app.audio.played_count(), before + 1, "sound 10");
    }

    /// `CancelHairWindow` (`:43666-43683`): the style/colour restore, the
    /// window closes, the Stylist talk ends (mood latch 1.0), sound 11.
    /// A non-Stylist talk is NOT closed by the cancel.
    #[test]
    fn the_cancel_restores_and_closes_the_stylist_talk() {
        let mut app = stylist_app();
        open(&mut app);
        // Change the hair while the window is open.
        app.player.as_mut().unwrap().hair = 9;
        app.player.as_mut().unwrap().hair_color = [99, 99, 99];
        app.shop_mood = 0.8;
        let before = app.audio.played_count();
        cancel(&mut app, false);
        assert!(!app.hair_window);
        assert_eq!(app.player.as_ref().unwrap().hair, 5, "restored");
        assert_eq!(app.player.as_ref().unwrap().hair_color, [10, 20, 30]);
        assert_eq!(app.talk_npc, None, "the Stylist talk ends");
        assert_eq!(app.shop_mood, 1.0, "NotInShop");
        assert_eq!(app.audio.played_count(), before + 1, "sound 11");
        // A quiet cancel plays nothing.
        let mut app2 = stylist_app();
        open(&mut app2);
        let before2 = app2.audio.played_count();
        cancel(&mut app2, true);
        assert_eq!(app2.audio.played_count(), before2, "quiet");
        // A non-Stylist talk survives the cancel.
        let mut app3 = stylist_app();
        app3.npcs[0].kind = NpcKind::Guide;
        open(&mut app3);
        cancel(&mut app3, false);
        assert_eq!(app3.talk_npc, Some(0), "the Guide talk stays");
    }

    /// `BuyHairWindow` (`:43685-43693`): sound 18, the window closes, the
    /// talk ends unconditionally, the bought style STAYS.
    #[test]
    fn the_buy_keeps_the_style_and_ends_the_talk() {
        let mut app = stylist_app();
        open(&mut app);
        app.player.as_mut().unwrap().hair = 9;
        app.player.as_mut().unwrap().hair_color = [99, 99, 99];
        let before = app.audio.played_count();
        buy(&mut app);
        assert!(!app.hair_window);
        assert_eq!(app.talk_npc, None);
        assert_eq!(app.shop_mood, 1.0);
        assert_eq!(app.player.as_ref().unwrap().hair, 9, "the bought style stays");
        assert_eq!(app.player.as_ref().unwrap().hair_color, [99, 99, 99]);
        assert_eq!(app.audio.played_count(), before + 1, "sound 18");
    }

    /// The force-close gate (`:43711-43715`): each of the six conditions
    /// cancels the window; a live Stylist talk with no overlay keeps it.
    #[test]
    fn the_force_close_gate_cancels_on_each_condition() {
        // The live state: the window stays.
        let mut app = stylist_app();
        open(&mut app);
        assert!(!force_close_gate(&mut app));
        assert!(app.hair_window);
        // Each condition fires the gate.
        for cond in 0..6 {
            let mut app = stylist_app();
            open(&mut app);
            match cond {
                0 => app.npc_chat_text = "x".into(),
                1 => app.inventory_open = true,
                2 => app.player.as_mut().unwrap().chest = 0,
                3 => app.shop_npc = Some(0),
                4 => app.talk_npc = None,
                _ => app.crafting_open = true,
            }
            assert!(force_close_gate(&mut app), "cond {cond} fires");
            assert!(!app.hair_window, "cond {cond} cancels");
        }
    }

    /// The price block (`:43786-43799`): the style/colour change flags read
    /// the latch, the mood multiply and the banker's round. The f32-bits
    /// vectors are `shop::hair_restyle_cost`'s own witnesses; this pins the
    /// flag resolution + the latch read.
    #[test]
    fn the_price_reads_the_change_flags_and_the_mood_latch() {
        let mut app = stylist_app();
        open(&mut app);
        // No change: 0.
        assert_eq!(restyle_price(&app), 0);
        // Style change only: 100000 at mood 1.0.
        app.player.as_mut().unwrap().hair = 9;
        assert_eq!(restyle_price(&app), 100000);
        // Colour change too: 120000.
        app.player.as_mut().unwrap().hair_color = [99, 99, 99];
        assert_eq!(restyle_price(&app), 120000);
        // The mood latch scales: 0.95 → 110000 (int(120000*0.95)=114000,
        // 114000/10000=11.4 → banker's 11 → 110000).
        app.shop_mood = 0.95;
        assert_eq!(restyle_price(&app), 110000);
        // The Discount Card: int(120000*0.8)=96000, int(96000*0.95)=91200,
        // 91200/10000=9.12 → banker's 9 → 90000.
        app.discount_available = true;
        assert_eq!(restyle_price(&app), 90000);
    }

    /// The Buy label (`:43800-43852`): the coin split + the BuyWithValue
    /// format, the zero price answers the bare Buy.
    #[test]
    fn the_buy_label_splits_the_coins_and_formats() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = stylist_app();
        open(&mut app);
        // Zero: the bare Buy.
        assert_eq!(buy_label(&app), "Buy");
        // 120000 = 12 gold: "Buy (12 gold )".
        app.player.as_mut().unwrap().hair = 9;
        app.player.as_mut().unwrap().hair_color = [99, 99, 99];
        assert_eq!(buy_label(&app), "Buy (12 gold )");
        // 100000 = 10 gold: "Buy (10 gold )".
        app.player.as_mut().unwrap().hair_color = [10, 20, 30];
        assert_eq!(buy_label(&app), "Buy (10 gold )");
    }

    /// The HSL seed/recompose (`:44001-44092`): the −1 bars seed from the
    /// current colour, the recompose writes the hair tint back.
    #[test]
    fn the_hsl_seed_and_recompose_round_trips() {
        let mut app = stylist_app();
        open(&mut app);
        // The bars are −1: the seed reads the current colour.
        seed_and_recompose(&mut app);
        assert!(app.hair_h_bar >= 0.0);
        // The recompose writes a colour close to the original (the HSL
        // round-trip is byte-exact for the in-gamut values).
        let c = app.player.as_ref().unwrap().hair_color;
        let (h, s, l) = crate::ui::color::rgb_to_hsl([c[0], c[1], c[2], 255]);
        assert!((h as f32 - app.hair_h_bar).abs() < 1e-3);
        assert!((s as f32 - app.hair_s_bar).abs() < 1e-3);
        assert!((l as f32 - app.hair_l_bar).abs() < 1e-3);
    }

    /// The Buy/Cancel click tail (`:43974-43989`): the Buy hover charges
    /// the purse and closes; a short purse refuses; the Cancel hover
    /// restores.
    #[test]
    fn the_buy_cancel_click_charges_and_closes() {
        let mut app = stylist_app();
        open(&mut app);
        app.player.as_mut().unwrap().hair = 9; // 100000 at mood 1.0
        app.coins = 100000;
        app.hair_buy_hover = true;
        assert!(buy_cancel_click(&mut app, true));
        assert!(!app.hair_window);
        assert_eq!(app.coins, 0);
        assert_eq!(app.player.as_ref().unwrap().hair, 9, "bought");
        // A short purse refuses, the window stays.
        let mut app2 = stylist_app();
        open(&mut app2);
        app2.player.as_mut().unwrap().hair = 9;
        app2.coins = 99999;
        app2.hair_buy_hover = true;
        assert!(!buy_cancel_click(&mut app2, true));
        assert!(app2.hair_window);
        assert_eq!(app2.coins, 99999);
        // The Cancel hover restores.
        let mut app3 = stylist_app();
        open(&mut app3);
        app3.player.as_mut().unwrap().hair = 9;
        app3.hair_cancel_hover = true;
        assert!(buy_cancel_click(&mut app3, true));
        assert!(!app3.hair_window);
        assert_eq!(app3.player.as_ref().unwrap().hair, 5, "restored");
    }

    /// The stylist list (`HairstyleUnlocksHelper.cs:51-111`): the base 213
    /// plus the Plantera four, the Martian ten, the Moon Lord one.
    #[test]
    fn the_stylist_list_grows_with_the_boss_flags() {
        let base = stylist_hairstyles(false, false, false);
        assert_eq!(base.len(), 213);
        assert_eq!(base, crate::ui::char_creation::HAIRSTYLES.to_vec());
        let plant = stylist_hairstyles(true, false, false);
        assert_eq!(plant.len(), 217);
        assert_eq!(&plant[213..], &[162, 164, 163, 145]);
        let mart = stylist_hairstyles(false, true, false);
        assert_eq!(mart.len(), 223);
        assert_eq!(&mart[213..], &[132, 131, 130, 129, 128, 127, 126, 125, 124, 123]);
        // The Moon Lord one needs BOTH martians and moonlord.
        let moon_only = stylist_hairstyles(false, false, true);
        assert_eq!(moon_only.len(), 213, "moonlord alone adds nothing");
        let all = stylist_hairstyles(true, true, true);
        assert_eq!(all.len(), 228);
        assert_eq!(all[227], 133);
    }
}
