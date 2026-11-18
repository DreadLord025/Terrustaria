//! S1-D-3 (aiStyle-53) — Plantera's Tentacle, `NPC.AI`'s
//! `else if (this.aiStyle == 53)` block (`NPC.cs:32241-32369`), ported 1:1
//! as a PURE velocity machine (code-only mode: no compiler/tests were run
//! on this branch; the witness tests below are written, not executed).
//!
//! Family identity, verified from the C# (NOT the projectile sentries —
//! `Projectile.cs`'s aiStyle 53 `AI_053_FrostHydra` is a different family,
//! landed with S1-D-4b-2): NPC aiStyle 53 has exactly ONE member,
//! `NPCID.PlanterasTentacle` (264) (`ID/NPCID.cs:11205`), the only
//! `this.aiStyle = 53;` row in `SetDefaults` (`NPC.cs:12410-12422`: 24×24,
//! damage 60, defense 20, lifeMax 1000, `noGravity` + `noTileCollide`,
//! HitSound NPCHit1 / DeathSound NPCDeath1, stock knockBackResist).
//! There is no `AI_053_*` method in `NPC.cs` — the arm is inline in `AI()`.
//! The tentacles spawn from Plantera's own aiStyle-51 arm
//! (`NPC.NewNPC(..., 264, base.whoAmI)`, `NPC.cs:31993`/`:32003`/`:32022`);
//! the expert hook-linked batch writes `ai[3] = hookSlot + 1`
//! (`:32004`), which this arm reads at `:32261-32264`.
//!
//! The port has no `NpcKind::PlanterasTentacle` yet (Plantera herself runs
//! the aiStyle-51 v1 stand-in, `npc.rs::update_plantera`), so this module
//! lands UNWIRED (the `npc_ai_style16.rs` fish precedent, 8754ef9f): pure
//! fns + the [`Ai53Ctx`]/[`Ai53Effects`] seam, keyed on the vanilla type so
//! the branch is reachable and testable before the kind lands. The wiring
//! step adds the kind + the `SetDefaults` row + the style-axis claim.
//!
//! Seam: the module only writes the fields the C# arm writes — velocity,
//! `ai[0]`/`ai[1]`, `localAI[0]`, `spriteDirection` (`Npc::sprite_dir`),
//! `rotation`, `netUpdate` — plus the orphan death (see the fold below).
//! It does NOT integrate position: the C# arm ends before
//! `UpdateCollision`; with `noGravity` + `noTileCollide` the generic pass
//! integrates straight through everything. `TargetClosest()` (`:32276`)
//! rides `npc_target::target_closest` UNMODIFIED (style 53 — the
//! `:77761` fighter arm is style-3-only, so no `directionY` write).
//!
//! Documented folds (all cited):
//! * `Main.netMode != 1` → true (the port is single-player) — the rearm
//!   timer block (`:32265-32275`) always runs; the ftw reflect draw's
//!   `netMode` is absent (the draw is unconditional under `getGoodWorld`).
//! * `StrikeNPCNoInteraction(9999, 0f, 0)` (`:32256`, the orphan strike)
//!   → an [`Ai53Effects::orphan_strike`] request. The C# path is
//!   `StrikeNPC(9999, 0, 0, crit: false, fromNet: false, owner: 255)`
//!   (`NPC.cs:81184-81187`): `flag = Damage >= 9999 && owner == 255`
//!   (`:81237`) → the instant-kill branch; knockBack 0 skips the whole
//!   knockback block (`:81387`). The 262/263/264 `HitEffect`
//!   (`:87165-87199`) is the dust/gore burst — a documented skip like
//!   every other NPC arm this round (its `Main.rand.Next(3)` /
//!   `Next(-2, 3)` rolls die with the NPC on that same tick; the port's
//!   death pass has no dust feed, so the strike lands as `hp = 0` +
//!   removal, and the stream divergence starts only at a tick where the
//!   original NPC is already gone). The wiring step routes the request
//!   through the strike path; the pure arm marks the death itself the
//!   way the port's `active = false` arms do (`npc.hp = 0`).
//! * `Main.expertMode` / `Main.getGoodWorld` have no runtime flags yet
//!   (the seeds only ride worldgen) — [`Ai53Ctx`] carries them and the
//!   wiring passes the world's flags; the rows themselves are complete.
//! * `reflectsProjectiles` has no `Npc` field yet — the ftw per-tick
//!   re-roll leaves as [`Ai53Effects::reflects_projectiles`]; the DRAW
//!   is kept in-stream regardless (the gate is `getGoodWorld` alone).
//! * `this.active = false` (`:32298`, the anchor-lost death) → `hp = 0`
//!   (the port's liveness convention, `npc.rs`'s `active = false` arms).
//!
//! ## RNG by construction
//!
//! The arm's ONLY `Main.rand` draws, in C# order:
//! 1. the ftw reflect re-roll `Next(10)` (`:32245`) — EVERY tick while
//!    `Main.getGoodWorld`, drawn BEFORE the `NPC.plantBoss < 0` early
//!    return (`:32254-32259`): an orphan tentacle on a getGood world
//!    still consumes one draw on its death tick. Kept even though the
//!    reflect flag itself is an effect (the flag write is not a draw
//!    gate — the draw happens on both branches of `== 0`).
//! 2. the rearm triple (`:32270-32272`), only when `localAI[0]` counts
//!    down to `<= 0`: `Next(120, 480)` (the timer), then
//!    `Next(-100, 101)` (ai[0]), then `Next(-100, 101)` (ai[1]) — in
//!    that order, and drawn BEFORE the anchor-active check
//!    (`:32296-32300`): a tentacle whose anchor died still re-arms (and
//!    draws) on the tick it deactivates.
//! Nothing else draws: the steering, the caps and the rotation tail are
//! pure maths. `UnifiedRandom` (`tr_core::unified_random`) is the exact
//! `Main.rand` stream; the witnesses mirror the cited `Next` calls on a
//! cloned stream and compare state (`PartialEq`), so a drifted order or
//! a skipped draw fails by construction.

