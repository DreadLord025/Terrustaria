// W4-d129 census — the `SceneMetrics.Scan`/`ScanTiles` driver seam
// (`scene_scan.rs`): the 169x124 window census, the aura frame gates, the
// banner decode convention and the RNG-free property. Written-not-run
// (code-only lane). The scan draws no randomness anywhere in the C#
// (`SceneMetrics.cs:326-448` has no `rand` call) — the witnesses pin that.

use super::*;
use tr_core::tile::FLAG_ACTIVE;

fn fixture_world() -> World {
    let mut world = World::new("w4-d129".into(), 1000, 400);
    world.world_surface = 100.0;
    world.rock_layer = 180.0;
    world
}

fn set_tile(world: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
    let tile = world.tile_mut(x, y).unwrap();
    tile.kind = kind;
    tile.frame_x = fx;
    tile.frame_y = fy;
    tile.flags |= FLAG_ACTIVE;
}

#[test]
fn w4_d129_the_scan_window_census_is_exactly_169_by_124() {
    // `SceneMetrics.cs:355-359`: ClampToWorld(CenteredRectangle(TileCenter,
    // ZoneScanSize)) — 169 wide, 124 tall, centred on the tile centre. A
    // fully-interior window counts every tile exactly once; the tile at the
    // centre and the four corners prove the half-extents (84 left / 84 right,
    // 62 up / 61 down — CenteredRectangle subtracts `size / 2` integer
    // division on the leading edge, `Utils.cs:952-955`).
    let mut world = fixture_world();
    let (cx, cy) = (500, 200);
    let mut placed = 0u32;
    for x in (cx - 84)..=(cx + 84) {
        for y in (cy - 62)..=(cy + 61) {
            set_tile(&mut world, x, y, 1, 0, 0);
            placed += 1;
        }
    }
    assert_eq!(placed, 169 * 124);
    // One tile beyond each edge stays uncounted.
    for (x, y) in [(cx - 85, cy), (cx + 85, cy), (cx, cy - 63), (cx, cy + 62)] {
        set_tile(&mut world, x, y, 2, 0, 0);
    }
    let m = scan_scene_at(&world, [cx as f32 * 16.0, cy as f32 * 16.0], false, false, false);
    assert_eq!(m.tile_counts[1], 169 * 124);
    assert_eq!(m.tile_counts[2], 0, "the window is exactly 169x124");
    let total: u32 = m.tile_counts.iter().sum();
    assert_eq!(total, 169 * 124, "no tile counted twice");
}

#[test]
fn w4_d129_the_scan_is_rng_free_and_repeatable() {
    // The C# scan chain (`Scan` `:326-351`, `ScanTiles` `:353-448`,
    // `AggregateTileCounts` `:579-659`, `CalculateZones` `:661-725`) contains
    // no randomness — two scans of the same world are field-identical, and
    // the scan leaves the wind stream untouched (the only RNG the weather
    // tick owns). `SceneMetrics` derives PartialEq — the whole-struct compare
    // is the census.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 215, 0, 0); // a lit campfire
    set_tile(&mut world, 501, 195, 49, 0, 0); // a water candle
    set_tile(&mut world, 502, 195, 27, 0, 0); // a sunflower
    let center = [500.0 * 16.0, 200.0 * 16.0];
    let a = scan_scene_at(&world, center, true, false, false);
    let b = scan_scene_at(&world, center, true, false, false);
    assert_eq!(a, b, "the scan is deterministic and RNG-free");
    // The wind walk's stream is not consumed by the scan: a Wind built from
    // the same seed keeps its counter/target pair across scans.
    let wind = tr_core::wind::Wind::new(7);
    let before = (wind.wind_counter(), wind.speed_target.to_bits());
    let _ = scan_scene_at(&world, center, false, false, false);
    let _ = scan_scene_at(&world, center, true, true, false);
    assert_eq!(
        (wind.wind_counter(), wind.speed_target.to_bits()),
        before,
        "the scan never touches the wind stream"
    );
}

