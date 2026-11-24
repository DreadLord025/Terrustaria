//! S1-D-3z (zombie fighter audit, fix B/C) — `AI_003_Fighters`' obstacle
//! scan tail (`NPC.cs:59831-60284`), shared by the zombie family's
//! `update_zombie` machine: the `flag5` ground probe (`:59831-59867`), the
//! 1-tile step-up assist (`:59872-59948`), the door bash (`:59949-60088`)
//! and the probe jump ladder (`:60089-60203`). The scan runs at the C#'s
//! position — after the chase/despawn arms and the `num107` velocity
//! ladder, BEFORE the gravity + collision integration (`AI_003_Fighters`
//! ends at `:60298`; `UpdateCollision` integrates afterwards).
//!
//! The zombie family's folds (documented, ANALYSIS "The zombie fighter
//! audit (S1-D-3z)"):
//! - `flag` (the hop flag, `:59831`'s `| flag`) is set only by the
//!   scarecrow 305-309 arm (`npc_ai3_speeds`' `fx.hop_flag`, dropped by
//!   `run_ai3_ladder` — the pre-existing deferral): false for every zombie
//!   kind, so the ground probe runs iff `velocity.Y == 0`.
//! - `flag6` (`:57019-57026`) = `velocity.X == 0 && !justHit`; `justHit`
//!   rides the port's `hurt_cd > 0` stand-in (the poltergeist convention).
//! - `flag8` (`:57067-57075`) is true for every zombie kind (none is in
//!   `CritterThatCanTurnOnPlayers` or the long type list).
//! - `num57` = 60 (the type-120 row's 180 cannot fire, `:57035-57038`).
//! - The probe column `num198` rides the plain `centre + 15 * direction`
//!   (`:59951`); the wide-body override list (`:59953`) names none of the
//!   zombie types.
//! - The expert-mode / type-586 hop (`:60149-60203`) is skipped: the port
//!   carries no `Main.expertMode` runtime flag and 586 is not a zombie
//!   kind. The `flag6 && ai[3] == 1` re-hop (`:60145-60148`) IS carried.
//! - The type arms after the ladder (`:60205-60277` — the 31/294-296
//!   family, 120's ×1.1, 287's dash, 460's ×1.3) name no zombie type.
//!
//! RNG census (W4-g4 lane 15, folded by the W4-d22 reconcile): the door
//! bash's only draw is `flag25`'s `Main.rand.Next(60)` (`:59997`), guarded
//! by `Main.player[target].ZoneGraveyard`. The zone is NOW WIRED: the sim
//! pre-stamps `Npc::target_zone_graveyard` from
//! `app.scene_metrics.zone_graveyard` (the d120 biome pass; the
//! single-player stand-in for the target's zone, the `wind_speed_current`
//! pre-pass convention), and BOTH scan entries take the caller's
//! `UpdateNPCs` stream. Outside a graveyard the `&&` still short-circuits
//! and ZERO draws are consumed (the legacy census holds for every
//! non-graveyard tick — the axis core stays draw-free there); inside one,
//! each 60-tick bash spends exactly one `Next(60)`. The ladder, the
//! step-up and the ground probe draw nothing. Draw position (the W4-d22
//! retire fold): the C# rolls the moans at the head (`:57340-57367`)
//! BEFORE this scan's bash draw (`:59997`); the port's moan emitter now
//! runs at the sim's PRE-SLOT head position (the C# order — the old
//! sim-tail position that put the bash draw first is retired, the
//! S1-E-21e moan-position remainder CLOSED). The bash's world mutation
//! (`WorldGen.OpenDoor`) rides the
//! `Npc::wants_door_bash` request drained in `world_sim/npcs.rs` (the
//! `wants_sphere` borrow-safe shape); the `KillTile(fail: true)` thud and
//! the `Door_Opened` cue play there on the `UpdateNPCs` stream (the C#
//! plays them inside `NPC.UpdateNPC` under the `Main.cs:17767` swap).
//!
//! Deliberate stand-ins (recorded in ANALYSIS):
//! - `Main.getGoodWorld` — unported, false (the `npc.rs` style-9
//!   precedent); `flag27`'s `(!bloodMoon || getGoodWorld)` folds to
//!   `!bloodMoon`.
//! - `player.insideUnbreakableWalls` (`flag26`, `:60002`) — no Player
//!   field, false; the +6 bash bonus never fires.
//! - `WorldGen.ShiftTallGate` (`:60074`) — NOW PORTED (W4-g4 P4,
//!   `tr_core::edit::shift_tall_gate`): the gate-388 bash shifts the gate
//!   to its open kind (389) in the drain; a refusal sets `ai[3] = num57`
//!   (`:60077`). The bash still counts `ai[1] += 2` and plays the thud.
//! - `WorldGen.ShiftTallGate` (`:60074`) — PORTED (W4-g24,
//!   `edit::shift_tall_gate`); the gate-388 bash's shift + refusal
//!   (`ai[3] = num57`) + the pitched-8 cue ride the `world_sim/npcs.rs`
//!   drain on the UpdateNPCs stream.
//! - `WorldGen.ShiftTallGate` (`:60074`) — NOW PORTED (W4-g4, re-pointed at
//!   the c42 canonical `tr_core::edit::shift_tall_gate` per research/95
//!   §3): the gate-388 bash shifts the gate to its open kind (389) in the
//!   drain; a refusal sets `ai[3] = num57` (`:60077`). The bash still
//!   counts `ai[1] += 2` and plays the thud.
//! - The step-up's `gfxOffY`/`stepSpeed` writes land on the NPC fields,
//!   but the render pass does not read `gfx_off_y` yet and the decay
//!   (`npc_ai_unicorn_b.rs:285`'s helper) is unwired — the lift itself
//!   (the `position.Y` write) is live; the visual offset is the documented
//!   remainder.

use crate::npc::{Npc, NpcKind};
use tr_core::tiledata as td;
use tr_core::{Tile, UnifiedRandom, World};

