//! The aiStyle-34/35/36 arms — the Skeletron Prime Vice, Cannon and Laser
//! (`NPC.AI()`'s inline `else if (this.aiStyle == 34/35/36)` blocks,
//! `NPC.cs:28245-28521` / `:28522-28756` / `:28757-28990`).
//!
//! S1-D-3 (prime-limbs round, code-only mode): this module is the faithful
//! 1:1 port of the three arms as pure step functions over `Npc` + a shared
//! context seam.
//!
//! S1-D-3 prime wiring (this round): WIRED — the style axis in
//! `npc.rs::update_rng` claims aiStyles 34/35/36 (`34 | 35 | 36 =>
//! ai_prime_limb_family`), which fills `PrimeLimbContext` from the sim's
//! head write-through (`world_sim/npcs.rs`'s `prime_head_*` fields) and
//! integrates `position += velocity` after the arm (`UpdateNPC`,
//! `NPC.cs:90498` — the pure fns leave it to the caller). The v1
//! `npc.rs::update_prime_limb` stand-in now serves ONLY the saw (129,
//! aiStyle 33) until its own wiring round; the head (127, `update_prime`)
//! and the saw's share are byte-identical. The module's landing round
//! followed the sibling pattern (`npc_ai_style28.rs`, commit `040df4c5`,
//! and the saw's `npc_ai_style33.rs`, commit `e5faaf42`): module + tests
//! first, dispatch claim second.
//!
//! Ids + styles VERIFIED against the source (the task brief's ids were off
//! by one — 127 is the HEAD, not the cannon):
//! * `NPCID.PrimeCannon = 128` (`ID/NPCID.cs:10933`), `SetDefaults`
//!   `NPC.cs:10573-10587`: 52×52, **aiStyle 35**, damage 30, defense 23,
//!   lifeMax 7000, `NPCHit4`/`NPCDeath14`, noGravity/noTileCollide, kb 0,
//!   netAlways.
//! * `NPCID.PrimeVice = 130` (`ID/NPCID.cs:10937`), `SetDefaults`
//!   `NPC.cs:10603-10617`: 52×52, **aiStyle 34**, damage 52, defense 34,
//!   lifeMax 9000, same flags.
//! * `NPCID.PrimeLaser = 131` (`ID/NPCID.cs:10939`), `SetDefaults`
//!   `NPC.cs:10618-10632`: 52×52, **aiStyle 36**, damage 29, defense 20,
//!   lifeMax 6000, same flags.
//! * (`NPCID.SkeletronPrime = 127`, `ID/NPCID.cs:10931`, aiStyle 32 — the
//!   head; `NPCID.PrimeSaw = 129`, aiStyle 33 — the sibling's arm.)
//! The port's static tables already carry the styles (`npc/data.rs:1952-
//! 1956`: `PrimeVice => 34`, `PrimeCannon => 35`, `PrimeLaser => 36`).
//! There are NO per-type branches inside any of the three arms — each
//! block below IS its family.
//!
//! The machines (`ai[2]` is the state, `ai[3]` the per-state counter,
//! `localAI[0]` the cannon/laser shot cadence):
//! * Vice (34, `:28245-28521`): the saw's dock/desync/despawn prologue
//!   (`:28247-28272`) + state 99's 0.96-damp return (`:28273-28323`), the
//!   0/3 block's DOUBLE `TargetClosest` (`:28332-28333`) + the 12-speed
//!   |vx|+|vy| < 2 snap-vs-0.97-damp chase (`:28344-28361`), the hover's
//!   600-tick `ai[2]++` + the 300/230 TOP-edge Y band + the asymmetric
//!   centre X band (headCentre+250 / headCentre, 0.3/-0.2 steps, ±9/-8
//!   caps, `:28373-28427`), the dock-rotation overwrite (`:28429-28433`);
//!   state 1's rise on the WIDER 280 dock with the head-velocity X blend
//!   (`:28435-28465`), state 2's four-dash cycle (`:28466-28481`), state
//!   4's head-velocity Y blend + ±500 centre gate + 17-speed side dash
//!   (`:28482-28507`), state 5's playerCentre−100 gate + four-cycle
//!   (`:28508-28520`).
//! * Cannon (35, `:28522-28756`): NO desync hysteresis and NO 99 state —
//!   only 0 (hover/fire) and 1 (bombard), everything else RETURNS
//!   (`:28679-28682`). State 0's head-gated drift (`localAI[0] += 2` on
//!   attack `:28543`, the −100 Y band ±6, the −120·side X point ±8; the
//!   hover's 1100-tick jump to 1 `:28595-28602`, the −150 Y band ±3, the
//!   +200/+160 X band ±8), the dock rotation (`:28652-28656`), then the
//!   UNCONDITIONAL 140-tick bomb at the REVERSED dock vector (`:28657-
//!   28675`). State 1's 300-tick return-to-0 (`:28683-28690`), the
//!   head-X/player-Y−80 6-speed aim with the OVERWRITTEN dead Y store
//!   (`:28692-28694`), the post-steer `TargetClosest` (`:28730`), the
//!   40-tick bomb at the player (`:28736-28753`).
//! * Laser (36, `:28757-28990`): also only 0/3 and 1 (`:28914-28917`
//!   returns). State 0/3's head-gated drift (`localAI[0] += 3` `:28778`,
//!   the −100 Y band ±6, the −120·side X point ±8; the hover's 800-tick
//!   `ai[2]++` `:28830-28836`, the −100 Y band ±3 at 0.1 steps, the
//!   −180·side X point ±8 at 0.14 steps), then the UNCONDITIONAL
//!   `TargetClosest` + player aim (`:28886-28891`) + the 200-tick laser
//!   volley (`:28892-28910`). State 1's 200-tick return-to-0
//!   (`:28918-28925`), the playerCentre−350/−20 7-speed aim
//!   (`:28926-28963`), the post-steer `TargetClosest` (`:28964`), the
//!   80-tick laser volley (`:28970-28988`).
//!
//! RNG by construction (every `Next` cited):
//! * Vice (34): the arm draws NO `Main.rand` — every counter is
//!   deterministic; the fn takes no RNG parameter (witness test).
//! * Cannon (35): two draws per bomb, X then Y, `Next(-40, 41)` each
//!   (`:28669-28670` the reversed-dock bomb, `:28748-28749` the player
//!   bomb) — the port's `UnifiedRandom::next_range(-40, 41)` is the exact
//!   `Next(min, maxExclusive)` stream (`UnifiedRandom.cs:120-132`).
//! * Laser (36): the same two-draw X-then-Y `Next(-40, 41)` pair per
//!   volley (`:28904-28905` / `:28982-28983`).
//! Both shooter arms gate the draws on `Main.netMode != 1` AND the
//! `localAI[0]` STRICT `>` threshold — a client, or a non-firing tick,
//! draws NOTHING (the stream-clone witness tests pin this).
//! The head-gone despawn's `HitEffect()` (`:28269`/`:28531`/`:28766`)
//! reaches the type-128..131 gore arm (`NPC.cs:89433-89469`) whose four
//! `Main.rand.Next(61, 64)` picks (`:89452/:89456/:89460/:89464`) are
//! COSMETIC gore — the dust/gore skip precedent (S1-D-3e/f/g and the
//! sibling saw round), so the despawn is the hp-0 convention alone.
//!
//! Faithfulness notes vs the live `update_prime_limb` (the wiring round
//! replaces each limb's share of it with its arm):
//! * The live arm has no state machine — one tether drift plus per-limb
//!   timers. The C# vice has SEVEN states, the cannon/laser three each.
//! * The cannon's live 40-tick player bomb is the C#'s state-1 cadence
//!   ONLY — state 0 fires every 140 ticks at the REVERSED dock vector
//!   (`:28667-28668`), not at the player, and the hover jump is 1100
//!   ticks (`:28596`), not the live arm's timer-less drift.
//! * The laser's live 80-tick volley is the C#'s state-1 cadence ONLY —
//!   state 0/3 fires every 200 ticks (`:28895`) after the hover's
//!   800-tick advance (`:28831`), and both volleys step the spawn point
//!   EIGHT velocity units ahead (`:28906-28907`/`:28984-28985`), not zero.
//! * The vice's live rise is the saw's 0.95/−0.1 shape — the C# damps
//!   only downward Y at 0.9 (`:28437-28440`), blends X with the head's
//!   velocity (`(vx·5 + head.vx)/6`, `:28446`), steps ±0.5 with the −9
//!   cap, and rises on the WIDER 280 dock (`:28442`).
//! * The desync hysteresis (vice only) is STRICT both ways: `> 800` arms
//!   99, `< 400` disarms (`:28254`/`:28259`).
//! * `spriteDirection = -(int)ai[0]` — the TRUNCATED int negation
//!   (`:28247`/`:28524`/`:28759`), written FIRST in all three arms,
//!   BEFORE the despawn early-out.
//! * The head-gone gate is `!active || aiStyle != 32` — any other style
//!   in the head slot despawns the limb; `ai[2] += 10` per tick,
//!   `> 50 || netMode != 2` kills (single-player dies on the first tick).
//! * The centre comparisons use INTEGER half-widths (`(float)(width/2)`
//!   on both entities) while the dock/aim vectors use the FLOAT centre
//!   (`width * 0.5f`) — two different centres, kept distinct.
//! * `Math.Sqrt`/`Math.Atan2` are DOUBLE calls on f32 operands in the C#
//!   — the port computes `(a as f64).sqrt() as f32` /
//!   `(y as f64).atan2(x as f64) as f32` (the f64-on-f32 pattern), NOT an
//!   f32 `sqrt`/`atan2`. The sums/products stay f32; the `12f / (float)
//!   Math.Sqrt(...)` divisions are f32 divides of an f64-rounded sqrt.
//! * Dead stores kept annotated: the vice's `num516` (`:28432`),
//!   `num519`'s first value (`:28444`, reused for the dash scale at
//!   `:28460`), `num522`'s first value (`:28487`, reused at `:28502`),
//!   and the cannon's `num531` head-Y read OVERWRITTEN by the
//!   player-Y−80 read (`:28693-28694`) — kept in order. The cannon's
//!   `num525` (`:28655`) and `num532` (`:28734`), and the laser's
//!   `num539`/`num546`, ARE read (the shot normalisers) — not dead.
//! * The normalisers DIVIDE by the distance with no epsilon floor — a
//!   zero distance yields the C#'s NaN/Inf verbatim (the live arm floors
//!   at 0.001). The cannon's reversed vector is `(0f - num) * scale`,
//!   kept verbatim.
//! * The arms do NOT integrate `position += velocity` — all three are
//!   noTileCollide, so `UpdateNPC` integrates after `AI()`
//!   (`NPC.cs:90498`); the caller owns the step.
//! * The despawn's fall-through is observationally identical to the
//!   early return: `ai[2]` is 10 or ≥ 60, so no state branch matches
//!   (`0/1/2/3/4/5/99` are all the tests; the cannon/laser `else`
//!   returns on `!= 1` anyway).
//! * `EncourageDespawn(10)` rides `Npc::encourage_despawn`
//!   (`NPC.cs:7249-7256` — only ever lowers the window).
//! * The shots ride `Npc::wants_hostile_shot` (`(type, x, y, vx, vy,
//!   damage)` — the style28 seam); the C#'s knockback `0f` and
//!   `Main.myPlayer` owner are the tuple's documented non-carried
//!   fields. Cannon bombs: type 102 (`ProjectileID.BombSkeletronPrime`,
//!   `ID/ProjectileID.cs:758`, aiStyle 16 — the rising bomb,
//!   `Projectile.cs:22496-22503`), damage 0. Laser: type 100
//!   (`ProjectileID.DeathLaser`, `ID/ProjectileID.cs:754`, aiStyle 1),
//!   damage 25 — the port's `projectile::DESTROYER_LASER` constant
//!   carries the same id (`projectile/data.rs:102`).
//! * `TargetClosest()` rides `npc_target::target_closest` (S1-D-3p) with
//!   the arm's own C# style and `faceTarget = true` (the default):
//!   vice `:28332-28333` (TWICE, kept), `:28455`, `:28497`; laser
//!   `:28886`, `:28964`. The cannon arm calls NONE.

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The head (`Main.npc[(int)this.ai[1]]`) reads the three arms make. The
/// caller (the future style-axis dispatch) fills it from the head's slot
/// — the port's `World.prime_*` write-through carries only the position
/// today, so the seam names every field the C# touches. Same shape as
/// the sibling's `Style33Context` plus the head VELOCITY the vice's
/// blends read (`:28446`/`:28489`).
#[derive(Clone, Copy, Debug)]
pub struct PrimeLimbContext {
    /// `Main.npc[(int)ai[1]].active` (`NPC.cs:28263`/`:28525`/`:28760`).
    pub head_active: bool,
    /// `Main.npc[(int)ai[1]].aiStyle` — must be 32 (the head).
    pub head_ai_style: i32,
    /// `Main.npc[(int)ai[1]].position` (the dock reads and every band).
    pub head_x: f32,
    pub head_y: f32,
    /// `Main.npc[(int)ai[1]].width` — an INT in the C# (`width / 2` is
    /// INTEGER division in every centre comparison).
    pub head_width: i32,
    /// `Main.npc[(int)ai[1]].velocity` — the vice's rise/fly blends
    /// (`:28446`/`:28489`); the cannon/laser never read it.
    pub head_vx: f32,
    pub head_vy: f32,
    /// `Main.npc[(int)ai[1]].ai[1]` — the head's phase: 0 hover,
    /// 1 charge, 2 rage, 3 despawn (`:28326`/`:28537`/`:28772`).
    pub head_ai1: f32,
    /// `Main.netMode` — 0 single-player, 1 client, 2 server. Gates the
    /// despawn (`!= 2`) and the shot draws (`!= 1`).
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for `TargetClosest`'s `targetSetFrame`
    /// stamp (`NPC.cs:77780`).
    pub ticker: u64,
}

