//! Biome probes and the background/water style state machines that follow the
//! camera: `Main.CalculateWaterStyle` and the `DrawBG` style/alpha transitions.

use glam::Vec2;
use tr_core::{TILE_PX, World};

/// True near either ocean edge. The generator's beach is 140 tiles wide; 200 leaves
/// room for the blend into the background.
pub(crate) fn is_ocean_biome(world: &World, camera: Vec2) -> bool {
    let x = (camera.x / TILE_PX) as i32;
    x < 200 || x > world.width - 200
}

/// Sand at the surface away from the beaches marks the inland desert.
pub(crate) fn is_desert_biome(world: &World, camera: Vec2) -> bool {
    let cam_tile = (camera.x / TILE_PX) as i32;
    if cam_tile < 220 || cam_tile > world.width - 220 {
        return false;
    }
    for dx in [-16, -8, 0, 8, 16] {
        let x = cam_tile + dx;
        let top = world.surface_of(x);
        if let Some(t) = world.tile(x, top) {
            if t.kind == 53 {
                return true;
            }
        }
    }
    false
}

pub(crate) fn is_snow_biome(world: &World, camera: Vec2) -> bool {
    let cam_tile = (camera.x / TILE_PX) as i32;
    for dx in [-16, -8, 0, 8, 16] {
        let x = cam_tile + dx;
        let top = world.surface_of(x);
        if let Some(t) = world.tile(x, top) {
            if matches!(t.kind, 147 | 161 | 162 | 163 | 164 | 200) {
                return true;
            }
        }
    }
    false
}

/// `Main.CalculateWaterStyle`: the `Misc/water_*` style index for the biome under the
/// camera. 0 is the forest default, 5 snow, 6 desert (12 underground desert), 13 ocean,
/// 7 shallow cave, 8 deep cave; lava (1), honey (11) and shimmer (14) are per-liquid
/// and never depend on the biome.
pub(crate) fn calculate_water_style(world: &World, camera: Vec2) -> usize {
    let tile_y = camera.y / TILE_PX;
    if is_desert_biome(world, camera) {
        return if tile_y > world.world_surface as f32 { 12 } else { 6 };
    }
    if is_snow_biome(world, camera) {
        return 5;
    }
    let tile_x = (camera.x / TILE_PX) as i32;
    if tile_x < 220 || tile_x > world.width - 220 {
        // `ZoneBeach` -> bgStyle 4 -> water 13.
        return 13;
    }
    if tile_y > world.rock_layer as f32 + 40.0 {
        // `ZoneGlowshroom` would pick 7 here; our generator has no glowshroom biome.
        return 8;
    }
    if tile_y > world.world_surface as f32 {
        return 7;
    }
    0
}

/// `Main.DrawBG_HandleBackgroundTransition` plus the `bgDelay` branch of
/// `Main.DrawBG`. The original's instant swap only pairs two forest variant
/// sets (`GetCurrentBackgroundOfStyle`), and all forest sets are tree set 1 in
/// our port, so it can never fire here.
///
/// Delay is 60 frames when entering a forest style, 30 otherwise; after the
/// switch `bgDelay` becomes -60 and holds a 60-frame cooldown.
pub(crate) fn advance_bg_style(style: &mut i32, delay: &mut i32, preferred: i32) {
    // `SurfaceBackgroundID.Sets.IsForest`: 0 and the four forest set styles.
    fn is_forest(style: i32) -> bool {
        matches!(style, 0 | 10 | 11 | 12)
    }
    if *delay < 0 {
        *delay += 1;
    } else if preferred != *style {
        let new_is_forest = is_forest(preferred);
        let wait = if new_is_forest { 60 } else { 30 };
        *delay += 1;
        if *delay > wait {
            *delay = -60;
            *style = preferred;
            if new_is_forest {
                *delay = 0;
            }
        }
    } else if *delay > 0 {
        *delay -= 1;
    }
}

/// `SurfaceBackgroundID.Sets.IsDesertVariant` (`ID/SurfaceBackgroundID.cs`):
/// the four styles whose per-biome desert sets can swap without a fade.
fn is_desert_variant(style: i32) -> bool {
    matches!(style, 2 | 5 | 13 | 14)
}