#[test]
fn w4_d129_the_aura_frame_gate_edges_match_scan_tiles() {
    // `SceneMetrics.cs:378-409`: the exact boundary frames. Campfire 215
    // frameY < 36 (35 lit, 36 unlit); the candles frameX < 18 (17 counts,
    // 18 does not); the lantern/star ranges are inclusive both ends.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 215, 0, 35);
    set_tile(&mut world, 501, 195, 49, 17, 0);
    set_tile(&mut world, 502, 195, 372, 17, 0);
    set_tile(&mut world, 503, 195, 646, 17, 0);
    set_tile(&mut world, 504, 195, 42, 0, 324); // heart lantern low edge
    set_tile(&mut world, 505, 195, 42, 0, 286); // star in bottle high edge
    let m = scan_scene_at(&world, [500.0 * 16.0, 200.0 * 16.0], false, false, false);
    assert!(m.has_campfire);
    assert_eq!(m.water_candle_count, 1);
    assert_eq!(m.peace_candle_count, 1);
    assert_eq!(m.shadow_candle_count, 1);
    assert!(m.has_heart_lantern);
    assert!(m.has_star_in_bottle);
    // The just-outside frames refuse every gate.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 215, 0, 36);
    set_tile(&mut world, 501, 195, 49, 18, 0);
    set_tile(&mut world, 502, 195, 42, 0, 323);
    set_tile(&mut world, 503, 195, 42, 0, 287);
    set_tile(&mut world, 504, 195, 42, 0, 359);
    let m = scan_scene_at(&world, [500.0 * 16.0, 200.0 * 16.0], false, false, false);
    assert!(!m.has_campfire);
    assert_eq!(m.water_candle_count, 0);
    assert!(!m.has_heart_lantern);
    assert!(!m.has_star_in_bottle);
}

#[test]
fn w4_d129_the_banner_decode_convention_pin() {
    // CENSUS FINDING (verify-only, machine unchanged): the C# tile-91 arm
    // (`SceneMetrics.cs:410-443`) computes TWO indices — `num` (frameX/18 +
    // 111/row) gates the blank cells, then `num3 = frameX/18 - 21` (+111/row,
    // -1 per crossed 311/312) indexes `NPCBannerBuff[num3]`. The C# placement
    // style is `22 + item - 1615` (`Item.cs:19230`) = banner + 21 (the
    // `BannerToItem` base band `1615 + banner - 1`, `BannerSystem.cs:400`),
    // so the -21 recovers the `NPCtoBanner` index (Blue Slime: item 1683 →
    // style 90 → frameX 1620 → num3 = 69 = `NPCtoBanner(1)`).
    // The port's `banner_num_from_frame` (`bestiary.rs:1100-1118`) returns
    // the `num` family WITHOUT the -21 shift, and the scan indexes
    // `npc_banner_buff[decode]` (`scene_scan.rs:195-204`) — so the port's
    // array lives in the PLACE-STYLE space (index = banner + 21), offset +21
    // from the C# `NPCBannerBuff` index. The port's own placement
    // (`dungeon_features.rs:1209-1216`, frameX = style*18) and item table
    // (`itemdata.rs` Slime Banner 1683 place_style 90) share that space, but
    // the buff reader `banner_buff_for` (`banners.rs:74-88`) and the hover
    // list (`resources.rs:477-489`) look up `npc_to_banner` indices — a +21
    // mismatch. This witness pins the port's convention AS-IS; the C#-index
    // reconciliation is a follow-up (machine changes are out of this
    // census lane's scope).
    // Slime banner: style 90 → frameX 1620, frameY 0 → decode 90.
    assert_eq!(
        tr_core::bestiary::banner_num_from_frame(1620, 0),
        Some(90)
    );
    // The blank cells refuse: frameX < 396 && frameY < 54.
    assert_eq!(tr_core::bestiary::banner_num_from_frame(378, 0), None);
    assert_eq!(tr_core::bestiary::banner_num_from_frame(0, 0), None);
    // Row wrap: +111 per full frameY row of 54.
    assert_eq!(
        tr_core::bestiary::banner_num_from_frame(0, 54),
        Some(111)
    );
    // The scan arm raises the buff at the decoded index.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 91, 1620, 0);
    let m = scan_scene_at(&world, [500.0 * 16.0, 200.0 * 16.0], false, false, false);
    assert!(m.has_banner);
    assert!(m.npc_banner_buff[90]);
    assert!(!m.npc_banner_buff[69], "the port indexes the unshifted decode");
}

