// W4-d129 census — `SceneMetrics.CalculateZones` (`SceneMetrics.cs:661-725`)
// and its wall-term half (`calculate_wall_zones`, `:686-692`). Written-not-run
// (code-only lane): the depth-band boundaries, the jungle underworld gate,
// the shimmer window + dungeon refusal, the sandstorm rain suppression and
// the wall-zone id sets. RNG-free — pure comparisons over the centre tile.

use super::*;

fn metrics_at(x: i32, y: i32) -> SceneMetrics {
    let mut m = SceneMetrics::new();
    m.tile_center = (x, y);
    m
}

#[test]
fn w4_d129_the_sky_band_boundary_is_the_float_035_product() {
    // `SceneMetrics.cs:665-666`: the band edge is
    // `worldSurface * 0.3499999940395355` (the C# float literal 0.35f widened
    // to double), NOT 0.35. On surface 100 the edge is 34.99999940395355:
    // row 34 is sky, row 35 is overworld.
    let mut m = metrics_at(100, 34);
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(m.zone_sky_height);
    assert!(!m.zone_overworld_height);
    let mut m = metrics_at(100, 35);
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(!m.zone_sky_height);
    assert!(m.zone_overworld_height);
    // `:666`: the surface row itself stays overworld (`<= worldSurface`).
    let mut m = metrics_at(100, 100);
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(m.zone_overworld_height);
    assert!(!m.below_surface);
    // `:667-670`: the dirt/rock/underworld edges are exclusive-low,
    // inclusive-high — row 200 is dirt (<= rockLayer), row 400 rock.
    let mut m = metrics_at(100, 200);
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(m.zone_dirt_layer_height);
    assert!(!m.zone_rock_layer_height);
    let mut m = metrics_at(100, 400);
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(m.zone_rock_layer_height);
    assert!(!m.zone_underworld_height);
}

#[test]
fn w4_d129_the_jungle_zone_refuses_the_underworld() {
    // `SceneMetrics.cs:674`: `EnoughTilesForJungle && !ZoneUnderworldHeight`.
    let mut m = metrics_at(100, 300);
    m.jungle_tile_count = thresholds::JUNGLE_TILE; // 140, the inclusive gate
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(m.zone_jungle);
    let mut m = metrics_at(100, 450); // under the underworld layer
    m.jungle_tile_count = thresholds::JUNGLE_TILE;
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(!m.zone_jungle, "the underworld refuses the jungle zone");
    // One tile short of the threshold stays off everywhere.
    let mut m = metrics_at(100, 300);
    m.jungle_tile_count = thresholds::JUNGLE_TILE - 1;
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(!m.zone_jungle);
}

#[test]
fn w4_d129_the_shimmer_window_and_the_dungeon_refusal() {
    // `SceneMetrics.cs:704`: `Y > worldSurface + 84 && Y < maxTilesY - 396`
    // — both edges exclusive. Surface 100, height 1200: rows 185..803.
    let mut m = metrics_at(100, 184);
    m.shimmer_tile_count = thresholds::SHIMMER_TILE;
    m.calculate_zones(100.0, 1000, 200.0, 1000, 1200, false, false);
    assert!(!m.underground_for_shimmering, "184 == surface+84 is outside");
    let mut m = metrics_at(100, 185);
    m.shimmer_tile_count = thresholds::SHIMMER_TILE;
    m.calculate_zones(100.0, 1000, 200.0, 1000, 1200, false, false);
    assert!(m.underground_for_shimmering);
    assert!(m.zone_shimmer);
    let mut m = metrics_at(100, 804);
    m.shimmer_tile_count = thresholds::SHIMMER_TILE;
    m.calculate_zones(100.0, 1000, 200.0, 1000, 1200, false, false);
    assert!(!m.underground_for_shimmering, "804 == maxTilesY-396 is outside");
    let mut m = metrics_at(100, 803);
    m.shimmer_tile_count = thresholds::SHIMMER_TILE;
    m.calculate_zones(100.0, 1000, 200.0, 1000, 1200, false, false);
    assert!(m.underground_for_shimmering);
    // `:705`: `!ZoneDungeon` — the driver-set dungeon flag refuses shimmer.
    let mut m = metrics_at(100, 185);
    m.shimmer_tile_count = thresholds::SHIMMER_TILE;
    m.zone_dungeon = true;
    m.calculate_zones(100.0, 1000, 200.0, 1000, 1200, false, false);
    assert!(!m.zone_shimmer);
}