/// `Tile.nactive()` (`Tile.cs:240-248`): active and NOT actuated — the
/// scan's own probe gate (the `npc_ai_unicorn.rs:41` precedent).
fn nactive(t: &Tile) -> bool {
    t.active() && t.flags & tr_core::tile::FLAG_INACTIVE == 0
}

/// `Main.tileSolid[t.type]` — the generated table.
fn solid_kind(kind: u16) -> bool {
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind)
}

/// `Main.tileSolidTop[t.type]`.
fn solid_top(kind: u16) -> bool {
    td::TILE_SOLID_TOP.has(kind)
}

/// `Tile.topSlope()` (`Tile.cs:290-298`): slope 1 or 2.
fn top_slope(t: &Tile) -> bool {
    let s = t.slope();
    s == 1 || s == 2
}

/// A missing tile reads as the C#'s `new Tile()` (`NPC.cs:59889-59912` /
/// `:59957-59988`'s null-init guards): inactive, type 0.
fn tile_or_blank(world: &World, x: i32, y: i32) -> Tile {
    world.tile(x, y).copied().unwrap_or_default()
}

/// `WorldGen.SolidTileNoPlatforms` (`WorldGen.cs:70196-70218`): an
/// active, non-platform, solid-or-solidTop, non-actuated tile.
/// Out-of-world and null cells count as SOLID (`InWorld`/null → `true` —
/// the vanilla quirk, kept 1:1).
fn solid_tile_no_platforms(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    nactive(t) && !td::SET_PLATFORMS.has(t.kind) && (solid_kind(t.kind) || solid_top(t.kind))
}

/// `WorldGen.SolidTileAllowBottomSlope` (`WorldGen.cs:70167-70189`): an
/// active, solid-or-solidTop, non-halfBrick, non-actuated tile whose top
/// is not a slope. Out-of-world/null → `true` (the same quirk). The
/// `PlatformProperTopFrame` refinement only reaches platforms with a top
/// slope, which cannot occur (the `tr-core/edit/kill.rs:507-520` port's
/// note); the tr-core helper is `pub(crate)`, so this is the app-side
/// twin, byte-identical for every reachable cell.
fn solid_tile_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    nactive(t)
        && (solid_kind(t.kind) || solid_top(t.kind))
        && !top_slope(t)
        && !t.half_brick()
}

/// The `flag24` door-bash type list (`NPC.cs:59996`), verbatim. NOTE 489
/// (BloodZombie) is NOT in the list — flag27 (`:60007`,
/// `(!bloodMoon || getGoodWorld) && !flag25 & flag24`) is false for it
/// on EVERY night, so its `ai[1]` accumulates and it opens doors on a
/// NORMAL night too. The flag24 types (the zombie family) reset `ai[1]`
/// every bash on a normal night and only accumulate on a BLOOD MOON
/// (or a graveyard 1/60 roll, flag25) — the blood-moon door-break is
/// the flag24 types' privilege, not the Blood Zombie's (W4-g4 lane 15
/// residual census, the W4-d22 reconcile fold: the old note had the
/// distinction inverted).
const DOOR_BASH_TYPES: [u16; 35] = [
    3, 691, 430, 590, 331, 332, 132, 161, 186, 187, 188, 189, 200, 223, 320, 321, 319, 21, 324,
    323, 322, 44, 196, 167, 77, 197, 202, 203, 449, 450, 451, 452, 481, 201, 635,
];

/// W4-d22 (style-3 axis) — `flag8` (`NPC.cs:57067-57075`): false for the
/// turn-on-player critters and the long type list, true for every other
/// fighter. The zombie family folds to true (none of its ids is listed —
/// the S1-D-3z module note); the scarecrows 305-309, the Scutlix 391, the
/// Drakomire 415, the Selenian 419 and the Yeti 343 fold to FALSE (their
/// bash gate `:59990` and the no-obstacle resets `:60140`/`:60280` skip).
pub(crate) fn fighter_flag8(t: u16) -> bool {
    if crate::npc_ai3_speeds::TURN_CRITTERS.contains(&t) {
        return false;
    }
    !matches!(
        t,
        343 | 47
            | 67
            | 109
            | 110
            | 111
            | 120
            | 163
            | 164
            | 239
            | 168
            | 199
            | 206
            | 214
            | 215
            | 216
            | 217
            | 218
            | 219
            | 220
            | 226
            | 243
            | 251
            | 257
            | 258
            | 290
            | 291
            | 292
            | 293
            | 305
            | 306
            | 307
            | 308
            | 309
            | 348
            | 349
            | 350
            | 351
            | 379
            | 591
            | 380
            | 381
            | 382
            | 383
            | 386
            | 391
            | 466
            | 464
            | 166
            | 469
            | 468
            | 471
            | 470
            | 480
            | 481
            | 482
            | 411
            | 424
            | 409
            | 425
            | 427
            | 426
            | 428
            | 580
            | 508
            | 415
            | 419
            | 520
            | 528
            | 529
            | 530
            | 532
            | 582
            | 624
            | 631
    ) && !(430..=436).contains(&t)
        && !(449..=452).contains(&t)
        && !(494..=506).contains(&t)
        && !(524..=527).contains(&t)
}

/// W4-d22 — the wide-body ahead-probe override list (`NPC.cs:59953`):
/// these types probe `centre + (width/2 + 16) * direction` instead of the
/// plain `centre + 15 * direction` (`:59951`).
fn wide_probe_body(t: u16) -> bool {
    matches!(
        t,
        109 | 163 | 164 | 199 | 236 | 239 | 257 | 258 | 290 | 391 | 425 | 427 | 426 | 580 | 508
            | 415 | 530 | 532 | 582
    )
}

/// W4-d22 — the post-ladder charge-hop type list (`NPC.cs:60205`): the
/// grounded close-range `vx *= 2` (cap ±3) + `vy = -4` hop. Among the
/// ported axis walkers only the GrayGrunt 385 is listed (389 GigaZapper
/// rides its own deferred shot machine); the 586 expert inner arm
/// (`:60209-60241`) is unported (no expert flag, 586 has no kind).
fn charge_hop_type(t: u16) -> bool {
    matches!(
        t,
        31 | 294 | 295 | 296 | 47 | 77 | 104 | 168 | 196 | 385 | 389 | 464 | 470
    ) || (524..=527).contains(&t)
}

