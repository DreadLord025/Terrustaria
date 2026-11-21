//! W4-e9 tail — the remaining unclaimed `NPC.cs` `SoundEngine.PlaySound`
//! sites' pure sound decisions. The re-census (232 total sites, 31
//! unclaimed by citation-range, minus `:40581` already in the S1-E-21g
//! roar bank and the three sphere HitEffect sites `:89528`/`:89540`/
//! `:89692` claimed by `task/W4-c13-npc-hit-sounds`'s
//! `queue_hit_effect_sound`) leaves 27 sites, every one on an UNPORTED
//! kind. They land here as pure decision helpers — the
//! `water_collision_splash`/`vile_spit_queue_cry` precedent: the gate
//! maths, the state writes and the `NpcSoundCue` queue are pinned before
//! the kinds land, so each kind's wiring is a one-call consumer.
//!
//! The cues drain through `world_sim/npcs.rs::play_npc_sound_request`
//! on the `UpdateNPCs` stream (the case-2/3/4/19/27/42/65 paths). RNG:
//! every helper here is draw-free — the case draws ride the drain's
//! play (attenuation-gated first), and the only gameplay roll any site
//! owns is the Queen Slime moan's `Next(360)` gate and the FairyCritter
//! far-approach's `Next(30)` gate, both passed in as a `roll` closure so
//! the caller's stream order stays explicit.
//!
//! Sites (the C# line → the helper):
//! * `:24449` Antlion spit Item5 → [`antlion_spit_cue`]
//! * `:37268`/`:37321` apparition/squidhead teleport Item8 →
//!   [`apparition_teleport_cue`]
//! * `:40632` SandShark style-542 cry → [`sand_shark_cry`]
//! * `:42983`/`:42987`/`:42997`/`:43001` Pal pain/joy →
//!   [`pal_idle_cry`] / [`pal_joy_cry`]
//! * `:43562`/`:43569`/`:43580`/`:43608` StatueMimic →
//!   [`statue_mimic_scare`] / [`statue_mimic_jump`] / [`statue_mimic_laugh`]
//! * `:43726`/`:47951`/`:48062` balloon pops NPCDeath63 →
//!   [`balloon_pop_cue`]
//! * `:44848` PirateGhost spawn NPCHit6 → [`pirate_ghost_spawn_cue`]
//! * `:45558`/`:45759` QueenSlime landing/idle →
//!   [`queen_slime_landing_cue`] / [`queen_slime_idle_moan`]
//! * `:47263`/`:47268`/`:47293` BloodNautilus →
//!   [`blood_nautilus_volley_cue`] / [`blood_nautilus_scream_cue`]
//! * `:48327`/`:48473`/`:48545` FairyCritter Pixie →
//!   [`fairy_circle_hum`] / [`fairy_far_hum`]
//! * `:48682` FairyEffects Item4 → [`fairy_effects_cue`]
//! * `:56403` Medusa stone-gaze NPCDeath17 → [`medusa_gaze_cue`]
//! * `:66429` BigMimic reflect Item150 → [`reflect_projectile_cue`]

use crate::npc::{Npc, NpcSoundCue, NpcSoundRequest};

/// Queue one cue at the C# call site's world-pixel position. The push
/// draws nothing (the case draws ride the drain's play).
fn queue(npc: &mut Npc, cue: NpcSoundCue, x: f32, y: f32) {
    npc.wants_sound.push(NpcSoundRequest { cue, x, y });
}

/// `NPC.cs:24445-24451` (aiStyle 19, the Antlion 69): while `ai[0] > 0`
/// the countdown plays `SoundID.Item5` at `base.position` on the EXACT
/// `ai[0] == 200` tick (the host path — `Main.netMode != 2`), then
/// decrements. Returns the post-decrement `ai[0]`; the cue queues only
/// on the 200 tick. The spit's `ai[0] = 200` re-arm (`:24461`/`:24476`)
/// is the machine's shot gate, not this helper's.
pub(crate) fn antlion_spit_cue(npc: &mut Npc, ai0: f32) -> f32 {
    if ai0 > 0.0 {
        if ai0 == 200.0 {
            queue(npc, NpcSoundCue::ItemCue(5), npc.x, npc.y);
        }
        return ai0 - 1.0;
    }
    ai0
}

