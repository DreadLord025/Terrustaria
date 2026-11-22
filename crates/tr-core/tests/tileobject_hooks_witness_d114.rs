//! W4-d114 witnesses — the TileObjectData HOOK-INVOCATION consumers and the
//! `ApplyNaturalObjectRules` flag consumers (S1-F-7 wave B, the unclaimed
//! slice after d112 `resolve_tile_def` / d113 `try_get_tile_bounds` / e11
//! tile-based death checks):
//!
//! * `CallPostPlacementPlayerHook` (`TileObjectData.cs:5269-5283`) →
//!   `tileobject::call_post_placement_player_hook` (+ the `_with_def` body),
//!   wired at `Player.cs:40144` (`edit::place_item_tile`) and `:45816`
//!   (`world_sim::editing::try_replanting_tree`).
//! * `OriginToTopLeft` (`:5285-5292`, the `Chest.AfterPlacement_Hook` opener
//!   `Chest.cs:562`) → `tileobject::origin_to_top_left`.
//! * `CheckWaterDeath(int, int)` / `CheckLavaDeath(int, int)`
//!   (`:5073-5081` / `:5097-5105`) → `check_water_death_for_style` /
//!   `check_lava_death_for_style` — the style-indexed readers of
//!   `UsesGlobalLiquidChecks`, the flag `ApplyNaturalObjectRules`
//!   (`:1847-1851`) sets on every natural object.
//!
//! WRITTEN NOT RUN (task constraint: no cargo in this lane); expected values
//! hand-derived from the cited C# lines + the generated registry defs.

use std::sync::atomic::{AtomicI32, Ordering};

use tr_core::tileobject::{
    call_post_placement_player_hook, call_post_placement_player_hook_with_def,
    check_lava_death_for_style, check_water_death_for_style, def_for_tile, get_tile_data,
    origin_to_top_left, PlacementHook, Point16, TileObjectDef, TileObjectInstance,
};
use tr_core::world::World;

fn world() -> World {
    World::new("t".into(), 16, 16)
}

fn instance(kind: u16, style: i32, alternate: i32) -> TileObjectInstance {
    TileObjectInstance {
        x_coord: 5,
        y_coord: 5,
        kind,
        style,
        alternate,
        random: -1,
    }
}

// ---- CallPostPlacementPlayerHook (TileObjectData.cs:5269-5283) ----

#[test]
fn w4_d114_post_hook_registry_defs_are_documented_noops() {
    // Wave A keeps every generated hook body as `hook: None` with the C#
    // method name in `callback` — the invocation seam answers false without
    // touching the world. Chest 21 (:3465), dresser 88 (:3202), item frame
    // 395 (:3950), training dummy 378 (:2998).
    let w = world();
    for (kind, callback, processed) in [
        (21u16, "Chest.AfterPlacement_Hook", false),
        (88, "Chest.AfterPlacement_Hook", false),
        (395, "TEItemFrame.Hook_AfterPlacement", true),
        (378, "TETrainingDummy.Hook_AfterPlacement", false),
    ] {
        let def = def_for_tile(kind).unwrap();
        let hook = def.hook_post_place_my_player;
        assert_eq!(hook.callback, callback);
        assert_eq!(hook.bad_return, -1);
        assert_eq!(hook.bad_response, 0);
        assert_eq!(hook.processed_coordinates, processed);
        assert!(hook.hook.is_none(), "wave-A documented callback seam");
        let data = instance(kind, 0, 0);
        assert!(!call_post_placement_player_hook(
            &w, 10, 12, kind, 0, 1, data.alternate, &data
        ));
    }
}

#[test]
fn w4_d114_post_hook_unregistered_kind_or_bad_style_is_false() {
    let w = world();
    // :5271-5275 — `_data[type] == null` (dirt 1) resolves nothing.
    let data = instance(1, 0, 0);
    assert!(!call_post_placement_player_hook(
        &w, 10, 12, 1, 0, 1, 0, &data
    ));
    // A negative style throws in C# (:5132-5135); the port degrades to false.
    let data = instance(21, 0, 0);
    assert!(!call_post_placement_player_hook(
        &w, 10, 12, 21, -1, 1, 0, &data
    ));
}

