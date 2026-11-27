//! W4-d24 (aiStyle-18) — the jellyfish/swimmer family, `NPC.AI`'s
//! `if (this.aiStyle == 18)` block (`NPC.cs:24153-24406`), ported 1:1 as
//! a PURE velocity machine (code-only mode: no compiler/tests were run
//! on this branch; the witness tests below are written, not executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 18` rows, whole-file grep)
//!
//! SIX vanilla members:
//! * BlueJellyfish 63 (`NPC.cs:9582-9595`: 26×26, dmg 25, def 4, 34 HP,
//!   NPCHit25/NPCDeath28, value 100, alpha 20, noGravity);
//! * PinkJellyfish 64 (`:9596-9609`: 26×26, dmg 30, def 6, 70 HP,
//!   NPCHit25/NPCDeath28, value 100, alpha 20, noGravity);
//! * GreenJellyfish 103 (`:10216-10229`: 26×26, dmg 80, def 30, 120 HP,
//!   NPCHit25/NPCDeath28, value 800, alpha 20, noGravity);
//! * Squid 221 (`:11840-11854`: 26×26, dmg 20, def 2, 30 HP,
//!   NPCHit1/NPCDeath1, value 100, alpha 20, rarity 1, noGravity);
//! * BloodJelly 242 (`:12119-12132`: 26×26, dmg 75, def 20, 150 HP,
//!   NPCHit13/NPCDeath19, value 800, alpha 20, noGravity);
//! * FungoFish 256 (`:12289-12303`: 26×26, dmg 90, def 20, 140 HP,
//!   NPCHit1/NPCDeath6, value 1200, alpha 20, npcSlots 0.3, noGravity).
//! None is ported yet — the module lands UNWIRED (the
//! `npc_ai_style17.rs` precedent): pure fns keyed on the vanilla type,
//! reachable through [`ai_018_arm`]'s `ty` parameter before the kinds
//! land. The wiring step adds the kinds + the style-axis claim.
//!
//! ## The machine
//!
//! * the expert zap-shield (`:24155-24203`): `flag26 = wet && ai[1]==1`
//!   is the shielded state (`dontTakeDamage` stays armed); the expert
//!   63/64/103/242 rows run the proximity charge (the wet+alive+CanHit+
//!   <150-px player gate adds +2 idle / −0.25 shielded per tick), the
//!   120-tick shield / 420-tick recharge cycle, and the dry reset.
//! * the glow (`:24204-24220`): `num263 = 1` (+0.5 shielded); the 63
//!   blue / 103 green / generic pink triples at the INTEGER-half-height
//!   centre (the C# uses `height / 2` on BOTH axes — the quirk is
//!   verbatim); 221/242 light nothing.
//! * the shield early-return (`:24225-24228`) after the direction-0
//!   `TargetClosest`.
//! * the wet machine (`:24229-24389`): the feet-slope flips (the
//!   `Center.X` int division), the collideX/Y bounces (the Y bounce
//!   arms `directionY`/`ai[0]`), the `flag27` chase gate
//!   (`!friendly` + `TargetClosest(faceTarget:false)` + the player
//!   wet/alive/CanHit), the chase damp ladder (0.98 base; 103 ×0.98
//!   again + 0.6 band; 221 ×0.99 + 1 band; 242 ×0.995 + 3 band) with
//!   the slow-speed 7/9-speed dash (the 221 ink `localAI[0]=1` latch),
//!   else the idle drift: the 0.02 X accel, the `vx*0.4` tilt, the ±1
//!   damp band, the `ai[0]` ±0.01 bob flip, the liquid>128 headroom
//!   probe and the double-promoted ±1.2 damp.
//! * the dry flop (`:24390-24405`): the `vx*0.1` spin, the grounded
//!   0.98 decay with the double-promoted ±0.01 snap, the 0.2 gravity
//!   with the 10 cap, `ai[0] = 1`.
//!
//! Seam: writes velocity, `ai[0..2]`, `localAI[0]`/`localAI[2]`,
//! `direction`/`directionY`, `rotation`, `dontTakeDamage`; reads
//! `wet`/`collideX`/`collideY` a tick late (the collision pass sets
//! them, `NPC.cs:92538+`). Does NOT integrate position. The
//! `Lighting.AddLight` leaves as [`Ai18Effects::light`]; the 221 ink
//! `localAI[0]` is NPC state (the FindFrame/HitEffect reader is the
//! wiring's). `Main.netMode` folds single-player; `Main.expertMode`
//! rides [`Ai18Ctx::expert`].
//!
//! ## RNG by construction
//!
//! The arm draws NO `Main.rand` — `NPC.cs:24153-24406` contains no
//! `Next`/`NextFloat`; neither does `TargetClosest`
//! (`NPC.cs:77652-77781`, the `npc_target.rs` port) nor `Collision.
//! CanHit` (`Collision.cs:1073-1092`, the `projectile::can_hit` port).
//! The witness `the_style18_arm_draws_no_rng` pins a `UnifiedRandom`
//! state across the shield, chase, idle and dry branches.
#![allow(dead_code)] // UNWIRED (W4-d24): the six kinds + the axis claim are the wiring step

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The per-tick context the arm reads besides the NPC and the world
/// (the `npc_ai_style16.rs::Ai16Ctx` shape).
pub struct Ai18Ctx<'a> {
    /// The `Main.player` slots as `TargetClosest` sees them.
    pub players: &'a [TargetCandidate],
    /// `Main.player[target].wet` (`:24168`/`:24283`) — the candidate
    /// struct carries `dead` but not `wet`.
    pub player_wet: bool,
    /// `Main.expertMode` (`:24164`) — the zap-shield rows.
    pub expert: bool,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp.
    pub ticker: u64,
}

