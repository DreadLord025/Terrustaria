//! The `ItemID.Sets` / `TileID.Sets` rows the item tooltip reads
//! (`Main.MouseText_DrawItemTooltip_GetLinesInfo`, `Main.cs:20631-21343`).
//! Pure membership tables only — the line builder lives in
//! [`super::tooltip`]. `ItemDef` (the generated table) carries no columns
//! for these flags, so the census keeps them here next to their consumer
//! (docs/research/64-item-tooltip-line-census.md).
//!
//! DEPRECATED-BY-UNION IN PART (docs/research/128 §3): the four tables
//! marked below (`has_vanity_effects`, `expert_only`, `tile_wand`,
//! `chlorophyte_extractinator_consumable`) retire into f28's GenItemData
//! columns at the f28c merge, and the wiring tile slices lose to f28's
//! generated bitsets. The PURE ID-set tables (well-fed, loadout-share
//! contexts, bank block, cursed keys, yoyo/DD2 kinds, alt-use, statue
//! carve-out) SURVIVE — this module stays their ONE home; f28's inline
//! copies dedupe INTO it. Nothing is deleted on this branch.

/// `BuffID.Sets.IsWellFed` (`BuffID.cs:43`): the three food buffs whose
/// items print the expert life-regen row (`Main.cs:21005-21009`).
pub(super) fn is_well_fed(buff_type: i32) -> bool {
    matches!(buff_type, 26 | 206 | 207)
}

/// `Item.hasVanityEffects` — the 21 writes: the direct kinds
/// (`Item.cs:7164,7307,11468,21270,28801,30254,32304,34739,38188,39183,
/// 39687,40271,40689,40698,40962,42290,42988,43719`), `DefaultToGolfBall`
/// (`:35459` — 3989, 4242-4255), `DefaultToInfoAccessory` (`:48622` — the
/// watches/compass/GPS family) and `DefaultToMusicBox` (`:48650` — also
/// `vanity=true`, so the social row's `!vanity` gate already excludes
/// them). The social row (`Main.cs:20664`) reads it.
/// DEPRECATED-BY-UNION (128 §3): retires into f28's generated
/// `d.has_vanity_effects` column (regen + FNV-pin `0x1dd2_84fc_4b36_2f46`).
pub(super) fn has_vanity_effects(kind: u16) -> bool {
    matches!(
        kind,
        485 | 497
            | 861
            | 1987
            | 3110
            | 3305
            | 3581
            | 3929
            | 4563
            | 4762
            | 4874
            | 5000
            | 5075
            | 5076
            | 5113
            | 5331
            | 5452
            | 5547
            // DefaultToGolfBall.
            | 3989
            | 4242..=4255
            // DefaultToInfoAccessory.
            | 15..=18
            | 393
            | 395
            | 707..=709
            | 3036
            | 3037
            | 3084
            | 3095
            | 3096
            | 3099
            | 3102
            | 3118..=3123
            // DefaultToMusicBox (vanity=true as well).
            | 562..=574
            | 1596..=1610
            | 1963..=1965
            | 2742
            | 3044
            | 3235..=3237
            | 3370
            | 3371
            | 3796
            | 3869
            | 4077..=4082
            | 4237
            | 4356..=4358
            | 4421
            | 4606
            | 4979
            | 4985
            | 4990..=4992
            | 5006
            | 5014..=5040
            | 5044
            | 5112
            | 5362
            | 5538
            | 5539
            | 5578..=5582
            | 5637..=5639
            | 6144..=6146
    )
}

/// `ItemSlot.canLoadoutShareAt` (`ItemSlot.cs:395-399`): contexts 8-12
/// (the armour/vanity/accessory/dye columns) — the loadout-share arm of
/// the tooltip head (`Main.cs:20638`).
pub(super) fn can_loadout_share_at(context: i32) -> bool {
    matches!(context, 8..=12)
}

/// `ChestUI.IsBlockedFromTransferIntoChest` (`ChestUI.cs:1291-1302`): the
/// favorited Money Trough 3213 refuses the piggy bank (bank index 0), the
/// favorited Void Bag 4131 / Void Vault 5325 refuse bank4 (index 3). The
/// tooltip prints `UI.ItemCannotBePlacedInsideItself` (`Main.cs:20659`).
pub(super) fn blocked_from_transfer_into_chest(kind: u16, bank: Option<usize>) -> bool {
    match bank {
        Some(0) => kind == 3213,
        Some(3) => kind == 4131 || kind == 5325,
        _ => false,
    }
}

/// `ItemID.Sets.UsesCursedByPlanteraTooltip` (`ItemID.cs:410`): the biome
/// keys + the Lightning Carrot — their first tooltip line becomes
/// `Lang.tip[59]` until Plantera falls (`Main.cs:20961-20965`).
pub(super) fn uses_cursed_by_plantera_tooltip(kind: u16) -> bool {
    matches!(kind, 1533 | 1534 | 1535 | 1536 | 1537 | 4714)
}

