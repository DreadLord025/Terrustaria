//! Multiplayer for Terraria-Rust: the packet core plus the join handshake,
//! layered on a threaded TCP transport.
//!
//! The wire format is a port of the C# side: every frame is
//! `[u16 LE length][u8 message id][body]` with the length counting the whole
//! packet (minimum 3, maximum 65535 — `NetMessage.cs:1677-1684`, the
//! `CheckBytes` length check at `NetMessage.cs:2486-2556`, the `NetPacket`
//! header at `Terraria/Net/NetPacket.cs:19-42`). The join sequence — the
//! client states 0→10 (`Netplay.cs:459-544` plus the state changes in the
//! `MessageBuffer.cs` cases) and the server-side slot states with the `GetData`
//! accept gate (`MessageBuffer.cs:148-172`) — drives the handshake.
//!
//! Transport: a listener thread plus one reader and one writer thread per peer
//! (the C# `AsyncReceive`/`AsyncSend` callbacks, `Netplay.cs:469-543`), so the
//! render loop never blocks on a socket: it only drains typed messages through
//! an mpsc queue (`Netplay.UpdateServerInMainThread`/`UpdateClientInMainThread`,
//! `Netplay.cs:96-102`, `:333-339` — the C# does the same on its main thread).
//! Decoding and the state machines are *not* in the reader threads: the gate
//! needs the game-thread state, so frames arrive as typed messages and
//! `netglue` runs the transition tables.

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// `"Terraria" + 326` — the version literal both sides compare
/// (`NetMessage.cs:132-134` writes it, `MessageBuffer.cs:203` reads it).
pub const CONNECT_VERSION: &str = "Terraria326";

/// `Main.timeOut`-equivalent: a silent peer is dropped after this idle span.
/// The C# terminates at `TimeOutTimer > 7200` ticks (`Main.cs:64610-64618`
/// server, `:64540-64544` client) ≈ 2 minutes at 60 ticks/s.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(120);

/// `Netplay.MaxConnections = 256` (`Netplay.cs:189`'s server-full kick).
pub const MAX_CONNECTIONS: usize = 256;

/// `Player.nameLen` (`Player.cs:57105`) — the name check in `MessageBuffer.cs:375`.
pub const NAME_LEN: usize = 20;

// ===========================================================================
// Message ids
// ===========================================================================

/// The `MessageID` ids (`MessageBuffer.cs:181-4483`'s switch). M6-NET-1 ports
/// the handshake set; the entity/tile ids (10 TileBlock, 13 PlayerState,
/// 21 ItemDrop, 23 NPCInfo, …) land with M6-NET-2/3.
pub mod id {
    pub const CONNECT: u8 = 1; // MessageBuffer.cs:184-214 — client→server version string
    pub const DISCONNECT: u8 = 2; // MessageBuffer.cs:215-222 — server→client reason
    pub const PLAYER_ID: u8 = 3; // MessageBuffer.cs:225-283
    pub const PLAYER_INFO: u8 = 4; // MessageBuffer.cs:284-396
    pub const PLAYER_INVENTORY: u8 = 5; // MessageBuffer.cs:398-406
    pub const WORLD_REQUEST: u8 = 6; // MessageBuffer.cs:462-472 — no body
    pub const WORLD_INFO: u8 = 7; // MessageBuffer.cs:473-658 — the world header
    pub const SPAWN_REQUEST: u8 = 8; // MessageBuffer.cs:671-806
    pub const STATUS: u8 = 9; // MessageBuffer.cs:877-888 — load progress
    pub const PLAYER_ACTIVE: u8 = 14; // MessageBuffer.cs:1061-1088
    pub const TILE_BLOCK: u8 = 10; // NetMessage.cs:424-426 / MessageBuffer.cs:889-894
    pub const NET_MODULE: u8 = 82; // NetPacket.cs:19-42 / MessageBuffer.cs:3243-3245
    pub const NPC_STRIKE: u8 = 28; // NetMessage.cs:844-850 / MessageBuffer.cs:1766-1817
    pub const PROJECTILE: u8 = 27; // NetMessage.cs:780-841 / MessageBuffer.cs:1711-1763
    pub const PROJECTILE_DEAD: u8 = 29; // NetMessage.cs case 29 / MessageBuffer.cs:1825-1850
    pub const SECTION_RANGE: u8 = 20; // NetMessage.cs:543-644 / MessageBuffer.cs:1335-1443
    pub const PLAYER_HP: u8 = 16; // MessageBuffer.cs:1095-1112
    pub const PASSWORD_REQUEST: u8 = 37; // MessageBuffer.cs:2203-2216 — no body
    pub const PASSWORD: u8 = 38; // MessageBuffer.cs:2218-2231
    pub const PLAYER_SPAWN: u8 = 12; // MessageBuffer.cs:908-930
    /// `49 InitialSpawn` (`MessageID.cs:111`): the *client* spawns into the
    /// world - the last message of the join bundle
    /// (`MessageBuffer.cs:874`, handled at `:2447-2453`). The port used to call
    /// this `WORLD_COMPLETE` and read it as "world stream done"; the stream is
    /// in fact complete when the section walk ends, and this id only means
    /// `State 6 -> 10` + `Player.Spawn(SpawningIntoWorld)`.
    pub const INITIAL_SPAWN: u8 = 49;
    /// `154 Ping` (`MessageID.cs:322`): the RTT round trip -
    /// `Ping.Update` sends it every 250 ms, the server echoes it straight back
    /// (`MessageBuffer.cs:4419-4428`). Note the port used NetModule 2 for this
    /// before; module 2 is `NetPingModule`, the *map* ping relay.
    pub const PING: u8 = 154;
    pub const CHEST_OPEN_REQUEST: u8 = 31; // MessageBuffer.cs:1868-1895
    pub const CHEST_ITEM: u8 = 32; // MessageBuffer.cs:1897-1918 / NetMessage.cs:864-877
    pub const PLAYER_CHEST: u8 = 33; // MessageBuffer.cs:1920-1982 / NetMessage.cs:879-911
    pub const SIGN_REQUEST: u8 = 46; // MessageBuffer.cs:2366-2375
    pub const SIGN_DATA: u8 = 47; // MessageBuffer.cs:2376-2405 / NetMessage.cs:979-986
    pub const PLAYER_BUFFS: u8 = 50; // MessageBuffer.cs:2454-2480 / NetMessage.cs:996-1008
    pub const ADD_NPC_BUFF: u8 = 53; // MessageBuffer.cs:2565-2575 / NetMessage.cs:1019-1022
    pub const NPC_BUFFS: u8 = 54; // MessageBuffer.cs:2577-2593 / NetMessage.cs:1024-1039
    pub const HIT_SWITCH: u8 = 59; // MessageBuffer.cs:2682-2693 / NetMessage.cs:1063-1066
    pub const PLAYER_CHEST_INDEX: u8 = 80; // MessageBuffer.cs:3216-3227 / NetMessage.cs:1175-1178
    pub const CHEST_SIZE: u8 = 155; // MessageBuffer.cs:4429-4438 / NetMessage.cs:1650-1653
    pub const CHEST_NAME: u8 = 69; // MessageBuffer.cs:3059-3078 / NetMessage.cs:1334-1341
    pub const PLAYER_STATE: u8 = 13; // NetMessage.cs:446-513 / MessageBuffer.cs:952-1059
    pub const ITEM_DROP: u8 = 21; // NetMessage.cs:646-669 / MessageBuffer.cs:1445-1535
    pub const ITEM_OWNER: u8 = 22; // NetMessage.cs:675-685 / MessageBuffer.cs:1552-1562
    pub const ITEM_DEAD: u8 = 151; // NetMessage.cs:671-674 / MessageBuffer.cs:1537-1550
    pub const NPC_INFO: u8 = 23; // NetMessage.cs:686-748 / MessageBuffer.cs:1563-1704
    /// `70 CatchNPC` (S1-P-i36): one `(short)` npc slot — the client's bug
    /// net caught it (`NPC.cs:79904`); the host converts
    /// (`MessageBuffer.cs:3102-3110`, `NetMessage.cs:1125-1127`).
    pub const CATCH_NPC: u8 = 70;
    /// `71 ReleaseNPC` (S1-P-i36): `(int)x, (int)y, (short)type, (byte)style`
    /// — a client released a captured critter (`NPC.cs:80111`); the host
    /// spawns (`MessageBuffer.cs:3112-3117`, `NetMessage.cs:1128-1133`).
    pub const RELEASE_NPC: u8 = 71;
    /// `97` — `AchievementsHelper.NotifyNPCKilled`'s per-player fan-out
    /// (`AchievementsHelper.cs:210-216`, `NetMessage.cs:1347-1349`): one
    /// `(short)npcNetId`, read only at `netMode == 1`
    /// (`MessageBuffer.cs:3558-3563`).
    pub const NPC_KILLED: u8 = 97;
    /// `98` — `AchievementsHelper.NotifyProgressionEvent`'s host broadcast
    /// (`AchievementsHelper.cs:227-237`, `NetMessage.cs:1350-1352`): one
    /// `(short)eventID`, read only at `netMode == 1`
    /// (`MessageBuffer.cs:3564-3569`).
    pub const PROGRESSION_EVENT: u8 = 98;

    // ---- S1-I-1 wave A: the join bundle's missing ids ----
    // The `SyncConnectedPlayer`/`SyncOnePlayer`/`SendNPCHousesAndTravelShop`/
    // `SendAnglerQuest`/`SendCreditsRollRemainingTimeToPlayer`/
    // `SendAllMarkersToPlayer` push (`NetMessage.cs:2806-2822`, `:2885-2968`,
    // `:2824-2852`). Bodies + codecs only; the send-path fan-out is S1-I-4.
    pub const TOGGLE_PVP: u8 = 30; // MessageID.cs:71 / NetMessage.cs:856-859 / MessageBuffer.cs:1851-1867
    /// `42 Unknown42` (`MessageID.cs:95`) — the mana sync
    /// (`NetMessage.cs:961-965`, `MessageBuffer.cs:2290-2306`).
    pub const PLAYER_MANA: u8 = 42;
    pub const TEAM_CHANGE: u8 = 45; // MessageID.cs:102 / NetMessage.cs:970-974 / MessageBuffer.cs:2325-2364
    /// `60 Unknown60` (`MessageID.cs:133`) — the NPC household banner
    /// (`NetMessage.cs:1067-1072`, `MessageBuffer.cs:2695-2739`; sent by
    /// `SendNPCHousesAndTravelShop`, `NetMessage.cs:2845`).
    pub const NPC_HOUSE: u8 = 60;
    pub const TRAVEL_SHOP: u8 = 72; // MessageID.cs:157 / NetMessage.cs:1134-1141 / MessageBuffer.cs:3118-3126
    pub const ANGLER_QUEST: u8 = 74; // MessageID.cs:161 / NetMessage.cs:1145-1148 / MessageBuffer.cs:3147-3153
    pub const REVENGE_MARKER: u8 = 126; // MessageID.cs:266 / NetMessage.cs:1515-1517 / MessageBuffer.cs:4043-4048
    pub const DEAD_PLAYER: u8 = 135; // MessageID.cs:284 / NetMessage.cs:1569-1571 / MessageBuffer.cs:4195-4203
    /// `140 SetMiscEventValues` (`MessageID.cs:294`) — kind 0 is the credits
    /// roll's remaining time (`NetMessage.cs:1591-1594`,
    /// `CreditsRollEvent.cs:27-33`, `MessageBuffer.cs:4238-4249`).
    pub const MISC_EVENT_VALUES: u8 = 140;
    pub const PROJECTILE_TRACKERS: u8 = 142; // MessageID.cs:298 / NetMessage.cs:1603-1610 / MessageBuffer.cs:4282-4297
    pub const SYNC_LOADOUT: u8 = 147; // MessageID.cs:308 / NetMessage.cs:1624-1628 / MessageBuffer.cs:4328-4343

    // ---- S1-I-1 wave E: the event/misc tail (W4-d99) ----
    // The census after the d89-d98 lanes' named regions: the weather-adjacent
    // 51 MiscDataSync (sundial/moondial; the slime rain itself rides 61's
    // -19), the invasion/event ids 61/78/112/113/114/116/143, the moon/lunar
    // 103, the teleport family 65/73/96 (+ module 8's pylon body in
    // `netmodules.rs`), and the misc leftovers 75/77/127/141.
    // Bodies + codecs only; the apply/relay arms and send fan-out stay
    // S1-I-4. Coordination corrections to the first census draft: 95
    // MurderSomeoneElsesPortal rides d93 (`112e3c99`); 100/101/106 (NPC
    // portal teleport / tower shields / smoke poof) ride d94 (`68f4713d`,
    // `de1d2b82`) — their constants and bodies land there, not here. 96's
    // player twin is claimed by NO in-flight player lane (d96/d97's
    // research/17 census leaves 84-96 NOT-FOUND; full remote-branch sweep
    // 2026-11-20 found no other claim), so the event lane takes it beside
    // the rest of the portal family. Not taken (other d-lanes' regions):
    // 17/19/63/64/79/86/87/89/105/108/109/110/121-125/133/149/156 tile
    // edit+entities+wiring (d95), 18/57/136/158/159 clock+world sync (d90),
    // 34/35/39/41/52/85/88/90/104/130/131/146/160 items+containers+shops
    // (d98), 36/43/55/62/66/84/99/102/115/117/118/120/134/150 player
    // (d96/97), 56/92/137/153 NPC (d91/92), 81/91/107/119 combat text+emote
    // bubbles+chat (UI lanes), 68/139/161/162 join/lobby (d89), 94 dev
    // commands, 128 golf, 132 sounds (c-lanes), 157 team tail (d96/97).
    pub const MISC_DATA_SYNC: u8 = 51; // MessageID.cs:115 / NetMessage.cs:1010-1013 / MessageBuffer.cs:2482-2530
    pub const SPAWN_BOSS_EVENT: u8 = 61; // MessageID.cs:135 / NetMessage.cs:1073-1076 / MessageBuffer.cs:2740-2878
    pub const TELEPORT_ENTITY: u8 = 65; // MessageID.cs:143 / NetMessage.cs:1088-1109 / MessageBuffer.cs:2949-3035
    pub const REQUEST_TELEPORT: u8 = 73; // MessageID.cs:159 / NetMessage.cs:1142-1144 / MessageBuffer.cs:3127-3146
    pub const ANGLER_QUEST_FINISHED: u8 = 75; // MessageID.cs:163 — no body either side (MessageBuffer.cs:3154-3163)
    pub const TEMPORARY_ANIMATION: u8 = 77; // MessageID.cs:167 / NetMessage.cs:1154-1159 / MessageBuffer.cs:3183-3185
    pub const INVASION_PROGRESS: u8 = 78; // MessageID.cs:169 / NetMessage.cs:1160-1165 / MessageBuffer.cs:3186-3191
    pub const TELEPORT_PLAYER_PORTAL: u8 = 96; // MessageID.cs:206 / NetMessage.cs:1337-1346 / MessageBuffer.cs:3538-3557
    pub const MOONLORD_COUNTDOWN: u8 = 103; // MessageID.cs:220 / NetMessage.cs:1383-1386 / MessageBuffer.cs:3708-3714
    pub const SPECIAL_FX: u8 = 112; // MessageID.cs:238 / NetMessage.cs:1424-1431 / MessageBuffer.cs:3815-3840
    pub const CRYSTAL_INVASION_START: u8 = 113; // MessageID.cs:240 / NetMessage.cs:1432-1435 / MessageBuffer.cs:3841-3854
    pub const CRYSTAL_INVASION_WIPE: u8 = 114; // MessageID.cs:242 — no body (DD2Event.cs:636, MessageBuffer.cs:3855-3860)
    pub const CRYSTAL_INVASION_WAIT: u8 = 116; // MessageID.cs:246 / NetMessage.cs:1436-1438 / MessageBuffer.cs:3861-3866
    pub const REMOVE_REVENGE_MARKER: u8 = 127; // MessageID.cs:268 / NetMessage.cs:1518-1520 / MessageBuffer.cs:4049-4057
    pub const LUCY_POPUP: u8 = 141; // MessageID.cs:296 / NetMessage.cs:1595-1602 / MessageBuffer.cs:4265-4281
    pub const CRYSTAL_INVASION_SKIP: u8 = 143; // MessageID.cs:300 — no body (MessageBuffer.cs:4298-4303)
    // ---- W4-c27: the entity-state ids (apply/relay arms land with them) ----
    pub const SET_TIME: u8 = 18; // MessageID.cs:45 / NetMessage.cs:531-536 / MessageBuffer.cs:1290-1298
    pub const SYNC_PLAYER_ZONE: u8 = 36; // MessageID.cs:83 / NetMessage.cs:933-944 / MessageBuffer.cs:2178-2202
    pub const ITEM_ANIM: u8 = 41; // MessageID.cs:93 ItemRotationAndAnimation / NetMessage.cs:956-960 / MessageBuffer.cs:2271-2289
    pub const MANA_EFFECT: u8 = 43; // MessageID.cs:97 / NetMessage.cs:966-969 / MessageBuffer.cs:2307-2324
    pub const ADD_PLAYER_BUFF_PVP: u8 = 55; // MessageID.cs:123 / NetMessage.cs:1041-1045 / MessageBuffer.cs:2594-2611
    pub const SYNC_DODGE: u8 = 62; // MessageID.cs:137 / NetMessage.cs:1077-1080 / MessageBuffer.cs:2879-2908
    pub const PLAYER_STEALTH: u8 = 84; // MessageID.cs:182 / NetMessage.cs:1197-1204 / MessageBuffer.cs:3246-3260

    /// The ad-hoc event ids `netglue` still owns. Kept out of the 1..162 range
    /// so the accept gate (`GetData`, `MessageBuffer.cs:148-172`) rejects them
    /// before the slot is fully in-world (State 10) — exactly what those
    /// messages need. The entity snapshots left this block in M6-NET-2 (they
    /// became the 1:1 `13/21/151/22/23`); the remaining ones have no C# body
    /// of their own in the port's scope yet.
    pub const INT_EDIT: u8 = 201; // tile edit request/echo
    pub const INT_PICKUP: u8 = 202; // item pickup request (the client's `SyncItem`→151 stand-in)
    pub const INT_JOINED: u8 = 203; // peer-join notice
    pub const INT_LEFT: u8 = 204; // peer-leave notice
    pub const INT_TIME: u8 = 205; // clock broadcast
}

// ===========================================================================
// Client state machine (Netplay.Connection.State)
// ===========================================================================

/// The client's connection states — the transitions are the `Netplay.cs:469-543`
/// `InnerClientLoop` plus the message cases that set `Connection.State`.
pub mod client_state {
    /// Not connected yet; the first connected tick sends `1 Connect`
    /// (`Netplay.cs:469-475`).
    pub const NOT_CONNECTED: i32 = 0;
    /// `1 Connect` sent, waiting for the server's answer (`Ping.Reset()`,
    /// `Netplay.cs:475`).
    pub const CONNECT_SENT: i32 = 1;
    /// `3 PlayerId` received: the player pushes its data
    /// (`MessageBuffer.cs:233-239`).
    pub const PLAYER_DATA: i32 = 2;
    /// The push ended with `6 WorldRequest` (`MessageBuffer.cs:277-281`).
    pub const WORLD_REQUESTED: i32 = 3;
    /// `7 WorldInfo` received (`MessageBuffer.cs:658`).
    pub const WORLD_INFO: i32 = 4;
    /// The world header is in, the tiles stream next (`Netplay.cs:484-505`'s
    /// `WorldGen.clearWorld` step — the port installs the streamed world).
    pub const WORLD_LOADING: i32 = 5;
    /// `8 SpawnRequest` sent (`Netplay.cs:511-516`).
    pub const SPAWN_REQUESTED: i32 = 6;
    /// `49 WorldComplete` received: the player spawns in (`SpawningIntoWorld`,
    /// `MessageBuffer.cs:2448-2452`).
    pub const IN_WORLD: i32 = 10;
}

/// The server's per-slot states — the C# `Netplay.Clients[i].State` setters.
/// (Distinct from [`client_state`]: the two sides keep their own counters.)
pub mod slot_state {
    /// Fresh slot: only `1 Connect` is accepted
    /// (`MessageBuffer.cs:199-202`'s guard and the `State == 0` gate).
    pub const FRESH: i32 = 0;
    /// Version matched, the server has a password: only `38 Password`
    /// (`MessageBuffer.cs:210-214`).
    pub const AWAITING_PASSWORD: i32 = -1;
    /// Connected (no password, or the right one); `3 PlayerId` was sent
    /// (`MessageBuffer.cs:207`/`:2223`).
    pub const CONNECTED: i32 = 1;
    /// The client answered with `6 WorldRequest` (`MessageBuffer.cs:467`).
    pub const WORLD_REQUESTED: i32 = 2;
    /// The client sent `8 SpawnRequest` (`MessageBuffer.cs:808`).
    pub const SPAWN_REQUESTED: i32 = 3;
    /// Fully in-world: the client sent `12 PlayerSpawn`
    /// (`MessageBuffer.cs:925-929`).
    pub const IN_WORLD: i32 = 10;
}

// ===========================================================================
// Wire primitives
// ===========================================================================

/// Packet invariants: `len < 3` is a protocol error (`CheckBytes`'s throw,
/// `NetMessage.cs:2490`) and `len > ushort::MAX` cannot be framed at all
/// (`SendData`'s backfill throw, `NetMessage.cs:1679`).
pub const MIN_PACKET: usize = 3;
pub const MAX_PACKET: usize = u16::MAX as usize;

/// `bitsByte` (`Terraria/Utils.cs`'s `BitsByte`): eight bools in one byte,
/// written as a single byte between the two float halves of a `Vector2` in
/// some bodies. `set(i, v)` mirrors C#'s indexer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BitsByte(pub u8);

impl BitsByte {
    pub fn new() -> Self {
        Self(0)
    }
    pub fn set(&mut self, i: usize, v: bool) {
        if i < 8 {
            if v {
                self.0 |= 1 << i;
            } else {
                self.0 &= !(1 << i);
            }
        }
    }
    pub fn get(&self, i: usize) -> bool {
        i < 8 && self.0 & (1 << i) != 0
    }
}

/// `Utils.ReadRGB`'s three bytes (`Utils.cs:1369-1374`): R, G, B in order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// A `NetworkText` in the wire's literal mode (`NetworkText.cs:65-70`:
/// mode byte + the string; the substitution list is written for the other
/// modes only). The port has no keyed-text network layer yet, so every
/// message we send is a literal.
#[derive(Clone, Debug, PartialEq)]
pub struct NetText(pub String);

impl NetText {
    /// The literal-mode wire form: the mode byte, then the string
    /// (`NetworkText.cs:65-70`).
    pub fn write_to(self, w: NetWriter) -> NetWriter {
        w.u8(0).str(&self.0)
    }
    pub fn read_from(r: &mut NetReader) -> Option<Self> {
        let mode = r.u8()?;
        let s = r.str()?;
        if mode == 0 {
            Some(Self(s))
        } else {
            // Keyed/format modes carry a substitution list we cannot resolve
            // (NetworkText.cs:72-80): keep the literal we have, note the mode.
            Some(Self(s))
        }
    }
}

/// `BinaryWriter`'s little-endian primitives, framed like the C# per-message
/// writers (`NetMessage.cs:118-129` reserves two length bytes, writes the id,
/// then the body; the backfill at `:1677-1684` closes it).
pub struct NetWriter {
    buf: Vec<u8>,
}

impl NetWriter {
    /// Opens a frame: two reserved length bytes then the message id, exactly
    /// like `SendData`'s `Position += 2; writer.Write((byte)msgType)`.
    pub fn open(msg_id: u8) -> Self {
        let mut buf = Vec::with_capacity(64);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(msg_id);
        Self { buf }
    }

    pub fn u8(mut self, v: u8) -> Self {
        self.buf.push(v);
        self
    }
    pub fn i8(mut self, v: i8) -> Self {
        self.buf.push(v as u8);
        self
    }
    pub fn u16(mut self, v: u16) -> Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn i16(mut self, v: i16) -> Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn u32(mut self, v: u32) -> Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn i32(mut self, v: i32) -> Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn u64(mut self, v: u64) -> Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn f32(mut self, v: f32) -> Self {
        self.buf.extend_from_slice(&v.to_bits().to_le_bytes());
        self
    }
    pub fn f64(mut self, v: f64) -> Self {
        self.buf.extend_from_slice(&v.to_bits().to_le_bytes());
        self
    }
    pub fn bool(mut self, v: bool) -> Self {
        self.buf.push(v as u8);
        self
    }
    pub fn bitsbyte(mut self, v: BitsByte) -> Self {
        self.buf.push(v.0);
        self
    }
    pub fn rgb(mut self, v: Rgb) -> Self {
        self.buf.push(v.0);
        self.buf.push(v.1);
        self.buf.push(v.2);
        self
    }
    /// `BinaryWriter.Write(string)` = the 7-bit-encoded length + the UTF-8
    /// bytes (`BinaryWriter.cs`'s `Write7BitEncodedInt`).
    pub fn str(mut self, s: &str) -> Self {
        let b = s.as_bytes();
        write_u7(&mut self.buf, b.len() as u32);
        self.buf.extend_from_slice(b);
        self
    }
    /// Length-prefixed raw bytes for the internal chunk/snapshot bodies.
    pub fn bytes(mut self, b: &[u8]) -> Self {
        self.buf.extend_from_slice(&(b.len() as u32).to_le_bytes());
        self.buf.extend_from_slice(b);
        self
    }
    /// Appends raw bytes with no length prefix (the deflate blob's tail of
    /// a `10 TileBlock` frame).
    pub fn raw(mut self, b: &[u8]) -> Self {
        self.buf.extend_from_slice(b);
        self
    }
    /// The 7-bit-encoded int on its own (`BinaryWriter`'s
    /// `Write7BitEncodedInt` without a following payload).
    pub fn u7(mut self, v: u32) -> Self {
        write_u7(&mut self.buf, v);
        self
    }

    /// Closes the frame: backfills the length (`NetMessage.cs:1680-1684`).
    /// The length covers the whole packet, header included.
    pub fn finish(mut self) -> Vec<u8> {
        let len = self.buf.len();
        assert!(len <= MAX_PACKET, "net: packet over {} bytes", MAX_PACKET);
        self.buf[0..2].copy_from_slice(&(len as u16).to_le_bytes());
        self.buf
    }
}

/// `BinaryReader`'s primitives; a truncated body returns `None`, which the
/// decoders turn into a dropped frame (the C# throws `EndOfStreamException`).
pub struct NetReader<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> NetReader<'a> {
    pub fn new(d: &'a [u8]) -> Self {
        Self { d, p: 0 }
    }
    /// Bytes still unread.
    pub fn remaining(&self) -> usize {
        self.d.len().saturating_sub(self.p)
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.p.checked_add(n)?;
        let s = self.d.get(self.p..end)?;
        self.p = end;
        Some(s)
    }
    pub fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|s| s[0])
    }
    pub fn i8(&mut self) -> Option<i8> {
        self.u8().map(|v| v as i8)
    }
    pub fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|s| u16::from_le_bytes([s[0], s[1]]))
    }
    pub fn i16(&mut self) -> Option<i16> {
        self.u16().map(|v| v as i16)
    }
    pub fn u32(&mut self) -> Option<u32> {
        self.take(4)
            .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    pub fn i32(&mut self) -> Option<i32> {
        self.u32().map(|v| v as i32)
    }
    pub fn u64(&mut self) -> Option<u64> {
        self.take(8)
            .map(|s| u64::from_le_bytes(s.try_into().expect("take(8)")))
    }
    pub fn f32(&mut self) -> Option<f32> {
        self.u32().map(f32::from_bits)
    }
    pub fn f64(&mut self) -> Option<f64> {
        self.u64().map(f64::from_bits)
    }
    pub fn bool(&mut self) -> Option<bool> {
        self.u8().map(|v| v != 0)
    }
    pub fn bitsbyte(&mut self) -> Option<BitsByte> {
        self.u8().map(BitsByte)
    }
    pub fn rgb(&mut self) -> Option<Rgb> {
        let r = self.u8()?;
        let g = self.u8()?;
        let b = self.u8()?;
        Some(Rgb(r, g, b))
    }
    pub fn str(&mut self) -> Option<String> {
        let n = self.read_u7()? as usize;
        let b = self.take(n)?;
        Some(String::from_utf8_lossy(b).into_owned())
    }
    /// Length-prefixed raw bytes (the internal chunk bodies).
    pub fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = self.u32()? as usize;
        self.take(n).map(|s| s.to_vec())
    }
    /// A standalone 7-bit-encoded int.
    pub fn u7(&mut self) -> Option<u32> {
        self.read_u7()
    }
    /// The unread remainder (the deflate blob of a `10` body).
    pub fn rest(&mut self) -> Option<&'a [u8]> {
        self.take(self.remaining())
    }
    fn read_u7(&mut self) -> Option<u32> {
        let mut v: u32 = 0;
        for shift in 0..5 {
            let b = self.u8()?;
            v |= ((b & 0x7F) as u32) << (shift * 7);
            if b & 0x80 == 0 {
                return Some(v);
            }
        }
        None
    }
}

fn write_u7(buf: &mut Vec<u8>, mut v: u32) {
    while v >= 0x80 {
        buf.push((v as u8) | 0x80);
        v >>= 7;
    }
    buf.push(v as u8);
}

/// Writes one frame. The frame from `NetWriter::finish` already carries its
/// own `[u16 length][id][body]` header (`SendPacket` sends the shared buffer
/// whose first two bytes are the length, `NetMessage.cs:1854-1858`), so the
/// length is written exactly once.
fn write_frame(w: &mut impl Write, frame: &[u8]) -> std::io::Result<()> {
    debug_assert!(frame.len() >= MIN_PACKET && frame.len() <= MAX_PACKET);
    w.write_all(frame)
}

/// Reads one framed packet and strips the length header, so the returned
/// payload starts at the id byte. `len < 3` is a protocol error — the C#
/// `CheckBytes` throws and the connection dies (`NetMessage.cs:2490`), so
/// this returns `Err`.
fn read_frame(r: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut len = [0u8; 2];
    r.read_exact(&mut len)?;
    let len = u16::from_le_bytes(len) as usize;
    if len < MIN_PACKET {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "net: packet shorter than 3 bytes",
        ));
    }
    // The length covers the whole packet (the two length bytes included), so
    // only `len - 2` more bytes follow on the wire.
    let mut buf = vec![0u8; len - 2];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

