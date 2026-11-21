//! NPC gore wave E — the HitEffect REMAINDER census came back EMPTY (the
//! HEAD gap `NPC.cs:81654-81659` is the signature + `!active` gate; every
//! gore inside `:81660-89834` is claimed by d55/e5/f10/f5), so this module
//! takes the 17 UNCLAIMED `Gore.NewGore*` sites in OTHER NPC methods
//! (research/71-npc-gore-wave-e-claims.md, W4-f4 wave E, code-only mode).
//!
//! Batch 1 (this file's first ten sites, E1-E10):
//!
//! | # | C# site | host | arm |
//! | --- | --- | --- | --- |
//! | E1-E3 | `:32028-32030` | `AI()` FungiSpore 261 | the `localAI[2]==0` birth trio 378/379/380 |
//! | E4 | `:43233` | `AI_015_KingSlime` | the shrink tick-60 crown 734 |
//! | E5 | `:45424` | `AI_121_QueenSlime` | the vanish tick-60 crown 1258 |
//! | E6 | `:60934` | `AI_001_Slimes` | the confetti slime (`ai[1]==1345`) ambient `Next(60)` gore 276-282 |
//! | E7 | `:62187` | `AI_110_Betsy` | the fire-breath tile-sweep per-cell `NewGoreDirect` 61-63 |
//! | E8 | `:63247` | `AI_107_ImprovedWalkers` | the 572/573 kobold-bomber explosion quintet 61-63 |
//! | E9 | `:63609` | `AI_107_ImprovedWalkers` | the 567 skeleton walk-drip 971/972 |
//! | E10 | `:65774` | `AI_108_DivingFlyer` | the 574/575 kobold-glider explosion quintet (E8's twin body) |
//!
//! Batch 2 (E11-E17: the TownBunny ear-flick `:68953`, the MoonLordCore
//! open quad `:71931-71934`, the SantaNK1 half-hp strike `:81354`, the
//! loveStruck heart `:91340`) lands in the next commit.
//!
//! ## RNG contract (the d55/g61 convention)
//!
//! Every draw rides the caller's ONE stream in C# evaluation order:
//! argument expressions BEFORE the `NewGore` internals they feed (the
//! `Gore.cs:1327-1328` Y-then-X kicks roll pool-side inside
//! `GorePool::new_gore`), post-write draws AFTER the allocator returned —
//! the 600 sentinel slot included (`Main.gore[600]` exists). The
//! interleaved DUST loops of the bomber/glider/Betsy/confetti hosts are
//! the dust lanes' (the f4 THIRD-batch remainder / g62): this module
//! consumes ONLY the gore-arm draws; the wiring round must run the
//! preceding dust draws first (the g59/g60 argument/body split,
//! `eea6975b`/`e55e2272`).
//!
//! ## Host gates (the caller's)
//!
//! * E1-E3: `type == 261 && localAI[2] == 0` (`:31654`/`:32026`); the
//!   caller stamps `localAI[2] = 1` AFTER the trio (`:32031`).
//! * E4/E5: the shrink/vanish state's `ai[0] == 60` tick (`:43231` /
//!   `:45422`) — the crown fires ONCE per teleport, before the Bottom
//!   snap. The 10-dust loops around them (`:43251`/`:43278`,
//!   `:45408`/`:45444`) are the dust lane's.
//! * E6: `ai[1] == 1345` (`:60923`); the `Next(30)` confetti-dust gate
//!   (`:60926-60931`) runs BEFORE this arm's `Next(60)` gate.
//! * E7: the sweep's per-cell `flag & flag2` exposed-solid test and
//!   `num37 = KillTile_GetTileDustAmount` (`:62166-62170`) are the
//!   caller's; the two tile-dust loops (`:62171-62183`) draw first.
//! * E8/E10: `ai[1] == 2` explosion tick (`:63204` / `:65739`); the
//!   4+20×2+6+12 dust loops (`:63216-63244` / `:65743-65771`) draw first.
//! * E9: `num41 > 100 && (int)num41 % 4 == 0` where `num41 = localAI[3]`
//!   AFTER the increment (`:63564-63565`); the walk-dust loop
//!   (`:63579-63604`) draws first.
//!
//! Witnesses: WRITTEN NOT RUN (code-only mode, no cargo per the lane
//! directive). See docs/research/71-npc-gore-wave-e-claims.md.
#![allow(dead_code)] // UNWIRED (the f4 code-only round): the host machines' wiring is the follow-up