/// `NPC.cs:37261-37331` (aiStyle 86): the ShadowFlameApparition (472)
/// and the AncientCultistSquidhead (521) each play `SoundID.Item8` at
/// `base.Center` on their `localAI[0] == 0` teleport-in tick, then the
/// counter climbs. `ty` selects the kind's window (472: `< 120`, 521:
/// `< 120`); the cue queues only on the 0 tick inside the window.
/// Returns the post-increment `localAI[0]`. The TargetClosest + the
/// ±2 velocity kick + the dust are the machine's seams.
pub(crate) fn apparition_teleport_cue(npc: &mut Npc, ty: i32, local0: f32) -> f32 {
    let window = match ty {
        472 | 521 => 120.0,
        _ => return local0,
    };
    if local0 < window {
        if local0 == 0.0 {
            let (cx, cy) = npc.center();
            queue(npc, NpcSoundCue::ItemCue(8), cx, cy);
        }
        return local0 + 1.0;
    }
    local0
}

/// `NPC.cs:40559-40637` (aiStyle 103, the Sand Shark 542): the style-542
/// cry's decision. `in_sand` is `flag120` (the centre tile is a
/// sand/sandstone/hardened-sand conversion OR `base.wet`, `:40551`);
/// `chase` is `flag121` (the player falling-or-still, alive, > 150 px
/// from the shark's centre, `:40555`); `ahead_sand` is `flag123` (the
/// velocity-projected tile is sand, or liquid when wet, `:40617-40622`);
/// `dist` is `base.Distance(targetRect.Center)`; `remix` picks the
/// 400/700 range (`:40623-40627`). The localAI[0] latch: `-1 && !in_sand`
/// → 20 (`:40559-40562`), `> 0` → decrement (`:40563-40566`). Inside the
/// sand + chase + `!ahead_sand` + sign-match + range + `ai[2]` gate
/// (`:40628`), the `localAI[0] == 0` tick plays `PlaySound(14, Center,
/// 542)` (`:40632`) and latches `-1`; `ai[2]` resets to `-30` regardless.
/// Returns `(local0, ai2, cry)` — the cry queues the case-14 style-542
/// moan (the `play_fighter_moan` slot-7 path, NO draws). The velocity
/// aim (`:40636`) is the machine's seam.
pub(crate) fn sand_shark_cry(
    npc: &mut Npc,
    in_sand: bool,
    chase: bool,
    ahead_sand: bool,
    dist: f32,
    remix: bool,
    local0: f32,
    ai2: f32,
) -> (f32, f32, bool) {
    let mut local0 = local0;
    let mut ai2 = ai2;
    if local0 == -1.0 && !in_sand {
        local0 = 20.0;
    }
    if local0 > 0.0 {
        local0 -= 1.0;
    }
    let mut cry = false;
    if in_sand && chase {
        let range = if remix { 700.0 } else { 400.0 };
        // `Math.Sign(base.velocity.X) == base.direction` (`:40628`) —
        // Math.Sign returns 0 at velocity.X == 0, so a zero velocity
        // matches only a zero direction (the C#'s exact comparison).
        let sign_match = (npc.vx.signum() as i32) == npc.facing;
        if !ahead_sand && sign_match && dist < range && (ai2 >= 30.0 || ai2 < 0.0) {
            if local0 == 0.0 {
                let (cx, cy) = npc.center();
                queue(npc, NpcSoundCue::SandSharkCry, cx, cy);
                local0 = -1.0;
                cry = true;
            }
            ai2 = -30.0;
        }
    }
    (local0, ai2, cry)
}

