//! W4-g1 lane a — the six FUNCTIONAL `Kill()` hooks missing for rowed+armed
//! kinds (census `docs/research/26d-projectile-kill-hooks-census.md`):
//!
//! - **452 PhantasmalEye** (`Projectile.cs:72987-73013`): Resize 144×144 →
//!   the dust/gore census → Resize 30×30 → `Damage()` (the hostile EVP burst).
//! - **454 PhantasmalSphere** (`:73014-73045`): the recentre-208 box → the
//!   ring dust + 3-gore census → `Damage()` (the hostile EVP burst).
//! - **459 ChargedBlasterOrb** (`:73338-73371`, the 459/709 SHARED burst —
//!   g1's claim per the f20 F3 cross-note, commit `01e0b1083` on
//!   `task/W4-f20-proj-ai-verify-2`, which ports the 709-ONLY half
//!   `:73372-73388` + the 1110 row): at `scale >= 1` the Resize-144 →
//!   `Damage()` (the friendly PVE burst) → dust/gore rings, in that C#
//!   order; below 1 the small 3+10 dust census, zero gores. The arm is
//!   459-gated here (709 is unrowed on this branch — d15 owns the row);
//!   at the merge the pool gate widens to `459 | 709` (kind-disjoint from
//!   f20's `sky_dragons_fury_kill`, which seats the 709-only half).
//! - **521 CrystalPulse** (`:73292-73327`): the 20-dust census, then the
//!   owner-gated `Next(3,6)`-count CrystalPulse2 (522) split — the velocity
//!   draws are STATE-BEARING and feed the spawns inline. The 522's own
//!   Kill arm (`:73271-73290`, the 10-dust census) lands with the shard.
//! - **448 SaucerMissile** (`:73603-73631`): the recentre-112 box → the
//!   dust/gore census → `Damage()` (the hostile EVP burst).
//! - **80 IceBlock** (`:75533-75553`): the `ai[0] >= 0` sound/dust census +
//!   the tile-127 `WorldGen.KillTile` break (the pool hook owns the tile
//!   write, `ice_block_kill_tile` in projectile.rs).
//!
//! C# sources read WHOLE (the golden rule): the `Kill()` head `:70642-70660`
//! (the `!active` guard + the `num = timeLeft` capture), each arm above, the
//! `Damage()` body `:12487-12516`, `Damage_CanDealDamage` `:12446-12485`
//! (none of the six kinds is on the refusal list), `Damage_EVP`
//! `:14773-14900` (the hostile player hit — the `BossNoCheese` cooldown
//! `:14782-14787` is the standing single-lane deviation, M4-HAZARD-1), and
//! `Colliding`'s per-kind gates `:15406-15412` (454) / `:15447-15450` (452) +
//! the fallback `:15887-15891`.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw stays in the C#'s order; the dust/gore
//! BODIES are skipped (the `projectile_ai_misc` convention) and the
//! `Dust.NewDust*` internal rolls go with the dust pool. The `Gore.NewGore`
//! internal kicks DO stay — `Next(10,31)` Y then `Next(-20,21)` X
//! (`Gore.cs:1327-1328`) — plus the explicit position `Next(100)` pair, the
//! `Next(61,64)` type and the two post-write `Next(-10,11)` kicks (the 7-draw
//! `gore()` row, the `celeb2_explosion` precedent; g16 upgrades these census
//! rows to the `GorePool` at its wiring wave). The 454/459 ring dust draws
//! TWO `next_double` per particle (the `RotatedBy(2π·NextDouble())` angle then
//! the `NextDouble()` radius, `Utils.cs:1675-1684`); the argument evaluation
//! order is left-to-right (`(width/2, 0).RotatedBy(2π·d1) * d2`).
//!
//! Sound cues (`Item14`/`Item27`/`Item110`/legacy 29/4/0) are the sound-lane
//! skip (g21-25); the `PlaySound` draws nothing on this stream in the port's
//! queue convention until the cue lands.
//!
//! CODE-ONLY MODE (W4-g1): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); every expected value is
//! hand-derived from the cited C# lines.

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::projectile::{Projectile, safe_normalize};

