//! Inventory panels: the geometry half of `Main.DrawInventory`
//! (`decompiled/Terraria/Main.cs:41001-42367`).
//!
//! Covers the 10x5 main grid, the coin and ammo columns, the three equipment
//! columns (`EquipPage == 0`), the misc-equip / misc-dye page (`EquipPage == 2`),
//! the trash slot and the three `Lang.inter` labels. The interaction
//! (`ItemSlot.Handle`) lives elsewhere; this module only resolves the hovered
//! slot so the controller can wire tooltips (`InventoryHit`).
//!
//! Scope note (plan Task 3): the reforge/guide slots, the page icons, the defence
//! counter, the guide text, the PVP icons and the hotbar lock are Tasks 4/5.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::{bestiary, slot, text};
use crate::assets;
use crate::inv::EquipSlots;
use crate::item::{ItemSlot, INV};

// ---------------------------------------------------------------------------
// Constants (every literal carries its `Main.cs` line)
// ---------------------------------------------------------------------------

/// `Main.cs:41017`: `Main.inventoryScale = 0.85f` for the grid and equipment.
const INVENTORY_SCALE: f32 = 0.85;
/// `Main.cs:41752`: `Main.inventoryScale = 0.6f` for the coin column.
const COIN_SCALE: f32 = 0.6;
/// `Main.cs:41770`: `Main.inventoryScale = 0.6f` for the ammo column.
const AMMO_SCALE: f32 = 0.6;
/// `Main.cs:41026-41027`: grid pivot `(20, 20)` and layout pitch `56f`.
const GRID_PIVOT: f32 = 20.0;
const GRID_PITCH: f32 = 56.0;
/// `Main.cs:41022-41028`: 10 columns x 5 rows, slot `i + j*10`.
const GRID_COLS: usize = 10;
const GRID_ROWS: usize = 5;
/// `Main.cs:41146`: `num20 = 174 + Main.mH` — the `174` base; `m_h` threads in
/// as an `equip_base_y` parameter (`0`, or `256` while the minimap HUD is live,
/// `Main.cs:45780`).
const EQUIP_BASE_Y: f32 = 174.0;
/// `Main.cs:41147,41149-41153`: `num21 = 950`; below it a `>= 10`-slot stack is
/// pulled up by `(int)(56 * inventoryScale * (num19 - 9))`.
const EQUIP_COLLAPSE_SCREEN_H: f32 = 950.0;
/// `Main.cs:41374`: `num33 = 4` — the extra offset of the accessory rows.
const EQUIP_ROW_SHIFT: f32 = 4.0;
/// `Main.cs:41223`: `r.Y = num24 + m * 47`.
const MISC_PITCH: f32 = 47.0;
/// `Main.cs:42352-42353`: `num = 448`, `num2 = 258` (pivot `(0, 0)`).
const TRASH_X: f32 = 448.0;
const TRASH_Y: f32 = 258.0;
/// `Main.cs:67274`: `Main.inventoryBack = new Color(220, 220, 220, 220)`.
const INVENTORY_BACK: [f32; 4] = [220.0 / 255.0; 4];
/// `Main.cs:41748-41751,41766-41769`: labels are drawn in `Main.mouseTextColor`.
/// The pulse is wired at integration; a fixed white is used here.
const LABEL_COLOR: [f32; 4] = [1.0; 4];
/// Fallback cell size when `Inventory_Back` is absent (`ItemSlot.cs:2499` uses
/// `TextureAssets.InventoryBack.Size()`; the retail sheet is 52x52).
const SLOT_FALLBACK_PX: f32 = 52.0;
/// `Main.cs:41381-41382`: `CanDemonHeartAccessoryBeShown()` and
/// `CanMasterModeAccessoryBeShown()`. The port models neither, so accessory
/// slots 8/9 and vanity slots 18/19 stay hidden (`Main.cs:41398,41471,41504`).
const DEMON_HEART_SLOT: bool = false;
const MASTER_SLOT: bool = false;
/// `Main.cs:41145`: `num19 = 8 + GetAmountOfExtraAccessorySlotsToShow()`.
const BASE_ACCESSORY_SLOTS: i32 = 8;
const EXTRA_ACCESSORY_SLOTS: i32 = 0;

/// `Player.GetAmountOfExtraAccessorySlotsToShow()` (`Player.cs:43950-43972`) as
/// the layout sees it — read by `Main.RefreshSettingsButtonStatus` (`:45508`)
/// and by the equipment column's own row count.
pub fn extra_accessory_slots() -> i32 {
    EXTRA_ACCESSORY_SLOTS
}

/// `Main.EquipPage == 2`: the misc-equip / misc-dye page.
pub const EQUIP_PAGE_MISC: i32 = 2;

/// `ItemSlot.LoadoutSlotColors[0, 0..2]` (`ItemSlot.cs:365-371`) — the loadout-1
/// (green) palette. `TryGetSlotColor` (`ItemSlot.cs:2807-2819`) maps context 8/10
/// (armour/accessory) to 0, 9/11 (vanity) to 1 and 12 (dye) to 2. The three loadouts
/// are coloured green/blue/red; a fresh character is on loadout 1.
const LOADOUT1_COLORS: [[f32; 3]; 3] = [
    [50.0 / 255.0, 106.0 / 255.0, 64.0 / 255.0],
    [46.0 / 255.0, 106.0 / 255.0, 98.0 / 255.0],
    [45.0 / 255.0, 85.0 / 255.0, 105.0 / 255.0],
];

