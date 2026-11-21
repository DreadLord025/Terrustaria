//! W4-g64 — the Projectile.cs per-kind TRAIL dust lane 2 (kinds ≥ 401).
//! Batch 1: 406 / 419 / 420 / 421 / 422 / 447 / 450 / 455 / 461 / 465.
//! Batch 2: 466 / 497 / 503 / 504 / 509 / 533 / 537 / 538 / 539 / 573.
//!
//! Scope (docs/research/66-proj-trail-dust-claims.md): the REPEATING in-flight
//! dust sites inside `AI()`'s per-kind arms — the dust a kind sheds while it
//! flies. One-shot spawn-latch bursts stay with the d53/e3/f5 co-lanes; death
//! bursts are g65's; kinds ≤ 400 are g63's.
//!
//! Contract (the S1-E-1 pool + d50's DustPool, `tr_core::particles`): every
//! draw rides the caller's **UpdateProjectiles** stream (`Main.cs:17775`'s
//! `SwapRandom("UpdateProjectiles")` wraps `Projectile.Update`), in per-slot
//! C# statement order — the explicit `Main.rand` argument draws FIRST (C#
//! evaluates arguments before the call), then the `NewDust` internals roll
//! inside `DustPool::new_dust` (`Dust.cs:75-256`), then the post-spawn writes
//! (which read `Main.dust[6000]` when the pool refuses — the sentinel slot
//! takes the writes exactly like the C#). `Dust` is a C# class: the
//! `Dust dustN = Main.dust[idx]` aliases below are pool-slot writes, ported
//! as `slot_mut` chains in the same order.
//!
//! UNWIRED pure port (the d50 / `npc_ai_*` code-only precedent): the wiring
//! wave threads `app.dust` + the frame's `spawn_ctx()` into the
//! `UpdateProjectiles` pass (`world_sim/projectiles.rs`) and calls each fn at
//! its cited site, passing the slot snapshot at the C# read moment. Lights
//! (`Lighting.AddLight` / `Utils.PlotTileLine`) stay the e6/f7 addlight-proj
//! lanes; gore bodies AND their gate draws stay d55/g16 (each skipped gate is
//! cited at its site); sounds stay d76. `customData` is the pool's documented
//! dropped field (`particles.rs`'s `Dust` record).
//!
//! Numeric conventions: `Utils.RotatedBy` (`Utils.cs:1675-1684`) is the
//! double-angle float-cos/sin spin — the shared `bow_launch::rotated_by`.
//! `Utils.ToRotation` (`Utils.cs:1655-1658`) is `(float)Math.Atan2(Y, X)` —
//! the DOUBLE atan2 cast to f32 ([`to_rotation`]). XNA `Vector2.Normalize`
//! divides by the length with NO zero guard — [`xna_normalize`] mirrors the
//! d50 convention (`v / v.length()`), not `normalize_or_zero`.
//! `MathHelper.Lerp(a,b,t) = a + (b-a)*t`.
//!
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by the
//! merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::bow_launch::rotated_by;

/// `Utils.ToRotation` (`Utils.cs:1655-1658`): `(float)Math.Atan2(v.Y, v.X)`
/// — the double atan2 of the widened floats, cast back.
fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// XNA `Vector2.Normalize` — NO zero guard (the d50 `dust_physics`
/// convention: `v / v.length()`, NaN on zero exactly like the source).
fn xna_normalize(v: Vec2) -> Vec2 {
    v / v.length()
}

/// `Math.Sign(float)` — -1 / 0 / +1 (the 447 column's facing read).
fn sign_f(v: f32) -> i32 {
    if v < 0.0 {
        -1
    } else if v > 0.0 {
        1
    } else {
        0
    }
}

/// Kind 406 — the Nebula Blaze's blue flame trail (`AI()` aiStyle 60,
/// `Projectile.cs:29197-29233`), inside the `ai[0] > 3f` window (the caller's
/// gate — the block sits under `:29003`'s `if`, after the `:29153` age add).
/// Six box dusts type 4 (alpha 175, colour (0,80,255,100), scale 1.2) at the
/// 6-px-inset box, each with THREE independent `Next(2)` alpha bumps, then
/// the `Next(4)` seventh-dust gate. RNG per iteration: 3×`Next(2)` AFTER the
/// spawn (the post-writes), then the tail `Next(4)`.
pub fn trail_406(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num470 = 175; // `:29199`
    let new_color2 = Color::new(0, 80, 255, 100); // `:29200`
    let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
    for num471 in 0..6 {
        // `:29203` — (velocity * i) / 6f, the multiply BEFORE the divide.
        let vector36 = vel * num471 as f32 / 6.0;
        let num472 = 6;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            pos + Vec2::ONE * 6.0,
            w - num472 * 2,
            h - num472 * 2,
            4,
            0.0,
            0.0,
            num470,
            new_color2,
            1.2,
        );
        let d = dust.slot_mut(idx);
        // `:29206-29217` — three separate Next(2) gates, +25 alpha each.
        if rng.next_max(2) == 0 {
            d.alpha += 25;
        }
        if rng.next_max(2) == 0 {
            d.alpha += 25;
        }
        if rng.next_max(2) == 0 {
            d.alpha += 25;
        }
        d.no_gravity = true; // `:29218`
        d.velocity *= 0.3; // `:29219`
        d.velocity += vel * 0.5; // `:29220`
        d.position = center; // `:29221`
        d.position.x -= vector36.x; // `:29222`
        d.position.y -= vector36.y; // `:29223`
        d.velocity *= 0.2; // `:29224`
    }
    // `:29226-29232` — the seventh dust on a Next(4).
    if rng.next_max(4) == 0 {
        let num474 = 6;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            pos + Vec2::ONE * 6.0,
            w - num474 * 2,
            h - num474 * 2,
            4,
            0.0,
            0.0,
            num470,
            new_color2,
            1.2,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.velocity += vel * 0.5;
    }
}

/// The aiStyle-73 pillar lift (`Projectile.cs:30490-30495`): `num569 = 2f`,
/// scaled by `timeLeft / 60f` when that is below 1 (the last-second fade).
fn pillar_lift(time_left: i32) -> f32 {
    let mut num569 = 2.0f32;
    let num570 = time_left as f32 / 60.0;
    if num570 < 1.0 {
        num569 *= num570;
    }
    num569
}

/// Kind 419 — the Solar pillar's ember drip (`AI()` aiStyle 73,
/// `Projectile.cs:30496-30515`): two type-222 dusts per tick. Per-iteration
/// draw order: the vector55 magnitude `NextDouble`, its rotation
/// `NextDouble`, the `NewDust` internals, the (later-overwritten) scale
/// `NextDouble` — the C# writes `scale = 1 + NextDouble*0.3` at `:30503`
/// then pins `scale = 0.6` at `:30511`; BOTH the draw and the dead write are
/// kept — then the position `Next(-2, 3)`.
pub fn trail_419(
    pos: Vec2,
    w: i32,
    h: i32,
    time_left: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num569 = pillar_lift(time_left);
    for _ in 0..2 {
        // `:30500` — (0, -lift) * (0.85 + NextDouble*0.2), spun by
        // (NextDouble - 0.5) * π/2 (the f64 literal).
        let mag = 0.85 + rng.next_double() as f32 * 0.2;
        let ang = (rng.next_double() - 0.5) * 1.5707963705062866;
        let vector55 = rotated_by(Vec2::new(0.0, -num569) * mag, ang);
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 222, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale = 1.0 + rng.next_double() as f32 * 0.3; // `:30503` (dead write)
        d.velocity *= 0.5; // `:30504`
        if d.velocity.y > 0.0 {
            d.velocity.y *= -1.0; // `:30505-30508`
        }
        let dx = 2 + rng.next_range(-2, 3); // `:30509`
        d.position -= Vec2::new(dx as f32, 0.0);
        d.velocity += vector55; // `:30510`
        d.scale = 0.6; // `:30511`
        d.fade_in = d.scale + 0.2; // `:30512`
        d.velocity.y *= 2.0; // `:30513`
    }
}

