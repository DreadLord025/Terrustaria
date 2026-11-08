//! In-game HUD: hotbar / open inventory, selected-item name, the item carried on the
//! cursor, hearts and breath, and the boss bar.

pub mod bars;
pub(crate) mod bestiary;
pub(crate) mod emotes;
pub mod framework;
pub mod cards;
pub mod char_creation;
pub mod char_select;
pub(crate) mod chest;
pub mod chrome;
pub mod color;
pub mod crafting;
pub mod cursor;
pub mod inventory;
pub mod keyboard;
pub mod minimap;
pub mod panel;
pub mod pointer;
pub mod resources;
pub mod save_list;
pub mod sliders;
pub mod select_hit;
pub mod slot;
pub mod text;
pub mod tooltip;
pub mod uistate;
pub mod world_creation;
pub mod world_select;

use glam::Vec2;
use tr_render::{SheetRect, SpriteRenderer};

use crate::assets;
use crate::inv;
use crate::item::{HOTBAR, INV, ItemSlot};
use crate::player::Player;

/// The shared rect hit-test: XNA `Rectangle.Contains(Point)` as the UI reads it
/// (`UIElement.cs:354-357`'s `ContainsPoint`, `UIColoredSlider.cs:110`,
/// `IngameOptions.cs:1557`): inclusive left/top edges, exclusive right/bottom.
/// The one copy behind every screen's click table and hover test — the former
/// per-screen `in_rect` / `rect_hover` / `contains` / `inside` duplicates.
///
/// Two source sites differ and keep their own comparisons: the HSL rows'
/// open-on-every-edge test is [`in_rect_strict`], and the inventory grid's
/// `Main.cs:41446` comparison is inclusive on the right/bottom too
/// (`inventory::hit_test`).
pub(crate) fn in_rect(mouse: Vec2, rect: [f32; 4]) -> bool {
    mouse.x >= rect[0]
        && mouse.x < rect[0] + rect[2]
        && mouse.y >= rect[1]
        && mouse.y < rect[1] + rect[3]
}

/// The strict four-side variant of [`in_rect`] — open on every edge — the HSL
/// colour rows' hit test (`Main.cs:51142` and the row twins: `mouseX > num53 -
/// 4 && mouseX < num53 + Hue.Width() + 4 && ...`).
pub(crate) fn in_rect_strict(mouse: Vec2, rect: [f32; 4]) -> bool {
    mouse.x > rect[0]
        && mouse.x < rect[0] + rect[2]
        && mouse.y > rect[1]
        && mouse.y < rect[1] + rect[3]
}

