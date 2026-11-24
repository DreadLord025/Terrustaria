//! W4-d111 witness pins for the generated `TileObjectData` registry — the
//! "port missing defs" task found the registry ALREADY COMPLETE (S1-F-7 wave A,
//! `tools/GenTileObjectData/gen.py`, 389 tiles / 604 defs), so this batch pins
//! verbatim C# def values instead of hand-porting duplicates.
//!
//! Batch 1 range: TileID order kinds 3..=270 — the first 120 registered tiles
//! (`REGISTERED_KINDS[..120]`, 3 through 270 inclusive), plus the full 389-kind
//! coverage census. Each pinned def below is transcribed field-for-field from
//! the cited `TileObjectData.cs` `Initialize` block (the copy-on-write chain
//! resolved by hand, cross-checked against `registry.rs`). Later batches
//! (d112/d113/d114/e11/e12) pin the next ranges; nothing here is missing data.

use tr_core::tileobject::{
    def_for_tile, AnchorData, GetStyleOverride, LiquidPlacement, TileObjectDirection,
    TABLE,
};

/// Every `TileObjectData.addTile` id of `Initialize`
/// (`TileObjectData.cs:2110-5014`): the 384 literal `addTile(n)` calls plus the
/// `for (int j = 435; j <= 439; j++)` loop (`:2149-2160`), in TileID order.
/// This is the census the W4-d11x "missing defs" tasks were cut against — the
/// generated registry covers ALL of them (S1-F-7 wave A), witnessed here.
const REGISTERED_KINDS: [u16; 389] = [
    3, 4, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 24, 26, 27, 28, 29, 31, 33, 34, 35, 36,
    42, 49, 50, 55, 61, 71, 73, 74, 77, 78, 79, 81,
    82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
    94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105,
    106, 110, 113, 114, 125, 126, 128, 132, 133, 134, 135, 136,
    138, 139, 141, 142, 143, 144, 149, 165, 171, 172, 173, 174,
    178, 184, 185, 186, 187, 201, 207, 209, 210, 212, 215, 216,
    217, 218, 219, 220, 227, 228, 231, 233, 235, 236, 237, 238,
    239, 240, 241, 242, 243, 244, 245, 246, 247, 254, 269, 270,
    271, 275, 276, 277, 278, 279, 280, 281, 282, 283, 285, 286,
    287, 288, 289, 290, 291, 292, 293, 294, 295, 296, 297, 298,
    299, 300, 301, 302, 303, 304, 305, 306, 307, 308, 309, 310,
    316, 317, 318, 319, 320, 324, 334, 335, 337, 338, 339, 349,
    354, 355, 356, 358, 359, 360, 361, 362, 363, 364, 372, 373,
    374, 375, 376, 377, 378, 386, 387, 388, 389, 390, 391, 392,
    393, 394, 395, 405, 406, 410, 411, 412, 413, 414, 419, 420,
    423, 424, 425, 427, 428, 429, 435, 436, 437, 438, 439, 440,
    441, 442, 443, 444, 445, 452, 453, 454, 455, 456, 457, 461,
    462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 475, 476,
    480, 484, 485, 486, 487, 488, 489, 490, 491, 493, 494, 497,
    499, 505, 506, 509, 510, 511, 520, 521, 522, 523, 524, 525,
    526, 527, 530, 531, 532, 533, 538, 542, 543, 544, 545, 547,
    548, 550, 551, 552, 553, 554, 555, 556, 558, 559, 560, 564,
    565, 567, 568, 569, 570, 572, 573, 579, 580, 581, 582, 590,
    591, 592, 593, 594, 595, 597, 598, 599, 600, 601, 602, 603,
    604, 605, 606, 607, 608, 609, 610, 611, 612, 613, 614, 615,
    617, 619, 620, 621, 622, 623, 624, 629, 630, 631, 632, 637,
    639, 640, 642, 643, 644, 645, 646, 647, 648, 649, 650, 651,
    652, 653, 654, 656, 657, 658, 660, 663, 664, 665, 693, 694,
    695, 696, 698, 699, 700, 701, 702, 703, 704, 705, 706, 707,
    709, 710, 711, 712, 713, 714, 715, 716, 720, 721, 723, 724,
    725, 733, 751, 752, 753,
];

/// W4-d111 batch range: the first 120 registered kinds, 3..=270 in TileID
/// order. d112 takes `REGISTERED_KINDS[120..]` onward (271+).
const BATCH1_END: usize = 120;

#[test]
fn w4_d111_every_registered_kind_resolves() {
    assert_eq!(TABLE.len(), REGISTERED_KINDS.len());
    for (i, &kind) in REGISTERED_KINDS.iter().enumerate() {
        let def = def_for_tile(kind)
            .unwrap_or_else(|| panic!("kind {kind} must resolve (addTile census)"));
        assert_eq!(def.kind, kind);
        assert_eq!(TABLE[i].kind, kind, "TABLE stays in TileID order");
    }
    assert_eq!(REGISTERED_KINDS[BATCH1_END - 1], 270, "batch 1 ends at 270");
    assert_eq!(REGISTERED_KINDS[BATCH1_END], 271, "d112 starts at 271");
}

