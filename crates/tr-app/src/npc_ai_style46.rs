//! The aiStyle-46 family — `NPC.AI()`'s `else if (this.aiStyle == 46)`
//! arm (`NPC.cs:31129-31346`), S1-D-3 (aiStyle-46 round). The Golem's
//! ATTACHED head: the ONLY style-46 user is type 246 `NPCID.GolemHead`
//! (`SetDefaults` case 246, `NPC.cs:12179-12192`: `noGravity`, 70×70,
//! aiStyle 46 `:12184`, dmg 64, def 20, 25000 HP, `HitSound` NPCHit4,
//! `DeathSound = null`, knockBackResist 0, `alpha = 255` — the silent
//! death). The detached head (249) is aiStyle 48, the fists (247/248)
//! aiStyle 47 — separate arms, separate rounds.
//!
//! The machine: the balance factor `num675` (`GetMyBalance` solo 1f,
//! `NPC.cs:18483-18491`; ftw +3; the temple/jungle/surface enrage ×2,
//! `:31131-31139`), the hard dock tether to the body's centre
//! `(-3, -57) * scale` (snap inside 100, else normalize to 100 with the
//! `velocity.X * 0.1` tilt, `:31146-31164`), the 26-tick alpha fade that
//! pins `ai[1] = 30` (`:31165-31173`), the phase-0 fireball cycle (the
//! 300 cadence with the balance-scaled window adds, the 8-speed 258 at
//! dmg 18, `:31174-31206`), the phase-1 machine (the every-tick
//! `TargetClosest`, the three-way `localAI[1]` head turn with the ±40
//! muzzle, the `num688 = (num675+3)/4` cadence with the 40%/20% DOUBLE
//! life tiers, the dmg-24 fireball, `:31207-31260`), the eye-beam volley
//! (the `num688` cadence with the 1/3, 1/4, 1/5 INTEGER life tiers, the
//! `!CanHit` +4, the `60 + Main.rand.Next(600)` gate, the in-band
//! two-shot ±18 / side single ±30 259s at dmg 28 with the 3-tick lead
//! and `timeLeft = 300`, `:31261-31336`) and the tail phase gate
//! (`life < lifeMax / 2`, INTEGER, `:31338-31345`).
//!
//! ## RNG order (by construction)
//!
//! The arm has EXACTLY ONE `Main.rand` call site: `Main.rand.Next(600)`
//! inside the beam gate `this.ai[2] > (float)(60 + Main.rand.Next(600))`
//! (`NPC.cs:31278`). The comparison is reached on EVERY phase-1
//! (`ai[0] == 1f`) tick and C# evaluates the right-hand side
//! unconditionally — one `Next(600)` draw per phase-1 tick, whether or
//! not the volley fires. Phase 0 draws NOTHING. `TargetClosest`
//! (`NPC.cs:77652-77781`) draws nothing; the fireball/beam
//! `Projectile.NewProjectile` calls draw inside the projectile's own
//! `SetDefaults`/AI, which the port's pool owns (the `wants_hostile_shot`
//! seam, the S1-D-3 convention) — not this arm's stream. Witness:
//! `s1_d_3_s46_the_arm_draws_one_next_600_per_phase1_tick`.
//!
//! ## Host-path folds (documented in ANALYSIS "The Golem head's aiStyle-46 machine")
//!
//! - `GetMyBalance()` (`:18483-18491`): the port has one session player
//!   (`statsAreScaledForThisManyPlayers <= 1`) → the 1f base; the
//!   multiplayer scaling table stays unported (the fists' precedent).
//! - `Main.getGoodWorld` has no runtime flag in the port yet (the seed
//!   only rides worldgen) — the wiring passes false; the +3 row is
//!   ported and witnessed (the S1-D-3-s9 precedent).
//! - `Main.player[target].ZoneLihzhardTemple`/`ZoneJungle` have no
//!   `Player` fields in the port — the wiring passes false/false, so the
//!   enrage ×2 rides whenever the player is out of the (untracked)
//!   zones; the flags struct keeps the gate complete for the wiring.
//! - `this.noTileCollide = true` (`:31140`): the port models it as the
//!   raw `position += velocity` integration at the tail (the
//!   `integrate_no_tile_collide` shape); `Npc` has no runtime flag.
//! - `StrikeNPCNoInteraction(9999, 0f, 0)` (`:31143`): the
//!   no-interaction kill folds to `hp = 0` (the `world_sim/npcs.rs`
//!   golemBoss-gone precedent — no gore/loot/kill credit).
//! - `Main.netMode != 1` (`:31201`/`:31256`/`:31304`/`:31330`): the port
//!   is single-player-hostile — the shots always request.
//! - `GetSpawnSource_ForProjectile()`: unported spawn-source tracking,
//!   the standing fold of every `wants_hostile_shot` arm.
//! - The beams' `timeLeft = 300` (`:31307`/`:31333`) rides the sim's
//!   timed-spawn resolution (`world_sim/npcs.rs`'s DESTROYER_LASER list
//!   at the shot drain) — GOLEM_EYE_BEAM joins that list at the MERGE
//!   STEP (the style-48 free head's `:31568` beams carry the same
//!   override, so the row serves both rounds).
//!
//! ## UNWIRED (the 8ac80b4e / aiStyle-42 precedent)
//!
//! The kind arm (`update_golem`'s `GolemHead` branch, `npc.rs`), the
//! `46 =>` style-axis claim and the FindFrame case-246 `localAI[0]` read
//! are the S1-D-3 golem family's MERGE STEP: the sibling aiStyle-45/47/48
//! rounds (body/fists/free head) are rewriting the same `update_golem`
//! arms and the findframe rounds the same case — this module lands the
//! faithful machine without touching those contested regions. Until the
//! wiring, the live head keeps its documented stand-in (the accel tether,
//! the raw-delta fireballs, the flat-300 beam gate — ROADMAP S1-D-3).