/// `ItemSlot.GetColorByLoadout` (`ItemSlot.cs:2778-2792`): the equipment/vanity/dye
/// backgrounds are `Inventory_Back13` (a white sheet) tinted by
/// `loadoutColor * Main.inventoryBack`, lerped toward white by the loadout-change
/// flash (`num`, normally 0 once 30 frames have passed). Contexts without a loadout
/// colour keep `inventoryBack` unchanged.
fn equip_tint(context: i32, inv_back: [f32; 4]) -> [f32; 4] {
    let idx = match context {
        8 | 10 => 0,
        9 | 11 => 1,
        12 => 2,
        _ => return inv_back,
    };
    let c = LOADOUT1_COLORS[idx];
    [
        c[0] * inv_back[0],
        c[1] * inv_back[1],
        c[2] * inv_back[2],
        inv_back[3],
    ]
}

// ---------------------------------------------------------------------------
// Hover result
// ---------------------------------------------------------------------------

/// The inventory panel a hovered slot belongs to; the controller maps this to the
/// tooltip item (plan Task 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverRef {
    /// Main 10x5 grid slot, `0..49`.
    Grid(usize),
    /// `Player.inventory[50..53]`, `0..3`.
    Coin(usize),
    /// `Player.inventory[54..57]`, `0..3`.
    Ammo(usize),
    /// The open bank's slot (`Player.bank[1..4]`, M3-STORE-1).
    Bank(usize),
    /// The transfer button row (M3-STORE-2): the `ChestUI.ButtonID` 0-3.
    BankButton(u8),
    /// `Player.armor[0..19]` (armour and accessories columns).
    Armor(usize),
    /// `Player.dye[0..9]`.
    Dye(usize),
    /// `Player.miscEquips[0..4]`.
    MiscEquip(usize),
    /// The accessory "hide the visuals" tick next to `Player.armor[i]`
    /// (`Main.cs:41426-41445`). Carries the armour index, `3..=9`.
    HideAccessory(usize),
    /// `Player.miscDyes[0..4]`.
    MiscDye(usize),
    /// `Player.trashItem`.
    Trash,
}

/// What the pointer is over after the panels were laid out.
#[derive(Clone, Copy, Debug, Default)]
pub struct InventoryHit {
    pub hovered: Option<HoverRef>,
    /// `(x, y)` to pass to `chrome::draw_defense_counter`, i.e. the armour column
    /// origin and the **last** accessory row's `y` (`Main.cs:41421-41424`, where
    /// `num37 == num35`). `None` on the misc page, where the counter is not drawn.
    pub defense_anchor: Option<(f32, f32)>,
}

// ---------------------------------------------------------------------------
// Pure geometry (unit-tested)
// ---------------------------------------------------------------------------

/// `Main.cs:41026-41027`: a grid cell's top-left, `(int)(20 + n*56*scale)`.
fn grid_pos(i: usize, j: usize, scale: f32) -> Vec2 {
    Vec2::new(
        (GRID_PIVOT + i as f32 * GRID_PITCH * scale) as i32 as f32,
        (GRID_PIVOT + j as f32 * GRID_PITCH * scale) as i32 as f32,
    )
}

/// `Main.cs:41756,41774`: `(int)(85 + row*56*scale + 20)` — coins and ammo share
/// the row formula.
fn coin_ammo_y(row: usize, scale: f32) -> f32 {
    (85.0 + row as f32 * GRID_PITCH * scale + 20.0) as i32 as f32
}

/// The three right-anchored equipment columns (`EquipPage == 0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EquipColumn {
    /// Armour + accessories, `Player.armor[0..9]`.
    Armor,
    /// Vanity set, `Player.armor[10..19]`.
    Vanity,
    /// Dyes, `Player.dye[0..9]`.
    Dye,
}

/// `Main.cs:41404,41475,41508`: `screenWidth - 64 - 28 - 47*k`.
fn equip_column_x(column: EquipColumn, screen_w: f32) -> f32 {
    match column {
        // `Main.cs:41404`: `Main.screenWidth - 64 - 28`.
        EquipColumn::Armor => screen_w - 92.0,
        // `Main.cs:41475`: `Main.screenWidth - 64 - 28 - 47`.
        EquipColumn::Vanity => screen_w - 139.0,
        // `Main.cs:41508`: `Main.screenWidth - 64 - 28 - 47 - 47`.
        EquipColumn::Dye => screen_w - 186.0,
    }
}

/// `Main.cs:41145-41153`: `num20 = 174 + Main.mH`, including the
/// `screenHeight < 950 && n >= 10` collapse (dead at 1600x900 with `n = 8`,
/// kept for fidelity). `m_h` is `Main.mH` (`Main.cs:41146`) — `256` while the
/// minimap HUD is live (`Main.cs:45780`), `0` otherwise.
fn equip_base_y(screen_h: f32, m_h: f32, n: i32, scale: f32) -> f32 {
    let mut y = EQUIP_BASE_Y + m_h;
    if screen_h < EQUIP_COLLAPSE_SCREEN_H && n >= 10 {
        // `num20 -= (int)(56f * Main.inventoryScale * (float)(num19 - 9));`
        y -= (GRID_PITCH * scale * (n - 9) as f32) as i32 as f32;
    }
    y
}

/// `Main.cs:41405,41408`: `(int)(num20 + row*56*scale)`, `+4` for the lower
/// block (`num38 > 2` / `num44 > 12` / `num47 > 2`, `Main.cs:41374`).
fn equip_row_y(base_y: f32, row: usize, lower: bool, scale: f32) -> f32 {
    let y = (base_y + row as f32 * GRID_PITCH * scale) as i32;
    (y + if lower { EQUIP_ROW_SHIFT as i32 } else { 0 }) as f32
}

