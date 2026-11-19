//! The aiStyle-14 flyer family — `NPC.AI()`'s `if (this.aiStyle == 14)` arm
//! (`NPC.cs:22849-23486`), S1-D-3 (aiStyle-14 slice, code-only mode).
//!
//! The C# arm serves 17 types (`SetDefaults`' `this.aiStyle = 14` rows:
//! 48/49/51/60/62/66/93/121/137/150/151/152/156/226/158/634/660 —
//! `NPC.cs:9364`/`:9378`/`:9409`/`:9543`/`:9572`/`:9629`/`:10052`/`:10470`/
//! `:10739`/`:10890`/`:10905`/`:10921`/`:10979`/`:11916`/`:15471`/`:17166`/
//! `:17342`). The port models four of them: CaveBat 49, JungleBat 51,
//! Hellbat 60, VoodooDemon 66 (`npc/data.rs::ai_style`). This module ports
//! the whole arm's STRUCTURE with the branches those four types take; the
//! unported types' own branches (226's velocity-sign facing `:22876-22894`,
//! 158's surface flee + 7-cap approach `:22899-22970` + Transform(159)
//! `:23346-23355` + double `ai[1]++` `:23357-23360`, 226's 4/2.5 approach
//! `:22971-23037`, 660's 0.35/0.3/6/5 approach `:23038-23115`, 48's wet arm
//! `:23333-23345` + feather shot `:23416-23436`, 156's trident shot
//! `:23458-23484`) are cited skips — they land with their kinds.
//!
//! WIRED (the S1-D-3 bat/fish reconcile, `task/S1-D-3-reconcile-batfish`):
//! `npc.rs::update_cave_bat` routes the four kinds through [`bat_ai`],
//! feeds `Entity.wet` via the `is_wet` pass + the session-player
//! `TargetCandidate`/`players_wet` slices, drains [`DemonSickleShot`] into
//! `wants_hostile_shot` (type 44, damage 21; the sim's timed-shot pass
//! applies the `timeLeft = 300` write, `NPC.cs:23450`), and integrates
//! through the no-gravity `integrate_flyer_collision` stand-in
//! (`UpdateCollision`, `NPC.cs:92523-92580`). The distrust-verbatim audit
//! re-read the whole arm against `NPC.cs:22849-23486` at the wiring —
//! verdict FAITHFUL (the ungated second generic pass, the Hellbat's own
//! arm, the X-before-Y shot draws and the short-circuit `Next(300)` all
//! match; no f1701eb0-class defect found). Every other module is
//! byte-identical.
//!
//! RNG by construction: the ONLY `Main.rand` draws on the ported paths are
//! the VoodooDemon/Demon (62/66) shot arm's — two `Next(-100, 101)` aim
//! jitters, X first then Y (`NPC.cs:23446-23447`), on the shot ticks
//! `ai[0] == 20/40/60/80` with `Collision.CanHit`; otherwise ONE
//! `Next(300)` reset roll per tick, drawn only when the firing-distance
//! flag holds (the C# `&&` short-circuits, `:23453`). The bat kinds
//! (49/51/60) draw nothing. `TargetClosest` is RNG-free (the Manhattan
//! scan, `NPC.cs:77652-77781`). The witness tests below pin the stream
//! position for each case (written, NOT run — code-only mode).
// WIRED by the S1-D-3 bat/fish reconcile (`update_cave_bat`'s migration).

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The read-only per-tick inputs the C# arm takes from `Main` statics.
pub struct Style14Env<'a> {
    pub world: &'a World,
    /// The `Main.player[]` slots as `TargetClosest` sees them (the
    /// `npc_target.rs` seam, S1-D-3p).
    pub players: &'a [TargetCandidate],
    /// `Main.player[j].wet` per slot — the `ai[1]` reset gate reads it
    /// (`NPC.cs:23363`); `TargetCandidate` does not carry `wet` (the
    /// target-scan never reads it), so it rides this parallel slice.
    pub players_wet: &'a [bool],
    /// `Main.netMode` — 1 (client) returns before the shot arms
    /// (`NPC.cs:23410-23413`).
    pub net_mode: i32,
    /// `Main.EverLastingTicker` — the `targetSetFrame` stamp
    /// (`NPC.cs:77780` via `TargetClosest`).
    pub ticker: u64,
}

