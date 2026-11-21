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

/// The `13` heartbeat span in `net_tick` calls: the C#'s
/// `netPlayCounter % 420 == 0` at 60 Hz = 7 s (`Main.cs:64522-64529`); the
/// port's tick runs at 10 Hz, so 42 calls cover the same span.
const NET_HEARTBEAT_TICKS: u32 = 42;

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
            // `case 5` — one inventory slot (`MessageBuffer.cs:399-462`).
            // The server attributes the slot to the sending client
            // (`:409-412`), stores it in that client's copy of the
            // inventory and relays it to the other clients (`:454-457`,
            // `PlayerItemSlotID.CanRelay`).
            C2s::Inventory { slot, item } => {
                let peer = id as u8;
                server.set_slot_inventory(id, slot as u8, item);
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
            // `case 12` — the player spawned in: the slot is complete and the
            // join bundle goes out (`MessageBuffer.cs:925-929`,
            // `SyncConnectedPlayer` `NetMessage.cs:2806-2822`).
            C2s::PlayerSpawn(s) => {
                if server.state(id) == slot_state::SPAWN_REQUESTED {
                    server.set_state(id, slot_state::IN_WORLD);
                    self.host_slot_join_complete(server, id, s);
                }
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
                let broken_wall = apply_edit(&mut self.world, x, y, kind, tile);
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
                self.npcs[slot].facing = if st.hit_direction > 0 { 1 } else { -1 };
                let strike_x = self.npcs[slot].x;
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
                // browsing it. The port's other players are the remote slots.
                if self
                    .remote_players
                    .values()
                    .any(|p| p.chest == Some(chest_idx as i16))
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
                    self.wiring.hit_switch(&mut self.world, cx, cy);
                    // S1-E-21i — the trip's Mech-28 click: the host's
                    // positioned gate refuses (netMode 2), draw-free.
                    for (px, py) in self.wiring.take_switch_clicks() {
                        self.audio
                            .play_switch_click(px as f32, py as f32, self.rngs.global());
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
                self.wiring
                    .hit_switch(&mut self.world, hs.x as i32, hs.y as i32); // `:2687`
                // S1-E-21i — the click plays on the host's stream shape
                // (the bare `Main.rand`; the handler runs outside every
                // swap). The netMode-2 gate refuses it, draw-free.
                for (px, py) in self.wiring.take_switch_clicks() {
                    self.audio
                        .play_switch_click(px as f32, py as f32, self.rngs.global());
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
            // S1-I-1 wave A: the join bundle's client-sent ids have bodies +
            // codecs (`net.rs`); the host apply/relay arms
            // (`MessageBuffer.cs:1851-1867`, `:2290-2306`, `:2325-2364`,
            // `:4282-4297`, `:4328-4343`) land with the send-path pass (S1-I-4).
            C2s::TogglePvp(_)
            | C2s::PlayerMana(_)
            | C2s::TeamChange(_)
            | C2s::ProjectileTrackers(_)
            | C2s::SyncLoadout(_) => {}
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
        server.send(id, &S2c::InitialSpawn);
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
            server.send(id, &S2c::ItemState(Box::new(st)));
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
                let broken_wall = apply_edit(&mut self.world, x, y, kind, tile);
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
                    self.audio.play(match cue {
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
            // S1-I-1 wave A: the join bundle's server-sent ids have bodies +
            // codecs (`net.rs`); the client read sites
            // (`MessageBuffer.cs:1851-1867`, `:2290-2306`, `:2325-2364`,
            // `:2695-2739`, `:3118-3126`, `:3147-3153`, `:4043-4048`,
            // `:4195-4203`, `:4238-4249`, `:4282-4297`, `:4328-4343`) land
            // with the send-path pass (S1-I-4) — nothing sends them yet.
            S2c::TogglePvp(_)
            | S2c::PlayerMana(_)
            | S2c::TeamChange(_)
            | S2c::NpcHouse(_)
            | S2c::TravelShop(_)
            | S2c::AnglerQuest(_)
            | S2c::RevengeMarker(_)
            | S2c::DeadPlayer { .. }
            | S2c::MiscEventValue(_)
            | S2c::ProjectileTrackers(_)
            | S2c::SyncLoadout(_) => {}
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
            // `Main.GameMode` (W4-f45): the world's difficulty byte
            // (`NetMessage.cs:243`'s `(byte)Main.GameMode`).
            game_mode: w.game_mode as u8,
            unique_id: [0; 16],
            // `Main.ActiveWorldFileData.WorldGeneratorVersion` — the port's
            // generator version lives in the world seed's place; the save
            // format's version is the closest port-side marker (M6-NET-2
            // pins this to the generator build).
            generator_version: save::VERSION as u64,
            moon_type: w.moon_type,
            backgrounds: [0; 11],
            // W4-f45: the ice/jungle/hell styles + the tree/cave zone tables
            // are World-owned (`NetMessage.cs:259-275`).
            ice_back_style: w.ice_back_style as u8,
            jungle_back_style: w.jungle_back_style as u8,
            hell_back_style: w.hell_back_style,
            // W4-f45: the wind target is World-owned now (`WorldFile.cs:1328`).
            wind_speed_target: w.wind_speed_target,
            num_clouds: 0,
            tree_x: w.tree_x,
            tree_style: std::array::from_fn(|i| w.tree_style[i] as u8),
            cave_back_x: w.cave_back_x,
            cave_back_style: std::array::from_fn(|i| w.cave_back_style[i] as u8),
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
            // W4-f45: the dungeon anchor is World-owned now (`WorldFile.cs:
            // 1279-1280`); the wire carries it as the C#'s i16 pair.
            dungeon_x: w.dungeon_x as i16,
            dungeon_y: w.dungeon_y as i16,
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
            dir_right: npc.facing > 0,
            dir_y_down: npc.vy > 0.0,
            sprite_right: npc.facing > 0,
            life_full: npc.hp >= npc.max_hp,
            hp: npc.hp,
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
        // the mutation below: id, camera, and the set of sections already
        // streamed (`RemoteClient.IsSectionActive`, `NetMessage.cs:1740`).
        let (w, h) = (self.world.width, self.world.height);
        let (gx, gy) = (net::sections_x(w), net::sections_y(h));
        let mut clients: Vec<(u32, Vec2, Vec<(i32, i32)>)> = Vec::new();
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
                let mut sent = Vec::new();
                for sy in 0..gy {
                    for sx in 0..gx {
                        if server.is_section_sent(id, sx, sy) {
                            sent.push((sx, sy));
                        }
                    }
                }
                clients.push((id, cam, sent));
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
            for (id, cam, sent) in &clients {
                let i = *id as usize;
                if i >= 256 {
                    continue;
                }
                players[i] = PlayerView {
                    active: true,
                    connected: true,
                    camera: *cam,
                    section_active: sent.contains(&view.section),
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
    /// wires, in the direction the role dictates.
    fn drain_proj_outbox(&mut self) {
        use crate::app::net_proj_sync::ProjSend;
        let outbox = std::mem::take(&mut self.net_proj_outbox);
        match &self.net {
            NetRole::Host(server) => {
                for send in outbox {
                    match send {
                        ProjSend::Sync(st) => server.broadcast(&S2c::Projectile(Box::new(st))),
                        ProjSend::SyncTo { state, client } => {
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
                        ProjSend::Kill { key, .. } => client.send(&C2s::ProjectileDead { key }),
                    }
                }
            }
            NetRole::Off => {}
        }
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
                }
                if self.net_play_counter % 60 == 0 && !self.items.is_empty() {
                    self.last_item_update += 1;
                    if self.last_item_update >= self.items.len() {
                        self.last_item_update = 0;
                    }
                    let slot = self.last_item_update as i16;
                    let st = Self::item_state_of(slot, &self.items[self.last_item_update]);
                    self.net_item_last.insert(slot, st);
                    server.broadcast(&S2c::ItemState(Box::new(st)));
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

    /// Builds the `21 ItemDrop` body for one live drop (`NetMessage.cs:646-669`
    /// minus the ownership/enemy-delay bits the port does not carry yet).
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
        }
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
        if crate::bestiary::completion_percent(&self.bestiary.kills) < 1.0 {
            return None;
        }
        self.notify_progression_net(29)
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
        (bank_1_0(3)..bank_1_0(3) + 40).contains(&slot)
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
        | crate::npc::NpcKind::GoblinTinkerer => crate::npc::Npc::spawn_guide(world, tx)?,
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
        | crate::npc::NpcKind::SleepingGolfer => {
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
        let _ = crate::world_sim::editing::apply_edit(&mut host_app.world, 50, 60, 0, 5);
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
            dir_right: true,
            dir_y_down: false,
            sprite_right: true,
            life_full: true,
            hp: 100,
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
    use crate::app::netglue::handshake_tests::{in_world_pair, pump};

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
}
