//! S1-D-3 (aiStyle-55) — the Creeper family, `NPC.AI`'s
//! `else if (this.aiStyle == 55)` block (`NPC.cs:32657-32741`), ported 1:1
//! as a PURE velocity machine (code-only mode: no compiler/tests were run
//! on this branch; the witness tests below are written, not executed).
//!
//! Family identity (verified from the C#, not the brief's guess): the ONLY
//! `this.aiStyle = 55;` row in `SetDefaults` is the **Creeper**, type 267
//! (`NPC.cs:12456-12469`, `NPCID.cs:11211`). The Brain of Cthulhu (266) is
//! aiStyle **54** (`NPC.cs:12443`) — style 55 is NOT a Brain phase-2 arm;
//! it is the orbiting Creeper swarm's own machine, reading the Brain's
//! centre through the static `NPC.crimsonBoss` slot (`NPC.cs:32372` sets
//! it from the Brain's `AI_054` head, `:8397`/`:95176` clear it).
//!
//! Seam (the `npc_ai_style16.rs` precedent, commit 8754ef9f): the module
//! only writes the fields the C# arm writes — velocity, `ai[0]`/`ai[1]`,
//! `netUpdate`, and the `TargetClosest` tracking fields through
//! `npc_target::target_closest` (reused unmodified). It does NOT integrate
//! position: the C# arm ends before `UpdateCollision`. The
//! `active = false` despawn (`:32661`) leaves as an
//! [`Ai55Effects::deactivate`] request — the port's `Npc` has no `active`
//! field (the sim's `update_brain_state` pass, `world_sim/npcs.rs:1038-1041`,
//! currently retains the Creepers away when the Brain dies).
//!
//! The live routing is UNCHANGED this round (wiring deferred): the kind
//! match still runs the `update_creeper` stand-in (`npc.rs:13424`), whose
//! audit defects against this block are recorded in ANALYSIS "The Creeper
//! family (S1-D-3 aiStyle-55)" — the missing 700-px leash, the missing
//! `TargetClosest`, the invented `.max(1.0)` distance guard, the
//! divide-then-multiply lunge order, and the `hurt_cd` cooldown standing in
//! for the one-tick `justHit` flag.
//!
//! RNG by construction: the arm's ONLY `Main.rand` draws are the lunge
//! roll at `:32686`,
//! `Main.netMode != 1 && ((Main.expertMode && Main.rand.Next(100) == 0)
//!   || Main.rand.Next(200) == 0)`:
//! * `Main.netMode != 1` folds to true (the port is single-player) — on a
//!   client the C# draws NOTHING here.
//! * Classic (`expertMode == false`): the `&&` short-circuits BEFORE
//!   `Next(100)` — it is never drawn; `Next(200)` is drawn EVERY tick the
//!   Creeper is inside the 90-px ring in state 0, even when the roll fails
//!   (the draw is the gate, not a consequence of it).
//! * Expert: `Next(100)` is drawn every in-ring state-0 tick; `Next(200)`
//!   is drawn ONLY when the first roll is non-zero (the `||`
//!   short-circuits on a zero `Next(100)` — one draw, not two).
//! * The far-orbit branch (`d > 90`, `:32672-32680`) returns BEFORE the
//!   roll — zero draws; the state-1 lunge/leash/recoil tail draws nothing.
//! All draws ride the caller's `UnifiedRandom` (the `Main.rand` analogue,
//! `tr_core::unified_random`); the witnesses mirror the cited draws on a
//! cloned stream and compare the full state (`UnifiedRandom: PartialEq`).

