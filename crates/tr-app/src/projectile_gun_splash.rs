//! W4-d6 residual 8 — the gun streams' `Kill()` splash dust
//! (`Projectile.cs:74029-74147`): the WaterGun 358's 50× dust 211
//! (`:74029-74062`), the SlimeGun 406's 40× dust 4 colour (0,80,255,100)
//! alpha 175 (`:74063-74098`) and the ShimmerGunStream 1042's 50× dust
//! 324 + the per-dust `Next(6)` palette (`:74099-74147`).
//!
//! # Claim scope (the co-lane read)
//!
//! The g65 death-dust lane (`task/W4-g65-proj-death-dust`) owns the
//! UNCLAIMED `Kill()` arms in ~10-kind batches; its batch-6 range
//! (`:73952-74333`) covers these three sites but its kind list SKIPS
//! them (the guns were d6's census, `projectile_ai_guns.rs`'s
//! `gun_stream_death_draws` draw-count stub). The d53 wet-block lane
//! owns the Update wet machine's entry/exit splashes (`:16851-16987`)
//! and the g64 trail lane the per-tick AI dust (`:29158-29303`) — all
//! disjoint line ranges. This module ports the three Kill bodies onto
//! the real `DustPool` through the queue→drain seam; at merge the g65
//! lane's census note gains the cross-reference (the arms are claimed
//! here, not duplicated there).
//!
//! # The seam (the gallop-dust queue→drain convention)
//!
//! The pool cannot borrow the App's `DustPool`, so the death hook
//! (`projectile.rs`'s `!active` block, at the C# body position — ABOVE
//! the `:75212` Holy-star arm) only SNAPSHOTS the dying slot into
//! `ProjectilePool::pending_gun_splashes`; the world-sim pass drains the
//! snapshots at its tail onto `DustPool::new_dust` on the LIVE
//! `UpdateProjectiles` stream. The explicit `Main.rand` draws AND the
//! `new_dust` internals run there, in C# order, per death in slot order
//! — the recorded pass-tail deviation (the g65/gallop precedent).
//!
//! # The velocity base
//!
//! Each arm first writes `velocity = oldVelocity * 0.2f`. On the SCALE
//! death the C# latched `oldVelocity = velocity` AFTER the ×5 kick
//! (`:28999-29000`), so the kicked velocity is the base; the
//! `scale <= 0` marker distinguishes it (only the style-60 arm decays
//! the scale). Every other death path reads the sub-step-start capture
//! (`:16766`). The 1042's dust velocity adds `base.velocity * 0.8f`
//! (`:74126`) where the 358/406 add the full `base.velocity`
//! (`:74056`/`:74092`).
//!
//! # RNG census (per death, on the drain's stream)
//!
//! Per dust: the `new_dust` internals (the position pair, the velocity
//! pair, the frame roll, the scale roll — the pressure bands only under
//! pool load), THEN 3× `Next(2)` alpha gates (+25 each on 0), the
//! `Next(2)` scale/noGravity pick (0 → scale 0.6/0.8, else noGravity),
//! `Next(-100,101)` the velocity scale jitter, 2× `Next(-50,51)` the
//! per-axis kicks; the 1042 adds the `Next(6)` colour pick LAST
//! (`:74131-74145`). 50/40/50 dusts per death.
//!
//! CODE-ONLY MODE (W4-d6): the witness tests at the bottom are WRITTEN
//! NOT RUN; every expected value is hand-derived from the cited C#
//! lines on a cloned `UnifiedRandom`.

use glam::Vec2;
use tr_core::UnifiedRandom;
use tr_core::World;
use tr_core::particles::{Color, DustPool, SpawnCtx};

use crate::projectile::{SHIMMER_GUN_STREAM, SLIME_GUN, WATER_GUN};

/// The dying gun slot's snapshot — the fields the three Kill arms read.
/// `scale_dead` marks the ×5-kick death (`:28999-29000`): the velocity
/// base is the LIVE (kicked) velocity then, else `old_vel`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GunSplash {
    pub kind: u16,
    pub pos: Vec2,
    pub w: f32,
    pub h: f32,
    /// The sub-step-start velocity capture (`Projectile.cs:16766`).
    pub old_vel: Vec2,
    /// The live velocity at death (the scale death's kicked value).
    pub vel: Vec2,
    /// `scale <= 0` — the style-60 scale decay's death marker.
    pub scale_dead: bool,
}