/// `AI_003_Fighters`' obstacle scan (`NPC.cs:59831-60284`) for the zombie
/// family, one fixed step at the C#'s pre-integration position. Returns
/// `true` when the ground probe's `flag23` early-return fired (`:59863-
/// 59866` — a null cell inside the body's column span): the C# returns
/// from `AI_003_Fighters` itself, skipping the step-up/bash/ladder tail;
/// the caller still integrates (the C#'s `UpdateCollision` runs after
/// `AI()` regardless).
///
/// `rng` is the caller's `UpdateNPCs` stream — the ONLY draw the scan can
/// make is the graveyard bash gate's `Next(60)` (`:59997`, W4-g4 lane 15,
/// the W4-d22 signature union); outside a graveyard it is untouched (the
/// zero-draw census).
pub(crate) fn fighter_obstacle_scan(
    npc: &mut Npc,
    world: &World,
    rng: &mut UnifiedRandom,
) -> bool {
    // W4-d22: the zombie fold — flag8 true, the hop flag false, num57 60
    // (the S1-D-3z module docs). The general entry below serves the axis.
    // The player centre rides the target rect (`npc_target`'s
    // SetTargetTrackingValues write); no zombie id is in the charge-hop
    // list, so the window read never fires for this entry.
    let r = npc.target_rect;
    let pc = (
        (r.0 + r.2 / 2) as f32,
        (r.1 + r.3 / 2) as f32,
    );
    fighter_obstacle_scan_g(npc, world, rng, true, false, 60.0, pc)
}