/// The side effects the pure arm cannot apply itself.
#[derive(Default, Debug, PartialEq)]
pub struct Ai18Effects {
    /// The `Lighting.AddLight` (`:24211`/`:24215`/`:24219`):
    /// `(tileX, tileY, [r, g, b])`, the C#'s int-truncated coordinates.
    pub light: Option<(i32, i32, [f32; 3])>,
}

/// One member's `SetDefaults` row (the census above).
pub struct JellyDefaults {
    pub ty: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub hit_sound: (i32, i32),
    pub death_sound: (i32, i32),
    pub value: f32,
}

/// The six rows, verbatim (`NPC.cs:9582`/`:9596`/`:10216`/`:11840`/
/// `:12119`/`:12289`). All are 26×26, aiStyle 18, alpha 20, noGravity.
pub const JELLY_ROWS: [JellyDefaults; 6] = [
    JellyDefaults { ty: 63, damage: 25, defense: 4, life_max: 34, hit_sound: (3, 25), death_sound: (4, 28), value: 100.0 },
    JellyDefaults { ty: 64, damage: 30, defense: 6, life_max: 70, hit_sound: (3, 25), death_sound: (4, 28), value: 100.0 },
    JellyDefaults { ty: 103, damage: 80, defense: 30, life_max: 120, hit_sound: (3, 25), death_sound: (4, 28), value: 800.0 },
    JellyDefaults { ty: 221, damage: 20, defense: 2, life_max: 30, hit_sound: (3, 1), death_sound: (4, 1), value: 100.0 },
    JellyDefaults { ty: 242, damage: 75, defense: 20, life_max: 150, hit_sound: (3, 13), death_sound: (4, 19), value: 800.0 },
    JellyDefaults { ty: 256, damage: 90, defense: 20, life_max: 140, hit_sound: (3, 1), death_sound: (4, 6), value: 1200.0 },
];

/// The target slot's candidate (the `npc_ai_style16.rs::target_of`
/// fallback — an unset/stale slot reads slot 0).
fn target_of<'a>(npc: &Npc, players: &'a [TargetCandidate]) -> &'a TargetCandidate {
    let i = if npc.target >= 0 && (npc.target as usize) < players.len() {
        npc.target as usize
    } else {
        0
    };
    &players[i]
}

/// `Tile.topSlope()` (`Tile.cs:290-298`): slope 1 or 2.
fn top_slope(t: &tr_core::tile::Tile) -> bool {
    matches!(t.slope(), 1 | 2)
}

