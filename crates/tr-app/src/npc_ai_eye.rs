//! S1-D-3 (eye re-port, audit A-1 seed A1-3): `AI_002_FloatingEye`
//! (`NPC.cs:52274-52731`) ported WHOLE as a pure step + `EyeEnv` seam.
//! WIRED by W4-b2b: the aiStyle-2 axis arm (`npc.rs::ai_002_eye_family`)
//! runs this machine for the seven ported kinds; the `update_eye`
//! stand-in and the `FlyerParams::FLOATING_EYE` false citation are
//! retired (the caster/style10 precedent, completed).
//!
//! The A-1 audit (`docs/research/54-audit-slime-eye.md` §2, branch
//! `docs/audit-slime-eye`) found the live arm is a normalized-aim
//! `0.12`-accel / `±3`-clamp chase plus an invented `sin(anim*3)*0.4`
//! position bob — **`FlyerParams::FLOATING_EYE {0.12, 3.0}`'s citation of
//! `AI_002` is FALSE**: neither constant appears anywhere in
//! `:52274-52731`. The real machine is axis-separated direction-driven
//! steering (`4/1.5 × (2 − scale)` caps), `oldVelocity × −0.5` bounce
//! kicks with direction-gated minimum floors, the dawn-flee
//! `EncourageDespawn(10)` gate, per-kind branches and the wet tail.
//!
//! ## Census (SetDefaults `aiStyle = 2`, 13 kinds)
//!
//! | type | kind | SetDefaults (`aiStyle = 2` line) | arm |
//! |---|---|---|---|
//! | 2 | DemonEye | `NPC.cs:8590` | default `:52639-52708` |
//! | 116 | TheHungryII | `:10397` | own `:52425-52500` |
//! | 133 | WanderingEye | `:10650` | half-life `:52502-52637` |
//! | 170/171/180 | Pigron trio | `:11132`/`:11145`/`:11283` | phasing `:52326-52423` |
//! | 190-194 | Cataract/Sleepy/Dialated/Green/PurpleEye | `:11431`/`:11444`/`:11457`/`:11470`/`:11483` | default |
//! | 317/318 | DemonEyeOwl/Spaceship | `:13011`/`:13024` | default |
//!
//! (`ID/NPCID.cs:10681`/`:10909`/`:10943`/`:11017-11037`/`:11057-11065`/
//! `:11311-11313`.) The port's seven `NpcKind`s (DemonEye 2, WanderingEye
//! 133, CataractEye 190, SleepyEye 191, DialatedEye 192, GreenEye 193,
//! PurpleEye 194) ride the default/133 arms; 116/170/171/180/317/318 have
//! no kinds yet — their branches are ported here in full, keyed on the
//! vanilla `npc_type` (the `npc_ai_style10.rs` convention), so the wiring
//! only needs the kind rows. Like style10, the machine keys on the
//! explicit type, never on `npc.kind`; the wiring passes
//! `npc.kind.net_id()`.
//!
//! ## RNG order (by construction)
//!
//! The arm's `Main.rand` draws, in C# order, per tick:
//! 1. `:52276` — types 170/171/180 ONLY: `Next(1000) == 0` gates the
//!    `SoundEngine.PlaySound(29)` moan. The draw runs FIRST, before the
//!    bounce kicks; the sound cue rides `EyeOut::pigron_moan` (the sound
//!    task's skip — the DRAW is not skipped).
//! 2. `:52493` — type 116 ONLY: `Next(40) == 0` gates its dust burst,
//!    inside its branch (after the steering).
//! 3. `:52710` — types 2/133/190-194 ONLY: `Next(40) == 0` gates the
//!    family dust burst. The C# `&&` short-circuits on the type list, so
//!    116/170/171/180/317/318 draw NOTHING here.
//!
//! The dawn-flee predicate (`:52752-52763`), `EncourageDespawn`
//! (`:7249-7256`), `TargetClosest` (`npc_target.rs`, RNG-free) and every
//! steering branch draw no RNG. So each ported eye kind consumes exactly
//! ONE `Next(40)` per tick; the Pigrons exactly ONE `Next(1000)`; 116 one
//! `Next(40)`; 317/318 ZERO. The `Dust.NewDust` INTERNAL draws
//! (`Dust.cs:114-148`'s pool-load gates + the position jitters) are the
//! dust-feed task's documented skip (S1-E-1) — the burst request rides
//! `EyeOut::dust` with the C#'s exact arguments; the witnesses pin the
//! arm-level counts with the mirrored-oracle stream-position method
//! (WRITTEN NOT RUN — code-only mode).
//!
//! ## Folds (documented, ANALYSIS "The eye aiStyle-2 re-port (S1-D-3)")
//!
//! * `this.noGravity = true` (`:52280`) — `Npc` has no `noGravity` field;
//!   the port's flyer arms never apply gravity (the caller's integration
//!   tail is flyer-shaped). The write is a no-op fold.
//! * `noTileCollide` — no `Npc` field either: the bounce gate reads
//!   `EyeEnv::no_tile_collide` (false for every ported eye kind) and the
//!   Pigron phase write rides `EyeOut::no_tile_collide`.
//! * `Main.IsItDay()` (`Main.cs:62825-62832`) — `remixWorld` has no
//!   runtime flag in the port; the gate reads `world.day_time` (the
//!   non-remix arm), the same fold every dawn gate uses.
//! * `Main.player[target].ZoneGraveyard` — no player field yet; the
//!   parallel `EyeEnv::player_graveyards` slice carries it (the style14
//!   `players_wet` convention); the wiring passes false until the scene
//!   metrics land. Slot 255 (the `ClearTarget` sentinel) is an inactive
//!   player in the C# — `ZoneGraveyard` false, so the out-of-range read
//!   folds to false.
//! * The arm does NOT integrate position: `NPC.Update`'s generic tail is
//!   the caller's (the `npc_ai_style31.rs` precedent). The lens drops are
//!   FAITHFUL already (`loot_db_gen.rs:540`, audit §2d) — untouched.

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The vanilla `NPCID`s of the aiStyle-2 family (the census above).
pub const DEMON_EYE: i32 = 2;
pub const THE_HUNGRY_II: i32 = 116;
pub const WANDERING_EYE: i32 = 133;
pub const PIGRON_CORRUPTION: i32 = 170;
pub const PIGRON_HALLOW: i32 = 171;
pub const PIGRON_CRIMSON: i32 = 180;
pub const CATARACT_EYE: i32 = 190;
pub const SLEEPY_EYE: i32 = 191;
pub const DIALATED_EYE: i32 = 192;
pub const GREEN_EYE: i32 = 193;
pub const PURPLE_EYE: i32 = 194;
pub const DEMON_EYE_OWL: i32 = 317;
pub const DEMON_EYE_SPACESHIP: i32 = 318;

