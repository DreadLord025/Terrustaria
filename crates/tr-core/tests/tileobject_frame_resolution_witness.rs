//! W4-e11 witness pins for the tile-based `TileObjectData.GetTileData(Tile)`
//! frame-resolution consumer (`TileObjectData.cs:5196-5261`) — the d111
//! research/08 "open half" (the registry itself is generated-complete, S1-F-7
//! wave A; this batch ports the RESOLUTION logic, not defs).
//!
//! Every expectation below is hand-derived from the cited `Initialize` chain
//! + the resolver arithmetic, cross-checked against `registry.rs`. WRITTEN
//! NOT RUN (code-only mode).

use tr_core::tile::Tile;
use tr_core::tileobject::{
    check_lava_death, check_water_death, def_for_tile, get_tile_data_for_tile, try_get_tile_bounds,
    GetStyleOverride, TileBounds,
};
use tr_core::world::World;

/// An active tile of `kind` with the given frames — the resolver reads only
/// `active`/`kind`/`frame_x`/`frame_y` (`TileObjectData.cs:5198-5213`).
fn framed(kind: u16, fx: i16, fy: i16) -> Tile {
    Tile {
        kind,
        frame_x: fx,
        frame_y: fy,
        flags: tr_core::tile::FLAG_ACTIVE,
        ..Default::default()
    }
}

/// Resolve and identify the def by (width, height, origin) — the registry's
/// statics are private, so the witness pins the def's SHAPE (each expected
/// target below has a shape distinct from its siblings).
fn shape(t: &Tile) -> Option<(i32, i32, i32, i32)> {
    get_tile_data_for_tile(t).map(|d| (d.width, d.height, d.origin.x, d.origin.y))
}

#[test]
fn stalactite_165_framey_bands_pick_subtiles_2_1_0() {
    // GetStyle_Stalactite (:5312-5327) on kind 165 (`Initialize` stalactite
    // chain): frameY >= 90 -> SubTiles[2] (D_165_S2, 1x1), >= 72 -> [1]
    // (D_165_S1, 1x1 top-anchored), >= 36 -> [0] (D_165_S0, 1x2 origin
    // (0,1)), else the base (1x2 origin (0,0)).
    let base = def_for_tile(165).unwrap();
    assert_eq!(base.get_style_override, GetStyleOverride::Stalactite);
    assert_eq!(base.subtiles.len(), 3);
    for (fy, want) in [
        (0i16, (1, 2, 0, 0)),
        (35, (1, 2, 0, 0)),
        (36, (1, 2, 0, 1)), // S0
        (71, (1, 2, 0, 1)),
        (72, (1, 1, 0, 0)), // S1
        (89, (1, 1, 0, 0)),
        (90, (1, 1, 0, 0)), // S2 — same shape as S1; identity pinned below
        (126, (1, 1, 0, 0)),
    ] {
        assert_eq!(shape(&framed(165, 0, fy)), Some(want), "fy {fy}");
    }
    // S1 vs S2 identity through the anchor split (S1 anchors top 169, S2
    // anchors bottom 11 — registry D_165_S1/D_165_S2).
    let s1 = get_tile_data_for_tile(&framed(165, 0, 72)).unwrap();
    let s2 = get_tile_data_for_tile(&framed(165, 0, 90)).unwrap();
    assert_eq!(s1.anchor_top.ty, 169);
    assert_eq!(s2.anchor_top.ty, 0);
    assert_eq!(s2.anchor_bottom.ty, 11);
    // frameX is irrelevant to the override (:5222-5226 short-circuit).
    assert_eq!(shape(&framed(165, 702, 90)), Some((1, 1, 0, 0)));
}

#[test]
fn small_piles_185_framey_18_flips_to_the_2x1_sub() {
    // GetStyle_SmallPiles (:5303-5310) on kind 185: frameY >= 18 ->
    // SubTiles[0] (D_185_S0, 2x1), else the base (1x1).
    let base = def_for_tile(185).unwrap();
    assert_eq!(base.get_style_override, GetStyleOverride::SmallPiles);
    assert_eq!(shape(&framed(185, 0, 0)), Some((1, 1, 0, 0)));
    assert_eq!(shape(&framed(185, 0, 17)), Some((1, 1, 0, 0)));
    assert_eq!(shape(&framed(185, 0, 18)), Some((2, 1, 0, 0)));
    assert_eq!(shape(&framed(185, 90, 36)), Some((2, 1, 0, 0)));
}

