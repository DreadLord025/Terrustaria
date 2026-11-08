//! The App's network glue — the two handshake state machines, the event pump,
//! the 10 Hz state tick and the in-process world install behind them.
//!
//! The machines are ports of the C# sides: the client's
//! `Netplay.Connection.State` walk (`Netplay.cs:459-544` plus the state
//! changes in the `MessageBuffer.cs` cases) and the server's per-slot states
//! with the `GetData` accept gate (`MessageBuffer.cs:148-172`) and the
//! handshake cases (`MessageBuffer.cs:184-2231`). The world blob arrives as
//! the internal `15 WorldChunk` stream (M6-NET-3 replaces the producer with
//! `10 TileBlock` sections); the install happens in-process — the C#'s
//! `relaunch`-shaped rebuild is unnecessary because every world-derived
//! system here is rebuildable without touching the renderer.

use std::time::Instant;

use glam::Vec2;
use tr_core::{save, World, TILE_PX};

use crate::app::{App, LiquidSystem, NetRole};
use crate::item;
use crate::map;
use crate::menu;
use crate::net::{self, id, slot_state, C2s, S2c};
use crate::player;
use crate::world_sim::editing::apply_edit;

/// The `GetData` exception whitelist (`MessageBuffer.cs:161`): the messages a
/// not-yet-in-world slot may still send (16 PlayerHP, 38 Password, …).
const GATE_WHITELIST: [u8; 9] = [93, 16, 42, 50, 38, 68, 147, 161, 12];

/// The `Lang.mp` kick texts the C# uses at these gates
/// (`MessageBuffer.cs:160/167/171` = `LegacyMultiplayer.2`, `:218` = `.4`,
/// `:2231` = `.1`, `:377-386` = the name keys).
const TXT_INVALID_OP: &str = "Invalid operation at this state.";
const TXT_WRONG_VERSION: &str = "You are not using the same version as this server.";
const TXT_BAD_PASSWORD: &str = "Incorrect password.";
const TXT_NAME_EMPTY: &str = "Name is empty.";
const TXT_NAME_TOO_LONG: &str = "Name is too long.";
const TXT_NAME_TAKEN: &str = "That name is taken.";

impl App {
    /// Drains network events without borrowing `self.net` across handlers, and
    /// keeps the idle timers alive (`TimeOutTimer`, `MessageBuffer.cs:127-134`).
    pub(crate) fn poll_net(&mut self) {
        let mut server_events = Vec::new();
        let mut client_events = Vec::new();
        match &self.net {
            NetRole::Host(server) => {
                while let Ok(ev) = server.events.try_recv() {
                    server_events.push(ev);
                }
            }
            NetRole::Client(client) => {
                while let Ok(ev) = client.events.try_recv() {
                    client_events.push(ev);
                }
            }
            NetRole::Off => {}
        }
        // The client's very first tick sends `1 Connect` (`Netplay.cs:469-475`).
        let connect_now = match &self.net {
            NetRole::Client(client) if self.net_state == net::client_state::NOT_CONNECTED => {
                self.net_state = net::client_state::CONNECT_SENT;
                Some(client)
            }
            _ => None,
        };
        if let Some(client) = connect_now {
            client.send(&C2s::Connect(net::CONNECT_VERSION.to_string()));
        }
        let role = std::mem::replace(&mut self.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            for ev in server_events {
                self.host_event(server, ev);
            }
            // The silent-peer sweep (`Main.cs:64610-64618`'s timeout terminate).
            if server.client_count() > 0 {
                for id in server.sweep_timeouts() {
                    self.host_slot_left(server, id);
                }
            }
        }
        if let NetRole::Client(client) = &role {
            for ev in client_events {
                self.client_event(client, ev);
            }
            // The client's own timeout (`Main.cs:64540-64544`).
            if self.net_state > net::client_state::NOT_CONNECTED
                && self.net_last_recv.elapsed() >= net::IDLE_TIMEOUT
            {
                self.net_disconnect("Timed out");
            }
        }
        // Restore the swapped-out role unless a handler dropped it (a
        // disconnect during the dispatch).
        if !self.net_dropped {
            self.net = role;
        }
        self.net_dropped = false;
    }

    /// The `GetData` accept gate (`MessageBuffer.cs:148-172`): a slot may only
    /// speak for its state. Kicks carry the same `Lang.mp` texts.
    fn host_gate(server: &net::Server, id: u32, msg: &C2s) -> bool {
        let st = server.state(id);
        let m = msg.msg_id();
        if st == slot_state::AWAITING_PASSWORD && m != id::PASSWORD {
            server.kick(id, TXT_BAD_PASSWORD);
            return false;
        }
        if st < slot_state::IN_WORLD && m > 12 && !GATE_WHITELIST.contains(&m) {
            server.kick(id, TXT_INVALID_OP);
            return false;
        }
        if st == slot_state::FRESH && m != id::CONNECT {
            server.kick(id, TXT_INVALID_OP);
            return false;
        }
        true
    }

