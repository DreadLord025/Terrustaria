//! W4-d112 witnesses for the `GetTileData(Tile)` frame-resolution consumer
//! (`tileobject::resolve_tile_def`, `TileObjectData.cs:5196-5261`) and the
//! three `GetStyleOverride` bodies (`GetStyle_Detritus/SmallPiles/Stalactite`
//! :5294-5327) — S1-F-7 wave B's remaining half, ported after the W4-d111
//! census proved the registry DATA already complete (389 tiles / 604 defs,
//! `gen.py --check` green). No registry data is touched here.
//!
//! Frame witnesses are derived from `TileObject.Place`'s frame write
//! (`TileObject.cs:60-76`): `CalculatePlacementStyle = style * StyleMultiplier
//! + Style (+ random)`, wrap (`num5 = style / wrap * line_skip`,
//! `style %= wrap`), then horizontal `frameX = CFW * style`,
//! `frameY = CFH * num5` (vertical swaps the axes). Per-cell offsets ride
//! `CoordinateWidth + CoordinatePadding` / `CoordinateHeights[l] + Padding`
//! (:96-108). The decode under test is the exact inverse (:5212-5240).
//!
//! WRITTEN NOT RUN (task constraint: no cargo in this lane); expected values
//! hand-derived from the cited C# lines + the generated registry defs.

use tr_core::tile::{Tile, FLAG_ACTIVE, FLAG_INACTIVE};
use tr_core::tileobject::{def_for_tile, get_tile_data, resolve_tile_def, GetStyleOverride};

/// An active tile of `kind` carrying the given stored frames.
fn framed(kind: u16, frame_x: i16, frame_y: i16) -> Tile {
    Tile {
        kind,
        frame_x,
        frame_y,
        flags: FLAG_ACTIVE,
        ..Default::default()
    }
}

#[test]
fn w4_d112_inactive_or_unregistered_resolves_nothing() {
    // :5198-5201 — the active() gate precedes everything.
    let mut t = framed(15, 0, 0);
    t.flags &= !FLAG_ACTIVE;
    assert!(resolve_tile_def(&t).is_none());
    // `Tile.active()` reads FLAG_ACTIVE only (:5198) — an actuated
    // (FLAG_INACTIVE) but active tile still resolves.
    let mut t = framed(15, 0, 0);
    t.flags |= FLAG_INACTIVE;
    assert!(resolve_tile_def(&t).is_some());
    // :5207-5211 — `_data[type] == null` for unregistered kinds (dirt 1).
    assert!(resolve_tile_def(&framed(1, 0, 0)).is_none());
}

#[test]
fn w4_d112_detritus_override_framey_36() {
    // GetStyle_Detritus (:5294-5301) on kind 233 (addTile :4974, override
    // assignment :4968, addSubTile(0) :4973 — the 2x2 Style2x2 sub-tile).
    let base = def_for_tile(233).unwrap();
    assert_eq!(base.get_style_override, GetStyleOverride::Detritus);
    let sub = get_tile_data(base, 0, 0).unwrap();
    assert_eq!(
        (sub.width, sub.height),
        (2, 2),
        "D_233_S0 = Style2x2 (:4969)"
    );
    // frameY < 36 -> the base def (3x2), regardless of frameX (:5222-5226
    // short-circuits the whole decode).
    for fy in [0, 18, 35] {
        let r = resolve_tile_def(&framed(233, 54, fy)).unwrap();
        assert_eq!((r.width, r.height), (3, 2), "frameY {fy}");
    }
    // frameY >= 36 -> SubTiles[0].
    for fy in [36, 54, 100] {
        let r = resolve_tile_def(&framed(233, 0, fy)).unwrap();
        assert_eq!((r.width, r.height), (2, 2), "frameY {fy}");
    }
}

