//! W4-e12 — verbatim per-def witnesses for the TileObjectData remainder range.
//!
//! Sixth/remainder lane of the TileObjectData batch (d111–d114/e11 before).
//! Census first: the "missing defs" premise is STALE — S1-F-7 wave A generated
//! the FULL registry. `python tools/GenTileObjectData/gen.py . --check` reports
//! `registry.rs is current (389 tiles, 604 defs)`, and the distinct `addTile`
//! kinds of `Initialize` (`TileObjectData.cs:2110-5014`, incl. the 435..439 loop
//! `:2149-2160`) equal the registry's top-level kinds exactly (389 == 389, empty
//! set-difference both ways). Zero placement defs are missing.
//!
//! This file pins the REMAINDER range — emission indices 501–604 of
//! `tileobject/registry.rs`, the last 104 defs, owned by tile kinds 605–753
//! (76 kinds; verified kind-disjoint from the earlier batches). Every expected
//! value below was read VERBATIM from the C# builder chain (the `CopyFrom`
//! base style + the per-kind overrides), NOT copied from the generated table —
//! the witnesses re-derive the C# and check the port against it.
//!
//! The genuinely missing TileObjectData DATA (the draw-field class:
//! `DrawXOffset/DrawYOffset/DrawFlip*/DrawStepDown/DrawStyleOffset/
//! DrawFrameOffsets` + the two visual overrides) is parsed by gen.py but not
//! emitted (S1-F-9 remainder, `tileobject.rs:50-55`; sole C# consumer
//! `TileObject.DrawPreview`, `TileObject.cs:855-953`). Its per-kind values for
//! this range are recorded in `docs/research/08-tile-framing.md` ("W4-e12").
//!
//! Data-only; no production behaviour touched; RNG untouched (pure data).
//! Witnesses WRITTEN NOT RUN (code-only mode).

use tr_core::tileobject::{
    def_for_tile, get_tile_data, AnchorData, GetStyleOverride, Interaction,
    LiquidPlacement as LP, TileObjectDef, TileObjectDirection as Dir,
};

/// The 11-kind `anchorInvalidTiles` local of `Initialize`
/// (`TileObjectData.cs:2112-2116`).
const INVALID: &[u16] = &[127, 138, 664, 665, 484, 711, 712, 713, 714, 715, 716];

/// One def's verbatim placement identity. Every field carries its C# source in
/// the per-def constructor below; the checker asserts the port resolves to it.
struct Want {
    kind: u16,
    w: i32,
    h: i32,
    ox: i32,
    oy: i32,
    dir: Dir,
    heights: &'static [i32],
    cw: i32,
    pad: i32,
    fx: i32,
    fy: i32,
    horiz: bool,
    wrap: i32,
    mult: i32,
    skip: i32,
    rand: i32,
    top: AnchorData,
    bot: AnchorData,
    left: AnchorData,
    right: AnchorData,
    wall: bool,
    valid: &'static [u16],
    invalid: &'static [u16],
    alt_tiles: &'static [u16],
    walls: &'static [u16],
    water: LP,
    lava: LP,
    wdeath: bool,
    ldeath: bool,
    custom: bool,
    global: bool,
    nalts: usize,
    nsubs: usize,
}

const E: AnchorData = AnchorData::EMPTY;
const fn a(ty: u16, n: i32) -> AnchorData {
    AnchorData::new(ty, n, 0)
}

