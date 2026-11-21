//! The `cursorItemIcon*` cursor-overlay feed (W4-g86 cursor-item-icon lane).
//!
//! The full system census + owner contract lives in
//! `docs/research/244-cursor-item-icon-seed.md`: 357 write sites across 7 C#
//! files (Player.cs alone carries 326 writes / 169 `cursorItemIconID`
//! writes — the ledger's "~170 sites"). The state fields
//! (`cursorItemIconID/Text/Push/Reversed`), the per-update
//! `MouseOversClear` (`Main.cs:46503-46513`) and the
//! `DrawInterface_40_InteractItemIcon` ID-override draw arm
//! (`Main.cs:45037-45115`) are the seeded system — the in-flight
//! `task/W4-g81-wire-draw-overlay` (tip `ebb5308a0`) owns the layer-40
//! held-kind arm + the 10-frame cache + the `cursorItemIconEnabled` latch
//! and explicitly leaves the ID override unported (its `ui/mod.rs`
//! comment). This module ports the SELF-CONTAINED pure decode arms in
//! ≤10-site batches, PORTED-UNWIRED (the `IsHopperInRangeOf` precedent):
//! each function is the exact C# arm maths, the wiring contract (which
//! fields the caller arms, in which order) is documented per function and
//! in research/244 §4. Zero RNG — every arm is integer maths.
//!
//! Batch 1 (this file's first landing): the altar hover icons —
//! `TileInteractionsMouseOver`'s tile-26 (`Player.cs:34524-34536`) and
//! tile-237 (`:34309-34314`) arms, 4 cursor write sites.

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
/// Wiring contract (research/244 §4, UNWIRED): the caller runs inside the
/// tile-hover pass (`TileInteractionsCheck` → `TileInteractionsMouseOver`,
/// `Player.cs:32039`), sets `noThrow = 2` + `cursorItemIconEnabled = true`
/// BEFORE the ID write (both arms, `:34526-34527`/`:34311-34312`), and the
/// layer-40 ID override draws it. The g86 outline pass is the SELECTION
/// feedback; this icon is the distinct hover overlay (ANALYSIS "The
/// altar-26 SUMMON-USE chain — census").
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
}
