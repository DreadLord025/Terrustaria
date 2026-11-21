//! W4-g48 (S1-I-4 apply arms, lane 1 — `MessageBuffer.GetData` ids 1-40):
//! the parked wave-A/B/I dispatcher arms wired in the C# `GetData` case
//! order. Each fn is one case's apply half; the dispatchers in `netglue.rs`
//! call them and own the relay sends (`NetMessage.TrySendData` sites).
//!
//! The message handler runs outside every `SwapRandom` (`Main.cs:16895`'s
//! NpcStrike precedent — S1-E-21i), so every RNG draw here rides the bare
//! global stream (`app.rngs.global()`).

use tr_core::{edit, TILE_PX};

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
    let (x0, y0) = (x as f32 * TILE_PX, y as f32 * TILE_PX);
    let (x1, y1) = (x0 + TILE_PX, y0 + TILE_PX);
    let overlaps = |px: f32, py: f32, w: f32, h: f32| {
        x0 < px + w && x1 > px && y0 < py + h && y1 > py
    };
    if let Some(p) = app.player.as_ref() {
        if !p.dead && overlaps(p.x, p.y, player::WIDTH, p.height) {
            return true;
        }
    }
    for peer in app.remote_players.values() {
        if !peer.dead && overlaps(peer.x, peer.y, player::WIDTH, player::HEIGHT) {
            return true;
        }
    }
    app.npcs.iter().any(|n| {
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
            // track framing is unported (`minecart.rs` module doc); the
            // relay still runs (the C# falls through to `:1277`).
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
            // — unported (`liquid.rs:709` records the gap); the relay runs.
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
        15 | 21 => {}
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