/// `Main.cs:41223`: `r.Y = num24 + m * 47` (integer arithmetic, no truncation).
fn misc_row_y(base_y: f32, m: usize) -> f32 {
    base_y + m as f32 * MISC_PITCH
}

/// The accessory "hide the visuals" tick (`Main.cs:41425-41454`):
///
/// ```csharp
/// int num41 = Main.screenWidth - 58;
/// int num42 = (int)((float)(num20 - 2) + (float)(num37 * 56) * Main.inventoryScale);
/// ... if (num38 > 2) { num40 += num33; num42 += num33; }
/// ```
///
/// `row` is the drawn row index (`num37`, which skips the un-unlocked slots) and
/// `lower` the `num38 > 2` shift — the same pair `equip_row_y` takes, but the
/// `- 2` is applied **before** the truncation, so the tick and its slot can
/// differ by a pixel.
pub fn tick_pos(screen_w: f32, base_y: f32, row: usize, lower: bool) -> Vec2 {
    let y = (base_y - 2.0 + row as f32 * GRID_PITCH * INVENTORY_SCALE) as i32 as f32;
    Vec2::new(
        screen_w - 58.0,
        y + if lower { EQUIP_ROW_SHIFT as i32 as f32 } else { 0.0 },
    )
}

/// `rectangle.Contains(new Point(Main.mouseX, Main.mouseY))`
/// (`Main.cs:41428`): the tick sheet is **not** scaled, so the box is its native
/// size, and `Rectangle.Contains` excludes the right/bottom edges.
pub fn tick_rect(pos: Vec2, tick: (f32, f32)) -> [f32; 4] {
    [pos.x, pos.y, tick.0, tick.1]
}

fn in_tick(mouse: Vec2, rect: [f32; 4]) -> bool {
    mouse.x >= rect[0]
        && mouse.x < rect[0] + rect[2]
        && mouse.y >= rect[1]
        && mouse.y < rect[1] + rect[3]
}

/// `Main.cs:41145`: the equipment columns' origin, `174 + Main.mH` shifted up
/// while the screen is too short for the full column (`equip_base_y`).
pub fn equip_page_base_y(screen_h: f32, m_h: f32) -> f32 {
    equip_base_y(
        screen_h,
        m_h,
        BASE_ACCESSORY_SLOTS + EXTRA_ACCESSORY_SLOTS,
        INVENTORY_SCALE,
    )
}

/// Every accessory tick of the equipment page as `(Player.armor index, rect)`,
/// in the order `ItemSlot.Draw` lays the column out — `Main.cs:41425-41460`'s
/// `num41/num42` for the `num38 > 2` slots only (armour slots 0..2 carry none).
pub fn accessory_tick_rects(
    screen: Vec2,
    base_y: f32,
    tick: (f32, f32),
) -> Vec<(usize, [f32; 4])> {
    let mut out = Vec::new();
    let mut row = 0usize;
    for idx in 0..10usize {
        if (idx == 8 && !DEMON_HEART_SLOT) || (idx == 9 && !MASTER_SLOT) {
            continue;
        }
        if idx > 2 {
            // `num38 > 2`: the accessories are the `lower` rows (`num33 = 4`).
            out.push((idx, tick_rect(tick_pos(screen.x, base_y, row, true), tick)));
        }
        row += 1;
    }
    out
}

/// `Main.cs:41426-41460`: a press on an accessory tick flips
/// `Player.hideVisibleAccessory[i]` (and nothing else — the tick is checked
/// before the slot, and the vanilla's `Main.mouseItem.IsAir` gate keeps it inert
/// while the cursor holds a stack).
pub fn accessory_tick_click(
    screen: Vec2,
    base_y: f32,
    mouse: Vec2,
    tick: (f32, f32),
    mouse_air: bool,
) -> Option<usize> {
    if !mouse_air {
        return None;
    }
    accessory_tick_rects(screen, base_y, tick)
        .into_iter()
        .find(|(_, r)| in_tick(mouse, *r))
        .map(|(i, _)| i)
}

/// `Main.cs:41446`: `mouseX >= x && mouseX <= x + W*scale && mouseY >= y &&
/// mouseY <= y + H*scale`, with `cell = W = H`. Inclusive on the right/bottom
/// edges too — unlike `ui::in_rect`'s `Rectangle.Contains` (exclusive
/// right/bottom), so the inventory keeps its own comparison instead of the
/// shared helper.
fn hit_test(mouse: Vec2, x: f32, y: f32, scale: f32, cell: f32) -> bool {
    mouse.x >= x && mouse.x <= x + cell * scale && mouse.y >= y && mouse.y <= y + cell * scale
}

// ---------------------------------------------------------------------------
// Draw
// ---------------------------------------------------------------------------

