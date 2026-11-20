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

// ---------------------------------------------------------------------------
// Batch 2 — kinds 630..654 (emission indices 521..547, 27 defs).
// ---------------------------------------------------------------------------

/// The stinkbug housing blockers 630/631 (`TileObjectData.cs:4446-4487`):
/// CopyFrom(Style1x1) + bottom SolidTile|SolidWithTop|SolidSide|AlternateTile
/// (=43), alternate tiles {124,561,574,575,576,577,578}, horizontal, then FOUR
/// alternates: 1 ceiling (top 169), 2 right (right 57), 3 left (left 57),
/// 4 wall (AnchorWall). The alternates keep the invalid list + alternate tiles.
fn stinkbug_blocker(kind: u16) -> Want {
    let mut w = style1x1(kind);
    w.bot = a(43, 1);
    w.alt_tiles = &[124, 561, 574, 575, 576, 577, 578];
    w.horiz = true;
    w.nalts = 4;
    w
}

#[test]
fn e12_w2_kind_630_631_stinkbug_blockers() {
    for kind in [630u16, 631] {
        check(kind, &stinkbug_blocker(kind));
        let d = def_for_tile(kind).unwrap();
        // A1 (:4450-4453 / :4471-4474): bottom empty, top 169, style 1.
        let a1 = get_tile_data(d, 0, 1).unwrap();
        assert_eq!((a1.style, a1.anchor_bottom, a1.anchor_top), (1, E, a(169, 1)));
        // A2 (:4454-4457): bottom empty, right 57, style 2.
        let a2 = get_tile_data(d, 0, 2).unwrap();
        assert_eq!((a2.style, a2.anchor_bottom, a2.anchor_right), (2, E, a(57, 1)));
        // A3 (:4458-4461): bottom empty, left 57, style 3.
        let a3 = get_tile_data(d, 0, 3).unwrap();
        assert_eq!((a3.style, a3.anchor_bottom, a3.anchor_left), (3, E, a(57, 1)));
        // A4 (:4462-4465): bottom empty, AnchorWall, style 4.
        let a4 = get_tile_data(d, 0, 4).unwrap();
        assert_eq!((a4.style, a4.anchor_bottom, a4.anchor_wall), (4, E, true));
        // The alternates inherit the invalid list + alternate tiles (CopyFrom).
        assert_eq!(a1.anchor_invalid_tiles, INVALID);
        assert_eq!(a1.anchor_alternate_tiles, &[124, 561, 574, 575, 576, 577, 578]);
    }
}

#[test]
fn e12_w2_kind_632_640_643_644_645_cages() {
    // 632 ScarletMacaw (:2725), 640 BlueMacaw (:2727), 643 Toucan (:2729),
    // 644 YellowCockatiel (:2731), 645 GrayCockatiel (:2733): bare Style6x3.
    for kind in [632u16, 640, 643, 644, 645] {
        check(kind, &style6x3(kind));
    }
}

#[test]
fn e12_w2_kind_637_ash_plants() {
    // 637 (:4321-4324): CopyFrom(Style1x1Plant_Height22 :4281-4287 — Style1x1
    // + horizontal + WaterPlacement NotAllowed + LavaDeath true) then
    // ApplyNaturalObjectRules (custom=false, global=true :1847-1851) + the
    // PlantCheck_CanPlaceHook (badReturn 0). WaterDeath stays false.
    let mut w = style1x1(637);
    w.horiz = true;
    w.water = LP::NotAllowed;
    w.custom = false;
    w.global = true;
    check(637, &w);
    let d = def_for_tile(637).unwrap();
    assert_eq!(d.hook_check_if_can_place.callback, "WorldGen.PlantCheck_CanPlaceHook");
    assert_eq!(d.hook_check_if_can_place.bad_return, 0);
    assert!(!d.hook_check_if_can_place.processed_coordinates);
}

