//! The `cursorItemIcon*` cursor-overlay feed (W4-g86 cursor-item-icon lane).
//!
//! The full system census + owner contract lives in
//! `docs/research/244-cursor-item-icon-seed.md`: 357 write sites across 7 C#
//! files (Player.cs alone carries 326 writes / 169 `cursorItemIconID`
//! writes — the ledger's "~170 sites"). Phase 1 (this lane) landed the
//! state fields (`cursorItemIconID/Text/Push/Reversed` on App beside g81's
//! `cursor_item_icon_enabled`), the full `MouseOversClear` quad
//! (`Main.cs:46503-46513`, `frame.rs`'s world-branch head) + the :16842
//! once-per-frame Reversed clear (`frame_head`), the layer-40 ID-override
//! draw arm (`DrawInterface_40_InteractItemIcon` `Main.cs:45057-45105`,
//! `ui/mod.rs` — g81's declared gap) and WIRED both batches below at their
//! C# call positions in `editing.rs`. This module holds the pure decode
//! arms in ≤10-site batches; the wiring writes ride the C# field order per
//! function. Zero RNG — every arm is integer maths.
//!
//! Batch 1 (WIRED at `update_editing`'s tile-hover head): the altar hover
//! icons — `TileInteractionsMouseOver`'s tile-26
//! (`Player.cs:34524-34536`) and tile-237 (`:34309-34314`) arms, 4 cursor
//! write sites.
//!
//! Batch 2 (WIRED at `update_editing`'s ItemCheck head): the `ItemCheck`
//! head's held-item icon arms (`Player.cs:42615-42650`) — the fishing
//! pole's bait icon (push 6) and the Clentaminator/Terraformer's solution
//! icon (push 10), 9 cursor write sites + the `Fishing_GetBait` scan
//! (`:42254-42280`) they read.

/// `Player.TileInteractionsMouseOver`'s altar arms (`Player.cs:34524-34536`
/// tile 26, `:34309-34314` tile 237): the hover cursor icon for the
/// demon/crimson altar and the Lihzahrd altar. Returns the
/// `cursorItemIconID` the arm writes; `None` = the kind has no altar arm.
///
/// Tile 26 (`:34528-34535`): `num41 = frameX / 18`, then the
/// `while (num41 > 2) { num41 -= 3; num42++; }` column-group loop — group 0
/// (the first 3 frame columns, the demon altar's 3-wide sheet) draws the
/// Demon Altar item 6135, every later group the Crimson Altar 6136. The
/// loop is kept verbatim (NOT `frameX / 54`): the C# truncates `frameX/18`
/// first, so a mid-group column (e.g. `frameX = 90`, column 5) still lands
/// in group 1 exactly like the source.
///
/// Tile 237 (`:34313`): always the Lihzahrd Power Cell 1293 — the summon
/// item, matching the outline feed's `HasItem(1293)` gate
/// (`world_sim/smart_interact.rs`, `FillPotentialTargetTiles` `:453-457`).
///
/// Wiring (research/244 phase 1, WIRED): `update_editing`'s tile-hover head
/// (`editing.rs`, the `TileInteractionsCheck` → `TileInteractionsMouseOver`
/// seam, `Player.cs:32039`) sets `noThrow = 2` + `cursorItemIconEnabled =
/// true` BEFORE the ID write (both arms, `:34526-34527`/`:34311-34312`),
/// and the layer-40 ID override draws it. The g86 outline pass is the
/// SELECTION feedback; this icon is the distinct hover overlay (ANALYSIS
/// "The altar-26 SUMMON-USE chain — census").
pub(crate) fn altar_hover_icon(kind: u16, frame_x: i16) -> Option<u16> {
    match kind {
        // `Player.cs:34524-34536` — the demon/crimson altar.
        26 => {
            let mut num41 = frame_x as i32 / 18;
            let mut num42 = 0;
            while num41 > 2 {
                num41 -= 3;
                num42 += 1;
            }
            Some(if num42 == 0 { 6135 } else { 6136 })
        }
        // `Player.cs:34309-34314` — the Lihzahrd altar.
        237 => Some(1293),
        _ => None,
    }
}

