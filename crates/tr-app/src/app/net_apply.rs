//! W4-g48 (S1-I-4 apply arms, lane 1 — `MessageBuffer.GetData` ids 1-40):
//! the parked wave-A/B/I dispatcher arms wired in the C# `GetData` case
//! order. Each fn is one case's apply half; the dispatchers in `netglue.rs`
//! call them and own the relay sends (`NetMessage.TrySendData` sites).
//!
//! The message handler runs outside every `SwapRandom` (`Main.cs:16895`'s
//! NpcStrike precedent — S1-E-21i), so every RNG draw here rides the bare
//! global stream (`app.rngs.global()`).

use tr_core::{edit, TILE_PX};

use crate::app::netmodules::ModuleBody;
use crate::app::App;
use crate::net::{self, S2c};
use crate::player;

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &tr_core::World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// The `17` reader's `Collision.EmptyTile(i, j)` entity half
/// (`Collision.cs:1366-1379`): the local player (unless dead/ghost) and every
/// live NPC refuse an occupied cell. The remote peers' positions are the
/// host's `remote_players` snapshots (the C# walks all 255 `Main.player`).
fn cell_blocked(app: &App, x: i32, y: i32) -> bool {
    entity_cell_blocked(app.player.as_ref(), &app.remote_players, &app.npcs, x, y)
}

/// The field-wise form of [`cell_blocked`] — the `19` trapdoor arm's closure
/// captures the three entity fields disjointly so the `&mut app.world` the
/// shift seam takes stays free (the `doors.rs::cell_blocked_by_entity`
/// convention, extended with the peer snapshots).
fn entity_cell_blocked(
    player: Option<&player::Player>,
    peers: &std::collections::HashMap<u32, net::PlayerNet>,
    npcs: &[crate::npc::Npc],
    x: i32,
    y: i32,
) -> bool {
    let (x0, y0) = (x as f32 * TILE_PX, y as f32 * TILE_PX);
    let (x1, y1) = (x0 + TILE_PX, y0 + TILE_PX);
    let overlaps = |px: f32, py: f32, w: f32, h: f32| {
        x0 < px + w && x1 > px && y0 < py + h && y1 > py
    };
    if let Some(p) = player {
        if !p.dead && overlaps(p.x, p.y, player::WIDTH, p.height) {
            return true;
        }
    }
    for peer in peers.values() {
        if !peer.dead && overlaps(peer.x, peer.y, player::WIDTH, player::HEIGHT) {
            return true;
        }
    }
    npcs.iter().any(|n| {
        let (w, h) = n.kind.size();
        overlaps(n.x, n.y, w, h)
    })
}

/// The `17` reader's `Chest.DestroyChest` half of
/// `CheckTileBreakability2_ShouldTileSurvive` (`WorldGen.cs:62487-62528`):
/// a stocked chest 21/467 or dresser 88 refuses the placement overwrite
/// (`flag16 = true`, the host answers `SendTileSquare(-1, x, y, 5)`); an
/// empty one is destroyed (the store entry removed, the local open index
/// closed silently, `Chest.cs:625-645`). The 470/475 TE arms have no port
/// tile entities (they read breakable — the C# `IsBreakable` with no entity
/// is true). Returns `true` when the tile SURVIVES.
fn should_tile_survive(app: &mut App, x: i32, y: i32) -> bool {
    let Some(t) = app.world.tile(x, y).copied() else {
        return false;
    };
    let anchor = if tr_core::tiledata::SET_BASIC_CHEST.has(t.kind) {
        // `:62500-62505` — frameX/18 folded mod 2, frameY/18 rows up.
        let mut num = i32::from(t.frame_x) / 18;
        while num > 1 {
            num -= 2;
        }
        Some((x - num, y - i32::from(t.frame_y) / 18))
    } else if t.kind == 88 {
        // `:62513-62514` — frameX/18 mod 3.
        Some((
            x - i32::from(t.frame_x) / 18 % 3,
            y - i32::from(t.frame_y) / 18,
        ))
    } else {
        None
    };
    let Some((ax, ay)) = anchor else {
        return false;
    };
    let Some(idx) = crate::chest::find_chest(&app.world_chests, ax, ay) else {
        return false; // no entry → destroyable (`Chest.cs:627-630`)
    };
    let stocked = app.world_chests[idx]
        .slots
        .iter()
        .any(|s| s.kind > 0 && s.count > 0);
    if stocked {
        return true; // `Chest.DestroyChest` refused (`Chest.cs:631-637`)
    }
    crate::chest::remove_chest(&mut app.world_chests, idx);
    if let Some(p) = app.player.as_mut() {
        if p.chest == idx as i32 {
            p.chest = crate::bank::CHEST_CLOSED; // `Chest.cs:640-643`
        }
    }
    false
}

/// `Chest.CanDestroyChest`'s item scan over the DISJOINT chest-store field
/// (the `world_sim/editing.rs` closure precedent — a whole-`App` borrow would
/// conflict with the `&mut app.world` the edit seams take).
fn holds_items(chests: &[crate::craft::WorldChest]) -> impl Fn(i32, i32) -> bool + '_ {
    move |x, y| {
        chests
            .iter()
            .any(|c| c.tile == (x, y) && c.slots.iter().any(|s| s.count > 0))
    }
}