#![allow(dead_code)] // the update_creeper wiring swap is the merge step (S1-D-3)

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// The per-tick context the arm reads besides the NPC and the world.
pub struct Ai55Ctx<'a> {
    /// The `Main.player` slots as `TargetClosest` sees them (the port has
    /// one session player; `npc_target.rs`'s seam).
    pub players: &'a [TargetCandidate],
    /// `Main.npc[NPC.crimsonBoss].Center` (`:32669-32670`, `:32716-32717`)
    /// — `None` is `NPC.crimsonBoss < 0` (`:32659`). The sim's
    /// `update_brain_state` pass writes the Brain's centre anchor
    /// (`world_sim/npcs.rs:993-995`); [`brain_anchor`] reads it.
    pub crimson_boss_center: Option<(f32, f32)>,
    /// `Main.expertMode` (`:32686`, `:32700`).
    pub expert_mode: bool,
    /// `Main.getGoodWorld` (`:32704`) — the ftw expert steer row.
    pub get_good_world: bool,
    /// `NPC.justHit` (`:32724`) — the ONE-TICK strike flag, not a cooldown.
    /// The port's `Npc` has no `justHit` field (the stand-in reads the
    /// `hurt_cd` cooldown, which stays positive for ~11 ticks); the caller
    /// supplies the flag at the wiring step (the `Ai16Ctx::just_hit`
    /// precedent).
    pub just_hit: bool,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed (the S1-D-3p
    /// precedent).
    pub ticker: u64,
}

/// The side effects the pure arm cannot apply itself.
#[derive(Default, Debug, PartialEq)]
pub struct Ai55Effects {
    /// `this.active = false` (`:32661`) — the brainless despawn. The C#
    /// also sets `netUpdate = true` (`:32662`); that write lands on the
    /// NPC directly, only the deactivation needs the pool.
    pub deactivate: bool,
}

/// The `NPC.crimsonBoss` centre as the sim's World anchor carries it
/// (`world_sim/npcs.rs:993-995` — the Brain's `position + size/2`, the
/// `Entity.Center` shape, `Entity.cs:50-55`).
pub fn brain_anchor(world: &World) -> Option<(f32, f32)> {
    if world.brain_present {
        Some((world.brain_x, world.brain_y))
    } else {
        None
    }
}

/// The family entry — `NPC.AI`'s `else if (this.aiStyle == 55)`
/// (`NPC.cs:32657-32741`). The only style-55 kind is the Creeper
/// (`NPC.cs:12460`), so there is no per-type dispatch inside the arm; the
/// `knockBackResist` the recoil branch reads (`:32728`) is the Creeper's
/// `SetDefaults` 0.8f (`NPC.cs:12468`, the port's
/// `Npc::knock_back_resist()` row).
pub fn ai_055_step(
    npc: &mut Npc,
    world: &World,
    ctx: &Ai55Ctx,
    rng: &mut UnifiedRandom,
) -> Ai55Effects {
    let kb = npc.knock_back_resist();
    ai_055_arm(npc, kb, world, ctx, rng)
}

/// The target slot's candidate. The C# indexes `Main.player[this.target]`
/// (255 slots always exist); the port's slice holds the session player, so
/// an unset/stale slot falls back to slot 0 — the same fallback
/// `SetTargetTrackingValues` applies (`NPC.cs:77731-77734`).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `Entity.Center` (`Entity.cs:50-55`): `position + (float)size / 2f`.
fn candidate_center(p: &TargetCandidate) -> (f32, f32) {
    (
        p.x + p.width as f32 / 2.0,
        p.y + p.height as f32 / 2.0,
    )
}