    /// Host side: the handshake dispatch plus the in-world requests.
    fn host_event(&mut self, server: &net::Server, ev: net::ServerEvent) {
        let net::ServerEvent::Frame { id, msg } = ev else {
            // A malformed frame is a protocol error, like the C# throw inside
            // GetData; a closed peer is cleaned up below.
            match ev {
                net::ServerEvent::BadFrame { id } => {
                    log::warn!("bad frame from slot {id}");
                    server.kick(id, TXT_INVALID_OP);
                }
                net::ServerEvent::Closed { id } => self.host_slot_left(server, id),
                net::ServerEvent::Frame { .. } => unreachable!(),
            }
            return;
        };
        if !Self::host_gate(server, id, &msg) {
            log::warn!("slot {id} kicked by the accept gate (msg {})", msg.msg_id());
            return;
        }
        match msg {
            // `case 1` — the version literal (`MessageBuffer.cs:199-214`).
            C2s::Connect(version) => {
                if server.state(id) != slot_state::FRESH {
                    return; // a duplicate Connect is ignored (`:200-202`)
                }
                if version != net::CONNECT_VERSION {
                    server.kick(id, TXT_WRONG_VERSION);
                    return;
                }
                if server.password().is_empty() {
                    server.set_state(id, slot_state::CONNECTED);
                    server.send(
                        id,
                        &S2c::PlayerId {
                            player_id: id as u8,
                        },
                    );
                } else {
                    server.set_state(id, slot_state::AWAITING_PASSWORD);
                    server.send(id, &S2c::PasswordRequest);
                }
            }
            // `case 38` — the password answer (`MessageBuffer.cs:2218-2231`).
            C2s::Password(pw) => {
                if pw == server.password() {
                    server.set_state(id, slot_state::CONNECTED);
                    server.send(
                        id,
                        &S2c::PlayerId {
                            player_id: id as u8,
                        },
                    );
                } else {
                    server.kick(id, TXT_BAD_PASSWORD);
                }
            }
            // `case 4` — the identity; the name checks (`MessageBuffer.cs:360-396`).
            C2s::PlayerInfo(pi) => {
                if pi.name.is_empty() {
                    server.kick(id, TXT_NAME_EMPTY);
                    return;
                }
                if pi.name.chars().count() > net::NAME_LEN {
                    server.kick(id, TXT_NAME_TOO_LONG);
                    return;
                }
                let taken = server
                    .slot_ids()
                    .iter()
                    .any(|&other| other != id && server.slot_name(other) == pi.name);
                if taken {
                    server.kick(id, TXT_NAME_TAKEN);
                    return;
                }
                server.set_slot_name(id, pi.name.clone());
                log::info!("slot {id} announced as {}", pi.name);
            }
            // `case 5` — one inventory slot. The server keeps no peer
            // inventory state yet (it lands with M6-NET-2's peer state), so
            // the message is accepted and dropped — see docs/ANALYSIS.md.
            C2s::Inventory { .. } => {}
            // `case 6` — the world request (`MessageBuffer.cs:462-472`).
            C2s::WorldRequest => {
                if server.state(id) == slot_state::CONNECTED {
                    server.set_state(id, slot_state::WORLD_REQUESTED);
                    let info = self.world_info();
                    server.send(id, &S2c::WorldInfo(Box::new(info)));
                }
            }
            // `case 8` — the spawn request; the sections stream next
            // (`MessageBuffer.cs:671-806`, here the internal `15` chunks).
            C2s::SpawnRequest(s) => {
                if server.state(id) == slot_state::WORLD_REQUESTED {
                    let (mut sx, mut sy) = (s.x, s.y);
                    let (w, h) = (self.world.width, self.world.height);
                    // The C# range check falls back to the default spawn
                    // (`MessageBuffer.cs:676-682`).
                    if !(10..w.saturating_sub(10)).contains(&sx)
                        || !(10..h.saturating_sub(10)).contains(&sy)
                    {
                        sx = self.world.spawn_x;
                        sy = self.world.spawn_y;
                    }
                    log::info!("slot {id} spawn request {sx},{sy}");
                    server.set_state(id, slot_state::SPAWN_REQUESTED);
                    let blob = compressed_world(&self.world);
                    server.send_world(id, &blob);
                }
            }
            // `case 12` — the player spawned in: the slot is complete and the
            // join bundle goes out (`MessageBuffer.cs:925-929`,
            // `SyncConnectedPlayer` `NetMessage.cs:2806-2822`).
            C2s::PlayerSpawn(s) => {
                if server.state(id) == slot_state::SPAWN_REQUESTED {
                    server.set_state(id, slot_state::IN_WORLD);
                    self.host_slot_join_complete(server, id, s);
                }
            }
            C2s::PlayerActive { active: true, .. } => {
                // The peer is in-world (`case 14`); make sure it has a
                // remote-player entry (the identity usually arrives first).
                if server.state(id) >= slot_state::IN_WORLD {
                    self.remote_players
                        .entry(id)
                        .or_insert_with(|| net::PlayerNet {
                            id,
                            name: server.slot_name(id),
                            ..Default::default()
                        });
                }
            }
            C2s::PlayerActive { active: false, .. } => {
                if server.state(id) >= slot_state::IN_WORLD {
                    self.remote_players.remove(&id);
                    server.broadcast_except(&S2c::Left { id }, id);
                }
            }
            // `case 16` — the health report; the server forces the subject to
            // the sender (`MessageBuffer.cs:1100-1103`).
            C2s::PlayerHp { hp, max_hp, .. } => {
                if let Some(p) = self.remote_players.get_mut(&id) {
                    p.hp = hp as i32;
                    p.max_hp = (max_hp as i32).max(20);
                }
            }
            // ---- internal, in-world only ----
            C2s::Player(p) => {
                self.remote_players.insert(p.id, p.clone());
                // The C# never echoes a snapshot back to its sender
                // (`SendData`'s `ignoreClient` routing, `NetMessage.cs:1692-1826`).
                server.broadcast_except(&S2c::Player(p), id);
            }
            C2s::Edit { x, y, kind, tile } => {
                log::info!("net edit kind {kind} at {x},{y} from {id}");
                apply_edit(&mut self.world, x, y, kind, tile);
                self.world_key = None;
                server.broadcast(&S2c::Edit { x, y, kind, tile });
            }
            C2s::Pickup { drop } => {
                let center = self
                    .remote_players
                    .get(&id)
                    .map(|p| Vec2::new(p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5));
                if let Some(center) = center {
                    if let Some(c) = self.items.get(drop as usize).map(|it| it.center()) {
                        if (c - center).length() < 60.0 {
                            self.items.remove(drop as usize);
                            server.broadcast(&S2c::Items(
                                self.items
                                    .iter()
                                    .enumerate()
                                    .map(|(i, it)| net::ItemNet {
                                        id: i as u32,
                                        item: it.kind,
                                        count: it.count,
                                        x: it.pos.x,
                                        y: it.pos.y,
                                    })
                                    .collect(),
                            ));
                        }
                    }
                }
            }
        }
    }

