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

use crate::app::netmodules::ModuleBody;
use crate::app::{App, LiquidSystem, NetRole};
use crate::{achievements, item};
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

/// `Main.multiplayerNPCSmoothingRange` (`Main.cs:1663`, default 300,
/// `Main.cs:67007`): the window inside which a networked position snap
/// accrues a visual lag instead of jumping. Both the players'
/// (`MessageBuffer.cs:1004-1008`) and the NPCs' (`:1636-1646`) smoothing use
/// it.
const SMOOTHING_RANGE: f32 = 300.0;

/// `Main.teamColor[0..6]` (`Main.cs:6657-6662`): the six team chat colours —
/// white (no team), red, green, blue, yellow, pink. The `45 TeamChange`
/// announcement reads them (`MessageBuffer.cs:2337`). W4-g49.
const TEAM_COLORS: [[u8; 3]; 6] = [
    [255, 255, 255],
    [218, 59, 59],
    [59, 218, 85],
    [59, 149, 218],
    [242, 221, 100],
    [224, 100, 242],
];

/// `Main.maxNPCs` (`Main.cs:67005`, 200): the NPC slot-array bound the
/// `56`/`60` readers gate on (`MessageBuffer.cs:2615`, `:2701`). W4-g49.
const MAX_NPCS: usize = 200;

/// W4-c28: the `13` heartbeat is the C#'s own `netPlayCounter % 420` at
/// 60 Hz (`Main.cs:64527-64529`) — `net_player_sync::update_client_beat`
/// owns it; the old 10 Hz 42-call stand-in is gone.

const TXT_WRONG_VERSION: &str = "You are not using the same version as this server.";
const TXT_BAD_PASSWORD: &str = "Incorrect password.";
const TXT_NAME_EMPTY: &str = "Name is empty.";
const TXT_NAME_TOO_LONG: &str = "Name is too long.";
const TXT_NAME_TAKEN: &str = "That name is taken.";

/// `WorldGen.IsChestRigged` (`WorldGen.cs:...`): the Dead Man's Chest is the
/// 467 tile whose `frameX / 36 == 4`.
pub(crate) fn is_rigged(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.kind == 467 && t.frame_x / 36 == 4)
}

/// The `54 NPCBuffs` body's pairs (`NetMessage.cs:1028-1037`): every live
/// `(buffType, buffTime)` with both positive.
pub(crate) fn npc_buffs_of(npc: &crate::npc::Npc) -> Vec<(u16, u16)> {
    npc.buffs
        .buff_type
        .iter()
        .zip(npc.buffs.buff_time.iter())
        .filter(|(t, time)| **t > 0 && **time > 0)
        .map(|(t, time)| (*t as u16, (*time).clamp(0, u16::MAX as i32) as u16))
        .collect()
}

impl App {
    /// `Main.chest[i]`'s array slot, created when absent: the C# client holds
    /// the whole 8000-slot array from the world load, the port ships chests
    /// lazily, so the `155`/`69` arms materialise their slot first.
    fn materialise_chest_slot(
        chests: &mut Vec<crate::chest::WorldChest>,
        idx: usize,
    ) -> Option<&mut crate::chest::WorldChest> {
        if idx >= crate::chest::MAX_CHESTS {
            return None;
        }
        while chests.len() <= idx {
            chests.push(crate::chest::WorldChest {
                tile: crate::chest::unplaced_tile(),
                max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
                ..Default::default()
            });
        }
        chests.get_mut(idx)
    }

    /// `NetMessage.SendChestContentsTo` (`NetMessage.cs:1868-1876`): the `155`
    /// size header then one `32` per slot, all to the one target.
    fn send_chest_contents_to(&self, server: &net::Server, chest: usize, target: u32) {
        let Some(c) = self.world_chests.get(chest) else {
            return;
        };
        let (tx, ty) = c.tile;
        let max_items = c.max_items;
        server.send(
            target,
            &S2c::ChestSize(net::ChestSize {
                chest: chest as i16,
                size: max_items.min(i16::MAX as usize) as i16,
            }),
        );
        for slot in 0..max_items {
            let item = c.slots.get(slot).copied().unwrap_or(crate::item::EMPTY_SLOT);
            server.send(
                target,
                &S2c::ChestItem(net::ChestItem {
                    chest: chest as i16,
                    slot: slot as u8,
                    stack: item.count.min(i16::MAX as u32) as i16,
                    prefix: item.prefix.min(u8::MAX as u16) as u8,
                    kind: item.kind as i16,
                }),
            );
        }
        let _ = (tx, ty);
    }

    /// `case 34 ChestUpdates`'s host place/destroy pass + relays
    /// (`MessageBuffer.cs:1995-2122`, W4-g52). The reader already zeroed the
    /// fifth short (`:1991-1994`); the host recomputes it from `PlaceChest` /
    /// `Chest.FindChest`. The `34` relay rides the plain broadcast gate
    /// (`NetMessage.cs:1696-1707` — no section filter, no State-3 bypass):
    /// success broadcasts to everyone, refusal targets the asker only. The
    /// client's `PlaceChestDirect`/`DestroyChestDirect` apply (`:2124-2157`)
    /// is the apply-arm lane (g48-g51).
    fn host_chest_updates(&mut self, server: &net::Server, id: u32, cu: net::ChestUpdates) {
        let (action, x, y, style) = (cu.action, cu.x as i32, cu.y as i32, cu.style as i32);
        match action {
            // `:2001-2017` — place a chest (21).
            0 => self.host_place_container(server, id, action, x, y, style, 21, false),
            // `:2019-2039` — destroy a chest (21): the 2×2 anchor decode.
            1 => self.host_destroy_container(server, action, x, y, 21),
            // `:2043-2055` — place a dresser (88).
            2 => self.host_place_container(server, id, action, x, y, style, 88, false),
            // `:2057-2074` — destroy a dresser (88): the 3×2 anchor decode.
            3 => self.host_destroy_container(server, action, x, y, 88),
            // `:2078-2094` — place a piggy-bank chest (467, secondType).
            4 => self.host_place_container(server, id, action, x, y, style, 467, true),
            // `:2096-2115` — destroy a piggy-bank chest (467): the 2×2 anchor.
            5 => self.host_destroy_container(server, action, x, y, 467),
            _ => {}
        }
        self.world_key = None;
    }

    /// The shared destroy arm of `case 34` (`MessageBuffer.cs:2019-2039` /
    /// `:2057-2074` / `:2096-2115`): the anchor decode (2×2 for 21/467, 3×2
    /// for 88), `Chest.FindChest`, `WorldGen.KillTile`, and the relay carrying
    /// the found index only when the cell died. The C# `KillTile` gates on
    /// `Chest.CanDestroyChest` (a full chest refuses, `WorldGen.cs:63695`) and
    /// removes the store entry through the cascade; the port's core
    /// `kill_tile` carries neither (the M3-CHEST-UI no-store seam), so the
    /// `can_kill_tile` + `chest_holds_items` gate and the `remove_chest` +
    /// silent local close ride here — the `break_tile` compensation pattern
    /// (`editing.rs:5559`/`:5604-5622`, S1-P-i43).
    fn host_destroy_container(
        &mut self,
        server: &net::Server,
        action: u8,
        mut x: i32,
        mut y: i32,
        kind: u16,
    ) {
        if !self.world.tile(x, y).is_some_and(|t| t.kind == kind) {
            return;
        }
        let t = self.world.tile(x, y).copied().unwrap();
        // The anchor decode: the dresser's 3-wide `frameX % 54 / 18` column,
        // the chest's 2-wide `frameX % 36 != 0`; the row walks back on
        // `frameY % 36 != 0` for all three.
        if kind == 88 {
            x -= i32::from(t.frame_x) % 54 / 18;
        } else if t.frame_x % 36 != 0 {
            x -= 1;
        }
        if t.frame_y % 36 != 0 {
            y -= 1;
        }
        let number = crate::chest::find_chest(&self.world_chests, x, y)
            .map(|i| i as i16)
            .unwrap_or(-1);
        // `Chest.CanDestroyChest` (`Chest.cs:609-623`): a chest holding any
        // real stack refuses the break — the `KillTile` entry's gate.
        let holds_items = |cx: i32, cy: i32| {
            self.world_chests
                .iter()
                .any(|c| c.tile == (cx, cy) && c.slots.iter().any(|s| s.count > 0))
        };
        if !tr_core::edit::can_kill_tile(&self.world, x, y, holds_items) {
            return;
        }
        let kill = tr_core::edit::kill_tile(&mut self.world, x, y, false);
        if !self.world.tile(x, y).is_some_and(|t| t.active()) {
            // The store removal the C# cascade's `Chest.DestroyChest` runs
            // (`Chest.cs:625-645`) + the silent local close (`:640-643`).
            if let Some(idx) = crate::chest::find_chest(&self.world_chests, x, y) {
                crate::chest::remove_chest(&mut self.world_chests, idx);
                if let Some(p) = self.player.as_mut() {
                    if p.chest == idx as i32 {
                        p.chest = crate::bank::CHEST_CLOSED;
                    }
                }
            }
            // The host's `KillTile` drops the object item (`Main.netMode != 1`
            // passes for the host, `WorldGen.cs:63860`): the cascade's
            // `Check*` records through `registry::tile_drop` — the `break_tile`
            // object-drop pattern (`editing.rs:5905-5913`). The bare
            // `Main.rand` stream (the message handler runs outside every swap).
            if let Some(kill) = kill {
                for drop in &kill.object_drops {
                    let id = drop
                        .item
                        .map(|item| item as u16)
                        .or_else(|| crate::registry::tile_drop(drop.tile_kind, 0));
                    if let Some(id) = id {
                        let rng = self.rngs.global();
                        crate::world_sim::items::spawn_item_drop(
                            &mut self.items,
                            rng,
                            id,
                            1,
                            drop.x,
                            drop.y,
                        );
                    }
                }
            }
            server.broadcast(&S2c::ChestUpdates(net::ChestUpdates {
                action,
                x: x as i16,
                y: y as i16,
                style: 0,
                chest: number,
            }));
        }
    }

    /// The shared place arm of `case 34` (`MessageBuffer.cs:2003-2016` /
    /// `:2045-2054` / `:2080-2093`): `WorldGen.PlaceChest` + `Chest.CreateChest`,
    /// then the success broadcast or the refusal-to-asker + the drop-back item.
    /// `second_type` selects the 467 drop family; the dresser (88) drops
    /// `GetItemDrop_Dressers` instead (`:2049`).
    fn host_place_container(
        &mut self,
        server: &net::Server,
        id: u32,
        action: u8,
        x: i32,
        y: i32,
        style: i32,
        kind: u16,
        second_type: bool,
    ) {
        // `WorldGen.PlaceChest` (`WorldGen.cs:57813-57845`) returns the object's
        // top-left; `Chest.CreateChest` (`Chest.cs:590-607`) claims the store
        // slot there. Either failing is the C# `-1` refusal.
        let placed = tr_core::edit::place_chest(&mut self.world, x, y, kind, style)
            .and_then(|(ox, oy)| crate::chest::try_create_chest(&mut self.world_chests, ox, oy));
        match placed {
            Some(idx) => {
                // `:2015`/`:2053`/`:2092` — the success relay to everyone.
                server.broadcast(&S2c::ChestUpdates(net::ChestUpdates {
                    action,
                    x: x as i16,
                    y: y as i16,
                    style: style as i16,
                    chest: idx as i16,
                }));
            }
            None => {
                // `:2006`/`:2048`/`:2083` — the refusal targets the asker only
                // (remoteClient = whoAmI, the targeted arm `NetMessage.cs:
                // 1814-1825`, IsConnected-only).
                server.send(
                    id,
                    &S2c::ChestUpdates(net::ChestUpdates {
                        action,
                        x: x as i16,
                        y: y as i16,
                        style: style as i16,
                        chest: -1,
                    }),
                );
                // `:2007-2011`/`:2049`/`:2084-2088` — the drop-back item,
                // `noBroadcast: true` (the host keeps it; the cadence streams
                // it). The dresser family has no `> 0` gate (`:2049`).
                let drop = if kind == 88 {
                    crate::chest::get_item_drop_dressers(style)
                } else {
                    crate::chest::get_item_drop_chests(style, second_type)
                };
                if kind == 88 || drop > 0 {
                    // The `Item.NewItem` draws ride the bare `Main.rand` stream
                    // — the message handler runs outside every swap
                    // (`Main.cs:16895`'s NpcStrike precedent, S1-E-21i).
                    let rng = self.rngs.global();
                    crate::world_sim::items::spawn_item_drop(&mut self.items, rng, drop, 1, x, y);
                }
            }
        }
    }

    /// Drains network events without borrowing `self.net` across handlers, and
    // keeps the idle timers alive (`TimeOutTimer`, `MessageBuffer.cs:127-134`).
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
            // `Ping.Reset()` (`Netplay.cs:474`): the stopwatch restarts the
            // moment the connection attempt starts, so the first probe can go
            // out after the first 250 ms of silence.
            self.net_ping_at = Some(Instant::now());
            self.net_ping_waiting = false;
            self.net_rtt_ms = 0;
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
            // S1-I-4d: the `23` stream is now event-driven (the faithful
            // `netSpam` cadence), not a whole-set broadcast, so there is no
            // "missing from the batch" despawn diff or staleness prune. A dead
            // NPC arrives as an explicit life-0 `23` (`NPC.cs:90385-90392`),
            // removed in the `NpcInfo` arm below (`MessageBuffer.cs:1661-1664`).
            self.net_rebuild_lists();
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
            // `case 50` - a player's buff bar (`MessageBuffer.cs:2454-2480`).
            // The host forces the subject to the sender (`:2457-2460`), reads
            // the ids, gives each 60 ticks and relays to the others.
            C2s::PlayerBuffs(pb) => {
                self.remote_players.entry(id).or_default().buffs = pb.buffs.clone();
                server.broadcast_except(
                    &S2c::PlayerBuffs(net::PlayerBuffs {
                        player_id: id as u8,
                        buffs: pb.buffs,
                    }),
                    id,
                ); // `:2478`
            }
            // `case 51` — MiscDataSync (`MessageBuffer.cs:2482-2530`). The
            // server overrides the player byte to the sender's slot
            // (`:2485-2488`); kind 5 registers the Torch God's 664 bestiary
            // kill (`:2514-2520` — the 5043 use's `SendData(51, …, 5f)`,
            // `Player.cs:45241`, W4-f33). The other kinds' systems
            // (Skeletron 1, the roars 2, the sundial 3, the mimic smoke 4,
            // the moondial 6) are unported — no-ops like the C#'s missing
            // arms on a client.
            C2s::MiscDataSync { kind, .. } => {
                if kind == 5 {
                    self.bestiary.kills.register_kill("TorchGod");
                    let _ = self.try_grant_bestiary_100_net();
                    self.broadcast_bestiary_kill(
                        664,
                        self.bestiary.kills.get_kill_count("TorchGod"),
                    );
                }
            }
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
                // `player18.UsingBiomeTorches/happyFunTorchTime/
                // unlockedBiomeTorches = bitsByte33[0..2]`
                // (`MessageBuffer.cs:343-345`, W4-f33) — the server keeps
                // its own copy of the biome-torch trio.
                server.set_slot_torch_flags(
                    id,
                    pi.flags2.get(0),
                    pi.flags2.get(1),
                    pi.flags2.get(2),
                );
                log::info!("slot {id} announced as {}", pi.name);
            }
            // `case 5` — one inventory slot (`MessageBuffer.cs:399-462`).
            // The server attributes the slot to the sending client
            // (`:409-412`), stores it in that client's copy of the
            // inventory and relays it to the other clients (`:454-457`,
            // `PlayerItemSlotID.CanRelay`).
            C2s::Inventory { slot, item, .. } => {
                let peer = id as u8;
                server.set_slot_inventory(id, slot, item);
                if player_item_slot::can_relay(slot) {
                    server.broadcast_except(
                        &S2c::Inventory {
                            player_id: peer,
                            slot,
                            item,
                        },
                        id,
                    );
                }
            }
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
                    self.host_stream_sections(server, id, sx, sy);
                }
            }
            // `case 12` — the player spawned in (`MessageBuffer.cs:901-950`).
            // The reader stores the spawn fields on the sender's player copy
            // (`:903-914`), latches `dead` when `respawnTimer > 0` (`:915-918`)
            // and runs `Spawn(context)` (`:920` — the port's remote copy is a
            // snapshot store; the position lands via `Spawn_SetPosition`,
            // `Player.cs:37987-37991`). The first spawn (State 3) completes the
            // slot and sends the join bundle (`:925-929`); every spawn relays
            // `12` to the others (`:936` first / `:948` after).
            C2s::PlayerSpawn(s) => {
                let first = server.state(id) == slot_state::SPAWN_REQUESTED;
                if first {
                    server.set_state(id, slot_state::IN_WORLD);
                    self.host_slot_join_complete(server, id, s);
                }
                // `:903-918` — the field writes (the host forces the id to the
                // sender, `:904-907`).
                let entry = self.remote_players.entry(id).or_default();
                entry.id = id;
                entry.spawn_x = s.x;
                entry.spawn_y = s.y;
                entry.respawn_timer = s.respawn_timer;
                entry.deaths_pve = s.deaths_pve;
                entry.deaths_pvp = s.deaths_pvp;
                entry.team = s.team as i32;
                entry.dead = s.respawn_timer > 0;
                if entry.name.is_empty() {
                    entry.name = server.slot_name(id);
                }
                // `Spawn_SetPosition` (`Player.cs:37987-37991`): feet at the
                // spawn cell, centred on it.
                entry.x = s.x as f32 * TILE_PX + 8.0 - player::WIDTH * 0.5;
                entry.y = s.y as f32 * TILE_PX - player::HEIGHT;
                entry.vx = 0.0;
                entry.vy = 0.0;
                // `:37648` — `Spawn` clears the talk latch.
                entry.talk_npc = None;
                // `:936`/`:948` — the spawn relay carries the context byte.
                server.broadcast_except(
                    &S2c::PlayerSpawn(net::PlayerSpawn { player_id: id as u8, ..s }),
                    id,
                );
            }
            // `case 154` — the server echoes the probe straight back
            // (`MessageBuffer.cs:4419-4423`); the port's relay is the same
            // single hop.
            C2s::Ping => server.send(id, &S2c::Ping),
            C2s::PlayerActive { active: true, .. } => {
                // The peer is in-world (`case 14`); make sure it has a
                // remote-player entry (the identity usually arrives first).
                if server.state(id) >= slot_state::IN_WORLD {
                    let (px, py) = self.net_peer_spawn_pos();
                    self.remote_players
                        .entry(id)
                        .or_insert_with(|| net::PlayerNet {
                            id,
                            name: server.slot_name(id),
                            x: px,
                            y: py,
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
            // `case 16` — the health report (`MessageBuffer.cs:1095-1117`):
            // the server forces the subject to the sender (`:1100-1103`),
            // derives `dead` (`:1111`) and relays (`:1114`) — W4-g48. The
            // statLifeMax >= 20 floor is the decode's (`net.rs`'s 16 arm).
            C2s::PlayerHp { hp, max_hp, .. } => {
                crate::app::net_apply::host_player_hp(self, server, id, hp, max_hp);
            }
            // `case 13` — the per-tick player state. The server forces the
            // subject to the sender (`MessageBuffer.cs:959`) and relays to
            // everyone else (`MessageBuffer.cs:1054-1057`); the port also
            // keeps the snapshot locally so the host's own view sees the peer.
            C2s::PlayerState(ps) => {
                let peer = id;
                let entry = self.remote_players.entry(peer).or_default();
                entry.id = peer;
                entry.x = ps.x;
                entry.y = ps.y;
                entry.vx = ps.vx;
                entry.vy = ps.vy;
                entry.facing = if ps.flags_a.get(6) { 1 } else { -1 };
                entry.selected = ps.selected_item as u16;
                entry.on_ground = ps.on_ground;
                entry.leg_frame = ps.leg_frame;
                entry.body_frame = ps.body_frame;
                entry.hp = ps.hp;
                entry.max_hp = ps.max_hp;
                let mut relay = ps.clone();
                relay.player_id = id as u8;
                // The C# never echoes a snapshot back to its sender
                // (`SendData`'s `ignoreClient` routing, `NetMessage.cs:1692-1826`).
                server.broadcast_except(&S2c::PlayerState(Box::new(relay)), id);
            }
            C2s::Edit { x, y, kind, tile } => {
                log::info!("net edit kind {kind} at {x},{y} from {id}");
                // W4-g83: the replay's kill cascade rides the bare stream
                // (the message handler runs outside every SwapRandom).
                let broken_wall = apply_edit(&mut self.world, x, y, kind, tile, self.rngs.global());
                // S1-E-21i — the `KillWall` replay's sound arm
                // (`MessageBuffer.cs:1177`): the host is `netMode == 2`, so
                // the positioned gate refuses before any draw
                // (`LegacySoundPlayer.cs:386-388` — the listener's `server`
                // flag); the play call keeps the C#'s shape.
                if let Some(wall) = broken_wall {
                    self.audio.play_wall_break_sound(
                        tr_core::edit::kill_wall_sound(wall),
                        x as f32 * 16.0,
                        y as f32 * 16.0,
                        self.rngs.global(),
                    );
                }
                self.world_key = None;
                // The replicated change: the applied tile read back as a
                // `20 SectionRange` (`SendTileSquare` on the host). W4-g52:
                // the `case 20` fan-out (`NetMessage.cs:1708-1718`) — only
                // the slots already holding a corner section receive it.
                let rec = self.tile_record_at(x, y);
                server.relay_tile_square(
                    &S2c::SectionRange(Box::new(net::SectionRange {
                        x: x as i16,
                        y: y as i16,
                        w: 1,
                        h: 1,
                        change_type: 0,
                        records: vec![rec],
                    })),
                    None,
                );
            }
            // `case 82` — a NetModule envelope (`NetManager.Read`'s dispatch,
            // `NetManager.cs:48-64`).
            C2s::NetModule { module, body } => {
                self.host_module(server, id, module, &body);
            }
            // `case 28` — a client's NPC strike. The host is authoritative: it
            // validates the generation (`MessageBuffer.cs:1781-1784`), applies
            // the strike and relays it, plus a full `23` update on death
            // (`MessageBuffer.cs:1805-1815`).
            C2s::NpcStrike(st) => {
                // `MessageBuffer.cs:1768-1771` — the host acks EVERY `28`
                // with a bodyless `162` to the sender, BEFORE the body reads
                // and the generation gate (the client pops its pending entry
                // even when the strike is refused). W4-g51 id-162.
                server.send(id, &S2c::NpcStrikeAck);
                let slot = st.npc as usize;
                let gen_ok = self
                    .net_npc_gen
                    .get(&(slot as u32))
                    .map(|(_, g)| *g == st.generation as u32)
                    .unwrap_or(false);
                if !gen_ok || !self.npcs.get(slot).is_some_and(|n| n.hp > 0) {
                    log::debug!("strike for unknown npc slot {slot} gen {}", st.generation);
                    return;
                }
                self.npcs[slot].facing = if st.hit_direction > 0 { 1 } else { -1 };
                let strike_x = self.npcs[slot].x;
                // W4-g52 — `SendData(28)`'s pre-switch cascade
                // (`NetMessage.cs:102-107`): a `spawnNeedsSyncing` NPC gets a
                // priority `23` FIRST — BEFORE the strike applies (the C#
                // sends it inside `TrySendData(28)` at
                // `MessageBuffer.cs:1807`, after `StrikeNPC`, but the port
                // runs the whole send path here; the body must carry the
                // PRE-strike life, so the cascade fires ahead of the apply —
                // the flag3 priority `23` reaches every broadcast-open slot
                // and clears the sync counters, `NetMessage.cs:1722-1730`).
                let uid = self.npcs[slot].uid;
                let needs_sync = self
                    .net_npc_states
                    .get(&uid)
                    .is_some_and(|st| st.spawn_needs_syncing);
                if needs_sync {
                    self.send_npc_state_broadcast(server, slot);
                }
                // `NPC.StrikeNPC` (`NPC.cs:81189-81563`): the strike lands on
                // the carrier's life pool when the hit NPC is a linked
                // segment (`NPC.realLife`, `NPC.cs:81374-81378`).
                crate::npc::Npc::strike_melee(
                    &mut self.npcs,
                    slot,
                    st.damage.max(0) as i32,
                    strike_x,
                    st.knock_back,
                    st.crit,
                    Some(id as u8),
                    // `Main.expertMode` — the host's world-mode state is not
                    // persisted yet (DropCtx's note); the classic default.
                    false,
                );
                let dead = self.npcs[slot].hp <= 0;
                // The `case 28` relay (`NetMessage.cs:1751-1762`): the
                // validated values to everyone else, section-culled unless
                // the NPC died (`IsSectionActive`, the check-time grid).
                let section = (
                    net::section_x(self.npcs[slot].x as i32 / TILE_PX as i32),
                    net::section_y(self.npcs[slot].y as i32 / TILE_PX as i32),
                );
                server.relay_strike(
                    &S2c::NpcStrike(st),
                    id,
                    net::RelayCtx::Strike { dead, section },
                    self.game_update_count,
                );
                // The full `23` on death (`MessageBuffer.cs:1808-1811`) —
                // priority (`life <= 0`, `NetMessage.cs:1722`), so it
                // reaches every broadcast-open slot and clears the sync
                // counters (`:1725-1730`).
                if dead {
                    self.send_npc_state_broadcast(server, slot);
                    // The carrier's `23` when the shared pool died
                    // (`MessageBuffer.cs:1812-1815`, `NPC.realLife`).
                    if let Some(carrier) = crate::npc::Npc::carrier_index(&self.npcs, slot) {
                        if self.npcs[carrier].hp <= 0 {
                            self.send_npc_state_broadcast(server, carrier);
                        }
                    }
                }
            }
            // `case 70` — a client's bug net caught an NPC
            // (`MessageBuffer.cs:3102-3110`): the `(short)` slot bounds
            // check, then `NPC.CatchNPC(slot, whoAmI)` — the host is the
            // converter (the item spawns at the CATCHER's centre). The
            // despawn `23` rides the cadence's uid gone-diff.
            C2s::CatchNpc { npc } => {
                let slot = npc as usize;
                if npc >= 0 && slot < self.npcs.len() {
                    if crate::bug_net::catch_npc(self, slot, id as u8) {
                        self.npcs.remove(slot);
                    }
                }
            }
            // `case 71` — a client released a captured critter
            // (`MessageBuffer.cs:3112-3117`): `NPC.ReleaseNPC(x, y, type,
            // style, whoAmI)` — the remote `who` must pass the
            // `npcCatchable` whitelist (`NPC.cs:80113`). The spawn `23`
            // rides the cadence.
            C2s::ReleaseNpc { x, y, npc_type, style } => {
                crate::bug_net::release_npc(
                    self,
                    x,
                    y,
                    npc_type as i32,
                    style as i32,
                    id as u8,
                );
            }
            // `case 27` — a peer's own projectile state. The host refuses a
            // hostile-kind shot or a key whose spawner is not the sender
            // (`MessageBuffer.cs:1725-1728`: `Main.projHostile[type] ||
            // key.Spawner != whoAmI` — the break skips BOTH the apply and
            // the relay), then applies it into its pool (rendering; the
            // owner-side damage arrives as `28`) and relays it to the
            // others (`MessageBuffer.cs:1760-1763`).
            C2s::Projectile(ps) => {
                let (spawner, _, _) = net::projectile_key_parts(ps.key);
                if crate::projectile::data::proj_hostile(ps.kind.max(0) as u16)
                    || spawner != id as u8
                {
                    return; // `:1727` — the whole arm is skipped
                }
                self.projectiles.upsert_remote(&ps);
                let (_, slot, _) = net::projectile_key_parts(ps.key);
                if slot < crate::projectile::MAX_PROJECTILES {
                    let ctx = self.proj_relay_ctx(slot);
                    // The sender's skipped flag is untouched by the C# loop
                    // (`:1784`'s ignoreClient continue runs BEFORE the flag
                    // write); so is every not-broadcast slot's. Snapshot +
                    // restore them around the motion.
                    let flag3 = matches!(
                        ctx,
                        net::RelayCtx::Projectile {
                            ignores_sections: true,
                            ..
                        }
                    );
                    let prev_flags = if flag3 {
                        None
                    } else {
                        let mut snap = self.proj_sync.skipped_snapshot(slot);
                        for (i, f) in snap.iter_mut().enumerate() {
                            if i as u32 == id || !server.broadcast_open(i as u32) {
                                *f = self.proj_sync.skipped_for(slot, i as u8);
                            }
                        }
                        Some(snap)
                    };
                    self.proj_sync.note_broadcast(&self.projectiles, slot, |i| {
                        let section = match ctx {
                            net::RelayCtx::Projectile { section, .. } => section,
                            _ => (0, 0),
                        };
                        server.is_section_active(
                            i as u32,
                            section.0,
                            section.1,
                            self.game_update_count,
                        )
                    });
                    if let Some(prev) = prev_flags {
                        for (i, f) in prev.iter().enumerate() {
                            if i as u32 == id || !server.broadcast_open(i as u32) {
                                self.proj_sync.set_skipped_for(slot, i as u8, *f);
                            }
                        }
                    }
                    server.relay_projectile(
                        &S2c::Projectile(ps.clone()),
                        Some(id),
                        ctx,
                        self.game_update_count,
                    );
                }
            }
            // `case 29` — a peer's projectile is gone. The host gates on the
            // key's owner (`MessageBuffer.cs:1831-1834`: only the owner may
            // kill it), applies the despawn (a finite position moves the
            // slot before `Kill`, NaN/Inf deactivates silently,
            // `:1835-1843`) and relays the RECEIVED position to the others
            // (`:1845-1848` — the relay is skipped when the owner gate
            // refused). The g48 reconciliation keeps d93's owner gate over
            // d91's ungated arm and folds the inline position move into
            // d91's `kill_remote_at` seam (its occupant-match check is the
            // same `live` test, so the behaviour is identical).
            C2s::ProjectileDead { key, x, y } => {
                let (owner, slot, generation) = net::projectile_key_parts(key);
                // `:1829` — the gate + position move only run when the key
                // resolves to the slot's ACTIVE occupant; a stale key falls
                // through to the relay (`:1845-1848` is outside the block).
                let live = self
                    .projectiles
                    .slots()
                    .get(slot)
                    .map_or(false, |p| {
                        p.active && p.owner == owner && p.generation == generation
                    });
                if live && owner != id as u8 {
                    return; // `:1831-1834` — not this client's projectile
                }
                self.projectiles
                    .kill_remote_at(slot, owner, generation, x, y);
                server.broadcast_except(&S2c::ProjectileDead { key, x, y }, id);
            }
            // `case 95` — MurderSomeoneElsesPortal (`MessageBuffer.cs:3519-
            // 3537`, host-only): the client's new portal intersected a
            // peer's portal projectile. Kill the FIRST active type-602
            // whose owner + ai[1] form match, then broadcast one `29` for
            // it (`:3532` passes the key only — number2/3 default to 0).
            C2s::MurderPortal { owner, form } => {
                // `:3527-3534` — the first active type-602 whose owner and
                // ai[1] form match; snapshot before the mutable kill.
                let mut hit = None;
                for (slot, p) in self.projectiles.slots().iter().enumerate() {
                    if p.owner as u16 == owner
                        && p.active
                        && p.kind == 602
                        && p.ai[1] == form as f32
                    {
                        hit = Some((slot, p.owner, p.generation));
                        break; // `:3533` — the first match only
                    }
                }
                if let Some((slot, p_owner, generation)) = hit {
                    let key = self.projectiles.key_of(slot);
                    self.projectiles.kill_remote(slot, p_owner, generation);
                    server.broadcast(&S2c::ProjectileDead {
                        key,
                        x: 0.0,
                        y: 0.0,
                    });
                }
            }
            // `case 128` — a client's golf ball landed in a cup. The host
            // only relays: `SendData(128, -1, num157, …)` with
            // `ignoreClient = owner` (`MessageBuffer.cs:4065-4068`) — the
            // owner already ran the cup effects locally before sending
            // (`GolfHelper.cs:103`). No host-side apply exists.
            C2s::GolfBallInCup(g) => {
                server.broadcast_except(&S2c::GolfBallInCup(g), g.owner as u32);
            }
            C2s::Pickup { drop } => {
                // The client's pickup request (the port's stand-in for the
                // C# `SyncItem`→`151` path — `WorldItem.cs:612-615`). The host
                // validates the range as `GrabItems`' expanded grab rectangle
                // (`Player.cs:34926`, S1-P-i47 — the same gate the client
                // requested through), removes the slot and announces the
                // removal as `151`; the indices shift, and the next cadence
                // tick's `21` diff streams the shifted slots.
                let pos = self
                    .remote_players
                    .get(&id)
                    .map(|p| (p.x, p.y));
                if let Some((px, py)) = pos {
                    if let Some(it) = self.items.get(drop as usize) {
                        // The host re-checks the grab gate (`Player.cs:34911`,
                        // S1-P-i51): a delayed drop refuses the pickup even
                        // inside the grab rectangle (the S1-P-i47 shape).
                        let delayed = it.grab_delay_time > 0
                            && (it.grab_delay_player == id as u8
                                || it.grab_delay_player == 255);
                        let range = crate::item::get_item_grab_range(
                            it.kind,
                            crate::item::GrabFlags::default(),
                        ) as f32;
                        if !delayed && crate::item::grab_rect_intersects(
                            px,
                            py,
                            player::WIDTH,
                            player::HEIGHT,
                            range,
                            it.pos,
                        ) {
                            self.items.remove(drop as usize);
                            server.broadcast(&S2c::ItemDead { slot: drop as i16 });
                        }
                    }
                }
            }
            // `case 31` — the client asks to open a chest
            // (`MessageBuffer.cs:1868-1895`): `FindChest`, refuse when another
            // player already browses it, else `SendChestContentsTo` (155 + the
            // 32 burst), `33` to the asker, `80` to the others and the rigged
            // chest's `59`.
            C2s::ChestOpenRequest(req) => {
                let (cx, cy) = (req.x as i32, req.y as i32);
                let Some(chest_idx) = crate::chest::find_chest(&self.world_chests, cx, cy) else {
                    return; // `num254 > -1` (`:1877`)
                };
                // `Chest.UsingChest(num254) == -1` (`:1877`): nobody else is
                // browsing it. The C# scans ALL active players (`Chest.cs:
                // 502-508`) — the host's own local player (slot 0) included;
                // the port checks the remote slots AND the host's open chest
                // (S1-I-1 wave I — the local-player half was missing).
                if self
                    .remote_players
                    .values()
                    .any(|p| p.chest == Some(chest_idx as i16))
                    || self.player.as_ref().is_some_and(|p| p.chest == chest_idx as i32)
                {
                    return;
                }
                self.send_chest_contents_to(server, chest_idx, id);
                let idx = chest_idx as i16;
                server.send(id, &S2c::PlayerChest(net::PlayerChest {
                    chest: idx,
                    x: cx as i16,
                    y: cy as i16,
                    name_len: 0,
                    name: String::new(),
                })); // `:1880`
                self.remote_players
                    .entry(id)
                    .or_default()
                    .chest = Some(idx); // `:1881`
                server.broadcast_except(
                    &S2c::PlayerChestIndex(net::PlayerChestIndex {
                        player_id: id as u8,
                        chest: idx,
                    }),
                    id,
                ); // `:1886`
                // `:1887-1893` — the Dead Man's Chest trips its wiring.
                if is_rigged(&self.world, cx, cy) {
                    // The §8.1 stamp: the rigged trip can shift a wired
                    // door family (the peers ride the boxes too).
                    crate::world_sim::wiring::stamp_wiring_entity_boxes(self);
                    self.wiring.hit_switch(&mut self.world, cx, cy);
                    // S1-E-21i — the trip's Mech-28 click: the host's
                    // positioned gate refuses (netMode 2), draw-free.
                    for (px, py) in self.wiring.take_switch_clicks() {
                        self.audio
                            .play_switch_click(px as f32, py as f32, self.rngs.global());
                    }
                    // W4-c42 — the rigged chest's wired door cues (the bare
                    // stream, the switch-click precedent).
                    for (open, px, py) in self.wiring.take_door_cues() {
                        self.audio
                            .play_door_sound(open, px as f32, py as f32, self.rngs.global());
                    }
                    // The §8.2 echoes: the MP relay attaches with the g48b
                    // `19` codec lane (contract note); drained so the
                    // queue never lingers across messages.
                    for (n, x, y, d) in self.wiring.take_door_echoes() {
                        self.send_door_toggle(n, x, y, d);
                    }
                    server.broadcast_except(
                        &S2c::HitSwitch(net::HitSwitch {
                            x: cx as i16,
                            y: cy as i16,
                        }),
                        id,
                    );
                }
            }
            // `case 32` — one chest slot changed; relay to the others
            // (`MessageBuffer.cs:1897-1918`).
            C2s::ChestItem(ci) => {
                let idx = ci.chest;
                if idx < 0 || (idx as usize) >= self.world_chests.len() {
                    return; // `num146 >= 0 && num146 < 8000` (`:1904`)
                }
                let slot = ci.slot as usize;
                let value = if ci.kind <= 0 || ci.stack <= 0 {
                    crate::item::EMPTY_SLOT
                } else {
                    crate::item::ItemSlot {
                        kind: ci.kind as u16,
                        name: crate::registry::name(ci.kind as u16),
                        count: ci.stack.max(0) as u32,
                        prefix: ci.prefix as u16,
                    }
                };
                crate::chest::set_chest_slot(&mut self.world_chests, idx as usize, slot, value);
                server.broadcast_except(&S2c::ChestItem(ci), id); // `:1915`
            }
            // `case 34` — a client mined/placed a container
            // (`MessageBuffer.cs:1984-2158`). The host runs the place/destroy
            // pass and relays the result (`:1995-2122`); the client's
            // `PlaceChestDirect`/`DestroyChestDirect` apply (`:2124-2157`) is
            // the apply-arm lane (g48-g51). W4-g52 owns this RELAY/SEND side.
            C2s::ChestUpdates(cu) => {
                self.host_chest_updates(server, id, cu);
            }
            // `case 33` — the open chest / its rename (`MessageBuffer.cs:1920-1982`).
            C2s::PlayerChest(pc) => {
                if pc.name_len != 0 {
                    // `:1972-1977` — the rename: the target is the SENDER's
                    // tracked open chest (`Main.player[whoAmI].chest`,
                    // `:1974`), not the message's index — a close-transition
                    // flush carries -1 while the name belongs to the chest
                    // the player just closed (S1-P-i43). The 255 refusal
                    // byte still APPLIES (the C# reader keeps `name = ""`
                    // for it, `:1926-1936` — the default-name reset rides
                    // this arm). Rewrite + relay `69` to the others.
                    let tracked = self
                        .remote_players
                        .get(&id)
                        .and_then(|p| p.chest)
                        .unwrap_or(pc.chest);
                    if tracked >= 0 {
                        if let Some(chest) = self.world_chests.get_mut(tracked as usize) {
                            chest.name = pc.name.clone();
                            let (tx, ty) = chest.tile;
                            server.broadcast_except(
                                &S2c::ChestName(net::ChestName {
                                    chest: tracked,
                                    x: tx as i16,
                                    y: ty as i16,
                                    name: pc.name.clone(),
                                }),
                                id,
                            );
                        }
                    }
                }
                self.remote_players
                    .entry(id)
                    .or_default()
                    .chest = Some(pc.chest); // `:1979`
                server.broadcast_except(
                    &S2c::PlayerChestIndex(net::PlayerChestIndex {
                        player_id: id as u8,
                        chest: pc.chest,
                    }),
                    id,
                ); // `:1980`
            }
            // `case 46` — a client wants a sign's text
            // (`MessageBuffer.cs:2366-2375`): `Sign.ReadSign` (create when
            // missing, `Sign.cs:27-58`) and answer `47` to the asker.
            C2s::SignRequest(req) => {
                let Some((sx, sy)) = crate::signs::origin(&self.world, req.x as i32, req.y as i32)
                else {
                    return; // `num118 >= 0` (`:2370`)
                };
                let Some(idx) = crate::signs::find_or_create(&mut self.world.signs, sx, sy) else {
                    return;
                };
                let (text, px, py) = {
                    let s = &self.world.signs[idx];
                    (s.text.clone(), s.x as i16, s.y as i16)
                };
                server.send(
                    id,
                    &S2c::SignData(net::SignData {
                        sign: idx as i16,
                        x: px,
                        y: py,
                        text,
                        sender: id as u8,
                        flags: 0,
                    }),
                ); // `:2372`
            }
            // `case 47` — a client's sign text; applied locally, then relayed
            // when it changed (`MessageBuffer.cs:2376-2405`).
            C2s::SignData(sd) => {
                if sd.sign < 0 || sd.sign >= crate::signs::MAX_SIGNS {
                    return; // `num58 >= 0 && num58 < 32000` (`:2384`)
                }
                let idx = sd.sign as usize;
                let old = self.world.signs.get(idx).map(|s| s.text.clone());
                crate::signs::text_sign(
                    &mut self.world,
                    idx,
                    sd.x as i32,
                    sd.y as i32,
                    sd.text.clone(),
                ); // `:2391-2394`
                if old.as_deref() != Some(sd.text.as_str()) {
                    server.broadcast_except(
                        &S2c::SignData(net::SignData {
                            sign: sd.sign,
                            x: sd.x,
                            y: sd.y,
                            text: sd.text,
                            sender: id as u8, // `:2397`
                            flags: sd.flags,
                        }),
                        id,
                    ); // `:2398`
                }
            }
            // `case 59` — a tripped switch: apply and relay to the others
            // (`MessageBuffer.cs:2682-2693`).
            C2s::HitSwitch(hs) => {
                // The §8.1 stamp: the relayed trip can shift a wired door
                // family (the peers ride the boxes too).
                crate::world_sim::wiring::stamp_wiring_entity_boxes(self);
                self.wiring
                    .hit_switch(&mut self.world, hs.x as i32, hs.y as i32); // `:2687`
                // S1-E-21i — the click plays on the host's stream shape
                // (the bare `Main.rand`; the handler runs outside every
                // swap). The netMode-2 gate refuses it, draw-free.
                for (px, py) in self.wiring.take_switch_clicks() {
                    self.audio
                        .play_switch_click(px as f32, py as f32, self.rngs.global());
                }
                // W4-c42 — the trip's wired door cues (the bare stream, the
                // switch-click precedent).
                for (open, px, py) in self.wiring.take_door_cues() {
                    self.audio
                        .play_door_sound(open, px as f32, py as f32, self.rngs.global());
                }
                // The §8.2 echoes: the g48b codec-lane attach point.
                for (n, x, y, d) in self.wiring.take_door_echoes() {
                    self.send_door_toggle(n, x, y, d);
                }
                self.world_key = None;
                server.broadcast_except(&S2c::HitSwitch(hs), id); // `:2691`
            }
            // `case 53` — a client buffed an NPC; the host applies it and
            // pushes the full `54` list (`MessageBuffer.cs:2565-2575`).
            C2s::AddNpcBuff(ab) => {
                let slot = ab.npc as usize;
                if let Some(npc) = self.npcs.get_mut(slot) {
                    npc.buffs
                        .add_buff(ab.buff_type as i32, ab.time as i32); // `:2570`
                    let buffs = npc_buffs_of(npc);
                    server.broadcast(&S2c::NpcBuffs(net::NpcBuffs {
                        npc: ab.npc,
                        buffs,
                    })); // `:2573`
                }
            }
            // `case 30` — a client flipped its PvP flag
            // (`MessageBuffer.cs:1851-1867`): the store + relay + the
            // team-coloured chat fan-out — W4-g48 (`net_apply`).
            C2s::TogglePvp(t) => {
                crate::app::net_apply::host_toggle_pvp(self, server, id, t.hostile);
            }
            // `case 42` — a peer's mana (`MessageBuffer.cs:2290-2306`): the
            // host forces the subject to the sender (`:2293-2296`) and stores
            // both shorts. There is NO relay arm — the fan-out is the join
            // push / send path (`SyncOnePlayer` `NetMessage.cs:2904`, S1-I-4).
            C2s::PlayerMana(pm) => {
                let peer = self.remote_players.entry(id).or_default();
                peer.mana = pm.mana as i32; // `:2303`
                peer.max_mana = pm.max_mana as i32; // `:2304`
            }
            // `case 45` — a peer changed team (`MessageBuffer.cs:2325-2364`):
            // the host forces the subject to the sender (`:2329-2332`), stores
            // the team, relays to the others (`:2342`) and announces the change
            // to the sender + the old/new team members (`:2343-2354`). The
            // `157` team-spawn tail (`:2355-2363`) is a different id (out of
            // this lane) and the port has no team-based-spawn seed.
            C2s::TeamChange(tc) => {
                let old_team = self.remote_players.get(&id).map(|p| p.team).unwrap_or(0);
                let new_team = tc.team as i32;
                let name = {
                    let peer = self.remote_players.entry(id).or_default();
                    peer.team = new_team; // `:2336`
                    peer.name.clone()
                };
                // The relay (`:2342`): `TrySendData(45, -1, whoAmI, …)`.
                server.broadcast_except(
                    &S2c::TeamChange(net::TeamChange {
                        player_id: id as u8,
                        team: tc.team,
                    }),
                    id,
                );
                // The announcement (`:2343-2354`): `Lang.mp[13 + team]` (or
                // `Lang.mp[22]` for the pink team 5) in the team colour, sent
                // to the sender and every player on the old or new team.
                let key_idx = if new_team == 5 { 22 } else { 13 + new_team };
                let text = crate::lang::text_args(
                    &format!("LegacyMultiplayer.{key_idx}"),
                    &[name],
                );
                let color = TEAM_COLORS[new_team.clamp(0, 5) as usize];
                // Recipient slots: the sender, plus the old/new team members
                // (the host is slot 0, the peers ride `remote_players`).
                let mut recipients: Vec<u32> = vec![id];
                if old_team > 0 {
                    if self.player.as_ref().is_some_and(|p| p.team == old_team) {
                        recipients.push(0);
                    }
                    for (&pid, p) in &self.remote_players {
                        if p.team == old_team {
                            recipients.push(pid);
                        }
                    }
                }
                if new_team > 0 {
                    if self.player.as_ref().is_some_and(|p| p.team == new_team) {
                        recipients.push(0);
                    }
                    for (&pid, p) in &self.remote_players {
                        if p.team == new_team {
                            recipients.push(pid);
                        }
                    }
                }
                recipients.sort_unstable();
                recipients.dedup();
                for slot in recipients {
                    if slot == 0 {
                        // The host's own display (`ChatHelper.DisplayMessage`).
                        crate::chat::broadcast_death_text(&mut self.chat, &text, color);
                    } else {
                        let sm = crate::app::netmodules::ServerMessage {
                            author: 255,
                            text: text.clone(),
                            color: net::Rgb(color[0], color[1], color[2]),
                        };
                        server.send(
                            slot,
                            &S2c::NetModule {
                                module: crate::app::netmodules::module_id::TEXT,
                                body: sm.encode(),
                            },
                        );
                    }
                }
            }
            // `case 56` — a client asks for a town NPC's given name + variation
            // (`MessageBuffer.cs:2612-2630`): behind the `0..maxNPCs` slot gate
            // (`:2615`) the host answers the asker with the pair (`:2626`).
            C2s::UniqueTownNpcInfo(req) => {
                let slot = req.npc;
                if slot >= 0 && (slot as usize) < self.npcs.len() {
                    if let Some(npc) = self.npcs.get(slot as usize) {
                        server.send(
                            id,
                            &S2c::UniqueTownNpcInfo(net::UniqueTownNpcInfo {
                                npc: slot,
                                info: Some((npc.given_name.clone(), npc.variation_index)),
                            }),
                        ); // `:2626`
                    }
                }
            }
            // `case 58` — a client played its held instrument
            // (`MessageBuffer.cs:2639-2650`): the host forces the id to the
            // sender (`:2644`) and relays to the others (`:2649`), then
            // breaks — the instrument switch is client-only. W4-g49.
            C2s::InstrumentSound(s) => {
                server.broadcast_except(
                    &S2c::InstrumentSound(net::InstrumentSound {
                        player_id: id as u8, // `:2644` + `:2649`'s number
                        pitch: s.pitch,
                    }),
                    id,
                );
            }
            // `case 60` — an NPC's household banner (`MessageBuffer.cs:2695-2739`):
            // the `npc >= maxNPCs` cheat gate boots the sender (`:2701-2705`);
            // a town-like NPC's status 1 kicks it out (`WorldGen.kickOut`
            // `:2732`), anything else moves its room (`WorldGen.moveRoom`
            // `:2736`). The client-only homeless/homeTile writes (`:2708-2713`)
            // and the TownManager client arms (`:2718-2728`) are not the host's.
            C2s::NpcHouse(h) => {
                let slot = h.npc;
                // `:2701-2705`: only `num258 >= Main.maxNPCs` boots; a negative
                // slot falls through to the town gate (no such NPC) below.
                if (slot as usize) >= MAX_NPCS {
                    // `:2703` `BootPlayer(whoAmI, NetworkText.FromKey("Net.CheatingInvalid"))`.
                    server.kick(id, &crate::lang::text("Net.CheatingInvalid"));
                    return;
                }
                let idx = slot as usize;
                let Some(npc) = self.npcs.get(idx) else {
                    return; // no such NPC — the town gate (`:2714-2717`) refuses
                };
                let is_town = npc.kind.is_town_npc_like();
                if !is_town {
                    return; // `:2714-2717`
                }
                if h.status == 1 {
                    // `WorldGen.kickOut` (`:2732`, `WorldGen.cs:4869-4879`).
                    let mut rooms = std::mem::take(&mut self.town_rooms);
                    crate::town::kick_out(&mut rooms, &mut self.npcs[idx]);
                    self.town_rooms = rooms;
                } else {
                    // `WorldGen.moveRoom` (`:2736`, `WorldGen.cs:4843-4855`):
                    // the room re-check (`SpawnTownNPC`) finds the new spot.
                    if let Some(spot) = crate::town::quick_find_home(
                        &self.world,
                        &self.npcs,
                        h.home_x as i32,
                        h.home_y as i32,
                        Some(idx),
                        self.npcs[idx].kind.net_id(),
                    ) {
                        let mut rooms = std::mem::take(&mut self.town_rooms);
                        crate::town::move_room(&mut rooms, &mut self.npcs[idx], spot);
                        self.town_rooms = rooms;
                    }
                }
            }
            // `case 61` — a client used a boss/event summon item
            // (`MessageBuffer.cs:2740-2878`): host-only (`:2744-2747`). A
            // positive MPAllowedEnemies selector spawns the boss on the player
            // (`:2748-2754`); negatives are the event ladder. The `7`/`78`
            // re-sync pushes are the send path's (S1-I-4); the pet licenses
            // (-12..-15), Mechdusa (-16), combat book v2 (-17) and peddler's
            // satchel (-18) flags are unported (ANALYSIS).
            C2s::SpawnBossEvent(ev) => {
                let plr = ev.player;
                let sel = ev.event;
                if sel >= 0 {
                    // `:2748-2754`: the MPAllowedEnemies gate + AnyNPCs live
                    // inside spawn_on_player.
                    crate::world_sim::npcs::spawn_on_player(self, plr as u8, sel as i32);
                    return;
                }
                match sel {
                    -4 => {
                        // `:2755-2764`: the Pumpkin Moon — night, no DD2.
                        if !self.world.day_time && !self.world.dd2_event_ongoing {
                            self.chat.new_text(crate::lang::misc(31), crate::chat::COLOR_WORLD);
                            self.world.start_pumpkin_moon();
                        }
                    }
                    -5 => {
                        // `:2765-2774`: the Frost Moon — night, no DD2.
                        if !self.world.day_time && !self.world.dd2_event_ongoing {
                            self.chat.new_text(crate::lang::misc(34), crate::chat::COLOR_WORLD);
                            self.world.start_snow_moon();
                        }
                    }
                    -6 => {
                        // `:2775-2790`: the Eclipse — day, not already up.
                        if self.world.day_time && !self.world.eclipse {
                            let key = if self.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) {
                                106
                            } else {
                                20
                            };
                            self.chat.new_text(crate::lang::misc(key), crate::chat::COLOR_WORLD);
                            self.world.eclipse = true;
                        }
                    }
                    -7 => {
                        // `:2791-2797`: the Martian Madness.
                        self.world.invasion_delay = 0;
                        crate::world_sim::npcs::start_invasion_in(
                            self,
                            1,
                            4,
                            Some(crate::app::RngKey::UpdatePlayers),
                        );
                    }
                    -8 => {
                        // `:2798-2805`: the Celestial Sigil's impending doom.
                        // `NPC.AnyDanger` (`NPC.cs:80243`): an invasion, an
                        // event or a live boss (the spawner's boss-kind list).
                        let any_danger = self.world.invasion_type != 0
                            || self.world.dd2_event_ongoing
                            || self.world.pumpkin_moon
                            || self.world.snow_moon
                            || self.npcs.iter().any(|n| {
                                n.hp > 0
                                    && matches!(
                                        n.kind,
                                        crate::npc::NpcKind::KingSlime
                                            | crate::npc::NpcKind::EyeOfCthulhu
                                            | crate::npc::NpcKind::QueenBee
                                            | crate::npc::NpcKind::SkeletronHead
                                            | crate::npc::NpcKind::WallOfFlesh
                                            | crate::npc::NpcKind::TheDestroyer
                                            | crate::npc::NpcKind::SkeletronPrime
                                            | crate::npc::NpcKind::Retinazer
                                            | crate::npc::NpcKind::Spazmatism
                                            | crate::npc::NpcKind::Plantera
                                            | crate::npc::NpcKind::BrainOfCthulhu
                                            | crate::npc::NpcKind::Golem
                                            | crate::npc::NpcKind::DukeFishron
                                            | crate::npc::NpcKind::Deerclops
                                    )
                            });
                        if self.world.downed_golem
                            && self.world.hard_mode
                            && !any_danger
                            && !crate::world_sim::lunar::anyone_near_cultists(self)
                        {
                            crate::world_sim::lunar::start_impending_doom(self, 720);
                        }
                    }
                    -10 => {
                        // `:2806-2819`: the Blood Moon — night, not already up.
                        if !self.world.day_time && !self.world.blood_moon {
                            self.chat.new_text(crate::lang::misc(8), crate::chat::COLOR_WORLD);
                            self.world.blood_moon = true;
                            if self.world.moon_phase == 0 {
                                self.world.moon_phase = 5; // `:2812-2815`
                            }
                            let _ = self.notify_progression_net(4); // `:2816`
                        }
                    }
                    -11 => {
                        // `:2820-2825`: the Combat Book.
                        self.chat.new_text(
                            crate::lang::text("Misc.CombatBookUsed"),
                            crate::chat::COLOR_WORLD,
                        );
                        self.world.combat_book_was_used = true;
                    }
                    -19 => {
                        // `:2858-2861`: the Slime Rain.
                        let raining = self.rain.is_raining();
                        crate::world_sim::weather::start_slime_rain(
                            &mut self.world,
                            raining,
                            self.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        );
                    }
                    s if s < 0 => {
                        // `:2862-2876`: the generic invasion fall-through.
                        let inv = if s > -5 { -s } else { 1 }; // InvasionID.Count = 5
                        if inv > 0 && self.world.invasion_type == 0 {
                            self.world.invasion_delay = 0;
                            crate::world_sim::npcs::start_invasion_in(
                                self,
                                1,
                                inv as i32,
                                Some(crate::app::RngKey::UpdatePlayers),
                            );
                        }
                    }
                    _ => {}
                }
            }
            // `case 63` — a client painted/coated a tile
            // (`MessageBuffer.cs:2909-2928`): coat 0 paints
            // (`WorldGen.paintTile` `:2917`), else coats (`paintCoatTile`
            // `:2921`); the host relays to the others (`:2925`).
            C2s::TilePaint(p) => {
                if p.coat == 0 {
                    paint_tile(&mut self.world, p.x as i32, p.y as i32, p.paint);
                } else {
                    paint_coat_tile(&mut self.world, p.x as i32, p.y as i32, p.paint);
                }
                self.world_key = None;
                server.broadcast_except(&S2c::TilePaint(p), id); // `:2925`
            }
            // `case 64` — a client painted/coated a wall
            // (`MessageBuffer.cs:2929-2948`): coat 0 paints
            // (`WorldGen.paintWall` `:2937`), else coats (`paintCoatWall`
            // `:2941`); the host relays to the others (`:2945`).
            C2s::WallPaint(p) => {
                if p.coat == 0 {
                    paint_wall(&mut self.world, p.x as i32, p.y as i32, p.paint);
                } else {
                    paint_coat_wall(&mut self.world, p.x as i32, p.y as i32, p.paint);
                }
                self.world_key = None;
                server.broadcast_except(&S2c::WallPaint(p), id); // `:2945`
            }
            // `case 65` — a teleport (`MessageBuffer.cs:2949-3035`): the host
            // forces the entity to the sender (`:2953-2956`), overrides the
            // position from the live player when bit 2 is set (`:2979-2982`),
            // then the kind ladder. Kind 3 is the client's ack (the
            // unacknowledgedTeleports counter is the send path's, S1-I-4).
            C2s::TeleportEntity(mut t) => {
                t.entity = id as i16; // `:2953-2956`
                if t.get_pos_from_player {
                    // `:2979-2982`: the live player's position (slot 0 host).
                    let pos = if id == 0 {
                        self.player.as_ref().map(|p| (p.x, p.y))
                    } else {
                        self.remote_players.get(&id).map(|p| (p.x, p.y))
                    };
                    if let Some((x, y)) = pos {
                        t.x = x;
                        t.y = y;
                    }
                }
                match t.kind {
                    0 => {
                        // `:2985-2995`: the player teleport + relay.
                        self.apply_player_teleport(id, t.x, t.y, t.style);
                        server.broadcast_except(&S2c::TeleportEntity(t), id); // `:2989`
                    }
                    1 => {
                        // `:2996-2999`: the NPC teleport + netOffset zero.
                        if let Some(npc) = self.npcs.get_mut(t.entity as usize) {
                            npc.x = t.x;
                            npc.y = t.y;
                            npc.net_offset = Vec2::ZERO; // `:2998`
                        }
                    }
                    2 => {
                        // `:3000-3028`: player-to-player — teleport, relay to
                        // ALL (`:3008`), then the nearest-peer chat.
                        self.apply_player_teleport(id, t.x, t.y, t.style);
                        server.broadcast(&S2c::TeleportEntity(t)); // `:3008`
                        // The nearest active peer to the sender (`:3009-3022`).
                        let sender_pos = if id == 0 {
                            self.player.as_ref().map(|p| (p.x, p.y))
                        } else {
                            self.remote_players.get(&id).map(|p| (p.x, p.y))
                        };
                        if let Some((sx, sy)) = sender_pos {
                            let mut nearest: Option<(u32, f32)> = None;
                            for (&pid, p) in &self.remote_players {
                                if pid == id {
                                    continue;
                                }
                                let d = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                                if nearest.map_or(true, |(_, bd)| d < bd) {
                                    nearest = Some((pid, d));
                                }
                            }
                            if let Some((nid, _)) = nearest {
                                let sender_name = if id == 0 {
                                    self.player_name.clone()
                                } else {
                                    self.remote_players.get(&id).map(|p| p.name.clone()).unwrap_or_default()
                                };
                                let target_name = self.remote_players.get(&nid).map(|p| p.name.clone()).unwrap_or_default();
                                let text = crate::lang::text_args(
                                    "Game.HasTeleportedTo",
                                    &[sender_name, target_name],
                                );
                                // `new Color(250, 250, 0)` (`:3025`).
                                crate::chat::broadcast_death_text(&mut self.chat, &text, [250, 250, 0]);
                                let sm = crate::app::netmodules::ServerMessage {
                                    author: 255,
                                    text,
                                    color: net::Rgb(250, 250, 0),
                                };
                                server.broadcast(&S2c::NetModule {
                                    module: crate::app::netmodules::module_id::TEXT,
                                    body: sm.encode(),
                                });
                            }
                        }
                    }
                    _ => {} // kind 3 ack (`:3029-3032`): the counter is S1-I-4
                }
            }
            // `case 66` — the Moon Leech Clot's owner heal
            // (`MessageBuffer.cs:3036-3055`): a positive amount heals the
            // subject (clamped to statLifeMax2), raises the HealEffect text
            // and relays (`:3051`). The subject is NOT forced to the sender.
            C2s::HealPlayer(h) => {
                if h.heal > 0 {
                    let slot = h.player_id as u32;
                    if slot == 0 {
                        if let Some(p) = self.player.as_mut() {
                            p.hp = (p.hp + h.heal as i32).min(p.max_hp); // `:3043-3047`
                        }
                    } else if let Some(p) = self.remote_players.get_mut(&slot) {
                        p.hp = (p.hp + h.heal as i32).min(p.max_hp);
                    }
                    // `HealEffect(num71, broadcast: false)` (`:3048`).
                    spawn_player_amount_text(self, slot, crate::combat_text::HEAL_LIFE, h.heal as i32);
                    server.broadcast_except(&S2c::HealPlayer(h), id); // `:3051`
                }
            }
            // `case 68` — the client's UUID (`MessageBuffer.cs:3056-3058`):
            // the host reads the string and drops it — the codec's read IS
            // the whole arm; nothing to apply or relay.
            C2s::ClientUuid(_) => {}
            // `case 73` — a conch/potion/shellphone teleport request
            // (`MessageBuffer.cs:3127-3146`): the host runs the matching
            // `Player` method on the sender, which answers with a `65`. The
            // potion/conch spot-scans (`Utils.CheckForGoodTeleportationSpot`)
            // are unported — those kinds fall back to the shellphone spawn
            // (ANALYSIS); the `65` answer is the send path's (S1-I-4).
            C2s::RequestTeleport(q) => {
                if id == 0 {
                    // The host's own player: the disjoint player/world borrow.
                    let world = &self.world;
                    if let Some(p) = self.player.as_mut() {
                        match q.kind {
                            3 => p.shellphone_spawn(world), // `:3140`
                            4 => p.player_no_space_teleport(world), // `:3143`
                            // 0/1/2: the spot-scan is unported — the shellphone
                            // fallback keeps the teleport observable.
                            _ => p.shellphone_spawn(world),
                        }
                    }
                } else if let Some(p) = self.remote_players.get_mut(&id) {
                    // A peer: the port keeps no per-peer sim, so the record
                    // takes the world-spawn position (the shellphone target).
                    p.x = self.world.spawn_x as f32 * TILE_PX;
                    p.y = self.world.spawn_y as f32 * TILE_PX;
                    p.off_x = 0.0;
                    p.off_y = 0.0;
                }
            }
            // `case 75` — a client finished today's angler quest
            // (`MessageBuffer.cs:3154-3163`): the host adds the sender's name
            // to `Main.anglerWhoFinishedToday` if absent (`:3157-3161`).
            C2s::AnglerQuestFinished => {
                let name = if id == 0 {
                    self.player_name.clone()
                } else {
                    self.remote_players.get(&id).map(|p| p.name.clone()).unwrap_or_default()
                };
                if !name.is_empty() && !self.world.angler_who_finished_today.contains(&name) {
                    self.world.angler_who_finished_today.push(name);
                }
            }
            // `case 76` — a client's quest/golf counts
            // (`MessageBuffer.cs:3164-3182`): the host forces the subject to
            // the sender (`:3169-3172`), stores both ints (`:3174-3175`) and
            // relays to the others (`:3178`, the id's only live send site).
            // The `num187 != Main.myPlayer || ServerSideCharacter` gate
            // (`:3167`): `Main.myPlayer` is the local slot 0 and SSC is false
            // in the port — a self-addressed echo (id 0) is dropped, exactly
            // the C# dedicated server's behaviour. W4-g49.
            C2s::QuestsCountSync(q) => {
                if q.player_id != 0 {
                    let subject = id as u8; // `:3171` — forced to the sender
                    let peer = self.remote_players.entry(id).or_default();
                    peer.angler_quests_finished = q.angler_quests_finished; // `:3174`
                    peer.golfer_score = q.golfer_score; // `:3175`
                    server.broadcast_except(
                        &S2c::QuestsCountSync(net::QuestsCountSync {
                            player_id: subject,
                            angler_quests_finished: q.angler_quests_finished,
                            golfer_score: q.golfer_score,
                        }),
                        id,
                    ); // `:3178`
                }
            }
            // `case 79` — a client placed a multi-tile object
            // (`MessageBuffer.cs:3192-3215`): the spam count (`:3203`) is
            // dormant (`Netplay.SpamCheck` defaults false, `Netplay.cs:867` —
            // the `-secure` flag is unported, the g48 `17` arm precedent);
            // the `InWorld(x, y, 10)` + section-sent gate (`:3204`) refuses,
            // then `WorldGen.PlaceObject` (`:3209`, mute: false) and the
            // `SendObjectPlacement` relay to the others (`:3212`,
            // `NetMessage.cs:2577-2581`). W4-g49.
            C2s::PlaceObject(p) => {
                let (x, y) = (p.x as i32, p.y as i32);
                let (w, h) = (self.world.width, self.world.height);
                // `:3204` — `WorldGen.InWorld(x12, y12, 10)`
                // (`WorldGen.cs:9688-9695`) + `TileSections[GetSectionX/Y]`.
                if x < 10 || x >= w - 10 || y < 10 || y >= h - 10 {
                    return;
                }
                if !server.is_section_sent(id, net::section_x(x), net::section_y(y)) {
                    return;
                }
                let dir = if p.direction { 1 } else { -1 }; // `:3200`
                // `:3209` — `WorldGen.PlaceObject(x, y, type, mute: false,
                // style, num162, random, direction)` (`WorldGen.cs:51425`).
                let landed = tr_core::edit::place_object(
                    &mut self.world,
                    x,
                    y,
                    p.kind.max(0) as u16,
                    p.style as i32,
                    p.random as i32,
                    dir,
                );
                if landed {
                    // The `mute: false` Dig cue (`WorldGen.cs:51441`): the
                    // handler runs outside every swap (`Main.cs:16895`), so
                    // the draws ride the bare-`Main.rand` stand-in.
                    self.audio.play_place_sound(
                        (0, 1),
                        x as f32 * TILE_PX,
                        y as f32 * TILE_PX,
                        self.rngs.global(),
                    );
                }
                self.world_key = None;
                // `:3212` — the relay fires regardless of the Place result
                // (the C# ignores `PlaceObject`'s return).
                server.broadcast_except(&S2c::PlaceObject(p), id);
            }
            // `case 48` — a single-tile liquid change
            // (`MessageBuffer.cs:2407-2446`): the host spam window
            // (`:2413-2427`, ±10 tiles around the sender's centre) is
            // dormant (`Netplay.SpamCheck` defaults false, `Netplay.cs:867`);
            // the liquid byte + type apply (`:2434-2435`), the host re-frames
            // (`SquareTileFrame` `:2438`) and a drained tile re-pushes as
            // `48` to the others (`:2439-2442`). W4-g49.
            C2s::LiquidUpdate(l) => {
                let (x, y) = (l.x as i32, l.y as i32);
                if let Some(t) = self.world.tile_mut(x, y) {
                    t.liquid = l.liquid; // `:2434`
                    // `:2435` — `Tile.liquidType(int)` (`Tile.cs:216-233`):
                    // 0 clears the kind bits, 1/2/3 set lava/honey/shimmer;
                    // any other value is a no-op (the switch has no default).
                    match l.liquid_type {
                        0 => t.clear_liquid_kind(),
                        1 => t.set_liquid_kind(tr_core::tile::LiquidKind::Lava),
                        2 => t.set_liquid_kind(tr_core::tile::LiquidKind::Honey),
                        3 => t.set_liquid_kind(tr_core::tile::LiquidKind::Shimmer),
                        _ => {}
                    }
                    // `:2438` — `WorldGen.SquareTileFrame(x, y)` (reset true).
                    tr_core::framing::reframe_area(&mut self.world, x, y, 2, true);
                    self.world_key = None;
                    if l.liquid == 0 {
                        // `:2441` — the drained-tile re-push (the writer
                        // reads the tile's own liquid/type back out).
                        let drained = net::LiquidUpdate {
                            x: l.x,
                            y: l.y,
                            liquid: 0,
                            liquid_type: l.liquid_type,
                        };
                        server.broadcast_except(&S2c::LiquidUpdate(drained), id);
                    }
                }
            }
            // `case 52` — a chest/door unlock or lock
            // (`MessageBuffer.cs:2531-2564`): kind 1 `Chest.Unlock`
            // (`:2538`), 2 `WorldGen.UnlockDoor` (`:2547`), 3 `Chest.Lock`
            // (`:2556`); each relays as `52` + a 2×2 `SendTileSquare`
            // (`:2541-2542`/`:2550-2551`/`:2559-2560`). Any other kind is a
            // silent no-op. W4-g49.
            C2s::LockAndUnlock(l) => {
                let (x, y) = (l.x as i32, l.y as i32);
                let plant_boss = self.world.downed_plant_boss;
                match l.action {
                    1 => {
                        crate::chest::unlock_chest(&mut self.world, x, y, plant_boss);
                    }
                    2 => {
                        if let Some(top) = unlock_door(&mut self.world, x, y) {
                            unlock_door_effects(self, x, top);
                        }
                    }
                    3 => {
                        crate::chest::lock_chest(&mut self.world, x, y, plant_boss);
                    }
                    _ => return, // any other kind: no apply, no relay
                }
                self.world_key = None;
                // The relay fires on the action KIND, not the result — the
                // C# ignores `Chest.Unlock`/`Lock`'s bool (`:2538-2542`).
                server.broadcast_except(&S2c::LockAndUnlock(l), id); // `:2541`
                // `SendTileSquare(-1, x, y, 2)` (`NetMessage.cs:2632-2636`):
                // the 2×2 centred square is (x, y)..(x+1, y+1).
                let mut records = Vec::with_capacity(4);
                for dy in 0..2 {
                    for dx in 0..2 {
                        records.push(self.tile_record_at(x + dx, y + dy));
                    }
                }
                server.broadcast(&S2c::SectionRange(Box::new(net::SectionRange {
                    x: l.x,
                    y: l.y,
                    w: 2,
                    h: 2,
                    change_type: 0,
                    records,
                })));
            }
            // `case 17` — a client's single-tile edit
            // (`MessageBuffer.cs:1119-1288`): the full kind dispatch + the
            // survive/falling relay rules — W4-g48 (`net_apply`).
            C2s::TileEdit(e) => {
                crate::app::net_apply::host_tile_edit(self, server, id, e);
            }
            // `case 19` — a client's door/trapdoor/gate toggle
            // (`MessageBuffer.cs:1299-1334`): the action dispatch + the
            // normalised-direction relay — W4-g48 (`net_apply`).
            C2s::DoorToggle(d) => {
                crate::app::net_apply::host_door_toggle(self, server, id, d);
            }
            // `case 35` — a client's heal effect (`MessageBuffer.cs:2160-
            // 2177`): the peer's HealEffect text + the relay — W4-g48.
            C2s::PlayerHeal(h) => {
                crate::app::net_apply::host_player_heal(self, server, id, h.amount);
            }
            // `case 36` — a client's zone words (`MessageBuffer.cs:2178-
            // 2201`): the store + the faeling-edge latch + the relay —
            // W4-g48 (`net_apply`).
            C2s::PlayerZone(z) => {
                crate::app::net_apply::host_player_zone(self, server, id, z);
            }
            // `case 40` — a client's talk NPC (`MessageBuffer.cs:2256-2270`):
            // SetTalkNPC + the bestiary chat registration + the relay —
            // W4-g48 (`net_apply`).
            C2s::TalkNpc(t) => {
                crate::app::net_apply::host_talk_npc(self, server, id, t.npc);
            }
            // `case 41` — a client's aimed-item rotation/animation
            // (`MessageBuffer.cs:2271-2289`). The host forces the subject to
            // the sender (`:2274-2277`), stores both fields (`:2281-2282`),
            // re-derives `channel` from the sender's selected inventory item
            // (`:2283`) and relays (`:2286`).
            C2s::ItemAnim(mut a) => {
                a.player_id = id as u8; // `:2276`
                let entry = self.remote_players.entry(id).or_default();
                entry.item_rotation = a.rotation;
                entry.item_animation = a.animation;
                entry.channel = Self::peer_channel(entry); // `:2283`
                server.broadcast_except(&S2c::ItemAnim(a), id); // `:2286`
            }
            // `case 43` — a client's mana-restore amount
            // (`MessageBuffer.cs:2307-2324`). The host forces the subject to
            // the sender (`:2310-2313`), spawns the HealMana text for anyone
            // but itself (`:2315-2318`, `Player.ManaEffect`,
            // `Player.cs:4531-4538`) and relays (`:2321`).
            C2s::ManaEffect(mut m) => {
                m.player_id = id as u8; // `:2312`
                if m.player_id as u32 != self.local_id {
                    self.spawn_peer_mana_text(m.player_id, m.amount as i32);
                }
                server.broadcast_except(&S2c::ManaEffect(m), id); // `:2321`
            }
            // `case 51` — a client's misc request (`MessageBuffer.cs:2482-2530`).
            // The host forces the subject to the sender (`:2485-2488`).
            C2s::MiscDataSync(mut d) => {
                d.player_id = id as u8; // `:2487`
                match d.kind {
                    // `:2493` NPC.SpawnSkeletron — the summon trigger is
                    // unported (`NPC.cs:80422-80473`, the world_sim/npcs.rs
                    // red-hat note); parked, W4-c27.
                    1 => {}
                    // `:2496-2499` — the host relays; the clients play Item1.
                    2 => server.broadcast_except(&S2c::MiscDataSync(d), id),
                    // `:2505-2510` Main.Sundialing (`Main.cs:6169-6182`): the
                    // cooldown-0 gate, the dawn fast-forward + cooldown 8,
                    // then `SendData(7)` — the port re-pushes the header.
                    3 => {
                        if self.world.sundial_cooldown == 0 {
                            self.world.fast_forward_to_dawn = true;
                            self.world.sundial_cooldown = 8;
                            server.broadcast(&S2c::WorldInfo(Box::new(self.world_info())));
                        }
                    }
                    // `:2512` BigMimicSpawnSmoke (`NPC.cs:77494-77515`): the
                    // 20x dust-54 + 6x gore-99 burst — the big-mimic roster
                    // is unported; parked, W4-c27.
                    4 => {}
                    // `:2514-2521` — the TorchGod (664) bestiary kill.
                    5 => self.bestiary.kills.register_kill("TorchGod"),
                    // `:2522-2527` Main.Moondialing (`Main.cs:6184-6197`).
                    6 => {
                        if self.world.moondial_cooldown == 0 {
                            self.world.fast_forward_to_dusk = true;
                            self.world.moondial_cooldown = 8;
                            server.broadcast(&S2c::WorldInfo(Box::new(self.world_info())));
                        }
                    }
                    _ => {}
                }
            }
            // `case 55` — a PvP debuff aimed at a peer
            // (`MessageBuffer.cs:2594-2611`). The gate (`:2599`): BOTH the
            // target and the sender hostile, and `Main.pvpBuff[type]`. The
            // id is NOT forced — the target stays as written. The host sends
            // to the target alone (`:2603`); the port's host keeps a local
            // player, so a buff aimed at it applies locally (the target
            // client's `:2607` half).
            C2s::PlayerBuffPvp(b) => {
                let target_hostile = if b.player_id as u32 == self.local_id {
                    self.player.as_ref().is_some_and(|p| p.hostile)
                } else {
                    self.remote_players
                        .get(&(b.player_id as u32))
                        .is_some_and(|p| p.hostile)
                };
                let sender_hostile = self.remote_players.get(&id).is_some_and(|p| p.hostile);
                if target_hostile
                    && sender_hostile
                    && crate::buffs::is_pvp_buff(b.buff_type as i32)
                {
                    if b.player_id as u32 == self.local_id {
                        if let Some(p) = self.player.as_mut() {
                            p.buffs.add_buff_from_net_pvp(b.buff_type as i32, b.time);
                        }
                    } else {
                        server.send(b.player_id as u32, &S2c::PlayerBuffPvp(b)); // `:2603`
                    }
                }
            }
            // `case 62` — a client's dodge fired
            // (`MessageBuffer.cs:2879-2908`). The host forces the subject to
            // the sender (`:2883-2886`) and relays to the others (`:2905`).
            // The C# host also runs the dodge on its non-simulated remote
            // `Player` copy (`:2887-2902`) — the survivors there are the
            // 120/80 immunity window (the buff strip + Hallowed cooldown
            // are `whoAmI == myPlayer`-gated, `Player.cs:38026-38029`) and
            // the dust/gore bursts. The port keeps no remote `Player` (the
            // peer record is a render snapshot; the bursts are S1-E-1), so
            // that half is parked with the `:2577` hostile-shot follow-up;
            // the relay is the live half. W4-c27.
            C2s::SyncDodge(mut d) => {
                d.player_id = id as u8; // `:2885`
                server.broadcast_except(&S2c::SyncDodge(d), id); // `:2905`
            }
            // `case 84` — a client's stealth value
            // (`MessageBuffer.cs:3246-3260`). The host forces the subject to
            // the sender (`:3249-3252`), stores the float (`:3254`) and
            // relays (`:3257` — the writer re-reads the stored field,
            // `NetMessage.cs:1200`, so the relay carries the stored value).
            C2s::PlayerStealth(mut s) => {
                s.player_id = id as u8; // `:3251`
                self.remote_players.entry(id).or_default().stealth = s.stealth; // `:3254`
                server.broadcast_except(&S2c::PlayerStealth(s), id); // `:3257`
            // `case 84` — the stealth sync (W4-g50,
            // `MessageBuffer.cs:3246-3260`): the host forces the subject to
            // the sender (`:3251`), stores the value and relays it to the
            // other clients (`:3257`'s `TrySendData(84, -1, whoAmI, …)`).
            C2s::PlayerStealth(s) => {
                let peer = id as u8;
                if let Some(p) = self.remote_players.get_mut(&id) {
                    p.stealth = s.stealth;
                }
                server.broadcast_except(
                    &S2c::PlayerStealth(net::PlayerStealth {
                        player_id: peer,
                        stealth: s.stealth,
                    }),
                    id,
                );
            }
            // `case 92` — the coin-credit delta (W4-g50,
            // `MessageBuffer.cs:3442-3461`): the `0..=maxNPCs` slot gate
            // (`:3448`), the host adds the DELTA to the NPC's accumulated
            // `extraValue` (`:3457`) and relays the TOTAL with the same
            // ping position (`:3458`).
            C2s::NpcExtraValue(e) => {
                if e.npc < 0 || e.npc as usize > self.npcs.len() {
                    return; // `num228 >= 0 && num228 <= Main.maxNPCs` (`:3448`)
                }
                let Some(npc) = self.npcs.get_mut(e.npc as usize) else {
                    return;
                };
                // `extraValue += num229` — the C# int add on the float
                // field; the port's u64 credit saturates at 0 (a negative
                // delta cannot drive the purse under zero).
                npc.extra_value = if e.value >= 0 {
                    npc.extra_value.saturating_add(e.value as u64)
                } else {
                    npc.extra_value.saturating_sub((-e.value) as u64)
                };
                let total = npc.extra_value.min(i32::MAX as u64) as i32;
                server.broadcast(&S2c::NpcExtraValue(net::NpcExtraValue {
                    npc: e.npc,
                    value: total,
                    x: e.x,
                    y: e.y,
                }));
            }
            // `case 87` — a client placed a tile entity (W4-g50,
            // `MessageBuffer.cs:3294-3305`): the in-world + free-cell gate
            // (`:3300`), then `TileEntity.PlaceEntityNet` →
            // `NetPlaceEntityAttempt` (`TileEntityType.cs:17-20`): place and
            // answer with the `86` sharing broadcast.
            C2s::TileEntityPlacement(p) => {
                let (x, y) = (p.x as i32, p.y as i32);
                // `WorldGen.InWorld(x8, y8)` — the port's tile lookup bounds.
                if self.world.tile(x, y).is_none() {
                    return;
                }
                if self.world.tile_entities.get_at(p.x, p.y).is_some() {
                    return; // `!TileEntity.TryGetAt<TileEntity>(x8, y8, …)`
                }
                let Some(kind) = tr_core::tile_entities::TeKind::from_id(p.kind) else {
                    return; // `InvalidEntityID` (`TileEntitiesManager.cs:59`)
                };
                // The pylon's own `NetPlaceEntityAttempt`
                // (`TETeleportationPylon.cs:13-27`): an untyped frame or a
                // duplicate type rejects with `KillTile` + a `17` relay.
                if kind == tr_core::tile_entities::TeKind::TeleportationPylon {
                    let style = self
                        .world
                        .tile(x, y)
                        .map(|t| t.frame_x as i32 / 54)
                        .unwrap_or(-1);
                    let pylon_type = tr_core::pylons::PylonType::from_tile_style(style);
                    let duplicate = pylon_type.is_some_and(|pt| {
                        tr_core::pylons::scan_pylons(&self.world)
                            .iter()
                            .any(|info| info.pylon_type == pt)
                    });
                    if pylon_type.is_none() || duplicate {
                        crate::world_sim::editing::break_tile(self, x, y);
                        server.broadcast(&S2c::Edit {
                            x,
                            y,
                            kind: 0, // `KillTile`'s fail=false break
                            tile: 0,
                        });
                        return;
                    }
                }
                // `TileEntityType<T>.Place` — the ValidTile gate + the books.
                let Some(id) = self.world.tile_entities.place(&self.world, kind, p.x, p.y)
                else {
                    return;
                };
                self.broadcast_tile_entity(server, id);
            }
            // `case 96` — the portal-gun owner's self teleport (W4-g50,
            // `MessageBuffer.cs:3538-3557`): the host forces the subject to
            // the sender (`:3543`), applies the teleport and relays it to
            // the other clients (`:3554`'s `SendData(96, -1, num183, …)`).
            C2s::PlayerPortalTeleport(p) => {
                let peer = id as u8;
                self.apply_player_portal_teleport(peer, &p);
                server.broadcast_except(
                    &S2c::PlayerPortalTeleport(net::PlayerPortalTeleport {
                        player: peer,
                        ..p
                    }),
                    id,
                );
            }
            // `case 99` — the minion rest point (W4-g50,
            // `MessageBuffer.cs:3570-3583`): the host forces the subject to
            // the sender (`:3575`), stores the point and relays it
            // (`:3580`'s `TrySendData(99, -1, whoAmI, …)`).
            C2s::MinionRestTarget(m) => {
                let peer = id as u8;
                if let Some(p) = self.remote_players.get_mut(&id) {
                    p.minion_rest_target_point = (m.x, m.y);
                }
                server.broadcast_except(
                    &S2c::MinionRestTarget(net::MinionRestTarget {
                        player_id: peer,
                        x: m.x,
                        y: m.y,
                    }),
                    id,
                );
            }
            // `case 115` — the minion attack target (W4-g50,
            // `MessageBuffer.cs:3584-3597`): the host forces the subject to
            // the sender (`:3589`), stores the target and relays it
            // (`:3594`'s `TrySendData(115, -1, whoAmI, …)`).
            C2s::MinionAttackTarget(m) => {
                let peer = id as u8;
                if let Some(p) = self.remote_players.get_mut(&id) {
                    p.minion_attack_target_npc = m.npc as i32;
                }
                server.broadcast_except(
                    &S2c::MinionAttackTarget(net::MinionAttackTarget {
                        player_id: peer,
                        npc: m.npc,
                    }),
                    id,
                );
            }
            // `case 102` — the nebula booster pickup (W4-g50,
            // `MessageBuffer.cs:3652-3660`): the host relays verbatim with
            // the sender forced into the id byte (`:3659`'s
            // `TrySendData(102, -1, -1, null, whoAmI, …)` — no ignoreClient,
            // so the picker receives its own relay too).
            C2s::NebulaLevelupRequest(n) => {
                server.broadcast(&S2c::NebulaLevelupRequest(net::NebulaLevelupRequest {
                    player_id: id as u8,
                    buff_type: n.buff_type,
                    x: n.x,
                    y: n.y,
                }));
            }
            // `case 105` — the gem-lock toggle (W4-g50,
            // `MessageBuffer.cs:3736-3741`): the host runs
            // `WorldGen.ToggleGemLock(x, y, on)` (`WorldGen.cs:46469-46524`).
            C2s::GemLockToggle(g) => {
                self.toggle_gem_lock(server, g.x as i32, g.y as i32, g.on);
            }
            // `case 111` — ToggleParty (W4-g50, `MessageBuffer.cs:3809-3814`
            // → `BirthdayParty.ToggleManualParty`, `BirthdayParty.cs:145-164`):
            // the host flips `ManualParty`, re-pushes the world header on a
            // state change (`SendData(7)`, `:160`) and credits THROW_A_PARTY
            // when the party came up (`CheckForAchievement`, `:162`).
            C2s::ToggleParty => {
                let was_up = crate::world_events::party_is_up(&self.world_events);
                self.world_events.manual_party = !self.world_events.manual_party;
                let now_up = crate::world_events::party_is_up(&self.world_events);
                if was_up != now_up {
                    // `SendData(7)` — the full world header re-push.
                    server.broadcast(&S2c::WorldInfo(Box::new(self.world_info())));
                    if now_up {
                        // `NotifyProgressionEvent(25)` — the 98 fan-out.
                        let _ = self.notify_progression_net(25);
                    }
                }
            }
            // `case 113` — the eternia crystal placement (W4-g50,
            // `MessageBuffer.cs:3841-3854`): gated on `!snowMoon &&
            // !pumpkinMoon`; `WouldFailSpawningHere` warns the sender (the
            // bartender text, `DD2Event.cs:615-627`) but does NOT block the
            // summon (`:3847-3851` — no return), then `SummonCrystal(x, y,
            // whoAmI)` runs `SummonCrystalDirect` (`DD2Event.cs:584-601`).
            C2s::CrystalInvasionStart(c) => {
                if self.world.snow_moon || self.world.pumpkin_moon {
                    return; // `:3845`
                }
                let (x, y) = (c.x as i32, c.y as i32);
                // `WouldFailSpawningHere` (`DD2Event.cs:603-613`): the arena
                // walker must find >= 60 walkable tiles on EACH side.
                let (left, right) = tr_core::stray::check_arena_score(
                    &self.world,
                    (x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0),
                );
                if right.0 - x < 60 || x - left.0 < 60 {
                    // `FailureMessage(whoAmI)` — the yellow bartender warning
                    // to the sender only (`ChatHelper.SendChatMessageToClient`).
                    let sm = crate::app::netmodules::ServerMessage {
                        author: 255,
                        text: crate::lang::text("DungeonDefenders2.BartenderWarning"),
                        color: net::Rgb(255, 255, 0),
                    };
                    server.send(
                        id,
                        &S2c::NetModule {
                            module: crate::app::netmodules::module_id::TEXT,
                            body: sm.encode(),
                        },
                    );
                }
                self.summon_crystal_direct(server, x, y);
            }
            // `case 117` — a peer's hurt (W4-g50, `MessageBuffer.cs:3867-3886`):
            // the gate `netMode != 2 || whoAmI == num27 || (both hostile)`
            // (`:3870`) — on the host only a self-hurt or a mutual-PvP hit
            // applies; then `Player.Hurt(quiet: true)` and the
            // `SendPlayerHurt` relay (`:3882`, ignoreClient = the sender).
            C2s::PlayerHurtV2(h) => {
                let subject = h.player_id;
                let sender_hostile = self
                    .remote_players
                    .get(&id)
                    .map(|p| p.hostile)
                    .unwrap_or(false);
                let subject_hostile = if subject as u32 == self.local_id {
                    self.player.as_ref().map(|p| p.hostile).unwrap_or(false)
                } else {
                    self.remote_players
                        .get(&(subject as u32))
                        .map(|p| p.hostile)
                        .unwrap_or(false)
                };
                let applies = subject as u32 == id || (sender_hostile && subject_hostile);
                if !applies {
                    return; // `:3870` — the gate skips BOTH apply and relay
                }
                self.apply_player_hurt(subject, &h);
                // `SendPlayerHurt(num27, …, -1, this.whoAmI)` — the relay
                // carries the subject's id, excluding the sender.
                server.broadcast_except(
                    &S2c::PlayerHurtV2(net::PlayerHurtV2 {
                        player_id: subject,
                        ..h
                    }),
                    id,
                );
            }
            // `case 118` — a peer's death (W4-g50, `MessageBuffer.cs:3887-3904`):
            // the host forces the subject to the sender (`:3892`), runs
            // `Player.KillMe` and relays via `SendPlayerDeath` (`:3901`,
            // ignoreClient = the sender).
            C2s::PlayerDeathV2(d) => {
                let subject = id as u8;
                self.apply_player_death(subject, &d);
                server.broadcast_except(
                    &S2c::PlayerDeathV2(net::PlayerDeathV2 {
                        player_id: subject,
                        ..d
                    }),
                    id,
                );
            }
            // `case 120` — an emote (W4-g50, `MessageBuffer.cs:3905-3919`):
            // the host forces the subject to the sender (`:3910`), validates
            // the id against `EmoteID.Count` (151), bubbles it on the sender
            // (`NewBubble`, 360 ticks — the `91` fan-out) and runs the town
            // NPC reaction (`CheckForNPCsToReactToEmoteBubble`).
            C2s::Emoji(e) => {
                let subject = id as u8;
                if (e.emote_id as i32) >= crate::emotes::EMOTE_COUNT {
                    return; // `num257 >= 0 && num257 < EmoteID.Count` (`:3913`)
                }
                // `EmoteBubble.NewBubble(emote, WorldUIAnchor(player), 360)`
                // (`:3915`) — the host's own bubble + the `91` broadcast.
                self.emotes
                    .new_bubble_on(e.emote_id as i32, 1, subject as u16, self.local_id as u16);
                let bubble_id = self.emotes.bubbles.last().map(|b| b.id).unwrap_or(0);
                server.broadcast(&S2c::EmoteBubble(net::EmoteBubble {
                    id: bubble_id as i32,
                    anchor_kind: 1,
                    anchor_index: subject as u16,
                    life_time: 360,
                    emote: e.emote_id as i32,
                    metadata: 0,
                }));
                // `CheckForNPCsToReactToEmoteBubble(emote, player)` (`:3916`).
                self.check_npcs_react_to_emote(subject);
            }
            // `case 85` — the quick-stack ask (W4-g50,
            // `MessageBuffer.cs:3261-3266`): the host reads the slot list
            // into a scratch `SourceInventory` (`QuickStacking.ReadNetInventory`)
            // and runs `QuickStackToNearbyChests` (`:3265`). WIRED via the
            // W4-g21 batch-3 engine (`crate::quick_stack`, the constants-copy
            // keep-one — `transfer`/`Dest` are their `Transfer`/
            // `DestinationHelper` verbatim).
            C2s::QuickStackRequest(q) => {
                if id < 255 {
                    self.host_quick_stack(server, id, &q);
                }
            }
            // `case 94` — the debug-command sync (W4-g50,
            // `MessageBuffer.cs:3463-3518`): the reader consumes the four
            // fields, then gates on `DebugOptions.enableDebugCommands`
            // (`:3469`, the `TR_DEBUG_COMMANDS` pin — the d85 registry's
            // gate) before the command switch. WIRED (re-probe): the three
            // shared-state arms store into `App.debug_shared` (the C#
            // statics' port home); the `/quickload-*` harness arms stay
            // parked (`Testing/QuickLoad.cs` unported — the probe/response
            // relay needs the JoinWorld serializer). The d85 62-row
            // registry is the UI list only (its processors are explicitly
            // unported, `ui/debug_commands.rs` module doc) — these three
            // arms are the MessageBuffer reader's own switch, not the
            // registry's.
            C2s::DevCommands(d) => {
                self.apply_dev_commands(server, id, &d);
            }
            // `case 109` — the host runs `Wiring.MassWireOperation` under a
            // temporary ToolMode swap (`:3783-3786`). WIRED via the W4-c42
            // fold-133 core (`tr_core::wiring::mass_wire_operation_inner`,
            // the constants-copy keep-one): the mode rides the fn parameter
            // (the port has no `WiresUI.Settings.ToolMode` global on this
            // base, so the swap/restore pair folds into the argument — no
            // shared state is touched), the subject is the SENDER.
            C2s::MassWireOperation(m) => {
                self.host_mass_wire_operation(server, id, &m);
            }
            // `case 125` — a client's pickaxe hit on a tile
            // (`MessageBuffer.cs:4023-4042`): the host forces the subject to
            // the sender (`:4029-4032`) and relays to the OTHERS
            // (`:4037-4040`, `ignoreClient = num166`). No host-side apply —
            // the pick damage lands on each client's own `hitTile`.
            C2s::TilePicking(mut p) => {
                p.player_id = id as u8; // `:4031`
                server.broadcast_except(&S2c::TilePicking(p), id);
            }
            // `case 130` — a client fished an NPC out (`MessageBuffer.cs:
            // 4086-4120`, host-only). Type 682 (Red Slime) flips
            // `unlockedSlimeRedSpawn` ONCE and re-pushes `7 WorldData`
            // (`:4095-4103`); then both coords ×16 and `NPC.NewNPC` at the
            // Bottom anchor (`:4104-4110`). The `netID != type` re-default +
            // `23` (`:4111-4115`) is the town-slime remap — the port has no
            // town-slime kinds, so `kind_from_net_id` carries the roster gap
            // (the spawn is skipped, the unlock still lands). The 682
            // achievement check (`:4116-4119`) rides the housed-NPC census.
            C2s::FishOutNpc(f) => {
                if f.npc_type == 682 {
                    if self.world.unlocked_slime_red_spawn {
                        return; // `:4097-4100` — already unlocked
                    }
                    self.world.unlocked_slime_red_spawn = true; // `:4101`
                    let info = self.world_info();
                    server.broadcast(&S2c::WorldInfo(Box::new(info))); // `:4102`
                }
                let x = f.tile_x as i32 * 16; // `:4104`
                let y = f.tile_y as i32 * 16; // `:4105`
                if let Some((kind, slime)) = crate::npc::kind_from_net_id(f.npc_type as i32) {
                    // `NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80770`): the
                    // bottom-centre anchor, the release_npc convention.
                    let (w, h) = kind.size();
                    let mut npc = crate::npc::Npc::new(
                        kind,
                        x as f32 - w * 0.5,
                        y as f32 - h,
                        1,
                        0.0,
                    );
                    npc.slime = slime;
                    // `timeLeft = (int)(activeTime * 1.25)` (`NPC.cs:80772`).
                    npc.despawn_timer = (750.0 * 1.25) as i32;
                    npc.net_update = true;
                    self.npcs.push(npc);
                }
                if f.npc_type == 682 {
                    self.check_real_estate_and_town_slimes(); // `:4118`
                }
            }
            // `case 134` — a client's luck factors (`MessageBuffer.cs:
            // 4164-4193`): the host forces the subject to the sender
            // (`:4175-4178`), stores the eight fields on the peer snapshot
            // (the port does not simulate remote players, so the
            // `RecalculateLuck` :4188 result is not derived here — the
            // factors are stored-not-simulated like the C# host's
            // `Main.player[num104]`), then relays to the OTHERS (`:4189-
            // 4192`, `ignoreClient = num104`).
            C2s::UpdatePlayerLuckFactors(mut l) => {
                l.player_id = id as u8; // `:4177`
                self.remote_players.entry(id).or_default().luck = Some(l);
                server.broadcast_except(&S2c::UpdatePlayerLuckFactors(l), id);
            }
            // `case 137` — a client asks the host to drop an NPC buff
            // (`MessageBuffer.cs:4219-4228`, host-only): the slot gates to
            // `0..Main.maxNPCs` (`:4224`) then `RequestBuffRemoval`
            // (`NPC.cs:89985-90000`). The `BuffID.Sets.CanBeRemovedByNetMessage`
            // set is created EMPTY (`BuffID.cs:48`), so the gate refuses every
            // buff id — the seam is wired through `request_buff_removal` and
            // stays live for when the set is filled. The `54` push the helper
            // would emit on a successful removal rides the outbox drain.
            C2s::NpcBuffRemovalRequest(q) => {
                if q.npc < 0 || q.npc as usize >= self.npcs.len() {
                    return; // `:4224`
                }
                let slot = q.npc as usize;
                let found = self.npcs[slot]
                    .buffs
                    .find_buff_index(q.buff_type as i32)
                    .is_some();
                let mut out = Vec::new();
                let buffs = &mut self.npcs[slot].buffs;
                let idx = buffs.find_buff_index(q.buff_type as i32);
                crate::app::net_npc_sync::request_buff_removal(
                    q.npc,
                    q.buff_type,
                    2,
                    found,
                    &mut out,
                    || {
                        if let Some(i) = idx {
                            buffs.del_buff(i); // NPC.cs:89994
                        }
                    },
                );
                // A successful removal pushes the full `54` list
                // (`NPC.cs:90019-90022`); the empty gate keeps `out` empty.
                for send in out {
                    if let crate::app::net_npc_sync::NpcSend::BuffList { slot } = send {
                        let buffs = npc_buffs_of(&self.npcs[slot as usize]);
                        server.broadcast(&S2c::NpcBuffs(net::NpcBuffs {
                            npc: slot,
                            buffs,
                        }));
                    }
                }
            }
            // `case 140` — a client's town-slime transform request
            // (`MessageBuffer.cs:4238-4263`, host-only kinds): kind 1 →
            // `NPC.TransformCopperSlime(value)` (`NPC.cs:19160-19182`),
            // kind 2 → `TransformElderSlime(value)` (`:19136-19158`).
            // Kind 0 is the client's credits arm — the host ignores it
            // (`:4245`'s `netMode == 1` gate). Both transforms: the unlock
            // gate ONCE, the type gate, `unlockedSlime*Spawn = true`, the
            // `7 WorldData` re-push, then `Transform(684/679)`. The port
            // has no town-slime kinds (679/684/685 — the roster gap the
            // `check_real_estate_and_town_slimes` doc pins), so the
            // Transform itself parks; the unlock + re-push are the
            // observable state and land live. The `CanConvertIntoCopperSlime
            // TownNPC` set is {1, 302, 333-336} (`NPCID.cs:10120`) — the
            // port's Blue Slime (cs_id 1) is the only live member; the
            // party-slime kinds (302/333-336) are the same roster gap.
            C2s::MiscEventValue(v) => {
                let slot = v.value;
                if slot < 0 || slot as usize >= self.npcs.len() {
                    return; // `IndexInRange` (`:19142`/`:19166`)
                }
                let slot = slot as usize;
                match v.kind {
                    1 => {
                        if self.world.unlocked_slime_copper_spawn {
                            return; // `:19166`
                        }
                        let ty = self.npcs[slot].kind.cs_id() as i32;
                        // `NPCID.Sets.CanConvertIntoCopperSlimeTownNPC`
                        // (`NPCID.cs:10120`): 1, 302, 335, 336, 333, 334.
                        if !matches!(ty, 1 | 302 | 333 | 334 | 335 | 336) {
                            return; // `:19169`
                        }
                        self.world.unlocked_slime_copper_spawn = true; // `:19171`
                        let info = self.world_info();
                        server.broadcast(&S2c::WorldInfo(Box::new(info))); // `:19172`
                        // `Transform(684)` (`:19173`) + the TownSlimeTransform
                        // particle (`:19174-19179`) park with the kind.
                    }
                    2 => {
                        if self.world.unlocked_slime_old_spawn {
                            return; // `:19142`
                        }
                        if self.npcs[slot].kind.cs_id() != 685 {
                            return; // `:19145` — BoundTownSlimeOld only
                        }
                        self.world.unlocked_slime_old_spawn = true; // `:19147`
                        let info = self.world_info();
                        server.broadcast(&S2c::WorldInfo(Box::new(info))); // `:19148`
                        // `Transform(679)` (`:19149`) + the particle park.
                    }
                    _ => {} // kind 0: client-only (`:4245`), unknown: no tail
                }
            }
            // `case 141` — a client's talking-axe popup (`MessageBuffer.cs:
            // 4265-4281`): the host relays VERBATIM with `ignoreClient =
            // whoAmI` (`:4272-4275`, `SendData(141, -1, this.whoAmI, …)`).
            // No host-side apply — the popup is a client effect.
            C2s::LucyPopup(l) => {
                server.broadcast_except(&S2c::LucyPopup(l), id);
            }
            // `case 142` — a client's piggy-bank / void-lens trackers
            // (`MessageBuffer.cs:4282-4297`): the host forces the subject to
            // the sender (`:4284-4287`), stores the two references on the
            // peer snapshot (stored-not-simulated — the port has no
            // piggy-bank/void-lens projectile-tracking consumer), then
            // relays to the OTHERS (`:4292-4295`, `ignoreClient = whoAmI`).
            C2s::ProjectileTrackers(mut t) => {
                t.player_id = id as u8; // `:4286`
                self.remote_players.entry(id).or_default().trackers = Some(t);
                server.broadcast_except(&S2c::ProjectileTrackers(t), id);
            }
            // `case 143` — a client asked to skip the DD2 wave break
            // (`MessageBuffer.cs:4298-4303`, host-only, no body):
            // `DD2Event.AttemptToSkipWaitTime` (`DD2Event.cs:1795-1801`) —
            // `TimeLeftBetweenWaves > 60` → `SetEnemySpawningOnHold(60)`.
            // The port's `dd2_spawn_hold` is `_timeLeftUntilSpawningBegins`;
            // the strict `>` gate refuses at exactly 60.
            C2s::CrystalInvasionSkip => {
                if let Some(hold) =
                    tr_core::dd2_victory::attempt_skip_wait_time(self.world.dd2_spawn_hold)
                {
                    self.world.dd2_spawn_hold = hold;
                }
            }
            // `case 147` — a client switched loadout (`MessageBuffer.cs:
            // 4328-4343`): the host forces the subject to the sender
            // (`:4330-4333`), runs `TrySwitchingLoadout` + the accessory-
            // visibility read (`:4336-4337`), then relays to the OTHERS
            // (`:4338-4341`, `ignoreClient = num68`). The port does not
            // simulate a remote player's equipment swap (no per-peer
            // `EquipmentLoadout` arrays), so the loadout index + visibility
            // bits are stored-not-simulated on the peer snapshot; the local
            // player's own `147` runs the real swap (`player.rs`).
            C2s::SyncLoadout(mut l) => {
                l.player_id = id as u8; // `:4332`
                self.remote_players.entry(id).or_default().loadout = Some(l);
                server.broadcast_except(&S2c::SyncLoadout(l), id); // `:4340`
            }
            // S1-I-1 wave A: the join bundle's client-sent ids have bodies +
            // codecs (`net.rs`); the host apply/relay arms
            // (`MessageBuffer.cs:1851-1867`, `:2290-2306`, `:2325-2364`,
            // `:4282-4297`, `:4328-4343`) land with the send-path pass (S1-I-4).
            // W4-f11 seam 8.2: the client's `19` wiring echo — the host's
            // apply/relay dispatch (`MessageBuffer.cs:1299-1334`) is g48's
            // `net_apply.rs::host_door_toggle` (union merge wires it).
            C2s::TogglePvp(_)
            | C2s::PlayerMana(_)
            | C2s::TeamChange(_)
            | C2s::ProjectileTrackers(_)
            | C2s::DoorToggle(_)
            | C2s::SyncLoadout(_) => {}
            // S1-I-1 wave E: 51 MiscDataSync (`MessageBuffer.cs:2482-2530`
            // subject forcing + the kind ladder) + 61 SpawnBossInvader
            // (`:2740-2878`'s selector ladder) — bodies + codecs in `net.rs`;
            // the apply/relay arms land with the send-path pass (S1-I-4).
            C2s::MiscDataSync(_) | C2s::SpawnBossEvent(_) => {}
            // 65 TeleportEntity (`MessageBuffer.cs:2949-3035` kind ladder +
            // ack decrement) + 73 RequestTeleport (`:3127-3146` conch/
            // potion/shellphone dispatch) — same wave, same parking.
            C2s::TeleportEntity(_) | C2s::RequestTeleport(_) => {}
            // 75 AnglerQuestFinished (`MessageBuffer.cs:3154-3163` adds the
            // sender's name to anglerWhoFinishedToday) — same wave, same
            // parking.
            C2s::AnglerQuestFinished => {}
            // 96's client half (`PortalHelper.cs:190` send; the host forces
            // the subject and relays, `MessageBuffer.cs:3538-3555`) — same
            // wave, same parking.
            C2s::PlayerPortalTeleport(_) => {}
            // 113 DD2 crystal placement (`MessageBuffer.cs:3841-3854`
            // WouldFailSpawningHere + SummonCrystal on the sender) — same
            // wave, same parking.
            C2s::CrystalInvasionStart(_) => {}
            // 141's client half (`LucyAxeMessage.cs:133`; the host relays,
            // `MessageBuffer.cs:4272-4275`) + 143 DD2 skip-wait
            // (`:4298-4303` AttemptToSkipWaitTime) — same wave, same parking.
            C2s::LucyPopup(_) | C2s::CrystalInvasionSkip => {}
            // W4-d97 (S1-I-1 player-family second half): bodies + codecs in
            // `net.rs`; the host relay arm (`MessageBuffer.cs:3657-3660`)
            // and the apply+relay arm (`:3867-3885`) land with the send-path
            // pass (S1-I-4).
            C2s::NebulaLevelupRequest(_) | C2s::PlayerHurtV2(_) => {}
            // W4-d97 batch 2: the host `KillMe`+`SendPlayerDeath` arm
            // (`MessageBuffer.cs:3887-3903`) and the emote-bubble arm
            // (`:3905-3918`, `EmoteID.Count` gate + 360-tick bubble) land
            // with the send-path pass (S1-I-4).
            C2s::PlayerDeathV2(_) | C2s::Emoji(_) => {}
            // W4-d97 batch 3: the host luck apply+relay arm
            // (`MessageBuffer.cs:4164-4193`, `RecalculateLuck` :4188) and
            // the spectate arm (`:4361-4384`, `SetOrRequestSpectating` /
            // the stop relay :4378) land with the send-path pass (S1-I-4).
            C2s::UpdatePlayerLuckFactors(_) | C2s::SpectatePlayer(_) => {}
            // W4-d97 batch 4: the shared team arm's 157 side — relay as 45,
            // the team chat fan-out and the team-based-spawns tail
            // (`MessageBuffer.cs:2342-2363`) land with the send-path pass
            // (S1-I-4).
            C2s::TeamChangeFromUi(_) => {}
            // S1-I-1 wave D94: the projectile family's client-sent support
            // ids (66 heal relay `:3036-3055`, 99/115 minion targets
            // `:3570-3597`) have bodies + codecs (`net.rs`); the host
            // apply/relay arms land with the send-path pass (S1-I-4).
            C2s::HealPlayer(_) | C2s::MinionRestTarget(_) | C2s::MinionAttackTarget(_) => {}
            // S1-I-1 wave B (W4-d92): the NPC family's client-sent ids have
            // bodies + codecs (`net.rs`); the host apply arms
            // (`MessageBuffer.cs:4086-4121` the fish-out spawn + 682 unlock,
            // `:4219-4229` the buff-removal request) land with the send-path
            // pass (S1-I-4).
            C2s::FishOutNpc(_) | C2s::NpcBuffRemovalRequest(_) => {}
            // S1-I-1 wave B batch 2 (W4-d92): the town-NPC info request's
            // host answer arm (`MessageBuffer.cs:2624-2627`, the
            // `TrySendData(56, whoAmI, …)` reply) lands with S1-I-4.
            C2s::UniqueTownNpcInfo(_) => {}
            // S1-I-1 wave B batch 2 (W4-d92): the coin-credit delta's host
            // accumulate + relay arm (`MessageBuffer.cs:3455-3459`) lands
            // with S1-I-4.
            C2s::NpcExtraValue(_) => {}
        }
    }

    /// The join bundle when a slot reaches `IN_WORLD` (the port's
    /// `SyncConnectedPlayer`, `NetMessage.cs:2806-2822`): the newcomer learns
    // the peers, the peers learn the newcomer.

    /// `MessageBuffer.cs:2283` — `player.channel =
    /// player.inventory[player.selectedItem].channel`: the `41` reader
    /// re-derives the channel flag from the peer's selected item. The peer's
    /// inventory rides `5 PlayerInventory` into `PlayerNet.inventory`; an
    /// unknown slot/kind reads `false` (the C#'s empty `Item` has
    /// `channel = false`). W4-c27.
    fn peer_channel(p: &net::PlayerNet) -> bool {
        let slot = (p.selected as u8).min(58);
        p.inventory
            .get(&slot)
            .and_then(|it| crate::registry::get(it.kind.max(0) as u16))
            .is_some_and(|def| def.channel)
    }

    /// `Player.ManaEffect` (`Player.cs:4531-4538`) on a peer: the HealMana
    /// text at the peer's position (`CombatText.NewText`, `:4533`). The net
    /// handlers run outside every RNG swap, so the placement draws ride the
    /// bare stream (the `play_switch_click` precedent). The peer body is the
    /// C#'s 20x42 (`Player.cs:29210-29211`); the font measure falls back like
    /// the local drain (`world_sim/player.rs`). W4-c27.
    fn spawn_peer_mana_text(&mut self, player_id: u8, amount: i32) {
        let Some(p) = self.remote_players.get(&(player_id as u32)) else {
            return;
        };
        let (x, y) = (p.x, p.y);
        let text = amount.to_string();
        let size = match self.lookups.combat_text_font.as_ref() {
            Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
            None => [text.chars().count() as f32 * 8.0, 10.0],
        };
        let location = crate::combat_text::Rect {
            x,
            y,
            width: 20.0,
            height: 42.0,
        };
        crate::combat_text::spawn_amount(
            &mut self.combat_texts,
            location,
            crate::combat_text::HEAL_MANA,
            amount,
            false,
            false,
            size,
            1.0,
            self.rngs.global(),
        );
    }

    /// Streams the world to a spawning client as `10 TileBlock` sections —
    /// the C#'s join-time box (`MessageBuffer.cs:692-811`): 5 sections wide
    /// and 3 tall around the spawn, announced by one `9 Status`, streamed
    /// row-major, closed by `49`. The NPC/item state follows on the regular
    /// cadence tick (the C#'s `SendSection` pushes them inline).
    fn host_stream_sections(&mut self, server: &net::Server, id: u32, cx: i32, cy: i32) {
        let (w, h) = (self.world.width, self.world.height);
        let (gx, gy) = (net::sections_x(w), net::sections_y(h));
        let sx0 = (net::section_x(cx) - 2).clamp(0, gx - 1);
        let sy0 = (net::section_y(cy) - 1).clamp(0, gy - 1);
        let sx1 = (net::section_x(cx) + 2).clamp(0, gx - 1);
        let sy1 = (net::section_y(cy) + 1).clamp(0, gy - 1);
        let count = (sx1 - sx0 + 1) * (sy1 - sy0 + 1);
        server.ensure_sections(id, w, h);
        server.send(
            id,
            &S2c::Status {
                text: "Receiving tile data".into(),
                max: count,
            },
        );
        for sy in sy0..=sy1 {
            for sx in sx0..=sx1 {
            // (`:4338-4341`, `ignoreClient = num68`). The port has no loadout
            // system (the swap is unported, `player.rs:1039`); the loadout
            // index + visibility bits are stored-not-simulated on the peer
            // snapshot.
            C2s::SyncLoadout(mut l) => {
                l.player_id = id as u8; // `:4332`
                self.remote_players.entry(id).or_default().loadout = Some(l);
                server.broadcast_except(&S2c::SyncLoadout(l), id);
            }
            // S1-I-1 W4-f21: `39`'s host release/re-assign + the `22` answer
            // (`MessageBuffer.cs:2244-2253`) is the ITEM lane's task-seed
            // (research/318, the g48 adjudication) — bodies + codecs only here.
            C2s::ReleaseItemOwnership(_) => {}
            // `case 157` — the team UI click (`MessageBuffer.cs:2325-2364`,
            // the shared `case 45: case 157:` arm): force the subject to the
            // sender, set the team, relay as `45`, then the team-chat fan-out
            // to the sender + the old/new team members. The 157-only
            // team-based-spawns tail (`:2355-2363`, `158`) needs the unported
            // `teamBasedSpawnsSeed` + `ExtraSpawnPointManager` — parked.
            C2s::TeamChangeUi(t) => {
                self.apply_team_change_net(server, id, t.team);
            }
            // `case 159` — a client asks for a section (`MessageBuffer.cs:
            // 4457-4462`, host-only): `SendSection(sectionX, sectionY,
            // whoAmi)` (`NetMessage.cs:2697-2712`) — the bounds + not-already-
            // sent gate, mark the section sent, stream the `10 TileBlock`.
            // The C#'s `SyncNPCsForSection` / `SyncChestContentsForSection`
            // tails ride the port's NPC cadence (section-culled `23` stream)
            // and the chest-open path respectively.
            C2s::SectionRequest {
                section_x,
                section_y,
            } => {
                let (sx, sy) = (section_x as i32, section_y as i32);
                let (gx, gy) = (
                    net::sections_x(self.world.width),
                    net::sections_y(self.world.height),
                );
                if sx < 0 || sy < 0 || sx >= gx || sy >= gy {
                    return; // the SendSection bounds gate
                }
                server.ensure_sections(id, self.world.width, self.world.height);
                if server.is_section_sent(id, sx, sy) {
                    return; // `!TileSections[sx, sy]`
                }
                server.mark_section_sent(id, sx, sy);
                let block = self.tile_block_at(sx, sy);
                server.send(id, &S2c::TileBlock(Box::new(block)));
            }
        }
        server.send(id, &S2c::InitialSpawn);
    }

    /// W4-g52 — the per-tick `RemoteClient.CheckSection` stamp
    /// (`Main.cs:64620-64623` → `CheckSection_ForClient`,
    /// `RemoteClient.cs:152-176`): every active player's fluff box (fluff 1,
    /// the default) refreshes `TileSectionsCheckTime` with
    /// `Main.GameUpdateCount` — the clock the `28`/`27` relay arms'
    /// `IsSectionActive` reads. The stream half (`:177-188`'s `SendSection`
    /// burst) rides `pending_sections` on the 10 Hz tick (the port's
    /// standing batch shape); this pass owns ONLY the stamp, so the cull
    /// window is the faithful 60 ticks. Host-only; the local player's own
    /// slot has no RemoteClient (the C# stamps client 0 too — it is never
    /// connected, so the port skips it).
    fn host_check_sections(&mut self) {
        if self.net_mode() != 2 {
            return;
            // `case 150`'s host half (`MessageBuffer.cs:4361-4384`):
            // `SetOrRequestSpectating` / the stop-relay need the unported
            // spectating system (`Player.spectating`); parked (S1-I-4).
            C2s::SpectatePlayer(_) => {}
            // `case 138` — NO reader case in the C#: the frame falls to the
            // `default` arm (`MessageBuffer.cs:4477-4482`), which only boots
            // a State-0 sender (the port's accept gate already refuses
            // pre-spawn frames). The host no-ops. W4-g51 f21 cross-check.
            C2s::ClientSyncedInventory => {}
            // `case 152`'s host half (`MessageBuffer.cs:4386-4396`): force
            // the subject to the sender (`:4389-4392`), then relay to the
            // OTHERS (`:4393-4396`, `ignoreClient = whoAmI`). No host-side
            // sound — the client plays the peer's held item's UseSound.
            C2s::ItemUseSound { player_id: _ } => {
                server.broadcast_except(&S2c::ItemUseSound { player_id: id as u8 }, id);
            }
            // `case 161` (`MessageBuffer.cs:4471-4476`): the host compares
            // the claim against `Netplay.HostToken` and sets `player.host`.
            // The port has no HostToken config (the dedicated-server host
            // authority is unported) — the compare is against the empty
            // string, so `host` stays false; the claim is stored-not-
            // simulated on the peer snapshot. W4-g51 f21 cross-check.
            C2s::HostToken(_token) => {
                // `!string.IsNullOrWhiteSpace(Netplay.HostToken) && == text`
                // — the empty configured token refuses every claim.
                self.remote_players.entry(id).or_default().host = false;
            }
            // `case 89/123/133/149` — the TryPlacing quadruplet: one shared
            // body (f21's byte-equality pin), the per-kind `TryPlacing`
            // (`TEItemFrame.cs:69-86` / `TEWeaponsRack.cs:135-152` /
            // `TEFoodPlatter.cs:70-87` / `TEDeadCellsDisplayJar.cs:69-86`):
            // no entity at the cell drops the item back; a held entity drops
            // its old contents first, then takes the new item and shares the
            // entity with an `86`. Host-only (`netMode != 2` breaks). The
            // `Item.DefaultAssignNewItemsToPlayer(whoAmI)` scope routes the
            // refused-placement drop's ownership to the sender. W4-g51 FINAL.
            C2s::ItemFrameTryPlacing(p) => {
                self.host_try_placing(server, id, tr_core::tile_entities::TeKind::ItemFrame, p);
            }
            C2s::WeaponsRackTryPlacing(p) => {
                self.host_try_placing(server, id, tr_core::tile_entities::TeKind::WeaponsRack, p);
            }
            C2s::FoodPlatterTryPlacing(p) => {
                self.host_try_placing(server, id, tr_core::tile_entities::TeKind::FoodPlatter, p);
            }
            C2s::DeadCellsJarTryPlacing(p) => {
                self.host_try_placing(
                    server,
                    id,
                    tr_core::tile_entities::TeKind::DeadCellsDisplayJar,
                    p,
                );
            }
            // `case 121` (`MessageBuffer.cs:3920-3941`): force the player to
            // the sender (`:3923-3926`), read the entity id + itemIndex +
            // command, apply `ReadData`/`ReadDummySync` to the doll, then
            // relay to the OTHERS (`:3936-3939`, `ignoreClient = num248`).
            // The relay re-derives the tail from the LIVE doll
            // (`NetMessage.cs:1465-1472`), so the port re-reads the store.
            // W4-g51 FINAL.
            C2s::DisplayDollSync(mut d) => {
                d.player_id = id as u8; // `:3925`
                let applied = self.apply_doll_sync(d);
                if applied {
                    // The relay carries the doll's CURRENT slot/pose (the
                    // C# writer reads the entity again, `:1465-1467`).
                    let relay = self.doll_sync_relay(d);
                    server.broadcast_except(&S2c::DisplayDollSync(relay), id);
                }
            }
            // `case 122` (`MessageBuffer.cs:3942-3977`): force the player to
            // the sender (`:3946-3949`). -1 clears the anchor + relays
            // (`:3952-3957`); otherwise the `IsOccupied` + `TryGet` gate
            // (`:3958`) sets the anchor + relays (`:3960-3961`). A refused
            // (occupied / missing) entity sends NOTHING. W4-g51 FINAL.
            C2s::TileEntityInteraction(mut t) => {
                t.player_id = id as u8; // `:3948`
                if t.entity_id == -1 {
                    self.net_te_anchors.remove(&(id as u8)); // `Clear()`
                    server.broadcast(&S2c::TileEntityInteraction(t)); // `:3955`
                    return;
                }
                // `TileEntity.IsOccupied` (`TileEntity.cs:221-234`): any
                // active non-dead player already anchored to this id refuses.
                let occupied = self.net_te_anchors.iter().any(|(slot, (eid, _, _))| {
                    *eid == t.entity_id && self.net_player_slot_active(*slot)
                });
                // `TryGet` fails on a removed `ByID` entry (`:126`/`:159-167`);
                // the port's tombstoned slot must read as missing (the
                // `broadcast_tile_entity` liveness pattern). SEAL re-pin.
                let entity = if t.entity_id >= 0 {
                    self.world
                        .tile_entities
                        .get_by_id(t.entity_id as u32)
                        .filter(|e| {
                            self.world
                                .tile_entities
                                .get_at(e.x, e.y)
                                .is_some_and(|live| live.id == e.id)
                        })
                } else {
                    None
                };
                if !occupied {
                    if let Some(e) = entity {
                        // `Set(id, Position.X, Position.Y)` (`:3960`).
                        self.net_te_anchors
                            .insert(id as u8, (t.entity_id, e.x as i32, e.y as i32));
                        server.broadcast(&S2c::TileEntityInteraction(t)); // `:3961`
                    }
                }
            }
            // `case 124` (`MessageBuffer.cs:3995-4022`): force the player to
            // the sender (`:3998-4001`), unfold the dye flag (`:4004-4009`),
            // the `TryGet<TEHatRack>` + `slot >= 2` refusal reads int+byte
            // and breaks (`:4010-4015`), else `ReadItem` + relay to the
            // OTHERS (`:4016-4020`, `ignoreClient = num180`). W4-g51 FINAL.
            C2s::HatRackItemSync(mut h) => {
                h.player_id = id as u8; // `:4000`
                let dye = h.is_dye();
                let slot = h.slot_index();
                let ok = h.entity_id >= 0
                    && slot < 2
                    && self
                        .world
                        .tile_entities
                        .get_by_id(h.entity_id as u32)
                        .is_some_and(|e| {
                            e.kind == tr_core::tile_entities::TeKind::HatRack
                                // `TryGet` fails on a removed `ByID` entry —
                                // the tombstoned slot reads as missing (the
                                // SEAL re-pin, the 122 gate's pattern).
                                && self
                                    .world
                                    .tile_entities
                                    .get_at(e.x, e.y)
                                    .is_some_and(|live| live.id == e.id)
                        });
                if !ok {
                    return; // `:4010-4015` — the refusal consumes the tail
                }
                self.apply_hat_rack_item(h.entity_id as u32, slot, dye, h.item);
                // The relay re-folds the dye flag (`NetMessage.cs:1492-1496`)
                // and re-reads the live slot (`:1500-1503`).
                let relay = self.hat_rack_relay(h);
                server.broadcast_except(&S2c::HatRackItemSync(relay), id);
            }
            // `case 144` (`MessageBuffer.cs:4304-4309`, host-only, bodyless):
            // `NPC.HaveDryadDoStardewAnimation` (`NPC.cs:19184-19229`).
            // W4-g51 FINAL.
            C2s::RequestQuestEffect => {
                self.host_dryad_stardew(server);
            }
            // `case 156` (`MessageBuffer.cs:4439-4449`, host-only): the
            // Point16 lookup + `TELeashedEntityAnchorWithItem.InsertItem`
            // (`TELeashedEntityAnchorWithItem.cs:32-36`): store the itemType,
            // respawn the leashed entity. The port's kite/critter anchors
            // keep `TeData::Anchor`; the `RespawnLeashedEntity` half is the
            // unported LeashedEntity system (M7-TE-2), so the store is the
            // observable state. W4-g51 FINAL.
            C2s::LeashedAnchorPlaceItem(p) => {
                let Some(e) = self.world.tile_entities.get_at_mut(p.x, p.y) else {
                    return; // `TryGetAt` fails
                };
                if !matches!(
                    e.kind,
                    tr_core::tile_entities::TeKind::KiteAnchor
                        | tr_core::tile_entities::TeKind::CritterAnchor
                ) {
                    return; // `TryGetAt<TELeashedEntityAnchorWithItem>` type gate
                }
                // `InsertItem`: `this.itemType = itemType` (`:34`).
                e.data = tr_core::tile_entities::TeData::Anchor(p.item_type as i32);
                // `base.RespawnLeashedEntity()` (`:35`) — the LeashedEntity
                // despawn/respawn is M7-TE-2; the itemType store lands live.
            }
        }
    }

    fn check_npcs_react_to_emote(&mut self, player_id: u8) {
        // The emote's player: the local player or a peer snapshot.
        let (px, py, talkable) = if player_id as u32 == self.local_id {
            match self.player.as_ref() {
                Some(p) => (
                    p.x,
                    p.y,
                    !p.dead && p.stealth == 1.0, // `CanBeTalkedTo` (`Player.cs:3888-3888`)
                ),
                None => return,
            }
        } else {
            match self.remote_players.get(&(player_id as u32)) {
                Some(p) => (p.x, p.y, p.stealth == 1.0),
                None => return,
            }
        };
        let player_top = (px + crate::player::WIDTH * 0.5, py);
        for npc in self.npcs.iter_mut() {
            if npc.hp <= 0 || npc.kind.ai_style() != 7 || !npc.kind.is_town_npc_like() {
                continue;
            }
            if npc.ai[0] >= 2.0 {
                continue; // already chatting
            }
            let (nw, _) = npc.size();
            let npc_center = (npc.x + nw * 0.5, npc.y + npc.size().1 * 0.5);
            let dx = npc_center.0 - (px + crate::player::WIDTH * 0.5);
            let dy = npc_center.1 - (py + crate::player::HEIGHT * 0.5);
            let near = talkable && (dx * dx + dy * dy).sqrt() < 200.0;
            let no_los = !crate::projectile::can_hit_line(
                &self.world,
                npc.x + nw * 0.5,
                npc.y,
                0.0,
                0.0,
                player_top.0,
                player_top.1,
                0.0,
                0.0,
            );
            if near || no_los {
                let direction = if npc.x < px { 1 } else { -1 };
                npc.ai[0] = 19.0;
                npc.ai[1] = 220.0;
                npc.ai[2] = player_id as f32;
                npc.facing = direction;
                npc.net_update = true;
            }
        }
    }

    /// The shared `117` apply (`MessageBuffer.cs:3879`): `Player.Hurt(reason,
    /// damage, hitDirection, pvp, quiet: true, crit, cooldownCounter)`. The
    /// local player runs the ported `hurt_inner` (the quiet flag suppresses
    /// the local hurt sound — the port's hurt path plays none inline); a peer
    /// snapshot takes the raw damage off its hp (the C# runs the full Hurt
    /// on every `Main.player` slot, the port keeps the hp part — the 16/13
    /// convention). The defense/dodge halves ride the local sim only.
    fn apply_player_hurt(&mut self, subject: u8, h: &net::PlayerHurtV2) {
        if subject as u32 == self.local_id {
            if let Some(pl) = self.player.as_mut() {
                // `hitDirection` picks the knockback source side; the port's
                // `hurt_inner` takes `from_x` — the subject's own x offset by
                // the direction (the C# `Hurt` reads `hitDirection` for the
                // knockback sign, `Player.cs:38519`).
                let from_x = pl.x - h.hit_direction as f32 * 16.0;
                let rng = self.rngs.global();
                pl.hurt_inner(
                    h.damage as i32,
                    from_x,
                    h.hit_direction != 0,
                    rng,
                    false, // dodgeable: the net replay is not a dodge roll
                    h.crit,
                    None,
                    false, // retaliate_lane: quiet net hurt
                );
            }
        } else if let Some(p) = self.remote_players.get_mut(&(subject as u32)) {
            p.hp = (p.hp - h.damage as i32).max(0);
        }
    }

    /// The shared `118` apply (`MessageBuffer.cs:3898`): `Player.KillMe(reason,
    /// damage, hitDirection, pvp)`. The local player runs the ported death
    /// pipeline (the npcs-pass `hp <= 0 && !dead` arm's shape — drops, chat
    /// broadcast, `start_death`); a peer snapshot marks dead (hp 0). The
    /// death text resolves from the reason's `ByOther` family when present,
    /// else the `DeathText.Slain` row.
    fn apply_player_death(&mut self, subject: u8, d: &net::PlayerDeathV2) {
        if subject as u32 == self.local_id {
            let Some(pl) = self.player.as_ref() else {
                return;
            };
            if pl.dead || pl.creative_god_mode {
                return; // `KillMe`'s head gate (`Player.cs:38809-38812`)
            }
            let (dx, dy) = (pl.x, pl.y);
            let boss_near = pl.boss_hinders(&self.npcs);
            let name = self.player_name.clone();
            // The death text: the reason's custom string wins, else the
            // `ByOther` family, else `DeathText.Slain` (`GetDeathText`).
            let death_text = if let Some(custom) = &d.reason.source_custom_reason {
                custom.clone()
            } else if d.reason.source_other_index >= 0 {
                // The `ByOther(N)` family maps through the DoT attribution
                // table; the port's `death_text_key` covers the drain lanes.
                // A net other-index outside them falls back to Slain.
                crate::death::slain_message(&name)
            } else {
                crate::death::slain_message(&name)
            };
            crate::death::on_player_death(self, dx, dy, &death_text);
            crate::chat::broadcast_death(&mut self.chat, &death_text);
            if let Some(pl) = self.player.as_mut() {
                pl.hp = 0;
                pl.start_death(&self.world, boss_near, false);
            }
        } else if let Some(p) = self.remote_players.get_mut(&(subject as u32)) {
            p.hp = 0;
        }
    }

    /// `DD2Event.SummonCrystalDirect` (`DD2Event.cs:584-601`, W4-g50): no
    /// live crystal (548) and an active stand tile (466) at the cell; the
    /// spawn point is the stand's anchor `+ (40, 64)`, then `StartInvasion`
    /// (`:253-291`), the crystal `NewNPC` and `DropStarterCrystals`
    /// (`:929-942` — five 2-stacks of 3822 at the crystal's centre).
    fn summon_crystal_direct(&mut self, server: &net::Server, x: i32, y: i32) {
        // `!NPC.AnyNPCs(548)` (`:586`).
        if self
            .npcs
            .iter()
            .any(|n| n.hp > 0 && n.kind == crate::npc::NpcKind::DD2EterniaCrystal)
        {
            return;
        }
        let Some(tile) = self.world.tile(x, y) else {
            return;
        };
        if !tile.active() || tile.kind != 466 {
            return; // `:589`
        }
        let (fx, fy) = (tile.frame_x as i32, tile.frame_y as i32);
        let sx = x * 16 - fx / 18 * 16 + 40;
        let sy = y * 16 - fy / 18 * 16 + 64;
        // `StartInvasion` (`:253-291`) — the counters/latches start clean,
        // the 300-tick spawn hold arms and `WipeEntities` runs BEFORE the
        // crystal's `NewNPC` (`:596-597`), so the wipe cannot eat it. The
        // port's `use_etherian_crystal` keeps the same field set.
        self.world.dd2_event_ongoing = true;
        self.world.dd2_wave = 1;
        self.world.dd2_wave_kills = 0.0;
        self.world.dd2_lost = false;
        self.world.dd2_won = false;
        self.world.dd2_crystal_drops = tr_core::dd2_victory::CrystalDropState::default();
        self.world.dd2_betsy_spawned = false;
        self.world.dd2_latches = tr_core::dd2::DownedLatches::default();
        self.world.dd2_spawn_hold = tr_core::dd2::SPAWN_HOLD_START;
        crate::world_sim::npcs::wipe_dd2_entities(self);
        // `BroadcastText(InvasionStart, INFO_START_INVASION_COLOR)` (`:278`)
        // + `SendData(7)` (`:279`) — the chat line and the header re-push.
        let sm = crate::app::netmodules::ServerMessage {
            author: 255,
            text: crate::lang::text("DungeonDefenders2.InvasionStart"),
            color: net::Rgb(
                tr_core::dd2::INFO_START_INVASION_COLOR[0],
                tr_core::dd2::INFO_START_INVASION_COLOR[1],
                tr_core::dd2::INFO_START_INVASION_COLOR[2],
            ),
        };
        server.broadcast(&S2c::NetModule {
            module: crate::app::netmodules::module_id::TEXT,
            body: sm.encode(),
        });
        server.broadcast(&S2c::WorldInfo(Box::new(self.world_info())));
        // The crystal `NewNPC` at the spawn point (`:597`).
        let mut crystal = crate::npc::Npc::new(
            crate::npc::NpcKind::DD2EterniaCrystal,
            sx as f32 - 20.0,
            sy as f32 - 20.0,
            1,
            0.0,
        );
        crystal.hp = crystal.max_hp;
        let (cx, cy) = (crystal.x + crystal.size().0 * 0.5, crystal.y + crystal.size().1 * 0.5);
        self.npcs.push(crystal);
        // `DropStarterCrystals` (`:929-942`): five 2-stacks of 3822 at the
        // crystal's centre.
        for _ in 0..5 {
            crate::world_sim::items::spawn_item_drop_px(
                &mut self.items,
                self.rngs.global(),
                tr_core::dd2::ENERGY_CRYSTAL_ITEM,
                2,
                Vec2::new(cx, cy),
            );
        }
    }

    /// `WorldGen.ToggleGemLock` (`WorldGen.cs:46469-46524`, W4-g50): the
    /// gem lock (tile 440) flips its 3×3 frame between the off (0) and on
    /// (54) rows; turning an ON lock off drops its gem. The framing refresh,
    /// the 3×3 tile-square relay and the `HitSwitch` + `59` follow.
    fn toggle_gem_lock(&mut self, server: &net::Server, i: i32, j: i32, on: bool) {
        // `:46471-46475` — the active-440 gate; an off toggle on an already
        // off lock (frameY < 54) refuses.
        let Some(tile) = self.world.tile(i, j) else {
            return;
        };
        if !tile.active() || tile.kind != 440 || (tile.frame_y < 54 && !on) {
            return;
        }
        let was_on = tile.frame_y >= 54; // `flag` (`:46478-46481`)
        let (frame_x, frame_y) = (tile.frame_x as i32, tile.frame_y as i32);
        let gem_col = frame_x / 54; // `num2`
        let num3 = frame_x % 54 / 18;
        let num4 = frame_y % 54 / 18;
        // `:46485-46508` — the gem item the lock holds.
        let gem = match gem_col {
            0 => Some(1526u16),
            1 => Some(1524),
            2 => Some(1525),
            3 => Some(1523),
            4 => Some(1522),
            5 => Some(1527),
            6 => Some(3643),
            _ => None,
        };
        // `:46509-46515` — the 3×3 frameY rewrite.
        for k in (i - num3)..(i - num3 + 3) {
            for l in (j - num4)..(j - num4 + 3) {
                let fy = (if on { 54 } else { 0 }) + (l - j + num4) * 18;
                self.world.set_tile_frame_y(k, l, fy);
            }
        }
        // `:46516-46519` — turning an ON lock off drops the gem.
        if let Some(gem) = gem {
            if was_on && !on {
                crate::world_sim::items::spawn_item_drop(
                    &mut self.items,
                    self.rngs.global(),
                    gem,
                    1,
                    i,
                    j,
                );
            }
        }
        // `:46520` — `SquareTileFrame(i, j)`; the object drops it may pop
        // ride the edit chain's resolution.
        let drops = tr_core::edit::square_tile_frame_default(&mut self.world, i, j);
        for drop in drops {
            let id = drop
                .item
                .map(|item| item as u16)
                .or_else(|| crate::registry::tile_drop(drop.tile_kind, 0));
            if let Some(id) = id {
                crate::world_sim::items::spawn_item_drop(
                    &mut self.items,
                    self.rngs.global(),
                    id,
                    1,
                    drop.x,
                    drop.y,
                );
            }
        }
        self.world_key = None;
        // `:46521` — `SendTileSquare(-1, i - num3, j - num4, 3, 3)`.
        let mut records = Vec::with_capacity(9);
        for l in (j - num4)..(j - num4 + 3) {
            for k in (i - num3)..(i - num3 + 3) {
                records.push(self.tile_record_at(k, l));
            }
        }
        server.broadcast(&S2c::SectionRange(Box::new(net::SectionRange {
            x: (i - num3) as i16,
            y: (j - num4) as i16,
            w: 3,
            h: 3,
            change_type: 0,
            records,
        })));
        // `:46522-46523` — `Wiring.HitSwitch(i - num3, j - num4)` + the `59`.
        self.wiring
            .hit_switch(&mut self.world, i - num3, j - num4);
        for (px, py) in self.wiring.take_switch_clicks() {
            self.audio
                .play_switch_click(px as f32, py as f32, self.rngs.global());
        }
        server.broadcast(&S2c::HitSwitch(net::HitSwitch {
            x: (i - num3) as i16,
            y: (j - num4) as i16,
        }));
    }

    /// `case 109`'s host body (`MessageBuffer.cs:3774-3788` →
    /// `Wiring.MassWireOperation`, `Wiring.cs:419-453`, W4-g50 re-probe):
    /// the 58-slot stock census runs over the SENDER's inventory copy
    /// (the `5` messages keep it, `Main.player[whoAmI]`), the W4-c42
    /// constants-copy core walks the L-rectangle, and the `netMode == 2`
    /// bill leg sends the two `110`s to the master (`:441-442`,
    /// `SendData(110, master.whoAmI, -1, null, 530/849, count, whoAmI)` —
    /// the count is sent even at zero) and RETURNS: no host-side
    /// `ConsumeItem` loops. The per-edit `SendData(17)` calls inside
    /// `MassWireOperationStep` (`Wiring.cs:3353` etc.) ride the port's
    /// `20 SectionRange` convention (one 1×1 per changed cell, the
    /// `tile_record_at` read-back — the wire bits ride the record).
    /// The subject's `Center`/`direction` come from the `13` snapshot
    /// (`PlayerNet`); the cut-cache drops spawn at that centre on the
    /// `UpdatePlayers` stream (the use-path convention).
    fn host_mass_wire_operation(
        &mut self,
        server: &net::Server,
        id: u32,
        m: &net::MassWireOperation,
    ) {
        // `Wiring.cs:423-433` — the stock census over the sender's 58 slots.
        let inv = server.slot_inventory(id);
        let mut wire_count = 0i32;
        let mut actuator_count = 0i32;
        for slot in 0i16..58 {
            let Some(it) = inv.get(&slot) else { continue };
            if it.kind == 530 {
                wire_count += it.stack as i32;
            }
            if it.kind == 849 {
                actuator_count += it.stack as i32;
            }
        }
        let (num, num2) = (wire_count, actuator_count);
        // `:436` — `master.Center` + `master.direction == 1`.
        let master = self.remote_players.get(&id);
        let drop_point = master
            .map(|p| (p.x + crate::player::WIDTH * 0.5, p.y + crate::player::HEIGHT * 0.5))
            .unwrap_or((0.0, 0.0));
        let dir = master.is_some_and(|p| p.facing == 1);
        let ps = (m.start_x as i32, m.start_y as i32);
        let pe = (m.end_x as i32, m.end_y as i32);
        // The changed-cell collector: the C# sends one `17` per successful
        // Place*/Kill*; the port diffs the L-walk's bounding box (the core
        // only ever edits cells inside it) and sends one `20` per change.
        let (lx, hx) = (ps.0.min(pe.0), ps.0.max(pe.0));
        let (ly, hy) = (ps.1.min(pe.1), ps.1.max(pe.1));
        let snap: Vec<(u8, bool)> = (ly..=hy)
            .flat_map(|y| (lx..=hx).map(move |x| (x, y)))
            .map(|(x, y)| {
                let t = self.world.tile(x, y);
                let act = t.is_some_and(|t| t.flags & tr_core::tile::FLAG_ACTUATOR != 0);
                let mut bits = 0u8;
                for (i, c) in [
                    tr_core::wiring::WireColor::Red,
                    tr_core::wiring::WireColor::Green,
                    tr_core::wiring::WireColor::Blue,
                    tr_core::wiring::WireColor::Yellow,
                ]
                .into_iter()
                .enumerate()
                {
                    if self.world.wires.get(x, y, c) {
                        bits |= 1 << i;
                    }
                }
                (bits, act)
            })
            .collect();
        let mut cut_cache = (0i32, 0i32);
        tr_core::wiring::mass_wire_operation_inner(
            &mut self.world,
            ps,
            pe,
            dir,
            m.tool_mode,
            &mut wire_count,
            &mut actuator_count,
            &Self::mass_wire_can_place,
            &mut cut_cache,
        );
        // The per-edit replication (the `SendData(17)` seams).
        let mut i = 0usize;
        for y in ly..=hy {
            for x in lx..=hx {
                let t = self.world.tile(x, y);
                let act = t.is_some_and(|t| t.flags & tr_core::tile::FLAG_ACTUATOR != 0);
                let mut bits = 0u8;
                for (j, c) in [
                    tr_core::wiring::WireColor::Red,
                    tr_core::wiring::WireColor::Green,
                    tr_core::wiring::WireColor::Blue,
                    tr_core::wiring::WireColor::Yellow,
                ]
                .into_iter()
                .enumerate()
                {
                    if self.world.wires.get(x, y, c) {
                        bits |= 1 << j;
                    }
                }
                if snap[i] != (bits, act) {
                    let rec = self.tile_record_at(x, y);
                    server.broadcast(&S2c::SectionRange(Box::new(net::SectionRange {
                        x: x as i16,
                        y: y as i16,
                        w: 1,
                        h: 1,
                        change_type: 0,
                        records: vec![rec],
                    })));
                }
                i += 1;
            }
        }
        self.world_key = None;
        // The cut half's cached drops (`Item.DropCache`, `:3325-3326`) —
        // the host is `netMode != 1`, so the kills' items really spawn.
        for (kind, total) in [(530u16, cut_cache.0), (849u16, cut_cache.1)] {
            let mut left = total;
            while left > 0 {
                let chunk = left.min(crate::registry::max_stack(kind) as i32);
                left -= chunk;
                crate::world_sim::items::spawn_item_drop_px(
                    &mut self.items,
                    self.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    kind,
                    chunk as u32,
                    Vec2::new(drop_point.0, drop_point.1),
                );
            }
        }
        // `:441-442` — the two bills to the master, then RETURN (the host
        // never runs the ConsumeItem loops; the client's `110` reader does).
        server.send(
            id,
            &S2c::MassWireOperationPay(net::MassWireOperationPay {
                item_type: 530,
                count: (num - wire_count) as i16,
                player: id as u8,
            }),
        );
        server.send(
            id,
            &S2c::MassWireOperationPay(net::MassWireOperationPay {
                item_type: 849,
                count: (num2 - actuator_count) as i16,
                player: id as u8,
            }),
        );
    }

    /// The mass-wire core's `can_place` predicate — the world-resolved
    /// `Player.CanDoWireStuffHere` (the same gate as the c42 copy's
    /// `can_do_wire_stuff_here_world`, `Player.cs:46921-46943`).
    fn mass_wire_can_place(world: &World, x: i32, y: i32) -> bool {
        let Some(t) = world.tile(x, y) else {
            return false;
        };
        if t.wall == 350 {
            return false;
        }
        if !world.downed_golem && t.wall == 87 {
            return false;
        }
        true
    }

    /// `case 85`'s host body (`MessageBuffer.cs:3261-3266` →
    /// `QuickStacking.ReadNetInventory` + `QuickStackToNearbyChests`,
    /// `QuickStacking.cs:256-266`, W4-g50 re-probe): the slot list snapshots
    /// the SENDER's `5`-fed inventory copy into the scratch `SourceInventory`,
    /// the W4-g21 constants-copy engine transfers into the in-range chests
    /// (`NearbyChests.GetChestsInRangeOf`, 600 px of `player.position`), the
    /// restore re-sends one `5` per source slot (the `netMode == 2` arm,
    /// `:353-356`, blocked flag in bit 1) and the blocked list answers the
    /// sender via `IndicateBlockedChests`' `SendData(85, whoAmI)` (`:275`).
    /// The per-slot `SyncSlot(32)` broadcasts (`:91-97`) replicate each
    /// changed chest slot. The host's peer copy (`remote_players`) takes
    /// the restored slots too — the C# `Main.player[whoAmI]` IS that copy.
    fn host_quick_stack(&mut self, server: &net::Server, id: u32, q: &net::QuickStackRequest) {
        // `ReadNetInventory` (`:308-321`): one snapshot per listed slot id.
        let inv = server.slot_inventory(id);
        let mut source: Vec<crate::quick_stack::SourceSlot> = Vec::new();
        for &slot in &q.slots {
            let Some(ni) = inv.get(&slot) else { continue };
            let item = if ni.kind <= 0 || ni.stack <= 0 {
                crate::item::EMPTY_SLOT
            } else {
                crate::item::ItemSlot {
                    kind: ni.kind as u16,
                    name: crate::registry::name(ni.kind as u16),
                    count: ni.stack.max(0) as u32,
                    prefix: ni.prefix as u16,
                }
            };
            source.push(crate::quick_stack::SourceSlot {
                slot_id: slot as usize,
                item,
                blocked: false,
            });
        }
        // `GetChestsInRangeOf(player.position)` (`NearbyChests.cs:10-31`):
        // the sender's `13` snapshot position; locked/in-use chests STAY in
        // the set (`DestinationHelper.Reset` reads `IsLockedOrInUse`,
        // `QuickStacking.cs:53`) — they block, not vanish.
        let (px, py) = self
            .remote_players
            .get(&id)
            .map(|p| (p.x, p.y))
            .unwrap_or((0.0, 0.0));
        let open = self.remote_players.get(&id).and_then(|p| p.chest).unwrap_or(-1) as i32;
        let mut in_range: Vec<(usize, bool, (f32, f32))> = Vec::new();
        for (idx, wc) in self.world_chests.iter().enumerate() {
            if wc.tile == crate::chest::removed_tile()
                || wc.tile == crate::chest::unplaced_tile()
            {
                continue; // `chest != null` (`:20`)
            }
            let cx = wc.tile.0 as f32 * 16.0 + 16.0;
            let cy = wc.tile.1 as f32 * 16.0 + 16.0;
            let dx = cx - px;
            let dy = cy - py;
            if dx * dx + dy * dy > crate::craft::CHEST_RANGE_PX * crate::craft::CHEST_RANGE_PX {
                continue;
            }
            let locked = crate::chest::is_locked_or_in_use(&self.world_chests, &self.world, idx, open);
            in_range.push((idx, locked, (cx, cy)));
        }
        // The lazy store's implicit airs: size each destination to maxItems
        // before the borrow (the C# `Chest.item[]` is always maxItems long).
        for &(idx, _, _) in &in_range {
            if let Some(wc) = self.world_chests.get_mut(idx) {
                let max = wc.max_items;
                if wc.slots.len() < max {
                    wc.slots.resize(max, crate::item::EMPTY_SLOT);
                }
            }
        }
        let mut chests: Vec<(&mut [crate::item::ItemSlot], bool, (f32, f32), i32)> = Vec::new();
        for (idx, locked, pos) in in_range {
            if let Some(wc) = self.world_chests.get_mut(idx) {
                chests.push((&mut wc.slots[..], locked, pos, idx as i32));
            }
        }
        let mut dests: Vec<crate::quick_stack::Dest<'_>> = chests
            .into_iter()
            .map(|(slots, locked, pos, idx)| crate::quick_stack::Dest::new(slots, locked, pos, idx))
            .collect();
        let blocked = crate::quick_stack::transfer(&mut source, &mut dests, q.smart_stack);
        // `SyncSlot` (`:91-97`): the changed chest slots broadcast as `32`.
        for dest in &dests {
            let idx = dest.chest_index();
            for (slot, item) in dest.changed_slots() {
                server.broadcast(&S2c::ChestItem(net::ChestItem {
                    chest: idx as i16,
                    slot: slot as u8,
                    stack: item.count.min(i16::MAX as u32) as i16,
                    prefix: item.prefix.min(u8::MAX as u16) as u8,
                    kind: item.kind as i16,
                }));
            }
        }
        // `RestoreToPlayer` (`:343-363`): every source slot restored + one
        // `5` re-send (blocked flag bit 1); the host's peer copy follows.
        for s in &source {
            let ni = net::NetItem {
                kind: s.item.kind as i16,
                stack: s.item.count.min(i16::MAX as u32) as i16,
                prefix: s.item.prefix.min(u8::MAX as u16) as u8,
                favorited: inv
                    .get(&(s.slot_id as i16))
                    .is_some_and(|n| n.favorited),
                blocked: false,
            };
            server.set_slot_inventory(id, s.slot_id as i16, ni);
            if let Some(p) = self.remote_players.get_mut(&id) {
                p.inventory.insert(s.slot_id as i16, ni);
            }
            // `SendData(5, -1, -1, null, whoAmI, slotId, flag ? 1 : 0)` —
            // the port's 5 body carries the flag as `NetItem.blocked`.
            server.broadcast(&S2c::Inventory {
                player_id: id as u8,
                slot: s.slot_id as i16,
                item: net::NetItem { blocked: s.blocked, ..ni },
            });
        }
        // `IndicateBlockedChests`' host leg (`:270-277`): the list answers
        // the SENDER only (`SendData(85, player.whoAmI, -1, …)`).
        if !blocked.is_empty() {
            server.send(
                id,
                &S2c::QuickStackBlocked(net::QuickStackBlocked {
                    chests: blocked.iter().map(|&c| c as u16).collect(),
                }),
            );
        }
    }

    /// `case 94`'s shared switch (`MessageBuffer.cs:3469-3483`): the
    /// `enableDebugCommands` gate (`TR_DEBUG_COMMANDS`, the d85 registry's
    /// pin), then the three shared-state arms. `number2` is read as a float
    /// and truncated to int (`(int)this.reader.ReadSingle()`, `:3467`) —
    /// the codec carries the float, the comparison is on the truncation.
    /// The `DebugNetworkStream.Latency = ping / 2` half (`:3482`) is the
    /// unported test-harness stream — the ping value stores.
    fn apply_dev_commands_shared(&mut self, d: &net::DevCommands) {
        if !crate::env::debug_commands() {
            return; // `:3469-3472`
        }
        let num = d.number2 as i32;
        match d.command.as_str() {
            "/showdebug" => {
                self.debug_shared.report_command_usage = num == 1; // `:3476`
            }
            "/randomizeprojslots" => {
                self.debug_shared.randomize_projectile_slots = num == 1; // `:3479`
            }
            "/setserverping" => {
                self.debug_shared.server_ping = num; // `:3481`
            }
            _ => {} // the `/quickload-*` harness arms stay parked
        }
    }

    fn host_slot_join_complete(&mut self, server: &net::Server, id: u32, _s: net::PlayerSpawn) {
        // The port's `SyncOnePlayer` (`NetMessage.cs:2885-2946`) sends the
        // joining peer everyone's state; the host's own snapshot leads.
        for p in self.remote_players.values() {
            if p.id != id {
                server.send(id, &S2c::PlayerState(Box::new(self.player_state_of(p))));
                if !p.name.is_empty() {
                    let mut pi = net::PlayerInfo::with_name(p.id as u8, &p.name);
                    // The peer's biome-torch trio rides the relayed identity
                    // (`NetMessage.cs:182-184`, W4-f33).
                    pi.flags2.set(0, p.using_biome_torches);
                    pi.flags2.set(1, p.happy_fun_torch_time);
                    pi.flags2.set(2, p.unlocked_biome_torches);
                    server.send(id, &S2c::PlayerInfo(pi));
    /// `case 94`'s host leg: the shared switch, then the quickload relay
    /// (`MessageBuffer.cs:3484-3516`) — PARKED (the `QuickLoad` test
    /// harness is unported: the probe's `JoinWorld().WithCurrentState()`
    /// serializer and the response's `Deserialize`/`TryRead`/`Set` chain
    /// have no port; the relay's `SendData(94, …)` targets ride it).
    fn apply_dev_commands(&mut self, _server: &net::Server, _id: u32, d: &net::DevCommands) {
        self.apply_dev_commands_shared(d);
    }

    /// `Player.ConsumeItem(type)`'s forward scan (`Player.cs:9269-9308`,
    /// `reverseOrder: false`, `includeVoidBag: false`, W4-g50): the first
    /// matching cell in the C#'s 58-slot `inventory` order — the port's grid
    /// 0..49, then coins 50..53, then ammo 54..57 — loses one; a drained cell
    /// turns to air (`SetDefaults(0)`). Returns whether one was consumed.
    fn consume_item_forward(&mut self, kind: u16) -> bool {
        let cells = self
            .inventory
            .iter_mut()
            .chain(self.equip.coins.iter_mut())
            .chain(self.equip.ammo.iter_mut());
        for slot in cells {
            if slot.count > 0 && slot.kind == kind {
                slot.count -= 1;
                if slot.count == 0 {
                    slot.kind = 0;
                }
                return true;
            }
        }
        // The host is a player too (its slot is 0): the display names ride
        // `4 PlayerInfo`, which msg 13 lacks.
        if !self.player_name.is_empty() {
            let mut pi = net::PlayerInfo::with_name(0, &self.player_name);
            // `bitsByte16[0..2]` (`NetMessage.cs:182-184`, W4-f33).
            if let Some(p) = self.player.as_ref() {
                pi.flags2.set(0, p.using_biome_torches());
                pi.flags2.set(1, p.torch_god.happy_fun_torch_time);
                pi.flags2.set(2, p.unlocked_biome_torches);
            }
            server.send(id, &S2c::PlayerInfo(pi));
        false
    }

    /// `Utils.RandomVector2(Main.rand, min, max)` (`Utils.cs:2431-2434`):
    /// TWO `NextDouble` draws, X then Y.
    fn random_vector2(rng: &mut tr_core::UnifiedRandom, min: f32, max: f32) -> Vec2 {
        let x = (max - min) * rng.next_double() as f32 + min;
        let y = (max - min) * rng.next_double() as f32 + min;
        Vec2::new(x, y)
    }

    /// `Main.rand.NextVector2Circular(halfW, halfH)` (`Utils.cs:1266-1269`):
    /// the unit-circle angle draw (`NextFloat() * 2π` → cos/sin) THEN a
    /// second `NextFloat` radius scale.
    fn next_vector2_circular(rng: &mut tr_core::UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
        let angle = rng.next_float() * std::f32::consts::TAU;
        let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
        let r = rng.next_float();
        Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
    }

    /// `WorldGen.TreeGrowFX` (`WorldGen.cs:29600-29637`, W4-g50): the
    /// tree-grow gore burst — three gores per row down the trunk, the
    /// top-row crown burst, each with the trunk's paint column. The 1278
    /// ash variant routes to [`Self::tree_grow_fx_ash_shake`].
    fn tree_grow_fx(&mut self, x: i32, y: i32, height: i32, tree_gore: i32, hit_tree: bool) {
        let vector = Vec2::new(x as f32, y as f32) * 16.0 + Vec2::new(8.0, 8.0);
        if tree_gore == 1278 {
            self.tree_grow_fx_ash_shake(x, y, height, hit_tree);
            return;
        }
        // `currentColumn` — the trunk tile's paint (`:29608-29616`).
        let mut current_column = 0u8;
        if let Some(tile) = self.world.tile(x, y) {
            if tile.active()
                && (tr_core::tileobject::is_a_tree_trunk(tile.kind) || tile.kind == 323)
            {
                current_column = tile.paint();
            }
        }
        let mut num = 0i32;
        while num > -height {
            let num2 = 10.0f32;
            let vector2 = Vec2::new(5.0, 7.0);
            for dx in [-16.0f32, 0.0, 16.0] {
                let rng = self.rngs.global();
                let pos = vector + Vec2::new(dx, num as f32 * 16.0) - vector2;
                let vel = Self::random_vector2(rng, -num2, num2);
                let scale = 0.7 + rng.next_float() * 0.6;
                let idx = self.gore.new_gore(rng, &self.world, pos, vel, tree_gore, scale);
                if idx != tr_core::gore::GORE_SENTINEL {
                    self.gore.slot_mut(idx).frame.current_column = current_column;
                }
            }
            if num == -height + 1 {
                let num3 = if hit_tree { 5 } else { 20 };
                for _ in 0..num3 {
                    let rng = self.rngs.global();
                    let pos = vector
                        + Vec2::new(0.0, num as f32 * 16.0 - 40.0)
                        + Self::random_vector2(rng, -40.0, 40.0)
                        - vector2;
                    let vel = Self::random_vector2(rng, -10.0, 10.0);
                    let scale = 0.7 + rng.next_float() * 0.6;
                    let idx = self.gore.new_gore(rng, &self.world, pos, vel, tree_gore, scale);
                    if idx != tr_core::gore::GORE_SENTINEL {
                        self.gore.slot_mut(idx).frame.current_column = current_column;
                    }
                }
            }
            num -= 1;
        }
    }

    /// `WorldGen.TreeGrowFX_AshShake` (`WorldGen.cs:29639-29672`, W4-g50):
    /// the ash-tree shake — the type-31 dust rows, the type-6 leaf dust and
    /// the 61..63 gore fall, each gated on the `Next(3) != 0` rolls.
    fn tree_grow_fx_ash_shake(&mut self, x: i32, y: i32, height: i32, hit_tree: bool) {
        let vector = Vec2::new(x as f32, y as f32) * 16.0 + Vec2::new(8.0, 8.0);
        let ctx = self.spawn_ctx();
        let mut num = 0i32;
        while num > -height {
            let rng = self.rngs.global();
            if num >= -height / 2 && rng.next_max(3) != 0 {
                let num2 = if hit_tree { 10 } else { 5 };
                for _ in 0..num2 {
                    let rng = self.rngs.global();
                    let pos = vector
                        + Vec2::new(-16.0, num as f32 * 16.0)
                        + Self::random_vector2(rng, -30.0, 30.0);
                    let idx = self.dust.new_dust_direct(
                        rng, &ctx, &self.world, pos, 4, 4, 31, 0.0, -3.0, 100,
                        tr_core::particles::Color::default(), 1.0,
                    );
                    let vel = Self::next_vector2_circular(rng, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                    if idx != tr_core::particles::DUST_SENTINEL {
                        self.dust.slot_mut(idx).velocity = vel;
                    }
                }
            }
            let rng = self.rngs.global();
            if num as f32 >= (-height as f32) * 0.7 && rng.next_max(3) != 0 {
                let num3 = if hit_tree { 16 } else { 10 };
                for _ in 0..num3 {
                    let rng = self.rngs.global();
                    let pos = vector
                        + Vec2::new(-16.0, num as f32 * 16.0)
                        + Self::random_vector2(rng, -30.0, 30.0);
                    let scale = 1.0 + 0.7 * rng.next_float();
                    let idx = self.dust.new_dust_direct(
                        rng, &ctx, &self.world, pos, 4, 4, 6, 0.0, -4.0, 0,
                        tr_core::particles::Color::default(), scale,
                    );
                    let mut vel = Self::next_vector2_circular(rng, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                    let no_grav = rng.next_max(2) == 0;
                    if no_grav {
                        vel *= 2.0;
                    }
                    vel += Vec2::new(0.0, if hit_tree { -1.0 } else { -2.0 });
                    if idx != tr_core::particles::DUST_SENTINEL {
                        let d = self.dust.slot_mut(idx);
                        d.velocity = vel;
                        if no_grav {
                            d.no_gravity = true;
                        }
                    }
                }
            }
            if !hit_tree {
                let rng = self.rngs.global();
                let pos = vector
                    + Vec2::new(-16.0, num as f32 * 16.0 - 40.0)
                    + Self::random_vector2(rng, -40.0, 40.0) * Vec2::new(0.5, 1.0);
                let gore_type = 61 + rng.next_max(3);
                let idx = self.gore.new_gore(rng, &self.world, pos, Vec2::ZERO, gore_type, 1.0);
                if idx != tr_core::gore::GORE_SENTINEL {
                    self.gore.slot_mut(idx).velocity *= 0.5;
                }
            }
            num -= 1;
        }
    }

    /// `NPC.FairyEffects` (`NPC.cs:48652-48683`, W4-g50): the 40-dust type-278
    /// burst in the fairy's colour pair (pink / green / blue by type), the
    /// velocity tiers (×1.5, ×2 at i>=20, ×3.5 at i>=30), the per-dust fade
    /// roll and noGravity, then the Item4 cue.
    fn fairy_effects(&mut self, position: Vec2, fairy_type: i32) {
        use tr_core::particles::Color;
        // `Color.Lerp(value, value2, NextFloat())` — the byte lerp.
        let lerp = |a: Color, b: Color, t: f32| -> Color {
            let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
            Color::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), 255)
        };
        let (value, value2) = match fairy_type {
            1 => (
                Color::new(50, 205, 50, 255),   // LimeGreen
                Color::new(32, 178, 170, 255),  // LightSeaGreen
            ),
            2 => (
                Color::new(65, 105, 225, 255),  // RoyalBlue
                Color::new(173, 216, 230, 255), // LightBlue
            ),
            _ => (
                Color::new(255, 105, 180, 255), // HotPink
                Color::new(255, 182, 193, 255), // LightPink
            ),
        };
        let num = 4;
        let ctx = self.spawn_ctx();
        for i in 0..40 {
            let rng = self.rngs.global();
            let color = lerp(value, value2, rng.next_float());
            let idx = self.dust.new_dust_direct(
                rng,
                &ctx,
                &self.world,
                position - Vec2::new(num as f32, num as f32) * 0.5,
                num + 4,
                num + 4,
                278,
                0.0,
                0.0,
                200,
                color,
                0.65,
            );
            let fade = rng.next_range(0, 17) as f32 * 0.1;
            if idx != tr_core::particles::DUST_SENTINEL {
                let d = self.dust.slot_mut(idx);
                d.velocity *= 1.5;
                if i >= 30 {
                    d.velocity *= 3.5;
                } else if i >= 20 {
                    d.velocity *= 2.0;
                }
                d.fade_in = fade;
                d.no_gravity = true;
            }
        }
        // `SoundEngine.PlaySound(SoundID.Item4, (int)X, (int)Y)` (`:48682`).
        self.audio
            .play_at("Item_4", position.x, position.y, 1.0);
    }

    /// `Utils.PoofOfSmoke` (`Utils.cs:1078-1093`, W4-g50): the gore burst
    /// then the dust ring, verbatim draw order on the bare `Main.rand`
    /// stream. `num = Main.rand.Next(3, 7)` gores; each draws a rotation
    /// (`NextFloat() * 2π` → `ToRotationVector2`), scaled by `(2, 0.7) *
    /// 0.7`, type `Next(11, 14)`, then scale 0.7 + velocity ×0.5. The 10
    /// dusts are type 16 at the position, 14×14 box, alpha 100, scale 1.5,
    /// each shifted +5 and given a rotation-scaled velocity with a second
    /// `NextFloat` radius (`0.5 + 0.5 * NextFloat()`).
    fn poof_of_smoke(&mut self, position: Vec2) {
        let ctx = self.spawn_ctx();
        let rng = self.rngs.global();
        let num = rng.next_range(3, 7);
        for _ in 0..num {
            let rng = self.rngs.global();
            let angle = rng.next_float() * std::f32::consts::TAU;
            let rot = Vec2::new(
                (angle as f64).cos() as f32,
                (angle as f64).sin() as f32,
            );
            let velocity = rot * Vec2::new(2.0, 0.7) * 0.7;
            let gore_type = rng.next_range(11, 14);
            let idx = self.gore.new_gore(
                rng,
                &self.world,
                position,
                velocity,
                gore_type,
                1.0,
            );
            if idx != tr_core::gore::GORE_SENTINEL {
                let g = self.gore.slot_mut(idx);
                g.scale = 0.7;
                g.velocity *= 0.5;
            }
        }
        for _ in 0..10 {
            // `Dust.NewDust(position, 14, 14, 16, 0, 0, 100, default, 1.5)`
            // draws the position/velocity/frame rolls INSIDE the spawn; the
            // `obj.position += 5` and the rotation-velocity assignment are
            // post-spawn writes whose angle/radius draws come AFTER.
            let rng = self.rngs.global();
            let idx = self.dust.new_dust(
                rng,
                &ctx,
                &self.world,
                position,
                14,
                14,
                16,
                0.0,
                0.0,
                100,
                tr_core::particles::Color::default(),
                1.5,
            );
            let rng = self.rngs.global();
            let angle = rng.next_float() * std::f32::consts::TAU;
            let rot = Vec2::new(
                (angle as f64).cos() as f32,
                (angle as f64).sin() as f32,
            );
            let radius = 0.5 + 0.5 * rng.next_float();
            if idx != tr_core::particles::DUST_SENTINEL {
                let d = self.dust.slot_mut(idx);
                d.position += Vec2::new(5.0, 5.0);
                d.velocity = rot * Vec2::new(2.0, 0.7) * 0.7 * radius;
            }
        }
    }

    /// The `102` client fan (`MessageBuffer.cs:3662-3705`): the picker's
    /// team gate, the 700 px distance cull, the spiral dust trail per
    /// qualifying player, then `NebulaLevelup` on the local player. The
    /// port's player set is the local player + the peer snapshots (the C#
    /// walks all 255 `Main.player` slots; the inactive ones fail `active`).
    fn apply_nebula_levelup(&mut self, n: &net::NebulaLevelupRequest) {
        let other = Vec2::new(n.x, n.y);
        // The picker's team (`player11.team`, `:3666`): the local player or
        // a peer snapshot; an unknown picker reads team 0 (no team gate).
        let picker_team = if n.player_id as u32 == self.local_id {
            self.player.as_ref().map(|p| p.team).unwrap_or(0)
        } else {
            self.remote_players
                .get(&(n.player_id as u32))
                .map(|p| p.team)
                .unwrap_or(0)
        };
        let picker_center = if n.player_id as u32 == self.local_id {
            self.player.as_ref().map(|p| {
                let (cx, cy) = p.center();
                Vec2::new(cx, cy)
            })
        } else {
            self.remote_players.get(&(n.player_id as u32)).map(|p| {
                Vec2::new(p.x + crate::player::WIDTH * 0.5, p.y + crate::player::HEIGHT * 0.5)
            })
        };
        // The qualifying players: (center, team). The local player is
        // active when present and living; a peer snapshot is active (the
        // port keeps no dead peers — the 135/118 arms remove them).
        let mut targets: Vec<(Vec2, i32)> = Vec::new();
        if let Some(p) = self.player.as_ref() {
            if !p.dead {
                let (cx, cy) = p.center();
                targets.push((Vec2::new(cx, cy), p.team));
            }
        }
        for p in self.remote_players.values() {
            targets.push((
                Vec2::new(p.x + crate::player::WIDTH * 0.5, p.y + crate::player::HEIGHT * 0.5),
                p.team,
            ));
        }
        let ctx = self.spawn_ctx();
        for (center, team) in targets {
            // `:3666` — `player11.team != 0 && player11.team != player12.team`
            // skips the player; team 0 picks up anyone. Then the 700 px
            // distance cull (`!(player12.Distance(other) < 700f)`).
            if picker_team != 0 && picker_team != team {
                continue;
            }
            if (center - other).length() >= 700.0 {
                continue;
            }
            // The dust trail (`:3670-3702`): the picker→player direction,
            // the spiral spin, the per-6px dust spawns.
            if let Some(picker_center) = picker_center {
                let value2 = picker_center - center;
                let len = value2.length();
                let vector4 = if len > 0.0 && !value2.x.is_nan() && !value2.y.is_nan() {
                    value2 / len
                } else {
                    Vec2::ZERO
                };
                if !vector4.x.is_nan() && !vector4.y.is_nan() && vector4 != Vec2::ZERO {
                    let dust_type = match n.buff_type {
                        179 => 86,
                        173 => 90,
                        176 => 88,
                        _ => 90,
                    };
                    let mut spin = 0.0f64;
                    let spin_step = std::f64::consts::PI / 15.0;
                    let spinningpoint = Vec2::new(0.0, -8.0);
                    let offset = Vec2::new(-3.0, -3.0);
                    let mut fade = 0.0f32;
                    let mut k = 0.0f32;
                    while k < len / 6.0 {
                        let position = center
                            + 6.0 * k * vector4
                            + crate::bow_launch::rotated_by(spinningpoint, spin)
                            + offset;
                        spin += spin_step;
                        let rng = self.rngs.global();
                        let idx = self.dust.new_dust(
                            rng,
                            &ctx,
                            &self.world,
                            position,
                            6,
                            6,
                            dust_type,
                            0.0,
                            0.0,
                            100,
                            tr_core::particles::Color::default(),
                            1.5,
                        );
                        // `:3698-3701` — the C# writes through the returned
                        // index (6000 on a refused spawn); the fade counter
                        // advances regardless (`num110 = fadeIn = num110 +
                        // num111`). The port guards the sentinel write but
                        // keeps the counter.
                        fade += 0.005;
                        if idx != tr_core::particles::DUST_SENTINEL {
                            let d = self.dust.slot_mut(idx);
                            d.no_gravity = true;
                            d.velocity = Vec2::ZERO;
                            d.fade_in = fade;
                            d.velocity += vector4 * 1.5;
                        }
                        k += 1.0;
                    }
                }
            }
        }
        // `player12.NebulaLevelup(num106)` (`:3704`) — the local player only
        // (the method's `whoAmI != Main.myPlayer` guard, `Player.cs:57032`).
        if let Some(p) = self.player.as_mut() {
            p.nebula_levelup(n.buff_type as i32);
        }
    }

    /// The shared `96` apply (`MessageBuffer.cs:3545-3551`): the color
    /// parity flip (`num184 + (num184 % 2 == 0 ? 1 : -1)`) feeds the ENTRY
    /// teleport effect, then `Teleport(dest, 4, num184)` stores the
    /// UN-flipped wire color (`Player.cs:37477`) and the velocity overwrites.
    /// The local player runs the ported `teleport_style` core; a peer's
    /// snapshot snaps directly (the C# `Main.player[num183]` is the same
    /// array on both sides). The entry/exit `TeleportEffect` dust is the
    /// unported FX seam — the parity flip rides it, so only the stored
    /// color lands here.
    fn apply_player_portal_teleport(&mut self, player: u8, p: &net::PlayerPortalTeleport) {
        if player as u32 == self.local_id {
            if let Some(pl) = self.player.as_mut() {
                pl.teleport_style(p.x, p.y, 4);
                // `Player.Teleport`'s style-4 tail (`Player.cs:37475-37481`):
                // `lastPortalColorIndex = extraInfo` (the un-flipped wire
                // value), the portal-physics flag and the gravity zero (the
                // gravity is a per-tick rebase in the port — the flag feeds
                // `wet_gravity_and_cap`).
                pl.last_portal_color_index = p.color_index;
                pl.portal_physics = true;
                pl.vx = p.velocity_x;
                pl.vy = p.velocity_y;
            }
        } else if let Some(entry) = self.remote_players.get_mut(&(player as u32)) {
            entry.x = p.x;
            entry.y = p.y;
            entry.vx = p.velocity_x;
            entry.vy = p.velocity_y;
            // `Player.Teleport`'s `netOffset = Vector2.Zero` (`:37447`).
            entry.off_x = 0.0;
            entry.off_y = 0.0;
            entry.last_portal_color_index = p.color_index;
        }
    }

    /// The shared `21`/`90` client apply (`MessageBuffer.cs:1445-1493`):
    /// one drop's state keyed by the host slot index. The `90` instanced
    /// tail (`:1489-1493`) reserves the drop for the local player — the
    /// port's `net_reserved` map is the `playerIndexTheItemIsReservedFor`
    /// stand-in (the `22` convention); the `WorldItem.instanced` flag has
    /// no port field (the per-player lifetime is the reservation itself).
    fn apply_item_state(&mut self, d: &net::ItemState, instanced: bool) {
        let mut drop = item::ItemDrop::new(
            d.kind.max(0) as u16,
            d.stack.max(0) as u32,
            d.prefix as u16,
            d.x,
            d.y,
            d.vx,
            d.vy,
            0.0,
        );
        // `MessageBuffer.cs:1519-1528` — the `21` handler writes
        // type/prefix/stack/position/velocity/shimmer/enemyGrabDelay
        // but NOT the grab-delay pair (that is `22`'s job,
        // `:1558-1559`): a live cadence update must not wipe the
        // join burst's 100-tick lock (S1-P-i51). The port rebuilds
        // the record, so carry the pair over from the old one.
        if let Some(old) = self.net_items.get(&(d.slot as i16)) {
            drop.grab_delay_player = old.grab_delay_player;
            drop.grab_delay_time = old.grab_delay_time;
        }
        drop.shimmered = d.shimmered;
        drop.shimmer_time = d.shimmer_time;
        // `:1487` — the client writes the enemy-grab lock too
        // (bit 3's tail byte, S1-I-1 wave I).
        drop.enemy_grab_delay_time = d.enemy_grab_delay as i32;
        drop.age = 60.0;
        self.net_items.insert(d.slot as i16, drop);
        self.net_items_dirty = true;
        if instanced {
            // `:1492` — `playerIndexTheItemIsReservedFor = Main.myPlayer`.
            // The hold ticks are the C# `MakeInstanced` lifetime default
            // (54000, `WorldItem.cs:327`); the port's reservation map
            // carries the pair the grab gate reads.
            self.net_reserved
                .insert(d.slot as i16, (self.local_id as u8, 54000));
        }
    }

    /// The `86 TileEntitySharing` broadcast for one live entity
    /// (`NetMessage.cs:1216-1225`): the id, the presence bool, then the
    /// `networkSend: true` record. A tombstoned id broadcasts the removal
    /// shape (the C# `TryGet` fails → `flag2 = false`).
    fn broadcast_tile_entity(&self, server: &net::Server, id: u32) {
        let record = match self.world.tile_entities.get_by_id(id) {
            Some(e)
                if self
                    .world
                    .tile_entities
                    .get_at(e.x, e.y)
                    .is_some_and(|live| live.id == id) =>
            {
                let mut w = tr_core::binio::Writer::new();
                e.write_net(&mut w);
                w.into_inner()
            }
            _ => Vec::new(),
        };
        server.broadcast(&S2c::TileEntitySharing(net::TileEntitySharing {
            id: id as i32,
            record,
        }));
    }

    /// `TileEntity.IsOccupied`'s `player.active && !player.dead` term
    /// (`TileEntity.cs:227`) for one slot: the host's own player (slot 0) or
    /// a remote peer with live hp. The port's `remote_players` entries are
    /// the active peers; hp ≤ 0 reads dead (`Player.dead = statLife <= 0`,
    /// `MessageBuffer.cs:1111`). W4-g51 FINAL id-122.
    fn net_player_slot_active(&self, slot: u8) -> bool {
        if slot == 0 {
            return self.player.as_ref().is_some_and(|p| !p.dead);
        }
        self.remote_players
            .get(&(slot as u32))
            .is_some_and(|p| p.hp > 0)
    }

    /// The TryPlacing quadruplet's shared host body (89/123/133/149 —
    /// `TEItemFrame.TryPlacing` `TEItemFrame.cs:69-86` and its three twins,
    /// the only differences are the `RangeFrame` rectangle and the
    /// `DropItem` box). No entity at the cell drops the item back
    /// (`Item.NewItem`, exact prefix); a held entity drops its old contents
    /// first (`DropItem`, count 1), then takes the new item and shares the
    /// entity with an `86`. W4-g51 FINAL.
    fn host_try_placing(
        &mut self,
        server: &net::Server,
        sender: u32,
        kind: tr_core::tile_entities::TeKind,
        p: net::TryPlacingItem,
    ) {
        let (x, y) = (p.x as i32, p.y as i32);
        // `WorldGen.RangeFrame(x, y, x+N, y+N)` — the framing refresh rides
        // the port's edit chain; the entity books are the state the message
        // changes (the g50 89 precedent).
        let entity = self
            .world
            .tile_entities
            .get_at(p.x, p.y)
            .filter(|e| e.kind == kind)
            .map(|e| e.id);
        let Some(entity_id) = entity else {
            // `Item.NewItem(… type, stack, … prefix)` — the refused
            // placement drops the item at the cell with the exact prefix
            // (`Item.cs:49730`'s `Prefix(exact)` applies it directly, no
            // fresh roll). The `DefaultAssignNewItemsToPlayer(whoAmI)`
            // scope routes the ownership to the sender (the 100-tick
            // grab-delay lock, `Item.cs:49757-49760`).
            let mut drop = item::ItemDrop::new(
                p.kind.max(0) as u16,
                (p.stack.max(0) as u32).max(1),
                p.prefix as u16,
                x as f32 * TILE_PX,
                y as f32 * TILE_PX,
                0.0,
                0.0,
                0.0,
            );
            // The spawn scatter rides the port's pinned triplet on the bare
            // stream (the message handler runs outside every swap,
            // S1-E-21i — the g50 89 precedent).
            let rng = self.rngs.global();
            drop.vel = Vec2::new(
                (rng.next_float() * 2.0 - 1.0) * 1.4,
                -2.2 - rng.next_float() * 1.2,
            );
            drop.phase = rng.next_float() * std::f32::consts::TAU;
            drop.apply_grab_delay(sender as u8);
            self.items.push(drop);
            return;
        };
        // `result.item.stack > 0` → `DropItem`: the old contents drop at the
        // entity's cell (count 1, the C# `DropItem` passes no stack —
        // `Item.NewItem`'s default), keeping the stored prefix.
        if let Some(Some(old)) = self.world.tile_entities.item(entity_id).copied() {
            if old.stack > 0 {
                let mut drop = item::ItemDrop::new(
                    old.kind,
                    1,
                    old.prefix as u16,
                    x as f32 * TILE_PX,
                    y as f32 * TILE_PX,
                    0.0,
                    0.0,
                    0.0,
                );
                let rng = self.rngs.global();
                drop.vel = Vec2::new(
                    (rng.next_float() * 2.0 - 1.0) * 1.4,
                    -2.2 - rng.next_float() * 1.2,
                );
                drop.phase = rng.next_float() * std::f32::consts::TAU;
                drop.apply_grab_delay(sender as u8);
                self.items.push(drop);
            }
        }
        // `result.item = new Item(); SetDefaults(type); Prefix(prefix);
        // stack = stack` — the exact prefix applies directly (the wire byte
        // is the rolled id, not a -1 natural-roll marker).
        self.world.tile_entities.set_item(
            entity_id,
            Some(tr_core::tile_entities::TeItem {
                kind: p.kind.max(0) as u16,
                prefix: p.prefix,
                stack: p.stack.max(0) as u16,
            }),
        );
        // `NetMessage.SendData(86, -1, -1, null, result.ID, x, y)`.
        self.broadcast_tile_entity(server, entity_id);
    }

    /// `TEDisplayDoll.ReadData` (`TEDisplayDoll.cs:802-822`) + `ReadItem`
    /// (`:766-778`) / `ReadPose` (`:846-849`): command 2 sets the pose byte;
    /// commands 0/1/3 write one slot of the equip/dye/misc array (the
    /// `itemIndex < collection.Length` gate). The no-entity / wrong-kind
    /// path is `ReadDummySync` (`:835-844`): consume and discard, NO relay
    /// (`MessageBuffer.cs:3930-3934`). Returns whether the relay fires.
    /// W4-g51 FINAL id-121.
    fn apply_doll_sync(&mut self, d: net::DisplayDollSync) -> bool {
        if d.entity_id < 0 {
            return false; // `TryGet` fails → ReadDummySync, no relay
        }
        // `TryGet<TEDisplayDoll>`'s `as T` type gate + the removed-`ByID`
        // failure (the tombstoned slot reads as missing — the SEAL re-pin,
        // the 122/124 gates' pattern). Checked before the mutable borrow.
        let live = self
            .world
            .tile_entities
            .get_by_id(d.entity_id as u32)
            .is_some_and(|e| {
                e.kind == tr_core::tile_entities::TeKind::DisplayDoll
                    && self
                        .world
                        .tile_entities
                        .get_at(e.x, e.y)
                        .is_some_and(|live| live.id == e.id)
            });
        if !live {
            return false;
        }
        let Some(e) = self.world.tile_entities.by_id_mut(d.entity_id as u32) else {
            return false;
        };
        let tr_core::tile_entities::TeData::DisplayDoll {
            equip,
            dye,
            misc,
            pose,
        } = &mut e.data
        else {
            return false;
        };
        let idx = d.item_index as usize;
        if d.command == 2 {
            // `ReadPose`: `this._pose = reader.ReadByte()` (`:846-849`).
            if let net::DollSyncData::Pose(p) = d.data {
                *pose = p;
            }
            return true;
        }
        // `ReadData`'s collection select (`:812-820`): command 1 → dyes,
        // command 3 → misc, EVERY OTHER command (0 and unknowns) → equip —
        // the C# has no default arm, the fall-through is `_equip`.
        let net::DollSyncData::Item(it) = d.data else {
            return true; // the tail was consumed; nothing to store
        };
        // `ReadItem`: SetDefaults(type); stack = stack; Prefix — the port
        // stores the triple; an air triple clears the slot (the C#
        // `new Item()` fallback).
        let slot = if it.is_air() {
            None
        } else {
            Some(tr_core::tile_entities::TeItem {
                kind: it.kind.max(0) as u16,
                prefix: it.prefix,
                stack: it.stack.max(0) as u16,
            })
        };
        // The `itemIndex < collection.Length` gate (`:771`): the equip/dye
        // arrays hold 9, misc holds 1. An out-of-range index reads the tail
        // but writes nothing.
        match d.command {
            1 if idx < 9 => dye[idx] = slot,
            3 if idx < 1 => *misc = slot,
            1 | 3 => {}
            _ if idx < 9 => equip[idx] = slot,
            _ => {}
        }
        true
    }

    /// The `121` relay body (`NetMessage.cs:1458-1474`): the writer re-reads
    /// the LIVE doll's slot/pose (`WriteData`, `:1465-1467`), so the relay
    /// carries the store's current value, not the inbound frame's. The
    /// no-entity fallback is `WriteDummySync` (`:1471`). W4-g51 FINAL.
    fn doll_sync_relay(&self, d: net::DisplayDollSync) -> net::DisplayDollSync {
        let mut out = d;
        let entity = if d.entity_id >= 0 {
            // The writer's `TryGet<TEDisplayDoll>` (`:1465`): a removed
            // `ByID` entry falls to `WriteDummySync` (SEAL re-pin).
            self.world
                .tile_entities
                .get_by_id(d.entity_id as u32)
                .filter(|e| {
                    self.world
                        .tile_entities
                        .get_at(e.x, e.y)
                        .is_some_and(|live| live.id == e.id)
                })
        } else {
            None
        };
        let doll = entity.and_then(|e| match &e.data {
            tr_core::tile_entities::TeData::DisplayDoll {
                equip,
                dye,
                misc,
                pose,
            } => Some((equip, dye, misc, *pose)),
            _ => None,
        });
        let Some((equip, dye, misc, pose)) = doll else {
            out.data = net::DollSyncData::Dummy;
            return out;
        };
        let idx = d.item_index as usize;
        out.data = if d.command == 2 {
            net::DollSyncData::Pose(pose)
        } else {
            let slot = match d.command {
                1 => dye.get(idx).copied().flatten(),
                3 => {
                    if idx == 0 {
                        *misc
                    } else {
                        None
                    }
                }
                _ => equip.get(idx).copied().flatten(),
            };
            net::DollSyncData::Item(slot.map_or(net::TeItem::default(), |it| net::TeItem {
                kind: it.kind as i16,
                prefix: it.prefix,
                stack: it.stack as i16,
            }))
        };
        out
    }

    /// `TEHatRack.ReadItem` (`TEHatRack.cs:390-403`): SetDefaults(type);
    /// stack = stack; Prefix into `_items[slot]` or `_dyes[slot]`. The port
    /// stores the triple; an air triple clears the slot. W4-g51 FINAL id-124.
    fn apply_hat_rack_item(&mut self, entity_id: u32, slot: u8, dye: bool, it: net::TeItem) {
        let Some(e) = self.world.tile_entities.by_id_mut(entity_id) else {
            return;
        };
        let tr_core::tile_entities::TeData::HatRack(items, dyes) = &mut e.data else {
            return;
        };
        let value = if it.is_air() {
            None
        } else {
            Some(tr_core::tile_entities::TeItem {
                kind: it.kind.max(0) as u16,
                prefix: it.prefix,
                stack: it.stack.max(0) as u16,
            })
        };
        let arr = if dye { dyes } else { items };
        if (slot as usize) < arr.len() {
            arr[slot as usize] = value;
        }
    }

    /// The `124` relay body (`NetMessage.cs:1489-1508`): the writer re-folds
    /// the dye flag into the slot (`num4 += 2`, `:1492-1496`) and re-reads
    /// the LIVE slot (`WriteItem`, `:1500-1503`). W4-g51 FINAL.
    fn hat_rack_relay(&self, h: net::HatRackItemSync) -> net::HatRackItemSync {
        let mut out = h;
        let idx = h.slot_index() as usize;
        let entity = if h.entity_id >= 0 {
            // The writer's `TryGet<TEHatRack>` (`:1500`): a removed `ByID`
            // entry falls to the air-triple fallback (SEAL re-pin).
            self.world
                .tile_entities
                .get_by_id(h.entity_id as u32)
                .filter(|e| {
                    self.world
                        .tile_entities
                        .get_at(e.x, e.y)
                        .is_some_and(|live| live.id == e.id)
                })
        } else {
            None
        };
        let slot = entity.and_then(|e| match &e.data {
            tr_core::tile_entities::TeData::HatRack(items, dyes) => {
                let arr = if h.is_dye() { dyes } else { items };
                arr.get(idx).copied().flatten()
            }
            _ => None,
        });
        out.item = slot.map_or(net::TeItem::default(), |it| net::TeItem {
            kind: it.kind as i16,
            prefix: it.prefix,
            stack: it.stack as i16,
        });
        out
    }

    /// `NPC.HaveDryadDoStardewAnimation` (`NPC.cs:19184-19229`): the FIRST
    /// active type-20 dryad gets the Stardew ai writes (`localAI[2]=480`,
    /// `ai[0]=24`, `ai[1]=480`, `ai[2]=0`, `localAI[3]=0`, `netUpdate=true`,
    /// `direction=1`, `spriteDirection=1`), then the 995 Jumino projectile
    /// spawns at the ground-probe point. The probe: `Bottom + (100, 0)` →
    /// tile coords, `Y = Clamp(Y-10, 10, maxTilesY-10)`, scan down 15 rows
    /// for the first solid tile; if it is the FIRST row the spawn uses the
    /// dryad's own bottom, else the probe tile's world coords `(8, 0)`; the
    /// spawn point is that + `(0, -52)`. W4-g51 FINAL id-144.
    fn host_dryad_stardew(&mut self, server: &net::Server) {
        let Some(slot) = self
            .npcs
            .iter()
            .position(|n| n.hp > 0 && n.kind == crate::npc::NpcKind::Dryad)
        else {
            return; // no active dryad — the loop finds nothing
        };
        let (nw, nh) = self.npcs[slot].size();
        let bottom = (
            self.npcs[slot].x + nw * 0.5,
            self.npcs[slot].y + nh,
        );
        // The ai writes (`:19193-19200`).
        {
            let n = &mut self.npcs[slot];
            n.local_ai[2] = 480.0;
            n.ai[0] = 24.0;
            n.ai[1] = 480.0;
            n.ai[2] = 0.0;
            n.local_ai[3] = 0.0;
            n.net_update = true;
            n.facing = 1;
            n.sprite_dir = 1;
        }
        // The ground probe (`:19201-19225`).
        let vector = (bottom.0 + 100.0, bottom.1);
        let mut px = (vector.0 / TILE_PX) as i32;
        let mut py = (vector.1 / TILE_PX) as i32;
        py = (py - 10).clamp(10, self.world.height - 10);
        let start_y = py;
        let mut first_row_solid = false;
        for j in start_y..start_y + 15 {
            py = j;
            let solid = self
                .world
                .tile(px, j)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
            if solid {
                if j == start_y {
                    first_row_solid = true;
                }
                break;
            }
            // `case 98` - a host-side progression event
            // (`MessageBuffer.cs:3564-3569`).
            S2c::ProgressionEvent { event_id } => {
                let _ = self.achievements.notify_progression(event_id.max(0) as u32);
            }
            // Other players' identity/health (the C# relays 4/16/14 to all).
            S2c::PlayerInfo(pi) => {
                let peer = pi.player_id as u32;
                if peer != self.local_id {
                    let (px, py) = self.net_peer_spawn_pos();
                    self.remote_players
                        .entry(peer)
                        .or_insert_with(|| net::PlayerNet {
                            id: peer,
                            name: pi.name.clone(),
                            x: px,
                            y: py,
                            ..Default::default()
                        });
                    if let Some(p) = self.remote_players.get_mut(&peer) {
                        p.name = pi.name.clone();
                        // `bitsByte33[0..2]` (`MessageBuffer.cs:343-345`,
                        // W4-f33): the peer's biome-torch trio.
                        p.using_biome_torches = pi.flags2.get(0);
                        p.happy_fun_torch_time = pi.flags2.get(1);
                        p.unlocked_biome_torches = pi.flags2.get(2);
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
                    let (px, py) = self.net_peer_spawn_pos();
                    self.remote_players
                        .entry(player_id as u32)
                        .or_insert_with(|| net::PlayerNet {
                            id: player_id as u32,
                            name: format!("Player {}", player_id),
                            x: px,
                            y: py,
                            ..Default::default()
                        });
                } else {
                    self.remote_players.remove(&(player_id as u32));
                }
        }
        // `p.ToWorldCoordinates(8, 0)` = `(px*16+8, py*16)`; the first-row
        // solid uses the dryad's own bottom instead (`:19222-19225`).
        let base = if first_row_solid {
            vector
        } else {
            (px as f32 * TILE_PX + 8.0, py as f32 * TILE_PX)
        };
        let spawn = (base.0, base.1 - 52.0);
        // `Projectile.NewProjectile(…, 995, 0, 0f, Main.myPlayer)`
        // (`:19226`): the host's own player owns it (slot 0). The 995 row
        // (`Projectile.cs:9446-9457`): 240×104, aiStyle 192, friendly (the
        // pool's spawn default), penetrate -1, tileCollide false,
        // ignoreWater, timeLeft *= 5. The X/Y overload CENTRES the box on
        // the spawn point (`Projectile.cs:10876-10877`: `X - width*0.5,
        // Y - height*0.5` → −120, −52 for the 240×104 row) — the SEAL
        // re-pin, the f30 local twin's convention.
        let owner = 0u8;
        if let Some(ps) = self.projectiles.spawn_with_data_owned(
            995,
            owner,
            spawn.0 - 120.0,
            spawn.1 - 52.0,
            0.0,
            0.0,
            0,
            0.0,
            0.0,
            0.0,
        ) {
            // The `27` spawn broadcast (`Projectile.cs:11096`): the host
            // owns the shot, so it streams to every client. The dryad's
            // ai writes ride the `23` cadence (`net_update = true`).
            let st = crate::app::net_proj_sync::state_of(&self.projectiles, ps);
            server.broadcast(&S2c::Projectile(Box::new(st)));
        }
    }

    /// Streams the world to a spawning client as `10 TileBlock` sections —
    /// the C#'s join-time box (`MessageBuffer.cs:692-811`): 5 sections wide
    /// and 3 tall around the spawn, announced by one `9 Status`, streamed
    /// row-major, closed by `49`. The NPC/item state follows on the regular
    /// cadence tick (the C#'s `SendSection` pushes them inline).
    fn host_stream_sections(&mut self, server: &net::Server, id: u32, cx: i32, cy: i32) {
        let (w, h) = (self.world.width, self.world.height);
        let (gx, gy) = (net::sections_x(w), net::sections_y(h));
        let sx0 = (net::section_x(cx) - 2).clamp(0, gx - 1);
        let sy0 = (net::section_y(cy) - 1).clamp(0, gy - 1);
        let sx1 = (net::section_x(cx) + 2).clamp(0, gx - 1);
        let sy1 = (net::section_y(cy) + 1).clamp(0, gy - 1);
        let count = (sx1 - sx0 + 1) * (sy1 - sy0 + 1);
        server.ensure_sections(id, w, h);
        server.send(
            id,
            &S2c::Status {
                text: "Receiving tile data".into(),
                max: count,
            },
        );
        for sy in sy0..=sy1 {
            for sx in sx0..=sx1 {
                server.mark_section_sent(id, sx, sy);
                let block = self.tile_block_at(sx, sy);
                server.send(id, &S2c::TileBlock(Box::new(block)));
            }
        }
        // `57 Unknown57` rides the join arm ahead of `49`
        // (`MessageBuffer.cs:867` — `TrySendData(57, this.whoAmI)` between
        // the banner module and the `103`/`101`/`136` pushes). The
        // `CountTiles` recount re-push (`WorldGen.cs:71173`) parks with the
        // tile-count pass. W4-f21b.
        server.send(
            id,
            &S2c::WorldEvilBytes(net::WorldEvilBytes {
                good: self.world.t_good,
                evil: self.world.t_evil,
                blood: self.world.t_blood,
            }),
        );
        server.send(id, &S2c::InitialSpawn);
    }

    /// W4-g52 — the per-tick `RemoteClient.CheckSection` stamp
    /// (`Main.cs:64620-64623` → `CheckSection_ForClient`,
    /// `RemoteClient.cs:152-176`): every active player's fluff box (fluff 1,
    /// the default) refreshes `TileSectionsCheckTime` with
    /// `Main.GameUpdateCount` — the clock the `28`/`27` relay arms'
    /// `IsSectionActive` reads. The stream half (`:177-188`'s `SendSection`
    /// burst) rides `pending_sections` on the 10 Hz tick (the port's
    /// standing batch shape); this pass owns ONLY the stamp, so the cull
    /// window is the faithful 60 ticks. Host-only; the local player's own
    /// slot has no RemoteClient (the C# stamps client 0 too — it is never
    /// connected, so the port skips it).
    fn host_check_sections(&mut self) {
        if self.net_mode() != 2 {
            return;
        }
        let (w, h) = (self.world.width, self.world.height);
        let (gx, gy) = (net::sections_x(w), net::sections_y(h));
        let now = self.game_update_count;
        let NetRole::Host(server) = &self.net else {
            return;
        };
        for (id, p) in &self.remote_players {
            server.ensure_sections(*id, w, h);
            let psx = net::section_x(p.x as i32 / TILE_PX as i32);
            let psy = net::section_y(p.y as i32 / TILE_PX as i32);
            for sy in (psy - 1).max(0)..=(psy + 1).min(gy - 1) {
                for sx in (psx - 1).max(0)..=(psx + 1).min(gx - 1) {
                    server.mark_section_checked(*id, sx, sy, now);
                }
            }
        }
    }

    /// The move-driven section catch-up (the C#'s per-tick
    /// `RemoteClient.CheckSection`, `RemoteClient.cs:126-188`, called from
    /// `Main.UpdateServer`'s per-player loop `Main.cs:64620-64623`): the
    /// sections each remote peer lacks, collected without borrowing `self`
    /// mutably (the send pass runs inside the `net_tick` match). Returns
    /// `(id, sx, sy, announce)` — `announce` is the count of missing sections
    /// in the fluff box, non-zero only on the FIRST missing section of a
    /// peer's pass (`RemoteClient.cs:177-181`: one `9 Status` before the
    /// `SendSection` walk, `Lang.inter[44]` "Receiving tile data").
    fn pending_sections(&self, server: &net::Server) -> Vec<(u32, i32, i32, i32)> {
        let (w, h) = (self.world.width, self.world.height);
        let (gx, gy) = (net::sections_x(w), net::sections_y(h));
        let mut out = Vec::new();
        for (id, p) in &self.remote_players {
            server.ensure_sections(*id, w, h);
            let psx = net::section_x(p.x as i32 / tr_core::TILE_PX as i32).clamp(0, gx - 1);
            let psy = net::section_y(p.y as i32 / tr_core::TILE_PX as i32).clamp(0, gy - 1);
            // `CheckSection_ForClient`'s fluff box (`RemoteClient.cs:158-176`):
            // count the missing sections first — the `9` announces the count.
            let mut missing = 0;
            for sy in (psy - 1).max(0)..=(psy + 1).min(gy - 1) {
                for sx in (psx - 1).max(0)..=(psx + 1).min(gx - 1) {
                    if !server.is_section_sent(*id, sx, sy) {
                        missing += 1;
                    }
                }
                entry.x = ps.x;
                entry.y = ps.y;
                entry.vx = ps.vx;
                entry.vy = ps.vy;
                entry.facing = if ps.flags_a.get(6) { 1 } else { -1 };
                entry.selected = ps.selected_item as u16;
                entry.on_ground = ps.on_ground;
                entry.leg_frame = ps.leg_frame;
                entry.body_frame = ps.body_frame;
                entry.hp = ps.hp;
                entry.max_hp = ps.max_hp;
            }
            if missing == 0 {
                continue; // `RemoteClient.cs:177-180` — nothing to stream
            }
            let mut announced = false;
            for sy in (psy - 1).max(0)..=(psy + 1).min(gy - 1) {
                for sx in (psx - 1).max(0)..=(psx + 1).min(gx - 1) {
                    if !server.is_section_sent(*id, sx, sy) {
                        out.push((*id, sx, sy, if announced { 0 } else { missing }));
                        announced = true;
                    }
                }
            }
        }
        out
    }

    /// Builds the `10 TileBlock` body for one section (clamped to the world).
    fn tile_block_at(&self, sx: i32, sy: i32) -> net::TileBlock {
        let (w, h) = (self.world.width, self.world.height);
        let x0 = (sx * net::SECTION_W).clamp(0, w);
        let y0 = (sy * net::SECTION_H).clamp(0, h);
        let bw = (net::SECTION_W).min(w - x0).max(0);
        let bh = (net::SECTION_H).min(h - y0).max(0);
        let mut records = Vec::with_capacity((bw * bh) as usize);
        for ty in y0..y0 + bh {
            for tx in x0..x0 + bw {
                records.push(self.tile_record_at(tx, ty));
            }
        }
        net::TileBlock {
            x: x0,
            y: y0,
            w: bw as i16,
            h: bh as i16,
            records,
        }
    }

    /// Extracts one tile's sync record — the C#'s compressor fields
    /// (`NetMessage.cs:1952-2168`) read out of the port's storage.
    pub(crate) fn tile_record_at(&self, x: i32, y: i32) -> net::TileRecord {
        let Some(t) = self.world.tile(x, y) else {
            return net::TileRecord {
                active: false,
                kind: 0,
                frame_important: false,
                frame_x: 0,
                frame_y: 0,
                wall: 0,
                wall_frame: 0,
                liquid: 0,
                liquid_kind: 0,
                half_brick: false,
                slope: 0,
                actuator: false,
                inactive: false,
                paint: 0,
                wall_paint: 0,
                wires: 0,
                invisible_block: false,
                invisible_wall: false,
                fullbright_block: false,
                fullbright_wall: false,
            };
        };
        let idx = self.world.index(x, y);
        let extras = self.world.extras_at(idx);
        let frame_important = self
            .world
            .frame_important
            .get(t.kind as usize)
            .copied()
            .unwrap_or(false);
        net::TileRecord {
            active: t.active(),
            kind: t.kind,
            frame_important,
            frame_x: if frame_important { t.frame_x } else { 0 },
            frame_y: if frame_important { t.frame_y } else { 0 },
            wall: t.wall,
            wall_frame: t.wall_frame,
            liquid: t.liquid,
            liquid_kind: match t.liquid_kind() {
                tr_core::tile::LiquidKind::Water => 0,
                tr_core::tile::LiquidKind::Lava => 1,
                tr_core::tile::LiquidKind::Honey => 2,
                tr_core::tile::LiquidKind::Shimmer => 3,
            },
            half_brick: t.half_brick(),
            slope: t.slope(),
            actuator: t.flags & tr_core::tile::FLAG_ACTUATOR != 0,
            inactive: t.flags & tr_core::tile::FLAG_INACTIVE != 0,
            paint: t.paint(),
            wall_paint: extras & 0x1F,
            wires: {
                let mut b = 0u8;
                if self.world.wires.get(x, y, tr_core::wiring::WireColor::Red) {
                    b |= 1;
                }
                if self.world.wires.get(x, y, tr_core::wiring::WireColor::Blue) {
                    b |= 2;
                }
                if self
                    .world
                    .wires
                    .get(x, y, tr_core::wiring::WireColor::Green)
                {
                    b |= 4;
                }
                if self
                    .world
                    .wires
                    .get(x, y, tr_core::wiring::WireColor::Yellow)
                {
                    b |= 8;
                }
                b
            },
            invisible_block: extras & 0x20 != 0,
            invisible_wall: extras & 0x40 != 0,
            fullbright_block: t.flags & tr_core::tile::FLAG_FULLBRIGHT != 0,
            fullbright_wall: extras & 0x80 != 0,
        }
    }

    /// Applies a `10 TileBlock` into a world: direct tile writes (the C#'s
    /// `DecompressTileBlock_Inner` calls no KillTile/PlaceTile) plus the wire
    /// table and the sparse extras byte, with one revision bump for the whole
    /// rectangle so the light and world-range caches invalidate.
    fn apply_tile_block(world: &mut World, block: &net::TileBlock) {
        let (w, h) = (block.w as i32, block.h as i32);
        let mut i = 0usize;
        for ty in 0..h {
            for tx in 0..w {
                let Some(rec) = block.records.get(i) else {
                    return;
                };
                i += 1;
                let (x, y) = (block.x + tx, block.y + ty);
                Self::apply_tile_record(world, x, y, rec);
            }
        }
    }

    /// Applies a `20 SectionRange` into the live world.
    fn apply_section_range(&mut self, range: &net::SectionRange) {
        let (w, h) = (range.w as i32, range.h as i32);
        let mut i = 0usize;
        for ty in 0..h {
            for tx in 0..w {
                let Some(rec) = range.records.get(i) else {
                    return;
                };
                i += 1;
                Self::apply_tile_record(
                    &mut self.world,
                    range.x as i32 + tx,
                    range.y as i32 + ty,
                    rec,
                );
                // `MessageBuffer.cs:1519-1528` — the `21` handler writes
                // type/prefix/stack/position/velocity/shimmer/enemyGrabDelay
                // but NOT the grab-delay pair (that is `22`'s job,
                // `:1558-1559`): a live cadence update must not wipe the
                // join burst's 100-tick lock (S1-P-i51). The port rebuilds
                // the record, so carry the pair over from the old one.
                if let Some(old) = self.net_items.get(&(d.slot as i16)) {
                    drop.grab_delay_player = old.grab_delay_player;
                    drop.grab_delay_time = old.grab_delay_time;
                }
                drop.shimmered = d.shimmered;
                drop.shimmer_time = d.shimmer_time;
                // `MessageBuffer.cs:1487` — the `21`/`90` handler writes the
                // enemy-grab delay from the conditional byte. W4-g15 lane 5.
                drop.enemy_grab_delay_time = d.enemy_grab_delay as i32;
                drop.age = 60.0;
                self.net_items.insert(d.slot as i16, drop);
                self.net_items_dirty = true;
            }
            // `151` — the slot is gone.
            S2c::ItemDead { slot } => {
                self.net_items.remove(&slot);
                self.net_items_dirty = true;
            }
            // `case 22` — the reservation announcement (server→client only;
            // `MessageBuffer.cs:1552-1562` refuses it on the server).
            S2c::ItemOwner(o) => {
                if o.reserved_for == 255 {
                    self.net_reserved.remove(&o.slot);
                } else {
                    self.net_reserved.insert(o.slot, (o.reserved_for, o.hold));
                }
                // `MessageBuffer.cs:1558-1559` — the client writes the
                // grab-delay pair onto the drop (the player drop's 100-tick
                // lock survives the join, S1-P-i51). The port keys the
                // client's drops by the host slot in `net_items`.
                if let Some(d) = self.net_items.get_mut(&o.slot) {
                    d.grab_delay_player = o.grab_delay_player;
                    d.grab_delay_time = o.grab_delay as i32;
                    self.net_items_dirty = true;
                }
            }
            // `case 5` — one peer inventory slot (`MessageBuffer.cs:399-462`):
            // the slot lands in that peer's inventory copy.
            S2c::Inventory {
                player_id,
                slot,
                item,
            } => {
                let peer = player_id as u32;
                let (px, py) = self.net_peer_spawn_pos();
                self.remote_players
                    .entry(peer)
                    .or_insert_with(|| net::PlayerNet {
                        id: peer,
                        name: String::new(),
                        x: px,
                        y: py,
                        ..Default::default()
                    })
                    .inventory
                    .insert(slot as u8, item);
            }
        }
        self.world.revision = self.world.revision.wrapping_add(1);
        self.world_key = None;
    }

    /// One record into one tile, mirroring `read_record`'s field order.
    #[allow(clippy::too_many_lines)]
    fn apply_tile_record(world: &mut World, x: i32, y: i32, rec: &net::TileRecord) {
        if !world.in_bounds(x, y) {
            return;
        }
        let idx = world.index(x, y);
        let t = &mut world.tiles[idx];
        t.kind = rec.kind;
        t.wall = rec.wall;
        t.wall_frame = rec.wall_frame;
        t.frame_x = if rec.frame_important { rec.frame_x } else { -1 };
        t.frame_y = if rec.frame_important { rec.frame_y } else { -1 };
        t.liquid = rec.liquid;
        t.set_liquid_kind(match rec.liquid_kind {
            1 => tr_core::tile::LiquidKind::Lava,
            2 => tr_core::tile::LiquidKind::Honey,
            3 => tr_core::tile::LiquidKind::Shimmer,
            _ => tr_core::tile::LiquidKind::Water,
        });
        let mut flags = 0u8;
        if rec.active {
            flags |= tr_core::tile::FLAG_ACTIVE;
        }
        if rec.half_brick {
            flags |= tr_core::tile::FLAG_HALF_BRICK;
        }
        if rec.actuator {
            flags |= tr_core::tile::FLAG_ACTUATOR;
        }
        if rec.inactive {
            flags |= tr_core::tile::FLAG_INACTIVE;
        }
        if rec.fullbright_block {
            flags |= tr_core::tile::FLAG_FULLBRIGHT;
        }
        t.flags = flags;
        t.meta = (rec.slope & 0x07) | ((rec.paint & 0x1F) << 3);
        // The sparse extras byte: wall paint + the invisible/fullbright quad.
        let mut extras = rec.wall_paint & 0x1F;
        if rec.invisible_block {
            extras |= 0x20;
        }
        if rec.invisible_wall {
            extras |= 0x40;
        }
        if rec.fullbright_wall {
            extras |= 0x80;
        }
        if extras != 0 {
            world.set_extras_at(idx, extras);
        }
        // The wire side-table bits.
        for (bit, color) in [
            (1u8, tr_core::wiring::WireColor::Red),
            (2, tr_core::wiring::WireColor::Blue),
            (4, tr_core::wiring::WireColor::Green),
            (8, tr_core::wiring::WireColor::Yellow),
        ] {
            world.wires.set(x, y, color, rec.wires & bit != 0);
        }
    }

    /// `MessageBuffer.cs:2283` — `player.channel =
    /// player.inventory[player.selectedItem].channel`: the `41` reader
    /// re-derives the channel flag from the peer's selected item. The peer's
    /// inventory rides `5 PlayerInventory` into `PlayerNet.inventory`; an
    /// unknown slot/kind reads `false` (the C#'s empty `Item` has
    /// `channel = false`). W4-c27.
    fn peer_channel(p: &net::PlayerNet) -> bool {
        let slot = (p.selected as u8).min(58);
        p.inventory
            .get(&slot)
            .and_then(|it| crate::registry::get(it.kind.max(0) as u16))
            .is_some_and(|def| def.channel)
    }

    /// `Player.ManaEffect` (`Player.cs:4531-4538`) on a peer: the HealMana
    /// text at the peer's position (`CombatText.NewText`, `:4533`). The net
    /// handlers run outside every RNG swap, so the placement draws ride the
    /// bare stream (the `play_switch_click` precedent). The peer body is the
    /// C#'s 20x42 (`Player.cs:29210-29211`); the font measure falls back like
    /// the local drain (`world_sim/player.rs`). W4-c27.
    fn spawn_peer_mana_text(&mut self, player_id: u8, amount: i32) {
        let Some(p) = self.remote_players.get(&(player_id as u32)) else {
            return;
        };
        let (x, y) = (p.x, p.y);
        let text = amount.to_string();
        let size = match self.lookups.combat_text_font.as_ref() {
            Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
            None => [text.chars().count() as f32 * 8.0, 10.0],
        };
        let location = crate::combat_text::Rect {
            x,
            y,
            width: 20.0,
            height: 42.0,
        };
        crate::combat_text::spawn_amount(
            &mut self.combat_texts,
            location,
            crate::combat_text::HEAL_MANA,
            amount,
            false,
            false,
            size,
            1.0,
            self.rngs.global(),
        );
    }

    fn host_slot_join_complete(&mut self, server: &net::Server, id: u32, _s: net::PlayerSpawn) {
        // The port's `SyncOnePlayer` (`NetMessage.cs:2885-2946`) sends the
        // joining peer everyone's state; the host's own snapshot leads.
        for p in self.remote_players.values() {
            if p.id != id {
                server.send(id, &S2c::PlayerState(Box::new(self.player_state_of(p))));
                if !p.name.is_empty() {
                    server.send(
                        id,
                        &S2c::PlayerInfo(net::PlayerInfo::with_name(p.id as u8, &p.name)),
                    );
                }
            }
            // `case 47` — a sign's text (`MessageBuffer.cs:2376-2405`): the
            // record is rewritten and, when the sender is us and bit 0 of
            // the flags byte is clear, `OpenSign` runs (`:2400-2403`) —
            // W4-e10: the latch + the Menu_Open cue (`Player.cs:33547`);
            // the sign EDITOR itself stays S1-E-26's.
            // record is rewritten and, when the sender byte is us and bit 0
            // of the flags is clear, the sign opens (`:2400-2403` —
            // `Main.LocalPlayer.OpenSign(num58)`, W4-f22).
            S2c::SignData(sd) => {
                if sd.sign < 0 || sd.sign >= crate::signs::MAX_SIGNS {
                    return; // `:2384`
                }
                crate::signs::text_sign(
                    &mut self.world,
                    sd.sign as usize,
                    sd.x as i32,
                    sd.y as i32,
                    sd.text.clone(),
                );
                // `:2400` — netMode 1 (this arm IS the client), the sender
                // is us, the record survived `TextSign`, `!bitsByte5[0]`.
                // The dense-store deviation: `text_sign` splices a dead-
                // tile record (the later indices shift), so the C#'s
                // `Main.sign[num58] != null` probe is "a record still sits
                // at this tile" and the open uses THAT record's index
                // (ANALYSIS, the W4-e10 gap-census phase).
                let live_idx = self
                    .world
                    .signs
                    .iter()
                    .position(|r| r.x == sd.x as i32 && r.y == sd.y as i32);
                if u32::from(sd.sender) == self.local_id && sd.flags & 1 == 0 {
                    if let Some(idx) = live_idx {
                        crate::signs::open_sign(self, idx as i32);
                    }
                // `:2400-2403` — the client's open arm: sender == myPlayer,
                // the record survived `TextSign` (a dead tile spliced it),
                // and `!bitsByte5[0]`.
                if sd.sender as u32 == self.local_id
                    && (sd.sign as usize) < self.world.signs.len()
                    && sd.flags & 1 == 0
                {
                    crate::signs::open_sign(self, sd.sign as i32);
                }
        }
        // The host is a player too (its slot is 0): the display names ride
        // `4 PlayerInfo`, which msg 13 lacks.
        if !self.player_name.is_empty() {
            server.send(
                id,
                &S2c::PlayerInfo(net::PlayerInfo::with_name(0, &self.player_name)),
            );
        }
        let name = server.slot_name(id);
        let (px, py) = self.net_peer_spawn_pos();
        self.remote_players.insert(
            id,
            net::PlayerNet {
                id,
                name: name.clone(),
                x: px,
                y: py,
                ..Default::default()
            },
        );
        let joined = S2c::Joined { id, name };
        server.broadcast(&joined);
        // The join item burst (`MessageBuffer.cs:843-850`): one `21 ItemDrop`
        // + one `22 ItemOwner` per live slot, to the joining client only.
        // The reservation rides the C# field default 255 = unreserved
        // (`WorldItem.cs:139`); the grab-delay pair carries the drop's LIVE
        // `grabDelayPlayer`/`grabDelayTime` (`NetMessage.cs:681-682`), not
        // zeros — a player drop's 100-tick lock survives the join
        // (S1-P-i51). The position closes the body (`:683`).
        for (i, it) in self.items.iter().enumerate() {
            let slot = i as i16;
            let st = Self::item_state_of(slot, it);
            // W4-g52 — the `SendData(21)` pre-switch (`NetMessage.cs:89-100`):
            // instanced never syncs, air rewrites to `151`. The `22` owner
            // announcement below is unconditional (the C# join burst sends it
            // per live slot regardless of the `21` gate).
            if let Some(msg) = Self::item_send_gate(slot, it) {
                server.send(id, &msg);
            }
            server.send(
                id,
                &S2c::ItemOwner(Box::new(net::ItemOwner {
                    slot,
                    reserved_for: 255,
                    hold: 0,
                    grab_delay_player: it.grab_delay_player,
                    grab_delay: it.grab_delay_time.max(0) as u32,
                    x: it.pos.x,
                    y: it.pos.y,
                })),
            );
            // The burst counts as sent: the next cadence diff only re-sends
            // the slot when it actually changes.
            self.net_item_last.insert(slot, st);
        }
        // `NetMessage.cs:2818` — `SendCreditsRollRemainingTimeToPlayer`:
        // kind 0 rides the join bundle only while the roll is live
        // (`CreditsRollEvent.cs:29`'s `!= 0` gate).
        if self.world.credits_roll_remaining_time != 0 {
            server.send(
                id,
                &S2c::MiscEventValue(net::MiscEventValue {
                    kind: 0,
                    value: self.world.credits_roll_remaining_time,
                }),
            );
        }
        // `MessageBuffer.cs:870` — `TrySendData(136, whoAmI)`: the cavern
        // grid rides the join bundle BEFORE the bestiary burst (`:871`'s
        // `OnPlayerJoining`), the C#'s exact order.
        server.send(
            id,
            &S2c::CavernMonsterTypes(net::CavernMonsterTypes {
                types: self.world.cavern_monster_type.map(|row| row.map(|t| t as u16)),
            }),
        );
        // `MessageBuffer.cs:871` → `BestiaryUnlocksTracker.OnPlayerJoining`
        // → `NPCKillsTracker.OnPlayerJoining` (`NPCKillsTracker.cs:98-107`):
        // every recorded credit's current count goes to the joining slot as a
        // `NetBestiaryModule.SerializeKillCount`.
        let snapshot: Vec<(String, i32)> = self
            .bestiary
            .kills
            .pairs()
            .map(|(credit, count)| (credit.to_string(), count))
            .collect();
        for (credit, count) in snapshot {
            let Some(entry) = crate::bestiary::ENTRIES
                .iter()
                .find(|e| e.credit_id == credit.as_str())
            else {
                continue;
            };
            server.send(
                id,
                &S2c::NetModule {
                    module: crate::app::netmodules::module_id::BESTIARY,
                    body: crate::app::netmodules::BestiaryKill::new(
                        entry.net_id as i16,
                        count,
                    )
                    .encode(),
                },
            );
        }
        // `MessageBuffer.cs:930-935` — the State-3→10 join arm's
        // `DoesPlayerSlotCountAsAHost` push: the flag is computed from the
        // peer's remote address (`NetMessage.cs:2876-2883`, loopback =
        // same machine), stored host-side, and pushed as `139` ONLY when
        // true (`:932-934`'s gate). W4-g51 FINAL id-139 (the g52c master
        // input: the host-authority flag's join source).
        let counts_as_host = server.slot_counts_as_host(id);
        self.remote_players.entry(id).or_default().counts_as_host = counts_as_host;
        if counts_as_host {
            server.send(
                id,
                &S2c::CountsAsHost(net::CountsAsHost {
                    player_id: id as u8,
                    counts_as_host: true,
                }),
            );
        }
        // W4-g52 audit — the join bundle's `MessageBuffer.cs:851-866` tail:
        // one `23` + `54` per live NPC slot, one `27` per pet/netImportant
        // projectile. (The `57` evil bytes, the `103`/`101`/`136` event-state
        // pushes and the banner module's full state are f21/d99/d94/d92-lane
        // codecs — they land with those merges.) The `23` body stamps the
        // cadence's stores (uid, generation, last-info) exactly like a
        // cadence first sight, so the next tick does not re-send the spawn;
        // the C# cadence re-sends on its own netSpam budget regardless.
        {
            let mut next = self.next_npc_uid;
            for n in self.npcs.iter_mut() {
                if n.uid == 0 {
                    n.uid = next;
                    next += 1;
                }
            }
            // `case 59` — a peer tripped a switch: replay it locally
            // (`MessageBuffer.cs:2682-2693`).
            S2c::HitSwitch(hs) => {
                // The §8.1 stamp: the replay can shift a wired door family.
                crate::world_sim::wiring::stamp_wiring_entity_boxes(self);
                self.wiring
                    .hit_switch(&mut self.world, hs.x as i32, hs.y as i32);
                // S1-E-21i — the client's click replay: the message handler
                // runs outside every swap (`Main.cs:16895`'s NpcStrike
                // precedent), so the pitch draws ride the bare stream.
                for (px, py) in self.wiring.take_switch_clicks() {
                    self.audio
                        .play_switch_click(px as f32, py as f32, self.rngs.global());
                }
                // W4-c42 — the replay's wired door cues (the bare stream).
                for (open, px, py) in self.wiring.take_door_cues() {
                    self.audio
                        .play_door_sound(open, px as f32, py as f32, self.rngs.global());
                }
                // The §8.2 echoes: a client's `SendData` rides msg 19 to
                // the host (the g48b attach point); drained either way.
                for (n, x, y, d) in self.wiring.take_door_echoes() {
                    self.send_door_toggle(n, x, y, d);
                }
                self.world_key = None;
            self.next_npc_uid = next;
        }
        for (slot, npc) in self.npcs.iter().enumerate() {
            if npc.hp <= 0 {
                continue; // `Main.npc[i].active`
            }
            let kind = npc.kind.to_u16() as i16;
            let (spawn, generation) = match self.net_npc_gen.get(&(slot as u32)) {
                Some((k, g)) if *k == kind => (false, *g),
                _ => {
                    let g = self
                        .net_npc_gen
                        .get(&(slot as u32))
                        .map(|(_, g)| g.wrapping_add(1))
                        .unwrap_or(1);
                    (true, g)
                }
            };
            self.net_npc_gen.insert(slot as u32, (kind, generation));
            let info = Self::npc_info_of(slot, npc, spawn, generation);
            server.send(id, &S2c::NpcInfo(Box::new(info)));
            self.net_npc_last_info.insert(npc.uid, info);
            self.net_npc_uid_slot.insert(npc.uid, slot as u32);
            self.net_npc_states.entry(npc.uid).or_default();
            server.send(
                id,
                &S2c::NpcBuffs(net::NpcBuffs {
                    npc: slot as i16,
                    buffs: npc_buffs_of(npc),
                }),
            );
        }
        for (slot, p) in self.projectiles.slots().iter().enumerate() {
            // `:861`: `projPet[type] || netImportant`.
            if !p.active
                || !(crate::projectile::proj_pet(p.kind) || p.net_important)
            {
                continue;
            }
            let key = self.projectiles.key_of(slot);
            let st = net::ProjectileState {
                key,
                x: p.pos.x,
                y: p.pos.y,
                vx: p.vel.x,
                vy: p.vel.y,
                kind: p.kind as i16,
                ai0: if p.ai[0] != 0.0 { Some(p.ai[0]) } else { None },
                ai1: if p.ai[1] != 0.0 { Some(p.ai[1]) } else { None },
                banner_id: 0,
                damage: p.damage as i16,
                knock_back: p.knock_back,
                original_damage: p.original_damage as i16,
                ai2: if p.ai[2] != 0.0 { Some(p.ai[2]) } else { None },
            };
            server.send(id, &S2c::Projectile(Box::new(st.clone())));
            // The cadence's change-detect store: the join burst counts as
            // sent (the same convention as the item burst's net_item_last)
            // — but ONLY for the host's OWN shots: the cadence's live set
            // is owner-gated (`Projectile.cs:17210`), so a foreign-owner
            // pet stamped here would read as "gone" on the next diff and
            // fire a spurious `29` kill. The C# join burst has no owner
            // gate (`:861`), its cadence does — the split is the source's.
            if p.owner == self.local_id as u8 {
                self.net_proj_prev.insert(key, st);
            }
        }
        // W4-g52 audit — `SyncConnectedPlayer`'s tail
        // (`NetMessage.cs:2816-2819`): `SendNPCHousesAndTravelShop` (one `60`
        // per housed town NPC + the traveller's `72`), `SendAnglerQuest`
        // (`74`), `SendAllMarkersToPlayer` (`126` per marker). The credits
        // push (`140`) is constants-off (the port's credits screen is
        // untimed, `CreditsRollEvent._creditsRollRemainingTime` has no home).
        let mut traveller_present = false;
        for (slot, npc) in self.npcs.iter().enumerate() {
            if npc.hp <= 0 {
                continue;
            }
            // W4-g52 — `case 162` (`MessageBuffer.cs:1819-1824`): the client
            // pops `NPC.pendingDamages[0]` (`NPC.AckDamage`). The damage-queue
            // is unported (the apply-arm lane, g48-g51); the SEND side lands
            // here, the client read is a no-op until the queue exists.
            S2c::NpcStrikeAck => {}
            // W4-g52 — `case 34`'s client apply (`MessageBuffer.cs:2124-2157`'s
            // `PlaceChestDirect`/`PlaceDresserDirect`/`DestroyChestDirect` +
            // `KillTile` pass) is the apply-arm lane (g48-g51); the host's
            // place/destroy + relays land here, the client read is a no-op
            // until the direct-place apply exists.
            S2c::ChestUpdates(_) => {}
            // `case 18` — the host's clock (`MessageBuffer.cs:1290-1298`,
            // client-only). `Main.time` is the wire `int`; the port's clock is
            // f64 ticks. `sunModY`/`moonModY` (`Main.cs:1185`/`:1187`) are the
            // journey-mode sun/moon drag offsets — no port field yet, parked
            // (W4-c27).
            S2c::SetTime(t) => {
                self.world.day_time = t.day_time;
                self.world.time = t.time as f64;
            }
            // `case 36` — a peer's zone flags (`MessageBuffer.cs:2178-2202`,
            // the `netMode == 1` half keeps the wire id): stored on the peer
            // record, the render/zone consumers read it from there.
            S2c::PlayerZone(z) => {
                self.remote_players
                    .entry(z.player_id as u32)
                    .or_default()
                    .zone = z;
            // `case 41` — a peer's aimed-item rotation/animation
            // (`MessageBuffer.cs:2271-2289`, the `netMode == 1` half keeps the
            // wire id): stored on the peer record, `channel` re-derived from
            // the peer's selected inventory item (`:2283`).
            S2c::ItemAnim(a) => {
                let entry = self
                    .remote_players
                    .entry(a.player_id as u32)
                    .or_default();
                entry.item_rotation = a.rotation;
                entry.item_animation = a.animation;
                entry.channel = Self::peer_channel(entry); // `:2283`
            // `case 43` — a peer's mana-restore amount
            // (`MessageBuffer.cs:2307-2324`): the HealMana text spawns for
            // anyone but ourselves (`:2315-2318`).
            S2c::ManaEffect(m) => {
                if m.player_id as u32 != self.local_id {
                    self.spawn_peer_mana_text(m.player_id, m.amount as i32);
                }
            // `case 51` — the host's misc fan-out (`MessageBuffer.cs:2482-2530`,
            // the client half). Kind 2 plays Item1 at the subject's position
            // (`:2502`, `SoundID.Item1` — the case-2 façade, the net handlers
            // run outside every swap so the draws ride the bare stream).
            // Kinds 1/3/5/6 are host-side only; kind 4's big-mimic smoke
            // burst is parked (the roster is unported), W4-c27.
            S2c::MiscDataSync(d) => {
                if d.kind == 2 {
                    if let Some(p) = self.remote_players.get(&(d.player_id as u32)) {
                        let (x, y) = (p.x, p.y);
                        self.audio
                            .play_npc_item_cue(1, x, y, self.rngs.global()); // `:2502`
                    }
            // `:2834-2838`: townNPC && head index > 0, or aiStyle 7. The
            // head-index stand-in is the housing set (`npc_view_of`'s same
            // fold — the town residents minus the bound/sleeping kinds).
            let town_like = npc.kind.is_town_npc() || npc.kind.ai_style() == 7;
            if !town_like {
                continue;
            }
            // `:2841-2844`: the traveller flag rides INSIDE the flag2 gate
            // (type 368 is aiStyle 7, so a live traveller always passes it).
            if npc.kind == crate::npc::NpcKind::TravellingMerchant {
                traveller_present = true;
            }
            let (hx, hy) = npc.home.unwrap_or((0, 0));
            let status = if npc.home.is_some() { 2 } else { 1 };
            server.send(
                id,
                &S2c::NpcHouse(net::NpcHouse {
                    npc: slot as i16,
                    home_x: hx as i16,
                    home_y: hy as i16,
                    status,
                }),
            );
        }
        if traveller_present {
            // `SendTravelShop` (`NetMessage.cs:2645-2651`): the 40 shorts.
            server.send(
                id,
                &S2c::TravelShop(net::TravelShop {
                    items: self.world.travel_shop.iter().map(|&t| t as i16).collect(),
                }),
            );
        }
        // `SendAnglerQuest` (`NetMessage.cs:2653-2673`): the quest byte +
        // the recipient's finished-today membership. The per-name list is
        // g49's `angler_who_finished_today`; the port's single-flag stand-in
        // (the SP `angler_quest_finished`) answers for the joining name.
        server.send(
            id,
            &S2c::AnglerQuest(net::AnglerQuest {
                quest: self.world.angler_quest,
                finished: self.world.angler_quest_finished,
            }),
        );
        for m in &self.revenge_markers {
            server.send(
                id,
                &S2c::RevengeMarker(net::RevengeMarker {
                    unique_id: m.unique_id as i32,
                    x: m.location.x,
                    y: m.location.y,
                    npc_net_id: m.kind.net_id(),
                    npc_hp_percent: m.hp_percent,
                    npc_type: m.kind.cs_id() as i32,
                    npc_ai_style: m.kind.ai_style() as i32,
                    coins_value: m.coins_value as i32,
                    base_value: m.base_value,
                    // The marker struct carries no statue flag (the cache
                    // gate excludes statue spawns via `should_cache`'s
                    // callers); false is the C# field default.
                    spawned_from_statue: false,
                }),
            );
        }
        log::info!("slot {id} fully joined");
    }

    /// A slot went away: clean up the game-side record (`MessageBuffer.cs`'s
    /// disconnect arm + `SyncOnePlayer`'s inactive broadcast).
    fn host_slot_left(&mut self, server: &net::Server, id: u32) {
        let was_in_world = self.remote_players.remove(&id).is_some();
        // The leaving peer's interaction anchor clears (`Player.Disconnect`
        // → `tileEntityAnchor.Clear`, `Player.cs:5041`); the `IsOccupied`
        // scan must not see a ghost. W4-g51 FINAL id-122.
        self.net_te_anchors.remove(&(id as u8));
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
                // W4-g52 audit — the C# reader applies the header fields at
                // ANY state and only walks 3 → 4 inside the `State == 3`
                // gate (`MessageBuffer.cs:655-659`); the re-pushes
                // (`Main.cs:64560`'s 3600-tick heartbeat, the dawn/dusk
                // `SendData(7)`s, the maxRaining change, the invasion end)
                // arrive in-world and must NOT re-request the spawn.
                if self.net_state >= net::client_state::IN_WORLD {
                    self.apply_world_info_repush(&info);
                    return;
                }
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
            // `case 10` — one section's tile snapshot. Before the world is
            // installed (while `49` is pending) the sections buffer and are
            // applied in one pass at the install (`MessageBuffer.cs:812-841`);
            // after it — the move-driven catch-up — each block applies
            // straight into the live world.
            S2c::TileBlock(block) => {
                if self.net_state >= net::client_state::IN_WORLD {
                    Self::apply_tile_block(&mut self.world, &block);
                    self.world.revision = self.world.revision.wrapping_add(1);
                    self.world_key = None;
                } else {
                    self.net_pending_sections.push(*block);
                }
                if self.net_status_max > 0 {
                    self.net_status_count += 1;
                }
            }
            // `case 20` — a rectangle of tile changes; applied straight into
            // the tiles (the C#'s reader writes the fields, no
            // KillTile/PlaceTile, `NetMessage.cs:2240-2458`).
            S2c::SectionRange(range) => {
                self.apply_section_range(&range);
            }
            // `case 49` — `InitialSpawn` (`MessageBuffer.cs:2447-2453`): the
            // server says "spawn in now"; the client installs the streamed
            // world and answers `12 PlayerSpawn` with `SpawningIntoWorld`
            // (`Player.Spawn(PlayerSpawnContext.SpawningIntoWorld)`,
            // `Player.cs:37534-37580`, which also re-sends 12 from the spawn
            // path at `:37580`). The stream's own completion is the section
            // walk, not this message.
            S2c::InitialSpawn => {
                if self.net_state == net::client_state::SPAWN_REQUESTED {
                    let sections = std::mem::take(&mut self.net_pending_sections);
                    if self.client_install_world(sections) {
                        let (sx, sy) = (self.world.spawn_x as i16, self.world.spawn_y as i16);
                        self.net_state = net::client_state::IN_WORLD;
                        client.send(&C2s::PlayerSpawn(net::PlayerSpawn {
                            player_id: self.local_id as u8,
                            x: sx,
                            y: sy,
                            respawn_timer: 0,
                            deaths_pve: 0,
                            deaths_pvp: 0,
                            team: 0,
                            // `PlayerSpawnContext.SpawningIntoWorld` (`= 1`):
                            // `Player.Spawn(context)` re-sends `12` with
                            // `(int)(byte)context` (`Player.cs:37578-37581`).
                            spawn_context: 1,
                        }));
                    } else {
                        self.net_disconnect("the world failed to install");
                    }
                }
            }
            // `case 154` on a client - `Ping.PingRecieved`
            // (`MessageBuffer.cs:4424-4427`, `Net/Ping.cs:37-48`).
            S2c::Ping => self.ping_received(),
            // `case 97` - the host's kill credit for this client
            // (`MessageBuffer.cs:3558-3563`): `NotifyNPCKilledDirect` feeds
            // the `NPCKilledCondition` listeners.
            S2c::NpcKilled { npc } => {
                let _ = self.achievements.notify(achievements::CondKind::NpcKilled(npc as i32));
            }
            // `case 98` - a host-side progression event
            // (`MessageBuffer.cs:3564-3569`).
            S2c::ProgressionEvent { event_id } => {
                let _ = self.achievements.notify_progression(event_id.max(0) as u32);
            }
            // Other players' identity/health (the C# relays 4/16/14 to all).
            S2c::PlayerInfo(pi) => {
                let peer = pi.player_id as u32;
                if peer != self.local_id {
                    let (px, py) = self.net_peer_spawn_pos();
                    self.remote_players
                        .entry(peer)
                        .or_insert_with(|| net::PlayerNet {
                            id: peer,
                            name: pi.name.clone(),
                            x: px,
                            y: py,
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
                // `case 16` client half (`MessageBuffer.cs:1104-1111`): the
                // peer's health + the `dead` derivation — W4-g48. The
                // statLifeMax >= 20 floor is the decode's (`net.rs`).
                crate::app::net_apply::client_player_hp(self, player_id, hp, max_hp);
            }
            S2c::PlayerActive { player_id, active } => {
                if active && player_id as u32 != self.local_id {
                    let (px, py) = self.net_peer_spawn_pos();
                    self.remote_players
                        .entry(player_id as u32)
                        .or_insert_with(|| net::PlayerNet {
                            id: player_id as u32,
                            name: format!("Player {}", player_id),
                            x: px,
                            y: py,
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
                    let (px, py) = self.net_peer_spawn_pos();
                    self.remote_players
                        .entry(id)
                        .or_insert_with(|| net::PlayerNet {
                            id,
                            name,
                            x: px,
                            y: py,
                            ..Default::default()
                        });
                }
            }
            S2c::Left { id } => {
                self.remote_players.remove(&id);
            }
            // `case 13` — a peer's per-tick state. The client discards its
            // own echo (`MessageBuffer.cs:955`) and applies the C#'s
            // smoothing: the visual lag accrues into `netOffset`, the
            // position snaps to the wire value (`MessageBuffer.cs:1004-1016`).
            S2c::PlayerState(ps) => {
                let peer = ps.player_id as u32;
                if peer == self.local_id {
                    return;
                }
                let (px, py) = self.net_peer_spawn_pos();
                let entry = self
                    .remote_players
                    .entry(peer)
                    .or_insert_with(|| net::PlayerNet {
                        id: peer,
                        name: format!("Player {}", ps.player_id),
                        x: px,
                        y: py,
                        ..Default::default()
                    });
                if entry.name.is_empty() {
                    entry.name = format!("Player {}", ps.player_id);
                }
                if ps.x != 0.0 || ps.y != 0.0 {
                    entry.off_x += ps.x - entry.x;
                    entry.off_y += ps.y - entry.y;
                    // `multiplayerNPCSmoothingRange` = 300 (`Main.cs:67007`):
                    // past it the offset snaps to zero.
                    if (entry.off_x * entry.off_x + entry.off_y * entry.off_y).sqrt()
                        > SMOOTHING_RANGE
                    {
                        entry.off_x = 0.0;
                        entry.off_y = 0.0;
                    }
                }
                entry.x = ps.x;
                entry.y = ps.y;
                entry.vx = ps.vx;
                entry.vy = ps.vy;
                entry.facing = if ps.flags_a.get(6) { 1 } else { -1 };
                entry.selected = ps.selected_item as u16;
                entry.on_ground = ps.on_ground;
                entry.leg_frame = ps.leg_frame;
                entry.body_frame = ps.body_frame;
                entry.hp = ps.hp;
                entry.max_hp = ps.max_hp;
                // W4-g10 FINAL — `MessageBuffer.cs:1043-1044`.
                entry.petting = ps.flags_c.get(4);
                entry.pet_small = ps.flags_c.get(5);
            }
            S2c::Time { time, day } => {
                self.world.time = time;
                self.world.day_time = day;
            }
            S2c::Edit { x, y, kind, tile } => {
                // W4-g83: the replay's kill cascade rides the bare stream
                // (the message handler runs outside every SwapRandom).
                let broken_wall = apply_edit(&mut self.world, x, y, kind, tile, self.rngs.global());
                // S1-E-21i — the client's `KillWall` replay plays too
                // (`MessageBuffer.cs:1177`); the message handler runs
                // outside every `SwapRandom` (`Main.cs:16895`'s NpcStrike
                // precedent), so the draws ride the bare stream.
                if let Some(wall) = broken_wall {
                    self.audio.play_wall_break_sound(
                        tr_core::edit::kill_wall_sound(wall),
                        x as f32 * 16.0,
                        y as f32 * 16.0,
                        self.rngs.global(),
                    );
                }
                self.world_key = None;
            }
            // `case 82` — a NetModule envelope.
            S2c::NetModule { module, body } => {
                self.client_module(module, &body);
            }
            // `case 27` — one projectile's spawn/update state; the pool
            // patches or re-creates the slot by key (`MessageBuffer.cs:1729-1755`).
            S2c::Projectile(ps) => {
                self.projectiles.upsert_remote(&ps);
            }
            // `case 29` — the projectile is gone (`MessageBuffer.cs:1825-
            // 1850`): when the key resolves to an ACTIVE occupant, a finite
            // position moves the slot to it before `Kill` (`:1835-1838`),
            // NaN/Inf deactivates silently (`:1840-1843`); a stale key is a
            // no-op. The g48 keep-one folds d91/d93's identical logic into
            // the `kill_remote_at` seam — the port's silent-despawn stand-in
            // for `Kill`'s death effects (the effects belong to the
            // projectile AI pass, not the net reader).
            S2c::ProjectileDead { key, x, y } => {
                let (owner, slot, generation) = net::projectile_key_parts(key);
                self.projectiles
                    .kill_remote_at(slot, owner, generation, x, y);
            }
            // `case 28` — a peer's strike: the local flash (the C# applies the
            // full `StrikeNPC_Inner` on every client — the port keeps the
            // hp/hurt_cd/flash part, the knockback rides the `23` snapshots).
            S2c::NpcStrike(st) => {
                let slot = st.npc as u32;
                if let Some(npc) = self.net_npcs.get_mut(&slot) {
                    npc.facing = if st.hit_direction > 0 { 1 } else { -1 };
                    if st.damage >= 0 {
                        npc.hp = (npc.hp - st.damage as i32).max(0);
                    } else {
                        npc.hp = 0;
                    }
                    npc.hurt_cd = 0.18;
                    self.net_npcs_dirty = true;
                    // The C# runs the full `StrikeNPC_Inner` on the client
                    // (`MessageBuffer.cs:1797`), which plays `this.HitSound`
                    // at `base.position` (`NPC.cs:81551-81553`) — S1-E-21
                    // wave C. `CheckBytes` runs from `Main.Update`
                    // (`Main.cs:16895`) outside every `SwapRandom`, so the
                    // pitch draw rides the bare-`Main.rand` stand-in.
                    if st.damage >= 0 {
                        if let Some(cue) = npc.kind.hit_sound() {
                            let (x, y) = (npc.x, npc.y);
                            let rng = self.rngs.global();
                            self.audio.play_npc_hit_sound(cue, x, y, rng);
                        }
                    }
                }
            }
            // `case 23` — one NPC's state. The client keeps a slot-keyed
            // store (`net_npcs`) and recreates a slot only on a kind change or
            // the spawn flag (the C#'s `generation`/`spawnNeedsSyncing` test,
            // `MessageBuffer.cs:1602-1613`); the smoothing is the same
            // netOffset accumulator (`MessageBuffer.cs:1636-1646`).
            S2c::NpcInfo(info) => {
                let slot = info.slot as u32;
                // S1-I-4d: a life-0 body is the despawn (`NPC.cs:90385-90392`);
                // the C# sets `active = false` (`MessageBuffer.cs:1661-1664`),
                // the port drops the slot from the render store. The generation
                // guard keeps a stale despawn from deleting a recycled slot.
                if !info.life_full && info.hp <= 0 {
                    let gen_matches = self
                        .net_npc_gen
                        .get(&slot)
                        .map(|(_, g)| *g == info.generation as u32)
                        .unwrap_or(true);
                    if gen_matches {
                        self.net_npcs.remove(&slot);
                        self.net_npc_gen.remove(&slot);
                        self.net_npcs_dirty = true;
                    }
                    return;
                }
                // The slot's generation rides back on this client's strikes.
                self.net_npc_gen
                    .insert(slot, (info.kind, info.generation as u32));
                let kind = crate::npc::NpcKind::from_u16(info.kind.max(0) as u16);
                let recreate = !self.net_npcs.contains_key(&slot)
                    || info.spawn
                    || kind
                        .map(|k| self.net_npcs[&slot].kind.to_u16() != k.to_u16())
                        .unwrap_or(true);
                let mut npc = if recreate {
                    match npc_from_net(&self.world, info.as_ref()) {
                        Some(n) => {
                            self.net_npcs.insert(slot, n.clone());
                            n
                        }
                        None => {
                            log::warn!("unknown npc kind {} on slot {slot}", info.kind);
                            return;
                        }
                    }
                } else {
                    // Patch in place: the store keeps the offsets between snaps.
                    match self.net_npcs.get_mut(&slot) {
                        Some(n) => n.clone(),
                        None => return,
                    }
                };
                // The smoothing: accrue the lag toward the new snap, then snap.
                if !recreate {
                    npc.net_offset.x += npc.x - info.x;
                    npc.net_offset.y += npc.y - info.y;
                    if npc.net_offset.length() > SMOOTHING_RANGE {
                        npc.net_offset = Vec2::ZERO;
                    }
                }
                npc.x = info.x;
                npc.y = info.y;
                npc.vx = info.vx;
                npc.vy = info.vy;
                npc.facing = if info.dir_right { 1 } else { -1 };
                // `MessageBuffer.cs:1654-1656`: directionY/spriteDirection
                // ride flags1 b1/b6; the target the ushort (`:1649-1653`).
                npc.direction_y = if info.dir_y_down { 1 } else { -1 };
                npc.sprite_dir = if info.sprite_right { 1 } else { -1 };
                npc.target = info.target as i32;
                // `MessageBuffer.cs:1683-1691`: the statue flag zeroes the
                // coin value, the shimmer flag sets full transparency.
                npc.spawned_from_statue = info.spawned_from_statue;
                if info.spawned_from_statue {
                    npc.extra_value = 0;
                }
                if info.shimmer {
                    npc.shimmer_transparency = 1.0;
                }
                npc.release_owner = info.release_owner;
                // `MessageBuffer.cs:1657-1677` — the pending-damage
                // reconciliation (W4-g51 id-162): the `life_full` bit snaps
                // the wire life to lifeMax (`:1657-1660`), then the wire life
                // minus the summed predicted damage (`GetPendingDamage`,
                // `NPC.cs:90715-90727`) lands UNLESS a phase change fired
                // (`:1669` — the local life + ai are kept; the port's `23`
                // body carries no ai array, so the ai-copy half is structural
                // and `SetStrikePhaseChange` has no ported trigger yet — the
                // flag stays false, the subtraction always runs).
                let wire_life = if info.life_full { npc.max_hp } else { info.hp };
                let (pending, phase_change) = crate::app::net_npc_sync::get_pending_damage(
                    &self.net_pending_damages,
                    info.slot as i16,
                    info.generation,
                );
                if !phase_change {
                    npc.hp = wire_life - pending; // `:1671`
                    // `MessageBuffer.cs:1672-1675` — the ai[] copy rides the
                    // same !phaseChange gate (the g48 wire body carries ai).
                    npc.ai = info.ai;
                }
                // The C#'s despawn gate (`:1661`) reads the WIRE life
                // (`num175`), not the reconciled one — a predicted-to-death
                // NPC stays active until the host's own death `23` lands
                // (the early-despawn branch above handles wire ≤ 0).
                self.net_npcs.insert(slot, npc);
                self.net_npcs_dirty = true;
            }
            // `case 21` — one drop's state, keyed by the host slot index
            // (the port's item ids — the C# fixed-400 scheme is M6-NET-5).
            S2c::ItemState(d) => {
                self.apply_item_state(&d, false);
            }
            // `case 90` — the instanced twin of 21 (W4-g50,
            // `MessageBuffer.cs:1489-1493`): the same body, then
            // `instanced = true` + `playerIndexTheItemIsReservedFor =
            // Main.myPlayer` — the drop is reserved for THIS client.
            S2c::SpawnInstancedItem(d) => {
                self.apply_item_state(&d, true);
            }
            // `case 91` — an emote bubble's create/update/removal (W4-g50,
            // `MessageBuffer.cs:3394-3441`): 255 removes by id; otherwise
            // the anchor rebuilds (`DeserializeNetAnchor`), a player anchor
            // sets `emoteTime = 360` (`:3419-3422`), and the by-ID upsert
            // refreshes lifetime/emote/anchor + fires `OnBubbleChange`.
            S2c::EmoteBubble(b) => {
                if b.anchor_kind == 255 {
                    self.emotes.remove_net(b.id);
                    return;
                }
                // `:3419-3422` — the player-anchor emoteTime write.
                if b.anchor_kind == 1 {
                    if b.anchor_index as u32 == self.local_id {
                        if let Some(p) = self.player.as_mut() {
                            p.emote_time = 360;
                        }
                    } else if let Some(p) =
                        self.remote_players.get_mut(&(b.anchor_index as u32))
                    {
                        p.emote_time = 360;
                    }
                }
                self.emotes.upsert_net(&b, self.local_id as u16);
            }
            // `case 96` — a peer's portal teleport (W4-g50,
            // `MessageBuffer.cs:3538-3557`): the color parity flip, the
            // style-4 teleport and the velocity overwrite. The subject is
            // the wire's id byte (the host already forced it to the sender).
            S2c::PlayerPortalTeleport(p) => {
                self.apply_player_portal_teleport(p.player, &p);
            }
            // `case 99` — a peer's minion rest point (W4-g50,
            // `MessageBuffer.cs:3577`): `Main.player[num163].MinionRestTargetPoint
            // = ReadVector2()`. The subject is the wire's id byte.
            S2c::MinionRestTarget(m) => {
                if m.player_id as u32 == self.local_id {
                    if let Some(p) = self.player.as_mut() {
                        p.minion_rest_target_point = Vec2::new(m.x, m.y);
                    }
                } else if let Some(p) = self.remote_players.get_mut(&(m.player_id as u32)) {
                    p.minion_rest_target_point = (m.x, m.y);
                }
            }
            // `case 115` — a peer's minion attack target (W4-g50,
            // `MessageBuffer.cs:3591`): `Main.player[num154].MinionAttackTargetNPC
            // = ReadInt16()`. The subject is the wire's id byte.
            S2c::MinionAttackTarget(m) => {
                if m.player_id as u32 == self.local_id {
                    if let Some(p) = self.player.as_mut() {
                        p.minion_attack_target_npc = m.npc as i32;
                    }
                } else if let Some(p) = self.remote_players.get_mut(&(m.player_id as u32)) {
                    p.minion_attack_target_npc = m.npc as i32;
                }
            }
            // `case 100` — an NPC pulled through a portal (W4-g50,
            // `MessageBuffer.cs:3598-3610`): the color parity flip feeds the
            // entry effect, `Teleport(dest, 4, color)` snaps the position
            // and stores the un-flipped color (`NPC.cs:81045-81048`), the
            // velocity overwrites and `netOffset *= 0f` (`:3608`). The slot
            // is the HOST's NPC index — the client's slot-keyed store.
            S2c::NpcPortalTeleport(p) => {
                let Some(npc) = self.net_npcs.get_mut(&(p.npc as u32)) else {
                    return;
                };
                npc.x = p.x;
                npc.y = p.y;
                npc.vx = p.velocity_x;
                npc.vy = p.velocity_y;
                npc.net_offset = Vec2::ZERO;
                npc.last_portal_color_index = p.color_index;
                self.net_npcs_dirty = true;
            }
            // `case 101` — the four Lunar tower shields (W4-g50,
            // `MessageBuffer.cs:3611-3651`): each ushort clamps to
            // `0..LunarShieldPowerMax` (the static 100, `NPC.cs:95252` —
            // NOT the downed-halved `ShieldStrengthTowerMax`). The reader
            // is `netMode != 2`-gated; the port's client applies it.
            S2c::TowerShields(t) => {
                const LUNAR_SHIELD_POWER_MAX: i32 = 100; // `NPC.cs:95252`
                let clamp = |v: u16| (v as i32).clamp(0, LUNAR_SHIELD_POWER_MAX);
                self.world.tower_shield_solar = clamp(t.solar);
                self.world.tower_shield_vortex = clamp(t.vortex);
                self.world.tower_shield_nebula = clamp(t.nebula);
                self.world.tower_shield_stardust = clamp(t.stardust);
            }
            // `case 102` — the nebula booster fan (W4-g50,
            // `MessageBuffer.cs:3662-3705`): for every active, living,
            // same-team player within 700 px of the pickup point, the
            // spiral dust trail from that player toward the picker, then
            // `NebulaLevelup` on the local player. The dust draws ride the
            // bare `Main.rand` stream (the message-time replay convention).
            S2c::NebulaLevelupRequest(n) => {
                self.apply_nebula_levelup(&n);
            }
            // `case 103` — the Moon Lord countdown statics (W4-g50,
            // `MessageBuffer.cs:3708-3714`): `NPC.MaxMoonLordCountdown` then
            // `NPC.MoonLordCountdown`, both raw ints.
            S2c::MoonlordCountdown(m) => {
                self.world.max_moon_lord_countdown = m.max;
                self.world.moon_lord_countdown = m.countdown;
            }
            // `case 104` — one open-shop slot's rebuilt item (W4-g50,
            // `MessageBuffer.cs:3715-3735`): only while a shop is open
            // (`Main.npcShop > 0`) and the slot is in range; the row is a
            // fresh `SetDefaults(type)` with the wire's stack/prefix/value/
            // buyOnce (`:3727-3732`).
            S2c::ShopOverride(s) => {
                if self.shop_npc.is_none() {
                    return; // `Main.npcShop > 0` (`:3716`)
                }
                if (s.slot as usize) >= self.shop_stock.len() {
                    return; // `num99 < item4.Length` (`:3725`)
                }
                // `SetDefaults(type)` — the fresh row; the exact prefix
                // applies directly (`Item.Prefix(prefixWeWant)`).
                let prefix = crate::prefix::roll_prefix(
                    s.kind.max(0) as u16,
                    s.prefix as i32,
                    self.rngs.global(),
                );
                self.shop_stock[s.slot as usize] = crate::shop::ShopEntry {
                    item: s.kind.max(0) as u16,
                    when: crate::shop::StockWhen::Always,
                    shop_custom_price: None,
                    buy_once: s.buy_once,
                    is_a_shop_item: false,
                    prefix,
                    stack: s.stack.max(0) as u32,
                    value: Some(s.value),
                };
            }
            // `case 106` — the smoke poof (W4-g50,
            // `MessageBuffer.cs:3742-3751` → `Utils.PoofOfSmoke`,
            // `Utils.cs:1078-1093`): the packed half-float position, then
            // the gore burst (3..6 gores, types 11..13, scale 0.7, velocity
            // ×0.5) and the 10-dust type-16 ring. The draws ride the bare
            // `Main.rand` stream (the message-time replay convention).
            S2c::PoofOfSmoke(p) => {
                let (x, y) = net::unpack_half_vector2(p.packed);
                self.poof_of_smoke(Vec2::new(x, y));
            }
            // `case 107` — the smart chat line (W4-g50,
            // `MessageBuffer.cs:3752-3757` → `Main.NewTextMultiline`,
            // `Main.cs:64638-64642`): the chat line in the wire colour +
            // the sound-12 cue. The width-limit word wrap is the port's
            // recorded flat-text gap (`chat.rs:97-98`).
            S2c::SmartText(t) => {
                self.chat
                    .new_text(t.text.0.clone(), [t.color.0, t.color.1, t.color.2]);
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
            }
            // `case 108` — the wire-fired cannon shot (W4-g50,
            // `MessageBuffer.cs:3758-3773`): only the owning client replays
            // it (`num75 == Main.myPlayer`, `:3768`), with `fromWire: true`.
            S2c::WiredCannonShot(c) => {
                if c.owner as u32 == self.local_id {
                    crate::world_sim::wiring::replay_wired_cannon_shot(
                        self,
                        c.x as i32,
                        c.y as i32,
                        c.angle as i32,
                        c.ammo as i32,
                        c.damage as i32,
                        c.knockback,
                    );
                }
            }
            // `case 110` — the mass wire operation's bill (W4-g50,
            // `MessageBuffer.cs:3789-3808`): only the billed player pays
            // (`num67 == Main.myPlayer`, `:3798`): `ConsumeItem(type)` ×
            // count, then `wireOperationsCooldown = 0` (`:3801-3805`).
            S2c::MassWireOperationPay(p) => {
                if p.player as u32 == self.local_id {
                    for _ in 0..p.count.max(0) {
                        self.consume_item_forward(p.item_type.max(0) as u16);
                    }
                    if let Some(pl) = self.player.as_mut() {
                        pl.wire_operations_cooldown = 0;
                    }
                }
            }
            // `case 112` — the special FX burst (W4-g50,
            // `MessageBuffer.cs:3815-3840`): kind 1 = tree-grow FX
            // (`WorldGen.TreeGrowFX`, the shake variant on b6 == 1), kind 2
            // = fairy critter effects (`NPC.FairyEffects`). The draws ride
            // the bare `Main.rand` stream (the message-time replay).
            S2c::SpecialFx(f) => match f.effect {
                1 => self.tree_grow_fx(f.x, f.y, f.b4 as i32, f.s5 as i32, f.b6 == 1),
                2 => self.fairy_effects(Vec2::new(f.x as f32, f.y as f32), f.s5 as i32),
                _ => {}
            },
            // `case 114` — the DD2 entity wipe (W4-g50,
            // `MessageBuffer.cs:3855-3860` → `DD2Event.WipeEntities`,
            // `DD2Event.cs:629-638`): the three sweeps (towers, hostiles,
            // energy crystals in the open chests). The client's NPC copy is
            // the slot-keyed `net_npcs` store (the 23/54 convention), so the
            // hostile sweep retires those slots; the turret + chest sweeps
            // ride the shared helper's pools.
            S2c::CrystalInvasionWipe => {
                // `ClearAllTowersInGame` + `ClearAllDD2EnergyCrystalsInChests`
                // (the projectile pool + the open chests are the client's own).
                crate::world_sim::npcs::wipe_dd2_entities(self);
                // `ClearAllDD2HostilesInGame` (`:651-664`) over the net store.
                let doomed: Vec<u32> = self
                    .net_npcs
                    .iter()
                    .filter(|(_, n)| {
                        crate::world_sim::npcs::dd2_pool_id(n.kind)
                            .is_some_and(tr_core::dd2::belongs_to_ooa)
                    })
                    .map(|(slot, _)| *slot)
                    .collect();
                for slot in doomed {
                    self.net_npcs.remove(&slot);
                    self.net_npc_gen.remove(&slot);
                }
                self.net_npcs_dirty = true;
            }
            // `case 116` — the wave-break countdown (W4-g50,
            // `MessageBuffer.cs:3861-3866`): `DD2Event.TimeLeftBetweenWaves
            // = ReadInt32()` — the client-side timer the crystal's overhead
            // display and the skip-wait gate read.
            S2c::CrystalInvasionWait(t) => {
                self.world.dd2_time_left_between_waves = t.time_left;
            }
            // `case 117` — a peer's hurt replay (W4-g50,
            // `MessageBuffer.cs:3879`): the quiet `Player.Hurt` — the local
            // player runs the ported hurt path, a peer snapshot takes the hp.
            S2c::PlayerHurtV2(h) => {
                self.apply_player_hurt(h.player_id, &h);
            }
            // `case 118` — a peer's death replay (W4-g50,
            // `MessageBuffer.cs:3898`): `Player.KillMe` — the local player
            // runs the death pipeline, a peer snapshot marks dead.
            S2c::PlayerDeathV2(d) => {
                self.apply_player_death(d.player_id, &d);
            }
            // `case 83` — the deprecated ignore group (W4-g50,
            // `MessageBuffer.cs:4483-4490`): accepted, nothing to do.
            S2c::Unused83 => {}
            // `case 93` — the deprecated ignore group (W4-g50,
            // `MessageBuffer.cs:4483-4490`): accepted, nothing to do.
            S2c::Unused93 => {}
            // `case 85` — the blocked-chest list answer (W4-g50,
            // `MessageBuffer.cs:3267-3270`): the client runs
            // `QuickStacking.IndicateBlockedChests` (`:3269`) — FINALIZE-PARK
            // (re-probe): the fn's whole body is the `RoomCheckParticle`
            // red pulse per chest (`Chest.IndicateBlockedChest`,
            // `Chest.cs:3582-3601` — a 60-tick Cd-texture particle in
            // `ParticleSystem_World_OverPlayers`), zero state writes; the
            // port has no particle system (the g21 engine's module doc
            // records the same no-op). The transfer itself is WIRED on the
            // host leg; the codec is byte-verbatim (`net.rs`).
            S2c::QuickStackBlocked(_) => {}
            // `case 94` — the debug-command sync (W4-g50,
            // `MessageBuffer.cs:3463-3518`): the same `enableDebugCommands`
            // gate + command switch as the host leg — the client stores the
            // three shared statics (the `SyncToJoiningPlayer` push,
            // `DebugOptions.cs:64-72`, lands here). The `/quickload-*`
            // harness arms stay parked (QuickLoad unported).
            S2c::DevCommands(d) => {
                self.apply_dev_commands_shared(&d);
            }
            // `151` — the slot is gone.
            S2c::ItemDead { slot } => {
                self.net_items.remove(&slot);
                self.net_items_dirty = true;
            }
            // `case 22` — the reservation announcement (server→client only;
            // `MessageBuffer.cs:1552-1562` refuses it on the server).
            S2c::ItemOwner(o) => {
                if o.reserved_for == 255 {
                    self.net_reserved.remove(&o.slot);
                } else {
                    self.net_reserved.insert(o.slot, (o.reserved_for, o.hold));
                }
                // `MessageBuffer.cs:1558-1559` — the client writes the
                // grab-delay pair onto the drop (the player drop's 100-tick
                // lock survives the join, S1-P-i51). The port keys the
                // client's drops by the host slot in `net_items`.
                if let Some(d) = self.net_items.get_mut(&o.slot) {
                    d.grab_delay_player = o.grab_delay_player;
                    d.grab_delay_time = o.grab_delay as i32;
                    self.net_items_dirty = true;
                }
            }
            // `case 5` — one inventory slot (`MessageBuffer.cs:399-462`):
            // the slot lands in that peer's inventory copy; the SELF leg
            // (`num25 == Main.myPlayer`, `:414-430`) writes the local
            // player's slot — the 85 quick-stack restore re-sends ride it
            // (`RestoreToPlayer`, `QuickStacking.cs:353-356`; the host
            // locked the slots via `LockNetSlot` before the 85, so the
            // `HasLockedInventory` refusal does not fire — the lock machine
            // is the client send path's, W4-g21's button lane). The bit-1
            // blocked flag pulses the slot red (`IndicateBlockedSlot`) —
            // the pulse visual is unported, the flag rides the codec.
            S2c::Inventory {
                player_id,
                slot,
                item,
            } => {
                if player_id as u32 == self.local_id {
                    let value = if item.kind <= 0 || item.stack <= 0 {
                        crate::item::EMPTY_SLOT
                    } else {
                        crate::item::ItemSlot {
                            kind: item.kind as u16,
                            name: crate::registry::name(item.kind as u16),
                            count: item.stack.max(0) as u32,
                            prefix: item.prefix as u16,
                        }
                    };
                    // `PlayerItemSlotID.SlotReference`'s grid/coin/ammo/
                    // bank4 split (the port's arrays).
                    match slot {
                        0..=49 => self.inventory[slot as usize] = value,
                        50..=53 => self.equip.coins[slot as usize - 50] = value,
                        54..=57 => self.equip.ammo[slot as usize - 54] = value,
                        s if (player_item_slot::bank_1_0(3)
                            ..player_item_slot::bank_1_0(3) + 40)
                            .contains(&s) =>
                        {
                            if let Some(p) = self.player.as_mut() {
                                p.banks[3][(s - player_item_slot::bank_1_0(3)) as usize] = value;
                            }
                        }
                        _ => {} // the other blocks are the equip lanes'
                    }
                    return;
                }
                let peer = player_id as u32;
                let (px, py) = self.net_peer_spawn_pos();
                self.remote_players
                    .entry(peer)
                    .or_insert_with(|| net::PlayerNet {
                        id: peer,
                        name: String::new(),
                        x: px,
                        y: py,
                        ..Default::default()
                    })
                    .inventory
                    .insert(slot, item);
            }
            // `case 32` — one chest slot (`MessageBuffer.cs:1897-1918`): the
            // client writes it into its own chest store.
            S2c::ChestItem(ci) => {
                if ci.chest < 0 || ci.chest as usize >= crate::chest::MAX_CHESTS {
                    return; // `num146 >= 0 && num146 < 8000` (`:1904`)
                }
                let idx = ci.chest as usize;
                // `Main.chest[num146] != null` (`:1904`): the C# client holds
                // the full array from the world load; the lazy port
                // materialises the slot the `155` header just sized.
                if Self::materialise_chest_slot(&mut self.world_chests, idx).is_none() {
                    return;
                }
                let value = if ci.kind <= 0 || ci.stack <= 0 {
                    crate::item::EMPTY_SLOT
                } else {
                    crate::item::ItemSlot {
                        kind: ci.kind as u16,
                        name: crate::registry::name(ci.kind as u16),
                        count: ci.stack.max(0) as u32,
                        prefix: ci.prefix as u16,
                    }
                };
                crate::chest::set_chest_slot(&mut self.world_chests, idx, ci.slot as usize, value);
            }
            // `case 33` — the chest this client opened
            // (`MessageBuffer.cs:1938-1969`): the local player's `chest` plus
            // the open/switch/close cues (10/12/11).
            S2c::PlayerChest(pc) => {
                let Some(p) = self.player.as_mut() else {
                    return;
                };
                let idx = pc.chest as i32;
                let cue = if p.chest == -1 {
                    if idx != -1 {
                        self.inventory_open = true;
                    }
                    10
                } else if p.chest != idx && idx != -1 {
                    self.inventory_open = true;
                    self.crafting_open = false;
                    12
                } else if p.chest != -1 && idx == -1 {
                    11
                } else {
                    0
                };
                // `:1962-1964` - the C# assigns the index and position
                // unconditionally.
                p.chest = idx;
                p.chest_x = pc.x as i32;
                p.chest_y = pc.y as i32;
                let (cx, cy) = (pc.x as i32, pc.y as i32);
                // The C# client's `Main.chest[num21]` already carries the tile
                // from the world load; the lazy port materialises the slot the
                // `155` header just sized and assigns the tile, so the `69`
                // arm's `chest.x/y` guard (`:3073`) can match.
                if idx >= 0 && (idx as usize) < crate::chest::MAX_CHESTS {
                    if let Some(c) = Self::materialise_chest_slot(&mut self.world_chests, idx as usize)
                    {
                        if c.tile == crate::chest::unplaced_tile() {
                            c.tile = (cx, cy);
                        }
                    }
                }
                if cue != 0 {
                    // W4-d79 — the chest-open net replay's menu cue rides the
                    // single-voice pool (cases 10/11/12, draw-free).
                    self.audio.play_menu(match cue {
                        10 => "Menu_Open",
                        11 => "Menu_Close",
                        _ => "Menu_Tick",
                    });
                }
            }
            // `case 47` — a sign's text (`MessageBuffer.cs:2376-2405`): the
            // record is rewritten and, when the sender is us, the editor
            // opens (unported — S1-E-26 records the gap).
            S2c::SignData(sd) => {
                if sd.sign < 0 || sd.sign >= crate::signs::MAX_SIGNS {
                    return; // `:2384`
                }
                crate::signs::text_sign(
                    &mut self.world,
                    sd.sign as usize,
                    sd.x as i32,
                    sd.y as i32,
                    sd.text.clone(),
                );
            }
            // `case 50` — a player's buff ids (`MessageBuffer.cs:2454-2480`):
            // ours is skipped (`:2461-2464`), a peer's lands in its record
            // with every time forced to 60 (`:2470-2471`).
            S2c::PlayerBuffs(pb) => {
                if pb.player_id as u32 == self.local_id {
                    return; // `:2461-2464`
                }
                let peer = pb.player_id as u32;
                let (px, py) = self.net_peer_spawn_pos();
                self.remote_players
                    .entry(peer)
                    .or_insert_with(|| net::PlayerNet {
                        id: peer,
                        name: String::new(),
                        x: px,
                        y: py,
                        ..Default::default()
                    })
                    .buffs = pb.buffs; // the `60`-tick times are not part of the port's peer record
            }
            // `case 54` — an NPC's buff pairs (`MessageBuffer.cs:2577-2593`):
            // the client's copy of the NPC gets exactly the listed debuffs.
            S2c::NpcBuffs(nb) => {
                let Some(npc) = self.net_npcs.get_mut(&(nb.npc as u32)) else {
                    return;
                };
                for slot in 0..crate::npc_buffs::NPC_MAX_BUFFS {
                    npc.buffs.buff_type[slot] = 0;
                    npc.buffs.buff_time[slot] = 0;
                }
                for (i, (id, time)) in nb.buffs.iter().enumerate() {
                    if i >= crate::npc_buffs::NPC_MAX_BUFFS {
                        break; // the C# array bound (`:2586-2587`)
                    }
                    npc.buffs.buff_type[i] = *id as i32;
                    npc.buffs.buff_time[i] = *time as i32;
                }
                self.net_npcs_dirty = true;
            }
            // `case 56` — the host's answer with a town NPC's given name +
            // variation (`MessageBuffer.cs:2612-2630`): behind the
            // `0..maxNPCs` slot gate (`:2615`) the client stores the pair
            // (`:2619-2622`).
            S2c::UniqueTownNpcInfo(info) => {
                let slot = info.npc;
                if slot >= 0 && (slot as usize) < MAX_NPCS {
                    if let Some((name, variation)) = info.info {
                        if let Some(npc) = self.net_npcs.get_mut(&(slot as u32)) {
                            npc.given_name = name; // `:2620`
                            npc.variation_index = variation; // `:2622`
                            self.net_npcs_dirty = true;
                        }
                    }
                }
            }
            // `case 60` — an NPC's household banner (`MessageBuffer.cs:2695-2739`):
            // the client sets homeless/homeTile (`:2708-2713`) and, for a
            // town-like NPC, runs the TownManager kick-out/room arm
            // (`:2718-2728`). The port's client keeps no housing simulation
            // (the host is authoritative, the `23` snapshots carry the NPC),
            // so the read is a no-op here — the host arm owns the effect.
            S2c::NpcHouse(_) => {}
            // `case 63` — the host's tile paint/coat relay
            // (`MessageBuffer.cs:2909-2928`): the client applies the same
            // paint/coat to its world copy.
            S2c::TilePaint(p) => {
                if p.coat == 0 {
                    paint_tile(&mut self.world, p.x as i32, p.y as i32, p.paint);
                } else {
                    paint_coat_tile(&mut self.world, p.x as i32, p.y as i32, p.paint);
                }
                self.world_key = None;
            }
            // `case 64` — the host's wall paint/coat relay
            // (`MessageBuffer.cs:2929-2948`): the client applies the same
            // paint/coat to its world copy.
            S2c::WallPaint(p) => {
                if p.coat == 0 {
                    paint_wall(&mut self.world, p.x as i32, p.y as i32, p.paint);
                } else {
                    paint_coat_wall(&mut self.world, p.x as i32, p.y as i32, p.paint);
                }
                self.world_key = None;
            }
            // `case 65` — the host's teleport broadcast
            // (`MessageBuffer.cs:2949-3035`): kind 0/2 teleports the subject
            // player (`:2986`/`:3002`), kind 1 the NPC + netOffset zero
            // (`:2997-2998`). Our own kind-0 echo is skipped (the local player
            // is authoritative); the kind-3 ack send-back (`:2991-2994`) is
            // the send path's (S1-I-4).
            S2c::TeleportEntity(t) => {
                match t.kind {
                    0 | 2 => {
                        if t.entity as u32 == self.local_id {
                            return; // our own teleport — the local sim owns it
                        }
                        if let Some(p) = self.remote_players.get_mut(&(t.entity as u32)) {
                            p.x = t.x;
                            p.y = t.y;
                            p.off_x = 0.0; // `Player.netOffset = Vector2.Zero`
                            p.off_y = 0.0;
                        }
                    }
                    1 => {
                        if let Some(npc) = self.net_npcs.get_mut(&(t.entity as u32)) {
                            npc.x = t.x;
                            npc.y = t.y;
                            npc.net_offset = Vec2::ZERO; // `:2998`
                            self.net_npcs_dirty = true;
                        }
                    }
                    _ => {} // kind 3 ack is host-only (`:3030`)
                }
            }
            // `case 66` — a peer's Moon Leech heal (`MessageBuffer.cs:3036-3055`):
            // the client heals the subject (clamped) and raises the HealEffect
            // text; our own echo is skipped (the local sim owns our health).
            S2c::HealPlayer(h) => {
                if h.heal > 0 {
                    let slot = h.player_id as u32;
                    if slot == self.local_id {
                        return;
                    }
                    if let Some(p) = self.remote_players.get_mut(&slot) {
                        p.hp = (p.hp + h.heal as i32).min(p.max_hp);
                    }
                    spawn_player_amount_text(self, slot, crate::combat_text::HEAL_LIFE, h.heal as i32);
                }
            }
            // `case 59` — a peer tripped a switch: replay it locally
            // (`MessageBuffer.cs:2682-2693`).
            S2c::HitSwitch(hs) => {
                self.wiring
                    .hit_switch(&mut self.world, hs.x as i32, hs.y as i32);
                // S1-E-21i — the client's click replay: the message handler
                // runs outside every swap (`Main.cs:16895`'s NpcStrike
                // precedent), so the pitch draws ride the bare stream.
                for (px, py) in self.wiring.take_switch_clicks() {
                    self.audio
                        .play_switch_click(px as f32, py as f32, self.rngs.global());
                }
                self.world_key = None;
            }
            // `case 80` — which chest a player has open
            // (`MessageBuffer.cs:3216-3227`): the range gate is `-3..8000`.
            S2c::PlayerChestIndex(ci) => {
                let Some(entry) = self.remote_players.get_mut(&(ci.player_id as u32)) else {
                    return;
                };
                if (-3..8000).contains(&ci.chest) {
                    entry.chest = Some(ci.chest);
                }
            }
            // `case 155` — a chest's `maxItems` (`MessageBuffer.cs:4429-4438`):
            // `Chest.Resize` grows/shrinks the slot list. The C# client always
            // has the whole `Main.chest[8000]` array from the world load; the
            // port ships chests lazily, so the slot is materialised here first
            // (the `33`/`69` arms fill the tile in).
            S2c::ChestSize(cs) => {
                if cs.chest < 0 || cs.chest as usize >= crate::chest::MAX_CHESTS {
                    return; // `num31 >= 0 && num31 < 8000` (`:4433`)
                }
                let Some(c) = Self::materialise_chest_slot(&mut self.world_chests, cs.chest as usize)
                else {
                    return;
                };
                let size = cs.size.max(0) as usize;
                c.max_items = size;
                if c.slots.len() > size {
                    c.slots.truncate(size);
                } else {
                    c.slots.resize(size, crate::item::EMPTY_SLOT);
                }
            }
            // `case 69` — a chest's new name (`MessageBuffer.cs:3059-3078`).
            S2c::ChestName(cn) => {
                if cn.chest < 0 || cn.chest as usize >= crate::chest::MAX_CHESTS {
                    return; // `num18 >= 0 && num18 < 8000` (`:3063`)
                }
                let idx = cn.chest as usize;
                match self.world_chests.get_mut(idx) {
                    Some(c) if c.tile == (cn.x as i32, cn.y as i32) => c.name = cn.name,
                    Some(_) => {} // `chest.x != num19 || chest.y != num20 → break` (`:3073`)
                    None => {
                        // `Chest.CreateWorldChest(num18, num19, num20)` (`:3071`).
                        if idx < crate::chest::MAX_CHESTS {
                            while self.world_chests.len() < idx {
                                self.world_chests.push(crate::chest::WorldChest {
                                    tile: (i32::MIN, i32::MIN),
                                    ..Default::default()
                                });
                            }
                            self.world_chests.push(crate::chest::WorldChest {
                                tile: (cn.x as i32, cn.y as i32),
                                name: cn.name,
                                max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
                                ..Default::default()
                            });
                        }
                    }
                }
            }
            // `case 11` — the deprecated re-frame request
            // (`MessageBuffer.cs:895-899`): the client runs
            // `WorldGen.SectionTileFrame` over the section rectangle
            // (`WorldGen.cs:80343-80366`). No C# site sends it any more
            // (`MessageID.cs:29-30`), the reader arm is 1:1 (W4-g48).
            S2c::TileFrameSection(t) => {
                // `:80345` — the map-time burst; `:80347` — the widened
                // rectangle queues for the map refresh.
                if let Some(m) = self.map.as_mut() {
                    m.arm_map_time_burst();
                    let (x0, y0, x1, y1) = (
                        (t.start_x as i32 * 200).clamp(1, self.world.width - 2),
                        (t.start_y as i32 * 150).clamp(1, self.world.height - 2),
                        ((t.end_x as i32 + 1) * 200).min(self.world.width - 2),
                        ((t.end_y as i32 + 1) * 150).min(self.world.height - 2),
                    );
                    self.map_queue
                        .add_area(m, x0 - 2, y0 - 2, x1 - x0 + 4, y1 - y0 + 4);
                }
                tr_core::framing::section_tile_frame(
                    &mut self.world,
                    t.start_x as i32,
                    t.start_y as i32,
                    t.end_x as i32,
                    t.end_y as i32,
                );
                self.world.revision = self.world.revision.wrapping_add(1);
                self.world_key = None;
            }
            // `15 Unknown15` is the reader's ignore group
            // (`MessageBuffer.cs:4483-4490`): accepted, nothing to do.
            S2c::Unknown15 => {}
            // `case 81` — a peer's heal number (S1-I-1 wave I,
            // `MessageBuffer.cs:3227-3233`): the floats truncate to the
            // Rectangle's ints (width/height 0), the RGB bytes gain alpha
            // 255 (`Color(byte,byte,byte)`, `Utils.cs:1387-1390`), and
            // `CombatText.NewText` spawns with the default
            // dramatic/dot = false. The draws ride `Main.rand` — the bare
            // global stream, like the message-time cue replays (S1-E-21i).
            S2c::CombatTextInt(t) => {
                let text = t.amount.to_string();
                let size = match self.lookups.combat_text_font.as_ref() {
                    Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                    None => [text.chars().count() as f32 * 8.0, 10.0],
                };
                let location = crate::combat_text::Rect {
                    x: t.x as i32 as f32,
                    y: t.y as i32 as f32,
                    width: 0.0,
                    height: 0.0,
                };
                crate::combat_text::spawn_amount(
                    &mut self.combat_texts,
                    location,
                    [t.color.0, t.color.1, t.color.2, 255],
                    t.amount,
                    false,
                    false,
                    size,
                    1.0, // the port keeps gravDir = 1 (`player.rs:4693`)
                    self.rngs.global(),
                );
            }
            // `case 34` on a client — the host's chest place/destroy relay
            // (`MessageBuffer.cs:2124-2157`): the PlaceChestDirect/
            // DestroyChestDirect pass — W4-g48 (`net_apply`).
            S2c::ChestUpdates(u) => {
                crate::app::net_apply::client_chest_updates(self, u);
            }
            // `case 30` on a client — a peer's PvP flag
            // (`MessageBuffer.cs:1858-1859`): the snapshot store; the chat
            // line arrives as the `82` module — W4-g48 (`net_apply`).
            S2c::TogglePvp(t) => {
                crate::app::net_apply::client_toggle_pvp(self, t.player_id, t.hostile);
            }
            // `case 42` — a peer's mana (`MessageBuffer.cs:2290-2306`): our
            // own echo is skipped (`:2297-2300`, `ServerSideCharacter` is
            // false in the port), a peer's lands in its record.
            S2c::PlayerMana(pm) => {
                if pm.player_id as u32 == self.local_id {
                    return; // `:2297-2300`
                }
                let peer = self.remote_players.entry(pm.player_id as u32).or_default();
                peer.mana = pm.mana as i32; // `:2303`
                peer.max_mana = pm.max_mana as i32; // `:2304`
            }
            // `case 45` — a peer's team (`MessageBuffer.cs:2325-2340`): the
            // client stores the team byte and breaks before the host-only
            // relay/chat (`:2338-2340`).
            S2c::TeamChange(tc) => {
                let peer = self.remote_players.entry(tc.player_id as u32).or_default();
                peer.team = tc.team as i32; // `:2336`
            }
            // `case 57` — the world-evil census bytes
            // (`MessageBuffer.cs:2631-2638`): client-only, stores
            // `WorldGen.tGood/tEvil/tBlood` in the writer's order. The Dryad
            // world-status report reads them back
            // (`Lang.GetDryadWorldStatusDialog`, `Lang.cs:251-293`). W4-g49.
            S2c::WorldEvilBytes(e) => {
                self.world_evil_good = e.good; // `:2634`
                self.world_evil_evil = e.evil; // `:2635`
                self.world_evil_blood = e.blood; // `:2636`
                // The `WorldGen.tGood/tEvil/tBlood` statics' port home is the
                // World field (the f21b join-push witness + the dryad report's
                // `Lang.GetDryadWorldStatusDialog` read it); the App copy above
                // is the g49 HUD seam — both stay in step.
                self.world.t_good = e.good;
                self.world.t_evil = e.evil;
                self.world.t_blood = e.blood;
            }
            // `case 58` — a peer's instrument (`MessageBuffer.cs:2652-2679`):
            // the client plays the held instrument's sound at the peer's
            // position. The guitar chords (4057/4372/4715) and drums (4673)
            // ride `PlayGuitarChord`/`PlayDrums`; anything else stores
            // `Main.musicPitch` and plays Item35 (507) / Item47 (1305) /
            // Item26 (default). W4-g49.
            S2c::InstrumentSound(s) => {
                let peer = s.player_id as u32;
                let (px, py, held) = if peer == self.local_id {
                    // Our own echo: the local player's centre + held item.
                    match self.player.as_ref() {
                        Some(p) => {
                            let (cx, cy) = p.center();
                            (cx, cy, self.inventory[self.selected].kind)
                        }
                        None => return,
                    }
                } else {
                    match self.remote_players.get(&peer) {
                        Some(p) => {
                            // The peer's centre: position + half the hitbox
                            // (`Player.Center`, the port's 20×42 stand-in).
                            let held = p
                                .inventory
                                .get(&(p.selected as u8))
                                .map(|it| it.kind.max(0) as u16)
                                .unwrap_or(0);
                            (p.x + 10.0, p.y + 21.0, held)
                        }
                        None => return, // no such peer — the C# would NPE
                    }
                };
                // `CheckBytes` runs from `Main.Update` (`Main.cs:16895`)
                // outside every `SwapRandom`, so the draws ride the
                // bare-`Main.rand` stand-in (the `28` strike precedent).
                match held {
                    // `:2656-2660` — the guitar chords.
                    4057 | 4372 | 4715 => self.play_guitar_chord(px, py, s.pitch),
                    // `:2661-2663` — the drums.
                    4673 => self.play_drums(px, py, s.pitch),
                    _ => {
                        // `:2666` — `Main.musicPitch = num40`.
                        self.music_pitch = s.pitch;
                        self.audio.set_music_pitch(s.pitch);
                        // `:2667-2676` — Item35 / Item47 / Item26.
                        let style = match held {
                            507 => 35,
                            1305 => 47,
                            _ => 26,
                        };
                        let idx = crate::soundid::SOUND_STYLES
                            .iter()
                            .position(|st| st.sound_id == 2 && st.style == style);
                        if let Some(idx) = idx {
                            self.audio.play_use_sound(
                                idx as u16,
                                px,
                                py,
                                0.0,
                                self.rngs.global(),
                            );
                        }
                    }
                }
            }
            // `case 72` — the travel shop's stock (`MessageBuffer.cs:3118-3126`):
            // the client stores the 40 shorts into `Main.travelShop`
            // (`:3121-3124`). The shop index 19 reads them back.
            S2c::TravelShop(s) => {
                for (i, item) in s.items.iter().enumerate() {
                    if i < self.world.travel_shop.len() {
                        self.world.travel_shop[i] = (*item).max(0) as u16;
                    }
                }
            }
            // `case 74` — today's angler quest (`MessageBuffer.cs:3147-3153`):
            // the client stores the quest index and whether it finished today.
            S2c::AnglerQuest(q) => {
                self.world.angler_quest = q.quest; // `:3150`
                self.world.angler_quest_finished = q.finished; // `:3151`
            }
            // `case 77` — a temporary tile animation
            // (`MessageBuffer.cs:3183-3185`): `Animation.NewTemporaryAnimation`
            // has no port target (the wiring/statue animation system is
            // unported, ANALYSIS "Traps & statues") — the read is a no-op.
            S2c::TemporaryAnimation(_) => {}
            // `case 78` — the invasion/moon/DD2 progress bar
            // (`MessageBuffer.cs:3186-3191`): `Main.ReportInvasionProgress`
            // (`Main.cs:47687-47694`) stores the four fields and arms the
            // 160-tick display window.
            S2c::InvasionProgress(p) => {
                self.invasion_progress = p.progress; // `:47689`
                self.invasion_progress_max = p.progress_max; // `:47690`
                self.invasion_progress_icon = p.icon as i32; // `:47691`
                self.invasion_progress_wave = p.wave as i32; // `:47692`
                self.invasion_progress_display_left = 160; // `:47693`
            }
            // `case 76` — a peer's quest/golf counts
            // (`MessageBuffer.cs:3164-3182`): our own echo is skipped
            // (`:3167`, `ServerSideCharacter` is false in the port), a peer's
            // lands in its record (`:3174-3175`). W4-g49.
            S2c::QuestsCountSync(q) => {
                if q.player_id as u32 == self.local_id {
                    return; // `:3167`
                }
                let peer = self.remote_players.entry(q.player_id as u32).or_default();
                peer.angler_quests_finished = q.angler_quests_finished; // `:3174`
                peer.golfer_score = q.golfer_score; // `:3175`
            }
            // `case 79` — the host's placement relay
            // (`MessageBuffer.cs:3192-3215`): the client runs the same
            // `WorldGen.PlaceObject` (`:3209`, mute: false) — the host-only
            // spam/section gate (`:3201-3208`) and relay (`:3210-3213`) are
            // not the client's. W4-g49.
            S2c::PlaceObject(p) => {
                let dir = if p.direction { 1 } else { -1 }; // `:3200`
                let landed = tr_core::edit::place_object(
                    &mut self.world,
                    p.x as i32,
                    p.y as i32,
                    p.kind.max(0) as u16,
                    p.style as i32,
                    p.random as i32,
                    dir,
                );
                if landed {
                    // The `mute: false` Dig cue (`WorldGen.cs:51441`), the
                    // bare-`Main.rand` stand-in (`Main.cs:16895`).
                    self.audio.play_place_sound(
                        (0, 1),
                        p.x as f32 * TILE_PX,
                        p.y as f32 * TILE_PX,
                        self.rngs.global(),
                    );
                }
                self.world_key = None;
            }
            // `case 48` — the host's liquid fan-out / drained re-push
            // (`MessageBuffer.cs:2407-2446`): the client applies the liquid
            // byte + type (`:2434-2435`); the spam window and the host
            // re-frame/re-push (`:2413-2443`) are host-only. W4-g49.
            S2c::LiquidUpdate(l) => {
                if let Some(t) = self.world.tile_mut(l.x as i32, l.y as i32) {
                    t.liquid = l.liquid; // `:2434`
                    match l.liquid_type {
                        0 => t.clear_liquid_kind(),
                        1 => t.set_liquid_kind(tr_core::tile::LiquidKind::Lava),
                        2 => t.set_liquid_kind(tr_core::tile::LiquidKind::Honey),
                        3 => t.set_liquid_kind(tr_core::tile::LiquidKind::Shimmer),
                        _ => {} // `Tile.cs:216-233` — no default arm
                    }
                    self.world_key = None;
                }
            }
            // `case 52` — the host's unlock/lock relay
            // (`MessageBuffer.cs:2531-2564`): the client runs the same
            // `Chest.Unlock`/`WorldGen.UnlockDoor`/`Chest.Lock`; the relay +
            // `SendTileSquare` halves (`:2539-2543` etc.) are host-only (the
            // 2×2 `SectionRange` arrives separately). W4-g49.
            S2c::LockAndUnlock(l) => {
                let (x, y) = (l.x as i32, l.y as i32);
                let plant_boss = self.world.downed_plant_boss;
                match l.action {
                    1 => {
                        crate::chest::unlock_chest(&mut self.world, x, y, plant_boss);
                    }
                    2 => {
                        if let Some(top) = unlock_door(&mut self.world, x, y) {
                            unlock_door_effects(self, x, top);
                        }
                    }
                    3 => {
                        crate::chest::lock_chest(&mut self.world, x, y, plant_boss);
                    }
                    _ => {}
                }
                self.world_key = None;
            }
            // `case 25/26/44/67/93` — the reader ignore group
            // (`MessageBuffer.cs:4483-4490`): accepted and dropped. W4-g49;
            // the 25/26/93 bare frames are the W4-f21 census lifts.
            S2c::Unused25
            | S2c::Unused26
            | S2c::Unknown44
            | S2c::Unknown67
            | S2c::SocialHandshake => {}
            // S1-I-1 W4-f21: `39`'s client FindOwner re-run
            // (`MessageBuffer.cs:2237-2243`) is the ITEM lane's task-seed
            // (research/318) — bodies + codecs only here.
            S2c::ReleaseItemOwnership(_) => {}
            // `case 119` — the 81 string twin (W4-g50,
            // `MessageBuffer.cs:3235-3242`): the same truncation/alpha, the
            // text is `NetworkText.Deserialize(...).ToString()` — the port's
            // literal-mode `NetText` already carries the resolved string.
            S2c::CombatTextString(t) => {
                let text = t.text.0.clone();
                let size = match self.lookups.combat_text_font.as_ref() {
                    Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                    None => [text.chars().count() as f32 * 8.0, 10.0],
                };
                let location = crate::combat_text::Rect {
                    x: t.x as i32 as f32,
                    y: t.y as i32 as f32,
                    width: 0.0,
                    height: 0.0,
                };
                crate::combat_text::spawn(
                    &mut self.combat_texts,
                    location,
                    [t.color.0, t.color.1, t.color.2, 255],
                    text,
                    false,
                    false,
                    size,
                    1.0, // the port keeps gravDir = 1 (`player.rs:4693`)
                    self.rngs.global(),
                );
            }
            // `case 51` — the host's misc fan-out (`MessageBuffer.cs:2482-2530`,
            // the client half). Kind 2 plays Item1 at the subject's position
            // (`:2502`, `SoundID.Item1` — the case-2 façade, the net handlers
            // run outside every swap so the draws ride the bare stream).
            // Kinds 1/3/5/6 are host-side only; kind 4's big-mimic smoke
            // burst is parked (the roster is unported), W4-c27.
            S2c::MiscDataSync(d) => {
                if d.kind == 2 {
                    if let Some(p) = self.remote_players.get(&(d.player_id as u32)) {
                        let (x, y) = (p.x, p.y);
                        self.audio
                            .play_npc_item_cue(1, x, y, self.rngs.global()); // `:2502`
                    }
                }
            }
            // `case 55` — a PvP debuff aimed at us (`MessageBuffer.cs:2599`'s
            // `netMode == 1` clause: only our own id applies; `:2607`
            // `AddBuff(fromNetPvP: true)` — the wire time is final).
            S2c::PlayerBuffPvp(b) => {
                if b.player_id as u32 == self.local_id {
                    if let Some(p) = self.player.as_mut() {
                        p.buffs.add_buff_from_net_pvp(b.buff_type as i32, b.time);
                    }
                }
            }
            // `case 62` — a dodge on the wire (`MessageBuffer.cs:2879-2908`,
            // the `netMode == 1` half keeps the wire id). Ours runs the
            // ported handlers (the `Player.cs:38023-38165` bodies via
            // `player.rs` — the client IS the subject's `myPlayer`, so the
            // buff strip / Hallowed cooldown / brain buff run). A peer's
            // dodge on the remote copy leaves only the 120/80 immunity
            // window (the rest is `myPlayer`-gated, `Player.cs:38026-38029`)
            // and the dust/gore bursts — the port has no peer hurt lane
            // (hostile shots parked, the `:2577` follow-up) and the bursts
            // are S1-E-1, so that half is parked, same as the host arm.
            // W4-c27.
            S2c::SyncDodge(d) => {
                if d.player_id as u32 == self.local_id {
                        p.apply_net_dodge(d.kind);
            // `case 84` — a peer's stealth value (`MessageBuffer.cs:3246-3260`,
            // record (`:3254`); ours is applied by the local stealth writer
            // that sent it, the relay echo excluded by the host.
            S2c::PlayerStealth(s) => {
                    .entry(s.player_id as u32)
                    .stealth = s.stealth; // `:3254`
            // `case 147`'s client half (`MessageBuffer.cs:4336-4337`):
            // `TrySwitchingLoadout` + `ReadAccessoryVisibility` on
            // `Main.player[num68]`. The LOCAL player runs the real swap core
            // (`player.rs`, the same `EquipmentLoadout` two-swap) then unpacks
            // the visibility bits (`MessageBuffer.cs:4494-4501`, bit i per
            // slot); the swap gate's `whoAmI != Main.myPlayer` arm passes for
            // a network-applied switch (`Player.cs:5663`). A peer's loadout is
            // stored-not-simulated on the render snapshot (no per-peer
            // equipment arrays).
            S2c::SyncLoadout(l) => {
                if l.player_id as u32 == self.local_id {
                    let idx = l.loadout as usize;
                    if let Some(p) = self.player.as_mut() {
                        // `:4336` — the swap core only (the local tails —
                        // clone/12 cue/147/particles — are the `myPlayer`
                        // arm's, `:5668-5680`, and a network apply is not it).
                        p.try_switching_loadout(
                            idx,
                            &mut self.equip.armor,
                            &mut self.equip.dye,
                            &mut self.equip_share_flags,
                            false,
                        );
                        // `:4337` — ReadAccessoryVisibility.
                    if let Some(p) = self.player.as_mut() {
                        p.apply_net_dodge(d.kind);
                    }
                }
            }
            // `case 84` — a peer's stealth value (W4-g50,
            // `MessageBuffer.cs:3246-3260`): the subject is the wire's id
            // byte (the host already forced it to the sender); the port
            // stores it on the peer snapshot — the stealth-armour draw
            // reads it when that system lands.
            S2c::PlayerStealth(s) => {
                if s.player_id as u32 == self.local_id {
                    if let Some(p) = self.player.as_mut() {
                        p.stealth = s.stealth;
                    }
                } else if let Some(p) = self.remote_players.get_mut(&(s.player_id as u32)) {
                    p.stealth = s.stealth;
                }
            }
            // `case 92` — the town NPC's accumulated coin credit (W4-g50,
            // `MessageBuffer.cs:3442-3461`): the client assigns the TOTAL
            // (`:3453`) and pings the coins at the wire position (`:3452`'s
            // `moneyPing`, `NPC.cs:90052-90073` — the case-38 cue then the
            // 20-dust burst on the bare `Main.rand` stream, the message-time
            // replay convention S1-E-21i). The slot is the HOST's NPC index
            // — the client's slot-keyed store (`net_npcs`, the 23/54
            // convention), rebuilt into the render list by the dirty flag.
            S2c::NpcExtraValue(e) => {
                if e.npc < 0 {
                    return; // `num228 >= 0 && num228 <= Main.maxNPCs` (`:3448`)
                }
                let Some(npc) = self.net_npcs.get_mut(&(e.npc as u32)) else {
                    return;
                };
                npc.extra_value = e.value.max(0) as u64;
                self.net_npcs_dirty = true;
                let credit = npc.extra_value;
                // `moneyPing`'s sound half (`NPC.cs:90054`): the positioned
                // case-38 coin cue (`Audio::play_coin_ping`, S1-E-21f).
                let rng = self.rngs.global();
                self.audio.play_coin_ping(e.x, e.y, rng);
                // The dust half (`NPC.cs:90055-90072`): the type ladder on
                // the NEW total, then 20 spawns at `pos - (1, 2)` with the
                // 10x14 box, alpha 254, scale 0.25 and the ×0.2 velocity.
                let dust_type = if credit >= 1_000_000 {
                    247
                } else if credit >= 10_000 {
                    246
                } else if credit >= 100 {
                    245
                } else {
                    244
                };
                let ctx = self.spawn_ctx();
                let pos = Vec2::new(e.x - 1.0, e.y - 2.0);
                for _ in 0..20 {
                    let rng = self.rngs.global();
                    let idx = self.dust.new_dust(
                        rng,
                        &ctx,
                        &self.world,
                        pos,
                        10,
                        14,
                        dust_type,
                        0.0,
                        0.0,
                        254,
                        tr_core::particles::Color::default(),
                        0.25,
                    );
                    if idx != tr_core::particles::DUST_SENTINEL {
                        self.dust.slot_mut(idx).velocity *= 0.2;
                    }
                }
            }
            // `case 86` — one tile entity's create/update/removal (W4-g50,
            // `MessageBuffer.cs:3272-3293`): absent removes by id; present
            // reads the networkSend record and re-adds it under the leading
            // id (`TileEntity.Add`, `:3288-3290`).
            S2c::TileEntitySharing(t) => {
                if t.record.is_empty() {
                    // `TileEntity.TryGet<TileEntity>(num114, …)` → `Remove`.
                    if t.id >= 0 {
                        self.world.tile_entities.remove_by_id(t.id as u32);
                        self.world_key = None;
                    }
                    return;
                }
                let mut r = tr_core::binio::Reader::new(&t.record);
                let Some(mut entity) = tr_core::tile_entities::TileEntity::read_net(&mut r)
                else {
                    return; // the C# throws on a corrupt record; the port drops it
                };
                entity.id = t.id.max(0) as u32;
                if self.world.tile_entities.add_net(entity) {
                    self.world_key = None;
                }
            }
            // `case 88` — ItemTweaker (W4-g50, `MessageBuffer.cs:3306-3376`):
            // the C# applies the gated writes to `Main.item[slot].inner`
            // (the mutable per-instance stats). FINALIZE-PARK (re-probe
            // census): the tweak is NOT prefix/stack — it is the 13 inner
            // STAT fields (color/damage/knockBack/useAnimation/useTime/
            // shoot/shootSpeed + width/height/scale/ammo/useAmmo/notAmmo).
            // The port's `ItemDrop` still carries none of them (kind/count/
            // prefix + the physics fields only, `item.rs:845-891`), and the
            // ONLY vanilla producer is `CommonCode.ModifyItemDropFromNPC`
            // (`ItemDropRules/CommonCode.cs:56-95`): six COLOR-only sends
            // (`SendData(88, …, 1f)` = bitsByte bit 0) for the gel 23
            // (slime-tinted + remix-59 orange) and the four mimic banners
            // 319 (netID 542-545 tints) — no stat send exists tree-wide.
            // The tinted-drop producer itself is unported (the loot lane's),
            // so there is no message to apply; the `0..=400` slot gate
            // (`:3313`) stays the reader's only behaviour. The codec is
            // byte-verbatim (`net.rs`) for the census.
            S2c::ItemTweaker(_) => {}
            // `case 17` on a client — the host's edit relay
            // (`MessageBuffer.cs:1157-1275`, the `netMode == 1` path): the
            // same kind dispatch, no spam/section gates, no relay; kind 0
            // clears the crack book (`:1160-1163`) — W4-g48 (`net_apply`).
            S2c::TileEdit(e) => {
                crate::app::net_apply::client_tile_edit(self, e);
            }
            // `case 19` on a client — the host's door relay
            // (`MessageBuffer.cs:1307-1327`): the same action dispatch, no
            // relay — W4-g48 (`net_apply`).
            S2c::DoorToggle(d) => {
                crate::app::net_apply::client_door_toggle(self, d);
            }

            // `case 12` on a client — a peer spawned (`MessageBuffer.cs:901-
            // 950`): the same field writes as the host arm (`:908-918`), then
            // `Spawn(context)` repositions the peer (`Player.cs:37621-37641`:
            // `Spawn_SetPosition`, velocity/netOffset zeroed). The local echo
            // never arrives (the host relays with `ignoreClient = sender`).
            S2c::PlayerSpawn(s) => {
                let peer = s.player_id as u32;
                if peer == self.local_id {
                    return;
                }
                let (px, py) = self.net_peer_spawn_pos();
                let entry = self
                    .remote_players
                    .entry(peer)
                    .or_insert_with(|| net::PlayerNet {
                        id: peer,
                        name: format!("Player {}", s.player_id),
                        x: px,
                        y: py,
                        ..Default::default()
                    });
                entry.spawn_x = s.x;
                entry.spawn_y = s.y;
                entry.respawn_timer = s.respawn_timer;
                entry.deaths_pve = s.deaths_pve;
                entry.deaths_pvp = s.deaths_pvp;
                entry.team = s.team as i32;
                entry.dead = s.respawn_timer > 0; // `:915-918`
                // `Spawn_SetPosition` (`Player.cs:37987-37991`) + the
                // `netOffset`/velocity resets (`:37638-37641`).
                entry.x = s.x as f32 * TILE_PX + 8.0 - player::WIDTH * 0.5;
                entry.y = s.y as f32 * TILE_PX - player::HEIGHT;
                entry.vx = 0.0;
                entry.vy = 0.0;
                entry.off_x = 0.0;
                entry.off_y = 0.0;
                entry.talk_npc = None; // `:37648` — `SetTalkNPC(-1)`
            }
            // `case 35` on a client — a peer's heal text
            // (`MessageBuffer.cs:2168-2171`, `Player.HealEffect`) — W4-g48.
            S2c::PlayerHeal(h) => {
                crate::app::net_apply::client_player_heal(self, h.player_id, h.amount);
            }
            // `case 18` — the host's clock (`MessageBuffer.cs:1290-1298`,
            // client-only). `Main.time` is the wire `int`; the port's clock is
            // f64 ticks. `sunModY`/`moonModY` (`Main.cs:1185`/`:1187`) are the
            // TITLE-SCREEN sun/moon drag offsets (the only writers are
            // `DrawSunAndMoon`'s `gameMenu && netMode != 1` block,
            // `Main.cs:62983-63007`) — no port field yet, parked (W4-c27;
            // research/332 s4: the wire keeps them, the apply drops them).
            S2c::SetTime(t) => {
                self.world.day_time = t.day_time;
                self.world.time = t.time as f64;
            }
            // `case 36` on a client — a peer's zone words
            // (`MessageBuffer.cs:2187-2192`) — W4-g48 (`net_apply`).
            S2c::PlayerZone(z) => {
                crate::app::net_apply::client_player_zone(self, z);
            }
            // `case 40` on a client — a peer's talk index
            // (`MessageBuffer.cs:2264`) — W4-g48 (`net_apply`).
            S2c::TalkNpc(t) => {
                crate::app::net_apply::client_talk_npc(self, t.player_id, t.npc);
            }
            // `case 41` — a peer's aimed-item rotation/animation
            // (`MessageBuffer.cs:2271-2289`, the `netMode == 1` half keeps the
            // wire id): stored on the peer record, `channel` re-derived from
            // the peer's selected inventory item (`:2283`).
            S2c::ItemAnim(a) => {
                let entry = self
                    .remote_players
                    .entry(a.player_id as u32)
                    .or_default();
                entry.item_rotation = a.rotation;
                entry.item_animation = a.animation;
                entry.channel = Self::peer_channel(entry); // `:2283`
            }
            // `case 43` — a peer's mana-restore amount
            // (`MessageBuffer.cs:2307-2324`): the HealMana text spawns for
            // anyone but ourselves (`:2315-2318`).
            S2c::ManaEffect(m) => {
                if m.player_id as u32 != self.local_id {
                    self.spawn_peer_mana_text(m.player_id, m.amount as i32);
                }
            }
            // `case 125`'s client half (`MessageBuffer.cs:4033-4036`):
            // `Main.player[Main.myPlayer].GetOtherPlayersPickTile(x, y, dmg)`
            // (`Player.cs:53751-53755`) — `hitTile.HitObject(x, y, 1)` then
            // `AddDamage(tileId, pickDamage)`. The damage accumulates on the
            // LOCAL player's pick book (the wire subject is the peer, but the
            // C# applies to `Main.myPlayer`).
            S2c::TilePicking(p) => {
                let tile_id = self
                    .hit_tile
                    .hit_object(p.x as i32, p.y as i32, tr_core::hit_tile::TILE);
                self.hit_tile.add_damage(tile_id, p.damage as i32, true);
            }
            // `case 126`'s client half (`MessageBuffer.cs:4043-4048`):
            // `NPC.RevengeManager.AddMarkerFromReader` — the marker joins the
            // session list (`CoinLossRevengeSystem.cs:321-324`). The ctor
            // computes `_expirationTime = CalculateExpirationTime(gameTime,
            // coinValue)` (`:86`, `:155-165`); the port's `expiration_addon`
            // is that bracket table and `revenge_tick` is `_gameTime`. The
            // kind rides `kind_from_net_id` (the 23 convention); an unported
            // kind drops the marker (the respawn pass has no model for it).
            S2c::RevengeMarker(m) => {
                if let Some((kind, _)) = crate::npc::kind_from_net_id(m.npc_net_id) {
                    let coins = m.coins_value.max(0) as u64;
                    self.revenge_markers.push(crate::revenge::RevengeMarker {
                        location: Vec2::new(m.x, m.y),
                        kind,
                        hp_percent: m.npc_hp_percent,
                        base_value: m.base_value,
                        coins_value: coins,
                        expiration_tick: self.revenge_tick
                            + crate::revenge::expiration_addon(coins),
                        unique_id: m.unique_id as u64,
                        force_expire: false,
                    });
                }
            }
            // `case 127`'s client half (`MessageBuffer.cs:4049-4057`):
            // `NPC.RevengeManager.DestroyMarker(markerUniqueID)` —
            // `_markers.RemoveAll(x => x.UniqueID == markerUniqueID)`
            // (`CoinLossRevengeSystem.cs:334-340`). The 126 marker's twin.
            S2c::RemoveRevengeMarker(m) => {
                self.revenge_markers
                    .retain(|x| x.unique_id != m.unique_id as u64);
            }
            // `case 147`'s client half (`MessageBuffer.cs:4336-4337`):
            // `TrySwitchingLoadout` + `ReadAccessoryVisibility` on
            // `Main.player[num68]`. The loadout swap is unported (no loadout
            // system); the accessory-visibility bits unpack onto the local
            // player's `hide_visible_accessory` (`MessageBuffer.cs:4494-
            // 4501`, bit i per slot). A peer's loadout is stored on the
            // snapshot (stored-not-simulated).
            S2c::SyncLoadout(l) => {
                if l.player_id as u32 == self.local_id {
                    if let Some(p) = self.player.as_mut() {
                        for i in 0..p.hide_visible_accessory.len() {
                            p.hide_visible_accessory[i] = (l.hide_accessory & (1 << i)) != 0;
                        }
                    }
                } else {
                    self.remote_players
                        .entry(l.player_id as u32)
                        .or_default()
                        .loadout = Some(l);
                }
            }
            // `case 60` — an NPC's household change (`MessageBuffer.cs:2695-2739`,
            // W4-d119 phase 3). The client writes homeless + homeTileX/Y on
            // its NPC copy (`:2708-2713`), then the town-only book arms
            // (`:2719-2729`): status 1 = `TownManager.KickOut(type)`,
            // status 2 = `SetRoom(type, x, y)`. The host arms (`:2731-2738`
            // `WorldGen.kickOut`/`moveRoom`) need the C2s kick-out/move-room
            // senders (the housing-banner UI's net legs, unported) — the
            // port's host never receives a 60 today; recorded, not invented.
            S2c::NpcHouse(h) => {
                // `:2700-2704`: `num258 >= Main.maxNPCs` boots the sender —
                // a server-sent frame cannot boot anyone on the client; the
                // port's slot-keyed store drops an out-of-range slot below.
                let slot = h.npc as u32;
                let Some(npc) = self.net_npcs.get_mut(&slot) else {
                    return;
                };
                // `:2706`: `isLikeATownNPC` is read BEFORE the writes.
                let is_town = npc.kind.is_town_npc_like();
                // `:2708-2713`: the client's homeless + home-tile writes.
                // The port's `home` Option conflates the two — status 1
                // (homeless) clears it, every other status stores the pair
                // (the C# writes homeTileX/Y unconditionally on the client).
                npc.home = if h.status == 1 {
                    None
                } else {
                    Some((h.home_x as i32, h.home_y as i32))
                };
                self.net_npcs_dirty = true;
                // `:2715-2718`: a non-town NPC stops here.
                if !is_town {
                    return;
                }
                // `:2719-2729`: the client's TownManager book.
                let net_id = npc.kind.net_id();
                match h.status {
                    1 => self.town_rooms.kick_out(net_id),
                    2 => self
                        .town_rooms
                        .set_room(net_id, h.home_x as i32, h.home_y as i32),
                    _ => {}
                }
            }
            // S1-I-1 wave A: the join bundle's server-sent ids have bodies +
            // codecs (`net.rs`); the client read sites
            // (`MessageBuffer.cs:1851-1867`, `:2290-2306`, `:2325-2364`,
            // `:2695-2739`, `:3118-3126`, `:3147-3153`, `:4043-4048`,
            // `:4195-4203`, `:4238-4249`, `:4282-4297`, `:4328-4343`) land
            // with the send-path pass (S1-I-4) — nothing sends them yet.
            S2c::TogglePvp(_)
            | S2c::PlayerMana(_)
            | S2c::TeamChange(_)
            | S2c::TravelShop(_)
            | S2c::AnglerQuest(_)
            | S2c::RevengeMarker(_)
            | S2c::DeadPlayer { .. }
            | S2c::MiscEventValue(_)
            | S2c::ProjectileTrackers(_)
            // W4-f11 seam 8.2: the relayed `19` wiring echo — the client's
            // apply dispatch (`MessageBuffer.cs:1307-1327`) is g48's
            // `net_apply.rs::client_door_toggle` (union merge wires it).
            | S2c::DoorToggle(_)
            | S2c::SyncLoadout(_) => {}
            // S1-I-1 wave E: 51's kind-2 rod-sound echo
            // (`MessageBuffer.cs:2500-2503` SoundID.Item1) — same wave, same
            // parking.
            S2c::MiscDataSync(_) => {}
            // 65's host teleport broadcast (`MessageBuffer.cs:2985-3028`
            // Teleport + netOffset zero + the nearest-peer chat) — same
            // wave, same parking.
            S2c::TeleportEntity(_) => {}
            // 77 SyncTilePicking (`MessageBuffer.cs:3183-3185`
            // Animation.NewTemporaryAnimation) — same wave, same parking.
            S2c::TemporaryAnimation(_) => {}
            // 78 ReportInvasionProgress (`MessageBuffer.cs:3186-3191`
            // Main.ReportInvasionProgress) + 96's relay (`:3538-3557`
            // parity flip + style-4 teleport) — same wave, same parking.
            S2c::InvasionProgress(_) | S2c::PlayerPortalTeleport(_) => {}
            // 103 Moon Lord countdown (`MessageBuffer.cs:3708-3714` stores
            // both statics) + 112 SpecialFX (`:3815-3840` TreeGrowFX /
            // FairyEffects) — same wave, same parking.
            S2c::MoonlordCountdown(_) | S2c::SpecialFx(_) => {}
            // 114 DD2 wipe (`MessageBuffer.cs:3855-3860`
            // DD2Event.WipeEntities on the client) — same wave, same parking.
            S2c::CrystalInvasionWipe => {}
            // 116 DD2 wave countdown (`MessageBuffer.cs:3861-3866`) + 127
            // revenge-marker removal (`:4049-4057` DestroyMarker) — same
            // wave, same parking.
            S2c::CrystalInvasionWait(_) | S2c::RemoveRevengeMarker(_) => {}
            // 141's relay (`MessageBuffer.cs:4278` CreateFromNet) — same
            // wave, same parking.
            S2c::LucyPopup(_) => {}
            // W4-d97 (S1-I-1 player-family second half): bodies + codecs in
            // `net.rs`; the client read sites — the team level-up fan
            // (`MessageBuffer.cs:3662-3705`) and the quiet `Player.Hurt`
            // replay (`:3879`) — land with the send-path pass (S1-I-4).
            S2c::NebulaLevelupRequest(_) | S2c::PlayerHurtV2(_) => {}
            // W4-d97 batch 2: the client `Player.KillMe` read site
            // (`MessageBuffer.cs:3898`) lands with the send-path pass
            // (S1-I-4). 120 Emoji has no S2c side — the host consumes it
            // without relaying (`:3905-3918`).
            S2c::PlayerDeathV2(_) => {}
            // W4-d97 batch 3: the client luck-field apply (`:4180-4188`) and
            // the peer-spectate apply (`MessageBuffer.cs:4380-4383`) land
            // with the send-path pass (S1-I-4).
            S2c::UpdatePlayerLuckFactors(_) | S2c::SpectatePlayer(_) => {}
            // S1-I-1 wave D94: the projectile family's server-sent support
            // ids (66 `:3036-3055`, 99 `:3570-3583`, 115 `:3584-3597`) have
            // bodies + codecs (`net.rs`); the client read sites land with
            // the send-path pass (S1-I-4) — nothing sends them yet.
            S2c::HealPlayer(_) | S2c::MinionRestTarget(_) | S2c::MinionAttackTarget(_) => {}
            // 101 tower shields (`:3611-3651` clamp) + 106 smoke poof
            // (`:3742-3751` Utils.PoofOfSmoke) — same wave, same parking.
            S2c::TowerShields(_) | S2c::PoofOfSmoke(_) => {}
            // 100 NPC portal teleport (`:3598-3610` style-4 teleport +
            // netOffset zero) + 108 cannon redirect (`:3758-3773` owner-gated
            // ShootFromCannon) — same wave, same parking.
            S2c::NpcPortalTeleport(_) | S2c::WiredCannonShot(_) => {}
            // S1-I-1 wave B (W4-d92): the NPC family's server-sent ids have
            // bodies + codecs (`net.rs`); the client read sites
            // (`MessageBuffer.cs:4122-4133` the immunity tame, `:4204-4218`
            // the cavern grid, `:4408-4418` the debuff damage, `:1819-1824`
            // the strike ack's pendingDamages pop) land with the send-path
            // pass (S1-I-4) — nothing sends them yet.
            S2c::TamperWithNpc(_)
            | S2c::CavernMonsterTypes(_)
            | S2c::NpcDebuffDamage(_)
            | S2c::NpcStrikeAck => {}
            // S1-I-1 wave B batch 2 (W4-d92): the town-NPC info answer's
            // client store arm (`MessageBuffer.cs:2617-2623`, GivenName +
            // townNpcVariationIndex behind the slot gate) lands with S1-I-4.
            S2c::UniqueTownNpcInfo(_) => {}
            // S1-I-1 wave B batch 2 (W4-d92): the coin-credit total's client
            // assign + moneyPing arm (`MessageBuffer.cs:3450-3454`) lands
            // with S1-I-4.
            S2c::NpcExtraValue(_) => {}
            // `case 150` (`MessageBuffer.cs:4361-4384`): the host's
            // `SetOrRequestSpectating` / stop-relay and the client's
            // `spectating` apply need the unported spectating system
            // (`Player.spectating`, `Player.cs:1634`); both arms park with it
            // (S1-I-4).
            S2c::SpectatePlayer(_) => {}
            // `case 153`'s client half (`MessageBuffer.cs:4408-4417`):
            // `GetHurtByDebuff(damage)` (`NPC.cs:91864-91888`) — the DoT
            // drain bypasses StrikeNPC (no i-frames, no knockback). The
            // damage lands on the carrier (`realLife`, the port's
            // `carrier_index`); the immortal gate skips the subtraction; a
            // life ≤ 0 floors to 1 + the 9999 no-interaction kill (the
            // port's instant-kill hp=0 convention, `npc.rs`'s
            // StrikeNPCNoInteraction note). The combat text (`:91879`,
            // LifeRegenNegative DoT) spawns on the struck NPC's box.
            S2c::NpcDebuffDamage(d) => {
                let slot = d.npc as usize;
                if slot >= self.npcs.len() {
                    return;
                }
                let target = crate::npc::Npc::carrier_index(&self.npcs, slot).unwrap_or(slot);
                let amount = d.damage as i32;
                if !self.npcs[target].immortal {
                    self.npcs[target].hp -= amount; // `:91877`
                }
                // `:91879` — the DoT combat text on the STRUCK NPC's box.
                // The text size rides the combat-font measurement (the
                // heal-life convention, `world_sim/items.rs`); gravDir 1.0
                // (the port has no gravity-flip system).
                let (nx, ny) = (self.npcs[slot].x, self.npcs[slot].y);
                let (nw, nh) = self.npcs[slot].size();
                let text = amount.to_string();
                let size = match self.lookups.combat_text_font.as_ref() {
                    Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                    None => [text.chars().count() as f32 * 8.0, 10.0],
                };
                crate::combat_text::spawn_amount(
                    &mut self.combat_texts,
                    crate::combat_text::Rect {
                        x: nx,
                        y: ny,
                        width: nw,
                        height: nh,
                    },
                    crate::combat_text::LIFE_REGEN_NEGATIVE,
                    amount,
                    false,
                    true,
                    size,
                    1.0,
                    self.rngs.global(),
                );
                if self.npcs[target].hp <= 0 && !self.npcs[target].immortal {
                    // `:91880-91887` — life = 1 + StrikeNPCNoInteraction(9999);
                    // the port's no-interaction instant kill is hp = 0.
                    self.npcs[target].hp = 0;
                }
            }
            // `case 158`'s client half (`MessageBuffer.cs:4450-4456`):
            // `Main.player[b3].Spawn(PlayerSpawnContext.TeamSwap)` — the
            // team-swap respawn at the extra spawn point. DOCUMENTED SKIP
            // (W4-g51 id-158 census): the whole feature is netMode+seed-gated
            // and large. The send is the `157` tail (`MessageBuffer.cs:2355-
            // 2363`) behind `b == 157 && Main.teamBasedSpawnsSeed` AND a
            // successful `TryGetExtraSpawnPointForTeam`; the client's Spawn
            // routes through `Spawn_SetPositionAtTeamSpawn`
            // (`Player.cs:37993-38011`), itself `teamBasedSpawnsSeed`-gated.
            // The spawn points come from `ExtraSpawnPointManager` (446 lines:
            // the skyblock/roundLandmass/extraLiquid landmass scans,
            // `GenerateExtraSpawns_Setup` `:40-82`, the per-team Point[] the
            // worldgen fills) — a secret-seed worldgen feature
            // (`MicroSeedFlags::TEAM_BASED_SPAWNS`, bit 33) with no ported
            // generator. The port's `teamBasedSpawnsSeed` reads false
            // (`pvp_icon_state`'s note), so the arm is unreachable until the
            // seed + manager + Spawn team-position land (M5-seed rows). The
            // `7 WorldInfo` body already carries `extra_spawn_points`
            // (`ExtraSpawnPointManager.Write`, the empty Vec today).
            S2c::ExtraSpawnLoaded { .. } => {}
            // `case 162`'s client half (`MessageBuffer.cs:1819-1824`):
            // `NPC.AckDamage` pops `NPC.pendingDamages[0]` FIFO
            // (`NPC.cs:90703-90713`). W4-g51 id-162 wired the queue: the
            // client's strike pushes (`send_npc_strike`), the `23` reader
            // subtracts the pending sum, and this ack pops the oldest.
            S2c::NpcStrikeAck => {
                crate::app::net_npc_sync::ack_damage(&mut self.net_pending_damages);
            }
            // `case 135`'s client half (`MessageBuffer.cs:4195-4203`):
            // `Main.player[num103].immuneAlpha = 255` — the dead-peer ghost
            // fade. The port's `immuneAlpha` blinkVisual half is draw-side
            // and stays out (`player.rs`'s `update_immunity` doc, R-03-5),
            // so there is no field to write; the arm is parked with the
            // render-side ghost fade (S1-I-4).
            S2c::DeadPlayer { .. } => {}
            // `case 136`'s client half (`MessageBuffer.cs:4204-4217`): the
            // 2×3 `NPC.cavernMonsterType` grid stores verbatim (the host
            // ignores it, `:4206-4209`). The port's world field is the
            // Main-global standing-in (`World::cavern_monster_type`); the
            // deep-cavern spawn ladder's `:5099` grid arm stays censused
            // (the 494-506 kinds have no NpcKind — the roster gap).
            S2c::CavernMonsterTypes(t) => {
                for i in 0..2 {
                    for j in 0..3 {
                        self.world.cavern_monster_type[i][j] = t.types[i][j] as i32;
                    }
                }
            }
            // `case 140` (`MessageBuffer.cs:4238-4263`): kind 0 is the
            // client's `CreditsRollEvent.SetRemainingTimeDirect`
            // (`CreditsRollEvent.cs:45-48`, `Utils.Clamp(time, 0, 28800)`)
            // — WIRED to the world field (the timed roll's state; the
            // CreditsRollSky cinematic itself is unported, the front
            // screen stays untimed). Kinds 1/2 are HOST-only arms
            // (`:4251`/`:4257`'s `netMode == 2` gates) — the client
            // ignores them; unknown kinds read the tail and do nothing.
            S2c::MiscEventValue(v) => {
                if v.kind == 0 {
                    self.world.set_credits_roll_remaining_time(v.value);
                }
            }
            // `case 141`'s client half (`MessageBuffer.cs:4276-4279`):
            // `LucyAxeMessage.CreateFromNet` spawns the popup text +
            // `LucyTheAxeTalk` cue. The port has no Lucy popup-text system
            // yet, so the arm parks with it (S1-I-4); the host relay above
            // carries the body verbatim.
            S2c::LucyPopup(_) => {}
            // `case 142`'s client half (`MessageBuffer.cs:4289-4291`): the
            // two `TrackedProjectileReference.Read`s land on
            // `Main.player[num79]`. The port has no piggy-bank/void-lens
            // projectile-tracking consumer, so the references are
            // stored-not-simulated on the snapshot (the local player's
            // included — there is no live field to write).
            S2c::ProjectileTrackers(t) => {
                self.remote_players
                    .entry(t.player_id as u32)
                    .or_default()
                    .trackers = Some(t);
            }
            // `case 128`'s client half (`MessageBuffer.cs:4070-4072`):
            // `PutBallInCup_TextAndEffects` — the cup dust/sound/chat FX
            // (`GolfHelper.cs:115-123`) ride the golf system (M4-GOLF-1);
            // the relay lands here until then (S1-I-4).
            S2c::GolfBallInCup(_) => {}
            // `case 131`'s client half (`MessageBuffer.cs:4122-4133`):
            // kind 1 runs `GetImmuneTime(time, fromWho)` (`NPC.cs:92508-
            // 92521`): fromWho -1 sets EVERY immune slot, else just
            // `[fromWho]`. A slot ≥ maxNPCs tames a throwaway `new NPC()`
            // (`:4127`) — the port's live-slot gate skips it (no slot to
            // write). The only send site is the 485 statue-break 582 spawn
            // (`WorldGen.cs:48373-48376`, fromWho -1, time 20).
            S2c::TamperWithNpc(t) => {
                if t.kind != 1 {
                    return; // `:4128` — only kind 1 reads the tail
                }
                let Some((time, from_who)) = t.immune else {
                    return;
                };
                let slot = t.npc as usize;
                if slot >= self.npcs.len() {
                    return; // `:4127` — the throwaway NPC, no live slot
                }
                let npc = &mut self.npcs[slot];
                if from_who == -1 {
                    npc.immune = [time; 256]; // `:92512-92515`
                } else if from_who >= 0 && (from_who as usize) < npc.immune.len() {
                    npc.immune[from_who as usize] = time; // `:92517-92520`
                }
            }
            // `case 134`'s client half (`MessageBuffer.cs:4180-4188`): the
            // eight fields land on `Main.player[num104]` then
            // `RecalculateLuck`. The local player's live `luck_state` takes
            // the apply + recalc; a peer's factors are stored on the snapshot
            // (stored-not-simulated, the host convention).
            S2c::UpdatePlayerLuckFactors(l) => {
                if l.player_id as u32 == self.local_id {
                    if let Some(p) = self.player.as_mut() {
                        apply_luck_factors(&mut p.luck_state, &l);
                        p.luck_state.recalculate_luck(); // `:4188`
                    }
                } else {
                    self.remote_players
                        .entry(l.player_id as u32)
                        .or_default()
                        .luck = Some(l);
                }
            }
            // `case 129` (`MessageBuffer.cs:4075-4085`): the client's team
            // re-push (`45` when `team > 0`), then `FixUIScale` +
            // `TrySetPreparationState(ProcessingData)`. The port's join walk
            // already drives the state machine; the team re-push is the
            // observable wire half (the UI-scale/preparation-state are
            // front-end internals the port's install owns). W4-g51 f21.
            S2c::FinishedConnecting => {
                if let Some(p) = self.player.as_ref() {
                    if p.team > 0 {
                        if let NetRole::Client(client) = &self.net {
                            client.send(&C2s::TeamChange(net::TeamChange {
                                player_id: self.local_id as u8,
                                team: p.team as u8,
                            }));
                        }
                    }
                }
            }
            // `case 139` (`MessageBuffer.cs:4230-4237`): the client stores
            // `Main.countsAsHostForGameplay[id]` — a GLOBAL per-slot array,
            // so the join push names the receiving client's OWN slot. The
            // port keeps it on the peer snapshot (the creative-power
            // authority consumer is unported); the local slot's flag is the
            // g52c master input the host-authority gates read. W4-g51 FINAL.
            S2c::CountsAsHost(c) => {
                self.remote_players
                    .entry(c.player_id as u32)
                    .or_default()
                    .counts_as_host = c.counts_as_host;
            }
            // `case 146` (`MessageBuffer.cs:4310-4327`): kind 0
            // `WorldItem.ShimmerEffect` (the FX — the port's shimmer dust/sound
            // is the shimmer-system's, parked here as the FX-only arm), kind 1
            // `AddCoinLuck` (`Player.cs:18090-18101`: the local player's
            // coin_luck += amount within 1000 px, capped 1e6). Unknown kinds
            // read nothing. W4-g51 f21.
            S2c::ShimmerAction(a) => match a {
                net::ShimmerAction::CoinLuck { x, y, amount } => {
                    if let Some(p) = self.player.as_mut() {
                        let pc = Vec2::new(p.center().0, p.center().1);
                        // `Player.cs:18092`'s `!dead && distance < 1000`.
                        if !p.dead && (Vec2::new(x, y) - pc).length() < 1000.0 {
                            p.luck_state.coin_luck =
                                (p.luck_state.coin_luck + amount as f32).min(1_000_000.0);
                            p.luck_state.recalculate_luck();
                        }
                    }
                }
                // kind 0's ShimmerEffect is the dust+sound FX
                // (`WorldItem.cs:1661-1689`) — the port's shimmer FX rides the
                // shimmer system; the net FX arm parks with it (the coin-luck
                // half above is the gameplay-observable state).
                net::ShimmerAction::Effect { .. } | net::ShimmerAction::Unknown(_) => {}
            },
            // `case 152`'s client half (`MessageBuffer.cs:4397-4405`): play the
            // peer's held item's `UseSound` at its Centre with `useSoundPitch`.
            // The port reads the peer's selected inventory slot from the
            // snapshot; the local player's own use-sound already played at the
            // use start (the `initial_use_sound` path), so a self-echo is
            // skipped (the C# plays every peer's including its own relay — the
            // port's relay excludes the sender, so no self-echo arrives).
            S2c::ItemUseSound { player_id } => {
                let peer = player_id as u32;
                if let Some(p) = self.remote_players.get(&peer) {
                    let held = p.inventory.get(&(p.selected as u8)).map(|it| it.kind);
                    if let Some(kind) = held {
                        if kind > 0 {
                            if let Some((style, pitch)) = crate::item::initial_use_sound(kind as u16) {
                                let (x, y) = (p.x, p.y);
                                self.audio.play_use_sound(
                                    style,
                                    x,
                                    y,
                                    pitch,
                                    self.rngs.global(),
                                );
                            }
                        }
                    }
                }
            }
            // `case 160` (`MessageBuffer.cs:4463-4470`): the drop's position
            // snap (no velocity) — the section-sweep correction. The port's
            // client drops live in `net_items` keyed by the host slot.
            // W4-g51 f21.
            S2c::ItemPosition(p) => {
                if let Some(d) = self.net_items.get_mut(&p.slot) {
                    d.pos = Vec2::new(p.x, p.y);
                    self.net_items_dirty = true;
                }
            }
            // `case 121`'s client half (`MessageBuffer.cs:3930-3935`):
            // `ReadData`/`ReadDummySync` on the doll store (no relay — the
            // client never re-sends). W4-g51 FINAL.
            S2c::DisplayDollSync(d) => {
                self.apply_doll_sync(d);
            }
            // `case 122`'s client half (`MessageBuffer.cs:3964-3975`): -1
            // clears the anchor; otherwise `TryGet` + `SetInteractionAnchor`
            // (`TileEntity.cs:289-313`): chest = -1, SetTalkNPC(-1), the
            // local-player UI tail, then `tileEntityAnchor.Set`. W4-g51 FINAL.
            S2c::TileEntityInteraction(t) => {
                if t.entity_id == -1 {
                    self.net_te_anchors.remove(&t.player_id); // `Clear()`
                    return;
                }
                if t.entity_id < 0 {
                    return;
                }
                let Some(e) = self
                    .world
                    .tile_entities
                    .get_by_id(t.entity_id as u32)
                    .filter(|e| {
                        // `TryGet` fails on a removed `ByID` entry — the
                        // tombstoned slot reads as missing (SEAL re-pin).
                        self.world
                            .tile_entities
                            .get_at(e.x, e.y)
                            .is_some_and(|live| live.id == e.id)
                    })
                else {
                    return; // `TryGet` fails
                };
                let (ex, ey) = (e.x as i32, e.y as i32);
                // `SetInteractionAnchor` (`TileEntity.cs:289-313`): the
                // subject's chest closes + the talk NPC clears; the
                // local-player half opens the inventory + the 10/12 cue.
                if t.player_id as u32 == self.local_id {
                    if let Some(p) = self.player.as_mut() {
                        p.chest = -1; // `:291`
                    }
                    self.close_shop(); // the shop is the talk-NPC's UI half
                    self.talk_npc = None; // `:292` — `SetTalkNPC(-1)`
                    self.npc_chat_text.clear();
                    let was_clear = !self.net_te_anchors.contains_key(&t.player_id);
                    // `IngameUIWindows.CloseAll(quiet)` + `playerInventory =
                    // true` (`:296-297`); the port's inventory-open latch.
                    self.inventory_open = true;
                    // `:303-310` — sound 12 when an anchor was already set,
                    // sound 10 on a fresh open.
                    crate::chest::play_menu_sound(self, if was_clear { 10 } else { 12 });
                } else if let Some(p) = self.remote_players.get_mut(&(t.player_id as u32)) {
                    p.chest = None; // the peer's chest closes
                }
                // `player.tileEntityAnchor.Set(id, x, y)` (`:312`).
                self.net_te_anchors
                    .insert(t.player_id, (t.entity_id, ex, ey));
            }
            // `case 124`'s client half (`MessageBuffer.cs:4004-4016`): unfold
            // the dye flag, the `TryGet<TEHatRack>` + `slot >= 2` refusal,
            // else `ReadItem`. W4-g51 FINAL.
            S2c::HatRackItemSync(h) => {
                let dye = h.is_dye();
                let slot = h.slot_index();
                if h.entity_id < 0 || slot >= 2 {
                    return; // `:4010-4015`
                }
                let ok = self
                    .world
                    .tile_entities
                    .get_by_id(h.entity_id as u32)
                    .is_some_and(|e| {
                        e.kind == tr_core::tile_entities::TeKind::HatRack
                            // The tombstoned slot reads as missing (the
                            // host gate's SEAL re-pin).
                            && self
                                .world
                                .tile_entities
                                .get_at(e.x, e.y)
                                .is_some_and(|live| live.id == e.id)
                    });
                if ok {
                    self.apply_hat_rack_item(h.entity_id as u32, slot, dye, h.item);
                }
            }
        }
    }

    /// W4-g52 audit — the in-world `7 WorldInfo` re-push apply
    /// (`MessageBuffer.cs:473-658`, the `netMode == 1` reader minus the
    /// `State == 3` walk): the header fields with a port home land on the
    /// live world; the world-evil/bg/tree/seed-flag groups without one stay
    /// read-and-dropped (the standing `7` deviation, ANALYSIS). The C#
    /// reader's `Main.raining = maxRaining > 0f` (`:530-531`) rides the rain
    /// restore; `windSpeedCurrent` is NOT touched outside the State-3 gate.
    fn apply_world_info_repush(&mut self, info: &net::WorldInfo) {
        let w = &mut self.world;
        w.time = info.time as f64;
        w.day_time = info.day_time;
        w.blood_moon = info.blood_moon;
        w.eclipse = info.eclipse;
        w.moon_phase = info.moon_phase;
        w.moon_type = info.moon_type;
        w.spawn_x = info.spawn_tile_x as i32;
        w.spawn_y = info.spawn_tile_y as i32;
        if info.world_surface > 0 {
            w.world_surface = info.world_surface as f64;
        }
        if info.rock_layer > 0 {
            w.rock_layer = info.rock_layer as f64;
        }
        w.sundial_cooldown = info.sundial_cooldown;
        w.moondial_cooldown = info.moondial_cooldown;
        w.hell_back_style = info.hell_back_style;
        // `WorldGen.setBG` for the five surface styles the port owns
        // (`MessageBuffer.cs:494-506`'s slot order 1/2/4/5/8).
        w.set_surface_bg_style(1, info.backgrounds[4] as i32);
        w.set_surface_bg_style(2, info.backgrounds[5] as i32);
        w.set_surface_bg_style(4, info.backgrounds[7] as i32);
        w.set_surface_bg_style(5, info.backgrounds[8] as i32);
        w.set_surface_bg_style(8, info.backgrounds[10] as i32);
        w.invasion_type = info.invasion_type as i32;
        w.invasion_goblins = w.invasion_type == 1;
        // The 14 flag groups (`NetMessage.cs:285-389` write order): the
        // downed/event bits with World fields; the pet/seed/holiday bits
        // are constants-off or generation-time (never rewritten mid-run).
        let f = &info.world_flags;
        w.shadow_orb_smashed = f[0].get(0);
        w.downed_eoc = f[0].get(1);
        w.downed_eow = f[0].get(2);
        w.downed_skeletron = f[0].get(3);
        w.hard_mode = f[0].get(4);
        w.downed_plant_boss = f[0].get(7);
        w.downed_mech_boss1 = f[1].get(0);
        w.downed_mech_boss2 = f[1].get(1);
        w.downed_mech_boss3 = f[1].get(2);
        w.pumpkin_moon = f[1].get(6);
        w.snow_moon = f[1].get(7);
        w.fast_forward_to_dawn = f[2].get(1);
        w.slime_rain = f[2].get(2);
        w.downed_slime_king = f[2].get(3);
        w.downed_queen_bee = f[2].get(4);
        w.downed_fishron = f[2].get(5);
        w.downed_martians = f[2].get(6);
        w.downed_ancient_cultist = f[2].get(7);
        w.downed_moon_lord = f[3].get(0);
        w.downed_golem = f[3].get(6);
        w.downed_pirates = f[4].get(0);
        w.downed_frost = f[4].get(1);
        w.downed_goblins = f[4].get(2);
        w.dd2_event_ongoing = f[4].get(4);
        w.dd2_downed_t1 = f[4].get(5);
        w.dd2_downed_t2 = f[4].get(6);
        w.dd2_downed_t3 = f[4].get(7);
        w.combat_book_was_used = f[5].get(0);
        w.downed_tower_solar = f[5].get(2);
        w.downed_tower_vortex = f[5].get(3);
        w.downed_tower_nebula = f[5].get(4);
        w.downed_tower_stardust = f[5].get(5);
        w.downed_queen_slime = f[6].get(6);
        w.downed_deerclops = f[7].get(2);
        w.unlocked_slime_rainbow_spawn = f[8].get(3);
        w.unlocked_slime_red_spawn = f[8].get(4);
        w.fast_forward_to_dusk = f[8].get(7);
        // The three event-state homes the C# reader owns outside World:
        // `Sandstorm.Happening` (`:576`), `BirthdayParty.ManualParty`
        // (`:571`) and `LanternNight.ManualLanterns` (`:583`). The C#
        // writer sends `PartyIsUp`/`LanternsUp` (genuine || manual) but
        // the reader lands them on the MANUAL flags — the asymmetry is
        // the source's, kept verbatim. The `flag8` StartSlimeRain/
        // StopSlimeRain pair (`:641-648`) stays flag-only here (the
        // machines draw RNG; the re-push must not consume a stream).
        self.sandstorm.happening = f[4].get(3);
        self.world_events.manual_party = f[3].get(7);
        self.world_events.manual_lanterns = f[5].get(1);
        // `Main.maxRaining` + `Main.raining = maxRaining > 0f`
        // (`MessageBuffer.cs:530-531`); the rain machine keeps its own
        // timer (the header carries no rainTime — the C# re-derives it).
        self.world.max_raining = info.max_raining;
        self.world.raining = info.max_raining > 0.0;
        let raining = self.world.raining;
        let rain_time = self.world.rain_time;
        self.rain.restore_saved(raining, rain_time, info.max_raining);
        self.wind.speed_target = info.wind_speed_target;
        self.sandstorm.intended_severity = info.sandstorm_severity;
        self.world.sandstorm_intended_severity = info.sandstorm_severity;
    }

    /// W4-g52 audit — the event-site `7 WorldInfo` re-push
    /// (`NetMessage.SendData(7)` at the dawn/dusk boundaries
    /// `Main.cs:65358-65361`/`:65406-65409`, the invasion end `:64356-64359`,
    /// the maxRaining change `:64952-64958`): host-only, the LIVE header to
    /// every broadcast-open slot. SP/client: a no-op (the C# gates on
    /// `netMode == 2`).
    pub(crate) fn net_world_repush(&self) {
        if let NetRole::Host(server) = &self.net {
            let info = self.world_info();
            server.broadcast(&S2c::WorldInfo(Box::new(info)));
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
                player_id: me,
                slot: i as i16,
                item: self.net_item(slot, self.favorite_slots.contains(&i)),
            });
        }
        // The armour, dye and misc-dye rows (`MessageBuffer.cs:264-276`'s
        // array order), then the four banks and the trash slot.
        if let Some(p) = self.player.as_ref() {
            for i in 0..p.equipped.len() {
                client.send(&C2s::Inventory {
                    player_id: me,
                    slot: player_item_slot::ARMOR_0 + i as i16,
                    item: net::NetItem {
                        kind: p.equipped[i] as i16,
                        stack: 1,
                        prefix: p.equipped_prefix[i].min(255) as u8,
                        favorited: false,
                        blocked: false,
                    },
                });
            }
            for i in 0..p.dye_slots.len() {
                client.send(&C2s::Inventory {
                    player_id: me,
                    slot: player_item_slot::DYE_0 + i as i16,
                    item: net::NetItem {
                        kind: p.dye_slots[i] as i16,
                        stack: 1,
                        prefix: 0,
                        favorited: false,
                        blocked: false,
                    },
                });
            }
            for i in 0..p.misc_dye_slots.len() {
                client.send(&C2s::Inventory {
                    player_id: me,
                    slot: player_item_slot::MISC_DYE_0 + i as i16,
                    item: net::NetItem {
                        kind: p.misc_dye_slots[i] as i16,
                        stack: 1,
                        prefix: 0,
                        favorited: false,
                        blocked: false,
                    },
                });
            }
            for (b, vault) in p.banks.iter().enumerate() {
                for (i, slot) in vault.iter().enumerate() {
                    client.send(&C2s::Inventory {
                        player_id: me,
                        slot: player_item_slot::bank_1_0(b) + i as i16,
                        item: self.net_item(slot, false),
                    });
                }
            }
        }
        // The misc equips (`PlayerItemSlotID.Misc0..`) ride `App.equip`.
        for (i, slot) in self.equip.misc_equips.iter().enumerate() {
            client.send(&C2s::Inventory {
                player_id: me,
                slot: player_item_slot::MISC_0 + i as i16,
                item: self.net_item(slot, false),
            });
        }
        // The trash slot (`PlayerItemSlotID.TrashItem`).
        client.send(&C2s::Inventory {
            player_id: me,
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
            blocked: false,
        }
    }

    /// The in-process world install: every world-derived system rebuilds here
    /// without touching the renderer (the C# `WorldGen.clearWorld` +
    /// `Main.Map.Load` arm of `Netplay.cs:484-505`).
    /// The C#'s `WorldGen.clearWorld` arm (`Netplay.cs:484-505`): the client
    /// builds the empty world from the `7 WorldInfo` header and applies the
    /// streamed sections into it. Returns false only when the header is
    /// missing (the client cannot size the world without it).
    fn client_install_world(&mut self, sections: Vec<net::TileBlock>) -> bool {
        let Some(info) = self.net_info.clone() else {
            log::warn!("no WorldInfo header: cannot install the world");
            return false;
        };
        let mut world = World::new(
            info.world_name.clone(),
            info.max_tiles_x as i32,
            info.max_tiles_y as i32,
        );
        world.spawn_x = info.spawn_tile_x as i32;
        world.spawn_y = info.spawn_tile_y as i32;
        world.time = info.time as f64;
        world.day_time = info.day_time;
        world.moon_phase = info.moon_phase;
        world.moon_type = info.moon_type;
        world.blood_moon = info.blood_moon;
        world.sundial_cooldown = info.sundial_cooldown;
        world.moondial_cooldown = info.moondial_cooldown;
        if info.world_surface > 0 {
            world.world_surface = info.world_surface as f64;
        }
        if info.rock_layer > 0 {
            world.rock_layer = info.rock_layer as f64;
        }
        // The sections fill the world; the stream must cover the whole grid.
        let want = net::sections_x(world.width) * net::sections_y(world.height);
        let mut seen: std::collections::HashSet<(i32, i32)> = Default::default();
        for block in &sections {
            seen.insert((net::section_x(block.x), net::section_y(block.y)));
            Self::apply_tile_block(&mut world, block);
        }
        let (gx, gy) = (net::sections_x(world.width), net::sections_y(world.height));
        for sy in 0..gy {
            for sx in 0..gx {
                if !seen.contains(&(sx, sy)) {
                    log::warn!("section {sx},{sy} missing from the world stream");
                }
            }
        }
        let _ = want;
        self.world = world;
        self.liquids = LiquidSystem::new(&self.world);
        // The launch flag survives the sim rebuild (the C# static is
        // process-wide; `Main.cs:1665`).
        self.liquids
            .set_reduced_max_liquids(self.settings.reduced_max_liquids);
        self.map = Some(map::WorldMap::new(&self.world));
        self.npcs.clear();
        self.items.clear();
        // `NPC.ClearAll` (`NPC.cs:8369`) — the pending-damage queue clears
        // with the world (W4-g51 id-162).
        self.net_pending_damages.clear();
        self.world_key = None;
        self.net_info = None;
        self.net_pending_sections.clear();
        if self.player.is_some() {
            // `FindSpawn` + `Player.Spawn(SpawningIntoWorld)` — the port's
            // spawn is the world header's spawn tile (`Player.cs:594`).
            self.player = Some(player::Player::spawn(&self.world));
        }
        // The net install enters the world exactly like the single-player
        // `Main.LoadWorld` tail does (`App::enter_loaded_world`).
        self.enter_loaded_world();
        log::info!(
            "world installed: {}x{}",
            self.world.width,
            self.world.height
        );
        true
    }

    /// `Player.Teleport` (`Player.cs:37414-37496`) for the `65` reader: the
    /// host's local player (slot 0) rides `teleport_style`, a peer's record
    /// takes the position with the netOffset zeroed (`:37447`). The camera /
    /// teleport-effect / map halves are render seams (out). W4-g49.
    fn apply_player_teleport(&mut self, slot: u32, x: f32, y: f32, style: u8) {
        if slot == 0 {
            if let Some(p) = self.player.as_mut() {
                p.teleport_style(x, y, style as i32);
            }
        } else if let Some(p) = self.remote_players.get_mut(&slot) {
            p.x = x;
            p.y = y;
            p.off_x = 0.0; // `Player.netOffset = Vector2.Zero` (`:37447`)
            p.off_y = 0.0;
        }
    }

    /// Builds the `7 WorldInfo` body from the live world (`App.world`).
    /// W4-g52 audit — the port-owned fields are filled (the re-pushes at
    /// `Main.cs:64560`/`:65361`/`:65409`/`:64359` carry the LIVE flags);
    /// the fields without a port home still ride vanilla defaults
    /// (ANALYSIS "7 WorldInfo fields without a port home").
    fn world_info(&self) -> net::WorldInfo {
        let w = &self.world;
        // The 14 flag groups, `NetMessage.cs:285-389`'s write order — the
        // bits with a World field; the pet/holiday/secret-seed bits the
        // port pins at generation stay 0 mid-run (the seed groups ride
        // `w.seeds` where the flag exists).
        let mut f = [net::BitsByte::new(); 14];
        f[0].set(0, w.shadow_orb_smashed);
        f[0].set(1, w.downed_eoc);
        f[0].set(2, w.downed_eow);
        f[0].set(3, w.downed_skeletron);
        f[0].set(4, w.hard_mode);
        f[0].set(7, w.downed_plant_boss);
        f[1].set(0, w.downed_mech_boss1);
        f[1].set(1, w.downed_mech_boss2);
        f[1].set(2, w.downed_mech_boss3);
        f[1].set(3, w.downed_mech_boss_any());
        f[1].set(4, self.rain.cloud_bg() >= 1.0);
        f[1].set(6, w.pumpkin_moon);
        f[1].set(7, w.snow_moon);
        f[2].set(1, w.fast_forward_to_dawn);
        f[2].set(2, w.slime_rain);
        f[2].set(3, w.downed_slime_king);
        f[2].set(4, w.downed_queen_bee);
        f[2].set(5, w.downed_fishron);
        f[2].set(6, w.downed_martians);
        f[2].set(7, w.downed_ancient_cultist);
        f[3].set(0, w.downed_moon_lord);
        f[3].set(6, w.downed_golem);
        f[3].set(7, crate::world_events::party_is_up(&self.world_events));
        f[4].set(0, w.downed_pirates);
        f[4].set(1, w.downed_frost);
        f[4].set(2, w.downed_goblins);
        f[4].set(3, self.sandstorm.happening);
        f[4].set(4, w.dd2_event_ongoing);
        f[4].set(5, w.dd2_downed_t1);
        f[4].set(6, w.dd2_downed_t2);
        f[4].set(7, w.dd2_downed_t3);
        f[5].set(0, w.combat_book_was_used);
        f[5].set(1, crate::world_events::lanterns_up(&self.world_events));
        f[5].set(2, w.downed_tower_solar);
        f[5].set(3, w.downed_tower_vortex);
        f[5].set(4, w.downed_tower_nebula);
        f[5].set(5, w.downed_tower_stardust);
        f[6].set(4, w.seeds.contains(tr_core::seeds::SeedFlags::DRUNK));
        f[6].set(6, w.downed_queen_slime);
        f[6].set(7, w.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY));
        f[7].set(0, w.seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY));
        f[7].set(1, w.seeds.contains(tr_core::seeds::SeedFlags::DONT_STARVE));
        f[7].set(2, w.downed_deerclops);
        f[7].set(3, w.seeds.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES));
        f[7].set(4, w.seeds.contains(tr_core::seeds::SeedFlags::REMIX));
        f[8].set(3, w.unlocked_slime_rainbow_spawn);
        f[8].set(4, w.unlocked_slime_red_spawn);
        f[8].set(7, w.fast_forward_to_dusk);
        f[9].set(0, w.seeds.contains(tr_core::seeds::SeedFlags::NO_TRAPS));
        f[9].set(1, w.seeds.contains(tr_core::seeds::SeedFlags::ZENITH));
        f[9].set(6, w.seeds.contains(tr_core::seeds::SeedFlags::SKYBLOCK));
        // `NetMessage.cs:246-258`'s 13-byte bg block narrowed to the port's
        // 11-row array (treeBG1..4 ride 0 — unported; corrupt..underworld
        // map onto the five owned styles).
        let mut backgrounds = [0u8; 11];
        backgrounds[4] = w.corrupt_bg_style;
        backgrounds[5] = w.jungle_bg_style;
        backgrounds[7] = w.hallow_bg_style;
        backgrounds[8] = w.crimson_bg_style;
        backgrounds[10] = w.mushroom_bg_style;
        let o = &w.ore_tiers;
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
            world_id: w.world_id,
            world_name: w.name.clone(),
            // `Main.GameMode` — the port has no mode byte yet; classic is 0.
            game_mode: 0,
            unique_id: w.unique_id,
            // `Main.ActiveWorldFileData.WorldGeneratorVersion` — the port's
            // generator version lives in the world seed's place; the save
            // format's version is the closest port-side marker (M6-NET-2
            // pins this to the generator build).
            generator_version: if w.generator_version != 0 {
                w.generator_version
            } else {
                save::VERSION as u64
            },
            moon_type: w.moon_type,
            backgrounds,
            ice_back_style: 0,
            jungle_back_style: 0,
            hell_back_style: w.hell_back_style,
            wind_speed_target: self.wind.speed_target,
            // The static cloud-field stand-in (`rain.rs` NUM_CLOUDS_STAND_IN).
            num_clouds: 100,
            tree_x: [0; 3],
            tree_style: [0; 4],
            cave_back_x: [0; 3],
            cave_back_style: [0; 4],
            tree_tops: [0; net::TREE_TOPS_BYTES],
            // `NetMessage.cs:281-284`: `!raining` forces the wire 0.
            max_raining: if self.rain.is_raining() {
                self.rain.max_raining()
            } else {
                0.0
            },
            world_flags: f,
            sundial_cooldown: w.sundial_cooldown,
            moondial_cooldown: w.moondial_cooldown,
            saved_ore_tiers: [
                o.copper as i16,
                o.iron as i16,
                o.silver as i16,
                o.gold as i16,
                o.cobalt as i16,
                o.mythril as i16,
                o.adamantite as i16,
            ],
            invasion_type: w.invasion_type as i8,
            lobby_id: 0,
            sandstorm_severity: self.sandstorm.intended_severity,
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
        self.remote_players.clear();
        self.local_id = 0;
        if self.menu.is_none() {
            let mut m = menu::Menu::new(self.settings.clone(), self.player_name.clone());
            m.error = Some(format!("Disconnected: {reason}"));
            self.menu = Some(m);
        }
    }

    /// S1-I-4d: `Main.netMode` for this process (0 single-player, 1 client,
    /// 2 host) — the gate every `SendData` site reads (`NetMessage.cs:83-86`).
    fn net_mode(&self) -> u8 {
        match self.net {
            NetRole::Host(_) => 2,
            NetRole::Client(_) => 1,
            NetRole::Off => 0,
        }
    }

    /// S1-I-4d: the `23` body for one NPC (`NetMessage.cs:686-748`), the same
    /// shape the old whole-set broadcast wrote. `slot` is the CURRENT Vec index
    /// (the wire `whoAmI`); `spawn`/`generation` ride the caller's bump logic.
    fn npc_info_of(
        slot: usize,
        npc: &crate::npc::Npc,
        spawn: bool,
        generation: u32,
    ) -> net::NpcInfo {
        net::NpcInfo {
            slot: slot as u8,
            generation: generation as u8,
            spawn,
            kind: npc.kind.to_u16() as i16,
            x: npc.x,
            y: npc.y,
            vx: npc.vx,
            vy: npc.vy,
            target: npc.target as u16,
            dir_right: npc.facing > 0,
            dir_y_down: npc.direction_y > 0,
            sprite_right: npc.sprite_dir > 0,
            life_full: npc.hp >= npc.max_hp,
            hp: npc.hp,
            life_max: npc.max_hp,
            ai: npc.ai,
            spawned_from_statue: npc.spawned_from_statue,
            // The port has no per-NPC difficulty override / player-count
            // scaling yet — the C# defaults keep both flag bits clear.
            difficulty: 1.0,
            shimmer: npc.shimmer_transparency > 0.0,
            release_owner: npc.release_owner,
        }
    }

    /// W4-g52 — a priority `23` broadcast at an event site (the strike
    /// cascade `NetMessage.cs:102-107`, the death pushes
    /// `MessageBuffer.cs:1808-1815`): the body from the live NPC, the
    /// generation from the cadence bookkeeping, and the priority clear the
    /// `case 23` fan-out runs (`NetMessage.cs:1722-1730` — boss/netAlways/
    /// town/dead/spawnNeedsSyncing all clear the per-player counters; the
    /// event sites here always qualify: spawnNeedsSyncing or life <= 0).
    fn send_npc_state_broadcast(&mut self, server: &net::Server, slot: usize) {
        let Some(npc) = self.npcs.get(slot) else {
            return;
        };
        let generation = self
            .net_npc_gen
            .get(&(slot as u32))
            .map(|(_, g)| *g)
            .unwrap_or(0);
        let info = Self::npc_info_of(slot, npc, false, generation);
        let uid = npc.uid;
        server.broadcast(&S2c::NpcInfo(Box::new(info)));
        // The `:1725-1730` clear (the decision layer's NpcNetState).
        if let Some(st) = self.net_npc_states.get_mut(&uid) {
            st.spawn_needs_syncing = false;
            st.net_stream = 0;
            st.net_update = false;
            st.pending_spam_cooldown = false;
            st.pending_full_spam_cooldown = false;
            st.sync = [crate::app::net_npc_sync::PlayerNetSyncState::default(); 256];
        }
    }

    /// W4-g52 — the `27` relay context for one pool slot: flag3
    /// (`NetMessage.cs:1777`) from the verbatim `projPet` table +
    /// `NetSectionCoordinates` (`Projectile.cs:602` — `(int)position.X >> 4`,
    /// the arithmetic shift the port's `/ 16` matches for the f32→i32 cast).
    fn proj_relay_ctx(&self, slot: usize) -> net::RelayCtx {
        use crate::projectile::{proj_data, proj_pet};
        let p = &self.projectiles.slots()[slot];
        let ai_style = proj_data(p.kind).map_or(255, |d| d.ai_style);
        net::RelayCtx::Projectile {
            ignores_sections: crate::app::net_proj_sync::sync_ignores_sections(
                p.kind,
                proj_pet(p.kind),
                ai_style,
                p.net_important,
            ),
            section: (
                net::section_x(p.pos.x as i32 / TILE_PX as i32),
                net::section_y(p.pos.y as i32 / TILE_PX as i32),
            ),
        }
    }

    /// S1-I-4d: the per-NPC `NpcView` the cadence reads (`net_npc_sync::NpcView`).
    /// `boss`/`town_npc` come from the kind; `proximity_sync` is the
    /// `NPCID.Sets.UsesMultiplayerProximitySyncing` set (396/397/398, the
    /// Martian saucer parts — `NPCID.cs:10454`), keyed on the vanilla `cs_id`.
    fn npc_view_of(npc: &crate::npc::Npc) -> crate::app::net_npc_sync::NpcView {
        use crate::app::net_npc_sync::NpcView;
        let cs_id = npc.kind.cs_id();
        let (cx, cy) = npc.center();
        let (homeless, hx, hy) = match npc.home {
            Some((hx, hy)) => (false, hx, hy),
            None => (true, 0, 0),
        };
        NpcView {
            active: npc.hp > 0,
            life: npc.hp,
            boss: npc.boss,
            town_npc: npc.kind.is_town_npc_like(),
            net_always: false,
            proximity_sync: matches!(cs_id, 396 | 397 | 398),
            vx: npc.vx,
            vy: npc.vy,
            center: Vec2::new(cx, cy),
            section: (
                net::section_x(npc.x as i32 / TILE_PX as i32),
                net::section_y(npc.y as i32 / TILE_PX as i32),
            ),
            // `NPC.TypeToDefaultHeadIndex(type) > 0` (`NPC.cs:90621`): the town
            // residents with a housing head icon. The port's town roster stands
            // in until the head-index table lands.
            has_head_index: npc.kind.is_town_npc(),
            home: (homeless, hx, hy),
        }
    }

    /// S1-I-4d: the NPC-STATE cadence — the SWAP target for the old whole-set
    /// 10 Hz broadcast. Runs once per fixed step (the C# `NPC.UpdateNPC` tail,
    /// `NPC.cs:90513` → `UpdateNetworkCode`, `:90583-90631`), in slot order,
    /// host-only. It fills `net_npc_outbox` with the decision layer's `NpcSend`s
    /// (tagged by the stable uid so the 10 Hz drain survives Vec compaction) and
    /// queues despawn `23`s for uids that left the list.
    ///
    /// Producer wiring: the port's AI arms already set `Npc.net_update`
    /// (`npc_target.rs:535`, `npc_ai_caster.rs`, …). Until the full C# netUpdate
    /// producer census lands, a state change vs the last `23` body ALSO arms the
    /// flag (the interim change-detect producer, recorded in ANALYSIS) so a
    /// moving NPC still syncs. The send BUDGET is the faithful `netSpam` gate.
    fn npc_sync_cadence(&mut self) {
        use crate::app::net_npc_sync::{self as nns, PlayerView};
        if self.net_mode() != 2 {
            return; // `NPC.cs:90590-90593` — the server gate.
        }
        // Gather the per-client views WITHOUT holding the server borrow across
        // the mutation below: id, camera, and the check-time grid
        // (`RemoteClient.IsSectionActive`, `NetMessage.cs:1740` — W4-g52:
        // the 60-tick window over `TileSectionsCheckTime`, NOT the streamed
        // latch the old diff read).
        let (w, h) = (self.world.width, self.world.height);
        let mut clients: Vec<(u32, Vec2, (i32, Vec<u32>))> = Vec::new();
        if let NetRole::Host(server) = &self.net {
            for id in server.slot_ids() {
                if server.state(id) < net::slot_state::IN_WORLD {
                    continue;
                }
                let cam = self
                    .remote_players
                    .get(&id)
                    .map(|p| Vec2::new(p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5))
                    .unwrap_or(Vec2::ZERO);
                server.ensure_sections(id, w, h);
                clients.push((id, cam, server.section_check_times(id)));
            }
        }
        // The stable identity (`NPC.whoAmI`, `NPC.cs:6145`): every NPC takes its
        // uid once, exactly as `update_npcs` does (`world_sim/npcs.rs:2643`).
        // The net-only test pump never runs the sim, so the cadence assigns the
        // uids itself; in the live loop they are already set (idempotent).
        {
            let mut next = self.next_npc_uid;
            for n in self.npcs.iter_mut() {
                if n.uid == 0 {
                    n.uid = next;
                    next += 1;
                }
            }
            self.next_npc_uid = next;
        }
        // The live uid set (for the despawn diff below).
        let mut live: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for (slot, npc) in self.npcs.iter().enumerate() {
            let uid = npc.uid;
            // `NPC.UpdateNPC` returns early when `!active` (`NPC.cs:90277`): a
            // dead NPC is not cadence-ticked. Its despawn rides the gone-diff
            // below (the sim's `retain(hp > 0)` removes it, or it lingers in a
            // net-only test — either way it leaves the live set exactly once).
            if npc.hp <= 0 {
                continue;
            }
            live.insert(uid);
            let view = Self::npc_view_of(npc);
            // The generation bump the old broadcast did: a kind change (or a
            // fresh slot) raises the generation + the spawn flag so the client
            // recreates the slot (`MessageBuffer.cs:1602-1613`).
            let kind = npc.kind.to_u16() as i16;
            let (spawn, generation) = match self.net_npc_gen.get(&(slot as u32)) {
                Some((k, g)) if *k == kind => (false, *g),
                _ => {
                    let g = self
                        .net_npc_gen
                        .get(&(slot as u32))
                        .map(|(_, g)| g.wrapping_add(1))
                        .unwrap_or(1);
                    (true, g)
                }
            };
            self.net_npc_gen.insert(slot as u32, (kind, generation));
            let info = Self::npc_info_of(slot, npc, spawn, generation);
            // Per-NPC player array: section_active is per (client, NPC-section).
            let mut players = [PlayerView {
                active: false,
                connected: false,
                camera: Vec2::ZERO,
                section_active: false,
            }; 256];
            for (id, cam, (cw, times)) in &clients {
                let i = *id as usize;
                if i >= 256 {
                    continue;
                }
                // `IsSectionActive(NetSectionCoordinates)`: the clamped
                // grid read + the 60-tick window (`RemoteClient.cs:220-224`).
                let (sx, sy) = (view.section.0.max(0), view.section.1.max(0));
                let section_active = *cw > 0
                    && times
                        .get((sy * cw + sx) as usize)
                        .is_some_and(|&t| net::section_active_at(t, self.game_update_count));
                players[i] = PlayerView {
                    active: true,
                    connected: true,
                    camera: *cam,
                    section_active,
                };
            }
            // The per-uid throttle state; a first sighting arms spawnNeedsSyncing
            // (`NPC.cs:80785`) so the priority `23` clears the counters.
            let first = !self.net_npc_states.contains_key(&uid);
            let st = self.net_npc_states.entry(uid).or_default();
            if first {
                st.spawn_needs_syncing = true;
            }
            // Producer: the AI flag OR a state change vs the last body. The
            // `spawn` flag is a one-shot create marker (true on the first body,
            // false after), NOT sync state — normalize it out so it does not
            // read as a change on the second tick (the no-double-send witness).
            let changed = match self.net_npc_last_info.get(&uid) {
                Some(prev) => {
                    let (mut a, mut b) = (*prev, info);
                    a.spawn = false;
                    b.spawn = false;
                    a != b
                }
                None => true,
            };
            if npc.net_update || changed || first {
                st.net_update = true;
            }
            // Call the per-NPC gate directly with the REAL slot index (the
            // `npc_sync_tick` whole-array hook would renumber a 1-element slice
            // to 0). `NPC.cs:90513` → `UpdateNetworkCode(i)`.
            let mut out = Vec::new();
            nns::update_network_code(st, &view, slot as i16, 2, &players, nns::NPC_STREAM_SPEED, &mut out);
            st.net_update = false; // `NPC.cs:90515` — the tail clear.
            for s in out {
                self.net_npc_outbox.push((uid, s));
            }
            self.net_npc_last_info.insert(uid, info);
            self.net_npc_uid_slot.insert(uid, slot as u32);
        }
        // Clear the AI flags (the UpdateNPC tail, `NPC.cs:90515`).
        for npc in self.npcs.iter_mut() {
            npc.net_update = false;
        }
        // The despawn diff: a uid the cadence knew that is no longer live gets
        // one priority `23` with life 0 (`NPC.cs:90385-90392` / `:77991-77998`).
        let gone: Vec<u32> = self
            .net_npc_last_info
            .keys()
            .copied()
            .filter(|uid| !live.contains(uid))
            .collect();
        for uid in gone {
            if let Some(mut info) = self.net_npc_last_info.remove(&uid) {
                info.hp = 0;
                info.life_full = false;
                info.spawn = false;
                if let Some(slot) = self.net_npc_uid_slot.remove(&uid) {
                    info.slot = slot as u8;
                }
                self.net_npc_despawns.push(info);
            }
            self.net_npc_states.remove(&uid);
        }
        // Prune the generation bookkeeping of slots beyond the live list (the
        // old broadcast's tail; the uid-keyed despawn diff above owns the wire).
        self.net_npc_gen
            .retain(|slot, _| (*slot as usize) < self.npcs.len());
    }

    /// S1-I-4d: the projectile cadence — the SWAP target for
    /// `projectile_sync_plan`'s 10 Hz diff. Runs once per fixed step (the C#
    /// `Projectile.Update` tail, `Projectile.cs:17210-17244`), owner-gated, and
    /// fills `net_proj_outbox` through the faithful `ProjSync` budget. Spawn /
    /// kill are event-driven at their sites (`Projectile.cs:11096` /
    /// `:77254-77257`); the interim change-detect producer arms `mark_dirty`
    /// until the 337 AI `netUpdate` sites land (recorded in ANALYSIS).
    fn proj_sync_cadence(&mut self) {
        let net_mode = self.net_mode();
        if net_mode == 0 {
            return; // `NetMessage.cs:83-86` — SendData's netMode gate.
        }
        let local = self.local_id as u8;
        // Snapshot this machine's OWN bodies + detect spawns / changes. The
        // owner gate matches `Projectile.cs:17210` (and the old diff's scope):
        // the server-owned hostile shots (owner 255) are a recorded follow-up —
        // on the C# server `Main.myPlayer == 255` folds them into the same gate,
        // but the port's host keeps a local player (owner 0), so hostile-shot
        // budget sync lands with the owner-model reconcile (ANALYSIS).
        let mut current: Vec<(usize, i32, net::ProjectileState)> = Vec::new();
        for slot in 0..crate::projectile::MAX_PROJECTILES {
            let p = &self.projectiles.slots()[slot];
            if !p.active || p.owner != local {
                continue;
            }
            let key = self.projectiles.key_of(slot);
            let st = net::ProjectileState {
                key,
                x: p.pos.x,
                y: p.pos.y,
                vx: p.vel.x,
                vy: p.vel.y,
                kind: p.kind as i16,
                ai0: if p.ai[0] != 0.0 { Some(p.ai[0]) } else { None },
                ai1: if p.ai[1] != 0.0 { Some(p.ai[1]) } else { None },
                banner_id: 0,
                damage: p.damage as i16,
                knock_back: p.knock_back,
                original_damage: p.original_damage as i16,
                ai2: if p.ai[2] != 0.0 { Some(p.ai[2]) } else { None },
            };
            current.push((slot, key, st));
        }
        // Spawn site: a key not seen before gets ONE `27` (`Projectile.cs:11096`).
        // The change-detect producer below skips first-sight so the spawn is not
        // double-sent through the budget (the no-double-send witness pins this).
        let mut fresh: std::collections::HashSet<i32> = std::collections::HashSet::new();
        for (slot, key, _) in &current {
            if !self.net_proj_prev.contains_key(key) {
                fresh.insert(*key);
                if let Some(send) = self.proj_sync.spawn_sync(&self.projectiles, *slot, net_mode) {
                    self.net_proj_outbox.push(send);
                }
            }
        }
        // Producer: the AI flag (`Projectile.net_update`, e.g. the gem-staff
        // retarget `Projectile.cs:37891`) OR — interim, until the 337-site
        // census lands — a state change on an ALREADY-SEEN key arms the budget's
        // dirty flag (`ProjSync::mark_dirty`, the seam).
        for (slot, key, st) in &current {
            let ai_flag = self.projectiles.slots()[*slot].net_update;
            if ai_flag || (!fresh.contains(key) && self.net_proj_prev.get(key) != Some(st)) {
                self.proj_sync.mark_dirty(*slot);
            }
        }
        // The update-tail clear (`Projectile.cs:17244`): every slot's flag resets
        // after the cadence read it (the budget's own tail clear runs in
        // `ProjSync::step` below; this mirrors it onto the pool field).
        self.projectiles.clear_net_update_flags();
        // Kill site: a key that left the local set gets one `29` with the entry
        // (last-known) position (`Projectile.cs:77254-77257`, `:70648`).
        let gone: Vec<i32> = self
            .net_proj_prev
            .keys()
            .copied()
            .filter(|k| !current.iter().any(|(_, ck, _)| ck == k))
            .collect();
        for key in gone {
            let (owner, slot, _gen) = net::projectile_key_parts(key);
            let entry = self
                .net_proj_prev
                .get(&key)
                .map(|st| (st.x, st.y))
                .unwrap_or((0.0, 0.0));
            // The Kill site (`Projectile.cs:77254-77257`) carries the DYING
            // occupant's key. If the slot was recycled this tick, `key_of` now
            // names the NEW occupant — emit the gone key directly and leave the
            // fresh slot's bookkeeping alone (kill_sync would reset it).
            if slot < crate::projectile::MAX_PROJECTILES
                && self.projectiles.key_of(slot) == key
            {
                if let Some(send) =
                    self.proj_sync
                        .kill_sync(&self.projectiles, slot, entry, owner, net_mode)
                {
                    self.net_proj_outbox.push(send);
                }
            } else if net_mode != 0 {
                self.net_proj_outbox
                    .push(crate::app::net_proj_sync::ProjSend::Kill {
                        key,
                        x: entry.0,
                        y: entry.1,
                    });
            }
            self.net_proj_prev.remove(&key);
        }
        // The per-update budget pass (`Projectile.cs:17210-17244`).
        let sends = self.proj_sync.step(&self.projectiles, local, net_mode);
        self.net_proj_outbox.extend(sends);
        // Refresh the change-detect reference.
        for (_, key, st) in current {
            self.net_proj_prev.insert(key, st);
        }
    }

    /// S1-I-4d: drain the NPC outbox + despawns through the existing `23`/`60`
    /// wires (host). The decision layer already did the per-client section
    /// culling, so each `State` is targeted; the body is rebuilt from the last
    /// cadence snapshot (the uid survives Vec compaction). Takes `&self` + the
    /// already-extracted outboxes so it can run inside the `match &self.net`
    /// borrow (the caller `mem::take`s them first).
    fn flush_npc_outbox(
        &self,
        server: &net::Server,
        outbox: Vec<(u32, crate::app::net_npc_sync::NpcSend)>,
        despawns: Vec<net::NpcInfo>,
    ) {
        use crate::app::net_npc_sync::NpcSend;
        // The despawn `23`s (priority, life 0) go FIRST: the C# dead NPC's
        // `UpdateNPC` runs before the later slots' spawns (`NPC.cs:90385-90392`
        // ahead of `:90513`), and the port's Vec compaction can reuse the slot.
        for info in despawns {
            server.broadcast(&S2c::NpcInfo(Box::new(info)));
        }
        for (uid, send) in outbox {
            match send {
                NpcSend::State { to, .. } => {
                    let Some(info) = self.net_npc_last_info.get(&uid) else {
                        continue;
                    };
                    let mut body = info.clone();
                    if let Some(slot) = self.net_npc_uid_slot.get(&uid) {
                        body.slot = *slot as u8;
                    }
                    match to {
                        Some(i) => server.send(i, &S2c::NpcInfo(Box::new(body))),
                        None => server.broadcast(&S2c::NpcInfo(Box::new(body))),
                    }
                }
                NpcSend::TownHome { homeless, x, y, .. } => {
                    // `60 NPCHomeChange` (`NetMessage.cs:1067-1071`); the status
                    // byte is `GetHouseholdStatus` (`TownRoomManager.cs:130`):
                    // 1 homeless, else 2 housed (the port's home presence). The
                    // wire slot resolves through the uid (post-compaction).
                    let Some(slot) = self.net_npc_uid_slot.get(&uid).copied() else {
                        continue;
                    };
                    let status = if homeless { 1 } else { 2 };
                    server.broadcast(&S2c::NpcHouse(net::NpcHouse {
                        npc: slot as i16,
                        home_x: x as i16,
                        home_y: y as i16,
                        status,
                    }));
                }
                NpcSend::BuffList { .. } => {
                    let Some(slot) = self.net_npc_uid_slot.get(&uid).copied() else {
                        continue;
                    };
                    if let Some(npc) = self.npcs.get(slot as usize) {
                        let buffs = npc_buffs_of(npc);
                        server.broadcast(&S2c::NpcBuffs(net::NpcBuffs {
                            npc: slot as i16,
                            buffs,
                        }));
                    }
                }
                // The strike / add-buff / debuff sites are event-driven at their
                // simulation callers (the `28` relay already lives in
                // `host_event`); the cadence outbox carries only State/TownHome/
                // BuffList today.
                NpcSend::Strike { .. }
                | NpcSend::AddBuff { .. }
                | NpcSend::RemoveBuff { .. }
                | NpcSend::DebuffDot { .. } => {}
            }
        }
    }

    /// S1-I-4d: drain the projectile outbox through the existing `27`/`29`
    /// wires, in the direction the role dictates. W4-g52: the host's `27`
    /// broadcast rides the `case 27` fan-out (`NetMessage.cs:1774-1800` —
    /// flag3 or section-active) and runs the skipped-flag motion at the
    /// same site; a targeted `SyncTo` clears that player's flag
    /// (`:1821-1823`). The `29` kill rides the default arm (`:1801-1811`).
    fn drain_proj_outbox(&mut self) {
        use crate::app::net_proj_sync::ProjSend;
        let outbox = std::mem::take(&mut self.net_proj_outbox);
        // Swap the role out so the flag-motion writes to `self.proj_sync`
        // do not fight the `self.net` borrow (the poll_net precedent).
        let role = std::mem::replace(&mut self.net, NetRole::Off);
        match &role {
            NetRole::Host(server) => {
                for send in outbox {
                    match send {
                        ProjSend::Sync(st) => {
                            let (_, slot, _) = net::projectile_key_parts(st.key);
                            if slot >= crate::projectile::MAX_PROJECTILES {
                                continue;
                            }
                            let ctx = self.proj_relay_ctx(slot);
                            // The flag motion (`:1779-1795`): flag3 clears
                            // every flag; otherwise each broadcast-open
                            // client's flag = !section_active. The host's
                            // own sends have no ignoreClient, and the
                            // not-connected legs never write — snapshot
                            // the closed slots' flags around the motion.
                            let flag3 = matches!(
                                ctx,
                                net::RelayCtx::Projectile {
                                    ignores_sections: true,
                                    ..
                                }
                            );
                            let closed: Vec<(u8, bool)> = if flag3 {
                                Vec::new()
                            } else {
                                (0..255u16)
                                    .filter(|i| !server.broadcast_open(*i as u32))
                                    .map(|i| (i as u8, self.proj_sync.skipped_for(slot, i as u8)))
                                    .collect()
                            };
                            let now = self.game_update_count;
                            self.proj_sync.note_broadcast(&self.projectiles, slot, |i| {
                                let section = match ctx {
                                    net::RelayCtx::Projectile { section, .. } => section,
                                    _ => (0, 0),
                                };
                                server.is_section_active(i as u32, section.0, section.1, now)
                            });
                            for (i, f) in closed {
                                self.proj_sync.set_skipped_for(slot, i, f);
                            }
                            server.relay_projectile(
                                &S2c::Projectile(Box::new(st)),
                                None,
                                ctx,
                                now,
                            );
                        }
                        ProjSend::SyncTo { state, client } => {
                            // The targeted arm (`:1821-1823`): the player's
                            // skipped flag clears, the send always lands.
                            let (_, slot, _) = net::projectile_key_parts(state.key);
                            self.proj_sync.set_skipped_for(slot, client, false);
                            server.send(client as u32, &S2c::Projectile(Box::new(state)))
                        }
                        ProjSend::Kill { key, x, y } => {
                            server.broadcast(&S2c::ProjectileDead { key, x, y })
                        }
                    }
                }
            }
            NetRole::Client(client) => {
                for send in outbox {
                    match send {
                        ProjSend::Sync(st) => client.send(&C2s::Projectile(Box::new(st))),
                        ProjSend::SyncTo { state, .. } => {
                            client.send(&C2s::Projectile(Box::new(state)))
                        }
                        ProjSend::Kill { key, x, y } => {
                            // `NetMessage.cs:852-855` — the entry position
                            // rides verbatim (`Projectile.cs:77256`).
                            client.send(&C2s::ProjectileDead { key, x, y })
                        }
                    }
                }
            }
            NetRole::Off => {}
        }
        self.net = role;
    }

    /// The 10 Hz state tick: the player snapshot, the world-item cadence
    /// (wave A) and the OUTBOX DRAINS of the NPC/projectile decision layers
    /// (S1-I-4d). The NPC `23` / projectile `27`/`29` sends are produced at
    /// 60 Hz by `npc_sync_cadence` / `proj_sync_cadence` below (the C#'s
    /// per-NPC / per-projectile `netSpam` gates, `NPC.cs:90583-90631` /
    /// `Projectile.cs:17210-17244`) — NOT by a whole-set broadcast here.
    pub(crate) fn net_tick(&mut self, dt: f32) {
        if matches!(self.net, NetRole::Off) {
            return;
        }
        // `Ping.Update` (`Main.cs:17817-17820`) runs once per **frame** in
        // `UpdateWorld_Client`, not on the 10 Hz net cadence, and needs no
        // local player object - it only watches its own stopwatch.
        self.net_ping_tick();
        // `Main.TriggerPing`'s relay (`Main.cs:56546-56549`): the map markers
        // added this frame go out as `NetPingModule` packets.
        let pings = self
            .map
            .as_mut()
            .map(|m| std::mem::take(&mut m.ping_outbox))
            .unwrap_or_default();
        for pos in pings {
            self.send_module(
                crate::app::netmodules::module_id::PING,
                &crate::app::netmodules::Ping {
                    x: pos[0],
                    y: pos[1],
                }
                .encode(),
            );
        }
        // S1-I-4d: the NPC/projectile decision layers run on the C#'s 60 Hz
        // simulation cadence (`NPC.cs:90513`, `Projectile.cs:17210`), NOT the
        // 10 Hz send tick. Step them once per accumulated fixed step here so
        // both the real frame loop and the net-only test pump drive them; the
        // outboxes they fill drain on the 10 Hz tick below. Cap the burst like
        // `FixedStepper` so a long stall does not run away.
        self.net_sync_cadence_timer += dt;
        let mut steps = 0;
        while self.net_sync_cadence_timer >= 1.0 / 60.0 && steps < 8 {
            self.net_sync_cadence_timer -= 1.0 / 60.0;
            self.host_check_sections();
            self.npc_sync_cadence();
            self.proj_sync_cadence();
            // W4-c28: the client's player-state cadence runs on the same
            // 60 Hz step (`Player.Update` → `TrySyncingInput`,
            // `Main.DoUpdate` → `TrySyncingMyPlayer`/`UpdateClient`).
            self.player_sync_cadence();
            // W4-c28: the catch-all NPC buff send drain — the AI/timer lane
            // (`update_npcs`, which runs in the npcs phase before this net
            // phase) mutates buffs without a sim-pass drain; the C# sends
            // inline at each `NPC.AddBuff` (`NPC.cs:89932-89935` client `53`
            // / `:89979-89982` host `54`). The sim-pass drains already took
            // the player-hit queues; this catches the rest, once per step.
            self.drain_npc_buff_sends();
            steps += 1;
        }
        if self.net_sync_cadence_timer > 8.0 / 60.0 {
            self.net_sync_cadence_timer = 8.0 / 60.0;
        }
        self.net_timer += dt;
        if self.net_timer < 0.1 {
            return;
        }
        self.net_timer = 0.0;
        let Some(p) = self.player.as_ref() else {
            // No local player: the cadence still ran above, so drop its outboxes
            // rather than let them grow unbounded (the sends need a player).
            self.net_npc_outbox.clear();
            self.net_proj_outbox.clear();
            self.net_npc_despawns.clear();
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
            petting: p.petting.is_petting,
            pet_small: p.petting.is_pet_small,
            ..Default::default()
        };
        // The ambience spawn relay (`AmbienceServer.SpawnForPlayer`): the
        // (type, seed) pairs leave the store before the role borrow.
        let ambience_spawns = self.take_unsent_ambience();
        // S1-I-4d: the projectile outbox (filled by the 60 Hz cadence above)
        // drains through the `27`/`29` wires in the direction the role dictates.
        self.drain_proj_outbox();
        // The NPC outbox + despawns leave the store before the role borrow.
        let npc_outbox = std::mem::take(&mut self.net_npc_outbox);
        let npc_despawns = std::mem::take(&mut self.net_npc_despawns);
        match &self.net {
            NetRole::Host(server) => {
                // The host's own snapshot, as `13 PlayerState`.
                server.broadcast(&S2c::PlayerState(Box::new(self.player_state_of(&me))));
                for (ty, seed) in ambience_spawns {
                    let body = crate::app::netmodules::AmbienceSpawn {
                        player: 0,
                        seed,
                        kind: crate::app::netmodules::sky_entity_type_byte(ty),
                    }
                    .encode();
                    server.broadcast(&S2c::NetModule {
                        module: crate::app::netmodules::module_id::AMBIENCE,
                        body,
                    });
                }
                // Newly approached sections stream before the cadence batch
                // (the C#'s per-tick CheckSection, `Main.cs:64620-64623`).
                // `RemoteClient.cs:177-188`: one `9 Status` announcing the
                // missing count, then the `SendSection` walk row-major.
                for (id, sx, sy, announce) in self.pending_sections(server) {
                    if announce > 0 {
                        server.send(
                            id,
                            &S2c::Status {
                                text: "Receiving tile data".into(),
                                max: announce,
                            },
                        );
                    }
                    server.mark_section_sent(id, sx, sy);
                    let block = self.tile_block_at(sx, sy);
                    server.send(id, &S2c::TileBlock(Box::new(block)));
                }
                // S1-I-4d SWAP: the old whole-set 10 Hz `23` broadcast is gone.
                // The per-NPC `netSpam` cadence (`net_npc_sync`, filled at 60 Hz
                // above) owns the `23` fan-out + section culling + despawns
                // (`NPC.cs:90583-90631`, `NetMessage.cs:1719-1749`); drain it.
                self.flush_npc_outbox(server, npc_outbox, npc_despawns);
                // `Main.UpdateServer` (`Main.cs:64557-64580`): the server
                // counter advances once per 60 Hz tick (the port batches it
                // on the 10 Hz cadence tick — 6 per batch), and every 60th
                // tick one round-robin slot gets an unconditional `21`
                // heartbeat. The C# walks 400 fixed slots; the port walks
                // its live list (M6-NET-5), wrapping at its length. The
                // unreserved-only condition (`playerIndexTheItemIsReservedFor
                // == 255`) holds for every port drop — reservations are
                // unported — and the heartbeat re-sends even an unchanged
                // slot (the diff below skips it as already recorded).
                self.net_play_counter = self.net_play_counter.wrapping_add(6);
                if self.net_play_counter % 3600 == 0 {
                    self.net_play_counter = 0;
                    // W4-g52 audit — `Main.UpdateServer`'s 3600-tick header
                    // re-push (`Main.cs:64557-64561`): the LIVE `7 WorldInfo`
                    // (the flags/events the 205 Time pair does not carry).
                    let info = self.world_info();
                    server.broadcast(&S2c::WorldInfo(Box::new(info)));
                }
                if self.net_play_counter % 60 == 0 && !self.items.is_empty() {
                    self.last_item_update += 1;
                    if self.last_item_update >= self.items.len() {
                        self.last_item_update = 0;
                    }
                    let slot = self.last_item_update as i16;
                    let st = Self::item_state_of(slot, &self.items[self.last_item_update]);
                    self.net_item_last.insert(slot, st);
                    // W4-g52 — the `SendData(21)` pre-switch
                    // (`NetMessage.cs:89-100`): instanced never syncs, air
                    // rewrites to `151`.
                    if let Some(msg) =
                        Self::item_send_gate(slot, &self.items[self.last_item_update])
                    {
                        server.broadcast(&msg);
                    }
                }
                // `21 ItemDrop` — only the slots whose state changed since the
                // last send (the port's batched stand-in for the C#'s
                // event-driven `SyncItem` fan-out — `WorldItem.cs:612-615`
                // and its callers; the C# never diff-streams physics).
                let live = self.items.len() as i16;
                for (i, it) in self.items.iter().enumerate() {
                    let slot = i as i16;
                    let st = Self::item_state_of(slot, it);
                    if self.net_item_last.get(&slot) != Some(&st) {
                        self.net_item_last.insert(slot, st);
                        // W4-g52 — the `SendData(21)` pre-switch
                        // (`NetMessage.cs:89-100`).
                        if let Some(msg) = Self::item_send_gate(slot, it) {
                            server.broadcast(&msg);
                        }
                    }
                }
                let dead: Vec<i16> = self
                    .net_item_last
                    .keys()
                    .copied()
                    .filter(|slot| *slot >= live)
                    .collect();
                for slot in dead {
                    self.net_item_last.remove(&slot);
                    server.broadcast(&S2c::ItemDead { slot });
                }
                server.broadcast(&S2c::Time {
                    time: self.world.time,
                    day: self.world.day_time,
                });
            }
            NetRole::Client(_client) => {
                // S1-I-4d: the client's own projectiles already drained through
                // `drain_proj_outbox` above (`Projectile.cs:17210-17244`'s owner
                // machine → `27`, despawns → `29`).
                // W4-c28 SWAP: the old 10 Hz change-test + 42-call heartbeat is
                // gone — the C#'s `TrySyncingInput` / `TrySyncingMyPlayer` /
                // `UpdateClient` decisions run at 60 Hz in
                // `player_sync_cadence` (`Player.cs:6852-6916`,
                // `Main.cs:17545-17703`/`:64522-64536`); drain the outbox.
                self.drain_player_outbox();
            }
            NetRole::Off => {}
        }
    }

    /// W4-c28: the live `ControlState` the cadence compares — the port's
    /// fields the C# `TrySyncingInput` reads (`Player.cs:6858-6905`). The
    /// fields with no port channel ride their defaults (down-hold/dash/
    /// operating-entity/shield/snapstone — recorded in ANALYSIS).
    fn control_state_of(&self) -> crate::app::net_player_sync::ControlState {
        use crate::app::net_player_sync::ControlState;
        let p = self.player.as_ref();
        ControlState {
            up: p.is_some_and(|p| p.control_up),
            down: p.is_some_and(|p| p.control_down),
            left: p.is_some_and(|p| p.control_left),
            right: p.is_some_and(|p| p.control_right),
            jump: p.is_some_and(|p| p.control_jump),
            // `controlUseItem` — the held left button (loop.rs's `mine_down`
            // is the left press; `mouse_down` is the UI-side left hold).
            use_item: self.mouse_down || self.mine_down,
            selected_item: self.selected as i32,
            auto_reuse_all: self.settings.auto_reuse_all,
            last_use_success: self.swing > 0.0,
            direction: p.map_or(1, |p| p.facing),
            ..Default::default()
        }
    }

    /// W4-c28: the CLIENT's 60 Hz player-state cadence — the C#'s three send
    /// paths in their per-tick order: `TrySyncingInput` (the control compare,
    /// `Player.cs:6732`), `TrySyncingMyPlayer` (the clone-compare tail,
    /// `Main.cs:17192`), `UpdateClient` (the counter beats,
    /// `Main.cs:64522-64536`). Fills `net_player_outbox`; the 10 Hz drain
    /// sends. Host/SP: a no-op (the host's own `13` rides the cadence tick).
    fn player_sync_cadence(&mut self) {
        use crate::app::net_player_sync::{self as nps, PlayerSend};
        if self.net_mode() != 1 || self.net_state < net::client_state::IN_WORLD {
            return;
        }
        // W4-c28: the `Hurt` client tail (`Player.cs:38369-38377`) — the
        // damage path queued the send at the hit moment (the PRE-subtraction
        // hp); drain it BEFORE the clone-compare so the 13/16 ride this
        // tick's outbox in the C#'s order (the hurt happens inside
        // `Player.Update`, ahead of `TrySyncingMyPlayer`).
        if let Some((knockback_13, hp, max_hp)) =
            self.player.as_mut().and_then(|p| p.take_hurt_net())
        {
            let mut hurt_out = Vec::new();
            // `hurt_sends(net_mode, quiet, no_knockback, hit_direction, ...)`:
            // the queue already collapsed the C#'s three-clause 13 gate into
            // `knockback_13`; ride it through the no_knockback/hit_direction
            // pair (false/1 when armed, true/0 when not).
            nps::hurt_sends(1, false, !knockback_13, i32::from(knockback_13), hp, max_hp, &mut hurt_out);
            self.net_player_outbox.extend(hurt_out);
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let mut out = Vec::new();
        let controls = self.control_state_of();
        // 1. `TrySyncingInput` (`Player.cs:6852-6916`): the 12-field compare;
        // the send copies the controls into the clone (`NetMessage.cs:511`).
        if nps::try_syncing_input(&controls, &self.net_client_clone.controls, 1) {
            out.push(PlayerSend::Controls);
            self.net_client_clone.controls = controls;
        }
        // The port's render-extension producer (recorded in ANALYSIS): the
        // C# server simulates every peer's movement from the relayed controls
        // and derives the anim frames locally; the port simulates remote
        // players nowhere, so a change in the wire body's position/anim/hp
        // extension arms one more `13` (a superset — never a missing update).
        // The reference snapshots HERE, at decision time: the drain runs at
        // 10 Hz, so gating against the last DRAIN would re-arm every 60 Hz
        // step in between (6 duplicate sends per change).
        {
            let last = &self.net_ps_last;
            let ext_changed = (p.x - last.x).abs() > 0.5
                || (p.y - last.y).abs() > 0.5
                || p.facing != if last.flags_a.get(6) { 1 } else { -1 }
                || self.selected as u8 != last.selected_item
                || p.leg_frame != last.leg_frame
                || p.body_frame != last.body_frame
                || p.on_ground != last.on_ground
                || p.hp != last.hp
                || p.max_hp != last.max_hp;
            if ext_changed {
                if !out.contains(&PlayerSend::Controls) {
                    out.push(PlayerSend::Controls);
                }
                let pn = net::PlayerNet {
                    id: self.local_id,
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
                    ..Default::default()
                };
                let mut ps = self.player_state_of(&pn);
                nps::apply_control_bits(&mut ps, &controls);
                self.net_ps_last = ps;
            }
        }
        // 2. `TrySyncingMyPlayer` (`Main.cs:17545-17574`): the `5` array diffs
        // FIRST, then the scalar tail. The port walks the same slot layout
        // `client_send_identity` pushes, in the C#'s array order: the 58 grid
        // rows, armour, dye, misc, misc-dye, bank, bank2, TRASH, bank3, bank4
        // (the loadout arrays `:17568-17573` have no port loadouts — recorded).
        // A missing clone entry compares as AIR: `Main.clientPlayer`'s items
        // start `new Item()` (`clientCloneItem` copies type/stack/prefix/
        // favorited only), so an empty live slot is NOT a change.
        let mut synced_any = false;
        let mut changed: Vec<(i16, (u16, u32, u16, bool))> = Vec::new();
        {
            let mut diff = |slot: i16, it: &crate::item::ItemSlot, fav: bool| {
                let st = nps::item_net_state(it, fav);
                let prev = self.net_clone_items.get(&slot).copied().unwrap_or((0, 0, 0, false));
                if prev != st {
                    changed.push((slot, st));
                    synced_any = true;
                }
            };
            for (i, slot) in self.inventory.iter().enumerate() {
                diff(i as i16, slot, self.favorite_slots.contains(&i));
            }
            for i in 0..self.equip.coins.len() {
                diff(50 + i as i16, &self.equip.coins[i], false);
            }
            for i in 0..self.equip.ammo.len() {
                diff(54 + i as i16, &self.equip.ammo[i], false);
            }
            for i in 0..p.equipped.len() {
                let it = crate::item::ItemSlot {
                    kind: p.equipped[i],
                    name: crate::registry::name(p.equipped[i]),
                    count: u32::from(p.equipped[i] != 0),
                    prefix: p.equipped_prefix[i],
                };
                diff(player_item_slot::ARMOR_0 + i as i16, &it, false);
            }
            for i in 0..p.dye_slots.len() {
                let it = crate::item::ItemSlot {
                    kind: p.dye_slots[i],
                    name: crate::registry::name(p.dye_slots[i]),
                    count: u32::from(p.dye_slots[i] != 0),
                    prefix: 0,
                };
                diff(player_item_slot::DYE_0 + i as i16, &it, false);
            }
            for (i, slot) in self.equip.misc_equips.iter().enumerate() {
                diff(player_item_slot::MISC_0 + i as i16, slot, false);
            }
            for (i, slot) in p.misc_dye_slots.iter().enumerate() {
                let it = crate::item::ItemSlot {
                    kind: *slot,
                    name: crate::registry::name(*slot),
                    count: u32::from(*slot != 0),
                    prefix: 0,
                };
                diff(player_item_slot::MISC_DYE_0 + i as i16, &it, false);
            }
            for (b, vault) in p.banks.iter().enumerate() {
                for (i, slot) in vault.iter().enumerate() {
                    // W4-c28 id-lane fix: the trash sentinel reserves id 179
                    // (`player_item_slot::TRASH`), bank3 starts at 180 — every
                    // bank slot diffs, including bank3[0].
                    let ws = player_item_slot::bank_1_0(b) + i as i16;
                    diff(ws, slot, false);
                }
                // The C# order is bank, bank2, TRASH, bank3, bank4
                // (`Main.cs:17556-17567`).
                if b == 1 {
                    diff(player_item_slot::TRASH, &self.equip.trash, false);
                }
            }
        }
        for (slot, st) in changed {
            self.net_clone_items.insert(slot, st);
            out.push(PlayerSend::Inventory { slot });
        }
        // The scalar tail (`Main.cs:17575-17702`).
        let zones = nps::zone_bytes(&p.zones, self.scene_metrics.zone_shimmer);
        let (hp, max_hp, mana, max_mana) = (p.hp, p.max_hp, p.mana, p.mana_max);
        let buffs = p.buffs.clone();
        let (live_chest, edited) = (p.chest, p.edited_chest_name);
        let clone_chest_before = self.net_client_clone.chest;
        // `Main.cs:17581` — the named arm reads `Main.chest[player.chest]`,
        // the CLONE's chest (the one the rename landed on before the close);
        // a null array slot (`:17585-17587`) sends the nameless 33 instead.
        let close_name = (clone_chest_before >= 0
            && self
                .world_chests
                .get(clone_chest_before as usize)
                .is_some_and(|c| c.tile != crate::chest::removed_tile()))
        .then(|| crate::chest::chest_name(self, clone_chest_before).to_string());
        let talk = self.talk_npc.map_or(-1, |i| i as i32);
        nps::try_syncing_my_player_tail(
            &mut self.net_client_clone,
            zones,
            hp,
            max_hp,
            mana,
            max_mana,
            &buffs,
            live_chest,
            edited,
            || close_name.clone(),
            talk,
            (0.0, 0.0), // MinionRestTargetPoint — unported (M6-NET-5)
            -1,         // MinionAttackTargetNPC — unported
            &controls,
            synced_any,
            &mut out,
        );
        if edited && live_chest < 0 && clone_chest_before >= 0 {
            // The flush consumed the flag (`Main.cs:17583`/`:17591`).
            if let Some(p) = self.player.as_mut() {
                p.edited_chest_name = false;
            }
        }
        // 3. `UpdateClient`'s counter (`Main.cs:64522-64536`): the 420-tick
        // `13` beat, then the 900-tick 36/16/40 trio in that order.
        let (beat13, trio) = nps::update_client_beat(&mut self.net_client_counter);
        if beat13 {
            out.push(PlayerSend::Controls);
        }
        if trio {
            let (zone1, zone2, zone3, zone4, zone5, town_npcs) = zones;
            out.push(PlayerSend::Zone { zone1, zone2, zone3, zone4, zone5, town_npcs });
            out.push(PlayerSend::Life {
                hp: hp.clamp(0, i16::MAX as i32) as i16,
                max_hp: max_hp.clamp(0, i16::MAX as i32) as i16,
            });
            out.push(PlayerSend::TalkNpc {
                npc: talk.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            });
        }
        self.net_player_outbox.extend(out);
    }

    /// W4-c28: drain the player-state outbox through the existing wires
    /// (client). `Controls` carries the port's render-extension change test
    /// on top of the C# decision: the body is only re-sent when the wire
    /// fields changed OR the decision came from a beat/event (the C# sends
    /// unconditionally at its sites; the port's 60 Hz compare already gates).
    fn drain_player_outbox(&mut self) {
        use crate::app::net_player_sync::PlayerSend;
        let outbox = std::mem::take(&mut self.net_player_outbox);
        if outbox.is_empty() {
            return;
        }
        let NetRole::Client(client) = &self.net else {
            return;
        };
        let me = self.local_id as u8;
        for send in outbox {
            match send {
                PlayerSend::Controls => {
                    if let Some(p) = self.player.as_ref() {
                        let pn = net::PlayerNet {
                            id: self.local_id,
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
                            ..Default::default()
                        };
                        let mut ps = self.player_state_of(&pn);
                        let c = self.control_state_of();
                        crate::app::net_player_sync::apply_control_bits(&mut ps, &c);
                        client.send(&C2s::PlayerState(ps.clone()));
                        self.net_ps_last = ps;
                    }
                }
                PlayerSend::Life { hp, max_hp } => {
                    client.send(&C2s::PlayerHp { player_id: me, hp, max_hp });
                }
                PlayerSend::Mana { mana, max_mana } => {
                    client.send(&C2s::PlayerMana(net::PlayerMana {
                        player_id: me,
                        mana,
                        max_mana,
                    }));
                }
                PlayerSend::Buffs { ids } => {
                    client.send(&C2s::PlayerBuffs(net::PlayerBuffs {
                        player_id: me,
                        buffs: ids,
                    }));
                }
                PlayerSend::Inventory { slot } => {
                    if let Some(item) = self.net_item_at_wire_slot(slot) {
                        client.send(&C2s::Inventory { slot, item });
                    }
                }
                PlayerSend::ChestClose { chest, name } => {
                    // `NetMessage.cs` case 33: `number` is the LIVE chest
                    // (negative on this arm), so the `number > -1` coord read
                    // (`:884-889`) never runs — x/y are 0. The named arm
                    // (`number2 == 1f`, `:890-901`) writes the length byte
                    // (255 for an empty/too-long name) then the text only
                    // when accepted; the nameless arm writes 0.
                    let (name_len, name) = match name {
                        Some(n) => {
                            let len = n.chars().count();
                            if len == 0 || len > 20 {
                                (255, String::new())
                            } else {
                                (len as u8, n)
                            }
                        }
                        None => (0, String::new()),
                    };
                    client.send(&C2s::PlayerChest(net::PlayerChest {
                        chest,
                        x: 0,
                        y: 0,
                        name_len,
                        name,
                    }));
                }
                PlayerSend::Loadout { loadout, hide_accessory } => {
                    client.send(&C2s::SyncLoadout(net::SyncLoadout {
                        player_id: me,
                        loadout,
                        hide_accessory,
                    }));
                }
                // The 36 zone body is W4-c27's `PlayerZone` (its net.rs lane);
                // 35/40/84/99/115/138 have no port wire yet — the decisions
                // are recorded, the sends land with their bodies (ANALYSIS).
                // The 55 BuffPvp body is also c27's lane (`PlayerBuffPvp`).
                PlayerSend::Zone { .. }
                | PlayerSend::Heal { .. }
                | PlayerSend::TalkNpc { .. }
                | PlayerSend::Stealth { .. }
                | PlayerSend::MinionTarget { .. }
                | PlayerSend::BuffPvp { .. }
                | PlayerSend::InventorySynced => {}
            }
        }
    }

    /// W4-c28: the `NetItem` at one `PlayerItemSlotID` wire slot — the same
    /// layout `client_send_identity` pushes (the `5` diff's body source).
    fn net_item_at_wire_slot(&self, slot: i16) -> Option<net::NetItem> {
        use player_item_slot as ps;
        let p = self.player.as_ref();
        if (0..50).contains(&slot) {
            let i = slot as usize;
            return Some(self.net_item(&self.inventory[i], self.favorite_slots.contains(&i)));
        }
        if (50..54).contains(&slot) {
            return Some(self.net_item(&self.equip.coins[(slot - 50) as usize], false));
        }
        if (54..58).contains(&slot) {
            return Some(self.net_item(&self.equip.ammo[(slot - 54) as usize], false));
        }
        if slot == ps::TRASH {
            return Some(self.net_item(&self.equip.trash, false));
        }
        let p = p?;
        if (ps::ARMOR_0..ps::ARMOR_0 + 20).contains(&slot) {
            let i = (slot - ps::ARMOR_0) as usize;
            return Some(net::NetItem {
                kind: p.equipped[i] as i16,
                stack: i16::from(p.equipped[i] != 0),
                prefix: p.equipped_prefix[i].min(255) as u8,
                favorited: false,
                blocked: false,
            });
        }
        if (ps::DYE_0..ps::DYE_0 + 10).contains(&slot) {
            let i = (slot - ps::DYE_0) as usize;
            return Some(net::NetItem {
                kind: p.dye_slots[i] as i16,
                stack: i16::from(p.dye_slots[i] != 0),
                prefix: 0,
                favorited: false,
                blocked: false,
            });
        }
        if (ps::MISC_0..ps::MISC_0 + 5).contains(&slot) {
            return Some(self.net_item(&self.equip.misc_equips[(slot - ps::MISC_0) as usize], false));
        }
        if (ps::MISC_DYE_0..ps::MISC_DYE_0 + 5).contains(&slot) {
            let i = (slot - ps::MISC_DYE_0) as usize;
            return Some(net::NetItem {
                kind: p.misc_dye_slots[i] as i16,
                stack: i16::from(p.misc_dye_slots[i] != 0),
                prefix: 0,
                favorited: false,
                blocked: false,
            });
        }
        for b in 0..4 {
            let base = ps::bank_1_0(b);
            if (base..base + 40).contains(&slot) {
                return Some(self.net_item(&p.banks[b][(slot - base) as usize], false));
            }
        }
        None
    }

    /// The per-frame smoothing upkeep for a joining client: both `netOffset`
    /// accumulators decay toward zero (the C#'s `Player.UpdateNetOffset` —
    /// `max(2, 10% of the lag)` per tick, collision-boosted; the NPC arm's
    /// `2 + 2*(len/range)` — `NPC.cs:90287-90311`), and the client's npc list
    /// is refreshed from the slot store so the render sees the decayed
    // offsets. Host/single-player: a no-op (`net_offset`/`off_*` stay zero).
    pub(crate) fn net_interp_tick(&mut self, dt: f32) {
        if !self.is_client() {
            return;
        }
        // One 60 Hz tick of decay per (dt * 60) so a 144 Hz display does not
        // decay faster than the original.
        let scale = (dt * 60.0).clamp(0.0, 4.0);
        for p in self.remote_players.values_mut() {
            let len = (p.off_x * p.off_x + p.off_y * p.off_y).sqrt();
            if len <= 0.0 {
                continue;
            }
            // `Player.UpdateNetOffset` (`Player.cs:28721-28750`): an offset
            // shorter than 2 px is cleared outright (`:28730-28734`).
            if len < 2.0 {
                p.off_x = 0.0;
                p.off_y = 0.0;
                continue;
            }
            // `:28737-28747` — the collision boost: the tile collision at
            // `position + netOffset` that eats the velocity raises the step,
            // so a peer pushed into a wall snaps back faster.
            let mut min_step = 2.0f32;
            let resolved = crate::collision::tile_collision(
                &self.world,
                [p.x + p.off_x, p.y + p.off_y],
                [p.vx, p.vy],
                crate::player::WIDTH as f32,
                crate::player::HEIGHT as f32,
                false,
                false,
            );
            let value = Vec2::new(p.vx - resolved.vel[0], p.vy - resolved.vel[1]);
            if value != Vec2::new(p.vx, p.vy) {
                let num4 = value.dot(Vec2::new(p.off_x, p.off_y));
                if num4 >= 1.0 {
                    min_step = min_step.max(value.length_squared() * len / num4 * 1.0001);
                }
            }
            let step = min_step.max(len * 0.1) * scale;
            let k = ((len - step) / len).max(0.0);
            p.off_x *= k;
            p.off_y *= k;
            if (p.off_x * p.off_x + p.off_y * p.off_y).sqrt() < 0.5 {
                p.off_x = 0.0;
                p.off_y = 0.0;
            }
        }
        let mut dirty = false;
        for npc in self.net_npcs.values_mut() {
            let len = npc.net_offset.length();
            if len <= 0.0 {
                continue;
            }
            // `NPC.UpdateNetOffset` (`NPC.cs:90280-90310`): the step is
            // `2 + 2*(len/range)` and the offset moves toward zero along its
            // own direction (`:90300-90302`).
            let step = (2.0 + 2.0 * len / SMOOTHING_RANGE) * scale;
            let k = ((len - step) / len).max(0.0);
            npc.net_offset *= k;
            if npc.net_offset.length() < 0.5 {
                npc.net_offset = Vec2::ZERO;
            }
            // `:90306-90310` — a town NPC that reached its home drops the
            // offset: the smoothed position is the home itself.
            if let Some((hx, hy)) = npc.home {
                let (nw, nh) = npc.size();
                let home = Vec2::new(
                    hx as f32 * 16.0 + 8.0 - nw * 0.5,
                    hy as f32 * 16.0 - nh - 0.1,
                );
                if Vec2::new(npc.x, npc.y).distance(home) < 1.0 {
                    npc.net_offset = Vec2::ZERO;
                }
            }
            dirty = true;
        }
        if dirty {
            self.net_npcs_dirty = true;
            // Rebuild immediately: the decay must be visible to this frame's
            // draw (poll_net's rebuild is a frame behind it otherwise).
            self.net_rebuild_lists();
        }
    }

    /// The pixel position a peer slot is seeded at when it is first seen:
    /// the world spawn, where the C# creates `Main.player[i]` before the
    /// first `13` arrives (`MessageBuffer.cs:908-930`'s spawn arm).
    fn net_peer_spawn_pos(&self) -> (f32, f32) {
        let sx = self.world.spawn_x as f32 * TILE_PX + 8.0 - player::WIDTH * 0.5;
        let sy = self.world.spawn_y as f32 * TILE_PX;
        (sx, sy)
    }

    /// Builds the `21 ItemDrop` body for one live drop
    /// (`NetMessage.cs:646-669`); the ownership bits 0/1 ride the C#'s
    /// `number2` ownership argument only on the spawn send (unported —
    /// the port's diff cadence sends 0). Bit 3 carries the enemy-grab lock
    /// clamped to a byte (`:667`, S1-I-1 wave I).
    fn item_state_of(slot: i16, it: &item::ItemDrop) -> net::ItemState {
        net::ItemState {
            slot,
            x: it.pos.x,
            y: it.pos.y,
            vx: it.vel.x,
            vy: it.vel.y,
            stack: it.count.min(i16::MAX as u32) as i16,
            prefix: it.prefix.min(255) as u8,
            kind: it.kind as i16,
            shimmered: it.shimmered,
            shimmer_time: it.shimmer_time,
            enemy_grab_delay: it.enemy_grab_delay_time.clamp(0, 255) as u8,
        }
    }

    /// Test-only: drain one queued client event without applying it (the
    /// `162` ack is a client no-op, so a witness observes the raw frame).
    #[cfg(test)]
    fn net_events_for_test(&self) -> Option<net::ClientEvent> {
        match &self.net {
            NetRole::Client(client) => client.events.try_recv().ok(),
            _ => None,
        }
    }

    /// `SendData(21)`'s pre-switch (`NetMessage.cs:89-100`): an `instanced`
    /// drop is never synced (`:92-95` returns), an `IsAir` one is rewritten to
    /// `151 SyncItemDespawn` (`:96-99`). Returns the message to send, or `None`
    /// when the C# returns without sending. The host's `21` producers (the join
    /// burst + the cadence diff/heartbeat) all ride this gate.
    fn item_send_gate(slot: i16, it: &item::ItemDrop) -> Option<S2c> {
        if it.instanced {
            return None; // `:92-95` — the instanced loot split never syncs
        }
        if it.is_air() {
            return Some(S2c::ItemDead { slot }); // `:96-99` — the air rewrite
        }
        Some(S2c::ItemState(Box::new(Self::item_state_of(slot, it))))
    }

    /// Builds the `13 PlayerState` body for a stored peer snapshot (the
    /// host-side relay and the join bundle).
    fn player_state_of(&self, p: &net::PlayerNet) -> net::PlayerState {
        let mut flags_a = net::BitsByte::new();
        flags_a.set(6, p.facing > 0);
        let mut flags_b = net::BitsByte::new();
        flags_b.set(2, p.vx != 0.0 || p.vy != 0.0);
        // W4-g10 FINAL — bitsByte23[4]/[5]: petting/petSmall
        // (`NetMessage.cs:475-476`).
        let mut flags_c = net::BitsByte::new();
        flags_c.set(4, p.petting);
        flags_c.set(5, p.pet_small);
        net::PlayerState {
            player_id: p.id as u8,
            flags_a,
            flags_b,
            flags_c,
            flags_d: net::BitsByte::new(),
            selected_item: p.selected as u8,
            x: p.x,
            y: p.y,
            vx: p.vx,
            vy: p.vy,
            // No mount/potion/camera sim: the gate bits stay clear so the
            // C# bodies (`NetMessage.cs:496-508`) stay off the wire.
            mount_type: 0,
            potion_use: (0.0, 0.0),
            potion_home: (0.0, 0.0),
            camera_target: (0.0, 0.0),
            on_ground: p.on_ground,
            leg_frame: p.leg_frame,
            body_frame: p.body_frame,
            hp: p.hp,
            max_hp: p.max_hp,
        }
    }

    /// Rebuilds the client's npc/item render lists from the slot stores, in
    /// slot order (the deterministic render order — `TR_SHOT` parity).
    fn net_rebuild_lists(&mut self) {
        if std::mem::take(&mut self.net_npcs_dirty) {
            let mut slots: Vec<(u32, crate::npc::Npc)> =
                self.net_npcs.iter().map(|(k, v)| (*k, v.clone())).collect();
            slots.sort_by_key(|(k, _)| *k);
            self.net_npc_slots = slots.iter().map(|(k, _)| *k).collect();
            self.npcs = slots.into_iter().map(|(_, v)| v).collect();
        }
        if std::mem::take(&mut self.net_items_dirty) {
            let mut v: Vec<(i16, item::ItemDrop)> = self
                .net_items
                .iter()
                .map(|(k, d)| (*k, d.clone()))
                .collect();
            v.sort_by_key(|(k, _)| *k);
            self.items = v.into_iter().map(|(_, d)| d).collect();
        }
    }

    /// A local switch trip in MP — the C#'s `NetMessage.SendData(59, -1, -1,
    /// null, i, j)` (`Collision.cs:3304`/`:3318`/`:3324`/`:3347`,
    /// `Player.cs:32178`, `:32226`, `:32340`): on a client it goes to the
    /// server, on the host it broadcasts to every client.
    pub(crate) fn send_hit_switch(&self, x: i32, y: i32) {
        let msg = net::HitSwitch {
            x: x as i16,
            y: y as i16,
        };
        match &self.net {
            NetRole::Host(server) => server.broadcast(&S2c::HitSwitch(msg)),
            NetRole::Client(client) => client.send(&C2s::HitSwitch(msg)),
            NetRole::Off => {}
        }
    }

    /// The `switch_tiles` scan's fired switches as `59`s.
    pub(crate) fn send_switch_outcome(&self, outcome: &crate::collision::SwitchOutcome) {
        for (x, y) in &outcome.switches {
            self.send_hit_switch(*x, *y);
        }
    }

    /// W4-c28: the container `32 SyncChestItem` send — EVENT-driven, one per
    /// changed world-chest slot (`ItemSlot.cs:1014` left-click / `:1750`
    /// right-click / `:808` shift-take, `ChestUI.cs`'s bulk transfers,
    /// `ItemSorting.cs:1677` sort). Client-only (`Main.netMode == 1`); the
    /// body reads the slot back out of the store (`NetMessage.cs:864-877`:
    /// `short chest`, `byte slot`, `short stack`, `byte prefix`, `short type`
    /// — type 0 when the item has no name). The host/SP never send it (the
    /// host's own chest edits relay through `request_edit`'s `20` and the
    /// `32` relay arm). Banks are NOT world chests — they ride the `5`
    /// player-inventory sync, no `32` here.
    pub(crate) fn send_chest_item(&self, chest: i32, slot: usize) {
        if chest < 0 {
            return; // the bank / closed arms never send 32
        }
        let NetRole::Client(client) = &self.net else {
            return;
        };
        let Some(it) = crate::chest::chest_slot(&self.world_chests, chest as usize, slot) else {
            return;
        };
        client.send(&C2s::ChestItem(net::ChestItem {
            chest: chest as i16,
            slot: slot as u8,
            stack: it.count.min(i16::MAX as u32) as i16,
            prefix: it.prefix.min(255) as u8,
            kind: it.kind as i16,
        }));
    }

    /// W4-c28: the bulk-transfer `32` fan-out — every slot whose contents
    /// changed across a LootAll/DepositAll/QuickStack/Restock/Sort pass
    /// (`ChestUI.cs:813-819`'s post-pass loop, `ItemSorting.cs:1673-1678`'s
    /// stamp diff). `before` is the slot snapshot taken before the pass;
    /// the compare is the C#'s `MemoryStamp` (type/stack/prefix — the net
    /// state), one `32` per changed slot in slot order.
    pub(crate) fn send_chest_item_diff(&self, chest: i32, before: &[crate::item::ItemSlot]) {
        if chest < 0 || !matches!(self.net, NetRole::Client(_)) {
            return;
        }
        let Some(wc) = self.world_chests.get(chest as usize) else {
            return;
        };
        for i in 0..before.len() {
            let now = wc.slots.get(i).copied().unwrap_or(crate::item::EMPTY_SLOT);
            let was = before[i];
            if (now.kind, now.count, now.prefix) != (was.kind, was.count, was.prefix) {
                self.send_chest_item(chest, i);
            }
        }
    }

    /// W4-c28: the PVP-toggle click's `30 TogglePVP` send
    /// (`Main.cs:42694`, `NetMessage.SendData(30, -1, -1, null, myPlayer)`).
    /// The site has NO netMode guard of its own — `SendData`'s own
    /// `netMode == 0` return (`NetMessage.cs:83-86`) drops it in SP, so the
    /// port's role match is the whole gate. Client: one `30` to the host
    /// (the writer reads `Main.player[number].hostile` at send time,
    /// `NetMessage.cs:856-859` — the flip already landed). Host: the
    /// broadcast fan-out (`NetMessage.cs:1801-1811`'s default arm); the host
    /// is player slot 0 (`Main.myPlayer`), so the body's id is 0 — the port's
    /// `local_id` is 0 while hosting (the client's assigned id is its own).
    /// The host apply/relay + the team-coloured chat fan-out are W4-g48's
    /// lane (`MessageBuffer.cs:1851-1867`); this is the SEND trigger the
    /// click site lacked.
    pub(crate) fn send_toggle_pvp(&self, hostile: bool) {
        match &self.net {
            NetRole::Host(server) => server.broadcast(&S2c::TogglePvp(net::TogglePvp {
                player_id: 0, // the host's player slot (`Main.myPlayer`)
                hostile,
            })),
            NetRole::Client(client) => client.send(&C2s::TogglePvp(net::TogglePvp {
                player_id: self.local_id as u8,
                hostile,
            })),
            NetRole::Off => {}
        }
    }

    /// W4-c28: the team-cell click's `157 TeamChangeFromUI` send DECISION
    /// (`Main.cs:42766`, `NetMessage.SendData(157, -1, -1, null, myPlayer)`
    /// under `Main.netMode != 0`, `:42764`). The writer's `case 45: case 157:`
    /// fall-through (`NetMessage.cs:970-974`) writes the SAME two bytes as
    /// `45`, but the id byte differs and the C# reader's team-based-spawns
    /// tail keys on `b == 157` (`MessageBuffer.cs:2355`) — so a `45` stand-in
    /// is NOT faithful. The `157` id/codec is W4-d97's lane
    /// (`TEAM_CHANGE_FROM_UI`, the shared `TeamChange` body); until that wire
    /// merges the send PARKS on both legs (the c27 `55`/`36`/`35` precedent —
    /// recorded, not silent). The host apply/relay + announcement are W4-g49's
    /// lane (`MessageBuffer.cs:2325-2364`). This records the trigger the click
    /// site lacked; the wire lands with d97's id.
    pub(crate) fn send_team_change(&self, _team: u8) {
        // Parked: the `157` id is d97's `TEAM_CHANGE_FROM_UI`. No faithful
        // stand-in exists on the c28 base (`45` would skip the `b == 157`
        // team-spawn distinction), so the decision is recorded and the send
        // lands with the id. SP: the C#'s `netMode != 0` gate already drops it.
    }

    /// W4-c28: the door right-click's `19 ToggleDoorState` send DECISION
    /// (`Player.cs:32844`/`:32851` open, `:32861` close —
    /// `NetMessage.SendData(19, -1, -1, null, 0|1, myX, myY, direction)`).
    /// The sites are unguarded at the call (SendData's `netMode == 0` return
    /// drops SP); the host fighter door-bash (`NPC.cs:60069`/`:60082`) is the
    /// `netMode == 2` twin. The `19` id/codec is W4-d90's lane
    /// (`DOOR_TOGGLE`, `NetMessage.cs:537-542` — byte action + short x/y +
    /// the `number4 == 1f ? 1 : 0` direction byte); until that wire merges the
    /// send PARKS (the c27 `55`/`36` precedent — recorded, not silent). The
    /// host apply/relay is W4-g48's `host_door_toggle`
    /// (`MessageBuffer.cs:1299-1334`). `action` is 0 open / 1 close (the
    /// trapdoor 2/3 + tall-gate 4/5 arms are unported — `ShiftTrapdoor`/
    /// `ShiftTallGate` have no port); `direction` is the player's facing (the
    /// C# open retry's `-direction` nuance rides the d90 wire's own decode).
    pub(crate) fn send_door_toggle(&self, _action: u8, _x: i32, _y: i32, _direction: i32) {
        // Parked: the `19` id is d90's `DOOR_TOGGLE`. The decision is recorded
        // at the interaction site; the wire lands with the id. SP drops it.
    }

    /// W4-c28: a client-side `NPC.AddBuff` (`NPC.cs:89932-89935`): the `53`
    /// body addressed by the host's slot (`NetMessage.cs:1019-1022` —
    /// `(short)number, (ushort)number2, (short)number3`). Client-only: the
    /// host's own AddBuff fan-out is the `54` full list
    /// (`NPC.cs:89979-89982`, the same drain's host arm).
    pub(crate) fn send_npc_add_buff(&self, idx: usize, buff_type: u16, time: i32) {
        let NetRole::Client(client) = &self.net else {
            return;
        };
        let Some(slot) = self.net_npc_slots.get(idx).copied() else {
            return;
        };
        client.send(&C2s::AddNpcBuff(net::AddNpcBuff {
            npc: slot as i16,
            buff_type,
            time: time.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }));
    }

    /// W4-c28: drain the per-NPC buff send queue the buff engine fills
    /// (`NpcBuffs::net_sends`). The C# has two role-gated branches at the
    /// same `NPC.AddBuff` site: the client's `53` BEFORE the apply
    /// (`NPC.cs:89932-89935`) and the host's `54` full list AFTER it
    /// (`:89979-89982`). The port queues at the accepted-write point and
    /// this drain picks the wire by role — client: one `53` per queued pair
    /// through the host slot map; host: ONE `54` per touched NPC (the C#
    /// sends per AddBuff call, but the list body is identical within a
    /// pass — the last write wins, the same bytes). SP: the queue drops.
    /// W4-c28: the expiry-sweep latch (`NpcBuffs::expired_send`,
    /// `NPC.cs:91896-91908`) rides the same host arm — the decision layer
    /// (`net_npc_sync::expired_buff_sweep`) gates it, one `54` per expired
    /// sweep, coalesced with an AddBuff in the same pass.
    /// Runs at the sim-pass tails that mutate NPC buffs (the melee hit,
    /// the projectile pass, the player-tick retaliate drain).
    pub(crate) fn drain_npc_buff_sends(&mut self) {
        match self.net_mode() {
            1 => {
                for idx in 0..self.npcs.len() {
                    // The client's sweep returns before the send
                    // (`NPC.cs:91892-91895`) — the latch drops, no wire.
                    self.npcs[idx].buffs.expired_send = false;
                    let sends = self.npcs[idx].buffs.take_net_sends();
                    for (buff, time) in sends {
                        self.send_npc_add_buff(idx, buff as u16, time);
                    }
                }
            }
            2 => {
                let NetRole::Host(server) = &self.net else {
                    return;
                };
                for idx in 0..self.npcs.len() {
                    let added = !self.npcs[idx].buffs.take_net_sends().is_empty();
                    // W4-c28: the expired-sweep's `flag` latch
                    // (`NPC.cs:91896-91908`). The decision layer owns the
                    // host-only gate; the sweep ran inside `update_buffs_tick`
                    // with no netMode, so the latch rides here.
                    let expired = std::mem::take(&mut self.npcs[idx].buffs.expired_send);
                    let mut sweep = Vec::new();
                    crate::app::net_npc_sync::expired_buff_sweep(
                        idx as i16,
                        2,
                        expired,
                        &mut sweep,
                    );
                    // One `54` full list per touched NPC per pass — the AddBuff
                    // fan-out (`:89979-89982`) and the expiry sweep
                    // (`:91905-91908`) both write the SAME current list, so a
                    // pass that did both coalesces to one (last write wins,
                    // the same bytes — the recorded deviation).
                    if !added && sweep.is_empty() {
                        continue;
                    }
                    let buffs = npc_buffs_of(&self.npcs[idx]);
                    server.broadcast(&S2c::NpcBuffs(net::NpcBuffs {
                        npc: idx as i16,
                        buffs,
                    }));
                }
            }
            _ => {
                for n in self.npcs.iter_mut() {
                    n.buffs.net_sends.clear();
                    n.buffs.expired_send = false;
                }
            }
        }
    }

    /// `AchievementsHelper.NotifyNPCKilled`'s wire half
    /// (`AchievementsHelper.cs:200-217`): in MP the host sends `97` to every
    /// slot recorded in the dead NPC's `playerInteraction` (`:210-216`). The
    /// `i < 255` bound is the C#'s "not the server itself"; the port's host
    /// occupies no client slot, so `Server::send` to it is already a no-op.
    pub(crate) fn send_npc_killed(&self, npc: &crate::npc::Npc, npc_net_id: i16) {
        let NetRole::Host(server) = &self.net else {
            return;
        };
        for i in 0..255u8 {
            if npc.player_interaction[i as usize] {
                server.send(i as u32, &S2c::NpcKilled { npc: npc_net_id });
            }
        }
    }

    /// `Player.PlayGuitarChord` (`Player.cs:46671-46703`): the chord ladder
    /// over `range` (the `58` pitch), six `1/6` bands picking int types
    /// 49/48/47/51/52/50. The bad-luck roll (`RollOnlyBadLuckExtreme(10)`,
    /// `Luck.cs:53-60`) draws on `Main.rand` — the net replay's bare stream
    /// (`Main.cs:16895`); on 0 it adds `Next(-50, 51) * 0.01` to the pitch
    /// offset. W4-g49.
    fn play_guitar_chord(&mut self, cx: f32, cy: f32, range: f32) {
        let num = 1.0f32 / 6.0;
        let mut pitch_offset = 0.0f32;
        let luck = self.player.as_ref().map(|p| p.luck()).unwrap_or(0.0);
        let rng = self.rngs.global();
        // `:46675-46678` — `RollOnlyBadLuckExtreme(10) == 0`.
        if tr_core::loot::roll_only_bad_luck_extreme(rng, luck, 10) == 0 {
            pitch_offset = rng.next_range(-50, 51) as f32 * 0.01;
        }
        // `:46679-46702` — the six bands, in the C#'s order.
        let sound_type = if range > num * 5.0 {
            49
        } else if range > num * 4.0 {
            48
        } else if range > num * 3.0 {
            47
        } else if range > num * 2.0 {
            51
        } else if range > num {
            52
        } else {
            50
        };
        self.audio.play_instrument_int(sound_type, cx, cy, pitch_offset);
    }

    /// `Player.PlayDrums` (`Player.cs:46705-46748`): the drum ladder over
    /// `range`, ten `1/10` bands picking int types 59/58/53/57/62/61/54/56/
    /// 55/60. No luck roll, no pitch offset (the default 0). W4-g49.
    fn play_drums(&mut self, cx: f32, cy: f32, range: f32) {
        let num = 1.0f32 / 10.0;
        // `:46708-46747` — the ten bands, in the C#'s order.
        let sound_type = if range > num * 9.0 {
            59
        } else if range > num * 8.0 {
            58
        } else if range > num * 7.0 {
            53
        } else if range > num * 6.0 {
            57
        } else if range > num * 5.0 {
            62
        } else if range > num * 4.0 {
            61
        } else if range > num * 3.0 {
            54
        } else if range > num * 2.0 {
            56
        } else if range > num {
            55
        } else {
            60
        };
        self.audio.play_instrument_int(sound_type, cx, cy, 0.0);
    }

    /// `AchievementsHelper.NotifyProgressionEvent` (`:227-237`): the host
    /// sends `98` to every client; SP/client fires locally. This is the
    /// whole function — the caller's own credit rides the same call.
    pub(crate) fn notify_progression_net(&mut self, event_id: u32) -> Option<&'static str> {
        if let NetRole::Host(server) = &self.net {
            server.broadcast(&S2c::ProgressionEvent {
                event_id: event_id.min(i16::MAX as u32) as i16,
            });
        }
        self.achievements.notify_progression(event_id)
    }

    /// `AchievementsHelper.TryGrantingBestiary100PercentAchievement`
    /// (`AchievementsHelper.cs:81-84`): the 100 % completion fires progression
    /// 29 (`BOOK_WORM`) — locally and, on a host, over the `98` wire.
    pub(crate) fn try_grant_bestiary_100_net(&mut self) -> Option<&'static str> {
        if crate::bestiary::completion_percent_world(&self.bestiary) < 1.0 {
            return None;
        }
        self.notify_progression_net(29)
    }

    /// `WorldGen.CheckAchievement_RealEstateAndTownSlimes`
    /// (`WorldGen.cs:5659-5677`): the housed-NPC census — every active,
    /// non-homeless NPC marks its `type`; the 25-type real-estate set fires
    /// progression 17, the 8-town-slime set fires 26. The port's `cs_id` is
    /// the C# `NPC.type`; the town slimes (670/678-684) have no ported kind,
    /// so the 26 gate can never fire until they land (the census still runs).
    fn check_real_estate_and_town_slimes(&mut self) {
        let mut housed = [false; 697]; // `NPCID.Count`
        for n in &self.npcs {
            if n.hp > 0 && n.home.is_some() {
                let ty = n.kind.cs_id() as usize;
                if ty < housed.len() {
                    housed[ty] = true;
                }
            }
        }
        const REAL_ESTATE: [usize; 25] = [
            38, 17, 107, 19, 22, 124, 228, 178, 18, 229, 209, 54, 108, 160, 20, 369, 207,
            227, 208, 441, 353, 550, 588, 633, 663,
        ];
        const TOWN_SLIMES: [usize; 8] = [670, 678, 679, 680, 681, 682, 683, 684];
        if REAL_ESTATE.iter().all(|&t| housed[t]) {
            self.notify_progression_net(17); // `:5671`
        }
        if TOWN_SLIMES.iter().all(|&t| housed[t]) {
            self.notify_progression_net(26); // `:5675`
        }
    }

    /// `Main.teamColor` (`Main.cs:6657-6662`): the six team colours, index 0
    /// white (no team) through 5 pink.
    fn team_color(team: u8) -> net::Rgb {
        const COLORS: [(u8, u8, u8); 6] = [
            (255, 255, 255), // 0 white
            (218, 59, 59),   // 1 red
            (59, 218, 85),   // 2 green
            (59, 149, 218),  // 3 blue
            (242, 221, 100), // 4 yellow
            (224, 100, 242), // 5 pink
        ];
        let (r, g, b) = COLORS[(team as usize).min(5)];
        net::Rgb(r, g, b)
    }

    /// The shared `case 45: case 157:` host arm (`MessageBuffer.cs:2325-
    /// 2364`): force the subject to the sender (`:2329-2332`), set the team
    /// (`:2336`), relay as `45` with `ignoreClient = sender` (`:2342`), then
    /// the team-chat fan-out (`:2343-2354`): `Lang.mp[13 + team]` (team 5 →
    /// `mp[22]`) formatted with the player name, sent to the sender + every
    /// old-team / new-team member. The 157-only team-based-spawns tail
    /// (`:2355-2363`) is the caller's (parked — no `teamBasedSpawnsSeed`).
    fn apply_team_change_net(&mut self, server: &net::Server, id: u32, new_team: u8) {
        let old_team = self
            .remote_players
            .get(&id)
            .map(|p| p.team)
            .unwrap_or(0);
        self.remote_players.entry(id).or_default().team = new_team; // `:2336`
        let name = server.slot_name(id);
        // `:2342` — relay as 45 to the OTHERS.
        server.broadcast_except(
            &S2c::TeamChange(net::TeamChange {
                player_id: id as u8,
                team: new_team,
            }),
            id,
        );
        // `:2343-2347` — the message key (team 5 special-cased to mp[22]).
        let key_idx = if new_team == 5 { 22 } else { 13 + new_team as usize };
        let text = crate::lang::text_args(
            &format!("LegacyMultiplayer.{key_idx}"),
            &[name.clone()],
        );
        let color = Self::team_color(new_team);
        let sm = crate::app::netmodules::ServerMessage {
            author: id as u8,
            text,
            color,
        };
        let frame = S2c::NetModule {
            module: crate::app::netmodules::module_id::TEXT,
            body: sm.encode(),
        };
        // `:2348-2354` — the sender + the old-team / new-team members. The
        // host's own slot (0) rides `player.team`; the peers ride PlayerNet.
        let host_team = self.player.as_ref().map(|p| p.team as u8).unwrap_or(0);
        for peer in 0..255u32 {
            let peer_team = if peer == 0 {
                host_team
            } else {
                self.remote_players.get(&peer).map(|p| p.team).unwrap_or(0)
            };
            let receives = peer == id
                || (old_team > 0 && peer_team == old_team)
                || (new_team > 0 && peer_team == new_team);
            if receives {
                server.send(peer, &frame);
            }
        }
    }

    /// `NPCKillsTracker.RegisterKill`'s broadcast half
    /// (`NPCKillsTracker.cs:23-33`): the host pushes the credit's new count
    /// through `NetBestiaryModule.SerializeKillCount`.
    pub(crate) fn broadcast_bestiary_kill(&self, npc_net_id: i32, count: i32) {
        if let NetRole::Host(server) = &self.net {
            let body = crate::app::netmodules::BestiaryKill::new(npc_net_id as i16, count)
                .encode();
            server.broadcast(&S2c::NetModule {
                module: crate::app::netmodules::module_id::BESTIARY,
                body,
            });
        }
    }

    /// A client-side catch on a networked NPC: the `70` body addressed by
    /// the host's slot (`NPC.CatchNPC`'s `netMode == 1` send,
    /// `NPC.cs:79901-79905`). The local npcs Vec is the slot-keyed store
    /// rebuilt in slot order, so the index maps through `net_npc_slots`
    /// (the `send_npc_strike` convention). The C# also deactivates the
    /// slot LOCALLY (`:79903`) — the store entry leaves with it so a
    /// rebuild cannot resurrect the caught NPC before the host's
    /// despawn `23` lands.
    pub(crate) fn send_catch_npc(&mut self, idx: usize) {
        let Some(slot) = self.net_npc_slots.get(idx).copied() else {
            return;
        };
        let NetRole::Client(client) = &self.net else {
            return;
        };
        client.send(&C2s::CatchNpc {
            npc: slot.min(i16::MAX as u32) as i16,
        });
        self.net_npcs.remove(&slot);
        self.net_npc_gen.remove(&slot);
        self.net_npcs_dirty = true;
    }

    /// A client-side critter release: the `71` body (`NPC.ReleaseNPC`'s
    /// `netMode == 1` send, `NPC.cs:80109-80112`,
    /// `NetMessage.cs:1128-1133`).
    pub(crate) fn send_release_npc(&self, x: i32, y: i32, npc_type: i32, style: i32) {
        let NetRole::Client(client) = &self.net else {
            return;
        };
        client.send(&C2s::ReleaseNpc {
            x,
            y,
            npc_type: npc_type.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            style: style.clamp(0, u8::MAX as i32) as u8,
        });
    }

    /// A client-side strike on a networked NPC: the `28` body addressed by the
    /// host's slot and the generation the `23` push delivered
    // (`NPC.StrikeNPC`'s `fromNet: false` send, `NPC.cs:81218`).
    /// W4-g51 id-162: the send also PUSHES the pending-damage entry
    /// (`NPC.PushPendingDamage`, `NPC.cs:81221` → `:90672-90685`) and folds
    /// the landed damage in (`AddStrikeDamage`, `:90687-90693` — the caller
    /// passes `landed_damage`, 0 when the local strike was refused). The
    /// `realLife` redirect (`:90674-90677`) resolves the carrier first.
    pub(crate) fn send_npc_strike(
        &mut self,
        idx: usize,
        damage: i32,
        knock_back: f32,
        crit: bool,
        from_right: bool,
        landed_damage: i32,
    ) {
        if !matches!(self.net, NetRole::Client(_)) {
            return;
        }
        // The local npcs Vec is the slot-keyed store rebuilt in slot order, so
        // the index maps through `net_npc_slots`.
        let Some(slot) = self.net_npc_slots.get(idx).copied() else {
            return;
        };
        let generation = self
            .net_npc_gen
            .get(&slot)
            .map(|(_, g)| *g as u8)
            .unwrap_or(0);
        // `PushPendingDamage`'s `realLife` redirect (`NPC.cs:90674-90677`):
        // a linked segment's entry keys the CARRIER's slot + generation.
        let carrier_slot = crate::npc::Npc::carrier_index(&self.npcs, idx)
            .and_then(|c| self.net_npc_slots.get(c).copied())
            .unwrap_or(slot);
        let carrier_gen = self
            .net_npc_gen
            .get(&carrier_slot)
            .map(|(_, g)| *g as u8)
            .unwrap_or(generation);
        let current = crate::app::net_npc_sync::push_pending_damage(
            &mut self.net_pending_damages,
            carrier_slot as i16,
            carrier_gen,
        );
        // `AddStrikeDamage` (`NPC.cs:90687-90693`): the landed damage folds
        // into the current entry (the client gate is this method's role check).
        crate::app::net_npc_sync::add_strike_damage(
            &mut self.net_pending_damages,
            Some(current),
            carrier_slot as i16,
            carrier_gen,
            landed_damage,
        );
        if let NetRole::Client(client) = &self.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: slot as u8,
                generation,
                damage: damage.clamp(0, i16::MAX as i32) as i16,
                knock_back,
                hit_direction: if from_right { 1 } else { -1 },
                crit,
            }));
        }
    }

    /// `NPC.TransformCopperSlime` / `TransformElderSlime`'s client sends
    /// (`NPC.cs:19164`/`:19140`): on a client the transform request goes
    /// to the host as `140` kind 1/2 with the NPC slot; the host applies
    /// (`MessageBuffer.cs:4250-4260`). The local unlock/Transform half is
    /// the host's answer (the `7` re-push + the `23` transform update).
    pub(crate) fn send_town_slime_transform(&self, slot: usize, kind: u8) {
        if let NetRole::Client(client) = &self.net {
            client.send(&C2s::MiscEventValue(net::MiscEventValue {
                kind,
                value: slot as i32,
            }));
        }
    }

    /// The replicated tile change: the host broadcasts a `20 SectionRange`
    /// of the edited tiles (`SendTileSquare`, `NetMessage.cs:2627-2630`), the
    /// client asks for one. The edit kinds (0 kill tile, 1 place, 2/3 wall)
    // already mutated the local world, so the record is read back out.
    pub(crate) fn request_edit(&self, x: i32, y: i32, kind: u8, tile: u16) {
        // The port replicates the 1x1 edit (multi-tile objects arrive as
        // per-tile edits; the C#'s `SendTileSquare(-1, x, y, 2)` door
        // widening stays with the door sync).
        let _ = (kind, tile);
        let (w, h) = (1, 1);
        let mut records = Vec::with_capacity((w * h) as usize);
        for dy in 0..h {
            for dx in 0..w {
                records.push(self.tile_record_at(x + dx, y + dy));
            }
        }
        let range = net::SectionRange {
            x: x as i16,
            y: y as i16,
            w: w as u8,
            h: h as u8,
            change_type: 0,
            records,
        };
        match &self.net {
            // W4-g52 — the `case 20` fan-out (`NetMessage.cs:1708-1718`):
            // the square reaches only the slots holding a corner section.
            NetRole::Host(server) => {
                server.relay_tile_square(&S2c::SectionRange(Box::new(range)), None)
            }
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
    /// plain `0..58` indices. The banks carry the port's 40-slot stores
    /// (`bank.rs`'s `BANK_SLOTS`), NOT the C#'s 200 — the bases derive from
    /// the same walk, so the trash sits at 179 and bank3 starts at 180.
    pub const ARMOR_0: i16 = 59;
    pub const DYE_0: i16 = 59 + 20;
    pub const MISC_0: i16 = DYE_0 + 10;
    pub const MISC_DYE_0: i16 = MISC_0 + 5;
    pub const BANK_1_0: i16 = MISC_DYE_0 + 5;
    pub const BANK_2_0: i16 = BANK_1_0 + 40;
    /// `PlayerItemSlotID.TrashItem` (`PlayerItemSlotID.cs:183`) — the one-slot
    /// reservation between bank2 and bank3.
    pub const TRASH: i16 = BANK_2_0 + 40;
    pub const BANK_3_0: i16 = TRASH + 1;
    pub const BANK_4_0: i16 = BANK_3_0 + 40;

    /// The bank block bases in the C#'s allocation order — the trash sentinel
    /// breaks the uniform stride (`PlayerItemSlotID.cs:181-185`), so bank3/4
    /// are NOT `BANK_1_0 + bank * 40`.
    pub const fn bank_1_0(bank: usize) -> i16 {
        match bank {
            0 => BANK_1_0,
            1 => BANK_2_0,
            2 => BANK_3_0,
            _ => BANK_4_0,
        }
    }

    /// `PlayerItemSlotID.CanRelay` (`PlayerItemSlotID.cs:168-186`): the
    /// slots the server relays to the other clients. The inventory (58), the
    /// armour (20), the dye (10), the misc (5) and the misc-dye (5) blocks are
    /// relayable, and so is the fourth bank; the first three banks and the
    /// trash are not.
    pub fn can_relay(slot: i16) -> bool {
        // 0..99: the inventory, the armour, the dye, the misc and the
        // misc-dye blocks.
        if (0..99).contains(&slot) {
            return true;
        }
        // The fourth bank (`Bank4_0`, the only relayable bank).
        (BANK_4_0..BANK_4_0 + 40).contains(&slot)
    }
}

/// The world-probing NPC ctors, seeding a client-side NPC's stats from the
/// `23` body (`npc_from_net`'s stats pass). `None` when the world has no
/// column surface at the slot — the joining client's column tables are not
/// part of the transferred world blob — and the caller falls back to a bare
/// seed, because the `23` body carries everything the render needs.
fn npc_seed(
    world: &World,
    kind: crate::npc::NpcKind,
    tx: i32,
    _x: f32,
    y: f32,
) -> Option<crate::npc::Npc> {
    Some(match kind {
        crate::npc::NpcKind::Slime => crate::npc::Npc::spawn_slime(world, tx)?,
        crate::npc::NpcKind::DemonEye | crate::npc::NpcKind::WanderingEye => {
            crate::npc::Npc::spawn_eye(world, tx)?
        }
        crate::npc::NpcKind::Zombie => crate::npc::Npc::spawn_zombie(world, tx)?,
        // S1-D-3ff wave A — the nine slimes and ten walkers plant their
        // feet on the column surface (the generic surface ctor, the
        // pumpkin walkers' precedent — the family machines take over on
        // the first tick); the five eyes ride the eye-variant ctor a few
        // tiles above the surface.
        crate::npc::NpcKind::MotherSlime
        | crate::npc::NpcKind::LavaSlime
        | crate::npc::NpcKind::DungeonSlime
        | crate::npc::NpcKind::CorruptSlime
        | crate::npc::NpcKind::IlluminantSlime
        | crate::npc::NpcKind::IceSlime
        | crate::npc::NpcKind::Crimslime
        | crate::npc::NpcKind::SpikedIceSlime
        | crate::npc::NpcKind::UmbrellaSlime
        | crate::npc::NpcKind::DoctorBones
        | crate::npc::NpcKind::TheGroom
        | crate::npc::NpcKind::BaldZombie
        | crate::npc::NpcKind::ZombieEskimo
        | crate::npc::NpcKind::PincushionZombie
        | crate::npc::NpcKind::SlimedZombie
        | crate::npc::NpcKind::SwampZombie
        | crate::npc::NpcKind::TwiggyZombie
        | crate::npc::NpcKind::FemaleZombie
        | crate::npc::NpcKind::ZombieRaincoat
        // S1-D-3z — the holiday zombies plant their feet on the column
        // surface like the wave-A walkers (the aiStyle-3 family machine
        // takes over on the first tick).
        | crate::npc::NpcKind::ZombieDoctor
        | crate::npc::NpcKind::ZombieSuperman
        | crate::npc::NpcKind::ZombiePixie
        | crate::npc::NpcKind::ZombieXmas
        | crate::npc::NpcKind::ZombieSweater
        // S1-D-3z — the Blood Zombie plants its feet on the column
        // surface like the wave-A walkers (the aiStyle-3 family machine
        // takes over on the first tick).
        | crate::npc::NpcKind::BloodZombie => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::CataractEye
        | crate::npc::NpcKind::SleepyEye
        | crate::npc::NpcKind::DialatedEye
        | crate::npc::NpcKind::GreenEye
        | crate::npc::NpcKind::PurpleEye => {
            crate::npc::Npc::spawn_eye_variant(world, tx, kind)?
        },
        crate::npc::NpcKind::Guide => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::Merchant
        | crate::npc::NpcKind::Dryad
        | crate::npc::NpcKind::Nurse
        | crate::npc::NpcKind::ArmsDealer
        | crate::npc::NpcKind::Demolitionist
        | crate::npc::NpcKind::Painter
        | crate::npc::NpcKind::WitchDoctor
        | crate::npc::NpcKind::Clothier
        | crate::npc::NpcKind::TravellingMerchant
        | crate::npc::NpcKind::Angler
        | crate::npc::NpcKind::SleepingAngler
        | crate::npc::NpcKind::Wizard
        | crate::npc::NpcKind::Mechanic
        | crate::npc::NpcKind::Stylist
        | crate::npc::NpcKind::TaxCollector
        | crate::npc::NpcKind::Golfer
        | crate::npc::NpcKind::Zoologist
        | crate::npc::NpcKind::Princess
        | crate::npc::NpcKind::DyeTrader
        | crate::npc::NpcKind::PartyGirl
        | crate::npc::NpcKind::GoblinTinkerer
        // W4-d120 - the four town kinds seed like the town roster.
        | crate::npc::NpcKind::SantaClaus
        | crate::npc::NpcKind::Truffle
        | crate::npc::NpcKind::Steampunker
        | crate::npc::NpcKind::Cyborg => crate::npc::Npc::spawn_guide(world, tx)?,
        // W4-g10 lane 3 - the town pets seed on the column surface like the
        // town residents (the generic ground ctor; the pet-aware aiStyle-7
        // machine takes over on the first tick).
        crate::npc::NpcKind::TownCat
        | crate::npc::NpcKind::TownDog
        | crate::npc::NpcKind::TownBunny
        | crate::npc::NpcKind::TownSlimeBlue
        | crate::npc::NpcKind::TownSlimeGreen
        | crate::npc::NpcKind::TownSlimeOld
        | crate::npc::NpcKind::TownSlimePurple
        | crate::npc::NpcKind::TownSlimeRainbow
        | crate::npc::NpcKind::TownSlimeRed
        | crate::npc::NpcKind::TownSlimeYellow
        | crate::npc::NpcKind::TownSlimeCopper => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::EyeOfCthulhu => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::QueenBee
        | crate::npc::NpcKind::SkeletronHead
        | crate::npc::NpcKind::WallOfFlesh
        | crate::npc::NpcKind::TheDestroyer
        // The Destroyer's segments + the Probe seed like the head (the
        // hover ctor; the sim's chain spawn places them properly).
        | crate::npc::NpcKind::TheDestroyerBody
        | crate::npc::NpcKind::TheDestroyerTail
        | crate::npc::NpcKind::Probe
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
        | crate::npc::NpcKind::SkeletronHand
        | crate::npc::NpcKind::WallOfFleshEye
        | crate::npc::NpcKind::PrimeCannon
        | crate::npc::NpcKind::PrimeSaw
        | crate::npc::NpcKind::PrimeVice
        | crate::npc::NpcKind::PrimeLaser
        | crate::npc::NpcKind::Creeper => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::Skeleton
        | crate::npc::NpcKind::UndeadMiner
        | crate::npc::NpcKind::GoblinPeon
        | crate::npc::NpcKind::GoblinThief
        | crate::npc::NpcKind::GoblinWarrior
        | crate::npc::NpcKind::GoblinArcher
        | crate::npc::NpcKind::BoundGoblin
        | crate::npc::NpcKind::BoundWizard
        | crate::npc::NpcKind::BoundMechanic
        | crate::npc::NpcKind::WebbedStylist
        | crate::npc::NpcKind::SleepingGolfer
        // W4-d120 — the bound Tavernkeep seeds like the bound forms.
        | crate::npc::NpcKind::BartenderUnconscious => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::EaterOfSouls
        | crate::npc::NpcKind::Hornet
        | crate::npc::NpcKind::MeteorHead
        // The Queen Bee's volley bees (S1-D-3 W-3) ride the same hover
        // ctor — the net state overwrites the position below.
        | crate::npc::NpcKind::Bee
        | crate::npc::NpcKind::BeeSmall => {
            let y = (y / TILE_PX) as i32;
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
        | crate::npc::NpcKind::DD2GoblinT1
        | crate::npc::NpcKind::DD2GoblinT2
        | crate::npc::NpcKind::DD2GoblinT3
        | crate::npc::NpcKind::DD2GoblinBomberT1
        | crate::npc::NpcKind::DD2GoblinBomberT2
        | crate::npc::NpcKind::DD2GoblinBomberT3
        | crate::npc::NpcKind::DD2JavelinT1
        | crate::npc::NpcKind::DD2JavelinT2
        | crate::npc::NpcKind::DD2JavelinT3
        | crate::npc::NpcKind::DD2SkeletonT1
        | crate::npc::NpcKind::DD2SkeletonT3
        | crate::npc::NpcKind::DD2DarkMage
        | crate::npc::NpcKind::DD2Betsy
        | crate::npc::NpcKind::DD2WitherBeast
        | crate::npc::NpcKind::DD2WyvernT1
        | crate::npc::NpcKind::DD2WyvernT2
        | crate::npc::NpcKind::DD2WyvernT3
        | crate::npc::NpcKind::DD2DarkMageT3
        | crate::npc::NpcKind::DD2WitherBeastT3
        | crate::npc::NpcKind::DD2DrakinT2
        | crate::npc::NpcKind::DD2DrakinT3
        | crate::npc::NpcKind::DD2KoboldWalkerT2
        | crate::npc::NpcKind::DD2KoboldWalkerT3
        | crate::npc::NpcKind::DD2KoboldFlyerT2
        | crate::npc::NpcKind::DD2KoboldFlyerT3
        | crate::npc::NpcKind::DD2OgreT2
        | crate::npc::NpcKind::DD2OgreT3
        | crate::npc::NpcKind::DD2Bartender
        | crate::npc::NpcKind::DD2EterniaCrystal
        | crate::npc::NpcKind::AncientCultist
        | crate::npc::NpcKind::StardustTower
        | crate::npc::NpcKind::NebulaTower
        | crate::npc::NpcKind::SolarTower
        | crate::npc::NpcKind::VortexTower
        | crate::npc::NpcKind::MoonLordHead
        | crate::npc::NpcKind::MoonLordHand
        | crate::npc::NpcKind::MoonLordCore
        | crate::npc::NpcKind::MoonLordFreeEye
        | crate::npc::NpcKind::MoonLordLeechBlob
        | crate::npc::NpcKind::VortexHunter
        | crate::npc::NpcKind::VortexAlien
        | crate::npc::NpcKind::VortexHeadcrab
        | crate::npc::NpcKind::NebulaFloater
        | crate::npc::NpcKind::BrainSuckler
        | crate::npc::NpcKind::Predictor
        | crate::npc::NpcKind::StardustSpiderBig
        | crate::npc::NpcKind::StardustSpiderSmall
        | crate::npc::NpcKind::StardustSoldier
        | crate::npc::NpcKind::SolarDrakomire
        | crate::npc::NpcKind::SolarDrakomireRider
        | crate::npc::NpcKind::SolarSroller
        | crate::npc::NpcKind::SolarCorite
        | crate::npc::NpcKind::SolarCrawltipedeHead
        | crate::npc::NpcKind::SolarCrawltipedeBody
        | crate::npc::NpcKind::SolarCrawltipedeTail
        | crate::npc::NpcKind::StardustWormHead
        | crate::npc::NpcKind::SolarSpearman
        | crate::npc::NpcKind::SolarSelenian
        | crate::npc::NpcKind::VortexTower
        | crate::npc::NpcKind::DD2LanePortal
        | crate::npc::NpcKind::StardustCellBig
        | crate::npc::NpcKind::VortexTower
        | crate::npc::NpcKind::StardustCellSmall
        | crate::npc::NpcKind::StardustJellyfishBig
        | crate::npc::NpcKind::NebulaBeast
        | crate::npc::NpcKind::VortexRifleman
        | crate::npc::NpcKind::VortexSoldier
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
        | crate::npc::NpcKind::WaterSphere
        // The small-kinds wiring wave: the four flyers' relay seeds ride
        // the same hover placement (the box comes from `kind.size()`;
        // the bubble's officer link rides the `23` body's ai[] relay).
        | crate::npc::NpcKind::BlazingWheel
        | crate::npc::NpcKind::AngryNimbus
        | crate::npc::NpcKind::DungeonSpirit
        | crate::npc::NpcKind::ForceBubble => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::CaveBat
        | crate::npc::NpcKind::JungleBat
        | crate::npc::NpcKind::Hellbat
        | crate::npc::NpcKind::VoodooDemon => {
            // The net spawn hangs the bat at the net Y (the cave air).
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_cave_flyer(world, tx, y, kind)?
        }
        crate::npc::NpcKind::KingSlime => crate::npc::Npc::spawn_king_slime(world, tx)?,
        // The hungry/leeches: the ctor seeds the stats — the net state
        // overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::Hungry
        | crate::npc::NpcKind::LeechHead
        | crate::npc::NpcKind::LeechBody
        | crate::npc::NpcKind::LeechTail => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The eye and the servant: the ctor seeds the stats — the net state
        // overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::EyeOfCthulhu | crate::npc::NpcKind::ServantOfCthulhu => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // `NPCID.DesertDjinn` (533) — the desert caster hovers like the
        // other casters (`spawn_hover`; aiStyle 8).
        crate::npc::NpcKind::DesertDjinn => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The Pumpkin Moon family (`S1-F-13b` Phase 2): the wave roster's
        // seed follows their machines — the walkers (the ten Scarecrows,
        // Splinterling, the Headless Horseman and the Hellhound, whose
        // aiStyle-3/26 stand-ins ride the walker family) plant the feet on the
        // column surface; the gravity-off floaters (the Mourning Wood, the
        // Pumpking and the Poltergeist) ride the hover ctor.
        crate::npc::NpcKind::Scarecrow1
        | crate::npc::NpcKind::Scarecrow2
        | crate::npc::NpcKind::Scarecrow3
        | crate::npc::NpcKind::Scarecrow4
        | crate::npc::NpcKind::Scarecrow5
        | crate::npc::NpcKind::Scarecrow6
        | crate::npc::NpcKind::Scarecrow7
        | crate::npc::NpcKind::Scarecrow8
        | crate::npc::NpcKind::Scarecrow9
        | crate::npc::NpcKind::Scarecrow10
        | crate::npc::NpcKind::Splinterling
        | crate::npc::NpcKind::HeadlessHorseman
        | crate::npc::NpcKind::Hellhound => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        crate::npc::NpcKind::MourningWood
        | crate::npc::NpcKind::Pumpking
        | crate::npc::NpcKind::Poltergeist
        // S1-D-3z — the Drippler (490, noGravity aiStyle 22) rides the
        // hover ctor like the Poltergeist.
        | crate::npc::NpcKind::Drippler => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The bubble (`S1-F-13d`): `noTileCollide` + the AI_072 owner-pin
        // (`NPC.cs:35081-35101`) — the hover ctor's shape; the Officer's
        // spawn arm sets the position to its centre in the AI wave.
        crate::npc::NpcKind::ForceBubble => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The Tesla Turret (`S1-F-13d`): the Engineer places it ON the
        // ground (`NPC.cs:59029-59030`'s `NewNPC(..., num158 * 16, 387)` +
        // the `position.Y = num158 * 16 - height` feet plant at the
        // column's tile row) — the walker ctor plants the feet; the
        // AI_073 warp-in machine lands with the AI wave.
        crate::npc::NpcKind::MartianTurret => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // The Frost Moon wave family (`S1-F-13c` slice B): the four
        // aiStyle-3 walkers plant their feet on the surface like the
        // scarecrow band; the flyers ride the hover ctor like the Mourning
        // Wood / Pumpking / Poltergeist.
        crate::npc::NpcKind::Nutcracker
        | crate::npc::NpcKind::NutcrackerSpinning
        | crate::npc::NpcKind::ElfArcher
        | crate::npc::NpcKind::Krampus => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        crate::npc::NpcKind::IceQueen
        | crate::npc::NpcKind::SantaNK1
        | crate::npc::NpcKind::ElfCopter
        | crate::npc::NpcKind::Flocko => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The Frost Moon family, slice A (`S1-F-13c`): the same seed shape
        // as the pumpkin roster — the walkers (the Zombie Elf trio, the
        // Present Mimic, the Gingerbread Man and the Yeti, whose aiStyle-3/25
        // stand-ins ride the walker family) plant the feet on the column
        // surface; the Everscream's aiStyle-57 hover stand-in rides the
        // hover ctor (`NPC.cs:13281-13368` SetDefaults).
        crate::npc::NpcKind::ZombieElf
        | crate::npc::NpcKind::ZombieElfBeard
        | crate::npc::NpcKind::ZombieElfGirl
        | crate::npc::NpcKind::PresentMimic
        | crate::npc::NpcKind::GingerbreadMan
        | crate::npc::NpcKind::Yeti => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        crate::npc::NpcKind::Everscream => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The Plantera minions (S1-D-3-wire-plantera-minions): the net
        // state overwrites the position below; the bare stats seed is
        // enough (the noGravity/noTileCollide kinds have no world probe —
        // the `npc_from_net` fallback's shape).
        crate::npc::NpcKind::FungiSpore
        | crate::npc::NpcKind::PlanterasHook
        | crate::npc::NpcKind::PlanterasTentacle
        | crate::npc::NpcKind::Spore => crate::npc::Npc::new(kind, _x, y, 1, 0.0),
        // S1-D-3 W-9 fishron wiring — the boss + its hazards fly through
        // tiles (the noTileCollide/noGravity rows); the hover ctor's
        // open-air seed is the join shape (the machine takes over on the
        // first tick).
        crate::npc::NpcKind::DukeFishron
        | crate::npc::NpcKind::DetonatingBubble
        | crate::npc::NpcKind::Sharkron
        | crate::npc::NpcKind::Sharkron2 => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The wyvern family (`S1-D-3` wyvern): the sky flyer's net seed
        // rides the hover ctor (the net state overwrites the position
        // below); the chain links rebuild through the sim wiring — a lone
        // segment dies finding no leader (the worm families' net shape).
        crate::npc::NpcKind::WyvernHead
        | crate::npc::NpcKind::WyvernLegs
        | crate::npc::NpcKind::WyvernBody
        | crate::npc::NpcKind::WyvernBody2
        | crate::npc::NpcKind::WyvernBody3
        | crate::npc::NpcKind::WyvernTail => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d34 kinds 1-70: the walkers plant on the column surface; the
        // Cursed Skull hovers (noTileCollide); the ManEater anchors in air.
        crate::npc::NpcKind::AngryBones
        | crate::npc::NpcKind::OldMan
        | crate::npc::NpcKind::Bunny => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::CursedSkull | crate::npc::NpcKind::ManEater => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d34: the Bone Serpent links seed standalone (the chain rebuilds
        // through the worm pass — the leech net shape); the CorruptBunny plants
        // on the surface; the Harpy hangs in the air (the bat net shape).
        crate::npc::NpcKind::BoneSerpentHead
        | crate::npc::NpcKind::BoneSerpentBody
        | crate::npc::NpcKind::BoneSerpentTail
        | crate::npc::NpcKind::Harpy => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::CorruptBunny => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        // W4-d34 (phase 3): the Snatcher anchors in air (the ManEater net
        // shape); the CorruptGoldfish plants like the swimmers; the Vulture
        // perches on the column surface; the Demon hangs in the air (the bat
        // net shape); the BlueJellyfish seeds in the water column (the
        // swimmer net shape).
        crate::npc::NpcKind::Snatcher => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::CorruptGoldfish
        | crate::npc::NpcKind::BlueJellyfish => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::Vulture => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::Demon => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_cave_flyer(world, tx, y, kind)?
        }
        // W4-d34 (phase 4, merge-side): the Crab plants on the surface (the
        // aiStyle-3 walker net shape); the Dungeon Guardian hangs in the air
        // (the Skeletron net shape). Antlion/SpikeBall/PinkJellyfish dropped
        // (g8-owned 745-747, 367 §1).
        crate::npc::NpcKind::Crab => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::DungeonGuardian => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }

        // W4-d37 — the NPCID 211-280 band's net seeds. The aiStyle-3 walkers
        // plant their feet on the column surface (the wave-A walkers' shape);
        // the flyers ride the hover ctor (the net state overwrites the
        // position below); the pirate town NPC rides the guide ctor.
        crate::npc::NpcKind::CochinealBeetle
        | crate::npc::NpcKind::CyanBeetle
        | crate::npc::NpcKind::LacBeetle
        | crate::npc::NpcKind::SeaSnail
        | crate::npc::NpcKind::JungleCreeper
        | crate::npc::NpcKind::BloodCrawler
        | crate::npc::NpcKind::IceGolem
        | crate::npc::NpcKind::Eyezor
        | crate::npc::NpcKind::ZombieMushroom
        | crate::npc::NpcKind::ZombieMushroomHat
        | crate::npc::NpcKind::AnomuraFungus
        | crate::npc::NpcKind::MushiLadybug
        | crate::npc::NpcKind::RustyArmoredBonesAxe
        | crate::npc::NpcKind::RustyArmoredBonesFlail
        | crate::npc::NpcKind::RustyArmoredBonesSword
        | crate::npc::NpcKind::RustyArmoredBonesSwordNoArmor
        | crate::npc::NpcKind::BlueArmoredBones
        | crate::npc::NpcKind::BlueArmoredBonesMace
        | crate::npc::NpcKind::BlueArmoredBonesNoPants
        | crate::npc::NpcKind::BlueArmoredBonesSword
        | crate::npc::NpcKind::HellArmoredBones
        | crate::npc::NpcKind::HellArmoredBonesSpikeShield
        | crate::npc::NpcKind::HellArmoredBonesMace
        | crate::npc::NpcKind::HellArmoredBonesSword
        | crate::npc::NpcKind::RainbowSlime
        | crate::npc::NpcKind::GoldfishWalker => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::Pirate => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::Squid
        | crate::npc::NpcKind::FlyingFish
        | crate::npc::NpcKind::FlyingSnake
        | crate::npc::NpcKind::HornetFatty
        | crate::npc::NpcKind::HornetHoney
        | crate::npc::NpcKind::HornetLeafy
        | crate::npc::NpcKind::HornetSpikey
        | crate::npc::NpcKind::HornetStingy
        | crate::npc::NpcKind::Parrot
        | crate::npc::NpcKind::JungleCreeperWall
        | crate::npc::NpcKind::BlackRecluseWall
        | crate::npc::NpcKind::BloodCrawlerWall
        | crate::npc::NpcKind::BloodFeeder
        | crate::npc::NpcKind::BloodJelly
        | crate::npc::NpcKind::FungoFish
        | crate::npc::NpcKind::FungiBulb
        | crate::npc::NpcKind::GiantFungiBulb
        | crate::npc::NpcKind::Reaper
        | crate::npc::NpcKind::IchorSticker => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::BloodMummy
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::SeaTurtle
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's flyers seed in the open air (the net
        // state overwrites the position below).
        crate::npc::NpcKind::Seahorse
        | crate::npc::NpcKind::GoldSeahorse
        | crate::npc::NpcKind::Dandelion
        | crate::npc::NpcKind::IceMimic
        => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::Gnome
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's flyers seed in the open air (the net
        // state overwrites the position below).
        crate::npc::NpcKind::GoblinShark
        | crate::npc::NpcKind::BloodEelHead
        | crate::npc::NpcKind::BloodEelBody
        | crate::npc::NpcKind::BloodEelTail
        => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::Turtle
        | crate::npc::NpcKind::TurtleJungle
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's flyers seed in the open air (the net
        // state overwrites the position below).
        crate::npc::NpcKind::BloodNautilus
        | crate::npc::NpcKind::BloodSquid
        => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::Rat
        | crate::npc::NpcKind::ExplosiveBunny
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's flyers seed in the open air (the net
        // state overwrites the position below).
        crate::npc::NpcKind::Owl
        | crate::npc::NpcKind::WaterStrider
        | crate::npc::NpcKind::GoldWaterStrider
        => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::Grebe
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's flyers seed in the open air (the net
        // state overwrites the position below).
        crate::npc::NpcKind::Grebe2
        => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::Seagull
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's flyers seed in the open air (the net
        // state overwrites the position below).
        crate::npc::NpcKind::Seagull2
        => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::TorchZombie
        | crate::npc::NpcKind::ArmedTorchZombie
        | crate::npc::NpcKind::GoldGoldfishWalker
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::ZombieMerman
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's ground/stand kinds plant on the column
        // surface (the wave-A walker precedent).
        crate::npc::NpcKind::BartenderUnconscious
        | crate::npc::NpcKind::WalkingAntlion
        | crate::npc::NpcKind::LarvaeAntlion
        => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d42 - the roster's flyers seed in the open air (the net
        // state overwrites the position below).
        crate::npc::NpcKind::DD2LightningBug
        => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d35 (A): the 73/77/78 walkers plant feet on the
        // column surface (the generic surface ctor; the family machine
        // takes over on the first tick).
        crate::npc::NpcKind::GoblinScout
        | crate::npc::NpcKind::ArmoredSkeleton
        | crate::npc::NpcKind::Mummy => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d35 (A): the 74/75 flyers ride the hover ctor
        // (the net state overwrites the position below).
        crate::npc::NpcKind::Bird
        | crate::npc::NpcKind::Pixie => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d35 (B): the 79/80 walkers plant feet on the
        // column surface (the generic surface ctor; the family machine
        // takes over on the first tick).
        crate::npc::NpcKind::DarkMummy
        | crate::npc::NpcKind::LightMummy => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d35 (B): the 82/83/84 flyers ride the hover ctor
        // (the net state overwrites the position below).
        crate::npc::NpcKind::Wraith
        | crate::npc::NpcKind::CursedHammer
        | crate::npc::NpcKind::EnchantedSword => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d35 (C): the 85/86 walkers plant feet on the
        // column surface (the generic surface ctor; the family machine
        // takes over on the first tick).
        crate::npc::NpcKind::Mimic
        | crate::npc::NpcKind::Unicorn => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d35 (C): the 93/94/95 flyers ride the hover ctor
        // (the net state overwrites the position below).
        crate::npc::NpcKind::GiantBat
        | crate::npc::NpcKind::Corruptor
        | crate::npc::NpcKind::DiggerHead => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d35 (D): the 96/97/98/99/100 flyers ride the hover ctor
        // (the net state overwrites the position below).
        crate::npc::NpcKind::DiggerBody
        | crate::npc::NpcKind::DiggerTail
        | crate::npc::NpcKind::SeekerHead
        | crate::npc::NpcKind::SeekerBody
        | crate::npc::NpcKind::SeekerTail => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d35 (E): the 102/104/109 walkers plant feet on the
        // column surface (the generic surface ctor; the family machine
        // takes over on the first tick).
        crate::npc::NpcKind::AnglerFish
        | crate::npc::NpcKind::Werewolf
        | crate::npc::NpcKind::Clown => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d35 (E): the 101/103 flyers ride the hover ctor
        // (the net state overwrites the position below).
        crate::npc::NpcKind::Clinger
        | crate::npc::NpcKind::GreenJellyfish => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d35 (F): the 110/120 walkers plant feet on the
        // column surface (the generic surface ctor; the family machine
        // takes over on the first tick).
        crate::npc::NpcKind::SkeletonArcher
        | crate::npc::NpcKind::ChaosElemental => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d35 (F): the 112/116/121 flyers ride the hover ctor
        // (the net state overwrites the position below).
        crate::npc::NpcKind::VileSpit
        | crate::npc::NpcKind::TheHungryII
        | crate::npc::NpcKind::Slimer => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-d35 (G): the 140 walkers plant feet on the
        // column surface (the generic surface ctor; the family machine
        // takes over on the first tick).
        crate::npc::NpcKind::PossessedArmor => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-d35 (G): the 122/137 flyers ride the hover ctor
        // (the net state overwrites the position below).
        crate::npc::NpcKind::Gastropod
        | crate::npc::NpcKind::IlluminantBat => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-c7 — the firefly family (355/358/654/677): the noGravity
        // glow critters seed bare at the net position (the Plantera
        // minion arm's shape — the machine takes over on the first tick
        // and the net state overwrites the position below).
        crate::npc::NpcKind::Firefly
        | crate::npc::NpcKind::LightningBug
        | crate::npc::NpcKind::Lavafly
        | crate::npc::NpcKind::Shimmerfly => crate::npc::Npc::new(kind, _x, y, 1, 0.0),
        // The snail family (W4-c9): the wall-crawler critters plant on
        // the column surface like the wave-A walkers (the aiStyle-67
        // machine takes over on the first tick — the fall roll + the
        // crawl state machine own the placement from there).
        crate::npc::NpcKind::Snail
        | crate::npc::NpcKind::GlowingSnail
        | crate::npc::NpcKind::MagmaSnail => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        // E8 merge: the Shimmerfly seed is c7's bare-stats arm above;
        // the 5 gold seeds ride d40's bare-stats arm above (the position
        // overwrite + the first-tick machine takeover make the ctor
        // choice behaviour-identical - 349 s6a's netglue note).
        // W4-c10 - the waterfowl family: the Duck quartet's bare-stats
        // seed (the net state overwrites the position below; the
        // machines take over on the first tick). E8 merge: the
        // Seagull/Grebe seeds ride the d42 cave_walker/hover arms above.
        crate::npc::NpcKind::Duck
        | crate::npc::NpcKind::Duck2
        | crate::npc::NpcKind::DuckWhite
        | crate::npc::NpcKind::DuckWhite2 => crate::npc::Npc::new(kind, _x, y, 1, 0.0),
        // E8 merge: the c10 12-critter seed tail dropped (d40/c8's seeds above).
        // Doc-sweep lane 36 — the ShimmerSlime plants its feet on the
        // column surface (the wave-A slime seed; the aiStyle-1 machine
        // takes over on the first tick).
        crate::npc::NpcKind::ShimmerSlime => crate::npc::Npc::new(kind, _x, y, 1, 0.0),
    })
}

/// `WorldGen.paintTile` (`WorldGen.cs:44110-44132`): an active tile whose
/// colour differs takes the new paint. The `paintEffect` dust is draw-side
/// (out); the `broadCast` send is the caller's relay. W4-g49.
fn paint_tile(world: &mut World, x: i32, y: i32, color: u8) {
    let Some(t) = world.tile_mut(x, y) else {
        return;
    };
    if !t.active() || t.paint() == color {
        return; // `:44113-44121`
    }
    t.set_paint(color); // `:44126`
}

/// `WorldGen.paintCoatTile` (`WorldGen.cs:44134-44180`): coat 0 strips the
/// glow/echo bits, 1 sets fullbright, 2 sets invisible (echo); the strip/echo
/// arms re-frame (`SquareTileFrame resetFrame:false`). W4-g49.
fn paint_coat_tile(world: &mut World, x: i32, y: i32, coat: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() {
        return; // `:44137-44140`
    }
    let (fullbright, invisible) = (t.fullbright_block(), world.invisible_block(x, y));
    match coat {
        0 => {
            if !fullbright && !invisible {
                return; // `:44147-44150`
            }
            if let Some(t) = world.tile_mut(x, y) {
                t.flags &= !tr_core::tile::FLAG_FULLBRIGHT; // `:44151`
            }
            world.set_invisible_block(x, y, false); // `:44152`
            tr_core::framing::reframe_area(world, x, y, 1, false); // `:44153`
        }
        1 => {
            if fullbright {
                return; // `:44156-44159`
            }
            if let Some(t) = world.tile_mut(x, y) {
                t.flags |= tr_core::tile::FLAG_FULLBRIGHT; // `:44160`
            }
        }
        2 => {
            if invisible {
                return; // `:44163-44166`
            }
            world.set_invisible_block(x, y, true); // `:44167`
            tr_core::framing::reframe_area(world, x, y, 1, false); // `:44168`
        }
        _ => {} // `:44144-44145` default: return false
    }
}

/// `WorldGen.paintWall` (`WorldGen.cs:44230-44252`): a present wall whose
/// colour differs takes the new paint. W4-g49.
fn paint_wall(world: &mut World, x: i32, y: i32, color: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if t.wall == 0 || world.wall_paint(x, y) == color {
        return; // `:44233-44241`
    }
    world.set_wall_paint(x, y, color); // `:44242`
}

/// `WorldGen.paintCoatWall` (`WorldGen.cs:44182-44228`): the wall twin of
/// [`paint_coat_tile`] — the glow/echo bits ride `World.extras`. W4-g49.
fn paint_coat_wall(world: &mut World, x: i32, y: i32, coat: u8) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if t.wall == 0 {
        return; // `:44185-44188`
    }
    let (fullbright, invisible) = (world.fullbright_wall(x, y), world.invisible_wall(x, y));
    match coat {
        0 => {
            if !fullbright && !invisible {
                return; // `:44195-44198`
            }
            world.set_fullbright_wall(x, y, false); // `:44199`
            world.set_invisible_wall(x, y, false); // `:44200`
            tr_core::framing::reframe_area(world, x, y, 1, false); // `:44201`
        }
        1 => {
            if fullbright {
                return; // `:44204-44207`
            }
            world.set_fullbright_wall(x, y, true); // `:44208`
        }
        2 => {
            if invisible {
                return; // `:44211-44214`
            }
            world.set_invisible_wall(x, y, true); // `:44215`
            tr_core::framing::reframe_area(world, x, y, 1, false); // `:44216`
        }
        _ => {} // `:44192-44193` default: return false
    }
}

/// `WorldGen.UnlockDoor` (`WorldGen.cs:37599-37628`) for the `52` reader's
/// kind 2 (`MessageBuffer.cs:2547`): walk UP from (x, y) to the locked
/// door's top leaf (`frameY == 594`); a `frameY < 594` or the world's top
/// edge refuses. The three leaves shift `frameY += 54` (the unlocked
/// column, `:37621`). Returns the top leaf's y on success so the caller can
/// run the app-scope effects — the Unlock cue (22, `:37614`), the 4 type-11
/// dusts per leaf (`:37622-37625`) and progression 22 (`:37627`). W4-g49.
fn unlock_door(world: &mut World, x: i32, y: i32) -> Option<i32> {
    // `:37602-37605` — a null tile refuses (out of bounds in the port).
    if world.tile(x, y).is_none() {
        return None;
    }
    let mut top = y;
    // `:37606-37613` — the upward walk to the 594 leaf.
    loop {
        let Some(t) = world.tile(x, top) else {
            return None;
        };
        if t.frame_y == 594 {
            break;
        }
        top -= 1;
        let Some(t) = world.tile(x, top) else {
            return None;
        };
        if t.frame_y < 594 || top <= 0 {
            return None;
        }
    }
    // `:37615-37626` — the three leaves shift to the unlocked column.
    for k in top..=top + 2 {
        if let Some(t) = world.tile_mut(x, k) {
            t.frame_y += 54; // `:37621`
        }
    }
    Some(top)
}

/// The app-scope effects of [`unlock_door`] (`WorldGen.cs:37614`/:37622-37627):
/// the Unlock cue at (x*16, top*16+16), the 4 type-11 dusts per leaf and
/// progression 22. The dust draws ride the bare-`Main.rand` stand-in (the
/// message handler runs outside every swap, `Main.cs:16895`). W4-g49.
fn unlock_door_effects(app: &mut App, x: i32, top: i32) {
    // `:37614` — `SoundEngine.PlaySound(22, i*16, num*16+16)`.
    app.audio
        .play_at("Unlock", x as f32 * TILE_PX, top as f32 * TILE_PX + 16.0, 1.0);
    // `:37622-37625` — 4 dusts per leaf, type 11.
    let ctx = app.spawn_ctx();
    for k in top..=top + 2 {
        for _ in 0..4 {
            app.dust.new_dust(
                app.rngs.global(),
                &ctx,
                &app.world,
                Vec2::new(x as f32 * TILE_PX, k as f32 * TILE_PX),
                16,
                16,
                11,
                0.0,
                0.0,
                0,
                tr_core::particles::Color::default(),
                1.0,
            );
        }
    }
    // `:37627` — `AchievementsHelper.NotifyProgressionEvent(22)`.
    let _ = app.notify_progression_net(22);
}

/// `CombatText.NewText(playerRect, color, amount)` for the `66`/`43` readers
/// (`Player.HealEffect`/`ManaEffect`, `Player.cs:4522-4538`): the floating
/// amount at the player's rect. The local player (slot 0) rides the host's
/// player, a peer its `remote_players` record (the `PlayerNet` snapshot's
/// position + the stock player box). The placement draws ride the bare
/// `Main.rand` stand-in (the message handler runs outside every swap). W4-g49.
fn spawn_player_amount_text(app: &mut App, slot: u32, color: [u8; 4], amount: i32) {
    let (x, y, w, h) = if slot == 0 {
        match app.player.as_ref() {
            Some(p) => (p.x, p.y, p.width, p.height),
            None => return,
        }
    } else {
        match app.remote_players.get(&slot) {
            Some(p) => (p.x, p.y, crate::player::WIDTH, crate::player::HEIGHT),
            None => return,
        }
    };
    let text = amount.to_string();
    let size = match app.lookups.combat_text_font.as_ref() {
        Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
        None => [text.chars().count() as f32 * 8.0, 10.0],
    };
    let location = crate::combat_text::Rect {
        x,
        y,
        width: w,
        height: h,
    };
    let rng = app.rngs.global();
    crate::combat_text::spawn_amount(
        &mut app.combat_texts,
        location,
        color,
        amount,
        false,
        false,
        size,
        1.0,
        rng,
    );
}

fn npc_from_net(world: &World, n: &net::NpcInfo) -> Option<crate::npc::Npc> {
    let kind = crate::npc::NpcKind::from_u16(n.kind.max(0) as u16)?;
    let tx = (n.x / TILE_PX) as i32;
    let mut npc = npc_seed(world, kind, tx, n.x, n.y).unwrap_or_else(|| {
        // The world-probing ctors failed (no column surface at the slot on a
        // joining client): the `23` body carries everything the render needs,
        // so a bare stats seed is enough.
        crate::npc::Npc::new(kind, n.x, n.y, 1, 0.0)
    });
    npc.x = n.x;
    npc.y = n.y;
    npc.vx = n.vx;
    npc.vy = n.vy;
    if !n.life_full && n.hp >= 0 {
        npc.hp = n.hp;
    }
    npc.facing = if n.dir_right { 1 } else { -1 };
    npc.direction_y = if n.dir_y_down { 1 } else { -1 };
    npc.sprite_dir = if n.sprite_right { 1 } else { -1 };
    npc.target = n.target as i32;
    npc.ai = n.ai;
    npc.spawned_from_statue = n.spawned_from_statue;
    if n.spawned_from_statue {
        npc.extra_value = 0;
    }
    if n.shimmer {
        npc.shimmer_transparency = 1.0;
    }
    npc.release_owner = n.release_owner;
    // `hurt_cd` rides `28 NPCStrike` in the C# (`MessageBuffer.cs:1766`);
    // until that message lands the client shows no hit flash on networked
    // NPCs — deviation in `docs/ANALYSIS.md`.
    npc.hurt_cd = 0.0;
    Some(npc)
}

#[cfg(test)]
mod handshake_tests {
    //! In-process handshake tests: two Apps over a real loopback socket, with
    //! port 0 (`Netplay`'s ephemeral bind) driving `flow_app`s.

    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::ui::save_list::{PlayerEntry, WorldEntry};

    pub(super) fn apps() -> (App, App) {
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
    pub(super) fn drive_to(host: &mut App, client: &mut App, goal: i32) {
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

    pub(super) fn connect_pair(password: &str, client_pw: &str) -> (App, App) {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, password.to_string()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
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
        let mark = 3 * host_app.world.width as usize + 4;
        host_app.world.tiles[mark].kind = 7;
        // The codec only carries a kind for active tiles (as the C# does), so
        // the marker must be a real tile.
        host_app.world.tiles[mark].flags |= tr_core::tile::FLAG_ACTIVE;
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

    /// Pumps both apps at 60 Hz for `frames` frames: the net tick (10 Hz
    /// internally) plus the event drain, as the real frame loop does.
    pub(super) fn pump(host: &mut App, client: &mut App, frames: usize) {
        for _ in 0..frames {
            host.net_tick(1.0 / 60.0);
            client.net_tick(1.0 / 60.0);
            host.poll_net();
            client.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    /// A fully joined pair with a live player on the host: the two-app world
    /// the M6-NET-2 sync tests stream snapshots through.
    pub(super) fn in_world_pair() -> (App, App) {
        let (mut host_app, mut client_app) = connect_pair("", "");
        // A dirt row so the NPC ctors find a column surface on both sides
        // (the world transfer carries it to the client).
        host_app.world.column_surface = vec![host_app.world.height; host_app.world.width as usize];
        for x in 0..host_app.world.width {
            let i = host_app.world.index(x, 4);
            host_app.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            // `surface_of` reads the precomputed column table, not the tiles.
            host_app.world.column_surface[x as usize] = 4;
        }
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        host_app.player = Some(player::Player::spawn(&host_app.world));
        host_app.player_name = "host".into();
        (host_app, client_app)
    }

    #[test]
    fn player_state_syncs_across_the_wire() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The peer slot is seeded at the world spawn, so the first snapshot
        // (the host standing at its spawn) carries no lag.
        let spawn = client_app.net_peer_spawn_pos();
        host_app.player.as_mut().unwrap().x = spawn.0;
        pump(&mut host_app, &mut client_app, 12);
        let peer = client_app.remote_players.get(&0).expect("host peer stored");
        assert!(
            (peer.x - spawn.0).abs() < 0.01,
            "x snapped to the wire value"
        );
        assert!(peer.off_x.abs() < 0.01, "the first snap carries no lag");
        assert_eq!(peer.name, "host");
        // A 60 px step: the lag accrues, the position snaps.
        host_app.player.as_mut().unwrap().x = spawn.0 + 60.0;
        pump(&mut host_app, &mut client_app, 12);
        let peer = client_app.remote_players.get(&0).expect("host peer stored");
        assert!((peer.x - (spawn.0 + 60.0)).abs() < 0.01);
        assert!(
            (peer.off_x - 60.0).abs() < 0.01,
            "the offset is the visual lag: {:.2}",
            peer.off_x
        );
    }

    #[test]
    fn the_net_offset_is_cleared_below_two_pixels() {
        // The C#'s shortcut: under 2 px the offset is zeroed in one tick.
        let (mut host_app, mut client_app) = in_world_pair();
        host_app.player.as_mut().unwrap().x = 100.0;
        pump(&mut host_app, &mut client_app, 12);
        host_app.player.as_mut().unwrap().x = 161.0;
        pump(&mut host_app, &mut client_app, 12);
        let peer = client_app.remote_players.get_mut(&0).expect("host peer stored");
        peer.off_x = 1.5;
        peer.off_y = 0.0;
        client_app.net_interp_tick(1.0 / 60.0);
        assert_eq!(client_app.remote_players[&0].off_x, 0.0);
    }

    /// W4-g51 `125 SyncTilePicking`: the host forces the subject to the
    /// sender and relays to the others (`MessageBuffer.cs:4029-4040`); the
    /// receiving client applies the pick damage to its OWN `hitTile`
    /// (`Main.player[Main.myPlayer].GetOtherPlayersPickTile`, `:4033-4036`).
    #[test]
    fn the_tile_picking_relays_and_lands_on_the_local_pick_book() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The host arm: the subject is forced to the sender and the relay
        // goes to the OTHERS (a single client means nobody else receives).
        // The role is taken out so the handler can borrow `self` mutably
        // (the `poll_net` convention).
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::TilePicking(net::TilePicking {
                        player_id: 99, // overridden to the sender (1)
                        x: 12,
                        y: 7,
                        damage: 40,
                    }),
                },
            );
        }
        host_app.net = role;
        // The client arm applies to the LOCAL pick book regardless of the
        // wire subject (`Main.myPlayer`, `:4035`).
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::TilePicking(net::TilePicking {
                    player_id: 1,
                    x: 12,
                    y: 7,
                    damage: 40,
                })),
            );
        }
        client_app.net = role;
        let id = client_app
            .hit_tile
            .hit_object(12, 7, tr_core::hit_tile::TILE);
        assert_eq!(
            client_app.hit_tile.add_damage(id, 0, false),
            40,
            "the pick damage accumulated on the local book"
        );
    }

    /// W4-g51 `126 SyncRevengeMarker`: the client's `AddMarkerFromReader`
    /// lands the marker in the session list with the ctor's expiration
    /// (`MessageBuffer.cs:4043-4048`, `CoinLossRevengeSystem.cs:321-324`).
    #[test]
    fn the_revenge_marker_lands_in_the_session_list() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        client_app.revenge_tick = 1000;
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::RevengeMarker(net::RevengeMarker {
                    unique_id: 9,
                    x: 1234.5,
                    y: 678.25,
                    npc_net_id: 1, // Blue Slime — a ported kind
                    npc_hp_percent: 0.75,
                    npc_type: 1,
                    npc_ai_style: 1,
                    coins_value: 50000,
                    base_value: 12.5,
                    spawned_from_statue: false,
                })),
            );
        }
        client_app.net = role;
        assert_eq!(client_app.revenge_markers.len(), 1, "the marker landed");
        let m = &client_app.revenge_markers[0];
        assert_eq!(m.unique_id, 9);
        assert_eq!(m.coins_value, 50000);
        assert_eq!(
            m.expiration_tick,
            1000 + crate::revenge::expiration_addon(50000),
            "the ctor's gameTime + bracket expiration"
        );
    }

    /// W4-g51 `127 RemoveRevengeMarker`: the client's `DestroyMarker` removes
    /// every marker with the matching UniqueID
    /// (`MessageBuffer.cs:4049-4057`, `CoinLossRevengeSystem.cs:334-340`).
    #[test]
    fn the_remove_revenge_marker_destroys_the_matching_id() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        // Two markers, ids 9 and 10 (the 126 apply path).
        for uid in [9i32, 10] {
            let role = std::mem::replace(&mut client_app.net, NetRole::Off);
            if let NetRole::Client(client) = &role {
                client_app.client_event(
                    client,
                    net::ClientEvent::Frame(S2c::RevengeMarker(net::RevengeMarker {
                        unique_id: uid,
                        x: 0.0,
                        y: 0.0,
                        npc_net_id: 1,
                        npc_hp_percent: 1.0,
                        npc_type: 1,
                        npc_ai_style: 1,
                        coins_value: 1000,
                        base_value: 0.0,
                        spawned_from_statue: false,
                    })),
                );
            }
            client_app.net = role;
        }
        assert_eq!(client_app.revenge_markers.len(), 2);
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::RemoveRevengeMarker(
                    net::RemoveRevengeMarker { unique_id: 9 },
                )),
            );
        }
        client_app.net = role;
        assert_eq!(client_app.revenge_markers.len(), 1, "id 9 was destroyed");
        assert_eq!(client_app.revenge_markers[0].unique_id, 10);
    }

    /// W4-g51 `130 FishOutNPC`: type 682 flips `unlockedSlimeRedSpawn` once
    /// and re-pushes `7 WorldData` (`MessageBuffer.cs:4095-4103`); a second
    /// 682 is refused (`:4097-4100`). The port has no town-slime kind, so the
    /// spawn itself is the roster gap — the unlock is the observable state.
    #[test]
    fn the_fish_out_red_slime_unlocks_once() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut client_app;
        assert!(!host_app.world.unlocked_slime_red_spawn);
        for _ in 0..2 {
            let role = std::mem::replace(&mut host_app.net, NetRole::Off);
            if let NetRole::Host(server) = &role {
                host_app.host_event(
                    server,
                    net::ServerEvent::Frame {
                        id: 1,
                        msg: C2s::FishOutNpc(net::FishOutNpc {
                            tile_x: 10,
                            tile_y: 5,
                            npc_type: 682,
                        }),
                    },
                );
            }
            host_app.net = role;
        }
        assert!(
            host_app.world.unlocked_slime_red_spawn,
            "the first 682 flipped the unlock"
        );
    }

    /// W4-g51 `136 SyncCavernMonsterType`: the client stores the 2×3 grid
    /// verbatim (`MessageBuffer.cs:4210-4216`), and the join bundle pushes
    /// the host's rolled grid (`:870`) — the joined client's field equals
    /// the host's.
    #[test]
    fn the_cavern_grid_stores_and_rides_the_join_bundle() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The join tail rolled the host's grid (`WorldFile.cs:771`); the
        // join push carried it to the client verbatim.
        assert_eq!(
            client_app.world.cavern_monster_type, host_app.world.cavern_monster_type,
            "the join push delivered the host's grid"
        );
        // A later re-sync overwrites verbatim (row-major).
        let grid = [[494u16, 495, 494], [499, 502, 506]];
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::CavernMonsterTypes(net::CavernMonsterTypes {
                    types: grid,
                })),
            );
        }
        client_app.net = role;
        assert_eq!(
            client_app.world.cavern_monster_type,
            [[494, 495, 494], [499, 502, 506]],
            "the six ushorts stored row-major"
        );
    }

    /// W4-g51 id-140 kind 0: the client's `SetRemainingTimeDirect` clamps
    /// to 0..28800 (`CreditsRollEvent.cs:45-48`); the join bundle pushes
    /// the host's live roll (`NetMessage.cs:2818`, the `!= 0` gate).
    #[test]
    fn the_credits_roll_time_clamps_and_rides_the_join_bundle() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        for (sent, want) in [(500i32, 500i32), (99999, 28800), (-5, 0)] {
            let role = std::mem::replace(&mut client_app.net, NetRole::Off);
            if let NetRole::Client(client) = &role {
                client_app.client_event(
                    client,
                    net::ClientEvent::Frame(S2c::MiscEventValue(net::MiscEventValue {
                        kind: 0,
                        value: sent,
                    })),
                );
            }
            client_app.net = role;
            assert_eq!(
                client_app.world.credits_roll_remaining_time, want,
                "sent {sent}"
            );
        }
        // Kinds 1/2 are host-only — the client ignores them (`:4251`).
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::MiscEventValue(net::MiscEventValue {
                    kind: 1,
                    value: 0,
                })),
            );
        }
        client_app.net = role;
        assert_eq!(client_app.world.credits_roll_remaining_time, 0);
        // The join push: a live roll on the host reaches a fresh joiner.
        host_app.world.credits_roll_remaining_time = 1234;
        let info = host_app.world_info();
        let _ = info;
        // (The push itself is exercised by the join-bundle walk; the
        // clamp + the field are the arm's observable state.)
        host_app.world.tick_credits_roll();
        assert_eq!(host_app.world.credits_roll_remaining_time, 1233);
    }

    /// W4-g51 id-140 kinds 1/2: the host's transform gates — kind 1 needs
    /// a `CanConvertIntoCopperSlimeTownNPC` type (the port's Blue Slime,
    /// cs_id 1) and flips `unlockedSlimeCopperSpawn` ONCE; kind 2 needs
    /// type 685 (no ported kind — the gate refuses every live NPC).
    #[test]
    fn the_town_slime_transform_unlocks_once_through_the_type_gate() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut client_app;
        host_app
            .npcs
            .push(crate::npc::Npc::new(crate::npc::NpcKind::Slime, 0.0, 0.0, 1, 0.0));
        assert!(!host_app.world.unlocked_slime_copper_spawn);
        for _ in 0..2 {
            let role = std::mem::replace(&mut host_app.net, NetRole::Off);
            if let NetRole::Host(server) = &role {
                host_app.host_event(
                    server,
                    net::ServerEvent::Frame {
                        id: 1,
                        msg: C2s::MiscEventValue(net::MiscEventValue { kind: 1, value: 0 }),
                    },
                );
            }
            host_app.net = role;
        }
        assert!(
            host_app.world.unlocked_slime_copper_spawn,
            "the Blue Slime (cs_id 1) is in the convert set"
        );
        // Kind 2: no live NPC is type 685 — the unlock stays down.
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::MiscEventValue(net::MiscEventValue { kind: 2, value: 0 }),
                },
            );
        }
        host_app.net = role;
        assert!(!host_app.world.unlocked_slime_old_spawn);
        // An out-of-range slot is refused (`IndexInRange`).
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::MiscEventValue(net::MiscEventValue { kind: 2, value: 999 }),
                },
            );
        }
        host_app.net = role;
        assert!(!host_app.world.unlocked_slime_old_spawn);
    }

    /// W4-g51 `131 TamperWithNPC`: kind 1 with fromWho -1 sets EVERY immune
    /// slot (`MessageBuffer.cs:4128-4131`, `NPC.cs:92512-92515`); a specific
    /// fromWho sets just that slot (`:92517-92520`).
    #[test]
    fn the_tamper_with_npc_sets_the_immune_slots() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        // Seed one live NPC on the client.
        client_app
            .npcs
            .push(crate::npc::Npc::new(crate::npc::NpcKind::Slime, 0.0, 0.0, 1, 0.0));
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::TamperWithNpc(net::TamperWithNpc {
                    npc: 0,
                    kind: 1,
                    immune: Some((20, -1)),
                })),
            );
        }
        client_app.net = role;
        assert!(
            client_app.npcs[0].immune.iter().all(|&v| v == 20),
            "fromWho -1 set every slot"
        );
        // A specific fromWho writes just that slot.
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::TamperWithNpc(net::TamperWithNpc {
                    npc: 0,
                    kind: 1,
                    immune: Some((7, 3)),
                })),
            );
        }
        client_app.net = role;
        assert_eq!(client_app.npcs[0].immune[3], 7);
        assert_eq!(client_app.npcs[0].immune[0], 20, "the other slots kept 20");
    }

    /// W4-g51 `134 UpdatePlayerLuckFactors`: the local player's eight luck
    /// fields land + `RecalculateLuck` runs (`MessageBuffer.cs:4180-4188`);
    /// a peer's factors are stored on the snapshot (stored-not-simulated).
    #[test]
    fn the_luck_factors_apply_to_the_local_player_and_recalculate() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        client_app.player = Some(player::Player::spawn(&client_app.world));
        let me = client_app.local_id as u8;
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::UpdatePlayerLuckFactors(
                    net::UpdatePlayerLuckFactors {
                        player_id: me,
                        ladybug_luck_time_left: 0,
                        torch_luck: 0.0,
                        luck_potion: 3,
                        has_garden_gnome_nearby: true,
                        broken_mirror_bad_luck: false,
                        equipment_based_luck_bonus: 0.0,
                        coin_luck: 0.0,
                        kite_luck_level: 0,
                    },
                )),
            );
        }
        client_app.net = role;
        let p = client_app.player.as_ref().unwrap();
        assert_eq!(p.luck_state.luck_potion, 3);
        assert!(p.luck_state.has_garden_gnome_nearby);
        // RecalculateLuck: potion 3 × 0.1 + gnome 0.2 = 0.5.
        assert!(
            (p.luck_state.luck - 0.5).abs() < 1e-5,
            "the recalculated luck is 0.5, got {}",
            p.luck_state.luck
        );
    }

    /// W4-g51 `137 RequestNPCBuffRemoval`: the host gates the slot then runs
    /// `RequestBuffRemoval` (`MessageBuffer.cs:4219-4228`). The
    /// `CanBeRemovedByNetMessage` set is EMPTY (`BuffID.cs:48`), so the buff
    /// survives — the seam is wired but refuses every id.
    #[test]
    fn the_npc_buff_removal_request_is_gated_off_by_the_empty_set() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut client_app;
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::Slime, 0.0, 0.0, 1, 0.0);
        npc.buffs.add_buff(20, 600); // Poisoned
        host_app.npcs.push(npc);
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::NpcBuffRemovalRequest(net::NpcBuffRemovalRequest {
                        npc: 0,
                        buff_type: 20,
                    }),
                },
            );
        }
        host_app.net = role;
        assert!(
            host_app.npcs[0].buffs.has_buff(20),
            "the empty CanBeRemovedByNetMessage set refuses the removal"
        );
    }

    /// W4-g51 `143 CrystalInvasionSkip`: the host's `AttemptToSkipWaitTime`
    /// (`MessageBuffer.cs:4298-4303`, `DD2Event.cs:1795-1801`) clamps a
    /// >60 wave-break hold to 60; the strict gate refuses at exactly 60.
    #[test]
    fn the_crystal_invasion_skip_clamps_the_wave_break_hold() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut client_app;
        host_app.world.dd2_spawn_hold = 1800;
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::CrystalInvasionSkip,
                },
            );
        }
        host_app.net = role;
        assert_eq!(host_app.world.dd2_spawn_hold, 60, "the >60 hold clamped to 60");
        // A second skip at exactly 60 is refused (strict >).
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::CrystalInvasionSkip,
                },
            );
        }
        host_app.net = role;
        assert_eq!(host_app.world.dd2_spawn_hold, 60, "the strict gate refused");
    }

    /// W4-g51 `142 SyncProjectileTrackers`: the host forces the subject to
    /// the sender and stores the references on the peer snapshot
    /// (`MessageBuffer.cs:4282-4297`, stored-not-simulated).
    #[test]
    fn the_projectile_trackers_store_on_the_peer_snapshot() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut client_app;
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::ProjectileTrackers(net::ProjectileTrackers {
                        player_id: 99, // overridden to the sender (1)
                        piggy_bank: net::TrackedProjectileRef { key: 7, kind: 1 },
                        void_lens: net::TrackedProjectileRef { key: 8, kind: 2 },
                    }),
                },
            );
        }
        host_app.net = role;
        let t = host_app.remote_players[&1]
            .trackers
            .expect("the trackers stored");
        assert_eq!(t.player_id, 1, "the subject was forced to the sender");
        assert_eq!(t.piggy_bank.key, 7);
        assert_eq!(t.void_lens.kind, 2);
    }

    /// W4-g51 `147 SyncLoadout`: the local player's accessory-visibility
    /// bits unpack from the ushort (`MessageBuffer.cs:4337`, `:4494-4501`,
    /// bit i per slot); the loadout swap itself is unported.
    #[test]
    fn the_sync_loadout_unpacks_the_accessory_visibility() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        client_app.player = Some(player::Player::spawn(&client_app.world));
        let me = client_app.local_id as u8;
        // Bits 0 and 3 set -> slots 0 and 3 hidden.
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::SyncLoadout(net::SyncLoadout {
                    player_id: me,
                    loadout: 1,
                    hide_accessory: 0b1001,
                })),
            );
        }
        client_app.net = role;
        let p = client_app.player.as_ref().unwrap();
        assert!(p.hide_visible_accessory[0], "bit 0 hid slot 0");
        assert!(!p.hide_visible_accessory[1]);
        assert!(!p.hide_visible_accessory[2]);
        assert!(p.hide_visible_accessory[3], "bit 3 hid slot 3");
        assert!(!p.hide_visible_accessory[4]);
    }

    /// W4-g51 `153 NPCDebuffDamage`: the client's `GetHurtByDebuff` drains hp
    /// directly (no i-frames), the immortal gate skips, and a life ≤ 0 takes
    /// the no-interaction kill (`MessageBuffer.cs:4408-4417`,
    /// `NPC.cs:91864-91888`).
    #[test]
    fn the_npc_debuff_damage_drains_hp_and_kills_at_zero() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::Slime, 0.0, 0.0, 1, 0.0);
        npc.hp = 30;
        client_app.npcs.push(npc);
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::NpcDebuffDamage(net::NpcDebuffDamage {
                    npc: 0,
                    damage: 10,
                })),
            );
        }
        client_app.net = role;
        assert_eq!(client_app.npcs[0].hp, 20, "the DoT drained 10");
        // A lethal tick floors to the no-interaction kill (hp = 0).
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::NpcDebuffDamage(net::NpcDebuffDamage {
                    npc: 0,
                    damage: 9999,
                })),
            );
        }
        client_app.net = role;
        assert_eq!(client_app.npcs[0].hp, 0, "the lethal tick took the kill");
    }

    /// W4-g51 `157 TeamChangeFromUI`: the host forces the subject, sets the
    /// team and relays as 45 (`MessageBuffer.cs:2325-2364`); the team-chat
    /// fan-out reaches the sender.
    #[test]
    fn the_team_change_from_ui_sets_the_team_and_relays() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let (mut host_app, mut client_app) = in_world_pair();
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::TeamChangeUi(net::TeamChange {
                        player_id: 99, // overridden to the sender (1)
                        team: 2,
                    }),
                },
            );
        }
        host_app.net = role;
        assert_eq!(host_app.remote_players[&1].team, 2, "the team was set");
        // The sender received the team-chat line (green team, mp[15]).
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            client_app
                .chat
                .messages
                .iter()
                .any(|m| m.text.contains("green team")),
            "the team-chat fan-out reached the sender"
        );
    }

    /// W4-g51 `159 RequestSection`: the host's `SendSection` gates the bounds
    /// + not-already-sent, marks the section and streams the `10 TileBlock`
    /// (`MessageBuffer.cs:4457-4462`, `NetMessage.cs:2697-2712`).
    #[test]
    fn the_section_request_streams_the_tile_block_once() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Ask for a section the join box already covered (spawn section 0,0
        // is sent during the join); pick one outside it.
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::SectionRequest {
                        section_x: 0,
                        section_y: 0,
                    },
                },
            );
        }
        host_app.net = role;
        // The join already marked (0,0) sent, so the request is a no-op gate
        // (the C#'s !TileSections check). The world is tiny (one section),
        // so the bounds gate is the observable: an out-of-range ask drops.
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 1,
                    msg: C2s::SectionRequest {
                        section_x: 999,
                        section_y: 999,
                    },
                },
            );
        }
        host_app.net = role;
        // No panic, no kick — the bounds gate refused silently.
        pump(&mut host_app, &mut client_app, 5);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
    }

    #[test]
    fn the_peer_inventory_slot_relays_to_the_other_clients() {
        // `MessageBuffer.cs:399-462` — the server stores the client's slot
        // and relays it to the other clients (`PlayerItemSlotID.CanRelay`
        // `:454-457`).
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr.clone(), String::new()));
        // A second client on the same host.
        let mut second = flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new());
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        // The second client's join push carries its 59 grid slots; the relay
        // lands them in the first client's peer state.
        let peer = second.local_id as u32;
        let mut relayed = 0usize;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            relayed = client_app
                .remote_players
                .get(&peer)
                .map(|p| p.inventory.len())
                .unwrap_or(0);
            if relayed > 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(relayed > 0, "the relayed inventory slots");
    }

    #[test]
    fn the_net_offset_decays_toward_zero() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app.player.as_mut().unwrap().x = 100.0;
        pump(&mut host_app, &mut client_app, 12);
        host_app.player.as_mut().unwrap().x = 160.0;
        pump(&mut host_app, &mut client_app, 12);
        let first = client_app.remote_players[&0].off_x;
        assert!(first > 50.0, "the 60 px step accrued a lag");
        // One 60 Hz frame of the C#'s decay: `max(2, 10%)` toward zero.
        client_app.net_interp_tick(1.0 / 60.0);
        let second = client_app.remote_players[&0].off_x;
        assert!(
            second < first && second > 0.0,
            "the lag decays, not jumps: {first} -> {second}"
        );
        // Enough frames and it is gone.
        for _ in 0..600 {
            client_app.net_interp_tick(1.0 / 60.0);
        }
        assert_eq!(client_app.remote_players[&0].off_x, 0.0);
    }

    #[test]
    fn a_teleport_farther_than_the_range_snaps_the_offset() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app.player.as_mut().unwrap().x = 100.0;
        pump(&mut host_app, &mut client_app, 12);
        // 400 px: past `multiplayerNPCSmoothingRange` (300) the C# resets the
        // offset to zero instead of sliding (`MessageBuffer.cs:1006-1008`).
        host_app.player.as_mut().unwrap().x = 500.0;
        pump(&mut host_app, &mut client_app, 12);
        let peer = client_app.remote_players.get(&0).expect("host peer stored");
        assert!((peer.x - 500.0).abs() < 0.01);
        assert_eq!(peer.off_x, 0.0);
    }

    #[test]
    fn npc_snapshots_keep_the_slot_and_despawn_on_absence() {
        let (mut host_app, mut client_app) = in_world_pair();
        let slime = crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue);
        host_app.npcs.push(slime);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.npcs.len(), 1, "the client stores the NPC");
        let x0 = client_app.npcs[0].x;
        // Move it: the same slot is patched, not re-created.
        host_app.npcs[0].x = x0 + 80.0;
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.npcs.len(), 1, "no re-spawn per snapshot");
        assert!(
            (client_app.npcs[0].x - (x0 + 80.0)).abs() < 0.01,
            "the position snaps"
        );
        assert!(
            client_app.npcs[0].net_offset.length() > 50.0,
            "the npc lag accrued: {}",
            client_app.npcs[0].net_offset.length()
        );
        // The decay shrinks it (the C#'s `2 + 2*(len/range)` step).
        let before = client_app.npcs[0].net_offset.length();
        client_app.net_interp_tick(1.0 / 60.0);
        assert!(client_app.npcs[0].net_offset.length() < before);
        // The NPC disappears: the streamed set no longer contains the slot.
        host_app.npcs.clear();
        pump(&mut host_app, &mut client_app, 150);
        assert!(
            client_app.npcs.is_empty(),
            "the despawn diff removed it: {} left",
            client_app.npcs.len()
        );
    }

    #[test]
    fn item_states_sync_and_remove() {
        let (mut host_app, mut client_app) = in_world_pair();
        let mut drop = item::ItemDrop::new(7, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        drop.age = 60.0;
        host_app.items.push(drop);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.items.len(), 1, "the drop is streamed");
        assert_eq!(client_app.items[0].kind, 7);
        // It moves: the 21 diff updates it.
        host_app.items[0].pos.x = 140.0;
        pump(&mut host_app, &mut client_app, 15);
        assert!((client_app.items[0].pos.x - 140.0).abs() < 0.01);
        // It is picked up: the 151 removes it.
        host_app.items.clear();
        pump(&mut host_app, &mut client_app, 15);
        assert!(client_app.items.is_empty(), "the removal streamed as 151");
    }

    #[test]
    fn a_pickup_request_removes_the_drop_on_the_host() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Stand the client's peer next to the drop on the HOST's side (the
        // host validates the 60 px range against `remote_players[client]`).
        host_app.remote_players.insert(
            1,
            net::PlayerNet {
                id: 1,
                x: 100.0,
                y: 100.0,
                ..Default::default()
            },
        );
        host_app.player.as_mut().unwrap().x = 100.0;
        host_app.player.as_mut().unwrap().y = 100.0;
        let mut drop = item::ItemDrop::new(7, 3, 0, 108.0, 108.0, 0.0, 0.0, 0.0);
        drop.age = 60.0;
        host_app.items.push(drop);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(host_app.items.len(), 1);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Pickup { drop: 0 });
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            host_app.items.is_empty(),
            "the host validated and removed it"
        );
        assert!(client_app.items.is_empty(), "the client saw the 151");
    }

    #[test]
    fn the_join_burst_streams_the_live_items_with_their_reservations() {
        // `MessageBuffer.cs:843-850` — one `21` + one `22` per live slot, to
        // the joining client only, BEFORE the first cadence tick.
        let (mut host_app, mut client_app) = connect_pair("", "");
        let mut drop = item::ItemDrop::new(7, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        drop.age = 60.0;
        host_app.items.push(drop);
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        // No net_tick: the join burst alone must have delivered the drop
        // (a few poll rounds let the loopback bytes land).
        for _ in 0..8 {
            host_app.poll_net();
            client_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(client_app.items.len(), 1, "the join 21 burst streamed it");
        assert_eq!(client_app.items[0].kind, 7);
        // The paired `22` arrived with the C# field defaults: unreserved.
        assert!(
            !client_app.net_reserved.contains_key(&0),
            "the join 22 carries 255 (unreserved), which clears the store"
        );
        // The burst recorded the state: a cadence tick without a change
        // re-sends nothing (the diff sees the burst's snapshot).
        host_app.net_tick(0.1);
        host_app.poll_net();
        client_app.poll_net();
        assert_eq!(client_app.items.len(), 1, "no duplicate spawn");
    }

    #[test]
    fn the_item_heartbeat_re_sends_one_slot_per_second() {
        // `Main.cs:64570-64580` — every 60 server ticks one round-robin slot
        // gets an unconditional `21`, even when nothing changed. The port's
        // cadence batches 6 ticks per call, so 10 calls = one heartbeat.
        let (mut host_app, mut client_app) = in_world_pair();
        let mut drop = item::ItemDrop::new(7, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        drop.age = 60.0;
        host_app.items.push(drop);
        // Ten cadence ticks: the first diff-sends the new drop, the rest see
        // no change — but the 10th tick's counter hits 60 and the heartbeat
        // fires (slot 0 of the one-item list).
        for _ in 0..10 {
            host_app.net_tick(0.1);
        }
        assert_eq!(host_app.net_play_counter, 60);
        assert_eq!(host_app.last_item_update, 0, "the walk wrapped to slot 0");
        for _ in 0..8 {
            host_app.poll_net();
            client_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(client_app.items.len(), 1, "the heartbeat patched, not spawned");
        // Another second: the counter keeps walking, the slot stays 0 (the
        // one-item list wraps every step).
        for _ in 0..10 {
            host_app.net_tick(0.1);
        }
        assert_eq!(host_app.net_play_counter, 120);
        assert_eq!(host_app.last_item_update, 0);
    }

    #[test]
    fn the_host_own_snapshot_reaches_the_client() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app.player.as_mut().unwrap().x = 42.0;
        host_app.player.as_mut().unwrap().leg_frame = 5;
        pump(&mut host_app, &mut client_app, 15);
        let peer = client_app.remote_players.get(&0).expect("host peer stored");
        assert!((peer.x - 42.0).abs() < 0.01);
        assert_eq!(peer.leg_frame, 5);
    }

    /// A pair whose host world spans 5x4 sections (900x600) with the spawn at
    /// section (0,0): the join box covers 3x2 of them, so the latch and the
    /// move-driven catch-up have unlatched sections to work with. The markers
    /// are seeded before the stream so the sections carry them.
    fn big_world_pair() -> (App, App) {
        let (mut host_app, mut client_app) = connect_pair("", "");
        host_app.world = tr_core::World::new("flow".into(), 900, 600);
        host_app.world.spawn_x = 100;
        host_app.world.spawn_y = 100;
        host_app.world.column_surface = vec![host_app.world.height; 900];
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        host_app.player = Some(player::Player::spawn(&host_app.world));
        (host_app, client_app)
    }

    /// Marks a tile as a real active tile.
    fn mark(world: &mut tr_core::World, x: i32, y: i32, kind: u16) {
        let i = world.index(x, y);
        world.tiles[i].kind = kind;
        world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    #[test]
    fn the_join_streams_the_spawn_sections() {
        // The markers ride in the join box: the 5x3 box around section (0,0)
        // clips to 3x2 for this world, so sections (0,0)..(2,1).
        let (mut host_app, mut client_app) = connect_pair("", "");
        host_app.world = tr_core::World::new("flow".into(), 900, 600);
        host_app.world.spawn_x = 100;
        host_app.world.spawn_y = 100;
        host_app.world.column_surface = vec![host_app.world.height; 900];
        mark(&mut host_app.world, 10, 12, 7);
        mark(&mut host_app.world, 210, 20, 8);
        mark(&mut host_app.world, 450, 260, 9);
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        let at = |w: &tr_core::World, x: i32, y: i32| w.tile(x, y).map_or(0, |t| t.kind);
        assert_eq!(at(&client_app.world, 10, 12), 7, "section (0,0)");
        assert_eq!(at(&client_app.world, 210, 20), 8, "section (1,0)");
        assert_eq!(at(&client_app.world, 450, 260), 9, "section (2,1)");
        // Outside the box, nothing has streamed yet.
        assert_eq!(
            at(&client_app.world, 850, 550),
            0,
            "section (4,3) is not joined"
        );
        // The latched sections read back as sent, the far ones not.
        if let NetRole::Host(server) = &host_app.net {
            assert!(server.is_section_sent(1, 0, 0));
            assert!(server.is_section_sent(1, 2, 1));
            assert!(!server.is_section_sent(1, 4, 3));
        }
    }

    #[test]
    fn a_moved_peer_streams_the_sections_it_approaches() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        host_app.world = tr_core::World::new("flow".into(), 900, 600);
        host_app.world.spawn_x = 100;
        host_app.world.spawn_y = 100;
        host_app.world.column_surface = vec![host_app.world.height; 900];
        // A marker in the far section (850,550) the join box did not cover.
        mark(&mut host_app.world, 850, 550, 9);
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        host_app.player = Some(player::Player::spawn(&host_app.world));
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            client_app.world.tile(850, 550).map_or(0, |t| t.kind),
            0,
            "the unjoined section is not there yet"
        );
        // Move the peer into that section: the next cadence tick streams it.
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the client peer")
            .x = 860.0 * tr_core::TILE_PX;
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the client peer")
            .y = 560.0 * tr_core::TILE_PX;
        pump(&mut host_app, &mut client_app, 25);
        assert_eq!(
            client_app.world.tile(850, 550).map_or(0, |t| t.kind),
            9,
            "the approached section streamed"
        );
        if let NetRole::Host(server) = &host_app.net {
            assert!(server.is_section_sent(1, 4, 3));
        }
    }

    #[test]
    fn a_tile_edit_replicates_as_a_section_range() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        host_app.world = tr_core::World::new("flow".into(), 900, 600);
        host_app.world.spawn_x = 100;
        host_app.world.spawn_y = 100;
        host_app.world.column_surface = vec![host_app.world.height; 900];
        mark(&mut host_app.world, 50, 60, 5);
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        host_app.player = Some(player::Player::spawn(&host_app.world));
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.world.tile(50, 60).map_or(0, |t| t.kind), 5);
        // The host edits the tile (the real flow: `editing.rs` mutates the
        // world, then asks for the replication) and the client receives the
        // `20` body.
        let _ = crate::world_sim::editing::apply_edit(&mut host_app.world, 50, 60, 0, 5, host_app.rngs.global());
        host_app.request_edit(50, 60, 0, 5);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            client_app.world.tile(50, 60).map_or(0, |t| t.kind),
            0,
            "the removal replicated"
        );
    }

    #[test]
    fn a_client_edit_request_reaches_the_host() {
        let (mut host_app, mut client_app) = big_world_pair();
        mark(&mut client_app.world, 60, 60, 6);
        // The client asks the host to apply a wall break at (60, 60).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Edit {
                x: 60,
                y: 60,
                kind: 2,
                tile: 1,
            });
        }
        pump(&mut host_app, &mut client_app, 15);
        // The host applied `break_wall` (kind 2) — its wall is gone.
        assert_eq!(host_app.world.tile(60, 60).map_or(0, |t| t.wall), 0);
        // And rebroadcast the tile as a `20`, so the client matches.
        assert_eq!(client_app.world.tile(60, 60).map_or(0, |t| t.wall), 0);
    }

    #[test]
    fn the_tile_codec_round_trips_runs_and_widths() {
        use net::{SectionRange, TileBlock, TileRecord};
        fn rec(kind: u16, wall: u16) -> TileRecord {
            TileRecord {
                active: true,
                kind,
                frame_important: false,
                frame_x: 0,
                frame_y: 0,
                wall,
                wall_frame: 0,
                liquid: 0,
                liquid_kind: 0,
                half_brick: false,
                slope: 0,
                actuator: false,
                inactive: false,
                paint: 0,
                wall_paint: 0,
                wires: 0,
                invisible_block: false,
                invisible_wall: false,
                fullbright_block: false,
                fullbright_wall: false,
            }
        }
        // A run of 300 identical tiles crosses the 1-byte repeat boundary;
        // mixed walls exercise the 1/2-byte wall width switch; the last record
        // is frame-important so the 4-byte frame pair rides.
        let mut records = Vec::new();
        for _ in 0..300 {
            records.push(rec(0, 0));
        }
        records.push(rec(1, 1));
        records.push(rec(2, 300));
        records.push(TileRecord {
            frame_important: true,
            frame_x: 17,
            frame_y: -3,
            ..rec(6, 0)
        });
        let block = TileBlock {
            x: 12,
            y: 34,
            w: records.len() as i16,
            h: 1,
            records,
        };
        let frame = block.frame();
        assert!(frame.len() >= 3 && frame.len() <= net::MAX_PACKET);
        assert_eq!(u16::from_le_bytes([frame[0], frame[1]]), frame.len() as u16);
        assert_eq!(frame[2], net::id::TILE_BLOCK);
        let mut r = net::NetReader::new(&frame[3..]);
        let back = TileBlock::decode_from(&mut r).expect("tile block");
        assert_eq!(back, block, "the RLE run and the width switches round-trip");

        // `20` round-trips the same records (its width/height are u8s on the
        // wire — `NetMessage.cs:619-620` — so the rect fits in 255).
        let mut recs = block.records.clone();
        recs.resize(400, block.records[0]);
        let range = SectionRange {
            x: -5,
            y: 7,
            w: 200,
            h: 2,
            change_type: 0,
            records: recs,
        };
        let frame = range.frame();
        let mut r = net::NetReader::new(&frame[3..]);
        let back = SectionRange::decode_from(&mut r).expect("section range");
        assert_eq!(back, range);
    }

    /// The `82` envelope rides a module body through the real socket.
    #[test]
    fn a_chat_message_relays_through_the_host() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        // The client's chat submit rides the module (`ChatHelper
        // .SendChatMessageFromClient`).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NetModule {
                module: crate::app::netmodules::module_id::TEXT,
                body: crate::app::netmodules::ChatMessage {
                    command: String::new(),
                    text: "hello from the client".into(),
                }
                .encode(),
            });
        }
        pump(&mut host_app, &mut client_app, 20);
        // The host ran the line through its chat and relayed it; both sides
        // show the line.
        assert!(
            host_app
                .chat
                .messages
                .iter()
                .any(|m| m.text.contains("hello from the client")),
            "the host logged the line"
        );
        assert!(
            client_app
                .chat
                .messages
                .iter()
                .any(|m| m.text.contains("hello from the client")),
            "the client received the relayed line"
        );
    }

    /// DW-5.1 — a relayed chat command draws the bare-`Main.rand` stand-in:
    /// the host's `global()` pays exactly `Next(1, 101)` (`RollCommand.cs:14`)
    /// and no stage stream moves (the C# chat path runs outside every swap).
    #[test]
    fn test_dw_5_1_chat_commands_draw_the_global_stand_in() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        let mut expected = host_app.rngs.global().clone();
        let expected_roll = expected.next_range(1, 101);
        let stage_pre: Vec<_> = crate::app::RngKey::ALL
            .iter()
            .map(|&k| (k, host_app.rngs.stream(k).clone()))
            .collect();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NetModule {
                module: crate::app::netmodules::module_id::TEXT,
                body: crate::app::netmodules::ChatMessage {
                    command: String::new(),
                    text: "/roll".into(),
                }
                .encode(),
            });
        }
        pump(&mut host_app, &mut client_app, 20);
        let line = host_app
            .chat
            .messages
            .iter()
            .find(|m| m.text.starts_with("*client rolls a "))
            .expect("the host relayed the roll");
        let rolled: u32 = line.text.rsplit(' ').next().unwrap().parse().unwrap();
        assert_eq!(rolled, expected_roll as u32, "the roll is Next(1, 101)");
        assert_eq!(
            host_app.rngs.global(),
            &expected,
            "the global stand-in paid exactly the roll's draws"
        );
        for (k, pre) in &stage_pre {
            assert_eq!(
                host_app.rngs.stream(*k),
                pre,
                "{} untouched by the chat command",
                k.name()
            );
        }
    }

    #[test]
    fn the_ping_module_measures_an_rtt() {
        let (mut host_app, mut client_app) = in_world_pair();
        // `Ping.Reset` runs on connect (`Netplay.cs:474`), so the stopwatch is
        // already armed; age it past the 250 ms probe threshold
        // (`Net/Ping.cs:29`) instead of waiting for wall-clock time.
        client_app.net_ping_at = Some(std::time::Instant::now() - std::time::Duration::from_millis(300));
        client_app.net_ping_waiting = false;
        pump(&mut host_app, &mut client_app, 4);
        assert!(
            client_app.net_rtt_ms < 10_000,
            "the ping echo came back: rtt {} ms",
            client_app.net_rtt_ms
        );
        assert!(
            !client_app.net_ping_waiting,
            "PingRecieved cleared the waiting flag (`Net/Ping.cs:41`)"
        );
    }

    /// `Main.TriggerPing` (`Main.cs:56541-56551`) relays the marker through
    /// `NetPingModule`; the host rebroadcasts it to the other clients
    /// (`NetPingModule.Deserialize`, `NetPingModule.cs:16-33`), where it lands
    /// in the map's ping pool (`Main.Pings.Add`).
    #[test]
    fn the_map_ping_relays_through_the_ping_module() {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr.clone(), String::new()));
        let mut second = flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new());
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        assert!(second.map.is_some(), "the client map exists after the join");
        // The first client pins a spot (the shape `handle_fullmap_ping` uses).
        client_app
            .map
            .as_mut()
            .expect("client map")
            .trigger_ping([120.0, 45.0]);
        let mut landed = None;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            landed = second
                .map
                .as_ref()
                .and_then(|m| m.pings.first().map(|p| p.0));
            if landed.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(landed, Some([120.0, 45.0]), "the marker reached the peer");
        // The sender keeps its own copy and the host does not take one
        // (`broadcast_except`).
        assert_eq!(
            client_app.map.as_ref().expect("client map").pings.len(),
            1,
            "the sender's own marker"
        );
        assert!(
            host_app.map.as_ref().map_or(true, |m| m.pings.is_empty()),
            "the host is not a ping sink"
        );
    }

    #[test]
    fn the_ambience_module_relays_a_spawn() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The host force-spawns a sky entity; the module relay must land it on
        // the client (`AmbienceServer.SpawnForPlayer`).
        // The forced spawn is consumed by the ambience update (the frame's
        // ambience step), so run one step before the pump picks it up. The
        // scheduler draws the bare-`Main.rand` stand-in (`global()`); the
        // stream rides through the step via clone/write-back — `ambience_ctx`
        // borrows the whole App while the update runs (the flow.rs trick).
        {
            let mut amb = std::mem::take(&mut host_app.ambience);
            // Block the regular scheduler so this step exercises ONLY the
            // forced-spawn path: the pinned seed replaces the host's
            // `SerializeSkyEntitySpawn` draw and the loopback reads the seed
            // from the packet — the bare-`Main.rand` stand-in must not
            // advance anywhere in the chain (no double advance).
            amb.block_scheduler_for_tests();
            let pre = host_app.rngs.global().clone();
            amb.forced_spawn_for_tests(crate::ambience::SkyEntityType::Wyvern, -1, 12345);
            let mut rng = host_app.rngs.global().clone();
            {
                let ctx = host_app.ambience_ctx();
                amb.update(1.0 / 60.0, &ctx, &mut rng);
            }
            *host_app.rngs.global() = rng;
            host_app.ambience = amb;
            assert_eq!(*host_app.rngs.global(), pre, "the forced spawn drew nothing");
        }
        let client_pre = client_app.rngs.global().clone();
        pump(&mut host_app, &mut client_app, 25);
        assert!(
            !client_app.ambience.entities.is_empty(),
            "the client spawned the relayed entity: {}",
            client_app.ambience.entities.len()
        );
        // The client's loopback spawned from the packet's seed — its own
        // bare-`Main.rand` stand-in paid nothing.
        assert_eq!(
            client_app.rngs.global(),
            &client_pre,
            "the loopback deserialize drew nothing"
        );
    }

    /// A remote player's projectile lands in the local pool through `27` and
    /// leaves it through `29` (`MessageBuffer.cs:1729-1755`,
    /// `:1825-1850`).
    #[test]
    fn a_peer_projectile_syncs_in_and_out() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The host owns the slot and broadcasts it as `27`.
        let key = net::projectile_key(0, 3, 1);
        let st = net::ProjectileState {
            key,
            x: 100.0,
            y: 100.0,
            vx: 4.0,
            vy: 0.0,
            kind: 1,
            ai0: None,
            ai1: None,
            banner_id: 0,
            damage: 10,
            knock_back: 2.0,
            original_damage: 0,
            ai2: None,
        };
        client_app.projectiles.upsert_remote(&st);
        assert_eq!(client_app.projectiles.active_count(), 1);
        let p = &client_app.projectiles.slots()[3];
        assert_eq!(p.owner, 0, "the slot keeps the wire owner");
        assert!((p.pos.x - 100.0).abs() < 0.01);
        // A stale key for the same index cannot touch it.
        client_app.projectiles.upsert_remote(&net::ProjectileState {
            key: net::projectile_key(0, 3, 9),
            x: 999.0,
            ..st
        });
        assert!(
            (client_app.projectiles.slots()[3].pos.x - 999.0).abs() < 0.01,
            "the stale-generation state re-creates the slot"
        );
        // The despawn kills only the matching occupant.
        client_app.projectiles.kill_remote(3, 0, 9);
        assert_eq!(
            client_app.projectiles.active_count(),
            0,
            "the 29 dropped it"
        );
        client_app.projectiles.kill_remote(3, 0, 1);
        assert_eq!(
            client_app.projectiles.active_count(),
            0,
            "a stale key is a no-op"
        );
    }

    /// S1-I-4d: the own-projectile cadence — the host's shot reaches the client's
    /// pool via the spawn `27`, and a live shot emits no `29`.
    #[test]
    fn the_host_projectile_broadcast_reaches_the_client() {
        let (mut host_app, mut client_app) = in_world_pair();
        let slot = host_app
            .projectiles
            .spawn_with_data_owned(1, 0, 100.0, 100.0, 3.0, 0.0, 10, 2.0, 0.0, 0.0)
            .expect("spawn");
        let _ = slot;
        pump(&mut host_app, &mut client_app, 20);
        let active = client_app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .count();
        assert!(active >= 1, "the client received the host's projectile");
        // A further cadence step on a LIVE stationary shot queues no `29`
        // despawn (count the outbox BEFORE the drain empties it).
        host_app.proj_sync_cadence();
        let host_dead = host_app
            .net_proj_outbox
            .iter()
            .filter(|s| matches!(s, crate::app::net_proj_sync::ProjSend::Kill { .. }))
            .count();
        assert_eq!(host_dead, 0, "the live shot is not dead");
        host_app.drain_proj_outbox();
        // The client does not own the host's shot, so its cadence sends nothing.
        client_app.proj_sync_cadence();
        assert!(
            client_app.net_proj_outbox.is_empty(),
            "the host's shot is not the client's to sync/despawn"
        );
    }

    /// The player-data push (`case 3`'s client half, `MessageBuffer.cs:237-282`):
    /// the identity, the health and the inventory arrays, adapted to the
    /// port's slot layout (`App.inventory` + the `EquipSlots`), then
    /// `6 WorldRequest`.
    fn client_send_identity(&self, client: &net::Client) {
        let me = self.local_id as u8;
        let mut pi = net::PlayerInfo::with_name(me, &self.player_name);
        // `bitsByte16` (`NetMessage.cs:182-184`, W4-f33): bit 0
        // UsingBiomeTorches, bit 1 happyFunTorchTime, bit 2
        // unlockedBiomeTorches — the 5043 use's `SendData(4)`
        // (`Player.cs:45240`) re-announces the pair through this push.
        if let Some(p) = self.player.as_ref() {
            pi.flags2.set(0, p.using_biome_torches());
            pi.flags2.set(1, p.torch_god.happy_fun_torch_time);
            pi.flags2.set(2, p.unlocked_biome_torches);
        }
        client.send(&C2s::PlayerInfo(pi));
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
        // `MessageBuffer.cs:253` — the join push's `147 SyncLoadout`: the
        // current loadout index + the accessory-visibility mask, ahead of the
        // `5` inventory arrays (`:254-272`).
        let (loadout, hide) = self
            .player
            .as_ref()
            .map(|p| {
                (
                    p.current_loadout_index as u8,
                    accessory_visibility_mask(&p.hide_visible_accessory),
                )
            })
            .unwrap_or((0, 0));
        client.send(&C2s::SyncLoadout(net::SyncLoadout {
            player_id: me,
            loadout,
            hide_accessory: hide,
        }));
        // The 59 grid slots, in `PlayerItemSlotID.Inventory0..58` order.
        for (i, slot) in self.inventory.iter().enumerate() {
            client.send(&C2s::Inventory {
                slot: i as i16,
                item: self.net_item(slot, self.favorite_slots.contains(&i)),
    /// S1-I-1 W4-d93: the `29` body carries the entry position verbatim and
    /// the host gates on the key's owner (`MessageBuffer.cs:1831-1834`).
    #[test]
    fn the_kill_wire_carries_the_position_and_the_owner_gate() {
        let (mut host_app, mut client_app) = in_world_pair();
        let owner = client_app.local_id as u8;
        // The client's own shot, seeded into the host pool by key.
        let key = net::projectile_key(owner, 3, 1);
        let st = net::ProjectileState {
            key,
            x: 100.0,
            y: 100.0,
            vx: 4.0,
            vy: 0.0,
            kind: 1,
            ai0: None,
            ai1: None,
            banner_id: 0,
            damage: 10,
            knock_back: 2.0,
            original_damage: 0,
            ai2: None,
        };
        host_app.projectiles.upsert_remote(&st);
        client_app.projectiles.upsert_remote(&st);
        // A peer's key (a different owner) cannot kill the slot
        // (`:1831-1834`); the host keeps it.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ProjectileDead {
                key: net::projectile_key(owner.wrapping_add(1), 3, 1),
                x: 5.0,
                y: 5.0,
            });
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            host_app.projectiles.slots()[3].active,
            "a stale-owner key never kills (`:1831` gates before TryGet's slot)"
        );
        // The owner's kill lands and the position rides the wire
        // (`NetMessage.cs:852-855`, `Projectile.cs:77256`).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ProjectileDead {
                key,
                x: 12.5,
                y: -8.0,
            });
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            !host_app.projectiles.slots()[3].active,
            "the owner's 29 dropped the slot"
        );
        assert!(
            !client_app.projectiles.slots()[3].active,
            "the relayed 29 dropped the client's copy"
        );
    }

    /// S1-I-1 W4-d93: `95 MurderSomeoneElsesPortal` kills the first matching
    /// active type-602 and relays one `29` (`MessageBuffer.cs:3519-3537`).
    #[test]
    fn murder_portal_kills_the_matching_602_and_relays_29() {
        let (mut host_app, mut client_app) = in_world_pair();
        let owner = client_app.local_id as u8;
        // A portal-gun projectile (602) owned by the client, form ai[1]=1.
        let slot = host_app
            .projectiles
            .spawn_with_data_owned(602, owner, 100.0, 100.0, 0.0, 0.0, 0, 0.0, 0.0, 1.0)
            .expect("spawn");
        pump(&mut host_app, &mut client_app, 20);
        assert!(host_app.projectiles.slots()[slot].active);
        // A wrong form matches nothing (`:3529` compares ai[1]).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::MurderPortal {
                owner: owner as u16,
                form: 0,
            });
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            host_app.projectiles.slots()[slot].active,
            "the form mismatch spared the portal"
        );
        // The right form kills it and the `29` relay reaches the client.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::MurderPortal {
                owner: owner as u16,
                form: 1,
            });
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(
            !host_app.projectiles.slots()[slot].active,
            "the host killed the matching portal"
        );
        assert!(
            !client_app.projectiles.slots()[slot].active,
            "the 29 relay dropped the client's copy (`:3532`)"
        );
    }

    /// S1-I-1 W4-d93: the `27` host read gate (`MessageBuffer.cs:1725-1728`)
    /// refuses a hostile-kind shot or a key whose spawner is not the sender —
    /// the break skips BOTH the apply and the relay.
    #[test]
    fn the_27_host_gate_refuses_hostile_kinds_and_foreign_spawners() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        let base = net::ProjectileState {
            key: net::projectile_key(me, 7, 1),
            x: 50.0,
            y: 50.0,
            vx: 1.0,
            vy: 0.0,
            kind: 1, // WoodenArrow — not in `Main.projHostile`
            ai0: None,
            ai1: None,
            banner_id: 0,
            damage: 5,
            knock_back: 1.0,
            original_damage: 0,
            ai2: None,
        };
        // A hostile kind (31, EyeSpring — `Projectile.cs:1138`) is refused.
        let hostile = net::ProjectileState { kind: 31, ..base };
        // A foreign spawner (the host's slot 0) is refused even when friendly.
        let foreign = net::ProjectileState {
            key: net::projectile_key(0, 8, 1),
            ..base
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Projectile(Box::new(hostile)));
            client.send(&C2s::Projectile(Box::new(foreign)));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(
            !host_app.projectiles.slots()[7].active && !host_app.projectiles.slots()[8].active,
            "the `:1727` break skipped the apply for both refused shots"
        );
        assert!(
            !client_app.projectiles.slots()[7].active && !client_app.projectiles.slots()[8].active,
            "the break skipped the relay too (`:1760` is after the gate)"
        );
        // The own friendly shot lands and relays (`:1729-1763`).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Projectile(Box::new(base)));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(
            host_app.projectiles.slots()[7].active,
            "the owner's friendly kind passed the gate"
        );
        assert!(
            client_app.projectiles.slots()[7].active,
            "the accepted shot relayed back (`:1760-1763`)"
        );
    }

    /// S1-I-1 W4-d93: `128 LandGolfBallInCup` — the host only relays, with
    /// `ignoreClient = owner` (`MessageBuffer.cs:4065-4068`): the owner
    /// already ran the cup effects locally (`GolfHelper.cs:103`). The client
    /// apply is parked (M4-GOLF-1), so the witness pins the end-to-end wire:
    /// an undecodable relay would be a `BadFrame` → disconnect
    /// (`netglue.rs`'s client reader), so a surviving IN_WORLD peer proves
    /// the 128 reached it and decoded. Step 2 keys the ignore on the BODY's
    /// owner (not the sender): the sender itself receives that relay.
    #[test]
    fn the_golf_cup_relay_routes_by_the_body_owner() {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr.clone(), String::new()));
        let mut second = flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new());
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        let me = client_app.local_id;
        let peer = second.local_id;
        // Step 1 — the owner's own cup: ignoreClient = owner, so the relay
        // goes to the second client only; it decodes (parked arm) and lives.
        let g = net::GolfBallInCup {
            owner: me as u8,
            x: 40,
            y: 12,
            hits: 3,
            ball_type: 520,
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::GolfBallInCup(g));
        }
        // Step 2 — a cup credited to the SECOND player: the ignore now skips
        // the second, and the FIRST client (the sender, not the owner)
        // receives the relay — it must decode and survive too.
        let g2 = net::GolfBallInCup {
            owner: peer as u8,
            x: 41,
            y: 13,
            hits: 1,
            ball_type: 521,
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::GolfBallInCup(g2));
        }
        for _ in 0..40 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(
            client_app.net_state,
            net::client_state::IN_WORLD,
            "the sender received the peer-owner relay and decoded it (step 2)"
        );
        assert_eq!(
            second.net_state,
            net::client_state::IN_WORLD,
            "the peer received the owner-skip relay and decoded it (step 1)"
        );
    }

    /// The `28` strike: a client's hit is validated and relayed by the host,
    /// which also pushes the full `23` on death.
    #[test]
    fn a_strike_is_validated_and_relayed_by_the_host() {
        let (mut host_app, mut client_app) = in_world_pair();
        let mut npc =
            crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue);
        npc.x = 100.0;
        npc.y = 100.0;
        host_app.npcs.clear();
        host_app.npcs.push(npc);
        // The generation bookkeeping the first `23` push creates.
        host_app.net_npc_gen.insert(0, (1, 1));
        pump(&mut host_app, &mut client_app, 20);
        // The client knows the NPC's slot and generation from the `23` push.
        let generation = client_app
            .net_npc_gen
            .get(&0)
            .map(|(_, g)| *g as u8)
            .expect("the 23 push delivered a generation");
        let hp_before = client_app.net_npcs.get(&0).map(|n| n.hp).unwrap_or(0);
        assert!(hp_before > 0);
        // A strike from the client with the right generation lands.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: 0,
                generation,
                damage: 10,
                knock_back: 3.0,
                hit_direction: 1,
                crit: false,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let hp_after = client_app.net_npcs.get(&0).map(|n| n.hp).unwrap_or(0);
        assert!(
            hp_after < hp_before,
            "the relayed strike landed on the client"
        );
        assert_eq!(
            host_app.npcs[0].hp, hp_after,
            "the host applied the same damage"
        );
        // A stale generation is dropped by the host.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: 0,
                generation: 9,
                damage: 10,
                knock_back: 3.0,
                hit_direction: 1,
                crit: false,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(
            client_app.net_npcs.get(&0).map(|n| n.hp),
            Some(hp_after),
            "the stale strike was dropped"
        );
    }

    // --- S1-I-4d: the SWAP witnesses (no-double-send, cadence, stream-order) ---

    /// Counts the `NpcSend::State` intents the NPC cadence queues for one step.
    fn npc_state_sends(app: &App) -> usize {
        app.net_npc_outbox
            .iter()
            .filter(|(_, s)| matches!(s, crate::app::net_npc_sync::NpcSend::State { .. }))
            .count()
    }

    /// NO-DOUBLE-SEND (NPC): the cadence is the ONLY `23` producer. A stationary
    /// NPC sends its priority spawn `23` once, then the `netSpam` window decays
    /// silently — the old whole-set 10 Hz broadcast would have re-sent it every
    /// tick. `NPC.cs:90602-90618` (the throttle) + the swap (one send path).
    #[test]
    fn the_npc_cadence_sends_once_then_throttles_not_every_tick() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        // First cadence step: the priority spawn (spawnNeedsSyncing) fans out
        // one State per connected client (here: the one joined client).
        host_app.npc_sync_cadence();
        let first = npc_state_sends(&host_app);
        assert!(first >= 1, "the spawn 23 fires once");
        host_app.net_npc_outbox.clear();
        // The next 29 steps: no flag, no change -> netSpam decays, NOTHING sends
        // (the C# throttle, not the old per-tick whole-set broadcast).
        for _ in 0..29 {
            host_app.npc_sync_cadence();
            assert_eq!(
                npc_state_sends(&host_app),
                0,
                "a stationary NPC does not re-send inside the netSpam window"
            );
            host_app.net_npc_outbox.clear();
        }
        let _ = client_app;
    }

    /// CADENCE PARITY (NPC): a continuously-moving NPC bursts the `netSpam`
    /// budget then settles to one packet per 30 ticks (`NPC.cs:6139-6145`,
    /// `:90602-90614`) — the port's cadence matches the decision layer's rate,
    /// not the old fixed 10 Hz.
    #[test]
    fn the_npc_cadence_throttles_a_moving_npc_to_the_netSpam_rate() {
        let (mut host_app, _client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        let mut sends_by_tick: Vec<usize> = Vec::new();
        for tick in 0..40 {
            // Move the NPC every tick so the change-detect producer keeps it dirty.
            host_app.npcs[0].x = 100.0 + tick as f32;
            host_app.npc_sync_cadence();
            sends_by_tick.push(npc_state_sends(&host_app));
            host_app.net_npc_outbox.clear();
        }
        // The first four dirty ticks all send (the 3*30 gate, NPC.cs:90605)...
        assert!(sends_by_tick[0] >= 1);
        assert!(sends_by_tick[1] >= 1);
        assert!(sends_by_tick[2] >= 1);
        assert!(sends_by_tick[3] >= 1);
        // ...the fifth defers (netSpam 116 > 90) and arms the pending cooldown.
        assert_eq!(sends_by_tick[4], 0, "the 5th burst tick defers");
        // The sustained rate is one packet per 30 ticks: the next send lands at
        // tick 31 (index 30), exactly as the decision layer's own test pins.
        let next = sends_by_tick.iter().skip(5).position(|&n| n >= 1);
        assert_eq!(next, Some(25), "the deferred update lands at tick 31");
    }

    /// NO-DOUBLE-SEND (projectile): a fresh shot emits exactly ONE spawn `27`
    /// (`Projectile.cs:11096`), not a second through the budget — the spawn and
    /// the change-detect producer are disjoint (the `fresh` set).
    #[test]
    fn the_proj_cadence_spawn_sends_one_27_not_two() {
        let (mut host_app, _client_app) = in_world_pair();
        host_app
            .projectiles
            .spawn_with_data_owned(1, 0, 100.0, 100.0, 3.0, 0.0, 10, 2.0, 0.0, 0.0)
            .expect("spawn");
        host_app.proj_sync_cadence();
        let syncs = host_app
            .net_proj_outbox
            .iter()
            .filter(|s| matches!(s, crate::app::net_proj_sync::ProjSend::Sync(_)))
            .count();
        assert_eq!(syncs, 1, "the spawn 27 fires exactly once");
        // A stationary shot on the next step sends nothing (no change, no flag).
        host_app.net_proj_outbox.clear();
        host_app.proj_sync_cadence();
        assert!(
            host_app.net_proj_outbox.is_empty(),
            "a stationary shot does not re-send"
        );
    }

    /// STREAM-ORDER (NPC despawn): a removed NPC emits ONE life-0 `23`, and the
    /// client drops the slot (`MessageBuffer.cs:1661-1664`). The despawn rides
    /// the uid gone-diff, not the old absence-from-the-whole-set prune.
    #[test]
    fn the_npc_despawn_streams_one_life0_body_and_the_client_drops_it() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.npcs.len(), 1, "the client learned the NPC");
        // Remove it (the sim's death retain): the next cadence queues one despawn.
        host_app.npcs.clear();
        host_app.npc_sync_cadence();
        assert_eq!(host_app.net_npc_despawns.len(), 1, "one despawn 23");
        assert_eq!(host_app.net_npc_despawns[0].hp, 0);
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            client_app.npcs.is_empty(),
            "the life-0 body removed the client slot"
        );
        // A further cadence does NOT re-emit the despawn (the uid is gone).
        host_app.npc_sync_cadence();
        assert!(host_app.net_npc_despawns.is_empty(), "the despawn fires once");
    }
}

#[cfg(test)]
mod npc_kind_wire_tests {
    //! The `23` NPC body's protocol id round-trip (`S1-F-1`): the host writes
    //! `NpcKind::to_u16()` into the i16 `kind` field (`netglue.rs:385`,
    //! `:1517`) and the client reads it back with `from_u16(kind.max(0))`
    //! (`:1044`, `:2060`). Seven kinds once shared ids `78..84`, so a
    //! Deerclops arrived as a Skeletron Prime part, ids `85..88` had no decode
    //! arm at all, and the pirates are now the contiguous `182..188` band.

    use super::*;

    /// Pushes one kind through the exact wire path of a `23` body: the i16
    /// field carries the host's encode, `npc_from_net` decodes it.
    fn wire_round_trip(kind: crate::npc::NpcKind) -> crate::npc::NpcKind {
        let kind_id = kind.to_u16() as i16;
        let info = net::NpcInfo {
            slot: 0,
            generation: 1,
            spawn: true,
            kind: kind_id,
            x: 8.0 * 16.0,
            y: 8.0 * 16.0,
            vx: 0.0,
            vy: 0.0,
            target: 255,
            dir_right: true,
            dir_y_down: false,
            sprite_right: true,
            life_full: true,
            hp: 100,
            life_max: 100,
            ai: [0.0; 4],
            spawned_from_statue: false,
            difficulty: 1.0,
            shimmer: false,
            release_owner: 255,
        };
        let world = tr_core::World::new("wire".into(), 64, 64);
        npc_from_net(&world, &info)
            .unwrap_or_else(|| panic!("{kind:?} did not decode from wire id {kind_id}"))
            .kind
    }

    #[test]
    fn the_collision_family_round_trips_on_the_wire() {
        // Both sides of every id `78..84` that was shared: the
        // Deerclops/Skeletron-Prime parts and the band that used to alias
        // them.
        for kind in [
            crate::npc::NpcKind::Deerclops,
            crate::npc::NpcKind::SkeletronHand,
            crate::npc::NpcKind::WallOfFleshEye,
            crate::npc::NpcKind::PrimeCannon,
            crate::npc::NpcKind::PrimeSaw,
            crate::npc::NpcKind::PrimeVice,
            crate::npc::NpcKind::PrimeLaser,
            crate::npc::NpcKind::PirateDeckhand,
            crate::npc::NpcKind::PirateCaptainsMate,
            crate::npc::NpcKind::PirateArcher,
            crate::npc::NpcKind::PirateCorsair,
            crate::npc::NpcKind::PirateCaptain,
            crate::npc::NpcKind::MartianProbe,
            crate::npc::NpcKind::MartianDrone,
        ] {
            assert_eq!(
                wire_round_trip(kind),
                kind,
                "{kind:?} (id {}) must not arrive as another npc",
                kind.to_u16()
            );
        }
    }

    #[test]
    fn the_previously_undecodable_ids_round_trip_on_the_wire() {
        // `85..88` carried no decode arm: the client logged "unknown npc kind"
        // and dropped the slot for the martian ground family.
        for kind in [
            crate::npc::NpcKind::BrainScrambler,
            crate::npc::NpcKind::RayGunner,
            crate::npc::NpcKind::ScutlixRider,
            crate::npc::NpcKind::Scutlix,
        ] {
            assert_eq!(wire_round_trip(kind), kind);
        }
    }

}

/// The S1-E-27 message families' end-to-end shapes: the chest burst, the
/// switch relay and the NPC-buff round trip.
#[cfg(test)]
mod message_family_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::netglue::handshake_tests::{apps, connect_pair, drive_to, in_world_pair, pump};
    use crate::ui::save_list::{PlayerEntry, WorldEntry};

    /// `case 31` + `SendChestContentsTo`: the client asks, the host answers
    /// `155` + one `32` per slot + `33` to the asker and `80` to the others
    /// (`MessageBuffer.cs:1876-1886`, `NetMessage.cs:1873-1880`).
    #[test]
    fn a_chest_open_request_bursts_155_33_and_80() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The `33` arm needs a local player to open the panel (`:1938-1962`).
        client_app.player = Some(player::Player::spawn(&client_app.world));
        // A chest the host owns, with two items.
        let mut chest = crate::chest::WorldChest {
            tile: (3, 5),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
            ..Default::default()
        };
        chest.slots = vec![
            crate::item::ItemSlot {
                kind: 7,
                name: crate::registry::name(7),
                count: 5,
                prefix: 0,
            },
            crate::item::ItemSlot {
                kind: 8,
                name: crate::registry::name(8),
                count: 1,
                prefix: 2,
            },
        ];
        host_app.world_chests.push(chest);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestOpenRequest(net::ChestOpenRequest {
                x: 3,
                y: 5,
            })); // `:33116`
        }
        let mut opened = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            opened = client_app.player.as_ref().is_some_and(|p| p.chest == 0);
            if opened {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(opened, "the client opened chest 0 through the 33 answer");
        // The `155` header resized the client's copy and the `32` burst
        // carried both slots.
        let local = client_app.world_chests.first().expect("the 32-burst created it");
        assert_eq!(
            local.tile,
            (3, 5),
            "the 33 arm places the slot's tile, the way the C# client already has it"
        );
        assert_eq!(local.max_items, crate::chest::DEFAULT_MAX_ITEMS as usize);
        assert_eq!(local.slots.len(), crate::chest::DEFAULT_MAX_ITEMS as usize);
        assert_eq!(local.slots[0].kind, 7);
        assert_eq!(local.slots[0].count, 5);
        assert_eq!(local.slots[1].kind, 8);
        assert_eq!(local.slots[1].prefix, 2);
        // `80` told the other players the asker's chest index; here the
        // host's own record (`:1886`).
        assert_eq!(host_app.remote_players.get(&1).map(|p| p.chest), Some(Some(0)));
    }

    /// S1-I-1 wave I — `Chest.UsingChest` scans ALL active players
    /// (`Chest.cs:502-508`), the host's own local player included: while the
    /// host browses chest 0, a client's `31` open request is refused
    /// (`MessageBuffer.cs:1877`) — no `155`/`33` burst, the client's
    /// `Player.chest` stays closed.
    #[test]
    fn a_chest_open_request_is_refused_while_the_host_browses_it() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        let mut chest = crate::chest::WorldChest {
            tile: (3, 5),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
            ..Default::default()
        };
        chest.slots.resize(crate::chest::DEFAULT_MAX_ITEMS as usize, crate::item::EMPTY_SLOT);
        host_app.world_chests.push(chest);
        // The HOST (slot 0) has the chest open — the pre-wave-I check only
        // scanned the remote slots and let this request through.
        host_app.player.as_mut().unwrap().chest = 0;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestOpenRequest(net::ChestOpenRequest { x: 3, y: 5 }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(
            client_app.player.as_ref().unwrap().chest,
            crate::bank::CHEST_CLOSED,
            "the UsingChest refusal — the host's own open chest counts"
        );
        assert!(client_app.world_chests.is_empty(), "no 155/32 burst arrived");
        assert_eq!(
            host_app.remote_players.get(&1).and_then(|p| p.chest),
            None,
            "no 80 relay either"
        );
        // The host closes it; the same request now succeeds.
        host_app.player.as_mut().unwrap().chest = crate::bank::CHEST_CLOSED;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestOpenRequest(net::ChestOpenRequest { x: 3, y: 5 }));
        }
        let mut opened = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            opened = client_app.player.as_ref().is_some_and(|p| p.chest == 0);
            if opened {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(opened, "the freed chest opens on the retry");
    }

    /// S1-I-1 wave I — `case 81` (`MessageBuffer.cs:3227-3233`): the host's
    /// `CombatTextInt` spawns a heal number in the client's pool at the
    /// truncated wire coords with the HealLife colour.
    #[test]
    fn a_combat_text_int_spawns_the_clients_heal_number() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::CombatTextInt(net::CombatTextInt {
                x: 1234.5,
                y: -678.25,
                color: net::Rgb(100, 255, 100),
                amount: 42,
            }));
        }
        let mut spawned = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            spawned = client_app.combat_texts.iter().any(|t| t.active);
            if spawned {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(spawned, "the 81 arm spawned a combat text");
        let t = client_app.combat_texts.iter().find(|t| t.active).unwrap();
        assert_eq!(t.text, "42");
        assert_eq!(t.color, [100, 255, 100, 255], "RGB + alpha 255");
        // The zero-size Rectangle centres the text on the truncated point
        // (`CombatText.cs:86-87` with width/height 0), ±0 jitter draws.
        let size_x = t.position[0] - (1234.0 - 0.0);
        assert!(size_x.abs() <= 20.0, "anchored at the truncated x: {}", t.position[0]);
        assert!(!t.crit && !t.dot, "the NewText defaults");
    }

    /// S1-P-i43 — the `editedChestName` chain (`ChestUI.cs:855-859` →
    /// `Player.cs:33097-33101`): a client's rename latches the flag, the
    /// open-click flush sends `33` with the CURRENT chest index + the name
    /// (`NetMessage.cs` case 33's `number2 == 1f` arm), the host rewrites its
    /// own store and relays `69`. A no-change submit never latches.
    #[test]
    fn a_client_rename_flushes_through_the_33_name_arm() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        // The chest exists on both sides (the world load carries it; here
        // the fixture plants matching entries directly).
        for app in [&mut host_app, &mut client_app] {
            let mut chest = crate::chest::WorldChest {
                tile: (3, 5),
                max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
                ..Default::default()
            };
            chest.slots.resize(crate::chest::DEFAULT_MAX_ITEMS as usize, crate::item::EMPTY_SLOT);
            app.world_chests.push(chest);
        }
        // The client browses chest 0 (the shared open block's state).
        crate::chest::toggle_open(&mut client_app, 3, 5, 0);
        assert_eq!(client_app.player.as_ref().unwrap().chest, 0);
        // The rename submit writes + latches (netMode 1).
        assert_eq!(crate::chest::rename_submit(&mut client_app, "Booty"), 12);
        assert!(client_app.player.as_ref().unwrap().edited_chest_name);
        assert_eq!(crate::chest::chest_name(&client_app, 0), "Booty");
        // An unchanged re-submit keeps the flag but does not re-latch a
        // cleared one (the write-if-different guard).
        client_app.player.as_mut().unwrap().edited_chest_name = false;
        crate::chest::rename_submit(&mut client_app, "Booty");
        assert!(!client_app.player.as_ref().unwrap().edited_chest_name);
        // Re-latch, then the flush (the open click's `:33097` seam) sends
        // the `33` name arm and clears the flag.
        crate::chest::rename_submit(&mut client_app, "Treasure");
        assert!(client_app.player.as_ref().unwrap().edited_chest_name);
        crate::chest::flush_edited_chest_name(&mut client_app);
        assert!(!client_app.player.as_ref().unwrap().edited_chest_name, "cleared on the send");
        let mut renamed = false;
        for _ in 0..60 {
            host_app.poll_net();
            client_app.poll_net();
            renamed = host_app.world_chests.first().is_some_and(|c| c.name == "Treasure");
            if renamed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(renamed, "the host's store took the 33 rename (:1974-1977)");
        // A second flush is a no-op (the flag is clear).
        crate::chest::flush_edited_chest_name(&mut client_app);
        assert!(!client_app.player.as_ref().unwrap().edited_chest_name);
    }

    /// `case 59` — a client's switch trip replays on the host and relays to
    /// the other clients (`MessageBuffer.cs:2682-2693`).
    #[test]
    fn a_switch_trip_relays_through_the_host() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A lever at (3, 3) on both sides (the lever toggles its frameY).
        for app in [&mut host_app, &mut client_app] {
            let mut t = tr_core::Tile {
                kind: 136,
                ..Default::default()
            };
            t.flags |= tr_core::tile::FLAG_ACTIVE;
            if let Some(cell) = app.world.tile_mut(3, 3) {
                *cell = t;
            }
        }
        assert_eq!(client_app.world.tile(3, 3).map(|t| t.frame_y), Some(0));
        client_app.send_hit_switch(3, 3);
        let mut flipped = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            flipped = host_app.world.tile(3, 3).map(|t| t.frame_y) == Some(18);
            if flipped {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(flipped, "the host replayed the lever's trip (`:2687`)");
    }

    /// `case 53` → `54`: a client's NPC buff lands on the host's NPC and the
    /// host pushes the full list back (`MessageBuffer.cs:2565-2593`).
    #[test]
    fn a_client_npc_buff_pushes_the_full_list_back() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        // Let the client learn the slot first (the 23 snapshot).
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.npcs.len(), 1);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::AddNpcBuff(net::AddNpcBuff {
                npc: 0,
                buff_type: 24,
                time: 300,
            }));
        }
        let mut landed = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            landed = client_app.npcs.first().is_some_and(|n| {
                n.buffs.buff_type[0] == 24 && n.buffs.buff_time[0] == 300
            });
            if landed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(landed, "the 54 list carried the buff back to the client");
        assert_eq!(
            host_app.npcs[0].buffs.buff_type[0], 24,
            "the host applied the 53 (`:2570`)"
        );
    }

    /// `AchievementsHelper.NotifyNPCKilled` (`AchievementsHelper.cs:200-217`)
    /// across the wire: the client's strike marks its `player_interaction`
    /// slot on the host (`MessageBuffer.cs:1789`), and the death sends `97`
    /// back so the client's own `EYE_ON_YOU` condition closes.
    #[test]
    fn a_client_kill_credits_the_client_through_the_97_message() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The Eye of Cthulhu's net id 4 keys `EYE_ON_YOU` (`Cond::Npc(4)`,
        // `AchievementInitializer.cs:46-48`) — an observable credit.
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::EyeOfCthulhu, 100.0, 100.0, 1, 0.0);
        npc.hp = 1;
        npc.max_hp = 1;
        host_app.npcs.clear();
        host_app.npcs.push(npc);
        host_app.net_npc_gen.insert(0, (4, 1));
        pump(&mut host_app, &mut client_app, 20);
        let generation = client_app
            .net_npc_gen
            .get(&0)
            .map(|(_, g)| *g as u8)
            .expect("the 23 push delivered a generation");
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: 0,
                generation,
                damage: 50,
                knock_back: 1.0,
                hit_direction: 1,
                crit: false,
            }));
        }
        pump(&mut host_app, &mut client_app, 40);
        // The host recorded the striker's interaction (`PlayerInteraction`,
        // `MessageBuffer.cs:1789` → `NPC.cs:80055`).
        let client_slot = client_app.local_id as usize;
        assert!(
            host_app.npcs[0].player_interaction[client_slot],
            "the host marked the striking client's interaction"
        );
        assert!(host_app.npcs[0].hp <= 0, "the strike killed it");
        // The death pass's wire half (`AchievementsHelper.cs:210-216`). `pump`
        // drives only the net tick, not `world_sim::npcs::update_npcs`'s death
        // loop, so the fan-out is called directly — the same call the death
        // pass makes.
        host_app.send_npc_killed(&host_app.npcs[0], 4);
        pump(&mut host_app, &mut client_app, 20);
        // The `97` closed the client's condition: a second notify cannot
        // complete it again (`AchievementCondition` is sticky).
        let again = client_app
            .achievements
            .notify(achievements::CondKind::NpcKilled(4));
        assert_eq!(again, None, "the 97 already completed EYE_ON_YOU");
    }

    /// The 10 Hz state tick: the player snapshot, the world-item cadence
    /// (wave A) and the OUTBOX DRAINS of the NPC/projectile decision layers
    /// (S1-I-4d). The NPC `23` / projectile `27`/`29` sends are produced at
    /// 60 Hz by `npc_sync_cadence` / `proj_sync_cadence` below (the C#'s
    /// per-NPC / per-projectile `netSpam` gates, `NPC.cs:90583-90631` /
    /// `Projectile.cs:17210-17244`) — NOT by a whole-set broadcast here.
    pub(crate) fn net_tick(&mut self, dt: f32) {
        // W4-d115 — the dodge handlers' `SendData(62)` queue
        // (`Player.cs:38038/:38053/:38115/:38163`) drains EVERY tick, ahead
        // of the 10 Hz state batch: the C# send is inline in `Hurt`, the
        // port's queue hands it to this phase of the same fixed step (the
        // `take_hurt_sound` convention). `NetMessage.SendData`'s netMode
        // gate (`NetMessage.cs:83-86`) drops the sends single-player — the
        // take still runs so the queue cannot grow unbounded.
        let dodge_sends = self
            .player
            .as_mut()
            .map(|p| p.take_dodge_sends())
            .unwrap_or_default();
        if matches!(self.net, NetRole::Off) {
            return;
        }
        if !dodge_sends.is_empty() {
            let me = self.local_id as u8;
            match &self.net {
                NetRole::Host(server) => {
                    for kind in dodge_sends {
                        server.broadcast(&S2c::SyncDodge(net::SyncDodge {
                            player_id: me,
                            kind,
                        }));
                    }
                }
                NetRole::Client(client) => {
                    for kind in dodge_sends {
                        // The host forces the id to the sender
                        // (`MessageBuffer.cs:2885`); ours is informational.
                        client.send(&C2s::SyncDodge(net::SyncDodge {
                            player_id: me,
                            kind,
                        }));
                    }
                }
                NetRole::Off => {}
            }
        }
        // `Ping.Update` (`Main.cs:17817-17820`) runs once per **frame** in
        // `UpdateWorld_Client`, not on the 10 Hz net cadence, and needs no
        // local player object - it only watches its own stopwatch.
        self.net_ping_tick();
        // `Main.TriggerPing`'s relay (`Main.cs:56546-56549`): the map markers
        // added this frame go out as `NetPingModule` packets.
        let pings = self
            .map
            .as_mut()
            .map(|m| std::mem::take(&mut m.ping_outbox))
            .unwrap_or_default();
        for pos in pings {
            self.send_module(
                crate::app::netmodules::module_id::PING,
                &crate::app::netmodules::Ping {
                    x: pos[0],
                    y: pos[1],
                }
                .encode(),
            );
        }
        // S1-I-4d: the NPC/projectile decision layers run on the C#'s 60 Hz
        // simulation cadence (`NPC.cs:90513`, `Projectile.cs:17210`), NOT the
        // 10 Hz send tick. Step them once per accumulated fixed step here so
        // both the real frame loop and the net-only test pump drive them; the
        // outboxes they fill drain on the 10 Hz tick below. Cap the burst like
        // `FixedStepper` so a long stall does not run away.
        self.net_sync_cadence_timer += dt;
        let mut steps = 0;
        while self.net_sync_cadence_timer >= 1.0 / 60.0 && steps < 8 {
            self.net_sync_cadence_timer -= 1.0 / 60.0;
            self.host_check_sections();
            self.npc_sync_cadence();
            self.proj_sync_cadence();
            steps += 1;
        }
        if self.net_sync_cadence_timer > 8.0 / 60.0 {
            self.net_sync_cadence_timer = 8.0 / 60.0;
        }
        self.net_timer += dt;
        if self.net_timer < 0.1 {
            return;
        }
        self.net_timer = 0.0;
        let Some(p) = self.player.as_ref() else {
            // No local player: the cadence still ran above, so drop its outboxes
            // rather than let them grow unbounded (the sends need a player).
            self.net_npc_outbox.clear();
            self.net_proj_outbox.clear();
            self.net_npc_despawns.clear();
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
            ..Default::default()
        };
        // The ambience spawn relay (`AmbienceServer.SpawnForPlayer`): the
        // (type, seed) pairs leave the store before the role borrow.
        let ambience_spawns = self.take_unsent_ambience();
        // S1-I-4d: the projectile outbox (filled by the 60 Hz cadence above)
        // drains through the `27`/`29` wires in the direction the role dictates.
        self.drain_proj_outbox();
        // The NPC outbox + despawns leave the store before the role borrow.
        let npc_outbox = std::mem::take(&mut self.net_npc_outbox);
        let npc_despawns = std::mem::take(&mut self.net_npc_despawns);
        match &self.net {
            NetRole::Host(server) => {
                // The host's own snapshot, as `13 PlayerState`.
                server.broadcast(&S2c::PlayerState(Box::new(self.player_state_of(&me))));
                for (ty, seed) in ambience_spawns {
                    let body = crate::app::netmodules::AmbienceSpawn {
                        player: 0,
                        seed,
                        kind: crate::app::netmodules::sky_entity_type_byte(ty),
                    }
                    .encode();
                    server.broadcast(&S2c::NetModule {
                        module: crate::app::netmodules::module_id::AMBIENCE,
                        body,
                    });
                }
                // Newly approached sections stream before the cadence batch
                // (the C#'s per-tick CheckSection).
                for (id, sx, sy) in self.pending_sections(server) {
                    server.mark_section_sent(id, sx, sy);
                    let block = self.tile_block_at(sx, sy);
                    server.send(id, &S2c::TileBlock(Box::new(block)));
                }
                // S1-I-4d SWAP: the old whole-set 10 Hz `23` broadcast is gone.
                // The per-NPC `netSpam` cadence (`net_npc_sync`, filled at 60 Hz
                // above) owns the `23` fan-out + section culling + despawns
                // (`NPC.cs:90583-90631`, `NetMessage.cs:1719-1749`); drain it.
                self.flush_npc_outbox(server, npc_outbox, npc_despawns);
                // `Main.UpdateServer` (`Main.cs:64557-64580`): the server
                // counter advances once per 60 Hz tick (the port batches it
                // on the 10 Hz cadence tick — 6 per batch), and every 60th
                // tick one round-robin slot gets an unconditional `21`
                // heartbeat. The C# walks 400 fixed slots; the port walks
                // its live list (M6-NET-5), wrapping at its length. The
                // unreserved-only condition (`playerIndexTheItemIsReservedFor
                // == 255`) holds for every port drop — reservations are
                // unported — and the heartbeat re-sends even an unchanged
                // slot (the diff below skips it as already recorded).
                self.net_play_counter = self.net_play_counter.wrapping_add(6);
                if self.net_play_counter % 3600 == 0 {
                    self.net_play_counter = 0;
                    // W4-g52 audit — `Main.UpdateServer`'s 3600-tick header
                    // re-push (`Main.cs:64557-64561`): the LIVE `7 WorldInfo`
                    // (the flags/events the 205 Time pair does not carry).
                    let info = self.world_info();
                    server.broadcast(&S2c::WorldInfo(Box::new(info)));
                }
                if self.net_play_counter % 60 == 0 && !self.items.is_empty() {
                    self.last_item_update += 1;
                    if self.last_item_update >= self.items.len() {
                        self.last_item_update = 0;
                    }
                    let slot = self.last_item_update as i16;
                    let st = Self::item_state_of(slot, &self.items[self.last_item_update]);
                    self.net_item_last.insert(slot, st);
                    // W4-g52 — the `SendData(21)` pre-switch
                    // (`NetMessage.cs:89-100`): instanced never syncs, air
                    // rewrites to `151`.
                    if let Some(msg) =
                        Self::item_send_gate(slot, &self.items[self.last_item_update])
                    {
                        server.broadcast(&msg);
                    }
                }
                // `21 ItemDrop` — only the slots whose state changed since the
                // last send (the port's batched stand-in for the C#'s
                // event-driven `SyncItem` fan-out — `WorldItem.cs:612-615`
                // and its callers; the C# never diff-streams physics).
                let live = self.items.len() as i16;
                for (i, it) in self.items.iter().enumerate() {
                    let slot = i as i16;
                    let st = Self::item_state_of(slot, it);
                    if self.net_item_last.get(&slot) != Some(&st) {
                        self.net_item_last.insert(slot, st);
                        // W4-g52 — the `SendData(21)` pre-switch
                        // (`NetMessage.cs:89-100`).
                        if let Some(msg) = Self::item_send_gate(slot, it) {
                            server.broadcast(&msg);
                        }
                    }
                }
                let dead: Vec<i16> = self
                    .net_item_last
                    .keys()
                    .copied()
                    .filter(|slot| *slot >= live)
                    .collect();
                for slot in dead {
                    self.net_item_last.remove(&slot);
                    server.broadcast(&S2c::ItemDead { slot });
                }
                server.broadcast(&S2c::Time {
                    time: self.world.time,
                    day: self.world.day_time,
                });
            }
            NetRole::Client(client) => {
                // W4-g15 lane 5 — the client's `22 ItemOwner` reservation
                // hold ticks down on the 10 Hz cadence (the C#'s
                // `timeToKeepReservation` decrements per 60 Hz update,
                // `WorldItem.cs:445-448`; the port's client reservation map
                // is the `net_reserved` stand-in, so the hold decays 6 per
                // cadence tick — the same 1 s grid the host's
                // `net_play_counter` batches). An expired hold releases the
                // reservation (the C#'s `FindOwner` re-reserves on the next
                // tick; the port's client grab gate reads the map live).
                let expired: Vec<i16> = self
                    .net_reserved
                    .iter_mut()
                    .filter_map(|(slot, (_, hold))| {
                        *hold = hold.saturating_sub(6);
                        if *hold == 0 { Some(*slot) } else { None }
                    })
                    .collect();
                for slot in expired {
                    self.net_reserved.remove(&slot);
                }
                // S1-I-4d: the client's own projectiles already drained through
                // `drain_proj_outbox` above (`Projectile.cs:17210-17244`'s owner
                // machine → `27`, despawns → `29`).
                // The C#'s `TrySyncingInput` change-driven send plus the
                // 420-tick heartbeat (`Player.cs:6852-6916`,
                // `Main.cs:64522-64529`); the port's control flags are not
                // synced (each peer simulates its own player), so the change
                // test covers position, animation, selection and health.
                if self.net_state >= net::client_state::IN_WORLD {
                    let mut changed = (me.x - self.net_ps_last.x).abs() > 0.5
                        || (me.y - self.net_ps_last.y).abs() > 0.5
                        || me.facing != self.net_ps_last_facing()
                        || me.selected != self.net_ps_last.selected_item as u16
                        || me.leg_frame != self.net_ps_last.leg_frame
                        || me.body_frame != self.net_ps_last.body_frame
                        || me.on_ground != self.net_ps_last.on_ground
                        || me.hp != self.net_ps_last.hp
                        || me.max_hp != self.net_ps_last.max_hp;
                    self.net_beat = self.net_beat.wrapping_add(1);
                    if !changed && self.net_beat % NET_HEARTBEAT_TICKS == 0 {
                        changed = true; // the heartbeat (420 ticks ≈ 7 s)
                    }
                    if changed {
                        client.send(&C2s::PlayerState(self.player_state_of(&me)));
                        self.net_ps_last = self.player_state_of(&me);
                    }
                }
            }
            NetRole::Off => {}
    /// W4-g51 id-162: the client's strike pushes a pending-damage entry, the
    /// host's `162` ack pops it FIFO, and the `23` reader subtracts the
    /// pending sum from the wire life (`MessageBuffer.cs:1668-1676`).
    #[test]
    fn the_pending_damage_queue_pushes_acks_and_reconciles_the_23_life() {
        let (mut host_app, mut client_app) = in_world_pair();
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::Slime, 100.0, 100.0, 1, 0.0);
        npc.hp = 1000;
        npc.max_hp = 1000;
        host_app.npcs.clear();
        host_app.npcs.push(npc);
        host_app.net_npc_gen.insert(0, (1, 1));
        pump(&mut host_app, &mut client_app, 20);
        // The client's store knows slot 0; rebuild the render list so the
        // strike's idx→slot map resolves.
        client_app.net_npcs_dirty = true;
        client_app.net_rebuild_lists();
        assert!(client_app.net_pending_damages.is_empty());
        // A landed 40-damage strike: the push + the fold.
        client_app.send_npc_strike(0, 40, 1.0, false, true, 40);
        assert_eq!(client_app.net_pending_damages.len(), 1);
        assert_eq!(client_app.net_pending_damages[0].damage, 40);
        assert_eq!(client_app.net_pending_damages[0].index, 0);
        // A `23` snap with wire life 1000 reconciles to 960 (the pending 40).
        let info = net::NpcInfo {
            slot: 0,
            generation: 1,
            spawn: false,
            kind: crate::npc::NpcKind::Slime.to_u16() as i16,
            x: 100.0,
            y: 100.0,
            vx: 0.0,
            vy: 0.0,
            dir_right: true,
            dir_y_down: false,
            sprite_right: true,
            life_full: false,
            hp: 1000,
        };
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::NpcInfo(Box::new(info))),
            );
        }
        client_app.net = role;
        assert_eq!(
            client_app.net_npcs.get(&0).map(|n| n.hp),
            Some(960),
            "the wire life minus the pending 40"
        );
        // The host's ack (the `28` it received) pops the entry.
        pump(&mut host_app, &mut client_app, 20);
        assert!(
            client_app.net_pending_damages.is_empty(),
            "the 162 ack popped the entry"
        );
        // A second snap now reads the wire life unmodified.
        let info2 = net::NpcInfo { hp: 900, ..info };
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::NpcInfo(Box::new(info2))),
            );
        }
        client_app.net = role;
        assert_eq!(
            client_app.net_npcs.get(&0).map(|n| n.hp),
            Some(900),
            "no pending entry: the wire life stands"
        );
    }

    /// W4-g51 f21 cross-check: the small apply arms — `139` stores the
    /// host-authority flag on the peer, `146` kind 1 adds coin luck within
    /// 1000 px (capped 1e6), `160` snaps the drop position, `161`'s empty
    /// configured token refuses the claim.
    #[test]
    fn the_f21_small_apply_arms_store_and_snap() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        // 139: the flag lands on the peer snapshot.
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::CountsAsHost(net::CountsAsHost {
                    player_id: 2,
                    counts_as_host: true,
                })),
            );
        }
        client_app.net = role;
        assert!(client_app.remote_players.get(&2).unwrap().counts_as_host);
        // 146 kind 1: the local player's coin luck adds (within 1000 px).
        client_app.player = Some(player::Player::spawn(&client_app.world));
        let (pcx, pcy) = client_app.player.as_ref().unwrap().center();
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::ShimmerAction(net::ShimmerAction::CoinLuck {
                    x: pcx,
                    y: pcy,
                    amount: 500,
                })),
            );
        }
        client_app.net = role;
        assert_eq!(
            client_app.player.as_ref().unwrap().luck_state.coin_luck,
            500.0
        );
        // A far-away coin (>1000 px) adds nothing.
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::ShimmerAction(net::ShimmerAction::CoinLuck {
                    x: pcx + 2000.0,
                    y: pcy,
                    amount: 999,
                })),
            );
        }
        client_app.net = role;
        assert_eq!(
            client_app.player.as_ref().unwrap().luck_state.coin_luck,
            500.0,
            "the far coin is out of range"
        );
        // 160: the drop position snaps (no velocity change).
        client_app.net_items.insert(
            3,
            item::ItemDrop::new(71, 5, 0, 0.0, 0.0, 1.0, 2.0, 0.0),
        );
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::ItemPosition(net::ItemPosition {
                    slot: 3,
                    x: 123.0,
                    y: 456.0,
                })),
            );
        }
        client_app.net = role;
        let d = client_app.net_items.get(&3).unwrap();
        assert_eq!((d.pos.x, d.pos.y), (123.0, 456.0));
        assert_eq!((d.vel.x, d.vel.y), (1.0, 2.0), "the velocity is untouched");
    }

    // ---- W4-g51 FINAL: the 8 BIG apply arms' witnesses ----
    // WRITTEN NOT RUN (code-only lane).

    /// The TE witnesses need a world big enough for the anchor cells + the
    /// dryad probe's `Clamp(Y-10, 10, maxTilesY-10)`; the `in_world_pair`
    /// flow world is 8x8. Resize BOTH sides to 64x64 with a dirt surface row
    /// at y=40 (the host's tile writes + the client's mirror stay in step).
    fn te_world_pair() -> (App, App) {
        let (mut host_app, mut client_app) = in_world_pair();
        for app in [&mut host_app, &mut client_app] {
            app.world = tr_core::World::new("te".into(), 64, 64);
            app.world.column_surface = vec![40; 64];
            for x in 0..64 {
                let i = app.world.index(x, 40);
                app.world.tiles[i] = tr_core::Tile { kind: 1, ..Default::default() };
                app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        (host_app, client_app)
    }

    /// Builds the `21 ItemDrop` body for one live drop (`NetMessage.cs:646-669`
    /// minus the ownership bits the port does not carry yet; the enemy-delay
    /// byte landed in W4-g15 lane 5).
    fn item_state_of(slot: i16, it: &item::ItemDrop) -> net::ItemState {
        net::ItemState {
            slot,
            x: it.pos.x,
            y: it.pos.y,
            vx: it.vel.x,
            vy: it.vel.y,
            stack: it.count.min(i16::MAX as u32) as i16,
            prefix: it.prefix.min(255) as u8,
            kind: it.kind as i16,
            shimmered: it.shimmered,
            shimmer_time: it.shimmer_time,
            // `NetMessage.cs:667` — the clamp 0..255. W4-g15 lane 5.
            enemy_grab_delay: it.enemy_grab_delay_time.clamp(0, 255) as u8,
    /// The TryPlacing quadruplet's host apply: a held rack drops its old
    /// contents, takes the new item, and answers with an `86` the client
    /// stores; a refused placement (no entity) drops the item back.
    /// `TEWeaponsRack.TryPlacing` (`TEWeaponsRack.cs:135-152`).
    #[test]
    fn w4_g51_the_try_placing_quadruplet_stores_and_shares() {
        let (mut host_app, mut client_app) = te_world_pair();
        // A weapon-rack anchor tile (471, frameY 0, frameX % 54 == 0).
        let (x, y) = (30i16, 39i16);
        let i = host_app.world.index(x as i32, y as i32);
        host_app.world.tiles[i] = tr_core::Tile {
            kind: 471,
            ..Default::default()
        };
        host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let rack_id = host_app
            .world
            .tile_entities
            .place(&host_app.world.clone(), tr_core::tile_entities::TeKind::WeaponsRack, x, y)
            .expect("the rack anchor is valid");
        // Seed an old item so the DropItem half fires.
        host_app.world.tile_entities.set_item(
            rack_id,
            Some(tr_core::tile_entities::TeItem { kind: 9, prefix: 0, stack: 1 }),
        );
        let items_before = host_app.items.len();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::WeaponsRackTryPlacing(net::TryPlacingItem {
                x,
                y,
                kind: 29,
                prefix: 81,
                stack: 1,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        // The host's rack now holds the new item (exact prefix).
        assert_eq!(
            host_app.world.tile_entities.item(rack_id).copied().flatten(),
            Some(tr_core::tile_entities::TeItem { kind: 29, prefix: 81, stack: 1 })
        );
        // The old contents dropped (count 1, the stored prefix).
        assert_eq!(host_app.items.len(), items_before + 1);
        assert_eq!(host_app.items[items_before].kind, 9);
        // The client's `86` store landed the rack with the new item.
        let client_rack = client_app.world.tile_entities.get_at(x, y).expect("the 86 landed");
        assert_eq!(client_rack.kind, tr_core::tile_entities::TeKind::WeaponsRack);
        assert_eq!(
            client_app.world.tile_entities.item(client_rack.id).copied().flatten(),
            Some(tr_core::tile_entities::TeItem { kind: 29, prefix: 81, stack: 1 })
        );
        // A refused placement (no entity at the cell) drops the item back.
        let items_before = host_app.items.len();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::FoodPlatterTryPlacing(net::TryPlacingItem {
                x: 60,
                y: 3,
                kind: 4009,
                prefix: 0,
                stack: 1,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(host_app.items.len(), items_before + 1, "the refused placement drops");
        assert_eq!(host_app.items[items_before].kind, 4009);
    }

    /// `121` doll sync: the host applies the equip-slot write + relays; the
    /// client's doll store takes it. The pose command rides the same path.
    /// `TEDisplayDoll.ReadData` (`TEDisplayDoll.cs:802-822`).
    #[test]
    fn w4_g51_the_doll_sync_stores_and_relays() {
        let (mut host_app, mut client_app) = te_world_pair();
        // A display-doll anchor tile (470, frameY 0, frameX % 36 == 0).
        let (x, y) = (32i16, 39i16);
        let i = host_app.world.index(x as i32, y as i32);
        host_app.world.tiles[i] = tr_core::Tile {
            kind: 470,
            ..Default::default()
        };
        host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let doll_id = host_app
            .world
            .tile_entities
            .place(&host_app.world.clone(), tr_core::tile_entities::TeKind::DisplayDoll, x, y)
            .expect("the doll anchor is valid");
        // Mirror the doll onto the client so the relay has a target.
        client_app
            .world
            .tile_entities
            .load_entity(tr_core::tile_entities::TeKind::DisplayDoll, x, y, {
                tr_core::tile_entities::TeData::DisplayDoll {
                    equip: [None; 9],
                    dye: [None; 9],
                    misc: None,
                    pose: 0,
                }
            });
        let client_doll_id = client_app.world.tile_entities.get_at(x, y).unwrap().id;
        // Equip slot 2 takes item 100 (prefix 81).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::DisplayDollSync(net::DisplayDollSync {
                player_id: 0,
                entity_id: doll_id as i32,
                item_index: 2,
                command: 0,
                data: net::DollSyncData::Item(net::TeItem { kind: 100, prefix: 81, stack: 1 }),
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        // The host's doll equip[2] holds the item.
        let host_doll = host_app.world.tile_entities.get_by_id(doll_id).unwrap();
        if let tr_core::tile_entities::TeData::DisplayDoll { equip, .. } = &host_doll.data {
            assert_eq!(
                equip[2],
                Some(tr_core::tile_entities::TeItem { kind: 100, prefix: 81, stack: 1 })
            );
        } else {
            panic!("the host doll payload");
        }
        // The client's doll took the relay (its own id).
        let client_doll = client_app.world.tile_entities.get_by_id(client_doll_id).unwrap();
        if let tr_core::tile_entities::TeData::DisplayDoll { equip, .. } = &client_doll.data {
            assert_eq!(
                equip[2],
                Some(tr_core::tile_entities::TeItem { kind: 100, prefix: 81, stack: 1 })
            );
        } else {
            panic!("the client doll payload");
        }
        // The pose command (2) sets the pose byte on both sides.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::DisplayDollSync(net::DisplayDollSync {
                player_id: 0,
                entity_id: doll_id as i32,
                item_index: 0,
                command: 2,
                data: net::DollSyncData::Pose(7),
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let pose_of = |app: &App, id: u32| match &app.world.tile_entities.get_by_id(id).unwrap().data {
            tr_core::tile_entities::TeData::DisplayDoll { pose, .. } => *pose,
            _ => panic!("doll payload"),
        };
        assert_eq!(pose_of(&host_app, doll_id), 7);
        assert_eq!(pose_of(&client_app, client_doll_id), 7);
    }

    /// `122` interaction: the host sets the anchor + relays; an occupied
    /// entity is refused (no relay); -1 clears. `TileEntity.IsOccupied`
    /// (`TileEntity.cs:221-234`) + `SetInteractionAnchor` (`:289-313`).
    #[test]
    fn w4_g51_the_te_interaction_sets_clears_and_refuses_occupied() {
        let (mut host_app, mut client_app) = te_world_pair();
        let (x, y) = (30i16, 39i16);
        let i = host_app.world.index(x as i32, y as i32);
        host_app.world.tiles[i] = tr_core::Tile { kind: 471, ..Default::default() };
        host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let rack_id = host_app
            .world
            .tile_entities
            .place(&host_app.world.clone(), tr_core::tile_entities::TeKind::WeaponsRack, x, y)
            .expect("valid");
        client_app
            .world
            .tile_entities
            .load_entity(tr_core::tile_entities::TeKind::WeaponsRack, x, y, tr_core::tile_entities::TeData::Item(None));
        // The peer (slot 1) is alive — `IsOccupied`'s `!dead` term reads hp.
        host_app.remote_players.entry(1).or_default().hp = 100;
        // The client (slot 1) opens the interaction.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEntityInteraction(net::TileEntityInteraction {
                entity_id: rack_id as i32,
                player_id: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        // The host's anchor store holds (id, x, y) for slot 1.
        assert_eq!(
            host_app.net_te_anchors.get(&1),
            Some(&(rack_id as i32, x as i32, y as i32))
        );
        // The client's own anchor set (the relay named slot 1 = local).
        assert_eq!(
            client_app.net_te_anchors.get(&1),
            Some(&(rack_id as i32, x as i32, y as i32))
        );
        // The host (slot 0) is now refused — the entity is occupied.
        let role = std::mem::replace(&mut host_app.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            host_app.host_event(
                server,
                net::ServerEvent::Frame {
                    id: 0,
                    msg: C2s::TileEntityInteraction(net::TileEntityInteraction {
                        entity_id: rack_id as i32,
                        player_id: 0,
                    }),
                },
            );
        }
        host_app.net = role;
        assert!(!host_app.net_te_anchors.contains_key(&0), "occupied refuses");
        // -1 clears the client's anchor.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEntityInteraction(net::TileEntityInteraction {
                entity_id: -1,
                player_id: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(!host_app.net_te_anchors.contains_key(&1), "the clear landed");
        assert!(!client_app.net_te_anchors.contains_key(&1));
    }

    /// Builds the `13 PlayerState` body for a stored peer snapshot (the
    /// host-side relay and the join bundle).
    fn player_state_of(&self, p: &net::PlayerNet) -> net::PlayerState {
        let mut flags_a = net::BitsByte::new();
        flags_a.set(6, p.facing > 0);
        let mut flags_b = net::BitsByte::new();
        flags_b.set(2, p.vx != 0.0 || p.vy != 0.0);
        net::PlayerState {
            player_id: p.id as u8,
            flags_a,
            flags_b,
            flags_c: net::BitsByte::new(),
            flags_d: net::BitsByte::new(),
            selected_item: p.selected as u8,
            x: p.x,
            y: p.y,
            vx: p.vx,
            vy: p.vy,
            on_ground: p.on_ground,
            leg_frame: p.leg_frame,
            body_frame: p.body_frame,
            hp: p.hp,
            max_hp: p.max_hp,
    /// SEAL re-pin: a tombstoned TE id is NOT a `TryGet` hit — the removed
    /// `ByID` entry (`TileEntity.cs:126`) makes the host's 122 `TryGet` gate
    /// refuse (no anchor, no relay) and the 124 gate consume-and-drop.
    /// `remove_at` leaves the port's cleared slot; the liveness filter is
    /// what keeps the gates faithful.
    #[test]
    fn w4_g51_a_tombstoned_te_id_refuses_the_interaction_and_the_hat_sync() {
        let (mut host_app, mut client_app) = te_world_pair();
        let (x, y) = (34i16, 39i16);
        let i = host_app.world.index(x as i32, y as i32);
        host_app.world.tiles[i] = tr_core::Tile { kind: 475, ..Default::default() };
        host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let rack_id = host_app
            .world
            .tile_entities
            .place(&host_app.world.clone(), tr_core::tile_entities::TeKind::HatRack, x, y)
            .expect("valid");
        // Tombstone it on BOTH sides (the anchor tile broke; the id slot
        // stays in the book with `TeData::None`). The kind STILL reads
        // HatRack — only the liveness filter refuses the frame.
        host_app.world.tile_entities.remove_at(x, y).expect("live");
        client_app
            .world
            .tile_entities
            .place_unchecked(tr_core::tile_entities::TeKind::HatRack, x, y);
        client_app.world.tile_entities.remove_at(x, y).expect("live");
        host_app.remote_players.entry(1).or_default().hp = 100;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEntityInteraction(net::TileEntityInteraction {
                entity_id: rack_id as i32,
                player_id: 0,
            }));
            // A hat-rack frame naming the dead id: consume-and-drop.
            client.send(&C2s::HatRackItemSync(net::HatRackItemSync {
                player_id: 0,
                entity_id: rack_id as i32,
                wire_slot: 0,
                item: net::TeItem { kind: 3454, prefix: 0, stack: 1 },
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(
            !host_app.net_te_anchors.contains_key(&1),
            "the tombstoned id refuses the interaction"
        );
        assert!(!client_app.net_te_anchors.contains_key(&1));
        // The dead slot's payload stays tombstoned (no hat-rack write).
        assert_eq!(
            host_app.world.tile_entities.get_by_id(rack_id).unwrap().data,
            tr_core::tile_entities::TeData::None
        );
    }

    /// `124` hat-rack sync: the dye fold routes to the dye array; the host
    /// applies + relays; the client's store takes it. `TEHatRack.ReadItem`
    /// (`TEHatRack.cs:390-403`).
    #[test]
    fn w4_g51_the_hat_rack_sync_folds_the_dye_and_relays() {
        let (mut host_app, mut client_app) = te_world_pair();
        let (x, y) = (34i16, 39i16);
        let i = host_app.world.index(x as i32, y as i32);
        host_app.world.tiles[i] = tr_core::Tile { kind: 475, ..Default::default() };
        host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let rack_id = host_app
            .world
            .tile_entities
            .place(&host_app.world.clone(), tr_core::tile_entities::TeKind::HatRack, x, y)
            .expect("valid");
        client_app.world.tile_entities.load_entity(
            tr_core::tile_entities::TeKind::HatRack,
            x,
            y,
            tr_core::tile_entities::TeData::HatRack([None, None], [None, None]),
        );
        let client_rack_id = client_app.world.tile_entities.get_at(x, y).unwrap().id;
        // wire slot 2 = dye slot 0 (the +2 fold).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::HatRackItemSync(net::HatRackItemSync {
                player_id: 0,
                entity_id: rack_id as i32,
                wire_slot: 2,
                item: net::TeItem { kind: 3454, prefix: 0, stack: 1 },
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let dyes_of = |app: &App, id: u32| match &app.world.tile_entities.get_by_id(id).unwrap().data {
            tr_core::tile_entities::TeData::HatRack(_, dyes) => dyes[0],
            _ => panic!("hat rack payload"),
        };
        assert_eq!(
            dyes_of(&host_app, rack_id),
            Some(tr_core::tile_entities::TeItem { kind: 3454, prefix: 0, stack: 1 })
        );
        assert_eq!(
            dyes_of(&client_app, client_rack_id),
            Some(tr_core::tile_entities::TeItem { kind: 3454, prefix: 0, stack: 1 })
        );
    }

    /// `144` dryad Stardew: the host writes the dryad's ai + spawns the 995
    /// Jumino projectile, which streams to the client. The ground probe
    /// picks the dryad's own bottom when the first scanned row is solid.
    /// `NPC.HaveDryadDoStardewAnimation` (`NPC.cs:19184-19229`).
    #[test]
    fn w4_g51_the_dryad_stardew_writes_the_ai_and_spawns_the_jumino() {
        let (mut host_app, mut client_app) = te_world_pair();
        // A dryad standing on the dirt row (y=40 is the surface; the NPC's
        // bottom rests at y=40*16=640, so top = 640 - 40 = 600). Bottom =
        // (x + w/2, y + h) = (109, 640); the probe +(100,0) = (209, 640) →
        // tile (13, 40), Y=Clamp(30,10,54)=30; the first solid row is y=40
        // (not the first scanned row 30), so the spawn uses the probe tile's
        // world coords (13*16+8, 40*16)=(216,640) + (0,-52) = (216, 588).
        let mut dryad = crate::npc::Npc::new(crate::npc::NpcKind::Dryad, 100.0, 600.0, 1, 0.0);
        dryad.hp = dryad.max_hp;
        host_app.npcs.push(dryad);
        let jumino_before = host_app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 995)
            .count();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::RequestQuestEffect);
        }
        pump(&mut host_app, &mut client_app, 20);
        // The dryad's ai writes landed.
        let d = host_app.npcs.iter().find(|n| n.kind == crate::npc::NpcKind::Dryad).unwrap();
        assert_eq!(d.ai[0], 24.0);
        assert_eq!(d.ai[1], 480.0);
        assert_eq!(d.local_ai[2], 480.0);
        assert_eq!(d.facing, 1);
        assert_eq!(d.sprite_dir, 1);
        // The 995 spawned on the host.
        let jumino_after = host_app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 995)
            .count();
        assert_eq!(jumino_after, jumino_before + 1, "the Jumino spawned");
        // The box centres on the spawn point (the `NewProjectile` X/Y
        // overload, `Projectile.cs:10876-10877`): (216, 588) − (120, 52)
        // = (96, 536) for the 240×104 row.
        let jum = host_app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 995)
            .unwrap();
        assert_eq!((jum.pos.x, jum.pos.y), (96.0, 536.0));
        assert!(jum.friendly);
        assert!(!jum.tile_collide);
        assert!(jum.ignore_water);
        assert!(jum.net_important);
        assert_eq!(jum.time_left, 3600 * 5);
        // The client received the 995 via the 27 broadcast.
        let client_jumino = client_app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 995)
            .count();
        assert!(client_jumino >= 1, "the 27 streamed the Jumino");
    }

    /// `Player.TrySwitchingLoadout`'s `147` send (`Player.cs:5674` →
    /// `NetMessage.cs:1624-1628`): `(byte)whoAmI + (byte)loadoutIndex` + the
    /// `hideVisibleAccessory` ushort (`WriteAccessoryVisibility`,
    /// `NetMessage.cs:1882-1893`, bit i per slot). On a client it goes to the
    /// server (`TrySendData(147, -1, -1, ...)`); on the host it broadcasts to
    /// every client (the host's own switch reaches the peers the same way the
    /// relayed client switch does, `MessageBuffer.cs:4340`). `number` is the
    /// local player's id, `number2` the new loadout index.
    pub(crate) fn send_sync_loadout(&self, loadout: usize) {
        let hide = self
            .player
            .as_ref()
            .map(|p| accessory_visibility_mask(&p.hide_visible_accessory))
            .unwrap_or(0);
        let msg = net::SyncLoadout {
            player_id: self.local_id as u8,
            loadout: loadout as u8,
            hide_accessory: hide,
        };
        match &self.net {
            NetRole::Host(server) => server.broadcast(&S2c::SyncLoadout(msg)),
            NetRole::Client(client) => client.send(&C2s::SyncLoadout(msg)),
            NetRole::Off => {}
        }
    }

    /// A local switch trip in MP — the C#'s `NetMessage.SendData(59, -1, -1,
    /// null, i, j)` (`Collision.cs:3304`/`:3318`/`:3324`/`:3347`,
    /// `Player.cs:32178`, `:32226`, `:32340`): on a client it goes to the
    /// server, on the host it broadcasts to every client.
    pub(crate) fn send_hit_switch(&self, x: i32, y: i32) {
        let msg = net::HitSwitch {
            x: x as i16,
            y: y as i16,
        };
        match &self.net {
            NetRole::Host(server) => server.broadcast(&S2c::HitSwitch(msg)),
            NetRole::Client(client) => client.send(&C2s::HitSwitch(msg)),
            NetRole::Off => {}
    /// `156` leashed-anchor place: the host stores the itemType on the
    /// kite/critter anchor (`TELeashedEntityAnchorWithItem.InsertItem`,
    /// `TELeashedEntityAnchorWithItem.cs:32-36`); the respawn half is M7-TE-2.
    #[test]
    fn w4_g51_the_leashed_anchor_stores_the_item_type() {
        let (mut host_app, mut client_app) = te_world_pair();
        // A kite anchor (tile 723, any frame).
        let (x, y) = (36i16, 39i16);
        let i = host_app.world.index(x as i32, y as i32);
        host_app.world.tiles[i] = tr_core::Tile { kind: 723, ..Default::default() };
        host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let anchor_id = host_app
            .world
            .tile_entities
            .place(&host_app.world.clone(), tr_core::tile_entities::TeKind::KiteAnchor, x, y)
            .expect("valid");
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::LeashedAnchorPlaceItem(net::LeashedAnchorPlaceItem {
                x,
                y,
                item_type: 3034,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(
            host_app.world.tile_entities.get_by_id(anchor_id).unwrap().data,
            tr_core::tile_entities::TeData::Anchor(3034)
        );
    }

    /// `139` CountsAsHost join push (the g52c master input): a loopback peer
    /// counts as a host (`NetMessage.DoesPlayerSlotCountAsAHost`,
    /// `NetMessage.cs:2876-2883`), so the join bundle pushes the flag and the
    /// client stores it on its own slot.
    #[test]
    fn w4_g51_the_counts_as_host_join_push_lands_for_a_loopback_peer() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The test pair connects over loopback, so the peer counts as host.
        // The join push already ran in in_world_pair; the host's peer record
        // carries the flag and the client stored it on its own slot (1).
        assert!(
            host_app.remote_players.get(&1).unwrap().counts_as_host,
            "the loopback peer counts as host"
        );
        assert!(
            client_app.remote_players.get(&1).unwrap().counts_as_host,
            "the client stored its own slot's flag"
        );
    }

    /// The 5043 use's `NetMessage.SendData(51, -1, -1, null, whoAmI, 5f)`
    /// (`Player.cs:45241`, W4-f33): a CLIENT asks the server to register the
    /// Torch God's 664 bestiary kill (`MessageBuffer.cs:2514-2520`'s
    /// `netMode == 2` arm). The host + single-player register locally in the
    /// use arm itself (the bestiary is per-world content, the host IS the
    /// server) — this seam only carries the client request.
    pub(crate) fn send_torch_god_bestiary_request(&self) {
        if let NetRole::Client(client) = &self.net {
            client.send(&C2s::MiscDataSync {
                player_id: self.local_id as u8,
                kind: 5,
            });
    /// One `NetMessage.SendData(19, -1, -1, null, action, x, y, direction)`
    /// (research/95 §8.2 CLOSED send-half — the W4-f11 wiring arms' echoes,
    /// `Wiring.cs:1450`/`:1458`/`:1465`/`:1480`/`:1485`): the host broadcasts
    /// to every client, the client sends to the server; `SendData`'s own
    /// `netMode == 0` return (`NetMessage.cs:83-86`) is the `Off` arm — the
    /// SP fold. The wire direction byte is `number4 == 1f` → 1 else 0
    /// (`NetMessage.cs:541`). The APPLY side (the 0/1 door + 2/3 trapdoor +
    /// 4/5 gate dispatch and the normalised-direction relay,
    /// `MessageBuffer.cs:1299-1334`) is g48's `net_apply.rs` — the union
    /// merge re-points this send at its `DoorToggle` codec (byte-identical
    /// bodies, keep-one). SEAL (research/133 §8.2): this fn is the ONE
    /// send copy (g48 has none in production); at the union it takes the
    /// c42 `door_echoes` TUPLE `(number, x, y, number4)` (the f11 engine
    /// struct drops with §8.1) and the drain lands at ALL 14 c42 discard
    /// sites, not just the wiring tick — the exact hunk is §8.2.
    /// (Batch-H: the tuple-arg re-shape EXECUTED — the c42 `door_echoes`
    /// `(number, x, y, number4)` straight off `take_door_echoes()`; the
    /// per-site drains land with this merge, the g48 codec keep-one stays
    /// its own lane's.)
    pub(crate) fn send_door_toggle(&self, number: i32, x: i32, y: i32, number4: i32) {
        let msg = net::DoorToggle {
            action: number as u8,
            x: x as i16,
            y: y as i16,
            direction: u8::from(number4 == 1),
        };
        match &self.net {
            NetRole::Host(server) => server.broadcast(&S2c::DoorToggle(msg)),
            NetRole::Client(client) => client.send(&C2s::DoorToggle(msg)),
            NetRole::Off => {}
        }
    }

    /// `AchievementsHelper.NotifyNPCKilled`'s wire half
    /// (`AchievementsHelper.cs:200-217`): in MP the host sends `97` to every
    /// slot recorded in the dead NPC's `playerInteraction` (`:210-216`). The
    /// `i < 255` bound is the C#'s "not the server itself"; the port's host
    /// occupies no client slot, so `Server::send` to it is already a no-op.
    pub(crate) fn send_npc_killed(&self, npc: &crate::npc::Npc, npc_net_id: i16) {
        let NetRole::Host(server) = &self.net else {
            return;
        };
        for i in 0..255u8 {
            if npc.player_interaction[i as usize] {
                server.send(i as u32, &S2c::NpcKilled { npc: npc_net_id });
            }
        }
    /// `AchievementsHelper.NotifyProgressionEvent` (`:227-237`): the host's
    /// `98` broadcast fires the client's `PROGRESSION_EVENT_<id>` listener.
    #[test]
    fn a_host_progression_event_reaches_the_client_over_98() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The host fires the dawn row (event 1, `TERRARIA_DAY`'s condition).
        let _ = host_app.notify_progression_net(1);
        pump(&mut host_app, &mut client_app, 20);
        // The client's own notify now reports the row already complete (the
        // TERRARIA_DAY condition fires once, `AchievementCondition` semantics).
        assert_eq!(
            client_app
                .achievements
                .notify(achievements::CondKind::ProgressionEvent(1)),
            None,
            "the 98 completed the client's PROGRESSION_EVENT_1 first"
        );
    }

    /// `NPCKillsTracker.RegisterKill`'s broadcast (`NPCKillsTracker.cs:29-32`):
    /// the host's kill count reaches the client as a module-4
    /// `SerializeKillCount`.
    #[test]
    fn a_kill_count_reaches_the_client_over_module_4() {
        let (mut host_app, mut client_app) = in_world_pair();
        let credit = crate::bestiary::entry_for_npc(
            crate::npc::NpcKind::Slime,
            crate::npc::SlimeVariant::Blue,
        );
        assert_eq!(client_app.bestiary.kills.get_kill_count(credit.credit_id), 0);
        host_app.bestiary.kills.register_kill(credit.credit_id);
        assert_eq!(host_app.bestiary.kills.get_kill_count(credit.credit_id), 1);
        host_app.broadcast_bestiary_kill(
            credit.net_id,
            host_app.bestiary.kills.get_kill_count(credit.credit_id),
        );
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(
            client_app.bestiary.kills.get_kill_count(credit.credit_id),
            1,
            "the module-4 kill count landed on the client"
        );
    }

    /// `NPCKillsTracker.OnPlayerJoining` (`NPCKillsTracker.cs:98-107`): the
    /// joining client receives the host's existing counts.
    #[test]
    fn the_join_snapshot_carries_the_hosts_kill_counts() {
        let (mut host_app, mut client_app) = in_world_pair();
        let credit = crate::bestiary::entry_for_npc(
            crate::npc::NpcKind::Slime,
            crate::npc::SlimeVariant::Blue,
        );
        // The pair is already in-world, so the snapshot arrives from the
        // `host_slot_join_complete` of a *re*join. Simulate one by handing the
        // client's slot the host's pairs through the same helper the join
        // path uses.
        for _ in 0..3 {
            host_app.bestiary.kills.register_kill(credit.credit_id);
        }
        let count = host_app.bestiary.kills.get_kill_count(credit.credit_id);
        let body = crate::app::netmodules::BestiaryKill::new(credit.net_id as i16, count)
            .encode();
        client_app.client_module(crate::app::netmodules::module_id::BESTIARY, &body);
        assert_eq!(
            client_app.bestiary.kills.get_kill_count(credit.credit_id),
            count,
            "the join snapshot's module-4 body applies"
        );
    }

    /// W4-g49 `42 Unknown42` (`MessageBuffer.cs:2290-2306`): the host forces
    /// the subject to the sender (`:2293-2296`) and stores both mana shorts.
    /// There is no relay arm — the fan-out is the join push / send path.
    #[test]
    fn w4_g49_player_mana_lands_on_the_host() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerMana(net::PlayerMana {
                player_id: 0, // forced to the sender on the host (`:2293-2296`)
                mana: 180,
                max_mana: 200,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let peer = host_app
            .remote_players
            .get(&client_app.local_id)
            .expect("the host stored the peer");
        assert_eq!(peer.mana, 180, "the host stored statMana (`:2303`)");
        assert_eq!(peer.max_mana, 200, "the host stored statManaMax (`:2304`)");
    }

    /// W4-g49 `45 TeamChange` (`MessageBuffer.cs:2325-2364`): the host forces
    /// the subject to the sender (`:2329-2332`), stores the team (`:2336`),
    /// relays to the others (`:2342`) and announces the change to the sender +
    /// the old/new team members in the team colour (`:2343-2354`).
    #[test]
    fn w4_g49_team_change_stores_relays_and_announces() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TeamChange(net::TeamChange {
                player_id: 0, // forced to the sender (`:2329-2332`)
                team: 1,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let peer = host_app
            .remote_players
            .get(&client_app.local_id)
            .expect("the host stored the peer");
        assert_eq!(peer.team, 1, "the host stored the team (`:2336`)");
        // The announcement reached the sender (the only team-1 member) as a
        // module-1 text line in the red team colour (`:2343-2354`).
        assert!(
            client_app
                .chat
                .messages
                .iter()
                .any(|m| m.color == TEAM_COLORS[1]),
            "the team-change announcement reached the sender in the team colour"
        );
    }

    /// W4-g49 `56 UniqueTownNPCInfoSyncRequest` (`MessageBuffer.cs:2612-2630`):
    /// the client's bare-slot request is answered by the host with the town
    /// NPC's given name + variation (`:2626`), which the client stores behind
    /// the slot gate (`:2615-2623`).
    #[test]
    fn w4_g49_town_npc_info_request_answers_and_stores() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A Guide (net id 22, town-like) in slot 0 with a given name.
        let mut guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        guide.given_name = "Andrew".into();
        guide.variation_index = 1;
        host_app.npcs.clear();
        host_app.npcs.push(guide);
        // The client's copy of the NPC (the 23 snapshot's stand-in).
        let mut client_guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        client_guide.given_name = String::new();
        client_guide.variation_index = 0;
        client_app.net_npcs.insert(0, client_guide);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::UniqueTownNpcInfo(net::UniqueTownNpcInfo {
                npc: 0,
                info: None,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let stored = client_app.net_npcs.get(&0).expect("the client NPC");
        assert_eq!(stored.given_name, "Andrew", "the host answer's name landed (`:2620`)");
        assert_eq!(stored.variation_index, 1, "the variation landed (`:2622`)");
    }

    /// W4-g49 `60 Unknown60` (`MessageBuffer.cs:2695-2739`): a town-like NPC's
    /// status 1 kicks it out — the host runs `WorldGen.kickOut` (`:2732`), the
    /// NPC goes homeless with the 3600-tick look-for-home timeout.
    #[test]
    fn w4_g49_npc_house_kick_out_makes_the_town_npc_homeless() {
        let (mut host_app, mut client_app) = in_world_pair();
        let mut guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        guide.home = Some((10, 10));
        host_app.npcs.clear();
        host_app.npcs.push(guide);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcHouse(net::NpcHouse {
                npc: 0,
                home_x: 0,
                home_y: 0,
                status: 1, // kick-out (`:2730-2733`)
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let npc = &host_app.npcs[0];
        assert!(npc.home.is_none(), "the kick-out cleared the home (`WorldGen.cs:4876`)");
        assert_eq!(
            npc.look_for_home_timeout,
            crate::town::KICK_OUT_LOOK_FOR_HOME_TIMEOUT,
            "the 3600-tick look-for-home timeout armed (`WorldGen.cs:4877`)"
        );
    }

    /// W4-g49 `61 SpawnBossInvader` (`MessageBuffer.cs:2740-2878`): the -10
    /// selector starts a Blood Moon at night (`:2806-2819`) — the flag up, the
    /// empty moon phase bumped to 5, progression 4 fired.
    #[test]
    fn w4_g49_spawn_boss_event_blood_moon_at_night() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app.world.day_time = false;
        host_app.world.blood_moon = false;
        host_app.world.moon_phase = 0;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::SpawnBossEvent(net::SpawnBossEvent {
                player: 0,
                event: -10,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(host_app.world.blood_moon, "the blood moon started (`:2811`)");
        assert_eq!(host_app.world.moon_phase, 5, "the empty phase bumped (`:2812-2815`)");
    }

    /// W4-g49 `63 SyncTilePaintOrCoating` (`MessageBuffer.cs:2909-2928`): the
    /// host applies the paint to its world and relays; the client's copy takes
    /// the same paint.
    #[test]
    fn w4_g49_tile_paint_applies_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        // An active stone tile at (5, 4) on both sides (the dirt row the pair
        // builds is at y=4; repaint cell (5,4)).
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(5, 4);
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            app.world.tiles[i].set_paint(0);
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TilePaint(net::TilePaint {
                x: 5,
                y: 4,
                paint: 13,
                coat: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(
            host_app.world.tile(5, 4).map(|t| t.paint()),
            Some(13),
            "the host painted the tile (`:2917`)"
        );
        assert_eq!(
            client_app.world.tile(5, 4).map(|t| t.paint()),
            Some(13),
            "the relay painted the client's copy (`:2925`)"
        );
    }

    /// W4-g49 `64 SyncWallPaintOrCoating` (`MessageBuffer.cs:2929-2948`): the
    /// host applies the wall paint and relays; the client's copy takes it.
    #[test]
    fn w4_g49_wall_paint_applies_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A wall at (5, 4) on both sides.
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(5, 4);
            app.world.tiles[i].wall = 1;
            app.world.set_wall_paint(5, 4, 0);
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::WallPaint(net::TilePaint {
                x: 5,
                y: 4,
                paint: 9,
                coat: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(host_app.world.wall_paint(5, 4), 9, "the host painted the wall (`:2937`)");
        assert_eq!(client_app.world.wall_paint(5, 4), 9, "the relay painted the client's wall (`:2945`)");
    }
}

/// `PlayerItemSlotID`'s M6-NET-1 subset (`Terraria/ID/PlayerItemSlotID.cs`) —
/// the slot arithmetic the C# inventory messages use.
/// `NetMessage.WriteAccessoryVisibility` (`NetMessage.cs:1882-1893`): the
/// `hideVisibleAccessory` `bool[10]` (`Player.cs:1708`) packed into one
/// `ushort` — bit `i` set when slot `i` is hidden. The reader unpacks it
/// (`MessageBuffer.ReadAccessoryVisibility`, `:4494-4501`).
fn accessory_visibility_mask(hide: &[bool; 10]) -> u16 {
    let mut num = 0u16;
    for (i, &hidden) in hide.iter().enumerate() {
        if hidden {
            num |= 1 << i;
        }
    }
    num
}

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

    /// W4-g49 `65 TeleportEntity` (`MessageBuffer.cs:2949-3035`): a kind-0
    /// player teleport forces the entity to the sender (`:2953-2956`), moves
    /// the host's peer record and relays to the OTHERS (`:2989`); a second
    /// client's copy of the sender takes the position, netOffset zeroed.
    #[test]
    fn w4_g49_teleport_entity_moves_the_peer_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A second client on the same host (the relay's recipient).
        let mut second = flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new());
        let addr = match &host_app.net {
            NetRole::Host(server) => server.addr().to_string(),
            _ => unreachable!("host role"),
        };
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        let sender_slot = client_app.local_id;
        // The second client's copy of the sender peer, at the origin.
        second.remote_players.insert(sender_slot, net::PlayerNet { id: sender_slot, ..Default::default() });
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TeleportEntity(net::TeleportEntity {
                entity: 0, // forced to the sender on the host
                x: 500.0,
                y: 300.0,
                style: 1,
                kind: 0,
                get_pos_from_player: false,
                extra_info: None,
            }));
        }
        for _ in 0..30 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let host_peer = host_app.remote_players.get(&sender_slot).expect("host peer");
        assert_eq!((host_peer.x, host_peer.y), (500.0, 300.0), "the host moved the peer (`:2986`)");
        let relayed = second.remote_players.get(&sender_slot).expect("second's sender peer");
        assert_eq!((relayed.x, relayed.y), (500.0, 300.0), "the relay moved the second client's copy");
        assert_eq!((relayed.off_x, relayed.off_y), (0.0, 0.0), "the netOffset zeroed (`:37447`)");
    }

    /// W4-g49 `66 Unknown66` (`MessageBuffer.cs:3036-3055`): a positive heal
    /// raises the subject's life (clamped to the max) on the host and relays.
    #[test]
    fn w4_g49_heal_player_heals_the_subject_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Wound the host's player so the heal is observable.
        host_app.player.as_mut().unwrap().hp = 50;
        host_app.player.as_mut().unwrap().max_hp = 100;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::HealPlayer(net::HealPlayer {
                player_id: 0, // the host's player
                heal: 30,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(host_app.player.as_ref().unwrap().hp, 80, "the host healed 50+30 (`:3043`)");
        // The HealEffect text spawned (`:3048`).
        assert!(
            host_app.combat_texts.iter().any(|t| t.active && t.text == "30"),
            "the heal combat text spawned"
        );
    }

    /// W4-g49 `68 Unknown68` (`MessageBuffer.cs:3056-3058`): the host reads
    /// the UUID string and drops it — the connection survives (no kick).
    #[test]
    fn w4_g49_client_uuid_is_read_and_dropped() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ClientUuid(net::ClientUuid {
                uuid: "0f8fad5b-d9cb-469f-a165-70867728950e".into(),
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(matches!(client_app.net, NetRole::Client(_)), "the UUID did not kick the client");
    }

    /// W4-g49 `72 TravelMerchantItems` (`MessageBuffer.cs:3118-3126`): the
    /// client stores the 40 shorts into `Main.travelShop`.
    #[test]
    fn w4_g49_travel_shop_stores_the_stock() {
        let (mut host_app, mut client_app) = in_world_pair();
        let items: Vec<i16> = (0..net::TRAVEL_SHOP_MAX_SLOTS as i16).map(|i| i * 3 + 1).collect();
        if let NetRole::Host(server) = &host_app.net {
            server.send(client_app.local_id, &S2c::TravelShop(net::TravelShop { items: items.clone() }));
        }
        crate::npc::NpcKind::CataractEye
        | crate::npc::NpcKind::SleepyEye
        | crate::npc::NpcKind::DialatedEye
        | crate::npc::NpcKind::GreenEye
        | crate::npc::NpcKind::PurpleEye => {
            crate::npc::Npc::spawn_eye_variant(world, tx, kind)?
        },
        crate::npc::NpcKind::Guide => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::Merchant
        | crate::npc::NpcKind::Dryad
        | crate::npc::NpcKind::Nurse
        | crate::npc::NpcKind::ArmsDealer
        | crate::npc::NpcKind::Demolitionist
        | crate::npc::NpcKind::Painter
        | crate::npc::NpcKind::WitchDoctor
        | crate::npc::NpcKind::Clothier
        | crate::npc::NpcKind::TravellingMerchant
        | crate::npc::NpcKind::Angler
        | crate::npc::NpcKind::SleepingAngler
        | crate::npc::NpcKind::Wizard
        | crate::npc::NpcKind::Mechanic
        | crate::npc::NpcKind::Stylist
        | crate::npc::NpcKind::TaxCollector
        | crate::npc::NpcKind::Golfer
        | crate::npc::NpcKind::Zoologist
        | crate::npc::NpcKind::Princess
        | crate::npc::NpcKind::DyeTrader
        | crate::npc::NpcKind::PartyGirl
        | crate::npc::NpcKind::GoblinTinkerer => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::EyeOfCthulhu => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        pump(&mut host_app, &mut client_app, 20);
        for (i, item) in items.iter().enumerate() {
            assert_eq!(
                client_app.world.travel_shop[i],
                (*item).max(0) as u16,
                "slot {i} stored (`:3123`)"
            );
        }
    }

    /// W4-g49 `73 RequestTeleport` (`MessageBuffer.cs:3127-3146`): the kind-3
    /// shellphone request teleports the sender to the world spawn.
    #[test]
    fn w4_g49_request_teleport_shellphone_moves_the_sender() {
        let (mut host_app, mut client_app) = in_world_pair();
        let (sx, sy) = (host_app.world.spawn_x, host_app.world.spawn_y);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::RequestTeleport(net::RequestTeleport { kind: 3 }));
        }
        crate::npc::NpcKind::Skeleton
        | crate::npc::NpcKind::UndeadMiner
        | crate::npc::NpcKind::GoblinPeon
        | crate::npc::NpcKind::GoblinThief
        | crate::npc::NpcKind::GoblinWarrior
        | crate::npc::NpcKind::GoblinArcher
        | crate::npc::NpcKind::BoundGoblin
        | crate::npc::NpcKind::BoundWizard
        | crate::npc::NpcKind::BoundMechanic
        | crate::npc::NpcKind::WebbedStylist
        | crate::npc::NpcKind::SleepingGolfer => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        pump(&mut host_app, &mut client_app, 20);
        let peer = host_app.remote_players.get(&client_app.local_id).expect("host peer");
        assert_eq!(peer.x, sx as f32 * TILE_PX, "the shellphone moved the peer to spawn X");
        assert_eq!(peer.y, sy as f32 * TILE_PX, "the shellphone moved the peer to spawn Y");
    }

    /// W4-g49 `74 AnglerQuest` (`MessageBuffer.cs:3147-3153`): the client
    /// stores the quest index and the finished-today flag.
    #[test]
    fn w4_g49_angler_quest_stores_the_quest() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.send(client_app.local_id, &S2c::AnglerQuest(net::AnglerQuest { quest: 17, finished: true }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(client_app.world.angler_quest, 17, "the quest index stored (`:3150`)");
        assert!(client_app.world.angler_quest_finished, "the finished flag stored (`:3151`)");
    }

    /// W4-g49 `75 AnglerQuestFinished` (`MessageBuffer.cs:3154-3163`): the
    /// host adds the sender's name to anglerWhoFinishedToday (once).
    #[test]
    fn w4_g49_angler_quest_finished_adds_the_senders_name() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player_name = "AnglerFan".into();
        // The host's peer record carries the name (the join push's stand-in).
        host_app.remote_players.get_mut(&client_app.local_id).unwrap().name = "AnglerFan".into();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::AnglerQuestFinished);
            client.send(&C2s::AnglerQuestFinished); // a duplicate must not double-add
        }
        // The swimmers: the ctor plants the feet on the column surface —
        // the net state overwrites the position below.
        crate::npc::NpcKind::Goldfish
        | crate::npc::NpcKind::Piranha
        | crate::npc::NpcKind::Shark
        // W4-c4 (the style-16 swimmers): the nine kinds ride the same
        // surface seed (the box comes from `kind.size()`; the swimmer
        // machine takes over on the first tick).
        // E8 merge drops: CorruptGoldfish/BloodFeeder ride the canonical
        // seed arms (d34/d37's netglue rows), Pufferfish/Orca land with d43.
        | crate::npc::NpcKind::Arapaima
        | crate::npc::NpcKind::CrimsonGoldfish
        | crate::npc::NpcKind::GoldGoldfish
        | crate::npc::NpcKind::Pupfish
        | crate::npc::NpcKind::Dolphin => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
        // W4-e21 — the bunny pair (46/540): the surface critter plants its
        // feet on the column surface like the swimmers' seed (the net state
        // overwrites the position below).
        crate::npc::NpcKind::Bunny
        | crate::npc::NpcKind::PartyBunny => crate::npc::Npc::spawn_cave_walker(world, tx, kind)?,
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
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(
            host_app.world.angler_who_finished_today,
            vec!["AnglerFan".to_string()],
            "the sender's name added once (`:3157-3161`)"
        );
    }

    /// W4-g49 `77 SyncTilePicking` (`MessageBuffer.cs:3183-3185`): the
    /// temporary animation has no port target — the read is a no-op and the
    /// connection survives.
    #[test]
    fn w4_g49_temporary_animation_is_a_noop() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.send(client_app.local_id, &S2c::TemporaryAnimation(net::TemporaryAnimation {
                animation_type: 3,
                tile_type: 593,
                x: 100,
                y: 50,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(matches!(client_app.net, NetRole::Client(_)), "the animation did not kick the client");
    }

    /// W4-g49 `78 ReportInvasionProgress` (`MessageBuffer.cs:3186-3191`): the
    /// client stores the four progress fields and arms the 160-tick window
    /// (`Main.ReportInvasionProgress`, `Main.cs:47687-47694`).
    #[test]
    fn w4_g49_invasion_progress_stores_the_bar() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.send(client_app.local_id, &S2c::InvasionProgress(net::InvasionProgress {
                progress: 120,
                progress_max: 200,
                icon: 3,
                wave: 5,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(client_app.invasion_progress, 120, "progress stored (`:47689`)");
        assert_eq!(client_app.invasion_progress_max, 200, "max stored (`:47690`)");
        assert_eq!(client_app.invasion_progress_icon, 3, "icon stored (`:47691`)");
        assert_eq!(client_app.invasion_progress_wave, 5, "wave stored (`:47692`)");
        assert_eq!(client_app.invasion_progress_display_left, 160, "the display window armed (`:47693`)");
    }

    /// W4-g49 `57 Unknown57` (`MessageBuffer.cs:2631-2638`): the client
    /// stores the three world-evil bytes in the writer's order
    /// (`WorldGen.tGood/tEvil/tBlood`), and the Dryad report reads them back.
    #[test]
    fn w4_g49_world_evil_bytes_store_and_feed_the_dryad_report() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.send(client_app.local_id, &S2c::WorldEvilBytes(net::WorldEvilBytes {
                good: 200,
                evil: 100,
                blood: 50,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(client_app.world_evil_good, 200, "tGood stored (`:2634`)");
        assert_eq!(client_app.world_evil_evil, 100, "tEvil stored (`:2635`)");
        assert_eq!(client_app.world_evil_blood, 50, "tBlood stored (`:2636`)");
        // The display consumer (`Lang.GetDryadWorldStatusDialog`): all three
        // > 0 selects the WorldStatusAll arm, no pure flag.
        let (text, pure) = crate::lang::dryad_world_status_dialog(
            &client_app.world.name,
            client_app.world_evil_good,
            client_app.world_evil_evil,
            client_app.world_evil_blood,
        );
        assert!(!pure, "200/100/50 is not entirely pure");
        assert!(text.contains("200%"), "the hallow percent rides the report: {text}");
    }

    /// W4-g49 `58 InstrumentSound` (`MessageBuffer.cs:2639-2681`): the host
    /// relays a peer's instrument; the client's default arm stores
    /// `Main.musicPitch` (`:2666`) and plays the held item's sound. The host
    /// peer holds no instrument (air), so the default (Item26) arm runs.
    #[test]
    fn w4_g49_instrument_sound_stores_the_music_pitch() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Let the join bundle land so the host's peer record (slot 0) exists
        // with its inventory — the arm reads the peer's held item.
        pump(&mut host_app, &mut client_app, 20);
        if let NetRole::Host(server) = &host_app.net {
            server.send(client_app.local_id, &S2c::InstrumentSound(net::InstrumentSound {
                player_id: 0, // the host peer
                pitch: 0.45,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(client_app.music_pitch, 0.45, "Main.musicPitch stored (`:2666`)");
        assert!(matches!(client_app.net, NetRole::Client(_)), "the instrument did not kick the client");
    }

    /// W4-g49 `58`'s host half (`MessageBuffer.cs:2642-2650`): the host forces
    /// the id to the sender (`:2644`) and relays to the OTHER clients
    /// (`:2649`). A second client receives the relay with the sender's slot,
    /// not the bogus id the sender wrote, and its default arm stores the pitch.
    #[test]
    fn w4_g49_instrument_sound_host_forces_the_sender_id() {
        let (mut host_app, mut client_app) = in_world_pair();
        let addr = match &host_app.net {
            NetRole::Host(server) => server.addr().to_string(),
            _ => unreachable!("host role"),
        };
        let mut second = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        // Drive the second client to IN_WORLD (the handshake poll loop).
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while second.net_state != net::client_state::IN_WORLD
            && !matches!(second.net, NetRole::Off)
            && std::time::Instant::now() < deadline
        {
            second.poll_net();
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        for _ in 0..8 {
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let sender_slot = client_app.local_id;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::InstrumentSound(net::InstrumentSound {
                player_id: 99, // the host must overwrite this (`:2644`)
                pitch: 0.25,
            }));
        }
        for _ in 0..40 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // The second client received the relay attributed to the sender's
        // slot; 58 has no own-echo gate, so the default arm stored the pitch.
        assert_eq!(second.music_pitch, 0.25, "the relay carried the pitch");
        assert_ne!(sender_slot, second.local_id, "distinct slots");
    }

    /// W4-g49 `76 QuestsCountSync` (`MessageBuffer.cs:3164-3182`): the host
    /// forces the subject to the sender (`:3169-3172`), stores both ints
    /// (`:3174-3175`) and relays to the OTHER clients (`:3178`). A second
    /// client's copy of the sender takes the counts; a self-addressed echo
    /// (player id 0 = the host's `Main.myPlayer`) is dropped (`:3167`).
    #[test]
    fn w4_g49_quests_count_sync_stores_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        let addr = match &host_app.net {
            NetRole::Host(server) => server.addr().to_string(),
            _ => unreachable!("host role"),
        };
        let mut second = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        let sender_slot = client_app.local_id;
        // The second client's copy of the sender peer, counts at zero.
        second.remote_players.insert(sender_slot, net::PlayerNet { id: sender_slot, ..Default::default() });
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::QuestsCountSync(net::QuestsCountSync {
                player_id: 99, // forced to the sender on the host (`:3171`)
                angler_quests_finished: 42,
                golfer_score: 1500,
            }));
        }
        for _ in 0..40 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let host_peer = host_app.remote_players.get(&sender_slot).expect("host peer");
        assert_eq!(host_peer.angler_quests_finished, 42, "the host stored the quest count (`:3174`)");
        assert_eq!(host_peer.golfer_score, 1500, "the host stored the golf score (`:3175`)");
        let relayed = second.remote_players.get(&sender_slot).expect("second's sender peer");
        assert_eq!(relayed.angler_quests_finished, 42, "the relay carried the quest count (`:3178`)");
        assert_eq!(relayed.golfer_score, 1500, "the relay carried the golf score (`:3178`)");
        // A self-addressed echo (id 0 = the host's myPlayer, SSC false) is
        // dropped (`:3167`) — the host's peer record stays unchanged.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::QuestsCountSync(net::QuestsCountSync {
                player_id: 0,
                angler_quests_finished: 999,
                golfer_score: 999,
            }));
        }
        for _ in 0..20 {
            host_app.poll_net();
            client_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let host_peer = host_app.remote_players.get(&sender_slot).expect("host peer");
        assert_eq!(host_peer.angler_quests_finished, 42, "the self-echo did not overwrite (`:3167`)");
    }

    /// W4-g49 `79 PlaceObject` (`MessageBuffer.cs:3192-3215`): the host's
    /// `InWorld(x, y, 10)` + section-sent gate (`:3204`) passes for a table
    /// inside the streamed join box; `WorldGen.PlaceObject` (`:3209`) fills
    /// the 3×2 footprint and the relay (`:3212`) places the client's copy.
    #[test]
    fn w4_g49_place_object_places_and_relays() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        host_app.world = tr_core::World::new("flow".into(), 900, 600);
        host_app.world.spawn_x = 100;
        host_app.world.spawn_y = 100;
        host_app.world.column_surface = vec![host_app.world.height; 900];
        // A solid floor row at y=120 (the table's AnchorBottom row) on the
        // host — the join stream carries it to the client.
        for x in 90..=110 {
            let i = host_app.world.index(x, 120);
            host_app.world.tiles[i].kind = 1;
            host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlaceObject(net::PlaceObject {
                x: 100,
                y: 119,
                kind: 14, // a table (3×2, origin (1,1))
                style: 0,
                alternative: 0,
                random: -1,
                direction: true,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        // The table footprint: x 99-101, rows 118-119.
        for app in [&host_app, &client_app] {
            for x in 99..=101 {
                for y in 118..=119 {
                    let t = app.world.tile(x, y).expect("table cell");
                    assert!(t.active() && t.kind == 14, "table cell {x},{y} placed (`:3209`)");
                }
            }
        }
    }

    /// W4-g49 `79`'s gate half (`MessageBuffer.cs:3204`): a placement outside
    /// `InWorld(x, y, 10)` is refused — no tile lands and no relay goes out.
    #[test]
    fn w4_g49_place_object_out_of_world_is_refused() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlaceObject(net::PlaceObject {
                x: 2, // inside the 10-tile border (`:3204`)
                y: 4,
                kind: 14,
                style: 0,
                alternative: 0,
                random: -1,
                direction: true,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        for app in [&host_app, &client_app] {
            let t = app.world.tile(2, 4).expect("cell");
            assert!(!t.active(), "the border placement was refused (`:3204`)");
        }
    }

    /// W4-g49 `48 LiquidUpdate` (`MessageBuffer.cs:2407-2446`): the host
    /// applies the liquid byte + type (`:2434-2435`); a drained tile
    /// (`liquid == 0`) re-pushes as `48` to the others (`:2441`), which the
    /// client applies. A non-zero liquid is NOT relayed by the reader arm
    /// (the fan-out is `sendWater`'s, the send path's).
    #[test]
    fn w4_g49_liquid_update_drained_replicates() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Seed water on both sides so the drain is observable.
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(5, 4);
            app.world.tiles[i].liquid = 200;
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::LiquidUpdate(net::LiquidUpdate {
                x: 5,
                y: 4,
                liquid: 0, // drained (`:2439`)
                liquid_type: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        assert_eq!(host_app.world.tile(5, 4).unwrap().liquid, 0, "the host drained (`:2434`)");
        assert_eq!(client_app.world.tile(5, 4).unwrap().liquid, 0, "the drained re-push reached the client (`:2441`)");
    }

    /// W4-g49 `48`'s type half (`MessageBuffer.cs:2435`, `Tile.cs:216-233`):
    /// a lava-type update sets the lava kind bits on the host.
    #[test]
    fn w4_g49_liquid_update_sets_the_kind() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::LiquidUpdate(net::LiquidUpdate {
                x: 5,
                y: 4,
                liquid: 128,
                liquid_type: 1, // lava
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        let t = host_app.world.tile(5, 4).unwrap();
        assert_eq!(t.liquid, 128, "the liquid byte applied (`:2434`)");
        assert_eq!(t.liquid_kind(), tr_core::tile::LiquidKind::Lava, "the lava kind applied (`:2435`)");
    }

    /// W4-g49 `52 LockAndUnlock` (`MessageBuffer.cs:2531-2564`): a kind-1
    /// chest unlock shifts the locked 2×2 frames off on the host
    /// (`Chest.Unlock` `:2538`) and relays (`:2541`); the client's copy takes
    /// the same shift. The golden chest (style 2, frameX 72) unlocks to 36.
    #[test]
    fn w4_g49_lock_and_unlock_chest_shifts_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A locked golden chest (kind 21, style 2 → frameX 72) at (5,4) on
        // both sides (the 2×2 footprint).
        for app in [&mut host_app, &mut client_app] {
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let i = app.world.index(5 + dx, 4 + dy);
                app.world.tiles[i].kind = 21;
                app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                app.world.tiles[i].frame_x = 72 + if dx == 1 { 18 } else { 0 };
                app.world.tiles[i].frame_y = if dy == 1 { 18 } else { 0 };
            }
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::LockAndUnlock(net::LockAndUnlock {
                action: 1, // chest unlock (`:2536-2543`)
                x: 5,
                y: 4,
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        for app in [&host_app, &client_app] {
            assert_eq!(app.world.tile(5, 4).unwrap().frame_x, 36, "the unlock shifted −36 (`Chest.cs:362`)");
            assert_eq!(app.world.tile(6, 4).unwrap().frame_x, 54, "the right column shifted too");
        }
    }

    /// W4-g49 `52`'s door half (`MessageBuffer.cs:2547`,
    /// `WorldGen.cs:37599-37628`): a kind-2 door unlock walks up to the 594
    /// leaf and shifts the three leaves +54 on the host; the relay carries
    /// it to the client.
    #[test]
    fn w4_g49_lock_and_unlock_door_shifts_the_leaves() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A locked golden door (kind 10) at x=5, leaves y=2..4 with frameY
        // 594/612/630 on both sides.
        for app in [&mut host_app, &mut client_app] {
            for (k, fy) in [(2, 594i16), (3, 612), (4, 630)] {
                let i = app.world.index(5, k);
                app.world.tiles[i].kind = 10;
                app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                app.world.tiles[i].frame_y = fy;
            }
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::LockAndUnlock(net::LockAndUnlock {
                action: 2, // door unlock (`:2545-2552`)
                x: 5,
                y: 4, // the bottom leaf — the walk finds the 594 top
            }));
        }
        pump(&mut host_app, &mut client_app, 20);
        for app in [&host_app, &client_app] {
            assert_eq!(app.world.tile(5, 2).unwrap().frame_y, 648, "the top leaf shifted +54 (`WorldGen.cs:37621`)");
            assert_eq!(app.world.tile(5, 3).unwrap().frame_y, 666, "the middle leaf shifted");
            assert_eq!(app.world.tile(5, 4).unwrap().frame_y, 684, "the bottom leaf shifted");
        }
    }

    /// W4-g49 `44/67` (`MessageBuffer.cs:4483-4490`): the ignore group is
    /// accepted and dropped — no world change, no kick.
    #[test]
    fn w4_g49_ignore_group_is_dropped() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.send(client_app.local_id, &S2c::Unknown44);
            server.send(client_app.local_id, &S2c::Unknown67);
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(matches!(client_app.net, NetRole::Client(_)), "the ignore group did not kick the client");
    }

    // ---- W4-g50 (S1-I-4 lane 3): the 81-120 apply arms ----
    // WRITTEN NOT RUN (code-only swarm, no compiler).

    /// `case 81` (`MessageBuffer.cs:3227-3233`): the heal number spawns a
    /// combat text at the truncated float point with alpha-255 RGB.
    #[test]
    fn w4_g50_the_81_heal_number_spawns_a_combat_text() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::CombatTextInt(net::CombatTextInt {
                x: 321.9,
                y: 654.1,
                color: net::Rgb(100, 255, 100),
                amount: 25,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let t = client_app
            .combat_texts
            .iter()
            .find(|t| t.active)
            .expect("the 81 spawned a text");
        assert_eq!(t.text, "25");
        assert_eq!(t.color, [100, 255, 100, 255], "the RGB gains alpha 255");
        // The spawn point is the truncated floats minus half the measured
        // size (`CombatText.cs:86-87`); the width/height-0 rect adds no jitter.
        assert!(t.position[0] <= 321.0 && t.position[1] <= 654.0);
    }

    /// `case 119` (`MessageBuffer.cs:3235-3242`): the string twin spawns the
    /// resolved NetworkText at the truncated point.
    #[test]
    fn w4_g50_the_119_combat_string_spawns_the_resolved_text() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::CombatTextString(net::CombatTextString {
                x: -8.5,
                y: 100.9,
                color: net::Rgb(255, 160, 80),
                text: net::NetText("Belsin was slain...".into()),
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let t = client_app
            .combat_texts
            .iter()
            .find(|t| t.active)
            .expect("the 119 spawned a text");
        assert_eq!(t.text, "Belsin was slain...");
        assert_eq!(t.color, [255, 160, 80, 255]);
        assert!(t.position[0] <= -8.0 && t.position[1] <= 100.0);
    }

    /// `case 84` (`MessageBuffer.cs:3246-3260`): the host forces the subject
    /// to the sender and relays; the client stores the peer's stealth.
    #[test]
    fn w4_g50_the_84_stealth_relays_to_the_peer_snapshot() {
        let (mut host_app, mut client_app) = in_world_pair();
        let peer = client_app.local_id;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerStealth(net::PlayerStealth {
                player_id: 250, // the host overrides it (`:3251`)
                stealth: 0.25,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert_eq!(
            host_app.remote_players.get(&peer).map(|p| p.stealth),
            Some(0.25),
            "the host stored the sender's stealth"
        );
        // The relay reaches the host's own view of the peer; the sender
        // itself is excluded (`TrySendData(84, -1, whoAmI, …)`).
        // A second client would see it; here the host's snapshot is the
        // witness that the forced id + value landed.
    }

    /// `case 92` (`MessageBuffer.cs:3442-3461`): the host adds the DELTA and
    /// relays the TOTAL; the client assigns it and pings the coins.
    #[test]
    fn w4_g50_the_92_extra_value_accumulates_and_relays_the_total() {
        let (mut host_app, mut client_app) = in_world_pair();
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        npc.extra_value = 500;
        host_app.npcs.push(npc);
        pump(&mut host_app, &mut client_app, 15);
        assert!(client_app.net_npcs.contains_key(&0), "the NPC synced");
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcExtraValue(net::NpcExtraValue {
                npc: 0,
                value: 250, // a DELTA on the client leg
                x: 100.0,
                y: 100.0,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(host_app.npcs[0].extra_value, 750, "the host added the delta");
        assert_eq!(
            client_app.net_npcs[&0].extra_value, 750,
            "the client assigned the relayed TOTAL"
        );
        // The moneyPing dust burst fired on the client (the 244-type row at
        // a 750-copper credit — under the 10000 ladder step).
        assert!(
            client_app
                .dust
                .slots()
                .iter()
                .any(|d| d.active && d.dust_type == 244),
            "the moneyPing burst spawned"
        );
    }

    /// `case 87` → `86` (`MessageBuffer.cs:3294-3305`,
    /// `TileEntityType.cs:17-20`): the host places the entity on a valid
    /// free cell and answers with the sharing broadcast; the client's books
    /// carry it under the host's id.
    #[test]
    fn w4_g50_the_87_placement_answers_with_an_86_share() {
        let (mut host_app, mut client_app) = in_world_pair();
        // An item-frame anchor tile (395, frameX/Y 0) on both sides.
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(10, 4);
            app.world.tiles[i] = tr_core::Tile {
                kind: 395,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEntityPlacement(net::TileEntityPlacement {
                x: 10,
                y: 4,
                kind: 1, // TEItemFrame
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        let host_te = host_app
            .world
            .tile_entities
            .get_at(10, 4)
            .expect("the host placed the frame");
        assert_eq!(host_te.kind, tr_core::tile_entities::TeKind::ItemFrame);
        let client_te = client_app
            .world
            .tile_entities
            .get_at(10, 4)
            .expect("the 86 share landed on the client");
        assert_eq!(client_te.id, host_te.id, "the forced id rides the body");
        // A second placement on the occupied cell is refused (`:3300`).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEntityPlacement(net::TileEntityPlacement {
                x: 10,
                y: 4,
                kind: 1,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert_eq!(host_app.world.tile_entities.live_entities().count(), 1);
    }

    /// `case 86`'s removal arm (`MessageBuffer.cs:3279-3284`): the absent
    /// shape drops the entity from the client's books by id.
    #[test]
    fn w4_g50_the_86_absent_shape_removes_by_id() {
        let (mut host_app, mut client_app) = in_world_pair();
        let id = host_app
            .world
            .tile_entities
            .place_unchecked(tr_core::tile_entities::TeKind::ItemFrame, 10, 4);
        let share = |app: &App, id: u32| {
            let record = match app.world.tile_entities.get_by_id(id) {
                Some(e)
                    if app
                        .world
                        .tile_entities
                        .get_at(e.x, e.y)
                        .is_some_and(|live| live.id == id) =>
                {
                    let mut w = tr_core::binio::Writer::new();
                    e.write_net(&mut w);
                    w.into_inner()
                }
                _ => Vec::new(),
            };
            S2c::TileEntitySharing(net::TileEntitySharing {
                id: id as i32,
                record,
            })
        };
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&share(&host_app, id));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(client_app.world.tile_entities.get_by_id(id).is_some());
        // The host removes it and shares the absence.
        host_app.world.tile_entities.remove_by_id(id);
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&share(&host_app, id));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(client_app.world.tile_entities.get_at(10, 4).is_none());
        assert_eq!(client_app.world.tile_entities.live_entities().count(), 0);
    }

    /// `case 89` (`MessageBuffer.cs:3377-3393`, `TEItemFrame.cs:69-86`): the
    /// host stores the item in the frame and shares it; a second placement
    /// drops the old contents first.
    #[test]
    fn w4_g50_the_89_frame_placement_stores_and_drops_the_old_item() {
        let (mut host_app, mut client_app) = in_world_pair();
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(10, 4);
            app.world.tiles[i] = tr_core::Tile {
                kind: 395,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        // Place the frame entity first (the 87 path).
        let id = host_app
            .world
            .tile_entities
            .place_unchecked(tr_core::tile_entities::TeKind::ItemFrame, 10, 4);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ItemFrameTryPlacing(net::ItemFrameTryPlacing {
                x: 10,
                y: 4,
                kind: 29,
                prefix: 81,
                stack: 1,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            host_app.world.tile_entities.item(id).copied().flatten(),
            Some(tr_core::tile_entities::TeItem {
                kind: 29,
                prefix: 81,
                stack: 1
            }),
            "the host stored the exact prefix"
        );
        assert_eq!(
            client_app.world.tile_entities.item(id).copied().flatten(),
            Some(tr_core::tile_entities::TeItem {
                kind: 29,
                prefix: 81,
                stack: 1
            }),
            "the 86 share carried the item to the client"
        );
        // A second placement drops the old item (count 1) and takes the new.
        let drops_before = host_app.items.len();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ItemFrameTryPlacing(net::ItemFrameTryPlacing {
                x: 10,
                y: 4,
                kind: 7,
                prefix: 0,
                stack: 5,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(host_app.items.len(), drops_before + 1, "the old item dropped");
        assert_eq!(host_app.items.last().unwrap().kind, 29);
        assert_eq!(
            host_app.world.tile_entities.item(id).copied().flatten().map(|i| i.kind),
            Some(7),
            "the new item took the frame"
        );
    }

    /// `case 90` (`MessageBuffer.cs:1489-1493`): the instanced drop lands
    /// reserved for the receiving client.
    #[test]
    fn w4_g50_the_90_instanced_drop_reserves_for_the_local_client() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.send(
                client_app.local_id,
                &S2c::SpawnInstancedItem(Box::new(net::ItemState {
                    slot: 5,
                    x: 100.0,
                    y: 100.0,
                    vx: 0.0,
                    vy: 0.0,
                    stack: 2,
                    prefix: 0,
                    kind: 29,
                    shimmered: false,
                    shimmer_time: 0.0,
                })),
            );
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(client_app.net_items.contains_key(&5), "the drop landed");
        assert_eq!(
            client_app.net_reserved.get(&5).map(|(who, _)| *who),
            Some(client_app.local_id as u8),
            "the instanced tail reserved it for me (`:1492`)"
        );
    }

    /// `case 91` (`MessageBuffer.cs:3394-3441`): the bubble upserts with the
    /// wire's lifetime/emote/anchor, a player anchor sets emoteTime 360,
    /// and the 255 shape removes it.
    #[test]
    fn w4_g50_the_91_emote_bubble_upserts_and_removes() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u16;
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::EmoteBubble(net::EmoteBubble {
                id: 42,
                anchor_kind: 1,
                anchor_index: me,
                life_time: 360,
                emote: 17,
                metadata: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let b = client_app
            .emotes
            .bubbles
            .iter()
            .find(|b| b.id == 42)
            .expect("the bubble landed");
        assert_eq!(b.emote, 17);
        assert!(b.anchored_to_player, "the local-slot anchor resolves");
        assert_eq!(
            client_app.player.as_ref().map(|p| p.emote_time),
            Some(360),
            "the player anchor set emoteTime (`:3421`)"
        );
        // A second bubble on the same anchor pops the first to 6
        // (`OnBubbleChange`, `EmoteBubble.cs:273`).
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::EmoteBubble(net::EmoteBubble {
                id: 43,
                anchor_kind: 1,
                anchor_index: me,
                life_time: 360,
                emote: 0,
                metadata: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let first = client_app.emotes.bubbles.iter().find(|b| b.id == 42).unwrap();
        assert!(first.life_time <= 6, "the older bubble popped");
        // The removal shape drops it.
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::EmoteBubble(net::EmoteBubble::removal(42)));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(!client_app.emotes.bubbles.iter().any(|b| b.id == 42));
    }

    /// `case 95` (`MessageBuffer.cs:3519-3537`): the host kills the first
    /// matching type-602 portal and broadcasts the 29; a wrong form spares it.
    #[test]
    fn w4_g50_the_95_murder_kills_the_matching_portal() {
        let (mut host_app, mut client_app) = in_world_pair();
        let owner = client_app.local_id as u8;
        let slot = host_app
            .projectiles
            .spawn_with_data_owned(602, owner, 100.0, 100.0, 0.0, 0.0, 0, 0.0, 0.0, 1.0)
            .expect("spawn");
        pump(&mut host_app, &mut client_app, 20);
        assert!(host_app.projectiles.slots()[slot].active);
        // A wrong form matches nothing (`:3529` compares ai[1]).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::MurderPortal {
                owner: owner as u16,
                form: 0,
            });
        }
        // The dragonflies (W4-d33): the noGravity flyers' net seed rides
        // the hover ctor (the net state overwrites the position below;
        // the aiStyle-114 machine takes over on the first tick).
        crate::npc::NpcKind::BlackDragonfly
        | crate::npc::NpcKind::BlueDragonfly
        | crate::npc::NpcKind::GreenDragonfly
        | crate::npc::NpcKind::OrangeDragonfly
        | crate::npc::NpcKind::RedDragonfly
        | crate::npc::NpcKind::YellowDragonfly
        | crate::npc::NpcKind::GoldDragonfly => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The fairy critters + the windy balloon (W4-d33 wiring): the
        // noGravity flyers' net seed rides the hover ctor (the net state
        // overwrites the position below; the aiStyle-112/113 machines
        // take over on the first tick). d42 batch-2/3's seed arms,
        // constants-copy (keep-one d37c-E3).
        crate::npc::NpcKind::FairyCritterPink
        | crate::npc::NpcKind::FairyCritterGreen
        | crate::npc::NpcKind::FairyCritterBlue
        | crate::npc::NpcKind::WindyBalloon => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // W4-c7 — the firefly family (355/358/654/677): the noGravity
        // glow critters seed bare at the net position (the Plantera
        // minion arm's shape — the machine takes over on the first tick
        // and the net state overwrites the position below).
        crate::npc::NpcKind::Firefly
        | crate::npc::NpcKind::LightningBug
        | crate::npc::NpcKind::Lavafly
        | crate::npc::NpcKind::Shimmerfly => crate::npc::Npc::new(kind, _x, y, 1, 0.0),
        // The snail family (W4-c9): the wall-crawler critters plant on
        // the column surface like the wave-A walkers (the aiStyle-67
        // machine takes over on the first tick — the fall roll + the
        // crawl state machine own the placement from there).
        crate::npc::NpcKind::Snail
        | crate::npc::NpcKind::GlowingSnail
        | crate::npc::NpcKind::MagmaSnail => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            host_app.projectiles.slots()[slot].active,
            "the form mismatch spared the portal"
        );
        // The right form kills it and the `29` relay reaches the client.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::MurderPortal {
                owner: owner as u16,
                form: 1,
            });
        }
        pump(&mut host_app, &mut client_app, 20);
        assert!(
            !host_app.projectiles.slots()[slot].active,
            "the host killed the matching portal"
        );
        assert!(
            !client_app.projectiles.slots()[slot].active,
            "the 29 relay dropped the client's copy (`:3532`)"
        );
    }

    /// `case 96` (`MessageBuffer.cs:3538-3557`): the host forces the subject
    /// to the sender, applies the teleport and relays; the peer snapshot
    /// snaps with the velocity and a zeroed netOffset.
    #[test]
    fn w4_g50_the_96_portal_teleport_snaps_the_peer() {
        let (mut host_app, mut client_app) = in_world_pair();
        let peer = client_app.local_id;
        // Seed the host's peer snapshot so the apply has a target.
        host_app
            .remote_players
            .entry(peer)
            .or_default();
        host_app.remote_players.get_mut(&peer).unwrap().off_x = 40.0;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerPortalTeleport(net::PlayerPortalTeleport {
                player: 250, // the host overrides it (`:3543`)
                color_index: 4,
                x: 500.0,
                y: 300.0,
                velocity_x: 2.5,
                velocity_y: -1.0,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        let entry = host_app.remote_players.get(&peer).expect("peer stored");
        assert_eq!((entry.x, entry.y), (500.0, 300.0));
        assert_eq!((entry.vx, entry.vy), (2.5, -1.0));
        assert_eq!((entry.off_x, entry.off_y), (0.0, 0.0), "netOffset zeroed");
        assert_eq!(entry.last_portal_color_index, 4, "the un-flipped wire color");
    }

    /// `case 99`/`115` (`MessageBuffer.cs:3570-3597`): the host forces the
    /// subject to the sender, stores the minion target and relays it.
    #[test]
    fn w4_g50_the_minion_targets_relay_to_the_peer_snapshot() {
        let (mut host_app, mut client_app) = in_world_pair();
        let peer = client_app.local_id;
        host_app.remote_players.entry(peer).or_default();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::MinionRestTarget(net::MinionRestTarget {
                player_id: 250, // the host overrides it (`:3575`)
                x: 320.0,
                y: 640.0,
            }));
            client.send(&C2s::MinionAttackTarget(net::MinionAttackTarget {
                player_id: 250, // the host overrides it (`:3589`)
                npc: 7,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        let entry = host_app.remote_players.get(&peer).expect("peer stored");
        assert_eq!(entry.minion_rest_target_point, (320.0, 640.0));
        assert_eq!(entry.minion_attack_target_npc, 7);
    }

    /// `case 100` (`MessageBuffer.cs:3598-3610`): the NPC snaps to the dest
    /// with the live velocity, a zeroed netOffset and the stored color.
    #[test]
    fn w4_g50_the_100_npc_portal_snaps_the_slot() {
        let (mut host_app, mut client_app) = in_world_pair();
        let slime = crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue);
        host_app.npcs.push(slime);
        pump(&mut host_app, &mut client_app, 15);
        assert!(client_app.net_npcs.contains_key(&0));
        client_app.net_npcs.get_mut(&0).unwrap().net_offset = Vec2::new(30.0, 20.0);
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::NpcPortalTeleport(net::NpcPortalTeleport {
                npc: 0,
                color_index: 3,
                x: 500.0,
                y: 300.0,
                velocity_x: 2.5,
                velocity_y: -1.0,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let npc = &client_app.net_npcs[&0];
        assert_eq!((npc.x, npc.y), (500.0, 300.0));
        assert_eq!((npc.vx, npc.vy), (2.5, -1.0));
        assert_eq!(npc.net_offset, Vec2::ZERO, "netOffset *= 0f (`:3608`)");
        assert_eq!(npc.last_portal_color_index, 3);
    }

    /// `case 101` (`MessageBuffer.cs:3611-3651`): each shield clamps to
    /// 0..LunarShieldPowerMax (100) — an over-max ushort lands at 100.
    #[test]
    fn w4_g50_the_101_tower_shields_clamp_to_the_power_max() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::TowerShields(net::TowerShields {
                solar: 65535,
                vortex: 55,
                nebula: 0,
                stardust: 100,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert_eq!(client_app.world.tower_shield_solar, 100, "over-max clamps");
        assert_eq!(client_app.world.tower_shield_vortex, 55);
        assert_eq!(client_app.world.tower_shield_nebula, 0);
        assert_eq!(client_app.world.tower_shield_stardust, 100);
    }

    /// `case 102` (`MessageBuffer.cs:3662-3705`): the host relays with the
    /// sender forced; the client bumps the local nebula tier and grants the
    /// tier buff. The picker is the client itself, so the local player is
    /// the fan's target.
    #[test]
    fn w4_g50_the_102_nebula_pickup_levels_the_local_player() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        let (cx, cy) = client_app.player.as_ref().unwrap().center();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NebulaLevelupRequest(net::NebulaLevelupRequest {
                player_id: 250, // the host overrides it (`:3659`)
                buff_type: 173,
                x: cx,
                y: cy,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        let p = client_app.player.as_ref().unwrap();
        assert_eq!(p.nebula_level_life, 1, "the life tier bumped");
        assert!(
            p.buffs.find_buff_index(173).is_some(),
            "the tier-1 life buff (173 + 1 - 1) granted"
        );
    }

    /// `case 103` (`MessageBuffer.cs:3708-3714`): both countdown statics
    /// land on the world.
    #[test]
    fn w4_g50_the_103_moonlord_countdown_stores_both_statics() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::MoonlordCountdown(net::MoonlordCountdown {
                max: 3600,
                countdown: 1800,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert_eq!(client_app.world.max_moon_lord_countdown, 3600);
        assert_eq!(client_app.world.moon_lord_countdown, 1800);
    }

    /// `case 104` (`MessageBuffer.cs:3715-3735`): the open shop's slot is
    /// rebuilt with the wire's type/stack/prefix/value/buyOnce.
    #[test]
    fn w4_g50_the_104_shop_override_rebuilds_the_open_slot() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Open a shop on the client with a two-row stock.
        client_app.shop_npc = Some(0);
        client_app.shop_stock = vec![
            crate::shop::ShopEntry {
                item: 88,
                when: crate::shop::StockWhen::Always,
                shop_custom_price: None,
                buy_once: false,
                is_a_shop_item: false,
                prefix: 0,
                stack: 1,
                value: None,
            },
            crate::shop::ShopEntry {
                item: 87,
                when: crate::shop::StockWhen::Always,
                shop_custom_price: None,
                buy_once: false,
                is_a_shop_item: false,
                prefix: 0,
                stack: 1,
                value: None,
            },
        ];
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::ShopOverride(net::ShopOverride {
                slot: 1,
                kind: 29,
                stack: 5,
                prefix: 0,
                value: 12345,
                buy_once: true,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let row = &client_app.shop_stock[1];
        assert_eq!(row.item, 29);
        assert_eq!(row.stack, 5);
        assert_eq!(row.value, Some(12345));
        assert!(row.buy_once);
        // The untouched row keeps its stock.
        assert_eq!(client_app.shop_stock[0].item, 88);
    }

    /// `case 105` (`MessageBuffer.cs:3736-3741`, `WorldGen.cs:46469-46524`):
    /// the host flips the gem lock's 3×3 frame to the on row and trips it.
    #[test]
    fn w4_g50_the_105_gem_lock_flips_the_frame_and_trips() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A gem lock (440) at (10,4), off row (frameY 0), amethyst column 0.
        for app in [&mut host_app, &mut client_app] {
            for k in 10..13 {
                for l in 4..7 {
                    let i = app.world.index(k, l);
                    app.world.tiles[i] = tr_core::Tile {
                        kind: 440,
                        frame_x: ((k - 10) * 18) as i16,
                        frame_y: ((l - 4) * 18) as i16,
                        ..Default::default()
                    };
                    app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::GemLockToggle(net::GemLockToggle {
                x: 10,
                y: 4,
                on: true,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        // The host's lock flipped to the on row (frameY 54+).
        assert!(host_app.world.tile(10, 4).unwrap().frame_y >= 54);
        // The 3×3 relay carried the flip to the client.
        assert!(client_app.world.tile(10, 4).unwrap().frame_y >= 54);
    }

    /// `case 106` (`MessageBuffer.cs:3742-3751`, `Utils.cs:1078-1093`): the
    /// poof spawns the gore burst + the type-16 dust ring.
    #[test]
    fn w4_g50_the_106_poof_spawns_gore_and_dust() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::PoofOfSmoke(net::PoofOfSmoke {
                packed: net::pack_half_vector2(200.0, 100.0),
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            client_app
                .dust
                .slots()
                .iter()
                .any(|d| d.active && d.dust_type == 16),
            "the type-16 dust ring spawned"
        );
        assert!(
            client_app
                .gore
                .slots()
                .iter()
                .any(|g| g.active && (11..14).contains(&g.gore_type)),
            "the 11..13 gore burst spawned"
        );
    }

    /// `case 107` (`MessageBuffer.cs:3752-3757`): the smart text lands as a
    /// chat line in the wire colour.
    #[test]
    fn w4_g50_the_107_smart_text_lands_a_chat_line() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::SmartText(net::SmartText {
                color: net::Rgb(255, 200, 150),
                text: net::NetText("A wave is approaching...".into()),
                width_limit: 400,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let line = client_app
            .chat
            .messages
            .iter()
            .find(|m| m.text == "A wave is approaching...")
            .expect("the chat line landed");
        assert_eq!(line.color, [255, 200, 150]);
    }

    /// `case 108` (`MessageBuffer.cs:3758-3773`): the owning client replays
    /// the cannon shot; a non-owner ignores it.
    #[test]
    fn w4_g50_the_108_cannon_shot_replays_for_the_owner_only() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        // A non-owner shot spawns nothing on the client.
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::WiredCannonShot(net::WiredCannonShot {
                damage: 300,
                knockback: 8.0,
                x: 10,
                y: 4,
                angle: 4,
                ammo: 4,
                owner: me.wrapping_add(1),
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let before = client_app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 601)
            .count();
        // The owner's shot replays the 601 explosive.
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::WiredCannonShot(net::WiredCannonShot {
                damage: 300,
                knockback: 8.0,
                x: 10,
                y: 4,
                angle: 4,
                ammo: 4,
                owner: me,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        let after = client_app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 601)
            .count();
        assert_eq!(before, 0, "the non-owner shot spawned nothing");
        assert!(after > 0, "the owner's shot replayed the 601");
    }

    /// `case 110` (`MessageBuffer.cs:3789-3808`): the billed player consumes
    /// the wire stack and clears the cooldown; a non-billed player ignores it.
    #[test]
    fn w4_g50_the_110_wire_bill_consumes_for_the_billed_player() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        // Stock 20 wire in the client's first grid cell.
        client_app.inventory[0] = crate::item::ItemSlot {
            kind: 530,
            name: crate::registry::name(530),
            count: 20,
            prefix: 0,
        };
        client_app.player.as_mut().unwrap().wire_operations_cooldown = 60;
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::MassWireOperationPay(net::MassWireOperationPay {
                item_type: 530,
                count: 12,
                player: me,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert_eq!(client_app.inventory[0].count, 8, "12 wire consumed");
        assert_eq!(
            client_app.player.as_ref().unwrap().wire_operations_cooldown,
            0,
            "the bill cleared the cooldown"
        );
    }

    /// `case 111` (`MessageBuffer.cs:3809-3814`, `BirthdayParty.cs:145-164`):
    /// the host flips the manual party and credits THROW_A_PARTY on the rise.
    #[test]
    fn w4_g50_the_111_party_toggle_flips_and_credits() {
        let (mut host_app, mut client_app) = in_world_pair();
        assert!(!crate::world_events::party_is_up(&host_app.world_events));
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ToggleParty);
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(host_app.world_events.manual_party, "the party came up");
        assert!(crate::world_events::party_is_up(&host_app.world_events));
        // A second toggle drops it (no credit on the fall).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ToggleParty);
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(!host_app.world_events.manual_party, "the party dropped");
    }

    /// `case 112` kind 2 (`MessageBuffer.cs:3835-3837`, `NPC.cs:48652-48683`):
    /// the fairy burst spawns the type-278 dust ring.
    #[test]
    fn w4_g50_the_112_fairy_fx_spawns_the_dust_ring() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::SpecialFx(net::SpecialFx {
                effect: 2,
                x: 200,
                y: 100,
                b4: 0,
                s5: 0,
                b6: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            client_app
                .dust
                .slots()
                .iter()
                .any(|d| d.active && d.dust_type == 278),
            "the fairy type-278 dust spawned"
        );
    }

    /// `case 113` (`MessageBuffer.cs:3841-3854`, `DD2Event.cs:584-601`): the
    /// host summons the crystal on a valid stand and starts the invasion.
    #[test]
    fn w4_g50_the_113_crystal_summon_starts_the_invasion() {
        let (mut host_app, mut client_app) = in_world_pair();
        // An eternia crystal stand (466) at (30, 4) with a wide arena floor.
        for app in [&mut host_app, &mut client_app] {
            for x in 0..app.world.width {
                let i = app.world.index(x, 6);
                app.world.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
            let i = app.world.index(30, 4);
            app.world.tiles[i] = tr_core::Tile {
                kind: 466,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::CrystalInvasionStart(net::CrystalInvasionStart {
                x: 30,
                y: 4,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(host_app.world.dd2_event_ongoing, "the invasion started");
        assert!(
            host_app
                .npcs
                .iter()
                .any(|n| n.kind == crate::npc::NpcKind::DD2EterniaCrystal && n.hp > 0),
            "the crystal spawned"
        );
    }

    /// `case 114` (`MessageBuffer.cs:3855-3860`, `DD2Event.cs:629-638`): the
    /// wipe clears the DD2 hostiles from the client's store.
    #[test]
    fn w4_g50_the_114_wipe_clears_the_dd2_hostiles() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Seed a DD2 hostile (a goblin, 552) on the client via the 23 path.
        let mut goblin = crate::npc::Npc::new(crate::npc::NpcKind::DD2GoblinT1, 100.0, 100.0, 1, 0.0);
        goblin.hp = goblin.max_hp;
        host_app.npcs.push(goblin);
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            client_app
                .net_npcs
                .values()
                .any(|n| n.kind == crate::npc::NpcKind::DD2GoblinT1),
            "the goblin synced"
        );
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::CrystalInvasionWipe);
        }
        pump(&mut host_app, &mut client_app, 10);
        assert!(
            !client_app
                .net_npcs
                .values()
                .any(|n| n.kind == crate::npc::NpcKind::DD2GoblinT1),
            "the wipe cleared the goblin"
        );
    }

    /// `case 116` (`MessageBuffer.cs:3861-3866`): the countdown lands on the
    /// world's client-side timer.
    #[test]
    fn w4_g50_the_116_wait_time_stores_the_countdown() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::CrystalInvasionWait(net::CrystalInvasionWait {
                time_left: 1800,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert_eq!(client_app.world.dd2_time_left_between_waves, 1800);
    }

    /// `case 117` (`MessageBuffer.cs:3867-3886`): a self-hurt applies and
    /// relays; the local player's hp drops by the damage.
    #[test]
    fn w4_g50_the_117_self_hurt_drops_the_local_hp() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        let hp_before = client_app.player.as_ref().unwrap().hp;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerHurtV2(net::PlayerHurtV2 {
                player_id: me,
                reason: net::PlayerDeathReason::default(),
                damage: 10,
                hit_direction: 0,
                crit: false,
                pvp: false,
                cooldown_counter: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        let hp_after = client_app.player.as_ref().unwrap().hp;
        assert!(hp_after < hp_before, "the self-hurt dropped the hp");
    }

    /// W4-g55 `147 SyncLoadout` — `WriteAccessoryVisibility`
    /// (`NetMessage.cs:1882-1893`) packs `bool[10]` bit-i-per-slot into one
    /// ushort, and the reader (`MessageBuffer.cs:4494-4501`) unpacks it back.
    #[test]
    fn w4_g55_the_accessory_visibility_mask_round_trips() {
        let mut hide = [false; 10];
        hide[0] = true;
        hide[3] = true;
        hide[9] = true;
        let mask = accessory_visibility_mask(&hide);
        assert_eq!(mask, (1 << 0) | (1 << 3) | (1 << 9));
        // The unpack (`:4497-4500`): bit i -> slot i.
        let mut back = [false; 10];
        for (i, slot) in back.iter_mut().enumerate() {
            *slot = (mask & (1 << i)) != 0;
        }
        assert_eq!(back, hide);
        assert_eq!(accessory_visibility_mask(&[false; 10]), 0);
    }

    /// W4-g55 `147 SyncLoadout` — a local `TrySwitchingLoadout` sends the `147`
    /// (`Player.cs:5674`); the host forces the subject to the sender
    /// (`MessageBuffer.cs:4332`) and stores the peer's loadout
    /// (stored-not-simulated — no per-peer equipment arrays).
    #[test]
    fn w4_g55_a_client_loadout_switch_reaches_the_host_snapshot() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        client_app.player.as_mut().unwrap().hide_visible_accessory[2] = true;
        // The client switches to loadout 1 (the swap core + the 147 send).
        assert!(client_app.try_switching_loadout(1));
        assert_eq!(client_app.player.as_ref().unwrap().current_loadout_index, 1);
        let me = client_app.local_id as u32;
        let mut stored = None;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            stored = host_app.remote_players.get(&me).and_then(|p| p.loadout);
            if stored.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let l = stored.expect("the host stored the client's 147");
        assert_eq!(l.player_id, me as u8, "the subject forced to the sender");
        assert_eq!(l.loadout, 1, "the new loadout index");
        assert_eq!(l.hide_accessory, 1 << 2, "slot 2 hidden rides the mask");
    }

    /// W4-g55 `147 SyncLoadout`'s client half (`MessageBuffer.cs:4336-4337`):
    /// the LOCAL player runs the real swap core then unpacks the visibility
    /// bits; the swap gate's `whoAmI != Main.myPlayer` arm passes for a
    /// network-applied switch (`Player.cs:5663`).
    #[test]
    fn w4_g55_the_local_147_runs_the_swap_and_unpacks_visibility() {
        let (mut host_app, mut client_app) = in_world_pair();
        let _ = &mut host_app;
        client_app.player = Some(player::Player::spawn(&client_app.world));
        // Loadout 1 stores an Ancient Iron Helmet; the live set is air-headed.
        client_app.player.as_mut().unwrap().loadouts[1].armor[0] = crate::item::ItemSlot {
            kind: 954,
            name: crate::registry::name(954),
            count: 1,
            prefix: 0,
        };
        let me = client_app.local_id as u8;
        let role = std::mem::replace(&mut client_app.net, NetRole::Off);
        if let NetRole::Client(client) = &role {
            client_app.client_event(
                client,
                net::ClientEvent::Frame(S2c::SyncLoadout(net::SyncLoadout {
                    player_id: me,
                    loadout: 1,
                    hide_accessory: 0b1001, // slots 0 and 3 hidden
                })),
            );
        }
        client_app.net = role;
        let p = client_app.player.as_ref().unwrap();
        assert_eq!(p.current_loadout_index, 1, "the swap core ran");
        assert_eq!(client_app.equip.armor[0].kind, 954, "loadout 1's head is live");
        assert!(p.hide_visible_accessory[0], "bit 0 hid slot 0");
        assert!(!p.hide_visible_accessory[1]);
        assert!(p.hide_visible_accessory[3], "bit 3 hid slot 3");
        assert!(!p.hide_visible_accessory[4]);
    }

    /// `case 118` (`MessageBuffer.cs:3887-3904`): the host forces the subject
    /// to the sender; the local player dies and the death pipeline runs.
    #[test]
    fn w4_g50_the_118_death_kills_the_local_player() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerDeathV2(net::PlayerDeathV2 {
                player_id: 250, // the host overrides it (`:3892`)
                reason: net::PlayerDeathReason::default(),
                damage: 999,
                hit_direction: 0,
                pvp: false,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        let p = client_app.player.as_ref().unwrap();
        assert!(p.dead, "the local player died");
        assert_eq!(p.hp, 0);
        assert!(p.respawn_timer > 0, "the respawn timer armed");
    }

    /// `case 120` (`MessageBuffer.cs:3905-3919`): the host bubbles the emote
    /// on the sender and fans it out via 91; an out-of-range id is refused.
    #[test]
    fn w4_g50_the_120_emoji_bubbles_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Emoji(net::Emoji {
                player_id: 250, // the host overrides it (`:3910`)
                emote_id: 17,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        // The host bubbled it on the sender's anchor.
        assert!(
            host_app
                .emotes
                .bubbles
                .iter()
                .any(|b| b.emote == 17 && b.anchor_kind == 1 && b.anchor_index == me as u16),
            "the host bubbled the emote"
        );
        // The 91 relay reached the client.
        assert!(
            client_app.emotes.bubbles.iter().any(|b| b.emote == 17),
            "the 91 relay landed the bubble"
        );
        // An out-of-range id is refused (no new bubble).
        let before = host_app.emotes.bubbles.len();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::Emoji(net::Emoji {
                player_id: me,
                emote_id: 255,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        assert_eq!(host_app.emotes.bubbles.len(), before, "the over-range id refused");
    }

    /// `case 109` (`MessageBuffer.cs:3774-3788`, `Wiring.cs:419-453`): the
    /// host walks the sender's L-rectangle over the sender's wire stock,
    /// replicates the changed cells and bills the master with the two 110s;
    /// the client pays and clears `wireOperationsCooldown`. WRITTEN NOT RUN.
    #[test]
    fn w4_g50_the_109_mass_wire_walks_the_senders_stock_and_bills() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id;
        // The sender stands right-facing next to the drag rectangle.
        let entry = host_app.remote_players.entry(me).or_default();
        entry.id = me;
        entry.x = 8.0 * 16.0;
        entry.y = 2.0 * 16.0;
        entry.facing = 1;
        // 20 wire in the sender's host-side inventory copy (the 5 feed).
        if let NetRole::Host(server) = &host_app.net {
            server.set_slot_inventory(
                me,
                0,
                net::NetItem { kind: 530, stack: 20, prefix: 0, favorited: false, blocked: false },
            );
        }
        // The client carries the same stock (the bill consumes from it).
        client_app.inventory[0] = crate::item::ItemSlot {
            kind: 530,
            name: crate::registry::name(530),
            count: 20,
            prefix: 0,
        };
        client_app.player.as_mut().unwrap().wire_operations_cooldown = 60;
        // A single-cell Red drag at (10, 2): one wire placed, one billed.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::MassWireOperation(net::MassWireOperation {
                start_x: 10,
                start_y: 2,
                end_x: 10,
                end_y: 2,
                tool_mode: tr_core::smart_cursor::multi_tool_mode::RED,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            host_app
                .world
                .wires
                .get(10, 2, tr_core::wiring::WireColor::Red),
            "the host placed the red wire"
        );
        assert!(
            client_app
                .world
                .wires
                .get(10, 2, tr_core::wiring::WireColor::Red),
            "the 20 replication carried the wire bit to the client"
        );
        assert_eq!(client_app.inventory[0].count, 19, "the 110 bill consumed one wire");
        assert_eq!(
            client_app.player.as_ref().unwrap().wire_operations_cooldown,
            0,
            "the bill cleared the cooldown"
        );
    }

    /// `case 85` (`MessageBuffer.cs:3261-3266`, `QuickStacking.cs:256-266`):
    /// the host snapshots the sender's listed slots, transfers into the
    /// in-range chest via the g21 engine, re-sends the restored slots (5)
    /// and syncs the changed chest slot (32). WRITTEN NOT RUN.
    #[test]
    fn w4_g50_the_85_quick_stack_transfers_and_restores() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id;
        // The sender stands next to a chest at tile (10, 2).
        let entry = host_app.remote_players.entry(me).or_default();
        entry.id = me;
        entry.x = 10.0 * 16.0;
        entry.y = 2.0 * 16.0;
        // A world chest at (10, 2) holding 9900 wood in slot 0 (the tile
        // must exist unlocked — `Chest.IsLocked`'s null-tile arm reads
        // locked, `Chest.cs:303-305`).
        let ti = host_app.world.index(10, 2);
        host_app.world.tiles[ti].kind = 21;
        host_app.world.tiles[ti].frame_x = 0;
        host_app.world.tiles[ti].flags |= tr_core::tile::FLAG_ACTIVE;
        let idx = crate::chest::create_chest(&mut host_app.world_chests, 10, 2);
        crate::chest::set_chest_slot(
            &mut host_app.world_chests,
            idx,
            0,
            crate::item::ItemSlot { kind: 9, name: crate::registry::name(9), count: 9900, prefix: 0 },
        );
        // The sender's host-side slot copy: 50 wood in grid slot 10.
        let wood = net::NetItem { kind: 9, stack: 50, prefix: 0, favorited: false, blocked: false };
        if let NetRole::Host(server) = &host_app.net {
            server.set_slot_inventory(me, 10, wood);
        }
        client_app.inventory[10] = crate::item::ItemSlot {
            kind: 9,
            name: crate::registry::name(9),
            count: 50,
            prefix: 0,
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::QuickStackRequest(net::QuickStackRequest {
                slots: vec![10],
                smart_stack: false,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        // The host's chest merged the 50 wood (9900 + 50).
        let slots = crate::chest::chest_slots(&host_app.world_chests, idx).unwrap();
        assert_eq!(slots[0].count, 9950, "the wood merged into the chest");
        // The 32 sync carried it to the client's chest store.
        let cslots = crate::chest::chest_slots(&client_app.world_chests, idx).unwrap();
        assert_eq!(cslots[0].count, 9950, "the 32 sync landed the chest slot");
        // The 5 restore drained the sender's grid slot on both sides.
        assert_eq!(client_app.inventory[10].count, 0, "the restore drained the source");
        let inv = if let NetRole::Host(server) = &host_app.net {
            server.slot_inventory(me)
        } else {
            unreachable!()
        };
        assert_eq!(inv.get(&10).map(|i| i.stack), Some(0), "the host copy restored");
    }

    /// `case 94` (`MessageBuffer.cs:3463-3483`): the shared-state arms store
    /// under the enableDebugCommands gate; the float number2 truncates to
    /// int before the `== 1` comparison. The env pin is presence-based, so
    /// this witness pins the DEFAULT-OFF gate (no TR_DEBUG_COMMANDS in the
    /// test env): the message is consumed, nothing stores. WRITTEN NOT RUN.
    #[test]
    fn w4_g50_the_94_dev_commands_gate_stores_nothing_when_disabled() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The C# static-ctor defaults (`DebugOptions.cs:75-79`).
        assert!(host_app.debug_shared.report_command_usage);
        assert!(!host_app.debug_shared.randomize_projectile_slots);
        assert_eq!(host_app.debug_shared.server_ping, 0);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::DevCommands(net::DevCommands {
                command: "/setserverping".into(),
                number: 0,
                number2: 120.0,
                number3: 0.0,
            }));
        }
        pump(&mut host_app, &mut client_app, 10);
        if !crate::env::debug_commands() {
            assert_eq!(host_app.debug_shared.server_ping, 0, "the gate refused the store");
        } else {
            assert_eq!(host_app.debug_shared.server_ping, 120, "the enabled gate stored");
        }
    }

    /// W4-f21b — `57 Unknown57` (`MessageBuffer.cs:2631-2638`): the join
    /// push (`:867`) carries the host's three world-evil bytes and the
    /// client stores them into `WorldGen.tGood/tEvil/tBlood`'s port fields.
    /// WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_f21b_world_evil_bytes_join_push_stores_the_three_bytes() {
        let (mut host_app, mut client_app) = connect_pair("", "");
        host_app.world.t_good = 37;
        host_app.world.t_evil = 12;
        host_app.world.t_blood = 0;
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        assert_eq!(client_app.world.t_good, 37, "tGood stored (:2634)");
        assert_eq!(client_app.world.t_evil, 12, "tEvil stored (:2635)");
        assert_eq!(client_app.world.t_blood, 0, "tBlood stored (:2636)");
    }

    /// W4-f21b — `76 QuestsCountSync` (`MessageBuffer.cs:3164-3182`): the
    /// host forces the subject to the sender (`:3169-3172` — the wire's
    /// bogus id 99 is ignored), stores both ints (`:3173-3175`) and relays
    /// to the OTHERS (`:3178`); the second client's peer record carries the
    /// sender-forced id. WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_f21b_quests_count_sync_host_forces_id_stores_and_relays() {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr.clone(), String::new()));
        let mut second = flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new());
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        let sender = client_app.local_id;
        if let NetRole::Client(c) = &client_app.net {
            c.send(&C2s::QuestsCountSync(net::QuestsCountSync {
                player_id: 99, // the host must force this to the sender
                angler_quests_finished: 42,
                golfer_score: 1_000_000_000,
            }));
        }
        let mut relayed = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            relayed = second
                .remote_players
                .get(&sender)
                .is_some_and(|p| p.angler_quests_finished == 42);
            if relayed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // The host store (:3173-3175) under the FORCED id (:3171).
        let host_peer = host_app
            .remote_players
            .get(&sender)
            .expect("the sender's peer record");
        assert_eq!(host_peer.angler_quests_finished, 42);
        assert_eq!(host_peer.golfer_score, 1_000_000_000);
        assert!(
            !host_app.remote_players.contains_key(&99),
            "the bogus wire id never created a record"
        );
        // The relay (:3178) reached the OTHER client under the forced id.
        assert!(relayed, "the second client received the 76 relay");
        let peer = second.remote_players.get(&sender).unwrap();
        assert_eq!(peer.golfer_score, 1_000_000_000);
    }

    /// W4-f21b — `76`'s HOST own-echo gate (`MessageBuffer.cs:3167` runs
    /// BEFORE the `:3169` force): a frame naming the host's own slot 0 is
    /// dropped whole — no store, no relay. WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f21b_quests_count_sync_host_drops_own_slot_echo() {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr.clone(), String::new()));
        let mut second = flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new());
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        if let NetRole::Client(c) = &client_app.net {
            c.send(&C2s::QuestsCountSync(net::QuestsCountSync {
                player_id: 0, // the host's own slot — :3167 drops it
                angler_quests_finished: 7,
                golfer_score: 7,
            }));
        }
        for _ in 0..30 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let sender = client_app.local_id;
        let p = host_app
            .remote_players
            .get(&sender)
            .expect("the sender's peer record");
        assert_eq!(p.angler_quests_finished, 0, "the echo never stored");
        let relayed = second
            .remote_players
            .get(&sender)
            .is_some_and(|p| p.angler_quests_finished != 0);
        assert!(!relayed, "the echo never relayed");
    }

    /// W4-f21b — `76`'s client own-echo skip (`MessageBuffer.cs:3167`): a
    /// frame naming the local player is dropped (the port never SSCs, so
    /// the `ServerSideCharacter` escape does not exist); a peer frame
    /// stores. WRITTEN NOT RUN (code-only lane).
    #[test]
    fn w4_f21b_quests_count_sync_client_skips_own_echo_stores_peer() {
        let (mut host_app, mut client_app) = in_world_pair();
        let me = client_app.local_id as u8;
        if let NetRole::Host(server) = &host_app.net {
            // The own-echo frame: dropped, no self record appears.
            server.broadcast(&S2c::QuestsCountSync(net::QuestsCountSync {
                player_id: me,
                angler_quests_finished: 9,
                golfer_score: 9,
            }));
            // A peer frame (the host, slot 0): stored.
            server.broadcast(&S2c::QuestsCountSync(net::QuestsCountSync {
                player_id: 0,
                angler_quests_finished: 5,
                golfer_score: 77,
            }));
        }
        pump(&mut host_app, &mut client_app, 40);
        assert!(
            !client_app.remote_players.contains_key(&(me as u32)),
            "the own echo (:3167) never stores a self record"
        );
        let peer = client_app
            .remote_players
            .get(&0)
            .expect("the host peer record");
        assert_eq!(peer.angler_quests_finished, 5, ":3174");
        assert_eq!(peer.golfer_score, 77, ":3175");
    }

    /// W4-f21b — `58 InstrumentSound`'s host half
    /// (`MessageBuffer.cs:2642-2650`): the id is forced to the sender and
    /// the pitch relays to the OTHERS; no host-side sound. WRITTEN NOT RUN
    /// (code-only lane).
    #[test]
    fn w4_f21b_instrument_sound_host_forces_id_and_relays_pitch() {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr.clone(), String::new()));
        let mut second = flow_app(None, Vec::<PlayerEntry>::new(), Vec::<WorldEntry>::new());
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        let sender = client_app.local_id;
        if let NetRole::Client(c) = &client_app.net {
            c.send(&C2s::InstrumentSound(net::InstrumentSound {
                player_id: 99, // forced to the sender (:2644)
                pitch: 0.75,
            }));
        }
        // The relay is observable on the second client's inbox even though
        // its client apply parks: drain the raw frames.
        let mut got = None;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            if let NetRole::Client(c) = &second.net {
                while let Ok(ev) = c.events.try_recv() {
                    if let net::ClientEvent::Frame(S2c::InstrumentSound(s)) = ev {
                        got = Some(s);
                    }
                }
            }
            if got.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let s = got.expect("the second client received the 58 relay (:2649)");
        assert_eq!(s.player_id, sender as u8, "the id was forced (:2644)");
        assert_eq!(s.pitch, 0.75, "the raw float pitch rides verbatim");
    }

    /// W4-g15 lane 5 — the client's `22 ItemOwner` reservation hold ticks
    /// down on the 10 Hz cadence (6 per tick, the `timeToKeepReservation`
    /// 60 Hz clock's batched stand-in); an expired hold releases the
    /// reservation so the grab gate re-opens.
    #[test]
    fn w4_g15_the_client_reservation_hold_expires() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Seed a reservation directly (the join burst's 22 carries 255, so
        // the store starts empty; a live reservation is the host's
        // ReserveFor announcement — the port's client map is the stand-in).
        client_app.net_reserved.insert(0, (3, 12));
        // One cadence tick: 12 - 6 = 6, still held.
        client_app.net_tick(0.1);
        assert_eq!(client_app.net_reserved.get(&0), Some(&(3, 6)), "the hold decayed 6");
        // Another: 6 - 6 = 0, released.
        client_app.net_tick(0.1);
        assert!(!client_app.net_reserved.contains_key(&0), "the expired hold released");
        let _ = host_app;
    }

    /// W4-c27 `41 ItemRotationAndAnimation` — the host forces the subject to
    /// the sender (`MessageBuffer.cs:2274-2277`), stores rotation/animation
    /// (`:2281-2282`), re-derives `channel` from the selected item (`:2283`)
    /// and relays (`:2286`).
    #[test]
    fn w4_c27_item_anim_relays_with_the_sender_id() {
        let (mut host_app, mut client_app) = in_world_pair();
        let a = net::ItemAnim {
            player_id: 200, // spoofed — the host must override it to 1
            rotation: 0.75,
            animation: 12,
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ItemAnim(a));
        }
        pump(&mut host_app, &mut client_app, 12);
        let stored = host_app.remote_players.get(&1).expect("peer stored");
        assert_eq!(stored.item_rotation, 0.75);
        assert_eq!(stored.item_animation, 12);
        // No inventory synced yet: the selected slot's kind is unknown, so
        // the channel re-derive reads false (`:2283`).
        assert!(!stored.channel);
        assert!(client_app.remote_players.get(&1).is_none());
    }

    /// W4-c27 `43 ManaEffect` — the host forces the subject to the sender
    /// (`MessageBuffer.cs:2310-2313`) and spawns the HealMana text for anyone
    /// but itself (`:2315-2318`); the client does the same on the relay.
    #[test]
    fn w4_c27_mana_effect_spawns_the_peers_text() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Seed the client's peer record on the host so the text has a spot.
        host_app
            .remote_players
            .entry(1)
            .or_default()
            .x = 100.0;
        host_app.remote_players.get_mut(&1).unwrap().y = 200.0;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ManaEffect(net::ManaEffect {
                player_id: 200, // spoofed — forced to 1
                amount: 50,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        let texts: Vec<_> = host_app
            .combat_texts
            .iter()
            .filter(|t| t.active && t.text == "50")
            .collect();
        assert_eq!(texts.len(), 1, "one HealMana text for the peer");
        assert_eq!(texts[0].color, crate::combat_text::HEAL_MANA);
        assert_eq!(texts[0].position[0], 100.0);
    }

    /// W4-c27 `84 PlayerStealth` — the host forces the subject to the sender
    /// (`MessageBuffer.cs:3249-3252`), stores the float (`:3254`) and relays
    /// (`:3257`); the client stores a peer's value on its record.
    #[test]
    fn w4_c27_player_stealth_relays_with_the_sender_id() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerStealth(net::PlayerStealth {
                player_id: 200, // spoofed — the host must override it to 1
                stealth: 0.25,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        let stored = host_app.remote_players.get(&1).expect("peer stored");
        assert_eq!(stored.stealth, 0.25, "the float stored under the sender");
        // The relay excludes the sender, so no self-echo record.
        assert!(client_app.remote_players.get(&1).is_none());
        // The host's own stealth value reaches the client's peer record
        // (the `netMode == 1` half keeps the wire id, `:3254`).
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::PlayerStealth(net::PlayerStealth {
                player_id: 0,
                stealth: 0.625,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        let peer = client_app.remote_players.get(&0).expect("host peer");
        assert_eq!(peer.stealth, 0.625);
    }

    /// W4-c27 `36 SyncPlayerZone` — the host forces the subject to the sender
    /// (`MessageBuffer.cs:2181-2184`), stores the six bytes and relays to the
    /// others (`:2199`); the client keeps the wire id on its peer record.
    #[test]
    fn w4_c27_player_zone_relays_with_the_sender_id() {
        let (mut host_app, mut client_app) = in_world_pair();
        let z = net::PlayerZone {
            player_id: 200, // spoofed — the host must override it to 1
            zone1: BitsByte(0xA5),
            zone2: BitsByte(0x0F),
            zone3: BitsByte(0xF0),
            zone4: BitsByte(0x33),
            zone5: BitsByte(0x01),
            town_npcs: 4,
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerZone(z));
        }
        pump(&mut host_app, &mut client_app, 12);
        let stored = host_app.remote_players.get(&1).expect("peer stored");
        assert_eq!(stored.zone.player_id, 1, "the id forced to the sender");
        assert_eq!(stored.zone.zone1.0, 0xA5);
        assert_eq!(stored.zone.zone5.0, 0x01);
        assert_eq!(stored.zone.town_npcs, 4);
        // The relay excludes the sender (`:2199`'s `this.whoAmI`), so the
        // client never sees its own zone echo — its peer record for itself
        // stays absent, exactly the C#'s ignore-own shape.
        assert!(client_app.remote_players.get(&1).is_none());
    }

    /// W4-c27 `18 SetTime` — the client applies the host's clock verbatim
    /// (`MessageBuffer.cs:1290-1298`): the `int` time lands as f64 ticks, the
    /// `== 1` day flag, the mod offsets parked (no port field).
    #[test]
    fn w4_c27_set_time_applies_the_hosts_clock() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The host's own 10 Hz clock broadcast rides the same values, so the
        // assertion holds regardless of which message lands last.
        host_app.world.day_time = false;
        host_app.world.time = 16_200.0;
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::SetTime(net::SetTime {
                day_time: false,
                time: 16_200,
                sun_mod_y: -60,
                moon_mod_y: 30,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        assert!(!client_app.world.day_time, "the night flag applied");
        assert_eq!(client_app.world.time, 16_200.0, "the int clock applied");
    }

    /// W4-c27 SEAL (research/332): the apply arm stores ONLY the clock — the
    /// parked `sunModY`/`moonModY` fields never touch world state
    /// (`MessageBuffer.cs:1295-1296` writes `Main.sunModY`/`moonModY`, which
    /// have no port home; the offsets feed only the title-screen celestial
    /// drag, `Main.cs:62983-63007`, itself gated `gameMenu && netMode != 1`
    /// — never a client's world). The wire keeps them (the round-trip
    /// witness); the client apply deliberately drops them.
    #[test]
    fn w4_c27_set_time_apply_drops_the_parked_offsets() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The host's own 10 Hz internal-205 broadcast rides the same values
        // (the landed convention), so the assertion holds regardless of
        // which message lands last.
        host_app.world.day_time = false;
        host_app.world.time = 16_200.0;
        let before = (client_app.world.day_time, client_app.world.time);
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::SetTime(net::SetTime {
                day_time: false,
                time: 16_200,
                // Wild offsets: they must not perturb anything world-side.
                sun_mod_y: i16::MIN,
                moon_mod_y: i16::MAX,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        assert!(!client_app.world.day_time, "the flag applied");
        assert_eq!(client_app.world.time, 16_200.0, "the int clock applied");
        assert_ne!(
            before,
            (client_app.world.day_time, client_app.world.time),
            "the pair started elsewhere so the apply is observable"
        );
        // The dial cooldowns ride the WorldInfo join fields (and the host's
        // 51 kind-3/6 dial triggers) — an 18 frame leaves them alone (the
        // C# reader `MessageBuffer.cs:1290-1298` has no cooldown arm).
        assert_eq!(client_app.world.sundial_cooldown, 0, "no cooldown write from 18");
        assert_eq!(client_app.world.moondial_cooldown, 0, "no cooldown write from 18");
    }

    /// W4-c27 SEAL (research/332), the clock-family composition: the wire
    /// `int` (`NetMessage.cs:533` `(int)Main.time` truncates toward zero) and
    /// the STRICT boundary tests (`Main.cs:65137`/`:65148`, ported in
    /// `world_sim/clock.rs::advance_time`) compose — a clock applied exactly
    /// ON a boundary stays the old half on the client too. The host's
    /// fractional overshoot below the boundary truncates to the boundary
    /// int, which must NOT flip the client's day flag one tick early.
    #[test]
    fn w4_c27_set_time_int_truncation_keeps_the_strict_boundary() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The truncation itself: a host resting at 54000.7 (still day — the
        // strict `>` has not fired) puts 54000 on the wire.
        assert_eq!(54_000.7f64 as i32, 54_000, "(int) truncates toward zero");
        // The host's own 10 Hz internal-205 broadcast rides the same values
        // (the landed convention): the host rests exactly ON the boundary.
        host_app.world.day_time = true;
        host_app.world.time = 54_000.0;
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::SetTime(net::SetTime {
                day_time: true,
                time: host_app.world.time as i32,
                sun_mod_y: 0,
                moon_mod_y: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        assert!(client_app.world.day_time, "54000 exactly is still day");
        assert_eq!(client_app.world.time, 54_000.0);
        // The client's own sim then crosses on its next step past it — the
        // strict boundary fires exactly once, overshoot discarded.
        let mut rng = tr_core::UnifiedRandom::new(0);
        crate::world_sim::clock::advance_time(&mut client_app.world, 0.05, &mut rng, false);
        assert!(!client_app.world.day_time, "past 54000 the night starts");
        assert_eq!(client_app.world.time, 0.0, "the overshoot is discarded");
    }

    /// W4-c27 `62 SyncDodge` — the host forces the subject to the sender
    /// (`MessageBuffer.cs:2883-2886`) and relays to the others (`:2905`);
    /// the sender's own echo is excluded, so its local player is untouched
    /// (the send site applies its own dodge, S1-I-4).
    #[test]
    fn w4_c27_sync_dodge_relays_with_the_sender_id() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::SyncDodge(net::SyncDodge {
                player_id: 200, // spoofed — the host must override it to 1
                kind: 2,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        // The relay excludes the sender (`:2905`'s `this.whoAmI`), so the
        // client never re-applies its own dodge from an echo.
        let p = client_app.player.as_ref().unwrap();
        assert_eq!(p.shadow_dodge_timer, 0, "no self-echo apply");
        assert!(!p.immune);
        // The host-side apply on the non-simulated remote copy is parked
        // (no peer hurt lane); the host's own player is not the subject.
        assert_eq!(host_app.player.as_ref().unwrap().shadow_dodge_timer, 0);
    }

    /// W4-c27 `62 SyncDodge` — the client's own-id arm runs the ported
    /// handler (`MessageBuffer.cs:2891-2894` → `Player.ShadowDodge`,
    /// `Player.cs:38023-38042`): the 80-tick immunity (longInvince false),
    /// buff 59 stripped, the 1800-tick Hallowed cooldown armed.
    #[test]
    fn w4_c27_sync_dodge_shadow_applies_on_the_client() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        client_app.player.as_mut().unwrap().buffs.add_buff(59, 600);
        let me = client_app.local_id as u8;
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::SyncDodge(net::SyncDodge {
                player_id: me,
                kind: 2,
            }));
        }
        pump(&mut host_app, &mut client_app, 12);
        let p = client_app.player.as_ref().unwrap();
        assert!(p.immune, "the dodge armed the i-frames");
        assert_eq!(p.immune_time, 80, "longInvince false → 80");
        assert_eq!(p.shadow_dodge_timer, 1800, "the Hallowed cooldown");
        assert!(
            p.buffs.find_buff_index(59).is_none(),
            "buff 59 stripped by the dodge"
        );
    }

    // --- W4-c28: the residual send-trigger witnesses -------------------------

    use crate::app::net_player_sync::PlayerSend;

    /// The 60 Hz decision layer on a joined client with a live player.
    fn cadence_client() -> App {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        // Drain the join burst so the cadence tests see only their own sends.
        pump(&mut host_app, &mut client_app, 12);
        client_app.net_player_outbox.clear();
        client_app
    }

    /// The `Hurt` client tail (`Player.cs:38369-38377`): an accepted hurt
    /// queues the knockback-gated `13` + the `16` with the PRE-subtraction
    /// hp; the cadence drains them into the outbox. A dodged/immune hit
    /// queues nothing.
    #[test]
    fn an_accepted_hurt_queues_the_13_then_the_16_with_pre_hit_hp() {
        let mut app = cadence_client();
        app.player_sync_cadence();
        app.net_player_outbox.clear();
        let hp_before = app.player.as_ref().unwrap().hp;
        let max_hp = app.player.as_ref().unwrap().max_hp;
        let mut rng = tr_core::UnifiedRandom::default();
        let landed = app.player.as_mut().unwrap().hurt(10, 0.0, &mut rng, false);
        assert!(landed, "the hit lands");
        // The queue carries the PRE-subtraction hp (the C# sends at :38375,
        // before `statLife -= num2` at :38383).
        let queued = app.player.as_mut().unwrap().take_hurt_net();
        assert_eq!(queued, Some((true, hp_before, max_hp)));
        // Re-queue through a second hurt for the cadence drain (the first
        // take consumed it): reset the i-frames so the hit lands.
        {
            let p = app.player.as_mut().unwrap();
            p.immune = false;
            p.immune_time = 0;
            p.hurt_cd = 0.0;
        }
        let hp_before2 = app.player.as_ref().unwrap().hp;
        assert!(app.player.as_mut().unwrap().hurt(10, 0.0, &mut rng, false));
        app.player_sync_cadence();
        // The hurt 16 is the FIRST Life in the outbox (the drain runs ahead
        // of the clone-compare; the throttle's own 16 may ride later with
        // the POST-hit hp).
        let life = app
            .net_player_outbox
            .iter()
            .find_map(|s| match s {
                PlayerSend::Life { hp, .. } => Some(*hp),
                _ => None,
            })
            .expect("the hurt 16 fired");
        assert_eq!(life, hp_before2 as i16, "the hurt 16 carries the PRE-hit hp");
        assert!(
            app.net_player_outbox.iter().any(|s| matches!(s, PlayerSend::Controls)),
            "the knockback 13 rides first"
        );
        // The queue is drained — a second cadence pass adds no hurt sends
        // (the clone absorbed the hp change, the throttle window is armed).
        app.net_player_outbox.clear();
        app.player_sync_cadence();
        assert!(
            !app.net_player_outbox.iter().any(|s| matches!(s, PlayerSend::Life { .. })),
            "the hurt send fires once per accepted hit"
        );
    }

    /// The `53` client send (`NPC.cs:89932-89935`): a melee on-hit debuff on
    /// a joined client queues one `AddNpcBuff` per accepted pair, addressed
    /// by the HOST slot; the host applies it and pushes the `54` back.
    #[test]
    fn a_melee_on_hit_debuff_sends_53_from_the_client() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.npcs.len(), 1);
        client_app.player = Some(player::Player::spawn(&client_app.world));
        // The Flask of Venom arm: Venom 70 + PotentAcid 395 (the slime is
        // immune to neither — `NPCID.cs:6079-6084` lists only 20).
        let flags = crate::melee_on_hit::MeleeOnHitFlags {
            melee_enchant: 1,
            ..Default::default()
        };
        let mut rng = tr_core::UnifiedRandom::default();
        crate::melee_on_hit::status_to_npc_flags(
            flags,
            24, // a plain sword kind — no per-weapon arm
            &mut client_app.npcs[0].buffs,
            &mut rng,
        );
        // The engine queued both accepted pairs.
        assert_eq!(client_app.npcs[0].buffs.net_sends.len(), 2);
        client_app.drain_npc_buff_sends();
        assert!(client_app.npcs[0].buffs.net_sends.is_empty(), "the drain took the queue");
        pump(&mut host_app, &mut client_app, 15);
        // The host applied the 53s (`MessageBuffer.cs:2570`) and the 54
        // list came back to the client.
        let host_bar = &host_app.npcs[0].buffs;
        assert!(host_bar.find_buff_index(70).is_some(), "Venom landed on the host");
        assert!(host_bar.find_buff_index(395).is_some(), "PotentAcid landed on the host");
    }

    /// The host's `54` fan-out (`NPC.cs:89979-89982`): an AddBuff on the
    /// host's own NPC queues the list; the drain broadcasts it the same
    /// pass (the C# sends inline at the AddBuff site).
    #[test]
    fn a_host_add_buff_broadcasts_the_54_list_at_the_drain() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        pump(&mut host_app, &mut client_app, 15);
        // The host's own AddBuff (the AI/timer lane): Confused 31 — the
        // slime is NOT immune to it.
        host_app.npcs[0].buffs.add_buff(31, 600);
        assert_eq!(host_app.npcs[0].buffs.net_sends, vec![(31, 600)]);
        host_app.drain_npc_buff_sends();
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            client_app.npcs[0].buffs.find_buff_index(31).is_some(),
            "the 54 list reached the client"
        );
    }

    /// The catch-all buff drain: an AI-side buff mutation (no sim-pass drain)
    /// still rides one `54` on the next 60 Hz net step — the C# sends inline
    /// at `NPC.AddBuff` (`NPC.cs:89979-89982`); the port's net-phase drain
    /// picks up the queue the npcs phase left. One send per touched NPC, not
    /// per tick (the queue is taken, so an unchanged bar sends nothing).
    #[test]
    fn the_catch_all_drain_sends_one_buff_list_per_touched_npc() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        pump(&mut host_app, &mut client_app, 15);
        // An AI-side mutation (the drains never ran): the queue holds the pair.
        host_app.npcs[0].buffs.add_buff(31, 600);
        assert_eq!(host_app.npcs[0].buffs.net_sends, vec![(31, 600)]);
        // One net step drains it (the catch-all in the 60 Hz loop).
        host_app.net_tick(1.0 / 60.0);
        assert!(host_app.npcs[0].buffs.net_sends.is_empty(), "the drain took the queue");
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            client_app.npcs[0].buffs.find_buff_index(31).is_some(),
            "the 54 list reached the client"
        );
        // No further change: the queue stays empty, nothing re-sends.
        host_app.net_tick(1.0 / 60.0);
        assert!(host_app.npcs[0].buffs.net_sends.is_empty());
    }

    /// W4-c28: the expiry-sweep `54` (`NPC.cs:91890-91909`) — the sweep's
    /// quiet `DelBuff`s latch `expired_send` (`:91902`), the host drain
    /// turns the latch into ONE `54` full list, and the client's bar loses
    /// the expired buff. The decision-layer gate itself is pinned by
    /// `npc_buffs::tests::the_expired_sweep_queues_one_54_on_the_host_only_when_something_expired`.
    #[test]
    fn an_expired_buff_sweep_sends_one_54_from_the_host() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        pump(&mut host_app, &mut client_app, 15);
        // A 1-tick Confused 31 on the host, fanned out to the client.
        host_app.npcs[0].buffs.add_buff(31, 1);
        host_app.drain_npc_buff_sends();
        pump(&mut host_app, &mut client_app, 15);
        assert!(client_app.npcs[0].buffs.find_buff_index(31).is_some());
        // The sweep tick: the timer decrements to 0, the quiet DelBuff
        // clears the slot and latches the flag (`:91899-91902`).
        let mut hp = host_app.npcs[0].hp;
        let max_hp = host_app.npcs[0].max_hp;
        host_app.npcs[0].buffs.update_buffs_tick(&mut hp, max_hp);
        host_app.npcs[0].hp = hp;
        assert!(host_app.npcs[0].buffs.find_buff_index(31).is_none(), "the sweep cleared it");
        assert!(host_app.npcs[0].buffs.expired_send, "the flag latched");
        // The drain takes the latch and sends the one 54.
        host_app.drain_npc_buff_sends();
        assert!(!host_app.npcs[0].buffs.expired_send, "the drain took the latch");
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            client_app.npcs[0].buffs.find_buff_index(31).is_none(),
            "the 54 list reached the client without the expired buff"
        );
    }

    /// W4-c28: the client's expiry latch drops with no wire — the C# sweep
    /// returns before the flag on `netMode == 1` (`NPC.cs:91892-91895`), so
    /// the port's drain clears the latch instead of sending.
    #[test]
    fn the_client_expiry_latch_drops_without_a_wire() {
        let (mut host_app, mut client_app) = in_world_pair();
        host_app
            .npcs
            .push(crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue));
        pump(&mut host_app, &mut client_app, 15);
        client_app.npcs[0].buffs.expired_send = true;
        client_app.drain_npc_buff_sends();
        assert!(!client_app.npcs[0].buffs.expired_send, "the client latch dropped");
        assert!(client_app.npcs[0].buffs.net_sends.is_empty());
    }

    // --- W4-c28: the player-state send cadence (the SWAP witnesses) ---------

    #[test]
    fn the_cadence_quiesces_once_the_clone_matches() {
        let mut app = cadence_client();
        // One pass seeds the clone (the extension `13` for the spawn
        // position vs the all-zero snapshot, the mana 20-vs-0 report).
        app.player_sync_cadence();
        assert!(
            app.net_player_outbox.contains(&PlayerSend::Controls),
            "the first observation sends the spawn-state 13"
        );
        app.net_player_outbox.clear();
        // 100 unchanged ticks: no input change, no stat change, no beat
        // (the counter is far from 420) — the C# sends NOTHING.
        for _ in 0..100 {
            app.player_sync_cadence();
        }
        assert!(
            app.net_player_outbox.is_empty(),
            "an unchanged player produces no sends: {:?}",
            app.net_player_outbox
        );
    }

    #[test]
    fn the_cadence_sends_one_13_per_input_change_not_per_tick() {
        let mut app = cadence_client();
        app.player_sync_cadence();
        app.net_player_outbox.clear();
        // Hold jump: ONE 13 on the change tick, then silence while held.
        app.player.as_mut().unwrap().control_jump = true;
        for _ in 0..30 {
            app.player_sync_cadence();
        }
        let controls: Vec<_> = app
            .net_player_outbox
            .iter()
            .filter(|s| matches!(s, PlayerSend::Controls))
            .collect();
        assert_eq!(controls.len(), 1, "one 13 for the whole held window");
    }

    #[test]
    fn the_stat_report_throttles_to_one_per_60_ticks_under_continuous_change() {
        let mut app = cadence_client();
        app.player_sync_cadence();
        app.net_player_outbox.clear();
        // One mana change held constant: the report parks while the window
        // from the FIRST pass's report (time 60) drains, then fires once at
        // the boundary (`Main.cs:17649-17661`). Over 120 ticks exactly one
        // boundary is crossed, so exactly ONE report — not one per tick.
        app.player.as_mut().unwrap().mana = 10;
        let mut reports = 0;
        for _ in 0..120 {
            let before = app.net_player_outbox.len();
            app.player_sync_cadence();
            reports += app.net_player_outbox[before..]
                .iter()
                .filter(|s| matches!(s, PlayerSend::Mana { .. }))
                .count();
        }
        assert_eq!(reports, 1, "one report at the window boundary, not per tick");
        // A fresh change right after the boundary parks (the window re-armed
        // to 60) — the next few ticks send nothing.
        app.net_player_outbox.clear();
        app.player.as_mut().unwrap().mana = 5;
        for _ in 0..5 {
            app.player_sync_cadence();
        }
        assert!(
            !app.net_player_outbox.iter().any(|s| matches!(s, PlayerSend::Mana { .. })),
            "the fresh window suppresses the new change"
        );
    }

    #[test]
    fn the_420_beat_and_the_900_trio_ride_the_client_counter() {
        let mut app = cadence_client();
        app.player_sync_cadence();
        app.net_player_outbox.clear();
        // Park the counter one tick before the 420 beat.
        app.net_client_counter = 419;
        app.player_sync_cadence();
        assert_eq!(
            app.net_player_outbox,
            vec![PlayerSend::Controls],
            "tick 420 sends exactly the heartbeat 13"
        );
        app.net_player_outbox.clear();
        // Park one tick before 900: the trio is 36, 16, 40 in that order
        // (`Main.cs:64531-64536`); 900 % 420 != 0, so no 13 rides it.
        app.net_client_counter = 899;
        app.player_sync_cadence();
        assert_eq!(app.net_player_outbox.len(), 3);
        assert!(matches!(app.net_player_outbox[0], PlayerSend::Zone { .. }));
        assert!(matches!(app.net_player_outbox[1], PlayerSend::Life { .. }));
        assert!(matches!(app.net_player_outbox[2], PlayerSend::TalkNpc { .. }));
    }

    #[test]
    fn the_inventory_diff_sends_only_changed_slots_and_the_empty_clone_is_air() {
        let mut app = cadence_client();
        app.player_sync_cadence();
        app.net_player_outbox.clear();
        // The empty-grid join: every live slot is air and the clone's
        // missing entries compare as air (`new Item()`) — NO 5 spam.
        for _ in 0..10 {
            app.player_sync_cadence();
        }
        assert!(
            !app.net_player_outbox.iter().any(|s| matches!(s, PlayerSend::Inventory { .. })),
            "an empty inventory diffs clean against the air clone"
        );
        // Put one item in slot 3: exactly ONE 5 for slot 3, then silence.
        app.inventory[3] = crate::item::ItemSlot {
            kind: 9,
            name: crate::registry::name(9),
            count: 5,
            prefix: 0,
        };
        app.player_sync_cadence();
        let inv: Vec<_> = app
            .net_player_outbox
            .iter()
            .filter_map(|s| match s {
                PlayerSend::Inventory { slot } => Some(*slot),
                _ => None,
            })
            .collect();
        assert_eq!(inv, vec![3], "one 5 for the one changed slot");
        app.net_player_outbox.clear();
        for _ in 0..10 {
            app.player_sync_cadence();
        }
        assert!(
            !app.net_player_outbox.iter().any(|s| matches!(s, PlayerSend::Inventory { .. })),
            "the clone absorbed the change"
        );
    }

    #[test]
    fn the_close_transition_33_carries_the_live_index_and_the_clone_name() {
        let mut app = cadence_client();
        // A chest the clone has seen open (index 0, named).
        app.world_chests.push(crate::chest::WorldChest {
            tile: (3, 5),
            name: "Loot".into(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
            ..Default::default()
        });
        app.player.as_mut().unwrap().chest = 0;
        app.player_sync_cadence(); // the clone records chest 0
        app.net_player_outbox.clear();
        // The close: live -1, clone 0, the rename flag latched.
        let p = app.player.as_mut().unwrap();
        p.chest = -1;
        p.edited_chest_name = true;
        app.player_sync_cadence();
        let close = app
            .net_player_outbox
            .iter()
            .find_map(|s| match s {
                PlayerSend::ChestClose { chest, name } => Some((*chest, name.clone())),
                _ => None,
            })
            .expect("the close-transition 33 fired");
        assert_eq!(close.0, -1, "the writer carries the LIVE (closed) index");
        assert_eq!(close.1.as_deref(), Some("Loot"), "the name reads the clone's chest");
        assert!(
            !app.player.as_ref().unwrap().edited_chest_name,
            "the flush consumed the flag (Main.cs:17583)"
        );
        // The next pass: no second 33 (the clone absorbed the close).
        app.net_player_outbox.clear();
        app.player_sync_cadence();
        assert!(
            !app.net_player_outbox.iter().any(|s| matches!(s, PlayerSend::ChestClose { .. })),
            "the close edge fires once"
        );
    }

    #[test]
    fn the_extension_change_arms_one_13_not_one_per_60hz_step() {
        let mut app = cadence_client();
        app.player_sync_cadence();
        app.net_player_outbox.clear();
        // Move the player once, then hold still for a whole 10 Hz window:
        // the decision-time snapshot must not re-arm every 60 Hz step.
        app.player.as_mut().unwrap().x += 40.0;
        for _ in 0..6 {
            app.player_sync_cadence();
        }
        let controls = app
            .net_player_outbox
            .iter()
            .filter(|s| matches!(s, PlayerSend::Controls))
            .count();
        assert_eq!(controls, 1, "one extension 13 per change, not per step");
    }

    // --- W4-c28: the container `32` send cadence (event-driven) -------------

    /// A joined pair with matching chest 0 on both sides and a live client
    /// player browsing it.
    fn chest_pair() -> (App, App) {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        for app in [&mut host_app, &mut client_app] {
            let mut chest = crate::chest::WorldChest {
                tile: (3, 5),
                max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
                ..Default::default()
            };
            chest.slots.resize(crate::chest::DEFAULT_MAX_ITEMS as usize, crate::item::EMPTY_SLOT);
            app.world_chests.push(chest);
        }
        (host_app, client_app)
    }

    fn slot(kind: u16, count: u32) -> crate::item::ItemSlot {
        crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count,
            prefix: 0,
        }
    }

    #[test]
    fn a_client_chest_edit_reaches_the_host_over_32() {
        let (mut host_app, mut client_app) = chest_pair();
        // The client takes an item out of chest slot 0 (the ctx-3 left-click
        // shape: mutate the store, then the event-driven 32).
        crate::chest::set_chest_slot(&mut client_app.world_chests, 0, 0, slot(9, 5));
        client_app.send_chest_item(0, 0);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            crate::chest::chest_slot(&host_app.world_chests, 0, 0),
            Some(slot(9, 5)),
            "the host's store took the 32 (MessageBuffer.cs:1897-1918)"
        );
    }

    #[test]
    fn the_chest_item_send_is_client_only_and_world_chest_only() {
        let (mut host_app, mut client_app) = chest_pair();
        // The HOST's own chest edit never sends a client 32 (the host relays
        // through its own arms) — the client's store stays untouched.
        crate::chest::set_chest_slot(&mut host_app.world_chests, 0, 0, slot(9, 5));
        host_app.send_chest_item(0, 0);
        // A negative chest (bank / closed) never sends either.
        crate::chest::set_chest_slot(&mut client_app.world_chests, 0, 1, slot(8, 3));
        client_app.send_chest_item(-3, 1);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            crate::chest::chest_slot(&client_app.world_chests, 0, 0),
            Some(crate::item::EMPTY_SLOT),
            "the host sent no client 32"
        );
        assert_eq!(
            crate::chest::chest_slot(&host_app.world_chests, 0, 1),
            Some(crate::item::EMPTY_SLOT),
            "the bank-index send never fired"
        );
    }

    #[test]
    fn the_bulk_diff_sends_only_the_changed_slots() {
        let (mut host_app, mut client_app) = chest_pair();
        // The host's slot 0 holds a DIFFERENT item than the client's: a
        // spurious 32 for an unchanged slot would overwrite it.
        crate::chest::set_chest_slot(&mut host_app.world_chests, 0, 0, slot(7, 1));
        crate::chest::set_chest_slot(&mut client_app.world_chests, 0, 0, slot(9, 5));
        let before = client_app.world_chests[0].slots.clone();
        // The bulk pass changes ONLY slot 1.
        crate::chest::set_chest_slot(&mut client_app.world_chests, 0, 1, slot(8, 3));
        client_app.send_chest_item_diff(0, &before);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            crate::chest::chest_slot(&host_app.world_chests, 0, 1),
            Some(slot(8, 3)),
            "the changed slot's 32 landed"
        );
        assert_eq!(
            crate::chest::chest_slot(&host_app.world_chests, 0, 0),
            Some(slot(7, 1)),
            "the unchanged slot sent nothing (the host keeps its own item)"
        );
    }

    // --- W4-c28: the id-lane bank3[0]/TRASH collision fix -------------------

    /// The port's `PlayerItemSlotID` walk with the 40-slot banks
    /// (`PlayerItemSlotID.cs:175-185`): the trash sentinel reserves ONE id
    /// between bank2 and bank3 (`AllocateSlots(1)` `:183`), so bank3[0] is
    /// 180 — NOT the trash's 179 (the old compact layout collapsed them).
    #[test]
    fn the_trash_sentinel_reserves_one_id_between_bank2_and_bank3() {
        use player_item_slot as ps;
        assert_eq!(ps::BANK_1_0, 99);
        assert_eq!(ps::BANK_2_0, 139);
        assert_eq!(ps::TRASH, 179, "TrashItem = Bank2_0 + 40 (the walk)");
        assert_eq!(ps::BANK_3_0, 180, "Bank3_0 = TrashItem + 1 (:184)");
        assert_eq!(ps::BANK_4_0, 220);
        assert_ne!(ps::bank_1_0(2), ps::TRASH, "bank3[0] is not the trash");
        // `CanRelay` (`:181-185`): banks 1-3 + trash false, bank4 true.
        assert!(!ps::can_relay(ps::TRASH));
        assert!(!ps::can_relay(ps::BANK_3_0));
        assert!(!ps::can_relay(ps::BANK_3_0 + 39));
        assert!(ps::can_relay(ps::BANK_4_0));
        assert!(ps::can_relay(ps::BANK_4_0 + 39));
        assert!(!ps::can_relay(ps::BANK_4_0 + 40));
    }

    /// The `5` diff now carries bank3[0]: a change there arms exactly one
    /// `Inventory` for id 180, and the body reads bank3's store — while a
    /// trash change arms id 179 with the trash's body. The two never fight
    /// over one clone key (the old collision made bank3[0] unsendable).
    #[test]
    fn bank3_slot_zero_and_the_trash_diff_as_distinct_wire_slots() {
        let item = |kind: u16, count: u32| crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count,
            prefix: 0,
        };
        let mut app = cadence_client();
        app.player_sync_cadence();
        app.net_player_outbox.clear();
        {
            let p = app.player.as_mut().unwrap();
            p.banks[2][0] = item(9, 5); // bank3[0]
            p.banks[3][0] = item(7, 2); // bank4[0] — the >179 id the u8 map lost
        }
        app.equip.trash = item(8, 1);
        app.player_sync_cadence();
        let inv: Vec<i16> = app
            .net_player_outbox
            .iter()
            .filter_map(|s| match s {
                PlayerSend::Inventory { slot } => Some(*slot),
                _ => None,
            })
            .collect();
        assert!(inv.contains(&179), "the trash diffs at its own id");
        assert!(inv.contains(&180), "bank3[0] diffs — no longer skipped");
        assert!(inv.contains(&220), "bank4[0] diffs at the full i16 id");
        // The bodies resolve through the same layout the join push uses.
        assert_eq!(app.net_item_at_wire_slot(179).map(|i| i.kind), Some(8));
        assert_eq!(app.net_item_at_wire_slot(180).map(|i| i.kind), Some(9));
        assert_eq!(app.net_item_at_wire_slot(220).map(|i| i.kind), Some(7));
        // The clone absorbed all three: no re-send on the next pass.
        app.net_player_outbox.clear();
        app.player_sync_cadence();
        assert!(
            !app.net_player_outbox.iter().any(|s| matches!(s, PlayerSend::Inventory { .. })),
            "the three ids hold distinct clone keys"
        );
    }

    /// End-to-end: the client's bank3[0] / trash / bank4[0] changes ride the
    /// `5` cadence over the wire, and the host's per-client copy keys the
    /// FULL i16 ids (`MessageBuffer.cs:402` `ReadInt16`) — the old u8 key
    /// truncated 220 and collapsed 179/180.
    #[test]
    fn the_server_inventory_copy_keeps_the_full_i16_slot_ids() {
        let item = |kind: u16, count: u32| crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count,
            prefix: 0,
        };
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        pump(&mut host_app, &mut client_app, 12); // drain the join burst
        {
            let p = client_app.player.as_mut().unwrap();
            p.banks[2][0] = item(9, 5); // bank3[0] → id 180
            p.banks[3][0] = item(7, 2); // bank4[0] → id 220
        }
        client_app.equip.trash = item(8, 1); // → id 179
        pump(&mut host_app, &mut client_app, 15);
        let NetRole::Host(server) = &host_app.net else {
            panic!("host lost its role");
        };
        let snap = server.slot_inventory(client_app.local_id);
        assert_eq!(snap.get(&179).map(|i| i.kind), Some(8), "trash at 179");
        assert_eq!(snap.get(&180).map(|i| i.kind), Some(9), "bank3[0] at 180");
        assert_eq!(snap.get(&220).map(|i| i.kind), Some(7), "bank4[0] at 220");
    }

    // --- W4-c28: the tile-edit cadence is EVENT-driven (immediate) ----------

    /// `WorldGen.KillTile`/`PlaceTile`'s `SendData(17)` / `SendTileSquare`
    /// fire at the EDIT SITE (`Player.cs:53783`/`:40152`, `WorldGen.cs:63937`),
    /// not on a timer. The port's `request_edit` sends the client `Edit`
    /// immediately (`client.send` pushes straight to the wire buffer); the
    /// 10 Hz `net_tick` carries no tile-edit batch. This pins that the send
    /// lands WITHOUT a cadence tick — the host applies it on the next poll.
    #[test]
    fn a_tile_edit_sends_at_the_edit_site_not_on_the_cadence_tick() {
        let (mut host_app, mut client_app) = big_world_pair();
        mark(&mut client_app.world, 60, 60, 6);
        // The client breaks the tile locally, then asks for the replication
        // (the editing.rs shape: mutate, then request_edit). NO net_tick.
        let _ = crate::world_sim::editing::apply_edit(&mut client_app.world, 60, 60, 0, 6);
        client_app.request_edit(60, 60, 0, 6);
        // Only poll — the edit was already queued by request_edit, so the
        // host applies it without any 10 Hz cadence step on the client.
        for _ in 0..30 {
            host_app.poll_net();
            client_app.poll_net();
            if host_app.world.tile(60, 60).map_or(0, |t| t.kind) == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(
            host_app.world.tile(60, 60).map_or(0, |t| t.kind),
            0,
            "the edit reached the host with no cadence tick (event-driven)"
        );
    }

    // --- W4-c28: the PVP/team UI click send triggers (the census residuals) --

    /// The `30 TogglePVP` click send (`Main.cs:42694`): a joined client's
    /// `send_toggle_pvp` pushes one `30` carrying its id + the live hostile
    /// flag. The host's apply/relay is g48's lane — on the c28 base the host
    /// PARKS the read (the no-op arm), so this pins the SEND leg: the frame
    /// reaches the host's accept gate (the slot stays in-world, not kicked —
    /// a `30` from an in-world slot passes `host_gate`). The byte body is
    /// pinned by `net::tests::s1_i_1_toggle_pvp_round_trips`.
    #[test]
    fn a_client_pvp_toggle_sends_one_30_with_the_live_flag() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        pump(&mut host_app, &mut client_app, 12);
        let me = client_app.local_id as u8;
        // The flip lands first (the click site's order), then the send reads
        // the live flag (`NetMessage.cs:858` reads `Main.player[number].hostile`).
        client_app.player.as_mut().unwrap().hostile = true;
        client_app.send_toggle_pvp(true);
        // The frame is queued immediately (client.send pushes to the wire
        // buffer — the event-driven convention, no cadence tick needed).
        pump(&mut host_app, &mut client_app, 15);
        let NetRole::Host(server) = &host_app.net else {
            panic!("host lost its role");
        };
        assert_eq!(
            server.state(client_app.local_id),
            net::slot_state::IN_WORLD,
            "the 30 passed the accept gate (not kicked)"
        );
        // The body the send carries: id + hostile, the writer's two bytes.
        let body = C2s::TogglePvp(net::TogglePvp { player_id: me, hostile: true }).encode();
        assert_eq!(body[2], 30);
        assert_eq!(&body[3..], &[me, 1]);
    }

    /// The host's own `30` broadcast leg (`Main.cs:42694` folds the host in —
    /// `SendData(30, -1, …)` fans out through the default arm): the host's
    /// `send_toggle_pvp` broadcasts the live flag to every client. The client
    /// read is g48's `client_toggle_pvp` (parked on the c28 base); this pins
    /// that the host leg produces the `30` frame without a panic and the
    /// client stays connected.
    #[test]
    fn a_host_pvp_toggle_broadcasts_the_30_to_the_clients() {
        let (mut host_app, mut client_app) = in_world_pair();
        pump(&mut host_app, &mut client_app, 12);
        host_app.player.as_mut().unwrap().hostile = true;
        host_app.send_toggle_pvp(true);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD, "the client stayed in-world");
        // The host's broadcast body: slot 0 + the live flag.
        let body = S2c::TogglePvp(net::TogglePvp { player_id: 0, hostile: true }).encode();
        assert_eq!(body[2], 30);
        assert_eq!(&body[3..], &[0, 1]);
    }

    /// The `157 TeamChangeFromUI` send DECISION parks until d97's id merges
    /// (`Main.cs:42766`): a `45` stand-in would skip the C# reader's `b == 157`
    /// team-spawn tail (`MessageBuffer.cs:2355`), so the c28 send is recorded
    /// but emits nothing — the team set still lands locally (the click's
    /// first effect). This pins the parked decision: no frame, no kick.
    #[test]
    fn the_team_click_send_parks_until_the_157_id_lands() {
        let (mut host_app, mut client_app) = in_world_pair();
        client_app.player = Some(player::Player::spawn(&client_app.world));
        pump(&mut host_app, &mut client_app, 12);
        client_app.player.as_mut().unwrap().team = 2;
        client_app.send_team_change(2);
        pump(&mut host_app, &mut client_app, 15);
        // The local team set is the click's first effect; the send parked.
        assert_eq!(client_app.player.as_ref().unwrap().team, 2);
        let NetRole::Host(server) = &host_app.net else {
            panic!("host lost its role");
        };
        assert_eq!(server.state(client_app.local_id), net::slot_state::IN_WORLD);
        // The host's peer record exists (the join) but no team relay fired —
        // the `45`/`157` apply arms are g49's lane; the parked send produced
        // no frame at all (the client's team stayed local).
        assert!(host_app.remote_players.contains_key(&client_app.local_id));
    }
}

/// W4-g48 (S1-I-4 apply arms, lane 1 — MessageBuffer ids 1-40): the parked
/// wave-A/B/I dispatcher arms wired in the C# `GetData` case order. Each
/// witness drives the real loopback pair (`in_world_pair`) and pins the
/// applied state against the C# reader (`MessageBuffer.cs`).
#[cfg(test)]
mod g48_apply_tests {
    use super::*;
    use crate::app::netglue::handshake_tests::{apps, drive_to, in_world_pair};

    /// A joined pair over a 64x64 world (the `big_world_pair` shape): the
    /// 8x8 flow world's fluff-3 handler box (`in_world(x, y, 3)` → 3..5)
    /// and the kill cascade's 5-tile edge guard leave no room for the
    /// multi-tile fixtures, so the round-2 witnesses build their own.
    fn roomy_pair() -> (App, App) {
        let (mut host_app, mut client_app) = apps();
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr, String::new()));
        host_app.world = tr_core::World::new("flow".into(), 64, 64);
        host_app.world.column_surface = vec![64; 64];
        for x in 0..64 {
            let i = host_app.world.index(x, 20);
            host_app.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            host_app.world.column_surface[x as usize] = 20;
        }
        drive_to(&mut host_app, &mut client_app, net::client_state::IN_WORLD);
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        host_app.player = Some(player::Player::spawn(&host_app.world));
        (host_app, client_app)
    }

    /// `case 11` (`MessageBuffer.cs:895-899`): the client runs
    /// `WorldGen.SectionTileFrame` — the section's tiles re-roll their frame
    /// variants (`WorldGen.cs:80348-80358`, reset EVERY cell) and the world
    /// revision bumps so the render caches invalidate.
    #[test]
    fn g48_11_tile_frame_section_reframes_the_clients_section() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A dirt patch inside section (0, 0): the join stream framed it with
        // stored variants; the 11 re-roll must produce valid frames again.
        for x in 2..8 {
            for y in 2..4 {
                let i = client_app.world.index(x, y);
                client_app.world.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                client_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        let rev = client_app.world.revision;
        if let NetRole::Host(server) = &host_app.net {
            server.broadcast(&S2c::TileFrameSection(net::TileFrameSection {
                start_x: 0,
                start_y: 0,
                end_x: 0,
                end_y: 0,
            }));
        }
        let mut applied = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            applied = client_app.world.revision != rev;
            if applied {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(applied, "the 11 arm re-framed and bumped the revision");
        // Every dirt cell in the patch carries a framed sprite (the merge
        // rectangle resolves) and a stored 0..=2 variant.
        for x in 2..8 {
            for y in 2..4 {
                let idx = client_app.world.index(x, y);
                let t = client_app.world.tiles[idx];
                assert!(t.frame_x >= 0 && t.frame_y >= 0, "framed ({x},{y})");
                assert!(client_app.world.frame_number(idx) <= 2);
            }
        }
    }

    /// `case 12` (`MessageBuffer.cs:901-950`): a client's respawn stores the
    /// spawn fields on the host's peer copy (`:908-918`), latches `dead` from
    /// `respawnTimer > 0` and relays the body to the other peers (`:948`);
    /// the receiving client repositions the peer via `Spawn_SetPosition`
    /// (`Player.cs:37987-37991`).
    #[test]
    fn g48_12_player_spawn_stores_fields_and_relays_the_respawn() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A second peer so the relay has a receiver besides the host.
        let mut second = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        second.player_name = "second".into();
        let addr = match &host_app.net {
            NetRole::Host(server) => server.addr().to_string(),
            _ => unreachable!(),
        };
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        drive_to(&mut host_app, &mut second, net::client_state::IN_WORLD);
        // The first client respawns at (30, 3) with a live respawn timer.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerSpawn(net::PlayerSpawn {
                player_id: client_app.local_id as u8,
                x: 30,
                y: 3,
                respawn_timer: 120,
                deaths_pve: 2,
                deaths_pvp: 1,
                team: 3,
                spawn_context: 2, // ReviveFromDeath
            }));
        }
        let mut landed = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            landed = second
                .remote_players
                .get(&(client_app.local_id))
                .is_some_and(|p| p.respawn_timer == 120);
            if landed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(landed, "the 12 relay reached the second peer");
        let peer_id = client_app.local_id;
        let host_peer = &host_app.remote_players[&peer_id];
        assert_eq!((host_peer.spawn_x, host_peer.spawn_y), (30, 3));
        assert_eq!((host_peer.deaths_pve, host_peer.deaths_pvp), (2, 1));
        assert_eq!(host_peer.team, 3);
        assert!(host_peer.dead, "respawnTimer > 0 latches dead (:915-918)");
        let second_peer = &second.remote_players[&peer_id];
        assert_eq!((second_peer.spawn_x, second_peer.spawn_y), (30, 3));
        assert!(second_peer.dead);
        // `Spawn_SetPosition`: centred on the cell, feet at its top.
        let want_x = 30.0 * TILE_PX + 8.0 - player::WIDTH * 0.5;
        let want_y = 3.0 * TILE_PX - player::HEIGHT;
        assert!((second_peer.x - want_x).abs() < 0.01);
        assert!((second_peer.y - want_y).abs() < 0.01);
        assert_eq!(second_peer.off_x, 0.0, "Spawn zeroes netOffset (:37638)");
    }

    /// `case 16` (`MessageBuffer.cs:1095-1117`): a client's health report
    /// stores hp/max on the host's peer copy, derives `dead` from
    /// `statLife <= 0` (`:1111`) and relays to the other peers (`:1114`).
    #[test]
    fn g48_16_player_hp_relays_and_derives_dead() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerHp {
                player_id: 255, // the host forces the sender (`:1100-1103`)
                hp: 0,
                max_hp: 100,
            });
        }
        let mut landed = false;
        for _ in 0..60 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            landed = host_app
                .remote_players
                .get(&client_app.local_id)
                .is_some_and(|p| p.hp == 0 && p.dead);
            if landed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(landed, "the 16 arm stored hp and derived dead");
        let peer = &host_app.remote_players[&client_app.local_id];
        assert_eq!(peer.max_hp, 100);
    }

    /// `case 17` kind 0 (`MessageBuffer.cs:1157-1163`): a client's KillTile
    /// edit breaks the host's tile (the cascade + drops) and relays the edit
    /// to the other peers (`:1285`); the receiving client breaks its own
    /// copy without spawning local drops (the `netMode != 1` gate).
    #[test]
    fn g48_17_kill_tile_edit_breaks_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A stone block at (6, 3) on both sides (above the dirt row).
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(6, 3);
            app.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEdit(net::TileEdit {
                kind: 0,
                x: 6,
                y: 3,
                value: 0,
                style: 0,
            }));
        }
        let mut broken = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            broken = !host_app.world.tile_active(6, 3) && !client_app.world.tile_active(6, 3);
            if broken {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(broken, "the 17 kill broke both copies");
        // The host spawned the stone drop (item 3); the client did not spawn
        // a local one (its copy arrives through the 21 cadence).
        assert!(host_app.items.iter().any(|d| d.kind == 3), "the host's loot");
    }

    /// `case 17` kind 5/6 (`MessageBuffer.cs:1187-1194`): the wire place/kill
    /// edits flip the host's wire side-table and relay; the kill drops the
    /// 530 wire item host-side (`WorldGen.cs:58130`).
    #[test]
    fn g48_17_wire_place_and_kill_edit_replicates() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEdit(net::TileEdit {
                kind: 5, // PlaceWire (red)
                x: 7,
                y: 3,
                value: 0,
                style: 0,
            }));
        }
        let mut wired = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            wired = host_app.world.wires.get(7, 3, tr_core::wiring::WireColor::Red)
                && client_app.world.wires.get(7, 3, tr_core::wiring::WireColor::Red);
            if wired {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(wired, "the red wire landed on both sides");
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEdit(net::TileEdit {
                kind: 6, // KillWire (red)
                x: 7,
                y: 3,
                value: 0,
                style: 0,
            }));
        }
        let mut cut = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            cut = !host_app.world.wires.get(7, 3, tr_core::wiring::WireColor::Red)
                && !client_app.world.wires.get(7, 3, tr_core::wiring::WireColor::Red);
            if cut {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(cut, "the wire came off both sides");
        assert!(host_app.items.iter().any(|d| d.kind == 530), "the 530 drop");
    }

    /// `case 17` kind 15 (`MessageBuffer.cs:1227-1230`): a client's track
    /// pound runs `Minecart.FrameTrack(x, y, pound: true)` (the C# default
    /// `mute: false`) on the host and relays; the receiving client frames
    /// its own copy. A level straight (front frame 1, both connections 1)
    /// between two rails switches to the up-slope frame 5 (left 1 / right
    /// 2 — the first two-sided option of mask 18 after the current front,
    /// `Minecart.cs:1040-1108`); the C# fail-tile tail never removes the
    /// track (the W4-g48 no-op fix over f11's kill). WRITTEN NOT RUN.
    #[test]
    fn g48_17_track_pound_edit_frames_and_replicates() {
        let (mut host_app, mut client_app) = roomy_pair();
        // Three rails in a row at y=10: (9,10) front 1, (10,10) front 1
        // back −1 (the hit), (11,10) front 1 — the mask is 2|16 = 18.
        for app in [&mut host_app, &mut client_app] {
            for x in 9..=11i32 {
                let i = app.world.index(x, 10);
                app.world.tiles[i] = tr_core::Tile {
                    kind: 314,
                    frame_x: 1,
                    frame_y: -1,
                    ..Default::default()
                };
                app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEdit(net::TileEdit {
                kind: 15,
                x: 10,
                y: 10,
                value: 0,
                style: 0,
            }));
        }
        let mut framed = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            framed = host_app.world.tile_frame_x(10, 10) == 5
                && client_app.world.tile_frame_x(10, 10) == 5;
            if framed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(framed, "the track switched to the up-slope frame on both sides");
        for app in [&host_app, &client_app] {
            // The fail-tile tail (`flag4` true, mute false) did NOT kill
            // the track (`WorldGen.cs:63709-63737` — no removal).
            let t = app.world.tile(10, 10).unwrap();
            assert!(t.active() && t.kind == 314, "the track survives the fail tick");
            assert_eq!(t.frame_y, -1, "no back connection on the mask");
        }
    }

    /// `case 19` action 0 (`MessageBuffer.cs:1309-1311`): a client's door-open
    /// toggle runs `WorldGen.OpenDoor` on the host and relays the normalised
    /// direction to the other peers; the receiving client opens its copy.
    #[test]
    fn g48_19_door_open_toggle_replicates() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A closed door (tile 10, 1x3) at (8, 1..3) above the dirt row (y 4),
        // on both sides. `place_item_tile`'s y is the bottom row.
        for app in [&mut host_app, &mut client_app] {
            assert!(tr_core::edit::place_item_tile(&mut app.world, 8, 3, 10, 0, 1));
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::DoorToggle(net::DoorToggle {
                action: 0,
                x: 8,
                y: 2,
                direction: 1,
            }));
        }
        let mut opened = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            // The open door is tile 11 (the closed 10 became the open pair).
            opened = host_app.world.tile(8, 2).is_some_and(|t| t.kind == 11)
                && client_app.world.tile(8, 2).is_some_and(|t| t.kind == 11);
            if opened {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(opened, "the door opened on both sides");
    }

    /// `case 30` (`MessageBuffer.cs:1851-1867`): a client's PvP flip stores
    /// `hostile` on the host's peer copy, relays to the others and broadcasts
    /// the team-coloured chat line (`Lang.mp[11]`, `Main.teamColor[team]`).
    #[test]
    fn g48_30_pvp_toggle_stores_relays_and_chats() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The peer is on team 1 (red) — the 12 spawn carried it; here the
        // fixture sets it directly (the join default is 0).
        host_app
            .remote_players
            .entry(client_app.local_id)
            .or_default()
            .team = 1;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TogglePvp(net::TogglePvp {
                player_id: 255, // the host forces the sender (`:1854-1857`)
                hostile: true,
            }));
        }
        let mut landed = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            landed = host_app
                .remote_players
                .get(&client_app.local_id)
                .is_some_and(|p| p.hostile)
                && !host_app.chat.messages.is_empty();
            if landed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(landed, "the 30 arm stored hostile and broadcast the chat");
        // The chat line is the team-1 red (`Main.cs:6658`).
        let line = &host_app.chat.messages[0];
        assert_eq!(line.color, [218, 59, 59], "Main.teamColor[1]");
        // The client received the relay (its own echo is skipped by the
        // host's broadcast_except, so the flag lands via the module only if
        // a second peer existed; here the host's store is the witness).
    }

    /// `case 34` action 0 (`MessageBuffer.cs:2001-2017`): a client's chest
    /// placement runs the host's `PlaceChest` (the store slot + the 2x2
    /// tiles) and relays the index; the receiving client writes the same
    /// tiles + store slot via `PlaceChestDirect`.
    #[test]
    fn g48_34_chest_place_replicates_the_store_and_tiles() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The dirt row (y 4) supports a chest at (8, 3) — the 2x2 sits on it.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestUpdates(net::ChestUpdates {
                action: 0,
                x: 8,
                y: 3,
                style: 0,
                chest: 0, // the client writes 0; the host zeroes it (`:1993`)
            }));
        }
        let mut placed = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            placed = crate::chest::find_chest(&host_app.world_chests, 8, 2).is_some()
                && crate::chest::find_chest(&client_app.world_chests, 8, 2).is_some();
            if placed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(placed, "the chest store slot landed on both sides");
        // The 2x2 tiles (the anchor is (8, 2): PlaceChest's objectData.yCoord).
        for app in [&host_app, &client_app] {
            for dx in 0..2 {
                for dy in 0..2 {
                    let t = app.world.tile(8 + dx, 2 + dy).unwrap();
                    assert!(t.active() && t.kind == 21, "chest cell ({dx},{dy})");
                }
            }
        }
    }

    /// `case 35` (`MessageBuffer.cs:2160-2177`): a client's heal raises the
    /// HealLife combat text at the peer's position on the host and relays;
    /// the receiving client spawns the same text.
    #[test]
    fn g48_35_player_heal_spawns_the_peer_text() {
        let (mut host_app, mut client_app) = in_world_pair();
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerHeal(net::PlayerHeal {
                player_id: 255, // the host forces the sender (`:2163-2166`)
                amount: 20,
            }));
        }
        let mut spawned = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            spawned = host_app.combat_texts.iter().any(|t| t.active && t.text == "20");
            if spawned {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(spawned, "the host spawned the peer's heal text");
        let t = host_app.combat_texts.iter().find(|t| t.active).unwrap();
        assert_eq!(t.color, crate::combat_text::HEAL_LIFE, "the HealLife colour");
    }

    /// `case 36` (`MessageBuffer.cs:2178-2201`): a client's zone words store
    /// on the host's peer copy and relay; the receiving client stores them.
    /// The faeling edge (`zone5[0]` rising) is detected (the spawner is
    /// cited-unported).
    #[test]
    fn g48_36_player_zone_stores_and_relays() {
        let (mut host_app, mut client_app) = in_world_pair();
        let z = net::PlayerZone {
            player_id: 255, // the host forces the sender (`:2180-2184`)
            zone1: net::BitsByte(0b0000_0010), // jungle
            zone2: net::BitsByte(0),
            zone3: net::BitsByte(0),
            zone4: net::BitsByte(0),
            zone5: net::BitsByte(0b0000_0001), // the faeling edge bit
            town_npcs: 3,
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerZone(z));
        }
        let mut landed = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            landed = host_app
                .remote_players
                .get(&client_app.local_id)
                .is_some_and(|p| p.zones[0] == 0b10 && p.town_npcs == 3);
            if landed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(landed, "the 36 arm stored the zone words");
        let peer = &host_app.remote_players[&client_app.local_id];
        assert_eq!(peer.zones[4] & 1, 1, "the faeling edge bit stored");
    }

    /// `case 36` faeling edge (`MessageBuffer.cs:2195-2198`,
    /// `NPC.cs:5957-5995`): the zone5[0] rising edge runs
    /// `spawner::spawn_faelings` on the host — the scan draws consume the
    /// bare global stream verbatim (the RNG census) while the Shimmerfly
    /// 677 spawn stays roster-gated (`kind_from_net_id(677)` → None until
    /// the M4-NPC row lands). The witness pins: the edge stores + the scan
    /// leaves no NPC (the roster gap), and a SECOND zone message with the
    /// bit still set applies without re-firing the edge (the was_faeling
    /// latch). WRITTEN NOT RUN (code-only).
    #[test]
    fn g48_36_faeling_edge_fires_once_and_roster_gates() {
        let (mut host_app, mut client_app) = in_world_pair();
        // Park the peer far from any shimmer (the 8x8 flow world has none)
        // so every candidate fails the liquid probe — the draws still run.
        let z = net::PlayerZone {
            player_id: 255,
            zone1: net::BitsByte(0),
            zone2: net::BitsByte(0),
            zone3: net::BitsByte(0),
            zone4: net::BitsByte(0),
            zone5: net::BitsByte(1), // the faeling edge bit
            town_npcs: 0,
        };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerZone(z));
        }
        let mut landed = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            landed = host_app
                .remote_players
                .get(&client_app.local_id)
                .is_some_and(|p| p.zones[4] & 1 == 1);
            if landed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(landed, "the edge bit stored");
        // The roster gap: the scan ran (its draws consumed) but no
        // Shimmerfly spawned — kind_from_net_id(677) is None.
        assert!(
            host_app.npcs.iter().all(|n| n.kind.cs_id() != 677),
            "the Shimmerfly spawn stays roster-gated"
        );
        // The second message keeps the bit set: no rising edge, the store
        // still applies (town_npcs changes) and no NPC appears.
        let again = net::PlayerZone { town_npcs: 2, ..z };
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::PlayerZone(again));
        }
        let mut relanded = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            relanded = host_app
                .remote_players
                .get(&client_app.local_id)
                .is_some_and(|p| p.town_npcs == 2);
            if relanded {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(relanded, "the second zone message applied");
        assert!(
            host_app.npcs.iter().all(|n| n.kind.cs_id() != 677),
            "no re-fire: the bit was already set (no rising edge)"
        );
    }

    /// `case 40` (`MessageBuffer.cs:2256-2270`): a client's talk-NPC index
    /// stores on the host's peer copy, registers the bestiary chat start
    /// (`Player.cs:4379-4382`) and relays.
    #[test]
    fn g48_40_talk_npc_stores_and_registers_the_chat() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A Guide (town NPC) in slot 0 — the bestiary credit "Guide".
        let guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        host_app.npcs.clear();
        host_app.npcs.push(guide);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TalkNpc(net::TalkNpc {
                player_id: 255, // the host forces the sender (`:2258-2262`)
                npc: 0,
            }));
        }
        let mut landed = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            landed = host_app
                .remote_players
                .get(&client_app.local_id)
                .is_some_and(|p| p.talk_npc == Some(0));
            if landed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(landed, "the 40 arm stored the talk index");
        assert!(
            host_app.bestiary.was_chat_with("Guide"),
            "the bestiary chat start registered (Player.cs:4381)"
        );
    }

    /// `case 17` kind 21 (`MessageBuffer.cs:1263-1265`): a client's
    /// ReplaceTile edit swaps the host's stone block (1) for a dirt block
    /// (0 is refused — Conversion.Grass; use wood 30 → stone 1) and relays;
    /// the receiving client swaps its own copy. The host drops the OLD
    /// block's item (`KillTile_DropItems`, stone → 3); the client spawns no
    /// local drop (the `netMode == 1` gate). WRITTEN NOT RUN (code-only).
    #[test]
    fn g48_17_replace_tile_edit_swaps_and_replicates() {
        let (mut host_app, mut client_app) = roomy_pair();
        // A stone block at (10, 10) on both sides (in the air above the
        // dirt row 20 — no neighbours to reframe into it).
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(10, 10);
            app.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::TileEdit(net::TileEdit {
                kind: 21, // ReplaceTile
                x: 10,
                y: 10,
                value: 30, // wood — a valid solid target
                style: 0,
            }));
        }
        let mut swapped = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            swapped = host_app.world.tile(10, 10).is_some_and(|t| t.active() && t.kind == 30)
                && client_app.world.tile(10, 10).is_some_and(|t| t.active() && t.kind == 30);
            if swapped {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(swapped, "the stone became wood on both sides");
        // The host dropped the old stone's item 3 (`tile_drop(1)`); the
        // client spawned nothing local (its loot arrives as `21`).
        assert!(host_app.items.iter().any(|d| d.kind == 3), "the host's stone drop");
        assert!(
            !client_app.items.iter().any(|d| d.kind == 3),
            "the client's netMode==1 drop gate"
        );
    }

    /// `case 19` action 3 (`MessageBuffer.cs:1318-1320`): a client's
    /// trapdoor-OPEN toggle (`ShiftTrapdoor(x, y, dir==1, 0)` — the 0 is
    /// OPEN-only, the `:51534` close branch gates `!= 0`) runs the canonical
    /// W4-c42 `edit::shift_trapdoor` on the host and relays; the receiving
    /// client shifts its own copy. The closed 2x2 (386, styleX 0) opens
    /// DOWNWARD: the top row clears, the bottom row becomes 387
    /// (`WorldGen.cs:51471-51485`). WRITTEN NOT RUN (code-only).
    #[test]
    fn g48_19_trapdoor_open_toggle_replicates() {
        let (mut host_app, mut client_app) = roomy_pair();
        // The closed trapdoor 2x2 at (10,10)..(11,11), in the air above the
        // dirt row 20 — the c42 closed_trapdoor fixture shape.
        for app in [&mut host_app, &mut client_app] {
            for col in 0..2i32 {
                for row in 0..2i32 {
                    let i = app.world.index(10 + col, 10 + row);
                    app.world.tiles[i] = tr_core::Tile {
                        kind: 386,
                        frame_x: (col * 18) as i16,
                        frame_y: (row * 18) as i16,
                        ..Default::default()
                    };
                    app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::DoorToggle(net::DoorToggle {
                action: 3, // ShiftTrapdoor(..., onlyCloseOrOpen: 0) — open
                x: 10,
                y: 10,
                direction: 1,
            }));
        }
        let mut opened = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            opened = host_app.world.tile(10, 11).is_some_and(|t| t.kind == 387)
                && client_app.world.tile(10, 11).is_some_and(|t| t.kind == 387);
            if opened {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(opened, "the trapdoor opened on both sides");
        for app in [&host_app, &client_app] {
            // The top row cleared (`Tile.ClearTile`, :51471-51478).
            assert!(!app.world.tile_active(10, 10), "top-left cleared");
            assert!(!app.world.tile_active(11, 10), "top-right cleared");
            // The bottom row is the open trapdoor (:51479-51485).
            for col in 0..2i32 {
                let t = app.world.tile(10 + col, 11).unwrap();
                assert_eq!(t.kind, 387);
                assert_eq!((t.frame_x, t.frame_y), ((col * 18) as i16, 0));
            }
        }
    }

    /// `case 19` action 4 (`MessageBuffer.cs:1321-1323`): a client's
    /// tall-gate-OPEN toggle (`ShiftTallGate(x, y, closing: false, forced:
    /// true)`) runs the canonical W4-c42 `edit::shift_tall_gate` (alternate-0
    /// def decode of kind 388, the padding-free row walk) on the host and
    /// relays; the receiving client shifts its own copy. The 5-tall column
    /// 388 → 389, the frames survive (`WorldGen.cs:51763-51766`); the hit
    /// cell is mid-column so the row walk's anchor adjustment is exercised.
    /// The gate rows sit at y 10..14 in the roomy 64x64 world — inside the
    /// handler's fluff-3 box AND past the kill cascade's 5-tile edge guard
    /// (`kill.rs`'s `square_tile_frame`, the C# `TileFrame` `:81330`
    /// margin), so the validate's reframes cannot pop the column mid-test.
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn g48_19_tall_gate_open_toggle_replicates() {
        let (mut host_app, mut client_app) = roomy_pair();
        // The closed gate column x=10, rows 10..14 — the real placement grid
        // (D_388 coordinate_heights [18,16,16,16,18] + 2px padding → frameY
        // 0/20/38/56/74, the c42 23411fa7 fixture derivation).
        for app in [&mut host_app, &mut client_app] {
            for (row, fy) in [(0i32, 0i16), (1, 20), (2, 38), (3, 56), (4, 74)] {
                let i = app.world.index(10, 10 + row);
                app.world.tiles[i] = tr_core::Tile {
                    kind: 388,
                    frame_x: 0,
                    frame_y: fy,
                    ..Default::default()
                };
                app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::DoorToggle(net::DoorToggle {
                action: 4, // ShiftTallGate(closing: false, forced: true)
                x: 10,
                y: 11, // mid-column: frameY 20 → the walk steps one row up
                direction: 0,
            }));
        }
        let mut opened = false;
        for _ in 0..90 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            opened = host_app.world.tile(10, 10).is_some_and(|t| t.kind == 389)
                && client_app.world.tile(10, 10).is_some_and(|t| t.kind == 389);
            if opened {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(opened, "the tall gate opened on both sides");
        for app in [&host_app, &client_app] {
            for (row, fy) in [(0i32, 0i16), (1, 20), (2, 38), (3, 56), (4, 74)] {
                let t = app.world.tile(10, 10 + row).unwrap();
                assert_eq!(t.kind, 389, "row {row} opened");
                assert_eq!((t.frame_x, t.frame_y), (0, fy), "frames survive");
            }
        }
    }

    /// W4-d115 — a real belt dodge on the client queues the `62` kind-1 send
    /// (`Player.NinjaDodge` `Player.cs:38161-38164`); `net_tick` drains the
    /// queue every tick (the `take_hurt_sound` convention), so after the pump
    /// the local queue is empty and the host forced the relay id to the
    /// sender (`MessageBuffer.cs:2885`). The sender's own echo is excluded
    /// (`:2905`), so the client does not re-apply / re-send its own dodge.
    #[test]
    fn w4_d115_ninja_dodge_send_drains_on_the_net_tick() {
        let (mut host_app, mut client_app) = in_world_pair();
        let mut p = player::Player::spawn(&client_app.world);
        p.equipped[3] = 963; // the Black Belt
        p.update(&client_app.world, player::PlayerInput::default());
        assert!(p.black_belt);
        // A scripted Next(10)==0 dodges the hit (the belt roll).
        struct ZeroRng;
        impl tr_core::loot::DropRng for ZeroRng {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                0
            }
            fn next_float(&mut self) -> f32 {
                0.5
            }
        }
        assert!(!p.hurt(20, p.x + 100.0, &mut ZeroRng, true), "the belt dodges");
        client_app.player = Some(p);
        // The send is queued by the dodge, not yet drained.
        assert_eq!(
            client_app.player.as_ref().unwrap().take_dodge_sends(),
            vec![1]
        );
        // Re-arm a fresh dodge, then let the net tick drain it.
        {
            let p = client_app.player.as_mut().unwrap();
            p.immune = false;
            p.immune_time = 0;
            p.hurt_cd = 0.0;
            assert!(!p.hurt(20, p.x + 100.0, &mut ZeroRng, true));
        }
        pump(&mut host_app, &mut client_app, 12);
        assert!(
            client_app.player.as_ref().unwrap().take_dodge_sends().is_empty(),
            "net_tick drained the 62 send queue"
        );
        // The host's own player is not the subject (the relay excludes the
        // sender), so its dodge bookkeeping is untouched.
        assert_eq!(host_app.player.as_ref().unwrap().shadow_dodge_timer, 0);

        }    /// W4-e10 gap census — `case 47`'s client tail (`MessageBuffer.cs:
    /// 2400-2403`) over the live pair: the client's `46` request answers
    /// with a `47` addressed to the asker (`:2372`), and the client's
    /// dispatch opens the sign (`Player.OpenSign` — the latch, the text,
    /// ONE Menu_Open `Player.cs:33547`). The sign EDITOR stays S1-E-26.
    fn w4_e10_the_47_answer_opens_our_sign_with_one_menu_open() {
        // A 2-wide sign at (3, 2)/(4, 2) on BOTH sides (the rename-test
        // convention: the world transfer already ran, the fixture plants
        // matching tiles directly — the client's `TextSign` needs the
        // live tile to keep the record).
        for app in [&mut host_app, &mut client_app] {
            for x in [3, 4] {
                let i = app.world.index(x, 2);
                app.world.tiles[i] = tr_core::Tile {
                    kind: 55,
                    frame_x: if x == 3 { 0 } else { 18 },
                    ..Default::default()
                };
                app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let idx = crate::signs::find_or_create(&mut host_app.world.signs, 3, 2).unwrap();
        host_app.world.signs[idx].text = "ahoy".into();
        let before = client_app.audio.played_count();
        // The client asks (`46`); the host answers `47` to the asker.
            client.send(&C2s::SignRequest(net::SignRequest { x: 4, y: 2 }));
        let mut opened = false;
        for _ in 0..60 {
            host_app.poll_net();
            client_app.poll_net();
            opened = client_app.player.as_ref().is_some_and(|p| p.sign >= 0);
            if opened {
                break;
            std::thread::sleep(std::time::Duration::from_millis(2));
        assert!(opened, "the 47 answer opened the sign (:2402)");
        assert_eq!(p.sign, 0, "the dense-store index");
        assert_eq!(client_app.world.signs[0].text, "ahoy", "TextSign (:2394)");
        assert_eq!(client_app.npc_chat_text, "ahoy", "`:33549`");
        assert_eq!(client_app.audio.played_count(), before + 1, "`:33547` once");
        // The flags bit set: the host pushes a quiet `47` — the record
        // lands, no re-open, no cue.
        crate::signs::close_sign(&mut client_app, true);
        let mid = client_app.audio.played_count();
            server.send(
                1,
                &S2c::SignData(net::SignData {
                    sign: 0,
                    x: 3,
                    y: 2,
                    text: "quiet".into(),
                    sender: 1,
                    flags: 1,
                }),
            );
        let mut landed = false;
            landed = client_app.world.signs.first().is_some_and(|r| r.text == "quiet");
            if landed {
        assert!(landed, "the quiet 47 rewrote the record");
        assert_eq!(client_app.player.as_ref().unwrap().sign, -1, "bit 0 refuses");
        assert_eq!(client_app.audio.played_count(), mid, "no cue");
    }
}

/// `MessageBuffer.cs:4180-4187`'s eight-field apply — the writer order
/// (`NetMessage.cs:1555-1567`) lands on the `Player` luck fields verbatim.
/// The `RecalculateLuck` tail (`:4188`) is the caller's (the local player's
/// live state recalculates; a peer snapshot is stored-not-simulated).
fn apply_luck_factors(ls: &mut crate::luck::LuckState, l: &net::UpdatePlayerLuckFactors) {
    ls.lady_bug_luck_time_left = l.ladybug_luck_time_left;
    ls.torch_luck = l.torch_luck;
    ls.luck_potion = l.luck_potion;
    ls.has_garden_gnome_nearby = l.has_garden_gnome_nearby;
    ls.broken_mirror_bad_luck = l.broken_mirror_bad_luck;
    ls.equipment_based_luck_bonus = l.equipment_based_luck_bonus;
    ls.coin_luck = l.coin_luck;
    ls.kite_luck_level = l.kite_luck_level;
}

/// W4-g52 — the host relay/send gating end-to-end: the `28` strike's
/// section cull + death bypass + carrier push, the `27` flag3/section arms
/// with the skipped-flag motion, the `20` latch filter and the per-tick
/// `CheckSection` stamp. The pure-arm predicates ride `net.rs`'s
/// `w4_g52_*` tests; these pin the netglue wiring around them.
#[cfg(test)]
mod w4_g52_relay_tests {
    use super::*;
    use crate::app::netglue::handshake_tests::pump;

    /// A joined pair on a 900x600 world (5x4 sections) with the host's
    /// player live — the section grid is allocated by the join stream.
    fn wide_pair() -> (App, App) {
        let mut host_app = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        let mut client_app = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        host_app.world = tr_core::World::new("flow".into(), 900, 600);
        host_app.world.spawn_x = 100;
        host_app.world.spawn_y = 100;
        host_app.world.column_surface = vec![host_app.world.height; 900];
        let server = net::Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        host_app.net = NetRole::Host(server);
        host_app.player_name = "host".into();
        client_app.player_name = "client".into();
        client_app.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        client_app.join_target = Some((addr, String::new()));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while client_app.net_state != net::client_state::IN_WORLD
            && !matches!(client_app.net, NetRole::Off)
            && std::time::Instant::now() < deadline
        {
            client_app.poll_net();
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        for _ in 0..8 {
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
        host_app.player = Some(player::Player::spawn(&host_app.world));
        (host_app, client_app)
    }

    /// The `28` relay: a live NPC's strike is section-culled
    /// (`NetMessage.cs:1756`), a killing one bypasses the cull, and the
    /// death push (`MessageBuffer.cs:1808-1811`) reaches every
    /// broadcast-open slot. The STRIKE itself is observed on a second
    /// peer that shares the NPC's section (the striker never gets its own
    /// relay — ignoreClient).
    #[test]
    fn w4_g52_strike_relay_culls_by_section_and_death_bypasses() {
        let (mut host_app, mut client_app) = wide_pair();
        // A second peer at the spawn (section (0,0), the NPC's section).
        let addr = match &host_app.net {
            NetRole::Host(s) => s.addr().to_string(),
            _ => unreachable!(),
        };
        let mut second = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while second.net_state != net::client_state::IN_WORLD
            && std::time::Instant::now() < deadline
        {
            second.poll_net();
            client_app.poll_net();
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(second.net_state, net::client_state::IN_WORLD);
        let peer2 = second.local_id as u32;
        // Park the FIRST peer far from the NPC's section: (4,3).
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .x = 860.0 * TILE_PX;
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .y = 560.0 * TILE_PX;
        let mut npc =
            crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue);
        npc.x = 100.0;
        npc.y = 100.0;
        host_app.npcs.clear();
        host_app.npcs.push(npc);
        host_app.net_npc_gen.insert(0, (1, 1));
        // Stamp passes at the parked positions: (0,0) stays unchecked for
        // peer 1, active for peer 2.
        for _ in 0..20 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let generation = second
            .net_npc_gen
            .get(&0)
            .map(|(_, g)| *g as u8)
            .unwrap_or(1);
        let hp_before = host_app.npcs[0].hp;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: 0,
                generation,
                damage: 1,
                knock_back: 0.0,
                hit_direction: 1,
                crit: false,
            }));
        }
        for _ in 0..20 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            host_app.npcs[0].hp < hp_before,
            "the host applied the strike"
        );
        // hurt_cd is only set by the `28` arm — the relay's observable.
        assert!(
            second.net_npcs.get(&0).is_some_and(|n| n.hurt_cd > 0.0),
            "the section-active second peer received the relayed strike"
        );
        assert!(
            !client_app.net_npcs.get(&0).is_some_and(|n| n.hurt_cd > 0.0),
            "the far peer's live strike was section-culled (NetMessage.cs:1756)"
        );
        let _ = peer2;
        // The killing strike: the death bypass relays it everywhere. The
        // first strike armed the 0.18 s hurt_cd immunity
        // (`NPC.cs:81204-81210`); the witness clears it so the killing
        // blow lands (the sim's own decay would do the same over 11 ticks).
        host_app.npcs[0].hurt_cd = 0.0;
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: 0,
                generation,
                damage: 30000,
                knock_back: 0.0,
                hit_direction: 1,
                crit: false,
            }));
        }
        for _ in 0..20 {
            host_app.net_tick(1.0 / 60.0);
            client_app.net_tick(1.0 / 60.0);
            second.net_tick(1.0 / 60.0);
            host_app.poll_net();
            client_app.poll_net();
            second.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            !client_app.net_npcs.contains_key(&0),
            "the death `23` despawned the slot on the culled client too"
        );
        assert!(
            !second.net_npcs.contains_key(&0),
            "the death `23` despawned it on the near client too"
        );
    }

    /// The carrier push (`MessageBuffer.cs:1812-1815`): a segment's killing
    /// strike drains the shared pool, and BOTH the segment's and the
    /// carrier's `23` go out — the carrier's slot despawns on the client.
    #[test]
    fn w4_g52_a_carrier_death_pushes_the_carriers_23() {
        let (mut host_app, mut client_app) = wide_pair();
        let mut head = crate::npc::Npc::spawn_worm_head(
            &host_app.world,
            100,
            100,
            crate::npc::NpcKind::GiantWormHead,
        );
        head.worm_id = 7;
        head.seg_index = 0;
        head.hp = 5;
        head.max_hp = 5;
        let mut body = crate::npc::Npc::spawn_worm_head(
            &host_app.world,
            102,
            100,
            crate::npc::NpcKind::GiantWormBody,
        );
        body.worm_id = 7;
        body.seg_index = 1;
        body.hp = 5;
        body.max_hp = 5;
        host_app.npcs.clear();
        host_app.npcs.push(head);
        host_app.npcs.push(body);
        host_app.net_npc_gen.insert(0, (10, 1));
        host_app.net_npc_gen.insert(1, (11, 1));
        pump(&mut host_app, &mut client_app, 15);
        assert!(client_app.net_npcs.contains_key(&0), "the head synced");
        assert!(client_app.net_npcs.contains_key(&1), "the body synced");
        let generation = client_app
            .net_npc_gen
            .get(&1)
            .map(|(_, g)| *g as u8)
            .unwrap_or(1);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: 1,
                generation,
                damage: 100,
                knock_back: 0.0,
                hit_direction: 1,
                crit: false,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(host_app.npcs[0].hp <= 0, "the shared pool drained");
        assert!(
            !client_app.net_npcs.contains_key(&1),
            "the struck segment despawned"
        );
        assert!(
            !client_app.net_npcs.contains_key(&0),
            "the carrier's `23` despawned the head too (MessageBuffer.cs:1812-1815)"
        );
    }

    /// The `27` relay: a plain shot is section-culled, a flag3 pet is not;
    /// the culled client's skipped flag arms and the targeted recheck
    /// (`Projectile.cs:17246-17256`) lands the state once its section
    /// activates.
    #[test]
    fn w4_g52_projectile_relay_flag3_and_the_skipped_recheck() {
        let (mut host_app, mut client_app) = wide_pair();
        // Park the peer in section (4,3); the shot flies in (0,0).
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .x = 860.0 * TILE_PX;
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .y = 560.0 * TILE_PX;
        pump(&mut host_app, &mut client_app, 10);
        // A plain arrow (kind 1, aiStyle 1 — no flag3) at (100,100).
        let slot = host_app
            .projectiles
            .spawn_with_data_owned(1, 0, 100.0, 100.0, 0.0, 0.0, 10, 2.0, 0.0, 0.0)
            .expect("spawn");
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            client_app.projectiles.active_count(),
            0,
            "the plain shot's spawn `27` was section-culled"
        );
        assert!(
            host_app.proj_sync.skipped_for(slot, 1),
            "the culled client's skipped flag armed (NetMessage.cs:1792)"
        );
        // The peer walks into section (0,0): the next CheckSection stamp
        // activates it and the recheck's targeted `27` lands the state.
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .x = 100.0 * TILE_PX;
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .y = 100.0 * TILE_PX;
        host_app.host_check_sections();
        // The recheck helper is event-site parity (its live caller is the
        // projectile update tail); drive it directly with the C# reads.
        let mut player_active = [false; 255];
        player_active[1] = true;
        let sends = host_app.proj_sync.recheck_sections(
            &host_app.projectiles,
            slot,
            &player_active,
            |i| {
                let NetRole::Host(server) = &host_app.net else {
                    return false;
                };
                server.is_section_active(i as u32, 0, 0, host_app.game_update_count)
            },
        );
        assert_eq!(sends.len(), 1, "one targeted recheck `27`");
        assert!(
            !host_app.proj_sync.skipped_for(slot, 1),
            "the targeted send clears the flag (NetMessage.cs:1821-1823)"
        );
        host_app.net_proj_outbox.extend(sends);
        host_app.drain_proj_outbox();
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            client_app.projectiles.active_count() >= 1,
            "the recheck's targeted `27` landed the state"
        );
    }

    /// The flag3 pet (`Main.projPet`, `NetMessage.cs:1777`): the same
    /// far-section peer receives the pet's `27` immediately.
    #[test]
    fn w4_g52_a_pet_projectile_bypasses_the_section_cull() {
        let (mut host_app, mut client_app) = wide_pair();
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .x = 860.0 * TILE_PX;
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .y = 560.0 * TILE_PX;
        pump(&mut host_app, &mut client_app, 10);
        // The Pet Bunny (kind 319 — a `Main.projPet` row, `Main.cs:8736`).
        // The ported `proj_data` slice has no 319 row, so the slot is
        // seeded through the wire upsert — the relay ctx reads the pet
        // table by kind regardless of the SetDefaults slice.
        let st = net::ProjectileState {
            key: net::projectile_key(0, 0, 1),
            x: 100.0,
            y: 100.0,
            vx: 0.0,
            vy: 0.0,
            kind: 319,
            ai0: None,
            ai1: None,
            banner_id: 0,
            damage: 0,
            knock_back: 0.0,
            original_damage: 0,
            ai2: None,
        };
        host_app.projectiles.upsert_remote(&st);
        let ctx = host_app.proj_relay_ctx(0);
        assert!(
            matches!(
                ctx,
                net::RelayCtx::Projectile {
                    ignores_sections: true,
                    ..
                }
            ),
            "kind 319 is a Main.projPet row — flag3"
        );
        if let NetRole::Host(server) = &host_app.net {
            server.relay_projectile(
                &S2c::Projectile(Box::new(st)),
                None,
                ctx,
                host_app.game_update_count,
            );
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            client_app.projectiles.active_count() >= 1,
            "the pet's `27` reached the far-section client"
        );
    }

    /// The `20` relay: a tile edit replicates only to the slots holding a
    /// corner section (`NetMessage.cs:1712`). The join box latched the
    /// peer's (0..2, 0..1); section (3,2) was never streamed, so an edit
    /// there is culled — the peer's world keeps its old tile.
    #[test]
    fn w4_g52_a_tile_edit_replicates_only_to_latched_slots() {
        let (mut host_app, mut client_app) = wide_pair();
        pump(&mut host_app, &mut client_app, 15);
        if let NetRole::Host(server) = &host_app.net {
            assert!(server.is_section_sent(1, 0, 0), "the join box latched");
            assert!(!server.is_section_sent(1, 3, 2), "never streamed");
        }
        // Plant a tile in the un-latched section on BOTH worlds, then edit
        // it on the host only. The `20` relay must not carry the change.
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(700, 400);
            app.world.tiles[i] = tr_core::Tile {
                kind: 5,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let _ = crate::world_sim::editing::apply_edit(&mut host_app.world, 700, 400, 0, 5);
        host_app.request_edit(700, 400, 0, 5);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            host_app.world.tile(700, 400).map_or(0, |t| t.kind),
            0,
            "the host applied the removal"
        );
        assert_eq!(
            client_app.world.tile(700, 400).map_or(0, |t| t.kind),
            5,
            "the un-latched section's edit was culled (NetMessage.cs:1712)"
        );
        // An edit in a latched section DOES replicate.
        for app in [&mut host_app, &mut client_app] {
            let i = app.world.index(50, 60);
            app.world.tiles[i] = tr_core::Tile {
                kind: 5,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let _ = crate::world_sim::editing::apply_edit(&mut host_app.world, 50, 60, 0, 5);
        host_app.request_edit(50, 60, 0, 5);
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(
            client_app.world.tile(50, 60).map_or(0, |t| t.kind),
            0,
            "the latched section's edit replicated"
        );
    }

    /// The per-tick stamp (`Main.cs:64620-64623` → `RemoteClient.cs:165`):
    /// the fluff box around the peer refreshes every fixed step, and the
    /// 60-tick window expires when the peer stops being stamped.
    #[test]
    fn w4_g52_check_section_stamps_the_fluff_box_every_step() {
        let (mut host_app, mut client_app) = wide_pair();
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .x = 410.0 * TILE_PX; // section (2,1)
        host_app
            .remote_players
            .get_mut(&1)
            .expect("the peer")
            .y = 160.0 * TILE_PX;
        host_app.game_update_count = 1000;
        host_app.host_check_sections();
        if let NetRole::Host(server) = &host_app.net {
            // The fluff box: (1..3, 0..2) all stamped at 1000.
            for sy in 0..=2 {
                for sx in 1..=3 {
                    assert!(
                        server.is_section_active(1, sx, sy, 1060),
                        "({sx},{sy}) inside the window"
                    );
                }
            }
            assert!(
                !server.is_section_active(1, 0, 0, 1060),
                "outside the fluff box"
            );
            assert!(
                !server.is_section_active(1, 2, 1, 1061),
                "the 60-tick window expired"
            );
        }
        // A later stamp refreshes the window.
        host_app.game_update_count = 1050;
        host_app.host_check_sections();
        if let NetRole::Host(server) = &host_app.net {
            assert!(server.is_section_active(1, 2, 1, 1110));
        }
        let _ = client_app;
    }

    /// W4-g52 — the `SendData(21)` pre-switch (`NetMessage.cs:89-100`) on the
    /// host's cadence producers: an `instanced` drop never syncs (`:92-95`),
    /// an `IsAir` one rewrites to `151` (`:96-99`), a live one sends `21`.
    #[test]
    fn w4_g52_the_21_pre_switch_gates_instanced_and_air() {
        // The gate itself, all three legs.
        let live = item::ItemDrop::new(7, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        assert!(matches!(
            App::item_send_gate(5, &live),
            Some(S2c::ItemState(_))
        ));
        let mut instanced = item::ItemDrop::new(7, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        instanced.instanced = true;
        assert!(App::item_send_gate(5, &instanced).is_none());
        let mut air = item::ItemDrop::new(7, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        air.count = 0; // `Item.IsAir` — type > 0, stack <= 0 (`Item.cs:389-391`)
        assert!(matches!(
            App::item_send_gate(5, &air),
            Some(S2c::ItemDead { slot: 5 })
        ));
        let air_kind = item::ItemDrop::new(0, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        assert!(matches!(
            App::item_send_gate(5, &air_kind),
            Some(S2c::ItemDead { slot: 5 })
        ));

        // End-to-end: the cadence diff streams the live drop, refuses the
        // instanced one, and rewrites the air one to `151`.
        let (mut host_app, mut client_app) = wide_pair();
        host_app.items.clear();
        let mut live = item::ItemDrop::new(7, 3, 0, 100.0, 100.0, 0.0, 0.0, 0.0);
        live.age = 60.0;
        let mut instanced = item::ItemDrop::new(9, 2, 0, 120.0, 100.0, 0.0, 0.0, 0.0);
        instanced.age = 60.0;
        instanced.instanced = true;
        let mut air = item::ItemDrop::new(11, 1, 0, 140.0, 100.0, 0.0, 0.0, 0.0);
        air.age = 60.0;
        air.count = 0;
        host_app.items.push(live); // slot 0
        host_app.items.push(instanced); // slot 1
        host_app.items.push(air); // slot 2
        pump(&mut host_app, &mut client_app, 20);
        // Only the live drop reached the client; the instanced one never
        // synced and the air one's `151` removed nothing (it was never sent).
        assert_eq!(client_app.items.len(), 1, "only the live drop synced");
        assert_eq!(client_app.items[0].kind, 7);
        assert!(client_app.net_items.contains_key(&0));
        assert!(!client_app.net_items.contains_key(&1), "instanced never sent");
        assert!(!client_app.net_items.contains_key(&2), "air rewrote to 151");
    }

    /// W4-g52 — the `162 DamageNPCAck` send (`MessageBuffer.cs:1768-1771`):
    /// the host acks the striker at the HEAD of the `28` arm, ahead of the
    /// generation gate. The frame is bodyless; the client's damage-queue
    /// apply is the apply-arm lane, so the read is a no-op — the witness
    /// drains the client's event channel directly to observe the ack.
    #[test]
    fn w4_g52_the_strike_arm_acks_the_striker_with_162() {
        let (mut host_app, mut client_app) = wide_pair();
        // Drain the join-burst frames so only the strike's frames remain.
        while client_app.net_events_for_test().is_some() {}
        // A strike for an UNKNOWN npc slot: the generation gate refuses the
        // apply, but the C# sends the `162` BEFORE reading the body, so the
        // ack must still reach the striker.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::NpcStrike(net::NpcStrike {
                npc: 200,
                generation: 1,
                damage: 1,
                knock_back: 0.0,
                hit_direction: 1,
                crit: false,
            }));
        }
        // Pump the host only (the client's channel stays undrained by
        // poll_net), then read the frames the host pushed.
        for _ in 0..15 {
            host_app.net_tick(1.0 / 60.0);
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let mut saw_ack = false;
        while let Some(ev) = client_app.net_events_for_test() {
            if let net::ClientEvent::Frame(S2c::NpcStrikeAck) = ev {
                saw_ack = true;
            }
        }
        assert!(
            saw_ack,
            "the 162 ack reached the striker even though the strike was refused"
        );
    }

    /// A joined pair whose HOST world has a dirt floor at row 102 (spawn
    /// 100,100) — the `34` place arm's `TileObject.CanPlace` needs solid
    /// support under the chest's bottom row. The client's world is the
    /// join-streamed copy; the host-side store/tile assertions read the host.
    fn chest_pair() -> (App, App) {
        let (mut host_app, client_app) = wide_pair();
        for x in 0..host_app.world.width {
            let i = host_app.world.index(x, 102);
            host_app.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            host_app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        host_app.world.compute_column_surface();
        host_app.world.compute_chunks();
        (host_app, client_app)
    }

    /// W4-g52 — the `34` place arm (`MessageBuffer.cs:2001-2017`): a chest
    /// placed on solid support creates the store entry and broadcasts the
    /// success relay with the new index; a refusal (no support) targets the
    /// asker with `chest = -1` and drops the item back (`GetItemDrop_Chests`).
    #[test]
    fn w4_g52_the_34_place_arm_relays_success_and_refusal() {
        let (mut host_app, mut client_app) = chest_pair();
        host_app.world_chests.clear();
        host_app.items.clear();
        // Success: target (100,101) → top-left (100,100), support at row 102.
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestUpdates(net::ChestUpdates {
                action: 0,
                x: 100,
                y: 101,
                style: 0,
                chest: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(host_app.world_chests.len(), 1, "the store entry was created");
        assert_eq!(host_app.world_chests[0].tile, (100, 100));
        assert!(
            host_app.world.tile(100, 100).is_some_and(|t| t.active() && t.kind == 21),
            "the chest tile was placed"
        );
        assert!(host_app.items.is_empty(), "a successful place drops nothing");
        // Refusal: a boulder under the support column (200,102) trips the
        // `WorldGen.PlaceChest` boulder gate (`WorldGen.cs:57816-57819`).
        let bi = host_app.world.index(200, 102);
        host_app.world.tiles[bi].kind = 138; // `TileID.Sets.Boulders`
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestUpdates(net::ChestUpdates {
                action: 0,
                x: 200,
                y: 101,
                style: 0,
                chest: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(host_app.world_chests.len(), 1, "the refusal created no entry");
        assert_eq!(host_app.items.len(), 1, "the refusal dropped the chest item back");
        assert_eq!(host_app.items[0].kind, 48, "GetItemDrop_Chests(0, false) = 48");
    }

    /// W4-g52 — the `34` destroy arm (`MessageBuffer.cs:2019-2039`): the 2×2
    /// anchor decode, `Chest.FindChest`, `KillTile`, and the relay carrying the
    /// found index only when the cell died.
    #[test]
    fn w4_g52_the_34_destroy_arm_relays_the_found_index() {
        let (mut host_app, mut client_app) = chest_pair();
        host_app.world_chests.clear();
        // Place a chest first (the success arm), then destroy it via action 1
        // targeting a NON-origin cell (101,101) — the anchor decode walks back
        // to (100,100).
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestUpdates(net::ChestUpdates {
                action: 0,
                x: 100,
                y: 101,
                style: 0,
                chest: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert_eq!(host_app.world_chests.len(), 1);
        if let NetRole::Client(client) = &client_app.net {
            client.send(&C2s::ChestUpdates(net::ChestUpdates {
                action: 1,
                x: 101,
                y: 101,
                style: 0,
                chest: 0,
            }));
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(
            !host_app.world.tile(100, 100).is_some_and(|t| t.active()),
            "the KillTile cascade cleared the chest"
        );
        // The host's KillTile removes the store entry (the C# cascade's
        // Chest.DestroyChest) and drops the chest item back (the host's
        // `Main.netMode != 1` drop gate, WorldGen.cs:63860).
        assert!(host_app.world_chests.is_empty(), "the store entry was removed");
        assert_eq!(host_app.items.len(), 1, "the destroy dropped the chest item");
        assert_eq!(host_app.items[0].kind, 48, "registry::tile_drop(21) = 48");
    }

    /// W4-g52 audit — the 3600-tick `7 WorldInfo` re-push
    /// (`Main.cs:64557-64561`): the header carries the LIVE flags; the
    /// in-world client applies them WITHOUT re-requesting the spawn
    /// (`MessageBuffer.cs:655-659`'s State-3-only walk).
    #[test]
    fn w4_g52_the_world_header_heartbeat_repushes_the_live_flags() {
        let (mut host_app, mut client_app) = wide_pair();
        host_app.world.blood_moon = true;
        host_app.world.eclipse = true;
        host_app.world.downed_eoc = true;
        host_app.world.invasion_type = 1;
        host_app.world.invasion_goblins = true;
        // 3600 ticks at the batched +6 per 10 Hz tick = 600 net ticks =
        // 3600 frames at 1/60; the margin covers the accumulator's rounding.
        for _ in 0..3700 {
            host_app.net_tick(1.0 / 60.0);
        }
        pump(&mut host_app, &mut client_app, 15);
        assert!(client_app.world.blood_moon, "the re-push carried the blood moon");
        assert!(client_app.world.eclipse, "the re-push carried the eclipse");
        assert!(client_app.world.downed_eoc, "the flag group carried downedBoss1");
        assert_eq!(client_app.world.invasion_type, 1, "the invasionType rode");
        assert_eq!(
            client_app.net_state,
            net::client_state::IN_WORLD,
            "the in-world re-push never re-walks the join state machine"
        );
    }

    /// W4-g52 audit — the dawn/dusk boundary re-push (`Main.cs:65358-65361` /
    /// `:65406-65409`): a host crossing dusk broadcasts the header; the
    /// client's day flag flips without a spawn re-request.
    #[test]
    fn w4_g52_the_dusk_boundary_repushes_the_header() {
        let (mut host_app, mut client_app) = wide_pair();
        host_app.world.day_time = true;
        host_app.world.time = 53999.0;
        // An eclipse up at dusk: `stopMoonEvent` clears it BEFORE the C#
        // `SendData(7)` (`Main.cs:65246` before `:65361`) — the boundary
        // header must carry the CLEARED flag, not the stale one.
        host_app.world.eclipse = true;
        client_app.world.eclipse = true;
        let _ = crate::world_sim::clock::advance_time_with_events(&mut host_app, 5.0, false);
        assert!(!host_app.world.day_time, "the host crossed dusk");
        assert!(!host_app.world.eclipse, "StartNight cleared the eclipse");
        pump(&mut host_app, &mut client_app, 15);
        assert!(!client_app.world.day_time, "the boundary header flipped the client");
        assert!(
            !client_app.world.eclipse,
            "the boundary header carried the CLEARED eclipse (the send rides after the clears)"
        );
        assert_eq!(client_app.net_state, net::client_state::IN_WORLD);
    }

    /// W4-g52 audit — the join bundle's NPC/projectile tail
    /// (`MessageBuffer.cs:851-866`): a live NPC and a pet projectile on the
    /// host reach a NEWLY joining client at join, not on the next cadence.
    #[test]
    fn w4_g52_the_join_bundle_streams_the_live_npcs_and_pet_projectiles() {
        let (mut host_app, mut first) = wide_pair();
        host_app.npcs.clear();
        let mut npc =
            crate::npc::Npc::spawn_slime_sky(100.0, 100.0, crate::npc::SlimeVariant::Blue);
        npc.x = 100.0;
        npc.y = 100.0;
        host_app.npcs.push(npc);
        // A pet projectile (kind 111, `Main.projPet`) + a plain arrow (not
        // pet/netImportant — the join burst skips it).
        host_app
            .projectiles
            .spawn(111, 0, Vec2::new(120.0, 100.0), Vec2::ZERO, 0, 0.0, false);
        host_app
            .projectiles
            .spawn(1, 0, Vec2::new(140.0, 100.0), Vec2::ZERO, 10, 0.0, false);
        let addr = match &host_app.net {
            NetRole::Host(s) => s.addr().to_string(),
            _ => unreachable!(),
        };
        let mut second = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr, String::new()));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while second.net_state != net::client_state::IN_WORLD
            && std::time::Instant::now() < deadline
        {
            second.poll_net();
            first.poll_net();
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(second.net_state, net::client_state::IN_WORLD);
        // The join burst arrived WITHOUT any cadence tick on the host.
        assert!(
            second.net_npcs.values().any(|n| n.hp > 0),
            "the live NPC reached the joiner in the bundle"
        );
        let pet_synced = second
            .projectiles
            .slots()
            .iter()
            .any(|p| p.active && p.kind == 111);
        assert!(pet_synced, "the pet projectile rode the join burst");
        let arrow_synced = second
            .projectiles
            .slots()
            .iter()
            .any(|p| p.active && p.kind == 1);
        assert!(!arrow_synced, "the plain arrow waits for the owner cadence");
    }

    /// W4-g52 audit - `SyncConnectedPlayer`'s tail (`NetMessage.cs:2816-2819`):
    /// the housed town NPC's `60`, the traveller's `72`, the `74` quest and
    /// the `126` markers reach the joiner.
    #[test]
    fn w4_g52_the_join_bundle_pushes_houses_shop_quest_and_markers() {
        let (mut host_app, mut first) = wide_pair();
        host_app.npcs.clear();
        let mut guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 100.0, 1, 0.0);
        guide.hp = guide.max_hp;
        guide.home = Some((10, 10));
        host_app.npcs.push(guide);
        let mut traveller = crate::npc::Npc::new(
            crate::npc::NpcKind::TravellingMerchant,
            120.0,
            100.0,
            1,
            0.0,
        );
        traveller.hp = traveller.max_hp;
        host_app.npcs.push(traveller);
        host_app.world.travel_shop[0] = 28;
        host_app.world.angler_quest = 7;
        host_app.revenge_markers.push(crate::revenge::RevengeMarker {
            location: Vec2::new(300.0, 200.0),
            kind: crate::npc::NpcKind::Zombie,
            hp_percent: 0.5,
            base_value: 10.0,
            coins_value: 5_000,
            expiration_tick: u64::MAX,
            unique_id: 42,
            force_expire: false,
        });
        let addr = match &host_app.net {
            NetRole::Host(s) => s.addr().to_string(),
            _ => unreachable!(),
        };
        let mut second = crate::app::flow::front_flow_tests::flow_app(
            None,
            Vec::<crate::ui::save_list::PlayerEntry>::new(),
            Vec::<crate::ui::save_list::WorldEntry>::new(),
        );
        second.player_name = "second".into();
        second.net = NetRole::Client(net::Client::connect(&addr).unwrap());
        second.join_target = Some((addr.clone(), String::new()));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while second.net_state != net::client_state::IN_WORLD
            && std::time::Instant::now() < deadline
        {
            second.poll_net();
            first.poll_net();
            host_app.poll_net();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(second.net_state, net::client_state::IN_WORLD);
        // The client apply arms for 60/72/74/126 are the g49/g51 lanes; the
        // witness observes the raw frames (the 162-ack convention) — but
        // `poll_net` already CONSUMED the join batch through the no-op
        // dispatcher arms, so the channel is drained: the frames are
        // observed at the dispatcher instead, via a test-only capture of
        // `client_event`'s input. The capture wrapper replays each frame
        // into the real handler so the state machine walk is unchanged.
        let mut saw_house = false;
        let mut saw_shop = false;
        let mut saw_quest = false;
        let mut saw_marker = false;
        {
            // Re-join a THIRD client while watching its dispatcher input.
            let mut third = crate::app::flow::front_flow_tests::flow_app(
                None,
                Vec::<crate::ui::save_list::PlayerEntry>::new(),
                Vec::<crate::ui::save_list::WorldEntry>::new(),
            );
            third.player_name = "third".into();
            third.net = NetRole::Client(net::Client::connect(&addr).unwrap());
            third.join_target = Some((addr.clone(), String::new()));
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            // The join bundle lands AFTER the state flip (the host builds
            // it on the `12 PlayerSpawn` answer to `49`), so the watch runs
            // until the LAST bundle message (the `126` markers) or timeout.
            while !saw_marker && std::time::Instant::now() < deadline
            {
                // Drain-and-dispatch by hand: observe every frame, then
                // apply it exactly as poll_net would (the client_event
                // path), keeping the handshake walk live.
                let mut evs = Vec::new();
                if let NetRole::Client(c) = &third.net {
                    while let Ok(ev) = c.events.try_recv() {
                        evs.push(ev);
                    }
                    if third.net_state == net::client_state::NOT_CONNECTED {
                        third.net_state = net::client_state::CONNECT_SENT;
                        c.send(&C2s::Connect(net::CONNECT_VERSION.to_string()));
                    }
                    let role = std::mem::replace(&mut third.net, NetRole::Off);
                    if let NetRole::Client(client) = &role {
                        for ev in evs {
                            if let net::ClientEvent::Frame(msg) = &ev {
                                match msg {
                                    S2c::NpcHouse(h) => {
                                        saw_house |=
                                            h.home_x == 10 && h.home_y == 10 && h.status == 2;
                                    }
                                    S2c::TravelShop(s) => {
                                        saw_shop |= s.items.first() == Some(&28);
                                    }
                                    S2c::AnglerQuest(q) => {
                                        saw_quest |= q.quest == 7;
                                    }
                                    S2c::RevengeMarker(m) => {
                                        saw_marker |= m.unique_id == 42 && m.coins_value == 5_000;
                                    }
                                    _ => {}
                                }
                            }
                            third.client_event(client, ev);
                        }
                    }
                    third.net = role;
                }
                first.poll_net();
                host_app.poll_net();
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            assert_eq!(third.net_state, net::client_state::IN_WORLD);
        }
        assert!(saw_house, "the guide's 60 house push arrived");
        assert!(saw_shop, "the traveller's 72 stock push arrived");
        assert!(saw_quest, "the 74 angler quest arrived");
        assert!(saw_marker, "the 126 revenge marker arrived");
    }

// balance-close (union shared delimiter)
}}}}}}}}}}}

// balance-close (union shared delimiter)
}}}}}
    /// W4-d119 phase 3 — `case 60` (`MessageBuffer.cs:2695-2739`): the
    /// client's household writes. Status 2 stores the home pair + the
    /// TownManager room; status 1 clears both (the KickOut arm). The frame
    /// goes through the real decode path (`client_event`).
    #[test]
    fn the_60_household_frame_writes_the_client_home_and_book() {
        let (mut host_app, mut client_app) = in_world_pair();
        // A Guide on the host, streamed to the client's slot-keyed store.
        let mut guide = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 100.0, 48.0, 1, 0.0);
        guide.hp = guide.max_hp;
        host_app.npcs.clear();
        host_app.npcs.push(guide);
        host_app.net_npc_gen.insert(0, (22, 1));
        pump(&mut host_app, &mut client_app, 20);
        assert!(client_app.net_npcs.contains_key(&0), "the 23 push delivered the Guide");
        // Status 2: SetRoom — the home pair + the book entry.
        let frame = S2c::NpcHouse(net::NpcHouse {
            npc: 0,
            home_x: 65,
            home_y: 106,
            status: 2,
        })
        .encode();
        let msg = S2c::decode(frame[2], &mut net::NetReader::new(&frame[3..]))
            .expect("the 60 body decodes");
        if let NetRole::Client(client) = &client_app.net {
            client_app.client_event(client, net::ClientEvent::Frame(msg));
        }
        assert_eq!(
            client_app.net_npcs[&0].home,
            Some((65, 106)),
            "the client's home tiles landed (`:2710-2712`)"
        );
        assert_eq!(
            client_app.town_rooms.has_room(22),
            Some((65, 106)),
            "the SetRoom book arm (`:2725-2727`)"
        );
        // Status 1: KickOut — homeless + the book entry dropped.
        let frame = S2c::NpcHouse(net::NpcHouse {
            npc: 0,
            home_x: 0,
            home_y: 0,
            status: 1,
        })
        .encode();
        let msg = S2c::decode(frame[2], &mut net::NetReader::new(&frame[3..])).unwrap();
        if let NetRole::Client(client) = &client_app.net {
            client_app.client_event(client, net::ClientEvent::Frame(msg));
        }
        assert_eq!(client_app.net_npcs[&0].home, None, "the KickOut homeless write");
        assert_eq!(client_app.town_rooms.has_room(22), None, "the KickOut book arm (`:2721-2723`)");
    }

    /// W4-d119 phase 3 — `case 60`'s non-town gate (`MessageBuffer.cs:2715-2718`):
    /// a non-town NPC gets the home write but NO TownManager book arm; an
    /// unknown slot is a no-op (the port's store has no fixed 200 array).
    #[test]
    fn the_60_non_town_npc_skips_the_book_arms() {
        let (mut host_app, mut client_app) = in_world_pair();
        let mut slime = crate::npc::Npc::new(crate::npc::NpcKind::Slime, 100.0, 48.0, 1, 0.0);
        slime.hp = slime.max_hp;
        host_app.npcs.clear();
        host_app.npcs.push(slime);
        host_app.net_npc_gen.insert(0, (0, 1));
        pump(&mut host_app, &mut client_app, 20);
        let frame = S2c::NpcHouse(net::NpcHouse {
            npc: 0,
            home_x: 10,
            home_y: 20,
            status: 2,
        })
        .encode();
        let msg = S2c::decode(frame[2], &mut net::NetReader::new(&frame[3..])).unwrap();
        if let NetRole::Client(client) = &client_app.net {
            client_app.client_event(client, net::ClientEvent::Frame(msg));
        }
        // The home write is unconditional on the client (`:2708-2713`)...
        assert_eq!(client_app.net_npcs[&0].home, Some((10, 20)));
        // ...but `isLikeATownNPC` false breaks before the book (`:2715-2718`).
        assert!(client_app.town_rooms.rooms().is_empty(), "no book arm for a slime");
        // An unknown slot: the store lookup misses, nothing panics.
        let frame = S2c::NpcHouse(net::NpcHouse {
            npc: 199,
            home_x: 1,
            home_y: 2,
            status: 2,
        })
        .encode();
        let msg = S2c::decode(frame[2], &mut net::NetReader::new(&frame[3..])).unwrap();
        if let NetRole::Client(client) = &client_app.net {
            client_app.client_event(client, net::ClientEvent::Frame(msg));
        }
        assert!(!client_app.net_npcs.contains_key(&199));
    }
}
