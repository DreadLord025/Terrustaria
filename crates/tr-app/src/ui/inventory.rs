//! Inventory panels: the geometry half of `Main.DrawInventory`
//! (`decompiled/Terraria/Main.cs:41001-42367`).
//!
//! Covers the 10x5 main grid, the coin and ammo columns, the three equipment
//! columns (`EquipPage == 0`), the NPC housing page (`EquipPage == 1`, see
//! `ui::housing`), the misc-equip / misc-dye page (`EquipPage == 2`), the trash
//! slot and the three `Lang.inter` labels. The interaction
//! (`ItemSlot.Handle`) lives elsewhere; this module only resolves the hovered
//! slot so the controller can wire tooltips (`InventoryHit`).
//!
//! Scope note (plan Task 3): the reforge/guide slots, the page icons, the defence
//! counter, the guide text, the PVP icons and the hotbar lock are Tasks 4/5.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::housing;
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

/// The reforge menu's input-slot hit box (`Main.cs:41673`):
/// `mouseX >= num51 && mouseX <= num51 + InventoryBack.Width() * inventoryScale`
/// and the same for Y with `num52`, where `num51 = 50`, `num52 = 270`
/// (`Main.cs:41581-41582`) and `inventoryScale = 0.85` (`Main.cs:41017`, restored
/// at `:41526` before the reforge block). The C# compares are inclusive on the
/// right/bottom. Neither coordinate rides `Main.mH` — the reforge block uses the
/// literal `(50, 270)`; `mH` only shifts the equipment column's `num20`
/// (`Main.cs:41146`). `cell` is the `Inventory_Back` width (52 px retail).
pub(crate) fn reforge_slot_hit(mouse: Vec2, cell: f32) -> bool {
    let (x, y) = (50.0_f32, 270.0_f32); // `Main.cs:41581-41582`
    mouse.x >= x
        && mouse.x <= x + cell * INVENTORY_SCALE
        && mouse.y >= y
        && mouse.y <= y + cell * INVENTORY_SCALE
}

/// The reforge button's hit box (`Main.cs:41639-41641`): the centre is
/// `(num59, num60) = (num51 + 70, num52 + 40) = (120, 310)` and the test is the
/// STRICT `mouseX > num59 - 15 && mouseX < num59 + 15` (and Y), i.e. the open
/// interval `(105, 135) x (295, 325)`. Like the slot it does not ride `Main.mH`.
pub(crate) fn reforge_button_hit(mouse: Vec2) -> bool {
    let (cx, cy) = (120.0_f32, 310.0_f32); // `Main.cs:41639-41640`
    mouse.x > cx - 15.0 && mouse.x < cx + 15.0 && mouse.y > cy - 15.0 && mouse.y < cy + 15.0
}

/// `Main.cs:41381-41382`: `CanDemonHeartAccessoryBeShown()` and
/// `CanMasterModeAccessoryBeShown()` — now LIVE per-player reads
/// (`Player.cs:4276-4292`, W4-g87): the Demon Heart's `extraAccessory` +
/// expert unlock, or a stored item in `armor[8]`/`armor[18]`/`dye[8]`, shows
/// the row; master mode is unported so slot 9 shows only while occupied.
/// The pair rides every column loop (`Main.cs:41398,41471,41504`) and the
/// row count (`num19`, `:41145`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExtraAccessoryShow {
    /// `flag4` — the Demon Heart row (armor 8 / vanity 18 / dye 8).
    pub demon_heart: bool,
    /// `flag5` — the master-mode row (armor 9 / vanity 19 / dye 9).
    pub master: bool,
    /// `flag6`/`flag7`/`flag8` — `IsItemSlotUnlockedAndUsable(8)`/`(9)`
    /// (`Main.cs:41403`/`:41474`/`:41507`): a shown-but-locked row draws the
    /// dim background and refuses the interact (`:41450`'s `flag6 ||
    /// Main.mouseItem.IsAir`). Both bits are the same per-slot value across
    /// the three columns (the C# re-reads the same fn per loop).
    pub slot8_unlocked: bool,
    pub slot9_unlocked: bool,
}

impl ExtraAccessoryShow {
    /// The per-frame snapshot off the local player (`Main.LocalPlayer`,
    /// `Main.cs:41381-41382` + the per-loop `flag6/7/8` reads). No player
    /// (the front end) shows neither and keeps the plain background.
    pub fn of(player: Option<&crate::player::Player>) -> Self {
        match player {
            Some(p) => Self {
                demon_heart: p.can_demon_heart_accessory_be_shown(),
                master: p.can_master_mode_accessory_be_shown(),
                slot8_unlocked: p.is_item_slot_unlocked_and_usable(8),
                slot9_unlocked: p.is_item_slot_unlocked_and_usable(9),
            },
            None => Self {
                slot8_unlocked: true,
                slot9_unlocked: true,
                ..Self::default()
            },
        }
    }

    /// `Player.GetAmountOfExtraAccessorySlotsToShow` (`Player.cs:4294-4306`).
    pub fn count(self) -> i32 {
        i32::from(self.demon_heart) + i32::from(self.master)
    }

    /// The column loops' skip test (`Main.cs:41398`/`:41471`/`:41504`):
    /// `idx` is the loop variable in each column's own index space
    /// (armor 0..9, vanity 10..19, dye 0..9).
    fn hides(self, idx: usize) -> bool {
        (matches!(idx, 8 | 18) && !self.demon_heart) || (matches!(idx, 9 | 19) && !self.master)
    }

    /// `IsItemSlotUnlockedAndUsable(idx)` for the drawn row (`Main.cs:41403`
    /// `flag6` / `:41474` `flag7` / `:41507` `flag8`); every other row is
    /// unlocked (the port's slot-9 master lock folds into `slot9_unlocked`).
    pub fn slot_unlocked(self, idx: usize) -> bool {
        match idx {
            8 | 18 => self.slot8_unlocked,
            9 | 19 => self.slot9_unlocked,
            _ => true,
        }
    }
}

/// `Main.cs:41145`: `num19 = 8 + GetAmountOfExtraAccessorySlotsToShow()`.
const BASE_ACCESSORY_SLOTS: i32 = 8;

/// `Main.EquipPage == 2`: the misc-equip / misc-dye page.
pub const EQUIP_PAGE_MISC: i32 = 2;
/// `EquipPage == 1` (`Main.cs:41368`): the NPC housing page.
const EQUIP_PAGE_HOUSING: i32 = 1;

/// `ItemSlot.LoadoutSlotColors` (`ItemSlot.cs:365-382`) — the three loadout
/// palettes (green / blue / red), each `[armor, vanity, dye]`.
/// `TryGetSlotColor` (`ItemSlot.cs:2807-2819`) maps context 8/10
/// (armour/accessory) to 0, 9/11 (vanity) to 1 and 12 (dye) to 2. A fresh
/// character is on loadout 0 (the green row).
const LOADOUT_SLOT_COLORS: [[[f32; 3]; 3]; 3] = [
    [
        [50.0 / 255.0, 106.0 / 255.0, 64.0 / 255.0],
        [46.0 / 255.0, 106.0 / 255.0, 98.0 / 255.0],
        [45.0 / 255.0, 85.0 / 255.0, 105.0 / 255.0],
    ],
    [
        [35.0 / 255.0, 106.0 / 255.0, 126.0 / 255.0],
        [50.0 / 255.0, 89.0 / 255.0, 140.0 / 255.0],
        [57.0 / 255.0, 70.0 / 255.0, 128.0 / 255.0],
    ],
    [
        [122.0 / 255.0, 63.0 / 255.0, 83.0 / 255.0],
        [104.0 / 255.0, 46.0 / 255.0, 85.0 / 255.0],
        [84.0 / 255.0, 37.0 / 255.0, 87.0 / 255.0],
    ],
];