/// The dock vector `(headCentre − offset·ai[0], head.Y + 230)` relative
/// to the limb's FLOAT centre (`width * 0.5f`) — recomputed at each C#
/// site. `offset` is 200 everywhere except the vice's state-1 rise
/// (280, `:28442`). The head half-width is INTEGER division.
fn dock_vector(npc: &Npc, ctx: &PrimeLimbContext, offset: f32) -> (f32, f32) {
    let (w, h) = npc.size();
    let cx = npc.x + w * 0.5;
    let cy = npc.y + h * 0.5;
    let head_cx = ctx.head_x + (ctx.head_width / 2) as f32;
    (head_cx - offset * npc.ai[0] - cx, ctx.head_y + 230.0 - cy)
}

/// `(float)Math.Sqrt(x*x + y*y)` — the f32 sum under a DOUBLE sqrt,
/// rounded back to f32 (the C#'s `Math.Sqrt(double)` call).
fn cs_sqrt(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// `(float)Math.Atan2(y, x)` — the DOUBLE atan2 on the f32 operands.
fn cs_atan2(y: f32, x: f32) -> f32 {
    (y as f64).atan2(x as f64) as f32
}

/// The kind/style-keyed dispatch — the future wiring round's entry
/// point. Returns `false` for any other kind (the saw rides its own
/// `npc_ai_style33.rs`). `rng` is `Main.rand`; only the cannon/laser
/// arms ever draw from it (the vice provably draws nothing).
pub fn prime_limb_step(
    npc: &mut Npc,
    world: &World,
    ctx: &PrimeLimbContext,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
) -> bool {
    use crate::npc::NpcKind;
    match npc.kind {
        NpcKind::PrimeVice => {
            style34_step(npc, world, ctx, players);
            true
        }
        NpcKind::PrimeCannon => {
            style35_step(npc, world, ctx, players, rng);
            true
        }
        NpcKind::PrimeLaser => {
            style36_step(npc, world, ctx, players, rng);
            true
        }
        _ => false,
    }
}

/// `NPC.AI()`'s aiStyle-34 arm — the Prime Vice (`NPC.cs:28245-28521`),
/// one tick. The grab-and-throw limb: dock hover → rise on the wide 280
/// dock → up to four 20-speed dashes at the player → the side sweep
/// (state 4/5's 17-speed dashes), then back to the dock.
pub fn style34_step(npc: &mut Npc, world: &World, ctx: &PrimeLimbContext, players: &[TargetCandidate]) {
    // `:28247` — `spriteDirection = -(int)this.ai[0]` (truncation),
    // written BEFORE the despawn gate.
    npc.sprite_dir = -(npc.ai[0] as i32);
    // `:28248-28251` — the dock distance for the desync gate.
    let (num508, num509) = dock_vector(npc, ctx, 200.0);
    let num510 = cs_sqrt(num508, num509);
    // `:28252-28262` — the hysteresis: STRICT > 800 arms state 99,
    // STRICT < 400 disarms; the band between holds the state.
    if npc.ai[2] != 99.0 {
        if num510 > 800.0 {
            npc.ai[2] = 99.0;
        }
    } else if num510 < 400.0 {
        npc.ai[2] = 0.0;
    }
    // `:28263-28272` — the head-gone despawn. `life = -1; HitEffect();
    // active = false` → the port's hp-0 convention; the HitEffect gore
    // (the four Next(61,64) picks, `:89452/:89456/:89460/:89464`) is
    // the cosmetic skip. The early return matches the fall-through:
    // ai[2] is 10 or ≥ 60, so no state branch below matches.
    if !ctx.head_active || ctx.head_ai_style != 32 {
        npc.ai[2] += 10.0;
        if npc.ai[2] > 50.0 || ctx.net_mode != 2 {
            npc.hp = 0;
            return;
        }
    }
    // The INTEGER half-width the centre comparisons use
    // (`(float)(base.width / 2)`, `:28299` etc.).
    let (w, _h) = npc.size();
    let half_w = (w as i32 / 2) as f32;
    let head_cx = ctx.head_x + (ctx.head_width / 2) as f32;
    if npc.ai[2] == 99.0 {
        // `:28273-28323` — the desync return: chase the head's TOP edge
        // on Y and CENTRE on X, damping only against the motion.
        if npc.y > ctx.head_y {
            // `:28275-28286`.
            if npc.vy > 0.0 {
                npc.vy *= 0.96;
            }
            npc.vy -= 0.1;
            if npc.vy > 8.0 {
                npc.vy = 8.0;
            }
        } else if npc.y < ctx.head_y {
            // `:28287-28298`.
            if npc.vy < 0.0 {
                npc.vy *= 0.96;
            }
            npc.vy += 0.1;
            if npc.vy < -8.0 {
                npc.vy = -8.0;
            }
        }
        if npc.x + half_w > head_cx {
            // `:28299-28310`.
            if npc.vx > 0.0 {
                npc.vx *= 0.96;
            }
            npc.vx -= 0.5;
            if npc.vx > 12.0 {
                npc.vx = 12.0;
            }
        }
        if npc.x + half_w < head_cx {
            // `:28311-28322`.
            if npc.vx < 0.0 {
                npc.vx *= 0.96;
            }
            npc.vx += 0.5;
            if npc.vx < -12.0 {
                npc.vx = -12.0;
            }
        }
    } else if npc.ai[2] == 0.0 || npc.ai[2] == 3.0 {
        // `:28324-28434` — the dock hover / attack chase, gated on the
        // head's phase.
        if ctx.head_ai1 == 3.0 {
            // `:28326-28329` — the head is despawning: follow it out.
            npc.encourage_despawn(10);
        }
        if ctx.head_ai1 != 0.0 {
            // `:28330-28370` — the head attacks: chase the player.
            // `:28332-28333` — TargetClosest TWICE, verbatim (the C#
            // scans the slots two times back to back; the second call
            // re-stamps the tracking values).
            target_closest(npc, world, players, true, 34, ctx.ticker);
            target_closest(npc, world, players, true, 34, ctx.ticker);
            let p = &players[npc.target as usize];
            if p.dead {
                // `:28334-28341` — a dead target: sink at 0.1 to 16.
                npc.vy += 0.1;
                if npc.vy > 16.0 {
                    npc.vy = 16.0;
                }
            } else {
                // `:28344-28368` — the 12-speed aim with INTEGER player
                // half-extents, then the |vx|+|vy| < 2 SNAP-vs-DAMP
                // switch (STRICT <, the C#'s Math.Abs sums in f32).
                let (cw, ch) = npc.size();
                let vector55 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
                let mut num511 = p.x + (p.width / 2) as f32 - vector55.0;
                let mut num512 = p.y + (p.height / 2) as f32 - vector55.1;
                let num513 = 12.0 / cs_sqrt(num511, num512);
                num511 *= num513;
                num512 *= num513;
                // `:28350` — the chase aim (OVERWRITTEN by the dock
                // rotation at `:28433` below; both writes kept).
                npc.rotation = cs_atan2(num512, num511) - 1.57;
                if npc.vx.abs() + npc.vy.abs() < 2.0 {
                    // `:28351-28357` — nearly still: snap onto the aim
                    // (the duplicate rotation write kept verbatim).
                    npc.rotation = cs_atan2(num512, num511) - 1.57;
                    npc.vx = num511;
                    npc.vy = num512;
                    npc.net_update = true;
                } else {
                    // `:28358-28361` — moving: damp BOTH components at
                    // 0.97 unconditionally (no against-motion test).
                    npc.vx *= 0.97;
                    npc.vy *= 0.97;
                }
                // `:28362-28368` — the 600-tick chase reset lives
                // INSIDE the alive branch (a dead target never ticks
                // it), back to state 0, NOT the hover's ++.
                npc.ai[3] += 1.0;
                if npc.ai[3] >= 600.0 {
                    npc.ai[2] = 0.0;
                    npc.ai[3] = 0.0;
                    npc.net_update = true;
                }
            }
        } else {
            // `:28371-28428` — the head hovers: hold the dock band.
            // `:28373-28379` — the 600-tick state advance FIRST.
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 600.0 {
                npc.ai[2] += 1.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
            // `:28380-28403` — the Y band compares the TOP edge against
            // head.Y + 300 / + 230; the 70-px gap is a dead band.
            if npc.y > ctx.head_y + 300.0 {
                if npc.vy > 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy -= 0.1;
                if npc.vy > 3.0 {
                    npc.vy = 3.0;
                }
            } else if npc.y < ctx.head_y + 230.0 {
                if npc.vy < 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy += 0.1;
                if npc.vy < -3.0 {
                    npc.vy = -3.0;
                }
            }
            // `:28404-28427` — the ASYMMETRIC X band: the right edge is
            // headCentre + 250, the left edge headCentre itself; the
            // damp is 0.94 and the steps/caps differ per side
            // (−0.3/+9 right, +0.2/−8 left).
            if npc.x + half_w > head_cx + 250.0 {
                if npc.vx > 0.0 {
                    npc.vx *= 0.94;
                }
                npc.vx -= 0.3;
                if npc.vx > 9.0 {
                    npc.vx = 9.0;
                }
            }
            if npc.x + half_w < head_cx {
                if npc.vx < 0.0 {
                    npc.vx *= 0.94;
                }
                npc.vx += 0.2;
                if npc.vx < -8.0 {
                    npc.vx = -8.0;
                }
            }
        }
        // `:28429-28433` — the dock rotation, recomputed and written
        // AFTER both sub-branches (it overwrites the chase aim).
        let (num514, num515) = dock_vector(npc, ctx, 200.0);
        // `:28432` — `num516` is a dead-store sqrt in the C# (computed,
        // never read); kept verbatim, no observable effect.
        let _num516 = cs_sqrt(num514, num515);
        npc.rotation = cs_atan2(num515, num514) + 1.57;
    } else if npc.ai[2] == 1.0 {
        // `:28435-28465` — the rise on the WIDE dock: damp only downward
        // Y at 0.9, blend X with the head's velocity, ±0.5 steps, −9 cap.
        if npc.vy > 0.0 {
            npc.vy *= 0.9;
        }
        // `:28441-28445` — the 280-offset dock (NOT 200).
        let (num517, num518) = dock_vector(npc, ctx, 280.0);
        // `:28444` — `num519`'s FIRST value is a dead-store sqrt (the
        // name is reused for the dash scale at `:28460` below).
        let _num519 = cs_sqrt(num517, num518);
        npc.rotation = cs_atan2(num518, num517) + 1.57;
        // `:28446-28452` — `(velocity.X * 5f + head.velocity.X) / 6f`.
        npc.vx = (npc.vx * 5.0 + ctx.head_vx) / 6.0;
        npc.vx += 0.5;
        npc.vy -= 0.5;
        if npc.vy < -9.0 {
            npc.vy = -9.0;
        }
        // `:28453-28464` — above head.Y − 280 (STRICT <, TOP edges):
        // re-target and dash at the player at 20.
        if npc.y < ctx.head_y - 280.0 {
            target_closest(npc, world, players, true, 34, ctx.ticker);
            npc.ai[2] = 2.0;
            let p = &players[npc.target as usize];
            let (cw, ch) = npc.size();
            let vector57 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
            let num517 = p.x + (p.width / 2) as f32 - vector57.0;
            let num518 = p.y + (p.height / 2) as f32 - vector57.1;
            let num519 = 20.0 / cs_sqrt(num517, num518);
            npc.vx = num517 * num519;
            npc.vy = num518 * num519;
            npc.net_update = true;
        }
    } else if npc.ai[2] == 2.0 {
        // `:28466-28481` — the dash ends below the player's TOP edge or
        // on any upward velocity: four dashes (ai[3] counts), then 3.
        let p = &players[npc.target as usize];
        if npc.y > p.y || npc.vy < 0.0 {
            if npc.ai[3] >= 4.0 {
                npc.ai[2] = 3.0;
                npc.ai[3] = 0.0;
            } else {
                npc.ai[2] = 1.0;
                npc.ai[3] += 1.0;
            }
        }
    } else if npc.ai[2] == 4.0 {
        // `:28482-28507` — the side sweep: dock rotation, Y blended with
        // the head's velocity, X pushed right at 0.5 to 12; once the
        // centre leaves headCentre ± 500, dash at the player at 17.
        let (num520, num521) = dock_vector(npc, ctx, 200.0);
        // `:28487` — `num522`'s FIRST value is a dead-store sqrt (the
        // name is reused for the dash scale at `:28502` below).
        let _num522 = cs_sqrt(num520, num521);
        npc.rotation = cs_atan2(num521, num520) + 1.57;
        npc.vy = (npc.vy * 5.0 + ctx.head_vy) / 6.0;
        npc.vx += 0.5;
        if npc.vx > 12.0 {
            npc.vx = 12.0;
        }
        // `:28495` — the ±500 gate compares CENTRES (integer halves),
        // both sides STRICT.
        if npc.x + half_w < head_cx - 500.0 || npc.x + half_w > head_cx + 500.0 {
            target_closest(npc, world, players, true, 34, ctx.ticker);
            npc.ai[2] = 5.0;
            let p = &players[npc.target as usize];
            let (cw, ch) = npc.size();
            let vector58 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
            let num520 = p.x + (p.width / 2) as f32 - vector58.0;
            let num521 = p.y + (p.height / 2) as f32 - vector58.1;
            let num522 = 17.0 / cs_sqrt(num520, num521);
            npc.vx = num520 * num522;
            npc.vy = num521 * num522;
            npc.net_update = true;
        }
    } else if npc.ai[2] == 5.0 && npc.x + half_w < players[npc.target as usize].x + (players[npc.target as usize].width / 2) as f32 - 100.0 {
        // `:28508-28520` — the side dash's end gate rides the ELSE-IF
        // CONDITION itself: only while the centre is left of
        // playerCentre − 100 (STRICT <). Four passes (ai[3]), then 0.
        if npc.ai[3] >= 4.0 {
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
        } else {
            npc.ai[2] = 4.0;
            npc.ai[3] += 1.0;
        }
    }
    // No `position += velocity` here: the vice is noTileCollide, so
    // `UpdateNPC` integrates after `AI()` (`NPC.cs:90498`).
}

/// `NPC.AI()`'s aiStyle-35 arm — the Prime Cannon (`NPC.cs:28522-28756`),
/// one tick. The bombard limb: dock hover with a 140-tick bomb lobbed
/// BACK along the dock vector, then the 1100-tick jump into the
/// player-tracking bombard state (40-tick bombs). NO desync hysteresis,
/// NO state 99 — any state other than 0/1 returns (`:28679-28682`).
/// `rng` is `Main.rand`: two `Next(-40, 41)` draws (X then Y) per fired
/// bomb (`:28669-28670` / `:28748-28749`), zero otherwise.
pub fn style35_step(
    npc: &mut Npc,
    world: &World,
    ctx: &PrimeLimbContext,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
) {
    // `:28524` — the truncated seat flip, BEFORE the despawn gate.
    npc.sprite_dir = -(npc.ai[0] as i32);
    // `:28525-28534` — the head-gone despawn (the hp-0 convention; the
    // HitEffect gore is the cosmetic skip). Early return = the C#'s
    // fall-through: ai[2] is 10 or ≥ 60, and the else arm returns on
    // `!= 1` anyway.
    if !ctx.head_active || ctx.head_ai_style != 32 {
        npc.ai[2] += 10.0;
        if npc.ai[2] > 50.0 || ctx.net_mode != 2 {
            npc.hp = 0;
            return;
        }
    }
    let (w, _h) = npc.size();
    let half_w = (w as i32 / 2) as f32;
    let head_cx = ctx.head_x + (ctx.head_width / 2) as f32;
    if npc.ai[2] == 0.0 {
        // `:28535-28676` — the dock hover + the reversed-dock bomb.
        if ctx.head_ai1 == 3.0 {
            // `:28537-28540`.
            npc.encourage_despawn(10);
        }
        if ctx.head_ai1 != 0.0 {
            // `:28541-28592` — the head attacks: tighten onto the
            // −100/−120·side anchor, and pre-load the shot cadence.
            npc.local_ai[0] += 2.0;
            // `:28544-28567` — the Y point head.Y − 100 (TOP edges,
            // STRICT both ways — equality holds), ±0.07 steps, ±6 caps.
            if npc.y > ctx.head_y - 100.0 {
                if npc.vy > 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy -= 0.07;
                if npc.vy > 6.0 {
                    npc.vy = 6.0;
                }
            } else if npc.y < ctx.head_y - 100.0 {
                if npc.vy < 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy += 0.07;
                if npc.vy < -6.0 {
                    npc.vy = -6.0;
                }
            }
            // `:28568-28591` — the X point headCentre − 120·ai[0]
            // (CENTRES, integer halves), ±0.1 steps, ±8 caps.
            if npc.x + half_w > head_cx - 120.0 * npc.ai[0] {
                if npc.vx > 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx -= 0.1;
                if npc.vx > 8.0 {
                    npc.vx = 8.0;
                }
            }
            if npc.x + half_w < head_cx - 120.0 * npc.ai[0] {
                if npc.vx < 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx += 0.1;
                if npc.vx < -8.0 {
                    npc.vx = -8.0;
                }
            }
        } else {
            // `:28593-28651` — the head hovers: the loose +200/+160
            // band and the 1100-tick jump into the bombard state.
            // `:28595-28602` — the jump FIRST (it zeroes localAI[0]
            // BEFORE the shot block below ticks it).
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 1100.0 {
                npc.local_ai[0] = 0.0;
                npc.ai[2] = 1.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
            // `:28603-28626` — the Y point head.Y − 150, ±0.04, ±3.
            if npc.y > ctx.head_y - 150.0 {
                if npc.vy > 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy -= 0.04;
                if npc.vy > 3.0 {
                    npc.vy = 3.0;
                }
            } else if npc.y < ctx.head_y - 150.0 {
                if npc.vy < 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy += 0.04;
                if npc.vy < -3.0 {
                    npc.vy = -3.0;
                }
            }
            // `:28627-28650` — the X band headCentre + 200 / + 160
            // (BOTH on the right of the head, regardless of the seat),
            // ±0.2 steps, ±8 caps.
            if npc.x + half_w > head_cx + 200.0 {
                if npc.vx > 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx -= 0.2;
                if npc.vx > 8.0 {
                    npc.vx = 8.0;
                }
            }
            if npc.x + half_w < head_cx + 160.0 {
                if npc.vx < 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx += 0.2;
                if npc.vx < -8.0 {
                    npc.vx = -8.0;
                }
            }
        }
        // `:28652-28656` — the dock rotation. `num525`'s sqrt is NOT
        // dead here: the shot below reuses it as the normaliser
        // denominator (`:28666`).
        let (mut num523, mut num524) = dock_vector(npc, ctx, 200.0);
        let mut num525 = cs_sqrt(num523, num524);
        npc.rotation = cs_atan2(num524, num523) + 1.57;
        // `:28657-28675` — the bomb: server/SP only, STRICT > 140 on
        // the cadence (the attack gate pre-loads it +2 per tick).
        if ctx.net_mode != 1 {
            npc.local_ai[0] += 1.0;
            if npc.local_ai[0] > 140.0 {
                npc.local_ai[0] = 0.0;
                // `:28663-28665` — speed 12, damage 0, type 102
                // (`ProjectileID.BombSkeletronPrime`).
                let num526 = 12.0f32;
                let num527 = 0i32;
                let num528 = crate::projectile::BOMB_SKELETRON;
                // `:28666-28668` — the REVERSED dock vector:
                // `(0f - num) * scale`, verbatim.
                num525 = num526 / num525;
                num523 = (0.0 - num523) * num525;
                num524 = (0.0 - num524) * num525;
                // `:28669-28670` — the two draws, X then Y.
                num523 += rng.next_range(-40, 41) as f32 * 0.01;
                num524 += rng.next_range(-40, 41) as f32 * 0.01;
                // `:28671-28673` — the spawn point steps FOUR velocity
                // units ahead of the limb's centre.
                let (cw, ch) = npc.size();
                let spawn_x = npc.x + cw * 0.5 + num523 * 4.0;
                let spawn_y = npc.y + ch * 0.5 + num524 * 4.0;
                npc.wants_hostile_shot
                    .push((num528, spawn_x, spawn_y, num523, num524, num527));
            }
        }
    } else {
        // `:28677-28755` — everything but state 1 returns.
        if npc.ai[2] != 1.0 {
            return;
        }
        // `:28683-28690` — the 300-tick bombard window.
        npc.ai[3] += 1.0;
        if npc.ai[3] >= 300.0 {
            npc.local_ai[0] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        // `:28691-28697` — the 6-speed aim: X at the HEAD's centre, Y
        // at the PLAYER's centre − 80. `:28693`'s head-Y read is a
        // DEAD STORE — `:28694` overwrites num531 before any use; both
        // reads kept in order.
        let (cw, ch) = npc.size();
        let vector60 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
        let mut num530 = head_cx - vector60.0;
        let _dead_num531 = ctx.head_y - vector60.1; // `:28693` — dead store, kept.
        let p = &players[npc.target as usize];
        let mut num531 = p.y + (p.height / 2) as f32 - 80.0 - vector60.1;
        let num532 = 6.0 / cs_sqrt(num530, num531);
        num530 *= num532;
        num531 *= num532;
        // `:28698-28729` — the four independent comparisons: X damps
        // 0.9 / steps 0.04, Y damps 0.9 / steps 0.08 (no caps).
        if npc.vx > num530 {
            if npc.vx > 0.0 {
                npc.vx *= 0.9;
            }
            npc.vx -= 0.04;
        }
        if npc.vx < num530 {
            if npc.vx < 0.0 {
                npc.vx *= 0.9;
            }
            npc.vx += 0.04;
        }
        if npc.vy > num531 {
            if npc.vy > 0.0 {
                npc.vy *= 0.9;
            }
            npc.vy -= 0.08;
        }
        if npc.vy < num531 {
            if npc.vy < 0.0 {
                npc.vy *= 0.9;
            }
            npc.vy += 0.08;
        }
        // `:28730-28735` — the re-target AFTER the steer, then the
        // player aim rotation (− 1.57). `num532`'s reused sqrt IS read
        // (the shot normaliser below).
        target_closest(npc, world, players, true, 35, ctx.ticker);
        let vector60 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
        let p = &players[npc.target as usize];
        let mut num530 = p.x + (p.width / 2) as f32 - vector60.0;
        let mut num531 = p.y + (p.height / 2) as f32 - vector60.1;
        let mut num532 = cs_sqrt(num530, num531);
        npc.rotation = cs_atan2(num531, num530) - 1.57;
        // `:28736-28754` — the 40-tick player bomb (STRICT > 40).
        if ctx.net_mode != 1 {
            npc.local_ai[0] += 1.0;
            if npc.local_ai[0] > 40.0 {
                npc.local_ai[0] = 0.0;
                // `:28742-28744` — speed 10, damage 0, type 102.
                let num533 = 10.0f32;
                let num534 = 0i32;
                let num535 = crate::projectile::BOMB_SKELETRON;
                num532 = num533 / num532;
                num530 *= num532;
                num531 *= num532;
                // `:28748-28749` — the two draws, X then Y.
                num530 += rng.next_range(-40, 41) as f32 * 0.01;
                num531 += rng.next_range(-40, 41) as f32 * 0.01;
                // `:28750-28752` — FOUR velocity units ahead.
                let spawn_x = vector60.0 + num530 * 4.0;
                let spawn_y = vector60.1 + num531 * 4.0;
                npc.wants_hostile_shot
                    .push((num535, spawn_x, spawn_y, num530, num531, num534));
            }
        }
    }
    // No `position += velocity` here: noTileCollide (`NPC.cs:90498`).
}

/// `NPC.AI()`'s aiStyle-36 arm — the Prime Laser (`NPC.cs:28757-28990`),
/// one tick. The beam limb: dock hover with a 200-tick DeathLaser volley
/// aimed at the player, the 800-tick `ai[2]++` into state 1 (the
/// player-flanking position at playerCentre − 350/− 20 with an 80-tick
/// volley), the 200-tick return to 0. NO desync hysteresis, NO state 99;
/// states other than 0/3/1 return (`:28914-28917`). `rng` is `Main.rand`:
/// two `Next(-40, 41)` draws (X then Y) per volley (`:28904-28905` /
/// `:28982-28983`), zero otherwise.
pub fn style36_step(
    npc: &mut Npc,
    world: &World,
    ctx: &PrimeLimbContext,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
) {
    // `:28759` — the truncated seat flip, BEFORE the despawn gate.
    npc.sprite_dir = -(npc.ai[0] as i32);
    // `:28760-28769` — the head-gone despawn (hp-0 convention; the
    // HitEffect gore is the cosmetic skip).
    if !ctx.head_active || ctx.head_ai_style != 32 {
        npc.ai[2] += 10.0;
        if npc.ai[2] > 50.0 || ctx.net_mode != 2 {
            npc.hp = 0;
            return;
        }
    }
    let (w, _h) = npc.size();
    let half_w = (w as i32 / 2) as f32;
    let head_cx = ctx.head_x + (ctx.head_width / 2) as f32;
    if npc.ai[2] == 0.0 || npc.ai[2] == 3.0 {
        // `:28770-28911` — the dock hover + the 200-tick volley.
        if ctx.head_ai1 == 3.0 {
            // `:28772-28775`.
            npc.encourage_despawn(10);
        }
        if ctx.head_ai1 != 0.0 {
            // `:28776-28827` — the head attacks: the −100/−120·side
            // anchor, and the shot cadence pre-loads +3 per tick.
            npc.local_ai[0] += 3.0;
            // `:28779-28802` — the Y point head.Y − 100 (TOP edges,
            // STRICT both ways), ±0.07 steps, ±6 caps.
            if npc.y > ctx.head_y - 100.0 {
                if npc.vy > 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy -= 0.07;
                if npc.vy > 6.0 {
                    npc.vy = 6.0;
                }
            } else if npc.y < ctx.head_y - 100.0 {
                if npc.vy < 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy += 0.07;
                if npc.vy < -6.0 {
                    npc.vy = -6.0;
                }
            }
            // `:28803-28826` — the X point headCentre − 120·ai[0]
            // (CENTRES, integer halves), ±0.1 steps, ±8 caps.
            if npc.x + half_w > head_cx - 120.0 * npc.ai[0] {
                if npc.vx > 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx -= 0.1;
                if npc.vx > 8.0 {
                    npc.vx = 8.0;
                }
            }
            if npc.x + half_w < head_cx - 120.0 * npc.ai[0] {
                if npc.vx < 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx += 0.1;
                if npc.vx < -8.0 {
                    npc.vx = -8.0;
                }
            }
        } else {
            // `:28828-28885` — the head hovers: the −100/−180·side
            // anchor and the 800-tick advance.
            // `:28830-28836` — the advance FIRST: ai[2]++ (0 → 1; from
            // 3 it would go to 4, which then returns forever — the C#
            // never enters 3 from inside this arm; kept verbatim).
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 800.0 {
                npc.ai[2] += 1.0;
                npc.ai[3] = 0.0;
                npc.net_update = true;
            }
            // `:28837-28860` — the Y point head.Y − 100, ±0.1, ±3.
            if npc.y > ctx.head_y - 100.0 {
                if npc.vy > 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy -= 0.1;
                if npc.vy > 3.0 {
                    npc.vy = 3.0;
                }
            } else if npc.y < ctx.head_y - 100.0 {
                if npc.vy < 0.0 {
                    npc.vy *= 0.96;
                }
                npc.vy += 0.1;
                if npc.vy < -3.0 {
                    npc.vy = -3.0;
                }
            }
            // `:28861-28884` — the X point headCentre − 180·ai[0],
            // ±0.14 steps, ±8 caps.
            if npc.x + half_w > head_cx - 180.0 * npc.ai[0] {
                if npc.vx > 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx -= 0.14;
                if npc.vx > 8.0 {
                    npc.vx = 8.0;
                }
            }
            if npc.x + half_w < head_cx - 180.0 * npc.ai[0] {
                if npc.vx < 0.0 {
                    npc.vx *= 0.96;
                }
                npc.vx += 0.14;
                if npc.vx < -8.0 {
                    npc.vx = -8.0;
                }
            }
        }
        // `:28886-28891` — the UNCONDITIONAL re-target + player aim
        // (both sub-branches reach it). `num539`'s sqrt IS read — the
        // shot normaliser below (`:28901`).
        target_closest(npc, world, players, true, 36, ctx.ticker);
        let (cw, ch) = npc.size();
        let vector61 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
        let p = &players[npc.target as usize];
        let mut num537 = p.x + (p.width / 2) as f32 - vector61.0;
        let mut num538 = p.y + (p.height / 2) as f32 - vector61.1;
        let mut num539 = cs_sqrt(num537, num538);
        npc.rotation = cs_atan2(num538, num537) - 1.57;
        // `:28892-28910` — the volley: server/SP only, STRICT > 200.
        if ctx.net_mode != 1 {
            npc.local_ai[0] += 1.0;
            if npc.local_ai[0] > 200.0 {
                npc.local_ai[0] = 0.0;
                // `:28898-28900` — speed 8, damage 25, type 100
                // (`ProjectileID.DeathLaser`; the port's constant is
                // named DESTROYER_LASER but carries the same id 100,
                // `projectile/data.rs:102`).
                let num540 = 8.0f32;
                let num541 = 25i32;
                let num542 = crate::projectile::DESTROYER_LASER;
                num539 = num540 / num539;
                num537 *= num539;
                num538 *= num539;
                // `:28904-28905` — the two draws, X then Y, ×0.05
                // (FIVE times the cannon's 0.01 jitter).
                num537 += rng.next_range(-40, 41) as f32 * 0.05;
                num538 += rng.next_range(-40, 41) as f32 * 0.05;
                // `:28906-28908` — the spawn point steps EIGHT velocity
                // units ahead (the cannon steps four).
                let spawn_x = vector61.0 + num537 * 8.0;
                let spawn_y = vector61.1 + num538 * 8.0;
                npc.wants_hostile_shot
                    .push((num542, spawn_x, spawn_y, num537, num538, num541));
            }
        }
    } else {
        // `:28912-28989` — everything but state 1 returns.
        if npc.ai[2] != 1.0 {
            return;
        }
        // `:28918-28925` — the 200-tick flank window.
        npc.ai[3] += 1.0;
        if npc.ai[3] >= 200.0 {
            npc.local_ai[0] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        // `:28926-28931` — the 7-speed aim at playerCentre − 350 on X,
        // playerCentre − 20 on Y (the flank point LEFT of the player,
        // regardless of the seat).
        let (cw, ch) = npc.size();
        let vector62 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
        let p = &players[npc.target as usize];
        let mut num544 = p.x + (p.width / 2) as f32 - 350.0 - vector62.0;
        let mut num545 = p.y + (p.height / 2) as f32 - 20.0 - vector62.1;
        let num546 = 7.0 / cs_sqrt(num544, num545);
        num544 *= num546;
        num545 *= num546;
        // `:28932-28963` — the four independent comparisons: X damps
        // 0.9 / steps 0.1, Y damps 0.9 / steps 0.03 (no caps).
        if npc.vx > num544 {
            if npc.vx > 0.0 {
                npc.vx *= 0.9;
            }
            npc.vx -= 0.1;
        }
        if npc.vx < num544 {
            if npc.vx < 0.0 {
                npc.vx *= 0.9;
            }
            npc.vx += 0.1;
        }
        if npc.vy > num545 {
            if npc.vy > 0.0 {
                npc.vy *= 0.9;
            }
            npc.vy -= 0.03;
        }
        if npc.vy < num545 {
            if npc.vy < 0.0 {
                npc.vy *= 0.9;
            }
            npc.vy += 0.03;
        }
        // `:28964-28969` — the re-target AFTER the steer, then the
        // player aim rotation. `num546`'s reused sqrt IS read (the
        // volley normaliser below).
        target_closest(npc, world, players, true, 36, ctx.ticker);
        let vector62 = (npc.x + cw * 0.5, npc.y + ch * 0.5);
        let p = &players[npc.target as usize];
        let mut num544 = p.x + (p.width / 2) as f32 - vector62.0;
        let mut num545 = p.y + (p.height / 2) as f32 - vector62.1;
        let mut num546 = cs_sqrt(num544, num545);
        npc.rotation = cs_atan2(num545, num544) - 1.57;
        // `:28970-28988` — the 80-tick volley (STRICT > 80).
        if ctx.net_mode != 1 {
            npc.local_ai[0] += 1.0;
            if npc.local_ai[0] > 80.0 {
                npc.local_ai[0] = 0.0;
                // `:28976-28978` — speed 10, damage 25, type 100.
                let num547 = 10.0f32;
                let num548 = 25i32;
                let num549 = crate::projectile::DESTROYER_LASER;
                num546 = num547 / num546;
                num544 *= num546;
                num545 *= num546;
                // `:28982-28983` — the two draws, X then Y, ×0.05.
                num544 += rng.next_range(-40, 41) as f32 * 0.05;
                num545 += rng.next_range(-40, 41) as f32 * 0.05;
                // `:28984-28986` — EIGHT velocity units ahead.
                let spawn_x = vector62.0 + num544 * 8.0;
                let spawn_y = vector62.1 + num545 * 8.0;
                npc.wants_hostile_shot
                    .push((num549, spawn_x, spawn_y, num544, num545, num548));
            }
        }
    }
    // No `position += velocity` here: noTileCollide (`NPC.cs:90498`).
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
    fn ctx() -> PrimeLimbContext {
        PrimeLimbContext {
            head_active: true,
            head_ai_style: 32,
            head_x: 2000.0,
            head_y: 1000.0,
            head_width: 80,
            head_vx: 0.0,
            head_vy: 0.0,
            head_ai1: 0.0,
            net_mode: 0,
            ticker: 0,
        }
    }

    /// A limb on the right seat (`ai[0] = 1`), 52x52. At (1814, 1204)
    /// its centre is (1840, 1230) - exactly the 200 dock (headCentre
    /// 2040 - 200, head.Y + 230).
    fn limb(kind: NpcKind, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(kind, x, y, 1, 0.0);
        n.ai[0] = 1.0;
        n.ai[1] = 0.0; // the head's slot (the ctx seam stands in)
        // The C# arms read `this.target` from the PREVIOUS state's
        // TargetClosest (states 2/5 and the cannon/laser steer read it
        // before re-targeting); `Npc::new` starts at the ClearTarget
        // 255, so the fixture stands in the acquired slot 0.
        n.target = 0;
        n
    }

    fn rng() -> UnifiedRandom {
        UnifiedRandom::new(12345)
    }

    // ---- the vice (34) ----

    /// `NPC.cs:28252-28262` - the vice's desync hysteresis is STRICT
    /// both ways: > 800 arms 99, < 400 disarms, the band holds.
    #[test]
    fn the_vice_desync_hysteresis_is_strict_both_ways() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        let mut n = limb(NpcKind::PrimeVice, 1814.0 + 800.0, 1204.0);
        style34_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 0.0, "800 is not > 800 (:28254)");
        let mut n2 = limb(NpcKind::PrimeVice, 1814.0 + 801.0, 1204.0);
        style34_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 99.0);
        let mut n3 = limb(NpcKind::PrimeVice, 1814.0 + 400.0, 1204.0);
        n3.ai[2] = 99.0;
        style34_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.ai[2], 99.0, "400 is not < 400 (:28259)");
        let mut n4 = limb(NpcKind::PrimeVice, 1814.0 + 399.0, 1204.0);
        n4.ai[2] = 99.0;
        style34_step(&mut n4, &w, &c, &p);
        assert_eq!(n4.ai[2], 0.0);
    }

    /// `NPC.cs:28247` + `:28263-28272` - the sprite flip is written
    /// BEFORE the despawn early-out (the draw-before-early-return
    /// trap), and the despawn counts on the server only.
    #[test]
    fn the_vice_flips_before_the_despawn_and_counts_on_the_server() {
        let w = air();
        let p = [cand(1500.0, 1500.0)];
        let mut c = ctx();
        c.head_active = false;
        let mut n = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        n.ai[0] = -1.0; // the left seat: -(int)-1 = +1
        style34_step(&mut n, &w, &c, &p);
        assert_eq!(n.hp, 0, "single-player dies at once (:28266)");
        assert_eq!(n.sprite_dir, 1, "the flip ran BEFORE the return (:28247)");
        c.net_mode = 2;
        let mut n2 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        for want in [10.0, 20.0, 30.0, 40.0, 50.0] {
            style34_step(&mut n2, &w, &c, &p);
            assert_eq!(n2.ai[2], want);
            assert!(n2.hp > 0, "{want} is not > 50");
        }
        style34_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.hp, 0, "60 > 50 (:28266)");
    }

    /// `NPC.cs:28373-28427` - the hover: the 600-tick advance, the
    /// 300/230 TOP-edge Y band (a 70-px dead band), the asymmetric
    /// centre X band (headCentre+250 / headCentre) with the 0.94 damp
    /// and the DIFFERENT per-side steps (-0.3/+9 vs +0.2/-8).
    #[test]
    fn the_vice_hover_bands_and_the_600_tick_advance() {
        let w = air();
        let c = ctx();
        let p = [cand(1500.0, 1500.0)];
        // Below the band (y 1301 > 1300): rise at 0.1, damp first.
        let mut n = limb(NpcKind::PrimeVice, 1814.0, 1301.0);
        n.vy = 2.0;
        style34_step(&mut n, &w, &c, &p);
        assert!((n.vy - (2.0 * 0.96 - 0.1)).abs() < 1e-5, "{}", n.vy);
        // Inside the dead band (1230 <= y <= 1300): no Y write.
        let mut n2 = limb(NpcKind::PrimeVice, 1814.0, 1260.0);
        n2.vy = 1.25;
        style34_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.vy, 1.25, "the 230..300 gap holds (:28380/:28392)");
        // The X band: centre 2291 > 2290 pushes left at 0.3 (0.94 damp).
        let mut n3 = limb(NpcKind::PrimeVice, 2265.0, 1260.0);
        n3.vx = 1.0;
        style34_step(&mut n3, &w, &c, &p);
        assert!((n3.vx - (1.0 * 0.94 - 0.3)).abs() < 1e-5, "{}", n3.vx);
        // Centre 2039 < 2040 pushes right at 0.2 (NOT 0.3).
        let mut n4 = limb(NpcKind::PrimeVice, 2013.0, 1260.0);
        style34_step(&mut n4, &w, &c, &p);
        assert!((n4.vx - 0.2).abs() < 1e-5, "{}", n4.vx);
        // Centre 2100 is INSIDE the band: no X write.
        let mut n5 = limb(NpcKind::PrimeVice, 2074.0, 1260.0);
        n5.vx = 0.75;
        style34_step(&mut n5, &w, &c, &p);
        assert_eq!(n5.vx, 0.75, "2040 <= 2100 <= 2290 holds");
        // The 600th tick advances 0 -> 1 and resets the counter.
        let mut n6 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        n6.ai[3] = 599.0;
        style34_step(&mut n6, &w, &c, &p);
        assert_eq!(n6.ai[2], 1.0, "ai[2]++ (:28376)");
        assert_eq!(n6.ai[3], 0.0);
        assert!(n6.net_update);
    }

    /// `NPC.cs:28344-28368` - the attack chase: the |vx|+|vy| < 2 SNAP
    /// onto the 12-speed aim (STRICT <), the >= 2 unconditional 0.97
    /// damp of BOTH components, the dead-target sink that never ticks
    /// the 600-reset, and the dock rotation overwriting the aim.
    #[test]
    fn the_vice_chase_snaps_when_still_and_damps_when_moving() {
        let w = air();
        let mut c = ctx();
        c.head_ai1 = 1.0;
        let p = [cand(2100.0, 1230.0)];
        // Still: snap onto the aim.
        let mut n = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        style34_step(&mut n, &w, &c, &p);
        let dx = 2110.0f32 - 1840.0;
        let dy = 1251.0f32 - 1230.0;
        let scale = 12.0 / cs_sqrt(dx, dy);
        assert_eq!(n.vx, dx * scale, "the snap (:28354)");
        assert_eq!(n.vy, dy * scale, "the snap (:28355)");
        assert!(n.net_update);
        assert_eq!(n.ai[3], 1.0, "the alive branch ticks the reset (:28362)");
        // The dock rotation overwrites the chase aim (:28433).
        let (ddx, ddy) = (2040.0 - 200.0 - 1840.0, 1230.0 - 1230.0);
        assert_eq!(n.rotation, cs_atan2(ddy, ddx) + 1.57);
        // Moving (|3| + |0| >= 2): both components damp at 0.97.
        let mut n2 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        n2.vx = 3.0;
        n2.vy = -1.0;
        style34_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.vx, 3.0 * 0.97, "unconditional damp (:28360)");
        assert_eq!(n2.vy, -1.0 * 0.97);
        // Exactly 2 (|2| + |0|): NOT < 2, so damp.
        let mut n3 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        n3.vx = 2.0;
        style34_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.vx, 2.0 * 0.97, "2 is not < 2 (:28351)");
        // Dead target: sink, and the 600-counter never ticks.
        let mut pd = cand(2100.0, 1230.0);
        pd.dead = true;
        let mut n4 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        style34_step(&mut n4, &w, &c, &[pd]);
        assert_eq!(n4.vy, 0.1, "the dead sink (:28336)");
        assert_eq!(n4.ai[3], 0.0, "the reset lives in the alive branch only");
    }

    /// `NPC.cs:28435-28464` - the rise: only downward Y damps (0.9),
    /// X blends with the head's velocity ((vx*5 + head.vx)/6), the
    /// +/-0.5 steps and the -9 cap, the WIDE 280 dock rotation, and
    /// the 20-speed dash strictly above head.Y - 280.
    #[test]
    fn the_vice_rise_blends_the_head_velocity_and_dashes_at_20() {
        let w = air();
        let mut c = ctx();
        c.head_vx = 6.0;
        let p = [cand(2100.0, 1230.0)];
        // y 721 is NOT < 720: rise only.
        let mut n = limb(NpcKind::PrimeVice, 1814.0, 721.0);
        n.ai[2] = 1.0;
        n.vx = 6.0;
        n.vy = 1.0;
        style34_step(&mut n, &w, &c, &p);
        assert_eq!(n.vx, (6.0 * 5.0 + 6.0) / 6.0 + 0.5, "the blend (:28446-28447)");
        assert!((n.vy - (1.0 * 0.9 - 0.5)).abs() < 1e-6, "{}", n.vy);
        assert_eq!(n.ai[2], 1.0, "721 is not < 720 (:28453)");
        // The rotation rides the 280 dock, not the 200 dock.
        let (cw, ch) = n.size();
        let ddx = 2040.0 - 280.0 - (1814.0 + cw * 0.5);
        let ddy = 1230.0 - (721.0 + ch * 0.5);
        assert_eq!(n.rotation, cs_atan2(ddy, ddx) + 1.57);
        // y 719 < 720: re-target and dash at 20.
        let mut n2 = limb(NpcKind::PrimeVice, 1814.0, 719.0);
        n2.ai[2] = 1.0;
        style34_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 2.0);
        let dx = 2110.0 - (1814.0 + cw * 0.5);
        let dy = 1251.0 - (719.0 + ch * 0.5);
        let s = 20.0 / cs_sqrt(dx, dy);
        assert_eq!(n2.vx, dx * s);
        assert_eq!(n2.vy, dy * s);
        assert!(n2.net_update);
    }

    /// `NPC.cs:28466-28481` + `:28495-28520` - the four-dash cycle,
    /// the +/-500 centre gate into the 17-speed side dash, and state
    /// 5's end gate riding the ELSE-IF CONDITION (playerCentre - 100).
    #[test]
    fn the_vice_dash_cycles_and_the_side_sweep() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        // State 2 below the player: back to the rise, ai[3]++.
        let mut n = limb(NpcKind::PrimeVice, 1814.0, 1600.0);
        n.ai[2] = 2.0;
        style34_step(&mut n, &w, &c, &p);
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.ai[3], 1.0);
        // The fourth pass ends the cycle: state 3, counter 0.
        let mut n2 = limb(NpcKind::PrimeVice, 1814.0, 1600.0);
        n2.ai[2] = 2.0;
        n2.ai[3] = 4.0;
        style34_step(&mut n2, &w, &c, &p);
        assert_eq!(n2.ai[2], 3.0);
        assert_eq!(n2.ai[3], 0.0);
        // State 4 inside +/-500: Y blends, X pushes right.
        let mut n3 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        n3.ai[2] = 4.0;
        n3.vy = 6.0;
        style34_step(&mut n3, &w, &c, &p);
        assert_eq!(n3.vy, (6.0 * 5.0 + 0.0) / 6.0, "the Y blend (:28489)");
        assert_eq!(n3.vx, 0.5);
        assert_eq!(n3.ai[2], 4.0, "centre 1840 is inside 1540..2540");
        // Centre 1539 < 1540: the 17-speed dash, state 5.
        let mut n4 = limb(NpcKind::PrimeVice, 1513.0, 1204.0);
        n4.ai[2] = 4.0;
        style34_step(&mut n4, &w, &c, &p);
        assert_eq!(n4.ai[2], 5.0);
        let (cw, ch) = n4.size();
        let dx = 2110.0 - (1513.0 + cw * 0.5);
        let dy = 1251.0 - (1204.0 + ch * 0.5);
        let s = 17.0 / cs_sqrt(dx, dy);
        assert_eq!(n4.vx, dx * s);
        assert_eq!(n4.vy, dy * s);
        // State 5: the gate is the CONDITION - centre 1840 < 2010
        // cycles back to 4; centre 2011 (>= 2010) does nothing.
        let mut n5 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        n5.ai[2] = 5.0;
        style34_step(&mut n5, &w, &c, &p);
        assert_eq!(n5.ai[2], 4.0);
        assert_eq!(n5.ai[3], 1.0);
        let mut n6 = limb(NpcKind::PrimeVice, 1985.0, 1204.0);
        n6.ai[2] = 5.0;
        style34_step(&mut n6, &w, &c, &p);
        assert_eq!(n6.ai[2], 5.0, "2011 is not < 2010 (:28508)");
        let mut n7 = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        n7.ai[2] = 5.0;
        n7.ai[3] = 4.0;
        style34_step(&mut n7, &w, &c, &p);
        assert_eq!(n7.ai[2], 0.0);
        assert_eq!(n7.ai[3], 0.0);
    }

    /// RNG by construction: the vice arm draws NOTHING - the stream
    /// is bit-identical after a full tick (the dispatcher witness).
    #[test]
    fn the_vice_draws_no_rng() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        let mut r = rng();
        let before = r.clone();
        let mut n = limb(NpcKind::PrimeVice, 1814.0, 1204.0);
        assert!(prime_limb_step(&mut n, &w, &c, &p, &mut r));
        assert_eq!(r, before, "the vice provably draws zero (:28245-28521)");
    }

    // ---- the cannon (35) ----

    /// `NPC.cs:28595-28602` - the hover's 1100-tick jump into state 1
    /// zeroes the shot cadence BEFORE the shot block ticks it; the
    /// head-gone despawn keeps the truncated flip (draw-before-return).
    #[test]
    fn the_cannon_hover_jumps_at_1100_and_flips_before_despawn() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        let mut n = limb(NpcKind::PrimeCannon, 1814.0, 1204.0);
        n.ai[3] = 1099.0;
        n.local_ai[0] = 90.0;
        style35_step(&mut n, &w, &c, &p, &mut rng());
        assert_eq!(n.ai[2], 1.0, "the 1100th tick (:28596)");
        assert_eq!(n.ai[3], 0.0);
        assert_eq!(n.local_ai[0], 1.0, "zeroed then ticked (:28598/:28659)");
        assert!(n.net_update);
        let mut n2 = limb(NpcKind::PrimeCannon, 1814.0, 1204.0);
        n2.ai[3] = 1098.0;
        style35_step(&mut n2, &w, &c, &p, &mut rng());
        assert_eq!(n2.ai[2], 0.0);
        assert_eq!(n2.ai[3], 1099.0);
        // The despawn: flip first, hp-0, no further writes.
        let mut cd = ctx();
        cd.head_ai_style = 33; // any non-32 style despawns (:28525)
        let mut n3 = limb(NpcKind::PrimeCannon, 1814.0, 1204.0);
        n3.ai[0] = -1.0;
        style35_step(&mut n3, &w, &cd, &p, &mut rng());
        assert_eq!(n3.hp, 0);
        assert_eq!(n3.sprite_dir, 1, "-(int)-1 ran first (:28524)");
        assert_eq!(n3.ai[3], 0.0, "the early return skips the state block");
    }

    /// `NPC.cs:28657-28675` - the state-0 bomb: STRICT > 140 on the
    /// cadence, the REVERSED dock vector (0f - num), the +/-0.4 jitter
    /// (two draws, X then Y), the spawn point FOUR velocity units ahead
    /// of the limb centre, type 102 / damage 0.
    #[test]
    fn the_cannon_idle_bomb_fires_back_along_the_dock() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        // Exactly on the dock: the dock vector is (0,0), the sqrt 0,
        // so the C# yields NaN velocities - kept verbatim (no epsilon
        // floor); move off-dock for the numeric witness instead.
        let mut n = limb(NpcKind::PrimeCannon, 1814.0, 1104.0); // centre (1840,1130)
        n.local_ai[0] = 140.0;
        let mut r = rng();
        let mut witness = r.clone();
        style35_step(&mut n, &w, &c, &p, &mut r);
        assert_eq!(n.wants_hostile_shot.len(), 1, "141 > 140 (:28660)");
        let (ty, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(ty, crate::projectile::BOMB_SKELETRON);
        assert_eq!(dmg, 0);
        // dock = (2040-200-1840, 1230-1130) = (0, 100); the sqrt is
        // exactly 100, scale 12/100 = 0.12; reversed = (-0*0.12,
        // -100*0.12) = (-0.0, -12) plus the two jitter draws.
        let jx = witness.next_range(-40, 41) as f32 * 0.01;
        let jy = witness.next_range(-40, 41) as f32 * 0.01;
        assert_eq!(vx, (0.0 - 0.0) * (12.0 / 100.0) + jx);
        assert_eq!(vy, (0.0 - 100.0) * (12.0 / 100.0) + jy);
        assert_eq!(sx, 1840.0 + vx * 4.0, "four units ahead (:28671)");
        assert_eq!(sy, 1130.0 + vy * 4.0);
        assert_eq!(n.local_ai[0], 0.0);
        assert_eq!(r, witness, "exactly two draws, X then Y");
        // Exactly 140 after the ++? 139 -> 140 is NOT > 140: no shot.
        let mut n2 = limb(NpcKind::PrimeCannon, 1814.0, 1104.0);
        n2.local_ai[0] = 139.0;
        let mut r2 = rng();
        let before = r2.clone();
        style35_step(&mut n2, &w, &c, &p, &mut r2);
        assert!(n2.wants_hostile_shot.is_empty(), "140 is not > 140");
        assert_eq!(n2.local_ai[0], 140.0);
        assert_eq!(r2, before, "a non-firing tick draws nothing");
    }

    /// `NPC.cs:28657` - the client gate: netMode == 1 never ticks the
    /// cadence and never draws.
    #[test]
    fn the_cannon_client_never_fires_or_draws() {
        let w = air();
        let mut c = ctx();
        c.net_mode = 1;
        let p = [cand(2100.0, 1230.0)];
        let mut n = limb(NpcKind::PrimeCannon, 1814.0, 1104.0);
        n.local_ai[0] = 500.0;
        let mut r = rng();
        let before = r.clone();
        style35_step(&mut n, &w, &c, &p, &mut r);
        assert!(n.wants_hostile_shot.is_empty());
        assert_eq!(n.local_ai[0], 500.0, "the ++ itself is gated (:28657-28659)");
        assert_eq!(r, before);
    }

    /// `NPC.cs:28683-28753` - the bombard state: the 300-tick return,
    /// the head-X/player-Y-80 aim (the dead head-Y store kept), the
    /// four 0.9-damp comparisons with the 0.04/0.08 steps, and the
    /// STRICT > 40 player bomb at speed 10.
    #[test]
    fn the_cannon_bombard_aims_and_fires_at_40() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        // The steer: X towards the head centre, Y towards player-80.
        let mut n = limb(NpcKind::PrimeCannon, 1814.0, 1104.0);
        n.ai[2] = 1.0;
        n.local_ai[0] = 40.0;
        let mut r = rng();
        let mut witness = r.clone();
        style35_step(&mut n, &w, &c, &p, &mut r);
        assert_eq!(n.wants_hostile_shot.len(), 1, "41 > 40 (:28739)");
        let (ty, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(ty, crate::projectile::BOMB_SKELETRON);
        assert_eq!(dmg, 0);
        // After the steer the aim re-reads: player centre (2110,1251)
        // minus limb centre (1840,1130) = (270,121), normalised to 10.
        let s = 10.0 / cs_sqrt(270.0, 121.0);
        let jx = witness.next_range(-40, 41) as f32 * 0.01;
        let jy = witness.next_range(-40, 41) as f32 * 0.01;
        assert_eq!(vx, 270.0 * s + jx);
        assert_eq!(vy, 121.0 * s + jy);
        assert_eq!(sx, 1840.0 + vx * 4.0);
        assert_eq!(sy, 1130.0 + vy * 4.0);
        assert_eq!(r, witness, "two draws per bomb");
        // The steer itself: the aim is (headCentre 2040 - 1840,
        // playerCentreY 1251 - 80 - 1130) = (+200, +41) scaled to 6 -
        // both positive, so vx 0 steps +0.04 and vy 0 steps +0.08
        // (no damp: the velocity is not > 0 on entry).
        assert_eq!(n.vx, 0.04, "the +0.04 step (:28712)");
        assert_eq!(n.vy, 0.08, "the +0.08 step (:28728)");
        // The 300th tick returns to 0.
        let mut n2 = limb(NpcKind::PrimeCannon, 1814.0, 1104.0);
        n2.ai[2] = 1.0;
        n2.ai[3] = 299.0;
        style35_step(&mut n2, &w, &c, &p, &mut rng());
        assert_eq!(n2.ai[2], 0.0);
        assert_eq!(n2.ai[3], 0.0);
        // The return zeroes the cadence (:28686), but the shot block
        // still runs THIS tick and ticks it once (:28738) -> 1.
        assert_eq!(n2.local_ai[0], 1.0, "zeroed then ++ (:28686/:28738)");
        // Any other state returns untouched (:28679-28682).
        let mut n3 = limb(NpcKind::PrimeCannon, 1814.0, 1104.0);
        n3.ai[2] = 99.0;
        n3.ai[3] = 7.0;
        style35_step(&mut n3, &w, &c, &p, &mut rng());
        assert_eq!(n3.ai[2], 99.0, "no 99 handling in this arm");
        assert_eq!(n3.ai[3], 7.0, "the return skips the counter");
    }

    // ---- the laser (36) ----

    /// `NPC.cs:28830-28836` - the hover's 800-tick ai[2]++ (NOT a
    /// jump-to-1: from state 3 it would reach 4, which then returns
    /// forever - kept verbatim), and the head-attack pre-load +3.
    #[test]
    fn the_laser_hover_advances_at_800_and_preloads_three() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        let mut n = limb(NpcKind::PrimeLaser, 1814.0, 1204.0);
        n.ai[3] = 799.0;
        style36_step(&mut n, &w, &c, &p, &mut rng());
        assert_eq!(n.ai[2], 1.0, "ai[2]++ (:28833)");
        assert_eq!(n.ai[3], 0.0);
        assert!(n.net_update);
        // From 3: ++ reaches 4 (no clamp - the C# verbatim).
        let mut n2 = limb(NpcKind::PrimeLaser, 1814.0, 1204.0);
        n2.ai[2] = 3.0;
        n2.ai[3] = 799.0;
        style36_step(&mut n2, &w, &c, &p, &mut rng());
        assert_eq!(n2.ai[2], 4.0);
        // The attack gate pre-loads +3 per tick (:28778).
        let mut ca = ctx();
        ca.head_ai1 = 1.0;
        let mut n3 = limb(NpcKind::PrimeLaser, 1814.0, 1204.0);
        style36_step(&mut n3, &w, &ca, &p, &mut rng());
        assert_eq!(n3.local_ai[0], 4.0, "+3 then the ++ (:28778/:28894)");
    }

    /// `NPC.cs:28886-28908` - the state-0 volley: the unconditional
    /// re-target + player aim, STRICT > 200, speed 8, damage 25, type
    /// 100, the x0.05 jitter (two draws), the spawn EIGHT units ahead.
    #[test]
    fn the_laser_hover_volley_fires_at_200() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        let mut n = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        n.local_ai[0] = 200.0;
        let mut r = rng();
        let mut witness = r.clone();
        style36_step(&mut n, &w, &c, &p, &mut r);
        assert_eq!(n.wants_hostile_shot.len(), 1, "201 > 200 (:28895)");
        let (ty, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(ty, crate::projectile::DESTROYER_LASER, "id 100");
        assert_eq!(dmg, 25);
        let s = 8.0 / cs_sqrt(270.0, 121.0);
        let jx = witness.next_range(-40, 41) as f32 * 0.05;
        let jy = witness.next_range(-40, 41) as f32 * 0.05;
        assert_eq!(vx, 270.0 * s + jx);
        assert_eq!(vy, 121.0 * s + jy);
        assert_eq!(sx, 1840.0 + vx * 8.0, "eight units ahead (:28906)");
        assert_eq!(sy, 1130.0 + vy * 8.0);
        assert_eq!(r, witness, "exactly two draws, X then Y");
        // The rotation is the player aim (- 1.57), computed BEFORE the
        // jitter (the aim reads the un-jittered vector).
        assert_eq!(n.rotation, cs_atan2(121.0, 270.0) - 1.57);
        // 199 -> 200 is NOT > 200: no shot, no draws.
        let mut n2 = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        n2.local_ai[0] = 199.0;
        let mut r2 = rng();
        let before = r2.clone();
        style36_step(&mut n2, &w, &c, &p, &mut r2);
        assert!(n2.wants_hostile_shot.is_empty());
        assert_eq!(r2, before);
    }

    /// `NPC.cs:28918-28986` - the flank state: the 200-tick return, the
    /// playerCentre-350/-20 aim with the 0.1/0.03 steps, the STRICT
    /// > 80 volley at speed 10, and the non-1 early return.
    #[test]
    fn the_laser_flank_steers_and_volleys_at_80() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        let mut n = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        n.ai[2] = 1.0;
        n.local_ai[0] = 80.0;
        let mut r = rng();
        let mut witness = r.clone();
        style36_step(&mut n, &w, &c, &p, &mut r);
        // The flank point: (2110 - 350, 1251 - 20) = (1760, 1231);
        // from centre (1840, 1130): (-80, +101). vx 0 > aimX: damp
        // guard fails (0 not > 0), step -0.1. vy 0 < aimY: step +0.03.
        assert_eq!(n.vx, -0.1, "the -0.1 step (:28938)");
        assert_eq!(n.vy, 0.03, "the +0.03 step (:28962)");
        assert_eq!(n.wants_hostile_shot.len(), 1, "81 > 80 (:28973)");
        let (ty, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(ty, crate::projectile::DESTROYER_LASER);
        assert_eq!(dmg, 25);
        let s = 10.0 / cs_sqrt(270.0, 121.0);
        let jx = witness.next_range(-40, 41) as f32 * 0.05;
        let jy = witness.next_range(-40, 41) as f32 * 0.05;
        assert_eq!(vx, 270.0 * s + jx);
        assert_eq!(vy, 121.0 * s + jy);
        assert_eq!(sx, 1840.0 + vx * 8.0);
        assert_eq!(sy, 1130.0 + vy * 8.0);
        assert_eq!(r, witness);
        // The 200th tick returns to 0 and zeroes the cadence.
        let mut n2 = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        n2.ai[2] = 1.0;
        n2.ai[3] = 199.0;
        style36_step(&mut n2, &w, &c, &p, &mut rng());
        assert_eq!(n2.ai[2], 0.0);
        // Zeroed at :28921, then the volley block ticks it (:28972).
        assert_eq!(n2.local_ai[0], 1.0, "(:28921/:28972)");
        // State 2 (or anything not 0/3/1) returns untouched.
        let mut n3 = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        n3.ai[2] = 2.0;
        n3.ai[3] = 5.0;
        style36_step(&mut n3, &w, &c, &p, &mut rng());
        assert_eq!(n3.ai[3], 5.0, "the early return (:28914-28917)");
    }

    /// `NPC.cs:28892` - the client gate: netMode == 1 never ticks the
    /// cadence and never draws (both volley sites).
    #[test]
    fn the_laser_client_never_fires_or_draws() {
        let w = air();
        let mut c = ctx();
        c.net_mode = 1;
        let p = [cand(2100.0, 1230.0)];
        let mut r = rng();
        let before = r.clone();
        let mut n = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        n.local_ai[0] = 500.0;
        style36_step(&mut n, &w, &c, &p, &mut r);
        assert!(n.wants_hostile_shot.is_empty());
        assert_eq!(n.local_ai[0], 500.0);
        let mut n2 = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        n2.ai[2] = 1.0;
        n2.local_ai[0] = 500.0;
        style36_step(&mut n2, &w, &c, &p, &mut r);
        assert!(n2.wants_hostile_shot.is_empty());
        assert_eq!(n2.local_ai[0], 500.0);
        assert_eq!(r, before, "both volley sites gated (:28892/:28970)");
    }

    /// The dispatcher: kind-keyed routing, the saw excluded, and the
    /// stream-clone witness - identical inputs + identical streams
    /// give identical state (RNG by construction, determinism pin).
    #[test]
    fn the_dispatcher_routes_by_kind_and_is_deterministic() {
        let w = air();
        let c = ctx();
        let p = [cand(2100.0, 1230.0)];
        // The saw is NOT this module's family.
        let mut saw = limb(NpcKind::PrimeSaw, 1814.0, 1204.0);
        let mut r0 = rng();
        assert!(!prime_limb_step(&mut saw, &w, &c, &p, &mut r0));
        // Two identical cannon runs: identical state AND stream.
        let mut a = limb(NpcKind::PrimeCannon, 1814.0, 1104.0);
        a.local_ai[0] = 140.0;
        let mut b = limb(NpcKind::PrimeCannon, 1814.0, 1104.0);
        b.local_ai[0] = 140.0;
        let mut ra = rng();
        let mut rb = rng();
        assert!(prime_limb_step(&mut a, &w, &c, &p, &mut ra));
        assert!(prime_limb_step(&mut b, &w, &c, &p, &mut rb));
        assert_eq!(a.wants_hostile_shot, b.wants_hostile_shot);
        assert_eq!(a.vx, b.vx);
        assert_eq!(a.vy, b.vy);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(ra, rb);
        // Two identical laser runs likewise.
        let mut la = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        la.local_ai[0] = 200.0;
        let mut lb = limb(NpcKind::PrimeLaser, 1814.0, 1104.0);
        lb.local_ai[0] = 200.0;
        let mut rla = rng();
        let mut rlb = rng();
        assert!(prime_limb_step(&mut la, &w, &c, &p, &mut rla));
        assert!(prime_limb_step(&mut lb, &w, &c, &p, &mut rlb));
        assert_eq!(la.wants_hostile_shot, lb.wants_hostile_shot);
        assert_eq!(rla, rlb);
    }
}