/// `Main.UpdateBGVisibility_FrontLayer_IsDesertVariantSwap`
/// (`Main.cs:64120-64143`): both styles must be desert variants, and both
/// must have a `Main.desertBackgroundSet` row for their biome
/// (`BackgroundVariant.HasAny`). `desert_set` is `World::desert_bg_set()`'s
/// [Pure, Corrupt, Crimson, Hallow] table.
fn is_desert_variant_swap(target: i32, original: i32, desert_set: &[[i32; 3]; 4]) -> bool {
    if target == original || !is_desert_variant(target) || !is_desert_variant(original) {
        return false;
    }
    let has_any = |style: i32| match style {
        2 => tr_core::world::desert_variant_has_any(desert_set[0]),
        5 => tr_core::world::desert_variant_has_any(desert_set[1]),
        13 => tr_core::world::desert_variant_has_any(desert_set[3]),
        14 => tr_core::world::desert_variant_has_any(desert_set[2]),
        _ => false,
    };
    has_any(target) && has_any(original)
}

/// The far-layer indexes a style targets (`Main.UpdateBGVisibility_BackLayer`,
/// `Main.cs:64145-64209`). Mushroom (9) targets *two* layers: 0 and 6.
fn far_layer_targets(style: i32, jungle_bg: i32, hallow_bg: i32) -> [Option<usize>; 2] {
    let one = |i: usize| [Some(i), None];
    match style {
        2 => one(1),
        // Jungle: `jungleBG == 5` sits behind the forest far layer.
        3 => one(if jungle_bg == 5 { 15 } else { 0 }),
        // Corrupt, and the corrupt/crimson desert-variant styles.
        1 | 5 | 13 | 14 => one(2),
        // Hallow: `hallowBG == 3` has its own far layer.
        6 => one(if hallow_bg == 3 { 6 } else { 2 }),
        4 => one(3),
        7 => one(4),
        8 => one(5),
        9 => [Some(0), Some(6)],
        10 | 11 | 12 => one(style as usize),
        _ => one(0),
    }
}

/// `Main.GetPreferredBGStyleForPlayer`'s desert branch (`Main.cs:64223-64226`):
/// `(corrupt && evil > blood && evil > holy) ? 5 : ((crimson && blood > holy)
/// ? 14 : (!hallow ? 2 : 13))`. Pure so the branch order is witnessable
/// without an `App`.
pub(crate) fn desert_preferred_style(
    zone_corrupt: bool,
    zone_crimson: bool,
    zone_hallow: bool,
    evil: i32,
    blood: i32,
    holy: i32,
) -> i32 {
    if zone_corrupt && evil > blood && evil > holy {
        5
    } else if zone_crimson && blood > holy {
        14
    } else if !zone_hallow {
        2
    } else {
        13
    }
}

/// `Main.UpdateBGVisibility_BackLayer` + `Main.UpdateBGVisibility_FrontLayer`
/// (`Main.cs:64087-64209`), the cross-fade half of `Main.DrawBG`.
///
/// The front array is indexed by *style* (so hallow 6, crimson 8, mushroom 9);
/// the far array uses the biome-layer index of `UpdateBGVisibility_BackLayer`.
/// `desert_set` feeds the desert-variant swap guard (`Main.cs:64109`,
/// `:64268`): while the incoming variant's own alpha is below 1, the outgoing
/// variant keeps its alpha (no fade-out) when both sets `HasAny`.
pub(crate) fn advance_bg_alphas(
    front: &mut [f32; 16],
    far: &mut [f32; 16],
    style: i32,
    jungle_bg: i32,
    hallow_bg: i32,
    desert_set: &[[i32; 3]; 4],
    speed: f32,
) {
    // `DrawBG_ModifyBGFarBackLayerAlpha` (`Main.cs:64258-64273`): the wanted
    // layers grow, every other non-zero layer fades out — unless the
    // desert-variant swap holds it (`:64268`). The C# passes the *far target
    // index* as `targetBiome` to the style-keyed swap predicate; for the four
    // desert-variant styles that index is 2 (Pure), so the guard fires only
    // when the outgoing far slot is itself a desert-variant style id with a
    // set — which the far indexing never produces (slots 5/13/14 stay 0).
    // The verbatim call is kept so the shape matches; it is inert by
    // construction, exactly like the original's.
    let targets = far_layer_targets(style, jungle_bg, hallow_bg);
    let far_target = targets[0].unwrap_or(0) as i32;
    for (i, a) in far.iter_mut().enumerate() {
        let flag = targets.contains(&Some(i));
        if flag
            || (*a != 0.0
                && (!is_desert_variant_swap(far_target, i as i32, desert_set)
                    || !(far[far_target as usize] < 1.0)))
        {
            *a = (*a + if flag { speed } else { -speed }).clamp(0.0, 1.0);
        }
    }
    // `UpdateBGVisibility_FrontLayer` (`Main.cs:64087-64118`): the target layer
    // fades in, the rest out — a desert-variant swap holds the outgoing layer
    // until the incoming one reaches 1 (`:64109`).
    for (i, a) in front.iter_mut().enumerate() {
        if style == i as i32 {
            *a = (*a + speed).min(1.0);
        } else if *a != 0.0
            && (!is_desert_variant_swap(style, i as i32, desert_set) || !(front[style as usize] < 1.0))
        {
            *a = (*a - speed).max(0.0);
        }
    }
}

