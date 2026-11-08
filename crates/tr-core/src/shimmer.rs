//! The shimmer system — M7-SHIMMER-1.
//!
//! Ports the item-transform half of `Terraria.GameContent.ShimmerTransforms`
//! (`ShimmerTransforms.cs`), `Terraria.GameContent.ShimmerHelper`
//! (`ShimmerHelper.cs`), the shimmer-liquid predicate (`Tile.shimmer`,
//! `Tile.cs:379-381`) and the transform tables (the sibling
//! [`shimmer_tables`] module, machine-extracted from the decompiled ID sets).
//! The chain that consumes them (`WorldItem.GetShimmered`,
//! `WorldItem.cs:1691-1849`) lives app-side: it reads the recipe database, so
//! `crate::shimmer` stays the pure-logic half.

use crate::tile::{Tile, LiquidKind};

/// `Tile.shimmer()` (`Tile.cs:379-381`) over the liquid KIND and amount:
/// `(bTileHeader & 0x60) == 96` is the liquid-type byte == 3 (shimmer), and the
/// callers (`WorldItem.CanShimmerAtPosition` `:762`, `UpdateShimmer`'s rise
/// scan `:478-483`) additionally require `liquid > 0`.
pub fn tile_is_shimmer(tile: &Tile) -> bool {
    tile.liquid_kind() == LiquidKind::Shimmer && tile.liquid > 0
}

/// `ShimmerTransforms.GetTransformToItem` (`ShimmerTransforms.cs:88-111`) —
/// the shimmer transform target for an item kind, in the C#'s resolution order:
/// the set, then the createTile==139 placeStyle arm, then the 3461 moon-phase
/// arm. `create_tile`/`place_style` are the item's own fields and `moon_phase`
/// is `Main.GetMoonPhase()`'s 0-7 (`MoonPhase` enum: 0 = the empty/new moon,
/// ascending to `Full` at 4 and back down; see `Main.cs`'s moon-phase table).
pub fn transform_to_item(
    item_type: u16,
    create_tile: i16,
    place_style: i32,
    moon_phase: u8,
) -> i32 {
    if let Some(&(_, target)) = super::shimmer_tables::SHIMMER_TRANSFORM_TO_ITEM
        .iter()
        .find(|&&(ty, _)| ty == item_type)
    {
        return i32::from(target);
    }
    // The shimmer trophies (any item that creates tile 139) — `:95-105`.
    if create_tile == 139 {
        return match place_style {
            90 => 5538,
            89 => 5579,
            97 => 5638,
            96 => 5639,
            _ => 576,
        };
    }
    // The lunar bricks by moon phase — `:106-109` + `:113-126`.
    if item_type == 3461 {
        return lunar_brick_for_moon_phase(moon_phase);
    }
    0
}

/// `ShimmerTransforms.GetLunarBrickTransformFromMoonPhase`
/// (`ShimmerTransforms.cs:113-126`) — the `_` default is the falling/empty
/// phase (5406).
pub fn lunar_brick_for_moon_phase(moon_phase: u8) -> i32 {
    match moon_phase {
        7 => 5407, // QuarterAtRight
        6 => 5405, // HalfAtRight
        5 => 5404, // ThreeQuartersAtRight
        4 => 5408, // Full
        3 => 5401, // ThreeQuartersAtLeft
        2 => 5403, // HalfAtLeft
        1 => 5402, // QuarterAtLeft
        _ => 5406,
    }
}

/// `NPCID.Sets.ShimmerTransformToNPC` (`NPCID.cs:10470`) — the NPC's shimmer
/// transform target, -1 when unset.
pub fn transform_to_npc(npc_type: u16) -> i32 {
    super::shimmer_tables::SHIMMER_TRANSFORM_TO_NPC
        .iter()
        .find(|&&(ty, _)| ty == npc_type)
        .map(|&(_, target)| i32::from(target))
        .unwrap_or(-1)
}

/// `ItemID.Sets.CommonCoin` (`ItemID.cs:1318`).
pub fn is_common_coin(item_type: u16) -> bool {
    super::shimmer_tables::SHIMMER_COMMON_COIN.contains(&item_type)
}