/// Returns -1 (`Chest.AfterPlacement_Hook`'s failure return, Chest.cs:564-566).
fn bad_hook(_w: &World, _x: i32, _y: i32, _k: u16, _s: i32, _d: i32, _a: i32) -> i32 {
    -1
}
fn ok_hook(_w: &World, _x: i32, _y: i32, _k: u16, _s: i32, _d: i32, _a: i32) -> i32 {
    0
}

#[test]
fn w4_d114_post_hook_bad_return_quirk() {
    // :5282 — `return hook(...) == badReturn`: with the chest metadata
    // (badReturn -1, the FAILURE return of Chest.AfterPlacement_Hook,
    // Chest.cs:564-566) the C# answers TRUE when the hook FAILED. Verbatim.
    let w = world();
    let def = TileObjectDef {
        hook_post_place_my_player: PlacementHook {
            hook: Some(bad_hook),
            callback: "test",
            bad_return: -1,
            bad_response: 0,
            processed_coordinates: false,
        },
        ..Default::default()
    };
    let data = instance(21, 0, 0);
    assert!(call_post_placement_player_hook_with_def(
        &w, 10, 12, 21, 0, 1, &def, &data
    ));
    // A successful hook (return 0 != -1) answers false.
    let ok = TileObjectDef {
        hook_post_place_my_player: PlacementHook {
            hook: Some(ok_hook),
            ..def.hook_post_place_my_player
        },
        ..Default::default()
    };
    assert!(!call_post_placement_player_hook_with_def(
        &w, 10, 12, 21, 0, 1, &ok, &data
    ));
}

/// Records the coordinates it was called with (the processed-coordinates
/// witness). Per-test atomics — tests run in parallel, no shared recorder.
static SEEN_X: AtomicI32 = AtomicI32::new(0);
static SEEN_Y: AtomicI32 = AtomicI32::new(0);
fn coord_hook(_w: &World, x: i32, y: i32, _k: u16, _s: i32, _d: i32, _a: i32) -> i32 {
    SEEN_X.store(x, Ordering::SeqCst);
    SEEN_Y.store(y, Ordering::SeqCst);
    0
}

#[test]
fn w4_d114_post_hook_processed_coordinates_subtract_origin() {
    // :5277-5281 — processedCoordinates subtracts the RESOLVED def's Origin;
    // unprocessed hooks receive the raw target (the chest passes false and
    // re-derives the top-left itself via OriginToTopLeft, Chest.cs:562).
    let w = world();
    let data = instance(21, 0, 0);
    let processed = TileObjectDef {
        origin: Point16::new(1, 2),
        hook_post_place_my_player: PlacementHook {
            hook: Some(coord_hook),
            callback: "test",
            bad_return: -1,
            bad_response: 0,
            processed_coordinates: true,
        },
        ..Default::default()
    };
    call_post_placement_player_hook_with_def(&w, 10, 12, 21, 0, 1, &processed, &data);
    assert_eq!((SEEN_X.load(Ordering::SeqCst), SEEN_Y.load(Ordering::SeqCst)), (9, 10));
    let raw = TileObjectDef {
        hook_post_place_my_player: PlacementHook {
            processed_coordinates: false,
            ..processed.hook_post_place_my_player
        },
        ..processed
    };
    call_post_placement_player_hook_with_def(&w, 10, 12, 21, 0, 1, &raw, &data);
    assert_eq!((SEEN_X.load(Ordering::SeqCst), SEEN_Y.load(Ordering::SeqCst)), (10, 12));
}

static SEEN_ALT: AtomicI32 = AtomicI32::new(0);
fn alt_hook(_w: &World, _x: i32, _y: i32, _k: u16, _s: i32, _d: i32, alt: i32) -> i32 {
    SEEN_ALT.store(alt, Ordering::SeqCst);
    0
}

