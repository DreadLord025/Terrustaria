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
/// branch's `SelectRandom` `Next(4)` go with it), the `light = 0.9` write
/// (`:23982`), then the `Next(20)` dust gate (the anniversary `Next(15)` is
/// the second `||` operand). `anniversary` is `Main.tenthAnniversaryWorld`
/// (the pass reads `world.seeds`).
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
        let _ = d.rng.next_range(0, 4); // SelectRandom's pick (gore skipped)
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
// (`projectile_ai_misc::ai_super_star_slash`); these emitters replace its
// count-only dust skip blocks at the merge. The gate draws d16 keeps
// (`Next(8)` / `Next(3)` + the `NextFloat` pairs) are exactly the emitters'
// leading draws — the reconciliation is a drop-in.
// ---------------------------------------------------------------------------

/// `:22292-22312` — the 977/1045 Zenith slash's dust: the `Next(8)` gate,
/// then the dust-172 (977) / dust-229 (1045) at the centre (noGravity, the
/// `NextVector2Circular(1,1)` + 0.5·vel velocity). The caller owns the
/// ×0.96 drag.
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
        for i in 0..8 {
            let t = smirror.next_range(68, 71);
            assert_eq!(spool.slot(i).dust_type, t, "the type arg precedes the internals");
            assert!(spool.slot(i).no_gravity);
        }
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
    /// Next(6). Mirrored on both seed flags.
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
        let mut qmirror = UnifiedRandom::new(161);
        let mut qd = ProjDust { pool: &mut qpool, ctx: &c, rng: &mut qrng };
        style6_burst(&q, &w, &mut qd, 0, 50, true, true);
        assert_eq!(active(&qpool).len(), 50);
        for num53 in 0..50 {
            let t = if qmirror.next_range(0, 3) == 0 { 309 } else { 0 };
            assert_eq!(qpool.slot(num53).dust_type, t, "the per-dust pick");
            let no_gravity = num53 % 3 != 0;
            assert_eq!(qpool.slot(num53).no_gravity, no_gravity);
            if !no_gravity {
                assert!(qpool.slot(num53).velocity.y < 0.0, "the −2.2 Y kick");
            }
        }
        assert_eq!(qrng.next_range(0, 1000), qmirror.next_range(0, 1000));
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
        let mut d = ProjDust { pool: &mut pool, ctx: &c, rng: &mut rng };
        style8_water_bolt_lava(&p, &w, &mut d);
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

}