#[test]
fn w4_d112_small_piles_override_framey_18() {
    // GetStyle_SmallPiles (:5303-5310) on kind 185 (addTile :4985, override
    // :4979, addSubTile(0) :4984 — the 2x1 Style2x1 sub-tile, wrap 53 :4982).
    let base = def_for_tile(185).unwrap();
    assert_eq!(base.get_style_override, GetStyleOverride::SmallPiles);
    let sub = get_tile_data(base, 0, 0).unwrap();
    assert_eq!(
        (sub.width, sub.height),
        (2, 1),
        "D_185_S0 = Style2x1 (:4980)"
    );
    assert_eq!(sub.style_wrap_limit, 53, ":4982");
    // frameY < 18 -> base 1x1; >= 18 -> SubTiles[0] 2x1.
    assert_eq!(resolve_tile_def(&framed(185, 0, 17)).unwrap().width, 1);
    for fy in [18, 35, 36] {
        assert_eq!(resolve_tile_def(&framed(185, 18, fy)).unwrap().width, 2);
    }
}

#[test]
fn w4_d112_stalactite_override_thresholds_36_72_90() {
    // GetStyle_Stalactite (:5312-5327) on kind 165 (addTile :5012, override
    // :4993, addSubTile(0/1/2) :4999/:5005/:5011). SubTiles[0] keeps height 2
    // (CopyFrom(newTile) :4994, origin (0,1)); S1/S2 are 1x1 (:5000/:5006).
    let base = def_for_tile(165).unwrap();
    assert_eq!(base.get_style_override, GetStyleOverride::Stalactite);
    assert_eq!(base.height, 2);
    let s0 = get_tile_data(base, 0, 0).unwrap();
    let s1 = get_tile_data(base, 1, 0).unwrap();
    let s2 = get_tile_data(base, 2, 0).unwrap();
    assert_eq!(s0.height, 2, "S0 = 1x2 (:4994)");
    assert_eq!((s1.width, s1.height), (1, 1));
    assert_eq!((s2.width, s2.height), (1, 1));
    // S1 hangs (anchor_top), S2 stands (anchor_bottom) — distinguishes the
    // two 1x1 sub-tiles (:4997-4998 vs :5003-5004 / :5009-5010).
    assert!(s1.anchor_bottom.tile_count == 0 && s1.anchor_top.tile_count != 0);
    assert!(s2.anchor_top.tile_count == 0 && s2.anchor_bottom.tile_count != 0);
    // The descending threshold ladder, boundaries included (:5314-5325).
    let at = |fy: i16| resolve_tile_def(&framed(165, 0, fy)).unwrap();
    for fy in [0, 35] {
        let r = at(fy);
        assert_eq!(r.height, 2, "base below 36 (frameY {fy})");
        assert_eq!(r.origin.y, 0, "the BASE origin (0,0), not S0's (0,1)");
    }
    for fy in [36, 71] {
        let r = at(fy);
        assert_eq!(r.height, 2, "SubTiles[0] at {fy}");
        assert_eq!(r.origin.y, 1, "S0 origin (0,1) :4996");
    }
    for fy in [72, 89] {
        let r = at(fy);
        assert_eq!(r.height, 1, "SubTiles[1] at {fy}");
        assert!(r.anchor_top.tile_count != 0, "S1 hangs at {fy}");
    }
    for fy in [90, 120] {
        let r = at(fy);
        assert_eq!(r.height, 1, "SubTiles[2] at {fy}");
        assert!(r.anchor_bottom.tile_count != 0, "S2 stands at {fy}");
    }
}