/// Kind 420 — the Vortex pillar's drip (`Projectile.cs:30516-30541`): the
/// 419 shape with the type roll `219`, `Next(5) == 0 → 222` (`:30521-30525`,
/// drawn BEFORE the spawn), the extra `velocity.X *= 0.5` (`:30536`) and no
/// final ×2 difference (the Y ×2 stays).
pub fn trail_420(
    pos: Vec2,
    w: i32,
    h: i32,
    time_left: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num569 = pillar_lift(time_left);
    for _ in 0..2 {
        let mag = 0.85 + rng.next_double() as f32 * 0.2; // `:30520`
        let ang = (rng.next_double() - 0.5) * 1.5707963705062866;
        let vector56 = rotated_by(Vec2::new(0.0, -num569) * mag, ang);
        let mut num574 = 219; // `:30521`
        if rng.next_max(5) == 0 {
            num574 = 222; // `:30522-30525`
        }
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, num574, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale = 1.0 + rng.next_double() as f32 * 0.3; // `:30528` (dead write)
        d.velocity *= 0.5; // `:30529`
        if d.velocity.y > 0.0 {
            d.velocity.y *= -1.0;
        }
        let dx = 2 + rng.next_range(-2, 3); // `:30534`
        d.position -= Vec2::new(dx as f32, 0.0);
        d.velocity += vector56; // `:30535`
        d.velocity.x *= 0.5; // `:30536`
        d.scale = 0.6; // `:30537`
        d.fade_in = d.scale + 0.2; // `:30538`
        d.velocity.y *= 2.0; // `:30539`
    }
}

