//! The town NPC's shimmer emergence — the pure decision core (W4-f30,
//! progression 43 NEW_DIGS).
//!
//! `NPC.AI_007_TownEntities`' state 25 (`NPC.cs:53187-53277`): the shimmer-
//! dunk transform. The full machine (the transparency ramp, the hue light /
//! 309 dust / particle visuals, the four-scan `TeleportToLandingSpot`) is the
//! liquid-shimmer lane's (W4-f47, `npc_ai_town.rs::town_shimmer_state25` on
//! its branch); THIS module ports the RNG-free decision core as a standalone
//! pure fn so the progression-43 wire has a conflict-free home — the variant
//! flip + the completion notify (`NPC.cs:53264`/`:53271-53273`).
//!
//! The split (docs/ANALYSIS.md "The town shimmer transform (W4-f30)"):
//! f47 owns the visual/teleport machine and its inline state-25 arm; f30
//! owns this decision core. The two meet at the merge — the wiring calls
//! `town_shimmer_step` for the state writes + the 43 notify, and f47's
//! `shimmer_fx` seam for the visuals. Until the shimmer TRIGGER lands (the
//! buff-353 transparency ramp crossing 0.9 → `GetShimmered`'s town-set arm
//! sets state 25, `NPC.cs:91578-91591` — f47's `drain_shimmer_transforms`),
//! the state never enters in play and the core is exercised by the witnesses.
//!
//! Deferred halves (cited): the `TargetClosest`/`GetTargetData` read
//! (`:53249-53250` — the `Math.Sign` result at `:53257` is DISCARDED, no
//! direction write), the `NetMessage.SendData(56)` variation relay (`:53265`),
//! the `Teleport(position, 12)` effect (`:53266` — the position is UNCHANGED,
//! the C# teleports to where it already stands; the teleportTime latch is the
//! visual seam), and the `ShimmerTownNPC` particle broadcast (`:53267-53270`).

use crate::npc::Npc;

/// One state-25 tick's outcome: whether the completion fired this tick and
/// whether the flipped variation is the shimmer variant (1) — the caller
/// fires progression 43 (`NPC.cs:53271-53273`) on `completed && variation 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TownShimmerOutcome {
    /// The completion arm ran (`ai[1] >= 75 && shimmerTransparency <= 0`,
    /// `NPC.cs:53251`): the ai/localAI zeroed, the (0,-4) kick, the variant
    /// flipped.
    pub completed: bool,
    /// `townNpcVariationIndex` after the flip (`:53264`) — 1 the shimmer
    /// variant, 0 the default. The 43 notify gates on `== 1`.
    pub variation_index: i32,
}