#[test]
fn w4_d129_the_dungeon_zone_gate_reads_count_depth_and_wall() {
    // `SceneMetrics.cs:685`: ZoneDungeon = EnoughTilesForDungeon (250) &&
    // TileCenter.Y > worldSurface && wallDungeon[centre wall]. The driver
    // (`scan_scene_full`) owns the gate; the wall set is 7/8/9/94..=99
    // (`Main.cs:10596-10607`).
    let mut world = fixture_world();
    // 250 dungeon bricks (41) at x 450..474, y 100..109 — inside BOTH the
    // row-150 window (y 88..212) and the row-50 window (y 0..112), so the
    // count gate passes in both and only the depth gate differs.
    for i in 0..250 {
        set_tile(&mut world, 450 + (i % 25), 100 + (i / 25), 41, 0, 0);
    }
    let center = [500.0 * 16.0, 150.0 * 16.0];
    // No dungeon wall at the centre: the zone stays off.
    let m = scan_scene_at(&world, center, false, false, false);
    assert_eq!(m.dungeon_tile_count, 250);
    assert!(!m.zone_dungeon);
    // Wall 7 arms it; a house wall (60) does not.
    world.tile_mut(500, 150).unwrap().wall = 7;
    let m = scan_scene_at(&world, center, false, false, false);
    assert!(m.zone_dungeon);
    world.tile_mut(500, 150).unwrap().wall = 60;
    let m = scan_scene_at(&world, center, false, false, false);
    assert!(!m.zone_dungeon);
    // Above the surface the depth gate refuses even with the wall + count.
    world.tile_mut(500, 150).unwrap().wall = 7;
    world.tile_mut(500, 50).unwrap().wall = 7;
    let m = scan_scene_at(&world, [500.0 * 16.0, 50.0 * 16.0], false, false, false);
    assert_eq!(m.dungeon_tile_count, 250);
    assert!(!m.zone_dungeon);
}
// W4-d129 census — the `SceneMetrics.Scan`/`ScanTiles` driver seam
// (`scene_scan.rs`): the 169x124 window census, the aura frame gates, the
// banner decode convention and the RNG-free property. Written-not-run
// (code-only lane). The scan draws no randomness anywhere in the C#
// (`SceneMetrics.cs:326-448` has no `rand` call) — the witnesses pin that.
// The banner convention pin was RECONCILED by W4-f31 (the decode now
// returns the C# `num3` / `NPCtoBanner` index); the file rides the f31
// branch ahead of d129's own merge — d129 keeps the identical updated pin.

use super::*;
use tr_core::tile::FLAG_ACTIVE;

fn fixture_world() -> World {
    let mut world = World::new("w4-d129".into(), 1000, 400);
    world.world_surface = 100.0;
    world.rock_layer = 180.0;
    world
}

fn set_tile(world: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
    let tile = world.tile_mut(x, y).unwrap();
    tile.kind = kind;
    tile.frame_x = fx;
    tile.frame_y = fy;
    tile.flags |= FLAG_ACTIVE;
}

#[test]
fn w4_d129_the_scan_window_census_is_exactly_169_by_124() {
    // `SceneMetrics.cs:355-359`: ClampToWorld(CenteredRectangle(TileCenter,
    // ZoneScanSize)) — 169 wide, 124 tall, centred on the tile centre. A
    // fully-interior window counts every tile exactly once; the tile at the
    // centre and the four corners prove the half-extents (84 left / 84 right,
    // 62 up / 61 down — CenteredRectangle subtracts `size / 2` integer
    // division on the leading edge, `Utils.cs:952-955`).
    let mut world = fixture_world();
    let (cx, cy) = (500, 200);
    let mut placed = 0u32;
    for x in (cx - 84)..=(cx + 84) {
        for y in (cy - 62)..=(cy + 61) {
            set_tile(&mut world, x, y, 1, 0, 0);
            placed += 1;
        }
    }
    assert_eq!(placed, 169 * 124);
    // One tile beyond each edge stays uncounted.
    for (x, y) in [(cx - 85, cy), (cx + 85, cy), (cx, cy - 63), (cx, cy + 62)] {
        set_tile(&mut world, x, y, 2, 0, 0);
    }
    let m = scan_scene_at(&world, [cx as f32 * 16.0, cy as f32 * 16.0], false, false, false);
    assert_eq!(m.tile_counts[1], 169 * 124);
    assert_eq!(m.tile_counts[2], 0, "the window is exactly 169x124");
    let total: u32 = m.tile_counts.iter().sum();
    assert_eq!(total, 169 * 124, "no tile counted twice");
}

