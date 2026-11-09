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

/// `Main.multiplayerNPCSmoothingRange` (`Main.cs:1663`, default 300,
/// `Main.cs:67007`): the window inside which a networked position snap
/// accrues a visual lag instead of jumping. Both the players'
/// (`MessageBuffer.cs:1004-1008`) and the NPCs' (`:1636-1646`) smoothing use
/// it.
const SMOOTHING_RANGE: f32 = 300.0;

/// The `13` heartbeat span in `net_tick` calls: the C#'s
/// `netPlayCounter % 420 == 0` at 60 Hz = 7 s (`Main.cs:64522-64529`); the
/// port's tick runs at 10 Hz, so 42 calls cover the same span.
const NET_HEARTBEAT_TICKS: u32 = 42;

/// A client NPC slot silent past this is despawned: the host streams the live
/// set at 10 Hz and says nothing when an NPC dies, so the client prunes on
/// staleness (two missed cadence ticks).
const NPC_STALE: std::time::Duration = std::time::Duration::from_millis(200);
const TXT_WRONG_VERSION: &str = "You are not using the same version as this server.";
const TXT_BAD_PASSWORD: &str = "Incorrect password.";
const TXT_NAME_EMPTY: &str = "Name is empty.";
const TXT_NAME_TOO_LONG: &str = "Name is too long.";
const TXT_NAME_TAKEN: &str = "That name is taken.";