/// `NPC.AI_007_TownEntities`' state-25 decision core (`NPC.cs:53187-53277`),
/// RNG-free: the `dontTakeDamage` latch, the `ai[1]==0` vx zero + wet-flag
/// clear, the `ai[2]` teleport hold, the `ai[1]` climb, the ≥30 transparency
/// decay (1/60 per tick, pinned while wet) + the (0, -4·t) rise, and the
/// ≥75 && t≤0 completion (the ai/localAI zero, the (0,-4) kick, the variant
/// flip). The visuals (the hue light / dust / particles) and the landing-spot
/// teleport are f47's seam — this core writes only the NPC state.
///
/// `wet` is the FRESH `Collision.WetCollision` probe (`:53218`), not the
/// cached swimmer flag — the caller supplies it (the port's
/// `npc_ai_slime::wet_collision`). `net_mode` gates the completion (`:53251`'s
/// `Main.netMode != 1`); the client (1) never completes.
///
/// Returns the outcome; the caller fires 43 on `completed && variation_index
/// == 1` and runs the visual/teleport seam. The C# RETURNS from the whole
/// AI_007 arm after state 25 (`:53276`) — the caller must not fall through.
pub fn town_shimmer_step(npc: &mut Npc, wet: bool, net_mode: i32) -> TownShimmerOutcome {
    let mut out = TownShimmerOutcome::default();
    // `:53189` — dontTakeDamage = true every state-25 tick.
    npc.dont_take_damage = true;
    // `:53190-53193` — ai[1]==0 zeroes vx.
    if npc.ai[1] == 0.0 {
        npc.vx = 0.0;
    }
    // `:53194-53197` — the wet flags clear. The port's NPC carries the
    // swimmer `wet` cache (cleared here); `shimmerWet`/`honeyWet`/`lavaWet`
    // are f47's liquid-shimmer fields (its branch adds them) — cited, the
    // wiring clears them once they land.
    npc.wet = false;
    // `:53198-53201` — the client-only early return (netMode 1, ai[1]==0)
    // is the wiring's gate; the core keeps climbing (the host path).
    // `:53202-53205` — the landing-spot teleport (ai[1]==0 && ai[2]<1) is
    // f47's `shimmer_teleport_to_landing_spot` seam — cited here.
    // `:53206-53213` — the ai[2] teleport hold: the countdown decrements,
    // at zero ai[1] arms 1, and the arm RETURNS (no climb, no visuals).
    if npc.ai[2] > 0.0 {
        npc.ai[2] -= 1.0;
        if npc.ai[2] <= 0.0 {
            npc.ai[1] = 1.0; // `:53211`
        }
        return out; // `:53213`
    }
    // `:53215` — the climb.
    npc.ai[1] += 1.0;
    // `:53216-53227` — at ≥30 the transparency decays 1/60 per tick UNLESS
    // wet (the fresh probe pins the climb at 30); velocity (0, -4·t).
    if npc.ai[1] >= 30.0 {
        if !wet {
            npc.shimmer_transparency = (npc.shimmer_transparency - 1.0 / 60.0).clamp(0.0, 1.0);
        } else {
            npc.ai[1] = 30.0; // `:53224`
        }
        npc.vx = 0.0;
        npc.vy = -4.0 * npc.shimmer_transparency; // `:53226`
    }
    // `:53228-53248` — the hue light + the 309 dust + the 1/15 particle
    // triple: f47's `shimmer_fx` seam (the draws spend rng) — cited here.
    // `:53249-53250` — TargetClosest + GetTargetData: the Math.Sign result
    // at `:53257` is DISCARDED (no direction write) — cited, no state effect.
    // `:53251-53275` — the completion: ai[1] ≥ 75 && t ≤ 0 (netMode != 1).
    if npc.ai[1] >= 75.0 && npc.shimmer_transparency <= 0.0 && net_mode != 1 {
        // `:53253-53256` — the ai zeroes.
        npc.ai = [0.0; 4];
        // `:53258` — the (0,-4) kick.
        npc.vx = 0.0;
        npc.vy = -4.0;
        // `:53259-53262` — the localAI zeroes.
        npc.local_ai = [0.0; 4];
        // `:53263` — netUpdate.
        npc.net_update = true;
        // `:53264` — the variant flip: 0 → 1, 1 → 0.
        npc.variation_index = if npc.variation_index != 1 { 1 } else { 0 };
        out.completed = true;
        out.variation_index = npc.variation_index;
        // `:53265` — SendData(56): the variation relay (the wiring's net seam).
        // `:53266` — Teleport(position, 12): the position UNCHANGED (the C#
        // teleports to where it already stands); the effect is f47's seam.
        // `:53267-53270` — the ShimmerTownNPC particle: f47's seam.
        // `:53271-53273` — progression 43 when the flip lands on 1: the
        // caller fires it off `out` (the core cannot borrow the achievements).
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A Dryad mid-emergence: state 25, the transparency at the 0.89 the
    /// `GetShimmered` town-set arm sets (`NPC.cs:91585`).
    fn emerging() -> Npc {
        let mut n = Npc::new(NpcKind::Dryad, 100.0, 100.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 25.0;
        n.shimmer_transparency = 0.89;
        n
    }

    /// `NPC.cs:53206-53213` — the ai[2] hold: the countdown decrements, at
    /// zero ai[1] arms 1, and the arm returns before the climb.
    #[test]
    fn f30_the_ai2_hold_returns_before_the_climb() {
        let mut n = emerging();
        n.ai[1] = 0.0;
        n.ai[2] = 2.0;
        let out = town_shimmer_step(&mut n, false, 0);
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.ai[1], 0.0, "the hold returns before the climb");
        assert!(!out.completed);
        assert!(n.dont_take_damage, ":53189");
        // The hold's last tick arms ai[1] = 1.
        let out = town_shimmer_step(&mut n, false, 0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[1], 1.0);
        assert!(!out.completed);
    }

    /// `NPC.cs:53215-53227` — the climb + the ≥30 transparency decay (1/60
    /// per tick) + the (0, -4·t) rise; the wet probe pins the climb at 30.
    #[test]
    fn f30_the_climb_decays_the_transparency_and_rises() {
        let mut n = emerging();
        n.ai[1] = 29.0;
        n.ai[2] = 0.0;
        let out = town_shimmer_step(&mut n, false, 0);
        assert_eq!(n.ai[1], 30.0);
        assert!((n.shimmer_transparency - (0.89 - 1.0 / 60.0)).abs() < 1e-6);
        assert!((n.vy - (-4.0 * n.shimmer_transparency)).abs() < 1e-6);
        assert_eq!(n.vx, 0.0);
        assert!(!out.completed);
        // The wet probe pins the climb at 30 (no decay).
        let mut n2 = emerging();
        n2.ai[1] = 40.0;
        n2.shimmer_transparency = 0.5;
        let out = town_shimmer_step(&mut n2, true, 0);
        assert_eq!(n2.ai[1], 30.0, "the wet probe pins the climb");
        assert_eq!(n2.shimmer_transparency, 0.5, "no decay while wet");
        assert!(!out.completed);
    }

    /// `NPC.cs:53251-53275` — the completion: ai[1] ≥ 75 && t ≤ 0 zeroes the
    /// ai/localAI, kicks (0,-4), flips the variation 0 → 1 and credits 43.
    #[test]
    fn f30_the_completion_flips_the_variant_and_credits_43() {
        let mut n = emerging();
        n.ai[1] = 74.0;
        n.shimmer_transparency = 0.0;
        n.variation_index = 0;
        let out = town_shimmer_step(&mut n, false, 0);
        // The climb reaches 75, the completion fires.
        assert!(out.completed);
        assert_eq!(out.variation_index, 1, "0 → 1, the shimmer variant");
        assert_eq!(n.ai, [0.0; 4], "the completion zeroes the ai");
        assert_eq!(n.local_ai, [0.0; 4]);
        assert_eq!(n.vx, 0.0);
        assert_eq!(n.vy, -4.0, "the (0,-4) kick");
        assert!(n.net_update);
        assert_eq!(n.variation_index, 1);
        // The 43 gate: completed && variation 1.
        assert!(out.completed && out.variation_index == 1, "43 fires");
        // A second completion (1 → 0) does NOT credit 43.
        let mut n2 = emerging();
        n2.ai[1] = 74.0;
        n2.shimmer_transparency = 0.0;
        n2.variation_index = 1;
        let out2 = town_shimmer_step(&mut n2, false, 0);
        assert!(out2.completed);
        assert_eq!(out2.variation_index, 0, "1 → 0, the default");
        assert!(!(out2.completed && out2.variation_index == 1), "43 refuses");
    }

    /// `NPC.cs:53251` — the client (netMode 1) never completes; the
    /// transparency stays put and the climb keeps going.
    #[test]
    fn f30_the_client_never_completes() {
        let mut n = emerging();
        n.ai[1] = 80.0;
        n.shimmer_transparency = 0.0;
        n.variation_index = 0;
        let out = town_shimmer_step(&mut n, false, 1);
        assert!(!out.completed, "netMode 1 refuses the completion");
        assert_eq!(n.variation_index, 0, "no flip on the client");
        assert_eq!(n.ai[0], 25.0, "the state holds");
    }

    /// `NPC.cs:53251` — the completion needs BOTH ai[1] ≥ 75 AND t ≤ 0; a
    /// high climb with transparency left does not complete.
    #[test]
    fn f30_the_completion_needs_both_gates() {
        let mut n = emerging();
        n.ai[1] = 80.0;
        n.shimmer_transparency = 0.1; // still fading
        let out = town_shimmer_step(&mut n, false, 0);
        assert!(!out.completed, "t > 0 refuses");
        assert_eq!(n.ai[0], 25.0);
        // ai[1] < 75 with t == 0 also refuses.
        let mut n2 = emerging();
        n2.ai[1] = 50.0;
        n2.shimmer_transparency = 0.0;
        let out2 = town_shimmer_step(&mut n2, false, 0);
        assert!(!out2.completed, "ai[1] < 75 refuses");
    }
}
