//! W4-f2 closeout (research/94 §10) — the NPC.cs PlaySound tail census's
//! UNCLAIMED sites' pure sound decisions. e9b's 27-site
//! `npc_sounds_tail.rs` re-census unioned against the d-band machines:
//! 25 of its 27 sites are already owned by a pushed machine module (the
//! d24 style-19 Antlion, the d32 style-86 teleports, the d33 style-103/
//! 112/113/117/121/122/125/126/127 families — each carries the site in
//! its sounds out-struct, the style75 seam), so e9b's duplicate decision
//! helpers DROP and the machines keep the sites (the SandShark keep-one
//! d33 precedent, research/94 §7). The bank rows/cues those machines'
//! wirings will drain ARE unioned into f2 (the `NpcSoundCue` tail arms +
//! `audio.rs`'s `NPC_Killed_17/63`, `Item_150/167/170/172`,
//! `Zombie_115-117` + the Pal/StatueMimic trackables + the case-65 arm).
//!
//! TWO sites remain unclaimed by ANY pushed branch (the full-branch
//! `git grep` sweep, research/94 §10): the Medusa stone-gaze cry
//! (`NPC.cs:56403` — d22's style-3 core claims only the 22 plain
//! walkers, the 480 gaze block is unported; d40's Medusa is data-only)
//! and the Big Mimic's projectile-reflect tink (`NPC.cs:66429` — d32's
//! style-87 emits the `reflect_request` scan out-field, but no branch
//! ports the `ReflectProjectile` body's sound). They land here as pure
//! staged helpers (the `water_exit_jump`/`vile_spit_queue_cry`
//! precedent): the gate maths + the `NpcSoundCue` queue pinned before
//! the kinds' machines land, so each wiring is a one-call consumer.
//!
//! The cues drain through `world_sim/npcs.rs::play_npc_sound_request`
//! on the `UpdateNPCs` stream. RNG: both helpers are draw-free — the
//! case-4 / case-2 pitch draws ride the drain's play
//! (attenuation-gated first).

#![allow(dead_code)] // UNWIRED - the 480 gaze machine / the reflect seam are the wiring steps (the style64/67 precedent).

use crate::npc::{Npc, NpcSoundCue, NpcSoundRequest};

/// Queue one cue at the C# call site's world-pixel position. The push
/// draws nothing (the case draws ride the drain's play).
fn queue(npc: &mut Npc, cue: NpcSoundCue, x: f32, y: f32) {
    npc.wants_sound.push(NpcSoundRequest { cue, x, y });
}

/// `NPC.cs:56400-56404` (AI_003_Fighters' Medusa 480 block): the
/// stone-gaze cry. The gaze counter's `num25 == 1` tick plays
/// `PlaySound(4, (int)position.X, (int)position.Y, 17)` (the int
/// overload, case 4 — NO NextFloat). `num25` is `ai[2] + num22 + num24`
/// (the gaze windup, `:56400`); the cue queues only on the 1 tick.
/// Returns `true` on the cry tick. The `NPC_Killed_17` bank row + the
/// `play_npc_death_int` style-17 arm are landed (research/94 §10).
pub(crate) fn medusa_gaze_cue(npc: &mut Npc, num25: f32) -> bool {
    if num25 == 1.0 {
        queue(npc, NpcSoundCue::NpcDeath(17), npc.x, npc.y);
        return true;
    }
    false
}

/// `NPC.cs:66427-66429` (`NPC.ReflectProjectile`, the Big Mimic 473-476's
/// expert reflect): the reflect tink. `SoundID.Item150` plays at the
/// PROJECTILE's position (`proj.position`, not the NPC's). The cue
/// queues the case-2 Item150 chime (the `Item_150` bank row is landed);
/// the reflect's velocity/damage rewrite is the projectile layer's
/// seam.
pub(crate) fn reflect_projectile_cue(npc: &mut Npc, proj_x: f32, proj_y: f32) {
    queue(npc, NpcSoundCue::ItemCue(150), proj_x, proj_y);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    fn npc() -> Npc {
        // The ChaosBall kind is a stand-in carrier (the helpers key on
        // the passed counter/position, not the kind); park it at a known
        // position.
        Npc::new(NpcKind::ChaosBall, 100.0, 200.0, 1, 0.0)
    }

    /// `NPC.cs:56403` + `:66429` — Medusa's gaze cry on num25==1 and the
    /// Big Mimic reflect tink at the projectile position; both pushes
    /// are draw-free.
    #[test]
    fn w4_f2_the_medusa_gaze_and_the_reflect_tink() {
        let mut n = npc();
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert!(medusa_gaze_cue(&mut n, 1.0));
        assert_eq!(n.wants_sound.len(), 1);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::NpcDeath(17));
        assert_eq!((n.wants_sound[0].x, n.wants_sound[0].y), (100.0, 200.0));
        n.wants_sound.clear();
        assert!(!medusa_gaze_cue(&mut n, 2.0));
        assert!(!medusa_gaze_cue(&mut n, 0.0));
        assert!(n.wants_sound.is_empty());
        reflect_projectile_cue(&mut n, 500.0, 600.0);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(150));
        assert_eq!((n.wants_sound[0].x, n.wants_sound[0].y), (500.0, 600.0));
        assert_eq!(rng.peek(), peek, "the queue pushes draw nothing");
    }
}