// ===========================================================================
// Sections (200x150 tiles — `Netplay.GetSectionX/Y`, Netplay.cs:770-777)
// ===========================================================================

/// A tile section's width and height (`Main.maxSectionsX/Y`,
/// `Main.cs:66792-66793`).
pub const SECTION_W: i32 = 200;
pub const SECTION_H: i32 = 150;

/// The tile → section mapping (`Netplay.GetSectionX`).
pub fn section_x(tile_x: i32) -> i32 {
    tile_x.div_euclid(SECTION_W)
}

/// `Netplay.GetSectionY`.
pub fn section_y(tile_y: i32) -> i32 {
    tile_y.div_euclid(SECTION_H)
}

/// The section grid dimensions for a world (`maxTilesX / 200`, rounded up).
pub fn sections_x(world_w: i32) -> i32 {
    (world_w + SECTION_W - 1) / SECTION_W
}

pub fn sections_y(world_h: i32) -> i32 {
    (world_h + SECTION_H - 1) / SECTION_H
}

// ===========================================================================
// Payload structs
// ===========================================================================

/// The render-facing remote-player snapshot. The wire fields arrive via
/// `13 PlayerState`; `off_x/off_y` are the client-side smoothing accumulator
/// (`Player.netOffset`'s port, `MessageBuffer.cs:1004-1008`) and never cross
/// the wire.
#[derive(Clone, Debug, Default)]
pub struct PlayerNet {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub facing: i32,
    pub selected: u16,
    pub on_ground: bool,
    pub leg_frame: u32,
    pub body_frame: u32,
    /// `Player.netOffset` — the visual lag toward zero; drawn at
    /// `x + off_x` (see `netglue::decay_net_offsets`).
    pub off_x: f32,
    pub off_y: f32,
    /// `Main.player[id].inventory` - the peer's slot state, fed by the
    /// `5 PlayerInventory` messages (`MessageBuffer.cs:399-462`).
    pub inventory: HashMap<u8, NetItem>,
    /// `Main.player[id].buffType[]`'s live ids, fed by `50 PlayerBuffs`
    /// (`MessageBuffer.cs:2454-2480` sets every listed buff's time to 60).
    pub buffs: Vec<u16>,
    /// `Main.player[id].chest` (`Player.cs` field): which container the peer
    /// has open, fed by `80 SyncPlayerChestIndex`
    /// (`MessageBuffer.cs:3216-3227`). `None` = closed (`-1`), the `Default`.
    pub chest: Option<i16>,
    /// `Main.player[id].zone1..5` + `townNPCs`, fed by `36 SyncPlayerZone`
    /// (`MessageBuffer.cs:2187-2192`). W4-c27.
    pub zone: PlayerZone,
    /// `Player.itemRotation`/`itemAnimation`, fed by `41
    /// ItemRotationAndAnimation` (`MessageBuffer.cs:2281-2282`); `channel`
    /// re-derived from the selected inventory item (`:2283`). W4-c27.
    pub item_rotation: f32,
    pub item_animation: i16,
    pub channel: bool,
    /// `Player.hostile`, the `55 AddPlayerBuffPvP` gate's input
    /// (`MessageBuffer.cs:2599`). Fed by `30 TogglePVP` when that arm lands
    /// (S1-I-4); until then `false` — the gate refuses, exactly the C#'s
    /// PvP-off shape. W4-c27.
    pub hostile: bool,
    /// `Player.stealth` (`Player.cs:1562`, the C# field default 1f), fed by
    /// `84 PlayerStealth` (`MessageBuffer.cs:3254`). The port's peer record
    /// keeps the `Default` 0 until the first sync — same as the local
    /// `Player` ctor's 0.0 (the shroomite-stealth writers are still unported,
    /// `npc_target.rs:83`). W4-c27.
    pub stealth: f32,
}

/// `msg 13 PlayerState`'s body (`NetMessage.cs:446-513`): the player id, four
/// BitsByte groups, the selected item, the full-float position and the
/// conditional velocity/mount/potion/camera bodies. The port has no mounts,
/// potions or camera-target sync, so those bits ride as flags with no bodies
/// (deviation in `docs/ANALYSIS.md`); the trailing anim/hp fields are the
/// port's two-client extension — the C# derives them from each peer's local
/// simulation, which the port does not run for remote players.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerState {
    pub player_id: u8,
    /// bitsByte21: controlUp/Down/Left/Right/Jump/UseItem/direction==1/controlDash.
    pub flags_a: BitsByte,
    /// bitsByte22: pulley/pulley2/velocity!=0/vortexStealth/gravDir/shieldRaised/ghost/mount.
    pub flags_b: BitsByte,
    /// bitsByte23: hoveringUp/voidVault/sitting/downedDD2/petting/petSmall/potionReturn/hoveringDown.
    pub flags_c: BitsByte,
    /// bitsByte24: sleeping/autoReuse/controlDownHold/operatingEntity/controlUseTile/cameraTarget/lastUseSuccess/snapStone.
    pub flags_d: BitsByte,
    pub selected_item: u8,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    // ---- port extension: the render fields the C# derives locally ----
    pub on_ground: bool,
    pub leg_frame: u32,
    pub body_frame: u32,
    pub hp: i32,
    pub max_hp: i32,
}

impl PlayerState {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut w = w
            .u8(self.player_id)
            .bitsbyte(self.flags_a)
            .bitsbyte(self.flags_b)
            .bitsbyte(self.flags_c)
            .bitsbyte(self.flags_d)
            .u8(self.selected_item)
            .f32(self.x)
            .f32(self.y);
        // `velocity != Vector2.Zero` gate (`NetMessage.cs:491-495`).
        if self.flags_b.get(2) {
            w = w.f32(self.vx).f32(self.vy);
        }
        // The port's render extension (two-client only; see the struct doc).
        w.bool(self.on_ground)
            .u32(self.leg_frame)
            .u32(self.body_frame)
            .i32(self.hp)
            .i32(self.max_hp)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let player_id = r.u8()?;
        let flags_a = r.bitsbyte()?;
        let flags_b = r.bitsbyte()?;
        let flags_c = r.bitsbyte()?;
        let flags_d = r.bitsbyte()?;
        let selected_item = r.u8()?;
        let x = r.f32()?;
        let y = r.f32()?;
        let (vx, vy) = if flags_b.get(2) {
            (r.f32()?, r.f32()?)
        } else {
            (0.0, 0.0)
        };
        let on_ground = r.bool()?;
        let leg_frame = r.u32()?;
        let body_frame = r.u32()?;
        let hp = r.i32()?;
        let max_hp = r.i32()?;
        Some(Self {
            player_id,
            flags_a,
            flags_b,
            flags_c,
            flags_d,
            selected_item,
            x,
            y,
            vx,
            vy,
            on_ground,
            leg_frame,
            body_frame,
            hp,
            max_hp,
        })
    }
}

/// `msg 23 NPCInfo`'s body (`NetMessage.cs:686-748`), adapted to the port's
/// NPC shape: the slot index, the generation counter (bumped on a kind change
/// so the client re-creates the slot, the C#'s `existing.generation !=
/// generation` spawn test), the anchored position, the velocity, the two flag
/// bytes and the width-switched life. The `ai[]`, `difficulty`,
/// `statsAreScaledForThisManyPlayers`, `releaseOwner` and `SyncAnchor` fields
/// have no port home (deviation in `docs/ANALYSIS.md`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpcInfo {
    /// The host's NPC slot index (`whoAmI`).
    pub slot: u8,
    /// Bumped on a kind change / fresh spawn → the client re-creates the slot.
    pub generation: u8,
    /// `spawnNeedsSyncing` — a full-create for a slot the client already has.
    pub spawn: bool,
    pub kind: i16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// flags1 b0: `direction > 0`.
    pub dir_right: bool,
    /// flags1 b1: `directionY > 0`.
    pub dir_y_down: bool,
    /// flags1 b6: `spriteDirection > 0`.
    pub sprite_right: bool,
    /// flags1 b7: `life == lifeMax` → the life block is omitted.
    pub life_full: bool,
    pub hp: i32,
}

impl NpcInfo {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut flags1 = BitsByte::new();
        flags1.set(0, self.dir_right);
        flags1.set(1, self.dir_y_down);
        flags1.set(6, self.sprite_right);
        flags1.set(7, self.life_full);
        let mut flags2 = BitsByte::new();
        flags2.set(3, self.spawn);
        let mut w = w
            .u8(self.slot)
            .u8(self.generation)
            .f32(self.x)
            .f32(self.y)
            .f32(self.vx)
            .f32(self.vy)
            .u16(0) // `target`: the port has no NPC targeting sync yet
            .bitsbyte(flags1)
            .bitsbyte(flags2)
            .i16(self.kind);
        // The width-switched life block (`NetMessage.cs:734-751`), skipped
        // when the NPC is at full health.
        if !self.life_full {
            let hp = self.hp.max(0);
            if hp > 0x7FFF {
                w = w.u8(4).i32(hp);
            } else if hp > 0x7F {
                w = w.u8(2).i16(hp as i16);
            } else {
                w = w.u8(1).i8(hp as i8);
            }
        }
        w
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let slot = r.u8()?;
        let generation = r.u8()?;
        let x = r.f32()?;
        let y = r.f32()?;
        let vx = r.f32()?;
        let vy = r.f32()?;
        let _target = r.u16()?;
        let flags1 = r.bitsbyte()?;
        let flags2 = r.bitsbyte()?;
        let kind = r.i16()?;
        let hp = if flags1.get(7) {
            -1 // full health: the client keeps its local lifeMax
        } else {
            match r.u8()? {
                2 => r.i16()? as i32,
                4 => r.i32()?,
                _ => r.i8()? as i32,
            }
        };
        Some(Self {
            slot,
            generation,
            spawn: flags2.get(3),
            kind,
            x,
            y,
            vx,
            vy,
            dir_right: flags1.get(0),
            dir_y_down: flags1.get(1),
            sprite_right: flags1.get(6),
            life_full: flags1.get(7),
            hp,
        })
    }
}

/// `msg 21 ItemDrop`'s body (`NetMessage.cs:646-669`): the slot, the
/// position/velocity, the stack, the prefix, the flag byte and the type, plus
/// the conditional shimmer fields. C#'s `noBroadcast`/`instanced` split and
/// the slot-400 spawn request are the M6-NET-5 follow-ups (deviation in
/// `docs/ANALYSIS.md`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemState {
    pub slot: i16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub stack: i16,
    pub prefix: u8,
    pub kind: i16,
    pub shimmered: bool,
    pub shimmer_time: f32,
}

impl ItemState {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut flags = BitsByte::new();
        flags.set(2, self.shimmered || self.shimmer_time > 0.0);
        let w = w
            .i16(self.slot)
            .f32(self.x)
            .f32(self.y)
            .f32(self.vx)
            .f32(self.vy)
            .i16(self.stack)
            .u8(self.prefix)
            .bitsbyte(flags)
            .i16(self.kind);
        if self.shimmered || self.shimmer_time > 0.0 {
            w.bool(self.shimmered).f32(self.shimmer_time)
        } else {
            w
        }
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let slot = r.i16()?;
        let x = r.f32()?;
        let y = r.f32()?;
        let vx = r.f32()?;
        let vy = r.f32()?;
        let stack = r.i16()?;
        let prefix = r.u8()?;
        let flags = r.bitsbyte()?;
        let kind = r.i16()?;
        let (shimmered, shimmer_time) = if flags.get(2) {
            (r.bool()?, r.f32()?)
        } else {
            (false, 0.0)
        };
        Some(Self {
            slot,
            x,
            y,
            vx,
            vy,
            stack,
            prefix,
            kind,
            shimmered,
            shimmer_time,
        })
    }
}

/// `msg 22 ItemOwner`'s body (`NetMessage.cs:675-685`): the reservation.
/// Server→client only; the varints are the C# 7-bit encodings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemOwner {
    pub slot: i16,
    /// `playerIndexTheItemIsReservedFor` (255 = none).
    pub reserved_for: u8,
    /// `timeToKeepReservation` in ticks.
    pub hold: u32,
    pub grab_delay_player: u8,
    pub grab_delay: u32,
    pub x: f32,
    pub y: f32,
}

impl ItemOwner {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        w.i16(self.slot)
            .u8(self.reserved_for)
            .u7(self.hold)
            .u8(self.grab_delay_player)
            .u7(self.grab_delay)
            .f32(self.x)
            .f32(self.y)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let slot = r.i16()?;
        let reserved_for = r.u8()?;
        let hold = r.u7()?;
        let grab_delay_player = r.u8()?;
        let grab_delay = r.u7()?;
        let x = r.f32()?;
        let y = r.f32()?;
        Some(Self {
            slot,
            reserved_for,
            hold,
            grab_delay_player,
            grab_delay,
            x,
            y,
        })
    }
}

/// `msg 5 PlayerInventory`'s item body (`NetMessage.cs:199-222`): the slot
/// index, the stack, the prefix, the type and a `BitsByte` whose bit 0 is
/// `favorited` (bit 1 is the server's "blocked slot" indicator, which the
/// port has no home for yet — deviation in `docs/ANALYSIS.md`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NetItem {
    pub kind: i16,
    pub stack: i16,
    pub prefix: u8,
    pub favorited: bool,
}

/// `msg 4 PlayerInfo`'s body (`NetMessage.cs:147-197`; the field order and the
/// BitsByte groups are 1:1, including the write-side clamps). Appearance
/// fields the port has no home for yet are carried and defaulted (see
/// `docs/ANALYSIS.md`).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerInfo {
    /// Server-forced to the sender's slot on read (`MessageBuffer.cs:284-286`).
    pub player_id: u8,
    pub skin_variant: u8,
    pub voice_variant: u8,
    pub voice_pitch_offset: f32,
    pub hair: u8,
    pub name: String,
    pub hair_dye: u8,
    pub hide_visible_accessory: u16,
    pub hide_misc: u8,
    pub hair_color: Rgb,
    pub skin_color: Rgb,
    pub eye_color: Rgb,
    pub shirt_color: Rgb,
    pub under_shirt_color: Rgb,
    pub pants_color: Rgb,
    pub shoe_color: Rgb,
    /// bit 0-2: difficulty (1/2/3), bit 2 of group 1 is extraAccessory
    /// (`NetMessage.cs:184-186`).
    pub flags1: BitsByte,
    pub flags2: BitsByte,
    pub flags3: BitsByte,
}

impl PlayerInfo {
    /// The defaults the port sends: the name is the only field with a home
    /// today (`App.player_name`); every appearance field rides the wire as
    /// vanilla-zero until the character milestone lands.
    pub fn with_name(player_id: u8, name: &str) -> Self {
        Self {
            player_id,
            skin_variant: 0,
            voice_variant: 0,
            voice_pitch_offset: 0.0,
            hair: 0,
            name: name.to_string(),
            hair_dye: 0,
            hide_visible_accessory: 0,
            hide_misc: 0,
            hair_color: Rgb(0, 0, 0),
            skin_color: Rgb(0, 0, 0),
            eye_color: Rgb(0, 0, 0),
            shirt_color: Rgb(0, 0, 0),
            under_shirt_color: Rgb(0, 0, 0),
            pants_color: Rgb(0, 0, 0),
            shoe_color: Rgb(0, 0, 0),
            flags1: BitsByte::new(),
            flags2: BitsByte::new(),
            flags3: BitsByte::new(),
        }
    }

    fn write_to(&self, w: NetWriter) -> NetWriter {
        w.u8(self.player_id)
            .u8(self.skin_variant)
            .u8(self.voice_variant)
            .f32(self.voice_pitch_offset)
            .u8(self.hair)
            .str(&self.name)
            .u8(self.hair_dye)
            .u16(self.hide_visible_accessory)
            .u8(self.hide_misc)
            .rgb(self.hair_color)
            .rgb(self.skin_color)
            .rgb(self.eye_color)
            .rgb(self.shirt_color)
            .rgb(self.under_shirt_color)
            .rgb(self.pants_color)
            .rgb(self.shoe_color)
            .bitsbyte(self.flags1)
            .bitsbyte(self.flags2)
            .bitsbyte(self.flags3)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let player_id = r.u8()?;
        let skin_variant = r.u8()?;
        let voice_variant = r.u8()?;
        let voice_pitch_offset = r.f32()?;
        let hair = r.u8()?;
        let name = r.str()?;
        let hair_dye = r.u8()?;
        let hide_visible_accessory = r.u16()?;
        let hide_misc = r.u8()?;
        let hair_color = r.rgb()?;
        let skin_color = r.rgb()?;
        let eye_color = r.rgb()?;
        let shirt_color = r.rgb()?;
        let under_shirt_color = r.rgb()?;
        let pants_color = r.rgb()?;
        let shoe_color = r.rgb()?;
        let flags1 = r.bitsbyte()?;
        let flags2 = r.bitsbyte()?;
        let flags3 = r.bitsbyte()?;
        Some(Self {
            player_id,
            skin_variant,
            voice_variant,
            voice_pitch_offset,
            // `hair >= 228` normalises to 0 (`MessageBuffer.cs:319`).
            hair: if hair >= 228 { 0 } else { hair },
            name,
            hair_dye,
            hide_visible_accessory,
            hide_misc,
            hair_color,
            skin_color,
            eye_color,
            shirt_color,
            under_shirt_color,
            pants_color,
            shoe_color,
            flags1,
            flags2,
            flags3,
        })
    }
}

/// `WorldGen.TreeTops.SyncSend`'s body: one byte per area
/// (`TreeTopsInfo.cs:45`, `AreaId.Count = 13` at `:34`).
pub const TREE_TOPS_BYTES: usize = 13;

/// `msg 7 WorldInfo`'s body (`NetMessage.cs:224-412`; the read side is
/// `MessageBuffer.cs:473-658`). The field order is 1:1. Fields the port has
/// no home for yet (background styles, cave backups, the tree tops, the seed
/// flags, the ore tiers) ride the wire as vanilla zeros/defaults and are
/// listed in `docs/ANALYSIS.md`.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldInfo {
    /// `Main.time` in day ticks (`NetMessage.cs:226`'s `(int)Main.time`).
    pub time: i32,
    pub day_time: bool,
    pub blood_moon: bool,
    pub eclipse: bool,
    pub moon_phase: u8,
    pub max_tiles_x: i16,
    pub max_tiles_y: i16,
    pub spawn_tile_x: i16,
    pub spawn_tile_y: i16,
    pub world_surface: i16,
    pub rock_layer: i16,
    pub world_id: i32,
    pub world_name: String,
    pub game_mode: u8,
    pub unique_id: [u8; 16],
    pub generator_version: u64,
    pub moon_type: u8,
    /// treeBG1..4, corrupt, jungle, snow, hallow, crimson, desert, ocean,
    /// mushroom, underworld (`NetMessage.cs:246-258`).
    pub backgrounds: [u8; 11],
    pub ice_back_style: u8,
    pub jungle_back_style: u8,
    pub hell_back_style: u8,
    pub wind_speed_target: f32,
    pub num_clouds: u8,
    pub tree_x: [i32; 3],
    pub tree_style: [u8; 4],
    pub cave_back_x: [i32; 3],
    pub cave_back_style: [u8; 4],
    pub tree_tops: [u8; TREE_TOPS_BYTES],
    pub max_raining: f32,
    /// The 14 `BitsByte` world-flag groups (`NetMessage.cs:285-389`).
    pub world_flags: [BitsByte; 14],
    pub sundial_cooldown: u8,
    pub moondial_cooldown: u8,
    /// Copper, Iron, Silver, Gold, Cobalt, Mythril, Adamantite
    /// (`NetMessage.cs:390-402`).
    pub saved_ore_tiers: [i16; 7],
    pub invasion_type: i8,
    /// 0 without a social API (`NetMessage.cs:405-410`).
    pub lobby_id: u64,
    pub sandstorm_severity: f32,
    /// `ExtraSpawnPointManager.Write(writer, networking: true)`
    /// (`ExtraSpawnPointManager.cs:448-456`: a count byte plus X/Y pairs).
    pub extra_spawn_points: Vec<(i16, i16)>,
    pub dungeon_x: i16,
    pub dungeon_y: i16,
}

impl WorldInfo {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut w = w
            .i32(self.time)
            .bitsbyte({
                let mut b = BitsByte::new();
                b.set(0, self.day_time);
                b.set(1, self.blood_moon);
                b.set(2, self.eclipse);
                b
            })
            .u8(self.moon_phase)
            .i16(self.max_tiles_x)
            .i16(self.max_tiles_y)
            .i16(self.spawn_tile_x)
            .i16(self.spawn_tile_y)
            .i16(self.world_surface)
            .i16(self.rock_layer)
            .i32(self.world_id)
            .str(&self.world_name)
            .u8(self.game_mode);
        // `UniqueId.ToByteArray()` — 16 raw bytes, no length prefix
        // (`NetMessage.cs:243`).
        for b in self.unique_id {
            w = w.u8(b);
        }
        w = w.u64(self.generator_version).u8(self.moon_type);
        for b in self.backgrounds {
            w = w.u8(b);
        }
        w = w
            .u8(self.ice_back_style)
            .u8(self.jungle_back_style)
            .u8(self.hell_back_style)
            .f32(self.wind_speed_target)
            .u8(self.num_clouds);
        for x in self.tree_x {
            w = w.i32(x);
        }
        for s in self.tree_style {
            w = w.u8(s);
        }
        for x in self.cave_back_x {
            w = w.i32(x);
        }
        for s in self.cave_back_style {
            w = w.u8(s);
        }
        for t in self.tree_tops {
            w = w.u8(t);
        }
        w = w.f32(self.max_raining);
        for b in self.world_flags {
            w = w.bitsbyte(b);
        }
        w = w.u8(self.sundial_cooldown).u8(self.moondial_cooldown);
        for ore in self.saved_ore_tiers {
            w = w.i16(ore);
        }
        w = w
            .i8(self.invasion_type)
            .u64(self.lobby_id)
            .f32(self.sandstorm_severity)
            .u8(self.extra_spawn_points.len() as u8);
        for (x, y) in &self.extra_spawn_points {
            w = w.i16(*x).i16(*y);
        }
        w.i16(self.dungeon_x).i16(self.dungeon_y)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let time = r.i32()?;
        let flags = r.bitsbyte()?;
        let moon_phase = r.u8()?;
        let max_tiles_x = r.i16()?;
        let max_tiles_y = r.i16()?;
        let spawn_tile_x = r.i16()?;
        let spawn_tile_y = r.i16()?;
        let world_surface = r.i16()?;
        let rock_layer = r.i16()?;
        let world_id = r.i32()?;
        let world_name = r.str()?;
        let game_mode = r.u8()?;
        let unique_id: [u8; 16] = r.take(16)?.try_into().ok()?;
        let generator_version = r.u64()?;
        let moon_type = r.u8()?;
        let mut backgrounds = [0u8; 11];
        for b in backgrounds.iter_mut() {
            *b = r.u8()?;
        }
        let ice_back_style = r.u8()?;
        let jungle_back_style = r.u8()?;
        let hell_back_style = r.u8()?;
        let wind_speed_target = r.f32()?;
        let num_clouds = r.u8()?;
        let mut tree_x = [0i32; 3];
        for x in tree_x.iter_mut() {
            *x = r.i32()?;
        }
        let mut tree_style = [0u8; 4];
        for s in tree_style.iter_mut() {
            *s = r.u8()?;
        }
        let mut cave_back_x = [0i32; 3];
        for x in cave_back_x.iter_mut() {
            *x = r.i32()?;
        }
        let mut cave_back_style = [0u8; 4];
        for s in cave_back_style.iter_mut() {
            *s = r.u8()?;
        }
        let tree_tops: [u8; TREE_TOPS_BYTES] = r
            .take(TREE_TOPS_BYTES)?
            .try_into()
            .expect("take(TREE_TOPS_BYTES)");
        let max_raining = r.f32()?;
        let mut world_flags = [BitsByte::new(); 14];
        for b in world_flags.iter_mut() {
            *b = r.bitsbyte()?;
        }
        let sundial_cooldown = r.u8()?;
        let moondial_cooldown = r.u8()?;
        let mut saved_ore_tiers = [0i16; 7];
        for ore in saved_ore_tiers.iter_mut() {
            *ore = r.i16()?;
        }
        let invasion_type = r.i8()?;
        let lobby_id = r.u64()?;
        let sandstorm_severity = r.f32()?;
        let n = r.u8()? as usize;
        let mut extra_spawn_points = Vec::with_capacity(n);
        for _ in 0..n {
            extra_spawn_points.push((r.i16()?, r.i16()?));
        }
        let dungeon_x = r.i16()?;
        let dungeon_y = r.i16()?;
        Some(Self {
            time,
            day_time: flags.get(0),
            blood_moon: flags.get(1),
            eclipse: flags.get(2),
            moon_phase,
            max_tiles_x,
            max_tiles_y,
            spawn_tile_x,
            spawn_tile_y,
            world_surface,
            rock_layer,
            world_id,
            world_name,
            game_mode,
            unique_id,
            generator_version,
            moon_type,
            backgrounds,
            ice_back_style,
            jungle_back_style,
            hell_back_style,
            wind_speed_target,
            num_clouds,
            tree_x,
            tree_style,
            cave_back_x,
            cave_back_style,
            tree_tops,
            max_raining,
            world_flags,
            sundial_cooldown,
            moondial_cooldown,
            saved_ore_tiers,
            invasion_type,
            lobby_id,
            sandstorm_severity,
            extra_spawn_points,
            dungeon_x,
            dungeon_y,
        })
    }
}

/// `msg 8 SpawnRequest`'s body (`NetMessage.cs:414-418`): the requested spawn
/// tile and the team.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnRequest {
    pub x: i32,
    pub y: i32,
    pub team: u8,
}

/// `msg 12 PlayerSpawn`'s body (`MessageBuffer.cs:908-920`): where the player
/// spawned, the respawn timer, the death counters and the team.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSpawn {
    pub x: i16,
    pub y: i16,
    pub respawn_timer: i32,
    pub deaths_pve: i16,
    pub deaths_pvp: i16,
    pub team: u8,
    /// `PlayerSpawnContext`, the low byte (`MessageBuffer.cs:919`).
    pub spawn_context: u8,
}

/// `msg 31 RequestChestOpen`'s body (`NetMessage.cs:860-863`): the chest
/// tile's top-left. The server answers `33` + the whole `32`/`155` contents
/// (`MessageBuffer.cs:1868-1895`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChestOpenRequest {
    pub x: i16,
    pub y: i16,
}

/// `msg 32 SyncChestItem`'s body (`NetMessage.cs:864-877`): one chest slot's
/// item. `stack`/`prefix`/`kind` are the C#'s written order (`short stack`,
/// `byte prefix`, `short type` - the type is zeroed when the item has no
/// name, `:870-873`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChestItem {
    /// `Chest` index (`short`).
    pub chest: i16,
    /// `Chest.item[]` slot (`byte`).
    pub slot: u8,
    pub stack: i16,
    pub prefix: u8,
    pub kind: i16,
}

/// `msg 33 SyncPlayerChest`'s body (`NetMessage.cs:879-911`): the chest index,
/// its tile position and the optional name (`name_len` = 1..=20 with the text
/// following, 0 = none, 255 = a too-long name the C# refuses,
/// `MessageBuffer.cs:1927-1937`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerChest {
    pub chest: i16,
    pub x: i16,
    pub y: i16,
    /// The C#'s `num24` byte: 0 none, 1..=20 a name follows, 255 refused.
    pub name_len: u8,
    pub name: String,
}

/// `msg 46 OpenSignRequest`'s body (`NetMessage.cs:975-978`): the sign tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignRequest {
    pub x: i16,
    pub y: i16,
}

/// `msg 47 OpenSignResponse`'s body (`NetMessage.cs:979-986`): the sign index,
/// its tile position, the text and the two flag bytes (`number2` = the
/// sender's `whoAmI`, `number3` = a `BitsByte` whose bit 0 suppresses the
/// client's `OpenSign`, `MessageBuffer.cs:2400-2403`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignData {
    pub sign: i16,
    pub x: i16,
    pub y: i16,
    pub text: String,
    pub sender: u8,
    pub flags: u8,
}

/// `msg 50 PlayerBuffs`'s body (`NetMessage.cs:996-1008`): the player id then
/// the live buff ids as `ushort`s, terminated by a zero. The reader gives
/// every listed buff a time of 60 ticks (`MessageBuffer.cs:2470-2471`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerBuffs {
    pub player_id: u8,
    pub buffs: Vec<u16>,
}

/// `msg 53 AddNPCBuff`'s body (`NetMessage.cs:1019-1022`): the NPC slot, the
/// buff id and its duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddNpcBuff {
    pub npc: i16,
    pub buff_type: u16,
    pub time: i16,
}

/// `msg 54 NPCBuffs`'s body (`NetMessage.cs:1024-1039`): the NPC slot then the
/// `(buff, time)` pairs as `ushort`s, terminated by a zero buff id
/// (`MessageBuffer.cs:2584-2589`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpcBuffs {
    pub npc: i16,
    pub buffs: Vec<(u16, u16)>,
}

/// `msg 59 HitSwitch`'s body (`NetMessage.cs:1063-1066`): the switch tile. The
/// receiver replays `Wiring.HitSwitch` on it (`MessageBuffer.cs:2682-2693`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitSwitch {
    pub x: i16,
    pub y: i16,
}

/// `msg 80 SyncPlayerChestIndex`'s body (`NetMessage.cs:1175-1178`): which
/// chest a player has open (`-3..8000`, `MessageBuffer.cs:3216-3227`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerChestIndex {
    pub player_id: u8,
    pub chest: i16,
}

/// `msg 155 SyncChestSize`'s body (`NetMessage.cs:1650-1653`): the chest index
/// and its `maxItems` (`MessageBuffer.cs:4429-4438`'s `Chest.Resize`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChestSize {
    pub chest: i16,
    pub size: i16,
}

/// `msg 69 ChestName`'s body (`NetMessage.cs:1334-1341`): the chest index, its
/// tile and the new name - the `33` rename's relay
/// (`MessageBuffer.cs:3059-3078`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChestName {
    pub chest: i16,
    pub x: i16,
    pub y: i16,
    pub name: String,
}

// ---- S1-I-1 wave A: the join bundle's payload bodies ----

