//! W4-f18 (aiStyle-7, the town-CRITTER path) — the Frog 361, the Duck 362
//! and the DuckWhite 364 (the Scorpion pair 366/367 lands with batch 3).
//! `NPC.AI`'s `if (this.aiStyle == 7)` dispatch into
//! `AI_007_TownEntities` (`NPC.cs:52965-55561`), the `NPCID.Sets.TownCritter`
//! subset (`ID/NPCID.cs:10459`), ported 1:1 as a PURE velocity machine
//! (code-only mode: no compiler/tests were run on this branch; the witness
//! tests below are written, not executed).
//!
//! The full town machine lives in `npc_ai_town.rs` (UNWIRED — it needs the
//! door-ops / partner-write / shot seams the town NPCs carry). The
//! critters never talk, attack, open doors or teleport home, so this
//! module ports ONLY the branches a `TownCritter` reaches, reusing the
//! town module's shared tile/collision helpers (`get_walk_prediction`,
//! `find_good_resting_spot`, `town_step_up`, `step_down`, `solid_tiles*`,
//! `drown_collision`). The kinds route here through the style-axis 7 arm
//! (`npc.rs`); the town NPCs keep their live `update_guide` stand-in until
//! the full town wiring lands.
//!
//! ## The critter path (every cited branch a TownCritter reaches)
//!
//! * The head (`:53278-53295`): a critter with no target (`target == 255`)
//!   selects one and faces it; the home tile stamps on the first grounded
//!   tick (`homeTileX == -1`). `directionY = -1` (`:53308`), a zero
//!   direction latches +1 (`:53309-53312`).
//! * `flag` (`:52982-52998`) = raining OR night OR eclipse OR slimeRain OR
//!   (storming above the surface) — the "go home" weather gate.
//! * The flag census for the ported critters (`:53451-53459`): flag8
//!   (300/447/610) false, flag9 (616/617/625) false, flag10 (361/445/687)
//!   TRUE for the Frog only, flag11 (IsTownSlime) false, flag12 =
//!   flag9|flag10 (the Frog breathes underwater), flag14/flag15 (flag11)
//!   false. flag5 (a player talks to it) false — critters are not talkable.
//!   flag16/flag17 (the danger scan, `:53479-53560`) read the sibling NPC
//!   pool; the SOLO path passes an empty pool so both fold false (the
//!   `npc_ai_style65` threat-scan deviation — the sim's pooled seam lands
//!   with the wiring follow-up).
//! * State 0 idle (`:53622-53786`): the localAI[3] countdown; the Frog's
//!   wet flee (`flag10 && wet`, `:53641-53648`); the else-branch's 0.1
//!   vx decay (`:53697-53712`), the ai[1] countdown, the drown check
//!   (`:53725-53736`, skipped for the Frog — flag12), the avoidFalling
//!   walk-start (`:53741-53761`), and the home-band direction flip /
//!   `Next(80)` 200-tick turn (`:53764-53786`).
//! * State 1 walk (`:53790-54244`): the resting-spot return is
//!   `!TownCritter`-gated (`:53792`, skipped); the home-band ai[1] drain
//!   (`:53804-53815`), the walk-end idle arm (`:53817-53832`, the critter
//!   `-= Next(100)`), the cap-1 / accel-0.07 walk (`:53865-53950`, the
//!   Frog's wet swim `:53917-53927`), the StepDown/StepUp (`:53951-53966`),
//!   the obstacle jump ladder -6/-5/-4.4 (`:54083-54162`), the
//!   keepwalking/avoidFalling flips, the critter jump boost ×1.2
//!   (`:54191-54194`), and the 30-tick stuck detector (`:54226-54242`).
//!
//! ## Documented folds / remainders (all cited)
//!
//! * `Main.netMode != 1` → true (single-player) — every host gate runs.
//! * The door open/close (`:53833-53864` / `:54043-54080`) is
//!   `townNPC`-gated — critters are not townNPCs, so it never runs.
//! * The teleport-home (`:53423-53450`) is `!homeless && townNPC`-gated —
//!   skipped. The pet idle animations (`:53672-53675` / `:53715-53718`)
//!   are `IsTownPet`-gated — skipped. The sitting (`AI_007_TryForcingSitting`)
//!   is `!TownCritter`-gated (`:53649`) — skipped.
//! * The danger scan's sibling pool is empty on the solo path (above);
//!   the flee hand-off (`TownPartnerWrite::Flee`) and the stinky /
//!   attack-type branches are town-NPC-only.
//! * `Main.IsItStorming` — the port's `World` carries no storm flag yet;
//!   the term folds false (the rain/night/eclipse/slimeRain terms stand).
//! * The integration tail is `UpdateNPC`'s post-AI gravity + the port's
//!   flyer-collision stand-in (`npc.rs::integrate_flyer_collision`), the
//!   `ai_066_crawler_family` shape — the C#'s `UpdateCollision` slope pass
//!   granularity is the established bat/fish reconcile deviation.
//!
//! ## RNG by construction
//!
//! The arm's `Main.rand` draws, in per-tick C# order (all on the caller's
//! `UnifiedRandom`): the idle walk-start `Next(300)` + the critter
//! `Next(200, 400)` (`:53746-53751`) OR the avoidFalling turn `Next(120)`
//! (`:53758`); the idle `Next(80)` turn (`:53782`); the walk-end
//! `Next(300)` + the critter `Next(100)` (`:53820-53824`); the
//! avoidFalling walk-stop `Next(50)` (`:54013`); the stuck-detector
//! `Next(300)` + the critter `Next(200, 400)` (`:54232-54236`). The Frog's
//! wet flee `Next(500, 700)` (`:53644`). `TargetClosest` and the tile
//! probes draw NOTHING.
#![allow(dead_code)] // the 366/367 scorpions + the pooled danger seam land with batch 3

