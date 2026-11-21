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

/// `NPCID.Sets.ShimmerTransformToItem` (`NPCID.cs:10468`) — the NPC's shimmer
/// ITEM drop target, -1 when unset (`NPC.GetShimmered`'s third arm).
pub fn npc_transform_to_item(npc_type: u16) -> i32 {
    super::shimmer_tables::SHIMMER_NPC_TRANSFORM_TO_ITEM
        .iter()
        .find(|&&(ty, _)| ty == npc_type)
        .map(|&(_, target)| i32::from(target))
        .unwrap_or(-1)
}

/// `NPCID.Sets.ShimmerTownTransform` (`NPCID.cs:10469`) — the town-variant
/// swap membership (`NPC.GetShimmered`'s fourth arm).
pub fn is_shimmer_town_transform(npc_type: u16) -> bool {
    super::shimmer_tables::SHIMMER_TOWN_TRANSFORM.contains(&npc_type)
}

/// `NPC.UpdateNPC_BuffApplyVFX`'s shimmer tail (`NPC.cs:91514-91544`): the
/// transparency ramp, verbatim order. While `shimmering` (buff 353 active):
/// +0.01 per tick, the `> 0.9` DOUBLE-compare crossing flag (the caller fires
/// `GetShimmered` — the C# gates it `netMode != 1`, the port is host-side),
/// then the 1.0 clamp. Else, with transparency > 0: `justHit` −0.1 FIRST,
/// then buff-immune[353] −0.015 else −0.001, then the 0 floor. Returns
/// `(new_transparency, crossed_0_9)` — the crossing is computed on the
/// PRE-clamp value, exactly like the C# `if` chain.
pub fn npc_shimmer_transparency_tick(
    shimmering: bool,
    transparency: f32,
    just_hit: bool,
    buff_immune_353: bool,
) -> (f32, bool) {
    let mut t = transparency;
    let mut crossed = false;
    if shimmering {
        t += 0.01; // :91516
        // :91517 — `(double)this.shimmerTransparency > 0.9`: the f32 sum
        // widened to f64 BEFORE the compare (the 0.9 literal is a double).
        if f64::from(t) > 0.9 {
            crossed = true;
        }
        if t > 1.0 {
            t = 1.0; // :91521-91524
        }
    } else if t > 0.0 {
        if just_hit {
            t -= 0.1; // :91528-91531
        }
        if buff_immune_353 {
            t -= 0.015; // :91532-91535
        } else {
            t -= 0.001; // :91536-91539
        }
        if t < 0.0 {
            t = 0.0; // :91540-91543
        }
    }
    (t, crossed)
}

/// What `NPC.GetShimmered` (`NPC.cs:91548-91592`) does, in the C# else-if
/// order. The caller executes: the statue poof and the item-drop arms also
/// set `NPC.noSpawnCycle = true` and deactivate; the transform arm rides
/// `NPC.Transform` + `WorldItem.ShimmerEffect`; the town arm zeroes the ai,
/// sets `shimmerTransparency = 0.89f` and deletes buff 353.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcShimmerOutcome {
    /// No arm matches (the C# falls through silently).
    Nothing,
    /// `SpawnedFromStatue` (`:91550-91560`): ShimmerEffect + noSpawnCycle +
    /// active = false (the server also zeroes life / sends 23).
    StatuePoof,
    /// `ShimmerTransformToNPC[type] >= 0` (`:91561-91565`).
    TransformNpc { new_type: u16 },
    /// `ShimmerTransformToItem[type] >= 0` (`:91566-91577`):
    /// SpawnShimmeredItem(stack 0 → 1) + ShimmerEffect + noSpawnCycle +
    /// active = false.
    DropItem { item_type: u16 },
    /// `ShimmerTownTransform[type]` (`:91578-91591`): ai[0]=25, ai[1..3]=0,
    /// netUpdate, transparency 0.89, FindBuffIndex(353) → DelBuff.
    TownVariant,
}

