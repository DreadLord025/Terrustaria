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
//! RNG census (the machine stays draw-free): the door bash's only draw is
//! `flag25`'s `Main.rand.Next(60)` (`:59997`), guarded by
//! `Main.player[target].ZoneGraveyard` — unported (false, the
//! `npc_ai.rs:270` stand-in), so the `&&` short-circuits and ZERO draws
//! are consumed. The ladder, the step-up and the ground probe draw
//! nothing. The bash's world mutation (`WorldGen.OpenDoor`) rides the
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
//! - `WorldGen.ShiftTallGate` (`:60074`) — NOW PORTED (W4-g4,
//!   `tr_core::edit::shift_tall_gate`): the gate-388 bash shifts the gate to
//!   its open kind (389) in the drain; a refusal sets `ai[3] = num57`
//!   (`:60077`). The bash still counts `ai[1] += 2` and plays the thud.
//! - The step-up's `gfxOffY`/`stepSpeed` writes land on the NPC fields,
//!   but the render pass does not read `gfx_off_y` yet and the decay
//!   (`npc_ai_unicorn_b.rs:285`'s helper) is unwired — the lift itself
//!   (the `position.Y` write) is live; the visual offset is the documented
//!   remainder.

use crate::npc::{Npc, NpcKind};
use tr_core::tiledata as td;
use tr_core::{Tile, World};

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
/// (BloodZombie) is NOT in the list — on a non-blood-moon night its
/// `ai[1]` resets every bash; on a blood moon `flag27` is false for
/// EVERY type, so the Blood Zombie accumulates and opens doors too.
const DOOR_BASH_TYPES: [u16; 35] = [
    3, 691, 430, 590, 331, 332, 132, 161, 186, 187, 188, 189, 200, 223, 320, 321, 319, 21, 324,
    323, 322, 44, 196, 167, 77, 197, 202, 203, 449, 450, 451, 452, 481, 201, 635,
];

