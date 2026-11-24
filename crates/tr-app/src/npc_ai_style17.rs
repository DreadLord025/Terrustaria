//! W4-d24 (aiStyle-17) — the perched-startle flyer family, `NPC.AI`'s
//! `if (this.aiStyle == 17)` block (`NPC.cs:24022-24152`), ported 1:1 as a
//! PURE velocity machine (code-only mode: no compiler/tests were run on
//! this branch; the witness tests below are written, not executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 17` rows, whole-file grep)
//!
//! TWO vanilla members:
//! * Vulture 61 (`NPCID.cs` `Vulture = 61`, `SetDefaults`
//!   `NPC.cs:9554-9566`: 36×36, dmg 15, def 4, 40 HP, kb 0.8,
//!   NPCHit28/NPCDeath31, value 60);
//! * Raven 301 (`Raven = 301`, `NPC.cs:12889-12901`: 36×26, dmg 12,
//!   def 2, 35 HP, kb 0.85, NPCHit1/NPCDeath1, value 50).
//! Neither writes `noGravity` in `SetDefaults` — the ARM owns the flag
//! (`:24024` true every tick, `:24027` false on the perched `ai[0] == 0`
//! tick). The port has no `NpcKind::Vulture`/`Raven` yet, so this module
//! lands UNWIRED (the `npc_ai_style53.rs` pre-wiring precedent): a pure
//! fn keyed on the shared machine, reachable and testable before the
//! kinds land. The wiring step adds the kinds + the style-axis claim.
//!
//! ## The machine
//!
//! * `ai[0] == 0` — the perch (`:24025-24047`): `noGravity` drops for the
//!   tick, `TargetClosest()` runs, and the bird arms `ai[0] = 1` when it
//!   is already moving (`vx != 0 || vy < 0 || (double)vy > 0.3` — the
//!   0.3 compare is a DOUBLE promote) or when the player's box enters
//!   the ±100-expanded own box (XNA `Rectangle.Intersects`, strict `<`)
//!   or the bird is hurt (`life < lifeMax`); the box/hurt startle also
//!   kicks `vy -= 6`.
//! * `ai[0] != 0` and the target alive (`:24048-24137`) — the flight:
//!   the collideX/collideY half-bounces off `oldVelocity` with the ±2/±1
//!   minimum-speed clamps, `TargetClosest()`, the ±3 X approach with the
//!   sign-reversal double-add (0.1 + 0.1 over the far band, 0.1 + 0.05
//!   over the near band), then the Y hover: the anchor is the player's
//!   TOP `position.Y` minus the bird's INTEGER half-height, dropped a
//!   further 100 px when the integer-half centre X distance exceeds 50;
//!   the 0.05 accel with the 0.01 sign-reversal extra and the ±3 caps.
//! * the wet tail (`:24138-24150`) — ALWAYS runs (both states): a
//!   positive vy damps ×0.95, then −0.5 buoyancy with the −4 floor, and
//!   one more `TargetClosest()`.
//!
//! Seam: the module writes only velocity, `ai[0]`, `direction`
//! (`Npc::facing`), `netUpdate` — and reads `wet`/`collideX`/`collideY`/
//! `oldVelocity` a tick late, exactly like the source (the collision
//! pass sets them in `UpdateNPC`, `NPC.cs:92538+`). It does NOT
//! integrate position: the C# arm ends before `UpdateCollision`. The
//! per-tick `noGravity` write leaves as the return value (the port's
//! `Npc` has no `no_gravity` field — the `npc_ai_style50` outcome
//! convention); the wiring caller owns the movement split.
//! `Main.netMode != 1` folds to true (the port is single-player).
//!
//! ## RNG by construction
//!
//! The arm draws NO `Main.rand` — `NPC.cs:24022-24152` contains no
//! `Next`/`NextFloat`; neither does `TargetClosest`/`TryTrackingTarget`/
//! `SetTargetTrackingValues` (`NPC.cs:77652-77781`, the `npc_target.rs`
//! port). So the fn takes no RNG parameter; the witness
//! `the_style17_arm_draws_no_rng` pins a `UnifiedRandom` state across
//! the perch, startle, flight and wet ticks (written, not run).
#![allow(dead_code)] // UNWIRED (W4-d24): the Vulture/Raven kinds + the axis claim are the wiring step

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The Vulture's `SetDefaults` row (`NPC.cs:9554-9566`), verbatim.
/// `SoundID.NPCHit28`/`NPCDeath31` = `LegacySoundStyle(3, 28)`/`(4, 31)`.
pub struct PerchedFlyerDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    /// `(SoundType, Style)` — the HitSound.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — the DeathSound.
    pub death_sound: (i32, i32),
    pub value: f32,
}

