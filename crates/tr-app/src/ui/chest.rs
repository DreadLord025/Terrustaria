//! The chest panel: the draw half of `Terraria/UI/ChestUI.cs` (1.4.5.8) — the
//! name line, the rename text buttons and the 10×4 slot window over the open
//! world chest. There is no panel background texture: the panel is text and
//! `Inventory_Back*` slot frames drawn over the inventory area (the research
//! doc §3; grep: no `Chest_Container` asset exists).
//!
//! State shape: the C# panel reads `Main.*`/`Chest` statics; the port threads
//! them through [`PanelState`], built once per frame at the `draw_hud` call
//! site (`app/frame.rs`). The click dispatch inside `ChestUI.DrawButton`
//! (`:281-308`) is `app/clicks.rs`'s — the rename pair (5/6) landed with
//! phase 5, the M3-STORE-2 transfers (0-3) with M3-CHEST-INT; the per-slot
//! `ItemSlot.Handle` hover dispatch (`:385`) is phase 4. The inline-edit
//! caret arm of `DrawName` (`:165-184`) is phase 5's rename flow.
//!
//! Out of this phase: the bank contexts (4/32, M3-STORE-1), Sort/ItemSorting
//! (anchor 4 stays label-less), the quick-stack glow
//! (`ItemSlot.PrepareForChest`, `ItemSlot.cs:467-481` — a glow-array resize
//! the port's glow system does not carry), tooltips (M7-UI-6).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::{slot, text};
use crate::assets;
use crate::audio;
use crate::chest::DEFAULT_MAX_ITEMS;
use crate::item::{EMPTY_SLOT, ItemSlot};

/// `ChestUI.ButtonID.Count` (`ChestUI.cs:36-41`): LootAll 0, DepositAll 1,
/// QuickStack 2, Restock 3, Sort 4, RenameChest 5, RenameChestCancel 6.
pub(crate) const BUTTON_COUNT: usize = 7;
/// `ChestUI.ButtonID.RenameChest` (`ChestUI.cs:32`).
pub(crate) const BUTTON_RENAME: usize = 5;
/// `ChestUI.ButtonID.RenameChestCancel` (`ChestUI.cs:34`).
pub(crate) const BUTTON_RENAME_CANCEL: usize = 6;

/// `Main.instance.invBottom` (`Main.cs:6608`, set once at world load) — the
/// panel's base row. The port's trash slot reads the same constant
/// (`ui/inventory.rs` `TRASH_Y`, `Main.cs:42353`).
const INV_BOTTOM: f32 = 258.0;
/// `ChestUI.cs:88`/`:363`: `Main.inventoryScale = 0.755f` for the whole panel.
const CHEST_SCALE: f32 = 0.755;
/// `ChestUI.cs:331-332`: `num = 10` columns, `num2 = 4` visible rows.
const COLS: usize = 10;
const VISIBLE_ROWS: usize = 4;
/// `ChestUI.cs:379`: the grid origin x; the 56 px layout pitch rides in the
/// position formulas (`:379-380`).
const GRID_X: f32 = 73.0;
const PITCH: f32 = 56.0;
/// `ChestUI.cs:44-46`: `buttonScaleMinimum` / `buttonScaleMaximum`.
pub(crate) const BUTTON_SCALE_MIN: f32 = 0.75;
const BUTTON_SCALE_MAX: f32 = 1.0;
/// `Main.cs:66748`: `Main.OurFavoriteColor = new Color(255, 231, 69)` — the
/// hovered button's tint (`ChestUI.cs:243`).
const OUR_FAVORITE_COLOR: [f32; 4] = [1.0, 231.0 / 255.0, 69.0 / 255.0, 1.0];
/// The retail `Inventory_Back` sheet is 52×52 — the hit-test cell the C#
/// derives from `TextureAssets.InventoryBack.Width()` (`ChestUI.cs:382`; the
/// `52.0` fallback is `ui/slot.rs`'s `SLOT_FALLBACK_PX` precedent).
const CELL_PX: f32 = 52.0;
/// `Main.cs:67274`: `Main.inventoryBack = new Color(220, 220, 220, 220)` — the
/// slot background tint (`ItemSlot.cs`'s `color2` default). Shared with the
/// inventory panel (`ui/inventory.rs` `INVENTORY_BACK`).
const SLOT_TINT: [f32; 4] = [220.0 / 255.0; 4];