/// `Main.DrawInventory`: the inventory panels (grid, coins, ammo, equipment,
/// misc equips, trash) and their labels. `mouse` resolves the hovered slot.
/// `m_h` is `Main.mH` (`Main.cs:41146`) — the equipment/misc pages sit at
/// `174 + m_h`.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    inventory: &[ItemSlot; INV],
    favorites: &[usize],
    equip: &EquipSlots,
    selected: usize,
    equip_page: i32,
    mouse: Vec2,
    m_h: f32,
    bank: Option<(&[ItemSlot], i32)>,
    chest_or_shop: bool,
    // `Player.hideVisibleAccessory` — the per-accessory visual-hide flags the
    // ticks beside the accessory column read and flip (`Main.cs:41425-41460`).
    hide: &[bool; 10],
    // `Main.mouseItem.IsAir`: every tick draw and hit is gated on it
    // (`Main.cs:41430`, `:41453`).
    mouse_air: bool,
) -> InventoryHit {
    // `Main.DrawInventory`'s button pair (`Main.cs:41012-41013`):
    // `DrawBestiaryIcon` rides beside `DrawEmoteBubblesButton`. The bestiary
    // icon is drawn by `ui::mod` just before this call; the emote bubbles
    // button lives here — its draw call was missing entirely until §14
    // (`BUG_REPORT.md`: the button had a hitbox and a click handler but no
    // sprite, so it was invisible).
    bestiary::draw_emote_button(r, lookups, mouse);

    // `ItemSlot.cs:2499`: cell = `InventoryBack.Size() * inventoryScale`.
    let cell = lookups
        .ui_back
        .map(|b| b.width as f32)
        .unwrap_or(SLOT_FALLBACK_PX);
    let mut hit = InventoryHit::default();

    // `Main.cs:41016`: "Inventory" (`Lang.inter[4]`) at (40, 0), scale 1.
    text::hud_text(r, lookups, 40.0, 0.0, 1.0, LABEL_COLOR, "Inventory");

    // `Main.cs:41022-41039`: the 10x5 grid, context 0, index = slot.
    for i in 0..GRID_COLS {
        for j in 0..GRID_ROWS {
            let slot_idx = i + j * GRID_COLS;
            let p = grid_pos(i, j, INVENTORY_SCALE);
            // `ItemSlot.cs:2149-2156`: the favorited arm swaps the slot's
            // background to the gold `Inventory_Back10`; the port's atlas
            // lacks the sheet, so the fallback tints the default one
            // (`slot::favorited_slot_tint` — the plan-authorized fallback).
            let tint = if slot::favorited_background_active(
                0,
                favorites.contains(&slot_idx),
                &inventory[slot_idx],
            ) {
                slot::favorited_slot_tint(INVENTORY_BACK)
            } else {
                INVENTORY_BACK
            };
            slot::draw_slot(
                r,
                lookups,
                &inventory[slot_idx],
                p.x,
                p.y,
                0,
                slot_idx as i32,
                slot_idx == selected,
                INVENTORY_SCALE,
                tint,
            );
            if hit.hovered.is_none() && hit_test(mouse, p.x, p.y, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Grid(slot_idx));
            }
        }
    }

    // `Main.cs:41748-41751`: "Coins" (`Lang.inter[26]`) at (496, 84), scale 0.75.
    text::hud_text(r, lookups, 496.0, 84.0, 0.75, LABEL_COLOR, "Coins");
    // `Main.cs:41752-41764`: coins at x 497, context 1, scale 0.6, index -1.
    for k in 0..crate::inv::COINS {
        let y = coin_ammo_y(k, COIN_SCALE);
        slot::draw_slot(
            r,
            lookups,
            &equip.coins[k],
            497.0,
            y,
            1,
            -1,
            false,
            COIN_SCALE,
            INVENTORY_BACK,
        );
        if hit.hovered.is_none() && hit_test(mouse, 497.0, y, COIN_SCALE, cell) {
            hit.hovered = Some(HoverRef::Coin(k));
        }
    }

    // `Main.cs:41766-41769`: "Ammo" (`Lang.inter[27]`) at (532, 84), scale 0.75.
    text::hud_text(r, lookups, 532.0, 84.0, 0.75, LABEL_COLOR, "Ammo");
    // `Main.cs:41770-41782`: ammo at x 534, context 2, scale 0.6, index -1.
    for k in 0..crate::inv::AMMO {
        let y = coin_ammo_y(k, AMMO_SCALE);
        slot::draw_slot(
            r,
            lookups,
            &equip.ammo[k],
            534.0,
            y,
            2,
            -1,
            false,
            AMMO_SCALE,
            INVENTORY_BACK,
        );
        if hit.hovered.is_none() && hit_test(mouse, 534.0, y, AMMO_SCALE, cell) {
            hit.hovered = Some(HoverRef::Ammo(k));
        }
    }

    if equip_page == EQUIP_PAGE_MISC {
        // `Main.cs:41172-41223`: misc equips (x screenWidth-92) and misc dyes
        // (x screenWidth-139), pitch 47, contexts 16-20 / 33.
        let base_y = equip_base_y(
            screen.y,
            m_h,
            BASE_ACCESSORY_SLOTS + EXTRA_ACCESSORY_SLOTS,
            INVENTORY_SCALE,
        );
        let misc_x = screen.x - 92.0; // `Main.cs:41177`
        // `Main.cs:41199-41215`: context per misc-equip row.
        const MISC_CONTEXT: [i32; crate::inv::MISC] = [19, 20, 18, 17, 16];
        for l in 0..2usize {
            // `Main.cs:41190`: `r.X = num23 + l * -47`.
            let x = misc_x - l as f32 * MISC_PITCH;
            for m in 0..crate::inv::MISC {
                let y = misc_row_y(base_y, m);
                if l == 0 {
                    slot::draw_slot(
                        r,
                        lookups,
                        &equip.misc_equips[m],
                        x,
                        y,
                        MISC_CONTEXT[m],
                        m as i32,
                        false,
                        INVENTORY_SCALE,
                        INVENTORY_BACK,
                    );
                    if hit.hovered.is_none() && hit_test(mouse, x, y, INVENTORY_SCALE, cell) {
                        hit.hovered = Some(HoverRef::MiscEquip(m));
                    }
                } else {
                    // `Main.cs:41219`: misc dyes use context 33.
                    slot::draw_slot(
                        r,
                        lookups,
                        &equip.misc_dyes[m],
                        x,
                        y,
                        33,
                        m as i32,
                        false,
                        INVENTORY_SCALE,
                        INVENTORY_BACK,
                    );
                    if hit.hovered.is_none() && hit_test(mouse, x, y, INVENTORY_SCALE, cell) {
                        hit.hovered = Some(HoverRef::MiscDye(m));
                    }
                }
            }
        }
    } else {
        // `Main.cs:41372-41524`: equipment columns (`EquipPage == 0`, and the
        // page-3 fallthrough).
        let base_y = equip_base_y(
            screen.y,
            m_h,
            BASE_ACCESSORY_SLOTS + EXTRA_ACCESSORY_SLOTS,
            INVENTORY_SCALE,
        );
        let armor_x = equip_column_x(EquipColumn::Armor, screen.x);
        let vanity_x = equip_column_x(EquipColumn::Vanity, screen.x);
        let dye_x = equip_column_x(EquipColumn::Dye, screen.x);

        // `Main.cs:41396-41463`: `Player.armor[0..9]` (context 8 / 10).
        let mut row = 0usize;
        for idx in 0..10usize {
            if (idx == 8 && !DEMON_HEART_SLOT) || (idx == 9 && !MASTER_SLOT) {
                continue;
            }
            let lower = idx > 2;
            let context = if lower { 10 } else { 8 };
            let y = equip_row_y(base_y, row, lower, INVENTORY_SCALE);
            slot::draw_slot(
                r,
                lookups,
                &equip.armor[idx],
                armor_x,
                y,
                context,
                idx as i32,
                false,
                INVENTORY_SCALE,
                equip_tint(context, INVENTORY_BACK),
            );
            if hit.hovered.is_none() && hit_test(mouse, armor_x, y, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Armor(idx));
            }
            // `Main.cs:41425-41460`: the "hide the visuals" tick beside every
            // **accessory** row (`num38 > 2`), drawn only while the cursor holds
            // nothing (`Main.mouseItem.IsAir`) and at `Color.White * 0.7f`. The
            // sheet is not scaled; the off sheet is picked when the accessory is
            // already hidden.
            if idx > 2 && mouse_air {
                let pos = tick_pos(screen.x, base_y, row, lower);
                let sheet = if hide[idx] {
                    lookups.ui_tick_off.as_ref()
                } else {
                    lookups.ui_tick_on.as_ref()
                };
                if let Some(rect) = sheet {
                    let (uv, tw, th) = super::chrome::sheet_frame(rect, lookups.atlas_size);
                    
                    r.push_ui_sprite([pos.x, pos.y], [tw, th], uv, [1.0, 1.0, 1.0, 0.7]);
                    // `rectangle.Contains(mouse)` (`:41428`) — the tick wins the
                    // hover over the slot under it and swaps the tooltip for the
                    // "Visible" / "Hidden" mouse text (`:41444`).
                    if hit.hovered.is_none() && in_tick(mouse, tick_rect(pos, (tw, th))) {
                        hit.hovered = Some(HoverRef::HideAccessory(idx));
                    }
                }
            }
            row += 1;
        }

        // `Main.cs:41469-41496`: `Player.armor[10..19]` (context 9 / 11).
        row = 0;
        for idx in 10..20usize {
            if (idx == 18 && !DEMON_HEART_SLOT) || (idx == 19 && !MASTER_SLOT) {
                continue;
            }
            let lower = idx > 12;
            let context = if lower { 11 } else { 9 };
            let y = equip_row_y(base_y, row, lower, INVENTORY_SCALE);
            slot::draw_slot(
                r,
                lookups,
                &equip.armor[idx],
                vanity_x,
                y,
                context,
                idx as i32,
                false,
                INVENTORY_SCALE,
                equip_tint(context, INVENTORY_BACK),
            );
            if hit.hovered.is_none() && hit_test(mouse, vanity_x, y, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Armor(idx));
            }
            row += 1;
        }

        // `Main.cs:41502-41523`: `Player.dye[0..9]` (context 12).
        row = 0;
        for idx in 0..crate::inv::DYE {
            if (idx == 8 && !DEMON_HEART_SLOT) || (idx == 9 && !MASTER_SLOT) {
                continue;
            }
            let lower = idx > 2;
            let y = equip_row_y(base_y, row, lower, INVENTORY_SCALE);
            slot::draw_slot(
                r,
                lookups,
                &equip.dye[idx],
                dye_x,
                y,
                12,
                idx as i32,
                false,
                INVENTORY_SCALE,
                equip_tint(12, INVENTORY_BACK),
            );
            if hit.hovered.is_none() && hit_test(mouse, dye_x, y, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Dye(idx));
            }
            row += 1;
        }

        // `Main.cs:41421-41424`: `DrawDefenseCounter(num39, num40)` is called while
        // the loop is at the last accessory row (`num37 == num35 == num19 - 1`);
        // `num39 = screenWidth - 92`, `num40` = that row's `y` (with the `+4` shift,
        // since row 7 > 2).
        let last_row = (BASE_ACCESSORY_SLOTS + EXTRA_ACCESSORY_SLOTS - 1) as usize;
        hit.defense_anchor = Some((armor_x, equip_row_y(base_y, last_row, true, INVENTORY_SCALE)));
    }

    // `Main.cs:42349-42366`: the trash slot at (448, 258), context 6, scale
    // 0.85 — (+5, +168) at 0.755 while `Main.ChestOrShopUIVisible` (bug 20).
    let ([trash_x, trash_y, _, _], trash_scale) = trash_rect(chest_or_shop);
    slot::draw_slot(
        r,
        lookups,
        &equip.trash,
        trash_x,
        trash_y,
        6,
        -1,
        false,
        trash_scale,
        INVENTORY_BACK,
    );
    if hit.hovered.is_none() && hit_test(mouse, trash_x, trash_y, trash_scale, cell) {
        hit.hovered = Some(HoverRef::Trash);
    }

    // The bank panel (M3-STORE-1): `ChestUI.Draw`'s 10x4 bank grid
    // (`UI/ChestUI.cs:337-397`, context 4 at scale 0.755) — the port keeps
    // the 0.85 cell geometry and appends the grid BELOW the inventory rows
    // (the vanilla interleaves it under the hotbar; the deviation keeps the
    // port's own grid clean). The titles (`Lang.inter[32]/[33]` and the
    // forge/vault item names, `ChestUI.cs:143-158`).
    if let Some((slots, chest)) = bank {
        let title = match chest {
            -2 => "Piggy Bank",
            -3 => "Safe",
            -4 => "Defender's Forge",
            _ => "Void Vault",
        };
        text::hud_text(r, lookups, 40.0, BANK_PIVOT_Y + m_h - 14.0, 0.85, LABEL_COLOR, title);
        // The transfer button row (`ChestUI.DrawButton`, `ChestUI.cs:187-228`):
        // the labels stacked top-to-bottom, 26 px apart — the vanilla sits at
        // x 506 right of the columns; the port's panel-fidelity parks the row
        // under the title. RenameChest (5) is bank-hidden (:198 — `chest <
        // -1`); the port's no-world-chest UI shows 0-3 only (Sort rides the
        // M3-CHEST-UI panel).
        for (id, label) in [
            (0u8, "Loot All"),
            (1, "Deposit All"),
            (2, "Quick Stack"),
            (3, "Restock"),
        ] {
            let p = bank_button_pos(id);
            let ready = hit.hovered.is_none()
                && hit_test(mouse, p.x, p.y + m_h, 1.0, BANK_BUTTON_HIT[0]);
            let tint = if ready { [1.0; 4] } else { [0.85; 4] };
            text::hud_text(r, lookups, p.x, p.y + m_h, 0.85, tint, label);
            if ready {
                hit.hovered = Some(HoverRef::BankButton(id));
            }
        }
        for idx in 0..crate::bank::BANK_SLOTS {
            let i = idx % 10;
            let j = idx / 10;
            let p = bank_pos(i, j);
            slot::draw_slot(
                r,
                lookups,
                &slots[idx],
                p.x,
                p.y + m_h,
                4,
                idx as i32,
                false,
                INVENTORY_SCALE,
                INVENTORY_BACK,
            );
            if hit.hovered.is_none() && hit_test(mouse, p.x, p.y + m_h, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Bank(idx));
            }
        }
    }

    hit
}