// ---- batch 1 verbatim def pins (TileObjectData.cs Initialize) ----

#[test]
fn w4_d111_platform_19_base_and_subtile() {
    // TileObjectData.cs:2127-2139 — the Initialize defaults (CoordinateHeights
    // {16}, width 16, padding 2, StyleHorizontal, wrap/multiplier 27,
    // UsesCustomCanPlace false, LavaDeath true) + addSubTile(13, 43, 47) with
    // the sub-tile's LavaDeath=false / LavaPlacement=Allowed (:2135-2137).
    let d = def_for_tile(19).unwrap();
    assert_eq!((d.width, d.height), (1, 1));
    assert_eq!(d.coordinate_heights, &[16]);
    assert_eq!(d.coordinate_width, 16);
    assert_eq!(d.coordinate_padding, 2);
    assert!(d.style_horizontal);
    assert_eq!((d.style_wrap_limit, d.style_multiplier), (27, 27));
    assert!(!d.uses_custom_can_place);
    assert!(d.lava_death);
    assert_eq!(d.lava_placement, LiquidPlacement::NotAllowed);
    assert_eq!(d.subtiles.len(), 3);
    for &(style, sub) in d.subtiles {
        assert!([13u16, 43, 47].contains(&style));
        assert!(!sub.lava_death, "sub-tile LavaDeath reset (:2136)");
        assert_eq!(sub.lava_placement, LiquidPlacement::Allowed);
        assert_eq!(sub.style_multiplier, 27);
    }
}

#[test]
fn w4_d111_closed_door_10_alternates_and_subtile() {
    // TileObjectData.cs:2223-2247 — 1x3, SolidTile top+bottom anchors,
    // wrap 36 / line-skip 3, two origin-shifted alternates (:2236-2241),
    // LinkedAlternates sub-tile at styles 19/48 (:2242-2246).
    let d = def_for_tile(10).unwrap();
    assert_eq!((d.width, d.height), (1, 3));
    assert_eq!(d.anchor_top, AnchorData::new(1, 1, 0));
    assert_eq!(d.anchor_bottom, AnchorData::new(1, 1, 0));
    assert!(d.uses_custom_can_place);
    assert_eq!((d.style_wrap_limit, d.style_line_skip), (36, 3));
    assert!(!d.style_horizontal);
    assert_eq!(d.alternates.len(), 2);
    assert_eq!(d.alternates[0].origin.y, 1);
    assert_eq!(d.alternates[1].origin.y, 2);
    assert_eq!(d.subtiles.len(), 2);
    assert_eq!(d.subtiles[0].0, 19);
    assert_eq!(d.subtiles[1].0, 48);
    let sub = d.subtiles[0].1;
    assert_eq!(sub.lava_placement, LiquidPlacement::Allowed);
    assert_eq!(sub.alternates.len(), 2, "LinkedAlternates copies (:2243)");
}

#[test]
fn w4_d111_open_door_11_direction_alternates() {
    // TileObjectData.cs:2248-2291 — 2x3 PlaceRight, wrap 36 / line-skip 2,
    // three PlaceLeft alternates with the +1 anchor check_start (:2268-2285),
    // LinkedAlternates sub-tile at 19/48 carrying all five alternates.
    let d = def_for_tile(11).unwrap();
    assert_eq!((d.width, d.height), (2, 3));
    assert_eq!(d.direction, TileObjectDirection::PlaceRight);
    assert_eq!((d.style_wrap_limit, d.style_line_skip), (36, 2));
    assert_eq!(d.alternates.len(), 5);
    // addAlternate(0) rows keep style 0; addAlternate(1) rows carry style 1.
    assert_eq!(d.alternates[2].origin, tr_core::tileobject::Point16::new(1, 0));
    assert_eq!(d.alternates[2].direction, TileObjectDirection::PlaceLeft);
    assert_eq!(d.alternates[2].style, 1);
    assert_eq!(d.alternates[2].anchor_top, AnchorData::new(1, 1, 1));
    assert_eq!(d.alternates[4].origin, tr_core::tileobject::Point16::new(1, 2));
    assert_eq!(d.subtiles.len(), 2);
    assert_eq!(d.subtiles[0].1.alternates.len(), 5, "LinkedAlternates (:2287)");
    assert_eq!(d.subtiles[0].1.lava_placement, LiquidPlacement::Allowed);
}

