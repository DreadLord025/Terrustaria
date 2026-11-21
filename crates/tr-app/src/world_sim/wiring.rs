//! The wiring tick (plan stage A4): the `Wiring.UpdateMech` cooldown queue,
//! the teleport events' entity half (`Wiring.Teleport`, `Wiring.cs:3120-3178`)
//! and — since M7-WIRE-2 — the device effects' app half: the statue spawns,
//! the trap emissions and the device shots.
//!
//! The engine half (`Wiring::update`, the frame decodes, the `CheckMech`
//! gates and the roll order) lives in `tr-core/src/wiring.rs`.

use glam::Vec2;

use crate::app::App;
use crate::npc::Npc;
use crate::npc::NpcKind;

/// `PressurePlateHelper.UpdatePlayerPosition`
/// (`PressurePlateHelper.cs:60-100`): the **weighted** pressure plates
/// (428) — the ones that stay pressed while the player stands on them.
///
/// The C#'s shape: the old tile list (the tiles the player covered last
/// tick, inset by one pixel) and the new one, the player's box deflated by
/// one pixel and the same box at the last position. A plate is *pressed*
/// when the player's box reaches the plate's top ten pixels and it was not
/// reached before, and *released* when the player left it — each transition
/// fires the plate's switch (`PokeLocation`, `:157-165`).
///
/// The C#'s helper keeps the last position per player; the port takes it as
/// a parameter (the sim captures the player's `prev_x`/`prev_y` before the
/// tick's movement — the same position the C#'s `PlayerLastPosition` holds).
pub(crate) fn update_pressure_plates(
    wiring: &mut tr_core::wiring::Wiring,
    world: &mut tr_core::world::World,
    player_x: f32,
    player_y: f32,
    last_x: f32,
    last_y: f32,
    width: f32,
    height: f32,
) {
    // `Collision.GetTilesIn` over the two boxes, each inset by one pixel
    // (`PressurePlateHelper.cs:62-64`).
    let old_tiles = tr_core::wind::get_tiles_in(
        world,
        (last_x + 1.0, last_y + 1.0),
        (last_x + width - 1.0, last_y + height - 1.0),
    );
    let new_tiles = tr_core::wind::get_tiles_in(
        world,
        (player_x + 1.0, player_y + 1.0),
        (player_x + width - 1.0, player_y + height - 1.0),
    );
    // The player's box, deflated by one pixel on each side (`:65-66`).
    let hit = (player_x + 1.0, player_y + 1.0, width - 2.0, height - 2.0);
    let old_hit = (last_x + 1.0, last_y + 1.0, width - 2.0, height - 2.0);
    // The plate's own rect: the tile's top ten pixels
    // (`pressurePlateBounds`, `PressurePlateHelper.cs:11`/:70-71).
    let plate_rect = |x: i32, y: i32| (x as f32 * 16.0, y as f32 * 16.0 + 6.0, 16.0, 10.0);
    let intersects = |r: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| -> bool {
        // `Rectangle.Intersects` (XNA): the strict overlap test.
        r.0 < b.0 + b.2 && r.0 + r.2 > b.0 && r.1 < b.1 + b.3 && r.1 + r.3 > b.1
    };
    // The release pass (`:68-81`): the plate the player left.
    for &(px, py) in &old_tiles {
        if !world
            .tile(px, py)
            .is_some_and(|t| t.active() && t.kind == 428)
        {
            continue;
        }
        if !intersects(hit, plate_rect(px, py)) && !new_tiles.contains(&(px, py)) {
            wiring.release_plate(world, px, py);
        }
    }
    // The press pass (`:82-95`): the plate the player reached.
    for &(px, py) in &new_tiles {
        if !world
            .tile(px, py)
            .is_some_and(|t| t.active() && t.kind == 428)
        {
            continue;
        }
        if intersects(hit, plate_rect(px, py))
            && (!old_tiles.contains(&(px, py)) || !intersects(old_hit, plate_rect(px, py)))
        {
            wiring.press_plate(world, px, py);
        }
    }
}

