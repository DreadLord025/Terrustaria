//! W4-e7 DOC-SWEEP lane 23 (strike half) — the Snapping Stone accessory
//! trio's strike machine: `Projectile.TryMakingSnappingStoneStrike`
//! (`Projectile.cs:14283-14395`) plus the `ParticleOrchestrator` visual
//! census (the `prefix.rs` cite-skip convention: the port has no particle
//! renderer, but every `Main.rand` draw the spawn bodies consume is
//! replayed here in the C#'s order so the shared-stream interleave stays
//! exact).
//!
//! The trio: Snapping Stone 6167 (`accSnappingStone`, the ranged strike,
//! `snappingType` 0), Pyroclast 6178 (`accPyroclast`, melee, type 1),
//! Armlet of Ruin 6179 (`accArmletOfRuin`, summon, type 2). The flags /
//! cooldown / ready lights landed in `player.rs` + `player_lights_b.rs`
//! (`5ae32127`); this module arms the 360-tick cooldown (`:14316`) the
//! countdown (`Player.Update_SnappingStone`, `Player.cs:28385-28410`)
//! ticks down, spawns the 1121/1131/1132 effect projectiles and adds the
//! `dmg += num13` bonus (`:14368-14382`).
//!
//! Call sites:
//! * melee — `Player.ProcessHitAgainstNPC` (`Player.cs:44338-44341`),
//!   accPyroclast → type 1. WIRED in `world_sim/editing.rs`'s
//!   `resolve_melee_strike`, between the crit roll (`:44298`) and
//!   `Main.DamageVar` (`:44342`) — the strike's bonus rides the pre-variance
//!   `dmg`, exactly the C#'s order.
//! * projectile — `Damage_PVE_Inner` (`Projectile.cs:13397-13408`):
//!   `(melee || ranged) & owned` + accPyroclast → type 1, `ranged & owned`
//!   + accSnappingStone → type 0, `changes.AddProjectileTagDamage & owned`
//!   + accArmletOfRuin → type 2. WIRED in `projectile.rs::damage_pve` at
//!   the `:13397` position (after the yoyo doublings, before the strike).
//!   The type-2 gate reads the tag stack's `ModifyHit` result — the SAME
//!   `tag_damage_for` computation the damage math ran at `:12919-12929`
//!   (the C# computes `changes` once at `:12920`; the port's `tag_damage_for`
//!   now also returns the `AddProjectileTagDamage` flag, so no second
//!   `ModifyHit` walk / no doubled whip-tag crit draw).
//! * recharge — `Update_SnappingStone`'s zero-crossing
//!   (`Player.cs:28392-28408`). WIRED in `world_sim/player.rs` at the
//!   `:24564` countdown site: the first worn member wins (Item30 / Item20 /
//!   Item28 at `MountedCenter`) and its recharge burst's draws replay.
//!
//! Deliberate folds (recorded, research/102 §1):
//! * `owner != Main.myPlayer` (`:14288`) folds to the port's local-owner
//!   gate — the callers only run for locally-simulated hits.
//! * `FindTargetWithLineOfSight`'s `ignoreTargets.Contains(i)`
//!   (`:45639`) is an NPC-index skip list; the shared port helper's
//!   `immune` parameter is the per-slot local-immunity row instead, so
//!   this module carries its own scan wrapper (the `:45632-45660` body
//!   with the index-list skip, reusing `chaseable`/`npc_center`/`can_hit`).
//! * The particle VISUALS are the documented skip; the RNG draws are kept
//!   (the census below). `Dust.NewDust`'s pool-pressure / off-screen early
//!   outs (`Dust.cs:97-148`) are assumed not to fire (the `prefix.rs`
//!   pool assumption); `ChildSafety.Disabled = true`
//!   (`ChildSafety.cs:33`) kills the dangerous-dust branch.
//! * The `Spawn_*` bodies' own sounds (Item72 `:605`, PyroclasticStone
//!   `:658`, ArmletOfRuin `:745`) ride the audio façade at the call sites;
//!   the 206/207 styles have no bank cue yet (the façade returns `None`
//!   after the same draws — a bank gap, not a behaviour gap).

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

use crate::npc::Npc;
use crate::player::Player;
use crate::projectile::{
    can_hit, chaseable, npc_center, ProjectilePool, SNAPPING_STONE_EFFECT,
    SNAPPING_STONE_MELEE_EFFECT, SNAPPING_STONE_SUMMON_EFFECT,
};

