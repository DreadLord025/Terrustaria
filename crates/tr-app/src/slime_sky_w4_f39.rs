// W4-f39 closeout census — `SlimeSky` (`GameContent/Skies/SlimeSky.cs`):
// the generate/reset draw sequences, the 24-wrap frame counter, the
// surface-line reset/deactivate ladder, the depth-band index walk and the
// byte-truncated tint. Written-not-run (code-only lane). Every RNG pin
// mirrors the draw sequence on a cloned UnifiedRandom and compares the
// full stream state.

use super::*;

/// A 4200x1200 world's inputs: surface 300 rows, the view at the surface
/// (`screenPosition.Y = 4800 - 360`), a 1280x720 backbuffer.
fn ctx() -> SlimeSkyCtx {
    SlimeSkyCtx {
        screen_position_y: 4440.0,
        screen_height: 720.0,
        max_tiles_x: 4200,
        max_tiles_y: 1200,
        world_surface: 300.0,
    }
}

const STEP: f32 = 1.0 / 60.0;

#[test]
fn w4_f39_generate_scatters_max_tiles_y_over_6_slimes_in_the_exact_draw_order() {
    // `GenerateSlimes` (`:87-112`): 1200/6 = 200 slots. Per slime:
    // Next(0, 4200) X, Next(minValue, num) Y, NextDouble speed, Next(2)
    // texture, Next(60) rare roll, then the conditional pair. The twin
    // replay pins the whole stream; the first slime's fields pin the
    // band arithmetic: num = (int)(4440*0.7 - 720) = 2388,
    // minValue = (int)(2388 - 300*16) = -2412.
    let mut sky = SlimeSky::new(7);
    let c = ctx();
    // One update tick: generate + one fall step (no slime reaches the
    // 4800 line from y <= 2388 at <= 9 px/tick, so the stream is exactly
    // the generate's).
    sky.update(STEP, true, 1, &c);
    assert!(sky.is_active());
    assert_eq!(sky.slimes.len(), 200);
    assert_eq!(sky.remaining, 200);
    let mut twin = UnifiedRandom::new((7u64 ^ 0x5117_5EED) as i32);
    let num = (4440.0f64 * 0.7 - 720.0) as i32;
    assert_eq!(num, 2388);
    let min_value = (num as f64 - 300.0 * 16.0) as i32;
    assert_eq!(min_value, -2412);
    let mut first: Option<(i32, i32, f32, u32)> = None;
    for i in 0..200 {
        let x = twin.next_range(0, 4200) * 16;
        let y = twin.next_range(min_value, num);
        let mut speed = 5.0 + 3.0 * twin.next_double() as f32;
        let depth = i as f32 / 200.0 * 1.75 + 1.6;
        let mut texture = twin.next_range(0, 2) as u32;
        let mut want_depth = depth;
        if twin.next_range(0, 60) == 0 {
            texture = 3;
            speed = 6.0 + 3.0 * twin.next_double() as f32;
            want_depth += 0.5;
        } else if twin.next_range(0, 30) == 0 {
            texture = 2;
            speed = 6.0 + 2.0 * twin.next_double() as f32;
        }
        let s = &sky.slimes[i];
        assert_eq!(s.position.x, x as f32, "slime {i} x");
        assert_eq!(s.position.y, y as f32, "slime {i} y");
        assert_eq!(s.speed, speed, "slime {i} speed");
        assert_eq!(s.texture, texture, "slime {i} texture");
        assert_eq!(s.depth, want_depth, "slime {i} depth");
        assert!(s.active);
        if i == 0 {
            first = Some((x, y, speed, texture));
        }
    }
    assert!(first.is_some());
    assert_eq!(sky.rng, twin, "the generate stream matches draw-for-draw");
}

#[test]
fn w4_f39_the_frame_counter_wraps_at_24_and_the_source_row_is_frame_over_6() {
    // The `Frame` setter (`:55`): `value % 24`; `GetSourceRectangle`
    // (`:61`): row `_frame / 6` of 4. 24 ticks of falling (the slimes
    // spawn above the surface line and fall < 8 px/tick, none resets in
    // 24 ticks from y <= 2388 vs the 4800 line) wrap every counter to 0.
    let mut sky = SlimeSky::new(7);
    let c = ctx();
    sky.update(STEP, true, 1, &c);
    for _ in 0..23 {
        sky.update(STEP, true, 1, &c);
    }
    assert!(sky.slimes.iter().all(|s| s.frame == 23));
    assert!(sky.slimes.iter().all(|s| s.frame_row() == 3));
    sky.update(STEP, true, 1, &c);
    assert!(sky.slimes.iter().all(|s| s.frame == 0), "the %24 wrap");
    assert!(sky.slimes.iter().all(|s| s.frame_row() == 0));
}

