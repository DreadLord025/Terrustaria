//! W4-f5 — the projectile dust seam, THIRD batch (the `Dust.NewDust` sites
//! 83-125 of `Projectile.cs`, `:24406-26501`): the aiStyle-11 fairy puff
//! (`:24406`, latent), the aiStyle-12 water/ichor stream (`:24527-24625`),
//! the aiStyle-14 thrown-light family (`:24903-25290`), the aiStyle-18
//! Demon Sickle trail (`:25473`), the aiStyle-20 drill spark (`:25578`), the
//! aiStyle-21 music-note puff (`:25596`), the aiStyle-22 Ice Rod block
//! (`:25665-25756`), the aiStyle-23 Eye Fire / Flames Trap (`:25900`), the
//! aiStyle-24 Crystal Storm / Crystal Shard (`:25947-25985`), the aiStyle-25
//! boulder trail (`:26035-26398`, latent) and the aiStyle-27 caster beams
//! (`:26451-26501`).
//!
//! Co-lane split (the in-flight probe): W4-d53 owns sites 1-40 (unpushed),
//! W4-e3 owns sites 41-82 (`:22230-24336`, the aiStyle-2/4/5/6/8/152 chains,
//! in-flight in its worktree) AND the `ProjectilePool::update` ->
//! `update_dust` plumbing + the `particles.rs` generic-RNG widening. This
//! lane reproduces that plumbing byte-compatibly (the keep-one-at-merge
//! precedent, the d51/e2 co-lanes) and wires ONLY its own dispatch arms —
//! the aiStyle-11/12/14/18/20/21/22/23/24/25/27 set, disjoint from e3's
//! 2/4/5/6/8/152. The g63/g64/g65 trail/death lanes are reserved (no pushed
//! claims) and untouched.
//!
//! The seam is LOCAL (the d50 dust-type-table lane has not landed): the pool
//! threads its live `DustPool` + the frame's `SpawnCtx` through
//! `ProjectilePool::update_dust` (the `Npc::update_rng_dust` precedent,
//! `npc.rs:4622`); the plain `update` runs on a scratch pool with a permissive
//! ctx (`Npc::update_rng`, `npc.rs:4595`), so the AI-level draws still ride
//! the caller's stream and the pool's own rolls land on the scratch pool —
//! the draw COUNT matches the live empty-pool path.
//!
//! RNG: every roll (the gates, the `NewDust` internals, the post-arms) rides
//! the pass's `UpdateProjectiles` stream (`Main.cs:17775-17777`'s
//! `SwapRandom("UpdateProjectiles")` wraps the whole walk, dust included) in
//! the C#'s order. The [`ProjDustRng3`] adapter maps the pool's
//! `Next`/`NextFloat` onto the pass's [`DropRng`] (`next_max(n)` =
//! `next_range(0, n)`, one draw). Argument draws evaluate BEFORE the
//! `NewDust` call (the C# argument-evaluation order); the post-arms read the
//! returned slot (the sentinel included — the C# writes `Main.dust[6000]` on
//! a refusal).
//!
//! Documented skips (each at its site): the `Gore.NewGore` calls interleaved
//! with these dust blocks (the gore lane is d55/e5 — their `Main.rand` rolls
//! go with them, the `ai_volcano` precedent), the `shader` write (`:24408`
//! — dropped field, `particles.rs`), the `Lighting.AddLight` arms (the light
//! lane is c19/e6), and the unrowed kinds' sites (18/72/86/87 fairy-orb,
//! 196 Smoke Bomb, 261 Boulder Staff, 538 Twinkle, 1092 Librarian Book,
//! 288 hostile Golden Shower, 326-328 Greek Fire, 1048/1053 boulders,
//! 1114 Daybloom Staff) — their kinds have no `proj_data` row, so the arm
//! never reaches them; the site is recorded here and stays latent until the
//! kind row lands (the d51 ungrantered-arm precedent).

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};
use tr_core::World;

use crate::projectile::Projectile;

/// The pool's [`DustRng`] over the pass's [`DropRng`] stream: `Next(max)` is
/// `Next(0, max)` (`UnifiedRandom.cs:136-139`), one draw either way.
pub(crate) struct ProjDustRng3<'a>(pub(crate) &'a mut dyn DropRng);

impl DustRng for ProjDustRng3<'_> {
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
pub(crate) struct ProjDust3<'a> {
    pub pool: &'a mut DustPool,
    pub ctx: &'a SpawnCtx,
    pub rng: &'a mut dyn DropRng,
}

impl ProjDust3<'_> {
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
            &mut ProjDustRng3(self.rng),
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
            &mut ProjDustRng3(self.rng),
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

