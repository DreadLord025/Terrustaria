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

/// `Main.teamColor[team]` (`Main.cs:6657-6662`) — the PVP chat line's colour
/// (`MessageBuffer.cs:1864`). An out-of-range team reads white (the C# would
/// throw; the wire byte is 0..5 in practice).
fn team_color(team: u8) -> [u8; 3] {
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
/// the normalised byte (`:1330`). The trapdoor/tall-gate shifts
/// (`WorldGen.ShiftTrapdoor` `:51447`, `ShiftTallGate` `:51723`) are unported
/// (the `npc_ai_town.rs` cited skip) — those actions relay without a world
/// change, exactly like the town-NPC gate arm.
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
            // `:1319`) — unported (the trapdoor 386/387 shift, the
            // `npc_ai_town.rs` cited skip); the relay still runs.
        }
        4 | 5 => {
            // `WorldGen.ShiftTallGate(x, y, closing, forced: true)` (`:1322`/
            // `:1325`) — unported (the tall gate 388/389, the same skip).
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
