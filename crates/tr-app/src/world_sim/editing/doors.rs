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

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, edit};

use crate::app::App;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use crate::registry;

use super::super::items::spawn_item_drop_px;

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
                // W4-e10 — `Player.cs:32782`: `PlaySound(15,
                // (int)position.X, (int)position.Y, 0)` — case 15 slot 0
                // (the playing guard, no draws), at the player's
                // TOP-LEFT, BEFORE `NPC.SpawnOnPlayer(245)` (`:32785`).
                let (plx, ply) = app
                    .player
                    .as_ref()
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0.0, 0.0));
                app.audio.play_roar(0, plx, ply);
                // `Player.cs:32782` — `PlaySound(15, (int)position.X,
                // (int)position.Y, 0)`: Style 0 → Roar_0, top-left,
                // draw-free (case 15's slot guard rides play_roar).
                let (rx, ry) =
                    app.player.as_ref().map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));
                app.audio.play_roar(0, rx, ry);
                spawn_golem_on_altar(app);
                break;
            }
        }
    }
    true
}

/// The golden-door key walk (`Player.cs:32798-32837`): the 58-slot
/// inventory scan for the Golden Key 1141 (`:32800-32817`) — the first
/// stack decrements, an emptied slot turns to air, `WorldGen.UnlockDoor`
/// shifts the frames and the Unlock cue plays at the door's middle row
/// (`WorldGen.cs:37614`, `PlaySound(22, i * 16, num * 16 + 16)` — case 22
/// draws NOTHING, `LegacySoundPlayer.cs:937-946`), then the progression
/// event 22 (`:37627`). The scan breaks on the first key (`:32815`). With
/// no inventory key and the Void Bag open, the bank4 walk mirrors it
/// (`:32818-32836`). The 4-per-row dust 11 (`:37622-37625`) is the visual
/// seam (no dust engine on this path).
fn unlock_golden_door(app: &mut App, tx: i32, ty: i32) {
    const GOLDEN_KEY: u16 = 1141; // `Player.cs:32798`
    let mut unlocked = false;
    // `:32800` — `for (num48 = 0; num48 < 58; num48++)`: the inventory grid
    // (0-49) + coins (50-53) + ammo (54-57); the trash slot 58 is outside.
    for slot in app
        .inventory
        .iter_mut()
        .chain(app.equip.coins.iter_mut())
        .chain(app.equip.ammo.iter_mut())
    {
        if slot.kind != GOLDEN_KEY || slot.count == 0 {
            continue;
        }
        slot.count -= 1;
        if slot.count == 0 {
            *slot = crate::item::EMPTY_SLOT;
        }
        fire_unlock_door(app, tx, ty);
        unlocked = true;
        break; // `:32815`
    }
    // `:32818-32836` — the void-bag fallback (`useVoidBag()`).
    if !unlocked && crate::chest::use_void_bag(app) {
        if let Some(p) = app.player.as_mut() {
            for slot in p.banks[3].iter_mut() {
                if slot.kind != GOLDEN_KEY || slot.count == 0 {
                    continue;
                }
                slot.count -= 1;
                if slot.count == 0 {
                    *slot = crate::item::EMPTY_SLOT;
                }
                fire_unlock_door(app, tx, ty);
                break;
            }
        }
    }
}