#[cfg(test)]
mod water_style_tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn test_world() -> World {
        let mut w = World::new("t".into(), 600, 800);
        w.world_surface = 200.0;
        w.rock_layer = 360.0;
        w.column_surface = vec![200; 600];
        w
    }

    fn put(w: &mut World, x: i32, kind: u16) {
        let i = w.index(x, 200);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn cam(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 * TILE_PX, y as f32 * TILE_PX)
    }

    #[test]
    fn forest_surface_is_style_zero() {
        let w = test_world();
        assert_eq!(calculate_water_style(&w, cam(300, 150)), 0);
    }

    #[test]
    fn shallow_cave_is_seven_and_deep_cave_is_eight() {
        let w = test_world();
        // `TileCenter.Y > worldSurface` but not past rockLayer + 40 -> 7.
        assert_eq!(calculate_water_style(&w, cam(300, 250)), 7);
        // Past rockLayer + 40 -> 8.
        assert_eq!(calculate_water_style(&w, cam(300, 450)), 8);
    }

    #[test]
    fn snow_water_is_five() {
        let mut w = test_world();
        put(&mut w, 300, 147);
        assert_eq!(calculate_water_style(&w, cam(300, 150)), 5);
    }

    #[test]
    fn desert_water_is_six_and_twelve_underground() {
        let mut w = test_world();
        put(&mut w, 300, 53);
        assert_eq!(calculate_water_style(&w, cam(300, 150)), 6);
        assert_eq!(calculate_water_style(&w, cam(300, 250)), 12);
    }

    #[test]
    fn ocean_edges_use_thirteen() {
        let w = test_world();
        assert_eq!(calculate_water_style(&w, cam(100, 150)), 13);
        assert_eq!(calculate_water_style(&w, cam(550, 150)), 13);
    }
}

#[cfg(test)]
mod bg_transition_tests {
    use super::*;

    /// The style-0 desert set: only Pure has rows (`WorldGen.cs:7505-7508`),
    /// so every variant-vs-variant swap guard is false.
    const NO_DESERT: [[i32; 3]; 4] = [[21, 20, -1], [-1, -1, -1], [-1, -1, -1], [-1, -1, -1]];

    #[test]
    fn switching_into_a_non_forest_style_waits_thirty_frames() {
        // `DrawBG_HandleBackgroundTransition`: delay 30, switch when bgDelay > 30.
        let (mut style, mut delay) = (0, 0);
        for _ in 0..30 {
            advance_bg_style(&mut style, &mut delay, 7);
        }
        assert_eq!((style, delay), (0, 30));
        advance_bg_style(&mut style, &mut delay, 7);
        assert_eq!((style, delay), (7, -60));
    }

    #[test]
    fn switching_into_forest_waits_sixty_frames_and_resets_the_delay() {
        let (mut style, mut delay) = (7, 0);
        for _ in 0..60 {
            advance_bg_style(&mut style, &mut delay, 0);
        }
        assert_eq!((style, delay), (7, 60));
        advance_bg_style(&mut style, &mut delay, 0);
        assert_eq!((style, delay), (0, 0));
    }

    #[test]
    fn negative_delay_is_a_sixty_frame_cooldown() {
        let (mut style, mut delay) = (7, -60);
        for _ in 0..60 {
            advance_bg_style(&mut style, &mut delay, 7);
            assert_eq!(style, 7);
        }
        assert_eq!(delay, 0);
        // Returning to style 0 is a forest style, so it waits 60 frames.
        for _ in 0..31 {
            advance_bg_style(&mut style, &mut delay, 0);
        }
        assert_eq!((style, delay), (7, 31));
        for _ in 0..30 {
            advance_bg_style(&mut style, &mut delay, 0);
        }
        assert_eq!(style, 0);
    }