/// One gore's draw row (the `celeb2_explosion::gore` shape, shared by the
/// 452/454/459/448 bursts): the position `Next(100)` pair, the `Next(61,64)`
/// type, the `NewGore` kicks (`Gore.cs:1327-1328`), the two post-write
/// `Next(-10,11)` velocity kicks (`:73008-73009` etc.).
fn gore(rng: &mut dyn DropRng) {
    let _ = rng.next_range(0, 100);
    let _ = rng.next_range(0, 100);
    let _ = rng.next_range(61, 64);
    let _ = rng.next_range(10, 31);
    let _ = rng.next_range(-20, 21);
    let _ = rng.next_range(-10, 11);
    let _ = rng.next_range(-10, 11);
}

/// One ring-dust particle's two DOUBLE draws (`:73024` etc.): the
/// `RotatedBy(2π·NextDouble())` angle, then the `NextDouble()` radius. The
/// dust body is the skip; only the draws stay.
fn ring_dust(rng: &mut dyn DropRng) {
    let _ = rng.next_double();
    let _ = rng.next_double();
}

/// `Entity.Resize` (`Projectile.cs:22353-22359`): the position snaps to the
/// old centre, the box grows, then `Center = position` recentres on the
/// INTEGER half (`width / 2` on the C#'s int field). The 452's two resizes
/// and the 459's scale-gated resize ride this; the 454/448 recentre shapes
/// are the arms' own (below).
fn resize(p: &mut Projectile, side: f32) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    p.pos = center;
    p.w = side;
    p.h = side;
    p.pos.x -= (side as i32 / 2) as f32;
    p.pos.y -= (side as i32 / 2) as f32;
}

/// **452 PhantasmalEye** — `Kill()`'s arm (`Projectile.cs:72987-73013`):
/// `Resize(144, 144)`, the 4-dust-31 + 40×2-dust-229 census (no explicit
/// draws), ONE gore, `Resize(30, 30)`, then `Damage()` (the pool hook runs
/// the EVP burst through the final 30×30 box). The legacy sound 29 style 103
/// is the sound skip.
pub(crate) fn phantasmal_eye_kill(p: &mut Projectile, rng: &mut dyn DropRng) {
    resize(p, 144.0); // `:72990`
    // `:72991-73003` — the 4 + 80 dusts roll nothing explicit.
    gore(rng); // `:73004-73010`
    resize(p, 30.0); // `:73011`
    // `:73012` — `Damage()` rides the pool hook (the 30×30 box).
}