    /// The join bundle when a slot reaches `IN_WORLD` (the port's
    /// `SyncConnectedPlayer`, `NetMessage.cs:2806-2822`): the newcomer learns
    /// the peers, the peers learn the newcomer.
    fn host_slot_join_complete(&mut self, server: &net::Server, id: u32, _s: net::PlayerSpawn) {
        for p in self.remote_players.values() {
            if p.id != id {
                server.send(id, &S2c::Player(p.clone()));
            }
        }
        let name = server.slot_name(id);
        self.remote_players.insert(
            id,
            net::PlayerNet {
                id,
                name: name.clone(),
                ..Default::default()
            },
        );
        let joined = S2c::Joined { id, name };
        server.broadcast(&joined);
        log::info!("slot {id} fully joined");
    }

    /// A slot went away: clean up the game-side record (`MessageBuffer.cs`'s
    /// disconnect arm + `SyncOnePlayer`'s inactive broadcast).
    fn host_slot_left(&mut self, server: &net::Server, id: u32) {
        let was_in_world = self.remote_players.remove(&id).is_some();
        if was_in_world {
            server.broadcast(&S2c::Left { id });
            log::info!("slot {id} left");
        }
    }

    /// Client side: the state machine walk, the world install and the
    /// in-world relays.
    fn client_event(&mut self, client: &net::Client, ev: net::ClientEvent) {
        let net::ClientEvent::Frame(msg) = ev else {
            match ev {
                net::ClientEvent::BadFrame => self.net_disconnect("bad frame from host"),
                net::ClientEvent::Closed => self.net_disconnect("connection closed"),
                net::ClientEvent::Frame(_) => unreachable!(),
            }
            return;
        };
        self.net_last_recv = Instant::now();
        match msg {
            // `case 37` — the server wants the password; auto-pass is on for
            // the join the user started (`MessageBuffer.cs:2203-2216`).
            S2c::PasswordRequest => {
                let pw = self
                    .join_target
                    .as_ref()
                    .map(|(_, pw)| pw.clone())
                    .unwrap_or_default();
                client.send(&C2s::Password(pw));
            }
            // `case 3` — our id, then the player-data push ends with
            // `6 WorldRequest` (`MessageBuffer.cs:230-282`).
            S2c::PlayerId { player_id } => {
                self.local_id = player_id as u32;
                log::info!("joined with id {player_id}");
                self.client_send_identity(client);
                self.net_state = net::client_state::WORLD_REQUESTED;
            }
            // `case 7` — the world header (`MessageBuffer.cs:473-658`), then the
            // state walk continues: 4 → 5 (`Netplay.cs:484-505`, the port has
            // no `clearWorld` arm to run first) → `FindSpawn` + `8
            // SpawnRequest` (`Netplay.cs:511-516`). The spawn is the header's
            // spawn tile — what `FindSpawn` computes on the cleared world.
            S2c::WorldInfo(info) => {
                let (sx, sy) = (info.spawn_tile_x as i32, info.spawn_tile_y as i32);
                self.net_info = Some(*info);
                self.net_state = net::client_state::WORLD_INFO;
                client.send(&C2s::SpawnRequest(net::SpawnRequest {
                    x: sx,
                    y: sy,
                    team: 0,
                }));
                self.net_state = net::client_state::SPAWN_REQUESTED;
            }
            // `case 9` — the load progress (`MessageBuffer.cs:143-146` counts
            // received messages while `StatusMax > 0`).
            S2c::Status { text, max } => {
                self.net_status_text = text;
                self.net_status_max = max;
                self.net_status_count = 0;
                if let Some(m) = self.menu.as_mut() {
                    m.connecting = true;
                }
            }
            // `15 WorldChunk` — one slice of the blob; indexed so a reorder
            // cannot corrupt the install.
            S2c::WorldChunk { index, total, data } => {
                let index = index.max(0) as usize;
                let total = total.max(1) as usize;
                if self.net_chunks.len() < total {
                    self.net_chunks.resize(total, None);
                }
                if index < total {
                    self.net_chunks[index] = Some(data);
                }
                if self.net_status_max > 0 {
                    self.net_status_count += 1;
                }
            }
            // `case 49` — the world stream is done (`MessageBuffer.cs:2448-2452`):
            // install in place (the port's `Player.Spawn(SpawningIntoWorld)`)
            // and announce the spawn with `12 PlayerSpawn`, which completes
            // the server slot (`MessageBuffer.cs:925-929`).
            S2c::WorldComplete => {
                if self.net_state == net::client_state::SPAWN_REQUESTED {
                    let blob: Vec<u8> = self.net_chunks.drain(..).flatten().flatten().collect();
                    self.net_chunks.clear();
                    if self.client_install_world(&blob) {
                        let (sx, sy) = (self.world.spawn_x as i16, self.world.spawn_y as i16);
                        self.net_state = net::client_state::IN_WORLD;
                        client.send(&C2s::PlayerSpawn(net::PlayerSpawn {
                            x: sx,
                            y: sy,
                            respawn_timer: 0,
                            deaths_pve: 0,
                            deaths_pvp: 0,
                            team: 0,
                            spawn_context: 0,
                        }));
                    } else {
                        self.net_disconnect("the world failed to decode");
                    }
                }
            }
            // Other players' identity/health (the C# relays 4/16/14 to all).
            S2c::PlayerInfo(pi) => {
                let peer = pi.player_id as u32;
                if peer != self.local_id {
                    self.remote_players
                        .entry(peer)
                        .or_insert_with(|| net::PlayerNet {
                            id: peer,
                            name: pi.name.clone(),
                            ..Default::default()
                        });
                    if let Some(p) = self.remote_players.get_mut(&peer) {
                        p.name = pi.name.clone();
                    }
                }
            }
            S2c::PlayerHp {
                player_id,
                hp,
                max_hp,
            } => {
                if let Some(p) = self.remote_players.get_mut(&(player_id as u32)) {
                    p.hp = hp as i32;
                    p.max_hp = (max_hp as i32).max(20);
                }
            }
            S2c::PlayerActive { player_id, active } => {
                if active && player_id as u32 != self.local_id {
                    self.remote_players
                        .entry(player_id as u32)
                        .or_insert_with(|| net::PlayerNet {
                            id: player_id as u32,
                            name: format!("Player {}", player_id),
                            ..Default::default()
                        });
                } else {
                    self.remote_players.remove(&(player_id as u32));
                }
            }
            // Disconnects: the C# sets statusText and reopens the menu
            // (`MessageBuffer.cs:215-222`).
            S2c::Disconnect(reason) => self.net_disconnect(&reason),
            // The join broadcast reaches the joiner too; the C# filters it by
            // slot on the read side, so we never list ourselves.
            S2c::Joined { id, name } => {
                if id != self.local_id {
                    self.remote_players
                        .entry(id)
                        .or_insert_with(|| net::PlayerNet {
                            id,
                            name,
                            ..Default::default()
                        });
                }
            }
            S2c::Left { id } => {
                self.remote_players.remove(&id);
            }
            S2c::Player(p) => {
                self.remote_players.insert(p.id, p);
            }
            S2c::Time { time, day } => {
                self.world.time = time;
                self.world.day_time = day;
            }
            S2c::Edit { x, y, kind, tile } => {
                apply_edit(&mut self.world, x, y, kind, tile);
                self.world_key = None;
            }
            S2c::Npcs(list) => {
                self.npcs = list
                    .iter()
                    .filter_map(|n| npc_from_net(&self.world, n))
                    .collect();
            }
            S2c::Items(list) => {
                self.items = list
                    .iter()
                    .map(|it| {
                        let mut d =
                            item::ItemDrop::new(it.item, it.count, 0, it.x, it.y, 0.0, 0.0, 0.0);
                        d.age = 60.0;
                        d
                    })
                    .collect();
            }
            // Unused in this milestone (the chunk stream replaces them).
            S2c::Inventory { .. } => {}
        }
    }