impl App {
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
            // The `23` despawn diff: the host streams every live NPC each
            // cadence tick, so a slot missing from this poll's batch is gone
            // (the C# equivalent turns life to 0 and keeps the slot).
            if !self.net_npc_seen.is_empty() {
                let seen: std::collections::HashSet<u32> = self.net_npc_seen.drain(..).collect();
                self.net_npcs.retain(|slot, _| seen.contains(slot));
                self.net_npcs_dirty = true;
            }
            // Staleness prune: a slot the host stopped streaming (it died, or
            // the host stopped simulating it) disappears after `NPC_STALE`.
            let now = Instant::now();
            let stale: Vec<u32> = self
                .net_npcs
                .keys()
                .copied()
                .filter(|slot| {
                    self.net_npcs_seen_at
                        .get(slot)
                        .is_some_and(|t| now.duration_since(*t) >= NPC_STALE)
                })
                .collect();
            if !stale.is_empty() {
                for slot in stale {
                    self.net_npcs.remove(&slot);
                    self.net_npcs_seen_at.remove(&slot);
                }
                self.net_npcs_dirty = true;
            }
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
                    self.host_stream_sections(server, id, sx, sy);
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
            // `case 16` — the health report; the server forces the subject to
            // the sender (`MessageBuffer.cs:1100-1103`).
            C2s::PlayerHp { hp, max_hp, .. } => {
                if let Some(p) = self.remote_players.get_mut(&id) {
                    p.hp = hp as i32;
                    p.max_hp = (max_hp as i32).max(20);
                }
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
                apply_edit(&mut self.world, x, y, kind, tile);
                self.world_key = None;
                // The replicated change: the applied tile read back as a
                // `20 SectionRange` (`SendTileSquare` on the host).
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
                let npc = &mut self.npcs[slot];
                npc.facing = if st.hit_direction > 0 { 1 } else { -1 };
                npc.hurt(st.damage.max(0) as i32, npc.x, st.knock_back, st.crit);
                let dead = self.npcs[slot].hp <= 0;
                // The relay carries the validated values to everyone else.
                server.broadcast_except(&S2c::NpcStrike(st), id);
                if dead {
                    // The full `23` update on death (`MessageBuffer.cs:1808-1811`).
                    let kind = self.npcs[slot].kind.to_u16() as i16;
                    let generation = self
                        .net_npc_gen
                        .get(&(slot as u32))
                        .map(|(_, g)| *g)
                        .unwrap_or(0);
                    let npc = &self.npcs[slot];
                    server.broadcast(&S2c::NpcInfo(Box::new(net::NpcInfo {
                        slot: st.npc,
                        generation: generation as u8,
                        spawn: false,
                        kind,
                        x: npc.x,
                        y: npc.y,
                        vx: npc.vx,
                        vy: npc.vy,
                        dir_right: npc.facing > 0,
                        dir_y_down: npc.vy > 0.0,
                        sprite_right: npc.facing > 0,
                        life_full: false,
                        hp: npc.hp,
                    })));
                }
            }
            // `case 27` — a peer's own projectile state. The host applies it
            // into its pool (rendering; the owner-side damage arrives as
            // `28`) and relays it to the others
            // (`MessageBuffer.cs:1760-1763`).
            C2s::Projectile(ps) => {
                self.projectiles.upsert_remote(&ps);
                server.broadcast_except(&S2c::Projectile(ps.clone()), id);
            }
            // `case 29` — a peer's projectile is gone; the host drops the slot
            // and relays (`MessageBuffer.cs:1844-1847`).
            C2s::ProjectileDead { key } => {
                let (owner, slot, generation) = net::projectile_key_parts(key);
                self.projectiles.kill_remote(slot, owner, generation);
                server.broadcast_except(
                    &S2c::ProjectileDead {
                        key,
                        x: f32::NAN,
                        y: f32::NAN,
                    },
                    id,
                );
            }
            C2s::Pickup { drop } => {
                // The client's pickup request (the port's stand-in for the
                // C# `SyncItem`→`151` path — `WorldItem.cs:612-615`). The host
                // validates the range, removes the slot and announces the
                // removal as `151`; the indices shift, and the next cadence
                // tick's `21` diff streams the shifted slots.
                let center = self
                    .remote_players
                    .get(&id)
                    .map(|p| Vec2::new(p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5));
                if let Some(center) = center {
                    if let Some(c) = self.items.get(drop as usize).map(|it| it.center()) {
                        if (c - center).length() < 60.0 {
                            self.items.remove(drop as usize);
                            server.broadcast(&S2c::ItemDead { slot: drop as i16 });
                        }
                    }
                }
            }
        }
    }

    /// The join bundle when a slot reaches `IN_WORLD` (the port's
    /// `SyncConnectedPlayer`, `NetMessage.cs:2806-2822`): the newcomer learns
    // the peers, the peers learn the newcomer.

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
        server.send(id, &S2c::WorldComplete);
    }

    /// The move-driven section catch-up (the C#'s per-tick
    /// `RemoteClient.CheckSection`, `RemoteClient.cs:126-188`): the sections
    /// each remote peer lacks, collected without borrowing `self` mutably
    /// (the send pass runs inside the `net_tick` match).
    fn pending_sections(&self, server: &net::Server) -> Vec<(u32, i32, i32)> {
        let (w, h) = (self.world.width, self.world.height);
        let (gx, gy) = (net::sections_x(w), net::sections_y(h));
        let mut out = Vec::new();
        for (id, p) in &self.remote_players {
            server.ensure_sections(*id, w, h);
            let psx = net::section_x(p.x as i32 / tr_core::TILE_PX as i32).clamp(0, gx - 1);
            let psy = net::section_y(p.y as i32 / tr_core::TILE_PX as i32).clamp(0, gy - 1);
            for sy in (psy - 1).max(0)..=(psy + 1).min(gy - 1) {
                for sx in (psx - 1).max(0)..=(psx + 1).min(gx - 1) {
                    if !server.is_section_sent(*id, sx, sy) {
                        out.push((*id, sx, sy));
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
    fn tile_record_at(&self, x: i32, y: i32) -> net::TileRecord {
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
            // `case 49` — the world stream is done (`MessageBuffer.cs:2448-2452`):
            // install in place (the port's `Player.Spawn(SpawningIntoWorld)`)
            // and announce the spawn with `12 PlayerSpawn`, which completes
            // the server slot (`MessageBuffer.cs:925-929`).
            S2c::WorldComplete => {
                if self.net_state == net::client_state::SPAWN_REQUESTED {
                    let sections = std::mem::take(&mut self.net_pending_sections);
                    if self.client_install_world(sections) {
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
                        self.net_disconnect("the world failed to install");
                    }
                }
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
            }
            S2c::Time { time, day } => {
                self.world.time = time;
                self.world.day_time = day;
            }
            S2c::Edit { x, y, kind, tile } => {
                apply_edit(&mut self.world, x, y, kind, tile);
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
            // `case 29` — the projectile is gone; a finite position means the
            // slot dies there, NaN deactivates it silently
            // (`MessageBuffer.cs:1825-1850`).
            S2c::ProjectileDead { key, .. } => {
                let (owner, slot, generation) = net::projectile_key_parts(key);
                self.projectiles.kill_remote(slot, owner, generation);
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
                }
            }
            // `case 23` — one NPC's state. The client keeps a slot-keyed
            // store (`net_npcs`) and recreates a slot only on a kind change or
            // the spawn flag (the C#'s `generation`/`spawnNeedsSyncing` test,
            // `MessageBuffer.cs:1602-1613`); the smoothing is the same
            // netOffset accumulator (`MessageBuffer.cs:1636-1646`). Seen
            // slots feed the despawn diff in `poll_net`.
            S2c::NpcInfo(info) => {
                let slot = info.slot as u32;
                self.net_npc_seen.push(slot);
                self.net_npcs_seen_at.insert(slot, Instant::now());
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
                if !info.life_full && info.hp >= 0 {
                    npc.hp = info.hp;
                }
                self.net_npcs.insert(slot, npc);
                self.net_npcs_dirty = true;
            }
            // `case 21` — one drop's state, keyed by the host slot index
            // (the port's item ids — the C# fixed-400 scheme is M6-NET-5).
            S2c::ItemState(d) => {
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
                drop.shimmered = d.shimmered;
                drop.shimmer_time = d.shimmer_time;
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
        self.map = Some(map::WorldMap::new(&self.world));
        self.npcs.clear();
        self.items.clear();
        self.world_key = None;
        self.net_info = None;
        self.net_pending_sections.clear();
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
    /// The `27` diff pass for this machine's own projectiles: the states that
    /// changed since the last send, gated by the C#'s send budget
    /// (`Projectile.cs:17224-17238`: budget 60, cost 5, refill 1/tick — the
    /// net tick runs at 10 Hz, so the refill is 10 here), plus the keys that
    /// left the locally-simulated set (the `29` despawns). The caller sends
    /// the plan in the direction its role dictates.
    fn projectile_sync_plan(&mut self) -> (Vec<net::ProjectileState>, Vec<i32>) {
        for v in self.net_proj_spam.values_mut() {
            *v = (*v - 10).max(0);
        }
        let local = self.local_id as u8;
        let mut current: Vec<net::ProjectileState> = Vec::new();
        for slot in 0..crate::projectile::MAX_PROJECTILES {
            let p = &self.projectiles.slots()[slot];
            if !p.active || p.owner != local {
                continue;
            }
            current.push(net::ProjectileState {
                key: self.projectiles.key_of(slot),
                x: p.pos.x,
                y: p.pos.y,
                vx: p.vel.x,
                vy: p.vel.y,
                kind: p.kind as i16,
                ai0: if p.ai[0] != 0.0 { Some(p.ai[0]) } else { None },
                ai1: if p.ai[1] != 0.0 { Some(p.ai[1]) } else { None },
                banner_id: 0,
                damage: p.damage.clamp(0, i16::MAX as i32) as i16,
                knock_back: p.knock_back,
                original_damage: 0,
                ai2: if p.ai[2] != 0.0 { Some(p.ai[2]) } else { None },
            });
        }
        let mut sends = Vec::new();
        for st in &current {
            if self.net_proj_last.get(&st.key) == Some(st) {
                continue;
            }
            let budget = self.net_proj_spam.entry(st.key).or_insert(0);
            if *budget >= 60 {
                continue; // parked until the budget frees (`netUpdate2`)
            }
            *budget += 5;
            sends.push(*st);
            self.net_proj_last.insert(st.key, *st);
        }
        let mut dead = Vec::new();
        for key in self.net_proj_last.keys().copied().collect::<Vec<_>>() {
            if !current.iter().any(|st| st.key == key) {
                dead.push(key);
            }
        }
        for key in &dead {
            self.net_proj_last.remove(key);
            self.net_proj_spam.remove(key);
        }
        (sends, dead)
    }

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
            ..Default::default()
        };
        // The ambience spawn relay (`AmbienceServer.SpawnForPlayer`): the
        // (type, seed) pairs leave the store before the role borrow.
        let ambience_spawns = self.take_unsent_ambience();
        // The `27` diff for this machine's own projectiles, sent in the
        // direction the role dictates below.
        let (proj_sends, proj_dead) = self.projectile_sync_plan();
        // The `82` module cadence (the 1 Hz ping on a client).
        self.net_module_tick();
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
                // The `27` broadcast for the host's own projectiles and the
                // `29` despawns (`Projectile.cs:17224-17238`).
                for st in &proj_sends {
                    server.broadcast(&S2c::Projectile(Box::new(*st)));
                }
                for key in &proj_dead {
                    server.broadcast(&S2c::ProjectileDead {
                        key: *key,
                        x: f32::NAN,
                        y: f32::NAN,
                    });
                }
                // Newly approached sections stream before the cadence batch
                // (the C#'s per-tick CheckSection).
                for (id, sx, sy) in self.pending_sections(server) {
                    server.mark_section_sent(id, sx, sy);
                    let block = self.tile_block_at(sx, sy);
                    server.send(id, &S2c::TileBlock(Box::new(block)));
                }
                // `23 NPCInfo` — one message per live NPC, the C#'s per-NPC
                // `SendData(23, -1, -1, null, whoAmI)` (`NPC.cs:90613`). The
                // port sends the whole live set each cadence tick (the C#'
                // `netSpam` per-NPC gate and the section culling are the
                // M6-NET-3 follow-ups), which doubles as the despawn feed.
                for (i, npc) in self.npcs.iter().enumerate() {
                    let slot = i as u32;
                    let kind = npc.kind.to_u16() as i16;
                    let (spawn, generation) = match self.net_npc_gen.get(&slot) {
                        Some((k, g)) if *k == kind => (false, *g),
                        _ => {
                            let g = self
                                .net_npc_gen
                                .get(&slot)
                                .map(|(_, g)| g.wrapping_add(1))
                                .unwrap_or(1);
                            (true, g)
                        }
                    };
                    self.net_npc_gen.insert(slot, (kind, generation));
                    server.broadcast(&S2c::NpcInfo(Box::new(net::NpcInfo {
                        slot: slot as u8,
                        generation: generation as u8,
                        spawn,
                        kind,
                        x: npc.x,
                        y: npc.y,
                        vx: npc.vx,
                        vy: npc.vy,
                        dir_right: npc.facing > 0,
                        dir_y_down: npc.vy > 0.0,
                        sprite_right: npc.facing > 0,
                        life_full: npc.hp >= npc.max_hp,
                        hp: npc.hp,
                    })));
                }
                // Prune the generation bookkeeping of slots that died.
                self.net_npc_gen
                    .retain(|slot, _| (*slot as usize) < self.npcs.len());
                // `21 ItemDrop` / `151` — only the slots whose state changed
                // since the last send, plus the vanished tail (the index
                // shift a removal causes). The C#'s default broadcast has no
                // per-item gate either (`NetMessage.cs:1801-1811`).
                let live = self.items.len() as i16;
                for (i, it) in self.items.iter().enumerate() {
                    let slot = i as i16;
                    let st = net::ItemState {
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
                    };
                    if self.net_item_last.get(&slot) != Some(&st) {
                        self.net_item_last.insert(slot, st);
                        server.broadcast(&S2c::ItemState(Box::new(st)));
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
                // The client's own projectiles go to the host as `27`
                // (`Projectile.cs:17210-17244`'s owner machine) and its
                // despawns as `29`.
                for st in &proj_sends {
                    client.send(&C2s::Projectile(Box::new(*st)));
                }
                for key in &proj_dead {
                    client.send(&C2s::ProjectileDead { key: *key });
                }
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
        }
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
            let step = (len * 0.1).max(2.0) * scale;
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
            let step = (2.0 + 2.0 * len / SMOOTHING_RANGE) * scale;
            let k = ((len - step) / len).max(0.0);
            npc.net_offset *= k;
            if npc.net_offset.length() < 0.5 {
                npc.net_offset = Vec2::ZERO;
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

    /// A client-side strike on a networked NPC: the `28` body addressed by the
    /// host's slot and the generation the `23` push delivered
    // (`NPC.StrikeNPC`'s `fromNet: false` send, `NPC.cs:81218`).
    pub(crate) fn send_npc_strike(
        &self,
        idx: usize,
        damage: i32,
        knock_back: f32,
        crit: bool,
        from_right: bool,
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

    /// The stored `13` snapshot's facing (`flags_a` bit 6).
    fn net_ps_last_facing(&self) -> i32 {
        if self.net_ps_last.flags_a.get(6) {
            1
        } else {
            -1
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
            NetRole::Host(server) => server.broadcast(&S2c::SectionRange(Box::new(range))),
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
        | crate::npc::NpcKind::GoblinTinkerer => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::EyeOfCthulhu => {
            let y = (y / TILE_PX) as i32;
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
        | crate::npc::NpcKind::BoundMechanic => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::EaterOfSouls
        | crate::npc::NpcKind::Hornet
        | crate::npc::NpcKind::MeteorHead => {
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
        | crate::npc::NpcKind::DD2Bartender
        | crate::npc::NpcKind::DD2EterniaCrystal
        | crate::npc::NpcKind::AncientCultist
        | crate::npc::NpcKind::StardustTower
        | crate::npc::NpcKind::NebulaTower
        | crate::npc::NpcKind::SolarTower
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
        | crate::npc::NpcKind::WaterSphere => {
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
        | crate::npc::NpcKind::LeechBody => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The eye and the servant: the ctor seeds the stats — the net state
        // overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::EyeOfCthulhu | crate::npc::NpcKind::ServantOfCthulhu => {
            let y = (y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
    })
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
    fn pump(host: &mut App, client: &mut App, frames: usize) {
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
    fn in_world_pair() -> (App, App) {
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
        crate::world_sim::editing::apply_edit(&mut host_app.world, 50, 60, 0, 5);
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

    #[test]
    fn the_ping_module_measures_an_rtt() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The client's 1 Hz ping cadence.
        pump(&mut host_app, &mut client_app, 40);
        assert!(
            client_app.net_rtt_ms < 10_000,
            "the ping echo came back: rtt {} ms",
            client_app.net_rtt_ms
        );
    }

    #[test]
    fn the_ambience_module_relays_a_spawn() {
        let (mut host_app, mut client_app) = in_world_pair();
        // The host force-spawns a sky entity; the module relay must land it on
        // the client (`AmbienceServer.SpawnForPlayer`).
        // The forced spawn is consumed by the ambience update (the frame's
        // ambience step), so run one step before the pump picks it up.
        {
            let mut amb = std::mem::take(&mut host_app.ambience);
            let mut rng = host_app.ambience_rng;
            amb.forced_spawn_for_tests(crate::ambience::SkyEntityType::Wyvern, -1, 12345);
            amb.update(1.0 / 60.0, &host_app.ambience_ctx(), &mut rng);
            host_app.ambience_rng = rng;
            host_app.ambience = amb;
        }
        pump(&mut host_app, &mut client_app, 25);
        assert!(
            !client_app.ambience.entities.is_empty(),
            "the client spawned the relayed entity: {}",
            client_app.ambience.entities.len()
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

    /// The own-projectile diff: the host's shot reaches the client's pool and
    /// its death comes back as `29`.
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
        let (_, proj_dead) = client_app.projectile_sync_plan();
        let (_, host_dead) = host_app.projectile_sync_plan();
        assert!(
            proj_dead.is_empty(),
            "the host's shot is not the client's to despwn"
        );
        assert!(host_dead.is_empty(), "the live shot is not dead");
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
}
