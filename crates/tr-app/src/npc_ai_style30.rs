//! The Retinazer's aiStyle-30 machine (S1-D-3 retinazer) — `NPC.AI()`'s
//! `else if (this.aiStyle == 30)` arm (`NPC.cs:26405-27014`). Type 125
//! (Retinazer) is the style's ONLY user (`NPCID.cs:10927` Retinazer = 125;
//! `SetDefaults` case 125, `NPC.cs:10522-10537`: 100x110, aiStyle 30,
//! def 10, dmg 45, 20000 HP, kb 0, noGravity + noTileCollide, boss,
//! value 120000, npcSlots 5).
//!
//! **WIRED (S1-D-3-wire-retinazer).** The mislabeled live arm —
//! `npc.rs::update_retinazer`, a port of the aiStyle-57 MourningWood arm
//! (`NPC.cs:32761-33043`) running under the Retinazer's kind dispatch
//! (wrong projectiles 325/326-328 vs 83/100, no laser phases, no 40%-hp
//! transform, no despawn gates, RNG stream mismatch — audit
//! `docs/research/51-audit-twins-skeletron.md` §1, R1-R6) — is RETIRED:
//! the style axis claims `30 =>` (`npc.rs`'s `match
//! self.kind.ai_style()`, the style31 precedent), the kind arm is an
//! `unreachable!` marker and `update_retinazer` is deleted (the faithful
//! type-325 machine lives in `npc_ai_pumpkin_boss.rs::mourning_wood_step`,
//! claimed by the axis's `57 if kind == MourningWood` arm). The twin
//! `timeLeft` list rides [`Npc::twin_time_lefts`], stamped by the sim's
//! pre-pass (`world_sim/npcs.rs`, the Plantera hook-anchor precedent) —
//! the twin-despawn coupling is live for the Retinazer; the Spazmatism's
//! style-31 dispatch still passes `CLASSIC_SOLO` (its own follow-up).
//!
//! The machine: the invalid-target retarget (`:26407-26410`), the
//! bottom-anchored aim `num397` with the ±6.283 wrap and the **0.1**
//! shortest-path turn with the twice-applied snap band (`:26412-26459`
//! — the style31 skeleton at 0.1, NOT 0.15, and NO queen quartering),
//! the every-tick `Next(5)` dust gate (`:26460-26465`), the twin-alive
//! despawn discouragement (`:26466-26475`), the day/dead flee
//! (`:26486-26491`), phase 0's hover/83-laser → 12-dash → decay cycle
//! with the FOUR-dash wrap (`:26492-26701`), the 40% hp hand-off into
//! the spin-up/spin-down (`:26702-26709`, `:26712-26772`) and phase 3's
//! 67-damage/20-defense writes + the 300-tick approach/100-DeathLaser
//! hover and the ±340 strafe/100-DeathLaser cycle (`:26773-27014`).
//!
//! RNG order on the caller's `UpdateNPCs` stream (`Main.cs:17767`), all
//! draws preserved even where the visual is a documented skip:
//! 1. `Next(5)` dust gate — EVERY tick, before the day gate (`:26460`);
//! 2. phase-0 laser jitter: `Next(-40, 41) * 0.08` X then Y
//!    (`:26636-26637`), only on a firing hover tick;
//! 3. spin transition burst, only when NOT entering `ai[0] == 3`:
//!    2 x (3 gores x 2 draws `Next(-30, 31) * 0.2`) (`:26747-26752`)
//!    then 20 dusts x 2 draws (`:26753-26756`) — 52 draws;
//! 4. per-tick spin dust: 2 draws `Next(-30, 31) * 0.2` (`:26760`),
//!    also on the transition tick, AFTER the burst.
//! **NO other draws**: the dashes (`:26645-26701`) roll nothing (unlike
//! the brief's "dash rolls" — the C# has none), and BOTH phase-3
//! DeathLaser shots (`:26890-26906`, `:26991-27003`) fire with NO jitter
//! (unlike the Spazmatism's 101 flamethrower, whose Y-first x0.01 pair
//! at `:27525-27526` has no style-30 counterpart). A plain hover,
//! approach, dash or phase-3 laser tick advances the stream by exactly
//! the one `Next(5)` gate draw.
//!
//! Deliberate stand-ins (house pattern, recorded in ANALYSIS):
//! - `Main.expertMode` / `Main.getGoodWorld` ride [`Style30Env`] — the
//!   WIRED dispatch reads them live (`player.buffs.expert_mode`, the
//!   world's FOR_THE_WORTHY seed — the `npc_ai_style43` precedent);
//!   the branch maths are ported and witness-tested (the 8.25/0.115
//!   hover, the +0.3x4 shot ladder, the 15 dash, the 9.5/0.175
//!   approach, the 10 laser, the +1.5 strafe timer).
//! - `NPC.IsMechQueenUp` (`NPC.cs:6868-6883`) needs an active type 127
//!   — the port has no such kind (the S1-D-3j precedent), so the
//!   dispatch passes `mech_queen: None`; the orbit-anchor / 14-speed /
//!   1200-tick / reflect branches stay ported behind the style31
//!   module's [`crate::npc_ai_style31::MechQueenAnchor`] seam (shared
//!   data struct — the anchor maths are identical bar the X sign, see
//!   `:26481`'s **-150** vs style31's +150).
//! - `NPC.npcsFoundForCheckActive[135]` (`:26581`, TheDestroyerBody —
//!   `NPCID.cs:10947`) gates the queen's shot cadence 90 vs 120; the
//!   env carries it as `destroyer_body_active` (false in the port).
//! - The twin-alive loop (`:26468-26474`) scans the other active
//!   125/126 slots; the pure fn takes their `timeLeft`s in slot order
//!   ([`Style30Env::twin_time_lefts`]). `Npc::update_rng` cannot see
//!   its siblings, so the sim's pre-pass stamps [`Npc::twin_time_lefts`]
//!   before each tick (the Plantera hook-anchor carrier precedent) and
//!   the dispatch borrows it; outside the sim (tests) the list is empty.
//! - `DiscourageDespawn` (`NPC.cs:7258-7265`) raises `timeLeft`; the
//!   port's `despawn_timer` carries a -1 "no timer" sentinel
//!   (`npc.rs`'s `encourage_despawn` convention), so the gate and the
//!   raise only run on an ACTIVE timer (>= 0).
//! - `Main.netMode != 1` is constant-true (the port is single-player,
//!   netMode 0): the shot blocks and the discourage loop always run;
//!   the `netMode == 1` early return (`:26869-26872`) is constant-false
//!   and folds away.
//! - The dust VISUALS are W4-e2's second batch (sites 26462/26755/
//!   26760): the gate drip, the 20-dust hand-off burst and the per-tick
//!   spin dust spawn through the live `DustPool` (`npc_dust.rs`'s
//!   shared twins' emitters) on the same stream, argument draws first.
//!   The gore bodies (143/7/6) stay W4-d55's — their six argument pairs
//!   are consumed here in order; the sounds (3 / 15) stay SKIPPED.
//! - `reflectsProjectiles = false` (`:26485`) / `= true` under the
//!   Mech Queen (`:26716`): the port has no NPC projectile-reflect
//!   flag; with no Mech Queen the C# value is constant-false, the
//!   port's default — a no-op, dropped.
//! - `HitSound = SoundID.NPCHit4` (`:26775`): the port's hit cues ride
//!   the static per-kind table (`npc/data.rs::hit_sound`, S1-E-21a);
//!   the dynamic phase-3 write is a documented skip.
//! - `GetAttackDamage_ForProjectiles(20, 19)` / `(25, 23)` / `(18, 17)`
//!   (`NPC.cs:7052-7055`, `Utils.Remap` over the difficulty): classic
//!   20 / 25 / 18, expert 19 / 23 / 17 — the env's `expert` bool picks
//!   the endpoint (no master difficulty in the port).
//! - The private aim/approach/dead-band helpers mirror the style31
//!   sibling's (`npc_ai_style31.rs:140-177`); a shared extraction is
//!   deferred to the wiring wave to keep the landed module untouched.