#[test]
fn w4_d114_post_hook_reads_data_alternate_not_the_parameter() {
    // :5271/:5282 — the body reads `data.alternate` everywhere; the standalone
    // `alternate` parameter is never read (kept 1:1 in the port signature).
    let w = world();
    let def = TileObjectDef {
        hook_post_place_my_player: PlacementHook {
            hook: Some(alt_hook),
            callback: "test",
            bad_return: -1,
            bad_response: 0,
            processed_coordinates: false,
        },
        ..Default::default()
    };
    let data = instance(21, 0, 3);
    call_post_placement_player_hook_with_def(&w, 10, 12, 21, 0, 1, &def, &data);
    assert_eq!(SEEN_ALT.load(Ordering::SeqCst), 3);
}

// ---- OriginToTopLeft (TileObjectData.cs:5285-5292) ----

#[test]
fn w4_d114_origin_to_top_left_shifts_by_resolved_origin() {
    // Chest 21: Origin (0,1) (:3462) — the anchor row is the BOTTOM row.
    assert_eq!(origin_to_top_left(21, 0, Point16::new(10, 12)), Point16::new(10, 11));
    // Oasis plants 530: Origin (1,1) (Style3x2 :3171, chain :3266-3272).
    assert_eq!(origin_to_top_left(530, 0, Point16::new(10, 12)), Point16::new(9, 11));
    // Sub-tile resolution: piano 87 style 15 keeps Origin (1,1) (:3205 copy).
    assert_eq!(origin_to_top_left(87, 15, Point16::new(10, 12)), Point16::new(9, 11));
}

#[test]
fn w4_d114_origin_to_top_left_leaves_unresolved_points_unchanged() {
    // :5288 — `tileData != null` gates the shift: unregistered dirt 1 and a
    // negative style (the C# throw, degraded) keep the point.
    assert_eq!(origin_to_top_left(1, 0, Point16::new(10, 12)), Point16::new(10, 12));
    assert_eq!(origin_to_top_left(21, -1, Point16::new(10, 12)), Point16::new(10, 12));
}

// ---- CheckWaterDeath/CheckLavaDeath(type, style) (:5073-5105) ----

#[test]
fn w4_d114_water_death_style_def_wins_when_not_global() {
    // Torch 4: UsesGlobalLiquidChecks false, WaterDeath true (StyleTorch
    // :4590, Main.tileWaterDeath[4] true, Main.cs:8062) → true; the style-8
    // underwater sub-tile sets WaterDeath = false (:4609) → false despite
    // the global table — the STYLE resolution matters on the water side too.
    assert!(check_water_death_for_style(4, 0));
    assert!(!check_water_death_for_style(4, 8));
    // Fireplace 405 / shadow candle 646 / peace candle 372: WaterDeath false
    // in the def (no WaterDeath assignment in their chains) but present in
    // Main.tileWaterDeath (Main.cs:8309/:9504/:9508) → the DEF wins: false.
    for kind in [405u16, 646, 372] {
        assert!(!def_for_tile(kind).unwrap().uses_global_liquid_checks);
        assert!(!def_for_tile(kind).unwrap().water_death);
        assert!(!check_water_death_for_style(kind, 0));
    }
    // Lava lamp 215: WaterDeath true (global table too, :8061) → true.
    assert!(check_water_death_for_style(215, 0));
}

#[test]
fn w4_d114_water_death_style_natural_objects_read_the_global_table() {
    // ApplyNaturalObjectRules (:1847-1851) sets UsesGlobalLiquidChecks — the
    // def's own flags stop mattering. Life fruit 236 sets WaterDeath = true
    // (:4004) but is natural (:4002) and 236 is NOT in Main.tileWaterDeath
    // → false (the global table wins over the def flag).
    let def = def_for_tile(236).unwrap();
    assert!(def.uses_global_liquid_checks && def.water_death);
    assert!(!check_water_death_for_style(236, 0));
    // Unregistered kinds answer the global table: vines 51 (Main.cs:8063)
    // true, dirt 1 false; 697 (:8064) true.
    assert!(check_water_death_for_style(51, 0));
    assert!(check_water_death_for_style(697, 0));
    assert!(!check_water_death_for_style(1, 0));
    // Negative style: the C# throw degrades to the global table.
    assert!(check_water_death_for_style(215, -1));
    assert!(!check_water_death_for_style(1, -1));
}