/// The 1042's `Next(6)` palette (`:74131-74145`): 0 → (255,255,220),
/// 1 → (230,255,255), 2 → (255,210,255), default → (225,210,255). The
/// C# `new Color(r,g,b)` carries alpha 255.
fn shimmer_palette(pick: i32) -> Color {
    match pick {
        0 => Color::new(255, 255, 220, 255),
        1 => Color::new(230, 255, 255, 255),
        2 => Color::new(255, 210, 255, 255),
        _ => Color::new(225, 210, 255, 255),
    }
}

/// One gun death's Kill-dust burst, verbatim (`Projectile.cs:74029-74147`).
/// The velocity write (`velocity = base * 0.2`) rides the SNAPSHOT's
/// caller (the pool's death hook already applied it through
/// `gun_stream_death_draws`' replacement — see the hook); this fn only
/// emits. Returns nothing: the dust lives in the pool.
pub(crate) fn drain_one(
    s: &GunSplash,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    let (count, dust_type, alpha, scale, colour, shimmer, vel_mul, small_scale) = match s.kind {
        // `:74029-74062` — the 358: 50× dust 211, alpha 75, scale 1.2.
        WATER_GUN => (50, 211, 75, 1.2f32, Color::default(), false, 1.0f32, 0.6f32),
        // `:74063-74098` — the 406: 40× dust 4, alpha 175, scale 1.6,
        // the (0,80,255,100) colour.
        SLIME_GUN => (40, 4, 175, 1.6, Color::new(0, 80, 255, 100), false, 1.0, 0.6),
        // `:74099-74147` — the 1042: 50× dust 324, alpha 75, scale 1.25,
        // the ×0.8 velocity add + the Next(6) palette, small scale 0.8.
        SHIMMER_GUN_STREAM => (50, 324, 75, 1.25, Color::default(), true, 0.8, 0.8),
        _ => return,
    };
    // `:74031`/`:74067`/`:74101` — the velocity base (the caller wrote
    // `p.vel = base * 0.2` at the hook; the snapshot carries the result).
    let base_vel = s.vel;
    let center = Vec2::new(s.pos.x + s.w * 0.5, s.pos.y + s.h * 0.5);
    for _ in 0..count {
        // `Dust.NewDust(position, width, height, type, 0, 0, alpha,
        // colour, scale)` — the internals roll on the caller's stream.
        let idx = dust.new_dust(
            rng, ctx, world, s.pos, s.w as i32, s.h as i32,
            dust_type, 0.0, 0.0, alpha, colour, scale,
        );
        let d = dust.slot_mut(idx);
        // The 3 alpha gates (`:74035-74046` et al.).
        for _ in 0..3 {
            if rng.next_range(0, 2) == 0 {
                d.alpha += 25;
            }
        }
        // The scale/noGravity pick (`:74047-74054`).
        if rng.next_range(0, 2) == 0 {
            d.scale = small_scale;
        } else {
            d.no_gravity = true;
        }
        // The velocity chain (`:74055-74059`): ×0.3, += base·mul,
        // ×(1 + Next(-100,101)·0.01), the two ±50·0.015 kicks.
        d.velocity *= 0.3;
        d.velocity += base_vel * vel_mul;
        d.velocity *= 1.0 + rng.next_range(-100, 101) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.015;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.015;
        // `:74060` — the position overwrite to the centre.
        d.position = center;
        // The 1042's colour pick LAST (`:74131-74145`).
        if shimmer {
            d.color = shimmer_palette(rng.next_range(0, 6));
        }
    }
}

