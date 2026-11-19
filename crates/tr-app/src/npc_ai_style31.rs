//! The Spazmatism's aiStyle-31 machine (S1-D-3 aiStyle-31) — `NPC.AI()`'s
//! `else if (this.aiStyle == 31)` arm (`NPC.cs:27015-27625`). Type 126
//! (Spazmatism) is the style's ONLY user (`SetDefaults` case 126,
//! `NPC.cs:10539-10555`: 100x110, aiStyle 31, def 10, dmg 50, 23000 HP,
//! kb 0, noGravity + noTileCollide, boss, value 120000).
//!
//! The machine: the invalid-target retarget (`:27017-27020`), the
//! bottom-anchored aim `num438` with the ±6.283 wrap and the 0.15
//! shortest-path turn with the twice-applied snap band (`:27022-27073`),
//! the every-tick `Next(5)` dust gate (`:27074`), the twin-alive despawn
//! discouragement (`:27080-27089`), the day/dead flee (`:27100-27105`),
//! phase 0's hover/shot → dash → decay cycle (`:27106-27324`), the 40%
//! hp hand-off into the spin-up/spin-down (`:27316-27323`,
//! `:27326-27386`) and phase 3's approach/flamethrower/dash cycle with
//! the 1.5x damage / +18 defense writes (`:27387-27624`).
//!
//! RNG order on the caller's `UpdateNPCs` stream (`Main.cs:17767`), all
//! draws preserved even where the visual is a documented skip:
//! 1. `Next(5)` dust gate — EVERY tick, before the day gate (`:27074`);
//! 2. phase-0 shot jitter: `Next(-40, 41) * 0.05` X then Y
//!    (`:27231-27232`);
//! 3. spin transition burst, only when NOT entering `ai[0] == 3`:
//!    2 x (3 gores x 2 draws `Next(-30, 31) * 0.2`) (`:27363-27365`)
//!    then 20 dusts x 2 draws (`:27369`) — 52 draws;
//! 4. per-tick spin dust: 2 draws `Next(-30, 31) * 0.2` (`:27374`),
//!    also on the transition tick, AFTER the burst;
//! 5. phase-3 laser jitter: `Next(-40, 41) * 0.01` — Y FIRST (`:27525`)
//!    then X (`:27526`), reversed vs the phase-0 shot.
//!
//! Deliberate stand-ins (house pattern, recorded in ANALYSIS):
//! - `Main.expertMode` / `Main.getGoodWorld` ride [`Style31Env`] — the
//!   port is single-difficulty classic, so the dispatch passes
//!   false/false and every expert/getGood branch folds to its classic
//!   value; the branch maths stay ported and witness-tested.
//! - `NPC.IsMechQueenUp` (`NPC.cs:6868-6883`) needs an active type 127
//!   (Mech Queen) — the port has no such kind (the S1-D-3j precedent),
//!   so the dispatch passes `mech_queen: None`; the orbit-anchor /
//!   reflect / 14-speed / 1200-tick branches stay ported behind the
//!   [`MechQueenAnchor`] seam.
//! - The twin-alive loop (`:27082-27088`) scans the other active
//!   125/126 slots; the pure fn takes their `timeLeft`s in slot order
//!   ([`Style31Env::twin_time_lefts`]). `Npc::update_rng` cannot see
//!   its siblings, so the dispatch passes `&[]` until the sim pass
//!   feeds the list (wiring deferred, the S1-F-9b precedent).
//! - `DiscourageDespawn` (`NPC.cs:7258-7265`) raises `timeLeft`; the
//!   port's `despawn_timer` carries a -1 "no timer" sentinel
//!   (`npc.rs`'s `encourage_despawn` convention), so the gate and the
//!   raise only run on an ACTIVE timer (>= 0).
//! - `Main.netMode != 1` is constant-true (the port is single-player,
//!   netMode 0): the shot blocks and the discourage loop always run.
//! - The dust/gore visuals and the phase sounds (3 / 15) are
//!   SKIPPED — no dust/gore layer — but every `Main.rand` draw
//!   they carry is kept in order (see the RNG list above). The
//!   flamethrower's Item34 loop (`:27493`) queues its play (S1-E-21g).
//! - `reflectsProjectiles = false` (`:27099`) / `= true` under the
//!   Mech Queen (`:27330`): the port has no NPC projectile-reflect
//!   flag; with no Mech Queen the C# value is constant-false, the
//!   port's default — a no-op, dropped.
//! - `HitSound = SoundID.NPCHit4` (`:27387`): the port's hit cues ride
//!   the static per-kind table (`npc/data.rs::hit_sound`, S1-E-21a);
//!   the dynamic phase-3 write is a documented skip.
//! - `GetAttackDamage_ForProjectiles(25, 22)` / `(30, 27)`
//!   (`NPC.cs:7052-7055`, `Utils.Remap` over the difficulty): classic
//!   25 / 30, expert 22 / 27 — the env's `expert` bool picks the
//!   endpoint (no master difficulty in the port).
//! - `num448 = num447` (`:27135`) and the queen tail's first
//!   `num462`/`num463` pair (`:27545-27546`, overwritten at
//!   `:27547-27548`) are dead C# stores — dropped, no behaviour.

use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// The Mech Queen anchor the `IsMechQueenUp` branches read: the type-127
/// body's `GetMechQueenCenter()` (`NPC.cs:50954-50961` — `Center +
/// (0, -14)`, the caller applies the offset) and its `velocity.X`
/// (`:27097`'s `nPC2.velocity.X * 0.025f` orbit tilt).
#[derive(Clone, Copy, Debug)]
pub struct MechQueenAnchor {
    pub center_x: f32,
    pub center_y: f32,
    pub vx: f32,
}

