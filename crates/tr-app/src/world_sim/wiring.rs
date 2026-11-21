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
    // The D8-residual device arms (W4-e20): the switch-track flips (314), the
    // hopper pulls (21/467) and the extractinator rolls (219/642).
    let mut switch_track_flips = Vec::new();
    let mut hopper_requests = Vec::new();
    let mut extractinator_requests = Vec::new();
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
        // The D8-residual device arms (W4-e20): the switch-track flips (314),
        // the hopper pulls (21/467) and the extractinator rolls (219/642). The
        // engine ran the case-314 CheckMech(5); the hopper/extractinator
        // anchors + CheckMech(60) + chest gates resolve app-side below.
        switch_track_flips.extend(app.wiring.take_switch_track_flips());
        hopper_requests.extend(app.wiring.take_hopper_requests());
        extractinator_requests.extend(app.wiring.take_extractinator_requests());
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
    // the knockback 2f, `Wiring.cs:1990-1992`). The flame jet 187 (aiStyle 38)
    // requests drop until M4-PROJ ports that AI; the trap darts, the spiky
    // ball 185 and the spear 186 (aiStyle 14) spawn through the pool. The
    // ceiling −7 arm emits one identical `Spear` request twice: the C#
    // evaluates its cap ONCE before both spawns, so identical requests share
    // one cap decision (and therefore one pool scan).
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
            } => spawn_device_kind(app, kind, x, y, vx, vy, damage, 2.0, 0.0, 0.0),
            // The `count_kind` is the cap's counting type (F1: 0 on the
            // floor), NOT the spawn kind — every arm of the family fires the
            // spiky ball (185).
            tr_core::wiring::TrapRequest::Spear {
                x, y, vx, vy, damage, ..
            } => spawn_device_kind(
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
            ),
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
        spawn_device_kind(
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
    // The switch-track flips (`HitWireSingle` case 314 → `Minecart.FlipSwitchTrack`,
    // `Wiring.cs:1744-1748`/`Minecart.cs:1296-1319`): the engine's CheckMech(5)
    // already gated the request; the flip reads the track-frame table.
    for (i, j) in switch_track_flips {
        crate::minecart::flip_switch_track(&mut app.world, i, j);
    }
    // The hopper pulls (`HitWireSingle` cases 21/467 → `Wiring.Hopper`,
    // `Wiring.cs:2638-2678`) and the extractinator rolls (cases 219/642 →
    // `Wiring.Extractinator`, `:2554-2584`): the anchor, CheckMech(60) and the
    // chest gates resolve here (the chest store + item pool live app-side).
    for (i, j, kind) in hopper_requests {
        hopper_pull(app, i, j, kind);
    }
    for (i, j, kind) in extractinator_requests {
        extractinator_roll(app, i, j, kind);
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
/// 200 base count decrements by the distance rings measured from the
/// request's cap anchor `(i*16+8, j*18+8)` — the 18 is verbatim (F3) —
/// against each matching projectile's **Center** (`Main.projectile[..].Center`,
/// `:1819`/`:1950`), counting the request's `count_kind` projectiles. 0 on
/// the floor (F1: `ProjectileID.None`, which no live slot can hold — the cap
/// is a no-op by construction), 185 on the ceiling.
fn spear_cap_allows(app: &App, req: &tr_core::wiring::TrapRequest) -> bool {
    let (cap_x, cap_y, count_kind) = match *req {
        tr_core::wiring::TrapRequest::Spear {
            cap_x,
            cap_y,
            count_kind,
            ..
        } => (cap_x, cap_y, count_kind),
        tr_core::wiring::TrapRequest::Shot { .. } => return true,
    };
    let mut count = 200;
    for p in app.projectiles.slots().iter().filter(|p| p.active) {
        if p.kind != count_kind {
            continue;
        }
        let (cx, cy) = p.center();
        let dx = cap_x - cx;
        let dy = cap_y - cy;
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

/// `ItemID.Sets.ItemsThatShouldNotBeInInventory` (`ID/ItemID.cs:345`) — the
/// hopper's grab gate (`Wiring.cs:2612`/`:2667`). Same set as `chest.rs`'s
/// `BANNED_IN_INVENTORY` (the save-side air rule).
const HOPPER_BANNED: &[u16] = &[
    58, 184, 1734, 1735, 1867, 1868, 3453, 3454, 3455, 4143, 5013, 6135, 6136,
];

/// `Utils.CoinsCount` (`Utils.cs:1019-1046`): the copper-equivalent value of
/// the coin slots (71×1, 72×100, 73×10000, 74×1000000). No ignore slots here.
fn coins_count(slots: &[crate::item::ItemSlot]) -> i64 {
    let mut total = 0i64;
    for s in slots {
        total += match s.kind {
            71 => i64::from(s.count),
            72 => i64::from(s.count) * 100,
            73 => i64::from(s.count) * 10_000,
            74 => i64::from(s.count) * 1_000_000,
            _ => 0,
        };
    }
    total
}

/// `Utils.CoinsSplit` (`Utils.cs:1048-1060`): the four-denomination split
/// `[copper, silver, gold, platinum]` of a copper-equivalent count.
fn coins_split(count: i64) -> [i64; 4] {
    let mut out = [0i64; 4];
    let mut acc = 0i64;
    let mut denom = 1_000_000i64;
    for idx in (0..4).rev() {
        out[idx] = (count - acc) / denom;
        acc += out[idx] * denom;
        denom /= 100;
    }
    out
}

/// `Wiring.TryMoveCoinsInChest` (`Wiring.cs:2703-2757`): the hopper's coin arm.
/// The chest's coins plus the drop's value (`value / 5 * stack`) are re-split
/// into denominations and laid into the first air/coin slots (scanned high→low
/// so platinum lands first); a slot shortfall returns false (no mutation). On
/// success every old coin is cleared and the new stacks written, the drop gone.
fn try_move_coins_in_chest(
    drop_kind: u16,
    drop_count: u32,
    slots: &mut [crate::item::ItemSlot],
    max_items: usize,
) -> bool {
    if !crate::registry::is_coin(drop_kind) {
        return false;
    }
    let value = i64::from(crate::registry::get(drop_kind).map_or(0, |d| d.value));
    let total = coins_count(&slots[..max_items.min(slots.len())])
        + value / 5 * i64::from(drop_count);
    let split = coins_split(total);
    // `:2713-2732`: reserve one slot per non-zero denomination, high→low,
    // skipping occupied non-coin slots; a shortfall aborts before any write.
    let mut targets = [usize::MAX; 4];
    let mut cursor = 0usize;
    for idx in (0..4).rev() {
        if split[idx] == 0 {
            continue;
        }
        loop {
            if cursor >= max_items {
                return false;
            }
            let air = slots.get(cursor).is_none_or(|s| s.kind == 0 || s.count == 0);
            if air || crate::registry::is_coin(slots.get(cursor).map_or(0, |s| s.kind)) {
                break;
            }
            cursor += 1;
        }
        targets[idx] = cursor;
        cursor += 1;
    }
    // `:2733-2740`: clear every existing coin.
    for s in slots.iter_mut().take(max_items.min(slots.len())) {
        if crate::registry::is_coin(s.kind) {
            *s = crate::item::EMPTY_SLOT;
        }
    }
    // `:2741-2754`: write the new stacks (capped at the denomination maxStack).
    for idx in (0..4).rev() {
        if split[idx] == 0 {
            continue;
        }
        let kind = 71 + idx as u16;
        let max = i64::from(crate::registry::max_stack(kind));
        let stack = split[idx].min(max);
        let t = targets[idx];
        if t >= slots.len() {
            slots.resize(t + 1, crate::item::EMPTY_SLOT);
        }
        slots[t] = crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: stack as u32,
            prefix: 0,
        };
    }
    true
}

/// `Wiring.TryAddingToStack` (`Wiring.cs:2772-2798`): merge the drop into a
/// matching chest stack up to its maxStack. Returns `num > 0` — true on ANY
/// merge (the C# `:2797`), even a partial one: the drop keeps its remainder
/// and `TryToPutItemInChest` returns true (the hopper then marks the drop
/// placed; the C# `worldItem.stack` carries the leftover, but the hopper's
/// `SyncItem`/visualize run regardless — the port consumes the whole drop on
/// a successful place, the single-player fold). The `chestItem.type == 0`
/// clone arm (`:2790-2796`) is unreachable here (a type-0 slot fails
/// `CanStack` first), so it is not ported.
fn try_adding_to_stack(
    drop_kind: u16,
    drop_prefix: u16,
    drop_count: &mut u32,
    slot: &mut crate::item::ItemSlot,
) -> bool {
    let max = crate::registry::max_stack(slot.kind);
    if slot.count >= max || slot.kind != drop_kind || slot.prefix != drop_prefix {
        return false;
    }
    let room = max - slot.count;
    let take = (*drop_count).min(room);
    if take == 0 {
        return false;
    }
    *drop_count -= take;
    slot.count += take;
    true
}

/// `Wiring.TryToPutItemInChest` (`Wiring.cs:2680-2701`): coins ride the coin
/// arm; otherwise the first stack merge, then the first empty slot. Returns
/// `(placed, remaining, via_empty_slot)` — `placed` is the C# bool (any merge
/// or empty-slot fill), `remaining` is the drop's leftover stack (the C#
/// `worldItem.stack` after the partial merge; 0 when fully consumed /
/// `TurnToAir`), `via_empty_slot` marks the `TryAddingToEmptySlot` arm so the
/// caller runs its `PlaySound(7)` (`:2765`) on the wiring stream.
fn try_put_item_in_chest(
    drop: &crate::item::ItemDrop,
    slots: &mut [crate::item::ItemSlot],
    max_items: usize,
) -> (bool, u32, bool) {
    if crate::registry::is_coin(drop.kind) {
        let placed = try_move_coins_in_chest(drop.kind, drop.count, slots, max_items);
        // `TryMoveCoinsInChest` always `TurnToAir`s the drop on success
        // (`Wiring.cs:2755`); a failure leaves it untouched. No cue on this
        // arm — only `TryAddingToEmptySlot` plays (`:2765`).
        return (placed, if placed { 0 } else { drop.count }, false);
    }
    let mut left = drop.count;
    // `:2686-2692`: the stack-merge pass over the chest's slots. A partial
    // merge returns true with a remainder (the C# `num > 0`, `:2797`).
    for slot in slots.iter_mut().take(max_items.min(slots.len())) {
        if try_adding_to_stack(drop.kind, drop.prefix, &mut left, slot) {
            return (true, left, false);
        }
    }
    // `:2693-2699`: the first empty slot takes the whole (remaining) drop —
    // `TryAddingToEmptySlot` (`:2759-2770`): clone + `newAndShiny = false`
    // fold + `PlaySound(7)` (the caller's cue, see the hopper).
    for idx in 0..max_items {
        let air = slots.get(idx).is_none_or(|s| s.kind == 0 || s.count == 0);
        if air {
            if idx >= slots.len() {
                slots.resize(idx + 1, crate::item::EMPTY_SLOT);
            }
            slots[idx] = crate::item::ItemSlot {
                kind: drop.kind,
                name: crate::registry::name(drop.kind),
                count: left,
                prefix: drop.prefix,
            };
            return (true, 0, true);
        }
    }
    (false, drop.count, false)
}

/// `Wiring.Hopper` (`Wiring.cs:2638-2678`, `HitWireSingle` cases 21/467): the
/// 2×2 anchor from the negative frame offsets, `CheckMech(60)`, the locked /
/// in-use gates, then every ground item inside the 192×192 grab box centred on
/// the chest is pulled into the chest (`TryToPutItemInChest`); a successful
/// pull sorts the chest (`ItemSorting.SortInventory`, withSync/withFeedback
/// false). The C#'s `VisualizeChestTransfer` is a draw-only effect (no port).
fn hopper_pull(app: &mut App, i: i32, j: i32, kind: u16) {
    let Some(tile) = app.world.tile(i, j).copied() else {
        return;
    };
    if tile.kind != kind {
        return;
    }
    // `:2643-2650`: the anchor — `frameX % 36 != 0` steps left, `frameY % 36
    // != 0` steps up.
    let mut ax = i;
    let mut ay = j;
    if tile.frame_x % 36 != 0 {
        ax -= 1;
    }
    if tile.frame_y % 36 != 0 {
        ay -= 1;
    }
    // `:2651-2659`: CheckMech(60), the locked tile, the chest's existence and
    // the in-use gate.
    if !app.wiring.check_mech(ax, ay, 60) || crate::chest::is_locked(&app.world, ax, ay) {
        return;
    }
    let open = app.player.as_ref().map_or(-1, |p| p.chest);
    let Some(idx) = crate::chest::find_chest(&app.world_chests, ax, ay) else {
        return;
    };
    if crate::chest::using_chest(&app.world_chests, idx, open) {
        return;
    }
    // `:2660`: the 192×192 grab box centred on `(ax*16+16, ay*16+16)`
    // (`HopperGrabHitboxSize`, `Wiring.cs:3429`; `Utils.CenteredRectangle`
    // floors the top-left, `Utils.cs:947-950`).
    let cx = ax as f32 * 16.0 + 16.0;
    let cy = ay as f32 * 16.0 + 16.0;
    let (gx, gy) = ((cx - 96.0) as i32 as f32, (cy - 96.0) as i32 as f32);
    let max_items = app.world_chests[idx].max_items;
    let mut pulled = false;
    // `:2663-2673`: the 400-slot item scan — active, local-reserved, not banned,
    // box-intersecting, and placeable. The port's single player folds the
    // `playerIndexTheItemIsReservedFor == Main.myPlayer` gate to "all drops".
    for drop in app.items.iter_mut() {
        if drop.count == 0 || HOPPER_BANNED.contains(&drop.kind) {
            continue;
        }
        // `worldItem.Hitbox.Intersects(value)` — the strict-`<` rectangle test
        // (edge-touching does not intersect, `Rectangle.Intersects`).
        if !crate::item::rects_intersect(drop.pos.x, drop.pos.y, crate::item::SIZE, crate::item::SIZE, gx, gy, 192.0, 192.0) {
            continue;
        }
        let slots = crate::chest::chest_slots_mut(&mut app.world_chests, idx)
            .expect("the chest exists");
        let (placed, remaining, via_empty_slot) = try_put_item_in_chest(drop, slots, max_items);
        if placed {
            // `:2765` — `TryAddingToEmptySlot`'s `SoundEngine.PlaySound(7)`:
            // the case-7 pitch draw (`Next(-10, 11) * 0.01`,
            // `LegacySoundPlayer.cs:725`) rides the wiring stream (the
            // `HitWireSingle` site's `Main.rand`, the S1-L-6c contract) —
            // played only on the empty-slot fill, never on a stack merge or
            // the coin arm (W4-f11 fidelity port, research-71 §3b.1).
            if via_empty_slot {
                app.audio.play_pickup_sound(false, app.wiring.rand_mut());
            }
            // `:2669-2671`: the drop's stack carries the leftover (a partial
            // merge); `TurnToAir` only when it emptied. `SyncItem`/
            // `VisualizeChestTransfer` are net/draw effects (no port).
            drop.count = remaining;
            if remaining == 0 {
                drop.kind = 0;
            }
            pulled = true;
        }
    }
    // `:2674-2677`: a successful pull sorts the chest (no sync, no feedback).
    if pulled {
        let mult = app.player.as_ref().map_or([1.0; 4], |p| {
            [p.melee_damage, p.ranged_damage, p.magic_damage, p.minion_damage]
        });
        if let Some(slots) = crate::chest::chest_slots_mut(&mut app.world_chests, idx) {
            crate::item_sorting::sort(slots, &[], false, mult);
        }
    }
}

/// `Wiring.TryFindChestForExtractinator` (`Wiring.cs:2586-2608`): the first
/// unlocked chest in the 8×8 region around the 3×3 anchor (`lookupX-2 ..
/// lookupX+5`, `lookupY-2 .. lookupY+5`), scanned row-major.
fn find_chest_for_extractinator(app: &App, ax: i32, ay: i32) -> Option<usize> {
    for x in ax - 2..=ax + 5 {
        for y in ay - 2..=ay + 5 {
            if crate::chest::is_locked(&app.world, x, y) {
                continue;
            }
            if let Some(idx) = crate::chest::find_chest(&app.world_chests, x, y) {
                return Some(idx);
            }
        }
    }
    None
}

/// `Wiring.Extractinator` (`Wiring.cs:2554-2584`, `HitWireSingle` cases
/// 219/642): the 3×3 anchor from `frameX/Y % 54 / 18`, `CheckMech(60)`, the
/// nearby-chest search and the in-use gate, then the LAST non-air chest slot
/// rolls through `ExtractinatorHelper.RollExtractinatorDrop`; a positive output
/// consumes one input and spawns the drop at the anchor (`Item.NewItem`,
/// 32×32 box). The roll + drop draw ride the wiring stream (the `HitWireSingle`
/// `Main.rand` site, the S1-L-6c contract).
fn extractinator_roll(app: &mut App, i: i32, j: i32, kind: u16) {
    let Some(tile) = app.world.tile(i, j).copied() else {
        return;
    };
    if tile.kind != kind {
        return;
    }
    // `:2559-2560`: the 3×3 anchor.
    let ax = i - i32::from(tile.frame_x % 54 / 18);
    let ay = j - i32::from(tile.frame_y % 54 / 18);
    // `:2561-2564`: CheckMech(60), the chest search, the in-use gate.
    if !app.wiring.check_mech(ax, ay, 60) {
        return;
    }
    let open = app.player.as_ref().map_or(-1, |p| p.chest);
    let Some(idx) = find_chest_for_extractinator(app, ax, ay) else {
        return;
    };
    if crate::chest::using_chest(&app.world_chests, idx, open) {
        return;
    }
    // `:2567-2583`: the last non-air slot rolls; a positive output consumes one
    // input and spawns the drop, then breaks (one extraction per wire hit).
    let max_items = app.world_chests[idx].max_items;
    let slots = crate::chest::chest_slots_mut(&mut app.world_chests, idx)
        .expect("the chest exists");
    for slot_idx in (0..max_items.min(slots.len())).rev() {
        let (in_kind, in_count) = (slots[slot_idx].kind, slots[slot_idx].count);
        if in_kind == 0 || in_count == 0 {
            continue;
        }
        let mode = tr_core::extract::extractinator_mode(in_kind);
        let (item_type, stack) = tr_core::extract::roll_extractinator_drop(
            mode,
            kind,
            app.world.hard_mode,
            app.wiring.rand_mut(),
        );
        if item_type > 0 {
            // `:2575-2578`: `--item2.stack <= 0` → TurnToAir.
            if in_count <= 1 {
                slots[slot_idx] = crate::item::EMPTY_SLOT;
            } else {
                slots[slot_idx].count = in_count - 1;
            }
            // `:2579`: `Item.NewItem(source, num*16, num2*16, 32, 32, ...)` —
            // the centre is `(ax*16+16, ay*16+16)`.
            let rng = app.wiring.rand_mut();
            crate::world_sim::items::spawn_item_drop_px(
                &mut app.items,
                rng,
                item_type as u16,
                stack.max(1) as u32,
                Vec2::new(ax as f32 * 16.0 + 16.0, ay as f32 * 16.0 + 16.0),
            );
            break;
        }
    }
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
) {
    if crate::projectile::proj_data(kind).is_some() {
        app.projectiles
            .spawn_with_data_ai(kind, x, y, vx, vy, damage, knock_back, ai0, ai1);
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
        // base and block both identical requests. The cap anchor is
        // `(i*16+8, j*18+8)` = (168, 368) for the trap at (10, 20); the four
        // 185s' CENTERS sit ~40 px below it (pos (161, 321+k) → center
        // (168, 328+k), distance 40−k < 50, each −50).
        let mut app = app_with(137, 18, -7 * 18);
        for k in 0..4 {
            app.projectiles
                .spawn_with_data(
                    crate::projectile::TRAP_SPIKY_BALL,
                    161.0,
                    321.0 + k as f32,
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

    #[test]
    fn dw_4_2_the_spiky_ball_cap_measures_from_the_verbatim_anchor_to_center() {
        // F3 (`Wiring.cs:1819`/`:1950`): the cap measures from `(i*16+8,
        // j*18+8)` — the 18 is verbatim — against the projectile's Center
        // (`pos + half-box`), NOT the spawn point against `pos`. The trap at
        // (10, 20): cap anchor (168, 368), spawn point (168, 342). Four 185s
        // at pos (161, 392+k) → CENTER (168, 399+k): the new reading measures
        // |368 − (399+k)| = 31+k < 50 (each −50, count 0 → BLOCKED); the old
        // spawn-point/pos reading measured hypot(7, 342−(392+k)) ≈ 50.5+ ≥ 50
        // (each −15, count 140 → ALLOWED). So this fixture discriminates the
        // fix: the verbatim anchor + Center reading blocks both requests.
        let mut app = app_with(137, 18, -7 * 18);
        for k in 0..4 {
            app.projectiles
                .spawn_with_data(
                    crate::projectile::TRAP_SPIKY_BALL,
                    161.0,
                    392.0 + k as f32,
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
        assert_eq!(live, 4, "the verbatim anchor + Center reading blocks both");
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

    /// A chest at `(x, y)` holding the given slots (the hopper/extractinator
    /// fixtures). The chest tile is a plain container (21, frame 0 — unlocked).
    fn chest_at(app: &mut App, x: i32, y: i32, slots: Vec<crate::item::ItemSlot>) -> usize {
        let i = app.world.index(x, y);
        app.world.tiles[i] = tr_core::tile::Tile {
            kind: 21,
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        let idx = crate::chest::create_chest(&mut app.world_chests, x, y);
        app.world_chests[idx].slots = slots;
        idx
    }

    fn slot(kind: u16, count: u32) -> crate::item::ItemSlot {
        crate::item::ItemSlot { kind, name: "", count, prefix: 0 }
    }

    #[test]
    fn dw_e20_the_switch_track_flips_front_and_back() {
        // `Minecart.FlipSwitchTrack` (`Minecart.cs:1296-1319`): a normal track
        // (frame 1 straight, type 0) with a back frame swaps front/back. The
        // device at (10, 20) carries front 1 / back 5; the wire hit flips them.
        let mut app = app_with(314, 1, 5);
        trip_and_tick(&mut app);
        assert_eq!(app.world.tile_frame_x(10, 20), 5, "front takes the back");
        assert_eq!(app.world.tile_frame_y(10, 20), 1, "back takes the front");
    }

    #[test]
    fn dw_e20_the_hopper_pulls_a_ground_item_into_the_chest() {
        // `Wiring.Hopper` (`Wiring.cs:2638-2678`): the 2×2 chest at (10, 20),
        // a dirt drop inside the 192×192 grab box is pulled in and the chest
        // sorted. The hopper tile (21) is the chest's own anchor cell.
        let mut app = app_with(21, 0, 0);
        let idx = chest_at(&mut app, 10, 20, vec![]);
        // A dirt drop at the chest centre (well inside the grab box).
        app.items.push(crate::item::ItemDrop::new(
            2, 5, 0, 10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0, 0.0, 0.0, 0.0,
        ));
        trip_and_tick(&mut app);
        assert_eq!(app.items[0].count, 0, "the drop was consumed");
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert!(slots.iter().any(|s| s.kind == 2 && s.count == 5), "the dirt landed");
    }

    #[test]
    fn dw_e20_the_hopper_merges_coins_into_denominations() {
        // `Wiring.TryMoveCoinsInChest` (`Wiring.cs:2703-2757`): a copper drop
        // (71, value 5 → 1 copper each) merges with the chest's coins and the
        // total re-splits. 150 copper in the chest + a 60-copper drop = 210 →
        // 2 silver (200) + 10 copper.
        let mut app = app_with(21, 0, 0);
        let idx = chest_at(&mut app, 10, 20, vec![slot(71, 150)]);
        app.items.push(crate::item::ItemDrop::new(
            71, 60, 0, 10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0, 0.0, 0.0, 0.0,
        ));
        trip_and_tick(&mut app);
        assert_eq!(app.items[0].count, 0, "the coin drop was consumed");
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        let copper: u32 = slots.iter().filter(|s| s.kind == 71).map(|s| s.count).sum();
        let silver: u32 = slots.iter().filter(|s| s.kind == 72).map(|s| s.count).sum();
        assert_eq!((copper, silver), (10, 2), "210 copper = 2 silver + 10 copper");
    }

    #[test]
    fn dw_e20_the_hopper_partial_merge_keeps_the_remainder() {
        // `Wiring.TryAddingToStack` (`Wiring.cs:2772-2798`): a partial merge
        // returns `num > 0` (true) and the drop keeps its remainder. Dirt
        // maxStack 9999; a chest slot at 9990 takes 9 of a 20-drop, leaving 11
        // on the ground (the C# `worldItem.stack` carries the leftover).
        let mut app = app_with(21, 0, 0);
        let idx = chest_at(&mut app, 10, 20, vec![slot(2, 9990)]);
        app.items.push(crate::item::ItemDrop::new(
            2, 20, 0, 10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0, 0.0, 0.0, 0.0,
        ));
        trip_and_tick(&mut app);
        assert_eq!(app.items[0].count, 11, "the remainder stays on the ground");
        assert_eq!(app.items[0].kind, 2, "the drop is still active");
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert_eq!(slots[0].count, 9999, "the chest stack filled to maxStack");
    }

    #[test]
    fn dw_e20_the_extractinator_rolls_the_last_chest_slot() {
        // `Wiring.Extractinator` (`Wiring.cs:2554-2584`): the 3×3 extractinator
        // at (10, 20), a chest nearby holding silt (424, mode 0). The last slot
        // rolls; a positive output consumes one input and spawns a drop. The
        // wiring RNG is seeded so the roll is deterministic — silt's coin arm
        // (`num7 = 1`, `Next(0, 2) == 0`) fires on the first draw for seed 7.
        let mut app = app_with(219, 0, 0);
        app.wiring.seed_rand(7);
        // The chest sits at (12, 20) — inside the 8×8 search region around the
        // anchor (10, 20), and NOT on the extractinator's own cells.
        let idx = chest_at(&mut app, 12, 20, vec![slot(424, 3)]);
        trip_and_tick(&mut app);
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert_eq!(slots[0].count, 2, "one silt consumed");
        assert!(!app.items.is_empty(), "the roll spawned a drop");
    }

    #[test]
    fn dw_e20_the_extractinator_skips_a_non_extractable_slot() {
        // `ExtractinatorHelper.RollExtractinatorDrop` mode −1 (`ExtractinatorHelper.cs:20-23`):
        // a non-extractable input (dirt, 2) rolls nothing — the slot is untouched
        // and no drop spawns. The scan continues to earlier slots; with only the
        // dirt slot the whole hit is a no-op.
        let mut app = app_with(219, 0, 0);
        app.wiring.seed_rand(7);
        let idx = chest_at(&mut app, 12, 20, vec![slot(2, 3)]);
        trip_and_tick(&mut app);
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert_eq!(slots[0].count, 3, "the dirt is untouched");
        assert!(app.items.is_empty(), "no drop from a mode -1 roll");
    }

    /// W4-f11 fidelity port (research-71 §3b.1): `TryAddingToEmptySlot`'s
    /// `PlaySound(7)` (`Wiring.cs:2765`) — the empty-slot hopper fill draws
    /// exactly ONE `Next(-10, 11)` pitch (`LegacySoundPlayer.cs:725`) on the
    /// wiring stream; the stack-merge and coin arms draw nothing (no cue).
    #[test]
    fn w4_f11_the_hopper_empty_slot_fill_draws_the_case7_pitch() {
        use tr_core::unified_random::UnifiedRandom;
        // Empty chest → the dirt drop enters via the empty-slot arm.
        let mut app = app_with(21, 0, 0);
        app.wiring.seed_rand(7);
        let idx = chest_at(&mut app, 10, 20, vec![]);
        app.items.push(crate::item::ItemDrop::new(
            2, 5, 0, 10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0, 0.0, 0.0, 0.0,
        ));
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-10, 11);
        trip_and_tick(&mut app);
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert!(slots.iter().any(|s| s.kind == 2 && s.count == 5), "the dirt landed");
        assert_eq!(
            app.wiring.rand_mut().peek(),
            witness.peek(),
            "exactly one case-7 pitch draw on the wiring stream"
        );
        // The stack-merge arm plays no cue: a second same-kind drop merges
        // into the landed stack without touching the stream.
        let peek = app.wiring.rand_mut().peek();
        app.items.push(crate::item::ItemDrop::new(
            2, 3, 0, 10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0, 0.0, 0.0, 0.0,
        ));
        // Drain the 60-tick hopper cooldown, then re-hit.
        for _ in 0..60 {
            update_wiring(&mut app, 1.0 / 60.0);
        }
        trip_and_tick(&mut app);
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert!(slots.iter().any(|s| s.kind == 2 && s.count == 8), "the merge ran");
        assert_eq!(app.wiring.rand_mut().peek(), peek, "the stack merge is draw-free");
        // The coin arm plays no cue either (the coin merge is draw-free).
        let peek = app.wiring.rand_mut().peek();
        app.items.push(crate::item::ItemDrop::new(
            71, 4, 0, 10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0, 0.0, 0.0, 0.0,
        ));
        for _ in 0..60 {
            update_wiring(&mut app, 1.0 / 60.0);
        }
        trip_and_tick(&mut app);
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert!(slots.iter().any(|s| s.kind == 71 && s.count == 4), "the coins landed");
        assert_eq!(app.wiring.rand_mut().peek(), peek, "the coin arm is draw-free");
    }

    /// W4-f11 port (research-71 §3b.3): `Utils.CoinsSplit`
    /// (`Utils.cs:1048-1060`) — 1234567 copper → [67, 45, 23, 1]
    /// (platinum 1, gold 23, silver 45, copper 67).
    #[test]
    fn w4_f11_coins_split_matches_the_csharp_denominations() {
        assert_eq!(coins_split(1_234_567), [67, 45, 23, 1]);
        assert_eq!(coins_split(0), [0, 0, 0, 0]);
        assert_eq!(coins_split(100), [0, 1, 0, 0]);
        assert_eq!(coins_split(1_000_000), [0, 0, 0, 1]);
    }

    /// W4-f11 port (research-71 §3b.3), re-pointed at the kept `hopper_pull`
    /// body: the grab rect is 192×192 centred on `(ax*16+16, ay*16+16)`
    /// (`Wiring.cs:2660`, `HopperGrabHitboxSize`; `Utils.CenteredRectangle`
    /// floors the top-left). Anchor (10, 20) → centre (176, 336), box
    /// [80, 272) × [240, 432); the strict-`<` test excludes the touching edge.
    #[test]
    fn w4_f11_the_hopper_grab_rect_is_192_centred() {
        let (cx, cy) = (10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0);
        assert_eq!((cx - 96.0, cy - 96.0), (80.0, 240.0), "the floored top-left");
        // A 14×14 drop at x 258 reaches 272 — the last intersecting column.
        let mut app = app_with(21, 0, 0);
        let idx = chest_at(&mut app, 10, 20, vec![]);
        app.items.push(crate::item::ItemDrop::new(
            2, 5, 0, 258.0, 330.0, 0.0, 0.0, 0.0,
        ));
        trip_and_tick(&mut app);
        assert_eq!(app.items[0].count, 0, "the inside-edge drop was grabbed");
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert!(slots.iter().any(|s| s.kind == 2 && s.count == 5));
        // At x 272 the drop's left edge touches the box's right edge — the
        // strict-`<` `Rectangle.Intersects` refuses (474bc8b5's rule).
        let mut app = app_with(21, 0, 0);
        let idx = chest_at(&mut app, 10, 20, vec![]);
        app.items.push(crate::item::ItemDrop::new(
            2, 5, 0, 272.0, 330.0, 0.0, 0.0, 0.0,
        ));
        trip_and_tick(&mut app);
        assert_eq!(app.items[0].count, 5, "the touching-edge drop stays out");
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert!(slots.iter().all(|s| s.count == 0), "the chest stayed empty");
    }

    /// W4-f11 port (research-71 §3b.3), re-pointed at `hopper_pull`: the
    /// locked gate (`Wiring.cs:2651`, `Chest.IsLocked`) — a golden chest
    /// (frameX 72, `Chest.cs:301-313`) refuses the grab; the drop stays.
    #[test]
    fn w4_f11_a_locked_chest_refuses_the_hopper() {
        let mut app = app_with(21, 0, 0);
        let idx = chest_at(&mut app, 10, 20, vec![]);
        // Lock the anchor tile (72 % 36 == 0 keeps the anchor at (10, 20)).
        let i = app.world.index(10, 20);
        app.world.tiles[i].frame_x = 72;
        app.items.push(crate::item::ItemDrop::new(
            2, 5, 0, 10.0 * 16.0 + 16.0, 20.0 * 16.0 + 16.0, 0.0, 0.0, 0.0,
        ));
        trip_and_tick(&mut app);
        assert_eq!(app.items[0].count, 5, "the locked chest kept the drop out");
        let slots = crate::chest::chest_slots(&app.world_chests, idx).unwrap();
        assert!(slots.iter().all(|s| s.count == 0), "nothing entered");
    }
}