use crate::npc::{Npc, NpcKind};
use crate::npc_ai_style31::MechQueenAnchor;
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// The world/difficulty inputs the pure machine cannot read itself.
pub struct Style30Env<'a> {
    /// The OTHER active type-125/126 NPCs' `timeLeft`s in slot order
    /// (`:26468-26474`'s `num400 != whoAmI` scan). Empty = no twin.
    pub twin_time_lefts: &'a [i32],
    /// `NPC.IsMechQueenUp` (`:6868-6883`) — None = no Mech Queen.
    pub mech_queen: Option<MechQueenAnchor>,
    /// `NPC.npcsFoundForCheckActive[135]` (`:26581`) — an active
    /// Destroyer body slows the queen's shot cadence 90 → 120.
    pub destroyer_body_active: bool,
    /// `Main.expertMode` — the port is classic, the dispatch passes false.
    pub expert: bool,
    /// `Main.getGoodWorld` — the port has no secret world seed, false.
    pub get_good: bool,
}

#[cfg(test)]
impl Style30Env<'static> {
    /// The stock env: classic single-player, no twin list, no Mech
    /// Queen. The axis dispatch builds a live env (the sim-stamped
    /// twin list + the difficulty flags); the tests use this constant.
    pub const CLASSIC_SOLO: Style30Env<'static> = Style30Env {
        twin_time_lefts: &[],
        mech_queen: None,
        destroyer_body_active: false,
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
    crate::npc_target::target_closest(npc, world, &[cand], true, 30, 0);
}

/// `NPC.ClearTarget()` (`NPC.cs:42924-42927`): `target = 255`.
fn clear_target(npc: &mut Npc) {
    npc.target = 255;
}

/// The aim `num397` (`NPC.cs:26412-26420`). The C# calls
/// `Math.Atan2(x: A, y: B)` with NAMED arguments — the value bound to
/// `y` (the NPC's bottom − 59 minus the player's centre Y) is the
/// atan2 numerator; the halves are INTEGER divisions (`width / 2`,
/// `height / 2`). Identical to the style31 aim (`:27022`).
fn aim_rotation(npc: &Npc, player: &Player) -> f32 {
    let (w, h) = npc.size();
    let a = npc.x + (w as i32 / 2) as f32 - player.x - (crate::player::WIDTH as i32 / 2) as f32;
    let b = npc.y + h - 59.0 - player.y - (player.height as i32 / 2) as f32;
    let mut num397 = b.atan2(a) + 1.57;
    // `:26413-26420` — the ±6.283 wrap (the C#'s truncated 2π).
    if num397 < 0.0 {
        num397 += 6.283;
    } else if (num397 as f64) > 6.283 {
        num397 -= 6.283;
    }
    num397
}

/// One axis of the C#'s approach (`:26543-26574`, `:26818-26849`,
/// `:26932-26963` — the three blocks are identical): step `accel`
/// toward the normalized target, with the unconditional double-add on
/// a sign reversal.
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

/// The ±0.1 dead-band clamp (`:26672-26679`, `:26763-26770` — the C#
/// compares in double).
fn clamp_dead_band(v: &mut f32) {
    if (*v as f64) > -0.1 && (*v as f64) < 0.1 {
        *v = 0.0;
    }
}

/// The twin-alive scan the sim's pre-pass runs for each eye
/// (`NPC.cs:26468-26474`, the style-31 sibling at `:27083-27089`): the
/// OTHER slots passing `is_twin` (active type 125/126 — the caller's
/// `hp > 0` liveness gate), their `timeLeft`s in SLOT order. The port's
/// `despawn_timer` -1 "no timer" sentinel reads as an active C# slot's
/// freshly-set `timeLeft` — the raise loop gates on OUR timer anyway
/// ([`Style30Env::twin_time_lefts`]), so the sentinel passes through
/// unchanged and only ever fails the `despawn_timer < t` comparison.
pub fn twin_time_left_scan(
    npcs: &[Npc],
    who: usize,
    is_twin: impl Fn(&Npc) -> bool,
) -> Vec<i32> {
    npcs.iter()
        .enumerate()
        .filter(|(j, n)| *j != who && is_twin(n))
        .map(|(_, n)| n.despawn_timer)
        .collect()
}