#[test]
fn detritus_233_framey_36_flips_to_the_2x2_sub() {
    // GetStyle_Detritus (:5294-5301) on kind 233: frameY >= 36 ->
    // SubTiles[0] (D_233_S0, 2x2 origin (0,1)), else the base (3x2 origin
    // (1,1)).
    let base = def_for_tile(233).unwrap();
    assert_eq!(base.get_style_override, GetStyleOverride::Detritus);
    assert_eq!(shape(&framed(233, 0, 0)), Some((3, 2, 1, 1)));
    assert_eq!(shape(&framed(233, 36, 35)), Some((3, 2, 1, 1)));
    assert_eq!(shape(&framed(233, 0, 36)), Some((2, 2, 0, 1)));
    assert_eq!(shape(&framed(233, 0, 54)), Some((2, 2, 0, 1)));
}

#[test]
fn torch_4_multiplier_6_walks_subtiles_then_alternates() {
    // Kind 4 (torch): horizontal, wrap 6, multiplier 6, 20-px coordinate
    // cells (:5212-5221: full_w = 22, full_h = 22). num5 = num4/6 picks the
    // sub-tile {8, 11, 17}, num6 = num4%6 the alternate band. Base
    // alternates A1(style 1)/A2(style 2)/A3(style 0); sub8 carries its OWN
    // alternates (S1-F-7a: GetTileData indexes the resolved sub-tile).
    let base = def_for_tile(4).unwrap();
    assert_eq!(base.style_multiplier, 6);
    assert_eq!(base.coordinate_full_width(), 22);
    // fx 0 -> num4 0 -> style 0 rem 0 -> base A3 (wall-anchored).
    let d = get_tile_data_for_tile(&framed(4, 0, 0)).unwrap();
    assert!(d.anchor_wall);
    // fx 22 -> rem 1 -> A1 (left-anchored 57).
    let d = get_tile_data_for_tile(&framed(4, 22, 0)).unwrap();
    assert_eq!(d.anchor_left.ty, 57);
    // fx 44 -> rem 2 -> A2 (right-anchored 57).
    let d = get_tile_data_for_tile(&framed(4, 44, 0)).unwrap();
    assert_eq!(d.anchor_right.ty, 57);
    // fx 66 -> rem 3 -> no band -> the base (bottom-anchored 9).
    let d = get_tile_data_for_tile(&framed(4, 66, 0)).unwrap();
    assert_eq!(d.anchor_bottom.ty, 9);
    assert!(!d.anchor_wall);
    // fx 1056 -> num 48 -> num4 48 -> style 8 rem 0 -> SubTiles[8]'s A3
    // (floor torch, wall-anchored, 20-px cells).
    let d = get_tile_data_for_tile(&framed(4, 1056, 0)).unwrap();
    assert!(d.anchor_wall);
    assert_eq!(d.coordinate_width, 20);
    // fx 1078 -> num4 49 -> style 8 rem 1 -> the SUB's own A1 (left 57) —
    // not the base's alternate list.
    let d = get_tile_data_for_tile(&framed(4, 1078, 0)).unwrap();
    assert_eq!(d.anchor_left.ty, 57);
    assert_eq!(d.anchor_bottom.ty, 0);
    // fx 198 -> num4 9 -> style 1 rem 3: sub 1 absent, base bands stop at
    // 2 -> the base def itself.
    let d = get_tile_data_for_tile(&framed(4, 198, 0)).unwrap();
    assert_eq!(d.anchor_bottom.ty, 9);
    assert_eq!(d.kind, 4);
}

#[test]
fn lightbulb_149_multiplier_1_collapses_the_remainder_to_zero() {
    // Kind 149: wrap 6, multiplier 1, alternates A1(style 12)/A2(style 6)/
    // A3(style 18), all range 0. num6 = num4 % 1 = 0 ALWAYS (:5221), and no
    // band contains 0 — the tile-based overload never returns these
    // alternates (they are addressed by GetTileData(type, style, alternate)
    // at placement). Every frame answers the base (bottom-anchored 1).
    let base = def_for_tile(149).unwrap();
    assert_eq!(base.style_wrap_limit, 6);
    assert_eq!(base.style_multiplier, 1);
    for (fx, fy) in [(0i16, 0i16), (108, 0), (216, 0), (324, 0), (0, 18)] {
        let d = get_tile_data_for_tile(&framed(149, fx, fy)).unwrap();
        assert_eq!(d.kind, 149, "fx {fx} fy {fy}");
        assert_eq!(d.anchor_bottom.ty, 1);
    }
}

