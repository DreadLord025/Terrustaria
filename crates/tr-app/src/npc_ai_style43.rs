//! S1-D-3 (aiStyle-43) — the Queen Bee's machine: `NPC.AI()`'s
//! `else if (this.aiStyle == 43)` arm (`NPC.cs:30190-30860`), the ONLY
//! user of the style in the C# (`SetDefaults` case 222, `NPC.cs:11859`;
//! `NPCID.QueenBee = 222`, `NPCID.cs:11121`). The boss cycles the hover
//! state 0 (the even `ai[1]` half aims/charges at 12+7·enrage, the odd
//! half turns), the state -1 random re-pick (`Next(3)` mapped 1→2, 2→3,
//! never repeating `ai[1]`), the state 2 climb to 200 px above the
//! player, the state 1 bee-spawn volley (`Next(210, 212)` = Bee 210 /
//! BeeSmall 211), the state 3 stinger barrage (projectile 719, damage
//! 11, `timeLeft = 300`), the state 4 long-range return (the 14/14
//! weighted steer) and the state 5 dead-player flee (the world-edge
//! drift + `EncourageDespawn(10)`).
//!
//! **UNWIRED module (code-only mode, 2026-11-18).** The port's
//! `NpcKind::QueenBee` already carries `ai_style() == 43`
//! (`npc.rs`'s style table) but runs the `update_queen_bee` v1 stand-in
//! (the EoC cycle shape with the bee's numbers — no enrage term, no
//! state machine, no bees/stingers). The kind tables and the hot
//! `npc.rs` dispatch are contested by the in-flight S1-D-3 sibling
//! branches, so this slice lands as pure functions; the `43 =>` axis
//! arm + the v1 removal + the spawn drains are the family's merge step
//! (the `npc_ai_style42.rs` UNWIRED pattern). Intended
//! call site: `update_rng`'s `match self.kind.ai_style()` axis
//! (`npc.rs`), a `43 => crate::npc_ai_style43::queen_bee_ai(...)` arm.
//!
//! The arm writes velocity only — the C#'s `AI()` never integrates the
//! position; `UpdateNPC` does (`NPC.cs:90274`, the `noTileCollide`
//! path adds velocity directly). The wiring step keeps the sim's
//! integration; this module must NOT add `x += vx` (the v1 did — that
//! was part of the stand-in, not the C# arm).
//!
//! RNG argument — every `Main.rand` draw in the arm is cited and
//! threaded through the caller's `UnifiedRandom` in C# order:
//! * state -1: the `do { Next(3) } while (== ai[1])` re-pick loop
//!   (`NPC.cs:30279-30292`) — 1+ draws per re-pick tick.
//! * state 1: `Next(20)` for the spawn-point X offset EVERY tick
//!   (`:30582`), then `Next(210, 212)` for the bee type only on a
//!   spawn tick (`:30628`).
//! * state 3: `Next(20)` EVERY tick (`:30701`), then the two jitter
//!   draws `Next(-num653, num653+1)` / `Next(-num654, num654+1)` only
//!   on a fire tick, X before Y (`:30739-30740`).
//! * states 0/2/4/5 and the head draw NOTHING — the witness test pins
//!   an untouched stream across those states.
//! `TargetClosest` (`NPC.cs:77652-77781`) is a deterministic scan.
//!
//! Deliberate stand-ins (recorded in docs/ANALYSIS.md "The Queen Bee's
//! aiStyle-43 machine"):
//! * `Main.expertMode` — the port is single-difficulty (the
//!   `eoc_fight.rs` `expert: bool` convention), so the wiring passes
//!   `expert = false`; the parameter keeps the full expert ladder
//!   portable and testable.
//! * `Main.getGoodWorld` — read live from `world.seeds`
//!   (`SeedFlags::FOR_THE_WORTHY`, the `variants.rs` convention); the
//!   `num619 += 0.5f` term is armed.
//! * `Main.netMode == 1` early return (`:30273-30276`) folds to the
//!   host path; the `netMode != 1` spawn gates (`:30626`, `:30717`)
//!   always fire on the host.
//! * The sounds (`SoundEngine.PlaySound(29, ..., 125)` `:30362`,
//!   `(3, ...)` `:30625`, `SoundID.Item17` `:30716`) are skipped — no
//!   tracked sound layer for this kind yet.
//! * The bee's `CanBeReplacedByOtherNPCs = true` (`:30632`) has no port
//!   field — dropped; `localAI[0] = 60` (`:30633`) rides the spawn
//!   payload for the wiring step.
//! * The expert state-1 player count (`:30590-30598`) loops the 255
//!   slots; the port's single session player makes `num641 / 2` = 0
//!   (integer division) — kept verbatim over the one candidate.
//! * The `!Main.player[target].active` re-pick term (`:30197`) folds
//!   open — the port's single session player is always active.
//! * `flag52` (`:30707`) and the dead stores `num637 = num633/num637`
//!   (`:30544`), `num640 = num644/...` (`:30644`), `num650 =
//!   num646/num650` (`:30795`) are computed-then-unread in the C# —
//!   dropped with this note (no side effects).
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the S1-D-3 merge step.

use crate::npc::Npc;
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// `SetDefaults` case 222 (`NPC.cs:11855-11871`): 66x66, aiStyle 43,
/// dmg 30, def 8, 3400 HP, kb 0, value 5 gold (50000 copper), boss,
/// noGravity + noTileCollide, npcSlots 7; HitSound NPCHit1 /
/// DeathSound NPCDeath66 (`:11863-11864`). The wiring-step row for the
/// kind tables (the port's `NpcKind::QueenBee` stats already match).
#[derive(Clone, Copy, Debug)]
pub struct KindDefaults {
    pub width: f32,
    pub height: f32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    pub value: f32,
    pub boss: bool,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub npc_slots: f32,
}

/// The cited `SetDefaults` row (see [`KindDefaults`]).
pub const QUEEN_BEE: KindDefaults = KindDefaults {
    width: 66.0,
    height: 66.0,
    ai_style: 43,
    damage: 30,
    defense: 8,
    life_max: 3400,
    knock_back_resist: 0.0,
    value: 50000.0,
    boss: true,
    no_gravity: true,
    no_tile_collide: true,
    npc_slots: 7.0,
};

/// `NPCID.Bee` (210) / `NPCID.BeeSmall` (211) (`NPCID.cs:11097-11099`)
/// — the state-1 volley's `Main.rand.Next(210, 212)` picks
/// (`NPC.cs:30628`). Neither has an `NpcKind` yet; the wiring step
/// adds the rows + the spawn drain.
pub const BEE: i32 = 210;
pub const BEE_SMALL: i32 = 211;

/// `ProjectileID.QueenBeeStinger` (719, `ProjectileID.cs:1992`) — the
/// state-3 barrage's shot (`NPC.cs:30742`), damage 11, knockback 0,
/// `timeLeft = 300` (`:30743`). No `proj_data` row yet; the payload
/// carries the lifetime for the wiring step's spawn drain.
pub const QUEEN_BEE_STINGER: u16 = 719;

/// The state-1 bee spawn request (`NPC.cs:30628-30634`): the type draw,
/// the `(int)vector77` truncation, the normalized ×5 velocity toward
/// the player and the `localAI[0] = 60` payload. The wiring step
/// spawns the NPC and sets `CanBeReplacedByOtherNPCs`/`netUpdate`.
#[derive(Clone, Copy, Debug)]
pub struct BeeSpawn {
    /// `Main.rand.Next(210, 212)` — [`BEE`] or [`BEE_SMALL`].
    pub kind: i32,
    /// `(int)vector77.X` (`:30628`) — truncated toward zero.
    pub x: f32,
    /// `(int)vector77.Y`.
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `Main.npc[num643].localAI[0] = 60f` (`:30633`).
    pub local_ai0: f32,
}

/// The state-3 stinger request (`NPC.cs:30739-30743`): the shot from
/// `vector79` with the jittered, normalized ×`num652` velocity, damage
/// 11, and the post-spawn `timeLeft = 300`.
#[derive(Clone, Copy, Debug)]
pub struct StingerShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    /// `Main.projectile[num658].timeLeft = 300` (`:30743`).
    pub time_left: i32,
}

/// One tick's spawn requests — at most one bee and one stinger per
/// tick (the C# fires each at most once per AI call).
#[derive(Clone, Copy, Debug, Default)]
pub struct QueenBeeOut {
    pub bee: Option<BeeSpawn>,
    pub stinger: Option<StingerShot>,
}

