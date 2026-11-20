// W4-d129 census — `SceneMetrics.AggregateTileCounts` (`SceneMetrics.cs:579-659`)
// and the `EnoughTilesFor*` threshold gates (`:252-272`, statics `:921-941`).
// Written-not-run (code-only lane): pins the ported seam's constants, the
// count arms and the cross-subtraction order against the C#. RNG-free by
// construction — the aggregate is pure tile-count arithmetic.

use super::*;

#[test]
fn w4_d129_the_threshold_statics_match_the_scene_metrics_ctor() {
    // `SceneMetrics`'s static ctor (`SceneMetrics.cs:921-941`): the zone
    // thresholds the `EnoughTilesFor*` getters compare against.
    assert_eq!(thresholds::SHIMMER_TILE, 300); // `:921`
    assert_eq!(thresholds::CORRUPTION_TILE, 300); // `:922`
    assert_eq!(thresholds::CRIMSON_TILE, 300); // `:924`
    assert_eq!(thresholds::HALLOW_TILE, 125); // `:926`
    assert_eq!(thresholds::JUNGLE_TILE, 140); // `:928`
    assert_eq!(thresholds::SNOW_TILE_NORMAL, 1500); // `:930`
    assert_eq!(thresholds::DESERT_TILE_NORMAL, 1500); // `:933`
    assert_eq!(thresholds::MUSHROOM_TILE, 100); // `:935`
    assert_eq!(thresholds::METEOR_TILE, 75); // `:937`
    assert_eq!(thresholds::DUNGEON_TILE, 250); // `:938`
    assert_eq!(thresholds::GRAVEYARD_TILE_MAX, 36); // `:939`
    assert_eq!(thresholds::GRAVEYARD_TILE_MIN, 16); // `:940`
    assert_eq!(thresholds::GRAVEYARD_TILE, 28); // `:941`
    // The scan window statics (`:917-919`): 1920/16 + 25*2 - 1 = 169,
    // 1200/16 + 25*2 - 1 = 124.
    assert_eq!(ZONE_SCAN_SIZE, (169, 124));
}

#[test]
fn w4_d129_the_holy_and_snow_tile_id_sets_match_the_csharp_sums() {
    // `SceneMetrics.cs:596`: HolyTileCount = 109+492+110+113+117+116+164+403+402.
    // `:597`: SnowTileCount = 147+148+161+162+164+163+200.
    let mut m = SceneMetrics::new();
    for id in [109u16, 492, 110, 113, 117, 116, 164, 403, 402] {
        m.tile_counts[id as usize] = 1;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.holy_tile_count, 9);
    // 164 and 403/402 double-count into other families; only the holy sum here.
    let mut m = SceneMetrics::new();
    for id in [147u16, 148, 161, 162, 164, 163, 200] {
        m.tile_counts[id as usize] = 1;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.snow_tile_count, 7);
}

#[test]
fn w4_d129_the_jungle_evil_blood_id_sets_match_the_non_remix_branch() {
    // `SceneMetrics.cs:606-608` (the non-remix branch; the remix arms `:600-602`
    // add 474/195 and drop 226 — a seed the port does not generate).
    let mut m = SceneMetrics::new();
    for id in [60u16, 61, 62, 74, 226, 225] {
        m.tile_counts[id as usize] = 1;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.jungle_tile_count, 6);
    let mut m = SceneMetrics::new();
    for id in [23u16, 661, 24, 25, 32, 112, 163, 400, 398] {
        m.tile_counts[id as usize] = 1;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.evil_tile_count, 9);
    let mut m = SceneMetrics::new();
    for id in [199u16, 662, 201, 203, 200, 401, 399, 234, 352] {
        m.tile_counts[id as usize] = 1;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.blood_tile_count, 9);
}