/// The type-62/66 shot request — `Projectile.NewProjectile` at
/// `NPC.cs:23449-23450`: type 44 (`ProjectileID.DemonSickle`,
/// `ProjectileID.cs`), damage 21, knockback 0, owner `Main.myPlayer`,
/// spawned at the NPC centre with the jittered aim; the caller sets
/// `timeLeft = 300` (`:23450`). The pure module never touches the pool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DemonSickleShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// `NPC.cs:23183`'s water-arm type list, verbatim order.
const WATER_GROUP: [i32; 11] = [49, 51, 60, 62, 66, 93, 137, 150, 151, 152, 634];

/// The aiStyle-14 arm (`NPC.cs:22849-23486`) for one NPC, one tick.
/// `this.noGravity = true` (`:22851`) is structural: the port's flyer
/// callers integrate without gravity (the `Npc` struct has no
/// `no_gravity` field; `update_hover`'s convention). Returns the
/// VoodooDemon's shot request on the shoot tick, else `None`.
pub fn bat_ai(npc: &mut Npc, env: &Style14Env<'_>, rng: &mut UnifiedRandom) -> Option<DemonSickleShot> {
    let net_id = npc.kind.net_id();
    // `:22852-22875` — the tile-bounce clamps.
    bounce(npc);
    // `:22876-22898` — type 226 faces by velocity sign when the target is
    // not hittable (unported kind); every other type runs `TargetClosest`
    // (`:22897`), faceTarget default true, aiStyle 14.
    target_closest(npc, env.world, env.players, true, 14, env.ticker);
    // `:22899-23182` — the per-type approach arms: 158/226/660 unported;
    // the else arm (`:23116-23182`) is the shared 0.1/4 X + 0.04/1.5 Y.
    generic_approach(npc);
    // `:23183-23332` — the water group. NOTE the C# structure: the wet
    // lift is gated on `base.wet`, but the SECOND approach pass below is
    // not — types 49/51/66 run the generic approach twice per tick, and
    // type 60 follows it with its own slower-Y arm. Faithful, not a bug.
    if WATER_GROUP.contains(&net_id) {
        if npc.wet {
            // `:23185-23197`.
            if npc.vy > 0.0 {
                npc.vy *= 0.95;
            }
            npc.vy -= 0.5;
            if npc.vy < -4.0 {
                npc.vy = -4.0;
            }
            target_closest(npc, env.world, env.players, true, 14, env.ticker);
        }
        if net_id == 60 {
            // `:23198-23264` — the Hellbat's own arm.
            hellbat_approach(npc);
        } else {
            // `:23265-23331` — the generic approach, again, verbatim.
            generic_approach(npc);
        }
    }
    // `:23333-23345` — type 48's wet arm (Harpy unported, skip).
    // `:23346-23355` — type 158's Transform(159) (VampireBat unported).
    // `:23356-23409` — the drift cycle (`ai[1]`/`ai[2]`).
    drift(npc, env);
    // `:23410-23413` — the client stops here.
    if env.net_mode == 1 {
        return None;
    }
    // `:23414-23415` — `player3` + the global firing-distance flag.
    let p = &env.players[npc.target as usize];
    let (cx, cy) = npc.center();
    let pcx = p.x + p.width as f32 / 2.0;
    let pcy = p.y + p.height as f32 / 2.0;
    let flag24 = global_firing_distance_check(cx, cy, pcx, pcy);
    // `:23416-23436` — type 48's feather shot (Harpy unported, skip).
    // `:23437-23457` — the Demon/VoodooDemon sickle shot.
    if net_id == 62 || net_id == 66 {
        npc.ai[0] += 1.0;
        if npc.ai[0] == 20.0 || npc.ai[0] == 40.0 || npc.ai[0] == 60.0 || npc.ai[0] == 80.0 {
            // `Collision.CanHit(this, player3)` (`Collision.cs:371-374`):
            // the two entity boxes.
            let (w, h) = npc.size();
            if crate::projectile::can_hit(
                env.world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
            ) {
                // `:23444-23448` — the 0.2-scale jittered aim; the X
                // jitter draws BEFORE the Y jitter (`:23446-23447`).
                let num240 = 0.2f32;
                let num241 = pcx - cx + rng.next_range(-100, 101) as f32;
                let num242 = pcy - cy + rng.next_range(-100, 101) as f32;
                let num243 = num240 / (num241 * num241 + num242 * num242).sqrt();
                // `:23449-23450` — NewProjectile(44, 21, 0f) at the
                // centre; timeLeft 300 is the caller's write.
                return Some(DemonSickleShot {
                    x: cx,
                    y: cy,
                    vx: num241 * num243,
                    vy: num242 * num243,
                });
            }
        } else if flag24 && npc.ai[0] >= (300 + rng.next_max(300)) as f32 {
            // `:23453-23456` — the reset roll. The `&&` short-circuit:
            // `Next(300)` is drawn only when flag24 holds AND the tick is
            // not a shot-value tick (the else-if). `Next(int)` =
            // `UnifiedRandom.cs:134-141` (`next_max`).
            npc.ai[0] = 0.0;
        }
    }
    // `:23458-23484` — type 156's trident shot (RedDevil unported, skip).
    None
}