#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the S1-D-3 merge step.

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// The world/player flags the arm's balance block reads
/// (`NPC.cs:31132-31139`). See the module header for the parked wiring.
#[derive(Clone, Copy, Debug, Default)]
pub struct Style46Flags {
    /// `Main.getGoodWorld` (`:31132`).
    pub get_good_world: bool,
    /// `Main.player[this.target].ZoneLihzhardTemple` (`:31136`).
    pub player_zone_temple: bool,
    /// `Main.player[this.target].ZoneJungle` (`:31136`).
    pub player_zone_jungle: bool,
}

/// The balance factor `num675` (`NPC.cs:31131-31139`): `GetMyBalance()`
/// (solo 1f, `:18485-18488`), `+3f` under `getGoodWorld` (`:31134`),
/// then `*2f` when the target is out of BOTH the temple and the jungle
/// OR above the surface line — the C# compares
/// `(double)player.Center.Y < Main.worldSurface * 16.0` (`:31136`), a
/// DOUBLE comparison against the f64 surface row.
pub fn balance_factor(flags: Style46Flags, player_center_y: f32, world_surface: f64) -> f32 {
    // `:31131` — the solo GetMyBalance fold.
    let mut num675 = 1.0f32;
    // `:31132-31135`.
    if flags.get_good_world {
        num675 += 3.0;
    }
    // `:31136-31139`.
    if (!flags.player_zone_temple && !flags.player_zone_jungle)
        || (player_center_y as f64) < world_surface * 16.0
    {
        num675 *= 2.0;
    }
    num675
}

