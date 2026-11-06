//! In-game HUD: hotbar / open inventory, selected-item name, the item carried on the
//! cursor, hearts and breath, and the boss bar.

pub mod bars;
pub(crate) mod bestiary;
pub(crate) mod emotes;
pub mod cards;
pub mod char_creation;
pub mod char_select;
pub mod chrome;
pub mod color;
pub mod crafting;
pub mod cursor;
pub mod inventory;
pub mod keyboard;
pub mod minimap;
pub mod panel;
pub mod resources;
pub mod save_list;
pub mod sliders;
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
pub fn draw_hud(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    lookups: &assets::Lookups,
    inventory: &[ItemSlot; INV],
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
            equip,
            selected,
            equip_page,
            mouse_pos,
            m_h,
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
        let label = item.name;
        let tw = text::hud_text_width(r, lookups, label, 1.0);
        let t = pulse.text;
        text::hud_text(r, lookups, 236.0 - tw * 0.5, 0.0, 1.0, [t, t, t, t], label);
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
        open_hover.and_then(|h| hover_item_for(equip, inventory, h))
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
) -> Option<&'a ItemSlot> {
    use inventory::HoverRef as H;
    match hit {
        H::Grid(i) => inventory.get(i),
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
