//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave D: sites 31-40 in
//! source order (`:53327`, `:53391`, `:53456`, `:53560`, `:53569`,
//! `:53583`, `:60816`, `:60860`, `:61488`, `:61535`).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 31-40 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 31 | `:53327` | `AI_147` state 4 gores (×3, the wave-C row) |
//! | 32 | `:53391` | `AI_147` state 6 gore (the wave-C row) |
//! | 33 | `:53456` | `CreateGroundExplosion` tile gore |
//! | 34 | `:53560` | `AI_163_Chum` wet ripple (×2, 1201) |
//! | 35 | `:53569` | `AI_163_Chum` wet splash (×2, 1208) |
//! | 36 | `:53583` | `AI_163_Chum` idle/wet gore (1201) |
//! | 37 | `:60816` | `AI_026` 881/934 frame gore |
//! | 38 | `:60860` | `AI_026` 854 frame gore |
//! | 39 | `:61488` | `AI_026` 875 dust-trail gore (1225) |
//! | 40 | `:61535` | `AI_026` 854 leaf gore (910) |
//!
//! ## RNG notes specific to this wave
//!
//! * Sites 31/32 delegate to the wave-C `celeb2_gore` keep-one (the row is
//!   identical at all seven `AI_147` sites).
//! * The ground-explosion tile walk (33) is the landed census
//!   `projectile_ai_misc.rs::create_ground_explosion` upgraded at the gore
//!   seam: the per-tile dust `NextFloat`s and the `Next(3)` gate stay the
//!   census's; this fn is the TAKEN branch only (the caller keeps the walk).
//!   The scale reads the mid-column `num` and the layer `num3` (`:53455-
//!   53456`); the post `velocity.Y` chain draws ONE `NextFloat`.
//! * The chum ripples (34/35) use `NextVector2CircularEdge` = ONE
//!   `NextFloat` (the unit, `Utils.cs:1271-1274` — no radius draw).
//! * The 1225 gore (39) draws `Next(6)` for its timeLeft INSIDE the pool
//!   (`Gore.cs:1382`); the 910 leaf (40) draws the SpecialAI==3 four
//!   (`Gore.cs:1391-1402`). Both are pool-internal — the witnesses replay
//!   them after the kicks.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_c::celeb2_gore;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::World;
use tr_core::loot::DropRng;
#[cfg(test)]
use tr_core::UnifiedRandom;

/// `Entity.Center` (`Entity.cs:50-60`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w / 2.0, p.pos.y + p.h / 2.0)
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`.
fn next_vector2_unit(rng: &mut dyn DropRng) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2CircularEdge` (`Utils.cs:1271-1274`) — the unit ONLY
/// (no radius draw, unlike Circular).
fn next_vector2_circular_edge(rng: &mut dyn DropRng, hw: f32, hh: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    Vec2::new(unit.x * hw, unit.y * hh)
}

/// Sites 31/32 — the `AI_147` state-4 (×3) / state-6 (×1) firework gores
/// (`Projectile.cs:53325-53331` / `:53389-53395`): the wave-C row, keep-one.
pub fn celeb2_gore_n(
    p: &Projectile,
    count: i32,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    for _ in 0..count {
        celeb2_gore(p, gore, rng, world);
    }
}

/// Site 33 — `CreateGroundExplosion`'s taken-branch tile gore
/// (`Projectile.cs:53453-53460`): the caller's walk has passed the
/// `num4 > 0 && Next(3) != 0` gate; `num5 = abs(num - j) / (MAX_SPREAD/2)`
/// (the mid-column distance), `NewGoreDirect(pos, Zero, 61 + Next(3),
/// 1 - num3*0.15 + num5*0.5)`; post `velocity.Y -= 0.1 + num3*0.5 +
/// num5*num3`, `velocity.Y *= NextFloat`, `position = (j*16+20, k*16+20)`.
pub fn ground_explosion_tile_gore(
    p: &Projectile,
    j: i32,
    k: i32,
    num3: i32,
    num5: f32,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let ty = 61 + rng.next_max(3);
    let scale = 1.0 - num3 as f32 * 0.15 + num5 * 0.5;
    let idx = gore.new_gore_direct(&mut crate::projectile::DynGoreRng(rng), world, p.pos, Vec2::ZERO, ty, scale);
    let g = gore.slot_mut(idx);
    g.velocity.y -= 0.1 + num3 as f32 * 0.5 + num5 * num3 as f32;
    g.velocity.y *= rng.next_float();
    g.position = Vec2::new(j as f32 * 16.0 + 20.0, k as f32 * 16.0 + 20.0);
}