#[test]
fn w4_f39_a_slime_past_the_surface_line_resets_to_minus_100_in_the_update_draw_order() {
    // `Update` (`:126-149`): frame++, Y += speed, then past
    // `worldSurface*16` the reset — depth re-laddered, X = Next(0,
    // maxTilesX)*16, Y = -100, texture Next(2) BEFORE speed NextDouble
    // (generate draws speed first — the orders differ), Next(60), the
    // conditional pair. One slime parked just above the line crosses in
    // one tick; the twin replay pins the reset's draws.
    let mut sky = SlimeSky::new(11);
    let c = ctx();
    sky.update(STEP, true, 1, &c);
    let pristine = sky.rng.clone();
    sky.slimes[3].position.y = 4800.0 - 1.0; // one fall crosses 4800
    sky.slimes[3].speed = 8.0; // >= the 5..9 roll band: the cross is certain
    let old_frame = sky.slimes[3].frame;
    sky.update(STEP, true, 1, &c);
    let s = &sky.slimes[3];
    assert_eq!(s.frame, (old_frame + 1) % 24);
    let mut twin = pristine;
    let x = twin.next_range(0, 4200) * 16;
    let mut texture = twin.next_range(0, 2) as u32;
    let mut speed = 5.0 + 3.0 * twin.next_double() as f32;
    let mut depth = 3.0 / 200.0 * 1.75 + 1.6;
    if twin.next_range(0, 60) == 0 {
        texture = 3;
        speed = 6.0 + 3.0 * twin.next_double() as f32;
        depth += 0.5;
    } else if twin.next_range(0, 30) == 0 {
        texture = 2;
        speed = 6.0 + 2.0 * twin.next_double() as f32;
    }
    assert_eq!(s.position, Vec2::new(x as f32, -100.0));
    assert_eq!(s.texture, texture);
    assert_eq!(s.speed, speed);
    assert_eq!(s.depth, depth);
    assert_eq!(sky.rng, twin, "only the crossed slime drew");
    assert_eq!(sky.remaining, 200, "a reset keeps the count");
}

#[test]
fn w4_f39_deactivate_drains_the_slimes_and_the_empty_sky_deactivates() {
    // `Deactivate` (`:236-239`) sets `_isLeaving`; every slime past the
    // line goes inactive (`:150-154`), `_slimesRemaining--`; at 0
    // `_isActive = false` (`:156-159`). The drain draws NOTHING.
    let mut sky = SlimeSky::new(11);
    let c = ctx();
    sky.update(STEP, true, 1, &c);
    for s in &mut sky.slimes {
        s.position.y = 4800.0 - 1.0;
        s.speed = 5.0;
    }
    let pristine = sky.rng.clone();
    sky.update(STEP, false, 1, &c); // the falling edge arms _isLeaving
    // All 200 crossed in the same tick: remaining hit 0, active dropped.
    assert!(!sky.is_active());
    assert_eq!(sky.remaining, 0);
    assert!(sky.slimes.iter().all(|s| !s.active));
    assert_eq!(sky.rng, pristine, "the leaving arm draws nothing");
    // A further update is inert (the SkyManager removes the effect).
    sky.update(STEP, false, 1, &c);
    assert!(!sky.is_active());
}

#[test]
fn w4_f39_the_band_walk_excludes_the_boundary_slime() {
    // The `Draw` index walk (`:168-186`): depths ascending 1.6..3.35;
    // the (1.0, 2.5] band starts at index 0 (1.6 < 2.5) and breaks at
    // the first depth <= 1.0 — none, so num2 = the last index. But a
    // band whose min lands ON a slime's depth breaks there and the
    // `j < num2` loop EXCLUDES it. Pin with hand-built depths.
    let mut sky = SlimeSky::new(1);
    sky.slimes = (0..4)
        .map(|i| SkySlime {
            active: true,
            position: Vec2::ZERO,
            depth: 1.6 + i as f32 * 0.4, // 1.6, 2.0, 2.4, 2.8
            speed: 5.0,
            texture: 0,
            frame: 0,
        })
        .collect();
    // The walk (`:170-182`): `num` = first index with depth < max; the
    // `depth <= min` BREAK fires on the first slime at/below min — with
    // ASCENDING depths that is index 0 whenever min >= 1.6, leaving
    // num2 = 0 and the `j < num2` loop EMPTY. This is the C# quirk: the
    // slime sky only draws at boundaries BELOW its shallowest depth
    // (the 1.0 front-row close and the flush), and even there the LAST
    // (nearest) slime is excluded by `j < num2`.
    // min 2.0: index 0's 1.6 <= 2.0 breaks immediately; num = 0
    // (1.6 < 3.0), num2 = 0 → empty band.
    assert_eq!(sky.band_indices(2.0, 3.0), Some((0, 0)));
    // min 1.0: no break (all > 1.0), num2 = 3 (the last index), num = 0.
    // j = 0..3 draws indices 0,1,2 — index 3 excluded.
    assert_eq!(sky.band_indices(1.0, 3.0), Some((0, 3)));
    // max 1.5: no slime below it → num stays -1 → None (`:183`).
    assert_eq!(sky.band_indices(1.0, 1.5), None);
    // max 2.1, min 1.0: num = 0, num2 = 3 — the first-below-max only
    // moves `num`.
    assert_eq!(sky.band_indices(1.0, 2.1), Some((0, 3)));
    // min 2.5: index 0's 1.6 <= 2.5 breaks → empty band again.
    assert_eq!(sky.band_indices(2.5, 3.0), Some((0, 0)));
    // The flush shape (min = -inf, max = the last boundary 2.04): num =
    // 0 (1.6 < 2.04), no break, num2 = 3 → j = 0..3.
    assert_eq!(sky.band_indices(f32::NEG_INFINITY, 2.04), Some((0, 3)));
    // A first slime AT min with a later slime below max: depths
    // 2.0, 2.4, 2.8, 3.2, min 2.0, max 3.0 — break at index 0, num = 0,
    // num2 = 0 → empty.
    sky.slimes[0].depth = 2.0;
    sky.slimes[1].depth = 2.4;
    sky.slimes[2].depth = 2.8;
    sky.slimes[3].depth = 3.2;
    assert_eq!(sky.band_indices(2.0, 3.0), Some((0, 0)));
}

