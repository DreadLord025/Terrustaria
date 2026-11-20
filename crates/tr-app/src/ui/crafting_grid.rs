//! The crafting recipe grid's nested element classes — W4-f23 (census lane 4,
//! research/18e): `NewCraftingUI.RecipeEntry` (`GameContent/UI/NewCraftingUI.cs:23-39`)
//! and `NewCraftingUI.ItemGrid : UIDynamicItemCollection<RecipeEntry>` (`:41-76`).
//! The d81-d83 census enumerated `Elements/` files only; this nested
//! `UIElement` subclass (the in-game crafting grid) was in no lane's rows.
//! `ui/crafting.rs` ports the panel chrome + `recipe_cell` geometry (the
//! `UIDynamicItemCollection` half, d82 row 30's `item_collection` module on
//! that lane); this ports the *entry state + DrawSlot machine* the geometry
//! feeds. `DisabledSlotColor` (`:226`, `:994`) and `IsGridPoint` (`:754-761`)
//! ride along — they are the class's own statics.
//!
//! Immediate-mode ruling (research/18): pure state/geometry helpers; the
//! `ItemSlot.Draw` call itself stays the renderer's (`ui::slot`, context 41).
//! Gamepad arms are ported as data (gamepad unported — research/18 dev. 10).
#![allow(dead_code)] // wiring lands when crafting.rs adopts the DrawSlot machine

use super::color;

/// `NewCraftingUI.DisabledSlotColor` (`NewCraftingUI.cs:226`, set in the
/// static ctor `:994`): `new Color(160, 160, 160, 255)` — the tint an
/// unavailable recipe's slot draws with (`:74`, `:563`).
pub const DISABLED_SLOT_COLOR: [u8; 4] = [160, 160, 160, 255];

/// `RecipeEntry` (`NewCraftingUI.cs:23-39`): the raw recipe index plus the
/// two -1-initialised list positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeEntry {
    /// `public readonly int index` (`:25`) — the `Main.recipe` index.
    pub index: usize,
    /// `availableIndex = -1` (`:27`) — position in `Main.availableRecipe`,
    /// -1 while the recipe's requirements are unmet.
    pub available_index: i32,
    /// `gridIndex = -1` (`:29`) — position in the shown grid list.
    pub grid_index: i32,
}

impl RecipeEntry {
    /// The ctor (`:35-38`): index set, both list positions -1.
    pub fn new(index: usize) -> Self {
        Self {
            index,
            available_index: -1,
            grid_index: -1,
        }
    }

    /// `Available => availableIndex >= 0` (`:31`).
    pub fn available(&self) -> bool {
        self.available_index >= 0
    }

    /// `Recipe => Main.recipe[this.index]` (`:33`) — the port's recipe table.
    pub(crate) fn recipe(&self) -> &'static crate::craft::Recipe {
        &crate::craft::RECIPES[self.index]
    }
}

/// `NewCraftingUI.IsGridPoint` (`:754-761`): the grid's gamepad link-point
/// window is `[20100, 21000)` — `point >= 20100 && point < 21000`.
pub fn is_grid_point(point: i32) -> bool {
    point >= 20100 && point < 21000
}

/// `ItemGrid.DrawSlot`'s observable outputs (`NewCraftingUI.cs:55-75`), in
/// the C#'s write order: the two highlight statics, then the hover latch,
/// then the `ItemSlot.Draw` tint. The draw call itself is context 41 at the
/// hitbox's `TopLeft()` (`:58`, `:74`; the base walk passes it,
/// `UIDynamicItemCollection.cs:52`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlotDraw {
    /// `ItemSlot.DrawSelectionHighlightForGridSlot` (`:61`/`:66`) — the
    /// context-41 `InventoryBack24` overlay gate (`ItemSlot.cs:2371-2374`).
    pub grid_selection_highlight: bool,
    /// `UILinkPointNavigator.Shortcuts.ItemSlotShouldHighlightAsSelected`
    /// (`:62`/`:67`) — gamepad-only, unported consumer, kept as data.
    pub gamepad_selected_highlight: bool,
    /// `hovering` (`:69-73`): the parent's `_hoveredEntry = entry` +
    /// `HandleCraftSlot(entry, 41)` fire only on this edge.
    pub hovered: bool,
    /// The `ItemSlot.Draw` tint (`:74`): `entry.Available ? Color.White :
    /// DisabledSlotColor`.
    pub tint: [u8; 4],
}

/// The base walk's hover gate (`UIDynamicItemCollection.cs:52`):
/// `base.IsMouseHovering && itemSlotHitbox.Contains(value) &&
/// !PlayerInput.IgnoreMouseInterface`, where `value` is
/// `UserInterface.ActiveInstance.MousePosition.ToPoint()` — the mouse
/// truncated to ints, `Rectangle.Contains` inclusive-left/exclusive-right.
pub fn slot_hovering(
    element_hovering: bool,
    hitbox: [i32; 4],
    mouse: [f32; 2],
    ignore_mouse_interface: bool,
) -> bool {
    let (mx, my) = (mouse[0] as i32, mouse[1] as i32);
    element_hovering
        && mx >= hitbox[0]
        && mx < hitbox[0] + hitbox[2]
        && my >= hitbox[1]
        && my < hitbox[1] + hitbox[3]
        && !ignore_mouse_interface
}