    /// The player-data push (`case 3`'s client half, `MessageBuffer.cs:237-282`):
    /// the identity, the health and the inventory arrays, adapted to the
    /// port's slot layout (`App.inventory` + the `EquipSlots`), then
    /// `6 WorldRequest`.
    fn client_send_identity(&self, client: &net::Client) {
        let me = self.local_id as u8;
        client.send(&C2s::PlayerInfo(net::PlayerInfo::with_name(
            me,
            &self.player_name,
        )));
        // The health (`Main.cs`'s 16 in the same push).
        let (hp, max_hp) = self
            .player
            .as_ref()
            .map(|p| (p.hp, p.max_hp))
            .unwrap_or((100, 100));
        client.send(&C2s::PlayerHp {
            player_id: me,
            hp: hp.clamp(0, i16::MAX as i32) as i16,
            max_hp: max_hp.clamp(0, i16::MAX as i32) as i16,
        });
        // The 59 grid slots, in `PlayerItemSlotID.Inventory0..58` order.
        for (i, slot) in self.inventory.iter().enumerate() {
            client.send(&C2s::Inventory {
                slot: i as i16,
                item: self.net_item(slot, self.favorite_slots.contains(&i)),
            });
        }
        // The armour, dye and misc-dye rows (`MessageBuffer.cs:264-276`'s
        // array order), then the four banks and the trash slot.
        if let Some(p) = self.player.as_ref() {
            for i in 0..p.equipped.len() {
                client.send(&C2s::Inventory {
                    slot: player_item_slot::ARMOR_0 + i as i16,
                    item: net::NetItem {
                        kind: p.equipped[i] as i16,
                        stack: 1,
                        prefix: p.equipped_prefix[i].min(255) as u8,
                        favorited: false,
                    },
                });
            }
            for i in 0..p.dye_slots.len() {
                client.send(&C2s::Inventory {
                    slot: player_item_slot::DYE_0 + i as i16,
                    item: net::NetItem {
                        kind: p.dye_slots[i] as i16,
                        stack: 1,
                        prefix: 0,
                        favorited: false,
                    },
                });
            }
            for i in 0..p.misc_dye_slots.len() {
                client.send(&C2s::Inventory {
                    slot: player_item_slot::MISC_DYE_0 + i as i16,
                    item: net::NetItem {
                        kind: p.misc_dye_slots[i] as i16,
                        stack: 1,
                        prefix: 0,
                        favorited: false,
                    },
                });
            }
            for (b, vault) in p.banks.iter().enumerate() {
                for (i, slot) in vault.iter().enumerate() {
                    client.send(&C2s::Inventory {
                        slot: player_item_slot::bank_1_0(b) + i as i16,
                        item: self.net_item(slot, false),
                    });
                }
            }
        }
        // The misc equips (`PlayerItemSlotID.Misc0..`) ride `App.equip`.
        for (i, slot) in self.equip.misc_equips.iter().enumerate() {
            client.send(&C2s::Inventory {
                slot: player_item_slot::MISC_0 + i as i16,
                item: self.net_item(slot, false),
            });
        }
        // The trash slot (`PlayerItemSlotID.TrashItem`).
        client.send(&C2s::Inventory {
            slot: player_item_slot::TRASH,
            item: self.net_item(&self.equip.trash, false),
        });
        // `case 6` — empty body.
        client.send(&C2s::WorldRequest);
    }