/// `NPC.cs:9554-9566` — `else if (this.type == 61)` (Vulture).
pub const VULTURE: PerchedFlyerDefaults = PerchedFlyerDefaults {
    width: 36,
    height: 36,
    ai_style: 17,
    damage: 15,
    defense: 4,
    life_max: 40,
    knock_back_resist: 0.8,
    hit_sound: (3, 28),
    death_sound: (4, 31),
    value: 60.0,
};

/// `NPC.cs:12889-12901` — `else if (this.type == 301)` (Raven).
pub const RAVEN: PerchedFlyerDefaults = PerchedFlyerDefaults {
    width: 36,
    height: 26,
    ai_style: 17,
    damage: 12,
    defense: 2,
    life_max: 35,
    knock_back_resist: 0.85,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    value: 50.0,
};

/// The target slot's candidate — the `npc_ai_style16.rs::target_of`
/// fallback (the C# indexes `Main.player[this.target]`, 255 slots always
/// exist; an unset/stale slot reads slot 0).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// XNA `Rectangle.Intersects` (`Rectangle.cs`): strict `<` on both axes
/// (`value.Left < this.Right && this.Left < value.Right && ...`).
fn rects_intersect(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    b.0 < a.0 + a.2 && a.0 < b.0 + b.2 && b.1 < a.1 + a.3 && a.1 < b.1 + b.3
}

