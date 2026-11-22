//! The aiStyle-8 caster family — `NPC.AI()`'s `if (this.aiStyle == 8)` arm
//! (`NPC.cs:20817-21376`), S1-D-3 (aiStyle-8 slice, code-only mode).
//!
//! IDENTITY NOTE: the task brief said "aiStyle 7 caster family"; the C#
//! dispatch is `aiStyle == 7 → AI_007_TownEntities` (the town-NPC family,
//! `NPC.cs:20812-20815`, sibling task/S1-D-3-town-ai) and
//! `aiStyle == 8 → the caster arm` (`:20817`). This module ports the
//! CASTER arm — aiStyle 8 — verified from the dispatch.
//!
//! The C# arm serves 13 types (`SetDefaults`' `this.aiStyle = 8` rows:
//! 24 FireImp / 29 GoblinSorcerer / 32 DarkCaster / 45 Tim / 172 RuneWizard /
//! 281-282 RaggedCaster / 283-284 Necromancer / 285-286 Diabolist /
//! 533 DesertDjinn / 693 LibrarianSkeleton — `NPC.cs:9083`/`:9125`/`:9326`/
//! `:11158`/`:12644-12714`/`:14978`/`:15690`/`:17727`). The port models five
//! of them (`npc/data.rs` + `bestiary.rs` net ids): FireImp 24,
//! GoblinSorcerer 29, DarkCaster 32, Tim 45, DesertDjinn 533. This module
//! ports the whole arm's STRUCTURE with the branches those five types
//! take; the unported types' own branches (172's alpha ramp `:20844-20854`
//! + six-tick trigger `:21023-21030` + jittered shadow beam `:21264-21274`,
//! 281-286's trigger ladders `:21011-21022`/`:21039-21050` + the 450/540
//! wraps + the aimed skeleton shots `:21177-21227` with
//! `GetAttackDamage_ForProjectiles` `:7052-7055`, 693's localAI clock
//! `:20839-20842`/`:20994-21005`/`:21106-21111` + the book teleport
//! `AI_AttemptToFindTeleportSpotNearBooks` `:19009-19085` + the book shot
//! `AI_FindNearbyBook` `:62308-62364` / `:21248-21263`) are cited skips —
//! they land with their kinds.
//!
//! UNWIRED (code-only mode, the d6bc30ed bat precedent): the live dispatch
//! still routes the five kinds through `npc.rs::update_caster` →
//! `npc_ai.rs::caster_cycle`, whose defects this re-port fixes (the instant
//! teleport vs the C#'s deferred ai[2]/ai[3] arrival, flag5 constant-on,
//! the missing flag4 half-speed gate, the teleport search's column-abort
//! semantics, the skipped Djinn volley + its arm-level draws — the full
//! catalogue in docs/ANALYSIS.md "The caster aiStyle-8 re-port"). The
//! migration replaces that call with [`caster_ai`] and drains
//! [`CasterShot`] through the NPC spawn seam / the projectile pool. Every
//! other module is byte-identical.
//!
//! The arm RETURNS at `:21375` — the generic `UpdateNPC` tail (gravity
//! `NPC.cs:90465+`, `position += velocity`, `UpdateCollision`) is the
//! caller's, NOT this module's (the live `caster_cycle` folds an invented
//! inline integration in; the faithful arm never integrates).
//!
//! RNG by construction (every `Main.rand` draw on the ported paths, in
//! order; `UnifiedRandom.cs:120-141` via `tr_core::UnifiedRandom`):
//! 1. Teleport-arrival bursts (`:20863-20922` + `:20932-20991`): types
//!    29/45 draw `Next(1, 3)` 50× per burst (the scale ARGUMENT — evaluated
//!    even when `Dust.NewDust` culls), two bursts = exactly 100 draws on
//!    the arrival tick; every other type's bursts draw 0. The `NewDust`
//!    INTERNAL draws are the dust task's (S1-E-1's `DustRng` seam) — a
//!    documented skip, as in every unwired AI module.
//! 2. The teleport search (`AI_AttemptToFindTeleportSpot` `:18949-18953`):
//!    2 draws per attempt (X column, then Y start), ≤100 attempts; the
//!    >2000-px Manhattan bail (`:18944`) draws NOTHING; the column scan
//!    itself draws nothing.
//! 3. The Djinn volley search (`:21137-21160`): on the `% 30 == 0 &&
//!    / 30 < 5` windup ticks (ai[1] = 120/90/60/30/0 AFTER the decrement),
//!    2 draws per iteration (X then Y, `Next(p.X-6, p.X+7)`), ≤50
//!    iterations, early-exit on the first accepted tile; the >2000-px
//!    length gate (`:21133`) and `netMode == 1` draw NOTHING.
//! 4. The ambient tail gates (`:21279-21374`): 29/45 `Next(5)` every tick,
//!    32 `Next(3)` every tick, the else group (24) `Next(6)` every tick,
//!    533 draws NOTHING (light only). The gated `NewDust` internals are
//!    the same dust-task skip.
//! No other block draws: the decay/init/triggers/wrap/countdown are pure
//! arithmetic, `TargetClosest` is RNG-free (`NPC.cs:77652-77781`). The
//! witness tests below pin the stream position for each case (written,
//! NOT run — code-only mode).
#![allow(dead_code)] // wiring lands with the update_caster migration

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The read-only per-tick inputs the C# arm takes from `Main` statics.
pub struct CasterEnv<'a> {
    pub world: &'a World,
    /// The `Main.player[]` slots as `TargetClosest` sees them (the
    /// `npc_target.rs` seam, S1-D-3p). Indexed by `npc.target` after the
    /// scan, like the C#'s `Main.player[this.target]` (`:20830`).
    pub players: &'a [TargetCandidate],
    /// `Main.player[j].velocity` per slot — the telefrag rect's
    /// 20-tick extrapolation (`NPC.cs:18988`, `num4 = 20f` `:18943`).
    /// `TargetCandidate` carries no velocity, so it rides this parallel
    /// slice (the style14 `players_wet` convention).
    pub player_velocities: &'a [(f32, f32)],
    /// `Main.netMode` — 1 (client) skips the teleport attempt (`:21083`)
    /// and every spawn arm (`:21122`/`:21169`/`:21179`/`:21234`).
    pub net_mode: i32,
    /// `Main.EverLastingTicker` — the `targetSetFrame` stamp
    /// (`NPC.cs:77780` via `TargetClosest`).
    pub ticker: u64,
    /// `Main.getGoodWorld` — the type-24 double-increment gate
    /// (`NPC.cs:21053`).
    pub get_good_world: bool,
    /// `NPC.AnyNPCs(113)` — the second half of that gate (`:21053`).
    pub any_npcs_113: bool,
    /// `NPC.AnyNPCs(35)` — a live Skeletron head; the teleport spot's
    /// dungeon-wall fallback (`NPC.cs:18962`).
    pub any_skeletron: bool,
    /// `FindFirstNPC(35)` with `ai[3] == 1` — the Dark Caster's red-hat
    /// latch source (`NPC.cs:20822-20827`; the sim's
    /// `World.skeletron_red_hat` write-through mirror).
    pub skeletron_red_hat: bool,
}