/// The `Main.*`/`Chest` state `ChestUI.Draw` reads, threaded from the App
/// (the `draw_hud` call site): the open chest's index/origin/tile, its slots,
/// the button/scroll state and the audio for the Menu_Tick cues. The plan's
/// `draw(app)` shape collapses into this bundle for the same borrow reason
/// phase 2 recorded — the draw site borrows `App` fields disjointly, and the
/// per-frame mutable button state cannot re-enter `&mut App` from inside
/// `draw_hud`.
pub(crate) struct PanelState<'a> {
    /// `Main.player[Main.myPlayer].chest` (`Player.cs:2876`) — the unified
    /// open-container index (M3-CHEST-INT): `-1` closed, `-2..-5` the banks
    /// (whose panel is the STORE-1 row — this one draws for `>= 0` only),
    /// `>= 0` the world-chest store index.
    pub(crate) chest: i32,
    /// `Player.chestX/chestY` (`Player.cs:2878`/:2880`) — the 2×2's origin.
    pub(crate) origin: (i32, i32),
    /// The active tile at the origin: `(kind, frameX)` — the name default's
    /// lookup piece (`ChestUI.cs:124-139`).
    pub(crate) tile: Option<(u16, i16)>,
    /// `Main.chest[chest].item` while a world chest is open (`chest > -1`) —
    /// full `ItemSlot`s since the M3-CHEST-INT upgrade. The bank slots
    /// (`chest < -1`) are M3-STORE-1's.
    pub(crate) slots: Option<&'a [ItemSlot]>,
    /// `NewCraftingUI.Visible` — the covering-crafting gate (`ChestUI.cs:86`).
    pub(crate) crafting_open: bool,
    /// `Main.editChest` (`Main.cs:1747`) — the live chest-rename editor (the
    /// port's shape: the chest-rename keyboard session is open).
    pub(crate) editing: bool,
    /// `Main.chest[chest].name` while a world chest is open (`ChestUI.cs:
    /// 115-122`) — the `WorldChest` entity's own name (the M6-SAVE-1 phase-3
    /// fold retired the `App.chest_names` side map), "" until renamed. A
    /// `.wld` load restores it from the file.
    pub(crate) name: &'a str,
    /// `Main.chest[chest].maxItems` (`Chest.cs:78`) — the per-chest slot count
    /// the grid window and the scroll clamp measure (`ChestUI.cs:376-377`):
    /// the file's own value for a loaded chest, the default 40 otherwise.
    pub(crate) max_items: u32,
    /// `Main.npcChatText` while the editor is live (`ChestUI.cs:113` — the
    /// keyboard's live text, `UIVirtualKeyboard.cs:1061-1067`'s sync).
    pub(crate) edit_text: &'a str,
    /// `Main.instance.textBlinkerCount/State` (`Main.cs:2287-2289`) — the
    /// DrawName caret's blinker, advanced only while the editor draws.
    pub(crate) blinker: &'a mut TextBlinker,
    /// `ChestUI.StartingRowForDrawing` (`ChestUI.cs:52`).
    pub(crate) scroll_row: &'a mut i32,
    /// `ChestUI.ButtonScale` (`ChestUI.cs:48`; the static ctor's
    /// `new float[7]` starts at **0.0**, `:1306` — the first drawn frame sits
    /// at 0 before `UpdateHover` clamps to 0.75, kept 1:1).
    pub(crate) button_scales: &'a mut [f32; BUTTON_COUNT],
    /// `ChestUI.ButtonHovered` (`ChestUI.cs:50`).
    pub(crate) button_hovered: &'a mut [bool; BUTTON_COUNT],
    /// The `SoundEngine.PlaySound(12)` cues (`ChestUI.cs:64`/:325).
    pub(crate) audio: &'a mut audio::Audio,
}

/// `ChestUI.Draw` (`ChestUI.cs:84-105`): the whole panel runs while a chest
/// is open and the crafting panel is closed (`!NewCraftingUI.Visible`; the
/// gamepad `PipsUseGrid` arm is unported). Closed — or covered — every
/// button decays back to its idle state (`:99-104`). The C# gate is
/// `chest != -1` for every container kind; the port's panel is the
/// world-chest surface (the STORE-1 two-panel shape — the banks keep their
/// `ui/inventory.rs` row), so the gate is `chest >= 0` and the bank values
/// fall to the idle reset.
/// The panel's visibility gate (`ChestUI.cs:86`'s `chest != -1` read through
/// the port's two-panel ownership, Phase-1 decision #2): the world-chest
/// surface draws (and dispatches) for `chest >= 0` only, and never under the
/// crafting panel — the bank values fall to the idle reset.
pub(crate) fn panel_visible(chest: i32, crafting_open: bool) -> bool {
    chest >= 0 && !crafting_open
}

pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mut p: PanelState<'_>,
    pulse: super::UiPulse,
    mouse: Vec2,
) {
    if panel_visible(p.chest, p.crafting_open) {
        draw_name(r, lookups, &mut p, pulse);
        draw_buttons(r, lookups, &mut p, pulse, mouse);
        draw_slots(r, lookups, &mut p);
    } else {
        // `ChestUI.cs:99-104` — the closed reset.
        for i in 0..BUTTON_COUNT {
            p.button_scales[i] = BUTTON_SCALE_MIN;
            p.button_hovered[i] = false;
        }
    }
}

/// `Main.instance.textBlinkerCount/State` (`Main.cs:2287-2289`) — the caret
/// blinker the DrawName edit arm advances (the port's chat keeps its own
/// `blink_timer`, `ui/mod.rs:410`; the chest editor owns this one).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TextBlinker {
    /// `textBlinkerCount`.
    pub(crate) count: u32,
    /// `textBlinkerState` — the caret draws while `true` (`:180`).
    pub(crate) on: bool,
}

impl TextBlinker {
    /// `ChestUI.cs:175-179`: `if (++textBlinkerCount >= 20) { toggle
    /// textBlinkerState; textBlinkerCount = 0; }` — the 20-frame half-period
    /// of the 40-frame blink.
    pub(crate) fn advance(&mut self) {
        self.count += 1;
        if self.count >= 20 {
            self.on = !self.on;
            self.count = 0;
        }
    }
}

/// `ChestUI.DrawName`'s text chain (`ChestUI.cs:110-141`): the live edit text
/// while `Main.editChest` (:111-114), else the stored `Chest.name`, else the
/// tile-style default (:115-141).
pub(crate) fn draw_name_text(
    editing: bool,
    edit_text: &str,
    name: &str,
    tile: Option<(u16, i16)>,
) -> String {
    if editing {
        return edit_text.to_string();
    }
    if !name.is_empty() {
        return name.to_string(); // `:115-122`
    }
    match tile {
        Some((kind, frame_x)) => crate::chest::default_chest_name(kind, frame_x),
        None => String::new(),
    }
}