#[test]
fn w4_d129_the_sandstorm_suppresses_the_rain_zone() {
    // `SceneMetrics.cs:698-703`: ZoneRain = raining && SurfaceAtmospherics;
    // ZoneSandstorm = desert && atmospherics && happening; a sandstorm then
    // clears ZoneRain. The surface gate is `Y <= worldSurface`
    // (`WorldGen.IsSurfaceForAtmospherics`, `WorldGen.cs:10986`, non-remix).
    let mut m = metrics_at(100, 50);
    m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, true, false);
    assert!(m.zone_rain);
    assert!(!m.zone_sandstorm);
    let mut m = metrics_at(100, 50);
    m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, true, true);
    assert!(m.zone_sandstorm);
    assert!(!m.zone_rain, "the sandstorm clears the rain zone");
    // Under the surface neither zone forms, even raining.
    let mut m = metrics_at(100, 150);
    m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, true, true);
    assert!(!m.surface_atmospherics);
    assert!(!m.zone_rain);
    assert!(!m.zone_sandstorm);
}

#[test]
fn w4_d129_the_wall_zone_id_pairs_and_the_gem_cave_range() {
    // `SceneMetrics.cs:686-690`: temple 87; granite 184|180; marble 183|178;
    // hive 108|86; gem cave 48..=53 inclusive. Both ids of each pair arm.
    let mut w = World::new("t".into(), 200, 200);
    let idx = w.index(100, 100);
    for (wall, pick) in [
        (87u16, 0),
        (184, 1),
        (180, 1),
        (183, 2),
        (178, 2),
        (108, 3),
        (86, 3),
        (48, 4),
        (53, 4),
    ] {
        w.tiles[idx].wall = wall;
        let mut m = metrics_at(100, 100);
        m.calculate_wall_zones(&w);
        let flags = [
            m.zone_lihzhard_temple,
            m.zone_granite,
            m.zone_marble,
            m.zone_hive,
            m.zone_gem_cave,
        ];
        for (i, f) in flags.iter().enumerate() {
            assert_eq!(*f, i == pick, "wall {wall} arm {i}");
        }
    }
    // The range neighbours stay off.
    for wall in [47u16, 54] {
        w.tiles[idx].wall = wall;
        let mut m = metrics_at(100, 100);
        m.calculate_wall_zones(&w);
        assert!(!m.zone_gem_cave, "wall {wall} is outside 48..=53");
    }
}

#[test]
fn w4_d129_the_underground_desert_wall_sets_and_gates() {
    // `SceneMetrics.cs:692`: desert && BelowSurface && (Sandstone set ||
    // HardenedSand set || 223) && !wallHouse. The sets are
    // `WallID.cs:42-43`: Sandstone {187,220,222,221,275,308,310,309},
    // HardenedSand {216,217,219,218,304,305,307,306}.
    let mut w = World::new("t".into(), 200, 200);
    let idx = w.index(100, 100);
    for wall in [223u16, 220, 275, 216, 304] {
        w.tiles[idx].wall = wall;
        let mut m = metrics_at(100, 100);
        m.zone_desert = true;
        m.below_surface = true;
        m.calculate_wall_zones(&w);
        assert!(m.zone_underground_desert, "wall {wall} arms the zone");
    }
    // Each gate alone refuses: no desert, above surface, a house wall (60),
    // an unrelated wall (1).
    for (desert, below, wall) in [
        (false, true, 223u16),
        (true, false, 223),
        (true, true, 60),
        (true, true, 1),
    ] {
        w.tiles[idx].wall = wall;
        let mut m = metrics_at(100, 100);
        m.zone_desert = desert;
        m.below_surface = below;
        m.calculate_wall_zones(&w);
        assert!(!m.zone_underground_desert, "({desert},{below},{wall})");
    }
}

#[test]
fn w4_d129_the_beach_zone_reads_the_ocean_bands() {
    // `SceneMetrics.cs:691` via `WorldGen.oceanDepths` (`WorldGen.cs:4447`):
    // x < 380 or x > maxTilesX - 380, above oceanLevel =
    // (worldSurface + rockLayer)/2 + 40. Surface 100, rock 200 → level 190.
    let mut m = metrics_at(379, 50);
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(m.zone_beach);
    let mut m = metrics_at(380, 50);
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(!m.zone_beach);
    let mut m = metrics_at(621, 50); // 1000 - 380 = 620, strictly greater
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(m.zone_beach);
    let mut m = metrics_at(100, 191); // below the ocean level
    m.calculate_zones(100.0, 400, 200.0, 1000, 500, false, false);
    assert!(!m.zone_beach);
}
