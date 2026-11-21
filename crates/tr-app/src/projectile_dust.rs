//! W4-e3 — the projectile dust seam (the SECOND batch of `Projectile.cs`
//! `Dust.NewDust` sites, sites 41+ of the file's line-ordered census; d53's
//! first ~40 — `:16851-22161` — stay that lane's claim, unpushed at this
//! writing). Batch 1 covers the aiStyle-2 thrown chain's head sites
//! (`:23211-23417`): the 93 puff, the 968 burst, the 162/281/240 bursts and
//! the 497 trail. Later batches add the spark window / 306 trail / 54 tail
//! (`:23541-23688`), the aiStyle-4 Vilethorn fade (`:23784-23812`), the
//! aiStyle-5 falling-star family (`:23954-24098`), the aiStyle-6 first-tick
//! burst (`:24128-24150`), the aiStyle-8 magic-bolt chain (`:24259-24336`)
//! and the STAGED aiStyle-152 emitters (`:22297-22345` — the machine is the
//! pushed W4-d16 lane's; see the section note).
//!
//! The seam rides the landed S1-E-1 pool (`tr_core::particles::DustPool`,
//! the verbatim `NewDust` family — the d50 lane's `UpdateDust` physics
//! chain is NOT a dependency): the pool threads its live `DustPool` + the
//! frame's `SpawnCtx` through `ProjectilePool::update_dust` (the
//! `Npc::update_rng_dust` precedent, `npc.rs:4622`); the plain `update`
//! runs on a scratch pool with a permissive ctx (`Npc::update_rng`,
//! `npc.rs:4595`), so the AI-level draws still ride the caller's stream
//! and the pool's own rolls land on the scratch pool — the draw COUNT
//! matches the live empty-pool path.
//!
//! RNG: every roll (the gates, the `NewDust` internals, the post-arms)
//! rides the pass's `UpdateProjectiles` stream (`Main.cs:17775-17777`'s
//! `SwapRandom("UpdateProjectiles")` wraps the whole walk, dust included)
//! in the C#'s order. The [`DustRng`] adapter maps the pool's
//! `Next`/`NextFloat` onto the pass's [`DropRng`] (`next_max(n)` =
//! `next_range(0, n)`, one draw).
//!
//! Documented skips (each at its site): the `Gore.NewGore` BODIES
//! interleaved with these dust blocks (the gore lane is d55/e5 — their
//! internal `Main.rand` rolls go with them, the `ai_volcano` precedent)
//! while their call-site TYPE/GATE argument draws stay on the stream
//! (they are AI-level draws the C# evaluates before the call), the
//! `shader` write (dropped field, `particles.rs`), and the
//! `Lighting.AddLight` arms (the light lane is c19/e6).

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};
use tr_core::World;

use crate::projectile::Projectile;

/// The pool's [`DustRng`] over the pass's [`DropRng`] stream: `Next(max)` is
/// `Next(0, max)` (`UnifiedRandom.cs:136-139`), one draw either way.
pub(crate) struct ProjDustRng<'a>(pub(crate) &'a mut dyn DropRng);

impl DustRng for ProjDustRng<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.next_range(0, max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.next_float()
    }
}

/// The per-pass dust context: the live pool (or the scratch pool), the
/// frame's `SpawnCtx` and the pass's stream. One bundle serves every armed
/// slot in the tick (the `Style44Env` precedent, `npc_ai_style44.rs:181`).
pub(crate) struct ProjDust<'a> {
    pub pool: &'a mut DustPool,
    pub ctx: &'a SpawnCtx,
    pub rng: &'a mut dyn DropRng,
}

impl ProjDust<'_> {
    /// `Dust.NewDust` (`Dust.cs:75-256`) on the pass's stream.
    #[allow(clippy::too_many_arguments)]
    fn new_dust(
        &mut self,
        world: &World,
        pos: Vec2,
        w: i32,
        h: i32,
        dust_type: i32,
        speed_x: f32,
        speed_y: f32,
        alpha: i32,
        color: Color,
        scale: f32,
    ) -> usize {
        self.pool.new_dust(
            &mut ProjDustRng(self.rng),
            self.ctx,
            world,
            pos,
            w,
            h,
            dust_type,
            speed_x,
            speed_y,
            alpha,
            color,
            scale,
        )
    }
}

