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
        // The inventory scan (`Player.cs:32766-32778`): the C# walks
        // `inventory[0..58]` — the grid + the coin/ammo rows (the trash 58
        // is OUTSIDE this loop, unlike the chest key's `<= 58`). The first
        // 1293 row decrements; a zero stack clears (`SetDefaults(0)`).
        // W4-g43 doc-sweep: the old grid-only scan missed the coin/ammo rows.
        for slot in app
            .inventory
            .iter_mut()
            .chain(app.equip.coins.iter_mut())
            .chain(app.equip.ammo.iter_mut())
        {
            if slot.kind == 1293 && slot.count > 0 {
                slot.count -= 1;
                if slot.count == 0 {
                    *slot = crate::item::EMPTY_SLOT;
                }
                // `:32782` — the roar BEFORE the summon (the C# order).
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
/// active/!solidTop/!halfBrick/slope-0/!inActive shape — an out-of-bounds
/// cell reads solid, the C# `null` case), then the Golem lands at the
/// column's `(up + up + down) / 3` pixel Y, targeting the player, with the
/// "HasAwoken" announcement (`ChatColors.BossOrEvent`). Returns false when
/// no altar is in range (the C#'s try/catch no-spawn path).
/// W4-g43 doc-sweep fidelity fixes vs the C#:
/// - the scan bounds are EXCLUSIVE-upper (`j < num2 + 20`, `:80494/:80496`
///   — the old `..=` included the +20 row/col);
/// - the `num4 <= 0 || num5 <= 0` refusal (`:80505`) — no altar found keeps
///   the 0/0 init and refuses, and an altar ON row/col 0 refuses too;
/// - `NPC.NewNPC` takes the spawn point as the BOTTOM centre (`:80770`,
///   `nPC.Bottom = (X, Y)` → `position = (X - w/2, Y - h)`): the old
///   top-left write sat the body 140 px too high;
/// - `target = plr` + `targetSetFrame` (`:80528-80529`);
/// - the announcement is `Announcement.HasAwoken` formatted with the NPC
///   name (`:80530`), `ChatColors.BossOrEvent` — not the server-message
///   "The Golem has awoken!" literal.
fn spawn_golem_on_altar(app: &mut App) -> bool {
    let Some(player) = app.player.as_ref() else {
        return false;
    };
    let (pcx, pcy) = player.center();
    let cx_t = (pcx / TILE_PX) as i32;
    let cy_t = (pcy / TILE_PX) as i32;
    // `:80492-80504` — the last-match scan (the C# keeps overwriting
    // num4/num5, so the LAST matching cell in row-major order wins).
    let mut altar: Option<(i32, i32)> = None;
    for j in (cx_t - 20)..(cx_t + 20) {
        for k in (cy_t - 20)..(cy_t + 20) {
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
    // `:80505-80508` — the zero-coordinate refusal.
    if ax <= 0 || ay <= 0 {
        return false;
    }
    // `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`): the null arm reads
    // TRUE; the active arm also refuses actuated tiles (`!inActive()`).
    let solid = |j: i32, k: i32| -> bool {
        match app.world.tile(j, k) {
            None => true,
            Some(t) => {
                t.active()
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                    && !t.half_brick()
                    && t.slope() == 0
                    && !t.inactive()
            }
        }
    };
    // `:80509-80526` — both scans start at the altar row; the up-scan's
    // exclusive lower bound is `num5 - 50` (the C# `num8 > num5 - 50`).
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
    // `:80527` — NewNPC(X: num4*16+8, Y: (num6+num6+num7)/3*16): the BOTTOM
    // centre (`:80770`), so the top-left comes off half the box / the height.
    let spawn_x = ax as f32 * TILE_PX + 8.0;
    let spawn_y = ((up + up + down) / 3) as f32 * TILE_PX;
    let (gw, gh) = NpcKind::Golem.size();
    let mut golem = Npc::new(NpcKind::Golem, spawn_x - gw * 0.5, spawn_y - gh, 1, 0.0);
    golem.hp = golem.max_hp;
    // `:80528-80529` — the target latch (the solo player is slot 0; the
    // EverLastingTicker stand-in is the port's frame counter convention).
    golem.target = app.local_id as i32;
    golem.target_set_frame = app.frame_count;
    app.npcs.push(golem);
    // `:80530` — `Announcement.HasAwoken` with the NPC's net name,
    // `ChatColors.BossOrEvent` (the WoF precedent's literal stand-in shape,
    // but the correct colour + the localized template).
    let text = crate::lang::text_args(
        "Announcement.HasAwoken",
        &[crate::lang::npc_name(245)],
    );
    crate::chat::broadcast_death_text(&mut app.chat, &text, crate::chat::COLOR_BOSS_OR_EVENT);
    true
}

/// The locked-door key payment (`Player.cs:32796-32838`, W4-g43 doc-sweep):
/// the inventory walk (grid 0-49 + coins 50-53 + ammo 54-57 — the C#'s
/// `inventory[0..58]`, the trash slot 58 is OUTSIDE this scan) pays the
/// first Temple Key 1141 stack and unlocks; only when NO inventory key
/// unlocked (`!flag9`, `:32818`) does the void-bag fallback walk bank4
/// (`:32820-32836`). `WorldGen.UnlockDoor` (`WorldGen.cs:37599-37628`)
/// shifts the three rows' frames off the locked window; its
/// `SoundEngine.PlaySound(22, i*16, num*16+16)` (`:37614`, the anchor row +
/// 16 px) plays here and `NotifyProgressionEvent(22)` (`:37627`,
/// TEMPLE_RAIDER — `AchievementInitializer.cs:112-114`) fires once per
/// unlock. The 12 dusts (`:37622-37625`) are the unported dust layer.
///
/// Temple-FINAL keep-one (the c36 co-land): the W4-c36 `ec832b440` +
/// `5f4032d02` `try_unlock_temple_door`/app-level `unlock_door` pair is the
/// duplicate half — this arm is the superset (the shared
/// `edit::is_locked_door` predicate over the inline frame window, the
/// `edit::unlock_door` Option-anchor refusal gate so the cue/progression
/// fire only on a real shift, the `world_key` clear, the next-click-open
/// witness). Both feed the SAME `notify_progression_net(22)` seam and the
/// registry's `Cond::Progression(22)` TEMPLE_RAIDER row
/// (`achievements_data.rs`, `AchievementInitializer.cs:112-114`) — one
/// receipt, no double-wire: the c36 pair drops at the merge.
fn unlock_temple_door(app: &mut App, tx: i32, ty: i32) {
    const TEMPLE_KEY: u16 = 1141; // ItemID.cs:3965
    let mut unlocked = false;
    // `:32800-32817` — the inventory scan (the chest key walk's layout:
    // the grid, then the coin/ammo rows; `break` on the first paid key).
    for slot in app
        .inventory
        .iter_mut()
        .chain(app.equip.coins.iter_mut())
        .chain(app.equip.ammo.iter_mut())
    {
        if slot.kind != TEMPLE_KEY || slot.count == 0 {
            continue;
        }
        slot.count -= 1;
        if slot.count == 0 {
            *slot = crate::item::EMPTY_SLOT;
        }
        if let Some(anchor) = edit::unlock_door(&mut app.world, tx, ty) {
            play_door_unlock(app, tx, anchor);
        }
        unlocked = true;
        break;
    }
    // `:32818-32837` — the void-bag fallback: `!flag9 && useVoidBag()`.
    if !unlocked && crate::chest::use_void_bag(app) {
        let Some(p) = app.player.as_mut() else {
            return;
        };
        for slot in p.banks[3].iter_mut() {
            if slot.kind != TEMPLE_KEY || slot.count == 0 {
                continue;
            }
            slot.count -= 1;
            if slot.count == 0 {
                *slot = crate::item::EMPTY_SLOT;
            }
            if let Some(anchor) = edit::unlock_door(&mut app.world, tx, ty) {
                play_door_unlock(app, tx, anchor);
            }
            break;
        }
    }
}

/// The unlock's cue + progression (`WorldGen.cs:37614`/:37627): the
/// `SoundID.Unlock` (22) at the anchor row's `(i*16, num*16+16)` and
/// `NotifyProgressionEvent(22)` — TEMPLE_RAIDER.
fn play_door_unlock(app: &mut App, tx: i32, anchor: i32) {
    app.audio.play_at(
        "Unlock",
        tx as f32 * TILE_PX,
        anchor as f32 * TILE_PX + 16.0,
        1.0,
    );
    let _ = app.notify_progression_net(22);
    app.world_key = None;
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
    // The locked-door key arm (`TileInteractionsUse`'s `type == 10` branch,
    // `Player.cs:32793-32838`, W4-g43 doc-sweep): `WorldGen.IsLockedDoor`
    // (`WorldGen.cs:69191-69197`) matches the golden/Lihzahrd locked frames
    // (tile 10, frameY 594-646, frameX < 54 — the temple door's style 11
    // rides 594/612/630). The 58-slot inventory scan (`:32800-32817`, the
    // grid + coins + ammo, NOT the trash) pays one Temple Key 1141 and
    // calls `WorldGen.UnlockDoor`; the void-bag fallback (`:32818-32837`,
    // `useVoidBag`) walks bank4 the same way. The click is consumed whether
    // or not a key was found (`flag2 = true`, `:32795`) — the door NEVER
    // opens on a locked click; the next click opens it (`OpenDoor`'s own
    // `IsLockedDoor` refusal, `:37654-37657`). The Plantera gate is the
    // KEY's drop (the Temple Key drops from Plantera, `loot_db_gen.rs`'s
    // 262 row), not a runtime check here.
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && edit::is_locked_door(t))
    {
        unlock_temple_door(app, tx, ty);
        return true;
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
            if let Some(p) = app.player.as_mut() {
                if within_snapping_range(p, tx, ty, 40.0) {
                    p.sit_down(&app.world, *seat_x, *seat_y, *direction, *seat_offset);
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
        edit::TileInteraction::PartyMonolithToggle => {
            // `Player.cs:32278-32284`: `PlaySound(28, myX * 16, myY * 16, 0)`
            // — the mech cue, the same case-28 path the switch clicks ride
            // (`LegacySoundPlayer.cs:997-1007`'s one `Next(-10, 11) * 0.01`
            // pitch draw behind the playing guard). The tile use runs in
            // `Player.ItemCheck` — the `UpdatePlayers` stream
            // (`Main.cs:17754`).
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            app.audio
                .play_switch_click(tx as f32 * TILE_PX, ty as f32 * TILE_PX, rng);
            // `BirthdayParty.ToggleManualParty` (`BirthdayParty.cs:202-217`):
            // `ManualParty = !ManualParty` (SP — the `netMode == 1` send arm
            // is unported); the up-state edge fires `CheckForAchievement` —
            // THROW_A_PARTY (25) credits only when the party came up (the
            // wiring drain's identical body, `world_sim/wiring.rs`).
            let was_up = crate::world_events::party_is_up(&app.world_events);
            app.world_events.manual_party = !app.world_events.manual_party;
            if !was_up && crate::world_events::party_is_up(&app.world_events) {
                let _ = app.notify_progression_net(25);
            }
            app.world_key = None;
            return true;
        }
        // W4-f41 final sweep (research/70 §1f follow-up): the cake (621)
        // right-click rides W4-f34's shared `StationBuff` arm below, with
        // the faithful pitched case-2 play folded into the 192 row
        // (`play_npc_item_cue(2, …)` — `SoundID.Item2 = LegacySoundStyle(2,
        // 2)`, `Player.cs:32604`); the other six rows keep the e28-recorded
        // unpitched stand-in.
        edit::TileInteraction::StationBuff {
            buff_type,
            buff_time,
        } => {
            // W4-e28 + W4-f34 — the seven station-buff arms
            // (`Player.cs:32594-32635`): `AddBuff(id, time)` THEN the cue at
            // `base.position` — the PLAYER's position, not the tile's (the
            // generic cue mapping below plays at the cell, so this arm owns
            // its own play). The click is consumed (`flag2 = true`). The
            // cue per arm: 29/348/150 → Item4 (:32598/:32610/:32628),
            // 192 → Item2 (:32604), 366 → Item179 (:32616, the style's own
            // `WithVolume(0.6f)`, `SoundID.cs:1563`), 159 → Item37 (:32622,
            // the case-2 ×0.5 volume tweak, `LegacySoundPlayer.cs:564-565`),
            // 93 → Item149 (:32634).
            if let Some(p) = app.player.as_mut() {
                p.buffs.add_buff(*buff_type, *buff_time);
                if *buff_type == 192 {
                    // The cake row plays the FAITHFUL case-2 path (research/
                    // 70 §1f's recorded follow-up, landed as its own noted
                    // change): the `GetRandomPitch` NextFloat on the
                    // audio-only stream, the attenuation gate, then ONE
                    // `Main.rand.Next(-6, 7) * 0.01` pitch draw
                    // (`LegacySoundPlayer.cs:632`) on the `UpdatePlayers`
                    // stream — the tile use runs in `Player.ItemCheck`
                    // (`Main.cs:17754`). The façade truncates the position
                    // (`SoundEngine.cs:88-91`).
                    let (px, py) = (p.x, p.y);
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    app.audio.play_npc_item_cue(2, px, py, rng);
                } else {
                    // The other six rows keep the e28-recorded unpitched
                    // stand-in (no case-2 pitch roll); the STATIC volume
                    // halves ride the volume_scale argument.
                    let (cue, volume) = match *buff_type {
                        366 => ("Item_179", 0.6),
                        159 => ("Item_37", 0.5),
                        93 => ("Item_149", 1.0),
                        _ => ("Item_4", 1.0), // 29 / 348 / 150
                    };
                    app.audio.play_at(cue, p.x, p.y, volume);
                }
            }
            app.world_key = None;
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
        // The party monolith is consumed in `interact_tile` ahead of this
        // mapping (the mech cue + ToggleManualParty ride the world events)
        // — no drops. The station-buff grants (incl. the cake 621) are
        // consumed there too (the buff + cue are app-side).
        edit::TileInteraction::PartyMonolithToggle => (None, Vec::new()),
        edit::TileInteraction::StationBuff { .. } => (None, Vec::new()),
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

    /// W4-e28/f34 — the station rig: one active station tile at (10, 19) on
    /// the object world, the player at spawn (in earshot of the tile).
    fn station_app(kind: u16) -> App {
        let mut w = object_world();
        set_tile(&mut w, 10, 19, kind);
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let player = Player::spawn(&app.world);
        app.player = Some(player);
        app
    }

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

    /// W4-f41 — the party monolith's right-click (`Player.cs:32278-32284`):
    /// the click is consumed, the mech cue plays (`PlaySound(28)`), and
    /// `BirthdayParty.ToggleManualParty` flips the manual party — the
    /// up-edge credits THROW_A_PARTY (25), the down-edge credits nothing
    /// (`BirthdayParty.cs:202-217` + `:167-173`, the wiring drain's body).
    #[test]
    fn w4_f41_the_party_monolith_click_toggles_the_party() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        let i = app.world.index(10, 19);
        app.world.tiles[i].kind = 455;
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        let cues0 = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 19), "the click is consumed");
        assert!(app.world_events.manual_party, "the toggle raised the party");
        assert!(
            app.audio.played_count() > cues0,
            "the mech cue played (:32281)"
        );
        // The up-edge credit (the wiring lane's progression probe shape).
        assert!(
            app.achievements
                .list
                .iter()
                .flat_map(|a| a.conditions.iter())
                .any(|c| c.name == "PROGRESSION_EVENT_25" && c.completed),
            "a party coming up credits THROW_A_PARTY"
        );
        // The mirror: a party already up, the re-click takes it down and
        // the achievement stays as it was (no second credit row needed —
        // the down edge runs no `CheckForAchievement` credit).
        assert!(interact_tile(&mut app, 10, 19));
        assert!(!app.world_events.manual_party, "the toggle lowered it");
        // A genuine party up: the manual toggle still flips the manual
        // flag, but `PartyIsUp` never falls, so no state edge fires.
        app.world_events.genuine_party = true;
        assert!(interact_tile(&mut app, 10, 19));
        assert!(app.world_events.manual_party);
        assert!(crate::world_events::party_is_up(&app.world_events));
    }

    // ---- W4-e28 + W4-f34: the station-buff app arms ------------------------

    #[test]
    fn w4_e28_the_crystal_ball_grants_clairvoyance_and_plays_item4() {
        // `Player.cs:32594-32599`: the click on tile 125 grants buff 29 for
        // 108000 ticks (30 min) and plays `SoundID.Item4` at the PLAYER's
        // position. The click is consumed; the cell keeps its kind.
        let mut app = station_app(125);
        let played0 = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 19), "the click is consumed");
        let p = app.player.as_ref().unwrap();
        let slot = p.buffs.find_buff_index(29).expect("Clairvoyance granted");
        assert_eq!(p.buffs.buff_time[slot], 108000);
        assert_eq!(app.audio.played_count(), played0 + 1, "one cue played");
        assert_eq!(app.world.tile(10, 19).unwrap().kind, 125, "no world write");
    }

    #[test]
    fn w4_e28_the_ammo_box_grants_buff_93_and_plays_item149() {
        // `Player.cs:32630-32635`: tile 287 grants buff 93 for 108000 ticks
        // and plays `SoundID.Item149`. The buff's effect arm
        // (`:10078-10081`) sets `ammoBox` — the 1/5 not-consume roll's flag.
        let mut app = station_app(287);
        let played0 = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 19));
        let p = app.player.as_ref().unwrap();
        let slot = p.buffs.find_buff_index(93).expect("Ammo Box granted");
        assert_eq!(p.buffs.buff_time[slot], 108000);
        assert_eq!(app.audio.played_count(), played0 + 1);
    }

    #[test]
    fn w4_e28_the_station_grants_draw_no_rng() {
        // RNG-free verify for the UNPITCHED rows: the plain `play_at` path
        // draws no pitch roll — no stream moves. (The cake 621's faithful
        // case-2 play DOES draw; its own witness pins that below.)
        let mut app = station_app(125);
        let players0 = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let global0 = app.rngs.global().clone();
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            players0,
            "UpdatePlayers untouched"
        );
        assert_eq!(*app.rngs.global(), global0, "global untouched");
        // The re-click refreshes the time, still no draws (the AddBuff
        // update-in-place path, `Player.cs:5402-5426`).
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            players0
        );
        let p = app.player.as_ref().unwrap();
        let slot = p.buffs.find_buff_index(29).unwrap();
        assert_eq!(p.buffs.buff_time[slot], 108000, "refreshed, not stacked");
    }

    #[test]
    fn w4_f34_the_cake_war_table_and_dead_cells_station_grant_their_buffs() {
        // `Player.cs:32600-32617`: tile 621 grants buff 192 (Sugar Rush) for
        // 7200 ticks + Item2; tile 464 grants 348 (War Table) for 108000 +
        // Item4; tile 699 grants 366 for 108000 + Item179. Each consumes the
        // click, plays ONE cue at the player position and writes no world.
        for (kind, buff, time) in [(621u16, 192, 7_200), (464, 348, 108_000), (699, 366, 108_000)]
        {
            let mut app = station_app(kind);
            let played0 = app.audio.played_count();
            assert!(interact_tile(&mut app, 10, 19), "tile {kind} consumed");
            let p = app.player.as_ref().unwrap();
            let slot = p.buffs.find_buff_index(buff).expect("the grant");
            assert_eq!(p.buffs.buff_time[slot], time, "tile {kind}'s duration");
            assert_eq!(app.audio.played_count(), played0 + 1, "one cue");
            assert_eq!(app.world.tile(10, 19).unwrap().kind, kind, "no world write");
        }
    }

    #[test]
    fn w4_f34_the_sharpening_and_bewitching_tables_grant_their_buffs() {
        // `Player.cs:32618-32629`: tile 377 grants buff 159 (Sharpened) for
        // 108000 ticks + Item37 (the case-2 ×0.5 volume rides the arm);
        // tile 354 grants 150 (Bewitched) for 108000 + Item4.
        for (kind, buff) in [(377u16, 159), (354u16, 150)] {
            let mut app = station_app(kind);
            let played0 = app.audio.played_count();
            assert!(interact_tile(&mut app, 10, 19), "tile {kind} consumed");
            let p = app.player.as_ref().unwrap();
            let slot = p.buffs.find_buff_index(buff).expect("the grant");
            assert_eq!(p.buffs.buff_time[slot], 108000);
            assert_eq!(app.audio.played_count(), played0 + 1, "one cue");
        }
    }

    /// W4-f34's neighbour no-draw witness, RE-DERIVED for the pitched-cue
    /// follow-up (research/70 §1f): the cake 621 left the RNG-free loop —
    /// its faithful case-2 play draws one pitch roll per audible click (the
    /// `w4_f41_the_cake_click_draws_the_pitched_case2_roll` witness pins
    /// that). The unpitched rows keep the e28 contract: no stream moves,
    /// the re-click refreshes in place, never stacks.
    #[test]
    fn w4_f34_the_neighbour_grants_draw_no_rng_and_refresh_not_stack() {
        for (kind, buff, time) in [(699u16, 366, 108_000), (464, 348, 108_000)] {
            let mut app = station_app(kind);
            let players0 = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
            let global0 = app.rngs.global().clone();
            assert!(interact_tile(&mut app, 10, 19));
            assert!(interact_tile(&mut app, 10, 19), "the re-click");
            assert_eq!(
                *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                players0,
                "tile {kind}: UpdatePlayers untouched"
            );
            assert_eq!(*app.rngs.global(), global0, "tile {kind}: global untouched");
            let p = app.player.as_ref().unwrap();
            let slot = p.buffs.find_buff_index(buff).unwrap();
            assert_eq!(p.buffs.buff_time[slot], time, "refreshed, not stacked");
        }
    }

    /// W4-f41 final sweep (research/70 §1f follow-up): the cake's faithful
    /// pitched case-2 play — ONE `Next(-6, 7) * 0.01` draw on the
    /// `UpdatePlayers` stream per audible click (`LegacySoundPlayer.cs:632`
    /// via `play_npc_item_cue(2, …)`), the global stream untouched, and the
    /// re-click refreshes 7200 in place (never stacks) while drawing again.
    #[test]
    fn w4_f41_the_cake_click_draws_the_pitched_case2_roll() {
        let mut app = station_app(621);
        let mut witness = app
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone();
        let _ = witness.next_range(-6, 7); // the ONE case-2 pitch draw
        let global0 = app.rngs.global().clone();
        let played0 = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).peek(),
            witness.peek(),
            "exactly one UpdatePlayers draw — the pitched case-2 roll"
        );
        assert_eq!(*app.rngs.global(), global0, "global untouched");
        assert_eq!(app.audio.played_count(), played0 + 1, "the Item_2 cue");
        // The re-click refreshes in place (`Player.cs:5402-5426`) and draws
        // the pitch roll again — the faithful path is not draw-free.
        let _ = witness.next_range(-6, 7);
        assert!(interact_tile(&mut app, 10, 19), "the re-click");
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).peek(),
            witness.peek()
        );
        let p = app.player.as_ref().unwrap();
        let slot = p.buffs.find_buff_index(192).unwrap();
        assert_eq!(p.buffs.buff_time[slot], 7_200, "refreshed, not stacked");
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
        // W4-g43 doc-sweep: NewNPC's BOTTOM-centre anchor (`NPC.cs:80770`) —
        // the body's top-left is (X - 140/2, Y - 140): the altar column's
        // centre 488 minus 70, the (20+20+36)/3 row's 400 minus 140.
        assert_eq!(golem.x, 30.0 * 16.0 + 8.0 - 70.0, "the altar column, bottom-centre");
        assert_eq!(golem.y, 25.0 * 16.0 - 140.0, "the (20+20+36)/3 row, bottom-centre");
        assert_eq!(golem.target, 0, "the summon targets the player (:80528)");
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

    /// W4-g43 doc-sweep — the altar's cell scan covers the coin/ammo rows
    /// (`Player.cs:32766`'s `inventory[0..58]`, the port's grid + coins +
    /// ammo): a Power Cell parked in an ammo slot pays exactly like a grid
    /// slot. The trash slot is OUTSIDE the C# loop (`n < 58`).
    #[test]
    fn the_altar_cell_scan_covers_the_ammo_rows() {
        let mut app = temple_door_app();
        // Reuse the door rig's world: plant the altar + the solid scans.
        for &(ax, ay, kind, fx, fy) in &[
            (30, 30, 237u16, 18i16, 0i16),
            (30, 20, 1, 0, 0),
            (30, 36, 1, 0, 0),
        ] {
            let i = app.world.index(ax, ay);
            app.world.tiles[i] = Tile {
                kind,
                frame_x: fx,
                frame_y: fy,
                ..Default::default()
            };
            app.world.tiles[i].flags |= FLAG_ACTIVE;
        }
        {
            let p = app.player.as_mut().unwrap();
            p.x = 30.0 * 16.0;
            p.y = 30.0 * 16.0;
        }
        app.world.hard_mode = true;
        app.world.downed_plant_boss = true;
        app.equip.ammo[0] = ItemSlot {
            kind: 1293,
            name: "Lihzahrd Power Cell".into(),
            count: 1,
            prefix: 0,
        };
        assert!(interact_tile(&mut app, 30, 30));
        assert_eq!(app.equip.ammo[0].count, 0, "the ammo-row cell paid");
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Golem && n.hp > 0),
            "the Golem spawns off the ammo-row cell"
        );
    }

    /// The temple-door rig: a Lihzahrd door (tile 10, style 11 — the locked
    /// frames 594/612/630, frameX 0) at column 10, rows 19-21, over a stone
    /// floor at 22 with a ceiling at 18 (the `open_door` anchor shape).
    fn temple_door_app() -> App {
        let mut w = World::new("t".into(), 80, 200);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        for x in 0..80 {
            for y in [18, 22] {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for (row, fy) in [(19, 594i16), (20, 612), (21, 630)] {
            let i = w.index(10, row);
            w.tiles[i] = Tile {
                kind: 10,
                frame_x: 0,
                frame_y: fy,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 8.0 * 16.0;
        player.y = 19.0 * 16.0;
        app.player = Some(player);
        app
    }

    /// W4-g43 doc-sweep — the Temple Key 1141 unlock (`Player.cs:32796-
    /// 32838` + `WorldGen.UnlockDoor` `WorldGen.cs:37599-37628`): the locked
    /// click pays one key, shifts the three rows +54 (594/612/630 →
    /// 648/666/684 — `IsLockedDoor` stops matching) and NEVER opens the
    /// door on the same click; the next click opens it. No key: the click
    /// is consumed free and the door stays locked.
    #[test]
    fn the_temple_key_unlocks_the_door_then_the_next_click_opens_it() {
        let mut app = temple_door_app();
        // No key: consumed, locked, no open, no TEMPLE_RAIDER credit.
        assert!(interact_tile(&mut app, 10, 20), "the locked click consumes");
        assert_eq!(app.world.tile(10, 19).unwrap().frame_y, 594, "still locked");
        assert_eq!(app.world.tile(10, 20).unwrap().kind, 10, "no open");
        assert!(
            !app.achievements.list.iter().any(|a| a.name == "TEMPLE_RAIDER" && a.completed),
            "no key, no unlock, no progression 22"
        );
        // One key in the inventory: paid, the frames shift, still closed.
        app.inventory[0] = ItemSlot {
            kind: 1141,
            name: "Temple Key".into(),
            count: 1,
            prefix: 0,
        };
        assert!(interact_tile(&mut app, 10, 20));
        assert_eq!(app.inventory[0].count, 0, "one key paid");
        assert_eq!(app.inventory[0].kind, 0, "the emptied slot turns to air");
        for (row, fy) in [(19, 648i16), (20, 666), (21, 684)] {
            assert_eq!(
                app.world.tile(10, row).unwrap().frame_y,
                fy,
                "the +54 shift, row {row}"
            );
            assert_eq!(app.world.tile(10, row).unwrap().kind, 10, "still closed");
        }
        // The keep-one receipt (the c36 co-land): progression 22 completes
        // the registry's TEMPLE_RAIDER row (`Cond::Progression(22)`,
        // `AchievementInitializer.cs:112-114`) — the SAME seam the c36
        // `try_unlock_temple_door` pair feeds, one credit not two.
        assert!(
            app.achievements.list.iter().any(|a| a.name == "TEMPLE_RAIDER" && a.completed),
            "the unlock fires NotifyProgressionEvent(22) → TEMPLE_RAIDER"
        );
        // The next click opens the door (the player faces right, the open
        // column lands at 10-11 — the entity gate is free of the player at
        // x 8*16 .. 8*16+20, the column 10 starts at 160 > 148).
        assert!(interact_tile(&mut app, 10, 20));
        assert_eq!(
            app.world.tile(10, 19).unwrap().kind,
            11,
            "the unlocked door opens on the NEXT click"
        );
    }

    /// W4-g43 — the void-bag fallback (`Player.cs:32818-32837`): no
    /// inventory key, but the Void Bag 4131 carried and a key in bank4 —
    /// the bank walk pays it and unlocks. Without the bag the bank key is
    /// never read.
    #[test]
    fn the_void_bag_holds_the_fallback_temple_key() {
        let mut app = temple_door_app();
        {
            let p = app.player.as_mut().unwrap();
            p.banks[3][0] = ItemSlot {
                kind: 1141,
                name: "Temple Key".into(),
                count: 1,
                prefix: 0,
            };
        }
        // No Void Bag: the bank key is invisible, the door stays locked.
        assert!(interact_tile(&mut app, 10, 20));
        assert_eq!(app.world.tile(10, 19).unwrap().frame_y, 594);
        assert_eq!(app.player.as_ref().unwrap().banks[3][0].count, 1);
        // The Void Bag 4131 in the inventory opens the fallback.
        app.inventory[1] = ItemSlot {
            kind: 4131,
            name: "Void Bag".into(),
            count: 1,
            prefix: 0,
        };
        assert!(interact_tile(&mut app, 10, 20));
        assert_eq!(app.world.tile(10, 19).unwrap().frame_y, 648, "unlocked");
        assert_eq!(
            app.player.as_ref().unwrap().banks[3][0].count,
            0,
            "the bank key paid"
        );
        // The fallback fires the SAME progression 22 receipt (the c36
        // co-land's keep-one seam — one TEMPLE_RAIDER credit).
        assert!(
            app.achievements.list.iter().any(|a| a.name == "TEMPLE_RAIDER" && a.completed),
            "the void-bag unlock fires progression 22 too"
        );
    }
}