/// `NPC.DespawnEncouragement_AIStyle2_FloatingEye_IsDiscouraged`
/// (`NPC.cs:52752-52763`): day (`Main.IsItDay()`) && the NPC's top-left
/// Y at or above the surface line (`(double)npcPosition.Y <=
/// Main.worldSurface * 16.0`) && the TARGET player not in a graveyard,
/// for types 2/133/190-194/317/318. Every other type returns false.
/// RNG-free.
pub fn despawn_discouraged(
    npc_type: i32,
    npc_y: f32,
    world_surface: f64,
    is_day: bool,
    target_zone_graveyard: bool,
) -> bool {
    // `:52754` — the three shared terms, in the C#'s order.
    if !target_zone_graveyard && is_day && (npc_y as f64) <= world_surface * 16.0 {
        // `:52756-52760` — the type list; the odd `return npcID == 318`
        // tail is the C#'s own shape (317 is in the != chain, 318 is the
        // final equality) — ported verbatim.
        if npc_type != DEMON_EYE
            && npc_type != WANDERING_EYE
            && npc_type != CATARACT_EYE
            && npc_type != SLEEPY_EYE
            && npc_type != DIALATED_EYE
            && npc_type != GREEN_EYE
            && npc_type != PURPLE_EYE
            && npc_type != DEMON_EYE_OWL
        {
            return npc_type == DEMON_EYE_SPACESHIP;
        }
        return true;
    }
    false
}

/// The family dust burst's arguments (`:52713-52715` / `:52496-52498`):
/// `Dust.NewDust((position.X, position.Y + height * 0.25f), width,
/// (int)(height * 0.5f), 5, velocity.X, 2f)` inside the `netOffset`
/// bracket, then `dust.velocity.X *= 0.5f; dust.velocity.Y *= 0.1f`.
/// The dust-feed wiring consumes this; the visual is the documented skip.
#[derive(Clone, Copy, Debug)]
pub struct EyeDust {
    /// The burst box's top-left, INCLUDING `netOffset` (the C# brackets
    /// the position add/remove around the `NewDust` call).
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
    /// `Dust.NewDust`'s `Type` argument — 5 (the smoke puff).
    pub dust_type: i32,
    /// The C#'s `SpeedX`/`SpeedY` arguments (pre-scaling).
    pub speed_x: f32,
    pub speed_y: f32,
    /// The post-spawn velocity multipliers (`×0.5` X, `×0.1` Y).
    pub vel_scale_x: f32,
    pub vel_scale_y: f32,
}

/// The arm's side effects the port has no fields/feeds for yet.
#[derive(Clone, Copy, Debug, Default)]
pub struct EyeOut {
    /// `SoundEngine.PlaySound(29, (int)position.X, (int)position.Y, 9)`
    /// (`:52278`) — true on the Pigron moan tick (the `Next(1000) == 0`
    /// gate fired). The sound feed is the wiring's; the draw already ran.
    pub pigron_moan: bool,
    /// The Pigron phase's `noTileCollide` write (`:52350`/`:52356`) —
    /// the caller's integration tail reads it (`Npc` has no field).
    /// `None` for every non-Pigron type (the branch never runs).
    pub no_tile_collide: Option<bool>,
    /// Type 116's `Lighting.AddLight((int)(position.X + (float)(width /
    /// 2)) / 16, (int)(position.Y + (float)(height / 2)) / 16, 0.3f,
    /// 0.2f, 0.1f)` (`:52428`) — INTEGER halves and INTEGER division,
    /// the `npc_lights.rs` convention. The light feed is S1-E-5's.
    pub light: Option<(i32, i32, [f32; 3])>,
    /// The family/116 dust burst when its `Next(40)` gate fired.
    pub dust: Option<EyeDust>,
}

/// The caller seam — everything the arm reads outside the NPC itself.
pub struct EyeEnv<'a> {
    pub world: &'a World,
    /// The `Main.player[]` slots as `TargetClosest` sees them (the
    /// `npc_target.rs` seam, S1-D-3p). Indexed by `npc.target` after the
    /// scan, like the C#'s `Main.player[this.target]` (`:52328`).
    pub players: &'a [TargetCandidate],
    /// `Main.player[j].ZoneGraveyard` per slot — the dawn gate's term
    /// (`:52754`). No player field yet: the wiring passes false (the
    /// module fold note); slot 255 folds to false (inactive player).
    pub player_graveyards: &'a [bool],
    /// `this.noTileCollide` at entry — false for every ported eye kind
    /// (no `Npc` field; the Pigron phase write rides `EyeOut`).
    pub no_tile_collide: bool,
    /// `Main.EverLastingTicker` — the `targetSetFrame` stamp
    /// (`npc_target.rs`; the zombie wiring passes 0 until plumbed).
    pub ticker: u64,
}