/// `NPC.cs:42978-42989` (AI_127_Pal): the idle pain cry. While
/// `ai[0] == 0 && soundDelay == 0`, the delay re-arms to
/// `Next(600, 1800)` and the kind plays its pain trackable at
/// `base.Center` (695 → PalCattivaPain, 696 → PalFoxparksPain). Returns
/// the new `soundDelay` (`None` = the gate failed, the delay unchanged);
/// the cue queues on a hit. The `Next(600, 1800)` roll rides the caller's
/// stream via `roll`.
pub(crate) fn pal_idle_cry(
    npc: &mut Npc,
    ty: i32,
    ai0: f32,
    sound_delay: i32,
    roll: &mut dyn FnMut(i32, i32) -> i32,
) -> Option<i32> {
    if ai0 != 0.0 || sound_delay != 0 {
        return None;
    }
    let delay = roll(600, 1800);
    let cue = match ty {
        695 => "pal_cattiva_pain",
        696 => "pal_foxparks_pain",
        _ => return Some(delay),
    };
    let (cx, cy) = npc.center();
    queue(npc, NpcSoundCue::Trackable(cue), cx, cy);
    Some(delay)
}

/// `NPC.cs:42990-43003` (AI_127_Pal): the joy cry. On the `ai[0] == 2`
/// reward state's first tick (`soundDelay != -2`), the delay latches `-2`
/// and the kind plays its joy trackable at `base.Center` (695 →
/// PalCattivaJoy, 696 → PalFoxparksJoy). Returns `true` when the latch
/// fired (the caller writes `soundDelay = -2`); the cue queues once.
pub(crate) fn pal_joy_cry(npc: &mut Npc, ty: i32, ai0: f32, sound_delay: i32) -> bool {
    if ai0 != 2.0 || sound_delay == -2 {
        return false;
    }
    let cue = match ty {
        695 => "pal_cattiva_joy",
        696 => "pal_foxparks_joy",
        _ => return true,
    };
    let (cx, cy) = npc.center();
    queue(npc, NpcSoundCue::Trackable(cue), cx, cy);
    true
}

/// `NPC.cs:43559-43563` (AI_126_StatueMimic): the scare cry on the
/// `localAI[2]` 0→1 latch (the mimic's reveal). Returns the post-latch
/// `localAI[2]`; the StatueMimicScare trackable queues at `base.Center`
/// only on the 0 tick.
pub(crate) fn statue_mimic_scare(npc: &mut Npc, local2: f32) -> f32 {
    if local2 == 0.0 {
        let (cx, cy) = npc.center();
        queue(npc, NpcSoundCue::Trackable("statuemimic_scare"), cx, cy);
        return 1.0;
    }
    local2
}

/// `NPC.cs:43564-43590` (AI_126_StatueMimic): the jump cry's two arms.
/// The falling arm (`velocity.Y > 0 && Top.Y > player.Bottom.Y &&
/// SolidCollision`) plays StatueMimicJump; the grounded arm
/// (`velocity.Y == 0`) plays it on the EXACT `ai[2] == 20` tick, then
/// decrements `ai[2]` (wrapping to 20 at `<= 0`). `falling_solid` is the
/// falling arm's solid probe; `grounded` selects the grounded arm.
/// Returns the post-write `ai[2]`; the cue queues on either arm's hit.
pub(crate) fn statue_mimic_jump(
    npc: &mut Npc,
    ai2: f32,
    falling_solid: bool,
    grounded: bool,
) -> f32 {
    let (cx, cy) = npc.center();
    if falling_solid {
        queue(npc, NpcSoundCue::Trackable("statuemimic_jump"), cx, cy);
        return ai2;
    }
    if grounded {
        if ai2 == 20.0 {
            queue(npc, NpcSoundCue::Trackable("statuemimic_jump"), cx, cy);
        }
        let mut ai2 = ai2 - 1.0;
        if ai2 <= 0.0 {
            ai2 = 20.0;
        }
        return ai2;
    }
    ai2
}

/// `NPC.cs:43604-43616` (AI_126_StatueMimic): the laugh cry. On the
/// target's death, the `ai[3] == 0` tick plays StatueMimicLaugh at
/// `base.Center` and latches `ai[3] = 300`; while `ai[3] > 200` it
/// decrements. Returns the post-write `ai[3]`; the cue queues only on
/// the 0 tick. (`player_dead` gates the whole arm.)
pub(crate) fn statue_mimic_laugh(npc: &mut Npc, ai3: f32, player_dead: bool) -> f32 {
    if !player_dead {
        return ai3;
    }
    let mut ai3 = ai3;
    if ai3 == 0.0 {
        let (cx, cy) = npc.center();
        queue(npc, NpcSoundCue::Trackable("statuemimic_laugh_0"), cx, cy);
        ai3 = 300.0;
    }
    if ai3 > 200.0 {
        ai3 -= 1.0;
    }
    ai3
}