use crate::npc::Npc;
use crate::npc_ai_town::{
    self as town, get_walk_prediction, find_good_resting_spot,
};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::unified_random::UnifiedRandom;
use tr_core::World;

/// `NPCID.Sets.TownCritter` members this module ports (the census above).
pub fn is_ported_critter(net_id: i32) -> bool {
    matches!(net_id, 361 | 362 | 364 | 366 | 367)
}

/// The per-tick context the arm reads besides the NPC and the world.
pub struct TownCritterCtx<'a> {
    /// The `Main.player` slots — feeds the head's `TargetClosest`
    /// (`:53280`). The port has one session player.
    pub players: &'a [TargetCandidate],
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`
    /// (`NPC.cs:77780`) — 0 until the ticker is plumbed (S1-D-3p).
    pub ticker: u64,
}

/// `NPC.UpdateHomeTileState` (`NPC.cs:52904-52914`) — the home-tile write.
/// The port's `Npc.home` folds homeless + the tiles (the town module's
/// seam); the message-60 send is the wiring's (RNG-free either way).
fn update_home_tile_state(npc: &mut Npc, x: i32, y: i32) {
    npc.home = Some((x, y));
}

/// One fixed step of the aiStyle-7 town-critter path
/// (`NPC.cs:52965-55561`, the TownCritter subset), in C# order. `npc`
/// must carry the kind's `SetDefaults` box (`kind.size()`). The machine
/// writes velocity / ai / localAI / direction / home; it does NOT
/// integrate position (the caller runs the gravity + collision tail).
pub fn town_critter_step(
    npc: &mut Npc,
    world: &World,
    ctx: &TownCritterCtx,
    rng: &mut UnifiedRandom,
) {
    let net_id = npc.kind.net_id();
    debug_assert!(is_ported_critter(net_id), "the ported TownCritter subset");
    let (w, h) = npc.size();

    // `:52982-52998` — flag, the go-home weather gate.
    let flag = world.raining
        || !world.day_time
        || world.eclipse
        || world.slime_rain;
    // `Main.IsItStorming && position.Y / 16 < worldSurface` — the port's
    // World carries no storm flag yet (the module doc's fold).

    // `:53278-53295` — the critter head: select + face the target, stamp
    // the home tile on the first grounded tick.
    if town::town_critter(net_id) && npc.target == 255 {
        target_closest(npc, world, ctx.players, true, 7, ctx.ticker);
        let p = &ctx.players[npc.target as usize];
        if npc.x < p.x {
            npc.facing = 1;
            npc.sprite_dir = npc.facing;
        }
        if npc.x > p.x {
            npc.facing = -1;
            npc.sprite_dir = npc.facing;
        }
        if npc.home.is_none() {
            // `:53293` — UpdateHomeTileState(homeless, (int)((X + width/2)
            // / 16), homeTileY). The port folds homeless into home; the
            // homeTileY rides the current feet row (the :53298 shape).
            let hx = ((npc.x + w / 2.0) / 16.0) as i32;
            let hy = ((npc.y + h + 4.0) / 16.0) as i32;
            update_home_tile_state(npc, hx, hy);
        }
    } else if npc.home.is_none() && npc.vy == 0.0 {
        // `:53296-53298` — the non-critter home stamp (the shimmering gate
        // folds false — the port has no shimmering critters).
        let hx = (npc.x + w * 0.5) as i32 / 16;
        let hy = ((npc.y + h + 4.0) / 16.0) as i32;
        update_home_tile_state(npc, hx, hy);
    }

    // `:53300-53303` — the tile coords + the resting-spot floor.
    let num6 = (npc.x + w / 2.0) as i32 / 16;
    let num7 = ((npc.y + h + 1.0) / 16.0) as i32;
    let (floor_x, floor_y) = find_good_resting_spot(world, npc, &[], num6, num7, world.day_time);

    // `:53308-53312` — directionY = -1, a zero direction latches +1.
    npc.direction_y = -1;
    if npc.facing == 0 {
        npc.facing = 1;
    }

    // `:53451-53459` — the flag census for the ported critters.
    let flag8 = matches!(net_id, 300 | 447 | 610); // false for the batch
    let flag10 = matches!(net_id, 361 | 445 | 687); // the Frog
    let flag12 = flag10; // flag9 false → flag12 = flag10
    // flag5 (talk) false, flag16/flag17 (danger) false on the solo path.
    let flag5 = false;
    let flag17 = false;

    // `:53622` — the state machine.
    if npc.ai[0] == 0.0 {
        // `:53632-53635` — the localAI[3] countdown.
        if npc.local_ai[3] > 0.0 {
            npc.local_ai[3] -= 1.0;
        }
        // `:53641-53648` — the Frog's wet flee (flag10 | flag11, wet).
        if flag10 && npc.wet {
            npc.ai[0] = 1.0;
            npc.ai[1] = 200.0 + rng.next_range(500, 700) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 0.0;
            npc.net_update = true;
        } else {
            // `:53695-53762` — the critter else-branch (the !TownCritter
            // home-return at :53649 is skipped).
            // `:53697-53700` — flag8's 0.5 vx damp (false for the batch).
            if flag8 {
                npc.vx *= 0.5;
            }
            // `:53701-53712` — the 0.1 vx decay toward 0.
            if npc.vx > 0.1 {
                npc.vx -= 0.1;
            } else if npc.vx < -0.1 {
                npc.vx += 0.1;
            } else {
                npc.vx = 0.0;
            }
            // `:53719-53722` — the ai[1] countdown.
            if npc.ai[1] > 0.0 {
                npc.ai[1] -= 1.0;
            }
            // `:53723-53724` — the idle walk prediction.
            let mut flag21 = true;
            let probe_x = ((npc.x + w / 2.0 + 15.0 * npc.facing as f32) / 16.0) as i32;
            let probe_y = ((npc.y + h - 16.0) / 16.0) as i32;
            let currently_drowning = npc.wet && !flag12;
            let (_keep, avoid_falling2) = get_walk_prediction(
                world, npc, &[], num6, floor_x, flag12, currently_drowning, probe_x, probe_y,
            );
            // `:53725-53736` — the drown walk-start (the Frog skips it —
            // flag12 true; the duck's wet && !flag12 path).
            if npc.wet
                && !flag12
                && town::check_if_will_drown(town::drown_collision(world, npc.x, npc.y, w, h, true))
            {
                npc.ai[0] = 1.0;
                npc.ai[1] = 200.0 + rng.next_max(300) as f32;
                npc.ai[2] = 0.0;
                if town::town_critter(net_id) {
                    npc.ai[1] += rng.next_range(200, 400) as f32;
                }
                npc.local_ai[3] = 0.0;
                npc.net_update = true;
            }
            // `:53737-53740` — avoidFalling refuses the walk-start.
            if avoid_falling2 {
                flag21 = false;
            }
            // `:53741-53761` — the timer-expiry walk-start / turn.
            if npc.ai[1] <= 0.0 {
                if flag21 && !avoid_falling2 {
                    npc.ai[0] = 1.0;
                    npc.ai[1] = 200.0 + rng.next_max(300) as f32;
                    npc.ai[2] = 0.0;
                    if town::town_critter(net_id) {
                        npc.ai[1] += rng.next_range(200, 400) as f32;
                    }
                    npc.local_ai[3] = 0.0;
                    npc.net_update = true;
                } else {
                    npc.facing *= -1;
                    npc.ai[1] = 60.0 + rng.next_max(120) as f32;
                    npc.net_update = true;
                }
            }
        }
        // `:53764-53786` — the home-band direction flip / the Next(80)
        // 200-tick turn (the !flag || good-resting-spot gate).
        let good_rest = town::is_in_a_good_resting_spot(
            net_id, npc.ai[0], npc.wet, world.day_time, num6, num7, floor_x, floor_y,
        );
        if !flag || good_rest {
            if num6 < floor_x - 25 || num6 > floor_x + 25 {
                if npc.local_ai[3] == 0.0 {
                    if num6 < floor_x - 50 && npc.facing == -1 {
                        npc.facing = 1;
                        npc.net_update = true;
                    } else if num6 > floor_x + 50 && npc.facing == 1 {
                        npc.facing = -1;
                        npc.net_update = true;
                    }
                }
            } else if rng.next_max(80) == 0 && npc.local_ai[3] == 0.0 {
                npc.local_ai[3] = 200.0;
                npc.facing *= -1;
                npc.net_update = true;
            }
        }
    } else if npc.ai[0] == 1.0 {
        // `:53792-53798` — the resting-spot return is !TownCritter-gated
        // (skipped for every ported critter).
        // `:53801` — flag22, the drown gate (the Frog's flag12 clears it).
        let flag22 = !flag12 && town::drown_collision(world, npc.x, npc.y, w, h, true);
        // `:53802-53816` — the home-band ai[1] drain + the countdown.
        if !flag22 {
            // `:53804-53814` — the !homeless && !tileDungeon home-band
            // drain (the port's home folds homeless; a critter with a home
            // outside the ±35 band drains 5 off the timer while walking
            // away). The tileDungeon read folds false (the surface critters
            // never stand in the dungeon).
            if npc.home.is_some() && (num6 < floor_x - 35 || num6 > floor_x + 35) {
                if npc.x < (floor_x * 16) as f32 && npc.facing == -1 {
                    npc.ai[1] -= 5.0;
                } else if npc.x > (floor_x * 16) as f32 && npc.facing == 1 {
                    npc.ai[1] -= 5.0;
                }
            }
            npc.ai[1] -= 1.0;
        }
        // `:53817-53832` — the walk-end idle arm.
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 300.0 + rng.next_max(300) as f32;
            npc.ai[2] = 0.0;
            if town::town_critter(net_id) {
                npc.ai[1] -= rng.next_max(100) as f32;
            } else {
                npc.ai[1] += rng.next_max(900) as f32;
            }
            npc.local_ai[3] = 60.0;
            npc.net_update = true;
        }
        // `:53865-53950` — the walk velocity. num21 = 1, num22 = 0.07 for
        // the batch (the 614/299/539/538/639-645/flag9/625/flag8 overrides
        // are all false; the friendly-flee :53907 needs flag17|flag22).
        let mut num21 = 1.0f32;
        let mut num22 = 0.07f32;
        if npc.friendly && (flag17 || flag22) {
            num21 = 1.5 + (1.0 - npc.hp as f32 / npc.max_hp as f32) * 0.9;
            num22 = 0.1;
        }
        // `:53917-53927` — the Frog's wet swim (flag10 && wet).
        if flag10 && npc.wet {
            if npc.vx.abs() < 0.05 && npc.vy.abs() < 0.05 {
                npc.vx += num21 * 10.0 * npc.facing as f32;
            } else {
                npc.vx *= 0.9;
            }
        } else if npc.vx < -num21 || npc.vx > num21 {
            // `:53928-53934` — the over-cap grounded brake.
            if npc.vy == 0.0 {
                npc.vx *= 0.8;
                npc.vy *= 0.8;
            }
        } else if npc.vx < num21 && npc.facing == 1 {
            // `:53935-53942`.
            npc.vx += num22;
            if npc.vx > num21 {
                npc.vx = num21;
            }
        } else if npc.vx > -num21 && npc.facing == -1 {
            // `:53943-53950` — the C#'s `> num21` clamp is a vanilla quirk
            // (it never fires on the negative accel); kept 1:1.
            npc.vx -= num22;
            if npc.vx > num21 {
                npc.vx = num21;
            }
        }
        // `:53951-53966` — the StepDown / StepUp.
        let home_y = npc.home.map(|(_, y)| y).unwrap_or(-1);
        let mut flag23 = true;
        if (home_y * 16 - 32) as f32 > npc.y {
            flag23 = false;
        }
        if !flag23 && npc.vy == 0.0 {
            town::step_down(world, npc);
        }
        if npc.vy >= 0.0 {
            town::town_step_up(world, npc, flag23);
        }
        // `:53967-54196` — the grounded obstacle / jump block.
        if npc.vy == 0.0 {
            let num23 = ((npc.x + w / 2.0 + 15.0 * npc.facing as f32) / 16.0) as i32;
            let num24 = ((npc.y + h - 16.0) / 16.0) as i32;
            let num25 = 180.0f32;
            let (mut keepwalking3, mut avoid_falling3) = get_walk_prediction(
                world, npc, &[], num6, floor_x, flag12, flag22, num23, num24,
            );
            let mut flag24 = false;
            // `:53975-53992` — the drown jump is isLikeATownNPC-gated
            // (false for the critters) — skipped, flag25 stays false.
            let flag25 = false;
            // `:53993-54017` — the avoidFalling walk-stop.
            if avoid_falling3 && !flag24 {
                let num29 = ((npc.x + w / 2.0) / 16.0) as i32;
                let mut num30 = 0;
                for num31 in -1..=1 {
                    if town::tile_at(world, num29 + num31, num24 + 1)
                        .is_some_and(|t| town::nactive(t) && town::raw_solid(t.kind))
                    {
                        num30 += 1;
                    }
                }
                if num30 <= 2 {
                    if npc.vx != 0.0 {
                        npc.net_update = true;
                    }
                    keepwalking3 = false;
                    avoid_falling3 = false;
                    npc.ai[0] = 0.0;
                    npc.ai[1] = 50.0 + rng.next_max(50) as f32;
                    npc.ai[2] = 0.0;
                    npc.local_ai[3] = 40.0;
                }
            }
            // `:54018-54023` — the stuck-at-X direction flip.
            if npc.x == npc.local_ai[3] && !flag24 {
                npc.facing *= -1;
                npc.net_update = true;
                npc.local_ai[3] = num25;
            }
            // `:54024-54038` — the drown countdown / the -1 latch.
            if flag22 && !flag24 {
                if npc.local_ai[3] > num25 {
                    npc.local_ai[3] = num25;
                }
                if npc.local_ai[3] > 0.0 {
                    npc.local_ai[3] -= 1.0;
                }
            } else {
                npc.local_ai[3] = -1.0;
            }
            // `:54039-54042` — the three probe tiles + the short flag.
            let tile5 = town::tile_at(world, num23, num24 - 2);
            let tile4 = town::tile_at(world, num23, num24 - 1);
            let tile3 = town::tile_at(world, num23, num24);
            let flag26 = (h as i32) / 16 < 3;
            // `:54043-54080` — the door open is townNPC-gated (skipped).
            // `:54081-54195` — the obstacle jump ladder.
            let moving_forward = (npc.vx < 0.0 && npc.facing == -1)
                || (npc.vx > 0.0 && npc.facing == 1);
            if moving_forward {
                let mut flag27 = false;
                let solid5 = tile5.is_some_and(|t| {
                    town::nactive(t) && town::raw_solid(t.kind) && !town::raw_solid_top(t.kind)
                });
                let solid4 = tile4.is_some_and(|t| {
                    town::nactive(t) && town::raw_solid(t.kind) && !town::raw_solid_top(t.kind)
                });
                let solid3 = tile3.is_some_and(|t| {
                    town::nactive(t) && town::raw_solid(t.kind) && !town::top_slope(t)
                });
                // `:54087-54112` — the 3-tile wall (the -6 jump).
                if solid5 && (!flag26 || solid4) {
                    if !town::solid_tiles_versatile(world, num23 - npc.facing * 2, num23 - npc.facing, num24 - 5, num24 - 1)
                        && !town::solid_tiles(world, num23, num23, num24 - 5, num24 - 3)
                    {
                        npc.vy = -6.0;
                        npc.net_update = true;
                    } else if !flag25 {
                        flag27 = true;
                    }
                } else if solid4 {
                    // `:54113-54129` — the 2-tile wall (the -5 jump).
                    if !town::solid_tiles_versatile(world, num23 - npc.facing * 2, num23 - npc.facing, num24 - 4, num24 - 1)
                        && !town::solid_tiles(world, num23, num23, num24 - 4, num24 - 2)
                    {
                        npc.vy = -5.0;
                        npc.net_update = true;
                    } else {
                        flag27 = true;
                    }
                } else if npc.y + h - (num24 * 16) as f32 > 20.0 && solid3 {
                    // `:54130-54146` — the step-up (the -4.4 jump).
                    if !town::solid_tiles_versatile(world, num23 - npc.facing * 2, num23, num24 - 3, num24 - 1)
                    {
                        npc.vy = -4.4;
                        npc.net_update = true;
                    } else {
                        flag27 = true;
                    }
                } else if avoid_falling3 {
                    // `:54147-54157` — the ledge turn.
                    if !flag25 {
                        flag27 = true;
                    }
                }
                // `:54163-54170` — flag28 (the friendly flee) is flag17-
                // gated (false on the solo path) — skipped.
                // `:54171-54176` — the obstacle direction + velocity flip.
                if flag27 {
                    npc.facing *= -1;
                    npc.vx *= -1.0;
                    npc.net_update = true;
                }
                // `:54177-54181` — keepwalking refreshes the timer.
                if keepwalking3 {
                    npc.ai[1] = 90.0;
                    npc.net_update = true;
                }
                // `:54182-54185` — a jump latches the stuck-X.
                if npc.vy < 0.0 {
                    npc.local_ai[3] = npc.x;
                }
            }
            // `:54187-54190` — the wet jump boost.
            if npc.vy < 0.0 && npc.wet {
                npc.vy *= 1.2;
            }
            // `:54191-54194` — the critter jump boost (×1.2, !flag8).
            if npc.vy < 0.0 && town::town_critter(net_id) && !flag8 {
                npc.vy *= 1.2;
            }
        } else if false {
            // `:54197-54225` — the flag15 (IsTownSlime) airborne ledge
            // brake — false for every ported critter (the module doc).
        }
        // `:54226-54242` — the 30-tick stuck detector.
        if npc.old_x == npc.x
            && npc.old_y == npc.y
            && npc.old_vx == npc.vx
            && npc.old_vy == npc.vy + 0.3
            && !flag17
        {
            npc.local_ai[2] += 1.0;
            if npc.local_ai[2] >= 30.0 {
                npc.local_ai[2] = 0.0;
                npc.ai[0] = 1.0;
                npc.ai[1] = 200.0 + rng.next_max(300) as f32;
                npc.ai[2] = 0.0;
                if town::town_critter(net_id) {
                    npc.ai[1] += rng.next_range(200, 400) as f32;
                }
                npc.facing *= -1;
                npc.vx *= -1.0;
                npc.net_update = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation; the RNG witnesses mirror the cited
    //! `Next` calls on a cloned `UnifiedRandom` and compare the stream
    //! state so a drifted draw order fails by construction.
    use super::*;
    use crate::npc::NpcKind;

    fn ground() -> World {
        let mut w = World::new("t".into(), 200, 120);
        w.world_surface = 30.0;
        w.day_time = true;
        // A flat floor at row 60.
        for x in 0..w.width {
            if let Some(t) = w.tile_mut(x, 60) {
                t.kind = 1; // dirt
                t.flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w
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

    fn duck(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::Duck, x, y, 1, 0.0)
    }

    fn ctx<'a>(players: &'a [TargetCandidate]) -> TownCritterCtx<'a> {
        TownCritterCtx { players, ticker: 0 }
    }

    #[test]
    fn the_head_faces_the_target_and_stamps_the_home_tile() {
        // `:53278-53295` — a targetless critter faces the player and
        // stamps home on the first tick.
        let w = ground();
        let players = [cand(1200.0, 900.0)];
        let mut n = duck(1000.0, 944.0); // feet on row 60 (944 + 26 = 970)
        n.target = 255;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(0);
        town_critter_step(&mut n, &w, &ctx(&players), &mut rng);
        assert_eq!(n.facing, 1, "the player is to the right");
        assert_eq!(n.sprite_dir, 1);
        assert!(n.home.is_some(), "the home tile stamps");
        assert_eq!(n.direction_y, -1, ":53308");
    }

    #[test]
    fn the_idle_decay_zeroes_a_small_velocity() {
        // `:53701-53712` — the 0.1 decay; a vx inside [-0.1, 0.1] zeroes.
        let w = ground();
        let players = [cand(1000.0, 900.0)];
        let mut n = duck(1000.0, 944.0);
        n.target = 0;
        n.home = Some((62, 60));
        n.ai[0] = 0.0;
        n.ai[1] = 500.0; // no walk-start this tick
        n.vx = 0.05;
        n.local_ai[3] = 100.0; // no Next(80) turn
        let mut rng = UnifiedRandom::new(0);
        let mut mirror = rng.clone();
        town_critter_step(&mut n, &w, &ctx(&players), &mut rng);
        assert_eq!(n.vx, 0.0, "the dead-band zeroes");
        // The idle arm draws only the Next(80) turn when localAI[3] == 0
        // and the band holds; here localAI[3] > 0 so NO draw fires.
        assert_eq!(rng, mirror, "a quiet idle tick draws nothing");
    }

    #[test]
    fn the_walk_accelerates_toward_the_cap_one() {
        // `:53935-53942` — vx < 1 && direction 1 → += 0.07, clamp 1.
        let w = ground();
        let players = [cand(1000.0, 900.0)];
        let mut n = duck(1000.0, 944.0);
        n.target = 0;
        n.home = Some((62, 60));
        n.ai[0] = 1.0;
        n.ai[1] = 500.0;
        n.vx = 0.0;
        n.vy = 0.0;
        n.facing = 1;
        let mut rng = UnifiedRandom::new(0);
        town_critter_step(&mut n, &w, &ctx(&players), &mut rng);
        assert_eq!(n.vx, 0.07, "one accel step");
    }

    #[test]
    fn the_walk_end_arms_the_critter_idle_timer() {
        // `:53817-53832` — ai[1] <= 0 → ai[0] 0, ai[1] = 300 + Next(300)
        // - Next(100) (the critter arm), localAI[3] = 60.
        let w = ground();
        let players = [cand(1000.0, 900.0)];
        let mut n = duck(1000.0, 944.0);
        n.target = 0;
        n.home = Some((62, 60));
        n.ai[0] = 1.0;
        n.ai[1] = 0.0; // expires this tick (the :53815 decrement → -1)
        n.vx = 0.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(5);
        let mut mirror = rng.clone();
        town_critter_step(&mut n, &w, &ctx(&players), &mut rng);
        assert_eq!(n.ai[0], 0.0);
        let base = mirror.next_max(300);
        let crit = mirror.next_max(100);
        assert_eq!(n.ai[1], 300.0 + base as f32 - crit as f32);
        assert_eq!(n.local_ai[3], 60.0);
    }

    #[test]
    fn the_frog_wet_flee_arms_the_swim_timer() {
        // `:53641-53648` — the Frog (flag10) wet → ai[0] 1, ai[1] =
        // 200 + Next(500, 700).
        let w = ground();
        let players = [cand(1000.0, 900.0)];
        let mut n = Npc::new(NpcKind::Frog, 1000.0, 950.0, 1, 0.0);
        n.target = 0;
        n.home = Some((62, 60));
        n.ai[0] = 0.0;
        n.wet = true;
        let mut rng = UnifiedRandom::new(9);
        let mut mirror = rng.clone();
        town_critter_step(&mut n, &w, &ctx(&players), &mut rng);
        assert_eq!(n.ai[0], 1.0);
        let draw = mirror.next_range(500, 700);
        assert_eq!(n.ai[1], 200.0 + draw as f32);
        assert_eq!(n.local_ai[3], 0.0);
    }
}