use crate::npc::Npc;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::{UnifiedRandom, World};

/// `Entity.Center` (`Entity.cs:50-55`): position + size/2 (float halves).
fn center(npc: &Npc) -> Vec2 {
    let (w, h) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0)
}

/// `Entity.Left` (`Entity.cs:62-67`): `(position.X, position.Y + height/2f)`.
fn left(npc: &Npc) -> Vec2 {
    let (_, h) = npc.size();
    Vec2::new(npc.x, npc.y + h / 2.0)
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`, centre = zero): cos/sin in
/// DOUBLE cast to f32, then `x·c − y·s` / `x·s + y·c` in that order.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let c = radians.cos() as f32;
    let s = radians.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`): TWO
/// `NextDouble` draws, left operand first, the double subtraction, then
/// the float-cos/sin spin (the bow_launch helper, same maths).
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    rotated_by(v, angle)
}

/// E1-E3 — the FungiSpore 261 birth trio (`NPC.cs:32026-32032`): the
/// `localAI[2]==0` one-shot spawns gores 378/379/380 at
/// `(position.X + Next(width), position.Y + Next(height))` — the X draw
/// precedes the Y draw (argument order), the velocity is `base.velocity`,
/// the scale `this.scale`. NO post-writes; the kicks roll pool-side.
/// The caller stamps `localAI[2] = 1` after (`:32031`).
pub fn fungi_spore_birth_gores(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let vel = Vec2::new(npc.vx, npc.vy);
    for ty in [378, 379, 380] {
        // `:32028-32030` — `Main.rand.Next(base.width)` then
        // `Main.rand.Next(base.height)`, both cast to f32.
        let px = npc.x + rng.next_max(w as i32) as f32;
        let py = npc.y + rng.next_max(h as i32) as f32;
        gore.new_gore(rng, world, Vec2::new(px, py), vel, ty, npc.scale);
    }
}

/// E4 — the King Slime shrink tick-60 crown (`NPC.cs:43231-43234`):
/// ONE gore 734 at `Center + (-40, -height/2)`, velocity `base.velocity`,
/// default scale 1f (the C# passes no Scale — NOT `this.scale`). The
/// caller gates `ai[0] == 60` inside the `ai[1]==5` shrink state.
pub fn king_slime_shrink_crown(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (_, h) = npc.size();
    // `:43233` — `-base.height / 2` is INTEGER division (int argument
    // widened to float), truncating toward zero on odd heights.
    let pos = center(npc) + Vec2::new(-40.0, (-(h as i32) / 2) as f32);
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), 734, 1.0);
}

/// E5 — the Queen Slime vanish tick-60 crown (`NPC.cs:45422-45425`):
/// E4's shape at gore 1258 (`AI_121_QueenSlime`'s case-2, `ai[1]==60`).
pub fn queen_slime_vanish_crown(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (_, h) = npc.size();
    // `:45424` — the same INTEGER `-base.height / 2` as E4.
    let pos = center(npc) + Vec2::new(-40.0, (-(h as i32) / 2) as f32);
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), 1258, 1.0);
}