/// The bank grid's pivot — one cell gap under the 5 inventory rows
/// (`20 + 5 * 56 * 0.85 = 258`), M3-STORE-1.
pub(crate) const BANK_PIVOT_Y: f32 = 270.0;
pub(crate) const BANK_COLS: usize = 10;
/// The transfer-button row: x 506 (the vanilla's `DrawButton` x, :187),
/// the 26 px pitch (:203), the 120x16 hit box.
pub(crate) const BANK_BUTTON_HIT: [f32; 2] = [120.0, 16.0];

fn bank_button_pos(id: u8) -> Vec2 {
    Vec2::new(506.0, BANK_PIVOT_Y + id as f32 * 26.0)
}

/// The transfer button under `mouse` (`None` off the row). `id` is the
/// `ChestUI.ButtonID` (0 LootAll / 1 DepositAll / 2 QuickStack / 3 Restock).
pub(crate) fn bank_button_at(mouse: Vec2, m_h: f32) -> Option<u8> {
    for id in 0..4u8 {
        let p = bank_button_pos(id);
        let m = Vec2::new(mouse.x, mouse.y - m_h);
        if mouse.x >= p.x
            && mouse.x <= p.x + BANK_BUTTON_HIT[0]
            && m.y >= p.y
            && m.y <= p.y + BANK_BUTTON_HIT[1]
        {
            return Some(id);
        }
    }
    None
}