    #[test]
    fn keeping_the_style_decays_a_positive_delay() {
        let (mut style, mut delay) = (0, 12);
        advance_bg_style(&mut style, &mut delay, 0);
        assert_eq!((style, delay), (0, 11));
    }

    #[test]
    fn snow_cross_fade_moves_both_alpha_sets() {
        let mut front = [0.0; 16];
        let mut far = [0.0; 16];
        front[0] = 1.0;
        far[0] = 1.0;
        // 20 frames at `backgroundLayerTransitionSpeed = 0.05` completes a fade.
        for _ in 0..20 {
            advance_bg_alphas(&mut front, &mut far, 7, 0, 0, &NO_DESERT, 0.05);
        }
        assert_eq!(front[0], 0.0);
        assert_eq!(far[0], 0.0);
        assert_eq!(front[7], 1.0);
        // `UpdateBGVisibility_BackLayer`: snow is far index 4.
        assert_eq!(far[4], 1.0);
        // No other slot may light up.
        assert_eq!(front.iter().filter(|a| **a != 0.0).count(), 1);
        assert_eq!(far.iter().filter(|a| **a != 0.0).count(), 1);
    }

    #[test]
    fn each_supported_style_selects_its_far_layer() {
        for (style, index) in [(0, 0), (2, 1), (4, 3), (7, 4)] {
            let mut front = [0.0; 16];
            let mut far = [0.0; 16];
            advance_bg_alphas(&mut front, &mut far, style, 0, 0, &NO_DESERT, 0.05);
            assert_eq!(front[style as usize], 0.05, "front style {style}");
            assert_eq!(far[index], 0.05, "far style {style}");
        }
    }

    #[test]
    fn every_front_layer_style_is_its_own_alpha_slot() {
        // `Main.bgAlphaFrontLayer` is indexed by `Main.bgStyle` directly, so the
        // five variant styles land on their own slots (`Main.cs:64099-64117`).
        for style in [1, 3, 6, 8, 9] {
            let mut front = [0.0; 16];
            let mut far = [0.0; 16];
            advance_bg_alphas(&mut front, &mut far, style, 0, 0, &NO_DESERT, 0.05);
            assert_eq!(front[style as usize], 0.05, "front style {style}");
            assert_eq!(front.iter().filter(|a| **a != 0.0).count(), 1);
        }
    }

    #[test]
    fn far_layer_mapping_matches_update_bg_visibility_back_layer() {
        // `Main.cs:64145-64209`, with the two WorldGen-dependent branches.
        let far = |style, jungle, hallow| {
            let mut front = [0.0; 16];
            let mut far = [0.0; 16];
            advance_bg_alphas(&mut front, &mut far, style, jungle, hallow, &NO_DESERT, 0.05);
            (0..16).filter(|&i| far[i] > 0.0).collect::<Vec<_>>()
        };
        assert_eq!(far(1, 0, 0), vec![2]); // corrupt
        assert_eq!(far(3, 0, 0), vec![0]); // jungle, jungleBG != 5
        assert_eq!(far(3, 5, 0), vec![15]); // jungle, jungleBG == 5
        assert_eq!(far(6, 0, 0), vec![2]); // hallow, hallowBG != 3
        assert_eq!(far(6, 0, 3), vec![6]); // hallow, hallowBG == 3
        assert_eq!(far(5, 0, 0), vec![2]); // desert-variant corrupt
        assert_eq!(far(13, 0, 0), vec![2]); // desert-variant hallow
        assert_eq!(far(14, 0, 0), vec![2]); // desert-variant crimson
        assert_eq!(far(8, 0, 0), vec![5]); // crimson
        // Mushroom targets **two** far layers: 0 and 6.
        assert_eq!(far(9, 0, 0), vec![0, 6]);
        // Forest variants are their own far layer.
        assert_eq!(far(10, 0, 0), vec![10]);
        assert_eq!(far(11, 0, 0), vec![11]);
        assert_eq!(far(12, 0, 0), vec![12]);
    }