/// The `17 TileManipulation` reader (`MessageBuffer.cs:1119-1288`) — the
/// host's apply + relay. `id` is the sender (`this.whoAmI`); the spam
/// counters (`:1141-1146`) are dormant (`Netplay.SpamCheck` defaults false,
/// `Netplay.cs:867` — the `-secure` flag is unported, ANALYSIS). The
/// unsent-section fail-force (`:1148-1151`) reads the slot's section latch.
/// The relay (`:1277-1287`) is verbatim: the survive-flag answers a 5-square,
/// the falling-kind placement that left the cell active sends nothing,
/// everything else relays as `17` to the others.
pub(crate) fn host_tile_edit(app: &mut App, server: &net::Server, id: u32, e: net::TileEdit) {
    let (x, y) = (e.x as i32, e.y as i32);
    let mut fail = e.value == 1; // `flag15` (`:1126`)
    if !in_world(&app.world, x, y, 3) {
        return; // `:1127-1130`
    }
    // `:1148-1151` — a section the peer never received forces the fail flag.
    if !server.is_section_sent(id, net::section_x(x), net::section_y(y)) {
        fail = true;
    }
    // `MapUpdateQueue.Add(num150, num151)` (`:1153`) — the single tile (the
    // kill's own `TileFrame` head queues its ring inside the cascade).
    if app.settings.map_enabled {
        if let Some(m) = app.map.as_mut() {
            app.map_queue.add_tile(m, x, y);
        }
    }
    let mut survived = false; // `flag16`
    let mut relay_value = e.value;
    // The kind dispatch (`:1157-1275`), in the C#'s if-chain order.
    match e.kind {
        0 => {
            // `WorldGen.KillTile(x, y, fail)` (`:1159`). The fail flag is the
            // dust/sound-only break (`WorldGen.cs:63709-63737`); the port's
            // kill seam has no effect-only flavour, so a forced fail skips
            // the break (the fail-dust is the visual seam, ANALYSIS).
            if !fail {
                crate::world_sim::editing::replicated_break(app, x, y, false, true);
            }
        }
        1 => {
            // `:1165-1174` — the survive check, then `PlaceTile(forced =
            // !survived, style)`. The port's placement route is the item-use
            // dispatcher (the same `TileObject.CanPlace`/`Place` the C#
            // `PlaceTile` runs for registered kinds).
            let mut forced = true;
            if should_tile_survive(app, x, y) {
                survived = true;
                forced = false;
            }
            let kind = e.value.max(0) as u16;
            if forced {
                let _ = edit::place_item_tile(&mut app.world, x, y, kind, e.style as i32, 1);
            } else {
                // The non-forced placement still runs (`PlaceTile` with
                // `forced: false` — the occupancy gate applies).
                if !cell_blocked(app, x, y) {
                    let _ = edit::place_item_tile(&mut app.world, x, y, kind, e.style as i32, 1);
                }
            }
            app.world_key = None;
        }
        2 => {
            // `WorldGen.KillWall(x, y, fail)` (`:1177`) — the fail flag is
            // the dust-only break; the port's seam has no fail flavour.
            if !fail {
                if let Some(wall) = edit::break_wall(&mut app.world, x, y) {
                    app.audio.play_wall_break_sound(
                        edit::kill_wall_sound(wall),
                        x as f32 * TILE_PX,
                        y as f32 * TILE_PX,
                        app.rngs.global(),
                    );
                }
                app.world_key = None;
            }
        }
        3 => {
            // `WorldGen.PlaceWall(x, y, num152)` (`:1181`).
            let _ = edit::place_wall(&mut app.world, x, y, e.value.max(0) as u16);
            app.world_key = None;
        }
        4 => {
            // `KillTile(fail, effectOnly: false, noItem: true)` (`:1185`) —
            // the actuator-rod break: the hit cell drops no item (the
            // cascade records still run).
            if !fail {
                crate::world_sim::editing::replicated_break(app, x, y, true, true);
            }
        }
        5 | 6 | 10 | 11 | 12 | 13 | 16 | 17 => {
            // The wire colours (`:1187-1237`): PlaceWire/KillWire ×4. The
            // C#'s sound 0 + the kill's 530 drop (`WorldGen.cs:58115`/
            // `:58126-58130`) — the host drops (`netMode != 1`).
            use tr_core::wiring::WireColor;
            let (color, place) = match e.kind {
                5 => (WireColor::Red, true),
                6 => (WireColor::Red, false),
                10 => (WireColor::Blue, true),
                11 => (WireColor::Blue, false),
                12 => (WireColor::Green, true),
                13 => (WireColor::Green, false),
                16 => (WireColor::Yellow, true),
                _ => (WireColor::Yellow, false),
            };
            let changed = if place {
                edit::place_wire(&mut app.world, x, y, color)
            } else {
                edit::kill_wire(&mut app.world, x, y, color)
            };
            if changed {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                if !place {
                    let rng = app.rngs.global();
                    crate::world_sim::items::spawn_item_drop(&mut app.items, rng, 530, 1, x, y);
                }
                app.world_key = None;
            }
        }
        7 => {
            // `WorldGen.PoundTile(x, y)` (`:1197`) — the half-brick toggle.
            let holds = holds_items(&app.world_chests);
            if edit::pound_tile(&mut app.world, x, y, false, holds) {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                app.world_key = None;
            }
        }
        8 | 9 => {
            // `PlaceActuator`/`KillActuator` (`:1201`/`:1205`).
            let changed = if e.kind == 8 {
                edit::place_actuator(&mut app.world, x, y)
            } else {
                edit::kill_actuator(&mut app.world, x, y)
            };
            if changed {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                if e.kind == 9 {
                    let rng = app.rngs.global();
                    crate::world_sim::items::spawn_item_drop(&mut app.items, rng, 849, 1, x, y);
                }
                app.world_key = None;
            }
        }
        14 => {
            // `WorldGen.SlopeTile(x, y, num152)` (`:1225`).
            let holds = holds_items(&app.world_chests);
            if edit::slope_tile(&mut app.world, x, y, e.value.max(0) as u8, false, holds) {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                app.world_key = None;
            }
        }
        15 => {
            // `Minecart.FrameTrack(x, y, pound: true)` (`:1229`) — the
            // C# default `mute: false`. The f11 canonical track-framing
            // seam (`tr_core::minecart`, the switch-cycle pound branch);
            // the fail-tile tail is the documented no-op fix over f11
            // (the C# `KillTile(fail: true)` never removes the track,
            // `WorldGen.cs:63709-63737` — the module doc). The relay
            // still runs (the C# falls through to `:1277`).
            if tr_core::minecart::frame_track(&mut app.world, x, y, true, false) {
                app.world_key = None;
            }
        }
        18 => {
            // `Wiring.PokeLogicGate` (`:1241-1245`) — the C# RETURNS (no
            // relay). `SetCurrentUser` has no port state (the wiring's
            // trigger attribution is the recorded M7-WIRE-2 gap).
            app.wiring.poke_logic_gate(&mut app.world, x, y);
            app.world_key = None;
            return;
        }
        19 => {
            // `Wiring.Actuate` (`:1246-1250`) — RETURN, no relay.
            let _ = app.wiring.actuate(&mut app.world, x, y);
            app.world_key = None;
            return;
        }
        20 => {
            // `:1251-1262` — the boulder break: KillTile, then the relay
            // value is 1 when the tile survived (still active, same type).
            if in_world(&app.world, x, y, 2) {
                let kind = app.world.tile(x, y).map(|t| t.kind);
                crate::world_sim::editing::replicated_break(app, x, y, false, true);
                let still = app
                    .world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && Some(t.kind) == kind);
                relay_value = if still { 1 } else { 0 };
                // `:1259` — the relay goes to EVERYONE (ignoreClient -1).
                server.broadcast(&S2c::TileEdit(net::TileEdit {
                    value: relay_value,
                    ..e
                }));
            }
            return;
        }
        21 => {
            // `WorldGen.ReplaceTile(x, y, (ushort)num152, num153)` (`:1264`)
            // — the smart-cursor block swap (the `edit::replace` family,
            // W4-g48; the former `liquid.rs:709` cited skip is closed).
            let _ = replace_tile_apply(app, Some(server), x, y, e.value.max(0) as u16, e.style as i32, true);
        }
        22 => {
            // `WorldGen.ReplaceWall(x, y, (ushort)num152)` (`:1269`).
            if let Some(old) = edit::replace_wall(&mut app.world, x, y, e.value.max(0) as u16) {
                app.audio.play_wall_break_sound(
                    edit::kill_wall_sound(old),
                    x as f32 * TILE_PX,
                    y as f32 * TILE_PX,
                    app.rngs.global(),
                );
                app.world_key = None;
            }
        }
        23 => {
            // `:1271-1275` — the slope-set pound: `CanPoundTile` gates, then
            // `slope(num152)` + `PoundTile`.
            let holds = holds_items(&app.world_chests);
            if edit::can_pound_tile(&app.world, x, y, false, holds) {
                if let Some(t) = app.world.tile_mut(x, y) {
                    t.set_slope(e.value.max(0) as u8);
                }
                let holds = holds_items(&app.world_chests);
                if edit::pound_tile(&mut app.world, x, y, false, holds) {
                    app.audio
                        .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                    app.world_key = None;
                }
            }
        }
        _ => {}
    }
    // The relay (`:1277-1287`).
    if survived {
        // `SendTileSquare(-1, x, y, 5)` (`:1281`) — the centred 5-square.
        let range = tile_square(app, x - 2, y - 2, 5, 5);
        server.broadcast(&S2c::SectionRange(Box::new(range)));
    } else {
        let falling_active = (e.kind == 1 || e.kind == 21)
            && tr_core::falling::is_falling_kind(e.value.max(0) as u16)
            && !app.world.tile_active(x, y);
        if !falling_active {
            server.broadcast_except(
                &S2c::TileEdit(net::TileEdit {
                    value: relay_value,
                    ..e
                }),
                id,
            );
        }
    }
}

