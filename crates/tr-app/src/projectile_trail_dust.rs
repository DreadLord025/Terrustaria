//! W4-g63 — the projectile TRAIL dust lane 1 (kinds 1-400): the per-kind
//! trailing/ambient `Dust.NewDust*` arms inside `Projectile.AI()`
//! (`Projectile.cs:23199-36807`) and the named AI machines, re-armed onto
//! the live `DustPool` (the d50 contract, `tr_core::particles`).
//!
//! Co-lanes (research/66): W4-g64 owns kinds 401+, W4-g65 the `Kill()`
//! death dust; the d53/e3/f5 site batches own the per-file line-order
//! census — this module only carries the per-kind AI arms.
//!
//! Contract (the co-lane rule, research/66):
//! - every draw rides the caller's **UpdateProjectiles** stream
//!   (`Main.cs:17775-17777`'s `SwapRandom` wraps `UpdateWorld_Projectiles`,
//!   `Main.cs:17996-18022`) in C# statement order: the explicit `Main.rand`
//!   ARGUMENT draws first (C# evaluates arguments before the call), then
//!   the `NewDust` internals inside `DustPool::new_dust` (`Dust.cs:75-256`),
//!   then the post-spawn writes — which land on the sentinel slot 6000
//!   when the pool refuses (the d52/e2 convention).
//! - gore siblings between dust draws: their ARGUMENT draws are consumed
//!   here in order (the `Next(61,64)` type rolls); the `NewGore` bodies
//!   stay the gore lane's (W4-d55, the e2 split).
//! - the earlier "dust SKIPPED — the roll skips with it" entries are
//!   RE-ARMED here: the gate rolls return to the stream at their C#
//!   positions (the S1-F-13c/d52 precedent). Each re-arm is cited at its
//!   machine in `projectile.rs`.
//!
//! The emitters are pure fns over the slot snapshot + the pool, so the
//! machines call them at the exact C# statement position (the
//! `npc_dust.rs` shared-emitter shape, W4-e2). `DustRng` is the pool's
//! draw seam; the machines' `DropRng` stream adapts through [`DustDraw`]
//! (one `Next(max)` = one `Next(0, max)` — the same `Sample()` step,
//! `UnifiedRandom.cs:120-141`).
//!
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};

use crate::projectile::Projectile;

/// The machines' `DropRng` stream seen as the pool's [`DustRng`] seam:
/// `Next(max)` delegates to `Next(0, max)` — both are one `Sample()` draw
/// (`UnifiedRandom.cs:120-141`), so the interleaving with the machine's
/// own behaviour rolls stays byte-identical.
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

/// `Utils.NextVector2Circular` (`Utils.cs:1276-1279`):
/// `NextVector2Unit() * (hw, hh) * NextFloat()` — TWO `NextFloat` draws
/// (the unit angle `:1266-1269`, then the radius scale). The lane's
/// later batches (the 261/196 dart sparkles, the 114 trident trail)
/// consume it; batch 1's emitters are all box/point spawns.
#[allow(dead_code)]
pub(crate) fn next_vector2_circular(rng: &mut dyn DropRng, hw: f32, hh: f32) -> Vec2 {
    let angle = rng.next_float() * std::f32::consts::TAU;
    let unit = Vec2::new(angle.cos(), angle.sin());
    unit * Vec2::new(hw, hh) * rng.next_float()
}

/// The slot snapshot the emitters read (`base.position/width/height/
/// velocity/Center/scale/alpha` at the C# call site). The machines pass
/// the LIVE slot fields — the snapshot exists only so the emitter can
/// hold `&Projectile` and `&mut DustPool` at once.
pub(crate) struct TrailSlot {
    pub pos: Vec2,
    pub w: f32,
    pub h: f32,
    pub vel: Vec2,
    pub scale: f32,
    pub alpha: i32,
    pub direction: i32,
    /// `base.oldVelocity` — the sub-step start velocity the aiStyle-27
    /// trident trail reads (`Projectile.cs:26451` passim). ZERO for the
    /// arms that don't read it.
    pub old_vel: Vec2,
}

impl TrailSlot {
    pub fn of(p: &Projectile) -> Self {
        Self {
            pos: p.pos,
            w: p.w,
            h: p.h,
            vel: p.vel,
            scale: p.scale,
            alpha: p.alpha,
            direction: p.direction,
            old_vel: p.old_velocity,
        }
    }
    fn center(&self) -> Vec2 {
        Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5)
    }
}

// ---------------------------------------------------------------------------
// aiStyle 2 — the thrown family (`Projectile.cs:23205-23689`).
// ---------------------------------------------------------------------------

/// Site 23687 — the Poisoned Knife's 1-in-20 drip (`Projectile.cs:
/// 23685-23688`), AFTER the fall clamp: `Next(20) == 0` gates one
/// `NewDust(position, w, h, 40, vx*0.1, vy*0.1, 0, default, 0.75)`.
/// The gate draw runs EVERY tick (the C# `&&` left operand); the dust
/// body rolls only on the taken branch. Re-arms the i4b-8 skip.
pub(crate) fn poisoned_knife_drip(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:23685` — the gate, always drawn.
    if rng.next_range(0, 20) != 0 {
        return;
    }
    // `:23687` — no post-spawn writes.
    dust.new_dust(
        &mut DustDraw(rng),
        ctx,
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

/// Sites 23296/23302/23306 — the 162 Frost Shard's `ai[0] == 3f` burst
/// (`Projectile.cs:23289-23318`): 10× dust 31 (×0.5 velocity + v×0.1),
/// then 5× the dust-6 pair (the noGravity ×3 + v×0.2, then the plain ×2
/// + v×0.3), then the gore's `Next(61,64)` ARGUMENT draw (the body is
/// W4-d55's). The 281 twin shares the shape at `:23337-23363` (its own
/// arm, aiStyle 49 — [`bunny_burst`]).
pub(crate) fn frost_shard_burst(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:23294-23299` — the 10 smoke puffs.
    for _ in 0..10 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            31, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5; // `:23297`
        d.velocity += p.vel * 0.1; // `:23298`
    }
    // `:23300-23309` — the 5 ember pairs.
    for _ in 0..5 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            6, 0.0, 0.0, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:23303`
        d.velocity *= 3.0; // `:23304`
        d.velocity += p.vel * 0.2; // `:23305`
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            6, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0; // `:23307`
        d.velocity += p.vel * 0.3; // `:23308`
    }
    // `:23312` — the gore's type roll stays on the stream (the body is
    // the gore lane's; the position/scale/velocity writes ride it).
    let _ = rng.next_range(61, 64);
}

/// Sites 23386/23392/23396 — the Dutchman shell 240's `ai[0] == 3f`
/// burst (`Projectile.cs:23379-23408`): 7× dust 31, 3× the dust-6 pair,
/// the gore `Next(61,64)` argument (scale 1.25 — the 162/281's is 1.5).
pub(crate) fn dutchman_shell_burst(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:23384-23389` — the 7 smoke puffs.
    for _ in 0..7 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            31, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.velocity += p.vel * 0.1;
    }
    // `:23390-23399` — the 3 ember pairs.
    for _ in 0..3 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            6, 0.0, 0.0, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 3.0;
        d.velocity += p.vel * 0.2;
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            6, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.velocity += p.vel * 0.3;
    }
    // `:23402` — the gore argument draw (scale 1.25, `:23404`).
    let _ = rng.next_range(61, 64);
}