fn check(kind: u16, want: &Want) {
    let d: &TileObjectDef =
        def_for_tile(kind).unwrap_or_else(|| panic!("kind {kind} must be registered"));
    assert_eq!(d.kind, want.kind, "kind {kind}: registry key");
    assert_eq!((d.width, d.height), (want.w, want.h), "kind {kind}: size");
    assert_eq!((d.origin.x, d.origin.y), (want.ox, want.oy), "kind {kind}: origin");
    assert_eq!(d.direction, want.dir, "kind {kind}: direction");
    assert_eq!(d.coordinate_heights, want.heights, "kind {kind}: coord heights");
    assert_eq!(d.coordinate_width, want.cw, "kind {kind}: coord width");
    assert_eq!(d.coordinate_padding, want.pad, "kind {kind}: coord padding");
    assert_eq!(
        (d.coordinate_padding_fix.x, d.coordinate_padding_fix.y),
        (want.fx, want.fy),
        "kind {kind}: padding fix"
    );
    assert_eq!(d.style_horizontal, want.horiz, "kind {kind}: style horizontal");
    assert_eq!(d.style_wrap_limit, want.wrap, "kind {kind}: wrap");
    assert_eq!(d.style_multiplier, want.mult, "kind {kind}: multiplier");
    assert_eq!(d.style_line_skip, want.skip, "kind {kind}: line skip");
    assert_eq!(d.random_style_range, want.rand, "kind {kind}: random range");
    assert!(d.specific_random_styles.is_empty(), "kind {kind}: SRS null in every def");
    assert!(!d.flatten_anchors, "kind {kind}: no range def flattens anchors");
    assert_eq!(d.anchor_top, want.top, "kind {kind}: anchor top");
    assert_eq!(d.anchor_bottom, want.bot, "kind {kind}: anchor bottom");
    assert_eq!(d.anchor_left, want.left, "kind {kind}: anchor left");
    assert_eq!(d.anchor_right, want.right, "kind {kind}: anchor right");
    assert_eq!(d.anchor_wall, want.wall, "kind {kind}: anchor wall");
    assert_eq!(d.anchor_valid_tiles, want.valid, "kind {kind}: valid tiles");
    assert_eq!(d.anchor_invalid_tiles, want.invalid, "kind {kind}: invalid tiles");
    assert_eq!(d.anchor_alternate_tiles, want.alt_tiles, "kind {kind}: alternate tiles");
    assert_eq!(d.anchor_valid_walls, want.walls, "kind {kind}: valid walls");
    assert_eq!(d.water_placement, want.water, "kind {kind}: water placement");
    assert_eq!(d.lava_placement, want.lava, "kind {kind}: lava placement");
    assert_eq!(d.water_death, want.wdeath, "kind {kind}: water death");
    assert_eq!(d.lava_death, want.ldeath, "kind {kind}: lava death");
    assert_eq!(d.uses_custom_can_place, want.custom, "kind {kind}: custom can place");
    assert_eq!(d.uses_global_liquid_checks, want.global, "kind {kind}: global liquid");
    assert_eq!(d.get_style_override, GetStyleOverride::None, "kind {kind}: no style override");
    assert_eq!(d.alternates.len(), want.nalts, "kind {kind}: alternate count");
    assert_eq!(d.subtiles.len(), want.nsubs, "kind {kind}: sub-tile count");
    // The range's per-TYPE overlays: every kind is frame-important
    // (Main.tileFrameImportant) and only 642 carries the Extractinator tag.
    assert!(d.frame_important, "kind {kind}: frame important");
    let inter = if kind == 642 { Interaction::Extractinator } else { Interaction::None };
    assert_eq!(d.interaction, inter, "kind {kind}: interaction overlay");
    // GetTileData(kind, 0, 0) resolves to the base def itself (:5126-5160).
    let base = get_tile_data(d, 0, 0).expect("base resolves");
    assert_eq!(base.kind, kind, "kind {kind}: GetTileData base");
}

// ---------------------------------------------------------------------------
// Batch 1 — kinds 605..629 (emission indices 501..520, 20 defs).
// ---------------------------------------------------------------------------