fn bank_pos(i: usize, j: usize) -> Vec2 {
    Vec2::new(
        (GRID_PIVOT + i as f32 * GRID_PITCH * INVENTORY_SCALE) as i32 as f32,
        (BANK_PIVOT_Y + j as f32 * GRID_PITCH * INVENTORY_SCALE) as i32 as f32,
    )
}

/// The bank cell at `mouse` for the click pass (the same `hit_test` shape,
/// App-side). `None` off the grid.
pub(crate) fn bank_slot_at(mouse: Vec2, m_h: f32) -> Option<usize> {
    let m = Vec2::new(mouse.x, mouse.y - m_h);
    for idx in 0..crate::bank::BANK_SLOTS {
        let i = idx % 10;
        let j = idx / 10;
        let p = bank_pos(i, j);
        if hit_test(m, p.x, p.y, INVENTORY_SCALE, SLOT_FALLBACK_PX) {
            return Some(idx);
        }
    }
    None
}

/// `Main.DrawTrashItemSlot`'s chest/shop scale (`Main.cs:42353-42356`): with a
/// container open the slot shrinks to 0.755 and shifts (+5, +168).
const TRASH_CHEST_SCALE: f32 = 0.755;
const TRASH_CHEST_DX: f32 = 5.0;
const TRASH_CHEST_DY: f32 = 168.0;

/// The trash slot's draw/hit rect — `(448, 258)` at scale 0.85, or
/// `(453, 426)` at 0.755 with a container open (`Main.cs:42349-42366`'s
/// `Main.ChestOrShopUIVisible` arm).
pub fn trash_rect(chest_or_shop: bool) -> ([f32; 4], f32) {
    if chest_or_shop {
        (
            [TRASH_X + TRASH_CHEST_DX, TRASH_Y + TRASH_CHEST_DY, 0.0, 0.0],
            TRASH_CHEST_SCALE,
        )
    } else {
        ([TRASH_X, TRASH_Y, 0.0, 0.0], INVENTORY_SCALE)
    }
}