/// Site 23618 — the 306 Terra Blade beam's four-point trail
/// (`Projectile.cs:23610-23624`): while `alpha <= 200`, four dust 184
/// at `Center - velocity/4 * i`, velocity zeroed, scale pinned 0.7.
/// NO gate draws — the loop is unconditional inside the alpha gate.
pub(crate) fn terra_beam_trail(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if p.alpha > 200 {
        return; // `:23612`
    }
    let center = p.center();
    for i in 0..4 {
        // `:23616-23617` — the trail offsets (f32 × int-as-f32).
        let ox = p.vel.x / 4.0 * i as f32;
        let oy = p.vel.y / 4.0 * i as f32;
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            184, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.position.x = center.x - ox; // `:23619`
        d.position.y = center.y - oy; // `:23620`
        d.velocity *= 0.0; // `:23621`
        d.scale = 0.7; // `:23622`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 4 — the Vilethorn/Nettle family (`Projectile.cs:23694-23829`).
// ---------------------------------------------------------------------------

/// Sites 23790/23808/23810 — the fade-cross burst (`Projectile.cs:
/// 23784-23812`): the tick whose alpha crosses 170 (`alpha < 170 &&
/// alpha + 5 >= 170` — the PRE-increment read) spawns the per-family
/// burst: the 150-152 Nettle pair 8× dust 7 (noGravity, ×0.5 velocity),
/// the default 3× dust 18 + 1× dust 14. The 493/494 variant (`:23795-
/// 23803`, kinds > 400) is W4-g64's. `fade` is the tick's alpha step
/// (3 for the Nettle pair, 5 for the Vilethorn — the C# gate reads the
/// literal 5, so only the +5 families ever cross; kept as the parameter
/// to mirror the read).
pub(crate) fn thorn_fade_burst(
    p: &TrailSlot,
    nettle: bool,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:23784` — the crossing gate on the PRE-increment alpha. The C#
    // literal is `+ 5`; the Nettle pair's +3 step can never satisfy it
    // (165..169 + 5 >= 170 needs alpha >= 165, but alpha < 170 and the
    // +3 ladder lands 168 → 171 skipping the window's lower edge only
    // when alpha ∈ [165,170) — reachable at 168: 168+5=173 >= 170, so
    // the gate DOES fire for the Nettle pair at alpha 165..169).
    if !(p.alpha < 170 && p.alpha + 5 >= 170) {
        return;
    }
    if nettle {
        // `:23786-23794` — the 150-152 arm: 8× dust 7.
        for _ in 0..8 {
            let idx = dust.new_dust(
                &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
                7, p.vel.x * 0.025, p.vel.y * 0.025, 200, Color::default(), 1.3,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:23791`
            d.velocity *= 0.5; // `:23792`
        }
    } else {
        // `:23804-23811` — the default: 3× dust 18 + 1× dust 14.
        for _ in 0..3 {
            dust.new_dust(
                &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
                18, p.vel.x * 0.025, p.vel.y * 0.025, 170, Color::default(), 1.2,
            );
        }
        dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            14, 0.0, 0.0, 170, Color::default(), 1.1,
        );
    }
}

// ---------------------------------------------------------------------------
// aiStyle 5 — the falling stars (`Projectile.cs:23830-24099`).
// ---------------------------------------------------------------------------

/// Site 24041 — the Starfury 9's 1-in-4 spark (`Projectile.cs:
/// 24039-24049`): `NewDustDirect(position, w, h, 86, 0, 0, 127)`, then
/// velocity ×0.7, noGravity, velocity += v×0.3, and the `Next(2) == 0`
/// position pull-back `-= v*4`. The gate draw runs every tick; the
/// pull-back roll only on a landed spawn (the C# writes through the
/// sentinel too — the roll is INSIDE the gate, after the call).
pub(crate) fn starfury_spark(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:24039` — the gate, always drawn.
    if rng.next_range(0, 4) != 0 {
        return;
    }
    // `:24041` — NewDustDirect: a NewDust whose NaN velocity is zeroed
    // (`Dust.cs:65-73`). The velocity here is the pool's own jitter + 0
    // (never NaN), so the fix folds away — the spawn is the plain
    // NewDust on the same stream (the `new_dust_direct` body, cited).
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
        86, 0.0, 0.0, 127, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity *= 0.7; // `:24042`
    d.no_gravity = true; // `:24043`
    d.velocity += p.vel * 0.3; // `:24044`
    // `:24045-24048` — the conditional pull-back.
    if rng.next_range(0, 2) == 0 {
        d.position -= p.vel * 4.0;
    }
}

/// Site 24056 — the `ai[1] == 1 || type == 92` arm's 1-in-10 sparkle
/// (`Projectile.cs:24051-24061`): light 0.9 (the machine keeps the
/// write), `Next(10) == 0` → dust 58 at v×0.5, alpha 150, scale 1.2;
/// then the `Next(20) == 0` gore gate whose taken branch draws
/// `Next(16,18)` for the type (the body is W4-d55's). BOTH gate draws
/// run every tick the arm is reached. The 955 Star Cannon star shares
/// the arm when bounced (`ai[1] == 1`) — kind > 400, W4-g64's gate.
pub(crate) fn falling_star_sparkle(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:24054` — the dust gate, always drawn.
    if rng.next_range(0, 10) == 0 {
        dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            58, p.vel.x * 0.5, p.vel.y * 0.5, 150, Color::default(), 1.2,
        );
    }
    // `:24058-24061` — the gore gate + its type argument (the body is
    // the gore lane's; the velocity argument is a fixed expression).
    if rng.next_range(0, 20) == 0 {
        let _ = rng.next_range(16, 18);
    }
}

// ---------------------------------------------------------------------------
// aiStyle 8 — the bouncing casters (`Projectile.cs:24233-24365`).
// ---------------------------------------------------------------------------