    fn net_item(&self, slot: &item::ItemSlot, favorited: bool) -> net::NetItem {
        net::NetItem {
            kind: slot.kind as i16,
            // The C# clamps the stack into the i16 (`MessageBuffer.cs:400`).
            stack: slot.count.min(i16::MAX as u32) as i16,
            prefix: slot.prefix.min(255) as u8,
            favorited,
        }
    }

    /// The in-process world install: every world-derived system rebuilds here
    /// without touching the renderer (the C# `WorldGen.clearWorld` +
    /// `Main.Map.Load` arm of `Netplay.cs:484-505`).
    fn client_install_world(&mut self, blob: &[u8]) -> bool {
        let Some(world) = decompressed_world(blob) else {
            log::warn!("received a world that failed to decode");
            return false;
        };
        // The stream must match the `7 WorldInfo` header the client accepted.
        if let Some(info) = self.net_info.as_ref() {
            if info.max_tiles_x as i32 != world.width || info.max_tiles_y as i32 != world.height {
                log::warn!(
                    "world size mismatch: header {}x{}, stream {}x{}",
                    info.max_tiles_x,
                    info.max_tiles_y,
                    world.width,
                    world.height
                );
            }
        }
        self.world = world;
        self.liquids = LiquidSystem::new(&self.world);
        self.map = Some(map::WorldMap::new(&self.world));
        self.npcs.clear();
        self.items.clear();
        self.world_key = None;
        self.net_chunks.clear();
        self.net_info = None;
        if self.player.is_some() {
            // `FindSpawn` + `Player.Spawn(SpawningIntoWorld)` — the port's
            // spawn is the world header's spawn tile (`Player.cs:594`).
            self.player = Some(player::Player::spawn(&self.world));
        }
        if self.menu.is_some() {
            self.close_menu();
        }
        self.front = None;
        log::info!(
            "world installed: {}x{}",
            self.world.width,
            self.world.height
        );
        true
    }

    /// Builds the `7 WorldInfo` body from the live world (`App.world`).
    fn world_info(&self) -> net::WorldInfo {
        let w = &self.world;
        net::WorldInfo {
            time: w.time as i32,
            day_time: w.day_time,
            blood_moon: w.blood_moon,
            eclipse: w.eclipse,
            moon_phase: w.moon_phase,
            max_tiles_x: w.width as i16,
            max_tiles_y: w.height as i16,
            spawn_tile_x: w.spawn_x as i16,
            spawn_tile_y: w.spawn_y as i16,
            world_surface: w.world_surface as i16,
            rock_layer: w.rock_layer as i16,
            world_id: 1,
            world_name: w.name.clone(),
            // `Main.GameMode` — the port has no mode byte yet; classic is 0.
            game_mode: 0,
            unique_id: [0; 16],
            // `Main.ActiveWorldFileData.WorldGeneratorVersion` — the port's
            // generator version lives in the world seed's place; the save
            // format's version is the closest port-side marker (M6-NET-2
            // pins this to the generator build).
            generator_version: save::VERSION as u64,
            moon_type: w.moon_type,
            backgrounds: [0; 11],
            ice_back_style: 0,
            jungle_back_style: 0,
            hell_back_style: w.hell_back_style,
            wind_speed_target: 0.0,
            num_clouds: 0,
            tree_x: [0; 3],
            tree_style: [0; 4],
            cave_back_x: [0; 3],
            cave_back_style: [0; 4],
            tree_tops: [0; net::TREE_TOPS_BYTES],
            max_raining: 0.0,
            world_flags: [net::BitsByte::new(); 14],
            sundial_cooldown: w.sundial_cooldown,
            moondial_cooldown: w.moondial_cooldown,
            saved_ore_tiers: [0; 7],
            invasion_type: 0,
            lobby_id: 0,
            sandstorm_severity: 0.0,
            extra_spawn_points: Vec::new(),
            dungeon_x: 0,
            dungeon_y: 0,
        }
    }

    /// Drops the client role and reopens the menu with the reason. Only the
    /// client paths call this; the role is `Off` during the dispatch (the
    /// `mem::replace` borrow dance), so this flags the drop and `poll_net`
    /// skips the restore instead of assigning the role back.
    fn net_disconnect(&mut self, reason: &str) {
        log::warn!("disconnected: {reason}");
        self.net_dropped = true;
        self.net_state = net::client_state::NOT_CONNECTED;
        self.net_chunks.clear();
        self.remote_players.clear();
        self.local_id = 0;
        if self.menu.is_none() {
            let mut m = menu::Menu::new(self.settings.clone(), self.player_name.clone());
            m.error = Some(format!("Disconnected: {reason}"));
            self.menu = Some(m);
        }
    }