/// `AI_002_FloatingEye` (`NPC.cs:52274-52731`), whole. `npc_type` is the
/// vanilla `NPCID` (the wiring passes `npc.kind.net_id()`); the machine
/// never reads `npc.kind` (the style10 convention). The position
/// integration is the caller's generic `NPC.Update` tail — this step
/// only writes velocity/steering/state, exactly like the C# arm.
pub fn floating_eye_ai(npc: &mut Npc, npc_type: i32, env: &EyeEnv<'_>, rng: &mut UnifiedRandom) -> EyeOut {
    let mut out = EyeOut::default();
    let (w, h) = npc.size();
    let pigron = npc_type == PIGRON_CORRUPTION || npc_type == PIGRON_HALLOW || npc_type == PIGRON_CRIMSON;
    // `:52276-52279` — the Pigron moan gate: the Next(1000) draw runs
    // FIRST every tick for 170/171/180 (short-circuit: other types draw
    // nothing). The PlaySound cue rides EyeOut (the sound task's skip).
    if pigron && rng.next_max(1000) == 0 {
        out.pigron_moan = true;
    }
    // `:52280` — `this.noGravity = true`: the no-field fold (module note).
    // `:52281-52307` — the bounce kicks, gated on !noTileCollide. The
    // kick reads oldVelocity (LAST tick's velocity, `NPC.cs:92603-92604`
    // — the port's `old_vx`/`old_vy`), NOT the current one, and floors
    // the result away from the wall so eyes never stick to ceilings.
    if !env.no_tile_collide {
        if npc.collide_x {
            // `:52285-52293`.
            npc.vx = npc.old_vx * -0.5;
            if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
                npc.vx = 2.0;
            }
            if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
                npc.vx = -2.0;
            }
        }
        if npc.collide_y {
            // `:52297-52305` — no direction gate on Y, both signs floor.
            npc.vy = npc.old_vy * -0.5;
            if npc.vy > 0.0 && npc.vy < 1.0 {
                npc.vy = 1.0;
            }
            if npc.vy < 0.0 && npc.vy > -1.0 {
                npc.vy = -1.0;
            }
        }
    }
    // `:52308-52325` — the dawn flee. The predicate reads the CURRENT
    // target's graveyard zone (before the else's TargetClosest re-pick).
    let graveyard = env
        .player_graveyards
        .get(npc.target as usize)
        .copied()
        .unwrap_or(false);
    // `Main.IsItDay()` — the remix fold reads world.day_time (module note).
    if despawn_discouraged(npc_type, npc.y, env.world.world_surface, env.world.day_time, graveyard) {
        // `EncourageDespawn(10)` (`NPC.cs:7249-7256`) — the port's
        // despawn-timer seam (`npc.rs:1855-1859`, the zombie wiring's
        // mapping of the C# timeLeft clamp).
        npc.encourage_despawn(10);
        npc.direction_y = -1;
        // `:52312-52320` — the C#'s own dead-store quirk, ported
        // VERBATIM: `if (vy > 0) direction = 1;` is immediately
        // overwritten by `direction = -1;`, then `if (vx > 0)
        // direction = 1;`. Net effect: facing = vx > 0 ? 1 : -1, but
        // the write ORDER is the source's (audit §2b).
        if npc.vy > 0.0 {
            npc.facing = 1;
        }
        npc.facing = -1;
        if npc.vx > 0.0 {
            npc.facing = 1;
        }
    } else {
        // `:52324` — TargetClosest (RNG-free, the S1-D-3p core).
        target_closest(npc, env.world, env.players, true, 2, env.ticker);
    }
    if pigron {
        // `:52326-52423` — the phasing machine (no NpcKind yet; ported
        // in full for the wiring round).
        pigron_phase(npc, npc_type, env, &mut out, w, h);
    } else if npc_type == THE_HUNGRY_II {
        // `:52425-52500` — The Hungry II's own arm (no NpcKind yet).
        hungry_ii(npc, env, rng, &mut out, w, h);
    } else if npc_type == WANDERING_EYE {
        // `:52502-52637` — the half-life enrage.
        wandering_eye_steer(npc);
    } else {
        // `:52639-52708` — the default arm (2/190-194/317/318).
        default_steer(npc);
    }
    // `:52710-52717` — the family dust gate. The C# `&&` short-circuits:
    // ONLY 2/133/190-194 draw the Next(40); the burst itself is the
    // documented skip (the request rides EyeOut).
    if (npc_type == DEMON_EYE
        || npc_type == WANDERING_EYE
        || npc_type == CATARACT_EYE
        || npc_type == SLEEPY_EYE
        || npc_type == DIALATED_EYE
        || npc_type == GREEN_EYE
        || npc_type == PURPLE_EYE)
        && rng.next_max(40) == 0
    {
        out.dust = Some(family_dust(npc, w, h));
    }
    // `:52718-52730` — the wet tail (every type EXCEPT the Pigrons):
    // falling drag ×0.95, the −0.5/tick buoyancy, the −4 rise cap, and
    // one more TargetClosest.
    if npc.wet && !pigron {
        if npc.vy > 0.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.5;
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
        target_closest(npc, env.world, env.players, true, 2, env.ticker);
    }
    out
}

/// The default steering arm (`NPC.cs:52639-52708`): axis-separated,
/// direction-driven — NO normalization, NO sqrt, NO bob. `num2 = 4f`,
/// `num3 = 1.5f`, both `*= 1f + (1f - scale)` (the `2 − scale` factor).
/// X: −0.1 toward `direction`, an EXTRA −0.1 while over the opposite
/// cap, +0.05 while still moving the wrong way, hard clamp ±num2.
/// Y: −0.04 toward `directionY`, extra −0.05 over-cap, +0.03 wrong-sign,
/// clamp ±num3. The `0.12`/`3.0` pair appears NOWHERE here.
fn default_steer(npc: &mut Npc) {
    // `:52641-52644` — the multiply order is the C#'s (base, then the
    // scale factor), not a folded `4 * (2 - scale)`.
    let mut num2 = 4.0f32;
    let mut num3 = 1.5f32;
    num2 *= 1.0 + (1.0 - npc.scale);
    num3 *= 1.0 + (1.0 - npc.scale);
    // `:52645-52660` — leftward.
    if npc.facing == -1 && npc.vx > 0.0 - num2 {
        npc.vx -= 0.1;
        if npc.vx > num2 {
            npc.vx -= 0.1;
        } else if npc.vx > 0.0 {
            npc.vx += 0.05;
        }
        if npc.vx < 0.0 - num2 {
            npc.vx = 0.0 - num2;
        }
    } else if npc.facing == 1 && npc.vx < num2 {
        // `:52661-52676` — rightward (the mirror).
        npc.vx += 0.1;
        if npc.vx < 0.0 - num2 {
            npc.vx += 0.1;
        } else if npc.vx < 0.0 {
            npc.vx -= 0.05;
        }
        if npc.vx > num2 {
            npc.vx = num2;
        }
    }
    // `:52677-52692` — upward.
    if npc.direction_y == -1 && npc.vy > 0.0 - num3 {
        npc.vy -= 0.04;
        if npc.vy > num3 {
            npc.vy -= 0.05;
        } else if npc.vy > 0.0 {
            npc.vy += 0.03;
        }
        if npc.vy < 0.0 - num3 {
            npc.vy = 0.0 - num3;
        }
    } else if npc.direction_y == 1 && npc.vy < num3 {
        // `:52693-52708` — downward.
        npc.vy += 0.04;
        if npc.vy < 0.0 - num3 {
            npc.vy += 0.05;
        } else if npc.vy < 0.0 {
            npc.vy -= 0.03;
        }
        if npc.vy > num3 {
            npc.vy = num3;
        }
    }
}