/// `(float)Math.Sqrt(x * x + y * y)` — the C# multiplies and adds in f32,
/// widens to f64 for `Math.Sqrt`, then casts the root back to f32
/// (`:32671`, `:32692`, `:32718`). (For sqrt the f64→f32 double rounding
/// is provably identical to a direct f32 sqrt — 53 ≥ 2·24+2 mantissa bits
/// — but the transcription keeps the C#'s literal shape.)
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA/FNA `Vector2.Normalize()` — `num = 1f / (float)Math.Sqrt(X*X +
/// Y*Y); X *= num; Y *= num`. UNGUARDED: a zero vector gives `1/0 = +inf`
/// and `0 * inf = NaN`, exactly like the C# at `:32703` (the expert steer
/// on a Creeper centred on its target). No `SafeNormalize` substitution.
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// The whole aiStyle-55 arm. `knock_back_resist` is `this.knockBackResist`
/// (`:32728`) — passed explicitly so the `== 0f` recoil-counter branch
/// stays reachable (the Creeper's own row is 0.8f, `NPC.cs:12468`; the
/// branch is dead for vanilla Creepers but is part of the arm, verbatim).
pub fn ai_055_arm(
    npc: &mut Npc,
    knock_back_resist: f32,
    world: &World,
    ctx: &Ai55Ctx,
    rng: &mut UnifiedRandom,
) -> Ai55Effects {
    let mut fx = Ai55Effects::default();

    // `:32659-32664` — `if (NPC.crimsonBoss < 0) { active = false;
    // netUpdate = true; return; }`.
    let (boss_cx, boss_cy) = match ctx.crimson_boss_center {
        Some(c) => c,
        None => {
            fx.deactivate = true;
            npc.net_update = true;
            return fx;
        }
    };

    // `:32665-32699` — the orbit state.
    if npc.ai[0] == 0.0 {
        // `:32667` — the recoil counter resets every orbit tick.
        npc.ai[1] = 0.0;
        // `:32668-32671` — vector98 = Center; the Brain delta and its
        // length.
        let (cx, cy) = npc.center();
        let mut num827 = boss_cx - cx;
        let mut num828 = boss_cy - cy;
        let mut num829 = sqrt_f32(num827, num828);
        // `:32672-32680` — beyond the 90-px ring: the 8-px homeward drift
        // through the 15/16 blend, then RETURN (before the lunge roll —
        // this branch draws no RNG).
        if num829 > 90.0 {
            num829 = 8.0 / num829;
            num827 *= num829;
            num828 *= num829;
            npc.vx = (npc.vx * 15.0 + num827) / 16.0;
            npc.vy = (npc.vy * 15.0 + num828) / 16.0;
            return fx;
        }
        // `:32681-32685` — inside the ring: the drift amplifies while the
        // Manhattan speed is under 8 (strict `<`; Y first, then X).
        if npc.vx.abs() + npc.vy.abs() < 8.0 {
            npc.vy *= 1.05;
            npc.vx *= 1.05;
        }
        // `:32686-32697` — the lunge roll. `Main.netMode != 1` folds to
        // true; the expert `Next(100)` half short-circuits on classic (no
        // draw), and the `||` skips `Next(200)` when the expert roll hits
        // 0. `Next(200)` is drawn even when it fails.
        let lunge = if ctx.expert_mode {
            let expert_roll = rng.next_max(100);
            expert_roll == 0 || rng.next_max(200) == 0
        } else {
            rng.next_max(200) == 0
        };
        if lunge {
            // `:32688` — TargetClosest() re-picks and re-faces (the
            // stand-in skipped this: no target/rect/facing update).
            target_closest(npc, world, ctx.players, true, 55, ctx.ticker);
            // `:32689-32694` — the aim at the picked player's centre:
            // reciprocal FIRST (`num829 = 8f / sqrt`), then the
            // component multiplies — NOT `(dx / d) * 8`. UNGUARDED: a
            // zero delta gives `8/0 = inf` and `0 * inf = NaN` (the
            // stand-in's `.max(1.0)` guard is invented).
            let (cx, cy) = npc.center();
            let p = target_of(npc, ctx.players);
            let (pcx, pcy) = candidate_center(p);
            num827 = pcx - cx;
            num828 = pcy - cy;
            num829 = 8.0 / sqrt_f32(num827, num828);
            npc.vx = num827 * num829;
            npc.vy = num828 * num829;
            // `:32695-32696`.
            npc.ai[0] = 1.0;
            npc.net_update = true;
        }
        // `:32698` — the orbit state returns whether or not the roll hit.
        return fx;
    }

    // `:32700-32714` — the expert homing steer during the lunge: the
    // unguarded Normalize, then the ftw 12-px/49-50 blend or the expert
    // 9-px/99-100 blend. Classic draws and writes nothing here.
    if ctx.expert_mode {
        let (cx, cy) = npc.center();
        let p = target_of(npc, ctx.players);
        let (pcx, pcy) = candidate_center(p);
        let (mut sx, mut sy) = normalize_xna(pcx - cx, pcy - cy);
        if ctx.get_good_world {
            sx *= 12.0;
            sy *= 12.0;
            npc.vx = (npc.vx * 49.0 + sx) / 50.0;
            npc.vy = (npc.vy * 49.0 + sy) / 50.0;
        } else {
            sx *= 9.0;
            sy *= 9.0;
            npc.vx = (npc.vx * 99.0 + sx) / 100.0;
            npc.vy = (npc.vy * 99.0 + sy) / 100.0;
        }
    }

    // `:32715-32721` — the 700-px leash: past it the lunge drops back to
    // the orbit WITHOUT needing a hit (strict `>`; the stand-in has no
    // leash at all, so its Creepers lunge forever).
    let (cx, cy) = npc.center();
    let num830 = boss_cx - cx;
    let num831 = boss_cy - cy;
    if sqrt_f32(num830, num831) > 700.0 {
        npc.ai[0] = 0.0;
    } else {
        // `:32724-32727` — no hit this tick: the lunge holds.
        if !ctx.just_hit {
            return fx;
        }
        // `:32728-32739` — the recoil: a knockback-immune body counts up
        // to 6 hits (`ai[1] > 5f`, strict), anything else drops on the
        // first hit.
        if knock_back_resist == 0.0 {
            npc.ai[1] += 1.0;
            if npc.ai[1] > 5.0 {
                npc.ai[0] = 0.0;
            }
        } else {
            npc.ai[0] = 0.0;
        }
    }
    fx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// A Creeper whose CENTRE is `(cx, cy)` (the 30×30 box,
    /// `NPC.cs:12458-12459`).
    fn creeper_at_center(cx: f32, cy: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Creeper, cx - 15.0, cy - 15.0, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// The classic context: one session player, the Brain at
    /// `(1000, 1000)`, no expert/ftw, no hit.
    fn ctx_classic<'a>(players: &'a [TargetCandidate]) -> Ai55Ctx<'a> {
        Ai55Ctx {
            players,
            crimson_boss_center: Some((1000.0, 1000.0)),
            expert_mode: false,
            get_good_world: false,
            just_hit: false,
            ticker: 7,
        }
    }

    /// `NPC.cs:32659-32664` — no Brain: deactivate + netUpdate, nothing
    /// else touched, and ZERO RNG draws (the return precedes the roll).
    #[test]
    fn s1_d_3_s55_no_brain_deactivates_without_draws() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let mut ctx = ctx_classic(&players);
        ctx.crimson_boss_center = None;
        let mut n = creeper_at_center(1010.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 3.0;
        n.vx = 2.0;
        n.vy = -1.0;
        let mut rng = UnifiedRandom::new(42);
        let mirror = rng.clone();
        let fx = ai_055_step(&mut n, &w, &ctx, &mut rng);
        assert!(fx.deactivate, "active = false (:32661)");
        assert!(n.net_update, "netUpdate = true (:32662)");
        assert_eq!(n.ai, [1.0, 3.0, 0.0, 0.0], "no state write");
        assert_eq!((n.vx, n.vy), (2.0, -1.0), "no velocity write");
        assert_eq!(rng, mirror, "the despawn arm draws nothing");
    }

    /// `NPC.cs:32667-32680` — the far orbit: ai[1] resets, the 8-px
    /// homeward drift rides the 15/16 blend in the C#'s multiply-then-
    /// blend order, and the early return precedes the roll (zero draws).
    /// The fixture's numbers are exact binary fractions, so the blend is
    /// pinned bit-for-bit.
    #[test]
    fn s1_d_3_s55_the_far_orbit_blends_home_and_draws_nothing() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        // Centre (1160, 1000): dx = -160, dy = 0, d = 160 > 90.
        let mut n = creeper_at_center(1160.0, 1000.0);
        n.ai[1] = 4.0;
        n.vx = 2.0;
        n.vy = -3.0;
        let mut rng = UnifiedRandom::new(42);
        let mirror = rng.clone();
        let fx = ai_055_step(&mut n, &w, &ctx, &mut rng);
        assert_eq!(fx, Ai55Effects::default());
        assert_eq!(n.ai[1], 0.0, "the counter resets (:32667)");
        // s = 8/160 = 0.05; dx*s = -8; (2*15 - 8)/16 = 22/16 = 1.375.
        assert_eq!(n.vx, (2.0f32 * 15.0 + (-160.0f32) * (8.0 / 160.0)) / 16.0);
        assert_eq!(n.vx, 1.375);
        assert_eq!(n.vy, (-3.0f32 * 15.0 + 0.0) / 16.0);
        assert_eq!(n.vy, -2.8125);
        assert_eq!(n.ai[0], 0.0, "still orbiting");
        assert_eq!(rng, mirror, "the >90 return precedes the roll");
    }

    /// `NPC.cs:32672` — the ring bound is STRICT `>`: a Creeper exactly
    /// 90 px out stays in the ring (it amplifies and rolls), one hair
    /// past 90 orbits home.
    #[test]
    fn s1_d_3_s55_the_90_ring_bound_is_strict() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        // Exactly 90: sqrt(8100) = 90 exactly.
        let mut n = creeper_at_center(1090.0, 1000.0);
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = rng.clone();
        let _ = mirror.next_max(200); // the in-ring tick draws once
        ai_055_step(&mut n, &w, &ctx, &mut rng);
        assert_eq!(rng, mirror, "d == 90 is INSIDE the ring (rolls)");
        // 90.5: outside, no draw.
        let mut n2 = creeper_at_center(1090.5, 1000.0);
        let mut rng2 = UnifiedRandom::new(42);
        let mirror2 = rng2.clone();
        ai_055_step(&mut n2, &w, &ctx, &mut rng2);
        assert_eq!(rng2, mirror2, "d > 90 orbits without a draw");
        assert!(n2.vx < 0.0, "the blend pulls toward the Brain");
    }

    /// `NPC.cs:32681-32685` — the amplify gate is the Manhattan speed
    /// strictly under 8: 3+4 = 7 amplifies both axes by 1.05, 4+4 = 8
    /// does not.
    #[test]
    fn s1_d_3_s55_the_amplify_gate_is_strict_manhattan_8() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        let mut n = creeper_at_center(1010.0, 1000.0);
        n.vx = 3.0;
        n.vy = 4.0;
        let mut rng = UnifiedRandom::new(42);
        ai_055_step(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.vx, 3.0f32 * 1.05);
        assert_eq!(n.vy, 4.0f32 * 1.05);
        let mut n2 = creeper_at_center(1010.0, 1000.0);
        n2.vx = 4.0;
        n2.vy = 4.0;
        let mut rng2 = UnifiedRandom::new(42);
        ai_055_step(&mut n2, &w, &ctx, &mut rng2);
        assert_eq!((n2.vx, n2.vy), (4.0, 4.0), "sum == 8 does not amplify");
    }

    /// `NPC.cs:32686` (classic) — exactly ONE `Next(200)` per in-ring
    /// state-0 tick, drawn even when the roll fails; the lunge fires iff
    /// the draw is 0. The mirrored stream pins the draw count and order
    /// by full-state comparison.
    #[test]
    fn s1_d_3_s55_the_classic_roll_is_one_next_200_even_on_failure() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        for seed in [0i32, 1, 42, 999, 12345] {
            let mut n = creeper_at_center(1010.0, 1000.0);
            let mut rng = UnifiedRandom::new(seed);
            let mut mirror = rng.clone();
            let roll = mirror.next_max(200);
            ai_055_step(&mut n, &w, &ctx, &mut rng);
            assert_eq!(rng, mirror, "seed {seed}: exactly one Next(200)");
            assert_eq!(
                n.ai[0] == 1.0,
                roll == 0,
                "seed {seed}: the lunge iff the roll is 0"
            );
        }
    }

    /// `NPC.cs:32686` (expert) — `Next(100)` first, every in-ring tick;
    /// `Next(200)` only when the first roll is non-zero (the `||`
    /// short-circuit). The mirrored stream pins both the two-draw and the
    /// one-draw shapes.
    #[test]
    fn s1_d_3_s55_the_expert_roll_draws_100_then_conditionally_200() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let mut ctx = ctx_classic(&players);
        ctx.expert_mode = true;
        let mut saw_one_draw = false;
        let mut saw_two_draws = false;
        for seed in 0..4000i32 {
            let mut n = creeper_at_center(1010.0, 1000.0);
            let mut rng = UnifiedRandom::new(seed);
            let mut mirror = rng.clone();
            let roll100 = mirror.next_max(100);
            let roll200 = if roll100 != 0 {
                Some(mirror.next_max(200))
            } else {
                None
            };
            ai_055_step(&mut n, &w, &ctx, &mut rng);
            assert_eq!(rng, mirror, "seed {seed}: the expert draw shape");
            let lunge = roll100 == 0 || roll200 == Some(0);
            assert_eq!(n.ai[0] == 1.0, lunge, "seed {seed}: the lunge gate");
            if roll100 == 0 {
                saw_one_draw = true;
            } else {
                saw_two_draws = true;
            }
            if saw_one_draw && saw_two_draws {
                break;
            }
        }
        assert!(saw_one_draw, "a Next(100)==0 seed exists (one draw)");
        assert!(saw_two_draws, "a Next(100)!=0 seed exists (two draws)");
    }

    /// `NPC.cs:32689-32694` — the lunge velocity is reciprocal-FIRST:
    /// `num829 = 8f / d` then `dx * num829`. The (15, 20) fixture (d = 25
    /// exactly) discriminates the orderings in f32: `8/25` rounds DOWN
    /// (0.31999999284…), so `15 * (8/25)` rounds to 4.7999997 while
    /// `(15/25) * 8` gives 4.8000002 — the stand-in's `dx / d * 8.0` is
    /// the WRONG one.
    #[test]
    fn s1_d_3_s55_the_lunge_multiplies_by_the_reciprocal_first() {
        let w = air();
        // Player centre = creeper centre + (15, 20): d = 25 exactly.
        let players = [cand(1010.0 + 15.0 - 10.0, 1000.0 + 20.0 - 21.0)];
        let ctx = ctx_classic(&players);
        // Hunt a classic lunge seed (Next(200) == 0).
        let mut fired = false;
        for seed in 0..20000i32 {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(200) != 0 {
                continue;
            }
            let mut n = creeper_at_center(1010.0, 1000.0);
            let mut rng = UnifiedRandom::new(seed);
            ai_055_step(&mut n, &w, &ctx, &mut rng);
            assert_eq!(n.ai[0], 1.0);
            let s = 8.0f32 / 25.0f32;
            assert_eq!(n.vx, 15.0f32 * s, "dx * (8/d), seed {seed}");
            assert_eq!(n.vy, 20.0f32 * s, "dy * (8/d), seed {seed}");
            assert_ne!(
                15.0f32 * s,
                (15.0f32 / 25.0f32) * 8.0f32,
                "the fixture discriminates the two orderings"
            );
            fired = true;
            break;
        }
        assert!(fired, "a lunge seed exists");
    }

    /// `NPC.cs:32688`, `:32695-32696` — the lunge runs TargetClosest (the
    /// shared selection: the rect tracks the picked player, the facing
    /// aims, the ticker stamps) and sets ai[0] = 1 + netUpdate.
    #[test]
    fn s1_d_3_s55_the_lunge_retargets_and_flags_net_update() {
        let w = air();
        // The player to the LEFT of the Creeper.
        let players = [cand(900.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        let mut fired = false;
        for seed in 0..20000i32 {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(200) != 0 {
                continue;
            }
            let mut n = creeper_at_center(1010.0, 1000.0);
            n.net_update = false;
            let mut rng = UnifiedRandom::new(seed);
            ai_055_step(&mut n, &w, &ctx, &mut rng);
            assert_eq!(n.ai[0], 1.0);
            assert!(n.net_update, "netUpdate = true (:32696)");
            assert_eq!(n.target, 0);
            assert_eq!(n.target_rect, (900, 979, 20, 42), "the rect tracks");
            assert_eq!(n.facing, -1, "the player is to the left");
            assert_eq!(n.target_set_frame, 7, "the ticker stamp (:77780)");
            assert!(n.vx < 0.0, "the lunge aims at the player");
            fired = true;
            break;
        }
        assert!(fired, "a lunge seed exists");
    }

    /// `NPC.cs:32692` — the lunge division is UNGUARDED: a target centred
    /// on the Creeper gives `8/0 = inf` and `0 * inf = NaN`. The
    /// stand-in's `.max(1.0)` floor is invented; the faithful arm keeps
    /// the C#'s NaN.
    #[test]
    fn s1_d_3_s55_the_lunge_has_no_invented_distance_floor() {
        let w = air();
        // Player centre == creeper centre (1010, 1000).
        let players = [cand(1010.0 - 10.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        let mut fired = false;
        for seed in 0..20000i32 {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(200) != 0 {
                continue;
            }
            let mut n = creeper_at_center(1010.0, 1000.0);
            let mut rng = UnifiedRandom::new(seed);
            ai_055_step(&mut n, &w, &ctx, &mut rng);
            assert!(n.vx.is_nan(), "0 * (8/0) = NaN, like the C#");
            assert!(n.vy.is_nan());
            fired = true;
            break;
        }
        assert!(fired, "a lunge seed exists");
    }

    /// `NPC.cs:32715-32727` — the state-1 tail: the 700-px leash is
    /// strict `>` (exactly 700 holds the lunge), it resets WITHOUT a hit,
    /// and inside it a quiet tick returns with no state write and no
    /// draws. Classic expert-off: the whole tail draws nothing.
    #[test]
    fn s1_d_3_s55_the_700_leash_is_strict_and_needs_no_hit() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        // Exactly 700 out, no hit: the lunge holds.
        let mut n = creeper_at_center(1700.0, 1000.0);
        n.ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(42);
        let mirror = rng.clone();
        ai_055_step(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.ai[0], 1.0, "d == 700 is inside the leash");
        assert_eq!(rng, mirror, "the state-1 tail draws nothing");
        // 700.5 out, no hit: the leash drops the lunge.
        let mut n2 = creeper_at_center(1700.5, 1000.0);
        n2.ai[0] = 1.0;
        let mut rng2 = UnifiedRandom::new(42);
        let mirror2 = rng2.clone();
        ai_055_step(&mut n2, &w, &ctx, &mut rng2);
        assert_eq!(n2.ai[0], 0.0, "d > 700 resets without a hit");
        assert_eq!(rng2, mirror2);
    }

    /// `NPC.cs:32724-32739` — the recoil: a knockback-RESISTING body
    /// (the Creeper's 0.8) drops to the orbit on the first hit; a
    /// knockback-immune body (kb == 0) counts hits and drops only past 5
    /// (the 6th hit — strict `>`).
    #[test]
    fn s1_d_3_s55_the_recoil_counts_six_hits_only_at_kb_zero() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let mut ctx = ctx_classic(&players);
        ctx.just_hit = true;
        // kb 0.8 (the Creeper's row): one hit drops the lunge.
        let mut n = creeper_at_center(1010.0, 1000.0);
        n.ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(42);
        ai_055_step(&mut n, &w, &ctx, &mut rng);
        assert_eq!(n.ai[0], 0.0, "the first hit recoils (kb != 0)");
        // kb 0: five hits hold, the sixth drops.
        let mut n2 = creeper_at_center(1010.0, 1000.0);
        n2.ai[0] = 1.0;
        let mut rng2 = UnifiedRandom::new(42);
        for hit in 1..=5 {
            ai_055_arm(&mut n2, 0.0, &w, &ctx, &mut rng2);
            assert_eq!(n2.ai[1], hit as f32, "the counter increments");
            assert_eq!(n2.ai[0], 1.0, "hit {hit} holds (ai[1] <= 5)");
        }
        ai_055_arm(&mut n2, 0.0, &w, &ctx, &mut rng2);
        assert_eq!(n2.ai[0], 0.0, "the sixth hit drops (ai[1] > 5)");
    }

    /// `NPC.cs:32700-32713` — the expert steer: the unguarded Normalize,
    /// the 9-px aim through the 99/100 blend (classic writes nothing).
    /// The (30, 40) fixture normalizes through 1/50 exactly.
    #[test]
    fn s1_d_3_s55_the_expert_steer_blends_the_normalized_aim() {
        let w = air();
        let players = [cand(1010.0 + 30.0 - 10.0, 1000.0 + 40.0 - 21.0)];
        let mut ctx = ctx_classic(&players);
        ctx.expert_mode = true;
        let mut n = creeper_at_center(1010.0, 1000.0);
        n.ai[0] = 1.0;
        n.vx = 5.0;
        n.vy = -2.0;
        let mut rng = UnifiedRandom::new(42);
        let mirror = rng.clone();
        ai_055_step(&mut n, &w, &ctx, &mut rng);
        let num = 1.0f32 / 50.0f32;
        let sx = 30.0f32 * num * 9.0;
        let sy = 40.0f32 * num * 9.0;
        assert_eq!(n.vx, (5.0f32 * 99.0 + sx) / 100.0);
        assert_eq!(n.vy, (-2.0f32 * 99.0 + sy) / 100.0);
        assert_eq!(n.ai[0], 1.0, "the steer holds the lunge");
        assert_eq!(rng, mirror, "the steer draws nothing");
        // Classic: no steer at all.
        let mut ctx2 = ctx_classic(&players);
        ctx2.expert_mode = false;
        let mut n2 = creeper_at_center(1010.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.vx = 5.0;
        n2.vy = -2.0;
        let mut rng2 = UnifiedRandom::new(42);
        ai_055_step(&mut n2, &w, &ctx2, &mut rng2);
        assert_eq!((n2.vx, n2.vy), (5.0, -2.0), "classic keeps the flight");
    }

    /// `NPC.cs:32704-32708` — the ftw row: 12-px aim, the 49/50 blend.
    #[test]
    fn s1_d_3_s55_the_ftw_steer_uses_12_and_the_49_50_blend() {
        let w = air();
        let players = [cand(1010.0 + 30.0 - 10.0, 1000.0 + 40.0 - 21.0)];
        let mut ctx = ctx_classic(&players);
        ctx.expert_mode = true;
        ctx.get_good_world = true;
        let mut n = creeper_at_center(1010.0, 1000.0);
        n.ai[0] = 1.0;
        n.vx = 5.0;
        n.vy = -2.0;
        let mut rng = UnifiedRandom::new(42);
        ai_055_step(&mut n, &w, &ctx, &mut rng);
        let num = 1.0f32 / 50.0f32;
        let sx = 30.0f32 * num * 12.0;
        let sy = 40.0f32 * num * 12.0;
        assert_eq!(n.vx, (5.0f32 * 49.0 + sx) / 50.0);
        assert_eq!(n.vy, (-2.0f32 * 49.0 + sy) / 50.0);
    }

    /// The stream-clone replay: ten in-ring ticks (the arm does not
    /// integrate position, so the Creeper stays in the ring), classic —
    /// the mirrored stream draws one `Next(200)` per state-0 tick and
    /// NOTHING after the lunge; the full state matches at the end.
    #[test]
    fn s1_d_3_s55_the_ten_tick_replay_matches_the_mirrored_stream() {
        let w = air();
        let players = [cand(1100.0, 1000.0 - 21.0)];
        let ctx = ctx_classic(&players);
        let mut n = creeper_at_center(1010.0, 1000.0);
        let mut rng = UnifiedRandom::new(2026);
        let mut mirror = rng.clone();
        for _ in 0..10 {
            if n.ai[0] == 0.0 {
                let _ = mirror.next_max(200);
            }
            ai_055_step(&mut n, &w, &ctx, &mut rng);
        }
        assert_eq!(rng, mirror, "the draw-per-tick shape over ten ticks");
    }

    /// `brain_anchor` — the World anchor the sim's `update_brain_state`
    /// writes (`world_sim/npcs.rs:993-995`, `:1039`).
    #[test]
    fn s1_d_3_s55_the_brain_anchor_reads_the_world_flag() {
        let mut w = air();
        assert_eq!(brain_anchor(&w), None, "no Brain -> crimsonBoss < 0");
        w.brain_present = true;
        w.brain_x = 1234.0;
        w.brain_y = 5678.0;
        assert_eq!(brain_anchor(&w), Some((1234.0, 5678.0)));
    }
}