/// `NPC.cs:43715-43727` / `:47939-47952` / `:48051-48063` (the
/// ClumsySlimeBalloon 686 + the WindyBalloon 594): the pop. On the
/// wet/collide kill gate the balloon plays `this.DeathSound`
/// (NPCDeath63, `SetDefaults` `:17636`/`:16716`) at `base.position` and
/// dies. `pop` is the gate (`base.wet || collideX || (collideY &&
/// oldVelocity.Y < 0)` — the WindyBalloon adds the slave's `wet`). The
/// cue queues only on a pop; the life=-1/HitEffect/deactivate are the
/// machine's death seams. Returns `pop`.
pub(crate) fn balloon_pop_cue(npc: &mut Npc, pop: bool) -> bool {
    if pop {
        queue(npc, NpcSoundCue::NpcDeath63, npc.x, npc.y);
    }
    pop
}

/// `NPC.cs:44845-44849` (AI_122_PirateGhost): the spawn cue on the
/// `localAI[0]` 0→1 latch — `SoundID.NPCHit6` at `base.position`.
/// Returns the post-latch `localAI[0]`; the cue queues only on the 0 tick.
pub(crate) fn pirate_ghost_spawn_cue(npc: &mut Npc, local0: f32) -> f32 {
    if local0 == 0.0 {
        queue(npc, NpcSoundCue::NpcHit6, npc.x, npc.y);
        return 1.0;
    }
    local0
}

/// `NPC.cs:45552-45562` (AI_121_QueenSlime): the landing thud. On the
/// `ai[2] == 1` descent's grounded tick (`velocity.Y == 0`) the state
/// resets (`ai[0..2] = 0`) and `SoundID.Item167` plays at `base.Center`
/// (the host path). `descent` is `ai[2] == 1`, `grounded` is
/// `velocity.Y == 0`. Returns `true` when the landing fired (the caller
/// resets the state + spawns the 922 crystal); the cue queues once.
pub(crate) fn queen_slime_landing_cue(npc: &mut Npc, descent: bool, grounded: bool) -> bool {
    if descent && grounded {
        let (cx, cy) = npc.center();
        queue(npc, NpcSoundCue::ItemCue(167), cx, cy);
        return true;
    }
    false
}

/// `NPC.cs:45753-45760` (AI_121_QueenSlime): the idle moan. After the
/// `life <= 0` return, `Next(360) == 0` plays `PlaySound(65,
/// base.position)` (the case-65 zombie bank). `roll` rides the caller's
/// stream; the cue queues only on a hit. Returns the gate result.
pub(crate) fn queen_slime_idle_moan(
    npc: &mut Npc,
    alive: bool,
    roll: &mut dyn FnMut(i32) -> i32,
) -> bool {
    if !alive {
        return false;
    }
    if roll(360) == 0 {
        queue(npc, NpcSoundCue::QueenSlimeMoan, npc.x, npc.y);
        return true;
    }
    false
}

/// `NPC.cs:47252-47271` (AI_117_BloodNautilus, case 0): the volley cue.
/// When `ai[1] > num2` (60) the burst picks its count by `ai[3]`:
/// `% 7 == 3` → 3 (NO sound), `% 2 == 0` → 2 + Item170, else → 1 +
/// Item170. Returns `(count, sound)` — the Item170 cue queues at
/// `base.Center` on the two sound arms. `ai1`/`ai3` are the counters.
pub(crate) fn blood_nautilus_volley_cue(npc: &mut Npc, ai1: f32, ai3: f32) -> (i32, bool) {
    if ai1 <= 60.0 {
        return (0, false);
    }
    let n = ai3 as i32;
    if n % 7 == 3 {
        return (3, false);
    }
    let (cx, cy) = npc.center();
    queue(npc, NpcSoundCue::ItemCue(170), cx, cy);
    if n % 2 == 0 {
        (2, true)
    } else {
        (1, true)
    }
}