/// One `WorldGen.UnlockDoor` call + its cue/progression tail
/// (`WorldGen.cs:37614`/:37627). The Unlock play is draw-free (case 22)
/// and 2D-attenuated at the door's middle row — the plain `play_at` path
/// (the `Chest.Unlock` precedent, `chest.rs:1061`).
fn fire_unlock_door(app: &mut App, tx: i32, ty: i32) {
    let Some(top) = edit::unlock_door(&mut app.world, tx, ty) else {
        return;
    };
    app.audio.play_at(
        "Unlock",
        tx as f32 * TILE_PX,
        (top + 1) as f32 * TILE_PX,
        1.0,
    );
    let _ = app.notify_progression_net(22); // `:37627`
    app.world_key = None;
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

/// W4-c36 DOC-SWEEP — the locked Lihzahrd door's key pay + unlock
/// (`Player.cs:32798-32834` + `WorldGen.UnlockDoor` `WorldGen.cs:37599-37627`):
/// the inventory scan (grid + coins + ammo — the C# `for < 58`, no trash
/// slot) pays one Temple Key 1141, then `UnlockDoor` walks up to the door's
/// frameY-594 top cell, plays `SoundID.Unlock` (22) at `(i*16, top*16+16)`,
/// shifts the three locked cells' frameY by +54 (the unlocked frames) and
/// fires progression 22 → TEMPLE_RAIDER. The `!flag9 && useVoidBag()`
/// fallback scans bank4 with the same pay/unlock shape. The dust burst
/// (4× Dust 11 per cell, `:37621-37624`) has no port layer here — the
/// standing tile-edit dust deviation; the MP `52` client send (`:32812`)
/// rides the same unported door-sync gap as the chest key walks.
fn try_unlock_temple_door(app: &mut App, tx: i32, ty: i32) {
    // `:32799-32813` — the inventory key scan. The pay lands first, the
    // unlock after the slot borrow releases (`unlock_door` takes the app).
    let mut paid = false;
    for slot in app
        .inventory
        .iter_mut()
        .chain(app.equip.coins.iter_mut())
        .chain(app.equip.ammo.iter_mut())
    {
        if slot.kind == 1141 && slot.count > 0 {
            slot.count -= 1;
            if slot.count == 0 {
                *slot = crate::item::EMPTY_SLOT;
            }
            paid = true;
            break;
        }
    }
    if paid {
        unlock_door(app, tx, ty);
        return;
    }
    // `:32816-32834` — the void-bag bank4 fallback.
    if crate::chest::use_void_bag(app) {
        let mut vault_paid = false;
        if let Some(p) = app.player.as_mut() {
            for slot in p.banks[3].iter_mut() {
                if slot.kind == 1141 && slot.count > 0 {
                    slot.count -= 1;
                    if slot.count == 0 {
                        *slot = crate::item::EMPTY_SLOT;
                    }
                    vault_paid = true;
                    break;
                }
            }
        }
        if vault_paid {
            unlock_door(app, tx, ty);
        }
    }
}

/// `WorldGen.UnlockDoor` (`WorldGen.cs:37599-37627`): the frameY-594 top
/// walk (a cell above 594 or the world edge aborts), the sound-22 cue, the
/// +54 frameY shift on the door's three cells and progression 22.
fn unlock_door(app: &mut App, i: i32, j: i32) {
    let mut num = j;
    loop {
        let Some(t) = app.world.tile(i, num) else { return };
        if t.frame_y == 594 {
            break;
        }
        num -= 1;
        let Some(t) = app.world.tile(i, num) else { return };
        if t.frame_y < 594 || num <= 0 {
            return;
        }
    }
    // `:37613` — `PlaySound(22, i * 16, num * 16 + 16)`.
    app.audio
        .play_at("Unlock", i as f32 * 16.0, num as f32 * 16.0 + 16.0, 1.0);
    for k in num..=num + 2 {
        if let Some(t) = app.world.tile_mut(i, k) {
            t.frame_y += 54;
        }
        app.world.mark_chunk(i, k);
    }
    app.world.revision = app.world.revision.wrapping_add(1);
    // `:37627` — TEMPLE_RAIDER.
    let _ = app.notify_progression_net(22);
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
    // W4-e10 gap census — the sign arm (`TileInteractionsUse`'s
    // `Main.tileSign[type]` branch, `Player.cs:32693-32723`): a re-click
    // on the OPEN sign closes it WITH the cue (`CloseSign()` `:32699`,
    // the `:33537` site); otherwise single-player opens locally
    // (`OpenSign(Sign.ReadSign)` `:32706` — create-if-missing, the
    // `:33547` Menu_Open) and the MP client asks the host with `46`
    // (`:32720`, the origin decode `:32710-32718` is `signs::origin`'s
    // frame probe). The arm consumes the click either way (`flag2`).
    // Sign kinds are disjoint from the furniture dispatch below, so the
    // branch sits ahead of `interact_at` like the altar's. The C# gates
    // on the CLICKED tile's kind (`Main.tileSign[type]` `:32693`) — the
    // origin probe alone could decode a non-sign cell onto a sign.
    let clicked_sign = app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && matches!(t.kind, 55 | 85 | 425 | 573));
    if clicked_sign {
        if let Some((sx, sy)) = crate::signs::origin(&app.world, tx, ty) {
            let current = app.player.as_ref().map_or(-1, |p| p.sign);
            let same = app
                .world
                .signs
                .iter()
                .position(|r| r.x == sx && r.y == sy)
                .is_some_and(|i| i as i32 == current);
            if same {
                crate::signs::close_sign(app, false); // `:32699`
            } else if let crate::app::NetRole::Client(client) = &app.net {
                client.send(&crate::net::C2s::SignRequest(crate::net::SignRequest {
                    x: sx as i16,
                    y: sy as i16,
                })); // `:32720` — the host answers `47`, the client arm opens
            } else if let Some(idx) = crate::signs::find_or_create(&mut app.world.signs, sx, sy) {
                crate::signs::open_sign(app, idx as i32); // `:32706`
            }
        }
    // The `Main.tileSign` arm (`TileInteractionsUse`, `Player.cs:32693-
    // 32724`, W4-f22): the re-click closes the open sign, single player
    // opens the record (sound 10 + the dialog), the MP client asks with
    // `46`. Every sign tile consumes the click. The arm sits between the
    // bed (`:32654-32692`) and the clock (`:32725`) in the C# chain — the
    // sign kinds have no registry def, so it reads the kind literals like
    // the altar does.
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && matches!(t.kind, 55 | 85 | 425 | 573))
    {
        return crate::signs::try_open_sign_at_cursor(
            app,
            glam::Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0),
        );
    }
    // W4-c36 DOC-SWEEP — the locked Lihzahrd door arm (`TileInteractionsUse`'s
    // `type == 10` branch, `Player.cs:32792-32836`): `WorldGen.IsLockedDoor`
    // (`:69191-69197`, tile 10, frameY 594-646, frameX < 54) + a Temple Key
    // 1141 in the 58-slot inventory scan (`:32799-32813`) — the key pays,
    // `WorldGen.UnlockDoor` (`:37599-37627`) shifts the three locked cells
    // down 54 frameY and fires progression 22 (TEMPLE_RAIDER). The void-bag
    // fallback (`:32816-32834`) scans bank4 with the same shape. The C#'s
    // `flag2 = true` consumes the click either way; the port's dispatcher
    // would answer `DoorBlocked` (open_door refuses the locked frames), so
    // the arm runs BEFORE `interact_at` and consumes on a locked door.
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && t.kind == 10 && (594..=646).contains(&t.frame_y) && t.frame_x < 54)
    {
        try_unlock_temple_door(app, tx, ty);
    // The pylon arm (`TileInteractionsUse`'s `type == 597` branch,
    // `Player.cs:32180-32212`): the right-click opens the fullscreen map
    // centred on the pylon network and selects the clicked pylon for the
    // network-line draw (`Main.MapPylonTile`). The anchor decodes the 3x4
    // body (`x - frameX/18%3, y - frameY/18%4`); the view fits every
    // revealed pylon (`pylon_view_fit`). The arm runs before the furniture
    // dispatch — the pylon has no `interact_at` interaction of its own.
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && t.kind == 597)
    {
        if app.settings.map_enabled {
            let tile = app.world.tile(tx, ty).copied().unwrap_or_default();
            let anchor = (
                tx - (tile.frame_x as i32 / 18 % 3),
                ty - (tile.frame_y as i32 / 18 % 4),
            );
            // `TryOpeningFullscreenMap` (`Player.cs:29103-29118`): the
            // inventory/sign/talk clears + sound 10 + the 2.5 scale + the
            // recentre ride the union helper (research/84 §10.3 step 2 —
            // the f29 tip's inline TryOpening clears dedupe into
            // `try_open_fullscreen_map`; `CloseSign(quiet)` has no port
            // counterpart, the sign editor is unported). The pylon-view
            // fit below then overrides the recentre.
            app.try_open_fullscreen_map();
            // recentre; `set_open(true)` already does the scale/recentre/
            // pylon-tile clear. `SetTalkNPC(-1)` + `npcChatCornerItem = 0`
            // close the chatter; `CloseSign(quiet)` has no port counterpart
            // (the sign editor is unported).
            app.inventory_open = false;
            app.talk_npc = None;
            app.npc_chat_text = String::new();
            app.close_shop();
            app.audio.play("Menu_Open"); // `SoundEngine.PlaySound(10)`
            let team_seed = app
                .world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::TEAM_BASED_SPAWNS);
            let pylons = tr_core::pylons::scan_pylons(&app.world);
            let revealed: Vec<(i32, i32)> = pylons
                .iter()
                .filter(|p| {
                    !team_seed
                        || app
                            .map
                            .as_ref()
                            .is_some_and(|m| m.tile_revealed(&app.world, p.x as i32, p.y as i32))
                })
                .map(|p| (p.x as i32, p.y as i32))
                .collect();
            let (sw, sh) = app.screen_size();
            let screen = glam::Vec2::new(sw, sh);
            if let Some(m) = app.map.as_mut() {
                m.set_open(true);
                m.pylon_tile = anchor;
                let (pos, scale) =
                    crate::map::pylon_view_fit(anchor, &revealed, screen, m.scale);
                m.pos = pos;
                m.scale = scale;
                // The fit overrides the helper's recentre arm: disarm it
                // and keep the pan clear (`recentre` already cleared it).
                m.pan_target = None;
                m.reset_pos = false;
            }
        }
        return true;
    }
    let direction = app.player.as_ref().map(|p| p.facing).unwrap_or(1);
    // W4-g24 — the golden-door unlock arm (`TileInteractionsUse`'s locked
    // type-10 branch, `Player.cs:32793-32835`): a locked door with a Golden
    // Key 1141 in the 58-slot inventory scan consumes one key and runs
    // `WorldGen.UnlockDoor` (`:32810`); the void-bag fallback (`:32818-32835`)
    // scans bank4 when no inventory key fired. The arm consumes the click
    // (`flag2 = true` at `:32795`) whether or not a key was found. The
    // `SendData(52)` client mirror (`:32813`) has no port wire flavour —
    // the local frame shift is the truth (the wire-tools precedent).
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && edit::is_locked_door(t))
    {
        unlock_golden_door(app, tx, ty);
        return true;
    }
    // W4-g24 — the trapdoor/tall-gate arms (`TileInteractionsUse`'s
    // 386/387 branch `Player.cs:32290-32303` and the 388/389 branch
    // `:32304-32312`): both consume the click (`flag2 = true`) before the
    // shift attempt. The trapdoor tries the player's side first, then the
    // mirror on a refuse (`:32294-32298`); the tall gate's `closing` is the
    // OPEN kind (389 → close). The cues ride the UpdatePlayers stream (the
    // tile use runs in `Player.ItemCheck`, `Main.cs:17754`); the pitched
    // case-8/9 draws happen only after the attenuation gate.
    let door_kind = app.world.tile(tx, ty).filter(|t| t.active()).map(|t| t.kind);
    if matches!(door_kind, Some(386 | 387)) {
        let (player, npcs) = (&app.player, &app.npcs);
        let blocked = |x: i32, y: i32| cell_blocked_by_entity(player.as_ref(), npcs, x, y);
        let above = app
            .player
            .as_ref()
            .is_some_and(|p| (ty as f32 * TILE_PX) > p.center().1);
        let (res1, cue1) =
            edit::shift_trapdoor(&mut app.world, tx, ty, above, -1, &blocked);
        let (res, cue) = if res1.shifted {
            (res1, cue1)
        } else {
            // `:32295-32298` — the mirror attempt with the flipped side.
            // The C# passes `myX, myY` (the ORIGINAL click) to both calls;
            // each `GetTopLeftAndStyles` re-snaps from there.
            let (res2, cue2) =
                edit::shift_trapdoor(&mut app.world, tx, ty, !above, -1, &blocked);
            if res2.shifted {
                // The first call refused (its fall-through 9 already played
                // below); the second shifted — its 8 plays instead.
                if let Some((cx, cy)) = res1.refuse_cue {
                    app.audio.play_door_sound(
                        false,
                        cx as f32,
                        cy as f32,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                }
                (res2, cue2)
            } else {
                // Both refused: the C# plays the fall-through 9 inside EACH
                // call (`:51594`) — two Door_Closed plays, in call order.
                if let Some((cx, cy)) = res1.refuse_cue {
                    app.audio.play_door_sound(
                        false,
                        cx as f32,
                        cy as f32,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                }
                (res2, cue2)
            }
        };
        if let Some((cx, cy)) = cue {
            // `:51470`/:51512/:51563 — the pitched 8 at the arm's position.
            app.audio.play_door_sound(
                true,
                cx as f32,
                cy as f32,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
        } else if let Some((cx, cy)) = res.refuse_cue {
            // `:51594` — the (second) fall-through refuse plays the pitched 9.
            app.audio.play_door_sound(
                false,
                cx as f32,
                cy as f32,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
        }
        spawn_shift_drops(app, &res.object_drops);
        app.world_key = None;
        return true;
    }
    if matches!(door_kind, Some(388 | 389)) {
        let closing = door_kind == Some(389); // `:32307` — flag4
        let (player, npcs) = (&app.player, &app.npcs);
        let blocked = |x: i32, y: i32| cell_blocked_by_entity(player.as_ref(), npcs, x, y);
        let (res, cue) =
            edit::shift_tall_gate(&mut app.world, tx, ty, closing, false, &blocked);
        if let Some((cx, cy)) = cue {
            // `:51762` — the pitched 8 (both directions play the OPEN cue,
            // the C# has one play site).
            app.audio.play_door_sound(
                true,
                cx as f32,
                cy as f32,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
        }
        spawn_shift_drops(app, &res.object_drops);
        app.world_key = None;
        return true;
    }
    // The entity half of `Collision.EmptyTile` (`Collision.cs:1359`): the
    // closing column must be free of the player and NPCs. Disjoint field
    // borrows — the world write happens inside `interact_at`.
    let (player, npcs) = (&app.player, &app.npcs);
    let cell_blocked = |x: i32, y: i32| cell_blocked_by_entity(player.as_ref(), npcs, x, y);
    // W4-g83: the door ops' cleanup kills' framing dispatch rides the
    // UpdatePlayers stream (the active genRand of the player tick).
    let result = edit::interact_at(
        &mut app.world,
        tx,
        ty,
        direction,
        &cell_blocked,
        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
    );
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
        // the shop close, the world route's mapping (chest.rs:525);
        // W4-e10: the prologue's `CloseSign()` (`:33087`, quiet=false).
        crate::signs::close_sign(app, false);
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
            // the click is consumed. The §8.1 stamp first.
            crate::world_sim::wiring::stamp_wiring_entity_boxes(app);
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
            // W4-c42 — the wired door cues ride the same UpdatePlayers
            // stream (the C# plays inside the trip's WorldGen body).
            for (open, px, py) in app.wiring.take_door_cues() {
                app.audio.play_door_sound(
                    open,
                    px as f32,
                    py as f32,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // The §8.2 echoes: SP discards (the netMode-0 return).
            for (n, x, y, d) in app.wiring.take_door_echoes() {
                app.send_door_toggle(n, x, y, d);
            }
            app.send_hit_switch(*x, *y); // `:32178`/`:32226`/`:32340`
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::MonolithToggle { x, y } => {
            // `Player.cs:32271-32277` (W4-c42 final — the g78 research/76
            // contested-seam request): `flag2 = true` + `PlaySound(28,
            // myX*16, myY*16, 0)` + `GamepadEnableGrappleCooldown()` +
            // `WorldGen.SwitchMonolith(myX, myY)`. The Mech-28 pitch draw
            // rides the UpdatePlayers stream (the tile-use precedent);
            // the flip runs the kept wired body with `running == false`
            // (the use path's SkipWire tail is inert).
            app.audio.play_switch_click(
                *x as f32 * TILE_PX,
                *y as f32 * TILE_PX,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            if let Some(p) = app.player.as_mut() {
                p.gamepad_enable_grapple_cooldown(); // `:32275`
            }
            app.wiring.use_switch_monolith(&mut app.world, *x, *y);
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
            // 93 → Item149 (:32634). The plain `play_at` path keeps the
            // e28-recorded unpitched stand-in (no case-2 pitch roll); the
            // STATIC volume halves ride the volume_scale argument.
            let (cue, volume) = match *buff_type {
                192 => ("Item_2", 1.0),
                366 => ("Item_179", 0.6),
                159 => ("Item_37", 0.5),
                93 => ("Item_149", 1.0),
                _ => ("Item_4", 1.0), // 29 / 348 / 150
            };
            if let Some(p) = app.player.as_mut() {
                p.buffs.add_buff(*buff_type, *buff_time);
                app.audio.play_at(cue, p.x, p.y, volume);
            }
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::CandleToggled => {
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::RightClickToggle { x, y, kind } => {
            // W4-e10 — the five right-click toggle arms
            // (`Player.cs:32228-32284`): ALL play `PlaySound(28,
            // myX * 16, myY * 16, 0)` at the CLICKED cell's top-left
            // BEFORE the toggle — the Mech-28 seam (case 28,
            // `LegacySoundPlayer.cs:995-1009`: the slot-0 playing guard,
            // then ONE `Next(-10, 11) * 0.01` pitch draw on
            // `UpdatePlayers` — the tile use runs in `Player.ItemCheck`,
            // `Main.cs:17754`).
            app.audio.play_switch_click(
                *x as f32 * TILE_PX,
                *y as f32 * TILE_PX,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            let frame = app
                .world
                .tile(*x, *y)
                .map(|t| (t.frame_x, t.frame_y))
                .unwrap_or((0, 0));
            match kind {
                // `:32232`: `WorldGen.SwitchMB` — the 2×2 frameX flip.
                139 => app.wiring.switch_mb(&mut app.world, *x, *y, frame.0, frame.1),
                // `:32269`: `WorldGen.SwitchFountain` — the 2×4 frameY flip.
                207 => {
                    app.wiring
                        .switch_fountain(&mut app.world, *x, *y, frame.0, frame.1)
                }
                // `:32276`: `WorldGen.SwitchMonolith` (the
                // `GamepadEnableGrappleCooldown` at `:32275` is a gamepad
                // input seam — no port).
                410 | 480 | 509 | 657 | 658 | 720 | 721 | 725 | 733 => {
                    app.wiring
                        .switch_monolith(&mut app.world, *x, *y, *kind, frame.0, frame.1)
                }
                // `:32283`: `BirthdayParty.ToggleManualParty`
                // (`BirthdayParty.cs:145-163`): the flag flip, then the
                // up-state edge credits THROW_A_PARTY (25) — the same
                // request the wiring case-455 arm drains.
                455 => {
                    let was_up = crate::world_events::party_is_up(&app.world_events);
                    app.world_events.manual_party = !app.world_events.manual_party;
                    if !was_up && crate::world_events::party_is_up(&app.world_events) {
                        let _ = app.achievements.notify_progression(25);
                    }
                }
                // The campfire's frameY flip already ran in the core
                // dispatch (`edit::interact_at`'s inline arm).
                _ => {}
            }
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::BuffStation { kind } => {
            // W4-e10 — the buff-station family (`Player.cs:32594-32634`): the
            // seven right-click buff tiles. Each arm sets `flag2 = true`,
            // grants its buff for the fixed duration (`AddBuff`), THEN plays
            // its `SoundID.ItemNN` style at `base.position` (the player's
            // TOP-LEFT, not the tile). The buff→style pair is the C# chain's
            // own (the style index is `soundid::SOUND_STYLES`' `ItemNN` row).
            // The interaction runs in `Player.ItemCheck` — the `UpdatePlayers`
            // stream (`Main.cs:17754`); the style façade's `GetRandomPitch`
            // NextFloat is audio-only, the case-2 pitch draw rides the stream
            // after the attenuation gate (the dial's Item4 seam, `:32642`).
            let (buff, time, style_name) = match kind {
                125 => (29, 108000, "Item4"),   // Crystal Ball — Clairvoyance
                621 => (192, 7200, "Item2"),    // Slice of Cake — Sugar Rush
                464 => (348, 108000, "Item4"),  // War Table
                699 => (366, 108000, "Item179"),// Dead Cells Potion Station
                377 => (159, 108000, "Item37"), // Sharpening Station
                354 => (150, 108000, "Item4"),  // Bewitching Table
                287 => (93, 108000, "Item149"), // Ammo Box
                _ => unreachable!("the dispatch only yields the seven station kinds"),
            };
            let (plx, ply) = app
                .player
                .as_ref()
                .map(|p| (p.x, p.y))
                .unwrap_or((0.0, 0.0));
            if let Some(p) = app.player.as_mut() {
                p.buffs.add_buff(buff, time); // `:32597`/`:32603`/… the AddBuff
            }
            if let Some(idx) = crate::soundid::SOUND_STYLES
                .iter()
                .position(|s| s.name == style_name)
            {
                app.audio.play_use_sound(
                    idx as u16,
                    plx,
                    ply,
                    0.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            app.world_key = None;
            return true;
        }
        // W4-c39 exec (research/114 §1h): the CakeEat app arm DROPPED —
        // f34's shared `StationBuff` arm is the survivor (the same unpitched
        // Item_2 stand-in; f41's pitched seam stays research/70 §1f's
        // recorded follow-up). f41c `f1f95be4` is canonical.
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
    // W4-c28: the door `19 ToggleDoorState` send decision
    // (`Player.cs:32844`/`:32851` open, `:32861` close). The fired open/close
    // arms queue the send (the blocked arm sends nothing — the C# gates the
    // send behind the `type != 10` / `CloseDoor` success). The `19` id parks
    // until d90's `DOOR_TOGGLE` codec merges; the decision is recorded here.
    // `direction` is the player's facing (the open-retry `-direction` nuance
    // rides the d90 wire's decode); the trapdoor/tall-gate arms are unported.
    match cue {
        Some("Door_Opened") => app.send_door_toggle(0, tx, ty, direction),
        Some("Door_Closed") => app.send_door_toggle(1, tx, ty, direction),
        _ => {}
    }
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
            // W4-e10 — the dials' `SoundID.Item4` (`Player.cs:32642`/
            // `:32652`) is a STYLE play at `base.position` (the player's
            // TOP-LEFT, not the tile): the wave-A façade — the
            // `GetRandomPitch` NextFloat audio-only, then case 2's one
            // `Next(-6, 7) * 0.01` pitch draw on `UpdatePlayers` after
            // the gate. The tile-corner `play_at` stand-in is retired.
            "Item_4" => {
                let (plx, ply) = app
                    .player
                    .as_ref()
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0.0, 0.0));
                if let Some(idx) = crate::soundid::SOUND_STYLES
                    .iter()
                    .position(|s| s.name == "Item4")
                {
                    app.audio.play_use_sound(idx as u16, plx, ply, 0.0, rng);
                }
            }
            _ => {
                // The dial's `SoundID.Item4` (`Player.cs:32642`/:32651 —
                // the style façade at `base.position`, the TOP-LEFT): the
                // GetRandomPitch NextFloat rides the audio-only stream,
                // case 2's pitch draw rides `UpdatePlayers`.
                if cue == "Item_4" {
                    if let Some(idx) = crate::soundid::SOUND_STYLES
                        .iter()
                        .position(|s| s.sound_id == 2 && s.style == 4)
                    {
                        let (rx, ry) = app
                            .player
                            .as_ref()
                            .map(|p| (p.x, p.y))
                            .unwrap_or((tx as f32 * TILE_PX, ty as f32 * TILE_PX));
                        app.audio.play_use_sound(idx as u16, rx, ry, 0.0, rng);
                    }
                } else {
                    app.audio
                        .play_at(cue, tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
                }
            }
        }
    }
    for (id, px, py) in drops {
        // The tile-interaction drops ride the `UpdatePlayers` stream (the
        // interaction runs in `Player.ItemCheck`, `Main.cs:17754`). The
        // positions are PIXELS — the C# `Item.NewItem` rect centre (the g78
        // FIX-ONCE, `ObjectDrop::spawn_center_px`).
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        spawn_item_drop_px(&mut app.items, rng, id, 1, Vec2::new(px, py));
    }
    app.world_key = None;
    true
}

/// The shift arms' cleanup drops (`validate_tile_square`'s refusal kills,
/// the closing row's cut/drip kills) — the `interaction_events` shape: the
/// registry tile_drop at the record cell, the UpdatePlayers stream.
fn spawn_shift_drops(app: &mut App, drops: &[edit::ObjectDrop]) {
    for d in drops {
        if let Some(item) = registry::tile_drop(d.tile_kind, 0) {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            spawn_item_drop(&mut app.items, rng, item, 1, d.x, d.y);
        }
    }
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
/// while the click still counts as consumed. The drop positions are PIXELS
/// — the C# `Item.NewItem` rect centre (`ObjectDrop::spawn_center_px`, the
/// g78 FIX-ONCE).
fn interaction_events(
    result: edit::TileInteraction,
) -> Option<(Option<&'static str>, Vec<(u16, f32, f32)>)> {
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
        // The right-click toggle family (the Mech-28 click + the wiring
        // halves) is consumed in `interact_tile` ahead of this mapping.
        edit::TileInteraction::RightClickToggle { .. } => (None, Vec::new()),
        // The switch trip is consumed in `interact_tile` ahead of this
        // mapping — no cue, no drops.
        edit::TileInteraction::HitSwitch { .. } => (None, Vec::new()),
        // The monolith toggle is consumed in `interact_tile` ahead of this
        // mapping (its Mech-28 plays there) — no cue, no drops.
        edit::TileInteraction::MonolithToggle { .. } => (None, Vec::new()),
        edit::TileInteraction::ChairSit { .. } => (None, Vec::new()),
        edit::TileInteraction::BedInteract { .. } => (None, Vec::new()),
        edit::TileInteraction::BedSleep { .. } => (None, Vec::new()),
        edit::TileInteraction::ClockRead { .. } => (None, Vec::new()),
        // W4-e28: the station-buff grant is consumed in `interact_tile`
        // ahead of this mapping — its cue plays at the PLAYER's position
        // (`base.position`), not the tile's, so it never rides this table.
        edit::TileInteraction::StationBuff { .. } => (None, Vec::new()),
        // The bank open is consumed in `interact_tile` ahead of this
        // mapping (the SoundID 10/11/12 keys are absent from the cue set).
        edit::TileInteraction::BankOpen { .. } => (None, Vec::new()),
        // The station probe is consumed in `interact_tile` ahead of this
        // mapping — no cue, no drops (the C# plays the craft-open sound 10
        // inside `OpenCloseFilter`'s non-quiet path, `NewCraftingUI.cs:894`).
        edit::TileInteraction::StationInteract { .. } => (None, Vec::new()),
        // W4-e10 — the buff-station family is consumed in `interact_tile` ahead
        // of this mapping (the buff grant + the `SoundID.ItemNN` style play run
        // there, `Player.cs:32594-32634`); the arms kill no tiles, so no drops.
        edit::TileInteraction::BuffStation { .. } => (None, Vec::new()),
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
        .filter_map(|d| {
            registry::tile_drop(d.tile_kind, 0).map(|id| {
                let (px, py) = d.spawn_center_px();
                (id, px, py)
            })
        })
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

/// The shared entity gate for the NPC-side callers (the zombie bash's tall
/// gate, W4-g24) — the same `Collision.EmptyTile` half the interaction path
/// uses, re-exported so `world_sim/npcs.rs` need not duplicate the rect math.
pub(crate) fn cell_blocked_by_entity_pub(
    player: Option<&Player>,
    npcs: &[Npc],
    x: i32,
    y: i32,
) -> bool {
    cell_blocked_by_entity(player, npcs, x, y)
}

/// The REAL `TownDoorOps` (W4-g24 doc-sweep — the W4-b3 phase-2 seam): the
/// aiStyle-7 arm's inline door attempts (`npc_ai_town::town_npc_ai`,
/// `NPC.cs:54047-54073`/`:53838-53854`) mutate the world through
/// `tr_core::edit::{open_door, close_door, shift_tall_gate}` and play the
/// success cues — b3's phase-1 `NoOpTownDoors` (every attempt false) is
/// replaced by this adapter when the town lane adopts it. The cues ride
/// the `UpdateNpcs` stream: the C# plays them inside `NPC.UpdateNPC` under
/// the `Main.cs:17767` swap (the zombie bash drain's convention,
/// `world_sim/npcs.rs`). The cleanup drops spawn on the same stream (the
/// bash drain's shape).
// The production consumer is the b3 phase-2 town wiring (the aiStyle-7
// dispatch still routes through `update_guide` on this branch); the test
// module exercises the adapter in every build. The allow keeps the default
// build warning-clean until the town lane adopts it.
#[allow(dead_code)]
pub(crate) struct AppTownDoors<'a> {
    pub app: &'a mut App,
}

impl crate::npc_ai_town::TownDoorOps for AppTownDoors<'_> {
    /// `WorldGen.OpenDoor` (`WorldGen.cs:37630-37784`): the pitched 8 at
    /// `(i*16, j*16)` (`:37727`) on success only; the refusal is silent.
    fn open_door(&mut self, x: i32, y: i32, direction: i32) -> bool {
        let mut drops = Vec::new();
        let opened = edit::open_door(&mut self.app.world, x, y, direction, &mut drops);
        if opened {
            self.app.audio.play_door_sound(
                true,
                x as f32 * TILE_PX,
                y as f32 * TILE_PX,
                self.app.rngs.stream(crate::app::RngKey::UpdateNpcs),
            );
            self.app.world_key = None;
        }
        spawn_town_door_drops(self.app, &drops);
        opened
    }

    /// `WorldGen.CloseDoor` (`WorldGen.cs:31805-31910`): the pitched 9 at
    /// `(i*16, j*16)` (`:31908`) on success only. `forced = false` — the
    /// town arm calls the plain overload (`NPC.cs:53838`).
    fn close_door(&mut self, x: i32, y: i32) -> bool {
        let mut drops = Vec::new();
        let (player, npcs) = (&self.app.player, &self.app.npcs);
        let blocked = |bx: i32, by: i32| cell_blocked_by_entity(player.as_ref(), npcs, bx, by);
        let closed =
            edit::close_door(&mut self.app.world, x, y, false, &blocked, &mut drops);
        if closed {
            self.app.audio.play_door_sound(
                false,
                x as f32 * TILE_PX,
                y as f32 * TILE_PX,
                self.app.rngs.stream(crate::app::RngKey::UpdateNpcs),
            );
            self.app.world_key = None;
        }
        spawn_town_door_drops(self.app, &drops);
        closed
    }

    /// `WorldGen.ShiftTallGate` (`WorldGen.cs:51723-51775`, PORTED —
    /// W4-g24 batch 2): the pitched 8 at the returned cue position
    /// (`:51762`, both directions play the OPEN cue — one play site).
    /// `forced = false` — the town arm calls the plain overload
    /// (`NPC.cs:53850`/`:54065`).
    fn shift_tall_gate(&mut self, x: i32, y: i32, closing: bool) -> bool {
        let (player, npcs) = (&self.app.player, &self.app.npcs);
        let blocked = |bx: i32, by: i32| cell_blocked_by_entity(player.as_ref(), npcs, bx, by);
        let (res, cue) =
            edit::shift_tall_gate(&mut self.app.world, x, y, closing, false, &blocked);
        if let Some((cx, cy)) = cue {
            self.app.audio.play_door_sound(
                true,
                cx as f32,
                cy as f32,
                self.app.rngs.stream(crate::app::RngKey::UpdateNpcs),
            );
            self.app.world_key = None;
        }
        spawn_town_door_drops(self.app, &res.object_drops);
        res.shifted
    }
}

/// The town door attempts' cleanup drops — the bash drain's shape
/// (`world_sim/npcs.rs`): the registry tile_drop at the record cell, the
/// `UpdateNpcs` stream.
fn spawn_town_door_drops(app: &mut App, drops: &[edit::ObjectDrop]) {
    for d in drops {
        if let Some(item) = registry::tile_drop(d.tile_kind, 0) {
            let rng = app.rngs.stream(crate::app::RngKey::UpdateNpcs);
            spawn_item_drop(&mut app.items, rng, item, 1, d.x, d.y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemSlot;
    use glam::Vec2;
    use tr_core::tile::{FLAG_ACTIVE, Tile};
    use tr_core::World;

    use super::super::tests::{object_world, set_tile, slot};

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

    /// W4-e10 — the golem altar's roar rides the case-15 slot-0 seam
    /// (`Player.cs:32782`: `PlaySound(15, (int)position.X, (int)position.Y,
    /// 0)`): one play, NO `UpdatePlayers` draw, before the spawn.
    #[test]
    fn w4_e10_the_golem_altar_roar_draws_nothing() {
        let mut w = object_world();
        // The altar's origin cell (frameX 18 / frameY 0, `:32770`'s scan).
        set_tile(&mut w, 12, 19, 237);
        let i = w.index(12, 19);
        w.tiles[i].frame_x = 18;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.world.hard_mode = true;
        app.world.downed_plant_boss = true;
        let mut player = Player::spawn(&app.world);
        player.x = 10.0 * TILE_PX;
        player.y = 17.0 * TILE_PX;
        app.player = Some(player);
        app.inventory[0] = ItemSlot {
            kind: 1293,
            name: "Lihzahrd Power Cell".into(),
            count: 1,
            prefix: 0,
        };
        let before = app.audio.played_count();
        let rng_before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        assert!(use_golem_altar(&mut app));
        assert_eq!(app.audio.played_count(), before + 1, "the altar's roar");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            rng_before,
            "case 15 draws nothing"
        );
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Golem && n.hp > 0),
            "the Golem spawned"
        );
    }

    /// W4-e10 — the dial cue is the `SoundID.Item4` style façade at the
    /// player's top-left (`Player.cs:32642`/`:32652`): exactly ONE case-2
    /// `Next(-6, 7)` pitch draw on `UpdatePlayers` after the gate (the
    /// façade's `GetRandomPitch` NextFloat is audio-only).
    #[test]
    fn w4_e10_the_dial_cue_spends_one_case2_pitch_draw() {
        let mut w = object_world();
        set_tile(&mut w, 10, 19, 356); // the Enchanted Sundial
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 9.0 * TILE_PX;
        player.y = 17.0 * TILE_PX;
        app.player = Some(player);
        let mut mirror = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let _ = mirror.next_range(-6, 7);
        let before = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 19), "the sundial fires");
        assert_eq!(app.audio.played_count(), before + 1, "the Item4 chime");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            mirror,
            "the one case-2 pitch draw, nothing else"
        );
    }

    /// W4-e10 — the right-click toggle family's Mech-28 click
    /// (`Player.cs:32231`/`:32237`/`:32268`/`:32274`/`:32281`:
    /// `PlaySound(28, myX * 16, myY * 16, 0)`): ONE case-28
    /// `Next(-10, 11)` pitch draw on `UpdatePlayers` after the gate, at
    /// the CLICKED cell — and the toggle runs (the fountain's frameY
    /// flip; the party centre's flag + the THROW_A_PARTY up-edge).
    #[test]
    fn w4_e10_the_right_click_toggles_play_one_mech_draw_each() {
        // The fountain: a full 2×4 at (10..12, 16..20), frameY (y-16)*18.
        let mut w = object_world();
        for x in 10..12 {
            for y in 16..20 {
                set_tile(&mut w, x, y, 207);
                let i = w.index(x, y);
                w.tiles[i].frame_x = ((x - 10) * 18) as i16;
                w.tiles[i].frame_y = ((y - 16) * 18) as i16;
            }
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 10.0 * TILE_PX;
        player.y = 18.0 * TILE_PX;
        app.player = Some(player);
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (10.0 * TILE_PX, 18.0 * TILE_PX),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        let mut mirror = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let _ = mirror.next_range(-10, 11); // the one case-28 pitch draw
        let before = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 18), "the fountain fires");
        assert_eq!(app.audio.played_count(), before + 1, "the Mech-28 click");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            mirror,
            "one case-28 draw, nothing else"
        );
        assert_eq!(app.world.tile_frame_y(10, 16), 72, "SwitchFountain flipped");
        // The party centre: the click flips the manual flag and the
        // up-edge credits THROW_A_PARTY — one more Mech draw.
        set_tile(&mut app.world, 12, 19, 455);
        let _ = mirror.next_range(-10, 11);
        let before = app.audio.played_count();
        assert!(!app.world_events.manual_party);
        assert!(interact_tile(&mut app, 12, 19), "the party centre fires");
        assert!(app.world_events.manual_party, "ToggleManualParty");
        assert_eq!(app.audio.played_count(), before + 1);
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            mirror
        );
        assert!(
            app.achievements.take_completed_events().contains(&"THROW_A_PARTY"),
            "the up-edge credits the party achievement"
        );
        // The silent twin: the gate refuses before the draw — the toggle
        // still runs (the C# plays no sound at volume 0, the state moves).
        let mut app2 = flow_app(None, vec![], vec![]);
        let mut w2 = object_world();
        set_tile(&mut w2, 12, 19, 455);
        app2.world = w2;
        app2.player = Some(Player::spawn(&app2.world));
        app2.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        let rng_before = app2.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let before2 = app2.audio.played_count();
        assert!(interact_tile(&mut app2, 12, 19));
        assert!(app2.world_events.manual_party, "the toggle is not audio-gated");
        assert_eq!(app2.audio.played_count(), before2, "silent: no play");
        assert_eq!(
            *app2.rngs.stream(crate::app::RngKey::UpdatePlayers),
            rng_before,
            "the muted Mech click is draw-free"
        );
    }

    /// W4-g24 — the golden-door unlock arm (`Player.cs:32793-32837`): a
    /// locked door with a Golden Key 1141 consumes ONE key, shifts the
    /// three rows +54 and plays the Unlock cue; the click is consumed
    /// (`flag2 = true`) even with no key. The cue is draw-free (case 22).
    #[test]
    fn w4_g24_the_golden_key_unlocks_the_door_and_burns_one_key() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        // The locked golden door at column 10, rows 17-19.
        for (row, fy) in [(17, 594i16), (18, 612), (19, 630)] {
            let i = app.world.index(10, row);
            app.world.tiles[i] = Tile {
                kind: 10,
                frame_x: 0,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        app.world.frame_important[10] = true;
        app.player = Some(Player::spawn(&app.world));
        app.inventory[0] = slot(1141, 2);
        let before = app.audio.played_count();
        // The click on the bottom row: consumed, one key burned, the frames
        // shifted, one Unlock play.
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(app.inventory[0].count, 1, "one key consumed");
        assert_eq!(app.world.tile(10, 17).unwrap().frame_y, 648);
        assert_eq!(app.world.tile(10, 19).unwrap().frame_y, 684);
        assert_eq!(app.audio.played_count(), before + 1, "the Unlock cue");
        // The unlocked door now opens through the normal door arm (the
        // unlock arm no longer claims it).
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(app.world.tile(10, 19).unwrap().kind, 11, "the door opened");
    }

    /// W4-g24 — no key: the locked-door click is still consumed
    /// (`Player.cs:32795`'s `flag2 = true` before the scan), nothing shifts
    /// and no cue plays.
    #[test]
    fn w4_g24_the_locked_door_without_a_key_consumes_silently() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        for (row, fy) in [(17, 594i16), (18, 612), (19, 630)] {
            let i = app.world.index(10, row);
            app.world.tiles[i] = Tile {
                kind: 10,
                frame_x: 0,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        app.world.frame_important[10] = true;
        app.player = Some(Player::spawn(&app.world));
        let before = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 19), "the click is consumed");
        assert_eq!(app.world.tile(10, 17).unwrap().frame_y, 594, "no shift");
        assert_eq!(app.audio.played_count(), before, "no cue");
    }

    /// W4-g24 — the trapdoor interact arm (`Player.cs:32290-32303`): the
    /// click on the open 2x2 closes it (the pitched 8 plays), the click is
    /// consumed; the mirror re-click re-opens it. The tall gate (388/389)
    /// flips through its own arm (`:32304-32312`).
    #[test]
    fn w4_g24_the_trapdoor_and_tall_gate_interact_arms() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.player = Some(Player::spawn(&app.world));
        // The open trapdoor 2x2 at (10, 18)-(11, 19), style 0.
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let i = app.world.index(10 + dx, 18 + dy);
            app.world.tiles[i] = Tile {
                kind: 386,
                frame_x: (dx * 18) as i16,
                frame_y: (dy * 18) as i16,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        app.world.frame_important[386] = true;
        app.world.frame_important[387] = true;
        // Park the player well clear (left of the door, same height) so the
        // entity gate passes; `player_above` reads false (centre below the
        // clicked row), the X==0 arm closes the door upward.
        let p = app.player.as_mut().unwrap();
        p.x = 4.0 * 16.0;
        p.y = 18.0 * 16.0;
        let before = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 18), "the click is consumed");
        // The door closed (one of the rows became 387).
        let closed = (0..2).any(|dx| {
            app.world.tile(10 + dx, 18).is_some_and(|t| t.kind == 387)
                || app.world.tile(10 + dx, 19).is_some_and(|t| t.kind == 387)
        });
        assert!(closed, "the trapdoor shifted");
        assert!(app.audio.played_count() > before, "the pitched cue played");
        // The tall gate: a closed 388 column at (15, 14-18).
        let fys = [0i16, 18, 34, 50, 66];
        for (row, fy) in fys.iter().enumerate() {
            let i = app.world.index(15, 14 + row as i32);
            app.world.tiles[i] = Tile {
                kind: 388,
                frame_x: 0,
                frame_y: *fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        app.world.frame_important[388] = true;
        app.world.frame_important[389] = true;
        let before = app.audio.played_count();
        assert!(interact_tile(&mut app, 15, 16), "the gate click consumed");
        assert_eq!(app.world.tile(15, 14).unwrap().kind, 389, "the gate opened");
        assert!(app.audio.played_count() > before, "the gate cue played");
    }

    /// W4-g24 doc-sweep — the REAL `TownDoorOps` (`AppTownDoors`, the
    /// W4-b3 phase-2 seam): the open/close/gate attempts mutate the world
    /// AND play the pitched cues on the `UpdateNpcs` stream (the C# plays
    /// inside `NPC.UpdateNPC`, `Main.cs:17767`). Each success draws exactly
    /// ONE `Next(-20, 21)` pitch (`LegacySoundPlayer.cs:736`/:747); the
    /// wrong-kind refusal is silent and draw-free.
    #[test]
    fn w4_g24_the_town_door_adapter_plays_the_real_cues() {
        use crate::npc_ai_town::TownDoorOps;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        // A closed door (tile 10) at column 10, rows 17-19.
        for (row, fy) in [(17, 0i16), (18, 18), (19, 36)] {
            let i = app.world.index(10, row);
            app.world.tiles[i] = Tile {
                kind: 10,
                frame_x: 0,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        app.world.frame_important[10] = true;
        app.world.frame_important[11] = true;
        let mut player = Player::spawn(&app.world);
        player.x = 4.0 * 16.0;
        player.y = 18.0 * 16.0;
        app.player = Some(player);
        // Open: one pitch draw on UpdateNpcs, the door turns to 11.
        let mut witness = tr_core::unified_random::UnifiedRandom::new(0);
        let _ = witness.next_range(-20, 21);
        let before = app.audio.played_count();
        assert!(AppTownDoors { app: &mut app }.open_door(10, 18, 1));
        assert_eq!(app.world.tile(10, 18).unwrap().kind, 11, "the door opened");
        assert_eq!(app.audio.played_count(), before + 1, "the pitched 8");
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdateNpcs).peek(),
            witness.peek(),
            "exactly one door-pitch draw on UpdateNpcs"
        );
        // Close: the pitched 9, the hinge turns back to 10.
        let before = app.audio.played_count();
        assert!(AppTownDoors { app: &mut app }.close_door(10, 18));
        assert_eq!(app.audio.played_count(), before + 1, "the pitched 9");
        assert_eq!(app.world.tile(10, 18).unwrap().kind, 10, "the door closed");
        // The tall gate: a closed 388 column at (15, 14-18) opens through
        // the adapter (the core port, the pitched-8 cue at the gate centre).
        let fys = [0i16, 18, 34, 50, 66];
        for (row, fy) in fys.iter().enumerate() {
            let i = app.world.index(15, 14 + row as i32);
            app.world.tiles[i] = Tile {
                kind: 388,
                frame_x: 0,
                frame_y: *fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        app.world.frame_important[388] = true;
        app.world.frame_important[389] = true;
        let before = app.audio.played_count();
        assert!(AppTownDoors { app: &mut app }.shift_tall_gate(15, 16, false));
        assert_eq!(app.world.tile(15, 14).unwrap().kind, 389, "the gate opened");
        assert_eq!(app.audio.played_count(), before + 1, "the gate cue");
        // The wrong-kind refusal: silent, draw-free.
        let peek = app.rngs.stream(crate::app::RngKey::UpdateNpcs).peek();
        let before = app.audio.played_count();
        assert!(!AppTownDoors { app: &mut app }.open_door(20, 18, 1));
        assert_eq!(app.audio.played_count(), before, "no cue on refusal");
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdateNpcs).peek(),
            peek,
            "the refusal draws nothing"
        );
    }

    /// W4-g24 FINAL — the b3 co-lane adoption contract: the b3 phase-1
    /// wiring (`origin/task/W4-b3-npc-ai-town-wire` @ `96a5d88f`,
    /// `npc.rs::ai_007_town_family`) passes `NoOpTownDoors` into
    /// `town_npc_ai`; adopting `AppTownDoors` is a ONE-LINE swap
    /// (`let mut doors = NoOpTownDoors;` → `let mut doors = AppTownDoors
    /// { app };` — the arm's `world`/`player` borrows must move into the
    /// adapter's `&mut App` first, the b3 phase-2 caller re-shape). This
    /// witness pins the CONTRACT the swap relies on: the adapter satisfies
    /// the SAME `&mut dyn TownDoorOps` object-safety the arm's signature
    /// demands (`npc_ai_town.rs:1520`), and one dyn-dispatched open/close
    /// pair plays exactly one cue each on `UpdateNpcs` — the same stream
    /// the b3 caller passes (`world_sim/npcs.rs`'s `RngKey::UpdateNpcs`).
    #[test]
    fn w4_g24_the_b3_adoption_contract_is_dyn_compatible() {
        use crate::npc_ai_town::TownDoorOps;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        for (row, fy) in [(17, 0i16), (18, 18), (19, 36)] {
            let i = app.world.index(10, row);
            app.world.tiles[i] = Tile {
                kind: 10,
                frame_x: 0,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        app.world.frame_important[10] = true;
        app.world.frame_important[11] = true;
        let mut player = Player::spawn(&app.world);
        player.x = 4.0 * 16.0;
        player.y = 18.0 * 16.0;
        app.player = Some(player);
        // The dyn coercion the town arm's `doors: &mut dyn TownDoorOps`
        // parameter takes — NoOpTownDoors and AppTownDoors are drop-in
        // interchangeable at that seam (b3 phase 2 swaps the binding).
        let before = app.audio.played_count();
        {
            let mut adapter = AppTownDoors { app: &mut app };
            let doors: &mut dyn TownDoorOps = &mut adapter;
            assert!(doors.open_door(10, 18, 1), "the dyn open lands");
            assert!(doors.close_door(10, 18), "the dyn close lands");
        }
        assert_eq!(
            app.audio.played_count(),
            before + 2,
            "one pitched cue per success through the dyn seam"
        );
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
        // GetDoorItem(0) = 25; the 16x16 door rect centre (10*16+8, 18*16+8)
        // — the g78 FIX-ONCE (`ObjectDrop::spawn_center_px`).
        assert_eq!(events, vec![(25, 168.0, 296.0)]);

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
        // The cleanup table drop: the 32x32 rect centre (10*16+16, 17*16+16).
        assert_eq!(events, vec![(32, 176.0, 288.0)]);
    }

    // ---- W4-e28: the Crystal Ball / Ammo Box station buffs -----------------

    /// A station app: the object-world floor with the station tile at
    /// (10, 19) and the player spawned on the floor.
    fn station_app(kind: u16) -> App {
        let mut w = object_world();
        set_tile(&mut w, 10, 19, kind);
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let player = Player::spawn(&app.world);
        app.player = Some(player);
        app
    }

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
        // The effect lands through the sim's buff-flag seam
        // (`grant_ammo_mastery`, the `:10080` arm — the buff loop's flag
        // the 1/5 not-consume roll reads).
        let mut p = app.player.take().unwrap();
        crate::world_sim::player::grant_ammo_mastery(
            &mut p,
            &app.equip.armor,
            &app.equip.misc_equips,
        );
        assert!(p.ammo_box, "the 1/5 not-consume flag");
        app.player = Some(p);
    }

    #[test]
    fn w4_e28_the_station_grants_draw_no_rng() {
        // RNG-free verify: the C# arms call only `AddBuff` + `PlaySound`
        // (the plain `play_at` path draws no pitch roll) — no stream moves.
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

    // ---- W4-f34: the five neighbour station buffs -------------------------

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

    #[test]
    fn w4_f34_the_neighbour_grants_draw_no_rng_and_refresh_not_stack() {
        // RNG-free verify (the e28 pattern): the C# arms call only `AddBuff`
        // + `PlaySound` — the plain `play_at` seam draws no pitch roll. The
        // re-click refreshes in place (`AddBuff_TryUpdatingExistingBuffTime`,
        // `Player.cs:5402-5426`), never stacks; the cake's 7200 stays 7200.
        for (kind, buff, time) in [(621u16, 192, 7_200), (699u16, 366, 108_000)] {
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

    /// W4-e10 — the buff-station family (`Player.cs:32594-32634`): each of the
    /// seven right-click buff tiles grants its buff for the fixed duration,
    /// THEN plays its `SoundID.ItemNN` style at the player's TOP-LEFT — ONE
    /// case-2 `Next(-6, 7)` pitch draw on `UpdatePlayers` after the gate (the
    /// façade's `GetRandomPitch` NextFloat is audio-only). The buff→style pair
    /// is the C# chain's own; the click is consumed whether or not the player
    /// is present (`flag2 = true`).
    #[test]
    fn w4_e10_the_buff_stations_grant_and_play_one_case2_draw_each() {
        // (tile kind, buff id, buff duration, style name) — the C# chain order.
        let stations: [(u16, i32, i32, &str); 7] = [
            (125, 29, 108000, "Item4"),    // Crystal Ball — Clairvoyance
            (621, 192, 7200, "Item2"),     // Slice of Cake — Sugar Rush
            (464, 348, 108000, "Item4"),   // War Table
            (699, 366, 108000, "Item179"), // Dead Cells Potion Station
            (377, 159, 108000, "Item37"),  // Sharpening Station
            (354, 150, 108000, "Item4"),   // Bewitching Table
            (287, 93, 108000, "Item149"),  // Ammo Box
        ];
        for (kind, buff, time, _style) in stations {
            let mut w = object_world();
            set_tile(&mut w, 10, 19, kind);
            let mut app = flow_app(None, vec![], vec![]);
            app.world = w;
            let mut player = Player::spawn(&app.world);
            player.x = 9.0 * TILE_PX;
            player.y = 17.0 * TILE_PX;
            app.player = Some(player);
            app.audio.set_listener(crate::sound::Listener {
                camera_center: (9.0 * TILE_PX, 17.0 * TILE_PX),
                sound_volume: 1.0,
                ambient_volume: 1.0,
                quiet_ambient_sounds: false,
                game_menu: false,
                generating_or_loading: false,
                server: false,
            });
            let mut mirror = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
            let _ = mirror.next_range(-6, 7); // the one case-2 pitch draw
            let before = app.audio.played_count();
            assert!(interact_tile(&mut app, 10, 19), "tile {kind} fires");
            assert_eq!(app.audio.played_count(), before + 1, "tile {kind}'s chime");
            assert_eq!(
                *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                mirror,
                "tile {kind}: the one case-2 pitch draw, nothing else"
            );
            let p = app.player.as_ref().unwrap();
            let slot = p.buffs.find_buff_index(buff).expect("the buff granted");
            assert_eq!(p.buffs.buff_time[slot], time, "tile {kind}'s duration");
        }
        // 699 also counts as a Bone Welder (`TileCountsAs` 699→13), but the
        // buff arm wins the click BEFORE the crafting-station tail — the
        // craft filter stays unset.
        let mut w = object_world();
        set_tile(&mut w, 10, 19, 699);
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        assert!(interact_tile(&mut app, 10, 19));
        assert!(app.craft_filter.is_none(), "the buff arm wins over the station tail");
        // The silent twin: volume 0 refuses the draw, but the buff still lands
        // (the C# `AddBuff` runs before the play, `:32597` < `:32598`).
        let mut w = object_world();
        set_tile(&mut w, 10, 19, 125);
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 0.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        let rng_before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let before = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(app.audio.played_count(), before, "volume 0 plays nothing");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            rng_before,
            "the gate refuses before the draw"
        );
        assert!(
            app.player.as_ref().unwrap().buffs.find_buff_index(29).is_some(),
            "the buff still lands at volume 0"
        );
    }

    /// W4-e10 gap census — the sign arm (`Player.cs:32693-32723`): the
    /// right-click on a sign tile opens it single-player (`OpenSign(
    /// Sign.ReadSign)` `:32706` — create-if-missing, ONE Menu_Open
    /// `:33547`, the record's text into the chat line); a re-click on the
    /// OPEN sign closes it with ONE Menu_Close (`:32699` → `:33537`).
    /// Both consume the click; neither draws gameplay RNG.
    #[test]
    fn w4_e10_the_sign_arm_opens_then_closes_with_one_menu_cue_each() {
        let mut w = object_world();
        // A 2-wide sign at (10, 18)/(11, 18).
        for x in [10, 11] {
            set_tile(&mut w, x, 18, 55);
            let i = w.index(x, 18);
            w.tiles[i].frame_x = if x == 10 { 0 } else { 18 };
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 10.0 * TILE_PX;
        player.y = 18.0 * TILE_PX;
        app.player = Some(player);
        let rng_before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let before = app.audio.played_count();
        // The right half's click decodes the same origin (`signs::origin`).
        assert!(interact_tile(&mut app, 11, 18), "the sign consumes the click");
        assert_eq!(app.audio.played_count(), before + 1, "`:33547` Menu_Open");
        assert_eq!(app.player.as_ref().unwrap().sign, 0, "the latch");
        assert_eq!(app.world.signs.len(), 1, "ReadSign create-if-missing");
        assert!(app.npc_chat_text.is_empty(), "the fresh record's text");
        // The re-click closes (the same-sign probe), ONE Menu_Close.
        let mid = app.audio.played_count();
        assert!(interact_tile(&mut app, 10, 18));
        assert_eq!(app.audio.played_count(), mid + 1, "`:33537` Menu_Close");
        assert_eq!(app.player.as_ref().unwrap().sign, -1, "`:32699`");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            rng_before,
            "the menu cues draw no gameplay RNG"
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
    /// W4-c42 FINAL (the g78 research/76 contested-seam request): the
    /// Echo Monolith 657's RIGHT-CLICK use arm (`Player.cs:32271-32277`) —
    /// `PlaySound(28)` + `GamepadEnableGrappleCooldown` (3 ticks) +
    /// `SwitchMonolith`: the 2×3 body flips frameY ±54 (the e21 nine-kind
    /// gate's kept body), the click is consumed, the frames toggle back on
    /// the second use. The SceneMetrics lit-row gate (frameY ≥ 54) is the
    /// g78 consumer — this pins the flip that feeds it. WRITTEN NOT RUN.
    #[test]
    fn c42_the_monolith_right_click_flips_the_body_and_arms_the_grapple_cooldown() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 64, 64);
        // The unlit 2×3 Echo Monolith body at columns 10/11, rows 18-20
        // (the g78 placement witness's grid: frameX 0/18, frameY 0/18/36).
        for row in 0..3i32 {
            for col in 0..2i32 {
                let i = w.index(10 + col, 18 + row);
                w.tiles[i] = Tile {
                    kind: 657,
                    frame_x: (col * 18) as i16,
                    frame_y: (row * 18) as i16,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        // A non-anchor cell click (the C# decodes the anchor from frames).
        assert!(interact_tile(&mut app, 11, 19), "the click is consumed");
        for row in 0..3i32 {
            for col in 0..2i32 {
                assert_eq!(
                    app.world.tile(10 + col, 18 + row).unwrap().frame_y,
                    (row * 18 + 54) as i16,
                    "the lit row (the SceneMetrics frameY>=54 gate feeds)"
                );
            }
        }
        assert_eq!(
            app.player.as_ref().unwrap().quick_grapple_cooldown,
            3,
            "GamepadEnableGrappleCooldown armed 3"
        );
        // The second use flips back (frameY >= 54 → −54).
        assert!(interact_tile(&mut app, 10, 18));
        for row in 0..3i32 {
            assert_eq!(app.world.tile(10, 18 + row).unwrap().frame_y, (row * 18) as i16);
        }
        // No wiring flood ran: the skip set is untouched (the use path's
        // `Wiring.running` tail is inert, `WorldGen.cs:51137`).
        assert!(app.wiring.take_door_cues().is_empty());
    }
    /// W4-c36 DOC-SWEEP — the locked Lihzahrd door (`Player.cs:32792-32834`
    /// + `WorldGen.UnlockDoor` `:37599-37627`): a Temple Key pays, the three
    /// locked cells shift +54 frameY, TEMPLE_RAIDER completes; no key keeps
    /// the door locked and consumes the click; the void-bag bank4 fallback
    /// pays from the vault. WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_c36_the_temple_key_unlocks_the_lihzahrd_door() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 80, 200);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        // The locked door column at x 40: frameY 594/612/630 (the C#
        // `IsLockedDoor` band), frameX 0.
        for (dy, fy) in [(0, 594i16), (1, 612), (2, 630)] {
            let i = w.index(40, 30 + dy);
            w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: fy, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important[10] = true;
        w.compute_column_surface();
        app.world = w;
        let mut p = Player::spawn(&app.world);
        p.x = 40.0 * 16.0;
        p.y = 31.0 * 16.0;
        app.player = Some(p);
        // No key: the click is consumed, the door stays locked, no credit.
        assert!(interact_tile(&mut app, 40, 31));
        assert_eq!(app.world.tile(40, 30).unwrap().frame_y, 594);
        assert!(
            !app.achievements.list.iter().any(|a| a.name == "TEMPLE_RAIDER" && a.completed),
            "no key, no unlock"
        );
        // The key in the grid pays one and unlocks all three cells.
        app.inventory[3] = ItemSlot { kind: 1141, name: "Temple Key", count: 2, prefix: 0 };
        assert!(interact_tile(&mut app, 40, 31));
        assert_eq!(app.inventory[3].count, 1, "one key paid");
        for (dy, fy) in [(0, 648i16), (1, 666), (2, 684)] {
            assert_eq!(app.world.tile(40, 30 + dy).unwrap().frame_y, fy, "+54 shift");
        }
        assert!(
            app.achievements.list.iter().any(|a| a.name == "TEMPLE_RAIDER" && a.completed),
            "progression 22 completes the row"
        );
        // The unlocked door no longer matches IsLockedDoor — the key arm
        // does not run again (the normal open_door dispatcher owns it now).
        // The row is already completed, so a hypothetical re-fire would be
        // invisible — the key count is the observable.
        let _ = interact_tile(&mut app, 40, 31);
        assert_eq!(app.inventory[3].count, 1, "no second pay on the unlocked door");
        // The void-bag fallback: a fresh locked column, no inventory key,
        // the bag + a bank4 key pay from the vault.
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 80, 200);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        for (dy, fy) in [(0, 594i16), (1, 612), (2, 630)] {
            let i = w.index(40, 30 + dy);
            w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: fy, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important[10] = true;
        w.compute_column_surface();
        app.world = w;
        let mut p = Player::spawn(&app.world);
        p.x = 40.0 * 16.0;
        p.y = 31.0 * 16.0;
        p.banks[3][0] = ItemSlot { kind: 1141, name: "Temple Key", count: 1, prefix: 0 };
        app.player = Some(p);
        app.inventory[0] = ItemSlot { kind: 4131, name: "Void Bag", count: 1, prefix: 0 };
        assert!(interact_tile(&mut app, 40, 31));
        assert_eq!(app.player.as_ref().unwrap().banks[3][0].count, 0, "the vault key paid");
        assert_eq!(app.world.tile(40, 30).unwrap().frame_y, 648);
        assert!(
            app.achievements.list.iter().any(|a| a.name == "TEMPLE_RAIDER" && a.completed),
            "the fallback fires the same progression"
        );
    /// W4-c28: the door right-click fires the `19 ToggleDoorState` send
    /// decision (`Player.cs:32844`/`:32861`). The `19` id parks until d90's
    /// `DOOR_TOGGLE` codec, so the send is a recorded no-op — this pins that
    /// the interaction still opens/closes the door and reaches the send arm
    /// without a panic (the decision is wired at the fired cue, not the
    /// blocked one). A closed door (tile 10) at (10, 18), floor row 20, the
    /// stone ceiling row 16 (`CanPlace`'s upper `SolidTile` anchor — the
    /// core `doorway_world` fixture shape), the player parked clear of the
    /// opening column.
    #[test]
    fn the_door_click_fires_the_19_send_decision_on_open_and_close() {
        let mut w = object_world();
        set_tile(&mut w, 10, 16, 1);
        assert!(edit::place_item_tile(&mut w, 10, 18, 10, 0, 1));
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        // Park left of the door so the +1 opening column (11) is clear and
        // the close occupancy gate passes.
        player.x = 8.0 * 16.0;
        player.y = 17.0 * 16.0;
        player.facing = 1;
        app.player = Some(player);
        // The open: tile 10 → 11, the DoorOpened cue → the action-0 send arm.
        assert!(interact_tile(&mut app, 10, 18), "the open consumed the click");
        assert_eq!(app.world.tile(10, 18).map(|t| t.kind), Some(11), "the door opened");
        // The close: tile 11 → 10, the DoorClosed cue → the action-1 send arm.
        assert!(interact_tile(&mut app, 10, 18), "the close consumed the click");
        assert_eq!(app.world.tile(10, 18).map(|t| t.kind), Some(10), "the door closed");
    }
}
})