/// The `17` reader's client half (`MessageBuffer.cs:1157-1275`, the
/// `netMode == 1` path): the same kind dispatch WITHOUT the spam/section
/// gates and the relay; kind 0 additionally clears the local crack book
/// (`HitTile.ClearAllTilesAtThisLocation`, `:1160-1163`).
pub(crate) fn client_tile_edit(app: &mut App, e: net::TileEdit) {
    let (x, y) = (e.x as i32, e.y as i32);
    let fail = e.value == 1;
    if !in_world(&app.world, x, y, 3) {
        return;
    }
    if app.settings.map_enabled {
        if let Some(m) = app.map.as_mut() {
            app.map_queue.add_tile(m, x, y);
        }
    }
    match e.kind {
        0 => {
            if !fail {
                // The client's KillTile spawns no drops (`netMode != 1`
                // gate, `WorldGen.cs:63860`) — the items arrive as `21`.
                crate::world_sim::editing::replicated_break(app, x, y, true, false);
                // `:1160-1163` — the crack book clears at the broken cell.
                let world = &app.world;
                app.hit_tile.clear_this_tile(
                    x,
                    y,
                    |wx, wy| world.tile_active(wx, wy),
                    |wx, wy| world.tile(wx, wy).map_or(0, |t| t.wall),
                );
            }
        }
        1 => {
            let kind = e.value.max(0) as u16;
            let _ = edit::place_item_tile(&mut app.world, x, y, kind, e.style as i32, 1);
            app.world_key = None;
        }
        2 => {
            if !fail {
                if let Some(wall) = edit::break_wall(&mut app.world, x, y) {
                    app.audio.play_wall_break_sound(
                        edit::kill_wall_sound(wall),
                        x as f32 * TILE_PX,
                        y as f32 * TILE_PX,
                        app.rngs.global(),
                    );
                }
                app.world_key = None;
            }
        }
        3 => {
            let _ = edit::place_wall(&mut app.world, x, y, e.value.max(0) as u16);
            app.world_key = None;
        }
        4 => {
            if !fail {
                crate::world_sim::editing::replicated_break(app, x, y, true, false);
            }
        }
        5 | 6 | 10 | 11 | 12 | 13 | 16 | 17 => {
            use tr_core::wiring::WireColor;
            let (color, place) = match e.kind {
                5 => (WireColor::Red, true),
                6 => (WireColor::Red, false),
                10 => (WireColor::Blue, true),
                11 => (WireColor::Blue, false),
                12 => (WireColor::Green, true),
                13 => (WireColor::Green, false),
                16 => (WireColor::Yellow, true),
                _ => (WireColor::Yellow, false),
            };
            let changed = if place {
                edit::place_wire(&mut app.world, x, y, color)
            } else {
                edit::kill_wire(&mut app.world, x, y, color)
            };
            if changed {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                // The client's KillWire drops nothing (`netMode != 1` gate,
                // `WorldGen.cs:58128`).
                app.world_key = None;
            }
        }
        7 => {
            let holds = holds_items(&app.world_chests);
            if edit::pound_tile(&mut app.world, x, y, false, holds) {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                app.world_key = None;
            }
        }
        8 | 9 => {
            let changed = if e.kind == 8 {
                edit::place_actuator(&mut app.world, x, y)
            } else {
                edit::kill_actuator(&mut app.world, x, y)
            };
            if changed {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                app.world_key = None;
            }
        }
        14 => {
            let holds = holds_items(&app.world_chests);
            if edit::slope_tile(&mut app.world, x, y, e.value.max(0) as u8, false, holds) {
                app.audio
                    .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                app.world_key = None;
            }
        }
        15 => {
            // The client runs the same `FrameTrack(pound: true)` (the C#
            // reader has one body for both netModes).
            if tr_core::minecart::frame_track(&mut app.world, x, y, true, false) {
                app.world_key = None;
            }
        }
        21 => {
            // The client runs the same `ReplaceTile` — the `netMode == 1`
            // gates inside skip the drops, the fossil chain and the rigged
            // switch (the loot arrives as `21`, the break relays as `17`).
            let _ = replace_tile_apply(app, None, x, y, e.value.max(0) as u16, e.style as i32, false);
        }
        18 => {
            app.wiring.poke_logic_gate(&mut app.world, x, y);
            app.world_key = None;
        }
        19 => {
            let _ = app.wiring.actuate(&mut app.world, x, y);
            app.world_key = None;
        }
        20 => {
            // The host's boulder relay (`:1259`): the client kills too (no
            // drops — the `netMode != 1` gate).
            if in_world(&app.world, x, y, 2) {
                crate::world_sim::editing::replicated_break(app, x, y, true, false);
            }
        }
        22 => {
            if let Some(old) = edit::replace_wall(&mut app.world, x, y, e.value.max(0) as u16) {
                app.audio.play_wall_break_sound(
                    edit::kill_wall_sound(old),
                    x as f32 * TILE_PX,
                    y as f32 * TILE_PX,
                    app.rngs.global(),
                );
                app.world_key = None;
            }
        }
        23 => {
            let holds = holds_items(&app.world_chests);
            if edit::can_pound_tile(&app.world, x, y, false, holds) {
                if let Some(t) = app.world.tile_mut(x, y) {
                    t.set_slope(e.value.max(0) as u8);
                }
                let holds = holds_items(&app.world_chests);
                if edit::pound_tile(&mut app.world, x, y, false, holds) {
                    app.audio
                        .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
                    app.world_key = None;
                }
            }
        }
        _ => {}
    }
}