/// `ItemSlot.GetColorByLoadout` (`ItemSlot.cs:2778-2792`): the equipment/vanity/dye
/// backgrounds are `Inventory_Back13` (a white sheet) tinted by
/// `loadoutColor * Main.inventoryBack`, lerped toward white by the loadout-change
/// flash (`num`, normally 0 once 30 frames have passed). Contexts without a loadout
/// colour keep `inventoryBack` unchanged. `flash` is the precomputed
/// `num * num * num` lerp factor (`Utils.Remap(timeSinceChange, 0, 30, 0.5, 0)`,
/// cubed; 0 when `FlashyEffectsInterface` is off or 30 frames have passed).
fn equip_tint(context: i32, inv_back: [f32; 4], loadout: usize, flash: f32) -> [f32; 4] {
    let idx = match context {
        8 | 10 => 0,
        9 | 11 => 1,
        12 => 2,
        _ => return inv_back,
    };
    let c = LOADOUT_SLOT_COLORS[loadout.min(2)][idx];
    let base = [
        c[0] * inv_back[0],
        c[1] * inv_back[1],
        c[2] * inv_back[2],
        inv_back[3],
    ];
    // `Color.Lerp(value, Color.White, num * num * num)` (`:2791`).
    [
        base[0] + (1.0 - base[0]) * flash,
        base[1] + (1.0 - base[1]) * flash,
        base[2] + (1.0 - base[2]) * flash,
        base[3] + (1.0 - base[3]) * flash,
    ]
}