/// `NPC.cs:22852-22875` — the collideX/collideY half-bounce with the
/// direction-aware minimum speeds (X ±2, Y ±1).
pub fn bounce(npc: &mut Npc) {
    if npc.collide_x {
        // `:22854-22862`.
        npc.vx = npc.old_vx * -0.5;
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
            npc.vx = 2.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
            npc.vx = -2.0;
        }
    }
    if npc.collide_y {
        // `:22866-22874`.
        npc.vy = npc.old_vy * -0.5;
        if npc.vy > 0.0 && npc.vy < 1.0 {
            npc.vy = 1.0;
        }
        if npc.vy < 0.0 && npc.vy > -1.0 {
            npc.vy = -1.0;
        }
    }
}

/// The shared approach (`NPC.cs:23116-23182`, repeated verbatim at
/// `:23265-23331`): X 0.1 accel with the opposite-side double-add and the
/// 0.05 near-zero brake, cap ±4; Y 0.04 accel / 0.05 double-add / 0.03
/// brake, cap ±1.5. The C#'s `(double)velocity.Y` comparisons promote the
/// exact f32 values against exactly-representable ±1.5 — the f32
/// comparison is identical.
pub fn generic_approach(npc: &mut Npc) {
    // `:23118-23149`.
    if npc.facing == -1 && npc.vx > -4.0 {
        npc.vx -= 0.1;
        if npc.vx > 4.0 {
            npc.vx -= 0.1;
        } else if npc.vx > 0.0 {
            npc.vx += 0.05;
        }
        if npc.vx < -4.0 {
            npc.vx = -4.0;
        }
    } else if npc.facing == 1 && npc.vx < 4.0 {
        npc.vx += 0.1;
        if npc.vx < -4.0 {
            npc.vx += 0.1;
        } else if npc.vx < 0.0 {
            npc.vx -= 0.05;
        }
        if npc.vx > 4.0 {
            npc.vx = 4.0;
        }
    }
    // `:23150-23181`.
    if npc.direction_y == -1 && npc.vy > -1.5 {
        npc.vy -= 0.04;
        if npc.vy > 1.5 {
            npc.vy -= 0.05;
        } else if npc.vy > 0.0 {
            npc.vy += 0.03;
        }
        if npc.vy < -1.5 {
            npc.vy = -1.5;
        }
    } else if npc.direction_y == 1 && npc.vy < 1.5 {
        npc.vy += 0.04;
        if npc.vy < -1.5 {
            npc.vy += 0.05;
        } else if npc.vy < 0.0 {
            npc.vy -= 0.03;
        }
        if npc.vy > 1.5 {
            npc.vy = 1.5;
        }
    }
}

