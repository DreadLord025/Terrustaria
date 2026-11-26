//! W4-d12 — the aiStyle-117 explosion family (`AI()`'s inline arm
//! `Projectile.cs:34459-34637`): the Solar Whip's explosion (612), the
//! Stardust Guardian's dash explosion (624), the Daybreak's explosion
//! (953) and the Mushroom Boi's explosion (1044). The Solar Counter (608)
//! shares the style's row but its spawn sites `Kill()` it immediately
//! (`Player.cs:21171`/`:38318`), so its burst rides the Kill path
//! (`:72177-72225`) — ported here as [`solar_counter_kill`].
//!
//! C# sources read WHOLE (the golden rule):
//! - the shared arm `Projectile.cs:34459-34637` (the `flag58`/`flag59`/
//!   `flag60` kind split, the scale/age/frame machine, the tick-1
//!   resized-box `Damage()`, the three dust families).
//! - the 608 Kill arm `Projectile.cs:72177-72225` (the 160×160 box
//!   `Damage()` + the dust/gore census).
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the `Dust.NewDust` BODIES and their internal rolls go with the
//! dust layer (ANALYSIS "Dust & gore", the `projectile_ai_misc`
//! precedent). The `Gore.NewGore` internal kicks DO stay — `Next(10, 31)`
//! on Y first, then `Next(-20, 21)` on X (`Gore.cs:1327-1328`, the
//! LEAD-REVIEW G3 standard). Per family:
//! - 612/953/978 (`flag58`): the four dust loops' `RotatedByRandom`
//!   `NextDouble` pairs + the `NextDouble` radius draws (`:34505-34535`).
//!   The 978 (Volcano) is named in `flag58` but its row is aiStyle 189
//!   (`:9253-9267`), so it never reaches this arm — the branch is kept
//!   verbatim for source fidelity (census F2).
//! - 624 (`flag59`): the two dust loops + the three-ring 200-particle
//!   burst's six `NextDouble` angle/radius draws, the per-particle
//!   `NextFloat` fadeIn + the `Next(4)` gate (`:34540-34627`).
//! - 1044 (`flag60`): ZERO draws — the `ParticleOrchestrator` request
//!   (`:34631-34635`) is the particle layer's documented skip.
//! - 608 (Kill): the four dust loops + the two gores' position/type/
//!   velocity draws (`:72183-72224`).
//!
//! Skips (ANALYSIS): the dust/gore bodies, `Lighting.AddLight`
//! (`:34487`), `SoundEngine.PlaySound` (the cue rides the pool's
//! `wants_sound` intent), the `GameShaders.Armor` shader writes, the
//! `hide` flag (draw-only, no port field — census F3) and the
//! `ParticleOrchestrator` request. The tick-1 `Damage()` runs through the
//! pool's `damage_pve_boxed` seam (the resized box, the 644-burst
//! precedent); the `Main.myPlayer == owner` gate folds to the
//! simulated-slot set (the pool only updates local-owned slots).
//!
//! Code-only round (W4-d12): the witness tests below are WRITTEN NOT RUN.

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::projectile::{
    Projectile, rotate, DAYBREAK_EXPLOSION, DEAD_CELLS_MUSHROOM_BOI_EXPLOSION,
    SOLAR_COUNTER, SOLAR_WHIP_SWORD_EXPLOSION, STARDUST_GUARDIAN_EXPLOSION, proj_frames,
};

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`):
/// `RotatedBy(NextDouble() * max - NextDouble() * max)` — TWO draws, left
/// operand first, the double subtraction, then the float-cos/sin spin
/// (the `bow_launch::rotated_by_random` precedent). The dust POSITION
/// feeds only the skipped body, but the two draws stay on the stream.
fn rotated_by_random_draws(rng: &mut dyn DropRng, max_radians: f64) -> f32 {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    angle as f32
}

/// `AI()`'s aiStyle-117 arm (`Projectile.cs:34459-34637`) — the shared
/// explosion machine. Returns `true` when the tick-1 resized-box
/// `Damage()` should run (the dispatch resolves it through the pool's
/// `damage_pve_boxed` seam AFTER this call, the 644-burst shape); the
/// resize itself lands here so the box is ready for the walk.
///
/// The C# order: the kind flags, the `ai[1] += 0.01` scale ramp, the
/// `ai[0]++` age, the `3 * projFrames` life cap (kill + return), the
/// 3-tick frame machine (the wrap sets `hide`, the draw-only skip), the
/// `alpha -= 63` fade, the `flag58` light (skip), the `ai[0] != 1` return,
/// then the tick-1 resize + `Damage()` + the per-family dust census.
pub(crate) fn ai_explosion(p: &mut Projectile) -> bool {
    // `:34464-34465` — the scale ramp.
    p.ai[1] += 0.01;
    p.scale = p.ai[1];
    // `:34466-34471` — the age + the `3 * projFrames` life cap. The C#
    // `Kill()` zeroes timeLeft and latches kill_time_left (the private
    // `Projectile::kill` shape, inlined — the module cannot reach it).
    p.ai[0] += 1.0;
    let frames = proj_frames(p.kind) as f32;
    if p.ai[0] >= 3.0 * frames {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return false;
    }
    // `:34472-34479` — the 3-tick frame machine; the wrap sets `hide =
    // true` (draw-only, no port field — census F3, the skip).
    p.frame_counter += 1;
    if p.frame_counter >= 3 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= proj_frames(p.kind) as i32 {
            // `hide = true` — SKIPPED (draw layer).
        }
    }
    // `:34480-34484` — the alpha fade.
    p.alpha -= 63;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:34485-34488` — the flag58 light cast: SKIPPED (no light layer).
    // `:34489-34492` — the tick-1 gate: every other tick returns here.
    if p.ai[0] != 1.0 {
        return false;
    }
    // `:34493-34500` — the tick-1 resize: the box grows to
    // `(int)(52 * scale)` recentred on the old centre. `num843` is 52 for
    // every kind (the `flag60` arm re-assigns 52, `:34494-34497`). The C#
    // does NOT restore the box afterwards (unlike the 644 burst), so the
    // resized box persists for the contact pass too.
    let num843 = 52.0f32;
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let side = (num843 * p.scale) as i32 as f32;
    p.pos = center;
    p.w = side;
    p.h = side;
    p.pos = center - Vec2::new(p.w * 0.5, p.h * 0.5);
    // `:34501` — `this.Damage()`: the dispatch runs the boxed walk AFTER
    // this call (the resized box is ready), THEN [`explosion_tick1_draws`]
    // for the dust census — the C# order (Damage `:34501` precedes the
    // dust loops `:34502+`), the `celeb2_death` resize→walk→draws shape.
    true
}

/// The tick-1 dust census (`Projectile.cs:34502-34636`), run AFTER the
/// dispatch's boxed `Damage()` walk (the C# order: `Damage()` `:34501`
/// precedes the dust loops). The three families are kind-disjoint.
pub(crate) fn explosion_tick1_draws(p: &Projectile, rng: &mut dyn DropRng) {
    let flag58 = p.kind == SOLAR_WHIP_SWORD_EXPLOSION
        || p.kind == DAYBREAK_EXPLOSION
        || p.kind == 978;
    let flag59 = p.kind == STARDUST_GUARDIAN_EXPLOSION;
    let flag60 = p.kind == DEAD_CELLS_MUSHROOM_BOI_EXPLOSION;
    // The velocity heading the UnitX loops rotate by (`:34525`/`:34532`/
    // `:34552`): `base.velocity.ToRotation()`.
    let vel_rot = p.vel.y.atan2(p.vel.x);
    let half_w = p.w * 0.5;
    if flag58 {
        // `:34504` — the Item14 cue: the pool's sound intent (the dispatch
        // forwards `wants_sound`; the arm parks it here).
        // `:34505-34509` — 4× dust-31: one RotatedByRandom pair + one
        // NextDouble radius each.
        for _ in 0..4 {
            let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
            let _ = rng.next_double();
        }
        // `:34510-34521` — 10× dust-6 pairs: the first (scale 2.7) takes
        // a RotatedByRandom pair + a NextDouble radius; the second (scale
        // 1.5) takes the same.
        for _ in 0..10 {
            let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
            let _ = rng.next_double();
            let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
            let _ = rng.next_double();
        }
        // `:34522-34528` — 5× dust-6 (scale 2.7): the UnitX form rotates
        // the random spin by the velocity heading — the same two draws.
        for _ in 0..5 {
            let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
            let _ = rotate(Vec2::X, vel_rot); // the heading spin (no draw)
        }
        // `:34529-34535` — 10× dust-31 (scale 1.5): the UnitX form again.
        for _ in 0..10 {
            let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
        }
    }
    if flag59 {
        // `:34539` — the Item14 cue (the sound intent).
        // `:34540-34548` — 20× dust-135 (scale 1.5): one RotatedByRandom
        // pair + one NextDouble radius each.
        for _ in 0..20 {
            let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
            let _ = rng.next_double();
        }
        // `:34549-34556` — 15× dust-135 (scale 2.7): the UnitX form, one
        // RotatedByRandom pair each (no radius draw).
        for _ in 0..15 {
            let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
        }
        // `:34557-34562` — the three-ring burst's six NextDouble draws:
        // three angles (num856/857/858) then three radii (num859/860/861,
        // each `7 + NextDouble * 7`).
        let num856 = rng.next_double() as f32 * (std::f32::consts::PI * 2.0);
        let num857 = rng.next_double() as f32 * (std::f32::consts::PI * 2.0);
        let num858 = rng.next_double() as f32 * (std::f32::consts::PI * 2.0);
        let num859 = 7.0 + rng.next_double() as f32 * 7.0;
        let num860 = 7.0 + rng.next_double() as f32 * 7.0;
        let num861 = 7.0 + rng.next_double() as f32 * 7.0;
        let _ = (num856, num857, num858, num859, num860, num861, half_w);
        // `:34572-34627` — the 200-particle ring: each particle's
        // NewDust velocity is an internal roll (skipped), then the
        // `NextFloat` fadeIn (`:34615`) + the `Next(4)` gate (`:34616`).
        // The `num863 <= 200` gate is always true (the loop bound), so
        // every particle draws both.
        for _ in 0..200 {
            let _ = rng.next_float(); // fadeIn (`:34615`)
            let _ = rng.next_range(0, 4); // the 2.5-fadeIn gate (`:34616`)
        }
    }
    if flag60 {
        // `:34629-34636` — the ParticleOrchestrator request: SKIPPED (the
        // particle layer). ZERO draws.
    }
}

/// W4-d12 F1 — `StatusNPC`'s explosion debuffs (`Projectile.cs:11355-11366`):
/// the 612 Solar Whip explosion and the 953 Daybreak explosion grant
/// Daybroken (189) for 300 ticks, unconditional. The 978 Volcano term
/// (OnFire 24/180, `:11359-11362`) stays verbatim — its row is aiStyle 189
/// (d19's decade), so it never reaches the 117 walks on this branch.
pub(crate) fn explosion_status_npc(kind: u16, npc: &mut crate::npc::Npc) {
    if kind == SOLAR_WHIP_SWORD_EXPLOSION || kind == DAYBREAK_EXPLOSION {
        npc.buffs.add_buff(189, 300);
    }
    if kind == 978 {
        npc.buffs.add_buff(24, 180);
    }
}

/// `Kill()`'s type-608 arm (`Projectile.cs:72177-72225`) — the Solar
/// Counter's death burst. The spawn sites `Kill()` the 608 immediately
/// (`Player.cs:21171`/`:38318`), so this is the ONLY path the 608's
/// damage takes. Returns `true` when the boxed `Damage()` should run
/// (the dispatch resolves it through `damage_pve_boxed`); the resize
/// lands here. The 160×160 box is the SetDefaults size (`:6556-6557`) —
/// the C# does NOT resize the 608 in Kill (unlike the 661's 160 resize
/// `:72228-72230`), it only re-pins `penetrate = maxPenetrate = -1`.
pub(crate) fn solar_counter_kill_resize(p: &mut Projectile) -> bool {
    if p.kind != SOLAR_COUNTER {
        return false;
    }
    // `:72179-72180` — the penetrate re-pin (the box stays 160×160).
    p.max_penetrate = -1;
    p.penetrate = -1;
    true
}

/// The 608 Kill arm's dust/gore census (`Projectile.cs:72182-72224`),
/// run AFTER the dispatch's boxed `Damage()` walk (the C# order:
/// `Damage()` `:72181` precedes the cue + dust).
pub(crate) fn solar_counter_kill_draws(p: &Projectile, rng: &mut dyn DropRng) {
    let vel_rot = p.vel.y.atan2(p.vel.x);
    // `:72183-72187` — 4× dust-31 (scale 1.5): one RotatedByRandom pair +
    // one NextDouble radius each.
    for _ in 0..4 {
        let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
        let _ = rng.next_double();
    }
    // `:72188-72201` — 30× dust-6 pairs: the first (scale 3.7) takes a
    // RotatedByRandom pair + a NextDouble radius; the second (scale 1.5)
    // takes the same.
    for _ in 0..30 {
        let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
        let _ = rng.next_double();
        let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
        let _ = rng.next_double();
    }
    // `:72202-72209` — 10× dust-6 (scale 2.7): the UnitX form, one
    // RotatedByRandom pair each.
    for _ in 0..10 {
        let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
        let _ = rotate(Vec2::X, vel_rot);
    }
    // `:72210-72216` — 10× dust-31 (scale 1.5): the UnitX form again.
    for _ in 0..10 {
        let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
    }
    // `:72217-72224` — 2× gore: the position pair (`Next(100)` X +
    // `Next(100)` Y, `:72219`), the type (`Next(61, 64)`), the
    // RotatedByRandom pair + the NextDouble radius (`:72220`), then the
    // two velocity kicks (`Next(-10, 11)` X + `Next(-10, 11)` Y,
    // `:72222-72223`). The NewGore internal kicks (`Next(10, 31)` Y +
    // `Next(-20, 21)` X, `Gore.cs:1327-1328`) stay too — the G3 standard.
    for _ in 0..2 {
        let _ = rng.next_range(0, 100); // position X (`:72219`)
        let _ = rng.next_range(0, 100); // position Y
        let _ = rng.next_range(61, 64); // the gore type
        // The NewGore internal kicks (Gore.cs:1327-1328): Y first, then X.
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
        let _ = rotated_by_random_draws(rng, std::f64::consts::PI);
        let _ = rng.next_double();
        let _ = rng.next_range(-10, 11); // velocity.X kick (`:72222`)
        let _ = rng.next_range(-10, 11); // velocity.Y kick (`:72223`)
    }
}

#[cfg(test)]
mod tests {
    //! W4-d12 — the aiStyle-117 explosion family. WRITTEN NOT RUN
    //! (code-only mode). Every expected value is hand-derived from the
    //! cited `Projectile.cs` lines; the RNG censuses pin the explicit
    //! draw order (the dust/gore bodies are the documented skip).
    use super::*;
    use crate::projectile::ProjectilePool;
    use tr_core::World;

    /// The scripted stream: records every draw (the i56 `CensusRng`
    /// shape). `next_range` returns `max - 1`; `next_float`/`next_double`
    /// return 0.5.
    struct CensusRng {
        draws: Vec<(i32, i32)>,
        floats: usize,
        doubles: usize,
    }
    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            max - 1
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.5
        }
        fn next_double(&mut self) -> f64 {
            self.doubles += 1;
            0.5
        }
    }

    fn explosion(kind: u16) -> Projectile {
        let d = crate::projectile::proj_data(kind).unwrap();
        Projectile {
            active: true,
            kind,
            owner: 0,
            pos: Vec2::new(400.0 - d.w * 0.5, 300.0 - d.h * 0.5),
            w: d.w,
            h: d.h,
            time_left: d.time_left,
            max_penetrate: d.penetrate,
            penetrate: d.penetrate,
            tile_collide: d.tile_collide,
            ignore_water: d.ignore_water,
            alpha: d.alpha,
            ai: [0.0, 0.85, 0.0, 0.0],
            ..Default::default()
        }
    }

    /// The rows pin (`Projectile.cs:6584-6597`/`:6717-6728`/`:8988-9001`/
    /// `:9875-9889`/`:6554-6566`).
    #[test]
    fn w4_d12_the_117_rows_match_set_defaults() {
        for (k, cd) in [
            (SOLAR_WHIP_SWORD_EXPLOSION, 6),
            (DAYBREAK_EXPLOSION, 80),
            (DEAD_CELLS_MUSHROOM_BOI_EXPLOSION, -1),
        ] {
            let d = crate::projectile::proj_data(k).unwrap();
            assert_eq!((d.w, d.h), (8.0, 8.0));
            assert_eq!(d.ai_style, 117);
            assert_eq!(d.alpha, 255);
            assert_eq!(d.time_left, 60);
            assert!(!d.tile_collide);
            assert!(d.ignore_water);
            assert_eq!(d.penetrate, -1);
            assert!(d.uses_local_immunity);
            assert_eq!(d.local_hit_cooldown, cd);
        }
        // The 624 has NO immunity pair (the stock reset).
        let g = crate::projectile::proj_data(STARDUST_GUARDIAN_EXPLOSION).unwrap();
        assert_eq!(g.ai_style, 117);
        assert!(!g.uses_local_immunity);
        assert_eq!(g.local_hit_cooldown, -2);
        // The 608: 160×160, timeLeft 3.
        let s = crate::projectile::proj_data(SOLAR_COUNTER).unwrap();
        assert_eq!((s.w, s.h), (160.0, 160.0), ":6556-6557");
        assert_eq!(s.time_left, 3, ":6561");
        assert_eq!(s.ai_style, 117);
        // The frame sheets: 612/953 = 5, the rest = 1.
        assert_eq!(proj_frames(SOLAR_WHIP_SWORD_EXPLOSION), 5, "Main.cs:8492");
        assert_eq!(proj_frames(DAYBREAK_EXPLOSION), 5, "Main.cs:8509");
        assert_eq!(proj_frames(STARDUST_GUARDIAN_EXPLOSION), 1);
        assert_eq!(proj_frames(DEAD_CELLS_MUSHROOM_BOI_EXPLOSION), 1);
        assert_eq!(proj_frames(SOLAR_COUNTER), 1);
    }

    /// The scale ramp + age + life cap (`:34464-34471`): ai[1] += 0.01
    /// feeds scale; ai[0]++ ages; the `3 * projFrames` cap kills.
    #[test]
    fn w4_d12_the_scale_ramp_and_life_cap() {
        let mut p = explosion(STARDUST_GUARDIAN_EXPLOSION); // 1 frame → cap 3
        // Tick 1: ai[1] 0.85 → 0.86, scale 0.86, ai[0] 1.
        let burst = ai_explosion(&mut p);
        assert!(burst, "tick 1 arms the resize");
        assert!((p.ai[1] - 0.86).abs() < 1e-5, ":34464");
        assert!((p.scale - 0.86).abs() < 1e-5, ":34465");
        assert_eq!(p.ai[0], 1.0, ":34466");
        // The resize: (int)(52 * 0.86) = (int)44.72 = 44.
        assert_eq!((p.w, p.h), (44.0, 44.0), ":34499");
        // Tick 2: no burst (ai[0] != 1).
        let burst2 = ai_explosion(&mut p);
        assert!(!burst2, ":34489-34491");
        assert_eq!(p.ai[0], 2.0);
        // Tick 3: ai[0] 3 >= 3 * 1 → kill.
        let burst3 = ai_explosion(&mut p);
        assert!(!burst3);
        assert!(!p.active, ":34467-34470");
    }

    /// The 5-frame life cap (`:34467`): the 612 lives 15 ticks (3 * 5).
    #[test]
    fn w4_d12_the_five_frame_life_cap() {
        let mut p = explosion(SOLAR_WHIP_SWORD_EXPLOSION);
        for _ in 0..14 {
            ai_explosion(&mut p);
        }
        assert!(p.active, "tick 14 < 15");
        assert_eq!(p.ai[0], 14.0);
        ai_explosion(&mut p);
        assert!(!p.active, "tick 15 >= 3 * 5 (:34467)");
    }

    /// The alpha fade + frame machine (`:34472-34484`): alpha −63/tick
    /// floored at 0; the 3-tick counter wraps the frame.
    #[test]
    fn w4_d12_the_alpha_fade_and_frame_machine() {
        let mut p = explosion(SOLAR_WHIP_SWORD_EXPLOSION);
        ai_explosion(&mut p);
        assert_eq!(p.alpha, 192, "255 − 63 (:34480)");
        assert_eq!(p.frame_counter, 1, ":34472");
        ai_explosion(&mut p);
        assert_eq!(p.alpha, 129);
        ai_explosion(&mut p);
        assert_eq!(p.alpha, 66);
        assert_eq!(p.frame_counter, 0, "the 3-tick wrap (:34473-34474)");
        assert_eq!(p.frame, 1, ":34475");
        ai_explosion(&mut p);
        assert_eq!(p.alpha, 3);
        ai_explosion(&mut p);
        assert_eq!(p.alpha, 0, "66 − 63 − 63 < 0 → 0 (:34481-34483)");
    }

    /// The flag58 dust census (`:34505-34535`): 4 + 20 + 5 + 10
    /// RotatedByRandom pairs (39 pairs = 78 doubles) + 24 radius doubles
    /// (4 + 20) = 102 doubles, ZERO ints/floats.
    #[test]
    fn w4_d12_the_flag58_dust_census() {
        let p = explosion(SOLAR_WHIP_SWORD_EXPLOSION);
        let mut rng = CensusRng { draws: Vec::new(), floats: 0, doubles: 0 };
        explosion_tick1_draws(&p, &mut rng);
        // 4 (loop1) + 20 (loop2 pairs) + 5 (loop3) + 10 (loop4) = 39 pairs
        // = 78 doubles; the radius draws: 4 (loop1) + 20 (loop2) = 24.
        assert_eq!(rng.doubles, 78 + 24, ":34505-34535");
        assert_eq!(rng.floats, 0);
        assert!(rng.draws.is_empty());
    }

    /// The flag59 dust census (`:34540-34627`): 20 pairs + 20 radius
    /// doubles (loop1) + 15 pairs (loop2) + 6 ring doubles + 200 floats +
    /// 200 Next(4) ints.
    #[test]
    fn w4_d12_the_flag59_dust_census() {
        let p = explosion(STARDUST_GUARDIAN_EXPLOSION);
        let mut rng = CensusRng { draws: Vec::new(), floats: 0, doubles: 0 };
        explosion_tick1_draws(&p, &mut rng);
        // loop1: 20 pairs (40 doubles) + 20 radius = 60; loop2: 15 pairs
        // (30 doubles); the ring: 6 doubles. Total 96 doubles.
        assert_eq!(rng.doubles, 60 + 30 + 6, ":34540-34562");
        assert_eq!(rng.floats, 200, "the per-particle fadeIn (:34615)");
        assert_eq!(rng.draws.len(), 200, "the per-particle Next(4) (:34616)");
        assert!(rng.draws.iter().all(|&(a, b)| (a, b) == (0, 4)));
    }

    /// The flag60 family draws NOTHING (`:34629-34636`, the
    /// ParticleOrchestrator skip).
    #[test]
    fn w4_d12_the_flag60_family_draws_nothing() {
        let p = explosion(DEAD_CELLS_MUSHROOM_BOI_EXPLOSION);
        let mut rng = CensusRng { draws: Vec::new(), floats: 0, doubles: 0 };
        explosion_tick1_draws(&p, &mut rng);
        assert_eq!(rng.doubles, 0);
        assert_eq!(rng.floats, 0);
        assert!(rng.draws.is_empty());
    }

    /// The 608 Kill resize (`:72179-72180`): the penetrate re-pin, the
    /// box stays 160×160.
    #[test]
    fn w4_d12_the_608_kill_resize() {
        let mut p = explosion(SOLAR_COUNTER);
        assert!(solar_counter_kill_resize(&mut p));
        assert_eq!(p.penetrate, -1, ":72180");
        assert_eq!(p.max_penetrate, -1, ":72179");
        assert_eq!((p.w, p.h), (160.0, 160.0), "the box stays (no resize)");
        // A non-608 kind is refused.
        let mut other = explosion(SOLAR_WHIP_SWORD_EXPLOSION);
        assert!(!solar_counter_kill_resize(&mut other));
    }

    /// The 608 Kill dust/gore census (`:72183-72224`): 4 + 60 + 10 + 10
    /// RotatedByRandom pairs (84 pairs = 168 doubles) + 64 radius doubles
    /// (4 + 60) + the 2 gores' draws.
    #[test]
    fn w4_d12_the_608_kill_census() {
        let p = explosion(SOLAR_COUNTER);
        let mut rng = CensusRng { draws: Vec::new(), floats: 0, doubles: 0 };
        solar_counter_kill_draws(&p, &mut rng);
        // dust: 4 (loop1) + 60 (loop2 pairs) + 10 (loop3) + 10 (loop4) =
        // 84 pairs = 168 doubles; radius: 4 + 60 = 64. gore: 2 pairs = 4
        // doubles + 2 radius = 2. Total 168 + 64 + 4 + 2 = 238 doubles.
        assert_eq!(rng.doubles, 168 + 64 + 4 + 2, ":72183-72224");
        // The gore int draws: 2 × [Next(100), Next(100), Next(61,64),
        // Next(10,31), Next(-20,21), Next(-10,11), Next(-10,11)] = 14.
        assert_eq!(rng.draws.len(), 14, "the 2 gores' 7 int draws each");
        assert_eq!(rng.draws[0], (0, 100), ":72219 position X");
        assert_eq!(rng.draws[2], (61, 64), "the gore type");
        assert_eq!(rng.draws[3], (10, 31), "the NewGore Y kick (Gore.cs:1327)");
        assert_eq!(rng.draws[4], (-20, 21), "the NewGore X kick (Gore.cs:1328)");
        assert_eq!(rng.floats, 0);
    }

    /// W4-d12 F1 — the explosion StatusNPC terms (`:11355-11366`): the
    /// 612/953 grant Daybroken 189/300 unconditionally; the 978 term stays
    /// verbatim (OnFire 24/180) though its 189 row is d19's.
    #[test]
    fn w4_d12_f1_explosion_daybroken_grants() {
        for kind in [SOLAR_WHIP_SWORD_EXPLOSION, DAYBREAK_EXPLOSION] {
            let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
            explosion_status_npc(kind, &mut n);
            assert!(n.buffs.find_buff_index(189).is_some(), "kind {kind}: Daybroken");
        }
        // The other explosion kinds grant nothing (624/1044 have no
        // StatusNPC term; the 608 dies before any contact).
        for kind in [STARDUST_GUARDIAN_EXPLOSION, DEAD_CELLS_MUSHROOM_BOI_EXPLOSION, SOLAR_COUNTER] {
            let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
            explosion_status_npc(kind, &mut n);
            assert!(n.buffs.find_buff_index(189).is_none(), "kind {kind}: no grant");
        }
        let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Zombie, 0.0, 0.0, 0, 0.0);
        explosion_status_npc(978, &mut n);
        assert!(n.buffs.find_buff_index(24).is_some(), "the 978 OnFire term (:11361)");
    }

    /// The dispatch integration: a spawned 612 runs the arm through the
    /// pool, the tick-1 resize lands and the boxed walk hits an NPC in
    /// the resized box.
    #[test]
    fn w4_d12_the_dispatch_resizes_and_damages() {
        let mut pool = ProjectilePool::new();
        let mut w = World::new("t".into(), 128, 128);
        let slot = pool
            .spawn_with_data_ai_owned(
                SOLAR_WHIP_SWORD_EXPLOSION, 0, 400.0, 300.0, 0.0, 0.0, 40, 0.0, 0.0, 0.85,
            )
            .unwrap();
        // An NPC inside the resized box (the 44×44 box centred on the
        // spawn corner + half the 8×8 row box = (404, 304)).
        let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Zombie, 395.0, 295.0, 1, 0.0);
        n.hp = 100;
        n.max_hp = 100;
        let mut npcs = [n];
        let mut rng = CensusRng { draws: Vec::new(), floats: 0, doubles: 0 };
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            None,
            crate::projectile::YoyoInputs::default(),
            crate::projectile::FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        let p = &pool.slots()[slot];
        assert_eq!(p.ai[0], 1.0, "the age advanced");
        assert_eq!((p.w, p.h), (44.0, 44.0), "the tick-1 resize (:34499)");
        assert!(npcs[0].hp < 100, "the boxed walk hit the NPC");
    }
}
