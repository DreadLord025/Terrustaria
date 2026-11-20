//! The aiStyle-32 arm — the Skeletron Prime head (`NPC.AI()`'s
//! `else if (this.aiStyle == 32)` inline block, `NPC.cs:27626-27942`).
//!
//! W4-d26: the faithful 1:1 port replacing the v1 `npc.rs::update_prime`
//! (the EoC-cycle stand-in: the `caster_counter`/`cast_windup` spawn
//! latch instead of `ai[0]`, the missing init/despawn/hover-reset
//! `TargetClosest`s, the missing `EncourageDespawn(500)`, the invented
//! per-tick facing write and world-bottom kill). The axis arm
//! `ai_032_prime_head_family` wires it; the v1 fn stays as the
//! direct-call reference the old pins read.
//!
//! The only type on this style is `NPCID.SkeletronPrime` (127); its
//! `SetDefaults` row (`NPC.cs:10556-10572`): 80×102, aiStyle 32,
//! damage 47, defense 24, lifeMax 28000, `NPCHit4`/`NPCDeath14`,
//! noGravity, noTileCollide, knockBackResist 0, boss, npcSlots 6.
//!
//! The machine (`ai[1]` is the phase):
//! * `:27628-27629` — damage/defense reset to the `defDamage`/
//!   `defDefense` rows every tick (the port's `*_override = None`),
//! * `:27630-27633` — `ai[3] != 0` marks the Mech Queen
//!   (`NPC.mechQueen = whoAmI`); the port has no Mech Queen, the
//!   `IsMechQueenUp` rows are constant-false (the style30 module's
//!   shared anchor seam) — documented fold,
//! * `:27634` — `reflectsProjectiles = false`: no port field, skip,
//! * `:27635-27660` — the one-shot init: `ai[0] == 0 && netMode != 1`
//!   → `TargetClosest`, `ai[0] = 1`, the four `NewNPC` limbs at
//!   `((int)(x + width/2), (int)y + height/2)` — the cannon 128 seat
//!   −1, the saw 129 seat +1, the vice 130 seat −1 with `ai[3] = 150`,
//!   the laser 131 seat +1 with `ai[3] = 150`; each inherits
//!   `ai[1] = whoAmI`, `target`, `netUpdate`. The port's seam is the
//!   `wants_limbs` flag + the sim's drain (`world_sim/npcs.rs`).
//! * `:27661-27671` — the despawn check: dead player OR |ΔX| > 6000
//!   OR |ΔY| > 6000 (TOP edges) → re-target and re-check; still gone
//!   → `ai[1] = 3`,
//! * `:27672-27676` — `IsItDay()` and not already 2/3 → the rage
//!   (`ai[1] = 2`); the `SoundEngine.PlaySound(15)` is the sound skip,
//! * phase 0 (`:27677-27759`) — the hover: `ai[2]++`, at ≥ 600 reset
//!   + phase 1 + `TargetClosest` + `netUpdate`; the rotation is
//!   `velocity.X / 15f` (the Mech-Queen AngleLerp folds out); the
//!   0.1/2/0.1/8 classic bands (expert 0.03/4/0.07/9.5), the Y band
//!   compares TOP edges against `player.Y − 200/− 500`, the X band
//!   CENTRES ±100; both damp 0.98 against the motion only,
//! * phase 1 (`:27760-27856`) — the charge: defense/damage ×2,
//!   `ai[2]++`, at ≥ 400 back to hover; `rotation += direction·0.3`;
//!   the straight-at-the-player speed 2 classic / 6 expert with the
//!   > 150 ×1.05 then nine > 200..600 ×1.1 ramps; the distance floors
//!   at 1 (`num485 <= 0`),
//! * phase 2 (`:27857-27892`) — the day rage: 9999/9999, the same
//!   spin, the `10 + d/100` speed clamped 8..32,
//! * phase 3 (`:27926-27938`) — `EncourageDespawn(500)`, `vy += 0.1`,
//!   the negative-vy ×0.95 and the vx ×0.95 (NO fall cap — the
//!   Mech-Queen branch's 13 cap folds out with the queen),
//! * any other phase (`:27893-27897`) — the early `return`.
//!
//! RNG by construction: the C# block (`NPC.cs:27626-27942`) draws NO
//! `Main.rand` (the two `PlaySound(15)` calls and `NewNPC`'s
//! getGood-gated draw are sound/constant-off skips), so the step
//! takes no stream parameter — the witness pins the caller's stream
//! untouched.
//!
//! The arm does NOT integrate: the head is noGravity/noTileCollide
//! (`SetDefaults :10566-10567`), so `UpdateNPC` owns the raw
//! `position += velocity` (`NPC.cs:90498`) — the caller's tail. The
//! C# writes neither `direction` nor `spriteDirection` here (the
//! `TargetClosest` face writes stand); the v1's per-tick velocity-
//! facing write and the world-bottom `hp = 0` kill are NOT in the
//! C# arm and are dropped (the generic despawn pass owns the kill).

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The env seam — the style30/31 convention.
#[derive(Clone, Copy, Debug)]
pub struct Style32Env {
    /// `Main.expertMode` (`:27708-27714`, `:27795-27838`) — the port's
    /// `player.buffs.expert_mode` mirror (the style30 wiring reads it
    /// live).
    pub expert: bool,
    /// `Main.netMode` (`:27635`) — 0 single-player: the init gate
    /// (`!= 1`) passes.
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for `TargetClosest`'s stamp — not
    /// plumbed into `Npc::update` yet (the `update_zombie` stand-in).
    pub ticker: u64,
}

