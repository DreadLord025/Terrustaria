//! In-game HUD: hotbar / open inventory, selected-item name, the item carried on the
//! cursor, hearts and breath, and the boss bar.

pub mod bars;
pub mod chrome;
pub mod color;
pub mod crafting;
pub mod cursor;
pub mod inventory;
pub mod minimap;
pub mod panel;
pub mod resources;
pub mod sliders;
pub mod slot;
pub mod text;
pub mod tooltip;

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::ItemSlot;
use crate::assets;
use crate::inv;
use crate::player::Player;
use crate::{HOTBAR, INV};

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
    pulse: UiPulse,
    m_h: f32,
    map_style: i32,
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
        // `Main.DrawPVPIcons` (`Main.cs:41003`).
        chrome::draw_pvp_icons(r, lookups, screen, equip_page, 0.0);
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
            crafting::draw(r, lookups, screen, mouse_pos, false, false);
        }
    } else if let Some(back) = lookups.hud_uv(lookups.ui_back.as_ref()) {
        let sel_back = lookups.hud_uv(lookups.ui_back_sel.as_ref()).unwrap_or(back);
        let mut x = 20.0;
        for i in 0..HOTBAR {
            let scale = hotbar_scales[i].clamp(0.75, 1.0);
            let slot = 52.0 * scale;
            let y = 20.0 + 22.0 * (1.0 - scale);
            let (uv, c) = if i == selected {
                (sel_back, 255.0 / 255.0)
            } else {
                (back, 200.0 / 255.0)
            };
            r.push_ui_sprite([x, y], [slot, slot], uv, [c, c, c, c]);
            slot::draw_slot_item(r, lookups, &inventory[i], x, y, slot, scale);
            slot::draw_slot_number(r, lookups, x, y, i, scale, i == selected);
            x += slot + 4.0;
        }
    }
    // Selected item name, centred above the bar like the original.
    let item = &inventory[selected.min(HOTBAR - 1)];
    if item.count > 0 && !inventory_open {
        let label = item.name;
        let tw = text::hud_text_width(r, lookups, label, 1.0);
        let tx = 236.0 - tw * 0.5;
        text::hud_text(r, lookups, tx + 1.0, 1.0, 1.0, [0.0, 0.0, 0.0, 1.0], label);
        text::hud_text(r, lookups, tx, 0.0, 1.0, [1.0, 0.95, 0.85, 1.0], label);
    }

    // `Main.mouseItem`: the stack carried on the cursor, centred on it.
    if mouse_item.count > 0 {
        slot::draw_slot_item(
            r,
            lookups,
            mouse_item,
            mouse_pos.x - 26.0,
            mouse_pos.y - 26.0,
            52.0,
            1.0,
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
