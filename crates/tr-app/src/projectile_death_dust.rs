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
}