/// `NPC.cs:47289-47294` (AI_117_BloodNautilus, case 1): the scream cue.
/// While `ai[1] < num3` (90), the EXACT `ai[1] == num3 - 1` (89) tick
/// plays `SoundID.Item172` at `base.Center`. Returns `true` on the scream
/// tick; the cue queues once.
pub(crate) fn blood_nautilus_scream_cue(npc: &mut Npc, ai1: f32) -> bool {
    if ai1 < 90.0 && ai1 == 89.0 {
        let (cx, cy) = npc.center();
        queue(npc, NpcSoundCue::ItemCue(172), cx, cy);
        return true;
    }
    false
}

/// `NPC.cs:48323-48328` / `:48543-48546` (AI_112_FairyCritter, cases 3
/// and 5): the circle hum. On the EXACT `ai[3] == 15` tick the fairy
/// plays `PlaySound(27, (int)position.X, (int)position.Y)` (the Pixie
/// case-27 path). Returns `true` on the hum tick; the cue queues once.
pub(crate) fn fairy_circle_hum(npc: &mut Npc, ai3: f32) -> bool {
    if ai3 == 15.0 {
        queue(npc, NpcSoundCue::Pixie, npc.x, npc.y);
        return true;
    }
    false
}

/// `NPC.cs:48468-48475` (AI_112_FairyCritter, case 4): the far-approach
/// hum. While the target is > 360 px away (`num9 > num10 + 60`), the
/// `Next(30) == 0` gate plays the Pixie cue at `base.position`. `roll`
/// rides the caller's stream; the cue queues only on a hit. Returns the
/// gate result.
pub(crate) fn fairy_far_hum(
    npc: &mut Npc,
    far: bool,
    roll: &mut dyn FnMut(i32) -> i32,
) -> bool {
    if far && roll(30) == 0 {
        queue(npc, NpcSoundCue::Pixie, npc.x, npc.y);
        return true;
    }
    false
}

/// `NPC.cs:48652-48683` (`NPC.FairyEffects`, the static shimmer-release
/// burst): after the 40-dust ring, `SoundID.Item4` plays at the truncated
/// `(int)Position.X/Y`. The dust is the documented skip; the cue queues
/// the case-2 Item4 chime. `x`/`y` are the burst centre (the fairy's
/// `base.Center` at the `ai[3] > 200` despawn, `:48578`).
pub(crate) fn fairy_effects_cue(npc: &mut Npc, x: f32, y: f32) {
    queue(npc, NpcSoundCue::ItemCue(4), x, y);
}

/// `NPC.cs:56400-56404` (AI_003_Fighters' Medusa 480 block): the
/// stone-gaze cry. The gaze counter's `num25 == 1` tick plays
/// `PlaySound(4, (int)position.X, (int)position.Y, 17)` (the int
/// overload, case 4). `num25` is `ai[2] + num22 + num24` (the gaze
/// windup, `:56400`); the cue queues only on the 1 tick. Returns `true`
/// on the cry tick.
pub(crate) fn medusa_gaze_cue(npc: &mut Npc, num25: f32) -> bool {
    if num25 == 1.0 {
        queue(npc, NpcSoundCue::NpcDeath(17), npc.x, npc.y);
        return true;
    }
    false
}