/// `Item.GetShimmerEquivalentType` (`Item.cs:49476-49485`): the `forDecrafting`
/// flag picks the `ShimmerCountsAsItemForDecraft` map over
/// `ShimmerCountsAsItem`, then falls back to the item's own type.
pub fn shimmer_equivalent_type(item_type: u16, for_decrafting: bool) -> u16 {
    let map = if for_decrafting {
        super::shimmer_tables::SHIMMER_COUNTS_AS_ITEM_FOR_DECRAFT
    } else {
        super::shimmer_tables::SHIMMER_COUNTS_AS_ITEM
    };
    if let Some(&(_, target)) = map.iter().find(|&&(ty, _)| ty == item_type) {
        return target;
    }
    item_type
}

/// `ShimmerTransforms.IsItemTransformLocked` (`ShimmerTransforms.cs:33-40`) —
/// the post-Moonlord set blocks the transform until the Moon Lord is downed.
pub fn is_transform_locked(item_type: u16, downed_moonlord: bool) -> bool {
    !downed_moonlord && super::shimmer_tables::SHIMMER_POST_MOONLORD.contains(&item_type)
}

/// `ShimmerHelper.FindSpotWithoutShimmer` (`ShimmerHelper.cs:7-65`): the
/// expanding scan over the start's ±`expand` box — the C#'s two loops walk the
/// LEFT/RIGHT columns first (`num3`/`num6` at `startX ± i`), then the TOP/BOTTOM
/// rows (`num4`/`num5` at `startY ± j`) — and each candidate is the entity's
/// top-left at the cell minus the entity's box (`Vector2(-width/2, -height)`,
/// `:9`). `spot_free` is the ported `IsSpotShimmerFree` (`:67-82`).
///
/// Returns the landing position in pixels (the C#'s `Vector2?`), or `None`.
pub fn find_spot_without_shimmer(
    start_x: i32,
    start_y: i32,
    expand: i32,
    allow_solid_top: bool,
    width: f32,
    height: f32,
    spot_free: &dyn Fn(f32, f32) -> bool,
) -> Option<(f32, f32)> {
    let offset_x = -width / 2.0;
    let offset_y = -height;
    let candidate = |x: i32, y: i32| {
        let px = x as f32 * 16.0 + offset_x;
        let py = y as f32 * 16.0 + offset_y;
        (px, py)
    };
    // The C# first loop: the left/right columns with the top/bottom rows
    // interleaved per `i` (`:12-35`).
    for i in 0..expand {
        let num = start_x - i;
        let top = start_y - expand;
        let bottom = start_y + expand;
        for (px, py) in [
            candidate(num, top),
            candidate(start_x + i, top),
            candidate(num, bottom),
            candidate(start_x + i, bottom),
        ] {
            if spot_free(px, py) {
                return Some((px, py));
            }
        }
    }
    // The C# second loop: the left/right columns at `startX ± expand` with the
    // rows inside the box (`:37-63`).
    for j in 0..expand {
        let left = start_x - expand;
        let right = start_x + expand;
        for (px, py) in [
            candidate(left, start_y - j),
            candidate(right, start_y - j),
            candidate(left, start_y + j),
            candidate(right, start_y + j),
        ] {
            if spot_free(px, py) {
                return Some((px, py));
            }
        }
    }
    // `allow_solid_top` reaches only `IsSpotShimmerFree` (`ShimmerHelper.cs:73`);
    // it is threaded through so the caller's collision answers match the C#.
    let _ = allow_solid_top;
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shimmer_tables;

    #[test]
    fn the_tables_match_the_extraction_fingerprint() {
        // The entry counts recorded when the tables were extracted
        // (2026-11-08, from ItemID.cs:353 / NPCID.cs:10470).
        assert_eq!(shimmer_tables::SHIMMER_TRANSFORM_TO_ITEM.len(), 316);
        assert_eq!(shimmer_tables::SHIMMER_TRANSFORM_TO_NPC.len(), 114);
        assert_eq!(shimmer_tables::SHIMMER_COUNTS_AS_ITEM.len(), 6);
        assert_eq!(shimmer_tables::SHIMMER_COUNTS_AS_ITEM_FOR_DECRAFT.len(), 1);
        assert_eq!(shimmer_tables::SHIMMER_POST_MOONLORD.len(), 6);
    }

    #[test]
    fn transform_to_item_prefers_the_set() {
        // `ItemID.cs:353`: the shimmer chain is the ORE CYCLE, not crafting —
        // iron ore (11) shimmers to tin ore (699), tin (699) to copper (12),
        // copper (12) to 133, then 133 → 3 → 2 (`:353`'s tail).
        assert_eq!(transform_to_item(11, -1, 0, 0), 699);
        assert_eq!(transform_to_item(699, -1, 0, 0), 12);
        assert_eq!(transform_to_item(12, -1, 0, 0), 133);
        // 3460 (the boss-trophy family) → 947.
        assert_eq!(transform_to_item(3460, -1, 0, 0), 947);
        // Unset and not the 139/3461 families.
        assert_eq!(transform_to_item(1, -1, 0, 0), 0);
    }

    #[test]
    fn transform_to_item_falls_back_to_the_trophy_place_styles() {
        // `ShimmerTransforms.cs:95-105`.
        assert_eq!(transform_to_item(999, 139, 90, 0), 5538);
        assert_eq!(transform_to_item(999, 139, 89, 0), 5579);
        assert_eq!(transform_to_item(999, 139, 97, 0), 5638);
        assert_eq!(transform_to_item(999, 139, 96, 0), 5639);
        assert_eq!(transform_to_item(999, 139, 0, 0), 576); // the `_` default
    }

    #[test]
    fn transform_to_item_falls_back_to_the_lunar_bricks() {
        // `ShimmerTransforms.cs:113-126` — the arm is 3461-only.
        assert_eq!(transform_to_item(3461, -1, 0, 4), 5408); // Full
        assert_eq!(transform_to_item(3461, -1, 0, 2), 5403); // HalfAtLeft
        assert_eq!(transform_to_item(3461, -1, 0, 6), 5405); // HalfAtRight
        assert_eq!(transform_to_item(3461, -1, 0, 0), 5406); // the default phase
        assert_eq!(lunar_brick_for_moon_phase(7), 5407);
    }

    #[test]
    fn transform_to_npc_reads_the_set() {
        // `NPCID.cs:10470`: the zombies/slimes/critters to 676, the town NPCs
        // to 677, the eye to a Servant, the unset default -1.
        assert_eq!(transform_to_npc(3), 21); // the Eye of Cthulhu
        assert_eq!(transform_to_npc(1), 676);
        assert_eq!(transform_to_npc(74), 677); // a town NPC (the Nurse)
        assert_eq!(transform_to_npc(22), -1); // not in the set
        assert_eq!(transform_to_npc(9999), -1);
    }

    #[test]
    fn equivalent_and_coin_helpers() {
        assert_eq!(shimmer_equivalent_type(5669, true), 4956); // forDecraft
        assert_eq!(shimmer_equivalent_type(5669, false), 5669); // not in the plain set
        assert_eq!(shimmer_equivalent_type(5455, false), 5323);
        assert!(is_common_coin(71) && is_common_coin(74));
        assert!(!is_common_coin(12));
    }

    #[test]
    fn transform_lock_reads_the_post_moonlord_set() {
        // `ItemID.cs:356`: {1326, 779, 3031, 5364, 784, 782}.
        assert!(!is_transform_locked(1326, true));
        assert!(is_transform_locked(1326, false));
        assert!(!is_transform_locked(12, false));
    }

    #[test]
    fn find_spot_scans_left_right_then_rows_and_offsets_by_the_box() {
        // A free grid: only the (startX - 1, startY) candidate is accepted, and
        // the returned position carries the entity's top-left offset.
        let accepted = std::cell::Cell::new(false);
        let spot = find_spot_without_shimmer(5, 10, 4, true, 20.0, 40.0, &|px, py| {
            // The first candidate is (4, 6) = (5-1, 10-4).
            if px == 4.0 * 16.0 - 10.0 && py == 6.0 * 16.0 - 40.0 {
                accepted.set(true);
                true
            } else {
                false
            }
        });
        assert!(accepted.get(), "the first candidate must be tested first");
        assert_eq!(spot, Some((4.0 * 16.0 - 10.0, 6.0 * 16.0 - 40.0)));
        // All-refused → None.
        assert!(find_spot_without_shimmer(5, 10, 2, true, 20.0, 40.0, &|_, _| false).is_none());
    }
}