/// The arm's spawn requests — the pure module never touches the pools.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CasterShot {
    /// `NPC.NewNPC` (`NPC.cs:21171`/`:21238`/`:21242`/`:21246`): the
    /// vanilla sphere type (25 BurningSphere / 30 ChaosBall / 33
    /// WaterSphere / 665 ChaosBallTim), the INTEGER spawn coordinates
    /// (`(int)position.X + width / 2 …` — int arithmetic, not float), and
    /// the `ai3` payload (`flag4 ? 1 : 0` for the Dark Caster, else 0).
    Sphere { npc_type: i32, x: i32, y: i32, ai3: f32 },
    /// `Projectile.NewProjectile` type 596, the Djinn's curse volley
    /// (`NPC.cs:21155`): damage 0, knockback 1, `ai1 = this.target`,
    /// zero velocity, spawned at the tile centre `(x*16+8, y*16+8)`.
    Curse { x: f32, y: f32, target: i32 },
}

/// `Main.wallDungeon` (`Main.cs:10598-10606`): walls 7/8/9/94-99.
fn wall_dungeon(wall: u16) -> bool {
    matches!(wall, 7 | 8 | 9 | 94 | 95 | 96 | 97 | 98 | 99)
}

/// `Utils.ToTileCoordinates` (`Utils.cs:1807-1810`): `(int)v >> 4` —
/// truncate toward zero FIRST, then arithmetic-shift (negative coords
/// floor, unlike the plain `/16`).
fn to_tile(v: f32) -> i32 {
    (v as i32) >> 4
}

