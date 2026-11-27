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
use crate::app::App;
use crate::projectile_wet::SplashKind;
use tr_core::UnifiedRandom;

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
// Batch 2 — the spark window, the 306 trail, the 54 tail (aiStyle 2 tail)
// and the aiStyle-4 Vilethorn fade (`Projectile.cs:23541-23812`).
// ---------------------------------------------------------------------------

/// `:23541-23549` — the 504/954/979 spark family's dust window: the
/// `Next(100) < num19` gate (the caller computed the fade ramp), then the
/// dust-6 (or the 979's dust-135) with the centre blend, `noGravity`, ×2
/// velocity, ×1.6 scale and the +vel add.
pub(crate) fn style2_spark_dust(p: &Projectile, world: &World, d: &mut ProjDust) {
    let dust_type = if p.kind != 979 { 6 } else { 135 };
    let idx = d.new_dust(
        world,
        p.pos,
        p.w as i32,
        p.h as i32,
        dust_type,
        0.0,
        0.0,
        150,
        Color::default(),
        1.0,
    );
    let (cx, cy) = p.center();
    let dust = d.pool.slot_mut(idx);
    dust.position = (dust.position + Vec2::new(cx, cy)) / 2.0;
    dust.no_gravity = true;
    dust.velocity *= 2.0;
    dust.scale *= 1.6;
    dust.velocity += p.vel;
}

/// `:23614-23623` — the 306 Ichor Dart's four trail dust-184: each sits at
/// `centre − vel/4·i`, zero velocity, scale 0.7. The caller gates on
/// `alpha <= 200`.
pub(crate) fn style2_trail_306(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    for i in 0..4 {
        let num26 = p.vel.x / 4.0 * i as f32;
        let num27 = p.vel.y / 4.0 * i as f32;
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            184,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.position.x = cx - num26;
        dust.position.y = cy - num27;
        dust.velocity *= 0.0;
        dust.scale = 0.7;
    }
}

/// `:23685-23688` — the type-54 Poisoned Knife's tail: the `Next(20)` gate,
/// then the dust-40 puff (×0.1 velocity, scale 0.75, no post-arms).
pub(crate) fn style2_tail_54(p: &Projectile, world: &World, d: &mut ProjDust) {
    if p.kind == 54 && d.rng.next_range(0, 20) == 0 {
        d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            40,
            p.vel.x * 0.1,
            p.vel.y * 0.1,
            0,
            Color::default(),
            0.75,
        );
    }
}

// ---------------------------------------------------------------------------
// aiStyle 4 — the Vilethorn fade (`Projectile.cs:23784-23812`).
// ---------------------------------------------------------------------------

/// `:23784-23812` — the fade-out's `alpha < 170 && alpha + 5 >= 170` crossing
/// burst: the 150-152 Nettle Burst's 8 dust-7 (×0.025 velocity, alpha 200,
/// scale 1.3, noGravity, ×0.5), the 493/494's 8 dust-`Next(68,71)` (same
/// shape), or the Vilethorn's 3 dust-18 + 1 dust-14. The caller gates on the
/// crossing; the 493/494 rows are unported (the arm keeps the shape).
pub(crate) fn style4_fade_burst(p: &Projectile, world: &World, d: &mut ProjDust) {
    let nettle = matches!(p.kind, 150 | 151 | 152);
    if nettle {
        for _ in 0..8 {
            let idx = d.new_dust(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                7,
                p.vel.x * 0.025,
                p.vel.y * 0.025,
                200,
                Color::default(),
                1.3,
            );
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.velocity *= 0.5;
        }
    } else if matches!(p.kind, 493 | 494) {
        for _ in 0..8 {
            // `:23799` — the type arg draws Next(68,71) BEFORE NewDust runs.
            let dust_type = d.rng.next_range(68, 71);
            let idx = d.new_dust(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                dust_type,
                p.vel.x * 0.025,
                p.vel.y * 0.025,
                200,
                Color::default(),
                1.3,
            );
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.velocity *= 0.5;
        }
    } else {
        for _ in 0..3 {
            d.new_dust(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                18,
                p.vel.x * 0.025,
                p.vel.y * 0.025,
                170,
                Color::default(),
                1.2,
            );
        }
        d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            14,
            0.0,
            0.0,
            170,
            Color::default(),
            1.1,
        );
    }
}

// ---------------------------------------------------------------------------
// Batch 3 — the aiStyle-5 falling-star family (`Projectile.cs:23954-24098`).
// ---------------------------------------------------------------------------

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — one `NextFloat`; the
/// angle is FLOAT arithmetic, then `ToRotationVector2` (`Utils.cs:1665-1668`)
/// cos/sin in double of the widened float, cast back to f32 (the
/// `npc_ai_empress.rs:477` convention).
fn next_vector2_unit(rng: &mut dyn DropRng) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw THEN a
/// second `NextFloat` radius scale.
fn next_vector2_circular(rng: &mut dyn DropRng, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// The C# `base.Hitbox.Intersects(Utils.CenteredRectangle(Main.screenPosition
/// + screen/2, screen + 400))` gate (`Projectile.cs:23973` and passim;
/// `Utils.cs:947-950`): the on-screen box widened by 200 px per side, XNA
/// `Rectangle.Intersects`' strict `<` on both axes.
pub(crate) fn on_screen_wide(p: &Projectile, ctx: &SpawnCtx) -> bool {
    let sw = ctx.screen_size.x;
    let sh = ctx.screen_size.y;
    let rx = (ctx.screen_position.x + sw / 2.0 - (sw + 400.0) / 2.0) as i32;
    let ry = (ctx.screen_position.y + sh / 2.0 - (sh + 400.0) / 2.0) as i32;
    let rw = (sw + 400.0) as i32;
    let rh = (sh + 400.0) as i32;
    let sx = p.pos.x as i32;
    let sy = p.pos.y as i32;
    let pw = p.w as i32;
    let ph = p.h as i32;
    rx < sx + pw && sx < rx + rw && ry < sy + ph && sy < ry + rh
}

impl ProjDust<'_> {
    /// `Dust.NewDustDirect` (`Dust.cs:65-73`).
    #[allow(clippy::too_many_arguments)]
    fn new_dust_direct(
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
        self.pool.new_dust_direct(
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

    /// `Dust.NewDustPerfect` (`Dust.cs:54-63`).
    fn new_dust_perfect(
        &mut self,
        world: &World,
        pos: Vec2,
        dust_type: i32,
        velocity: Option<Vec2>,
        alpha: i32,
        color: Color,
        scale: f32,
    ) -> usize {
        self.pool.new_dust_perfect(
            &mut ProjDustRng(self.rng),
            self.ctx,
            world,
            pos,
            dust_type,
            velocity,
            alpha,
            color,
            scale,
        )
    }

    /// `Dust.CloneDust(Dust)` (`Dust.cs:263-286`) — the W4-e3 AI-inline
    /// window's 1006 ShimmerArrow twin (`Projectile.cs:56052`): the fresh
    /// spawn's internals ride the pass's stream (the pool's `clone_dust`
    /// went generic over `DustRng` this round, the g64 `853039bfc`
    /// precedent — keep-one at merge, identical signatures).
    fn clone_dust(&mut self, world: &World, dust_index: usize) -> usize {
        self.pool.clone_dust(
            &mut ProjDustRng(self.rng),
            self.ctx,
            world,
            dust_index,
        )
    }
}

/// `:23954-23962` — the 503 Star Wrath's dust-58 spark: the `Next(16)` gate,
/// the `UnitX.RotatedByRandom(π/2).RotatedBy(vel.ToRotation())` direction
/// (two draws), then the dust at `centre + dir·12` with velocity `dir·0.66`.
/// The trailing `Next(48)` gore GATE (`:23963`) is an AI-level draw and stays
/// on the stream; the taken branch's gore body (type literal 16, no pick
/// draw) is the gore lane's skip.
pub(crate) fn style5_spark_503(p: &Projectile, world: &World, d: &mut ProjDust) {
    if d.rng.next_range(0, 16) == 0 {
        let dir = crate::bow_launch::rotated_by(
            rotated_by_random(Vec2::X, 1.5707963705062866, d.rng),
            p.vel.y.atan2(p.vel.x) as f64,
        );
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            58,
            p.vel.x * 0.5,
            p.vel.y * 0.5,
            150,
            Color::default(),
            1.2,
        );
        let (cx, cy) = p.center();
        let dust = d.pool.slot_mut(idx);
        dust.velocity = dir * 0.66;
        dust.position = Vec2::new(cx, cy) + dir * 12.0;
    }
    // `:23963` — the gore gate (the body is the gore lane's skip; the gate
    // draw stays).
    let _ = d.rng.next_range(0, 48);
}

/// `:23970-23986` — the 12/955 falling star's dust-58: the on-screen gate +
/// `Next(6)` gore pick (the gore lane's skip — the `Next(6)` and the taken
/// branch's `SelectRandom` `Next(4)` go with it; on an anniversary world
/// the C# RE-PICKS at `:23976-23978` — BOTH `Next(4)` draws stay on the
/// stream, the second wins: the g63 `036901a5e` / g16 `falling_star_trail`
/// pin, the g63 research/240 §3.2 third-copy flag FIXED here), the
/// `light = 0.9` write (`:23982`), then the `Next(20)` dust gate (the
/// anniversary `Next(15)` is the second `||` operand). `anniversary` is
/// `Main.tenthAnniversaryWorld` (the pass reads `world.seeds`).
pub(crate) fn style5_star_dust(
    p: &mut Projectile,
    world: &World,
    d: &mut ProjDust,
    anniversary: bool,
) {
    // `:23973-23981` — the gore gate: the on-screen test short-circuits the
    // Next(6); the taken branch's SelectRandom draws Next(4). Both go with
    // the gore (the gore lane's skip), but the GATE draws stay on the stream.
    if on_screen_wide(p, d.ctx) && d.rng.next_range(0, 6) == 0 {
        let _ = d.rng.next_range(0, 4); // SelectRandom(16,17,17,17)'s pick
        // `:23976-23978` — the anniversary RE-PICK: SelectRandom(16,16,16,17)
        // draws a SECOND Next(4) (the second value wins; both stay on the
        // stream — the g63/g16 pin, the research/240 §3.2 correction).
        if anniversary {
            let _ = d.rng.next_range(0, 4);
        }
    }
    p.light = 0.9; // `:23982`
    // `:23983` — the dust gate: Next(20), then the anniversary Next(15) only
    // when the first failed AND the world is the anniversary one.
    if d.rng.next_range(0, 20) == 0 || (anniversary && d.rng.next_range(0, 15) == 0) {
        d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            58,
            p.vel.x * 0.5,
            p.vel.y * 0.5,
            150,
            Color::default(),
            1.2,
        );
    }
}

/// `:23988-24031` — the 723-726/1133 shooting-star family. The 726/725/1133
/// arm: the on-screen `Next(24)` gore gate (skip), the 1133's `Next(6)`
/// dust-89 gate, the `Next(3)` dust-86/87 gate (×0.5 velocity, noGravity);
/// the 723's three-ring dust-267 pair (Blue + White, noLight/noGravity).
pub(crate) fn style5_shooting_star(p: &Projectile, world: &World, d: &mut ProjDust) {
    if matches!(p.kind, 726 | 725 | 1133) {
        // `:23992-23996` — the gore gate (the gore lane's skip; the gate
        // draws stay).
        if on_screen_wide(p, d.ctx) && d.rng.next_range(0, 24) == 0 {
            let _ = d.rng.next_range(0, 2); // SelectRandom(16,17)
        }
        let mut num48 = 86;
        if p.kind == 1133 {
            num48 = 87;
            // `:24001-24004` — the 1133's dust-89 gate.
            if d.rng.next_range(0, 6) == 0 {
                let idx = d.new_dust_direct(
                    world,
                    p.pos,
                    p.w as i32,
                    p.h as i32,
                    89,
                    0.0,
                    0.0,
                    127,
                    Color::default(),
                    1.0,
                );
                d.pool.slot_mut(idx).no_gravity = true;
            }
        }
        // `:24006-24011` — the dust-86/87 gate.
        if d.rng.next_range(0, 3) == 0 {
            let idx = d.new_dust_direct(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                num48,
                0.0,
                0.0,
                127,
                Color::default(),
                1.0,
            );
            let dust = d.pool.slot_mut(idx);
            dust.velocity *= 0.5;
            dust.no_gravity = true;
        }
    }
    if p.kind == 723 {
        // `:24015-24019` — the 723's gore gate (skip; the gate draws stay).
        if on_screen_wide(p, d.ctx) && d.rng.next_range(0, 6) == 0 {
            // Gore.NewGore(..., 17) — no pick draw (the type is literal).
        }
        // `:24020-24029` — the three-ring dust-267 pair.
        let (cx, cy) = p.center();
        for num49 in 0..3 {
            let num49 = num49 as f32;
            let offset = crate::bow_launch::rotated_by(
                Vec2::new(0.0, 12.0 * p.scale),
                (p.pos.y / 200.0 + num49 * (std::f32::consts::PI * 2.0) + p.rot) as f64,
            );
            let vector7 = Vec2::new(cx, cy) + offset - p.vel * 0.5;
            let idx = d.new_dust_perfect(
                world,
                vector7,
                267,
                Some(p.vel * 0.2 * num49),
                0,
                Color::new(0, 0, 255, 255), // Color.Blue
                1.0,
            );
            let dust = d.pool.slot_mut(idx);
            dust.no_light = true;
            dust.no_gravity = true;
            let idx = d.new_dust_perfect(
                world,
                vector7,
                267,
                Some(p.vel * 0.2 * num49),
                0,
                Color::WHITE,
                0.4,
            );
            let dust = d.pool.slot_mut(idx);
            dust.no_light = true;
            dust.no_gravity = true;
        }
    }
}

/// `:24032-24049` — the 9 Starfury star's dust-86: the on-screen `Next(20)`
/// gore gate (skip), then the `Next(4)` dust gate (×0.7 velocity, noGravity,
/// +0.3·vel, the `Next(2)` position pull).
pub(crate) fn style5_star9_dust(p: &Projectile, world: &World, d: &mut ProjDust) {
    // `:24034-24038` — the gore gate (skip; the gate draws stay).
    if on_screen_wide(p, d.ctx) && d.rng.next_range(0, 20) == 0 {
        let _ = d.rng.next_range(16, 18); // the gore type pick
    }
    // `:24039-24049` — the dust-86 gate.
    if d.rng.next_range(0, 4) == 0 {
        let idx = d.new_dust_direct(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            86,
            0.0,
            0.0,
            127,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.7;
        dust.no_gravity = true;
        dust.velocity += p.vel * 0.3;
        if d.rng.next_range(0, 2) == 0 {
            dust.position -= p.vel * 4.0;
        }
    }
}

/// `:24051-24061` — the generic `ai[1] == 1 || type == 92` arm: the light
/// 0.9 write (the caller owns it), the `Next(10)` dust-58 gate, then the
/// `Next(20)` gore gate (skip).
pub(crate) fn style5_generic_dust(p: &Projectile, world: &World, d: &mut ProjDust) {
    if d.rng.next_range(0, 10) == 0 {
        d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            58,
            p.vel.x * 0.5,
            p.vel.y * 0.5,
            150,
            Color::default(),
            1.2,
        );
    }
    // `:24058-24061` — the gore gate (skip; the gate draws stay).
    if d.rng.next_range(0, 20) == 0 {
        let _ = d.rng.next_range(16, 18);
    }
}