/// E6 — the confetti slime's ambient gore (`NPC.cs:60932-60937`,
/// `AI_001_Slimes`' `ai[1]==1345` arm): the `Next(60)==0` gate, then ONE
/// gore of type `Next(276,283)` at `base.position` with velocity (0,-10)
/// — the type argument draws BEFORE the allocator's kicks. Post-writes:
/// `scale *= 1 + Next(-20,21)*0.01` then
/// `velocity.X += Next(-50,51)*0.05`. The `Next(30)` confetti-DUST gate
/// (`:60926-60931`) runs FIRST on the same stream — the caller's.
/// Returns whether the gate fired (the witness pins the draw order).
pub fn confetti_slime_ambient_gore(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) -> bool {
    // `:60932` — the gate draw runs every qualifying tick.
    if rng.next_max(60) != 0 {
        return false;
    }
    // `:60934` — the type argument, then the kicks (pool-side).
    let ty = rng.next_range(276, 283);
    let idx = gore.new_gore(
        rng,
        world,
        Vec2::new(npc.x, npc.y),
        Vec2::new(0.0, -10.0),
        ty,
        1.0,
    );
    // `:60935-60936` — the post-writes, sentinel slot included.
    let g = gore.slot_mut(idx);
    g.scale *= 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
    g.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
    true
}

/// E7 — one Betsy fire-breath sweep cell (`NPC.cs:62184-62191`): the
/// `num37 > 0 && Next(3) != 0` gate, then `NewGoreDirect` of
/// `61 + Next(3)` at `base.position`, velocity zero, scale
/// `1 - num36*0.15 + num38*0.5` where `num36` is the sweep's LITERAL 4
/// (`:62160`, never written) and `num38 = |point.X - i| / (num34/2)`
/// is INTEGER division (num34 = 30 → 15). Post-writes:
/// `velocity.Y -= 0.1 + num36*0.5 + num38*num36`, then
/// `velocity.Y *= NextFloat()`, then the position snap to
/// `(i*16+20, j*16+20)`. The two tile-dust loops draw BEFORE this arm.
pub fn betsy_sweep_cell_gore(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    num37: i32,
    point_x: i32,
    i: i32,
    j: i32,
) -> bool {
    // `:62184` — the count gate short-circuits the Next(3) draw.
    if num37 <= 0 || rng.next_max(3) == 0 {
        return false;
    }
    // `:62160` — num36 is the literal 4; `:62186` — INTEGER division.
    let num36 = 4.0f32;
    let num38 = ((point_x - i).abs() / 15) as f32;
    let scale = 1.0 - num36 * 0.15 + num38 * 0.5;
    // `:62187` — the type argument draws before the kicks.
    let ty = 61 + rng.next_max(3);
    let idx = gore.new_gore_direct(
        rng,
        world,
        Vec2::new(npc.x, npc.y),
        Vec2::ZERO,
        ty,
        scale,
    );
    // `:62188-62190` — the post-writes, sentinel slot included.
    let g = gore.slot_mut(idx);
    g.velocity.y -= 0.1 + num36 * 0.5 + num38 * num36;
    g.velocity.y *= rng.next_float();
    g.position = Vec2::new((i * 16 + 20) as f32, (j * 16 + 20) as f32);
    true
}

/// E8/E10 — the kobold explosion quintet (`NPC.cs:63245-63252`, the
/// 572/573 walker; `:65772-65779`, the 574/575 glider — the SAME body
/// verbatim). Five gores: position argument
/// `base.position + ((width * Next(100))/100f, (height * Next(100))/100f)
/// - One*10` — the INTEGER products cast to f32, X draw before Y, then
/// the type `Next(61,64)`; velocity default ZERO, scale default 1f.
/// Post-writes: position = `Center + UnitY.RotatedByRandom(π) *
/// (NextDouble() * width / 2f)` (two NextDouble for the spin, THEN the
/// radius NextDouble), `velocity *= 0.3`, then the X and Y
/// `Next(-10,11)*0.05` jitters in that order. The 4+20×2+6+12 dust loops
/// draw BEFORE this arm (the caller's).
pub fn kobold_explosion_gores(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let (wi, hi) = (w as i32, h as i32);
    let c = center(npc);
    for _ in 0..5 {
        // `:63247` / `:65774` — the argument draws in C# order.
        let px = npc.x + (wi * rng.next_max(100)) as f32 / 100.0 - 10.0;
        let py = npc.y + (hi * rng.next_max(100)) as f32 / 100.0 - 10.0;
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, Vec2::new(px, py), Vec2::ZERO, ty, 1.0);
        // `:63248` / `:65775` — the spin (2 draws) then the radius (1).
        // The C# literal is the FLOAT π widened to double.
        let spin = rotated_by_random(Vec2::Y, 3.1415927410125732, rng);
        let radius = rng.next_double() as f32 * w / 2.0;
        let g = gore.slot_mut(idx);
        g.position = c + spin * radius;
        // `:63249-63251` / `:65776-65778`.
        g.velocity *= 0.3;
        g.velocity.x += rng.next_range(-10, 11) as f32 * 0.05;
        g.velocity.y += rng.next_range(-10, 11) as f32 * 0.05;
    }
}