/// `Player.Fishing_GetBait` (`Player.cs:42254-42280`): the bait scan the
/// `ItemCheck` head's pole icon reads — the ammo slots 54-57 FIRST
/// (`inventory[54..58]`, the port's `equip.ammo`), then the grid 0-49
/// (`inventory[0..50]`, NOT the coin rows — the C# loop is `j < 50`).
/// First `stack > 0 && bait > 0` slot wins; returns `(baitPower, baitType)`
/// with `(0, 0)` when no bait is carried. The scan is the SAME one
/// `ConsumeBait` uses (`:52452-52470`, the `fishing_use` precedent) — this
/// is the icon-side read, no consumption. WIRED (research/244 phase 1) via
/// [`item_check_head_icon`] at `update_editing`'s head.
pub(crate) fn fishing_get_bait(
    grid: &[crate::item::ItemSlot],
    ammo: &[crate::item::ItemSlot],
) -> (i32, u16) {
    // `:42258-42266` — the ammo row first.
    for slot in ammo {
        let bait = crate::registry::get(slot.kind).map_or(0, |d| d.bait as i32);
        if slot.count > 0 && bait > 0 {
            return (bait, slot.kind);
        }
    }
    // `:42271-42279` — the grid 0-49 (the coin/ammo rows are NOT scanned).
    for slot in grid.iter().take(50) {
        let bait = crate::registry::get(slot.kind).map_or(0, |d| d.bait as i32);
        if slot.count > 0 && bait > 0 {
            return (bait, slot.kind);
        }
    }
    (0, 0)
}

/// The `cursorItemIconPush` offset the `ItemCheck` head's arms write:
/// the fishing pole's bait icon pushes 6 (`Player.cs:42622`), the
/// Clentaminator's solution icon pushes 10 (`:42633`/`:42645`). The draw
/// adds it to the base 10 (`Main.cs:45089`, `10 + cursorItemIconPush`).
pub(crate) const BAIT_ICON_PUSH: i32 = 6;
/// See [`BAIT_ICON_PUSH`] — the solution arm's push (`Player.cs:42633`).
pub(crate) const SOLUTION_ICON_PUSH: i32 = 10;