/// The `16 PlayerLifeMana` reader's apply half (`MessageBuffer.cs:1095-1117`):
/// the host stores the sender's health (`:1100-1111`, the id forced to the
/// sender, `statLifeMax` floored at 20 by the decode, `dead` derived) and
/// relays to the others (`:1114`). The client arm stores a peer's health.
/// The local-echo skip (`:1098`) never fires on the port's wire (the host
/// relays with `ignoreClient = sender`).
pub(crate) fn host_player_hp(app: &mut App, server: &net::Server, id: u32, hp: i16, max_hp: i16) {
    if let Some(p) = app.remote_players.get_mut(&id) {
        p.hp = hp as i32;
        p.max_hp = (max_hp as i32).max(20);
        p.dead = p.hp <= 0; // `:1111`
    }
    server.broadcast_except(
        &S2c::PlayerHp {
            player_id: id as u8,
            hp,
            max_hp,
        },
        id,
    ); // `:1114`
}

/// The `16` client half: a peer's health lands in the snapshot store.
pub(crate) fn client_player_hp(app: &mut App, player_id: u8, hp: i16, max_hp: i16) {
    if let Some(p) = app.remote_players.get_mut(&(player_id as u32)) {
        p.hp = hp as i32;
        p.max_hp = (max_hp as i32).max(20);
        p.dead = p.hp <= 0; // `:1111`
    }
}

/// `Main.teamColor[team]` (`Main.cs:6657-6662`) — the PVP chat line's colour
/// (`MessageBuffer.cs:1864`). An out-of-range team reads white (the C# would
/// throw; the wire byte is 0..5 in practice). `i32` = the `Player.team`
/// field's type (the g48/g49 union widened `PlayerNet.team` to `i32`).
fn team_color(team: i32) -> [u8; 3] {
    match team {
        1 => [218, 59, 59],
        2 => [59, 218, 85],
        3 => [59, 149, 218],
        4 => [242, 221, 100],
        5 => [224, 100, 242],
        _ => [255, 255, 255],
    }
}

/// The `19 ToggleDoorState` reader (`MessageBuffer.cs:1299-1334`): the
/// action byte maps 0/1 door open/close, 2/3 trapdoor, 4/5 tall gate; the
/// direction normalises `!= 0` to 1 else -1 (`:1306`) and the host relays
/// the normalised byte (`:1330`). The trapdoor/tall-gate shifts run the
/// `edit::shift_trapdoor`/`shift_tall_gate` canonical bodies (W4-c42
/// `edit/shift.rs`, the research/95 union-exec — landed byte-identical on
/// this lane; the former `npc_ai_town.rs` cited skip is closed).
pub(crate) fn host_door_toggle(app: &mut App, server: &net::Server, id: u32, d: net::DoorToggle) {
    let (x, y) = (d.x as i32, d.y as i32);
    if !in_world(&app.world, x, y, 3) {
        return; // `:1304`
    }
    let dir = if d.direction != 0 { 1 } else { -1 }; // `num47` (`:1306`)
    apply_door_action(app, d.action, x, y, dir);
    // `:1330` — the relay carries the NORMALISED direction byte.
    server.broadcast_except(
        &S2c::DoorToggle(net::DoorToggle {
            direction: if dir == 1 { 1 } else { 0 },
            ..d
        }),
        id,
    );
}

/// The `19` client half: the same action dispatch, no relay.
pub(crate) fn client_door_toggle(app: &mut App, d: net::DoorToggle) {
    let (x, y) = (d.x as i32, d.y as i32);
    if !in_world(&app.world, x, y, 3) {
        return;
    }
    let dir = if d.direction != 0 { 1 } else { -1 };
    apply_door_action(app, d.action, x, y, dir);
}

