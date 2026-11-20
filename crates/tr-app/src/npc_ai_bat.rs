//! The aiStyle-8 caster family — `NPC.AI()`'s `if (this.aiStyle == 8)` block
//! (`NPC.cs:20817-21376`). Despite the task slug ("bat/flyer family"), the C#'s
//! aiStyle 8 is the **teleporting caster** machine — the bats (49/51/60) ride
//! aiStyle 14 (`NPC.cs:22849`, the port's `update_cave_bat`); the style axis is
//! the C#'s dispatch key, so this module owns style 8:
//!
//! * type 24 Fire Imp (`SetDefaults` `NPC.cs:9078-9092`),
//! * type 29 Goblin Sorcerer (`:9326` group),
//! * type 32 Dark Caster (`:9121-9134`),
//! * type 45 Tim (`:11158` group),
//! * type 533 Desert Djinn (`:14974-14987`),
//! * types 281-286 the hardmode dungeon casters (`:12640-12723`) — unported
//!   kinds, deferred,
//! * type 172 Rune Wizard (`:17727` group) and 693 Librarian Skeleton —
//!   unported kinds, deferred.
//!
//! The seam (S1-D-3, code-only mode): `Npc::update_caster` (`npc.rs`, the
//! kind-dispatched arm — it also carries the Poltergeist aiStyle-22 stand-in)
//! folds the port's `caster_counter`/`cast_windup` mirrors into `ai[0]`/`ai[1]`,
//! runs `ai_style_8_step`, folds them back out and drains the shoot intent into
//! `wants_sphere`. Every other dispatch arm stays byte-identical.
//!
//! RNG order is by construction: every `Main.rand` draw this machine makes is
//! the teleport search's `Next(targetTileX - range, targetTileX + range + 1)`
//! then `Next(targetTileY - range, ...)` pair per attempt (`NPC.cs:18952-18953`,
//! cited inline below). W4-d52 wires the PORTED kinds' dust arms through the
//! live `DustPool` on the caller's stream: the teleport bursts' `Next(1, 3)`
//! scales (`:20867`/`:20936`) + the 50-dust loops, and the ambient tail gates
//! (`:21282` 29/45, `:21292` 32, `:21366` the else group). The unported kinds'
//! arms (172 `:21319`, 693/281-286 `:20882-20902`/`:21331-21355`) stay cited
//! skips — they land with their kinds. The Djinn's curse volley
//! (`:21140-21141`) and the Rune Wizard's aim jitter (`:21268-21269`) remain
//! the unported arms' draws.