/// The Hellbat's second pass (`NPC.cs:23200-23263`): the same X shape as
/// the generic arm but a 0.07 double-add / 0.03 brake, and the slower Y
/// (0.04 / 0.03 double-add / 0.02 brake, cap ±1.5).
pub fn hellbat_approach(npc: &mut Npc) {
    // `:23200-23231`.
    if npc.facing == -1 && npc.vx > -4.0 {
        npc.vx -= 0.1;
        if npc.vx > 4.0 {
            npc.vx -= 0.07;
        } else if npc.vx > 0.0 {
            npc.vx += 0.03;
        }
        if npc.vx < -4.0 {
            npc.vx = -4.0;
        }
    } else if npc.facing == 1 && npc.vx < 4.0 {
        npc.vx += 0.1;
        if npc.vx < -4.0 {
            npc.vx += 0.07;
        } else if npc.vx < 0.0 {
            npc.vx -= 0.03;
        }
        if npc.vx > 4.0 {
            npc.vx = 4.0;
        }
    }
    // `:23232-23263`.
    if npc.direction_y == -1 && npc.vy > -1.5 {
        npc.vy -= 0.04;
        if npc.vy > 1.5 {
            npc.vy -= 0.03;
        } else if npc.vy > 0.0 {
            npc.vy += 0.02;
        }
        if npc.vy < -1.5 {
            npc.vy = -1.5;
        }
    } else if npc.direction_y == 1 && npc.vy < 1.5 {
        npc.vy += 0.04;
        if npc.vy < -1.5 {
            npc.vy += 0.03;
        } else if npc.vy < 0.0 {
            npc.vy -= 0.02;
        }
        if npc.vy > 1.5 {
            npc.vy = 1.5;
        }
    }
}

/// The drift cycle (`NPC.cs:23356-23409`): `ai[1]++` (158's second
/// increment is its own unported branch), the >200 gate with the
/// dry-target + line-of-sight reset, the per-type accel/cap rows
/// (48/62/66: 0.12/0.07/3/1.25; else 0.2/0.1/4/1.5), the >1000 wrap, the
/// `ai[2]` vertical/horizontal shove and the ±300 wrap.
fn drift(npc: &mut Npc, env: &Style14Env<'_>) {
    let net_id = npc.kind.net_id();
    npc.ai[1] += 1.0; // `:23356`.
    if npc.ai[1] > 200.0 {
        // `:23361-23409`.
        let p = &env.players[npc.target as usize];
        let (w, h) = npc.size();
        // `:23363-23366` — `!player.wet && Collision.CanHit(boxes)`.
        if !env.players_wet[npc.target as usize]
            && crate::projectile::can_hit(
                env.world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
            )
        {
            npc.ai[1] = 0.0;
        }
        // `:23367-23377` — the tunables.
        let (num231, num232, num233, num234) = if net_id == 48 || net_id == 62 || net_id == 66 {
            (0.12f32, 0.07f32, 3.0f32, 1.25f32)
        } else {
            (0.2f32, 0.1f32, 4.0f32, 1.5f32)
        };
        // `:23378-23381`.
        if npc.ai[1] > 1000.0 {
            npc.ai[1] = 0.0;
        }
        // `:23382-23393`.
        npc.ai[2] += 1.0;
        if npc.ai[2] > 0.0 {
            if npc.vy < num234 {
                npc.vy += num232;
            }
        } else if npc.vy > -num234 {
            npc.vy -= num232;
        }
        // `:23394-23404`.
        if npc.ai[2] < -150.0 || npc.ai[2] > 150.0 {
            if npc.vx < num233 {
                npc.vx += num231;
            }
        } else if npc.vx > -num233 {
            npc.vx -= num231;
        }
        // `:23405-23408`.
        if npc.ai[2] > 300.0 {
            npc.ai[2] = -300.0;
        }
    }
}