/// `ItemGrid.DrawSlot` (`NewCraftingUI.cs:55-75`). `selected_recipe_index`
/// is the parent's `_selectedRecipeIndex` (`:196`, `int?` — `None` is null,
/// which never equals an entry index, `:66`); `using_gamepad` +
/// `current_point` drive the `:59` arm (`PlayerInput.UsingGamepad &&
/// IsGridPoint(UILinkPointNavigator.CurrentPoint)`).
pub fn draw_slot(
    entry: &RecipeEntry,
    selected_recipe_index: Option<usize>,
    using_gamepad: bool,
    current_point: i32,
    hovering: bool,
) -> SlotDraw {
    // `int context = 41` (`:58`) is constant — the caller's draw context.
    let (grid_selection_highlight, gamepad_selected_highlight) =
        if using_gamepad && is_grid_point(current_point) {
            // `:61-62`: the grid overlay off, the gamepad shortcut rides hover.
            (false, hovering)
        } else {
            // `:66-67`: the overlay marks the selected recipe; shortcut off.
            (selected_recipe_index == Some(entry.index), false)
        };
    SlotDraw {
        grid_selection_highlight,
        gamepad_selected_highlight,
        hovered: hovering, // `:69-73`
        tint: if entry.available() {
            [255, 255, 255, 255] // `Color.White` (`:74`)
        } else {
            DISABLED_SLOT_COLOR
        },
    }
}

/// The draw tint as the renderer's `[f32; 4]` (`ItemSlot.Draw`'s `drawColor`).
pub fn slot_tint(entry: &RecipeEntry) -> [f32; 4] {
    color::tint(if entry.available() {
        [255, 255, 255, 255]
    } else {
        DISABLED_SLOT_COLOR
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ctor + `Available` (`NewCraftingUI.cs:27-38`): both positions
    /// start -1, `Available` is the `availableIndex >= 0` gate.
    #[test]
    fn entries_start_unavailable_until_placed() {
        let e = RecipeEntry::new(7);
        assert_eq!(e.available_index, -1);
        assert_eq!(e.grid_index, -1);
        assert!(!e.available());
        let mut placed = e;
        placed.available_index = 0;
        assert!(placed.available(), "index 0 is available (`>= 0`)");
        assert_eq!(placed.recipe().create, crate::craft::RECIPES[7].create);
    }

    /// `IsGridPoint` (`:754-761`): the half-open `[20100, 21000)` window.
    #[test]
    fn the_grid_point_window_is_half_open() {
        assert!(!is_grid_point(20099));
        assert!(is_grid_point(20100));
        assert!(is_grid_point(20999));
        assert!(!is_grid_point(21000));
        assert!(!is_grid_point(0));
    }

    /// `DrawSlot`'s two arms (`:59-68`) + the tint pick (`:74`): mouse mode
    /// marks the selected recipe on the grid overlay, gamepad mode moves the
    /// highlight to the hover-driven shortcut; unavailable recipes grey to
    /// (160,160,160,255).
    #[test]
    fn draw_slot_swaps_the_highlight_by_input_mode() {
        let mut e = RecipeEntry::new(3);
        e.available_index = 2;
        // Mouse arm: selected -> overlay on, shortcut off.
        let d = draw_slot(&e, Some(3), false, 20150, false);
        assert!(d.grid_selection_highlight);
        assert!(!d.gamepad_selected_highlight);
        assert_eq!(d.tint, [255, 255, 255, 255]);
        // Not selected (or null selection) -> overlay off.
        assert!(!draw_slot(&e, Some(4), false, 0, false).grid_selection_highlight);
        assert!(!draw_slot(&e, None, false, 0, false).grid_selection_highlight);
        // Gamepad arm inside the window: overlay OFF even when selected,
        // shortcut = hovering (`:61-62`).
        let g = draw_slot(&e, Some(3), true, 20150, true);
        assert!(!g.grid_selection_highlight);
        assert!(g.gamepad_selected_highlight);
        assert!(g.hovered);
        // Gamepad outside the window falls back to the mouse arm (`:59`).
        assert!(draw_slot(&e, Some(3), true, 21000, false).grid_selection_highlight);
        // Unavailable: the DisabledSlotColor grey, alpha 255 (`:994`).
        let u = RecipeEntry::new(3);
        assert_eq!(
            draw_slot(&u, None, false, 0, false).tint,
            [160, 160, 160, 255]
        );
        assert_eq!(
            slot_tint(&u),
            [160.0 / 255.0, 160.0 / 255.0, 160.0 / 255.0, 1.0]
        );
    }

    /// The base walk's hover gate (`UIDynamicItemCollection.cs:52`): all
    /// three conjuncts, int-truncated mouse, exclusive right/bottom edges.
    #[test]
    fn hovering_needs_element_hover_containment_and_mouse_interface() {
        let hb = [100, 200, 44, 44];
        assert!(slot_hovering(true, hb, [100.9, 200.9], false));
        assert!(slot_hovering(true, hb, [143.9, 243.9], false));
        assert!(
            !slot_hovering(true, hb, [144.0, 220.0], false),
            "right edge excluded"
        );
        assert!(
            !slot_hovering(true, hb, [120.0, 244.0], false),
            "bottom edge excluded"
        );
        assert!(!slot_hovering(false, hb, [120.0, 220.0], false));
        assert!(
            !slot_hovering(true, hb, [120.0, 220.0], true),
            "IgnoreMouseInterface"
        );
    }
}