/// `NPC.GetShimmered` (`NPC.cs:91548-91592`) as a decision function — the
/// branch order IS the resolution order.
pub fn resolve_npc_shimmer(npc_type: u16, spawned_from_statue: bool) -> NpcShimmerOutcome {
    if spawned_from_statue {
        return NpcShimmerOutcome::StatuePoof; // :91550
    }
    let to_npc = transform_to_npc(npc_type);
    if to_npc >= 0 {
        return NpcShimmerOutcome::TransformNpc { new_type: to_npc as u16 }; // :91561
    }
    let to_item = npc_transform_to_item(npc_type);
    if to_item >= 0 {
        return NpcShimmerOutcome::DropItem { item_type: to_item as u16 }; // :91566
    }
    if is_shimmer_town_transform(npc_type) {
        return NpcShimmerOutcome::TownVariant; // :91578
    }
    NpcShimmerOutcome::Nothing
}

/// `NPC.AppearsFriendlyToHunterPotion` (`NPC.cs:91008-91017`):
/// `friendly || catchItem > 0 || (damage == 0 && lifeMax == 5)`, then the
/// `NPCID.Sets.HunterPotionFriendlyOverride` map — the ONE row is 689 →
/// false (`NPCID.cs:10128-10132`, the Princess's shimmer variant reads
/// hostile to the potion).
pub fn appears_friendly_to_hunter_potion(
    npc_type: u16,
    friendly: bool,
    catch_item: i32,
    damage: i32,
    life_max: i32,
) -> bool {
    let mut result = friendly || catch_item > 0 || (damage == 0 && life_max == 5);
    if npc_type == 689 {
        result = false; // the override row
    }
    result
}

/// `NPC.CurrentlyShimmerTransparent` (`NPC.cs:90995-91006`): transparency
/// > 0 AND (no detectCreature → true; detectCreature → the friendly
/// predicate). `detect_creature` is `Main.SceneMetrics.PerspectivePlayer.
/// detectCreature` (`CanApplyHunterPotionEffects`, `:91019-91022`).
pub fn currently_shimmer_transparent(
    transparency: f32,
    detect_creature: bool,
    npc_type: u16,
    friendly: bool,
    catch_item: i32,
    damage: i32,
    life_max: i32,
) -> bool {
    if transparency > 0.0 {
        if detect_creature {
            return appears_friendly_to_hunter_potion(
                npc_type, friendly, catch_item, damage, life_max,
            );
        }
        return true;
    }
    false
}