/// `Item.expert` — the 25 `this.expert = true` writes in `Item.SetDefaults`
/// (`Item.cs:13923,28599,28647,29748,29757,29898,30614,30626,30647,30655,
/// 30663,30669,30678,30781,32271,34163,37575,39327,39951,40025,40183,40201,
/// 40881,40952,41995`), all unconditional in their case bodies. The
/// `GameUI.Expert` row (`Main.cs:21022-21026`) and the −12 disco name
/// colour (`:20329-20332`) read it; `ItemDef` carries no column.
/// DEPRECATED-BY-UNION (128 §3): retires into f28's generated `d.expert`
/// column — the census would silently rot when a wave touches `Item.cs`.
pub(super) fn expert_only(kind: u16) -> bool {
    matches!(
        kind,
        1131 | 3090
            | 3097
            | 3223
            | 3224
            | 3245
            | 3318..=3333
            | 3335
            | 3336
            | 3337
            | 3353..=3356
            | 3367
            | 3577
            | 3860..=3862
            | 4444
            | 4782
            | 4954
            | 4957
            | 4987
            | 4989
            | 5100
            | 5111
            | 5289
    )
}

/// The yoyo-logo kinds (`Main.cs:21016-21021`): a blank line is appended
/// and its index becomes `yoyoLogo` (the draw pass swaps it for the
/// OneDropLogo texture, `:20410-20438` — unported, the blank line keeps
/// the geometry).
pub(super) fn yoyo_logo(kind: u16) -> bool {
    matches!(
        kind,
        3262 | 3282..=3286 | 3315 | 3316 | 3317 | 3291 | 3389
    )
}

/// The DD2 tower/popper kinds whose tooltip prints `Lang.misc[104]` until
/// the Eternia Crystal has been defended (`Main.cs:21000-21004`).
pub(super) fn dd2_needs_etherian_mana(kind: u16) -> bool {
    matches!(
        kind,
        3818 | 3819 | 3820 | 3824 | 3825 | 3826 | 3829 | 3830 | 3831 | 3832 | 3833 | 3834
    )
}

/// `Item.tileWand` (`Item.cs:113`, stock −1): the six kinds that consume a
/// material to place their tile — 832/933 → Wood 9 (`Item.cs:11149`/
/// `:12161`), 932 → Gel 154 (`:12148`), 1129 → Fallen Star 1124 (`:13888`),
/// 3360/3361 → Bone 620 (`:30697`/`:30710`). The tooltip prints
/// `Lang.tip[52] + Lang.GetItemNameValue(tileWand)` (`Main.cs:20828-20832`).
/// DEPRECATED-BY-UNION (128 §3): retires into f28's generated `d.tile_wand`
/// column; the six-write witness retires with it (§6 sets-tables row).
pub(super) fn tile_wand(kind: u16) -> Option<u16> {
    match kind {
        832 | 933 => Some(9),
        932 => Some(154),
        1129 => Some(1124),
        3360 | 3361 => Some(620),
        _ => None,
    }
}

/// `ItemID.Sets.PlaceTileOnAltUse` (`ItemID.cs:412`): the placeable kinds
/// whose "Can be placed" row shows even without `consumable`
/// (`Main.cs:20921`).
pub(super) fn place_tile_on_alt_use(kind: u16) -> bool {
    const SET: &[u16] = &[
        261, 1338, 1992, 1994, 1995, 1996, 1997, 1998, 1999, 2000, 2001, 2002, 2003, 2004, 2006,
        2007, 2015, 2016, 2017, 2018, 2019, 2121, 2122, 2123, 2156, 2157, 2205, 2673, 2740, 2889,
        2890, 2891, 2892, 2893, 2894, 2895, 3191, 3192, 3193, 3194, 3563, 3564, 4068, 4069, 4070,
        4274, 4334, 4335, 4336, 4337, 4338, 4339, 4340, 4359, 4361, 4362, 4363, 4367, 4368, 4369,
        4370, 4371, 4373, 4374, 4375, 4379, 4395, 4418, 4419, 4464, 4465, 4480, 4482, 4610, 4611,
        4612, 4613, 4648, 4649, 4650, 4651, 4669, 4670, 4671, 4674, 4675, 4676, 4677, 4681, 4683,
        4684, 4831, 4832, 4833, 4834, 4835, 4836, 4837, 4838, 4839, 4840, 4841, 4842, 4843, 4844,
        4845, 4847, 4849, 4961, 5132, 5212, 5300, 5311, 5312, 5313, 5350, 5511,
    ];
    SET.contains(&kind)
}

/// `Item.chlorophyteExtractinatorConsumable` — set only by
/// `MakeUsableWithChlorophyteExtractinator` (`Item.cs:48505-48513`), called
/// for 86 (`:2405`), 1329 (`:16477`) and 2337-2339 (`:23497`). These refuse
/// the plain "Consumable" row (`Main.cs:20932`).
/// DEPRECATED-BY-UNION (128 §3): retires into f28's generated
/// `d.extractinator_consumable` column.
pub(super) fn chlorophyte_extractinator_consumable(kind: u16) -> bool {
    matches!(kind, 86 | 1329 | 2337 | 2338 | 2339)
}