use crate::npc::{Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use tr_core::UnifiedRandom;
use tr_core::World;
use tr_core::TILE_PX;

/// The one live player slot as the teleport search's telefrag gate sees it
/// (`NPC.cs:18982-18996` loops `Main.player`; the port has one session
/// player). Carries the velocity for the C#'s ±20-tick prediction rect
/// (`:18988-18989`).
#[derive(Clone, Copy, Debug)]
pub struct TelefragPlayer {
    /// `player.Hitbox` — `(int)position.X/Y`, 20×42 (`Player.cs`'s box).
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
    /// `player.velocity` — the `num4 = 20f` extrapolation (`:18943`).
    pub vx: f32,
    pub vy: f32,
    /// `player.active && !player.dead` (`:18985`) — a dead player does
    /// not block the spot.
    pub dead: bool,
}

impl TelefragPlayer {
    /// The session player's box + velocity (the port's stock 20×42 hitbox).
    pub fn from_player(p: &Player) -> Self {
        Self {
            x: p.x,
            y: p.y,
            width: crate::player::WIDTH as i32,
            height: crate::player::HEIGHT as i32,
            vx: p.vx,
            vy: p.vy,
            dead: p.dead,
        }
    }
}

/// `Main.wallDungeon` (`Main.cs:10596-10606`): walls 7/8/9/94-99.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// The C#'s type-32 / 281-286 / 693 group (`NPC.cs:18960`) — the teleport
/// spot must sit in front of a dungeon wall unless Skeletron is alive.
fn dungeon_wall_gated(kind: NpcKind) -> bool {
    matches!(kind, NpcKind::DarkCaster)
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): out of bounds = solid; a tile counts when
/// `active() && !inActive() && tileSolid[type] && !tileSolidTop[type]`.
fn solid_tiles(world: &World, start_x: i32, end_x: i32, start_y: i32, end_y: i32) -> bool {
    if start_x < 0 || end_x >= world.width || start_y < 0 || end_y >= world.height - 40 {
        return true;
    }
    for i in start_x..=end_x {
        for j in start_y..=end_y {
            if let Some(t) = world.tile(i, j) {
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

/// `NPC.AI_AttemptToFindTeleportSpot` (`NPC.cs:18937-19007`), the aiStyle-8
/// call shape (`:21097`: `rangeFromTargetTile = num88`, the other parameters
/// at their defaults — telefrag 5 tiles, fluff 1, not centered, not in air).
///
/// Per attempt: one X draw then one Y draw (`Main.rand.Next` at
/// `:18952`/`:18953` — the ONLY two draws this machine makes), the column
/// scanned downward from the drawn Y. Every rejection before the telefrag
/// test `continue`s the scan (`:18957`, `:18973`, `:18978`); only the
/// telefrag check ends the attempt — chosen or not (the `break` at
/// `:19003` sits outside the `flag2` test).
///
/// `skeletron_alive` is `NPC.AnyNPCs(35)` (`:18962`) — the port passes the
/// sim's live-head mirror (`world.skeletron_head_present`, false with no
/// live head). B-1 fix: the old `skeletron_red_hat` stand-in read the
/// hands count out of `ai[3]` (a live head with one hand read alive but a
/// two-hand head did not); the count now rides its own World mirror and
/// this gate reads presence directly.
pub fn attempt_find_teleport_spot(
    world: &World,
    rng: &mut UnifiedRandom,
    kind: NpcKind,
    self_center: (f32, f32),
    target_tile: (i32, i32),
    radius: i32,
    player: &TelefragPlayer,
    skeletron_alive: bool,
) -> Option<(i32, i32)> {
    // `:18939-18940` — `(int)base.Center.X / 16` (truncate, then divide).
    let num = self_center.0 as i32 / 16;
    let num2 = self_center.1 as i32 / 16;
    // `:18944-18948` — beyond 2000 Manhattan px the attempt counter starts
    // at 100: the while condition fails immediately, no draws at all.
    let mut attempts = 0;
    if (num * 16 - target_tile.0 * 16).abs() + (num2 * 16 - target_tile.1 * 16).abs() > 2000 {
        attempts = 100;
    }
    while attempts < 100 {
        attempts += 1;
        // The two `Main.rand.Next` draws, X first then Y (`:18952-18953`).
        let x = rng.next_range(target_tile.0 - radius, target_tile.0 + radius + 1);
        let y_start = rng.next_range(target_tile.1 - radius, target_tile.1 + radius + 1);
        let mut i = y_start;
        while i < target_tile.1 + radius {
            // `:18955-18958` — the self box / the inactive skip continue.
            // `nactive()` (`Tile.cs:240-247`) is active-and-not-actuated.
            if (i >= num2 - 1 && i <= num2 + 1 && x >= num - 1 && x <= num + 1)
                || !world
                    .tile(x, i)
                    .is_some_and(|t| t.active() && !t.inactive())
            {
                i += 1;
                continue;
            }
            let mut ok = true;
            // `:18960-18970` — the dungeon-wall gate for 32/281-286/693
            // (else the lava check on the tile ABOVE the floor).
            if dungeon_wall_gated(kind)
                && !world
                    .tile(x, i - 1)
                    .is_some_and(|t| WALL_DUNGEON.contains(&t.wall))
            {
                if !skeletron_alive {
                    ok = false;
                }
            } else if world.tile(x, i - 1).is_some_and(|t| {
                t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
            }) {
                ok = false;
            }
            // `:18971-18974` — the gate/solid rejection CONTINUES the scan
            // (the C#'s `continue` at `:18973`), the floor itself must be
            // tileSolid.
            if !ok
                || !world
                    .tile(x, i)
                    .is_some_and(|t| tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                i += 1;
                continue;
            }
            // `:18976-18979` — the body space: `SolidTiles(x-1, x+1, i-4,
            // i-1)` must be false (fluff 1, not centered); blocked CONTINUES.
            if solid_tiles(world, x - 1, x + 1, i - 4, i - 1) {
                i += 1;
                continue;
            }
            // `:18980-18996` — the telefrag rect: the floor tile's 16×16
            // box inflated by 5 tiles (80 px), tested against the player's
            // hitbox UNION its ±(int)(velocity * 20) displaced copy
            // (`Modified` + `Rectangle.Union`, `Utils.cs:1603-1606`).
            // A dead player is skipped (`:18985`).
            let rect = (x * 16 - 80, i * 16 - 80, 176, 176);
            let mut blocked = false;
            if !player.dead {
                let hx = player.x as i32;
                let hy = player.y as i32;
                let dx = (player.vx * 20.0) as i32;
                let dy = (player.vy * 20.0) as i32;
                let ux = hx.min(hx + dx);
                let uy = hy.min(hy + dy);
                let uw = player.width + dx.abs();
                let uh = player.height + dy.abs();
                blocked = ux < rect.0 + rect.2
                    && rect.0 < ux + uw
                    && uy < rect.1 + rect.3
                    && rect.1 < uy + uh;
            }
            if !blocked {
                // `:18998-19002` — chosen.
                return Some((x, i));
            }
            // `:19003` — the telefrag reject ends the attempt either way.
            break;
        }
    }
    None
}

/// The aiStyle-8 step's outcome: the sphere/imp spawn intent the caller
/// drains into `wants_sphere` (kind, bottom-centre X, bottom-centre Y, the
/// Water Sphere's `ai[3]` red-hat payload).
#[derive(Clone, Copy, Debug)]
pub struct CasterShot {
    pub kind: NpcKind,
    pub x: f32,
    pub y: f32,
    pub ai3: f32,
}

/// The whole `if (this.aiStyle == 8)` block (`NPC.cs:20817-21376`) for the
/// ported kinds. State rides `npc.ai[0]` (the cycle counter), `npc.ai[1]`
/// (the windup countdown), `npc.ai[2]`/`ai[3]` (the pending teleport tile —
/// consumed on the NEXT tick, `:20855-20992`). The caller folds its
/// `caster_counter`/`cast_windup` mirrors into `ai[0]`/`ai[1]` around this.
///
/// Order is vanilla: the red-hat latch (`:20819-20828`), `TargetClosest`
/// (`:20829` — the shared `npc_target` helper), the 0.93 stand decay
/// (`:20831-20835`), the 500 init (`:20836-20843`), the pending-teleport
/// placement (`:20855-20992`, dust skipped), `ai[0]++` (`:20993`), the
/// firing-distance gate (`:21006`), the per-type triggers (`:21011-21082`),
/// the 650 wrap + teleport attempt (`:21083-21112`), the windup countdown +
/// the shoot (`:21113-21278`). The generic gravity + `UpdateCollision`
/// (`:90465-90493`) runs AFTER the AI returns in `NPC.UpdateNPC`; the port
/// applies it here at the tail (the documented ordering deviation, ANALYSIS).
pub fn ai_style_8_step(
    npc: &mut Npc,
    world: &World,
    player: &Player,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) -> Option<CasterShot> {
    // `:20819` — `flag4 = RedHatSkeletronAdjustmentsEnabled()`.
    let mut flag4 = npc.red_hat_skeletron_adjustments_enabled();
    // `:20820-20828` — the Dark Caster latches `localAI[3] = 1` while the
    // first Skeletron head's `ai[3] == 1` (`FindFirstNPC(35)`); the port's
    // sim write-through mirror stands for it. No reset arm exists.
    if npc.kind == NpcKind::DarkCaster && world.skeletron_red_hat {
        npc.local_ai[3] = 1.0;
        flag4 = true;
    }
    // `:20829` — `TargetClosest()` (the default `faceTarget: true`). The
    // stock candidate mirrors the zombie wiring (`npc.rs::update_zombie`):
    // aggro/npcTypeNoAggro/tankPet have no port fields yet; the ticker is
    // not plumbed (0 — the stamp's only reader is the unported fade).
    let cand = TargetCandidate {
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
    target_closest(npc, world, &[cand], true, 8, 0);
    // `:20831-20835` — the stand decay with the ±0.1 snap.
    npc.vx *= 0.93;
    if npc.vx > -0.1 && npc.vx < 0.1 {
        npc.vx = 0.0;
    }
    // `:20836-20843` — the cycle init (the 693 localAI[1] arm is deferred).
    if npc.ai[0] == 0.0 {
        npc.ai[0] = 500.0;
    }
    // `:20844-20854` — the type-172 alpha ramp: deferred (Rune Wizard is
    // not a ported kind).
    // `:20855-20992` — the pending teleport: place the body on the tile,
    // zero the velocity, clear ai[2]/ai[3]. W4-d52 wires the two 50-dust
    // bursts (sites 20867/20876/20888-20918 departure, 20936/20945/
    // 20957-20987 arrival) through the live pool: 29/45 draw Next(1,3)
    // as the scale ARGUMENT 50× per burst (the draw happens even when
    // NewDust culls); 32's type-172 and the else group's type-6 use
    // constant scales. The unported kinds' arms (172/693/281-286) stay
    // cited skips. S1-E-21g wires BOTH Item8 plays (`:20862` the
    // departure at the OLD position BEFORE the move, `:20931` the
    // arrival AFTER it) — the queue keeps the C#'s order around the
    // placement.
    if npc.ai[2] != 0.0 && npc.ai[3] != 0.0 {
        let (w, h) = npc.size();
        // `:20862` — the departure puff at the pre-teleport position.
        npc.wants_sound.push(crate::npc::NpcSoundRequest {
            cue: crate::npc::NpcSoundCue::Item8,
            x: npc.x,
            y: npc.y,
        });
        // `:20863-20922` — the departure burst (the netOffset pair
        // `:20857`/`:20923` nets to zero single-player).
        caster_teleport_burst(npc, rng, dust, dust_ctx, world);
        // `:20924-20925` — `(float)(base.width / 2)` is INTEGER division.
        npc.x = npc.ai[2] * 16.0 - (w as i32 / 2) as f32 + 8.0;
        npc.y = npc.ai[3] * 16.0 - h;
        npc.vx = 0.0;
        npc.vy = 0.0;
        npc.ai[2] = 0.0;
        npc.ai[3] = 0.0;
        // `:20931` — the arrival puff at the placed position.
        npc.wants_sound.push(crate::npc::NpcSoundRequest {
            cue: crate::npc::NpcSoundCue::Item8,
            x: npc.x,
            y: npc.y,
        });
        // `:20932-20991` — the arrival burst at the NEW position.
        caster_teleport_burst(npc, rng, dust, dust_ctx, world);
    }
    // `:20993` — the counter tick (the 693 localAI block `:20994-21005` is
    // deferred with its kind).
    npc.ai[0] += 1.0;
    // `:21006-21010` — `flag5 = AI_GlobalFiringDistanceCheck(Center,
    // player.Center)`; the Goblin Sorcerer (29) is always in range.
    let mut flag5 = global_firing_distance_check(npc.center(), player.center());
    if npc.kind == NpcKind::GoblinSorcerer {
        flag5 = true;
    }
    // `:21011-21050` — the 283/284, 172, 533, 281/282 trigger arms. Only
    // the Djinn's is ported; the dungeon casters / Rune Wizard are deferred.
    if npc.kind == NpcKind::DesertDjinn {
        // `:21031-21038` — the single 180 trigger winds the 181-tick volley.
        if flag5 && npc.ai[0] == 180.0 {
            npc.ai[1] = 181.0;
            npc.net_update = true;
        }
    } else {
        // `:21051-21074` — the else group. `:21053-21060`'s getGoodWorld
        // Fire-Imp-vs-WoF stall needs `NPC.AnyNPCs(113)`; the port's WoF
        // presence mirror (`world.wof_present`) stands for it.
        if npc.kind == NpcKind::FireImp
            && world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY)
            && world.wof_present
        {
            npc.ai[0] += 1.0;
            if npc.ai[0] % 2.0 == 1.0 {
                npc.ai[0] -= 1.0;
            }
        }
        // `:21061-21068` — the red-hat double-step: a second `++` with
        // the odd rollback. An even counter advances +2 per tick (the
        // red-hat casters fire twice as fast); an odd one lands on the
        // next even value (+1).
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
    // `:21075-21078` — the 285/286 early wrap: deferred kinds.
    // `:21079-21082` — the Djinn's 360 wrap feeds the 650 block below.
    if npc.kind == NpcKind::DesertDjinn && npc.ai[0] >= 360.0 {
        npc.ai[0] = 650.0;
    }
    // `:21083-21112` — the teleport attempt (single-player: netMode != 1).
    if npc.ai[0] >= 650.0 {
        npc.ai[0] = 1.0;
        // `:21086-21087` — `(int)player.Center.X / 16` (truncate first).
        let (pcx, pcy) = player.center();
        let target_tile = (pcx as i32 / 16, pcy as i32 / 16);
        // `:21089-21093` — num88 = 20, the Fire Imp 5.
        let num88 = if npc.kind == NpcKind::FireImp { 5 } else { 20 };
        // `:21094` — the 693 near-books attempt is deferred with its kind.
        // `:21097` — the shared attempt; `AnyNPCs(35)` rides the sim's
        // live-head presence mirror (B-1: was the red-hat hands-count
        // stand-in).
        if let Some((tx, ty)) = attempt_find_teleport_spot(
            world,
            rng,
            npc.kind,
            npc.center(),
            target_tile,
            num88,
            &TelefragPlayer::from_player(player),
            world.skeletron_head_present,
        ) {
            // `:21099-21104` — the destination rides ai[2]/ai[3]; the
            // placement happens on the NEXT tick (`:20855`).
            npc.ai[1] = num88 as f32;
            npc.ai[2] = tx as f32;
            npc.ai[3] = ty as f32;
        }
        npc.net_update = true;
        // `:21106-21111` — the 693 localAI reset: deferred.
    }
    // `:21113-21278` — the windup countdown and the shoot.
    let mut shot = None;
    if npc.ai[1] > 0.0 {
        let num89 = 25.0f32;
        npc.ai[1] -= 1.0;
        if npc.kind == NpcKind::DesertDjinn {
            // `:21117-21162` — the `% 30` curse volley draws Main.rand
            // (`:21140-21141`) per shot and is skipped wholesale (the
            // S1-E-5 RNG rule, recorded in ANALYSIS): the Djinn never fires.
        } else if npc.kind == NpcKind::FireImp {
            // `:21164-21173` — the imp spawns the Burning Sphere 25 at
            // ai[1] == 10, at `(int)pos.X + width/2 + direction*8,
            // (int)pos.Y + 20` (the int truncation is the call site's).
            if npc.ai[1] == 10.0 {
                // S1-E-21g — `NPC.cs:21168`: the imp's own cast puff
                // (Item8 BEFORE the spawn, the C#'s order).
                npc.wants_sound.push(crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::Item8,
                    x: npc.x,
                    y: npc.y,
                });
                let (w, _) = npc.size();
                shot = Some(CasterShot {
                    kind: NpcKind::BurningSphere,
                    x: npc.x as i32 as f32 + (w as i32 / 2) as f32 + npc.facing as f32 * 8.0,
                    y: npc.y as i32 as f32 + 20.0,
                    ai3: 0.0,
                });
            }
        } else if npc.ai[1] == num89 {
            // `:21175-21277` — the standard shoot tick. The 281-286
            // projectile arm (`:21177-21227`) and the 693/172 arms are
            // deferred with their kinds.
            // S1-E-21g — `NPC.cs:21230-21233`: the cast's Item8 puff
            // (`this.type != 172` — the Rune Wizard is silent; no ported
            // kind is 172, every caster here plays).
            npc.wants_sound.push(crate::npc::NpcSoundRequest {
                cue: crate::npc::NpcSoundCue::Item8,
                x: npc.x,
                y: npc.y,
            });
            let (w, _) = npc.size();
            // `:21238`/`:21242`/`:21246` — `(int)pos.X + width/2,
            // (int)pos.Y - 8` (NPC.NewNPC's Bottom convention, `:80770`).
            let (sx, sy) = (
                npc.x as i32 as f32 + (w as i32 / 2) as f32,
                npc.y as i32 as f32 - 8.0,
            );
            match npc.kind {
                NpcKind::GoblinSorcerer => {
                    shot = Some(CasterShot { kind: NpcKind::ChaosBall, x: sx, y: sy, ai3: 0.0 });
                }
                NpcKind::Tim => {
                    // `:21242` — type 665, the ChaosBallTim clone; the port
                    // folds it into ChaosBall (identical stats and name).
                    shot = Some(CasterShot { kind: NpcKind::ChaosBall, x: sx, y: sy, ai3: 0.0 });
                }
                NpcKind::DarkCaster => {
                    // `:21246` — the Water Sphere 33 carries
                    // `ai[3] = flag4 ? 1 : 0`.
                    shot = Some(CasterShot {
                        kind: NpcKind::WaterSphere,
                        x: sx,
                        y: sy,
                        ai3: if flag4 { 1.0 } else { 0.0 },
                    });
                }
                _ => {}
            }
        }
    }
    // `:21279-21374` — the per-type dust/light tail: the lights are the
    // frame pass's feed (`npc_lights.rs::npc_light`, S1-E-5). W4-d52
    // wires the ported kinds' dust arms (the netOffset pair
    // `:21279`/`:21374` nets to zero single-player): 29/45 the Next(5)
    // gate + type-27 (`:21282-21288`, site 21284), 32 the Next(3) gate +
    // type-172 (`:21292-21299`, site 21294), the else group (24) the
    // Next(6) gate + type-6 (`:21366-21372`, site 21368). 533 draws
    // NOTHING (light only, `:21358-21361`); 172/693/281-286's arms are
    // unported-kind skips.
    caster_ambient_dust(npc, rng, dust, dust_ctx, world);
    // The generic movement tail — `NPC.UpdateNPC`'s gravity + collision
    // (`NPC.cs:90465-90493`) runs after `AI()` returns; the port applies it
    // at the end of this step (the ordering deviation is recorded in
    // ANALYSIS: the windup countdown sees the pre-gravity position, exactly
    // like the C#'s AI-then-collision split).
    let (w, h) = npc.size();
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
    }
    npc.vy = (npc.vy + crate::npc::GRAVITY).min(crate::npc::MAX_FALL);
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
        npc.on_ground = false;
    } else if npc.vy > 0.0 {
        let ty = ((ny + h) / TILE_PX).floor() as i32;
        npc.y = ty as f32 * TILE_PX - h;
        npc.vy = 0.0;
        npc.on_ground = true;
    } else {
        npc.vy = 0.0;
    }
    shot
}

/// `NPC.AI_GlobalFiringDistanceCheck` (`NPC.cs:55835-55841`): the target's
/// centre point seeds `Utils.CenteredRectangle(center, MaxWorldViewSize)`
/// (1920×1200, `Main.cs:677-681`/`:66714`), deflated by 50 per side, and the
/// shoot position's truncated point must sit inside. Both `.ToPoint()`
/// calls truncate toward zero (`Utils.cs:952-955`'s Point overload).
pub fn global_firing_distance_check(shoot: (f32, f32), target: (f32, f32)) -> bool {
    let rx = target.0 as i32 - 1920 / 2 + 50;
    let ry = target.1 as i32 - 1200 / 2 + 50;
    let (rw, rh) = (1920 - 100, 1200 - 100);
    let sx = shoot.0 as i32;
    let sy = shoot.1 as i32;
    sx >= rx && sx < rx + rw && sy >= ry && sy < ry + rh
}

/// W4-d52 — one 50-dust teleport burst (`NPC.cs:20863-20922` departure /
/// `:20932-20991` arrival — the two loops are per-type identical). The
/// ported kinds: 29/45 → type 27 with the `Next(1,3)` scale ARGUMENT
/// (drawn even when `NewDust` culls), 32 → type 172 scale 1.5, 533 →
/// type 27 scale 2.5, the else group (24) → type 6 scale 2.5. The
/// unported kinds' arms (172/693/281-286) are cited skips — they land
/// with their kinds. All post-spawn writes land on whatever slot
/// `NewDust` returned (sentinel included).
fn caster_teleport_burst(
    npc: &Npc,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
) {
    use tr_core::particles::Color;
    let (w, h) = npc.size();
    let pos = glam::Vec2::new(npc.x, npc.y);
    let (wi, hi) = (w as i32, h as i32);
    for _ in 0..50 {
        match npc.kind {
            // `:20865-20873` / `:20934-20942` — 29/45, type 27.
            NpcKind::GoblinSorcerer | NpcKind::Tim => {
                // The scale argument draws FIRST (even on a cull).
                let scale_arg = rng.next_range(1, 3) as f32;
                let idx = dust.new_dust(
                    rng, ctx, world, pos, wi, hi, 27, 0.0, 0.0, 100, Color::default(), scale_arg,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 3.0;
                // `:20869` reads the dust's FINAL scale (post-NewDust).
                if d.scale > 1.0 {
                    d.no_gravity = true;
                }
            }
            // `:20874-20879` / `:20943-20948` — 32, type 172.
            NpcKind::DarkCaster => {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, wi, hi, 172, 0.0, 0.0, 100, Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 3.0;
                d.no_gravity = true;
            }
            // `:20910-20915` / `:20979-20984` — 533, type 27 scale 2.5.
            NpcKind::DesertDjinn => {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, wi, hi, 27, 0.0, 0.0, 100, Color::default(), 2.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 3.0;
                d.no_gravity = true;
            }
            // `:20916-20921` / `:20985-20990` — the else group (24), type 6.
            _ => {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, wi, hi, 6, 0.0, 0.0, 100, Color::default(), 2.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 3.0;
                d.no_gravity = true;
            }
        }
    }
}

/// W4-d52 — the caster's per-tick ambient dust tail (`NPC.cs:21279-21374`):
/// 29/45 the `Next(5)==0` gate + type 27 (`:21282-21288`), 32 the
/// `Next(3)!=0` gate + type 172 (`:21292-21299`), the else group (24) the
/// `Next(6)==0` gate + type 6 (`:21366-21372`). 533 draws NOTHING (light
/// only). The gate draw runs EVERY tick for the gated kinds. Position is
/// `(x, y + 2)`; the unported kinds' arms are cited skips.
fn caster_ambient_dust(
    npc: &Npc,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &World,
) {
    use tr_core::particles::Color;
    let (w, h) = npc.size();
    let pos = glam::Vec2::new(npc.x, npc.y + 2.0);
    let (wi, hi) = (w as i32, h as i32);
    match npc.kind {
        NpcKind::GoblinSorcerer | NpcKind::Tim => {
            // `:21282` — Next(5) == 0 gates the dust.
            if rng.next_max(5) != 0 {
                return;
            }
            let idx = dust.new_dust(
                rng, ctx, world, pos, wi, hi, 27, npc.vx * 0.2, npc.vy * 0.2, 100,
                Color::default(), 1.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x *= 0.5;
            d.velocity.y = -2.0;
        }
        NpcKind::DarkCaster => {
            // `:21292` — Next(3) != 0 gates the dust.
            if rng.next_max(3) == 0 {
                return;
            }
            let idx = dust.new_dust(
                rng, ctx, world, pos, wi, hi, 172, npc.vx * 0.2, npc.vy * 0.2, 100,
                Color::default(), 0.9,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x *= 0.3;
            d.velocity.y *= 0.2;
            d.velocity.y -= 1.0;
        }
        NpcKind::DesertDjinn => {
            // `:21358-21361` — light only, no dust, no draw.
        }
        _ => {
            // `:21366` — the else group (24), Next(6) == 0 gates the dust.
            if rng.next_max(6) != 0 {
                return;
            }
            let idx = dust.new_dust(
                rng, ctx, world, pos, wi, hi, 6, npc.vx * 0.2, npc.vy * 0.2, 100,
                Color::default(), 1.65,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity.x *= 1.0;
            d.velocity.y *= 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn player_at(x: f32, y: f32) -> Player {
        let w = ground_air();
        let mut p = Player::spawn(&w);
        p.x = x;
        p.y = y;
        p
    }

    /// W4-d52 — the solo-test entry: a fresh scratch pool + the permissive
    /// whole-world ctx (the `update_rng` house pattern), so the witnesses
    /// keep their exact call shape while the dust arms draw on the same
    /// stream. The live sim threads the real pool through the 6-arg step.
    fn step_solo(
        n: &mut Npc,
        w: &World,
        p: &Player,
        rng: &mut UnifiedRandom,
    ) -> Option<CasterShot> {
        let mut dust = tr_core::particles::DustPool::new();
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        ai_style_8_step(n, w, p, rng, &mut dust, &ctx)
    }

    /// `NPC.cs:20836-20843` + `:20993` — the first tick inits ai[0] to 500
    /// and the same tick's increment leaves 501; hand arithmetic:
    /// 0 → 500 → 501, then +1 per tick.
    #[test]
    fn the_counter_inits_at_500_and_ticks() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        let mut rng = UnifiedRandom::new(7);
        step_solo(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[0], 501.0, "0 -> 500 init -> 501 (same-tick ++)");
        step_solo(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[0], 502.0);
    }

    /// `NPC.cs:21069-21073` + `:21113-21116` + `:21175` — the trigger at
    /// ai[0] == 100 sets ai[1] = 30, the SAME tick's countdown reads 29;
    /// the shoot fires when the countdown reaches 25 (five ticks later:
    /// 29, 28, 27, 26, 25).
    #[test]
    fn the_standard_trigger_winds_30_and_fires_at_25() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        // Resting on the 120-row floor (y = 1920 - 40): the position is
        // integral, so the C#'s `(int)pos` truncation is exact and the
        // post-step position equals the shoot-tick position.
        let mut n = Npc::new(NpcKind::Tim, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 99.0; // the next tick's ++ lands exactly on 100
        let mut rng = UnifiedRandom::new(7);
        let shot = step_solo(&mut n, &w, &p, &mut rng);
        assert!(shot.is_none());
        assert_eq!(n.ai[0], 100.0);
        assert_eq!(n.ai[1], 29.0, "30 minus the same-tick decrement");
        let mut fired = None;
        for expect in [28.0, 27.0, 26.0, 25.0] {
            let s = step_solo(&mut n, &w, &p, &mut rng);
            assert_eq!(n.ai[1], expect);
            if expect == 25.0 {
                fired = s;
            } else {
                assert!(s.is_none());
            }
        }
        let s = fired.expect("the 25 tick fires");
        assert_eq!(s.kind, NpcKind::ChaosBall);
        // `:21242` — Bottom = ((int)x + width/2, (int)y - 8).
        assert_eq!(s.x, n.x as i32 as f32 + 9.0);
        assert_eq!(s.y, n.y as i32 as f32 - 8.0);
    }

    /// `NPC.cs:21164-21173` — the Fire Imp fires at ai[1] == 10, twenty
    /// ticks after the 30 windup, at `(int)x + w/2 + direction*8, (int)y+20`.
    #[test]
    fn the_fire_imp_fires_at_10_with_the_direction_offset() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::FireImp, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 99.0;
        let mut rng = UnifiedRandom::new(7);
        let mut fired = None;
        for _ in 0..25 {
            if let Some(s) = step_solo(&mut n, &w, &p, &mut rng) {
                fired = Some(s);
                break;
            }
        }
        let s = fired.expect("the imp fires");
        assert_eq!(s.kind, NpcKind::BurningSphere);
        assert_eq!(n.ai[1], 10.0);
        assert_eq!(s.x, n.x as i32 as f32 + 9.0 + n.facing as f32 * 8.0);
        assert_eq!(s.y, n.y as i32 as f32 + 20.0);
    }

    /// `NPC.cs:21031-21038` + `:21079-21082` — the Djinn's single 180
    /// trigger sets 181 (reads 180 after the same-tick decrement) and the
    /// 360 wrap resets to 1 via the 650 block; the rand-drawing volley
    /// never fires.
    #[test]
    fn the_djinn_triggers_at_180_wraps_at_360_and_never_fires() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::DesertDjinn, 96.0 * 16.0, 120.0 * 16.0 - 62.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 179.0;
        let mut rng = UnifiedRandom::new(7);
        let s = step_solo(&mut n, &w, &p, &mut rng);
        assert!(s.is_none());
        assert_eq!(n.ai[0], 180.0);
        assert_eq!(n.ai[1], 180.0, "181 minus the same-tick decrement");
        let mut wrapped = false;
        for _ in 0..400 {
            let before = n.ai[0];
            let s = step_solo(&mut n, &w, &p, &mut rng);
            assert!(s.is_none(), "the Djinn's volley stays skipped");
            assert!(n.ai[0] < 650.0, "the wrap keeps the counter under 650");
            if before == 359.0 {
                assert_eq!(n.ai[0], 1.0, "360 -> 650 -> 1 in one tick");
                wrapped = true;
            }
        }
        assert!(wrapped, "the Djinn wrapped at 360 within 400 ticks");
    }

    /// `NPC.cs:21061-21068` — the red-hat double-step: the unconditional
    /// `ai[0]++` (`:20993`) plus flag4's second `++` with the odd rollback.
    /// From an EVEN counter the pair advances +2 and lands even; the
    /// red-hat casters therefore cast twice as fast. Hand arithmetic from
    /// ai[0]=98: `:20993` ++→99, flag4 ++→100 (even, kept) → the 100
    /// trigger winds ai[1]=30, the same-tick `--` reads 29.
    #[test]
    fn the_red_hat_double_step_hits_the_trigger_twice_as_fast() {
        let mut w = ground_air();
        w.skeletron_red_hat = true;
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::DarkCaster, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 98.0;
        let mut rng = UnifiedRandom::new(7);
        let s = step_solo(&mut n, &w, &p, &mut rng);
        assert!(s.is_none(), "the 29 windup has not reached 25");
        assert_eq!(n.ai[0], 100.0, "98 +2 (the flag4 double-step)");
        assert_eq!(n.ai[1], 29.0, "the 100 trigger's 30 minus the same-tick --");
        assert_eq!(n.local_ai[3], 1.0, "the latch (:20825)");
        assert_eq!(n.ai[0] % 2.0, 0.0, "flag4 always lands the counter even");
        // A non-latched caster from 98 advances only +1 → 99, no trigger.
        let mut m = Npc::new(NpcKind::Tim, 96.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        m.hp = m.max_hp;
        m.ai[0] = 98.0;
        let mut rng2 = UnifiedRandom::new(7);
        step_solo(&mut m, &w, &p, &mut rng2);
        assert_eq!(m.ai[0], 99.0, "no flag4: the plain +1");
        assert_eq!(m.ai[1], 0.0, "99 is not a trigger tick");
    }

    /// `NPC.cs:18952-18953` — the search draws exactly two Next calls per
    /// attempt, X first then Y: a stream fork proves the first successful
    /// attempt consumed the same two draws in the same order.
    #[test]
    fn the_teleport_search_draws_x_then_y_per_attempt() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        let mut rng = UnifiedRandom::new(7);
        let spot = attempt_find_teleport_spot(
            &w,
            &mut rng,
            NpcKind::Tim,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            false,
        );
        assert!(spot.is_some(), "the flat ground offers spots");
        let (sx, sy) = spot.unwrap();
        // The chosen spot passes every gate: a solid floor, free body
        // space, and the telefrag rect clear of the resting player.
        assert!(w
            .tile(sx, sy)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)));
        assert!(!solid_tiles(&w, sx - 1, sx + 1, sy - 4, sy - 1));
        assert!(
            (sx * 16 + 96 <= 1600 || sx * 16 - 80 >= 1620),
            "the telefrag rect misses the player's 1600..1620 hitbox"
        );
        // The stream proof: the search consumed a whole number of (X, Y)
        // pairs in exactly the C#'s ranges — a probe replaying those pairs
        // from the same seed lands on the search stream's state. Any extra
        // or reordered draw would desynchronize it.
        let mut pairs = 0;
        let mut probe = UnifiedRandom::new(7);
        loop {
            let _x = probe.next_range(80, 121);
            let _y = probe.next_range(97, 138);
            pairs += 1;
            if probe == rng {
                break;
            }
            assert!(pairs <= 100, "at most 100 attempts");
        }
        assert!(pairs >= 1);
    }

    /// `NPC.cs:18944-18948` — beyond 2000 Manhattan px the attempt counter
    /// starts at 100: zero draws, no spot.
    #[test]
    fn the_far_target_burns_no_draws() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        // Self at tile 10, target at tile 300: |160 - 4800| > 2000.
        let spot = attempt_find_teleport_spot(
            &w,
            &mut rng,
            NpcKind::Tim,
            (10.0 * 16.0, 110.0 * 16.0),
            (300, 117),
            20,
            &tp,
            false,
        );
        assert!(spot.is_none());
        assert_eq!(rng, before, "the >2000 gate consumes no draws");
    }

    /// `NPC.cs:18960-18966` — the Dark Caster's dungeon-wall gate: on a
    /// wall-less world with no Skeletron the search rejects every spot
    /// (but still draws its pairs); a live head opens the gate.
    #[test]
    fn the_dark_caster_needs_a_dungeon_wall_or_a_live_skeletron() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let tp = TelefragPlayer::from_player(&p);
        let mut rng = UnifiedRandom::new(7);
        let spot = attempt_find_teleport_spot(
            &w,
            &mut rng,
            NpcKind::DarkCaster,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            false,
        );
        assert!(spot.is_none(), "no dungeon walls, no head: no spot");
        let mut rng2 = UnifiedRandom::new(7);
        let spot2 = attempt_find_teleport_spot(
            &w,
            &mut rng2,
            NpcKind::DarkCaster,
            (40.0 * 16.0, 110.0 * 16.0),
            (100, 117),
            20,
            &tp,
            true,
        );
        assert!(spot2.is_some(), "AnyNPCs(35) opens the gate");
    }

    /// `NPC.cs:55835-55841` — the firing rect: the target centre ±(960,600)
    /// deflated 50 → ±(910,550); the boundary points truncate first.
    #[test]
    fn the_firing_rect_is_the_deflated_max_world_view() {
        let target = (1600.0, 1600.0);
        // Inside: 909 px right, 549 px down.
        assert!(global_firing_distance_check((2509.0, 2149.0), target));
        // Outside: 910 px right (the deflated edge is exclusive).
        assert!(!global_firing_distance_check((2510.0, 1600.0), target));
        assert!(!global_firing_distance_check((1600.0, 2150.0), target));
        // The truncation: 2509.9 still reads 2509.
        assert!(global_firing_distance_check((2509.9, 1600.0), target));
    }

    /// `NPC.cs:20855-20930` — the pending teleport places the body on the
    /// NEXT tick: ai[2]/ai[3] hold the tile, the placement zeroes the
    /// velocity and the pair; X centres with INTEGER width/2.
    #[test]
    fn the_pending_teleport_places_on_the_next_tick() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[2] = 60.0;
        n.ai[3] = 120.0;
        n.vx = 5.0;
        n.vy = 3.0;
        let mut rng = UnifiedRandom::new(7);
        step_solo(&mut n, &w, &p, &mut rng);
        // 60*16 - 18/2 + 8 = 960 - 9 + 8 = 959; the gravity tail then
        // settles the feet on the 120-row floor (y = 1920 - 40 = 1880).
        assert_eq!(n.x, 959.0);
        assert_eq!(n.y, 120.0 * 16.0 - 40.0);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0);
        assert_eq!(n.vy, 0.0, "the placement zeroes vy before the floor stop");
    }

    /// W4-d52 — one `NewDust` default block on an empty pool under the
    /// 50 % band (`Dust.cs:153-193`): the two position rolls, the
    /// velocity pair, the frame `Next(3)`, the scale roll; the type-6
    /// tweak adds one `Next(-10,6)` (`Dust.cs:214-220`), 27/172 none.
    fn burst_block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if dust_type == 6 {
            r.next_range(-10, 6);
        }
    }

    /// W4-d52 — `NPC.cs:20863-20922` / `:20932-20991`: the 50-dust burst's
    /// draw census per ported kind. 29/45 draw the `Next(1,3)` scale
    /// ARGUMENT before every block (even on a cull); 32/533/24 use
    /// constant scales. The mirror replays one burst per kind from one
    /// seed; the pool drains between kinds so the census stays on the
    /// empty-pool path.
    #[test]
    fn w4_d52_caster_teleport_burst_draws_match_the_oracle() {
        let w = ground_air();
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        for kind in [
            NpcKind::GoblinSorcerer,
            NpcKind::Tim,
            NpcKind::DarkCaster,
            NpcKind::FireImp,
            NpcKind::DesertDjinn,
        ] {
            let mut n = Npc::new(kind, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
            n.hp = n.max_hp;
            let (bw, bh) = n.size();
            let (bw, bh) = (bw as i32, bh as i32);
            let mut rng = UnifiedRandom::new(52);
            let mut probe = UnifiedRandom::new(52);
            let mut dust = tr_core::particles::DustPool::new();
            caster_teleport_burst(&n, &mut rng, &mut dust, &ctx, &w);
            for i in 0..50 {
                match kind {
                    NpcKind::GoblinSorcerer | NpcKind::Tim => {
                        probe.next_range(1, 3);
                        burst_block(&mut probe, 27, bw, bh);
                    }
                    NpcKind::DarkCaster => burst_block(&mut probe, 172, bw, bh),
                    NpcKind::DesertDjinn => burst_block(&mut probe, 27, bw, bh),
                    _ => burst_block(&mut probe, 6, bw, bh),
                }
                assert_eq!(rng.peek(), probe.peek(), "{kind:?} burst stream @{i}");
                // Drain per dust: the census stays on the empty-pool
                // path (the live pool's pressure bands are the d50
                // chain's wiring-wave concern).
                for s in dust.slots_mut() {
                    s.active = false;
                }
            }
            // The post-spawn writes: velocity x3, noGravity (29/45 only
            // when the FINAL scale > 1 — the scale-2 argument always
            // clears it, the scale-1 arm keeps gravity). The drain above
            // cleared `active` only — the field values survive.
            let d = dust.slot(0);
            match kind {
                NpcKind::GoblinSorcerer | NpcKind::Tim => {
                    assert_eq!(d.dust_type, 27);
                    assert!(d.scale > 1.0 || !d.no_gravity);
                }
                NpcKind::DarkCaster => {
                    assert_eq!(d.dust_type, 172);
                    assert!(d.no_gravity);
                }
                NpcKind::DesertDjinn => {
                    assert_eq!(d.dust_type, 27);
                    assert!(d.no_gravity);
                }
                _ => {
                    assert_eq!(d.dust_type, 6);
                    assert!(d.no_gravity);
                }
            }
        }
    }

    /// W4-d52 — the full pending-teleport tick draws BOTH bursts (the
    /// departure at the OLD position, the arrival at the NEW one) on the
    /// caller's stream, around the placement. The mirror replays the
    /// Tim census: 2 x 50 x (the `Next(1,3)` argument + the 27 block).
    /// The teleport search itself draws nothing here (ai[2]/ai[3] are
    /// pre-armed, the `:20855` gate takes the placement directly).
    #[test]
    fn w4_d52_the_teleport_tick_draws_both_bursts() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[2] = 60.0;
        n.ai[3] = 120.0;
        let (bw, bh) = n.size();
        let (bw, bh) = (bw as i32, bh as i32);
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        let mut rng = UnifiedRandom::new(52);
        let mut probe = UnifiedRandom::new(52);
        let mut dust = tr_core::particles::DustPool::new();
        ai_style_8_step(&mut n, &w, &p, &mut rng, &mut dust, &ctx);
        for burst in 0..2 {
            for i in 0..50 {
                probe.next_range(1, 3);
                burst_block(&mut probe, 27, bw, bh);
                assert_eq!(rng.peek(), probe.peek(), "burst {burst} dust {i}");
            }
        }
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 100);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[3], 0.0);
    }

    /// W4-d52 — `NPC.cs:21279-21374`: the ambient tail's per-kind gate
    /// census. 29/45 `Next(5)==0`, 32 `Next(3)!=0`, the else group (24)
    /// `Next(6)==0` — the gate draw runs EVERY tick; 533 draws NOTHING.
    /// The mirror replays 200 ticks per kind from one seed, draining the
    /// pool per tick so the census stays on the empty-pool path.
    #[test]
    fn w4_d52_caster_ambient_dust_gates_match_the_oracle() {
        let w = ground_air();
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        for kind in [
            NpcKind::GoblinSorcerer,
            NpcKind::Tim,
            NpcKind::DarkCaster,
            NpcKind::FireImp,
            NpcKind::DesertDjinn,
        ] {
            let mut n = Npc::new(kind, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
            n.hp = n.max_hp;
            n.vx = 2.0;
            n.vy = -1.0;
            let (bw, bh) = n.size();
            let (bw, bh) = (bw as i32, bh as i32);
            let mut rng = UnifiedRandom::new(52);
            let mut probe = UnifiedRandom::new(52);
            let mut dust = tr_core::particles::DustPool::new();
            let mut passes = 0;
            for tick in 0..200 {
                caster_ambient_dust(&n, &mut rng, &mut dust, &ctx, &w);
                let pass = match kind {
                    NpcKind::GoblinSorcerer | NpcKind::Tim => probe.next_max(5) == 0,
                    NpcKind::DarkCaster => probe.next_max(3) != 0,
                    NpcKind::DesertDjinn => false,
                    _ => probe.next_max(6) == 0,
                };
                if pass {
                    passes += 1;
                    let ty = match kind {
                        NpcKind::GoblinSorcerer | NpcKind::Tim => 27,
                        NpcKind::DarkCaster => 172,
                        _ => 6,
                    };
                    burst_block(&mut probe, ty, bw, bh);
                }
                assert_eq!(rng.peek(), probe.peek(), "{kind:?} ambient stream @{tick}");
                for s in dust.slots_mut() {
                    s.active = false;
                }
            }
            match kind {
                NpcKind::DesertDjinn => {
                    assert_eq!(passes, 0, "533 is light-only");
                    assert_eq!(rng.peek(), UnifiedRandom::new(52).peek(), "533 drew nothing");
                }
                NpcKind::DarkCaster => assert!(passes > 100, "the 2-in-3 gate fired {passes}/200"),
                _ => assert!(passes > 5 && passes < 80, "the gate fired {passes}/200"),
            }
        }
    }

    /// W4-d52 — the ambient post-spawn writes per kind: 29/45 noGravity +
    /// velocity.X * 0.5 + velocity.Y = -2 (`:21285-21287`); 32 noGravity +
    /// X * 0.3, Y * 0.2 then -1 (`:21295-21298`); the else group noGravity
    /// + the C#'s literal x1 writes (`:21369-21371`).
    #[test]
    fn w4_d52_caster_ambient_dust_writes_the_post_fields() {
        let w = ground_air();
        let ctx = tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * TILE_PX, w.height as f32 * TILE_PX),
            max_dust_to_draw: 6000,
        };
        for kind in [NpcKind::Tim, NpcKind::DarkCaster, NpcKind::FireImp] {
            let mut n = Npc::new(kind, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
            n.hp = n.max_hp;
            n.vx = 2.0;
            n.vy = -1.0;
            let mut rng = UnifiedRandom::new(0);
            let mut dust = tr_core::particles::DustPool::new();
            loop {
                caster_ambient_dust(&n, &mut rng, &mut dust, &ctx, &w);
                if dust.slot(0).active {
                    break;
                }
            }
            let d = dust.slot(0);
            assert!(d.no_gravity, "{kind:?} noGravity");
            match kind {
                NpcKind::Tim => {
                    assert_eq!(d.dust_type, 27);
                    assert_eq!(d.velocity.y, -2.0, "the Y overwrite");
                }
                NpcKind::DarkCaster => {
                    assert_eq!(d.dust_type, 172);
                    // The block's Y roll * 0.2 - 1 stays below -0.6.
                    assert!(d.velocity.y < -0.6, "the Y * 0.2 - 1 lift");
                }
                _ => assert_eq!(d.dust_type, 6),
            }
        }
    }
}