#[test]
fn w4_d129_the_scan_is_rng_free_and_repeatable() {
    // The C# scan chain (`Scan` `:326-351`, `ScanTiles` `:353-448`,
    // `AggregateTileCounts` `:579-659`, `CalculateZones` `:661-725`) contains
    // no randomness — two scans of the same world are field-identical, and
    // the scan leaves the wind stream untouched (the only RNG the weather
    // tick owns). `SceneMetrics` derives PartialEq — the whole-struct compare
    // is the census.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 215, 0, 0); // a lit campfire
    set_tile(&mut world, 501, 195, 49, 0, 0); // a water candle
    set_tile(&mut world, 502, 195, 27, 0, 0); // a sunflower
    let center = [500.0 * 16.0, 200.0 * 16.0];
    let a = scan_scene_at(&world, center, true, false, false);
    let b = scan_scene_at(&world, center, true, false, false);
    assert_eq!(a, b, "the scan is deterministic and RNG-free");
    // The wind walk's stream is not consumed by the scan: a Wind built from
    // the same seed keeps its counter/target pair across scans.
    let wind = tr_core::wind::Wind::new(7);
    let before = (wind.wind_counter(), wind.speed_target.to_bits());
    let _ = scan_scene_at(&world, center, false, false, false);
    let _ = scan_scene_at(&world, center, true, true, false);
    assert_eq!(
        (wind.wind_counter(), wind.speed_target.to_bits()),
        before,
        "the scan never touches the wind stream"
    );
}

#[test]
fn w4_d129_the_aura_frame_gate_edges_match_scan_tiles() {
    // `SceneMetrics.cs:378-409`: the exact boundary frames. Campfire 215
    // frameY < 36 (35 lit, 36 unlit); the candles frameX < 18 (17 counts,
    // 18 does not); the lantern/star ranges are inclusive both ends.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 215, 0, 35);
    set_tile(&mut world, 501, 195, 49, 17, 0);
    set_tile(&mut world, 502, 195, 372, 17, 0);
    set_tile(&mut world, 503, 195, 646, 17, 0);
    set_tile(&mut world, 504, 195, 42, 0, 324); // heart lantern low edge
    set_tile(&mut world, 505, 195, 42, 0, 286); // star in bottle high edge
    let m = scan_scene_at(&world, [500.0 * 16.0, 200.0 * 16.0], false, false, false);
    assert!(m.has_campfire);
    assert_eq!(m.water_candle_count, 1);
    assert_eq!(m.peace_candle_count, 1);
    assert_eq!(m.shadow_candle_count, 1);
    assert!(m.has_heart_lantern);
    assert!(m.has_star_in_bottle);
    // The just-outside frames refuse every gate.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 215, 0, 36);
    set_tile(&mut world, 501, 195, 49, 18, 0);
    set_tile(&mut world, 502, 195, 42, 0, 323);
    set_tile(&mut world, 503, 195, 42, 0, 287);
    set_tile(&mut world, 504, 195, 42, 0, 359);
    let m = scan_scene_at(&world, [500.0 * 16.0, 200.0 * 16.0], false, false, false);
    assert!(!m.has_campfire);
    assert_eq!(m.water_candle_count, 0);
    assert!(!m.has_heart_lantern);
    assert!(!m.has_star_in_bottle);
}