/// `NPC.AI_GlobalFiringDistanceCheck` (`NPC.cs:55835-55841`):
/// `Utils.CenteredRectangle(targetPos.ToPoint(), Main.MaxWorldViewSize)`
/// (the Point overload — int centre − size/2), `Inflate(-50, -50)`
/// (x += 50, width −= 100), `Contains(shootPos.ToPoint())` — XNA's
/// half-open rect test. `Main.MaxWorldViewSize` = 1920×1200
/// (`Main.cs:677-681`). `ToPoint` truncates both centres to ints.
pub fn global_firing_distance_check(shoot_x: f32, shoot_y: f32, target_x: f32, target_y: f32) -> bool {
    let rx = target_x as i32 - 1920 / 2 + 50;
    let ry = target_y as i32 - 1200 / 2 + 50;
    let (rw, rh) = (1920 - 100, 1200 - 100);
    let (sx, sy) = (shoot_x as i32, shoot_y as i32);
    sx >= rx && sx < rx + rw && sy >= ry && sy < ry + rh
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc
    //! per user directive). Each pins one C# behaviour or the RNG stream
    //! position (`UnifiedRandom::peek` is the no-advance probe,
    //! `UnifiedRandom.cs:90-103`).
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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

    /// A solid column at tile x 66 between the NPC (centre ~1011) and a
    /// player at x 1300 — blocks `Collision.CanHit`'s walk (the
    /// `npc_target.rs` wall fixture).
    fn walled() -> World {
        let mut w = air();
        for y in 55..70 {
            let i = w.index(66, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    fn env<'a>(w: &'a World, players: &'a [TargetCandidate], wet: &'a [bool]) -> Style14Env<'a> {
        Style14Env {
            world: w,
            players,
            players_wet: wet,
            net_mode: 0,
            ticker: 7,
        }
    }

    fn near(v: f32, expect: f32) -> bool {
        (v - expect).abs() < 1e-5
    }

    /// `NPC.cs:23116-23182` + `:23265-23331` — the dry CaveBat runs the
    /// generic approach TWICE per tick (the water-group's second pass is
    /// not gated on `wet`): 0 → −0.1 → −0.2 X, 0 → −0.04 → −0.08 Y.
    #[test]
    fn s1_d_3_s14_the_generic_approach_runs_twice_for_the_bats() {
        let w = air();
        // Player left + above → facing −1, directionY −1.
        let players = [cand(900.0, 900.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::CaveBat, 1000.0, 1000.0, 1, 0.0);
        let mut rng = UnifiedRandom::new(1);
        assert!(bat_ai(&mut n, &env(&w, &players, &wet), &mut rng).is_none());
        assert!(near(n.vx, -0.2), "two −0.1 passes: {}", n.vx);
        assert!(near(n.vy, -0.08), "two −0.04 passes: {}", n.vy);
    }

    /// `NPC.cs:23198-23263` — the Hellbat's second pass is its OWN arm
    /// (0.02 Y brake, not 0.03): vy 0.5 → generic 0.49 → hellbat 0.47,
    /// not the double-generic 0.48. directionY −1 (player above) with a
    /// descending vy exercises the brake branches.
    #[test]
    fn s1_d_3_s14_the_hellbat_gets_its_own_second_pass() {
        let w = air();
        // Player left + above → facing −1, directionY −1.
        let players = [cand(900.0, 900.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::Hellbat, 1000.0, 1000.0, 1, 0.0);
        n.vy = 0.5;
        let mut rng = UnifiedRandom::new(1);
        bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert!(near(n.vx, -0.2), "0.1 + 0.1: {}", n.vx);
        // generic: 0.5 − 0.04 + 0.03 = 0.49; hellbat: − 0.04 + 0.02.
        assert!(near(n.vy, 0.47), "the 0.02 brake, not 0.03: {}", n.vy);
    }

    /// `NPC.cs:22852-22875` — the half-bounce with the direction-aware
    /// minimum speeds.
    #[test]
    fn s1_d_3_s14_the_collision_bounce_clamps() {
        let mut n = Npc::new(NpcKind::CaveBat, 1000.0, 1000.0, -1, 0.0);
        n.collide_x = true;
        n.old_vx = -3.0; // bounce → +1.5, facing −1 → clamp 2.
        n.collide_y = true;
        n.old_vy = -1.2; // bounce → +0.6 → clamp 1.
        bounce(&mut n);
        assert_eq!(n.vx, 2.0);
        assert_eq!(n.vy, 1.0);
        // Outside the clamp window the half-bounce stands.
        let mut n2 = Npc::new(NpcKind::CaveBat, 1000.0, 1000.0, -1, 0.0);
        n2.collide_x = true;
        n2.old_vx = 6.0; // → −3, no clamp (negative while facing −1).
        n2.collide_y = true;
        n2.old_vy = 4.0; // → −2, below −1 so no clamp.
        bounce(&mut n2);
        assert_eq!(n2.vx, -3.0);
        assert_eq!(n2.vy, -2.0);
    }

    /// `NPC.cs:23185-23197` — the wet lift: the 0.95 down-damp, the −0.5
    /// ascent, the −4 cap, and the second `TargetClosest`. With the
    /// player above (directionY −1) and vy ≤ −1.5 the approach passes are
    /// no-ops, so the cap is directly observable.
    #[test]
    fn s1_d_3_s14_the_wet_arm_lifts_and_retargets() {
        let w = air();
        let players = [cand(900.0, 900.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::JungleBat, 1000.0, 1000.0, 1, 0.0);
        n.wet = true;
        n.vy = -3.9; // −0.5 → −4.4 → cap −4; approach no-ops at −4.
        let mut rng = UnifiedRandom::new(1);
        bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert_eq!(n.vy, -4.0, "the −4 ascent cap");
        assert_eq!(
            n.target_rect,
            (900, 900, 20, 42),
            "the wet arm re-runs TargetClosest (:23196)"
        );
        // The descending case: pass 1 runs BEFORE the wet arm —
        // 2 − 0.04 − 0.05 = 1.91; wet: ×0.95 = 1.8145, −0.5 = 1.3145;
        // pass 2: −0.04 + 0.03 → 1.3045.
        let mut n2 = Npc::new(NpcKind::JungleBat, 1000.0, 1000.0, 1, 0.0);
        n2.wet = true;
        n2.vy = 2.0;
        bat_ai(&mut n2, &env(&w, &players, &wet), &mut rng);
        assert!(near(n2.vy, 1.3045), "pass1 + damp/lift + pass2: {}", n2.vy);
    }

    /// `NPC.cs:23356-23408` — the drift cycle: the >200 dry-target reset,
    /// the `ai[2]` shove and the +300 → −300 wrap. vy −2 with directionY
    /// −1 keeps the approach passes silent.
    #[test]
    fn s1_d_3_s14_the_drift_cycle_resets_and_wraps() {
        let w = air();
        let players = [cand(900.0, 900.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::CaveBat, 1000.0, 1000.0, 1, 0.0);
        n.ai[1] = 200.0; // ++ → 201 > 200; dry + CanHit → 0.
        n.ai[2] = 300.0; // ++ → 301 > 0 → vy += 0.1; > 300 → −300.
        n.vy = -2.0;
        let mut rng = UnifiedRandom::new(1);
        bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert_eq!(n.ai[1], 0.0, "the dry-target line-of-sight reset");
        assert_eq!(n.ai[2], -300.0, "the wrap");
        assert!(near(n.vy, -1.9), "the generic 0.1 shove: {}", n.vy);
        // A wet target keeps the counter (the reset gate fails).
        let wet_true = [true];
        let mut n2 = Npc::new(NpcKind::CaveBat, 1000.0, 1000.0, 1, 0.0);
        n2.ai[1] = 200.0;
        n2.vy = -2.0;
        bat_ai(&mut n2, &env(&w, &players, &wet_true), &mut rng);
        assert_eq!(n2.ai[1], 201.0, "a wet target is not reset (:23363)");
    }

    /// `NPC.cs:23371-23377` + `:23378-23381` — the VoodooDemon's slow
    /// drift row (0.07 shove) and the >1000 wrap when the reset gate
    /// stays shut (a wall blocks CanHit).
    #[test]
    fn s1_d_3_s14_the_voodoo_drift_uses_the_slow_params() {
        let w = walled();
        let players = [cand(1300.0, 979.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::VoodooDemon, 1000.0, 1000.0, 1, 0.0);
        n.ai[1] = 200.0; // → 201, CanHit false → no reset.
        n.ai[2] = 0.0; // → 1 > 0 → vy += 0.07 (the 66 row).
        n.vy = -2.0;
        let mut rng = UnifiedRandom::new(1);
        bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert_eq!(n.ai[1], 201.0);
        assert!(near(n.vy, -1.93), "the 0.07 shove, not 0.1: {}", n.vy);
        // The >1000 wrap.
        let mut n2 = Npc::new(NpcKind::VoodooDemon, 1000.0, 1000.0, 1, 0.0);
        n2.ai[1] = 1000.0; // → 1001, no reset → >1000 → 0.
        n2.vy = -2.0;
        bat_ai(&mut n2, &env(&w, &players, &wet), &mut rng);
        assert_eq!(n2.ai[1], 0.0, "the 1000 wrap (:23378-23381)");
    }

    /// `NPC.cs:23444-23450` — the shot tick draws exactly two
    /// `Next(-100, 101)` jitters, X FIRST then Y (`:23446-23447`), and the
    /// aim is the 0.2-normalised jittered vector. The stream-position
    /// witness: after the tick the live rng equals a clone advanced by
    /// the two draws in that order.
    #[test]
    fn s1_d_3_s14_the_voodoo_shot_draws_x_then_y() {
        let w = air();
        let players = [cand(1100.0, 1000.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::VoodooDemon, 1000.0, 1000.0, 1, 0.0);
        n.ai[0] = 19.0; // ++ → 20, the first shot tick.
        let mut rng = UnifiedRandom::new(12345);
        let mut witness = UnifiedRandom::new(12345);
        let shot = bat_ai(&mut n, &env(&w, &players, &wet), &mut rng).expect("the shot tick fires");
        let (cx, cy) = n.center();
        let pcx = 1100.0 + 10.0;
        let pcy = 1000.0 + 21.0;
        let dx = pcx - cx + witness.next_range(-100, 101) as f32;
        let dy = pcy - cy + witness.next_range(-100, 101) as f32;
        let s = 0.2f32 / (dx * dx + dy * dy).sqrt();
        assert_eq!(shot.x, cx);
        assert_eq!(shot.y, cy);
        assert_eq!(shot.vx, dx * s, "the X jitter draws first");
        assert_eq!(shot.vy, dy * s);
        assert_eq!(rng.peek(), witness.peek(), "exactly two draws, X then Y");
        assert_eq!(n.ai[0], 20.0);
    }

    /// `NPC.cs:23453` — the reset roll: ONE `Next(300)` per tick when the
    /// firing-distance flag holds and the tick is not a shot value.
    /// ai[0] 250 → 251 < 300 + roll, so no reset, but the draw lands.
    #[test]
    fn s1_d_3_s14_the_reset_roll_draws_once_only_in_range() {
        let w = air();
        let players = [cand(1100.0, 1000.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::VoodooDemon, 1000.0, 1000.0, 1, 0.0);
        n.ai[0] = 250.0;
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let roll = witness.next_max(300);
        let shot = bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert!(shot.is_none());
        assert_eq!(n.ai[0], 251.0, "251 < 300 + {} — no reset", roll);
        assert_eq!(rng.peek(), witness.peek(), "exactly one Next(300)");
    }

    /// `NPC.cs:23453`'s `&&` short-circuit — out of the firing window
    /// (`AI_GlobalFiringDistanceCheck`, `:55835-55841`) the roll is never
    /// drawn: the stream does not move.
    #[test]
    fn s1_d_3_s14_out_of_view_draws_nothing() {
        let w = air();
        // 5000 px away — outside the 1820-wide inflated view rect.
        let players = [cand(6000.0, 1000.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::VoodooDemon, 1000.0, 1000.0, 1, 0.0);
        n.ai[0] = 250.0;
        let mut rng = UnifiedRandom::new(7);
        let before = rng.peek();
        bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert_eq!(n.ai[0], 251.0);
        assert_eq!(rng.peek(), before, "flag24 false → no draw");
    }

    /// `NPC.cs:23440-23452` — a shot-value tick with a blocked line of
    /// sight draws NOTHING: the else-if (the reset roll) is not evaluated
    /// when the ai[0] value matched the shot set.
    #[test]
    fn s1_d_3_s14_a_blocked_shot_tick_draws_nothing() {
        let w = walled();
        let players = [cand(1300.0, 979.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::VoodooDemon, 1000.0, 1000.0, 1, 0.0);
        n.ai[0] = 19.0; // → 20, but the wall blocks CanHit.
        let mut rng = UnifiedRandom::new(7);
        let before = rng.peek();
        let shot = bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert!(shot.is_none(), "no line of sight, no sickle");
        assert_eq!(n.ai[0], 20.0);
        assert_eq!(rng.peek(), before, "the shot set skips the reset roll");
    }

    /// The RNG-free witness: the three bat kinds (49/51/60) never touch
    /// `Main.rand` anywhere in the arm — 600 ticks each (through the wet
    /// lift, the drift cycle and its resets), the stream does not move.
    #[test]
    fn s1_d_3_s14_the_bats_draw_no_rng() {
        let w = air();
        let players = [cand(1100.0, 1000.0)];
        let wet = [false];
        for kind in [NpcKind::CaveBat, NpcKind::JungleBat, NpcKind::Hellbat] {
            let mut n = Npc::new(kind, 1000.0, 1000.0, 1, 0.0);
            let mut rng = UnifiedRandom::new(99);
            let before = rng.peek();
            for tick in 0..600 {
                n.wet = tick % 120 < 30; // exercise the wet arm too
                assert!(bat_ai(&mut n, &env(&w, &players, &wet), &mut rng).is_none());
            }
            assert_eq!(rng.peek(), before, "{:?} drew from Main.rand", kind);
        }
    }

    /// `NPC.cs:23410-23413` — the client (netMode 1) returns before the
    /// shot arms: ai[0] never increments, no shot, no draw; the drift
    /// (ai[1]) still runs.
    #[test]
    fn s1_d_3_s14_the_client_returns_before_the_shot_arm() {
        let w = air();
        let players = [cand(1100.0, 1000.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::VoodooDemon, 1000.0, 1000.0, 1, 0.0);
        n.ai[0] = 19.0;
        let mut e = env(&w, &players, &wet);
        e.net_mode = 1;
        let mut rng = UnifiedRandom::new(7);
        let before = rng.peek();
        assert!(bat_ai(&mut n, &e, &mut rng).is_none());
        assert_eq!(n.ai[0], 19.0, "the client never counts the shot timer");
        assert_eq!(n.ai[1], 1.0, "the drift runs on the client too");
        assert_eq!(rng.peek(), before);
    }

    /// `NPC.cs:22897` → `NPC.cs:77652-77781` — the shared selection runs
    /// every tick: the rect tracks the player, the facing points at it,
    /// and the ticker stamps `targetSetFrame` (`:77780`).
    #[test]
    fn s1_d_3_s14_the_target_selection_stamps_the_ticker() {
        let w = air();
        let players = [cand(900.0, 900.0)];
        let wet = [false];
        let mut n = Npc::new(NpcKind::CaveBat, 1000.0, 1000.0, 1, 0.0);
        let mut rng = UnifiedRandom::new(1);
        bat_ai(&mut n, &env(&w, &players, &wet), &mut rng);
        assert_eq!(n.target, 0);
        assert_eq!(n.target_rect, (900, 900, 20, 42));
        assert_eq!(n.facing, -1, "the player is to the left");
        assert_eq!(n.direction_y, -1, "and above");
        assert_eq!(n.target_set_frame, 7);
    }

    /// `NPC.cs:55835-55841` — the firing window is the int-truncated
    /// half-open rect: ±910 px X / ±550 px Y around the truncated target
    /// centre, right/bottom edges exclusive.
    #[test]
    fn s1_d_3_s14_the_firing_window_is_the_inflated_view_rect() {
        // Target centre (1000, 1000): rect x 90..1910, y 450..1550.
        assert!(global_firing_distance_check(1000.0, 1000.0, 1000.0, 1000.0));
        assert!(global_firing_distance_check(90.0, 450.0, 1000.0, 1000.0));
        assert!(!global_firing_distance_check(89.0, 1000.0, 1000.0, 1000.0));
        assert!(global_firing_distance_check(1909.0, 1549.0, 1000.0, 1000.0));
        assert!(
            !global_firing_distance_check(1910.0, 1000.0, 1000.0, 1000.0),
            "the right edge is exclusive (XNA Contains)"
        );
        // The truncation: a .9 centre floors before the rect is built.
        assert!(global_firing_distance_check(1909.0, 1000.0, 1000.9, 1000.0));
    }
}