/// `ChestUI.DrawName` (`ChestUI.cs:107-185`): the chest's name line at
/// (504, invBottom) — the stored name, else the tile-style default, and the
/// inline edit text with the blinking caret while `Main.editChest` is live.
fn draw_name(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    p: &mut PanelState<'_>,
    pulse: super::UiPulse,
) {
    let text = draw_name_text(p.editing, p.edit_text, p.name, p.tile);
    // `ChestUI.cs:159-161`: `Color.White * (1f - (255f - mouseTextColor) / 255f
    // * 0.5f)` with the alpha forced back to opaque — `pulse.text` is
    // `Main.mouseTextColor / 255`.
    let f = 1.0 - (1.0 - pulse.text) * 0.5;
    // `ChestUI.cs:163-164`: `DrawColorCodedStringWithShadow(..., (504, invBottom),
    // origin zero, scale one, spread 1.5)`.
    hud_text_shadowed(r, lookups, 504.0, INV_BOTTOM, 1.0, [f, f, f, 1.0], &text);
    if p.editing {
        // `ChestUI.cs:165-184` — the caret after the text (`vector.X +=
        // MeasureString(text).X`); the IME panel/composition arms (:168-174)
        // are out per plan. The blinker advances once per editing draw
        // (:175-179) and the caret draws while `textBlinkerState == 1`
        // (:180-183) in the text's own colour.
        p.blinker.advance();
        if p.blinker.on {
            let x = 504.0 + text::hud_text_width(r, lookups, &text, 1.0);
            hud_text_shadowed(r, lookups, x, INV_BOTTOM, 1.0, [f, f, f, 1.0], "|");
        }
    }
}

/// `ChestUI.DrawButtons` (`ChestUI.cs:187-193`): every button anchor sits at
/// (506, invBottom + 40) plus `ID * 26` rows (`:203`).
fn draw_buttons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    p: &mut PanelState<'_>,
    pulse: super::UiPulse,
    mouse: Vec2,
) {
    for id in 0..BUTTON_COUNT {
        draw_button(r, lookups, p, pulse, mouse, id, 506.0, INV_BOTTOM + 40.0);
    }
}

/// `ChestUI.DrawButton` (`ChestUI.cs:195-309`): one text button — the
/// visibility rules (`:198-202`), the centred label, the hover rect + the
/// `OurFavoriteColor` tint (`:236-243`) and the `UpdateHover` animation. The
/// click dispatch (`:281-308`) is `app/clicks.rs`'s (the rename pair landed
/// with phase 5, the M3-STORE-2 transfers with M3-CHEST-INT).
fn draw_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    p: &mut PanelState<'_>,
    pulse: super::UiPulse,
    mouse: Vec2,
    id: usize,
    x: f32,
    y: f32,
) {
    // `ChestUI.cs:198-202`: the rename pair's visibility — rename hides for
    // the bank slots (`chest < -1`, M3-STORE-1) and cancel hides unless the
    // rename editor is live (phase 5's `Main.editChest`).
    if !button_visible(id, p.chest, p.editing) {
        update_hover(p, id, false);
        return;
    }
    // The label-less anchors (Sort 4, `inter[122]` — out of scope) keep the
    // hover/scale state alive but draw nothing.
    let y = y + id as f32 * 26.0; // `ChestUI.cs:203`
    let Some(label) = button_label(id, p.editing) else {
        update_hover(p, id, false);
        return;
    };
    let num = p.button_scales[id];
    // `ChestUI.cs:230`: `MeasureString(text)`.
    let w = text::hud_text_width(r, lookups, label, 1.0);
    let h = font_height(lookups);
    // `ChestUI.cs:231-233`: `Color.White * 0.97f * (1 - (255 - mouseTextColor)
    // / 255 * 0.5)`, alpha forced opaque.
    let f = 0.97 * (1.0 - (1.0 - pulse.text) * 0.5);
    let mut color = [f, f, f, 1.0];
    // `ChestUI.cs:234-235`: `num2 = (int)(vector.X * num / 2)` — the X advance
    // and the `origin = vector / 2` draw cancel, so the label's left edge sits
    // on x=506 and it is vertically centred on the row anchor.
    let num2 = (w * num / 2.0) as i32 as f32;
    let x = x + num2;
    // `ChestUI.cs:236-240`: the hover rect, widened once already hovered (the
    // re-hover rect keeps the edge-hugging cursor on the button).
    let rect = if p.button_hovered[id] {
        [x - num2 - 10.0, y - 12.0, w * num + 16.0, 24.0]
    } else {
        [x - num2, y - 12.0, w * num, 24.0]
    };
    let hovering = float_contains(mouse, rect);
    if hovering {
        color = OUR_FAVORITE_COLOR; // `ChestUI.cs:243`
    }
    // `ChestUI.cs:245`: the centred draw — top-left
    // `(X, Y) - vector/2 * num`.
    hud_text_shadowed(
        r,
        lookups,
        x - w * num / 2.0,
        y - h * num / 2.0,
        num,
        color,
        label,
    );
    // `ChestUI.cs:271-276` + `UpdateHover` (`:58-82`): the hover-start edge
    // plays 12 (`SoundID.MenuTick`, `SoundID.cs:46`). (The click dispatch
    // `:282-308` joins phase 5 / M3-STORE-2.)
    let sound = update_hover(p, id, hovering);
    if sound != 0 {
        p.audio.play("Menu_Tick");
    }
}