/// Fixed 60 Hz wiring stepping: the mech queue's timers/traps plus the
/// teleporter events computed by the last `TripWire` (`Wiring.cs:651-661`).
/// The engine half (`Wiring::update` + the pad pairs) lives in
/// `tr-core/src/wiring.rs`.
pub(crate) fn update_wiring(app: &mut App, dt: f32) {
    // The actuation cascade's object drops (`SquareTileFrame(resetFrame:
    // false)` inside `DeActive`/`ReActive`), resolved through
    // `registry::tile_drop` exactly like the break path's object drops.
    let mut actuation_drops = Vec::new();
    // The trap emissions (`HitWireSingle`'s case 137) and the device shots
    // (cannons/fireworks/explosives/mine/geyser/toilets) drain into the
    // spawn loop after the step — their caps read the projectile pool.
    let mut trap_requests = Vec::new();
    let mut device_shots = Vec::new();
    // THROW_A_PARTY (25) rides after the fixed step: the `98` broadcast needs
    // `&mut App`, which the wiring closure holds across its body.
    let mut party_credits = 0;
    app.wiring_step.tick(dt.min(0.25), |_| {
        app.wiring.update(&mut app.world);
        // `Wiring.Teleport()` per registered pad pair (`Wiring.cs:3120-3178`):
        // every living player/NPC standing in a pad's 48×48 box moves to the
        // other pad. The `teleporting` re-entry flag and the MP section sync
        // are single-player no-ops.
        // `HitWireSingle`'s statue case (105): the engine parses the style,
        // runs `CheckMech(30/600/300/900)` and emits the spawn request; the app
        // runs the `MechSpawn` caps against its entity lists and constructs the
        // ported kinds (`NPC.MechSpawn`, `NPC.cs:7420`; `Item.MechSpawn`,
        // `Item.cs:49385`).
        for spawn in app.wiring.take_statue_spawns() {
            match spawn {
                tr_core::wiring::StatueSpawn::Npc {
                    x,
                    y,
                    npc_id,
                    cap_ids,
                } => {
                    // `NPC.MechSpawn` (`NPC.cs:7420-7506`): the caps over the
                    // app's npc list — 3 within 200 px, 6 within 600 px, 10
                    // global, grouped by the C#'s type families. Every id in
                    // `cap_ids` (the spawn id when empty) must pass.
                    let caps: Vec<u16> = cap_ids.iter().map(|&c| c as u16).collect();
                    if !crate::wiring_stub::npc_mech_spawn(
                        app.npcs.iter().map(|n| (n.x, n.y, n.kind.cs_id())),
                        &caps,
                        npc_id as u16,
                        x as f32,
                        y as f32,
                    ) {
                        continue;
                    }
                    if let Some(mut npc) = statue_npc(npc_id, x, y) {
                        // `Wiring.cs:2471-2477`: `value = 0`, `npcSlots = 0`,
                        // `SpawnedFromStatue`, `CanBeReplacedByOtherNPCs` — the
                        // port models the loot-gating flag; the other three have
                        // no field yet (ANALYSIS note).
                        npc.spawned_from_statue = true;
                        app.npcs.push(npc);
                    }
                }
                tr_core::wiring::StatueSpawn::Item {
                    x,
                    y,
                    item_id,
                    cap_ids,
                } => {
                    // `Item.MechSpawn` (`Item.cs:49385-49406`): the same caps
                    // over the dropped items, rings 300/800 px, one exact type
                    // per cap id (the seasonal ids ride along — F6). The caps
                    // read the PRE-remap ids on the ground exactly as the C#
                    // does (`Wiring.cs:2383` checks 58/1734/1867 before the
                    // `NewItem` call).
                    let caps: Vec<u16> = cap_ids.iter().map(|&c| c as u16).collect();
                    if !crate::wiring_stub::item_mech_spawn(
                        app.items.iter().map(|d| (d.pos.x, d.pos.y, d.kind)),
                        &caps,
                        item_id as u16,
                        x as f32,
                        y as f32,
                    ) {
                        continue;
                    }
                    // S1-P-i52 — `Item.NewItem`'s seasonal remap
                    // (`Item.cs:49668-49714`): the heart statue drops the
                    // Candy Apple/Candy Cane under the seasonal flags, the
                    // star statue the Soul Cake/Sugar Plum. The remap draw
                    // (only the anniversary/both-seasons arms draw) rides
                    // the wiring stream — `Main.rand` at the `HitWireSingle`
                    // site (the port's wiring-RNG stand-in, M7-WIRE-1).
                    let kind = crate::world_sim::items::seasonal_pickup_kind(
                        item_id as u16,
                        app.world
                            .seeds
                            .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY),
                        app.world
                            .micro_seeds
                            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN),
                        app.world
                            .micro_seeds
                            .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS),
                        app.wiring.rand_mut(),
                    );
                    app.items.push(crate::item::ItemDrop {
                        kind,
                        count: 1,
                        shimmer_time: 0.0,
                        shimmered: false,
                        shimmer_pending: false,
                        shimmer_wet: false,
                        prefix: 0,
                        pos: Vec2::new(x as f32, y as f32),
                        vel: Vec2::ZERO,
                        age: 0.0,
                        phase: 0.0,
                        grab_delay_time: 0,
                        grab_delay_player: 255,
                        being_grabbed: false,
                        on_conveyor: false,
                    });
                }
                tr_core::wiring::StatueSpawn::Teleport { x, y, roster } => {
                    // `Wiring.cs:2413-2440`/`:2442-2468`: the collection (first
                    // 50 roster NPCs in index order), one roll from the core
                    // stream when the collection is non-empty, and only the
                    // rolled NPC moves — to `(x - width/2, y - height - 1)` with
                    // the roll's style 14.
                    let mut candidates = Vec::new();
                    for (idx, n) in app.npcs.iter().enumerate() {
                        if n.hp > 0 && roster.contains(&n.kind.cs_id()) {
                            candidates.push(idx);
                            if candidates.len() >= 50 {
                                break;
                            }
                        }
                    }
                    if candidates.is_empty() {
                        continue;
                    }
                    let roll = app.wiring.rand_next(candidates.len() as i32) as usize;
                    let npc = &mut app.npcs[candidates[roll]];
                    let (w, h) = npc.size();
                    npc.x = x as f32 - w * 0.5;
                    npc.y = y as f32 - h - 1.0;
                }
                // The boulder statue's projectile 99 (aiStyle 25) has no ported
                // family — dropped with the ANALYSIS note, like the other
                // not-yet-ported projectile families.
                tr_core::wiring::StatueSpawn::Boulder { .. } => {}
            }
        }
        // The actuation cascade's object drops (`SquareTileFrame(resetFrame:
        // false)` inside `DeActive`/`ReActive`) are drained after the step —
        // spawning goes through `spawn_item_drop(&mut App)`, and passing the
        // whole `app` into the closure would break its disjoint capture of
        // `app.wiring_step`'s receiver borrow.
        actuation_drops.extend(app.wiring.take_actuation_drops());
        // The trap emissions (`HitWireSingle`'s case 137): the spear's
        // proximity cap reads the pool, so the drain splits it from the
        // direct shots.
        trap_requests.extend(app.wiring.take_trap_requests());
        // The device shots (cannons, fireworks, explosives/mine/geyser/toilet).
        device_shots.extend(app.wiring.take_device_shots());
        // The announcement box's texts (`HitWireSingle`'s case 425 →
        // `Main.NewTextMultiline`, `Wiring.cs:1202-1233`): the C# `Color.Pink`
        // chat line. `AnnouncementBoxRange == -1` announces unconditionally;
        // otherwise only a player within the range of the box's first cell
        // centre (`num7 * 16 + 16, num8 * 16 + 16`) hears it.
        for (bx, by, text) in app.wiring.take_announcement_events() {
            let in_range = app.world.announcement_box_range == -1
                || app.player.as_ref().is_some_and(|p| {
                    let (cx, cy) = p.center();
                    let dx = cx - (bx as f32 * 16.0 + 16.0);
                    let dy = cy - (by as f32 * 16.0 + 16.0);
                    (dx * dx + dy * dy).sqrt() <= app.world.announcement_box_range as f32
                });
            if in_range {
                crate::chat::broadcast_death_text(&mut app.chat, &text, [255, 192, 203]);
            }
        }
        // The party centre's toggles (`HitWireSingle`'s case 455 →
        // `BirthdayParty.ToggleManualParty`, `BirthdayParty.cs:202-217`):
        // `ManualParty = !ManualParty` per request, and the up-state edge
        // fires `CheckForAchievement` — THROW_A_PARTY (25) credits only when
        // the party came up (`:157-164`, `:167-173`, S1-E-18f).
        for _ in 0..app.wiring.take_party_toggles() {
            let was_up = crate::world_events::party_is_up(&app.world_events);
            app.world_events.manual_party = !app.world_events.manual_party;
            if !was_up && crate::world_events::party_is_up(&app.world_events) {
                party_credits += 1;
            }
        }
        for (from, to) in app.wiring.take_teleport_events() {
            let ax = (from.0 * 16.0) as i32;
            let ay = (from.1 * 16.0) as i32 - 48;
            let bx = (to.0 * 16.0) as i32;
            let by = (to.1 * 16.0) as i32 - 48;
            let dx = (bx - ax) as f32;
            let dy = (by - ay) as f32;
            // `TeleporterHitboxIntersects` (`Wiring.cs:3181-3189`): the union
            // of the 48×48 box and the entity box fits both spans.
            let hits = |ex: f32, ey: f32, ew: f32, eh: f32| {
                let ux0 = ax.max(ex as i32);
                let uy0 = ay.max(ey as i32);
                let ux1 = (ax + 48).min((ex as f32 + ew) as i32);
                let uy1 = (ay + 48).min((ey as f32 + eh) as i32);
                ux1 >= ux0 && uy1 >= uy0
            };
            if !app.wiring.block_player_teleportation() {
                if let Some(p) = &mut app.player {
                    let px = p.x;
                    let py = p.y;
                    if !p.dead && hits(px, py, crate::player::WIDTH, crate::player::HEIGHT) {
                        // `Player.Teleport`'s camera transition
                        // (`Player.cs:37463`, `Main.cs:62572` — the wiring pad
                        // rides the default style, so the lerp delay is 0).
                        let old_center = glam::Vec2::from(p.center());
                        let mut state = std::mem::take(&mut app.camera_state);
                        crate::world_sim::camera::start_camera_transition_for_player_teleport(
                            &mut state,
                            old_center,
                            0.1,
                            0,
                        );
                        app.camera_state = state;
                        p.teleport(px + dx, py + dy);
                    }
                }
            }
            // `Wiring.cs:3147-3157`: the NPC gates — `lifeMax > 5`, no bosses,
            // `noTileCollide` and the `TeleportationImmune` set (none of the
            // implemented kinds is immune; bosses = King Slime).
            for n in app.npcs.iter_mut() {
                if n.max_hp > 5 && n.hp > 0 && n.kind != NpcKind::KingSlime {
                    let nx = n.x;
                    let ny = n.y;
                    let (w, h) = n.size();
                    if hits(nx, ny, w, h) {
                        n.x = nx + dx;
                        n.y = ny + dy;
                    }
                }
            }
        }
        // S1-E-21i — the timer expiries' Mech-28 clicks (`UpdateMech`'s
        // case-144 `HitSwitch`, `Wiring.cs:2159`'s port; `WorldGen.cs:71509`
        // calls `Wiring.UpdateMech` from `UpdateWorld`): the wiring tick
        // rides the `UpdateWorldGen` swap (`Main.cs:17802-17805` →
        // `UpdateWorld_WorldGenAndInvasion`), so the pitch draws use that
        // stream — drained at the step tail like the statue spawns.
        for (px, py) in app.wiring.take_switch_clicks() {
            app.audio.play_switch_click(
                px as f32,
                py as f32,
                app.rngs.stream(crate::app::RngKey::UpdateWorldGen),
            );
        }
    });
    for _ in 0..party_credits {
        let _ = app.notify_progression_net(25);
    }
    // The actuation drops spawn after the fixed step (see the drain comment
    // above); the trap requests and the device shots spawn right after (their
    // caps read the pool).
    for drop in actuation_drops {
        if let Some(id) = crate::registry::tile_drop(drop.tile_kind, 0) {
            // The spawn joins the wiring block's own seeded stream (the
            // S1-L-6c contract; the C# truth is `UpdateWorldGen` — ANALYSIS).
            let rng = app.wiring.rand_mut();
            crate::world_sim::items::spawn_item_drop(&mut app.items, rng, id, 1, drop.x, drop.y);
        }
    }
    // The trap emissions spawn after the fixed step (`NewProjectile` with
    // the knockback 2f, `Wiring.cs:1990-1992`). The trap darts, the spiky
    // ball 185, the spear 186 (aiStyle 14) and the flame jet 187 (aiStyle
    // 38 — the stationary emitter of the 188 children) spawn through the
    // pool. The jet is stock friendly/hostile FALSE (`Projectile.cs
    // :2741-2751`) — the spawn site flips the pool's friendly-true default
    // off; its damage rides the 188 children (the W4-d4 drain lift, keep-one
    // union with the W4-g43 row). The ceiling −7 arm emits one identical
    // `Spear` request twice: the C# evaluates its cap ONCE before both
    // spawns, so identical requests share one cap decision (and therefore
    // one pool scan).
    let mut decided: Vec<tr_core::wiring::TrapRequest> = Vec::new();
    let mut blocked: Vec<tr_core::wiring::TrapRequest> = Vec::new();
    for req in trap_requests {
        let allow = match req {
            tr_core::wiring::TrapRequest::Shot { .. } => true,
            tr_core::wiring::TrapRequest::Spear { .. } => {
                if blocked.contains(&req) {
                    false
                } else if decided.contains(&req) {
                    true
                } else {
                    let allow = spear_cap_allows(app, &req);
                    if allow {
                        decided.push(req);
                    } else {
                        blocked.push(req);
                    }
                    allow
                }
            }
        };
        if !allow {
            continue;
        }
        match req {
            tr_core::wiring::TrapRequest::Shot {
                x,
                y,
                vx,
                vy,
                kind,
                damage,
            } => {
                let slot = spawn_device_kind(app, kind, x, y, vx, vy, damage, 2.0, 0.0, 0.0);
                // The flame jet 187 is stock `friendly = false`
                // (`Projectile.cs:2741-2751` sets neither flag); the pool's
                // spawn defaults friendly true, so the site flips it off
                // (the jet damages nothing itself — the 188 children do).
                // The W4-d4 drain lift, keep-one union.
                if kind == crate::projectile::FLAMETHROWER_TRAP {
                    if let Some(s) = slot {
                        app.projectiles.slots_mut()[s].friendly = false;
                    }
                }
            }
            // The `count_kind` is the cap's counting type (F1: 0 on the
            // floor), NOT the spawn kind — every arm of the family fires the
            // spiky ball (185).
            tr_core::wiring::TrapRequest::Spear {
                x, y, vx, vy, damage, ..
            } => {
                let _ = spawn_device_kind(
                    app,
                    crate::projectile::TRAP_SPIKY_BALL,
                    x,
                    y,
                    vx,
                    vy,
                    damage,
                    2.0,
                    0.0,
                    0.0,
                );
            }
        }
    }
    // The device shots spawn after the fixed step (`NewProjectile`,
    // `Wiring.cs:1413` snowball / `WorldGen.cs:62174` rocket / `:1522` cannon /
    // `:1499` mine / `:3113` geyser / `:1504` toilet). The families whose
    // aiStyle is not ported (the rockets 167+/415-418, the fountain 419-422,
    // the cannonball 162, the bunny 281, the snowball 166, the geyser 654, the
    // toilet 733) drop here with the ANALYSIS note; the explosive blast
    // families (108/164/178/601) spawn through the pool.
    for shot in device_shots {
        // `Wiring.cs:1526-1539`: the fountain's dedupe — an active
        // `aiStyle == 73` projectile with the same `ai[0]/ai[1]` anchor skips
        // the shot. The scan reads the pool, so it lives here (decision D1:
        // the roll already happened in the engine).
        if (419..=422).contains(&shot.kind) && fountain_anchored(app, shot.ai0, shot.ai1) {
            continue;
        }
        // `WorldGen.cs:50683-50685`: `BunnyCannonCanFire` refuses only the
        // spawn — the cooldowns were already assigned in the engine.
        if shot.kind == crate::projectile::CANNON_BUNNY && !bunny_cannon_can_fire(app) {
            continue;
        }
        let _ = spawn_device_kind(
            app,
            shot.kind,
            shot.x,
            shot.y,
            shot.vx,
            shot.vy,
            shot.damage,
            shot.knock_back,
            shot.ai0,
            shot.ai1,
        );
    }
}