/// `msg 30 TogglePVP`'s body (`NetMessage.cs:856-859`): the player id then
/// `Player.hostile`. The server overrides the id to the sender
/// (`MessageBuffer.cs:1853-1857`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TogglePvp {
    pub player_id: u8,
    pub hostile: bool,
}

/// `msg 42 Unknown42`'s body (`NetMessage.cs:961-965`): the player id,
/// `statMana` and `statManaMax`, both `short` (`MessageBuffer.cs:2290-2306`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerMana {
    pub player_id: u8,
    pub mana: i16,
    pub max_mana: i16,
}

/// `msg 45 TeamChange`'s body (`NetMessage.cs:970-974`): the player id then
/// `Player.team` as a byte (`MessageBuffer.cs:2325-2336`). `157
/// TeamChangeFromUI` shares the body but adds the team-spawn tail
/// (`:2355-2363`); it is not a join-bundle id and stays unported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeamChange {
    pub player_id: u8,
    pub team: u8,
}

/// `msg 60 Unknown60`'s body (`NetMessage.cs:1067-1072`): the NPC slot, its
/// `homeTileX`/`homeTileY` (`short`s) and the household status byte —
/// `WorldGen.TownManager.GetHouseholdStatus` (`NetMessage.cs:2845`; the
/// reader's arms: 1 = homeless/kick-out, 2 = room set,
/// `MessageBuffer.cs:2708-2737`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcHouse {
    pub npc: i16,
    pub home_x: i16,
    pub home_y: i16,
    pub status: u8,
}

/// `msg 72 TravelMerchantItems`'s body (`NetMessage.cs:1134-1141`): the
/// `Main.travelShop` array — `Main.TravelShopMaxSlots = 40` (`Main.cs:67029`)
/// `short` item types (`MessageBuffer.cs:3118-3126`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TravelShop {
    pub items: Vec<i16>,
}

/// `Main.TravelShopMaxSlots` (`Main.cs:67029`).
pub const TRAVEL_SHOP_MAX_SLOTS: usize = 40;

/// `msg 74 AnglerQuest`'s body (`NetMessage.cs:1145-1148`): `Main.anglerQuest`
/// as a byte and whether the recipient's name is in
/// `Main.anglerWhoFinishedToday` (`MessageBuffer.cs:3147-3153` sets
/// `anglerQuestFinished`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnglerQuest {
    pub quest: u8,
    pub finished: bool,
}

/// `msg 126 SyncRevengeMarker`'s body — `RevengeMarker.WriteSelfTo`
/// (`CoinLossRevengeSystem.cs:275-286`), read field-for-field by
/// `AddMarkerFromReader` (`:321-324`). Sent by `SendCoinLossRevengeMarker`
/// (`NetMessage.cs:2621-2625`); the join push is `SendAllMarkersToPlayer`
/// (`CoinLossRevengeSystem.cs:498-507`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RevengeMarker {
    pub unique_id: i32,
    pub x: f32,
    pub y: f32,
    pub npc_net_id: i32,
    pub npc_hp_percent: f32,
    pub npc_type: i32,
    pub npc_ai_style: i32,
    pub coins_value: i32,
    pub base_value: f32,
    pub spawned_from_statue: bool,
}

/// `msg 140 SetMiscEventValues`'s body (`NetMessage.cs:1591-1594`): a kind
/// byte then an `int`. The join bundle only sends kind 0 — the credits roll's
/// remaining frames (`CreditsRollEvent.cs:27-33`); kinds 1/2 are the server's
/// slime transforms (`MessageBuffer.cs:4250-4260`) and stay unported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiscEventValue {
    pub kind: u8,
    pub value: i32,
}

/// `TrackedProjectileReference.Write/Read`
/// (`DataStructures/TrackedProjectileReference.cs:22-26`, `:33-37`): the
/// projectile key as an `int` then the type as a `short`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackedProjectileRef {
    pub key: i32,
    pub kind: i16,
}

/// `msg 142 SyncProjectileTrackers`'s body (`NetMessage.cs:1603-1610`): the
/// player id, then `piggyBankProjTracker` and `voidLensChest`
/// (`Player.cs:2872-2874`), each one `TrackedProjectileReference`
/// (`MessageBuffer.cs:4282-4297`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileTrackers {
    pub player_id: u8,
    pub piggy_bank: TrackedProjectileRef,
    pub void_lens: TrackedProjectileRef,
}

/// `msg 147 SyncLoadout`'s body (`NetMessage.cs:1624-1628`): the player id,
/// the `CurrentLoadoutIndex` byte, then `hideVisibleAccessory` packed into one
/// `ushort` by `WriteAccessoryVisibility` (`NetMessage.cs:1882-1893` — bit `i`
/// per slot, `Player.cs:1708`'s `bool[10]`); the reader unpacks it
/// (`MessageBuffer.cs:4494-4501`) after `TrySwitchingLoadout` (`:4336`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncLoadout {
    pub player_id: u8,
    pub loadout: u8,
    pub hide_accessory: u16,
}

// ---- S1-I-1 wave E: the event/misc tail bodies (W4-d99) ----

/// `msg 51 MiscDataSync`'s body (`NetMessage.cs:1010-1013`): the subject byte
/// then the kind byte. Subjects are a player slot for kinds 1/2/3/6 (the
/// Skeletron summon `Player.cs:6411`-adjacent rod use, the rod's sound relay,
/// the sundial `Main.cs:6175`, the moondial `:6190`) and an NPC slot for
/// kind 4 (the big mimic's spawn smoke, `NPC.cs:77498`); kind 5 registers the
/// bestiary kill (`MessageBuffer.cs:2514-2520`). The host forces the subject
/// to the sender for player kinds (`:2485-2488`) and only kind 2 relays
/// (`:2498`) — the apply/relay arms are parked for S1-I-4. Both directions
/// (the client asks, the host echoes kind 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiscDataSync {
    pub subject: u8,
    pub kind: u8,
}

/// `msg 61 SpawnBossInvader`'s body (`NetMessage.cs:1073-1076`): two shorts —
/// the player to spawn on and the event selector. Positive selectors are
/// `NPCID` boss types gated by `NPCID.Sets.MPAllowedEnemies`
/// (`MessageBuffer.cs:2748-2753`); negatives are the event ladder: -1/-2/-3
/// invasions, -4 pumpkin moon, -5 frost moon, -6 eclipse, -7 martians, -8
/// impending doom, -10 blood moon, -11/-17 combat books, -12..-15 pet
/// licenses, -16 Mechdusa (`NPC.cs:19794`), -18 peddler's satchel, -19 slime
/// rain (`Main.cs:64752`), other negatives fall through to
/// `StartInvasion(-selector)` (`:2862-2875`). Client→server only — every
/// send site is a `netMode == 1` arm; the host answers with `7`/`78` pushes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnBossEvent {
    pub player: i16,
    pub event: i16,
}

/// `msg 65 TeleportEntity`'s body (`NetMessage.cs:1088-1109`): a BitsByte
/// (bit 0/1 = the kind's low two bits, bit 2 = "take the position from the
/// player", bit 3 = the extraInfo int follows), `(short)` entity slot, the
/// destination floats, `(byte)` teleport style, then the optional
/// `(int)` extraInfo (the portal color for style 4). Kinds: 0 player
/// teleport (`Player.cs:45513` rod-of-discord etc.), 1 NPC teleport
/// (`NPC.cs:81056`, host, style != 4), 2 player-to-player (`Player.cs:42363`),
/// 3 the client's ack decrementing `unacknowledgedTeleports`
/// (`MessageBuffer.cs:2991-2994`, `:3029-3032`). The host forces the entity
/// to the sender (`:2953-2956`), overrides the position from the live player
/// when bit 2 is set (`:2979-2982`) and counts unacknowledged teleports on
/// send (`NetMessage.cs:1104-1107`) — all S1-I-4. Both directions. Note the
/// C# writes bit 3 only when `number7 != 0`: senders pass `None` for 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TeleportEntity {
    pub entity: i16,
    pub x: f32,
    pub y: f32,
    pub style: u8,
    /// 0 player, 1 NPC, 2 player-to-player, 3 ack.
    pub kind: u8,
    pub get_pos_from_player: bool,
    pub extra_info: Option<i32>,
}

impl TeleportEntity {
    fn flags(&self) -> BitsByte {
        let mut b = BitsByte::new();
        b.set(0, self.kind & 1 == 1);
        b.set(1, self.kind & 2 == 2);
        b.set(2, self.get_pos_from_player);
        b.set(3, self.extra_info.is_some());
        b
    }
}

/// `msg 73 RequestTeleport`'s body (`NetMessage.cs:1142-1144`): one byte —
/// the teleport item kind: 0 teleportation potion, 1 magic conch
/// (`Player.cs:43029`), 2 demon conch (`:43062`), 3 shellphone home
/// (`:43103`), 4 no-space fallback (`:6069`). The host runs the matching
/// `Player` method on the sender (`MessageBuffer.cs:3127-3146`), which
/// answers with a `65`. Client→server only — every send site is a
/// `netMode == 1` arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestTeleport {
    pub kind: u8,
}

/// `msg 77 SyncTilePicking`'s body (`NetMessage.cs:1154-1159`):
/// `(short)` animationType, `(ushort)` tileType, `(short)` x, `(short)` y —
/// `Animation.NewTemporaryAnimation`'s four args verbatim
/// (`MessageBuffer.cs:3183-3185`). Sent only by the host's
/// `SendTemporaryAnimation` (`NetMessage.cs:2590-2596`, `netMode == 2`) to
/// one client; server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporaryAnimation {
    pub animation_type: i16,
    pub tile_type: u16,
    pub x: i16,
    pub y: i16,
}

/// `msg 78 ReportInvasionProgress`'s body (`NetMessage.cs:1160-1165`):
/// `(int)` progress, `(int)` progressMax, `(sbyte)` icon, `(sbyte)` wave —
/// `Main.ReportInvasionProgress`'s four args verbatim
/// (`MessageBuffer.cs:3186-3191`, client-only read). The classic invasion
/// sends icon `invasionType + 3` (`Main.cs:47683`), the moons send 1/2
/// (`NPC.cs:78583`/`:78717`), DD2 sends 3 + the wave
/// (`DD2Event.cs:191` et al.). Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvasionProgress {
    pub progress: i32,
    pub progress_max: i32,
    pub icon: i8,
    pub wave: i8,
}

/// `msg 96 TeleportPlayerThroughPortal`'s body (`NetMessage.cs:1337-1346`):
/// `(byte)` player, `(short)` number4 (the portal color index
/// `owner * 2 + ai[1]`, `PortalHelper.cs:181`), the destination floats, then
/// `WriteVector2(player.velocity)` — the player's *current* velocity read
/// from the live entity at send time, NOT the destination echoed back.
/// The portal-gun's owner client sends it after teleporting itself
/// (`PortalHelper.cs:188-192`, followed by a `13` push); the host relays
/// (`MessageBuffer.cs:3552-3555`), the client flips the color parity
/// (`num184 + (num184 % 2 == 0 ? 1 : -1)`), teleports style 4 and overwrites
/// the velocity (`:3549-3551`, parked for S1-I-4). Both directions — the
/// NPC twin (100) rides d94.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerPortalTeleport {
    pub player: u8,
    pub color_index: i16,
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

/// `msg 103 WorldFightMode`'s body (`NetMessage.cs:1383-1386`): the two
/// `NPC` statics as raw ints — `MaxMoonLordCountdown` then `MoonLordCountdown`
/// (`NPC.cs:6031-6033`). Sent by the apocalypse start (`WorldGen.cs:86755`)
/// and the join push (`MessageBuffer.cs:868`); the client stores both
/// (`:3708-3714`). Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoonlordCountdown {
    pub max: i32,
    pub countdown: i32,
}

/// `msg 112 SpecialFX`'s body (`NetMessage.cs:1424-1431`): `(byte)` effect
/// kind, `(int)` x, `(int)` y, `(byte)` number4, `(short)` number5,
/// `(byte)` number6. Kind 1 = tree-grow FX (number4 = treeHeight,
/// number5 = passStyle, number6 = the shake flag — `WorldGen.cs:29375`,
/// the shake variant `:63412`); kind 2 = fairy critter effects (number5 =
/// `type - 583`, `NPC.cs:48584`). The reader tests number6 `== 1`
/// (`MessageBuffer.cs:3822`) and relays kind 1 verbatim (`:3832`).
/// Server→client only — every send site is a `netMode == 2` arm; the
/// reader's relay branch covers a host receiving one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecialFx {
    pub effect: u8,
    pub x: i32,
    pub y: i32,
    pub b4: u8,
    pub s5: i16,
    pub b6: u8,
}

/// `msg 113 DD2EventSummonCrystal`'s body (`NetMessage.cs:1432-1435`):
/// `(short)` x, `(short)` y — the crystal's tile coords. The client sends
/// it when placing the eternia crystal (`DD2Event.SummonCrystal`,
/// `DD2Event.cs:574-577`); the host checks `WouldFailSpawningHere` and
/// summons on the sender (`MessageBuffer.cs:3841-3854`, gated on
/// `!snowMoon && !pumpkinMoon`). Client→server only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrystalInvasionStart {
    pub x: i16,
    pub y: i16,
}

/// `msg 116 DD2EventTimeLeftUntilWaveStart`'s body (`NetMessage.cs:1436-1438`):
/// one raw `(int)` — `DD2Event._timeLeftUntilSpawningBegins`
/// (`DD2Event.SetEnemySpawningOnHold`, `DD2Event.cs:947-950`, host-only
/// send). The client stores `TimeLeftBetweenWaves`
/// (`MessageBuffer.cs:3861-3866`). Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrystalInvasionWait {
    pub time_left: i32,
}

/// `msg 127 RemoveRevengeMarker`'s body (`NetMessage.cs:1518-1520`): one raw
/// `(int)` — the marker's `UniqueID`. The dedicated server sends it when a
/// marker spawns its enemy or expires/goes invalid
/// (`CoinLossRevengeSystem.cs:451`/`:473`/`:477`, all `Main.dedServ` gates);
/// the client destroys the marker (`MessageBuffer.cs:4049-4057`). The `126`
/// marker's twin (wave A). Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoveRevengeMarker {
    pub unique_id: i32,
}

/// `msg 141 LucyTheAxeMessage`'s body (`NetMessage.cs:1595-1602`):
/// `(byte)` message source, `(byte)` variation, the velocity floats, then
/// `(int)` x, `(int)` y — the position ints come AFTER the velocity on the
/// wire (`LucyAxeMessage.cs:133`'s send order). The client creates the
/// popup via `CreateFromNet(source, variation, new Vector2(x, y), velocity)`
/// (`MessageBuffer.cs:4278`); the host relays verbatim (`:4274`). Both
/// directions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LucyPopup {
    pub source: u8,
    pub variation: u8,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub x: i32,
    pub y: i32,
/// `msg 18 SetTime`'s body (`NetMessage.cs:531-536`): the day flag as one
/// byte, `(int)Main.time`, then the `Main.sunModY`/`moonModY` offsets — both
/// `short` (`Main.cs:1185`/`:1187`). The reader is client-only
/// (`MessageBuffer.cs:1290-1298`, `netMode == 1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetTime {
    pub day_time: bool,
    /// `Main.time` — the wire is an `int`; the port's clock is the f64 ticks.
    pub time: i32,
    pub sun_mod_y: i16,
    pub moon_mod_y: i16,
}

/// `msg 36 SyncPlayerZone`'s body (`NetMessage.cs:933-944`): the player id
/// then the five `Player.zone1..zone5` BitsBytes (`Player.cs:1364-1372`) and
/// `(byte)Player.townNPCs`. The host overrides the id to the sender, applies
/// the bytes and relays (`MessageBuffer.cs:2178-2202`); bit 0 of `zone5` is
/// `ZoneShimmer` (`Player.cs:3811-3821`) — its false→true edge spawns the
/// Faelings on the host (`NPC.cs:5957`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerZone {
    pub player_id: u8,
    pub zone1: u8,
    pub zone2: u8,
    pub zone3: u8,
    pub zone4: u8,
    pub zone5: u8,
    pub town_npcs: u8,
}

/// `msg 41 ItemRotationAndAnimation`'s body (`NetMessage.cs:956-960`): the
/// player id, the `itemRotation` float (`Player.cs:1498`) then
/// `(short)itemAnimation` (`Player.cs:3014` is an `int`; the writer
/// truncates). The host forces the id to the sender
/// (`MessageBuffer.cs:2273-2277`), stores both fields, re-derives `channel`
/// from the selected item (`:2283`) and relays (`:2286`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemAnim {
    pub player_id: u8,
    pub rotation: f32,
    pub animation: i16,
}

/// `msg 43 ManaEffect`'s body (`NetMessage.cs:966-969`): the player id then
/// `(short)number2` — the mana-restore amount `Player.ManaEffect` passes
/// (`Player.cs:4531-4538`). The host forces the id to the sender
/// (`MessageBuffer.cs:2309-2313`), the receiver spawns the HealMana text for
/// anyone but itself (`:2315-2318`) and the host relays (`:2321`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManaEffect {
    pub player_id: u8,
    pub amount: i16,
}

/// `msg 51 MiscDataSync`'s body (`NetMessage.cs:1010-1013`): two bytes —
/// `number` (the subject: a player slot for kinds 1/2, an NPC slot for kind
/// 4) and `number2`, the kind switch (`MessageBuffer.cs:2490-2528`):
/// 1 SpawnSkeletron, 2 the Item1 use sound at the player, 3 Sundialing
/// (host-only), 4 BigMimicSpawnSmoke, 5 the TorchGod bestiary kill
/// (host-only), 6 Moondialing (host-only). The host forces the subject to
/// the sender (`:2485-2488`). W4-c27.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiscDataSync {
    pub player_id: u8,
    pub kind: u8,
}

/// `msg 55 AddPlayerBuffPvP`'s body (`NetMessage.cs:1041-1045`): the player
/// id, `(ushort)number2` the buff type and `(int)number3` the time
/// (`MessageBuffer.cs:2596-2598`). The host applies only when BOTH players
/// are hostile and `Main.pvpBuff[type]` (`:2599`) and sends it to the TARGET
/// alone (`:2603`); the client applies only to itself (`:2599`'s second
/// clause) via `AddBuff(fromNetPvP: true)` (`:2607`, `Player.cs:5287-5290`
/// skips the time extension). W4-c27.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerBuffPvp {
    pub player_id: u8,
    pub buff_type: u16,
    pub time: i32,
}

/// `msg 62 SyncDodge`'s body (`NetMessage.cs:1077-1080`): the player id and
/// `(byte)number2`, the dodge kind — 1 Ninja, 2 Shadow, 4 BrainOfConfusion,
/// 5 MysticSash (`MessageBuffer.cs:2887-2902`; 3 is the retired Holy dodge,
/// no arm). The host forces the id to the sender (`:2883-2886`), runs the
/// dodge on the subject and relays to the others (`:2903-2906`). Send sites:
/// `Player.NinjaDodge` (`Player.cs:38163`), `ShadowDodge` (`:38038`),
/// `BrainOfConfusionDodge` (`:38053`), `DoMysticSashDodge` (`:38115`).
/// W4-c27.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncDodge {
    pub player_id: u8,
    pub kind: u8,
}

/// `msg 84 PlayerStealth`'s body (`NetMessage.cs:1197-1204`): the player id
/// then the LIVE `Main.player[id].stealth` float — the writer re-reads the
/// field, `number2` never crosses the wire. The host forces the id to the
/// sender (`MessageBuffer.cs:3249-3252`), stores the float (`:3254`) and
/// relays (`:3257`, re-read on send). Send sites: the shroomite/vortex
/// stealth transitions (`Player.cs:25753`/`:25800`/`:25843`/`:25869`/
/// `:44139`/`:44474`). W4-c27.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerStealth {
    pub player_id: u8,
    pub stealth: f32,
}

// ===========================================================================
// Messages
// ===========================================================================

/// Client→server messages. The handshake ids are 1:1; the internal events
/// (200+) are `netglue`'s ad-hoc protocol until M6-NET-2.
#[derive(Clone, Debug)]
pub enum C2s {
    /// `1 Connect` — the version literal (`MessageBuffer.cs:203`).
    Connect(String),
    /// `38 Password` — the server password, sent after `37 PasswordRequest`.
    Password(String),
    /// `4 PlayerInfo` — this client's identity.
    PlayerInfo(PlayerInfo),
    /// `5 PlayerInventory` — one inventory slot.
    Inventory {
        slot: i16,
        item: NetItem,
    },
    /// `6 WorldRequest` — no body.
    WorldRequest,
    /// `8 SpawnRequest` — the requested spawn.
    SpawnRequest(SpawnRequest),
    /// `12 PlayerSpawn` — where this client spawned.
    PlayerSpawn(PlayerSpawn),
    /// `14 PlayerActive` — this client is in-world (the wire carries the
    /// sender's id, `NetMessage.cs:515-517`; the server overrides it).
    PlayerActive {
        player_id: u8,
        active: bool,
    },
    /// `16 PlayerHP` — this client's health (same id override, `:519-523`).
    PlayerHp {
        player_id: u8,
        hp: i16,
        max_hp: i16,
    },
    /// `13 PlayerState` — this client's per-tick state (the C#'s
    /// `TrySyncingInput`/heartbeat sends, `Player.cs:6852-6916`,
    /// `Main.cs:64522-64529`).
    PlayerState(PlayerState),
    /// `82` — a NetModule envelope: the module id plus the module's own body
    /// (`NetPacket.cs:19-42`'s header; `netglue` dispatches by module id).
    NetModule {
        module: u16,
        body: Vec<u8>,
    },
    /// `28 NPCStrike` — this client's hit on an NPC (`NPC.StrikeNPC`'s
    /// `fromNet: false` send, `NPC.cs:81218`).
    NpcStrike(NpcStrike),
    /// `70 CatchNPC` — this client's bug net caught the NPC in the host's
    /// slot (`NPC.CatchNPC`'s `netMode == 1` arm, `NPC.cs:79901-79905`).
    CatchNpc {
        npc: i16,
    },
    /// `71 ReleaseNPC` — this client released a captured critter at the
    /// world point (`NPC.ReleaseNPC`'s `netMode == 1` arm,
    /// `NPC.cs:80109-80112`).
    ReleaseNpc {
        x: i32,
        y: i32,
        npc_type: i16,
        style: u8,
    },
    /// `27 Projectile` — this client's own projectile state (the owner's
    /// machine simulates and sends, `Projectile.cs:17210-17244`).
    Projectile(Box<ProjectileState>),
    /// `29` — one of this client's projectiles is gone.
    ProjectileDead {
        key: i32,
    },
    /// `154 Ping` - this client's RTT probe, no body (`Net/Ping.cs:31`).
    Ping,
    /// `31 RequestChestOpen` - ask the host for a chest's contents
    /// (`Player.cs:32897`/`:33116`).
    ChestOpenRequest(ChestOpenRequest),
    /// `32 SyncChestItem` - one chest slot this client changed
    /// (`Player.cs:31520`, the `1023` violin cases; `NetMessage.cs:1878`'s
    /// chest drops).
    ChestItem(ChestItem),
    /// `46 OpenSignRequest` - a client asks for a sign's text
    /// (`Player.cs:32720`).
    SignRequest(SignRequest),
    /// `47 OpenSignResponse` - a client submitted a sign's new text
    /// (`Main.SubmitSignText`, `Main.cs:40694-40704`).
    SignData(SignData),
    /// `33 SyncPlayerChest` - a client set the open chest / its rename
    /// (`Player.cs:32885`, `:33099`).
    PlayerChest(PlayerChest),
    /// `50 PlayerBuffs` - this client's buff ids (`Main.cs:17675`).
    PlayerBuffs(PlayerBuffs),
    /// `59 HitSwitch` - a client tripped a switch (`Collision.cs:3303-3304`,
    /// `Player.cs:32178` and the other `HitSwitch` call sites).
    HitSwitch(HitSwitch),
    /// `53 AddNPCBuff` - a client applied a buff to an NPC
    /// (`NPC.AddBuff`'s `netMode == 1` send, `NPC.cs:89932-89935`).
    AddNpcBuff(AddNpcBuff),
    /// `30 TogglePVP` - a client flipped its PvP flag
    /// (`Main.cs`'s PvP toggle; `MessageBuffer.cs:1851-1867` relays it).
    TogglePvp(TogglePvp),
    /// `42 Unknown42` - a client's mana sync (`Player.cs`'s mana sends;
    /// `MessageBuffer.cs:2290-2306`).
    PlayerMana(PlayerMana),
    /// `45 TeamChange` - a client changed team (`MessageBuffer.cs:2325-2364`).
    TeamChange(TeamChange),
    /// `142 SyncProjectileTrackers` - a client's piggy-bank / void-lens
    /// trackers (`MessageBuffer.cs:4282-4297` relays them).
    ProjectileTrackers(ProjectileTrackers),
    /// `147 SyncLoadout` - a client switched loadout
    /// (`MessageBuffer.cs:4328-4343` relays it).
    SyncLoadout(SyncLoadout),
    /// `51 MiscDataSync` - a client's sundial/moondial/rod/mimic ask
    /// (`MessageBuffer.cs:2482-2530`; kind 2 echoes back).
    MiscDataSync(MiscDataSync),
    /// `61 SpawnBossInvader` - a client used a boss/event summon item
    /// (`MessageBuffer.cs:2740-2878`'s selector ladder).
    SpawnBossEvent(SpawnBossEvent),
    /// `65 TeleportEntity` - a client teleported (kind 0/2) or acks the
    /// host's teleport (kind 3, `MessageBuffer.cs:2949-3035`).
    TeleportEntity(TeleportEntity),
    /// `73 RequestTeleport` - a client used a conch/potion/shellphone
    /// (`MessageBuffer.cs:3127-3146` runs it on the sender).
    RequestTeleport(RequestTeleport),
    /// `75 AnglerQuestFinished` - a client finished today's angler quest,
    /// no body (`MessageBuffer.cs:3154-3163` adds the sender's name;
    /// `Main.cs:40254`).
    AnglerQuestFinished,
    /// `96 TeleportPlayerThroughPortal` - the portal-gun owner teleported
    /// itself (`PortalHelper.cs:190`; the host relays, `:3552-3555`).
    PlayerPortalTeleport(PlayerPortalTeleport),
    /// `113 DD2EventSummonCrystal` - a client placed the eternia crystal
    /// (`DD2Event.cs:576`; the host summons, `MessageBuffer.cs:3841-3854`).
    CrystalInvasionStart(CrystalInvasionStart),
    /// `141 LucyTheAxeMessage` - a talking-axe popup
    /// (`LucyAxeMessage.cs:133`; the host relays, `:4274`).
    LucyPopup(LucyPopup),
    /// `143 DD2EventSkipWaitTime` - a client asked to skip the DD2 wait,
    /// no body (`DD2Event.cs:1790`, `MessageBuffer.cs:4298-4303`).
    CrystalInvasionSkip,
    /// `36 SyncPlayerZone` - a client's zone flags changed (the zone-change
    /// test `Main.cs:17634`, the 900-tick heartbeat `:64533`;
    /// `MessageBuffer.cs:2178-2202` forces the id to the sender and relays).
    PlayerZone(PlayerZone),
    /// `41 ItemRotationAndAnimation` - a client's aimed-item rotation
    /// (`Player.cs:43693`/`:47374`/`:47380` beside a `13`;
    /// `MessageBuffer.cs:2271-2289`). W4-c27.
    ItemAnim(ItemAnim),
    /// `43 ManaEffect` - a client's mana-restore text amount
    /// (`Player.ManaEffect`'s send, `Player.cs:4534-4537`;
    /// `MessageBuffer.cs:2307-2324`). W4-c27.
    ManaEffect(ManaEffect),
    /// `51 MiscDataSync` - a client's misc request (kinds 1/2/3/6;
    /// `MessageBuffer.cs:2482-2530`). W4-c27.
    MiscDataSync(MiscDataSync),
    /// `55 AddPlayerBuffPvP` - a client applied a PvP debuff to a peer
    /// (`Player.AddBuff`'s remote branch, `Player.cs:5269-5275`;
    /// `MessageBuffer.cs:2594-2611`). W4-c27.
    PlayerBuffPvp(PlayerBuffPvp),
    /// `62 SyncDodge` - a client's dodge fired (`Player.NinjaDodge`/
    /// `ShadowDodge`/`BrainOfConfusionDodge`/`DoMysticSashDodge`'s sends;
    /// `MessageBuffer.cs:2879-2908`). W4-c27.
    SyncDodge(SyncDodge),
    /// `84 PlayerStealth` - a client's stealth value changed (the
    /// shroomite/vortex transitions, `Player.cs:25753` etc.;
    /// `MessageBuffer.cs:3246-3260`). W4-c27.
    PlayerStealth(PlayerStealth),
    // ---- internal (no 1:1 body in the port's scope) ----
    Edit {
        x: i32,
        y: i32,
        kind: u8,
        tile: u16,
    },
    Pickup {
        drop: u32,
    },
}