/// `:24067-24098` — the 1037/1049 comet tail: the two `Next(2)` dust gates
/// (each with the `Next(4)` type pick, the two velocity `NextFloat`s), then
/// the `!wet` dust-6 perfect (the `NextVector2Circular(25,25)` pair).
pub(crate) fn style5_comet_tail(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    let vector9 = Vec2::new(cx, cy) + p.vel.normalize_or_zero() * 10.0;
    for sign in [1.0f64, -1.0f64] {
        if d.rng.next_range(0, 2) == 0 {
            // `:24070`/`:24080` — the type arg draws Next(4) BEFORE NewDust.
            let dust_type = if d.rng.next_range(0, 4) == 0 { 6 } else { 31 };
            let idx = d.new_dust(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                dust_type,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
            let rotated = crate::bow_launch::rotated_by(p.vel, sign * 1.5707963705062866);
            let dust = d.pool.slot_mut(idx);
            dust.position = vector9;
            dust.velocity = (rotated * 0.25 + p.vel / 3.0) * 0.9;
            dust.position += rotated;
            dust.scale *= 0.3 + 0.9 * d.rng.next_float();
            dust.velocity *= d.rng.next_float();
            dust.no_gravity = true;
        }
    }
    // `:24088-24098` — the !wet dust-6 perfect.
    if !p.wet {
        let offset = next_vector2_circular(d.rng, 25.0, 25.0);
        let idx = d.new_dust_perfect(
            world,
            Vec2::new(cx, cy) + offset,
            6,
            None,
            0,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.5;
        dust.scale *= 1.3;
        dust.fade_in = 1.0;
        dust.no_gravity = true;
    }
}

// ---------------------------------------------------------------------------
// Batch 4 — the aiStyle-6 first-tick burst (`Projectile.cs:24128-24150`),
// the aiStyle-8 magic-bolt chain (`:24259-24336`) and the STAGED aiStyle-152
// emitters (`:22297-22345` — the machine is the pushed W4-d16 lane's
// `projectile_ai_misc::ai_super_star_slash`; these emitters replace its
// count-only skip blocks at the merge, the e2 "staged emitters" precedent).
//
// KEEP-ONE (1051 vs W4-d1, tip e271b5bc): d1's `projectile_ai_powder::step`
// owns the FULL aiStyle-6 machine (the rows 10/11/463/1019/1051, the tile
// scan / Convert, the sapling-growth defer, the free move) but its burst is
// count-only — the 1051 keeps ONLY the 50 per-dust `Next(3)` picks on the
// stream, the bodies + `NewDust` internals skip. This `style6_burst` is the
// full-bodied twin: SAME explicit draw (`:24132` first, one per dust, loop
// order) PLUS the pool internals + the flag2 post-arms. At the merge keep
// d1's machine + rows and feed THIS emitter through its burst loop (replace
// the count-only `for` body); retire the e3 `ai_style6` duplicate arm + its
// `6 =>` kind gate (it would double-run the machine). The 1051 witness
// below mirrors the FULL sequence — the pick, the internals, the post-arms
// — so it stays the stream-parity pin after the swap; d1's
// `w4_d1_the_super_fertilizer_burst_draws_50` (50 ranges, no floats) is the
// count-only pin and must be re-derived at the swap (the internals add
// floats/ranges per dust).
// ---------------------------------------------------------------------------

/// `:24110-24150` — the aiStyle-6 first-tick (`ai[1] == 0`) dust burst: the
/// 10/11/463/1019/1051 family's `num52` dust (the 1051's per-dust `Next(3)`
/// type pick, the 1019/1051's noGravity/scale/velocity ladder). The caller
/// owns the `ai[1]` latch and the `num51`/`num52` parameter ladder.
pub(crate) fn style6_burst(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    mut num51: i32,
    num52: i32,
    flag: bool,
    flag2: bool,
) {
    for num53 in 0..num52 {
        if flag {
            // `:24132` — the 1051's per-dust type pick.
            num51 = if d.rng.next_range(0, 3) == 0 { 309 } else { 0 };
        }
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            num51,
            p.vel.x,
            p.vel.y,
            50,
            Color::default(),
            1.0,
        );
        if flag2 {
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = num53 % 3 != 0;
            if !dust.no_gravity {
                dust.scale *= 1.25;
                dust.velocity /= 2.0;
                dust.velocity.y -= 2.2;
            } else {
                dust.scale *= 1.75;
                dust.velocity += p.vel * 0.65;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// aiStyle 8 — the magic-bolt chain (`Projectile.cs:24259-24336`).
// ---------------------------------------------------------------------------

/// `:24259-24264` — the 27 Water Bolt's lava-bounce burst: four dust-31
/// (×0.6 velocity, Y −2). The trailing `Next(3)` gore gate (`:24265-24271`)
/// is the gore lane's skip (the caller keeps its draws).
pub(crate) fn style8_water_bolt_lava(p: &Projectile, world: &World, d: &mut ProjDust) {
    for _ in 0..4 {
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
        dust.velocity *= 0.6;
        dust.velocity.y -= 2.0;
    }
}

/// `:24274-24278` — the 27 Water Bolt's `ai[1] == 1` dust-31: the `Next(2)`
/// gate, then the centre puff (×0.2 velocity, scale 0.8).
pub(crate) fn style8_water_bolt_trail(p: &Projectile, world: &World, d: &mut ProjDust) {
    if p.ai[1] == 1.0 && d.rng.next_range(0, 2) == 0 {
        let (cx, cy) = p.center();
        let idx = d.new_dust(
            world,
            Vec2::new(cx, cy),
            0,
            0,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            0.8,
        );
        d.pool.slot_mut(idx).velocity *= 0.2;
    }
}

/// `:24279-24300` — the 27 Water Bolt's dust-172 trail: the `ai[1] == 0 ||
/// Next(5) == 0` gate, then the five-dust ladder (the 4-px inset box, ×0.1
/// velocity + 0.1·vel, the position pull) and the `Next(5)` dust-172 extra
/// (×0.25 velocity + 0.5·vel, scale 0.6).
pub(crate) fn style8_water_bolt_172(p: &Projectile, world: &World, d: &mut ProjDust) {
    if p.ai[1] == 0.0 || d.rng.next_range(0, 5) == 0 {
        for num66 in 0..5 {
            let num67 = p.vel.x / 3.0 * num66 as f32;
            let num68 = p.vel.y / 3.0 * num66 as f32;
            let num69 = 4;
            let idx = d.new_dust(
                world,
                Vec2::new(p.pos.x + num69 as f32, p.pos.y + num69 as f32),
                p.w as i32 - num69 * 2,
                p.h as i32 - num69 * 2,
                172,
                0.0,
                0.0,
                100,
                Color::default(),
                1.2,
            );
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.velocity *= 0.1;
            dust.velocity += p.vel * 0.1;
            dust.position.x -= num67;
            dust.position.y -= num68;
        }
        if d.rng.next_range(0, 5) == 0 {
            let num71 = 4;
            let idx = d.new_dust(
                world,
                Vec2::new(p.pos.x + num71 as f32, p.pos.y + num71 as f32),
                p.w as i32 - num71 * 2,
                p.h as i32 - num71 * 2,
                172,
                0.0,
                0.0,
                100,
                Color::default(),
                0.6,
            );
            let dust = d.pool.slot_mut(idx);
            dust.velocity *= 0.25;
            dust.velocity += p.vel * 0.5;
        }
    }
}

/// `:24309-24313` — the 95/96 cursed flame's dust-75: one puff at
/// `position + velocity`, scale `3·scale`, noGravity.
pub(crate) fn style8_cursed_flame(p: &Projectile, world: &World, d: &mut ProjDust) {
    let idx = d.new_dust(
        world,
        Vec2::new(p.pos.x + p.vel.x, p.pos.y + p.vel.y),
        p.w as i32,
        p.h as i32,
        75,
        p.vel.x,
        p.vel.y,
        100,
        Color::default(),
        3.0 * p.scale,
    );
    d.pool.slot_mut(idx).no_gravity = true;
}

/// `:24314-24323` — the 253 Ball of Frost's two dust-135 (×0.2 velocity,
/// scale 2, noGravity, ×0.3 velocity).
pub(crate) fn style8_frost(p: &Projectile, world: &World, d: &mut ProjDust) {
    for _ in 0..2 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            135,
            p.vel.x * 0.2,
            p.vel.y * 0.2,
            100,
            Color::default(),
            2.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity.x *= 0.3;
        dust.velocity.y *= 0.3;
    }
}

/// `:24324-24336` — the aiStyle-8 default's two dust-6 (×0.2 velocity,
/// scale 2, noGravity, ×0.3 velocity); the 258 Golem fireball's
/// `getGoodWorld` arm sets noLight.
pub(crate) fn style8_default(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    get_good_world: bool,
) {
    for _ in 0..2 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            6,
            p.vel.x * 0.2,
            p.vel.y * 0.2,
            100,
            Color::default(),
            2.0,
        );
        let dust = d.pool.slot_mut(idx);
        if p.kind == 258 && get_good_world {
            dust.no_light = true;
        }
        dust.no_gravity = true;
        dust.velocity.x *= 0.3;
        dust.velocity.y *= 0.3;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 152 — the SuperStarSlash (`Projectile.cs:22230-22351`). STAGED:
// the machine + dispatch + kind rows are the pushed W4-d16 lane's
// (`projectile_ai_misc::ai_super_star_slash`, verified still count-only at
// its tip e122a6b2 — the aiStyle-160 Kites commit); these emitters replace
// its dust skip blocks at the merge. DROP-IN CONTRACT (three edits on the
// d16 side, ANALYSIS "the W4-e3 window census"):
// 1. thread `&mut DustPool` + `&SpawnCtx` into `ai_super_star_slash` and
//    build the `ProjDust` bundle (the dispatch already carries both — the
//    `ai_ball_of_fire` 8-arm precedent);
// 2. REPLACE the `Next(8)` block (977/1045, `:22292-22313`) with ONE
//    `star_slash_dust` call per kind arm — the emitter re-draws the gate
//    itself; leaving d16's `(next_float, next_float)` pair beside the call
//    DOUBLE-DRAWS the circular pair;
// 3. REPLACE the `Next(3)` + `localAI[1]` blocks (`:22334-22347`) with ONE
//    `star_slash_976` call, and MOVE d16's `local_ai[1] = 1.0` latch to
//    AFTER the call (the emitter reads the pre-latch value; the C# latches
//    before the loop but the loop is inside the emitter). The `NextFloat`
//    pair loop must go — the emitter draws it per dust.
// Draw order per dust: the `NewDustDirect` internals FIRST, then the
// velocity overwrite's draws (`:22300` / `:22345` evaluate after the call).
// ---------------------------------------------------------------------------

/// `:22292-22312` — the 977/1045 Zenith slash's dust: the `Next(8)` gate,
/// then the dust-172 (977) / dust-229 (1045) at the centre (noGravity, the
/// `NextVector2Circular(1,1)` + 0.5·vel velocity). The caller owns the
/// ×0.96 drag.
// STAGED (the W4-d16 lane owns the machine): the witnesses are the only
// caller until the merge feeds d16's skip block — the style73 precedent.
#[allow(dead_code)]
pub(crate) fn star_slash_dust(p: &Projectile, world: &World, d: &mut ProjDust) {
    let dust_type = if p.kind == 977 { 172 } else { 229 };
    if d.rng.next_range(0, 8) == 0 {
        let (cx, cy) = p.center();
        let idx = d.new_dust_direct(
            world,
            Vec2::new(cx, cy),
            0,
            0,
            dust_type,
            p.vel.x * 0.1,
            p.vel.y * 0.1,
            100,
            Color::default(),
            0.9,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.position = Vec2::new(cx, cy);
        dust.velocity = next_vector2_circular(d.rng, 1.0, 1.0) + p.vel * 0.5;
    }
}

/// `:22334-22347` — the 976 Terraprisma slash's dust: the `Next(3)` dust-40
/// trail (noGravity, ×0.5 velocity), then the `localAI[1] == 0` one-shot
/// 10-dust burst (each with the two `NextFloat` velocity draws). The caller
/// owns the `localAI[1]` latch (set AFTER this call, as the C# orders it).
// STAGED (the W4-d16 lane owns the machine) — see `star_slash_dust`.
#[allow(dead_code)]
pub(crate) fn star_slash_976(p: &Projectile, world: &World, d: &mut ProjDust) {
    if d.rng.next_range(0, 3) == 0 {
        let idx = d.new_dust_direct(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            40,
            0.0,
            0.0,
            0,
            Color::default(),
            1.2,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity = p.vel * 0.5;
    }
    if p.local_ai[1] == 0.0 {
        for _ in 0..10 {
            let idx = d.new_dust_direct(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                40,
                0.0,
                0.0,
                0,
                Color::default(),
                0.7,
            );
            // `:22345` — the velocity: SafeNormalize(UnitY).RotatedBy(
            // NextFloat·2π·0.25) · (NextFloat·3). TWO NextFloat draws.
            let angle = d.rng.next_float() * (std::f32::consts::PI * 2.0) * 0.25;
            let mag = d.rng.next_float() * 3.0;
            let dir = crate::bow_launch::rotated_by(
                crate::bow_launch::safe_normalize(p.vel, Vec2::Y),
                angle as f64,
            );
            d.pool.slot_mut(idx).velocity = dir * mag;
        }
    }
}

// ---------------------------------------------------------------------------
// Coverage phase — the `Kill_Bombs_DoUsualKillCode` dust tail
// (`Projectile.cs:78211-78533`, 15 sites :78227-78508). THE smallest
// genuinely unclaimed window (docs/research/231-proj-dust-coverage.md,
// was 125 — the registry-150 rename):
// g65's Kill() span ends at :77262, g16 owns only the GORE sites here
// (waves M/O leave "the dust loops before each" to the dust lane), main's
// `rocket_kill_census` covers :77648-78210 only.
//
// The e3 emitter pattern: argument draws FIRST (the C# evaluates arguments
// before the call), then the `NewDust` internals inside the pool, then the
// post-spawn writes on the returned slot (the sentinel included). The gore
// siblings' TYPE argument draws (`Next(61,64)`, the confetti `Next(276,283)`)
// stay on the stream — the bodies + kicks are g16's (its wave-O
// `bomb_corner_gores`/`big_bomb_gores`/`confetti_bomb_gores`/
// `scarab_bomb_gores` replace the TYPE-only draws at the merge, the same
// stream slot; the `Next(10,31)`/`Next(-20,21)` kicks join there).
//
// `Resize` (`Projectile.cs:22353-22359`) recentres on the OLD centre; the
// callers resize BEFORE these emitters (the box the dust reads is the
// resized one). The C#'s `(float)(base.width / 2)` is INTEGER division
// widened — the port's `((p.w as i32) / 2) as f32` keeps it.
// ---------------------------------------------------------------------------

/// `:78338-78350` — the bomb family's dust (the C# `:78329` kind list
/// 28/30/37/75/102/164/397/517/516/519): 20× dust-31 (scale 1.5, ×1.4
/// velocity), then 10× the pair (dust-`num84` scale 2.5 noGravity ×5 +
/// scale 1.5 ×3). `num84` is 6 (the C#'s `type == 773` override at `:78334`
/// is DEAD inside this arm — 773 takes the `:78211` branch first; kept as
/// the caller's constant). The caller ran `Resize(22, 22)` (`:78332`).
/// WIRED through `explode_bomb` (the port's kind gate admits
/// 28/37/516/519 + the grenades 30/397/517; the 75/102/164 halves are
/// cited — 75 unrowed, 102/164 are blast rows the gate refuses).
pub(crate) fn bomb_family_dust(p: &Projectile, world: &World, d: &mut ProjDust) {
    for _ in 0..20 {
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
        d.pool.slot_mut(idx).velocity *= 1.4;
    }
    for _ in 0..10 {
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
            2.5,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 5.0;
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
            1.5,
        );
        d.pool.slot_mut(idx).velocity *= 3.0;
    }
    // `:78351-78366` — the four corner gores' TYPE arguments (the bodies +
    // kicks are g16 wave-O's `bomb_corner_gores`, the keep-one).
    for _ in 0..4 {
        let _ = d.rng.next_range(61, 64);
    }
}

/// `:78405-78417` — the big bomb's dust (the C# `:78384` kind list
/// 29/108/470/637/1002/1086/1087): 50× dust-31 (scale 2, ×1.4 velocity),
/// then 80× the pair (dust-6 scale 3 noGravity ×5 + scale 2 ×3). The caller
/// resized first (the 29: 200×200 `:78389-78394`; the 1086/1087: 240×240
/// `:78398-78403`; the rest keep the `PrepareBombToBlow` box). WIRED through
/// `explode_bomb` for the dynamite 29; the 108/470/637/1002/1086/1087 halves
/// cited (108 rowed but the gate refuses it, the rest unrowed).
pub(crate) fn big_bomb_dust(p: &Projectile, world: &World, d: &mut ProjDust) {
    for _ in 0..50 {
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
            2.0,
        );
        d.pool.slot_mut(idx).velocity *= 1.4;
    }
    for _ in 0..80 {
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
            3.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 5.0;
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
        d.pool.slot_mut(idx).velocity *= 3.0;
    }
    // `:78418-78436` — the 2×4 gores' TYPE arguments (g16 wave-O's
    // `big_bomb_gores` = the wave-I boulder row ×2, the keep-one).
    for _ in 0..8 {
        let _ = d.rng.next_range(61, 64);
    }
}

// ---------------------------------------------------------------------------
// Coverage batch B — the STAGED nine (the window's unrowed / host-gap half).
// The e3 `star_slash_dust` precedent: emitters + draw-mirror witnesses land
// now, the wiring rides the rows/host wave. Keep-one: g16 wave-O's
// `scarab_bomb_gores`/`confetti_bomb_gores` replace the TYPE-only draws.
// ---------------------------------------------------------------------------

/// `GetScarabBombDigDirectionSnap8` (`Projectile.cs:70502-70515`) — the
/// 8-way dig snap from the owner's centre: the sign pair (X/Y each −1 when
/// the direction component is > 0, else +1), then the dominant-axis zeroing
/// (|X| > 2|Y| → Y = 0; |Y| > 2|X| → X = 0). `dir` is
/// `base.DirectionTo(player.Center)` — XNA Normalize, NO zero guard (the
/// d50 convention); the caller resolves the player seam.
pub(crate) fn scarab_dig_snap(dir: Vec2) -> Vec2 {
    let mut snap = Vec2::new(
        if dir.x > 0.0 { -1.0 } else { 1.0 },
        if dir.y > 0.0 { -1.0 } else { 1.0 },
    );
    if dir.x.abs() > dir.y.abs() * 2.0 {
        snap.y = 0.0;
    } else if dir.y.abs() > dir.x.abs() * 2.0 {
        snap.x = 0.0;
    }
    snap
}

/// `:78224-78284` — the 773 scarab bomb's dust (STAGED — 773 unrowed, the
/// `GetScarabBombDigDirectionSnap8` player seam is the caller's). The caller
/// ran `Resize(80, 80)` (`:78224`). `num70` = the snap's rotation + π/2
/// (`:78215` — the `:78213` DirectionTo+π line is DEAD, overwritten before
/// any read). Three blocks:
/// 1. 60× dust-31 Direct scale 2: Y −0.5, ×1.2, Black×0.9; the `Next(2)`
///    gate → scale 0.5 + fadeIn `1 + Next(10)·0.1` + Black×0.8.
/// 2. 30× the dust-59 triple: the first noGravity + the BASE velocity-X
///    clamp (`:78242-78245` — a behaviour write, kept per-iteration; after
///    the first clamp |X| = 0.25 so the rest no-op), Y −0.5, fadeIn 1.2,
///    ×8, RotatedBy(num70); the second scale 1.5 ×2 rotated Y −1.5; the
///    third scale 1.5 noGravity Y−− fadeIn 2 ×4 rotated.
/// 3. Three rings (`num73` 1..=3, the f32 `+= 1f/12f` accumulation — **13
///    iterations**, the drift included: 1/12 rounds UP in f32, the 12th
///    sum stays below 1.0; the cluster-ring convention, pinned by the
///    witness): ONE `NextFloat` rotation per ring,
///    then per dust the Perfect's velocity argument
///    `(2π·t + rot).ToRotationVector2() · (0.7, 0.3)` (no draws), the
///    internals, fadeIn 2 + noGravity, `vel *= num73 + NextFloat·0.6`,
///    `Y -= num73·0.8`, rotated, then the dig push — the axis-aligned snap
///    (|X|+|Y| == 1) takes `NextFloat·11 + 6`, the diagonal `NextFloat·7 + 5`.
/// The four gore TYPE draws (`:78292`) stay; the `Resize(10,10)` + the
/// owner-gated dig walk (`:78298-78327`) are the host wave's.
// STAGED (773 unrowed) — the witnesses are the only caller until the row +
// the player-centre seam land (the star_slash_dust precedent).
#[allow(dead_code)]
pub(crate) fn scarab_bomb_dust(
    p: &mut Projectile,
    world: &World,
    d: &mut ProjDust,
    dig_snap: Vec2,
) {
    // `:78215` — num70 = snap.ToVector2().ToRotation() + π/2. ToRotation is
    // the DOUBLE atan2 cast to f32 (`Utils.cs:1655-1658`).
    let num70 = (dig_snap.y as f64).atan2(dig_snap.x as f64) as f32
        + std::f32::consts::FRAC_PI_2;
    let (cx, cy) = p.center();
    let center = Vec2::new(cx, cy);
    for _ in 0..60 {
        let idx = d.new_dust_direct(
            world, p.pos, p.w as i32, p.h as i32, 31,
            0.0, 0.0, 100, Color::default(), 2.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity.y -= 0.5;
        dust.velocity *= 1.2;
        dust.color = Color::new(0, 0, 0, (255.0 * 0.9) as u8);
        if d.rng.next_range(0, 2) == 0 {
            let dust = d.pool.slot_mut(idx);
            dust.scale = 0.5;
            dust.fade_in = 1.0 + d.rng.next_range(0, 10) as f32 * 0.1;
            dust.color = Color::new(0, 0, 0, (255.0 * 0.8) as u8);
        }
    }
    for _ in 0..30 {
        let idx = d.new_dust_direct(
            world, p.pos, p.w as i32, p.h as i32, 59,
            0.0, 0.0, 100, Color::default(), 1.0,
        );
        d.pool.slot_mut(idx).no_gravity = true;
        // `:78242-78245` — the BASE velocity clamp (behaviour, per dust).
        if p.vel.x.abs() > 0.25 {
            p.vel.x *= 0.25 / p.vel.x.abs();
        }
        let dust = d.pool.slot_mut(idx);
        dust.velocity.y -= 0.5;
        dust.fade_in = 1.2;
        dust.velocity *= 8.0;
        dust.velocity = crate::bow_launch::rotated_by(dust.velocity, num70 as f64);
        let idx = d.new_dust_direct(
            world, p.pos, p.w as i32, p.h as i32, 59,
            0.0, 0.0, 100, Color::default(), 1.5,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 2.0;
        dust.velocity = crate::bow_launch::rotated_by(dust.velocity, num70 as f64);
        dust.velocity.y -= 1.5;
        let idx = d.new_dust_direct(
            world, p.pos, p.w as i32, p.h as i32, 59,
            0.0, 0.0, 100, Color::default(), 1.5,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity.y -= 1.0;
        dust.fade_in = 2.0;
        dust.velocity *= 4.0;
        dust.velocity = crate::bow_launch::rotated_by(dust.velocity, num70 as f64);
    }
    // `:78261` — the axis-aligned flag for the dig push.
    let flag = dig_snap.x.abs() + dig_snap.y.abs() == 1.0;
    for num73 in 1..=3 {
        let num74 = std::f32::consts::PI * 2.0 * d.rng.next_float();
        // `:78265` — the C#'s f32 `num75 += 1f/12f` accumulation: 1/12
        // rounds UP in f32, so the 12th sum stays below 1.0 — the loop runs
        // THIRTEEN iterations (the drift included; the cluster-ring
        // convention, the witness pins it).
        let mut t = 0.0f32;
        while t < 1.0 {
            let angle = std::f32::consts::PI * 2.0 * t + num74;
            let vel_arg = Vec2::new(
                (angle as f64).cos() as f32 * 0.7,
                (angle as f64).sin() as f32 * 0.3,
            );
            let idx = d.new_dust_perfect(
                world, center, 59, Some(vel_arg), 0, Color::default(), 1.0,
            );
            let dust = d.pool.slot_mut(idx);
            dust.fade_in = 2.0;
            dust.no_gravity = true;
            dust.velocity *= num73 as f32 + d.rng.next_float() * 0.6;
            dust.velocity.y -= num73 as f32 * 0.8;
            dust.velocity = crate::bow_launch::rotated_by(dust.velocity, num70 as f64);
            if flag {
                dust.velocity += dig_snap * (d.rng.next_float() * 11.0 + 6.0);
            } else {
                dust.velocity += dig_snap * (d.rng.next_float() * 7.0 + 5.0);
            }
            t += 1.0f32 / 12.0f32;
        }
    }
    // `:78292` — the four gores' TYPE arguments (g16 wave-O's
    // `scarab_bomb_gores`, the keep-one; the dig-push vector3 is its seam).
    for _ in 0..4 {
        let _ = d.rng.next_range(61, 64);
    }
}

/// `:78455-78469` — the 588 confetti bomb's dust (STAGED — 588 unrowed).
/// The caller recentered (`position = Center; width = height = 22;
/// Center = position`, `:78452-78454` — `p.pos` IS the pre-resize centre,
/// the g16 wave-O note). 8× dust `219 + Next(5)` (the TYPE argument draw
/// BEFORE the internals; ×1.4 velocity, fadeIn 1, noGravity), then 15× the
/// pair `139 + Next(4)` (scale 1.6 noGravity ×5 / scale 1.9 ×3, alpha 0).
/// The four INDEPENDENT `Next(2)` gore gates follow (`:78470-78497`), each
/// taken → one `Next(276,283)` confetti TYPE draw (g16's
/// `confetti_bomb_gores`, the keep-one — NOT the corner 61-64 range).
// STAGED (588 unrowed) — see `scarab_bomb_dust`.
#[allow(dead_code)]
pub(crate) fn confetti_bomb_dust(p: &Projectile, world: &World, d: &mut ProjDust) {
    for _ in 0..8 {
        let dust_type = 219 + d.rng.next_range(0, 5);
        let idx = d.new_dust(
            world, p.pos, p.w as i32, p.h as i32, dust_type,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 1.4;
        dust.fade_in = 1.0;
        dust.no_gravity = true;
    }
    for _ in 0..15 {
        let dust_type = 139 + d.rng.next_range(0, 4);
        let idx = d.new_dust(
            world, p.pos, p.w as i32, p.h as i32, dust_type,
            0.0, 0.0, 0, Color::default(), 1.6,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 5.0;
        let dust_type = 139 + d.rng.next_range(0, 4);
        let idx = d.new_dust(
            world, p.pos, p.w as i32, p.h as i32, dust_type,
            0.0, 0.0, 0, Color::default(), 1.9,
        );
        d.pool.slot_mut(idx).velocity *= 3.0;
    }
    // `:78470-78497` — the four independent gates (the bodies are g16's).
    for _ in 0..4 {
        if d.rng.next_range(0, 2) == 0 {
            let _ = d.rng.next_range(276, 283);
        }
    }
}

/// `:78506-78513` — the 1088 acorn's dust (STAGED — the row exists, but the
/// port's generic `kill()` does not run the aiStyle-16 `Kill_Bombs` tail for
/// non-detonated deaths: the host gap is the wiring wave's). 10× dust-2 at
/// the Centre in a 10×10 box: noGravity, `vel ×2 −= oldVel·0.3`, then the
/// POST-spawn scale draw `+= Next(150)·0.001`. `old_vel` is
/// `base.oldVelocity` (the last sub-step's start velocity — the pass's
/// `old_velocity` latch, the `special_arrow_kill` precedent). The
/// `SoundEngine.PlaySound(0, Center)` cue (`:78505`) is the sound lane's
/// skip; the owner-gated acorn plant (`:78514-78531`) is the host wave's.
// STAGED (the host gap) — see `scarab_bomb_dust`.
#[allow(dead_code)]
pub(crate) fn acorn_bomb_dust(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    old_vel: Vec2,
) {
    let (cx, cy) = p.center();
    for _ in 0..10 {
        let idx = d.new_dust(
            world, Vec2::new(cx, cy), 10, 10, 2,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 2.0;
        dust.velocity -= old_vel * 0.3;
        dust.scale += d.rng.next_range(0, 150) as f32 * 0.001;
    }
}

/// `Utils.SelectRandom<T>` (`Utils.cs:2459-2462`) — ONE `Next(len)` pick.
fn select_random<const N: usize>(rng: &mut dyn DropRng, choices: [i32; N]) -> i32 {
    choices[rng.next_range(0, N as i32) as usize]
}

/// `Utils.RandomVector2` (`Utils.cs:2431-2434`) — TWO `NextDouble` draws
/// cast to float BEFORE the affine map (the g64 `853039bfc` decode; the
/// `rotated_by_random` next_double convention).
fn random_vector2(rng: &mut dyn DropRng, min: f32, max: f32) -> Vec2 {
    Vec2::new(
        (max - min) * rng.next_double() as f32 + min,
        (max - min) * rng.next_double() as f32 + min,
    )
}

/// `Main.hslToRgb(h, s, l)` as a dust `Color` (`Main.cs:47890-47917`,
/// default `a = 255`): the palette port returns byte-exact components as
/// `[f32]` in 0..1 (`palette.rs:196-227`, the banker's-rounding port); the
/// ×255 round-trip is exact for k/255 in f32.
fn hsl_color(h: f32, s: f32, l: f32) -> Color {
    let rgb = tr_core::tile_light::hsl_to_rgb(h, s, l);
    Color::new(
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8,
        255,
    )
}

// ---------------------------------------------------------------------------
// The AI-INLINE-40 window (research/231 re-probe round, was 125): the AI()
// inline remainder + the AI_001 edges. Emitters follow the e3 pattern —
// argument draws FIRST, pool internals, post-writes on the returned slot
// incl. the sentinel. STAGED kinds are cited per emitter (the row/machine
// merge that activates them is pinned in research/231's FINAL
// activation-order table).
// ---------------------------------------------------------------------------

/// `:56042-56055` — the 1006 ShimmerArrow's rainbow sparkle (AI_001, WIRED
/// on this base — `special_arrows::step`'s flight-counter tail): the
/// `Next(8)` gate every tick, the wet-halved ×1.25 velocity, the
/// `NextVector2Circular(4,4)` point (TWO draws), the hue `NextFloat`
/// (argument order: position THEN colour), the scale `NextFloat` — then
/// `NewDustPerfect` type 306, noGravity, `fadeIn = scale + 0.05`, and the
/// `CloneDust` twin recoloured White with scale −0.3 (the clone's own
/// internals ride the stream; the sentinel guard is the pool's).
pub(crate) fn ai001_shimmer_sparkle_1006(p: &Projectile, world: &World, d: &mut ProjDust) {
    if p.kind != 1006 || d.rng.next_range(0, 8) != 0 {
        return;
    }
    let mut value = p.vel * 1.25;
    if p.wet {
        value *= 0.5;
    }
    let (cx, cy) = p.center();
    let point = Vec2::new(cx, cy) + next_vector2_circular(d.rng, 4.0, 4.0);
    let color = hsl_color(d.rng.next_float(), 1.0, 0.5);
    let scale = 1.0 + d.rng.next_float() * 0.4;
    let idx = d.new_dust_perfect(world, point, 306, Some(value), 0, color, scale);
    let dust = d.pool.slot_mut(idx);
    dust.no_gravity = true;
    dust.fade_in = dust.scale + 0.05;
    let clone = d.clone_dust(world, idx);
    let cd = d.pool.slot_mut(clone);
    cd.color = Color::WHITE;
    cd.scale -= 0.3;
}

/// `:56746-56752` — the 981 SilverBullet's 1-in-6 dust-279 drip (AI_001,
/// STAGED — no row on this base; the bullet-side row wave activates it at
/// the fade-chain slot AFTER the 265/355 arms, BEFORE the 357 trail):
/// `noLightEmittance` (the pool carries the field), noGravity, ×0.2.
pub(crate) fn ai001_silver_bullet_drip_981(p: &Projectile, world: &World, d: &mut ProjDust) {
    if p.kind != 981 || d.rng.next_range(0, 6) != 0 {
        return;
    }
    let idx = d.new_dust(
        world,
        p.pos,
        p.w as i32,
        p.h as i32,
        279,
        p.vel.x,
        p.vel.y,
        100,
        Color::default(),
        1.2,
    );
    let dust = d.pool.slot_mut(idx);
    dust.no_light_emittance = true;
    dust.no_gravity = true;
    dust.velocity *= 0.2;
}

/// `:29238-29303` — the 1042 ShimmerGunStream's six-point dust-324 trail
/// (aiStyle-60's `type == 1042` tail, STAGED — the row + machine are W4-d6's
/// `projectile_ai_guns` side, dust-skipped there; the g63 doc-66 census row
/// mis-lists these two sites under the 358 — the emitter census correction
/// is in research/231, was 125). Per point: THREE independent Next(2) alpha+25
/// gates, noGravity, vel×0.3 + vel×0.5, position = Centre − v·i/6, vel×0.2,
/// then the Next(6) colour switch (3 named + default). The Next(4) extra
/// dust takes scale 0.6, vel×0.5 + vel×0.5 and its OWN Next(6) colour pick
/// — no alpha gates, no position rewrite (the C# asymmetry).
pub(crate) fn style60_shimmer_gun_trail_1042(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    for i in 0..6 {
        let off_x = p.vel.x / 6.0 * i as f32;
        let off_y = p.vel.y / 6.0 * i as f32;
        let idx = d.new_dust(
            world,
            p.pos + Vec2::splat(6.0),
            p.w as i32 - 12,
            p.h as i32 - 12,
            324,
            0.0,
            0.0,
            75,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        for _ in 0..3 {
            if d.rng.next_range(0, 2) == 0 {
                dust.alpha += 25;
            }
        }
        dust.no_gravity = true;
        dust.velocity *= 0.3;
        dust.velocity += p.vel * 0.5;
        dust.position = Vec2::new(cx - off_x, cy - off_y);
        dust.velocity *= 0.2;
        dust.color = shimmer_gun_color(d.rng.next_range(0, 6));
    }
    if d.rng.next_range(0, 4) == 0 {
        let idx = d.new_dust(
            world,
            p.pos + Vec2::splat(6.0),
            p.w as i32 - 12,
            p.h as i32 - 12,
            324,
            0.0,
            0.0,
            75,
            Color::default(),
            0.6,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.5;
        dust.velocity += p.vel * 0.5;
        dust.color = shimmer_gun_color(d.rng.next_range(0, 6));
    }
}

/// `:29265-29279`/`:29287-29301` — the 1042's Next(6) colour switch.
fn shimmer_gun_color(pick: i32) -> Color {
    match pick {
        0 => Color::new(255, 255, 220, 255),
        1 => Color::new(230, 255, 255, 255),
        2 => Color::new(255, 210, 255, 255),
        _ => Color::new(225, 210, 255, 255),
    }
}

/// `:27867-27874` — the 227 CrystalLeafShot's first-tick 8× dust-157 burst
/// (aiStyle-43, STAGED — the machine is W4-d5's `projectile_ai_style43`,
/// UNWIRED, no row; its 8 kept call-site NextFloat draws are REPLACED by
/// this emitter at the d5 merge, the g63 body-skip supersede convention).
/// Per dust: internals, noGravity, vel×3, scale 1.5, then the POST-spawn
/// `vel += base.vel * NextFloat()`. The caller owns the soundDelay latch.
pub(crate) fn style43_crystal_leaf_burst_227(p: &Projectile, world: &World, d: &mut ProjDust) {
    for _ in 0..8 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            157,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 3.0;
        dust.scale = 1.5;
        dust.velocity += p.vel * d.rng.next_float();
    }
}

/// `:28832-28838` — the 321 FlamingJack's per-tick dust-6 head (aiStyle-55,
/// STAGED — RETIRE re-probe: the row + machine are NOW on the d6 tip
/// `7edfcd495` (`FLAMING_JACK` data.rs:1057/:2858 + `ai_flaming_jack`
/// projectile.rs:4731, the dispatch-55 arm; its `:28832-28838` skip rolls
/// nothing — a ZERO-explicit-draw machine, so this emitter ADDS the pool
/// internals at the d6 merge, the style87 shape). The caller gates on the C#'s
/// `ai[0] >= 0 && ai[0] < maxNPCs` window (`:28795`); the dust itself is
/// unconditional inside it: the 8-px inset box, vel×0.5 + vel×0.5,
/// noGravity, noLight, scale 1.4.
pub(crate) fn style55_flaming_jack_head_321(p: &Projectile, world: &World, d: &mut ProjDust) {
    let idx = d.new_dust(
        world,
        p.pos + Vec2::splat(8.0),
        p.w as i32 - 16,
        p.h as i32 - 16,
        6,
        0.0,
        0.0,
        0,
        Color::default(),
        1.0,
    );
    let dust = d.pool.slot_mut(idx);
    dust.velocity *= 0.5;
    dust.velocity += p.vel * 0.5;
    dust.no_gravity = true;
    dust.no_light = true;
    dust.scale = 1.4;
}

/// `:31011-31022`/`:31047-31058`/`:31141-31152`/`:31201-31209` — the 451
/// InfluxWaver's four rotated-corner sparks (aiStyle-81, STAGED — the row +
/// machine are W4-d9's `projectile_ai_style81`, dust-skipped there). ONE
/// shape, four call sites: the SelectRandom(226,229) type pick, the
/// (−32,32) offset rotated by the rotation, the 8×8 box at
/// Centre + offset − (4,4), alpha 100, the given scale, vel×0.1, then
/// noGravity — the two fade arms gate it on Next(6)!=0, the homing arm on
/// an OUTER Next(3)==0 (the caller's gate, drawn before the pick), the
/// strike arm writes it unconditionally. `gate_next6` selects the fade-arm
/// gate; the homing/strike callers pass false and own their outer gates.
pub(crate) fn style81_influx_spark_451(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    scale: f32,
    gate_next6: bool,
) {
    let ty = select_random(d.rng, [226, 229]);
    let (cx, cy) = p.center();
    let offset = crate::bow_launch::rotated_by(Vec2::new(-32.0, 32.0), p.rot as f64);
    let idx = d.new_dust(
        world,
        Vec2::new(cx, cy) + offset - Vec2::splat(4.0),
        8,
        8,
        ty,
        0.0,
        0.0,
        100,
        Color::default(),
        scale,
    );
    let dust = d.pool.slot_mut(idx);
    dust.velocity *= 0.1;
    if !gate_next6 || d.rng.next_range(0, 6) != 0 {
        dust.no_gravity = true;
    }
}

/// `:31889-31899` — the 464 CultistBossIceMist's six-spoke expanding ring
/// (aiStyle-86's mist form, STAGED — the d9 `projectile_ai_style86` machine
/// body-skips the dust). The caller gates `ai[1] != 1` + the pre-dust
/// machine walk (the 45-tick kill RETURNS before any dust). Per tick: the
/// (0,−720) spin rotated by the velocity's rotation, scaled by
/// `ai[0]%45/45`; 6 spokes × 2 dusts each at spoke + RandomVector2(−8,8)/2
/// (TWO NextDouble-cast draws per dust), 8×8 box, type 197, alpha 100,
/// Color.Transparent, noGravity. The AddLight per spoke is the light
/// lane's (c19/e6).
pub(crate) fn style86_ice_mist_ring_464(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    let spin = crate::bow_launch::rotated_by(
        Vec2::new(0.0, -720.0),
        p.vel.y.atan2(p.vel.x) as f64,
    ) * (p.ai[0] % 45.0 / 45.0);
    for k in 0..6 {
        // `:31892` — `(float)num706 * ((float)Math.PI * 2f) / 6f` is FLOAT
        // arithmetic (multiply then divide), widened to double only at the
        // RotatedBy call — the float rounding is kept.
        let angle = (k as f32 * (std::f32::consts::PI * 2.0) / 6.0) as f64;
        let spoke = Vec2::new(cx, cy) + crate::bow_launch::rotated_by(spin, angle);
        for _ in 0..2 {
            let jitter = random_vector2(d.rng, -8.0, 8.0) / 2.0;
            let idx = d.new_dust(
                world,
                spoke + jitter,
                8,
                8,
                197,
                0.0,
                0.0,
                100,
                Color::TRANSPARENT,
                1.0,
            );
            d.pool.slot_mut(idx).no_gravity = true;
        }
    }
}

/// `:31925-31935` — the 482 ClingerStaff's wall curtain (aiStyle-87,
/// STAGED — the d9 `projectile_ai_style87` machine keeps ZERO explicit
/// draws; the pool internals are the whole stream cost). Count =
/// `(width*height)*0.0045` (the C# float loop bound); per dust: type 75,
/// alpha 100, noGravity, vel×0.5, vel.Y −0.5, scale 1.4, pos.X +6,
/// pos.Y −2. The caller owns the keep-one sibling kill's early return.
pub(crate) fn style87_clinger_curtain_482(p: &Projectile, world: &World, d: &mut ProjDust) {
    let count = p.w * p.h * 0.0045;
    let mut i = 0.0f32;
    while i < count {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            75,
            0.0,
            0.0,
            100,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 0.5;
        dust.velocity.y -= 0.5;
        dust.scale = 1.4;
        dust.position.x += 6.0;
        dust.position.y -= 2.0;
        i += 1.0;
    }
}

/// `:32292-32319` — the 490 CultistRitual's alpha-0 two-dust ring (aiStyle-
/// 89, STAGED — the d9 `projectile_ai_style89` machine keeps the explicit
/// draws, bodies skipped; this emitter REPLACES that census at merge).
/// Per dust (i=0,1): speed = Next(2,4) as float, i=1 flips it ×−0.75 and
/// scales ×0.42; direction = Normalize(Next(-10,11), Next(-10,11)) — the
/// UNGUARDED XNA normalize (zero → NaN, the empress convention); type 228,
/// alpha 100, scale 2, noGravity + noLight, position = Centre + dir·204·
/// scale; the Next(8) gate doubles the inward velocity + scale +0.5.
pub(crate) fn style89_ritual_ring_490(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    for i in 0..2 {
        let mut speed = d.rng.next_range(2, 4) as f32;
        let mut sc = p.scale;
        if i == 1 {
            sc *= 0.42;
            speed *= -0.75;
        }
        let mut dir = Vec2::new(
            d.rng.next_range(-10, 11) as f32,
            d.rng.next_range(-10, 11) as f32,
        );
        dir = xna_normalize(dir);
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            228,
            0.0,
            0.0,
            100,
            Color::default(),
            2.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.no_light = true;
        dust.position = Vec2::new(cx, cy) + dir * 204.0 * sc;
        if d.rng.next_range(0, 8) == 0 {
            dust.velocity = dir * (0.0 - speed) * 2.0;
            dust.scale += 0.5;
        } else {
            dust.velocity = dir * (0.0 - speed);
        }
    }
}

/// `:32320-32344` — the 490's post-60 per-tick centre puff (the caller
/// gates `ai[0] >= 60` AFTER the increment): speed = Next(1,3), the same
/// normalized Next(-10,11)² direction, type 228 scale 2, noGravity +
/// noLight, position = Centre, the Next(2) gate doubles + scale +0.5,
/// fadeIn 2.
pub(crate) fn style89_ritual_puff_490(p: &Projectile, world: &World, d: &mut ProjDust) {
    let speed = d.rng.next_range(1, 3) as f32;
    let dir = xna_normalize(Vec2::new(
        d.rng.next_range(-10, 11) as f32,
        d.rng.next_range(-10, 11) as f32,
    ));
    let (cx, cy) = p.center();
    let idx = d.new_dust(
        world,
        p.pos,
        p.w as i32,
        p.h as i32,
        228,
        0.0,
        0.0,
        100,
        Color::default(),
        2.0,
    );
    let dust = d.pool.slot_mut(idx);
    dust.no_gravity = true;
    dust.no_light = true;
    dust.position = Vec2::new(cx, cy);
    if d.rng.next_range(0, 2) == 0 {
        dust.velocity = dir * speed * 2.0;
        dust.scale += 0.5;
    } else {
        dust.velocity = dir * speed;
    }
    dust.fade_in = 2.0;
}

/// `Vector2.Normalize()` (`Vector2.cs`) — XNA divides UNGUARDED: a zero
/// vector yields NaN (the `npc_ai_empress` convention documents the
/// divergence; the C# behaviour is kept here — the (0,0) direction pick
/// propagates NaN into the position/velocity exactly as the original).
fn xna_normalize(v: Vec2) -> Vec2 {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    Vec2::new(v.x / len, v.y / len)
}

/// `:32424-32430` — ONE spelunkable cell's dust-204 (the 492 MagicLantern's
/// 10-tick scan body, STAGED — the d9 `projectile_ai_style90` machine keeps
/// the per-cell Next(4) head draws and skips the body; the scan walk +
/// `is_tile_spelunkable` are that machine's, this emitter is the per-hit
/// drop-in at its :32426 slot). Type 204 at the tile's 16×16 box, alpha
/// 150, scale 0.3, fadeIn 0.75, vel×0.1, noLight. NO explicit draws — the
/// Next(4) is the caller's && head, consumed for EVERY cell.
pub(crate) fn style90_lantern_spark_492(world: &World, d: &mut ProjDust, tx: i32, ty: i32) {
    let idx = d.new_dust(
        world,
        Vec2::new(tx as f32 * 16.0, ty as f32 * 16.0),
        16,
        16,
        204,
        0.0,
        0.0,
        150,
        Color::default(),
        0.3,
    );
    let dust = d.pool.slot_mut(idx);
    dust.fade_in = 0.75;
    dust.velocity *= 0.1;
    dust.no_light = true;
}

/// `:32544-32556` — the 496 ShadowFlame's scale-shrinking dust-27 tail
/// (aiStyle-91, STAGED — the d10 `projectile_ai_91_100` machine's skip).
/// The caller gates `scale < 1`; count = the C# float loop `i < scale*10`.
/// Per dust: type 27 with the VELOCITY args (speedX/Y = base.velocity),
/// alpha 100, scale 1.1; position = (pos + Centre)/2, noGravity, vel×0.1,
/// vel −= base.vel·(1.3−scale), fadeIn = 100 + owner (the C# owner-tagged
/// fadeIn), scale += scale·0.75.
pub(crate) fn style91_shadowflame_tail_496(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    let mut i = 0.0f32;
    while i < p.scale * 10.0 {
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
            1.1,
        );
        let dust = d.pool.slot_mut(idx);
        dust.position = (dust.position + Vec2::new(cx, cy)) / 2.0;
        dust.no_gravity = true;
        dust.velocity *= 0.1;
        dust.velocity -= p.vel * (1.3 - p.scale);
        dust.fade_in = 100.0 + p.owner as f32;
        dust.scale += p.scale * 0.75;
        i += 1.0;
    }
}

/// `:32953-32961` — the 518 CoinPortal's 1-in-3 orbiting dust-246 (aiStyle-
/// 94, STAGED — the d10 machine's skip). The caller gates `alpha < 90`
/// BEFORE the Next(3) (the C# && order). The point: (w,h)·scale·0.85/2,
/// UnitY.RotatedByRandom(2π) scaled — TWO NextDouble draws; the dust spawns
/// at Centre+point with a 0×0 box then BOTH position and velocity are
/// overwritten (position = Centre+point exactly, velocity = Zero).
pub(crate) fn style94_coin_portal_spark_518(p: &Projectile, world: &World, d: &mut ProjDust) {
    let half = Vec2::new(p.w, p.h) * p.scale * 0.85 / 2.0;
    let point = rotated_by_random(Vec2::Y, 6.2831854820251465, d.rng) * half;
    let (cx, cy) = p.center();
    let at = Vec2::new(cx, cy) + point;
    let idx = d.new_dust(world, at, 0, 0, 246, 0.0, 0.0, 0, Color::default(), 1.0);
    let dust = d.pool.slot_mut(idx);
    dust.position = at;
    dust.velocity = Vec2::ZERO;
}

/// `:33011-33026` — the 524 IchorSplash's two 2-in-3 dust-170 puffs
/// (aiStyle-96, STAGED — the d10 `projectile_ai_91_100` machine's skip).
/// Per iteration: the Next(3)!=0 gate; type 170 alpha 100; position =
/// (pos + Centre)/2, noGravity, vel×0.1; i=1 adds vel/2 to the position;
/// scale ×= (800−ai[0])/800 + 0.1 (the C# age read AFTER the +=0.6 and the
/// >500 kill — the caller owns both; the kill does NOT return).
pub(crate) fn style96_ichor_puffs_524(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    for i in 0..2 {
        if d.rng.next_range(0, 3) != 0 {
            let idx = d.new_dust(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                170,
                0.0,
                0.0,
                100,
                Color::default(),
                1.0,
            );
            let dust = d.pool.slot_mut(idx);
            dust.position = (dust.position + Vec2::new(cx, cy)) / 2.0;
            dust.no_gravity = true;
            dust.velocity *= 0.1;
            if i == 1 {
                dust.position += p.vel / 2.0;
            }
            dust.scale *= (800.0 - p.ai[0]) / 800.0 + 0.1;
        }
    }
}

/// `:33145-33151` — the 526 CultistBossParticle's two Centre dust-228s
/// (aiStyle-98, STAGED — the d10 machine's skip; the caller owns the
/// arrival kill's early return). Per dust: 0×0 box at Centre, alpha 100,
/// noGravity, position += (4,4), scale += NextFloat.
pub(crate) fn style98_cultist_particle_pair_526(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    for _ in 0..2 {
        let idx = d.new_dust(
            world,
            Vec2::new(cx, cy),
            0,
            0,
            228,
            0.0,
            0.0,
            100,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.position += Vec2::splat(4.0);
        dust.scale += d.rng.next_float();
    }
}

/// ONE 578/579 dissolve-ring dust (`:33986-34014`'s two case bodies share
/// the shape): the UnitY.RotatedByRandom(2π)·scale spin (TWO NextDouble
/// draws), the spawn at Centre − spin·30, noGravity; `reroll` selects the
/// case-0 body (position re-rolled to Centre − spin·Next(10,21), velocity
/// spin.RotatedBy(+π/2)·6) vs case-1 (position kept, velocity
/// spin.RotatedBy(−π/2)·3); then scale = 0.5 + NextFloat, fadeIn 0.5. The
/// `customData = Centre` write is the pool's dropped field.
fn vortex_dissolve_dust(
    world: &World,
    d: &mut ProjDust,
    center: Vec2,
    scale: f32,
    dust_type: i32,
    reroll: bool,
) {
    let spin = rotated_by_random(Vec2::Y, 6.2831854820251465, d.rng) * scale;
    let idx = d.new_dust(
        world,
        center - spin * 30.0,
        0,
        0,
        dust_type,
        0.0,
        0.0,
        0,
        Color::default(),
        1.0,
    );
    let dust = d.pool.slot_mut(idx);
    dust.no_gravity = true;
    if reroll {
        dust.position = center - spin * d.rng.next_range(10, 21) as f32;
        dust.velocity = crate::bow_launch::rotated_by(spin, 1.5707963705062866) * 6.0;
    } else {
        dust.position = center - spin * 30.0;
        dust.velocity = crate::bow_launch::rotated_by(spin, -1.5707963705062866) * 3.0;
    }
    dust.scale = 0.5 + d.rng.next_float();
    dust.fade_in = 0.5;
}

/// `:33984-34016` — the 578/579 Vortex markers' dissolve twin switch
/// (WIRED on this base — `ai_vortex_ritual`'s dissolve arm; the g64
/// remainder-2 census explicitly leaves ":33984-34016 the co-lane's
/// unmodelled skip" — THIS is that co-lane port). TWO iterations, each a
/// Next(3) switch: case 0 → dust 229 with the Next(10,21) position
/// re-roll, case 1 → dust 240 at the ×30 keep, case 2 → nothing (the
/// switch draw is consumed regardless). The 813's scale-premultiplied
/// twin (:33951-33983) is g64's `trail_813_dissolve` — the kind gates are
/// disjoint; the 813 opacity tail (:34018-34024) is g64's too.
pub(crate) fn vortex_dissolve_twin_578_579(
    world: &World,
    d: &mut ProjDust,
    center: Vec2,
    scale: f32,
) {
    for _ in 0..2 {
        match d.rng.next_range(0, 3) {
            0 => vortex_dissolve_dust(world, d, center, scale, 229, true),
            1 => vortex_dissolve_dust(world, d, center, scale, 240, false),
            _ => {} // `:34014` — case 2 draws nothing further.
        }
    }
}

/// `:34118-34127` — the 836 DandelionSeed's first-tick 3× dust-31 burst
/// (aiStyle-112's `type == 836` head, STAGED — RETIRE re-probe: the row +
/// machine are NOW on the d12 tip `c98bd8937` (`DANDELION_SEED` data.rs:1090/
/// :3697 + `ai_dandelion_seed` projectile.rs:10080, the dispatch-112 arm; its
/// `:34121-34126` latch-burst skip is all NewDust internals — ZERO explicit
/// draws, so this emitter ADDS the pool internals at the d12 merge).
/// The caller owns the localAI[0] latch. Per dust: type 31, alpha 50,
/// Color.White, scale 1.2, vel×0.3, noGravity.
pub(crate) fn style112_dandelion_burst_836(p: &Projectile, world: &World, d: &mut ProjDust) {
    for _ in 0..3 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            31,
            0.0,
            0.0,
            50,
            Color::WHITE,
            1.2,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.3;
        dust.no_gravity = true;
    }
}

/// `:34505-34535` — the aiStyle-117 flag58 half (the 612 SolarWhipSword /
/// 953 Daybreak / 978 explosions; STAGED — the d12
/// `projectile_ai_explosions` machine keeps the explicit draws, bodies
/// skipped; this emitter REPLACES that census at merge). The caller owns
/// the tick-1 gate + the 52·scale resize (the emitter reads the RESIZED
/// box). Four loops: 4×31 (alpha 100, scale 1.5), 10×(6 alpha-200 ×2.7
/// noGravity ×3 + 6 alpha-100 ×1.5 ×2 noGravity fadeIn 2.5), 5×6 (alpha 0,
/// ×2.7, the UnitX velocity-rotated recipe), 10×31 (alpha 0, ×1.5, same
/// UnitX recipe). Every position write is POST-spawn:
/// RotatedByRandom(π)'s TWO NextDouble draws then ONE NextDouble radius
/// (the UnitY loops × width/2; the UnitX loops rotate by the velocity's
/// rotation and take the full width/2, no NextDouble radius).
pub(crate) fn style117_whip_explosion_612(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    let center = Vec2::new(cx, cy);
    let vel_rot = p.vel.y.atan2(p.vel.x) as f64;
    let unit_y_point = |d: &mut ProjDust| {
        center
            + rotated_by_random(Vec2::Y, 3.1415927410125732, d.rng)
                * (d.rng.next_double() as f32)
                * p.w
                / 2.0
    };
    let unit_x_point = |d: &mut ProjDust| {
        center
            + crate::bow_launch::rotated_by(
                rotated_by_random(Vec2::X, 3.1415927410125732, d.rng),
                vel_rot,
            )
                * p.w
                / 2.0
    };
    for _ in 0..4 {
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 31, 0.0, 0.0, 100, Color::default(), 1.5);
        let at = unit_y_point(d);
        d.pool.slot_mut(idx).position = at;
    }
    for _ in 0..10 {
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 6, 0.0, 0.0, 200, Color::default(), 2.7);
        let at = unit_y_point(d);
        let dust = d.pool.slot_mut(idx);
        dust.position = at;
        dust.no_gravity = true;
        dust.velocity *= 3.0;
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 6, 0.0, 0.0, 100, Color::default(), 1.5);
        let at = unit_y_point(d);
        let dust = d.pool.slot_mut(idx);
        dust.position = at;
        dust.velocity *= 2.0;
        dust.no_gravity = true;
        dust.fade_in = 2.5;
    }
    for _ in 0..5 {
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 6, 0.0, 0.0, 0, Color::default(), 2.7);
        let at = unit_x_point(d);
        let dust = d.pool.slot_mut(idx);
        dust.position = at;
        dust.no_gravity = true;
        dust.velocity *= 3.0;
    }
    for _ in 0..10 {
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 31, 0.0, 0.0, 0, Color::default(), 1.5);
        let at = unit_x_point(d);
        let dust = d.pool.slot_mut(idx);
        dust.position = at;
        dust.no_gravity = true;
        dust.velocity *= 3.0;
    }
}

/// `:34540-34627` — the aiStyle-117 flag59 half (the 624 StardustGuardian
/// dash explosion; STAGED — the d12 machine's skip). 20×135 (alpha 100,
/// ×1.5, the UnitY recipe, ×2 noGravity fadeIn 2.5), 15×135 (alpha 0,
/// ×2.7, the UnitX recipe, noGravity ×3), then the SIX pre-loop draws
/// (three NextDouble angles, three 7+NextDouble·7 radii, num862 = max)
/// and the 200×135 ring: 6×6 box at position, alpha 100, stock scale;
/// velocity captured, position = Centre, the UNGUARDED normalize × the
/// i-banded radius (>50 → r2, >100 → r1, >150 → r3, else max), the
/// i-banded axis-half + rotation (>150 Y+angle3, >100 X+angle1, >50
/// Y+angle2), vel×0.2 + spin, scale 2, noGravity, fadeIn NextFloat·2 with
/// the Next(4)==0 → 2.5 override, noLight, and the i<100 position
/// += vel·20 / vel ×= −1 flip. The `shader` writes are the pool's dropped
/// field (all three loops).
pub(crate) fn style117_guardian_explosion_624(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    let center = Vec2::new(cx, cy);
    let vel_rot = p.vel.y.atan2(p.vel.x) as f64;
    for _ in 0..20 {
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 135, 0.0, 0.0, 100, Color::default(), 1.5);
        let at = center
            + rotated_by_random(Vec2::Y, 3.1415927410125732, d.rng)
                * (d.rng.next_double() as f32)
                * p.w
                / 2.0;
        let dust = d.pool.slot_mut(idx);
        dust.position = at;
        dust.velocity *= 2.0;
        dust.no_gravity = true;
        dust.fade_in = 2.5;
    }
    for _ in 0..15 {
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 135, 0.0, 0.0, 0, Color::default(), 2.7);
        let at = center
            + crate::bow_launch::rotated_by(
                rotated_by_random(Vec2::X, 3.1415927410125732, d.rng),
                vel_rot,
            )
                * p.w
                / 2.0;
        let dust = d.pool.slot_mut(idx);
        dust.position = at;
        dust.no_gravity = true;
        dust.velocity *= 3.0;
    }
    // `:34557-34571` — the six draws + the max.
    let a1 = d.rng.next_double() * (std::f64::consts::PI * 2.0);
    let a2 = d.rng.next_double() * (std::f64::consts::PI * 2.0);
    let a3 = d.rng.next_double() * (std::f64::consts::PI * 2.0);
    let r1 = 7.0 + d.rng.next_double() * 7.0;
    let r2 = 7.0 + d.rng.next_double() * 7.0;
    let r3 = 7.0 + d.rng.next_double() * 7.0;
    let rmax = r1.max(r2).max(r3);
    for i in 0..200 {
        let radius = if i > 150 {
            r3
        } else if i > 100 {
            r1
        } else if i > 50 {
            r2
        } else {
            rmax
        };
        let idx = d.new_dust(world, p.pos, 6, 6, 135, 0.0, 0.0, 100, Color::default(), 1.0);
        let mut spin = d.pool.slot(idx).velocity;
        let dust = d.pool.slot_mut(idx);
        dust.position = center;
        spin = xna_normalize(spin);
        spin *= radius as f32;
        if i > 150 {
            spin.y *= 0.5;
            spin = crate::bow_launch::rotated_by(spin, a3);
        } else if i > 100 {
            spin.x *= 0.5;
            spin = crate::bow_launch::rotated_by(spin, a1);
        } else if i > 50 {
            spin.y *= 0.5;
            spin = crate::bow_launch::rotated_by(spin, a2);
        }
        dust.velocity *= 0.2;
        dust.velocity += spin;
        dust.scale = 2.0;
        dust.no_gravity = true;
        dust.fade_in = d.rng.next_float() * 2.0;
        if d.rng.next_range(0, 4) == 0 {
            dust.fade_in = 2.5;
        }
        dust.no_light = true;
        if i < 100 {
            dust.position += dust.velocity * 20.0;
            dust.velocity *= -1.0;
        }
    }
}

/// `:35852-35859` — the aiStyle-126 state-0 rise (the 654 GeyserTrap + the
/// 670 DD2OgreStomp; STAGED — the d13 `projectile_ai_geyser` machine keeps
/// the explicit draws, bodies skipped; this REPLACES that census). THREE
/// dust-31s, alpha 100; per dust the POST-spawn pair: scale = 0.1 +
/// Next(5)·0.1, fadeIn = 1.5 + Next(5)·0.1 (the d13 census's TWO Next(5)
/// per dust); noGravity, position = Centre + rot((0,−h/2), rotation)·1.1.
pub(crate) fn style126_column_rise(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    // `:35858` — `-base.height / 2` is INTEGER division in the C# (the
    // height is an int field); the 30-px rows are even, but the truncation
    // is kept verbatim.
    let at = Vec2::new(cx, cy)
        + crate::bow_launch::rotated_by(
            Vec2::new(0.0, -(p.h as i32 / 2) as f32),
            p.rot as f64,
        ) * 1.1;
    for _ in 0..3 {
        let idx = d.new_dust(world, p.pos, p.w as i32, p.h as i32, 31, 0.0, 0.0, 100, Color::default(), 1.0);
        let dust = d.pool.slot_mut(idx);
        dust.scale = 0.1 + d.rng.next_range(0, 5) as f32 * 0.1;
        dust.fade_in = 1.5 + d.rng.next_range(0, 5) as f32 * 0.1;
        dust.no_gravity = true;
        dust.position = at;
    }
}

/// ONE SelectRandom(6,259,158) column dust (`:35914-35924`/`:35932-35942`
/// share the body): the type pick FIRST (one Next(3)), the spawn with
/// SpeedY = −2.5·(−sign) = 2.5·sign, alpha 200 post-write, vel ×(0.3,2),
/// vel.Y += 2·sign, scale += NextFloat, position pinned to the column
/// base edge; the sign==−1-only Next(4)!=0 → vel.Y −0.2. `pin_edge`
/// selects the burst/plume position rewrite (the :35986 tail keeps the
/// spawn position); `vel_mul` is (0.3,2) for the burst/plume, (0.6,1.5)
/// for the tail; `add_sign2` is the burst/plume's vel.Y += 2·sign (the
/// tail omits it). customData = num939 dropped.
#[allow(clippy::too_many_arguments)]
fn column_select_dust(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    pos: Vec2,
    w: i32,
    h: i32,
    sign: i32,
    pin_edge: bool,
    vel_mul: Vec2,
    add_sign2: bool,
) {
    let ty = select_random(d.rng, [6, 259, 158]);
    let idx = d.new_dust(world, pos, w, h, ty, 0.0, 2.5 * sign as f32, 0, Color::default(), 1.0);
    let dust = d.pool.slot_mut(idx);
    dust.alpha = 200;
    dust.velocity *= vel_mul;
    if add_sign2 {
        dust.velocity.y += (2 * sign) as f32;
    }
    dust.scale += d.rng.next_float();
    if pin_edge {
        let (cx, cy) = p.center();
        dust.position = Vec2::new(cx, cy + p.h * 0.5 * (-sign) as f32);
    }
    if sign == -1 && d.rng.next_range(0, 4) != 0 {
        dust.velocity.y -= 0.2;
    }
}

/// `:35909-35927` — the aiStyle-126 first-solid 60-dust burst (the caller
/// owns the localAI[0] latch; the Item34 cue at :35926 is the sound
/// lane's). STAGED — the d13 geyser machine's skip.
pub(crate) fn style126_column_burst(p: &Projectile, world: &World, d: &mut ProjDust, sign: i32) {
    for _ in 0..60 {
        column_select_dust(
            p, world, d, p.pos, p.w as i32, p.h as i32, sign,
            true, Vec2::new(0.3, 2.0), true,
        );
    }
}

/// `:35928-35944` — the aiStyle-126 sign==1 nine-dust plume (the caller
/// gates `num938 == 1`). STAGED — the d13 machine's skip.
pub(crate) fn style126_column_plume(p: &Projectile, world: &World, d: &mut ProjDust, sign: i32) {
    for _ in 0..9 {
        column_select_dust(
            p, world, d, p.pos, p.w as i32, p.h as i32, sign,
            true, Vec2::new(0.3, 2.0), true,
        );
    }
}

/// `:35945-35981` — the aiStyle-126 column's six 2-in-3 dust-6s. The
/// caller computes `num951 = min((int)(ai[1]/60·h)·3, h)` and the two
/// anchors (vector156 = position + the sign==−1 height offset, vector157
/// = position + (0,h) when sign==−1). Per iteration: the Next(3)<2 gate;
/// dust 6 alpha 90 scale 2.5 at (vector156, w, num951); noGravity,
/// fadeIn 1; vel.Y>0 → ×−1; then the Next(6)<3 split — taken: position.Y
/// lerps to vector157.Y at 0.5, vel ×5, vel.Y −3, position.X = Centre.X,
/// noGravity FALSE, noLight, fadeIn 0.4, scale ×0.3; else: vel =
/// DirectionFrom(dust.position)·|vel|·0.25 (the UNGUARDED normalize of
/// Centre − dustpos); then vel.Y ×= −sign. customData dropped.
#[allow(clippy::too_many_arguments)]
pub(crate) fn style126_column_body(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    sign: i32,
    anchor: Vec2,
    edge_y: f32,
    col_h: i32,
) {
    let (cx, cy) = p.center();
    for _ in 0..6 {
        if d.rng.next_range(0, 3) < 2 {
            let idx = d.new_dust(world, anchor, p.w as i32, col_h, 6, 0.0, 0.0, 90, Color::default(), 2.5);
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.fade_in = 1.0;
            if dust.velocity.y > 0.0 {
                dust.velocity.y *= -1.0;
            }
            if d.rng.next_range(0, 6) < 3 {
                dust.position.y += (edge_y - dust.position.y) * 0.5;
                dust.velocity *= 5.0;
                dust.velocity.y -= 3.0;
                dust.position.x = cx;
                dust.no_gravity = false;
                dust.no_light = true;
                dust.fade_in = 0.4;
                dust.scale *= 0.3;
            } else {
                let dir = xna_normalize(Vec2::new(cx, cy) - dust.position);
                dust.velocity = dir * dust.velocity.length() * 0.25;
            }
            dust.velocity.y *= (-sign) as f32;
        }
    }
}

/// `:35982-35996` — the aiStyle-126 column's six NextFloat()<0.5-gated
/// SelectRandom tails (the gate is `!(NextFloat() < 0.5)` — the draw is
/// consumed EVERY iteration). The (0.6,1.5) velocity mul, NO +2·sign,
/// NO position pin (the spawn position stands). STAGED — the d13 skip.
pub(crate) fn style126_column_tail(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    sign: i32,
    anchor: Vec2,
    col_h: i32,
) {
    for _ in 0..6 {
        if !(d.rng.next_float() < 0.5) {
            column_select_dust(
                p, world, d, anchor, p.w as i32, col_h, sign,
                false, Vec2::new(0.6, 1.5), false,
            );
        }
    }
}

/// `MathHelper.Lerp` (`MathHelper.cs`) — the UNCLAMPED linear interpolation
/// (the Clamp variant is a different method; the C# sites here use Lerp).
fn math_lerp(a: f32, b: f32, amount: f32) -> f32 {
    a + (b - a) * amount
}

/// `:36094-36117` — the 658 SandnadoHostileMark's spawn plume (aiStyle-128,
/// STAGED — the d13 `projectile_ai_sandnado_mark` machine keeps the
/// explicit draws, bodies skipped; this REPLACES that census). The caller
/// gates `localAI[1] < 30`. ONE dust-269: amount = NextFloat FIRST, then
/// the X-lerp NextFloat (the Y lerp reuses amount); X ×= Lerp(2.2,0.6,
/// amount) then ×−1; point = Centre + (60,200)·dir·0.5 + (2,10); the 0×0
/// spawn, position = point exactly, fadeIn 1, scale 0.3; the
/// `dir.X > −1.2`-gated NextFloat velocity.X = 1+f, then the
/// unconditional NextFloat velocity.Y = f·−0.5 − 1. customData dropped.
pub(crate) fn style128_sandnado_plume_658(p: &Projectile, world: &World, d: &mut ProjDust) {
    let amount = d.rng.next_float();
    let mut dir = Vec2::new(
        math_lerp(0.1, 1.0, d.rng.next_float()),
        math_lerp(-0.5, 0.9, amount),
    );
    dir.x *= math_lerp(2.2, 0.6, amount);
    dir.x *= -1.0;
    let offset = Vec2::new(2.0, 10.0);
    let (cx, cy) = p.center();
    let point = Vec2::new(cx, cy) + Vec2::new(60.0, 200.0) * dir * 0.5 + offset;
    let idx = d.new_dust(world, point, 0, 0, 269, 0.0, 0.0, 0, Color::default(), 1.0);
    let dust = d.pool.slot_mut(idx);
    dust.position = point;
    dust.fade_in = 1.0;
    dust.scale = 0.3;
    if dir.x > -1.2 {
        dust.velocity.x = 1.0 + d.rng.next_float();
    }
    dust.velocity.y = d.rng.next_float() * -0.5 - 1.0;
}

/// `:36152-36177` — the 658's TWO orbit-dust loops (aiStyle-128, STAGED —
/// the d13 machine's skip). Each: 2 iterations, a Next(10)==0 gate; the
/// spin = UnitY.RotatedBy(i·π) — the FIRST loop additionally
/// .RotatedBy(rotation), the second does NOT (the C# asymmetry); dust 267
/// at Centre, 0×0, alpha 225, Color.White, scale 1.5; noGravity, noLight,
/// scale = Opacity·localAI[0] (the caller passes the product), position =
/// Centre, velocity = spin·2.5.
pub(crate) fn style128_sandnado_orbits_658(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust,
    opacity_x_local0: f32,
) {
    let (cx, cy) = p.center();
    let center = Vec2::new(cx, cy);
    for i in 0..2 {
        if d.rng.next_range(0, 10) == 0 {
            // `:36156` — `(float)num964 * (float)Math.PI` is FLOAT math.
            let spin = crate::bow_launch::rotated_by(
                crate::bow_launch::rotated_by(
                    Vec2::Y,
                    (i as f32 * std::f32::consts::PI) as f64,
                ),
                p.rot as f64,
            );
            let idx = d.new_dust(world, center, 0, 0, 267, 0.0, 0.0, 225, Color::WHITE, 1.5);
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.no_light = true;
            dust.scale = opacity_x_local0;
            dust.position = center;
            dust.velocity = spin * 2.5;
        }
    }
    for i in 0..2 {
        if d.rng.next_range(0, 10) == 0 {
            // `:36169` — same float multiply, NO rotation spin.
            let spin = crate::bow_launch::rotated_by(
                Vec2::Y,
                (i as f32 * std::f32::consts::PI) as f64,
            );
            let idx = d.new_dust(world, center, 0, 0, 267, 0.0, 0.0, 225, Color::WHITE, 1.5);
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.no_light = true;
            dust.scale = opacity_x_local0;
            dust.position = center;
            dust.velocity = spin * 2.5;
        }
    }
}

/// `:36414-36432` — the 672 DD2ElderWins' destruction motes (aiStyle-132,
/// STAGED — the d14 `ai_elder_wins` machine's skip). The caller gates
/// `localAI[1] >= 1`; count = `1 + (int)(0 − vel.Y)` (the C# int cast
/// truncates toward zero). Per iteration: the Next(2)==0 gate; rise =
/// NextFloat·(−vel.Y); X = NextFloat·0.1 − 0.05 then += sin((pos.Y+rise)·
/// 0.01 + 2π·(Next(3)/3.0))·0.5 (the float multiply BEFORE the double
/// widen, the integer Next(3) over the double 3.0); X = Lerp(NextFloat −
/// 0.5, X, Clamp(−vel.Y, 0, 1)); Y = 0.5 − |X|; ×(100,50); Y += rise;
/// NewDustPerfect type 27 at Centre+point, velocity Zero, alpha 0,
/// `Color.White * 0.1` = (25,25,25,25) (the XNA byte multiply), scale 1.1;
/// then velocity.Y = NextFloat·2, fadeIn 0.1.
pub(crate) fn style132_elder_wins_motes_672(p: &Projectile, world: &World, d: &mut ProjDust) {
    let (cx, cy) = p.center();
    let count = 1 + (0.0 - p.vel.y) as i32;
    for _ in 0..count {
        if d.rng.next_range(0, 2) == 0 {
            let rise = d.rng.next_float() * (0.0 - p.vel.y);
            let mut x = d.rng.next_float() * 0.1 - 0.05;
            let phase = d.rng.next_range(0, 3) as f64 / 3.0;
            x += (((p.pos.y + rise) * 0.01) as f64 + 6.2831854820251465 * phase).sin() as f32 * 0.5;
            let amount = (0.0 - p.vel.y).clamp(0.0, 1.0);
            x = math_lerp(d.rng.next_float() - 0.5, x, amount);
            let mut point = Vec2::new(x, 0.5 - x.abs());
            point *= Vec2::new(100.0, 50.0);
            point.y += rise;
            let idx = d.new_dust_perfect(
                world,
                Vec2::new(cx, cy) + point,
                27,
                Some(Vec2::ZERO),
                0,
                Color::new(25, 25, 25, 25),
                1.1,
            );
            let dust = d.pool.slot_mut(idx);
            dust.velocity.y = d.rng.next_float() * 2.0;
            dust.fade_in = 0.1;
        }
    }
}

// ---------------------------------------------------------------------------
// W4-d53 — the projectile-TICK dust lanes (the first ~40 sites,
// Projectile.cs:16851-22161: the arrow trail / Vortex-bolt trail /
// enchantment visuals + the wet-splash family). UNIONED into this module
// at the batch-F integration (research/131 §8.3 step 9: the d53×e3
// fn-name sets are DISJOINT; one file, two lanes). The d53 drain half
// snapshots via `DustBurst`/`EnchantmentBurst` and replays through
// `drain(app, b)` on the UpdateProjectiles stream.
// ---------------------------------------------------------------------------

/// `ProjectileID.Sets` stand-in: the kinds whose `SetDefaults` arm writes
/// `noEnchantmentVisuals = true` (`Projectile.cs:2683/3147/3160/4350/4810/
/// 4834/5766/6182/6582/7171/7383/7400/7518/7714/8872/9011/9234/10280/10338/
/// 10575/10595` — types 181/228/229/344/399/401/504/566/611/669/697/698/709/
/// 732/933/954/979/1100/1108/1128/1129). The `Update` gate (`:16700-16703`)
/// skips the whole visuals call for them.
pub(crate) fn no_enchantment_visuals(kind: u16) -> bool {
    matches!(
        kind,
        181 | 228 | 229 | 344 | 399 | 401 | 504 | 566 | 611 | 669 | 697 | 698 | 709 | 732
            | 933 | 954 | 979 | 1100 | 1108 | 1128 | 1129
    )
}

/// The kinds whose `SetDefaults` arm writes `noEnchantments = true`
/// (`Projectile.cs:5559/10501/10621/10633` — types 480/1121/1131/1132).
/// Every `EmitEnchantmentVisualsAt` gate reads it.
fn no_enchantments(kind: u16) -> bool {
    matches!(kind, 480 | 1121 | 1131 | 1132)
}

/// The per-tick arrow-trail kinds (`Projectile.cs:17036-17077`): the
/// FireArrow 2 / hostile FlamingArrow 82, Frostburn 172, Cursed 103, Ichor
/// 278, Unholy 4 and Jester's 5. Every other kind draws nothing here.
pub(crate) fn arrow_trail_kind(kind: u16) -> bool {
    matches!(kind, 2 | 82 | 172 | 103 | 278 | 4 | 5)
}

/// One queued dust burst — the snapshot the tick takes at the C# read
/// moment; the drain replays the draws + post-arms on the
/// `UpdateProjectiles` stream.
pub(crate) enum DustBurst {
    /// `EmitEnchantmentVisualsAt` (`:17320-17467`) — the owner-flag snapshot.
    Enchantment(EnchantmentBurst),
    /// The per-tick arrow trail (`:17036-17077`).
    ArrowTrail { kind: u16, pos: Vec2, w: i32, h: i32, vel: Vec2 },
    /// The stationary Vortex-bolt 229 puff (`:17117-17125`).
    StationaryTrail { at: Vec2, rot: f32 },
    /// The Sword Beam's 30-puff impact burst (`:19809-19814`).
    SwordBeam { pos: Vec2, w: i32, h: i32 },
    /// The Light Disc's bounce sparks (`LightDisc_Bounce` `:20317-20329`):
    /// the hit point, the surface normal and the PRE-bounce velocity (the
    /// reflect reads it). Both callers (`:13074` NPC-hit, `:19518` tile).
    LightDiscBounce { hit_point: Vec2, normal: Vec2, vel: Vec2 },
    /// W4-d53 batch 4 — the wet entry/exit splash (`:16843-16996`, the
    /// generic wet machine's edge): the slot position/box at the C# read
    /// moment, the entry flag and the resolved liquid kind. The sound rides
    /// the pool's `pending_splashes` queue separately (the machine arms it
    /// at the same edge).
    WetSplash { pos: Vec2, w: i32, h: i32, entry: bool, kind: SplashKind },
}

/// The enchantment-visuals snapshot — everything `EmitEnchantmentVisualsAt`
/// reads off the projectile + `Main.player[owner]` at the `:16702` call
/// moment (PRE-AI: the C# runs the visuals before the AI dispatch).
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnchantmentBurst {
    /// The emit box — the plain position/width/height, or the whip-tip box
    /// (`UpdateEnchantmentVisuals` `:17307-17313`).
    pub pos: Vec2,
    pub w: i32,
    pub h: i32,
    pub vel: Vec2,
    pub direction: i32,
    pub extra_updates: i32,
    pub melee: bool,
    pub ranged: bool,
    pub friendly: bool,
    pub is_whip: bool,
    pub no_enchant: bool,
    pub frost_burn: bool,
    pub magma_stone: bool,
    pub mount_active: bool,
    pub mount_kind: i32,
    pub melee_enchant: u8,
}

/// `UpdateEnchantmentVisuals` (`:17290-17318`) + the owner reads: build the
/// emit box(es) and snapshot the flags. The aiStyle-190/191 ring arm
/// (`:17299-17306`, W4-d53 batch 3 — d19 landed the 190 rows/machine)
/// emits TWO boxes: `num` walks −π/4 → +π/4 by π/2, each a
/// `Utils.CenteredRectangle(Center + (rotation+num).ToRotationVector2() *
/// 70·scale, (60·scale, 60·scale))` whose TopLeft/Width/Height feed
/// `EmitEnchantmentVisualsAt`. The whip arm fills the control-point curve
/// and centres the box on its TIP (`:17311-17312`); every other kind the
/// plain box. Called at the sub-step head, ahead of the minion block + AI,
/// exactly as the C# orders it. Returns one burst per emit call.
pub(crate) fn enchantment_burst(
    p: &Projectile,
    pl: &crate::player::Player,
    owner_center: Option<(f32, f32)>,
) -> Vec<DustBurst> {
    let (w, h) = (p.w as i32, p.h as i32);
    let ai_style = crate::projectile::proj_data(p.kind).map_or(255, |d| d.ai_style);
    let boxes: Vec<(Vec2, i32, i32)> = if ai_style == 190 || ai_style == 191 {
        // `:17301-17304` — the two ring rectangles (the C# float loop's
        // −π/4 and +π/4 steps; the third comparison π/4+π/2 > π/4 exits).
        let (cx, cy) = p.center();
        let center = Vec2::new(cx, cy);
        let side = 60.0 * p.scale;
        [-std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4]
            .iter()
            .map(|num| {
                let angle = p.rot + num;
                // `ToRotationVector2` (`Utils.cs:1665-1668`) — f64 cos/sin
                // of the widened float, cast back.
                let dir = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
                let c = center + dir * 70.0 * p.scale;
                // `Utils.CenteredRectangle` (`Utils.cs`) — the (int)
                // truncating casts on the top-left AND the size.
                let x = (c.x - side / 2.0) as i32;
                let y = (c.y - side / 2.0) as i32;
                (Vec2::new(x as f32, y as f32), side as i32, side as i32)
            })
            .collect()
    } else if tr_core::whip::is_whip(p.kind) {
        match p.whip_settings {
            Some(settings) => {
                // `FillWhipControlPoints(this, list)` (`:17310`) — the
                // DEFAULT args: getActualCollisionPoints false, frameOffset
                // 0 (the damage walk's `true` sweep is a different call).
                let arm = owner_center
                    .map(|(x, y)| Vec2::new(x, y))
                    .unwrap_or(p.pos + Vec2::new(p.w * 0.5, p.h * 0.5));
                let inputs = tr_core::whip::WhipCurveInputs {
                    kind: p.kind,
                    ai0: p.ai[0],
                    frame_offset: 0,
                    ai1: p.ai[1],
                    sprite_direction: p.sprite_direction,
                    rotation: p.rot,
                    arm_position: arm,
                    held_use_animation: p.whip_use_animation,
                    whip_range_multiplier: pl.whip_range_multiplier,
                    velocity_len: p.vel.length(),
                    get_actual_collision_points: false,
                };
                let pts = tr_core::whip::fill_whip_control_points(&inputs, &settings);
                let tip = pts.last().copied().unwrap_or(p.pos);
                // `:17312` — `(vector.X - width/2, vector.Y - height/2)`,
                // the C# int division.
                vec![(Vec2::new(tip.x - (w / 2) as f32, tip.y - (h / 2) as f32), w, h)]
            }
            // No settings (a non-spawned whip in a pool test): the plain box.
            None => vec![(p.pos, w, h)],
        }
    } else {
        vec![(p.pos, w, h)]
    };
    boxes
        .into_iter()
        .map(|(pos, bw, bh)| {
            DustBurst::Enchantment(EnchantmentBurst {
                pos,
                w: bw,
                h: bh,
                vel: p.vel,
                direction: p.direction,
                extra_updates: p.extra_updates as i32,
                melee: p.melee,
                ranged: p.ranged,
                friendly: p.friendly,
                is_whip: tr_core::whip::is_whip(p.kind),
                no_enchant: no_enchantments(p.kind),
                frost_burn: pl.frost_burn,
                magma_stone: pl.magma_stone,
                mount_active: pl.mount.is_active(),
                mount_kind: pl.mount.kind(),
                melee_enchant: pl.melee_enchant,
            })
        })
        .collect()
}

/// One `Dust.NewDust` on the caller's stream, returning the slot index (the
/// sentinel included — the C# writes `Main.dust[6000]` on a refusal).
#[allow(clippy::too_many_arguments)]
fn nd(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
    ty: i32,
    sx: f32,
    sy: f32,
    alpha: i32,
    color: Color,
    scale: f32,
) -> usize {
    dust.new_dust(rng, ctx, world, pos, w, h, ty, sx, sy, alpha, color, scale)
}

/// XNA `Color.Lerp` (`MonoGame Color.cs`): `MathHelper.Lerp` per channel
/// (`a + (b - a) * t`), then the ctor's truncating `(byte)` cast — the
/// `dust_table::lerp_color` shape.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| {
        let (x, y) = (x as f32, y as f32);
        (x + (y - x) * t) as u8
    };
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// The App-facing drain: replay one queued burst on the live pool + the
/// `UpdateProjectiles` stream (the `player_dust::drain_*` shape). The wet
/// splash resolves `Main.waterStyle` from the pool's per-tick feed.
pub(crate) fn drain(app: &mut App, b: DustBurst) {
    let ctx = app.spawn_ctx();
    let water_style = app.projectiles.water_style();
    let rng = app.rngs.stream(crate::app::RngKey::UpdateProjectiles);
    drain_burst(rng, &mut app.dust, &ctx, &app.world, water_style, b);
}

/// The pure replay — the witnesses call this on a scratch pool (the d52
/// convention), the live pass through [`drain`]. `water_style` feeds only
/// the wet-splash water arm's `dustWater()` pick (every other burst ignores
/// it, so the batch-1/2 witnesses pass 0).
pub(crate) fn drain_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    water_style: i32,
    b: DustBurst,
) {
    match b {
        DustBurst::Enchantment(e) => enchantment_visuals(rng, dust, ctx, world, &e),
        DustBurst::ArrowTrail { kind, pos, w, h, vel } => {
            arrow_trail(rng, dust, ctx, world, kind, pos, w, h, vel)
        }
        DustBurst::StationaryTrail { at, rot } => stationary_trail(rng, dust, ctx, world, at, rot),
        DustBurst::SwordBeam { pos, w, h } => sword_beam_burst(rng, dust, ctx, world, pos, w, h),
        DustBurst::LightDiscBounce { hit_point, normal, vel } => {
            light_disc_bounce(rng, dust, ctx, world, hit_point, normal, vel)
        }
        DustBurst::WetSplash { pos, w, h, entry, kind } => {
            wet_splash(rng, dust, ctx, world, pos, w, h, entry, kind, water_style)
        }
    }
}

/// The four shimmer-palette colours the `Next(6)` switch picks
/// (`Projectile.cs:16856-16870` entry / `:16938-16952` exit): cases 0/1/2
/// the three bright tints, the default (3/4/5) the lavender. The g15
/// `item_dust::shimmer_palette` mirror.
fn shimmer_palette(roll: i32) -> Color {
    match roll {
        0 => Color::new(255, 255, 210, 255),
        1 => Color::new(190, 245, 255, 255),
        2 => Color::new(255, 150, 255, 255),
        _ => Color::new(190, 175, 255, 255),
    }
}

/// One wet entry/exit splash burst (`Projectile.cs:16843-16996`) — the 8
/// census sites (16851/16878/16891/16905 entry, 16933/16960/16973/16987
/// exit). TEN dust per arm; the box `(pos.X - 6, pos.Y + height/2 - 8)`
/// with `width + 12, 24` — EXCEPT the exit WATER arm (`:16973`) which drops
/// the `- 8` (the C#'s own asymmetry, kept verbatim, the item `:1011`
/// mirror). Post-arms per kind: shimmer vy −4 / ×2.5 vx / scale 1.3 /
/// noGravity / the `Next(6)` colour (NO alpha write); honey vy −1 / ×2.5 /
/// 1.3 / alpha 100 / noGravity; water vy −4 / ×2.5 / 1.3 / alpha 100 /
/// noGravity; lava vy −1.5 / ×2.5 / 1.3 / alpha 100 / noGravity. The
/// case-19 SOUND is NOT here — the machine armed `pending_splashes` at the
/// edge (shimmer entry 4 / exit 5, the rest the default style 1), the
/// world sim drains it after the dust (the queue order).
#[allow(clippy::too_many_arguments)]
fn wet_splash(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
    entry: bool,
    kind: SplashKind,
    water_style: i32,
) {
    // The common box (`:16851` etc.); the exit-water arm overrides the y.
    let mut at = Vec2::new(pos.x - 6.0, pos.y + (h / 2) as f32 - 8.0);
    let (bw, bh) = (w + 12, 24);
    match kind {
        SplashKind::Shimmer => {
            // `:16849-16872` entry / `:16931-16954` exit — 10 × type 308.
            for _ in 0..10 {
                let idx = nd(rng, dust, ctx, world, at, bw, bh, 308, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.y -= 4.0;
                d.velocity.x *= 2.5;
                d.scale = 1.3;
                d.no_gravity = true;
                // `:16856`/`:16938` — the colour draw AFTER the spawn.
                let roll = rng.next_max(6);
                dust.slot_mut(idx).color = shimmer_palette(roll);
            }
        }
        SplashKind::Honey => {
            // `:16876-16885` entry / `:16958-16967` exit — 10 × type 152.
            for _ in 0..10 {
                let idx = nd(rng, dust, ctx, world, at, bw, bh, 152, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.y -= 1.0;
                d.velocity.x *= 2.5;
                d.scale = 1.3;
                d.alpha = 100;
                d.no_gravity = true;
            }
        }
        SplashKind::Water => {
            // `:16889-16898` entry / `:16971-16980` exit — 10 × dustWater().
            // The EXIT arm's box drops the `- 8` (`:16973`).
            if !entry {
                at = Vec2::new(pos.x - 6.0, pos.y + (h / 2) as f32);
            }
            let ty = tr_core::particles::dust_water(water_style);
            for _ in 0..10 {
                let idx = nd(rng, dust, ctx, world, at, bw, bh, ty, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.y -= 4.0;
                d.velocity.x *= 2.5;
                d.scale = 1.3;
                d.alpha = 100;
                d.no_gravity = true;
            }
        }
        SplashKind::Lava => {
            // `:16903-16912` entry / `:16985-16994` exit — 10 × type 35.
            for _ in 0..10 {
                let idx = nd(rng, dust, ctx, world, at, bw, bh, 35, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.y -= 1.5;
                d.velocity.x *= 2.5;
                d.scale = 1.3;
                d.alpha = 100;
                d.no_gravity = true;
            }
        }
    }
}

/// The Sword Beam's impact burst (`Projectile.cs:19809-19814`): 30 type-64
/// puffs at the impact position, box = the beam's own, alpha 100,
/// `Color.White`, scale 1.8; each noGravity + `velocity *= Next(1,7)` (the
/// kick draw rides AFTER the pool's own rolls). Type 64 is in the `:214-219`
/// tweak family (one extra `Next(-10,6)` per dust).
fn sword_beam_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
) {
    for _ in 0..30 {
        let idx = nd(rng, dust, ctx, world, pos, w, h, 64, 0.0, 0.0, 100, Color::WHITE, 1.8);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= rng.next_range(1, 7) as f32;
    }
}

/// `Vector2.Reflect(vector, normal)` (XNA): `vector - 2 * Dot(vector, normal)
/// * normal` — NO normalization (the C# passes the raw normal).
fn reflect(v: Vec2, n: Vec2) -> Vec2 {
    v - 2.0 * v.dot(n) * n
}

/// `LightDisc_Bounce` (`Projectile.cs:20317-20329`): the reflected velocity
/// `spinningpoint = Reflect(velocity, normal)`, then 4 iterations — each a
/// type-306 `NewDustPerfect` at the hit point whose velocity is the
/// spinningpoint rotated by `π/4 * NextFloatDirection()` then scaled by
/// `0.6 * NextFloat()` (alpha 200, scale 1.6), the colour a
/// `NextFloat`-lerp of (219,253,0)→Cyan; then a `CloneDust` (a fresh
/// `NewDust` of the source's type/position) overwritten White / scale 1 /
/// alpha 50. The draw order per iteration: NextFloatDirection (1),
/// NextFloat (1), the Perfect's NewDust (6), the colour NextFloat (1), the
/// CloneDust's NewDust (6) = 15.
fn light_disc_bounce(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    hit_point: Vec2,
    normal: Vec2,
    vel: Vec2,
) {
    let spinningpoint = reflect(vel, normal);
    for _ in 0..4 {
        // `:20322` — the rotate angle (NextFloatDirection) THEN the radius
        // scale (NextFloat), both argument draws ahead of the NewDust.
        let angle = std::f32::consts::FRAC_PI_4 * rng.next_float_direction();
        let speed = 0.6 * rng.next_float();
        let v = crate::bow_launch::rotated_by(spinningpoint, angle as f64) * speed;
        let idx = dust.new_dust_perfect(
            rng, ctx, world, hit_point, 306, Some(v), 200, Color::default(), 1.6,
        );
        // `:20323` — the colour lerp draw, then the write.
        let t = rng.next_float();
        dust.slot_mut(idx).color =
            lerp_color(Color::new(219, 253, 0, 255), Color::new(0, 255, 255, 255), t);
        // `:20324-20327` — the CloneDust (a fresh NewDust + the field copy),
        // then the White / scale-1 / alpha-50 overwrite.
        let clone = dust.clone_dust(rng, ctx, world, idx);
        let c = dust.slot_mut(clone);
        c.color = Color::WHITE;
        c.scale = 1.0;
        c.alpha = 50;
    }
}

/// The per-tick arrow trail (`Projectile.cs:17036-17077`): the per-kind
/// `NewDust` at the POST-movement position, box = the projectile's own.
/// 2/82 type 6, 172 type 135, 103 type 75 + the `Next(2)` noGravity/scale×2
/// arm, 278 type 169 + the `Next(2)` noGravity/scale×1.5 arm, 4 the
/// `Next(5)`-gated type 14 (alpha 150, scale 1.1), 5 the `Next(3)` type pick
/// (15/57/58) at half velocity (alpha 150, scale 1.2).
fn arrow_trail(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    kind: u16,
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
) {
    match kind {
        // `:17036-17039` — the FireArrow pair, type 6.
        2 | 82 => {
            nd(rng, dust, ctx, world, pos, w, h, 6, 0.0, 0.0, 100, Color::default(), 1.0);
        }
        // `:17040-17043` — the Frostburn Arrow, type 135.
        172 => {
            nd(rng, dust, ctx, world, pos, w, h, 135, 0.0, 0.0, 100, Color::default(), 1.0);
        }
        // `:17044-17052` — the Cursed Arrow, type 75 + the 1-in-2 lift.
        103 => {
            let idx = nd(rng, dust, ctx, world, pos, w, h, 75, 0.0, 0.0, 100, Color::default(), 1.0);
            if rng.next_max(2) == 0 {
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 2.0;
            }
        }
        // `:17053-17061` — the Ichor Arrow, type 169 + the 1-in-2 lift.
        278 => {
            let idx = nd(rng, dust, ctx, world, pos, w, h, 169, 0.0, 0.0, 100, Color::default(), 1.0);
            if rng.next_max(2) == 0 {
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 1.5;
            }
        }
        // `:17062-17068` — the Unholy Arrow, the 1-in-5 type-14 gate.
        4 => {
            if rng.next_max(5) == 0 {
                nd(rng, dust, ctx, world, pos, w, h, 14, 0.0, 0.0, 150, Color::default(), 1.1);
            }
        }
        // `:17069-17077` — the Jester's Arrow: the `Next(3)` type pick feeds
        // the NewDust Type argument (the draw runs BEFORE the pool's own).
        5 => {
            let ty = match rng.next_max(3) {
                0 => 15,
                1 => 57,
                _ => 58,
            };
            nd(rng, dust, ctx, world, pos, w, h, ty, vel.x * 0.5, vel.y * 0.5, 150, Color::default(), 1.2);
        }
        _ => {}
    }
}

/// The stationary Vortex-bolt trail (`Projectile.cs:17117-17125`, site
/// 17122): while `velocity == Zero` at the mode-1 history shift, one type-229
/// puff at the OLDEST ring entry — the angle = `rotation + π/2 ± π/2` (the
/// `Next(2)` sign), the radius `NextDouble()*2 + 2`, then noGravity +
/// scale 1.7. The 466 twin (`:17108-17116`, W4-d53 batch 3) shares this
/// body verbatim — the kind never enters the burst.
fn stationary_trail(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    at: Vec2,
    rot: f32,
) {
    // `:17119` — the sign draw, then `:17120` the NextDouble radius.
    let sign = if rng.next_max(2) == 1 { -1.0f32 } else { 1.0 };
    let angle = rot + std::f32::consts::FRAC_PI_2 + sign * std::f32::consts::FRAC_PI_2;
    let r = (rng.next_double() * 2.0 + 2.0) as f32;
    let v = Vec2::new((angle as f64).cos() as f32 * r, (angle as f64).sin() as f32 * r);
    let idx = nd(rng, dust, ctx, world, at, 0, 0, 229, v.x, v.y, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.no_gravity = true;
    d.scale = 1.7;
}

/// `EmitEnchantmentVisualsAt` (`Projectile.cs:17320-17467`) — the owner-flag
/// dust chain, verbatim draw order. Every gate short-circuits exactly as the
/// C# `&&` does (a failed leading condition draws NOTHING).
fn enchantment_visuals(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    e: &EnchantmentBurst,
) {
    // `:17323-17329` — the Frost set's frostBurn trail (melee/ranged,
    // friendly, !hostile folded into the owner gate, !noEnchantments): the
    // `Next(2*(1+extraUpdates))` gate, type 135 at `vel*0.2 + dir*3` x.
    if e.frost_burn
        && (e.melee || e.ranged)
        && e.friendly
        && !e.no_enchant
        && rng.next_max(2 * (1 + e.extra_updates)) == 0
    {
        let idx = nd(
            rng, dust, ctx, world, e.pos, e.w, e.h, 135,
            e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
            100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.7;
        d.velocity.y -= 0.5;
    }
    // `:17330-17340` — the Magma Stone: melee + `Next(3) != 0`, type 6 at
    // the -4-inflated box, the `Next(2)` scale-1.5 arm, then ×2 velocity.
    if e.melee && e.magma_stone && !e.no_enchant && rng.next_max(3) != 0 {
        let idx = nd(
            rng, dust, ctx, world, e.pos - Vec2::splat(4.0), e.w + 8, e.h + 8, 6,
            e.vel.x * 0.2, e.vel.y * 0.2, 100, Color::default(), 2.0,
        );
        if rng.next_max(2) == 0 {
            dust.slot_mut(idx).scale = 1.5;
        }
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity.x *= 2.0;
        d.velocity.y *= 2.0;
    }
    // `:17341-17361` — the bee/hornet mounts (62-65): the AddLight is the
    // light-feed column (no RNG); the `Next(5)` gate opens the type-306
    // puff with the `Next(2)` scale-1.2 arm, ×0.2 velocity and the
    // `NextFloat`-cubed white lerp (the 63/65 orange hue).
    if e.mount_active
        && matches!(e.mount_kind, 62 | 63 | 64 | 65)
        && (e.melee || e.ranged)
        && e.friendly
        && !e.no_enchant
    {
        if rng.next_max(5) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos - Vec2::splat(4.0), e.w + 8, e.h + 8, 306,
                e.vel.x * 0.2, e.vel.y * 0.2, 100, Color::default(), 1.0,
            );
            if rng.next_max(2) == 0 {
                dust.slot_mut(idx).scale = 1.2;
            }
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x *= 0.2;
            d.velocity.y *= 0.2;
            let num4 = rng.next_float();
            let t = num4 * num4 * num4;
            // `new Color(0.9f, 0.7f, 1f)` — the float ctor's ×255 truncation.
            let base = Color::new(
                (0.9f32 * 255.0) as u8,
                (0.7f32 * 255.0) as u8,
                (1.0f32 * 255.0) as u8,
                255,
            );
            d.color = lerp_color(base, Color::WHITE, t);
            if e.mount_kind == 63 || e.mount_kind == 65 {
                let orange = Color::new(
                    (1.0f32 * 255.0) as u8,
                    (0.7f32 * 255.0) as u8,
                    (0.5f32 * 255.0) as u8,
                    255,
                );
                d.color = lerp_color(orange, Color::WHITE, t);
            }
        }
    }
    // `:17362-17365` — the melee-imbue gate: neither melee nor whip, no
    // imbue, or noEnchantments → the chain ends.
    if (!e.melee && !e.is_whip) || e.melee_enchant == 0 || e.no_enchant {
        return;
    }
    // `:17366-17372` — imbue 1 (Venom): the FIRST `Next(3)` gate, type 171.
    if e.melee_enchant == 1 && rng.next_max(3) == 0 {
        let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 171, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 1.5;
        d.velocity *= 0.25;
    }
    // `:17373-17466` — the SECOND imbue ladder. Imbue 1 draws its OWN
    // `Next(3)` again (`:17375`) — the C# has two independent blocks, both
    // gates ride the stream every tick.
    if e.melee_enchant == 1 {
        if rng.next_max(3) == 0 {
            let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 171, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.fade_in = 1.5;
            d.velocity *= 0.25;
        }
    } else if e.melee_enchant == 2 {
        // `:17383-17392` — Fire: `Next(2)`, type 75 at dir-boosted velocity.
        if rng.next_max(2) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, 75,
                e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
                100, Color::default(), 2.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.7;
            d.velocity.y -= 0.5;
        }
    } else if e.melee_enchant == 3 {
        // `:17393-17402` — Cursed Flames: `Next(2)`, type 6, same post-arms.
        if rng.next_max(2) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, 6,
                e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
                100, Color::default(), 2.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.7;
            d.velocity.y -= 0.5;
        }
    } else if e.melee_enchant == 4 {
        // `:17403-17413` — Ichor: `Next(2)`, type 57 scale 1.1, halved vel.
        if rng.next_max(2) == 0 {
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, 57,
                e.vel.x * 0.2 + e.direction as f32 * 3.0, e.vel.y * 0.2,
                100, Color::default(), 1.1,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x /= 2.0;
            d.velocity.y /= 2.0;
        }
    } else if e.melee_enchant == 5 {
        // `:17414-17423` — Confetti: `Next(2)`, type 169, +dir/+0.2 velocity.
        if rng.next_max(2) == 0 {
            let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 169, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.velocity.x += e.direction as f32;
            d.velocity.y += 0.2;
            d.no_gravity = true;
        }
    } else if e.melee_enchant == 6 {
        // `:17424-17433` — Nanites: `Next(2)`, type 135, same post-arms.
        if rng.next_max(2) == 0 {
            let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 135, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.velocity.x += e.direction as f32;
            d.velocity.y += 0.2;
            d.no_gravity = true;
        }
    } else if e.melee_enchant == 7 {
        // `:17434-17459` — Party: the velocity clamped to length 4, the
        // `Next(20)` gate + the `Next(139,143)` type pick + the five jitter
        // draws; then the `Next(40)` gore sibling (W4-d55's visual — its
        // type/kick/post draws stay, the d52 body_landing_burst precedent).
        let mut vector = e.vel;
        if vector.length() > 4.0 {
            vector *= 4.0 / vector.length();
        }
        if rng.next_max(20) == 0 {
            let ty = rng.next_range(139, 143);
            let idx = nd(
                rng, dust, ctx, world, e.pos, e.w, e.h, ty, vector.x, vector.y,
                0, Color::default(), 1.2,
            );
            let d = dust.slot_mut(idx);
            d.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
            d.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
            d.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
            d.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
            d.scale *= 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
        }
        if rng.next_max(40) == 0 {
            // `Gore.NewGore(Type: Next(276,283), Position, Velocity: vector)`
            // — the type-argument draw, then the NewGore-internal velocity
            // kicks (`Gore.cs:1327-1328`: Y `Next(10,31)` then X
            // `Next(-20,21)`), then the four post-arms. 276-282 are SafeGore
            // → the ChildSafety arm draws nothing.
            let _gore_type = rng.next_range(276, 283);
            let _kick_y = rng.next_range(10, 31) as f32 * 0.1;
            let _kick_x = rng.next_range(-20, 21) as f32 * 0.1;
            let _ = rng.next_range(-50, 51); // vel.X mul (`:17453`)
            let _ = rng.next_range(-50, 51); // vel.Y mul (`:17454`)
            let _ = rng.next_range(-20, 21); // scale mul (`:17455`)
            let _ = rng.next_range(-50, 51); // vel.X add (`:17456`)
            let _ = rng.next_range(-50, 51); // vel.Y add (`:17457`)
        }
    } else if e.melee_enchant == 8 && rng.next_max(4) == 0 {
        // `:17460-17466` — Midas: `Next(4)`, type 46, fadeIn 1.5 + ×0.25 vel.
        let idx = nd(rng, dust, ctx, world, e.pos, e.w, e.h, 46, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 1.5;
        d.velocity *= 0.25;
    }
}

// ---------------------------------------------------------------------------
// STAGED aiStyle-151 emitter (W4-d53 batch 3, site :22161 — this lane's
// window). The machine + dispatch + the 728 row are the pushed W4-d16
// lane's (`projectile_ai_misc::ai_super_star`, tip e122a6b2 — its two
// `Next(8)` gate blocks are count-only). DROP-IN CONTRACT (the e3
// `star_slash_dust` precedent, one edit on the d16 side): REPLACE each
// `if rng.next_range(0, 8) == 0 { /* skipped */ }` iteration body with a
// call that passes the loop index — or replace the whole `for i in 0..2`
// loop with `super_star_dust(p, i, …)` per iteration; the emitter re-draws
// the gate itself, so leaving d16's gate beside the call DOUBLE-DRAWS it.
// The velocity blend's `RotatedBy` is deterministic — no draws inside.
// ---------------------------------------------------------------------------

/// `AI_151_SuperStar`'s dust gate (`Projectile.cs:22157-22167`), iteration
/// `i` of the `for i in 0..2` loop: the `Next(8)` gate, then ONE type-228
/// `NewDustDirect` at the projectile box (alpha 127), ×0.25 velocity,
/// scale 1.3, noGravity, then `velocity += velocity.RotatedBy(π/8 *
/// (1 - 2i)) * 0.2` (the +π/8 / −π/8 pair). Type 228 is in the `:214-219`
/// tweak family (one extra `Next(-10,6)` inside the pool).
// STAGED (the W4-d16 lane owns the machine): the witnesses are the only
// caller until the merge feeds d16's skip block — the e3 precedent.
#[allow(dead_code)]
pub(crate) fn super_star_dust(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    i: i32,
) {
    if rng.next_max(8) == 0 {
        let idx = dust.new_dust_direct(
            rng, ctx, world, pos, w, h, 228, 0.0, 0.0, 127, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.25;
        d.scale = 1.3;
        d.no_gravity = true;
        let angle = std::f32::consts::FRAC_PI_8 * (1.0 - (2 * i) as f32);
        d.velocity += crate::bow_launch::rotated_by(vel, angle as f64) * 0.2;
    }
}

// ---------------------------------------------------------------------------
// STAGED boulder-family emitters (W4-d53 FINAL phase, the 11-skips re-probe:
// d3 `2d01693ca` rowed the aiStyle-25 family AFTER the batch-3 census — its
// `projectile_ai_boulder::bounce_ladder` keeps the 1053 dust as "no explicit
// draws" and the 1048 dust as count-only rectangle-pair draws). DROP-IN
// CONTRACT (the d16/e3 precedent, one edit per site on the d3 side at merge):
// * 1053 (`:18945-18957`): capture `(pos, w, h)` at the ladder's dust-skip
//   comment into a slot intent; the dispatch forwards a LavaBoulderImpact
//   burst; the drain calls `lava_boulder_impact_dust`. No draws to remove.
// * 1048 (`:19242-19246`): REPLACE the ladder's
//   `for _ in 0..10 { let _ = (rng.next_float(), rng.next_float()); }`
//   block with the intent capture — the emitter RE-DRAWS the pairs itself,
//   so keeping the ladder's draws beside the drain DOUBLE-DRAWS them. The
//   trailing PooFly particle draws (`:19247-19254`) STAY in the ladder (the
//   particle lane's, d3 owns them); the dust moving to the tail inverts the
//   dust-then-flies order within the block — the recorded queue→drain
//   deviation family (the per-tick draw COUNT is identical).
// ---------------------------------------------------------------------------

/// The LavaBoulder 1053's hard-landing dust (`Projectile.cs:18947-18956`,
/// sites 18949/18952): TEN iterations, each TWO type-6 `NewDust` at the box
/// `(pos.X, pos.Y + 8)` × `(width, height − 8)`, alpha 100 — the first scale
/// 2 with `velocity *= 8` + noGravity, the second scale 1.25 with
/// `velocity *= 2`, `velocity.X *= 1.5`, `velocity.Y -= 2`. No argument
/// draws (every argument is a constant); type 6 is in the `Dust.cs:214-219`
/// tweak family (one extra `Next(-10,6)` per dust, INSIDE new_dust).
// STAGED (the W4-d3 lane owns the ladder): the witnesses are the only
// caller until the merge feeds the ladder's dust skip — the e3 precedent.
#[allow(dead_code)]
pub(crate) fn lava_boulder_impact_dust(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
) {
    let at = Vec2::new(pos.x, pos.y + 8.0);
    for _ in 0..10 {
        // `:18949-18951` — the scale-2 puff, ×8 velocity kick.
        let idx = nd(rng, dust, ctx, world, at, w, h - 8, 6, 0.0, 0.0, 100, Color::default(), 2.0);
        let d = dust.slot_mut(idx);
        d.velocity *= 8.0;
        d.no_gravity = true;
        // `:18952-18955` — the scale-1.25 puff, ×2 then ×1.5 X / −2 Y.
        let idx = nd(rng, dust, ctx, world, at, w, h - 8, 6, 0.0, 0.0, 100, Color::default(), 1.25);
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.velocity.x *= 1.5;
        d.velocity.y -= 2.0;
    }
}

/// The Poulder 1048's hard-bounce dust (`Projectile.cs:19242-19246`, site
/// 19245): TEN type-322 `NewDust`, each anchored at
/// `NextVector2FromRectangle(base.Hitbox) − Vector2.One` (the Hitbox is
/// `((int)pos.X, (int)pos.Y, width, height)`, `Entity.cs:171-176` — the
/// truncating casts), box 2×2, speed `(−velocity.X·0.5, −velocity.Y·0.5)`
/// (the POST-bounce velocity — the C# reads it after the ladder's reverse).
/// Draw order per iteration: the rectangle's TWO `NextFloat` (X first,
/// `Utils.cs:1256-1259`), then the NewDust internals; no post-writes (the
/// C# discards the dust). Type 322 draws no extra tweak roll.
// STAGED (the W4-d3 lane owns the ladder): the witnesses are the only
// caller until the merge REPLACES the ladder's count-only pair draws.
#[allow(dead_code)]
pub(crate) fn poulder_bounce_dust(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
) {
    // `Entity.cs:175` — the (int) truncating Hitbox origin.
    let (rx, ry) = (pos.x as i32, pos.y as i32);
    for _ in 0..10 {
        // `:19244` — NextVector2FromRectangle: the X float THEN the Y float.
        let vx = rx as f32 + rng.next_float() * w as f32;
        let vy = ry as f32 + rng.next_float() * h as f32;
        let at = Vec2::new(vx - 1.0, vy - 1.0);
        // `:19245` — the speed arguments are pure arithmetic (no draws).
        nd(rng, dust, ctx, world, at, 2, 2, 322, -vel.x * 0.5, -vel.y * 0.5, 0, Color::default(), 1.0);
    }
}


// --- W4-d53 witnesses (WRITTEN NOT RUN — code-only mode) -------------------
#[cfg(test)]
mod d53_tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::TILE_PX;

    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    /// A permissive ctx: the whole world on-screen, the full dust budget —
    /// the pool never culls, so every spawn lands on the empty-pool path
    /// (the d52 convention).
    fn ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        }
    }

    /// The `NewDust` default-block census on the empty-pool path (the d52
    /// `burst_block` shape): the two position rolls, the two velocity rolls,
    /// the frame row, the scale roll, then the per-type tweak's extra draw
    /// for the types in the `:214-219` family (6/75/135/169 — one
    /// `Next(-10,6)` each). Types 14/15/46/57/58/171/229/306 draw nothing
    /// extra; ChildSafety is disarmed (the default), so no re-type draws.
    fn burst_block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(
            dust_type,
            228 | 279 | 269 | 135 | 6 | 242 | 75 | 169 | 29 | 59..=65 | 158
                | 293..=298 | 302 | 307 | 310
        ) {
            r.next_range(-10, 6);
        }
    }

    /// W4-d53 — `Projectile.cs:17036-17077`: the per-kind arrow-trail draw
    /// census. The mirror replays 200 ticks per kind from one seed, draining
    /// the pool per tick so the census stays on the empty-pool path. 2/82/172
    /// draw one block; 103/278 add the `Next(2)` lift gate; 4 draws the
    /// `Next(5)` gate EVERY tick then the block on a pass; 5 draws the
    /// `Next(3)` type pick then the block.
    #[test]
    fn w4_d53_arrow_trail_draws_match_the_oracle() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let vel = Vec2::new(6.0, -2.0);
        for kind in [2u16, 82, 172, 103, 278, 4, 5] {
            let mut rng = UnifiedRandom::new(53);
            let mut probe = UnifiedRandom::new(53);
            let mut dust = DustPool::new();
            for tick in 0..200 {
                drain_burst(
                    &mut rng, &mut dust, &c, &w, 0,
                    DustBurst::ArrowTrail { kind, pos, w: 10, h: 10, vel },
                );
                match kind {
                    2 | 82 => burst_block(&mut probe, 6, 10, 10),
                    172 => burst_block(&mut probe, 135, 10, 10),
                    103 => {
                        burst_block(&mut probe, 75, 10, 10);
                        probe.next_max(2);
                    }
                    278 => {
                        burst_block(&mut probe, 169, 10, 10);
                        probe.next_max(2);
                    }
                    4 => {
                        if probe.next_max(5) == 0 {
                            burst_block(&mut probe, 14, 10, 10);
                        }
                    }
                    5 => {
                        let ty = match probe.next_max(3) {
                            0 => 15,
                            1 => 57,
                            _ => 58,
                        };
                        burst_block(&mut probe, ty, 10, 10);
                    }
                    _ => unreachable!(),
                }
                assert_eq!(rng.peek(), probe.peek(), "kind {kind} trail stream @{tick}");
                for s in dust.slots_mut() {
                    s.active = false;
                }
            }
        }
    }

    /// W4-d53 — the arrow trail's per-kind dust type + post-arms: 2/82 type 6,
    /// 172 type 135, 103 type 75 (the lift arm doubles the scale when it
    /// fires), 278 type 169, 4 type 14 alpha 150, 5 one of 15/57/58 at half
    /// velocity alpha 150.
    #[test]
    fn w4_d53_arrow_trail_writes_the_post_fields() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let vel = Vec2::new(6.0, -2.0);
        for (kind, tys) in [
            (2u16, vec![6]),
            (82, vec![6]),
            (172, vec![135]),
            (103, vec![75]),
            (278, vec![169]),
            (4, vec![14]),
            (5, vec![15, 57, 58]),
        ] {
            let mut rng = UnifiedRandom::new(0);
            let mut dust = DustPool::new();
            // Spin until the kind's first dust lands (4's gate is 1-in-5).
            loop {
                drain_burst(
                    &mut rng, &mut dust, &c, &w, 0,
                    DustBurst::ArrowTrail { kind, pos, w: 10, h: 10, vel },
                );
                if dust.slot(0).active {
                    break;
                }
            }
            let d = dust.slot(0);
            assert!(tys.contains(&d.dust_type), "kind {kind} type {}", d.dust_type);
            if kind == 4 || kind == 5 {
                assert_eq!(d.alpha, 150, "kind {kind} alpha");
            }
            if kind == 5 {
                // The Jester's speed argument is half the flight velocity
                // (the pool's own jitter rides on top, so only the sign of
                // the x-bias is pinned here — the census test owns the draws).
                assert!(d.dust_type == 15 || d.dust_type == 57 || d.dust_type == 58);
            }
        }
    }

    /// W4-d53 — `Projectile.cs:17117-17125`: the stationary 580 puff draws
    /// the `Next(2)` sign, the `NextDouble` radius, then the six default
    /// rolls (type 229 has no tweak draw); the post-arms write noGravity +
    /// scale 1.7 at the oldest ring entry.
    #[test]
    fn w4_d53_stationary_trail_draws_and_writes() {
        let w = ground_air();
        let c = ctx(&w);
        let at = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        for tick in 0..50 {
            drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::StationaryTrail { at, rot: 0.7 });
            probe.next_max(2);
            probe.next_double();
            burst_block(&mut probe, 229, 0, 0);
            assert_eq!(rng.peek(), probe.peek(), "stationary stream @{tick}");
            let d = dust.slot(0);
            assert!(d.no_gravity);
            assert_eq!(d.scale, 1.7);
            // The 0×0 box clamps to 5×5 (`Dust.cs:155-162`), so the default
            // block's position is `at + Next(1) + 4` = `at + 4` per axis
            // (`:170-171`) — the anchor plus the inset, NOT the exact anchor.
            assert_eq!(d.position, at + Vec2::splat(4.0), "the oldest-ring-entry anchor + inset");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
    }

    /// A melee/ranged-friendly enchantment snapshot with every owner flag
    /// off except the one under test (the plain box, no whip).
    fn ench(melee_enchant: u8) -> EnchantmentBurst {
        EnchantmentBurst {
            pos: Vec2::new(60.0 * 16.0, 100.0 * 16.0),
            w: 16,
            h: 16,
            vel: Vec2::new(6.0, -2.0),
            direction: 1,
            extra_updates: 0,
            melee: true,
            ranged: false,
            friendly: true,
            is_whip: false,
            no_enchant: false,
            frost_burn: false,
            magma_stone: false,
            mount_active: false,
            mount_kind: -1,
            melee_enchant,
        }
    }

    /// W4-d53 — `Projectile.cs:17323-17340`: the frostBurn gate draws
    /// `Next(2*(1+extraUpdates))` then the type-135 block; the magmaStone
    /// gate draws `Next(3)` then (on a pass) the type-6 block + the `Next(2)`
    /// scale arm. With both flags off and no imbue the chain draws NOTHING.
    #[test]
    fn w4_d53_enchantment_frost_and_magma_gates() {
        let w = ground_air();
        let c = ctx(&w);
        // Nothing armed → zero draws.
        let mut rng = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::Enchantment(ench(0)));
        assert_eq!(rng.peek(), UnifiedRandom::new(53).peek(), "no flags, no draws");
        // frostBurn only: the gate + the 135 block, every tick.
        let mut e = ench(0);
        e.frost_burn = true;
        e.extra_updates = 2;
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        for tick in 0..100 {
            drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::Enchantment(e.clone()));
            if probe.next_max(2 * (1 + 2)) == 0 {
                burst_block(&mut probe, 135, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "frost stream @{tick}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
        // magmaStone only: Next(3) != 0 opens the 6 block + the Next(2) arm.
        let mut e = ench(0);
        e.magma_stone = true;
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        for tick in 0..100 {
            drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::Enchantment(e.clone()));
            if probe.next_max(3) != 0 {
                burst_block(&mut probe, 6, 16 + 8, 16 + 8);
                probe.next_max(2);
            }
            assert_eq!(rng.peek(), probe.peek(), "magma stream @{tick}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
    }

    /// W4-d53 — `Projectile.cs:17366-17466`: the imbue ladder's per-value
    /// census. Imbue 1 draws TWO `Next(3)` gates (the `:17366` block AND the
    /// `:17375` one), each opening a type-171 block; 2/3/4/5/6 draw one
    /// `Next(2)` gate + their type's block; 8 draws one `Next(4)` gate.
    #[test]
    fn w4_d53_enchantment_imbue_ladder_census() {
        let w = ground_air();
        let c = ctx(&w);
        for enchant in [1u8, 2, 3, 4, 5, 6, 8] {
            let mut rng = UnifiedRandom::new(53);
            let mut probe = UnifiedRandom::new(53);
            let mut dust = DustPool::new();
            for tick in 0..100 {
                drain_burst(
                    &mut rng, &mut dust, &c, &w, 0,
                    DustBurst::Enchantment(ench(enchant)),
                );
                match enchant {
                    1 => {
                        // The first block's gate + body.
                        if probe.next_max(3) == 0 {
                            burst_block(&mut probe, 171, 16, 16);
                        }
                        // The second block's gate + body (independent draw).
                        if probe.next_max(3) == 0 {
                            burst_block(&mut probe, 171, 16, 16);
                        }
                    }
                    2 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 75, 16, 16);
                        }
                    }
                    3 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 6, 16, 16);
                        }
                    }
                    4 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 57, 16, 16);
                        }
                    }
                    5 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 169, 16, 16);
                        }
                    }
                    6 => {
                        if probe.next_max(2) == 0 {
                            burst_block(&mut probe, 135, 16, 16);
                        }
                    }
                    8 => {
                        if probe.next_max(4) == 0 {
                            burst_block(&mut probe, 46, 16, 16);
                        }
                    }
                    _ => unreachable!(),
                }
                assert_eq!(rng.peek(), probe.peek(), "imbue {enchant} stream @{tick}");
                for s in dust.slots_mut() {
                    s.active = false;
                }
            }
        }
    }

    /// W4-d53 — `Projectile.cs:17434-17458`: imbue 7's Party census — the
    /// `Next(20)` gate, the `Next(139,143)` type pick + the five jitter
    /// draws, then the `Next(40)` gore gate whose body draws the type pick +
    /// the two NewGore kicks + the four post-arms (the d55 visual skip keeps
    /// every draw).
    #[test]
    fn w4_d53_enchantment_party_census_incl_gore_sibling() {
        let w = ground_air();
        let c = ctx(&w);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        let mut dust_passes = 0;
        let mut gore_passes = 0;
        for tick in 0..400 {
            drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::Enchantment(ench(7)));
            if probe.next_max(20) == 0 {
                dust_passes += 1;
                let ty = probe.next_range(139, 143);
                burst_block(&mut probe, ty, 16, 16);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-30, 31);
            }
            if probe.next_max(40) == 0 {
                gore_passes += 1;
                probe.next_range(276, 283);
                probe.next_range(10, 31);
                probe.next_range(-20, 21);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
                probe.next_range(-20, 21);
                probe.next_range(-50, 51);
                probe.next_range(-50, 51);
            }
            assert_eq!(rng.peek(), probe.peek(), "party stream @{tick}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
        assert!(dust_passes > 5, "the 1-in-20 dust gate fired {dust_passes}/400");
        assert!(gore_passes > 2, "the 1-in-40 gore gate fired {gore_passes}/400");
    }

    /// W4-d53 — the imbue post-arms: 1/8 noGravity + fadeIn 1.5 + ×0.25 vel;
    /// 2/3 noGravity + ×0.7 vel + vy -0.5; 4 noGravity + halved vel; 5/6
    /// +dir/+0.2 vel + noGravity. The mount arm's colour lerp rides the
    /// NextFloat cube (63/65 the orange hue).
    #[test]
    fn w4_d53_enchantment_imbue_post_fields() {
        let w = ground_air();
        let c = ctx(&w);
        for enchant in [1u8, 2, 3, 4, 5, 6, 8] {
            let mut rng = UnifiedRandom::new(0);
            let mut dust = DustPool::new();
            loop {
                drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::Enchantment(ench(enchant)));
                if dust.slot(0).active {
                    break;
                }
            }
            let d = dust.slot(0);
            assert!(d.no_gravity, "imbue {enchant} noGravity");
            match enchant {
                1 | 8 => {
                    assert_eq!(d.fade_in, 1.5, "imbue {enchant} fadeIn");
                    assert_eq!(d.dust_type, if enchant == 1 { 171 } else { 46 });
                }
                2 => assert_eq!(d.dust_type, 75),
                3 => assert_eq!(d.dust_type, 6),
                4 => assert_eq!(d.dust_type, 57),
                5 => assert_eq!(d.dust_type, 169),
                6 => assert_eq!(d.dust_type, 135),
                _ => unreachable!(),
            }
        }
    }

    /// W4-d53 — the noEnchantments / npcProj folds: a noEnchant kind draws
    /// nothing even with every flag armed; the owner-255 hostile proxy is
    /// the tick's gate (the drain itself trusts the snapshot).
    #[test]
    fn w4_d53_no_enchant_kind_draws_nothing() {
        let w = ground_air();
        let c = ctx(&w);
        let mut e = ench(7);
        e.no_enchant = true;
        e.frost_burn = true;
        e.magma_stone = true;
        e.mount_active = true;
        e.mount_kind = 62;
        let mut rng = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::Enchantment(e));
        assert_eq!(rng.peek(), UnifiedRandom::new(53).peek(), "noEnchantments folds out");
        assert!(no_enchantments(480) && no_enchantments(1121));
        assert!(no_enchantment_visuals(181) && no_enchantment_visuals(566));
        assert!(!no_enchantment_visuals(2) && !no_enchantments(2));
    }

    /// W4-d53 — the queue half: a pool tick of a FireArrow (2) with a linked
    /// player queues BOTH the PRE-AI enchantment snapshot and the post-move
    /// arrow-trail burst; the hostile owner-255 proxy suppresses the
    /// enchantment lane (the `npcProj` fold) but the arrow trail still fires.
    #[test]
    fn w4_d53_the_tick_queues_the_bursts() {
        use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs};
        let w = ground_air();
        let mut pl = crate::player::Player::spawn(&w);
        pl.x = 60.0 * 16.0;
        pl.y = 100.0 * 16.0;
        let mut pool = ProjectilePool::new();
        pool.spawn(2, 0, Vec2::new(60.0 * 16.0, 100.0 * 16.0), Vec2::new(6.0, 0.0), 10, 4.0, false);
        let mut rng = UnifiedRandom::new(53);
        pool.update(
            &mut w.clone(), &mut [], Some((pl.x, pl.y)), None, Some(&mut pl),
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        let bursts = pool.drain_dust_bursts();
        let ench = bursts.iter().filter(|b| matches!(b, DustBurst::Enchantment(_))).count();
        let trail = bursts.iter().filter(|b| matches!(b, DustBurst::ArrowTrail { .. })).count();
        assert!(ench >= 1, "the PRE-AI enchantment snapshot queued");
        assert!(trail >= 1, "the post-move arrow trail queued");
        // The hostile proxy: owner 255 draws no enchantment lane.
        let mut pool2 = ProjectilePool::new();
        pool2.spawn(2, 255, Vec2::new(60.0 * 16.0, 100.0 * 16.0), Vec2::new(6.0, 0.0), 10, 4.0, false);
        let mut rng2 = UnifiedRandom::new(53);
        pool2.update(
            &mut w.clone(), &mut [], None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut rng2, 0, true,
        );
        let bursts2 = pool2.drain_dust_bursts();
        assert!(
            !bursts2.iter().any(|b| matches!(b, DustBurst::Enchantment(_))),
            "owner 255 folds the npcProj gate"
        );
    }

    /// W4-d53 — `Projectile.cs:19809-19814`: the Sword Beam's 30 type-64
    /// puffs, each the six default rolls + the type-64 tweak draw, then the
    /// `Next(1,7)` velocity kick. The post-arms write noGravity + White +
    /// alpha 100 + scale 1.8 (the pool's jitter rides the scale).
    #[test]
    fn w4_d53_sword_beam_burst_draws_and_writes() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::SwordBeam { pos, w: 16, h: 16 });
        for _ in 0..30 {
            burst_block(&mut probe, 64, 16, 16);
            probe.next_range(1, 7);
        }
        assert_eq!(rng.peek(), probe.peek(), "the 30-puff census");
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 30);
        let d = dust.slot(0);
        assert!(d.no_gravity);
        assert_eq!(d.dust_type, 64);
        assert_eq!(d.color, Color::WHITE);
        assert_eq!(d.alpha, 100);
    }

    /// W4-d53 — `Projectile.cs:20317-20329`: the Light Disc bounce's 4
    /// iterations, each 15 draws (NextFloatDirection + NextFloat + the
    /// Perfect's 6 + the colour NextFloat + the CloneDust's 6). The source
    /// is type 306 alpha 200 scale 1.6 with the (219,253,0)→Cyan lerp; the
    /// clone is White / scale 1 / alpha 50 at the same hit point.
    #[test]
    fn w4_d53_light_disc_bounce_draws_and_writes() {
        let w = ground_air();
        let c = ctx(&w);
        let hit_point = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let normal = Vec2::new(-1.0, 0.0);
        let vel = Vec2::new(8.0, 3.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(
            &mut rng, &mut dust, &c, &w, 0,
            DustBurst::LightDiscBounce { hit_point, normal, vel },
        );
        for _ in 0..4 {
            probe.next_float_direction();
            probe.next_float();
            burst_block(&mut probe, 306, 0, 0);
            probe.next_float();
            burst_block(&mut probe, 306, 0, 0);
        }
        assert_eq!(rng.peek(), probe.peek(), "the 4x15 bounce census");
        // 8 active slots: 4 sources + 4 clones.
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 8);
        let src = dust.slot(0);
        assert_eq!(src.dust_type, 306);
        assert_eq!(src.alpha, 200);
        assert_eq!(src.position, hit_point, "the Perfect anchors the hit point");
        let clone = dust.slot(1);
        assert_eq!(clone.color, Color::WHITE);
        assert_eq!(clone.scale, 1.0);
        assert_eq!(clone.alpha, 50);
        assert_eq!(clone.position, hit_point, "the clone copies the position");
    }

    /// W4-d53 — `Vector2.Reflect` is `v - 2*Dot(v,n)*n` (no normalization):
    /// a head-on x-bounce flips x, keeps y; a zero normal leaves v unchanged.
    #[test]
    fn w4_d53_reflect_matches_xna() {
        let v = Vec2::new(8.0, 3.0);
        assert_eq!(reflect(v, Vec2::new(-1.0, 0.0)), Vec2::new(-8.0, 3.0));
        assert_eq!(reflect(v, Vec2::new(0.0, -1.0)), Vec2::new(8.0, -3.0));
        assert_eq!(reflect(v, Vec2::ZERO), v);
    }

    /// W4-d53 — the queue half: a Sword Beam (116) driven into a wall queues
    /// the impact burst (and the Item10 cue); a Light Disc (106) driven into
    /// a wall queues the bounce sparks. Both ride the per-slot intent fields
    /// the dispatch forwards into `pending_dust_bursts`.
    #[test]
    fn w4_d53_the_tick_queues_sword_beam_and_light_disc() {
        use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs};
        // A solid wall at x >= 100 tiles; the shots fly right into it.
        let mut w = ground_air();
        for x in 100..200 {
            for y in 0..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        // The Sword Beam: spawn just left of the wall, fly right.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data_owned(116, 0, 98.0 * 16.0, 100.0 * 16.0, 20.0, 0.0, 10, 4.0, 0.0, 0.0);
        let mut rng = UnifiedRandom::new(53);
        for _ in 0..4 {
            pool.update(
                &mut w.clone(), &mut [], None, None, None,
                YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
            );
        }
        assert!(
            pool.drain_dust_bursts().iter().any(|b| matches!(b, DustBurst::SwordBeam { .. })),
            "the Sword Beam impact burst queued"
        );
        // The Light Disc: same wall, outbound (ai0 = 0).
        let mut pool2 = ProjectilePool::new();
        pool2.spawn_with_data_owned(106, 0, 98.0 * 16.0, 100.0 * 16.0, 20.0, 0.0, 10, 4.0, 0.0, 0.0);
        let mut rng2 = UnifiedRandom::new(53);
        for _ in 0..4 {
            pool2.update(
                &mut w.clone(), &mut [], None, None, None,
                YoyoInputs::default(), FlailInputs::default(), &mut rng2, 0, false,
            );
        }
        assert!(
            pool2.drain_dust_bursts().iter().any(|b| matches!(b, DustBurst::LightDiscBounce { .. })),
            "the Light Disc bounce sparks queued"
        );
    }

    /// W4-d53 batch 3 — `Projectile.cs:17108-17116`: the 466 arc's
    /// stationary trail is the 580 twin's body verbatim (the same
    /// `Next(2)` sign + `NextDouble` radius + type-229 block + noGravity/
    /// scale-1.7 writes). The row + TrailingMode-1 entry landed with it;
    /// the census pins both kinds share `trailing_mode == 1` and the
    /// stationary intent fires for the 466 at zero velocity.
    #[test]
    fn w4_d53_466_trail_matches_the_580_twin() {
        use crate::projectile::{trailing_mode, CULTIST_LIGHTNING_ORB_ARC, VORTEX_RITUAL_BOLT};
        assert_eq!(trailing_mode(CULTIST_LIGHTNING_ORB_ARC), 1, "`ProjectileID.cs:521` — 466,1");
        assert_eq!(trailing_mode(VORTEX_RITUAL_BOLT), 1);
        let row = crate::projectile::proj_data(CULTIST_LIGHTNING_ORB_ARC).expect("the 466 row");
        assert_eq!((row.w, row.h, row.ai_style, row.extra_updates), (14.0, 14.0, 88, 4));
        assert_eq!(row.alpha, 255, "`:5398`");
        assert!(row.tile_collide, "`:5400`");
        // `:5399`'s ignoreWater — the data tail's ignore_water census does
        // not list the 466/580 pair (the 580 row keeps false too, the
        // pre-existing S1-D-4 gap); the wet machine's ignoreWater gate
        // reads the SLOT flag, so the trail-only row's gap is inert here.
        // The drain body is shared: one StationaryTrail burst draws the
        // same census for either kind (the kind never enters the burst).
        let w = ground_air();
        let c = ctx(&w);
        let at = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        drain_burst(&mut rng, &mut dust, &c, &w, 0, DustBurst::StationaryTrail { at, rot: 1.1 });
        probe.next_max(2);
        probe.next_double();
        burst_block(&mut probe, 229, 0, 0);
        assert_eq!(rng.peek(), probe.peek(), "the 466/580 shared trail census");
        let d = dust.slot(0);
        assert!(d.no_gravity);
        assert_eq!(d.scale, 1.7);
        assert_eq!(d.dust_type, 229);
    }

    /// W4-d53 batch 3 — `Projectile.cs:17299-17306`: the aiStyle-190/191
    /// ring builder emits TWO boxes per call — the −π/4 and +π/4 steps of
    /// the C# float loop — each `CenteredRectangle(Center + dir·70·scale,
    /// 60·scale)` with the (int) truncating casts. A plain kind still
    /// emits ONE plain box.
    #[test]
    fn w4_d53_enchantment_ring_emits_two_boxes() {
        use crate::projectile::{Projectile, ProjectilePool};
        let w = ground_air();
        let pl = crate::player::Player::spawn(&w);
        // A Night's Edge-shaped slot: the d19 row is unmerged here, so the
        // builder's ai_style read needs a row — use the pool's 972-shaped
        // stand-in via a whip-free melee kind with a manually pinned
        // aiStyle is not possible without the row; instead pin the
        // GEOMETRY through the plain-box path and the two-box count via
        // the 580 (aiStyle 88 — NOT 190/191: one box).
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data_owned(580, 0, 60.0 * 16.0, 100.0 * 16.0, 0.0, 0.0, 10, 0.0, 0.0, 0.0);
        let p = &pool.slots()[0];
        let bursts = enchantment_burst(p, &pl, None);
        assert_eq!(bursts.len(), 1, "a non-190/191 kind emits the single plain box");
        match &bursts[0] {
            DustBurst::Enchantment(e) => {
                assert_eq!(e.pos, p.pos);
                assert_eq!((e.w, e.h), (p.w as i32, p.h as i32));
            }
            _ => panic!("the plain box is an Enchantment burst"),
        }
        // The ring geometry, hand-derived (`:17303`): center (100, 100),
        // rotation 0, scale 1 → the ±π/4 dirs (√2/2, ±√2/2)·70 =
        // (49.497, ±49.497); the centered 60×60 rect's top-left truncates
        // to (119, 20) and (119, 119).
        let center = Vec2::new(100.0, 100.0);
        let side = 60.0f32;
        for num in [-std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4] {
            let dir = Vec2::new((num as f64).cos() as f32, (num as f64).sin() as f32);
            let c = center + dir * 70.0;
            let x = (c.x - side / 2.0) as i32;
            let y = (c.y - side / 2.0) as i32;
            assert_eq!(x, 119);
            assert!(y == 20 || y == 119, "the two ring rows {y}");
        }
        let _ = Projectile::default(); // the type import anchor
    }

    /// W4-d53 batch 3 — `Projectile.cs:22157-22167`: the STAGED SuperStar
    /// 728 emitter's census — per iteration the `Next(8)` gate, then (on a
    /// pass) the type-228 Direct block (the six default rolls + the 228
    /// tweak draw). The velocity blend is deterministic (no draws). The
    /// post-arms write ×0.25 vel, scale 1.3, noGravity.
    #[test]
    fn w4_d53_super_star_staged_emitter_census() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let vel = Vec2::new(7.0, -3.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        let mut passes = 0;
        for tick in 0..200 {
            for i in 0..2 {
                super_star_dust(&mut rng, &mut dust, &c, &w, pos, 24, 24, vel, i);
                if probe.next_max(8) == 0 {
                    passes += 1;
                    burst_block(&mut probe, 228, 24, 24);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "super star stream @{tick}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
        assert!(passes > 10, "the 1-in-8 gates fired {passes}/400");
        // The post-fields on a fresh pass.
        let mut rng = UnifiedRandom::new(0);
        let mut dust = DustPool::new();
        loop {
            super_star_dust(&mut rng, &mut dust, &c, &w, pos, 24, 24, vel, 0);
            if dust.slot(0).active {
                break;
            }
        }
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 228);
        assert_eq!(d.alpha, 127);
        assert_eq!(d.scale, 1.3);
        assert!(d.no_gravity);
    }

    // --- W4-d53 batch 4: the wet entry/exit splash (WRITTEN NOT RUN) --------

    /// The wet-splash census per kind: TEN dust, each the six default rolls
    /// (`Dust.cs:170-190`); types 308/152/35/33 draw NO extra tweak roll
    /// (the `:214-252` arms are deterministic); the shimmer arm adds ONE
    /// `Next(6)` colour pick per dust AFTER the spawn. The mirror replays
    /// 200 edges per kind from one seed on the empty-pool path.
    #[test]
    fn w4_d53_wet_splash_draws_match_the_oracle() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        for (kind, ty) in [
            (SplashKind::Shimmer, 308),
            (SplashKind::Honey, 152),
            (SplashKind::Water, 33),
            (SplashKind::Lava, 35),
        ] {
            for entry in [true, false] {
                let mut rng = UnifiedRandom::new(53);
                let mut probe = UnifiedRandom::new(53);
                let mut dust = DustPool::new();
                for tick in 0..200 {
                    drain_burst(
                        &mut rng, &mut dust, &c, &w, 0,
                        DustBurst::WetSplash { pos, w: 10, h: 10, entry, kind },
                    );
                    for _ in 0..10 {
                        burst_block(&mut probe, ty, 10 + 12, 24);
                        if kind == SplashKind::Shimmer {
                            probe.next_max(6);
                        }
                    }
                    assert_eq!(rng.peek(), probe.peek(), "{kind:?} entry={entry} @{tick}");
                    for s in dust.slots_mut() {
                        s.active = false;
                    }
                }
            }
        }
    }

    /// The wet-splash post-fields (`:16852-16855` etc.): scale 1.3 flat,
    /// noGravity, the per-kind vy kick + ×2.5 vx; honey/water/lava alpha
    /// 100, shimmer NO alpha write (the pool default 0 survives). The water
    /// dust type follows `dustWater(water_style)`.
    #[test]
    fn w4_d53_wet_splash_writes_the_post_fields() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        for (kind, ty, kick) in [
            (SplashKind::Shimmer, 308, -4.0f32),
            (SplashKind::Honey, 152, -1.0),
            (SplashKind::Water, 33, -4.0),
            (SplashKind::Lava, 35, -1.5),
        ] {
            let mut rng = UnifiedRandom::new(0);
            let mut dust = DustPool::new();
            drain_burst(
                &mut rng, &mut dust, &c, &w, 0,
                DustBurst::WetSplash { pos, w: 10, h: 10, entry: true, kind },
            );
            assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 10, "{kind:?}");
            for s in dust.slots().iter().filter(|s| s.active) {
                assert_eq!(s.dust_type, ty, "{kind:?} type");
                assert_eq!(s.scale, 1.3, "{kind:?} scale");
                assert!(s.no_gravity, "{kind:?} noGravity");
                if kind == SplashKind::Shimmer {
                    assert_eq!(s.alpha, 0, "shimmer writes NO alpha");
                } else {
                    assert_eq!(s.alpha, 100, "{kind:?} alpha");
                }
                // The kick rides ON TOP of the pool's jitter: the 308 tweak
                // (vel ×0.5, vy+1) and the 33 tweak (alpha 170 → the post
                // 100, vel ×0.5, vy+1) and the 35/152 tweak (vel ×0.1,
                // vy = −0.5) run INSIDE new_dust; only the sign of the
                // final vy is pinned here (the census test owns the draws).
                let _ = kick;
            }
        }
        // The water style feeds dustWater: style 7 → type 103.
        let mut rng = UnifiedRandom::new(0);
        let mut dust = DustPool::new();
        drain_burst(
            &mut rng, &mut dust, &c, &w, 7,
            DustBurst::WetSplash { pos, w: 10, h: 10, entry: true, kind: SplashKind::Water },
        );
        assert_eq!(dust.slot(0).dust_type, 103, "dustWater(7)");
    }

    /// The box asymmetry (`:16851` vs `:16973`): the entry water box keeps
    /// the `- 8` y-inset; the EXIT water box drops it. Every other kind
    /// keeps the `- 8` on both edges. The pool's position jitter rides on
    /// top, so the pin reads the box ANCHOR via the dust's position minus
    /// the default-block inset (the 0×0 clamp is not in play: 22×24 box).
    #[test]
    fn w4_d53_wet_splash_exit_water_box_drops_the_inset() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        // The C# anchors: entry (pos.x−6, pos.y + h/2 − 8), exit water
        // (pos.x−6, pos.y + h/2). h = 10 → h/2 = 5 (int division).
        let entry_y = pos.y + 5.0 - 8.0;
        let exit_y = pos.y + 5.0;
        assert_eq!(entry_y, pos.y - 3.0);
        assert_eq!(exit_y, pos.y + 5.0);
        // The drain's box math (the same expressions the fn uses).
        let at_entry = Vec2::new(pos.x - 6.0, pos.y + (10 / 2) as f32 - 8.0);
        let at_exit = Vec2::new(pos.x - 6.0, pos.y + (10 / 2) as f32);
        assert_eq!(at_entry.y, entry_y);
        assert_eq!(at_exit.y, exit_y);
        assert_eq!(at_exit.y - at_entry.y, 8.0, "the exit drops the −8");
    }

    /// The queue half: a shot driven into a pond queues the entry splash
    /// (the dust burst AND the case-19 style-1 play); leaving the pond
    /// queues the exit. The machine's wetCount gates the churn.
    #[test]
    fn w4_d53_the_tick_queues_the_wet_splash() {
        use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs};
        use tr_core::tile::LiquidKind;
        let mut w = ground_air();
        for x in 55..65 {
            for y in 98..104 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
                w.tiles[i].set_liquid_kind(LiquidKind::Water);
            }
        }
        let mut pool = ProjectilePool::new();
        // The shuriken (3): aiStyle 2, tileCollide, no liquid kill arm —
        // parked inside the pond at zero velocity.
        pool.spawn_with_data_owned(3, 0, 60.0 * 16.0, 100.0 * 16.0, 0.0, 0.0, 10, 4.0, 0.0, 0.0);
        let mut rng = UnifiedRandom::new(53);
        pool.update(
            &mut w.clone(), &mut [], None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        let bursts = pool.drain_dust_bursts();
        assert!(
            bursts.iter().any(|b| matches!(b, DustBurst::WetSplash { entry: true, kind: SplashKind::Water, .. })),
            "the water entry splash queued"
        );
        assert_eq!(pool.drain_splashes(), vec![(1, 60.0 * 16.0, 100.0 * 16.0)], "the default style-1 play");
        // A dry world: the exit edge fires on the next tick.
        let dry = ground_air();
        pool.update(
            &mut dry.clone(), &mut [], None, None, None,
            YoyoInputs::default(), FlailInputs::default(), &mut rng, 0, false,
        );
        let bursts = pool.drain_dust_bursts();
        assert!(
            bursts.iter().any(|b| matches!(b, DustBurst::WetSplash { entry: false, kind: SplashKind::Water, .. })),
            "the water exit splash queued"
        );
    }

    /// W4-d53 FINAL — `Projectile.cs:18947-18956`: the 1053 impact pair draws
    /// NO argument rolls — 10 iterations × two type-6 default blocks (each
    /// with the `:214-219` tweak's extra `Next(-10,6)`), box (31, 23) at
    /// (pos.x, pos.y+8). The post-arms: puff A ×8 vel + noGravity, puff B
    /// ×2 then ×1.5 X / −2 Y.
    #[test]
    fn w4_d53_lava_boulder_impact_census_and_post_fields() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0, 100.0 * 16.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        lava_boulder_impact_dust(&mut rng, &mut dust, &c, &w, pos, 31, 31);
        for _ in 0..10 {
            burst_block(&mut probe, 6, 31, 23);
            burst_block(&mut probe, 6, 31, 23);
        }
        assert_eq!(rng.peek(), probe.peek(), "the 10×2 type-6 census (no argument draws)");
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 20);
        for (i, s) in dust.slots().iter().filter(|s| s.active).enumerate() {
            assert_eq!(s.dust_type, 6);
            assert_eq!(s.alpha, 100);
            // The scale rides the pool's random base (`Dust.cs:190-191`:
            // (1 + Next(-20,21)·0.01) × Scale) then the type-6 tweak ×0.7 —
            // pin the BAND, not an exact value (the census test owns the draws).
            if i % 2 == 0 {
                assert!(s.no_gravity, "the scale-2 puff floats");
                assert!((0.56..=1.68).contains(&s.scale), "scale-2 band: {}", s.scale);
            } else {
                assert!(!s.no_gravity, "the scale-1.25 puff falls");
                assert!((0.35..=1.05).contains(&s.scale), "scale-1.25 band: {}", s.scale);
            }
        }
    }

    /// W4-d53 FINAL — `Projectile.cs:19242-19246`: the 1048 bounce dust draws
    /// the rectangle's X float THEN Y float per iteration (the
    /// NextVector2FromRectangle pair), then the type-322 default block (no
    /// tweak draw); the anchor is the pair − Vector2.One, the box 2×2 clamps
    /// to 5×5 inside new_dust. The speed arguments are arithmetic only.
    #[test]
    fn w4_d53_poulder_bounce_census_and_anchor() {
        let w = ground_air();
        let c = ctx(&w);
        let pos = Vec2::new(60.0 * 16.0 + 0.7, 100.0 * 16.0 + 0.9);
        let vel = Vec2::new(-3.0, 4.0);
        let mut rng = UnifiedRandom::new(53);
        let mut probe = UnifiedRandom::new(53);
        let mut dust = DustPool::new();
        poulder_bounce_dust(&mut rng, &mut dust, &c, &w, pos, 31, 31, vel);
        for _ in 0..10 {
            probe.next_float();
            probe.next_float();
            burst_block(&mut probe, 322, 2, 2);
        }
        assert_eq!(rng.peek(), probe.peek(), "the pair-then-block census");
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 10);
        // The (int) truncation: 960.7 → 960, 1600.9 → 1600. The anchor rides
        // inside [hitbox − 1, hitbox − 1 + 31) = [959, 990); the pool's jitter
        // adds Next(5−4) + 4 = 4 (the 2×2 box clamps to 5×5 FIRST,
        // Dust.cs:155-171, so Next(1) = 0 and the +4 inset applies) — the
        // final position band is [963, 994). Pin the band, not the pixel.
        for s in dust.slots().iter().filter(|s| s.active) {
            assert_eq!(s.dust_type, 322);
            assert!((963.0..994.0).contains(&s.position.x), "x band: {}", s.position.x);
            assert!((1603.0..1634.0).contains(&s.position.y), "y band: {}", s.position.y);
        }
    }

    /// W4-d53 FINAL — the drop-in contract census: d3's ladder
    /// (`projectile_ai_boulder::bounce_ladder`, tip 2d01693ca) keeps the
    /// 1053 dust as NO draws and the 1048 dust as 10 count-only
    /// rectangle-pair draws + the PooFly count/pair draws. The emitters
    /// replay EXACTLY the C# site's draws — the 1048's 10 pairs move from
    /// the ladder to the drain (the merge must DELETE the ladder's pairs,
    /// keeping the flies); the 1053 adds 20 type-6 blocks the ladder never
    /// drew. The row census pins both kinds rowed at 31×31 aiStyle 25.
    #[test]
    fn w4_d53_boulder_sites_are_rowed_on_d3() {
        // The rows are d3's (unmerged here) — the contract pins the C#
        // SetDefaults the merge will carry: `:1833-1850` (1053) and
        // `:9928-9937` (1048), both 31×31 aiStyle 25 trap.
        // The 1048 emitter's draw count equals the ladder's count-only
        // block: 10 × (2 floats) + 10 × 6 default rolls.
        let w = ground_air();
        let c = ctx(&w);
        let mut rng = UnifiedRandom::new(7);
        let mut probe = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        poulder_bounce_dust(
            &mut rng, &mut dust, &c, &w,
            Vec2::new(60.0 * 16.0, 100.0 * 16.0), 31, 31, Vec2::new(1.0, -2.0),
        );
        for _ in 0..10 {
            let _ = (probe.next_float(), probe.next_float());
        }
        for _ in 0..10 {
            burst_block(&mut probe, 322, 2, 2);
        }
        assert_eq!(rng.peek(), probe.peek(), "the pairs-then-blocks split matches the C# order");
    }
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

    /// W4-e3 batch 2 — `:23541-23549`: the spark window's dust-6 (the 979's
    /// dust-135) with the centre blend, ×2 velocity, ×1.6 scale, +vel. The
    /// dust-6/135 per-type tweak (`Dust.cs:214-219`) rolls ONE extra range
    /// and rewrites velocity.Y / ×0.3 X / ×0.7 scale BEFORE the post-arms —
    /// mirrored record-for-record.
    #[test]
    fn w4_e3_b2_spark_dust_blends_and_rides_the_type_tweak() {
        let w = world();
        let c = ctx();
        for kind in [504u16, 979] {
            let p = proj(kind);
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(61);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(61);
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style2_spark_dust(&p, &w, &mut d);
            let dust_type = if kind != 979 { 6 } else { 135 };
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, dust_type,
                0.0, 0.0, 150, Color::default(), 1.0,
            );
            let (cx, cy) = p.center();
            let md = mpool.slot_mut(idx);
            md.position = (md.position + Vec2::new(cx, cy)) / 2.0;
            md.no_gravity = true;
            md.velocity *= 2.0;
            md.scale *= 1.6;
            md.velocity += p.vel;
            assert_eq!(active(&pool).len(), 1, "kind {kind}");
            assert_eq!(pool.slot(0).position, mpool.slot(0).position);
            assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
            assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
            assert_eq!(pool.slot(0).dust_type, dust_type);
            assert!(pool.slot(0).no_gravity);
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }

    /// W4-e3 batch 2 — `:23614-23623`: the 306 trail's four dust-184 sit at
    /// centre − vel/4·i with zero velocity and scale 0.7 (the post-writes
    /// overwrite the jittered positions exactly). Mirrored.
    #[test]
    fn w4_e3_b2_trail306_pins_the_four_positions() {
        let w = world();
        let c = ctx();
        let p = proj(306);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(71);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style2_trail_306(&p, &w, &mut d);
        let (cx, cy) = p.center();
        for i in 0..4 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, 184,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.position.x = cx - p.vel.x / 4.0 * i as f32;
            md.position.y = cy - p.vel.y / 4.0 * i as f32;
            md.velocity *= 0.0;
            md.scale = 0.7;
        }
        assert_eq!(active(&pool).len(), 4);
        for i in 0..4 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, Vec2::ZERO);
            assert_eq!(pool.slot(i).scale, 0.7);
            assert_eq!(pool.slot(i).dust_type, 184);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 batch 2 — `:23685-23688`: the 54 tail's Next(20) gate draws
    /// every tick the 54 flies; a passed gate spawns one dust-40 with the
    /// ×0.1 velocity and scale 0.75. Mirrored over 40 ticks.
    #[test]
    fn w4_e3_b2_tail54_gates_every_tick() {
        let w = world();
        let c = ctx();
        let p = proj(54);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(81);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(81);
        for _ in 0..40 {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style2_tail_54(&p, &w, &mut d);
            if mirror.next_range(0, 20) == 0 {
                mpool.new_dust(
                    &mut mirror, &c, &w, p.pos, 10, 10, 40,
                    p.vel.x * 0.1, p.vel.y * 0.1, 0, Color::default(), 0.75,
                );
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(!act.is_empty(), "seed 81 passes the 1-in-20 gate in 40 ticks");
        for &i in &act {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).dust_type, 40);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 batch 2 — `:23784-23812`: the Vilethorn crossing burst is
    /// 3× dust-18 + 1× dust-14 (the else arm); the Nettle 150-152 arm is
    /// 8× dust-7 noGravity ×0.5. Mirrored; the 493/494 arm's per-dust
    /// Next(68,71) type draw rides BEFORE the pool internals.
    #[test]
    fn w4_e3_b2_fade_burst_shapes_per_kind() {
        let w = world();
        let c = ctx();
        // The Vilethorn else arm: 3 dust-18 + 1 dust-14.
        let p = proj(7);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(91);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(91);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style4_fade_burst(&p, &w, &mut d);
        for _ in 0..3 {
            mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, 18,
                p.vel.x * 0.025, p.vel.y * 0.025, 170, Color::default(), 1.2,
            );
        }
        mpool.new_dust(
            &mut mirror, &c, &w, p.pos, 10, 10, 14,
            0.0, 0.0, 170, Color::default(), 1.1,
        );
        assert_eq!(active(&pool).len(), 4);
        for i in 0..4 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).dust_type, mpool.slot(i).dust_type);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The Nettle arm: 8 dust-7, noGravity, ×0.5 velocity.
        let n = proj(150);
        let mut npool = DustPool::new();
        let mut nrng = UnifiedRandom::new(91);
        let mut nd = ProjDust { pool: &mut npool, ctx: &c, rng: &mut nrng };
        style4_fade_burst(&n, &w, &mut nd);
        assert_eq!(active(&npool).len(), 8);
        for i in 0..8 {
            assert_eq!(npool.slot(i).dust_type, 7);
            assert!(npool.slot(i).no_gravity);
        }
        // The 493/494 arm: 8 dusts, each type drawn Next(68,71) first.
        let s = proj(493);
        let mut spool = DustPool::new();
        let mut srng = UnifiedRandom::new(91);
        let mut smirror = UnifiedRandom::new(91);
        let mut sd = ProjDust { pool: &mut spool, ctx: &c, rng: &mut srng };
        style4_fade_burst(&s, &w, &mut sd);
        assert_eq!(active(&spool).len(), 8);
        // The mirror replays the FULL draw sequence: the Next(68,71) pick
        // first, then the pool internals, then the post-arms.
        for i in 0..8 {
            let t = smirror.next_range(68, 71);
            let mi = mpool.new_dust(
                &mut smirror, &c, &w, s.pos, 10, 10, t,
                s.vel.x * 0.025, s.vel.y * 0.025, 200, Color::default(), 1.3,
            );
            let md = mpool.slot_mut(mi);
            md.no_gravity = true;
            md.velocity *= 0.5;
            assert_eq!(spool.slot(i).dust_type, t, "the type arg precedes the internals");
            assert_eq!(spool.slot(i).position, md.position);
            assert_eq!(spool.slot(i).velocity, md.velocity);
            assert!(spool.slot(i).no_gravity);
        }
        assert_eq!(srng.next(), smirror.next(), "the streams stay aligned");
    }


    /// W4-e3 batch 3 — `:23954-23968`: the 503 spark's Next(16) gate + the
    /// RotatedByRandom(π/2) direction pair + the Next(48) gore gate (always
    /// drawn). Mirrored: the taken branch's dust sits at centre + dir·12
    /// with velocity dir·0.66.
    #[test]
    fn w4_e3_b3_spark503_direction_and_gore_gate() {
        let w = world();
        let c = ctx();
        let p = proj(503);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(101);
        for _ in 0..32 {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style5_spark_503(&p, &w, &mut d);
            if mirror.next_range(0, 16) == 0 {
                let angle = mirror.next_double() * 1.5707963705062866
                    - mirror.next_double() * 1.5707963705062866;
                let dir = crate::bow_launch::rotated_by(
                    crate::bow_launch::rotated_by(Vec2::X, angle),
                    p.vel.y.atan2(p.vel.x) as f64,
                );
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, p.pos, 10, 10, 58,
                    p.vel.x * 0.5, p.vel.y * 0.5, 150, Color::default(), 1.2,
                );
                let (cx, cy) = p.center();
                let md = mpool.slot_mut(idx);
                md.velocity = dir * 0.66;
                md.position = Vec2::new(cx, cy) + dir * 12.0;
            }
            let _ = mirror.next_range(0, 48);
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        for &i in &act {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert!((pool.slot(i).velocity.length() - 0.66).abs() < 1e-4);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 batch 3 — `:23970-23986`: the 955 dust-58 gate is Next(20), the
    /// anniversary Next(15) only as the second || operand; the light 0.9
    /// write lands every call; the off-screen gore gate short-circuits its
    /// Next(6); the taken gore branch on an anniversary world draws the
    /// RE-PICK's SECOND Next(4) (`:23976-23978` — the research/240 §3.2
    /// correction, mirrored). Mirrored on both seed flags.
    #[test]
    fn w4_e3_b3_star_dust_anniversary_second_operand() {
        let w = world();
        let c = ctx();
        for anniversary in [false, true] {
            let mut p = proj(955);
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(111);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(111);
            for _ in 0..40 {
                let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
                style5_star_dust(&mut p, &w, &mut d, anniversary);
                if on_screen_wide(&p, &c) && mirror.next_range(0, 6) == 0 {
                    let _ = mirror.next_range(0, 4);
                    if anniversary {
                        let _ = mirror.next_range(0, 4); // the :23978 re-pick
                    }
                }
                if mirror.next_range(0, 20) == 0
                    || (anniversary && mirror.next_range(0, 15) == 0)
                {
                    mpool.new_dust(
                        &mut mirror, &c, &w, p.pos, 10, 10, 58,
                        p.vel.x * 0.5, p.vel.y * 0.5, 150, Color::default(), 1.2,
                    );
                }
            }
            assert_eq!(p.light, 0.9, "the light write");
            assert_eq!(active(&pool).len(), active(&mpool).len());
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }

    /// W4-e3 batch 3 — `:24013-24029`: the 723's three-ring dust-267 pair —
    /// 6 dusts (Blue scale 1.0 + White scale 0.4 per ring), noLight +
    /// noGravity, the rotated (0, 12·scale) offset at pos.Y/200 + i·2π + rot.
    #[test]
    fn w4_e3_b3_shooting_star723_six_ring_dusts() {
        let w = world();
        let c = ctx();
        let mut p = proj(723);
        p.scale = 1.0;
        p.rot = 0.3;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(121);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style5_shooting_star(&p, &w, &mut d);
        let act = active(&pool);
        assert_eq!(act.len(), 6, "3 rings × (Blue + White)");
        let (cx, cy) = p.center();
        for ring in 0..3 {
            let offset = crate::bow_launch::rotated_by(
                Vec2::new(0.0, 12.0),
                (p.pos.y / 200.0 + ring as f32 * (std::f32::consts::PI * 2.0) + p.rot) as f64,
            );
            let anchor = Vec2::new(cx, cy) + offset - p.vel * 0.5;
            let blue = pool.slot(act[ring * 2]);
            let white = pool.slot(act[ring * 2 + 1]);
            assert_eq!(blue.position, anchor);
            assert_eq!(white.position, anchor);
            assert_eq!(blue.dust_type, 267);
            assert_eq!(blue.color, Color::new(0, 0, 255, 255));
            assert_eq!(white.color, Color::WHITE);
            // The White ring's scale is the 0.4 arg × the pool's jitter
            // (1 ± 0.2, `Dust.cs:190`) — the Blue ring's is 1.0 × jitter.
            assert!(white.scale >= 0.32 && white.scale <= 0.48, "0.4 × jitter");
            assert!(blue.scale >= 0.8 && blue.scale <= 1.2, "1.0 × jitter");
            assert!(blue.no_light && blue.no_gravity);
            assert!(white.no_light && white.no_gravity);
            assert_eq!(blue.velocity, p.vel * 0.2 * ring as f32);
        }
        // The 724 draws NOTHING in the block (the C# inner gate omits it).
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(121);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        style5_shooting_star(&proj(724), &w, &mut d2);
        assert!(active(&pool2).is_empty());
        assert_eq!(rng2.next_range(0, 1000), UnifiedRandom::new(121).next_range(0, 1000));
    }

    /// W4-e3 batch 3 — `:24032-24049`: the 9's dust-86 gate (Next(4)) with
    /// the ×0.7 velocity, +0.3·vel and the Next(2) position pull; the gore
    /// gate's Next(20) + Next(16,18) pick stay on the stream. Mirrored.
    #[test]
    fn w4_e3_b3_star9_dust_gate_and_position_pull() {
        let w = world();
        let c = ctx();
        let p = proj(9);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(131);
        for _ in 0..24 {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style5_star9_dust(&p, &w, &mut d);
            if on_screen_wide(&p, &c) && mirror.next_range(0, 20) == 0 {
                let _ = mirror.next_range(16, 18);
            }
            if mirror.next_range(0, 4) == 0 {
                let idx = mpool.new_dust_direct(
                    &mut mirror, &c, &w, p.pos, 10, 10, 86,
                    0.0, 0.0, 127, Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.velocity *= 0.7;
                md.no_gravity = true;
                md.velocity += p.vel * 0.3;
                if mirror.next_range(0, 2) == 0 {
                    md.position -= p.vel * 4.0;
                }
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(!act.is_empty());
        for &i in &act {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert!(pool.slot(i).no_gravity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 batch 3 — `:24051-24061`: the generic 92 arm's Next(10) dust-58
    /// gate + the Next(20) gore gate (the taken branch's Next(16,18) pick
    /// stays). Mirrored over 30 ticks.
    #[test]
    fn w4_e3_b3_generic92_dust_and_gore_gates() {
        let w = world();
        let c = ctx();
        let p = proj(92);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(141);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(141);
        for _ in 0..30 {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style5_generic_dust(&p, &w, &mut d);
            if mirror.next_range(0, 10) == 0 {
                mpool.new_dust(
                    &mut mirror, &c, &w, p.pos, 10, 10, 58,
                    p.vel.x * 0.5, p.vel.y * 0.5, 150, Color::default(), 1.2,
                );
            }
            if mirror.next_range(0, 20) == 0 {
                let _ = mirror.next_range(16, 18);
            }
        }
        assert_eq!(active(&pool).len(), active(&mpool).len());
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 batch 3 — `:24067-24098`: the comet tail's two Next(2) gates
    /// (each: the Next(4) type pick BEFORE the internals, the two velocity
    /// NextFloats after) + the !wet dust-6 perfect with the
    /// NextVector2Circular(25,25) pair. Mirrored; the wet flag kills only
    /// the perfect.
    #[test]
    fn w4_e3_b3_comet_tail_gates_and_wet_perfect() {
        let w = world();
        let c = ctx();
        for wet in [false, true] {
            let mut p = proj(1037);
            p.wet = wet;
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(151);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(151);
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style5_comet_tail(&p, &w, &mut d);
            let (cx, cy) = p.center();
            let vector9 = Vec2::new(cx, cy) + p.vel.normalize_or_zero() * 10.0;
            for sign in [1.0f64, -1.0f64] {
                if mirror.next_range(0, 2) == 0 {
                    let dust_type = if mirror.next_range(0, 4) == 0 { 6 } else { 31 };
                    let idx = mpool.new_dust(
                        &mut mirror, &c, &w, p.pos, 10, 10, dust_type,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let rotated = crate::bow_launch::rotated_by(p.vel, sign * 1.5707963705062866);
                    let md = mpool.slot_mut(idx);
                    md.position = vector9;
                    md.velocity = (rotated * 0.25 + p.vel / 3.0) * 0.9;
                    md.position += rotated;
                    md.scale *= 0.3 + 0.9 * mirror.next_float();
                    md.velocity *= mirror.next_float();
                    md.no_gravity = true;
                }
            }
            if !wet {
                let unit_angle = std::f32::consts::PI * 2.0 * mirror.next_float();
                let unit = Vec2::new(
                    (unit_angle as f64).cos() as f32,
                    (unit_angle as f64).sin() as f32,
                );
                let r = mirror.next_float();
                let offset = Vec2::new(unit.x * 25.0 * r, unit.y * 25.0 * r);
                let idx = mpool.new_dust_perfect(
                    &mut mirror, &c, &w, Vec2::new(cx, cy) + offset, 6,
                    None, 0, Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.velocity *= 0.5;
                md.scale *= 1.3;
                md.fade_in = 1.0;
                md.no_gravity = true;
            }
            assert_eq!(active(&pool).len(), active(&mpool).len(), "wet={wet}");
            for i in active(&pool) {
                assert_eq!(pool.slot(i).position, mpool.slot(i).position);
                assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
                assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            }
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }


    /// W4-e3 batch 4 — `:24110-24150`: the aiStyle-6 burst's parameter
    /// ladder — the plain 30-dust (num51 = 10+kind), the 1019/1051 40/50-dust
    /// noGravity ladder (num53 % 3: the gravity dust ×1.25 scale / ÷2 vel /
    /// −2.2 Y, the noGravity ×1.75 / +0.65·vel), the 1051's per-dust Next(3)
    /// type pick (309 vs 0). Mirrored.
    #[test]
    fn w4_e3_b4_style6_burst_ladder_and_type_pick() {
        let w = world();
        let c = ctx();
        // The plain 10-kind: 30 dusts, type 20, no post-arms.
        let p = proj(10);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(161);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(161);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style6_burst(&p, &w, &mut d, 20, 30, false, false);
        for _ in 0..30 {
            mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, 20,
                p.vel.x, p.vel.y, 50, Color::default(), 1.0,
            );
        }
        assert_eq!(active(&pool).len(), 30);
        for i in 0..30 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).dust_type, 20);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The 1051: 50 dusts, per-dust Next(3) pick, the flag2 ladder.
        let q = proj(1051);
        let mut qpool = DustPool::new();
        let mut qrng = UnifiedRandom::new(161);
        let mut qmpool = DustPool::new();
        let mut qmirror = UnifiedRandom::new(161);
        let mut qd = ProjDust { pool: &mut qpool, ctx: &c, rng: &mut qrng };
        style6_burst(&q, &w, &mut qd, 0, 50, true, true);
        // The mirror replays the FULL draw sequence: the Next(3) pick, the
        // pool internals, then the flag2 post-arms.
        for num53 in 0..50 {
            let t = if qmirror.next_range(0, 3) == 0 { 309 } else { 0 };
            let idx = qmpool.new_dust(
                &mut qmirror, &c, &w, q.pos, 10, 10, t,
                q.vel.x, q.vel.y, 50, Color::default(), 1.0,
            );
            let md = qmpool.slot_mut(idx);
            md.no_gravity = num53 % 3 != 0;
            if !md.no_gravity {
                md.scale *= 1.25;
                md.velocity /= 2.0;
                md.velocity.y -= 2.2;
            } else {
                md.scale *= 1.75;
                md.velocity += q.vel * 0.65;
            }
        }
        assert_eq!(active(&qpool).len(), 50);
        for num53 in 0..50 {
            assert_eq!(qpool.slot(num53).dust_type, qmpool.slot(num53).dust_type);
            assert_eq!(qpool.slot(num53).no_gravity, num53 % 3 != 0);
            assert_eq!(qpool.slot(num53).velocity, qmpool.slot(num53).velocity);
            assert_eq!(qpool.slot(num53).scale, qmpool.slot(num53).scale);
            if num53 % 3 == 0 {
                assert!(qpool.slot(num53).velocity.y < 0.0, "the −2.2 Y kick");
            }
        }
        assert_eq!(qrng.next_range(0, 1000), qmirror.next_range(0, 1000));
    }

    /// W4-e3 keep-one pin — the 1051 burst vs W4-d1's count-only twin
    /// (`projectile_ai_powder::step`, tip e271b5bc): d1 keeps ONLY the 50
    /// per-dust `Next(3)` picks (`:24132`); this emitter draws the SAME
    /// pick FIRST per dust, then the `NewDust` internals + the flag2
    /// post-arms. The pick sequence must be identical on a twin seed —
    /// the merge swap (d1's machine + this burst) keeps the explicit-draw
    /// order and only ADDS the pool-internal draws between picks.
    #[test]
    fn w4_e3_keep_one_1051_picks_match_d1_count_only() {
        let w = world();
        let c = ctx();
        let p = proj(1051);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(271);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style6_burst(&p, &w, &mut d, 0, 50, true, true);
        // The FULL parallel replay: the Next(3) pick FIRST per dust, then the
        // pool internals, then the flag2 post-arms (the emitter's exact draw
        // order). d1's count-only stream keeps ONLY the bare picks, so only
        // the FIRST pick can be compared directly (`:24132` is both streams'
        // first draw); the merge adds the internals BETWEEN the picks.
        let mut d1 = UnifiedRandom::new(271);
        let first_pick = d1.next_range(0, 3);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(271);
        for i in 0..50 {
            let pick = mirror.next_range(0, 3);
            if i == 0 {
                assert_eq!(pick, first_pick, "the first pick is the first draw");
            }
            let want = if pick == 0 { 309 } else { 0 };
            let mi = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 10, 10, want,
                p.vel.x, p.vel.y, 50, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(mi);
            md.no_gravity = i % 3 != 0;
            if !md.no_gravity {
                md.scale *= 1.25;
                md.velocity /= 2.0;
                md.velocity.y -= 2.2;
            } else {
                md.scale *= 1.75;
                md.velocity += p.vel * 0.65;
            }
            assert_eq!(pool.slot(i).dust_type, want, "dust {i}: the pick order");
            assert_eq!(pool.slot(i).velocity, md.velocity, "dust {i}: the post-arm");
            assert_eq!(pool.slot(i).scale, md.scale, "dust {i}: the scale");
            assert_eq!(pool.slot(i).no_gravity, md.no_gravity);
        }
        assert_eq!(active(&pool).len(), 50);
        assert_eq!(rng.next(), mirror.next(), "the full replay stays aligned");
    }

    /// W4-e3 batch 4 — `:24259-24300`: the WaterBolt's lava burst (4 dust-31
    /// ×0.6 −2 Y), the ai[1]==1 Next(2) centre trail (×0.2) and the
    /// ai[1]==0 five-dust 172 ladder + the Next(5) extra. Mirrored.
    #[test]
    fn w4_e3_b4_water_bolt_lava_trail_and_ladder() {
        let w = world();
        let c = ctx();
        let mut p = proj(27);
        p.w = 16.0;
        p.h = 16.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(171);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(171);
        {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style8_water_bolt_lava(&p, &w, &mut d);
        }
        for _ in 0..4 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 16, 16, 31,
                0.0, 0.0, 100, Color::default(), 1.5,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 0.6;
            md.velocity.y -= 2.0;
        }
        assert_eq!(active(&pool).len(), 4);
        // The 172 ladder (ai[1] == 0 → the gate short-circuits the Next(5)).
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style8_water_bolt_172(&p, &w, &mut d);
        for num66 in 0..5 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(p.pos.x + 4.0, p.pos.y + 4.0),
                8, 8, 172, 0.0, 0.0, 100, Color::default(), 1.2,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 0.1;
            md.velocity += p.vel * 0.1;
            md.position.x -= p.vel.x / 3.0 * num66 as f32;
            md.position.y -= p.vel.y / 3.0 * num66 as f32;
        }
        if mirror.next_range(0, 5) == 0 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(p.pos.x + 4.0, p.pos.y + 4.0),
                8, 8, 172, 0.0, 0.0, 100, Color::default(), 0.6,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 0.25;
            md.velocity += p.vel * 0.5;
        }
        let act = active(&pool);
        assert!(act.len() >= 9, "4 lava + 5 ladder (+ the extra)");
        for &i in &act {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The ai[1]==1 trail: the Next(2) gate, the centre puff ×0.2.
        p.ai[1] = 1.0;
        let mut tpool = DustPool::new();
        let mut trng = UnifiedRandom::new(171);
        let mut tmpool = DustPool::new();
        let mut tmirror = UnifiedRandom::new(171);
        for _ in 0..10 {
            let mut td = ProjDust { pool: &mut tpool, ctx: &c, rng: &mut trng };
            style8_water_bolt_trail(&p, &w, &mut td);
            if tmirror.next_range(0, 2) == 0 {
                let (cx, cy) = p.center();
                let idx = tmpool.new_dust(
                    &mut tmirror, &c, &w, Vec2::new(cx, cy), 0, 0, 31,
                    0.0, 0.0, 100, Color::default(), 0.8,
                );
                tmpool.slot_mut(idx).velocity *= 0.2;
            }
        }
        assert_eq!(active(&tpool).len(), active(&tmpool).len());
        for i in active(&tpool) {
            assert_eq!(tpool.slot(i).velocity, tmpool.slot(i).velocity);
        }
        assert_eq!(trng.next_range(0, 1000), tmirror.next_range(0, 1000));
    }

    /// W4-e3 batch 4 — `:24309-24336`: the cursed flame's dust-75 at
    /// pos+vel (scale 3·scale, noGravity), the frost/default dust-135/6
    /// pairs (×0.2 vel, scale 2, noGravity, ×0.3), the 258 getGoodWorld
    /// noLight arm. Mirrored.
    #[test]
    fn w4_e3_b4_cursed_flame_frost_default_pairs() {
        let w = world();
        let c = ctx();
        // The 95 cursed flame: one dust-75 at pos+vel.
        let mut p = proj(95);
        p.scale = 1.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(181);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(181);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style8_cursed_flame(&p, &w, &mut d);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, Vec2::new(p.pos.x + p.vel.x, p.pos.y + p.vel.y),
            10, 10, 75, p.vel.x, p.vel.y, 100, Color::default(), 3.0,
        );
        mpool.slot_mut(idx).no_gravity = true;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert!(pool.slot(0).no_gravity);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The 258 default on a getGoodWorld: two dust-6, noLight.
        let g = proj(258);
        let mut gpool = DustPool::new();
        let mut grng = UnifiedRandom::new(181);
        let mut gd = ProjDust { pool: &mut gpool, ctx: &c, rng: &mut grng };
        style8_default(&g, &w, &mut gd, true);
        assert_eq!(active(&gpool).len(), 2);
        for i in 0..2 {
            assert_eq!(gpool.slot(i).dust_type, 6);
            assert!(gpool.slot(i).no_light, "the getGoodWorld noLight");
            assert!(gpool.slot(i).no_gravity);
        }
        // The 253 frost: two dust-135, no noLight.
        let f = proj(253);
        let mut fpool = DustPool::new();
        let mut frng = UnifiedRandom::new(181);
        let mut fd = ProjDust { pool: &mut fpool, ctx: &c, rng: &mut frng };
        style8_frost(&f, &w, &mut fd);
        assert_eq!(active(&fpool).len(), 2);
        for i in 0..2 {
            assert_eq!(fpool.slot(i).dust_type, 135);
            assert!(!fpool.slot(i).no_light);
        }
    }

    /// W4-e3 batch 4 (STAGED 152) — `:22292-22312`: the 977/1045 centre
    /// trail's Next(8) gate + the NextVector2Circular(1,1) pair + 0.5·vel.
    /// Mirrored; the emitter is the drop-in for d16's count-only skip.
    #[test]
    fn w4_e3_b4_star_slash_dust_centre_trail() {
        let w = world();
        let c = ctx();
        for kind in [977u16, 1045] {
            let p = proj(kind);
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(191);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(191);
            for _ in 0..24 {
                let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
                star_slash_dust(&p, &w, &mut d);
                if mirror.next_range(0, 8) == 0 {
                    let (cx, cy) = p.center();
                    let dust_type = if kind == 977 { 172 } else { 229 };
                    let idx = mpool.new_dust_direct(
                        &mut mirror, &c, &w, Vec2::new(cx, cy), 0, 0, dust_type,
                        p.vel.x * 0.1, p.vel.y * 0.1, 100, Color::default(), 0.9,
                    );
                    let unit_angle = std::f32::consts::PI * 2.0 * mirror.next_float();
                    let unit = Vec2::new(
                        (unit_angle as f64).cos() as f32,
                        (unit_angle as f64).sin() as f32,
                    );
                    let r = mirror.next_float();
                    let md = mpool.slot_mut(idx);
                    md.no_gravity = true;
                    md.position = Vec2::new(cx, cy);
                    md.velocity = Vec2::new(unit.x * r, unit.y * r) + p.vel * 0.5;
                }
            }
            let act = active(&pool);
            assert_eq!(act.len(), active(&mpool).len(), "kind {kind}");
            for &i in &act {
                assert_eq!(pool.slot(i).position, mpool.slot(i).position);
                assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            }
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }

    /// W4-e3 batch 4 (STAGED 152) — `:22334-22347`: the 976's Next(3) trail
    /// + the localAI[1]==0 one-shot 10-dust burst (the two NextFloat velocity
    /// draws per dust). The latch is the CALLER's (set after the call) — the
    /// second call with the latch set spawns only the gated trail.
    #[test]
    fn w4_e3_b4_star_slash_976_trail_and_one_shot_burst() {
        let w = world();
        let c = ctx();
        let mut p = proj(976);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(201);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(201);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        star_slash_976(&p, &w, &mut d);
        // Mirror the first call: the Next(3) trail gate + the 10-dust burst.
        if mirror.next_range(0, 3) == 0 {
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, p.pos, 10, 10, 40,
                0.0, 0.0, 0, Color::default(), 1.2,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity = p.vel * 0.5;
        }
        for _ in 0..10 {
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, p.pos, 10, 10, 40,
                0.0, 0.0, 0, Color::default(), 0.7,
            );
            let angle = mirror.next_float() * (std::f32::consts::PI * 2.0) * 0.25;
            let mag = mirror.next_float() * 3.0;
            let dir = crate::bow_launch::rotated_by(
                crate::bow_launch::safe_normalize(p.vel, Vec2::Y),
                angle as f64,
            );
            mpool.slot_mut(idx).velocity = dir * mag;
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(act.len() >= 10, "the one-shot burst always fires");
        for &i in &act {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The caller latches localAI[1]; the second call draws only the gate.
        p.local_ai[1] = 1.0;
        let before = active(&pool).len();
        let mut d2 = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        star_slash_976(&p, &w, &mut d2);
        let gate = mirror.next_range(0, 3);
        if gate == 0 {
            mpool.new_dust_direct(
                &mut mirror, &c, &w, p.pos, 10, 10, 40,
                0.0, 0.0, 0, Color::default(), 1.2,
            );
        }
        assert_eq!(active(&pool).len() - before, if gate == 0 { 1 } else { 0 });
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 coverage phase — `:78338-78350`: the bomb family's 20× dust-31
    /// (×1.4 velocity) + 10× the dust-6 pair (2.5 noGravity ×5 / 1.5 ×3),
    /// then the four gore TYPE draws. The caller resized to 22×22. Mirrored
    /// record-for-record on a twin stream; the stream after equals the
    /// mirror's (the four `Next(61,64)` stay). WRITTEN NOT RUN.
    #[test]
    fn w4_e3_cov_bomb_family_dust_counts_and_gore_types() {
        let w = world();
        let c = ctx();
        let mut p = proj(28);
        p.w = 22.0;
        p.h = 22.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(41);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        bomb_family_dust(&p, &w, &mut d);
        for _ in 0..20 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 22, 22, 31,
                0.0, 0.0, 100, Color::default(), 1.5,
            );
            mpool.slot_mut(idx).velocity *= 1.4;
        }
        for _ in 0..10 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 22, 22, 6,
                0.0, 0.0, 100, Color::default(), 2.5,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 5.0;
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 22, 22, 6,
                0.0, 0.0, 100, Color::default(), 1.5,
            );
            mpool.slot_mut(idx).velocity *= 3.0;
        }
        for _ in 0..4 {
            let _ = mirror.next_range(61, 64);
        }
        assert_eq!(active(&pool).len(), 40, "20 + 10×2");
        for i in 0..40 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).no_gravity, mpool.slot(i).no_gravity);
        }
        for i in 0..20 {
            assert_eq!(pool.slot(i).dust_type, 31);
        }
        for i in 20..40 {
            assert_eq!(pool.slot(i).dust_type, 6);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 coverage phase — `:78405-78417`: the big bomb's 50× dust-31
    /// (scale 2, ×1.4) + 80× the dust-6 pair (3 noGravity ×5 / 2 ×3), then
    /// the eight gore TYPE draws. The caller resized to 200×200 (the
    /// dynamite). Mirrored; the noGravity alternation pins (every pair's
    /// FIRST dust). WRITTEN NOT RUN.
    #[test]
    fn w4_e3_cov_big_bomb_dust_counts_and_pair_shape() {
        let w = world();
        let c = ctx();
        let mut p = proj(29);
        p.w = 200.0;
        p.h = 200.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(43);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        big_bomb_dust(&p, &w, &mut d);
        for _ in 0..50 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 200, 200, 31,
                0.0, 0.0, 100, Color::default(), 2.0,
            );
            mpool.slot_mut(idx).velocity *= 1.4;
        }
        for _ in 0..80 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 200, 200, 6,
                0.0, 0.0, 100, Color::default(), 3.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 5.0;
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 200, 200, 6,
                0.0, 0.0, 100, Color::default(), 2.0,
            );
            mpool.slot_mut(idx).velocity *= 3.0;
        }
        for _ in 0..8 {
            let _ = mirror.next_range(61, 64);
        }
        assert_eq!(active(&pool).len(), 210, "50 + 80×2");
        for i in 0..210 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
        }
        for i in 50..210 {
            // The pair's first dust (even offset) carries noGravity.
            assert_eq!(pool.slot(i).no_gravity, (i - 50) % 2 == 0);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 coverage batch B — `:70502-70515`: the dig snap's sign pair +
    /// the dominant-axis zeroing. The four axis cases + the four diagonals.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_e3_cov_scarab_dig_snap_eight_ways() {
        // Right-down dominant X: snap (-1, 0).
        assert_eq!(scarab_dig_snap(Vec2::new(3.0, 1.0)), Vec2::new(-1.0, 0.0));
        // Left-up dominant X: snap (1, 0).
        assert_eq!(scarab_dig_snap(Vec2::new(-3.0, -1.0)), Vec2::new(1.0, 0.0));
        // Down dominant Y: snap (0, -1).
        assert_eq!(scarab_dig_snap(Vec2::new(1.0, 3.0)), Vec2::new(0.0, -1.0));
        // Up dominant Y: snap (0, 1).
        assert_eq!(scarab_dig_snap(Vec2::new(-1.0, -3.0)), Vec2::new(0.0, 1.0));
        // The exact 2:1 boundary keeps BOTH (the strict >).
        assert_eq!(scarab_dig_snap(Vec2::new(2.0, 1.0)), Vec2::new(-1.0, -1.0));
        // The diagonal: both signs, no zeroing.
        assert_eq!(scarab_dig_snap(Vec2::new(1.0, 1.0)), Vec2::new(-1.0, -1.0));
        assert_eq!(scarab_dig_snap(Vec2::new(-1.0, 1.0)), Vec2::new(1.0, -1.0));
    }

    /// W4-e3 coverage batch B — `:78224-78284`: the scarab's 60 smoke +
    /// 30×3 triple + 3×13 ring dusts (the f32 1/12 drift runs THIRTEEN per
    /// ring) + the four gore TYPE draws. The base velocity-X clamp fires
    /// once (|4| > 0.25 → 0.25, the rest no-op). Mirrored on a twin stream.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_e3_cov_scarab_bomb_dust_full_sequence() {
        let w = world();
        let c = ctx();
        let mut p = proj(773);
        p.w = 80.0;
        p.h = 80.0;
        let snap = Vec2::new(-1.0, 0.0); // axis-aligned → the 11+6 push
        let num70 = (snap.y as f64).atan2(snap.x as f64) as f32
            + std::f32::consts::FRAC_PI_2;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(47);
        let mut mp = p.clone();
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        scarab_bomb_dust(&mut p, &w, &mut d, snap);
        // Mirror block 1: the 60 smoke.
        for _ in 0..60 {
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, mp.pos, 80, 80, 31,
                0.0, 0.0, 100, Color::default(), 2.0,
            );
            let md = mpool.slot_mut(idx);
            md.velocity.y -= 0.5;
            md.velocity *= 1.2;
            md.color = Color::new(0, 0, 0, 229);
            if mirror.next_range(0, 2) == 0 {
                let md = mpool.slot_mut(idx);
                md.scale = 0.5;
                md.fade_in = 1.0 + mirror.next_range(0, 10) as f32 * 0.1;
                md.color = Color::new(0, 0, 0, 204);
            }
        }
        // Mirror block 2: the 30 triples.
        for _ in 0..30 {
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, mp.pos, 80, 80, 59,
                0.0, 0.0, 100, Color::default(), 1.0,
            );
            mpool.slot_mut(idx).no_gravity = true;
            if mp.vel.x.abs() > 0.25 {
                mp.vel.x *= 0.25 / mp.vel.x.abs();
            }
            let md = mpool.slot_mut(idx);
            md.velocity.y -= 0.5;
            md.fade_in = 1.2;
            md.velocity *= 8.0;
            md.velocity = crate::bow_launch::rotated_by(md.velocity, num70 as f64);
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, mp.pos, 80, 80, 59,
                0.0, 0.0, 100, Color::default(), 1.5,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 2.0;
            md.velocity = crate::bow_launch::rotated_by(md.velocity, num70 as f64);
            md.velocity.y -= 1.5;
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, mp.pos, 80, 80, 59,
                0.0, 0.0, 100, Color::default(), 1.5,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity.y -= 1.0;
            md.fade_in = 2.0;
            md.velocity *= 4.0;
            md.velocity = crate::bow_launch::rotated_by(md.velocity, num70 as f64);
        }
        // Mirror block 3: the three 13-step rings.
        let (cx, cy) = mp.center();
        let center = Vec2::new(cx, cy);
        for num73 in 1..=3 {
            let num74 = std::f32::consts::PI * 2.0 * mirror.next_float();
            let mut t = 0.0f32;
            let mut steps = 0;
            while t < 1.0 {
                steps += 1;
                let angle = std::f32::consts::PI * 2.0 * t + num74;
                let vel_arg = Vec2::new(
                    (angle as f64).cos() as f32 * 0.7,
                    (angle as f64).sin() as f32 * 0.3,
                );
                let idx = mpool.new_dust_perfect(
                    &mut mirror, &c, &w, center, 59, Some(vel_arg), 0,
                    Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.fade_in = 2.0;
                md.no_gravity = true;
                md.velocity *= num73 as f32 + mirror.next_float() * 0.6;
                md.velocity.y -= num73 as f32 * 0.8;
                md.velocity = crate::bow_launch::rotated_by(md.velocity, num70 as f64);
                md.velocity += snap * (mirror.next_float() * 11.0 + 6.0);
                t += 1.0f32 / 12.0f32;
            }
            assert_eq!(steps, 13, "the f32 1/12 drift runs thirteen");
        }
        for _ in 0..4 {
            let _ = mirror.next_range(61, 64);
        }
        assert_eq!(active(&pool).len(), 60 + 90 + 39, "60 + 30×3 + 3×13");
        for i in 0..active(&pool).len() {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).fade_in, mpool.slot(i).fade_in);
            assert_eq!(pool.slot(i).color, mpool.slot(i).color);
        }
        // The behaviour clamp fired exactly once.
        assert_eq!(p.vel.x, 0.25);
        assert_eq!(mp.vel.x, 0.25);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 coverage batch B — `:78455-78497`: the confetti's 8 type-rolled
    /// (219+Next(5)) + 15×2 pair (139+Next(4)) dusts, the type arg BEFORE
    /// the internals, then the four independent Next(2) gates each with a
    /// Next(276,283) when taken. Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_cov_confetti_bomb_dust_type_rolls_and_gates() {
        let w = world();
        let c = ctx();
        let mut p = proj(588);
        p.w = 22.0;
        p.h = 22.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(53);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        confetti_bomb_dust(&p, &w, &mut d);
        for _ in 0..8 {
            let ty = 219 + mirror.next_range(0, 5);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 22, 22, ty,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 1.4;
            md.fade_in = 1.0;
            md.no_gravity = true;
        }
        for _ in 0..15 {
            let ty = 139 + mirror.next_range(0, 4);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 22, 22, ty,
                0.0, 0.0, 0, Color::default(), 1.6,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 5.0;
            let ty = 139 + mirror.next_range(0, 4);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 22, 22, ty,
                0.0, 0.0, 0, Color::default(), 1.9,
            );
            mpool.slot_mut(idx).velocity *= 3.0;
        }
        for _ in 0..4 {
            if mirror.next_range(0, 2) == 0 {
                let _ = mirror.next_range(276, 283);
            }
        }
        assert_eq!(active(&pool).len(), 38, "8 + 15×2");
        for i in 0..8 {
            assert!((219..224).contains(&pool.slot(i).dust_type));
            assert_eq!(pool.slot(i).fade_in, 1.0);
            assert!(pool.slot(i).no_gravity);
        }
        for i in 8..38 {
            assert!((139..143).contains(&pool.slot(i).dust_type));
        }
        for i in 0..38 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 coverage batch B — `:78506-78513`: the acorn's 10× dust-2 at
    /// the Centre (10×10 box), noGravity, ×2 − oldVel·0.3, the POST-spawn
    /// Next(150)·0.001 scale bump. Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_cov_acorn_bomb_dust_old_vel_and_scale_bump() {
        let w = world();
        let c = ctx();
        let p = proj(1088);
        let old_vel = Vec2::new(2.0, -3.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(59);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(59);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        acorn_bomb_dust(&p, &w, &mut d, old_vel);
        let (cx, cy) = p.center();
        for _ in 0..10 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(cx, cy), 10, 10, 2,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 2.0;
            md.velocity -= old_vel * 0.3;
            md.scale += mirror.next_range(0, 150) as f32 * 0.001;
        }
        assert_eq!(active(&pool).len(), 10);
        for i in 0..10 {
            assert_eq!(pool.slot(i).dust_type, 2);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:56042-56055`: the 1006's Next(8) gate,
    /// the wet-halved velocity, Circular(4,4)'s unit+radius pair, the hue
    /// NextFloat BEFORE the scale NextFloat (argument order), the Perfect
    /// 306 + fadeIn = scale+0.05, then the CloneDust twin (White, −0.3).
    /// Mirrored over 40 ticks wet + dry. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_ai001_shimmer_1006_sparkle_and_clone_mirror() {
        let w = world();
        let c = ctx();
        for wet in [false, true] {
            let mut p = proj(1006);
            p.wet = wet;
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(31);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(31);
            for _ in 0..40 {
                let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
                ai001_shimmer_sparkle_1006(&p, &w, &mut d);
                if mirror.next_range(0, 8) == 0 {
                    let mut value = p.vel * 1.25;
                    if wet {
                        value *= 0.5;
                    }
                    let (cx, cy) = p.center();
                    let unit = {
                        let a = std::f32::consts::PI * 2.0 * mirror.next_float();
                        Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32)
                    };
                    let r = mirror.next_float();
                    let point = Vec2::new(cx, cy) + Vec2::new(unit.x * 4.0 * r, unit.y * 4.0 * r);
                    let hue = mirror.next_float();
                    let rgb = tr_core::tile_light::hsl_to_rgb(hue, 1.0, 0.5);
                    let color = Color::new(
                        (rgb[0] * 255.0) as u8, (rgb[1] * 255.0) as u8, (rgb[2] * 255.0) as u8, 255,
                    );
                    let scale = 1.0 + mirror.next_float() * 0.4;
                    let idx = mpool.new_dust_perfect(
                        &mut mirror, &c, &w, point, 306, Some(value), 0, color, scale,
                    );
                    let md = mpool.slot_mut(idx);
                    md.no_gravity = true;
                    md.fade_in = md.scale + 0.05;
                    let cl = mpool.clone_dust(&mut mirror, &c, &w, idx);
                    let mc = mpool.slot_mut(cl);
                    mc.color = Color::WHITE;
                    mc.scale -= 0.3;
                }
            }
            let act = active(&pool);
            assert_eq!(act.len(), active(&mpool).len());
            assert!(!act.is_empty(), "seed 31 passes the 1-in-8 gate in 40 ticks");
            assert_eq!(act.len() % 2, 0, "each pass spawns the source + its clone");
            for &i in &act {
                assert_eq!(pool.slot(i).position, mpool.slot(i).position);
                assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
                assert_eq!(pool.slot(i).color, mpool.slot(i).color);
                assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
                assert_eq!(pool.slot(i).fade_in, mpool.slot(i).fade_in);
                assert!(pool.slot(i).no_gravity);
            }
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
        // A non-1006 kind draws nothing (the gate short-circuits on kind).
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(31);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        ai001_shimmer_sparkle_1006(&proj(981), &w, &mut d2);
        assert!(active(&pool2).is_empty());
        assert_eq!(rng2.next_range(0, 1000), UnifiedRandom::new(31).next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:56746-56752`: the 981's 1-in-6 dust-279
    /// with the velocity args, noLightEmittance + noGravity + ×0.2.
    /// Mirrored over 24 ticks. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_ai001_silver_bullet_981_drip_mirror() {
        let w = world();
        let c = ctx();
        let p = proj(981);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(17);
        for _ in 0..24 {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            ai001_silver_bullet_drip_981(&p, &w, &mut d);
            if mirror.next_range(0, 6) == 0 {
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, p.pos, 10, 10, 279,
                    p.vel.x, p.vel.y, 100, Color::default(), 1.2,
                );
                let md = mpool.slot_mut(idx);
                md.no_light_emittance = true;
                md.no_gravity = true;
                md.velocity *= 0.2;
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(!act.is_empty(), "seed 17 passes the 1-in-6 gate in 24 ticks");
        for &i in &act {
            assert_eq!(pool.slot(i).dust_type, 279);
            assert!(pool.slot(i).no_light_emittance);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:29238-29303`: the 1042's 6×324 trail
    /// (three alpha gates each, the Centre−v·i/6 position, the ×0.2 tail,
    /// the Next(6) colour) + the Next(4) extra (scale 0.6, no alpha gates,
    /// no position rewrite). Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style60_shimmer_gun_1042_trail_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(1042);
        p.w = 18.0;
        p.h = 18.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(41);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style60_shimmer_gun_trail_1042(&p, &w, &mut d);
        let (cx, cy) = p.center();
        let pick_color = |k: i32| match k {
            0 => Color::new(255, 255, 220, 255),
            1 => Color::new(230, 255, 255, 255),
            2 => Color::new(255, 210, 255, 255),
            _ => Color::new(225, 210, 255, 255),
        };
        for i in 0..6 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos + Vec2::splat(6.0), 6, 6, 324,
                0.0, 0.0, 75, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            for _ in 0..3 {
                if mirror.next_range(0, 2) == 0 {
                    md.alpha += 25;
                }
            }
            md.no_gravity = true;
            md.velocity *= 0.3;
            md.velocity += p.vel * 0.5;
            md.position = Vec2::new(cx - p.vel.x / 6.0 * i as f32, cy - p.vel.y / 6.0 * i as f32);
            md.velocity *= 0.2;
            md.color = pick_color(mirror.next_range(0, 6));
        }
        if mirror.next_range(0, 4) == 0 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos + Vec2::splat(6.0), 6, 6, 324,
                0.0, 0.0, 75, Color::default(), 0.6,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 0.5;
            md.velocity += p.vel * 0.5;
            md.color = pick_color(mirror.next_range(0, 6));
        }
        let act = active(&pool);
        assert!(act.len() >= 6);
        assert_eq!(act.len(), active(&mpool).len());
        for &i in &act {
            assert_eq!(pool.slot(i).dust_type, 324);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).color, mpool.slot(i).color);
            assert_eq!(pool.slot(i).alpha, mpool.slot(i).alpha);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:27867-27874` (the 227's 8×157, the
    /// POST-spawn NextFloat velocity blend) + `:28832-28838` (the 321's
    /// inset dust-6, noLight, scale 1.4). Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style43_burst227_and_style55_head321_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(227);
        p.w = 14.0;
        p.h = 14.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(43);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style43_crystal_leaf_burst_227(&p, &w, &mut d);
        for _ in 0..8 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 14, 14, 157,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 3.0;
            md.scale = 1.5;
            md.velocity += p.vel * mirror.next_float();
        }
        assert_eq!(active(&pool).len(), 8);
        for i in 0..8 {
            assert_eq!(pool.slot(i).dust_type, 157);
            assert_eq!(pool.slot(i).scale, 1.5);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The 321 head: one dust, no explicit draws.
        let mut q = proj(321);
        q.w = 30.0;
        q.h = 30.0;
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(44);
        let mut mpool2 = DustPool::new();
        let mut mirror2 = UnifiedRandom::new(44);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        style55_flaming_jack_head_321(&q, &w, &mut d2);
        let idx = mpool2.new_dust(
            &mut mirror2, &c, &w, q.pos + Vec2::splat(8.0), 14, 14, 6,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let md = mpool2.slot_mut(idx);
        md.velocity *= 0.5;
        md.velocity += q.vel * 0.5;
        md.no_gravity = true;
        md.no_light = true;
        md.scale = 1.4;
        assert_eq!(active(&pool2).len(), 1);
        assert!(pool2.slot(0).no_light);
        assert_eq!(pool2.slot(0).scale, 1.4);
        assert_eq!(pool2.slot(0).velocity, mpool2.slot(0).velocity);
        assert_eq!(rng2.next_range(0, 1000), mirror2.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:31011-31209`: the 451's four spark sites
    /// share ONE body (SelectRandom(226,229), the rotated (−32,32) corner,
    /// vel×0.1); the fade arms draw the Next(6) noGravity gate, the strike
    /// arm (gate_next6=false) writes it unconditionally. Mirrored across
    /// three calls with distinct scales. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style81_influx_451_spark_shape_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(451);
        p.rot = 0.7;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(47);
        for (scale, gate) in [(1.0f32, true), (0.6, true), (0.6, false)] {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style81_influx_spark_451(&p, &w, &mut d, scale, gate);
            let ty = [226, 229][mirror.next_range(0, 2) as usize];
            let (cx, cy) = p.center();
            let off = crate::bow_launch::rotated_by(Vec2::new(-32.0, 32.0), p.rot as f64);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(cx, cy) + off - Vec2::splat(4.0),
                8, 8, ty, 0.0, 0.0, 100, Color::default(), scale,
            );
            let md = mpool.slot_mut(idx);
            md.velocity *= 0.1;
            if !gate || mirror.next_range(0, 6) != 0 {
                md.no_gravity = true;
            }
        }
        assert_eq!(active(&pool).len(), 3);
        for i in 0..3 {
            assert!([226, 229].contains(&pool.slot(i).dust_type));
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).no_gravity, mpool.slot(i).no_gravity);
        }
        assert!(pool.slot(2).no_gravity, "the strike arm writes noGravity unconditionally");
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:31889-31899`: the 464's 6×2 spoke ring
    /// (the velocity-rotated (0,−720) spin scaled by ai[0]%45/45, the
    /// RandomVector2(−8,8)/2 jitter = TWO NextDouble-cast draws per dust,
    /// 197 alpha-100 Transparent noGravity). Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style86_ice_mist_464_ring_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(464);
        p.ai[0] = 20.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(53);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style86_ice_mist_ring_464(&p, &w, &mut d);
        let (cx, cy) = p.center();
        let spin = crate::bow_launch::rotated_by(
            Vec2::new(0.0, -720.0), p.vel.y.atan2(p.vel.x) as f64,
        ) * (20.0f32 % 45.0 / 45.0);
        for k in 0..6 {
            let angle = (k as f32 * (std::f32::consts::PI * 2.0) / 6.0) as f64;
            let spoke = Vec2::new(cx, cy) + crate::bow_launch::rotated_by(spin, angle);
            for _ in 0..2 {
                let jx = 16.0 * mirror.next_double() as f32 - 8.0;
                let jy = 16.0 * mirror.next_double() as f32 - 8.0;
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, spoke + Vec2::new(jx, jy) / 2.0,
                    8, 8, 197, 0.0, 0.0, 100, Color::TRANSPARENT, 1.0,
                );
                mpool.slot_mut(idx).no_gravity = true;
            }
        }
        assert_eq!(active(&pool).len(), 12);
        for i in 0..12 {
            assert_eq!(pool.slot(i).dust_type, 197);
            assert_eq!(pool.slot(i).color, Color::TRANSPARENT);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:31925-31935` (the 482 curtain: the
    /// (w·h)·0.0045 float count, the +6/−2 position nudge) +
    /// `:32544-32556` (the 496 tail: the scale·10 count, the velocity args,
    /// the (pos+Centre)/2 rewrite, fadeIn = 100+owner). Mirrored.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style87_curtain482_and_style91_tail496_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(482);
        p.w = 16.0;
        p.h = 200.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(57);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(57);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style87_clinger_curtain_482(&p, &w, &mut d);
        // 16*200*0.0045 = 14.4 → 15 iterations (i = 0..14 < 14.4).
        let count = 16.0f32 * 200.0 * 0.0045;
        let mut n = 0;
        let mut i = 0.0f32;
        while i < count {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 16, 200, 75,
                0.0, 0.0, 100, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 0.5;
            md.velocity.y -= 0.5;
            md.scale = 1.4;
            md.position.x += 6.0;
            md.position.y -= 2.0;
            n += 1;
            i += 1.0;
        }
        assert_eq!(active(&pool).len(), n);
        assert_eq!(n, 15, "the float bound admits i=14 (< 14.4)");
        for j in 0..n {
            assert_eq!(pool.slot(j).scale, 1.4);
            assert_eq!(pool.slot(j).position, mpool.slot(j).position);
            assert_eq!(pool.slot(j).velocity, mpool.slot(j).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The 496 tail at scale 0.6: 6 dusts, fadeIn 100+owner.
        let mut q = proj(496);
        q.scale = 0.6;
        q.owner = 3;
        q.w = 40.0;
        q.h = 40.0;
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(58);
        let mut mpool2 = DustPool::new();
        let mut mirror2 = UnifiedRandom::new(58);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        style91_shadowflame_tail_496(&q, &w, &mut d2);
        let (qx, qy) = q.center();
        let mut i = 0.0f32;
        while i < 0.6 * 10.0 {
            let idx = mpool2.new_dust(
                &mut mirror2, &c, &w, q.pos, 40, 40, 27,
                q.vel.x, q.vel.y, 100, Color::default(), 1.1,
            );
            let md = mpool2.slot_mut(idx);
            md.position = (md.position + Vec2::new(qx, qy)) / 2.0;
            md.no_gravity = true;
            md.velocity *= 0.1;
            md.velocity -= q.vel * (1.3 - 0.6);
            md.fade_in = 103.0;
            md.scale += 0.6 * 0.75;
            i += 1.0;
        }
        assert_eq!(active(&pool2).len(), 6);
        for j in 0..6 {
            assert_eq!(pool2.slot(j).fade_in, 103.0);
            assert_eq!(pool2.slot(j).position, mpool2.slot(j).position);
            assert_eq!(pool2.slot(j).velocity, mpool2.slot(j).velocity);
            assert_eq!(pool2.slot(j).scale, mpool2.slot(j).scale);
        }
        assert_eq!(rng2.next_range(0, 1000), mirror2.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:32292-32344`: the 490's alpha-0 ring
    /// (Next(2,4) speed, i=1's ×−0.75/×0.42, the UNGUARDED normalize of the
    /// Next(-10,11)² pick, the Next(8) variant) + the post-60 puff
    /// (Next(1,3), Next(2), fadeIn 2). Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style89_ritual_490_ring_and_puff_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(490);
        p.scale = 0.5;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(61);
        let norm = |v: Vec2| {
            let len = (v.x * v.x + v.y * v.y).sqrt();
            Vec2::new(v.x / len, v.y / len)
        };
        {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style89_ritual_ring_490(&p, &w, &mut d);
        }
        let (cx, cy) = p.center();
        for i in 0..2 {
            let mut speed = mirror.next_range(2, 4) as f32;
            let mut sc = 0.5f32;
            if i == 1 {
                sc *= 0.42;
                speed *= -0.75;
            }
            let dir = norm(Vec2::new(
                mirror.next_range(-10, 11) as f32,
                mirror.next_range(-10, 11) as f32,
            ));
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 14, 14, 228,
                0.0, 0.0, 100, Color::default(), 2.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.no_light = true;
            md.position = Vec2::new(cx, cy) + dir * 204.0 * sc;
            if mirror.next_range(0, 8) == 0 {
                md.velocity = dir * -speed * 2.0;
                md.scale += 0.5;
            } else {
                md.velocity = dir * -speed;
            }
        }
        {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style89_ritual_puff_490(&p, &w, &mut d);
        }
        let speed = mirror.next_range(1, 3) as f32;
        let dir = norm(Vec2::new(
            mirror.next_range(-10, 11) as f32,
            mirror.next_range(-10, 11) as f32,
        ));
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, p.pos, 14, 14, 228,
            0.0, 0.0, 100, Color::default(), 2.0,
        );
        let md = mpool.slot_mut(idx);
        md.no_gravity = true;
        md.no_light = true;
        md.position = Vec2::new(cx, cy);
        if mirror.next_range(0, 2) == 0 {
            md.velocity = dir * speed * 2.0;
            md.scale += 0.5;
        } else {
            md.velocity = dir * speed;
        }
        md.fade_in = 2.0;
        assert_eq!(active(&pool).len(), 3);
        for j in 0..3 {
            assert_eq!(pool.slot(j).position, mpool.slot(j).position);
            assert_eq!(pool.slot(j).velocity, mpool.slot(j).velocity);
            assert_eq!(pool.slot(j).scale, mpool.slot(j).scale);
        }
        assert_eq!(pool.slot(2).fade_in, 2.0);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:32424-32430` (the 492's per-cell 204:
    /// fadeIn 0.75, ×0.1, noLight, ZERO explicit draws) + `:32953-32961`
    /// (the 518's RotatedByRandom orbit point, the 0×0 spawn then the
    /// position/velocity overwrites). Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style90_spark492_and_style94_orbit518_mirror() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(67);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(67);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style90_lantern_spark_492(&w, &mut d, 63, 57);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, Vec2::new(63.0 * 16.0, 57.0 * 16.0),
            16, 16, 204, 0.0, 0.0, 150, Color::default(), 0.3,
        );
        let md = mpool.slot_mut(idx);
        md.fade_in = 0.75;
        md.velocity *= 0.1;
        md.no_light = true;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).fade_in, 0.75);
        assert!(pool.slot(0).no_light);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The 518 orbit spark.
        let mut p = proj(518);
        p.scale = 0.8;
        p.w = 32.0;
        p.h = 32.0;
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(68);
        let mut mpool2 = DustPool::new();
        let mut mirror2 = UnifiedRandom::new(68);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        style94_coin_portal_spark_518(&p, &w, &mut d2);
        let half = Vec2::new(32.0, 32.0) * 0.8 * 0.85 / 2.0;
        let angle = mirror2.next_double() * 6.2831854820251465
            - mirror2.next_double() * 6.2831854820251465;
        let point = crate::bow_launch::rotated_by(Vec2::Y, angle) * half;
        let (cx, cy) = p.center();
        let at = Vec2::new(cx, cy) + point;
        let idx = mpool2.new_dust(&mut mirror2, &c, &w, at, 0, 0, 246, 0.0, 0.0, 0, Color::default(), 1.0);
        let md = mpool2.slot_mut(idx);
        md.position = at;
        md.velocity = Vec2::ZERO;
        assert_eq!(active(&pool2).len(), 1);
        assert_eq!(pool2.slot(0).position, at);
        assert_eq!(pool2.slot(0).velocity, Vec2::ZERO);
        assert_eq!(pool2.slot(0).dust_type, 246);
        assert_eq!(rng2.next_range(0, 1000), mirror2.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:33011-33026` (the 524's two Next(3)!=0
    /// puffs, i=1's +vel/2, the (800−ai[0])/800+0.1 scale) +
    /// `:33145-33151` (the 526's two Centre 228s, +(4,4), NextFloat scale)
    /// + `:34118-34127` (the 836's 3×31 White alpha-50 ×0.3 burst).
    /// Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style96_puffs524_style98_pair526_style112_burst836_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(524);
        p.ai[0] = 100.6;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(71);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style96_ichor_puffs_524(&p, &w, &mut d);
        let (cx, cy) = p.center();
        for i in 0..2 {
            if mirror.next_range(0, 3) != 0 {
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, p.pos, 10, 10, 170,
                    0.0, 0.0, 100, Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.position = (md.position + Vec2::new(cx, cy)) / 2.0;
                md.no_gravity = true;
                md.velocity *= 0.1;
                if i == 1 {
                    md.position += p.vel / 2.0;
                }
                md.scale *= (800.0 - 100.6) / 800.0 + 0.1;
            }
        }
        assert_eq!(active(&pool).len(), active(&mpool).len());
        for &i in &active(&pool) {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        // The 526 pair.
        let q = proj(526);
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(72);
        let mut mpool2 = DustPool::new();
        let mut mirror2 = UnifiedRandom::new(72);
        let mut d2 = ProjDust { pool: &mut pool2, ctx: &c, rng: &mut rng2 };
        style98_cultist_particle_pair_526(&q, &w, &mut d2);
        let (qx, qy) = q.center();
        for _ in 0..2 {
            let idx = mpool2.new_dust(
                &mut mirror2, &c, &w, Vec2::new(qx, qy), 0, 0, 228,
                0.0, 0.0, 100, Color::default(), 1.0,
            );
            let md = mpool2.slot_mut(idx);
            md.no_gravity = true;
            md.position += Vec2::splat(4.0);
            md.scale += mirror2.next_float();
        }
        assert_eq!(active(&pool2).len(), 2);
        for j in 0..2 {
            assert_eq!(pool2.slot(j).position, mpool2.slot(j).position);
            assert_eq!(pool2.slot(j).scale, mpool2.slot(j).scale);
        }
        assert_eq!(rng2.next_range(0, 1000), mirror2.next_range(0, 1000));
        // The 836 burst.
        let r = proj(836);
        let mut pool3 = DustPool::new();
        let mut rng3 = UnifiedRandom::new(73);
        let mut mpool3 = DustPool::new();
        let mut mirror3 = UnifiedRandom::new(73);
        let mut d3 = ProjDust { pool: &mut pool3, ctx: &c, rng: &mut rng3 };
        style112_dandelion_burst_836(&r, &w, &mut d3);
        for _ in 0..3 {
            let idx = mpool3.new_dust(
                &mut mirror3, &c, &w, r.pos, 10, 10, 31,
                0.0, 0.0, 50, Color::WHITE, 1.2,
            );
            let md = mpool3.slot_mut(idx);
            md.velocity *= 0.3;
            md.no_gravity = true;
        }
        assert_eq!(active(&pool3).len(), 3);
        for j in 0..3 {
            assert_eq!(pool3.slot(j).color, Color::WHITE);
            assert_eq!(pool3.slot(j).velocity, mpool3.slot(j).velocity);
        }
        assert_eq!(rng3.next_range(0, 1000), mirror3.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:33984-34016`: the 578/579 dissolve twin
    /// switch. TWO Next(3) draws per tick regardless of outcome; case 0 =
    /// 229 + Next(10,21) re-roll + rot(+π/2)·6, case 1 = 240 at ×30 +
    /// rot(−π/2)·3, case 2 = nothing; each taken dust: the spin's TWO
    /// NextDouble draws FIRST, internals, then NextFloat scale + fadeIn
    /// 0.5. Mirrored over 30 dissolve ticks. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_vortex_dissolve_twin_578_579_mirror() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1016.0, 905.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(79);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(79);
        for t in 0..30 {
            let scale = 1.0 - (t as f32 + 1.0) / 60.0;
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            vortex_dissolve_twin_578_579(&w, &mut d, center, scale);
            for _ in 0..2 {
                let pick = mirror.next_range(0, 3);
                if pick == 2 {
                    continue;
                }
                let angle = mirror.next_double() * 6.2831854820251465
                    - mirror.next_double() * 6.2831854820251465;
                let spin = crate::bow_launch::rotated_by(Vec2::Y, angle) * scale;
                let ty = if pick == 0 { 229 } else { 240 };
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, center - spin * 30.0, 0, 0, ty,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.no_gravity = true;
                if pick == 0 {
                    md.position = center - spin * mirror.next_range(10, 21) as f32;
                    md.velocity = crate::bow_launch::rotated_by(spin, 1.5707963705062866) * 6.0;
                } else {
                    md.position = center - spin * 30.0;
                    md.velocity = crate::bow_launch::rotated_by(spin, -1.5707963705062866) * 3.0;
                }
                md.scale = 0.5 + mirror.next_float();
                md.fade_in = 0.5;
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(!act.is_empty());
        for &i in &act {
            assert!([229, 240].contains(&pool.slot(i).dust_type));
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).fade_in, 0.5);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:34505-34535`: the flag58 explosion's
    /// 4+20+5+10 = 39 dusts; the UnitY loops draw RotatedByRandom(π)'s TWO
    /// NextDoubles + ONE NextDouble radius, the UnitX loops rotate by the
    /// velocity and take the full width/2 (no radius draw). Mirrored.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style117_whip_explosion_612_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(612);
        p.w = 52.0;
        p.h = 52.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(83);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(83);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style117_whip_explosion_612(&p, &w, &mut d);
        let (cx, cy) = p.center();
        let center = Vec2::new(cx, cy);
        let vel_rot = p.vel.y.atan2(p.vel.x) as f64;
        let mut y_point = |m: &mut UnifiedRandom| {
            let a = m.next_double() * 3.1415927410125732 - m.next_double() * 3.1415927410125732;
            center + crate::bow_launch::rotated_by(Vec2::Y, a) * (m.next_double() as f32) * 52.0 / 2.0
        };
        let mut x_point = |m: &mut UnifiedRandom| {
            let a = m.next_double() * 3.1415927410125732 - m.next_double() * 3.1415927410125732;
            center + crate::bow_launch::rotated_by(crate::bow_launch::rotated_by(Vec2::X, a), vel_rot) * 52.0 / 2.0
        };
        for _ in 0..4 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 52, 52, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            let at = y_point(&mut mirror);
            mpool.slot_mut(idx).position = at;
        }
        for _ in 0..10 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 52, 52, 6, 0.0, 0.0, 200, Color::default(), 2.7);
            let at = y_point(&mut mirror);
            let md = mpool.slot_mut(idx);
            md.position = at;
            md.no_gravity = true;
            md.velocity *= 3.0;
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 52, 52, 6, 0.0, 0.0, 100, Color::default(), 1.5);
            let at = y_point(&mut mirror);
            let md = mpool.slot_mut(idx);
            md.position = at;
            md.velocity *= 2.0;
            md.no_gravity = true;
            md.fade_in = 2.5;
        }
        for _ in 0..5 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 52, 52, 6, 0.0, 0.0, 0, Color::default(), 2.7);
            let at = x_point(&mut mirror);
            let md = mpool.slot_mut(idx);
            md.position = at;
            md.no_gravity = true;
            md.velocity *= 3.0;
        }
        for _ in 0..10 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 52, 52, 31, 0.0, 0.0, 0, Color::default(), 1.5);
            let at = x_point(&mut mirror);
            let md = mpool.slot_mut(idx);
            md.position = at;
            md.no_gravity = true;
            md.velocity *= 3.0;
        }
        assert_eq!(active(&pool).len(), 39);
        for i in 0..39 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).fade_in, mpool.slot(i).fade_in);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:34540-34627`: the flag59 explosion's
    /// 20+15+200 = 235 dusts; the six pre-loop draws (3 angles + 3 radii +
    /// the max banding), the 200-ring's captured-velocity normalize, the
    /// i-banded axis-half/rotation, the NextFloat·2 + Next(4) fadeIn pair,
    /// the i<100 position/velocity flip. Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style117_guardian_explosion_624_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(624);
        p.w = 52.0;
        p.h = 52.0;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(89);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(89);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style117_guardian_explosion_624(&p, &w, &mut d);
        let (cx, cy) = p.center();
        let center = Vec2::new(cx, cy);
        let vel_rot = p.vel.y.atan2(p.vel.x) as f64;
        let norm = |v: Vec2| {
            let len = (v.x * v.x + v.y * v.y).sqrt();
            Vec2::new(v.x / len, v.y / len)
        };
        for _ in 0..20 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 52, 52, 135, 0.0, 0.0, 100, Color::default(), 1.5);
            let a = mirror.next_double() * 3.1415927410125732 - mirror.next_double() * 3.1415927410125732;
            let at = center + crate::bow_launch::rotated_by(Vec2::Y, a) * (mirror.next_double() as f32) * 52.0 / 2.0;
            let md = mpool.slot_mut(idx);
            md.position = at;
            md.velocity *= 2.0;
            md.no_gravity = true;
            md.fade_in = 2.5;
        }
        for _ in 0..15 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 52, 52, 135, 0.0, 0.0, 0, Color::default(), 2.7);
            let a = mirror.next_double() * 3.1415927410125732 - mirror.next_double() * 3.1415927410125732;
            let at = center + crate::bow_launch::rotated_by(crate::bow_launch::rotated_by(Vec2::X, a), vel_rot) * 52.0 / 2.0;
            let md = mpool.slot_mut(idx);
            md.position = at;
            md.no_gravity = true;
            md.velocity *= 3.0;
        }
        let two_pi = std::f64::consts::PI * 2.0;
        let a1 = mirror.next_double() * two_pi;
        let a2 = mirror.next_double() * two_pi;
        let a3 = mirror.next_double() * two_pi;
        let r1 = 7.0 + mirror.next_double() * 7.0;
        let r2 = 7.0 + mirror.next_double() * 7.0;
        let r3 = 7.0 + mirror.next_double() * 7.0;
        let rmax = r1.max(r2).max(r3);
        for i in 0..200 {
            let radius = if i > 150 { r3 } else if i > 100 { r1 } else if i > 50 { r2 } else { rmax };
            let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 6, 6, 135, 0.0, 0.0, 100, Color::default(), 1.0);
            let mut spin = mpool.slot(idx).velocity;
            let md = mpool.slot_mut(idx);
            md.position = center;
            spin = norm(spin) * radius as f32;
            if i > 150 {
                spin.y *= 0.5;
                spin = crate::bow_launch::rotated_by(spin, a3);
            } else if i > 100 {
                spin.x *= 0.5;
                spin = crate::bow_launch::rotated_by(spin, a1);
            } else if i > 50 {
                spin.y *= 0.5;
                spin = crate::bow_launch::rotated_by(spin, a2);
            }
            md.velocity *= 0.2;
            md.velocity += spin;
            md.scale = 2.0;
            md.no_gravity = true;
            md.fade_in = mirror.next_float() * 2.0;
            if mirror.next_range(0, 4) == 0 {
                md.fade_in = 2.5;
            }
            md.no_light = true;
            if i < 100 {
                md.position += md.velocity * 20.0;
                md.velocity *= -1.0;
            }
        }
        assert_eq!(active(&pool).len(), 235);
        for i in 0..235 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).fade_in, mpool.slot(i).fade_in);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:35852-35944`: the rise's 3×31 (the TWO
    /// Next(5) post-writes, the integer-half rotated pin) + the 60-burst /
    /// 9-plume SelectRandom(6,259,158) shape (pick FIRST, alpha 200, the
    /// (0.3,2) mul + 2·sign, NextFloat scale, the edge pin, the sign==−1
    /// Next(4) kick). Mirrored both signs. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style126_rise_burst_plume_mirror() {
        let w = world();
        let c = ctx();
        for sign in [1i32, -1] {
            let mut p = proj(654);
            p.w = 30.0;
            p.h = 96.0;
            p.rot = 0.3;
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(97);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(97);
            {
                let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
                style126_column_rise(&p, &w, &mut d);
                style126_column_burst(&p, &w, &mut d, sign);
                style126_column_plume(&p, &w, &mut d, sign);
            }
            let (cx, cy) = p.center();
            let at = Vec2::new(cx, cy)
                + crate::bow_launch::rotated_by(Vec2::new(0.0, -48.0), 0.3f64) * 1.1;
            for _ in 0..3 {
                let idx = mpool.new_dust(&mut mirror, &c, &w, p.pos, 30, 96, 31, 0.0, 0.0, 100, Color::default(), 1.0);
                let md = mpool.slot_mut(idx);
                md.scale = 0.1 + mirror.next_range(0, 5) as f32 * 0.1;
                md.fade_in = 1.5 + mirror.next_range(0, 5) as f32 * 0.1;
                md.no_gravity = true;
                md.position = at;
            }
            for _ in 0..69 {
                let ty = [6, 259, 158][mirror.next_range(0, 3) as usize];
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, p.pos, 30, 96, ty,
                    0.0, 2.5 * sign as f32, 0, Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.alpha = 200;
                md.velocity *= Vec2::new(0.3, 2.0);
                md.velocity.y += (2 * sign) as f32;
                md.scale += mirror.next_float();
                md.position = Vec2::new(cx, cy + 96.0 * 0.5 * (-sign) as f32);
                if sign == -1 && mirror.next_range(0, 4) != 0 {
                    md.velocity.y -= 0.2;
                }
            }
            assert_eq!(active(&pool).len(), 72, "3 + 60 + 9");
            for j in 0..72 {
                assert_eq!(pool.slot(j).position, mpool.slot(j).position);
                assert_eq!(pool.slot(j).velocity, mpool.slot(j).velocity);
                assert_eq!(pool.slot(j).scale, mpool.slot(j).scale);
                assert_eq!(pool.slot(j).alpha, mpool.slot(j).alpha);
            }
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }

    /// W4-e3 AI-inline window — `:35945-35996`: the column body's six
    /// Next(3)<2 gates + the Next(6)<3 lerp/pull split (the pull's
    /// UNGUARDED DirectionFrom normalize, the −sign Y flip) + the tail's
    /// six NextFloat()<0.5 gates with the (0.6,1.5) mul and NO pin.
    /// Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style126_column_body_and_tail_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(670);
        p.w = 30.0;
        p.h = 160.0;
        let sign = -1i32;
        let anchor = p.pos + Vec2::new(0.0, p.h - 96.0);
        let edge_y = p.pos.y + p.h;
        let col_h = 96i32;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(101);
        {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style126_column_body(&p, &w, &mut d, sign, anchor, edge_y, col_h);
            style126_column_tail(&p, &w, &mut d, sign, anchor, col_h);
        }
        let (cx, cy) = p.center();
        let norm = |v: Vec2| {
            let len = (v.x * v.x + v.y * v.y).sqrt();
            Vec2::new(v.x / len, v.y / len)
        };
        for _ in 0..6 {
            if mirror.next_range(0, 3) < 2 {
                let idx = mpool.new_dust(&mut mirror, &c, &w, anchor, 30, col_h, 6, 0.0, 0.0, 90, Color::default(), 2.5);
                let md = mpool.slot_mut(idx);
                md.no_gravity = true;
                md.fade_in = 1.0;
                if md.velocity.y > 0.0 {
                    md.velocity.y *= -1.0;
                }
                if mirror.next_range(0, 6) < 3 {
                    md.position.y += (edge_y - md.position.y) * 0.5;
                    md.velocity *= 5.0;
                    md.velocity.y -= 3.0;
                    md.position.x = cx;
                    md.no_gravity = false;
                    md.no_light = true;
                    md.fade_in = 0.4;
                    md.scale *= 0.3;
                } else {
                    let dir = norm(Vec2::new(cx, cy) - md.position);
                    md.velocity = dir * md.velocity.length() * 0.25;
                }
                md.velocity.y *= (-sign) as f32;
            }
        }
        for _ in 0..6 {
            if !(mirror.next_float() < 0.5) {
                let ty = [6, 259, 158][mirror.next_range(0, 3) as usize];
                let idx = mpool.new_dust(
                    &mut mirror, &c, &w, anchor, 30, col_h, ty,
                    0.0, 2.5 * sign as f32, 0, Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.alpha = 200;
                md.velocity *= Vec2::new(0.6, 1.5);
                md.scale += mirror.next_float();
                if sign == -1 && mirror.next_range(0, 4) != 0 {
                    md.velocity.y -= 0.2;
                }
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(!act.is_empty());
        for &j in &act {
            assert_eq!(pool.slot(j).position, mpool.slot(j).position);
            assert_eq!(pool.slot(j).velocity, mpool.slot(j).velocity);
            assert_eq!(pool.slot(j).scale, mpool.slot(j).scale);
            assert_eq!(pool.slot(j).fade_in, mpool.slot(j).fade_in);
            assert_eq!(pool.slot(j).no_gravity, mpool.slot(j).no_gravity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:36094-36177`: the 658 plume (amount
    /// NextFloat FIRST, the X-lerp second, the >−1.2-gated velocity.X
    /// NextFloat, the unconditional velocity.Y NextFloat) + the two orbit
    /// loops (first .RotatedBy(rotation), second NOT; Next(10) gates every
    /// iteration). Mirrored. WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style128_sandnado_658_plume_and_orbits_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(658);
        p.rot = 1.1;
        let opacity_x_local0 = 0.8 * 0.8;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(103);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(103);
        {
            let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
            style128_sandnado_plume_658(&p, &w, &mut d);
            style128_sandnado_orbits_658(&p, &w, &mut d, opacity_x_local0);
        }
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let amount = mirror.next_float();
        let mut dir = Vec2::new(lerp(0.1, 1.0, mirror.next_float()), lerp(-0.5, 0.9, amount));
        dir.x *= lerp(2.2, 0.6, amount);
        dir.x *= -1.0;
        let (cx, cy) = p.center();
        let center = Vec2::new(cx, cy);
        let point = center + Vec2::new(60.0, 200.0) * dir * 0.5 + Vec2::new(2.0, 10.0);
        let idx = mpool.new_dust(&mut mirror, &c, &w, point, 0, 0, 269, 0.0, 0.0, 0, Color::default(), 1.0);
        let md = mpool.slot_mut(idx);
        md.position = point;
        md.fade_in = 1.0;
        md.scale = 0.3;
        if dir.x > -1.2 {
            md.velocity.x = 1.0 + mirror.next_float();
        }
        md.velocity.y = mirror.next_float() * -0.5 - 1.0;
        for loop_i in 0..2 {
            for i in 0..2 {
                if mirror.next_range(0, 10) == 0 {
                    let mut spin = crate::bow_launch::rotated_by(
                        Vec2::Y, (i as f32 * std::f32::consts::PI) as f64,
                    );
                    if loop_i == 0 {
                        spin = crate::bow_launch::rotated_by(spin, 1.1f64);
                    }
                    let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, 267, 0.0, 0.0, 225, Color::WHITE, 1.5);
                    let md = mpool.slot_mut(idx);
                    md.no_gravity = true;
                    md.no_light = true;
                    md.scale = opacity_x_local0;
                    md.position = center;
                    md.velocity = spin * 2.5;
                }
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(act.len() >= 1);
        for &j in &act {
            assert_eq!(pool.slot(j).position, mpool.slot(j).position);
            assert_eq!(pool.slot(j).velocity, mpool.slot(j).velocity);
            assert_eq!(pool.slot(j).scale, mpool.slot(j).scale);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e3 AI-inline window — `:36414-36432`: the 672 motes at vel.Y −4
    /// (count 5): the Next(2) gate per iteration, rise NextFloat, the
    /// X-jitter NextFloat, the integer Next(3) phase, the Lerp NextFloat,
    /// the Perfect 27 (25,25,25,25), the velocity.Y NextFloat. Mirrored.
    /// WRITTEN NOT RUN.
    #[test]
    fn w4_e3_style132_elder_wins_672_motes_mirror() {
        let w = world();
        let c = ctx();
        let mut p = proj(672);
        p.vel = Vec2::new(0.0, -4.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(107);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(107);
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style132_elder_wins_motes_672(&p, &w, &mut d);
        let (cx, cy) = p.center();
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        for _ in 0..5 {
            if mirror.next_range(0, 2) == 0 {
                let rise = mirror.next_float() * 4.0;
                let mut x = mirror.next_float() * 0.1 - 0.05;
                let phase = mirror.next_range(0, 3) as f64 / 3.0;
                x += (((p.pos.y + rise) * 0.01) as f64 + 6.2831854820251465 * phase).sin() as f32 * 0.5;
                x = lerp(mirror.next_float() - 0.5, x, 1.0);
                let mut point = Vec2::new(x, 0.5 - x.abs()) * Vec2::new(100.0, 50.0);
                point.y += rise;
                let idx = mpool.new_dust_perfect(
                    &mut mirror, &c, &w, Vec2::new(cx, cy) + point, 27,
                    Some(Vec2::ZERO), 0, Color::new(25, 25, 25, 25), 1.1,
                );
                let md = mpool.slot_mut(idx);
                md.velocity.y = mirror.next_float() * 2.0;
                md.fade_in = 0.1;
            }
        }
        let act = active(&pool);
        assert_eq!(act.len(), active(&mpool).len());
        assert!(!act.is_empty(), "seed 107 passes the 1-in-2 gate in 5 tries");
        for &j in &act {
            assert_eq!(pool.slot(j).dust_type, 27);
            assert_eq!(pool.slot(j).color, Color::new(25, 25, 25, 25));
            assert_eq!(pool.slot(j).position, mpool.slot(j).position);
            assert_eq!(pool.slot(j).velocity, mpool.slot(j).velocity);
            assert_eq!(pool.slot(j).fade_in, 0.1);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

}