/// The C#'s `NPC.NewNPC` (`NPC.cs:80750-80780`): the entity is constructed at
/// the request's anchor as its **bottom-center** (`nPC.Bottom = (X, Y)`), with
/// `SetDefaults` standing in for the port's `Npc::new`. The statue-spawnable
/// kinds map through their C# ids; the kinds without a port row drop (the C#
/// would spawn them — the roster gap is the ANALYSIS subset note).
fn statue_npc(npc_id: i16, x: i32, y: i32) -> Option<Npc> {
    let kind = match npc_id {
        1 => NpcKind::Slime,
        2 => NpcKind::DemonEye,
        6 => NpcKind::EaterOfSouls,
        21 => NpcKind::Zombie,
        24 => NpcKind::FireImp,
        42 => NpcKind::Hornet,
        49 => NpcKind::CaveBat,
        55 => NpcKind::Goldfish,
        58 => NpcKind::Piranha,
        65 => NpcKind::Shark,
        _ => return None,
    };
    let (w, h) = kind.size();
    Some(Npc::new(kind, x as f32 - w * 0.5, y as f32 - h, 1, 0.0))
}

/// The spiky ball's proximity cap (`Wiring.cs:1814-1822`/`:1945-1953`): the
/// 200 base count decrements by the distance rings measured from
/// `(i*16+8, j*18+8)` — the 18 is verbatim — against the request's
/// `count_kind` projectiles. 0 on the floor (F1: `ProjectileID.None`, which no
/// live slot can hold — the cap is a no-op by construction), 185 on the
/// ceiling.
fn spear_cap_allows(app: &App, req: &tr_core::wiring::TrapRequest) -> bool {
    let (x, y, count_kind) = match *req {
        tr_core::wiring::TrapRequest::Spear {
            x,
            y,
            count_kind,
            ..
        } => (x, y, count_kind),
        tr_core::wiring::TrapRequest::Shot { .. } => return true,
    };
    let mut count = 200;
    for p in app.projectiles.slots().iter().filter(|p| p.active) {
        if p.kind != count_kind {
            continue;
        }
        let dx = (x - p.pos.x).abs();
        let dy = (y - p.pos.y).abs();
        let dist = (dx * dx + dy * dy).sqrt();
        count -= if dist < 50.0 {
            50
        } else if dist < 100.0 {
            15
        } else if dist < 200.0 {
            10
        } else if dist < 300.0 {
            8
        } else if dist < 400.0 {
            6
        } else if dist < 500.0 {
            5
        } else if dist < 700.0 {
            4
        } else if dist < 900.0 {
            3
        } else if dist < 1200.0 {
            2
        } else {
            1
        };
    }
    count > 0
}