/// W4-d22 (style-3 axis) — the GENERAL scan: `flag8` from the type gate,
/// `hop_flag` the ladder's `fx.hop_flag` (the scarecrow 305-309 arm,
/// `NPC.cs:57847`), `num57` the idle cap (60; the type-120 row's 180),
/// `player_center` the C#'s `Main.player[target].Center` for the
/// charge-hop window (`:60214`). Every type fold the S1-D-3z module
/// documented is now computed from the live type — the zombie family's
/// results are unchanged (flag8 true for all 17 ids, none in the
/// wide-probe/step-up-reach/charge-hop lists).
///
/// W4-d22 reconcile — the signature UNION with W4-g4 lane 15: `rng` is the
/// caller's `UpdateNPCs` stream (g4b's threading), the type folds are the
/// axis generalisation (d22's). The only draw is the graveyard bash gate
/// (`:59997`); outside a graveyard the scan stays draw-free.
pub(crate) fn fighter_obstacle_scan_g(
    npc: &mut Npc,
    world: &World,
    rng: &mut UnifiedRandom,
    flag8: bool,
    hop_flag: bool,
    _num57: f32,
    player_center: (f32, f32),
) -> bool {
    let (w, h) = npc.size();
    let t = npc.kind.cs_id();
    // `:57019-57026` — flag6: parked and not just-hit.
    let flag6 = npc.vx == 0.0 && npc.hurt_cd <= 0.0;
    // `:59831-59867` — the flag5 ground probe. The gate is
    // `(velocity.Y == 0) | flag` — the scarecrow hop flag rides along.
    let mut flag5 = false;
    if npc.vy == 0.0 || hop_flag {
        // `:59833-59838` — the C#'s `(int)` casts truncate toward zero
        // (positions are non-negative in-world; the cast keeps the shape).
        let num185 = (npc.y + h + 7.0) as i32 / 16;
        let num186 = (npc.y - 9.0) as i32 / 16;
        let num187 = npc.x as i32 / 16;
        let num188 = (npc.x + w) as i32 / 16;
        let num189 = (npc.x + 8.0) as i32 / 16;
        let num190 = (npc.x + w - 8.0) as i32 / 16;
        let mut flag23 = false;
        for num191 in num189..=num190 {
            let tile = world.tile(num191, num185);
            let tile2 = world.tile(num191, num186);
            // `:59844-59848` — a null cell inside the body span: flag23,
            // continue (the head probe below is skipped for this column).
            if num191 >= num187 && num191 <= num188 && tile.is_none() {
                flag23 = true;
                continue;
            }
            // `:59849-59853` — a solid head-room cell kills flag5 and
            // breaks the loop.
            if tile2.is_some() && solid_tile_no_platforms(world, num191, num186) {
                flag5 = false;
                break;
            }
            // `:59854-59857` — a solid floor cell under the body sets it.
            if !flag23
                && num191 >= num187
                && num191 <= num188
                && tile.is_some_and(|t| nactive(t) && solid_kind(t.kind))
            {
                flag5 = true;
            }
        }
        // `:59859-59862` — no floor and rising: the jump is cancelled.
        if !flag5 && npc.vy < 0.0 {
            npc.vy = 0.0;
        }
        // `:59863-59866` — the null-cell early return.
        if flag23 {
            return true;
        }
    }
    // `:59868-59871` — the type-428 flag5 reset (the VortexHeadcrab).
    if t == 428 {
        flag5 = false;
    }
    // `:59872-59948` — the 1-tile step-up assist (the unicorn machine's
    // inline copy shape, `npc_ai_unicorn.rs:328-387`, but the AI_003
    // probe rows). Only while falling or level; the type-580 directionY
    // gate (`:59872`) is live for the axis kinds.
    if npc.vy >= 0.0 && (t != 580 || npc.direction_y != 1) {
        let num192 = if npc.vx < 0.0 {
            -1i32
        } else if npc.vx > 0.0 {
            1
        } else {
            0
        };
        // `:59883-59886` — the probe at the NEXT x position.
        let vx_pos = npc.x + npc.vx;
        let num193 =
            ((vx_pos + (w as i32 / 2) as f32 + ((w as i32 / 2 + 1) * num192) as f32) / 16.0) as i32;
        let num194 = ((npc.y + h - 1.0) / 16.0) as i32;
        // `:59887` — `WorldGen.InWorld(num193, num194, 4)`: the 4-tile
        // margin gate, verbatim.
        if num193 >= 4
            && num194 >= 4
            && num193 < world.width as i32 - 4
            && num194 < world.height as i32 - 4
        {
            let t0 = tile_or_blank(world, num193, num194);
            let t1 = tile_or_blank(world, num193, num194 - 1);
            let t2 = tile_or_blank(world, num193, num194 - 2);
            let t3 = tile_or_blank(world, num193, num194 - 3);
            let t4 = tile_or_blank(world, num193 - num192, num194 - 3);
            let tm4 = tile_or_blank(world, num193, num194 - 4);
            // `:59913` — the ledge gate, verbatim conjunct order.
            let base_ok = (nactive(&t0)
                && !top_slope(&t0)
                && !top_slope(&t1)
                && solid_kind(t0.kind)
                && !solid_top(t0.kind))
                || (t1.half_brick() && nactive(&t1));
            let head_clear = !nactive(&t1)
                || !solid_kind(t1.kind)
                || solid_top(t1.kind)
                || (t1.half_brick()
                    && (!nactive(&tm4) || !solid_kind(tm4.kind) || solid_top(tm4.kind)));
            let body_clear = (!nactive(&t2) || !solid_kind(t2.kind) || solid_top(t2.kind))
                && (!nactive(&t3) || !solid_kind(t3.kind) || solid_top(t3.kind))
                && (!nactive(&t4) || !solid_kind(t4.kind));
            if (num193 as f32 * 16.0) < vx_pos + w
                && (num193 as f32 * 16.0 + 16.0) > vx_pos
                && base_ok
                && head_clear
                && body_clear
            {
                // `:59915-59945` — the lift: half-brick offsets, the 16.1
                // reach (+8 for the 163/164/236/239/530 list, `:59928`),
                // the gfxOffY record and the 1/2 step speed.
                let mut num195 = num194 as f32 * 16.0;
                if t0.half_brick() {
                    num195 += 8.0;
                }
                if t1.half_brick() {
                    num195 -= 8.0;
                }
                if num195 < npc.y + h {
                    let num196 = npc.y + h - num195;
                    let mut num197 = 16.1f32;
                    if matches!(t, 163 | 164 | 236 | 239 | 530) {
                        num197 += 8.0;
                    }
                    if num196 <= num197 {
                        npc.gfx_off_y += npc.y + h - num195;
                        npc.y = num195 - h;
                        npc.step_speed = if num196 < 9.0 { 1.0 } else { 2.0 };
                    }
                }
            }
        }
    }
    if flag5 {
        // `:59951-59956` — the ahead probe: centre + 15 * direction, the
        // body's bottom - 15 row; the wide-body override list switches to
        // centre + (width/2 + 16) * direction.
        let reach = if wide_probe_body(t) {
            (w as i32 / 2 + 16) as f32
        } else {
            15.0
        };
        let num198 = ((npc.x + (w as i32 / 2) as f32 + reach * npc.facing as f32) / 16.0) as i32;
        let num199 = ((npc.y + h - 15.0) / 16.0) as i32;
        let ahead = tile_or_blank(world, num198, num199 - 1);
        // `:59990` — the door (10) / tall gate (388) bash gate.
        if nactive(&ahead) && (ahead.kind == 10 || ahead.kind == 388) && flag8 {
            // `:59992-59993` — the bash counter ticks, the idle counter
            // resets.
            npc.ai[2] += 1.0;
            npc.ai[3] = 0.0;
            if npc.ai[2] >= 60.0 {
                // `:59996` — flag24: the door-bash type list.
                let flag24 = DOOR_BASH_TYPES.contains(&npc.kind.cs_id());
                // `:59997` — flag25: `ZoneGraveyard && rand.Next(60) == 0`.
                // W4-g4 lane 15 (the d22 reconcile fold): the zone rides
                // the sim's pre-stamp (`target_zone_graveyard` ←
                // `scene_metrics.zone_graveyard`, the d120 biome pass).
                // Outside a graveyard the `&&` short-circuits and the
                // draw is NEVER consumed (the legacy zero-draw census);
                // inside one, the bash spends exactly one Next(60) on
                // the UpdateNPCs stream. flag25 feeds flag27 below: a
                // graveyard 1/60 roll cancels the normal-night ai[1]
                // reset, so even the flag24 types accumulate toward the
                // open on that tick.
                let flag25 = npc.target_zone_graveyard && rng.next_max(60) == 0;
                // `:59998-60006` — flag26: the target inside unbreakable
                // walls. No Player field — false (module docs).
                let flag26 = false;
                // `:60007-60011` — flag27: the non-blood-moon reset gate.
                // `Main.getGoodWorld` is unported (false — the style-9
                // precedent), so `(!bloodMoon || getGoodWorld)` folds to
                // `!bloodMoon`.
                let mut flag27 = (!world.blood_moon) && !flag25 && flag24;
                if flag26 {
                    flag27 = false;
                }
                if flag27 {
                    npc.ai[1] = 0.0;
                }
                // `:60016` — the bash push-back.
                npc.vx = 0.5 * (-npc.facing) as f32;
                // `:60017-60022` — +5 per door bash, +2 per gate bash.
                let num200 = if ahead.kind == 388 { 2.0 } else { 5.0 };
                npc.ai[1] += num200;
                // `:60023-60034` — the +6 unbreakable bonus (flag26, never
                // fires), the type-27 GoblinThief +1 and the 31/294-296
                // AngryBones-family +6 (`else if` chain — the W4-g4 FINAL
                // census, `55840d9a`: NOT the mummies 78-80/630; the
                // AngryBones kinds are unported, the cs_id gate keeps the
                // arm verbatim for their landing).
                if flag26 {
                    npc.ai[1] += 6.0;
                } else if t == 27 {
                    npc.ai[1] += 1.0;
                } else if matches!(t, 31 | 294 | 295 | 296) {
                    npc.ai[1] += 6.0;
                }
                // `:60035` — the bash counter resets.
                npc.ai[2] = 0.0;
                // `:60036-60045` — at 10 the door opens; the type-460
                // always-open arm (`:60042-60045`).
                let mut flag28 = false;
                if npc.ai[1] >= 10.0 {
                    flag28 = true;
                    npc.ai[1] = 10.0;
                }
                if t == 460 {
                    flag28 = true;
                }
                // `:60046` — the failure thud (`KillTile(fail: true)`)
                // plays on EVERY bash; the open request rides the drain
                // (`world_sim/npcs.rs`), which plays the thud + the
                // `Door_Opened` cue on the UpdateNPCs stream.
                npc.wants_door_bash = Some((num198, num199 - 1, npc.facing, flag28));
                // `:60047-60086` — the open itself: the port is the host
                // (netMode 0), so the `(netMode != 1 || !flag28) && flag28
                // && netMode != 1` gate folds to `flag28`. The type-26
                // KillTile arm names no zombie kind. The door (10) open
                // rides `edit::open_door` in the drain; a refusal sets
                // `ai[3] = num57` (`:60064`) there. The tall gate (388)
                // `ShiftTallGate` rides `edit::shift_tall_gate` in the
                // drain (W4-g4 P4); a refusal sets `ai[3] = num57`
                // (`:60077`).
                // `ShiftTallGate` is PORTED (W4-g24) — the drain shifts it
                // and plays the pitched 8; the bash counted +2 above.
                // `ShiftTallGate` rides `edit::shift_tall_gate` in the drain
                // (W4-g4); a refusal sets `ai[3] = num57` (`:60077`).
            }
        } else {
            // `:60089-60203` — the probe jump ladder.
            // `:60091-60095` — num201 = spriteDirection; the type-425
            // VortexRifleman flips it (the flight aim faces backwards).
            let num201 = if t == 425 { -npc.sprite_dir } else { npc.sprite_dir };
            // `:60096` — only while moving WITH the facing.
            if (npc.vx < 0.0 && num201 == -1) || (npc.vx > 0.0 && num201 == 1) {
                // `:60098-60110` — the 2/3-tile wall tiers (height >= 32).
                if h >= 32.0 && solid_tile_no_platforms(world, num198, num199 - 2) {
                    if solid_tile_no_platforms(world, num198, num199 - 3) {
                        npc.vy = -8.0;
                        npc.net_update = true;
                    } else {
                        npc.vy = -7.0;
                        npc.net_update = true;
                    }
                } else if solid_tile_no_platforms(world, num198, num199 - 1) {
                    // `:60111-60128` — the 1-tile wall: -6f; the type-624
                    // gnome arm (`:60113-60122`) jumps -8 and turns away
                    // from a low ceiling — 624 has no port kind, the arm
                    // stays documented-unreachable.
                    npc.vy = -6.0;
                    npc.net_update = true;
                } else if npc.y + h - (num199 as f32 * 16.0) > 20.0
                    && !top_slope(&tile_or_blank(world, num198, num199))
                    && solid_tile_no_platforms(world, num198, num199)
                {
                    // `:60129-60133` — the sunken-floor hop: -5f.
                    npc.vy = -5.0;
                    npc.net_update = true;
                } else if npc.direction_y < 0
                    && t != 67
                    && !solid_tile_allow_bottom_slope(world, num198, num199 + 1)
                    && !solid_tile_allow_bottom_slope(world, num198 + npc.facing, num199 + 1)
                {
                    // `:60134-60139` — the gap leap: -8f and vx ×1.5 (the
                    // type-67 face-monster exclusion, `:60134`).
                    npc.vy = -8.0;
                    npc.vx *= 1.5;
                    npc.net_update = true;
                } else if flag8 {
                    // `:60140-60144` — no obstacle: the bash counters
                    // reset.
                    npc.ai[1] = 0.0;
                    npc.ai[2] = 0.0;
                }
                // `:60145-60148` — the parked re-hop: flag6 && ai[3] == 1.
                if npc.vy == 0.0 && flag6 && npc.ai[3] == 1.0 {
                    npc.vy = -5.0;
                }
                // `:60149-60203` — the expert-mode / type-586 hop: skipped
                // (no expertMode flag, 586 has no port kind — the S1-D-3z
                // module docs; the -7.9 platform hop is inside the gate).
            }
            // `:60205-60257` — the charge hop: grounded, inside the
            // 100×50 window, moving with the facing at >= 1: vx ×2 (cap
            // ±3) + vy -4. The 586 inner arm is expert-only (skipped).
            if charge_hop_type(t) && npc.vy == 0.0 {
                let (px, py) = (npc.x + (w as i32 / 2) as f32, npc.y + (h as i32 / 2) as f32);
                let (tx, ty) = player_center;
                if (px - tx).abs() < 100.0
                    && (py - ty).abs() < 50.0
                    && ((npc.facing > 0 && npc.vx >= 1.0) || (npc.facing < 0 && npc.vx <= -1.0))
                {
                    npc.vx *= 2.0;
                    if npc.vx > 3.0 {
                        npc.vx = 3.0;
                    }
                    if npc.vx < -3.0 {
                        npc.vx = -3.0;
                    }
                    npc.vy = -4.0;
                    npc.net_update = true;
                }
            }
            // `:60258-60261` — the type-120 rising ×1.1 (no port kind).
            // `:60262-60272` — the type-287 dash (no port kind).
            // `:60273-60277` — the type-460 rising boost (no port kind).
        }
    } else if flag8 {
        // `:60280-60284` — no floor under the body: the bash counters
        // reset.
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
    }
    // `:60285-60297` — the type-120 teleport: no port kind.
    false
}