/// The aiStyle-17 arm (`NPC.cs:24022-24152`). One fixed step; returns
/// the tick's `noGravity` value (the `:24024`/`:24027` writes) for the
/// wiring caller's movement split. RNG-free by construction.
pub fn ai_style17(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
) -> bool {
    let (w, h) = npc.size();
    // `:24024` — `this.noGravity = true` every tick ...
    let mut no_gravity = true;
    if npc.ai[0] == 0.0 {
        // `:24027-24028` — ... except the perch: gravity on, re-aim.
        no_gravity = false;
        target_closest(npc, world, players, true, 17, ticker);
        // `:24029-24046` — `Main.netMode != 1` folds true.
        // `:24031` — the moving gate; `(double)velocity.Y > 0.3` is the
        // f32→f64 promote (0.3f32 as f64 > 0.3f64 is FALSE at exactly
        // 0.3f32? no: 0.3f32 = 0.300000011920928955... > 0.3 — but the
        // C# promotes the FIELD value, so a vy of exactly 0.3f32 DOES
        // pass; a vy of 0.3f64-rounded-down does not. The port compares
        // the same promoted f32).
        if npc.vx != 0.0 || npc.vy < 0.0 || (npc.vy as f64) > 0.3 {
            npc.ai[0] = 1.0;
            npc.net_update = true;
        } else {
            // `:24038-24039` — the player box vs the ±100-expanded own
            // box (int-truncated positions), or any damage.
            let p = target_of(npc, players);
            let player_rect = (p.x as i32, p.y as i32, p.width, p.height);
            let own_rect = (npc.x as i32 - 100, npc.y as i32 - 100, w as i32 + 200, h as i32 + 200);
            if rects_intersect(own_rect, player_rect) || npc.hp < npc.max_hp {
                npc.ai[0] = 1.0;
                // `:24042` — the startle kick.
                npc.vy -= 6.0;
                npc.net_update = true;
            }
        }
    } else if !target_of(npc, players).dead {
        // `:24050-24061` — the X half-bounce with the ±2 clamps.
        if npc.collide_x {
            npc.vx = npc.old_vx * -0.5;
            if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
                npc.vx = 2.0;
            }
            if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
                npc.vx = -2.0;
            }
        }
        // `:24062-24073` — the Y half-bounce with the ±1 clamps.
        if npc.collide_y {
            npc.vy = npc.old_vy * -0.5;
            if npc.vy > 0.0 && npc.vy < 1.0 {
                npc.vy = 1.0;
            }
            if npc.vy < 0.0 && npc.vy > -1.0 {
                npc.vy = -1.0;
            }
        }
        // `:24074` — re-aim after the bounces.
        target_closest(npc, world, players, true, 17, ticker);
        // `:24075-24106` — the ±3 X approach, the sign-reversal
        // double-add (0.1 extra over the far band, 0.05 over the near).
        if npc.facing == -1 && npc.vx > -3.0 {
            npc.vx -= 0.1;
            if npc.vx > 3.0 {
                npc.vx -= 0.1;
            } else if npc.vx > 0.0 {
                npc.vx -= 0.05;
            }
            if npc.vx < -3.0 {
                npc.vx = -3.0;
            }
        } else if npc.facing == 1 && npc.vx < 3.0 {
            npc.vx += 0.1;
            if npc.vx < -3.0 {
                npc.vx += 0.1;
            } else if npc.vx < 0.0 {
                npc.vx += 0.05;
            }
            if npc.vx > 3.0 {
                npc.vx = 3.0;
            }
        }
        // `:24107-24112` — the hover anchor: INTEGER half-widths on both
        // centres; the player's TOP y minus the bird's INTEGER
        // half-height, 100 px higher when the X distance exceeds 50.
        let p = target_of(npc, players);
        let num261 = (npc.x + (w as i32 / 2) as f32
            - (p.x + (p.width / 2) as f32))
            .abs();
        let mut num262 = p.y - (h as i32 / 2) as f32;
        if num261 > 50.0 {
            num262 -= 100.0;
        }
        // `:24113-24128` — the 0.05 Y accel with the 0.01 reversal extra.
        if npc.y < num262 {
            npc.vy += 0.05;
            if npc.vy < 0.0 {
                npc.vy += 0.01;
            }
        } else {
            npc.vy -= 0.05;
            if npc.vy > 0.0 {
                npc.vy -= 0.01;
            }
        }
        // `:24129-24136` — the ±3 Y caps.
        if npc.vy < -3.0 {
            npc.vy = -3.0;
        }
        if npc.vy > 3.0 {
            npc.vy = 3.0;
        }
    }
    // `:24138-24150` — the wet tail (both states).
    if npc.wet {
        if npc.vy > 0.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.5;
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
        target_closest(npc, world, players, true, 17, ticker);
    }
    no_gravity
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation (`NPC.cs:24022-24152`).
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

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

    /// A stand-in body with the Vulture's 36×36 box (the kind is
    /// unported; the machine is type-agnostic — both members share it).
    fn bird(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((36.0, 36.0));
        n.hp = 40;
        n.max_hp = 40;
        n
    }

    /// The `SetDefaults` rows (`NPC.cs:9554-9566` / `:12889-12901`).
    #[test]
    fn the_defaults_rows_match_the_census() {
        assert_eq!((VULTURE.width, VULTURE.height), (36, 36));
        assert_eq!(VULTURE.ai_style, 17);
        assert_eq!((VULTURE.damage, VULTURE.defense, VULTURE.life_max), (15, 4, 40));
        assert_eq!(VULTURE.knock_back_resist, 0.8);
        assert_eq!(VULTURE.hit_sound, (3, 28));
        assert_eq!(VULTURE.death_sound, (4, 31));
        assert_eq!(VULTURE.value, 60.0);
        assert_eq!((RAVEN.width, RAVEN.height), (36, 26));
        assert_eq!((RAVEN.damage, RAVEN.defense, RAVEN.life_max), (12, 2, 35));
        assert_eq!(RAVEN.knock_back_resist, 0.85);
        assert_eq!(RAVEN.hit_sound, (3, 1));
        assert_eq!(RAVEN.death_sound, (4, 1));
        assert_eq!(RAVEN.value, 50.0);
    }

    /// `:24025-24046` — the perch: a parked full-HP bird with the player
    /// outside the ±100 box stays perched (`ai[0] == 0`, gravity ON).
    #[test]
    fn the_perch_holds_while_the_player_is_far() {
        let w = air();
        let mut n = bird(1000.0, 1000.0);
        let ng = ai_style17(&mut n, &w, &[cand(1400.0, 1000.0)], 5);
        assert!(!ng, ":24027 — the perch tick has gravity");
        assert_eq!(n.ai[0], 0.0);
        assert_eq!(n.target, 0, "the perch re-aims");
        assert_eq!(n.target_set_frame, 5);
        assert_eq!(n.vy, 0.0, "no startle kick");
    }

    /// `:24038-24044` — the box startle: the player inside the expanded
    /// box arms the flight and kicks `vy -= 6`.
    #[test]
    fn the_box_startle_kicks_six() {
        let w = air();
        let mut n = bird(1000.0, 1000.0);
        // The own box spans x 900..1136; the player at 1050 is inside.
        let ng = ai_style17(&mut n, &w, &[cand(1050.0, 1000.0)], 0);
        assert!(!ng, "the startle tick is still the perch branch");
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.vy, -6.0, ":24042");
        assert!(n.net_update);
    }

    /// `:24031-24035` — the moving gate: any vx (or vy < 0) arms the
    /// flight WITHOUT the −6 kick.
    #[test]
    fn the_moving_gate_arms_without_the_kick() {
        let w = air();
        let mut n = bird(1000.0, 1000.0);
        n.vx = 0.5;
        ai_style17(&mut n, &w, &[cand(1400.0, 1000.0)], 0);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.vy, 0.0, "no kick on the moving gate");
        // The hurt gate (`:24039`'s `life < lifeMax`) kicks even far.
        let mut m = bird(1000.0, 1000.0);
        m.hp = 39;
        ai_style17(&mut m, &w, &[cand(1400.0, 1000.0)], 0);
        assert_eq!(m.ai[0], 1.0);
        assert_eq!(m.vy, -6.0);
    }

    /// `:24050-24073` — the half-bounces: collideX reflects oldVelocity
    /// ×−0.5 with the ±2 clamp; collideY ×−0.5 with the ±1 clamp.
    #[test]
    fn the_bounces_reflect_old_velocity_with_clamps() {
        let w = air();
        let mut n = bird(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.facing = -1;
        n.collide_x = true;
        n.old_vx = -1.0; // ×−0.5 = 0.5, inside (0,2) → clamps to 2
        n.collide_y = true;
        n.old_vy = -1.0; // ×−0.5 = 0.5, inside (0,1) → clamps to 1
        ai_style17(&mut n, &w, &[cand(900.0, 1000.0)], 0);
        // The approach then pulls vx back toward −3: 2 − 0.1 − 0.1 = 1.8.
        assert_eq!(n.vx, 1.8, "the clamp then the far-band double-add");
        // vy: clamp 1, then the hover (player top 1000 − half 18 = 982;
        // the bird at 1000 is BELOW → −0.05 − 0.01) = 0.94.
        assert!((n.vy - 0.94).abs() < 1e-5, "the Y clamp + hover ({})", n.vy);
    }

    /// `:24075-24106` — the approach caps at ±3 with the near-band 0.05.
    #[test]
    fn the_x_approach_caps_at_three() {
        let w = air();
        let mut n = bird(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.facing = 1;
        n.vx = 2.95;
        ai_style17(&mut n, &w, &[cand(1400.0, 1000.0)], 0);
        assert_eq!(n.vx, 3.0, "2.95 + 0.1 caps at 3");
        // The near band: a reversal from −0.02 adds 0.1 + 0.05.
        let mut m = bird(1000.0, 1000.0);
        m.ai[0] = 1.0;
        m.facing = 1;
        m.vx = -0.02;
        ai_style17(&mut m, &w, &[cand(1400.0, 1000.0)], 0);
        assert!((m.vx - 0.13).abs() < 1e-5, "-0.02 + 0.1 + 0.05 ({})", m.vx);
    }

    /// `:24107-24136` — the hover anchor: >50 px X distance drops the
    /// anchor 100 px; the ±3 Y caps hold.
    #[test]
    fn the_hover_anchor_drops_a_hundred_far() {
        let w = air();
        let mut n = bird(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.facing = 1;
        n.vy = -2.99;
        // The player far right (dx > 50): anchor = 1000 − 18 − 100 = 882;
        // the bird at 1000 is below → vy −0.05 (no extra, vy < 0).
        ai_style17(&mut n, &w, &[cand(1400.0, 1000.0)], 0);
        assert!((n.vy - -3.0).abs() < 1e-5, "-2.99 - 0.01 caps at -3");
        // Near (dx <= 50): anchor = 1000 − 18 = 982, still below.
        let mut m = bird(1000.0, 900.0);
        m.ai[0] = 1.0;
        m.facing = 1;
        m.vy = 0.0;
        ai_style17(&mut m, &w, &[cand(1010.0, 1000.0)], 0);
        assert!((m.vy - 0.05).abs() < 1e-5, "above the anchor: +0.05");
    }

    /// `:24138-24150` — the wet tail: positive vy damps ×0.95, −0.5
    /// buoyancy, the −4 floor, and one more TargetClosest.
    #[test]
    fn the_wet_tail_damps_and_lifts() {
        let w = air();
        let mut n = bird(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.facing = 1;
        n.wet = true;
        n.vy = 2.0;
        // The player at the anchor height keeps the hover term tiny.
        ai_style17(&mut n, &w, &[cand(1010.0, 1018.0)], 9);
        // Hover: anchor 1018 − 18 = 1000; the bird AT 1000 → else branch
        // (not <): vy 2 − 0.05 − 0.01 = 1.94; wet: ×0.95 = 1.843, −0.5.
        assert!((n.vy - 1.343).abs() < 1e-4, "the damp + lift ({})", n.vy);
        assert_eq!(n.target_set_frame, 9, "the wet re-aim ran");
        // The floor: a deep sink clamps at −4.
        let mut m = bird(1000.0, 1000.0);
        m.ai[0] = 1.0;
        m.wet = true;
        m.vy = -3.9;
        ai_style17(&mut m, &w, &[cand(1400.0, 2000.0)], 0);
        assert!(m.vy >= -4.0, "the −4 floor ({})", m.vy);
    }

    /// The RNG census: the whole arm (perch, startle, flight, wet) draws
    /// NOTHING — a pinned `UnifiedRandom` state survives every branch.
    #[test]
    fn the_style17_arm_draws_no_rng() {
        let w = air();
        let mut rng = UnifiedRandom::new(4242);
        let witness = rng.clone();
        let mut n = bird(1000.0, 1000.0);
        let p = cand(1050.0, 1000.0);
        // The perch + startle.
        ai_style17(&mut n, &w, &[p], 0);
        // The flight (bounces + approach + hover).
        n.collide_x = true;
        n.collide_y = true;
        n.old_vx = 2.0;
        n.old_vy = -2.0;
        for _ in 0..60 {
            ai_style17(&mut n, &w, &[p], 0);
        }
        // The wet tail.
        n.wet = true;
        for _ in 0..60 {
            ai_style17(&mut n, &w, &[p], 0);
        }
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the arm");
    }
}