#[test]
fn w4_d114_lava_death_style_def_vs_global_and_subtile_resolution() {
    // Piano 87 base: LavaDeath true (Style3x2 :3178) → true; style 15 sub-tile
    // sets LavaDeath = false (:3206) → false — the STYLE resolution matters.
    assert!(check_lava_death_for_style(87, 0));
    assert!(!check_lava_death_for_style(87, 15));
    assert!(!def_for_tile(87).unwrap().uses_global_liquid_checks);
    // Natural objects read Main.tileLavaDeath: oasis plants 530 def LavaDeath
    // false (:3268) but natural (:3271) and Main.tileLavaDeath[530] = true
    // (Main.cs:7387) → true. Ash plants 637 def LavaDeath true (inherited
    // from Style1x1Plant_Height22 :4292) but natural (:4326) and
    // Main.tileLavaDeath[637] = false (Main.cs:7383) → false. Grass 3:
    // natural (:4298), table true (Main.cs:8068) → true.
    assert!(check_lava_death_for_style(530, 0));
    assert!(!check_lava_death_for_style(637, 0));
    assert!(check_lava_death_for_style(3, 0));
    // Unregistered dirt 1: not in the lava-death table → false.
    assert!(!check_lava_death_for_style(1, 0));
}

// ---- ApplyNaturalObjectRules census (:1847-1851, all 25 call sites) ----

#[test]
fn w4_d114_apply_natural_object_rules_flag_census() {
    // Every `ApplyNaturalObjectRules()` call site in Initialize sets
    // UsesCustomCanPlace = false + UsesGlobalLiquidChecks = true (:1849-1850).
    // Top-level kinds, by call-site line: 530 (:3271), 28 (:3549), 31 (:3995),
    // 236 (:4002), 238 (:4019), the plant band 3/24/71/110/201/61/637
    // (:4298-4323) and 73/74/113 (:4326-4335), 184 (:4687), the Style1x1Drip
    // base (:4707, copied by 373/374/375 :4709-4714 and 461/709), 233
    // (:4967), 185 (:4978), 165 (:4992).
    for kind in [
        530u16, 28, 31, 236, 238, 3, 24, 71, 110, 201, 61, 637, 73, 74, 113, 184, 373, 374,
        375, 461, 709, 233, 185, 165,
    ] {
        let def = def_for_tile(kind)
            .unwrap_or_else(|| panic!("kind {kind} registered (addTile census)"));
        assert!(def.uses_global_liquid_checks, "kind {kind} natural (:1850)");
        assert!(!def.uses_custom_can_place, "kind {kind} natural (:1849)");
    }
    // The sub-tile call sites: 233 sub0 (:4972), 185 sub0 (:4983), 165 sub0/1/2
    // (:4995/:5001/:5008) — each carries the flags on the SUB-TILE def.
    for (kind, style) in [(233u16, 0), (185, 0), (165, 0), (165, 1), (165, 2)] {
        let sub = get_tile_data(def_for_tile(kind).unwrap(), style, 0).unwrap();
        assert!(sub.uses_global_liquid_checks, "{kind} sub{style} (:1850)");
        assert!(!sub.uses_custom_can_place, "{kind} sub{style} (:1849)");
    }
    // The 184 alternates (:4688-4699, addAlternate 33/66/99 at :4691/:4695/
    // :4699) CopyFrom the natural newTile — the flags ride the copy
    // (alternate indices 1..=3).
    let base = def_for_tile(184).unwrap();
    for alt in 1..=3 {
        let a = get_tile_data(base, 0, alt).unwrap();
        assert!(a.uses_global_liquid_checks, "184 alt{alt} copies :4687");
        assert!(!a.uses_custom_can_place);
    }
}
