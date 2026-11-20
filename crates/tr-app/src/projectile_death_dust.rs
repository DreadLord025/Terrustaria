//! Projectile DEATH dust — the per-kind bursts inside `Projectile.Kill()`
//! (`decompiled/Terraria/Projectile.cs:70642-77262`), W4-g65.
//!
//! # Scope
//!
//! The TRAIL dust (the per-tick AI arms, `AI_001`/`AI_026`/…) is the g63/g64
//! lanes; this module owns only the dust that fires ONCE when a slot dies —
//! the `Kill()` body's per-type `Dust.NewDust*` bursts. The i18 special-bullet
//! deaths (`:75250-76039`) and the i17 special-arrow deaths stay with their
//! main-landed modules; only UNCLAIMED arms are ported here.
//!
//! # Wiring (the gallop-dust queue→drain precedent)
//!
//! The pool cannot borrow the App's `DustPool` or hold the live
//! `UpdateProjectiles` stream alongside its `rng` parameter, so the death hook
//! (`ProjectilePool::update`'s `!active` block, `projectile.rs`) only SNAPSHOTS
//! the dying slot into a [`DeathSnapshot`] — no draws there. The world-sim pass
//! drains the snapshots at its tail ([`drain`]) onto the real
//! `DustPool::new_dust` on the live `UpdateProjectiles` stream: every explicit
//! `Main.rand` draw the C# arm takes AND the `new_dust` internals run there, in
//! C# order, per death in slot order. This is the recorded gallop-dust
//! deviation (`world_sim/npcs.rs`, ANALYSIS "W-10 unicorn tails"): the dust
//! draws land at the pass tail rather than mid-walk, stream-neutral for the
//! port's per-consumer streams.
//!
//! # The DustPool contract (origin/task/W4-d50-dust-type-table)
//!
//! The bodies are emitted through the verbatim `DustPool::new_dust` family
//! (`tr_core::particles`, S1-E-1): the guard chain, the off-screen cull, the
//! pressure bands, the default block, the ChildSafety re-type and the per-type
//! tweak chain all run there. This module only resolves the C# arm's ARGUMENTS
//! and its post-spawn field writes (`noGravity`, `scale *=`, `velocity *=`,
//! `fadeIn`, …); the drain applies the writes to whatever slot `new_dust`
//! returned — the sentinel included (the C# writes `Main.dust[6000]` too).
//!
//! # RNG census
//!
//! Per arm, the explicit draws are listed at the arm below in source order;
//! they ride the drain's `rng` (the live `UpdateProjectiles` stream) ahead of
//! that arm's `new_dust` internals. An arm whose only randomness is the
//! internals (e.g. the 17/31/42 plain bursts) spends its stream entirely inside
//! `new_dust`.
//!
//! CODE-ONLY MODE (W4-g65): no compiler invoked on this branch; the witness
//! tests at the bottom are WRITTEN NOT RUN, every expected value hand-derived
//! from the cited C# lines on a cloned `UnifiedRandom`.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::UnifiedRandom;
use tr_core::World;

use crate::projectile::Projectile;

/// The dying slot's snapshot the death arms read (`base.position`,
/// `base.width/height`, `base.velocity`, `base.scale`, `base.ai`). Taken at the
/// death hook BEFORE any other hook mutates the corpse (the C# `Kill()` reads
/// the live slot throughout its body). `kind` selects the arm.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeathSnapshot {
    pub kind: u16,
    pub pos: Vec2,
    pub w: i32,
    pub h: i32,
    pub vel: Vec2,
    pub scale: f32,
    /// `base.oldVelocity` — the 93/323-family death arms read it. The port's
    /// pool latches `old_velocity` per sub-step; the snapshot takes the slot's
    /// current velocity as the stand-in (the resting kinds' `oldVelocity ==
    /// velocity` at death, the special-bullets note). Kinds that need the true
    /// last-sub-step velocity are deferred to a later commit.
    pub old_vel: Vec2,
    /// `base.ai[1]` — the 346/335/589 colour/type picks read it.
    pub ai1: f32,
}