/// `NPC.AI_GlobalFiringDistanceCheck` (`NPC.cs:55835-55841`): the
/// `Main.MaxWorldViewSize` (1920×1200, `Main.cs:66714`) Point-centred
/// rectangle (`Utils.cs:952-955` — INTEGER halving), inflated −50, with
/// the half-open `Rectangle.Contains` on the shooter's truncated Point.
pub fn global_firing_distance_check(shoot_x: f32, shoot_y: f32, target_x: f32, target_y: f32) -> bool {
    let cx = target_x as i32;
    let cy = target_y as i32;
    // CenteredRectangle(Point(1920, 1200)) then Inflate(-50, -50).
    let rx = cx - 1920 / 2 + 50;
    let ry = cy - 1200 / 2 + 50;
    let (rw, rh) = (1920 - 100, 1200 - 100);
    let sx = shoot_x as i32;
    let sy = shoot_y as i32;
    sx >= rx && sx < rx + rw && sy >= ry && sy < ry + rh
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): out-of-bounds counts as SOLID (the four
/// early returns), and the per-tile test is
/// `active && !inActive && tileSolid && !tileSolidTop` — platforms do NOT
/// block the body space.
pub(crate) fn solid_tiles(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if x0 < 0 || x1 >= world.width || y0 < 0 || y1 >= world.height - 40 {
        return true;
    }
    for x in x0..=x1 {
        for y in y0..=y1 {
            if let Some(t) = world.tile(x, y) {
                if t.active()
                    && t.flags & tr_core::tile::FLAG_INACTIVE == 0
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// `NPC.AI_AttemptToFindTeleportSpot` (`NPC.cs:18937-19007`), the
/// aiStyle-8 call's defaults (`:21097` passes only destination + the
/// target tile): range 20 (the parameter default — num88 is the windup,
/// NOT the radius), telefrag 5 tiles, fluff 1, not centred, not in-air.
///
/// Faithful details the live `npc_ai.rs::find_teleport_spot` drops:
/// * the >2000-px Manhattan bail (`:18944-18948`) — `num3 = 100` skips the
///   whole loop, ZERO draws;
/// * the dungeon-wall gate (`:18960-18966`) for 32/281-286/693 with the
///   `AnyNPCs(35)` fallback — and its ELSE-IF quirk: when the gate branch
///   is taken the lava check (`:18967`) never runs;
/// * the column ABORT (`:21003`'s unconditional `break`): the first tile
///   that passes the floor+body gates ends the column even when the
///   telefrag union rejects it — the search does NOT scan deeper;
/// * the telefrag union (`:18980-18997`): the 16×16 spot rect inflated 80
///   vs EVERY live player's hitbox unioned with its `(int)(velocity*20)`
///   offset copy.
#[allow(clippy::too_many_arguments)]
fn find_teleport_spot(
    env: &CasterEnv<'_>,
    rng: &mut UnifiedRandom,
    net_id: i32,
    self_center: (f32, f32),
    target_tile: (i32, i32),
    radius: i32,
) -> Option<(i32, i32)> {
    let world = env.world;
    // `:18939-18940` — `(int)Center.X / 16`: truncate, then INTEGER /16.
    let num = self_center.0 as i32 / 16;
    let num2 = self_center.1 as i32 / 16;
    // `:18944-18948` — the Manhattan bail (tile coords × 16).
    if (num * 16 - target_tile.0 * 16).abs() + (num2 * 16 - target_tile.1 * 16).abs() > 2000 {
        return None;
    }
    let dungeon_kind = net_id == 32 || (281..=286).contains(&net_id) || net_id == 693;
    for _ in 0..100 {
        // `:18952-18953` — X first, then the Y start; one draw each.
        let x = rng.next_range(target_tile.0 - radius, target_tile.0 + radius + 1);
        let y_start = rng.next_range(target_tile.1 - radius, target_tile.1 + radius + 1);
        for i in y_start..target_tile.1 + radius {
            // `:18955-18958` — the caster's own ±1 box, then !nactive.
            if (i >= num2 - 1 && i <= num2 + 1 && x >= num - 1 && x <= num + 1)
                || !world.tile(x, i).is_some_and(|t| t.active())
            {
                continue;
            }
            let wall_above = world.tile(x, i - 1).map_or(0, |t| t.wall);
            let mut ok = true;
            // `:18960-18970` — the dungeon gate, ELSE-IF lava.
            if dungeon_kind && !wall_dungeon(wall_above) {
                if !env.any_skeletron {
                    ok = false;
                }
            } else if world.tile(x, i - 1).is_some_and(|t| {
                t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
            }) {
                ok = false;
            }
            // `:18971-18974` — the floor must be tileSolid (platforms
            // count HERE, unlike the body check).
            if !ok
                || !world
                    .tile(x, i)
                    .is_some_and(|t| tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                continue;
            }
            // `:18976` — !SolidTiles(x-1, x+1, i-4, i-1) (fluff 1,
            // not centred: rows i-3-1 .. i-1).
            if solid_tiles(world, x - 1, x + 1, i - 4, i - 1) {
                continue;
            }
            // `:18980-18997` — the telefrag union over every live player.
            let (rx, ry, rw, rh) = (x * 16 - 80, i * 16 - 80, 176, 176);
            for (j, p) in env.players.iter().enumerate() {
                if !p.active || p.dead {
                    continue;
                }
                let (vx, vy) = env.player_velocities.get(j).copied().unwrap_or((0.0, 0.0));
                // value = Hitbox; value2 = value.Modified((int)(v*20));
                // Union(value2, value).
                let ox = (vx * 20.0) as i32;
                let oy = (vy * 20.0) as i32;
                let px = p.x as i32;
                let py = p.y as i32;
                let ux0 = px.min(px + ox);
                let uy0 = py.min(py + oy);
                let ux1 = (px + p.width).max(px + ox + p.width);
                let uy1 = (py + p.height).max(py + oy + p.height);
                if rx < ux1 && rx + rw > ux0 && ry < uy1 && ry + rh > uy0 {
                    ok = false;
                    break;
                }
            }
            if ok {
                return Some((x, i));
            }
            // `:19003` — the unconditional break: telefrag failure ends
            // the COLUMN; the next attempt draws a fresh X/Y pair.
            break;
        }
    }
    None
}

/// The teleport-arrival dust bursts' ARM-LEVEL draws (`:20863-20922` and
/// `:20932-20991`): types 29/45 pass `Main.rand.Next(1, 3)` as the scale
/// argument 50× per burst — the draw happens even when `NewDust` culls.
/// Every other ported type's burst arms use constant scales (0 draws).
/// The `NewDust` internals are the dust task's documented skip.
fn teleport_burst_draws(rng: &mut UnifiedRandom, net_id: i32) {
    if net_id == 29 || net_id == 45 {
        for _ in 0..50 {
            rng.next_range(1, 3); // `:20867` / `:20936`
        }
    }
}

/// The aiStyle-8 arm (`NPC.cs:20817-21376`) for one NPC, one tick.
/// Returns the tick's spawn request, if any. The generic `UpdateNPC`
/// movement tail is the caller's (the arm returns at `:21375`).
pub fn caster_ai(npc: &mut Npc, env: &CasterEnv<'_>, rng: &mut UnifiedRandom) -> Option<CasterShot> {
    let net_id = npc.kind.net_id();
    // `:20819-20828` — flag4 = RedHatSkeletronAdjustmentsEnabled (for the
    // ported kinds only 32 can be true: localAI[3] == 1, `:66673`), then
    // the Dark Caster's sticky latch from the live red-hat Skeletron.
    let mut flag4 = net_id == 32 && npc.local_ai[3] == 1.0;
    if net_id == 32 && env.skeletron_red_hat {
        npc.local_ai[3] = 1.0;
        flag4 = true;
    }
    // `:20829` — TargetClosest (the S1-D-3p helper; faceTarget default
    // true, aiStyle 8). RNG-free.
    target_closest(npc, env.world, env.players, true, 8, env.ticker);
    // `:20830` — the target player snapshot, read once.
    let p = &env.players[npc.target as usize];
    let pcx = p.x + p.width as f32 / 2.0;
    let pcy = p.y + p.height as f32 / 2.0;
    // `:20831-20835` — the X stand decay + the ±0.1 snap.
    npc.vx *= 0.93;
    if npc.vx > -0.1 && npc.vx < 0.1 {
        npc.vx = 0.0;
    }
    // `:20836-20843` — the 500 counter init (693's localAI[1] = 150 is
    // the unported kind's skip).
    if npc.ai[0] == 0.0 {
        npc.ai[0] = 500.0;
    }
    // `:20844-20854` — type 172's alpha ramp (RuneWizard unported, skip).
    // `:20855-20992` — the DEFERRED teleport arrival: last tick's search
    // wrote ai[2]/ai[3]; the position snap happens HERE, next tick.
    if npc.ai[2] != 0.0 && npc.ai[3] != 0.0 {
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        // (172's alpha = 255 `:20858-20861` — unported.) The Item8 sounds
        // (`:20862`/`:20931`) are the sound task's skip.
        teleport_burst_draws(rng, net_id); // burst 1, at the OLD position
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
        // `:20924-20925` — `(float)(width / 2)` is INTEGER halving;
        // height is the int box height.
        let (w, h) = npc.size();
        npc.x = npc.ai[2] * 16.0 - (w as i32 / 2) as f32 + 8.0;
        npc.y = npc.ai[3] * 16.0 - (h as i32) as f32;
        npc.net_offset *= 0.0; // `:20926`
        npc.vx = 0.0;
        npc.vy = 0.0;
        npc.ai[2] = 0.0;
        npc.ai[3] = 0.0;
        teleport_burst_draws(rng, net_id); // burst 2, at the NEW position
    }
    // `:20993` — the cycle tick. (`:20994-21005` — 693's localAI clock,
    // unported skip.)
    npc.ai[0] += 1.0;
    // `:21006-21010` — the firing-distance gate; type 29 is exempt.
    let (cx, cy) = npc.center();
    let mut flag5 = global_firing_distance_check(cx, cy, pcx, pcy);
    if net_id == 29 {
        flag5 = true;
    }
    // `:21011-21074` — the per-type shot triggers. 283/284 (`:21011`),
    // 172 (`:21023`), 281/282 (`:21039`) are unported skips.
    if net_id == 533 {
        // `:21031-21038` — the Djinn's single 180 trigger → the 181 windup.
        if flag5 && npc.ai[0] == 180.0 {
            npc.ai[1] = 181.0;
            npc.net_update = true;
        }
    } else {
        // `:21053-21060` — the getGoodWorld Fire Imp double-increment.
        if env.get_good_world && net_id == 24 && env.any_npcs_113 {
            npc.ai[0] += 1.0;
            if npc.ai[0] % 2.0 == 1.0 {
                npc.ai[0] -= 1.0;
            }
        }
        // `:21061-21068` — the red-hat half-speed gate: the extra ++ and
        // the odd snap-back freeze ai[0] on even values.
        if flag4 {
            npc.ai[0] += 1.0;
            if npc.ai[0] % 2.0 == 1.0 {
                npc.ai[0] -= 1.0;
            }
        }
        // `:21069-21073` — the standard 100/200/300 triggers.
        if flag5 && (npc.ai[0] == 100.0 || npc.ai[0] == 200.0 || npc.ai[0] == 300.0) {
            npc.ai[1] = 30.0;
            npc.net_update = true;
        }
    }
    // `:21075-21078` — 285/286's >400 wrap (unported skip).
    // `:21079-21082` — the Djinn's early wrap feeds the SAME-tick teleport.
    if net_id == 533 && npc.ai[0] >= 360.0 {
        npc.ai[0] = 650.0;
    }
    // `:21083-21112` — the teleport attempt (server/single-player only).
    if npc.ai[0] >= 650.0 && env.net_mode != 1 {
        npc.ai[0] = 1.0;
        // `:21086-21087` — `(int)Center.X / 16` (truncate, then int /16).
        let target_tile = (pcx as i32 / 16, pcy as i32 / 16);
        // `:21089-21093` — num88 is ONLY the post-teleport windup
        // (`ai[1] = num88`, `:21101`); the search call `:21097` passes
        // just (destination, num86, num87), so rangeFromTargetTile takes
        // its DEFAULT 20 (`:18937`) for EVERY type — the live
        // CasterParams::FIRE_IMP.teleport_radius = 5 is a defect.
        let num88 = if net_id == 24 { 5 } else { 20 };
        // `:21094` — 693's book search first (unported skip).
        if let Some((tx, ty)) =
            find_teleport_spot(env, rng, net_id, (cx, cy), target_tile, 20)
        {
            // `:21101-21103` — the DESTINATION rides ai[2]/ai[3]; the
            // position snap is NEXT tick's arrival block, not here.
            npc.ai[1] = num88 as f32;
            npc.ai[2] = tx as f32;
            npc.ai[3] = ty as f32;
        }
        npc.net_update = true; // `:21105` (even on a failed search)
        // `:21106-21111` — 693's localAI reset (unported skip).
    }
    // `:21113-21278` — the windup countdown + the shoot arms.
    let mut shot = None;
    if npc.ai[1] > 0.0 {
        let num89 = 25; // `:21115`
        npc.ai[1] -= 1.0;
        if net_id == 533 {
            // `:21117-21162` — the curse volley on the %30 ticks below
            // the 5th (ai[1] = 120/90/60/30/0 after the decrement).
            if npc.ai[1] % 30.0 == 0.0 && npc.ai[1] / 30.0 < 5.0 && env.net_mode != 1 {
                let (scx, scy) = npc.center();
                let self_pt = (to_tile(scx), to_tile(scy));
                let tgt_pt = (to_tile(pcx), to_tile(pcy));
                let (dx, dy) = (pcx - scx, pcy - scy);
                // `:21133-21136` — the >2000-px gate draws NOTHING.
                if (dx * dx + dy * dy).sqrt() <= 2000.0 {
                    for _ in 0..50 {
                        // `:21140-21141` — X first, then Y.
                        let x = rng.next_range(tgt_pt.0 - 6, tgt_pt.0 + 6 + 1);
                        let y = rng.next_range(tgt_pt.1 - 6, tgt_pt.1 + 6 + 1);
                        // `:21142` — outside BOTH the target's ±2 box and
                        // the caster's ±6 box, and the tile not active.
                        let outside_target =
                            y < tgt_pt.1 - 2 || y > tgt_pt.1 + 2 || x < tgt_pt.0 - 2 || x > tgt_pt.0 + 2;
                        let outside_self =
                            y < self_pt.1 - 6 || y > self_pt.1 + 6 || x < self_pt.0 - 6 || x > self_pt.0 + 6;
                        if outside_target
                            && outside_self
                            && !env.world.tile(x, y).is_some_and(|t| t.active())
                        {
                            // `:21145-21152` — no lava, no solid ±2 box.
                            if env.world.tile(x, y).is_some_and(|t| {
                                t.liquid > 0
                                    && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
                            }) {
                                continue;
                            }
                            if solid_tiles(env.world, x - 2, x + 2, y - 2, y + 2) {
                                continue;
                            }
                            // `:21155` — the 596 curse, tile centre.
                            shot = Some(CasterShot::Curse {
                                x: (x * 16 + 8) as f32,
                                y: (y * 16 + 8) as f32,
                                target: npc.target,
                            });
                            break;
                        }
                    }
                }
            }
        } else if net_id == 24 {
            // `:21164-21174` — the Fire Imp's Burning Sphere at ai[1]==10.
            if npc.ai[1] == 10.0 && env.net_mode != 1 {
                let (w, _) = npc.size();
                shot = Some(CasterShot::Sphere {
                    npc_type: 25,
                    x: npc.x as i32 + w as i32 / 2 + npc.facing * 8,
                    y: npc.y as i32 + 20,
                    ai3: 0.0,
                });
            }
        } else if npc.ai[1] == num89 as f32 {
            // `:21175-21277` — the standard shoot tick. 281-286's aimed
            // skeleton shots (`:21177-21227`) and 693's book shot
            // (`:21248-21263`) / 172's shadow beam (`:21264-21274`) are
            // unported skips.
            if env.net_mode != 1 {
                let (w, _) = npc.size();
                let sx = npc.x as i32 + w as i32 / 2;
                let sy = npc.y as i32 - 8;
                match net_id {
                    29 => shot = Some(CasterShot::Sphere { npc_type: 30, x: sx, y: sy, ai3: 0.0 }), // `:21238`
                    45 => shot = Some(CasterShot::Sphere { npc_type: 665, x: sx, y: sy, ai3: 0.0 }), // `:21242`
                    32 => {
                        // `:21246` — the red-hat state rides the sphere.
                        shot = Some(CasterShot::Sphere {
                            npc_type: 33,
                            x: sx,
                            y: sy,
                            ai3: if flag4 { 1.0 } else { 0.0 },
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    // `:21279-21374` — the ambient tail: the netOffset bracket, the
    // per-type dust GATE draws (the NewDust internals + the AddLight
    // calls are the dust/light tasks' skips — S1-E-1/S1-E-5).
    npc.x += npc.net_offset.x;
    npc.y += npc.net_offset.y;
    if net_id == 29 || net_id == 45 {
        let _ = rng.next_max(5); // `:21282` — Next(5) == 0 gates the dust
    } else if net_id == 32 {
        let _ = rng.next_max(3); // `:21292` — Next(3) != 0 gates the dust
    } else if net_id == 533 {
        // `:21358-21361` — light only, NO draw.
    } else {
        // `:21362-21373` — the else group (24): light + Next(6) == 0.
        let _ = rng.next_max(6);
    }
    npc.x -= npc.net_offset.x;
    npc.y -= npc.net_offset.y;
    shot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE, FLAG_LIQUID_LAVA};

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
        World::new("t".into(), 400, 240)
    }

    /// A floor row at ty=100 across the map (kind 1 = stone, solid).
    fn floored() -> World {
        let mut w = air();
        for x in 0..400 {
            let i = w.index(x, 100);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    fn env<'a>(
        w: &'a World,
        players: &'a [TargetCandidate],
        vels: &'a [(f32, f32)],
    ) -> CasterEnv<'a> {
        CasterEnv {
            world: w,
            players,
            player_velocities: vels,
            net_mode: 0,
            ticker: 7,
            get_good_world: false,
            any_npcs_113: false,
            any_skeletron: false,
            skeletron_red_hat: false,
        }
    }

    fn sorcerer(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::GoblinSorcerer, x, y, 1, 0.0)
    }

    /// The stream-position witness: run the arm on `n`, and count how
    /// many draws the shared stream advanced. Exact method: capture the
    /// next five raw draws AFTER the arm, then find the unique n whose
    /// n-draw prefix of a fresh same-seed stream reproduces them (the
    /// peek/draw-before-increment convention: `peek` never advances,
    /// `next` does; five matching probes pin the position).
    fn draws_used(seed: i32, f: impl FnOnce(&mut UnifiedRandom)) -> i32 {
        let mut a = UnifiedRandom::new(seed);
        f(&mut a);
        let probes: [i32; 5] = std::array::from_fn(|_| a.next());
        for n in 0..=300 {
            let mut b = UnifiedRandom::new(seed);
            for _ in 0..n {
                b.next();
            }
            if probes.iter().all(|&p| b.next() == p) {
                return n;
            }
        }
        panic!("no draw count in 0..=300 reproduces the stream position");
    }

    /// `NPC.cs:20836-20843` + `:20993` — the 0-init becomes 500 THEN
    /// increments: the first tick reads 501, not 500.
    #[test]
    fn the_counter_inits_to_500_then_ticks() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut n = sorcerer(1900.0, 1560.0);
        let mut rng = UnifiedRandom::new(1);
        caster_ai(&mut n, &env(&w, &ps, &vs), &mut rng);
        assert_eq!(n.ai[0], 501.0);
    }

    /// `:20831-20835` — the 0.93 decay and the ±0.1 snap-to-zero.
    #[test]
    fn the_x_stand_decays_and_snaps() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut n = sorcerer(1900.0, 1560.0);
        n.vx = 0.105; // × 0.93 = 0.09765 < 0.1 → snapped
        let mut rng = UnifiedRandom::new(1);
        caster_ai(&mut n, &env(&w, &ps, &vs), &mut rng);
        assert_eq!(n.vx, 0.0);
        n.vx = 2.0;
        caster_ai(&mut n, &env(&w, &ps, &vs), &mut rng);
        assert!((n.vx - 1.86).abs() < 1e-6);
    }

    /// `:21006-21010` + `:21069-21073` — the trigger needs flag5 (the
    /// 1920×1200 −50 rect) for the Dark Caster, but the Goblin Sorcerer
    /// (29) is exempt. A target 3000 px away silences 32, not 29.
    #[test]
    fn the_firing_gate_is_real_and_29_is_exempt() {
        let w = floored();
        let ps = [cand(5000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut rng = UnifiedRandom::new(1);
        let mut dc = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc.ai[0] = 99.0; // ++ → 100, the trigger tick
        caster_ai(&mut dc, &env(&w, &ps, &vs), &mut rng);
        assert_eq!(dc.ai[1], 0.0, "out of the firing rect: no windup");
        let mut gs = sorcerer(1900.0, 1560.0);
        gs.ai[0] = 99.0;
        caster_ai(&mut gs, &env(&w, &ps, &vs), &mut rng);
        // The trigger sets 30, then the SAME tick's countdown (`:21116`)
        // decrements it to 29.
        assert_eq!(gs.ai[1], 29.0, "type 29 forces flag5");
        assert!(gs.net_update);
    }

    /// `:55835-55841` — the rect is Point-centred, inflated −50, and
    /// half-open: the left/top edges are inside, right/bottom outside.
    #[test]
    fn the_firing_rect_edges_are_half_open() {
        // Target centre (1000, 1000): rect x = 1000-960+50 = 90,
        // w 1820 → [90, 1910); y = 1000-600+50 = 450, h 1100 → [450, 1550).
        assert!(global_firing_distance_check(90.0, 450.0, 1000.0, 1000.0));
        assert!(!global_firing_distance_check(89.9, 450.0, 1000.0, 1000.0));
        assert!(global_firing_distance_check(1909.9, 1549.9, 1000.0, 1000.0));
        assert!(!global_firing_distance_check(1910.0, 1550.0, 1000.0, 1000.0));
    }

    /// `:21061-21068` — the red-hat latch halves the cycle: ai[0] only
    /// lands on even values while flag4 holds.
    #[test]
    fn the_red_hat_latch_halves_the_cycle() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut e = env(&w, &ps, &vs);
        e.skeletron_red_hat = true;
        let mut dc = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc.ai[0] = 10.0;
        let mut rng = UnifiedRandom::new(1);
        caster_ai(&mut dc, &e, &mut rng);
        assert_eq!(dc.local_ai[3], 1.0, "the latch sticks (:20825)");
        assert_eq!(dc.ai[0], 12.0, "10 → 11 → 12 → odd? no: 12 even, stays");
        caster_ai(&mut dc, &e, &mut rng);
        assert_eq!(dc.ai[0], 14.0);
        // Without the latch the same two ticks run 10 → 11 → 12.
        let mut plain = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        plain.ai[0] = 10.0;
        caster_ai(&mut plain, &env(&w, &ps, &vs), &mut rng);
        caster_ai(&mut plain, &env(&w, &ps, &vs), &mut rng);
        assert_eq!(plain.ai[0], 12.0);
    }

    /// `:21083-21112` + `:20855-20930` — the teleport is DEFERRED: the
    /// 650 tick writes ai[2]/ai[3] and leaves the position alone; the
    /// NEXT tick snaps (integer half-width), zeroes velocity/netOffset
    /// and clears ai[2]/ai[3].
    #[test]
    fn the_teleport_defers_the_position_snap_one_tick() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut n = sorcerer(1900.0, 1560.0);
        n.ai[0] = 649.0; // ++ → 650 → the search tick
        let mut rng = UnifiedRandom::new(3);
        caster_ai(&mut n, &env(&w, &ps, &vs), &mut rng);
        assert_eq!(n.ai[0], 1.0, "the counter resets (:21085)");
        assert!(n.ai[2] != 0.0 && n.ai[3] != 0.0, "a spot was found");
        let (x0, y0) = (n.x, n.y);
        let (tx, ty) = (n.ai[2], n.ai[3]);
        caster_ai(&mut n, &env(&w, &ps, &vs), &mut rng);
        let (w_px, h_px) = n.size();
        assert_eq!(n.x, tx * 16.0 - (w_px as i32 / 2) as f32 + 8.0);
        assert_eq!(n.y, ty * 16.0 - (h_px as i32) as f32);
        assert!((n.x != x0) || (n.y != y0), "the snap moved the caster");
        assert_eq!((n.vx, n.vy), (0.0, 0.0));
        assert_eq!((n.ai[2], n.ai[3]), (0.0, 0.0));
    }

    /// `:20867`/`:20936` — the arrival tick draws EXACTLY 100 Next(1,3)
    /// for type 29 (50 per burst, argument-position draws) plus the one
    /// ambient Next(5); the Dark Caster's arrival draws only its ambient
    /// Next(3).
    #[test]
    fn the_arrival_bursts_draw_100_for_29_and_0_for_32() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut n = sorcerer(1900.0, 1560.0);
        n.ai[2] = 130.0;
        n.ai[3] = 100.0;
        let used = draws_used(5, |rng| {
            caster_ai(&mut n, &env(&w, &ps, &vs), rng);
        });
        assert_eq!(used, 101, "100 burst draws + 1 ambient Next(5)");
        let mut dc = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc.ai[2] = 130.0;
        dc.ai[3] = 100.0;
        let used = draws_used(5, |rng| {
            caster_ai(&mut dc, &env(&w, &ps, &vs), rng);
        });
        assert_eq!(used, 1, "no burst draws, 1 ambient Next(3)");
    }

    /// `:18944-18948` — the >2000-px Manhattan bail draws NOTHING and
    /// finds nothing; the trigger/wrap still run.
    #[test]
    fn the_far_target_bail_draws_nothing() {
        let w = floored();
        let ps = [cand(9000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut n = sorcerer(1900.0, 1560.0);
        n.ai[0] = 649.0;
        let used = draws_used(9, |rng| {
            caster_ai(&mut n, &env(&w, &ps, &vs), rng);
        });
        assert_eq!(used, 1, "only the ambient Next(5)");
        assert_eq!((n.ai[2], n.ai[3]), (0.0, 0.0), "no spot");
        assert!(n.net_update, ":21105 fires even on a failed search");
    }

    /// `:21031-21038` + `:21079-21082` + `:21117-21119` — the Djinn's
    /// 180 trigger sets 181; the countdown fires the volley search at
    /// ai[1] = 120/90/60/30/0 (NOT 180/150 — the /30 < 5 gate), and the
    /// 360 wrap feeds the same-tick teleport.
    #[test]
    fn the_djinn_volley_window_and_early_wrap() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut dj = Npc::new(NpcKind::DesertDjinn, 1900.0, 1560.0, 1, 0.0);
        dj.ai[0] = 179.0; // ++ → 180 → the trigger
        let mut rng = UnifiedRandom::new(2);
        caster_ai(&mut dj, &env(&w, &ps, &vs), &mut rng);
        // The trigger sets 181, then the same tick's countdown decrements
        // to 180 — and 180/30 = 6 ≥ 5, so no volley yet (`:21119`).
        assert_eq!(dj.ai[1], 180.0);
        // Tick the countdown: 181 → 180 (no volley: /30 = 6). Pin ai[0]
        // low each sub-case so the 180 trigger / 360 wrap inject no draws.
        let before = draws_used(2, |rng| {
            dj.ai[0] = 10.0;
            dj.ai[1] = 181.0;
            caster_ai(&mut dj, &env(&w, &ps, &vs), rng);
        });
        assert_eq!(dj.ai[1], 180.0);
        assert_eq!(before, 0, "180/30 = 6 ≥ 5: no search, no draws (533 ambient is light-only)");
        // 121 → 120: the first volley tick draws (2 per iteration).
        let used = draws_used(2, |rng| {
            dj.ai[0] = 10.0;
            dj.ai[1] = 121.0;
            caster_ai(&mut dj, &env(&w, &ps, &vs), rng);
        });
        assert_eq!(dj.ai[1], 120.0);
        assert!(used >= 2 && used % 2 == 0, "the search draws X/Y pairs");
        // The early wrap: 359 → 360 → 650 → the teleport, same tick.
        dj.ai[0] = 359.0;
        dj.ai[1] = 0.0;
        caster_ai(&mut dj, &env(&w, &ps, &vs), &mut rng);
        assert_eq!(dj.ai[0], 1.0, "360 → 650 → the search resets to 1");
    }

    /// `:21164-21174` — the Fire Imp spawns the Burning Sphere 25 at
    /// ai[1] == 10 with INTEGER coordinates and the direction × 8 offset.
    #[test]
    fn the_fire_imp_spawns_at_windup_10_with_int_coords() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut imp = Npc::new(NpcKind::FireImp, 1900.7, 1560.9, 1, 0.0);
        imp.ai[0] = 10.0; // keep it off the triggers/teleport
        imp.ai[1] = 11.0; // -- → 10 → the spawn
        let mut rng = UnifiedRandom::new(4);
        let shot = caster_ai(&mut imp, &env(&w, &ps, &vs), &mut rng);
        let (w_px, _) = imp.size();
        assert_eq!(
            shot,
            Some(CasterShot::Sphere {
                npc_type: 25,
                x: 1900 + w_px as i32 / 2 + imp.facing * 8,
                y: 1560 + 20,
                ai3: 0.0,
            })
        );
    }

    /// `:21246` — the latched Dark Caster's Water Sphere carries
    /// ai3 = 1; an unlatched one carries 0.
    #[test]
    fn the_dark_caster_sphere_carries_the_red_hat_payload() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let mut e = env(&w, &ps, &vs);
        e.skeletron_red_hat = true;
        let mut dc = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc.ai[0] = 10.0;
        dc.ai[1] = 26.0; // -- → 25 → the shoot tick
        let mut rng = UnifiedRandom::new(6);
        let shot = caster_ai(&mut dc, &e, &mut rng);
        assert!(matches!(shot, Some(CasterShot::Sphere { npc_type: 33, ai3, .. }) if ai3 == 1.0));
        let mut plain = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        plain.ai[0] = 10.0;
        plain.ai[1] = 26.0;
        let shot = caster_ai(&mut plain, &env(&w, &ps, &vs), &mut rng);
        assert!(matches!(shot, Some(CasterShot::Sphere { npc_type: 33, ai3, .. }) if ai3 == 0.0));
    }

    /// `:18960-18970` — the else-if quirk: the dungeon gate branch is
    /// taken when the wall is NOT dungeon; a live Skeletron keeps ok
    /// true and the lava else-if NEVER runs. A dungeon wall falls to the
    /// else-if, so lava rejects it normally.
    #[test]
    fn the_dungeon_wall_gate_and_the_else_if_lava_quirk() {
        let mut w = floored();
        // NO dungeon wall above the floor row, but lava ON it.
        for x in 0..400 {
            let i = w.index(x, 99);
            w.tiles[i] = Tile { wall: 0, liquid: 255, ..Default::default() };
            w.tiles[i].flags |= FLAG_LIQUID_LAVA;
        }
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        // No wall, no Skeletron: the gate branch rejects every candidate.
        let mut dc = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc.ai[0] = 649.0;
        let mut rng = UnifiedRandom::new(8);
        caster_ai(&mut dc, &env(&w, &ps, &vs), &mut rng);
        assert_eq!((dc.ai[2], dc.ai[3]), (0.0, 0.0), "no wall, no Skeletron: refused");
        // With a live Skeletron the gate branch keeps ok true — and the
        // lava else-if is SKIPPED, so the lava-covered spot is admitted.
        let mut e = env(&w, &ps, &vs);
        e.any_skeletron = true;
        let mut dc2 = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc2.ai[0] = 649.0;
        caster_ai(&mut dc2, &e, &mut rng);
        assert!(dc2.ai[2] != 0.0, "AnyNPCs(35) admits the spot; the lava else-if never runs");
        // A dungeon wall (7) takes the ELSE-IF: lava now rejects.
        for x in 0..400 {
            let i = w.index(x, 99);
            w.tiles[i].wall = 7;
        }
        let mut dc3 = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc3.ai[0] = 649.0;
        caster_ai(&mut dc3, &env(&w, &ps, &vs), &mut rng);
        assert_eq!((dc3.ai[2], dc3.ai[3]), (0.0, 0.0), "dungeon wall + lava: the else-if rejects");
        // Clear the lava: the dungeon spot is admitted.
        for x in 0..400 {
            let i = w.index(x, 99);
            w.tiles[i].liquid = 0;
            w.tiles[i].flags &= !FLAG_LIQUID_LAVA;
        }
        let mut dc4 = Npc::new(NpcKind::DarkCaster, 1900.0, 1560.0, 1, 0.0);
        dc4.ai[0] = 649.0;
        caster_ai(&mut dc4, &env(&w, &ps, &vs), &mut rng);
        assert!(dc4.ai[2] != 0.0, "dungeon wall, no lava: admitted");
    }

    /// `:21279-21374` — the ambient gate draws exactly one roll per tick
    /// for 29 (Next(5)), 32 (Next(3)) and 24 (Next(6)); 533 draws none.
    #[test]
    fn the_ambient_gates_draw_one_per_tick_except_533() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        for (kind, expect) in [
            (NpcKind::GoblinSorcerer, 1),
            (NpcKind::DarkCaster, 1),
            (NpcKind::FireImp, 1),
            (NpcKind::DesertDjinn, 0),
        ] {
            let mut n = Npc::new(kind, 1900.0, 1560.0, 1, 0.0);
            n.ai[0] = 10.0;
            let used = draws_used(11, |rng| {
                caster_ai(&mut n, &env(&w, &ps, &vs), rng);
            });
            assert_eq!(used, expect, "{kind:?} ambient draw count");
        }
    }

    /// Determinism witness: the same seed and inputs produce the same
    /// state and the same draw count over a full 700-tick cycle+teleport.
    #[test]
    fn the_arm_is_deterministic_over_700_ticks() {
        let w = floored();
        let ps = [cand(2000.0, 1500.0)];
        let vs = [(0.0, 0.0)];
        let run = |seed: i32| {
            let mut n = sorcerer(1900.0, 1560.0);
            let mut rng = UnifiedRandom::new(seed);
            let mut shots = 0;
            for _ in 0..700 {
                if caster_ai(&mut n, &env(&w, &ps, &vs), &mut rng).is_some() {
                    shots += 1;
                }
            }
            (n.x, n.y, n.ai, n.vx, n.vy, shots, rng.peek())
        };
        assert_eq!(run(42), run(42));
    }
}