/// `Utils.RotatedByRandom` (`Utils.cs:1697-1700`) — TWO `NextDouble` draws,
/// the double subtraction, then the float-cos/sin spin (`Utils.RotatedBy`,
/// `Utils.cs:1675-1684`). The draws ride `next_double` (the real `Sample()`,
/// the `bow_launch.rs:90` convention) so the f64 angle precision survives —
/// `next_float() as f64` would truncate the mantissa before the spin.
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut dyn DropRng) -> Vec2 {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    crate::bow_launch::rotated_by(v, angle)
}

// ---------------------------------------------------------------------------
// aiStyle 2 — the thrown family (`Projectile.cs:23205-23689`).
// ---------------------------------------------------------------------------

/// `:23211-23218` — the type-93 (the Poison Staff's bolt) head: the `Next(5)`
/// gate, then the dust-57 puff with `noGravity`, `fadeIn = 1` and the ×0.3
/// velocity damp. The gate draws EVERY tick the 93 flies (the C# `&&`
/// evaluates `Next(5)` first).
pub(crate) fn style2_head_93(p: &Projectile, world: &World, d: &mut ProjDust) {
    if p.kind == 93 && d.rng.next_range(0, 5) == 0 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            57,
            p.vel.x * 0.2 + p.direction as f32 * 3.0,
            p.vel.y * 0.2,
            100,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.fade_in = 1.0;
        dust.velocity.x *= 0.3;
        dust.velocity.y *= 0.3;
    }
}

/// `:23219-23236` — the 968 Pew-matic shot's one-shot 8-dust burst. The
/// caller owns the `localAI[0]` latch; this runs only on the latch tick.
/// `owner_vel` is `Main.player[owner].velocity` (the pass snapshots it; the
/// scratch path passes zero). Per dust: the scale arg's `NextFloat`, the
/// `NewDust` internals, `RotatedByRandom(0.7)`'s two draws, the velocity
/// `NextFloat`, then the post-arms.
pub(crate) fn style2_burst_968(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    owner_vel: Vec2,
) {
    // `:23225` — `num2 = Remap(ownerVel.Length(), 0, 3, 0, 0.5)`.
    let num2 = crate::projectile::remap(owner_vel.length(), 0.0, 3.0, 0.0, 0.5);
    // `:23226` — the burst anchor: centre − (8,8) + 25 px along the velocity.
    let (cx, cy) = p.center();
    let anchor = Vec2::new(cx - 8.0, cy - 8.0)
        + crate::bow_launch::safe_normalize(p.vel, Vec2::ZERO) * 25.0;
    for _ in 0..8 {
        // `:23229` — the scale arg draws its NextFloat BEFORE NewDust runs.
        let scale = 0.75 + d.rng.next_float() * 0.3;
        let idx = d.new_dust(
            world,
            anchor,
            16,
            16,
            31,
            p.vel.x,
            p.vel.y,
            50,
            Color::default(),
            scale,
        );
        // `:23230-23234` — the velocity recipe, then fadeIn/position/noGravity.
        let mut vel = rotated_by_random(p.vel * (0.2 + num2), 0.699999988079071, d.rng)
            + owner_vel * 0.25;
        vel *= d.rng.next_float();
        let dust = d.pool.slot_mut(idx);
        dust.velocity = vel;
        dust.fade_in = 0.1;
        dust.position -= dust.velocity;
        dust.no_gravity = true;
    }
}

/// `:23292-23318` / `:23337-23363` / `:23382-23408` — the 162/281/240
/// `ai[0] == 3f` burst: `n31` dust-31 puffs (×0.5 velocity + 0.1·vel), then
/// `n6` pairs (the dust-6 ×3 + 0.2·vel noGravity, the dust-6 ×2 + 0.3·vel).
/// The trailing `Gore.NewGore`'s TYPE argument (`Next(61,64)`,
/// `:23312`/`:23357`/`:23402`) is an AI-level draw and stays on the stream;
/// only the `Gore.NewGore` body's internal jitter is the gore lane's skip.
pub(crate) fn style2_burst_31_6(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    n31: i32,
    n6: i32,
) {
    for _ in 0..n31 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.5;
        dust.velocity += p.vel * 0.1;
    }
    for _ in 0..n6 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            6,
            0.0,
            0.0,
            100,
            Color::default(),
            2.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 3.0;
        dust.velocity += p.vel * 0.2;
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            6,
            0.0,
            0.0,
            100,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 2.0;
        dust.velocity += p.vel * 0.3;
    }
    // `:23312`/`:23357`/`:23402` — the trailing gore's TYPE argument draws
    // `Next(61,64)` at the call site (AI-level, stays on the stream); the
    // `Gore.NewGore` body's internal jitter is the gore lane's skip.
    let _ = d.rng.next_range(61, 64);
}

