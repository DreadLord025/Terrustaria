//! The tile right-click INTERACT/DOORS family of the editing loop (S1-K-4e):
//! `interact_tile` (the port's `Player.TileInteractionsUse` hook), the door
//! open/close/blocked cue + drop mapping (`interaction_events`), the furniture
//! arms (bank/chest toggle, lever/switch/timer, chair sit, bed spawn/sleep,
//! clock, candle, station filter), the Lihzahrd Altar summon
//! (`Player.cs:32755-32790`, `NPC.cs:80475-80531`), the snap gate
//! (`Player.IsWithinSnappngRangeToTile`), the `Player.CheckSpawn` bed checks
//! and the `Collision.EmptyTile` entity half, moved verbatim out of
//! `world_sim/editing.rs`; `interact_tile` is `pub(super)` and re-imported by
//! `editing`, so the `update_editing` caller seam is unchanged, and
//! `bed_spawn_valid`/`check_bed_spawn` are `pub(crate) use`-re-exported so
//! `player.rs`/`world_sim/player.rs` stay unchanged.

use tr_core::tiledata as td;
use tr_core::{TILE_PX, edit};

use crate::app::App;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use crate::registry;

use super::super::items::spawn_item_drop;

/// The right-click interaction hook (the port's `TileInteractionsUse`):
/// dispatches the world interaction at `(tx, ty)` through
/// [`tr_core::edit::interact_at`] and applies its effects — the door cue
/// (`SoundEngine.PlaySound(8/:9)`, `WorldGen.cs:37732/:31906`), the drop
/// spawns of any cleanup kills, the frame-cache invalidation. Returns whether
/// the click was consumed; `false` lets the caller fall through to the place
/// path.
/// `TileInteractionsUse`'s Lihzahrd Altar arm (`Player.cs:32755-32790`):
/// the gates are `!NPC.AnyNPCs(245) && Main.hardMode &&
/// NPC.downedPlantBoss`; the 58-slot inventory scan consumes one Power
/// Cell 1293; the roar is `SoundID.Roar` (15) and the summon is
/// `NPC.SpawnOnPlayer(plr, 245)`. The click is consumed whether or not
/// the gates pass (the C#'s `flag2 = true`).
fn use_golem_altar(app: &mut App) -> bool {
    let gated = app
        .npcs
        .iter()
        .any(|n| n.hp > 0 && n.kind == NpcKind::Golem)
        || !app.world.hard_mode
        || !app.world.downed_plant_boss;
    if !gated {
        // The inventory scan: the first 1293 row decrements; a zero
        // stack clears (`SetDefaults(0)`) exactly like the C#.
        for slot in 0..app.inventory.len() {
            if app.inventory[slot].kind == 1293 && app.inventory[slot].count > 0 {
                app.inventory[slot].count -= 1;
                if app.inventory[slot].count == 0 {
                    app.inventory[slot].kind = 0;
                }
                app.audio.play("Roar_0");
                spawn_golem_on_altar(app);
                break;
            }
        }
    }
    true
}

/// `NPC.SpawnOnPlayer`'s case-245 arm (`NPC.cs:80475-80531`): the
/// ±20-tile scan around the player for the altar's origin cell
/// (`type == 237 && frameX == 18 && frameY == 0`), the 50-tile solid
/// scans up and down that column (`WorldGen.SolidTile`'s
/// active/!solidTop/!halfBrick/slope-0 shape — an out-of-bounds cell
/// reads solid, the C# `null` case), then the Golem lands at the
/// column's `(up + down) / 3` pixel Y, targeting the player, with the
/// "HasAwoken" announcement. Returns false when no altar is in range
/// (the C#'s try/catch no-spawn path).
fn spawn_golem_on_altar(app: &mut App) -> bool {
    let Some(player) = app.player.as_ref() else {
        return false;
    };
    let (pcx, pcy) = player.center();
    let cx_t = (pcx / TILE_PX) as i32;
    let cy_t = (pcy / TILE_PX) as i32;
    let mut altar: Option<(i32, i32)> = None;
    for j in (cx_t - 20)..=(cx_t + 20) {
        for k in (cy_t - 20)..=(cy_t + 20) {
            if app
                .world
                .tile(j, k)
                .is_some_and(|t| t.active() && t.kind == 237 && t.frame_x == 18 && t.frame_y == 0)
            {
                altar = Some((j, k));
            }
        }
    }
    let Some((ax, ay)) = altar else {
        return false;
    };
    let solid = |j: i32, k: i32| -> bool {
        match app.world.tile(j, k) {
            None => true,
            Some(t) => {
                t.active()
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                    && !t.half_brick()
                    && t.slope() == 0
            }
        }
    };
    let mut up = ay - 15;
    for k in ((ay - 49).max(0)..=ay).rev() {
        if solid(ax, k) {
            up = k;
            break;
        }
    }
    let mut down = ay - 15;
    for k in ay..(ay + 50) {
        if solid(ax, k) {
            down = k;
            break;
        }
    }
    let spawn_x = ax as f32 * TILE_PX + 8.0;
    let spawn_y = ((up + up + down) / 3) as f32 * TILE_PX;
    let mut golem = Npc::new(NpcKind::Golem, spawn_x, spawn_y, 1, 0.0);
    golem.hp = golem.max_hp;
    app.npcs.push(golem);
    app.chat
        .new_text("The Golem has awoken!", crate::chat::COLOR_SERVER_MESSAGE);
    true
}