/// The `19` action switch (`MessageBuffer.cs:1307-1327`). The door open/close
/// run the `tr_core::edit` seams (the same `WorldGen.OpenDoor`/`CloseDoor`
/// the interaction path uses); the close is `forced: true` (`:1313`). The
/// sounds are the seams' own (`WorldGen.cs:37727`/:31908) — the message
/// handler's bare-stream precedent (S1-E-21i).
fn apply_door_action(app: &mut App, action: u8, x: i32, y: i32, dir: i32) {
    match action {
        0 => {
            // `WorldGen.OpenDoor(x, y, num47)` (`:1310`).
            let mut drops = Vec::new();
            if edit::open_door(&mut app.world, x, y, dir, &mut drops) {
                app.audio.play_door_sound(
                    true,
                    x as f32 * TILE_PX,
                    y as f32 * TILE_PX,
                    app.rngs.global(),
                );
                spawn_object_drops(app, drops);
                app.world_key = None;
            }
        }
        1 => {
            // `WorldGen.CloseDoor(x, y, forced: true)` (`:1313`) — the
            // forced close skips the occupancy gate (the wired caller).
            let mut drops = Vec::new();
            if edit::close_door(&mut app.world, x, y, true, &|_, _| false, &mut drops) {
                app.audio.play_door_sound(
                    false,
                    x as f32 * TILE_PX,
                    y as f32 * TILE_PX,
                    app.rngs.global(),
                );
                spawn_object_drops(app, drops);
                app.world_key = None;
            }
        }
        2 | 3 => {
            // `WorldGen.ShiftTrapdoor(x, y, num47 == 1, 1/0)` (`:1316`/
            // `:1319`): action 2 passes `onlyCloseOrOpen: 1` (CLOSE-only —
            // the `:51450` open branch gates `!= 1`), action 3 passes 0
            // (OPEN-only — the `:51534` close branch gates `!= 0`); the C#
            // call sites, not the wiring default −1. The canonical body is
            // the W4-c42 `edit/shift.rs`
            // (research/95 §2 union-exec: paint + fullbright coating, the
            // in-body per-site cues incl. the refusal thud, `touch()`, the
            // injected `can_kill` slice). No wiring flood runs on the
            // message handler — `wire_skips: None` (the C#
            // `Main.netMode != 1 && Wiring.running` gate is false here).
            let only = if action == 2 { 1 } else { 0 };
            let mut drops = Vec::new();
            let mut cues = Vec::new();
            {
                // The entity half of `Collision.EmptyTile(ignoreTiles:
                // true)` (research/95 §8.1): the REAL player/peer/NPC box
                // overlap (the `cell_blocked` reader convention) — tr-core
                // carries no entity store, so the closure injects it; the
                // field-wise capture keeps the `&mut app.world` borrow free
                // (the block ends the borrows before the drop spawn).
                let (player, peers, npcs) = (&app.player, &app.remote_players, &app.npcs);
                let blocked =
                    |x: i32, y: i32| entity_cell_blocked(player.as_ref(), peers, npcs, x, y);
                // The close guard rides the FULL `WorldGen.CanKillTile` (the
                // reader is the player path, not the wiring flood — the f11
                // injection choice, research/95 §2): the chest-store scan
                // reads the disjoint `world_chests` field.
                let chests = &app.world_chests;
                let can_kill = |w: &tr_core::World, x: i32, y: i32| {
                    edit::can_kill_tile(w, x, y, |cx, cy| {
                        chests
                            .iter()
                            .any(|c| c.tile == (cx, cy) && c.slots.iter().any(|s| s.count > 0))
                    })
                };
                let _ = edit::shift_trapdoor(
                    &mut app.world,
                    x,
                    y,
                    dir == 1,
                    only,
                    &blocked,
                    &can_kill,
                    &mut drops,
                    None,
                    &mut cues,
                );
            }
            // The in-body cues (`PlaySound(8/9)` at the adjusted anchor)
            // play on the bare global stream — the message handler runs
            // outside every SwapRandom (S1-E-21i).
            for (open, px, py) in cues {
                app.audio.play_door_sound(
                    open,
                    px as f32,
                    py as f32,
                    app.rngs.global(),
                );
            }
            spawn_object_drops(app, drops);
            app.world_key = None;
        }
        4 | 5 => {
            // `WorldGen.ShiftTallGate(x, y, closing, forced: true)` (`:1322`/
            // `:1325`): action 4 opens (`closing: false`), 5 closes; the
            // reader ALWAYS passes `forced: true` — the entity gate is
            // skipped entirely (the closure is never consulted). The
            // canonical W4-c42 body: alternate-0 def decode of kind 388,
            // the padding-free row walk, the radius-4 reframe superset.
            let mut drops = Vec::new();
            let mut cues = Vec::new();
            let _ = edit::shift_tall_gate(
                &mut app.world,
                x,
                y,
                action == 5,
                true,
                &|_, _| false,
                &mut drops,
                None,
                &mut cues,
            );
            for (open, px, py) in cues {
                app.audio.play_door_sound(
                    open,
                    px as f32,
                    py as f32,
                    app.rngs.global(),
                );
            }
            spawn_object_drops(app, drops);
            app.world_key = None;
        }
        _ => {}
    }
}

/// The door seams' object drops (`edit::ObjectDrop` → `registry::tile_drop`,
/// the `doors.rs` `interaction_events` shape) on the bare global stream.
fn spawn_object_drops(app: &mut App, drops: Vec<edit::ObjectDrop>) {
    for d in drops {
        if let Some(id) = crate::registry::tile_drop(d.tile_kind, 0) {
            let rng = app.rngs.global();
            crate::world_sim::items::spawn_item_drop(&mut app.items, rng, id, 1, d.x, d.y);
        }
    }
}

/// The `30 TogglePVP` reader (`MessageBuffer.cs:1851-1867`): the host forces
/// the id to the sender (`:1854-1857`), stores `hostile` (`:1859`), relays
/// (`:1862`) and broadcasts the team-coloured chat line (`:1863-1864`,
/// `Lang.mp[11]` on / `Lang.mp[12]` off, `NetworkText.FromKey(key, name)`).
pub(crate) fn host_toggle_pvp(app: &mut App, server: &net::Server, id: u32, hostile: bool) {
    let team = app
        .remote_players
        .get(&id)
        .map(|p| p.team)
        .unwrap_or(0);
    if let Some(p) = app.remote_players.get_mut(&id) {
        p.hostile = hostile; // `:1859`
    }
    server.broadcast_except(
        &S2c::TogglePvp(net::TogglePvp {
            player_id: id as u8,
            hostile,
        }),
        id,
    ); // `:1862`
    // `:1863-1864` — the chat fan-out. The C# broadcasts to EVERYONE (no
    // ignoreClient); the port's `NetTextModule` relay is the same shape
    // (`host_module`'s TEXT arm). The keyed text resolves through the lang
    // manager (`{0}` = the player's name).
    let name = server.slot_name(id);
    let key = if hostile { 11 } else { 12 };
    let text = crate::lang::text_args(&format!("LegacyMultiplayer.{key}"), &[name]);
    let color = team_color(team);
    app.chat.new_text(text.clone(), color);
    let sm = crate::app::netmodules::ServerMessage {
        author: 255, // the server's own line
        text,
        color: net::Rgb(color[0], color[1], color[2]),
    };
    server.broadcast(&S2c::NetModule {
        module: crate::app::netmodules::module_id::TEXT,
        body: sm.encode(),
    });
}

/// The `30` client half (`MessageBuffer.cs:1858-1859`): a peer's PvP flag
/// lands in the snapshot store (the chat line arrives as the `82` module).
pub(crate) fn client_toggle_pvp(app: &mut App, player_id: u8, hostile: bool) {
    if let Some(p) = app.remote_players.get_mut(&(player_id as u32)) {
        p.hostile = hostile;
    }
}

/// `WorldGen.PlaceChest(x, y, type, notNearOtherChests: false, style)`
/// (`WorldGen.cs:57813-57846`): the boulder gate below, then
/// `TileObject.CanPlace`/`Place` + `Chest.CreateChest`. Returns the store
/// index or -1. The `netMode == 1` send tail (`:57837-57844`) is the local
/// placement path's, not the reader's — the reader relays its own `34`.
fn place_chest(app: &mut App, x: i32, y: i32, kind: u16, style: i32) -> i32 {
    // `:57816-57819` — a boulder under either column refuses.
    for dx in 0..2 {
        if app
            .world
            .tile(x + dx, y + 1)
            .is_some_and(|t| tr_core::tiledata::SET_BOULDERS.has(t.kind))
        {
            return -1;
        }
    }
    let Some(def) = tr_core::tileobject::def_for_tile(kind) else {
        return -1;
    };
    let mut rand = std::mem::take(&mut app.world.place_rng);
    let instance = tr_core::tileobject::can_place(
        &app.world,
        x,
        y,
        def,
        style,
        1,
        None,
        &mut rand,
        app.world.tile_object_random_cache,
    );
    app.world.place_rng = rand;
    let Some(instance) = instance else {
        return -1;
    };
    if !tr_core::tileobject::place(&mut app.world, &instance) {
        return -1;
    }
    // `Chest.CreateChest(objectData.xCoord, objectData.yCoord)` (`:57830`).
    match crate::chest::find_empty_chest(&app.world_chests, instance.x_coord, instance.y_coord) {
        Some(_) => {
            let idx = crate::chest::create_chest(
                &mut app.world_chests,
                instance.x_coord,
                instance.y_coord,
            );
            idx as i32
        }
        None => -1,
    }
}

