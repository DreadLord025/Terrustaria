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

/// The permissive dust ctx for the witness fixtures (the whole world
/// on-screen, the full 6000 budget — the scratch pool never culls, so the
/// draw count matches the live empty-pool path; the `Npc::update_rng`
/// convention, `npc.rs:4595`).
#[cfg(test)]
pub(crate) fn test_dust_ctx(world: &World) -> SpawnCtx {
    SpawnCtx {
        game_menu: false,
        game_paused: false,
        generating: false,
        screen_position: Vec2::ZERO,
        screen_size: Vec2::new(
            world.width as f32 * tr_core::TILE_PX,
            world.height as f32 * tr_core::TILE_PX,
        ),
        max_dust_to_draw: 6000,
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

// ---------------------------------------------------------------------------
// aiStyle 18 — the Demon Sickle / Scythe (`Projectile.cs:25367-25476`).
// ---------------------------------------------------------------------------

/// `:25471-25475` — the 44/45's two dust-27 (noGravity). The caller owns the
/// rotation/acceleration ahead of this; the 263/274 early-return kinds never
/// reach it (unrowed).
pub(crate) fn style18_scythe_trail(p: &Projectile, world: &World, d: &mut ProjDust3) {
    for _ in 0..2 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            27,
            0.0,
            0.0,
            100,
            Color::default(),
            1.0,
        );
        d.pool.slot_mut(idx).no_gravity = true;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 20 — the drill / chainsaw (`Projectile.cs:25481-25584`).
// ---------------------------------------------------------------------------

/// `:25576-25583` — the 1-in-6 drill spark: a dust-31 at
/// `position + velocity·Next(6,10)·0.1` (the offset draw evaluates BEFORE the
/// `NewDust`), alpha 80, scale 1.4; post-arms position.X −4, noGravity,
/// ×0.2 velocity, then velocity.Y = −Next(7,13)·0.15. The caller owns the
/// `velocity.X *= 1 + Next(-3,4)·0.01` jitter ahead of this.
pub(crate) fn style20_drill_spark(p: &Projectile, world: &World, d: &mut ProjDust3) {
    if d.rng.next_range(0, 6) == 0 {
        // `:25578` — the position offset's `Next(6,10)` draws first.
        let off = d.rng.next_range(6, 10) as f32 * 0.1;
        let idx = d.new_dust(
            world,
            p.pos + p.vel * off,
            p.w as i32,
            p.h as i32,
            31,
            0.0,
            0.0,
            80,
            Color::default(),
            1.4,
        );
        let dust = d.pool.slot_mut(idx);
        dust.position.x -= 4.0;
        dust.no_gravity = true;
        dust.velocity *= 0.2;
        dust.velocity.y = -(d.rng.next_range(7, 13) as f32) * 0.15;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 21 — the music notes (`Projectile.cs:25585-25606`).
// ---------------------------------------------------------------------------

/// `:25594-25599` — the 76/77/78's 1-in-3 dust-27 puff (alpha 80, noGravity,
/// ×0.2 velocity). The caller owns the decel ahead of this.
pub(crate) fn style21_note_puff(p: &Projectile, world: &World, d: &mut ProjDust3) {
    if d.rng.next_range(0, 3) == 0 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            27,
            0.0,
            0.0,
            80,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 0.2;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 22 — the Ice Rod block (`Projectile.cs:25607-25860`).
// ---------------------------------------------------------------------------

/// `:25665-25667` — the standing phase's dust-67 (noGravity, ×0.3 velocity).
/// The caller owns the probe kill scan ahead of this; the C# `Kill()` does
/// NOT return, so the dust still spawns on the dying tick.
pub(crate) fn style22_standing(p: &Projectile, world: &World, d: &mut ProjDust3) {
    let idx = d.new_dust(
        world,
        p.pos,
        p.w as i32,
        p.h as i32,
        67,
        0.0,
        0.0,
        0,
        Color::default(),
        1.0,
    );
    let dust = d.pool.slot_mut(idx);
    dust.no_gravity = true;
    dust.velocity *= 0.3;
}

/// `:25672-25685` — the placed phase's dust: the `ai[0] == -1f` placement
/// tick's 10× dust-67 burst (alpha 0, scale 1.1, noGravity, ×1.3 velocity),
/// else the 1-in-30 dust-67 puff (alpha 100, ×0.2 velocity). The caller owns
/// the `ai[0] < 0f` gate ahead of this.
pub(crate) fn style22_placed(p: &Projectile, world: &World, d: &mut ProjDust3) {
    if p.ai[0] == -1.0 {
        for _ in 0..10 {
            let idx = d.new_dust(
                world,
                p.pos,
                p.w as i32,
                p.h as i32,
                67,
                0.0,
                0.0,
                0,
                Color::default(),
                1.1,
            );
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.velocity *= 1.3;
        }
    } else if d.rng.next_range(0, 30) == 0 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            67,
            0.0,
            0.0,
            100,
            Color::default(),
            1.0,
        );
        d.pool.slot_mut(idx).velocity *= 0.2;
    }
}

/// `:25756-25758` — the flying phase's dust-67 (noGravity, ×0.3 velocity).
/// The caller owns the probe kill scan + the `!active` return ahead of this.
pub(crate) fn style22_flying(p: &Projectile, world: &World, d: &mut ProjDust3) {
    let idx = d.new_dust(
        world,
        p.pos,
        p.w as i32,
        p.h as i32,
        67,
        0.0,
        0.0,
        0,
        Color::default(),
        1.0,
    );
    let dust = d.pool.slot_mut(idx);
    dust.no_gravity = true;
    dust.velocity *= 0.3;
}

// ---------------------------------------------------------------------------
// aiStyle 23 — the Eye Fire / Flames Trap (`Projectile.cs:25861-25935`).
// ---------------------------------------------------------------------------

/// `:25896-25928` — the 101/188's launch dust: the `num192 == 6 || Next(2) == 0`
/// gate (the 188's 6 always fires, the 101's 75 is 1-in-2), then one dust at
/// `velocity·0.2` speed, alpha 100; the `Next(3) != 0 || (75 && Next(3) == 0)`
/// post-gate scales ×3 + velocity ×2 + noGravity, the 188 ×1.25 / else ×1.5
/// scale, the ×1.2 velocity, the `num191` launch-ramp scale, and the 75's
/// `+velocity` (×0.5 if still gravity-bound). The caller owns the `ai[0]`
/// ramp (8→0.25, 9→0.5, 10→0.75, 11+→1.0) and the `ai[0]++` ahead of this.
pub(crate) fn style23_eye_fire(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust3,
    num191: f32,
) {
    // `:25891-25895` — the dust type: 75 for the 101 Eye Fire, else 6 (the
    // 188 Flames Trap).
    let num192 = if p.kind == crate::projectile::EYE_FIRE { 75 } else { 6 };
    // `:25896` — the 6 always fires; the 75 is 1-in-2.
    if num192 == 6 || d.rng.next_range(0, 2) == 0 {
        // `:25900` — the dust at velocity·0.2.
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            num192,
            p.vel.x * 0.2,
            p.vel.y * 0.2,
            100,
            Color::default(),
            1.0,
        );
        // `:25901-25907` — the noGravity/scale/velocity post-gate.
        if d.rng.next_range(0, 3) != 0 || (num192 == 75 && d.rng.next_range(0, 3) == 0) {
            let dust = d.pool.slot_mut(idx);
            dust.no_gravity = true;
            dust.scale *= 3.0;
            dust.velocity.x *= 2.0;
            dust.velocity.y *= 2.0;
        }
        // `:25908-25915` — the 188 ×1.25 / else ×1.5 scale.
        {
            let dust = d.pool.slot_mut(idx);
            if p.kind == crate::projectile::FLAMES_TRAP {
                dust.scale *= 1.25;
            } else {
                dust.scale *= 1.5;
            }
            // `:25916-25918` — the ×1.2 velocity + the launch-ramp scale.
            dust.velocity.x *= 1.2;
            dust.velocity.y *= 1.2;
            dust.scale *= num191;
        }
        // `:25919-25926` — the 75's +velocity (×0.5 if still gravity-bound).
        if num192 == 75 {
            let dust = d.pool.slot_mut(idx);
            dust.velocity += p.vel;
            if !dust.no_gravity {
                dust.velocity *= 0.5;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// aiStyle 24 — the Crystal Storm / Crystal Shard (`Projectile.cs:25936-26014`).
// ---------------------------------------------------------------------------

/// `:25945-25951` — the 1134 Crystal Shard's first-tick 6× dust-70 burst
/// (noGravity, ×2.5 velocity, ×0.9 scale). The caller owns the `localAI[0]`
/// latch + the sound cue ahead of this.
pub(crate) fn style24_shard_burst(p: &Projectile, world: &World, d: &mut ProjDust3) {
    for _ in 0..6 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            70,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 2.5;
        dust.scale *= 0.9;
    }
}

/// `:25958-25964` — the 94 Crystal Storm's 1-in-4 dust-70 puff (noGravity,
/// ×0.5 velocity, ×0.9 scale). The caller owns the `ai[1]++` ahead of this.
pub(crate) fn style24_storm_puff(p: &Projectile, world: &World, d: &mut ProjDust3) {
    if d.rng.next_range(0, 4) == 0 {
        let idx = d.new_dust(
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            70,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let dust = d.pool.slot_mut(idx);
        dust.no_gravity = true;
        dust.velocity *= 0.5;
        dust.scale *= 0.9;
    }
}

// ---------------------------------------------------------------------------
// aiStyle 27 — the caster beams (`Projectile.cs:26420-26554`).
// ---------------------------------------------------------------------------

/// `:26449-26456` — the 114 Unholy Trident's fade dust: the
/// `Next(2) == 0 && NextFloat + 0.2 < num211` gate, then a dust-27 at
/// `(position.X, position.Y + 4)`, 8×8, `oldVelocity` speed, alpha 100,
/// scale `1 + 0.6·NextFloat`; post-arms ×−0.5 velocity, position += vel·5,
/// position += NextVector2Circular(16,16), ×−1 velocity. The caller owns the
/// `num211` Remap ahead of this.
pub(crate) fn style27_trident_fade(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust3,
    old_vel: Vec2,
    num211: f32,
) {
    if d.rng.next_range(0, 2) == 0 && d.rng.next_float() + 0.2 < num211 {
        // `:26451` — the scale argument's NextFloat draws BEFORE NewDust.
        let scale = 1.0 + 0.6 * d.rng.next_float();
        let idx = d.new_dust(
            world,
            Vec2::new(p.pos.x, p.pos.y + 4.0),
            8,
            8,
            27,
            old_vel.x,
            old_vel.y,
            100,
            Color::default(),
            scale,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= -0.5;
        dust.position += dust.velocity * 5.0;
        dust.position += next_vector2_circular(d.rng, 16.0, 16.0);
        dust.velocity *= -1.0;
    }
}

/// `:26461-26468` — the 115 hostile twin's first-hit 20-step dust-27 trail:
/// for `num213` in 5..25, a dust at `position − velocity·(30/num213)·80`,
/// 8×8, `oldVelocity` speed, alpha 100, scale 0.9; post-arms ×0.25 velocity,
/// velocity −= velocity·5. The caller owns the `localAI[1] < 5f` latch.
pub(crate) fn style27_hostile_trail(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust3,
    old_vel: Vec2,
) {
    for num213 in 5..25 {
        let bx = p.vel.x * (30.0 / num213 as f32);
        let by = p.vel.y * (30.0 / num213 as f32);
        let idx = d.new_dust(
            world,
            Vec2::new(p.pos.x - bx * 80.0, p.pos.y - by * 80.0),
            8,
            8,
            27,
            old_vel.x,
            old_vel.y,
            100,
            Color::default(),
            0.9,
        );
        let dust = d.pool.slot_mut(idx);
        dust.velocity *= 0.25;
        dust.velocity -= p.vel * 5.0;
    }
}

/// `:26470-26479` — the 173 Enchanted Beam's `localAI[1] > 7f` dust: the
/// `Next(3)` type pick (0→15, 1→57, else→58) draws BEFORE NewDust, at
/// `position − velocity·4 + (2,2)`, 8×8, alpha 100, scale 1.25; post-arm
/// ×0.1 velocity.
pub(crate) fn style27_enchanted(p: &Projectile, world: &World, d: &mut ProjDust3) {
    let dust_type = match d.rng.next_range(0, 3) {
        0 => 15,
        1 => 57,
        _ => 58,
    };
    let idx = d.new_dust(
        world,
        Vec2::new(p.pos.x - p.vel.x * 4.0 + 2.0, p.pos.y + 2.0 - p.vel.y * 4.0),
        8,
        8,
        dust_type,
        0.0,
        0.0,
        100,
        Color::default(),
        1.25,
    );
    d.pool.slot_mut(idx).velocity *= 0.1;
}

/// `:26480-26487` — the 132 Terra Beam's `localAI[1] > 7f` two dust-107: each
/// at `position − velocity·4 + (2,2)`, 8×8, `oldVelocity` speed, alpha 100,
/// scale 1.25, ×−0.25 velocity; the second also position −= velocity·0.5.
pub(crate) fn style27_terra(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust3,
    old_vel: Vec2,
) {
    let pos = Vec2::new(p.pos.x - p.vel.x * 4.0 + 2.0, p.pos.y + 2.0 - p.vel.y * 4.0);
    let idx = d.new_dust(world, pos, 8, 8, 107, old_vel.x, old_vel.y, 100, Color::default(), 1.25);
    d.pool.slot_mut(idx).velocity *= -0.25;
    let idx = d.new_dust(world, pos, 8, 8, 107, old_vel.x, old_vel.y, 100, Color::default(), 1.25);
    let dust = d.pool.slot_mut(idx);
    dust.velocity *= -0.25;
    dust.position -= p.vel * 0.5;
}

/// `:26494-26498` — the 114/115's `localAI[1] >= 15f` pulse dust-27: at
/// `(position.X, position.Y + 4)`, 8×8, `oldVelocity` speed, alpha 100,
/// scale 0.6, ×−0.25 velocity.
pub(crate) fn style27_trident_pulse(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust3,
    old_vel: Vec2,
) {
    let idx = d.new_dust(
        world,
        Vec2::new(p.pos.x, p.pos.y + 4.0),
        8,
        8,
        27,
        old_vel.x,
        old_vel.y,
        100,
        Color::default(),
        0.6,
    );
    d.pool.slot_mut(idx).velocity *= -0.25;
}

/// `:26499-26504` — the 116 Sword Beam's `localAI[1] >= 15f` pulse dust-64:
/// at `position − velocity·5 + (2,2)`, 8×8, `oldVelocity` speed, alpha 100,
/// scale 1.5, ×−0.25 velocity, noGravity.
pub(crate) fn style27_sword_pulse(
    p: &Projectile,
    world: &World,
    d: &mut ProjDust3,
    old_vel: Vec2,
) {
    let idx = d.new_dust(
        world,
        Vec2::new(p.pos.x - p.vel.x * 5.0 + 2.0, p.pos.y + 2.0 - p.vel.y * 5.0),
        8,
        8,
        64,
        old_vel.x,
        old_vel.y,
        100,
        Color::default(),
        1.5,
    );
    let dust = d.pool.slot_mut(idx);
    dust.velocity *= -0.25;
    dust.no_gravity = true;
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
// =====================================================================
// Witness tests - WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines; the RNG
// order is asserted against a scripted stream (the d51/e2 draw-mirror
// convention). The scratch pool + permissive ctx never cull, so the draw
// count matches the live empty-pool path.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, Projectile};
    use tr_core::TILE_PX;

    /// A scripted stream: `ranges` is consumed in order (the gate / argument
    /// / NewDust-internal draws), `floats` likewise; every draw is recorded.
    struct Script {
        ranges: Vec<i32>,
        floats: Vec<f32>,
        ri: usize,
        fi: usize,
        log: Vec<String>,
    }
    impl Script {
        fn new(ranges: Vec<i32>, floats: Vec<f32>) -> Self {
            Self { ranges, floats, ri: 0, fi: 0, log: Vec::new() }
        }
    }
    impl DropRng for Script {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.log.push(format!("range({min},{max})"));
            let v = self.ranges[self.ri];
            self.ri += 1;
            v
        }
        fn next_float(&mut self) -> f32 {
            self.log.push("float".to_string());
            let v = self.floats[self.fi];
            self.fi += 1;
            v
        }
    }

    fn world() -> World {
        World::new("f5".into(), 512, 512)
    }

    fn proj(kind: u16, vx: f32, vy: f32) -> Projectile {
        let dd = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = dd.w;
        p.h = dd.h;
        p.scale = dd.scale;
        p.alpha = dd.alpha;
        p.time_left = dd.time_left;
        p
    }

    fn active(pool: &DustPool) -> usize {
        (0..6000).filter(|&i| pool.slot(i).active).count()
    }

    /// `:24522-24540` - the Golden Shower trail: 3x dust-170 (inset 14,
    /// noGravity, x0.1 vel + 0.5xvel, position backed off vel/3*i), then the
    /// 1-in-8 sparkle (inset 16, scale 0.5, x0.25 vel + 0.5xvel). The gate
    /// returns 0 (taken): 3 NewDust (6 draws each) + 1 gate + 1 NewDust.
    #[test]
    fn w4_f5_style12_golden_trail_spawns_three_plus_sparkle() {
        let w = world();
        let p = proj(crate::projectile::GOLDEN_SHOWER, 3.0, 1.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        // 3 trail NewDust (pos.x, pos.y, vel.x, vel.y, frame, scale) x3,
        // the Next(8) gate = 0, the sparkle NewDust x6.
        let ranges = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut s = Script::new(ranges, vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style12_golden_trail(&p, &w, &mut d);
        assert_eq!(active(&pool), 4, "3 trail + 1 sparkle");
        // The first trail dust: type 170, noGravity, velocity x0.1 + 0.5xvel.
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 170);
        assert!(t.no_gravity);
        assert!((t.velocity.x - (0.0 * 0.1 + 3.0 * 0.5)).abs() < 1e-5);
        assert!((t.velocity.y - (0.0 * 0.1 + 1.0 * 0.5)).abs() < 1e-5);
        // The sparkle (slot 3): scale 0.5 base, x0.25 vel + 0.5xvel.
        let sp = pool.slot(3);
        assert!((sp.scale - 0.5).abs() < 1e-5, "the sparkle scale arg 0.5");
        assert!((sp.velocity.x - (0.0 * 0.25 + 3.0 * 0.5)).abs() < 1e-5);
        // The gate draw is the 19th range (after 3x6 = 18 NewDust draws).
        assert_eq!(s.log[18], "range(0,8)");
    }

    /// `:24575-24580` - the lava bounce: 4x dust-31 (scale 1.5, x0.6 vel,
    /// Y -2). No gate draws.
    #[test]
    fn w4_f5_style12_lava_burst_spawns_four() {
        let w = world();
        let p = proj(crate::projectile::AQUA_SCEPTER, 2.0, 3.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 24], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style12_lava_burst(&p, &w, &mut d);
        assert_eq!(active(&pool), 4);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 31);
        assert!((t.scale - 1.5).abs() < 1e-5);
        assert!((t.velocity.x - 0.0 * 0.6).abs() < 1e-5);
        assert!((t.velocity.y - (0.0 * 0.6 - 2.0)).abs() < 1e-5);
    }

    /// `:24601-24602` - the bounced puff: one dust-31 at centre, scale 0.8,
    /// x0.1 velocity.
    #[test]
    fn w4_f5_style12_bounced_puff_is_one_centre_dust() {
        let w = world();
        let p = proj(crate::projectile::AQUA_SCEPTER, 4.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 6], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style12_bounced_puff(&p, &w, &mut d);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 31);
        assert!((t.scale - 0.8).abs() < 1e-5);
        // NewDustPerfect-style: position = centre (the 0x0 box jitter is +4).
        let (cx, cy) = p.center();
        assert!((t.position.x - cx).abs() < 5.0);
        assert!((t.position.y - cy).abs() < 5.0);
    }

    /// `:25230-25244` - the 480 flame: first dust-75 unconditional (scale
    /// +Next(50)*0.01, Y -2), second 1-in-2 (scale +0.3+Next(50)*0.01, x0.1
    /// vel). Gate returns 0 (taken). Type 75 carries the NewDust per-type
    /// tweak (a 7th draw: velocity.y override, x0.3 vel.x, x0.7 scale).
    #[test]
    fn w4_f5_style14_flame_480_spawns_two_on_taken_gate() {
        let w = world();
        let p = proj(480, 1.0, 1.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        // first NewDust (6 + 1 tweak) + Next(50) scale + Next(2) gate=0 +
        // second NewDust (6 + 1 tweak) + Next(50) scale.
        let mut s = Script::new(vec![0,0,0,0,0,0,0, 25, 0, 0,0,0,0,0,0,0, 25], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style14_flame_480(&p, &w, &mut d);
        assert_eq!(active(&pool), 2);
        assert_eq!(pool.slot(0).dust_type, 75);
        assert_eq!(pool.slot(1).dust_type, 75);
        // first scale = (1 + 0)*0.7 + 25*0.01 = 0.95; second = 0.7 + 0.3 + 0.25 = 1.25.
        assert!((pool.slot(0).scale - 0.95).abs() < 1e-5);
        assert!((pool.slot(1).scale - 1.25).abs() < 1e-5);
        assert!(pool.slot(0).no_gravity);
        assert!(pool.slot(1).no_gravity);
    }

    /// `:25257-25264` - the Greek Fire: 2-in-3 dust-6 Direct (gate != 0),
    /// scale +NextFloat*0.8+0.3, velocity +1xvel, Y -2. Type 6 carries the
    /// NewDust per-type tweak (a 7th draw).
    #[test]
    fn w4_f5_style14_greek_fire_spawns_on_two_thirds() {
        let w = world();
        let p = proj(crate::projectile::GREEK_FIRE_1, 2.0, 0.5);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        // Next(3) gate = 1 (taken, != 0), NewDust (6 + 1 tweak), NextFloat scale.
        let mut s = Script::new(vec![1, 0,0,0,0,0,0,0], vec![0.5]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style14_greek_fire(&p, &w, &mut d);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 6);
        assert!(t.no_gravity);
        // scale = (1 + 0)*0.7 + 0.5*0.8 + 0.3 = 1.4.
        assert!((t.scale - 1.4).abs() < 1e-5);
        // velocity: tweak vel.x = 0*0.3 = 0, vel.y = 0; post Y -2 then + vel:
        // x = 0 + 2 = 2, y = (0 - 2) + 0.5 = -1.5.
        assert!((t.velocity.x - 2.0).abs() < 1e-5);
        assert!((t.velocity.y - (-1.5)).abs() < 1e-5);
        // The gate is the first draw.
        assert_eq!(s.log[0], "range(0,3)");
    }

    /// `:25257` - the Greek Fire gate miss (Next(3) == 0) spawns nothing.
    #[test]
    fn w4_f5_style14_greek_fire_misses_on_zero() {
        let w = world();
        let p = proj(crate::projectile::GREEK_FIRE_1, 2.0, 0.5);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style14_greek_fire(&p, &w, &mut d);
        assert_eq!(active(&pool), 0);
        assert_eq!(s.log.len(), 1, "only the gate draws");
    }

    /// `:25471-25475` - the Demon Sickle trail: 2x dust-27 (noGravity).
    /// Type 27 has no per-type tweak (6 draws each).
    #[test]
    fn w4_f5_style18_scythe_trail_spawns_two() {
        let w = world();
        let p = proj(crate::projectile::DEMON_SICKLE, 3.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 12], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style18_scythe_trail(&p, &w, &mut d);
        assert_eq!(active(&pool), 2);
        assert_eq!(pool.slot(0).dust_type, 27);
        assert!(pool.slot(0).no_gravity);
        assert!(pool.slot(1).no_gravity);
    }

    /// `:25576-25583` - the drill spark: 1-in-6 gate, the position offset
    /// Next(6,10) draws BEFORE NewDust, then the post-arms (X -4, noGravity,
    /// x0.2 vel, Y = -Next(7,13)*0.15). Type 31 has no tweak.
    #[test]
    fn w4_f5_style20_drill_spark_on_taken_gate() {
        let w = world();
        let p = proj(crate::projectile::DRILL_COBALT, 5.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        // Next(6) gate=0, Next(6,10) offset=8, NewDust (6), Next(7,13)=10.
        let mut s = Script::new(vec![0, 8, 0,0,0,0,0,0, 10], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style20_drill_spark(&p, &w, &mut d);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 31);
        assert!(t.no_gravity);
        assert!((t.scale - 1.4).abs() < 1e-5);
        // velocity.Y = -10*0.15 = -1.5.
        assert!((t.velocity.y - (-1.5)).abs() < 1e-5);
        // The offset draw is the 2nd, the Y draw is the last.
        assert_eq!(s.log[1], "range(6,10)");
        assert_eq!(s.log[8], "range(7,13)");
    }

    /// `:25576` - the drill spark gate miss (Next(6) != 0) spawns nothing.
    #[test]
    fn w4_f5_style20_drill_spark_misses() {
        let w = world();
        let p = proj(crate::projectile::DRILL_COBALT, 5.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![1], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style20_drill_spark(&p, &w, &mut d);
        assert_eq!(active(&pool), 0);
        assert_eq!(s.log.len(), 1);
    }

    /// `:25594-25599` - the music note puff: 1-in-3 dust-27 (alpha 80,
    /// noGravity, x0.2 vel).
    #[test]
    fn w4_f5_style21_note_puff_on_taken_gate() {
        let w = world();
        let p = proj(crate::projectile::QUARTER_NOTE, 2.0, 1.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0, 0,0,0,0,0,0], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style21_note_puff(&p, &w, &mut d);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 27);
        assert_eq!(t.alpha, 80);
        assert!(t.no_gravity);
        assert!((t.velocity.x - 0.0 * 0.2).abs() < 1e-5);
    }

    /// `:25665-25667` - the Ice Rod standing dust-67 (noGravity, x0.3 vel).
    #[test]
    fn w4_f5_style22_standing_is_one_dust67() {
        let w = world();
        let p = proj(crate::projectile::ICE_BLOCK, 1.0, 1.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 6], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style22_standing(&p, &w, &mut d);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 67);
        assert!(t.no_gravity);
        assert!((t.velocity.x - 0.0 * 0.3).abs() < 1e-5);
    }

    /// `:25672-25679` - the Ice Rod placement tick (ai[0] == -1): 10x dust-67
    /// burst (alpha 0, scale 1.1, noGravity, x1.3 vel).
    #[test]
    fn w4_f5_style22_placed_bursts_ten_on_minus_one() {
        let w = world();
        let mut p = proj(crate::projectile::ICE_BLOCK, 0.0, 0.0);
        p.ai[0] = -1.0;
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 60], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style22_placed(&p, &w, &mut d);
        assert_eq!(active(&pool), 10);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 67);
        assert!((t.scale - 1.1).abs() < 1e-5);
        assert!(t.no_gravity);
    }

    /// `:25681-25685` - the Ice Rod placed puff (ai[0] != -1): 1-in-30
    /// dust-67 (alpha 100, x0.2 vel). Gate returns 0 (taken).
    #[test]
    fn w4_f5_style22_placed_puff_on_thirtieth() {
        let w = world();
        let mut p = proj(crate::projectile::ICE_BLOCK, 0.0, 0.0);
        p.ai[0] = -5.0;
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0, 0,0,0,0,0,0], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style22_placed(&p, &w, &mut d);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.alpha, 100);
        assert!((t.velocity.x - 0.0 * 0.2).abs() < 1e-5);
        assert_eq!(s.log[0], "range(0,30)");
    }

    /// `:25945-25951` - the Crystal Shard first-tick burst: 6x dust-70
    /// (noGravity, x2.5 vel, x0.9 scale).
    #[test]
    fn w4_f5_style24_shard_burst_spawns_six() {
        let w = world();
        let p = proj(crate::projectile::CRYSTAL_SHARD_MELEE, 2.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 36], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style24_shard_burst(&p, &w, &mut d);
        assert_eq!(active(&pool), 6);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 70);
        assert!(t.no_gravity);
        assert!((t.scale - 0.9).abs() < 1e-5);
        assert!((t.velocity.x - 0.0 * 2.5).abs() < 1e-5);
    }

    /// `:25958-25964` - the Crystal Storm 1-in-4 dust-70 puff (noGravity,
    /// x0.5 vel, x0.9 scale).
    #[test]
    fn w4_f5_style24_storm_puff_on_fourth() {
        let w = world();
        let p = proj(crate::projectile::CRYSTAL_STORM, 2.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0, 0,0,0,0,0,0], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style24_storm_puff(&p, &w, &mut d);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 70);
        assert!((t.scale - 0.9).abs() < 1e-5);
        assert!((t.velocity.x - 0.0 * 0.5).abs() < 1e-5);
        assert_eq!(s.log[0], "range(0,4)");
    }

    /// `:25896-25928` - the Eye Fire (101 -> dust-75, 1-in-2 gate) launch
    /// dust at num191 = 1.0. The Next(2) gate=0 (taken), NewDust (6+1 tweak),
    /// the Next(3) post-gate=1 (!= 0, taken: noGravity, x3 scale, x2 vel),
    /// the x1.5 scale (non-188), x1.2 vel, x1.0 ramp, +vel (75).
    #[test]
    fn w4_f5_style23_eye_fire_seventy_five_on_taken_gates() {
        let w = world();
        let p = proj(crate::projectile::EYE_FIRE, 2.0, 1.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        // Next(2)=0, NewDust(7), Next(3)=1 (post-gate taken).
        let mut s = Script::new(vec![0, 0,0,0,0,0,0,0, 1], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style23_eye_fire(&p, &w, &mut d, 1.0);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 75);
        assert!(t.no_gravity, "the post-gate taken sets noGravity");
        // scale: (1)*0.7 [tweak] *3 [post] *1.5 [non-188] *1.0 [ramp] = 3.15.
        assert!((t.scale - 3.15).abs() < 1e-4);
        // The first gate is Next(2), the post-gate is Next(3).
        assert_eq!(s.log[0], "range(0,2)");
        assert_eq!(s.log[8], "range(0,3)");
    }

    /// `:25896` - the Flames Trap (188 -> dust-6) always fires (num192 == 6,
    /// no Next(2) gate). The post-gate Next(3)=0 with num192 != 75 means the
    /// noGravity branch is NOT taken (0 != 0 is false, and the 75 clause is
    /// false), so the dust keeps gravity.
    #[test]
    fn w4_f5_style23_flames_trap_six_always_fires() {
        let w = world();
        let p = proj(crate::projectile::FLAMES_TRAP, 2.0, 1.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        // No Next(2) gate (num192 == 6). NewDust(7), Next(3)=0 (post-gate
        // NOT taken: 0 != 0 false, 75 clause false).
        let mut s = Script::new(vec![0,0,0,0,0,0,0, 0], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style23_eye_fire(&p, &w, &mut d, 0.25);
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 6);
        assert!(!t.no_gravity, "the post-gate missed keeps gravity");
        // scale: (1)*0.7 [tweak] *1.25 [188] *0.25 [ramp] = 0.21875.
        assert!((t.scale - 0.21875).abs() < 1e-4);
        // The first draw is NewDust's pos.x (no Next(2) gate).
        assert_eq!(s.log[0], "range(0,10)");
    }

    /// `:26461-26468` - the hostile Trident's 20-step dust-27 trail
    /// (num213 = 5..25). Type 27 has no tweak (6 draws each = 120).
    #[test]
    fn w4_f5_style27_hostile_trail_spawns_twenty() {
        let w = world();
        let p = proj(crate::projectile::UNHOLY_TRIDENT_HOSTILE, 4.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 120], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style27_hostile_trail(&p, &w, &mut d, Vec2::new(4.0, 0.0));
        assert_eq!(active(&pool), 20);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 27);
        assert!((t.scale - 0.9).abs() < 1e-5);
        // velocity = (oldVel + jitter)*0.25 - vel*5: x = (4+0)*0.25 - 4*5 = -19.
        assert!((t.velocity.x - (-19.0)).abs() < 1e-4);
    }

    /// `:26470-26479` - the Enchanted Beam's Next(3) type pick (0->15,
    /// 1->57, 2->58). The pick draws BEFORE NewDust.
    #[test]
    fn w4_f5_style27_enchanted_type_pick() {
        let w = world();
        let p = proj(crate::projectile::ENCHANTED_BEAM, 3.0, 0.0);
        for (pick, want) in [(0, 15), (1, 57), (2, 58)] {
            let mut pool = DustPool::new();
            let ctx = test_dust_ctx(&w);
            let mut s = Script::new(vec![pick, 0,0,0,0,0,0], vec![]);
            let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
            style27_enchanted(&p, &w, &mut d);
            assert_eq!(active(&pool), 1);
            assert_eq!(pool.slot(0).dust_type, want, "pick {pick} -> {want}");
            assert!((pool.slot(0).scale - 1.25).abs() < 1e-5);
            assert_eq!(s.log[0], "range(0,3)");
        }
    }

    /// `:26480-26487` - the Terra Beam's two dust-107 (x-0.25 vel, the
    /// second also position -= vel*0.5).
    #[test]
    fn w4_f5_style27_terra_spawns_two() {
        let w = world();
        let p = proj(crate::projectile::TERRA_BEAM, 3.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 12], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style27_terra(&p, &w, &mut d, Vec2::new(3.0, 0.0));
        assert_eq!(active(&pool), 2);
        assert_eq!(pool.slot(0).dust_type, 107);
        assert_eq!(pool.slot(1).dust_type, 107);
        // Both x-0.25 vel: x = (3+0)*-0.25 = -0.75.
        assert!((pool.slot(0).velocity.x - (-0.75)).abs() < 1e-5);
        assert!((pool.slot(1).velocity.x - (-0.75)).abs() < 1e-5);
    }

    /// `:26499-26504` - the Sword Beam's pulse dust-64 (x-0.25 vel,
    /// noGravity). Type 64 carries the NewDust tweak (59..=65, a 7th draw).
    #[test]
    fn w4_f5_style27_sword_pulse_is_dust64_nogravity() {
        let w = world();
        let p = proj(crate::projectile::SWORD_BEAM, 3.0, 0.0);
        let mut pool = DustPool::new();
        let ctx = test_dust_ctx(&w);
        let mut s = Script::new(vec![0; 7], vec![]);
        let mut d = ProjDust3 { pool: &mut pool, ctx: &ctx, rng: &mut s };
        style27_sword_pulse(&p, &w, &mut d, Vec2::new(3.0, 0.0));
        assert_eq!(active(&pool), 1);
        let t = pool.slot(0);
        assert_eq!(t.dust_type, 64);
        assert!(t.no_gravity);
        // scale = (1)*0.7 [tweak] *1.5 [arg] = 1.05.
        assert!((t.scale - 1.05).abs() < 1e-5);
        // velocity.x = (3*0.3 [tweak] + 0)*-0.25 = -0.225.
        assert!((t.velocity.x - (-0.225)).abs() < 1e-5);
    }
}