#![allow(dead_code)] // the kind + style-axis wiring is the merge step (S1-D-3)

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// `NPCID.PlanterasTentacle` (`ID/NPCID.cs:11205`) — the family's one
/// member (`SetDefaults`, `NPC.cs:12410-12422`).
pub const PLANTERAS_TENTACLE: i32 = 264;

/// One `Main.npc[slot]` as the arm reads it. The C# reads two slots:
/// `NPC.plantBoss` (the hp bands) and `num790` (the anchor — the boss, or
/// the `ai[3] - 1` hook). The port's NPC list is a `Vec`, so the caller
/// resolves both slots and hands the arm their live state.
#[derive(Clone, Copy, Debug)]
pub struct Ai53NpcView {
    /// `Main.npc[slot].active` (`:32296`). The port's liveness is
    /// `hp > 0` (`npc.rs::can_be_chased_by`'s convention).
    pub active: bool,
    /// `position` (`:32301-32302`).
    pub x: f32,
    pub y: f32,
    /// `width`/`height` — the anchor centre is `position + (width / 2)`
    /// with the C#'s INTEGER division (`:32301-32302`, `(float)(... / 2)`
    /// casts an int quotient; odd widths lose the half pixel).
    pub width: i32,
    pub height: i32,
    /// `life`/`lifeMax` — read on `NPC.plantBoss` ONLY (`:32279-32289`),
    /// never on the hook anchor.
    pub life: i32,
    pub life_max: i32,
}

/// The per-tick context the arm reads besides the NPC and the world.
pub struct Ai53Ctx<'a> {
    /// The `Main.player` slots as `TargetClosest` sees them (the
    /// `npc_target.rs` seam).
    pub players: &'a [TargetCandidate],
    /// `NPC.plantBoss` (`NPC.cs:6127`) — the live Plantera's slot, or −1.
    /// `OnSpawn` sets it (`:8416-8419`), `checkDead`/`SetDefaults` clear
    /// it to −1 (`:8391-8394`, `:95175`).
    pub plant_boss: i32,
    /// `Main.npc[NPC.plantBoss]` — the hp-band reads (`:32279-32289`).
    /// `None` when `plant_boss < 0` (the arm returns before reading it).
    pub boss: Option<Ai53NpcView>,
    /// `Main.npc[num790]` (`:32260-32264`, `:32296-32302`) — the anchor:
    /// the boss, or `ai[3] - 1` when `ai[3] > 0` (the expert hook link,
    /// written by Plantera's spawn batch `:32004`). `None` models a slot
    /// the port does not hold; the C# array always has 200 slots, so an
    /// out-of-range `ai[3]` reads an INACTIVE stock NPC — the caller
    /// passes `Some(inactive view)` for that, not `None`, when the slot
    /// exists but is empty. `None` is treated as inactive (the same
    /// `!active` death).
    pub anchor: Option<Ai53NpcView>,
    /// `Main.expertMode` (`:32287-32291`).
    pub expert_mode: bool,
    /// `Main.getGoodWorld` (`:32243-32253`, `:32292-32295`).
    pub get_good_world: bool,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed (the S1-D-3p
    /// precedent).
    pub ticker: u64,
}