/// The `34 ChestUpdates` reader (`MessageBuffer.cs:1984-2158`) — the host's
/// apply + relay. `b4` maps 0/1 chest place/destroy (21), 2/3 dresser (88),
/// 4/5 chest2 (467); the host zeroes the wire index (`:1991-1994`) and
/// answers the placement's store index (or -1 + the item drop on refusal).
/// The destroy arms decode the object's top-left, `KillTile` it and relay
/// the found index when the tile died.
pub(crate) fn host_chest_updates(app: &mut App, server: &net::Server, id: u32, u: net::ChestUpdates) {
    let (mut x, mut y) = (u.x as i32, u.y as i32);
    let style = u.style as i32;
    let action = u.action;
    // The placement kinds and their tile types (`:2003`/:2045/:2080).
    let (place_kind, destroy_kind): (u16, u16) = match action {
        0 | 1 => (21, 21),
        2 | 3 => (88, 88),
        4 | 5 => (467, 467),
        _ => return,
    };
    if matches!(action, 0 | 2 | 4) {
        // The placement arm (`:2001-2017`/:2043-2055/:2078-2094`).
        let result = place_chest(app, x, y, place_kind, style);
        if result == -1 {
            // The refusal: relay -1 to the asker + drop the item
            // (`Item.NewItem(..., noBroadcast: true)`, `:2010`).
            let drop = if action == 2 {
                crate::chest::item_drop_dressers(style)
            } else {
                crate::chest::item_drop_chests(style, action == 4)
            };
            if drop > 0 {
                let rng = app.rngs.global();
                crate::world_sim::items::spawn_item_drop(&mut app.items, rng, drop, 1, x, y);
            }
            server.send(
                id,
                &S2c::ChestUpdates(net::ChestUpdates {
                    chest: -1,
                    ..u
                }),
            );
            return;
        }
        // The success: relay the index to EVERYONE (`:2015`).
        server.broadcast(&S2c::ChestUpdates(net::ChestUpdates {
            chest: result as i16,
            ..u
        }));
        app.world_key = None;
        return;
    }
    // The destroy arms (`:2019-2039`/:2057-2074/:2096-2115`): the tile must
    // be the object kind; decode the top-left, FindChest, KillTile, relay
    // the index when the tile died.
    let Some(t) = app.world.tile(x, y).copied() else {
        return;
    };
    if t.kind != destroy_kind {
        return; // the C# `goto default` falls through to nothing
    }
    if action == 3 {
        // Dresser: frameX % 54 / 18 columns, frameY % 36 row (`:2061-2065`).
        x -= i32::from(t.frame_x) % 54 / 18;
        if t.frame_y % 36 != 0 {
            y -= 1;
        }
    } else {
        // Chest: frameX % 36 / frameY % 36 (`:2023-2030`/:2100-2107).
        if t.frame_x % 36 != 0 {
            x -= 1;
        }
        if t.frame_y % 36 != 0 {
            y -= 1;
        }
    }
    let number = crate::chest::find_chest(&app.world_chests, x, y)
        .map(|i| i as i32)
        .unwrap_or(-1);
    crate::world_sim::editing::replicated_break(app, x, y, false, true);
    if !app.world.tile_active(x, y) {
        // `:2035` — the relay carries style 0 and the found index.
        server.broadcast(&S2c::ChestUpdates(net::ChestUpdates {
            action,
            x: x as i16,
            y: y as i16,
            style: 0,
            chest: number as i16,
        }));
    }
}

/// The `34` client half (`MessageBuffer.cs:2124-2157`): the direct
/// placement/destroy pass — `PlaceChestDirect`/`PlaceDresserDirect` write the
/// tiles + the store slot at the host's index, `DestroyChestDirect` removes
/// it; the refusal (-1) kills the tile. Sound 0 plays on a placement.
pub(crate) fn client_chest_updates(app: &mut App, u: net::ChestUpdates) {
    let (x, y) = (u.x as i32, u.y as i32);
    let style = u.style as i32;
    let id = u.chest as i32;
    match u.action {
        0 | 4 => {
            if id == -1 {
                crate::world_sim::editing::replicated_break(app, x, y, true, false);
                return;
            }
            app.audio
                .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
            let kind = if u.action == 0 { 21 } else { 467 };
            crate::chest::place_chest_direct(app, x, y, kind, style, id);
            app.world_key = None;
        }
        2 => {
            if id == -1 {
                crate::world_sim::editing::replicated_break(app, x, y, true, false);
                return;
            }
            app.audio
                .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
            crate::chest::place_dresser_direct(app, x, y, 88, style, id);
            app.world_key = None;
        }
        _ => {
            // The destroy default (`:2153-2156`).
            crate::chest::destroy_chest_direct(app, x, y, id);
            crate::world_sim::editing::replicated_break(app, x, y, true, false);
            app.world_key = None;
        }
    }
}

/// The `35 PlayerHeal` reader (`MessageBuffer.cs:2160-2177`): the host forces
/// the id to the sender (`:2163-2166`), raises `HealEffect` for a peer
/// (`:2168-2171` — the local player without ServerSideCharacter is skipped;
/// the port's host has no ServerSideCharacter, so a peer's heal always
/// spawns the text) and relays (`:2174`).
pub(crate) fn host_player_heal(app: &mut App, server: &net::Server, id: u32, amount: i16) {
    heal_effect_at_peer(app, id, amount);
    server.broadcast_except(
        &S2c::PlayerHeal(net::PlayerHeal {
            player_id: id as u8,
            amount,
        }),
        id,
    ); // `:2174`
}

/// The `35` client half: a peer's heal text (`Player.HealEffect`,
/// `Player.cs:4522-4529` — `CombatText.NewText` at the peer's position with
/// the HealLife colour, the raw amount).
pub(crate) fn client_player_heal(app: &mut App, player_id: u8, amount: i16) {
    heal_effect_at_peer(app, player_id as u32, amount);
}