/// The world/difficulty inputs the pure machine cannot read itself.
pub struct Style31Env<'a> {
    /// The OTHER active type-125/126 NPCs' `timeLeft`s in slot order
    /// (`:27082-27088`'s `num441 != whoAmI` scan). Empty = no twin.
    pub twin_time_lefts: &'a [i32],
    /// `NPC.IsMechQueenUp` (`:6868-6883`) — None = no Mech Queen.
    pub mech_queen: Option<MechQueenAnchor>,
    /// `Main.expertMode` — the port is classic, the dispatch passes false.
    pub expert: bool,
    /// `Main.getGoodWorld` — the port has no secret world seed, false.
    pub get_good: bool,
}

impl Style31Env<'static> {
    /// The dispatch's stock env: classic single-player, no twin list
    /// (the sim-pass wiring is deferred), no Mech Queen.
    pub const CLASSIC_SOLO: Style31Env<'static> = Style31Env {
        twin_time_lefts: &[],
        mech_queen: None,
        expert: false,
        get_good: false,
    };
}

/// `NPC.TargetClosest()` (`NPC.cs:77652-77666`) through the shared
/// selection (`npc_target.rs`) — the single session player as slot 0,
/// the stock aggro/no-aggro/tank values (the S1-D-3p convention).
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
    crate::npc_target::target_closest(npc, world, &[cand], true, 31, 0);
}

/// `NPC.ClearTarget()` (`NPC.cs:42924-42927`): `target = 255`.
fn clear_target(npc: &mut Npc) {
    npc.target = 255;
}

/// The aim `num438` (`NPC.cs:27022-27030`). The C# calls
/// `Math.Atan2(x: A, y: B)` with NAMED arguments — the value bound to
/// `y` (the NPC's bottom − 59 minus the player's centre Y) is the
/// atan2 numerator; the halves are INTEGER divisions (`width / 2`,
/// `height / 2`).
fn aim_rotation(npc: &Npc, player: &Player) -> f32 {
    let (w, h) = npc.size();
    let a = npc.x + (w as i32 / 2) as f32 - player.x - (crate::player::WIDTH as i32 / 2) as f32;
    let b = npc.y + h - 59.0 - player.y - (player.height as i32 / 2) as f32;
    let mut num438 = b.atan2(a) + 1.57;
    // `:27023-27030` — the ±6.283 wrap (the C#'s truncated 2π).
    if num438 < 0.0 {
        num438 += 6.283;
    } else if (num438 as f64) > 6.283 {
        num438 -= 6.283;
    }
    num438
}

/// One axis of the C#'s approach (`:27152-27183`, `:27440-27471` — the
/// two blocks are identical): step `accel` toward the normalized
/// target, with the unconditional double-add on a sign reversal.
fn approach_axis(v: &mut f32, target: f32, accel: f32) {
    if *v < target {
        *v += accel;
        if *v < 0.0 && target > 0.0 {
            *v += accel;
        }
    } else if *v > target {
        *v -= accel;
        if *v > 0.0 && target < 0.0 {
            *v -= accel;
        }
    }
}

/// The ±0.1 dead-band clamp (`:27286-27293`, `:27377-27384`,
/// `:27595-27602` — the C# compares in double).
fn clamp_dead_band(v: &mut f32) {
    if (*v as f64) > -0.1 && (*v as f64) < 0.1 {
        *v = 0.0;
    }
}

/// One fixed step of the Spazmatism (type 126) through the aiStyle-31
/// arm (`NPC.cs:27015-27625`), ending with the `noTileCollide`
/// integration (`NPC.cs:90494-90498` — `position += velocity`, no tile
/// interaction; the `oldVelocity` capture + collide-flag reset ride the
/// `UpdateCollision` tail convention, `:92538-92540`/`:92605-92607`).
/// The caller's `rng` is the `UpdateNPCs` stream; the draw order is the
/// module header's list.
pub fn spazmatism_ai(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    env: &Style31Env,
) {
    debug_assert_eq!(npc.kind, NpcKind::Spazmatism);
    ai_body(npc, world, player, rng, env);
    // The C#'s `return`s leave `AI()` only — `NPC.Update`'s
    // `noTileCollide` integration (`:90494-90498`) still runs.
    npc.old_vx = npc.vx;
    npc.old_vy = npc.vy;
    npc.collide_x = false;
    npc.collide_y = false;
    npc.x += npc.vx;
    npc.y += npc.vy;
}