/// The draw surface the strike machine needs on top of [`DropRng`]:
/// `Next(max)` and `NextFloatDirection`. The PVE walk threads
/// `&mut dyn DropRng` (the pool's tests pass `FastRandom`), so the machine
/// is generic over this trait. The blanket impl expresses both through
/// `next_range`/`next_float` — for `UnifiedRandom` these are the SAME
/// single `Sample()` step as `next_max`/`next_float_direction`
/// (`next_range(0, max)` = `(sample * max) as i32`, `next_float() * 2 - 1`
/// = `(sample as f32) * 2 - 1`), so the stream advance is exact; the
/// production path always passes the named `UnifiedRandom` stream.
pub(crate) trait StrikeRng: DropRng {
    fn strike_next_max(&mut self, max: i32) -> i32 {
        self.next_range(0, max)
    }
    fn strike_next_float_direction(&mut self) -> f32 {
        self.next_float() * 2.0 - 1.0
    }
}

impl<T: DropRng + ?Sized> StrikeRng for T {}

/// One queued strike sound: the `SOUND_STYLES` name, the world position,
/// the C# call's `pitchOffset` and `volumeScale` (`SoundEngine.cs:88-99`).
/// The melee/recharge sites play inline (the `UpdatePlayers` stream); the
/// PVE site queues them on the pool (`pending_striking_sounds`) and the
/// world-sim pass drains them on the `UpdateProjectiles` stream — the
/// S1-E-21c queue→drain convention (the case-2 draws defer to the pass
/// tail, the recorded ANALYSIS deviation; the particle DUST draws stay
/// inline in the C#'s order).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StrikeSound {
    pub style: &'static str,
    pub x: f32,
    pub y: f32,
    pub pitch_offset: f32,
    pub volume_scale: f32,
}

