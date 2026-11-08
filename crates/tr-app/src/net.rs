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
    pub const WORLD_CHUNK: u8 = 15; // internal (dead id in 1.4.5.8): the whole-world
                                    // blob stream. M6-NET-3 replaces the producer with `10 TileBlock` sections.
    pub const PLAYER_HP: u8 = 16; // MessageBuffer.cs:1095-1112
    pub const PASSWORD_REQUEST: u8 = 37; // MessageBuffer.cs:2203-2216 — no body
    pub const PASSWORD: u8 = 38; // MessageBuffer.cs:2218-2231
    pub const PLAYER_SPAWN: u8 = 12; // MessageBuffer.cs:908-930
    pub const WORLD_COMPLETE: u8 = 49; // MessageBuffer.cs:2448-2452 — world load done
    pub const PLAYER_STATE: u8 = 13; // NetMessage.cs:446-513 / MessageBuffer.cs:952-1059
    pub const ITEM_DROP: u8 = 21; // NetMessage.cs:646-669 / MessageBuffer.cs:1445-1535
    pub const ITEM_OWNER: u8 = 22; // NetMessage.cs:675-685 / MessageBuffer.cs:1552-1562
    pub const ITEM_DEAD: u8 = 151; // NetMessage.cs:671-674 / MessageBuffer.cs:1537-1550
    pub const NPC_INFO: u8 = 23; // NetMessage.cs:686-748 / MessageBuffer.cs:1563-1704

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
    /// `15 WorldChunk` — internal: one slice of the whole-world blob.
    WorldChunk {
        index: i32,
        total: i32,
        data: Vec<u8>,
    },
    /// `49 WorldComplete` — the world stream is done.
    WorldComplete,
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
            _ => return None,
        })
    }
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
            Self::WorldChunk { .. } => id::WORLD_CHUNK,
            Self::WorldComplete => id::WORLD_COMPLETE,
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
        }
    }

    pub fn encode(&self) -> Vec<u8> {
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
            Self::WorldChunk { index, total, data } => w.i32(*index).i32(*total).bytes(data),
            Self::WorldComplete => w,
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
            Self::Edit { x, y, kind, tile } => w.i32(*x).i32(*y).u8(*kind).u16(*tile),
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
            id::WORLD_CHUNK => {
                let index = r.i32()?;
                let total = r.i32()?;
                let data = r.bytes()?;
                Self::WorldChunk { index, total, data }
            }
            id::WORLD_COMPLETE => Self::WorldComplete,
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
            _ => return None,
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

    /// Streams the world blob as `WorldChunk` messages (the internal `15`
    /// seam for M6-NET-3's `10 TileBlock` sections): the frame cap forces the
    /// split, and the count is announced through `9 Status` first.
    pub fn send_world(&self, id: u32, blob: &[u8]) {
        const PAYLOAD_CAP: usize = MAX_PACKET - 2 - 1 - 4 - 4;
        if blob.is_empty() {
            self.send(
                id,
                &S2c::Status {
                    text: "Receiving world".into(),
                    max: 1,
                },
            );
            self.send(
                id,
                &S2c::WorldChunk {
                    index: 0,
                    total: 1,
                    data: Vec::new(),
                },
            );
            self.send(id, &S2c::WorldComplete);
            return;
        }
        let total = blob.len().div_ceil(PAYLOAD_CAP);
        self.send(
            id,
            &S2c::Status {
                text: "Receiving world".into(),
                max: total as i32,
            },
        );
        let mut index: i32 = 0;
        for chunk in blob.chunks(PAYLOAD_CAP) {
            self.send(
                id,
                &S2c::WorldChunk {
                    index,
                    total: total as i32,
                    data: chunk.to_vec(),
                },
            );
            index += 1;
        }
        self.send(id, &S2c::WorldComplete);
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
        ];
        for msg in &msgs {
            let frame = msg.encode();
            assert!(frame.len() >= MIN_PACKET && frame.len() <= MAX_PACKET);
            assert_eq!(u16::from_le_bytes([frame[0], frame[1]]), frame.len() as u16);
            assert_eq!(frame[2], msg.msg_id());
            let back = C2s::decode(frame[2], &mut NetReader::new(&frame[3..])).expect("decode");
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
            S2c::WorldChunk {
                index: 1,
                total: 3,
                data: vec![1, 2, 3],
            },
            S2c::WorldComplete,
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
            S2c::ItemOwner(Box::new(ItemOwner {
                slot: 5,
                reserved_for: 2,
                hold: 15,
                grab_delay_player: 255,
                grab_delay: 100,
                x: 16.0,
                y: 32.0,
            })),
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
}