#[test]
fn closed_door_10_vertical_line_skip_3_decodes_the_frame_pair() {
    // Kind 10: VERTICAL, wrap 36, multiplier 1, line_skip 3 (:5235-5238:
    // style = num/3*36 + num2, remainder = num%3). full_h = 3*18 = 54.
    let base = def_for_tile(10).unwrap();
    assert!(!base.style_horizontal);
    assert_eq!(base.style_line_skip, 3);
    // fx 0, fy 18 (middle cell): num 0, num2 0 (full_h 54) -> style 0
    // rem 0 -> A1 (origin (0,1), the first band [0,0] in list order).
    let d = get_tile_data_for_tile(&framed(10, 0, 18)).unwrap();
    assert_eq!((d.origin.x, d.origin.y), (0, 1));
    // fx 18, fy 36: num 1 rem 1 — A2's Style is ALSO 0 (range 0), so no
    // band contains 1 -> the base (origin (0,0)).
    let d = get_tile_data_for_tile(&framed(10, 18, 36)).unwrap();
    assert_eq!((d.origin.x, d.origin.y), (0, 0));
    assert_eq!(d.kind, 10);
    // fx 36, fy 0: num 2 rem 2 -> the base again.
    let d = get_tile_data_for_tile(&framed(10, 36, 0)).unwrap();
    assert_eq!((d.origin.x, d.origin.y), (0, 0));
    assert_eq!(d.kind, 10);
    // fx 342 (style 19 door), fy 18: num 19 -> style = 19/3*36 + 0 = 216,
    // rem 19%3 = 1. Sub 216 absent, no band -> the base. (The tall-door
    // sub-tile styles 19/48 are addressed by GetTileData(type, style), not
    // by frames — the frame grid wraps at 3 styles per column.)
    let d = get_tile_data_for_tile(&framed(10, 342, 18)).unwrap();
    assert_eq!(d.kind, 10);
}

#[test]
fn open_door_11_line_skip_2_remainder_selects_the_leaf_alternates() {
    // Kind 11 (open door): VERTICAL, wrap 36, multiplier 1, line_skip 2,
    // full_w = 2*18 = 36. :5237-5238: style = num/2*36 + num2, remainder =
    // num%2. Alternates in ADD order (:2262-2285): A1/A2 (Style 0, origins
    // (0,1)/(0,2)) then A3/A4/A5 (Style 1, origins (1,0)/(1,1)/(1,2)) — the
    // remainder picks the first alternate whose Style band contains it
    // (:5251-5257), so remainder 1 -> A3 (origin (1,0), the right leaf's TOP
    // row; the per-row origins (1,1)/(1,2) are placement-addressed, never
    // frame-decoded because the row rides `num2`, not the remainder).
    let base = def_for_tile(11).unwrap();
    assert_eq!(base.style_line_skip, 2);
    // Left leaf top cell (fx 0, fy 0): num 0 rem 0 -> A1 (origin (0,1)).
    let d = get_tile_data_for_tile(&framed(11, 0, 0)).unwrap();
    assert_eq!((d.origin.x, d.origin.y), (0, 1));
    // Right leaf top cell (fx 54, fy 0): num 1 rem 1 -> A3 (origin (1,0)).
    let d = get_tile_data_for_tile(&framed(11, 54, 0)).unwrap();
    assert_eq!((d.origin.x, d.origin.y), (1, 0));
    // fx 36 is still num 1 (full_w 36) -> the right leaf again.
    let d = get_tile_data_for_tile(&framed(11, 36, 0)).unwrap();
    assert_eq!((d.origin.x, d.origin.y), (1, 0));
    // fx 72 -> num 2 rem 0 -> back to the left-leaf band.
    let d = get_tile_data_for_tile(&framed(11, 72, 0)).unwrap();
    assert_eq!((d.origin.x, d.origin.y), (0, 1));
}

#[test]
fn sunflower_20_multiplier_1_collapses_the_remainder_to_zero() {
    // Kind 20 (sunflower): multiplier 1, alternates A1..A10 with
    // Style = 3*i and RandomStyleRange = 3. num6 = num4 % 1 = 0 ALWAYS
    // (:5221), and every band starts at 3 — the tile-based overload never
    // returns these alternates (the RandomStyleRange bands only bite when
    // StyleMultiplier > 1, like the torch's 6). Every frame answers the
    // base; the alternates are placement-time (`TileObject.cs:761-825`).
    let base = def_for_tile(20).unwrap();
    assert_eq!(base.style_multiplier, 1);
    assert_eq!(base.alternates.len(), 10);
    for alt in base.alternates {
        assert_eq!(alt.random_style_range, 3);
    }
    for k in [0i16, 3, 6, 7, 9, 10, 30, 33] {
        let d = get_tile_data_for_tile(&framed(20, k * 18, 0)).unwrap();
        assert_eq!(d.kind, 20, "fx {}", k * 18);
    }
}