#[test]
fn w4_f39_the_tint_is_the_byte_truncated_sky_lift_times_the_depth_tier() {
    // `:193`: Color(sky*0.9 + 0.1) * 0.8 — each step truncates to a
    // byte. White sky: (1*0.9+0.1) = 1 → 255 → *0.8 = 204 → tier 1
    // (depth 1.4) * 0.8 = 163.2 → 163. Depth 2.2 → tier 0.8*0.8 = 0.64:
    // 204 * 0.64 = 130.56 → 130. Depth 3.1 → 0.6*0.8 = 0.48: 97.92 → 97.
    let white = slime_tint([1.0, 1.0, 1.0], 1.4);
    assert_eq!(white, [163.0 / 255.0; 4]);
    let mid = slime_tint([1.0, 1.0, 1.0], 2.2);
    assert_eq!(mid, [130.0 / 255.0; 4]);
    let far = slime_tint([1.0, 1.0, 1.0], 3.1);
    assert_eq!(far, [97.0 / 255.0; 4]);
    // The alpha rides the same chain (ToVector4's W = 1).
    // A half-grey sky: 0.5*0.9+0.1 = 0.55 → 140.25 → 140 → *0.8 = 112
    // → tier 0.9*0.8 = 0.72 (depth 1.6): 80.64 → 80.
    // The alpha rides the same chain (ToVector4's W = 1): 255 → 204 →
    // *0.72 = 146.88 → 146.
    let grey = slime_tint([0.5, 0.5, 0.5], 1.6);
    assert_eq!(grey[0], 80.0 / 255.0);
    assert_eq!(grey[3], 146.0 / 255.0, "alpha rides the tier chain too");
    // The tier edges are EXCLUSIVE (> 3 / > 2.5 / > 2 / > 1.5): depth
    // 3.0 is NOT > 3, so it takes the 0.7 tier: 204 * 0.56 = 114.24 → 114.
    assert_eq!(slime_tint([1.0; 3], 3.0)[0], 114.0 / 255.0);
}

#[test]
fn w4_f39_the_day_rate_loop_runs_the_update_max_one_day_rate_times() {
    // `SkyManager.Update` (`SkyManager.cs:24-31`): `num = max(1,
    // dayRate)` iterations per 60 Hz step. A slime crossing the line on
    // iteration 1 resets to Y -100; iteration 2 falls it again by its
    // NEW speed and advances the frame a second time — two frame steps
    // in one tick. Speed 8 guarantees the cross whatever generate rolled.
    let mut sky = SlimeSky::new(11);
    let c = ctx();
    sky.update(STEP, true, 1, &c);
    sky.slimes[0].position.y = 4800.0 - 1.0;
    sky.slimes[0].speed = 8.0;
    let f0 = sky.slimes[0].frame;
    sky.update(STEP, true, 2, &c);
    assert_eq!(sky.slimes[0].frame, (f0 + 2) % 24, "two iterations");
    assert_eq!(sky.slimes[0].position.y, -100.0 + sky.slimes[0].speed);
    // dayRate 0 still runs once (`num < 1 → num = 1`).
    let f1 = sky.slimes[1].frame;
    sky.update(STEP, true, 0, &c);
    assert_eq!(sky.slimes[1].frame, (f1 + 1) % 24);
}

#[test]
fn w4_f39_reset_clears_the_active_flag_and_the_next_rain_regenerates() {
    // `Reset` (`:241-244`): `_isActive = false` only. The port's edge
    // detector re-arms on the next rising `slimeRain` (the load path,
    // `WorldFile.cs:767-769`).
    let mut sky = SlimeSky::new(3);
    let c = ctx();
    sky.update(STEP, true, 1, &c);
    assert!(sky.is_active());
    sky.reset();
    assert!(!sky.is_active());
    // The flag never fell (was_up cleared) — the next update with the
    // rain still up regenerates.
    sky.update(STEP, true, 1, &c);
    assert!(sky.is_active());
    assert_eq!(sky.slimes.len(), 200);
}
