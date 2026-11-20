//! The dynamic item grid — W4-d82 (research/18b row 30):
//! `UIDynamicItemCollection<TEntry>`
//! (`GameContent/UI/Elements/UIDynamicItemCollection.cs:15-166`), the abstract
//! 44×44-slot grid `UICreativeItemGrid` (row 23) and the bestiary/creative
//! collections subclass. The port carries the *geometry + draw budget* as pure
//! code; the per-entry `DrawSlot` stays the subclass's seam.
//!
//! The grid centres its columns in the element rect, culls rows against the
//! parent `UIList`'s view area (`GetViewCullingArea`, `UIElement.cs:359-362`),
//! and budgets texture loads per frame: one slot per column for draws, four
//! per load for the prefetch pass (`:44-70`).

/// `sizePerEntryX`/`sizePerEntryY` (`UIDynamicItemCollection.cs:21-23`).
pub const SLOT: i32 = 44;

/// `SnapPointName_ItemSlot` (`:13`) — the gamepad anchor name (gamepad unported).
pub const SNAP_POINT_NAME: &str = "DynamicItemCollectionSlot";

/// `DrawSelf`'s scale pin (`:40`): `Main.inventoryScale = 3465f / 4095f`.
pub fn inventory_scale() -> f32 {
    3465.0 / 4095.0
}

/// C# `int / int` truncates toward zero (unlike Rust floor-div for negatives).
fn cdiv(a: i32, b: i32) -> i32 {
    a / b
}

/// `UpdateSize` (`:160-165`): `_itemsPerLine = rect.Width / 44` (the rect is
/// `ToRectangle()`, so the width is already truncated), `MinHeight =
/// 44 * ceil(count / itemsPerLine)`. Returns `(items_per_line, min_height_px)`.
/// A width under 44 gives 0 columns — the C# then throws in the `Ceiling`
/// division; the port returns `(0, 0)` and the caller must not lay out.
pub fn update_size(rect_w: f32, count: usize) -> (i32, f32) {
    let per_line = cdiv(rect_w as i32, SLOT);
    if per_line <= 0 {
        return (0, 0.0);
    }
    let rows = (count as f32 / per_line as f32).ceil();
    (per_line, (SLOT as f32) * rows)
}

/// `GetGridParameters` (`:81-97`) — the visible window. `rect` is the element's
/// `ToRectangle()` (truncated ints), `cull` the parent's view-culling rect.
/// Returns `(start_x, start_y, start_item_index, end_item_index)`.
pub fn grid_parameters(
    rect: [i32; 4],
    cull: [i32; 4],
    items_per_line: i32,
    count: usize,
) -> (i32, i32, i32, i32) {
    // `:85` — Center.X is the float centre (`CalculatedStyle.cs:27`), the
    // half-width cast truncates: `(int)(44 * perLine * 0.5f)`.
    let center_x = rect[0] as f32 + rect[2] as f32 * 0.5;
    let start_x = center_x as i32 - ((SLOT * items_per_line) as f32 * 0.5) as i32;
    let mut start_y = rect[1];
    let mut start_index = 0i32;
    let mut end_index = count as i32;
    // `:89-91` — rows scrolled above the view's top: num is <= 0, the grid
    // shifts DOWN by -num rows and skips -num*perLine entries.
    let num = cdiv(rect[1].min(cull[1]) - cull[1], SLOT);
    start_y += -num * SLOT;
    start_index += -num * items_per_line;
    // `:92-96` — the view holds ceil(height/44) rows plus one spare row.
    let num2 = (cull[3] as f32 / SLOT as f32).ceil() as i32 * items_per_line;
    if end_index > num2 + start_index + items_per_line {
        end_index = num2 + start_index + items_per_line;
    }
    (start_x, start_y, start_index, end_index)
}

/// `GetItemSlotHitbox` (`:75-79`): slot `i`'s 44×44 rect in the visible window.
pub fn item_slot_hitbox(start_x: i32, start_y: i32, start_index: i32, i: i32, per_line: i32) -> [i32; 4] {
    let num = i - start_index;
    [
        start_x + num % per_line * SLOT,
        start_y + num / per_line * SLOT,
        SLOT,
        SLOT,
    ]
}