/// The trash cell at `mouse` for the click pass — the same `hit_test` shape
/// the draw pass's hover uses, at the same shifted rect (`Main.cs:42359-42365`
/// hit-tests the drawn slot).
pub(crate) fn trash_slot_at(mouse: Vec2, chest_or_shop: bool) -> bool {
    let ([x, y, _, _], scale) = trash_rect(chest_or_shop);
    hit_test(mouse, x, y, scale, SLOT_FALLBACK_PX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALE: f32 = INVENTORY_SCALE; // 0.85
    const SCREEN_W: f32 = 1600.0;
    const SCREEN_H: f32 = 900.0;

    fn slot_ij(slot: usize) -> (usize, usize) {
        (slot % GRID_COLS, slot / GRID_COLS)
    }

    #[test]
    fn grid_positions_at_1600x900() {
        // Slots 0, 9, 10, 49 at `Main.inventoryScale = 0.85` (`Main.cs:41026`).
        assert_eq!(grid_pos(slot_ij(0).0, slot_ij(0).1, SCALE), Vec2::new(20.0, 20.0));
        assert_eq!(grid_pos(slot_ij(9).0, slot_ij(9).1, SCALE), Vec2::new(448.0, 20.0));
        assert_eq!(grid_pos(slot_ij(10).0, slot_ij(10).1, SCALE), Vec2::new(20.0, 67.0));
        assert_eq!(grid_pos(slot_ij(49).0, slot_ij(49).1, SCALE), Vec2::new(448.0, 210.0));
    }

    #[test]
    fn trash_slot_hit_matches_the_draw_geometry() {
        // `Main.cs:42352-42353`: (448, 258) at scale 0.85, cell 52 —
        // inclusive edges, the same shape the draw pass's hover uses.
        assert!(trash_slot_at(Vec2::new(448.0, 258.0), false));
        assert!(trash_slot_at(Vec2::new(470.0, 280.0), false));
        assert!(trash_slot_at(
            Vec2::new(448.0 + 52.0 * 0.85, 258.0 + 52.0 * 0.85),
            false
        ));
        assert!(!trash_slot_at(Vec2::new(447.9, 280.0), false));
        assert!(!trash_slot_at(Vec2::new(470.0, 257.9), false));
        assert!(!trash_slot_at(Vec2::new(492.3, 280.0), false));
        assert!(!trash_slot_at(Vec2::new(470.0, 302.3), false));
    }

    #[test]
    fn coin_and_ammo_rows() {
        // `Main.cs:41756,41774`: 105, 138, 172, 205 at scale 0.6.
        assert_eq!(coin_ammo_y(0, COIN_SCALE), 105.0);
        assert_eq!(coin_ammo_y(1, COIN_SCALE), 138.0);
        assert_eq!(coin_ammo_y(2, COIN_SCALE), 172.0);
        assert_eq!(coin_ammo_y(3, COIN_SCALE), 205.0);
        // Ammo shares the formula.
        for k in 0..4 {
            assert_eq!(coin_ammo_y(k, AMMO_SCALE), coin_ammo_y(k, COIN_SCALE));
        }
    }

    #[test]
    fn equipment_column_x_at_1600() {
        // `Main.cs:41404,41475,41508`: 1600-92, -139, -186.
        assert_eq!(equip_column_x(EquipColumn::Armor, SCREEN_W), 1508.0);
        assert_eq!(equip_column_x(EquipColumn::Vanity, SCREEN_W), 1461.0);
        assert_eq!(equip_column_x(EquipColumn::Dye, SCREEN_W), 1414.0);
    }

    #[test]
    fn equipment_base_y_and_collapse() {
        // `Main.cs:41146`: 174 at 1600x900 with n = 8 and `mH = 0` (no collapse).
        assert_eq!(equip_base_y(SCREEN_H, 0.0, 8, SCALE), 174.0);
        // `Main.cs:41149-41153`: dead at 1600x900 but ported; n = 10 and
        // screenHeight < 950 pulls it up by (int)(56*0.85*1) = 47.
        assert_eq!(equip_base_y(SCREEN_H, 0.0, 10, SCALE), 127.0);
        // `Main.cs:41145`: with all 8 base slots + 2 extras.
        assert_eq!(equip_base_y(1000.0, 0.0, 10, SCALE), 174.0);
    }

    #[test]
    fn m_h_pushes_the_equipment_block_down() {
        // `Main.cs:41146`: `num20 = 174 + Main.mH` — 430 with the minimap HUD
        // live (`mH = 256`, `Main.cs:45780`) and 174 without it.
        assert_eq!(equip_base_y(900.0, 256.0, 8, SCALE), 430.0);
        assert_eq!(equip_base_y(900.0, 0.0, 8, SCALE), 174.0);
    }

    #[test]
    fn equip_tint_uses_loadout1_palette() {
        // `ItemSlot.cs:365-371` row 0 = green/teal/blue; multiplied by
        // `Main.inventoryBack` (`ItemSlot.cs:2785`).
        let back = INVENTORY_BACK;
        let armor = equip_tint(10, back);
        assert!((armor[0] - 50.0 / 255.0 * back[0]).abs() < 1e-6);
        assert!(armor[1] > armor[0] && armor[1] > armor[2], "armour slot is green");
        let vanity = equip_tint(11, back);
        assert!(vanity[2] > vanity[0], "vanity slot is teal");
        let dye = equip_tint(12, back);
        assert!(dye[2] > dye[1], "dye slot is blue");
        // Contexts without a loadout colour keep the plain background.
        assert_eq!(equip_tint(16, back), back);
        assert_eq!(equip_tint(33, back), back);
    }

    #[test]
    fn equipment_row_shift_of_four() {
        // `Main.cs:41374,41412-41414`: the `> 2` rows drop 4 px.
        assert_eq!(equip_row_y(174.0, 0, false, SCALE), 174.0);
        assert_eq!(equip_row_y(174.0, 2, false, SCALE), 269.0);
        assert_eq!(equip_row_y(174.0, 3, true, SCALE), 320.0);
        assert_eq!(
            equip_row_y(174.0, 3, true, SCALE) - equip_row_y(174.0, 3, false, SCALE),
            EQUIP_ROW_SHIFT
        );
    }

    #[test]
    fn misc_row_pitch_is_47() {
        // `Main.cs:41223`: num24 + m*47, num24 = 174.
        assert_eq!(misc_row_y(174.0, 0), 174.0);
        assert_eq!(misc_row_y(174.0, 1), 221.0);
        assert_eq!(misc_row_y(174.0, 2), 268.0);
        assert_eq!(misc_row_y(174.0, 3), 315.0);
        assert_eq!(misc_row_y(174.0, 4), 362.0);
    }

    #[test]
    fn trash_position() {
        // `Main.cs:42352-42353`.
        assert_eq!((TRASH_X, TRASH_Y), (448.0, 258.0));
    }

    /// `Main.cs:41425-41427`: `num41 = screenWidth - 58` and `num42 =
    /// (int)((num20 - 2) + num37*56*scale)`, `+num33` (4) on the accessory rows.
    #[test]
    fn the_accessory_ticks_sit_at_the_vanillas_x_and_row() {
        let base_y = equip_page_base_y(SCREEN_H, 0.0);
        assert_eq!(base_y, 174.0, "`num20 = 174 + Main.mH` at 900 px");
        // Row 0 is the helmet (no tick); the first accessory is row 3, so its
        // tick is row 3's y minus 2, shifted by the 4 px lower block.
        let tick = tick_pos(SCREEN_W, base_y, 0, false);
        assert_eq!(tick, Vec2::new(1542.0, 172.0));
        let rects = accessory_tick_rects(Vec2::new(SCREEN_W, SCREEN_H), base_y, (16.0, 16.0));
        // `EQUIP_PAGE_MISC`'s sibling: with neither the Demon Heart nor master
        // mode, `Player.armor[8]`/`[9]` are not drawn, so the column ends at
        // index 7 — five accessories, five ticks (armour 0..2 carry none).
        assert_eq!(rects.len(), 5, "armour 3..=7 carry the ticks");
        assert_eq!(rects[0].0, 3, "the first accessory is `Player.armor[3]`");
        // `(int)(174 - 2 + 3*56*0.85) + 4` = 314 + 4.
        assert_eq!(rects[0].1[0], 1542.0);
        assert_eq!(rects[0].1[1], 318.0);
        // The last row is 7: `(int)(172 + 7*47.6) + 4` = 505 + 4.
        assert_eq!(rects[4].0, 7);
        assert_eq!(rects[4].1[1], 509.0);
    }

    /// `Rectangle.Contains` (`Main.cs:41428`) excludes the right/bottom edges and
    /// the tick is only live while the cursor holds nothing (`:41430`).
    #[test]
    fn the_tick_hit_is_half_open_and_needs_an_air_cursor() {
        let screen = Vec2::new(SCREEN_W, SCREEN_H);
        let base_y = equip_page_base_y(SCREEN_H, 0.0);
        let r = accessory_tick_rects(screen, base_y, (16.0, 16.0))[0].1;
        let inside = Vec2::new(r[0] + 1.0, r[1] + 1.0);
        assert_eq!(
            accessory_tick_click(screen, base_y, inside, (16.0, 16.0), true),
            Some(3)
        );
        // The right and bottom edges are outside the rectangle.
        let edge = Vec2::new(r[0] + r[2], r[1] + 1.0);
        assert_eq!(accessory_tick_click(screen, base_y, edge, (16.0, 16.0), true), None);
        let edge = Vec2::new(r[0] + 1.0, r[1] + r[3]);
        assert_eq!(accessory_tick_click(screen, base_y, edge, (16.0, 16.0), true), None);
        // `Main.mouseItem.IsAir` (`:41430`): a held stack disables every tick.
        assert_eq!(
            accessory_tick_click(screen, base_y, inside, (16.0, 16.0), false),
            None
        );
        // The armour slots 0..2 have no tick at all.
        for (idx, _) in accessory_tick_rects(screen, base_y, (16.0, 16.0)) {
            assert!(idx > 2, "slot {idx} must not carry a tick");
        }
    }

    /// The `num38 > 2` gate means the tick rows are offset by the three armour
    /// slots: row 3 of the column is the first accessory.
    #[test]
    fn the_tick_row_follows_the_slot_row() {
        let base_y = equip_page_base_y(SCREEN_H, 0.0);
        for (idx, rect) in accessory_tick_rects(Vec2::new(SCREEN_W, SCREEN_H), base_y, (16.0, 16.0))
        {
            // The armour column draws rows 0..=7 in `Player.armor` order, so the
            // armour index **is** the drawn row (`num37`).
            let slot_y = equip_row_y(base_y, idx, true, SCALE);
            assert!(
                (rect[1] - (slot_y - 2.0)).abs() <= 1.0,
                "slot {idx}: tick {} vs slot {slot_y}",
                rect[1]
            );
        }
    }
}