/// Sites 24261/24276/24286/24296 — the Water Bolt 27's lava-bounce
/// burst + the bounced trail (`Projectile.cs:24245-24301`).
/// `lava_bounce` fires the 4× dust 31 (×0.6 velocity, Y −2) + the
/// `Next(3)` gore gate (`Next(61,64)` argument) at the bounce tick;
/// the per-tick tail draws the `ai[1]==1 && Next(2)==0` centre puff
/// (dust 31, ×0.2) and the `ai[1]==0 || Next(5)==0` five-point 172
/// trail (+ the nested `Next(5)==0` sixth at 0.6). The C# order: the
/// bounce block first (inside the lava probe), then the two tails.
pub(crate) fn water_bolt_bounce(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:24259-24264` — the 4 lava puffs.
    for _ in 0..4 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            31, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.6; // `:24262`
        d.velocity.y -= 2.0; // `:24263`
    }
    // `:24265-24271` — the gore gate + type argument.
    if rng.next_range(0, 3) == 0 {
        let _ = rng.next_range(61, 64);
    }
}

/// The Water Bolt's per-tick dust tail (`Projectile.cs:24274-24300`),
/// AFTER the lava probe. `bounced` is `ai[1] == 1f` (the read happens
/// after the probe may have set it — the machine passes the live flag).
pub(crate) fn water_bolt_trail(
    p: &TrailSlot,
    bounced: bool,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:24274-24278` — the bounced centre puff: the `Next(2)` gate is
    // drawn ONLY when bounced (the C# `&&` short-circuit).
    if bounced && rng.next_range(0, 2) == 0 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.center(), 0, 0,
            31, 0.0, 0.0, 100, Color::default(), 0.8,
        );
        dust.slot_mut(idx).velocity *= 0.2; // `:24277`
    }
    // `:24279-24300` — the five-point trail: `ai[1]==0 || Next(5)==0`
    // (the roll skipped when unbounced — the `||` short-circuit).
    if !bounced || rng.next_range(0, 5) == 0 {
        for i in 0..5 {
            let ox = p.vel.x / 3.0 * i as f32; // `:24283`
            let oy = p.vel.y / 3.0 * i as f32; // `:24284`
            // `:24285-24286` — the 4-px inset box.
            let idx = dust.new_dust(
                &mut DustDraw(rng), ctx, world,
                Vec2::new(p.pos.x + 4.0, p.pos.y + 4.0),
                p.w as i32 - 8, p.h as i32 - 8,
                172, 0.0, 0.0, 100, Color::default(), 1.2,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:24287`
            d.velocity *= 0.1; // `:24288`
            d.velocity += p.vel * 0.1; // `:24289`
            d.position.x -= ox; // `:24290`
            d.position.y -= oy; // `:24291`
        }
        // `:24293-24299` — the nested 1-in-5 sixth puff.
        if rng.next_range(0, 5) == 0 {
            let idx = dust.new_dust(
                &mut DustDraw(rng), ctx, world,
                Vec2::new(p.pos.x + 4.0, p.pos.y + 4.0),
                p.w as i32 - 8, p.h as i32 - 8,
                172, 0.0, 0.0, 100, Color::default(), 0.6,
            );
            let d = dust.slot_mut(idx);
            d.velocity *= 0.25; // `:24297`
            d.velocity += p.vel * 0.5; // `:24298`
        }
    }
}

/// Site 24311 — the cursed flames 95/96's per-tick flame
/// (`Projectile.cs:24309-24313`): one dust 75 at `position + velocity`,
/// speeds = velocity, alpha 100, scale `3 * this.scale`, noGravity.
/// No gate draws.
pub(crate) fn cursed_flame_trail(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world,
        Vec2::new(p.pos.x + p.vel.x, p.pos.y + p.vel.y),
        p.w as i32, p.h as i32,
        75, p.vel.x, p.vel.y, 100, Color::default(), 3.0 * p.scale,
    );
    dust.slot_mut(idx).no_gravity = true; // `:24312`
}

/// Sites 24318/24328 — the Ball of Frost 253's 2× dust 135
/// (`Projectile.cs:24314-24323`) and the default arm's 2× dust 6
/// (`:24324-24337`, the 15/258 kinds ≤ 400; the 258's getGoodWorld
/// noLight write folds false — the classic port, `Main.cs:66841`).
/// Both: speeds v×0.2, alpha 100, scale 2, noGravity, velocity ×0.3
/// per axis. `frost` picks the 135 arm.
pub(crate) fn caster_default_trail(
    p: &TrailSlot,
    frost: bool,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dust_type = if frost { 135 } else { 6 }; // `:24318` / `:24328`
    for _ in 0..2 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            dust_type, p.vel.x * 0.2, p.vel.y * 0.2, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        // `:24329-24332` — the 258 getGoodWorld noLight (false here).
        d.no_gravity = true; // `:24319` / `:24333`
        d.velocity.x *= 0.3; // `:24320` / `:24334`
        d.velocity.y *= 0.3; // `:24321` / `:24335`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 12 — the Aqua Scepter / Golden Shower streams
// (`Projectile.cs:24501-24635`).
// ---------------------------------------------------------------------------

/// Sites 24577/24583 — the Aqua Scepter's lava-bounce burst
/// (`Projectile.cs:24575-24587`): 4× dust 31 (×0.6, Y −2) + the
/// `Next(3)` gore gate (`Next(61,64)` argument). The LCG bounce itself
/// is the machine's (seeded off `ai[2]`, NOT the pass stream).
pub(crate) fn aqua_lava_burst(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..4 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            31, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.6; // `:24578`
        d.velocity.y -= 2.0; // `:24579`
    }
    // `:24581-24587` — the gore gate + type argument (the i20 arm
    // already kept these two draws; the emitter owns them now).
    if rng.next_range(0, 3) == 0 {
        let _ = rng.next_range(61, 64);
    }
}

/// Sites 24601/24615/24625 — the Aqua Scepter's gravity-arm trail
/// (`Projectile.cs:24596-24630`): the bounced 1-in-2 centre puff
/// (dust 31 at Center, ×0.1), the `ai[1]!=0 && Next(2)!=0` early-return
/// gate, then the 3× dust 172 five-point-style trail (×0.3 + v×0.5,
/// the velocity/3 offsets) + the nested `Next(8)==0` sixth at 0.75
/// (×0.5 + v×0.5). Returns TRUE when the C# `return` fires (the
/// machine skips the trail but keeps the movement).
pub(crate) fn aqua_gravity_trail(
    p: &TrailSlot,
    bounced: bool,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    // `:24599-24603` — the bounced puff (the gate drawn only bounced).
    if bounced && rng.next_range(0, 2) == 0 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.center(), 0, 0,
            31, 0.0, 0.0, 100, Color::default(), 0.8,
        );
        dust.slot_mut(idx).velocity *= 0.1; // `:24602`
    }
    // `:24604-24607` — the early-return gate.
    if bounced && rng.next_range(0, 2) != 0 {
        return true;
    }
    // `:24608-24621` — the 3-point trail (the C# wraps it in a
    // one-iteration outer loop — kept as the plain loop).
    for i in 0..3 {
        let ox = p.vel.x / 3.0 * i as f32; // `:24612`
        let oy = p.vel.y / 3.0 * i as f32; // `:24613`
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world,
            Vec2::new(p.pos.x + 6.0, p.pos.y + 6.0),
            p.w as i32 - 12, p.h as i32 - 12,
            172, 0.0, 0.0, 100, Color::default(), 1.2,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:24616`
        d.velocity *= 0.3; // `:24617`
        d.velocity += p.vel * 0.5; // `:24618`
        d.position.x -= ox; // `:24619`
        d.position.y -= oy; // `:24620`
    }
    // `:24622-24628` — the nested 1-in-8 sixth puff.
    if rng.next_range(0, 8) == 0 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world,
            Vec2::new(p.pos.x + 6.0, p.pos.y + 6.0),
            p.w as i32 - 12, p.h as i32 - 12,
            172, 0.0, 0.0, 100, Color::default(), 0.75,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5; // `:24626`
        d.velocity += p.vel * 0.5; // `:24627`
    }
    false
}