#[test]
fn w4_d112_chair_multiplier_and_alternate_window() {
    // Kind 15: StyleMultiplier 2 (:4145), horizontal, wrap 2 (:4144),
    // base alternate D_15_A1 style 1 range 0 (addAlternate(1) :4150,
    // PlaceRight). Place(style s, alt a): num4 = 2s + a.Style
    // (CalculatePlacementStyle :1990-1998), then wrap: frameX =
    // 18 * (num4 % 2), frameY = 40 * (num4 / 2) (TileObject.cs:62-76), so the
    // right-facing alternate (Style 1) lands at frameX = 18 for every s.
    let base = def_for_tile(15).unwrap();
    assert_eq!(base.style_multiplier, 2);
    // Left-facing style 3 -> num4 = 6 -> num5 = 3, num6 = 0: no sub-tile 3,
    // no alternate window at 0 -> the base def (PlaceLeft, frameX 0).
    let r = resolve_tile_def(&framed(15, 0, 120)).unwrap();
    assert_eq!(r.direction, base.direction);
    // Its right-facing alternate's bottom cell: num4 = 2*3 + 1 = 7, so
    // frameX = 18*(7%2) = 18 and frameY = 40*3 + 18 = 138 -> num2 = 3,
    // num4 = 3*2 + 1 = 7, num6 = 1 -> D_15_A1 (style 1 <= 1 <= 1+0).
    let r = resolve_tile_def(&framed(15, 18, 138)).unwrap();
    assert_eq!(r.style, 1);
    assert_ne!(r.direction, base.direction, "alternate flips to PlaceRight");
    // Sub-tile 16 (toilet, addSubTile(16, 47...) :4155): place style 16 ->
    // num4 = 32 -> frameX = 18*(32%2) = 0, frameY = 40 * 16 = 640 -> decode
    // num4 = 32, num5 = 16, num6 = 0.
    let r = resolve_tile_def(&framed(15, 0, 640)).unwrap();
    assert_eq!(r.lava_death, false, "D_15_S16 LavaDeath=false");
    assert_eq!(r.direction, base.direction, "S16 keeps PlaceLeft");
    // Its right-facing alternate's bottom cell (:4165-4170, Style 1):
    // num4 = 16*2 + 1 = 33 -> frameX = 18, frameY = 640 + 18 = 658 ->
    // num4 = 33, num6 = 1 -> S16's OWN alternate (:5249 reads the RESOLVED
    // sub-tile's alternates).
    let r = resolve_tile_def(&framed(15, 18, 658)).unwrap();
    assert_eq!(r.style, 1);
    assert_ne!(r.direction, base.direction);
}

#[test]
fn w4_d112_door_line_skip_redecode_and_subtile_alternates() {
    // Kind 10: vertical, wrap 36, line skip 3 (:2234-2235), mult 1.
    // Place(style 0, alt 0): frameX = 0, frameY = 0; rows 18 apart
    // (CoordinateHeights 16+2, :2230-2232). CFH = 54, so all three rows of
    // one door share num2 = frameY/54. num6 = num % 3 (the skip re-decode,
    // :5238) — for the 1-wide door at frameX 0 it is 0 on every row.
    let base = def_for_tile(10).unwrap();
    assert_eq!(base.style_line_skip, 3);
    // Both base alternates carry Style 0 (addAlternate(0) twice, :2238/
    // :2241) — the [0,0] window of the FIRST (A1, origin (0,1)) wins for
    // num6 = 0; A2 is only reachable through GetTileData(type, style, 2).
    for fy in [0, 18, 36] {
        let r = resolve_tile_def(&framed(10, 0, fy)).unwrap();
        assert_eq!(r.origin.y, 1, "A1 first-match at frameY {fy}");
        assert!(r.lava_death, "door alternates keep LavaDeath (:2229)");
    }
    // frameX 18 (num = 1): num6 = 1 % 3 = 1 -> no [0,0] window -> the base
    // def itself (:5260), origin (0,0).
    let r = resolve_tile_def(&framed(10, 18, 0)).unwrap();
    assert_eq!(r.origin.y, 0);
    assert!(r.lava_death);
    // Style 19 (Lihzahrd door, addSubTile(19, 48) :2246): place style 19 ->
    // num5 = 19/36*3 = 0, num4 = 19 -> frameX 0, frameY 54*19 = 1026.
    // Decode: num = 0, num2 = 19 -> skip path num5 = 0/3*36 + 19 = 19,
    // num6 = 0 -> SubTiles[19], then ITS alternates (both Style 0, linked
    // :2243) -> S19_A1 first-match (origin (0,1)).
    for fy in [1026, 1044, 1062] {
        let r = resolve_tile_def(&framed(10, 0, fy)).unwrap();
        assert_eq!(r.origin.y, 1, "S19_A1 at frameY {fy}");
        assert!(!r.lava_death, "S19 LavaDeath=false (:2244)");
    }
    // frameX 18 (num = 1): num5 = 1/3*36 + 19 = 19 still, num6 = 1 -> no
    // window -> S19 itself (origin (0,0), lava-safe).
    let r = resolve_tile_def(&framed(10, 18, 1026)).unwrap();
    assert_eq!(r.origin.y, 0);
    assert!(!r.lava_death);
    // frameX 54 (num = 3): num5 = 3/3*36 + 19 = 55 -> no sub-tile 55 -> the
    // BASE def (lava dies) — the skip wrap rolls into the next style block.
    let r = resolve_tile_def(&framed(10, 54, 1026)).unwrap();
    assert!(r.lava_death);
    assert_eq!(r.origin.y, 1, "base scan still first-matches A1 at num6 0");
}