/// One atlas sprite stretched over `dst` from the whole `sheet` — the shared
/// `push_ui_sprite` over the atlas UVs (the former per-screen `push_whole`
/// copies of `char_select` / `world_select` / `char_creation`).
pub(crate) fn push_whole(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &SheetRect,
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let uv = [
        sheet.x as f32 / aw,
        sheet.y as f32 / ah,
        (sheet.x + sheet.width) as f32 / aw,
        (sheet.y + sheet.height) as f32 / ah,
    ];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

/// The hover/idle colour pick every UI colour table shares (the
/// `MouseOver`/`MouseOut` pairs): one of the two pre-tint byte colours by the
/// hover flag. `char_select`/`world_select`'s card pairs and the keyboard's
/// key / toggle / submit tables select both their background and border
/// through it.
pub(crate) fn hover_pair(idle: [u8; 4], hovered: [u8; 4], on: bool) -> [u8; 4] {
    if on { hovered } else { idle }
}

/// Per-frame animated UI values shared by the HUD passes: `Main.mouseTextColor`
/// (which multiplies every interface label/colour) and `Main.cursorScale` (the
/// heart/mana pulse, `Main.cs:52491`).
#[derive(Clone, Copy, Debug)]
pub struct UiPulse {
    /// `Main.mouseTextColor / 255` (`Main.cs:18216`; oscillates `190/255..=1.0`).
    pub text: f32,
    /// `Main.cursorScale` = `cursorAlpha * 0.3 + 0.7 + 0.1` (`Main.cs:52491`).
    pub cursor_scale: f32,
}

impl Default for UiPulse {
    fn default() -> Self {
        Self {
            text: 1.0,
            cursor_scale: 1.0,
        }
    }
}

/// Vanilla HUD layout: hotbar top-left (or the open 10x5 inventory grid), hearts
/// top-right, breath bubbles above the player and the boss bar at the bottom centre
/// (`GUIHotbarDrawInner`, `Main.DrawInventory`, `ClassicPlayerResourcesDisplaySet`,
/// `BigProgressBarHelper`). `m_h` is `Main.mH`
/// (`Main.DrawInterface_16_MapOrMinimap`, `Main.cs:45771-45798`) — `256` while
/// the minimap HUD is live, `0` otherwise; it pushes the equipment block down.
/// `map_style` is `Main.mapStyle` (`Main.cs:66750`), driving the map-style icon
/// row's active tint (`Main.cs:41083-41086`).
#[allow(clippy::too_many_arguments)]
/// The death screen's text block — `Main.DrawDeathText`'s shape
/// (`Main.cs:45324-45341`): the death broadcast (the big `DeathText` font,
/// centered 60 px above the screen's midpoint) and the softcore
/// "Dropped coins: X" line 50 px lower (the small `MouseText` font). The
/// alpha rides the death-screen fade — the port draws at the fixed
/// full-bright (the respawn timer's fade lands with the death-screen pass).
pub(crate) fn draw_death_screen(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    death_text: &str,
    lost_coins: u64,
    lost_coin_string: &str,
) {
    let color = [1.0, 1.0, 1.0, 1.0];
    text::draw_death_string(r, lookups, screen.x, screen.y / 2.0 - 60.0, color, death_text);
    if lost_coins > 0 {
        let line = format!("Dropped coins: {lost_coin_string}");
        text::draw_dropped_coins(r, lookups, screen.x, screen.y / 2.0 - 10.0, color, &line);
    }
}

/// The invasion progress bar — `Main.DrawInvasionProgress`'s non-wave arm
/// (`Main.cs:47807-47830`): the 200x45 `DrawInvBG` panel at
/// `(screenWidth - 120, screenHeight - 40)` (the centre-anchored rect), the
/// `ColorBar` track, the yellow fill (255,241,51), the orange edge
/// (255,165,0,127), the black rest, and the `{label}` + `{percent}%` text
/// through `DrawBorderString`. The alpha ramps 0.05 per tick toward the
/// display timer; the icon's texture row is the group's Extra sheet (the
/// port draws the label only — the Extra sheets land with the icon row).
pub(crate) fn draw_invasion_progress(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    progress: i32,
    progress_max: i32,
    progress_icon: i32,
    alpha: f32,
) {
    // The label: `Lang.inter[88]` (the goblins) / `[85]` (the pirates) /
    // `[86]` (the frost) — the en literals stand in.
    let label = match progress_icon {
        4 => "Goblin Army",
        5 => "Pirate Invasion",
        6 => "Frost Legion",
        _ => "Invasion",
    };
    let num = 0.5 + alpha * 0.5;
    let num7 = (200.0 * num) as i32;
    let num8 = (45.0 * num) as i32;
    // The panel centre: `(screenWidth - 120, screenHeight - 40)`.
    let cx = screen.x - 120.0;
    let cy = screen.y - 40.0;
    let px = cx - num7 as f32 / 2.0;
    let py = cy - num8 as f32 / 2.0;
    // `Utils.DrawInvBG`'s fill (the (63,65,151,255) * 0.785 backdrop).
    r.push_fx_rect(
        [px, py],
        [num7 as f32, num8 as f32],
        [
            63.0 / 255.0 * 0.785,
            65.0 / 255.0 * 0.785,
            151.0 / 255.0 * 0.785,
            255.0 / 255.0 * 0.785,
        ],
    );
    if progress_max != 0 {
        let frac = (progress as f32 / progress_max as f32).clamp(0.0, 1.0);
        let num11 = 169.0 * num;
        let num12 = 8.0 * num;
        // The `ColorBar` track.
        r.push_fx_rect(
            [cx - num11 / 2.0, cy - num12 / 2.0],
            [num11, num12],
            [1.0, 1.0, 1.0, alpha],
        );
        let fill_x = cx - num11 / 2.0 + (frac - 0.5) * num11;
        let fill_w = num11 * frac;
        // The yellow fill.
        r.push_fx_rect(
            [fill_x, cy - num12 / 2.0],
            [fill_w, num12],
            [1.0, 241.0 / 255.0, 51.0 / 255.0, alpha],
        );
        // The orange edge (the 2-px strip).
        r.push_fx_rect(
            [fill_x, cy - num12 / 2.0],
            [2.0, num12],
            [1.0, 165.0 / 255.0, 0.0, 127.0 / 255.0 * alpha],
        );
        // The black rest.
        r.push_fx_rect(
            [fill_x + fill_w, cy - num12 / 2.0],
            [num11 * (1.0 - frac), num12],
            [0.0, 0.0, 0.0, alpha],
        );
        // The percent line above the bar (`DrawBorderString`, the MouseText).
        let percent = format!("{}%", (progress as f32 * 100.0 / progress_max as f32) as i32);
        let w = text::hud_text_width(r, lookups, &percent, num);
        text::border_string(
            r,
            lookups,
            cx - w / 2.0,
            cy - num12 / 2.0 - 14.0,
            [1.0, 1.0, 1.0, alpha],
            &percent,
        );
    }
    // The label line under the bar (`DrawBorderString`).
    let w = text::hud_text_width(r, lookups, label, num);
    text::border_string(
        r,
        lookups,
        cx - w / 2.0,
        cy + num8 as f32 / 2.0,
        [1.0, 1.0, 1.0, alpha],
        label,
    );
}

pub fn draw_hud(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    lookups: &assets::Lookups,
    inventory: &[ItemSlot; INV],
    favorites: &[usize],
    equip: &inv::EquipSlots,
    equip_page: i32,
    selected: usize,
    hotbar_scales: &[f32; HOTBAR],
    player: Option<&Player>,
    boss: Option<(i32, i32)>,
    inventory_open: bool,
    crafting_open: bool,
    mouse_item: &ItemSlot,
    mouse_pos: Vec2,
    cursor_state: &cursor::CursorState,
    craft: &[usize],
    pulse: UiPulse,
    m_h: f32,
    map_style: i32,
    bestiary_open: bool,
    bestiary_selected: Option<usize>,
    kill_tracker: &tr_core::bestiary::KillTracker,
    chat_state: &mut crate::chat::ChatMonitor,
    emotes_state: &crate::emotes::EmoteSystem,
    shop: Option<(&[u16], f32)>,
    chest_panel: chest::PanelState<'_>,
    reforge: Option<(&crate::item::ItemSlot, u64, i32)>,
    bank: Option<(&[ItemSlot], i32)>,
) {
    // `DrawInterface_25_...` (`Main.cs:43181`): the classic life / mana / breath
    // resources are the first interface layer, before the inventory (27) and the
    // hotbar (30).
    resources::draw(r, lookups, screen, player, camera, zoom, pulse);

    // `ItemSlot.Draw` / `Main.DrawInventory`: with the inventory open the hotbar is
    // row 0 of a 10x5 grid of 56 px cells at (20,20) at `inventoryScale = 0.85`
    // (context 0: grey `InventoryBack`, hotbar row `InventoryBack9`); closed, the ten
    // slots animate between 0.75 and 1.0 (context 13: grey back at 200/255, the
    // selected one yellow `InventoryBack14` at full white).
    // `Main.DrawInterface`'s chat tail (`Main.cs:40928-40997`): the input
    // strip + the log while open, the blinker advancing per drawn frame.
    draw_chat(r, lookups, screen, chat_state);

    // `EmoteBubble.DrawAll` + the `UIEmotesMenu` state (M6-EMOTE-1).
    let (pc, pdir) = player
        .map(|p| {
            let (cx, cy) = p.center();
            (Vec2::new(cx, cy), p.facing)
        })
        .unwrap_or((Vec2::new(camera.x, camera.y), 1));
    emotes::draw(r, lookups, screen, emotes_state, pc, pdir, mouse_pos);

    // `Main.DrawBestiaryIcon(num, num2)` (`Main.cs:41013`): the button rides
    // the open inventory's top row, before the slots.
    if inventory_open {
        bestiary::draw_inventory_button(r, lookups, mouse_pos);
    }

    // `IngameFancyUI.OpenUIState(Main.BestiaryUI)` (`Main.cs:42459`): the
    // bestiary covers the interface while open (the state's draw replaces
    // the HUD behind it).
    if bestiary_open {
        bestiary::draw_screen(r, lookups, screen, kill_tracker, bestiary_selected, mouse_pos);
        return;
    }

    let mut open_hover: Option<inventory::HoverRef> = None;
    if inventory_open {
        // `Main.DrawInventory` (`Main.cs:41001`): the bounded 10x5 grid, the coin and
        // ammo columns, the equipment/vanity/dye columns (or the misc-equip page),
        // the trash slot and the "Inventory"/"Coins"/"Ammo" labels.
        let hit = inventory::draw(
            r,
            lookups,
            screen,
            inventory,
            favorites,
            equip,
            selected,
            equip_page,
            mouse_pos,
            m_h,
            bank,
        );
        open_hover = hit.hovered;
        // `num20 = 174 + Main.mH` (`Main.cs:41146`).
        let num20 = 174.0 + m_h;
        // `Main.DrawPageIcons(num20 - 32)` (`Main.cs:41154`).
        chrome::draw_page_icons(r, lookups, screen, equip_page, num20 - 32.0, mouse_pos);
        // `Main.DrawDefenseCounter(num39, num40)` (`Main.cs:41421-41424`): drawn
        // beside the last accessory row, only on the equipment pages. Defence is
        // `Player.statDefense`, not modelled yet, so a fresh character's 0.
        if let Some((dx, dy)) = hit.defense_anchor {
            chrome::draw_defense_counter(r, lookups, dx, dy, 0, 0.85);
        }
        // `Main.DrawGuideCraftText(73, 331 + num50)` (`Main.cs:41694`); the guide
        // slot cannot be opened in the port yet, so the empty label is drawn.
        chrome::draw_guide_text(r, lookups, 73.0, 331.0 + (screen.y - 600.0) / 2.0, None);
        // The shop row (`Main.DrawShop`): the stock's names and the
        // happiness-adjusted prices in a text column above the inventory
        // grid (the vanilla draws the item slots at (406, 258+); the slot
        // frames land with the slot-work pass).
        if let Some((stock, mood)) = shop {
            text::hud_text(r, lookups, 40.0, 34.0, 0.85, [1.0; 4], "Shop");
            for (i, id) in stock.iter().enumerate().take(10) {
                let y = 50.0 + i as f32 * 16.0;
                let def = &crate::itemdata::ITEMS[*id as usize];
                let price = crate::shop::buy_price(*id, mood);
                text::hud_text(
                    r,
                    lookups,
                    40.0,
                    y,
                    0.85,
                    [1.0; 4],
                    &format!("{} - {} copper", def.name, price),
                );
            }
        }
        // The reforge menu (`Main.DrawInventory`'s inline block,
        // `Main.cs:41557-41685`): the (50, 270) input slot, the (120, 310)
        // button (30x30), the cost label above the button (`Lang.inter[46]`
        // + the coin breakdown — the port prints the copper total), the
        // `Lang.inter[20]` "place an item here" hint on the empty slot and
        // the held item's name; the cooldown greys the label. The port's
        // text-panel fidelity rides the same coordinates + m_h offset.
        if let Some((slot, cost, cooldown)) = reforge {
            text::hud_text(r, lookups, 50.0, 250.0 + m_h, 0.85, [1.0; 4], "Reforge");
            if slot.count > 0 {
                let def = &crate::itemdata::ITEMS[slot.kind as usize];
                text::hud_text(
                    r,
                    lookups,
                    76.0,
                    274.0 + m_h,
                    0.85,
                    [1.0; 4],
                    &format!("{} x{}", def.name, slot.count),
                );
            } else {
                text::hud_text(r, lookups, 76.0, 274.0 + m_h, 0.85, [0.8; 4], "place an item here");
            }
            let label = if cooldown > 0 {
                "cooling down".to_string()
            } else {
                format!("Reforge - {} copper", cost)
            };
            let ready = cooldown == 0;
            let tint: [f32; 4] = if ready { [1.0; 4] } else { [0.55; 4] };
            text::hud_text(r, lookups, 120.0, 316.0 + m_h, 0.85, tint, &label);
        }
        // `Main.cs:41831`: `ChestUI.Draw` runs after the shop pass — the chest
        // panel draws together with the inventory (`OpenChest` sets
        // `playerInventory`, `Player.cs:33517`, chest.rs:409). The closed arm
        // decays the button state (`ChestUI.cs:99-104`).
        chest::draw(r, lookups, chest_panel, pulse, mouse_pos);
        // `Main.DrawPVPIcons` (`Main.cs:41003`): the icons ride the open
        // inventory's downward push (`Main.mH`) — the hardcoded 0.0 kept them
        // glued to the top while the minimap HUD pushed the panel down.
        chrome::draw_pvp_icons(r, lookups, screen, equip_page, m_h);
        chrome::draw_hotbar_lock(r, lookups, false);
        // `Main.DrawInventory`'s map-style icon row (`Main.cs:41055-41126`),
        // after `DrawHotbarLockIcon` (`:41046`) and before the crafting panel
        // (`:41706`).
        chrome::draw_map_style_icons(r, lookups, screen, map_style, mouse_pos);
        // `Main.cs:41706-41709`: with `NewCraftingUI.Visible` the crafting panel is
        // drawn after `Main.DrawInventory`'s own passes. The guide slot cannot be
        // opened in the port yet (see `draw_guide_text` above), so `guide_mode`
        // stays off, and a fresh character has neither expert mode nor the
        // creative inventory unblocked.
        if crafting_open {
            crafting::draw(r, lookups, screen, mouse_pos, craft, false, false);
            // Hovering a recipe shows the crafted item's tooltip
            // (`Main.HoverOverCraftingItemButton`, `Main.cs:42250`).
            if let Some(idx) = crafting::hit_test(screen, lookups, mouse_pos, craft) {
                let (kind, count) = crate::craft::RECIPES[idx].create;
                let hover = crate::item::ItemSlot {
                    kind,
                    name: crate::registry::name(kind),
                    count,
                    prefix: 0,
                };
                tooltip::draw_item_tooltip(r, lookups, mouse_pos, Some(&hover), screen, pulse);
            }
        }
    } else {
        // `GUIHotbarDrawInner` (`Main.cs:43600-43639`): the closed hotbar is a row
        // of context-13 `ItemSlot.Draw` calls at the per-slot `hotbarScale`, the x
        // advancing by `(int)(InventoryBack.Width() * scale) + 4` from x=20 and the
        // row `y` compensating with `(int)(20 + 22 * (1 - scale))`. The scale lerp
        // itself runs in `app::frame` (`Main.cs:43604-43611`); `draw_slot` derives
        // the `(255,255,255,(int)(75+150*scale))` tint (`Main.cs:43615`) and the
        // context-13 background (`ItemSlot.cs:2232-2242`).
        let mut x = 20.0;
        for i in 0..HOTBAR {
            let scale = hotbar_scales[i].clamp(0.75, 1.0);
            let (y, advance, _) = slot::closed_hotbar_metrics(scale);
            slot::draw_slot(
                r,
                lookups,
                &inventory[i],
                x,
                y,
                13,
                i as i32,
                i == selected,
                scale,
                [1.0; 4],
            );
            x += advance;
        }
    }
    // Selected item name, centred on x=236 at y=0 like the original
    // (`GUIHotbarDrawInner`, `Main.cs:43593-43595`): `FontAssets.MouseText` at
    // scale 1, tinted `Color(Main.mouseTextColor x4)` — no shadow pass.
    let item = &inventory[selected.min(HOTBAR - 1)];
    if item.count > 0 && !inventory_open {
        // `Lang.GetItemNameValue` via the `ItemName.<const>` key.
        let label = crate::lang::item_name(item.kind);
        let tw = text::hud_text_width(r, lookups, &label, 1.0);
        let t = pulse.text;
        text::hud_text(r, lookups, 236.0 - tw * 0.5, 0.0, 1.0, [t, t, t, t], &label);
    }

    // `Main.mouseItem`: `DrawInterface_38_MouseCarriedObject` (`Main.cs:45173-45195`)
    // draws it through `ItemSlot.Draw` at context 21 — no background
    // (`ItemSlot.cs:2236-2240`) — at `(Main.mouseX, Main.mouseY)` with
    // `inventoryScale = Main.cursorScale`, so the icon centres on
    // `position + InventoryBack.Size() / 2 * cursorScale` and keeps the 32 px clamp.
    // (The vanilla `DoGlowingMouseItemDraw` shader pass is not ported.)
    if mouse_item.count > 0 {
        slot::draw_slot(
            r,
            lookups,
            mouse_item,
            mouse_pos.x,
            mouse_pos.y,
            21,
            0,
            false,
            pulse.cursor_scale,
            [1.0; 4],
        );
    }

    // Boss bar: `BigProgressBarHelper.DrawFancyBar` (`BigProgressBarHelper.cs:25-57`,
    // the 4-arg overload via `CommonBossBigProgressBar`) — the 456x22 bar at the
    // bottom centre (`ui/bars.rs`).
    if let Some((hp, max_hp)) = boss {
        bars::draw_fancy_bar(r, lookups, screen, hp, max_hp);
    }

    // `Main.DrawPendingMouseText()` → `Main.MouseText` (`Main.cs:20162`, `20322`):
    // the hovered slot's item tooltip, drawn over the interface just before the
    // cursor. With the inventory open the hover comes from the panel layout
    // (`inventory::draw`); closed, only the hotbar row is hoverable.
    let hover_item: Option<&ItemSlot> = if inventory_open {
        open_hover.and_then(|h| hover_item_for(equip, inventory, h, bank.map(|b| b.0)))
    } else {
        tooltip::hovered_slot(mouse_pos, false, selected, hotbar_scales, inventory)
            .map(|i| &inventory[i])
    };
    tooltip::draw_item_tooltip(r, lookups, mouse_pos, hover_item, screen, pulse);

    // `Main.DrawPendingMouseText()` then `Main.DrawCursor(...)` are the last
    // things `Main.DrawInterface` draws (`Main.cs:44939`); the OS pointer is
    // hidden while the vanilla `Cursor_0` is on screen.
    cursor::draw(r, lookups, cursor_state, mouse_pos);
}

/// Maps an `inventory::HoverRef` (which panel the pointer is over) to the item it
/// holds, so the tooltip layer can show its name.
fn hover_item_for<'a>(
    equip: &'a inv::EquipSlots,
    inventory: &'a [ItemSlot; INV],
    hit: inventory::HoverRef,
    bank: Option<&'a [ItemSlot]>,
) -> Option<&'a ItemSlot> {
    use inventory::HoverRef as H;
    match hit {
        H::Grid(i) => inventory.get(i),
        H::Bank(i) => bank.and_then(|b| b.get(i)),
        // The buttons carry no item: the tooltip stays off.
        H::BankButton(_) => None,
        H::Coin(i) => equip.coins.get(i),
        H::Ammo(i) => equip.ammo.get(i),
        H::Armor(i) => equip.armor.get(i),
        H::Dye(i) => equip.dye.get(i),
        H::MiscEquip(i) => equip.misc_equips.get(i),
        H::MiscDye(i) => equip.misc_dyes.get(i),
        H::Trash => Some(&equip.trash),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_rect_is_inclusive_exclusive_like_rectangle_contains() {
        // `UIElement.cs:354-357`: `Rectangle.Contains` includes the left/top
        // edges and excludes the right/bottom. (Moved with the code from
        // `crafting.rs`'s `rect_hover` test — the helper it pinned is now this
        // shared function.)
        let r = [10.0, 20.0, 40.0, 46.0];
        assert!(in_rect(Vec2::new(10.0, 20.0), r));
        assert!(in_rect(Vec2::new(49.9, 65.9), r));
        assert!(!in_rect(Vec2::new(50.0, 30.0), r));
        assert!(!in_rect(Vec2::new(30.0, 66.0), r));
        assert!(!in_rect(Vec2::new(9.9, 30.0), r));
        assert!(!in_rect(Vec2::new(30.0, 19.9), r));
    }
}