#[test]
fn w4_d129_the_sand_mushroom_meteor_dungeon_honey_sets_match() {
    // `SceneMetrics.cs:613`: the 12-id sand family.
    let mut m = SceneMetrics::new();
    for id in [53u16, 112, 116, 234, 397, 398, 402, 399, 396, 400, 403, 401] {
        m.tile_counts[id as usize] = 1;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.sand_tile_count, 12);
    // `:610`: the glowing-mushroom blocks 70/71/72/528.
    let mut m = SceneMetrics::new();
    for id in [70u16, 71, 72, 528] {
        m.tile_counts[id as usize] = 1;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.mushroom_tile_count, 4);
    // `:611-612`: meteor = 37; dungeon = 41/43/44/481/482/483.
    let mut m = SceneMetrics::new();
    m.tile_counts[37] = 75;
    for id in [41u16, 43, 44, 481, 482, 483] {
        m.tile_counts[id as usize] = 42;
    }
    m.aggregate_tile_counts();
    assert_eq!(m.meteor_tile_count, 75);
    assert_eq!(m.dungeon_tile_count, 252);
    // `:595`: HoneyBlockCount = _tileCounts[229].
    let mut m = SceneMetrics::new();
    m.tile_counts[229] = 5;
    m.aggregate_tile_counts();
    assert_eq!(m.honey_block_count, 5);
}

#[test]
fn w4_d129_the_sunflower_suppression_is_minus_ten_per_flower() {
    // `SceneMetrics.cs:581`: `num = -10` (the infectedSeed ×3 arm `:582-584`
    // is a seed the port does not generate). Each sunflower subtracts 10 from
    // BOTH the evil and the blood sums (`:607-608`).
    let mut m = SceneMetrics::new();
    m.tile_counts[23] = 320; // 320 corruption tiles
    m.tile_counts[27] = 3; // 3 sunflowers = -30
    m.aggregate_tile_counts();
    assert_eq!(m.evil_tile_count, 290);
    assert!(!m.enough_tiles_for_corruption(), "290 < 300");
    // Without the flowers the same block crosses the 300 gate.
    let mut m = SceneMetrics::new();
    m.tile_counts[23] = 320;
    m.aggregate_tile_counts();
    assert_eq!(m.evil_tile_count, 320);
    assert!(m.enough_tiles_for_corruption());
}

#[test]
fn w4_d129_the_cross_subtraction_uses_the_pre_subtraction_holy_count() {
    // `SceneMetrics.cs:641-657`: holyBefore is snapshotted BEFORE the holy
    // count loses evil+blood; evil and blood each lose the ORIGINAL holy.
    let mut m = SceneMetrics::new();
    m.tile_counts[109] = 200; // holy 200
    m.tile_counts[23] = 150; // evil 150
    m.tile_counts[199] = 100; // blood 100
    m.aggregate_tile_counts();
    // holy = 200 - 150 - 100 = -50 → 0; evil = 150 - 200 → 0; blood likewise.
    assert_eq!(m.holy_tile_count, 0);
    assert_eq!(m.evil_tile_count, 0);
    assert_eq!(m.blood_tile_count, 0);
    // The order witness: evil 300, blood 50, holy 125 →
    // holy = 125-300-50 → 0; evil = 300-125 = 175; blood = 50-125 → 0.
    let mut m = SceneMetrics::new();
    m.tile_counts[109] = 125;
    m.tile_counts[23] = 300;
    m.tile_counts[199] = 50;
    m.aggregate_tile_counts();
    assert_eq!(m.holy_tile_count, 0);
    assert_eq!(m.evil_tile_count, 175);
    assert_eq!(m.blood_tile_count, 0);
}

#[test]
fn w4_d129_the_graveyard_cancel_and_the_desert_floor_match() {
    // `SceneMetrics.cs:615-628`: tombstones minus half the sunflowers
    // (integer division), the >16 cancel of HasSunflower, the zero floor.
    let mut m = SceneMetrics::new();
    m.tile_counts[85] = 30;
    m.tile_counts[27] = 3; // 30 - 1 = 29 > 16 → the sunflower cancels
    m.aggregate_tile_counts();
    assert_eq!(m.graveyard_tile_count, 29);
    assert!(!m.has_sunflower);
    assert!(m.enough_tiles_for_graveyard(), "29 >= 28");
    // `:658`: DesertSandTileCount = max(0, Sand - OceanSand).
    let mut m = SceneMetrics::new();
    m.tile_counts[53] = 100;
    m.ocean_sand_tile_count = 150;
    m.aggregate_tile_counts();
    assert_eq!(m.desert_sand_tile_count, 0);
    assert!(!m.enough_tiles_for_desert());
}