/// `Tile.leftSlope()` (`Tile.cs:310-318`): slope 2 or 4.
fn left_slope(t: &tr_core::tile::Tile) -> bool {
    matches!(t.slope(), 2 | 4)
}

/// The aiStyle-18 arm (`NPC.cs:24153-24406`) for one NPC, one tick,
/// keyed on the vanilla type (the family's six members share the
/// machine; the per-type rows are the branches below). RNG-free.
pub fn ai_018_arm(ty: i32, npc: &mut Npc, world: &World, ctx: &Ai18Ctx) -> Ai18Effects {
    let mut fx = Ai18Effects::default();
    let (w, h) = npc.size();
    // `:24155-24163` — flag26: the wet zap-shield state.
    let flag26 = npc.wet && npc.ai[1] == 1.0;
    if !flag26 {
        npc.dont_take_damage = false;
    }
    // `:24164-24203` — the expert shield cycle (63/64/103/242 only).
    if ctx.expert && (ty == 63 || ty == 64 || ty == 103 || ty == 242) {
        if npc.wet {
            // `:24168` — the proximity gate: target >= 0, the player
            // wet + alive + CanHit + the <150-px centre distance.
            if npc.target >= 0 {
                let p = target_of(npc, ctx.players);
                let (pcx, pcy) = (p.x + p.width as f32 * 0.5, p.y + p.height as f32 * 0.5);
                let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
                let dist = ((pcx - cx).powi(2) + (pcy - cy).powi(2)).sqrt();
                if ctx.player_wet
                    && !p.dead
                    && crate::projectile::can_hit(
                        world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
                    )
                    && dist < 150.0
                {
                    if npc.ai[1] == 0.0 {
                        npc.ai[2] += 2.0;
                    } else {
                        npc.ai[2] -= 0.25;
                    }
                }
            }
            if flag26 {
                // `:24179-24187` — shielded: invulnerable, 120-tick end.
                npc.dont_take_damage = true;
                npc.ai[2] += 1.0;
                if npc.ai[2] >= 120.0 {
                    npc.ai[1] = 0.0;
                }
            } else {
                // `:24188-24196` — charging: the 420-tick arm.
                npc.ai[2] += 1.0;
                if npc.ai[2] >= 420.0 {
                    npc.ai[1] = 1.0;
                    npc.ai[2] = 0.0;
                }
            }
        } else {
            // `:24198-24202` — dry: both counters reset.
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
        }
    }
    // `:24204-24220` — the glow. num263 = 1 (+0.5 shielded). The C#
    // centre uses `height / 2` on BOTH axes (the INTEGER half-height
    // quirk — `:24211` reads `(float)(base.height / 2)` for X too).
    let mut num263 = 1.0f32;
    if flag26 {
        num263 += 0.5;
    }
    let half_h = (h as i32 / 2) as f32;
    let li = (npc.x + half_h) as i32 / 16;
    let lj = (npc.y + half_h) as i32 / 16;
    if ty == 63 {
        fx.light = Some((li, lj, [0.05 * num263, 0.15 * num263, 0.4 * num263]));
    } else if ty == 103 {
        fx.light = Some((li, lj, [0.05 * num263, 0.45 * num263, 0.1 * num263]));
    } else if ty != 221 && ty != 242 {
        fx.light = Some((li, lj, [0.35 * num263, 0.05 * num263, 0.2 * num263]));
    }
    // `:24221-24224` — the direction-0 re-aim.
    if npc.facing == 0 {
        target_closest(npc, world, ctx.players, true, 18, ctx.ticker);
    }
    // `:24225-24228` — the shielded early return.
    if flag26 {
        return fx;
    }
    if npc.wet {
        // `:24231-24258` — the feet-slope flips: `num264 = (int)Center.X
        // / 16` (the FLOAT half-width centre, then int), `num265 =
        // (int)(position.Y + height) / 16`.
        let num264 = (npc.x + w * 0.5) as i32 / 16;
        let num265 = (npc.y + h) as i32 / 16;
        let foot = world
            .tile(num264, num265)
            .filter(|t| top_slope(t))
            .or_else(|| world.tile(num264, num265 + 1).filter(|t| top_slope(t)));
        if let Some(t) = foot {
            if left_slope(t) {
                npc.facing = -1;
                npc.vx = npc.vx.abs() * -1.0;
            } else {
                npc.facing = 1;
                npc.vx = npc.vx.abs();
            }
        }
        // `:24259-24263` — the X bounce flips the facing too.
        if npc.collide_x {
            npc.vx *= -1.0;
            npc.facing *= -1;
        }
        // `:24264-24278` — the Y bounce arms directionY + ai[0].
        if npc.collide_y {
            if npc.vy > 0.0 {
                npc.vy = npc.vy.abs() * -1.0;
                npc.direction_y = -1;
                npc.ai[0] = -1.0;
            } else if npc.vy < 0.0 {
                npc.vy = npc.vy.abs();
                npc.direction_y = 1;
                npc.ai[0] = 1.0;
            }
        }
        // `:24279-24287` — flag27: the chase gate.
        let mut flag27 = false;
        if !npc.friendly {
            target_closest(npc, world, ctx.players, false, 18, ctx.ticker);
            let p = target_of(npc, ctx.players);
            if ctx.player_wet
                && !p.dead
                && crate::projectile::can_hit(
                    world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
                )
            {
                flag27 = true;
            }
        }
        if flag27 {
            // `:24288-24330` — the chase: the aim tilt, the damp ladder.
            npc.local_ai[2] = 1.0;
            npc.rotation = npc.vy.atan2(npc.vx) + 1.57;
            npc.vx *= 0.98;
            npc.vy *= 0.98;
            let mut num266 = 0.2f32;
            if ty == 103 {
                npc.vx *= 0.98;
                npc.vy *= 0.98;
                num266 = 0.6;
            }
            if ty == 221 {
                npc.vx *= 0.99;
                npc.vy *= 0.99;
                num266 = 1.0;
            }
            if ty == 242 {
                npc.vx *= 0.995;
                npc.vy *= 0.995;
                num266 = 3.0;
            }
            // `:24309-24329` — the slow-speed dash.
            if npc.vx > -num266 && npc.vx < num266 && npc.vy > -num266 && npc.vy < num266 {
                if ty == 221 {
                    // `:24311-24314` — the Squid's ink latch.
                    npc.local_ai[0] = 1.0;
                }
                target_closest(npc, world, ctx.players, true, 18, ctx.ticker);
                let mut num267 = 7.0f32;
                if ty == 103 {
                    num267 = 9.0;
                }
                let p = target_of(npc, ctx.players);
                let (vx26, vy26) = (npc.x + w * 0.5, npc.y + h * 0.5);
                let mut num268 = p.x + (p.width / 2) as f32 - vx26;
                let mut num269 = p.y + (p.height / 2) as f32 - vy26;
                let num270 = num267 / (num268 * num268 + num269 * num269).sqrt();
                num268 *= num270;
                num269 *= num270;
                npc.vx = num268;
                npc.vy = num269;
            }
            return fx;
        }
        // `:24332-24338` — the idle drift.
        npc.local_ai[2] = 0.0;
        npc.vx += npc.facing as f32 * 0.02;
        npc.rotation = npc.vx * 0.4;
        if npc.vx < -1.0 || npc.vx > 1.0 {
            npc.vx *= 0.95;
        }
        // `:24339-24354` — the ai[0] bob flip.
        if npc.ai[0] == -1.0 {
            npc.vy -= 0.01;
            if npc.vy < -1.0 {
                npc.ai[0] = 1.0;
            }
        } else {
            npc.vy += 0.01;
            if npc.vy > 1.0 {
                npc.ai[0] = -1.0;
            }
        }
        // `:24355-24383` — the headroom probe: the INTEGER-half centre,
        // liquid>128 above + a solid below arms the rise; else sink.
        let num271 = (npc.x + (w as i32 / 2) as f32) as i32 / 16;
        let num272 = (npc.y + (h as i32 / 2) as f32) as i32 / 16;
        let above_wet = world.tile(num271, num272 - 1).is_some_and(|t| t.liquid > 128);
        if above_wet {
            if world.tile(num271, num272 + 1).is_some_and(|t| t.active()) {
                npc.ai[0] = -1.0;
            } else if world.tile(num271, num272 + 2).is_some_and(|t| t.active()) {
                npc.ai[0] = -1.0;
            }
        } else {
            npc.ai[0] = 1.0;
        }
        // `:24384-24387` — the double-promoted ±1.2 damp.
        if (npc.vy as f64) > 1.2 || (npc.vy as f64) < -1.2 {
            npc.vy *= 0.99;
        }
        return fx;
    }
    // `:24390-24405` — the dry flop.
    npc.rotation += npc.vx * 0.1;
    if npc.vy == 0.0 {
        npc.vx *= 0.98;
        if (npc.vx as f64) > -0.01 && (npc.vx as f64) < 0.01 {
            npc.vx = 0.0;
        }
    }
    npc.vy += 0.2;
    if npc.vy > 10.0 {
        npc.vy = 10.0;
    }
    npc.ai[0] = 1.0;
    fx
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation (`NPC.cs:24153-24406`).
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

    /// A stand-in body with the family's 26×26 box (the kinds are
    /// unported; the machine keys on the `ty` parameter).
    fn jelly(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((26.0, 26.0));
        n.hp = 34;
        n.max_hp = 34;
        n
    }

    fn ctx<'a>(players: &'a [TargetCandidate], player_wet: bool, expert: bool) -> Ai18Ctx<'a> {
        Ai18Ctx { players, player_wet, expert, ticker: 0 }
    }

    /// The six `SetDefaults` rows (the census).
    #[test]
    fn the_defaults_rows_match_the_census() {
        let want = [
            (63, 25, 4, 34, (3, 25), (4, 28), 100.0),
            (64, 30, 6, 70, (3, 25), (4, 28), 100.0),
            (103, 80, 30, 120, (3, 25), (4, 28), 800.0),
            (221, 20, 2, 30, (3, 1), (4, 1), 100.0),
            (242, 75, 20, 150, (3, 13), (4, 19), 800.0),
            (256, 90, 20, 140, (3, 1), (4, 6), 1200.0),
        ];
        for (r, wnt) in JELLY_ROWS.iter().zip(want) {
            assert_eq!((r.ty, r.damage, r.defense, r.life_max), (wnt.0, wnt.1, wnt.2, wnt.3));
            assert_eq!(r.hit_sound, wnt.4);
            assert_eq!(r.death_sound, wnt.5);
            assert_eq!(r.value, wnt.6);
        }
    }

    /// `:24204-24220` — the glow triples: the 63 blue, the 103 green,
    /// the generic pink; 221/242 none. The centre is the INTEGER
    /// half-HEIGHT on both axes (the C# quirk).
    #[test]
    fn the_glow_rows_and_the_half_height_quirk() {
        let w = air();
        let p = [cand(1400.0, 1000.0)];
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        let fx = ai_018_arm(63, &mut n, &w, &ctx(&p, false, false));
        // (1000 + 13) / 16 = 63 on BOTH axes.
        assert_eq!(fx.light, Some((63, 63, [0.05, 0.15, 0.4])));
        let fx = ai_018_arm(103, &mut n, &w, &ctx(&p, false, false));
        assert_eq!(fx.light, Some((63, 63, [0.05, 0.45, 0.1])));
        let fx = ai_018_arm(64, &mut n, &w, &ctx(&p, false, false));
        assert_eq!(fx.light, Some((63, 63, [0.35, 0.05, 0.2])));
        for ty in [221, 242] {
            let fx = ai_018_arm(ty, &mut n, &w, &ctx(&p, false, false));
            assert_eq!(fx.light, None, "type {ty} lights nothing");
        }
    }

    /// `:24155-24163` + `:24225-24228` — the shielded state: wet +
    /// ai[1]==1 keeps dontTakeDamage, glows ×1.5 and returns early (no
    /// velocity writes).
    #[test]
    fn the_shield_holds_and_returns_early() {
        let w = air();
        let p = [cand(1010.0, 1000.0)];
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        n.wet = true;
        n.ai[1] = 1.0;
        n.dont_take_damage = true;
        n.vx = 1.23;
        let fx = ai_018_arm(63, &mut n, &w, &ctx(&p, true, false));
        assert!(n.dont_take_damage, "the shield stays armed");
        assert_eq!(n.vx, 1.23, "the early return skips the machine");
        // `:24225-24228` — the ×1.5 glow on the f32 base triplet.
        let want = [0.05f32 * 1.5, 0.15 * 1.5, 0.4 * 1.5];
        assert_eq!(fx.light.unwrap().2, want, "the ×1.5 glow");
        // Not shielded: dontTakeDamage clears.
        n.ai[1] = 0.0;
        ai_018_arm(63, &mut n, &w, &ctx(&p, true, false));
        assert!(!n.dont_take_damage);
    }

    /// `:24164-24203` — the expert cycle: the proximity charge (+2),
    /// the 420-tick arm, the 120-tick shield, the dry reset.
    #[test]
    fn the_expert_zap_cycle() {
        let w = air();
        let p = [cand(1010.0, 1000.0)]; // inside 150 px, CanHit in air
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        n.wet = true;
        n.target = 0;
        // The proximity charge: ai[1]==0 → +2, then the idle +1 = 3.
        ai_018_arm(63, &mut n, &w, &ctx(&p, true, true));
        assert_eq!(n.ai[2], 3.0, "+2 proximity + the idle +1");
        // The 420 arm.
        n.ai[2] = 419.0;
        n.ai[1] = 0.0;
        ai_018_arm(63, &mut n, &w, &ctx(&p, false, true));
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.ai[2], 0.0);
        // The shielded tick: +1, invulnerable; at 120 the shield ends.
        n.ai[2] = 119.0;
        ai_018_arm(63, &mut n, &w, &ctx(&p, false, true));
        assert!(n.dont_take_damage);
        assert_eq!(n.ai[1], 0.0, "the 120-tick shield end");
        // The dry reset.
        n.wet = false;
        n.ai[1] = 1.0;
        n.ai[2] = 50.0;
        ai_018_arm(63, &mut n, &w, &ctx(&p, false, true));
        assert_eq!((n.ai[1], n.ai[2]), (0.0, 0.0));
        // The non-expert / non-zap kinds skip the whole block.
        let mut m = jelly(1000.0, 1000.0);
        m.wet = true;
        m.ai[2] = 419.0;
        ai_018_arm(256, &mut m, &w, &ctx(&p, true, true));
        assert_eq!(m.ai[1], 0.0, "256 has no zap rows");
    }

    /// `:24259-24278` — the wet bounces: collideX flips vx AND facing;
    /// the collideY sign arms directionY + ai[0].
    #[test]
    fn the_wet_bounces_arm_the_bob() {
        let w = air();
        let p = [cand(1400.0, 1000.0)];
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        n.wet = true;
        n.collide_x = true;
        n.vx = 0.5;
        n.collide_y = true;
        n.vy = 0.5;
        ai_018_arm(64, &mut n, &w, &ctx(&p, false, false));
        assert_eq!(n.facing, -1, "the X bounce flips the facing");
        assert_eq!(n.direction_y, -1);
        // ai[0]: the bounce armed −1, the bob step then ran (vy −0.01),
        // and the dry-headroom probe re-arms +1 (no liquid above).
        assert_eq!(n.ai[0], 1.0, "the headroom probe wins in air");
        assert!(n.vx < 0.0, "the X reflection + the −1 drift");
    }

    /// `:24288-24330` — the chase: the damp ladder and the slow-speed
    /// dash (the 7-speed aim; the 103's 9; the 221 ink latch).
    #[test]
    fn the_chase_damps_and_dashes() {
        let w = air();
        let p = [cand(1100.0, 1000.0)];
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        n.wet = true;
        n.vx = 0.1;
        n.vy = 0.0;
        let fx = ai_018_arm(64, &mut n, &w, &ctx(&p, true, false));
        // The dash fired: the aim from (1013,1013) to (1110,1021) at 7.
        let dx = 1110.0f32 - 1013.0;
        let dy = 1021.0f32 - 1013.0;
        let s = 7.0 / (dx * dx + dy * dy).sqrt();
        assert!((n.vx - dx * s).abs() < 1e-4, "the 7-speed dash ({})", n.vx);
        assert!((n.vy - dy * s).abs() < 1e-4);
        assert_eq!(n.local_ai[2], 1.0, "the chase latch");
        assert!(fx.light.is_some());
        // The Squid's ink latch + the 1-wide band.
        let mut m = jelly(1000.0, 1000.0);
        m.facing = 1;
        m.wet = true;
        m.vx = 0.5; // inside the 221's ±1 band after ×0.98×0.99
        ai_018_arm(221, &mut m, &w, &ctx(&p, true, false));
        assert_eq!(m.local_ai[0], 1.0, "the ink latch (:24313)");
        // The 103's double damp: 0.98 × 0.98.
        let mut g = jelly(1000.0, 1000.0);
        g.facing = 1;
        g.wet = true;
        g.vx = 2.0; // outside the 0.6 band — no dash
        g.vy = 2.0;
        ai_018_arm(103, &mut g, &w, &ctx(&p, true, false));
        assert!((g.vx - 2.0 * 0.98 * 0.98).abs() < 1e-5, "the double damp");
    }

    /// `:24332-24387` — the idle drift: the 0.02 accel, the vx*0.4
    /// tilt, the ±1 damp band, the bob flip and the headroom probe.
    #[test]
    fn the_idle_drift_and_headroom() {
        let w = air();
        let p = [cand(1400.0, 1000.0)];
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        n.wet = true;
        n.vx = 0.0;
        n.vy = 0.0;
        n.ai[0] = 1.0;
        ai_018_arm(64, &mut n, &w, &ctx(&p, false, false));
        assert!((n.vx - 0.02).abs() < 1e-6, "the 0.02 accel");
        assert!((n.rotation - 0.008).abs() < 1e-6, "vx * 0.4");
        assert!((n.vy - 0.01).abs() < 1e-6, "the +0.01 bob");
        assert_eq!(n.local_ai[2], 0.0);
        assert_eq!(n.ai[0], 1.0, "no liquid above → sink");
        // The ±1 damp band: vx 1.5 → +0.02 then ×0.95.
        let mut m = jelly(1000.0, 1000.0);
        m.facing = 1;
        m.wet = true;
        m.vx = 1.5;
        ai_018_arm(64, &mut m, &w, &ctx(&p, false, false));
        assert!((m.vx - 1.52 * 0.95).abs() < 1e-5, "the over-band damp");
    }

    /// `:24390-24405` — the dry flop: the spin, the grounded decay +
    /// snap, the 0.2 gravity / 10 cap, ai[0] = 1.
    #[test]
    fn the_dry_flop_decays_and_falls() {
        let w = air();
        let p = [cand(1400.0, 1000.0)];
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        n.wet = false;
        n.vx = 0.005;
        n.vy = 0.0;
        n.rotation = 0.0;
        ai_018_arm(64, &mut n, &w, &ctx(&p, false, false));
        assert_eq!(n.vx, 0.0, "the ±0.01 snap");
        assert_eq!(n.vy, 0.2, "the 0.2 gravity");
        assert_eq!(n.ai[0], 1.0);
        assert!((n.rotation - 0.0005).abs() < 1e-6, "the PRE-decay spin");
        // The fall cap.
        let mut m = jelly(1000.0, 1000.0);
        m.facing = 1;
        m.vy = 9.95;
        ai_018_arm(64, &mut m, &w, &ctx(&p, false, false));
        assert_eq!(m.vy, 10.0, "the 10 cap");
    }

    /// The RNG census: every branch (shield, chase, idle, dry) draws
    /// NOTHING — a pinned `UnifiedRandom` state survives them all.
    #[test]
    fn the_style18_arm_draws_no_rng() {
        let w = air();
        let mut rng = UnifiedRandom::new(99);
        let witness = rng.clone();
        let p = [cand(1010.0, 1000.0)];
        let mut n = jelly(1000.0, 1000.0);
        n.facing = 1;
        for ty in [63, 64, 103, 221, 242, 256] {
            for wet in [false, true] {
                for expert in [false, true] {
                    n.wet = wet;
                    n.ai[1] = 0.0;
                    ai_018_arm(ty, &mut n, &w, &ctx(&p, true, expert));
                    n.ai[1] = 1.0;
                    ai_018_arm(ty, &mut n, &w, &ctx(&p, true, expert));
                }
            }
        }
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the arm");
    }
}