/// `WorldGen.BunnyCannonCanFire` (`WorldGen.cs:50700-50747`): at least one free
/// NPC slot (of the first 100) survives the live type-281 projectiles, and the
/// bunnies never reach 4. The port's npc list holds only live NPCs, so the
/// free-slot count is `100 - live` bounded by the list.
fn bunny_cannon_can_fire(app: &App) -> bool {
    let live = app.npcs.len().min(100);
    let mut free = (100 - live) as i32;
    let mut bunnies = 0;
    for p in app.projectiles.slots().iter().filter(|p| p.active) {
        if p.kind == crate::projectile::CANNON_BUNNY {
            bunnies += 1;
            if bunnies >= 4 {
                return false;
            }
            free -= 1;
            if free <= 0 {
                return false;
            }
        }
    }
    free >= 1
}

/// The fountain's dedupe (`Wiring.cs:1531-1538`): an active projectile of the
/// fountain family anchored at `(ai0, ai1)` blocks a new one.
fn fountain_anchored(app: &App, ai0: f32, ai1: f32) -> bool {
    app.projectiles
        .slots()
        .iter()
        .filter(|p| p.active)
        .any(|p| (419..=422).contains(&p.kind) && p.ai[0] == ai0 && p.ai[1] == ai1)
}

/// The shared spawn: the port's pool when the family has `ProjData`, a silent
/// drop otherwise (the ANALYSIS note lists the missing aiStyle families).
/// Returns the spawned slot so the caller can apply the C# `SetDefaults`
/// flags the pool's spawn defaults overwrite (the flame jet's
/// friendly-false flip — the W4-d4 drain lift, keep-one union).
fn spawn_device_kind(
    app: &mut App,
    kind: u16,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    damage: i32,
    knock_back: f32,
    ai0: f32,
    ai1: f32,
) -> Option<usize> {
    if crate::projectile::proj_data(kind).is_some() {
        app.projectiles
            .spawn_with_data_ai(kind, x, y, vx, vy, damage, knock_back, ai0, ai1)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn world() -> tr_core::world::World {
        tr_core::world::World::new("t".into(), 64, 64)
    }

    /// The weighted pressure plate (428) under the player's feet: the
    /// player stands on it, then leaves.
    #[test]
    fn the_weighted_plate_presses_and_releases() {
        let mut w = world();
        let i = w.index(10, 19);
        w.tiles[i].kind = 428; // the WeightedPressurePlate
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut wiring = tr_core::wiring::Wiring::new();
        // The player's box [168, 188] x [278, 320]; the plate's own rect is
        // its tile's lower ten pixels (`PressurePlateHelper.cs:11`/:70-71)
        // — 310..320 — which the player's deflated box (279..319) reaches.
        let px = 10.0 * 16.0 + 8.0;
        let py = 20.0 * 16.0 - 42.0;
        // The player arrives from above: the old position was off the plate.
        update_pressure_plates(&mut wiring, &mut w, px, py, px, py - 64.0, 20.0, 42.0);
        assert!(wiring.plate_is_pressed(10, 19), "the plate is held");
        // Standing still changes nothing.
        update_pressure_plates(&mut wiring, &mut w, px, py, px, py, 20.0, 42.0);
        assert!(wiring.plate_is_pressed(10, 19), "still held");
        // The player walks away: the plate releases.
        update_pressure_plates(&mut wiring, &mut w, px, py - 128.0, px, py, 20.0, 42.0);
        assert!(!wiring.plate_is_pressed(10, 19), "the plate released");
    }

    /// The standard device rig: the device at (10, 20) with the given frame,
    /// the wired switch at (8, 20) and the red wire line between them.
    fn world_with_device(kind: u16, frame_x: i16, frame_y: i16) -> tr_core::world::World {
        let mut w = world();
        let i = w.index(10, 20);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let si = w.index(8, 20);
        w.tiles[si] = Tile {
            kind: 135,
            ..Default::default()
        };
        w.tiles[si].flags |= FLAG_ACTIVE;
        for x in [8, 9, 10] {
            w.wires.set(x, 20, tr_core::wiring::WireColor::Red, true);
        }
        w
    }

    fn app_with(kind: u16, frame_x: i16, frame_y: i16) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = world_with_device(kind, frame_x, frame_y);
        app
    }

    /// One trip plus one wiring tick: the trip accumulates the requests, the
    /// tick drains them (the player's use path runs the trip).
    fn trip_and_tick(app: &mut App) {
        app.wiring.trip_wire(&mut app.world, 8, 20, 1, 1);
        update_wiring(app, 1.0 / 60.0);
    }

    fn npc_of(kind: crate::npc::NpcKind, x: f32, y: f32) -> crate::npc::Npc {
        let mut npc = crate::npc::Npc::new(kind, x, y, 1, 0.0);
        npc.hp = npc.max_hp;
        npc
    }

    /// S1-E-18f — `BirthdayParty.ToggleManualParty`'s up-state edge fires
    /// `CheckForAchievement` (`BirthdayParty.cs:157-164`, `:167-173`), which
    /// credits THROW_A_PARTY (25); the down edge credits nothing.
    #[test]
    fn s1_e_18f_the_party_centre_fires_throw_a_party() {
        let mut app = app_with(455, 0, 0);
        trip_and_tick(&mut app);
        assert!(app.world_events.manual_party, "the toggle raised the party");
        assert!(
            progression(&app, 25),
            "a party coming up credits THROW_A_PARTY"
        );
        // The mirror: a party already up, a trip takes it down — `PartyIsUp`
        // falls, so `CheckForAchievement`'s `if (PartyIsUp)` never fires.
        let mut down = app_with(455, 0, 0);
        down.world_events.manual_party = true;
        trip_and_tick(&mut down);
        assert!(!down.world_events.manual_party, "the toggle lowered it");
        assert!(
            !progression(&down, 25),
            "bringing the party down credits nothing"
        );
    }

    /// Whether the `PROGRESSION_EVENT_<id>` condition (`AchievementCondition
    /// .Name`, the save key) is completed on any row.
    fn progression(app: &App, id: u32) -> bool {
        let key = format!("PROGRESSION_EVENT_{id}");
        app.achievements
            .list
            .iter()
            .flat_map(|a| a.conditions.iter())
            .any(|c| c.name == key && c.completed)
    }

    #[test]
    fn dw_4_1_the_new_data_rows_and_the_ai_spawn() {
        // DW-4.1: the device families whose aiStyle is ported carry their
        // verbatim `SetDefaults` slice, and `spawn_with_data_ai` writes the
        // C#'s `ai0/ai1` arguments (the fireworks fountain's anchor).
        let mine = crate::projectile::proj_data(crate::projectile::LANDMINE_BLAST).unwrap();
        assert_eq!((mine.w, mine.h), (128.0, 128.0));
        assert_eq!(mine.ai_style, 16);
        assert_eq!(mine.time_left, 2);
        let blast = crate::projectile::proj_data(crate::projectile::EXPLOSIVES_BLAST).unwrap();
        assert_eq!((blast.w, blast.h), (260.0, 260.0));
        assert!(blast.trap);
        assert!(!blast.tile_collide);
        let bomb = crate::projectile::proj_data(crate::projectile::CANNON_BOMB).unwrap();
        assert_eq!(bomb.time_left, 2);
        let explosive = crate::projectile::proj_data(crate::projectile::CANNON_EXPLOSIVE).unwrap();
        assert_eq!(explosive.extra_updates, 30);
        let venom = crate::projectile::proj_data(crate::projectile::TRAP_VENOM_DART).unwrap();
        assert_eq!((venom.w, venom.h), (10.0, 10.0));
        assert_eq!(venom.ai_style, 1);
        // The renamed trap family keeps its verbatim values.
        assert_eq!(crate::projectile::TRAP_SPEAR, 186);
        assert_eq!(crate::projectile::TRAP_SPIKY_BALL, 185);
        assert_eq!(crate::projectile::TRAP_VENOM_DART, 980);
        let mut pool = crate::projectile::ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai(
                crate::projectile::EXPLOSIVES_BLAST,
                1.0,
                2.0,
                0.0,
                0.0,
                500,
                10.0,
                7.0,
                9.0,
            )
            .expect("a free slot");
        assert_eq!(pool.slots()[slot].ai[0], 7.0);
        assert_eq!(pool.slots()[slot].ai[1], 9.0);
        assert_eq!(pool.slots()[slot].damage, 500);
    }

    #[test]
    fn dw_4_3_the_slime_statue_spawns_at_the_verbatim_bottom_center() {
        // `Wiring.cs:2270` (`num140`, `num141` — the body's bottom edge) and
        // `NPC.NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80770`): the ported kind
        // lands with its top-left at the anchor minus half its box.
        let mut app = app_with(105, 4 * 36, 0); // style 4 → the Blue Slime, y − 12
        trip_and_tick(&mut app);
        assert_eq!(app.npcs.len(), 1, "the slime spawns");
        let npc = &app.npcs[0];
        let (w, h) = npc.kind.size();
        let anchor_x = 10.0 * 16.0 + 16.0;
        let anchor_y = (20.0 + 3.0) * 16.0 - 12.0;
        assert!(
            (npc.x - (anchor_x - w * 0.5)).abs() < 0.01,
            "bottom-center x: {} vs {}",
            npc.x,
            anchor_x - w * 0.5
        );
        assert!(
            (npc.y - (anchor_y - h)).abs() < 0.01,
            "bottom-center y: {} vs {}",
            npc.y,
            anchor_y - h
        );
        assert!(npc.spawned_from_statue, "Wiring.cs:2475");
    }

    #[test]
    fn dw_4_3_the_unrostered_statue_kinds_drop() {
        // The roster gap: a style whose C# id has no ported kind emits its
        // request and the app drops it (the ANALYSIS subset note) — no NPC,
        // no panic.
        let mut app = app_with(105, 66 * 36, 0); // style 66 → the Wraith (82)
        trip_and_tick(&mut app);
        assert!(app.npcs.is_empty(), "the unported kind drops");
    }

    #[test]
    fn dw_4_3_the_item_caps_filter_by_type() {
        // `Item.MechSpawn` (`Item.cs:49385-49406`) counts only the requested
        // type: ten unrelated items must not block the heart statue.
        let mut app = app_with(105, 37 * 36, 0); // the heart statue (item 58)
        for i in 0..10 {
            app.items.push(crate::item::ItemDrop {
                kind: 999 + i,
                count: 1,
                shimmer_time: 0.0,
                shimmered: false,
                shimmer_pending: false,
                shimmer_wet: false,
                prefix: 0,
                pos: Vec2::new(160.0, 320.0),
                vel: Vec2::ZERO,
                age: 0.0,
                phase: 0.0,
                grab_delay_time: 0,
                grab_delay_player: 255,
                being_grabbed: false,
                on_conveyor: false,
            });
        }
        trip_and_tick(&mut app);
        assert_eq!(app.items.len(), 11, "the heart drops despite the noise");
        let heart = app.items.last().unwrap();
        assert_eq!(heart.kind, 58, "the C# ItemID.Heart");
        let anchor_y = (20.0 + 3.0) * 16.0 - 16.0;
        assert!(
            (heart.pos.y - anchor_y).abs() < 0.01,
            "the item anchor: {} vs {}",
            heart.pos.y,
            anchor_y
        );
        // The 600-tick cooldown holds the second trip.
        trip_and_tick(&mut app);
        assert_eq!(app.items.len(), 11, "the 600-tick cooldown holds");
    }

    // ---- S1-P-i52 — the statue drops' seasonal remap (written, NOT run) ----

    /// `Item.NewItem`'s remap at the statue drain (`Item.cs:49668-49714`
    /// via `Wiring.cs:2385`/`:2404`): the endless-Halloween world drops the
    /// Candy Apple 1734 / Soul Cake 1735, the endless-Christmas world the
    /// Candy Cane 1867 / Sugar Plum 1868 — deterministically, no wiring
    /// draw. The classic world keeps 58/184 (the dw_4_3 pin).
    #[test]
    fn s1_p_i52_the_statue_drops_remap_under_the_seasonal_seeds() {
        for (statue_style, halloween_kind, xmas_kind) in
            [(37i16, 1734u16, 1867u16), (2, 1735, 1868)]
        {
            // Halloween alone.
            let mut app = app_with(105, statue_style * 36, 0);
            app.world
                .micro_seeds
                .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN);
            let peek = app.wiring.rand_mut().peek();
            trip_and_tick(&mut app);
            assert_eq!(app.items.len(), 1);
            assert_eq!(app.items[0].kind, halloween_kind, "style {statue_style} halloween");
            assert_eq!(app.wiring.rand_mut().peek(), peek, "the deterministic remap draws nothing");
            // Christmas alone.
            let mut app = app_with(105, statue_style * 36, 0);
            app.world
                .micro_seeds
                .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS);
            let peek = app.wiring.rand_mut().peek();
            trip_and_tick(&mut app);
            assert_eq!(app.items[0].kind, xmas_kind, "style {statue_style} christmas");
            assert_eq!(app.wiring.rand_mut().peek(), peek);
        }
    }

    /// Both seasons: ONE `Next(2)` on the wiring stream per statue drop
    /// (`Item.cs:49683`/`:49687` — `NextFromList`, `Utils.cs:1213-1216`),
    /// consumed AFTER the caps passed. The anniversary seed wins with a
    /// `Next(3)` (`:49672`/`:49676`).
    #[test]
    fn s1_p_i52_the_both_seasons_statue_drop_draws_once_on_the_wiring_stream() {
        use tr_core::unified_random::UnifiedRandom;
        let mut app = app_with(105, 37 * 36, 0);
        app.world
            .micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN);
        app.world
            .micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS);
        app.wiring.seed_rand(11);
        let mut twin = UnifiedRandom::new(11);
        let want = [1734u16, 1867][twin.next_max(2) as usize];
        trip_and_tick(&mut app);
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, want, "the Next(2) pick");
        assert_eq!(app.wiring.rand_mut().peek(), twin.peek(), "exactly one draw");
        // The anniversary world: Next(3), list {1734, 1867, 58}.
        let mut app = app_with(105, 37 * 36, 0);
        app.world
            .seeds
            .insert(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
        app.wiring.seed_rand(11);
        let mut twin = UnifiedRandom::new(11);
        let want = [1734u16, 1867, 58][twin.next_max(3) as usize];
        trip_and_tick(&mut app);
        assert_eq!(app.items[0].kind, want, "the anniversary Next(3) pick");
        assert_eq!(app.wiring.rand_mut().peek(), twin.peek());
    }

    /// A CAP-blocked statue spawn consumes NO remap draw — the C# evaluates
    /// `Item.MechSpawn` before `Item.NewItem` (`Wiring.cs:2383`'s `&&`
    /// chain), so the seasonal roll never runs when the caps refuse.
    #[test]
    fn s1_p_i52_the_capped_statue_spawn_draws_no_remap_roll() {
        let mut app = app_with(105, 37 * 36, 0);
        app.world
            .micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN);
        app.world
            .micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS);
        // Ten hearts on the ground at the anchor: the global cap refuses.
        for _ in 0..10 {
            app.items.push(crate::item::ItemDrop::new(
                58, 1, 0, 160.0, 320.0, 0.0, 0.0, 0.0,
            ));
        }
        let peek = app.wiring.rand_mut().peek();
        trip_and_tick(&mut app);
        assert_eq!(app.items.len(), 10, "the cap refused the spawn");
        assert_eq!(app.wiring.rand_mut().peek(), peek, "no remap draw on a refusal");
    }

    #[test]
    fn dw_4_4_the_teleport_statue_moves_only_the_rolled_npc() {
        // `Wiring.cs:2442-2468`: the roster collection (≤50, index order), the
        // single roll from the core stream, and the move of the rolled NPC
        // only — to `(x - w/2, y - h - 1)`.
        // Roster B (`Wiring.cs:2442-2468`) is the town NPCs — the Nurse (18)
        // and the Dryad (20) are ported.
        let mut app = app_with(105, 41 * 36, 0);
        app.npcs.push(npc_of(crate::npc::NpcKind::Nurse, 100.0, 100.0));
        app.npcs.push(npc_of(crate::npc::NpcKind::Dryad, 200.0, 200.0));
        // A dead roster NPC never collects.
        let mut dead = npc_of(crate::npc::NpcKind::Nurse, 300.0, 300.0);
        dead.hp = 0;
        app.npcs.push(dead);
        // The roll from the same seeded stream, computed independently.
        let roll = {
            let mut probe = tr_core::wiring::Wiring::new();
            probe.seed_rand(5);
            probe.rand_next(2)
        };
        app.wiring.seed_rand(5);
        trip_and_tick(&mut app);
        let picked = roll as usize;
        let other = 1 - picked;
        let other_original = if other == 0 { 100.0 } else { 200.0 };
        assert_eq!(
            app.npcs[other].x, other_original,
            "the unpicked NPC stays put"
        );
        let anchor_x = 10.0 * 16.0 + 16.0;
        let anchor_y = (20.0 + 3.0) * 16.0;
        let moved = &app.npcs[picked];
        let (w, h) = moved.kind.size();
        assert!(
            (moved.x - (anchor_x - w * 0.5)).abs() < 0.01,
            "the moved NPC's x: {} vs {}",
            moved.x,
            anchor_x - w * 0.5
        );
        assert!(
            (moved.y - (anchor_y - h - 1.0)).abs() < 0.01,
            "the moved NPC's y: {} vs {}",
            moved.y,
            anchor_y - h - 1.0
        );
    }

    #[test]
    fn dw_4_4_an_empty_roster_rolls_and_moves_nothing() {
        // The C# rolls only when the collection is non-empty — an empty
        // roster (roster A holds no ported kind yet) is a no-op.
        let mut app = app_with(105, 40 * 36, 0);
        app.npcs.push(npc_of(crate::npc::NpcKind::Slime, 100.0, 100.0));
        trip_and_tick(&mut app);
        assert_eq!((app.npcs[0].x, app.npcs[0].y), (100.0, 100.0));
    }

    #[test]
    fn dw_4_2_the_ceiling_spiky_ball_cap_counts_kind_185() {
        // `Wiring.cs:1945-1953`: the ceiling arm's cap counts live SpikyBallTrap
        // (185) projectiles within the rings; four near ones exhaust the 200
        // base and block both identical requests.
        let mut app = app_with(137, 18, -7 * 18);
        for k in 0..4 {
            app.projectiles
                .spawn_with_data(
                    crate::projectile::TRAP_SPIKY_BALL,
                    168.0,
                    296.0 + k as f32,
                    0.0,
                    0.0,
                    40,
                    2.0,
                )
                .expect("a free slot");
        }
        trip_and_tick(&mut app);
        let live = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == crate::projectile::TRAP_SPIKY_BALL)
            .count();
        assert_eq!(live, 4, "both ceiling requests are blocked by the cap");
    }

    #[test]
    fn dw_4_2_the_ceiling_spiky_ball_fires_twice_when_clear() {
        // F2: with a clear pool the ceiling arm's two identical requests both
        // spawn (the shared cap decision) — five live 185s after the drain.
        let mut app = app_with(137, 18, -7 * 18);
        trip_and_tick(&mut app);
        let live = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == crate::projectile::TRAP_SPIKY_BALL)
            .count();
        assert_eq!(live, 2, "one cap decision, two spawns");
    }

    /// The temple style-2 flame trap (`Wiring.cs:1772-1801`'s variant 2,
    /// frameY 36): the jet 187 now SPAWNS through the drain (the W4-g43
    /// row + the W4-d4 machine, keep-one union) with the C#'s stock
    /// friendly/hostile FALSE — the spawn site flips the pool's
    /// friendly-true default off (`Projectile.cs:2741-2751` sets neither
    /// flag; the jet's damage rides its 188 children). frameX 18 → +1 X:
    /// the jet spawns at `(10*16+8+10, 20*16+8)` = (178, 328) with
    /// velocity (5, 0), damage 40, knockback 2. WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_g43_the_flame_trap_spawns_the_jet_friendly_false() {
        let mut app = app_with(137, 18, 2 * 18);
        trip_and_tick(&mut app);
        let jet = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == crate::projectile::FLAMETHROWER_TRAP)
            .expect("the 187 jet spawns (the style-2 trap fires)");
        assert_eq!(jet.pos, Vec2::new(178.0, 328.0), "the +10 px X offset");
        assert_eq!(jet.vel, Vec2::new(5.0, 0.0), "the flame speed 5");
        assert_eq!(jet.damage, 40);
        assert_eq!(jet.knock_back, 2.0, "the shared trap knockback 2f");
        assert!(!jet.friendly, "the spawn site flips the friendly default off");
        assert_eq!(jet.time_left, 60, "the row's 60-tick life");
    }

    #[test]
    fn dw_4_2_the_floor_spiky_ball_cap_never_blocks() {
        // F1: the floor arm counts `ProjectileID.None` (0) — no live slot can
        // hold kind 0, so the cap is a no-op and the shot lands.
        let mut app = app_with(137, 18, 3 * 18);
        trip_and_tick(&mut app);
        let live = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == crate::projectile::TRAP_SPIKY_BALL)
            .count();
        assert_eq!(live, 1, "the floor cap never blocks");
    }

    #[test]
    fn dw_4_2_the_bunny_gate_blocks_only_the_spawn() {
        // `WorldGen.cs:50683-50685`: `BunnyCannonCanFire` refuses only the
        // spawn — and with no ported bunny projectile family the request
        // drops anyway; the gate itself must not panic or spawn.
        let mut app = app_with(209, 72 + 18, 54); // the bunny cannon's firing cell
        trip_and_tick(&mut app);
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            0,
            "the bunny projectile has no ported family — dropped"
        );
    }

    #[test]
    fn dw_4_2_the_fountain_dedupe_blocks_the_anchor() {
        // `Wiring.cs:1526-1539`: the dedupe reads the pool's active fountains
        // for the same `ai[0]/ai[1]` anchor. The fountains (419-422) have no
        // ported data, so today the request drops at the spawn anyway; this
        // pins the drain's current behaviour and the predicate on an empty
        // pool — the anchored-block path activates with the aiStyle-73 family.
        let mut app = app_with(338, 0, 0);
        assert!(!fountain_anchored(&app, 10.0, 20.0), "an empty pool anchors nothing");
        trip_and_tick(&mut app);
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            0,
            "the fountain family is unported — dropped"
        );
    }

    /// The dedupe predicate over a bare pool (the helper reads `App`).
    fn fountain_anchored_pool(pool: &crate::projectile::ProjectilePool, x: f32, y: f32) -> bool {
        pool.slots()
            .iter()
            .filter(|p| p.active)
            .any(|p| (419..=422).contains(&p.kind) && p.ai[0] == x && p.ai[1] == y)
    }
}