/// The chat draw (`Main.cs:40928-40997` + `RemadeChatMonitor.DrawChat`):
/// the blinker advances once per drawn frame, the closed chat draws nothing
/// (the port has no `CanBeShownWhenChatIsClosed` sources), the open chat
/// draws the log at `(88, screenHeight - 58)` upward 22 px per line and the
/// input strip with the `|` cursor.
fn draw_chat(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    chat: &mut crate::chat::ChatMonitor,
) {
    // `textBlinkerCount`/`textBlinkerState` (`:40928-40944`) — 20-frame
    // toggle, like the menu input screens.
    chat.blink_timer = chat.blink_timer.wrapping_add(1);
    if chat.blink_timer >= 20 {
        chat.blink_timer = 0;
        chat.blink_on = !chat.blink_on;
    }
    if !chat.open {
        return;
    }
    // `RemadeChatMonitor.DrawChat` (`:55-96`): the newest-first log, 22 px
    // rows, capped by the port's 20-line window (`_showCount`'s screen-fit
    // stand-in).
    for (i, line) in chat.messages.iter().take(20).enumerate() {
        let c = [
            line.color[0] as f32 / 255.0,
            line.color[1] as f32 / 255.0,
            line.color[2] as f32 / 255.0,
            1.0,
        ];
        text::hud_text(
            r,
            lookups,
            88.0,
            screen.y - 58.0 - i as f32 * 22.0,
            1.0,
            c,
            &line.text,
        );
    }
    // The `Text_Back` strip (`:40947-40976`): a head piece (the sheet minus
    // 100 px), 300 px middle tiles, then the trimmed tail — the source runs
    // the same three-piece loop for `screenWidth > 800`.
    if let Some(sheet) = lookups.ui_text_back {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let y = screen.y - 36.0;
        let h = sheet.height as f32;
        let w = sheet.width as f32;
        let head = (w - 100.0).max(0.0);
        let mut draw_piece = |x: f32, piece_w: f32, src_x: f32, src_w: f32| {
            if piece_w <= 0.0 || src_w <= 0.0 {
                return;
            }
            let x0 = sheet.x as f32 + src_x;
            let uv = [x0 / aw, sheet.y as f32 / ah, (x0 + src_w) / aw, (sheet.y as f32 + h) / ah];
            r.push_ui_sprite([x, y], [piece_w, h], uv, [100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0]);
        };
        // `num = screenWidth - 300` paces the loop; the tail consumes the
        // remainder.
        let mut num = screen.x - 300.0;
        let mut x = 78.0;
        draw_piece(x, head, 0.0, head);
        x += head;
        num -= 400.0;
        while num > 0.0 {
            if num > 300.0 {
                draw_piece(x, 300.0, 100.0, w - 200.0);
                x += 300.0;
                num -= 300.0;
            } else {
                let tail = num;
                draw_piece(x, tail, w - tail, tail);
                num = 0.0;
            }
        }
    }
    // The input text (`:40986-40994`): white at `(88, screenHeight - 30)`,
    // the `|` blinker appended while `textBlinkerState == 1`.
    let cursor = if chat.blink_on { "|" } else { "" };
    text::hud_text(
        r,
        lookups,
        88.0,
        screen.y - 30.0,
        1.0,
        [1.0, 1.0, 1.0, 1.0],
        &format!("{}{}", chat.text, cursor),
    );
}