/// The `AI_003_Fighters` head (`NPC.cs:55858-55861`): the player's bottom
/// edge exactly level with the NPC's flips `directionY` up. Runs at the
/// machine's entry, before the chase gate.
pub(crate) fn fighter_head_direction_y(npc: &mut Npc, player_bottom: f32) {
    let (_, h) = npc.size();
    if player_bottom == npc.y + h {
        npc.direction_y = -1;
    }
}

/// The zombie family gate for the scan — every kind `update_zombie`
/// serves (the `npc.rs` dispatch arm's list). Kept here so the drain and
/// the witnesses share one source.
pub(crate) fn is_fighter_zombie(kind: NpcKind) -> bool {
    matches!(
        kind,
        NpcKind::Zombie
            | NpcKind::DoctorBones
            | NpcKind::TheGroom
            | NpcKind::BaldZombie
            | NpcKind::ZombieEskimo
            | NpcKind::PincushionZombie
            | NpcKind::SlimedZombie
            | NpcKind::SwampZombie
            | NpcKind::TwiggyZombie
            | NpcKind::FemaleZombie
            | NpcKind::ZombieRaincoat
            | NpcKind::ZombieDoctor
            | NpcKind::ZombieSuperman
            | NpcKind::ZombiePixie
            | NpcKind::ZombieXmas
            | NpcKind::ZombieSweater
            | NpcKind::BloodZombie
    )
}