/// `Projectile.TryMakingSnappingStoneStrike` (`Projectile.cs:14283-14395`).
/// `targeter` is the hitting entity's box (the player for the melee site,
/// the projectile slot for the PVE site — the C# `Entity targeter`),
/// `victim` the struck NPC's index, `dmg` the pre-`DamageVar` damage the
/// bonus adds to (`ref int dmg`), `snapping_type` 0/1/2.
///
/// The `owner != Main.myPlayer` half of the `:14288` gate folds into the
/// callers (local-only hits); the cooldown half is here. The sounds the
/// C# plays inline (the particle bodies' Item72/PyroclasticStone/
/// ArmletOfRuin + the owner-side Item30/Item20/Item43) are QUEUED on the
/// pool (`drain_striking_sounds`) — the S1-E-21c queue→drain convention:
/// the pool cannot borrow the audio mid-walk, so the case-2 draws defer
/// to the pass tail (the recorded ANALYSIS deviation); the particle DUST
/// draws stay inline in the C#'s order.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_making_snapping_stone_strike<R: StrikeRng + ?Sized>(
    pool: &mut ProjectilePool,
    world: &World,
    npcs: &mut [Npc],
    player: &mut Player,
    owner: u8,
    targeter: (f32, f32, f32, f32),
    victim: usize,
    dmg: &mut i32,
    snapping_type: u8,
    rng: &mut R,
) {
    // `:14288` — the cooldown gate (the myPlayer half folded, module doc).
    if player.acc_snapping_stone_cooldown > 0 {
        return;
    }
    let ranged = snapping_type == 0;
    let melee = snapping_type == 1;
    let summon = snapping_type == 2;
    // `:14292-14312` — the per-class damage / bonus base / range.
    let mut proj_damage = 30;
    let mut max_range = 120.0f32;
    let max_strikes = 2;
    let mut bonus_base = 40;
    let bonus_per_miss = 10;
    if ranged {
        proj_damage = (proj_damage as f32 * player.ranged_damage) as i32;
    } else if melee {
        proj_damage = (45.0 * player.melee_damage) as i32;
        bonus_base = 70;
        max_range = 300.0;
    } else if summon {
        proj_damage = (150.0 * player.minion_damage) as i32;
        bonus_base = 150;
        max_range = 400.0;
    }
    // `:14313-14316` — the skip list starts with the victim; the cooldown
    // arms BEFORE the scan (a scan that finds nothing still cools down).
    let mut skip: Vec<usize> = vec![victim];
    player.acc_snapping_stone_cooldown = 360;
    let knock_back = 5.0;
    let mut proj_kind = SNAPPING_STONE_EFFECT;
    if melee {
        proj_kind = SNAPPING_STONE_MELEE_EFFECT;
    } else if summon {
        proj_kind = SNAPPING_STONE_SUMMON_EFFECT;
    }
    // `:14330-14362` — the 2-strike scan loop. `num7` counts the landed
    // extra strikes; the particle `MovementVector.Y` is `num7 + 1`.
    let mut struck = 0usize;
    for _ in 0..max_strikes {
        // `:14333` — FindTargetWithLineOfSight(targeter, maxRange, skip).
        let found = find_target_skipping(world, npcs, targeter, max_range, &skip);
        let Some(idx) = found else { break };
        let (ncx, ncy) = npcs[idx].center();
        let (nw, nh) = npcs[idx].size();
        let (nx, ny) = (npcs[idx].x, npcs[idx].y);
        // `:14339-14340` — NewProjectile at the NPC centre, velocity
        // (0, 0.1), timeLeft pinned to 2. The port's spawn takes the
        // top-left (`NewProjectile` centres the box, `:10876-10877`), so
        // the 10×10 row is re-centred (the `spawn_inferno_blast` shape).
        if let Some(slot) = pool.spawn_with_data_owned(
            proj_kind,
            owner,
            ncx - 5.0,
            ncy - 5.0,
            0.0,
            0.1,
            proj_damage,
            knock_back,
            0.0,
            0.0,
        ) {
            let p = &mut pool.slots_mut()[slot];
            p.time_left = 2;
            if melee {
                // `:14343-14345` — the class flip rides the crit comparison.
                let flag4 = player.melee_crit > player.ranged_crit;
                p.melee = flag4;
                p.ranged = !flag4;
            }
        }
        if melee {
            // `:14346` — the extra target takes Hellfire 323 for 180.
            npcs[idx].buffs.add_buff(323, 180);
        }
        // `:14348-14359` — the per-target particle spawn. The C# draws the
        // `num12` scale (`:14353`, one NextFloat) FIRST, then the settings'
        // `NextFromRectangle` position (`:14354`, two int draws) — the port
        // keeps that order.
        let radius = (nw * nw + nh * nh).sqrt() / 2.0;
        let radius = radius.min(100.0);
        let scale = 0.3 + 1.4 * rng.next_float(); // :14353
        // :14354 — NextFromRectangle(nPC.Hitbox): Next(Left, Right) then
        // Next(Top, Bottom) (`Utils.cs:1246-1249`), the int-truncated box
        // (`Entity.cs:171-176`).
        let hx = nx as i32;
        let hy = ny as i32;
        let px = rng.next_range(hx, hx + nw as i32) as f32;
        let py = rng.next_range(hy, hy + nh as i32) as f32;
        // Lerp(point, centre, 0.6) — the value is visual-only; the draws
        // above are the stream cost.
        let _position = Vec2::new(px + (ncx - px) * 0.6, py + (ncy - py) * 0.6);
        let mut sounds = Vec::new();
        spawn_particle_draws(snapping_type, radius * scale, (struck + 1) as f32, rng, &mut sounds, _position);
        pool.queue_striking_sounds(sounds);
        // `:14360-14361` — the struck NPC joins the skip list.
        skip.push(idx);
        struck += 1;
    }
    // `:14363-14367` — the victim's own burst: MovementVector = (hitbox
    // diagonal / 2, 0), NO clamp (the 100 cap is the loop's alone) and no
    // settings draws (the position is the centre, not a rectangle roll).
    let (vcx, vcy) = npcs[victim].center();
    let (vw, vh) = npcs[victim].size();
    let victim_radius = (vw * vw + vh * vh).sqrt() / 2.0;
    let mut sounds = Vec::new();
    spawn_particle_draws(snapping_type, victim_radius, 0.0, rng, &mut sounds, Vec2::new(vcx, vcy));
    // `:14368-14382` — the damage bonus: base + 10 per MISSED strike slot.
    let mut bonus = bonus_base + bonus_per_miss * (max_strikes - struck) as i32;
    if ranged {
        bonus = (bonus as f32 * player.ranged_damage) as i32;
    } else if melee {
        bonus = (bonus as f32 * player.melee_damage.max(player.ranged_damage)) as i32;
        // `:14376` — the victim takes Hellfire too (the melee arm only).
        npcs[victim].buffs.add_buff(323, 180);
    } else if summon {
        bonus = (bonus as f32 * player.minion_damage) as i32;
    }
    *dmg += bonus;
    // `:14383-14394` — the owner-side cue at MountedCenter.
    let (mcx, mcy) = player.mounted_center();
    let style = if ranged {
        // :14385 — Item30, pitchOffset 0.3, volumeScale 0.08.
        Some(("Item30", 0.3, 0.08))
    } else if melee {
        // :14389 — Item20, volumeScale 0.08 (no pitch offset).
        Some(("Item20", 0.0, 0.08))
    } else if summon {
        // :14393 — Item43, pitchOffset 0.3, volumeScale 0.08.
        Some(("Item43", 0.3, 0.08))
    } else {
        None
    };
    if let Some((name, pitch, volume)) = style {
        sounds.push(StrikeSound { style: name, x: mcx, y: mcy, pitch_offset: pitch, volume_scale: volume });
    }
    pool.queue_striking_sounds(sounds);
}