/// E9 — the DD2SkeletonT3 567 walk-drip (`NPC.cs:63607-63610`): the
/// `num41 > 100 && (int)num41 % 4 == 0` gate (num41 = localAI[3] AFTER
/// the tick's increment, `:63564-63565`), then ONE `NewGoreDirect` at
/// `base.Left`, velocity (0,-3), type `SelectRandom(971,972)` (one
/// `Next(2)` pick), scale 0.85; post `velocity.X *= 0.8`. The walk-dust
/// loop draws BEFORE this arm. Returns whether the gate fired.
pub fn dd2_skeleton_walk_drip(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    local_ai3: f32,
) -> bool {
    // `:63607` — the float > 100 compare, then the int truncation.
    if !(local_ai3 > 100.0 && (local_ai3 as i32) % 4 == 0) {
        return false;
    }
    // `:63609` — SelectRandom = Next(len) over the list (`Utils.cs:2459`).
    let ty = [971, 972][rng.next_max(2) as usize];
    let idx = gore.new_gore_direct(rng, world, left(npc), Vec2::new(0.0, -3.0), ty, 0.85);
    gore.slot_mut(idx).velocity.x *= 0.8;
    true
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// A boss-shaped stand-in: the unported types ride a placeholder kind
    /// with the C# box through `size_override` (the d55 `boss` precedent).
    fn boss(x: f32, y: f32, w: f32, h: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 100, 0.0);
        n.size_override = Some((w, h));
        n.hp = 0;
        n.scale = 1.0;
        n
    }

    /// E1-E3 (`NPC.cs:32028-32030`): the trio draws X then Y per gore
    /// (Next(width), Next(height)), then the two kicks — 4 draws per gore
    /// ×3; the types are 378/379/380 at `this.scale`, velocity carried.
    #[test]
    fn w4_f4_e_the_fungi_spore_birth_draws_x_then_y_per_gore() {
        let w = air();
        let mut n = boss(100.0, 100.0, 30.0, 30.0);
        n.vx = 1.5;
        n.vy = -0.5;
        n.scale = 1.3;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(7);
        fungi_spore_birth_gores(&n, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(7);
        let mut expected = Vec::new();
        for _ in 0..3 {
            let px = 100.0 + twin.next_max(30) as f32;
            let py = 100.0 + twin.next_max(30) as f32;
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            expected.push((px, py, ky, kx));
        }
        assert_eq!(rng.peek(), twin.peek(), "the trio mirrors the C# draws");
        for (k, (px, py, ky, kx)) in expected.iter().enumerate() {
            let g = gore.slot(k);
            assert_eq!(g.gore_type, [378, 379, 380][k]);
            assert_eq!(g.scale, 1.3);
            assert_eq!(g.position, Vec2::new(*px, *py));
            assert_eq!(
                g.velocity,
                Vec2::new(1.5 + *kx as f32 * 0.1, -0.5 - *ky as f32 * 0.1)
            );
        }
    }

    /// E4/E5 (`NPC.cs:43233`/`:45424`): the crown anchors at
    /// `Center + (-40, -height/2)` with the INTEGER half on odd heights;
    /// scale 1f (NOT this.scale), velocity carried + the two kicks only.
    #[test]
    fn w4_f4_e_the_slime_crowns_anchor_the_integer_half() {
        let w = air();
        let mut n = boss(100.0, 100.0, 98.0, 93.0); // odd height 93
        n.vx = 2.0;
        n.vy = 1.0;
        n.scale = 1.7; // deliberately NOT the crown's scale
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(3);
        king_slime_shrink_crown(&n, &mut gore, &mut rng, &w);
        queen_slime_vanish_crown(&n, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(3);
        for _ in 0..2 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "two kicks per crown, nothing else");
        // Center = (149, 146.5); -(93)/2 truncates to -46 → y = 100.5.
        let anchor = Vec2::new(109.0, 100.5);
        assert_eq!(gore.slot(0).position, anchor);
        assert_eq!(gore.slot(1).position, anchor);
        assert_eq!(gore.slot(0).gore_type, 734);
        assert_eq!(gore.slot(1).gore_type, 1258);
        assert_eq!(gore.slot(0).scale, 1.0);
        assert_eq!(gore.slot(1).scale, 1.0);
    }

    /// E6 (`NPC.cs:60932-60936`): the Next(60) gate runs every tick; on a
    /// fire the type Next(276,283) draws BEFORE the kicks, then the scale
    /// and X-jitter post-draws in order. A missed gate consumes ONE draw.
    #[test]
    fn w4_f4_e_the_confetti_gore_draws_type_kicks_then_posts() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 22.0);
        // Find a seed whose first Next(60) fires.
        let mut seed = 0;
        loop {
            if UnifiedRandom::new(seed).next_max(60) == 0 {
                break;
            }
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(confetti_slime_ambient_gore(&n, &mut gore, &mut rng, &w));
        let mut twin = UnifiedRandom::new(seed);
        assert_eq!(twin.next_max(60), 0);
        let ty = twin.next_range(276, 283);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        let sc = twin.next_range(-20, 21);
        let jx = twin.next_range(-50, 51);
        assert_eq!(rng.peek(), twin.peek(), "gate, type, kicks, scale, jitter");
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.scale, 1.0 + sc as f32 * 0.01);
        assert_eq!(
            g.velocity,
            Vec2::new(kx as f32 * 0.1 + jx as f32 * 0.05, -10.0 - ky as f32 * 0.1)
        );
        // The miss path: a seed whose first Next(60) != 0 draws exactly one.
        let miss = if UnifiedRandom::new(0).next_max(60) == 0 { 1 } else { 0 };
        let mut rng2 = UnifiedRandom::new(miss);
        let mut gore2 = GorePool::new();
        assert!(!confetti_slime_ambient_gore(&n, &mut gore2, &mut rng2, &w));
        let mut twin2 = UnifiedRandom::new(miss);
        let _ = twin2.next_max(60);
        assert_eq!(rng2.peek(), twin2.peek(), "the miss consumes only the gate");
        assert!(!gore2.slot(0).active);
    }

    /// E7 (`NPC.cs:62184-62190`): the count gate short-circuits the
    /// Next(3); a fired cell draws Next(3), type Next(3), the kicks, then
    /// the NextFloat Y-scale; the position snaps to (i*16+20, j*16+20)
    /// and num38 uses INTEGER |point.X - i| / 15.
    #[test]
    fn w4_f4_e_the_betsy_cell_gore_mirrors_the_sweep_draws() {
        let w = air();
        let n = boss(100.0, 100.0, 120.0, 90.0);
        // num37 == 0: zero draws.
        let mut rng0 = UnifiedRandom::new(1);
        let mut g0 = GorePool::new();
        assert!(!betsy_sweep_cell_gore(&n, &mut g0, &mut rng0, &w, 0, 10, 10, 10));
        assert_eq!(rng0.peek(), UnifiedRandom::new(1).peek(), "no draws at count 0");
        // Find a seed whose Next(3) != 0 (the gate fires).
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(3) == 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        // point_x 25, i 10 → |15|/15 = 1 → num38 = 1.
        assert!(betsy_sweep_cell_gore(&n, &mut gore, &mut rng, &w, 3, 25, 10, 12));
        let mut twin = UnifiedRandom::new(seed);
        assert_ne!(twin.next_max(3), 0);
        let ty = 61 + twin.next_max(3);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        let ys = twin.next_float();
        assert_eq!(rng.peek(), twin.peek(), "gate, type, kicks, Y-scale");
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.scale, 1.0 - 4.0 * 0.15 + 1.0 * 0.5); // 0.9
        // velocity.Y = (0 - ky*0.1 - (0.1 + 2 + 4)) * ys.
        let vy = (-ky as f32 * 0.1 - (0.1 + 4.0 * 0.5 + 1.0 * 4.0)) * ys;
        assert!((g.velocity.y - vy).abs() < 1e-5);
        assert_eq!(g.velocity.x, kx as f32 * 0.1);
        assert_eq!(g.position, Vec2::new(180.0, 212.0)); // (10*16+20, 12*16+20)
    }

    /// E8/E10 (`NPC.cs:63247-63251`): per gore the argument draws (X, Y,
    /// type) then the kicks, then the post spin (2 NextDouble) + radius
    /// (1 NextDouble) + the X/Y jitters — 10 draws ×5; the position lands
    /// on the UnitY spin around Center, velocity ×0.3 + the jitters.
    #[test]
    fn w4_f4_e_the_kobold_quintet_draws_ten_per_gore_in_order() {
        let w = air();
        let n = boss(100.0, 100.0, 32.0, 32.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(9);
        kobold_explosion_gores(&n, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(9);
        let c = Vec2::new(116.0, 116.0);
        for k in 0..5 {
            let px = 100.0 + (32 * twin.next_max(100)) as f32 / 100.0 - 10.0;
            let py = 100.0 + (32 * twin.next_max(100)) as f32 / 100.0 - 10.0;
            let ty = twin.next_range(61, 64);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let a = twin.next_double() * 3.1415927410125732
                - twin.next_double() * 3.1415927410125732;
            let (s, co) = (a.sin() as f32, a.cos() as f32);
            let spin = Vec2::new(0.0 * co - 1.0 * s, 0.0 * s + 1.0 * co);
            let radius = twin.next_double() as f32 * 32.0 / 2.0;
            let jx = twin.next_range(-10, 11);
            let jy = twin.next_range(-10, 11);
            let g = gore.slot(k);
            assert_eq!(g.gore_type, ty);
            assert_eq!(g.scale, 1.0);
            // The allocator's argument position is overwritten by the spin.
            let _ = (px, py);
            assert_eq!(g.position, c + spin * radius);
            assert_eq!(
                g.velocity,
                Vec2::new(
                    kx as f32 * 0.1 * 0.3 + jx as f32 * 0.05,
                    -ky as f32 * 0.1 * 0.3 + jy as f32 * 0.05
                )
            );
        }
        assert_eq!(rng.peek(), twin.peek(), "50 draws, the cited order");
        // The 61-63 types are sticky-exempt (`Gore.cs:1341`).
        assert!(!gore.slot(0).sticky);
    }

    /// E9 (`NPC.cs:63607-63609`): the gate is `> 100` AND `(int)%4 == 0`;
    /// a fire draws the Next(2) pick then the kicks; the drip lands at
    /// Left with velocity (0,-3) kicked, scale 0.85, post X ×0.8.
    #[test]
    fn w4_f4_e_the_skeleton_drip_gates_on_the_int_mod_four() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 60.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(4);
        // 100 is NOT > 100; 101..103 fail %4; 104 fires.
        for v in [100.0, 101.0, 102.0, 103.0] {
            assert!(!dd2_skeleton_walk_drip(&n, &mut gore, &mut rng, &w, v));
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(4).peek(), "the misses draw nothing");
        assert!(dd2_skeleton_walk_drip(&n, &mut gore, &mut rng, &w, 104.0));
        let mut twin = UnifiedRandom::new(4);
        let ty = [971, 972][twin.next_max(2) as usize];
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "pick then kicks");
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.scale, 0.85);
        assert_eq!(g.position, Vec2::new(100.0, 130.0)); // Left = (x, y+h/2)
        assert_eq!(g.velocity.x, kx as f32 * 0.1 * 0.8);
        assert!((g.velocity.y - (-3.0 - ky as f32 * 0.1)).abs() < 1e-6);
    }
}