#[cfg(test)]
mod tests {
    //! S1-D-3z (zombie fighter audit) — tests-as-code, WRITTEN NOT RUN
    //! (code-only mode — no compiler invoked). Every assertion pins a
    //! constant to its C# literal (`NPC.cs:59831-60284`).
    use super::*;
    use crate::npc::Npc;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::{World, TILE_PX};

    /// A 64x64 world, dirt ground at row 30 (the `npc.rs` tests' shape).
    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn solid(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// A zombie standing on the ground at column `tx`, feet on row 30's
    /// top (y = 440), facing +1, moving right.
    fn zombie_at(w: &World, tx: i32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, tx as f32 * TILE_PX + 8.0 - 9.0, 440.0, 1, 0.0);
        n.vx = 1.0;
        n.sprite_dir = 1;
        n.facing = 1;
        n
    }

    /// The probe jump ladder (`NPC.cs:60098-60133`): a 3-tile wall fires
    /// -8, a 2-tile wall -7, a 1-tile wall -6. The probe column is
    /// `centre + 15 * direction` (`:59951`), the probe row `bottom - 15`
    /// (`:59952`). The zombie at column 20 (x = 319, centre 328) probes
    /// column (328 + 15)/16 = 21, row (440 + 40 - 15)/16 = 29; the wall
    /// cells are rows 28/27/26 (num199 - 1/-2/-3).
    #[test]
    fn the_jump_ladder_fires_the_three_wall_tiers() {
        for (wall_rows, want) in [(&[28][..], -6.0), (&[28, 27][..], -7.0), (&[28, 27, 26][..], -8.0)] {
            let mut w = ground();
            for &r in wall_rows {
                solid(&mut w, 21, r);
            }
            let mut n = zombie_at(&w, 20);
            fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
            assert_eq!(n.vy, want, "wall rows {wall_rows:?}");
            assert!(n.net_update, "the ladder arms netUpdate");
        }
    }

    /// The step-up assist (`NPC.cs:59872-59948`): a ONE-tile ledge with
    /// clear head room lifts the body onto it (the `position.Y` write),
    /// records `gfxOffY` and sets `stepSpeed` 2 (a 16px lift, >= 9) — it
    /// does NOT jump. The ledge is the step-up probe column's row-29
    /// cell (the body's bottom row), one tile above the ground the feet
    /// rest on.
    #[test]
    fn the_one_tile_ledge_is_stepped_not_jumped() {
        let mut w = ground();
        // The step-up probe: num193 = (vx_pos + width/2 + (width/2+1)*dir)/16.
        // At x = 319, vx 1: vx_pos 320, num193 = (320+9+10)/16 = 21;
        // num194 = (440+40-1)/16 = 29. The ledge cell (21, 29) is solid,
        // (21, 28) clear — a 1-tile step. (The jump ladder's probe column
        // is 22, clear, so it does not fire.)
        solid(&mut w, 21, 29);
        let mut n = zombie_at(&w, 20);
        n.x = 319.0;
        let before_y = n.y;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        // Lifted onto the ledge top (row 29's top = 464): y = 464 - 40.
        assert_eq!(n.y, 464.0 - 40.0, "the step-up lift");
        assert_eq!(n.gfx_off_y, 16.0, "gfxOffY records the 16px lift");
        assert_eq!(n.step_speed, 2.0, "a >=9px lift sets stepSpeed 2");
        assert_eq!(n.vy, 0.0, "no jump on a stepped ledge");
        assert!(n.y < before_y, "the body rose");
    }

    /// The door bash (`NPC.cs:59990-60086`): a closed door (tile 10) in
    /// the probe column ticks `ai[2]` each scan; at 60 the bash fires —
    /// `vx = 0.5 * -direction`, `ai[1] += 5`, `ai[2]` resets, and the
    /// `wants_door_bash` request carries the cell + direction. On a
    /// non-blood-moon night the zombie (type 3, in flag24) resets `ai[1]`
    /// first (flag27), so it never reaches 10; on a blood moon flag27 is
    /// false and `ai[1]` accumulates to the open threshold.
    #[test]
    fn the_door_bash_counts_and_requests_the_open() {
        let mut w = ground();
        // The door at the probe cell (21, 28) — num199 - 1.
        let i = w.index(21, 28);
        w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut n = zombie_at(&w, 20);
        // Blood moon: flag27 false, ai[1] accumulates.
        w.blood_moon = true;
        n.ai[2] = 59.0; // the next scan hits 60 → the bash fires.
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        assert_eq!(n.ai[2], 0.0, "the bash counter resets");
        assert_eq!(n.ai[1], 5.0, "+5 per door bash");
        assert_eq!(n.vx, -0.5, "the bash push-back");
        let (tx, ty, dir, open) = n.wants_door_bash.expect("the bash request");
        assert_eq!((tx, ty, dir), (21, 28, 1), "the door cell + direction");
        assert!(!open, "ai[1] = 5 < 10: not yet open");
        // A second bash (ai[1] 5 → 10) arms the open.
        n.ai[2] = 59.0;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        assert_eq!(n.ai[1], 10.0, "ai[1] caps at 10");
        assert!(n.wants_door_bash.expect("the second request").3, "the open armed");
    }