    /// Sends our player (and, when hosting, the authoritative state) at 10 Hz.
    /// The in-world gate is the accept gate's mirror: the internal messages
    /// are only legal at `IN_WORLD` (`MessageBuffer.cs:161`).
    pub(crate) fn net_tick(&mut self, dt: f32) {
        if matches!(self.net, NetRole::Off) {
            return;
        }
        self.net_timer += dt;
        if self.net_timer < 0.1 {
            return;
        }
        self.net_timer = 0.0;
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let me = net::PlayerNet {
            id: self.local_id,
            name: self.player_name.clone(),
            x: p.x,
            y: p.y,
            vx: p.vx,
            vy: p.vy,
            hp: p.hp,
            max_hp: p.max_hp,
            facing: p.facing,
            selected: self.selected as u16,
            on_ground: p.on_ground,
            leg_frame: p.leg_frame,
            body_frame: p.body_frame,
        };
        match &self.net {
            NetRole::Host(server) => {
                server.broadcast(&S2c::Player(me));
                let npcs: Vec<net::NpcNet> = self
                    .npcs
                    .iter()
                    .enumerate()
                    .map(|(i, n)| net::NpcNet {
                        id: i as u32,
                        kind: n.kind.to_u16(),
                        x: n.x,
                        y: n.y,
                        vx: n.vx,
                        vy: n.vy,
                        hp: n.hp,
                        max_hp: n.max_hp,
                        facing: n.facing,
                        on_ground: n.on_ground,
                        anim: n.anim,
                        hurt_cd: n.hurt_cd,
                    })
                    .collect();
                server.broadcast(&S2c::Npcs(npcs));
                let items: Vec<net::ItemNet> = self
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, it)| net::ItemNet {
                        id: i as u32,
                        item: it.kind,
                        count: it.count,
                        x: it.pos.x,
                        y: it.pos.y,
                    })
                    .collect();
                server.broadcast(&S2c::Items(items));
                server.broadcast(&S2c::Time {
                    time: self.world.time,
                    day: self.world.day_time,
                });
            }
            NetRole::Client(client) => {
                if self.net_state >= net::client_state::IN_WORLD {
                    client.send(&C2s::Player(me));
                }
            }
            NetRole::Off => {}
        }
    }

    /// Client: asks the host to apply an edit (the host echoes it back to everyone).
    pub(crate) fn request_edit(&self, x: i32, y: i32, kind: u8, tile: u16) {
        match &self.net {
            NetRole::Host(server) => server.broadcast(&S2c::Edit { x, y, kind, tile }),
            NetRole::Client(client) => client.send(&C2s::Edit { x, y, kind, tile }),
            NetRole::Off => {}
        }
    }
}

/// `PlayerItemSlotID`'s M6-NET-1 subset (`Terraria/ID/PlayerItemSlotID.cs`) —
/// the slot arithmetic the C# inventory messages use.
mod player_item_slot {
    /// The 1:1 `PlayerItemSlotID` block arithmetic the inventory messages use
    /// (`Terraria/ID/PlayerItemSlotID.cs`); the 59 grid slots are sent as
    /// plain `0..58` indices.
    pub const ARMOR_0: i16 = 59;
    pub const DYE_0: i16 = 59 + 20;
    pub const MISC_0: i16 = DYE_0 + 10;
    pub const MISC_DYE_0: i16 = MISC_0 + 5;
    pub const BANK_1_0: i16 = MISC_DYE_0 + 5;
    pub const BANK_2_0: i16 = BANK_1_0 + 40;
    pub const TRASH: i16 = BANK_2_0 + 40;

    /// `PlayerItemSlotID.Bank1_0 + bank * 40` (the `Bank1_0..Bank4_39` block).
    pub const fn bank_1_0(bank: usize) -> i16 {
        BANK_1_0 + (bank as i16) * 40
    }
}

/// Compresses a world for the chunk stream (zlib, speed over ratio: the tile
/// array is mostly empty cells and compresses well).
pub(crate) fn compressed_world(world: &World) -> Vec<u8> {
    use std::io::Write;
    let bytes = save::to_bytes(world).unwrap_or_default();
    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    let _ = enc.write_all(&bytes);
    enc.finish().unwrap_or_default()
}

fn decompressed_world(data: &[u8]) -> Option<World> {
    use std::io::Read;
    let mut out = Vec::new();
    let mut dec = flate2::read::ZlibDecoder::new(data);
    dec.read_to_end(&mut out).ok()?;
    save::parse(&out).ok()
}

