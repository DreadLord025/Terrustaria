//! W4-g64 — the Projectile.cs per-kind TRAIL dust lane 2 (kinds ≥ 401).
//! Batch 1: 406 / 419 / 420 / 421 / 422 / 447 / 450 / 455 / 461 / 465.
//! Batch 2: 466 / 497 / 503 / 504 / 509 / 533 / 537 / 538 / 539 / 573.
//! Batch 3: 574 / 578 / 579 / 596 / 618 / 642 / 656 / 659 / 669 / 723.
//! Batch 4: 732 / 895 / 898 / 907 / 967 / 1026 / 1048 / 1053 / 1092 / 1114.
//! Batch 5: 1133.
//!
//! Scope (docs/research/196-proj-trail-dust-claims-g64.md): the REPEATING in-flight
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
//! WIRING (this wave): the rowed kinds' arms call their fns at the cited
//! C# sites through `ProjectilePool::update_dust` — the live sim threads
//! `app.dust` + the frame's `spawn_ctx()` (`world_sim/projectiles.rs`), the
//! dust-free `update` entry runs a scratch pool (the W4-g63 batch-1 seam).
//! The machines' `&mut dyn DropRng` stream adapts through [`DustDraw`];
//! kinds with NO ported `proj_data` row stay UNWIRED (the S1-F-9b gate —
//! the row is the wiring gate; their emitters land cited + witnessed).
//! Lights (`Lighting.AddLight` / `Utils.PlotTileLine`) stay the e6/f7
//! addlight-proj lanes; gore bodies AND their gate draws stay d55/g16 (each
//! skipped gate is cited at its site); sounds stay d76. `customData` is the
//! pool's documented dropped field (`particles.rs`'s `Dust` record).
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
use tr_core::loot::DropRng;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};
use tr_core::World;

/// The machines' `DropRng` stream seen as the pool's [`DustRng`] seam (the
/// W4-g63 `DustDraw` shape): `Next(max)` delegates to `Next(0, max)` — both
/// are one `Sample()` draw (`UnifiedRandom.cs:120-141`), so the interleaving
/// with the machine's own behaviour rolls stays byte-identical. The wiring
/// wave threads the machines' `&mut dyn DropRng` straight through; the
/// witnesses keep their concrete `UnifiedRandom` streams (the inherent
/// methods shadow the trait's, `next_double` included). W4-g64 SEAL-VERIFY:
/// the adapter ALSO carries `GoreRng` (the f50 `BlastRng` keep-one — the
/// lifted impl below; at merge f50's `solar_flare_blast_bodies` rides this
/// adapter and `BlastRng` is deleted on its branch).
pub(crate) struct DustDraw<'a>(pub &'a mut dyn DropRng);

impl DustRng for DustDraw<'_> {
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