/// Sites 34/35 — `AI_163_Chum`'s wet ripples (`Projectile.cs:53558-53575`):
/// the 0.5-step loop spawns TWO 1201 gores (scale ×0.7, velocity =
/// CircularEdge(10,10), Y flipped up); the 2-iteration loop spawns TWO
/// 1208 gores (scale ×0.7 + NextFloat×0.15, CircularEdge(4,4), Y up). The
/// caller gates on `frameCounter < 10 && wet`.
pub fn chum_wet_ripples(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let pos = p.pos + Vec2::new(0.0, 6.0);
    // `:53558-53566` — the 0.5-step loop: 0.0, 0.5 → two iterations.
    for _ in 0..2 {
        let idx = gore.new_gore_direct(&mut crate::projectile::DynGoreRng(rng), world, pos, Vec2::ZERO, 1201, p.scale * 0.7);
        let g = gore.slot_mut(idx);
        g.velocity = next_vector2_circular_edge(rng, 10.0, 10.0);
        if g.velocity.y > 0.0 {
            g.velocity.y *= -1.0;
        }
    }
    // `:53567-53575` — the two 1208 splashes.
    for _ in 0..2 {
        let scale = p.scale * 0.7 + rng.next_float() * 0.15;
        let idx = gore.new_gore_direct(&mut crate::projectile::DynGoreRng(rng), world, pos, Vec2::ZERO, 1208, scale);
        let g = gore.slot_mut(idx);
        g.velocity = next_vector2_circular_edge(rng, 4.0, 4.0);
        if g.velocity.y > 0.0 {
            g.velocity.y *= -1.0;
        }
    }
}

/// Site 36 — `AI_163_Chum`'s idle/wet gore (`Projectile.cs:53581-53592`):
/// the `Next(wet ? 90 : 5)` gate ALWAYS draws; when it hits 0 and NOT the
/// `!wet && vel.Length() < 0.8` flag, ONE 1201 at scale ×0.7; wet → velocity
/// `(NextFloatDirection*0.75, NextFloat)`, dry → Y flipped positive.
/// Returns true when the gore spawned.
pub fn chum_idle_gore(
    p: &Projectile,
    wet: bool,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) -> bool {
    let max = if wet { 90 } else { 5 };
    let gate = rng.next_max(max);
    let flag = !wet && p.vel.length() < 0.8;
    if gate != 0 || flag {
        return false;
    }
    let pos = p.pos + Vec2::new(0.0, 6.0);
    let idx = gore.new_gore_direct(&mut crate::projectile::DynGoreRng(rng), world, pos, Vec2::ZERO, 1201, p.scale * 0.7);
    let g = gore.slot_mut(idx);
    if wet {
        g.velocity = Vec2::new(rng.next_float_direction() * 0.75, rng.next_float());
    } else if g.velocity.y < 0.0 {
        g.velocity.y = -g.velocity.y;
    }
    true
}

/// Site 37 — `AI_026`'s 881/934 frame gore (`Projectile.cs:60807-60817`):
/// the caller gates on `frame < 6 || frame > 11`; type 1226 (881) / 1261
/// (934) at `(Center.X, position.Y)`, velocity ×0.5. No draws beyond kicks.
pub fn ai026_881_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let ty = if p.kind == 934 { 1261 } else { 1226 };
    let pos = Vec2::new(center(p).x, p.pos.y);
    gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, pos, p.vel * 0.5, ty, 1.0);
}

/// Site 38 — `AI_026`'s 854 frame gore (`Projectile.cs:60856-60861`): the
/// caller gates on `frame < 13`; type 1269 at `(Center.X, position.Y)`,
/// velocity ×0.5. No draws beyond kicks.
pub fn ai026_854_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let pos = Vec2::new(center(p).x, p.pos.y);
    gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, pos, p.vel * 0.5, 1269, 1.0);
}

/// Site 39 — `AI_026`'s 875 dust-trail gore (`Projectile.cs:61486-61493`):
/// the `Next(3)` gate; `NewGorePerfect` at `Center + (-10 + Next(-20,20)*
/// 0.5, -10 + Next(-20,20)*0.5)`, velocity ×0.1, type 1225, scale `0.5 +
/// NextFloat`. The pool's 1225 arm draws `Next(6)` for the timeLeft
/// (`Gore.cs:1382`). The trailing dust (CircularEdge + NewDust) is the dust
/// lane's.
pub fn ai026_875_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if rng.next_max(3) != 0 {
        return;
    }
    let jitter = Vec2::new(
        -10.0 + rng.next_range(-20, 20) as f32 * 0.5,
        -10.0 + rng.next_range(-20, 20) as f32 * 0.5,
    );
    let scale = 0.5 + rng.next_float();
    gore.new_gore_perfect(&mut crate::projectile::DynGoreRng(rng), world, center(p) + jitter, p.vel * 0.1, 1225, scale);
}

