//! The aiStyle-33 arm — the Skeletron Prime Saw (`NPC.AI()`'s
//! `else if (this.aiStyle == 33)` inline block, `NPC.cs:27943-28244`).
//!
//! S1-D-3 (aiStyle-33 round, code-only mode): this module is the faithful
//! 1:1 port of the arm as a pure step function over `Npc` + a small
//! context seam. It is NOT wired into the live dispatch yet —
//! `npc.rs::update_prime_limb` (the v1 shared tether approximation for
//! all four limbs) keeps running until the wiring round (the sibling
//! D-3 rounds land the same way: module + tests first, dispatch claim
//! second — the `npc_ai_style28.rs` pattern, commit `040df4c5`). Every
//! other file is byte-identical except the one `mod` line in `main.rs`
//! and the docs.
//!
//! The only type on this style is `NPCID.PrimeSaw` (129,
//! `ID/NPCID.cs:10935`); its `SetDefaults` row (`NPC.cs:10588-10602`):
//! 52×52, aiStyle 33, damage 56, defense 38, lifeMax 9000,
//! `SoundID.NPCHit4`/`NPCDeath14`, noGravity, noTileCollide,
//! knockBackResist 0, netAlways. The port's static tables already carry
//! it (`npc/data.rs`: `ai_style() == 33`, size (52, 52), 9000 HP,
//! 56 damage). There are NO per-type branches inside the arm — the
//! whole block below is the family. The siblings ride their own arms:
//! the cannon 128 → aiStyle 35 (`:28522`), the vice 130 → aiStyle 34
//! (`:28245`), the laser 131 → aiStyle 36 (`:28757`).
//!
//! The machine (`ai[2]` is the state, `ai[3]` the per-state counter):
//! * the dock is `(head.Center.X − 200·ai[0], head.position.Y + 230)`
//!   (`:27946-27947`); `ai[0]` is the seat side (−1 left / +1 right,
//!   the head's spawn pass writes it, `:27645`), `ai[1]` the head's
//!   NPC slot (`:27646`).
//! * the >800 / <400 desync hysteresis into state 99 (`:27949-27959`),
//! * the head-gone despawn (`:27961-27970`): `ai[2] += 10` per tick,
//!   `> 50 || Main.netMode != 2` kills — single-player dies at once,
//! * state 0/3 (`:28022-28152`): the head's `ai[1]` gates the attack
//!   chase (7-speed aim, 600-tick reset) vs the dock hover (the
//!   320/260 Y band, the asymmetric X band, the 300-tick `ai[2]++`),
//! * state 1 (`:28153-28178`): the rise above `head.Y − 200`, then the
//!   22-speed dash at the player,
//! * state 2 (`:28179-28185`): the dash ends below the player or on
//!   any upward velocity → state 3,
//! * state 4 (`:28186-28239`): the post-dash chase (7-speed, 600-tick
//!   reset to 0),
//! * state 5 (`:28240-28243`): the crossed-player reset (never entered
//!   from inside THIS arm — a sibling arm's write, kept verbatim).
//!
//! RNG by construction: the C# arm (`NPC.cs:27943-28244`) draws NO
//! `Main.rand` — every counter is deterministic (`ai[3]` ticks, the
//! hp-free state gates). The one `HitEffect()` call on the head-gone
//! despawn (`:27967`) reaches the type-128..131 gore arm
//! (`:89433-89469`) whose four `Main.rand.Next(61, 64)` draws
//! (`:89452/:89456/:89460/:89464`) are COSMETIC gore picks;
//! the port has no NPC `HitEffect` port (the dust/gore skip precedent,
//! S1-D-3e/f/g), so the despawn is the `hp = 0` convention alone and
//! the port takes no RNG parameter at all. The witness test
//! `the_arm_is_deterministic_without_an_rng` pins two identical inputs
//! to identical state.
//!
//! Faithfulness notes vs the live `update_prime_limb` (the wiring round
//! replaces the saw's share of it with this arm):
//! * The live arm has no state machine at all — one tether drift
//!   (±0.07/±0.04 accel, ±8/±3 caps) plus a 300-tick dash timer. The
//!   C# has SEVEN states with different maths each; all ported here.
//! * The desync hysteresis is STRICT both ways: `> 800` arms state 99,
//!   `< 400` disarms (`:27951`/`:27956`) — 800 and 400 themselves hold.
//! * The head-gone gate is `!active || aiStyle != 32` (`:27961`) — a
//!   head slot holding any other style despawns the limb too; the
//!   live arm only checks `prime_present`.
//! * `spriteDirection = -(int)ai[0]` (`:27960`) — the TRUNCATED int
//!   negation, not a sign test (the live arm's `< 0` ternary agrees
//!   only for ±1 seats).
//! * The hover band compares `base.position.Y` (the TOP edge,
//!   `:28098`/`:28110`) against `head.Y + 320/+260`, while the X band
//!   compares CENTRES with INTEGER half-widths (`:28122`/`:28134`) —
//!   and the left edge is `headCenter − 250`, not the dock's −200·side.
//! * The dock aim rotation (`atan2 + 1.57f`, `:28151`) OVERWRITES the
//!   chase aim rotation (`atan2 − 1.57f`, `:28047`) at the end of the
//!   0/3 block — both writes are kept, in order.
//! * The chase normaliser DIVIDES by the distance with no epsilon
//!   floor (`:28044`, `:28173`, `:28192`) — a zero distance yields the
//!   C#'s NaN/Inf verbatim (the live arm floors at 0.001).
//! * The arm does NOT integrate `position += velocity` — the saw is
//!   `noTileCollide`, so `UpdateNPC` integrates after `AI()`
//!   (`NPC.cs:90498`); the caller owns the step.
//! * `life = -1; HitEffect(); active = false` (`:27966-27968`) has no
//!   port fields — the port's death convention `hp = 0` stands in and
//!   the step returns early (same as the live arm and `npc_ai_style28`).
//!   The early return is observationally identical to the C# falling
//!   through: `ai[2]` is 10 (SP) or ≥ 60 (server), so NO state branch
//!   matches (`0/1/2/3/4/5/99` are all the C# tests) and nothing else
//!   runs after `:27970` anyway.
//! * `EncourageDespawn(10)` (`:28026`) rides the existing
//!   `Npc::encourage_despawn` (`NPC.cs:7249-7256` — only ever lowers
//!   the window).
//! * The three `TargetClosest()` calls (`:28030`, `:28168`, `:28188`)
//!   ride `npc_target::target_closest` (S1-D-3p) with the C# style 33
//!   and `faceTarget = true` (the default).
#![allow(dead_code)] // wiring deferred (code-only mode) — the dispatch claim is the follow-up round

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The head (`Main.npc[(int)this.ai[1]]`) reads the arm makes. The
/// caller (the future style-axis arm) fills it from the head's slot —
/// the port's `World.prime_*` write-through carries only the position
/// today, so the seam names every field the C# touches.
#[derive(Clone, Copy, Debug)]
pub struct Style33Context {
    /// `Main.npc[(int)ai[1]].active` (`NPC.cs:27961`).
    pub head_active: bool,
    /// `Main.npc[(int)ai[1]].aiStyle` (`:27961`) — must be 32.
    pub head_ai_style: i32,
    /// `Main.npc[(int)ai[1]].position` (`:27946-27947` and every
    /// re-read below).
    pub head_x: f32,
    pub head_y: f32,
    /// `Main.npc[(int)ai[1]].width` — an INT in the C# (`:27946` reads
    /// `width / 2` with INTEGER division).
    pub head_width: i32,
    /// `Main.npc[(int)ai[1]].ai[1]` — the head's phase (`:28024`,
    /// `:28028`): 0 hover, 1 charge, 2 rage, 3 despawn.
    pub head_ai1: f32,
    /// `Main.netMode` (`:27964`) — 0 single-player, 1 client, 2 server.
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for `TargetClosest`'s `targetSetFrame`
    /// stamp (`NPC.cs:77780`).
    pub ticker: u64,
}