/// One fixed step of the Retinazer (type 125) through the aiStyle-30
/// arm (`NPC.cs:26405-27014`), ending with the `noTileCollide`
/// integration (`NPC.cs:90494-90498` — `position += velocity`, no tile
/// interaction; the `oldVelocity` capture + collide-flag reset ride the
/// `UpdateCollision` tail convention, `:92538-92540`/`:92605-92607`).
/// The caller's `rng` is the `UpdateNPCs` stream; the draw order is the
/// module header's list.
pub fn retinazer_ai(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    env: &Style30Env,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) {
    debug_assert_eq!(npc.kind, NpcKind::Retinazer);
    ai_body(npc, world, player, rng, env, dust, dust_ctx);
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
/// of the aiStyle-30 block (the integration tail above still runs).
fn ai_body(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    env: &Style30Env,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) {
    let (w, h) = npc.size();
    // `:26407-26410` — the invalid-target retarget.
    if npc.target < 0 || npc.target == 255 || player.dead {
        target_closest(npc, world, player);
    }
    // `:26411` — `dead2` (the port's single player is slot 0; the
    // `!active` half of the retarget gate is constant-false here).
    let dead2 = player.dead;
    // `:26412-26420` — the aim.
    let num397 = aim_rotation(npc, player);
    // `:26421` — the turn rate: 0.1, NO queen quartering (the style31
    // arm quarters under the queen at `:27033-27035`; style 30 does not).
    let num398 = 0.1f32;
    // `:26422-26443` — the shortest-path turn (the 3.1415 half-turn).
    if npc.rotation < num397 {
        if ((num397 - npc.rotation) as f64) > 3.1415 {
            npc.rotation -= num398;
        } else {
            npc.rotation += num398;
        }
    } else if npc.rotation > num397 {
        if ((npc.rotation - num397) as f64) > 3.1415 {
            npc.rotation += num398;
        } else {
            npc.rotation -= num398;
        }
    }
    // `:26444-26447` — the first snap band.
    if npc.rotation > num397 - num398 && npc.rotation < num397 + num398 {
        npc.rotation = num397;
    }
    // `:26448-26455` — the rotation wrap.
    if npc.rotation < 0.0 {
        npc.rotation += 6.283;
    } else if (npc.rotation as f64) > 6.283 {
        npc.rotation -= 6.283;
    }
    // `:26456-26459` — the second snap band (after the wrap).
    if npc.rotation > num397 - num398 && npc.rotation < num397 + num398 {
        npc.rotation = num397;
    }
    // `:26460-26465` — the dust gate: the draw runs EVERY tick, before
    // the day gate. W4-e2 (site 26462): the passed gate spawns the
    // type-5 drip on the live pool — the shared twins' emitter.
    let dust_gate = rng.next_max(5) == 0;
    crate::npc_dust::twin_drip(
        dust_gate, npc.x, npc.y, w as i32, h, npc.vx, rng, dust, dust_ctx, world,
    );
    // `:26466-26475` — the twin-alive despawn discouragement: at night,
    // alive, with the timer under 10, every other active 125/126's
    // `timeLeft - 1` raises ours (`DiscourageDespawn`, `:7258-7265`).
    // The port's -1 sentinel keeps the gate/raise on active timers
    // only (the module note). netMode != 1 is constant-true.
    if !world.day_time && !dead2 && npc.despawn_timer >= 0 && npc.despawn_timer < 10 {
        for &twin_time_left in env.twin_time_lefts {
            let t = twin_time_left - 1;
            if npc.despawn_timer < t {
                npc.despawn_timer = t;
            }
        }
    }
    // `:26476-26484` — the Mech Queen orbit anchor `vector36`:
    // `Utils.RotatedBy(queen.vx * 0.025, queenCenter + (-150, -250) *
    // 0.75, queenCenter)` (`Utils.cs:1675-1684`) — the X offset is
    // NEGATIVE here (style31's `:27093` uses +150).
    let mut vector36 = (0.0f32, 0.0f32);
    if let Some(q) = env.mech_queen {
        let radians = q.vx * 0.025;
        let c = (radians as f64).cos() as f32;
        let s = (radians as f64).sin() as f32;
        let ox = -150.0 * 0.75;
        let oy = -250.0 * 0.75;
        vector36 = (q.center_x + ox * c - oy * s, q.center_y + ox * s + oy * c);
    }
    // `:26485` — `reflectsProjectiles = false`: constant-false without
    // the Mech Queen, the port's default (the module note).
    // `:26486-26491` — the day/dead flee: rise 0.04/tick, arm the
    // 10-tick despawn, leave the machine (the integration still runs).
    if world.day_time || dead2 {
        npc.vy -= 0.04;
        npc.encourage_despawn(10);
        return;
    }
    if npc.ai[0] == 0.0 {
        if npc.ai[1] == 0.0 {
            // `:26496-26507` — the hover speed/accel (expert 8.25/
            // 0.115, getGood x1.15). NO per-tick retarget here (the
            // style31 hover re-picks at `:27110`; style 30 does not).
            let mut num401 = 7.0f32;
            let mut num402 = 0.1f32;
            if env.expert {
                num401 = 8.25;
                num402 = 0.115;
            }
            if env.get_good {
                num401 *= 1.15;
                num402 *= 1.15;
            }
            // `:26508-26512` — the side: the NPC's integer half-width
            // vs the player's FULL width edge.
            let mut num403 = 1;
            if npc.x + ((w as i32 / 2) as f32) < player.x + crate::player::WIDTH {
                num403 = -1;
            }
            // `:26513-26515` — the ±300 side-offset hover point, 300 px
            // ABOVE the player's centre.
            let (mut cx, mut cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
            let (pcx, pcy) = player.center();
            let mut num404 = pcx + (num403 * 300) as f32 - cx;
            let mut num405 = pcy - 300.0 - cy;
            // `:26516-26523` — the Mech Queen replaces the hover point
            // with the orbit anchor at speed 14.
            if env.mech_queen.is_some() {
                num401 = 14.0;
                num404 = vector36.0 - cx;
                num405 = vector36.1 - cy;
            }
            // `:26524-26525` — the distance; `num407` keeps the
            // PRE-normalize length for the shot gate (`:26592`).
            let mut num406 = (num404 * num404 + num405 * num405).sqrt();
            let num407 = num406;
            if env.mech_queen.is_some() {
                // `:26526-26537` — the queen's capped 60:1 blend.
                if num406 > num401 {
                    num406 = num401 / num406;
                    num404 *= num406;
                    num405 *= num406;
                }
                let num408 = 60.0;
                npc.vx = (npc.vx * (num408 - 1.0) + num404) / num408;
                npc.vy = (npc.vy * (num408 - 1.0) + num405) / num408;
            } else {
                // `:26538-26575` — the classic normalize + approach.
                num406 = num401 / num406;
                num404 *= num406;
                num405 *= num406;
                approach_axis(&mut npc.vx, num404, num402);
                approach_axis(&mut npc.vy, num405, num402);
            }
            // `:26576-26582` — the 600-tick hover / 60-tick shot
            // cadence (queen: 1200, and 90 or 120 by the active
            // Destroyer body — type 135, `NPCID.cs:10947`).
            let mut num409 = 600;
            let mut num410 = 60;
            if env.mech_queen.is_some() {
                num409 = 1200;
                num410 = if env.destroyer_body_active { 120 } else { 90 };
            }
            // `:26583-26591` — the hover flips into the dash.
            npc.ai[2] += 1.0;
            if npc.ai[2] >= num409 as f32 {
                npc.ai[1] = 1.0;
                npc.ai[2] = 0.0;
                npc.ai[3] = 0.0;
                clear_target(npc);
                npc.net_update = true;
            } else if npc.y + h < player.y && num407 < 400.0 {
                // `:26592` — the shot gate: the NPC's bottom ABOVE the
                // player's top AND the hover point within 400 px.
                // `:26594-26617` — the shot timer (the expert's four
                // +0.3 hp bands, getGood's +0.5).
                if !player.dead {
                    npc.ai[3] += 1.0;
                    if env.expert && (npc.hp as f64) < (npc.max_hp as f64) * 0.9 {
                        npc.ai[3] += 0.3;
                    }
                    if env.expert && (npc.hp as f64) < (npc.max_hp as f64) * 0.8 {
                        npc.ai[3] += 0.3;
                    }
                    if env.expert && (npc.hp as f64) < (npc.max_hp as f64) * 0.7 {
                        npc.ai[3] += 0.3;
                    }
                    if env.expert && (npc.hp as f64) < (npc.max_hp as f64) * 0.6 {
                        npc.ai[3] += 0.3;
                    }
                    if env.get_good {
                        npc.ai[3] += 0.5;
                    }
                }
                // `:26618-26642` — the eye laser (83) at the cadence.
                if npc.ai[3] >= num410 as f32 {
                    npc.ai[3] = 0.0;
                    cx = npc.x + w * 0.5;
                    cy = npc.y + h * 0.5;
                    num404 = pcx - cx;
                    num405 = pcy - cy;
                    // netMode != 1 is constant-true (single-player).
                    // `:26626-26632` — speed 9 (expert 10.5), the
                    // `GetAttackDamage_ForProjectiles(20, 19)` endpoint.
                    let num411 = if env.expert { 10.5 } else { 9.0 };
                    let dmg = if env.expert { 19 } else { 20 };
                    num406 = num411 / (num404 * num404 + num405 * num405).sqrt();
                    num404 *= num406;
                    num405 *= num406;
                    // `:26636-26637` — the jitter: X draw FIRST, x0.08.
                    num404 += rng.next_range(-40, 41) as f32 * 0.08;
                    num405 += rng.next_range(-40, 41) as f32 * 0.08;
                    // `:26638-26640` — the spawn at center + vel * 15.
                    npc.wants_hostile_shot.push((
                        crate::projectile::EYE_LASER,
                        cx + num404 * 15.0,
                        cy + num405 * 15.0,
                        num404,
                        num405,
                        dmg,
                    ));
                }
            }
        } else if npc.ai[1] == 1.0 {
            // `:26645-26663` — the dash: snap the aim, the 12 speed
            // (expert 15, getGood +2 ADDITIVE), the direct set. No RNG.
            npc.rotation = num397;
            let mut num414 = 12.0f32;
            if env.expert {
                num414 = 15.0;
            }
            if env.get_good {
                num414 += 2.0;
            }
            let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
            let (pcx, pcy) = player.center();
            let num415 = pcx - cx;
            let num416 = pcy - cy;
            let num417 = num414 / (num415 * num415 + num416 * num416).sqrt();
            npc.vx = num415 * num417;
            npc.vy = num416 * num417;
            npc.ai[1] = 2.0;
        } else if npc.ai[1] == 2.0 {
            // `:26665-26700` — the dash decay: 25 ticks of travel, then
            // the 0.96 damp with the ±0.1 dead-band; before that the
            // rotation rides the velocity.
            npc.ai[2] += 1.0;
            if npc.ai[2] >= 25.0 {
                npc.vx *= 0.96;
                npc.vy *= 0.96;
                clamp_dead_band(&mut npc.vx);
                clamp_dead_band(&mut npc.vy);
            } else {
                npc.rotation = npc.vy.atan2(npc.vx) - 1.57;
            }
            // `:26685-26700` — the 70-tick cycle: FOUR dashes, then
            // back to the hover.
            if npc.ai[2] >= 70.0 {
                npc.ai[3] += 1.0;
                npc.ai[2] = 0.0;
                clear_target(npc);
                npc.rotation = num397;
                if npc.ai[3] >= 4.0 {
                    npc.ai[1] = 0.0;
                    npc.ai[3] = 0.0;
                } else {
                    npc.ai[1] = 1.0;
                }
            }
        }
        // `:26702-26709` — the 40% hp hand-off into the spin.
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
        // `:26714-26717` — the queen's spin reflects projectiles (no
        // port flag; the module note).
        // `:26718-26733` — the spin ramp: +0.005 up to 0.5 (phase 1),
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
        // `:26734-26735`.
        npc.rotation += npc.ai[2];
        npc.ai[1] += 1.0;
        // `:26736-26759` — the 100-tick phase hand-off.
        if npc.ai[1] >= 100.0 {
            npc.ai[0] += 1.0;
            npc.ai[1] = 0.0;
            if npc.ai[0] == 3.0 {
                npc.ai[2] = 0.0;
            } else {
                // `:26746-26757` — the transition burst: the sounds
                // (3, 15) are skipped; the gore (143/7/6) / dust
                // visuals are skipped but their 52 draws stay, in
                // order: 2 x (3 gores x 2 draws) then 20 dusts x 2.
                for _ in 0..2 {
                    for _ in 0..3 {
                        let _gx = rng.next_range(-30, 31) as f32 * 0.2;
                        let _gy = rng.next_range(-30, 31) as f32 * 0.2;
                    }
                }
                // W4-e2 (site 26755): the 20 type-5 dusts spawn on the
                // live pool — argument pair then internals per dust.
                crate::npc_dust::twin_spin_burst(
                    npc.x, npc.y, w as i32, h as i32, rng, dust, dust_ctx, world,
                );
            }
        }
        // `:26760` — the per-tick spin dust: 2 draws, also on the
        // transition tick, AFTER the burst. W4-e2 (site 26760): the
        // type-5 dust spawns on the live pool.
        crate::npc_dust::twin_spin_tick(
            npc.x, npc.y, w as i32, h as i32, rng, dust, dust_ctx, world,
        );
        // `:26761-26770` — the 0.98 damp with the ±0.1 dead-band.
        npc.vx *= 0.98;
        npc.vy *= 0.98;
        clamp_dead_band(&mut npc.vx);
        clamp_dead_band(&mut npc.vy);
        return;
    }
    // `:26773-26775` — phase 3: the HitSound write is a documented skip
    // (the static cue table); the damage/defense writes ride the
    // overrides. `defDamage` 45 / `defDefense` 10 are the case-125
    // `SetDefaults` values (`NPC.cs:10527-10528`): (int)(45 * 1.5) = 67,
    // 10 + 10 = 20.
    npc.damage_override = Some((45.0f64 * 1.5) as i32);
    npc.defense_override = Some(10 + 10);
    if npc.ai[1] == 0.0 {
        // `:26778-26789` — the approach: speed 8, accel 0.15 (expert
        // 9.5/0.175, getGood x1.15). NO side offset — straight at the
        // point 300 px above the player.
        let mut num420 = 8.0f32;
        let mut num421 = 0.15f32;
        if env.expert {
            num420 = 9.5;
            num421 = 0.175;
        }
        if env.get_good {
            num420 *= 1.15;
            num421 *= 1.15;
        }
        let (mut cx, mut cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
        let (pcx, pcy) = player.center();
        let mut num422 = pcx - cx;
        let mut num423 = pcy - 300.0 - cy;
        // `:26793-26800` — the queen's orbit anchor at speed 14.
        if env.mech_queen.is_some() {
            num420 = 14.0;
            num422 = vector36.0 - cx;
            num423 = vector36.1 - cy;
        }
        let mut num424 = (num422 * num422 + num423 * num423).sqrt();
        if env.mech_queen.is_some() {
            // `:26802-26812` — the queen's capped 4:1 blend (NOT the
            // phase-0 hover's 60:1).
            if num424 > num420 {
                num424 = num420 / num424;
                num422 *= num424;
                num423 *= num424;
            }
            npc.vx = (npc.vx * 4.0 + num422) / 5.0;
            npc.vy = (npc.vy * 4.0 + num423) / 5.0;
        } else {
            // `:26813-26850` — the classic normalize + approach.
            num424 = num420 / num424;
            num422 *= num424;
            num423 *= num424;
            approach_axis(&mut npc.vx, num422, num421);
            approach_axis(&mut npc.vy, num423, num421);
        }
        // `:26851-26864` — the 300-tick approach (1200 under the queen)
        // flips into the strafe. `TargetClosest()` here — NOT
        // ClearTarget (the phase-0 flip clears; this one re-picks).
        let num425 = if env.mech_queen.is_some() { 1200 } else { 300 };
        npc.ai[2] += 1.0;
        if npc.ai[2] >= num425 as f32 {
            npc.ai[1] = 1.0;
            npc.ai[2] = 0.0;
            npc.ai[3] = 0.0;
            target_closest(npc, world, player);
            npc.net_update = true;
        }
        // `:26865-26868` — NO `else`: the aim re-derives and the
        // rotation SNAPS to the plain atan2 every approach tick (the
        // preamble's 0.1 turn is overridden here), also on the flip.
        cx = npc.x + w * 0.5;
        cy = npc.y + h * 0.5;
        num422 = pcx - cx;
        num423 = pcy - cy;
        npc.rotation = num423.atan2(num422) - 1.57;
        // `:26869-26872` — the netMode == 1 return is constant-false
        // (single-player), folded away.
        // `:26873-26889` — the laser timer with the hp ladder.
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
        // `:26890-26906` — the DeathLaser (100) past 180 behind
        // `Collision.CanHit` (`Collision.cs:381-479`, the port's box
        // walk). NO jitter draws (the module's RNG note).
        if npc.local_ai[1] > 180.0
            && crate::projectile::can_hit(
                world,
                npc.x,
                npc.y,
                w,
                h,
                player.x,
                player.y,
                crate::player::WIDTH,
                player.height,
            )
        {
            npc.local_ai[1] = 0.0;
            // `:26893-26899` — speed 8.5 (expert 10), the
            // `GetAttackDamage_ForProjectiles(25, 23)` endpoint.
            let num426 = if env.expert { 10.0 } else { 8.5 };
            let dmg = if env.expert { 23 } else { 25 };
            num424 = num426 / (num422 * num422 + num423 * num423).sqrt();
            num422 *= num424;
            num423 *= num424;
            // `:26903-26905` — the spawn at center + vel * 15.
            npc.wants_hostile_shot.push((
                crate::projectile::DESTROYER_LASER,
                cx + num422 * 15.0,
                cy + num423 * 15.0,
                num422,
                num423,
                dmg,
            ));
        }
        return;
    }
    // `:26909-27013` — the strafe: EVERY non-zero ai[1] rides this
    // block (the C# has no `else if` guard — states 1, 2 and any other
    // value all strafe; there is no separate phase-3 dash).
    // `:26909-26913` — the side (the same full-width test).
    let mut num429 = 1;
    if npc.x + ((w as i32 / 2) as f32) < player.x + crate::player::WIDTH {
        num429 = -1;
    }
    // `:26914-26925` — speed 8, accel 0.2 (expert 9.5/0.25, getGood
    // x1.15).
    let mut num430 = 8.0f32;
    let mut num431 = 0.2f32;
    if env.expert {
        num430 = 9.5;
        num431 = 0.25;
    }
    if env.get_good {
        num430 *= 1.15;
        num431 *= 1.15;
    }
    // `:26926-26931` — the ±340 side-offset point at the player's
    // centre height, normalized to the speed.
    let (mut cx, mut cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let (pcx, pcy) = player.center();
    let mut num432 = pcx + (num429 * 340) as f32 - cx;
    let mut num433 = pcy - cy;
    let mut num434 = num430 / (num432 * num432 + num433 * num433).sqrt();
    num432 *= num434;
    num433 *= num434;
    // `:26932-26963` — the approach.
    approach_axis(&mut npc.vx, num432, num431);
    approach_axis(&mut npc.vy, num433, num431);
    // `:26964-26967` — the aim re-derives; the rotation snaps to the
    // plain atan2 every strafe tick.
    cx = npc.x + w * 0.5;
    cy = npc.y + h * 0.5;
    num432 = pcx - cx;
    num433 = pcy - cy;
    npc.rotation = num433.atan2(num432) - 1.57;
    // netMode != 1 is constant-true.
    // `:26970-26990` — the laser timer: the fractional hp ladder
    // (+0.5 / +0.75 / +1 / +1.5) and the expert's flat +1.5.
    npc.local_ai[1] += 1.0;
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
        npc.local_ai[1] += 0.5;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
        npc.local_ai[1] += 0.75;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
        npc.local_ai[1] += 1.0;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.1 {
        npc.local_ai[1] += 1.5;
    }
    if env.expert {
        npc.local_ai[1] += 1.5;
    }
    // `:26991-27003` — the DeathLaser (100) past 60 behind CanHit.
    // Speed 9 with NO expert variant; NO jitter draws.
    if npc.local_ai[1] > 60.0
        && crate::projectile::can_hit(
            world,
            npc.x,
            npc.y,
            w,
            h,
            player.x,
            player.y,
            crate::player::WIDTH,
            player.height,
        )
    {
        npc.local_ai[1] = 0.0;
        let num435 = 9.0f32;
        // `GetAttackDamage_ForProjectiles(18, 17)`'s endpoint.
        let dmg = if env.expert { 17 } else { 18 };
        num434 = num435 / (num432 * num432 + num433 * num433).sqrt();
        num432 *= num434;
        num433 *= num434;
        // `:27000-27002` — the spawn at center + vel * 15.
        npc.wants_hostile_shot.push((
            crate::projectile::DESTROYER_LASER,
            cx + num432 * 15.0,
            cy + num433 * 15.0,
            num432,
            num433,
            dmg,
        ));
    }
    // `:27005-27013` — the 180-tick strafe cycle back into the
    // approach, with `TargetClosest()` (not ClearTarget).
    npc.ai[2] += 1.0;
    if npc.ai[2] >= 180.0 {
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
        npc.ai[3] = 0.0;
        target_closest(npc, world, player);
        npc.net_update = true;
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

    fn ret(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Retinazer, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    fn env() -> Style30Env<'static> {
        Style30Env::CLASSIC_SOLO
    }

    /// The `Dust.NewDust` internals census (`Dust.cs:170-190`): position
    /// X then Y `Next(w-4)`/`Next(h-4)`, velocity X then Y
    /// `Next(-20,21)`, `frame.Y Next(3)`, `scale Next(-20,21)` — six draws.
    /// The dimensions pass through the C#'s own `max(_, 5)` clamp.
    fn witness_dust(rng: &mut UnifiedRandom, width: i32, height: i32) {
        let n2 = width.max(5);
        let n3 = height.max(5);
        let _ = rng.next_max(n2 - 4);
        let _ = rng.next_max(n3 - 4);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
    }

    /// The per-tick ambient drip (`:26460-26465`): one `Next(5)` gate,
    /// then (on a hit) one type-5 box dust at `(w, (int)(h*0.5))`.
    fn witness_drip(rng: &mut UnifiedRandom, w: i32, h: f32) {
        if rng.next_max(5) == 0 {
            witness_dust(rng, w, (h * 0.5) as i32);
        }
    }

    /// The per-tick spin dust (`:26760`): the two `Next(-30, 31)` argument
    /// draws, then the NewDust internals — 8 draws per spin tick.
    fn witness_spin_tick(rng: &mut UnifiedRandom, width: i32, height: i32) {
        let _ = rng.next_range(-30, 31);
        let _ = rng.next_range(-30, 31);
        witness_dust(rng, width, height);
    }

    /// W4-e2: the tests' scratch dust seam (the `update_rng` permissive
    /// convention — whole world on-screen, full budget, no cull).
    fn dctx() -> tr_core::particles::SpawnCtx {
        tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(4000.0, 4000.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The SetDefaults identity (`NPC.cs:10522-10537`, `NPCID.cs:10927`
    /// Retinazer = 125) and the aim's named-argument atan2 (`:26412`):
    /// the bottom-59 anchor, the integer halves, the +1.57 offset.
    #[test]
    fn s1_d_3_a30_the_identity_and_the_bottom_anchored_aim() {
        let w = world();
        let p = player(&w);
        let n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        assert_eq!(n.size(), (100.0, 110.0));
        assert_eq!(n.max_hp, 20000);
        assert_eq!(n.kind.ai_style(), 30);
        // Re-derive `num397` independently: atan2(y: B, x: A).
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

    /// The turn (`:26421-26459`): a rotation far from the aim steps
    /// exactly **0.1** (not the Spazmatism's 0.15) along the shortest
    /// path; inside the snap band it lands ON the aim.
    #[test]
    fn s1_d_3_a30_the_turn_steps_the_tenth_and_snaps_the_band() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        let aim = aim_rotation(&n, &p);
        // Park well outside the band, on the short side.
        n.rotation = aim + 1.0;
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert!(
            (n.rotation - (aim + 1.0 - 0.1)).abs() < 1e-5,
            "the turn steps 0.1 toward the aim: {}",
            n.rotation
        );
        // Inside the band after the step: the snap lands on the aim
        // exactly (aim + 0.05 - 0.1 = aim - 0.05, strictly inside).
        // Restore the pre-tick position/velocity so the aim is the
        // same value (the integration tail moved the body).
        n.x = 60.0 * TILE_PX;
        n.y = 60.0 * TILE_PX;
        n.vx = 0.0;
        n.vy = 0.0;
        n.rotation = aim + 0.05;
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.rotation, aim, "the snap band lands on the aim");
    }

    /// RNG witness: the dust gate (`:26460`) draws exactly one
    /// `Next(5)` on a plain hover tick — the stream advances by that
    /// one draw and no other.
    #[test]
    fn s1_d_3_a30_the_dust_gate_draws_one_next_five_per_tick() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(rng.peek(), witness.peek(), "exactly one draw, in order");
    }

    /// The day gate (`:26486-26491`): vy -0.04, the 10-tick despawn
    /// encouragement, no phase advance — and the integration tail
    /// still moves the body (the C# `return` leaves AI() only).
    #[test]
    fn s1_d_3_a30_the_day_flee_rises_and_arms_the_despawn() {
        let mut w = world();
        w.day_time = true;
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        let (x0, y0) = (n.x, n.y);
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.vy, -0.04);
        assert_eq!(n.despawn_timer, 10);
        assert_eq!(n.ai, [0.0; 4], "the day gate precedes the phases");
        assert_eq!((n.x, n.y), (x0, y0 - 0.04), "the tail integrates");
        // The dust gate drew BEFORE the day return.
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_max(5);
        assert_eq!(rng.peek(), witness.peek());
    }

    /// The hover approach (`:26538-26575`): the ±300 side offset, the
    /// -300 vertical offset, the 7-speed normalize, the 0.1 accel with
    /// the sign-reversal double-add.
    #[test]
    fn s1_d_3_a30_the_hover_approaches_the_offset_point() {
        let w = world();
        let p = player(&w);
        // The NPC LEFT of the player: side -1, the hover point 300 px
        // left of and 300 px above the player's centre.
        let mut n = ret(20.0 * TILE_PX, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        let (cx, cy) = (20.0 * TILE_PX + 50.0, 60.0 * TILE_PX + 55.0);
        let (pcx, pcy) = p.center();
        let dx = pcx - 300.0 - cx;
        let dy = pcy - 300.0 - cy;
        let k = 7.0 / (dx * dx + dy * dy).sqrt();
        // vx 0 < target, no reversal: a single 0.1 step.
        assert!((n.vx - 0.1).abs() < 1e-5, "vx {}", n.vx);
        assert!(dx * k > 0.1, "the target outruns the first step");
        assert!(dy * k < -0.1, "the point is above: vy steps down");
        assert!((n.vy - (-0.1)).abs() < 1e-5, "vy {}", n.vy);
        // The reversal double-add: -0.15 + 0.1 = -0.05 is still
        // negative while the target is positive, so the accel lands
        // twice (-0.15 + 0.2 = 0.05).
        n.vx = -0.15;
        n.vy = 0.0;
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert!((n.vx - (-0.15 + 0.2)).abs() < 1e-4, "double-add: {}", n.vx);
    }

    /// The shot gate (`:26592`): the timer only advances when the
    /// NPC's bottom is ABOVE the player's top and the hover point is
    /// within 400 px; below the player it stays frozen.
    #[test]
    fn s1_d_3_a30_the_shot_gate_needs_the_npc_above_the_player() {
        let w = world();
        let p = player(&w);
        // BELOW the player (y 70 tiles, bottom 1230 > player top 960):
        // the timer never advances.
        let mut n = ret(90.0 * TILE_PX, 70.0 * TILE_PX);
        n.ai = [0.0, 0.0, 0.0, 30.0];
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.ai[3], 30.0, "below the player: no timer advance");
        assert!(n.wants_hostile_shot.is_empty());
        // ABOVE the player (y 40 tiles, bottom 750 < 960), hover point
        // ~180 px away (< 400): the timer advances.
        let mut n = ret(90.0 * TILE_PX, 40.0 * TILE_PX);
        n.ai = [0.0, 0.0, 0.0, 30.0];
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.ai[3], 31.0, "above + near: the timer advances");
    }

    /// The 600-tick flip (`:26583-26591`): ai[1]=1, the counters reset,
    /// ClearTarget (255), netUpdate.
    #[test]
    fn s1_d_3_a30_the_six_hundred_tick_hover_flips_into_the_dash() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 0.0, 599.0, 30.0];
        n.target = 0;
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0);
        assert_eq!(n.target, 255, "ClearTarget");
        assert!(n.net_update);
    }

    /// RNG witness + shot shape (`:26618-26640`): at ai[3] 59 above the
    /// player the tick draws the gate then the X-then-Y jitter x0.08;
    /// the eye laser (83) leaves at speed 9 + jitter, damage 20, from
    /// center + vel * 15.
    #[test]
    fn s1_d_3_a30_the_hover_shot_draws_x_then_y_and_fires_the_83() {
        let w = world();
        let p = player(&w);
        let mut n = ret(90.0 * TILE_PX, 40.0 * TILE_PX);
        n.ai = [0.0, 0.0, 0.0, 59.0];
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        let jx = witness.next_range(-40, 41) as f32 * 0.08;
        let jy = witness.next_range(-40, 41) as f32 * 0.08;
        let (cx, cy) = (n.x + 50.0, n.y + 55.0);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(rng.peek(), witness.peek(), "gate + X + Y, in order");
        assert_eq!(n.ai[3], 0.0, "the shot resets the timer");
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::EYE_LASER);
        assert_eq!(dmg, 20, "GetAttackDamage_ForProjectiles(20, 19) classic");
        let (pcx, pcy) = p.center();
        let k = 9.0 / ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt();
        let evx = (pcx - cx) * k + jx;
        let evy = (pcy - cy) * k + jy;
        assert_eq!((vx, vy), (evx, evy));
        assert_eq!((sx, sy), (cx + evx * 15.0, cy + evy * 15.0));
    }

    /// The dash (`:26645-26663`): the aim snap, the direct 12-speed
    /// set (expert 15, getGood +2 ADDITIVE), ai[1]=2 — and NO draws
    /// beyond the dust gate (the C# dash rolls nothing).
    #[test]
    fn s1_d_3_a30_the_dash_sets_the_twelve_speed_and_draws_nothing() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 1.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(rng.peek(), witness.peek(), "the dash draws nothing");
        let speed = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!((speed - 12.0).abs() < 1e-3, "speed {speed}");
        assert_eq!(n.ai[1], 2.0);
        // Expert 15, getGood adds 2 on top (17).
        let mut e = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        e.ai = [0.0, 1.0, 0.0, 0.0];
        let expert = Style30Env { expert: true, get_good: true, ..env() };
        retinazer_ai(&mut e, &w, &p, &mut rng, &expert, &mut tr_core::particles::DustPool::new(), &dctx());
        let speed = (e.vx * e.vx + e.vy * e.vy).sqrt();
        assert!((speed - 17.0).abs() < 1e-3, "expert+getGood speed {speed}");
    }

    /// The dash decay (`:26665-26700`): the 0.96 damp only from the
    /// 25th tick, the velocity rotation before it, the 70-tick cycle's
    /// FOUR-dash wrap.
    #[test]
    fn s1_d_3_a30_the_dash_decays_and_the_cycle_wraps_at_four() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 2.0, 24.0, 0.0];
        n.vx = 10.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert!((n.vx - 9.6).abs() < 1e-5, "the 25th tick damps 0.96: {}", n.vx);
        // Before the 25th: the rotation rides the velocity, no damp.
        n.ai = [0.0, 2.0, 0.0, 0.0];
        n.vx = 10.0;
        n.vy = 0.0;
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.vx, 10.0);
        assert!((n.rotation - (0.0f32.atan2(10.0) - 1.57)).abs() < 1e-5);
        // The 70-tick wrap: dash 4 returns to the hover.
        n.ai = [0.0, 2.0, 69.0, 3.0];
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!((n.ai[1], n.ai[2], n.ai[3]), (0.0, 0.0, 0.0));
        assert_eq!(n.target, 255, "ClearTarget on the cycle");
        // Dash 3 of 4 keeps dashing.
        n.ai = [0.0, 2.0, 69.0, 2.0];
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!((n.ai[1], n.ai[3]), (1.0, 3.0));
    }

    /// The 40% hand-off (`:26702-26709`): the phase resets to the spin
    /// with every counter zeroed and netUpdate armed.
    #[test]
    fn s1_d_3_a30_the_forty_percent_handoff_resets_into_the_spin() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.hp = n.max_hp * 39 / 100;
        n.ai = [0.0, 2.0, 12.0, 3.0];
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.ai, [1.0, 0.0, 0.0, 0.0]);
        assert!(n.net_update);
    }

    /// RNG witness, the spin (`:26734-26770`): a plain spin tick draws
    /// the gate + the 2 dust draws; the 100-tick transition into
    /// ai[0]==2 adds the 52-draw burst BEFORE the per-tick pair; the
    /// transition into ai[0]==3 zeroes ai[2] and draws no burst.
    #[test]
    fn s1_d_3_a30_the_spin_burst_draws_fifty_two_then_the_pair() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [1.0, 0.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(42);
        let (wpc, hpc) = n.size();
        let mut witness = rng.clone();
        witness_drip(&mut witness, wpc as i32, hpc);
        witness_spin_tick(&mut witness, wpc as i32, hpc as i32);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(rng.peek(), witness.peek(), "drip gate + spin pair");
        assert!((n.ai[2] - 0.005).abs() < 1e-6, "the ramp steps 0.005");
        // The transition into phase 2: the 12 gore + 40 burst-argument
        // draws, each dust followed by its own NewDust internals, then
        // the per-tick pair.
        n.ai = [1.0, 99.0, 0.5, 0.0];
        let mut witness = rng.clone();
        witness_drip(&mut witness, wpc as i32, hpc);
        for _ in 0..2 {
            for _ in 0..3 {
                let _ = witness.next_range(-30, 31);
                let _ = witness.next_range(-30, 31);
            }
        }
        for _ in 0..20 {
            let _ = witness.next_range(-30, 31);
            let _ = witness.next_range(-30, 31);
            witness_dust(&mut witness, wpc as i32, hpc as i32);
        }
        witness_spin_tick(&mut witness, wpc as i32, hpc as i32);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(rng.peek(), witness.peek(), "burst + pair, in order");
        // The transition into phase 3: no burst, ai[2] zeroed.
        n.ai = [2.0, 99.0, 0.005, 0.0];
        let mut witness = rng.clone();
        witness_drip(&mut witness, wpc as i32, hpc);
        witness_spin_tick(&mut witness, wpc as i32, hpc as i32);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.ai[0], 3.0);
        assert_eq!(n.ai[2], 0.0, "the phase-3 entry zeroes the spin");
        assert_eq!(rng.peek(), witness.peek(), "no burst into phase 3");
    }

    /// Phase 3's stat writes (`:26773-26774`): damage 67, defense 20,
    /// every tick of the phase.
    #[test]
    fn s1_d_3_a30_phase_three_writes_the_damage_and_defense() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [3.0, 0.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.damage_override, Some(67), "(int)(45 * 1.5)");
        assert_eq!(n.defense_override, Some(20), "10 + 10");
        assert_eq!(n.defense(), 20);
    }

    /// RNG witness + DeathLaser shape (`:26865-26905`): the approach's
    /// rotation snap overrides the preamble turn; at localAI[1] 180
    /// the tick fires the 100 at speed 8.5, damage 25, from center +
    /// vel * 15 — with NO jitter draws (gate only).
    #[test]
    fn s1_d_3_a30_the_hover_laser_fires_the_100_with_no_jitter() {
        let w = world();
        let p = player(&w);
        let mut n = ret(90.0 * TILE_PX, 40.0 * TILE_PX);
        n.ai = [3.0, 0.0, 0.0, 0.0];
        n.local_ai = [0.0, 180.0, 0.0, 0.0];
        let (cx, cy) = (n.x + 50.0, n.y + 55.0);
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(rng.peek(), witness.peek(), "the laser draws NOTHING");
        assert_eq!(n.local_ai[1], 0.0, "the laser resets the timer");
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, sx, sy, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::DESTROYER_LASER);
        assert_eq!(dmg, 25, "GetAttackDamage_ForProjectiles(25, 23) classic");
        let (pcx, pcy) = p.center();
        let k = 8.5 / ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt();
        let evx = (pcx - cx) * k;
        let evy = (pcy - cy) * k;
        assert_eq!((vx, vy), (evx, evy));
        assert_eq!((sx, sy), (cx + evx * 15.0, cy + evy * 15.0));
        // The rotation snapped to the plain atan2 (:26868), not the
        // preamble's 0.1 turn.
        assert_eq!(n.rotation, (pcy - cy).atan2(pcx - cx) - 1.57);
    }

    /// The 300-tick flip (`:26856-26864`): ai[1]=1, the counters reset,
    /// **TargetClosest** (target 0, NOT 255), netUpdate — and the laser
    /// timer still advances on the flip tick (no `else`).
    #[test]
    fn s1_d_3_a30_the_three_hundred_tick_flip_retargets() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [3.0, 0.0, 299.0, 0.0];
        n.target = 255;
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0);
        assert_eq!(n.target, 0, "TargetClosest, not ClearTarget");
        assert!(n.net_update);
        assert_eq!(n.local_ai[1], 1.0, "the timer runs on the flip tick");
    }

    /// The strafe (`:26909-27003`): EVERY non-zero ai[1] rides it; the
    /// fractional hp ladder + the expert's +1.5; at >60 the 100 fires
    /// at speed 9 (no expert variant), damage 18; the 180-tick wrap
    /// re-picks the target.
    #[test]
    fn s1_d_3_a30_the_strafe_ladder_and_the_one_eighty_wrap() {
        let w = world();
        let p = player(&w);
        // ai[1] = 2 (any non-zero value): the strafe runs. hp under
        // 10%: 1 + 0.5 + 0.75 + 1 + 1.5 = 4.75 per tick.
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.hp = n.max_hp / 20; // 5% — under every band
        n.ai = [3.0, 2.0, 0.0, 0.0];
        n.local_ai = [0.0, 60.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(rng.peek(), witness.peek(), "the strafe laser draws NOTHING");
        assert_eq!(n.local_ai[1], 0.0, "60 + 4.75 > 60: fired and reset");
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, _, _, vx, vy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::DESTROYER_LASER);
        assert_eq!(dmg, 18, "GetAttackDamage_ForProjectiles(18, 17) classic");
        let speed = (vx * vx + vy * vy).sqrt();
        assert!((speed - 9.0).abs() < 1e-3, "speed 9, no expert variant");
        // The expert's flat +1.5 on a healthy body: 60 + 1 + 1.5 = 62.5.
        let mut e = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        e.ai = [3.0, 1.0, 0.0, 0.0];
        e.local_ai = [0.0, 58.0, 0.0, 0.0];
        let expert = Style30Env { expert: true, ..env() };
        retinazer_ai(&mut e, &w, &p, &mut rng, &expert, &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(e.local_ai[1], 0.0, "58 + 2.5 > 60: the expert fires");
        assert_eq!(e.wants_hostile_shot[0].5, 17, "expert damage 17");
        // The 180-tick wrap back into the approach.
        n.ai = [3.0, 2.0, 179.0, 0.0];
        n.local_ai = [0.0; 4];
        n.target = 255;
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!((n.ai[1], n.ai[2], n.ai[3]), (0.0, 0.0, 0.0));
        assert_eq!(n.target, 0, "TargetClosest on the wrap");
        assert!(n.net_update);
    }

    /// The CanHit gates (`:26890`, `:26991`): a solid wall between the
    /// boxes stops the DeathLaser — the timer keeps climbing past the
    /// threshold, no shot, no draws beyond the gate.
    #[test]
    fn s1_d_3_a30_the_wall_blocks_the_death_laser() {
        let mut w = world();
        // The wall column (x 1552-1568) sits strictly between the NPC
        // box (1440-1540) and the player box (1600+).
        for y in 30..80 {
            let i = w.index(97, y);
            w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let p = player(&w);
        let mut n = ret(90.0 * TILE_PX, 40.0 * TILE_PX);
        n.ai = [3.0, 0.0, 0.0, 0.0];
        n.local_ai = [0.0, 180.0, 0.0, 0.0];
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _gate = witness.next_max(5);
        retinazer_ai(&mut n, &w, &p, &mut rng, &env(), &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(rng.peek(), witness.peek(), "only the dust gate drew");
        assert_eq!(n.local_ai[1], 181.0, "the timer climbed, no reset");
        assert!(n.wants_hostile_shot.is_empty());
    }

    /// The twin discouragement (`:26466-26475`): at night with an
    /// active timer under 10, the sibling's timeLeft - 1 raises ours;
    /// day / no-timer / healthy-timer ticks leave it alone.
    #[test]
    fn s1_d_3_a30_the_twin_alive_loop_discourages_the_despawn() {
        let w = world();
        let p = player(&w);
        let mut n = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.despawn_timer = 5;
        let twins = Style30Env { twin_time_lefts: &[300, 120], ..env() };
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut n, &w, &p, &mut rng, &twins, &mut tr_core::particles::DustPool::new(), &dctx());
        // 300 - 1 = 299 raises 5; 120 - 1 = 119 cannot lower it.
        assert_eq!(n.despawn_timer, 299);
        // The -1 sentinel: no active timer, no raise.
        n.despawn_timer = -1;
        retinazer_ai(&mut n, &w, &p, &mut rng, &twins, &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.despawn_timer, -1);
        // A healthy timer (>= 10) skips the loop.
        n.despawn_timer = 50;
        retinazer_ai(&mut n, &w, &p, &mut rng, &twins, &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(n.despawn_timer, 50);
    }

    /// The sim feed (S1-D-3-wire-retinazer): `twin_time_left_scan` is
    /// the `num400 != whoAmI` slot scan (`:26468-26474`) — it skips the
    /// eye's own slot, keeps slot order, and the `is_twin` gate drops
    /// the dead/non-twin slots. Feeding the scan's output through the
    /// env raises the despawn timer exactly like the hand-built list.
    #[test]
    fn s1_d_3_a30_the_sim_feed_scans_the_other_twin_slots() {
        let w = world();
        let p = player(&w);
        let mut me = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        me.despawn_timer = 5;
        let mut me_slot = ret(60.0 * TILE_PX, 60.0 * TILE_PX);
        me_slot.despawn_timer = 5;
        let mut sibling = ret(70.0 * TILE_PX, 60.0 * TILE_PX);
        sibling.despawn_timer = 300;
        let mut dead = Npc::new(NpcKind::Spazmatism, 0.0, 0.0, 1, 0.0);
        dead.hp = 0;
        dead.despawn_timer = 999;
        let npcs = vec![dead, me_slot, sibling];
        let is_twin = |n: &Npc| {
            (n.kind == NpcKind::Retinazer || n.kind == NpcKind::Spazmatism) && n.hp > 0
        };
        // Slot 1 = me: the scan sees only slot 2's 300 (slot 0 is dead).
        let list = twin_time_left_scan(&npcs, 1, is_twin);
        assert_eq!(list, vec![300], "own slot skipped, dead slot dropped, slot order");
        let env = Style30Env { twin_time_lefts: &list, ..env() };
        let mut rng = UnifiedRandom::new(7);
        retinazer_ai(&mut me, &w, &p, &mut rng, &env, &mut tr_core::particles::DustPool::new(), &dctx());
        assert_eq!(me.despawn_timer, 299, "the fed list discourages the despawn");
    }
}