/// The arm body proper — every `return` mirrors the C#'s `return` out
/// of `AI_031`'s block (the integration tail above still runs).
fn ai_body(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    env: &Style31Env,
) {
    let (w, h) = npc.size();
    // `:27017-27020` — the invalid-target retarget.
    if npc.target < 0 || npc.target == 255 || player.dead {
        target_closest(npc, world, player);
    }
    // `:27021` — `dead3` (the port's single player is slot 0; the
    // `!active` half of the retarget gate is constant-false here).
    let dead3 = player.dead;
    // `:27022-27030` — the aim.
    let num438 = aim_rotation(npc, player);
    // `:27031-27035` — the turn rate (the Mech Queen's phase-3 hover
    // quarters it).
    let mut num439 = 0.15f32;
    if env.mech_queen.is_some() && npc.ai[0] == 3.0 && npc.ai[1] == 0.0 {
        num439 *= 0.25;
    }
    // `:27036-27057` — the shortest-path turn (the 3.1415 half-turn).
    if npc.rotation < num438 {
        if ((num438 - npc.rotation) as f64) > 3.1415 {
            npc.rotation -= num439;
        } else {
            npc.rotation += num439;
        }
    } else if npc.rotation > num438 {
        if ((npc.rotation - num438) as f64) > 3.1415 {
            npc.rotation += num439;
        } else {
            npc.rotation -= num439;
        }
    }
    // `:27058-27061` — the first snap band.
    if npc.rotation > num438 - num439 && npc.rotation < num438 + num439 {
        npc.rotation = num438;
    }
    // `:27062-27069` — the rotation wrap.
    if npc.rotation < 0.0 {
        npc.rotation += 6.283;
    } else if (npc.rotation as f64) > 6.283 {
        npc.rotation -= 6.283;
    }
    // `:27070-27073` — the second snap band (after the wrap).
    if npc.rotation > num438 - num439 && npc.rotation < num438 + num439 {
        npc.rotation = num438;
    }
    // `:27074-27079` — the dust gate: the draw runs EVERY tick, before
    // the day gate; the dust itself is a documented skip.
    let _dust_gate = rng.next_max(5) == 0;
    // `:27080-27089` — the twin-alive despawn discouragement: at night,
    // alive, with the timer under 10, every other active 125/126's
    // `timeLeft - 1` raises ours (`DiscourageDespawn`, `:7258-7265`).
    // The port's -1 sentinel keeps the gate/raise on active timers
    // only (the module note). netMode != 1 is constant-true.
    if !world.day_time && !dead3 && npc.despawn_timer >= 0 && npc.despawn_timer < 10 {
        for &twin_time_left in env.twin_time_lefts {
            let t = twin_time_left - 1;
            if npc.despawn_timer < t {
                npc.despawn_timer = t;
            }
        }
    }
    // `:27090-27098` — the Mech Queen orbit anchor `vector42`:
    // `Utils.RotatedBy(queen.vx * 0.025, queenCenter + (150, -250) *
    // 0.75, queenCenter)` (`Utils.cs:1675-1684` — float cos/sin of the
    // double radians, result = center + rotated offset).
    let mut vector42 = (0.0f32, 0.0f32);
    if let Some(q) = env.mech_queen {
        let radians = q.vx * 0.025;
        let c = (radians as f64).cos() as f32;
        let s = (radians as f64).sin() as f32;
        let ox = 150.0 * 0.75;
        let oy = -250.0 * 0.75;
        vector42 = (q.center_x + ox * c - oy * s, q.center_y + ox * s + oy * c);
    }
    // `:27099` — `reflectsProjectiles = false`: constant-false without
    // the Mech Queen, the port's default (the module note).
    // `:27100-27105` — the day/dead flee: rise 0.04/tick, arm the
    // 10-tick despawn, leave the machine (the integration still runs).
    if world.day_time || dead3 {
        npc.vy -= 0.04;
        npc.encourage_despawn(10);
        return;
    }
    if npc.ai[0] == 0.0 {
        if npc.ai[1] == 0.0 {
            // `:27110` — the hover re-picks its target every tick.
            target_closest(npc, world, player);
            // `:27111-27117` — the hover speed/accel (getGood x1.15).
            let mut num442 = 12.0f32;
            let mut num443 = 0.4f32;
            if env.get_good {
                num442 *= 1.15;
                num443 *= 1.15;
            }
            // `:27118-27122` — the side: the NPC's integer half-width
            // vs the player's FULL width edge.
            let mut num444 = 1;
            if npc.x + ((w as i32 / 2) as f32) < player.x + crate::player::WIDTH {
                num444 = -1;
            }
            // `:27123-27125` — the ±400 side-offset hover point.
            let (mut cx, mut cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
            let (pcx, pcy) = player.center();
            let mut num445 = pcx + (num444 * 400) as f32 - cx;
            let mut num446 = pcy - cy;
            // `:27126-27133` — the Mech Queen replaces the hover point
            // with the orbit anchor at speed 14.
            if env.mech_queen.is_some() {
                num442 = 14.0;
                num445 = vector42.0 - cx;
                num446 = vector42.1 - cy;
            }
            // `:27134` — the distance (`num448` at `:27135` is a dead
            // store, dropped).
            let mut num447 = (num445 * num445 + num446 * num446).sqrt();
            if env.mech_queen.is_some() {
                // `:27136-27146` — the queen's capped 4:1 blend.
                if num447 > num442 {
                    num447 = num442 / num447;
                    num445 *= num447;
                    num446 *= num447;
                }
                npc.vx = (npc.vx * 4.0 + num445) / 5.0;
                npc.vy = (npc.vy * 4.0 + num446) / 5.0;
            } else {
                // `:27147-27184` — the classic normalize + approach.
                num447 = num442 / num447;
                num445 *= num447;
                num446 *= num447;
                approach_axis(&mut npc.vx, num445, num443);
                approach_axis(&mut npc.vy, num446, num443);
            }
            // `:27185-27198` — the 600-tick hover (1200 under the
            // queen) flips into the dash.
            let num449 = if env.mech_queen.is_some() { 1200 } else { 600 };
            npc.ai[2] += 1.0;
            if npc.ai[2] >= num449 as f32 {
                npc.ai[1] = 1.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                clear_target(npc);
                npc.net_update = true;
            } else {
                // `:27199-27212` — the shot timer (the expert's <80%
                // hp +0.6, getGood's +0.4).
                if !player.dead {
                    npc.ai[3] += 1.0;
                    if env.expert && (npc.hp as f64) < (npc.max_hp as f64) * 0.8 {
                        npc.ai[3] += 0.6;
                    }
                    if env.get_good {
                        npc.ai[3] += 0.4;
                    }
                }
                // `:27213-27237` — the cursed flame (96) at 60.
                if npc.ai[3] >= 60.0 {
                    npc.ai[3] = 0.0;
                    cx = npc.x + w * 0.5;
                    cy = npc.y + h * 0.5;
                    num445 = pcx - cx;
                    num446 = pcy - cy;
                    // netMode != 1 is constant-true (single-player).
                    // `:27221-27227` — speed 12 (expert 14), the
                    // `GetAttackDamage_ForProjectiles(25, 22)` endpoint.
                    let num450 = if env.expert { 14.0 } else { 12.0 };
                    let dmg = if env.expert { 22 } else { 25 };
                    num447 = num450 / (num445 * num445 + num446 * num446).sqrt();
                    num445 *= num447;
                    num446 *= num447;
                    // `:27231-27232` — the jitter: X draw FIRST.
                    num445 += rng.next_range(-40, 41) as f32 * 0.05;
                    num446 += rng.next_range(-40, 41) as f32 * 0.05;
                    // `:27233-27235` — the spawn at center + vel * 4.
                    npc.wants_hostile_shot.push((
                        crate::projectile::CURSED_FLAME,
                        cx + num445 * 4.0,
                        cy + num446 * 4.0,
                        num445,
                        num446,
                        dmg,
                    ));
                }
            }
        } else if npc.ai[1] == 1.0 {
            // `:27240-27278` — the dash: snap the aim, the 13 speed
            // with the expert hp ladder and the getGood x1.2, then the
            // direct velocity set.
            npc.rotation = num438;
            let mut num453 = 13.0f32;
            if env.expert {
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.9 {
                    num453 += 0.5;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.8 {
                    num453 += 0.5;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.7 {
                    num453 += 0.55;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.6 {
                    num453 += 0.6;
                }
                if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
                    num453 += 0.65;
                }
            }
            if env.get_good {
                num453 *= 1.2;
            }
            let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
            let (pcx, pcy) = player.center();
            let num454 = pcx - cx;
            let num455 = pcy - cy;
            let num456 = num453 / (num454 * num454 + num455 * num455).sqrt();
            npc.vx = num454 * num456;
            npc.vy = num455 * num456;
            npc.ai[1] = 2.0;
        } else if npc.ai[1] == 2.0 {
            // `:27279-27315` — the dash decay: 8 ticks of travel, then
            // the 0.9 damp with the ±0.1 dead-band; before that the
            // rotation rides the velocity.
            npc.ai[2] += 1.0;
            if npc.ai[2] >= 8.0 {
                npc.vx *= 0.9;
                npc.vy *= 0.9;
                clamp_dead_band(&mut npc.vx);
                clamp_dead_band(&mut npc.vy);
            } else {
                npc.rotation = npc.vy.atan2(npc.vx) - 1.57;
            }
            // `:27299-27314` — the 42-tick cycle: ten dashes, then
            // back to the hover.
            if npc.ai[2] >= 42.0 {
                npc.ai[3] += 1.0;
                npc.ai[2] = 0.0;
                clear_target(npc);
                npc.rotation = num438;
                if npc.ai[3] >= 10.0 {
                    npc.ai[1] = 0.0;
                    npc.ai[3] = 0.0;
                } else {
                    npc.ai[1] = 1.0;
                }
            }
        }
        // `:27316-27323` — the 40% hp hand-off into the spin.
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.4 {
            npc.ai[0] = 1.0;
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            npc.net_update = true;
        }
        return;
    }
    if npc.ai[0] == 1.0 || npc.ai[0] == 2.0 {
        // `:27328-27331` — the queen's spin reflects projectiles (no
        // port flag; the module note).
        // `:27332-27347` — the spin ramp: +0.005 up to 0.5 (phase 1),
        // -0.005 down to 0 (phase 2).
        if npc.ai[0] == 1.0 {
            npc.ai[2] += 0.005;
            if (npc.ai[2] as f64) > 0.5 {
                npc.ai[2] = 0.5;
            }
        } else {
            npc.ai[2] -= 0.005;
            if npc.ai[2] < 0.0 {
                npc.ai[2] = 0.0;
            }
        }
        // `:27348-27349`.
        npc.rotation += npc.ai[2];
        npc.ai[1] += 1.0;
        // `:27350-27373` — the 100-tick phase hand-off.
        if npc.ai[1] >= 100.0 {
            npc.ai[0] += 1.0;
            npc.ai[1] = 0.0;
            if npc.ai[0] == 3.0 {
                npc.ai[2] = 0.0;
            } else {
                // `:27360-27371` — the transition burst: the sounds
                // (3, 15) are skipped; the gore/dust visuals are
                // skipped but their 52 draws stay, in order: 2 x (3
                // gores x 2 draws) then 20 dusts x 2 draws.
                for _ in 0..2 {
                    for _ in 0..3 {
                        let _gx = rng.next_range(-30, 31) as f32 * 0.2;
                        let _gy = rng.next_range(-30, 31) as f32 * 0.2;
                    }
                }
                for _ in 0..20 {
                    let _dx = rng.next_range(-30, 31) as f32 * 0.2;
                    let _dy = rng.next_range(-30, 31) as f32 * 0.2;
                }
            }
        }
        // `:27374` — the per-tick spin dust: 2 draws, also on the
        // transition tick, AFTER the burst.
        let _dx = rng.next_range(-30, 31) as f32 * 0.2;
        let _dy = rng.next_range(-30, 31) as f32 * 0.2;
        // `:27375-27384` — the 0.98 damp with the ±0.1 dead-band.
        npc.vx *= 0.98;
        npc.vy *= 0.98;
        clamp_dead_band(&mut npc.vx);
        clamp_dead_band(&mut npc.vy);
        return;
    }
    // `:27387-27389` — phase 3: the HitSound write is a documented skip
    // (the static cue table); the damage/defense writes ride the
    // overrides. `defDamage` 50 / `defDefense` 10 are the case-126
    // `SetDefaults` values (`NPC.cs:10544-10545`): (int)(50 * 1.5) = 75,
    // 10 + 18 = 28.
    npc.damage_override = Some((50.0f64 * 1.5) as i32);
    npc.defense_override = Some(10 + 18);
    if npc.ai[1] == 0.0 {
        // `:27392-27401` — the approach: speed 4, accel 0.1, the ±180
        // side offset (the same full-width side test).
        let mut num459 = 4.0f32;
        let mut num460 = 0.1f32;
        let mut num461 = 1;
        if npc.x + ((w as i32 / 2) as f32) < player.x + crate::player::WIDTH {
            num461 = -1;
        }
        let (mut cx, mut cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let mut num462 = pcx + (num461 * 180) as f32 - cx;
        let mut num463 = pcy - cy;
        let mut num464 = (num462 * num462 + num463 * num463).sqrt();
        if env.mech_queen.is_none() {
            // `:27405-27431` — the expert's distance ladder.
            if env.expert {
                if num464 > 300.0 {
                    num459 += 0.5;
                }
                if num464 > 400.0 {
                    num459 += 0.5;
                }
                if num464 > 500.0 {
                    num459 += 0.55;
                }
                if num464 > 600.0 {
                    num459 += 0.55;
                }
                if num464 > 700.0 {
                    num459 += 0.6;
                }
                if num464 > 800.0 {
                    num459 += 0.6;
                }
            }
            // `:27432-27436` — getGood x1.15 on both.
            if env.get_good {
                num459 *= 1.15;
                num460 *= 1.15;
            }
            // `:27437-27471` — the normalize + approach.
            num464 = num459 / num464;
            num462 *= num464;
            num463 *= num464;
            approach_axis(&mut npc.vx, num462, num460);
            approach_axis(&mut npc.vy, num463, num460);
        }
        // `:27473-27486` — the 400-tick approach (1200 under the queen)
        // flips into the dash. No `else`: the flamethrower below runs
        // on the flip tick too.
        let num465 = if env.mech_queen.is_some() { 1200 } else { 400 };
        npc.ai[2] += 1.0;
        if npc.ai[2] >= num465 as f32 {
            npc.ai[1] = 1.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            clear_target(npc);
            npc.net_update = true;
        }
        // `:27487-27541` — the flamethrower behind `Collision.CanHit`
        // (`Collision.cs:381-479`, the port's box walk).
        if crate::projectile::can_hit(
            world,
            npc.x,
            npc.y,
            w,
            h,
            player.x,
            player.y,
            crate::player::WIDTH,
            player.height,
        ) {
            // `:27489-27494` — the 23-tick flamethrower sound loop: the
            // counter wraps past 22 and the wrap plays Item34 (S1-E-21g
            // wires the cue; the queue rides the machine's stream).
            npc.local_ai[2] += 1.0;
            if npc.local_ai[2] > 22.0 {
                npc.local_ai[2] = 0.0;
                npc.wants_sound.push(crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::Item34,
                    x: npc.x,
                    y: npc.y,
                });
            }
            // netMode != 1 is constant-true.
            // `:27497-27513` — the shot timer with the hp ladder.
            npc.local_ai[1] += 1.0;
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
                npc.local_ai[1] += 1.0;
            }
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
                npc.local_ai[1] += 1.0;
            }
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
                npc.local_ai[1] += 1.0;
            }
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
                npc.local_ai[1] += 2.0;
            }
            // `:27514-27539` — the eye fire (101) past 8.
            if npc.local_ai[1] > 8.0 {
                npc.local_ai[1] = 0.0;
                let num466 = 6.0f32;
                // `GetAttackDamage_ForProjectiles(30, 27)`'s endpoint.
                let dmg = if env.expert { 27 } else { 30 };
                cx = npc.x + w * 0.5;
                cy = npc.y + h * 0.5;
                num462 = pcx - cx;
                num463 = pcy - cy;
                num464 = num466 / (num462 * num462 + num463 * num463).sqrt();
                num462 *= num464;
                // `:27525-27526` — the jitter: Y draw FIRST (reversed
                // vs the phase-0 shot), x0.01.
                num463 = num463 * num464 + rng.next_range(-40, 41) as f32 * 0.01;
                num462 += rng.next_range(-40, 41) as f32 * 0.01;
                // `:27527-27528` — half the body's velocity rides along.
                num463 += npc.vy * 0.5;
                num462 += npc.vx * 0.5;
                // `:27529-27530` — the spawn BEHIND the aim (center -
                // vel * 1).
                cx -= num462 * 1.0;
                cy -= num463 * 1.0;
                if env.mech_queen.is_some() {
                    // `:27531-27537` — the queen fires along the
                    // rotation: `(rotation + PI/2).ToRotationVector2()`
                    // (`Utils.cs:1665-1668`) x 6 + velocity * 0.5, the
                    // spawn at center - vel * 3.
                    let a = npc.rotation + std::f32::consts::PI / 2.0;
                    let qx = a.cos() * num466 + npc.vx * 0.5;
                    let qy = a.sin() * num466 + npc.vy * 0.5;
                    num462 = qx;
                    num463 = qy;
                    cx = npc.x + w * 0.5 - qx * 3.0;
                    cy = npc.y + h * 0.5 - qy * 3.0;
                }
                npc.wants_hostile_shot.push((
                    crate::projectile::EYE_FIRE,
                    cx,
                    cy,
                    num462,
                    num463,
                    dmg,
                ));
            }
        }
        if env.mech_queen.is_some() {
            // `:27542-27561` — the queen's 60:1 orbit blend toward
            // `vector42` at the capped 14 speed (the first
            // `num462`/`num463` pair at `:27545-27546` is overwritten
            // at `:27547-27548` — dead stores, dropped). `vector46`
            // here is the post-shot spawn point when the flamethrower
            // fired this tick, the centre otherwise (the C# reuses the
            // variable).
            let num459 = 14.0f32;
            num462 = vector42.0 - cx;
            num463 = vector42.1 - cy;
            num464 = (num462 * num462 + num463 * num463).sqrt();
            if num464 > num459 {
                num464 = num459 / num464;
                num462 *= num464;
                num463 *= num464;
            }
            let num469 = 60.0;
            npc.vx = (npc.vx * (num469 - 1.0) + num462) / num469;
            npc.vy = (npc.vy * (num469 - 1.0) + num463) / num469;
        }
    } else if npc.ai[1] == 1.0 {
        // `:27563-27578` — the phase-3 dash: sound 15 skipped, the aim
        // snap, the 14 speed (+2.5 expert), the direct set.
        npc.rotation = num438;
        let mut num470 = 14.0f32;
        if env.expert {
            num470 += 2.5;
        }
        let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let num471 = pcx - cx;
        let num472 = pcy - cy;
        let num473 = num470 / (num471 * num471 + num472 * num472).sqrt();
        npc.vx = num471 * num473;
        npc.vy = num472 * num473;
        npc.ai[1] = 2.0;
    } else {
        // `:27580-27585` — any other state leaves the machine.
        if npc.ai[1] != 2.0 {
            return;
        }
        // `:27586-27607` — the decay: the expert counts 1.5/tick, the
        // 0.93 damp starts at 50; before that the rotation rides the
        // velocity.
        npc.ai[2] += 1.0;
        if env.expert {
            npc.ai[2] += 0.5;
        }
        if npc.ai[2] >= 50.0 {
            npc.vx *= 0.93;
            npc.vy *= 0.93;
            clamp_dead_band(&mut npc.vx);
            clamp_dead_band(&mut npc.vy);
        } else {
            npc.rotation = npc.vy.atan2(npc.vx) - 1.57;
        }
        // `:27608-27623` — the 80-tick cycle: six dashes, then back to
        // the approach.
        if npc.ai[2] >= 80.0 {
            npc.ai[3] += 1.0;
            npc.ai[2] = 0.0;
            clear_target(npc);
            npc.rotation = num438;
            if npc.ai[3] >= 6.0 {
                npc.ai[1] = 0.0;
                npc.ai[3] = 0.0;
            } else {
                npc.ai[1] = 1.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::TILE_PX;

    /// An open-air test world (no tiles — `can_hit`'s walk sees air
    /// everywhere), night by default so the machine runs its phases.
    fn world() -> World {
        let mut w = World::new("t".into(), 200, 120);
        w.day_time = false;
        w
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 100.0 * TILE_PX;
        p.y = 60.0 * TILE_PX;
        p
    }

    fn spaz(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Spazmatism, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    fn env() -> Style31Env<'static> {
        Style31Env::CLASSIC_SOLO
    }

    /// The SetDefaults identity (`NPC.cs:10539-10555`) and the aim's
    /// named-argument atan2 (`:27022`): the bottom-59 anchor, the
    /// integer halves, the +1.57 offset.
    #[test]
    fn s1_d_3_a31_the_identity_and_the_bottom_anchored_aim() {
        let w = world();
        let p = player(&w);
        let n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        assert_eq!(n.size(), (100.0, 110.0));
        assert_eq!(n.max_hp, 23000);
        assert_eq!(n.kind.ai_style(), 31);
        // Re-derive `num438` independently: atan2(y: B, x: A).
        let a = n.x + 50.0 - p.x - 10.0;
        let b = n.y + 110.0 - 59.0 - p.y - 21.0;
        let mut expect = b.atan2(a) + 1.57;
        if expect < 0.0 {
            expect += 6.283;
        } else if (expect as f64) > 6.283 {
            expect -= 6.283;
        }
        assert_eq!(aim_rotation(&n, &p), expect);
    }

    /// The turn (`:27036-27073`): a rotation far from the aim steps
    /// exactly 0.15 along the shortest path; inside the snap band it
    /// lands ON the aim.
    #[test]
    fn s1_d_3_a31_the_turn_steps_the_rate_and_snaps_the_band() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        let aim = aim_rotation(&n, &p);
        // Park well outside the band, on the short side.
        n.rotation = aim + 1.0;
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert!(
            (n.rotation - (aim + 1.0 - 0.15)).abs() < 1e-5,
            "the turn steps 0.15 toward the aim: {}",
            n.rotation
        );
        // Inside the band after the step: the snap lands on the aim
        // exactly (aim + 0.05 - 0.15 = aim - 0.1, strictly inside).
        // Restore the pre-tick position/velocity so the aim is the
        // same value (the integration tail moved the body).
        n.x = 60.0 * TILE_PX;
        n.y = 60.0 * TILE_PX;
        n.vx = 0.0;
        n.vy = 0.0;
        n.rotation = aim + 0.05;
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.rotation, aim, "the snap band lands on the aim");
    }

    /// RNG witness: the dust gate (`:27074`) draws exactly one
    /// `Next(5)` on a plain hover tick — the stream advances by that
    /// one draw and no other.
    #[test]
    fn s1_d_3_a31_the_dust_gate_draws_one_next_five_per_tick() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(rng.peek(), witness.peek(), "exactly one draw, in order");
    }

    /// The day gate (`:27100-27105`): vy -0.04, the 10-tick despawn
    /// encouragement, no phase advance — and the integration tail
    /// still moves the body (the C# `return` leaves AI() only).
    #[test]
    fn s1_d_3_a31_the_day_flee_rises_and_arms_the_despawn() {
        let mut w = world();
        w.day_time = true;
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        let (x0, y0) = (n.x, n.y);
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.vy, -0.04);
        assert_eq!(n.despawn_timer, 10);
        assert_eq!(n.ai, [0.0; 4], "the day gate precedes the phases");
        assert_eq!((n.x, n.y), (x0, y0 - 0.04), "the tail integrates");
        // The dust gate drew BEFORE the day return.
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_max(5);
        assert_eq!(rng.peek(), witness.peek());
    }

    /// The hover approach (`:27147-27184`): the ±400 side offset, the
    /// 12-speed normalize, the 0.4 accel with the sign-reversal
    /// double-add.
    #[test]
    fn s1_d_3_a31_the_hover_approaches_the_side_offset_point() {
        let w = world();
        let p = player(&w);
        // The NPC LEFT of the player: side -1, the hover point 400 px
        // left of the player's centre.
        let mut n = spaz(20.0 * TILE_PX, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        let (cx, cy) = (20.0 * TILE_PX + 50.0, 60.0 * TILE_PX + 55.0);
        let (pcx, pcy) = p.center();
        let dx = pcx - 400.0 - cx;
        let dy = pcy - cy;
        let k = 12.0 / (dx * dx + dy * dy).sqrt();
        // vx 0 < target, no reversal: a single 0.4 step.
        assert!((n.vx - 0.4).abs() < 1e-5, "vx {}", n.vx);
        assert!(dx * k > 0.4, "the target outruns the first step");
        // The reversal double-add: -0.5 + 0.4 is still negative while
        // the target is positive, so the accel lands twice.
        n.vx = -0.5;
        n.vy = 0.0;
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert!((n.vx - (-0.5 + 0.8)).abs() < 1e-4, "double-add: {}", n.vx);
    }

    /// The 600-tick flip (`:27190-27198`): ai[1]=1, the counters reset,
    /// ClearTarget (255), netUpdate.
    #[test]
    fn s1_d_3_a31_the_six_hundred_tick_hover_flips_into_the_dash() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 0.0, 599.0, 30.0];
        n.target = 0;
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0);
        assert_eq!(n.target, 255, "ClearTarget");
        assert!(n.net_update);
    }

    /// RNG witness + shot shape (`:27213-27235`): at ai[3] 59 the tick
    /// draws the gate then the X-then-Y jitter; the cursed flame (96)
    /// leaves at speed 12 + jitter, damage 25, from center + vel * 4.
    #[test]
    fn s1_d_3_a31_the_hover_shot_draws_x_then_y_and_fires_the_96() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 0.0, 0.0, 59.0];
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        let jx = witness.next_range(-40, 41) as f32 * 0.05;
        let jy = witness.next_range(-40, 41) as f32 * 0.05;
        let (cx, cy) = (n.x + 50.0, n.y + 55.0);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(rng.peek(), witness.peek(), "gate + X + Y, in order");
        assert_eq!(n.ai[3], 0.0, "the shot resets the timer");
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::CURSED_FLAME);
        assert_eq!(dmg, 25, "GetAttackDamage_ForProjectiles(25, 22) classic");
        let (pcx, pcy) = p.center();
        let k = 12.0 / ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt();
        let evx = (pcx - cx) * k + jx;
        let evy = (pcy - cy) * k + jy;
        assert_eq!((vx, vy), (evx, evy));
        assert_eq!((sx, sy), (cx + evx * 4.0, cy + evy * 4.0));
    }

    /// The dash (`:27240-27278`): the aim snap, the direct 13-speed
    /// set, ai[1]=2. The expert ladder at <50% hp adds the full 2.8.
    #[test]
    fn s1_d_3_a31_the_dash_sets_the_thirteen_speed() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 1.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        let speed = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((speed - 13.0).abs() < 1e-3, "speed {speed}");
        assert_eq!(n.ai[1], 2.0);
        // The expert ladder: hp under every band sums 0.5+0.5+0.55+
        // 0.6+0.65 = 2.8 (:27246-27265).
        let mut e = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        e.ai = [0.0, 1.0, 0.0, 0.0];
        e.hp = e.max_hp * 4 / 10; // 40% — under every band
        let expert = Style31Env { expert: true, ..env() };
        spazmatism_ai(&mut e, &w, &p, &mut rng, &expert);
        let speed = (e.vx * e.vx + e.vy * e.vy).sqrt();
        assert!((speed - 15.8).abs() < 1e-3, "expert speed {speed}");
    }

    /// The dash decay (`:27279-27314`): the 0.9 damp only from the 8th
    /// tick, the velocity rotation before it, the 42-tick cycle's
    /// ten-dash wrap.
    #[test]
    fn s1_d_3_a31_the_dash_decays_and_the_cycle_wraps_at_ten() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 2.0, 7.0, 0.0];
        n.vx = 10.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.vx, 9.0, "the 8th tick damps 0.9");
        // Before the 8th: the rotation rides the velocity, no damp.
        n.ai = [0.0, 2.0, 0.0, 0.0];
        n.vx = 10.0;
        n.vy = 0.0;
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.vx, 10.0);
        assert!((n.rotation - (0.0f32.atan2(10.0) - 1.57)).abs() < 1e-5);
        // The 42-tick wrap: dash 10 returns to the hover.
        n.ai = [0.0, 2.0, 41.0, 9.0];
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!((n.ai[1], n.ai[2], n.ai[3]), (0.0, 0.0, 0.0));
        assert_eq!(n.target, 255, "ClearTarget on the cycle");
    }

    /// The 40% hand-off (`:27316-27323`): the phase resets to the spin
    /// with every counter zeroed and netUpdate armed.
    #[test]
    fn s1_d_3_a31_the_forty_percent_handoff_resets_into_the_spin() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.hp = n.max_hp * 39 / 100;
        n.ai = [0.0, 2.0, 12.0, 4.0];
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.ai, [1.0, 0.0, 0.0, 0.0]);
        assert!(n.net_update);
    }

    /// RNG witness, the spin (`:27348-27384`): a plain spin tick draws
    /// the gate + the 2 dust draws; the 100-tick transition into
    /// ai[0]==2 adds the 52-draw burst BEFORE the per-tick pair; the
    /// transition into ai[0]==3 zeroes ai[2] and draws no burst.
    #[test]
    fn s1_d_3_a31_the_spin_burst_draws_fifty_two_then_the_pair() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [1.0, 0.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        let _d1 = witness.next_range(-30, 31);
        let _d2 = witness.next_range(-30, 31);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(rng.peek(), witness.peek(), "gate + 2 dust draws");
        assert!((n.ai[2] - 0.005).abs() < 1e-6, "the ramp steps 0.005");
        // The transition into phase 2: 52 burst draws + the pair.
        n.ai = [1.0, 99.0, 0.5, 0.0];
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        for _ in 0..52 {
            let _ = witness.next_range(-30, 31);
        }
        let _d1 = witness.next_range(-30, 31);
        let _d2 = witness.next_range(-30, 31);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(rng.peek(), witness.peek(), "52 burst + 2, in order");
        // The transition into phase 3: no burst, ai[2] zeroed.
        n.ai = [2.0, 99.0, 0.005, 0.0];
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        let _d1 = witness.next_range(-30, 31);
        let _d2 = witness.next_range(-30, 31);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.ai[0], 3.0);
        assert_eq!(n.ai[2], 0.0, "the phase-3 entry zeroes the spin");
        assert_eq!(rng.peek(), witness.peek(), "no burst into phase 3");
    }

    /// Phase 3's stat writes (`:27387-27389`): damage 75, defense 28,
    /// every tick of the phase.
    #[test]
    fn s1_d_3_a31_phase_three_writes_the_damage_and_defense() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [3.0, 0.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(n.damage_override, Some(75), "(int)(50 * 1.5)");
        assert_eq!(n.defense_override, Some(28), "10 + 18");
        assert_eq!(n.defense(), 28);
    }

    /// RNG witness + laser shape (`:27514-27538`): the eye fire (101)
    /// at speed 6 with the Y-FIRST jitter, half the body velocity, the
    /// spawn at center - vel; damage 30.
    #[test]
    fn s1_d_3_a31_the_laser_draws_y_then_x_and_fires_the_101() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [3.0, 0.0, 0.0, 0.0];
        n.local_ai = [0.0, 8.0, 0.0, 0.0];
        let (cx, cy) = (n.x + 50.0, n.y + 55.0);
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        let jy = witness.next_range(-40, 41) as f32 * 0.01;
        let jx = witness.next_range(-40, 41) as f32 * 0.01;
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(rng.peek(), witness.peek(), "gate + Y + X, in order");
        assert_eq!(n.local_ai[1], 0.0, "the laser resets the timer");
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::EYE_FIRE);
        assert_eq!(dmg, 30, "GetAttackDamage_ForProjectiles(30, 27) classic");
        // The approach ran FIRST (:27440-27471): from rest, one 0.1
        // step on each axis toward the ±180-offset point — the body
        // velocity the laser reads is (0.1, -0.1).
        assert_eq!((n.vx, n.vy), (0.1, -0.1));
        let (pcx, pcy) = p.center();
        let k = 6.0 / ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt();
        let evy = (pcy - cy) * k + jy + (-0.1) * 0.5;
        let evx = (pcx - cx) * k + jx + 0.1 * 0.5;
        assert!((vx - evx).abs() < 1e-4 && (vy - evy).abs() < 1e-4);
        assert!((sx - (cx - evx)).abs() < 1e-4 && (sy - (cy - evy)).abs() < 1e-4);
    }

    /// The CanHit gate (`:27487`): a solid wall between the boxes stops
    /// the flamethrower counters — no laser draws beyond the gate.
    #[test]
    fn s1_d_3_a31_the_wall_blocks_the_flamethrower() {
        let mut w = world();
        for y in 40..80 {
            let i = w.index(80, y);
            w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [3.0, 0.0, 0.0, 0.0];
        n.local_ai = [0.0, 8.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!(rng.peek(), witness.peek(), "only the dust gate drew");
        assert_eq!(n.local_ai[1], 8.0, "the timer never advanced");
        assert!(n.wants_hostile_shot.is_empty());
    }

    /// The phase-3 dash + decay (`:27563-27623`): the 14 speed (+2.5
    /// expert), the 1.5/tick expert counter, the 0.93 damp from 50,
    /// the six-dash wrap at 80.
    #[test]
    fn s1_d_3_a31_the_phase_three_dash_and_six_cycle() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [3.0, 1.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        let speed = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((speed - 14.0).abs() < 1e-3, "speed {speed}");
        assert_eq!(n.ai[1], 2.0);
        // The expert's 1.5/tick counter and the 0.93 damp at 50.
        let mut e = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        e.ai = [3.0, 2.0, 49.0, 0.0];
        e.vx = 10.0;
        let expert = Style31Env { expert: true, ..env() };
        spazmatism_ai(&mut e, &w, &p, &mut rng, &expert);
        assert_eq!(e.ai[2], 50.5, "49 + 1 + 0.5");
        assert!((e.vx - 9.3).abs() < 1e-4, "the 0.93 damp: {}", e.vx);
        // The six-dash wrap.
        n.ai = [3.0, 2.0, 79.0, 5.0];
        spazmatism_ai(&mut n, &w, &p, &mut rng, &env());
        assert_eq!((n.ai[1], n.ai[2], n.ai[3]), (0.0, 0.0, 0.0));
        assert_eq!(n.target, 255);
    }

    /// The twin discouragement (`:27080-27089`): at night with an
    /// active timer under 10, the sibling's timeLeft - 1 raises ours;
    /// day / no-timer / healthy-timer ticks leave it alone.
    #[test]
    fn s1_d_3_a31_the_twin_alive_loop_discourages_the_despawn() {
        let w = world();
        let p = player(&w);
        let mut n = spaz(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.despawn_timer = 5;
        let twins = Style31Env { twin_time_lefts: &[300, 120], ..env() };
        let mut rng = UnifiedRandom::new(7);
        spazmatism_ai(&mut n, &w, &p, &mut rng, &twins);
        // 300 - 1 = 299 raises 5; 120 - 1 = 119 cannot lower it.
        assert_eq!(n.despawn_timer, 299);
        // The -1 sentinel: no active timer, no raise.
        n.despawn_timer = -1;
        spazmatism_ai(&mut n, &w, &p, &mut rng, &twins);
        assert_eq!(n.despawn_timer, -1);
        // A healthy timer (>= 10) skips the loop.
        n.despawn_timer = 50;
        spazmatism_ai(&mut n, &w, &p, &mut rng, &twins);
        assert_eq!(n.despawn_timer, 50);
    }
}