/// Server→client messages, mirroring [`C2s`]'s families.
#[derive(Clone, Debug)]
pub enum S2c {
    /// `2 Disconnect` — the reason text (kicks use the `Lang.mp` keys the C#
    /// uses at those gates).
    Disconnect(String),
    /// `37 PasswordRequest` — no body.
    PasswordRequest,
    /// `3 PlayerId` — the assigned slot id.
    PlayerId {
        player_id: u8,
    },
    /// `4 PlayerInfo` — another peer's identity.
    PlayerInfo(PlayerInfo),
    /// `5 PlayerInventory` — one slot of another peer's inventory.
    Inventory {
        player_id: u8,
        slot: i16,
        item: NetItem,
    },
    /// `7 WorldInfo` — the world header.
    WorldInfo(Box<WorldInfo>),
    /// `9 Status` — the load progress (`max` is the message count expected,
    /// the client counts received messages — `MessageBuffer.cs:143-146`).
    Status {
        text: String,
        max: i32,
    },
    /// `49 InitialSpawn` - the server's "spawn in now", no body
    /// (`MessageBuffer.cs:874`/`:2447-2453`).
    InitialSpawn,
    /// `14 PlayerActive` — another peer's presence.
    PlayerActive {
        player_id: u8,
        active: bool,
    },
    /// `16 PlayerHP` — another peer's health.
    PlayerHp {
        player_id: u8,
        hp: i16,
        max_hp: i16,
    },
    /// `154 Ping` - the server's echo back to the prober
    /// (`MessageBuffer.cs:4419-4423`), no body.
    Ping,
    /// `97 AchievementNPCKilled` — the host's per-player kill credit
    /// (`MessageBuffer.cs:3558-3563`): one `(short)npcNetId`.
    NpcKilled { npc: i16 },
    /// `98 AchievementProgressionEvent` — the host's progression broadcast
    /// (`MessageBuffer.cs:3564-3569`): one `(short)eventID`.
    ProgressionEvent { event_id: i16 },
    /// `32 SyncChestItem` - one chest slot, relayed to the other clients
    /// (`MessageBuffer.cs:1913-1916`).
    ChestItem(ChestItem),
    /// `33 SyncPlayerChest` - the chest a player opened
    /// (`MessageBuffer.cs:1880`, `:1980`; `Player.cs:32885`).
    PlayerChest(PlayerChest),
    /// `47 OpenSignResponse` - a sign's text (`MessageBuffer.cs:2372`,
    /// `:2398`).
    SignData(SignData),
    /// `50 PlayerBuffs` - a player's buff ids (`MessageBuffer.cs:2478`).
    PlayerBuffs(PlayerBuffs),
    /// `54 NPCBuffs` - an NPC's buff pairs (`MessageBuffer.cs:2573`).
    NpcBuffs(NpcBuffs),
    /// `59 HitSwitch` - the switch a peer tripped, relayed
    /// (`MessageBuffer.cs:2691`).
    HitSwitch(HitSwitch),
    /// `80 SyncPlayerChestIndex` - which chest a player has open
    /// (`MessageBuffer.cs:1886`, `:1980`, `NetMessage.cs:2906`).
    PlayerChestIndex(PlayerChestIndex),
    /// `155 SyncChestSize` - a chest's `maxItems`, the `32` burst's header
    /// (`NetMessage.SendChestContentsTo`, `MessageBuffer.cs:4429-4438`).
    ChestSize(ChestSize),
    /// `69 ChestName` - a renamed chest, the `33` rename's relay
    /// (`MessageBuffer.cs:1977`, `:3059-3078`).
    ChestName(ChestName),
    /// `30 TogglePVP` - a peer's PvP flag, relayed by the host
    /// (`MessageBuffer.cs:1862`).
    TogglePvp(TogglePvp),
    /// `42 Unknown42` - a peer's mana (`NetMessage.cs:961-965`; the join
    /// bundle's `SyncOnePlayer` push, `:2904`).
    PlayerMana(PlayerMana),
    /// `45 TeamChange` - a peer's team, relayed by the host
    /// (`MessageBuffer.cs:2342`; join push `NetMessage.cs:2903`).
    TeamChange(TeamChange),
    /// `60 Unknown60` - an NPC's household banner, the join bundle's
    /// `SendNPCHousesAndTravelShop` push (`NetMessage.cs:2845`).
    NpcHouse(NpcHouse),
    /// `72 TravelMerchantItems` - the travel shop's stock, sent when the
    /// traveller is housed (`NetMessage.cs:2848-2851`, `:2645-2651`).
    TravelShop(TravelShop),
    /// `74 AnglerQuest` - today's angler quest, the join bundle's
    /// `SendAnglerQuest` push (`NetMessage.cs:2653-2673`).
    AnglerQuest(AnglerQuest),
    /// `126 SyncRevengeMarker` - one coin-loss revenge marker, the join
    /// bundle's `SendAllMarkersToPlayer` push
    /// (`CoinLossRevengeSystem.cs:498-507`).
    RevengeMarker(RevengeMarker),
    /// `135 DeadPlayer` - a peer died: the join bundle sends it when
    /// `statLife <= 0` (`NetMessage.cs:2897-2900`); the client raises
    /// `immuneAlpha` to 255 (`MessageBuffer.cs:4195-4203`).
    DeadPlayer { player_id: u8 },
    /// `140 SetMiscEventValues` - kind 0, the credits roll's remaining time
    /// (`CreditsRollEvent.cs:27-33`, join push `NetMessage.cs:2818`).
    MiscEventValue(MiscEventValue),
    /// `142 SyncProjectileTrackers` - a peer's trackers (join push
    /// `NetMessage.cs:2907`, relay `MessageBuffer.cs:4294`).
    ProjectileTrackers(ProjectileTrackers),
    /// `147 SyncLoadout` - a peer's loadout (join push
    /// `NetMessage.cs:2908`, relay `MessageBuffer.cs:4340`).
    SyncLoadout(SyncLoadout),
    /// `51 MiscDataSync` - the host's kind-2 rod-sound echo
    /// (`MessageBuffer.cs:2498`).
    MiscDataSync(MiscDataSync),
    /// `65 TeleportEntity` - the host's player/NPC teleport broadcast
    /// (`MessageBuffer.cs:2989`/`:3008`, `NPC.cs:81056`).
    TeleportEntity(TeleportEntity),
    /// `77 SyncTilePicking` - a temporary tile animation for one client
    /// (`NetMessage.SendTemporaryAnimation`, `MessageBuffer.cs:3183-3185`).
    TemporaryAnimation(TemporaryAnimation),
    /// `78 ReportInvasionProgress` - the invasion/moon/DD2 progress bar
    /// (`Main.cs:47683`, `DD2Event.cs:191`; `MessageBuffer.cs:3186-3191`).
    InvasionProgress(InvasionProgress),
    /// `96 TeleportPlayerThroughPortal` - the host's relay of a player's
    /// portal teleport (`MessageBuffer.cs:3554`; style 4 + parity flip).
    PlayerPortalTeleport(PlayerPortalTeleport),
    /// `103 WorldFightMode` - the Moon Lord countdown statics
    /// (`WorldGen.cs:86755`, join push `MessageBuffer.cs:868`).
    MoonlordCountdown(MoonlordCountdown),
    /// `112 SpecialFX` - tree-grow / fairy effects
    /// (`WorldGen.cs:29375`, `NPC.cs:48584`; `MessageBuffer.cs:3815-3840`).
    SpecialFx(SpecialFx),
    /// `114 DD2EventWipeEntities` - the host wiped the Old One's Army,
    /// no body (`DD2Event.cs:636`, `MessageBuffer.cs:3855-3860`).
    CrystalInvasionWipe,
    /// `116 DD2EventTimeLeftUntilWaveStart` - the between-waves countdown
    /// (`DD2Event.cs:949`, `MessageBuffer.cs:3861-3866`).
    CrystalInvasionWait(CrystalInvasionWait),
    /// `127 RemoveRevengeMarker` - a coin-loss revenge marker is gone
    /// (`CoinLossRevengeSystem.cs:451`, `MessageBuffer.cs:4049-4057`).
    RemoveRevengeMarker(RemoveRevengeMarker),
    /// `141 LucyTheAxeMessage` - the host's relay of a talking-axe popup
    /// (`MessageBuffer.cs:4274`; the client CreateFromNet, `:4278`).
    LucyPopup(LucyPopup),
    /// `18 SetTime` - the host's clock (`NetMessage.cs:531-536`; the client
    /// applies it at `netMode == 1`, `MessageBuffer.cs:1290-1298`). W4-c27.
    SetTime(SetTime),
    /// `36 SyncPlayerZone` - a peer's zone flags, relayed by the host
    /// (`MessageBuffer.cs:2199`). W4-c27.
    PlayerZone(PlayerZone),
    /// `41 ItemRotationAndAnimation` - a peer's aimed-item rotation, relayed
    /// by the host (`MessageBuffer.cs:2286`). W4-c27.
    ItemAnim(ItemAnim),
    /// `43 ManaEffect` - a peer's mana-restore text amount, relayed by the
    /// host (`MessageBuffer.cs:2321`). W4-c27.
    ManaEffect(ManaEffect),
    /// `51 MiscDataSync` - the host's misc fan-out (kinds 2/4;
    /// `MessageBuffer.cs:2498`, `:2512`). W4-c27.
    MiscDataSync(MiscDataSync),
    /// `55 AddPlayerBuffPvP` - a PvP debuff aimed at us, sent by the host to
    /// the target alone (`MessageBuffer.cs:2603`). W4-c27.
    PlayerBuffPvp(PlayerBuffPvp),
    /// `62 SyncDodge` - a peer's dodge, relayed by the host
    /// (`MessageBuffer.cs:2905`). W4-c27.
    SyncDodge(SyncDodge),
    /// `84 PlayerStealth` - a peer's stealth value, relayed by the host
    /// (`MessageBuffer.cs:3257` — the writer re-reads the stored float).
    /// W4-c27.
    PlayerStealth(PlayerStealth),
    /// `82` - a NetModule envelope (see `C2s::NetModule`).
    NetModule {
        module: u16,
        body: Vec<u8>,
    },
    /// `27 Projectile` — one projectile's spawn/update state
    /// (`NetMessage.cs:780-841`).
    Projectile(Box<ProjectileState>),
    /// `29` — the projectile is gone (`NetMessage.cs`'s case 29; a NaN
    /// position deactivates it silently).
    ProjectileDead {
        key: i32,
        x: f32,
        y: f32,
    },
    /// `28 NPCStrike` — a validated strike, relayed by the host.
    NpcStrike(NpcStrike),
    /// `10 TileBlock` — one section's tile snapshot (the C#'s
    /// `SendSection`, `NetMessage.cs:2697-2712`).
    TileBlock(Box<TileBlock>),
    /// `20 SectionRange` — a rectangle of tile changes
    /// (`SendTileSquare`, `NetMessage.cs:2627-2630`).
    SectionRange(Box<SectionRange>),
    /// `23 NPCInfo` — one NPC's state (the C#'s per-NPC `SendData(23)`,
    /// `NPC.cs:90613`).
    NpcInfo(Box<NpcInfo>),
    /// `21 ItemDrop` — one drop's state.
    ItemState(Box<ItemState>),
    /// `151` — the item slot is gone (`NetMessage.cs:96-99`'s air rewrite).
    ItemDead {
        slot: i16,
    },
    /// `22 ItemOwner` — the reservation announcement.
    ItemOwner(Box<ItemOwner>),
    // ---- internal (no 1:1 body in the port's scope) ----
    Joined {
        id: u32,
        name: String,
    },
    Left {
        id: u32,
    },
    PlayerState(Box<PlayerState>),
    Time {
        time: f64,
        day: bool,
    },
    Edit {
        x: i32,
        y: i32,
        kind: u8,
        tile: u16,
    },
}