/// `ChestUI.DrawSlots` (`ChestUI.cs:329-391`): the 10×4 slot window over the
/// chest's items — context 3, the `(int)`-truncated cell positions, the
/// per-frame scroll clamp (`:368-370`) and the `num4 < item.Length` bound
/// (`:377`; the array length is [`DEFAULT_MAX_ITEMS`] until M6-SAVE-2's
/// `Resize`, decision #3).
fn draw_slots(r: &mut SpriteRenderer, lookups: &assets::Lookups, p: &mut PanelState<'_>) {
    // The bank slots (`chest == -2..-5`) are M3-STORE-1's — the port never
    // carries their `slots` yet.
    let Some(slots) = p.slots else {
        return;
    };
    // `ChestUI.cs:368-370`: the per-frame window clamp.
    *p.scroll_row = clamp_scroll_row(*p.scroll_row, p.max_items);
    for i in 0..COLS {
        for j in 0..VISIBLE_ROWS {
            // `ChestUI.cs:376-377`: the absolute chest slot; `item.Length` is
            // the chest's own `maxItems` (`p.max_items`) — the 40-slot default
            // until a `.wld` record resizes it.
            let abs = i + j * COLS + *p.scroll_row as usize * COLS;
            if abs >= p.max_items as usize {
                continue;
            }
            let pos = slot_pos(i, j);
            // The store slot IS the draw view since the M3-CHEST-INT upgrade —
            // the fabricated view is gone; the implicit airs beyond
            // `slots.len()` are the C#'s air items (the `chest.rs` indexed
            // encoding).
            let view = slots.get(abs).copied().unwrap_or(EMPTY_SLOT);
            slot::draw_slot(
                r,
                lookups,
                &view,
                pos.x,
                pos.y,
                3,
                abs as i32,
                false,
                CHEST_SCALE,
                SLOT_TINT,
            );
        }
    }
}

/// `ChestUI.UpdateHover` (`ChestUI.cs:58-82`): the ±0.05 scale animation
/// between [`BUTTON_SCALE_MIN`] and [`BUTTON_SCALE_MAX`] and the hover-start
/// `SoundEngine.PlaySound(12)` (`:64`). Returns the legacy sound id the C#
/// plays — 12 on the hover edge, 0 for silence — so the caller plays it and
/// the tests pin it (the phase-2 `toggle_open` pattern).
fn update_hover(p: &mut PanelState<'_>, id: usize, hovering: bool) -> u32 {
    if hovering {
        let edge = !p.button_hovered[id]; // `:62-64` — the hover-start arm
        p.button_hovered[id] = true;
        p.button_scales[id] = (p.button_scales[id] + 0.05).min(BUTTON_SCALE_MAX); // `:67-71`
        if edge {
            return 12; // `:64` — `SoundEngine.PlaySound(12)`
        }
        0
    } else {
        p.button_hovered[id] = false;
        p.button_scales[id] = (p.button_scales[id] - 0.05).max(BUTTON_SCALE_MIN); // `:76-80`
        0
    }
}

/// `Main.DoScrollingInInventory`'s chest arm (`Main.cs:17435-17452`) +
/// `ChestUI.Scroll` (`ChestUI.cs:311-327`): wheel notches scroll the slot
/// window while a chest is open, the window can move (`LastHighestChestRow >
/// 0`) and the pointer is inside [`display_rectangle`]; a changed row plays
/// 12 (`:325`). Returns whether the wheel was consumed — the C# `flag = false`
/// that stops the crafting-list scroll (`Main.cs:17443`).
pub(crate) fn scroll(app: &mut crate::app::App, notches: i32, mouse: Vec2) -> bool {
    // `Main.cs:17436` — `chest != -1 && ChestUI.LastHighestChestRow > 0`. The
    // row count is a pure function of the open chest's `maxItems`
    // (`ChestUI.cs:301-302`'s `Main.chest[chest].maxItems`) — the default 40
    // makes the gate dead, exactly like the C#'s for a full-size chest. The
    // port's scrollable grid is the world-chest panel's — bank values refuse
    // (the STORE-1 two-panel shape, `ui::chest::draw`'s gate).
    let max_items = crate::chest::chest_max_items(&app.world_chests, app.open_container());
    if app.open_container() < 0 || highest_scroll_row(max_items) <= 0 {
        return false;
    }
    // `Main.cs:17442` — `LastChestDisplayRectangle.Contains(MouseScreen)`.
    if !super::in_rect(mouse, display_rectangle()) {
        return false;
    }
    // `ChestUI.cs:313-321`: wheel up scrolls up one row per notch (the C#
    // `while num != 0 { Scroll(num); num -= sign }` loop — only the sign
    // reaches `Scroll`).
    for _ in 0..notches.unsigned_abs() {
        if scroll_notch(&mut app.chest_scroll_row, notches > 0, max_items) {
            // `ChestUI.cs:325` — `SoundEngine.PlaySound(12)`.
            app.audio.play("Menu_Tick");
        }
    }
    true // `Main.cs:17443` — `flag = false`
}

/// `ChestUI.Scroll` (`ChestUI.cs:311-327`) for one wheel notch: ∓1 row,
/// clamped to `0..=ceil(maxItems/10) - 4`; `true` when the row changed (the
/// caller plays 12). `max_items` rides in for the tests and M6-SAVE-2's
/// `Resize`.
fn scroll_notch(row: &mut i32, wheel_up: bool, max_items: u32) -> bool {
    let before = *row;
    *row += if wheel_up { -1 } else { 1 };
    *row = (*row).clamp(0, highest_scroll_row(max_items));
    *row != before
}

/// `ChestUI.cs:364`: `LastChestDisplayRectangle = new Rectangle(73,
/// invBottom, (int)(10*56*scale), (int)(4*56*scale))` — the wheel gate and
/// phase 4's click area. `[x, y, w, h]`.
pub(crate) fn display_rectangle() -> [f32; 4] {
    [
        GRID_X,
        INV_BOTTOM,
        (COLS as f32 * PITCH * CHEST_SCALE) as i32 as f32,
        (VISIBLE_ROWS as f32 * PITCH * CHEST_SCALE) as i32 as f32,
    ]
}