/// The side effects the pure arm cannot apply itself.
#[derive(Default, Debug, PartialEq)]
pub struct Ai53Effects {
    /// The orphan strike `StrikeNPCNoInteraction(9999, 0f, 0)`
    /// (`:32256`): `(damage, knockBack, hitDirection)`, verbatim. The
    /// arm also sets `npc.hp = 0` (the port's `active = false`
    /// convention) so the pure step is self-contained; the wiring may
    /// route the strike through the real damage path instead (the
    /// 262/263/264 `HitEffect` dust/gore is the documented skip).
    pub orphan_strike: Option<(i32, f32, i32)>,
    /// The ftw per-tick `reflectsProjectiles` re-roll (`:32243-32253`):
    /// `Some(true)` on `Next(10) == 0`, `Some(false)` otherwise, `None`
    /// on a non-ftw tick (the C# leaves the flag untouched). The port's
    /// `Npc` has no `reflects_projectiles` field yet — the wiring adds
    /// it and applies this write.
    pub reflects_projectiles: Option<bool>,
}

/// The anchor centre — `Main.npc[num790].position + (width / 2, height / 2)`
/// (`NPC.cs:32301-32303`). The halves are INTEGER divisions cast to
/// float, exactly like the C# (`(float)(Main.npc[num790].width / 2)` —
/// `width` is `int`, so `width / 2` truncates BEFORE the cast).
fn anchor_center(anchor: &Ai53NpcView) -> (f32, f32) {
    (
        anchor.x + (anchor.width / 2) as f32,
        anchor.y + (anchor.height / 2) as f32,
    )
}

/// The family entry — `NPC.AI`'s `else if (this.aiStyle == 53)`
/// (`NPC.cs:32241-32369`). `npc` must carry the tentacle's 24×24 box
/// (the `SetDefaults` row `:12412-12413`); until the kind lands, the
/// caller sets `size_override = Some((24.0, 24.0))` (the King Slime
/// precedent, `npc.rs:1051`).
pub fn ai_053_step(
    npc: &mut Npc,
    world: &World,
    ctx: &Ai53Ctx,
    rng: &mut UnifiedRandom,
) -> Ai53Effects {
    ai_053_arm(PLANTERAS_TENTACLE, npc, world, ctx, rng)
}