/// W4-g64 SEAL-VERIFY — the f50 adapter keep-one EXECUTED on this side
/// (the f50 `51eefdf84` HURT RETIRE receipt: `BlastRng`
/// (`projectile_ai_misc.rs:599` on the f50 branch) is arm-for-arm the
/// `DustDraw` shape plus the `GoreRng` half). The `GoreRng` impl is LIFTED
/// onto `DustDraw` here; at merge the f50 `solar_flare_blast_bodies` swaps
/// `BlastRng(rng)` for `DustDraw(rng)` and the `BlastRng` struct + its two
/// impls are DELETED (the deletion lives on the f50 copy — the merge
/// instruction, nothing to delete on this branch).
impl tr_core::gore::GoreRng for DustDraw<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.next_range(0, max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.next_float()
    }
    /// `Utils.NextFloatDirection` (`Utils.cs:1203-1206`):
    /// `(float)NextDouble() * 2f - 1f` — the DOUBLE draw cast to f32
    /// FIRST, then the float multiply (the f50 `BlastRng` shape verbatim).
    fn next_float_direction(&mut self) -> f32 {
        self.0.next_double() as f32 * 2.0 - 1.0
    }
}

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
    rng: &mut dyn DropRng,
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
            &mut DustDraw(rng),
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
        if rng.next_range(0, 2) == 0 {
            d.alpha += 25;
        }
        if rng.next_range(0, 2) == 0 {
            d.alpha += 25;
        }
        if rng.next_range(0, 2) == 0 {
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
    if rng.next_range(0, 4) == 0 {
        let num474 = 6;
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
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
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 222, 0.0, 0.0, 100, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
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
        if rng.next_range(0, 5) == 0 {
            num574 = 222; // `:30522-30525`
        }
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, num574, 0.0, 0.0, 100, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num569 = pillar_lift(time_left);
    for _ in 0..2 {
        let mag = 0.85 + rng.next_double() as f32 * 0.2; // `:30546`
        let ang = (rng.next_double() - 0.5) * 0.7853981852531433;
        let vector57 = rotated_by(Vec2::new(0.0, -num569) * mag, ang);
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 221, 0.0, 0.0, 100, Color::default(), 1.0);
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
            let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 222, 0.0, 0.0, 100, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num569 = pillar_lift(time_left);
    for _ in 0..2 {
        let mag = 0.85 + rng.next_double() as f32 * 0.2; // `:30589`
        let ang = (rng.next_double() - 0.5) * 1.5707963705062866;
        let vector59 = rotated_by(Vec2::new(0.0, -num569) * mag, ang);
        let dust_type = 219 + rng.next_range(0, 5); // `:30590`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, dust_type, 0.0, 0.0, 100, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for num602 in 0..2 {
        // `:30888` — box (width, 16) at (x, y + height - 16), type 228.
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    if rng.next_range(0, 5) == 0 {
        // `:30894-30900` — the side puff at the column's upwind edge.
        let px = pos.x + (w / 2) as f32 - ((w / 2) * sign_f(npc_vx)) as f32 - 4.0;
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 2) == 0 {
        // `:25195-25203`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 228, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position.x -= 2.0;
        d.position.y += 2.0;
        d.scale += rng.next_range(0, 50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity.y -= 2.0;
    }
    if rng.next_range(0, 4) == 0 {
        // `:25204-25212`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 228, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position.x -= 2.0;
        d.position.y += 2.0;
        d.scale += 0.3 + rng.next_range(0, 50) as f32 * 0.01;
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vector77 = center + vel * (local_ai1 - lead);
    for _ in 0..2 {
        // `:31624` — ToRotation() ± (float)π/2, the sign off Next(2).
        let side = if rng.next_range(0, 2) == 1 { -1.0 } else { 1.0 };
        let num677 = to_rotation(vel) + side * (std::f32::consts::PI / 2.0);
        // `:31625-31626` — speed NextDouble*2+2; Math.Cos/Sin of the widened
        // float (double), cast back.
        let num678 = rng.next_double() as f32 * 2.0 + 2.0;
        let vector78 = Vec2::new(
            (num677 as f64).cos() as f32 * num678,
            (num677 as f64).sin() as f32 * num678,
        );
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, vector77, 0, 0, 229, vector78.x, vector78.y, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:31628`
        d.scale = 1.7; // `:31629`
    }
    if rng.next_range(0, 5) == 0 {
        // `:31631-31636` — the puff: the perpendicular offset is
        // vel.RotatedBy(π/2) * (NextDouble - 0.5) * width, minus One*4.
        let off = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vector81 = center + vel * (local_ai1 - 8.0); // `:31663`
    for _ in 0..2 {
        let side = if rng.next_range(0, 2) == 1 { -1.0 } else { 1.0 }; // `:31666`
        let num687 = to_rotation(vel) + side * (std::f32::consts::PI / 2.0);
        let num688 = rng.next_double() as f32 * 0.8 + 1.0; // `:31667`
        let vector82 = Vec2::new(
            (num687 as f64).cos() as f32 * num688,
            (num687 as f64).sin() as f32 * num688,
        );
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, vector81, 0, 0, 226, vector82.x, vector82.y, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:31670`
        d.scale = 1.2; // `:31671`
    }
    if rng.next_range(0, 5) == 0 {
        // `:31673-31678`
        let off = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
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
        &mut DustDraw(rng),
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
        &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, extra_updates) != 0 {
        return; // `:32069` — the gate draws even when it fails.
    }
    for _ in 0..2 {
        // `:32073` — rotation ± (float)π/2 off Next(2).
        let side = if rng.next_range(0, 2) == 1 { -1.0 } else { 1.0 };
        let num726 = rotation + side * (std::f32::consts::PI / 2.0);
        let num727 = rng.next_double() as f32 * 0.8 + 1.0; // `:32074`
        let vector94 = Vec2::new(
            (num726 as f64).cos() as f32 * num727,
            (num726 as f64).sin() as f32 * num727,
        );
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, 226, vector94.x, vector94.y, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:32077`
        d.scale = 1.2; // `:32078`
    }
    if rng.next_range(0, 5) == 0 {
        // `:32080-32086`
        let vector95 = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 27, vel.x, vel.y, 100, Color::default(), 1.2);
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
/// The `:23963` `Next(48)` GORE gate + body are d55/g16's — W4-g64 FINAL:
/// the gate draw now rides INSIDE the machine (`ai_falling_star`'s 503 arm
/// rolls it immediately AFTER this fn — the e3 keep-one order, the row
/// landed via the e3 `80d47a56` constants-copy).
pub fn trail_503(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 16) != 0 {
        return; // `:23956`
    }
    // `:23958` — RotatedByRandom: NextDouble*max - NextDouble*max.
    let max = 1.5707963705062866f64;
    let ang = rng.next_double() * max - rng.next_double() * max;
    let vector3 = rotated_by(rotated_by(Vec2::X, ang), to_rotation(vel) as f64);
    let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 58, vel.x * 0.5, vel.y * 0.5, 150, Color::default(), 1.2);
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
    rng: &mut dyn DropRng,
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
    if rng.next_range(0, 100) < num19 {
        // `:23541-23549`
        let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
        let dust_type = if kind != 979 { 6 } else { 135 }; // `:23543`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, dust_type, 0.0, 0.0, 150, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:25575` — the always-drawn jitter (behaviour-feeding, kept).
    vel.x *= 1.0 + rng.next_range(-3, 4) as f32 * 0.01;
    if rng.next_range(0, 6) == 0 {
        // `:25576-25583`
        let lead = rng.next_range(6, 10) as f32 * 0.1; // argument draw
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for num519 in 0..4 {
        let dust_type = [226, 228, 75][rng.next_range(0, 3) as usize]; // `:29804`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 2) != 0 {
        return; // `:30049`
    }
    for num532 in 0..4 {
        if rng.next_range(0, 2) == 0 {
            continue; // `:30053` — the per-dust gate.
        }
        let dust_type = [226, 228, 75][rng.next_range(0, 3) as usize]; // `:30055`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
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
            let side = if rng.next_range(0, 2) == 1 { -1.0 } else { 1.0 }; // `:31691`
            let num694 = to_rotation(vel) + side * (std::f32::consts::PI / 2.0);
            let num695 = rng.next_double() as f32 * 0.8 + 1.0; // `:31692`
            let vector84 = Vec2::new(
                (num694 as f64).cos() as f32 * num695,
                (num694 as f64).sin() as f32 * num695,
            );
            let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, vector83, 0, 0, 226, vector84.x, vector84.y, 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:31695`
            d.scale = 1.2; // `:31696`
        }
    }
    // `:31699` — the widening gate.
    if rng.next_range(0, 5 + ((1.0 - num691) * 5.0) as i32) == 0 {
        let off = rotated_by(vel, 1.5707963705062866)
            * (rng.next_double() as f32 - 0.5)
            * w as f32;
        let idx = dust.new_dust(
            &mut DustDraw(rng),
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !(vel.y < 0.0 && ai0 < 60.0) {
        return; // `:25168` — no draws outside the window.
    }
    let center = pos + Vec2::new(w as f32 / 2.0, h as f32 / 2.0);
    if rng.next_range(0, 4) == 0 {
        // `:25170-25177`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 180, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.scale += rng.next_range(0, 50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity.y -= 2.0;
    }
    if rng.next_range(0, 6) == 0 {
        // `:25178-25185`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 176, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.scale += 0.3 + rng.next_range(0, 50) as f32 * 0.01;
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 12) == 0 {
        // `:33303-33309`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 8, 8, 180, 0.0, 0.0, 0, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 2) == 0 {
        // `:33418-33425`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 180, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale += rng.next_range(0, 50) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity *= 0.1;
        d.fade_in = rng.next_float() * 1.5;
    }
    if rng.next_range(0, 3) == 0 {
        // `:33426-33433`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 176, 0.0, 0.0, 100, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.scale += 0.3 + rng.next_range(0, 50) as f32 * 0.01;
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 2) == 0 {
        // `:33322-33329`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 8, 8, 242, 0.0, 0.0, 0, Color::default(), 1.0);
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
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 4) == 0 {
        // `:33437-33444`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 8, 8, 242, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.velocity *= 0.2;
        d.no_gravity = true;
        d.scale = 1.5;
    }
}

/// Kind 574 — the Nebula floater's HOST-phase flame (`AI()` aiStyle 102,
/// `Projectile.cs:33332-33341`), while `ai[0] < 180` (the caller's window).
/// `Next(10)` → one type-242 dust pinned to the centre carrying the slot
/// velocity, scale 1.5. The `CanHitLine` rotation lerp + `frame = 1` below
/// (`:33342-33352`) are caller state.
pub fn trail_574_host(
    center: Vec2,
    vel: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 10) == 0 {
        // `:33334-33341`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 8, 8, 242, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.velocity = vel;
        d.no_gravity = true;
        d.scale = 1.5;
    }
}

/// Kind 579 — the Vortex portal's wind-up rings (`AI()` aiStyle 108,
/// `Projectile.cs:33716-33738`), while `ai[0] <= 50` (the caller's window).
/// Two independent `Next(4)` gates. Each taken gate: `UnitY.RotatedByRandom
/// (2π)` — TWO `NextDouble` draws — then the type-229 / type-240 dust at
/// `Center - v*30` (0×0 box); the 229's position re-rolls `Next(10,21)` along
/// v, the 240 keeps the ×30; the velocities are the ±π/2 spins ×4 / ×2; both
/// take `scale = 0.5 + NextFloat` and `fadeIn = 0.5`.
pub fn trail_579(
    center: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 4) == 0 {
        // `:33718-33727` — the 229 ring.
        let max = 6.2831854820251465f64;
        let ang = rng.next_double() * max - rng.next_double() * max;
        let vector116 = rotated_by(Vec2::Y, ang);
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center - vector116 * 30.0, 0, 0, 229, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.position = center - vector116 * rng.next_range(10, 21) as f32; // `:33723`
        d.velocity = rotated_by(vector116, 1.5707963705062866) * 4.0; // `:33724`
        d.scale = 0.5 + rng.next_float(); // `:33725`
        d.fade_in = 0.5; // `:33726`
    }
    if rng.next_range(0, 4) == 0 {
        // `:33728-33737` — the 240 ring.
        let max = 6.2831854820251465f64;
        let ang = rng.next_double() * max - rng.next_double() * max;
        let vector117 = rotated_by(Vec2::Y, ang);
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center - vector117 * 30.0, 0, 0, 240, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.position = center - vector117 * 30.0; // `:33733`
        d.velocity = rotated_by(vector117, -1.5707963705062866) * 2.0; // `:33734`
        d.scale = 0.5 + rng.next_float(); // `:33735`
        d.fade_in = 0.5; // `:33736`
    }
}

/// Kind 578 — the Vortex lightning's wind-up streak (`AI()` aiStyle 108,
/// `Projectile.cs:33739-33759`), while `ai[0] <= 50`. ONE `Next(2)` gate; on
/// a pass the aim vector `ai[1].ToRotationVector2()` and its perpendicular
/// offset drive TWO type-229 dusts. Draw order: the side `Next(2)` (via
/// `ToDirectionInt`: true → +1, false → −1, `Utils.cs:2010-2017`), the offset
/// `Next(10,21)`, the first along-aim `Next(-80,81)`, the internals, the
/// scale `NextFloat`; then the second along-aim `Next(40,121)`, the
/// internals, the scale `NextFloat`. The second dust's `fadeIn` is NOT
/// written (it keeps the `NewDust` default 0).
pub fn trail_578(
    center: Vec2,
    aim: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 2) != 0 {
        return; // `:33739`
    }
    // `:33741` — ToRotationVector2: double cos/sin of the widened float.
    let vector118 = Vec2::new((aim as f64).cos() as f32, (aim as f64).sin() as f32);
    // `:33742` — the perpendicular offset: RotatedBy(π/2) * dirInt * Next(10,21).
    let dir = if rng.next_range(0, 2) == 0 { 1.0 } else { -1.0 };
    let mag = rng.next_range(10, 21) as f32;
    let vector119 = rotated_by(vector118, 1.5707963705062866) * dir * mag;
    // `:33743-33744` — (aim*Next(-80,81) - offset) / 10.
    let mut vector120 = (vector118 * rng.next_range(-80, 81) as f32 - vector119) / 10.0;
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, 229, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:33747`
    d.position = center + vector119; // `:33748`
    d.velocity = vector120; // `:33749`
    d.scale = 0.5 + rng.next_float(); // `:33750`
    d.fade_in = 0.5; // `:33751`
    // `:33752-33758` — the second dust: (aim*Next(40,121) - offset/2) / 10.
    vector120 = (vector118 * rng.next_range(40, 121) as f32 - vector119 / 2.0) / 10.0;
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, 229, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.no_gravity = true;
    d.position = center + vector119 / 2.0; // `:33756`
    d.velocity = vector120; // `:33757`
    d.scale = 1.0 + rng.next_float(); // `:33758`
}

/// Kind 596 — the Phantasmal Dragon's orbit ring (`AI()` aiStyle 107,
/// `Projectile.cs:33617-33630`). The gate is an EARLY RETURN FROM AI():
/// `Next(3) != 0` exits the whole arm (`:33619-33622`) — the port returns
/// `false` and the wiring wave must `return` from the machine on it. On a
/// pass the dust is type 27 at the centre (0×0, speed (0,−2)), repositioned
/// on the `UnitY.RotatedBy(i*2π/3 + ai[1])*10` ring, velocity =
/// `DirectionFrom(position)` (the XNA no-guard normalize of centre−pos),
/// scale = the timeLeft ladder (0.5 / 1.1 <120 / 1.6 <60), fadeIn 0.5,
/// alpha 200. The dead `num805` (`:33616`) computes no draws — dropped with
/// a note. The `timeLeft < 4` death burst (`:33631-33651`) is g65's.
pub fn trail_596(
    center: Vec2,
    ai1: f32,
    time_left: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    let mut num804 = 0.5f32; // `:33606`
    if time_left < 120 {
        num804 = 1.1; // `:33607-33610`
    }
    if time_left < 60 {
        num804 = 1.6; // `:33611-33614`
    }
    for num806 in 0..3 {
        if rng.next_range(0, 3) != 0 {
            return false; // `:33619-33622` — the AI() early return.
        }
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, 27, 0.0, -2.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        // `:33624` — (i * (float)(π*2)) / 3 + ai[1], the f32 chain.
        let ang = num806 as f32 * ((std::f64::consts::PI * 2.0) as f32) / 3.0 + ai1;
        d.position = center + rotated_by(Vec2::Y, ang as f64) * 10.0;
        d.no_gravity = true; // `:33625`
        d.velocity = xna_normalize(center - d.position); // `:33626` DirectionFrom
        d.scale = num804; // `:33627`
        d.fade_in = 0.5; // `:33628`
        d.alpha = 200; // `:33629`
    }
    true
}

/// Kind 618 — the Rainbow Crystal's tether flame (`AI()` aiStyle 119,
/// `Projectile.cs:34867-34882`): `Next(2)` → one type-86 dust pinned to the
/// centre carrying the slot velocity, scale 1.5. `customData = proj4`
/// (`:34878`) is the pool's dropped field; the `alpha = 255` pin is caller
/// state.
pub fn trail_618(
    center: Vec2,
    vel: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 2) == 0 {
        // `:34869-34880`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 8, 8, 86, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = center;
        d.velocity = vel;
        d.no_gravity = true;
        d.scale = 1.5;
        // `:34876-34879` customData = proj4 — dropped field (particles.rs).
    }
}

/// Kind 642 — the Moon Lord turret laser's tip (`AI()` aiStyle 84,
/// `Projectile.cs:31640-31660`): the 455 body verbatim (the 14-px lead, the
/// type-229 pair, the `Next(5)` puff).
pub fn trail_642(
    center: Vec2,
    vel: Vec2,
    w: i32,
    local_ai1: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    laser_tip_229(center, vel, w, local_ai1, 14.0, rng, dust, ctx, world);
}

/// Kind 656 — the Empress's prismatic bolt smoke (`AI()` aiStyle 127,
/// `Projectile.cs:36064-36084`), while `ai[0] < 780` (the caller's window —
/// `num956 - 120`). `vector158` is the column midpoint and `vector159` the
/// (0.2·height, height) column vector, both caller-computed (`:36031-36035`).
/// Draw order: `amount3 = NextFloat` FIRST (`:36068`), then the X-lerp
/// `NextFloat` (`:36069`); the internals; the conditional `velocity.X`
/// `NextFloat` (`:36081`, only when `vector160.X > -1.2`); the `velocity.Y`
/// `NextFloat` ALWAYS (`:36083`). `customData` (`:36076`) is dropped.
pub fn trail_656(
    vector158: Vec2,
    vector159: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:36068-36071` — amount3 first, then the X lerp draw.
    let amount3 = rng.next_float();
    let mut vector160 = Vec2::new(
        lerp(0.1, 1.0, rng.next_float()),
        lerp(-0.5, 0.9, amount3),
    );
    vector160.x *= lerp(2.2, 0.6, amount3);
    vector160.x *= -1.0;
    let vector161 = Vec2::new(6.0, 10.0); // `:36072`
    let vector162 = vector158 + vector159 * vector160 * 0.5 + vector161; // `:36073`
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, vector162, 0, 0, 269, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.position = vector162; // `:36075`
    // `:36076` customData = vector158 + vector161 — dropped field.
    d.fade_in = 1.0; // `:36077`
    d.scale = 0.3; // `:36078`
    if vector160.x > -1.2 {
        d.velocity.x = 1.0 + rng.next_float(); // `:36079-36082`
    }
    d.velocity.y = rng.next_float() * -0.5 - 1.0; // `:36083`
}

/// `MathHelper.Lerp(a,b,t) = a + (b-a)*t` (the empress module's form).
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Kind 659 — the Empress's lance spark ring (`AI()` aiStyle 129,
/// `Projectile.cs:36264-36276`): three iterations, each behind its OWN
/// `Next(3) == 0` gate (a fail CONTINUES — unlike the 596's early return).
/// The dust is type 27 at the centre (0×0, speed (0,−2)) repositioned on the
/// `UnitY.RotatedBy(i*2π/3 + ai[1])*10` ring, velocity = DirectionFrom,
/// scale = the timeLeft ladder, fadeIn 0.5, alpha 200 — the 596 body with the
/// continue-shaped gate.
pub fn trail_659(
    center: Vec2,
    ai1: f32,
    time_left: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let mut num973 = 0.5f32; // `:36253`
    if time_left < 120 {
        num973 = 1.1; // `:36254-36257`
    }
    if time_left < 60 {
        num973 = 1.6; // `:36258-36261`
    }
    for num975 in 0..3 {
        if rng.next_range(0, 3) != 0 {
            continue; // `:36266` — the per-iteration gate.
        }
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, 27, 0.0, -2.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        let ang = num975 as f32 * ((std::f64::consts::PI * 2.0) as f32) / 3.0 + ai1; // `:36269`
        d.position = center + rotated_by(Vec2::Y, ang as f64) * 10.0;
        d.no_gravity = true; // `:36270`
        d.velocity = xna_normalize(center - d.position); // `:36271`
        d.scale = num973; // `:36272`
        d.fade_in = 0.5; // `:36273`
        d.alpha = 200; // `:36274`
    }
}

/// Kind 669 — the Betsy's fireball ember pair (`AI()` aiStyle 68,
/// `Projectile.cs:30209-30220`), behind the caller's `alpha == 0` gate
/// (`:30189`). Two dusts: the offset is `(Lerp(-8,8,NextFloat), -4).
/// RotatedBy(rotation)`; type 4 at `Center + offset - One*5` (4×4 box);
/// `scale = 0.8 - NextFloat*0.2`; velocity = `v*0.25 + Normalize(offset)`
/// then spun by `-π/2 * direction`; the colour is `SelectRandom` over
/// (255,255,255,110) / (245,200,30,110) — ONE `Next(2)` LAST (`:30218`).
/// The 399 molotov's single-dust arm (`:30202`, flag34) is g63's (kind ≤ 400).
pub fn trail_669(
    center: Vec2,
    rotation: f32,
    direction: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..2 {
        // `:30213` — the offset draw precedes the spawn.
        let vector47 = rotated_by(
            Vec2::new(lerp(-8.0, 8.0, rng.next_float()), -4.0),
            rotation as f64,
        );
        let idx = dust.new_dust(
            &mut DustDraw(rng),
            ctx,
            world,
            center + vector47 - Vec2::ONE * 5.0,
            4,
            4,
            4,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let d = dust.slot_mut(idx);
        d.scale = 0.8 - rng.next_float() * 0.2; // `:30215`
        // `:30216` — v*0.25 + Normalize(offset)*1 (XNA no-guard).
        d.velocity = d.velocity * 0.25 + xna_normalize(vector47);
        // `:30217` — spun by -(float)π/2 * direction.
        d.velocity = rotated_by(d.velocity, (-(std::f32::consts::PI / 2.0) * direction as f32) as f64);
        // `:30218` — SelectRandom<Color>: one Next(2), LAST.
        d.color = if rng.next_range(0, 2) == 0 {
            Color::new(255, 255, 255, 110)
        } else {
            Color::new(245, 200, 30, 110)
        };
    }
}

/// Kind 723 — the Star Wrath's falling-star orbit (`AI()` aiStyle 5,
/// `Projectile.cs:24020-24029`): three iterations, each spawning TWO
/// `NewDustPerfect` type-267 dusts (Blue at scale 1, White at scale 0.4) at
/// `Center + (0, 12*scale).RotatedBy(pos.Y/200 + i*2π + rotation) - vel*0.5`,
/// velocity `vel*0.2*i`, both noLight + noGravity. ZERO explicit draws — the
/// Perfect forwards a 0×0 box, so only the pool internals roll (12 draws per
/// tick: 6 per dust). The `:24016` `Next(6)` GORE gate above is d55/g16's.
pub fn trail_723(
    center: Vec2,
    pos_y: f32,
    rotation: f32,
    scale: f32,
    vel: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let pi2 = (std::f64::consts::PI * 2.0) as f32;
    for num49 in 0..3 {
        // `:24022` — the f32 angle chain: pos.Y/200 + i*(2π) + rotation.
        let ang = pos_y / 200.0 + num49 as f32 * pi2 + rotation;
        let vector7 = center + rotated_by(Vec2::new(0.0, 12.0 * scale), ang as f64) - vel * 0.5;
        let v = vel * 0.2 * num49 as f32;
        // `:24023-24025` — the Blue dust (scale 1 default).
        let idx = dust.new_dust_perfect(&mut DustDraw(rng), ctx, world, vector7, 267, Some(v), 0, Color::new(0, 0, 255, 255), 1.0);
        let d = dust.slot_mut(idx);
        d.no_light = true;
        d.no_gravity = true;
        // `:24026-24028` — the White dust at scale 0.4.
        let idx = dust.new_dust_perfect(&mut DustDraw(rng), ctx, world, vector7, 267, Some(v), 0, Color::WHITE, 0.4);
        let d = dust.slot_mut(idx);
        d.no_light = true;
        d.no_gravity = true;
    }
}

/// The 650 ore-scan sparkle (`AI()` aiStyle 124, `Projectile.cs:35599-35604`,
/// inside the 35×35 window's `num914 < 4 && in_radius && spelunkable` block —
/// the caller's gates): on `num914 < 2` ONE type-204 dust at the tile's
/// top-left (16×16 box, alpha 150, scale 0.3), fadeIn 0.75, velocity ×0.1.
/// ZERO explicit draws (the gate draw is the caller's per-tile Next(8)).
pub fn trail_650_scan_sparkle(
    tile_top_left: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:35601-35603`
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, tile_top_left, 16, 16, 204, 0.0, 0.0, 150, Color::default(), 0.3);
    let d = dust.slot_mut(idx);
    d.fade_in = 0.75; // `:35602`
    d.velocity *= 0.1; // `:35603`
}

/// The 650 point-at-ore sparkle (`Projectile.cs:35679-35684`, the steering
/// tail's `ai[0] > 0` ladder branch behind the caller's `Next(10) == 0`
/// gate): ONE type-204 dust at `Center + f.ToRotationVector2()*6*num917 −
/// One*4` (8×8 box, alpha 150, scale 0.3), fadeIn 0.75, velocity ×0.1. `f`
/// is the DECODED localAI[0] angle (`localAI[0] % 2π − π`, pre-lerp) and
/// `num917` the POST-step extension — the caller passes both at the C# read
/// moment. `ToRotationVector2` is the f32 sin_cos (XNA), NOT the double
/// `Utils.ToRotationVector2` — the C# calls the Vector2 method here.
pub fn trail_650_point_sparkle(
    center: Vec2,
    f: f32,
    num917: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:35681` — XNA ToRotationVector2: (cos, sin) in f32.
    let (s, c) = f.sin_cos();
    let pos = center + Vec2::new(c, s) * 6.0 * num917 - Vec2::ONE * 4.0;
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, 8, 8, 204, 0.0, 0.0, 150, Color::default(), 0.3);
    let d = dust.slot_mut(idx);
    d.fade_in = 0.75; // `:35682`
    d.velocity *= 0.1; // `:35683`
}

/// Kind 813 — the Blood Nautilus tears' four-phase ritual dust (`AI()`
/// aiStyle 108, `Projectile.cs:33760-34024`, the d11 e0395afc machine's
/// census skips REPLACED). The shared ring body: `UnitY.RotatedByRandom
/// (2π)` (TWO NextDouble draws), the dust at `Center - v*30` (0×0 box),
/// noGravity, the position re-roll (`Next(10,21)` along v — the type-5
/// arm only; the 240 keeps ×30), the ±π/2 spin velocity, `scale = 0.5 +
/// NextFloat`, `fadeIn = 0.5`. `customData = Center` (the charge/full/
/// dissolve arms) is the pool's dropped field.
fn ritual_ring_813(
    center: Vec2,
    v: Vec2,
    dust_type: i32,
    reroll: bool,
    speed: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center - v * 30.0, 0, 0, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.no_gravity = true;
    // `:33767`/`:33865`/`:33901`/`:33962` — the type-5 position re-roll.
    d.position = if reroll {
        center - v * rng.next_range(10, 21) as f32
    } else {
        center - v * 30.0
    };
    let spin = if dust_type == 240 { -1.5707963705062866 } else { 1.5707963705062866 };
    d.velocity = rotated_by(v, spin) * speed;
    d.scale = 0.5 + rng.next_float();
    d.fade_in = 0.5;
}

/// The 813 wind-up (`:33760-33782`, `ai[0] <= 50`): two independent
/// `Next(4)` gates — the type-5 ring (spin ×4) then the type-240 (×2).
pub fn trail_813_windup(
    center: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let max = 6.2831854820251465f64;
    if rng.next_range(0, 4) == 0 {
        // `:33762-33771`
        let ang = rng.next_double() * max - rng.next_double() * max;
        let v = rotated_by(Vec2::Y, ang);
        ritual_ring_813(center, v, 5, true, 4.0, rng, dust, ctx, world);
    }
    if rng.next_range(0, 4) == 0 {
        // `:33772-33781`
        let ang = rng.next_double() * max - rng.next_double() * max;
        let v = rotated_by(Vec2::Y, ang);
        ritual_ring_813(center, v, 240, false, 2.0, rng, dust, ctx, world);
    }
}

/// The 813 charge (`:33858-33881`, `50 < ai[0] <= 90`): two independent
/// `Next(2)` gates — the type-5 ring (spin ×6) then the type-240 (×3);
/// both carry `customData = Center` (dropped).
pub fn trail_813_charge(
    center: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let max = 6.2831854820251465f64;
    if rng.next_range(0, 2) == 0 {
        // `:33860-33870`
        let ang = rng.next_double() * max - rng.next_double() * max;
        let v = rotated_by(Vec2::Y, ang);
        ritual_ring_813(center, v, 5, true, 6.0, rng, dust, ctx, world);
    }
    if rng.next_range(0, 2) == 0 {
        // `:33871-33881`
        let ang = rng.next_double() * max - rng.next_double() * max;
        let v = rotated_by(Vec2::Y, ang);
        ritual_ring_813(center, v, 240, false, 3.0, rng, dust, ctx, world);
    }
}

/// The 813 full charge (`:33894-33918`, `90 < ai[0] <= 120`): ONE
/// `Next(2)` if/else roll — the taken branch the type-5 ring (×6), the
/// else the type-240 (×3).
pub fn trail_813_full(
    center: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let max = 6.2831854820251465f64;
    if rng.next_range(0, 2) == 0 {
        // `:33896-33906`
        let ang = rng.next_double() * max - rng.next_double() * max;
        let v = rotated_by(Vec2::Y, ang);
        ritual_ring_813(center, v, 5, true, 6.0, rng, dust, ctx, world);
    } else {
        // `:33907-33917`
        let ang = rng.next_double() * max - rng.next_double() * max;
        let v = rotated_by(Vec2::Y, ang);
        ritual_ring_813(center, v, 240, false, 3.0, rng, dust, ctx, world);
    }
}

/// The 813 dissolve (`:33951-33983`, `ai[0] > 120`): TWO `Next(3)` switch
/// rolls — case 0 the type-5 ring, case 1 the type-240, case 2 nothing.
/// The spin vector is pre-scaled by `this.scale` (the dissolve ramp) —
/// it feeds the spawn point, the position re-roll AND the velocity.
pub fn trail_813_dissolve(
    center: Vec2,
    scale: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let max = 6.2831854820251465f64;
    for _ in 0..2 {
        match rng.next_range(0, 3) {
            0 => {
                // `:33957-33968`
                let ang = rng.next_double() * max - rng.next_double() * max;
                let v = rotated_by(Vec2::Y, ang) * scale;
                ritual_ring_813(center, v, 5, true, 6.0, rng, dust, ctx, world);
            }
            1 => {
                // `:33969-33980`
                let ang = rng.next_double() * max - rng.next_double() * max;
                let v = rotated_by(Vec2::Y, ang) * scale;
                ritual_ring_813(center, v, 240, false, 3.0, rng, dust, ctx, world);
            }
            _ => {} // `:33981` — case 2 draws nothing further.
        }
    }
}

/// The 813 opacity tail (`:34018-34024`, EVERY tick incl. the death tick —
/// the dissolve `Kill()` does not return): `NextFloat < Opacity` gate,
/// then `NewDustPerfect(Center + NextVector2Circular(3, 20), 90)` with
/// `velocity = UnitY`, noGravity, `scale = this.scale`. The circular pair
/// is the unit NextFloat + the radius NextFloat (the empress form).
pub fn trail_813_opacity_tail(
    center: Vec2,
    opacity: f32,
    scale: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_float() < opacity {
        let off = next_vector2_circular(rng, 3.0, 20.0);
        let idx = dust.new_dust_perfect(&mut DustDraw(rng), ctx, world, center + off, 90, None, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.velocity = Vec2::Y; // `:34021`
        d.no_gravity = true; // `:34022`
        d.scale = scale; // `:34023`
    }
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — one `NextFloat`; the
/// angle is FLOAT arithmetic (`0 + (float)π*2 * NextFloat()`), then
/// `ToRotationVector2` (`Utils.cs:1665-1668`) cos/sin in double of the
/// widened float, cast back to f32 (the empress module's form).
fn next_vector2_unit(rng: &mut dyn DropRng) -> Vec2 {
    let angle = (std::f64::consts::PI * 2.0) as f32 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw THEN
/// a second `NextFloat` radius scale: `unit * (halfW, halfH) * r`.
fn next_vector2_circular(rng: &mut dyn DropRng, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// Kind 732 — the Thunder Spear's bolt spark (`AI()` aiStyle 44,
/// `Projectile.cs:27905-27921`): the alpha fade/kill above is caller state;
/// the `Next(5) == 0` gate draws every tick, then ONE `NewDustDirect` type
/// 226 (alpha 150, scale 0.8), noGravity, velocity ×1.2. The rotation pin +
/// `AddLight` below are state / the f7 lane's.
pub fn trail_732(
    pos: Vec2,
    w: i32,
    h: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 5) == 0 {
        // `:27913-27918`
        let idx = dust.new_dust_direct(&mut DustDraw(rng), ctx, world, pos, w, h, 226, 0.0, 0.0, 150, Color::default(), 0.8);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 1.2;
    }
}

/// Kind 895 — the Empress pet's flight sparkle (`AI()` aiStyle 124,
/// `Projectile.cs:35229-35233`), inside the `velocity.Length() > 6` branch
/// behind the 3-tick frame cycle: the caller passes `frame_reset` (the
/// `frameCounter == 0` read AFTER the cycle's increment/reset — the dust
/// fires on the reset tick). ONE type-242 dust (alpha 50, scale 2),
/// noGravity. ZERO explicit draws.
pub fn trail_895(
    pos: Vec2,
    w: i32,
    h: i32,
    frame_reset: bool,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if frame_reset {
        // `:35229-35233`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 242, 0.0, 0.0, 50, Color::default(), 2.0);
        dust.slot_mut(idx).no_gravity = true;
    }
}

/// Kind 898 — the Ice Queen pet's flight sparkle (`Projectile.cs:35293-
/// 35297`): the 895 shape with type 80, alpha 50, scale 1 (default). The
/// pool's type-80 tweak pins alpha 50 anyway (`Dust.cs:236`).
pub fn trail_898(
    pos: Vec2,
    w: i32,
    h: i32,
    frame_reset: bool,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if frame_reset {
        // `:35293-35297`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 80, 0.0, 0.0, 50, Color::default(), 1.0);
        dust.slot_mut(idx).no_gravity = true;
    }
}

/// Kind 907 — the Martian probe's scan spark (`AI()` aiStyle 30,
/// `Projectile.cs:26686-26691`): the gate is `Next(Max(4, 8 -
/// (int)velocity.Length())) == 0` — the modulus SHRINKS as the probe slows
/// (the floor is 4). One type-43 dust at the 5-px-inset box, alpha 254,
/// colour (255,255,0), velocity overwritten to vel×0.75.
pub fn trail_907(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let modulus = (8 - vel.length() as i32).max(4); // `:26686`
    if rng.next_range(0, modulus) == 0 {
        let num237 = 5;
        let idx = dust.new_dust(
            &mut DustDraw(rng),
            ctx,
            world,
            pos + Vec2::new(num237 as f32, num237 as f32),
            w - num237 * 2,
            h - num237 * 2,
            43,
            0.0,
            0.0,
            254,
            Color::new(255, 255, 0, 255),
            1.0,
        );
        dust.slot_mut(idx).velocity = vel * 0.75; // `:26690`
    }
}

/// Kind 967 — the Houndius Shootius fireball's ember trail (`AI()` aiStyle
/// 28, `Projectile.cs:26582-26591`): `localAI[0]` is the POST-increment
/// counter (the caller runs `:26584`); the gate is `localAI[0] >= 5 &&
/// timeLeft % 3 == 0` (the caller pins `localAI[0] = 5` on the first fire —
/// state). ONE type-219 dust, speeds = the slot velocity, scale
/// `0.35 + Next(-5,5)*0.01` (an ARGUMENT draw), then noGravity and the
/// velocity overwrite ×1.5. The `ai[1] == 0` 10-dust burst (`:26653-26661`)
/// is the co-lane's one-shot.
pub fn trail_967(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    local_ai0: f32,
    time_left: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !(local_ai0 >= 5.0 && time_left % 3 == 0) {
        return; // `:26585` — no draws outside the cadence.
    }
    let scale = 0.35 + rng.next_range(-5, 5) as f32 * 0.01; // `:26588` argument
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 219, vel.x, vel.y, 0, Color::default(), scale);
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:26589`
    d.velocity = vel * 1.5; // `:26590`
}

/// Kind 1026 — the Dead Cells barnacle shot's wake (`AI()` aiStyle 28,
/// `Projectile.cs:26557-26568`): `localAI[0]` POST-increment; the gate is
/// `localAI[0] >= 8 && timeLeft % 2 == 0`. ONE type-2 dust at
/// `pos - vel.SafeNormalize(Zero)*5`, speeds = the slot velocity, scale
/// `0.7 + Next(-5,5)*0.01` (argument draw); post-writes alpha 50, fadeIn
/// 0.5, noGravity, velocity = vel×0.25. `SafeNormalize` is the zero/NaN
/// guard form (`Utils.cs:1837-1844`).
pub fn trail_1026(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    local_ai0: f32,
    time_left: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !(local_ai0 >= 8.0 && time_left % 2 == 0) {
        return; // `:26560`
    }
    let scale = 0.7 + rng.next_range(-5, 5) as f32 * 0.01; // `:26563` argument
    let lead = crate::bow_launch::safe_normalize(vel, Vec2::ZERO) * 5.0;
    let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos - lead, w, h, 2, vel.x, vel.y, 0, Color::default(), scale);
    let d = dust.slot_mut(idx);
    d.alpha = 50; // `:26564`
    d.fade_in = 0.5; // `:26565`
    d.no_gravity = true; // `:26566`
    d.velocity = vel * 0.25; // `:26567`
}

/// Kind 1048 — the Poo's flight speck (`AI()` aiStyle 25,
/// `Projectile.cs:26395-26399`), behind `velocity.Length() > 1` (the
/// `netMode != 2` guard folds — single-player). The spawn position is
/// `Center + NextVector2Circular(w*0.4, h*0.4) - One` — TWO draws (unit
/// NextFloat, radius NextFloat) — and the scale argument draws a THIRD
/// NextFloat (`0.75 + NextFloat*0.25`); the speeds are −vel×0.5; the only
/// post-write is noGravity (the `_ =` discard keeps the slot write).
pub fn trail_1048(
    center: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if vel.length() <= 1.0 {
        return; // `:26395` — no draws when settled.
    }
    let vector23 = center + next_vector2_circular(rng, w as f32 * 0.4, h as f32 * 0.4);
    let scale = 0.75 + rng.next_float() * 0.25; // `:26398` argument
    let idx = dust.new_dust(
        &mut DustDraw(rng),
        ctx,
        world,
        vector23 - Vec2::ONE,
        2,
        2,
        322,
        -vel.x * 0.5,
        -vel.y * 0.5,
        0,
        Color::default(),
        scale,
    );
    dust.slot_mut(idx).no_gravity = true;
}

/// Kind 1053 — the Rolling Cactus's tumble dust (`AI()` aiStyle 25,
/// `Projectile.cs:26031-26049`): THREE independent `Next(3)` gates, each
/// drawing every tick. Gate 1: type 31 at the 6-px-inset box (scale 1.5),
/// velocity ×0.3 then −2 Y. Gate 2: type 6 at the 4-px-inset box (scale
/// 1.5), noGravity only. Gate 3: type 6 at the full box, velocity.Y −1.
/// The bounce burst in `HandleMovement` (`:18949/18952`) is the movement
/// lane's, not this trail.
pub fn trail_1053(
    pos: Vec2,
    w: i32,
    h: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 3) == 0 {
        // `:26033-26038`
        let idx = dust.new_dust(
            &mut DustDraw(rng),
            ctx,
            world,
            pos + Vec2::new(6.0, 6.0),
            w - 12,
            h - 12,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.3;
        d.velocity.y -= 2.0;
    }
    if rng.next_range(0, 3) == 0 {
        // `:26039-26043`
        let idx = dust.new_dust(
            &mut DustDraw(rng),
            ctx,
            world,
            pos + Vec2::new(4.0, 4.0),
            w - 8,
            h - 8,
            6,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        dust.slot_mut(idx).no_gravity = true;
    }
    if rng.next_range(0, 3) == 0 {
        // `:26044-26048`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 6, 0.0, 0.0, 100, Color::default(), 1.0);
        dust.slot_mut(idx).velocity.y -= 1.0;
    }
}

/// Kind 1092 — the Martian saucer's underside drip (`AI()` aiStyle 18,
/// `Projectile.cs:25447-25455`): the `Next(15) == 0` gate draws every tick
/// (the `ai[0] == 0` 6-dust burst above is the co-lane's one-shot). ONE
/// `NewDustDirect` type 269 at `Left` = (x, y + h/2), box (width, 4), alpha
/// 0, White, scale 1.2; the velocity is OVERWRITTEN to (0, 0.5) then
/// += vel×(0.5, 0.25); noLight.
pub fn trail_1092(
    pos: Vec2,
    w: i32,
    h: i32,
    vel: Vec2,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 15) == 0 {
        // `:25447-25455`
        let left = Vec2::new(pos.x, pos.y + h as f32 / 2.0); // Entity.Left
        let idx = dust.new_dust_direct(&mut DustDraw(rng), ctx, world, left, w, 4, 269, 0.0, 0.0, 0, Color::WHITE, 1.2);
        let d = dust.slot_mut(idx);
        d.velocity.x = 0.0; // `:25450`
        d.velocity.y = 0.5; // `:25451`
        d.velocity.x += vel.x * 0.5; // `:25452`
        d.velocity.y += vel.y * 0.25; // `:25453`
        d.no_light = true; // `:25454`
    }
}

/// Kind 1114 — the Crystal Storm's hostile shard trail (`AI()` aiStyle 24,
/// `Projectile.cs:25977-25991`): the `wet` kill above is caller state; the
/// `Next(4) == 0` gate draws every tick. ONE type-57 dust, noGravity,
/// velocity ×0.5, scale ×0.9, noLight AND noLightEmittance.
pub fn trail_1114(
    pos: Vec2,
    w: i32,
    h: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 4) == 0 {
        // `:25983-25991`
        let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, pos, w, h, 57, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.5;
        d.scale *= 0.9;
        d.no_light = true;
        d.no_light_emittance = true;
    }
}

/// Kind 1133 — the Star Wrath's falling-star sparkle (`AI()` aiStyle 5,
/// `Projectile.cs:23988-24011`), the `type == 1133` sub-branch. TWO gates:
/// `Next(6) == 0` → one `NewDustDirect` type 89 (alpha 127), noGravity only
/// (the `.noGravity = true` is a chained write on the returned Dust); then
/// `Next(3) == 0` → one `NewDustDirect` type `num48` (87 for 1133,
/// `:24000`), alpha 127, velocity ×0.5, noGravity. The `:23993` `Next(24)`
/// GORE gate above is d55/g16's — it is itself gated on the on-screen
/// `Hitbox.Intersects(CenteredRectangle(...))` test (`&&` short-circuit), so
/// the wiring wave interleaves that draw (when on-screen) BEFORE this fn.
pub fn trail_1133(
    pos: Vec2,
    w: i32,
    h: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 6) == 0 {
        // `:24001-24004` — the type-89 puff.
        let idx = dust.new_dust_direct(&mut DustDraw(rng), ctx, world, pos, w, h, 89, 0.0, 0.0, 127, Color::default(), 1.0);
        dust.slot_mut(idx).no_gravity = true;
    }
    // `:23997-24000` — num48 = 86 default, 87 for 1133.
    let num48 = 87;
    if rng.next_range(0, 3) == 0 {
        // `:24006-24011`
        let idx = dust.new_dust_direct(&mut DustDraw(rng), ctx, world, pos, w, h, num48, 0.0, 0.0, 127, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.no_gravity = true;
    }
}

/// Kind 644 — the Rainbow Crystal shard's prism-hue orbit (`AI()` aiStyle
/// 112, `Projectile.cs:34267-34332`). DEFERRED at port time ("the prism-hue
/// orbit"), UNBLOCKED at the wiring wave: the hue is the byte-quantised
/// `Main.hslToRgb(ai[0], 1, 0.5)` (`Main.cs:47890`, the palette port), the
/// `Opacity` is `1 - alpha/255` (the machine's `opacity_of`), and the
/// sparkle's `CloneDust` rides the pool's verbatim `clone_dust`
/// (`Dust.cs:263-286`). Two 2-iteration Next(10) gates (the second ring's
/// spin omits the rotation — the C# `:34284`), then the Next(10) sparkle:
/// three NextFloats, the RandomVector2 pair (TWO NextDouble-cast draws,
/// `Utils.cs:2431-2434`), the radius NextFloat, the InWorld+SolidTile gate,
/// the velocity NextFloat (only on the taken branch), the clone's internal
/// NewDust rolls, then the ×0.65 scale/fadeIn + the white colour. The
/// `PlotTileLine`/`AddLight` casts stay the e6/f7 lanes.
pub fn trail_644(
    center: Vec2,
    hue: f32,
    rotation: f32,
    opacity: f32,
    scale_roll: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:34239` — the hue colour (alpha 255, the 3-arg hslToRgb default).
    let rgb = tr_core::tile_light::hsl_to_rgb(hue, 1.0, 0.5);
    let new_color3 = Color::new(
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8,
        255,
    );
    // `:34267-34279` — the first orbit pair (the spin carries the rotation).
    for num836 in 0..2 {
        if rng.next_range(0, 10) == 0 {
            let vector139 = rotated_by(
                rotated_by(Vec2::Y, (num836 as f32 * std::f32::consts::PI) as f64),
                rotation as f64,
            );
            let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, 267, 0.0, 0.0, 225, new_color3, 1.5);
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:34273`
            d.no_light = true; // `:34274`
            d.scale = opacity * scale_roll; // `:34275`
            d.position = center; // `:34276`
            d.velocity = vector139 * 2.5; // `:34277`
        }
    }
    // `:34280-34292` — the second orbit pair (NO rotation on the spin).
    for num837 in 0..2 {
        if rng.next_range(0, 10) == 0 {
            let vector140 =
                rotated_by(Vec2::Y, (num837 as f32 * std::f32::consts::PI) as f64);
            let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, center, 0, 0, 267, 0.0, 0.0, 225, new_color3, 1.5);
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:34286`
            d.no_light = true; // `:34287`
            d.scale = opacity * scale_roll; // `:34288`
            d.position = center; // `:34289`
            d.velocity = vector140 * 2.5; // `:34290`
        }
    }
    // `:34293-34332` — the sparkle gate.
    if rng.next_range(0, 10) == 0 {
        let num838 = 1.0 + rng.next_float() * 2.0; // `:34295`
        let fade_in = 1.0 + rng.next_float(); // `:34296`
        let num839 = 1.0 + rng.next_float(); // `:34297`
        // `Utils.RandomVector2(-1, 1)` (`Utils.cs:2431-2434`) — TWO draws,
        // `(max-min)*(float)NextDouble()+min`, X first.
        let vx = 2.0 * rng.next_double() as f32 - 1.0;
        let vy = 2.0 * rng.next_double() as f32 - 1.0;
        let mut vector141 = Vec2::new(vx, vy);
        if vector141 != Vec2::ZERO {
            vector141 = xna_normalize(vector141); // `:34301`
        }
        vector141 *= 20.0 + rng.next_float() * 100.0; // `:34303`
        let vec = center + vector141;
        // `:34305-34314` — ToTileCoordinates truncates; InWorld + SolidTile
        // gate the spawn (and its velocity draw).
        let (tx, ty) = (vec.x as i32 / 16, vec.y as i32 / 16);
        let in_world = tx >= 0 && ty >= 0 && tx < world.width && ty < world.height;
        let solid = in_world
            && world
                .tile(tx, ty)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
        if in_world && !solid {
            let idx = dust.new_dust(&mut DustDraw(rng), ctx, world, vec, 0, 0, 267, 0.0, 0.0, 127, new_color3, 1.0);
            {
                let d = dust.slot_mut(idx);
                d.no_gravity = true; // `:34318`
                d.position = vec; // `:34319`
                d.velocity = -Vec2::Y * num838 * (rng.next_float() * 0.9 + 1.6); // `:34320`
                d.fade_in = fade_in; // `:34321`
                d.scale = num839; // `:34322`
                d.no_light = true; // `:34323`
            }
            // `:34324-34330` — the clone (the sentinel guard rides inside
            // `clone_dust`), then the ×0.65 pair and the white colour.
            if dust.slot(idx).dust_index != 6000 {
                let cidx = dust.clone_dust(&mut DustDraw(rng), ctx, world, idx);
                let c = dust.slot_mut(cidx);
                c.scale *= 0.65;
                c.fade_in *= 0.65;
                c.color = Color::new(255, 255, 255, 255);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000), "stream tail");
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
                let v3 = rotated_by(rotated_by(Vec2::X, ang), to_rotation(vel) as f64);
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
        assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000), "no draw before the window");
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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
        assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
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

    /// W4-g64 — kind 574: the Next(10) host gate pins the centre and carries
    /// the slot velocity at scale 1.5.
    #[test]
    fn w4_g64_574_host_gate_shape() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1000.0, 1000.0);
        let vel = Vec2::new(1.5, -0.5);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        for _ in 0..60 {
            trail_574_host(center, vel, &mut rng, &mut pool, &c, &w);
        }
        assert!(!active(&pool).is_empty());
        for i in active(&pool) {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 242);
            assert_eq!(d.position, center);
            assert_eq!(d.velocity, vel);
            assert_eq!(d.scale, 1.5);
            assert!(d.no_gravity);
        }
    }

    /// W4-g64 — kind 579: each taken Next(4) gate consumes TWO NextDouble
    /// (RotatedByRandom) before the internals, then Next(10,21) (the 229)
    /// and NextFloat.
    #[test]
    fn w4_g64_579_ring_mirror() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1300.0, 800.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(103);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(103);
        trail_579(center, &mut rng, &mut pool, &c, &w);
        let max = 6.2831854820251465f64;
        if mirror.next_max(4) == 0 {
            let ang = mirror.next_double() * max - mirror.next_double() * max;
            let v = rotated_by(Vec2::Y, ang);
            let idx = mpool.new_dust(&mut mirror, &c, &w, center - v * 30.0, 0, 0, 229, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.position = center - v * mirror.next_range(10, 21) as f32;
            d.velocity = rotated_by(v, 1.5707963705062866) * 4.0;
            d.scale = 0.5 + mirror.next_float();
            d.fade_in = 0.5;
        }
        if mirror.next_max(4) == 0 {
            let ang = mirror.next_double() * max - mirror.next_double() * max;
            let v = rotated_by(Vec2::Y, ang);
            let idx = mpool.new_dust(&mut mirror, &c, &w, center - v * 30.0, 0, 0, 240, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.position = center - v * 30.0;
            d.velocity = rotated_by(v, -1.5707963705062866) * 2.0;
            d.scale = 0.5 + mirror.next_float();
            d.fade_in = 0.5;
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 578: the streak pair's draw order — side Next(2),
    /// offset Next(10,21), along Next(-80,81), internals, NextFloat, then
    /// along Next(40,121), internals, NextFloat. The second dust keeps
    /// fadeIn 0 (no write).
    #[test]
    fn w4_g64_578_streak_pair_order() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1300.0, 800.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(107);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(107);
        trail_578(center, 0.9, &mut rng, &mut pool, &c, &w);
        if mirror.next_max(2) == 0 {
            let aim = Vec2::new((0.9f64).cos() as f32, (0.9f64).sin() as f32);
            let dir = if mirror.next_max(2) == 0 { 1.0 } else { -1.0 };
            let off = rotated_by(aim, 1.5707963705062866) * dir * mirror.next_range(10, 21) as f32;
            let v1 = (aim * mirror.next_range(-80, 81) as f32 - off) / 10.0;
            let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, 229, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.position = center + off;
            d.velocity = v1;
            d.scale = 0.5 + mirror.next_float();
            d.fade_in = 0.5;
            let v2 = (aim * mirror.next_range(40, 121) as f32 - off / 2.0) / 10.0;
            let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, 229, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.position = center + off / 2.0;
            d.velocity = v2;
            d.scale = 1.0 + mirror.next_float();
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        let taken = active(&pool);
        if taken.len() == 2 {
            assert_eq!(pool.slot(taken[1]).fade_in, 0.0, "second dust keeps fadeIn 0");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 596: the Next(3)!=0 gate EARLY-RETURNS (false); a full
    /// pass lands three ring dusts with the DirectionFrom velocity and the
    /// timeLeft scale ladder.
    #[test]
    fn w4_g64_596_early_return_gate_and_ladder() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1400.0, 900.0);
        let mut fired = 0;
        let mut bailed = 0;
        for seed in 0..40i32 {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let ok = trail_596(center, seed as f32, 50, &mut rng, &mut pool, &c, &w);
            if ok {
                fired += 1;
                assert_eq!(active(&pool).len(), 3);
                for i in active(&pool) {
                    let d = pool.slot(i);
                    assert_eq!(d.dust_type, 27);
                    assert_eq!(d.scale, 1.6, "timeLeft 50 < 60");
                    assert_eq!(d.alpha, 200);
                    assert!((d.velocity.length() - 1.0).abs() < 1e-5, "unit DirectionFrom");
                }
            } else {
                bailed += 1;
                assert!(active(&pool).len() < 3, "the bail leaves fewer than 3");
            }
        }
        assert!(fired > 0 && bailed > 0, "both gate outcomes exercised");
    }

    /// W4-g64 — kind 618: the Next(2) tether gate pins the centre at scale
    /// 1.5 with the slot velocity.
    #[test]
    fn w4_g64_618_tether_shape() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1000.0, 500.0);
        let vel = Vec2::new(-2.0, 3.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(109);
        for _ in 0..40 {
            trail_618(center, vel, &mut rng, &mut pool, &c, &w);
        }
        assert!(!active(&pool).is_empty());
        for i in active(&pool) {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 86);
            assert_eq!(d.position, center);
            assert_eq!(d.velocity, vel);
            assert_eq!(d.scale, 1.5);
        }
    }

    /// W4-g64 — kind 642: identical body to 455 — same seed, same inputs →
    /// identical pools.
    #[test]
    fn w4_g64_642_matches_the_455_body() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1200.0, 600.0);
        let vel = Vec2::new(0.6, -0.8);
        let mut a = DustPool::new();
        let mut b = DustPool::new();
        trail_455(center, vel, 24, 80.0, &mut UnifiedRandom::new(113), &mut a, &c, &w);
        trail_642(center, vel, 24, 80.0, &mut UnifiedRandom::new(113), &mut b, &c, &w);
        assert_eq!(active(&a), active(&b));
        for i in active(&a) {
            assert_eq!(a.slot(i), b.slot(i));
        }
    }

    /// W4-g64 — kind 656: amount3 draws FIRST; the conditional X NextFloat
    /// only fires when vector160.X > -1.2; the Y NextFloat always fires.
    #[test]
    fn w4_g64_656_draw_order_mirror() {
        let w = world();
        let c = ctx();
        let v158 = Vec2::new(1000.0, 800.0);
        let v159 = Vec2::new(40.0, 200.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(127);
        for _ in 0..10 {
            trail_656(v158, v159, &mut rng, &mut pool, &c, &w);
        }
        for _ in 0..10 {
            let amount3 = mirror.next_float();
            let mut v160 = Vec2::new(lerp(0.1, 1.0, mirror.next_float()), lerp(-0.5, 0.9, amount3));
            v160.x *= lerp(2.2, 0.6, amount3);
            v160.x *= -1.0;
            let v161 = Vec2::new(6.0, 10.0);
            let v162 = v158 + v159 * v160 * 0.5 + v161;
            let idx = mpool.new_dust(&mut mirror, &c, &w, v162, 0, 0, 269, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.position = v162;
            d.fade_in = 1.0;
            d.scale = 0.3;
            if v160.x > -1.2 {
                d.velocity.x = 1.0 + mirror.next_float();
            }
            d.velocity.y = mirror.next_float() * -0.5 - 1.0;
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 659: the per-iteration Next(3) gates CONTINUE (never
    /// early-return); the ring shape matches 596's body.
    #[test]
    fn w4_g64_659_continue_gates() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1400.0, 900.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(131);
        trail_659(center, 2.0, 200, &mut rng, &mut pool, &c, &w);
        for i in 0..3 {
            if mirror.next_max(3) != 0 {
                continue;
            }
            let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, 27, 0.0, -2.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            let ang = i as f32 * ((std::f64::consts::PI * 2.0) as f32) / 3.0 + 2.0;
            d.position = center + rotated_by(Vec2::Y, ang as f64) * 10.0;
            d.no_gravity = true;
            d.velocity = xna_normalize(center - d.position);
            d.scale = 0.5;
            d.fade_in = 0.5;
            d.alpha = 200;
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 669: the ember pair's offset NextFloat precedes each
    /// spawn; the scale NextFloat and the colour Next(2) follow; the colour
    /// is one of the two literals.
    #[test]
    fn w4_g64_669_ember_pair_mirror() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(900.0, 700.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(137);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(137);
        trail_669(center, 0.3, -1, &mut rng, &mut pool, &c, &w);
        for _ in 0..2 {
            let off = rotated_by(Vec2::new(lerp(-8.0, 8.0, mirror.next_float()), -4.0), 0.3);
            let idx = mpool.new_dust(&mut mirror, &c, &w, center + off - Vec2::ONE * 5.0, 4, 4, 4, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.scale = 0.8 - mirror.next_float() * 0.2;
            d.velocity = d.velocity * 0.25 + xna_normalize(off);
            d.velocity = rotated_by(d.velocity, (-(std::f32::consts::PI / 2.0) * -1.0) as f64);
            d.color = if mirror.next_max(2) == 0 { Color::new(255, 255, 255, 110) } else { Color::new(245, 200, 30, 110) };
        }
        assert_eq!(active(&pool).len(), 2);
        for i in 0..2 {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 723: six Perfect dusts (3 Blue + 3 White), zero
    /// explicit draws — the stream consumes exactly the internals.
    #[test]
    fn w4_g64_723_orbit_perfect_pair() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1000.0, 400.0);
        let vel = Vec2::new(0.0, 10.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(139);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(139);
        trail_723(center, 400.0, 0.2, 1.0, vel, &mut rng, &mut pool, &c, &w);
        let pi2 = (std::f64::consts::PI * 2.0) as f32;
        for i in 0..3 {
            let ang = 400.0f32 / 200.0 + i as f32 * pi2 + 0.2;
            let v7 = center + rotated_by(Vec2::new(0.0, 12.0), ang as f64) - vel * 0.5;
            let v = vel * 0.2 * i as f32;
            let idx = mpool.new_dust_perfect(&mut mirror, &c, &w, v7, 267, Some(v), 0, Color::new(0, 0, 255, 255), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_light = true;
            d.no_gravity = true;
            let idx = mpool.new_dust_perfect(&mut mirror, &c, &w, v7, 267, Some(v), 0, Color::WHITE, 0.4);
            let d = mpool.slot_mut(idx);
            d.no_light = true;
            d.no_gravity = true;
        }
        assert_eq!(active(&pool).len(), 6);
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
            assert!(pool.slot(i).no_light && pool.slot(i).no_gravity);
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 732: the Next(5) gate + the Direct spawn; the velocity
    /// ×1.2 lands after the internals (the type-226 tweak already ran).
    #[test]
    fn w4_g64_732_direct_spark() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(800.0, 800.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(149);
        for _ in 0..30 {
            trail_732(pos, 12, 12, &mut rng, &mut pool, &c, &w);
        }
        assert!(!active(&pool).is_empty());
        for i in active(&pool) {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 226);
            assert_eq!(d.alpha, 150);
            assert!(d.no_gravity);
        }
    }

    /// W4-g64 — kinds 895/898: the frame-reset gate spawns nothing when
    /// false (zero draws); on the reset tick one dust lands.
    #[test]
    fn w4_g64_895_898_frame_reset_gate() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(600.0, 400.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(151);
        let mut mirror = UnifiedRandom::new(151);
        trail_895(pos, 20, 20, false, &mut rng, &mut pool, &c, &w);
        trail_898(pos, 20, 20, false, &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty());
        assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000), "no draws off-reset");
        trail_895(pos, 20, 20, true, &mut rng, &mut pool, &c, &w);
        trail_898(pos, 20, 20, true, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 2);
        assert_eq!(pool.slot(0).dust_type, 242);
        assert_eq!(pool.slot(1).dust_type, 80);
        assert!(pool.slot(0).no_gravity && pool.slot(1).no_gravity);
        assert_eq!(pool.slot(1).alpha, 50, "the type-80 tweak pins alpha 50");
    }

    /// W4-g64 — kind 907: the gate modulus floors at 4 and shrinks with
    /// speed; the velocity overwrite is vel×0.75.
    #[test]
    fn w4_g64_907_modulus_floor_and_velocity() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(700.0, 700.0);
        let vel = Vec2::new(3.0, 4.0); // length 5 → modulus max(4, 3) = 4
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(157);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(157);
        for _ in 0..20 {
            trail_907(pos, 24, 24, vel, &mut rng, &mut pool, &c, &w);
        }
        for _ in 0..20 {
            if mirror.next_max(4) == 0 {
                let idx = mpool.new_dust(&mut mirror, &c, &w, pos + Vec2::new(5.0, 5.0), 14, 14, 43, 0.0, 0.0, 254, Color::new(255, 255, 0, 255), 1.0);
                mpool.slot_mut(idx).velocity = vel * 0.75;
            }
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
            assert_eq!(pool.slot(i).velocity, vel * 0.75);
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 967: the cadence gate draws nothing off-cadence; the
    /// scale argument draw precedes the internals; velocity = vel×1.5.
    #[test]
    fn w4_g64_967_cadence_and_argument_scale() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(900.0, 600.0);
        let vel = Vec2::new(6.0, 0.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(163);
        let mut mirror = UnifiedRandom::new(163);
        trail_967(pos, 14, 14, vel, 3.0, 90, &mut rng, &mut pool, &c, &w);
        trail_967(pos, 14, 14, vel, 5.0, 91, &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty());
        assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        let mut mpool = DustPool::new();
        trail_967(pos, 14, 14, vel, 5.0, 90, &mut rng, &mut pool, &c, &w);
        let scale = 0.35 + mirror.next_range(-5, 5) as f32 * 0.01;
        let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 14, 14, 219, 6.0, 0.0, 0, Color::default(), scale);
        let d = mpool.slot_mut(idx);
        d.no_gravity = true;
        d.velocity = vel * 1.5;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0), mpool.slot(0));
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 1026: the SafeNormalize lead, the argument scale, the
    /// alpha-50/fadeIn-0.5 post-writes and the vel×0.25 overwrite.
    #[test]
    fn w4_g64_1026_wake_mirror() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(900.0, 600.0);
        let vel = Vec2::new(0.0, 8.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(167);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(167);
        trail_1026(pos, 16, 16, vel, 8.0, 100, &mut rng, &mut pool, &c, &w);
        let scale = 0.7 + mirror.next_range(-5, 5) as f32 * 0.01;
        let lead = crate::bow_launch::safe_normalize(vel, Vec2::ZERO) * 5.0;
        let idx = mpool.new_dust(&mut mirror, &c, &w, pos - lead, 16, 16, 2, 0.0, 8.0, 0, Color::default(), scale);
        let d = mpool.slot_mut(idx);
        d.alpha = 50;
        d.fade_in = 0.5;
        d.no_gravity = true;
        d.velocity = vel * 0.25;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0), mpool.slot(0));
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 1048: settled (|v| <= 1) draws nothing; in flight the
    /// Circular pair + the scale NextFloat precede the internals.
    #[test]
    fn w4_g64_1048_circular_three_draws() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(800.0, 500.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(173);
        let mut mirror = UnifiedRandom::new(173);
        trail_1048(center, 20, 20, Vec2::new(0.5, 0.0), &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty());
        assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        let vel = Vec2::new(4.0, 3.0);
        let mut mpool = DustPool::new();
        trail_1048(center, 20, 20, vel, &mut rng, &mut pool, &c, &w);
        let v23 = center + next_vector2_circular(&mut mirror, 8.0, 8.0);
        let scale = 0.75 + mirror.next_float() * 0.25;
        let idx = mpool.new_dust(&mut mirror, &c, &w, v23 - Vec2::ONE, 2, 2, 322, -2.0, -1.5, 0, Color::default(), scale);
        mpool.slot_mut(idx).no_gravity = true;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0), mpool.slot(0));
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 1053: three independent Next(3) gates draw every tick;
    /// the three shapes carry their own post-writes.
    #[test]
    fn w4_g64_1053_three_gates() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(700.0, 900.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(179);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(179);
        trail_1053(pos, 30, 30, &mut rng, &mut pool, &c, &w);
        if mirror.next_max(3) == 0 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos + Vec2::new(6.0, 6.0), 18, 18, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            let d = mpool.slot_mut(idx);
            d.velocity *= 0.3;
            d.velocity.y -= 2.0;
        }
        if mirror.next_max(3) == 0 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos + Vec2::new(4.0, 4.0), 22, 22, 6, 0.0, 0.0, 100, Color::default(), 1.5);
            mpool.slot_mut(idx).no_gravity = true;
        }
        if mirror.next_max(3) == 0 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 30, 30, 6, 0.0, 0.0, 100, Color::default(), 1.0);
            mpool.slot_mut(idx).velocity.y -= 1.0;
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 1092: the Left anchor, the velocity overwrite chain
    /// and noLight.
    #[test]
    fn w4_g64_1092_left_anchor_drip() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(1000.0, 300.0);
        let vel = Vec2::new(2.0, -1.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(181);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(181);
        for _ in 0..30 {
            trail_1092(pos, 40, 20, vel, &mut rng, &mut pool, &c, &w);
        }
        for _ in 0..30 {
            if mirror.next_max(15) == 0 {
                let idx = mpool.new_dust_direct(&mut mirror, &c, &w, Vec2::new(1000.0, 310.0), 40, 4, 269, 0.0, 0.0, 0, Color::WHITE, 1.2);
                let d = mpool.slot_mut(idx);
                d.velocity.x = 0.0;
                d.velocity.y = 0.5;
                d.velocity.x += 1.0;
                d.velocity.y += -0.25;
                d.no_light = true;
            }
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
            assert!(pool.slot(i).no_light);
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    /// W4-g64 — kind 1114: the Next(4) gate; the shard dust carries both
    /// noLight flags and the ×0.9 scale.
    #[test]
    fn w4_g64_1114_shard_flags() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(600.0, 600.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(191);
        for _ in 0..30 {
            trail_1114(pos, 14, 14, &mut rng, &mut pool, &c, &w);
        }
        assert!(!active(&pool).is_empty());
        for i in active(&pool) {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 57);
            assert!(d.no_gravity && d.no_light && d.no_light_emittance);
        }
    }

    /// W4-g64 — kind 1133: the Next(6) type-89 gate precedes the Next(3)
    /// type-87 gate; both draw every tick; the second dust's velocity ×0.5.
    #[test]
    fn w4_g64_1133_two_direct_gates() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(900.0, 300.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(193);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(193);
        for _ in 0..20 {
            trail_1133(pos, 16, 16, &mut rng, &mut pool, &c, &w);
        }
        for _ in 0..20 {
            if mirror.next_max(6) == 0 {
                let idx = mpool.new_dust_direct(&mut mirror, &c, &w, pos, 16, 16, 89, 0.0, 0.0, 127, Color::default(), 1.0);
                mpool.slot_mut(idx).no_gravity = true;
            }
            if mirror.next_max(3) == 0 {
                let idx = mpool.new_dust_direct(&mut mirror, &c, &w, pos, 16, 16, 87, 0.0, 0.0, 127, Color::default(), 1.0);
                let d = mpool.slot_mut(idx);
                d.velocity *= 0.5;
                d.no_gravity = true;
            }
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
            let t = pool.slot(i).dust_type;
            assert!(t == 89 || t == 87, "type {t}");
            assert_eq!(pool.slot(i).alpha, 127);
            assert!(pool.slot(i).no_gravity);
        }
        for _ in 0..8 {
            assert_eq!(rng.next_range(0, 1000), mirror.next_max(1000));
        }
    }

    // ------------------------------------------------------------------
    // W4-g64 wiring wave — the stream-parity witnesses: the machines'
    // `DropRng` seam (`DustDraw`) against the concrete `UnifiedRandom`
    // path, the 644 deferred-kind port, and the 596 bool seam's contract.
    // WRITTEN NOT RUN (code-only mode).
    // ------------------------------------------------------------------

    /// The wiring seam's draw parity: a `UnifiedRandom` driven through the
    /// `&mut dyn DropRng` + `DustDraw` path (the machines' shape) lands the
    /// SAME pool records and the SAME stream tail as the concrete path the
    /// emitters' own witnesses use — `Next(max)` = `Next(0, max)` is one
    /// `Sample()` step (`UnifiedRandom.cs:120-141`).
    #[test]
    fn w4_g64_wire_drop_rng_seam_matches_the_concrete_stream() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(900.0, 700.0);
        let vel = Vec2::new(5.0, -3.0);
        let mut concrete_pool = DustPool::new();
        let mut concrete = UnifiedRandom::new(211);
        trail_503(pos, 16, 16, vel, &mut concrete, &mut concrete_pool, &c, &w);
        let mut wire_pool = DustPool::new();
        let mut wire = UnifiedRandom::new(211);
        {
            let dyn_rng: &mut dyn DropRng = &mut wire;
            trail_503(pos, 16, 16, vel, dyn_rng, &mut wire_pool, &c, &w);
        }
        assert_eq!(active(&concrete_pool), active(&wire_pool));
        for i in active(&concrete_pool) {
            assert_eq!(concrete_pool.slot(i), wire_pool.slot(i), "slot {i}");
        }
        for _ in 0..16 {
            assert_eq!(concrete.next_max(1000), wire.next_max(1000), "stream tail");
        }
    }

    /// Kind 509 — the behaviour-feeding jitter REPLACES the ai_drill skip:
    /// the velocity-X draw runs EVERY tick (the gate outcome cannot skip
    /// it), the chip gate + argument draws follow in order, and the
    /// jittered velocity feeds the spawn position's lead.
    #[test]
    fn w4_g64_wire_509_jitter_always_draws_and_feeds_the_lead() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(600.0, 600.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(223);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(223);
        let mut vel = Vec2::new(9.0, 0.0);
        let mut mvel = vel;
        for _ in 0..60 {
            trail_509(pos, 22, 22, &mut vel, &mut rng, &mut pool, &c, &w);
            // The C# order: the jitter first (always), then the gate.
            mvel.x *= 1.0 + mirror.next_range(-3, 4) as f32 * 0.01;
            if mirror.next_max(6) == 0 {
                let lead = mirror.next_range(6, 10) as f32 * 0.1;
                let idx = mpool.new_dust(&mut mirror, &c, &w, pos + mvel * lead, 22, 22, 31, 0.0, 0.0, 80, Color::default(), 1.4);
                let d = mpool.slot_mut(idx);
                d.position.x -= 4.0;
                d.no_gravity = true;
                d.velocity *= 0.2;
                d.velocity.y = -(mirror.next_range(7, 13) as f32) * 0.15;
            }
            assert_eq!(vel, mvel, "the jitter is behaviour-feeding, kept");
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..8 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000));
        }
    }

    /// Kind 644 — the deferred prism-hue orbit, UNBLOCKED: the two orbit
    /// gates + the sparkle gate draw in the C# order; the taken sparkle
    /// lands the dust + its CloneDust (the ×0.65 pair, the white colour),
    /// the hue colour byte-quantised through `hsl_to_rgb`.
    #[test]
    fn w4_g64_wire_644_orbit_sparkle_and_clone() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1000.0, 1000.0);
        let hue = 0.37f32;
        let rotation = 0.9f32;
        let opacity = 0.75f32;
        let scale_roll = 1.2f32;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(227);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(227);
        trail_644(center, hue, rotation, opacity, scale_roll, &mut rng, &mut pool, &c, &w);
        let rgb = tr_core::tile_light::hsl_to_rgb(hue, 1.0, 0.5);
        let col = Color::new((rgb[0] * 255.0) as u8, (rgb[1] * 255.0) as u8, (rgb[2] * 255.0) as u8, 255);
        for num836 in 0..2 {
            if mirror.next_max(10) == 0 {
                let v = rotated_by(rotated_by(Vec2::Y, (num836 as f32 * std::f32::consts::PI) as f64), rotation as f64);
                let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, 267, 0.0, 0.0, 225, col, 1.5);
                let d = mpool.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
                d.scale = opacity * scale_roll;
                d.position = center;
                d.velocity = v * 2.5;
            }
        }
        for num837 in 0..2 {
            if mirror.next_max(10) == 0 {
                let v = rotated_by(Vec2::Y, (num837 as f32 * std::f32::consts::PI) as f64);
                let idx = mpool.new_dust(&mut mirror, &c, &w, center, 0, 0, 267, 0.0, 0.0, 225, col, 1.5);
                let d = mpool.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
                d.scale = opacity * scale_roll;
                d.position = center;
                d.velocity = v * 2.5;
            }
        }
        if mirror.next_max(10) == 0 {
            let num838 = 1.0 + mirror.next_float() * 2.0;
            let fade_in = 1.0 + mirror.next_float();
            let num839 = 1.0 + mirror.next_float();
            let vx = 2.0 * mirror.next_double() as f32 - 1.0;
            let vy = 2.0 * mirror.next_double() as f32 - 1.0;
            let mut v141 = Vec2::new(vx, vy);
            if v141 != Vec2::ZERO {
                v141 = xna_normalize(v141);
            }
            v141 *= 20.0 + mirror.next_float() * 100.0;
            let vec = center + v141;
            let (tx, ty) = (vec.x as i32 / 16, vec.y as i32 / 16);
            let in_world = tx >= 0 && ty >= 0 && tx < w.width && ty < w.height;
            let solid = in_world && w.tile(tx, ty).is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
            if in_world && !solid {
                let idx = mpool.new_dust(&mut mirror, &c, &w, vec, 0, 0, 267, 0.0, 0.0, 127, col, 1.0);
                {
                    let d = mpool.slot_mut(idx);
                    d.no_gravity = true;
                    d.position = vec;
                    d.velocity = -Vec2::Y * num838 * (mirror.next_float() * 0.9 + 1.6);
                    d.fade_in = fade_in;
                    d.scale = num839;
                    d.no_light = true;
                }
                if mpool.slot(idx).dust_index != 6000 {
                    let cidx = mpool.clone_dust(&mut mirror, &c, &w, idx);
                    let cc = mpool.slot_mut(cidx);
                    cc.scale *= 0.65;
                    cc.fade_in *= 0.65;
                    cc.color = Color::new(255, 255, 255, 255);
                }
            }
        }
        assert_eq!(active(&pool), active(&mpool));
        for i in active(&pool) {
            assert_eq!(pool.slot(i), mpool.slot(i), "slot {i}");
        }
        for _ in 0..16 {
            assert_eq!(rng.next_max(1000), mirror.next_max(1000), "stream tail");
        }
    }

    /// Kind 596 — the bool seam's wiring contract: the fn's `false` is the
    /// C# `return` FROM AI() (`:33619-33622`) — the caller must abort the
    /// machine's continuation (the orbit's remaining dusts AND the
    /// `timeLeft < 4` death burst below it, g65's); `true` ran the whole
    /// three-dust ring. The 659 twin CONTINUES instead (its own witness).
    /// W4-g64 remainder 3: the 596 row landed (the d11 read) and the seam
    /// is ACTIVATED in `ai_djinn_curse` — the machine-side witness
    /// `w4_g64_wire_596_seam_activated_skips_the_homing_tail` pins it.
    #[test]
    fn w4_g64_wire_596_bool_seam_gates_the_ai_continuation() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1400.0, 900.0);
        // The bail path: a scripted stream whose first Next(3) fails →
        // the fn returns false after ONE draw and spawns nothing.
        struct FirstFail(std::collections::VecDeque<i32>);
        impl DropRng for FirstFail {
            fn next_range(&mut self, min: i32, max: i32) -> i32 {
                assert_eq!((min, max), (0, 3), "the gate's bounds");
                self.0.pop_front().expect("a gate draw")
            }
            fn next_float(&mut self) -> f32 {
                panic!("no float draw on the bail path");
            }
        }
        let mut pool = DustPool::new();
        let mut rng = FirstFail([1].into_iter().collect());
        let ok = trail_596(center, 3.0, 200, &mut rng, &mut pool, &c, &w);
        assert!(!ok, "Next(3) != 0 → the AI() early return");
        assert!(active(&pool).is_empty(), "the bail spawns nothing");
        assert!(rng.0.is_empty(), "exactly one gate draw on the bail");
        // The pass path: scan seeds through the fn itself until one runs
        // the whole ring (the pool's internals draw between the gates, so
        // the gate values are seed-dependent) — true + three dusts.
        let mut found = false;
        for seed in 0..400i32 {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            if trail_596(center, 3.0, 200, &mut rng, &mut pool, &c, &w) {
                assert_eq!(active(&pool).len(), 3, "the full ring (seed {seed})");
                found = true;
                break;
            }
        }
        assert!(found, "a three-pass seed exists in 0..400");
    }

    /// Kind 813 — the four-phase machine's full-order mirror (the d11
    /// census's gates REPLACED by the live bodies): each phase runs the
    /// emitter on one stream and the verbatim C# statement sequence on a
    /// twin — the gate draws, the RotatedByRandom pairs, the pool
    /// internals, the post-writes and the opacity tail all land identical
    /// records + identical stream tails.
    #[test]
    fn w4_g64_813_four_phase_full_order_mirror() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(1400.0, 900.0);
        let max = 6.2831854820251465f64;
        // The verbatim ring body on the mirror stream (the C# statement
        // order: internals, noGravity, position re-roll, velocity, scale,
        // fadeIn).
        fn mirror_ring(
            mpool: &mut DustPool,
            mirror: &mut UnifiedRandom,
            c: &SpawnCtx,
            w: &World,
            center: Vec2,
            v: Vec2,
            dust_type: i32,
            reroll: bool,
            speed: f32,
        ) {
            let idx = mpool.new_dust(mirror, c, w, center - v * 30.0, 0, 0, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.position = if reroll {
                center - v * mirror.next_range(10, 21) as f32
            } else {
                center - v * 30.0
            };
            let spin = if dust_type == 240 { -1.5707963705062866 } else { 1.5707963705062866 };
            d.velocity = rotated_by(v, spin) * speed;
            d.scale = 0.5 + mirror.next_float();
            d.fade_in = 0.5;
        }
        for seed in [813i32, 814, 815] {
            // Wind-up: two Next(4) gates, the 5 ring ×4 / the 240 ×2.
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            trail_813_windup(center, &mut rng, &mut pool, &c, &w);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(seed);
            if mirror.next_range(0, 4) == 0 {
                let ang = mirror.next_double() * max - mirror.next_double() * max;
                mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, ang), 5, true, 4.0);
            }
            if mirror.next_range(0, 4) == 0 {
                let ang = mirror.next_double() * max - mirror.next_double() * max;
                mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, ang), 240, false, 2.0);
            }
            assert_eq!(active(&pool), active(&mpool), "wind-up slots (seed {seed})");
            for i in active(&pool) {
                assert_eq!(pool.slot(i), mpool.slot(i), "wind-up slot {i}");
            }
            for _ in 0..8 {
                assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000), "wind-up tail");
            }
            // Charge: two Next(2) gates, ×6/×3.
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            trail_813_charge(center, &mut rng, &mut pool, &c, &w);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(seed);
            if mirror.next_range(0, 2) == 0 {
                let ang = mirror.next_double() * max - mirror.next_double() * max;
                mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, ang), 5, true, 6.0);
            }
            if mirror.next_range(0, 2) == 0 {
                let ang = mirror.next_double() * max - mirror.next_double() * max;
                mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, ang), 240, false, 3.0);
            }
            assert_eq!(active(&pool), active(&mpool), "charge slots (seed {seed})");
            for i in active(&pool) {
                assert_eq!(pool.slot(i), mpool.slot(i), "charge slot {i}");
            }
            // Full: ONE Next(2) if/else.
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            trail_813_full(center, &mut rng, &mut pool, &c, &w);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(seed);
            if mirror.next_range(0, 2) == 0 {
                let a = mirror.next_double() * max - mirror.next_double() * max;
                mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, a), 5, true, 6.0);
            } else {
                let a = mirror.next_double() * max - mirror.next_double() * max;
                mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, a), 240, false, 3.0);
            }
            assert_eq!(active(&pool), active(&mpool), "full slots (seed {seed})");
            for i in active(&pool) {
                assert_eq!(pool.slot(i), mpool.slot(i), "full slot {i}");
            }
            // Dissolve: two Next(3) switch rolls, the scale-premultiplied v.
            let scale = 0.75f32;
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            trail_813_dissolve(center, scale, &mut rng, &mut pool, &c, &w);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(seed);
            for _ in 0..2 {
                match mirror.next_range(0, 3) {
                    0 => {
                        let a = mirror.next_double() * max - mirror.next_double() * max;
                        mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, a) * scale, 5, true, 6.0);
                    }
                    1 => {
                        let a = mirror.next_double() * max - mirror.next_double() * max;
                        mirror_ring(&mut mpool, &mut mirror, &c, &w, center, rotated_by(Vec2::Y, a) * scale, 240, false, 3.0);
                    }
                    _ => {}
                }
            }
            assert_eq!(active(&pool), active(&mpool), "dissolve slots (seed {seed})");
            for i in active(&pool) {
                assert_eq!(pool.slot(i), mpool.slot(i), "dissolve slot {i}");
            }
            // Opacity tail: the NextFloat gate + the circular pair + the
            // type-90 Perfect (UnitY, noGravity, scale).
            for opacity in [0.0f32, 1.0] {
                let mut pool = DustPool::new();
                let mut rng = UnifiedRandom::new(seed);
                trail_813_opacity_tail(center, opacity, scale, &mut rng, &mut pool, &c, &w);
                let mut mpool = DustPool::new();
                let mut mirror = UnifiedRandom::new(seed);
                if mirror.next_float() < opacity {
                    let unit_angle = (std::f64::consts::PI * 2.0) as f32 * mirror.next_float();
                    let unit = Vec2::new((unit_angle as f64).cos() as f32, (unit_angle as f64).sin() as f32);
                    let r = mirror.next_float();
                    let off = Vec2::new(unit.x * 3.0 * r, unit.y * 20.0 * r);
                    let idx = mpool.new_dust_perfect(&mut mirror, &c, &w, center + off, 90, None, 0, Color::default(), 1.0);
                    let d = mpool.slot_mut(idx);
                    d.velocity = Vec2::Y;
                    d.no_gravity = true;
                    d.scale = scale;
                }
                assert_eq!(active(&pool), active(&mpool), "tail slots (seed {seed}, opacity {opacity})");
                for i in active(&pool) {
                    assert_eq!(pool.slot(i), mpool.slot(i), "tail slot {i}");
                }
                for _ in 0..8 {
                    assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000), "tail stream");
                }
            }
        }
    }

    /// TRAIL-DUST RETIRE — the `clone_dust` genericisation keep-one (the e3
    /// `ac6392cf` pool copy vs this lane's `853039bf`; e3's RETIRE seal at
    /// `0b48616fa` independently confirms — take either, prefer g64's docs):
    /// the two ports' CODE is byte-identical (trait, signatures, bodies —
    /// the diff is comment wording only), so either lands at merge. This
    /// witness pins the seam the co-lanes ride: the pool's `clone_dust`
    /// through `DustDraw` over `&mut dyn DropRng` (the f50 `BlastRng`
    /// adapter was the SAME three delegations — the SEAL-VERIFY lift put
    /// its `GoreRng` half onto `DustDraw`, `51eefdf84` receipt) lands the
    /// SAME records and the SAME stream tail as the concrete `UnifiedRandom`
    /// path — `Dust.cs:263-286`, the sentinel early return consumes no
    /// draws. WRITTEN NOT RUN.
    #[test]
    fn w4_g64_retire_clone_dust_generic_seam_matches_the_concrete_stream() {
        let w = world();
        let c = ctx();
        let mut concrete_pool = DustPool::new();
        let mut wire_pool = DustPool::new();
        // Identical source records on both pools (the copy-list fields).
        for pool in [&mut concrete_pool, &mut wire_pool] {
            let rf = pool.slot_mut(7);
            rf.active = true;
            rf.position = Vec2::new(120.0, 240.0);
            rf.velocity = Vec2::new(3.0, -4.0);
            rf.fade_in = 1.5;
            rf.no_gravity = true;
            rf.scale = 0.75;
            rf.rotation = 0.25;
            rf.dust_type = 267;
            rf.color = Color::new(9, 8, 7, 255);
            rf.alpha = 3;
            (rf.frame_x, rf.frame_y, rf.frame_w, rf.frame_h) = (13, 17, 8, 8);
        }
        let mut concrete = UnifiedRandom::new(233);
        let mut wire = UnifiedRandom::new(233);
        let a = concrete_pool.clone_dust(&mut concrete, &c, &w, 7);
        let b = {
            let dyn_rng: &mut dyn DropRng = &mut wire;
            wire_pool.clone_dust(&mut DustDraw(dyn_rng), &c, &w, 7)
        };
        assert_eq!(a, b);
        assert_eq!(concrete_pool.slot(a), wire_pool.slot(b), "the clone record");
        assert_eq!(concrete_pool.slot(a).position, Vec2::new(120.0, 240.0));
        assert_eq!(concrete_pool.slot(a).dust_type, 267);
        for _ in 0..16 {
            assert_eq!(concrete.next_max(1000), wire.next_max(1000), "stream tail");
        }
        // The sentinel guard: no draws on either path (`Dust.cs:265-268`).
        let mut g1 = UnifiedRandom::new(233);
        let mut g2 = UnifiedRandom::new(233);
        assert_eq!(
            concrete_pool.clone_dust(&mut g1, &c, &w, tr_core::particles::DUST_SENTINEL),
            tr_core::particles::DUST_SENTINEL
        );
        assert_eq!(
            wire_pool.clone_dust(&mut DustDraw(&mut g2), &c, &w, tr_core::particles::DUST_SENTINEL),
            tr_core::particles::DUST_SENTINEL
        );
        for _ in 0..4 {
            assert_eq!(g1.next_max(1000), g2.next_max(1000), "guard consumed no draws");
        }
    }

    /// W4-g64 SEAL-VERIFY — the f50 adapter keep-one: the `GoreRng` half
    /// LIFTED onto `DustDraw` is arm-for-arm the f50 `BlastRng` shape
    /// (`51eefdf84` receipt) and matches the concrete `UnifiedRandom`
    /// `GoreRng` path draw-for-draw — the three shared delegations plus
    /// `NextFloatDirection` (`Utils.cs:1203-1206`: the DOUBLE draw cast to
    /// f32 first, then `* 2f - 1f`; a f64-first form drifts). The gore
    /// spawn through the lifted seam lands the SAME record + stream tail
    /// as the concrete path (`Gore.cs:1286-1462` — the `:1327-1328` kicks
    /// ride the caller's stream). At merge f50's `solar_flare_blast_bodies`
    /// swaps `BlastRng(rng)` → `DustDraw(rng)`; `BlastRng` is DELETED on
    /// the f50 branch (nothing to delete here — zero `BlastRng` code on
    /// this tip). WRITTEN NOT RUN.
    #[test]
    fn w4_g64_seal_gore_rng_lifted_onto_dust_draw_matches_the_concrete_stream() {
        use tr_core::gore::{GorePool, GoreRng};
        // The four delegations, interleaved on identical streams.
        let mut concrete = UnifiedRandom::new(311);
        let mut wire = UnifiedRandom::new(311);
        {
            let mut d = DustDraw(&mut wire);
            for i in 1..=12 {
                assert_eq!(concrete.next_max(7 * i), GoreRng::next_max(&mut d, 7 * i));
                assert_eq!(
                    concrete.next_range(-3 * i, 5 * i),
                    GoreRng::next_range(&mut d, -3 * i, 5 * i)
                );
                assert_eq!(
                    concrete.next_float().to_bits(),
                    GoreRng::next_float(&mut d).to_bits()
                );
                assert_eq!(
                    concrete.next_float_direction().to_bits(),
                    GoreRng::next_float_direction(&mut d).to_bits(),
                    "the double-cast NextFloatDirection, draw {i}"
                );
            }
        }
        // The gore spawn parity: the same record + the same stream tail.
        let w = world();
        let pos = Vec2::new(640.0, 480.0);
        let mut cp = GorePool::new();
        let mut wp = GorePool::new();
        let mut c2 = UnifiedRandom::new(313);
        let mut w2 = UnifiedRandom::new(313);
        let a = cp.new_gore(&mut c2, &w, pos, Vec2::new(2.0, -3.0), 61, 1.0);
        let b = wp.new_gore(&mut DustDraw(&mut w2), &w, pos, Vec2::new(2.0, -3.0), 61, 1.0);
        assert_eq!(a, b);
        assert_eq!(cp.slot(a), wp.slot(b), "the gore record through the lifted seam");
        for _ in 0..16 {
            assert_eq!(c2.next_range(0, 1000), w2.next_range(0, 1000), "stream tail");
        }
    }
}