/// Kind 421 — the Nebula pillar's drip (`Projectile.cs:30542-30582`): two
/// type-221 dusts (the ×0.1 damp, the π/4 spin range, the ×2.5 Y) plus the
/// `timeLeft % 10 == 0` nine-dust ring. The ring's magnitude `num578` is ONE
/// `NextDouble` (`:30563`) shared by all nine; each ring dust keeps the dead
/// `scale = 0.7 + NextDouble*0.3` write (`:30569`) and the `velocity *= 0f`
/// zero (`:30570` — the following `> 0` flip is dead but kept).
pub fn trail_421(
    pos: Vec2,
    w: i32,
    h: i32,
    time_left: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num569 = pillar_lift(time_left);
    for _ in 0..2 {
        let mag = 0.85 + rng.next_double() as f32 * 0.2; // `:30546`
        let ang = (rng.next_double() - 0.5) * 0.7853981852531433;
        let vector57 = rotated_by(Vec2::new(0.0, -num569) * mag, ang);
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 221, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale = 1.0 + rng.next_double() as f32 * 0.3; // `:30549` (dead write)
        d.velocity *= 0.1; // `:30550`
        if d.velocity.y > 0.0 {
            d.velocity.y *= -1.0;
        }
        let dx = 2 + rng.next_range(-2, 3); // `:30555`
        d.position -= Vec2::new(dx as f32, 0.0);
        d.velocity += vector57; // `:30556`
        d.scale = 0.6; // `:30557`
        d.fade_in = d.scale + 0.2; // `:30558`
        d.velocity.y *= 2.5; // `:30559`
    }
    if time_left % 10 == 0 {
        // `:30561-30581` — the ring.
        let num578 = 0.85 + rng.next_double() as f32 * 0.2; // `:30563`
        for num579 in 0..9 {
            // `:30566` — ((i-4)/5f, -lift*num578); the (i-4) casts to float
            // BEFORE the divide.
            let vector58 = Vec2::new((num579 - 4) as f32 / 5.0, -num569 * num578);
            let idx = dust.new_dust(rng, ctx, world, pos, w, h, 222, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.scale = 0.7 + rng.next_double() as f32 * 0.3; // `:30569` (dead)
            d.velocity *= 0.0; // `:30570`
            if d.velocity.y > 0.0 {
                d.velocity.y *= -1.0; // dead (velocity is zero) — kept
            }
            let dx = 2 + rng.next_range(-2, 3); // `:30575`
            d.position -= Vec2::new(dx as f32, 0.0);
            d.velocity += vector58; // `:30576`
            d.scale = 0.6; // `:30577`
            d.fade_in = d.scale + 0.2; // `:30578`
            d.velocity.y *= 2.0; // `:30579`
        }
    }
}

/// Kind 422 — the Stardust pillar's drip (`Projectile.cs:30583-30603`): the
/// 419 shape with the type roll `219 + Next(5)` (`:30590`, an ARGUMENT draw —
/// it rolls before the `NewDust` internals).
pub fn trail_422(
    pos: Vec2,
    w: i32,
    h: i32,
    time_left: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num569 = pillar_lift(time_left);
    for _ in 0..2 {
        let mag = 0.85 + rng.next_double() as f32 * 0.2; // `:30589`
        let ang = (rng.next_double() - 0.5) * 1.5707963705062866;
        let vector59 = rotated_by(Vec2::new(0.0, -num569) * mag, ang);
        let dust_type = 219 + rng.next_max(5); // `:30590`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, dust_type, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale = 1.0 + rng.next_double() as f32 * 0.3; // `:30592` (dead write)
        d.velocity *= 0.5; // `:30593`
        if d.velocity.y > 0.0 {
            d.velocity.y *= -1.0;
        }
        let dx = 2 + rng.next_range(-2, 3); // `:30598`
        d.position -= Vec2::new(dx as f32, 0.0);
        d.velocity += vector59; // `:30599`
        d.scale = 0.6; // `:30600`
        d.fade_in = d.scale + 0.2; // `:30601`
        d.velocity.y *= 2.0; // `:30602`
    }
}

/// Kind 447 — the Moon Lord's phantasmal deathray COLUMN smoke (`AI()`
/// aiStyle 79, `Projectile.cs:30884-30901`). The caller runs the `:30840-
/// 30883` column scan first (position/height are the scanned column, the
/// `nPC11` host's centre and the target player's bottom); `npc_vx` is the
/// host NPC's `velocity.X` at the read moment. Two type-228 base puffs, then
/// the `Next(5)` type-31 side puff. `width / 2` is INTEGER division
/// (`:30896`); `Math.Sign(npc_vx)` is the float sign.
pub fn trail_447(
    pos: Vec2,
    w: i32,
    h: i32,
    npc_vx: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for num602 in 0..2 {
        // `:30888` — box (width, 16) at (x, y + height - 16), type 228.
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(pos.x, pos.y + h as f32 - 16.0),
            w,
            16,
            228,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:30889`
        d.velocity *= 0.5; // `:30890`
        // `:30891` — velocity.X -= (i - npcVx*2/3).
        d.velocity.x -= num602 as f32 - npc_vx * 2.0 / 3.0;
        d.scale = 2.8; // `:30892`
    }
    if rng.next_max(5) == 0 {
        // `:30894-30900` — the side puff at the column's upwind edge.
        let px = pos.x + (w / 2) as f32 - ((w / 2) * sign_f(npc_vx)) as f32 - 4.0;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(px, pos.y + h as f32 - 16.0),
            4,
            16,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5; // `:30897`
        d.velocity.x -= npc_vx / 2.0; // `:30898`
        d.velocity.y = -d.velocity.y.abs(); // `:30899`
    }
}

/// Kind 450 — the Bee Gun's bee smoke (`AI()` aiStyle 14,
/// `Projectile.cs:25188-25212`; the `ai[1] == 0` sound latch is the caller's
/// state, d76's cue). Two independent gates: `Next(2)` → one type-228 with
/// the −2/+2 offset and the `Next(50)` scale bump; `Next(4)` → a second with
/// the +0.3 bump and the ×0.1 damp. The gates draw EVERY tick.
pub fn trail_450(
    pos: Vec2,
    w: i32,
    h: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(2) == 0 {
        // `:25195-25203`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 228, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position.x -= 2.0;
        d.position.y += 2.0;
        d.scale += rng.next_max(50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity.y -= 2.0;
    }
    if rng.next_max(4) == 0 {
        // `:25204-25212`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 228, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position.x -= 2.0;
        d.position.y += 2.0;
        d.scale += 0.3 + rng.next_max(50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity *= 0.1;
    }
}

/// The aiStyle-84 laser-tip pair shared by 455/642 (`Projectile.cs:31619-
/// 31639` / `:31640-31660`, identical bodies): two type-229 side sparks at
/// `Center + velocity*(localAI[1] - lead)` plus the `Next(5)` type-31 puff.
/// Per-spark draws: `Next(2)` (the ±π/2 side), `NextDouble` (the 2..4 speed),
/// then the internals. The puff's offset draw is one `NextDouble`. The
/// `PlotTileLine` light (`:31637-31638`) is the f7 addlight lane's.
fn laser_tip_229(
    center: Vec2,
    vel: Vec2,
    w: i32,
    local_ai1: f32,
    lead: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vector77 = center + vel * (local_ai1 - lead);
    for _ in 0..2 {
        // `:31624` — ToRotation() ± (float)π/2, the sign off Next(2).
        let side = if rng.next_max(2) == 1 { -1.0 } else { 1.0 };
        let num677 = to_rotation(vel) + side * (std::f32::consts::PI / 2.0);
        // `:31625-31626` — speed NextDouble*2+2; Math.Cos/Sin of the widened
        // float (double), cast back.
        let num678 = rng.next_double() as f32 * 2.0 + 2.0;
        let vector78 = Vec2::new(
            (num677 as f64).cos() as f32 * num678,
            (num677 as f64).sin() as f32 * num678,
        );
        let idx = dust.new_dust(rng, ctx, world, vector77, 0, 0, 229, vector78.x, vector78.y, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:31628`
        d.scale = 1.7; // `:31629`
    }
    if rng.next_max(5) == 0 {
        // `:31631-31636` — the puff: the perpendicular offset is
        // vel.RotatedBy(π/2) * (NextDouble - 0.5) * width, minus One*4.
        let off = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            vector77 + off - Vec2::ONE * 4.0,
            8,
            8,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.velocity.y = -d.velocity.y.abs();
    }
}

/// Kind 455 — the Phantasmal Deathray's tip (`Projectile.cs:31619-31639`):
/// [`laser_tip_229`] with the 14-px lead.
pub fn trail_455(
    center: Vec2,
    vel: Vec2,
    w: i32,
    local_ai1: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    laser_tip_229(center, vel, w, local_ai1, 14.0, rng, dust, ctx, world);
}

/// Kind 461 — the Charged Blaster Laser's tip (`Projectile.cs:31661-31681`):
/// the 8-px lead, the 1..1.8 speed (`NextDouble*0.8 + 1`), type 226 at scale
/// 1.2; the same `Next(5)` puff.
pub fn trail_461(
    center: Vec2,
    vel: Vec2,
    w: i32,
    local_ai1: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vector81 = center + vel * (local_ai1 - 8.0); // `:31663`
    for _ in 0..2 {
        let side = if rng.next_max(2) == 1 { -1.0 } else { 1.0 }; // `:31666`
        let num687 = to_rotation(vel) + side * (std::f32::consts::PI / 2.0);
        let num688 = rng.next_double() as f32 * 0.8 + 1.0; // `:31667`
        let vector82 = Vec2::new(
            (num687 as f64).cos() as f32 * num688,
            (num687 as f64).sin() as f32 * num688,
        );
        let idx = dust.new_dust(rng, ctx, world, vector81, 0, 0, 226, vector82.x, vector82.y, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:31670`
        d.scale = 1.2; // `:31671`
    }
    if rng.next_max(5) == 0 {
        // `:31673-31678`
        let off = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            vector81 + off - Vec2::ONE * 4.0,
            8,
            8,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.velocity.y = -d.velocity.y.abs();
    }
}

/// Kind 465 — the Phantasmal Eye's wake (`AI()` aiStyle 88,
/// `Projectile.cs:32011-32044`), behind the caller's `alpha < 150 &&
/// ai[0] < 180` gate (`:32007-32010`). Two single-iteration loops: the
/// 0.2-width spark (velocity = the outward unit ×2) and the 0.6-width spark
/// (velocity zero). Each draws ONE `NextDouble` for the spin; the ±0.5
/// clamps (`:32014-32021`) are DEAD (NextDouble ∈ [0,1) already lands in
/// [-0.5, 0.5)) — kept verbatim. The spin angle is `num719 * (float)(π*2)`
/// — the f32 product widened for RotatedBy.
pub fn trail_465(
    center: Vec2,
    vel: Vec2,
    w: i32,
    scale: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let pi2 = (std::f64::consts::PI * 2.0) as f32;
    // `:32011-32027` — the near spark.
    let mut num719 = rng.next_double() as f32 * 1.0 - 0.5;
    if num719 < -0.5 {
        num719 = -0.5;
    }
    if num719 > 0.5 {
        num719 = 0.5;
    }
    let vector92 = rotated_by(
        rotated_by(Vec2::new(-(w as f32) * 0.2 * scale, 0.0), (num719 * pi2) as f64),
        to_rotation(vel) as f64,
    );
    let idx = dust.new_dust(
        rng,
        ctx,
        world,
        center - Vec2::ONE * 5.0,
        10,
        10,
        226,
        -vel.x / 3.0,
        -vel.y / 3.0,
        150,
        Color::TRANSPARENT,
        0.7,
    );
    let d = dust.slot_mut(idx);
    d.position = center + vector92; // `:32024`
    // `:32025` — Normalize(position - Center) * 2 (XNA, no zero guard).
    d.velocity = xna_normalize(d.position - center) * 2.0;
    d.no_gravity = true; // `:32026`
    // `:32028-32044` — the far spark (velocity ZERO, the 0.6 width).
    let mut num722 = rng.next_double() as f32 * 1.0 - 0.5;
    if num722 < -0.5 {
        num722 = -0.5;
    }
    if num722 > 0.5 {
        num722 = 0.5;
    }
    let vector93 = rotated_by(
        rotated_by(Vec2::new(-(w as f32) * 0.6 * scale, 0.0), (num722 * pi2) as f64),
        to_rotation(vel) as f64,
    );
    let idx = dust.new_dust(
        rng,
        ctx,
        world,
        center - Vec2::ONE * 5.0,
        10,
        10,
        226,
        -vel.x / 3.0,
        -vel.y / 3.0,
        150,
        Color::TRANSPARENT,
        0.7,
    );
    let d = dust.slot_mut(idx);
    d.velocity = Vec2::ZERO; // `:32041` (before the position write — order kept)
    d.position = center + vector93; // `:32042`
    d.no_gravity = true; // `:32043`
}

/// Kind 466 — the Phantasmal Bolt's stationary sparkle (`AI()` aiStyle 88,
/// `Projectile.cs:32069-32087`), inside the `velocity == Zero` branch behind
/// the `Next(extraUpdates) == 0` gate (the caller's stationary wait; the
/// `frameCounter`/oldPos kill above it is the machine's state). Two type-226
/// sparks spun off `this.rotation` (NOT the velocity — the bolt is stopped),
/// then the `Next(5)` type-31 puff. The puff's offset reads the ZERO velocity
/// verbatim (the offset collapses to `-One*4`; the draw is kept).
pub fn trail_466(
    center: Vec2,
    vel: Vec2,
    w: i32,
    rotation: f32,
    extra_updates: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(extra_updates) != 0 {
        return; // `:32069` — the gate draws even when it fails.
    }
    for _ in 0..2 {
        // `:32073` — rotation ± (float)π/2 off Next(2).
        let side = if rng.next_max(2) == 1 { -1.0 } else { 1.0 };
        let num726 = rotation + side * (std::f32::consts::PI / 2.0);
        let num727 = rng.next_double() as f32 * 0.8 + 1.0; // `:32074`
        let vector94 = Vec2::new(
            (num726 as f64).cos() as f32 * num727,
            (num726 as f64).sin() as f32 * num727,
        );
        let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 226, vector94.x, vector94.y, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:32077`
        d.scale = 1.2; // `:32078`
    }
    if rng.next_max(5) == 0 {
        // `:32080-32086`
        let vector95 = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            center + vector95 - Vec2::ONE * 4.0,
            8,
            8,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.velocity.y = -d.velocity.y.abs();
    }
}

/// Kind 497 — the Dutchman's cannonball smoke (`AI()` aiStyle 2,
/// `Projectile.cs:23411-23417`): ONE type-27 dust per tick, speeds = the
/// slot velocity, alpha 100, scale 1.2; the position halves toward the
/// centre, then the ×0.3 damp and the −vel×0.1 drift. ZERO explicit draws —
/// only the pool internals roll. The `ai[0]` age/gravity tail (`:23418+`)
/// is the caller's state (already in `ai_style2`).
pub fn trail_497(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
    let idx = dust.new_dust(rng, ctx, world, pos, w, h, 27, vel.x, vel.y, 100, Color::default(), 1.2);
    let d = dust.slot_mut(idx);
    d.position = (d.position + center) / 2.0; // `:23414`
    d.no_gravity = true; // `:23415`
    d.velocity *= 0.3; // `:23416`
    d.velocity -= vel * 0.1; // `:23417`
}

/// Kind 503 — the Star Wrath's star spark (`AI()` aiStyle 5,
/// `Projectile.cs:23954-23962`): the `Next(16) == 0` gate draws EVERY tick;
/// on a pass, `Vector2.UnitX.RotatedByRandom(π/2)` consumes TWO `NextDouble`
/// draws (left operand first, `Utils.cs:1697-1700`), spun again by the
/// velocity's rotation, then ONE type-58 dust (speeds vel×0.5, alpha 150,
/// scale 1.2) whose velocity/position are overwritten from the spin vector.
/// The `:23963` `Next(48)` GORE gate + body are d55/g16's — the wiring wave
/// interleaves that draw AFTER this fn (it is the arm's next statement).
pub fn trail_503(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(16) != 0 {
        return; // `:23956`
    }
    // `:23958` — RotatedByRandom: NextDouble*max - NextDouble*max.
    let max = 1.5707963705062866f64;
    let ang = rng.next_double() * max - rng.next_double() * max;
    let vector3 = rotated_by(rotated_by(Vec2::UNIT_X, ang), to_rotation(vel) as f64);
    let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
    let idx = dust.new_dust(rng, ctx, world, pos, w, h, 58, vel.x * 0.5, vel.y * 0.5, 150, Color::default(), 1.2);
    let d = dust.slot_mut(idx);
    d.velocity = vector3 * 0.66; // `:23960`
    d.position = center + vector3 * 12.0; // `:23961`
}

/// Kind 504 — the Jester's arrow spark window (`AI()` aiStyle 2,
/// `Projectile.cs:23520-23550`; the 954/979 riders share the arm — 979 spawns
/// type 135 instead of 6). `ai0` is the POST-increment counter (the caller
/// runs `:23523`'s `ai[0]++` first). The `num19` probability ladder
/// (`:23526-23540`) and the `Next(100) < num19` gate draw every tick past
/// `ai0 > 3`; the `:23534` Kill is the caller's state (already in
/// `ai_style2`) — the C# still spawns on the kill tick, kept. The dust's
/// position halves toward the centre, ×2 velocity, ×1.6 scale, +vel.
pub fn trail_504(
    kind: u16,
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    ai0: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if ai0 <= 3.0 {
        return; // `:23524` — no draw inside the window.
    }
    let mut num19 = 100; // `:23526`
    if ai0 > 20.0 {
        let num21 = ai0 - 20.0;
        num19 = (100.0f32 * (1.0 - num21 / 40.0)) as i32; // `:23531`
    }
    if ai0 <= 10.0 {
        num19 = ai0 as i32 * 10; // `:23539`
    }
    if rng.next_max(100) < num19 {
        // `:23541-23549`
        let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
        let dust_type = if kind != 979 { 6 } else { 135 }; // `:23543`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, dust_type, 0.0, 0.0, 150, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = (d.position + center) / 2.0; // `:23544`
        d.no_gravity = true; // `:23545`
        d.velocity *= 2.0; // `:23546`
        d.scale *= 1.6; // `:23547`
        d.velocity += vel; // `:23548`
    }
}

/// Kind 509 — the drill's chip dust (`AI()` aiStyle 20,
/// `Projectile.cs:25575-25583`), at the END of the arm (after the hand pin
/// and the rotation writes). The `:25575` velocity-X jitter
/// (`*= 1 + Next(-3,4)*0.01`) is BEHAVIOUR-FEEDING and always drawn — the
/// port's `ai_drill` skipped it with the dust layer; the wiring wave replaces
/// that skip with this fn's head (the draw order is the arm's tail). Then the
/// `Next(6) == 0` gate: the spawn position's `Next(6,10)` argument draw rides
/// the JITTERED velocity; the post-write `velocity.Y` draws `Next(7,13)`.
pub fn trail_509(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: &mut Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:25575` — the always-drawn jitter (behaviour-feeding, kept).
    vel.x *= 1.0 + rng.next_range(-3, 4) as f32 * 0.01;
    if rng.next_max(6) == 0 {
        // `:25576-25583`
        let lead = rng.next_range(6, 10) as f32 * 0.1; // argument draw
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            pos + *vel * lead,
            w,
            h,
            31,
            0.0,
            0.0,
            80,
            Color::default(),
            1.4,
        );
        let d = dust.slot_mut(idx);
        d.position.x -= 4.0; // `:25579`
        d.no_gravity = true; // `:25580`
        d.velocity *= 0.2; // `:25581`
        d.velocity.y = -(rng.next_range(7, 13) as f32) * 0.15; // `:25582`
    }
}

/// Kind 533 — the Deadly Sphere's DASH ring (`AI()` aiStyle 65,
/// `Projectile.cs:29800-29812`), when the dash ladder's `ai[0] == 8` (the
/// caller's gate — the ladder itself is `ai_deadly_sphere`'s state). Four
/// dusts: the type is `SelectRandom(226, 228, 75)` — ONE `Next(3)` argument
/// draw BEFORE the internals (the named `Type:` argument is written first);
/// the offset is `One.RotatedBy(i*π/2).RotatedBy(rotation)`; the scale's
/// `NextFloat` draws AFTER the spawn.
pub fn trail_533_dash_ring(
    center: Vec2,
    rotation: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for num519 in 0..4 {
        let dust_type = [226, 228, 75][rng.next_max(3) as usize]; // `:29804`
        let idx = dust.new_dust(rng, ctx, world, center, 0, 0, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        // `:29806` — the double spin chain.
        let vector39 = rotated_by(
            rotated_by(Vec2::ONE, (num519 as f32 * (std::f32::consts::PI / 2.0)) as f64),
            rotation as f64,
        );
        d.position = center + vector39 * 10.0; // `:29807`
        d.velocity = vector39; // `:29808` (×1f)
        d.scale = 0.6 + rng.next_float() * 0.5; // `:29809`
        d.no_gravity = true; // `:29810`
    }
}

/// Kind 533 — the Deadly Sphere's SPIN ring (`Projectile.cs:30049-30066`),
/// when the frame-table block's `ai[0] == 2` (the caller's gate). The outer
/// `Next(2) == 0` draws once; each of the four iterations draws its OWN
/// `Next(2) != 0` gate (a fail skips that dust's draws entirely). The taken
/// dusts match the dash ring but at scale `0.3 + NextFloat*0.5` and with
/// `noLight = true`; `customData = this` (`:30062`) is the pool's dropped
/// field.
pub fn trail_533_spin_ring(
    center: Vec2,
    rotation: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(2) != 0 {
        return; // `:30049`
    }
    for num532 in 0..4 {
        if rng.next_max(2) == 0 {
            continue; // `:30053` — the per-dust gate.
        }
        let dust_type = [226, 228, 75][rng.next_max(3) as usize]; // `:30055`
        let idx = dust.new_dust(rng, ctx, world, center, 0, 0, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        let vector43 = rotated_by(
            rotated_by(Vec2::ONE, (num532 as f32 * (std::f32::consts::PI / 2.0)) as f64),
            rotation as f64,
        );
        d.position = center + vector43 * 10.0; // `:30058`
        d.velocity = vector43; // `:30059`
        d.scale = 0.3 + rng.next_float() * 0.5; // `:30060`
        d.no_gravity = true; // `:30061`
        // `:30062` customData = this — dropped field (particles.rs).
        d.no_light = true; // `:30063`
    }
}

/// Kind 537 — the Nebula Blaze EX's laser tip (`AI()` aiStyle 84,
/// `Projectile.cs:31682-31707`): the 461 shape with the age-scaled spark
/// count. `num691 = Clamp(Lerp(0,1,localAI[0]/30),0,1)` (Lerp(0,1,t) = t);
/// `num692 = (int)(2*num691)` — 0/1/2 sparks; the puff gate widens as the
/// beam ages: `Next(5 + (int)((1-num691)*5))`.
pub fn trail_537(
    center: Vec2,
    vel: Vec2,
    w: i32,
    local_ai0: f32,
    local_ai1: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:31684` — MathHelper.Lerp(0,1,t) = 0 + (1-0)*t = t, then Clamp.
    let num691 = (local_ai0 / 30.0).clamp(0.0, 1.0);
    let vector83 = center + vel * (local_ai1 - 8.0); // `:31685`
    let num692 = (2.0f32 * num691) as i32; // `:31686`
    if num692 > 0 {
        for _ in 0..num692 {
            let side = if rng.next_max(2) == 1 { -1.0 } else { 1.0 }; // `:31691`
            let num694 = to_rotation(vel) + side * (std::f32::consts::PI / 2.0);
            let num695 = rng.next_double() as f32 * 0.8 + 1.0; // `:31692`
            let vector84 = Vec2::new(
                (num694 as f64).cos() as f32 * num695,
                (num694 as f64).sin() as f32 * num695,
            );
            let idx = dust.new_dust(rng, ctx, world, vector83, 0, 0, 226, vector84.x, vector84.y, 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:31695`
            d.scale = 1.2; // `:31696`
        }
    }
    // `:31699` — the widening gate.
    if rng.next_max(5 + ((1.0 - num691) * 5.0) as i32) == 0 {
        let off = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            vector83 + off - Vec2::ONE * 4.0,
            8,
            8,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.velocity.y = -d.velocity.y.abs();
    }
}

/// Kind 538 — the Rainbow Gun's rising paint drip (`AI()` aiStyle 14,
/// `Projectile.cs:25161-25187`), behind the `velocity.Y < 0 && ai[0] < 60`
/// gate (checked here — the `localAI[1]` sound latch above it is d76's).
/// Two independent gates draw EVERY eligible tick: `Next(4)` → type 180 at
/// the centre with the `Next(50)` scale bump and −2 Y; `Next(6)` → type 176
/// with the +0.3 bump and the ×0.1 damp.
pub fn trail_538(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    ai0: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !(vel.y < 0.0 && ai0 < 60.0) {
        return; // `:25168` — no draws outside the window.
    }
    let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
    if rng.next_max(4) == 0 {
        // `:25170-25177`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 180, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.scale += rng.next_max(50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity.y -= 2.0;
    }
    if rng.next_max(6) == 0 {
        // `:25178-25185`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 176, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.scale += 0.3 + rng.next_max(50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity *= 0.1;
    }
}

/// Kind 539 — the Tempest minion's HOST-phase sparkle (`AI()` aiStyle 102,
/// `Projectile.cs:33301-33309`), while `ai[0] < 210` (the caller's window —
/// the host-drag/homing above it is `ai_nebula_eye`'s shape). `Next(12)` →
/// one type-180 dust pinned to the centre, ×0.2 velocity.
pub fn trail_539_host(
    center: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(12) == 0 {
        // `:33303-33309`
        let idx = dust.new_dust(rng, ctx, world, center, 8, 8, 180, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.velocity *= 0.2;
        d.no_gravity = true;
    }
}

/// Kind 539 — the Tempest shot's FLIGHT-phase paint drip (`Projectile.cs:
/// 33408-33434`), once `ai[0] >= 210` (the caller's window; the frame cycle
/// above is state). Two gates: `Next(2)` → type 180, `Next(3)` → type 176;
/// both take the `Next(50)` scale bump, the ×0.1 damp and the
/// `NextFloat*1.5` fadeIn (the fadeIn draw is LAST).
pub fn trail_539_flight(
    pos: Vec2,
    w: i32,
    h: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(2) == 0 {
        // `:33418-33425`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 180, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale += rng.next_max(50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity *= 0.1;
        d.fade_in = rng.next_float() * 1.5;
    }
    if rng.next_max(3) == 0 {
        // `:33426-33433`
        let idx = dust.new_dust(rng, ctx, world, pos, w, h, 176, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale += 0.3 + rng.next_max(50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity *= 0.1;
        d.fade_in = rng.next_float() * 1.5;
    }
}

/// Kind 573 — the Nebula minion's HOST-phase flame (`AI()` aiStyle 102,
/// `Projectile.cs:33320-33331`), while `ai[0] < 90` (the caller's window).
/// `Next(2)` → one type-242 dust pinned to the centre carrying the slot
/// velocity, scale 1.5. The `alpha = 255` pin (`:33330`) is caller state.
pub fn trail_573_host(
    center: Vec2,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(2) == 0 {
        // `:33322-33329`
        let idx = dust.new_dust(rng, ctx, world, center, 8, 8, 242, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.velocity = vel;
        d.no_gravity = true;
        d.scale = 1.5;
    }
}

/// Kind 573 — the Nebula shot's FLIGHT-phase flame (`Projectile.cs:33435-
/// 33446`), once `ai[0] >= 90`. `Next(4)` → one type-242 dust at the centre,
/// ×0.2 velocity, scale 1.5. The `alpha = 0` pin is caller state.
pub fn trail_573_flight(
    center: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_max(4) == 0 {
        // `:33437-33444`
        let idx = dust.new_dust(rng, ctx, world, center, 8, 8, 242, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.velocity *= 0.2;
        d.no_gravity = true;
        d.scale = 1.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("g64".into(), 200, 200)
    }

    /// A permissive ctx: the whole world on-screen, the full budget — the
    /// pool never culls (the e2 `ctx()` convention).
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

    /// W4-g64 — kind 406: six dusts + the gated seventh; the three alpha
    /// bumps roll AFTER each spawn. A mirrored stream through the same pool
    /// calls lands identical records and an identical stream tail.
    #[test]
    fn w4_g64_406_mirrors_the_alpha_ladder_and_seventh_gate() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(1000.0, 900.0);
        let vel = Vec2::new(6.0, -2.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(41);
        trail_406(pos, 30, 30, vel, &mut rng, &mut pool, &c, &w);
        let color = Color::new(0, 80, 255, 100);
        for i in 0..6 {
            let v36 = vel * i as f32 / 6.0;
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos + Vec2::ONE * 6.0, 18, 18, 4, 0.0, 0.0, 175, color, 1.2);
            let d = mpool.slot_mut(idx);
            for _ in 0..3 {
                if mirror.next_max(2) == 0 {
                    d.alpha += 25;
                }
            }
            d.no_gravity = true;
            d.velocity *= 0.3;
            d.velocity += vel * 0.5;
            d.position = pos + Vec2::new(15.0, 15.0) - v36;
            d.velocity *= 0.2;
        }
        if mirror.next_max(4) == 0 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos + Vec2::ONE * 6.0, 18, 18, 4, 0.0, 0.0, 175, color, 1.2);
            let d = mpool.slot_mut(idx);
            d.velocity *= 0.5;
            d.velocity += vel * 0.5;
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..16 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000), "stream tail");
        }
    }

    /// W4-g64 — kind 419: the per-iteration draw order is magnitude, spin,
    /// internals, the DEAD scale write's NextDouble, then Next(-2,3). The
    /// final scale is 0.6 and fadeIn 0.8 regardless of the dead draw.
    #[test]
    fn w4_g64_419_keeps_the_dead_scale_draw_and_pins_the_shape() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(800.0, 700.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(7);
        trail_419(pos, 40, 600, 120, &mut rng, &mut pool, &c, &w);
        for _ in 0..2 {
            let mag = 0.85 + mirror.next_double() as f32 * 0.2;
            let ang = (mirror.next_double() - 0.5) * 1.5707963705062866;
            let v55 = rotated_by(Vec2::new(0.0, -2.0) * mag, ang);
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 40, 600, 222, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.scale = 1.0 + mirror.next_double() as f32 * 0.3;
            d.velocity *= 0.5;
            if d.velocity.y > 0.0 {
                d.velocity.y *= -1.0;
            }
            let dx = 2 + mirror.next_range(-2, 3);
            d.position -= Vec2::new(dx as f32, 0.0);
            d.velocity += v55;
            d.scale = 0.6;
            d.fade_in = 0.8;
            d.velocity.y *= 2.0;
        }
        assert_eq!(active(&pool).len(), 2);
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
            assert_eq!(pool.slot(i).scale, 0.6);
            assert_eq!(pool.slot(i).fade_in, 0.8);
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 420: the type roll draws BEFORE the internals; the
    /// extra X ×0.5 lands after the vector add.
    #[test]
    fn w4_g64_420_type_roll_precedes_the_spawn() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(800.0, 700.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        trail_420(pos, 40, 600, 30, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 2);
        for i in active(&pool) {
            let d = pool.slot(i);
            assert!(d.dust_type == 219 || d.dust_type == 222);
            assert_eq!(d.scale, 0.6);
            // The lift is halved at timeLeft 30 (30/60 < 1).
            assert!(d.velocity.y.abs() < 4.0);
        }
    }

    /// W4-g64 — kind 421: the %10 ring fires only on the cadence; its nine
    /// dusts share ONE magnitude draw. timeLeft 100 → 2 + 9 dusts; 101 → 2.
    #[test]
    fn w4_g64_421_ring_cadence_and_shared_magnitude() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(800.0, 700.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        trail_421(pos, 40, 600, 101, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 2, "no ring off-cadence");
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        trail_421(pos, 40, 600, 100, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 11, "2 drips + 9 ring dusts");
        // The ring's X velocities are (i-4)/5 — symmetric around 0.
        let ring: Vec<f32> = (2..11).map(|i| pool.slot(i).velocity.x).collect();
        for (k, vx) in ring.iter().enumerate() {
            assert!((vx - (k as f32 - 4.0) / 5.0).abs() < 1e-6);
        }
    }

    /// W4-g64 — kind 422: the type is 219 + Next(5) ∈ [219, 223].
    #[test]
    fn w4_g64_422_type_range() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        for _ in 0..20 {
            trail_422(Vec2::new(800.0, 700.0), 40, 600, 120, &mut rng, &mut pool, &c, &w);
        }
        for i in active(&pool) {
            let t = pool.slot(i).dust_type;
            assert!((219..=223).contains(&t), "type {t}");
        }
    }

    /// W4-g64 — kind 447: two base puffs + the gated side puff; the side
    /// puff's X anchor uses INTEGER width/2 and Math.Sign(npcVx).
    #[test]
    fn w4_g64_447_column_puffs_and_sign_anchor() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(1000.0, 500.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(29);
        trail_447(pos, 31, 160, -3.0, &mut rng, &mut pool, &c, &w);
        for i in 0..2 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(1000.0, 644.0), 31, 16, 228, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.5;
            d.velocity.x -= i as f32 - (-3.0f32) * 2.0 / 3.0;
            d.scale = 2.8;
        }
        if mirror.next_max(5) == 0 {
            // width/2 = 15 (integer); Sign(-3) = -1 → x + 15 + 15 - 4.
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(1026.0, 644.0), 4, 16, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            let d = mpool.slot_mut(idx);
            d.velocity *= 0.5;
            d.velocity.x -= -1.5;
            d.velocity.y = -d.velocity.y.abs();
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 450: both gates draw every tick; a failed gate spawns
    /// nothing but still consumes its Next.
    #[test]
    fn w4_g64_450_gates_draw_every_tick() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        let mut mirror = UnifiedRandom::new(31);
        for _ in 0..30 {
            trail_450(Vec2::new(900.0, 800.0), 16, 16, &mut rng, &mut pool, &c, &w);
        }
        // The stream consumed at least the 60 gate draws.
        let mut gates = 0;
        for _ in 0..30 {
            let _ = mirror.next_max(2);
            let _ = mirror.next_max(4);
            gates += 2;
        }
        assert_eq!(gates, 60);
        for i in active(&pool) {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 228);
            assert!(d.no_gravity);
        }
    }

    /// W4-g64 — kind 455: the tip pair's side/speed draws precede each
    /// spawn; the puff's offset NextDouble follows the Next(5) gate.
    #[test]
    fn w4_g64_455_tip_pair_mirror() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1200.0, 600.0);
        let vel = Vec2::new(1.0, 0.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(37);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(37);
        trail_455(center, vel, 36, 100.0, &mut rng, &mut pool, &c, &w);
        let tip = center + vel * (100.0 - 14.0);
        for _ in 0..2 {
            let side = if mirror.next_max(2) == 1 { -1.0 } else { 1.0 };
            let ang = 0.0f32 + side * (std::f32::consts::PI / 2.0);
            let spd = mirror.next_double() as f32 * 2.0 + 2.0;
            let v = Vec2::new((ang as f64).cos() as f32 * spd, (ang as f64).sin() as f32 * spd);
            let idx = mpool.new_dust(&mut mirror, &c, &w, tip, 0, 0, 229, v.x, v.y, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.7;
        }
        if mirror.next_max(5) == 0 {
            let off = rotated_by(vel, 1.5707963705062866) * (mirror.next_double() as f32 - 0.5) * 36.0;
            let idx = mpool.new_dust(&mut mirror, &c, &w, tip + off - Vec2::ONE * 4.0, 8, 8, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            let d = mpool.slot_mut(idx);
            d.velocity *= 0.5;
            d.velocity.y = -d.velocity.y.abs();
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 461: the 8-px lead and the 1..1.8 speed band.
    #[test]
    fn w4_g64_461_lead_and_speed_band() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1200.0, 600.0);
        let vel = Vec2::new(0.0, 1.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        trail_461(center, vel, 20, 50.0, &mut rng, &mut pool, &c, &w);
        let sparks = active(&pool);
        assert!(sparks.len() >= 2);
        for &i in sparks.iter().take(2) {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 226);
            assert_eq!(d.scale, 1.2);
            // The spawn box is 0x0 → clamped 5x5; the position jitter is
            // [4,5) around the tip; the speed rides the velocity argument.
            let len = d.velocity.length();
            assert!(len >= 0.9 && len <= 2.1, "speed {len}");
        }
    }

    /// W4-g64 — kind 465: two sparks per tick, the near one's velocity is
    /// the outward unit ×2, the far one's is zero; both keep the dead ±0.5
    /// clamp draws on the stream.
    #[test]
    fn w4_g64_465_wake_pair_mirror() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1500.0, 900.0);
        let vel = Vec2::new(3.0, 4.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(47);
        trail_465(center, vel, 30, 1.0, &mut rng, &mut pool, &c, &w);
        let pi2 = (std::f64::consts::PI * 2.0) as f32;
        let rot = to_rotation(vel);
        for (mult, zero_vel) in [(0.2f32, false), (0.6, true)] {
            let spin = mirror.next_double() as f32 * 1.0 - 0.5;
            let off = rotated_by(
                rotated_by(Vec2::new(-30.0 * mult, 0.0), (spin * pi2) as f64),
                rot as f64,
            );
            let idx = mpool.new_dust(&mut mirror, &c, &w, center - Vec2::ONE * 5.0, 10, 10, 226, -1.0, -4.0 / 3.0, 150, Color::TRANSPARENT, 0.7);
            let d = mpool.slot_mut(idx);
            if zero_vel {
                d.velocity = Vec2::ZERO;
                d.position = center + off;
            } else {
                d.position = center + off;
                d.velocity = xna_normalize(d.position - center) * 2.0;
            }
            d.no_gravity = true;
        }
        assert_eq!(active(&pool).len(), 2);
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        assert_eq!(pool.slot(0).velocity.length(), 2.0);
        assert_eq!(pool.slot(1).velocity, Vec2::ZERO);
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 466: the Next(extraUpdates) gate draws even on failure;
    /// a pass spawns the rotation-spun pair (the velocity is ZERO — the puff
    /// offset collapses but its NextDouble is kept).
    #[test]
    fn w4_g64_466_stationary_gate_and_pair() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1500.0, 900.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(53);
        trail_466(center, Vec2::ZERO, 20, 0.7, 3, &mut rng, &mut pool, &c, &w);
        if mirror.next_max(3) == 0 {
            for _ in 0..2 {
                let side = if mirror.next_max(2) == 1 { -1.0 } else { 1.0 };
                let ang = 0.7f32 + side * (std::f32::consts::PI / 2.0);
                let spd = mirror.next_double() as f32 * 0.8 + 1.0;
                let v = Vec2::new((ang as f64).cos() as f32 * spd, (ang as f64).sin() as f32 * spd);
                let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, 226, v.x, v.y, 0, Color::default(), 1.0);
                let d = mpool.slot_mut(idx);
                d.no_gravity = true;
                d.scale = 1.2;
            }
            if mirror.next_max(5) == 0 {
                let off = rotated_by(Vec2::ZERO, 1.5707963705062866) * (mirror.next_double() as f32 - 0.5) * 20.0;
                let idx = mpool.new_dust(&mut mirror, &c, &w, center + off - Vec2::ONE * 4.0, 8, 8, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = mpool.slot_mut(idx);
                d.velocity *= 0.5;
                d.velocity.y = -d.velocity.y.abs();
            }
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 497: one dust, zero explicit draws; the position halves
    /// toward the centre and the velocity chain is ×0.3 then −vel×0.1.
    #[test]
    fn w4_g64_497_single_smoke_chain() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(1000.0, 800.0);
        let vel = Vec2::new(4.0, 1.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(59);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(59);
        trail_497(pos, 20, 20, vel, &mut rng, &mut pool, &c, &w);
        let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 20, 20, 27, 4.0, 1.0, 100, Color::default(), 1.2);
        let d = mpool.slot_mut(idx);
        d.position = (d.position + Vec2::new(1010.0, 810.0)) / 2.0;
        d.no_gravity = true;
        d.velocity *= 0.3;
        d.velocity -= vel * 0.1;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0), mpool.slot(0));
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 503: the Next(16) gate draws every tick; a pass takes
    /// TWO NextDouble (RotatedByRandom) then the internals; the spawn's
    /// velocity/position are the spin-vector overwrites.
    #[test]
    fn w4_g64_503_rotated_by_random_two_draws() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(900.0, 300.0);
        let vel = Vec2::new(0.0, 8.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(61);
        for _ in 0..40 {
            trail_503(pos, 16, 16, vel, &mut rng, &mut pool, &c, &w);
        }
        for _ in 0..40 {
            if mirror.next_max(16) == 0 {
                let max = 1.5707963705062866f64;
                let ang = mirror.next_double() * max - mirror.next_double() * max;
                let v3 = rotated_by(rotated_by(Vec2::UNIT_X, ang), to_rotation(vel) as f64);
                let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 16, 16, 58, 0.0, 4.0, 150, Color::default(), 1.2);
                let d = mpool.slot_mut(idx);
                d.velocity = v3 * 0.66;
                d.position = Vec2::new(908.0, 308.0) + v3 * 12.0;
            }
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 504: no draws inside ai0 <= 3; the ladder probability
    /// gates the Next(100); the type flips to 135 for the 979 rider.
    #[test]
    fn w4_g64_504_window_ladder_and_type_flip() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(700.0, 500.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(67);
        let mut mirror = UnifiedRandom::new(67);
        trail_504(504, pos, 10, 10, Vec2::new(9.0, 0.0), 2.0, &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty(), "no dust before the window");
        assert_eq!(rng.next_max(1000), mirror.next_max(1000), "no draw before the window");
        for ai0 in [5.0f32, 15.0, 25.0, 45.0] {
            trail_504(979, pos, 10, 10, Vec2::new(9.0, 0.0), ai0, &mut rng, &mut pool, &c, &w);
        }
        for i in active(&pool) {
            assert_eq!(pool.slot(i).dust_type, 135, "979 spawns 135");
            assert!(pool.slot(i).no_gravity);
        }
    }

    /// W4-g64 — kind 509: the jitter draw ALWAYS runs (behaviour-feeding);
    /// the gate + argument draws follow in order.
    #[test]
    fn w4_g64_509_jitter_always_draws() {
        let w = world();
        let c = ctx();
        let mut vel = Vec2::new(10.0, 0.0);
        let mut rng = UnifiedRandom::new(71);
        let mut mirror = UnifiedRandom::new(71);
        let mut pool = DustPool::new();
        trail_509(Vec2::new(600.0, 600.0), 20, 20, &mut vel, &mut rng, &mut pool, &c, &w);
        let jitter = mirror.next_range(-3, 4);
        assert_eq!(vel.x, 10.0 * (1.0 + jitter as f32 * 0.01));
        // Mirror the gate + the taken branch so the streams stay in lockstep.
        if mirror.next_max(6) == 0 {
            let lead = mirror.next_range(6, 10) as f32 * 0.1;
            let mut mpool = DustPool::new();
            let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(600.0, 600.0) + vel * lead, 20, 20, 31, 0.0, 0.0, 80, Color::default(), 1.4);
            let d = mpool.slot_mut(idx);
            d.position.x -= 4.0;
            d.no_gravity = true;
            d.velocity *= 0.2;
            d.velocity.y = -(mirror.next_range(7, 13) as f32) * 0.15;
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 533 dash ring: four dusts, the SelectRandom Next(3)
    /// argument draw precedes each spawn; the scale NextFloat follows.
    #[test]
    fn w4_g64_533_dash_ring_mirror() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1100.0, 700.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(73);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(73);
        trail_533_dash_ring(center, 0.4, &mut rng, &mut pool, &c, &w);
        for i in 0..4 {
            let t = [226, 228, 75][mirror.next_max(3) as usize];
            let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, t, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            let v = rotated_by(rotated_by(Vec2::ONE, (i as f32 * (std::f32::consts::PI / 2.0)) as f64), 0.4);
            d.position = center + v * 10.0;
            d.velocity = v;
            d.scale = 0.6 + mirror.next_float() * 0.5;
            d.no_gravity = true;
        }
        assert_eq!(active(&pool).len(), 4);
        for i in 0..4 {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 533 spin ring: the outer Next(2) and the four per-dust
    /// Next(2) gates all draw; the taken dusts carry noLight.
    #[test]
    fn w4_g64_533_spin_ring_gates_and_nolight() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1100.0, 700.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(79);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(79);
        trail_533_spin_ring(center, 1.1, &mut rng, &mut pool, &c, &w);
        if mirror.next_max(2) == 0 {
            for i in 0..4 {
                if mirror.next_max(2) == 0 {
                    continue;
                }
                let t = [226, 228, 75][mirror.next_max(3) as usize];
                let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, t, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = mpool.slot_mut(idx);
                let v = rotated_by(rotated_by(Vec2::ONE, (i as f32 * (std::f32::consts::PI / 2.0)) as f64), 1.1);
                d.position = center + v * 10.0;
                d.velocity = v;
                d.scale = 0.3 + mirror.next_float() * 0.5;
                d.no_gravity = true;
                d.no_light = true;
            }
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
            assert!(pool.slot(i).no_light);
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 537: the spark count is (int)(2*clamp(age/30)) — 0 at
    /// age 0, 1 at 15..29, 2 at >=30; the puff gate widens 10 → 5.
    #[test]
    fn w4_g64_537_age_scaled_count_and_gate() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1200.0, 600.0);
        let vel = Vec2::new(1.0, 0.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(83);
        // age 0: only the widened Next(10) puff gate can fire.
        trail_537(center, vel, 20, 0.0, 40.0, &mut rng, &mut pool, &c, &w);
        for i in active(&pool) {
            assert_eq!(pool.slot(i).dust_type, 31, "no sparks at age 0");
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(83);
        trail_537(center, vel, 20, 45.0, 40.0, &mut rng, &mut pool, &c, &w);
        let sparks = active(&pool).iter().filter(|&&i| pool.slot(i).dust_type == 226).count();
        assert_eq!(sparks, 2, "two sparks past age 30");
    }

    /// W4-g64 — kind 538: the rising-window gate draws nothing outside it;
    /// inside, both gates draw every tick.
    #[test]
    fn w4_g64_538_window_and_gates() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(800.0, 900.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(89);
        let mut mirror = UnifiedRandom::new(89);
        // Falling (vy > 0) or old (ai0 >= 60): no draws.
        trail_538(pos, 12, 12, Vec2::new(0.0, 2.0), 10.0, &mut rng, &mut pool, &c, &w);
        trail_538(pos, 12, 12, Vec2::new(0.0, -2.0), 60.0, &mut rng, &mut pool, &c, &w);
        assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        assert!(active(&pool).is_empty());
        for _ in 0..30 {
            trail_538(pos, 12, 12, Vec2::new(0.0, -2.0), 10.0, &mut rng, &mut pool, &c, &w);
        }
        for i in active(&pool) {
            let d = pool.slot(i);
            assert!(d.dust_type == 180 || d.dust_type == 176);
            assert_eq!(d.position, Vec2::new(806.0, 906.0), "pinned to centre");
            assert!(d.no_gravity);
        }
    }

    /// W4-g64 — kinds 539/573: the host/flight gate pairs pin the centre and
    /// carry the documented damp/scale chains.
    #[test]
    fn w4_g64_539_573_host_flight_shapes() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1000.0, 1000.0);
        let vel = Vec2::new(2.0, -1.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        for _ in 0..60 {
            trail_539_host(center, &mut rng, &mut pool, &c, &w);
            trail_539_flight(Vec2::new(1000.0, 1000.0), 14, 14, &mut rng, &mut pool, &c, &w);
            trail_573_host(center, vel, &mut rng, &mut pool, &c, &w);
            trail_573_flight(center, &mut rng, &mut pool, &c, &w);
        }
        for i in active(&pool) {
            let d = pool.slot(i);
            match d.dust_type {
                180 | 176 => assert!(d.no_gravity),
                242 => {
                    assert!(d.no_gravity);
                    assert_eq!(d.scale, 1.5);
                    assert_eq!(d.position, center);
                }
                t => panic!("unexpected type {t}"),
            }
        }
        assert!(!active(&pool).is_empty(), "60 ticks of gates spawn something");
    }
}