/// The whole aiStyle-53 arm, keyed on the vanilla `NPC.type` (the family
/// has one member; the parameter keeps the shape of the other unwired
/// family modules so a future clone — none exists in 1.4.5.8 — slots in).
pub fn ai_053_arm(
    ty: i32,
    npc: &mut Npc,
    world: &World,
    ctx: &Ai53Ctx,
    rng: &mut UnifiedRandom,
) -> Ai53Effects {
    debug_assert_eq!(ty, PLANTERAS_TENTACLE, "aiStyle 53 has one member (264)");
    let _ = ty; // the debug_assert compiles out in release
    let mut fx = Ai53Effects::default();

    // `:32243-32253` — the ftw reflect re-roll. The `Next(10)` draw runs
    // on EVERY ftw tick, BEFORE the orphan check below; both branches
    // write the flag (true on 0, false otherwise — it is a per-tick
    // re-roll, not a latch).
    if ctx.get_good_world {
        fx.reflects_projectiles = Some(rng.next_max(10) == 0);
    }

    // `:32254-32259` — no Plantera: the instant-kill strike + netUpdate,
    // then RETURN (the rearm timer and TargetClosest never run).
    if ctx.plant_boss < 0 {
        fx.orphan_strike = Some((9999, 0.0, 0));
        // The strike's `flag` (`NPC.cs:81237`, 9999 + owner 255) kills
        // outright; the port's death convention is `hp = 0`.
        npc.hp = 0;
        npc.net_update = true;
        return fx;
    }

    // `:32260-32264` — the anchor slot: the boss, or the expert hook
    // (`ai[3]` carries hookSlot + 1, `:32004`). The cast truncates.
    // (Resolved by the caller into `ctx.anchor`; the C# reads
    // `Main.npc[num790]` live at `:32296`/`:32301`.)

    // `:32265-32275` — the rearm timer (`Main.netMode != 1` folds to
    // true). The decrement runs EVERY tick; at `<= 0` the triple draw
    // fires IN ORDER (timer, ai[0], ai[1]) — BEFORE the anchor-active
    // check below, so a tentacle whose anchor just died still draws.
    npc.local_ai[0] -= 1.0;
    if npc.local_ai[0] <= 0.0 {
        npc.local_ai[0] = rng.next_range(120, 480) as f32;
        npc.ai[0] = rng.next_range(-100, 101) as f32;
        npc.ai[1] = rng.next_range(-100, 101) as f32;
        npc.net_update = true;
    }

    // `:32276` — `this.TargetClosest()` (faceTarget defaults true). The
    // arm never reads `this.target` afterwards — the call only writes
    // the tracking fields (direction/directionY/targetRect/netUpdate).
    target_closest(npc, world, ctx.players, true, 53, ctx.ticker);

    // `:32277-32295` — the accel `num791` and the orbit radius `num792`.
    // The hp bands read `NPC.plantBoss` (NOT the hook anchor), as DOUBLE
    // comparisons (`(double)life < (double)lifeMax * 0.25`).
    let mut num791 = 0.2f32;
    let mut num792 = 200.0f32;
    let boss = ctx.boss.expect("plant_boss >= 0 implies a boss view");
    if (boss.life as f64) < (boss.life_max as f64) * 0.25 {
        num792 += 100.0;
    }
    if (boss.life as f64) < (boss.life_max as f64) * 0.1 {
        num792 += 100.0;
    }
    if ctx.expert_mode {
        // `:32289` — the expert radius rides the TENTACLE's own hp, in
        // f32 (`(1f - (float)this.life / (float)this.lifeMax) * 300f`).
        num792 += (1.0 - npc.hp as f32 / npc.max_hp as f32) * 300.0;
        num791 += 0.3;
    }
    if ctx.get_good_world {
        num791 += 4.0;
    }

    // `:32296-32300` — the anchor lost: `active = false`, RETURN. (The
    // `NPC.plantBoss < 0` half of the gate is already covered above;
    // `plantBoss` cannot change between the two reads in one tick.)
    let anchor_alive = ctx.anchor.is_some_and(|a| a.active);
    if !anchor_alive {
        npc.hp = 0; // `this.active = false` — the port's convention.
        return fx;
    }
    let anchor = ctx.anchor.unwrap();

    // `:32301-32310` — the orbit target: the anchor centre + the
    // (ai[0], ai[1]) offset, normalized to the radius. The C# builds
    // `num797 = (centre.X + ai[0]) - centre.X` — the add-then-subtract
    // is KEPT (f32 rounding: for a large centre and a small offset the
    // round-trip can lose low bits; simplifying to `ai[0]` would drift).
    let (num793, num794) = anchor_center(&anchor);
    let num795 = num793 + npc.ai[0];
    let num796 = num794 + npc.ai[1];
    let mut num797 = num795 - num793;
    let mut num798 = num796 - num794;
    // `:32308` — `num792 / (float)Math.Sqrt(num797*num797 + num798*num798)`:
    // the sum is f32, `Math.Sqrt` is DOUBLE, the result casts back to
    // f32 BEFORE the divide. NO zero guard: at ai == (0, 0) the C#
    // produces Infinity/NaN and the steering compares all fail (NaN
    // comparisons are false) — the velocity is left untouched. Rust's
    // IEEE f32/f64 reproduces that exactly; do NOT add a guard.
    let num799 = num792 / ((num797 * num797 + num798 * num798) as f64).sqrt() as f32;
    num797 *= num799;
    num798 *= num799;

    // `:32311-32342` — the four steering blocks. The compares ride
    // `base.position` (NOT Center); the sign-reversal damp `*= 0.9`
    // fires only when the velocity still points AGAINST the pull after
    // the accel add. The X pair is if/else-if; the Y pair is a SEPARATE
    // if/else-if (both axes steer on the same tick).
    if npc.x < num793 + num797 {
        npc.vx += num791;
        if npc.vx < 0.0 && num797 > 0.0 {
            npc.vx *= 0.9;
        }
    } else if npc.x > num793 + num797 {
        npc.vx -= num791;
        if npc.vx > 0.0 && num797 < 0.0 {
            npc.vx *= 0.9;
        }
    }
    if npc.y < num794 + num798 {
        npc.vy += num791;
        if npc.vy < 0.0 && num798 > 0.0 {
            npc.vy *= 0.9;
        }
    } else if npc.y > num794 + num798 {
        npc.vy -= num791;
        if npc.vy > 0.0 && num798 < 0.0 {
            npc.vy *= 0.9;
        }
    }

    // `:32343-32358` — the four hard caps, in the C#'s order (X > 8,
    // X < −8, Y > 8, Y < −8). NaN fails every compare and passes through.
    if npc.vx > 8.0 {
        npc.vx = 8.0;
    }
    if npc.vx < -8.0 {
        npc.vx = -8.0;
    }
    if npc.vy > 8.0 {
        npc.vy = 8.0;
    }
    if npc.vy < -8.0 {
        npc.vy = -8.0;
    }

    // `:32359-32368` — the aim rotation: TWO separate ifs (not
    // if/else-if); at num797 == 0 neither fires and spriteDirection /
    // rotation keep their old values. The left-facing fold adds the
    // LITERAL `3.14f`, not PI (the npc.rs rotation precedent).
    if num797 > 0.0 {
        npc.sprite_dir = 1;
        npc.rotation = num798.atan2(num797);
    }
    if num797 < 0.0 {
        npc.sprite_dir = -1;
        npc.rotation = num798.atan2(num797) + 3.14;
    }
    fx
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation; the RNG witnesses mirror the
    //! cited `Next` calls on a cloned `UnifiedRandom` and compare the
    //! stream state (`PartialEq`) so a drifted draw order fails by
    //! construction.
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

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

    /// A stand-in tentacle body: Plantera (262) carries no aiStyle-53
    /// row, so the witnesses drive `ai_053_arm(264, ...)` on a spare
    /// kind with the C#'s 24×24 box + 1000 hp (`NPC.cs:12412-12417`)
    /// pinned via `size_override` — the kind itself lands at wiring.
    fn tentacle(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(crate::npc::NpcKind::Plantera, x, y, 1, 0.0);
        n.size_override = Some((24.0, 24.0));
        n.hp = 1000;
        n.max_hp = 1000;
        n
    }

    /// A boss/anchor view at (x, y), 46×46 (any even box — the odd-box
    /// truncation has its own witness), full hp.
    fn view(x: f32, y: f32) -> Ai53NpcView {
        Ai53NpcView {
            active: true,
            x,
            y,
            width: 46,
            height: 46,
            life: 30000,
            life_max: 30000,
        }
    }

    fn ctx<'a>(
        players: &'a [TargetCandidate],
        boss: Ai53NpcView,
        anchor: Ai53NpcView,
    ) -> Ai53Ctx<'a> {
        Ai53Ctx {
            players,
            plant_boss: 0,
            boss: Some(boss),
            anchor: Some(anchor),
            expert_mode: false,
            get_good_world: false,
            ticker: 0,
        }
    }

    /// `NPC.cs:32254-32259` — no Plantera: the 9999 strike + netUpdate
    /// + return. The rearm timer never runs (localAI[0] untouched) and
    /// NO rand is drawn on a classic world.
    #[test]
    fn s1_d_3_s53_the_orphan_tentacle_strikes_itself_without_drawing() {
        let w = air();
        let players = [cand(900.0, 900.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 5.0;
        let mut c = ctx(&players, view(800.0, 800.0), view(800.0, 800.0));
        c.plant_boss = -1;
        c.boss = None;
        let mut rng = UnifiedRandom::new(7);
        let untouched = rng.clone();
        let fx = ai_053_step(&mut n, &w, &c, &mut rng);
        assert_eq!(rng, untouched, "the classic orphan path draws nothing");
        assert_eq!(fx.orphan_strike, Some((9999, 0.0, 0)), ":32256 verbatim");
        assert_eq!(n.hp, 0, "the instant-kill strike (flag, :81237)");
        assert!(n.net_update, ":32257");
        assert_eq!(n.local_ai[0], 5.0, "the return precedes the timer");
        assert_eq!(fx.reflects_projectiles, None, "not a ftw world");
    }

    /// `NPC.cs:32243-32259` — on a getGood world the orphan's death tick
    /// STILL draws the reflect `Next(10)` (the draw precedes the return).
    /// Stream-clone replay: one `next_max(10)` and nothing else.
    #[test]
    fn s1_d_3_s53_the_ftw_orphan_draws_the_reflect_roll_before_dying() {
        let w = air();
        let players = [cand(900.0, 900.0)];
        let mut n = tentacle(1000.0, 1000.0);
        let mut c = ctx(&players, view(800.0, 800.0), view(800.0, 800.0));
        c.plant_boss = -1;
        c.boss = None;
        c.get_good_world = true;
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = rng.clone();
        let want_reflect = mirror.next_max(10) == 0;
        let fx = ai_053_step(&mut n, &w, &c, &mut rng);
        assert_eq!(rng, mirror, "exactly one Next(10), before the return");
        assert_eq!(fx.reflects_projectiles, Some(want_reflect));
        assert_eq!(fx.orphan_strike, Some((9999, 0.0, 0)));
        assert_eq!(n.hp, 0);
    }

    /// `NPC.cs:32265-32275` — the rearm triple, in order: the timer
    /// counts down every tick; at <= 0 the draws are Next(120, 480),
    /// Next(-100, 101), Next(-100, 101). Stream-clone replay pins both
    /// the values and the order.
    #[test]
    fn s1_d_3_s53_the_rearm_draws_the_cited_triple_in_order() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 1.0; // this tick decrements to 0 → rearm
        let boss = view(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(42);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(120, 480);
        let ax = mirror.next_range(-100, 101);
        let ay = mirror.next_range(-100, 101);
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert_eq!(rng, mirror, "the triple, in C# order, nothing else");
        assert_eq!(n.local_ai[0], timer as f32, ":32270");
        assert_eq!(n.ai[0], ax as f32, ":32271");
        assert_eq!(n.ai[1], ay as f32, ":32272");
        assert!(n.net_update, ":32273");
    }

    /// `NPC.cs:32267-32268` — a live timer only decrements; NO draw.
    #[test]
    fn s1_d_3_s53_the_live_timer_decrements_without_drawing() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 50.0;
        n.ai[0] = 100.0; // a fixed orbit offset, right of the anchor
        n.ai[1] = 0.0;
        let boss = view(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(3);
        let untouched = rng.clone();
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert_eq!(rng, untouched, "the steering path draws nothing");
        assert_eq!(n.local_ai[0], 49.0);
        assert_eq!(n.ai[0], 100.0, "no rearm — the offset stands");
    }

    /// `NPC.cs:32296-32300` — the anchor lost: `active = false` and
    /// return, but the rearm triple ALREADY drew this tick (the check
    /// sits after the timer block). Stream-clone replay: the triple is
    /// consumed even though the tentacle dies.
    #[test]
    fn s1_d_3_s53_the_dead_anchor_deactivates_after_the_rearm_draw() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 1.0;
        let boss = view(1000.0, 1000.0);
        let mut dead_anchor = view(1000.0, 1000.0);
        dead_anchor.active = false;
        let mut rng = UnifiedRandom::new(99);
        let mut mirror = rng.clone();
        let timer = mirror.next_range(120, 480);
        let ax = mirror.next_range(-100, 101);
        let ay = mirror.next_range(-100, 101);
        let fx = ai_053_step(&mut n, &w, &ctx(&players, boss, dead_anchor), &mut rng);
        assert_eq!(rng, mirror, "the triple draws before the death check");
        assert_eq!(n.local_ai[0], timer as f32);
        assert_eq!(n.ai[0], ax as f32);
        assert_eq!(n.ai[1], ay as f32);
        assert_eq!(n.hp, 0, ":32298 active = false");
        assert_eq!(fx.orphan_strike, None, "no strike — the boss lives");
        assert_eq!(n.vx, 0.0, "the steering never ran");
    }

    /// `NPC.cs:32301-32302` — the anchor centre uses INTEGER half
    /// extents: an odd 47×47 box centres at +23, not +23.5.
    #[test]
    fn s1_d_3_s53_the_anchor_center_truncates_the_integer_halves() {
        let odd = Ai53NpcView {
            width: 47,
            height: 47,
            ..view(100.0, 200.0)
        };
        assert_eq!(anchor_center(&odd), (123.0, 223.0));
        let even = view(100.0, 200.0); // 46×46
        assert_eq!(anchor_center(&even), (123.0, 223.0));
    }

    /// `NPC.cs:32311-32358` — the steering: position left of the orbit
    /// point accelerates +0.2; the sign-reversal damp fires when the
    /// velocity still points against the pull; the caps clamp at ±8.
    /// Anchor at (1000, 1000) (46×46 → centre 1023, 1023), offset
    /// (100, 0) → orbit point (1223, 1023), radius 200 → num797 = 200.
    #[test]
    fn s1_d_3_s53_the_steer_accelerates_damps_and_caps() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(1000.0, 1023.0); // left of 1223, on the Y line
        n.local_ai[0] = 50.0;
        n.ai[0] = 100.0;
        n.ai[1] = 0.0;
        n.vx = -0.3; // still against the pull after +0.2: (-0.3 + 0.2) * 0.9 = -0.09
        let boss = view(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(5);
        let untouched = rng.clone();
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert_eq!(rng, untouched);
        assert!(
            (n.vx - -0.09).abs() < 1e-6,
            "the reversal damp (:32314-32317), got {}",
            n.vx
        );
        assert_eq!(n.vy, 0.0, "y == the orbit line — neither Y arm fires");
        assert_eq!(n.sprite_dir, 1, "num797 > 0 (:32359-32363)");
        assert_eq!(n.rotation, 0.0f32.atan2(200.0), ":32362");
        // The cap: a velocity already past 8 clamps.
        n.vx = 9.0;
        n.x = 1000.0;
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert_eq!(n.vx, 8.0, ":32343-32346");
    }

    /// `NPC.cs:32319-32326` + `:32364-32368` — the left-facing mirror:
    /// offset (−100, 0) → num797 = −200; right of the orbit point the X
    /// accel is −0.2 with its own damp; spriteDirection −1 and the
    /// rotation folds by the LITERAL 3.14f (not PI).
    #[test]
    fn s1_d_3_s53_the_left_aim_folds_by_the_literal_314() {
        let w = air();
        let players = [cand(600.0, 1000.0)];
        let mut n = tentacle(1100.0, 1023.0); // right of the orbit point 823
        n.local_ai[0] = 50.0;
        n.ai[0] = -100.0;
        n.ai[1] = 0.0;
        n.vx = 0.3; // still against the pull after -0.2: (0.3 - 0.2) * 0.9 = 0.09
        let boss = view(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(6);
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert!((n.vx - 0.09).abs() < 1e-6, "got {}", n.vx);
        assert_eq!(n.sprite_dir, -1);
        assert_eq!(
            n.rotation,
            0.0f32.atan2(-200.0) + 3.14,
            "the literal 3.14f (:32367), not PI"
        );
        assert_ne!(n.rotation, 0.0f32.atan2(-200.0) + std::f32::consts::PI);
    }

    /// `NPC.cs:32308-32310` — at ai == (0, 0) the normalize divides by
    /// zero: num799 = +Inf, num797/num798 = 0 * Inf = NaN. Every steering
    /// compare and both rotation ifs fail (NaN), so the velocity and the
    /// rotation keep their old values — NO invented zero guard.
    #[test]
    fn s1_d_3_s53_the_zero_offset_nan_leaves_the_state_untouched() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 50.0;
        n.ai[0] = 0.0;
        n.ai[1] = 0.0;
        n.vx = 1.25;
        n.vy = -2.5;
        n.rotation = 0.75;
        n.sprite_dir = -1;
        let boss = view(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(8);
        let untouched = rng.clone();
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert_eq!(rng, untouched);
        assert_eq!(n.vx, 1.25, "NaN fails every compare (:32311-32358)");
        assert_eq!(n.vy, -2.5);
        assert_eq!(n.rotation, 0.75, "neither rotation if fires");
        assert_eq!(n.sprite_dir, -1);
    }

    /// `NPC.cs:32279-32286` — the boss hp bands widen the orbit radius:
    /// 200 base, +100 below 25%, +100 more below 10% (BOTH bands stack).
    /// The bands are DOUBLE compares on lifeMax * 0.25 / * 0.1. With the
    /// offset (100, 0) the normalized num797 equals the radius, so the
    /// orbit point's distance pins the band.
    #[test]
    fn s1_d_3_s53_the_boss_hp_bands_widen_the_orbit_radius() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        // Pin num797 through the steering geometry: place the tentacle
        // exactly ON the candidate orbit point's X, 1 px BELOW its Y
        // line (y = 24 vs the line 23) — vy == -0.2 (the pull is UP) and
        // vx == 0 only when the radius matches the band; a wrong radius
        // puts x off the point and the X accel fires.
        let probe = |life: i32, r: f32| -> (f32, f32) {
            let mut n = tentacle(23.0 + r, 24.0);
            n.local_ai[0] = 50.0;
            n.ai[0] = 100.0;
            n.ai[1] = 0.0;
            let mut boss = view(0.0, 0.0);
            boss.life = life;
            let mut rng = UnifiedRandom::new(1);
            ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
            (n.vx, n.vy)
        };
        // Full hp: r = 200 → on the X point, below the Y line.
        assert_eq!(probe(30000, 200.0), (0.0, -0.2));
        assert_eq!(probe(30000, 300.0).0, -0.2, "x right of the point");
        // 24% hp: below 0.25 only → r = 300.
        assert_eq!(probe(7200, 300.0), (0.0, -0.2));
        assert_eq!(probe(7200, 200.0).0, 0.2, "x left of the point");
        // 9% hp: both bands → r = 400.
        assert_eq!(probe(2700, 400.0), (0.0, -0.2));
        // The band edge is EXCLUSIVE (<): exactly 25% keeps r = 200.
        assert_eq!(probe(7500, 200.0), (0.0, -0.2));
        // Exactly 10% keeps the single +100 (r = 300): 30000.0 * 0.1
        // rounds to exactly 3000.0 in f64, and 3000 < 3000 is false.
        assert_eq!(probe(3000, 300.0), (0.0, -0.2));
    }

    /// `NPC.cs:32287-32295` — expert: the radius adds
    /// `(1 - life/lifeMax) * 300` on the TENTACLE's own hp (f32), the
    /// accel becomes 0.5; ftw adds +4 more (accel 4.5). A half-hp
    /// tentacle at full-hp boss: r = 200 + 150 = 350, accel 0.5.
    #[test]
    fn s1_d_3_s53_the_expert_radius_rides_the_tentacles_own_hp() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(23.0 + 350.0, 24.0); // on the X point, 1 below Y
        n.hp = 500;
        n.max_hp = 1000;
        n.local_ai[0] = 50.0;
        n.ai[0] = 100.0;
        n.ai[1] = 0.0;
        let boss = view(0.0, 0.0); // full-hp boss: no band widening
        let mut c = ctx(&players, boss, boss);
        c.expert_mode = true;
        let mut rng = UnifiedRandom::new(2);
        ai_053_step(&mut n, &w, &c, &mut rng);
        assert_eq!(n.vx, 0.0, "the radius is 350 — on the orbit point");
        assert_eq!(n.vy, -0.5, "the expert accel, pulling up (:32290)");
        // ftw stacks +4 on the accel.
        let mut n2 = tentacle(23.0 + 350.0, 24.0);
        n2.hp = 500;
        n2.max_hp = 1000;
        n2.local_ai[0] = 50.0;
        n2.ai[0] = 100.0;
        c.get_good_world = true;
        let mut mirror = rng.clone();
        let _want = mirror.next_max(10); // the ftw reflect draw
        ai_053_step(&mut n2, &w, &c, &mut rng);
        assert_eq!(rng, mirror, "ftw adds exactly the reflect draw");
        assert_eq!(n2.vy, -4.5, "0.2 + 0.3 + 4, pulling up (:32294)");
    }

    /// `NPC.cs:32261-32264` + `:32279-32289` — the expert hook anchor:
    /// `ai[3] > 0` steers around `Main.npc[ai[3] - 1]`, but the hp bands
    /// still read the BOSS. A low-hp boss (r = 400) with a full-hp hook
    /// at a different position: the orbit point rides the HOOK centre.
    #[test]
    fn s1_d_3_s53_the_hook_anchor_moves_the_orbit_but_not_the_bands() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(523.0 + 400.0, 524.0); // hook centre (523,523)
        n.local_ai[0] = 50.0;
        n.ai[0] = 100.0;
        n.ai[1] = 0.0;
        n.ai[3] = 7.0; // hook slot 6 (:32004's +1 convention)
        let mut boss = view(0.0, 0.0);
        boss.life = 2700; // 9% — both bands, r = 400
        let hook = view(500.0, 500.0); // full hp — irrelevant to the bands
        let mut rng = UnifiedRandom::new(4);
        ai_053_step(&mut n, &w, &ctx(&players, boss, hook), &mut rng);
        assert_eq!(n.vx, 0.0, "on the HOOK's orbit point, radius 400");
        assert_eq!(n.vy, -0.2);
    }

    /// `NPC.cs:32276` — the TargetClosest call writes the tracking
    /// fields (the arm itself never reads target). The player to the
    /// right faces +1; style 53 is not the style-3 fighter arm, so
    /// directionY keeps its value.
    #[test]
    fn s1_d_3_s53_the_target_closest_runs_with_style_53() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 50.0;
        n.ai[0] = 100.0;
        n.facing = -1;
        n.direction_y = 1;
        let boss = view(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(10);
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert_eq!(n.target, 0, "the scan picked the session player");
        assert_eq!(n.facing, 1, "the player is to the right");
        assert_eq!(n.direction_y, 1, "no style-3 bottom-edge write");
    }

    /// The ftw reflect re-roll on a LIVE tick: exactly one Next(10),
    /// before any rearm draw; both outcomes write the flag (a per-tick
    /// re-roll, not a latch — `:32245-32252`).
    #[test]
    fn s1_d_3_s53_the_ftw_reflect_rerolls_every_tick_before_the_rearm() {
        let w = air();
        let players = [cand(1600.0, 1000.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 1.0; // the rearm fires too — 4 draws total
        let boss = view(1000.0, 1000.0);
        let mut c = ctx(&players, boss, boss);
        c.get_good_world = true;
        let mut rng = UnifiedRandom::new(12345);
        let mut mirror = rng.clone();
        let reflect = mirror.next_max(10) == 0;
        let timer = mirror.next_range(120, 480);
        let ax = mirror.next_range(-100, 101);
        let ay = mirror.next_range(-100, 101);
        let fx = ai_053_step(&mut n, &w, &c, &mut rng);
        assert_eq!(rng, mirror, "reflect first, then the rearm triple");
        assert_eq!(fx.reflects_projectiles, Some(reflect));
        assert_eq!(n.local_ai[0], timer as f32);
        assert_eq!(n.ai[0], ax as f32);
        assert_eq!(n.ai[1], ay as f32);
    }

    /// `NPC.cs:32327-32342` — the Y steering is a SEPARATE if/else-if
    /// pair: both axes accelerate on the same tick, each with its own
    /// reversal damp.
    #[test]
    fn s1_d_3_s53_both_axes_steer_on_the_same_tick() {
        let w = air();
        let players = [cand(1600.0, 1600.0)];
        let mut n = tentacle(1000.0, 1000.0);
        n.local_ai[0] = 50.0;
        n.ai[0] = 100.0;
        n.ai[1] = 100.0; // orbit point (1023+~141.4, 1023+~141.4)
        n.vx = -0.3;
        n.vy = -0.3;
        let boss = view(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(13);
        ai_053_step(&mut n, &w, &ctx(&players, boss, boss), &mut rng);
        assert!(
            (n.vx - -0.09).abs() < 1e-6 && (n.vy - -0.09).abs() < 1e-6,
            "both reversal damps fired, got ({}, {})",
            n.vx,
            n.vy
        );
        assert_eq!(n.sprite_dir, 1);
        assert!(
            (n.rotation - std::f32::consts::FRAC_PI_4).abs() < 1e-5,
            "the 45° aim (:32362), got {}",
            n.rotation
        );
    }
}