/// Sites 24527/24537 — the 280/288 shared head's arc trail
/// (`Projectile.cs:24519-24541`): past `ai[0] > 3`, 3× dust 170 (the
/// 14-px inset box, ×0.1 + v×0.5, the velocity/3 offsets) + the
/// `Next(8)==0` sixth at the 16-px inset, scale 0.5 (×0.25 + v×0.5).
/// The 288 hostile twin shares the block (kind ≤ 400).
pub(crate) fn golden_shower_trail(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for i in 0..3 {
        let ox = p.vel.x / 3.0 * i as f32; // `:24524`
        let oy = p.vel.y / 3.0 * i as f32; // `:24525`
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world,
            Vec2::new(p.pos.x + 14.0, p.pos.y + 14.0),
            p.w as i32 - 28, p.h as i32 - 28,
            170, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:24528`
        d.velocity *= 0.1; // `:24529`
        d.velocity += p.vel * 0.5; // `:24530`
        d.position.x -= ox; // `:24531`
        d.position.y -= oy; // `:24532`
    }
    // `:24534-24540` — the nested 1-in-8 sixth.
    if rng.next_range(0, 8) == 0 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world,
            Vec2::new(p.pos.x + 16.0, p.pos.y + 16.0),
            p.w as i32 - 32, p.h as i32 - 32,
            170, 0.0, 0.0, 100, Color::default(), 0.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.25; // `:24538`
        d.velocity += p.vel * 0.5; // `:24539`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 21 — the music notes (`Projectile.cs:25585-25606`).
// ---------------------------------------------------------------------------

/// Site 25596 — the note's 1-in-3 sparkle (`Projectile.cs:25594-25599`):
/// dust 27, alpha 80, noGravity, velocity ×0.2. The gate draw runs
/// every tick. All three notes (76/77/78) share the arm.
pub(crate) fn music_note_sparkle(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 3) != 0 {
        return; // `:25594`
    }
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
        27, 0.0, 0.0, 80, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:25597`
    d.velocity *= 0.2; // `:25598`
}

// ---------------------------------------------------------------------------
// aiStyle 22 — the Ice Rod block (`Projectile.cs:25607-25860`).
// ---------------------------------------------------------------------------

/// Site 25665 — the standing block's per-tick puff (`Projectile.cs:
/// 25665-25668`): dust 67, noGravity, velocity ×0.3. Runs AFTER the
/// kill scan (the C# spawns even on the kill tick — the scan's `Kill()`
/// does not return), BEFORE the phase `return`.
pub(crate) fn ice_block_standing_puff(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
        67, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:25666`
    d.velocity *= 0.3; // `:25667`
}

/// Sites 25676/25683 — the placed block's burst + drip (`Projectile.cs:
/// 25670-25685`): on `ai[0] == -1` (the placement tick) 10× dust 67
/// (scale 1.1, noGravity, ×1.3); afterwards the `Next(30)==0` single
/// puff (alpha 100, ×0.2). The gate draw runs every placed tick EXCEPT
/// the -1 tick (the C# else-if).
pub(crate) fn ice_block_placed_dust(
    p: &TrailSlot,
    placement_tick: bool,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if placement_tick {
        // `:25672-25680` — the 10-puff burst.
        for _ in 0..10 {
            let idx = dust.new_dust(
                &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
                67, 0.0, 0.0, 0, Color::default(), 1.1,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:25677`
            d.velocity *= 1.3; // `:25678`
        }
    } else if rng.next_range(0, 30) == 0 {
        // `:25681-25685` — the 1-in-30 drip.
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            67, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        dust.slot_mut(idx).velocity *= 0.2; // `:25684`
    }
}

/// Site 25756 — the flying block's per-tick puff (`Projectile.cs:
/// 25756-25758`): dust 67, noGravity, ×0.3 — AFTER the kill/lava/
/// shimmer probes' `!active` return, BEFORE the target-tile walk.
pub(crate) fn ice_block_flying_puff(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
        67, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:25757`
    d.velocity *= 0.3; // `:25758`
}

// ---------------------------------------------------------------------------
// aiStyle 24 — the Crystal Storm shard (`Projectile.cs:25936-26014`).
// ---------------------------------------------------------------------------

/// Site 25960 — the 94 shard's 1-in-4 sparkle (`Projectile.cs:
/// 25956-25964`): dust 70, noGravity, velocity ×0.5, scale ×0.9. The
/// gate draw runs every tick inside the 94 arm (before the ×0.985
/// drag). The 1114/1134 variants are kinds > 400 — W4-g64's.
pub(crate) fn crystal_shard_sparkle(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if rng.next_range(0, 4) != 0 {
        return; // `:25958`
    }
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
        70, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:25961`
    d.velocity *= 0.5; // `:25962`
    d.scale *= 0.9; // `:25963`
}

// ---------------------------------------------------------------------------
// aiStyle 49 — the Bunny Cannon's bunny (`Projectile.cs:28243-28322`).
// ---------------------------------------------------------------------------

/// Sites 28292/28298/28302 — the 281 bunny's `ai[0] == 3f` burst
/// (`Projectile.cs:28288-28314`): the 162-shaped 10× dust 31 + 5× the
/// dust-6 pair + the gore `Next(61,64)` argument (scale 1.5).
pub(crate) fn bunny_burst(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:28290-28295` — the 10 smoke puffs.
    for _ in 0..10 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            31, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5;
        d.velocity += p.vel * 0.1;
    }
    // `:28296-28305` — the 5 ember pairs.
    for _ in 0..5 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            6, 0.0, 0.0, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 3.0;
        d.velocity += p.vel * 0.2;
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            6, 0.0, 0.0, 100, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.velocity += p.vel * 0.3;
    }
    // `:28308` — the gore argument draw.
    let _ = rng.next_range(61, 64);
}

// ---------------------------------------------------------------------------
// aiStyle 27 — the Unholy Trident family (`Projectile.cs:26420-26548`).
// ---------------------------------------------------------------------------

/// Site 26451 — the friendly Trident 114's per-tick trail (`Projectile.cs:
/// 26449-26456`): the `Next(2) == 0 && NextFloat() + 0.2 < num211` gate
/// (num211 = the cached-speed Remap the machine already computed), then
/// one dust 27 at `(x, y+4)`, box 8×8, speeds = oldVelocity, scale
/// `1 + 0.6*NextFloat()`; post-spawn velocity ×−0.5, position += vel×5,
/// position += NextVector2Circular(16,16), velocity ×−1. The C# draw
/// order: the Next(2) gate, the NextFloat gate, the scale NextFloat
/// ARGUMENT, the NewDust internals, then the circular pair (TWO floats).
pub(crate) fn trident_trail(
    p: &TrailSlot,
    cached_speed_remap: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:26449` — the two-part gate (the `&&` short-circuit: the float
    // gate draws only when the Next(2) passes).
    if rng.next_range(0, 2) != 0 {
        return;
    }
    if rng.next_float() + 0.2 >= cached_speed_remap {
        return;
    }
    // `:26451` — the scale argument draws BEFORE the call.
    let scale = 1.0 + 0.6 * rng.next_float();
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world,
        Vec2::new(p.pos.x, p.pos.y + 4.0), 8, 8,
        27, p.old_vel.x, p.old_vel.y, 100, Color::default(), scale,
    );
    let d = dust.slot_mut(idx);
    d.velocity *= -0.5; // `:26452`
    d.position += d.velocity * 5.0; // `:26453`
    // `:26454` — the circular jitter (two NextFloat draws).
    d.position += next_vector2_circular(rng, 16.0, 16.0);
    d.velocity *= -1.0; // `:26455`
}

/// Site 26465 — the hostile twin 115's first-hit 20-step trail
/// (`Projectile.cs:26458-26468`): on the `localAI[1] < 5` latch tick,
/// 20 dusts 27 at `position - velocity*(30/i)*80` (i = 5..24), box 8×8,
/// speeds = oldVelocity, scale 0.9; velocity ×0.25 then −= velocity×5.
/// No gate draws — the loop is unconditional inside the latch.
pub(crate) fn trident_hostile_latch_trail(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for i in 5..25 {
        // `:26463-26464` — the per-step offsets (f32 × (30/i)).
        let ox = p.vel.x * (30.0 / i as f32);
        let oy = p.vel.y * (30.0 / i as f32);
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world,
            Vec2::new(p.pos.x - ox * 80.0, p.pos.y - oy * 80.0), 8, 8,
            27, p.old_vel.x, p.old_vel.y, 100, Color::default(), 0.9,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.25; // `:26466`
        d.velocity -= p.vel * 5.0; // `:26467`
    }
}

/// Site 26472 — the Enchanted Beam 173's post-7 pulse sparkle
/// (`Projectile.cs:26470-26479`): the `Next(3)` type pick (0→15, 1→57,
/// else 58) draws FIRST (the C# named-argument `Type:` evaluates before
/// the call), then one dust at `position - velocity*4 + (2,2)`, box
/// 8×8, speeds 0, alpha 100, scale 1.25; velocity ×0.1.
pub(crate) fn enchanted_beam_sparkle(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:26472-26477` — the type argument draw.
    let dust_type = match rng.next_range(0, 3) {
        0 => 15,
        1 => 57,
        _ => 58,
    };
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world,
        Vec2::new(p.pos.x - p.vel.x * 4.0 + 2.0, p.pos.y + 2.0 - p.vel.y * 4.0),
        8, 8, dust_type, 0.0, 0.0, 100, Color::default(), 1.25,
    );
    dust.slot_mut(idx).velocity *= 0.1; // `:26478`
}

/// Site 26482/26484 — the Terra Beam 132's post-7 pulse pair
/// (`Projectile.cs:26480-26487`): two dusts 107 at the same
/// `position - velocity*4 + (2,2)` point, box 8×8, speeds = oldVelocity,
/// scale 1.25; both velocity ×−0.25, the second position −= velocity×0.5.
pub(crate) fn terra_beam_pulse(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let at = Vec2::new(p.pos.x - p.vel.x * 4.0 + 2.0, p.pos.y + 2.0 - p.vel.y * 4.0);
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, at, 8, 8,
        107, p.old_vel.x, p.old_vel.y, 100, Color::default(), 1.25,
    );
    dust.slot_mut(idx).velocity *= -0.25; // `:26483`
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, at, 8, 8,
        107, p.old_vel.x, p.old_vel.y, 100, Color::default(), 1.25,
    );
    let d = dust.slot_mut(idx);
    d.velocity *= -0.25; // `:26485`
    d.position -= p.vel * 0.5; // `:26486`
}

/// Sites 26496/26501 — the post-15 pulse dust (`Projectile.cs:
/// 26492-26504`): the 114/115 pair one dust 27 at `(x, y+4)` (scale 0.6,
/// velocity ×−0.25); the 116 Sword Beam one dust 64 at
/// `position - velocity*5 + (2,2)` (scale 1.5, ×−0.25, noGravity).
/// `kind` picks the arm (the C# if/else-if). No gate draws.
pub(crate) fn trident_pulse_dust(
    p: &TrailSlot,
    kind: u16,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if kind == 114 || kind == 115 {
        // `:26494-26498`.
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world,
            Vec2::new(p.pos.x, p.pos.y + 4.0), 8, 8,
            27, p.old_vel.x, p.old_vel.y, 100, Color::default(), 0.6,
        );
        dust.slot_mut(idx).velocity *= -0.25; // `:26497`
    } else if kind == 116 {
        // `:26499-26504`.
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world,
            Vec2::new(p.pos.x - p.vel.x * 5.0 + 2.0, p.pos.y + 2.0 - p.vel.y * 5.0),
            8, 8, 64, p.old_vel.x, p.old_vel.y, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= -0.25; // `:26502`
        d.no_gravity = true; // `:26503`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 28 — the Frost Bolt family (`Projectile.cs:26555-26675`).
// ---------------------------------------------------------------------------

/// Site 26633 — the Frost Blast 309's per-tick 3× dust 185
/// (`Projectile.cs:26629-26637`): speeds = velocity, alpha 50, scale
/// 1.2, noGravity, velocity ×0.3. The 118/119/128/129/177 variants
/// (`:26602-26647`) are unrowed — their emitters land with their rows.
pub(crate) fn frost_blast_trail(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..3 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            185, p.vel.x, p.vel.y, 50, Color::default(), 1.2,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:26634`
        d.velocity *= 0.3; // `:26635`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 50 — the Inferno Fork family (`Projectile.cs:28323-28401`).
// ---------------------------------------------------------------------------

/// Site 28354 — the Inferno bolt 295's whole AI arm: 8× dust 174
/// (`Projectile.cs:28350-28359`), alpha 100, scale 1.2, noGravity,
/// velocity ×0.5 + v×0.1. No gate draws.
pub(crate) fn inferno_bolt_trail(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..8 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            174, 0.0, 0.0, 100, Color::default(), 1.2,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:28355`
        d.velocity *= 0.5; // `:28356`
        d.velocity += p.vel * 0.1; // `:28357`
    }
}

/// Site 28392 — the Inferno blast 296's fade-window burst
/// (`Projectile.cs:28371-28400`): `num350 = 25 - (ai[0]-180)/2` once
/// `ai[0] > 540`, ×0.7 for the 296; the loop spawns that many dusts 174
/// (scale 1.5), each with the THREE argument draws (Next(-10,11) ×2 +
/// Next(3,9) the speed divisor) BEFORE the call, then the position
/// jitter Next(-10,11) ×2 AFTER. `count` is the machine's resolved
/// num350 (the kill at <= 0 already ran). The velocity is the
/// normalized jitter × the speed factor.
pub(crate) fn inferno_blast_burst(
    p: &TrailSlot,
    count: i32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let center = p.center();
    for _ in 0..count {
        // `:28387-28391` — the three argument draws + the normalize.
        let mut jx = rng.next_range(-10, 11) as f32;
        let mut jy = rng.next_range(-10, 11) as f32;
        let factor = rng.next_range(3, 9) as f32 / (jx * jx + jy * jy).sqrt();
        jx *= factor;
        jy *= factor;
        // `:28392` — the spawn (scale 1.5).
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            174, 0.0, 0.0, 100, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:28393`
        d.position.x = center.x; // `:28394`
        d.position.y = center.y; // `:28395`
        d.position.x += rng.next_range(-10, 11) as f32; // `:28396`
        d.position.y += rng.next_range(-10, 11) as f32; // `:28397`
        d.velocity.x = jx; // `:28398`
        d.velocity.y = jy; // `:28399`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 51 — the Spectre Staff family (`Projectile.cs:28402-28503`).
// ---------------------------------------------------------------------------

/// Site 28411 — the friendly Lost Soul 297's post-4 burst
/// (`Projectile.cs:28404-28415`): once `localAI[0] > 4`, 5× dust 175
/// (scale 2, noGravity, velocity zeroed). No gate draws.
pub(crate) fn spectre_soul_burst(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..5 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            175, 0.0, 0.0, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:28412`
        d.velocity *= 0.0; // `:28413`
    }
}

/// Site 28426 — the hostile Lost Soul 293's per-tick 9× dust 175
/// (`Projectile.cs:28424-28429`): scale 1.3, noGravity, velocity zeroed.
/// No gate draws (the sound latch is the machine's).
pub(crate) fn spectre_hostile_trail(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..9 {
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            175, 0.0, 0.0, 100, Color::default(), 1.3,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:28427`
        d.velocity *= 0.0; // `:28428`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 23 — the Eye Fire / Flames Trap (`Projectile.cs:25861-25935`).
// ---------------------------------------------------------------------------

/// Site 25900 — the Eye Fire 101 / Flames Trap 188 launch-ramp dust
/// (`Projectile.cs:25875-25928`): past `ai[0] > 7`, the 8/9/10 scale
/// ladder (0.25/0.5/0.75, else 1), the counter++, then the type pick
/// (101 → 75, else 6). The `num192 == 6 || Next(2) == 0` gate (the
/// 188's 6 short-circuits it; the 101's 75 draws), one dust at v×0.2
/// alpha 100, then the `Next(3) != 0 || (75 && Next(3) == 0)`
/// noGravity/×3/×2 boost, the 188 ×1.25 vs ×1.5 scale, the ×1.2
/// velocity, the ×num191 scale, and the 75's velocity += v (+ the
/// !noGravity ×0.5). `ramp` is the PRE-increment ai[0] (the ladder
/// read); the machine increments after.
pub(crate) fn eye_fire_ramp_dust(
    p: &TrailSlot,
    kind: u16,
    ramp: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:25877-25889` — the scale ladder.
    let num191 = if ramp == 8.0 {
        0.25
    } else if ramp == 9.0 {
        0.5
    } else if ramp == 10.0 {
        0.75
    } else {
        1.0
    };
    // `:25891-25895` — the type pick.
    let num192 = if kind == 101 { 75 } else { 6 };
    // `:25896` — `num192 == 6 || Next(2) == 0`: the 188 (type 6)
    // short-circuits the gate; the 101 (type 75) draws the Next(2).
    if num192 != 6 && rng.next_range(0, 2) != 0 {
        return;
    }
    // `:25900` — the spawn (scale 1, the default).
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
        num192, p.vel.x * 0.2, p.vel.y * 0.2, 100, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    // `:25901-25907` — the boost gate: `Next(3) != 0 || (75 && Next(3)==0)`.
    // The first Next(3) always draws; the second only when the first == 0
    // AND the type is 75 (the `||` right operand's `&&`).
    let first = rng.next_range(0, 3);
    let boost = first != 0 || (num192 == 75 && rng.next_range(0, 3) == 0);
    if boost {
        d.no_gravity = true;
        d.scale *= 3.0;
        d.velocity.x *= 2.0;
        d.velocity.y *= 2.0;
    }
    // `:25908-25915` — the 188 ×1.25 vs ×1.5 scale.
    d.scale *= if kind == 188 { 1.25 } else { 1.5 };
    // `:25916-25918` — the ×1.2 velocity + the ×num191 scale.
    d.velocity.x *= 1.2;
    d.velocity.y *= 1.2;
    d.scale *= num191;
    // `:25919-25926` — the 75's velocity += v (+ the !noGravity ×0.5).
    if num192 == 75 {
        d.velocity += p.vel;
        if !d.no_gravity {
            d.velocity *= 0.5;
        }
    }
}

// ---------------------------------------------------------------------------
// aiStyle 59 — the Spectre Wrath (`Projectile.cs:28938-28993`).
// ---------------------------------------------------------------------------

/// Site 28987 — the Spectre Wrath 356's per-tick 5-point ring
/// (`Projectile.cs:28983-28992`): five dusts 175 (scale 1.3, noGravity,
/// velocity zeroed) at `position - (vx*0.2*i, -vy*0.2*i)` — the X
/// offset subtracts `vx*0.2*i`, the Y subtracts `(0 - vy*0.2)*i` (so
/// the Y trail rides +vy). No gate draws; the ring runs every tick
/// (the C# block sits after the steering, before the arm's end).
pub(crate) fn spectre_wrath_ring(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for i in 0..5 {
        let ox = p.vel.x * 0.2 * i as f32; // `:28985`
        let oy = (0.0 - p.vel.y * 0.2) * i as f32; // `:28986`
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, p.pos, p.w as i32, p.h as i32,
            175, 0.0, 0.0, 100, Color::default(), 1.3,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:28988`
        d.velocity *= 0.0; // `:28989`
        d.position.x -= ox; // `:28990`
        d.position.y -= oy; // `:28991`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 65 — the Sharknado bolt (`Projectile.cs:29534-29603`).
// ---------------------------------------------------------------------------

/// Site 29549 — the Sharknado bolt 385's post-10 six-dust ring
/// (`Projectile.cs:29544-29554`): six dusts 172 (scale 1.4, noGravity,
/// noLight) at the rotated box-edge point + the random velocity vector;
/// each draws `NextDouble()` (the angle) + `Next(3,8)` (the speed)
/// BEFORE the spawn, then velocity /= 4 and −= velocity. The C# uses
/// the f64 literal π (3.1415927410125732) for the angle and the f32
/// `Math.PI / num503` for the per-dust rotation. `center` is the bolt's
/// Centre; the machine passes the live velocity.
pub(crate) fn sharknado_bolt_ring(
    p: &TrailSlot,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num503 = 6;
    let center = p.center();
    let norm = p.vel.normalize_or_zero();
    for i in 0..num503 {
        // `:29547` — the box-edge point, rotated by (i - (6/2 - 1))·π/6.
        let edge = norm * Vec2::new(p.w / 2.0, p.h) * 0.75;
        let angle = (i - (num503 / 2 - 1)) as f32 * std::f32::consts::PI / num503 as f32;
        let vector37 = crate::projectile::rotate(edge, angle) + center;
        // `:29548` — the two argument draws: the NextDouble angle (the
        // f64 π literal) + the Next(3,8) speed.
        let a = (rng.next_double() * 3.1415927410125732) as f32 - std::f32::consts::FRAC_PI_2;
        let speed = rng.next_range(3, 8) as f32;
        let vector38 = Vec2::new(a.cos(), a.sin()) * speed;
        // `:29549` — the spawn at vector37 + vector38, speeds vector38×2.
        let idx = dust.new_dust(
            &mut DustDraw(rng), ctx, world, vector37 + vector38, 0, 0,
            172, vector38.x * 2.0, vector38.y * 2.0, 100, Color::default(), 1.4,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:29550`
        d.no_light = true; // `:29551`
        d.velocity /= 4.0; // `:29552`
        d.velocity -= p.vel; // `:29553`
    }
}

// ---------------------------------------------------------------------------
// aiStyle 68 — the Molotov cocktail (`Projectile.cs:30166-30238`).
// ---------------------------------------------------------------------------

/// Site 30202 — the Molotov 399's `alpha == 0` fire dust
/// (`Projectile.cs:30189-30207`, flag34 = type 399): the (4,−8) spin
/// point (X −4 when direction == −1) rotated by the rotation, one
/// dust 6 at `Center + spin − (5,5)`, box 4×4, scale 1.5, noGravity;
/// velocity = velocity×0.25 + Normalize(spin), then rotated by
/// −π/2·direction. No gate draws (the alpha==0 window is the gate,
/// the machine checks it). The 669's flag35 two-dust block is kind
/// > 400 — W4-g64's.
pub(crate) fn molotov_fire_dust(
    p: &TrailSlot,
    rotation: f32,
    rng: &mut dyn DropRng,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:30193-30199` — the spin point.
    let mut spin = Vec2::new(4.0, -8.0);
    if p.direction == -1 {
        spin.x = -4.0;
    }
    let spin = crate::projectile::rotate(spin, rotation);
    // `:30202` — the spawn at Center + spin − (5,5).
    let idx = dust.new_dust(
        &mut DustDraw(rng), ctx, world,
        p.center() + spin - Vec2::splat(5.0), 4, 4,
        6, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.scale = 1.5; // `:30203`
    d.no_gravity = true; // `:30204`
    // `:30205` — velocity×0.25 + Normalize(spin).
    d.velocity = d.velocity * 0.25 + spin.normalize_or_zero();
    // `:30206` — the −π/2·direction rotation.
    d.velocity = crate::projectile::rotate(
        d.velocity,
        -std::f32::consts::FRAC_PI_2 * p.direction as f32,
    );
}

// ---------------------------------------------------------------------------
// Witnesses — WRITTEN NOT RUN (code-only mode). Each mirrors the C#
// draw sequence on a recording stream + a scratch pool with the
// permissive ctx (the npc.rs `dust_ctx` convention), so the pool never
// culls and every spawn lands.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::TILE_PX;

    /// A recording stream: every `(min, max)` range draw and every
    /// float lands in order (the `projectile_ai_darts` Recorder shape).
    struct Recorder {
        ranges: Vec<(i32, i32)>,
        floats: usize,
        /// The value every range draw returns (the gate-forcing knob).
        roll: i32,
    }
    impl Recorder {
        fn new(roll: i32) -> Self {
            Self { ranges: Vec::new(), floats: 0, roll }
        }
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            self.roll
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.5
        }
    }

    fn world() -> World {
        World::new("t".into(), 200, 120)
    }

    /// The permissive ctx (the whole world on-screen, full budget) —
    /// the pool never culls, so every spawn lands and the draw count
    /// matches the live empty-pool path (the npc.rs `dust_ctx`).
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

    fn slot(kind_pos: Vec2, vel: Vec2) -> TrailSlot {
        TrailSlot {
            pos: kind_pos,
            w: 16.0,
            h: 16.0,
            vel,
            scale: 1.0,
            alpha: 0,
            direction: 1,
            old_vel: Vec2::ZERO,
        }
    }

    /// The live dust count (the sentinel excluded).
    fn live(dust: &DustPool) -> usize {
        dust.slots()[..tr_core::particles::DUST_SENTINEL]
            .iter()
            .filter(|d| d.active)
            .count()
    }

    /// W4-g63 — the 54 drip: the `Next(20)` gate draws EVERY tick; a
    /// refused gate spawns nothing, a taken gate spawns ONE dust 40
    /// whose velocity carries the v×0.1 speeds (plus the pool jitter).
    #[test]
    fn w4_g63_the_poisoned_knife_drip_gates_every_tick() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(4.0, 2.0));
        let mut dust = DustPool::new();
        // Refused gate (roll 1): one draw, no dust.
        let mut rng = Recorder::new(1);
        poisoned_knife_drip(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(rng.ranges, vec![(0, 20)]);
        assert_eq!(live(&dust), 0);
        // Taken gate (roll 0): the gate + the 7 NewDust internals
        // (2 position, 2 velocity, 1 frame row, 1 scale = 6 range
        // draws — the pressure bands are skipped below 0.5·6000).
        let mut rng = Recorder::new(0);
        poisoned_knife_drip(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(rng.ranges[0], (0, 20), "the gate first");
        assert_eq!(live(&dust), 1);
        let d = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(d.dust_type, 40);
        assert!((d.scale - 0.75).abs() < 0.2, "0.75 × the ±0.2 jitter");
    }

    /// W4-g63 — the 162 burst: 10 + 5×2 = 20 dusts, then the gore
    /// argument roll LAST; the ember pair's first dust is noGravity.
    #[test]
    fn w4_g63_the_frost_shard_burst_draws_twenty_and_the_gore_roll() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(3.0, -1.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        frost_shard_burst(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 20, "10 smoke + 5 pairs");
        assert_eq!(*rng.ranges.last().unwrap(), (61, 64), "the gore roll last");
        let types: Vec<i32> = dust.slots()[..20].iter().map(|d| d.dust_type).collect();
        assert_eq!(types[..10], [31; 10]);
        assert_eq!(types[10..], [6; 10]);
        assert!(dust.slot(10).no_gravity, "the pair's first is noGravity");
        assert!(!dust.slot(11).no_gravity, "the pair's second is not");
    }

    /// W4-g63 — the 240 burst: 7 + 3×2 = 13 dusts + the gore roll.
    #[test]
    fn w4_g63_the_dutchman_shell_burst_draws_thirteen() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(2.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        dutchman_shell_burst(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 13, "7 smoke + 3 pairs");
        assert_eq!(*rng.ranges.last().unwrap(), (61, 64));
    }

    /// W4-g63 — the 306 trail: four dust 184 at the Centre − v/4·i
    /// points, velocity zeroed, scale 0.7; the alpha > 200 gate skips
    /// whole (no draws).
    #[test]
    fn w4_g63_the_terra_beam_trail_pins_four_points() {
        let w = world();
        let c = ctx(&w);
        let mut p = slot(Vec2::new(100.0, 100.0), Vec2::new(8.0, 4.0));
        p.alpha = 200;
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        terra_beam_trail(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 4);
        // The Centre is (108, 108); the offsets are v/4·i = (2,1)·i.
        for i in 0..4 {
            let d = dust.slot(i);
            assert_eq!(d.dust_type, 184);
            assert!((d.position.x - (108.0 - 2.0 * i as f32)).abs() < 1e-4);
            assert!((d.position.y - (108.0 - 1.0 * i as f32)).abs() < 1e-4);
            assert_eq!(d.velocity, Vec2::ZERO);
            assert!((d.scale - 0.7).abs() < 1e-6);
        }
        // Above the gate: nothing, no draws.
        p.alpha = 201;
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        terra_beam_trail(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 0);
        assert!(rng.ranges.is_empty());
    }

    /// W4-g63 — the Water Bolt tail: unbounced draws NO centre-puff
    /// gate (the `&&` short-circuit), the five-point trail runs (the
    /// `||` left operand), the nested `Next(5)` sixth fires on roll 0.
    #[test]
    fn w4_g63_the_water_bolt_trail_short_circuits_like_the_csharp() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(6.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        water_bolt_trail(&p, false, &mut rng, &mut dust, &c, &w);
        // No (0,2) centre gate; the trail's nested (0,5) fires.
        assert!(!rng.ranges.contains(&(0, 2)), "the unbounced puff gate skips");
        assert_eq!(live(&dust), 6, "5 trail + the nested sixth");
        // Bounced + roll 0: the centre puff fires, then the trail gate
        // (0,5)==0 runs the trail, then the early-return gate is NOT
        // reached (the C# order: puff, trail gate, trail, sixth).
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        water_bolt_trail(&p, true, &mut rng, &mut dust, &c, &w);
        assert_eq!(rng.ranges[0], (0, 2), "the bounced puff gate first");
        assert_eq!(live(&dust), 7, "the centre puff + 5 + the sixth");
    }

    /// W4-g63 — the Aqua gravity trail's early return: bounced +
    /// `Next(2) != 0` returns TRUE after the puff gate, spawning
    /// nothing further (the C# `return` leaves AI(), not Update()).
    #[test]
    fn w4_g63_the_aqua_trail_early_return_keeps_the_draw_order() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(6.0, 0.0));
        let mut dust = DustPool::new();
        // Roll 1: the puff gate (0,2) refuses, the return gate (0,2)
        // takes the != 0 branch -> TRUE, no trail.
        let mut rng = Recorder::new(1);
        let returned = aqua_gravity_trail(&p, true, &mut rng, &mut dust, &c, &w);
        assert!(returned);
        assert_eq!(rng.ranges, vec![(0, 2), (0, 2)]);
        assert_eq!(live(&dust), 0);
        // Roll 0: the puff fires, the return gate passes, the 3-point
        // trail + the nested (0,8) sixth run.
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        let returned = aqua_gravity_trail(&p, true, &mut rng, &mut dust, &c, &w);
        assert!(!returned);
        assert_eq!(live(&dust), 5, "the puff + 3 trail + the sixth");
        assert_eq!(rng.ranges[0], (0, 2));
        assert_eq!(rng.ranges[1], (0, 2));
        assert_eq!(*rng.ranges.last().unwrap(), (0, 8));
    }

    /// W4-g63 — the ice block's placed phase: the -1 tick bursts 10
    /// (no gate draw), every later tick draws the (0,30) gate.
    #[test]
    fn w4_g63_the_ice_block_placed_burst_then_drip() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::ZERO);
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        ice_block_placed_dust(&p, true, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 10);
        assert!(rng.ranges.is_empty(), "the burst tick draws no gate");
        let mut rng = Recorder::new(1);
        ice_block_placed_dust(&p, false, &mut rng, &mut dust, &c, &w);
        assert_eq!(rng.ranges, vec![(0, 30)], "the drip gate every later tick");
        assert_eq!(live(&dust), 10, "roll 1 refuses the drip");
    }

    /// W4-g63 — the 281 bunny burst mirrors the 162 shape (20 dusts +
    /// the gore roll) at the aiStyle-49 site.
    #[test]
    fn w4_g63_the_bunny_burst_mirrors_the_frost_shard() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(3.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        bunny_burst(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 20);
        assert_eq!(*rng.ranges.last().unwrap(), (61, 64));
    }

    /// W4-g63 — the thorn fade-cross gate: alpha 168 (+5 >= 170) fires
    /// the Nettle 8× dust 7; alpha 164 (+5 = 169 < 170) skips whole.
    #[test]
    fn w4_g63_the_thorn_fade_cross_gate() {
        let w = world();
        let c = ctx(&w);
        let mut p = slot(Vec2::new(100.0, 100.0), Vec2::new(32.0, 0.0));
        p.alpha = 168;
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        thorn_fade_burst(&p, true, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 8, "the Nettle arm's 8 dust 7");
        assert!(dust.slots()[..8].iter().all(|d| d.dust_type == 7 && d.no_gravity));
        p.alpha = 164;
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        thorn_fade_burst(&p, true, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 0, "below the crossing window");
        // The Vilethorn default arm: 3× dust 18 + 1× dust 14.
        p.alpha = 165;
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        thorn_fade_burst(&p, false, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 4);
        assert_eq!(dust.slot(3).dust_type, 14);
    }

    /// W4-g63 — the 114 trident trail's two-part gate: the Next(2)
    /// short-circuits the float gate; a taken pair draws the scale
    /// argument BEFORE the spawn and the circular pair AFTER (the
    /// C# order: 2 gates + 1 scale + 6 internals + 2 circular floats).
    #[test]
    fn w4_g63_the_trident_trail_gate_order() {
        let w = world();
        let c = ctx(&w);
        let mut p = slot(Vec2::new(100.0, 100.0), Vec2::new(6.0, 0.0));
        p.old_vel = Vec2::new(5.0, 1.0);
        // Refused Next(2) (roll 1): one draw, no dust, no floats.
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(1);
        trident_trail(&p, 1.0, &mut rng, &mut dust, &c, &w);
        assert_eq!(rng.ranges, vec![(0, 2)]);
        assert_eq!(rng.floats, 0);
        assert_eq!(live(&dust), 0);
        // Taken pair (roll 0, float 0.5 + 0.2 < 1.0): the scale float,
        // the spawn, the circular pair (2 floats) — 3 floats total.
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        trident_trail(&p, 1.0, &mut rng, &mut dust, &c, &w);
        assert_eq!(rng.ranges[0], (0, 2), "the Next(2) gate first");
        assert_eq!(rng.floats, 3, "the float gate + the scale + the circular pair");
        assert_eq!(live(&dust), 1);
        assert_eq!(dust.slot(0).dust_type, 27);
    }

    /// W4-g63 — the 115 hostile latch trail: 20 dusts 27 at the
    /// velocity*(30/i)*80 offsets (i = 5..24), no gate draws.
    #[test]
    fn w4_g63_the_hostile_latch_trail_draws_twenty() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(1000.0, 1000.0), Vec2::new(1.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        trident_hostile_latch_trail(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 20);
        assert!(dust.slots()[..20].iter().all(|d| d.dust_type == 27));
        // The first offset: velocity.X * (30/5) * 80 = 480.
        let d = dust.slot(0);
        assert!((d.position.x - (1000.0 - 480.0)).abs() < 1.0, "the i=5 offset");
    }

    /// W4-g63 — the 173 enchanted sparkle: the Next(3) type pick draws
    /// FIRST (0→15, 1→57, else 58), then the spawn.
    #[test]
    fn w4_g63_the_enchanted_sparkle_type_pick() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(4.0, 0.0));
        for (roll, want) in [(0, 15), (1, 57), (2, 58)] {
            let mut dust = DustPool::new();
            let mut rng = Recorder::new(roll);
            enchanted_beam_sparkle(&p, &mut rng, &mut dust, &c, &w);
            assert_eq!(rng.ranges[0], (0, 3), "the type pick first");
            assert_eq!(dust.slot(0).dust_type, want);
        }
    }

    /// W4-g63 — the 295 inferno bolt: 8× dust 174, noGravity, the
    /// velocity ×0.5 + v×0.1; no gate draws.
    #[test]
    fn w4_g63_the_inferno_bolt_trail_draws_eight() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(10.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        inferno_bolt_trail(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 8);
        assert!(dust.slots()[..8].iter().all(|d| d.dust_type == 174 && d.no_gravity));
    }

    /// W4-g63 — the 296 inferno blast burst: each dust draws 3 argument
    /// ranges + 2 position-jitter ranges (5 per dust), the velocity is
    /// the normalized jitter.
    #[test]
    fn w4_g63_the_inferno_blast_burst_draw_order() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(0.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        inferno_blast_burst(&p, 3, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 3);
        // Per dust: (-10,11) ×2 + (3,9) the arguments, then (-10,11) ×2
        // the position jitter (the 6 NewDust internals interleave).
        let arg_ranges: Vec<_> = rng.ranges.iter().filter(|r| **r == (3, 9)).collect();
        assert_eq!(arg_ranges.len(), 3, "one speed divisor per dust");
    }

    /// W4-g63 — the 297 spectre burst (5× dust 175, scale 2) vs the 293
    /// hostile trail (9× dust 175, scale 1.3); both velocity-zeroed.
    #[test]
    fn w4_g63_the_spectre_burst_and_trail() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(3.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        spectre_soul_burst(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 5);
        assert!(dust.slots()[..5].iter().all(|d| d.dust_type == 175 && d.velocity == Vec2::ZERO));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        spectre_hostile_trail(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 9);
    }

    /// W4-g63 — the eye-fire ramp: the 188 (type 6) short-circuits the
    /// Next(2) gate (no draw), the 101 (type 75) draws it; the boost
    /// gate's first Next(3) always draws, the second only on a 0 + 75.
    #[test]
    fn w4_g63_the_eye_fire_ramp_gate_chain() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(4.0, 0.0));
        // The 188 at ramp 8: no Next(2) gate (the 6 short-circuit), the
        // first Next(3) draws (roll 0 → the boost's second Next(3) is
        // NOT drawn — the type is 6, not 75).
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        eye_fire_ramp_dust(&p, 188, 8.0, &mut rng, &mut dust, &c, &w);
        assert!(!rng.ranges.contains(&(0, 2)), "the 188 short-circuits the Next(2)");
        assert_eq!(live(&dust), 1);
        assert_eq!(dust.slot(0).dust_type, 6);
        // The 101 at ramp 8: the Next(2) gate draws (roll 0 passes),
        // the first Next(3) == 0 → the second Next(3) draws (the 75).
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        eye_fire_ramp_dust(&p, 101, 8.0, &mut rng, &mut dust, &c, &w);
        assert_eq!(rng.ranges[0], (0, 2), "the 101 draws the Next(2) gate");
        assert_eq!(live(&dust), 1);
        assert_eq!(dust.slot(0).dust_type, 75);
        // The 101's Next(2) refused (roll 1): no dust.
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(1);
        eye_fire_ramp_dust(&p, 101, 8.0, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 0);
    }

    /// W4-g63 — the 356 spectre-wrath ring: 5× dust 175 at the
    /// velocity-offset points, velocity zeroed, no gate draws.
    #[test]
    fn w4_g63_the_spectre_wrath_ring_draws_five() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(10.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        spectre_wrath_ring(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 5);
        assert!(dust.slots()[..5].iter().all(|d| d.dust_type == 175 && d.no_gravity));
        // The i=1 X offset: vx*0.2*1 = 2 → position.X = 100 - 2.
        assert!((dust.slot(1).position.x - 98.0).abs() < 1.0);
    }

    /// W4-g63 — the 385 sharknado ring: 6× dust 172, each with the
    /// NextDouble + Next(3,8) argument pair (the Next(3,8) draws 6×).
    #[test]
    fn w4_g63_the_sharknado_ring_draws_six() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(6.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        sharknado_bolt_ring(&p, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 6);
        assert!(dust.slots()[..6].iter().all(|d| d.dust_type == 172 && d.no_gravity && d.no_light));
        let speed_draws: Vec<_> = rng.ranges.iter().filter(|r| **r == (3, 8)).collect();
        assert_eq!(speed_draws.len(), 6, "one Next(3,8) per dust");
    }

    /// W4-g63 — the 399 molotov fire dust: one dust 6 at the rotated
    /// spin point, scale 1.5, noGravity; no gate draws (the alpha==0
    /// window is the machine's gate).
    #[test]
    fn w4_g63_the_molotov_fire_dust() {
        let w = world();
        let c = ctx(&w);
        let p = slot(Vec2::new(100.0, 100.0), Vec2::new(4.0, 0.0));
        let mut dust = DustPool::new();
        let mut rng = Recorder::new(0);
        molotov_fire_dust(&p, 0.0, &mut rng, &mut dust, &c, &w);
        assert_eq!(live(&dust), 1);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 6);
        assert!(d.no_gravity);
        assert!((d.scale - 1.5).abs() < 1e-6);
        // The spin point (4,-8) rotated by 0 → the spawn at
        // Center + (4,-8) - (5,5) = (108-1, 108-13) = (107, 95).
        assert!((d.position.x - 107.0).abs() < 1.0);
        assert!((d.position.y - 95.0).abs() < 1.0);
    }
}