/// Site 40 — `AI_026`'s 854 leaf gore (`Projectile.cs:61529-61538`): the
/// `Next(6)` gate + the caller's solid/liquid tile probe; `NewGorePerfect`
/// at `Center + (Next(-30,30)*0.5, Next(-30,30)*0.5)`, velocity ×-0.2, type
/// 910; post `Frame.CurrentColumn = 0`, `timeLeft = 1`. The pool's
/// SpecialAI==3 arm draws the four leaf rolls (`Gore.cs:1391-1402`).
pub fn ai026_854_leaf(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if rng.next_max(6) != 0 {
        return;
    }
    let jitter = Vec2::new(
        rng.next_range(-30, 30) as f32 * 0.5,
        rng.next_range(-30, 30) as f32 * 0.5,
    );
    let idx = gore.new_gore_perfect(&mut crate::projectile::DynGoreRng(rng), world, center(p) + jitter, p.vel * -0.2, 910, 1.0);
    let g = gore.slot_mut(idx);
    g.frame.current_column = 0;
    g.time_left = 1;
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    fn proj() -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.pos = Vec2::new(100.0, 100.0);
        p.vel = Vec2::new(2.0, -1.0);
        p.w = 20.0;
        p.h = 20.0;
        p.scale = 1.0;
        p
    }

    /// `Projectile.cs:53453-53460` — the tile gore: Next(3) type + kicks +
    /// ONE NextFloat (the Y scale); the position write is the tile centre.
    #[test]
    fn w4_g16_the_ground_tile_gore_scales_by_layer_and_column() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(3);
        ground_explosion_tile_gore(&p, 8, 9, 2, 0.5, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(3);
        let ty = 61 + twin.next_max(3);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let _ = twin.next_float();
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.scale, 1.0 - 2.0 * 0.15 + 0.5 * 0.5);
        assert_eq!(g.position, Vec2::new(8.0 * 16.0 + 20.0, 9.0 * 16.0 + 20.0));
    }

    /// `Projectile.cs:53558-53575` — the wet ripples: 2×1201 (kicks + ONE
    /// CircularEdge float each) then 2×1208 (NextFloat scale + kicks + ONE
    /// CircularEdge float each).
    #[test]
    fn w4_g16_the_chum_ripples_draw_one_float_per_edge() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(5);
        chum_wet_ripples(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(5);
        for _ in 0..2 {
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_float(); // CircularEdge(10,10) unit
        }
        for _ in 0..2 {
            let _ = twin.next_float(); // the scale roll
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_float(); // CircularEdge(4,4) unit
        }
        assert_eq!(rng.peek(), twin.peek());
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1201, 1201, 1208, 1208]);
        for g in &gore.slots()[..4] {
            assert!(g.velocity.y <= 0.0, "the ripple flips Y up");
        }
    }

    /// `Projectile.cs:53581-53592` — the idle gate: Next(5) dry / Next(90)
    /// wet ALWAYS draws; the dry slow-velocity flag suppresses the spawn
    /// AFTER the gate.
    #[test]
    fn w4_g16_the_chum_idle_gate_draws_before_the_flag() {
        let w = air();
        let mut p = proj();
        p.vel = Vec2::new(0.1, 0.0); // length < 0.8 → the dry flag
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(5) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(!chum_idle_gore(&p, false, &mut gore, &mut rng, &w));
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(5);
        assert_eq!(rng.peek(), twin.peek(), "the gate drew, the flag suppressed");
        // Wet: no flag; a taken gate spawns + the two velocity floats.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(90) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(chum_idle_gore(&p, true, &mut gore, &mut rng, &w));
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(90);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let _ = twin.next_float_direction(); // NextFloatDirection = NextDouble*2-1
        let _ = twin.next_float(); // the Y NextFloat
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, 1201);
    }

    /// `Projectile.cs:61486-61493` — the 875 gore: Next(3) gate, the
    /// Next(-20,20) jitter pair, the NextFloat scale, the kicks, then the
    /// pool's 1225 Next(6) timeLeft roll.
    #[test]
    fn w4_g16_the_875_gore_draws_the_pool_timeleft_roll() {
        let w = air();
        let p = proj();
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(3) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        ai026_875_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(3);
        let _ = twin.next_range(-20, 20);
        let _ = twin.next_range(-20, 20);
        let _ = twin.next_float();
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let tl = twin.next_max(6); // Gore.cs:1382
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, 1225);
        assert_eq!(g.time_left, 10 + tl);
    }

    /// `Projectile.cs:61529-61538` — the 854 leaf: Next(6) gate, the
    /// Next(-30,30) jitter pair, the kicks, then the SpecialAI==3 four
    /// (two NextFloat + Next(8) row + Next(8) counter); the post-writes
    /// (column 0, timeLeft 1) draw nothing.
    #[test]
    fn w4_g16_the_854_leaf_draws_the_special_ai_four() {
        let w = air();
        let p = proj();
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(6) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        ai026_854_leaf(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(6);
        let _ = twin.next_range(-30, 30);
        let _ = twin.next_range(-30, 30);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let _ = twin.next_float(); // SpecialAI vel X
        let _ = twin.next_float(); // SpecialAI vel Y
        let row = twin.next_max(8);
        let counter = twin.next_max(8);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, 910);
        assert_eq!(g.time_left, 1);
        assert_eq!(g.frame.current_column, 0);
        assert_eq!(g.frame.current_row, row as u8);
        assert_eq!(g.frame_counter, counter as u8);
    }
}