#[test]
fn w4_d129_the_banner_decode_convention_pin() {
    // RECONCILED (W4-f31, the d129 escalation fix): the C# tile-91 arm
    // (`SceneMetrics.cs:410-443`) computes TWO indices — `num` (frameX/18 +
    // 111/row) gates the blank cells, then `num3 = frameX/18 - 21` (+111/row,
    // -1 per crossed 311/312) indexes `NPCBannerBuff[num3]`. The C# placement
    // style is `22 + item - 1615` (`Item.cs:19230`) = banner + 21 (the
    // `BannerToItem` base band `1615 + banner - 1`, `BannerSystem.cs:400`),
    // so the -21 recovers the `NPCtoBanner` index (Blue Slime: item 1683 →
    // style 90 → frameX 1620 → num3 = 69 = `NPCtoBanner(1)`).
    // The census found the port's `banner_num_from_frame` returning the
    // `num` family WITHOUT the -21 shift — the array lived in place-style
    // space, +21 off the buff reader `banner_buff_for` (`banners.rs`) and
    // the hover list (`resources.rs::banner_hover_names`), which read
    // `npc_to_banner` indices. W4-f31 ported the C# `num3` expression
    // verbatim (`bestiary.rs::banner_num_from_frame`): the decode now
    // returns the `NPCtoBanner` index and the scan, the buff reader and the
    // hover list all share the C# convention. This pin now witnesses the
    // RECONCILED space (the as-is place-style pin is retired).
    // Slime banner: style 90 → frameX 1620, frameY 0 → decode 69.
    assert_eq!(
        tr_core::bestiary::banner_num_from_frame(1620, 0),
        Some(69)
    );
    assert_eq!(
        tr_core::bestiary::banner_num_from_frame(1620, 0),
        Some(tr_core::bestiary::npc_to_banner(1))
    );
    // The blank cells refuse: frameX < 396 && frameY < 54.
    assert_eq!(tr_core::bestiary::banner_num_from_frame(378, 0), None);
    assert_eq!(tr_core::bestiary::banner_num_from_frame(0, 0), None);
    // Row wrap: +111 per full frameY row of 54 — frameX 0, frameY 54 is
    // num 111, num3 = 111 - 21 = 90 (the C# expression, no hole crossed).
    assert_eq!(
        tr_core::bestiary::banner_num_from_frame(0, 54),
        Some(90)
    );
    // The scan arm raises the buff at the NPCtoBanner index.
    let mut world = fixture_world();
    set_tile(&mut world, 500, 195, 91, 1620, 0);
    let m = scan_scene_at(&world, [500.0 * 16.0, 200.0 * 16.0], false, false, false);
    assert!(m.has_banner);
    assert!(m.npc_banner_buff[69], "the C# num3 index");
    assert!(!m.npc_banner_buff[90], "the place-style index stays cold");
}

#[test]
fn w4_d129_the_dungeon_zone_gate_reads_count_depth_and_wall() {
    // `SceneMetrics.cs:685`: ZoneDungeon = EnoughTilesForDungeon (250) &&
    // TileCenter.Y > worldSurface && wallDungeon[centre wall]. The driver
    // (`scan_scene_full`) owns the gate; the wall set is 7/8/9/94..=99
    // (`Main.cs:10596-10607`).
    let mut world = fixture_world();
    // 250 dungeon bricks (41) at x 450..474, y 100..109 — inside BOTH the
    // row-150 window (y 88..212) and the row-50 window (y 0..112), so the
    // count gate passes in both and only the depth gate differs.
    for i in 0..250 {
        set_tile(&mut world, 450 + (i % 25), 100 + (i / 25), 41, 0, 0);
    }
    let center = [500.0 * 16.0, 150.0 * 16.0];
    // No dungeon wall at the centre: the zone stays off.
    let m = scan_scene_at(&world, center, false, false, false);
    assert_eq!(m.dungeon_tile_count, 250);
    assert!(!m.zone_dungeon);
    // Wall 7 arms it; a house wall (60) does not.
    world.tile_mut(500, 150).unwrap().wall = 7;
    let m = scan_scene_at(&world, center, false, false, false);
    assert!(m.zone_dungeon);
    world.tile_mut(500, 150).unwrap().wall = 60;
    let m = scan_scene_at(&world, center, false, false, false);
    assert!(!m.zone_dungeon);
    // Above the surface the depth gate refuses even with the wall + count.
    world.tile_mut(500, 150).unwrap().wall = 7;
    world.tile_mut(500, 50).unwrap().wall = 7;
    let m = scan_scene_at(&world, [500.0 * 16.0, 50.0 * 16.0], false, false, false);
    assert_eq!(m.dungeon_tile_count, 250);
    assert!(!m.zone_dungeon);
}