/// `Utils.NextVector2Unit` (`Utils.cs:1260-1263`) — one `NextFloat` angle,
/// the f64 cos/sin spin (`ToRotationVector2`).
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

// ---------------------------------------------------------------------------
// aiStyle 12 — the water/ichor stream (`Projectile.cs:24501-24635`).
// ---------------------------------------------------------------------------

/// `:24522-24540` — the 280/288 Golden Shower's `ai[0] > 3f` trail: the
/// three dust-170 (inset 14, noGravity, ×0.1 velocity + 0.5×vel, position
/// backed off by `vel/3·i`), then the 1-in-8 dust-170 sparkle (inset 16,
/// scale 0.5, ×0.25 velocity + 0.5×vel). The caller owns the `ai[0] > 3f`
/// gate and the `velocity.Y += 0.075f` arc ahead of this.
pub(crate) fn style12_golden_trail(p: &Projectile, world: &World, d: &mut ProjDust3) {
    // `:24522-24533` — the three-dust trail.
    for i in 0..3 {
        let bx = p.vel.x / 3.0 * i as f32;
        let by = p.vel.y / 3.0 * i as f32;
        let idx = d.new_dust(
            world,
            p.pos + Vec2::new(14.0, 14.0),
            p.w as i32 - 28,
            p.h as i32 - 28,
            170,
            0.0,
            0.0,
            100,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 0.1;
        dust.velocity += p.vel * 0.5;
        dust.position.x -= bx;
        dust.position.y -= by;
    }
    // `:24534-24540` — the 1-in-8 sparkle.
    if d.rng.next_range(0, 8) == 0 {
        let idx = d.new_dust(
            world,
            p.pos + Vec2::new(16.0, 16.0),
            p.w as i32 - 32,
            p.h as i32 - 32,
            170,
            0.0,
            0.0,
            100,
            Color::default(),
            0.5,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.25;
        dust.velocity += p.vel * 0.5;
    }
}

/// `:24575-24580` — the 22 Aqua Scepter's lava-bounce burst: four dust-31
/// (scale 1.5, ×0.6 velocity, Y −2). The caller owns the LCG-seeded bounce
/// and the `ai[1] = 1f` latch ahead of this; the trailing `Next(3)` gore gate
/// (`:24581-24587`) stays on the stream at the caller (the gore lane's skip).
pub(crate) fn style12_lava_burst(p: &Projectile, world: &World, d: &mut ProjDust3) {
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

/// `:24601-24602` — the 22's bounced-stream 1-in-2 dust-31 puff (centre,
/// scale 0.8, ×0.1 velocity). The caller owns the `ai[1] == 1f && Next(2) == 0`
/// gate.
pub(crate) fn style12_bounced_puff(p: &Projectile, world: &World, d: &mut ProjDust3) {
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
    d.pool.slot_mut(idx).velocity *= 0.1;
}

/// `:24608-24629` — the 22's trail: the three dust-172 (inset 6, scale 1.2,
/// noGravity, ×0.3 velocity + 0.5×vel, position backed off by `vel/3·i`),
/// then the 1-in-8 dust-172 extra (inset 6, scale 0.75, ×0.5 velocity +
/// 0.5×vel). The C# wraps both in a `for (num105 = 0; num105 < 1; ...)` loop
/// that runs once — ported as a single pass. The caller owns the
/// `ai[1] != 0f && Next(2) != 0` early-return ahead of this.
pub(crate) fn style12_aqua_trail(p: &Projectile, world: &World, d: &mut ProjDust3) {
    // `:24610-24621` — the three-dust trail.
    for i in 0..3 {
        let bx = p.vel.x / 3.0 * i as f32;
        let by = p.vel.y / 3.0 * i as f32;
        let idx = d.new_dust(
            world,
            p.pos + Vec2::new(6.0, 6.0),
            p.w as i32 - 12,
            p.h as i32 - 12,
            172,
            0.0,
            0.0,
            100,
            Color::default(),
            1.2,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 0.3;
        dust.velocity += p.vel * 0.5;
        dust.position.x -= bx;
        dust.position.y -= by;
    }
    // `:24622-24628` — the 1-in-8 extra.
    if d.rng.next_range(0, 8) == 0 {
        let idx = d.new_dust(
            world,
            p.pos + Vec2::new(6.0, 6.0),
            p.w as i32 - 12,
            p.h as i32 - 12,
            172,
            0.0,
            0.0,
            100,
            Color::default(),
            0.75,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.5;
        dust.velocity += p.vel * 0.5;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 14 — the thrown-light family (`Projectile.cs:24830-25307`).
// ---------------------------------------------------------------------------

/// `:25230-25244` — the 480 Cursed Dart flame's two dust-75: the first
/// unconditional (position −2/+2, scale +Next(50)·0.01, noGravity, Y −2),
/// then the 1-in-2 second (scale +0.3+Next(50)·0.01, noGravity, ×0.1
/// velocity). The caller owns the `alpha = 255` pin ahead of this.
pub(crate) fn style14_flame_480(p: &Projectile, world: &World, d: &mut ProjDust3) {
    // `:25230-25235` — the first dust-75.
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
    {
        let dust = d.pool.slot_mut(idx);
        dust.position.x -= 2.0;
        dust.position.y += 2.0;
        dust.scale += d.rng.next_range(0, 50) as f32 * 0.01;
        dust.no_gravity = true;
        dust.velocity.y -= 2.0;
    }
    // `:25236-25244` — the 1-in-2 second dust-75.
    if d.rng.next_range(0, 2) == 0 {
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
        dust.position.x -= 2.0;
        dust.position.y += 2.0;
        dust.scale += 0.3 + d.rng.next_range(0, 50) as f32 * 0.01;
        dust.no_gravity = true;
        dust.velocity *= 0.1;
    }
}

/// `:25282-25296` — the 400-402 Molotov fire / 1107-1109 Swordfish bone's
/// two dust-6: the first unconditional (position −2/+2, scale
/// +Next(50)·0.01, noGravity, Y −2), then the 1-in-2 second (scale
/// +0.3+Next(50)·0.01, noGravity, ×0.1 velocity). The caller owns the wet
/// kill ahead of this; the `ai[1]` sound gate (`:25277-25281`) is a different
/// kind (326-328) and never fires for these.
pub(crate) fn style14_fire_dust(p: &Projectile, world: &World, d: &mut ProjDust3) {
    // `:25282-25287` — the first dust-6.
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
    {
        let dust = d.pool.slot_mut(idx);
        dust.position.x -= 2.0;
        dust.position.y += 2.0;
        dust.scale += d.rng.next_range(0, 50) as f32 * 0.01;
        dust.no_gravity = true;
        dust.velocity.y -= 2.0;
    }
    // `:25288-25296` — the 1-in-2 second dust-6.
    if d.rng.next_range(0, 2) == 0 {
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
        dust.position.x -= 2.0;
        dust.position.y += 2.0;
        dust.scale += 0.3 + d.rng.next_range(0, 50) as f32 * 0.01;
        dust.no_gravity = true;
        dust.velocity *= 0.1;
    }
}

/// `:25257-25264` — the 326-328 Greek Fire's 2-in-3 dust-6 Direct: the
/// `Next(3) != 0` gate, then a `NewDustDirect` (alpha 0, scale 1) whose
/// velocity.Y −2, noGravity, scale +NextFloat·0.8+0.3, velocity +1×vel.
/// The caller owns the wet kill and the `ai[1]` sound latch ahead of this.
pub(crate) fn style14_greek_fire(p: &Projectile, world: &World, d: &mut ProjDust3) {
    if d.rng.next_range(0, 3) != 0 {
        let idx = d.new_dust_direct(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            6,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity.y -= 2.0;
        dust.no_gravity = true;
        dust.scale += d.rng.next_float() * 0.8 + 0.3;
        dust.velocity += p.vel;
    }
}

/// `:25195-25212` — the 450 Saucer Scrap's two dust-228: the 1-in-2 first
/// (position −2/+2, scale +Next(50)·0.01, noGravity, Y −2), then the 1-in-4
/// second (scale +0.3+Next(50)·0.01, noGravity, ×0.1 velocity). The caller
/// owns the `ai[1]` sound latch ahead of this.
pub(crate) fn style14_saucer_450(p: &Projectile, world: &World, d: &mut ProjDust3) {
    // `:25195-25203` — the 1-in-2 first dust-228.
    if d.rng.next_range(0, 2) == 0 {
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
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.position.x -= 2.0;
        dust.position.y += 2.0;
        dust.scale += d.rng.next_range(0, 50) as f32 * 0.01;
        dust.no_gravity = true;
        dust.velocity.y -= 2.0;
    }
    // `:25204-25212` — the 1-in-4 second dust-228.
    if d.rng.next_range(0, 4) == 0 {
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
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.position.x -= 2.0;
        dust.position.y += 2.0;
        dust.scale += 0.3 + d.rng.next_range(0, 50) as f32 * 0.01;
        dust.no_gravity = true;
        dust.velocity *= 0.1;
    }
}