/// `Player.HealEffect(amount)` for a remote peer — the heal combat text at
/// the peer's snapshot position (the port's `PlayerNet` is the C#
/// `Main.player[id]`). The draws ride the bare global stream (the message
/// handler's S1-E-21i precedent).
fn heal_effect_at_peer(app: &mut App, peer: u32, amount: i16) {
    let Some(p) = app.remote_players.get(&peer) else {
        return;
    };
    let (px, py) = (p.x, p.y);
    let text = (amount as i32).to_string();
    let size = match app.lookups.combat_text_font.as_ref() {
        Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
        None => [text.chars().count() as f32 * 8.0, 10.0],
    };
    let location = crate::combat_text::Rect {
        x: px,
        y: py,
        width: player::WIDTH,
        height: player::HEIGHT,
    };
    crate::combat_text::spawn_amount(
        &mut app.combat_texts,
        location,
        crate::combat_text::HEAL_LIFE,
        amount as i32,
        false,
        false,
        size,
        1.0,
        app.rngs.global(),
    );
}

/// The `36 SyncPlayerZone` reader (`MessageBuffer.cs:2178-2201`): the host
/// forces the id to the sender (`:2180-2184`), stores the six bytes
/// (`:2187-2192`), latches the `zone5[0]` rising edge into
/// `NPC.Spawner.SpawnFaelings` (`:2195-2198` — the live
/// `spawner::spawn_faelings` seam, W4-g48; the Shimmerfly 677 spawn is
/// roster-gated inside until the kind lands, the scan draws run verbatim)
/// and relays (`:2199`).
pub(crate) fn host_player_zone(app: &mut App, server: &net::Server, id: u32, z: net::PlayerZone) {
    let was_faeling = app
        .remote_players
        .get(&id)
        .is_some_and(|p| p.zones[4] & 1 != 0);
    if let Some(p) = app.remote_players.get_mut(&id) {
        p.zones = [z.zone1.0, z.zone2.0, z.zone3.0, z.zone4.0, z.zone5.0];
        p.town_npcs = z.town_npcs;
        // The c27 whole-struct mirror (the render/zone consumers' copy).
        p.zone = net::PlayerZone { player_id: id as u8, ..z };
    }
    // `:2195-2198` — the faeling edge: `NPC.Spawner.SpawnFaelings(player16)`
    // on the zone5[0] rising edge (W4-g48 — the seam is live; the Shimmerfly
    // 677 kind is roster-gated inside, the scan's draws run verbatim either
    // way, `spawner::spawn_faelings`). The peer's snapshot position stands
    // in for the C# `Main.player[num129]`; the draws ride the bare global
    // stream (the message handler's S1-E-21i precedent).
    let now_faeling = z.zone5.0 & 1 != 0;
    if !was_faeling && now_faeling {
        if let Some(p) = app.remote_players.get(&id) {
            let (px, py) = (p.x, p.y);
            let center = (px + player::WIDTH * 0.5, py + player::HEIGHT * 0.5);
            let world = &app.world;
            let npcs = &mut app.npcs;
            let rng = app.rngs.global();
            let spawned = crate::spawner::spawn_faelings(world, npcs, (px, py), center, rng);
            if spawned > 0 {
                app.world_key = None;
            }
        }
    }
    server.broadcast_except(
        &S2c::PlayerZone(net::PlayerZone {
            player_id: id as u8,
            ..z
        }),
        id,
    ); // `:2199`
}

/// The `36` client half: a peer's zone words land in the snapshot store.
pub(crate) fn client_player_zone(app: &mut App, z: net::PlayerZone) {
    if let Some(p) = app.remote_players.get_mut(&(z.player_id as u32)) {
        p.zones = [z.zone1.0, z.zone2.0, z.zone3.0, z.zone4.0, z.zone5.0];
        p.town_npcs = z.town_npcs;
        // The c27 whole-struct mirror (the wire id kept, `:2187-2192`).
        p.zone = z;
    }
}

/// The `40 SyncTalkNPC` reader (`MessageBuffer.cs:2256-2270`): the host
/// forces the id to the sender (`:2258-2262`), applies `SetTalkNPC`
/// (`Player.cs:4375-4390` — the talk index + the host-side bestiary chat
/// registration, `:4379-4382`) and relays (`:2267`).
pub(crate) fn host_talk_npc(app: &mut App, server: &net::Server, id: u32, npc: i16) {
    if let Some(p) = app.remote_players.get_mut(&id) {
        p.talk_npc = if npc >= 0 { Some(npc) } else { None };
    }
    // `Player.SetTalkNPC`'s `netMode != 1` bestiary arm (`Player.cs:4379-
    // 4382`): the host registers the chat start with the NPC's credit.
    if npc >= 0 {
        if let Some(n) = app.npcs.get(npc as usize) {
            let entry = crate::bestiary::try_entry_for_npc(n.kind, n.slime);
            if let Some(entry) = entry {
                app.bestiary.set_was_chat_with_directly(entry.credit_id);
            }
        }
    }
    server.broadcast_except(
        &S2c::TalkNpc(net::TalkNpc {
            player_id: id as u8,
            npc,
        }),
        id,
    ); // `:2267`
}

/// The `40` client half: a peer's talk index lands in the snapshot store
/// (the client's `SetTalkNPC` bestiary arm is `netMode != 1`-gated off).
pub(crate) fn client_talk_npc(app: &mut App, player_id: u8, npc: i16) {
    if let Some(p) = app.remote_players.get_mut(&(player_id as u32)) {
        p.talk_npc = if npc >= 0 { Some(npc) } else { None };
    }
}

/// The `20 SectionRange` body for a `SendTileSquare(whoAmi, x, y, w, h)`
/// (`NetMessage.cs:2627-2630`) — the live tiles read back out.
pub(crate) fn tile_square(app: &App, x: i32, y: i32, w: i32, h: i32) -> net::SectionRange {
    let mut records = Vec::with_capacity((w * h) as usize);
    for dy in 0..h {
        for dx in 0..w {
            records.push(app.tile_record_at(x + dx, y + dy));
        }
    }
    net::SectionRange {
        x: x as i16,
        y: y as i16,
        w: w as u8,
        h: h as u8,
        change_type: 0,
        records,
    }
}