#[test]
fn boulder_215_wrap_16_horizontal_subtile_pick() {
    // Kind 215: horizontal, wrap 16, multiplier 1, subs {1,4,9,17,20,25}.
    // full_w = 3*18 = 54. fx 54 -> style 1 -> S1 (water_placement Allowed
    // vs the base's NotAllowed — registry D_215/D_215_S1).
    let base = def_for_tile(215).unwrap();
    assert_eq!(base.style_wrap_limit, 16);
    let d = get_tile_data_for_tile(&framed(215, 0, 0)).unwrap();
    assert_eq!(
        d.water_placement,
        tr_core::tileobject::LiquidPlacement::NotAllowed
    );
    let d = get_tile_data_for_tile(&framed(215, 54, 0)).unwrap();
    assert_eq!(
        d.water_placement,
        tr_core::tileobject::LiquidPlacement::Allowed
    );
    // fy 36 wraps (full_h = 2*18 = 36): style = 1*16 + 0 = 16 -> no sub 16
    // -> the base.
    let d = get_tile_data_for_tile(&framed(215, 0, 36)).unwrap();
    assert_eq!(
        d.water_placement,
        tr_core::tileobject::LiquidPlacement::NotAllowed
    );
    // fy 36, fx 54 -> style 17 -> S17 exists.
    let d = get_tile_data_for_tile(&framed(215, 54, 36)).unwrap();
    assert!(d.kind == 0, "a sub-tile, not the base");
}

#[test]
fn try_get_tile_bounds_walks_the_frame_remainder() {
    // TryGetTileBounds (:5163-5194) on a 2x2 chest (kind 21, coordinate
    // heights [16,18], padding 2 -> full_w 36, full_h 38, CoordinateWidth
    // 16): the bottom-right cell (frameX 18, frameY 20) walks left once
    // (18 >= 16) and up once (20 >= 16) -> origin (x-1, y-1), size 2x2.
    let mut w = World::new("bounds".into(), 20, 20);
    let i = w.index(10, 10);
    w.tiles[i] = framed(21, 18, 20);
    assert_eq!(
        try_get_tile_bounds(&w, 10, 10),
        Some(TileBounds {
            x: 9,
            y: 9,
            width: 2,
            height: 2
        })
    );
    // The top-left cell (0,0) stays put.
    w.tiles[i] = framed(21, 0, 0);
    assert_eq!(
        try_get_tile_bounds(&w, 10, 10),
        Some(TileBounds {
            x: 10,
            y: 10,
            width: 2,
            height: 2
        })
    );
    // An inactive cell / unregistered kind -> None (the C# false).
    let mut t = framed(21, 0, 0);
    t.flags &= !tr_core::tile::FLAG_ACTIVE;
    w.tiles[i] = t;
    assert_eq!(try_get_tile_bounds(&w, 10, 10), None);
    w.tiles[i] = framed(1, 0, 0);
    assert_eq!(try_get_tile_bounds(&w, 10, 10), None);
}

#[test]
fn tile_death_checks_resolve_the_frame_then_fall_back_to_the_globals() {
    // CheckWaterDeath(Tile)/CheckLavaDeath(Tile) (:5083-5119).
    // Torch 4: own flags water_death/lava_death true, global checks OFF —
    // both die regardless of the global tables.
    let t = framed(4, 0, 0);
    assert!(check_water_death(&t));
    assert!(check_lava_death(&t));
    // Torch sub 8 (fx 1056 -> num4 48 -> style 8 rem 0 -> S8_A3):
    // water_death false, lava_death false — the RESOLVED sub-tile alternate's
    // flags answer, not the base's.
    let t = framed(4, 1056, 0);
    assert!(!check_water_death(&t));
    assert!(!check_lava_death(&t));
    // Stalactite 165: UsesGlobalLiquidChecks -> the Main tables. 165 is in
    // neither tileWaterDeath nor tileLavaDeath (Main.cs assignments).
    let t = framed(165, 0, 90);
    assert!(!check_water_death(&t));
    assert!(!check_lava_death(&t));
    // Unregistered kind 51 (a global-table member, Main.cs:8063/:8101):
    // no def -> the globals answer.
    let t = framed(51, 0, 0);
    assert!(check_water_death(&t));
    assert!(check_lava_death(&t));
    // Inactive -> false (:5085-5088/:5109-5112).
    let mut t = framed(4, 0, 0);
    t.flags &= !tr_core::tile::FLAG_ACTIVE;
    assert!(!check_water_death(&t));
    assert!(!check_lava_death(&t));
    // Chest 21: lava_death false, no global membership -> survives lava.
    assert!(!check_lava_death(&framed(21, 0, 0)));
}