/// `NPC.AI()`'s aiStyle-32 arm (`NPC.cs:27626-27942`), one tick.
/// `players` is the target-scan slice (the port's session player is
/// slot 0). No shot is fired — the head is a contact weapon; the
/// limbs (styles 33-36) carry the attacks.
pub fn style32_step(npc: &mut Npc, world: &World, players: &[TargetCandidate], env: &Style32Env) {
    // `:27628-27629` — `damage = defDamage; defense = defDefense`.
    npc.damage_override = None;
    npc.defense_override = None;
    // `:27630-27634` — the Mech Queen mark + `reflectsProjectiles`
    // have no port fields (the documented fold above).
    // `:27635-27660` — the one-shot limb-spawn init.
    if npc.ai[0] == 0.0 && env.net_mode != 1 {
        target_closest(npc, world, players, true, 32, env.ticker);
        npc.ai[0] = 1.0;
        // The four `NewNPC`s ride the sim's drain (`wants_limbs`,
        // `world_sim/npcs.rs`'s seat table — the C#'s order cannon,
        // saw, vice, laser with the seats and the two 150 counters).
        npc.wants_limbs = true;
    }
    // `:27661-27671` — the despawn check with the re-target re-check.
    // The C# reads `Main.player[this.target]`; an unacquired target
    // (255) is an INACTIVE slot — `dead` reads false there, but the
    // port's slice has one slot: the axis arm stands the target in
    // as 0 before the step (the limb family's mapping).
    let gone = |n: &Npc| -> bool {
        let p = &players[n.target as usize];
        p.dead || (n.x - p.x).abs() > 6000.0 || (n.y - p.y).abs() > 6000.0
    };
    if gone(npc) {
        target_closest(npc, world, players, true, 32, env.ticker);
        if gone(npc) {
            npc.ai[1] = 3.0;
        }
    }
    // `:27672-27676` — the day-time rage (the PlaySound(15) skip).
    if world.day_time && npc.ai[1] != 3.0 && npc.ai[1] != 2.0 {
        npc.ai[1] = 2.0;
    }
    let p = &players[npc.target as usize];
    let (w, h) = npc.size();
    let cx = npc.x + w * 0.5;
    let cy = npc.y + h * 0.5;
    let pcx = p.x + (p.width / 2) as f32;
    let pcy = p.y + (p.height / 2) as f32;
    if npc.ai[1] == 0.0 {
        // `:27678-27685` — the 600-tick hover → the charge.
        npc.ai[2] += 1.0;
        if npc.ai[2] >= 600.0 {
            npc.ai[2] = 0.0;
            npc.ai[1] = 1.0;
            target_closest(npc, world, players, true, 32, env.ticker);
            npc.net_update = true;
        }
        // `:27686-27693` — the rotation (the queen AngleLerp folds out).
        npc.rotation = npc.vx / 15.0;
        // `:27694-27714` — the band constants (the queen's 300/350 +
        // the ±450 offset fold out with `IsMechQueenUp` false).
        let (mut num475, mut num476, mut num477, mut num478) = (0.1f32, 2.0f32, 0.1f32, 8.0f32);
        let (num479, num480) = (200.0f32, 500.0f32);
        if env.expert {
            num475 = 0.03;
            num476 = 4.0;
            num477 = 0.07;
            num478 = 9.5;
        }
        // `:27715-27737` — the Y band: TOP edges, the 0.98 damp only
        // against the motion, the one-sided caps.
        if npc.y > p.y - num479 {
            if npc.vy > 0.0 {
                npc.vy *= 0.98;
            }
            npc.vy -= num475;
            if npc.vy > num476 {
                npc.vy = num476;
            }
        } else if npc.y < p.y - num480 {
            if npc.vy < 0.0 {
                npc.vy *= 0.98;
            }
            npc.vy += num475;
            if npc.vy < -num476 {
                npc.vy = -num476;
            }
        }
        // `:27738-27759` — the X band: CENTRES ±100 (the queen's
        // num481 offset is 0).
        if cx > pcx + 100.0 {
            if npc.vx > 0.0 {
                npc.vx *= 0.98;
            }
            npc.vx -= num477;
            if npc.vx > num478 {
                npc.vx = num478;
            }
        }
        if cx < pcx - 100.0 {
            if npc.vx < 0.0 {
                npc.vx *= 0.98;
            }
            npc.vx += num477;
            if npc.vx < -num478 {
                npc.vx = -num478;
            }
        }
    } else if npc.ai[1] == 1.0 {
        // `:27761-27762` — the charge doubles the rows.
        npc.defense_override = Some(npc.defense() * 2);
        npc.damage_override = Some(npc.kind.contact_damage() * 2);
        // `:27763-27772` — the 400-tick cycle (the ai[2]==2 roar is
        // the sound skip).
        npc.ai[2] += 1.0;
        if npc.ai[2] >= 400.0 {
            npc.ai[2] = 0.0;
            npc.ai[1] = 0.0;
        }
        // `:27773-27780` — the roll (the queen AngleLerp folds out).
        npc.rotation += npc.facing as f32 * 0.3;
        // `:27781-27793` — the aim with the <= 0 distance floor.
        let num483 = pcx - cx;
        let num484 = pcy - cy;
        let mut num485 = (num483 * num483 + num484 * num484).sqrt();
        if num485 <= 0.0 {
            num485 = 1.0;
        }
        // `:27794-27838` — the speed: 2 classic; 6 expert with the
        // > 150 ×1.05 and the nine > 200..600 ×1.1 ramps (each
        // threshold is its own `if`, all kept in order).
        let mut num486 = 2.0f32;
        if env.expert {
            num486 = 6.0;
            if num485 > 150.0 {
                num486 *= 1.05;
            }
            for threshold in [200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0, 600.0] {
                if num485 > threshold {
                    num486 *= 1.1;
                }
            }
        }
        // `:27839-27845` — the queen's ×0.6/0.75 + the near-player
        // SafeNormalize clamp fold out (`IsMechQueenUp` false).
        let k = num486 / num485;
        npc.vx = num483 * k;
        npc.vy = num484 * k;
    } else if npc.ai[1] == 2.0 {
        // `:27858-27859` — the rage rows.
        npc.damage_override = Some(9999);
        npc.defense_override = Some(9999);
        // `:27860-27867` — the same roll.
        npc.rotation += npc.facing as f32 * 0.3;
        // `:27868-27891` — the 8..32 distance-ramped chase.
        let num488 = pcx - cx;
        let num489 = pcy - cy;
        let mut num490 = (num488 * num488 + num489 * num489).sqrt();
        if num490 <= 0.0 {
            num490 = 1.0;
        }
        let mut num491 = 10.0 + num490 / 100.0;
        if num491 < 8.0 {
            num491 = 8.0;
        }
        if num491 > 32.0 {
            num491 = 32.0;
        }
        let k = num491 / num490;
        npc.vx = num488 * k;
        npc.vy = num489 * k;
    } else if npc.ai[1] == 3.0 {
        // `:27926-27938` — the despawn drift (the queen's twin/
        // destroyer hand-offs + the 13 cap fold out).
        npc.encourage_despawn(500);
        npc.vy += 0.1;
        if npc.vy < 0.0 {
            npc.vy *= 0.95;
        }
        npc.vx *= 0.95;
    } else {
        // `:27893-27897` — `if (this.ai[1] != 3f) return;` — no other
        // phase writes anything (the integration is the caller's).
        return;
    }
    // No `position += velocity` here — `UpdateNPC` integrates after
    // `AI()` (`NPC.cs:90498`, noGravity/noTileCollide).
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
        let mut w = World::new("t".into(), 400, 400);
        w.day_time = false; // night — the day gate stays off
        w
    }

    fn env() -> Style32Env {
        Style32Env { expert: false, net_mode: 0, ticker: 0 }
    }

    /// The head at (2000, 1000), 80×102, phase 0 past the init.
    fn head() -> Npc {
        let mut n = Npc::new(NpcKind::SkeletronPrime, 2000.0, 1000.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 1.0; // the init already ran
        n.target = 0;
        n
    }

    /// `:27635-27660` — the init runs ONCE: `ai[0]` 0 → 1, the
    /// target acquired, the limb request armed; the second tick
    /// re-arms nothing.
    #[test]
    fn the_init_targets_and_arms_the_limbs_once() {
        let w = air();
        let p = [cand(1500.0, 1500.0)];
        let mut n = head();
        n.ai[0] = 0.0;
        n.target = 0;
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.ai[0], 1.0, "the one-shot latch (:27638)");
        assert!(n.wants_limbs, "the NewNPC quartet rides the drain");
        assert_eq!(n.target, 0, "TargetClosest ran (:27637)");
        n.wants_limbs = false;
        style32_step(&mut n, &w, &p, &env());
        assert!(!n.wants_limbs, "ai[0] == 1 never re-arms");
    }

    /// `:27661-27671` — the despawn check re-targets and RE-CHECKS:
    /// a player 6001 px away arms phase 3; exactly 6000 holds.
    #[test]
    fn the_despawn_check_is_strict_6000_and_retargets() {
        let w = air();
        let mut n = head();
        // |ΔX| = 6000 exactly: holds phase 0.
        let p = [cand(2000.0 + 6000.0, 1000.0)];
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.ai[1], 0.0, "6000 is not > 6000 (:27662)");
        // 6001: phase 3.
        let mut n2 = head();
        let p2 = [cand(2000.0 + 6001.0, 1000.0)];
        style32_step(&mut n2, &w, &p2, &env());
        assert_eq!(n2.ai[1], 3.0);
        // A dead player: phase 3.
        let mut n3 = head();
        let mut p3 = cand(1500.0, 1500.0);
        p3.dead = true;
        style32_step(&mut n3, &w, &[p3], &env());
        assert_eq!(n3.ai[1], 3.0);
    }

    /// `:27672-27676` — the day gate arms the rage only from 0/1.
    #[test]
    fn the_day_rage_arms_only_outside_2_and_3() {
        let mut w = air();
        w.day_time = true;
        let p = [cand(1500.0, 1500.0)];
        let mut n = head();
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.ai[1], 2.0);
        let mut n3 = head();
        n3.ai[1] = 3.0;
        style32_step(&mut n3, &w, &p, &env());
        assert_eq!(n3.ai[1], 3.0, "the despawn phase holds");
    }

    /// `:27678-27685` — the 600th hover tick arms the charge, resets
    /// the counter, re-targets and stamps netUpdate.
    #[test]
    fn the_hover_rolls_into_the_charge_at_600() {
        let w = air();
        let p = [cand(1500.0, 1500.0)];
        let mut n = head();
        n.ai[2] = 599.0;
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.ai[2], 0.0);
        assert!(n.net_update);
        // 598 → 599 holds the hover.
        let mut n2 = head();
        n2.ai[2] = 598.0;
        style32_step(&mut n2, &w, &p, &env());
        assert_eq!(n2.ai[1], 0.0);
        assert_eq!(n2.ai[2], 599.0);
    }

    /// `:27694-27759` — the classic hover bands: TOP-edge Y against
    /// player.Y − 200/− 500, CENTRE X ±100, the 0.98 against-motion
    /// damp, the 0.1 steps and the one-sided 2/8 caps.
    #[test]
    fn the_hover_bands_damp_and_cap() {
        let w = air();
        // Player at (2030, 1300): the head's centre x 2040 is inside
        // ±100 of pcx 2040 — no X write; y 1000 > 1300 − 200 = 1100?
        // No: 1000 < 1100 and > 1300 − 500 = 800 → the Y dead band.
        let p = [cand(2030.0, 1300.0)];
        let mut n = head();
        n.vx = 0.75;
        n.vy = 1.25;
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.vx, 0.75, "the X dead band holds");
        assert_eq!(n.vy, 1.25, "the Y dead band holds");
        assert_eq!(n.rotation, 0.75 / 15.0, "vx/15 (:27692)");
        // Below the band (y 1101 > 1100): rise; a falling vy damps.
        let mut n2 = head();
        n2.y = 1101.0;
        n2.vy = 2.0;
        style32_step(&mut n2, &w, &p, &env());
        assert!((n2.vy - (2.0 * 0.98 - 0.1)).abs() < 1e-5, "{}", n2.vy);
        // Above the band (y 799 < 800): sink, the −2 cap.
        let mut n3 = head();
        n3.y = 799.0;
        n3.vy = -4.0;
        style32_step(&mut n3, &w, &p, &env());
        assert_eq!(n3.vy, -2.0, "the one-sided −2 cap (:27733)");
        // Right of pcx + 100: pushed left at 0.1, the 8 cap.
        let mut n4 = head();
        n4.x = 2200.0; // centre 2240 > 2040 + 100
        n4.vx = 14.0;
        style32_step(&mut n4, &w, &p, &env());
        assert_eq!(n4.vx, 8.0, "the 8 cap (:27745-27748)");
    }

    /// `:27761-27845` — the charge: ×2 rows, the 400-tick reset, the
    /// direction roll, the classic speed 2 straight at the player.
    #[test]
    fn the_charge_doubles_the_rows_and_chases_at_two() {
        let w = air();
        let p = [cand(2300.0, 1000.0)]; // straight right of the head
        let mut n = head();
        n.ai[1] = 1.0;
        n.ai[2] = 399.0;
        let base_def = n.defense();
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.ai[1], 0.0, "the 400th tick resets (:27768)");
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.defense_override, Some(base_def * 2));
        assert_eq!(n.damage_override, Some(NpcKind::SkeletronPrime.contact_damage() * 2));
        // The aim: player centre (2310, 1021), head centre (2040,
        // 1051) — delta (270, −30), speed 2 along it.
        let d = (270.0f32 * 270.0 + 30.0 * 30.0).sqrt();
        assert!((n.vx - 270.0 * 2.0 / d).abs() < 1e-4, "{}", n.vx);
        assert!((n.vy - (-30.0) * 2.0 / d).abs() < 1e-4, "{}", n.vy);
        assert_eq!(n.rotation, 0.3, "direction +1 rolls +0.3 (:27779)");
    }

    /// `:27795-27838` — the expert ramps: 6 base, > 150 ×1.05, then
    /// ×1.1 per crossed 200..600 threshold. At d 650 all ten fire.
    #[test]
    fn the_expert_charge_ramps_multiply_in_order() {
        let w = air();
        // Player centre exactly 650 right of the head's centre.
        let p = [cand(2040.0 + 650.0 - 10.0, 1051.0 - 21.0)];
        let mut n = head();
        n.ai[1] = 1.0;
        let e = Style32Env { expert: true, net_mode: 0, ticker: 0 };
        style32_step(&mut n, &w, &p, &e);
        let mut want = 6.0f32 * 1.05;
        for _ in 0..9 {
            want *= 1.1;
        }
        assert!((n.vx - want).abs() < 1e-2, "{} vs {}", n.vx, want);
        assert!(n.vy.abs() < 1e-3, "the horizontal aim");
    }

    /// `:27858-27891` — the rage: 9999 rows, the 10 + d/100 speed
    /// clamped 8..32.
    #[test]
    fn the_rage_chases_at_the_clamped_ramp() {
        let w = air();
        let p = [cand(2300.0, 1000.0)];
        let mut n = head();
        n.ai[1] = 2.0;
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.damage_override, Some(9999));
        assert_eq!(n.defense_override, Some(9999));
        let d = (270.0f32 * 270.0 + 30.0 * 30.0).sqrt();
        let speed = (10.0 + d / 100.0).clamp(8.0, 32.0);
        let sp = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((sp - speed).abs() < 1e-3, "{sp} vs {speed}");
        // Far away: the 32 cap.
        let mut n2 = head();
        n2.ai[1] = 2.0;
        let p2 = [cand(5000.0, 1000.0)];
        style32_step(&mut n2, &w, &p2, &env());
        let sp2 = (n2.vx * n2.vx + n2.vy * n2.vy).sqrt();
        assert!((sp2 - 32.0).abs() < 1e-3, "the 32 cap ({sp2})");
    }

    /// `:27926-27938` — the despawn drift: EncourageDespawn(500),
    /// vy += 0.1 with the negative-only ×0.95, vx ×0.95, NO fall cap.
    #[test]
    fn the_despawn_drift_encourages_500_and_damps() {
        let w = air();
        let p = [cand(1500.0, 1500.0)];
        let mut n = head();
        n.ai[1] = 3.0;
        n.vx = 10.0;
        n.vy = -2.0;
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.despawn_timer, 500, "EncourageDespawn(500) (:27928)");
        assert!((n.vx - 9.5).abs() < 1e-5, "the vx damp");
        assert!((n.vy - ((-2.0 + 0.1) * 0.95)).abs() < 1e-5, "the negative-vy damp");
        // A positive vy does NOT damp (and no 13 cap — the queen fold).
        let mut n2 = head();
        n2.ai[1] = 3.0;
        n2.vy = 20.0;
        style32_step(&mut n2, &w, &p, &env());
        assert_eq!(n2.vy, 20.1, "no cap outside the queen branch");
    }

    /// `:27893-27897` — an unknown phase returns with no writes.
    #[test]
    fn an_unknown_phase_writes_nothing() {
        let w = air();
        let p = [cand(1500.0, 1500.0)];
        let mut n = head();
        n.ai[1] = 7.0;
        n.vx = 3.0;
        n.vy = 4.0;
        style32_step(&mut n, &w, &p, &env());
        assert_eq!(n.vx, 3.0);
        assert_eq!(n.vy, 4.0);
        assert_eq!(n.rotation, 0.0);
    }

    /// RNG by construction: the C# block has no `Main.rand` — the
    /// step takes no stream, so two identical inputs stay identical.
    #[test]
    fn the_arm_is_deterministic_without_an_rng() {
        let w = air();
        let p = [cand(1500.0, 1500.0)];
        let mut a = head();
        let mut b = head();
        for _ in 0..700 {
            style32_step(&mut a, &w, &p, &env());
            style32_step(&mut b, &w, &p, &env());
            a.x += a.vx;
            a.y += a.vy;
            b.x += b.vx;
            b.y += b.vy;
        }
        assert_eq!(a.ai, b.ai);
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(a.wants_limbs, b.wants_limbs);
    }
}