/// The `Style6x3` base (`TileObjectData.cs:2696-2706`): 6x3, origin (3,2),
/// bottom SolidTile|Table|SolidSide (=13) width 6, custom, heights 16x3,
/// cw 16 pad 2, LavaDeath=true (Calculate folds LavaPlacement=NotAllowed).
/// StyleHorizontal stays the base false (the base block never sets it).
fn style6x3(kind: u16) -> Want {
    Want {
        kind, w: 6, h: 3, ox: 3, oy: 2, dir: Dir::None,
        heights: &[16, 16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(13, 6), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w1_big_cages_copy_style6x3() {
    // 605..612 the gem squirrel/amber bunny cages (:2767-2782): bare
    // CopyFrom(Style6x3). (632/640/643/644/645/710 ride the same base in w2/w4.)
    for kind in [605u16, 606, 607, 608, 609, 610, 611, 612] {
        check(kind, &style6x3(kind));
    }
}

#[test]
fn e12_w1_kind_613_614_potted_lava_plants() {
    // 613 PottedLavaPlants (:3123-3132): CopyFrom(Style3x3 :3097-3105, bottom 15) then Height 5,
    // origin (1,4), bottom SolidTile|SolidWithTop|SolidSide (=11) width 3,
    // heights 16x5, LavaDeath=false + LavaPlacement=Allowed, StyleHorizontal.
    let mut w = style3x3_tall(613, 5, 4);
    w.heights = &[16, 16, 16, 16, 16];
    check(613, &w);
    // 614 PottedLavaPlantTendrils (:3133-3142): the same with Height 6, origin (1,5), heights 16x6.
    let mut w = style3x3_tall(614, 6, 5);
    w.heights = &[16, 16, 16, 16, 16, 16];
    check(614, &w);
}

/// The 613/614 shape: Style3x3 with a taller height, bottom 11 (no Table),
/// LavaDeath cleared and LavaPlacement Allowed, horizontal styles.
fn style3x3_tall(kind: u16, h: i32, oy: i32) -> Want {
    Want {
        kind, w: 3, h, ox: 1, oy, dir: Dir::None,
        heights: &[16, 16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: true, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(11, 3), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::Allowed, wdeath: false, ldeath: false,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w1_kind_615_willow_saplings() {
    // 615 VanityTreeWillowSaplings (:4948-4964): 1x2 origin (0,1), bottom SolidTile (=1) width 1, custom,
    // heights {16,18}, cw16 pad2, valid {2,477,492,60,109,199,23}, horizontal,
    // WaterPlacement NotAllowed, LavaDeath=false, RandomStyleRange 3, mult 3.
    let w = Want {
        kind: 615, w: 1, h: 2, ox: 0, oy: 1, dir: Dir::None,
        heights: &[16, 18], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: true, wrap: 0, mult: 3, skip: 1, rand: 3,
        top: E, bot: a(1, 1), left: E, right: E, wall: false,
        valid: &[2, 477, 492, 60, 109, 199, 23], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::NotAllowed, lava: LP::NotAllowed, wdeath: false, ldeath: false,
        custom: true, global: false, nalts: 0, nsubs: 0,
    };
    check(615, &w);
}

#[test]
fn e12_w1_kind_617_master_trophy_base() {
    // 617 MasterTrophyBase (:3424-3436): CopyFrom(Style3x4 :3383-3393) then LavaDeath=false,
    // Direction PlaceLeft, StyleHorizontal=false, StyleMultiplier 2,
    // StyleWrapLimit 2; one PlaceRight alternate style 1. (The visual overrides
    // WVO=2/LSVO=0 :3429/:3432 are the un-emitted draw class — research/08.)
    let w = Want {
        kind: 617, w: 3, h: 4, ox: 1, oy: 3, dir: Dir::PlaceLeft,
        heights: &[16, 16, 16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 2, mult: 2, skip: 1, rand: 0,
        top: E, bot: a(11, 3), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: false,
        custom: true, global: false, nalts: 1, nsubs: 0,
    };
    check(617, &w);
    let d = def_for_tile(617).unwrap();
    let alt = get_tile_data(d, 0, 1).expect("617 alternate 1");
    assert_eq!(alt.direction, Dir::PlaceRight, "617 A1 PlaceRight (:3434)");
    assert_eq!(alt.style, 1, "617 A1 style 1 (addAlternate(1) :3435)");
}

/// The `StyleSmallCage` base (`TileObjectData.cs:2657-2667`): 3x2 origin (1,1),
/// bottom SolidTile|Table|SolidSide (=13) width 3, custom, heights 16x2,
/// cw16 pad2, LavaDeath=true. Horizontal stays base false.
fn style_small_cage(kind: u16) -> Want {
    Want {
        kind, w: 3, h: 2, ox: 1, oy: 1, dir: Dir::None,
        heights: &[16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(13, 3), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w1_kind_619_629_small_cages() {
    // 619 TruffleWormCage (:2674-2675) and 629 StinkbugCage (:2694-2695):
    // bare CopyFrom(StyleSmallCage).
    check(619, &style_small_cage(619));
    check(629, &style_small_cage(629));
}

/// The `Style2x2` base (`TileObjectData.cs:3451-3460`): 2x2 origin (0,1),
/// bottom SolidTile|SolidWithTop|Table|SolidSide (=15) width 2, custom,
/// heights 16x2, cw16 pad2, LavaDeath=true. Horizontal stays base false.
fn style2x2(kind: u16) -> Want {
    Want {
        kind, w: 2, h: 2, ox: 0, oy: 1, dir: Dir::None,
        heights: &[16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(15, 2), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w1_kind_620_621_622_jar_cake_kettle() {
    // 620 EmpressButterflyJar (:3728-3730), 621 SliceOfCake (:3609-3611),
    // 622 TeaKettle (:3612-3614): CopyFrom(Style2x2)
    // + DrawYOffset 2 (draw class, not emitted). Placement identity = Style2x2.
    for kind in [620u16, 621, 622] {
        check(kind, &style2x2(kind));
    }
}

#[test]
fn e12_w1_kind_623_potted_crystal_plants() {
    // 623 PottedCrystalPlants (:2877-2883): CopyFrom(Style2xX :2860-2869, 2x3 horizontal bottom 11)
    // then Height 5, origin (1,4), heights 16x5, LavaDeath=true (already).
    let w = Want {
        kind: 623, w: 2, h: 5, ox: 1, oy: 4, dir: Dir::None,
        heights: &[16, 16, 16, 16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: true, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(11, 2), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    };
    check(623, &w);
}

/// The `Style1x1` base (`TileObjectData.cs:4240-4250`): 1x1 origin (0,0),
/// bottom SolidTile|SolidWithTop|SolidSide (=11) width 1, invalid = the
/// 11-kind local, custom, heights {16}, cw16 pad2, LavaDeath=true.
fn style1x1(kind: u16) -> Want {
    Want {
        kind, w: 1, h: 1, ox: 0, oy: 0, dir: Dir::None,
        heights: &[16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(11, 1), left: E, right: E, wall: false,
        valid: &[], invalid: INVALID, alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w1_kind_624_abigails_flower() {
    // 624 AbigailsFlower (:4256-4257): bare CopyFrom(Style1x1).
    check(624, &style1x1(624));
}