impl C2s {
    pub fn msg_id(&self) -> u8 {
        match self {
            Self::Connect(_) => id::CONNECT,
            Self::Password(_) => id::PASSWORD,
            Self::PlayerInfo(_) => id::PLAYER_INFO,
            Self::Inventory { .. } => id::PLAYER_INVENTORY,
            Self::WorldRequest => id::WORLD_REQUEST,
            Self::SpawnRequest(_) => id::SPAWN_REQUEST,
            Self::PlayerSpawn(_) => id::PLAYER_SPAWN,
            Self::PlayerActive { .. } => id::PLAYER_ACTIVE,
            Self::PlayerHp { .. } => id::PLAYER_HP,
            Self::PlayerState(_) => id::PLAYER_STATE,
            Self::NetModule { .. } => id::NET_MODULE,
            Self::NpcStrike(_) => id::NPC_STRIKE,
            Self::CatchNpc { .. } => id::CATCH_NPC,
            Self::ReleaseNpc { .. } => id::RELEASE_NPC,
            Self::Projectile(_) => id::PROJECTILE,
            Self::ProjectileDead { .. } => id::PROJECTILE_DEAD,
            Self::Ping => id::PING,
            Self::ChestOpenRequest(_) => id::CHEST_OPEN_REQUEST,
            Self::ChestItem(_) => id::CHEST_ITEM,
            Self::SignRequest(_) => id::SIGN_REQUEST,
            Self::SignData(_) => id::SIGN_DATA,
            Self::HitSwitch(_) => id::HIT_SWITCH,
            Self::AddNpcBuff(_) => id::ADD_NPC_BUFF,
            Self::PlayerChest(_) => id::PLAYER_CHEST,
            Self::PlayerBuffs(_) => id::PLAYER_BUFFS,
            Self::TogglePvp(_) => id::TOGGLE_PVP,
            Self::PlayerMana(_) => id::PLAYER_MANA,
            Self::TeamChange(_) => id::TEAM_CHANGE,
            Self::ProjectileTrackers(_) => id::PROJECTILE_TRACKERS,
            Self::SyncLoadout(_) => id::SYNC_LOADOUT,
            Self::MiscDataSync(_) => id::MISC_DATA_SYNC,
            Self::SpawnBossEvent(_) => id::SPAWN_BOSS_EVENT,
            Self::TeleportEntity(_) => id::TELEPORT_ENTITY,
            Self::RequestTeleport(_) => id::REQUEST_TELEPORT,
            Self::AnglerQuestFinished => id::ANGLER_QUEST_FINISHED,
            Self::PlayerPortalTeleport(_) => id::TELEPORT_PLAYER_PORTAL,
            Self::CrystalInvasionStart(_) => id::CRYSTAL_INVASION_START,
            Self::LucyPopup(_) => id::LUCY_POPUP,
            Self::CrystalInvasionSkip => id::CRYSTAL_INVASION_SKIP,
            Self::PlayerZone(_) => id::SYNC_PLAYER_ZONE,
            Self::ItemAnim(_) => id::ITEM_ANIM,
            Self::ManaEffect(_) => id::MANA_EFFECT,
            Self::MiscDataSync(_) => id::MISC_DATA_SYNC,
            Self::PlayerBuffPvp(_) => id::ADD_PLAYER_BUFF_PVP,
            Self::SyncDodge(_) => id::SYNC_DODGE,
            Self::PlayerStealth(_) => id::PLAYER_STEALTH,
            Self::Edit { .. } => id::INT_EDIT,
            Self::Pickup { .. } => id::INT_PICKUP,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let w = NetWriter::open(self.msg_id());
        let w = match self {
            Self::Connect(v) => w.str(v),
            Self::Password(v) => w.str(v),
            Self::PlayerInfo(pi) => pi.write_to(w),
            Self::Inventory { slot, item } => w
                .i16(*slot)
                .i16(item.stack)
                .u8(item.prefix)
                .i16(item.kind)
                .bitsbyte({
                    let mut b = BitsByte::new();
                    b.set(0, item.favorited);
                    b
                }),
            Self::WorldRequest => w,
            Self::Ping => w,
            Self::ChestOpenRequest(c) => w.i16(c.x).i16(c.y),
            Self::ChestItem(c) => w
                .i16(c.chest)
                .u8(c.slot)
                .i16(c.stack)
                .u8(c.prefix)
                .i16(c.kind),
            Self::SignRequest(s) => w.i16(s.x).i16(s.y),
            Self::SignData(s) => w
                .i16(s.sign)
                .i16(s.x)
                .i16(s.y)
                .str(&s.text)
                .u8(s.sender)
                .u8(s.flags),
            Self::PlayerChest(c) => {
                // `NetMessage.cs:903-910`: the length byte, then the C#
                // `BinaryWriter.Write(string)` (a 7-bit length + bytes) when
                // the name was accepted — `255`/`0` write no text at all.
                let w = w.i16(c.chest).i16(c.x).i16(c.y).u8(c.name_len);
                if c.name_len == 0 || c.name_len == 255 {
                    w
                } else {
                    w.str(&c.name)
                }
            }
            Self::PlayerBuffs(b) => {
                let mut w = w.u8(b.player_id);
                for id in &b.buffs {
                    w = w.u16(*id);
                }
                w.u16(0)
            }
            Self::HitSwitch(h) => w.i16(h.x).i16(h.y),
            Self::AddNpcBuff(a) => w.i16(a.npc).u16(a.buff_type).i16(a.time),
            Self::TogglePvp(t) => w.u8(t.player_id).bool(t.hostile),
            Self::PlayerMana(m) => w.u8(m.player_id).i16(m.mana).i16(m.max_mana),
            Self::TeamChange(t) => w.u8(t.player_id).u8(t.team),
            Self::ProjectileTrackers(t) => w
                .u8(t.player_id)
                .i32(t.piggy_bank.key)
                .i16(t.piggy_bank.kind)
                .i32(t.void_lens.key)
                .i16(t.void_lens.kind),
            Self::SyncLoadout(l) => w.u8(l.player_id).u8(l.loadout).u16(l.hide_accessory),
            // `NetMessage.cs:1010-1013`: (byte)subject + (byte)kind.
            Self::MiscDataSync(m) => w.u8(m.subject).u8(m.kind),
            // `NetMessage.cs:1073-1076`: (short)player + (short)event.
            Self::SpawnBossEvent(e) => w.i16(e.player).i16(e.event),
            // `NetMessage.cs:1088-1103`: BitsByte + (short)entity + the
            // destination floats + (byte)style + the gated (int)extraInfo.
            Self::TeleportEntity(t) => {
                let w = w
                    .bitsbyte(t.flags())
                    .i16(t.entity)
                    .f32(t.x)
                    .f32(t.y)
                    .u8(t.style);
                match t.extra_info {
                    Some(v) => w.i32(v),
                    None => w,
                }
            }
            // `NetMessage.cs:1142-1144`: (byte)kind.
            Self::RequestTeleport(q) => w.u8(q.kind),
            // No writer arm — bodyless (`MessageBuffer.cs:3154-3163`).
            Self::AnglerQuestFinished => w,
            // `NetMessage.cs:1337-1345`: (byte)player + (short)color + the
            // destination floats + WriteVector2(the player's live velocity).
            Self::PlayerPortalTeleport(p) => w
                .u8(p.player)
                .i16(p.color_index)
                .f32(p.x)
                .f32(p.y)
                .f32(p.velocity_x)
                .f32(p.velocity_y),
            // `NetMessage.cs:1432-1435`: (short)x + (short)y.
            Self::CrystalInvasionStart(c) => w.i16(c.x).i16(c.y),
            // `NetMessage.cs:1595-1602`: (byte)source + (byte)variation +
            // the velocity floats + (int)x + (int)y.
            Self::LucyPopup(l) => w
                .u8(l.source)
                .u8(l.variation)
                .f32(l.velocity_x)
                .f32(l.velocity_y)
                .i32(l.x)
                .i32(l.y),
            // No writer arm — bodyless (`DD2Event.cs:1790` SendData(143)).
            Self::CrystalInvasionSkip => w,
            // `NetMessage.cs:933-944`: (byte)number + zone1..5 + (byte)townNPCs.
            Self::PlayerZone(z) => w
                .u8(z.player_id)
                .u8(z.zone1)
                .u8(z.zone2)
                .u8(z.zone3)
                .u8(z.zone4)
                .u8(z.zone5)
                .u8(z.town_npcs),
            // `NetMessage.cs:956-960`: (byte)number + itemRotation +
            // (short)itemAnimation.
            Self::ItemAnim(a) => w.u8(a.player_id).f32(a.rotation).i16(a.animation),
            // `NetMessage.cs:966-969`: (byte)number + (short)number2.
            Self::ManaEffect(m) => w.u8(m.player_id).i16(m.amount),
            // `NetMessage.cs:1010-1013`: (byte)number + (byte)number2.
            Self::MiscDataSync(d) => w.u8(d.player_id).u8(d.kind),
            // `NetMessage.cs:1041-1045`: (byte)number + (ushort)number2 +
            // (int)number3.
            Self::PlayerBuffPvp(b) => w.u8(b.player_id).u16(b.buff_type).i32(b.time),
            // `NetMessage.cs:1077-1080`: (byte)number + (byte)number2.
            Self::SyncDodge(d) => w.u8(d.player_id).u8(d.kind),
            // `NetMessage.cs:1197-1204`: (byte)id + the live stealth float.
            Self::PlayerStealth(s) => w.u8(s.player_id).f32(s.stealth),
            Self::SpawnRequest(s) => w.i32(s.x).i32(s.y).u8(s.team),
            Self::PlayerSpawn(s) => w
                .i16(s.x)
                .i16(s.y)
                .i32(s.respawn_timer)
                .i16(s.deaths_pve)
                .i16(s.deaths_pvp)
                .u8(s.team)
                .u8(s.spawn_context),
            Self::PlayerActive { player_id, active } => w.u8(*player_id).u8(u8::from(*active)),
            Self::PlayerHp {
                player_id,
                hp,
                max_hp,
            } => w.u8(*player_id).i16(*hp).i16(*max_hp),
            Self::PlayerState(ps) => ps.write_to(w),
            Self::Edit { x, y, kind, tile } => w.i32(*x).i32(*y).u8(*kind).u16(*tile),
            Self::Pickup { drop } => w.u32(*drop),
            Self::NetModule { module, body } => w.u16(*module).raw(body),
            Self::NpcStrike(st) => st.write_to(w),
            // `NetMessage.cs:1125-1127` — one `(short)` slot.
            Self::CatchNpc { npc } => w.i16(*npc),
            // `NetMessage.cs:1128-1133` — x, y, `(short)` type, `(byte)` style.
            Self::ReleaseNpc { x, y, npc_type, style } => {
                w.i32(*x).i32(*y).i16(*npc_type).u8(*style)
            }
            Self::Projectile(p) => return p.frame(),
            Self::ProjectileDead { key } => {
                return NetWriter::open(id::PROJECTILE_DEAD)
                    .i32(*key)
                    .f32(f32::NAN)
                    .f32(f32::NAN)
                    .finish();
            }
        };
        w.finish()
    }

    /// Decodes the body after `msg_id` was consumed. `None` = malformed.
    pub fn decode(msg_id: u8, r: &mut NetReader) -> Option<Self> {
        Some(match msg_id {
            id::CONNECT => Self::Connect(r.str()?),
            id::PASSWORD => Self::Password(r.str()?),
            id::PLAYER_INFO => Self::PlayerInfo(PlayerInfo::read_from(r)?),
            id::PLAYER_INVENTORY => {
                let slot = r.i16()?;
                let stack = r.i16()?;
                let prefix = r.u8()?;
                let kind = r.i16()?;
                let flags = r.bitsbyte()?;
                Self::Inventory {
                    slot,
                    item: NetItem {
                        kind,
                        // The C# clamps the stack to ≥ 0 (`MessageBuffer.cs:400`).
                        stack: stack.max(0),
                        prefix,
                        favorited: flags.get(0),
                    },
                }
            }
            id::WORLD_REQUEST => Self::WorldRequest,
            id::PING => Self::Ping,
            id::CHEST_OPEN_REQUEST => Self::ChestOpenRequest(ChestOpenRequest {
                x: r.i16()?,
                y: r.i16()?,
            }),
            id::CHEST_ITEM => Self::ChestItem(ChestItem {
                chest: r.i16()?,
                slot: r.u8()?,
                stack: r.i16()?,
                prefix: r.u8()?,
                kind: r.i16()?,
            }),
            id::SIGN_REQUEST => Self::SignRequest(SignRequest {
                x: r.i16()?,
                y: r.i16()?,
            }),
            id::SIGN_DATA => Self::SignData(read_sign_data(r)?),
            id::PLAYER_CHEST => Self::PlayerChest(read_player_chest(r)?),
            id::PLAYER_BUFFS => Self::PlayerBuffs(PlayerBuffs {
                player_id: r.u8()?,
                buffs: read_terminated_u16(r),
            }),
            id::HIT_SWITCH => Self::HitSwitch(HitSwitch {
                x: r.i16()?,
                y: r.i16()?,
            }),
            id::ADD_NPC_BUFF => Self::AddNpcBuff(AddNpcBuff {
                npc: r.i16()?,
                buff_type: r.u16()?,
                time: r.i16()?,
            }),
            id::TOGGLE_PVP => Self::TogglePvp(TogglePvp {
                player_id: r.u8()?,
                hostile: r.bool()?,
            }),
            id::PLAYER_MANA => Self::PlayerMana(PlayerMana {
                player_id: r.u8()?,
                mana: r.i16()?,
                max_mana: r.i16()?,
            }),
            id::TEAM_CHANGE => Self::TeamChange(TeamChange {
                player_id: r.u8()?,
                team: r.u8()?,
            }),
            id::PROJECTILE_TRACKERS => Self::ProjectileTrackers(ProjectileTrackers {
                player_id: r.u8()?,
                piggy_bank: TrackedProjectileRef {
                    key: r.i32()?,
                    kind: r.i16()?,
                },
                void_lens: TrackedProjectileRef {
                    key: r.i32()?,
                    kind: r.i16()?,
                },
            }),
            id::SYNC_LOADOUT => Self::SyncLoadout(SyncLoadout {
                player_id: r.u8()?,
                loadout: r.u8()?,
                hide_accessory: r.u16()?,
            }),
            id::MISC_DATA_SYNC => Self::MiscDataSync(MiscDataSync {
                subject: r.u8()?,
                kind: r.u8()?,
            }),
            id::SPAWN_BOSS_EVENT => Self::SpawnBossEvent(SpawnBossEvent {
                player: r.i16()?,
                event: r.i16()?,
            }),
            id::TELEPORT_ENTITY => Self::TeleportEntity(read_teleport_entity(r)?),
            id::REQUEST_TELEPORT => Self::RequestTeleport(RequestTeleport { kind: r.u8()? }),
            id::ANGLER_QUEST_FINISHED => Self::AnglerQuestFinished,
            id::TELEPORT_PLAYER_PORTAL => Self::PlayerPortalTeleport(PlayerPortalTeleport {
                player: r.u8()?,
                color_index: r.i16()?,
                x: r.f32()?,
                y: r.f32()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
            }),
            id::CRYSTAL_INVASION_START => Self::CrystalInvasionStart(CrystalInvasionStart {
                x: r.i16()?,
                y: r.i16()?,
            }),
            id::LUCY_POPUP => Self::LucyPopup(LucyPopup {
                source: r.u8()?,
                variation: r.u8()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
                x: r.i32()?,
                y: r.i32()?,
            }),
            id::CRYSTAL_INVASION_SKIP => Self::CrystalInvasionSkip,
            // `MessageBuffer.cs:2180-2192`: (byte)id + zone1..5 + townNPCs.
            id::SYNC_PLAYER_ZONE => Self::PlayerZone(PlayerZone {
                player_id: r.u8()?,
                zone1: r.u8()?,
                zone2: r.u8()?,
                zone3: r.u8()?,
                zone4: r.u8()?,
                zone5: r.u8()?,
                town_npcs: r.u8()?,
            }),
            // `MessageBuffer.cs:2273-2282`: (byte)id + float + (short).
            id::ITEM_ANIM => Self::ItemAnim(ItemAnim {
                player_id: r.u8()?,
                rotation: r.f32()?,
                animation: r.i16()?,
            }),
            // `MessageBuffer.cs:2309-2314`: (byte)id + (short)amount.
            id::MANA_EFFECT => Self::ManaEffect(ManaEffect {
                player_id: r.u8()?,
                amount: r.i16()?,
            }),
            // `MessageBuffer.cs:2484-2489`: two bytes.
            id::MISC_DATA_SYNC => Self::MiscDataSync(MiscDataSync {
                player_id: r.u8()?,
                kind: r.u8()?,
            }),
            // `MessageBuffer.cs:2596-2598`: byte + ushort + int.
            id::ADD_PLAYER_BUFF_PVP => Self::PlayerBuffPvp(PlayerBuffPvp {
                player_id: r.u8()?,
                buff_type: r.u16()?,
                time: r.i32()?,
            }),
            // `MessageBuffer.cs:2881-2882`: two bytes.
            id::SYNC_DODGE => Self::SyncDodge(SyncDodge {
                player_id: r.u8()?,
                kind: r.u8()?,
            }),
            // `MessageBuffer.cs:3248-3253`: byte + float.
            id::PLAYER_STEALTH => Self::PlayerStealth(PlayerStealth {
                player_id: r.u8()?,
                stealth: r.f32()?,
            }),
            id::SPAWN_REQUEST => Self::SpawnRequest(SpawnRequest {
                x: r.i32()?,
                y: r.i32()?,
                team: r.u8()?,
            }),
            id::PLAYER_SPAWN => Self::PlayerSpawn(PlayerSpawn {
                x: r.i16()?,
                y: r.i16()?,
                respawn_timer: r.i32()?,
                deaths_pve: r.i16()?,
                deaths_pvp: r.i16()?,
                team: r.u8()?,
                spawn_context: r.u8()?,
            }),
            id::PLAYER_ACTIVE => Self::PlayerActive {
                player_id: r.u8()?,
                active: r.u8()? != 0,
            },
            id::PLAYER_HP => Self::PlayerHp {
                player_id: r.u8()?,
                hp: r.i16()?,
                max_hp: r.i16()?,
            },
            id::PLAYER_STATE => Self::PlayerState(PlayerState::read_from(r)?),
            id::INT_EDIT => Self::Edit {
                x: r.i32()?,
                y: r.i32()?,
                kind: r.u8()?,
                tile: r.u16()?,
            },
            id::INT_PICKUP => Self::Pickup { drop: r.u32()? },
            id::NET_MODULE => Self::NetModule {
                module: r.u16()?,
                body: r.rest()?.to_vec(),
            },
            id::NPC_STRIKE => Self::NpcStrike(NpcStrike::read_from(r)?),
            // `MessageBuffer.cs:3102-3110` — one `(short)` slot.
            id::CATCH_NPC => Self::CatchNpc { npc: r.i16()? },
            // `MessageBuffer.cs:3112-3117` — x, y, type, style.
            id::RELEASE_NPC => Self::ReleaseNpc {
                x: r.i32()?,
                y: r.i32()?,
                npc_type: r.i16()?,
                style: r.u8()?,
            },
            id::PROJECTILE => Self::Projectile(Box::new(ProjectileState::read_from(r)?)),
            id::PROJECTILE_DEAD => Self::ProjectileDead { key: r.i32()? },
            _ => return None,
        })
    }
}

/// The zero-terminated `ushort` run the C# uses for the buff lists
/// (`MessageBuffer.cs:2468-2473`, `:2584-2589`) - it stops at the first 0.
fn read_terminated_u16(r: &mut NetReader) -> Vec<u16> {
    let mut out = Vec::new();
    while let Some(id) = r.u16() {
        if id == 0 {
            break;
        }
        out.push(id);
    }
    out
}

/// `47 OpenSignResponse` (`MessageBuffer.cs:2376-2383`): index, tile, text,
/// then the two flag bytes.
fn read_sign_data(r: &mut NetReader) -> Option<SignData> {
    Some(SignData {
        sign: r.i16()?,
        x: r.i16()?,
        y: r.i16()?,
        text: r.str()?,
        sender: r.u8()?,
        flags: r.u8()?,
    })
}

/// `33 SyncPlayerChest` (`MessageBuffer.cs:1920-1937`): the length byte gates
/// the name - `1..=20` reads it, `255` is a refused name and anything else is
/// treated as "no name" (`num24 = 0`).
fn read_player_chest(r: &mut NetReader) -> Option<PlayerChest> {
    let chest = r.i16()?;
    let x = r.i16()?;
    let y = r.i16()?;
    let mut name_len = r.u8()?;
    let name = if name_len != 0 {
        if name_len <= 20 {
            let text = r.str()?;
            name_len = text.len() as u8;
            text
        } else {
            if name_len != 255 {
                name_len = 0;
            }
            String::new()
        }
    } else {
        String::new()
    };
    Some(PlayerChest {
        chest,
        x,
        y,
        name_len,
        name,
    })
}

/// `65 TeleportEntity` (`MessageBuffer.cs:2949-2978`): the BitsByte first,
/// `(short)` entity, the destination Vector2, `(byte)` style, then the
/// `(int)` extraInfo only when bit 3 is set; kind is reassembled from bits
/// 0/1 (`num82++` / `num82 += 2`).
fn read_teleport_entity(r: &mut NetReader) -> Option<TeleportEntity> {
    let flags = r.bitsbyte()?;
    let entity = r.i16()?;
    let x = r.f32()?;
    let y = r.f32()?;
    let style = r.u8()?;
    let mut kind = 0u8;
    if flags.get(0) {
        kind += 1;
    }
    if flags.get(1) {
        kind += 2;
    }
    let extra_info = if flags.get(3) { Some(r.i32()?) } else { None };
    Some(TeleportEntity {
        entity,
        x,
        y,
        style,
        kind,
        get_pos_from_player: flags.get(2),
        extra_info,
    })
}

impl S2c {
    pub fn msg_id(&self) -> u8 {
        match self {
            Self::Disconnect(_) => id::DISCONNECT,
            Self::PasswordRequest => id::PASSWORD_REQUEST,
            Self::PlayerId { .. } => id::PLAYER_ID,
            Self::PlayerInfo(_) => id::PLAYER_INFO,
            Self::Inventory { .. } => id::PLAYER_INVENTORY,
            Self::WorldInfo(_) => id::WORLD_INFO,
            Self::Status { .. } => id::STATUS,
            Self::TileBlock(_) => id::TILE_BLOCK,
            Self::SectionRange(_) => id::SECTION_RANGE,
            Self::NetModule { .. } => id::NET_MODULE,
            Self::Projectile(_) => id::PROJECTILE,
            Self::ProjectileDead { .. } => id::PROJECTILE_DEAD,
            Self::NpcStrike(_) => id::NPC_STRIKE,
            Self::InitialSpawn => id::INITIAL_SPAWN,
            Self::Ping => id::PING,
            Self::NpcKilled { .. } => id::NPC_KILLED,
            Self::ProgressionEvent { .. } => id::PROGRESSION_EVENT,
            Self::PlayerActive { .. } => id::PLAYER_ACTIVE,
            Self::PlayerHp { .. } => id::PLAYER_HP,
            Self::NpcInfo(_) => id::NPC_INFO,
            Self::ItemState(_) => id::ITEM_DROP,
            Self::ItemDead { .. } => id::ITEM_DEAD,
            Self::ItemOwner(_) => id::ITEM_OWNER,
            Self::Joined { .. } => id::INT_JOINED,
            Self::Left { .. } => id::INT_LEFT,
            Self::PlayerState(_) => id::PLAYER_STATE,
            Self::Time { .. } => id::INT_TIME,
            Self::Edit { .. } => id::INT_EDIT,
            Self::ChestItem(_) => id::CHEST_ITEM,
            Self::PlayerChest(_) => id::PLAYER_CHEST,
            Self::SignData(_) => id::SIGN_DATA,
            Self::PlayerBuffs(_) => id::PLAYER_BUFFS,
            Self::NpcBuffs(_) => id::NPC_BUFFS,
            Self::HitSwitch(_) => id::HIT_SWITCH,
            Self::PlayerChestIndex(_) => id::PLAYER_CHEST_INDEX,
            Self::ChestSize(_) => id::CHEST_SIZE,
            Self::ChestName(_) => id::CHEST_NAME,
            Self::TogglePvp(_) => id::TOGGLE_PVP,
            Self::PlayerMana(_) => id::PLAYER_MANA,
            Self::TeamChange(_) => id::TEAM_CHANGE,
            Self::NpcHouse(_) => id::NPC_HOUSE,
            Self::TravelShop(_) => id::TRAVEL_SHOP,
            Self::AnglerQuest(_) => id::ANGLER_QUEST,
            Self::RevengeMarker(_) => id::REVENGE_MARKER,
            Self::DeadPlayer { .. } => id::DEAD_PLAYER,
            Self::MiscEventValue(_) => id::MISC_EVENT_VALUES,
            Self::ProjectileTrackers(_) => id::PROJECTILE_TRACKERS,
            Self::SyncLoadout(_) => id::SYNC_LOADOUT,
            Self::MiscDataSync(_) => id::MISC_DATA_SYNC,
            Self::TeleportEntity(_) => id::TELEPORT_ENTITY,
            Self::TemporaryAnimation(_) => id::TEMPORARY_ANIMATION,
            Self::InvasionProgress(_) => id::INVASION_PROGRESS,
            Self::PlayerPortalTeleport(_) => id::TELEPORT_PLAYER_PORTAL,
            Self::MoonlordCountdown(_) => id::MOONLORD_COUNTDOWN,
            Self::SpecialFx(_) => id::SPECIAL_FX,
            Self::CrystalInvasionWipe => id::CRYSTAL_INVASION_WIPE,
            Self::CrystalInvasionWait(_) => id::CRYSTAL_INVASION_WAIT,
            Self::RemoveRevengeMarker(_) => id::REMOVE_REVENGE_MARKER,
            Self::LucyPopup(_) => id::LUCY_POPUP,
            Self::SetTime(_) => id::SET_TIME,
            Self::PlayerZone(_) => id::SYNC_PLAYER_ZONE,
            Self::ItemAnim(_) => id::ITEM_ANIM,
            Self::ManaEffect(_) => id::MANA_EFFECT,
            Self::MiscDataSync(_) => id::MISC_DATA_SYNC,
            Self::PlayerBuffPvp(_) => id::ADD_PLAYER_BUFF_PVP,
            Self::SyncDodge(_) => id::SYNC_DODGE,
            Self::PlayerStealth(_) => id::PLAYER_STEALTH,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        // The tile messages build their frame directly: the deflate blob and
        // the uncompressed per-tile bodies bypass the builder chain.
        match self {
            Self::TileBlock(b) => return b.frame(),
            Self::SectionRange(r) => return r.frame(),
            Self::NetModule { module, body } => {
                return NetWriter::open(id::NET_MODULE)
                    .u16(*module)
                    .raw(body)
                    .finish();
            }
            Self::Projectile(p) => return p.frame(),
            Self::ProjectileDead { key, x, y } => {
                return NetWriter::open(id::PROJECTILE_DEAD)
                    .i32(*key)
                    .f32(*x)
                    .f32(*y)
                    .finish();
            }
            _ => {}
        }
        let w = NetWriter::open(self.msg_id());
        let w = match self {
            Self::Disconnect(reason) => NetText(reason.clone()).write_to(w),
            Self::PasswordRequest => w,
            Self::PlayerId { player_id } => w.u8(*player_id).bool(false),
            Self::PlayerInfo(pi) => pi.write_to(w),
            Self::Inventory {
                player_id,
                slot,
                item,
            } => w
                .u8(*player_id)
                .i16(*slot)
                .i16(item.stack)
                .u8(item.prefix)
                .i16(item.kind)
                .bitsbyte({
                    let mut b = BitsByte::new();
                    b.set(0, item.favorited);
                    b
                }),
            Self::WorldInfo(wi) => wi.write_to(w),
            Self::Status { text, max } => NetText(text.clone()).write_to(w.i32(*max)).u8(0),
            Self::InitialSpawn => w,
            Self::Ping => w,
            Self::NpcKilled { npc } => w.i16(*npc),
            Self::ProgressionEvent { event_id } => w.i16(*event_id),
            Self::PlayerActive { player_id, active } => w.u8(*player_id).u8(u8::from(*active)),
            Self::PlayerHp {
                player_id,
                hp,
                max_hp,
            } => w.u8(*player_id).i16(*hp).i16(*max_hp),
            Self::NpcInfo(n) => n.write_to(w),
            Self::ItemState(d) => d.write_to(w),
            Self::ItemDead { slot } => w.i16(*slot),
            Self::ItemOwner(o) => o.write_to(w),
            Self::Joined { id, name } => w.u32(*id).str(name),
            Self::Left { id } => w.u32(*id),
            Self::PlayerState(ps) => ps.write_to(w),
            Self::Time { time, day } => w.f64(*time).bool(*day),
            Self::NpcStrike(st) => st.write_to(w),
            Self::Edit { x, y, kind, tile } => w.i32(*x).i32(*y).u8(*kind).u16(*tile),
            Self::ChestItem(c) => w
                .i16(c.chest)
                .u8(c.slot)
                .i16(c.stack)
                .u8(c.prefix)
                .i16(c.kind),
            Self::PlayerChest(c) => {
                // `NetMessage.cs:903-910`: the length byte, then the C#
                // `BinaryWriter.Write(string)` (a 7-bit length + bytes) when
                // the name was accepted — `255`/`0` write no text at all.
                let w = w.i16(c.chest).i16(c.x).i16(c.y).u8(c.name_len);
                if c.name_len == 0 || c.name_len == 255 {
                    w
                } else {
                    w.str(&c.name)
                }
            }
            Self::SignData(s) => w
                .i16(s.sign)
                .i16(s.x)
                .i16(s.y)
                .str(&s.text)
                .u8(s.sender)
                .u8(s.flags),
            Self::PlayerBuffs(b) => {
                let mut w = w.u8(b.player_id);
                for id in &b.buffs {
                    w = w.u16(*id);
                }
                w.u16(0)
            }
            Self::NpcBuffs(nb) => {
                let mut w = w.i16(nb.npc);
                for (id, time) in &nb.buffs {
                    w = w.u16(*id).u16(*time);
                }
                w.u16(0)
            }
            Self::HitSwitch(h) => w.i16(h.x).i16(h.y),
            Self::PlayerChestIndex(c) => w.u8(c.player_id).i16(c.chest),
            Self::ChestSize(c) => w.i16(c.chest).i16(c.size),
            Self::ChestName(c) => w.i16(c.chest).i16(c.x).i16(c.y).str(&c.name),
            Self::TogglePvp(t) => w.u8(t.player_id).bool(t.hostile),
            Self::PlayerMana(m) => w.u8(m.player_id).i16(m.mana).i16(m.max_mana),
            Self::TeamChange(t) => w.u8(t.player_id).u8(t.team),
            Self::NpcHouse(h) => w.i16(h.npc).i16(h.home_x).i16(h.home_y).u8(h.status),
            Self::TravelShop(s) => {
                let mut w = w;
                for item in &s.items {
                    w = w.i16(*item);
                }
                w
            }
            Self::AnglerQuest(q) => w.u8(q.quest).bool(q.finished),
            Self::RevengeMarker(m) => w
                .i32(m.unique_id)
                .f32(m.x)
                .f32(m.y)
                .i32(m.npc_net_id)
                .f32(m.npc_hp_percent)
                .i32(m.npc_type)
                .i32(m.npc_ai_style)
                .i32(m.coins_value)
                .f32(m.base_value)
                .bool(m.spawned_from_statue),
            Self::DeadPlayer { player_id } => w.u8(*player_id),
            Self::MiscEventValue(v) => w.u8(v.kind).i32(v.value),
            Self::ProjectileTrackers(t) => w
                .u8(t.player_id)
                .i32(t.piggy_bank.key)
                .i16(t.piggy_bank.kind)
                .i32(t.void_lens.key)
                .i16(t.void_lens.kind),
            Self::SyncLoadout(l) => w.u8(l.player_id).u8(l.loadout).u16(l.hide_accessory),
            // `NetMessage.cs:1010-1013`: (byte)subject + (byte)kind — the
            // host's kind-2 echo (`MessageBuffer.cs:2498`).
            Self::MiscDataSync(m) => w.u8(m.subject).u8(m.kind),
            // `NetMessage.cs:1088-1103`: BitsByte + (short)entity + the
            // destination floats + (byte)style + the gated (int)extraInfo.
            Self::TeleportEntity(t) => {
                let w = w
                    .bitsbyte(t.flags())
                    .i16(t.entity)
                    .f32(t.x)
                    .f32(t.y)
                    .u8(t.style);
                match t.extra_info {
                    Some(v) => w.i32(v),
                    None => w,
                }
            }
            // `NetMessage.cs:1154-1159`: (short)animationType +
            // (ushort)tileType + (short)x + (short)y.
            Self::TemporaryAnimation(a) => w
                .i16(a.animation_type)
                .u16(a.tile_type)
                .i16(a.x)
                .i16(a.y),
            // `NetMessage.cs:1160-1165`: (int)progress + (int)max +
            // (sbyte)icon + (sbyte)wave.
            Self::InvasionProgress(p) => w
                .i32(p.progress)
                .i32(p.progress_max)
                .i8(p.icon)
                .i8(p.wave),
            // `NetMessage.cs:1337-1345`: (byte)player + (short)color + the
            // destination floats + WriteVector2(the player's live velocity).
            Self::PlayerPortalTeleport(p) => w
                .u8(p.player)
                .i16(p.color_index)
                .f32(p.x)
                .f32(p.y)
                .f32(p.velocity_x)
                .f32(p.velocity_y),
            // `NetMessage.cs:1383-1386`: the two NPC statics as raw ints,
            // MaxMoonLordCountdown first.
            Self::MoonlordCountdown(m) => w.i32(m.max).i32(m.countdown),
            // `NetMessage.cs:1424-1431`: (byte)effect + (int)x + (int)y +
            // (byte)number4 + (short)number5 + (byte)number6.
            Self::SpecialFx(f) => w
                .u8(f.effect)
                .i32(f.x)
                .i32(f.y)
                .u8(f.b4)
                .i16(f.s5)
                .u8(f.b6),
            // No writer arm — bodyless (`DD2Event.cs:636` SendData(114)).
            Self::CrystalInvasionWipe => w,
            // `NetMessage.cs:1436-1438`: the raw (int) countdown.
            Self::CrystalInvasionWait(t) => w.i32(t.time_left),
            // `NetMessage.cs:1518-1520`: the raw (int) marker UniqueID.
            Self::RemoveRevengeMarker(m) => w.i32(m.unique_id),
            // `NetMessage.cs:1595-1602`: (byte)source + (byte)variation +
            // the velocity floats + (int)x + (int)y.
            Self::LucyPopup(l) => w
                .u8(l.source)
                .u8(l.variation)
                .f32(l.velocity_x)
                .f32(l.velocity_y)
                .i32(l.x)
                .i32(l.y),
            // `NetMessage.cs:531-536`: (byte)dayTime + (int)time + sunModY +
            // moonModY (both `short`, `Main.cs:1185`/`:1187`).
            Self::SetTime(t) => w
                .u8(u8::from(t.day_time))
                .i32(t.time)
                .i16(t.sun_mod_y)
                .i16(t.moon_mod_y),
            // `NetMessage.cs:933-944`: (byte)number + zone1..5 + (byte)townNPCs.
            Self::PlayerZone(z) => w
                .u8(z.player_id)
                .u8(z.zone1)
                .u8(z.zone2)
                .u8(z.zone3)
                .u8(z.zone4)
                .u8(z.zone5)
                .u8(z.town_npcs),
            // `NetMessage.cs:956-960`: (byte)number + itemRotation +
            // (short)itemAnimation.
            Self::ItemAnim(a) => w.u8(a.player_id).f32(a.rotation).i16(a.animation),
            // `NetMessage.cs:966-969`: (byte)number + (short)number2.
            Self::ManaEffect(m) => w.u8(m.player_id).i16(m.amount),
            // `NetMessage.cs:1010-1013`: (byte)number + (byte)number2.
            Self::MiscDataSync(d) => w.u8(d.player_id).u8(d.kind),
            // `NetMessage.cs:1041-1045`: (byte)number + (ushort)number2 +
            // (int)number3.
            Self::PlayerBuffPvp(b) => w.u8(b.player_id).u16(b.buff_type).i32(b.time),
            // `NetMessage.cs:1077-1080`: (byte)number + (byte)number2.
            Self::SyncDodge(d) => w.u8(d.player_id).u8(d.kind),
            // `NetMessage.cs:1197-1204`: (byte)id + the live stealth float.
            Self::PlayerStealth(s) => w.u8(s.player_id).f32(s.stealth),
            // Returned early above; unreachable through the builder chain.
            Self::TileBlock(_)
            | Self::SectionRange(_)
            | Self::NetModule { .. }
            | Self::Projectile(_)
            | Self::ProjectileDead { .. } => unreachable!(),
        };
        w.finish()
    }

    pub fn decode(msg_id: u8, r: &mut NetReader) -> Option<Self> {
        Some(match msg_id {
            id::DISCONNECT => Self::Disconnect(NetText::read_from(r)?.0),
            id::PASSWORD_REQUEST => Self::PasswordRequest,
            id::PLAYER_ID => {
                let player_id = r.u8()?;
                let _flag = r.bool()?; // `ServerSpecialFlags[2]`, MessageBuffer.cs:239
                Self::PlayerId { player_id }
            }
            id::PLAYER_INFO => Self::PlayerInfo(PlayerInfo::read_from(r)?),
            id::PLAYER_INVENTORY => {
                let player_id = r.u8()?;
                let slot = r.i16()?;
                let stack = r.i16()?;
                let prefix = r.u8()?;
                let kind = r.i16()?;
                let flags = r.bitsbyte()?;
                Self::Inventory {
                    player_id,
                    slot,
                    item: NetItem {
                        kind,
                        stack: stack.max(0),
                        prefix,
                        favorited: flags.get(0),
                    },
                }
            }
            id::WORLD_INFO => Self::WorldInfo(Box::new(WorldInfo::read_from(r)?)),
            id::STATUS => {
                let max = r.i32()?;
                let text = NetText::read_from(r)?.0;
                let _flags = r.u8()?; // the server's special flags, MessageBuffer.cs:884
                Self::Status { text, max }
            }
            id::TILE_BLOCK => Self::TileBlock(Box::new(TileBlock::decode_from(r)?)),
            id::SECTION_RANGE => Self::SectionRange(Box::new(SectionRange::decode_from(r)?)),
            id::NET_MODULE => Self::NetModule {
                module: r.u16()?,
                body: r.rest()?.to_vec(),
            },
            id::PROJECTILE => Self::Projectile(Box::new(ProjectileState::read_from(r)?)),
            id::PROJECTILE_DEAD => Self::ProjectileDead {
                key: r.i32()?,
                x: r.f32()?,
                y: r.f32()?,
            },
            id::NPC_STRIKE => Self::NpcStrike(NpcStrike::read_from(r)?),
            id::INITIAL_SPAWN => Self::InitialSpawn,
            id::PING => Self::Ping,
            id::NPC_KILLED => Self::NpcKilled { npc: r.i16()? },
            id::PROGRESSION_EVENT => Self::ProgressionEvent {
                event_id: r.i16()?,
            },
            id::PLAYER_ACTIVE => Self::PlayerActive {
                player_id: r.u8()?,
                active: r.u8()? != 0,
            },
            id::PLAYER_HP => Self::PlayerHp {
                player_id: r.u8()?,
                hp: r.i16()?,
                max_hp: r.i16()?,
            },
            id::INT_JOINED => Self::Joined {
                id: r.u32()?,
                name: r.str()?,
            },
            id::INT_LEFT => Self::Left { id: r.u32()? },
            id::PLAYER_STATE => Self::PlayerState(Box::new(PlayerState::read_from(r)?)),
            id::NPC_INFO => Self::NpcInfo(Box::new(NpcInfo::read_from(r)?)),
            id::ITEM_DROP => Self::ItemState(Box::new(ItemState::read_from(r)?)),
            id::ITEM_DEAD => Self::ItemDead { slot: r.i16()? },
            id::ITEM_OWNER => Self::ItemOwner(Box::new(ItemOwner::read_from(r)?)),
            id::INT_TIME => Self::Time {
                time: r.f64()?,
                day: r.bool()?,
            },
            id::INT_EDIT => Self::Edit {
                x: r.i32()?,
                y: r.i32()?,
                kind: r.u8()?,
                tile: r.u16()?,
            },
            id::CHEST_ITEM => Self::ChestItem(ChestItem {
                chest: r.i16()?,
                slot: r.u8()?,
                stack: r.i16()?,
                prefix: r.u8()?,
                kind: r.i16()?,
            }),
            id::PLAYER_CHEST => Self::PlayerChest(read_player_chest(r)?),
            id::SIGN_DATA => Self::SignData(read_sign_data(r)?),
            id::PLAYER_BUFFS => Self::PlayerBuffs(PlayerBuffs {
                player_id: r.u8()?,
                buffs: read_terminated_u16(r),
            }),
            id::NPC_BUFFS => {
                let npc = r.i16()?;
                let mut buffs = Vec::new();
                loop {
                    let id = r.u16()?;
                    if id == 0 {
                        break;
                    }
                    buffs.push((id, r.u16()?));
                }
                Self::NpcBuffs(NpcBuffs { npc, buffs })
            }
            id::HIT_SWITCH => Self::HitSwitch(HitSwitch {
                x: r.i16()?,
                y: r.i16()?,
            }),
            id::PLAYER_CHEST_INDEX => Self::PlayerChestIndex(PlayerChestIndex {
                player_id: r.u8()?,
                chest: r.i16()?,
            }),
            id::CHEST_SIZE => Self::ChestSize(ChestSize {
                chest: r.i16()?,
                size: r.i16()?,
            }),
            id::CHEST_NAME => Self::ChestName(ChestName {
                chest: r.i16()?,
                x: r.i16()?,
                y: r.i16()?,
                name: r.str()?,
            }),
            id::TOGGLE_PVP => Self::TogglePvp(TogglePvp {
                player_id: r.u8()?,
                hostile: r.bool()?,
            }),
            id::PLAYER_MANA => Self::PlayerMana(PlayerMana {
                player_id: r.u8()?,
                mana: r.i16()?,
                max_mana: r.i16()?,
            }),
            id::TEAM_CHANGE => Self::TeamChange(TeamChange {
                player_id: r.u8()?,
                team: r.u8()?,
            }),
            id::NPC_HOUSE => Self::NpcHouse(NpcHouse {
                npc: r.i16()?,
                home_x: r.i16()?,
                home_y: r.i16()?,
                status: r.u8()?,
            }),
            id::TRAVEL_SHOP => {
                // `MessageBuffer.cs:3121-3124`: exactly TravelShopMaxSlots shorts.
                let mut items = Vec::with_capacity(TRAVEL_SHOP_MAX_SLOTS);
                for _ in 0..TRAVEL_SHOP_MAX_SLOTS {
                    items.push(r.i16()?);
                }
                Self::TravelShop(TravelShop { items })
            }
            id::ANGLER_QUEST => Self::AnglerQuest(AnglerQuest {
                quest: r.u8()?,
                finished: r.bool()?,
            }),
            id::REVENGE_MARKER => Self::RevengeMarker(RevengeMarker {
                unique_id: r.i32()?,
                x: r.f32()?,
                y: r.f32()?,
                npc_net_id: r.i32()?,
                npc_hp_percent: r.f32()?,
                npc_type: r.i32()?,
                npc_ai_style: r.i32()?,
                coins_value: r.i32()?,
                base_value: r.f32()?,
                spawned_from_statue: r.bool()?,
            }),
            id::DEAD_PLAYER => Self::DeadPlayer {
                player_id: r.u8()?,
            },
            id::MISC_EVENT_VALUES => Self::MiscEventValue(MiscEventValue {
                kind: r.u8()?,
                value: r.i32()?,
            }),
            id::PROJECTILE_TRACKERS => Self::ProjectileTrackers(ProjectileTrackers {
                player_id: r.u8()?,
                piggy_bank: TrackedProjectileRef {
                    key: r.i32()?,
                    kind: r.i16()?,
                },
                void_lens: TrackedProjectileRef {
                    key: r.i32()?,
                    kind: r.i16()?,
                },
            }),
            id::SYNC_LOADOUT => Self::SyncLoadout(SyncLoadout {
                player_id: r.u8()?,
                loadout: r.u8()?,
                hide_accessory: r.u16()?,
            }),
            id::MISC_DATA_SYNC => Self::MiscDataSync(MiscDataSync {
                subject: r.u8()?,
                kind: r.u8()?,
            }),
            id::TELEPORT_ENTITY => Self::TeleportEntity(read_teleport_entity(r)?),
            id::TEMPORARY_ANIMATION => Self::TemporaryAnimation(TemporaryAnimation {
                animation_type: r.i16()?,
                tile_type: r.u16()?,
                x: r.i16()?,
                y: r.i16()?,
            }),
            id::INVASION_PROGRESS => Self::InvasionProgress(InvasionProgress {
                progress: r.i32()?,
                progress_max: r.i32()?,
                icon: r.i8()?,
                wave: r.i8()?,
            }),
            id::TELEPORT_PLAYER_PORTAL => Self::PlayerPortalTeleport(PlayerPortalTeleport {
                player: r.u8()?,
                color_index: r.i16()?,
                x: r.f32()?,
                y: r.f32()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
            }),
            id::MOONLORD_COUNTDOWN => Self::MoonlordCountdown(MoonlordCountdown {
                max: r.i32()?,
                countdown: r.i32()?,
            }),
            id::SPECIAL_FX => Self::SpecialFx(SpecialFx {
                effect: r.u8()?,
                x: r.i32()?,
                y: r.i32()?,
                b4: r.u8()?,
                s5: r.i16()?,
                b6: r.u8()?,
            }),
            id::CRYSTAL_INVASION_WIPE => Self::CrystalInvasionWipe,
            id::CRYSTAL_INVASION_WAIT => Self::CrystalInvasionWait(CrystalInvasionWait {
                time_left: r.i32()?,
            }),
            id::REMOVE_REVENGE_MARKER => Self::RemoveRevengeMarker(RemoveRevengeMarker {
                unique_id: r.i32()?,
            }),
            id::LUCY_POPUP => Self::LucyPopup(LucyPopup {
                source: r.u8()?,
                variation: r.u8()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
                x: r.i32()?,
                y: r.i32()?,
            // `MessageBuffer.cs:1290-1298`: byte == 1 day flag + int + 2 shorts.
            id::SET_TIME => Self::SetTime(SetTime {
                day_time: r.u8()? == 1,
                time: r.i32()?,
                sun_mod_y: r.i16()?,
                moon_mod_y: r.i16()?,
            }),
            // `MessageBuffer.cs:2180-2192`: (byte)id + zone1..5 + townNPCs.
            id::SYNC_PLAYER_ZONE => Self::PlayerZone(PlayerZone {
                player_id: r.u8()?,
                zone1: r.u8()?,
                zone2: r.u8()?,
                zone3: r.u8()?,
                zone4: r.u8()?,
                zone5: r.u8()?,
                town_npcs: r.u8()?,
            }),
            // `MessageBuffer.cs:2273-2282`: (byte)id + float + (short).
            id::ITEM_ANIM => Self::ItemAnim(ItemAnim {
                player_id: r.u8()?,
                rotation: r.f32()?,
                animation: r.i16()?,
            }),
            // `MessageBuffer.cs:2309-2314`: (byte)id + (short)amount.
            id::MANA_EFFECT => Self::ManaEffect(ManaEffect {
                player_id: r.u8()?,
                amount: r.i16()?,
            }),
            // `MessageBuffer.cs:2484-2489`: two bytes.
            id::MISC_DATA_SYNC => Self::MiscDataSync(MiscDataSync {
                player_id: r.u8()?,
                kind: r.u8()?,
            }),
            // `MessageBuffer.cs:2596-2598`: byte + ushort + int.
            id::ADD_PLAYER_BUFF_PVP => Self::PlayerBuffPvp(PlayerBuffPvp {
                player_id: r.u8()?,
                buff_type: r.u16()?,
                time: r.i32()?,
            }),
            // `MessageBuffer.cs:2881-2882`: two bytes.
            id::SYNC_DODGE => Self::SyncDodge(SyncDodge {
                player_id: r.u8()?,
                kind: r.u8()?,
            }),
            // `MessageBuffer.cs:3248-3253`: byte + float.
            id::PLAYER_STEALTH => Self::PlayerStealth(PlayerStealth {
                player_id: r.u8()?,
                stealth: r.f32()?,
            }),
            _ => return None,
        })
    }
}

// ===========================================================================
// msg 10 TileBlock / msg 20 SectionRange — the tile codec
// ===========================================================================

/// The tile kinds the C# save-compression refuses to batch
/// (`TileID.Sets.AllowsSaveCompressionBatching`, `ID/TileID.cs:484`).
const NO_BATCH: [u16; 4] = [520, 423, 723, 724];

/// One tile's synced state, flattened out of the port's `Tile` (12 packed
/// bytes), the sparse `extras` byte and the wire side-table. The field set is
/// the C# compressor's written fields (`NetMessage.cs:1952-2168`) mapped
/// onto the port's storage; `frame_important` rides along so the codec can
/// apply the `frameX/Y` gate, and `wall_frame` (the C#'s `bTileHeader2`) is
/// the one added byte — the C# relies on `CopyFrom` inside a run, the port
/// needs it on the wire. Both are documented in `docs/ANALYSIS.md`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileRecord {
    pub active: bool,
    pub kind: u16,
    /// Only meaningful when `frame_important` (the C#'s
    /// `Main.tileFrameImportant[type]` gate).
    pub frame_important: bool,
    pub frame_x: i16,
    pub frame_y: i16,
    pub wall: u16,
    pub wall_frame: u8,
    pub liquid: u8,
    /// 0 water, 1 lava, 2 honey, 3 shimmer (`Tile.cs:65-68`).
    pub liquid_kind: u8,
    pub half_brick: bool,
    pub slope: u8,
    pub actuator: bool,
    pub inactive: bool,
    /// Block paint 0..31 (the C#'s `color()` / the port's `meta` high bits).
    pub paint: u8,
    /// Wall paint 0..31 (the port's `extras` low bits).
    pub wall_paint: u8,
    /// Bit 0 red, 1 blue, 2 green, 3 yellow (`World::wires`).
    pub wires: u8,
    pub invisible_block: bool,
    pub invisible_wall: bool,
    pub fullbright_block: bool,
    pub fullbright_wall: bool,
}

impl TileRecord {
    /// `b` (the always-present flag byte, `NetMessage.cs:1944`).
    fn flags_b(&self) -> u8 {
        let mut b = 0u8;
        if self.active {
            b |= 0x02;
        }
        if self.wall != 0 {
            b |= 0x04;
        }
        if self.liquid != 0 {
            // water 0x08, lava 0x10, honey 0x18, shimmer 0x08 + b3 bit7.
            b |= 0x08 << (self.liquid_kind & 1);
            if self.liquid_kind >= 2 {
                b |= 0x10;
            }
        }
        if self.kind > 255 {
            b |= 0x20;
        }
        b
    }

    /// `b2` — wires and the shape code.
    fn flags_b2(&self) -> u8 {
        let mut b = 0u8;
        if self.wires & 1 != 0 {
            b |= 0x02;
        }
        if self.wires & 2 != 0 {
            b |= 0x04;
        }
        if self.wires & 4 != 0 {
            b |= 0x08;
        }
        if self.half_brick {
            b |= 0x10;
        } else if self.slope != 0 {
            b |= (self.slope + 1) << 4;
        }
        b
    }

    /// `b3` — actuator, colors, wire4, the wall high byte and shimmer.
    fn flags_b3(&self) -> u8 {
        let mut b = 0u8;
        if self.actuator {
            b |= 0x02;
        }
        if self.inactive {
            b |= 0x04;
        }
        if self.paint != 0 {
            b |= 0x08;
        }
        if self.wall_paint != 0 {
            b |= 0x10;
        }
        if self.wires & 8 != 0 {
            b |= 0x20;
        }
        if self.wall > 255 {
            b |= 0x40;
        }
        if self.liquid != 0 && self.liquid_kind == 3 {
            b |= 0x80;
        }
        b
    }

    /// `b4` — the invisible/fullbright quad. Bit 0 is unused by the C# and
    /// carries the port's `frame_important` flag (the C# reads its global
    /// `Main.tileFrameImportant[]` table; the port's table is per-world and
    /// the reader thread has no world — deviation in `docs/ANALYSIS.md`).
    fn flags_b4(&self) -> u8 {
        let mut b = 0u8;
        if self.frame_important {
            b |= 0x01;
        }
        if self.invisible_block {
            b |= 0x02;
        }
        if self.invisible_wall {
            b |= 0x04;
        }
        if self.fullbright_block {
            b |= 0x08;
        }
        if self.fullbright_wall {
            b |= 0x10;
        }
        b
    }

    /// Writes one record: the nested flag bytes, the payload in the C#'s
    /// order and the trailing repeat count (`NetMessage.cs:1945`,
    /// `:2206-2207`).
    fn write_into(&self, out: &mut Vec<u8>, repeat: u32) {
        // The nested presence bits are set bottom-up, exactly as the C# does
        // (`NetMessage.cs:2169-2187`): b4 != 0 sets b3's bit 0, b3 != 0 sets
        // b2's bit 0, b2 != 0 sets b's bit 0 — otherwise a lone b3 (a wall
        // high byte, say) would never reach the wire.
        let mut b = self.flags_b();
        let mut b2 = self.flags_b2();
        let mut b3 = self.flags_b3();
        let b4 = self.flags_b4();
        if b4 != 0 {
            b3 |= 0x01;
        }
        if b3 != 0 {
            b2 |= 0x01;
        }
        if b2 != 0 {
            b |= 0x01;
        }
        if repeat > 255 {
            b |= 0x80;
        } else if repeat > 0 {
            b |= 0x40;
        }
        out.push(b);
        if b & 0x01 != 0 {
            out.push(b2);
        }
        if b2 & 0x01 != 0 {
            out.push(b3);
        }
        if b3 & 0x01 != 0 {
            out.push(b4);
        }
        if self.active {
            out.push((self.kind & 0xFF) as u8);
            if self.kind > 255 {
                out.push((self.kind >> 8) as u8);
            }
            if self.frame_important {
                out.extend_from_slice(&self.frame_x.to_le_bytes());
                out.extend_from_slice(&self.frame_y.to_le_bytes());
            }
        }
        if self.paint != 0 {
            out.push(self.paint);
        }
        if self.wall != 0 {
            out.push((self.wall & 0xFF) as u8);
            // The port extension: the C#'s `CopyFrom` covers it inside a run.
            out.push(self.wall_frame);
        }
        if self.wall_paint != 0 {
            out.push(self.wall_paint);
        }
        if self.liquid != 0 {
            out.push(self.liquid);
        }
        if self.wall > 255 {
            out.push((self.wall >> 8) as u8);
        }
        if repeat > 0 {
            if repeat <= 255 {
                out.push(repeat as u8);
            } else {
                out.extend_from_slice(&(repeat as u16).to_le_bytes());
            }
        }
    }
}

/// One section's tile snapshot — the C#'s `CompressTileBlock` body
/// (`NetMessage.cs:1895-1904`: a raw-deflate stream with the rectangle
/// header, then the records).
#[derive(Clone, Debug, PartialEq)]
pub struct TileBlock {
    pub x: i32,
    pub y: i32,
    pub w: i16,
    pub h: i16,
    /// Exactly `w * h` records, row-major y-outer/x-inner.
    pub records: Vec<TileRecord>,
}

impl TileBlock {
    /// Builds the wire frame `[u16 len][u8 10][deflate blob]`.
    pub fn frame(&self) -> Vec<u8> {
        let mut inner: Vec<u8> = Vec::new();
        inner.extend_from_slice(&self.x.to_le_bytes());
        inner.extend_from_slice(&self.y.to_le_bytes());
        inner.extend_from_slice(&self.w.to_le_bytes());
        inner.extend_from_slice(&self.h.to_le_bytes());
        // The run-length batching: a run continues while the next record is
        // identical and its kind is batchable (`NetMessage.cs:1921-1947`).
        let mut i = 0usize;
        while i < self.records.len() {
            let rec = self.records[i];
            let mut run = 1usize;
            if !NO_BATCH.contains(&rec.kind) {
                while i + run < self.records.len() && self.records[i + run] == rec {
                    run += 1;
                }
            }
            rec.write_into(&mut inner, (run - 1) as u32);
            i += run;
        }
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::fast());
        let _ = enc.write_all(&inner);
        let blob = enc.finish().unwrap_or_default();
        let total = 2 + 1 + blob.len();
        let mut frame = Vec::with_capacity(total);
        frame.extend_from_slice(&(total as u16).to_le_bytes());
        frame.push(id::TILE_BLOCK);
        frame.extend_from_slice(&blob);
        debug_assert!(total <= MAX_PACKET, "net: tile block over the frame cap");
        frame
    }

    /// Parses a `10` body (the reader threads strip the length+id already).
    pub fn decode_from(r: &mut NetReader) -> Option<Self> {
        let blob = r.rest()?;
        let mut dec = DeflateDecoder::new(blob);
        let mut inner = Vec::new();
        dec.read_to_end(&mut inner).ok()?;
        let mut b = NetReader::new(&inner);
        let x = b.i32()?;
        let y = b.i32()?;
        let w = b.i16()?;
        let h = b.i16()?;
        let want = (w as i32 * h as i32).max(0) as usize;
        let mut records: Vec<TileRecord> = Vec::with_capacity(want);
        while records.len() < want {
            let rec = read_record(&mut b)?;
            let repeat = match (rec_flags_inline::last_b() & 0xC0) >> 6 {
                0 => 0usize,
                1 => b.u8()? as usize,
                _ => b.u16()? as usize,
            };
            for _ in 0..=repeat {
                records.push(rec);
                if records.len() == want {
                    break;
                }
            }
        }
        Some(Self {
            x,
            y,
            w,
            h,
            records,
        })
    }
}

mod rec_flags_inline {
    /// The last record's `b` byte, stashed by `read_record` for the repeat
    /// count (the C# reads the count from the same flag byte).
    pub static mut LAST_B: u8 = 0;
    pub fn last_b() -> u8 {
        unsafe { LAST_B }
    }
}

/// Reads one record back (`NetMessage.cs:2259-2418`'s field order).
fn read_record(b: &mut NetReader) -> Option<TileRecord> {
    let b0 = b.u8()?;
    let b2 = if b0 & 0x01 != 0 { b.u8()? } else { 0 };
    let b3 = if b2 & 0x01 != 0 { b.u8()? } else { 0 };
    let b4 = if b3 & 0x01 != 0 { b.u8()? } else { 0 };
    // The frame pair's presence must be known before the payload is read.
    let frame_important = b4 & 0x01 != 0;
    unsafe { rec_flags_inline::LAST_B = b0 };
    let mut rec = TileRecord {
        active: b0 & 0x02 != 0,
        kind: 0,
        frame_important,
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
    if rec.active {
        rec.kind = b.u8()? as u16;
        if b0 & 0x20 != 0 {
            rec.kind |= (b.u8()? as u16) << 8;
        }
        if rec.frame_important {
            rec.frame_x = b.i16()?;
            rec.frame_y = b.i16()?;
        }
    }
    if b3 & 0x08 != 0 {
        rec.paint = b.u8()?;
    }
    if b0 & 0x04 != 0 {
        rec.wall = b.u8()? as u16;
        rec.wall_frame = b.u8()?;
    }
    if b3 & 0x10 != 0 {
        rec.wall_paint = b.u8()?;
    }
    if b0 & 0x18 != 0 {
        rec.liquid = b.u8()?;
        rec.liquid_kind = ((b0 & 0x18) >> 3) - 1 + if b3 & 0x80 != 0 { 3 } else { 0 };
    }
    if b3 & 0x40 != 0 {
        rec.wall |= (b.u8()? as u16) << 8;
    }
    if b2 & 0x02 != 0 {
        rec.wires |= 1;
    }
    if b2 & 0x04 != 0 {
        rec.wires |= 2;
    }
    if b2 & 0x08 != 0 {
        rec.wires |= 4;
    }
    if b2 & 0x20 != 0 {
        rec.wires |= 8;
    }
    if b2 & 0x10 != 0 {
        rec.half_brick = true;
    } else if b2 & 0x40 != 0 || b2 & 0x20 != 0 {
        // The shape code sits in bits 4-6: 1 half-brick, 2..5 slopes.
        let code = (b2 & 0x70) >> 4;
        if code == 1 {
            rec.half_brick = true;
        } else {
            rec.slope = code - 1;
        }
    }
    if b3 & 0x02 != 0 {
        rec.actuator = true;
    }
    if b3 & 0x04 != 0 {
        rec.inactive = true;
    }
    if b4 & 0x02 != 0 {
        rec.invisible_block = true;
    }
    if b4 & 0x04 != 0 {
        rec.invisible_wall = true;
    }
    if b4 & 0x08 != 0 {
        rec.fullbright_block = true;
    }
    if b4 & 0x10 != 0 {
        rec.fullbright_wall = true;
    }
    Some(rec)
}

/// A rectangle of tile changes — `SendTileSquare`'s body
/// (`NetMessage.cs:543-644`): uncompressed, three flag bytes per tile.
#[derive(Clone, Debug, PartialEq)]
pub struct SectionRange {
    pub x: i16,
    pub y: i16,
    pub w: u8,
    pub h: u8,
    /// `TileChangeType` — the port has no switch family yet, so it rides 0.
    pub change_type: u8,
    /// Exactly `w * h` records, row-major.
    pub records: Vec<TileRecord>,
}

impl SectionRange {
    /// Builds the wire frame `[u16 len][u8 20][header][per-tile bodies]`.
    pub fn frame(&self) -> Vec<u8> {
        let mut w = NetWriter::open(id::SECTION_RANGE)
            .i16(self.x)
            .i16(self.y)
            .u8(self.w)
            .u8(self.h)
            .u8(self.change_type);
        for r in &self.records {
            let mut f1 = 0u8;
            if r.active {
                f1 |= 0x01;
            }
            if r.wall != 0 {
                f1 |= 0x04;
            }
            if r.liquid != 0 {
                f1 |= 0x08;
            }
            if r.wires & 1 != 0 {
                f1 |= 0x10;
            }
            if r.half_brick {
                f1 |= 0x20;
            }
            if r.actuator {
                f1 |= 0x40;
            }
            if r.inactive {
                f1 |= 0x80;
            }
            let mut f2 = 0u8;
            if r.wires & 2 != 0 {
                f2 |= 0x01;
            }
            if r.wires & 4 != 0 {
                f2 |= 0x02;
            }
            if r.paint != 0 {
                f2 |= 0x04;
            }
            if r.wall_paint != 0 {
                f2 |= 0x08;
            }
            f2 |= (r.slope & 0x07) << 4;
            if r.wires & 8 != 0 {
                f2 |= 0x80;
            }
            let mut f3 = 0u8;
            if r.frame_important {
                // Port extension: the C# reads its global frame table here.
                f3 |= 0x10;
            }
            if r.fullbright_block {
                f3 |= 0x01;
            }
            if r.fullbright_wall {
                f3 |= 0x02;
            }
            if r.invisible_block {
                f3 |= 0x04;
            }
            if r.invisible_wall {
                f3 |= 0x08;
            }
            w = w
                .bitsbyte(BitsByte(f1))
                .bitsbyte(BitsByte(f2))
                .bitsbyte(BitsByte(f3));
            if f2 & 0x04 != 0 {
                w = w.u8(r.paint);
            }
            if f2 & 0x08 != 0 {
                w = w.u8(r.wall_paint);
            }
            if r.active {
                w = w.u16(r.kind);
                if r.frame_important {
                    w = w.i16(r.frame_x).i16(r.frame_y);
                }
            }
            if r.wall != 0 {
                w = w.u16(r.wall);
            }
            if r.liquid != 0 {
                w = w.u8(r.liquid).u8(r.liquid_kind);
            }
        }
        w.finish()
    }

    /// Parses a `20` body.
    pub fn decode_from(r: &mut NetReader) -> Option<Self> {
        let x = r.i16()?;
        let y = r.i16()?;
        let w = r.u8()?;
        let h = r.u8()?;
        let change_type = r.u8()?;
        let want = (w as usize) * (h as usize);
        let mut records = Vec::with_capacity(want);
        for _ in 0..want {
            let f1 = r.bitsbyte()?.0;
            let f2 = r.bitsbyte()?.0;
            let f3 = r.bitsbyte()?.0;
            let mut rec = TileRecord {
                active: f1 & 0x01 != 0,
                kind: 0,
                frame_important: f3 & 0x10 != 0,
                frame_x: 0,
                frame_y: 0,
                wall: 0,
                wall_frame: 0,
                liquid: 0,
                liquid_kind: 0,
                half_brick: f1 & 0x20 != 0,
                slope: (f2 & 0x70) >> 4,
                actuator: f1 & 0x40 != 0,
                inactive: f1 & 0x80 != 0,
                paint: 0,
                wall_paint: 0,
                wires: 0,
                invisible_block: f3 & 0x04 != 0,
                invisible_wall: f3 & 0x08 != 0,
                fullbright_block: f3 & 0x01 != 0,
                fullbright_wall: f3 & 0x02 != 0,
            };
            if f2 & 0x04 != 0 {
                rec.paint = r.u8()?;
            }
            if f2 & 0x08 != 0 {
                rec.wall_paint = r.u8()?;
            }
            if rec.active {
                rec.kind = r.u16()?;
                if rec.frame_important {
                    rec.frame_x = r.i16()?;
                    rec.frame_y = r.i16()?;
                }
            }
            if f1 & 0x04 != 0 {
                rec.wall = r.u16()?;
            }
            if f1 & 0x08 != 0 {
                rec.liquid = r.u8()?;
                rec.liquid_kind = r.u8()?;
            }
            if f2 & 0x01 != 0 {
                rec.wires |= 1;
            }
            if f2 & 0x02 != 0 {
                rec.wires |= 2;
            }
            if f1 & 0x10 != 0 {
                rec.wires |= 4;
            }
            if f2 & 0x80 != 0 {
                rec.wires |= 8;
            }
            records.push(rec);
        }
        Some(Self {
            x,
            y,
            w,
            h,
            change_type,
            records,
        })
    }
}

// ---------------------------------------------------------------------------
// msg 27 / 29 — the projectile spawn+update and the despawn
// ---------------------------------------------------------------------------

/// `ProjectileKey`'s packing (`DataStructures/ProjectileKey.cs:16-31`):
/// bits 0-7 the spawner (the owner), 8-17 the slot index, 18-31 the
/// generation. The port's pool slots are stable, so the key rides as the
/// same layout.
pub fn projectile_key(owner: u8, slot: usize, generation: u16) -> i32 {
    ((generation as i32) << 18) | ((slot as i32) << 8) | owner as i32
}

/// The inverse split of [`projectile_key`].
pub fn projectile_key_parts(key: i32) -> (u8, usize, u16) {
    let owner = (key & 0xFF) as u8;
    let slot = ((key >> 8) & 0x3FF) as usize;
    let generation = ((key >> 18) & 0x3FFF) as u16;
    (owner, slot, generation)
}

/// `msg 27`'s body (`NetMessage.cs:780-841`): the key, the position/velocity,
/// the type and the flag-gated extras. Absent conditional fields read as zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectileState {
    /// The `ProjectileKey` — see [`projectile_key`].
    pub key: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub kind: i16,
    /// `ai[0]` — non-None when flags1 bit 0.
    pub ai0: Option<f32>,
    /// `ai[1]` — non-None when flags1 bit 1.
    pub ai1: Option<f32>,
    /// `bannerIdToRespondTo` — flags1 bit 3.
    pub banner_id: u16,
    /// flags1 bit 4.
    pub damage: i16,
    /// flags1 bit 5.
    pub knock_back: f32,
    /// flags1 bit 6.
    pub original_damage: i16,
    /// `ai[2]` — flags2 bit 0.
    pub ai2: Option<f32>,
}

impl ProjectileState {
    fn flags1(&self) -> u8 {
        let mut b = 0u8;
        if self.ai0.is_some() {
            b |= 0x01;
        }
        if self.ai1.is_some() {
            b |= 0x02;
        }
        if self.ai2.is_some() {
            b |= 0x04;
        }
        if self.banner_id != 0 {
            b |= 0x08;
        }
        if self.damage != 0 {
            b |= 0x10;
        }
        if self.knock_back != 0.0 {
            b |= 0x20;
        }
        if self.original_damage != 0 {
            b |= 0x40;
        }
        b
    }