fn npc_from_net(world: &World, n: &net::NpcNet) -> Option<crate::npc::Npc> {
    let kind = crate::npc::NpcKind::from_u16(n.kind)?;
    let tx = (n.x / TILE_PX) as i32;
    let mut npc = match kind {
        crate::npc::NpcKind::Slime => crate::npc::Npc::spawn_slime(world, tx)?,
        crate::npc::NpcKind::DemonEye | crate::npc::NpcKind::WanderingEye => {
            crate::npc::Npc::spawn_eye(world, tx)?
        }
        crate::npc::NpcKind::Zombie => crate::npc::Npc::spawn_zombie(world, tx)?,
        crate::npc::NpcKind::Guide => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::Merchant
        | crate::npc::NpcKind::Dryad
        | crate::npc::NpcKind::Nurse
        | crate::npc::NpcKind::ArmsDealer
        | crate::npc::NpcKind::Demolitionist
        | crate::npc::NpcKind::Painter
        | crate::npc::NpcKind::WitchDoctor
        | crate::npc::NpcKind::Clothier
        | crate::npc::NpcKind::GoblinTinkerer => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::EyeOfCthulhu => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::QueenBee
        | crate::npc::NpcKind::SkeletronHead
        | crate::npc::NpcKind::WallOfFlesh
        | crate::npc::NpcKind::TheDestroyer
        | crate::npc::NpcKind::SkeletronPrime
        | crate::npc::NpcKind::Retinazer
        | crate::npc::NpcKind::Spazmatism
        | crate::npc::NpcKind::Plantera
        | crate::npc::NpcKind::BrainOfCthulhu
        | crate::npc::NpcKind::Golem
        | crate::npc::NpcKind::GolemHead
        | crate::npc::NpcKind::GolemFistLeft
        | crate::npc::NpcKind::GolemFistRight
        | crate::npc::NpcKind::GolemHeadFree
        | crate::npc::NpcKind::Deerclops
        | crate::npc::NpcKind::Creeper => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::Skeleton
        | crate::npc::NpcKind::UndeadMiner
        | crate::npc::NpcKind::GoblinPeon
        | crate::npc::NpcKind::GoblinThief
        | crate::npc::NpcKind::GoblinWarrior
        | crate::npc::NpcKind::GoblinArcher
        | crate::npc::NpcKind::BoundGoblin => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        crate::npc::NpcKind::EaterOfSouls
        | crate::npc::NpcKind::Hornet
        | crate::npc::NpcKind::MeteorHead => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The swimmers: the ctor plants the feet on the column surface —
        // the net state overwrites the position below.
        crate::npc::NpcKind::Goldfish
        | crate::npc::NpcKind::Piranha
        | crate::npc::NpcKind::Shark => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // The worm segments: the net spawns each link standalone (the chain
        // links rebuild through the init — the head; a lone body/tail dies
        // finding no leader — the net-chain sync lands with the netglue
        // worm work).
        crate::npc::NpcKind::DevourerHead
        | crate::npc::NpcKind::DevourerBody
        | crate::npc::NpcKind::DevourerTail
        | crate::npc::NpcKind::GiantWormHead
        | crate::npc::NpcKind::GiantWormBody
        | crate::npc::NpcKind::GiantWormTail
        | crate::npc::NpcKind::EaterOfWorldsHead
        | crate::npc::NpcKind::EaterOfWorldsBody
        | crate::npc::NpcKind::EaterOfWorldsTail => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        // The casters/spheres: the ctor only seeds the stats — the net
        // state overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::GoblinSorcerer
        | crate::npc::NpcKind::GoblinThief
        | crate::npc::NpcKind::GoblinWarrior
        | crate::npc::NpcKind::GoblinArcher
        | crate::npc::NpcKind::GoblinPeon
        | crate::npc::NpcKind::GoblinSummoner
        | crate::npc::NpcKind::SnowmanGangsta
        | crate::npc::NpcKind::SnowBalla
        | crate::npc::NpcKind::MysterySnowman
        | crate::npc::NpcKind::MartianOfficer
        | crate::npc::NpcKind::GrayGrunt
        | crate::npc::NpcKind::MartianEngineer
        | crate::npc::NpcKind::GigaZapper
        | crate::npc::NpcKind::MartianWalker
        | crate::npc::NpcKind::MartianProbe
        | crate::npc::NpcKind::MartianDrone
        | crate::npc::NpcKind::BrainScrambler
        | crate::npc::NpcKind::RayGunner
        | crate::npc::NpcKind::ScutlixRider
        | crate::npc::NpcKind::Scutlix
        | crate::npc::NpcKind::MartianSaucer
        | crate::npc::NpcKind::MartianSaucerTurret
        | crate::npc::NpcKind::MartianSaucerCannon
        | crate::npc::NpcKind::MartianSaucerCore
        | crate::npc::NpcKind::FlyingDutchman
        | crate::npc::NpcKind::FlyingDutchmanCannon
        | crate::npc::NpcKind::PirateDeckhand
        | crate::npc::NpcKind::PirateCaptainsMate
        | crate::npc::NpcKind::PirateArcher
        | crate::npc::NpcKind::PirateCorsair
        | crate::npc::NpcKind::PirateCaptain
        | crate::npc::NpcKind::Tim
        | crate::npc::NpcKind::DarkCaster
        | crate::npc::NpcKind::FireImp
        | crate::npc::NpcKind::ChaosBall
        | crate::npc::NpcKind::BurningSphere
        | crate::npc::NpcKind::WaterSphere => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::CaveBat
        | crate::npc::NpcKind::JungleBat
        | crate::npc::NpcKind::Hellbat
        | crate::npc::NpcKind::VoodooDemon => {
            // The net spawn hangs the bat at the net Y (the cave air).
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_cave_flyer(world, tx, y, kind)?
        }
        crate::npc::NpcKind::KingSlime => crate::npc::Npc::spawn_king_slime(world, tx)?,
        // The hungry/leeches: the ctor seeds the stats — the net state
        // overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::Hungry
        | crate::npc::NpcKind::LeechHead
        | crate::npc::NpcKind::LeechBody => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The eye and the servant: the ctor seeds the stats — the net state
        // overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::EyeOfCthulhu | crate::npc::NpcKind::ServantOfCthulhu => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
    };
    npc.x = n.x;
    npc.y = n.y;
    npc.vx = n.vx;
    npc.vy = n.vy;
    npc.hp = n.hp;
    npc.max_hp = n.max_hp;
    npc.facing = n.facing;
    npc.on_ground = n.on_ground;
    npc.anim = n.anim;
    npc.hurt_cd = n.hurt_cd;
    Some(npc)
}