/// `NPC.AI()`'s aiStyle-46 arm (`NPC.cs:31129-31346`) — the attached
/// Golem head's whole tick. `world.golem_present/golem_x/golem_y` carry
/// `NPC.golemBoss` + `Main.npc[golemBoss].Center` (the sim's
/// write-through, `NPC.cs:19507`); the head's tether does NOT fold the
/// body's velocity (the fists' `golem_vx` term is aiStyle-47 only,
/// `World.golem_vx`'s doc).
pub fn golem_head_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    flags: Style46Flags,
    rng: &mut UnifiedRandom,
) {
    let (pcx, pcy) = player.center();
    // `:31131-31139` — the balance factor.
    let num675 = balance_factor(flags, pcy, world.world_surface);
    // `:31140` — `noTileCollide = true`: the raw integration at the tail.
    // `:31141-31145` — the body-gone kill: StrikeNPCNoInteraction(9999,
    // 0f, 0) folds to hp = 0, and the C# RETURNS before any velocity or
    // counter write.
    if !world.golem_present {
        npc.hp = 0;
        return;
    }
    let (w, _h) = npc.size();
    let (cx, cy) = npc.center();
    let scale = npc.scale;
    // `:31146-31164` — the dock tether. num676 = 100f; the dock is the
    // body's centre offset (-3, -57) * scale (the X offset subtracted
    // AFTER the delta, `:31150` — same result, kept in the C#'s order).
    let num676 = 100.0f32;
    let mut num677 = world.golem_x - cx;
    let num678 = world.golem_y - cy - 57.0 * scale;
    num677 -= 3.0 * scale;
    let mut num679 = (num677 * num677 + num678 * num678).sqrt();
    if num679 < num676 {
        // `:31152-31157` — inside the 100 ring: the RAW delta velocity
        // (a hard snap, not an approach), rotation zeroed.
        npc.rotation = 0.0;
        npc.vx = num677;
        npc.vy = num678;
    } else {
        // `:31158-31164` — outside: normalize to the 100 speed, the
        // tilt rides the WRITTEN velocity.X * 0.1f.
        num679 = num676 / num679;
        npc.vx = num677 * num679;
        npc.vy = num678 * num679;
        npc.rotation = npc.vx * 0.1;
    }
    // `:31165-31173` — the spawn fade: alpha -= 10 floored at 0 (the
    // u8 saturating_sub is the C#'s `if (alpha < 0) alpha = 0`), and
    // ai[1] pinned at 30 every fade tick.
    if npc.alpha > 0 {
        npc.alpha = npc.alpha.saturating_sub(10);
        npc.ai[1] = 30.0;
    }
    // The session-player candidate for the arm's TargetClosest calls
    // (`NPC.cs:77652-77781` via npc_target) — the zombie wiring's stock
    // aggro/noAggro/tank fields (S1-D-3p: the stealth/tank systems are
    // unported). The ticker is 0 until the caller migration (the
    // zombie precedent).
    let cand = TargetCandidate {
        active: true,
        dead: player.dead,
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    if npc.ai[0] == 0.0 {
        // `:31174-31206` — phase 0: the plain ++ then the balance-scaled
        // window adds. num680 = 300; the window is the first/last 20
        // (`ai[1] < 20f || ai[1] > (float)(num680 - 20)`, both STRICT).
        npc.ai[1] += 1.0;
        let num680 = 300;
        if npc.ai[1] < 20.0 || npc.ai[1] > (num680 - 20) as f32 {
            // `:31180-31181` — 2f * (num675 - 1f) / 3f, mouth open.
            npc.ai[1] += 2.0 * (num675 - 1.0) / 3.0;
            npc.local_ai[0] = 1.0;
        } else {
            // `:31185-31186` — 1f * (num675 - 1f) / 2f, mouth closed.
            npc.ai[1] += 1.0 * (num675 - 1.0) / 2.0;
            npc.local_ai[0] = 0.0;
        }
        if npc.ai[1] >= num680 as f32 {
            // `:31188-31205` — the fireball: TargetClosest FIRST, the
            // counter reset, then the 8-speed aim from Center + (0, 10).
            target_closest(npc, world, &[cand], true, 46, 0);
            npc.ai[1] = 0.0;
            let sx = cx;
            let sy = cy + 10.0 * scale;
            let num681 = 8.0f32;
            let mut num682 = pcx - sx;
            let mut num683 = pcy - sy;
            let num684 = num681 / (num682 * num682 + num683 * num683).sqrt();
            num682 *= num684;
            num683 *= num684;
            // `:31199-31203` — 258 (GolemFireball), dmg 18, knockback 0.
            npc.wants_hostile_shot.push((
                crate::projectile::GOLEM_FIREBALL,
                sx,
                sy,
                num682,
                num683,
                18,
            ));
        }
    } else if npc.ai[0] == 1.0 {
        // `:31207-31260` — phase 1: TargetClosest EVERY tick, then the
        // three-way head turn about Center.X ± width (the FULL width,
        // not half — `:31211`/`:31216`) with the ±40 muzzle offset.
        target_closest(npc, world, &[cand], true, 46, 0);
        let mut sx84 = cx;
        let sy84 = cy + 10.0 * scale;
        if pcx < cx - w {
            npc.local_ai[1] = -1.0;
            sx84 -= 40.0 * scale;
        } else if pcx > cx + w {
            npc.local_ai[1] = 1.0;
            sx84 += 40.0 * scale;
        } else {
            npc.local_ai[1] = 0.0;
        }
        // The FindFrame/render mirror (`NPC.cs:72953-72959` reads
        // localAI[1]; the port's case-246 arm reads head_turn).
        npc.head_turn = npc.local_ai[1] as i32;
        // `:31225-31234` — num688 = (num675 + 3f) / 4f; the fireball
        // cadence doubles/triples under the 40%/20% life tiers — DOUBLE
        // comparisons (`(double)life < (double)lifeMax * 0.4`).
        let num688 = (num675 + 3.0) / 4.0;
        npc.ai[1] += num688;
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.4 {
            npc.ai[1] += num688;
        }
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.2 {
            npc.ai[1] += num688;
        }
        // `:31235-31243` — the mouth window on the post-add counter.
        let num689 = 300;
        if npc.ai[1] < 20.0 || npc.ai[1] > (num689 - 20) as f32 {
            npc.local_ai[0] = 1.0;
        } else {
            npc.local_ai[0] = 0.0;
        }
        if npc.ai[1] >= num689 as f32 {
            // `:31244-31259` — the second TargetClosest of the fire
            // tick, the reset, the 8-speed 258 at dmg 24 from the
            // turn-offset muzzle (vector84 computed at the phase head).
            target_closest(npc, world, &[cand], true, 46, 0);
            npc.ai[1] = 0.0;
            let num690 = 8.0f32;
            let mut num691 = pcx - sx84;
            let mut num692 = pcy - sy84;
            let num693 = num690 / (num691 * num691 + num692 * num692).sqrt();
            num691 *= num693;
            num692 *= num693;
            npc.wants_hostile_shot.push((
                crate::projectile::GOLEM_FIREBALL,
                sx84,
                sy84,
                num691,
                num692,
                24,
            ));
        }
        // `:31261-31277` — the beam cadence: num688 per tick, +num688
        // at each of the life < lifeMax/3, /4, /5 tiers — INTEGER
        // divisions (`this.life < this.lifeMax / 3`, int/int), NOT the
        // fireball's double tiers; +4f while the 1×1 centre walk
        // cannot hit the player (`Collision.CanHit`, `:31274`).
        npc.ai[2] += num688;
        if npc.hp < npc.max_hp / 3 {
            npc.ai[2] += num688;
        }
        if npc.hp < npc.max_hp / 4 {
            npc.ai[2] += num688;
        }
        if npc.hp < npc.max_hp / 5 {
            npc.ai[2] += num688;
        }
        if !crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0) {
            npc.ai[2] += 4.0;
        }
        // `:31278` — the gate: ai[2] > (float)(60 + Main.rand.Next(600)).
        // The draw happens EVERY phase-1 tick (the C# evaluates the
        // right-hand side unconditionally) — the arm's ONLY rand call.
        if npc.ai[2] > (60 + rng.next_max(600)) as f32 {
            npc.ai[2] = 0.0;
            // `:31281-31282` — 259 (GolemEyeBeam), dmg 28.
            if npc.local_ai[1] == 0.0 {
                // `:31283-31310` — in-band: TWO shots from ±18 * scale
                // at the 11 speed, the 3-tick lead, timeLeft = 300.
                for num699 in 0..2 {
                    let mut sx = cx;
                    let sy = cy - 22.0 * scale;
                    if num699 == 0 {
                        sx -= 18.0 * scale;
                    } else {
                        sx += 18.0 * scale;
                    }
                    let num700 = 11.0f32;
                    let mut num701 = pcx - sx;
                    let mut num702 = pcy - sy;
                    let num703 = num700 / (num701 * num701 + num702 * num702).sqrt();
                    num701 *= num703;
                    num702 *= num703;
                    npc.wants_hostile_shot.push((
                        crate::projectile::GOLEM_EYE_BEAM,
                        sx + num701 * 3.0,
                        sy + num702 * 3.0,
                        num701,
                        num702,
                        28,
                    ));
                }
            } else if npc.local_ai[1] != 0.0 {
                // `:31311-31335` — a side turn: ONE shot from ±30 *
                // scale at the 12 speed, the same lead + timeLeft.
                let mut sx = cx;
                let sy = cy - 22.0 * scale;
                if npc.local_ai[1] == -1.0 {
                    sx -= 30.0 * scale;
                } else if npc.local_ai[1] == 1.0 {
                    sx += 30.0 * scale;
                }
                let num705 = 12.0f32;
                let mut num706 = pcx - sx;
                let mut num707 = pcy - sy;
                let num708 = num705 / (num706 * num706 + num707 * num707).sqrt();
                num706 *= num708;
                num707 *= num708;
                npc.wants_hostile_shot.push((
                    crate::projectile::GOLEM_EYE_BEAM,
                    sx + num706 * 3.0,
                    sy + num707 * 3.0,
                    num706,
                    num707,
                    28,
                ));
            }
        }
    }
    // `:31338-31345` — the tail phase gate: life < lifeMax / 2, an
    // INTEGER division (25000/2 is exact, but the odd-max_hp scaling
    // rows floor — keep int/int).
    if npc.hp < npc.max_hp / 2 {
        npc.ai[0] = 1.0;
    } else {
        npc.ai[0] = 0.0;
    }
    // The generic `UpdateNPC` direction → spriteDirection mirror (the
    // other arms' convention); the arm itself never writes direction —
    // the TargetClosest calls own the facing.
    npc.sprite_dir = npc.facing;
    // `noTileCollide` integration (`NPC.Update`'s position += velocity,
    // the `integrate_no_tile_collide` shape) + the port's below-world
    // despawn guard (the golem family's standing convention).
    npc.x += npc.vx;
    npc.y += npc.vy;
    if npc.y > world.height as f32 * TILE_PX + 400.0 {
        npc.hp = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// The body docked at (1600, 1600) in an open air world; the head
    /// starts ON the dock (centre (1597, 1543)) so the tether writes
    /// zero velocity and the counters are the only motion.
    fn docked() -> (World, Player, Npc) {
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 50.0;
        w.golem_present = true;
        w.golem_x = 1600.0;
        w.golem_y = 1600.0;
        let mut player = Player::spawn(&w);
        // In-band (|pcx - 1597| <= 70) and below the surface line
        // (800): with the stock false zone flags the enrage x2 rides.
        player.x = 1590.0;
        player.y = 1600.0;
        let mut head = Npc::new(NpcKind::GolemHead, 1597.0 - 35.0, 1543.0 - 35.0, -1, 0.0);
        head.hp = head.max_hp;
        head.alpha = 0;
        (w, player, head)
    }

    fn flags() -> Style46Flags {
        Style46Flags::default()
    }

    /// `NPC.cs:31146-31157` — inside the 100 ring the tether writes the
    /// RAW delta (a snap, not the old stand-in's accel toward it) and
    /// zeroes the rotation; the dock is the body centre + (-3, -57) *
    /// scale.
    #[test]
    fn s1_d_3_s46_the_dock_snap_writes_the_raw_delta() {
        let (w, player, mut head) = docked();
        // Displace the head 30 right / 20 down from the dock.
        head.x += 30.0;
        head.y += 20.0;
        head.rotation = 0.5;
        let mut rng = UnifiedRandom::new(1);
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.vx, -30.0, "the raw delta, not an approach");
        assert_eq!(head.vy, -20.0);
        assert_eq!(head.rotation, 0.0, "the snap zeroes the tilt");
        // The integration lands the head exactly on the dock.
        assert_eq!(head.x + 35.0, 1597.0);
        assert_eq!(head.y + 35.0, 1543.0);
        // The scale terms: at scale 2 the dock is (-6, -114).
        let (w2, player2, mut h2) = docked();
        h2.scale = 2.0;
        h2.x = 1600.0 - 6.0 - 35.0;
        h2.y = 1600.0 - 114.0 - 35.0;
        golem_head_step(&mut h2, &w2, &player2, flags(), &mut rng);
        assert_eq!(h2.vx, 0.0, "the scaled dock holds");
        assert_eq!(h2.vy, 0.0);
    }

    /// `NPC.cs:31158-31164` — outside the ring the velocity normalizes
    /// to exactly 100 and the rotation is velocity.X * 0.1f.
    #[test]
    fn s1_d_3_s46_the_far_tether_caps_at_100_and_tilts() {
        let (w, player, mut head) = docked();
        head.x += 300.0;
        head.y += 400.0; // delta (-300, -400), length 500
        let mut rng = UnifiedRandom::new(1);
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.vx, -60.0, "-300 * (100/500)");
        assert_eq!(head.vy, -80.0);
        assert_eq!(head.rotation, -6.0, "velocity.X * 0.1");
    }

    /// `NPC.cs:31165-31173` + SetDefaults `:12191` — the 255 alpha
    /// fades by 10 a tick (26 ticks to zero) and pins ai[1] = 30 each
    /// fade tick BEFORE the phase machine adds to it.
    #[test]
    fn s1_d_3_s46_the_alpha_fade_holds_the_cadence_at_30() {
        let (w, player, mut head) = docked();
        head.alpha = 255;
        let mut rng = UnifiedRandom::new(1);
        for t in 1..=26u16 {
            golem_head_step(&mut head, &w, &player, flags(), &mut rng);
            let want = 255u8.saturating_sub((t * 10).min(255) as u8);
            assert_eq!(head.alpha, want, "the fade at tick {t}");
            // Phase 0 with the enrage x2 (num675 = 2): 30 + 1 + the
            // mid-window 0.5 = 31.5 — the pin resets every fade tick.
            assert_eq!(head.ai[1], 31.5, "the fade pin + the phase add");
        }
        assert_eq!(head.alpha, 0);
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[1], 33.0, "the pin lifts after the fade");
    }

    /// `NPC.cs:31141-31145` — golemBoss < 0: the 9999 no-interaction
    /// strike (the hp = 0 fold) and the RETURN before any write.
    #[test]
    fn s1_d_3_s46_the_body_gone_arm_kills_without_moving() {
        let (mut w, player, mut head) = docked();
        w.golem_present = false;
        head.x += 50.0;
        head.vx = 3.0;
        head.ai[1] = 7.0;
        let mut rng = UnifiedRandom::new(1);
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.hp, 0, "the silent death");
        assert_eq!(head.vx, 3.0, "no velocity write before the return");
        assert_eq!(head.ai[1], 7.0, "no counter write");
        assert_eq!(head.x, 1597.0 - 35.0 + 50.0, "no integration");
    }

    /// `NPC.cs:31338-31345` — the tail gate is life < lifeMax / 2
    /// (INTEGER): at exactly half the head stays phase 0.
    #[test]
    fn s1_d_3_s46_the_phase_gate_is_integer_half_life() {
        let (w, player, mut head) = docked();
        head.hp = head.max_hp / 2;
        let mut rng = UnifiedRandom::new(1);
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[0], 0.0, "the strict < at the half");
        head.hp = head.max_hp / 2 - 1;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[0], 1.0);
        // And back up: the gate is a latch-free two-way write.
        head.hp = head.max_hp;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[0], 0.0);
    }

    /// `NPC.cs:31188-31204` — the phase-0 fireball: 258 at dmg 18, the
    /// 8-speed NORMALIZED aim (the old stand-in passed the raw delta),
    /// the spawn at Center + (0, 10 * scale), the counter reset.
    #[test]
    fn s1_d_3_s46_the_phase0_fireball_is_normalized_to_8() {
        let (w, player, mut head) = docked();
        let mut rng = UnifiedRandom::new(1);
        let mut fired = None;
        for _ in 0..400 {
            let (cx, cy) = head.center();
            golem_head_step(&mut head, &w, &player, flags(), &mut rng);
            if let Some(shot) = head.wants_hostile_shot.pop() {
                fired = Some((shot, cx, cy));
                break;
            }
        }
        let ((kind, sx, sy, vx, vy, dmg), cx, cy) = fired.expect("the fireball fires");
        assert_eq!(kind, crate::projectile::GOLEM_FIREBALL);
        assert_eq!(dmg, 18, "the phase-0 damage");
        assert_eq!(sx, cx, "the muzzle x");
        assert_eq!(sy, cy + 10.0, "the muzzle y (:31192)");
        let speed = (vx * vx + vy * vy).sqrt();
        assert!((speed - 8.0).abs() < 1e-3, "the 8 speed ({speed})");
        assert_eq!(head.ai[1], 0.0, "the reset");
        assert_eq!(head.target, 0, "TargetClosest ran (:31190)");
    }

    /// `NPC.cs:31207-31259` — phase 1: the three-way turn about
    /// Center.X ± the FULL width, the ±40 muzzle, the dmg-24 fireball.
    #[test]
    fn s1_d_3_s46_the_phase1_fireball_takes_the_side_offset() {
        let (w, mut player, mut head) = docked();
        head.hp = head.max_hp / 3; // phase 1 via the tail gate
        head.ai[0] = 1.0;
        // The player far RIGHT: pcx > cx + 70.
        player.x = 1900.0;
        let mut rng = UnifiedRandom::new(1);
        let mut fired = None;
        for _ in 0..400 {
            let (cx, cy) = head.center();
            golem_head_step(&mut head, &w, &player, flags(), &mut rng);
            assert_eq!(head.local_ai[1], 1.0, "the right turn every tick");
            assert_eq!(head.head_turn, 1, "the FindFrame mirror");
            while let Some(shot) = head.wants_hostile_shot.pop() {
                if shot.0 == crate::projectile::GOLEM_FIREBALL {
                    fired = Some((shot, cx, cy));
                }
            }
            if fired.is_some() {
                break;
            }
        }
        let ((_, sx, sy, vx, vy, dmg), cx, cy) = fired.expect("the phase-1 fireball");
        assert_eq!(dmg, 24);
        assert_eq!(sx, cx + 40.0, "the +40 side muzzle (:31219)");
        assert_eq!(sy, cy + 10.0);
        let speed = (vx * vx + vy * vy).sqrt();
        assert!((speed - 8.0).abs() < 1e-3, "the 8 speed");
        // The left mirror.
        let (w2, mut player2, mut h2) = docked();
        h2.hp = h2.max_hp / 3;
        h2.ai[0] = 1.0;
        player2.x = 1200.0;
        golem_head_step(&mut h2, &w2, &player2, flags(), &mut rng);
        assert_eq!(h2.local_ai[1], -1.0);
        assert_eq!(h2.head_turn, -1);
    }

    /// `NPC.cs:31178-31186` / `:31236-31243` — localAI[0] (the mouth
    /// flag FindFrame case 246 reads, `:72934`/`:72945`) is written by
    /// the AI on the first/last 20 of the 300 cadence, both phases.
    #[test]
    fn s1_d_3_s46_the_mouth_flag_rides_the_window() {
        let (w, player, mut head) = docked();
        let mut rng = UnifiedRandom::new(1);
        // Phase 0: ai[1] = 5 -> 6 < 20 -> the window add, mouth open.
        // (FindFrame case 246 reads this flag — the read-side swap is the
        // merge step; the flag itself is the AI's write, pinned here.)
        head.ai[1] = 5.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.local_ai[0], 1.0);
        // Mid-window: closed.
        head.ai[1] = 150.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.local_ai[0], 0.0);
        // The last-20 window: 295 -> 296 (+2/3) > 280 -> open, no fire.
        head.ai[1] = 295.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.local_ai[0], 1.0);
        assert!(head.ai[1] < 300.0, "296.67 stays under the fire gate");
        // Phase 1 writes the same flag (num688 = 1.25 with num675 = 2;
        // hp 8333 < 10000 takes the 0.4 DOUBLE tier — two adds a tick).
        head.ai[0] = 1.0;
        head.hp = head.max_hp / 3;
        head.ai[1] = 150.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[1], 152.5);
        assert_eq!(head.local_ai[0], 0.0);
        head.ai[1] = 295.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.local_ai[0], 1.0, "297.5 > 280");
    }

    /// `NPC.cs:31261-31273` — the beam tiers are INTEGER divisions:
    /// 25000/3 = 8333, so hp == 8333 takes NO tier add (the float
    /// 8333.33 comparison would); hp == 8332 takes exactly one.
    #[test]
    fn s1_d_3_s46_the_beam_life_tiers_are_integer_divisions() {
        let (w, player, mut head) = docked();
        head.ai[0] = 1.0;
        head.hp = head.max_hp / 3; // 8333
        head.ai[2] = 0.0;
        let mut rng = UnifiedRandom::new(1);
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        // num688 = (2+3)/4 = 1.25; no tier (8333 !< 8333, !< 6250,
        // !< 5000); the sight is clear. The gate cannot fire at 1.25.
        assert_eq!(head.ai[2], 1.25, "the integer /3 boundary");
        head.hp = head.max_hp / 3 - 1; // 8332 < 8333
        head.ai[2] = 0.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[2], 2.5, "one tier add");
        // The /4 and /5 boundaries: hp 6250 -> one tier, 6249 -> two.
        head.hp = head.max_hp / 4;
        head.ai[2] = 0.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[2], 2.5, "6250 !< 6250");
        head.hp = head.max_hp / 4 - 1;
        head.ai[2] = 0.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[2], 3.75);
        head.hp = head.max_hp / 5 - 1; // 4999: all three tiers
        head.ai[2] = 0.0;
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[2], 5.0);
    }

    /// `NPC.cs:31274-31277` — the blocked 1×1 centre walk adds 4f to
    /// the beam cadence.
    #[test]
    fn s1_d_3_s46_the_beam_gate_adds_four_when_blocked() {
        let (mut w, mut player, mut head) = docked();
        // The player to the right, a solid column between the centres.
        player.x = 1900.0;
        for y in 90..105 {
            let i = w.index(105, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        head.ai[0] = 1.0;
        head.hp = head.max_hp / 3;
        head.ai[2] = 0.0;
        let mut rng = UnifiedRandom::new(1);
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[2], 1.25 + 4.0, "num688 + the blocked +4");
    }

    /// `NPC.cs:31278-31335` — the volley: ai[2] preset past the max
    /// threshold (659) fires regardless of the draw; in-band TWO 259s
    /// at dmg 28 / speed 11 from ±18 with the 3-tick lead; a side turn
    /// ONE at speed 12 from ±30.
    #[test]
    fn s1_d_3_s46_the_beam_volley_two_shot_in_band() {
        let (w, player, mut head) = docked();
        head.ai[0] = 1.0;
        head.hp = head.max_hp / 3;
        head.ai[2] = 700.0; // > 60 + 599 for any draw
        let mut rng = UnifiedRandom::new(1);
        let (cx, cy) = head.center();
        golem_head_step(&mut head, &w, &player, flags(), &mut rng);
        assert_eq!(head.ai[2], 0.0, "the volley resets");
        let beams: Vec<_> = head
            .wants_hostile_shot
            .iter()
            .filter(|s| s.0 == crate::projectile::GOLEM_EYE_BEAM)
            .copied()
            .collect();
        assert_eq!(beams.len(), 2, "the in-band two-shot");
        for (i, &(_, sx, sy, vx, vy, dmg)) in beams.iter().enumerate() {
            assert_eq!(dmg, 28);
            let speed = (vx * vx + vy * vy).sqrt();
            assert!((speed - 11.0).abs() < 1e-3, "the 11 speed");
            let side = if i == 0 { -18.0 } else { 18.0 };
            // The spawn carries the 3-tick lead along the aim.
            let dx = player.center().0 - (cx + side);
            let dy = player.center().1 - (cy - 22.0);
            let k = 11.0 / (dx * dx + dy * dy).sqrt();
            assert!((sx - (cx + side + dx * k * 3.0)).abs() < 1e-3);
            assert!((sy - (cy - 22.0 + dy * k * 3.0)).abs() < 1e-3);
        }
        // The side turn: one 12-speed shot from +30.
        let (w2, mut player2, mut h2) = docked();
        player2.x = 1900.0;
        h2.ai[0] = 1.0;
        h2.hp = h2.max_hp / 3;
        h2.ai[2] = 700.0;
        let (cx2, cy2) = h2.center();
        golem_head_step(&mut h2, &w2, &player2, flags(), &mut rng);
        let beams2: Vec<_> = h2
            .wants_hostile_shot
            .iter()
            .filter(|s| s.0 == crate::projectile::GOLEM_EYE_BEAM)
            .copied()
            .collect();
        assert_eq!(beams2.len(), 1, "the side single");
        let (_, sx, _, vx, vy, _) = beams2[0];
        let speed = (vx * vx + vy * vy).sqrt();
        assert!((speed - 12.0).abs() < 1e-3, "the 12 speed");
        assert!(sx > cx2 + 30.0, "the +30 muzzle + the lead");
        let _ = cy2;
    }

    /// The RNG witness (the module header): exactly ONE Next(600) per
    /// phase-1 tick, ZERO in phase 0 — two seeded streams stay in
    /// lockstep with a manual next_max(600) reference.
    #[test]
    fn s1_d_3_s46_the_arm_draws_one_next_600_per_phase1_tick() {
        let (w, player, head) = docked();
        // Phase 0: 120 ticks, zero draws.
        let mut h0 = head.clone();
        let mut rng_a = UnifiedRandom::new(42);
        let mut rng_b = UnifiedRandom::new(42);
        for _ in 0..120 {
            golem_head_step(&mut h0, &w, &player, flags(), &mut rng_a);
        }
        for _ in 0..8 {
            assert_eq!(rng_a.next(), rng_b.next(), "phase 0 draws nothing");
        }
        // Phase 1: one draw per tick (the gate's right-hand side is
        // unconditional), volley or not.
        let mut h1 = head.clone();
        h1.ai[0] = 1.0;
        h1.hp = h1.max_hp / 3;
        let mut rng_c = UnifiedRandom::new(42);
        let mut rng_d = UnifiedRandom::new(42);
        for _ in 0..600 {
            golem_head_step(&mut h1, &w, &player, flags(), &mut rng_c);
            h1.wants_hostile_shot.clear();
            let _ = rng_d.next_max(600);
            // Keep the head in phase 1 (the tail gate would too).
            h1.ai[0] = 1.0;
        }
        for _ in 0..8 {
            assert_eq!(rng_c.next(), rng_d.next(), "one Next(600) per tick");
        }
    }

    /// `NPC.cs:31131-31139` — the balance table: solo 1, the enrage x2
    /// out of the zones OR above the surface, the ftw +3 BEFORE the x2.
    #[test]
    fn s1_d_3_s46_the_balance_enrage_table() {
        let surface = 50.0; // the line at y = 800
        // Out of both zones, below the surface: x2.
        assert_eq!(balance_factor(flags(), 900.0, surface), 2.0);
        // In the temple AND below the surface: no enrage.
        let temple = Style46Flags {
            player_zone_temple: true,
            ..Default::default()
        };
        assert_eq!(balance_factor(temple, 900.0, surface), 1.0);
        // In the jungle only: still no enrage (the && gate).
        let jungle = Style46Flags {
            player_zone_jungle: true,
            ..Default::default()
        };
        assert_eq!(balance_factor(jungle, 900.0, surface), 1.0);
        // In the temple but ABOVE the surface line: the || arm enrages.
        assert_eq!(balance_factor(temple, 799.0, surface), 2.0);
        // Exactly on the line: the strict < keeps the temple calm.
        assert_eq!(balance_factor(temple, 800.0, surface), 1.0);
        // ftw: (1 + 3) * 2 = 8 enraged, 4 calm.
        let ftw = Style46Flags {
            get_good_world: true,
            ..Default::default()
        };
        assert_eq!(balance_factor(ftw, 900.0, surface), 8.0);
        let ftw_temple = Style46Flags {
            get_good_world: true,
            player_zone_temple: true,
            ..Default::default()
        };
        assert_eq!(balance_factor(ftw_temple, 900.0, surface), 4.0);
    }
}