    fn write_to(&self, w: NetWriter) -> NetWriter {
        let flags2 = self.ai2.is_some();
        let mut w = w
            .i32(self.key)
            .f32(self.x)
            .f32(self.y)
            .f32(self.vx)
            .f32(self.vy)
            .i16(self.kind)
            .bitsbyte(BitsByte(self.flags1()));
        if flags2 {
            w = w.bitsbyte(BitsByte(0x01));
        }
        if let Some(v) = self.ai0 {
            w = w.f32(v);
        }
        if let Some(v) = self.ai1 {
            w = w.f32(v);
        }
        if self.banner_id != 0 {
            w = w.u16(self.banner_id);
        }
        if self.damage != 0 {
            w = w.i16(self.damage);
        }
        if self.knock_back != 0.0 {
            w = w.f32(self.knock_back);
        }
        if self.original_damage != 0 {
            w = w.i16(self.original_damage);
        }
        if let Some(v) = self.ai2 {
            w = w.f32(v);
        }
        w
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let key = r.i32()?;
        let x = r.f32()?;
        let y = r.f32()?;
        let vx = r.f32()?;
        let vy = r.f32()?;
        let kind = r.i16()?;
        let f1 = r.bitsbyte()?.0;
        let f2 = if f1 & 0x04 != 0 { r.bitsbyte()?.0 } else { 0 };
        Some(Self {
            key,
            x,
            y,
            vx,
            vy,
            kind,
            ai0: if f1 & 0x01 != 0 { Some(r.f32()?) } else { None },
            ai1: if f1 & 0x02 != 0 { Some(r.f32()?) } else { None },
            banner_id: if f1 & 0x08 != 0 { r.u16()? } else { 0 },
            damage: if f1 & 0x10 != 0 { r.i16()? } else { 0 },
            knock_back: if f1 & 0x20 != 0 { r.f32()? } else { 0.0 },
            original_damage: if f1 & 0x40 != 0 { r.i16()? } else { 0 },
            ai2: if f2 & 0x01 != 0 { Some(r.f32()?) } else { None },
        })
    }

    /// Builds the frame `[u16 len][u8 27][body]`.
    pub fn frame(&self) -> Vec<u8> {
        self.write_to(NetWriter::open(id::PROJECTILE)).finish()
    }
}

/// `msg 28`'s body (`NetMessage.cs:844-850`): the NPC slot, the generation,
/// the damage, the knockback, the hit direction (offset by one on the wire)
/// and the crit flag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpcStrike {
    /// The NPC's slot index.
    pub npc: u8,
    /// The generation the sender struck — the host drops a mismatch
    /// (`MessageBuffer.cs:1781-1784`).
    pub generation: u8,
    pub damage: i16,
    pub knock_back: f32,
    /// The wire value is `hitDirection + 1`; `0xFF` wraps to 0 for the -1
    /// sentinel (`NetMessage.cs:847`).
    pub hit_direction: i8,
    pub crit: bool,
}

impl NpcStrike {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        w.u8(self.npc)
            .u8(self.generation)
            .i16(self.damage)
            .f32(self.knock_back)
            .u8(self.hit_direction.wrapping_add(1) as u8)
            .bool(self.crit)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let npc = r.u8()?;
        let generation = r.u8()?;
        let damage = r.i16()?;
        let knock_back = r.f32()?;
        let hit_direction = (r.u8()? as i8).wrapping_sub(1);
        let crit = r.bool()?;
        Some(Self {
            npc,
            generation,
            damage,
            knock_back,
            hit_direction,
            crit,
        })
    }
}

// ===========================================================================
// Transport — the game-thread-facing queues
// ===========================================================================

/// Events the host's game loop consumes: raw messages, to be gated and
/// dispatched by `netglue`'s state tables (the `GetData` logic — the reader
/// threads stay I/O-only so the states are never read/written concurrently).
#[derive(Debug)]
pub enum ServerEvent {
    /// One decoded message from the peer in slot `id`.
    Frame { id: u32, msg: C2s },
    /// The frame could not be decoded; `netglue` kicks the peer.
    BadFrame { id: u32 },
    /// The peer went away (reader EOF or a kick).
    Closed { id: u32 },
}

/// Events a joining client's game loop consumes.
#[derive(Debug)]
pub enum ClientEvent {
    Frame(S2c),
    BadFrame,
    Closed,
}

/// One peer's outbound queue: frames plus a "close after this one" flag so a
/// kick's `Disconnect` is flushed before the socket shuts down (`Netplay.cs`
/// `KickClient`'s pre-written full buffer, `:193-215`).
struct Outbox {
    queue: Mutex<VecDeque<(Vec<u8>, bool)>>,
    cv: Condvar,
}

impl Outbox {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            cv: Condvar::new(),
        }
    }
    fn push(&self, frame: Vec<u8>, close: bool) {
        if let Ok(mut q) = self.queue.lock() {
            q.push_back((frame, close));
        }
        self.cv.notify_one();
    }
    fn pop(&self) -> (Vec<u8>, bool) {
        let mut q = self.queue.lock().expect("net outbox lock");
        loop {
            if let Some(f) = q.pop_front() {
                return f;
            }
            q = self.cv.wait(q).expect("net outbox lock");
        }
    }
}

/// A server-side client slot: the connection's state (the C#
/// `Netplay.Clients[i].State`), the name the peer announced and the idle
/// stamp that drives `TimeOutTimer` (`MessageBuffer.cs:127-134`).
#[derive(Clone, Debug)]
pub struct Slot {
    pub state: i32,
    pub name: String,
    pub last_seen: Instant,
    /// `Main.player[id].inventory` — the peer's slot state, fed by the
    /// `5 PlayerInventory` messages (`MessageBuffer.cs:399-462`). The C#'s
    /// server keeps the same per-client copy of the inventory.
    pub inventory: HashMap<u8, NetItem>,
    /// `RemoteClient.TileSections` (`RemoteClient.cs:31`): the flattened
    /// `sections_w * sections_h` latch grid of the sections already streamed.
    pub sections: Vec<bool>,
    /// The grid width the latch is sized for (0 = not allocated yet).
    pub sections_w: i32,
}

type Slots = Arc<Mutex<HashMap<u32, Slot>>>;

/// Host side: the listener, the per-peer slots and the senders map.
pub struct Server {
    pub events: Receiver<ServerEvent>,
    slots: Slots,
    clients: Arc<Mutex<HashMap<u32, Arc<Outbox>>>>,
    addr: String,
    password: String,
}

impl Server {
    /// Binds `0.0.0.0:port` so LAN and internet clients can reach the host
    /// (`Netplay.cs`'s `TcpListener`). `port = 0` picks an ephemeral port —
    /// read it back with [`Server::addr`] (tests do exactly that).
    /// The password gates the join (`Netplay.ServerPassword`); an empty one
    /// skips `37 PasswordRequest` (`MessageBuffer.cs:208-214`).
    pub fn start(port: u16, password: String) -> std::io::Result<Server> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        let bound = listener.local_addr()?;
        // A wildcard bind reports `0.0.0.0`, which no client can dial
        // (WSAEADDRNOTAVAIL); report the loopback dial-back instead. The UI
        // shows the LAN addresses separately.
        let addr = if bound.ip().is_unspecified() {
            format!("127.0.0.1:{}", bound.port())
        } else {
            bound.to_string()
        };
        let (tx, rx) = mpsc::channel();
        let clients: Arc<Mutex<HashMap<u32, Arc<Outbox>>>> = Arc::new(Mutex::new(HashMap::new()));
        let slots: Slots = Arc::new(Mutex::new(HashMap::new()));
        let clients_accept = clients.clone();
        let slots_accept = slots.clone();
        thread::spawn(move || {
            let mut next_id: u32 = 1;
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let _ = stream.set_nodelay(true);
                // `MaxConnections = 256`: the accept-time server-full kick
                // (`Netplay.cs:189`, `KickClient` `:193-215`) — pre-writing the
                // disconnect before the slot exists.
                let full = {
                    let s = slots_accept.lock().expect("net slots lock");
                    s.len() >= MAX_CONNECTIONS
                };
                if full {
                    let frame = S2c::Disconnect("Server is full".to_string()).encode();
                    let mut w = stream;
                    let _ = write_frame(&mut w, &frame);
                    let _ = w.shutdown(std::net::Shutdown::Both);
                    continue;
                }
                let id = next_id;
                next_id += 1;
                let outbox = Arc::new(Outbox::new());
                clients_accept
                    .lock()
                    .expect("net clients lock")
                    .insert(id, outbox.clone());
                slots_accept.lock().expect("net slots lock").insert(
                    id,
                    Slot {
                        state: slot_state::FRESH,
                        name: String::new(),
                        last_seen: Instant::now(),
                        inventory: HashMap::new(),
                        sections: Vec::new(),
                        sections_w: 0,
                    },
                );
                let Ok(mut reader) = stream.try_clone() else {
                    continue;
                };
                let mut writer = stream;
                let out = outbox.clone();
                thread::spawn(move || loop {
                    let (frame, close) = out.pop();
                    if write_frame(&mut writer, &frame).is_err() {
                        break;
                    }
                    if close {
                        let _ = writer.shutdown(std::net::Shutdown::Both);
                        break;
                    }
                });
                let tx = tx.clone();
                let slots = slots_accept.clone();
                let clients = clients_accept.clone();
                thread::spawn(move || {
                    loop {
                        let frame = match read_frame(&mut reader) {
                            Ok(f) => f,
                            Err(_) => break,
                        };
                        slots
                            .lock()
                            .expect("net slots lock")
                            .entry(id)
                            .and_modify(|s| s.last_seen = Instant::now());
                        let msg_id = frame[0];
                        let ev = match C2s::decode(msg_id, &mut NetReader::new(&frame[1..])) {
                            Some(msg) => ServerEvent::Frame { id, msg },
                            None => ServerEvent::BadFrame { id },
                        };
                        if tx.send(ev).is_err() {
                            break;
                        }
                    }
                    clients.lock().expect("net clients lock").remove(&id);
                    slots.lock().expect("net slots lock").remove(&id);
                    let _ = tx.send(ServerEvent::Closed { id });
                });
            }
        });
        Ok(Server {
            events: rx,
            slots,
            clients,
            addr,
            password,
        })
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn client_count(&self) -> usize {
        self.slots.lock().map(|s| s.len()).unwrap_or(0)
    }

    pub fn state(&self, id: u32) -> i32 {
        self.slots
            .lock()
            .ok()
            .and_then(|s| s.get(&id).map(|c| c.state))
            .unwrap_or(slot_state::FRESH)
    }

    pub fn set_state(&self, id: u32, state: i32) {
        if let Ok(mut s) = self.slots.lock() {
            if let Some(c) = s.get_mut(&id) {
                c.state = state;
            }
        }
    }

    pub fn slot_name(&self, id: u32) -> String {
        self.slots
            .lock()
            .ok()
            .and_then(|s| s.get(&id).map(|c| c.name.clone()))
            .unwrap_or_default()
    }

    pub fn set_slot_name(&self, id: u32, name: String) {
        if let Ok(mut s) = self.slots.lock() {
            if let Some(c) = s.get_mut(&id) {
                c.name = name;
            }
        }
    }

    /// Stores one peer inventory slot — `Main.player[id].inventory[slot]`
    /// (`MessageBuffer.cs:399-462`'s `slot.Item = item`).
    pub fn set_slot_inventory(&self, id: u32, slot: u8, item: NetItem) {
        if let Ok(mut s) = self.slots.lock() {
            if let Some(c) = s.get_mut(&id) {
                c.inventory.insert(slot, item);
            }
        }
    }

    /// The peer's inventory slots (a snapshot).
    pub fn slot_inventory(&self, id: u32) -> HashMap<u8, NetItem> {
        self.slots
            .lock()
            .ok()
            .and_then(|s| s.get(&id).map(|c| c.inventory.clone()))
            .unwrap_or_default()
    }

    /// All connected slot ids (snapshot for sweeps/iterations).
    pub fn slot_ids(&self) -> Vec<u32> {
        self.slots
            .lock()
            .map(|s| s.keys().copied().collect())
            .unwrap_or_default()
    }

    fn outbox(&self, id: u32) -> Option<Arc<Outbox>> {
        self.clients.lock().ok().and_then(|c| c.get(&id).cloned())
    }

    /// Sends one message to a single slot.
    pub fn send(&self, id: u32, msg: &S2c) {
        if let Some(o) = self.outbox(id) {
            o.push(msg.encode(), false);
        }
    }

    /// Broadcasts to every slot except `except` — the C# `SendData`'s
    /// default `ignoreClient` routing (`NetMessage.cs:1692-1826`).
    pub fn broadcast(&self, msg: &S2c) {
        let frame = msg.encode();
        if let Ok(clients) = self.clients.lock() {
            for o in clients.values() {
                o.push(frame.clone(), false);
            }
        }
    }

    pub fn broadcast_except(&self, msg: &S2c, except: u32) {
        let frame = msg.encode();
        if let Ok(clients) = self.clients.lock() {
            for (id, o) in clients.iter() {
                if *id != except {
                    o.push(frame.clone(), false);
                }
            }
        }
    }

    /// Kicks a slot with a reason: the `Disconnect` is queued with the
    /// close-after flag so it is flushed before the socket shuts
    /// (`Netplay.cs:193-215`'s `KickClient`).
    pub fn kick(&self, id: u32, reason: &str) {
        if let Some(o) = self.outbox(id) {
            o.push(S2c::Disconnect(reason.to_string()).encode(), true);
        }
    }

    /// Drops peers idle for longer than [`IDLE_TIMEOUT`]; returns their ids so
    /// the caller can clean up game state (`Main.cs:64610-64618`'s timeout
    /// terminate, driven by the `TimeOutTimer` the readers reset).
    pub fn sweep_timeouts(&self) -> Vec<u32> {
        let mut dead = Vec::new();
        if let Ok(mut s) = self.slots.lock() {
            let now = Instant::now();
            let ids: Vec<u32> = s.keys().copied().collect();
            for id in ids {
                let timed_out = s
                    .get(&id)
                    .map(|c| now.duration_since(c.last_seen) >= IDLE_TIMEOUT)
                    .unwrap_or(false);
                if timed_out {
                    s.remove(&id);
                    dead.push(id);
                }
            }
        }
        for id in &dead {
            self.kick(*id, "Timed out");
        }
        dead
    }

    /// Allocates (or resizes) a slot's section latch grid for a world of the
    /// given dimensions — the C#'s `Netplay.Clients[i].TileSections`
    /// (`RemoteClient.cs:31`), the "which sections this peer already holds"
    /// bookkeeping.
    pub fn ensure_sections(&self, id: u32, world_w: i32, world_h: i32) {
        let want = (sections_x(world_w) * sections_y(world_h)).max(0) as usize;
        let w = sections_x(world_w);
        if let Ok(mut s) = self.slots.lock() {
            if let Some(c) = s.get_mut(&id) {
                if c.sections.len() != want || c.sections_w != w {
                    c.sections = vec![false; want];
                    c.sections_w = w;
                }
            }
        }
    }

    /// Whether the section at `(sx, sy)` was already streamed to this slot.
    pub fn is_section_sent(&self, id: u32, sx: i32, sy: i32) -> bool {
        let w = self
            .slots
            .lock()
            .ok()
            .and_then(|s| s.get(&id).map(|c| c.sections_w))
            .unwrap_or(0);
        if w == 0 || sx < 0 || sy < 0 {
            return false;
        }
        self.slots
            .lock()
            .ok()
            .and_then(|s| s.get(&id).map(|c| c.sections[(sy * w + sx) as usize]))
            .unwrap_or(false)
    }

    /// Locks the section as sent (`SendSection`'s latch, `NetMessage.cs:2707`).
    pub fn mark_section_sent(&self, id: u32, sx: i32, sy: i32) {
        let w = self
            .slots
            .lock()
            .ok()
            .and_then(|s| s.get(&id).map(|c| c.sections_w))
            .unwrap_or(0);
        if w == 0 || sx < 0 || sy < 0 {
            return;
        }
        if let Ok(mut s) = self.slots.lock() {
            if let Some(c) = s.get_mut(&id) {
                let i = (sy * w + sx) as usize;
                if i < c.sections.len() {
                    c.sections[i] = true;
                }
            }
        }
    }
}