    /// The blood-moon gate (`NPC.cs:60007`): on a NORMAL night the zombie
    /// (type 3, in flag24) takes flag27 → `ai[1] = 0` before the +5, so it
    /// never accumulates. The Blood Zombie (489, NOT in flag24) never
    /// takes flag27 — it accumulates on a normal night too (the W4-g4
    /// residual census corrected the inverted note).
    #[test]
    fn the_blood_moon_gate_resets_the_bash_counter() {
        let mut w = ground();
        let i = w.index(21, 28);
        w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.blood_moon = false; // normal night
        let mut n = zombie_at(&w, 20);
        n.ai[1] = 8.0;
        n.ai[2] = 59.0;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        // flag27 resets ai[1] to 0, then +5 → 5 (not 13).
        assert_eq!(n.ai[1], 5.0, "the normal-night reset");
        assert!(!n.wants_door_bash.expect("the request").3, "never opens on a normal night");
    }

    /// W4-g4 lane 15 residual (the W4-d22 reconcile fold) — the Blood
    /// Zombie's NORMAL-night door open (`NPC.cs:59996`/`:60007`): 489 is
    /// NOT in flag24, so flag27 is false for it on every night — two
    /// bashes arm the open even without a blood moon. The flag24 zombie
    /// twin resets (the test above). This is the census's "fighters also
    /// open doors normally" arm: the non-flag24 fighters, not the zombie
    /// family. WRITTEN NOT RUN.
    #[test]
    fn w4_g4_the_blood_zombie_opens_doors_on_a_normal_night() {
        let mut w = ground();
        let i = w.index(21, 28);
        w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.blood_moon = false; // NORMAL night — no blood moon needed
        let mut n = zombie_at(&w, 20);
        n.kind = NpcKind::BloodZombie;
        n.ai[1] = 8.0;
        n.ai[2] = 59.0;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        // No flag27 reset: 8 + 5 → cap 10, the open armed on the FIRST
        // bash (the flag24 zombie would reset to 5).
        assert_eq!(n.ai[1], 10.0, "no reset — the non-flag24 accumulation");
        assert!(n.wants_door_bash.expect("the request").3, "the open armed");
        // The flag24 control on the same fixture: the zombie resets.
        let mut z = zombie_at(&w, 20);
        z.ai[1] = 8.0;
        z.ai[2] = 59.0;
        fighter_obstacle_scan(&mut z, &w, &mut UnifiedRandom::new(0));
        assert_eq!(z.ai[1], 5.0, "the flag24 zombie resets on a normal night");
        assert!(!z.wants_door_bash.expect("the request").3);
    }

    /// W4-d22 g4d (the g4 FINAL fold, `55840d9a`) — the type-bonus bash
    /// arms (`NPC.cs:60027-60030`): the GoblinThief (27, NOT in flag24)
    /// adds +1 per bash. The Thief accumulates on a normal night
    /// (non-flag24) and reaches the open in TWO bashes (6 + 6 = 12 →
    /// cap 10), one fewer than the plain +5 fighter. WRITTEN NOT RUN.
    /// (The g4 witness's door row corrected for the port: the Thief's
    /// 18×38 box — `npc/data.rs` `SetDefaults` case 27 — puts the probe
    /// row at `num199 - 1 = (440 + 38 - 15)/16 - 1 = 27`, not the
    /// zombie-box row 28.)
    #[test]
    fn w4_g4_the_goblin_thief_bash_adds_one() {
        let mut w = ground();
        let i = w.index(21, 27);
        w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.blood_moon = false; // normal night — the Thief is NOT in flag24
        let mut n = zombie_at(&w, 20);
        n.kind = NpcKind::GoblinThief;
        n.ai[2] = 59.0;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        assert_eq!(n.ai[1], 6.0, "+5 door bash +1 Thief bonus");
        n.ai[2] = 59.0;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        assert_eq!(n.ai[1], 10.0, "6 + 6 caps at 10");
        assert!(n.wants_door_bash.expect("the second request").3, "the open armed in two bashes");
    }

    /// W4-d22 g4d (the g4 FINAL fold, `55840d9a`) — the +6 arm's census
    /// correction (`NPC.cs:60031-60034`): the arm names the AngryBones
    /// family 31/294-296 ONLY — NOT the Skeleton 21. The Skeleton (in
    /// flag24) takes the normal-night reset then the plain +5; the
    /// AngryBones kinds are unported (the d38 lane rows 294-296), so the
    /// scan-level control pins the negative census. WRITTEN NOT RUN.
    #[test]
    fn w4_g4_the_skeleton_family_bash_bonus_census() {
        let mut w = ground();
        let i = w.index(21, 28);
        w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.blood_moon = false; // normal night — the Skeleton IS in flag24
        let mut n = zombie_at(&w, 20);
        n.kind = NpcKind::Skeleton;
        n.ai[1] = 8.0;
        n.ai[2] = 59.0;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        // flag27 resets to 0, then the plain +5 (21 is NOT in the
        // 31/294-296 +6 arm — the census correction).
        assert_eq!(n.ai[1], 5.0, "the Skeleton takes the plain +5 reset");
        assert!(!n.wants_door_bash.expect("the request").3);
    }