/// The loadout-change flash factor (`ItemSlot.cs:2786-2791`):
/// `num = Utils.Remap(timeForVisualEffects - lastChange, 0, 30, 0.5, 0)` (clamped),
/// then `num^3`. `Main.FlashyEffectsInterface` off forces 0. A never-changed
/// stamp (`NEG_INFINITY`) yields 0.
pub fn loadout_flash(last_change: f32, time_for_visuals: f32, flashy: bool) -> f32 {
    if !flashy || !last_change.is_finite() {
        return 0.0;
    }
    let elapsed = time_for_visuals - last_change;
    // `Utils.GetLerpValue(0, 30, elapsed, clamped: true)` then `Lerp(0.5, 0, t)`.
    let t = (elapsed / 30.0).clamp(0.0, 1.0);
    let num = 0.5 + (0.0 - 0.5) * t;
    num * num * num
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
    /// The misc-equip page's pet/light tick (`Main.cs:41226-41256`), rows 0/1.
    HideMisc(usize),
    /// The misc-equip page's super-cart tick (`Main.cs:41258-41283`), row 2.
    SuperCart,
    /// `Player.miscDyes[0..4]`.
    MiscDye(usize),
    /// The quick-stack-all button (`Main.cs:41833-41868`, W4-g21 batch 3).
    QuickStackButton,
    /// The sort-inventory button (`Main.cs:41869-41897`, W4-g21 batch 3).
    SortButton,
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
    /// The hovered housing cell as `(head, npc)` (`Main.cs:42059-42065`) — the
    /// tooltip layer turns it into a name (`housing::label`). Copy-friendly, so
    /// the hit stays `Copy`.
    pub housing: Option<(usize, Option<usize>)>,
    /// `Main.hidePVPAndTeamIcons` (`Main.cs:42053-42056`) — set by the housing
    /// page once its columns reach the PVP icons.
    pub hide_pvp_icons: bool,
    /// The quick-stack/sort buttons' hover-edge tick count (`DoStatefulTickSound`,
    /// `Main.cs:41865/:41890` → `:3244`): the caller plays this many `Menu_Tick`
    /// cues after the draw (the audio borrow stays with the chest panel during
    /// the call, W4-g21 batch 3).
    pub hover_ticks: u32,
    /// The hovered loadout button (`Main.DrawLoadoutButtons`' `state`,
    /// `Main.cs:41956-41990`), -1 = none. Drives the `DoStatefulTickSound` 12
    /// cue and the `UI.LoadoutN` mouse text.
    pub loadout_hover: i32,
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
/// `extra` is the LIVE `GetAmountOfExtraAccessorySlotsToShow()` pair.
pub fn equip_page_base_y(screen_h: f32, m_h: f32, extra: ExtraAccessoryShow) -> f32 {
    equip_base_y(
        screen_h,
        m_h,
        BASE_ACCESSORY_SLOTS + extra.count(),
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
    extra: ExtraAccessoryShow,
) -> Vec<(usize, [f32; 4])> {
    let mut out = Vec::new();
    let mut row = 0usize;
    for idx in 0..10usize {
        if extra.hides(idx) {
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
    extra: ExtraAccessoryShow,
) -> Option<usize> {
    if !mouse_air {
        return None;
    }
    accessory_tick_rects(screen, base_y, tick, extra)
        .into_iter()
        .find(|(_, r)| in_tick(mouse, *r))
        .map(|(i, _)| i)
}

/// `Main.cs:41226`: the misc tick box sits at `(r.Left + 34, r.Top - 2)` in
/// the slot's NATIVE tick size (`Rectangle.Contains` — half-open edges).
pub fn misc_tick_pos(screen_w: f32, base_y: f32, m: usize) -> Vec2 {
    let x = screen_w - 92.0; // `Main.cs:41177`
    Vec2::new(x + 34.0, misc_row_y(base_y, m) - 2.0)
}

/// `Main.cs:41265-41266`: the super-cart tick box recentres on the pet tick's
/// box — `r2 = (X + W/2, Y + H/2, W, H)` then `Offset(-W/2, -H/2)`. The two
/// integer half-width offsets cancel exactly, so the box lands on the same
/// `(r.Left + 34, r.Top - 2)` anchor as the pet/light ticks (row 2).
pub fn super_cart_tick_pos(screen_w: f32, base_y: f32, _tick: (f32, f32)) -> Vec2 {
    misc_tick_pos(screen_w, base_y, 2)
}

/// The misc-page tick hit (`Main.cs:41228-41283`), in the C#'s order: the
/// pet/light tick first (rows 0/1, `num25 != -1`), then the super-cart tick
/// (row 2, `flag2 = unlockedSuperCart`). Returns `(row, super_cart)`.
/// `mouse_air` gates both (`Main.mouseItem.IsAir`, `:41228/:41258`).
pub fn misc_tick_click(
    screen_w: f32,
    base_y: f32,
    mouse: Vec2,
    tick: (f32, f32),
    mouse_air: bool,
    unlocked_super_cart: bool,
) -> Option<(usize, bool)> {
    if !mouse_air {
        return None;
    }
    for m in 0..2usize {
        if in_tick(mouse, tick_rect(misc_tick_pos(screen_w, base_y, m), tick)) {
            return Some((m, false));
        }
    }
    if unlocked_super_cart
        && in_tick(mouse, tick_rect(super_cart_tick_pos(screen_w, base_y, tick), tick))
    {
        return Some((2, true));
    }
    None
}

/// `Main.cs:41833-41868`: the quick-stack-all button at `(498, 244)` in the
/// `ChestStack[0]` sheet's native size; the hover box is inclusive on every
/// edge (`mouseX >= num79 && mouseX <= num79 + num81`, `:41842`). The sheet
/// fallback is 30x30 (the retail `UI/ChestStack_0` cell).
pub fn quick_stack_button_hit(mouse: Vec2, size: (f32, f32)) -> bool {
    let (x, y) = (498.0, 244.0); // `Main.cs:41836-41837`
    mouse.x >= x && mouse.x <= x + size.0 && mouse.y >= y && mouse.y <= y + size.1
}

/// `Main.cs:41869-41897`: the sort-inventory button at `(534, 244)`, a fixed
/// 30x30 box (`num86 = num87 = 30`, `:41874-41875`), inclusive edges (`:41878`).
pub fn sort_button_hit(mouse: Vec2) -> bool {
    let (x, y) = (534.0, 244.0); // `Main.cs:41872-41873`
    mouse.x >= x && mouse.x <= x + 30.0 && mouse.y >= y && mouse.y <= y + 30.0
}

/// `Main.cs:41446`: `mouseX >= x && mouseX <= x + W*scale && mouseY >= y &&
/// mouseY <= y + H*scale`, with `cell = W = H`. Inclusive on the right/bottom
/// edges too — unlike `ui::in_rect`'s `Rectangle.Contains` (exclusive
/// right/bottom), so the inventory keeps its own comparison instead of the
/// shared helper.
fn hit_test(mouse: Vec2, x: f32, y: f32, scale: f32, cell: f32) -> bool {
    mouse.x >= x && mouse.x <= x + cell * scale && mouse.y >= y && mouse.y <= y + cell * scale
}

/// The equipment cell under `mouse` as `(context, index)` — the three
/// right-hand columns of `Main.DrawInventory`'s armour block (`Main.cs:41396-41524`:
/// `Player.armor[0..9]` ctx 8/10, `armor[10..19]` ctx 9/11, `dye[0..9]` ctx 12)
/// and the misc page's two columns (`:41172-41223`: `miscEquips` ctx 16-20,
/// `miscDyes` ctx 33). `index` is the array index the C# passes to
/// `ItemSlot.Handle`. The draw pass hovers with this geometry and the click pass
/// resolves the same cell, so the two copies cannot drift.
///
/// The housing page (`EquipPage == 1`) owns its own cells (`housing::hover`).
pub(crate) fn equip_cell_at(
    mouse: Vec2,
    screen: Vec2,
    m_h: f32,
    equip_page: i32,
    cell: f32,
    extra: ExtraAccessoryShow,
) -> Option<(i32, usize)> {
    let base_y = equip_base_y(
        screen.y,
        m_h,
        BASE_ACCESSORY_SLOTS + extra.count(),
        INVENTORY_SCALE,
    );
    if equip_page == EQUIP_PAGE_MISC {
        // `Main.cs:41177,41190`: misc equips at `screenWidth - 92`, misc dyes
        // one 47 px pitch left.
        let misc_x = screen.x - 92.0;
        // `Main.cs:41199-41219`: context per misc-equip row.
        const MISC_CONTEXT: [i32; crate::inv::MISC] = [19, 20, 18, 17, 16];
        for l in 0..2usize {
            let x = misc_x - l as f32 * MISC_PITCH;
            for m in 0..crate::inv::MISC {
                let y = misc_row_y(base_y, m);
                if hit_test(mouse, x, y, INVENTORY_SCALE, cell) {
                    return Some(if l == 0 { (MISC_CONTEXT[m], m) } else { (33, m) });
                }
            }
        }
        return None;
    }
    if equip_page == EQUIP_PAGE_HOUSING {
        return None;
    }
    let armor_x = equip_column_x(EquipColumn::Armor, screen.x);
    let vanity_x = equip_column_x(EquipColumn::Vanity, screen.x);
    let dye_x = equip_column_x(EquipColumn::Dye, screen.x);
    // `Main.cs:41396-41463`: `Player.armor[0..9]` (context 8, 10 below row 2).
    let mut row = 0usize;
    for idx in 0..10usize {
        if extra.hides(idx) {
            continue;
        }
        let y = equip_row_y(base_y, row, idx > 2, INVENTORY_SCALE);
        if hit_test(mouse, armor_x, y, INVENTORY_SCALE, cell) {
            return Some((if idx > 2 { 10 } else { 8 }, idx));
        }
        row += 1;
    }
    // `Main.cs:41469-41496`: `Player.armor[10..19]` (context 9, 11 below row 2).
    row = 0;
    for idx in 10..20usize {
        if extra.hides(idx) {
            continue;
        }
        let y = equip_row_y(base_y, row, idx > 12, INVENTORY_SCALE);
        if hit_test(mouse, vanity_x, y, INVENTORY_SCALE, cell) {
            return Some((if idx > 12 { 11 } else { 9 }, idx));
        }
        row += 1;
    }
    // `Main.cs:41502-41523`: `Player.dye[0..9]` (context 12).
    row = 0;
    for idx in 0..crate::inv::DYE {
        if extra.hides(idx) {
            continue;
        }
        let y = equip_row_y(base_y, row, idx > 2, INVENTORY_SCALE);
        if hit_test(mouse, dye_x, y, INVENTORY_SCALE, cell) {
            return Some((12, idx));
        }
        row += 1;
    }
    None
}

/// One coin/ammo cell through the shared draw path: the favorited arm swaps the
/// gold sheet, everything else keeps `Main.inventoryBack`.
fn draw_row_slot(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &crate::item::ItemSlot,
    x: f32,
    y: f32,
    context: i32,
    scale: f32,
    favored: bool,
    pulse: super::UiPulse,
    coin_slot: &mut super::coin_slot::CoinSlotState,
    vanilla_slot: i32,
    mouse_air: bool,
    mouse_is_dye: bool,
) {
    if favored {
        slot::draw_slot_favorited(
            r, lookups, item, x, y, context, -1, false, scale, INVENTORY_BACK,
            pulse,
            Some(coin_slot), vanilla_slot, mouse_air, mouse_is_dye,
        );
    } else {
        slot::draw_slot(
            r, lookups, item, x, y, context, -1, false, scale, INVENTORY_BACK,
            pulse,
            Some(coin_slot), vanilla_slot, mouse_air, mouse_is_dye,
        );
    }
}

/// The ctx-1 (coin) and ctx-2 (ammo) cells as `(context, index)`:
/// `Main.cs:41752-41782` draws the four coins at x 497 and the four ammo cells
/// at x 534, both `inventoryScale` 0.6 with the shared `coin_ammo_y` rows. The
/// draw pass hovers with this and the click pass resolves the same cell, so the
/// two geometry copies cannot drift.
pub(crate) fn coin_ammo_hit(mouse: Vec2, cell: f32) -> Option<(i32, usize)> {
    for k in 0..crate::inv::COINS {
        if hit_test(mouse, 497.0, coin_ammo_y(k, COIN_SCALE), COIN_SCALE, cell) {
            return Some((1, k));
        }
    }
    for k in 0..crate::inv::AMMO {
        if hit_test(mouse, 534.0, coin_ammo_y(k, AMMO_SCALE), AMMO_SCALE, cell) {
            return Some((2, k));
        }
    }
    None
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
    npcs: &[crate::npc::Npc],
    mouse: Vec2,
    m_h: f32,
    bank: Option<(&[ItemSlot], i32)>,
    // W4-g56 — the void vault's per-slot favorite bits (`Item.favorited`
    // on `bank4.item`, the `.plr` bools `Player.cs:55495`): context 32's
    // favorited arm swaps to `Inventory_Back19` (`ItemSlot.cs:2155-2158`).
    // `None` for the other banks (their slots cannot favorite).
    bank_favorites: Option<&[bool; crate::bank::BANK_SLOTS]>,
    // W4-g56 — the chest-glow state (`ItemSlot.Draw:2314-2326`): the open
    // container's per-slot glow time/hue (contexts 4/32 here, 3 in the chest
    // panel). `None` outside the glow system's reach (the closed inventory).
    glow: Option<&crate::ui::glow::GlowState>,
    chest_or_shop: bool,
    // `Player.hideVisibleAccessory` — the per-accessory visual-hide flags the
    // ticks beside the accessory column read and flip (`Main.cs:41425-41460`).
    hide: &[bool; 10],
    // `Main.mouseItem.IsAir`: every tick draw and hit is gated on it
    // (`Main.cs:41430`, `:41453`).
    mouse_air: bool,
    // `Main.cs:41381-41382`: the LIVE CanDemonHeart/CanMaster accessory-show
    // pair (`Player.cs:4276-4292`) — the slot-8 row joins the columns once
    // the Demon Heart unlock (or a stored item) shows it.
    extra: ExtraAccessoryShow,
    // `Player.hideMisc` + `Player.unlockedSuperCart`/`enabledSuperCart` — the
    // misc-page pet/light/super-cart ticks (`Main.cs:41226-41283`, W4-g21
    // batch 2 hosts 58/59).
    hide_misc: &[bool; 2],
    unlocked_super_cart: bool,
    enabled_super_cart: bool,
    // `Player.Settings.StackToChestsPreferredMode` — the quick-stack button's
    // frame (`Main.cs:41866`, W4-g21 batch 3 host 63).
    stack_to_chests_mode: u8,
    // `Main.allChestStackHover`/`inventorySortMouseOver` (`Main.cs:1297/:1299`):
    // the buttons' hover latches — the draw advances them via
    // `DoStatefulTickSound` (`:41865/:41890`); the hover-edge ticks are returned
    // in `InventoryHit::hover_ticks` for the caller to play (the audio borrow
    // stays with the chest panel during the call).
    all_chest_stack_hover: &mut bool,
    inventory_sort_hover: &mut bool,
    // `Player.CurrentLoadoutIndex` (`Player.cs:3207`) — the equipment-slot
    // palette + the loadout buttons' active frame (`Main.cs:41394,41963`).
    current_loadout: usize,
    // The loadout-change flash factor (`ItemSlot.GetColorByLoadout:2786-2791`,
    // precomputed by `loadout_flash`).
    loadout_flash: f32,
    // `GetEffectiveArmor`/`GetEffectiveDye` (`Player.cs:5690-5764`): the
    // shared item + its source loadout for an AIR equip slot (the ghost icon
    // + the Extra_313-315 badge), `None` when the slot is live or unshared.
    // Computed by the caller from the Player.
    shared_armor: &[Option<(ItemSlot, usize)>; 20],
    shared_dye: &[Option<(ItemSlot, usize)>; 10],
    // The LIVE equip set's share word (`App.equip_share_flags`, bit i =
    // armor[i], bit 20+j = dye[j]) — a non-air favorited slot draws the
    // Extra_310-312 badge (`ItemSlot.cs:2485-2488` + `:2733-2739`).
    equip_share_flags: u32,
    // W4-g35: the item-slot glow clocks (`ItemSlot.cs:2891-2929`) — the
    // miscCounter/disco/global-time trio the glow tints read.
    pulse: super::UiPulse,
    // The `CoinSlot` machine (`ui/coin_slot.rs`) — `ItemSlot.Draw`'s
    // `CoinSlot.UpdateDrawState` (`ItemSlot.cs:2349`) feeds the grid/coin/
    // ammo/bank cells (W4-d122 S1-H-9 follow-up).
    coin_slot: &mut super::coin_slot::CoinSlotState,
    // `Main.mouseItem.dye > 0` — the dye-slot dim gate
    // (`GetDimSlotForMouseItem`, `ItemSlot.cs:1382`).
    mouse_is_dye: bool,
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
            // background to the gold `Inventory_Back10` (`slot::draw_slot_favorited`)
            // and keeps `Main.inventoryBack` as the tint.
            if slot::favorited_background_active(
                0,
                favorites.contains(&slot_idx),
                &inventory[slot_idx],
            ) {
                slot::draw_slot_favorited(
                    r,
                    lookups,
                    &inventory[slot_idx],
                    p.x,
                    p.y,
                    0,
                    slot_idx as i32,
                    slot_idx == selected,
                    INVENTORY_SCALE,
                    INVENTORY_BACK,
                    pulse,
                    Some(&mut *coin_slot),
                    slot_idx as i32,
                    mouse_air,
                    mouse_is_dye,
                );
            } else {
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
                    INVENTORY_BACK,
                    pulse,
                    Some(&mut *coin_slot),
                    slot_idx as i32,
                    mouse_air,
                    mouse_is_dye,
                );
            }
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
        // `ItemSlot.cs:2149-2156`: a favorited stack draws the gold sheet; the
        // row's key is its vanilla inventory index (`inv::coin_slot_index`).
        let favored = slot::favorited_background_active(
            1,
            favorites.contains(&crate::inv::coin_slot_index(k)),
            &equip.coins[k],
        );
        draw_row_slot(
            r,
            lookups,
            &equip.coins[k],
            497.0,
            y,
            1,
            COIN_SCALE,
            favored,
            pulse,
            coin_slot,
            crate::inv::coin_slot_index(k) as i32,
            mouse_air,
            mouse_is_dye,
        );
        if hit.hovered.is_none() && coin_ammo_hit(mouse, cell) == Some((1, k)) {
            hit.hovered = Some(HoverRef::Coin(k));
        }
    }

    // `Main.cs:41766-41769`: "Ammo" (`Lang.inter[27]`) at (532, 84), scale 0.75.
    text::hud_text(r, lookups, 532.0, 84.0, 0.75, LABEL_COLOR, "Ammo");
    // `Main.cs:41770-41782`: ammo at x 534, context 2, scale 0.6, index -1.
    for k in 0..crate::inv::AMMO {
        let y = coin_ammo_y(k, AMMO_SCALE);
        let favored = slot::favorited_background_active(
            2,
            favorites.contains(&crate::inv::ammo_slot_index(k)),
            &equip.ammo[k],
        );
        draw_row_slot(
            r,
            lookups,
            &equip.ammo[k],
            534.0,
            y,
            2,
            AMMO_SCALE,
            favored,
            pulse,
            coin_slot,
            crate::inv::ammo_slot_index(k) as i32,
            mouse_air,
            mouse_is_dye,
        );
        if hit.hovered.is_none() && coin_ammo_hit(mouse, cell) == Some((2, k)) {
            hit.hovered = Some(HoverRef::Ammo(k));
        }
    }

    if equip_page == EQUIP_PAGE_MISC {
        // `Main.cs:41172-41223`: misc equips (x screenWidth-92) and misc dyes
        // (x screenWidth-139), pitch 47, contexts 16-20 / 33.
        let base_y = equip_base_y(
            screen.y,
            m_h,
            BASE_ACCESSORY_SLOTS + extra.count(),
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
                        pulse,
                        None,
                        m as i32,
                        mouse_air,
                        mouse_is_dye,
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
                        pulse,
                        None,
                        m as i32,
                        mouse_air,
                        mouse_is_dye,
                    );
                    if hit.hovered.is_none() && hit_test(mouse, x, y, INVENTORY_SCALE, cell) {
                        hit.hovered = Some(HoverRef::MiscDye(m));
                    }
                }
            }
        }
        // `Main.cs:41225-41298`: the misc-page ticks — pet/light rows 0/1
        // (`InventoryTickOn/Off`, `:41230-41233`) and the super-cart row 2
        // (`Extra[255]/[256]`, `:41260-41264`), drawn at `Color.White * 0.7f`
        // (`:41293`) while the cursor holds nothing. The tick box wins the
        // hover over the slot under it (`flag3`, `:41284`).
        if mouse_air {
            let tick = lookups
                .ui_tick_on
                .as_ref()
                .map(|s| (s.width as f32, s.height as f32))
                .unwrap_or((16.0, 16.0));
            for m in 0..2usize {
                let pos = misc_tick_pos(screen.x, base_y, m);
                let sheet = if hide_misc[m] {
                    lookups.ui_tick_off.as_ref()
                } else {
                    lookups.ui_tick_on.as_ref()
                };
                if let Some(rect) = sheet {
                    let (uv, tw, th) = super::chrome::sheet_frame(rect, lookups.atlas_size);
                    r.push_ui_sprite([pos.x, pos.y], [tw, th], uv, [1.0, 1.0, 1.0, 0.7]);
                    if hit.hovered.is_none() && in_tick(mouse, tick_rect(pos, (tw, th))) {
                        hit.hovered = Some(HoverRef::HideMisc(m));
                    }
                }
            }
            if unlocked_super_cart {
                let pos = super_cart_tick_pos(screen.x, base_y, tick);
                let sheet = if enabled_super_cart {
                    lookups.extra_255.as_ref()
                } else {
                    lookups.extra_256.as_ref()
                };
                if let Some(rect) = sheet {
                    let (uv, tw, th) = super::chrome::sheet_frame(rect, lookups.atlas_size);
                    r.push_ui_sprite([pos.x, pos.y], [tw, th], uv, [1.0, 1.0, 1.0, 0.7]);
                    if hit.hovered.is_none() && in_tick(mouse, tick_rect(pos, (tw, th))) {
                        hit.hovered = Some(HoverRef::SuperCart);
                    }
                }
            }
        }
    } else if equip_page == EQUIP_PAGE_HOUSING {
        // `Main.cs:41993-42107`: the housing page — one `Inventory_Back11` cell
        // per town NPC type the world holds, in `NPCHeadID.Sets.HeadListOrder`,
        // starting at the equipment columns' top row (`inventoryTop`) on the
        // right edge. Vanilla draws no armour on this page, so the `else` arm
        // below stays page 0/3.
        let base_y = equip_base_y(
            screen.y,
            m_h,
            BASE_ACCESSORY_SLOTS + extra.count(),
            INVENTORY_SCALE,
        );
        let cells = housing::layout(
            &housing::heads_for(npcs),
            [screen.x, screen.y],
            base_y,
            INVENTORY_SCALE,
        );
        housing::draw(r, lookups, &cells, INVENTORY_SCALE);
        // `:42059-42065`: the mouse-text probe (`Main.mouseText = true`, then
        // the cell's name). Vanilla also sets `player.mouseInterface`, i.e. this
        // text is the one on screen for the page.
        hit.housing = housing::hover(&cells, [mouse.x, mouse.y], INVENTORY_SCALE);
        hit.hide_pvp_icons = housing::hide_pvp_icons(&cells, screen.x);
    } else {
        // `Main.cs:41372-41524`: equipment columns (`EquipPage == 0`, and the
        // page-3 fallthrough).
        let base_y = equip_base_y(
            screen.y,
            m_h,
            BASE_ACCESSORY_SLOTS + extra.count(),
            INVENTORY_SCALE,
        );
        let armor_x = equip_column_x(EquipColumn::Armor, screen.x);
        let vanity_x = equip_column_x(EquipColumn::Vanity, screen.x);
        let dye_x = equip_column_x(EquipColumn::Dye, screen.x);

        // `Main.DrawLoadoutButtons(num20, flag4, flag5)` (`Main.cs:41394`):
        // the three loadout buttons beside the armour column, drawn before
        // the slots. The hover state rides the hit for the tick sound + text.
        hit.loadout_hover = super::loadout::draw(
            r,
            lookups,
            screen.x,
            base_y,
            current_loadout,
            [mouse.x, mouse.y],
        );

        // `Main.cs:41396-41463`: `Player.armor[0..9]` (context 8 / 10).
        let mut row = 0usize;
        for idx in 0..10usize {
            if extra.hides(idx) {
                continue;
            }
            let lower = idx > 2;
            let context = if lower { 10 } else { 8 };
            let y = equip_row_y(base_y, row, lower, INVENTORY_SCALE);
            // `Main.cs:41403`/`:41452`: `flag6 = IsItemSlotUnlockedAndUsable`
            // — a SHOWN-but-locked row (a stored item, no unlock) draws the
            // dim `(80,80,80,80)` background instead of `Main.inventoryBack`.
            let back = if extra.slot_unlocked(idx) {
                INVENTORY_BACK
            } else {
                [80.0 / 255.0; 4] // `Main.cs:41393` color2 (80,80,80,80)
            };
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
                // ONE tint channel: the loadout palette over the row's
                // background (the shown-but-locked dim is the `back` local).
                equip_tint(context, back, current_loadout, loadout_flash),
                pulse,
                None,
                idx as i32,
                mouse_air,
                mouse_is_dye,
            );
            // `ItemSlot.cs:2474-2488`: an air slot shared from another loadout
            // draws the shared item's ghost icon over the empty glyph, plus
            // the Extra_313-315 badge; a live shared item gets the
            // Extra_310-312 badge (`:2733-2739`).
            if equip.armor[idx].count == 0 {
                if let Some((shared, from)) = &shared_armor[idx] {
                    slot::draw_shared_item(r, lookups, shared, *from, armor_x, y, INVENTORY_SCALE);
                }
            } else if equip_share_flags & (1 << idx) != 0 {
                slot::draw_share_badge(
                    r,
                    lookups,
                    super::loadout::share_badge_index(current_loadout, false),
                    armor_x,
                    y,
                    INVENTORY_SCALE,
                );
            }
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
            if extra.hides(idx) {
                continue;
            }
            let lower = idx > 12;
            let context = if lower { 11 } else { 9 };
            let y = equip_row_y(base_y, row, lower, INVENTORY_SCALE);
            // `Main.cs:41474`/`:41493`: `flag7` — the vanity twin of flag6.
            let back = if extra.slot_unlocked(idx) {
                INVENTORY_BACK
            } else {
                [80.0 / 255.0; 4]
            };
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
                // ONE tint channel: the loadout palette over the row's
                // background (the shown-but-locked dim is the `back` local).
                equip_tint(context, back, current_loadout, loadout_flash),
                pulse,
                None,
                idx as i32,
                mouse_air,
                mouse_is_dye,
            );
            if equip.armor[idx].count == 0 {
                if let Some((shared, from)) = &shared_armor[idx] {
                    slot::draw_shared_item(r, lookups, shared, *from, vanity_x, y, INVENTORY_SCALE);
                }
            } else if equip_share_flags & (1 << idx) != 0 {
                slot::draw_share_badge(
                    r,
                    lookups,
                    super::loadout::share_badge_index(current_loadout, false),
                    vanity_x,
                    y,
                    INVENTORY_SCALE,
                );
            }
            if hit.hovered.is_none() && hit_test(mouse, vanity_x, y, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Armor(idx));
            }
            row += 1;
        }

        // `Main.cs:41502-41523`: `Player.dye[0..9]` (context 12).
        row = 0;
        for idx in 0..crate::inv::DYE {
            if extra.hides(idx) {
                continue;
            }
            let lower = idx > 2;
            let y = equip_row_y(base_y, row, lower, INVENTORY_SCALE);
            // `Main.cs:41507`/`:41520`: `flag8` — the dye twin of flag6.
            let back = if extra.slot_unlocked(idx) {
                INVENTORY_BACK
            } else {
                [80.0 / 255.0; 4]
            };
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
                equip_tint(12, back, current_loadout, loadout_flash),
                pulse,
                None,
                idx as i32,
                mouse_air,
                mouse_is_dye,
            );
            if equip.dye[idx].count == 0 {
                if let Some((shared, from)) = &shared_dye[idx] {
                    slot::draw_shared_item(r, lookups, shared, *from, dye_x, y, INVENTORY_SCALE);
                }
            } else if equip_share_flags & (1 << (crate::inv::ARMOR + idx)) != 0 {
                slot::draw_share_badge(
                    r,
                    lookups,
                    super::loadout::share_badge_index(current_loadout, false),
                    dye_x,
                    y,
                    INVENTORY_SCALE,
                );
            }
            if hit.hovered.is_none() && hit_test(mouse, dye_x, y, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Dye(idx));
            }
            row += 1;
        }

        // `Main.cs:41421-41424`: `DrawDefenseCounter(num39, num40)` is called while
        // the loop is at the last accessory row (`num37 == num35 == num19 - 1`);
        // `num39 = screenWidth - 92`, `num40` = that row's `y` (with the `+4` shift,
        // since every accessory row > 2). With the Demon Heart row shown the
        // anchor drops to row 8 (`num19 = 9`).
        let last_row = (BASE_ACCESSORY_SLOTS + extra.count() - 1) as usize;
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
        pulse,
        None,
        -1,
        mouse_air,
        mouse_is_dye,
    );
    if hit.hovered.is_none() && hit_test(mouse, trash_x, trash_y, trash_scale, cell) {
        hit.hovered = Some(HoverRef::Trash);
    }

    // The bank panel (M3-STORE-1): `ChestUI.Draw`'s 10x4 bank grid
    // (`UI/ChestUI.cs:337-397`, context 4 at scale 0.755) — the port keeps
    // the 0.85 cell geometry and appends the grid BELOW the inventory rows
    // (the vanilla interleaves it under the hotbar; the deviation keeps the
    // port's own grid clean). The titles (`Lang.inter[32]/[33]` and the
    // forge/vault item names, `ChestUI.cs:143-158`). W4-g56: the void vault
    // (chest −5) draws context 32 (`ChestUI.cs:356-359`) — the same
    // `Inventory_Back2` frame, but its slots CAN favorite
    // (`canFavoriteAt[32]`, `ItemSlot.cs:386-394`) and the favorited arm
    // swaps to `Inventory_Back19` (`:2155-2158`).
    if let Some((slots, chest)) = bank {
        let title = match chest {
            -2 => "Piggy Bank",
            -3 => "Safe",
            -4 => "Defender's Forge",
            _ => "Void Vault",
        };
        let context = if chest == -5 { 32 } else { 4 };
        text::hud_text(r, lookups, 40.0, BANK_PIVOT_Y + m_h - 14.0, 0.85, LABEL_COLOR, title);
        // The transfer button row (`ChestUI.DrawButton`, `ChestUI.cs:187-228`):
        // the labels stacked top-to-bottom, 26 px apart — the vanilla sits at
        // x 506 right of the columns; the port's panel-fidelity parks the row
        // under the title. RenameChest (5) is bank-hidden (:198 — `chest <
        // -1`); the port's no-world-chest UI shows 0-3 + Sort (4, W4-g56 —
        // `ItemSorting.SortChest` covers every container kind,
        // `ItemSorting.cs:1629-1653`).
        for (id, label) in [
            (0u8, "Loot All"),
            (1, "Deposit All"),
            (2, "Quick Stack"),
            (3, "Restock"),
            (4, "Sort"),
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
            let favored = context == 32
                && bank_favorites.is_some_and(|f| f[idx])
                && slots[idx].count > 0;
            // W4-g56 — the chest-glow entries for this slot (`ItemSlot.Draw:
            // 2314-2326`): the bank/vault contexts (4/32) glow when the Sort
            // feedback armed them; the favorite arm still wins.
            let slot_glow = glow.map(|g| (g.chest_time_at(idx), g.chest_hue_at(idx)));
            if favored {
                slot::draw_slot_glow(
                    r,
                    lookups,
                    &slots[idx],
                    p.x,
                    p.y + m_h,
                    context,
                    idx as i32,
                    false,
                    INVENTORY_SCALE,
                    INVENTORY_BACK,
                    pulse,
                    true,
                    slot_glow,
                    Some(&mut *coin_slot),
                    idx as i32,
                    mouse_air,
                    mouse_is_dye,
                );
            } else {
                slot::draw_slot_glow(
                    r,
                    lookups,
                    &slots[idx],
                    p.x,
                    p.y + m_h,
                    context,
                    idx as i32,
                    false,
                    INVENTORY_SCALE,
                    INVENTORY_BACK,
                    pulse,
                    false,
                    slot_glow,
                    Some(&mut *coin_slot),
                    idx as i32,
                    mouse_air,
                    mouse_is_dye,
                );
            }
            if hit.hovered.is_none() && hit_test(mouse, p.x, p.y + m_h, INVENTORY_SCALE, cell) {
                hit.hovered = Some(HoverRef::Bank(idx));
            }
        }
    }

    // `Main.cs:41833-41897`: the quick-stack-all + sort-inventory buttons —
    // both ride `chest == -1 && npcShop == 0` (the port's `bank.is_none()` +
    // `!chest_or_shop` covers the world-chest/shop case; the bank panel keeps
    // its own transfer row). The hover latches advance via DoStatefulTickSound
    // (`:41865/:41890`), the hover-edge tick plays sound 12.
    if bank.is_none() && !chest_or_shop {
        // The quick-stack button (`:41833-41868`): frame `num78` starts 0,
        // becomes 1 on hover, then `+ (int)mode * 2` (`:41866`).
        let stack_size = lookups
            .chest_stack
            .first()
            .and_then(|s| s.as_ref())
            .map(|s| (s.width as f32, s.height as f32))
            .unwrap_or((30.0, 30.0));
        let stack_hover = quick_stack_button_hit(mouse, stack_size);
        let frame = (if stack_hover { 1 } else { 0 }) + (stack_to_chests_mode as usize) * 2;
        if let Some(rect) = lookups.chest_stack.get(frame).and_then(|s| s.as_ref()) {
            let (uv, w, h) = super::chrome::sheet_frame(rect, lookups.atlas_size);
            r.push_ui_sprite([498.0, 244.0], [w, h], uv, [1.0; 4]);
        }
        if stack_hover && hit.hovered.is_none() {
            hit.hovered = Some(HoverRef::QuickStackButton);
        }
        // `DoStatefulTickSound(ref allChestStackHover, state2)` (`:41865`).
        if *all_chest_stack_hover != stack_hover {
            *all_chest_stack_hover = stack_hover;
            if stack_hover {
                hit.hover_ticks += 1; // `:3244`
            }
        }
        // The sort button (`:41869-41897`): `InventorySort[hover ? 1 : 0]`.
        let sort_hover = sort_button_hit(mouse);
        if let Some(rect) = lookups.inventory_sort[if sort_hover { 1 } else { 0 }].as_ref() {
            let (uv, w, h) = super::chrome::sheet_frame(rect, lookups.atlas_size);
            r.push_ui_sprite([534.0, 244.0], [w, h], uv, [1.0; 4]);
        }
        if sort_hover && hit.hovered.is_none() {
            hit.hovered = Some(HoverRef::SortButton);
        }
        if *inventory_sort_hover != sort_hover {
            *inventory_sort_hover = sort_hover;
            if sort_hover {
                hit.hover_ticks += 1; // `:3244`
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
/// `ChestUI.ButtonID` (0 LootAll / 1 DepositAll / 2 QuickStack / 3 Restock /
/// 4 Sort — W4-g56 added the bank Sort row, `ItemSorting.cs:1629-1653`).
pub(crate) fn bank_button_at(mouse: Vec2, m_h: f32) -> Option<u8> {
    for id in 0..5u8 {
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

/// `Main.DrawInventory`'s shop grid (`Main.cs:41798-41818`): the 10×4
/// context-15 cells at `(int)(73 + i*56*0.755)`, `(int)(invBottom +
/// j*56*0.755)` — `Main.invBottom = 258` (`Main.cs:6608`),
/// `Main.inventoryScale = 0.755` (`:41798`). The same layout the chest
/// panel draws at (`ChestUI.cs:379-380`, `ui::chest::slot_pos`).
pub(crate) const SHOP_SCALE: f32 = 0.755;
pub(crate) const SHOP_COLS: usize = 10;
pub(crate) const SHOP_ROWS: usize = 4;

/// One shop cell's top-left — `Main.cs:41807-41808`'s truncated positions.
pub(crate) fn shop_slot_pos(i: usize, j: usize) -> Vec2 {
    Vec2::new(
        (73.0 + i as f32 * GRID_PITCH * SHOP_SCALE) as i32 as f32,
        (258.0 + j as f32 * GRID_PITCH * SHOP_SCALE) as i32 as f32,
    )
}

/// The shop cell under the cursor — `Main.cs:41811`'s per-cell hit test
/// (inclusive on all four edges, the `InventoryBack` size × 0.755), the
/// slot index `i + j*10` (`:41809`). `cell` is the sheet width the draw
/// uses (`lookups.ui_back`, the 52 px retail fallback).
pub(crate) fn shop_slot_at(mouse: Vec2, cell: f32) -> Option<usize> {
    for i in 0..SHOP_COLS {
        for j in 0..SHOP_ROWS {
            let p = shop_slot_pos(i, j);
            let size = cell * SHOP_SCALE;
            if mouse.x >= p.x
                && mouse.x <= p.x + size
                && mouse.y >= p.y
                && mouse.y <= p.y + size
            {
                return Some(i + j * SHOP_COLS);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALE: f32 = INVENTORY_SCALE; // 0.85
    const SCREEN_W: f32 = 1600.0;
    const SCREEN_H: f32 = 900.0;
    /// The classic no-unlock snapshot (`Main.cs:41381-41382` both false).
    const NO_EXTRA: ExtraAccessoryShow = ExtraAccessoryShow {
        demon_heart: false,
        master: false,
        slot8_unlocked: false,
        slot9_unlocked: false,
    };
    /// The Demon-Heart-used expert snapshot (W4-g87's live grant).
    const DH_EXTRA: ExtraAccessoryShow = ExtraAccessoryShow {
        demon_heart: true,
        master: false,
        slot8_unlocked: true,
        slot9_unlocked: false,
    };

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
    fn reforge_slot_hit_matches_the_csharp_rect() {
        // `Main.cs:41673`: `mouseX >= 50 && mouseX <= 50 + InventoryBack.Width()
        // * inventoryScale` (and Y with 270) — INCLUSIVE edges, cell 52 at
        // scale 0.85 (`Main.cs:41017`), so the box is [50, 94.2] x [270, 314.2].
        // No `Main.mH` term: the reforge block's coordinates are literal
        // (`Main.cs:41581-41582`); the minimap push shifts only the equipment
        // column's `num20` (`Main.cs:41146`).
        let cell = 52.0;
        assert!(reforge_slot_hit(Vec2::new(50.0, 270.0), cell));
        assert!(reforge_slot_hit(Vec2::new(70.0, 290.0), cell));
        assert!(reforge_slot_hit(Vec2::new(50.0 + 52.0 * 0.85, 270.0 + 52.0 * 0.85), cell));
        assert!(!reforge_slot_hit(Vec2::new(49.9, 290.0), cell));
        assert!(!reforge_slot_hit(Vec2::new(70.0, 269.9), cell));
        assert!(!reforge_slot_hit(Vec2::new(94.3, 290.0), cell));
        assert!(!reforge_slot_hit(Vec2::new(70.0, 314.3), cell));
        // The old port added `(screenHeight - 600) / 2` — at 900 px that
        // pushed the box to y 420..464; the literal rect refuses it.
        assert!(!reforge_slot_hit(Vec2::new(70.0, 440.0), cell));
    }

    #[test]
    fn reforge_button_hit_is_the_strict_30x30() {
        // `Main.cs:41639-41641`: centre (120, 310), STRICT `> cx-15 && < cx+15`
        // (and Y) — the open interval (105, 135) x (295, 325). The edges
        // themselves are OUT (unlike the slot's inclusive compare).
        assert!(reforge_button_hit(Vec2::new(120.0, 310.0)));
        assert!(reforge_button_hit(Vec2::new(105.1, 295.1)));
        assert!(reforge_button_hit(Vec2::new(134.9, 324.9)));
        assert!(!reforge_button_hit(Vec2::new(105.0, 310.0)));
        assert!(!reforge_button_hit(Vec2::new(135.0, 310.0)));
        assert!(!reforge_button_hit(Vec2::new(120.0, 295.0)));
        assert!(!reforge_button_hit(Vec2::new(120.0, 325.0)));
        // No `Main.mH` term here either — the 900-px screen's +150 offset
        // (the old port's shape) misses the literal centre.
        assert!(!reforge_button_hit(Vec2::new(120.0, 460.0)));
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
        // `Main.inventoryBack` (`ItemSlot.cs:2785`). No flash (loadout 0).
        let back = INVENTORY_BACK;
        let armor = equip_tint(10, back, 0, 0.0);
        assert!((armor[0] - 50.0 / 255.0 * back[0]).abs() < 1e-6);
        assert!(armor[1] > armor[0] && armor[1] > armor[2], "armour slot is green");
        let vanity = equip_tint(11, back, 0, 0.0);
        assert!(vanity[2] > vanity[0], "vanity slot is teal");
        let dye = equip_tint(12, back, 0, 0.0);
        assert!(dye[2] > dye[1], "dye slot is blue");
        // Contexts without a loadout colour keep the plain background.
        assert_eq!(equip_tint(16, back, 0, 0.0), back);
        assert_eq!(equip_tint(33, back, 0, 0.0), back);
    }

    /// W4-g55 — the tint follows the current loadout's palette row
    /// (`ItemSlot.LoadoutSlotColors[loadoutIndex]`, `ItemSlot.cs:2825`) and the
    /// change-flash lerps it toward white (`GetColorByLoadout:2791`).
    #[test]
    fn w4_g55_the_tint_follows_the_loadout_and_flash() {
        let back = INVENTORY_BACK;
        // Loadout 1 (blue row): the armour slot's blue channel dominates.
        let l1 = equip_tint(10, back, 1, 0.0);
        assert!(l1[2] > l1[0], "loadout 2 armour is blue");
        // Loadout 2 (red row): the armour slot's red channel dominates.
        let l2 = equip_tint(10, back, 2, 0.0);
        assert!(l2[0] > l2[1] && l2[0] > l2[2], "loadout 3 armour is red");
        // A full flash (factor 1) lerps every channel to white.
        let flashed = equip_tint(10, back, 0, 1.0);
        assert_eq!(flashed, [1.0; 4]);
        // The flash factor decays over 30 frames and is 0 when flashy is off.
        assert_eq!(loadout_flash(0.0, 0.0, true), 0.125); // 0.5^3 at elapsed 0
        assert_eq!(loadout_flash(0.0, 30.0, true), 0.0);
        assert_eq!(loadout_flash(0.0, 0.0, false), 0.0);
        assert_eq!(loadout_flash(f32::NEG_INFINITY, 100.0, true), 0.0);
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
        let base_y = equip_page_base_y(SCREEN_H, 0.0, NO_EXTRA);
        assert_eq!(base_y, 174.0, "`num20 = 174 + Main.mH` at 900 px");
        // Row 0 is the helmet (no tick); the first accessory is row 3, so its
        // tick is row 3's y minus 2, shifted by the 4 px lower block.
        let tick = tick_pos(SCREEN_W, base_y, 0, false);
        assert_eq!(tick, Vec2::new(1542.0, 172.0));
        let rects = accessory_tick_rects(
            Vec2::new(SCREEN_W, SCREEN_H),
            base_y,
            (16.0, 16.0),
            NO_EXTRA,
        );
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
        let base_y = equip_page_base_y(SCREEN_H, 0.0, NO_EXTRA);
        let r = accessory_tick_rects(screen, base_y, (16.0, 16.0), NO_EXTRA)[0].1;
        let inside = Vec2::new(r[0] + 1.0, r[1] + 1.0);
        assert_eq!(
            accessory_tick_click(screen, base_y, inside, (16.0, 16.0), true, NO_EXTRA),
            Some(3)
        );
        // The right and bottom edges are outside the rectangle.
        let edge = Vec2::new(r[0] + r[2], r[1] + 1.0);
        assert_eq!(
            accessory_tick_click(screen, base_y, edge, (16.0, 16.0), true, NO_EXTRA),
            None
        );
        let edge = Vec2::new(r[0] + 1.0, r[1] + r[3]);
        assert_eq!(
            accessory_tick_click(screen, base_y, edge, (16.0, 16.0), true, NO_EXTRA),
            None
        );
        // `Main.mouseItem.IsAir` (`:41430`): a held stack disables every tick.
        assert_eq!(
            accessory_tick_click(screen, base_y, inside, (16.0, 16.0), false, NO_EXTRA),
            None
        );
        // The armour slots 0..2 have no tick at all.
        for (idx, _) in accessory_tick_rects(screen, base_y, (16.0, 16.0), NO_EXTRA) {
            assert!(idx > 2, "slot {idx} must not carry a tick");
        }
    }

    /// The `num38 > 2` gate means the tick rows are offset by the three armour
    /// slots: row 3 of the column is the first accessory.
    #[test]
    fn the_tick_row_follows_the_slot_row() {
        let base_y = equip_page_base_y(SCREEN_H, 0.0, NO_EXTRA);
        for (idx, rect) in accessory_tick_rects(
            Vec2::new(SCREEN_W, SCREEN_H),
            base_y,
            (16.0, 16.0),
            NO_EXTRA,
        ) {
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

    /// `equip_cell_at` resolves the same cells `draw` hovers: the armour,
    /// vanity and dye columns, the misc page's two columns, and nothing on the
    /// housing page.
    #[test]
    fn equip_cell_at_matches_the_drawn_columns() {
        let screen = Vec2::new(SCREEN_W, SCREEN_H);
        let base_y = equip_page_base_y(SCREEN_H, 0.0, NO_EXTRA);
        let (armor_x, vanity_x, dye_x) = (
            equip_column_x(EquipColumn::Armor, SCREEN_W),
            equip_column_x(EquipColumn::Vanity, SCREEN_W),
            equip_column_x(EquipColumn::Dye, SCREEN_W),
        );
        // The centre of a cell is inside its own rectangle.
        let at = |x: f32, row: usize, lower: bool| {
            Vec2::new(x + 10.0, equip_row_y(base_y, row, lower, SCALE) + 10.0)
        };
        assert_eq!(
            equip_cell_at(at(armor_x, 0, false), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((8, 0))
        );
        assert_eq!(
            equip_cell_at(at(armor_x, 2, false), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((8, 2))
        );
        // `num38 > 2` switches to context 10 and the 4 px lower block.
        assert_eq!(
            equip_cell_at(at(armor_x, 3, true), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((10, 3))
        );
        assert_eq!(
            equip_cell_at(at(armor_x, 7, true), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((10, 7))
        );
        // Vanity `Player.armor[10..19]`.
        assert_eq!(
            equip_cell_at(at(vanity_x, 0, false), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((9, 10))
        );
        assert_eq!(
            equip_cell_at(at(vanity_x, 3, true), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((11, 13))
        );
        // Dyes.
        assert_eq!(
            equip_cell_at(at(dye_x, 0, false), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((12, 0))
        );
        assert_eq!(
            equip_cell_at(at(dye_x, 3, true), screen, 0.0, 0, 52.0, NO_EXTRA),
            Some((12, 3))
        );
        // Misc page: `miscEquips` at screenWidth-92, `miscDyes` one 47 left.
        let misc_x = SCREEN_W - 92.0;
        let misc = |x: f32, row: usize| Vec2::new(x + 10.0, misc_row_y(base_y, row) + 10.0);
        assert_eq!(
            equip_cell_at(
                misc(misc_x, 0),
                screen,
                0.0,
                EQUIP_PAGE_MISC,
                52.0,
                NO_EXTRA
            ),
            Some((19, 0))
        );
        assert_eq!(
            equip_cell_at(
                misc(misc_x, 4),
                screen,
                0.0,
                EQUIP_PAGE_MISC,
                52.0,
                NO_EXTRA
            ),
            Some((16, 4))
        );
        assert_eq!(
            equip_cell_at(
                misc(misc_x - MISC_PITCH, 2),
                screen,
                0.0,
                EQUIP_PAGE_MISC,
                52.0,
                NO_EXTRA
            ),
            Some((33, 2))
        );
        // The housing page answers `None` (its own cells own the pointer).
        assert_eq!(
            equip_cell_at(
                at(armor_x, 0, false),
                screen,
                0.0,
                EQUIP_PAGE_HOUSING,
                52.0,
                NO_EXTRA
            ),
            None
        );
        // Nothing outside the columns.
        assert_eq!(
            equip_cell_at(Vec2::new(10.0, 10.0), screen, 0.0, 0, 52.0, NO_EXTRA),
            None
        );
    }

    /// W4-g87 — the Demon Heart's slot-8 row JOINS the columns once the
    /// unlock shows it (`Main.cs:41381`/`:41398` flag4): the armour column
    /// gains a 9th drawn row (index 8, drawn row 8), the tick column a 6th
    /// tick, and the defense anchor drops one row (`num35 = num19 - 1 = 8`).
    #[test]
    fn w4_g87_the_demon_heart_row_joins_the_columns() {
        let screen = Vec2::new(SCREEN_W, SCREEN_H);
        let base_y = equip_page_base_y(SCREEN_H, 0.0, DH_EXTRA);
        assert_eq!(base_y, 174.0, "9 rows < 10: no collapse at 900 px");
        let rects = accessory_tick_rects(screen, base_y, (16.0, 16.0), DH_EXTRA);
        assert_eq!(rects.len(), 6, "armour 3..=8 carry the ticks");
        assert_eq!(rects[5].0, 8, "the Demon Heart row is `Player.armor[8]`");
        // Row 8: `(int)(172 + 8*47.6) + 4` = 552 + 4.
        assert_eq!(rects[5].1[1], 556.0);
        // The cell hit resolves row 8 in all three columns.
        let armor_x = equip_column_x(EquipColumn::Armor, SCREEN_W);
        let vanity_x = equip_column_x(EquipColumn::Vanity, SCREEN_W);
        let dye_x = equip_column_x(EquipColumn::Dye, SCREEN_W);
        let at =
            |x: f32, row: usize| Vec2::new(x + 10.0, equip_row_y(base_y, row, true, SCALE) + 10.0);
        assert_eq!(
            equip_cell_at(at(armor_x, 8), screen, 0.0, 0, 52.0, DH_EXTRA),
            Some((10, 8))
        );
        assert_eq!(
            equip_cell_at(at(vanity_x, 8), screen, 0.0, 0, 52.0, DH_EXTRA),
            Some((11, 18))
        );
        assert_eq!(
            equip_cell_at(at(dye_x, 8), screen, 0.0, 0, 52.0, DH_EXTRA),
            Some((12, 8))
        );
        // Slot 9 stays hidden (no master mode).
        assert_eq!(
            equip_cell_at(at(armor_x, 9), screen, 0.0, 0, 52.0, DH_EXTRA),
            None
        );
        // The tick click flips index 8's hide flag target.
        let r = rects[5].1;
        assert_eq!(
            accessory_tick_click(
                screen,
                base_y,
                Vec2::new(r[0] + 1.0, r[1] + 1.0),
                (16.0, 16.0),
                true,
                DH_EXTRA
            ),
            Some(8)
        );
    }

    /// W4-g87 — a SHOWN-but-LOCKED row (a stored item, no unlock: the
    /// `CanDemonHeartAccessoryBeShown` item legs) keeps the dim background
    /// (`Main.cs:41452` flag6 false → color2) while an unlocked row keeps
    /// `Main.inventoryBack`; the classic rows are always unlocked.
    #[test]
    fn w4_g87_a_shown_locked_row_dims() {
        let stored = ExtraAccessoryShow {
            demon_heart: true, // shown by the stored item
            master: false,
            slot8_unlocked: false, // but NOT unlocked (classic world)
            slot9_unlocked: false,
        };
        assert!(!stored.hides(8), "the stored row shows");
        assert!(!stored.slot_unlocked(8), "but it is locked → dim");
        assert!(stored.slot_unlocked(3), "the classic rows stay unlocked");
        assert!(DH_EXTRA.slot_unlocked(8), "the used-heart row is unlocked");
        assert!(
            !DH_EXTRA.slot_unlocked(9),
            "slot 9 stays locked (no master)"
        );
    }
}