/// `ChestUI.cs:376-386`: the grid hit-test — the absolute chest slot under the
/// cursor, `None` off-grid. The C# tests the drawn (truncated) cells, unlike
/// the inventory panel's untruncated math (`app/clicks.rs:24-34`).
pub(crate) fn slot_at(mouse: Vec2, scroll_row: i32) -> Option<usize> {
    for i in 0..COLS {
        for j in 0..VISIBLE_ROWS {
            let abs = i + j * COLS + scroll_row as usize * COLS;
            if abs >= DEFAULT_MAX_ITEMS as usize {
                continue;
            }
            let p = slot_pos(i, j);
            // `ChestUI.cs:382`: `Utils.FloatIntersect` against the drawn cell
            // (`InventoryBack.Size() * scale` — the 52×52 retail sheet).
            if float_contains(
                mouse,
                [p.x, p.y, CELL_PX * CHEST_SCALE, CELL_PX * CHEST_SCALE],
            ) {
                return Some(abs);
            }
        }
    }
    None
}

/// `ChestUI.cs:236-240` + the click dispatch `:281-308`: the button row's
/// hit-test for `handle_chest_clicks` — every visible, labelled button's rect
/// (the transfers 0-3, the rename pair 5/6; Sort 4 stays label-less and takes
/// no clicks). Generalised from the rename pair's test (the same rect +
/// widen rules, `:236-240`): the C# answers the possibly widened re-hover
/// rect (`flag`) per visible button, in the `DrawButtons` id order.
pub(crate) fn chest_button_hit(
    lookups: &assets::Lookups,
    mouse: Vec2,
    scales: &[f32; BUTTON_COUNT],
    hovered: &[bool; BUTTON_COUNT],
    chest: i32,
    editing: bool,
) -> Option<usize> {
    for id in 0..BUTTON_COUNT {
        if !button_visible(id, chest, editing) {
            continue; // `ChestUI.cs:198-202`
        }
        let Some(label) = button_label(id, editing) else {
            continue; // the label-less anchors (Sort 4) take no clicks
        };
        let w = button_text_width(lookups, label);
        let (_, y) = button_anchor(id);
        let num = scales[id];
        // `ChestUI.cs:236-240`: `(X - num2, Y - 12, vector.X * num, 24)` — the
        // X advance cancels against the centred draw, so the rect's left edge
        // is x=506; the already-hovered rect widens by (−10, +16) (`:239`).
        let rect = if hovered[id] {
            [506.0 - 10.0, y - 12.0, w * num + 16.0, 24.0]
        } else {
            [506.0, y - 12.0, w * num, 24.0]
        };
        if float_contains(mouse, rect) {
            return Some(id);
        }
    }
    None
}

// --- Shared pieces -----------------------------------------------------------

/// `ChestUI.cs:191`/`:203`: the button anchor — `(506, invBottom + 40 +
/// ID * 26)`.
fn button_anchor(id: usize) -> (f32, f32) {
    (506.0, INV_BOTTOM + 40.0 + id as f32 * 26.0)
}

/// `ChestUI.cs:198-202`: the rename pair's visibility — rename hides for the
/// bank slots (`chest < -1`, M3-STORE-1), cancel hides unless the rename
/// editor is live.
fn button_visible(id: usize, chest: i32, editing: bool) -> bool {
    !((id == BUTTON_RENAME && chest < -1) || (id == BUTTON_RENAME_CANCEL && !editing))
}

/// `ChestUI.cs:205-229`: the text buttons' labels. The transfers 0-3 are the
/// `Lang.inter[29]/[30]/[31]/[82]` en-US strings the bank row already
/// hardcodes (`ui/inventory.rs`'s M3-STORE-1 row — `:209`/:212/:215/:218);
/// Sort (4, `inter[122]` :227) stays label-less — ItemSorting is out of
/// scope (the plan's OUT). The rename strings are the vanilla en-US
/// localization (`Lang.inter[Main.editChest ? 47 : 61]`/`Lang.inter[63]`,
/// `:221`/:224 — inter[47] "Save", inter[61] "Rename", inter[63] "Cancel"),
/// carried like the port's hardcoded "Inventory"/"Coins"/"Ammo" labels.
fn button_label(id: usize, editing: bool) -> Option<&'static str> {
    match id {
        // `Lang.inter[29]` (:209).
        0 => Some("Loot All"),
        // `Lang.inter[30]` (:212).
        1 => Some("Deposit All"),
        // `Lang.inter[31]` (:215).
        2 => Some("Quick Stack"),
        // `Lang.inter[82]` (:218).
        3 => Some("Restock"),
        // `Lang.inter[Main.editChest ? 47 : 61]` (:221).
        BUTTON_RENAME => Some(if editing { "Save" } else { "Rename" }),
        // `Lang.inter[63]` (:224).
        BUTTON_RENAME_CANCEL => Some("Cancel"),
        _ => None,
    }
}

/// `FontAssets.MouseText.Value.MeasureString(text).Y` (`ChestUI.cs:230`) — the
/// port's font metrics carry the line spacing; the bitmap-font fallback (the
/// PNG-only screenshot runs) reads a nominal 20 px line.
fn font_height(lookups: &assets::Lookups) -> f32 {
    lookups.ui_font.as_ref().map_or(20.0, |f| f.line_spacing)
}

/// `ChestUI.cs:230` (`MeasureString`) without a renderer — the hit tests read
/// the font directly so event-time routing (phase 5) needs no draw pass; the
/// bitmap-font fallback estimates the built-in advance.
fn button_text_width(lookups: &assets::Lookups, label: &str) -> f32 {
    match lookups.ui_font.as_ref() {
        Some(f) => text::font_width(f, label, 1.0),
        None => 6.0 * label.chars().count() as f32,
    }
}

/// `ChestUI.cs:368-370`: the per-frame window clamp —
/// `Utils.Clamp(StartingRowForDrawing, 0, ceil(maxItems/10) - 4)`.
fn clamp_scroll_row(row: i32, max_items: u32) -> i32 {
    row.clamp(0, highest_scroll_row(max_items))
}