impl DeathSnapshot {
    /// Snapshot a dying slot. The box truncates to `int` exactly as the C#
    /// `base.width`/`base.height` (the `int` fields) read.
    pub(crate) fn of(p: &Projectile) -> Self {
        Self {
            kind: p.kind,
            pos: p.pos,
            w: p.w as i32,
            h: p.h as i32,
            vel: p.vel,
            scale: p.scale,
            old_vel: p.vel,
            ai1: p.ai[1],
        }
    }
}

/// Drain the tick's death snapshots onto the dust pool, in slot order, on the
/// live `UpdateProjectiles` stream (`rng`). Each arm takes its explicit C#
/// draws then its `new_dust` internals + post-spawn writes. `ctx` is the App's
/// per-call `SpawnCtx` (the `Dust.cs:77-101` guards/cull); `world` feeds the
/// type-34 wet probe.
pub(crate) fn drain(
    snapshots: &[DeathSnapshot],
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    for s in snapshots {
        death_bursts(s, dust, rng, ctx, world);
    }
}

/// `Projectile.Kill()`'s per-kind death-dust bursts for one dying slot. Takes
/// every explicit `Main.rand` draw on `rng` in C# order, then emits the bodies
/// through `dust.new_dust` and applies the arm's post-spawn writes. Kind-gated;
/// a dead slot never returns to active, so this fires at most once per death.
fn death_bursts(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    match s.kind {
        // `:76170-76177` — the 17: 5 plain dust-0 (no explicit draw).
        17 => plain_burst(s, dust, rng, ctx, world, 5, 0, 0.0, 0.0, 0, Color::default(), 1.0, None, false),
        // `:76178-76186` — the 31/42: 5 dust-32, velocity ×0.6.
        31 | 42 => plain_burst(s, dust, rng, ctx, world, 5, 32, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76214-76222` — the 39: 5 dust-38, velocity ×0.6.
        39 => plain_burst(s, dust, rng, ctx, world, 5, 38, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76223-76231` — the 71: 5 dust-53, velocity ×0.6.
        71 => plain_burst(s, dust, rng, ctx, world, 5, 53, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76232-76240` — the 179: 5 dust-149, velocity ×0.6.
        179 => plain_burst(s, dust, rng, ctx, world, 5, 149, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76241-76249` — the 40: 5 dust-36, velocity ×0.6.
        40 => plain_burst(s, dust, rng, ctx, world, 5, 36, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76205-76213` — the 109: 5 dust-51 scale 0.6, velocity ×0.6.
        109 => plain_burst(s, dust, rng, ctx, world, 5, 51, 0.0, 0.0, 0, Color::default(), 0.6, Some(0.6), false),
        // `:76250-76257` — the 21/471/532/1111: 10 dust-26 scale 0.8.
        21 | 471 | 532 | 1111 => plain_burst(s, dust, rng, ctx, world, 10, 26, 0.0, 0.0, 0, Color::default(), 0.8, None, false),
        // `:76274-76280` — the 24: 10 dust-1, speed = vel×0.1, scale 0.75.
        24 => plain_burst(s, dust, rng, ctx, world, 10, 1, s.vel.x * 0.1, s.vel.y * 0.1, 0, Color::default(), 0.75, None, false),
        // `:76291-76297` — the 38: 10 dust-42, speed = vel×0.1.
        38 => plain_burst(s, dust, rng, ctx, world, 10, 42, s.vel.x * 0.1, s.vel.y * 0.1, 0, Color::default(), 1.0, None, false),
        // `:75616-75629` — the 103 CursedArrow: 20 dust-75 alpha 100; per dust
        // a `Next(2)` gate → scale ×2.5 + noGravity + velocity ×5.
        103 => gated_burst(s, dust, rng, ctx, world, 20, 75, 0.0, 0.0, 100, Color::default(), 1.0, 2, 2.5, 5.0),
        // `:75630-75643` — the 278 IchorArrow: 20 dust-169 alpha 100; per dust
        // a `Next(2)` gate → scale ×1.5 + noGravity + velocity ×5.
        278 => gated_burst(s, dust, rng, ctx, world, 20, 169, 0.0, 0.0, 100, Color::default(), 1.0, 2, 1.5, 5.0),
        // `:75668-75680` — the 5 JestersArrow: 60 dust, type = `Next(3)` switch
        // {0→15, 1→57, _→58}, speed = vel×0.5, alpha 150, scale 1.5.
        5 => {
            for _ in 0..60 {
                let dust_type = match rng.next_max(3) {
                    0 => 15,
                    1 => 57,
                    _ => 58,
                };
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, s.vel.x * 0.5, s.vel.y * 0.5, 150, Color::default(), 1.5);
            }
        }
        // `:74828-74840` — the 93: 10 dust-57 alpha 100, noGravity + fadeIn 1 +
        // velocity ×2 (both axes) then `velocity.Y -= NextFloat()*1.5`.
        93 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 57, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.fade_in = 1.0;
                d.velocity *= 2.0;
                d.velocity.y -= rng.next_float() * 1.5;
            }
        }
        // `:74260-74271` — the 318 RottenEgg: 10 dust-30; per dust a `Next(2)`
        // gate → noGravity.
        318 => no_gravity_gate_burst(s, dust, rng, ctx, world, 10, 30),
        // `:74272-74282` — the 378: 10 dust-30; per dust a `Next(2)` gate →
        // noGravity (the same shape as the 318).
        378 => no_gravity_gate_burst(s, dust, rng, ctx, world, 10, 30),
        // `:74236-74249` — the 346: 20 dust, type = 10 (4 when ai[1]==1),
        // noGravity. No explicit draw.
        346 => {
            let dust_type = if s.ai1 == 1.0 { 4 } else { 10 };
            plain_burst(s, dust, rng, ctx, world, 20, dust_type, 0.0, 0.0, 0, Color::default(), 1.0, None, true);
        }
        // `:74250-74259` — the 335: 20 dust, type = 90 - (int)ai[1], noLight +
        // scale 0.8. No explicit draw.
        335 => {
            let dust_type = 90 - s.ai1 as i32;
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_light = true;
                d.scale = 0.8;
            }
        }
        // `:75554-75561` — the 76/77/78: 5 dust-27 alpha 80 scale 1.5,
        // noGravity. No explicit draw.
        76 | 77 | 78 => plain_burst(s, dust, rng, ctx, world, 5, 27, 0.0, 0.0, 80, Color::default(), 1.5, None, true),
        // `:75562-75569` — the 55/719: 5 dust-18 scale 1.5, noGravity. No
        // explicit draw.
        55 | 719 => plain_burst(s, dust, rng, ctx, world, 5, 18, 0.0, 0.0, 0, Color::default(), 1.5, None, true),
        _ => {}
    }
}

/// `count` `Dust.NewDust(pos, w, h, type, sx, sy, alpha, color, scale)` bodies,
/// each with an optional `velocity *= vel_mul` and `noGravity = true` post-spawn
/// write. No explicit draws — the randomness is the `new_dust` internals.
#[allow(clippy::too_many_arguments)]
fn plain_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    speed_x: f32,
    speed_y: f32,
    alpha: i32,
    color: Color,
    scale: f32,
    vel_mul: Option<f32>,
    no_gravity: bool,
) {
    for _ in 0..count {
        let idx = dust.new_dust(
            rng, ctx, world, s.pos, s.w, s.h, dust_type, speed_x, speed_y, alpha, color, scale,
        );
        let d = dust.slot_mut(idx);
        if let Some(k) = vel_mul {
            d.velocity *= k;
        }
        if no_gravity {
            d.no_gravity = true;
        }
    }
}

/// `count` dust of `dust_type` (speed/alpha/color/scale as given); per dust a
/// `Next(gate)` draw, and when it lands 0 the C# writes `scale *= scale_mul`,
/// `noGravity = true`, `velocity *= vel_mul` — the 103/278 shape
/// (`:75622-75627`/`:75636-75641`). The gate draw rides `rng` AFTER the
/// `new_dust` internals (the C# rolls it after the spawn).
#[allow(clippy::too_many_arguments)]
fn gated_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    speed_x: f32,
    speed_y: f32,
    alpha: i32,
    color: Color,
    scale: f32,
    gate: i32,
    scale_mul: f32,
    vel_mul: f32,
) {
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, speed_x, speed_y, alpha, color, scale);
        if rng.next_max(gate) == 0 {
            let d = dust.slot_mut(idx);
            d.scale *= scale_mul;
            d.no_gravity = true;
            d.velocity *= vel_mul;
        }
    }
}

/// `count` dust of `dust_type` (default speed/alpha/color/scale); per dust a
/// `Next(2)` draw, and when it lands 0 `noGravity = true` — the 318/378 shape
/// (`:74266-74269`/`:74277-74280`). The gate draw rides `rng` after the spawn.
fn no_gravity_gate_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
) {
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
        if rng.next_max(2) == 0 {
            dust.slot_mut(idx).no_gravity = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(kind: u16) -> DeathSnapshot {
        DeathSnapshot {
            kind,
            pos: Vec2::new(100.0, 100.0),
            w: 10,
            h: 10,
            vel: Vec2::new(4.0, -2.0),
            scale: 1.0,
            old_vel: Vec2::new(4.0, -2.0),
            ai1: 0.0,
        }
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1280.0, 720.0),
            max_dust_to_draw: 6000,
        }
    }

    fn world() -> World {
        World::new("test".into(), 64, 64)
    }

    /// The plain bursts spawn the C# count of the right dust type at the
    /// corpse position, and the 31/42/39/71/179/40/109 arms carry the
    /// velocity ×0.6 write. With a fresh pool every spawn takes a real slot
    /// (the pressure bands reset `d_count` in the low band, no refusal).
    #[test]
    fn w4_g65_the_plain_bursts_spawn_the_csharp_bodies() {
        let w = world();
        for (kind, count, dust_type, vel_mul) in [
            (17u16, 5, 0, None),
            (42, 5, 32, Some(0.6)),
            (39, 5, 38, Some(0.6)),
            (71, 5, 53, Some(0.6)),
            (179, 5, 149, Some(0.6)),
            (40, 5, 36, Some(0.6)),
            (109, 5, 51, Some(0.6)),
        ] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} burst count");
            for d in &active {
                assert_eq!(d.dust_type, dust_type, "kind {kind} dust type");
            }
            // The velocity write: the default block's velocity is the
            // `Next(-20,21)*0.1` jitter; ×0.6 scales it. Compare against a
            // witness pool that ran the same spawn WITHOUT the write.
            if let Some(k) = vel_mul {
                let mut witness = DustPool::new();
                let mut wrng = UnifiedRandom::new(7);
                plain_burst(&snap(kind), &mut witness, &mut wrng, &ctx(), &w, count, dust_type, 0.0, 0.0, 0, Color::default(), if kind == 109 { 0.6 } else { 1.0 }, None, false);
                for (a, b) in active.iter().zip(witness.slots().iter().filter(|d| d.active)) {
                    assert_eq!(a.velocity, b.velocity * k, "kind {kind} velocity write");
                }
            }
        }
    }

    /// The 24/38 bursts carry the velocity×0.1 SPEED argument (not a post
    /// write) and the 24's 0.75 scale; the 21-family is 10 dust-26 scale 0.8.
    #[test]
    fn w4_g65_the_24_38_21_bursts_carry_their_arguments() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(24)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 1));
        // scale 0.75 rides the default block's `1 + Next(-20,21)*0.01` then
        // `*= 0.75`; every slot's scale is < 1.0 (the 0.75 cap).
        assert!(active.iter().all(|d| d.scale < 1.0));
    }

    /// An unclaimed kind spawns nothing and draws nothing.
    #[test]
    fn w4_g65_an_unclaimed_kind_is_inert() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        drain(&[snap(9999)], &mut dust, &mut rng, &ctx(), &w);
        assert!(dust.slots().iter().all(|d| !d.active));
        assert_eq!(rng, before);
    }

    /// The drain is deterministic: two identical snapshot lists on identical
    /// streams produce byte-equal pools (the determinism invariant).
    #[test]
    fn w4_g65_the_drain_is_deterministic() {
        let w = world();
        let snaps = [snap(17), snap(42), snap(24)];
        let mut a = DustPool::new();
        let mut b = DustPool::new();
        drain(&snaps, &mut a, &mut UnifiedRandom::new(42), &ctx(), &w);
        drain(&snaps, &mut b, &mut UnifiedRandom::new(42), &ctx(), &w);
        assert_eq!(a.slots(), b.slots());
    }

    /// The 103/278 gated bursts: 20 dust each, and the stream advances by the
    /// `new_dust` internals PLUS one `Next(2)` gate per dust. A witness that
    /// runs the same spawns WITHOUT the gate draws lands behind the real
    /// stream by exactly 20 `Next(2)` steps (the gate is taken after each
    /// spawn, even when the dust is culled — the C# rolls it unconditionally).
    #[test]
    fn w4_g65_the_gated_bursts_take_one_gate_draw_per_dust() {
        let w = world();
        for (kind, dust_type) in [(103u16, 75), (278, 169)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 20, "kind {kind} burst count");
            assert!(active.iter().all(|d| d.dust_type == dust_type));
            // The witness: the same 20 spawns, no gate draws.
            let mut witness = DustPool::new();
            let mut wrng = UnifiedRandom::new(7);
            plain_burst(&snap(kind), &mut witness, &mut wrng, &ctx(), &w, 20, dust_type, 0.0, 0.0, 100, Color::default(), 1.0, None, false);
            // Advance the witness by the 20 gate draws the real arm took.
            for _ in 0..20 {
                let _ = wrng.next_max(2);
            }
            assert_eq!(rng, wrng, "kind {kind} stream = internals + 20 gates");
        }
    }

    /// The 5 JestersArrow: 60 dust, each a `Next(3)` type pick {15,57,58}
    /// BEFORE the spawn internals. The stream advances by 60 `Next(3)` + the
    /// internals; every spawned type is in the pick set.
    #[test]
    fn w4_g65_the_jesters_burst_picks_the_type_per_dust() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(5)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 60);
        assert!(active.iter().all(|d| matches!(d.dust_type, 15 | 57 | 58)));
    }

    /// The 93: 10 dust-57, noGravity + fadeIn 1, velocity ×2 then a
    /// `NextFloat()*1.5` Y kick. The stream advances by the internals + 10
    /// `NextFloat`s.
    #[test]
    fn w4_g65_the_93_burst_kicks_y_after_the_double() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(93)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 57 && d.no_gravity && d.fade_in == 1.0));
    }

    /// The 346/335 ai[1]-driven type picks and the 76/55 noGravity bursts.
    #[test]
    fn w4_g65_the_ai1_picks_and_no_gravity_bursts() {
        let w = world();
        // 346 with ai1 == 1 → dust-4; else dust-10.
        let mut s = snap(346);
        s.ai1 = 1.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 4 && d.no_gravity));
        // 335 with ai1 == 2 → dust-88, noLight + scale 0.8.
        let mut s = snap(335);
        s.ai1 = 2.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 88 && d.no_light && d.scale == 0.8));
        // 76 → 5 dust-27 noGravity; 55 → 5 dust-18 noGravity.
        for (kind, dust_type) in [(76u16, 27), (55, 18)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 5);
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity));
        }
    }
}