/// The Wandering Eye's half-life arm (`NPC.cs:52502-52637`): under 50%
/// life (`(double)life < (double)lifeMax * 0.5` — the f64 compare) the
/// caps jump to 6 (X) / 4 (Y) with 0.1/0.1 accel and 0.1/0.1 over-cap
/// brakes; at or above half it runs 4/1.5 caps with 0.1/0.04 accel —
/// the default arm's numbers but with the WRONG-SIGN brakes still
/// +0.05 (X) / +0.03 (Y) and the over-cap X brake 0.1 (not 0.1+0.1
/// folded). Both halves keep the same if-else shape as the default.
fn wandering_eye_steer(npc: &mut Npc) {
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
        // `:52506-52537` — the enraged X (±6, 0.1 accel).
        if npc.facing == -1 && npc.vx > -6.0 {
            npc.vx -= 0.1;
            if npc.vx > 6.0 {
                npc.vx -= 0.1;
            } else if npc.vx > 0.0 {
                npc.vx += 0.05;
            }
            if npc.vx < -6.0 {
                npc.vx = -6.0;
            }
        } else if npc.facing == 1 && npc.vx < 6.0 {
            npc.vx += 0.1;
            if npc.vx < -6.0 {
                npc.vx += 0.1;
            } else if npc.vx < 0.0 {
                npc.vx -= 0.05;
            }
            if npc.vx > 6.0 {
                npc.vx = 6.0;
            }
        }
        // `:52538-52569` — the enraged Y (±4, 0.1 accel).
        if npc.direction_y == -1 && npc.vy > -4.0 {
            npc.vy -= 0.1;
            if npc.vy > 4.0 {
                npc.vy -= 0.1;
            } else if npc.vy > 0.0 {
                npc.vy += 0.05;
            }
            if npc.vy < -4.0 {
                npc.vy = -4.0;
            }
        } else if npc.direction_y == 1 && npc.vy < 4.0 {
            npc.vy += 0.1;
            if npc.vy < -4.0 {
                npc.vy += 0.1;
            } else if npc.vy < 0.0 {
                npc.vy -= 0.05;
            }
            if npc.vy > 4.0 {
                npc.vy = 4.0;
            }
        }
    } else {
        // `:52573-52604` — the healthy X (±4, 0.1 accel).
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
        // `:52605-52636` — the healthy Y (±1.5, 0.04 accel; the C#'s
        // `(double)` casts on the gate/over-cap compares).
        if npc.direction_y == -1 && (npc.vy as f64) > -1.5 {
            npc.vy -= 0.04;
            if (npc.vy as f64) > 1.5 {
                npc.vy -= 0.05;
            } else if npc.vy > 0.0 {
                npc.vy += 0.03;
            }
            if (npc.vy as f64) < -1.5 {
                npc.vy = -1.5;
            }
        } else if npc.direction_y == 1 && (npc.vy as f64) < 1.5 {
            npc.vy += 0.04;
            if (npc.vy as f64) < -1.5 {
                npc.vy += 0.05;
            } else if npc.vy < 0.0 {
                npc.vy -= 0.03;
            }
            if (npc.vy as f64) > 1.5 {
                npc.vy = 1.5;
            }
        }
    }
}

/// The Hungry II's arm (`NPC.cs:52425-52500`): a second TargetClosest,
/// the centre light (INTEGER halves + INTEGER /16), ±6 X caps with
/// 0.1/0.1/0.2 ramps, the ASYMMETRIC Y gates (`directionY == -1 &&
/// vy > -2.5` vs `directionY == 1 && vy < 1.5` — the fall gate is 1.5,
/// not 2.5, the C#'s own asymmetry) with 0.04 accel and ±2.5 clamps,
/// then its own `Next(40)` dust gate (`:52493`).
fn hungry_ii(npc: &mut Npc, env: &EyeEnv<'_>, rng: &mut UnifiedRandom, out: &mut EyeOut, w: f32, h: f32) {
    // `:52427`.
    target_closest(npc, env.world, env.players, true, 2, env.ticker);
    // `:52428` — `(int)(position.X + (float)(width / 2)) / 16`: the
    // half is INTEGER division, the sum truncates, then INTEGER /16.
    let lx = ((npc.x + (w as i32 / 2) as f32) as i32) / 16;
    let ly = ((npc.y + (h as i32 / 2) as f32) as i32) / 16;
    out.light = Some((lx, ly, [0.3, 0.2, 0.1]));
    // `:52429-52460` — X.
    if npc.facing == -1 && npc.vx > -6.0 {
        npc.vx -= 0.1;
        if npc.vx > 6.0 {
            npc.vx -= 0.1;
        } else if npc.vx > 0.0 {
            npc.vx -= 0.2;
        }
        if npc.vx < -6.0 {
            npc.vx = -6.0;
        }
    } else if npc.facing == 1 && npc.vx < 6.0 {
        npc.vx += 0.1;
        if npc.vx < -6.0 {
            npc.vx += 0.1;
        } else if npc.vx < 0.0 {
            npc.vx += 0.2;
        }
        if npc.vx > 6.0 {
            npc.vx = 6.0;
        }
    }
    // `:52461-52492` — Y (the asymmetric gates; `(double)` casts).
    if npc.direction_y == -1 && (npc.vy as f64) > -2.5 {
        npc.vy -= 0.04;
        if (npc.vy as f64) > 2.5 {
            npc.vy -= 0.05;
        } else if npc.vy > 0.0 {
            npc.vy -= 0.15;
        }
        if (npc.vy as f64) < -2.5 {
            npc.vy = -2.5;
        }
    } else if npc.direction_y == 1 && (npc.vy as f64) < 1.5 {
        npc.vy += 0.04;
        if (npc.vy as f64) < -2.5 {
            npc.vy += 0.05;
        } else if npc.vy < 0.0 {
            npc.vy += 0.15;
        }
        if (npc.vy as f64) > 2.5 {
            npc.vy = 2.5;
        }
    }
    // `:52493-52500` — the 116 dust gate (its own Next(40), inside the
    // branch; the family gate at :52710 excludes 116).
    if rng.next_max(40) == 0 {
        out.dust = Some(family_dust(npc, w, h));
    }
}