    /// The else-flag8 reset (`NPC.cs:60280-60284`): with no floor under
    /// the body (flag5 false) the bash counters reset. The body floats
    /// ABOVE the ground (y = 400, bottom 440 = row 30's top): the floor
    /// probe row (bottom + 7)/16 = 27 is air, so flag5 stays false.
    #[test]
    fn the_no_floor_arm_resets_the_bash_counters() {
        let w = ground();
        let mut n = zombie_at(&w, 20);
        n.y = 400.0; // floating: the probe row 27 is air, flag5 false
        n.vy = 0.0;
        n.ai[1] = 7.0;
        n.ai[2] = 3.0;
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(0));
        assert_eq!((n.ai[1], n.ai[2]), (0.0, 0.0), "the no-floor reset");
    }

    /// The head directionY refresh (`NPC.cs:55858-55861`): the player's
    /// bottom edge exactly level with the zombie's flips directionY up.
    #[test]
    fn the_head_flips_direction_y_on_bottom_equality() {
        let w = ground();
        let mut n = zombie_at(&w, 20);
        n.direction_y = 1;
        // The zombie's bottom = y + 40 = 480.
        fighter_head_direction_y(&mut n, 480.0);
        assert_eq!(n.direction_y, -1, "bottom equality flips up");
        n.direction_y = 1;
        fighter_head_direction_y(&mut n, 481.0);
        assert_eq!(n.direction_y, 1, "no equality: unchanged");
    }

    // ---- W4-g4 lane 15 (the W4-d22 reconcile fold) — the graveyard bash
    // gate (`NPC.cs:59997`). WRITTEN NOT RUN (code-only mode). The zone
    // rides the sim's pre-stamp (`Npc::target_zone_graveyard`); the
    // witnesses set it directly (the solo convention). The door fixture
    // mirrors `the_door_bash_counts_and_requests_the_open`: a closed door
    // (10) at the probe cell (21, 28), ai[2] = 59 so the next scan fires
    // the 60-tick bash.

    /// A blood-moon door fixture ready to bash (ai[2] = 59 → 60).
    fn bash_ready() -> (World, Npc) {
        let mut w = ground();
        let i = w.index(21, 28);
        w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.blood_moon = true; // flag27 false: ai[1] accumulates either way
        let mut n = zombie_at(&w, 20);
        n.ai[2] = 59.0;
        (w, n)
    }

    /// Outside a graveyard the `&&` short-circuits: the bash fires but
    /// the stream is UNTOUCHED (the legacy zero-draw census, now pinned
    /// against the live wiring — a regression that drops the zone guard
    /// would spend a Next(60) here).
    #[test]
    fn w4_g4_outside_a_graveyard_the_bash_draws_nothing() {
        let (w, mut n) = bash_ready();
        n.target_zone_graveyard = false;
        let mut rng = UnifiedRandom::new(7);
        let pristine = rng.clone();
        fighter_obstacle_scan(&mut n, &w, &mut rng);
        assert!(n.wants_door_bash.is_some(), "the bash still fires");
        assert_eq!(rng, pristine, "the zone guard short-circuits the draw");
    }

    /// Inside a graveyard the bash spends EXACTLY ONE Next(60) on the
    /// caller stream — the stream-clone replay pins the draw shape and
    /// its position (after the counter tick, before the push-back).
    #[test]
    fn w4_g4_in_a_graveyard_the_bash_spends_one_next_60() {
        let (w, mut n) = bash_ready();
        n.target_zone_graveyard = true;
        let mut rng = UnifiedRandom::new(7);
        let mut mirror = rng.clone();
        let _want = mirror.next_max(60); // the ONE cited draw (:59997)
        fighter_obstacle_scan(&mut n, &w, &mut rng);
        assert!(n.wants_door_bash.is_some(), "the bash fires");
        assert_eq!(rng, mirror, "exactly one Next(60), nothing else");
    }

    /// flag25's effect (`:60007`): on a NORMAL night the zombie (type 3,
    /// in flag24) takes flag27 → ai[1] resets before the +5. A graveyard
    /// Next(60) == 0 sets flag25, cancelling flag27 — the pre-existing
    /// ai[1] = 8 survives and the +5 reaches 13 → capped at 10, the open
    /// armed. The seed search finds a first Next(60) == 0 (the town
    /// door-gate witness convention, `npc_ai_town.rs`).
    #[test]
    fn w4_g4_the_graveyard_roll_cancels_the_normal_night_reset() {
        let (mut w, mut n) = bash_ready();
        w.blood_moon = false; // the normal-night flag27 reset domain
        n.target_zone_graveyard = true;
        n.ai[1] = 8.0;
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(60) == 0 {
                break;
            }
            seed += 1;
        }
        fighter_obstacle_scan(&mut n, &w, &mut UnifiedRandom::new(seed));
        // flag25 true → flag27 false: no reset, 8 + 5 = 13 → cap 10.
        assert_eq!(n.ai[1], 10.0, "the graveyard roll keeps the accumulation");
        assert!(n.wants_door_bash.expect("the request").3, "the open armed");
        // The control: the same fixture with a NON-zero roll resets
        // (flag27) → 0 + 5 = 5, no open.
        let (mut w2, mut n2) = bash_ready();
        w2.blood_moon = false;
        n2.target_zone_graveyard = true;
        n2.ai[1] = 8.0;
        let mut nonzero = seed + 1;
        while UnifiedRandom::new(nonzero).next_max(60) == 0 {
            nonzero += 1;
        }
        fighter_obstacle_scan(&mut n2, &w2, &mut UnifiedRandom::new(nonzero));
        assert_eq!(n2.ai[1], 5.0, "the non-zero roll keeps the normal-night reset");
        assert!(!n2.wants_door_bash.expect("the request").3, "no open");
    }

    /// W4-d22 reconcile — the GENERAL entry (`fighter_obstacle_scan_g`,
    /// the axis core's scan) carries the SAME graveyard gate: the
    /// zombie-fold parameters (flag8 true, hop false, num57 60) replay
    /// the g4b stream contract through the union signature. Outside the
    /// zone the axis core stays draw-free (the module header's census);
    /// inside, one Next(60).
    #[test]
    fn w4_d22_the_axis_scan_g_carries_the_same_graveyard_gate() {
        let (w, mut n) = bash_ready();
        n.target_zone_graveyard = false;
        let mut rng = UnifiedRandom::new(11);
        let pristine = rng.clone();
        let pc = (400.0, 460.0);
        fighter_obstacle_scan_g(&mut n, &w, &mut rng, true, false, 60.0, pc);
        assert!(n.wants_door_bash.is_some(), "the bash fires");
        assert_eq!(rng, pristine, "outside the zone: zero draws (the axis census)");
        let (w2, mut n2) = bash_ready();
        n2.target_zone_graveyard = true;
        let mut rng2 = UnifiedRandom::new(11);
        let mut mirror = rng2.clone();
        let _ = mirror.next_max(60);
        fighter_obstacle_scan_g(&mut n2, &w2, &mut rng2, true, false, 60.0, pc);
        assert_eq!(rng2, mirror, "inside the zone: exactly one Next(60)");
    }
}