pub(super) fn interact_tile(app: &mut App, tx: i32, ty: i32) -> bool {
    // S1-P-i55 — the golf tee's right-click arm (`TileInteractionsUse`'s
    // `type == 494` branch, `Player.cs:32342-32354`): EVERY owned golf ball
    // dies (the C# loop tests type+owner only — no `active` gate), then the
    // preferred ball spawns on THIS tee with `ai1 = -1` (the sentinel).
    // The arm consumes the click (`flag2 = true`) whether or not a ball
    // spawned — the C# has no reach check here (the click's own tile-range
    // gate ran upstream).
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && t.kind == crate::projectile_ai_golf::TILE_GOLF_TEE)
    {
        let owner = app.local_id as u8;
        for q in app.projectiles.slots_mut() {
            if tr_core::golf_state::is_a_golf_ball(q.kind as i32) && q.owner == owner {
                q.active = false;
                q.time_left = 0;
            }
        }
        let preferred = crate::projectile_ai_golf::preferred_golf_ball_from_inventory(
            app.selected,
            &app.inventory,
            &app.player.as_ref().map(|p| p.equipped).unwrap_or([0; 20]),
        );
        // `:32353` — the spawn at `(x*16 + 8.5, y*16 + 6)` CENTRE, zero
        // velocity, damage 0, ai0 0 / ai1 -1. The port's seam takes the
        // top-left, so the row's half-box comes off.
        let row = crate::projectile::proj_data(preferred);
        let (hw, hh) = row.map_or((3.5, 3.5), |d| (d.w * 0.5, d.h * 0.5));
        app.projectiles.spawn_with_data_ai_owned(
            preferred,
            owner,
            tx as f32 * 16.0 + 8.5 - hw,
            (ty * 16) as f32 + 6.0 - hh,
            0.0,
            0.0,
            0,
            0.0,
            0.0,
            -1.0,
        );
        crate::projectile_ai_golf::cull_extra_golf_balls(&mut app.projectiles, owner, preferred, 0);
        return true;
    }
    // W4-d8 DOC-SWEEP (5) — the firework fountain's right-click arm
    // (`TileInteractionsUse`'s `type == 338` branch, `Player.cs:32355-
    // 32377`): the anchor shifts UP one when the clicked cell's frameY is
    // 18 (the fountain's bottom row), the pool dedupe (an active aiStyle-73
    // shot anchored at the same tile skips the spawn), then ONE fountain
    // of `419 + Next(4)` at `(x*16 + 8, y*16 + 2)`, zero velocity, damage
    // 0, ai0/ai1 = the anchor tile. The `originatedFromActivableTile` flag
    // (`:32375`) has no consumer in the 1.4.5.8 decompile (the field is
    // written at 5 sites, read nowhere) — the port drops it (the census
    // note). The click is consumed (`flag2 = true`) whether or not the
    // dedupe passed. The wiring twin (`Wiring.cs:1522-1546`) already lives
    // in tr-core's `firework_fountain` + the world_sim dedupe (decision
    // D1); this is the proj-spawn side's own arm.
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && t.kind == 338)
    {
        // `:32358-32362` — the anchor: the clicked cell, up one on the
        // bottom row.
        let mut anchor_y = ty;
        if app.world.tile(tx, anchor_y).is_some_and(|t| t.frame_y == 18) {
            anchor_y -= 1;
        }
        // `:32363-32371` — the dedupe scan: an active aiStyle-73 fountain
        // anchored at (tx, anchor_y) blocks the spawn.
        let already = app.projectiles.slots().iter().any(|p| {
            p.active
                && crate::projectile::FIREWORK_FOUNTAINS.contains(&p.kind)
                && p.ai[0] == tx as f32
                && p.ai[1] == anchor_y as f32
        });
        if !already {
            // `:32374` — `419 + Main.rand.Next(4)` on the UpdatePlayers
            // stream (the TileInteractionsUse ride, `Main.cs:17754`).
            let kind = 419
                + app
                    .rngs
                    .stream(crate::app::RngKey::UpdatePlayers)
                    .next_range(0, 4) as u16;
            // The C# passes the CENTRE; the port's seam takes the top-left
            // — the fountain row's 4×4 half-box comes off (the golf-tee
            // precedent above).
            let (hw, hh) = crate::projectile::proj_data(kind)
                .map_or((2.0, 2.0), |d| (d.w * 0.5, d.h * 0.5));
            app.projectiles.spawn_with_data_ai_owned(
                kind,
                app.local_id as u8,
                tx as f32 * 16.0 + 8.0 - hw,
                anchor_y as f32 * 16.0 + 2.0 - hh,
                0.0,
                0.0,
                0,
                0.0,
                tx as f32,
                anchor_y as f32,
            );
        }
        return true;
    }
    // The Lihzahrd Altar arm (`TileInteractionsUse`'s `type == 237`
    // branch, `Player.cs:32755-32790`): the right-click on the altar
    // consumes the click and, with a Power Cell 1293 in the INVENTORY
    // (the vanilla scans the inventory, not the held item), no living
    // Golem, hardmode and Plantera down, summons the boss. The arm runs
    // before the furniture dispatch — the altar has no `interact_at`
    // interaction of its own.
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && t.kind == 237)
    {
        return use_golem_altar(app);
    }
    let direction = app.player.as_ref().map(|p| p.facing).unwrap_or(1);
    // The entity half of `Collision.EmptyTile` (`Collision.cs:1359`): the
    // closing column must be free of the player and NPCs. Disjoint field
    // borrows — the world write happens inside `interact_at`.
    let (player, npcs) = (&app.player, &app.npcs);
    let cell_blocked = |x: i32, y: i32| cell_blocked_by_entity(player.as_ref(), npcs, x, y);
    let result = edit::interact_at(&mut app.world, tx, ty, direction, &cell_blocked);
    // The furniture interactions that touch the player/world state ahead of
    // the cue/drop mapping.
    if let edit::TileInteraction::BankOpen { bank } = result {
        // The bank tiles ride the same shared open block the world chests
        // use (`Player.cs:33230-33252` via `chest::toggle_open`, the
        // M3-CHEST-INT unify): the re-click closes (`chest = -1`, sound 11),
        // a fresh open plays 10, a switch plays 12. `OpenChest` rides inside
        // — `chest`/`chestX`/`chestY` (the C# origin decode,
        // `chest::bank_open_origin`) and `Main.playerInventory = true`
        // (`:33516-33525`); the close keeps the inventory open (the C#
        // never touches it here — the STORE-1 pair-close deviation is
        // gone). The prologue's sign/talk/chat clears (:33087-33096) are
        // the shop close, the world route's mapping (chest.rs:525).
        app.close_shop();
        let (ox, oy) = crate::chest::bank_open_origin(&app.world, tx, ty);
        let sound = crate::chest::toggle_open(app, ox, oy, bank);
        crate::chest::play_menu_sound(app, sound);
        app.world_key = None;
        return true;
    }
    match &result {
        edit::TileInteraction::HitSwitch { x, y } => {
            // `Player.cs:32174-32178`: the used tile is a lever (132), a
            // switch (136) or a timer (144) — `Wiring.HitSwitch` fires and
            // the click is consumed.
            app.wiring.hit_switch(&mut app.world, *x, *y);
            // S1-E-21i — the Mech-28 click (`Wiring.cs:285`/`:310`/`:375`):
            // the pitch draw rides the `UpdatePlayers` stream — the tile
            // use runs in `Player.ItemCheck` (`Main.cs:17754`).
            for (px, py) in app.wiring.take_switch_clicks() {
                app.audio.play_switch_click(
                    px as f32,
                    py as f32,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            app.send_hit_switch(*x, *y); // `:32178`/`:32226`/`:32340`
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::ChairSit {
            seat_x,
            seat_y,
            direction,
            seat_offset,
        } => {
            // `Player.cs:32313-32330`: the 40-px snap gate
            // (`IsWithinSnappngRangeToTile`) wraps the `sitting.SitDown`
            // call; a refused snap still consumes the click.
            // W4-f32 — the `IsAToilet` extra info
            // (`PlayerSittingHelper.cs:166-180`): tile 497 always, chair 15
            // on the frameY/40 == 1/20 styles; `TryToPoop`'s gate reads it.
            let is_toilet = app.world.tile(tx, ty).is_some_and(|t| {
                t.kind == 497 || (t.kind == 15 && matches!(t.frame_y / 40, 1 | 20))
            });
            if let Some(p) = app.player.as_mut() {
                if within_snapping_range(p, tx, ty, 40.0) {
                    p.sit_down(&app.world, *seat_x, *seat_y, *direction, *seat_offset);
                    // `sit_down` toggles off at the same seat — the flag
                    // follows the live sitting state.
                    p.sitting_on_toilet = p.sitting && is_toilet;
                }
            }
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::BedInteract {
            set_spawn,
            spawn_x,
            spawn_y,
        } => {
            if *set_spawn {
                // `Player.cs:32678-32691`: the same cell toggles the spawn
                // off; `CheckSpawn` gates the change with its messages.
                if app.world.bed_spawn == Some((*spawn_x, *spawn_y)) {
                    app.world.bed_spawn = None;
                    // `RemoveSpawn` → `FindSpawn`'s re-scan; the port resets
                    // to the generated spawn.
                    if let Some(p) = app.player.as_mut() {
                        p.spawn_x = app.world.spawn_x as f32 * tr_core::TILE_PX + 8.0
                            - crate::player::WIDTH as f32 * 0.5;
                        p.spawn_y = app.world.spawn_y as f32 * tr_core::TILE_PX
                            - crate::player::HEIGHT as f32;
                    }
                    app.chat
                        .new_text("Spawn point removed!", crate::chat::COLOR_SERVER_MESSAGE);
                } else if check_bed_spawn(&app.world, *spawn_x, *spawn_y, &mut app.chat) {
                    app.world.bed_spawn = Some((*spawn_x, *spawn_y));
                    // `ChangeSpawn` (`Player.cs:54635`).
                    if let Some(p) = app.player.as_mut() {
                        p.spawn_x = *spawn_x as f32 * tr_core::TILE_PX + 8.0
                            - crate::player::WIDTH as f32 * 0.5;
                        p.spawn_y =
                            *spawn_y as f32 * tr_core::TILE_PX - crate::player::HEIGHT as f32;
                    }
                    app.chat
                        .new_text("Spawn point set!", crate::chat::COLOR_SERVER_MESSAGE);
                }
            }
            // The sleep half arrives as `BedSleep` below — this arm is the
            // spawn toggle (and the consumed-but-inert decode fallback).
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::BedSleep {
            anchor_x,
            anchor_y,
            direction,
            bed_style,
        } => {
            // `Player.cs:32669-32677`: the 96-px snap gate wraps
            // `sleeping.StartSleeping`; a refused snap still consumes the
            // click.
            let slept = if let Some(p) = app.player.as_mut() {
                if within_snapping_range(p, tx, ty, 96.0) {
                    p.sleep_down(&app.world, *anchor_x, *anchor_y, *direction, *bed_style);
                    p.sleeping
                } else {
                    false
                }
            } else {
                false
            };
            // S1-P-i35 — `PlayerSleepingHelper.StartSleeping`'s
            // `RemoveAllFishingBobbers` (`PlayerSleepingHelper.cs:174`): the
            // sleep teardown kills the owner's bobbers (a reeled catch still
            // rides home through the kill hook).
            if slept {
                let owner = app.local_id as u8;
                let level = app.player.as_ref().map_or(0, |p| {
                    p.fishing_conditions(
                        &app.inventory,
                        &app.equip.coins,
                        &app.equip.ammo,
                        app.selected,
                        &app.world,
                        app.rain.is_raining(),
                        app.rain.cloud_bg(),
                    )
                    .4
                });
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                app.projectiles.remove_all_fishing_bobbers(owner, level, rng);
            }
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::ClockRead { text } => {
            app.chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
            return true;
        }
        edit::TileInteraction::CandleToggled => {
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::StationInteract { kind } => {
            // `Player.cs:33279-33284` (the `TileInteractionsUse` tail): the
            // clicked tile's filter lands in `CraftingUI._lastFilter` via
            // `OpenCloseFilter` (`:33301`/`:33297`); `this.AdjTiles()`
            // (`:33282`) — the port recomputes the adjacency per query. A
            // resolved filter consumes the click (`flag2 = true`); a tile
            // that is neither a station nor a water source leaves it false,
            // so the place path takes over.
            app.craft_filter = crate::craft::get_crafting_filter_for_tile(*kind);
            if app.craft_filter.is_some() {
                app.world_key = None;
                return true;
            }
            return false;
        }
        _ => {}
    }
    let Some((cue, drops)) = interaction_events(result) else {
        return false;
    };
    if let Some(cue) = cue {
        // S1-E-21i — the door open/close plays upgrade to the faithful
        // case-8/9 pitched path (`WorldGen.OpenDoor` `WorldGen.cs:37727` /
        // `CloseDoor` `:31908`; `LegacySoundPlayer.cs:728-749`'s one
        // `Next(-20, 21) * 0.01` draw). The interaction runs in
        // `Player.ItemCheck` — the `UpdatePlayers` stream (`Main.cs:17754`).
        // The other cues (the dial's Item_4) keep the plain play.
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        match cue {
            "Door_Opened" => {
                app.audio
                    .play_door_sound(true, tx as f32 * TILE_PX, ty as f32 * TILE_PX, rng);
            }
            "Door_Closed" => {
                app.audio
                    .play_door_sound(false, tx as f32 * TILE_PX, ty as f32 * TILE_PX, rng);
            }
            _ => {
                app.audio
                    .play_at(cue, tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
            }
        }
    }
    for (id, x, y) in drops {
        // The tile-interaction drops ride the `UpdatePlayers` stream (the
        // interaction runs in `Player.ItemCheck`, `Main.cs:17754`).
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        spawn_item_drop(&mut app.items, rng, id, 1, x, y);
    }
    app.world_key = None;
    true
}

/// `Player.IsWithinSnappngRangeToTile` (`Player.cs:17365-17372`): the tile
/// centre to `(Center.X, Bottom.Y - 16)` distance — the sit (40 px) and
/// sleep (96 px) snap gates.
fn within_snapping_range(p: &crate::player::Player, tx: i32, ty: i32, distance: f32) -> bool {
    let cx = tx as f32 * TILE_PX + 8.0;
    let cy = ty as f32 * TILE_PX + 8.0;
    let dx = cx - (p.x + crate::player::WIDTH * 0.5);
    let dy = cy - (p.y + p.height - 16.0);
    (dx * dx + dy * dy).sqrt() <= distance
}

/// The App-free half of [`interact_tile`]: maps the dispatcher result to the
/// door/dial cue and the drop spawns. `None` = no interaction (fall-through
/// to the place path); a blocked door or a refused dial yields no cue
/// (vanilla plays `PlaySound(8/:9)`/`SoundID.Item4` only on the fired paths),
/// while the click still counts as consumed.
fn interaction_events(
    result: edit::TileInteraction,
) -> Option<(Option<&'static str>, Vec<(u16, i32, i32)>)> {
    let (cue, object_drops) = match result {
        edit::TileInteraction::None => return None,
        edit::TileInteraction::DoorOpened { object_drops } => (Some("Door_Opened"), object_drops),
        edit::TileInteraction::DoorClosed { object_drops } => (Some("Door_Closed"), object_drops),
        edit::TileInteraction::DoorBlocked { object_drops } => (None, object_drops),
        // `SoundID.Item4` on both dial arms (`Player.cs:32642`/:32652); the
        // key is the vanilla asset name (see `audio::SFX_CUES`). The dials
        // kill no tiles, so there are never drops.
        edit::TileInteraction::DialStarted { .. } => (Some("Item_4"), Vec::new()),
        // The refused gate (`Player.cs:32638`): still consumed (`flag2 = true`,
        // `:32637`) — no cue, no drops, no state change.
        edit::TileInteraction::DialRefused => (None, Vec::new()),
        // The furniture interactions are consumed in `interact_tile` ahead of
        // this mapping; the arms only keep the match exhaustive.
        edit::TileInteraction::CandleToggled => (None, Vec::new()),
        // The switch trip is consumed in `interact_tile` ahead of this
        // mapping — no cue, no drops.
        edit::TileInteraction::HitSwitch { .. } => (None, Vec::new()),
        edit::TileInteraction::ChairSit { .. } => (None, Vec::new()),
        edit::TileInteraction::BedInteract { .. } => (None, Vec::new()),
        edit::TileInteraction::BedSleep { .. } => (None, Vec::new()),
        edit::TileInteraction::ClockRead { .. } => (None, Vec::new()),
        // The bank open is consumed in `interact_tile` ahead of this
        // mapping (the SoundID 10/11/12 keys are absent from the cue set).
        edit::TileInteraction::BankOpen { .. } => (None, Vec::new()),
        // The station probe is consumed in `interact_tile` ahead of this
        // mapping — no cue, no drops (the C# plays the craft-open sound 10
        // inside `OpenCloseFilter`'s non-quiet path, `NewCraftingUI.cs:894`).
        edit::TileInteraction::StationInteract { .. } => (None, Vec::new()),
        // M7-EXTRACT-1: the extractinator is NOT a click-edge interaction in
        // vanilla — `PlaceThing_ItemInExtractinator` runs inside the held-item
        // swing (`Player.cs:39591`, `:41665`), so the press edge falls through
        // to the place/swing block, whose `world_sim/extract.rs` arm owns the
        // extraction (the held-item gate, the roll, the drop, the consumption).
        // Returning `None` here keeps `interact_tile`'s "not consumed" answer
        // and the chest/shop/door seam untouched.
        edit::TileInteraction::Extractinator => return None,
    };
    let events = object_drops
        .iter()
        .filter_map(|d| registry::tile_drop(d.tile_kind, 0).map(|id| (id, d.x, d.y)))
        .collect();
    Some((cue, events))
}

/// `Player.CheckSpawn` (`Player.cs:54533-54590`) without the messages — the
/// checks the revive path and the load rebuild share (the C# runs the same
/// `CheckSpawn` inside `Player.Spawn`, `:37571-37575`). `Err` carries the
/// failure reason the chat wrapper prints.
pub(crate) fn bed_spawn_valid(
    world: &tr_core::World,
    x: i32,
    y: i32,
) -> Result<(), BedSpawnFailure> {
    if x < 10 || x > world.width as i32 - 10 || y < 10 || y > world.width as i32 - 10 {
        return Err(BedSpawnFailure::Bounds);
    }
    if !world
        .tile(x, y - 1)
        .is_some_and(|t| t.active() && t.kind == 79)
    {
        return Err(BedSpawnFailure::Gone);
    }
    for i in x - 1..=x + 1 {
        for j in y - 3..y {
            if let Some(t) = world.tile(i, j) {
                // `CheckSpawn`'s `tileSolid[379]` window (`Player.cs:54535`,
                // the frame toggle `Main.cs:9468`/`:17753`/`:17799`): the
                // candelabra is solid for the whole check — the generated
                // `TILE_SOLID` already carries that base state.
                if (t.active() || t.has_any_liquid())
                    && td::TILE_SOLID.has(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
                {
                    return Err(BedSpawnFailure::Obstructed);
                }
            }
        }
    }
    match tr_core::housing::check_room(world, x, y - 1) {
        Ok(_) => Ok(()),
        Err(reason) => Err(BedSpawnFailure::Room(reason)),
    }
}

/// The `CheckSpawn` failure kinds — the chat wrapper maps them to the
/// `Game.Bed*` texts (`Player.cs:54566`/`:54573-54587`).
pub(crate) enum BedSpawnFailure {
    Bounds,
    Gone,
    Obstructed,
    Room(tr_core::housing::RoomFailure),
}

/// `Player.CheckSpawn` (`Player.cs:54533-54590`): the bounds (the C# checks
/// `y` against `maxTilesX` — mirrored), the bed tile above the spawn cell,
/// the 3x3 area above free of solid non-solid-top tiles, and the room check.
/// The failure messages ride the chat like `Main.NewText`.
pub(crate) fn check_bed_spawn(
    world: &tr_core::World,
    x: i32,
    y: i32,
    chat: &mut crate::chat::ChatMonitor,
) -> bool {
    match bed_spawn_valid(world, x, y) {
        Ok(()) => true,
        Err(BedSpawnFailure::Obstructed) => {
            chat.new_text("The bed is obstructed.", crate::chat::COLOR_SERVER_MESSAGE);
            false
        }
        Err(BedSpawnFailure::Room(reason)) => {
            let text = match reason {
                tr_core::housing::RoomFailure::TooCloseToWorldEdge => {
                    "This bed is too close to the edge of the world."
                }
                tr_core::housing::RoomFailure::RoomIsTooBig => "This room is too big.",
                tr_core::housing::RoomFailure::RoomIsTooSmall => "This room is too small.",
                tr_core::housing::RoomFailure::HoleInWallIsTooBig => {
                    "This room has holes in its walls."
                }
                tr_core::housing::RoomFailure::TooManyUnsafeWalls => {
                    "This room has too many unsafe walls."
                }
                _ => "This room is not valid housing.",
            };
            chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
            false
        }
        Err(_) => false,
    }
}

/// The entity half of `Collision.EmptyTile(i, j, ignoreTiles: true)`
/// (`Collision.cs:1359-1378`): the cell rectangle must not intersect any
/// player or NPC rectangle (XNA `Rectangle.Intersects` semantics — strict
/// overlap, touching edges are free). The port keeps no dead/ghost flags for
/// the single local player. Free of `App` so the gate is unit-testable
/// without the app harness.
fn cell_blocked_by_entity(player: Option<&Player>, npcs: &[Npc], x: i32, y: i32) -> bool {
    let (x0, y0) = (x as f32 * TILE_PX, y as f32 * TILE_PX);
    let (x1, y1) = (x0 + TILE_PX, y0 + TILE_PX);
    if let Some(p) = player {
        if x0 < p.x + crate::player::WIDTH
            && x1 > p.x
            && y0 < p.y + crate::player::HEIGHT
            && y1 > p.y
        {
            return true;
        }
    }
    npcs.iter().any(|n| {
        let (w, h) = n.kind.size();
        x0 < n.x + w && x1 > n.x && y0 < n.y + h && y1 > n.y
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemSlot;
    use glam::Vec2;
    use tr_core::tile::{FLAG_ACTIVE, Tile};
    use tr_core::World;

    use super::super::tests::{object_world, set_tile};

    /// The bed rig: a floor row at y 60 with a style-0 bed placed at
    /// (40, 59) — columns 39-42, rows 58-59 (`place_item_tile`'s y is the
    /// bottom row) — and the player parked near the sleep anchor.
    fn bed_app() -> App {
        let mut w = World::new("t".into(), 80, 200);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        for x in 0..80 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        assert!(edit::place_item_tile(&mut w, 40, 59, 79, 0, -1));
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        // The sleep anchor for the clicked top-half cell (41, 58)
        // (`GetSleepingTargetInfo`): column `41 - 36/18 + 1 = 40`, row `59`
        // → `(648, 960)` — the player's bottom starts 17.9 px from the tile
        // centre, inside the 96-px gate.
        player.x = 638.0;
        player.y = 918.0;
        app.player = Some(player);
        app
    }

    #[test]
    fn the_bed_sleep_half_snaps_within_96_px_and_toggles_off() {
        // DW-1.3/DW-1.6 — `Player.cs:32669-32677`: the 96-px gate wraps
        // `sleeping.StartSleeping`; the same-anchor re-click wakes
        // (`PlayerSleepingHelper.cs:167-171`); the pose rotation arms
        // (`:73-74`).
        let mut app = bed_app();
        assert!(interact_tile(&mut app, 41, 58));
        let p = app.player.as_ref().unwrap();
        assert!(p.sleeping);
        assert_eq!(p.facing, -1);
        assert_eq!(p.sleeping_style, 0);
        assert!((p.full_rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // The spawn cell stayed untouched (the sleep half is not the spawn
        // half, `IsHoveringOverABottomSideOfABed`).
        assert_eq!(app.world.bed_spawn, None);
        // The same anchor re-click wakes.
        assert!(interact_tile(&mut app, 41, 58));
        assert!(!app.player.as_ref().unwrap().sleeping);
    }

    #[test]
    fn the_bed_sleep_refuses_beyond_96_px_but_consumes_the_click() {
        // The gate at `Player.cs:32670-32676`: an out-of-range click still
        // sets `flag2 = true` (consumed, no fall-through to the place path)
        // but `StartSleeping` never runs.
        let mut app = bed_app();
        let p = app.player.as_mut().unwrap();
        p.x = 638.0 + 7.0 * 16.0; // ~130 px from the tile centre
        assert!(interact_tile(&mut app, 41, 58));
        assert!(!app.player.as_ref().unwrap().sleeping);
    }

    #[test]
    fn the_spawn_scan_counts_the_candelabra_solid() {
        // DW-1.4 — `CheckSpawn`'s `tileSolid[379]` window (`Player.cs:54535`,
        // the frame toggle `Main.cs:9468`/`:17753`/`:17799`): the candelabra
        // is solid for the whole check — one hovering inside the 3x3 box
        // above the spawn cell obstructs it (the generated `TILE_SOLID`
        // already carries that base state).
        let mut app = bed_app();
        // The spawn cell for the bottom-half click (40, 59): column
        // `39 + 2 = 41`, row `60`. The open-air room check fails on its own,
        // so the scan is isolated by its message.
        let mut chat = crate::chat::ChatMonitor::default();
        let _ = check_bed_spawn(&app.world, 41, 60, &mut chat);
        let open_air = chat.messages.last().unwrap().text.clone();
        // The candelabra at (40, 57) — inside the scan box (40..=42,
        // 57..=59), above the bed's top row.
        let i = app.world.index(40, 57);
        app.world.tiles[i] = Tile {
            kind: 379,
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        let mut chat = crate::chat::ChatMonitor::default();
        assert!(!check_bed_spawn(&app.world, 41, 60, &mut chat));
        assert_eq!(chat.messages.last().unwrap().text, "The bed is obstructed.");
        assert_ne!(open_air, "The bed is obstructed.");
    }

    // ---- Phase 4 (DW-4.1/DW-4.3): the interaction hook ---------------------

    #[test]
    fn test_DW_4_1_interaction_events_map_results_to_cues_and_drops() {
        use edit::TileInteraction;
        // `None` = no interaction: the caller falls through to the place path
        // with no cue and no drops.
        assert!(interaction_events(TileInteraction::None).is_none());

        let (cue, events) = interaction_events(TileInteraction::DoorOpened {
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 10,
                x: 10,
                y: 18,
                ..Default::default()
            }],
        })
        .unwrap();
        assert_eq!(cue, Some("Door_Opened")); // PlaySound(8), WorldGen.cs:37732
        assert_eq!(events, vec![(25, 10, 18)]); // GetDoorItem(0) = 25

        let (cue, events) = interaction_events(TileInteraction::DoorClosed {
            object_drops: vec![],
        })
        .unwrap();
        assert_eq!(cue, Some("Door_Closed")); // PlaySound(9), WorldGen.cs:31906
        assert!(events.is_empty());

        // The blocked path carries cleanup drops with no cue — `PlaySound`
        // sits behind every failure return in the C# bodies.
        let (cue, events) = interaction_events(TileInteraction::DoorBlocked {
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 14,
                x: 10,
                y: 17,
                ..Default::default()
            }],
        })
        .unwrap();
        assert_eq!(cue, None);
        assert_eq!(events, vec![(32, 10, 17)]);
    }

    #[test]
    fn dw_2_1_interaction_events_map_dial_results_to_cue() {
        use edit::TileInteraction;
        // The fired dial plays `SoundID.Item4` and spawns no drops — the
        // dial arms kill no tiles (`Player.cs:32642`/:32652).
        let (cue, events) =
            interaction_events(TileInteraction::DialStarted { sundial: true }).unwrap();
        assert_eq!(cue, Some("Item_4"));
        assert!(events.is_empty());
        let (cue, events) =
            interaction_events(TileInteraction::DialStarted { sundial: false }).unwrap();
        assert_eq!(cue, Some("Item_4"));
        assert!(events.is_empty());
        // The refused gate stays consumed-but-silent (`flag2 = true`,
        // `Player.cs:32637`; no `PlaySound` behind the `:32638`/:32648 gate).
        let (cue, events) = interaction_events(TileInteraction::DialRefused).unwrap();
        assert_eq!(cue, None);
        assert!(events.is_empty());
    }

    #[test]
    fn test_DW_4_3_cell_blocked_by_entity() {
        // The entity half of `Collision.EmptyTile(ignoreTiles: true)`
        // (`Collision.cs:1359-1378`): strict rectangle overlap against the
        // player and NPCs.
        let w = object_world();
        let mut player = Player::spawn(&w);
        player.x = 160.0;
        player.y = 304.0; // the 20x42 hitbox covers cell (10, 19)
        assert!(cell_blocked_by_entity(Some(&player), &[], 10, 19));
        assert!(!cell_blocked_by_entity(Some(&player), &[], 15, 19));
        assert!(!cell_blocked_by_entity(None, &[], 10, 19));

        // A slime spawned on the surface covers the cells its 30x24 body
        // intersects.
        let slime = Npc::spawn_slime(&w, 20).unwrap();
        assert!(cell_blocked_by_entity(
            None,
            std::slice::from_ref(&slime),
            20,
            19
        ));
        assert!(!cell_blocked_by_entity(
            None,
            std::slice::from_ref(&slime),
            5,
            19
        ));
    }

    /// The switch family (`Player.cs:32171-32179`): using a lever (132), a
    /// switch (136) or a timer (144) trips the wiring.
    #[test]
    fn using_a_switch_trips_the_wiring() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 136; // the switch tile
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        assert!(interact_tile(&mut app, 10, 19), "the click is consumed");
        // `Wiring.HitSwitch`'s switch arm (`:279-288`) flips the tile's
        // frame between 0 and 18.
        assert_eq!(app.world.tile_frame_y(10, 19), 18, "the switch flipped");
    }

    /// The lever (132) and the timer (144) take the same arm.
    #[test]
    fn the_lever_and_the_timer_take_the_same_arm() {
        for kind in [132u16, 144] {
            let mut app = flow_app(None, vec![], vec![]);
            app.world = object_world();
            let i = app.world.index(10, 19);
            app.world.tiles[i].kind = kind;
            app.world.tiles[i].flags |= FLAG_ACTIVE;
            assert!(interact_tile(&mut app, 10, 19), "kind {kind}");
        }
    }

    // ---- M3-STORE-1: the bank open toggle + the void-vault pickup -------

    #[test]
    fn test_store_1_the_piggy_right_click_toggles_the_bank() {
        // `Player.cs:33122-33138` + the shared open block (:33230-33252, the
        // M3-CHEST-INT unify routes it through `chest::toggle_open`): the
        // right-click opens chest -2 (the piggy), the re-click closes to -1;
        // the inventory opens with the bank (`OpenChest`'s
        // `Main.playerInventory = true`, `:33517`) and STAYS open on the
        // chest close — the C# close arm never touches `playerInventory`
        // (the world-chest panel's close behaves the same; the STORE-1
        // pair-close deviation is gone). The furniture interact tests
        // drive `interact_tile` directly (the route's press edge is latched
        // at `update_editing`'s head).
        let mut w = object_world();
        // The piggy rides ON a table: the table row at 20, the piggy at 19.
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_tile(&mut w, 10, 19, 29);
        set_tile(&mut w, 11, 19, 29);
        w.frame_important[29] = true;
        if let Some(t) = w.tile_mut(10, 19) {
            t.frame_x = 0;
        }
        if let Some(t) = w.tile_mut(11, 19) {
            t.frame_x = 18;
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        // The first click opens chest -2.
        assert!(interact_tile(&mut app, 10, 19));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -2, "the piggy opens chest -2");
        assert!(app.inventory_open, "the bank opens the inventory");
        assert_eq!(
            (p.chest_x, p.chest_y),
            (10, 19),
            "OpenChest sets the origin for the banks too (:33524-33525)"
        );
        // The re-click closes; the inventory stays open (C# 1:1).
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the re-click closes"
        );
        assert!(app.inventory_open, "the close keeps the inventory open");
        assert_eq!(
            (
                app.player.as_ref().unwrap().chest_x,
                app.player.as_ref().unwrap().chest_y
            ),
            (10, 19),
            "the close keeps chestX/Y (:33236)"
        );
    }

    /// A container tile with its sheet frame set (the 2×2 chest shape's
    /// corners — `chest.rs`'s `place_chest` test fixture).
    fn set_container_tile(w: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].frame_x = fx;
        w.tiles[i].frame_y = fy;
    }

    /// The world-position of tile `(x, y)`'s centre (the
    /// `chest::try_open_chest_at_cursor` route's `(px / TILE_PX).floor()`).
    fn world_click(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 * TILE_PX + 8.0, y as f32 * TILE_PX + 8.0)
    }

    /// DW-1.1 — opening a bank closes an open world chest (`Player.chest`,
    /// `Player.cs:2876`, the M3-CHEST-INT unify): the world chest opened by
    /// the right-click route (`chest::try_open_chest_at_cursor`, store index
    /// 0) is replaced when the bank tile's click runs the shared open block
    /// — the cross-close is free with one field and one open path.
    #[test]
    fn test_DW_1_1_opening_a_bank_closes_the_world_chest() {
        let mut w = object_world();
        // A world chest 2×2 at (30, 15) — the frame corners 0/18.
        set_container_tile(&mut w, 30, 15, 21, 0, 0);
        set_container_tile(&mut w, 31, 15, 21, 18, 0);
        set_container_tile(&mut w, 30, 16, 21, 0, 18);
        set_container_tile(&mut w, 31, 16, 21, 18, 18);
        // The piggy on its table row (the store-1 fixture shape).
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_container_tile(&mut w, 10, 19, 29, 0, 0);
        set_container_tile(&mut w, 11, 19, 29, 18, 0);
        w.frame_important[29] = true;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));

        // The world chest opens (store index 0, the lazy entry).
        assert!(crate::chest::try_open_chest_at_cursor(
            &mut app,
            world_click(30, 15)
        ));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, 0, "the world chest is open");
        assert_eq!((p.chest_x, p.chest_y), (30, 15));

        // The piggy's click runs the shared block through the bank arm: the
        // world chest is replaced by the bank — one field, no second open
        // container.
        assert!(interact_tile(&mut app, 10, 19));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -2, "the bank replaced the world chest");
        assert_eq!((p.chest_x, p.chest_y), (10, 19), "the origin moved too");
        assert!(app.inventory_open, "the bank opened with the inventory");
    }

    /// DW-1.1 — the reverse: opening a world chest closes an open bank.
    #[test]
    fn test_DW_1_1_opening_a_world_chest_closes_the_bank() {
        let mut w = object_world();
        set_container_tile(&mut w, 30, 15, 21, 0, 0);
        set_container_tile(&mut w, 31, 15, 21, 18, 0);
        set_container_tile(&mut w, 30, 16, 21, 0, 18);
        set_container_tile(&mut w, 31, 16, 21, 18, 18);
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_container_tile(&mut w, 10, 19, 29, 0, 0);
        set_container_tile(&mut w, 11, 19, 29, 18, 0);
        w.frame_important[29] = true;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));

        // The bank opens first.
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(app.player.as_ref().unwrap().chest, -2);
        // The world chest's right-column click: the origin decodes to
        // (30, 15), the bank closes (the shared block's `num76 != chest`
        // switch arm).
        assert!(crate::chest::try_open_chest_at_cursor(
            &mut app,
            world_click(31, 15)
        ));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, 0, "the world chest replaced the bank");
        assert_eq!((p.chest_x, p.chest_y), (30, 15));
    }

    /// DW-1.3 — the bank open/close/re-click path goes through
    /// `chest::toggle_open` (`Player.cs:33230-33252`, the shared block): the
    /// fresh open sets `chest`/`chestX`/`chestY` — the C# origin decode lands
    /// on the piggy's left column even when the RIGHT tile is clicked
    /// (`Player.cs:33060-33079`) — and the inventory opens with it; the
    /// re-click closes (the inventory stays open — the C# close arm never
    /// touches `playerInventory`); a second bank's click switches (the
    /// sound-12 arm). The sound ids themselves are pinned by
    /// `chest::tests::test_DW_1_3_toggle_open_returns_the_shared_block_sound_ids`
    /// (`toggle_open`'s return contract; this arm plays them through
    /// `play_menu_sound`).
    #[test]
    fn test_DW_1_3_bank_clicks_route_through_toggle_open() {
        let mut w = object_world();
        // The piggy on its table row.
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_container_tile(&mut w, 10, 19, 29, 0, 0);
        set_container_tile(&mut w, 11, 19, 29, 18, 0);
        w.frame_important[29] = true;
        // A safe (97) 2×2 elsewhere — the switch target.
        set_container_tile(&mut w, 20, 17, 97, 0, 0);
        set_container_tile(&mut w, 21, 17, 97, 18, 0);
        set_container_tile(&mut w, 20, 18, 97, 0, 18);
        set_container_tile(&mut w, 21, 18, 97, 18, 18);
        w.frame_important[97] = true;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));

        // The RIGHT piggy tile's click: the origin decodes to (10, 19).
        assert!(interact_tile(&mut app, 11, 19));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -2, "the piggy opens chest -2");
        assert_eq!(
            (p.chest_x, p.chest_y),
            (10, 19),
            "the C# origin decode (:33060-33079)"
        );
        assert!(
            app.inventory_open,
            "OpenChest's playerInventory = true (:33517)"
        );
        // The re-click closes; the inventory stays open.
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the re-click closes"
        );
        assert!(app.inventory_open, "the close keeps the inventory open");
        // A fresh piggy open, then the safe's click switches.
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(app.player.as_ref().unwrap().chest, -2);
        assert!(interact_tile(&mut app, 20, 17));
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -3,
            "the switch opens the safe"
        );
    }

    #[test]
    fn the_golem_altar_consumes_a_cell_and_summons() {
        // `TileInteractionsUse`'s altar arm (`Player.cs:32755-32790`) +
        // `NPC.SpawnOnPlayer`'s case-245 column scan (`NPC.cs:80475-80531`):
        // the Power Cell 1293 in the inventory with hardmode and Plantera
        // down pays one cell and puts the Golem at the altar column's
        // `(up + up + down) / 3`; a missing gate refuses free.
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = tr_core::World::new("t".into(), 64, 200);
        for x in 0..64 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        // The altar's origin cell (237, frame 18/0) with a ceiling 10
        // above and a floor 6 below for the solid scans.
        for &(ax, ay, kind, fx, fy) in &[
            (30, 30, 237u16, 18i16, 0i16),
            (30, 20, 1, 0, 0),
            (30, 36, 1, 0, 0),
        ] {
            let i = w.index(ax, ay);
            w.tiles[i] = tr_core::Tile {
                kind,
                frame_x: fx,
                frame_y: fy,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        app.world = w;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 30.0 * 16.0;
        player.y = 30.0 * 16.0;
        app.player = Some(player);
        app.world.hard_mode = true;
        app.world.downed_plant_boss = true;
        // No cell: the click is consumed but nothing spawns.
        assert!(
            interact_tile(&mut app, 30, 30),
            "the altar consumes the click"
        );
        assert!(
            app.npcs.iter().all(|n| n.kind != NpcKind::Golem),
            "the missing cell refuses"
        );
        // The cell: one paid, the Golem at the (20 + 20 + 36)/3 row.
        app.inventory[app.selected] = ItemSlot {
            kind: 1293,
            name: "Lihzahrd Power Cell".into(),
            count: 2,
            prefix: 0,
        };
        assert!(interact_tile(&mut app, 30, 30));
        assert_eq!(app.inventory[app.selected].count, 1, "one cell paid");
        let golem = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Golem)
            .expect("the Golem spawns");
        assert_eq!(golem.x, 30.0 * 16.0 + 8.0, "the altar column");
        assert_eq!(golem.y, 25.0 * 16.0, "the (20+20+36)/3 row");
        // A living Golem refuses the re-summon; hardmode off refuses too.
        interact_tile(&mut app, 30, 30);
        assert_eq!(app.inventory[app.selected].count, 1, "the living-boss gate");
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::Golem {
                n.hp = 0;
            }
        }
        app.world.hard_mode = false;
        app.inventory[app.selected].count = 2;
        interact_tile(&mut app, 30, 30);
        assert_eq!(
            app.inventory[app.selected].count, 2,
            "the hardmode gate refuses free"
        );
        assert!(
            app.npcs
                .iter()
                .all(|n| n.kind != NpcKind::Golem || n.hp == 0),
            "the hardmode gate refuses"
        );
    }

    /// W4-d8 DOC-SWEEP (5) — the firework fountain's right-click arm
    /// (`Player.cs:32355-32377`, WRITTEN NOT RUN): the anchor shifts up on
    /// the bottom row (frameY 18), the dedupe blocks a second fountain on
    /// the same anchor, the spawn rolls `419 + Next(4)` with ai0/ai1 = the
    /// anchor tile, damage 0, zero velocity.
    #[test]
    fn w4_d8_the_fountain_right_click_spawns_and_dedupes() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 64, 200);
        // A 2×2 fountain at (30, 30)-(31, 31): the top row frameY 0, the
        // bottom row frameY 18 (the C# `frameY == 18` anchor shift).
        for &(fx, fy) in &[(0i16, 0i16), (18, 0), (0, 18), (18, 18)] {
            let x = 30 + (fx / 18) as i32;
            let y = 30 + (fy / 18) as i32;
            let i = w.index(x, y);
            w.tiles[i] = Tile { kind: 338, frame_x: fx, frame_y: fy, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 30.0 * 16.0;
        player.y = 32.0 * 16.0;
        app.player = Some(player);
        // The click on the BOTTOM row (30, 31): the anchor shifts to 30.
        assert!(interact_tile(&mut app, 30, 31), "the fountain consumes the click");
        let shots: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(shots.len(), 1, "one fountain");
        let f = shots[0];
        assert!(
            crate::projectile::FIREWORK_FOUNTAINS.contains(&f.kind),
            "419 + Next(4), kind {}",
            f.kind
        );
        assert_eq!((f.ai[0], f.ai[1]), (30.0, 30.0), "the shifted anchor");
        assert_eq!(f.damage, 0, "the tile-interaction spawn passes Damage 0");
        assert_eq!(f.vel, Vec2::ZERO, "the fountain sits still");
        assert_eq!(f.time_left, 600, "the NewProjectile seed (`:11033-11048`)");
        // The dedupe: a second click on the SAME anchor spawns nothing.
        assert!(interact_tile(&mut app, 30, 30));
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "the anchor dedupe (`:32363-32371`)"
        );
        // A click on the TOP row (31, 30) hits the same anchor (frameY 0,
        // no shift) — still deduped.
        assert!(interact_tile(&mut app, 31, 30));
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "the top row shares the anchor"
        );
        // Kill the fountain: the next click spawns a fresh one.
        for p in app.projectiles.slots_mut() {
            if p.active {
                p.active = false;
                p.time_left = 0;
            }
        }
        assert!(interact_tile(&mut app, 30, 31));
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "the dead anchor frees the spawn"
        );
    }
}