#[test]
fn w4_d112_lamp_line_skip_with_zero_wrap() {
    // Kind 93: vertical, wrap 0 (-> 1 in the DECODE, :5215-5218; Place does
    // not wrap when StyleWrapLimit == 0, TileObject.cs:62), line skip 2
    // (:2405), sub-tiles 23/42 (lava-safe :2411-2413) and 40 (water-safe
    // :2415-2417). CFH = 54. Place style s -> frameX 0, frameY 54*s.
    // Decode: num = 0, num2 = s; skip path num5 = 0/2 + s = s, num6 = 0.
    let base = def_for_tile(93).unwrap();
    assert_eq!(base.style_wrap_limit, 0);
    assert_eq!(base.style_line_skip, 2);
    // Style 2 (no sub-tile 2): base def.
    let r = resolve_tile_def(&framed(93, 0, 2 * 54)).unwrap();
    assert!(r.lava_death);
    // Style 23 (frameY 1242): num5 = 23, num6 = 0 -> SubTiles[23].
    let r = resolve_tile_def(&framed(93, 0, 23 * 54)).unwrap();
    assert!(!r.lava_death, "S23 LavaDeath=false (:2411)");
    assert_eq!(
        r.lava_placement,
        tr_core::tileobject::LiquidPlacement::Allowed,
        "S23 LavaPlacement=Allowed (:2412)"
    );
    // The next sheet row down (frameY + 54 -> num2 = 24): num5 = 24 -> no
    // sub-tile -> base again (the line-skip pairs rows).
    let r = resolve_tile_def(&framed(93, 0, 24 * 54)).unwrap();
    assert!(r.lava_death);
    // Style 40 (frameY 2160): num5 = 40 -> SubTiles[40] (water allowed,
    // lava still forbidden).
    let r = resolve_tile_def(&framed(93, 0, 40 * 54)).unwrap();
    assert!(r.lava_death, "S40 keeps LavaDeath (:2414-2417)");
    assert_eq!(
        r.water_placement,
        tr_core::tileobject::LiquidPlacement::Allowed,
        "S40 WaterPlacement=Allowed (:2416)"
    );
    // The horizontal axis feeds num through the skip: frameX 36 (num = 2),
    // frameY 54 (num2 = 1) -> num5 = 2/2 + 1 = 2 -> base.
    let r = resolve_tile_def(&framed(93, 36, 54)).unwrap();
    assert!(r.lava_death);
}

#[test]
fn w4_d112_sunflower_alternate_range_windows() {
    // Kind 20: horizontal, wrap 0 (-> 1), mult 1 (:4878 sets only
    // RandomStyleRange; StyleMultiplier keeps the base 1, :1802), CFH =
    // 16+2 + 18+2 = 38. Ten alternates, styles 3/6/.../30, each
    // RandomStyleRange 3 (:4879-4908, addAlternate(3k)).
    // :5220-5221 splits the frame index into style = num4 / StyleMultiplier
    // and remainder = num4 % StyleMultiplier. With StyleMultiplier 1 the
    // remainder is ALWAYS 0, so the [Style, Style + RandomStyleRange] bands —
    // every one of which starts at 3 — can never match a frame-decoded tile:
    // all offsets answer the base def (:5260). The alternates are
    // placement-addressed (GetTileData(type, style, alternate) :5126-5160),
    // pinned at the end through the index-based overload.
    let base = def_for_tile(20).unwrap();
    assert_eq!(base.alternates.len(), 10);
    assert_eq!(base.random_style_range, 3);
    assert_eq!(base.style_multiplier, 1);
    for (i, alt) in base.alternates.iter().enumerate() {
        assert_eq!(alt.style, 3 * (i as i32 + 1), "A{} style", i + 1);
        assert_eq!(alt.random_style_range, 3, "A{} range", i + 1);
    }
    for offset in 0..=34 {
        let r = resolve_tile_def(&framed(20, 0, offset * 38)).unwrap();
        assert_eq!(
            (r.kind, r.style, r.alternates.len()),
            (20, 0, 10),
            "offset {offset}: the base def, remainder 0"
        );
    }
    // The windows exist but only the placement-time lookup reaches them.
    let a1 = get_tile_data(base, 3, 1).unwrap();
    assert_eq!((a1.style, a1.random_style_range), (3, 3));
    let a10 = get_tile_data(base, 30, 10).unwrap();
    assert_eq!((a10.style, a10.random_style_range), (30, 3));
}