#[cfg(test)]
mod handshake_tests {
    //! In-process handshake tests: two Apps over a real loopback socket, with
    //! port 0 (`Netplay`'s ephemeral bind) driving `flow_app`s.

    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::ui::save_list::{PlayerEntry, WorldEntry};

    fn apps() -> (App, App) {
        (
            flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new()),
            flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new()),
        )
    }

    /// Pumps both sides until the client reaches `goal`, the connection drops
    /// or the deadline passes. The sleep is the frame time the real loop has
    /// between polls: without a yield the reader/writer threads never get
    /// scheduled on a loaded core, so the deadline — not a round count —
    /// makes the handshake tests load-independent.
    fn drive_to(host: &mut App, client: &mut App, goal: i32) {
        // env-gated exit trace
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while client.net_state != goal
            && !matches!(client.net, NetRole::Off)
            && std::time::Instant::now() < deadline
        {
            client.poll_net();
            host.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // The real loop polls every frame; here the client's goal can arrive
        // in the same instant the host still owes a dispatch (the client's
        // `12 PlayerSpawn` completes the slot), so drain a few extra rounds.
        for _ in 0..8 {
            host.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    /// Pumps until the client role is gone (a kick arrived or the socket died).
    fn drive_until_dropped(host: &mut App, client: &mut App) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !matches!(client.net, NetRole::Off) && std::time::Instant::now() < deadline {
            client.poll_net();
            host.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    fn connect_pair(password: &str, client_pw: &str) -> (App, App) {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, password.to_string()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr, client_pw.to_string()));
        (host_app, client_app)
    }

    #[test]
    fn handshake_drives_both_sides_to_in_world() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(
            client_app.net_state,
            net::client_state::IN_WORLD,
            "client never reached IN_WORLD (state {})",
            client_app.net_state
        );
        // The world installed in place is the host's world.
        assert_eq!(client_app.world.width, host_app.world.width);
        assert_eq!(client_app.world.height, host_app.world.height);
        assert_eq!(client_app.world.name, host_app.world.name);
        // The menu is gone — the game runs.
        assert!(client_app.menu.is_none());
        assert!(client_app.front.is_none());
        // The server slot completed too and remembers the name.
        if let NetRole::Host(server) = &host_app.net {
            assert_eq!(server.state(1), net::slot_state::IN_WORLD);
            assert_eq!(server.slot_name(1), "client");
            assert_eq!(server.client_count(), 1);
        } else {
            panic!("host lost its role");
        }
        // The join bundle: the host lists the client, the client lists nobody
        // else (the host itself only shows through the 10 Hz snapshots).
        assert_eq!(host_app.remote_players.len(), 1);
        assert!(client_app.remote_players.is_empty());
    }

    #[test]
    fn password_gates_the_join() {
        let (mut host_app, mut client_app) = connect_pair("s3cret", "s3cret");
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        if let NetRole::Host(server) = &host_app.net {
            assert_eq!(server.state(1), net::slot_state::IN_WORLD);
        }
    }

    #[test]
    fn wrong_password_is_kicked() {
        let (mut host_app, mut client_app) = connect_pair("s3cret", "nope");
        drive_until_dropped(&mut host_app, &mut client_app);
        assert!(matches!(client_app.net, NetRole::Off));
        let err = client_app.menu.as_ref().and_then(|m| m.error.clone());
        assert_eq!(err.as_deref(), Some("Disconnected: Incorrect password."));
    }

    #[test]
    fn wrong_version_is_kicked() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        // Bypass the FSM's Connect: send a bogus version directly.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Connect("bogus-1.2.3".into()));
        }
        drive_until_dropped(&mut host_app, &mut client_app);
        assert!(matches!(client_app.net, NetRole::Off));
        let err = client_app.menu.as_ref().and_then(|m| m.error.clone());
        assert_eq!(
            err.as_deref(),
            Some("Disconnected: You are not using the same version as this server.")
        );
    }

    #[test]
    fn a_fresh_slot_may_only_send_connect() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        // The accept gate (`MessageBuffer.cs:170`): msg 4 at State 0.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerInfo(net::PlayerInfo::with_name(1, "sneaky")));
        }
        drive_until_dropped(&mut host_app, &mut client_app);
        assert!(matches!(client_app.net, NetRole::Off));
        let err = client_app.menu.as_ref().and_then(|m| m.error.clone());
        assert_eq!(
            err.as_deref(),
            Some("Disconnected: Invalid operation at this state.")
        );
    }

    #[test]
    fn a_duplicate_connect_is_ignored() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Connect(net::CONNECT_VERSION.into()));
        }
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        // The FSM's own Connect plus the manual replay still complete.
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        if let NetRole::Host(server) = &host_app.net {
            assert_eq!(server.state(1), net::slot_state::IN_WORLD);
        }
    }

    #[test]
    fn the_world_blob_round_trips() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        // Put a mark in the host world the install must reproduce.
        host_app.world.tiles[3 * host_app.world.width as usize + 4].kind = 7;
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        let at = |w: &tr_core::World| w.tiles[3 * w.width as usize + 4].kind;
        assert_eq!(at(&client_app.world), at(&host_app.world));
    }

    #[test]
    fn the_load_status_reaches_the_client() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        // Enough time for the Status line to land with the world stream.
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        // Either still streaming (status set) or already complete.
        assert!(
            client_app.net_state == net::client_state::IN_WORLD
                || !client_app.net_status_text.is_empty(),
            "no load status was received (state {})",
            client_app.net_state
        );
    }
}