/// `NPC.TargetClosest()` through the shared selection
/// (`npc_target.rs`, `NPC.cs:77652-77781`) — the single session player
/// as slot 0 with the stock aggro/no-aggro/tank values (the zombie's
/// inline candidate, `npc.rs`'s `update_zombie`, S1-D-3p convention).
/// RNG-free (`:77652-77781` has no `Next`).
fn target_closest(npc: &mut Npc, world: &World, player: &Player) {
    let cand = crate::npc_target::TargetCandidate {
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
    // The `EverLastingTicker` stamp's only reader is the unported
    // target-line fade — 0 until the caller migration (S1-D-3p).
    crate::npc_target::target_closest(npc, world, &[cand], true, 43, 0);
}

/// The enrage term `num619` (`NPC.cs:30202-30214`): +1 above the world
/// surface (`(double)(position.Y / 16f) < Main.worldSurface` — a f32
/// divide promoted to f64 for the STRICT comparison), +1 outside the
/// player's jungle zone, +0.5 on the get-good seed.
pub fn enrage_term(npc_y: f32, world_surface: f64, zone_jungle: bool, get_good_world: bool) -> f32 {
    let mut num619 = 0.0f32;
    // `:30203-30206`.
    if ((npc_y / 16.0) as f64) < world_surface {
        num619 += 1.0;
    }
    // `:30207-30210`.
    if !zone_jungle {
        num619 += 1.0;
    }
    // `:30211-30214`.
    if get_good_world {
        num619 += 0.5;
    }
    num619
}

/// `Vector2.Normalize()` (XNA `Vector2.cs`): divide by `Length()`;
/// a zero vector stays zero (0/0 → the C# leaves NaN — the port's
/// glam-free fold keeps zero, matching every reachable call site:
/// the flee/return/bee vectors are non-degenerate in play; the guard
/// is the `projectile.rs:3927` NaN-safe convention).
fn normalize(x: f32, y: f32) -> (f32, f32) {
    let len = (x * x + y * y).sqrt();
    if len == 0.0 {
        (0.0, 0.0)
    } else {
        (x / len, y / len)
    }
}

/// One fixed step of the aiStyle-43 arm (`NPC.cs:30190-30860`).
/// `zone_jungle` is `Main.player[target].ZoneJungle` (the wiring step
/// feeds the biome scan, the `ambience.rs` field); `expert` is
/// `Main.expertMode` (false until the difficulty system lands — see
/// the module doc); `rng` is the caller's `Main.rand` stream (the
/// draw sites are enumerated in the module doc). Returns the tick's
/// bee/stinger spawn requests; writes velocity, ai/localAI, facing,
/// defense and the despawn window in place. Does NOT integrate the
/// position (the C# `AI()` doesn't — `UpdateNPC` does).
pub fn queen_bee_ai(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    zone_jungle: bool,
    expert: bool,
    rng: &mut UnifiedRandom,
) -> QueenBeeOut {
    let mut out = QueenBeeOut::default();
    let (w, h) = npc.size();
    // `:30192-30196` — the expert defense ramp over `defDefense` (8):
    // `(int)(20f * (1f - life / lifeMax))`, f32 divide, truncation.
    if expert {
        let num618 = (20.0 * (1.0 - npc.hp as f32 / npc.max_hp as f32)) as i32;
        npc.defense_override = Some(QUEEN_BEE.defense + num618);
    }
    // `:30197-30200` — re-pick when the target is unset/dead/inactive;
    // the port's single session player is slot 0 and always active (the
    // `!active` term folds open).
    if npc.target < 0 || npc.target == 255 || player.dead {
        target_closest(npc, world, player);
    }
    // `:30201` — dead4 latches AFTER the re-pick.
    let dead4 = player.dead;
    // `:30202-30214` — the enrage term.
    let num619 = enrage_term(
        npc.y,
        world.world_surface,
        zone_jungle,
        world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
    );
    // `:30215` — `Vector2.Distance(base.Center, player.Center)`: the
    // float-half centres (`Entity.cs:50-55`), sqrt of the squared sum.
    let (cx, cy) = npc.center();
    let (pcx, pcy) = player.center();
    let num620 = ((cx - pcx) * (cx - pcx) + (cy - pcy) * (cy - pcy)).sqrt();
    // `:30216-30227` — outside the flee state: the timeLeft floor and
    // the 3000-px leash. The C# `timeLeft < 60 → 60` is verbatim (the
    // port's -1 "never despawn" default becomes 60 here exactly as the
    // C#'s default 600 would clamp — the wiring step's boss-despawn
    // gate owns the survival semantics).
    if npc.ai[0] != 5.0 {
        if npc.despawn_timer < 60 {
            npc.despawn_timer = 60;
        }
        if num620 > 3000.0 {
            npc.ai[0] = 4.0;
            npc.net_update = true;
        }
    }
    // `:30228-30232` — a dead target overrides into the flee.
    if dead4 {
        npc.ai[0] = 5.0;
        npc.net_update = true;
    }
    // The C# half-extent forms: `width * 0.5f` (float) for the
    // vector75..81 centres, `(width / 2)` (INTEGER) for the aim
    // deltas — both kept verbatim (66/20 are even, so they agree
    // here, but the forms must not drift).
    let half_w = (w as i32 / 2) as f32;
    let half_h = (h as i32 / 2) as f32;
    let p_half_w = (crate::player::WIDTH as i32 / 2) as f32;
    let p_half_h = (player.height as i32 / 2) as f32;
    if npc.ai[0] == 5.0 {
        // `:30233-30270` — the flee: Y damps ×0.98, the X drifts toward
        // the nearer world edge (the half-world split is
        // `maxTilesX * 8` — INTEGER px, half of `maxTilesX * 16`).
        npc.vy *= 0.98;
        npc.facing = if npc.vx < 0.0 { -1 } else { 1 };
        npc.sprite_dir = npc.facing;
        if npc.x < (world.width * 8) as f32 {
            // `:30245-30256`.
            if npc.vx > 0.0 {
                npc.vx *= 0.98;
            } else {
                npc.local_ai[0] = 1.0;
            }
            npc.vx -= 0.08;
        } else {
            // `:30257-30268`.
            if npc.vx < 0.0 {
                npc.vx *= 0.98;
            } else {
                npc.local_ai[0] = 1.0;
            }
            npc.vx += 0.08;
        }
        // `:30269` — EncourageDespawn(10) (only ever lowers).
        npc.encourage_despawn(10);
    } else if npc.ai[0] == -1.0 {
        // `:30271-30297` — the re-pick: `Next(3)` mapped 1→2, 2→3
        // (so the picks are {0, 2, 3}), redrawn while equal to the
        // previous state in `ai[1]`. The `netMode == 1` early return
        // (`:30273-30276`) folds to the host path.
        let num621 = npc.ai[1];
        let mut num622;
        loop {
            num622 = rng.next_max(3);
            match num622 {
                1 => num622 = 2,
                2 => num622 = 3,
                _ => {}
            }
            if num622 as f32 != num621 {
                break;
            }
        }
        npc.ai[0] = num622 as f32;
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
        npc.net_update = true;
    } else if npc.ai[0] == 0.0 {
        // `:30298-30522` — the hover/charge cycle.
        // `:30300-30315` — the charge count: 2, +1 per expert HP
        // third (INTEGER lifeMax/2, /3, /5 divides).
        let mut num623 = 2;
        if expert {
            if npc.hp < npc.max_hp / 2 {
                num623 += 1;
            }
            if npc.hp < npc.max_hp / 3 {
                num623 += 1;
            }
            if npc.hp < npc.max_hp / 5 {
                num623 += 1;
            }
        }
        // `:30316-30323` — the cycle end: ai[1] past
        // `2 * (num623 + (int)(1f * num619))` on an EVEN tick → re-pick.
        if npc.ai[1] > (2 * (num623 + (1.0 * num619) as i32)) as f32 && npc.ai[1] % 2.0 == 0.0 {
            npc.ai[0] = -1.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.net_update = true;
            return out;
        }
        if npc.ai[1] % 2.0 == 0.0 {
            // `:30324-30431` — the even half: re-aim, then either the
            // level charge or the vertical hover.
            target_closest(npc, world, player);
            if (cy - pcy).abs() < 20.0 + 20.0 * num619 {
                // `:30327-30364` — the level charge: localAI[0] = 1
                // (the charging sprite half), ai[1]++, the normalized
                // ×num624 velocity. Sound 29/125 skipped.
                npc.local_ai[0] = 1.0;
                npc.ai[1] += 1.0;
                npc.ai[2] = 0.0;
                npc.net_update = true;
                let mut num624 = 12.0f32;
                if expert {
                    num624 = 16.0;
                    // `:30337-30352` — the four f64 HP fractions.
                    if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
                        num624 += 2.0;
                    }
                    if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
                        num624 += 2.0;
                    }
                    if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
                        num624 += 2.0;
                    }
                    if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
                        num624 += 2.0;
                    }
                }
                num624 += 7.0 * num619;
                // `:30355-30360` — vector75 float halves, the aim
                // deltas INTEGER halves, sqrt-normalized.
                let num625 = player.x + p_half_w - (npc.x + w * 0.5);
                let num626 = player.y + p_half_h - (npc.y + h * 0.5);
                let num627 = num624 / (num625 * num625 + num626 * num626).sqrt();
                npc.vx = num625 * num627;
                npc.vy = num626 * num627;
                npc.sprite_dir = npc.facing;
                return out;
            }
            // `:30365-30430` — the hover: localAI[0] = 0, the vertical
            // approach (num629 accel, ±num628 cap) and the X band
            // (>600 approach, <300 retreat, else ×0.8 damp; ±16 cap).
            npc.local_ai[0] = 0.0;
            let mut num628 = 12.0f32;
            let mut num629 = 0.15f32;
            if expert {
                // `:30370-30389` — the f64 HP ladder (note the +2/+2
                // jumps and the 0.1 accel step at the last tier).
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
                    num628 += 1.0;
                    num629 += 0.05;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
                    num628 += 1.0;
                    num629 += 0.05;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
                    num628 += 2.0;
                    num629 += 0.05;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
                    num628 += 2.0;
                    num629 += 0.1;
                }
            }
            num628 += 3.0 * num619;
            num629 += 0.5 * num619;
            // `:30393-30400` — INTEGER half-height comparison.
            if npc.y + half_h < player.y + p_half_h {
                npc.vy += num629;
            } else {
                npc.vy -= num629;
            }
            if npc.vy < 0.0 - num628 {
                npc.vy = 0.0 - num628;
            }
            if npc.vy > num628 {
                npc.vy = num628;
            }
            // `:30409-30420` — the X band (INTEGER halves both sides).
            let dx = (npc.x + half_w - (player.x + p_half_w)).abs();
            if dx > 600.0 {
                npc.vx += 0.15 * npc.facing as f32;
            } else if dx < 300.0 {
                npc.vx -= 0.15 * npc.facing as f32;
            } else {
                npc.vx *= 0.8;
            }
            if npc.vx < -16.0 {
                npc.vx = -16.0;
            }
            if npc.vx > 16.0 {
                npc.vx = 16.0;
            }
            npc.sprite_dir = npc.facing;
            return out;
        }
        // `:30432-30521` — the odd half: the turn-around. The facing
        // follows the velocity sign; num630 is the turn distance
        // (600, the expert HP ladder 300/450/500/550, minus
        // `(int)(100f * num619)`).
        npc.facing = if npc.vx < 0.0 { -1 } else { 1 };
        npc.sprite_dir = npc.facing;
        let mut num630 = 600;
        if expert {
            // `:30444-30459` — the f64 ladder, ascending thresholds.
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
                num630 = 300;
            } else if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
                num630 = 450;
            } else if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
                num630 = 500;
            } else if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
                num630 = 550;
            }
        }
        // `:30461-30465` — num631: -1 when the player is to the RIGHT
        // (the "facing away" test), INTEGER halves.
        let mut num631 = 1;
        if npc.x + half_w < player.x + p_half_w {
            num631 = -1;
        }
        num630 -= (100.0 * num619) as i32;
        let mut flag50 = false;
        // `:30467-30477` — facing away past num630, or vertically past
        // num630 * 1.5f (a FLOAT multiply of the int distance).
        if npc.facing == num631 && (cx - pcx).abs() > num630 as f32 {
            npc.ai[2] = 1.0;
            flag50 = true;
        }
        if (cy - pcy).abs() > num630 as f32 * 1.5 {
            npc.ai[2] = 1.0;
            flag50 = true;
        }
        // `:30478-30481` — the enrage brake fires only on the turn
        // tick (`&` — non-short-circuit in the C#, same result).
        if (num619 > 0.0) & flag50 {
            npc.vx *= 0.5;
            npc.vy *= 0.5;
        }
        if npc.ai[2] == 1.0 {
            // `:30482-30517` — the decel: re-aim, ×0.9 (expert extra
            // ×0.9 per INTEGER HP third), enrage ×0.7, and the
            // |vx|+|vy| < num632 hand-off to the next even tick.
            target_closest(npc, world, player);
            npc.sprite_dir = npc.facing;
            npc.local_ai[0] = 0.0;
            npc.vx *= 0.9;
            npc.vy *= 0.9;
            let mut num632 = 0.1f32;
            if expert {
                if npc.hp < npc.max_hp / 2 {
                    npc.vx *= 0.9;
                    npc.vy *= 0.9;
                    num632 += 0.05;
                }
                if npc.hp < npc.max_hp / 3 {
                    npc.vx *= 0.9;
                    npc.vy *= 0.9;
                    num632 += 0.05;
                }
                if npc.hp < npc.max_hp / 5 {
                    npc.vx *= 0.9;
                    npc.vy *= 0.9;
                    num632 += 0.05;
                }
            }
            if num619 > 0.0 {
                npc.vx *= 0.7;
                npc.vy *= 0.7;
            }
            if npc.vx.abs() + npc.vy.abs() < num632 {
                npc.ai[2] = 0.0;
                npc.ai[1] += 1.0;
                npc.net_update = true;
            }
        } else {
            // `:30518-30521` — still flying straight: the charge sprite.
            npc.local_ai[0] = 1.0;
        }
    } else if npc.ai[0] == 2.0 {
        // `:30523-30577` — the climb to 200 px above the player.
        target_closest(npc, world, player);
        npc.sprite_dir = npc.facing;
        // `:30527` — num633 = 12f feeds only the dead store at
        // `:30544` — kept for the citation, never read.
        let _num633 = 12.0f32;
        let mut num634 = 0.07f32;
        if expert {
            num634 = 0.1;
        }
        // `:30533-30536` — vector76 float halves, INTEGER aim halves,
        // the -200f Y offset BEFORE the sqrt.
        let num635 = player.x + p_half_w - (npc.x + w * 0.5);
        let num636 = player.y + p_half_h - 200.0 - (npc.y + h * 0.5);
        let num637 = (num635 * num635 + num636 * num636).sqrt();
        if num637 < 200.0 {
            // `:30537-30543` — close enough: into the bee volley.
            npc.ai[0] = 1.0;
            npc.ai[1] = 0.0;
            npc.net_update = true;
            return out;
        }
        // `:30544` — `num637 = num633 / num637` is a dead store in the
        // C# (never read) — dropped.
        // `:30545-30576` — the ±num634 approach with the sign-reversal
        // double-add on both axes.
        if npc.vx < num635 {
            npc.vx += num634;
            if npc.vx < 0.0 && num635 > 0.0 {
                npc.vx += num634;
            }
        } else if npc.vx > num635 {
            npc.vx -= num634;
            if npc.vx > 0.0 && num635 < 0.0 {
                npc.vx -= num634;
            }
        }
        if npc.vy < num636 {
            npc.vy += num634;
            if npc.vy < 0.0 && num636 > 0.0 {
                npc.vy += num634;
            }
        } else if npc.vy > num636 {
            npc.vy -= num634;
            if npc.vy > 0.0 && num636 < 0.0 {
                npc.vy -= num634;
            }
        }
    } else if npc.ai[0] == 1.0 {
        // `:30578-30689` — the bee volley.
        npc.local_ai[0] = 0.0;
        target_closest(npc, world, player);
        // `:30582` — vector77: the spawn point, INTEGER half-width +
        // `Next(20) * direction` (the FIRST rand draw of the tick),
        // height × 0.8f.
        let v77x = npc.x + half_w + (rng.next_max(20) * npc.facing) as f32;
        let v77y = npc.y + h * 0.8;
        // `:30583-30586` — vector78 float centre, INTEGER aim halves.
        let mut num638 = player.x + p_half_w - (npc.x + w * 0.5);
        let mut num639 = player.y + p_half_h - (npc.y + h * 0.5);
        let num640 = (num638 * num638 + num639 * num639).sqrt();
        npc.ai[1] += 1.0;
        if expert {
            // `:30590-30598` — the live-player count within 1000 px;
            // the port's one session player makes `num641 / 2` = 0
            // (INTEGER divide) — kept verbatim.
            let mut num641 = 0;
            if !player.dead && ((cx - pcx) * (cx - pcx) + (cy - pcy) * (cy - pcy)).sqrt() < 1000.0 {
                num641 += 1;
            }
            npc.ai[1] += (num641 / 2) as f32;
            // `:30599-30614` — the four f64 HP fractions, +0.25f each.
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
                npc.ai[1] += 0.25;
            }
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
                npc.ai[1] += 0.25;
            }
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
                npc.ai[1] += 0.25;
            }
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
                npc.ai[1] += 0.25;
            }
        }
        // `:30616-30622` — the volley tick: ai[1] past
        // `(int)(40f - 18f * num619)` → reset, ai[2]++.
        let mut flag51 = false;
        if npc.ai[1] > (40.0 - 18.0 * num619) as i32 as f32 {
            npc.ai[1] = 0.0;
            npc.ai[2] += 1.0;
            flag51 = true;
        }
        // `:30623-30636` — the spawn: CanHit from vector77 AND the
        // volley tick. Sound 3 skipped; the host gate folds open.
        if crate::projectile::can_hit(
            world,
            v77x,
            v77y,
            1.0,
            1.0,
            player.x,
            player.y,
            crate::player::WIDTH,
            player.height,
        ) && flag51
        {
            // `:30628-30634` — the type draw (the SECOND rand draw of
            // a spawn tick), the (int) truncation, the normalized ×5
            // velocity from the NPC centre to the player centre.
            let kind = rng.next_range(BEE, BEE_SMALL + 1);
            let (nx, ny) = normalize(pcx - cx, pcy - cy);
            out.bee = Some(BeeSpawn {
                kind,
                x: v77x as i32 as f32,
                y: v77y as i32 as f32,
                vx: nx * 5.0,
                vy: ny * 5.0,
                local_ai0: 60.0,
            });
        }
        // `:30637-30681` — the movement: past 400 px or no sight from
        // 30 px above the spawn point → the 14/0.1 approach from
        // vector77; else ×0.9 damp.
        if num640 > 400.0
            || !crate::projectile::can_hit(
                world,
                v77x,
                v77y - 30.0,
                1.0,
                1.0,
                player.x,
                player.y,
                crate::player::WIDTH,
                player.height,
            )
        {
            let num645 = 0.1f32;
            // `:30639-30644` — num644 = 14f feeds only the dead store
            // `num640 = num644 / sqrt(...)` (`:30644`, never read) —
            // both dropped, the citation kept.
            let _num644 = 14.0f32;
            num638 = player.x + p_half_w - v77x;
            num639 = player.y + p_half_h - v77y;
            if npc.vx < num638 {
                npc.vx += num645;
                if npc.vx < 0.0 && num638 > 0.0 {
                    npc.vx += num645;
                }
            } else if npc.vx > num638 {
                npc.vx -= num645;
                if npc.vx > 0.0 && num638 < 0.0 {
                    npc.vx -= num645;
                }
            }
            if npc.vy < num639 {
                npc.vy += num645;
                if npc.vy < 0.0 && num639 > 0.0 {
                    npc.vy += num645;
                }
            } else if npc.vy > num639 {
                npc.vy -= num645;
                if npc.vy > 0.0 && num639 < 0.0 {
                    npc.vy -= num645;
                }
            }
        } else {
            npc.vx *= 0.9;
            npc.vy *= 0.9;
        }
        npc.sprite_dir = npc.facing;
        // `:30683-30688` — six volleys done → re-pick, ai[1] = 1 (the
        // re-pick never repeats state 1).
        if npc.ai[2] > 5.0 {
            npc.ai[0] = -1.0;
            npc.ai[1] = 1.0;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 3.0 {
        // `:30690-30835` — the stinger barrage. num646 is written but
        // never read in the C# (its only readers are the two dead
        // stores `:30757`/`:30795`) — kept for the citation, `_`-named.
        let mut _num646 = 4.0f32;
        let mut num647 = 0.05f32;
        if expert {
            num647 = 0.075;
            _num646 = 6.0;
        }
        num647 += 0.2 * num619;
        _num646 += 6.0 * num619;
        // `:30701-30705` — vector79 (the Next(20) draw, FIRST of the
        // tick), vector80 float centre, the -300f Y offset.
        let v79x = npc.x + half_w + (rng.next_max(20) * npc.facing) as f32;
        let v79y = npc.y + h * 0.8;
        let mut num648 = player.x + p_half_w - (npc.x + w * 0.5);
        let mut num649 = player.y + p_half_h - 300.0 - (npc.y + h * 0.5);
        let num650 = (num648 * num648 + num649 * num649).sqrt();
        npc.ai[1] += 1.0;
        // `:30707` — flag52 is declared-then-unread in the C# — dropped.
        // `:30708-30713` — the fire period: 40; the expert ladder
        // 15/25/35/30 (the f64 0.1 test, then INTEGER /3 and /2);
        // minus `(int)(5f * num619)`.
        let mut num651 = 40;
        if expert {
            num651 = if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
                15
            } else if npc.hp < npc.max_hp / 3 {
                25
            } else if npc.hp >= npc.max_hp / 2 {
                35
            } else {
                30
            };
        }
        num651 -= (5.0 * num619) as i32;
        // `:30714-30745` — the fire tick: `ai[1] % num651 == num651-1`,
        // the NPC's BOTTOM above the player's TOP (`position.Y`, not
        // the centre), and CanHit from vector79. Sound Item17 skipped.
        if npc.ai[1] % num651 as f32 == (num651 - 1) as f32
            && npc.y + h < player.y
            && crate::projectile::can_hit(
                world,
                v79x,
                v79y,
                1.0,
                1.0,
                player.x,
                player.y,
                crate::player::WIDTH,
                player.height,
            )
        {
            // `:30719-30728` — the shot speed: 8, expert +2, expert
            // below 10% HP +3, +7·enrage.
            let mut num652 = 8.0f32;
            if expert {
                num652 += 2.0;
            }
            if expert && (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
                num652 += 3.0;
            }
            num652 += 7.0 * num619;
            // `:30729-30738` — the jitter windows, floored at 1.
            let mut num653 = (80.0 - 39.0 * num619) as i32;
            let mut num654 = (40.0 - 19.0 * num619) as i32;
            if num653 < 1 {
                num653 = 1;
            }
            if num654 < 1 {
                num654 = 1;
            }
            // `:30739-30741` — the aim uses FLOAT player halves here
            // (`width * 0.5f`, unlike the movement deltas); the X
            // jitter draw precedes the Y draw.
            let num655 = player.x + crate::player::WIDTH * 0.5 - v79x
                + rng.next_range(-num653, num653 + 1) as f32;
            let num656 = player.y + player.height * 0.5 - v79y
                + rng.next_range(-num654, num654 + 1) as f32;
            let num657 = num652 / (num655 * num655 + num656 * num656).sqrt();
            // `:30742-30743` — Projectile.NewProjectile(719, 11, 0f)
            // from vector79; timeLeft = 300.
            out.stinger = Some(StingerShot {
                x: v79x,
                y: v79y,
                vx: num655 * num657,
                vy: num656 * num657,
                damage: 11,
                time_left: 300,
            });
        }
        // `:30746-30828` — the movement: no sight from 30 px above
        // vector79 → the 14/0.1 chase (enrage OVERRIDES the accel to
        // 0.5); else past 100 px → re-aim + the num646/num647 approach
        // with the ×2f reversal boost; else hold.
        if !crate::projectile::can_hit(
            world,
            v79x,
            v79y - 30.0,
            1.0,
            1.0,
            player.x,
            player.y,
            crate::player::WIDTH,
            player.height,
        ) {
            _num646 = 14.0;
            num647 = 0.1;
            if num619 > 0.0 {
                num647 = 0.5;
            }
            num648 = player.x + p_half_w - v79x;
            num649 = player.y + p_half_h - v79y;
            // `:30757` — the `num650 = num646 / sqrt(...)` recompute
            // is a dead store (the branch never reads num650 again) —
            // dropped.
            if npc.vx < num648 {
                npc.vx += num647;
                if npc.vx < 0.0 && num648 > 0.0 {
                    npc.vx += num647;
                }
            } else if npc.vx > num648 {
                npc.vx -= num647;
                if npc.vx > 0.0 && num648 < 0.0 {
                    npc.vx -= num647;
                }
            }
            if npc.vy < num649 {
                npc.vy += num647;
                if npc.vy < 0.0 && num649 > 0.0 {
                    npc.vy += num647;
                }
            } else if npc.vy > num649 {
                npc.vy -= num647;
                if npc.vy > 0.0 && num649 < 0.0 {
                    npc.vy -= num647;
                }
            }
        } else if num650 > 100.0 {
            target_closest(npc, world, player);
            npc.sprite_dir = npc.facing;
            // `:30795` — `num650 = num646 / num650` is a dead store in
            // the C# (the comparisons below use num648/num649 only) —
            // dropped.
            if npc.vx < num648 {
                npc.vx += num647;
                if npc.vx < 0.0 && num648 > 0.0 {
                    // `:30801` — the ×2f reversal boost (this branch
                    // only — the blocked branch above adds once).
                    npc.vx += num647 * 2.0;
                }
            } else if npc.vx > num648 {
                npc.vx -= num647;
                if npc.vx > 0.0 && num648 < 0.0 {
                    npc.vx -= num647 * 2.0;
                }
            }
            if npc.vy < num649 {
                npc.vy += num647;
                if npc.vy < 0.0 && num649 > 0.0 {
                    npc.vy += num647 * 2.0;
                }
            } else if npc.vy > num649 {
                npc.vy -= num647;
                if npc.vy > 0.0 && num649 < 0.0 {
                    npc.vy -= num647 * 2.0;
                }
            }
        }
        // `:30829-30834` — the barrage end: ai[1] past
        // `num651 * (20f - 5f * num619)` → re-pick, ai[1] = 3.
        if npc.ai[1] > num651 as f32 * (20.0 - 5.0 * num619) {
            npc.ai[0] = -1.0;
            npc.ai[1] = 3.0;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 4.0 {
        // `:30836-30859` — the long-range return: the 14/14 weighted
        // steer `(velocity * 14 + normalized * 14) / 15`, then the
        // 2000-px hand-off back to the re-pick.
        npc.local_ai[0] = 1.0;
        let num659 = 14.0f32;
        let num660 = 14.0f32;
        let (nx, ny) = normalize(pcx - cx, pcy - cy);
        npc.vx = (npc.vx * num660 + nx * num659) / (num660 + 1.0);
        npc.vy = (npc.vy * num660 + ny * num659) / (num660 + 1.0);
        npc.facing = if npc.vx < 0.0 { -1 } else { 1 };
        npc.sprite_dir = npc.facing;
        if num620 < 2000.0 {
            npc.ai[0] = -1.0;
            npc.local_ai[0] = 0.0;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        let mut w = World::new("t".into(), 400, 240);
        // The surface at row 100 (1600 px) — the fixtures sit below it
        // (y 2000+) so the enrage term's surface half stays 0.
        w.world_surface = 100.0;
        w
    }

    fn bee(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::QueenBee, x, y, 1, 0.0);
        n.boss = true;
        n
    }

    fn player_at(w: &World, x: f32, y: f32) -> Player {
        let mut p = Player::spawn(w);
        p.x = x;
        p.y = y;
        p
    }

    /// The cited SetDefaults row (`NPC.cs:11855-11871`) matches the
    /// port's kind tables — the wiring step's defaults pin.
    #[test]
    fn s1_d_3_s43_the_defaults_row_matches_the_kind_tables() {
        assert_eq!(QUEEN_BEE.ai_style, NpcKind::QueenBee.ai_style() as i32);
        let (w, h) = NpcKind::QueenBee.size();
        assert_eq!((QUEEN_BEE.width, QUEEN_BEE.height), (w, h));
        assert_eq!(QUEEN_BEE.life_max, NpcKind::QueenBee.max_hp());
        let n = bee(0.0, 0.0);
        assert_eq!(QUEEN_BEE.defense, n.defense());
        assert_eq!(QUEEN_BEE.knock_back_resist, n.knock_back_resist());
        assert_eq!(QUEEN_BEE.value, n.value());
    }

    /// `NPC.cs:30202-30214` — the enrage term: +1 above the surface
    /// (STRICT `<`, the f32 divide promoted to f64), +1 outside the
    /// jungle, +0.5 on get-good; the terms sum to 2.5.
    #[test]
    fn s1_d_3_s43_the_enrage_term_sums_the_three_gates() {
        // Below the surface, in the jungle, plain seed: 0.
        assert_eq!(enrage_term(1600.0, 100.0, true, false), 0.0);
        // Exactly ON the surface row: the strict < keeps 0.
        assert_eq!(enrage_term(1599.9, 100.0, true, false), 1.0);
        assert_eq!(enrage_term(1600.0, 100.0, true, false), 0.0);
        // Outside the jungle only: 1.
        assert_eq!(enrage_term(2000.0, 100.0, false, false), 1.0);
        // Above + outside: 2; get-good adds the half.
        assert_eq!(enrage_term(100.0, 100.0, false, false), 2.0);
        assert_eq!(enrage_term(100.0, 100.0, false, true), 2.5);
        assert_eq!(enrage_term(2000.0, 100.0, true, true), 0.5);
    }

    /// `NPC.cs:30192-30196` — the expert defense ramp is
    /// `defDefense + (int)(20 * (1 - life/lifeMax))` over the BASE 8
    /// (no compounding across ticks); non-expert leaves the defense.
    #[test]
    fn s1_d_3_s43_the_expert_defense_ramps_over_def_defense() {
        let w = air();
        let p = player_at(&w, 2100.0, 2000.0);
        let mut n = bee(2000.0, 2000.0);
        n.hp = n.max_hp / 2; // 1700/3400 → 20 * 0.5 = 10.
        n.ai[0] = 4.0; // the return state: no rand, no re-pick writes.
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, true, &mut rng);
        assert_eq!(n.defense_override, Some(18));
        // A second tick at the same HP must not compound.
        n.ai[0] = 4.0;
        queen_bee_ai(&mut n, &w, &p, true, true, &mut rng);
        assert_eq!(n.defense_override, Some(18));
        // Non-expert: no defense write.
        let mut n2 = bee(2000.0, 2000.0);
        n2.hp = 1;
        n2.ai[0] = 4.0;
        queen_bee_ai(&mut n2, &w, &p, true, false, &mut rng);
        assert_eq!(n2.defense_override, None);
        // The truncation: 3399/3400 → 20 * (1/3400) = 0.0058 → 0.
        let mut n3 = bee(2000.0, 2000.0);
        n3.hp = 3399;
        n3.ai[0] = 4.0;
        queen_bee_ai(&mut n3, &w, &p, true, true, &mut rng);
        assert_eq!(n3.defense_override, Some(8));
    }

    /// `NPC.cs:30216-30232` — the head gates: the 3000-px leash sets
    /// state 4 (STRICT >), a dead player overrides to 5, and the
    /// timeLeft floor only runs outside state 5.
    #[test]
    fn s1_d_3_s43_the_leash_and_dead_player_gates() {
        let w = air();
        // 3000 px exactly: no leash (strict >). The NPC centre is
        // (+33, +33), the player's (+10, +21) — player.x +23 and
        // player.y +12 make the centre distance exactly 3000 on x.
        let p = player_at(&w, 2000.0 + 3000.0 + 23.0, 2000.0 + 12.0);
        let mut n = bee(2000.0, 2000.0);
        n.despawn_timer = 600;
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert_eq!(n.ai[0], 0.0, "dx == 3000 is inside the leash");
        assert_eq!(n.despawn_timer, 600, "600 is above the 60 floor");
        // Far: state 4.
        let far = player_at(&w, 6000.0, 2000.0);
        let mut n2 = bee(2000.0, 2000.0);
        n2.despawn_timer = -1;
        queen_bee_ai(&mut n2, &w, &far, true, false, &mut rng);
        assert_eq!(n2.ai[0], 4.0);
        assert_eq!(n2.despawn_timer, 60, "the -1 default clamps to 60");
        assert!(n2.net_update);
        // Dead AND far: the dead override wins (5, not 4).
        let mut dead = player_at(&w, 6000.0, 2000.0);
        dead.dead = true;
        let mut n3 = bee(2000.0, 2000.0);
        queen_bee_ai(&mut n3, &w, &dead, true, false, &mut rng);
        assert_eq!(n3.ai[0], 5.0);
        // State 5 on entry: no timeLeft floor, EncourageDespawn(10).
        let mut n4 = bee(2000.0, 2000.0);
        n4.ai[0] = 5.0;
        n4.despawn_timer = 600;
        queen_bee_ai(&mut n4, &w, &p, true, false, &mut rng);
        assert_eq!(n4.despawn_timer, 10, "the flee encourages 10");
    }

    /// `NPC.cs:30233-30270` — the flee: vy ×0.98, the left-half drift
    /// (vx>0 damps ×0.98 then -0.08; vx<=0 sets localAI[0]=1 then
    /// -0.08), the facing from the velocity sign, the sprite copy.
    #[test]
    fn s1_d_3_s43_the_flee_drifts_toward_the_left_edge() {
        let w = air(); // width 400 → the half-world split at 3200 px.
        let p = player_at(&w, 2100.0, 2000.0);
        let mut n = bee(2000.0, 2000.0);
        n.ai[0] = 5.0;
        n.vx = 10.0;
        n.vy = 10.0;
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert!((n.vy - 10.0 * 0.98).abs() < 1e-6, "vy {}", n.vy);
        assert!((n.vx - (10.0 * 0.98 - 0.08)).abs() < 1e-6, "vx {}", n.vx);
        assert_eq!(n.facing, 1);
        assert_eq!(n.sprite_dir, 1);
        // Already drifting left: localAI[0] = 1, no damp.
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[0] = 5.0;
        n2.vx = -5.0;
        queen_bee_ai(&mut n2, &w, &p, true, false, &mut rng);
        assert!((n2.vx - (-5.0 - 0.08)).abs() < 1e-6, "vx {}", n2.vx);
        assert_eq!(n2.local_ai[0], 1.0);
        assert_eq!(n2.facing, -1);
        // Right half (x >= 3200): the mirror drifts +0.08.
        let mut n3 = bee(3300.0, 2000.0);
        n3.ai[0] = 5.0;
        n3.vx = 5.0;
        queen_bee_ai(&mut n3, &w, &p, true, false, &mut rng);
        assert_eq!(n3.vx, 5.0 + 0.08);
        assert_eq!(n3.local_ai[0], 1.0);
        let mut n4 = bee(3300.0, 2000.0);
        n4.ai[0] = 5.0;
        n4.vx = -5.0;
        queen_bee_ai(&mut n4, &w, &p, true, false, &mut rng);
        assert!((n4.vx - (-5.0 * 0.98 + 0.08)).abs() < 1e-6, "vx {}", n4.vx);
    }

    /// `NPC.cs:30271-30297` — the re-pick maps Next(3) 1→2, 2→3 (the
    /// picks are {0, 2, 3}), never repeats ai[1], and zeroes ai[1]/[2].
    #[test]
    fn s1_d_3_s43_the_repick_never_repeats_the_last_state() {
        let w = air();
        let p = player_at(&w, 2100.0, 2000.0);
        let mut rng = UnifiedRandom::new(7);
        for last in [0.0f32, 2.0, 3.0] {
            for _ in 0..50 {
                let mut n = bee(2000.0, 2000.0);
                n.ai[0] = -1.0;
                n.ai[1] = last;
                n.ai[2] = 9.0;
                queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
                assert!(
                    n.ai[0] == 0.0 || n.ai[0] == 2.0 || n.ai[0] == 3.0,
                    "the mapped pick is 0/2/3, got {}",
                    n.ai[0]
                );
                assert_ne!(n.ai[0], last, "the do-while redraws the repeat");
                assert_eq!(n.ai[1], 0.0);
                assert_eq!(n.ai[2], 0.0);
                assert!(n.net_update);
            }
        }
    }

    /// `NPC.cs:30316-30323` — the cycle end: STRICT `ai[1] >
    /// 2*(num623 + (int)num619)` on an EVEN tick; the enrage term
    /// truncates (0.5 → 0).
    #[test]
    fn s1_d_3_s43_the_cycle_end_gate_is_strict_and_even() {
        let w = air();
        let p = player_at(&w, 2100.0, 2000.0);
        let mut rng = UnifiedRandom::new(1);
        // num623 = 2, enrage 0 → threshold 4. ai[1] = 4: NOT past.
        let mut n = bee(2000.0, 2000.0);
        n.ai[1] = 4.0;
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert_eq!(n.ai[0], 0.0, "4 > 4 is false");
        // ai[1] = 6: past AND even → re-pick.
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[1] = 6.0;
        queen_bee_ai(&mut n2, &w, &p, true, false, &mut rng);
        assert_eq!(n2.ai[0], -1.0);
        assert_eq!(n2.ai[1], 0.0);
        // ai[1] = 7: past but ODD → no re-pick (the odd half runs).
        let mut n3 = bee(2000.0, 2000.0);
        n3.ai[1] = 7.0;
        queen_bee_ai(&mut n3, &w, &p, true, false, &mut rng);
        assert_eq!(n3.ai[0], 0.0);
        // Enrage 0.5 (get-good only) truncates to 0: threshold stays 4.
        let mut wg = air();
        wg.seeds = tr_core::seeds::SeedFlags::from(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
        let mut n4 = bee(2000.0, 2000.0);
        n4.ai[1] = 4.0;
        queen_bee_ai(&mut n4, &wg, &p, true, false, &mut rng);
        assert_eq!(n4.ai[0], 0.0, "(int)(1f * 0.5f) = 0");
    }

    /// `NPC.cs:30327-30364` — the level charge: the STRICT 20-px Y
    /// window (+20·enrage), the normalized ×12 velocity (+7·enrage),
    /// localAI[0]=1, ai[1]++, ai[2]=0.
    #[test]
    fn s1_d_3_s43_the_level_charge_aims_at_twelve_plus_enrage() {
        let w = air();
        // Player centre exactly 20 px below the NPC centre: the
        // strict < fails → the hover branch (localAI[0] = 0).
        let p = player_at(&w, 2400.0 - 10.0, 2000.0 + 33.0 - 21.0 + 20.0);
        let mut n = bee(2000.0, 2000.0);
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert_eq!(n.local_ai[0], 0.0, "dy == 20 is outside the window");
        // 19 px below: the charge fires at speed 12.
        let p2 = player_at(&w, 2400.0 - 10.0, 2000.0 + 33.0 - 21.0 + 19.0);
        let mut n2 = bee(2000.0, 2000.0);
        queen_bee_ai(&mut n2, &w, &p2, true, false, &mut rng);
        assert_eq!(n2.local_ai[0], 1.0);
        assert_eq!(n2.ai[1], 1.0);
        let speed = (n2.vx * n2.vx + n2.vy * n2.vy).sqrt();
        assert!((speed - 12.0).abs() < 1e-3, "speed {speed}");
        // Enrage 1 (outside the jungle): the window widens to 40 and
        // the speed to 19.
        let p3 = player_at(&w, 2400.0 - 10.0, 2000.0 + 33.0 - 21.0 + 39.0);
        let mut n3 = bee(2000.0, 2000.0);
        queen_bee_ai(&mut n3, &w, &p3, false, false, &mut rng);
        assert_eq!(n3.local_ai[0], 1.0, "39 < 20 + 20*1");
        let speed3 = (n3.vx * n3.vx + n3.vy * n3.vy).sqrt();
        assert!((speed3 - 19.0).abs() < 1e-3, "speed {speed3}");
        // The expert ladder: full HP → 16; below 10% → 24.
        let mut n4 = bee(2000.0, 2000.0);
        queen_bee_ai(&mut n4, &w, &p2, true, true, &mut rng);
        let s4 = (n4.vx * n4.vx + n4.vy * n4.vy).sqrt();
        assert!((s4 - 16.0).abs() < 1e-3, "expert full HP speed {s4}");
        let mut n5 = bee(2000.0, 2000.0);
        n5.hp = 300; // < 340 = 10%.
        queen_bee_ai(&mut n5, &w, &p2, true, true, &mut rng);
        let s5 = (n5.vx * n5.vx + n5.vy * n5.vy).sqrt();
        assert!((s5 - 24.0).abs() < 1e-3, "expert low HP speed {s5}");
    }

    /// `NPC.cs:30365-30430` — the hover: the ±(12+3·enrage) Y cap, the
    /// 0.15+0.5·enrage accel, the X band (>600 / <300 / ×0.8) and the
    /// ±16 X cap.
    #[test]
    fn s1_d_3_s43_the_hover_caps_and_x_band() {
        let w = air();
        // Player far above (dy > 20): the hover branch. dx = 700 > 600.
        let p = player_at(&w, 2700.0, 1000.0);
        let mut n = bee(2000.0, 2000.0);
        n.facing = 1;
        n.vx = 20.0; // past the +16 cap.
        n.vy = -20.0; // past the -12 cap.
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert_eq!(n.local_ai[0], 0.0);
        assert_eq!(n.vy, -12.0, "the Y cap is 12 + 3*0");
        assert_eq!(n.vx, 16.0, "the X cap is 16");
        // The mid band (300 <= dx <= 600): ×0.8 damp.
        let p2 = player_at(&w, 2450.0, 1000.0);
        let mut n2 = bee(2000.0, 2000.0);
        n2.vx = 10.0;
        queen_bee_ai(&mut n2, &w, &p2, true, false, &mut rng);
        assert_eq!(n2.vx, 8.0);
        // The close band (dx < 300): the retreat subtracts 0.15*dir.
        let p3 = player_at(&w, 2100.0, 1000.0);
        let mut n3 = bee(2000.0, 2000.0);
        n3.facing = 1;
        n3.vx = 10.0;
        queen_bee_ai(&mut n3, &w, &p3, true, false, &mut rng);
        assert!((n3.vx - (10.0 - 0.15)).abs() < 1e-6, "vx {}", n3.vx);
        // The accel toward the player: the player is ABOVE (1000 <
        // 2000) → vy -= 0.15 from 0 (`:30393-30400`).
        let mut n4 = bee(2000.0, 2000.0);
        queen_bee_ai(&mut n4, &w, &p, true, false, &mut rng);
        assert_eq!(n4.vy, -0.15);
        // Enrage 1: cap 15, accel 0.15 + 0.5 (f32 sum — tolerance).
        let mut n5 = bee(2000.0, 2000.0);
        queen_bee_ai(&mut n5, &w, &p, false, false, &mut rng);
        assert!((n5.vy - -0.65).abs() < 1e-6, "vy {}", n5.vy);
        n5.vy = -20.0;
        queen_bee_ai(&mut n5, &w, &p, false, false, &mut rng);
        // -20 - 0.65 = -20.65 → clamped to -15.
        assert_eq!(n5.vy, -15.0);
    }

    /// `NPC.cs:30432-30521` — the odd half: the turn triggers (facing
    /// away past num630, or |dy| > num630·1.5), the enrage ×0.5 brake
    /// ONLY on the trigger tick, the decel chain and the
    /// |vx|+|vy| < num632 hand-off.
    #[test]
    fn s1_d_3_s43_the_turn_brake_and_decel_handoff() {
        let w = air();
        // Player to the RIGHT (num631 = -1), NPC velocity NEGATIVE →
        // facing -1 == num631 (facing away), dx ≈ 690 > 600 → the turn
        // triggers (`:30462-30472`).
        let p = player_at(&w, 2700.0, 2000.0 + 12.0);
        let mut n = bee(2000.0, 2000.0);
        n.ai[1] = 1.0; // odd.
        n.vx = -8.0; // facing becomes -1.
        n.vy = 4.0;
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert_eq!(n.ai[2], 1.0, "the turn armed");
        // No enrage → no ×0.5; the decel ×0.9 applies.
        assert!((n.vx - (-8.0 * 0.9)).abs() < 1e-6, "vx {}", n.vx);
        assert!((n.vy - 4.0 * 0.9).abs() < 1e-6, "vy {}", n.vy);
        // Enrage 1: num630 = 500, dx 690 > 500 → trigger + ×0.5 brake
        // then ×0.9 then ×0.7.
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[1] = 1.0;
        n2.vx = -8.0;
        n2.vy = 4.0;
        queen_bee_ai(&mut n2, &w, &p, false, false, &mut rng);
        assert!((n2.vx - (-8.0 * 0.5 * 0.9 * 0.7)).abs() < 1e-5, "vx {}", n2.vx);
        assert!((n2.vy - (4.0 * 0.5 * 0.9 * 0.7)).abs() < 1e-5, "vy {}", n2.vy);
        // The hand-off: tiny velocity → ai[2]=0, ai[1]++.
        let mut n3 = bee(2000.0, 2000.0);
        n3.ai[1] = 1.0;
        n3.ai[2] = 1.0;
        n3.vx = 0.05;
        n3.vy = 0.0;
        // dx small, dy small: no new trigger, but ai[2] already 1.
        let p3 = player_at(&w, 2100.0, 2000.0 + 12.0);
        queen_bee_ai(&mut n3, &w, &p3, true, false, &mut rng);
        assert_eq!(n3.ai[2], 0.0);
        assert_eq!(n3.ai[1], 2.0);
        assert!(n3.net_update);
        // No trigger and ai[2]==0: localAI[0]=1 (the charge sprite).
        let mut n4 = bee(2000.0, 2000.0);
        n4.ai[1] = 1.0;
        n4.vx = 8.0;
        queen_bee_ai(&mut n4, &w, &p3, true, false, &mut rng);
        assert_eq!(n4.local_ai[0], 1.0);
        assert_eq!(n4.ai[2], 0.0);
    }

    /// `NPC.cs:30533-30543` — the climb's STRICT 200-px hand-off into
    /// state 1 with ai[1]=0; the -200f offset rides BEFORE the sqrt.
    #[test]
    fn s1_d_3_s43_the_climb_hands_off_inside_200() {
        let w = air();
        // The aim point is (pcx, pcy - 200). Same x, the NPC centre
        // 200 px ABOVE the aim point: pcy = cy + 200 + 200 = 2433 →
        // num636 = (2433 - 200) - 2033 = 200 → dist exactly 200: no
        // hand-off (strict <). Player.y = pcy - 21.
        let p = player_at(&w, 2033.0 - 10.0, 2433.0 - 21.0);
        let mut n = bee(2000.0, 2000.0);
        n.ai[0] = 2.0;
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert_eq!(n.ai[0], 2.0, "dist == 200 holds the climb");
        // 199 px: the hand-off.
        let p2 = player_at(&w, 2033.0 - 10.0, 2432.0 - 21.0);
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[0] = 2.0;
        queen_bee_ai(&mut n2, &w, &p2, true, false, &mut rng);
        assert_eq!(n2.ai[0], 1.0);
        assert_eq!(n2.ai[1], 0.0);
        assert!(n2.net_update);
        // Far: the ±0.07 approach with the reversal double-add — the
        // first add must leave vx still negative (-0.1 + 0.07 = -0.03)
        // for the second to fire (`:30548-30551`).
        let p3 = player_at(&w, 2600.0, 2600.0);
        let mut n3 = bee(2000.0, 2000.0);
        n3.ai[0] = 2.0;
        n3.vx = -0.1; // moving left while the aim is right → double.
        queen_bee_ai(&mut n3, &w, &p3, true, false, &mut rng);
        assert!((n3.vx - (-0.1 + 0.07 + 0.07)).abs() < 1e-6, "vx {}", n3.vx);
        // Expert: 0.1 per add — the first add leaves vx negative
        // (-0.15 + 0.1 = -0.05), so the second fires too.
        let mut n4 = bee(2000.0, 2000.0);
        n4.ai[0] = 2.0;
        n4.vx = -0.15;
        queen_bee_ai(&mut n4, &w, &p3, true, true, &mut rng);
        assert!((n4.vx - (-0.15 + 0.1 + 0.1)).abs() < 1e-6, "vx {}", n4.vx);
    }

    /// `NPC.cs:30582-30636`, `:30683-30688` — the volley: the STRICT
    /// `(int)(40 - 18·enrage)` threshold, the bee type draw in
    /// {210, 211}, the (int) truncation, the normalized ×5 velocity,
    /// localAI[0]=60, and the ai[2] > 5 exit with ai[1]=1.
    #[test]
    fn s1_d_3_s43_the_bee_volley_spawns_and_exits() {
        let w = air();
        // The player below the surface row (y 2400 > 1600) so the
        // enrage term's surface half stays 0 in every fixture.
        let p = player_at(&w, 2200.0, 2400.0);
        let mut rng = UnifiedRandom::new(3);
        // ai[1] = 39 → ++ = 40: NOT past 40 (strict >), no spawn.
        let mut n = bee(2000.0, 2000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 39.0;
        let out = queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert!(out.bee.is_none());
        assert_eq!(n.ai[1], 40.0);
        // ai[1] = 40 → ++ = 41 > 40: the volley tick, ai[2]++, spawn.
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[0] = 1.0;
        n2.ai[1] = 40.0;
        let out2 = queen_bee_ai(&mut n2, &w, &p, true, false, &mut rng);
        assert_eq!(n2.ai[1], 0.0);
        assert_eq!(n2.ai[2], 1.0);
        let b = out2.bee.expect("the open-air CanHit passes");
        assert!(b.kind == BEE || b.kind == BEE_SMALL);
        assert_eq!(b.x, b.x.trunc(), "the (int) truncation");
        assert_eq!(b.local_ai0, 60.0);
        let sp = (b.vx * b.vx + b.vy * b.vy).sqrt();
        assert!((sp - 5.0).abs() < 1e-3, "the normalized x5, got {sp}");
        // Enrage 1: the threshold is (int)(40-18) = 22.
        let mut n3 = bee(2000.0, 2000.0);
        n3.ai[0] = 1.0;
        n3.ai[1] = 22.0; // ++ = 23 > 22.
        let out3 = queen_bee_ai(&mut n3, &w, &p, false, false, &mut rng);
        assert!(out3.bee.is_some());
        assert_eq!(n3.ai[2], 1.0);
        // The exit: ai[2] = 5 → ++ = 6 > 5 → re-pick with ai[1] = 1.
        let mut n4 = bee(2000.0, 2000.0);
        n4.ai[0] = 1.0;
        n4.ai[1] = 40.0;
        n4.ai[2] = 5.0;
        queen_bee_ai(&mut n4, &w, &p, true, false, &mut rng);
        assert_eq!(n4.ai[0], -1.0);
        assert_eq!(n4.ai[1], 1.0);
    }

    /// `NPC.cs:30708-30745`, `:30829-30834` — the barrage: the
    /// `ai[1] % 40 == 39` fire tick, the NPC-bottom < player-TOP gate,
    /// the 719/11/300 payload, the ±80/±40 jitter windows (enrage
    /// shrinks them to 41/21, floored at 1), and the
    /// `40 * (20 - 5·enrage)` exit with ai[1]=3.
    #[test]
    fn s1_d_3_s43_the_stinger_barrage_fires_on_the_period() {
        let w = air();
        // The player far BELOW (the NPC bottom 2066 < player.y 2400).
        let p = player_at(&w, 2200.0, 2400.0);
        let mut rng = UnifiedRandom::new(5);
        // ai[1] = 37 → ++ = 38: 38 % 40 != 39 → no shot.
        let mut n = bee(2000.0, 2000.0);
        n.ai[0] = 3.0;
        n.ai[1] = 37.0;
        let out = queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert!(out.stinger.is_none());
        // ai[1] = 38 → ++ = 39: the fire tick.
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[0] = 3.0;
        n2.ai[1] = 38.0;
        let out2 = queen_bee_ai(&mut n2, &w, &p, true, false, &mut rng);
        let s = out2.stinger.expect("the fire tick");
        assert_eq!(s.damage, 11);
        assert_eq!(s.time_left, 300);
        // The spawn point: x + 33 + Next(20)*facing, y + 66*0.8.
        assert!(s.x >= 2033.0 && s.x < 2053.0, "x {}", s.x);
        assert_eq!(s.y, 2000.0 + 66.0 * 0.8);
        let sp = (s.vx * s.vx + s.vy * s.vy).sqrt();
        assert!((sp - 8.0).abs() < 1e-3, "the normalized x8, got {sp}");
        // The player ABOVE the NPC bottom: no shot even on the tick.
        let above = player_at(&w, 2200.0, 1900.0);
        let mut n3 = bee(2000.0, 2000.0);
        n3.ai[0] = 3.0;
        n3.ai[1] = 38.0;
        let out3 = queen_bee_ai(&mut n3, &w, &above, true, false, &mut rng);
        assert!(out3.stinger.is_none());
        // The exit: ai[1] past 40*20 = 800 → re-pick, ai[1] = 3.
        let mut n4 = bee(2000.0, 2000.0);
        n4.ai[0] = 3.0;
        n4.ai[1] = 800.0; // ++ = 801 > 800.
        queen_bee_ai(&mut n4, &w, &p, true, false, &mut rng);
        assert_eq!(n4.ai[0], -1.0);
        assert_eq!(n4.ai[1], 3.0);
        // Enrage 1: the period is 40-5 = 35, the exit 35*15 = 525.
        let mut n5 = bee(2000.0, 2000.0);
        n5.ai[0] = 3.0;
        n5.ai[1] = 525.0; // ++ = 526 > 525.
        queen_bee_ai(&mut n5, &w, &p, false, false, &mut rng);
        assert_eq!(n5.ai[0], -1.0);
        assert_eq!(n5.ai[1], 3.0);
        // The expert period at full HP: 35 (life >= lifeMax/2).
        let mut n6 = bee(2000.0, 2000.0);
        n6.ai[0] = 3.0;
        n6.ai[1] = 33.0; // ++ = 34 == 35-1 → fires.
        let out6 = queen_bee_ai(&mut n6, &w, &p, true, true, &mut rng);
        let s6 = out6.stinger.expect("the expert 35 period");
        let sp6 = (s6.vx * s6.vx + s6.vy * s6.vy).sqrt();
        assert!((sp6 - 10.0).abs() < 1e-3, "expert speed 8+2, got {sp6}");
    }

    /// `NPC.cs:30746-30828` — the barrage movement: the blocked chase
    /// adds num647 ONCE on reversal; the visible >100 branch adds
    /// ×2f; enrage OVERRIDES the blocked accel to 0.5.
    #[test]
    fn s1_d_3_s43_the_barrage_reversal_boosts_differ_per_branch() {
        let w = air();
        // Visible, far (>100): the ×2f boost — the first add leaves
        // vx negative (-0.1 + 0.05 = -0.05) so the boost fires.
        let p = player_at(&w, 2600.0, 2600.0);
        let mut n = bee(2000.0, 2000.0);
        n.ai[0] = 3.0;
        n.vx = -0.1;
        let mut rng = UnifiedRandom::new(9);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        // num647 = 0.05: -0.1 + 0.05 + 0.10 = 0.05.
        assert!((n.vx - 0.05).abs() < 1e-6, "vx {}", n.vx);
        // Blocked (a solid wall between the spawn point and the
        // player): the single add. Wall column at tile x 140
        // (px 2240..2256) spanning the sight rows.
        let mut wb = air();
        for y in 100..180 {
            let i = wb.index(140, y);
            wb.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            wb.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[0] = 3.0;
        n2.vx = -0.2;
        queen_bee_ai(&mut n2, &wb, &p, true, false, &mut rng);
        // num647 = 0.1: -0.2 + 0.1 + 0.1 = 0.0 (single extra add).
        assert!(n2.vx.abs() < 1e-6, "vx {}", n2.vx);
        // Enrage 1 + blocked: the accel OVERRIDES to 0.5.
        let mut n3 = bee(2000.0, 2000.0);
        n3.ai[0] = 3.0;
        n3.vx = -0.6;
        queen_bee_ai(&mut n3, &wb, &p, false, false, &mut rng);
        assert!((n3.vx - (-0.6 + 0.5 + 0.5)).abs() < 1e-6, "vx {}", n3.vx);
    }

    /// `NPC.cs:30836-30859` — the return: the (v·14 + n̂·14)/15 steer,
    /// localAI[0]=1, and the STRICT 2000-px hand-off (num620 measured
    /// at the tick head).
    #[test]
    fn s1_d_3_s43_the_return_steers_and_hands_off() {
        let w = air();
        // ~2477 px away (centre to centre): the steer, no hand-off.
        // The centres are both offset (+33, +33) vs (+10, +21) — keep
        // y equal so the aim is pure +x and the normalized vector is
        // exactly (1, 0).
        let p = player_at(&w, 2000.0 + 2500.0, 2000.0 + 12.0);
        let mut n = bee(2000.0, 2000.0);
        n.ai[0] = 4.0;
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(1);
        queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert_eq!(n.ai[0], 4.0);
        assert_eq!(n.local_ai[0], 1.0);
        // The aim is +x: vx = (0*14 + 1*14)/15 = 14/15.
        assert!((n.vx - 14.0 / 15.0).abs() < 1e-3, "vx {}", n.vx);
        // Inside 2000: the hand-off, localAI[0] back to 0.
        let p2 = player_at(&w, 2000.0 + 1500.0, 2000.0 + 12.0);
        let mut n2 = bee(2000.0, 2000.0);
        n2.ai[0] = 4.0;
        queen_bee_ai(&mut n2, &w, &p2, true, false, &mut rng);
        assert_eq!(n2.ai[0], -1.0);
        assert_eq!(n2.local_ai[0], 0.0);
    }

    /// The RNG witness: states 0/2/4/5 draw NOTHING (the stream is
    /// byte-identical to an untouched control); the rand-drawing
    /// states are deterministic per seed and differ across seeds.
    #[test]
    fn s1_d_3_s43_the_rng_draws_only_in_the_cited_states() {
        let w = air();
        let p = player_at(&w, 2200.0, 2400.0);
        // The draw-free states: 0 (both halves), 2, 4, 5.
        for state in [0.0f32, 2.0, 4.0, 5.0] {
            let mut rng = UnifiedRandom::new(42);
            let mut control = UnifiedRandom::new(42);
            for tick in 0..120 {
                let mut n = bee(2000.0, 2000.0);
                n.ai[0] = state;
                n.ai[1] = (tick % 8) as f32; // both parities of state 0.
                queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
            }
            assert_eq!(
                rng.next(),
                control.next(),
                "state {state} must not touch the stream"
            );
        }
        // The drawing states: same seed → same trajectory; different
        // seeds diverge. The trace carries the bee spawn X (the direct
        // Next(20) payload) beside the position/state, so the volley
        // draws witness the stream even where the movement is
        // draw-independent. The NPC sits far below-right of the player
        // so the volley's movement stays in the 14/0.1 approach (the
        // ×0.9 damp branch would erase the draw's effect).
        let far = player_at(&w, 1000.0, 1000.0);
        let run = |seed: i32| -> Vec<(f32, f32, f32, Option<f32>)> {
            let mut rng = UnifiedRandom::new(seed);
            let mut n = bee(2000.0, 2000.0);
            n.ai[0] = 1.0;
            let mut trace = Vec::new();
            for _ in 0..300 {
                let out = queen_bee_ai(&mut n, &w, &far, true, false, &mut rng);
                // The sim's integration (UpdateNPC, not the arm).
                n.x += n.vx;
                n.y += n.vy;
                trace.push((n.x, n.y, n.ai[0], out.bee.map(|b| b.x)));
            }
            trace
        };
        assert_eq!(run(11), run(11), "the same seed replays exactly");
        assert_ne!(run(11), run(12), "the Next(20) offset is live");
    }

    /// The wall gate: a solid column between the spawn point and the
    /// player blocks the bee (`Collision.CanHit`, `NPC.cs:30623`) —
    /// the volley tick still resets ai[1] and bumps ai[2].
    #[test]
    fn s1_d_3_s43_the_bee_spawn_needs_line_of_sight() {
        let mut w = air();
        for y in 100..180 {
            let i = w.index(140, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The player beyond the wall column (tile x 140 = px 2240..
        // 2256); y 2400 keeps the surface enrage half at 0.
        let p = player_at(&w, 2600.0, 2400.0);
        let mut n = bee(2000.0, 2000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 40.0;
        let mut rng = UnifiedRandom::new(3);
        let out = queen_bee_ai(&mut n, &w, &p, true, false, &mut rng);
        assert!(out.bee.is_none(), "the wall blocks the spawn");
        assert_eq!(n.ai[1], 0.0, "the volley tick still ran");
        assert_eq!(n.ai[2], 1.0);
    }
}