#[test]
fn w4_d112_torch_subtile_then_its_alternates() {
    // Kind 4: horizontal, wrap 6, mult 6 (:4585-4586 pattern), 22px grid
    // (CoordinateWidth 20 + Padding 2). Place style 8 -> num4 = 48 ->
    // wrap num5 = 48/6 = 8, style 0 -> frameX 0, frameY 22*8 = 176.
    // Decode: num = 0, num2 = 8 -> num4 = 8*6 + 0 = 48 -> num5 = 8,
    // num6 = 0 -> SubTiles[8] (D_4_S8), whose alternates are styles 1/2/0
    // (LinkedAlternates :4608) — num6 0 first-matches S8_A3 (style 0, the
    // wall torch, anchor_wall true :4605-4606).
    let base = def_for_tile(4).unwrap();
    assert_eq!(base.style_multiplier, 6);
    let r = resolve_tile_def(&framed(4, 0, 176)).unwrap();
    assert!(!r.lava_death, "S8 torch family is lava-safe (:4610)");
    assert!(r.anchor_wall, "S8_A3 is the wall variant");
    assert_eq!(r.style, 0);
    // num6 = 1 (frameX 22): S8_A1 style 1 (left-anchored).
    let r = resolve_tile_def(&framed(4, 22, 176)).unwrap();
    assert_eq!(r.style, 1);
    assert!(r.anchor_left.tile_count != 0);
    // num6 = 2 (frameX 44): S8_A2 style 2 (right-anchored).
    let r = resolve_tile_def(&framed(4, 44, 176)).unwrap();
    assert_eq!(r.style, 2);
    assert!(r.anchor_right.tile_count != 0);
    // num6 = 3 (frameX 66): no window (styles 1/2/0, ranges 0) -> S8 itself.
    let r = resolve_tile_def(&framed(4, 66, 176)).unwrap();
    assert_eq!(r.alternates.len(), 3, "the sub-tile, not an alternate");
    assert!(!r.anchor_wall);
    // Base-level walk without a sub-tile: style 0 frames (0,0): num6 = 0 ->
    // windows [1,1],[2,2],[0,0]; offset 0 matches A3 (style 0, the WALL
    // variant, addAlternate(0) :4606) LAST in list order — first-match scan
    // reaches it.
    let r = resolve_tile_def(&framed(4, 0, 0)).unwrap();
    assert_eq!(r.style, 0);
    assert!(r.lava_death, "D_4_A3 keeps the base lava death (:4591)");
    assert!(r.anchor_wall, "A3 is the wall torch (:4605)");
    // offset 1 -> A1 (style 1, anchor_left present :4597).
    let r = resolve_tile_def(&framed(4, 22, 0)).unwrap();
    assert_eq!(r.style, 1);
    assert!(r.anchor_left.tile_count != 0);
}

#[test]
fn w4_d112_override_tag_census_matches_initialize() {
    // Exactly three kinds carry a GetStyleOverride in Initialize
    // (:4968 Detritus -> 233, :4979 SmallPiles -> 185, :4993 Stalactite ->
    // 165); every other generated def is None.
    let mut tagged: Vec<(u16, GetStyleOverride)> = Vec::new();
    for def in tr_core::tileobject::TABLE {
        if def.get_style_override != GetStyleOverride::None {
            tagged.push((def.kind, def.get_style_override));
        }
    }
    tagged.sort_by_key(|(k, _)| *k);
    assert_eq!(
        tagged,
        vec![
            (165, GetStyleOverride::Stalactite),
            (185, GetStyleOverride::SmallPiles),
            (233, GetStyleOverride::Detritus),
        ]
    );
}