/// `ChestUI.cs:368`: `num3 = (int)Math.Max(0.0, Math.Ceiling(maxItems / 10f)
/// - 4.0)` — the last scroll row (0 while every row fits on screen). Reads
/// `maxItems` dynamically (decision #3).
fn highest_scroll_row(max_items: u32) -> i32 {
    ((max_items as f32 / COLS as f32).ceil() - VISIBLE_ROWS as f32).max(0.0) as i32
}

/// `ChestUI.cs:379-380`: a slot cell's top-left — `(int)(73 + i*56*scale)` and
/// `(int)(invBottom + j*56*scale)`.
fn slot_pos(i: usize, j: usize) -> Vec2 {
    Vec2::new(
        (GRID_X + i as f32 * PITCH * CHEST_SCALE) as i32 as f32,
        (INV_BOTTOM + j as f32 * PITCH * CHEST_SCALE) as i32 as f32,
    )
}

/// `Utils.FloatIntersect` with the zero-size mouse rect (`ChestUI.cs:89`,
/// `:236`, `:382`): a degenerate rect touching the edge still intersects, so
/// the test is inclusive on every edge — unlike `ui::in_rect`'s
/// `Rectangle.Contains`.
fn float_contains(mouse: Vec2, rect: [f32; 4]) -> bool {
    mouse.x >= rect[0]
        && mouse.x <= rect[0] + rect[2]
        && mouse.y >= rect[1]
        && mouse.y <= rect[1] + rect[3]
}