/// Client side: the reader/writer threads plus the event queue for the game
/// loop (`Netplay.Connection`'s socket pair, `Netplay.cs:447`).
pub struct Client {
    pub events: Receiver<ClientEvent>,
    out: Arc<Outbox>,
}

impl Client {
    pub fn connect(addr: &str) -> std::io::Result<Client> {
        let stream = TcpStream::connect(addr)?;
        let _ = stream.set_nodelay(true);
        let mut reader = stream.try_clone()?;
        let mut writer = stream;
        let out = Arc::new(Outbox::new());
        let out_writer = out.clone();
        thread::spawn(move || loop {
            let (frame, close) = out_writer.pop();
            if write_frame(&mut writer, &frame).is_err() {
                break;
            }
            if close {
                let _ = writer.shutdown(std::net::Shutdown::Both);
                break;
            }
        });
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let frame = match read_frame(&mut reader) {
                Ok(f) => f,
                Err(_) => {
                    let _ = tx.send(ClientEvent::Closed);
                    break;
                }
            };
            let msg_id = frame[0];
            let ev = match S2c::decode(msg_id, &mut NetReader::new(&frame[1..])) {
                Some(msg) => ClientEvent::Frame(msg),
                None => ClientEvent::BadFrame,
            };
            if tx.send(ev).is_err() {
                break;
            }
        });
        Ok(Client { events: rx, out })
    }

    pub fn send(&self, msg: &C2s) {
        self.out.push(msg.encode(), false);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn writer_reader_round_trip_primitives() {
        let w = NetWriter::open(id::WORLD_INFO)
            .u8(7)
            .i8(-3)
            .u16(65535)
            .i16(-32768)
            .u32(0xDEAD_BEEF)
            .i32(-123456)
            .u64(0x0123_4567_89AB_CDEF)
            .f32(0.5)
            .f64(-1.25e-9)
            .bool(true)
            .bitsbyte(BitsByte(0b1010_0011))
            .rgb(Rgb(1, 2, 3))
            .str("héllo 世界")
            .bytes(&[9, 8, 7])
            .finish();
        // The frame is [len][id][body]: len covers everything, id at 2.
        assert_eq!(w[2], id::WORLD_INFO);
        assert_eq!(u16::from_le_bytes([w[0], w[1]]), w.len() as u16);
        let body = &w[3..];
        let mut r = NetReader::new(body);
        assert_eq!(r.u8(), Some(7));
        assert_eq!(r.i8(), Some(-3));
        assert_eq!(r.u16(), Some(65535));
        assert_eq!(r.i16(), Some(-32768));
        assert_eq!(r.u32(), Some(0xDEAD_BEEF));
        assert_eq!(r.i32(), Some(-123456));
        assert_eq!(r.u64(), Some(0x0123_4567_89AB_CDEF));
        assert_eq!(r.f32(), Some(0.5));
        assert_eq!(r.f64(), Some(-1.25e-9));
        assert_eq!(r.bool(), Some(true));
        assert_eq!(r.bitsbyte(), Some(BitsByte(0b1010_0011)));
        assert_eq!(r.rgb(), Some(Rgb(1, 2, 3)));
        assert_eq!(r.str().as_deref(), Some("héllo 世界"));
        assert_eq!(r.bytes(), Some(vec![9, 8, 7]));
        // The body is fully consumed: one more read past it fails.
        assert!(r.u8().is_none());
    }

    #[test]
    fn long_string_uses_seven_bit_length() {
        let s = "x".repeat(600);
        let w = NetWriter::open(id::CONNECT).str(&s).finish();
        // 600 needs two 7-bit bytes (0x80|0x04, 0x58).
        assert_eq!(&w[3..5], &[0xD8, 0x04]);
        let mut r = NetReader::new(&w[3..]);
        assert_eq!(r.str().as_deref(), Some(s.as_str()));
    }

    #[test]
    fn truncated_body_decodes_to_none() {
        let mut r = NetReader::new(&[]);
        assert!(r.u32().is_none());
        assert!(r.str().is_none());
        // A 7-bit length that never terminates is also malformed.
        let mut r = NetReader::new(&[0x80, 0x80]);
        assert!(r.str().is_none());
    }

    #[test]
    fn player_info_round_trip() {
        let pi = PlayerInfo {
            player_id: 3,
            skin_variant: 1,
            voice_variant: 2,
            voice_pitch_offset: -0.5,
            hair: 12,
            name: "Rusty".into(),
            hair_dye: 7,
            hide_visible_accessory: 0b1010_1010,
            hide_misc: 3,
            hair_color: Rgb(1, 2, 3),
            skin_color: Rgb(4, 5, 6),
            eye_color: Rgb(7, 8, 9),
            shirt_color: Rgb(10, 11, 12),
            under_shirt_color: Rgb(13, 14, 15),
            pants_color: Rgb(16, 17, 18),
            shoe_color: Rgb(19, 20, 21),
            flags1: BitsByte(0b1101),
            flags2: BitsByte(0b0110),
            flags3: BitsByte(0x80),
        };
        let w = NetWriter::open(id::PLAYER_INFO);
        let body = pi.write_to(w).finish();
        let mut r = NetReader::new(&body[3..]);
        let back = PlayerInfo::read_from(&mut r).expect("player info");
        assert_eq!(back, pi);
    }

    #[test]
    fn world_info_round_trip() {
        let wi = WorldInfo {
            time: 13500,
            day_time: true,
            blood_moon: false,
            eclipse: true,
            moon_phase: 4,
            max_tiles_x: 4000,
            max_tiles_y: 1200,
            spawn_tile_x: 2025,
            spawn_tile_y: 300,
            world_surface: 350,
            rock_layer: 500,
            world_id: 7,
            world_name: "Testworld".into(),
            game_mode: 0,
            unique_id: [9; 16],
            generator_version: 0x0102_0304_0506_0708,
            moon_type: 3,
            backgrounds: [1; 11],
            ice_back_style: 1,
            jungle_back_style: 2,
            hell_back_style: 1,
            wind_speed_target: 0.4,
            num_clouds: 120,
            tree_x: [11, 22, 33],
            tree_style: [1, 2, 3, 4],
            cave_back_x: [44, 55, 66],
            cave_back_style: [2, 1, 2, 1],
            tree_tops: [7; TREE_TOPS_BYTES],
            max_raining: 0.25,
            world_flags: std::array::from_fn(|i| BitsByte(i as u8)),
            sundial_cooldown: 2,
            moondial_cooldown: 1,
            saved_ore_tiers: [1, 2, 3, 4, 5, 6, 7],
            invasion_type: 3,
            lobby_id: 0,
            sandstorm_severity: 0.75,
            extra_spawn_points: vec![(10, 20), (30, 40)],
            dungeon_x: 123,
            dungeon_y: 456,
        };
        let body = wi.write_to(NetWriter::open(id::WORLD_INFO)).finish();
        let mut r = NetReader::new(&body[3..]);
        let back = WorldInfo::read_from(&mut r).expect("world info");
        assert_eq!(back, wi);
    }

    #[test]
    fn messages_round_trip_through_the_frame() {
        let msgs: Vec<C2s> = vec![
            C2s::Connect("Terraria326".into()),
            C2s::Password("secret".into()),
            C2s::PlayerInfo(PlayerInfo::with_name(5, "Ann")),
            C2s::Inventory {
                slot: 12,
                item: NetItem {
                    kind: 42,
                    stack: 99,
                    prefix: 3,
                    favorited: true,
                },
            },
            C2s::WorldRequest,
            C2s::SpawnRequest(SpawnRequest {
                x: 100,
                y: -50,
                team: 2,
            }),
            C2s::PlayerSpawn(PlayerSpawn {
                x: 1,
                y: 2,
                respawn_timer: 0,
                deaths_pve: 3,
                deaths_pvp: 4,
                team: 1,
                spawn_context: 0,
            }),
            C2s::PlayerActive {
                player_id: 4,
                active: true,
            },
            C2s::PlayerHp {
                player_id: 4,
                hp: 90,
                max_hp: 200,
            },
            C2s::PlayerState(PlayerState::default()),
            C2s::Edit {
                x: 5,
                y: 6,
                kind: 1,
                tile: 7,
            },
            C2s::Pickup { drop: 3 },
            C2s::NpcStrike(NpcStrike {
                npc: 7,
                generation: 2,
                damage: -13,
                knock_back: 5.5,
                hit_direction: -1,
                crit: true,
            }),
            C2s::Projectile(Box::new(ProjectileState {
                key: projectile_key(3, 42, 9),
                x: 16.0,
                y: -32.5,
                vx: 1.5,
                vy: -2.0,
                kind: 4,
                ai0: Some(0.25),
                ai1: None,
                banner_id: 0,
                damage: 12,
                knock_back: 3.0,
                original_damage: 0,
                ai2: Some(-1.0),
            })),
            C2s::ProjectileDead {
                key: projectile_key(255, 7, 1),
            },
            C2s::Ping,
            // `31-33`, `46-47`, `50`, `53`, `59` — the message families of
            // S1-E-27 (`MessageBuffer.cs:1868-1982`, `:2366-2405`,
            // `:2454-2480`, `:2565-2575`, `:2682-2693`).
            C2s::ChestOpenRequest(ChestOpenRequest { x: 100, y: 200 }),
            C2s::ChestItem(ChestItem {
                chest: 3,
                slot: 7,
                stack: 99,
                prefix: 2,
                kind: 42,
            }),
            C2s::PlayerChest(PlayerChest {
                chest: 3,
                x: 100,
                y: 200,
                name_len: 5,
                name: "chest".into(),
            }),
            // `num24 == 0` (`:1925-1936`): no name follows the length byte.
            C2s::PlayerChest(PlayerChest {
                chest: 3,
                x: 100,
                y: 200,
                name_len: 0,
                name: String::new(),
            }),
            C2s::SignRequest(SignRequest { x: 10, y: 20 }),
            C2s::SignData(SignData {
                sign: 4,
                x: 10,
                y: 20,
                text: "hello".into(),
                sender: 2,
                flags: 0,
            }),
            C2s::PlayerBuffs(PlayerBuffs {
                player_id: 3,
                buffs: vec![1, 2, 24],
            }),
            C2s::HitSwitch(HitSwitch { x: 40, y: 50 }),
            C2s::AddNpcBuff(AddNpcBuff {
                npc: 6,
                buff_type: 24,
                time: 300,
            }),
            // `70`/`71` — the bug net's catch/release (S1-P-i36,
            // `MessageBuffer.cs:3102-3117`).
            C2s::CatchNpc { npc: 12 },
            C2s::ReleaseNpc {
                x: 1600,
                y: 3200,
                npc_type: 356,
                style: 5,
            },
        ];
        for msg in &msgs {
            let frame = msg.encode();
            assert!(frame.len() >= MIN_PACKET && frame.len() <= MAX_PACKET);
            assert_eq!(u16::from_le_bytes([frame[0], frame[1]]), frame.len() as u16);
            assert_eq!(frame[2], msg.msg_id());
            let back = C2s::decode(frame[2], &mut NetReader::new(&frame[3..]))
                .unwrap_or_else(|| panic!("decode failed for {:?} (id {})", msg, frame[2]));
            let re = back.encode();
            assert_eq!(re, frame, "re-encode mismatch for {:?}", msg.msg_id());
        }
    }

    #[test]
    fn server_messages_round_trip() {
        let msgs: Vec<S2c> = vec![
            S2c::Disconnect("bye".into()),
            S2c::PasswordRequest,
            S2c::PlayerId { player_id: 7 },
            S2c::PlayerInfo(PlayerInfo::with_name(1, "Bob")),
            S2c::Inventory {
                player_id: 2,
                slot: 3,
                item: NetItem::default(),
            },
            S2c::WorldInfo(Box::new(WorldInfo {
                time: 5,
                day_time: true,
                blood_moon: false,
                eclipse: false,
                moon_phase: 1,
                max_tiles_x: 100,
                max_tiles_y: 200,
                spawn_tile_x: 3,
                spawn_tile_y: 4,
                world_surface: 5,
                rock_layer: 6,
                world_id: 1,
                world_name: "w".into(),
                game_mode: 0,
                unique_id: [0; 16],
                generator_version: 0,
                moon_type: 0,
                backgrounds: [0; 11],
                ice_back_style: 0,
                jungle_back_style: 0,
                hell_back_style: 0,
                wind_speed_target: 0.0,
                num_clouds: 0,
                tree_x: [0; 3],
                tree_style: [0; 4],
                cave_back_x: [0; 3],
                cave_back_style: [0; 4],
                tree_tops: [0; TREE_TOPS_BYTES],
                max_raining: 0.0,
                world_flags: [BitsByte::new(); 14],
                sundial_cooldown: 0,
                moondial_cooldown: 0,
                saved_ore_tiers: [0; 7],
                invasion_type: 0,
                lobby_id: 0,
                sandstorm_severity: 0.0,
                extra_spawn_points: vec![],
                dungeon_x: 0,
                dungeon_y: 0,
            })),
            S2c::Status {
                text: "Receiving world".into(),
                max: 12,
            },
            S2c::TileBlock(Box::new(TileBlock {
                x: 0,
                y: 0,
                w: 2,
                h: 1,
                records: vec![
                    TileRecord {
                        active: true,
                        kind: 7,
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
                    },
                    TileRecord {
                        active: true,
                        kind: 7,
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
                    },
                ],
            })),
            S2c::InitialSpawn,
            S2c::Ping,
            S2c::PlayerActive {
                player_id: 1,
                active: true,
            },
            S2c::PlayerHp {
                player_id: 1,
                hp: 5,
                max_hp: 6,
            },
            S2c::Joined {
                id: 1,
                name: "a".into(),
            },
            S2c::Left { id: 1 },
            S2c::Projectile(Box::new(ProjectileState {
                key: projectile_key(0, 5, 2),
                x: 1.0,
                y: 2.0,
                vx: 0.5,
                vy: -0.5,
                kind: 6,
                ai0: None,
                ai1: Some(1.0),
                banner_id: 3,
                damage: 9,
                knock_back: 0.0,
                original_damage: 9,
                ai2: None,
            })),
            S2c::ProjectileDead {
                key: projectile_key(1, 4, 3),
                x: 8.0,
                y: 9.0,
            },
            S2c::NpcStrike(NpcStrike {
                npc: 3,
                generation: 1,
                damage: 40,
                knock_back: 2.5,
                hit_direction: 1,
                crit: false,
            }),
            S2c::PlayerState(Box::new(PlayerState {
                player_id: 1,
                flags_a: BitsByte(0b0100_0000),
                flags_b: BitsByte(0b0000_0100),
                flags_c: BitsByte::new(),
                flags_d: BitsByte::new(),
                selected_item: 3,
                x: 12.0,
                y: 34.0,
                vx: 0.5,
                vy: -1.0,
                on_ground: true,
                leg_frame: 2,
                body_frame: 7,
                hp: 80,
                max_hp: 100,
            })),
            S2c::NpcInfo(Box::new(NpcInfo {
                slot: 3,
                generation: 2,
                spawn: false,
                kind: 4,
                x: 64.0,
                y: 96.0,
                vx: 0.0,
                vy: 0.0,
                dir_right: true,
                dir_y_down: false,
                sprite_right: true,
                life_full: false,
                hp: 40,
            })),
            // The life width switch: the > 127 and > 32767 arms.
            S2c::NpcInfo(Box::new(NpcInfo {
                slot: 4,
                generation: 1,
                spawn: true,
                kind: -1,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                dir_right: false,
                dir_y_down: true,
                sprite_right: false,
                life_full: false,
                hp: 9000,
            })),
            S2c::ItemState(Box::new(ItemState {
                slot: 5,
                x: 16.0,
                y: 32.0,
                vx: 1.0,
                vy: -2.0,
                stack: 12,
                prefix: 7,
                kind: 42,
                shimmered: true,
                shimmer_time: 30.0,
            })),
            S2c::ItemState(Box::new(ItemState {
                slot: 6,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                stack: 1,
                prefix: 0,
                kind: 1,
                shimmered: false,
                shimmer_time: 0.0,
            })),
            S2c::ItemDead { slot: 9 },
            S2c::NetModule {
                module: 1,
                body: b"hello module".to_vec(),
            },
            S2c::ItemOwner(Box::new(ItemOwner {
                slot: 5,
                reserved_for: 2,
                hold: 15,
                grab_delay_player: 255,
                grab_delay: 100,
                x: 16.0,
                y: 32.0,
            })),
            // The `32/33/47/50/54/59/69/80/155` families (S1-E-27).
            S2c::ChestItem(ChestItem {
                chest: 2,
                slot: 9,
                stack: 250,
                prefix: 1,
                kind: 700,
            }),
            S2c::PlayerChest(PlayerChest {
                chest: 2,
                x: 30,
                y: 40,
                name_len: 3,
                name: "abc".into(),
            }),
            S2c::SignData(SignData {
                sign: 1,
                x: 2,
                y: 3,
                text: "sign".into(),
                sender: 4,
                flags: 1,
            }),
            S2c::PlayerBuffs(PlayerBuffs {
                player_id: 1,
                buffs: vec![5, 26],
            }),
            S2c::NpcBuffs(NpcBuffs {
                npc: 12,
                buffs: vec![(24, 300), (20, 60)],
            }),
            S2c::HitSwitch(HitSwitch { x: -1, y: 400 }),
            S2c::PlayerChestIndex(PlayerChestIndex {
                player_id: 3,
                chest: -3,
            }),
            S2c::ChestSize(ChestSize {
                chest: 2,
                size: 40,
            }),
            S2c::ChestName(ChestName {
                chest: 2,
                x: 30,
                y: 40,
                name: "renamed".into(),
            }),
        ];
        for msg in &msgs {
            let frame = msg.encode();
            assert_eq!(u16::from_le_bytes([frame[0], frame[1]]), frame.len() as u16);
            let back = S2c::decode(frame[2], &mut NetReader::new(&frame[3..])).expect("decode");
            assert_eq!(back.encode(), frame);
        }
    }

    #[test]
    fn short_packet_is_a_protocol_error() {
        let mut cursor = std::io::Cursor::new(2u16.to_le_bytes());
        assert!(read_frame(&mut cursor).is_err());
    }

    /// `MessageID.cs:111` / `:322`: `49` is `InitialSpawn` and `154` is `Ping`.
    /// The port used to name 49 `WORLD_COMPLETE` and never sent 154 at all -
    /// the RTT diagnostic rode `NetModule 2` instead, which in the C# is
    /// `NetPingModule`, the *map* ping relay (`NetworkInitializer.cs:13`).
    #[test]
    fn the_spawn_and_ping_ids_match_the_message_table() {
        assert_eq!(id::INITIAL_SPAWN, 49);
        assert_eq!(id::PING, 154);
        assert_eq!(S2c::InitialSpawn.encode()[2], 49);
        assert_eq!(S2c::Ping.encode()[2], 154);
        assert_eq!(C2s::Ping.encode()[2], 154);
        assert_eq!(S2c::InitialSpawn.msg_id(), id::INITIAL_SPAWN);
        assert_eq!(S2c::Ping.msg_id(), id::PING);
        assert_eq!(C2s::Ping.msg_id(), id::PING);
        // `PlayerSpawnContext.SpawningIntoWorld == 1`: `Player.Spawn` re-sends
        // the context byte (`Player.cs:37578-37581`), so the join answer must
        // carry 1, not `ReviveFromDeath`'s 0.
        let spawn = C2s::PlayerSpawn(PlayerSpawn {
            x: 1,
            y: 2,
            respawn_timer: 0,
            deaths_pve: 0,
            deaths_pvp: 0,
            team: 0,
            spawn_context: 1,
        })
        .encode();
        assert_eq!(*spawn.last().unwrap(), 1u8, "the spawn context byte");
        let back = C2s::decode(spawn[2], &mut NetReader::new(&spawn[3..])).expect("decode");
        match back {
            C2s::PlayerSpawn(s) => assert_eq!(s.spawn_context, 1),
            other => panic!("expected 12 PlayerSpawn, got {:?}", other.msg_id()),
        }
    }

    /// `97`/`98` (`MessageBuffer.cs:3558-3569`): both bodies are one
    /// `(short)number`, and the ids land at the right bytes.
    #[test]
    fn the_achievement_messages_round_trip() {
        let k = S2c::NpcKilled { npc: 4 }.encode();
        assert_eq!(k[2], 97);
        assert_eq!(&k[3..], &4i16.to_le_bytes());
        match S2c::decode(k[2], &mut NetReader::new(&k[3..])).expect("decode") {
            S2c::NpcKilled { npc } => assert_eq!(npc, 4),
            other => panic!("expected 97, got {:?}", other.msg_id()),
        }
        let p = S2c::ProgressionEvent { event_id: 29 }.encode();
        assert_eq!(p[2], 98);
        assert_eq!(&p[3..], &29i16.to_le_bytes());
        match S2c::decode(p[2], &mut NetReader::new(&p[3..])).expect("decode") {
            S2c::ProgressionEvent { event_id } => assert_eq!(event_id, 29),
            other => panic!("expected 98, got {:?}", other.msg_id()),
        }
    }

    // ---- S1-I-1 wave A: per-id round trips for the join bundle ----
    // Each test pins the frame's id byte, the exact C# body bytes
    // (`NetMessage.cs`'s writer arms) and the decode round trip.

    #[test]
    fn s1_i_1_toggle_pvp_round_trips() {
        // `NetMessage.cs:856-859`: (byte)number + Player.hostile.
        let t = TogglePvp {
            player_id: 7,
            hostile: true,
        };
        let f = S2c::TogglePvp(t).encode();
        assert_eq!(f[2], 30);
        assert_eq!(&f[3..], &[7, 1]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TogglePvp(d) => assert_eq!(d, t),
            other => panic!("expected 30, got {:?}", other.msg_id()),
        }
        let c = C2s::TogglePvp(t).encode();
        assert_eq!(c[2], 30);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TogglePvp(d) => assert_eq!(d, t),
            other => panic!("expected 30, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_player_mana_round_trips() {
        // `NetMessage.cs:961-965`: (byte)number + (short)statMana + (short)statManaMax.
        let m = PlayerMana {
            player_id: 3,
            mana: 180,
            max_mana: 200,
        };
        let f = S2c::PlayerMana(m).encode();
        assert_eq!(f[2], 42);
        let mut body = vec![3];
        body.extend_from_slice(&180i16.to_le_bytes());
        body.extend_from_slice(&200i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerMana(d) => assert_eq!(d, m),
            other => panic!("expected 42, got {:?}", other.msg_id()),
        }
        let c = C2s::PlayerMana(m).encode();
        assert_eq!(c[2], 42);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerMana(d) => assert_eq!(d, m),
            other => panic!("expected 42, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_team_change_round_trips() {
        // `NetMessage.cs:970-974`: (byte)number + (byte)Player.team.
        let t = TeamChange {
            player_id: 2,
            team: 4,
        };
        let f = S2c::TeamChange(t).encode();
        assert_eq!(f[2], 45);
        assert_eq!(&f[3..], &[2, 4]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TeamChange(d) => assert_eq!(d, t),
            other => panic!("expected 45, got {:?}", other.msg_id()),
        }
        let c = C2s::TeamChange(t).encode();
        assert_eq!(c[2], 45);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TeamChange(d) => assert_eq!(d, t),
            other => panic!("expected 45, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_npc_house_round_trips() {
        // `NetMessage.cs:1067-1072`: (short)npc + (short)homeTileX +
        // (short)homeTileY + (byte)householdStatus.
        let h = NpcHouse {
            npc: 17,
            home_x: 4200,
            home_y: 1100,
            status: 2,
        };
        let f = S2c::NpcHouse(h).encode();
        assert_eq!(f[2], 60);
        let mut body = Vec::new();
        body.extend_from_slice(&17i16.to_le_bytes());
        body.extend_from_slice(&4200i16.to_le_bytes());
        body.extend_from_slice(&1100i16.to_le_bytes());
        body.push(2);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::NpcHouse(d) => assert_eq!(d, h),
            other => panic!("expected 60, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_travel_shop_round_trips() {
        // `NetMessage.cs:1134-1141`: TravelShopMaxSlots = 40 shorts
        // (`Main.cs:67029`), no count prefix (`MessageBuffer.cs:3121-3124`).
        let s = TravelShop {
            items: (0..TRAVEL_SHOP_MAX_SLOTS as i16).map(|i| i * 7).collect(),
        };
        let f = S2c::TravelShop(s.clone()).encode();
        assert_eq!(f[2], 72);
        assert_eq!(f.len(), 3 + 2 * TRAVEL_SHOP_MAX_SLOTS);
        let mut body = Vec::new();
        for item in &s.items {
            body.extend_from_slice(&item.to_le_bytes());
        }
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TravelShop(d) => assert_eq!(d, s),
            other => panic!("expected 72, got {:?}", other.msg_id()),
        }
        // A truncated shop (39 slots) must not decode.
        assert!(S2c::decode(72, &mut NetReader::new(&body[..78])).is_none());
    }

    #[test]
    fn s1_i_1_angler_quest_round_trips() {
        // `NetMessage.cs:1145-1148`: (byte)Main.anglerQuest +
        // anglerWhoFinishedToday.Contains(name).
        let q = AnglerQuest {
            quest: 13,
            finished: true,
        };
        let f = S2c::AnglerQuest(q).encode();
        assert_eq!(f[2], 74);
        assert_eq!(&f[3..], &[13, 1]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::AnglerQuest(d) => assert_eq!(d, q),
            other => panic!("expected 74, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_revenge_marker_round_trips() {
        // `CoinLossRevengeSystem.cs:275-286` (WriteSelfTo) vs `:321-324`
        // (AddMarkerFromReader): int, Vector2, int, float, int, int, int,
        // float, bool.
        let m = RevengeMarker {
            unique_id: 9,
            x: 1234.5,
            y: 678.25,
            npc_net_id: 44,
            npc_hp_percent: 0.75,
            npc_type: 26,
            npc_ai_style: 3,
            coins_value: 50000,
            base_value: 12.5,
            spawned_from_statue: true,
        };
        let f = S2c::RevengeMarker(m).encode();
        assert_eq!(f[2], 126);
        let mut body = Vec::new();
        body.extend_from_slice(&9i32.to_le_bytes());
        body.extend_from_slice(&1234.5f32.to_le_bytes());
        body.extend_from_slice(&678.25f32.to_le_bytes());
        body.extend_from_slice(&44i32.to_le_bytes());
        body.extend_from_slice(&0.75f32.to_le_bytes());
        body.extend_from_slice(&26i32.to_le_bytes());
        body.extend_from_slice(&3i32.to_le_bytes());
        body.extend_from_slice(&50000i32.to_le_bytes());
        body.extend_from_slice(&12.5f32.to_le_bytes());
        body.push(1);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::RevengeMarker(d) => assert_eq!(d, m),
            other => panic!("expected 126, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_dead_player_round_trips() {
        // `NetMessage.cs:1569-1571`: (byte)number, nothing else.
        let f = S2c::DeadPlayer { player_id: 5 }.encode();
        assert_eq!(f[2], 135);
        assert_eq!(&f[3..], &[5]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::DeadPlayer { player_id } => assert_eq!(player_id, 5),
            other => panic!("expected 135, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_misc_event_value_round_trips() {
        // `NetMessage.cs:1591-1594`: (byte)number + (int)number2; the join
        // bundle's kind 0 is the credits roll (`CreditsRollEvent.cs:27-33`).
        let v = MiscEventValue {
            kind: 0,
            value: 28800,
        };
        let f = S2c::MiscEventValue(v).encode();
        assert_eq!(f[2], 140);
        let mut body = vec![0];
        body.extend_from_slice(&28800i32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::MiscEventValue(d) => assert_eq!(d, v),
            other => panic!("expected 140, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_projectile_trackers_round_trips() {
        // `NetMessage.cs:1603-1610`: (byte)number + two
        // TrackedProjectileReference.Write bodies — (int)Key + (short)type
        // (`TrackedProjectileReference.cs:22-26`).
        let t = ProjectileTrackers {
            player_id: 6,
            piggy_bank: TrackedProjectileRef {
                key: 123456,
                kind: 700,
            },
            void_lens: TrackedProjectileRef {
                key: -2,
                kind: 701,
            },
        };
        let f = S2c::ProjectileTrackers(t).encode();
        assert_eq!(f[2], 142);
        let mut body = vec![6];
        body.extend_from_slice(&123456i32.to_le_bytes());
        body.extend_from_slice(&700i16.to_le_bytes());
        body.extend_from_slice(&(-2i32).to_le_bytes());
        body.extend_from_slice(&701i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ProjectileTrackers(d) => assert_eq!(d, t),
            other => panic!("expected 142, got {:?}", other.msg_id()),
        }
        let c = C2s::ProjectileTrackers(t).encode();
        assert_eq!(c[2], 142);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ProjectileTrackers(d) => assert_eq!(d, t),
            other => panic!("expected 142, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1_sync_loadout_round_trips() {
        // `NetMessage.cs:1624-1628`: (byte)number + (byte)CurrentLoadoutIndex
        // + the hideVisibleAccessory ushort (`WriteAccessoryVisibility`,
        // `:1882-1893`: bit i per bool[10] slot).
        let hide: u16 = (1 << 0) | (1 << 3) | (1 << 9);
        let l = SyncLoadout {
            player_id: 4,
            loadout: 2,
            hide_accessory: hide,
        };
        let f = S2c::SyncLoadout(l).encode();
        assert_eq!(f[2], 147);
        let mut body = vec![4, 2];
        body.extend_from_slice(&hide.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SyncLoadout(d) => assert_eq!(d, l),
            other => panic!("expected 147, got {:?}", other.msg_id()),
        }
        let c = C2s::SyncLoadout(l).encode();
        assert_eq!(c[2], 147);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::SyncLoadout(d) => assert_eq!(d, l),
            other => panic!("expected 147, got {:?}", other.msg_id()),
        }
        // The visibility packing itself: bool[10] -> ushort -> bool[10].
        let mut flags = [false; 10];
        flags[0] = true;
        flags[3] = true;
        flags[9] = true;
        let mut packed: u16 = 0;
        for (i, f) in flags.iter().enumerate() {
            if *f {
                packed |= 1 << i;
            }
        }
        assert_eq!(packed, hide);
        for (i, f) in flags.iter().enumerate() {
            assert_eq!((packed & (1 << i)) != 0, *f);
        }
    }

    #[test]
    fn w4_d99_misc_data_sync_round_trips() {
        // `NetMessage.cs:1010-1013` (case 51), verbatim: (byte)number
        // (subject) then (byte)number2 (kind). Kind 3 = the sundial ask
        // (`Main.cs:6175`, subject Main.myPlayer); the host forces the
        // subject to the sender (`MessageBuffer.cs:2485-2488`) and only
        // kind 2 echoes (`:2498`) — both directions carry the same layout.
        let m = MiscDataSync { subject: 2, kind: 3 };
        let f = C2s::MiscDataSync(m).encode();
        assert_eq!(f[2], 51);
        assert_eq!(&f[3..], &[2u8, 3]);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::MiscDataSync(d) => assert_eq!(d, m),
            other => panic!("expected 51, got {:?}", other.msg_id()),
        }
        // The host's kind-2 rod-sound echo (`MessageBuffer.cs:2498`).
        let echo = MiscDataSync { subject: 2, kind: 2 };
        let s = S2c::MiscDataSync(echo).encode();
        assert_eq!(s[2], 51);
        assert_eq!(&s[3..], &[2u8, 2]);
        match S2c::decode(s[2], &mut NetReader::new(&s[3..])).expect("decode") {
            S2c::MiscDataSync(d) => assert_eq!(d, echo),
            other => panic!("expected 51, got {:?}", other.msg_id()),
        }
        // Kind 4's subject is an NPC slot (the big mimic smoke,
        // `NPC.cs:77498`) — the byte rides verbatim either way.
        let mimic = MiscDataSync { subject: 199, kind: 4 };
        let f = S2c::MiscDataSync(mimic).encode();
        assert_eq!(&f[3..], &[199u8, 4]);
    }

    #[test]
    fn w4_d99_spawn_boss_event_round_trips() {
        // `NetMessage.cs:1073-1076` (case 61), verbatim: (short)number (the
        // player to spawn on) + (short)number2 (the selector). Positive =
        // an NPCID boss (Eye of Cthulhu 4, `Player.cs:43782`); negative =
        // the event ladder — pin -19 (slime rain, `Main.cs:64752`) so the
        // sign bit survives the i16 round trip. Client→server only.
        for (player, event) in [(1i16, 4i16), (0, -19), (2, -16)] {
            let e = SpawnBossEvent { player, event };
            let f = C2s::SpawnBossEvent(e).encode();
            assert_eq!(f[2], 61);
            let mut body = Vec::new();
            body.extend_from_slice(&player.to_le_bytes());
            body.extend_from_slice(&event.to_le_bytes());
            assert_eq!(&f[3..], &body);
            match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                C2s::SpawnBossEvent(d) => assert_eq!(d, e),
                other => panic!("expected 61, got {:?}", other.msg_id()),
            }
        }
    }

    #[test]
    fn w4_d99_teleport_entity_round_trips() {
        // `NetMessage.cs:1088-1103` (case 65), verbatim: BitsByte27 (bit0/1
        // = kind low bits, bit2 = getPosFromPlayer, bit3 = extraInfo gate),
        // (short)entity, float x, float y, (byte)style, gated (int)extra.
        // Kind 0 with extraInfo 0 writes NO tail (`number7 != 0` gate) —
        // the rod-of-discord send (`Player.cs:45513`, style 1).
        let rod = TeleportEntity {
            entity: 3,
            x: 3200.5,
            y: -64.25,
            style: 1,
            kind: 0,
            get_pos_from_player: false,
            extra_info: None,
        };
        let f = C2s::TeleportEntity(rod).encode();
        assert_eq!(f[2], 65);
        let mut body = vec![0u8]; // flags: kind 0, no bit2, no bit3
        body.extend_from_slice(&3i16.to_le_bytes());
        body.extend_from_slice(&3200.5f32.to_le_bytes());
        body.extend_from_slice(&(-64.25f32).to_le_bytes());
        body.push(1);
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::TeleportEntity(d) => assert_eq!(d, rod),
            other => panic!("expected 65, got {:?}", other.msg_id()),
        }
        // Kind 2 (player-to-player, `Player.cs:42363`) → bit1 only; the
        // host broadcast (`MessageBuffer.cs:3008`) with a portal color.
        let p2p = TeleportEntity {
            entity: 1,
            x: 100.0,
            y: 200.0,
            style: 4,
            kind: 2,
            get_pos_from_player: true,
            extra_info: Some(5),
        };
        let s = S2c::TeleportEntity(p2p).encode();
        assert_eq!(s[2], 65);
        let mut body = vec![0b1110u8]; // bit1 kind, bit2 getpos, bit3 extra
        body.extend_from_slice(&1i16.to_le_bytes());
        body.extend_from_slice(&100.0f32.to_le_bytes());
        body.extend_from_slice(&200.0f32.to_le_bytes());
        body.push(4);
        body.extend_from_slice(&5i32.to_le_bytes());
        assert_eq!(&s[3..], &body);
        match S2c::decode(s[2], &mut NetReader::new(&s[3..])).expect("decode") {
            S2c::TeleportEntity(d) => assert_eq!(d, p2p),
            other => panic!("expected 65, got {:?}", other.msg_id()),
        }
        // Kind 1 (NPC teleport, `NPC.cs:81056`) → bit0; kind 3 (the client
        // ack, `MessageBuffer.cs:2993`) → bits 0+1.
        for (kind, flags) in [(1u8, 0b0001u8), (3, 0b0011)] {
            let t = TeleportEntity {
                entity: 7,
                x: 0.0,
                y: 0.0,
                style: 2,
                kind,
                get_pos_from_player: false,
                extra_info: None,
            };
            let f = C2s::TeleportEntity(t).encode();
            assert_eq!(f[3], flags);
            match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                C2s::TeleportEntity(d) => assert_eq!(d, t),
                other => panic!("expected 65, got {:?}", other.msg_id()),
            }
        }
    }

    #[test]
    fn w4_d99_request_teleport_round_trips() {
        // `NetMessage.cs:1142-1144` (case 73), verbatim: one (byte)kind —
        // the reader's ladder 0..4 (`MessageBuffer.cs:3128-3145`:
        // potion/magic conch/demon conch/shellphone/no-space).
        for kind in 0u8..=4 {
            let q = RequestTeleport { kind };
            let f = C2s::RequestTeleport(q).encode();
            assert_eq!(f[2], 73);
            assert_eq!(&f[3..], &[kind]);
            match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                C2s::RequestTeleport(d) => assert_eq!(d, q),
                other => panic!("expected 73, got {:?}", other.msg_id()),
            }
        }
    }

    #[test]
    fn w4_d99_angler_quest_finished_is_bodyless() {
        // No `case 75` writer arm in NetMessage.cs and the reader consumes
        // nothing (`MessageBuffer.cs:3154-3163` — it only adds the sender's
        // name on the host): the frame is exactly [len=3][75].
        let f = C2s::AnglerQuestFinished.encode();
        assert_eq!(f, vec![3, 0, 75]);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::AnglerQuestFinished => {}
            other => panic!("expected 75, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn w4_d99_temporary_animation_round_trips() {
        // `NetMessage.cs:1154-1159` (case 77), verbatim: (short)number
        // (animationType), (ushort)number2 (tileType), (short)number3 (x),
        // (short)number4 (y) — `SendTemporaryAnimation`'s argument order
        // (`:2594`); the reader feeds Animation.NewTemporaryAnimation
        // (`MessageBuffer.cs:3184`). Server→client, one addressee.
        let a = TemporaryAnimation {
            animation_type: 3,
            tile_type: 593, // the campfire toggle (`Wiring.cs:1704`)
            x: 1234,
            y: -56,
        };
        let f = S2c::TemporaryAnimation(a).encode();
        assert_eq!(f[2], 77);
        let mut body = Vec::new();
        body.extend_from_slice(&3i16.to_le_bytes());
        body.extend_from_slice(&593u16.to_le_bytes());
        body.extend_from_slice(&1234i16.to_le_bytes());
        body.extend_from_slice(&(-56i16).to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TemporaryAnimation(d) => assert_eq!(d, a),
            other => panic!("expected 77, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_d99_invasion_progress_round_trips() {
        // `NetMessage.cs:1160-1165` (case 78), verbatim: (int)number
        // (progress), (int)number2 (max), (sbyte)number3 (icon),
        // (sbyte)number4 (wave). Pin the DD2 shape (icon 3, wave 5,
        // `DD2Event.cs:191`) and a negative sbyte so the sign survives.
        for p in [
            InvasionProgress {
                progress: 120,
                progress_max: 200,
                icon: 3,
                wave: 5,
            },
            InvasionProgress {
                progress: 0,
                progress_max: 1,
                icon: -1,
                wave: -2,
            },
        ] {
            let f = S2c::InvasionProgress(p).encode();
            assert_eq!(f[2], 78);
            let mut body = Vec::new();
            body.extend_from_slice(&p.progress.to_le_bytes());
            body.extend_from_slice(&p.progress_max.to_le_bytes());
            body.push(p.icon as u8);
            body.push(p.wave as u8);
            assert_eq!(&f[3..], &body);
            match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::InvasionProgress(d) => assert_eq!(d, p),
                other => panic!("expected 78, got {:?}", other.msg_id()),
            }
        }
    }

    #[test]
    fn w4_d99_player_portal_teleport_round_trips() {
        // `NetMessage.cs:1337-1345` (case 96), verbatim field order:
        // (byte)number (player), (short)number4 (color index), number2 (x),
        // number3 (y), WriteVector2(player.velocity) — the velocity is the
        // player's live one, deliberately different from the destination
        // so a swapped pair cannot pass. Both directions
        // (`PortalHelper.cs:190` client send, `MessageBuffer.cs:3554` relay).
        let p = PlayerPortalTeleport {
            player: 1,
            color_index: 4, // owner 2 * 2 + ai[1] 0 (`PortalHelper.cs:181`)
            x: 2048.5,
            y: -256.25,
            velocity_x: -2.5,
            velocity_y: -0.1, // the `ent.velocity.Y = -num * 0.1f` floor
        };
        let mut body = vec![1u8];
        body.extend_from_slice(&4i16.to_le_bytes());
        body.extend_from_slice(&2048.5f32.to_le_bytes());
        body.extend_from_slice(&(-256.25f32).to_le_bytes());
        body.extend_from_slice(&(-2.5f32).to_le_bytes());
        body.extend_from_slice(&(-0.1f32).to_le_bytes());
        let f = C2s::PlayerPortalTeleport(p).encode();
        assert_eq!(f[2], 96);
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::PlayerPortalTeleport(d) => assert_eq!(d, p),
            other => panic!("expected 96, got {:?}", other.msg_id()),
        }
        let s = S2c::PlayerPortalTeleport(p).encode();
        assert_eq!(s[2], 96);
        assert_eq!(&s[3..], &body);
        match S2c::decode(s[2], &mut NetReader::new(&s[3..])).expect("decode") {
            S2c::PlayerPortalTeleport(d) => assert_eq!(d, p),
            other => panic!("expected 96, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_d99_moonlord_countdown_round_trips() {
        // `NetMessage.cs:1383-1386` (case 103), verbatim: the raw
        // NPC.MaxMoonLordCountdown int THEN NPC.MoonLordCountdown —
        // deliberately different values so a swapped pair cannot pass;
        // the client stores them in that order (`MessageBuffer.cs:3711-3712`).
        let m = MoonlordCountdown {
            max: 600,
            countdown: 360,
        };
        let f = S2c::MoonlordCountdown(m).encode();
        assert_eq!(f[2], 103);
        let mut body = Vec::new();
        body.extend_from_slice(&600i32.to_le_bytes());
        body.extend_from_slice(&360i32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::MoonlordCountdown(d) => assert_eq!(d, m),
            other => panic!("expected 103, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_d99_special_fx_round_trips() {
        // `NetMessage.cs:1424-1431` (case 112), verbatim: (byte)number
        // (effect), (int)number2 (x), (int)number3 (y), (byte)number4,
        // (short)number5, (byte)number6. Kind 1 = the shaken-tree send
        // (`WorldGen.cs:63412`: height 1, passStyle, shake flag 1); kind 2
        // = fairy (`NPC.cs:48584`: number5 = type - 583). The reader tests
        // number6 == 1 (`MessageBuffer.cs:3822`).
        for f in [
            SpecialFx {
                effect: 1,
                x: 1200,
                y: 340,
                b4: 1,
                s5: 7,
                b6: 1,
            },
            SpecialFx {
                effect: 2,
                x: -64,
                y: 800,
                b4: 0,
                s5: 3, // fairyCritter type 586 - 583
                b6: 0,
            },
        ] {
            let frame = S2c::SpecialFx(f).encode();
            assert_eq!(frame[2], 112);
            let mut body = vec![f.effect];
            body.extend_from_slice(&f.x.to_le_bytes());
            body.extend_from_slice(&f.y.to_le_bytes());
            body.push(f.b4);
            body.extend_from_slice(&f.s5.to_le_bytes());
            body.push(f.b6);
            assert_eq!(&frame[3..], &body);
            match S2c::decode(frame[2], &mut NetReader::new(&frame[3..])).expect("decode") {
                S2c::SpecialFx(d) => assert_eq!(d, f),
                other => panic!("expected 112, got {:?}", other.msg_id()),
            }
        }
    }

    #[test]
    fn w4_d99_crystal_invasion_start_round_trips() {
        // `NetMessage.cs:1432-1435` (case 113), verbatim: (short)number (x)
        // + (short)number2 (y) — the SummonCrystal tile coords
        // (`DD2Event.cs:576`). Negative coords pin the sign.
        let c = CrystalInvasionStart { x: -120, y: 450 };
        let f = C2s::CrystalInvasionStart(c).encode();
        assert_eq!(f[2], 113);
        let mut body = Vec::new();
        body.extend_from_slice(&(-120i16).to_le_bytes());
        body.extend_from_slice(&450i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::CrystalInvasionStart(d) => assert_eq!(d, c),
            other => panic!("expected 113, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_d99_crystal_invasion_wipe_is_bodyless() {
        // No `case 114` writer arm and the reader consumes nothing
        // (`MessageBuffer.cs:3855-3860` DD2Event.WipeEntities; the send is
        // `SendData(114)` with no args, `DD2Event.cs:636`): [len=3][114].
        let f = S2c::CrystalInvasionWipe.encode();
        assert_eq!(f, vec![3, 0, 114]);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::CrystalInvasionWipe => {}
            other => panic!("expected 114, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn w4_d99_crystal_invasion_wait_round_trips() {
        // `NetMessage.cs:1436-1438` (case 116), verbatim: the raw
        // (int)number — `_timeLeftUntilSpawningBegins`
        // (`DD2Event.cs:949`); the client stores TimeLeftBetweenWaves
        // (`MessageBuffer.cs:3864`).
        let t = CrystalInvasionWait { time_left: 1800 };
        let f = S2c::CrystalInvasionWait(t).encode();
        assert_eq!(f[2], 116);
        assert_eq!(&f[3..], &1800i32.to_le_bytes());
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::CrystalInvasionWait(d) => assert_eq!(d, t),
            other => panic!("expected 116, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_d99_remove_revenge_marker_round_trips() {
        // `NetMessage.cs:1518-1520` (case 127), verbatim: the raw
        // (int)number — the marker UniqueID (`CoinLossRevengeSystem.cs:451`);
        // the client destroys it (`MessageBuffer.cs:4051-4054`). Matches
        // the 126 marker body's leading unique_id field.
        let m = RemoveRevengeMarker { unique_id: 42 };
        let f = S2c::RemoveRevengeMarker(m).encode();
        assert_eq!(f[2], 127);
        assert_eq!(&f[3..], &42i32.to_le_bytes());
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::RemoveRevengeMarker(d) => assert_eq!(d, m),
            other => panic!("expected 127, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_d99_lucy_popup_round_trips() {
        // `NetMessage.cs:1595-1602` (case 141), verbatim: (byte)number
        // (source), (byte)number2 (variation), number3/number4 (the
        // velocity floats) THEN number5/number6 (the position ints) —
        // deliberately different magnitudes so a swapped pair cannot
        // pass. Both directions (`LucyAxeMessage.cs:133` client send,
        // `MessageBuffer.cs:4274` relay).
        let l = LucyPopup {
            source: 2,
            variation: 7,
            velocity_x: 1.5,
            velocity_y: -3.25,
            x: 51200,
            y: -1024,
        };
        let mut body = vec![2u8, 7];
        body.extend_from_slice(&1.5f32.to_le_bytes());
        body.extend_from_slice(&(-3.25f32).to_le_bytes());
        body.extend_from_slice(&51200i32.to_le_bytes());
        body.extend_from_slice(&(-1024i32).to_le_bytes());
        let f = C2s::LucyPopup(l).encode();
        assert_eq!(f[2], 141);
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::LucyPopup(d) => assert_eq!(d, l),
            other => panic!("expected 141, got {:?}", other.msg_id()),
        }
        let s = S2c::LucyPopup(l).encode();
        assert_eq!(s[2], 141);
        assert_eq!(&s[3..], &body);
        match S2c::decode(s[2], &mut NetReader::new(&s[3..])).expect("decode") {
            S2c::LucyPopup(d) => assert_eq!(d, l),
            other => panic!("expected 141, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_d99_crystal_invasion_skip_is_bodyless() {
        // No `case 143` writer arm and the reader consumes nothing
        // (`MessageBuffer.cs:4298-4303` AttemptToSkipWaitTime on the host;
        // the send is `SendData(143)` with no args, `DD2Event.cs:1790`):
        // [len=3][143].
        let f = C2s::CrystalInvasionSkip.encode();
        assert_eq!(f, vec![3, 0, 143]);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::CrystalInvasionSkip => {}
            other => panic!("expected 143, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
    // ---- W4-c27: the entity-state ids — byte-verbatim witnesses ----
    // Each test pins the frame's id byte, the exact C# body bytes
    // (`NetMessage.cs`'s writer arms) and the decode round trip.

    #[test]
    fn w4_c27_set_time_round_trips() {
        // `NetMessage.cs:531-536`: (byte)dayTime + (int)Main.time +
        // sunModY/moonModY, both `short` (`Main.cs:1185`/`:1187`).
        let t = SetTime {
            day_time: false,
            time: 32_400,
            sun_mod_y: -120,
            moon_mod_y: 45,
        };
        let f = S2c::SetTime(t).encode();
        assert_eq!(f[2], 18);
        let mut body = vec![0u8];
        body.extend_from_slice(&32_400i32.to_le_bytes());
        body.extend_from_slice(&(-120i16).to_le_bytes());
        body.extend_from_slice(&45i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SetTime(d) => assert_eq!(d, t),
            other => panic!("expected 18, got {:?}", other.msg_id()),
        }
        // `MessageBuffer.cs:1293`: the day flag is `== 1`, not `!= 0` —
        // a byte 2 reads as night.
        let mut body2 = vec![2u8];
        body2.extend_from_slice(&0i32.to_le_bytes());
        body2.extend_from_slice(&0i16.to_le_bytes());
        body2.extend_from_slice(&0i16.to_le_bytes());
        match S2c::decode(18, &mut NetReader::new(&body2)).expect("decode") {
            S2c::SetTime(d) => assert!(!d.day_time, "byte 2 is not day"),
            other => panic!("expected 18, got {:?}", other.msg_id()),
        }
        // Truncated bodies are malformed.
        assert!(S2c::decode(18, &mut NetReader::new(&[1, 0, 0])).is_none());
    }

    #[test]
    fn w4_c27_player_zone_round_trips() {
        // `NetMessage.cs:933-944`: (byte)number + zone1..5 + (byte)townNPCs.
        // zone5 bit 0 is ZoneShimmer (`Player.cs:3811-3821`).
        let z = PlayerZone {
            player_id: 6,
            zone1: 0b1010_0101,
            zone2: 0x0F,
            zone3: 0xF0,
            zone4: 0x33,
            zone5: 0b0000_0001,
            town_npcs: 3,
        };
        let body = [6, 0b1010_0101, 0x0F, 0xF0, 0x33, 0x01, 3];
        let f = S2c::PlayerZone(z).encode();
        assert_eq!(f[2], 36);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerZone(d) => assert_eq!(d, z),
            other => panic!("expected 36, got {:?}", other.msg_id()),
        }
        let c = C2s::PlayerZone(z).encode();
        assert_eq!(c[2], 36);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerZone(d) => assert_eq!(d, z),
            other => panic!("expected 36, got {:?}", other.msg_id()),
        }
        assert!(C2s::decode(36, &mut NetReader::new(&body[..6])).is_none());
    }

    #[test]
    fn w4_c27_item_anim_round_trips() {
        // `NetMessage.cs:956-960`: (byte)number + itemRotation float +
        // (short)itemAnimation.
        let a = ItemAnim {
            player_id: 2,
            rotation: -1.25,
            animation: 17,
        };
        let mut body = vec![2u8];
        body.extend_from_slice(&(-1.25f32).to_le_bytes());
        body.extend_from_slice(&17i16.to_le_bytes());
        let f = S2c::ItemAnim(a).encode();
        assert_eq!(f[2], 41);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemAnim(d) => assert_eq!(d, a),
            other => panic!("expected 41, got {:?}", other.msg_id()),
        }
        let c = C2s::ItemAnim(a).encode();
        assert_eq!(c[2], 41);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ItemAnim(d) => assert_eq!(d, a),
            other => panic!("expected 41, got {:?}", other.msg_id()),
        }
        assert!(C2s::decode(41, &mut NetReader::new(&body[..6])).is_none());
    }

    #[test]
    fn w4_c27_mana_effect_round_trips() {
        // `NetMessage.cs:966-969`: (byte)number + (short)number2.
        let m = ManaEffect {
            player_id: 5,
            amount: -40,
        };
        let mut body = vec![5u8];
        body.extend_from_slice(&(-40i16).to_le_bytes());
        let f = S2c::ManaEffect(m).encode();
        assert_eq!(f[2], 43);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ManaEffect(d) => assert_eq!(d, m),
            other => panic!("expected 43, got {:?}", other.msg_id()),
        }
        let c = C2s::ManaEffect(m).encode();
        assert_eq!(c[2], 43);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ManaEffect(d) => assert_eq!(d, m),
            other => panic!("expected 43, got {:?}", other.msg_id()),
        }
        assert!(C2s::decode(43, &mut NetReader::new(&[5])).is_none());
    }

    #[test]
    fn w4_c27_misc_data_sync_round_trips() {
        // `NetMessage.cs:1010-1013`: (byte)number + (byte)number2.
        let d = MiscDataSync {
            player_id: 3,
            kind: 6,
        };
        let f = S2c::MiscDataSync(d).encode();
        assert_eq!(f[2], 51);
        assert_eq!(&f[3..], &[3, 6]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::MiscDataSync(x) => assert_eq!(x, d),
            other => panic!("expected 51, got {:?}", other.msg_id()),
        }
        let c = C2s::MiscDataSync(d).encode();
        assert_eq!(c[2], 51);
        assert_eq!(&c[3..], &[3, 6]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::MiscDataSync(x) => assert_eq!(x, d),
            other => panic!("expected 51, got {:?}", other.msg_id()),
        }
        assert!(C2s::decode(51, &mut NetReader::new(&[3])).is_none());
    }

    #[test]
    fn w4_c27_player_buff_pvp_round_trips() {
        // `NetMessage.cs:1041-1045`: (byte)number + (ushort)number2 +
        // (int)number3.
        let b = PlayerBuffPvp {
            player_id: 1,
            buff_type: 400,
            time: 600,
        };
        let mut body = vec![1u8];
        body.extend_from_slice(&400u16.to_le_bytes());
        body.extend_from_slice(&600i32.to_le_bytes());
        let f = S2c::PlayerBuffPvp(b).encode();
        assert_eq!(f[2], 55);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerBuffPvp(x) => assert_eq!(x, b),
            other => panic!("expected 55, got {:?}", other.msg_id()),
        }
        let c = C2s::PlayerBuffPvp(b).encode();
        assert_eq!(c[2], 55);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerBuffPvp(x) => assert_eq!(x, b),
            other => panic!("expected 55, got {:?}", other.msg_id()),
        }
        assert!(C2s::decode(55, &mut NetReader::new(&body[..5])).is_none());
    }

    #[test]
    fn w4_c27_sync_dodge_round_trips() {
        // `NetMessage.cs:1077-1080`: (byte)number + (byte)number2.
        let d = SyncDodge {
            player_id: 4,
            kind: 5, // MysticSash (`Player.cs:38115`)
        };
        let f = S2c::SyncDodge(d).encode();
        assert_eq!(f[2], 62);
        assert_eq!(&f[3..], &[4, 5]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SyncDodge(x) => assert_eq!(x, d),
            other => panic!("expected 62, got {:?}", other.msg_id()),
        }
        let c = C2s::SyncDodge(d).encode();
        assert_eq!(c[2], 62);
        assert_eq!(&c[3..], &[4, 5]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::SyncDodge(x) => assert_eq!(x, d),
            other => panic!("expected 62, got {:?}", other.msg_id()),
        }
        assert!(C2s::decode(62, &mut NetReader::new(&[4])).is_none());
    }

    #[test]
    fn w4_c27_player_stealth_round_trips() {
        // `NetMessage.cs:1197-1204`: (byte)id + the live stealth float.
        let s = PlayerStealth {
            player_id: 2,
            stealth: 0.625,
        };
        let mut body = vec![2u8];
        body.extend_from_slice(&0.625f32.to_le_bytes());
        let f = S2c::PlayerStealth(s).encode();
        assert_eq!(f[2], 84);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerStealth(x) => assert_eq!(x, s),
            other => panic!("expected 84, got {:?}", other.msg_id()),
        }
        let c = C2s::PlayerStealth(s).encode();
        assert_eq!(c[2], 84);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerStealth(x) => assert_eq!(x, s),
            other => panic!("expected 84, got {:?}", other.msg_id()),
        }
        assert!(C2s::decode(84, &mut NetReader::new(&body[..4])).is_none());
    }
}