    #[test]
    fn w4_g71_desert_variant_swap_holds_the_outgoing_alpha_until_one() {
        // `Main.cs:64109`: with both sets populated (style 51), a 2 -> 5 swap
        // keeps front[2] at 1 while front[5] climbs; once front[5] hits 1 the
        // outgoing layer fades normally.
        let set51 = tr_core::world::desert_bg_set(51);
        let mut front = [0.0; 16];
        let mut far = [0.0; 16];
        front[2] = 1.0;
        far[1] = 1.0;
        for _ in 0..19 {
            advance_bg_alphas(&mut front, &mut far, 5, 0, 0, &set51, 0.05);
        }
        // front[5] = 0.95 < 1: the outgoing desert layer is held.
        assert!((front[5] - 0.95).abs() < 1e-5, "front5 {}", front[5]);
        assert_eq!(front[2], 1.0, "the outgoing variant is held at 1");
        advance_bg_alphas(&mut front, &mut far, 5, 0, 0, &set51, 0.05);
        assert_eq!(front[5], 1.0);
        // Now the guard's `!(front[value] < 1f)` flips: front[2] fades.
        advance_bg_alphas(&mut front, &mut far, 5, 0, 0, &set51, 0.05);
        assert!((front[2] - 0.95).abs() < 1e-5, "front2 {}", front[2]);
        // The far side: both variants target far slot 2 (the corrupt-desert
        // mountains), which stays lit through the swap.
        assert_eq!(far[2], 1.0);
    }

    #[test]
    fn w4_g71_desert_variant_swap_needs_both_sets() {
        // Style 0: only Pure has rows — a 2 -> 5 swap fades front[2] normally
        // (`HasAny(5)` is false, `Main.cs:64138-64142`).
        let mut front = [0.0; 16];
        let mut far = [0.0; 16];
        front[2] = 1.0;
        advance_bg_alphas(&mut front, &mut far, 5, 0, 0, &NO_DESERT, 0.05);
        assert!((front[2] - 0.95).abs() < 1e-5, "front2 {}", front[2]);
        assert!((front[5] - 0.05).abs() < 1e-5);
        // Non-variant pairs never hold: 2 -> 7 (snow) fades even with set 51.
        let set51 = tr_core::world::desert_bg_set(51);
        let mut front = [0.0; 16];
        let mut far = [0.0; 16];
        front[2] = 1.0;
        advance_bg_alphas(&mut front, &mut far, 7, 0, 0, &set51, 0.05);
        assert!((front[2] - 0.95).abs() < 1e-5);
        // The predicate itself (`Main.cs:64120-64143`).
        assert!(is_desert_variant_swap(5, 2, &set51));
        assert!(is_desert_variant_swap(13, 14, &set51));
        assert!(!is_desert_variant_swap(5, 5, &set51)); // same style
        assert!(!is_desert_variant_swap(5, 7, &set51)); // snow is no variant
        assert!(!is_desert_variant_swap(5, 2, &NO_DESERT)); // Corrupt empty
    }

    #[test]
    fn w4_g71_desert_preferred_style_matches_get_preferred_bg_style() {
        // `Main.cs:64225`'s ternary chain, every arm.
        // Corrupt desert with evil strictly above both blood and holy -> 5.
        assert_eq!(desert_preferred_style(true, false, false, 300, 100, 50), 5);
        // Evil tied with blood -> the corrupt arm fails (strict >).
        assert_eq!(desert_preferred_style(true, false, false, 100, 100, 50), 2);
        // Evil tied with holy -> fails too.
        assert_eq!(desert_preferred_style(true, false, false, 100, 50, 100), 2);
        // Crimson with blood > holy -> 14 (even when corrupt also present but
        // its arm failed).
        assert_eq!(desert_preferred_style(true, true, false, 50, 300, 100), 14);
        assert_eq!(desert_preferred_style(false, true, false, 0, 300, 100), 14);
        // Crimson with blood <= holy -> falls to the hallow check.
        assert_eq!(desert_preferred_style(false, true, false, 0, 100, 100), 2);
        assert_eq!(desert_preferred_style(false, true, true, 0, 100, 100), 13);
        // Pure hallow desert -> 13; nothing -> 2.
        assert_eq!(desert_preferred_style(false, false, true, 0, 0, 300), 13);
        assert_eq!(desert_preferred_style(false, false, false, 0, 0, 0), 2);
        // The corrupt arm outranks crimson when both would fire.
        assert_eq!(desert_preferred_style(true, true, true, 500, 300, 100), 5);
    }
}