/// `Projectile.FindTargetWithLineOfSight` (`Projectile.cs:45632-45660`)
/// with the `ignoreTargets` index list — the shared port helper's
/// `immune` slice is the local-immunity row, not this list (module doc).
/// The body is the C#'s: nearest chaseable NPC inside `max_range` with a
/// clear `Collision.CanHit` line, distance-strict (`num2 < num`).
fn find_target_skipping(
    world: &World,
    npcs: &[Npc],
    targeter: (f32, f32, f32, f32),
    max_range: f32,
    skip: &[usize],
) -> Option<usize> {
    let (x, y, w, h) = targeter;
    let center = Vec2::new(x + w * 0.5, y + h * 0.5);
    let mut best = max_range;
    let mut result = None;
    for (i, n) in npcs.iter().enumerate() {
        if skip.contains(&i) {
            continue; // :45639
        }
        if !chaseable(n) {
            continue; // :45644
        }
        let distance = center.distance(npc_center(n)); // :45651
        if distance < best && can_hit(world, x, y, w, h, n.x, n.y, n.size().0, n.size().1) {
            best = distance;
            result = Some(i);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// The ParticleOrchestrator visual census (cite-skip WITH draws kept).
//
// Per-arm draw counts (the WRITTEN census, research/102 §1 updated):
// * Spawn_SnappingStone (`ParticleOrchestrator.cs:593-646`): 2 FadingParticles
//   (draw-free) + 20 dust. Each dust: Next(3) (`:627`, the flag pick) THEN
//   NewDustDirect's 6 (`Dust.cs:170-175`, `:190`) + [if flag: Next(6)
//   (`:638`)]. Total = 20×6 + 20 + (#flag-true) = 140 + (#flag-true).
//   The sound Item72 (`:605`) plays BEFORE the loop.
// * Spawn_SnappingStoneMelee (`:648-731`): 1 NextFloatDirection (`:661`),
//   1 red PrettySparkle (draw-free), 1 yellow PrettySparkle (draw-free),
//   then 2 iterations × 2 dust, each NewDustPerfect's 6 + the type-6
//   extra Next(-10, 6) (`Dust.cs:214-216`) + NextFloatDirection + NextFloat
//   (the velocity, `:723`/`:726`) = 9. Total 1 + 4×9 = 37. Sound
//   PyroclasticStone (`:658`) before the sparkles.
// * Spawn_SnappingStoneSummon (`:733-813`): 2 NextFloat (the Extra[34]
//   particle's RotationVelocity + Rotation, `:783-784`), then 15 dust-172
//   + 8 dust-211, each NewDustPerfect's 6 + NextVector2Circular(10,10)'s 2
//   + Next(6) (`:794`/`:807`) = 9. Total 2 + 23×9 = 209. Sound ArmletOfRuin
//   (`:745`) before the particles. (The 2+1 FadingParticles are draw-free.)
// * Spawn_SnappingStoneRecharge (`:454-470`): 10 dust-226 × NewDust's 6
//   (w/h = the player box) = 60. No sound in the body (the Item30 cue is
//   Update_SnappingStone's own, `Player.cs:28396`).
// * Spawn_PyroclasticStoneRecharge (`:437-452`): 15 dust-6 × (6 + the
//   type-6 extra 1 + NextVector2Circular(2,4)'s 2) = 15×9 = 135.
// * Spawn_ArmletOfRuinRecharge (`:418-435`): 10 dust-15 × 6 = 60.
// ---------------------------------------------------------------------------

/// `Dust.NewDust`'s 6 draws (`Dust.cs:170-175`, `:190`) under the pool
/// assumption (the `prefix.rs::reforge_new_dust_draws` shape), with the
/// caller's box clamped to the 5 floor (`:155-162`).
fn new_dust_draws<R: StrikeRng + ?Sized>(rng: &mut R, w: i32, h: i32) {
    let w = w.max(5);
    let h = h.max(5);
    rng.strike_next_max(w - 4); // :170
    rng.strike_next_max(h - 4); // :171
    rng.next_range(-20, 21); // :172
    rng.next_range(-20, 21); // :173
    rng.strike_next_max(3); // :175
    rng.next_range(-20, 21); // :190
}

/// The type-6 fire dust's extra velocity.Y draw (`Dust.cs:214-216`).
fn dust_type_tail<R: StrikeRng + ?Sized>(rng: &mut R, dust_type: i32) {
    if dust_type == 6 {
        rng.next_range(-10, 6); // :216
    }
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`): the unit angle +
/// the radius scale — 2 draws (the `prefix.rs` circular shape).
fn circular_draws<R: StrikeRng + ?Sized>(rng: &mut R) {
    rng.next_float();
    rng.next_float();
}

/// One strike/recharge particle spawn's draws + its body sound, in the
/// C#'s order. `movement_x`/`movement_y` are the settings' MovementVector
/// (the visual scale/delay inputs — only the summon arm's `num` clamp
/// reads them, and it clamps to the same 0.12 either way; the values are
/// kept for the citation walk). The visuals themselves are the skip.
fn spawn_particle_draws<R: StrikeRng + ?Sized>(
    snapping_type: u8,
    movement_x: f32,
    movement_y: f32,
    rng: &mut R,
    sounds: &mut Vec<StrikeSound>,
    position: Vec2,
) {
    match snapping_type {
        0 => {
            // Spawn_SnappingStone (:593-646). Item72 at the position (:605).
            sounds.push(StrikeSound { style: "Item72", x: position.x, y: position.y, pitch_offset: 0.0, volume_scale: 1.0 });
            let _num = (movement_x / 256.0).min(0.12); // :602/:606-609
            let _delay = movement_y as i32; // :603
            for _ in 0..20 {
                // :627-628 — NewDustDirect(position, 0, 0, flag ? 226 : 15).
                let flag = rng.strike_next_max(3) != 0;
                let dust_type = if flag { 226 } else { 15 };
                new_dust_draws(rng, 0, 0);
                dust_type_tail(rng, dust_type);
                if flag {
                    rng.strike_next_max(6); // :638
                }
            }
        }
        1 => {
            // Spawn_SnappingStoneMelee (:648-731). PyroclasticStone (:658).
            sounds.push(StrikeSound { style: "PyroclasticStone", x: position.x, y: position.y, pitch_offset: 0.0, volume_scale: 0.5 });
            rng.strike_next_float_direction(); // :661 — num4
            for _ in 0..2 {
                // :721-729 — 2 dust per sparkle, type 6, NewDustPerfect.
                // The velocity ARGUMENT draws first (C# evaluates the
                // arguments before NewDustPerfect → NewDust's body):
                // NextFloatDirection + NextFloat, THEN the 6 + type-6 tail.
                for _ in 0..2 {
                    rng.strike_next_float_direction(); // :723/:726 — RotatedBy angle
                    rng.next_float(); // :723/:726 — the velocity scale
                    new_dust_draws(rng, 0, 0);
                    dust_type_tail(rng, 6);
                }
            }
        }
        2 => {
            // Spawn_SnappingStoneSummon (:733-813). ArmletOfRuin (:745).
            sounds.push(StrikeSound { style: "ArmletOfRuin", x: position.x, y: position.y, pitch_offset: 0.0, volume_scale: 0.5 });
            rng.strike_next_float_direction(); // :783 — RotationVelocity
            rng.next_float(); // :784 — Rotation
            for dust_type in [172, 211] {
                let count = if dust_type == 172 { 15 } else { 8 };
                for _ in 0..count {
                    // :791/:804 — NewDustPerfect(position, type,
                    // NextVector2Circular(10,10)): the circular ARGUMENT
                    // draws (2) BEFORE NewDust's body (6); 172/211 have no
                    // type tail. Then the Next(6) gravity roll.
                    circular_draws(rng);
                    new_dust_draws(rng, 0, 0);
                    dust_type_tail(rng, dust_type);
                    rng.strike_next_max(6); // :794/:807
                }
            }
        }
        _ => {}
    }
}

/// `Player.Update_SnappingStone`'s zero-crossing recharge
/// (`Player.cs:28392-28408`): the first worn member wins. Returns the
/// sound cue (the caller plays it at `MountedCenter` on the
/// `UpdatePlayers` stream) and replays the recharge burst's draws on the
/// same stream. `None` when no member is worn (the C# plays nothing).
pub(crate) fn snapping_stone_recharge<R: StrikeRng + ?Sized>(
    player: &Player,
    rng: &mut R,
) -> Option<StrikeSound> {
    let (mcx, mcy) = player.mounted_center();
    let (w, h) = (player.width as i32, player.height as i32);
    if player.acc_snapping_stone {
        // :28396-28397 — Item30 (-0.9 pitch, 0.08 volume) + the 10-dust
        // SnappingStoneRecharge burst (226, the player box).
        for _ in 0..10 {
            new_dust_draws(rng, w, h); // Spawn_SnappingStoneRecharge :459
            dust_type_tail(rng, 226);
        }
        Some(StrikeSound { style: "Item30", x: mcx, y: mcy, pitch_offset: -0.9, volume_scale: 0.08 })
    } else if player.acc_pyroclast {
        // :28401-28402 — Item20 (default) + the 15-dust PyroclasticStone
        // Recharge burst (6, circular velocity).
        for _ in 0..15 {
            // :442 — NewDust(player box, type 6) FIRST (the velocity is a
            // post-spawn assignment at :448, NOT an argument).
            new_dust_draws(rng, w, h); // Spawn_PyroclasticStoneRecharge :442
            dust_type_tail(rng, 6);
            circular_draws(rng); // :448 — NextVector2Circular(2,4)
        }
        Some(StrikeSound { style: "Item20", x: mcx, y: mcy, pitch_offset: 0.0, volume_scale: 1.0 })
    } else if player.acc_armlet_of_ruin {
        // :28406-28407 — Item28 (-0.9 pitch, 0.08 volume) + the 10-dust
        // ArmletOfRuinRecharge burst (15, the player box).
        for _ in 0..10 {
            new_dust_draws(rng, w, h); // Spawn_ArmletOfRuinRecharge :423
            dust_type_tail(rng, 15);
        }
        Some(StrikeSound { style: "Item28", x: mcx, y: mcy, pitch_offset: -0.9, volume_scale: 0.08 })
    } else {
        None
    }
}

/// Play one queued strike/recharge sound through the style façade
/// (`SoundEngine.cs:93-99` → `Audio::play_use_sound_scaled`), honouring the
/// C# call's `volumeScale` (`LegacySoundPlayer.cs:1411`'s post-attenuation
/// multiply). The gameplay draws ride `rng` (the hitting lane's stream);
/// the `GetRandomPitch` NextFloat rides the audio-only stream inside the
/// façade. The 206/207 styles (PyroclasticStone/ArmletOfRuin) have no bank
/// cue yet — the façade draws identically then returns `None` (a bank gap).
pub(crate) fn play_striking_sound(
    audio: &mut crate::audio::Audio,
    sound: StrikeSound,
    rng: &mut UnifiedRandom,
) {
    let Some(idx) = crate::soundid::SOUND_STYLES
        .iter()
        .position(|s| s.name == sound.style)
    else {
        return;
    };
    let _ = audio.play_use_sound_scaled(
        idx as u16,
        sound.x,
        sound.y,
        sound.pitch_offset,
        sound.volume_scale,
        rng,
    );
}

/// The melee site's gate (`Player.cs:44338-44341`): accPyroclast → type 1.
/// Kept as a named helper so the editing.rs call reads like the C#.
pub(crate) fn melee_strike_type(player: &Player) -> Option<u8> {
    if player.acc_pyroclast {
        Some(1)
    } else {
        None
    }
}

/// The PVE site's three gates (`Projectile.cs:13397-13408`), in order.
/// `melee`/`ranged` are the slot's class flags, `tag_damage` the
/// `changes.AddProjectileTagDamage` the tag stack computed. Returns the
/// types to fire (0..3 of them — the C# runs three independent ifs).
pub(crate) fn pve_strike_types(
    player: &Player,
    melee: bool,
    ranged: bool,
    tag_damage: bool,
) -> Vec<u8> {
    let mut out = Vec::new();
    if (melee || ranged) && player.acc_pyroclast {
        out.push(1); // :13397-13400
    }
    if ranged && player.acc_snapping_stone {
        out.push(0); // :13401-13404
    }
    if tag_damage && player.acc_armlet_of_ruin {
        out.push(2); // :13405-13408
    }
    out
}

// ---------------------------------------------------------------------------
// Witness tests-as-code — WRITTEN NOT RUN (code-only mode). Verified by
// re-reading the cited C# and hand arithmetic; run them with
// `cargo test -p tr-app snapping_stone` once a compiler is available.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 1600.0;
        p.y = 800.0;
        p
    }

    fn zombie(cx: f32, cy: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        let (nw, nh) = n.size();
        n.x = cx - nw * 0.5;
        n.y = cy - nh * 0.5;
        n
    }

    /// The cooldown gate (`:14288`): a cooling owner strikes nothing — no
    /// bonus, no spawns, no draws, no sounds.
    #[test]
    fn w4_e7_the_strike_refuses_while_cooling() {
        let w = air();
        let mut p = player(&w);
        p.acc_snapping_stone_cooldown = 10;
        let mut npcs = vec![zombie(1620.0, 810.0)];
        let mut pool = ProjectilePool::new();
        let mut rng = UnifiedRandom::new(7);
        let before = rng.peek();
        let mut dmg = 10;
        let hitbox = (p.x, p.y, p.width, p.height);
        try_making_snapping_stone_strike(
            &mut pool, &w, &mut npcs, &mut p, 0,
            hitbox, 0, &mut dmg, 0, &mut rng,
        );
        assert_eq!(dmg, 10, "no bonus while cooling");
        assert!(pool.drain_striking_sounds().is_empty());
        assert_eq!(rng.peek(), before, "the gate draws nothing");
        assert_eq!(p.acc_snapping_stone_cooldown, 10, "the cooldown is untouched");
        assert!(pool.slots().iter().all(|s| !s.active), "no spawn");
    }

    /// A ready ranged strike with no extra target in range: the victim
    /// burst still fires, the bonus is the full 40 + 10×2 = 60 (×1 ranged),
    /// the cooldown arms to 360, and no effect projectile spawns.
    #[test]
    fn w4_e7_the_ranged_strike_bonuses_the_missed_slots() {
        let w = air();
        let mut p = player(&w);
        let mut npcs = vec![zombie(1620.0, 810.0)];
        let mut pool = ProjectilePool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut dmg = 10;
        let hitbox = (p.x, p.y, p.width, p.height);
        try_making_snapping_stone_strike(
            &mut pool, &w, &mut npcs, &mut p, 0,
            hitbox, 0, &mut dmg, 0, &mut rng,
        );
        assert_eq!(p.acc_snapping_stone_cooldown, 360, "the setter (:14316)");
        assert_eq!(dmg, 70, "40 + 10×2 missed slots, ×1 ranged (:14368-14371)");
        assert!(pool.slots().iter().all(|s| !s.active), "no extra target → no spawn");
    }

    /// The melee strike's class flip + Hellfire: an extra target in range
    /// spawns 1131 with melee/ranged set by the crit comparison, takes
    /// buff 323/180, and the victim takes it too (`:14346`/`:14376`).
    #[test]
    fn w4_e7_the_melee_strike_flips_class_and_burns() {
        let w = air();
        let mut p = player(&w);
        p.melee_crit = 10;
        p.ranged_crit = 4;
        let mut npcs = vec![zombie(1620.0, 810.0), zombie(1660.0, 810.0)];
        let mut pool = ProjectilePool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut dmg = 10;
        let hitbox = (p.x, p.y, p.width, p.height);
        try_making_snapping_stone_strike(
            &mut pool, &w, &mut npcs, &mut p, 0,
            hitbox, 0, &mut dmg, 1, &mut rng,
        );
        let spawned: Vec<_> = pool.slots().iter().filter(|s| s.active).collect();
        assert_eq!(spawned.len(), 1, "one extra target in the 300 range");
        assert_eq!(spawned[0].kind, SNAPPING_STONE_MELEE_EFFECT);
        assert_eq!(spawned[0].time_left, 2, "the timeLeft pin (:14340)");
        assert!(spawned[0].melee && !spawned[0].ranged, "meleeCrit > rangedCrit (:14343-14345)");
        assert_eq!(spawned[0].damage, 45, "45 × meleeDamage 1 (:14303)");
        assert!(npcs[1].buffs.has_buff(323), "the extra target burns (:14346)");
        assert!(npcs[0].buffs.has_buff(323), "the victim burns (:14376)");
        // bonus = (70 + 10×1) × max(melee 1, ranged 1) = 80.
        assert_eq!(dmg, 90, "10 + 80 (:14368-14375)");
    }

    /// The summon strike's 150 base + 150 bonus base, the 400 range, and
    /// the 1132 row (no class flag — the summon arm sets none).
    #[test]
    fn w4_e7_the_summon_strike_uses_the_minion_numbers() {
        let w = air();
        let mut p = player(&w);
        p.minion_damage = 2.0;
        let mut npcs = vec![zombie(1620.0, 810.0)];
        let mut pool = ProjectilePool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut dmg = 0;
        let hitbox = (p.x, p.y, p.width, p.height);
        try_making_snapping_stone_strike(
            &mut pool, &w, &mut npcs, &mut p, 0,
            hitbox, 0, &mut dmg, 2, &mut rng,
        );
        // bonus = (150 + 10×2) × minionDamage 2 = 340.
        assert_eq!(dmg, 340, "(:14368/:14380)");
        assert!(!npcs[0].buffs.has_buff(323), "the summon arm never burns");
    }

    /// The recharge dispatch (`:28394-28408`): the first worn member wins,
    /// each with its own cue + burst draw count.
    #[test]
    fn w4_e7_the_recharge_picks_the_first_worn_member() {
        let w = air();
        let mut p = player(&w);
        // No member → no cue, no draws.
        let mut rng = UnifiedRandom::new(7);
        let before = rng.peek();
        assert!(snapping_stone_recharge(&p, &mut rng).is_none());
        assert_eq!(rng.peek(), before);
        // Snapping Stone wins over Pyroclast (the if-else-if order).
        p.acc_snapping_stone = true;
        p.acc_pyroclast = true;
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let cue = snapping_stone_recharge(&p, &mut rng).unwrap();
        assert_eq!(cue.style, "Item30");
        assert_eq!(cue.pitch_offset, -0.9);
        for _ in 0..10 {
            // Spawn_SnappingStoneRecharge: 10 × NewDust(player box) = 60.
            new_dust_draws(&mut witness, p.width as i32, p.height as i32);
        }
        assert_eq!(rng.peek(), witness.peek(), "SnappingStoneRecharge = 60 draws");
        // Pyroclast alone: 15 × (6 + type-6 tail 1 + circular 2) = 135.
        p.acc_snapping_stone = false;
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let cue = snapping_stone_recharge(&p, &mut rng).unwrap();
        assert_eq!(cue.style, "Item20");
        for _ in 0..15 {
            new_dust_draws(&mut witness, p.width as i32, p.height as i32);
            witness.next_range(-10, 6);
            witness.next_float();
            witness.next_float();
        }
        assert_eq!(rng.peek(), witness.peek(), "PyroclasticStoneRecharge = 135 draws");
        // Armlet alone: 10 × 6 = 60.
        p.acc_pyroclast = false;
        p.acc_armlet_of_ruin = true;
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let cue = snapping_stone_recharge(&p, &mut rng).unwrap();
        assert_eq!(cue.style, "Item28");
        for _ in 0..10 {
            new_dust_draws(&mut witness, p.width as i32, p.height as i32);
        }
        assert_eq!(rng.peek(), witness.peek(), "ArmletOfRuinRecharge = 60 draws");
    }

    /// The per-arm spawn census (the WRITTEN draw counts, module doc).
    /// Each arm is replayed against a hand-built twin stream — state
    /// equality pins the count AND the order (the prefix.rs convention).
    #[test]
    fn w4_e7_the_particle_arms_consume_the_cited_draws() {
        // Spawn_SnappingStone: 20 × (Next(3) + NewDust 6 + [flag → Next(6)]).
        // With a fixed seed the flag pattern is deterministic; replay it.
        let mut rng = UnifiedRandom::new(11);
        let mut witness = UnifiedRandom::new(11);
        let mut sounds = Vec::new();
        spawn_particle_draws(0, 50.0, 1.0, &mut rng, &mut sounds, Vec2::ZERO);
        for _ in 0..20 {
            let flag = witness.next_max(3) != 0;
            new_dust_draws(&mut witness, 0, 0);
            if flag {
                witness.next_max(6);
            }
        }
        assert_eq!(rng.peek(), witness.peek(), "Spawn_SnappingStone census");
        assert_eq!(sounds[0].style, "Item72");
        // Spawn_SnappingStoneMelee: 1 + 4×(2 + 6 + 1) = 37.
        let mut rng = UnifiedRandom::new(11);
        let mut witness = UnifiedRandom::new(11);
        let mut sounds = Vec::new();
        spawn_particle_draws(1, 50.0, 1.0, &mut rng, &mut sounds, Vec2::ZERO);
        witness.next_float_direction();
        for _ in 0..4 {
            witness.next_float_direction();
            witness.next_float();
            new_dust_draws(&mut witness, 0, 0);
            witness.next_range(-10, 6);
        }
        assert_eq!(rng.peek(), witness.peek(), "Spawn_SnappingStoneMelee = 37 draws");
        assert_eq!(sounds[0].style, "PyroclasticStone");
        // Spawn_SnappingStoneSummon: 2 + 23×(2 + 6 + 1) = 209.
        let mut rng = UnifiedRandom::new(11);
        let mut witness = UnifiedRandom::new(11);
        let mut sounds = Vec::new();
        spawn_particle_draws(2, 50.0, 1.0, &mut rng, &mut sounds, Vec2::ZERO);
        witness.next_float_direction();
        witness.next_float();
        for _ in 0..23 {
            witness.next_float();
            witness.next_float();
            new_dust_draws(&mut witness, 0, 0);
            witness.next_max(6);
        }
        assert_eq!(rng.peek(), witness.peek(), "Spawn_SnappingStoneSummon = 209 draws");
        assert_eq!(sounds[0].style, "ArmletOfRuin");
    }

    /// The PVE gate table (`:13397-13408`): three independent ifs.
    #[test]
    fn w4_e7_the_pve_gates_are_three_independent_ifs() {
        let w = air();
        let mut p = player(&w);
        assert!(pve_strike_types(&p, true, true, true).is_empty(), "no flags → nothing");
        p.acc_pyroclast = true;
        p.acc_snapping_stone = true;
        p.acc_armlet_of_ruin = true;
        // melee+ranged+tag → all three, in the C# order 1, 0, 2.
        assert_eq!(pve_strike_types(&p, true, true, true), vec![1, 0, 2]);
        // ranged only → pyroclast (melee||ranged) + snapping (ranged).
        assert_eq!(pve_strike_types(&p, false, true, false), vec![1, 0]);
        // melee only, no tag → pyroclast alone.
        assert_eq!(pve_strike_types(&p, true, false, false), vec![1]);
        // tag only → armlet alone.
        assert_eq!(pve_strike_types(&p, false, false, true), vec![2]);
    }
}