/// The `17` kind-21 `WorldGen.ReplaceTile` apply (`WorldGen.cs:62558-62580`,
/// W4-g48 — the `edit::replace` family). The C# tail order: dust → sound →
/// `KillTile_DropItems(includeLargeObjectDrops: true)` →
/// `AttemptFossilShattering` → the rigged-chest switch → the replacement.
/// `server: Some` is the host (the drops spawn, the fossil chain runs with
/// its `17` relays, the rigged chest fires `HitSwitch` + `59`); `None` is
/// the client (the `netMode == 1` gates skip all four — the loot arrives as
/// `21` messages). Returns whether the replacement ran (the gates refused
/// otherwise); the caller's relay runs either way (`:1277`).
fn replace_tile_apply(
    app: &mut App,
    server: Option<&net::Server>,
    x: i32,
    y: i32,
    target: u16,
    style: i32,
    host: bool,
) -> bool {
    let Some(mut plan) = edit::replace_tile_prepare(&app.world, x, y, target) else {
        return false;
    };
    let (ax, ay) = (plan.x, plan.y);
    let old = plan.old;
    // `KillTile_GetTileDustAmount(fail: false)` + the MakeTileDust burst
    // (`:62570-62574`) — the port has no per-tick dust layer (the recorded
    // visual seam, the kind-0 precedent); the count is derived, not spawned.
    let _dust = crate::collision::tile_dust_amount(old.kind, false, old.frame_x);
    // `KillTile_PlaySounds(x, y, fail: false, tileSafely)` (`:62575`) — the
    // ORIGINAL cell's cue table, on the moved anchor.
    let sounds = edit::kill_tile_sound(old.kind, old.frame_x, false);
    let (px, py) = (ax as f32 * TILE_PX, ay as f32 * TILE_PX);
    for cue in [sounds.primary, sounds.extra, sounds.shatter]
        .into_iter()
        .flatten()
    {
        if let Some((name, volume)) = crate::world_sim::editing::kill_cue(cue) {
            app.audio.play_at(name, px, py, volume);
        }
    }
    if host {
        // `KillTile_DropItems(x, y, tileSafely, includeLargeObjectDrops:
        // true)` (`:62576`, `WorldGen.cs:63948-63962`): the large-object
        // switch (chest/dresser/campfire, `:63973-63986`) overrides the
        // per-cell drop, then the per-cell slice runs — the SAME
        // `break_drop_events` hit-cell path the break uses (no held item:
        // no staff/axe).
        let large: Option<u16> = match old.kind {
            21 | 467 => Some(crate::chest::item_drop_chests(
                i32::from(old.frame_x) / 36,
                old.kind == 467,
            )),
            88 => Some(crate::chest::item_drop_dressers(i32::from(old.frame_x) / 54)),
            215 => Some(crate::chest::campfire_item_drop(i32::from(old.frame_x) / 54)),
            _ => None,
        };
        let hit = crate::world_sim::editing::HitCell {
            x: ax,
            y: ay,
            kind: old.kind,
            frame_x: old.frame_x,
            frame_y: old.frame_y,
        };
        let ctx = crate::world_sim::editing::drop_ctx(app);
        let mut rolls: Vec<(u16, u32)> = Vec::new();
        if let Some(id) = large.filter(|id| *id > 0) {
            rolls.push((id, 1));
        } else {
            let kill = edit::TileKill { kind: old.kind, ..Default::default() };
            let (events, _portals) = crate::world_sim::editing::break_drop_events(
                &kill,
                &app.world,
                Some(hit),
                false,
                0,
                app.rngs.global(),
                &ctx,
            );
            rolls.extend(events.into_iter().map(|(id, n, _, _)| (id, n)));
        }
        for (id, count) in rolls {
            let rng = app.rngs.global();
            crate::world_sim::items::spawn_item_drop(&mut app.items, rng, id, count, ax, ay);
        }
        // `AttemptFossilShattering(x, y, tileSafely, fail: false)` (`:62577`,
        // `WorldGen.cs:63484-63513`): the desert fossil 404's 3x3 chain —
        // genRand.Next(15) per neighbour (4 when unsupported / the !fail
        // top-centre), a 0 rolls the KillTile(noItem: true) + the `17`
        // relay to EVERYONE. The `fossilBreak` re-entrancy guard never
        // binds here (the only fossil caller on the handler; the chain's
        // own kills are 404s whose KillTile runs the fossil arm again in
        // C# — guarded off; the port's replicated_break has no fossil
        // recursion, order-identical).
        if old.kind == 404 {
            for k in ax - 1..=ax + 1 {
                for l in ay - 1..=ay + 1 {
                    let mut max = 15;
                    if !edit::solid_tile(&app.world, k, l + 1) {
                        max = 4;
                    } else if k == ax && l == ay - 1 {
                        max = 4; // `!fail` — the reader passes false
                    }
                    let fossil = app
                        .world
                        .tile(k, l)
                        .is_some_and(|t| t.active() && t.kind == 404);
                    if (k != ax || l != ay) && fossil {
                        // The genRand stand-in: one global throw seeds the
                        // FastRandom (the break_drop_events convention).
                        let seed = app.rngs.global().next() as u64;
                        let mut gen = tr_core::tile_light::FastRandom::new(seed);
                        if gen.next(max) == 0 {
                            crate::world_sim::editing::replicated_break(app, k, l, true, true);
                            if let Some(server) = server {
                                // `SendData(17, -1, -1, null, 0, k, l)`
                                // (`:63507`) — kind 0, value/style 0.
                                server.broadcast(&S2c::TileEdit(net::TileEdit {
                                    kind: 0,
                                    x: k as i16,
                                    y: l as i16,
                                    value: 0,
                                    style: 0,
                                }));
                            }
                        }
                    }
                }
            }
        }
        // The rigged-chest probe (`:62586-62590`): BEFORE the swap (the
        // frame check reads the OLD tiles). `IsChestRigged` = the 467
        // frameX/36 == 4 Dead Man's Chest (`netglue::is_rigged`).
        if tr_core::tiledata::SET_BASIC_CHEST.has(target)
            && crate::app::netglue::is_rigged(&app.world, ax, ay)
        {
            app.wiring.hit_switch(&mut app.world, ax, ay);
            if let Some(server) = server {
                server.broadcast(&S2c::HitSwitch(net::HitSwitch {
                    x: ax as i16,
                    y: ay as i16,
                }));
            }
        }
    }
    // The seed-flag reads (`Main.onlyShimmerOceanWorlds` —
    // `SpecialSeedFeatures.cs:93` / `Main.cs:2674-2684`; the plain-tenth
    // arm `:62637`).
    use tr_core::seeds::SeedFlags;
    let s = app.world.seeds;
    let rainbow_sand = s.contains(SeedFlags::DRUNK)
        && s.contains(SeedFlags::TENTH_ANNIVERSARY)
        && !s.contains(SeedFlags::REMIX)
        && !s.contains(SeedFlags::ZENITH)
        && !s.contains(SeedFlags::NOT_THE_BEES);
    let tenth_plain = s.contains(SeedFlags::TENTH_ANNIVERSARY)
        && !s.contains(SeedFlags::REMIX)
        && !s.contains(SeedFlags::NOT_THE_BEES)
        && !s.contains(SeedFlags::DONT_STARVE);
    let holds = holds_items(&app.world_chests);
    edit::replace_tile_commit(
        &mut app.world,
        &mut plan,
        target,
        style,
        holds,
        rainbow_sand,
        tenth_plain,
    );
    // The EliminateNaturalExtras kills' records — always empty (the C#
    // `_Single` discards them); the resolution shape stays uniform.
    if host {
        spawn_object_drops(app, std::mem::take(&mut plan.drops));
    }
    app.world_key = None;
    true
}