/// `TileID.Sets.Wiring.IsAMechanism` (`TileID.cs:184`).
fn wiring_is_a_mechanism(tile: u16) -> bool {
    const SET: &[u16] = &[
        4, 10, 11, 21, 33, 34, 35, 42, 92, 93, 95, 100, 105, 126, 130, 131, 137, 139, 141, 142,
        143, 149, 207, 209, 210, 212, 215, 216, 219, 235, 244, 255, 256, 257, 258, 259, 260, 261,
        262, 263, 264, 265, 266, 267, 268, 314, 335, 338, 349, 356, 386, 387, 388, 389, 405, 406,
        411, 419, 420, 421, 422, 425, 429, 443, 452, 455, 467, 480, 497, 506, 509, 531, 546, 557,
        564, 565, 593, 594, 642, 657, 658, 663, 720, 721, 725, 733,
    ];
    SET.contains(&tile)
}

/// `TileID.Sets.Wiring.IsATrigger` (`TileID.cs:185`).
fn wiring_is_a_trigger(tile: u16) -> bool {
    const SET: &[u16] = &[132, 135, 136, 144, 411, 423, 440, 441, 468, 476];
    SET.contains(&tile)
}

/// `ItemID.Sets.IsWireableStatue` (`ItemID.cs:1581`): the statue kinds whose
/// tile 105 still counts as wireable (`Main.cs:20942`'s carve-out).
fn is_wireable_statue(kind: u16) -> bool {
    const SET: &[u16] = &[
        438, 440, 441, 443, 444, 445, 446, 449, 452, 453, 454, 459, 463, 464, 466, 470, 471, 473,
        476, 477, 478, 2672, 3651, 3652, 3653, 3654, 3655, 3656, 3657, 3658, 3659, 3660, 3661,
        3662, 3708, 3709, 3710, 3711, 3712, 3713, 3714, 3715, 3716, 3717, 3718, 3719, 3720, 4276,
        4342, 4355, 4360, 4397, 4466, 5317, 5318, 5319,
    ];
    SET.contains(&kind)
}

/// The "Wireable" row's gate (`Main.cs:20942`):
/// `(IsATrigger[tile] || IsAMechanism[tile]) & (tile != 105 || IsWireableStatue[type])`.
/// DEPRECATED-BY-UNION IN PART (128 §2 row 18 / §3): the two hand-copied
/// tile slices lose to f28's generated `SET_IS_A_TRIGGER` /
/// `SET_IS_A_MECHANISM` bitsets; the `is_wireable_statue` carve-out
/// SURVIVES here as the ONE statue table (f28's inline copy dedupes in).
pub(super) fn wireable(kind: u16, tile: Option<u16>) -> bool {
    let Some(t) = tile else {
        return false;
    };
    (wiring_is_a_trigger(t) || wiring_is_a_mechanism(t)) && (t != 105 || is_wireable_statue(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The six tileWand writes (`Item.cs:11149/12148/12161/13888/30697/30710`).
    #[test]
    fn w4_e16_the_tile_wand_table_matches_the_six_writes() {
        assert_eq!(tile_wand(832), Some(9));
        assert_eq!(tile_wand(932), Some(154));
        assert_eq!(tile_wand(933), Some(9));
        assert_eq!(tile_wand(1129), Some(1124));
        assert_eq!(tile_wand(3360), Some(620));
        assert_eq!(tile_wand(3361), Some(620));
        assert_eq!(tile_wand(1), None);
    }

    /// `Main.cs:20942`: a mechanism tile is wireable; tile 105 only for the
    /// statue set (`ItemID.cs:1581`).
    #[test]
    fn w4_e16_the_wireable_gate_carves_out_the_statues() {
        assert!(wireable(48, Some(21)), "the chest is a mechanism");
        assert!(wireable(529, Some(135)), "the pressure plate is a trigger");
        assert!(wireable(441, Some(105)), "the Goblin Statue is wireable");
        assert!(!wireable(442, Some(105)), "a non-set statue kind is not");
        assert!(!wireable(2, Some(0)), "dirt is neither");
        assert!(!wireable(1, None));
    }

    /// `ItemID.cs:412` / `Item.cs:48505-48513` spot rows.
    #[test]
    fn w4_e16_the_alt_use_and_extractinator_sets() {
        assert!(place_tile_on_alt_use(4367), "the Blue Kite");
        assert!(place_tile_on_alt_use(261));
        assert!(!place_tile_on_alt_use(2), "dirt rides consumable instead");
        assert!(chlorophyte_extractinator_consumable(86));
        assert!(chlorophyte_extractinator_consumable(2339));
        assert!(!chlorophyte_extractinator_consumable(28));
    }
}