#[test]
fn e12_w2_kind_639_642_style2x2_and_3x3() {
    // 639 ManaCrystal (:3986-3987): bare CopyFrom(Style2x2).
    check(639, &style2x2(639));
    // 642 ChlorophyteExtractinator (:4043-4045): CopyFrom(Style3x3 — the SECOND
    // definition :4021-4030, bottom 11 no Table) + DrawYOffset 2 (draw class).
    let w = Want {
        kind: 642, w: 3, h: 3, ox: 1, oy: 2, dir: Dir::None,
        heights: &[16, 16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(11, 3), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    };
    check(642, &w);
}

/// The `StyleOnTable1x1` base (`TileObjectData.cs:2325-2334`): 1x1 origin
/// (0,0), bottom Table (=4) width 1, custom, heights {16}, cw16 pad2,
/// LavaDeath=true. Horizontal stays base false.
fn style_on_table(kind: u16) -> Want {
    Want {
        kind, w: 1, h: 1, ox: 0, oy: 0, dir: Dir::None,
        heights: &[16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(4, 1), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w2_kind_646_shadow_candle() {
    // 646 (:2370-2373): CopyFrom(StyleOnTable1x1) + heights {20} + DrawYOffset
    // -4 (draw class). Placement identity: the 20-px row.
    let mut w = style_on_table(646);
    w.heights = &[20];
    check(646, &w);
}

/// The `Style3x2` base (`TileObjectData.cs:3169-3179`): 3x2 origin (1,1),
/// bottom 11 width 3, custom, heights 16x2, cw16 pad2, horizontal,
/// LavaDeath=true.
fn style3x2(kind: u16) -> Want {
    Want {
        kind, w: 3, h: 2, ox: 1, oy: 1, dir: Dir::None,
        heights: &[16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: true, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(11, 3), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w2_kind_647_648_651_echo_piles() {
    // 647 LargePilesEcho (:3217-3219): Style3x2 + DrawYOffset 2 (draw class).
    check(647, &style3x2(647));
    // 648 LargePiles2Echo (:3220-3223): Style3x2 + StyleWrapLimit 35.
    let mut w = style3x2(648);
    w.wrap = 35;
    check(648, &w);
    // 651 PlantDetritus3x2Echo (:3227-3229): bare Style3x2 + DrawYOffset 2.
    check(651, &style3x2(651));
}

/// The `Style2x1` base (`TileObjectData.cs:2798-2808`): 2x1 origin (0,0),
/// bottom 11 width 2, custom, heights {16}, cw16 pad2, horizontal,
/// LavaDeath=true.
fn style2x1(kind: u16) -> Want {
    Want {
        kind, w: 2, h: 1, ox: 0, oy: 0, dir: Dir::None,
        heights: &[16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: true, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(11, 2), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w2_kind_649_small_piles_echo() {
    // 649 SmallPiles2x1Echo (:2838-2841): Style2x1 + StyleWrapLimit 53.
    let mut w = style2x1(649);
    w.wrap = 53;
    check(649, &w);
}

#[test]
fn e12_w2_kind_650_small_piles_1x1_echo() {
    // 650 (:4411-4414): Style1x1 + DrawYOffset 2 (draw class) + horizontal.
    let mut w = style1x1(650);
    w.horiz = true;
    check(650, &w);
}

#[test]
fn e12_w2_kind_652_653_654_style2x2_variants() {
    // 652 PlantDetritus2x2Echo (:3537-3540): Style2x2 + horizontal.
    let mut w = style2x2(652);
    w.horiz = true;
    check(652, &w);
    // 653 PotsEcho (:3541-3547): Style2x2 + bottom 11 (no Table) + horizontal
    // + StyleWrapLimit 3 (the styleWrapLimit local :3541).
    let mut w = style2x2(653);
    w.bot = a(11, 2);
    w.horiz = true;
    w.wrap = 3;
    check(653, &w);
    // 654 TNTBarrel (:3654-3658): Style2x2 + heights {16,18} + LavaDeath true.
    let mut w = style2x2(654);
    w.heights = &[16, 18];
    check(654, &w);
}

// ---------------------------------------------------------------------------
// Batch 3 — kinds 656..698 (emission indices 548..567, 20 defs).
// ---------------------------------------------------------------------------

#[test]
fn e12_w3_kind_656_glow_tulip() {
    // 656 (:4269-4270): bare CopyFrom(Style1x1).
    check(656, &style1x1(656));
}

/// The `Style2xX` base (`TileObjectData.cs:2860-2869`): 2x3 origin (1,2),
/// bottom 11 width 2, custom, heights 16x3, cw16 pad2, horizontal. The base
/// block does NOT set LavaDeath (the newTile cursor was reset to _baseObject
/// by the preceding addTile), so LavaDeath stays false and LavaPlacement
/// stays the SetupBaseObject NotAllowed.
fn style2xX(kind: u16) -> Want {
    Want {
        kind, w: 2, h: 3, ox: 1, oy: 2, dir: Dir::None,
        heights: &[16, 16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: true, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: E, bot: a(11, 2), left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: false,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w3_kind_657_658_663_monoliths() {
    // 657 EchoMonolith (:2907-2912) and 658 ShimmerMonolith (:2913-2918):
    // CopyFrom(Style2xX) + Height 3/origin (1,2)/heights 16x3 (the base values
    // re-written) + DrawYOffset 2 (draw class). 663 Moondial (:3803-3804):
    // bare CopyFrom(Style2xX).
    check(657, &style2xX(657));
    check(658, &style2xX(658));
    check(663, &style2xX(663));
}

/// The `Style1x2Top` base (`TileObjectData.cs:2432-2441`): 1x2 origin (0,0),
/// TOP SolidTile|SolidSide|PlanterBox (=521) width 1, custom, heights 16x2,
/// cw16 pad2, LavaDeath=true. Horizontal stays base false.
fn style1x2top(kind: u16) -> Want {
    Want {
        kind, w: 1, h: 2, ox: 0, oy: 0, dir: Dir::None,
        heights: &[16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: false, wrap: 0, mult: 1, skip: 1, rand: 0,
        top: a(521, 1), bot: E, left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 0,
    }
}

#[test]
fn e12_w3_kind_660_shimmerfly_bottle() {
    // 660 (:2463-2469): CopyFrom(Style1x2Top) + DrawYOffset -2 (draw class) +
    // ONE platform alternate (top Platform=256, DrawYOffset -10, style 0 —
    // addAlternate(0) keeps Style 0).
    let mut w = style1x2top(660);
    w.nalts = 1;
    check(660, &w);
    let d = def_for_tile(660).unwrap();
    let alt = get_tile_data(d, 0, 1).unwrap();
    assert_eq!(alt.anchor_top, a(256, 1), "660 A1 platform anchor (:2466)");
    assert_eq!(alt.style, 0, "660 A1 style 0 (addAlternate(0) :2468)");
}

#[test]
fn e12_w3_kind_664_665_boulders() {
    // 664 BouncyBoulder (:3631-3634): Style2x2 + heights {16,18} +
    // LavaDeath=false (the lava placement stays NotAllowed — no own-liquid
    // fold clears it; Calculate only folds death->NotAllowed).
    let mut w = style2x2(664);
    w.heights = &[16, 18];
    w.ldeath = false;
    check(664, &w);
    // 665 LifeCrystalBoulder (:3975-3976): bare CopyFrom(Style2x2).
    check(665, &style2x2(665));
}

#[test]
fn e12_w3_kind_693_stalactite_1x1_echo() {
    // 693 (:4415-4425): CopyFrom(Style1x1) + horizontal + bottom empty + top
    // 169 + StyleWrapLimit 39 (the num2 local :4415); ONE sub-tile object
    // registered under styles 39..77 (addSubTileRange(39,39) :4424 — the SAME
    // object under every style, :2043-2070): top empty, bottom 11.
    let mut w = style1x1(693);
    w.horiz = true;
    w.bot = E;
    w.top = a(169, 1);
    w.wrap = 39;
    w.nsubs = 39;
    check(693, &w);
    let d = def_for_tile(693).unwrap();
    // Every style 39..77 resolves the same sub-tile; style 38 stays the base.
    let sub39 = get_tile_data(d, 39, 0).unwrap();
    assert_eq!(sub39.anchor_top, E);
    assert_eq!(sub39.anchor_bottom, a(11, 1));
    for style in 40..=77 {
        let sub = get_tile_data(d, style, 0).unwrap();
        assert_eq!(sub.anchor_bottom, a(11, 1), "693 sub style {style}");
    }
    let base38 = get_tile_data(d, 38, 0).unwrap();
    assert_eq!(base38.kind, 693, "style 38 keeps the ceiling base");
    assert_eq!(base38.anchor_top, a(169, 1));
}

#[test]
fn e12_w3_kind_694_stalactite_1x2_echo() {
    // 694 (:4227-4239): CopyFrom(Style1x2 :4131-4140 — 1x2 origin (0,1)
    // bottom 11) + horizontal + bottom empty + top 169 + wrap 39 + origin
    // (0,0); ONE sub-tile under styles 39..77: origin (0,1), top empty,
    // bottom 11.
    let w = Want {
        kind: 694, w: 1, h: 2, ox: 0, oy: 0, dir: Dir::None,
        heights: &[16, 16], cw: 16, pad: 2, fx: 0, fy: 0,
        horiz: true, wrap: 39, mult: 1, skip: 1, rand: 0,
        top: a(169, 1), bot: E, left: E, right: E, wall: false,
        valid: &[], invalid: &[], alt_tiles: &[], walls: &[],
        water: LP::Allowed, lava: LP::NotAllowed, wdeath: false, ldeath: true,
        custom: true, global: false, nalts: 0, nsubs: 39,
    };
    check(694, &w);
    let d = def_for_tile(694).unwrap();
    let sub = get_tile_data(d, 39, 0).unwrap();
    assert_eq!((sub.origin.x, sub.origin.y), (0, 1), "694 sub origin (:4235)");
    assert_eq!(sub.anchor_top, E);
    assert_eq!(sub.anchor_bottom, a(11, 1));
}

#[test]
fn e12_w3_kind_695_696_replicas() {
    // 695 DemonAltarReplica (:3234-3237): Style3x2 + LavaDeath=false +
    // heights {16,18}.
    let mut w = style3x2(695);
    w.ldeath = false;
    w.heights = &[16, 18];
    check(695, &w);
    // 696 ShadowOrbsReplica (:3988-3991): Style2x2 + horizontal + bottom empty.
    let mut w = style2x2(696);
    w.horiz = true;
    w.bot = E;
    check(696, &w);
}

#[test]
fn e12_w3_kind_698_display_jar() {
    // 698 DeadCellsDisplayJar (:2470-2502): CopyFrom(Style1x2Top) +
    // DrawYOffset -2 + horizontal + DrawXOffset 10 + DrawFrameOffsets (draw
    // class) + top 681 (SolidTile|SolidSide|AlternateTile|SolidBottom|
    // PlanterBox) + alternate tiles {213,366,365,353,214,449,450,451} + the
    // TEDeadCellsDisplayJar.Hook_AfterPlacement post hook (-1, processed).
    // ONE platform alternate (top 256, style 0) + TWO sub-tiles (styles 1/2),
    // each LinkedAlternates with its OWN platform alternate.
    let mut w = style1x2top(698);
    w.horiz = true;
    w.top = a(681, 1);
    w.alt_tiles = &[213, 366, 365, 353, 214, 449, 450, 451];
    w.nalts = 1;
    w.nsubs = 2;
    check(698, &w);
    let d = def_for_tile(698).unwrap();
    assert_eq!(
        d.hook_post_place_my_player.callback,
        "TEDeadCellsDisplayJar.Hook_AfterPlacement"
    );
    assert_eq!(d.hook_post_place_my_player.bad_return, -1);
    assert!(d.hook_post_place_my_player.processed_coordinates);
    // The base alternate (:2482-2485): platform top, style 0.
    let alt = get_tile_data(d, 0, 1).unwrap();
    assert_eq!(alt.anchor_top, a(256, 1));
    // Sub-tile 1 (:2486-2493): keeps the base top 681 + its OWN alternate.
    let sub1 = get_tile_data(d, 1, 0).unwrap();
    assert_eq!(sub1.anchor_top, a(681, 1));
    assert_eq!(sub1.alternates.len(), 1, "698 sub1 linked alternate");
    // GetTileData indexes the RESOLVED sub-tile's alternates (:5148-5157).
    let sub1_alt = get_tile_data(d, 1, 1).unwrap();
    assert_eq!(sub1_alt.anchor_top, a(256, 1), "698 sub1 platform alternate");
    let sub2 = get_tile_data(d, 2, 0).unwrap();
    assert_eq!(sub2.anchor_top, a(681, 1));
    let sub2_alt = get_tile_data(d, 2, 1).unwrap();
    assert_eq!(sub2_alt.anchor_top, a(256, 1), "698 sub2 platform alternate");
}