/// The Pigron phasing machine (`NPC.cs:52326-52423`): visible while the
/// target has line of sight (`Collision.CanHit`), otherwise `ai[0]`
/// counts up; at 300 unseen ticks `ai[1] = 1` — the phase: alpha 200,
/// `noTileCollide = true`, `wet = false`. Re-sighting while phased AND
/// not inside solid tiles resets both counters. The phase writes
/// `rotation = vy * 0.1 * direction`, re-targets, then steers ±4 X
/// (0.08/0.04/0.2) and ±2.5 Y (0.1/0.05/0.15) with the PLAYER-BOX
/// overlap gates (not just the sign).
fn pigron_phase(npc: &mut Npc, _npc_type: i32, env: &EyeEnv<'_>, out: &mut EyeOut, w: f32, h: f32) {
    let p = &env.players[npc.target as usize];
    // `:52328-52340` — the sight gate. `Collision.CanHit` rides the
    // port's Manhattan walk (`projectile::can_hit`, the style42
    // convention); `Collision.SolidCollision` rides
    // `Player::solid_collision` (`Collision.cs:2715`'s port).
    if crate::projectile::can_hit(env.world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32) {
        if npc.ai[1] > 0.0 && !crate::player::Player::solid_collision(env.world, npc.x, npc.y, w, h) {
            npc.ai[1] = 0.0;
            npc.ai[0] = 0.0;
            npc.net_update = true;
        }
    } else if npc.ai[1] == 0.0 {
        npc.ai[0] += 1.0;
    }
    // `:52341-52346` — the 300-tick phase trigger.
    if npc.ai[0] >= 300.0 {
        npc.ai[1] = 1.0;
        npc.ai[0] = 0.0;
        npc.net_update = true;
    }
    // `:52347-52357` — the phase state writes.
    if npc.ai[1] == 0.0 {
        npc.alpha = 0;
        out.no_tile_collide = Some(false);
    } else {
        npc.wet = false;
        npc.alpha = 200;
        out.no_tile_collide = Some(true);
    }
    // `:52358-52359` — the tilt + the second TargetClosest.
    npc.rotation = npc.vy * 0.1 * npc.facing as f32;
    target_closest(npc, env.world, env.players, true, 2, env.ticker);
    let p = &env.players[npc.target as usize];
    // `:52360-52391` — X: the gate also needs the boxes NOT overlapping
    // on the approach axis.
    if npc.facing == -1 && npc.vx > -4.0 && npc.x > p.x + p.width as f32 {
        npc.vx -= 0.08;
        if npc.vx > 4.0 {
            npc.vx -= 0.04;
        } else if npc.vx > 0.0 {
            npc.vx -= 0.2;
        }
        if npc.vx < -4.0 {
            npc.vx = -4.0;
        }
    } else if npc.facing == 1 && npc.vx < 4.0 && npc.x + w < p.x {
        npc.vx += 0.08;
        if npc.vx < -4.0 {
            npc.vx += 0.04;
        } else if npc.vx < 0.0 {
            npc.vx += 0.2;
        }
        if npc.vx > 4.0 {
            npc.vx = 4.0;
        }
    }
    // `:52392-52423` — Y (the `(double)` casts; ±2.5, 0.1 accel).
    if npc.direction_y == -1 && (npc.vy as f64) > -2.5 && npc.y > p.y + p.height as f32 {
        npc.vy -= 0.1;
        if (npc.vy as f64) > 2.5 {
            npc.vy -= 0.05;
        } else if npc.vy > 0.0 {
            npc.vy -= 0.15;
        }
        if (npc.vy as f64) < -2.5 {
            npc.vy = -2.5;
        }
    } else if npc.direction_y == 1 && (npc.vy as f64) < 2.5 && npc.y + h < p.y {
        npc.vy += 0.1;
        if (npc.vy as f64) < -2.5 {
            npc.vy += 0.05;
        } else if npc.vy < 0.0 {
            npc.vy += 0.15;
        }
        if (npc.vy as f64) > 2.5 {
            npc.vy = 2.5;
        }
    }
}

