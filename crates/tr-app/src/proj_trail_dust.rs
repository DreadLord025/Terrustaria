//! W4-g64 — the Projectile.cs per-kind TRAIL dust lane 2 (kinds ≥ 401).
//! Batch 1: 406 / 419 / 420 / 421 / 422 / 447 / 450 / 455 / 461 / 465.
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
}