/// `Player.ItemCheck`'s head icon arms (`Player.cs:42615-42650`), the
/// local-player block (`Main.myPlayer == whoAmI`, `:42608`). Both arms
/// gate on `!cursorItemIconEnabled` — an earlier hover arm (the tile pass
/// runs first in `Player.Update`) wins; the held-item icon only fills an
/// idle cursor. Returns `(iconID, push)` for the arm that fires, `None`
/// when neither does.
///
/// Arm 1 (`:42615-42624`): a held fishing pole (`item.stack > 0 &&
/// item.fishingPole > 0`) draws the first bait `Fishing_GetBait` finds
/// (`baitType > 0`), push 6.
///
/// Arm 2 (`:42625-42650`): the Clentaminator 779 / Terraformer 5134 draw
/// the first solution matching `item.useAmmo` (780) — the ammo slots
/// 54-57 first (`:42627-42636`), then `inventory[0..54]`
/// (`:42639-42648`, the C# `j < 54` — the 50 grid slots PLUS the 4 coin
/// rows, unlike the bait scan's `j < 50`), push 10. The ammo-row scan
/// breaks on the first match; the 0-53 scan only runs when the ammo row
/// found nothing (`!cursorItemIconEnabled`, `:42637`). The port's layout
/// splits the C# 58-slot array: `grid` = inventory[0..50], `coins` =
/// inventory[50..54], `ammo` = inventory[54..58].
///
/// Wiring (research/244 phase 1, WIRED): `update_editing`'s head
/// (`editing.rs`) runs this AFTER the CCed gate (`:42598`) and BEFORE the
/// countdowns / the early-dismount head (the `ItemCheck_HandleMount`
/// neighbour, `:42652`); on `Some((id, push))` it writes
/// `cursorItemIconEnabled = true`, `cursorItemIconID = id`,
/// `cursorItemIconPush = push` (the C# order). NO `ItemIconCacheUpdate`
/// here — these arms do not arm the linger (only the tool latches do).
pub(crate) fn item_check_head_icon(
    held: crate::item::ItemSlot,
    grid: &[crate::item::ItemSlot],
    coins: &[crate::item::ItemSlot],
    ammo: &[crate::item::ItemSlot],
    icon_enabled: bool,
) -> Option<(u16, i32)> {
    if icon_enabled || held.count == 0 {
        return None;
    }
    let def = crate::registry::get(held.kind)?;
    // `:42615-42624` — the fishing pole's bait icon.
    if def.fishing_pole > 0 {
        let (_, bait_type) = fishing_get_bait(grid, ammo);
        if bait_type > 0 {
            return Some((bait_type, BAIT_ICON_PUSH));
        }
        // The C# falls through to the 779/5134 check with the icon still
        // disabled — a pole is neither, so the arm ends here.
        return None;
    }
    // `:42625-42650` — the Clentaminator/Terraformer's solution icon.
    if held.kind == 779 || held.kind == 5134 {
        // `:42627-42636` — the ammo row (inventory[54..58]).
        for slot in ammo {
            let slot_ammo = crate::registry::get(slot.kind).map_or(0, |d| d.ammo);
            if slot_ammo == def.use_ammo && slot.count > 0 {
                return Some((slot.kind, SOLUTION_ICON_PUSH));
            }
        }
        // `:42639-42648` — inventory[0..54]: the grid then the coin rows.
        for slot in grid.iter().take(50).chain(coins.iter()) {
            let slot_ammo = crate::registry::get(slot.kind).map_or(0, |d| d.ammo);
            if slot_ammo == def.use_ammo && slot.count > 0 {
                return Some((slot.kind, SOLUTION_ICON_PUSH));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-g86 batch 1 — the demon altar's first 3 frame columns (group 0)
    /// draw the Demon Altar item 6135 (`Player.cs:34535`'s `num42 == 0`
    /// arm). WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_g86_the_demon_altar_columns_draw_6135() {
        for frame_x in [0i16, 18, 36] {
            assert_eq!(
                altar_hover_icon(26, frame_x),
                Some(6135),
                "frameX {frame_x}: column {} stays in group 0",
                frame_x / 18
            );
        }
    }

    /// W4-g86 batch 1 — every later column group draws the Crimson Altar
    /// 6136: the subtract-3 loop counts groups (`:34530-34534`). Column 3
    /// (frameX 54) loops once (3 > 2 → 0, group 1); column 6 (frameX 108)
    /// loops twice (6 → 3 → 0, group 2); a MID-group column 5 (frameX 90)
    /// loops once (5 → 2, group 1) — the verbatim loop, not `frameX / 54`,
    /// decides. WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_crimson_altar_columns_draw_6136() {
        for frame_x in [54i16, 72, 90, 108, 162] {
            assert_eq!(
                altar_hover_icon(26, frame_x),
                Some(6136),
                "frameX {frame_x}: column {} leaves group 0",
                frame_x / 18
            );
        }
    }

    /// W4-g86 batch 1 — the Lihzahrd altar 237 always draws the Power Cell
    /// 1293 (`Player.cs:34313`, no frame decode), and a non-altar kind has
    /// no arm (`None` — the caller's match default writes nothing).
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_lihzahrd_altar_draws_the_power_cell() {
        assert_eq!(altar_hover_icon(237, 0), Some(1293));
        assert_eq!(altar_hover_icon(237, 18), Some(1293));
        // The workbench 18 rides its own arm (`:34518-34523`), not this one.
        assert_eq!(altar_hover_icon(18, 0), None);
        assert_eq!(altar_hover_icon(695, 0), None);
    }

    fn slot(kind: u16, count: u32) -> crate::item::ItemSlot {
        crate::item::ItemSlot { kind, name: "", count, prefix: 0 }
    }

    /// W4-g86 batch 2 — `Fishing_GetBait` (`Player.cs:42254-42280`): the
    /// ammo row wins over the grid, the first live bait slot wins inside
    /// a row, an empty stack is skipped, and no bait returns (0, 0).
    /// WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_g86_the_bait_scan_prefers_the_ammo_row() {
        let grid = [crate::item::EMPTY_SLOT; 50];
        let mut ammo = [crate::item::EMPTY_SLOT; 4];
        // Grid-only: the first live bait (2674 Apprentice, bait 15).
        let mut g = grid;
        g[7] = slot(2674, 3);
        g[9] = slot(2675, 1);
        assert_eq!(fishing_get_bait(&g, &ammo), (15, 2674));
        // Ammo row beats the grid even with a weaker bait.
        ammo[2] = slot(2674, 1);
        g[7] = slot(2673, 1); // Truffle Worm, bait 666
        assert_eq!(fishing_get_bait(&g, &ammo), (15, 2674));
        // An empty ammo stack is skipped (`stack > 0`).
        ammo[2] = slot(2674, 0);
        assert_eq!(fishing_get_bait(&g, &ammo), (666, 2673));
        // No bait anywhere.
        let g2 = grid;
        let a2 = [crate::item::EMPTY_SLOT; 4];
        assert_eq!(fishing_get_bait(&g2, &a2), (0, 0));
    }

    /// W4-g86 batch 2 — the pole arm (`Player.cs:42615-42624`): a held
    /// Golden Fishing Rod 2294 (fishingPole 50) draws the bait with push
    /// 6; no bait → no icon; an already-enabled icon blocks the arm
    /// (`!cursorItemIconEnabled`, the hover pass wins). WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_pole_draws_the_bait_push_6() {
        let grid = [crate::item::EMPTY_SLOT; 50];
        let coins = [crate::item::EMPTY_SLOT; 4];
        let mut ammo = [crate::item::EMPTY_SLOT; 4];
        ammo[0] = slot(2675, 5); // Journeyman Bait, bait 30
        let pole = slot(2294, 1);
        assert_eq!(
            item_check_head_icon(pole, &grid, &coins, &ammo, false),
            Some((2675, 6))
        );
        // The hover icon already armed: the head arm refuses.
        assert_eq!(item_check_head_icon(pole, &grid, &coins, &ammo, true), None);
        // No bait: the arm ends (a pole is not 779/5134).
        let empty_ammo = [crate::item::EMPTY_SLOT; 4];
        assert_eq!(item_check_head_icon(pole, &grid, &coins, &empty_ammo, false), None);
        // An empty held stack never arms (`item.stack > 0`).
        assert_eq!(item_check_head_icon(slot(2294, 0), &grid, &coins, &ammo, false), None);
    }

    /// W4-g86 batch 2 — the solution arm (`Player.cs:42625-42650`): the
    /// Clentaminator 779 (useAmmo 780) draws the first ammo-780 solution
    /// with push 10 — the ammo row first, then the grid, then the COIN
    /// rows (`j < 54`); a non-matching ammo (Wooden Arrow 40, `ammo` 40 ≠
    /// 780) is skipped. The Terraformer 5134 rides the same arm.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_g86_the_clentaminator_draws_the_solution_push_10() {
        let mut grid = [crate::item::EMPTY_SLOT; 50];
        let mut coins = [crate::item::EMPTY_SLOT; 4];
        let mut ammo = [crate::item::EMPTY_SLOT; 4];
        let clent = slot(779, 1);
        // Grid-only Green Solution 780 (ammo 780).
        grid[3] = slot(780, 99);
        assert_eq!(
            item_check_head_icon(clent, &grid, &coins, &ammo, false),
            Some((780, 10))
        );
        // The ammo row beats the grid.
        ammo[1] = slot(780, 5);
        grid[3] = crate::item::EMPTY_SLOT;
        assert_eq!(
            item_check_head_icon(clent, &grid, &coins, &ammo, false),
            Some((780, 10))
        );
        // The coin rows are scanned (`j < 54`) when grid+ammo are empty.
        let empty_ammo = [crate::item::EMPTY_SLOT; 4];
        coins[2] = slot(780, 1);
        assert_eq!(
            item_check_head_icon(clent, &grid, &coins, &empty_ammo, false),
            Some((780, 10))
        );
        // A non-matching ammo (arrow 40) never fires; the Terraformer
        // 5134 shares the arm.
        let coins2 = [crate::item::EMPTY_SLOT; 4];
        let mut g2 = [crate::item::EMPTY_SLOT; 50];
        g2[0] = slot(40, 99);
        assert_eq!(item_check_head_icon(clent, &g2, &coins2, &empty_ammo, false), None);
        g2[0] = slot(780, 1);
        assert_eq!(
            item_check_head_icon(slot(5134, 1), &g2, &coins2, &empty_ammo, false),
            Some((780, 10))
        );
        // A plain held item (Iron Pickaxe 1) has no head arm.
        assert_eq!(item_check_head_icon(slot(1, 1), &g2, &coins2, &empty_ammo, false), None);
    }
}