/// The `DrawSelf` budget walk (`:41-70`) as a pure plan. `loaded[i]` says the
/// entry's item texture is resident. Returns `(drawn, to_load)`: the indices
/// drawn this frame (hover flag computed by the caller per slot) and the
/// indices the prefetch pass requests (`LoadItem`, `:67`).
///
/// Pass 1 (`:44-57`): budget = `items_per_line`; each visible entry draws; an
/// unloaded texture costs one extra budget unit (`:48-51`); the walk breaks at
/// budget <= 0 *after* the draw. Pass 2 (`:58-70`): starting at `end_index`,
/// wrap through the whole list; each unloaded entry is queued and costs 4.
pub fn draw_plan(
    loaded: &[bool],
    start_index: i32,
    end_index: i32,
    items_per_line: i32,
) -> (Vec<usize>, Vec<usize>) {
    let count = loaded.len();
    let mut budget = items_per_line;
    let mut drawn = Vec::new();
    for i in start_index..end_index {
        let idx = i as usize;
        if idx >= count {
            break;
        }
        if !loaded[idx] {
            budget -= 1; // `:50`
        }
        drawn.push(idx); // `:52` DrawSlot
        if budget <= 0 {
            break; // `:53-56`
        }
    }
    let mut to_load = Vec::new();
    if count > 0 {
        for j in 0..count {
            if budget <= 0 {
                break; // `:60-63`
            }
            let idx = (j + end_index as usize) % count; // `:64`
            if !loaded[idx] {
                to_load.push(idx); // `:67` LoadItem
                budget -= 4; // `:68`
            }
        }
    }
    (drawn, to_load)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `UpdateSize` (`:160-165`): 400 px holds 9 columns; 20 entries need 3 rows.
    #[test]
    fn the_size_pins_columns_and_min_height() {
        assert_eq!(update_size(400.0, 20), (9, 132.0));
        assert_eq!(update_size(440.0, 10), (10, 44.0));
        assert_eq!(update_size(43.9, 5), (0, 0.0), "under one column: no layout");
        assert_eq!(inventory_scale(), 3465.0 / 4095.0);
    }

    /// `GetGridParameters` (`:81-97`): a 400-wide rect centred at x=300 with 9
    /// columns starts at 300 - 198 = 102; a view scrolled 88 px (2 rows) into
    /// the list skips 18 entries and shifts the grid down 88.
    #[test]
    fn the_grid_window_culls_and_centres() {
        let rect = [100, 0, 400, 400];
        let cull = [0, 88, 800, 200];
        let (sx, sy, si, ei) = grid_parameters(rect, cull, 9, 100);
        assert_eq!(sx, 300 - 198);
        assert_eq!(sy, 88);
        assert_eq!(si, 18);
        // ceil(200/44)=5 rows * 9 + 9 spare + 18 start = 72.
        assert_eq!(ei, 72);
        // Unscrolled: start at the rect top, index 0.
        let (sx2, sy2, si2, _) = grid_parameters(rect, [0, 0, 800, 200], 9, 100);
        assert_eq!((sx2, sy2, si2), (102, 0, 0));
    }

    /// `GetItemSlotHitbox` (`:75-79`): row-major 44 px cells.
    #[test]
    fn the_hitboxes_are_row_major_44_cells() {
        assert_eq!(item_slot_hitbox(102, 0, 0, 0, 9), [102, 0, 44, 44]);
        assert_eq!(item_slot_hitbox(102, 0, 0, 9, 9), [102, 44, 44, 44]);
        assert_eq!(item_slot_hitbox(102, 0, 0, 10, 9), [146, 44, 44, 44]);
    }

    /// The budget walk (`:44-70`): 3 columns, all loaded → 3 draws, no loads;
    /// an unloaded slot costs one extra, and the prefetch costs 4 per load.
    #[test]
    fn the_draw_budget_charges_loads() {
        let loaded = [true; 12];
        let (drawn, to_load) = draw_plan(&loaded, 0, 12, 3);
        assert_eq!(drawn, vec![0, 1, 2]);
        assert!(to_load.is_empty());
        // Slot 1 unloaded: budget 3 → draw 0 (3), the extra charge drops it to
        // 2 then draw 1, draw 2 leaves 1 (>0, no break); the loop ends at 12.
        let mut l2 = [true; 12];
        l2[1] = false;
        let (d2, t2) = draw_plan(&l2, 0, 12, 3);
        assert_eq!(d2, vec![0, 1, 2]);
        // Prefetch starts at end_index=12 → wraps to 0; slot 1 costs 4 → -3.
        assert_eq!(t2, vec![1]);
    }
}