/// `NPC.UpdateNPC_UpdateGravity`'s wet tail (`NPC.cs:90937-90954`): when
/// `base.wet`, the gravity/cap pair is OVERRIDDEN (after the depth scaling
/// at `:90926-90936`) — shimmer 0.15/5.5 first, honey 0.1/4, else 0.2/7.
/// Returns `Some((gravity, max_fall))` while wet, `None` dry (the caller
/// keeps the depth-scaled pair).
pub fn npc_wet_gravity(wet: bool, shimmer_wet: bool, honey_wet: bool) -> Option<(f32, f32)> {
    if !wet {
        return None;
    }
    if shimmer_wet {
        Some((0.15, 5.5)) // :90939-90943
    } else if honey_wet {
        Some((0.1, 4.0)) // :90944-90948
    } else {
        Some((0.2, 7.0)) // :90949-90953
    }
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

    /// Doc-sweep lane 36 — the transform-table COMPLETENESS census: every
    /// entry pair re-checked against the decompiled sets' heads/tails and
    /// the family cycles (the mechanical 316/114-pair diff ran clean at
    /// sweep time — this pins the load-bearing rows so a future extraction
    /// drift fails by name). The COIN side is NOT a transform-table arm:
    /// `GetShimmered`'s first branch (`WorldItem.cs:1697-1718`) reads
    /// `ItemID.Sets.CommonCoin` (`ItemID.cs:1318`, {71,72,73,74}) and the
    /// tier multipliers live in the chain (silver x100 / gold x10000 /
    /// platinum floor-1 x1000000), NOT in `ShimmerTransformToItem` — the
    /// coins have NO set entries (71-74 absent), so `transform_to_item`
    /// returns 0 for them and the coin arm wins by order.
    #[test]
    fn w4_f47ds_the_transform_table_census_pins_the_family_rows() {
        use shimmer_tables::SHIMMER_TRANSFORM_TO_ITEM as T;
        let get = |k: u16| T.iter().find(|&&(a, _)| a == k).map(|&(_, b)| b);
        // The ore cycle head/tail (`ItemID.cs:353`'s first 18 pairs).
        assert_eq!(get(3460), Some(947));
        assert_eq!(get(3), Some(2));
        assert_eq!(get(9), Some(2));
        // The gem cycle (182 -> 178 -> 179 -> 177 -> 180 -> 181 -> 133).
        for (a, b) in [(182, 178), (178, 179), (179, 177), (177, 180), (180, 181), (181, 133)] {
            assert_eq!(get(a), Some(b), "gem cycle {a}");
        }
        // The torch/campfire family: every torch kind -> 5353 (23 rows,
        // the mechanical recount of `ItemID.cs:353`'s 8..5293 band).
        let torch_rows = T.iter().filter(|&&(_, b)| b == 5353).count();
        assert_eq!(torch_rows, 23, "the 5353 family's row count");
        assert_eq!(get(8), Some(5353));
        assert_eq!(get(5293), Some(5353));
        // The evil-powder/purification pairs and the biome-solution family.
        assert_eq!(get(67), Some(66));
        assert_eq!(get(5127), Some(5128));
        // The post-Moonlord relic pairs (the lock set's members all have
        // transform rows — the lock gates them, not the table).
        for k in shimmer_tables::SHIMMER_POST_MOONLORD {
            assert!(get(*k).is_some(), "post-Moonlord {k} has a row");
        }
        // The tail pairs (the Extractinator junk / coin-grenade family).
        assert_eq!(get(824), Some(5996));
        assert_eq!(get(5997), Some(825));
        assert_eq!(get(576), Some(6146));
        // The COIN kinds are NOT in the set (the chain's CommonCoin arm).
        for k in [71u16, 72, 73, 74] {
            assert_eq!(get(k), None, "coin {k} rides the CommonCoin arm");
            assert!(is_common_coin(k));
        }
        // The NPC set's load-bearing rows: the 17 ->676 slime/critter rows
        // and the 82 ->677 critter rows (the mechanical recount of
        // `NPCID.cs:10470` — the doc-sweep item-4 census).
        let n676 = shimmer_tables::SHIMMER_TRANSFORM_TO_NPC
            .iter()
            .filter(|&&(_, b)| b == 676)
            .count();
        let n677 = shimmer_tables::SHIMMER_TRANSFORM_TO_NPC
            .iter()
            .filter(|&&(_, b)| b == 677)
            .count();
        assert_eq!(n676, 17, "the ->676 rows (NPCID.cs:10470)");
        assert_eq!(n677, 82, "the ->677 rows");
        assert_eq!(transform_to_npc(591), 449);
        assert_eq!(transform_to_npc(436), 452);
        assert_eq!(transform_to_npc(432), 450);
        assert_eq!(transform_to_npc(435), 451);
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

    #[test]
    fn the_npc_set_fingerprints_match_the_extraction() {
        // NPCID.cs:10468 — 15 pairs; :10469 — 29 ids.
        assert_eq!(shimmer_tables::SHIMMER_NPC_TRANSFORM_TO_ITEM.len(), 15);
        assert_eq!(shimmer_tables::SHIMMER_TOWN_TRANSFORM.len(), 29);
        // The gem critters → their gems, the moths → amber, 448 → the RoD.
        assert_eq!(npc_transform_to_item(651), 182); // diamond critter
        assert_eq!(npc_transform_to_item(639), 181); // amethyst critter
        assert_eq!(npc_transform_to_item(652), 999);
        assert_eq!(npc_transform_to_item(448), 5341);
        assert_eq!(npc_transform_to_item(1), -1);
        // The town set: the Guide (22) yes, the shimmer slime (676) no.
        assert!(is_shimmer_town_transform(22));
        assert!(is_shimmer_town_transform(368));
        assert!(!is_shimmer_town_transform(676));
        assert!(!is_shimmer_town_transform(1));
    }

    #[test]
    fn the_transparency_ramp_rises_and_crosses_at_the_double_compare() {
        // +0.01 per shimmering tick (:91516).
        let (t, c) = npc_shimmer_transparency_tick(true, 0.0, false, false);
        assert_eq!(t, 0.01);
        assert!(!c);
        // The f32 accumulation quirk: f32(0.89) + f32(0.01) rounds to
        // EXACTLY f32(0.9) = 0.89999997615814… which widened to f64 is
        // still BELOW the 0.9 double literal (:91517) — no crossing yet.
        let (t, c) = npc_shimmer_transparency_tick(true, 0.89, false, false);
        assert_eq!(t, 0.9f32);
        assert!(!c, "f32 0.9 as f64 is below the double 0.9 literal");
        // One more tick lands above → the crossing fires.
        let (t, c) = npc_shimmer_transparency_tick(true, t, false, false);
        assert!(c);
        assert!(f64::from(t) > 0.9);
        // The clamp at 1.0 (:91521-91524).
        let (t, _) = npc_shimmer_transparency_tick(true, 1.0, false, false);
        assert_eq!(t, 1.0);
    }

    #[test]
    fn the_transparency_decay_is_just_hit_then_the_immune_split() {
        // justHit −0.1 FIRST, then the −0.015 / −0.001 split (:91528-91539).
        let (t, c) = npc_shimmer_transparency_tick(false, 1.0, true, false);
        assert!((t - (1.0 - 0.1 - 0.001)).abs() < 1e-6);
        assert!(!c);
        let (t, _) = npc_shimmer_transparency_tick(false, 1.0, false, true);
        assert!((t - (1.0 - 0.015)).abs() < 1e-6);
        // The 0 floor; a dry zero stays zero (no decay arm runs).
        let (t, _) = npc_shimmer_transparency_tick(false, 0.05, true, true);
        assert_eq!(t, 0.0);
        let (t, _) = npc_shimmer_transparency_tick(false, 0.0, true, false);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn the_npc_resolution_order_is_statue_npc_item_town() {
        // The statue wins over every set (:91550).
        assert_eq!(resolve_npc_shimmer(1, true), NpcShimmerOutcome::StatuePoof);
        // The NPC set (:91561): the blue slime → 676, the eye → 21.
        assert_eq!(
            resolve_npc_shimmer(1, false),
            NpcShimmerOutcome::TransformNpc { new_type: 676 }
        );
        // The ITEM set (:91566): the diamond critter 651 → 182.
        assert_eq!(
            resolve_npc_shimmer(651, false),
            NpcShimmerOutcome::DropItem { item_type: 182 }
        );
        // The town set (:91578): the Guide 22.
        assert_eq!(resolve_npc_shimmer(22, false), NpcShimmerOutcome::TownVariant);
        // Unset everywhere → nothing (the C# falls through).
        assert_eq!(resolve_npc_shimmer(9999, false), NpcShimmerOutcome::Nothing);
    }

    #[test]
    fn the_hunter_predicate_and_the_transparency_gate() {
        // friendly || catchItem > 0 || (damage == 0 && lifeMax == 5).
        assert!(appears_friendly_to_hunter_potion(1, true, 0, 10, 100));
        assert!(appears_friendly_to_hunter_potion(1, false, 300, 10, 100));
        assert!(appears_friendly_to_hunter_potion(1, false, 0, 0, 5));
        assert!(!appears_friendly_to_hunter_potion(1, false, 0, 10, 100));
        // The 689 override row (NPCID.cs:10128-10132) beats friendly.
        assert!(!appears_friendly_to_hunter_potion(689, true, 300, 0, 5));
        // transparency 0 → never transparent, whatever the potion says.
        assert!(!currently_shimmer_transparent(0.0, true, 1, true, 0, 0, 5));
        // No detectCreature → any positive transparency hides the NPC.
        assert!(currently_shimmer_transparent(0.5, false, 1, false, 0, 10, 100));
        // detectCreature → the friendly predicate decides.
        assert!(currently_shimmer_transparent(0.5, true, 1, true, 0, 10, 100));
        assert!(!currently_shimmer_transparent(0.5, true, 1, false, 0, 10, 100));
    }

    #[test]
    fn the_wet_gravity_tail_overrides_in_shimmer_honey_water_order() {
        // NPC.cs:90937-90954 — dry keeps the caller's pair.
        assert_eq!(npc_wet_gravity(false, true, true), None);
        assert_eq!(npc_wet_gravity(true, true, true), Some((0.15, 5.5)));
        assert_eq!(npc_wet_gravity(true, false, true), Some((0.1, 4.0)));
        // Lava has no own arm — the water else covers it (:90949-90953).
        assert_eq!(npc_wet_gravity(true, false, false), Some((0.2, 7.0)));
    }
}