/// `AI_003_Fighters`' obstacle scan (`NPC.cs:59831-60284`) for the zombie
/// family, one fixed step at the C#'s pre-integration position. Returns
/// `true` when the ground probe's `flag23` early-return fired (`:59863-
/// 59866` — a null cell inside the body's column span): the C# returns
/// from `AI_003_Fighters` itself, skipping the step-up/bash/ladder tail;
/// the caller still integrates (the C#'s `UpdateCollision` runs after
/// `AI()` regardless).
pub(crate) fn fighter_obstacle_scan(npc: &mut Npc, world: &World) -> bool {
    let (w, h) = npc.size();
    // `:57019-57026` — flag6: parked and not just-hit.
    let flag6 = npc.vx == 0.0 && npc.hurt_cd <= 0.0;
    // `:57067-57075` — flag8: true for every zombie kind (module docs).
    let flag8 = true;
    // `:59831-59867` — the flag5 ground probe. `flag` (the hop flag) is
    // false for the zombie family (module docs), so the gate is vy == 0.
    let mut flag5 = false;
    if npc.vy == 0.0 {
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
    // `:59868-59871` — the type-428 flag5 reset: not a zombie kind.
    // `:59872-59948` — the 1-tile step-up assist (the unicorn machine's
    // inline copy shape, `npc_ai_unicorn.rs:328-387`, but the AI_003
    // probe rows). Only while falling or level; the type-580 directionY
    // gate names no zombie kind.
    if npc.vy >= 0.0 {
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
                // reach (the 163/164/236/239/530 +8 list names no zombie
                // kind), the gfxOffY record and the 1/2 step speed.
                let mut num195 = num194 as f32 * 16.0;
                if t0.half_brick() {
                    num195 += 8.0;
                }
                if t1.half_brick() {
                    num195 -= 8.0;
                }
                if num195 < npc.y + h {
                    let num196 = npc.y + h - num195;
                    let num197 = 16.1f32;
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
        // `:59951-59952` — the ahead probe: centre + 15 * direction, the
        // body's bottom - 15 row (the wide-body override list `:59953`
        // names no zombie kind).
        let num198 = ((npc.x + (w as i32 / 2) as f32 + (15 * npc.facing) as f32) / 16.0) as i32;
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
                // The graveyard zone is unported (false — the
                // `npc_ai.rs:270` stand-in), so the `&&` short-circuits
                // and the draw is NEVER consumed (the RNG census).
                let flag25 = false;
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
                // fires), the type-27 +1 and the 31/294-296 +6: none is a
                // zombie kind.
                if flag26 {
                    npc.ai[1] += 6.0;
                }
                // `:60035` — the bash counter resets.
                npc.ai[2] = 0.0;
                // `:60036-60045` — at 10 the door opens (the type-460
                // always-open arm names no zombie kind).
                let mut flag28 = false;
                if npc.ai[1] >= 10.0 {
                    flag28 = true;
                    npc.ai[1] = 10.0;
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
                // `ShiftTallGate` rides `edit::shift_tall_gate` in the drain
                // (W4-g4); a refusal sets `ai[3] = num57` (`:60077`).
            }
        } else {
            // `:60089-60203` — the probe jump ladder.
            // `:60091-60095` — num201 = spriteDirection (the type-425
            // flip names no zombie kind).
            let num201 = npc.sprite_dir;
            // `:60096` — only while moving WITH the facing.
            if (npc.vx < 0.0 && num201 == -1) || (npc.vx > 0.0 && num201 == 1) {
                // `:60098-60110` — the 2/3-tile wall tiers (the zombie
                // box is 40 tall, >= 32).
                if h >= 32.0 && solid_tile_no_platforms(world, num198, num199 - 2) {
                    if solid_tile_no_platforms(world, num198, num199 - 3) {
                        npc.vy = -8.0;
                        npc.net_update = true;
                    } else {
                        npc.vy = -7.0;
                        npc.net_update = true;
                    }
                } else if solid_tile_no_platforms(world, num198, num199 - 1) {
                    // `:60111-60128` — the 1-tile wall: -6f (the type-624
                    // gnome arm names no zombie kind).
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
                    && !solid_tile_allow_bottom_slope(world, num198, num199 + 1)
                    && !solid_tile_allow_bottom_slope(world, num198 + npc.facing, num199 + 1)
                {
                    // `:60134-60139` — the gap leap: -8f and vx ×1.5 (the
                    // type-67 exclusion names no zombie kind).
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
                // (no expertMode flag, 586 not a zombie kind — module
                // docs).
            }
            // `:60205-60277` — the post-ladder type arms (31/294-296/47/
            // 77/104/168/196/385/389/464/470/524-527, 120, 287, 460): none
            // is a zombie kind.
        }
    } else if flag8 {
        // `:60280-60284` — no floor under the body: the bash counters
        // reset.
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
    }
    // `:60285-60297` — the type-120 teleport: not a zombie kind.
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
            fighter_obstacle_scan(&mut n, &w);
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
        fighter_obstacle_scan(&mut n, &w);
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
        fighter_obstacle_scan(&mut n, &w);
        assert_eq!(n.ai[2], 0.0, "the bash counter resets");
        assert_eq!(n.ai[1], 5.0, "+5 per door bash");
        assert_eq!(n.vx, -0.5, "the bash push-back");
        let (tx, ty, dir, open) = n.wants_door_bash.expect("the bash request");
        assert_eq!((tx, ty, dir), (21, 28, 1), "the door cell + direction");
        assert!(!open, "ai[1] = 5 < 10: not yet open");
        // A second bash (ai[1] 5 → 10) arms the open.
        n.ai[2] = 59.0;
        fighter_obstacle_scan(&mut n, &w);
        assert_eq!(n.ai[1], 10.0, "ai[1] caps at 10");
        assert!(n.wants_door_bash.expect("the second request").3, "the open armed");
    }

    /// The blood-moon gate (`NPC.cs:60007`): on a NORMAL night the zombie
    /// (type 3, in flag24) takes flag27 → `ai[1] = 0` before the +5, so it
    /// never accumulates; the Blood Zombie (489, NOT in flag24) also
    /// resets on a normal night but accumulates on a blood moon.
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
        fighter_obstacle_scan(&mut n, &w);
        // flag27 resets ai[1] to 0, then +5 → 5 (not 13).
        assert_eq!(n.ai[1], 5.0, "the normal-night reset");
        assert!(!n.wants_door_bash.expect("the request").3, "never opens on a normal night");
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
        fighter_obstacle_scan(&mut n, &w);
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
}