/// The shared dust burst arguments (`:52712-52716` / `:52495-52499`):
/// the `netOffset` bracket, the quarter-height offset, the INTEGER
/// `(int)(height * 0.5f)` box height, type 5, `(vx, 2f)` speeds, the
/// `×0.5`/`×0.1` post-spawn velocity scales.
fn family_dust(npc: &Npc, w: f32, h: f32) -> EyeDust {
    EyeDust {
        x: npc.x + npc.net_offset.x,
        y: npc.y + npc.net_offset.y + h * 0.25,
        width: w as i32,
        height: (h * 0.5) as i32,
        dust_type: 5,
        speed_x: npc.vx,
        speed_y: 2.0,
        vel_scale_x: 0.5,
        vel_scale_y: 0.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::npc_target::session_candidate;
    use crate::player::Player;
    use tr_core::TILE_PX;

    fn air() -> World {
        let mut w = World::new("t".into(), 200, 120);
        // The default world spawns at DAY with worldSurface = height/4
        // (= 30 tiles = 480 px); the movement tests place their eyes at
        // y >= 1000 px (below the line) but pin night anyway so the flee
        // gate can never fire by accident.
        w.day_time = false;
        w
    }

    fn eye(kind: NpcKind, x: f32, y: f32) -> Npc {
        Npc::new(kind, x, y, 1, 0.0)
    }

    fn player_at(w: &World, x: f32, y: f32) -> Player {
        let mut p = Player::spawn(w);
        p.x = x;
        p.y = y;
        p
    }

    /// Night below the surface: the flee gate never fires; the env's
    /// graveyard slice is all-false (the wiring fold).
    fn env<'a>(w: &'a World, cands: &'a [TargetCandidate], graves: &'a [bool]) -> EyeEnv<'a> {
        EyeEnv { world: w, players: cands, player_graveyards: graves, no_tile_collide: false, ticker: 0 }
    }

    /// The default arm from rest (`:52645-52708`): facing −1 draws
    /// vx to −0.1 (NOT −0.12), directionY 1 draws vy to +0.04. No bob:
    /// the position never moves inside the arm.
    #[test]
    fn s1_d_3_eye_the_default_accel_is_axis_separated_not_normalized() {
        let mut n = eye(NpcKind::DemonEye, 1000.0, 1000.0);
        n.facing = -1;
        n.direction_y = 1;
        default_steer(&mut n);
        assert_eq!(n.vx, -0.1, "the X accel is the flat 0.1, not 0.12");
        assert_eq!(n.vy, 0.04, "the Y accel is 0.04");
        assert_eq!((n.x, n.y), (1000.0, 1000.0), "no position bob in the arm");
    }

    /// The over-cap extra brake and the wrong-sign brake
    /// (`:52647-52655`): vx = 5 (> num2 = 4) with facing −1 loses
    /// 0.1 + 0.1 = 0.2; vx = 1 (wrong sign, under cap) loses 0.1 but
    /// gains 0.05 back.
    #[test]
    fn s1_d_3_eye_the_over_cap_and_wrong_sign_brakes() {
        let mut n = eye(NpcKind::DemonEye, 0.0, 0.0);
        n.facing = -1;
        n.vx = 5.0;
        default_steer(&mut n);
        assert!((n.vx - 4.8).abs() < 1e-5, "over-cap: 5 - 0.1 - 0.1 = 4.8, got {}", n.vx);
        let mut m = eye(NpcKind::DemonEye, 0.0, 0.0);
        m.facing = -1;
        m.vx = 1.0;
        default_steer(&mut m);
        assert!((m.vx - 0.95).abs() < 1e-5, "wrong-sign: 1 - 0.1 + 0.05 = 0.95, got {}", m.vx);
    }

    /// The scale factor (`:52641-52644`): scale 0.5 doubles the caps
    /// to 6 / 2.25 (`4 * (1 + (1 - 0.5))`); the clamp fires at −6.
    #[test]
    fn s1_d_3_eye_the_caps_take_the_two_minus_scale_factor() {
        let mut n = eye(NpcKind::DemonEye, 0.0, 0.0);
        n.scale = 0.5;
        n.facing = -1;
        n.vx = -5.95;
        default_steer(&mut n);
        assert_eq!(n.vx, -6.0, "the 0.5-scale cap is 6, clamped");
        let mut m = eye(NpcKind::DemonEye, 0.0, 0.0);
        m.scale = 0.5;
        m.direction_y = 1;
        m.vy = 2.24;
        default_steer(&mut m);
        assert_eq!(m.vy, 2.25, "the 0.5-scale Y cap is 2.25");
    }

    /// The bounce kicks read OLD velocity (`:52285-52305`): old_vx −3
    /// with facing −1 kicks to +1.5 → floored to 2; old_vy −1 kicks to
    /// +0.5 → floored to 1; a zero old velocity is NOT floored (0 is
    /// neither > 0 nor < 0).
    #[test]
    fn s1_d_3_eye_the_bounce_kicks_use_old_velocity_with_floors() {
        let w = air();
        let graves = [false];
        let mut rng = UnifiedRandom::new(1);
        // The target far LEFT so TargetClosest keeps facing −1.
        let p2 = player_at(&w, 100.0, 3000.0);
        let c2 = [session_candidate(&p2)];
        let mut n2 = eye(NpcKind::DemonEye, 1000.0, 3000.0);
        n2.facing = -1;
        n2.collide_x = true;
        n2.collide_y = true;
        n2.old_vx = -3.0;
        n2.old_vy = -1.0;
        n2.vx = 99.0; // the CURRENT velocity must be ignored by the kick
        n2.vy = 99.0;
        floating_eye_ai(&mut n2, DEMON_EYE, &env(&w, &c2, &graves), &mut rng);
        // Kick: vx = old(-3) * -0.5 = 1.5 → floor 2 (facing −1, 0 < vx < 2);
        // then the default steer: −0.1, then the wrong-sign +0.05 → 1.95.
        assert!((n2.vx - 1.95).abs() < 1e-5, "kick 2 then -0.1 +0.05, got {}", n2.vx);
        // Kick: vy = old(-1) * -0.5 = 0.5 → floor 1; directionY from the
        // target (same row → 1): steer +0.04 → 1.04.
        assert!((n2.vy - 1.04).abs() < 1e-5, "kick 1 then steer +0.04, got {}", n2.vy);
        // A zero old velocity is NOT floored (0 is neither > 0 nor < 0).
        let mut z = eye(NpcKind::DemonEye, 1000.0, 3000.0);
        z.facing = -1;
        z.collide_x = true;
        z.old_vx = 0.0;
        z.vx = 99.0;
        floating_eye_ai(&mut z, DEMON_EYE, &env(&w, &c2, &graves), &mut rng);
        assert!((z.vx - -0.1).abs() < 1e-5, "kick 0 (no floor), steer -0.1, got {}", z.vx);
    }

    /// The dawn flee (`:52308-52321` + `:52752-52763`): day + at/above
    /// the surface + no graveyard → EncourageDespawn(10), directionY −1,
    /// and the dead-store quirk's net facing (vx > 0 → 1, else −1).
    #[test]
    fn s1_d_3_eye_the_dawn_flee_encourages_despawn_and_climbs() {
        let mut w = air();
        w.day_time = true;
        w.world_surface = 80.0;
        let p = player_at(&w, 1000.0, 1000.0);
        let cands = [session_candidate(&p)];
        let graves = [false];
        let mut n = eye(NpcKind::DemonEye, 900.0, 80.0 * TILE_PX); // exactly ON the line: <= fires
        n.vx = 2.0;
        n.vy = 2.0;
        let mut rng = UnifiedRandom::new(1);
        floating_eye_ai(&mut n, DEMON_EYE, &env(&w, &cands, &graves), &mut rng);
        assert_eq!(n.despawn_timer, 10, "EncourageDespawn(10)");
        assert_eq!(n.direction_y, -1);
        assert_eq!(n.facing, 1, "the quirk's last write: vx > 0 → 1");
        // vx < 0 → the −1 store survives (the steer then keeps facing −1:
        // the player's centre 1010 < the eye's 915).
        let mut m = eye(NpcKind::DemonEye, 900.0, 80.0 * TILE_PX);
        m.vx = -2.0;
        m.vy = 2.0;
        floating_eye_ai(&mut m, DEMON_EYE, &env(&w, &cands, &graves), &mut rng);
        assert_eq!(m.facing, -1, "vy > 0's direction = 1 is the dead store");
        // Below the surface: no flee (TargetClosest ran instead).
        let mut b = eye(NpcKind::DemonEye, 900.0, 80.0 * TILE_PX + 1.0);
        floating_eye_ai(&mut b, DEMON_EYE, &env(&w, &cands, &graves), &mut rng);
        assert_eq!(b.despawn_timer, -1, "below the line the gate is off");
    }

    /// The predicate's terms (`:52752-52763`): graveyard cancels, night
    /// cancels, the type list (2/133/190-194/317/318 in; 116/170 out).
    #[test]
    fn s1_d_3_eye_the_discouraged_predicate_terms_and_type_list() {
        assert!(despawn_discouraged(2, 100.0, 80.0, true, false));
        assert!(!despawn_discouraged(2, 100.0, 80.0, true, true), "graveyard");
        assert!(!despawn_discouraged(2, 100.0, 80.0, false, false), "night");
        assert!(!despawn_discouraged(2, 80.0 * 16.0 + 1.0, 80.0, true, false), "below");
        for t in [133, 190, 191, 192, 193, 194, 317, 318] {
            assert!(despawn_discouraged(t, 100.0, 80.0, true, false), "type {t}");
        }
        for t in [116, 170, 171, 180, 3] {
            assert!(!despawn_discouraged(t, 100.0, 80.0, true, false), "type {t} out");
        }
    }

    /// The Wandering Eye's half-life enrage (`:52502-52637`): the f64
    /// compare is STRICT (< half): at exactly half the healthy arm runs
    /// (0.04 Y accel); one hp under it the enraged 0.1 fires.
    #[test]
    fn s1_d_3_eye_the_wandering_eye_enrages_under_half_life() {
        let mut n = eye(NpcKind::WanderingEye, 0.0, 0.0);
        n.max_hp = 500;
        n.hp = 250; // exactly half → healthy
        n.direction_y = 1;
        wandering_eye_steer(&mut n);
        assert_eq!(n.vy, 0.04, "at half the healthy 0.04 accel runs");
        n.hp = 249;
        n.vy = 0.0;
        wandering_eye_steer(&mut n);
        assert_eq!(n.vy, 0.1, "under half the enraged 0.1 accel runs");
        // The enraged Y clamp is 4 (not 1.5).
        n.vy = 3.95;
        wandering_eye_steer(&mut n);
        assert_eq!(n.vy, 4.0, "the enraged cap is 4");
        // The enraged X cap is 6.
        n.facing = 1;
        n.vx = 5.95;
        wandering_eye_steer(&mut n);
        assert_eq!(n.vx, 6.0);
    }

    /// The wet tail (`:52718-52730`): falling drag then buoyancy then
    /// the −4 cap; the Pigrons are exempt.
    #[test]
    fn s1_d_3_eye_the_wet_tail_drags_buoys_and_caps() {
        let w = air();
        let p = player_at(&w, 2000.0, 3000.0);
        let cands = [session_candidate(&p)];
        let graves = [false];
        let mut n = eye(NpcKind::DemonEye, 1000.0, 3000.0);
        n.wet = true;
        n.vy = 2.0;
        let mut rng = UnifiedRandom::new(1);
        floating_eye_ai(&mut n, DEMON_EYE, &env(&w, &cands, &graves), &mut rng);
        // vy = 2.0 is ABOVE the 1.5 downward gate: no Y steer this tick.
        // The tail: falling drag 2.0 * 0.95, then buoyancy −0.5 = 1.4.
        assert!((n.vy - (2.0f32 * 0.95 - 0.5)).abs() < 1e-5, "got {}", n.vy);
        // The rise cap: a strongly rising eye clamps at −4. TargetClosest
        // sets directionY 1 (same-row player), so the steer adds +0.04
        // then the wrong-sign over-cap +0.05: −3.8 → −3.71; the tail's
        // −0.5 → −4.21 → clamped −4.
        let mut m = eye(NpcKind::DemonEye, 1000.0, 3000.0);
        m.wet = true;
        m.vy = -3.8;
        m.direction_y = -1;
        floating_eye_ai(&mut m, DEMON_EYE, &env(&w, &cands, &graves), &mut rng);
        assert_eq!(m.vy, -4.0, "-3.71 - 0.5 = -4.21 → clamped -4");
    }

    /// RNG by construction: the ported eye kinds draw EXACTLY one
    /// Next(40) per tick (the family dust gate) — no other draw anywhere
    /// in the arm (the flee gate, TargetClosest and the steering are
    /// RNG-free). Mirrored-oracle stream-position witness.
    #[test]
    fn s1_d_3_eye_the_family_kinds_draw_exactly_one_next_forty() {
        let w = air();
        let p = player_at(&w, 2000.0, 3000.0);
        let cands = [session_candidate(&p)];
        let graves = [false];
        for (kind, ty) in [
            (NpcKind::DemonEye, DEMON_EYE),
            (NpcKind::WanderingEye, WANDERING_EYE),
            (NpcKind::CataractEye, CATARACT_EYE),
            (NpcKind::SleepyEye, SLEEPY_EYE),
            (NpcKind::DialatedEye, DIALATED_EYE),
            (NpcKind::GreenEye, GREEN_EYE),
            (NpcKind::PurpleEye, PURPLE_EYE),
        ] {
            let mut n = eye(kind, 1000.0, 3000.0);
            let mut rng = UnifiedRandom::new(42);
            let mut oracle = rng.clone();
            let gate = oracle.next_max(40) == 0;
            let out = floating_eye_ai(&mut n, ty, &env(&w, &cands, &graves), &mut rng);
            assert_eq!(rng.peek(), oracle.peek(), "type {ty}: exactly one Next(40)");
            assert_eq!(out.dust.is_some(), gate, "the burst rides the gate");
        }
    }

    /// The dawn-flee tick draws the SAME single Next(40) — the flee
    /// gate and EncourageDespawn are RNG-free (the audit's dawn-flee
    /// RNG question: the gates draw nothing).
    #[test]
    fn s1_d_3_eye_the_dawn_flee_tick_still_draws_only_the_dust_gate() {
        let mut w = air();
        w.day_time = true;
        w.world_surface = 80.0;
        let p = player_at(&w, 1000.0, 1000.0);
        let cands = [session_candidate(&p)];
        let graves = [false];
        let mut n = eye(NpcKind::DemonEye, 900.0, 500.0);
        let mut rng = UnifiedRandom::new(7);
        let mut oracle = rng.clone();
        let _ = oracle.next_max(40);
        floating_eye_ai(&mut n, DEMON_EYE, &env(&w, &cands, &graves), &mut rng);
        assert_eq!(n.despawn_timer, 10, "the flee fired");
        assert_eq!(rng.peek(), oracle.peek(), "no extra draw on the flee path");
    }

    /// The Pigron draws Next(1000) FIRST and NO Next(40) (the family
    /// gate's type list excludes 170/171/180); the moan cue rides the
    /// gate result.
    #[test]
    fn s1_d_3_eye_the_pigron_draws_one_next_thousand_and_no_forty() {
        let w = air();
        let p = player_at(&w, 2000.0, 3000.0);
        let cands = [session_candidate(&p)];
        let graves = [false];
        let mut n = eye(NpcKind::DemonEye, 1000.0, 3000.0); // vehicle kind; the machine keys on npc_type
        n.size_override = Some((44.0, 36.0)); // the Pigron box (`NPC.cs:11129-11130`)
        let mut rng = UnifiedRandom::new(42);
        let mut oracle = rng.clone();
        let moan = oracle.next_max(1000) == 0;
        let out = floating_eye_ai(&mut n, PIGRON_CORRUPTION, &env(&w, &cands, &graves), &mut rng);
        assert_eq!(rng.peek(), oracle.peek(), "exactly one Next(1000), no Next(40)");
        assert_eq!(out.pigron_moan, moan);
        assert!(out.dust.is_none(), "the family dust gate excludes the Pigrons");
    }

    /// The Pigron phase machine (`:52337-52357`): 300 unseen ticks flip
    /// ai[1] to 1 — alpha 200, noTileCollide true, wet cleared; the
    /// re-sight (CanHit true, not solid) resets both counters.
    #[test]
    fn s1_d_3_eye_the_pigron_phases_after_300_unseen_ticks() {
        let mut w = air();
        // A solid wall between the NPC and the player blocks CanHit.
        for y in 40..100 {
            let i = w.index(80, y);
            w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let p = player_at(&w, 100.0 * TILE_PX, 60.0 * TILE_PX);
        let cands = [session_candidate(&p)];
        let graves = [false];
        let mut n = eye(NpcKind::DemonEye, 60.0 * TILE_PX, 60.0 * TILE_PX);
        n.size_override = Some((44.0, 36.0));
        n.ai[0] = 299.0;
        n.wet = true;
        let mut rng = UnifiedRandom::new(1);
        let out = floating_eye_ai(&mut n, PIGRON_HALLOW, &env(&w, &cands, &graves), &mut rng);
        assert_eq!(n.ai[1], 1.0, "the 300th unseen tick phases");
        assert_eq!(n.ai[0], 0.0);
        assert_eq!(n.alpha, 200);
        assert!(!n.wet, "the phase clears wet");
        assert_eq!(out.no_tile_collide, Some(true));
        assert!(n.net_update);
        // Re-sight: open air, phased, not inside solids → both reset.
        let w2 = air();
        let p2 = player_at(&w2, 70.0 * TILE_PX, 60.0 * TILE_PX);
        let c2 = [session_candidate(&p2)];
        let mut m = eye(NpcKind::DemonEye, 60.0 * TILE_PX, 60.0 * TILE_PX);
        m.size_override = Some((44.0, 36.0));
        m.ai = [123.0, 1.0, 0.0, 0.0];
        let out2 = floating_eye_ai(&mut m, PIGRON_CRIMSON, &env(&w2, &c2, &graves), &mut rng);
        assert_eq!(m.ai[1], 0.0, "the re-sight resets the phase");
        assert_eq!(m.ai[0], 0.0);
        assert_eq!(m.alpha, 0);
        assert_eq!(out2.no_tile_collide, Some(false));
    }

    /// The Hungry II arm (`:52425-52500`): the integer-division light
    /// tile + (0.3, 0.2, 0.1), the asymmetric fall gate (vy < 1.5, not
    /// 2.5), one Next(40) and NO family-gate draw.
    #[test]
    fn s1_d_3_eye_the_hungry_ii_light_gates_and_single_draw() {
        let w = air();
        let p = player_at(&w, 2000.0, 3000.0);
        let cands = [session_candidate(&p)];
        let graves = [false];
        let mut n = eye(NpcKind::DemonEye, 160.0, 176.0);
        n.size_override = Some((30.0, 32.0)); // `NPC.cs:10394-10395`
        // The arm's own TargetClosest (:52427) re-aims: the player below
        // and right → facing 1, directionY 1.
        n.vy = 1.6; // above the fall gate's 1.5: NO Y steer this tick
        let mut rng = UnifiedRandom::new(42);
        let mut oracle = rng.clone();
        let gate = oracle.next_max(40) == 0;
        let out = floating_eye_ai(&mut n, THE_HUNGRY_II, &env(&w, &cands, &graves), &mut rng);
        // (int)(160 + 30/2) / 16 = 175/16 = 10; (int)(176 + 32/2)/16 = 12.
        assert_eq!(out.light, Some((10, 12, [0.3, 0.2, 0.1])));
        assert_eq!(n.vy, 1.6, "the fall gate is vy < 1.5 (directionY 1 from the re-aim) — 1.6 does not steer");
        assert_eq!(rng.peek(), oracle.peek(), "exactly one Next(40) (the 116 gate)");
        assert_eq!(out.dust.is_some(), gate);
    }

    /// The bounce gate is `!noTileCollide` (`:52281`): a phased Pigron
    /// (env flag true) keeps its velocity through a collision tick.
    #[test]
    fn s1_d_3_eye_the_no_tile_collide_flag_skips_the_kicks() {
        let w = air();
        let p = player_at(&w, 2000.0, 3000.0);
        let cands = [session_candidate(&p)];
        let graves = [false];
        let mut n = eye(NpcKind::DemonEye, 1000.0, 3000.0);
        n.collide_x = true;
        n.collide_y = true;
        n.old_vx = -4.0;
        n.old_vy = -4.0;
        n.vx = 1.0;
        n.vy = 1.0;
        let e = EyeEnv { no_tile_collide: true, ..env(&w, &cands, &graves) };
        let mut rng = UnifiedRandom::new(1);
        floating_eye_ai(&mut n, DEMON_EYE, &e, &mut rng);
        // No kick: vx stayed 1.0 before the steer (facing 1 from the
        // target → +0.1); vy 1.0 + 0.04.
        assert!((n.vx - 1.1).abs() < 1e-5, "no kick, just the steer: {}", n.vx);
        assert!((n.vy - 1.04).abs() < 1e-5, "no kick, just the steer: {}", n.vy);
    }

    /// The dust request carries the C#'s exact NewDust arguments
    /// (`:52712-52716`): the netOffset bracket, the quarter-height
    /// offset, the INTEGER half-height box, type 5, (vx, 2) speeds and
    /// the ×0.5/×0.1 post scales.
    #[test]
    fn s1_d_3_eye_the_dust_request_matches_the_new_dust_arguments() {
        let n = eye(NpcKind::DemonEye, 100.0, 200.0);
        let d = family_dust(&n, 30.0, 32.0);
        assert_eq!((d.x, d.y), (100.0, 208.0), "y + 32 * 0.25");
        assert_eq!((d.width, d.height), (30, 16));
        assert_eq!(d.dust_type, 5);
        assert_eq!((d.speed_x, d.speed_y), (0.0, 2.0));
        assert_eq!((d.vel_scale_x, d.vel_scale_y), (0.5, 0.1));
    }
}