/// The dock vector `(num493, num494)` — recomputed at each C# site
/// (`:27946-27947`, `:28148-28149`, `:28156-28157`, `:28235-28236`):
/// the head's centre (`width / 2` is INTEGER division) minus
/// `200·ai[0]`, and `head.Y + 230`, both relative to the saw's FLOAT
/// centre (`width * 0.5f`).
fn dock_vector(npc: &Npc, ctx: &Style33Context) -> (f32, f32) {
    let (w, h) = npc.size();
    let cx = npc.x + w * 0.5;
    let cy = npc.y + h * 0.5;
    let head_cx = ctx.head_x + (ctx.head_width / 2) as f32;
    (head_cx - 200.0 * npc.ai[0] - cx, ctx.head_y + 230.0 - cy)
}

/// `NPC.AI()`'s aiStyle-33 arm (`NPC.cs:27943-28244`), one tick.
/// `players` is the target-scan slice (at least one slot — the C#
/// always has the 255 `Main.player` slots; the port's session player
/// is slot 0). No shot is fired by this arm — the saw is a contact
/// weapon; the dash IS the attack.
pub fn style33_step(npc: &mut Npc, world: &World, ctx: &Style33Context, players: &[TargetCandidate]) {
    let (w, _h) = npc.size();
    // `:27945-27948` — the dock distance for the desync gate.
    let (num493, num494) = dock_vector(npc, ctx);
    let num495 = (num493 * num493 + num494 * num494).sqrt();
    // `:27949-27959` — the hysteresis: STRICT > 800 arms state 99,
    // STRICT < 400 disarms it; the band between holds the state.
    if npc.ai[2] != 99.0 {
        if num495 > 800.0 {
            npc.ai[2] = 99.0;
        }
    } else if num495 < 400.0 {
        npc.ai[2] = 0.0;
    }
    // `:27960` — `spriteDirection = -(int)this.ai[0]` (truncation).
    npc.sprite_dir = -(npc.ai[0] as i32);
    // `:27961-27970` — the head-gone despawn. `life = -1; HitEffect();
    // active = false` → the port's hp-0 convention; the HitEffect gore
    // (the four Next(61,64) picks, `:89452/:89456/:89460/:89464`) is
    // the cosmetic skip.
    if !ctx.head_active || ctx.head_ai_style != 32 {
        npc.ai[2] += 10.0;
        if npc.ai[2] > 50.0 || ctx.net_mode != 2 {
            npc.hp = 0;
            return;
        }
    }
    // The INTEGER half-width the centre comparisons use
    // (`(float)(base.width / 2)`, `:27997` etc.).
    let half_w = (w as i32 / 2) as f32;
    let head_cx = ctx.head_x + (ctx.head_width / 2) as f32;
    if npc.ai[2] == 99.0 {
        // `:27971-28021` — the desync return: chase the head's TOP edge
        // on Y and CENTRE on X, damping only against the motion.
        if npc.y > ctx.head_y {
            // `:27973-27984`.
            if npc.vy > 0.0 {
                npc.vy *= 0.96;
            }
            npc.vy -= 0.1;
            if npc.vy > 8.0 {
                npc.vy = 8.0;
            }
        } else if npc.y < ctx.head_y {
            // `:27985-27996`.
            if npc.vy < 0.0 {
                npc.vy *= 0.96;
            }
            npc.vy += 0.1;
            if npc.vy < -8.0 {
                npc.vy = -8.0;
            }
        }
        if npc.x + half_w > head_cx {
            // `:27997-28008`.
            if npc.vx > 0.0 {
                npc.vx *= 0.96;
            }
            npc.vx -= 0.5;
            if npc.vx > 12.0 {
                npc.vx = 12.0;
            }
        }
        if npc.x + half_w < head_cx {
            // `:28009-28020`.
            if npc.vx < 0.0 {
                npc.vx *= 0.96;
            }
            npc.vx += 0.5;
            if npc.vx < -12.0 {
                npc.vx = -12.0;
            }
        }
    } else if npc.ai[2] == 0.0 || npc.ai[2] == 3.0 {
        // `:28022-28152` — the dock hover / attack chase, gated on the
        // head's phase.
        if ctx.head_ai1 == 3.0 {
            // `:28024-28027` — the head is despawning: follow it out.
            npc.encourage_despawn(10);
        }
        if ctx.head_ai1 != 0.0 {
            // `:28028-28088` — the head attacks: chase the player.
            target_closest(npc, world, players, true, 33, ctx.ticker);
            let p = &players[npc.target as usize];
            if p.dead {
                // `:28031-28038` — a dead target: sink at 0.1 to 16.
                npc.vy += 0.1;
                if npc.vy > 16.0 {
                    npc.vy = 16.0;
                }
            } else {
                // `:28041-28079` — the 7-speed aim with INTEGER player
                // half-extents (`:28042-28043`), the four independent
                // 0.97-damp/0.05-step comparisons.
                let (cw, ch) = npc.size();
                let vector50 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
                let mut num496 = p.x + (p.width / 2) as f32 - vector50.0;
                let mut num497 = p.y + (p.height / 2) as f32 - vector50.1;
                let num498 = 7.0 / (num496 * num496 + num497 * num497).sqrt();
                num496 *= num498;
                num497 *= num498;
                // `:28047` — the chase aim (OVERWRITTEN by the dock
                // rotation at `:28151` below; both writes kept).
                npc.rotation = num497.atan2(num496) - 1.57;
                if npc.vx > num496 {
                    if npc.vx > 0.0 {
                        npc.vx *= 0.97;
                    }
                    npc.vx -= 0.05;
                }
                if npc.vx < num496 {
                    if npc.vx < 0.0 {
                        npc.vx *= 0.97;
                    }
                    npc.vx += 0.05;
                }
                if npc.vy > num497 {
                    if npc.vy > 0.0 {
                        npc.vy *= 0.97;
                    }
                    npc.vy -= 0.05;
                }
                if npc.vy < num497 {
                    if npc.vy < 0.0 {
                        npc.vy *= 0.97;
                    }
                    npc.vy += 0.05;
                }
            }
            // `:28081-28087` — the 600-tick chase reset (back to 0,
            // NOT the hover's ++).
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 600.0 {
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
        } else {
            // `:28089-28146` — the head hovers: hold the dock band.
            // `:28091-28097` — the 300-tick state advance FIRST.
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 300.0 {
                npc.ai[2] += 1.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
            // `:28098-28121` — the Y band compares the TOP edge
            // (`base.position.Y`) against head.Y + 320 / + 260; the
            // 60-px gap between them is a dead band (no Y write).
            if npc.y > ctx.head_y + 320.0 {
                if npc.vy > 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy -= 0.04;
                if npc.vy > 3.0 {
                    npc.vy = 3.0;
                }
            } else if npc.y < ctx.head_y + 260.0 {
                if npc.vy < 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy += 0.04;
                if npc.vy < -3.0 {
                    npc.vy = -3.0;
                }
            }
            // `:28122-28145` — the ASYMMETRIC X band: the right edge is
            // the head's centre, the left edge headCentre − 250 (NOT
            // the dock's −200·side).
            if npc.x + half_w > head_cx {
                if npc.vx > 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx -= 0.3;
                if npc.vx > 12.0 {
                    npc.vx = 12.0;
                }
            }
            if npc.x + half_w < head_cx - 250.0 {
                if npc.vx < 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx += 0.3;
                if npc.vx < -12.0 {
                    npc.vx = -12.0;
                }
            }
        }
        // `:28147-28151` — the dock rotation, recomputed and written
        // AFTER both sub-branches (it overwrites the chase aim).
        let (num499, num500) = dock_vector(npc, ctx);
        // `:28150` — `num501` is a dead-store sqrt in the C# (computed,
        // never read); kept verbatim, no observable effect.
        let _num501 = (num499 * num499 + num500 * num500).sqrt();
        npc.rotation = num500.atan2(num499) + 1.57;
    } else if npc.ai[2] == 1.0 {
        // `:28153-28178` — the rise: damp X, accelerate up to −8.
        let (num502, num503) = dock_vector(npc, ctx);
        // `:28158` — `num504`'s FIRST value is a dead-store sqrt (the
        // name is reused for the dash scale at `:28173` below).
        let _num504 = (num502 * num502 + num503 * num503).sqrt();
        npc.rotation = num503.atan2(num502) + 1.57;
        npc.vx *= 0.95;
        npc.vy -= 0.1;
        if npc.vy < -8.0 {
            npc.vy = -8.0;
        }
        // `:28166-28177` — above head.Y − 200 (STRICT <, TOP edges):
        // re-target and dash at the player at 22.
        if npc.y < ctx.head_y - 200.0 {
            target_closest(npc, world, players, true, 33, ctx.ticker);
            npc.ai[2] = 2.0;
            let p = &players[npc.target as usize];
            let (cw, ch) = npc.size();
            let vector52 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
            let num502 = p.x + (p.width / 2) as f32 - vector52.0;
            let num503 = p.y + (p.height / 2) as f32 - vector52.1;
            let num504 = 22.0 / (num502 * num502 + num503 * num503).sqrt();
            npc.vx = num502 * num504;
            npc.vy = num503 * num504;
            npc.net_update = true;
        }
    } else if npc.ai[2] == 2.0 {
        // `:28179-28185` — the dash ends once below the player's TOP
        // edge or on any upward velocity.
        let p = &players[npc.target as usize];
        if npc.y > p.y || npc.vy < 0.0 {
            npc.ai[2] = 3.0;
        }
    } else if npc.ai[2] == 4.0 {
        // `:28186-28239` — the post-dash chase: the same 7-speed aim
        // (no dead-target branch here), the 600-tick reset, the dock
        // rotation at the end.
        target_closest(npc, world, players, true, 33, ctx.ticker);
        let p = &players[npc.target as usize];
        let (cw, ch) = npc.size();
        let vector53 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
        let mut num505 = p.x + (p.width / 2) as f32 - vector53.0;
        let mut num506 = p.y + (p.height / 2) as f32 - vector53.1;
        let num507 = 7.0 / (num505 * num505 + num506 * num506).sqrt();
        num505 *= num507;
        num506 *= num507;
        if npc.vx > num505 {
            if npc.vx > 0.0 {
                npc.vx *= 0.97;
            }
            npc.vx -= 0.05;
        }
        if npc.vx < num505 {
            if npc.vx < 0.0 {
                npc.vx *= 0.97;
            }
            npc.vx += 0.05;
        }
        if npc.vy > num506 {
            if npc.vy > 0.0 {
                npc.vy *= 0.97;
            }
            npc.vy -= 0.05;
        }
        if npc.vy < num506 {
            if npc.vy < 0.0 {
                npc.vy *= 0.97;
            }
            npc.vy += 0.05;
        }
        // `:28227-28233`.
        npc.ai[3] += 1.0;
        if npc.ai[3] >= 600.0 {
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        // `:28234-28238`.
        let (num505, num506) = dock_vector(npc, ctx);
        // `:28237` — `num507`'s reused value is a dead-store sqrt (the
        // name was the chase scale at `:28192` above).
        let _num507 = (num505 * num505 + num506 * num506).sqrt();
        npc.rotation = num506.atan2(num505) + 1.57;
    } else if npc.ai[2] == 5.0 {
        // `:28240-28243` — the crossed-player reset: moving right past
        // the player's centre, or moving left past it → state 0. (No
        // write inside this arm ever sets 5 — a sibling limb's dash
        // hand-off does; kept verbatim.)
        let p = &players[npc.target as usize];
        let p_cx = p.x + (p.width / 2) as f32;
        if (npc.vx > 0.0 && npc.x + half_w > p_cx) || (npc.vx < 0.0 && npc.x + half_w < p_cx) {
            npc.ai[2] = 0.0;
        }
    }
    // No `position += velocity` here: the saw is noTileCollide, so
    // `UpdateNPC` integrates after `AI()` (`NPC.cs:90498`).
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// The session player as the scan sees it (stock inputs, the
    /// `npc_target` wiring's convention).
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
        World::new("t".into(), 400, 400)
    }

    /// The head at (2000, 1000), width 80 (the C#'s 127 row,
    /// `NPC.cs:10558`), hovering (`ai[1] == 0`), single-player.
    fn ctx() -> Style33Context {
        Style33Context {
            head_active: true,
            head_ai_style: 32,
            head_x: 2000.0,
            head_y: 1000.0,
            head_width: 80,
            head_ai1: 0.0,
            net_mode: 0,
            ticker: 0,
        }
    }

    /// The saw on the right seat (`ai[0] = 1`, `:27645`), 52×52.
    /// At (1814, 1204) its centre is (1840, 1230) — exactly the dock
    /// (headCentre 2040 − 200, head.Y + 230).
    fn saw(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::PrimeSaw, x, y, 1, 0.0);
        n.ai[0] = 1.0;
        n.ai[1] = 0.0; // the head's slot (the ctx seam stands in)
        n
    }

    /// `NPC.cs:27949-27959` — the desync hysteresis is STRICT both
    /// ways: > 800 arms 99, < 400 disarms, the band between holds.
    /// The dock is (1840, 1230); a saw at x 1814+dx has dock distance
    /// |dx| (dy 0).
    #[test]
    fn the_desync_hysteresis_is_strict_both_ways() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        // Exactly 800: holds state 0.
        let mut n = saw(1814.0 + 800.0, 1204.0);
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 0.0, "800 is not > 800 (:27951)");
        // 801: arms 99.
        let mut n2 = saw(1814.0 + 801.0, 1204.0);
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 99.0);
        // In 99 at exactly 400: holds.
        let mut n3 = saw(1814.0 + 400.0, 1204.0);
        n3.ai[2] = 99.0;
        style33_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.ai[2], 99.0, "400 is not < 400 (:27956)");
        // 399: disarms to 0.
        let mut n4 = saw(1814.0 + 399.0, 1204.0);
        n4.ai[2] = 99.0;
        style33_step(&mut n4, &w, &c, &p);
        assert_eq!(n4.ai[2], 0.0);
    }

    /// `NPC.cs:27960` — `spriteDirection = -(int)ai[0]`: the right
    /// seat (+1) draws flipped (−1), the left seat (−1) draws +1.
    #[test]
    fn the_sprite_direction_is_the_negated_seat() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        let mut n = saw(1814.0, 1204.0);
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.sprite_dir, -1, "-(int)1");
        let mut n2 = saw(1814.0, 1204.0);
        n2.ai[0] = -1.0;
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.sprite_dir, 1, "-(int)-1");
    }

    /// `NPC.cs:27961-27970` — the head-gone despawn: single-player
    /// (`netMode != 2`) dies on the FIRST tick; the server counts
    /// `ai[2] += 10` and dies only when STRICTLY > 50 (50 survives,
    /// 60 dies). A head slot with the wrong aiStyle counts too.
    #[test]
    fn the_head_gone_despawn_counts_on_the_server_only() {
        let w = air();
        let p = [cand(1500.0, 1500.0)];
        let mut c = ctx();
        c.head_active = false;
        let mut n = saw(1814.0, 1204.0);
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.hp, 0, "single-player dies at once (:27964)");
        // Server: 10, 20, ... 50 all survive; 60 dies.
        c.net_mode = 2;
        let mut n2 = saw(1814.0, 1204.0);
        for want in [10.0, 20.0, 30.0, 40.0, 50.0] {
            style33_step(&mut n2, &w, &c, &p);
            assert_eq!(n2.ai[2], want);
            assert!(n2.hp > 0, "{want} is not > 50");
        }
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.hp, 0, "60 > 50 (:27964)");
        // The wrong-style head gate.
        let mut c3 = ctx();
        c3.head_ai_style = 35;
        let mut n3 = saw(1814.0, 1204.0);
        style33_step(&mut n3, &w, &c3, &p);
        assert_eq!(n3.hp, 0, "aiStyle != 32 despawns too (:27961)");
    }

    /// `NPC.cs:28091-28121` — the hover: the 300-tick counter advances
    /// the state (0 → 1), and the Y band compares TOP edges against
    /// head.Y + 320 / + 260 with a dead band between; the accel is
    /// 0.04 with the one-sided ±3 caps and the against-motion 0.96.
    #[test]
    fn the_hover_bands_and_the_300_tick_advance() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        // Below the band (y 1321 > 1320): rise at 0.04.
        let mut n = saw(1814.0, 1321.0);
        n.vy = 2.0; // moving down: damped first
        style33_step(&mut n, &w, &c, &p);
        assert!((n.vy - (2.0 * 0.96 - 0.04)).abs() < 1e-5, "{}", n.vy);
        // Inside the dead band (1260 <= y <= 1320): no Y write.
        let mut n2 = saw(1814.0, 1300.0);
        n2.vy = 1.25;
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.vy, 1.25, "the 260..320 gap holds (:28098/:28110)");
        // Above the band (y 1259 < 1260): sink, cap −3.
        let mut n3 = saw(1814.0, 1259.0);
        n3.vy = -4.0;
        style33_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.vy, -3.0, "the one-sided −3 cap (:28117-28120)");
        // The 300th tick advances 0 → 1 and resets the counter.
        let mut n4 = saw(1814.0, 1204.0);
        n4.ai[3] = 299.0;
        style33_step(&mut n4, &w, &c, &p);
        assert_eq!(n4.ai[2], 1.0, "ai[2]++ (:28094)");
        assert_eq!(n4.ai[3], 0.0);
        assert!(n4.net_update);
        // 298 → 299: holds.
        let mut n5 = saw(1814.0, 1204.0);
        n5.ai[3] = 298.0;
        style33_step(&mut n5, &w, &c, &p);
        assert_eq!(n5.ai[2], 0.0);
        assert_eq!(n5.ai[3], 299.0);
    }

    /// `NPC.cs:28122-28145` — the X band is ASYMMETRIC: the right edge
    /// is the head's centre (2040), the left edge headCentre − 250
    /// (1790) — not the dock's ±200. The accel is 0.3, the caps ±12.
    #[test]
    fn the_hover_x_band_is_asymmetric() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        // Centre 2041 > 2040: pushed left at 0.3.
        let mut n = saw(2015.0, 1290.0); // centre x 2041, y in dead band
        style33_step(&mut n, &w, &c, &p);
        assert!((n.vx - (-0.3)).abs() < 1e-5, "{}", n.vx);
        // Centre 1789 < 1790: pushed right at 0.3.
        let mut n2 = saw(1763.0, 1290.0);
        style33_step(&mut n2, &w, &c, &p);
        assert!((n2.vx - 0.3).abs() < 1e-5, "{}", n2.vx);
        // Centre 1840 (the dock) is INSIDE the band: no X write —
        // even though the dock is 200 left of the head's centre.
        let mut n3 = saw(1814.0, 1290.0);
        n3.vx = 0.75;
        style33_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.vx, 0.75, "1790 <= 1800 <= 2040 holds");
    }

    /// `NPC.cs:28028-28087` + `:28147-28151` — the attack chase: the
    /// 7-speed aim steps at 0.05 with the 0.97 against-motion damp,
    /// the 600-tick reset goes to state 0 (not ++), and the DOCK
    /// rotation overwrites the chase aim.
    #[test]
    fn the_attack_chase_and_the_dock_rotation_overwrite() {
        let w = air();
        let mut c = ctx();
        c.head_ai1 = 1.0; // the head charges
        let p = [cand(1500.0, 1500.0)];
        let mut n = saw(1814.0, 1204.0); // centre (1840, 1230)
        n.ai[2] = 3.0;
        style33_step(&mut n, &w, &c, &p);
        // The aim: player centre (1510, 1521), delta (−330, 291).
        // The chase steps TOWARD it: vx −0.05, vy +0.05 (from 0).
        assert!((n.vx - (-0.05)).abs() < 1e-5, "{}", n.vx);
        assert!((n.vy - 0.05).abs() < 1e-5, "{}", n.vy);
        // The final rotation is the DOCK aim, not the chase aim: the
        // saw sits exactly on the dock → atan2(0, 0) + 1.57 = 1.57.
        assert_eq!(n.rotation, 1.57, "the dock overwrite (:28151)");
        assert_eq!(n.ai[3], 1.0);
        // The 600th tick resets to 0 (state 3 → 0, not 4).
        let mut n2 = saw(1814.0, 1204.0);
        n2.ai[2] = 3.0;
        n2.ai[3] = 599.0;
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 0.0, "the chase reset (:28084)");
        assert_eq!(n2.ai[3], 0.0);
        assert!(n2.net_update);
        // A dead target sinks instead of chasing (:28031-28038).
        let mut dead = cand(1500.0, 1500.0);
        dead.dead = true;
        let mut n3 = saw(1814.0, 1204.0);
        n3.vy = 15.95;
        style33_step(&mut n3, &w, &c, &[dead]);
        assert_eq!(n3.vy, 16.0, "the 16 cap (:28034-28037)");
        assert_eq!(n3.vx, 0.0, "no chase on a dead target");
    }

    /// `NPC.cs:28024-28027` — the head's despawn phase (`ai[1] == 3`)
    /// encourages the limb's despawn at 10 ticks (only lowers).
    #[test]
    fn the_heads_despawn_phase_encourages_the_limbs() {
        let w = air();
        let mut c = ctx();
        c.head_ai1 = 3.0;
        let p = [cand(1500.0, 1500.0)];
        let mut n = saw(1814.0, 1204.0);
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.despawn_timer, 10, "EncourageDespawn(10) (:28026)");
        // ai[1] == 3 is != 0: the chase branch runs too.
        assert_eq!(n.ai[3], 1.0);
    }

    /// `NPC.cs:28153-28178` — the rise: X damps at 0.95, Y accelerates
    /// −0.1 to the −8 cap; crossing head.Y − 200 (STRICT <, TOP edges)
    /// re-targets and dashes at 22 toward the player's centre.
    #[test]
    fn the_rise_and_the_22_speed_dash() {
        let w = air();
        let c = ctx();
        let p = [cand(1830.0, 1500.0)];
        // Below the gate (y 800 == head.Y − 200): still rising.
        let mut n = saw(1814.0, 800.0);
        n.ai[2] = 1.0;
        n.vx = 2.0;
        n.vy = -7.95;
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 1.0, "800 is not < 800 (:28166)");
        assert!((n.vx - 1.9).abs() < 1e-5, "the 0.95 damp");
        assert_eq!(n.vy, -8.0, "the −8 cap (:28162-28165)");
        // Above the gate (y 799): the dash. Player centre (1840, 1521),
        // saw centre (1840, 825): straight down at 22.
        let mut n2 = saw(1814.0, 799.0);
        n2.ai[2] = 1.0;
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 2.0);
        assert!(n2.vx.abs() < 1e-3, "{}", n2.vx);
        assert!((n2.vy - 22.0).abs() < 1e-3, "{}", n2.vy);
        assert!(n2.net_update);
        assert_eq!(n2.target, 0, "TargetClosest ran (:28168)");
    }

    /// `NPC.cs:28179-28185` — the dash ends below the player's TOP
    /// edge OR on any upward velocity; otherwise it holds.
    #[test]
    fn the_dash_ends_below_the_player_or_on_the_upswing() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        // State 2 reads `players[target]` WITHOUT re-targeting
        // (`:28181`); the dash (state 1) set it. Stand in with slot 0.
        let mut n = saw(1814.0, 1501.0); // y > player.y
        n.ai[2] = 2.0;
        n.target = 0;
        n.vy = 5.0;
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 3.0);
        // Above the player but rising: also ends.
        let mut n2 = saw(1814.0, 1400.0);
        n2.ai[2] = 2.0;
        n2.target = 0;
        n2.vy = -0.5;
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 3.0);
        // Above and still falling: holds.
        let mut n3 = saw(1814.0, 1400.0);
        n3.ai[2] = 2.0;
        n3.target = 0;
        n3.vy = 5.0;
        style33_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.ai[2], 2.0);
    }

    /// `NPC.cs:28186-28239` — state 4 chases at the 7-speed aim (no
    /// dead branch), resets to 0 at 600, and ends on the dock rotation.
    #[test]
    fn the_state_4_chase_resets_at_600() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        let mut n = saw(1814.0, 1204.0);
        n.ai[2] = 4.0;
        n.ai[3] = 599.0;
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0);
        assert!(n.net_update);
        assert_eq!(n.rotation, 1.57, "the dock rotation (:28238)");
        // A fresh state-4 tick steps toward the player (left and below).
        let mut n2 = saw(1814.0, 1204.0);
        n2.ai[2] = 4.0;
        style33_step(&mut n2, &w, &c, &p);
        assert!((n2.vx - (-0.05)).abs() < 1e-5);
        assert!((n2.vy - 0.05).abs() < 1e-5);
    }

    /// `NPC.cs:28240-28243` — state 5 resets only when the motion has
    /// CROSSED the player's centre in the same direction.
    #[test]
    fn the_state_5_crossed_player_reset() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)]; // centre x 1510
        // State 5 reads `players[target]` WITHOUT re-targeting
        // (`:28240`); stand in with slot 0.
        // Moving right, centre 1511 > 1510: reset.
        let mut n = saw(1485.0, 1204.0);
        n.ai[2] = 5.0;
        n.target = 0;
        n.vx = 3.0;
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 0.0);
        // Moving right but still left of the player: holds.
        let mut n2 = saw(1400.0, 1204.0);
        n2.ai[2] = 5.0;
        n2.target = 0;
        n2.vx = 3.0;
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 5.0);
        // Moving left past the centre: reset.
        let mut n3 = saw(1400.0, 1204.0);
        n3.ai[2] = 5.0;
        n3.target = 0;
        n3.vx = -3.0;
        style33_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.ai[2], 0.0);
    }

    /// `NPC.cs:27971-28021` — the desync return chases the head with
    /// the 0.96 against-motion damp, the 0.1/0.5 steps and the 8/12
    /// one-sided caps.
    #[test]
    fn the_desync_return_damps_and_caps() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        // Far below-right of the head: rise and move left.
        let mut n = saw(2500.0, 1500.0);
        n.ai[2] = 99.0;
        n.vx = 10.0; // moving right (away): damped
        n.vy = 1.0; // moving down (away): damped
        style33_step(&mut n, &w, &c, &p);
        assert!((n.vx - (10.0 * 0.96 - 0.5)).abs() < 1e-4, "{}", n.vx);
        assert!((n.vy - (1.0 * 0.96 - 0.1)).abs() < 1e-5, "{}", n.vy);
        // The caps are one-sided and run AFTER the damp + step: a
        // falling vy 9 damps to 8.64, steps to 8.54, then caps to 8.
        let mut n2 = saw(2500.0, 1500.0);
        n2.ai[2] = 99.0;
        n2.vy = 9.0;
        style33_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.vy, 8.0, "the 8 cap (:27980-27983)");
        let mut n3 = saw(2500.0, 1500.0);
        n3.ai[2] = 99.0;
        n3.vx = 14.0; // 14·0.96 − 0.5 = 12.94 → the 12 cap
        style33_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.vx, 12.0, "the 12 cap (:28004-28007)");
    }

    /// `NPC.cs:27946` — the head's half-width is INTEGER division:
    /// an odd width 81 halves to 40, not 40.5. Pin it through state
    /// 99's X comparison at the exact tie point: the saw's centre x
    /// 2040.5 vs the int-divided head centre 2040 → pushed left; a
    /// float half would tie and hold. The saw sits far below the head
    /// so the dock distance (> 400) keeps state 99 armed.
    #[test]
    fn the_head_half_width_is_integer_division() {
        let w = air();
        let mut c = ctx();
        c.head_width = 81; // int 81 / 2 = 40 → headCentre 2040
        let p = [cand(1500.0, 1500.0)];
        // Saw centre (2040.5, 1726): dock (1840, 1230), distance ~535
        // (> 400, holds 99); centre x 2040.5 > headCentre 2040.
        let mut n = saw(2014.5, 1700.0);
        n.ai[2] = 99.0;
        style33_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 99.0, "the far dock keeps 99");
        assert_eq!(n.vx, -0.5, "the int-divided head centre (:27946)");
        // The float-half variant (40.5) would put the head centre at
        // 2040.5 — simulate it by shifting the head +0.5: the centres
        // tie, neither X comparison fires, vx holds at 0.
        let mut c2 = ctx();
        c2.head_width = 81;
        c2.head_x = 2000.5;
        let mut n2 = saw(2014.5, 1700.0);
        n2.ai[2] = 99.0;
        style33_step(&mut n2, &w, &c2, &p);
        assert_eq!(n2.vx, 0.0, "the tie holds — the 0.5 is the int cut");
    }

    /// RNG by construction: the C# arm (`NPC.cs:27943-28244`) contains
    /// no `Main.rand.Next` — the port takes no RNG parameter, so two
    /// identical inputs must produce identical state. This is the
    /// witness that the arm cannot perturb any RNG stream.
    #[test]
    fn the_arm_is_deterministic_without_an_rng() {
        let w = air();
        let mut c = ctx();
        c.head_ai1 = 1.0;
        let p = [cand(1500.0, 1500.0)];
        let mut a = saw(1814.0, 1204.0);
        let mut b = saw(1814.0, 1204.0);
        for _ in 0..50 {
            style33_step(&mut a, &w, &c, &p);
            style33_step(&mut b, &w, &c, &p);
        }
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
        assert_eq!(a.vx, b.vx);
        assert_eq!(a.vy, b.vy);
        assert_eq!(a.hp, b.hp);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(a.sprite_dir, b.sprite_dir);
        assert_eq!(a.ai, b.ai);
        assert_eq!(a.target, b.target);
        assert_eq!(a.net_update, b.net_update);
        assert_eq!(a.wants_hostile_shot, b.wants_hostile_shot);
        assert!(a.wants_hostile_shot.is_empty(), "the saw fires nothing");
    }
}
