//! W4-g1 lane a — the FUNCTIONAL `Kill()` hooks this lane OWNS after the
//! d78 FINAL dedupe (census `docs/research/160-projectile-kill-hooks-census.md`,
//! adjudication `26f-d78-final-kill-dedupe.md` §1, commit `fd59c51a` on
//! `task/W4-d78-ui-sounds-menus`): **459/521/522/80 KEEP; the hostile EVP
//! trio 452/454/448 DROPPED** — d78's `hostile_death_blast` +
//! `damage_evp_boxed` port `Damage_EVP` (`:14773-14900`) verbatim (DamageVar,
//! hostileDamageScaling, the ×2, the BossNoCheese cooldown lane), where this
//! lane's first-round `death_burst_player_hit` rode `hurt_with_source` with
//! RAW damage and its General-lane immune gate would refuse the 452/454
//! boss blasts the C# `:14799` lands. g65's death-dust bodies carry the
//! same census draws inside the bodies — keeping g1's standalone censuses
//! would double-roll the stream.
//!
//! - **459 ChargedBlasterOrb** (`:73338-73371`, the 459/709 SHARED burst —
//!   g1's claim per the f20 F3 cross-note, commit `01e0b1083` on
//!   `task/W4-f20-proj-ai-verify-2`, which ports the 709-ONLY half
//!   `:73372-73388` + the 1110 row): at `scale >= 1` the Resize-144 →
//!   `Damage()` (the friendly PVE burst) → dust/gore rings, in that C#
//!   order; below 1 the small 3+10 dust census, zero gores. The pool gate
//!   is `459 | 709` (the 709 row lives on d15 — row-gated inert here;
//!   kind-disjoint from f20's `sky_dragons_fury_kill`, which seats the
//!   709-only half).
//! - **521 CrystalPulse** (`:73292-73327`): the 20-dust census, then the
//!   owner-gated `Next(3,6)`-count CrystalPulse2 (522) split — the velocity
//!   draws are STATE-BEARING and feed the spawns inline. The 522's own
//!   Kill arm (`:73271-73290`, the 10-dust census) lands with the shard.
//! - **80 IceBlock** (`:75533-75553`): the `ai[0] >= 0` sound/dust census +
//!   the tile-127 `WorldGen.KillTile` break (the pool hook owns the tile
//!   write, the ICE_BLOCK arm of `ProjectilePool::kill_hooks_a`).
//!
//! C# sources read WHOLE (the golden rule): the `Kill()` head `:70642-70660`
//! (the `!active` guard + the `num = timeLeft` capture), each arm above, the
//! `Damage()` body `:12487-12516`, `Damage_CanDealDamage` `:12446-12485`
//! (none of the kept kinds is on the refusal list), and the `Entity.Resize`
//! maths `:22353-22359`.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw stays in the C#'s order; the dust/gore
//! BODIES are skipped (the `projectile_ai_misc` convention) and the
//! `Dust.NewDust*` internal rolls go with the dust pool. The `Gore.NewGore`
//! internal kicks DO stay — `Next(10,31)` Y then `Next(-20,21)` X
//! (`Gore.cs:1327-1328`) — plus the explicit position `Next(100)` pair, the
//! `Next(61,64)` type and the two post-write `Next(-10,11)` kicks (the 7-draw
//! `gore()` row, the `celeb2_explosion` precedent; g16 upgrades these census
//! rows to the `GorePool` at its wiring wave). The 459 ring dust draws
//! TWO `next_double` per particle (the `RotatedBy(2π·NextDouble())` angle then
//! the `NextDouble()` radius, `Utils.cs:1675-1684`); the argument evaluation
//! order is left-to-right (`(width/2, 0).RotatedBy(2π·d1) * d2`).
//!
//! MERGER NOTE (the g65 keep-one, research/26i): g65's pushed death-dust
//! bodies (`459 | 709` / `521` / `522` / `80` arms, `projectile_death_dust.rs`
//! @ `b0842907c`) carry these SAME explicit draws inside their bodies — at
//! the g1×g65 merge the census calls here DROP per-kind (the bodies
//! supersede them, the 26i §3 pattern); the STATE (resize + Damage + the
//! 522 split spawns + the tile break) stays g1's.
//!
//! Sound cues (`Item27`/`Item110`/`Item118`) are the sound-lane skip
//! (g21-25/d77 — the 521 Item110 cue `:73294` and the 80 Item27 gate
//! `:75537` are already armed on d77's `kill_tail_cue`, cue-first, per
//! 26f §5); the `PlaySound` draws nothing on this stream in the port's
//! queue convention until the cue lands.
//!
//! CODE-ONLY MODE (W4-g1): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); every expected value is
//! hand-derived from the cited C# lines.

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::projectile::{Projectile, safe_normalize};

/// One gore's draw row (the `celeb2_explosion::gore` shape, the 459 burst):
/// the position `Next(100)` pair, the `Next(61,64)` type, the `NewGore`
/// kicks (`Gore.cs:1327-1328`), the two post-write `Next(-10,11)` velocity
/// kicks (`:73369-73370`).
fn gore(rng: &mut dyn DropRng) {
    let _ = rng.next_range(0, 100);
    let _ = rng.next_range(0, 100);
    let _ = rng.next_range(61, 64);
    let _ = rng.next_range(10, 31);
    let _ = rng.next_range(-20, 21);
    let _ = rng.next_range(-10, 11);
    let _ = rng.next_range(-10, 11);
}

/// One ring-dust particle's two DOUBLE draws (`:73356` etc.): the
/// `RotatedBy(2π·NextDouble())` angle, then the `NextDouble()` radius. The
/// dust body is the skip; only the draws stay.
fn ring_dust(rng: &mut dyn DropRng) {
    let _ = rng.next_double();
    let _ = rng.next_double();
}

/// `Entity.Resize` (`Projectile.cs:22353-22359`): the position snaps to the
/// old centre, the box grows, then `Center = position` recentres on the
/// INTEGER half (`width / 2` on the C#'s int field). The 459's scale-gated
/// resize rides this.
fn resize(p: &mut Projectile, side: f32) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    p.pos = center;
    p.w = side;
    p.h = side;
    p.pos.x -= (side as i32 / 2) as f32;
    p.pos.y -= (side as i32 / 2) as f32;
}

// (d78 FINAL drop, 26f §1 @ fd59c51a: the 452 PhantasmalEye / 454
// PhantasmalSphere / 448 SaucerMissile arms lived here — d78's
// `hostile_death_blast` + `damage_evp_boxed` own the resize state + the
// verbatim Damage_EVP hit, g65's pushed bodies own the dust/gore draws;
// g1's raw-damage `death_burst_player_hit` + standalone censuses DROPPED.)

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

/// W4-g1 lane a — the witnesses (WRITTEN NOT RUN, code-only mode). The draw
/// sequences replay the C#'s explicit `Main.rand` calls in order; the
/// scripted rng returns fixed values so the state-bearing 521 split is
/// hand-checkable.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, CRYSTAL_PULSE, ICE_BLOCK, CHARGED_BLASTER_ORB};

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