/// `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333-353`): four
/// black-shadowed copies at the ±1.5 px offsets (the C# `spread` argument),
/// then the text — the same structure as `ui/slot.rs`'s item-font twin; the
/// shared `text.rs` helper is a phase-6 cleanup candidate (`text.rs` is
/// outside this phase's file scope).
fn hud_text_shadowed(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    txt: &str,
) {
    let shadow = [0.0, 0.0, 0.0, color[3]];
    for (dx, dy) in [(-1.0f32, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        text::hud_text(r, lookups, x + dx * 1.5, y + dy * 1.5, scale, shadow, txt);
    }
    text::hud_text(r, lookups, x, y, scale, color, txt);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    #[test]
    fn dw_3_2_row_scroll_math_from_max_items() {
        // `ChestUI.cs:368`: `(int)Math.Max(0.0, Math.Ceiling(maxItems / 10f) - 4.0)`.
        // 40 → 0 scroll; 80 → the clamp bound 4; the boundary 41 → 1.
        assert_eq!(highest_scroll_row(40), 0);
        assert_eq!(highest_scroll_row(41), 1);
        assert_eq!(highest_scroll_row(80), 4);
        assert_eq!(highest_scroll_row(1), 0);
        assert_eq!(highest_scroll_row(0), 0);
        // `ChestUI.cs:369` / `:322`: `Utils.Clamp(row, 0, num3)` — the draw
        // and the wheel share the clamp.
        assert_eq!(clamp_scroll_row(-1, 80), 0);
        assert_eq!(clamp_scroll_row(9, 80), 4);
        assert_eq!(clamp_scroll_row(4, 80), 4);
        // The default 40-item chest never scrolls.
        assert_eq!(clamp_scroll_row(3, 40), 0);
        assert_eq!(clamp_scroll_row(-5, 40), 0);
    }

    #[test]
    fn dw_3_2_scroll_notch_moves_and_clamps_the_window() {
        // `ChestUI.cs:311-327` — one wheel notch per call, clamped, `true`
        // only when the row moved (the sound condition).
        let mut row = 0;
        assert!(scroll_notch(&mut row, false, 80)); // wheel down → row 1
        assert_eq!(row, 1);
        assert!(scroll_notch(&mut row, true, 80)); // wheel up → row 0
        assert_eq!(row, 0);
        assert!(!scroll_notch(&mut row, true, 80)); // the top clamp: no move
        assert_eq!(row, 0);
        let mut row = 4;
        assert!(!scroll_notch(&mut row, false, 80)); // the bottom clamp
        assert_eq!(row, 4);
        assert!(scroll_notch(&mut row, true, 80));
        assert_eq!(row, 3);
        // A full default 40-item chest cannot scroll (`:368`'s bound is 0).
        let mut row = 0;
        assert!(!scroll_notch(&mut row, false, 40));
        assert_eq!(row, 0);
    }

    #[test]
    fn dw_3_2_scroll_wheel_gate_matches_doscrollingininventory() {
        // `Main.cs:17435-17452`: the gate is `chest != -1 &&
        // LastHighestChestRow > 0` and the pointer inside the grid rect;
        // with the default 40-item chest the row gate is dead, so the wheel
        // is not consumed even inside the rect (it falls through to the
        // crafting list, `flag` stays true).
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let inside = Vec2::new(200.0, 300.0);
        assert!(!scroll(&mut app, 1, inside)); // chest == -1
        app.player.as_mut().unwrap().chest = 0;
        assert!(!scroll(&mut app, 1, inside)); // LastHighestChestRow == 0
        assert_eq!(app.chest_scroll_row, 0);
        // The rect gate would reject an outside pointer even with rows.
        assert!(!scroll(&mut app, 1, Vec2::new(600.0, 300.0)));
    }

    #[test]
    fn dw_3_4_rename_buttons_render_at_id_slot_positions() {
        // `ChestUI.cs:191`/`:203`: x=506, y = invBottom + 40 + ID * 26.
        assert_eq!(
            button_anchor(BUTTON_RENAME),
            (506.0, 258.0 + 40.0 + 5.0 * 26.0)
        );
        assert_eq!(
            button_anchor(BUTTON_RENAME_CANCEL),
            (506.0, 258.0 + 40.0 + 6.0 * 26.0)
        );
        // The click rects — idle scale 0.75, the left edge on x=506 (the
        // centring advance cancels), 24 px tall centred on the anchor row.
        let scales = [BUTTON_SCALE_MIN; BUTTON_COUNT];
        let idle = [false; BUTTON_COUNT];
        // The headless test has no retail font; `button_text_width` falls
        // back to the estimate, so the probe point sits a few px in.
        let hit = |mouse, hovered: &[bool; BUTTON_COUNT], editing| {
            chest_button_hit(
                &assets::Lookups::default(),
                mouse,
                &scales,
                hovered,
                0,
                editing,
            )
        };
        assert_eq!(
            hit(Vec2::new(510.0, 428.0), &idle, false),
            Some(BUTTON_RENAME)
        );
        assert_eq!(hit(Vec2::new(510.0, 454.0), &idle, false), None);
        // The cancel row answers only while the rename editor is live.
        assert_eq!(
            hit(Vec2::new(510.0, 454.0), &idle, true),
            Some(BUTTON_RENAME_CANCEL)
        );
        // The already-hovered rect widens by (−10, +16) (`ChestUI.cs:239`).
        let mut hovered = [false; BUTTON_COUNT];
        hovered[BUTTON_RENAME] = true;
        assert_eq!(
            hit(Vec2::new(497.0, 428.0), &hovered, false),
            Some(BUTTON_RENAME)
        );
        assert_eq!(hit(Vec2::new(497.0, 428.0), &idle, false), None);
    }

    /// DW-3.5 — the transfer buttons 0-3 draw the `Lang.inter[29/30/31/82]`
    /// labels (`ChestUI.cs:206-218`, the en-US strings the bank row already
    /// carries) at the reserved anchors, their hit regions answer, and the
    /// label-less Sort anchor (4, `inter[122]` — out of scope) draws nothing
    /// and takes no clicks (`ChestUI.cs:226-228` stays unported).
    #[test]
    fn test_DW_3_5_transfer_button_labels_and_hit_regions() {
        // The labels are the bank row's hardcoded English strings
        // (`ui/inventory.rs`'s M3-STORE-1 row).
        assert_eq!(button_label(0, false), Some("Loot All"));
        assert_eq!(button_label(1, false), Some("Deposit All"));
        assert_eq!(button_label(2, false), Some("Quick Stack"));
        assert_eq!(button_label(3, false), Some("Restock"));
        assert_eq!(button_label(4, false), None, "Sort stays label-less");
        // The anchors are the reserved rows: (506, invBottom + 40 + ID*26)
        // (`ChestUI.cs:203`).
        assert_eq!(button_anchor(0), (506.0, 298.0));
        assert_eq!(button_anchor(3), (506.0, 376.0));
        // The hit regions fire at each row (idle scale 0.75, a few px in —
        // the headless font estimate).
        let scales = [BUTTON_SCALE_MIN; BUTTON_COUNT];
        let idle = [false; BUTTON_COUNT];
        let hit =
            |mouse| chest_button_hit(&assets::Lookups::default(), mouse, &scales, &idle, 0, false);
        assert_eq!(hit(Vec2::new(510.0, 300.0)), Some(0));
        assert_eq!(hit(Vec2::new(510.0, 326.0)), Some(1));
        assert_eq!(hit(Vec2::new(510.0, 352.0)), Some(2));
        assert_eq!(hit(Vec2::new(510.0, 378.0)), Some(3));
        // Anchor 4's row answers nothing — no label, no rect.
        assert_eq!(hit(Vec2::new(510.0, 404.0)), None);
        // Off the row entirely: None.
        assert_eq!(hit(Vec2::new(480.0, 300.0)), None);
    }

    /// DW-3.5 — the chest panel stays hidden for the bank values (the
    /// Phase-1 two-panel ownership, decision #2): the draw gate — and the
    /// dispatch gate that shares it — run for `chest >= 0` only, so an open
    /// bank never draws (or dispatches) a second button row.
    #[test]
    fn test_DW_3_5_bank_values_draw_no_chest_panel() {
        assert!(panel_visible(0, false), "a world chest draws the panel");
        assert!(!panel_visible(-1, false), "closed: the idle reset");
        assert!(
            !panel_visible(-2, false),
            "piggy bank: STORE-1's panel only"
        );
        assert!(
            !panel_visible(-5, false),
            "void vault: STORE-1's panel only"
        );
        assert!(!panel_visible(0, true), "the crafting panel covers it");
    }

    #[test]
    fn dw_3_4_cancel_hidden_unless_editing() {
        // `ChestUI.cs:198-202`.
        assert!(button_visible(BUTTON_RENAME, 0, false));
        assert!(button_visible(BUTTON_RENAME, -1, false));
        assert!(!button_visible(BUTTON_RENAME, -2, false)); // the bank rule
        assert!(!button_visible(BUTTON_RENAME_CANCEL, 0, false));
        assert!(button_visible(BUTTON_RENAME_CANCEL, 0, true));
    }

    #[test]
    fn slot_positions_match_chestui() {
        // `ChestUI.cs:379-380`: `(int)(73 + i*56*0.755)`,
        // `(int)(258 + j*56*0.755)`.
        assert_eq!(slot_pos(0, 0), Vec2::new(73.0, 258.0));
        assert_eq!(slot_pos(1, 0), Vec2::new(115.0, 258.0)); // 73 + 42.28 → 115
        assert_eq!(slot_pos(9, 3), Vec2::new(453.0, 384.0)); // 453.52 / 384.84
        // `ChestUI.cs:364`: the display rectangle (73, 258, 422, 169).
        assert_eq!(display_rectangle(), [73.0, 258.0, 422.0, 169.0]);
    }

    #[test]
    fn slot_at_scrolls_with_the_window() {
        // `ChestUI.cs:376-386`: the drawn cell's rect holds the absolute
        // chest slot.
        assert_eq!(slot_at(Vec2::new(75.0, 260.0), 0), Some(0));
        assert_eq!(slot_at(Vec2::new(160.0, 310.0), 0), Some(12)); // cell (2,1)
        assert_eq!(slot_at(Vec2::new(460.0, 390.0), 0), Some(39)); // cell (9,3)
        assert_eq!(slot_at(Vec2::new(60.0, 260.0), 0), None); // left of the grid
        assert_eq!(slot_at(Vec2::new(200.0, 250.0), 0), None); // above the grid
        // A scrolled window shows the later slots: cell (0,0) of row 1 is
        // chest slot 10. The `num4 < item.Length` bound (`:377`) still cuts
        // the cells past 40 — row 1's bottom-right cell would be slot 49.
        assert_eq!(slot_at(Vec2::new(75.0, 260.0), 1), Some(10));
        assert_eq!(slot_at(Vec2::new(460.0, 390.0), 1), None);
    }

    #[test]
    fn update_hover_ramps_the_scale_and_pins_the_hover_edge_menu_tick() {
        // `ChestUI.UpdateHover` (`ChestUI.cs:58-82`): the ±0.05 scale
        // animation with its clamps and the hover-start
        // `SoundEngine.PlaySound(12)` (`:64`) — the helper returns the legacy
        // id so the test pins it (the phase-2 `toggle_open` pattern).
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = PanelState {
            chest: 0,
            origin: (0, 0),
            tile: None,
            slots: None,
            crafting_open: false,
            editing: false,
            name: "",
            max_items: DEFAULT_MAX_ITEMS,
            edit_text: "",
            blinker: &mut app.text_blinker,
            scroll_row: &mut app.chest_scroll_row,
            button_scales: &mut app.chest_button_scales,
            button_hovered: &mut app.chest_button_hovered,
            audio: &mut app.audio,
        };
        // The static ctor's `new float[7]` (`ChestUI.cs:1306`): scale 0.0.
        assert_eq!(p.button_scales[BUTTON_RENAME], 0.0);
        // Entering the rect is the edge: sound 12 (`:64`), +0.05.
        assert_eq!(update_hover(&mut p, BUTTON_RENAME, true), 12);
        assert_eq!(p.button_scales[BUTTON_RENAME], 0.05);
        // Holding the hover ramps without another sound.
        assert_eq!(update_hover(&mut p, BUTTON_RENAME, true), 0);
        assert_eq!(p.button_scales[BUTTON_RENAME], 0.1);
        // The ramp clamps at `buttonScaleMaximum` (`:68-71`).
        for _ in 0..20 {
            assert_eq!(update_hover(&mut p, BUTTON_RENAME, true), 0);
        }
        assert_eq!(p.button_scales[BUTTON_RENAME], BUTTON_SCALE_MAX);
        // Un-hovering decays 0.05 per frame, clamped at `buttonScaleMinimum`
        // (`:76-80`) — never below 0.75, never another sound.
        for _ in 0..30 {
            assert_eq!(update_hover(&mut p, BUTTON_RENAME, false), 0);
        }
        assert_eq!(p.button_scales[BUTTON_RENAME], BUTTON_SCALE_MIN);
        // Leaving and re-entering the rect fires the edge sound again.
        assert_eq!(update_hover(&mut p, BUTTON_RENAME, true), 12);
        assert!((p.button_scales[BUTTON_RENAME] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn draw_name_text_follows_the_draw_name_chain() {
        // `ChestUI.cs:110-141`: the edit text while `Main.editChest`
        // (:111-114), else the stored name (:115-122), else the tile-style
        // default (:124-140).
        assert_eq!(draw_name_text(true, "Editing", "Old", Some((21, 0))), "Editing");
        assert_eq!(draw_name_text(false, "Editing", "Custom", Some((21, 0))), "Custom");
        assert_eq!(draw_name_text(false, "", "", Some((21, 0))), "Chest");
        assert_eq!(draw_name_text(false, "", "", Some((21, 72))), "Locked Gold Chest");
        assert_eq!(draw_name_text(false, "", "", Some((467, 144))), "Dead Mans Chest");
        assert_eq!(draw_name_text(false, "", "", None), "");
    }

    #[test]
    fn button_label_shows_save_while_editing() {
        // `ChestUI.cs:221` — `Lang.inter[Main.editChest ? 47 : 61]`:
        // inter[47] "Save", inter[61] "Rename" (the en-US localization;
        // the dispatch's "Cancel" note was the pre-verification guess).
        assert_eq!(button_label(BUTTON_RENAME, false), Some("Rename"));
        assert_eq!(button_label(BUTTON_RENAME, true), Some("Save"));
        // `Lang.inter[63]` (:224) — the cancel button's label never changes.
        assert_eq!(button_label(BUTTON_RENAME_CANCEL, false), Some("Cancel"));
        assert_eq!(button_label(BUTTON_RENAME_CANCEL, true), Some("Cancel"));
    }

    #[test]
    fn text_blinker_toggles_every_twentieth_edit_frame() {
        // `ChestUI.cs:175-183` — `++textBlinkerCount >= 20` flips
        // `textBlinkerState`; the caret draws only while the state is 1. The
        // counter starts at 0/state 0 (`Main.cs:2287-2289`'s zero init), so
        // the first 20 editing draws show no caret.
        let mut b = TextBlinker::default();
        assert!(!b.on);
        for _ in 0..19 {
            b.advance();
            assert!(!b.on, "the first 20 draws stay dark");
        }
        b.advance(); // the 20th: count hits 20 → state 1, count resets
        assert!(b.on);
        assert_eq!(b.count, 0);
        for _ in 0..19 {
            b.advance();
            assert!(b.on, "the next 20 draws show the caret");
        }
        b.advance(); // the 40th: state flips back
        assert!(!b.on);
        assert_eq!(b.count, 0);
    }
}