#[test]
fn w4_d111_herb_3_natural_object_rules() {
    // TileObjectData.cs:4281-4300 — CopyFrom(Style1x1) (:4240-4250) →
    // Style1x1Plant_Height22 (:4281-4287) → ApplyNaturalObjectRules (:1847-1851:
    // UsesCustomCanPlace=false, UsesGlobalLiquidChecks=true) + the
    // PlantCheck_CanPlaceHook callback name (:4299). AnchorBottom keeps
    // Style1x1's SolidTile|SolidWithTop|SolidSide (=11) and the shared
    // anchorInvalidTiles list (:2112-2116).
    let d = def_for_tile(3).unwrap();
    assert_eq!(d.anchor_bottom, AnchorData::new(11, 1, 0));
    assert_eq!(
        d.anchor_invalid_tiles,
        &[127u16, 138, 664, 665, 484, 711, 712, 713, 714, 715, 716]
    );
    assert!(d.style_horizontal);
    assert_eq!(d.water_placement, LiquidPlacement::NotAllowed);
    assert!(d.lava_death);
    assert!(!d.uses_custom_can_place);
    assert!(d.uses_global_liquid_checks);
    assert_eq!(d.hook_check_if_can_place.callback, "WorldGen.PlantCheck_CanPlaceHook");
    assert!(d.hook_check_if_can_place.hook.is_none(), "documented callback seam");
}

#[test]
fn w4_d111_stalactite_165_style_override_and_subtiles() {
    // TileObjectData.cs:4986-5012 — Style1x2 copy, AnchorTop
    // SolidTile|SolidSide|AlternateTile|SolidBottom (=169), StyleWrapLimit=num
    // (num=39, :4227), GetStyle_Stalactite (:4993), three sub-tiles:
    // 0 = the tip (origin (0,1), bottom 11), 1 = mid (top 169), 2 = base
    // (bottom 11), each 1-wide with StyleHorizontal (:5002/:5007).
    let d = def_for_tile(165).unwrap();
    assert_eq!((d.width, d.height), (1, 2));
    assert_eq!(d.anchor_top, AnchorData::new(169, 1, 0));
    assert_eq!(d.anchor_bottom, AnchorData::EMPTY);
    assert_eq!(d.style_wrap_limit, 39);
    assert_eq!(d.get_style_override, GetStyleOverride::Stalactite);
    assert!(d.uses_global_liquid_checks);
    assert_eq!(d.subtiles.len(), 3);
    let (s0, s1, s2) = (d.subtiles[0].1, d.subtiles[1].1, d.subtiles[2].1);
    assert_eq!(d.subtiles[0].0, 0);
    assert_eq!(s0.origin, tr_core::tileobject::Point16::new(0, 1));
    assert_eq!(s0.anchor_bottom, AnchorData::new(11, 1, 0));
    assert_eq!(s0.get_style_override, GetStyleOverride::Stalactite, "CopyFrom (:4994)");
    assert_eq!(s1.anchor_top, AnchorData::new(169, 1, 0));
    assert_eq!(s1.anchor_bottom, AnchorData::EMPTY);
    assert_eq!(s2.anchor_top, AnchorData::EMPTY);
    assert_eq!(s2.anchor_bottom, AnchorData::new(11, 1, 0));
    assert!(s1.style_horizontal && s2.style_horizontal);
}

#[test]
fn w4_d111_xmas_tree_171_hook_data() {
    // TileObjectData.cs:2161-2171 — 4x8, origin (1,7), AnchorBottom
    // SolidTile|SolidWithTop (=3) count 4, PlaceXmasTree_Direct override with
    // bad_return -1 / processedCoordinates true (:2166), padding 0 (:2169),
    // eight 16px rows; LavaDeath never set → base false (SetupBaseObject
    // :1764, the per-addTile fresh cursor :2012).
    let d = def_for_tile(171).unwrap();
    assert_eq!((d.width, d.height), (4, 8));
    assert_eq!(d.origin, tr_core::tileobject::Point16::new(1, 7));
    assert_eq!(d.anchor_bottom, AnchorData::new(3, 4, 0));
    assert!(d.uses_custom_can_place);
    let h = d.hook_place_override;
    assert_eq!(h.callback, "WorldGen.PlaceXmasTree_Direct");
    assert_eq!(h.bad_return, -1);
    assert!(h.processed_coordinates);
    assert_eq!(d.coordinate_heights, &[16; 8]);
    assert_eq!(d.coordinate_padding, 0);
    assert!(!d.lava_death);
}

#[test]
fn w4_d111_small_piles_185_subtile_wrap() {
    // TileObjectData.cs:4975-4985 — Style1x1 copy + StyleHorizontal +
    // ApplyNaturalObjectRules + GetStyle_SmallPiles (:4979); sub-tile 0 is a
    // Style2x1 copy with StyleWrapLimit 53 (:4982), bottom anchor 11 count 2.
    let d = def_for_tile(185).unwrap();
    assert_eq!((d.width, d.height), (1, 1));
    assert!(d.style_horizontal);
    assert!(d.uses_global_liquid_checks);
    assert!(!d.uses_custom_can_place);
    assert_eq!(d.get_style_override, GetStyleOverride::SmallPiles);
    assert_eq!(d.subtiles.len(), 1);
    let sub = d.subtiles[0].1;
    assert_eq!(d.subtiles[0].0, 0);
    assert_eq!((sub.width, sub.height), (2, 1));
    assert_eq!(sub.style_wrap_limit, 53);
    assert_eq!(sub.anchor_bottom, AnchorData::new(11, 2, 0));
    assert!(sub.uses_global_liquid_checks);
}