/// **454 PhantasmalSphere** — `Kill()`'s arm (`Projectile.cs:73014-73045`):
/// `position = Center`, the 208×208 box recentred on the integer half, the
/// 7 ring dusts + 60×2 ring dusts (TWO doubles each), THREE gores, then
/// `Damage()` (the pool hook, the 208×208 box). The legacy sound 4 style 6
/// is the sound skip.
pub(crate) fn phantasmal_sphere_kill(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:73017-73020` — the recentre-208 shape (the arm's own, not Resize).
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    p.pos = center;
    p.w = 208.0;
    p.h = 208.0;
    p.pos.x -= (p.w as i32 / 2) as f32;
    p.pos.y -= (p.h as i32 / 2) as f32;
    // `:73021-73025` — 7 ring dusts.
    for _ in 0..7 {
        ring_dust(rng);
    }
    // `:73026-73036` — 60 × 2 ring dusts.
    for _ in 0..60 {
        ring_dust(rng);
        ring_dust(rng);
    }
    // `:73037-73043` — three gores.
    for _ in 0..3 {
        gore(rng);
    }
    // `:73044` — `Damage()` rides the pool hook (the 208×208 box).
}

/// **459 ChargedBlasterOrb** — `Kill()`'s 459/709 SHARED arm
/// (`Projectile.cs:73338-73352`), the resize + burst-flag half: at
/// `scale >= 1` the Resize-144 and the `Damage()` flag (the pool hook runs
/// the PVE burst BETWEEN this and the draw census below — the C# `:73351`
/// `Damage()` precedes the `:73353` dust rings, so the per-hit crit /
/// DamageVar draws stay ahead of the ring draws on the stream). Below 1:
/// no resize, no `Damage()`. Returns whether the burst fired.
pub(crate) fn charged_blaster_orb_kill(p: &mut Projectile) -> bool {
    let burst = p.scale >= 1.0; // `:73343`
    if burst {
        // `:73345-73347` — the resize (the Damage flag rides the return).
        resize(p, 144.0);
    }
    burst
}

/// The 459/709 shared arm's draw census (`Projectile.cs:73340-73342` /
/// `:73348-73350` counts, `:73353-73371` loops): at burst the 7+30 ring
/// dusts (TWO doubles each) and TWO gores; below scale 1 the small 3+10
/// ring-dust census, zero gores. Runs AFTER the pool's `Damage()` burst
/// (the C# body order).
pub(crate) fn charged_blaster_orb_kill_draws(burst: bool, rng: &mut dyn DropRng) {
    let (n31, n226, n_gore) = if burst { (7, 30, 2) } else { (3, 10, 0) };
    // `:73353-73357` — the dust-31 ring.
    for _ in 0..n31 {
        ring_dust(rng);
    }
    // `:73358-73364` — the dust-226 ring.
    for _ in 0..n226 {
        ring_dust(rng);
    }
    // `:73365-73371` — the gores (0 below scale 1).
    for _ in 0..n_gore {
        gore(rng);
    }
}

/// **448 SaucerMissile** — `Kill()`'s arm (`Projectile.cs:73603-73631`):
/// the recentre-112 box, the 4-dust-31 + 40×2-dust-228 census (no explicit
/// draws), ONE gore, then `Damage()` (the pool hook, the 112×112 box). The
/// `Item14` cue is the sound skip.
pub(crate) fn saucer_missile_kill(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:73606-73609` — the recentre-112 shape (the arm's own).
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    p.pos = center;
    p.w = 112.0;
    p.h = 112.0;
    p.pos.x -= (p.w as i32 / 2) as f32;
    p.pos.y -= (p.h as i32 / 2) as f32;
    // `:73610-73622` — the 4 + 80 dusts roll nothing explicit.
    gore(rng); // `:73623-73629`
    // `:73630` — `Damage()` rides the pool hook (the 112×112 box).
}

/// **522 CrystalPulse2** — `Kill()`'s own arm (`Projectile.cs:73271-73290`),
/// the draw census: the 10-dust-254 loop, one `Next(3)` gate per dust, then
/// the taken branch's two `Next(-10,11)` draws OR the else branch's one
/// (the SAME shape as the 521 census below, different constants only). No
/// functional state; the `Item118` cue is the sound skip. The pool death
/// hook runs this for every 522 corpse (the ai[1] > 40 kill, the lifetime
/// kill, the tile kill alike).
pub(crate) fn crystal_pulse2_kill_draws(rng: &mut dyn DropRng) {
    // `:73274-73290` — the 10-dust census.
    for _ in 0..10 {
        if rng.next_range(0, 3) == 0 {
            let _ = rng.next_range(-10, 11); // fadeIn
            let _ = rng.next_range(-10, 11); // scale
        } else {
            let _ = rng.next_range(-10, 11); // scale
        }
    }
}

/// **521 CrystalPulse** — `Kill()`'s arm (`Projectile.cs:73292-73327`): the
/// 20-dust-254 census (one `Next(3)` gate per dust, then the taken branch's
/// two `Next(-10,11)` draws OR the else branch's one), then the owner-gated
/// split: `Next(3,6)` shards, each a `Next(-100,101)` velocity pair (re-rolled
/// while both zero), normalised, scaled by `Next(70,101)·0.1`, spawned as a
/// 522 at `oldPosition + (width/2, height/2)` with damage `(int)(damage·0.8)`
/// and knockback `·0.8`. The velocity draws are STATE-BEARING — they ride the
/// stream here and feed the returned spawn requests. The `Item110` cue is the
/// sound skip. `local_owner` folds the C# `Main.myPlayer == this.owner` gate.
pub(crate) fn crystal_pulse_kill(
    p: &Projectile,
    rng: &mut dyn DropRng,
    local_owner: u8,
) -> Vec<PulseShard> {
    // `:73295-73311` — the 20-dust census.
    for _ in 0..20 {
        if rng.next_range(0, 3) == 0 {
            let _ = rng.next_range(-10, 11); // fadeIn
            let _ = rng.next_range(-10, 11); // scale
        } else {
            let _ = rng.next_range(-10, 11); // scale
        }
    }
    let mut shards = Vec::new();
    if p.owner != local_owner {
        return shards; // `:73312`
    }
    let count = rng.next_range(3, 6); // `:73314`
    // `:73324` — the spawn centre is `oldPosition + (width/2, height/2)`
    // (the integer halves of the C#'s int fields).
    let sx = p.old_position.x + (p.w as i32 / 2) as f32;
    let sy = p.old_position.y + (p.h as i32 / 2) as f32;
    let damage = (p.damage as f64 * 0.8) as i32; // `:73324`
    let kb = p.knock_back * 0.8;
    for _ in 0..count {
        // `:73317-73321` — the velocity pair, re-rolled while both zero.
        let mut vx = rng.next_range(-100, 101) as f32;
        let mut vy = rng.next_range(-100, 101) as f32;
        while vx == 0.0 && vy == 0.0 {
            vx = rng.next_range(-100, 101) as f32;
            vy = rng.next_range(-100, 101) as f32;
        }
        // `:73322-73323` — normalise, then the speed scale.
        let dir = safe_normalize(Vec2::new(vx, vy));
        let speed = rng.next_range(70, 101) as f32 * 0.1;
        let v = dir * speed;
        shards.push(PulseShard {
            x: sx,
            y: sy,
            vx: v.x,
            vy: v.y,
            damage,
            kb,
        });
    }
    shards
}

/// One CrystalPulse2 (522) split spawn the 521 hook queues
/// (`Projectile.cs:73324`): the C# spawn CENTRE (`NewProjectile` subtracts
/// w/2, h/2 — the pool spawn takes the top-left, so the hook recentres by the
/// 522 row's 8×8 box).
pub(crate) struct PulseShard {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub kb: f32,
}

/// **522 CrystalPulse2** — `AI_029_GemStaffs`'s own arm
/// (`Projectile.cs:37742-37773`): `ai[1]++`, the `(60 − ai[1])/60` fade
/// factor (dust-only), the `ai[1] > 40` kill, `velocity.Y += 0.2` clamped 18,
/// `velocity.X *= 0.98`. The 2+1 dust spawns are the dust-lane skip (they
/// roll nothing explicit). The C# `Kill()` does NOT return — the velocity
/// writes still run on the kill tick (the `ai_001` precedent). Returns
/// whether the slot died (the pool's death convention zeroes it).
pub(crate) fn crystal_pulse2_step(p: &mut Projectile) -> bool {
    p.ai[1] += 1.0; // `:37744`
    // `:37745` — the fade factor feeds only the skipped dust.
    if p.ai[1] > 40.0 {
        // `:37746-37749` — the kill (the `kill()` convention: the `num`
        // capture `:70649` rides kill_time_left; the pool death hook runs
        // the 522 Kill arm's draw census, `crystal_pulse2_kill_draws`).
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
    // `:37750-37755` — the gravity + drag (run even on the kill tick).
    p.vel.y += 0.2;
    if p.vel.y > 18.0 {
        p.vel.y = 18.0;
    }
    p.vel.x *= 0.98;
    !p.active
}

/// **80 IceBlock** — `Kill()`'s arm (`Projectile.cs:75533-75553`), the
/// draw-census half: at `ai[0] >= 0` the `Item27` cue (sound skip) + the
/// 10-dust-67 burst (no explicit draws). The tile-127 break (`:75543-75552`)
/// needs `&mut World`, so the pool hook owns it (the ICE_BLOCK arm of
/// `ProjectilePool::kill_hooks_a`); this fn is the RNG census only and rolls
/// NOTHING (the dust bodies are the skip). Kept for the witness + the g63-65
/// dust wiring.
pub(crate) fn ice_block_kill_draws(p: &Projectile, _rng: &mut dyn DropRng) {
    if p.ai[0] >= 0.0 {
        // `:75537-75541` — the 10 dust-67s roll nothing explicit.
    }
}

/// `Colliding`'s per-kind gate for the hostile death bursts
/// (`Projectile.cs:15406-15412` / `:15447-15450`): the 454 refuses while
/// `0 <= ai[0] < 60 && ai[1] != -1`; the 452 refuses while `ai[0] < 2`; the
/// 448 has no gate (the fallback `:15888` overlap). The pool's EVP burst
/// reads this AFTER the box overlap.
pub(crate) fn hostile_burst_colliding(p: &Projectile) -> bool {
    match p.kind {
        452 => p.ai[0] >= 2.0,
        454 => !(p.ai[0] >= 0.0 && p.ai[0] < 60.0 && p.ai[1] != -1.0),
        _ => true,
    }
}

/// W4-g1 lane a — the witnesses (WRITTEN NOT RUN, code-only mode). The draw
/// sequences replay the C#'s explicit `Main.rand` calls in order; the
/// scripted rng returns fixed values so the state-bearing 521 split is
/// hand-checkable.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, CRYSTAL_PULSE, ICE_BLOCK, PHANTASMAL_EYE, PHANTASMAL_SPHERE, CHARGED_BLASTER_ORB, SAUCER_MISSILE};

    /// A scripted stream: `next_range` pops the front of `ranges` (clamped
    /// into [min,max) for sanity), `next_double` pops `doubles`, `next_float`
    /// pops `floats`. Every draw is RECORDED so the census order is pinned.
    struct ScriptRng {
        ranges: std::collections::VecDeque<i32>,
        doubles: std::collections::VecDeque<f64>,
        log: Vec<&'static str>,
        range_log: Vec<(i32, i32)>,
    }
    impl ScriptRng {
        fn new(ranges: &[i32], doubles: &[f64]) -> Self {
            Self {
                ranges: ranges.iter().copied().collect(),
                doubles: doubles.iter().copied().collect(),
                log: Vec::new(),
                range_log: Vec::new(),
            }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.log.push("range");
            self.range_log.push((min, max));
            let v = self.ranges.pop_front().unwrap_or(min);
            v.clamp(min, max - 1)
        }
        fn next_float(&mut self) -> f32 {
            self.log.push("float");
            self.doubles.pop_front().unwrap_or(0.0) as f32
        }
        fn next_double(&mut self) -> f64 {
            self.log.push("double");
            self.doubles.pop_front().unwrap_or(0.0)
        }
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(320.0, 320.0);
        p.w = d.w;
        p.h = d.h;
        p.damage = 40;
        p.knock_back = 4.0;
        p.scale = d.scale;
        p
    }

    /// The 452 arm: Resize 144 → ONE gore (7 range draws) → Resize 30. The
    /// final box is 30×30 recentred on the ORIGINAL centre (`:72990`/`:73011`
    /// — both resizes keep the centre).
    #[test]
    fn w4_g1a_the_phantasmal_eye_kill_resizes_and_censuses_one_gore() {
        let mut p = proj(PHANTASMAL_EYE);
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let mut rng = ScriptRng::new(&[50, 50, 62, 20, 0, 5, -5], &[]);
        phantasmal_eye_kill(&mut p, &mut rng);
        assert_eq!((p.w, p.h), (30.0, 30.0), "the trailing Resize(30,30)");
        assert_eq!(p.pos, center - Vec2::new(15.0, 15.0), "recentred on the centre");
        assert_eq!(rng.range_log.len(), 7, "one gore row");
        assert_eq!(rng.range_log[0], (0, 100));
        assert_eq!(rng.range_log[2], (61, 64));
        assert_eq!(rng.range_log[3], (10, 31));
        assert_eq!(rng.range_log[4], (-20, 21));
    }

    /// The 454 arm: the 208 box + 7 + 120 ring dusts (2 doubles each = 254)
    /// + 3 gores (21 ranges). The box recentres on the original centre.
    #[test]
    fn w4_g1a_the_phantasmal_sphere_kill_draws_254_doubles_and_21_ranges() {
        let mut p = proj(PHANTASMAL_SPHERE);
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let mut rng = ScriptRng::new(&[0; 21], &[0.0; 254]);
        phantasmal_sphere_kill(&mut p, &mut rng);
        assert_eq!((p.w, p.h), (208.0, 208.0));
        assert_eq!(p.pos, center - Vec2::new(104.0, 104.0));
        assert_eq!(rng.log.iter().filter(|s| **s == "double").count(), 254);
        assert_eq!(rng.range_log.len(), 21, "three gore rows");
    }

    /// The 459 arm at scale 1: the Resize-144 + burst flag TRUE, then the
    /// census (37 ring dusts = 74 doubles + 2 gores = 14 ranges). Below 1:
    /// flag FALSE, no resize, 13 ring dusts (26 doubles), zero gores. The
    /// pool runs `Damage()` BETWEEN the two halves (the C# :73351 order).
    #[test]
    fn w4_g1a_the_charged_blaster_orb_kill_gates_on_scale() {
        let mut big = proj(CHARGED_BLASTER_ORB);
        big.scale = 1.0;
        let mut rng = ScriptRng::new(&[0; 14], &[0.0; 74]);
        let burst = charged_blaster_orb_kill(&mut big);
        assert!(burst);
        assert_eq!((big.w, big.h), (144.0, 144.0));
        assert!(rng.log.is_empty(), "the resize half draws nothing");
        charged_blaster_orb_kill_draws(burst, &mut rng);
        assert_eq!(rng.range_log.len(), 14);
        assert_eq!(rng.log.iter().filter(|s| **s == "double").count(), 74);

        let mut small = proj(CHARGED_BLASTER_ORB);
        small.scale = 0.85;
        let (w, h) = (small.w, small.h);
        let mut rng = ScriptRng::new(&[], &[0.0; 26]);
        let burst = charged_blaster_orb_kill(&mut small);
        assert!(!burst);
        assert_eq!((small.w, small.h), (w, h), "no resize below scale 1");
        charged_blaster_orb_kill_draws(burst, &mut rng);
        assert_eq!(rng.range_log.len(), 0, "zero gores below scale 1");
        assert_eq!(rng.log.iter().filter(|s| **s == "double").count(), 26);
    }

    /// The 448 arm: the 112 box + ONE gore (7 ranges), no explicit dust
    /// draws. The box recentres on the original centre.
    #[test]
    fn w4_g1a_the_saucer_missile_kill_resizes_112_and_censuses_one_gore() {
        let mut p = proj(SAUCER_MISSILE);
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let mut rng = ScriptRng::new(&[0; 7], &[]);
        saucer_missile_kill(&mut p, &mut rng);
        assert_eq!((p.w, p.h), (112.0, 112.0));
        assert_eq!(p.pos, center - Vec2::new(56.0, 56.0));
        assert_eq!(rng.range_log.len(), 7);
    }

    /// The 521 split: the 20-dust census (20 `Next(3)` gates; scripted all
    /// non-zero → one `Next(-10,11)` each = 40 ranges), then `Next(3,6)` → 3
    /// shards, each a velocity pair + a speed draw. The scripted velocity
    /// (100, 0) normalises to +X; speed `Next(70,101)` → 70 → 7.0. Damage
    /// `(int)(40·0.8) = 32`, knockback `4·0.8 = 3.2`. The spawn centre is
    /// `oldPosition + (w/2, h/2)`.
    #[test]
    fn w4_g1a_the_crystal_pulse_kill_splits_three_shards_in_draw_order() {
        let mut p = proj(CRYSTAL_PULSE);
        p.old_position = Vec2::new(300.0, 300.0);
        // 20 gates (all 1 → else branch) + 20 scales, then count 3, then
        // per shard: vx 100, vy 0, speed 70.
        let mut ranges: Vec<i32> = Vec::new();
        for _ in 0..20 {
            ranges.push(1); // the Next(3) gate (non-zero)
            ranges.push(0); // the else Next(-10,11) scale
        }
        ranges.push(3); // the shard count
        for _ in 0..3 {
            ranges.push(100); // vx
            ranges.push(0); // vy
            ranges.push(70); // speed
        }
        let mut rng = ScriptRng::new(&ranges, &[]);
        let shards = crystal_pulse_kill(&p, &mut rng, 0);
        assert_eq!(shards.len(), 3);
        for s in &shards {
            assert_eq!((s.x, s.y), (307.0, 307.0), "oldPos + (14/2, 14/2)");
            assert!((s.vx - 7.0).abs() < 1e-4 && s.vy.abs() < 1e-4, "+X at 7.0");
            assert_eq!(s.damage, 32);
            assert!((s.kb - 3.2).abs() < 1e-4);
        }
        // The draw order: 40 census ranges, the count, then 3×3 shard draws.
        assert_eq!(rng.range_log.len(), 40 + 1 + 9);
        assert_eq!(rng.range_log[40], (3, 6), "the count draw after the census");
        assert_eq!(rng.range_log[41], (-100, 101), "shard 0's vx");
    }

    /// The 521 split's owner gate: a foreign owner draws ONLY the 20-dust
    /// census (40 ranges), no count, no shards.
    #[test]
    fn w4_g1a_the_crystal_pulse_kill_foreign_owner_spawns_nothing() {
        let mut p = proj(CRYSTAL_PULSE);
        p.owner = 255;
        let mut ranges: Vec<i32> = Vec::new();
        for _ in 0..20 {
            ranges.push(1);
            ranges.push(0);
        }
        let mut rng = ScriptRng::new(&ranges, &[]);
        let shards = crystal_pulse_kill(&p, &mut rng, 0);
        assert!(shards.is_empty());
        assert_eq!(rng.range_log.len(), 40, "the census only");
    }

    /// The 522 Kill census: 10 `Next(3)` gates; scripted all non-zero →
    /// one `Next(-10,11)` each = 20 ranges, zero doubles. The scripted
    /// zero gates take the two-draw branch (30 ranges).
    #[test]
    fn w4_g1a_the_crystal_pulse2_kill_draws_census() {
        let mut ranges: Vec<i32> = Vec::new();
        for _ in 0..10 {
            ranges.push(1); // the Next(3) gate (non-zero)
            ranges.push(0); // the else Next(-10,11) scale
        }
        let mut rng = ScriptRng::new(&ranges, &[]);
        crystal_pulse2_kill_draws(&mut rng);
        assert_eq!(rng.range_log.len(), 20);
        assert_eq!(rng.range_log[0], (0, 3));
        assert_eq!(rng.range_log[1], (-10, 11));

        let mut ranges: Vec<i32> = Vec::new();
        for _ in 0..10 {
            ranges.push(0); // the gate taken
            ranges.push(0); // fadeIn
            ranges.push(0); // scale
        }
        let mut rng = ScriptRng::new(&ranges, &[]);
        crystal_pulse2_kill_draws(&mut rng);
        assert_eq!(rng.range_log.len(), 30, "the taken branch draws two");
    }

    /// The 522 arm: ai[1] counts up, the kill fires at > 40 (tick 41), the
    /// gravity/drag run even on the kill tick. At ai[1] = 40 the slot lives.
    #[test]
    fn w4_g1a_the_crystal_pulse2_kills_at_41_and_still_integrates() {
        let mut p = proj(522);
        p.ai[1] = 39.0;
        p.vel = Vec2::new(10.0, 0.0);
        assert!(!crystal_pulse2_step(&mut p), "tick 40 lives");
        assert_eq!(p.ai[1], 40.0);
        assert!((p.vel.x - 9.8).abs() < 1e-4, "the 0.98 drag");
        assert!((p.vel.y - 0.2).abs() < 1e-4, "the 0.2 gravity");
        assert!(crystal_pulse2_step(&mut p), "tick 41 kills");
        assert!(!p.active);
        assert!((p.vel.y - 0.4).abs() < 1e-4, "the kill tick still integrates");
    }

    /// The hostile burst gates: the 452 refuses below ai[0] 2, the 454
    /// refuses inside the 0..60 window unless ai[1] == -1, the 448 always
    /// passes (the fallback overlap is the pool's).
    #[test]
    fn w4_g1a_the_hostile_burst_colliding_gates() {
        let mut eye = proj(PHANTASMAL_EYE);
        eye.ai[0] = 1.0;
        assert!(!hostile_burst_colliding(&eye));
        eye.ai[0] = 2.0;
        assert!(hostile_burst_colliding(&eye));
        let mut sphere = proj(PHANTASMAL_SPHERE);
        sphere.ai[0] = 30.0;
        sphere.ai[1] = 0.0;
        assert!(!hostile_burst_colliding(&sphere), "inside the window");
        sphere.ai[1] = -1.0;
        assert!(hostile_burst_colliding(&sphere), "ai[1] == -1 opens it");
        sphere.ai[0] = 60.0;
        sphere.ai[1] = 0.0;
        assert!(hostile_burst_colliding(&sphere), "past the window");
        let missile = proj(SAUCER_MISSILE);
        assert!(hostile_burst_colliding(&missile));
    }

    /// The 80 draw census rolls nothing (the dust bodies are the skip); the
    /// ai[0] < 0 arm skips even the sound.
    #[test]
    fn w4_g1a_the_ice_block_kill_draws_roll_nothing() {
        let mut p = proj(ICE_BLOCK);
        p.ai[0] = 5.0;
        let mut rng = ScriptRng::new(&[], &[]);
        ice_block_kill_draws(&p, &mut rng);
        assert!(rng.log.is_empty());
    }
}