/// `NPC.cs:66427-66429` (`NPC.ReflectProjectile`, the Big Mimic 473-476's
/// expert reflect): the reflect tink. `SoundID.Item150` plays at the
/// PROJECTILE's position (`proj.position`, not the NPC's). The cue queues
/// the case-2 Item150 chime; the reflect's velocity/damage rewrite is the
/// projectile layer's seam.
pub(crate) fn reflect_projectile_cue(npc: &mut Npc, proj_x: f32, proj_y: f32) {
    queue(npc, NpcSoundCue::ItemCue(150), proj_x, proj_y);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    fn npc() -> Npc {
        // The ChaosBall kind is a stand-in carrier (the helpers key on the
        // passed type/centre, not the kind); park it at a known position.
        let mut n = Npc::new(NpcKind::ChaosBall, 100.0, 200.0, 1, 0.0);
        n.vx = 3.0;
        n
    }

    /// `NPC.cs:24445-24451` — the Antlion spit queues Item5 only on the
    /// ai[0]==200 tick, decrements every positive tick, and the push is
    /// draw-free.
    #[test]
    fn w4_e9_the_antlion_spit_queues_on_the_200_tick() {
        let mut n = npc();
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(antlion_spit_cue(&mut n, 201.0), 200.0);
        assert!(n.wants_sound.is_empty(), "201 → 200, no cry yet");
        assert_eq!(antlion_spit_cue(&mut n, 200.0), 199.0);
        assert_eq!(n.wants_sound.len(), 1);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(5));
        assert_eq!((n.wants_sound[0].x, n.wants_sound[0].y), (100.0, 200.0));
        assert_eq!(antlion_spit_cue(&mut n, 0.0), 0.0);
        assert_eq!(n.wants_sound.len(), 1, "the zero tick queues nothing");
        assert_eq!(rng.peek(), peek, "the queue push draws nothing");
    }

    /// `NPC.cs:37261-37331` — the apparition/squidhead teleport queues
    /// Item8 at the centre on the localAI[0]==0 tick only.
    #[test]
    fn w4_e9_the_apparition_teleport_queues_on_the_zero_tick() {
        let mut n = npc();
        assert_eq!(apparition_teleport_cue(&mut n, 472, 0.0), 1.0);
        assert_eq!(n.wants_sound.len(), 1);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(8));
        let (cx, cy) = n.center();
        assert_eq!((n.wants_sound[0].x, n.wants_sound[0].y), (cx, cy));
        n.wants_sound.clear();
        assert_eq!(apparition_teleport_cue(&mut n, 521, 5.0), 6.0);
        assert!(n.wants_sound.is_empty(), "the non-zero tick is silent");
        assert_eq!(apparition_teleport_cue(&mut n, 472, 120.0), 120.0);
        assert!(n.wants_sound.is_empty(), "the window end is silent");
    }

    /// `NPC.cs:40559-40637` — the Sand Shark cry: the localAI[0] latch,
    /// the sand+chase+range+ai[2] gate, the style-542 queue and the -30
    /// reset; the cry fires once per latch cycle.
    #[test]
    fn w4_e9_the_sand_shark_cry_latches_and_queues_style_542() {
        let mut n = npc();
        n.facing = 1; // vx 3 > 0 → sign-match
        // local0 == 0, in sand, chasing, ahead NOT sand, dist 200 < 400,
        // ai2 == 30 → the cry fires, latches -1, ai2 → -30.
        let (l, a, cry) = sand_shark_cry(&mut n, true, true, false, 200.0, false, 0.0, 30.0);
        assert!(cry);
        assert_eq!(l, -1.0);
        assert_eq!(a, -30.0);
        assert_eq!(n.wants_sound.len(), 1);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::SandSharkCry);
        // The -1 latch out of sand → 20, then decrements; no cry.
        n.wants_sound.clear();
        let (l, _, cry) = sand_shark_cry(&mut n, false, true, false, 200.0, false, -1.0, -30.0);
        assert!(!cry);
        assert_eq!(l, 19.0, "-1 → 20 → 19");
        // The range gate: dist 500 > 400 (classic) refuses; remix 700 allows.
        n.wants_sound.clear();
        let (_, _, cry) = sand_shark_cry(&mut n, true, true, false, 500.0, false, 0.0, 30.0);
        assert!(!cry, "classic range refuses 500");
        let (_, _, cry) = sand_shark_cry(&mut n, true, true, false, 500.0, true, 0.0, 30.0);
        assert!(cry, "remix range allows 500");
        // ahead_sand refuses the cry.
        n.wants_sound.clear();
        let (_, _, cry) = sand_shark_cry(&mut n, true, true, true, 200.0, false, 0.0, 30.0);
        assert!(!cry);
    }

    /// `NPC.cs:42978-43003` — the Pal idle pain re-arms the delay + queues
    /// the kind's trackable; the joy latches -2 once.
    #[test]
    fn w4_e9_the_pal_cries_queue_the_kind_trackables() {
        let mut n = npc();
        let mut rng = UnifiedRandom::new(7);
        let delay = pal_idle_cry(&mut n, 695, 0.0, 0, &mut |lo, hi| rng.next_range(lo, hi));
        assert!(delay.is_some());
        assert_eq!(n.wants_sound.len(), 1);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::Trackable("pal_cattiva_pain"));
        // The gate: a non-zero delay or ai0 refuses.
        n.wants_sound.clear();
        assert!(pal_idle_cry(&mut n, 696, 0.0, 5, &mut |lo, hi| rng.next_range(lo, hi)).is_none());
        assert!(pal_idle_cry(&mut n, 696, 1.0, 0, &mut |lo, hi| rng.next_range(lo, hi)).is_none());
        assert!(n.wants_sound.is_empty());
        // The joy: ai0 == 2, delay != -2 → latch + queue; the -2 tick refuses.
        assert!(pal_joy_cry(&mut n, 696, 2.0, 0));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::Trackable("pal_foxparks_joy"));
        n.wants_sound.clear();
        assert!(!pal_joy_cry(&mut n, 696, 2.0, -2));
        assert!(n.wants_sound.is_empty());
    }

    /// `NPC.cs:43559-43616` — the Statue Mimic's scare/jump/laugh latches.
    #[test]
    fn w4_e9_the_statue_mimic_cries_latch_once() {
        let mut n = npc();
        assert_eq!(statue_mimic_scare(&mut n, 0.0), 1.0);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::Trackable("statuemimic_scare"));
        n.wants_sound.clear();
        assert_eq!(statue_mimic_scare(&mut n, 1.0), 1.0);
        assert!(n.wants_sound.is_empty());
        // The jump: the grounded ai2==20 tick plays; the wrap 1 → 20.
        assert_eq!(statue_mimic_jump(&mut n, 20.0, false, true), 19.0);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::Trackable("statuemimic_jump"));
        n.wants_sound.clear();
        assert_eq!(statue_mimic_jump(&mut n, 1.0, false, true), 20.0, "the wrap");
        assert!(n.wants_sound.is_empty(), "the wrap tick is silent");
        // The falling arm plays regardless of ai2.
        assert_eq!(statue_mimic_jump(&mut n, 5.0, true, false), 5.0);
        assert_eq!(n.wants_sound.len(), 1);
        // The laugh: the dead-target ai3==0 tick plays + latches 300 → 299.
        n.wants_sound.clear();
        assert_eq!(statue_mimic_laugh(&mut n, 0.0, true), 299.0);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::Trackable("statuemimic_laugh_0"));
        n.wants_sound.clear();
        assert_eq!(statue_mimic_laugh(&mut n, 0.0, false), 0.0, "alive: no laugh");
        assert!(n.wants_sound.is_empty());
    }

    /// `NPC.cs:43726`/`:47951`/`:48062` + `:44848` — the balloon pop queues
    /// NPCDeath63 on the gate; the Pirate Ghost queues NPCHit6 on the latch.
    #[test]
    fn w4_e9_the_balloon_pop_and_pirate_ghost_spawn() {
        let mut n = npc();
        assert!(balloon_pop_cue(&mut n, true));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::NpcDeath63);
        n.wants_sound.clear();
        assert!(!balloon_pop_cue(&mut n, false));
        assert!(n.wants_sound.is_empty());
        assert_eq!(pirate_ghost_spawn_cue(&mut n, 0.0), 1.0);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::NpcHit6);
        n.wants_sound.clear();
        assert_eq!(pirate_ghost_spawn_cue(&mut n, 1.0), 1.0);
        assert!(n.wants_sound.is_empty());
    }

    /// `NPC.cs:45552-45760` — the Queen Slime landing thud + the idle
    /// moan's Next(360) gate.
    #[test]
    fn w4_e9_the_queen_slime_landing_and_idle_moan() {
        let mut n = npc();
        assert!(queen_slime_landing_cue(&mut n, true, true));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(167));
        n.wants_sound.clear();
        assert!(!queen_slime_landing_cue(&mut n, true, false));
        assert!(!queen_slime_landing_cue(&mut n, false, true));
        assert!(n.wants_sound.is_empty());
        // The idle moan: a hitting roll queues; a miss / dead does not.
        let mut hit = |_: i32| 0;
        assert!(queen_slime_idle_moan(&mut n, true, &mut hit));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::QueenSlimeMoan);
        n.wants_sound.clear();
        let mut miss = |_: i32| 1;
        assert!(!queen_slime_idle_moan(&mut n, true, &mut miss));
        assert!(!queen_slime_idle_moan(&mut n, false, &mut hit));
        assert!(n.wants_sound.is_empty());
    }

    /// `NPC.cs:47252-47294` — the Blood Nautilus volley count/sound split
    /// and the scream's exact 89 tick.
    #[test]
    fn w4_e9_the_blood_nautilus_volley_and_scream() {
        let mut n = npc();
        // ai3 % 7 == 3 → count 3, no sound (e.g. ai3 = 3).
        assert_eq!(blood_nautilus_volley_cue(&mut n, 61.0, 3.0), (3, false));
        assert!(n.wants_sound.is_empty());
        // ai3 % 2 == 0 → count 2 + Item170 (ai3 = 4).
        assert_eq!(blood_nautilus_volley_cue(&mut n, 61.0, 4.0), (2, true));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(170));
        n.wants_sound.clear();
        // else → count 1 + Item170 (ai3 = 5).
        assert_eq!(blood_nautilus_volley_cue(&mut n, 61.0, 5.0), (1, true));
        // ai1 <= 60 → no volley.
        n.wants_sound.clear();
        assert_eq!(blood_nautilus_volley_cue(&mut n, 60.0, 4.0), (0, false));
        assert!(n.wants_sound.is_empty());
        // The scream: only ai1 == 89.
        assert!(blood_nautilus_scream_cue(&mut n, 89.0));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(172));
        n.wants_sound.clear();
        assert!(!blood_nautilus_scream_cue(&mut n, 88.0));
        assert!(!blood_nautilus_scream_cue(&mut n, 90.0));
        assert!(n.wants_sound.is_empty());
    }

    /// `NPC.cs:48327`/`:48473`/`:48545`/`:48682` — the FairyCritter hums
    /// and the FairyEffects chime.
    #[test]
    fn w4_e9_the_fairy_hums_and_effects_chime() {
        let mut n = npc();
        assert!(fairy_circle_hum(&mut n, 15.0));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::Pixie);
        n.wants_sound.clear();
        assert!(!fairy_circle_hum(&mut n, 14.0));
        assert!(n.wants_sound.is_empty());
        let mut hit = |_: i32| 0;
        assert!(fairy_far_hum(&mut n, true, &mut hit));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::Pixie);
        n.wants_sound.clear();
        assert!(!fairy_far_hum(&mut n, false, &mut hit));
        assert!(n.wants_sound.is_empty());
        fairy_effects_cue(&mut n, 300.0, 400.0);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(4));
        assert_eq!((n.wants_sound[0].x, n.wants_sound[0].y), (300.0, 400.0));
    }

    /// `NPC.cs:56403` + `:66429` — Medusa's gaze cry on num25==1 and the
    /// Big Mimic reflect tink at the projectile position.
    #[test]
    fn w4_e9_the_medusa_gaze_and_the_reflect_tink() {
        let mut n = npc();
        assert!(medusa_gaze_cue(&mut n, 1.0));
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::NpcDeath(17));
        n.wants_sound.clear();
        assert!(!medusa_gaze_cue(&mut n, 2.0));
        assert!(n.wants_sound.is_empty());
        reflect_projectile_cue(&mut n, 500.0, 600.0);
        assert_eq!(n.wants_sound[0].cue, NpcSoundCue::ItemCue(150));
        assert_eq!((n.wants_sound[0].x, n.wants_sound[0].y), (500.0, 600.0));
    }
}