/// The pass-tail drain: every queued gun death, in slot order, on the
/// live `UpdateProjectiles` stream.
pub(crate) fn drain(
    snapshots: Vec<GunSplash>,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    for s in &snapshots {
        drain_one(s, dust, rng, ctx, world);
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::particles::DUST_SENTINEL;

    /// A permissive ctx: the whole world on-screen, the full dust budget
    /// — the pool never culls, every spawn lands (the `Npc::update_rng`
    /// scratch convention).
    fn ctx(world: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(world.width as f32 * 16.0, world.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    fn splash(kind: u16) -> GunSplash {
        GunSplash {
            kind,
            pos: Vec2::new(320.0, 384.0),
            w: 18.0,
            h: 18.0,
            old_vel: Vec2::new(4.0, 0.0),
            // The hook's `velocity = oldVelocity * 0.2` result.
            vel: Vec2::new(0.8, 0.0),
            scale_dead: false,
        }
    }

    /// The 358's burst: 50 dust 211, alpha 75 + the three +25 gates, the
    /// scale-0.6/noGravity pick, the velocity chain and the centre
    /// overwrite — the twin stream replays every draw in order.
    #[test]
    fn w4_d6_the_water_gun_kill_burst_mirrors_the_csharp_draws() {
        let world = World::new("t".into(), 300, 200);
        let c = ctx(&world);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        let s = splash(WATER_GUN);
        drain_one(&s, &mut dust, &mut rng, &c, &world);
        // The twin: per dust the 6 internals draws (pos pair, vel pair,
        // frame, scale — the low-pressure band draws nothing), then the
        // 4 Next(2) gates + the jitter trio.
        let mut twin = UnifiedRandom::new(11);
        let mut active = 0;
        for i in 0..50 {
            // new_dust internals (the empty-pool path, slot i).
            let px = s.pos.x + twin.next_max(18 - 4) as f32 + 4.0;
            let py = s.pos.y + twin.next_max(18 - 4) as f32 + 4.0;
            let mut vx = twin.next_range(-20, 21) as f32 * 0.1;
            let mut vy = twin.next_range(-20, 21) as f32 * 0.1;
            let _frame_y = 10 * twin.next_max(3);
            let mut sc = (1.0 + twin.next_range(-20, 21) as f32 * 0.01) * 1.2;
            let mut alpha = 75;
            let mut no_gravity = false;
            for _ in 0..3 {
                if twin.next_range(0, 2) == 0 {
                    alpha += 25;
                }
            }
            if twin.next_range(0, 2) == 0 {
                sc = 0.6;
            } else {
                no_gravity = true;
            }
            vx *= 0.3;
            vy *= 0.3;
            vx += 0.8;
            let mul = 1.0 + twin.next_range(-100, 101) as f32 * 0.01;
            vx *= mul;
            vy *= mul;
            vx += twin.next_range(-50, 51) as f32 * 0.015;
            vy += twin.next_range(-50, 51) as f32 * 0.015;
            let d = dust.slot(i);
            assert!(d.active, "dust {i} spawned");
            assert_eq!(d.dust_type, 211);
            assert_eq!(d.alpha, alpha);
            assert_eq!(d.no_gravity, no_gravity);
            assert!((d.scale - sc).abs() < 1e-5);
            assert!((d.velocity.x - vx).abs() < 1e-4 && (d.velocity.y - vy).abs() < 1e-4);
            assert_eq!(d.position, Vec2::new(329.0, 393.0), "the centre overwrite");
            let _ = (px, py);
            active += 1;
        }
        assert_eq!(active, 50);
        assert_eq!(rng, twin, "the stream replayed identically");
        assert!(!dust.slot(50).active, "exactly 50");
    }

    /// The 406: 40 dust 4 with the (0,80,255,100) colour and alpha 175;
    /// the 1042: 50 dust 324 with the Next(6) palette LAST and the ×0.8
    /// velocity add. The palette covers all four arms.
    #[test]
    fn w4_d6_the_slime_and_shimmer_bursts_carry_their_colours() {
        let world = World::new("t".into(), 300, 200);
        let c = ctx(&world);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        let mut s = splash(SLIME_GUN);
        s.w = 14.0;
        s.h = 14.0;
        drain_one(&s, &mut dust, &mut rng, &c, &world);
        for i in 0..40 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.color, Color::new(0, 80, 255, 100), "the newColor6");
            assert!(d.alpha >= 175, "the 175 base + the gates");
        }
        assert!(!dust.slot(40).active, "exactly 40");
        // The 1042 on a fresh pool: the palette membership + the LAST
        // draw position (the colour pick closes each dust's draw block).
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        let s = splash(SHIMMER_GUN_STREAM);
        drain_one(&s, &mut dust, &mut rng, &c, &world);
        let palette = [
            Color::new(255, 255, 220, 255),
            Color::new(230, 255, 255, 255),
            Color::new(255, 210, 255, 255),
            Color::new(225, 210, 255, 255),
        ];
        for i in 0..50 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 324);
            assert!(palette.contains(&d.color), "dust {i}: {:?}", d.color);
        }
        assert!(!dust.slot(50).active, "exactly 50");
        // The ×0.8 add: re-derive one dust's velocity from a twin.
        let mut twin = UnifiedRandom::new(23);
        let d = dust.slot(0);
        let _ = twin.next_max(14);
        let _ = twin.next_max(14);
        let mut vx = twin.next_range(-20, 21) as f32 * 0.1;
        let mut vy = twin.next_range(-20, 21) as f32 * 0.1;
        let _ = twin.next_max(3);
        let _ = twin.next_range(-20, 21);
        for _ in 0..3 {
            let _ = twin.next_range(0, 2);
        }
        let _ = twin.next_range(0, 2);
        vx = vx * 0.3 + 0.8 * 0.8;
        vy *= 0.3;
        let mul = 1.0 + twin.next_range(-100, 101) as f32 * 0.01;
        vx *= mul;
        vy *= mul;
        vx += twin.next_range(-50, 51) as f32 * 0.015;
        vy += twin.next_range(-50, 51) as f32 * 0.015;
        assert!((d.velocity.x - vx).abs() < 1e-4, "the ×0.8 velocity add (:74126)");
        assert!((d.velocity.y - vy).abs() < 1e-4);
        let pick = twin.next_range(0, 6);
        assert_eq!(d.color, shimmer_palette(pick), "the palette pick LAST");
        assert_eq!(rng, {
            // The full 50-dust replay is the first witness's shape; here
            // only the draw-count contract: 50 × (6 internals + 4 gates
            // + 3 jitter + 1 colour) = 700 draws.
            let mut t = UnifiedRandom::new(23);
            for _ in 0..(50 * 14) {
                t.next();
            }
            t
        }, "50 × 14 draws");
    }

    /// The scale-death marker: the hook's velocity base is the KICKED
    /// velocity (the pool wrote `p.vel = kicked * 0.2`), so the burst
    /// reads it — the snapshot's `vel` field carries the result and the
    /// drain needs no branch (the marker is the hook's contract).
    #[test]
    fn w4_d6_the_scale_death_burst_reads_the_kicked_base() {
        let world = World::new("t".into(), 300, 200);
        let c = ctx(&world);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(5);
        let mut s = splash(SLIME_GUN);
        s.w = 14.0;
        s.h = 14.0;
        s.scale_dead = true;
        // The ×5 kick on (2,0) → (10,0), then the arm's ×0.2 → (2,0).
        s.vel = Vec2::new(2.0, 0.0);
        drain_one(&s, &mut dust, &mut rng, &c, &world);
        // Every dust's velocity carries the +2.0 base (pre-jitter the
        // ×0.3 internals + 2.0); the jitter can shrink it, so pin the
        // twin instead: the first dust's velocity re-derived.
        let mut twin = UnifiedRandom::new(5);
        let _ = twin.next_max(10);
        let _ = twin.next_max(10);
        let mut vx = twin.next_range(-20, 21) as f32 * 0.1;
        let mut vy = twin.next_range(-20, 21) as f32 * 0.1;
        let _ = twin.next_max(3);
        let _ = twin.next_range(-20, 21);
        for _ in 0..4 {
            let _ = twin.next_range(0, 2);
        }
        vx = vx * 0.3 + 2.0;
        vy *= 0.3;
        let mul = 1.0 + twin.next_range(-100, 101) as f32 * 0.01;
        vx *= mul;
        vy *= mul;
        vx += twin.next_range(-50, 51) as f32 * 0.015;
        vy += twin.next_range(-50, 51) as f32 * 0.015;
        let d = dust.slot(0);
        assert!((d.velocity.x - vx).abs() < 1e-4, "the kicked base 2.0, not 0.4");
        assert!((d.velocity.y - vy).abs() < 1e-4);
        let _ = DUST_SENTINEL;
    }

    /// An unclaimed kind emits nothing and draws nothing.
    #[test]
    fn w4_d6_other_kinds_stay_inert() {
        let world = World::new("t".into(), 300, 200);
        let c = ctx(&world);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        drain_one(&splash(297), &mut dust, &mut rng, &c, &world);
        assert!(!dust.slot(0).active);
        assert_eq!(rng, UnifiedRandom::new(9), "no draws");
    }
}