/// `:23413-23417` — the 497 Bone Throwing Knife's per-tick dust-27 trail:
/// the position blends toward the centre, `noGravity`, ×0.3 velocity minus
/// 0.1·vel. Runs EVERY tick (no gate).
pub(crate) fn style2_trail_497(p: &Projectile, world: &World, d: &mut ProjDust) {
    let idx = d.new_dust(
        world,
        p.pos,
        p.w as i32,
        p.h as i32,
        27,
        p.vel.x,
        p.vel.y,
        100,
        Color::default(),
        1.2,
    );
    let (cx, cy) = p.center();
    let dust = d.pool.slot_mut(idx);
    dust.position = (dust.position + Vec2::new(cx, cy)) / 2.0;
    dust.no_gravity = true;
    dust.velocity *= 0.3;
    dust.velocity -= p.vel * 0.1;
}
// ---------------------------------------------------------------------------
// Witness tests-as-code — WRITTEN NOT RUN (code-only mode, W4-e3 batch 1).
// Verified by re-reading the cited C# and mirroring every draw through the
// same pool call on a twin stream; run with
// `cargo test -p tr-app projectile_dust` once a compiler is available.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

    fn world() -> World {
        World::new("e3".into(), 200, 200)
    }

    /// A permissive ctx: the whole world on-screen, the full budget — the
    /// pool never culls (the `update_rng` scratch convention).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    fn proj(kind: u16) -> Projectile {
        Projectile {
            active: true,
            kind,
            pos: Vec2::new(1000.0, 900.0),
            vel: Vec2::new(4.0, 2.0),
            w: 10.0,
            h: 10.0,
            direction: 1,
            ..Default::default()
        }
    }

    /// `:23211-23218` — the 93 head: the Next(5) gate draws every tick; a
    /// passed gate spawns one dust-57 with the direction kick and the ×0.3
    /// damp. Mirrored over 20 ticks: identical records + stream position.
    #[test]
    fn w4_e3_b1_head93_gates_every_tick_and_damps() {
        let w = world();
        let c = ctx();
        let p = proj(93);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(11);
        for _ in 0..20 {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style2_head_93(&p, &w, &mut d);
            if mirror.next_range(0, 5) == 0 {
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, p.pos, 10, 10, 57,
                    p.vel.x * 0.2 + 3.0, p.vel.y * 0.2, 100, Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.no_gravity = true;
                md.fade_in = 1.0;
                md.velocity.x *= 0.3;
                md.velocity.y *= 0.3;
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(!act.is_empty(), "seed 11 passes the 1-in-5 gate in 20 ticks");
        for &i in &act {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).dust_type, 57);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).fade_in, 1.0);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // A non-93 kind draws nothing at all.
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(11);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        style2_head_93(&proj(48), &w, &mut d2);
        assert!(active(&pool2).is_empty());
        assert_eq!(rng2.next_range(0, 1000), UnifiedRandom::new(11).next_range(0, 1000));
    }

    /// `:23219-23236` — the 968 burst: 8 dusts, the scale arg's NextFloat
    /// BEFORE the pool internals, then RotatedByRandom's two NextDouble +
    /// the velocity NextFloat. The owner-speed remap clamps at 0.5 for a
    /// length-5 velocity. Mirrored record-for-record.
    #[test]
    fn w4_e3_b1_burst968_mirrors_the_argument_order() {
        let w = world();
        let c = ctx();
        let p = proj(968);
        let owner_vel = Vec2::new(3.0, 4.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(21);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(21);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style2_burst_968(&p, &w, &mut d, owner_vel);
        // Mirror: num2 = Remap(5, 0, 3, 0, 0.5) = 0.5 (clamped).
        let num2 = 0.5f32;
        let (cx, cy) = p.center();
        let anchor = Vec2::new(cx - 8.0, cy - 8.0)
            + crate::bow_launch::safe_normalize(p.vel, Vec2::ZERO) * 25.0;
        for _ in 0..8 {
            let scale = 0.75 + mirror.next_float() * 0.3;
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, anchor, 16, 16, 31,
                p.vel.x, p.vel.y, 50, Color::default(), scale,
            );
            let angle = mirror.next_double() * 0.699999988079071
                - mirror.next_double() * 0.699999988079071;
            let mut vel = crate::bow_launch::rotated_by(p.vel * (0.2 + num2), angle)
                + owner_vel * 0.25;
            vel *= mirror.next_float();
            let md = mpool.slot_mut(idx);
            md.velocity = vel;
            md.fade_in = 0.1;
            md.position -= md.velocity;
            md.no_gravity = true;
        }
        assert_eq!(active(&pool).len(), 8);
        for i in 0..8 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).fade_in, 0.1);
            assert!(pool.slot(i).no_gravity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// `:23292-23318` — the 162/281 burst (10× dust-31 + 5× dust-6 pairs =
    /// 20 dusts) and the trailing gore TYPE draw. Mirrored; the stream
    /// after the burst equals the mirror's (the Next(61,64) stays).
    #[test]
    fn w4_e3_b1_burst31_6_counts_and_keeps_the_gore_type_draw() {
        let w = world();
        let c = ctx();
        let p = proj(162);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(31);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style2_burst_31_6(&p, &w, &mut d, 10, 5);
        for _ in 0..10 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, 31,
                0.0, 0.0, 100, Color::default(), 1.5,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 0.5;
            md.velocity += p.vel * 0.1;
        }
        for _ in 0..5 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, 6,
                0.0, 0.0, 100, Color::default(), 2.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 3.0;
            md.velocity += p.vel * 0.2;
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, 6,
                0.0, 0.0, 100, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 2.0;
            md.velocity += p.vel * 0.3;
        }
        let _ = mirror.next_range(61, 64);
        assert_eq!(active(&pool).len(), 20);
        for i in 0..20 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).dust_type, mpool.slot(i).dust_type);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The 240 shell's (7, 3) shape: 7 + 3*2 = 13 dusts.
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(31);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        style2_burst_31_6(&proj(240), &w, &mut d2, 7, 3);
        assert_eq!(active(&pool2).len(), 13);
    }

    /// `:23413-23417` — the 497 trail: one dust-27 per tick, the centre
    /// blend, noGravity, ×0.3 velocity − 0.1·vel. Mirrored.
    #[test]
    fn w4_e3_b1_trail497_blends_to_the_centre() {
        let w = world();
        let c = ctx();
        let p = proj(497);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(41);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style2_trail_497(&p, &w, &mut d);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, p.pos, 10, 10, 27,
            p.vel.x, p.vel.y, 100, Color::default(), 1.2,
        );
        let (cx, cy) = p.center();
        let md = mpool.slot_mut(idx);
        md.position = (md.position + Vec2::new(cx, cy)) / 2.0;
        md.no_gravity = true;
        md.velocity *= 0.3;
        md.velocity -= p.vel * 0.1;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert!(pool.slot(0).no_gravity);
        assert_eq!(pool.slot(0).dust_type, 27);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// The off-screen cull (`Dust.cs:97-101`): a spawn outside the widened
    /// screen box returns the sentinel and consumes NO pool internals — the
    /// 497 trail (no gate) draws nothing when the ctx is far away.
    #[test]
    fn w4_e3_b1_offscreen_cull_consumes_no_internals() {
        let w = world();
        let far = SpawnCtx {
            screen_position: Vec2::new(100000.0, 100000.0),
            screen_size: Vec2::new(800.0, 600.0),
            ..ctx()
        };
        let p = proj(497);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(51);
        let mut d = ProjDust { pool: &mut pool, ctx: &far, rng: &mut rng };
        style2_trail_497(&p, &w, &mut d);
        assert!(active(&pool).is_empty(), "the cull returns the sentinel");
        assert_eq!(
            rng.next_range(0, 1000),
            UnifiedRandom::new(51).next_range(0, 1000),
            "no pool internals were drawn"
        );
    }
}
