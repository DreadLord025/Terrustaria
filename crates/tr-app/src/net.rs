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
    /// `34 ChestUpdates` (S1-I-1 wave I): the chest/dresser/piggy-bank
    /// place/destroy relay (`NetMessage.cs:913-927`,
    /// `MessageBuffer.cs:1984-2158`; the client send sites
    /// `Player.cs:53787-53795`).
    pub const CHEST_UPDATES: u8 = 34;
    /// `81 CombatTextInt` (S1-I-1 wave I): a peer's damage number —
    /// `(float)x, (float)y, RGB, (int)amount` (`NetMessage.cs:1179-1186`,
    /// `MessageBuffer.cs:3227-3233`, host→client only).
    pub const COMBAT_TEXT_INT: u8 = 81;
    // (`35 PLAYER_HEAL` / `36 PLAYER_ZONE` are the W4-d96 constants below —
    // keep-one dedupe at the g48 merge, d96/d98 both landed them.)
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
    /// `157 TeamChangeFromUI` (`MessageID.cs:328`) — the writer shares 45's
    /// case (`NetMessage.cs:970-974`) so the body is the same `TeamChange`;
    /// the reader adds the team-spawn tail (`MessageBuffer.cs:2355-2363`:
    /// `teamBasedSpawnsSeed` + an extra spawn point → `CheckSection` +
    /// `158 ExtraSpawnSectionLoaded`).
    pub const TEAM_CHANGE_UI: u8 = 157;

    // ---- S1-I-1 wave B: the world-sync region (section send / tile square /
    // spawn tile), census against `MessageID.cs` — bodies + codecs only, the
    // dispatcher apply/relay arms and the send fan-out are S1-I-4. ----
    /// `17 TileManipulation` (`MessageID.cs:43`) — the single-tile edit,
    /// still live: writer `NetMessage.cs:524-530`, reader
    /// `MessageBuffer.cs:1119-1288` (host relay `:1285`, the `20` fallback
    /// `SendTileSquare(-1, x, y, 5)` when a placement survives, `:1281`).
    /// research/17 called it superseded by 20 — wrong, both are sent; the
    /// port's internal `INT_EDIT` stand-in is unchanged by this body.
    pub const TILE_EDIT: u8 = 17;
    // `11 TILE_FRAME_SECTION` is the W4-d89 constant below (keep-one dedupe
    // at the g48 merge — d89/d90 both landed the id).
    pub const DOOR_TOGGLE: u8 = 19; // MessageID.cs:47 / NetMessage.cs:537-542 / MessageBuffer.cs:1299-1334
    pub const TILE_PAINT: u8 = 63; // MessageID.cs:139 / NetMessage.cs:1081-1087 / MessageBuffer.cs:2909-2928
    pub const WALL_PAINT: u8 = 64; // MessageID.cs:141 / NetMessage.cs:1081-1087 / MessageBuffer.cs:2929-2948
    pub const TILE_PICKING: u8 = 125; // MessageID.cs:264 / NetMessage.cs:1509-1514 / MessageBuffer.cs:4023-4042
    /// `158 ExtraSpawnSectionLoaded` (`MessageID.cs:330`) — the team-swap
    /// respawn after the extra spawn section streamed in
    /// (`NetMessage.cs:1659-1661`; reader `MessageBuffer.cs:4450-4456`
    /// calls `Spawn(PlayerSpawnContext.TeamSwap)`).
    pub const EXTRA_SPAWN_LOADED: u8 = 158;
    /// `159 RequestSection` (`MessageID.cs:332`) — the client asks for a
    /// section (`NetMessage.cs:1662-1665` writes `short`s; the reader reads
    /// `ushort`s, `MessageBuffer.cs:4457-4462` → `SendSection`). The port
    /// streams unrequested (`host_stream_sections`); this is the body for
    /// the on-request path.
    pub const SECTION_REQUEST: u8 = 159;

    // ---- S1-I-1 wave B (W4-d89): the join/lobby 1-16 region census ----
    // Every id 1..=16 now has a constant; the two the wave-A census missed:
    /// `11 TileFrameSection` (`MessageID.cs:29-30`, marked `[Old]` — framing
    /// happens as needed after TileSection). The writer arm DOES exist
    /// (`NetMessage.cs:427-432`: four `(short)`s) and the client reads it
    /// (`MessageBuffer.cs:895-899` → `WorldGen.SectionTileFrame`,
    /// `WorldGen.cs:80343`); no C# call site sends it any more.
    pub const TILE_FRAME_SECTION: u8 = 11;
    /// `15 Unknown15` (`MessageID.cs:38-39`, `[Old("Deprecated.")]`): no
    /// writer arm; the reader's deprecated group (`MessageBuffer.cs:4483-
    /// 4490`, with 25/26/44/67/83/93) accepts and ignores it.
    pub const UNKNOWN_15: u8 = 15;

    // ---- S1-I-1 projectile-family first half (W4-d93) ----
    // Census vs the C# table: 27/28/29 already had ids + bodies (M6-NET);
    // 29's C2s body now carries the Vector2 verbatim (it used to hardcode
    // NaN); 95 was NOT-FOUND and lands here. The NPC-side 28 arms belong to
    // the NPC half (W4-d91/d92); the minion/portal-teleport ids (99/100/115)
    // are the second half (W4-d94).
    /// `95 MurderSomeoneElsesPortal` (`MessageID.cs:204`) — a client's new
    /// portal intersected a peer's portal-gun projectile (type 602):
    /// `(ushort)owner + (byte)ai[1] form` (`NetMessage.cs:1333-1336`, sent
    /// by `PortalHelper.RemoveIntersectingPortals`, `PortalHelper.cs:364-367`).
    /// Host-only read (`MessageBuffer.cs:3519-3537`): kill the first matching
    /// active 602 and relay one `29`. Client→server only — the C# has no
    /// host send site for 95.
    pub const MURDER_PORTAL: u8 = 95;
    /// `128 LandGolfBallInCup` (`MessageID.cs:270`) — a client's golf ball
    /// (a projectile) dropped into a cup: `(byte)owner + (ushort)hitX +
    /// (ushort)hitY + (ushort)numberOfHits + (ushort)ballType`
    /// (`NetMessage.cs:1521-1527`, sent by `GolfHelper.ContactListener.
    /// PutBallInCup` at `netMode == 1`, `GolfHelper.cs:107-110`). The host
    /// relays it to the others (`MessageBuffer.cs:4058-4074`, the
    /// `ignoreClient = owner` resend); the client runs the cup text/effects
    /// (`PutBallInCup_TextAndEffects`, the golf FX of M4-GOLF-1).
    pub const GOLF_BALL_IN_CUP: u8 = 128;
    /// `109 MassWireOperation` (`MessageID.cs:232`) — the Grand Design's
    /// wire-drag (projectile 651, `Projectile.cs:72053-72066`) or the
    /// Multicolor Wrench's (`Player.cs:46911`): `(short)ps.X + (short)ps.Y +
    /// (short)pe.X + (short)pe.Y + (byte)WiresUI.Settings.ToolMode`
    /// (`NetMessage.cs:1412-1418`). Client→server only — the host runs
    /// `Wiring.MassWireOperation` under a temporary ToolMode swap and never
    /// relays the 109 (`MessageBuffer.cs:3774-3788`).
    pub const MASS_WIRE_OPERATION: u8 = 109;
    /// `110 MassWireOperationPay` (`MessageID.cs:234`) — the host's wire/
    /// actuator bill after a mass operation: `(short)itemType + (short)count
    /// + (byte)player` (`NetMessage.cs:1419-1423`), sent twice — 530 Wire and
    /// 849 Actuator (`Wiring.cs:439-443`). Server→client only; the client
    /// consumes the items and clears `wireOperationsCooldown` when the bill
    /// is its own (`MessageBuffer.cs:3789-3807`).
    pub const MASS_WIRE_OPERATION_PAY: u8 = 110;

    // ---- W4-d96 (S1-I-1 player-family first half): the low-region ids ----
    // The census + the d96/d97/c27 lane split live in
    // `docs/research/17-netcode.md` ("W4 player-family census"). Bodies +
    // codecs only; the dispatcher apply/relay arms stay parked (S1-I-4).
    /// `35 PlayerHeal` (`MessageID.cs:81`) — the heal combat-text relay
    /// (`NetMessage.cs:929-932`, `MessageBuffer.cs:2160-2177`; the send site
    /// is `Player.HealEffect`, `Player.cs:4522-4529`).
    pub const PLAYER_HEAL: u8 = 35;
    /// `40 SyncTalkNPC` (`MessageID.cs:91`) — which NPC a player talks to
    /// (`NetMessage.cs:952-955`, `MessageBuffer.cs:2256-2270`;
    /// `Player.SetTalkNPC`, `Player.cs:4375`).
    pub const TALK_NPC: u8 = 40;
    /// `36 SyncPlayerZone` (`MessageID.cs:83`) — the five zone BitsBytes plus
    /// the town-NPC count (`NetMessage.cs:933-944`,
    /// `MessageBuffer.cs:2178-2201`; the send sites are the zone-change test
    /// `Main.cs:17634` and the 900-tick heartbeat `:64533`).
    pub const PLAYER_ZONE: u8 = 36;
    /// `41 ItemRotationAndAnimation` (`MessageID.cs:93`) — the aimed-item
    /// rotation + animation counter (`NetMessage.cs:956-960`,
    /// `MessageBuffer.cs:2271-2289`; sent beside a `13` at the useStyle-5/13
    /// aim sites, `Player.cs:43693`, `:47374`, `:47380`).
    pub const ITEM_ANIM: u8 = 41;
    /// `43 ManaEffect` (`MessageID.cs:97`) — the mana combat-text relay,
    /// `35 PlayerHeal`'s mana twin (`NetMessage.cs:966-969`,
    /// `MessageBuffer.cs:2307-2324`; the send site is `Player.ManaEffect`,
    /// `Player.cs:4531-4538`).
    pub const MANA_EFFECT: u8 = 43;

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
    /// `49 InitialSpawn` received: the player spawns in (`SpawningIntoWorld`,
    /// `MessageBuffer.cs:2447-2453`). NOT "world stream complete" — the old
    /// `WORLD_COMPLETE` reading was wrong (research/17 deviation 3, fixed
    /// S1-E-28); the stream completes with the section walk itself.
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
    /// `Player.hostile` — fed by `30 TogglePVP` (`MessageBuffer.cs:1859`).
    pub hostile: bool,
    /// `Player.team` — fed by `12 PlayerSpawn` (`MessageBuffer.cs:914`); the
    /// PVP chat line's colour keys on it (`:1864`, `Main.teamColor`).
    pub team: u8,
    /// `Player.SpawnX/SpawnY` — fed by `12` (`MessageBuffer.cs:909-910`).
    pub spawn_x: i16,
    pub spawn_y: i16,
    /// `Player.respawnTimer` / the death counters (`MessageBuffer.cs:911-913`).
    pub respawn_timer: i32,
    pub deaths_pve: i16,
    pub deaths_pvp: i16,
    /// `Player.dead` — the `12` reader latches it when `respawnTimer > 0`
    /// (`MessageBuffer.cs:915-918`); the `16` reader derives it from
    /// `statLife <= 0` (`:1111`).
    pub dead: bool,
    /// `Player.zone1..zone5` — fed by `36 SyncPlayerZone`
    /// (`MessageBuffer.cs:2187-2191`); `zone5` bit 0 is the faeling edge.
    pub zones: [u8; 5],
    /// `Player.townNPCs` (`MessageBuffer.cs:2192`).
    pub town_npcs: u8,
    /// `Player.talkNPC` — fed by `40 SyncTalkNPC`
    /// (`MessageBuffer.cs:2264`); `-1` = not talking, the C# field default.
    /// `None` is the port's `-1` (the `Default`).
    pub talk_npc: Option<i16>,
}

/// `msg 13 PlayerState`'s body (`NetMessage.cs:446-513`): the player id, four
/// BitsByte groups, the selected item, the full-float position and the
/// conditional velocity/mount/potion/camera bodies — all verbatim, in the C#
/// gate order (`:492-508`: velocity b2, mount b7, potion c6, camera d5). The
/// port stores the mount/potion/camera values but simulates none of them
/// (the reader's `SetMount`/potion/camera arms are S1-I-4); the trailing
/// anim/hp fields are the port's two-client extension — the C# derives them
/// from each peer's local simulation, which the port does not run for remote
/// players (deviation in `docs/ANALYSIS.md`).
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
    /// `(ushort)mount.Type`, gated by flags_b bit 7 (`NetMessage.cs:496-499`;
    /// the reader `SetMount`/`Dismount`, `MessageBuffer.cs:1021-1028`).
    pub mount_type: u16,
    /// `PotionOfReturnOriginalUsePosition`, gated by flags_c bit 6
    /// (`NetMessage.cs:500-504`; reader `MessageBuffer.cs:1029-1038`).
    pub potion_use: (f32, f32),
    /// `PotionOfReturnHomePosition`, the second Vector2 of the same gate.
    pub potion_home: (f32, f32),
    /// `netCameraTarget`, gated by flags_d bit 5 (`NetMessage.cs:505-508`;
    /// reader `MessageBuffer.cs:1051`).
    pub camera_target: (f32, f32),
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
        // `velocity != Vector2.Zero` gate (`NetMessage.cs:492-495`).
        if self.flags_b.get(2) {
            w = w.f32(self.vx).f32(self.vy);
        }
        // `mount.Active` gate (`NetMessage.cs:496-499`): (ushort)mount.Type.
        if self.flags_b.get(7) {
            w = w.u16(self.mount_type);
        }
        // `PotionOfReturnOriginalUsePosition.HasValue` gate
        // (`NetMessage.cs:500-504`): TWO Vector2s, use then home.
        if self.flags_c.get(6) {
            w = w
                .f32(self.potion_use.0)
                .f32(self.potion_use.1)
                .f32(self.potion_home.0)
                .f32(self.potion_home.1);
        }
        // `netCameraTarget.HasValue` gate (`NetMessage.cs:505-508`).
        if self.flags_d.get(5) {
            w = w.f32(self.camera_target.0).f32(self.camera_target.1);
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
        // `MessageBuffer.cs:993-997`: the velocity stays Zero without bit 2.
        let (vx, vy) = if flags_b.get(2) {
            (r.f32()?, r.f32()?)
        } else {
            (0.0, 0.0)
        };
        // `MessageBuffer.cs:1021-1028`: SetMount(type) or Dismount — the port
        // stores the type only (no mount sim yet), 0 = dismounted.
        let mount_type = if flags_b.get(7) { r.u16()? } else { 0 };
        // `MessageBuffer.cs:1029-1038`: the two Vector2s or both null.
        let (potion_use, potion_home) = if flags_c.get(6) {
            ((r.f32()?, r.f32()?), (r.f32()?, r.f32()?))
        } else {
            ((0.0, 0.0), (0.0, 0.0))
        };
        // `MessageBuffer.cs:1051`: netCameraTarget or null — read AFTER the
        // sleeping/autoreuse/... flag copies but at this wire position.
        let camera_target = if flags_d.get(5) {
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
            mount_type,
            potion_use,
            potion_home,
            camera_target,
            on_ground,
            leg_frame,
            body_frame,
            hp,
            max_hp,
        })
    }
}

/// `msg 23 NPCInfo`'s body (`NetMessage.cs:686-763`, read at
/// `MessageBuffer.cs:1563-1704`) — the verbatim C# layout: the slot byte, the
/// generation byte (bumped on a kind change so the client re-creates the
/// slot, the C#'s `existing.generation != generation` spawn test), the
/// SyncAnchor-offset position, the velocity, the `(ushort)target`, the two
/// flag bytes, the flag-gated `ai[0..4]` floats, the `(short)netID`, then the
/// optional player-count byte, difficulty float, width-switched life and the
/// catchable-kind `releaseOwner` byte. `x`/`y` are the RAW position — the
/// `Size * NPCID.Sets.SyncAnchor[type]` offset (`NPCID.cs:10457`: only kind
/// 50, the bound town NPC, is non-zero) is applied by the builders
/// (`netglue::npc_info_of`) and undone by the client, exactly like the C#.
/// `statsAreScaledForThisManyPlayers` has no port home yet and rides its 1
/// default (flags2 b0 clear); `difficulty` likewise rides 1.0 (b2 clear).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpcInfo {
    /// The host's NPC slot index (`whoAmI`).
    pub slot: u8,
    /// Bumped on a kind change / fresh spawn → the client re-creates the slot.
    pub generation: u8,
    /// `spawnNeedsSyncing` (flags2 b3) — a full-create for a slot the client
    /// already has.
    pub spawn: bool,
    pub kind: i16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// The `(ushort)target` (`NetMessage.cs:693`); the port's `Npc.target`
    /// sentinel is 255 (`NPC.cs:8539`'s `ClearTarget`).
    pub target: u16,
    /// flags1 b0: `direction > 0`.
    pub dir_right: bool,
    /// flags1 b1: `directionY > 0`.
    pub dir_y_down: bool,
    /// flags1 b6: `spriteDirection > 0`.
    pub sprite_right: bool,
    /// flags1 b7: `life == lifeMax` → the life block is omitted.
    pub life_full: bool,
    pub hp: i32,
    /// `lifeMax` — the width byte's selector (`NetMessage.cs:737-744`:
    /// `> 32767` → 4, `> 127` → 2, else 1), NOT the current life. On decode
    /// it rides the local kind's max (`MessageBuffer.cs:1657-1660`).
    pub life_max: i32,
    /// `ai[0..4]` (`NPC.maxAI = 4`, `NPC.cs:95158`): flags1 b2..b5 gate one
    /// float each, written in index order between flags2 and the netID
    /// (`NetMessage.cs:704-724`).
    pub ai: [f32; 4],
    /// `SpawnedFromStatue` (flags2 b1, `NetMessage.cs:713`): the client zeroes
    /// the coin value (`MessageBuffer.cs:1683-1687`).
    pub spawned_from_statue: bool,
    /// `difficulty` (flags2 b2): written only when `!= 1f`
    /// (`NetMessage.cs:714`, `:730-733`).
    pub difficulty: f32,
    /// flags2 b4: `spawnNeedsSyncing && shimmerTransparency > 0f`
    /// (`NetMessage.cs:716`); the client sets `shimmerTransparency = 1f`
    /// (`MessageBuffer.cs:1688-1691`).
    pub shimmer: bool,
    /// `releaseOwner` — the trailing byte, present only for
    /// `Main.npcCatchable[kind]` kinds (`NetMessage.cs:759-762`,
    /// `MessageBuffer.cs:1696-1699`). 255 = unowned.
    pub release_owner: u8,
}

impl NpcInfo {
    /// `NPCID.Sets.SyncAnchor` (`NPCID.cs:10457`): the only non-zero row is
    /// vanilla type 50 (King Slime) → `(0.5, 1)`. The offset is
    /// `Size * anchor` — the port uses the kind's base `size()` (the C#'s
    /// live `Size` rides King Slime's hp-scaled `size_override`, a render
    /// detail the wire snapshot does not carry; deviation in ANALYSIS).
    fn sync_anchor_px(kind: i16) -> (f32, f32) {
        match crate::npc::NpcKind::from_u16(kind.max(0) as u16) {
            Some(k) if k.cs_id() == 50 => {
                let (w, h) = k.size();
                (w * 0.5, h)
            }
            _ => (0.0, 0.0),
        }
    }

    /// `Main.npcCatchable[type]` (`Main.cs:9326-9418`) keyed on the VANILLA
    /// type — the wire `kind` is the port-local protocol id, so map through
    /// `cs_id` first (`bug_net::npc_catchable` is the verbatim set). The
    /// `type >= 0 && type < NPCID.Count` bounds gate (`NetMessage.cs:759`)
    /// rides the `cs_id` sentinel (`u16::MAX` = unmapped).
    fn catchable(kind: i16) -> bool {
        crate::npc::NpcKind::from_u16(kind.max(0) as u16).is_some_and(|k| {
            let cs = k.cs_id();
            cs < crate::npc::NpcKind::CS_ID_COUNT as u16
                && crate::bug_net::npc_catchable(cs as i32)
        })
    }

    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut flags1 = BitsByte::new();
        flags1.set(0, self.dir_right);
        flags1.set(1, self.dir_y_down);
        // b2..b5: the ai[] presence bits (`NetMessage.cs:704-707`).
        for i in 0..4 {
            flags1.set(i + 2, self.ai[i] != 0.0);
        }
        flags1.set(6, self.sprite_right);
        flags1.set(7, self.life_full);
        let mut flags2 = BitsByte::new();
        // b0 `statsAreScaledForThisManyPlayers > 1`: the port keeps the 1
        // default (no multiplayer stat scaling yet) — the bit stays clear.
        flags2.set(1, self.spawned_from_statue);
        flags2.set(2, self.difficulty != 1.0);
        flags2.set(3, self.spawn);
        flags2.set(4, self.spawn && self.shimmer);
        let (ax, ay) = Self::sync_anchor_px(self.kind);
        let mut w = w
            .u8(self.slot)
            .u8(self.generation)
            .f32(self.x + ax)
            .f32(self.y + ay)
            .f32(self.vx)
            .f32(self.vy)
            .u16(self.target)
            .bitsbyte(flags1)
            .bitsbyte(flags2);
        // The ai floats, in index order, BEFORE the netID
        // (`NetMessage.cs:718-725`).
        for i in 0..4 {
            if self.ai[i] != 0.0 {
                w = w.f32(self.ai[i]);
            }
        }
        w = w.i16(self.kind);
        if self.difficulty != 1.0 {
            w = w.f32(self.difficulty);
        }
        // The width-switched life block (`NetMessage.cs:734-758`): the width
        // byte is chosen by lifeMax, skipped when the NPC is at full health.
        if !self.life_full {
            let hp = self.hp.max(0);
            if self.life_max > 32767 {
                w = w.u8(4).i32(hp);
            } else if self.life_max > 127 {
                w = w.u8(2).i16(hp as i16);
            } else {
                w = w.u8(1).i8(hp as i8);
            }
        }
        if Self::catchable(self.kind) {
            w = w.u8(self.release_owner);
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
        let target = r.u16()?;
        let flags1 = r.bitsbyte()?;
        let flags2 = r.bitsbyte()?;
        let mut ai = [0.0f32; 4];
        for i in 0..4 {
            if flags1.get(i + 2) {
                ai[i] = r.f32()?;
            }
        }
        let kind = r.i16()?;
        // flags2 b0: the player-count byte (`MessageBuffer.cs:1589-1593`) —
        // consumed; the port's SetDefaults takes no scaling override yet.
        if flags2.get(0) {
            let _player_count = r.u8()?;
        }
        let difficulty = if flags2.get(2) { r.f32()? } else { 1.0 };
        let hp = if flags1.get(7) {
            -1 // full health: the client keeps its local lifeMax
        } else {
            match r.u8()? {
                2 => r.i16()? as i32,
                4 => r.i32()?,
                _ => r.i8()? as i32,
            }
        };
        let release_owner = if Self::catchable(kind) { r.u8()? } else { 255 };
        let (ax, ay) = Self::sync_anchor_px(kind);
        Some(Self {
            slot,
            generation,
            spawn: flags2.get(3),
            kind,
            x: x - ax,
            y: y - ay,
            vx,
            vy,
            target,
            dir_right: flags1.get(0),
            dir_y_down: flags1.get(1),
            sprite_right: flags1.get(6),
            life_full: flags1.get(7),
            hp,
            // The width byte is re-derived from the local kind on re-encode
            // (the client's lifeMax, `MessageBuffer.cs:1657-1660`); an
            // unknown kind keeps the narrow byte-1 arm.
            life_max: crate::npc::NpcKind::from_u16(kind.max(0) as u16)
                .map(|k| k.max_hp())
                .unwrap_or(1),
            ai,
            spawned_from_statue: flags2.get(1),
            difficulty,
            shimmer: flags2.get(4),
            release_owner,
        })
    }
}

/// `msg 21 ItemDrop`'s body (`NetMessage.cs:646-669`): the slot, the
/// position/velocity, the stack, the prefix, the flag byte and the type, plus
/// the conditional shimmer fields and the bit-3 enemy-grab tail byte
/// (S1-I-1 wave I). C#'s `noBroadcast`/`instanced` split and
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
    /// `WorldItem.enemyGrabDelayTime` (`WorldItem.cs:28`) — the monster-grab
    /// lock (`:405`); bit 3 carries it clamped to a byte
    /// (`NetMessage.cs:652`/`:665-668`, S1-I-1 wave I).
    pub enemy_grab_delay: u8,
}

impl ItemState {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        // `NetMessage.cs:646-669`: bit 2 shimmer, bit 3 enemyGrabDelayTime.
        let mut flags = BitsByte::new();
        flags.set(2, self.shimmered || self.shimmer_time > 0.0);
        flags.set(3, self.enemy_grab_delay > 0);
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
        let w = if self.shimmered || self.shimmer_time > 0.0 {
            w.bool(self.shimmered).f32(self.shimmer_time)
        } else {
            w
        };
        // `:665-668` — `(byte)MathHelper.Clamp(enemyGrabDelayTime, 0f, 255f)`;
        // the field is already the clamped byte on this struct.
        if self.enemy_grab_delay > 0 {
            w.u8(self.enemy_grab_delay)
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
        // `MessageBuffer.cs:1455`/`:1459` — bit 3's conditional tail byte.
        let enemy_grab_delay = if flags.get(3) { r.u8()? } else { 0 };
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
            enemy_grab_delay,
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
    /// Group 1: difficulty 1 → bit 0, 2 → bit 1, 3 → bit **3**; bit 2 is
    /// `extraAccessory` (`NetMessage.cs:166-179`; the reader re-derives the
    /// same mapping, `MessageBuffer.cs:323-341`).
    pub flags1: BitsByte,
    pub flags2: BitsByte,
    pub flags3: BitsByte,
}

impl PlayerInfo {
    /// The defaults the port sends: the name is the only field with a home
    /// today (`App.player_name`); every appearance field rides the wire as
    /// vanilla-zero until the character milestone lands. Note the read side
    /// clamps `voiceVariant` into 1..4 (`MessageBuffer.cs:300`), so a decoded
    /// `with_name` carries `voice_variant = 1`, not the written 0.
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
            // `MathHelper.Clamp(skinVariant, 0, PlayerVariantID.Count - 1)`
            // (`MessageBuffer.cs:298`; `PlayerVariantID.Count = 12`,
            // `ID/PlayerVariantID.cs:55`).
            skin_variant: skin_variant.min(11),
            // `Utils.Clamp(voiceVariant, 1, 4)` (`MessageBuffer.cs:300`).
            voice_variant: voice_variant.clamp(1, 4),
            // NaN → 0, then `Utils.Clamp(voicePitchOffset, -1, 1)`
            // (`MessageBuffer.cs:302-306`).
            voice_pitch_offset: if voice_pitch_offset.is_nan() {
                0.0
            } else {
                voice_pitch_offset.clamp(-1.0, 1.0)
            },
            // `hair >= 228` normalises to 0 (`MessageBuffer.cs:308-311`).
            hair: if hair >= 228 { 0 } else { hair },
            // `ReadString().Trim().Trim()` (`MessageBuffer.cs:312`).
            name: name.trim().to_string(),
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

/// `msg 12 PlayerSpawn`'s body (`NetMessage.cs:433-445`; the read side is
/// `MessageBuffer.cs:901-951`): the player id first, then where the player
/// spawned, the respawn timer, the death counters, the team and the spawn
/// context. The server forces the id to the sender (`MessageBuffer.cs:903-907`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSpawn {
    /// `Player.whoAmI`; server-forced on read (`MessageBuffer.cs:903-907`).
    pub player_id: u8,
    pub x: i16,
    pub y: i16,
    pub respawn_timer: i32,
    pub deaths_pve: i16,
    pub deaths_pvp: i16,
    pub team: u8,
    /// `PlayerSpawnContext`, the low byte (`MessageBuffer.cs:919`).
    pub spawn_context: u8,
}

/// `msg 11 TileFrameSection`'s body (`NetMessage.cs:427-432`): the section
/// rectangle as four `(short)`s — `WorldGen.SectionTileFrame(sectionStartX,
/// sectionStartY, sectionEndXInclusive, sectionEndYInclusive)`
/// (`MessageBuffer.cs:895-899`, `WorldGen.cs:80343`). Deprecated on the wire
/// (`MessageID.cs:29-30`): no C# `SendData(11)` call site remains.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileFrameSection {
    pub start_x: i16,
    pub start_y: i16,
    pub end_x: i16,
    pub end_y: i16,
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

/// `msg 34 ChestUpdates`'s body (`NetMessage.cs:913-927`, S1-I-1 wave I):
/// the action byte (0/2/4 place chest/dresser/piggy-bank, 1/3/5 destroy —
/// the client send sites `Player.cs:53785-53796`), the tile, the style, and
/// the resulting chest index. The writer's last short is `number5` on the
/// server and a literal `0` on the client (`:918-927`); the host reader
/// zeroes it before its place/destroy pass (`MessageBuffer.cs:1991-1994`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChestUpdates {
    /// The C#'s `b4` action byte.
    pub action: u8,
    pub x: i16,
    pub y: i16,
    pub style: i16,
    /// `PlaceChest`'s return (`-1` = refused) or `Chest.FindChest`'s index
    /// on the destroy relays; the client's own send writes 0.
    pub chest: i16,
}

/// `msg 81 CombatTextInt`'s body (`NetMessage.cs:1179-1186`, S1-I-1 wave I):
/// two floats, `WriteRGB`'s three bytes and the `(int)` amount. The client
/// truncates the floats and spawns `CombatText.NewText` at
/// `Rectangle(x, y, 0, 0)` (`MessageBuffer.cs:3227-3233`); the one C# send
/// site is `NPC.HealEffect`'s broadcast (`NPC.cs:77440-77443`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CombatTextInt {
    pub x: f32,
    pub y: f32,
    pub color: Rgb,
    pub amount: i32,
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

// (`msg 35 PlayerHeal` / `msg 36 SyncPlayerZone` bodies are the W4-d96
// structs below — keep-one dedupe at the g48 merge; d96's zone words are
// `BitsByte`, d98's plain `u8` form encodes the identical seven bytes.)

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

/// `msg 128 LandGolfBallInCup`'s body (`NetMessage.cs:1521-1527`, S1-I-1
/// W4-d93): `(byte)number` (the owner), `(ushort)number5`/`(ushort)number6`
/// (the cup's tile coords), `(ushort)number2` (the hit count, the ball's
/// `ai[1]`) and `(ushort)number3` (the ball's projectile type). The writer's
/// field order is NOT the argument order — the reader mirrors it
/// (`MessageBuffer.cs:4060-4064`: owner, x, y, hits, type).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GolfBallInCup {
    pub owner: u8,
    pub x: u16,
    pub y: u16,
    pub hits: u16,
    pub ball_type: u16,
}

/// `msg 109 MassWireOperation`'s body (`NetMessage.cs:1412-1418`, S1-I-1
/// W4-d93): the drag's start/end tile points as four `(short)`s, then the
/// `(byte)` tool mode — the `WiresUI.Settings.MultiToolMode` flags
/// (`WiresUI.cs:13-21`: Red 1 / Green 2 / Blue 4 / Yellow 8 / Actuator 0x10 /
/// Cutter 0x20). The reader runs the operation under a temporary ToolMode
/// swap and restores it (`MessageBuffer.cs:3781-3786`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MassWireOperation {
    pub start_x: i16,
    pub start_y: i16,
    pub end_x: i16,
    pub end_y: i16,
    pub tool_mode: u8,
}

/// `msg 110 MassWireOperationPay`'s body (`NetMessage.cs:1419-1423`, S1-I-1
/// W4-d93): `(short)number` (the item type — 530 Wire or 849 Actuator),
/// `(short)number2` (the consumed count) and `(byte)number3` (the billed
/// player). The client only pays when `number3 == Main.myPlayer`
/// (`MessageBuffer.cs:3797`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MassWireOperationPay {
    pub item_type: i16,
    pub count: i16,
    pub player: u8,
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
/// TeamChangeFromUI` shares the case and the body (`NetMessage.cs:970-971`)
/// but adds the team-spawn tail (`MessageBuffer.cs:2355-2363` → `158`);
/// its id + codec landed with wave B (`id::TEAM_CHANGE_UI`).
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

// ---- S1-I-1 wave B: the world-sync region's payload bodies ----

/// `msg 17 TileManipulation`'s body (`NetMessage.cs:524-530`): the edit kind
/// byte, the tile as two `short`s, the `short` payload and a `byte` style.
/// The reader (`MessageBuffer.cs:1119-1288`) dispatches kinds 0-23
/// (KillTile/PlaceTile/KillWall/PlaceWall/wires/slope/actuator/gates/replace,
/// `:1157-1275`); the host counts spam (`:1141-1146`), forces `fail` when the
/// section was never sent (`:1148-1151`) and relays as `17` — or a
/// `SendTileSquare(-1, x, y, 5)` when the placement survived (`:1281`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileEdit {
    /// `b13` — the edit kind (`MessageBuffer.cs:1157-1275`).
    pub kind: u8,
    pub x: i16,
    pub y: i16,
    /// `num152` — the tile/wall type or the slope id; kind 14's slope.
    pub value: i16,
    /// `num153` — `PlaceTile`'s style (`MessageBuffer.cs:1173`).
    pub style: u8,
}

// `msg 11 TileFrameSection`'s body is the W4-d89 struct above (keep-one
// dedupe at the g48 merge — d89/d90 both landed it, byte-identical).

/// `msg 19 ToggleDoorState`'s body (`NetMessage.cs:537-542`): the action
/// byte, the tile and `number4 == 1f` → a direction byte (1 = right/down).
/// The reader maps 0/1 open/close door, 2/3 trapdoor up/down, 4/5 tall gate
/// open/close and relays with the normalised direction
/// (`MessageBuffer.cs:1299-1334`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoorToggle {
    /// `b5` — 0 open, 1 close, 2/3 trapdoor, 4/5 tall gate.
    pub action: u8,
    pub x: i16,
    pub y: i16,
    /// `num47`: the reader reads `!= 0` → 1 else -1; the wire byte is 0/1.
    pub direction: u8,
}

/// `msg 63/64 SyncTilePaintOrCoating / SyncWallPaintOrCoating`'s shared body
/// (`NetMessage.cs:1081-1087`): the tile, the paint id byte and the coat flag
/// byte — `coat == 0` paints (`WorldGen.paintTile`/`paintWall`), anything
/// else coats (`paintCoatTile`/`paintCoatWall`,
/// `MessageBuffer.cs:2909-2948`), then the host relays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TilePaint {
    pub x: i16,
    pub y: i16,
    pub paint: u8,
    /// `b12`/`b10`: 0 = paint, non-zero = illuminant coat.
    pub coat: u8,
}

/// `msg 125 SyncTilePicking`'s body (`NetMessage.cs:1509-1514`): the player
/// id (host-overridden to the sender), the picked tile and the pick damage
/// byte (`Player.GetOtherPlayersPickTile`, `Player.cs:53751`; the host
/// relays to the others, `MessageBuffer.cs:4023-4042`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TilePicking {
    pub player_id: u8,
    pub x: i16,
    pub y: i16,
    pub damage: u8,
}

// ---- W4-d96 (S1-I-1 player-family first half): the low-region bodies ----
// Bodies + codecs only; the dispatcher apply/relay arms stay parked (S1-I-4).

/// `msg 35 PlayerHeal`'s body (`NetMessage.cs:929-932`): the player id then
/// the heal amount as a `short`. The server forces the id to the sender
/// (`MessageBuffer.cs:2162-2166`); the read side raises `HealEffect` for a
/// peer (`:2168-2171`) and relays (`:2172-2175`). The send site is
/// `Player.HealEffect` (`Player.cs:4522-4529`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerHeal {
    pub player_id: u8,
    pub amount: i16,
}

/// `msg 40 SyncTalkNPC`'s body (`NetMessage.cs:952-955`): the player id then
/// `Player.talkNPC` as a `short` (`-1` = not talking). The server forces the
/// id to the sender (`MessageBuffer.cs:2258-2262`), applies `SetTalkNPC`
/// (`Player.cs:4375`) and relays (`:2265-2268`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TalkNpc {
    pub player_id: u8,
    pub npc: i16,
}

/// `msg 36 SyncPlayerZone`'s body (`NetMessage.cs:933-944`): the player id,
/// the five `zone1..5` BitsBytes (`Player.cs:1364-1372`) then
/// `(byte)townNPCs` (`Player.cs:1412` is an `int`; the writer truncates).
/// The server forces the id to the sender (`MessageBuffer.cs:2180-2184`),
/// stores the six bytes (`:2187-2192`) and relays (`:2199`); the
/// `zone5[0]` false→true edge spawns faelings host-side (`:2195-2198`,
/// `NPC.Spawner.SpawnFaelings` — the apply arm is S1-I-4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerZone {
    pub player_id: u8,
    pub zone1: BitsByte,
    pub zone2: BitsByte,
    pub zone3: BitsByte,
    pub zone4: BitsByte,
    pub zone5: BitsByte,
    /// `(byte)player.townNPCs` — the count of town NPCs near the player.
    pub town_npcs: u8,
}

/// `msg 41 ItemRotationAndAnimation`'s body (`NetMessage.cs:956-960`): the
/// player id, the `itemRotation` float (`Player.cs:1498`) then
/// `(short)itemAnimation` (`Player.cs:3014` is an `int`; the writer
/// truncates). The server forces the id to the sender
/// (`MessageBuffer.cs:2273-2277`), stores both fields, re-derives `channel`
/// from the selected item (`:2281-2283`) and relays (`:2284-2287`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemAnim {
    pub player_id: u8,
    pub rotation: f32,
    pub animation: i16,
}

/// `msg 43 ManaEffect`'s body (`NetMessage.cs:966-969`): the player id then
/// the mana amount as a `short` — `35 PlayerHeal`'s mana twin. The server
/// forces the id to the sender (`MessageBuffer.cs:2309-2313`); the read side
/// raises `ManaEffect` for a peer (`:2315-2318`, no `ServerSideCharacter`
/// escape here — unlike 35's `:2168`) and relays (`:2319-2322`). The send
/// site is `Player.ManaEffect` (`Player.cs:4531-4538`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManaEffect {
    pub player_id: u8,
    pub amount: i16,
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
    /// `5 PlayerInventory` — one inventory slot. The wire carries the
    /// sender's id first (`NetMessage.cs:201`); the server forces it to the
    /// sender on read (`MessageBuffer.cs:409-412`).
    Inventory {
        player_id: u8,
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
    /// `29` — one of this client's projectiles is gone
    /// (`Projectile.cs:77254-77257`): the key plus the **entry** position
    /// (`NetMessage.cs:852-855` writes `Vector2(number2, number3)`; the
    /// owner-gated `Kill` tail passes the entry snapshot, `Projectile.cs:77256`).
    /// The NaN/NaN pair is the silent-deactivate variant
    /// (`Projectile.cs:16697`, the host's case-95 relay passes 0/0).
    ProjectileDead {
        key: i32,
        x: f32,
        y: f32,
    },
    /// `95 MurderSomeoneElsesPortal` — this client's new portal intersected
    /// a peer's portal projectile (`PortalHelper.cs:364-367`): the peer's
    /// owner id and the portal form (`ai[1]`), so the host can kill the
    /// matching type-602 slot (`MessageBuffer.cs:3519-3537`).
    MurderPortal {
        owner: u16,
        form: u8,
    },
    /// `128 LandGolfBallInCup` — this client's golf ball dropped into a cup
    /// (`GolfHelper.ContactListener.PutBallInCup`, `GolfHelper.cs:107-110`,
    /// `netMode == 1` only); the host relays it with `ignoreClient = owner`
    /// (`MessageBuffer.cs:4058-4074`).
    GolfBallInCup(GolfBallInCup),
    /// `109 MassWireOperation` — this client's Grand Design / Multicolor
    /// Wrench drag (`Projectile.cs:72060`, `Player.cs:46911`); the host runs
    /// the operation for the sender (`MessageBuffer.cs:3774-3788`).
    MassWireOperation(MassWireOperation),
    /// `154 Ping` - this client's RTT probe, no body (`Net/Ping.cs:31`).
    Ping,
    /// `31 RequestChestOpen` - ask the host for a chest's contents
    /// (`Player.cs:32897`/`:33116`).
    ChestOpenRequest(ChestOpenRequest),
    /// `32 SyncChestItem` - one chest slot this client changed
    /// (`Player.cs:31520`, the `1023` violin cases; `NetMessage.cs:1878`'s
    /// chest drops).
    ChestItem(ChestItem),
    /// `34 ChestUpdates` - this client mined/placed a container
    /// (`Player.cs:53785-53796`'s destroy asks, `Chest.cs:577-583`'s place
    /// asks; S1-I-1 wave I).
    ChestUpdates(ChestUpdates),
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
    // (`35 PlayerHeal` / `36 SyncPlayerZone` are the W4-d96 variants below —
    // keep-one dedupe at the g48 merge.)
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
    // ---- S1-I-1 wave B: the world-sync region (bodies + codecs; the
    // dispatcher arms are S1-I-4) ----
    /// `17 TileManipulation` - a client's single-tile edit
    /// (`Player.cs:39626`, `DelegateMethods.cs:515`; host relay
    /// `MessageBuffer.cs:1285`).
    TileEdit(TileEdit),
    /// `19 ToggleDoorState` - a client opened/closed a door/trapdoor/gate
    /// (`Player.cs:32844-32861`, `:32301-32310`; relay
    /// `MessageBuffer.cs:1330`).
    DoorToggle(DoorToggle),
    /// `63 SyncTilePaintOrCoating` - a client painted/coated a tile
    /// (`MessageBuffer.cs:2909-2928` relays it).
    TilePaint(TilePaint),
    /// `64 SyncWallPaintOrCoating` - a client painted/coated a wall
    /// (`MessageBuffer.cs:2929-2948` relays it).
    WallPaint(TilePaint),
    /// `125 SyncTilePicking` - a client's pickaxe hit on a tile
    /// (`MessageBuffer.cs:4023-4042` overrides the id and relays).
    TilePicking(TilePicking),
    /// `157 TeamChangeFromUI` - the team change from the UI
    /// (`Main.cs:42766`); same body as `45`, the reader adds the team-spawn
    /// tail (`MessageBuffer.cs:2355-2363`). The host relays as `45`.
    TeamChangeUi(TeamChange),
    /// `159 RequestSection` - a client asks for a section
    /// (`Player.cs:41534`, `WorldItem.cs:375`); the writer writes `short`s,
    /// the reader reads `ushort`s (`MessageBuffer.cs:4460`).
    SectionRequest {
        section_x: u16,
        section_y: u16,
    },

    /// `35 PlayerHeal` - a client's heal effect (`Player.HealEffect`'s
    /// broadcast, `Player.cs:4522-4529`; `MessageBuffer.cs:2160-2177`).
    PlayerHeal(PlayerHeal),
    /// `40 SyncTalkNPC` - a client started/stopped talking to an NPC
    /// (`Player.SetTalkNPC`, `Player.cs:4375`; `MessageBuffer.cs:2256-2270`).
    TalkNpc(TalkNpc),
    /// `36 SyncPlayerZone` - a client's zone flags changed
    /// (`Main.cs:17634`'s change test, the 900-tick heartbeat `:64533`;
    /// `MessageBuffer.cs:2178-2201`).
    PlayerZone(PlayerZone),
    /// `41 ItemRotationAndAnimation` - a client's aimed-item rotation
    /// (`Player.cs:43693`/`:47374`/`:47380` beside a `13`;
    /// `MessageBuffer.cs:2271-2289`).
    ItemAnim(ItemAnim),
    /// `43 ManaEffect` - a client's mana effect (`Player.ManaEffect`'s
    /// broadcast, `Player.cs:4531-4538`; `MessageBuffer.cs:2307-2324`).
    ManaEffect(ManaEffect),
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
    /// `12 PlayerSpawn` — a peer spawned in (`NetMessage.cs:433-445`; the
    /// host relays it once the slot is in-world, `MessageBuffer.cs:936`/`:948`).
    PlayerSpawn(PlayerSpawn),
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
    /// `34 ChestUpdates` - the host's place/destroy result, relayed
    /// (`MessageBuffer.cs:2006-2112`'s `TrySendData(34, …)` arms;
    /// S1-I-1 wave I).
    ChestUpdates(ChestUpdates),
    /// `81 CombatTextInt` - a peer's heal number (`NPC.HealEffect`'s
    /// broadcast, `NPC.cs:77442`; S1-I-1 wave I).
    CombatTextInt(CombatTextInt),
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
    // (`35 PlayerHeal` / `36 SyncPlayerZone` are the W4-d96 variants below —
    // keep-one dedupe at the g48 merge.)
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
    /// `128 LandGolfBallInCup` - a peer's golf ball landed in a cup, relayed
    /// to everyone but the owner (`MessageBuffer.cs:4067`); the client runs
    /// the cup text/effects (`:4071`, M4-GOLF-1).
    GolfBallInCup(GolfBallInCup),
    /// `110 MassWireOperationPay` - the host's wire/actuator bill for a mass
    /// operation, sent to the billed player only (`Wiring.cs:439-443`); the
    /// client consumes the items when the bill is its own
    /// (`MessageBuffer.cs:3789-3807`).
    MassWireOperationPay(MassWireOperationPay),
    /// `147 SyncLoadout` - a peer's loadout (join push
    /// `NetMessage.cs:2908`, relay `MessageBuffer.cs:4340`).
    SyncLoadout(SyncLoadout),
    /// `35 PlayerHeal` - a peer's heal effect, relayed by the host
    /// (`MessageBuffer.cs:2172-2175`).
    PlayerHeal(PlayerHeal),
    /// `40 SyncTalkNPC` - a peer's talk NPC, relayed by the host
    /// (`MessageBuffer.cs:2265-2268`).
    TalkNpc(TalkNpc),
    /// `36 SyncPlayerZone` - a peer's zone flags, relayed by the host
    /// (`MessageBuffer.cs:2199`).
    PlayerZone(PlayerZone),
    /// `41 ItemRotationAndAnimation` - a peer's aimed-item rotation,
    /// relayed by the host (`MessageBuffer.cs:2284-2287`).
    ItemAnim(ItemAnim),
    /// `43 ManaEffect` - a peer's mana effect, relayed by the host
    /// (`MessageBuffer.cs:2319-2322`).
    ManaEffect(ManaEffect),
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
    /// `11 TileFrameSection` — the deprecated re-frame request
    /// (`NetMessage.cs:427-432`; the client runs `WorldGen.SectionTileFrame`,
    /// `MessageBuffer.cs:895-899`). No C# site sends it; codec kept for the
    /// id-table census (S1-I-1 wave B).
    TileFrameSection(TileFrameSection),
    /// `15 Unknown15` — deprecated (`MessageID.cs:38-39`), no body; the
    /// reader's ignore group (`MessageBuffer.cs:4483-4490`).
    Unknown15,
    /// `20 SectionRange` — a rectangle of tile changes
    /// (`SendTileSquare`, `NetMessage.cs:2627-2630`).
    SectionRange(Box<SectionRange>),
    // ---- S1-I-1 wave B: the world-sync region (bodies + codecs; the
    // dispatcher arms are S1-I-4) ----
    // `11 TileFrameSection` is the W4-d89 variant above (keep-one dedupe).
    /// `17 TileManipulation` — the host's relay of a client's edit
    /// (`MessageBuffer.cs:1285`).
    TileEdit(TileEdit),
    /// `19 ToggleDoorState` — the host's door/trapdoor/gate relay
    /// (`MessageBuffer.cs:1330`).
    DoorToggle(DoorToggle),
    /// `63 SyncTilePaintOrCoating` — the host's paint relay
    /// (`MessageBuffer.cs:2925`).
    TilePaint(TilePaint),
    /// `64 SyncWallPaintOrCoating` — the host's paint relay
    /// (`MessageBuffer.cs:2945`).
    WallPaint(TilePaint),
    /// `125 SyncTilePicking` — another player's pickaxe hit
    /// (`MessageBuffer.cs:4039`).
    TilePicking(TilePicking),
    /// `158 ExtraSpawnSectionLoaded` — the team-swap respawn: the extra
    /// spawn section is in, `Spawn(TeamSwap)` (`MessageBuffer.cs:4450-4456`,
    /// sent by the `157` tail `:2361`).
    ExtraSpawnLoaded {
        player_id: u8,
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
            Self::NetModule { .. } => id::NET_MODULE,
            Self::NpcStrike(_) => id::NPC_STRIKE,
            Self::CatchNpc { .. } => id::CATCH_NPC,
            Self::ReleaseNpc { .. } => id::RELEASE_NPC,
            Self::Projectile(_) => id::PROJECTILE,
            Self::ProjectileDead { .. } => id::PROJECTILE_DEAD,
            Self::MurderPortal { .. } => id::MURDER_PORTAL,
            Self::GolfBallInCup(_) => id::GOLF_BALL_IN_CUP,
            Self::MassWireOperation(_) => id::MASS_WIRE_OPERATION,
            Self::Ping => id::PING,
            Self::ChestOpenRequest(_) => id::CHEST_OPEN_REQUEST,
            Self::ChestItem(_) => id::CHEST_ITEM,
            Self::ChestUpdates(_) => id::CHEST_UPDATES,
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
            Self::TileEdit(_) => id::TILE_EDIT,
            Self::DoorToggle(_) => id::DOOR_TOGGLE,
            Self::TilePaint(_) => id::TILE_PAINT,
            Self::WallPaint(_) => id::WALL_PAINT,
            Self::TilePicking(_) => id::TILE_PICKING,
            Self::TeamChangeUi(_) => id::TEAM_CHANGE_UI,
            Self::SectionRequest { .. } => id::SECTION_REQUEST,

            Self::PlayerHeal(_) => id::PLAYER_HEAL,
            Self::TalkNpc(_) => id::TALK_NPC,
            Self::PlayerZone(_) => id::PLAYER_ZONE,
            Self::ItemAnim(_) => id::ITEM_ANIM,
            Self::ManaEffect(_) => id::MANA_EFFECT,
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
            // `NetMessage.cs:199-222`: (byte)number leads the slot body.
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
            Self::WorldRequest => w,
            Self::Ping => w,
            Self::ChestOpenRequest(c) => w.i16(c.x).i16(c.y),
            Self::ChestItem(c) => w
                .i16(c.chest)
                .u8(c.slot)
                .i16(c.stack)
                .u8(c.prefix)
                .i16(c.kind),
            // `NetMessage.cs:913-927` — the client branch writes the literal
            // `0` short (`:926`); the host's `number5` relay rides S2c.
            Self::ChestUpdates(c) => w.u8(c.action).i16(c.x).i16(c.y).i16(c.style).i16(0),
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
            // (35/36 encode at the d96 arms below — keep-one at the g48 merge.)
            Self::PlayerMana(m) => w.u8(m.player_id).i16(m.mana).i16(m.max_mana),
            Self::TeamChange(t) => w.u8(t.player_id).u8(t.team),
            Self::ProjectileTrackers(t) => w
                .u8(t.player_id)
                .i32(t.piggy_bank.key)
                .i16(t.piggy_bank.kind)
                .i32(t.void_lens.key)
                .i16(t.void_lens.kind),
            Self::SyncLoadout(l) => w.u8(l.player_id).u8(l.loadout).u16(l.hide_accessory),
            // ---- S1-I-1 wave B ----
            // `NetMessage.cs:524-530`: byte kind + short x + short y +
            // short value + byte style.
            Self::TileEdit(t) => w.u8(t.kind).i16(t.x).i16(t.y).i16(t.value).u8(t.style),
            // `NetMessage.cs:537-542`: byte action + short x + short y +
            // byte direction (the C# writes `number4 == 1f`).
            Self::DoorToggle(d) => w.u8(d.action).i16(d.x).i16(d.y).u8(d.direction),
            // `NetMessage.cs:1081-1087`: short x + short y + byte paint +
            // byte coat.
            Self::TilePaint(p) | Self::WallPaint(p) => {
                w.i16(p.x).i16(p.y).u8(p.paint).u8(p.coat)
            }
            // `NetMessage.cs:1509-1514`: byte player + short x + short y +
            // byte damage.
            Self::TilePicking(p) => w.u8(p.player_id).i16(p.x).i16(p.y).u8(p.damage),
            // `NetMessage.cs:970-974` — 157 shares 45's case.
            Self::TeamChangeUi(t) => w.u8(t.player_id).u8(t.team),
            // `NetMessage.cs:1662-1665`: two `(short)`s (the reader reads
            // them as ushorts, `MessageBuffer.cs:4460` — same bytes).
            Self::SectionRequest {
                section_x,
                section_y,
            } => w.u16(*section_x).u16(*section_y),

            // `NetMessage.cs:929-932`: (byte)number + (short)number2.
            Self::PlayerHeal(h) => w.u8(h.player_id).i16(h.amount),
            // `NetMessage.cs:952-955`: (byte)number + (short)talkNPC.
            Self::TalkNpc(t) => w.u8(t.player_id).i16(t.npc),
            // `NetMessage.cs:933-944`: (byte)number + zone1..5 + (byte)townNPCs.
            Self::PlayerZone(z) => w
                .u8(z.player_id)
                .bitsbyte(z.zone1)
                .bitsbyte(z.zone2)
                .bitsbyte(z.zone3)
                .bitsbyte(z.zone4)
                .bitsbyte(z.zone5)
                .u8(z.town_npcs),
            // `NetMessage.cs:956-960`: (byte)number + itemRotation + (short)itemAnimation.
            Self::ItemAnim(a) => w.u8(a.player_id).f32(a.rotation).i16(a.animation),
            // `NetMessage.cs:966-969`: (byte)number + (short)number2.
            Self::ManaEffect(m) => w.u8(m.player_id).i16(m.amount),
            Self::SpawnRequest(s) => w.i32(s.x).i32(s.y).u8(s.team),
            // `NetMessage.cs:433-445`: (byte)number leads the spawn body.
            Self::PlayerSpawn(s) => w
                .u8(s.player_id)
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
            // `NetMessage.cs:852-855` — `number` (the key) +
            // `Vector2(number2, number3)`; the position rides verbatim.
            Self::ProjectileDead { key, x, y } => {
                return NetWriter::open(id::PROJECTILE_DEAD)
                    .i32(*key)
                    .f32(*x)
                    .f32(*y)
                    .finish();
            }
            // `NetMessage.cs:1333-1336` — `(ushort)number + (byte)number2`.
            Self::MurderPortal { owner, form } => w.u16(*owner).u8(*form),
            // `NetMessage.cs:1521-1527` — owner byte, then the four ushorts
            // in the WRITER's order (x, y, hits, type — not the arg order).
            Self::GolfBallInCup(g) => w.u8(g.owner).u16(g.x).u16(g.y).u16(g.hits).u16(g.ball_type),
            // `NetMessage.cs:1412-1418` — four (short)s, then the (byte) mode.
            Self::MassWireOperation(m) => w
                .i16(m.start_x)
                .i16(m.start_y)
                .i16(m.end_x)
                .i16(m.end_y)
                .u8(m.tool_mode),
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
                // `MessageBuffer.cs:401-406`: the id byte leads the body.
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
                        // The C# clamps the stack to ≥ 0 (`NetMessage.cs:210-213`).
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
            // `MessageBuffer.cs:1986-1994` — the host reads the fifth short
            // then zeroes it; the client's send wrote 0 anyway.
            id::CHEST_UPDATES => Self::ChestUpdates(ChestUpdates {
                action: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                style: r.i16()?,
                chest: {
                    let _ = r.i16()?;
                    0
                },
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
            // (35/36 decode at the d96 arms below — keep-one at the g48 merge.)
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
            // ---- S1-I-1 wave B ----
            // `MessageBuffer.cs:1121-1125`: byte, short, short, short, byte.
            id::TILE_EDIT => Self::TileEdit(TileEdit {
                kind: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                value: r.i16()?,
                style: r.u8()?,
            }),
            // `MessageBuffer.cs:1301-1306`: byte, short, short, byte.
            id::DOOR_TOGGLE => Self::DoorToggle(DoorToggle {
                action: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                direction: r.u8()?,
            }),
            // `MessageBuffer.cs:2911-2914` / `:2931-2934`: short, short,
            // byte, byte.
            id::TILE_PAINT | id::WALL_PAINT => {
                let p = TilePaint {
                    x: r.i16()?,
                    y: r.i16()?,
                    paint: r.u8()?,
                    coat: r.u8()?,
                };
                if msg_id == id::TILE_PAINT {
                    Self::TilePaint(p)
                } else {
                    Self::WallPaint(p)
                }
            }
            // `MessageBuffer.cs:4025-4028`: byte, short, short, byte.
            id::TILE_PICKING => Self::TilePicking(TilePicking {
                player_id: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                damage: r.u8()?,
            }),
            // `MessageBuffer.cs:2325-2333` — 157 shares 45's reader arm.
            id::TEAM_CHANGE_UI => Self::TeamChangeUi(TeamChange {
                player_id: r.u8()?,
                team: r.u8()?,
            }),
            // `MessageBuffer.cs:4460`: two ReadUInt16 (the writer wrote
            // shorts — the non-negative section indices are the same bytes).
            id::SECTION_REQUEST => Self::SectionRequest {
                section_x: r.u16()?,
                section_y: r.u16()?,
            },

            // `MessageBuffer.cs:2162-2167`: (byte)id + (short)amount.
            id::PLAYER_HEAL => Self::PlayerHeal(PlayerHeal {
                player_id: r.u8()?,
                amount: r.i16()?,
            }),
            // `MessageBuffer.cs:2258-2263`: (byte)id + (short)talkNPC.
            id::TALK_NPC => Self::TalkNpc(TalkNpc {
                player_id: r.u8()?,
                npc: r.i16()?,
            }),
            // `MessageBuffer.cs:2180-2192`: (byte)id + zone1..5 + townNPCs.
            id::PLAYER_ZONE => Self::PlayerZone(PlayerZone {
                player_id: r.u8()?,
                zone1: r.bitsbyte()?,
                zone2: r.bitsbyte()?,
                zone3: r.bitsbyte()?,
                zone4: r.bitsbyte()?,
                zone5: r.bitsbyte()?,
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
            id::SPAWN_REQUEST => Self::SpawnRequest(SpawnRequest {
                x: r.i32()?,
                y: r.i32()?,
                team: r.u8()?,
            }),
            // `MessageBuffer.cs:903-919`: the id byte leads the body.
            id::PLAYER_SPAWN => Self::PlayerSpawn(PlayerSpawn {
                player_id: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                respawn_timer: r.i32()?,
                deaths_pve: r.i16()?,
                deaths_pvp: r.i16()?,
                team: r.u8()?,
                spawn_context: r.u8()?,
            }),
            // `MessageBuffer.cs:1063-1081`: the active byte is tested `== 1`
            // (`:1070`), anything else deactivates.
            id::PLAYER_ACTIVE => Self::PlayerActive {
                player_id: r.u8()?,
                active: r.u8()? == 1,
            },
            // `MessageBuffer.cs:1097-1111`: (byte)id + (short)statLife +
            // (short)statLifeMax; the read side floors statLifeMax at 20
            // (`:1107-1110`) and derives `dead = statLife <= 0` (`:1111`,
            // the apply arm's job — S1-I-4).
            id::PLAYER_HP => Self::PlayerHp {
                player_id: r.u8()?,
                hp: r.i16()?,
                max_hp: r.i16()?.max(20),
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
            // `MessageBuffer.cs:1827-1828` — the key, then the Vector2.
            id::PROJECTILE_DEAD => Self::ProjectileDead {
                key: r.i32()?,
                x: r.f32()?,
                y: r.f32()?,
            },
            // `MessageBuffer.cs:3521-3522` — (ushort) owner + (byte) form.
            id::MURDER_PORTAL => Self::MurderPortal {
                owner: r.u16()?,
                form: r.u8()?,
            },
            // `MessageBuffer.cs:4060-4064` — owner, x, y, hits, type.
            id::GOLF_BALL_IN_CUP => Self::GolfBallInCup(GolfBallInCup {
                owner: r.u8()?,
                x: r.u16()?,
                y: r.u16()?,
                hits: r.u16()?,
                ball_type: r.u16()?,
            }),
            // `MessageBuffer.cs:3777-3781` — the four shorts + the mode byte.
            id::MASS_WIRE_OPERATION => Self::MassWireOperation(MassWireOperation {
                start_x: r.i16()?,
                start_y: r.i16()?,
                end_x: r.i16()?,
                end_y: r.i16()?,
                tool_mode: r.u8()?,
            }),
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
            Self::TileFrameSection(_) => id::TILE_FRAME_SECTION,
            Self::Unknown15 => id::UNKNOWN_15,
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
            Self::PlayerSpawn(_) => id::PLAYER_SPAWN,
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
            Self::ChestUpdates(_) => id::CHEST_UPDATES,
            Self::CombatTextInt(_) => id::COMBAT_TEXT_INT,
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
            Self::GolfBallInCup(_) => id::GOLF_BALL_IN_CUP,
            Self::MassWireOperationPay(_) => id::MASS_WIRE_OPERATION_PAY,
            Self::SyncLoadout(_) => id::SYNC_LOADOUT,
            Self::TileEdit(_) => id::TILE_EDIT,
            Self::DoorToggle(_) => id::DOOR_TOGGLE,
            Self::TilePaint(_) => id::TILE_PAINT,
            Self::WallPaint(_) => id::WALL_PAINT,
            Self::TilePicking(_) => id::TILE_PICKING,
            Self::ExtraSpawnLoaded { .. } => id::EXTRA_SPAWN_LOADED,

            Self::PlayerHeal(_) => id::PLAYER_HEAL,
            Self::TalkNpc(_) => id::TALK_NPC,
            Self::PlayerZone(_) => id::PLAYER_ZONE,
            Self::ItemAnim(_) => id::ITEM_ANIM,
            Self::ManaEffect(_) => id::MANA_EFFECT,
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
            // `NetMessage.cs:427-432` — four (short)s, verbatim order.
            Self::TileFrameSection(t) => w.i16(t.start_x).i16(t.start_y).i16(t.end_x).i16(t.end_y),
            Self::Unknown15 => w,
            Self::InitialSpawn => w,
            Self::Ping => w,
            Self::NpcKilled { npc } => w.i16(*npc),
            Self::ProgressionEvent { event_id } => w.i16(*event_id),
            Self::PlayerActive { player_id, active } => w.u8(*player_id).u8(u8::from(*active)),
            // `NetMessage.cs:433-445`: (byte)number leads the spawn body.
            Self::PlayerSpawn(s) => w
                .u8(s.player_id)
                .i16(s.x)
                .i16(s.y)
                .i32(s.respawn_timer)
                .i16(s.deaths_pve)
                .i16(s.deaths_pvp)
                .u8(s.team)
                .u8(s.spawn_context),
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
            // `NetMessage.cs:913-923` — the server branch writes `number5`.
            Self::ChestUpdates(c) => w
                .u8(c.action)
                .i16(c.x)
                .i16(c.y)
                .i16(c.style)
                .i16(c.chest),
            // `NetMessage.cs:1179-1186` — two floats, RGB, (int)amount.
            Self::CombatTextInt(t) => w.f32(t.x).f32(t.y).rgb(t.color).i32(t.amount),
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
            // (35/36 encode at the d96 arms below — keep-one at the g48 merge.)
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
            // `NetMessage.cs:1521-1527` — the relay writes the same case-128
            // arm (`MessageBuffer.cs:4067` re-sends with number5/6 = x/y).
            Self::GolfBallInCup(g) => w.u8(g.owner).u16(g.x).u16(g.y).u16(g.hits).u16(g.ball_type),
            // `NetMessage.cs:1419-1423` — (short)type, (short)count, (byte)plr.
            Self::MassWireOperationPay(p) => w.i16(p.item_type).i16(p.count).u8(p.player),
            Self::SyncLoadout(l) => w.u8(l.player_id).u8(l.loadout).u16(l.hide_accessory),
            // ---- S1-I-1 wave B ----
            // (`11 TileFrameSection` encodes at the d89 arm above — keep-one.)
            // `NetMessage.cs:524-530`.
            Self::TileEdit(t) => w.u8(t.kind).i16(t.x).i16(t.y).i16(t.value).u8(t.style),
            // `NetMessage.cs:537-542`.
            Self::DoorToggle(d) => w.u8(d.action).i16(d.x).i16(d.y).u8(d.direction),
            // `NetMessage.cs:1081-1087`.
            Self::TilePaint(p) | Self::WallPaint(p) => {
                w.i16(p.x).i16(p.y).u8(p.paint).u8(p.coat)
            }
            // `NetMessage.cs:1509-1514`.
            Self::TilePicking(p) => w.u8(p.player_id).i16(p.x).i16(p.y).u8(p.damage),
            // `NetMessage.cs:1659-1661`: one byte.
            Self::ExtraSpawnLoaded { player_id } => w.u8(*player_id),

            // `NetMessage.cs:929-932`: (byte)number + (short)number2.
            Self::PlayerHeal(h) => w.u8(h.player_id).i16(h.amount),
            // `NetMessage.cs:952-955`: (byte)number + (short)talkNPC.
            Self::TalkNpc(t) => w.u8(t.player_id).i16(t.npc),
            // `NetMessage.cs:933-944`: (byte)number + zone1..5 + (byte)townNPCs.
            Self::PlayerZone(z) => w
                .u8(z.player_id)
                .bitsbyte(z.zone1)
                .bitsbyte(z.zone2)
                .bitsbyte(z.zone3)
                .bitsbyte(z.zone4)
                .bitsbyte(z.zone5)
                .u8(z.town_npcs),
            // `NetMessage.cs:956-960`: (byte)number + itemRotation + (short)itemAnimation.
            Self::ItemAnim(a) => w.u8(a.player_id).f32(a.rotation).i16(a.animation),
            // `NetMessage.cs:966-969`: (byte)number + (short)number2.
            Self::ManaEffect(m) => w.u8(m.player_id).i16(m.amount),
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
            // `MessageBuffer.cs:895-899` — four (short)s into SectionTileFrame.
            id::TILE_FRAME_SECTION => Self::TileFrameSection(TileFrameSection {
                start_x: r.i16()?,
                start_y: r.i16()?,
                end_x: r.i16()?,
                end_y: r.i16()?,
            }),
            id::UNKNOWN_15 => Self::Unknown15,
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
            // `MessageBuffer.cs:1063-1081`: the active byte is tested `== 1`
            // (`:1070`), anything else deactivates.
            id::PLAYER_ACTIVE => Self::PlayerActive {
                player_id: r.u8()?,
                active: r.u8()? == 1,
            },
            // `MessageBuffer.cs:903-919`: the id byte leads the body.
            id::PLAYER_SPAWN => Self::PlayerSpawn(PlayerSpawn {
                player_id: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                respawn_timer: r.i32()?,
                deaths_pve: r.i16()?,
                deaths_pvp: r.i16()?,
                team: r.u8()?,
                spawn_context: r.u8()?,
            }),
            // `MessageBuffer.cs:1097-1111`: (byte)id + (short)statLife +
            // (short)statLifeMax; the read side floors statLifeMax at 20
            // (`:1107-1110`) and derives `dead = statLife <= 0` (`:1111`,
            // the apply arm's job — S1-I-4).
            id::PLAYER_HP => Self::PlayerHp {
                player_id: r.u8()?,
                hp: r.i16()?,
                max_hp: r.i16()?.max(20),
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
            // `MessageBuffer.cs:1986-1990` — the client keeps the fifth
            // short (the host's `num35` result index).
            id::CHEST_UPDATES => Self::ChestUpdates(ChestUpdates {
                action: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                style: r.i16()?,
                chest: r.i16()?,
            }),
            // `MessageBuffer.cs:3227-3233` — float pair, RGB, int amount.
            id::COMBAT_TEXT_INT => Self::CombatTextInt(CombatTextInt {
                x: r.f32()?,
                y: r.f32()?,
                color: r.rgb()?,
                amount: r.i32()?,
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
            // (35/36 decode at the d96 arms below — keep-one at the g48 merge.)
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
            // ---- S1-I-1 wave B ----
            // (`11 TILE_FRAME_SECTION` decodes at the d89 arm above — keep-one.)
            id::TILE_EDIT => Self::TileEdit(TileEdit {
                kind: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                value: r.i16()?,
                style: r.u8()?,
            }),
            id::DOOR_TOGGLE => Self::DoorToggle(DoorToggle {
                action: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                direction: r.u8()?,
            }),
            id::TILE_PAINT | id::WALL_PAINT => {
                let p = TilePaint {
                    x: r.i16()?,
                    y: r.i16()?,
                    paint: r.u8()?,
                    coat: r.u8()?,
                };
                if msg_id == id::TILE_PAINT {
                    Self::TilePaint(p)
                } else {
                    Self::WallPaint(p)
                }
            }
            id::TILE_PICKING => Self::TilePicking(TilePicking {
                player_id: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
                damage: r.u8()?,
            }),
            // `MessageBuffer.cs:4453`: one byte, then Spawn(TeamSwap).
            id::EXTRA_SPAWN_LOADED => Self::ExtraSpawnLoaded {
                player_id: r.u8()?,
            },

            // `MessageBuffer.cs:4060-4064` — owner, x, y, hits, type.
            id::GOLF_BALL_IN_CUP => Self::GolfBallInCup(GolfBallInCup {
                owner: r.u8()?,
                x: r.u16()?,
                y: r.u16()?,
                hits: r.u16()?,
                ball_type: r.u16()?,
            }),
            // `MessageBuffer.cs:3795-3797` — type, count, player.
            id::MASS_WIRE_OPERATION_PAY => Self::MassWireOperationPay(MassWireOperationPay {
                item_type: r.i16()?,
                count: r.i16()?,
                player: r.u8()?,
            }),

            // `MessageBuffer.cs:2162-2167`: (byte)id + (short)amount.
            id::PLAYER_HEAL => Self::PlayerHeal(PlayerHeal {
                player_id: r.u8()?,
                amount: r.i16()?,
            }),
            // `MessageBuffer.cs:2258-2263`: (byte)id + (short)talkNPC.
            id::TALK_NPC => Self::TalkNpc(TalkNpc {
                player_id: r.u8()?,
                npc: r.i16()?,
            }),
            // `MessageBuffer.cs:2180-2192`: (byte)id + zone1..5 + townNPCs.
            id::PLAYER_ZONE => Self::PlayerZone(PlayerZone {
                player_id: r.u8()?,
                zone1: r.bitsbyte()?,
                zone2: r.bitsbyte()?,
                zone3: r.bitsbyte()?,
                zone4: r.bitsbyte()?,
                zone5: r.bitsbyte()?,
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

    /// The `case 4` read-side clamps (`MessageBuffer.cs:297-312`), verbatim:
    /// skinVariant → 0..11, voiceVariant → 1..4, NaN pitch → 0 then ±1,
    /// hair ≥ 228 → 0, name trimmed.
    #[test]
    fn w4_d96_player_info_read_clamps() {
        let mut pi = PlayerInfo::with_name(3, "  Rusty  ");
        pi.skin_variant = 200;
        pi.voice_variant = 9;
        pi.voice_pitch_offset = 2.5;
        pi.hair = 250;
        let body = pi.write_to(NetWriter::open(id::PLAYER_INFO)).finish();
        let back = PlayerInfo::read_from(&mut NetReader::new(&body[3..])).expect("decode");
        assert_eq!(back.skin_variant, 11); // PlayerVariantID.Count - 1
        assert_eq!(back.voice_variant, 4);
        assert_eq!(back.voice_pitch_offset, 1.0);
        assert_eq!(back.hair, 0);
        assert_eq!(back.name, "Rusty");
        // voiceVariant 0 clamps UP to 1 (`Utils.Clamp(voiceVariant, 1, 4)`).
        let zero = PlayerInfo::with_name(3, "Ann");
        let body = zero.write_to(NetWriter::open(id::PLAYER_INFO)).finish();
        let back = PlayerInfo::read_from(&mut NetReader::new(&body[3..])).expect("decode");
        assert_eq!(back.voice_variant, 1);
        // A NaN pitch normalises to 0 before the clamp (`:302-305`).
        let mut nan = PlayerInfo::with_name(3, "Ann");
        nan.voice_pitch_offset = f32::NAN;
        let body = nan.write_to(NetWriter::open(id::PLAYER_INFO)).finish();
        let back = PlayerInfo::read_from(&mut NetReader::new(&body[3..])).expect("decode");
        assert_eq!(back.voice_pitch_offset, 0.0);
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
                player_id: 5,
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
                player_id: 5,
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
                x: 12.5,
                y: -8.0,
            },
            C2s::MurderPortal { owner: 2, form: 1 },
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
            // S1-I-1 wave I: the client's 34 destroy ask (chest 0 on the
            // wire — the writer's else branch, `NetMessage.cs:926`).
            C2s::ChestUpdates(ChestUpdates {
                action: 1,
                x: 100,
                y: 200,
                style: 0,
                chest: 0,
            }),
            // S1-I-1 wave I census remainder: the 35/36 bodies.
            C2s::PlayerHeal(PlayerHeal {
                player_id: 2,
                amount: 20,
            }),
            C2s::PlayerZone(PlayerZone {
                player_id: 2,
                zone1: BitsByte(1),
                zone2: BitsByte(2),
                zone3: BitsByte(4),
                zone4: BitsByte(8),
                zone5: BitsByte(16),
                town_npcs: 3,
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
            // max_hp >= 20: the read side floors it (`MessageBuffer.cs:1107-1110`).
            S2c::PlayerHp {
                player_id: 1,
                hp: 5,
                max_hp: 100,
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
                mount_type: 0,
                potion_use: (0.0, 0.0),
                potion_home: (0.0, 0.0),
                camera_target: (0.0, 0.0),
                on_ground: true,
                leg_frame: 2,
                body_frame: 7,
                hp: 80,
                max_hp: 100,
            })),
            // All three conditional tails gated on (`NetMessage.cs:496-508`):
            // mount u16, potion use+home Vector2s, camera Vector2.
            S2c::PlayerState(Box::new(PlayerState {
                player_id: 2,
                flags_a: BitsByte::new(),
                flags_b: BitsByte(0b1000_0100),
                flags_c: BitsByte(0b0100_0000),
                flags_d: BitsByte(0b0010_0000),
                selected_item: 0,
                x: 0.0,
                y: 0.0,
                vx: 2.0,
                vy: 3.0,
                mount_type: 8,
                potion_use: (100.0, 200.0),
                potion_home: (300.0, 400.0),
                camera_target: (500.0, 600.0),
                on_ground: false,
                leg_frame: 0,
                body_frame: 0,
                hp: 100,
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
                target: 255,
                dir_right: true,
                dir_y_down: false,
                sprite_right: true,
                life_full: false,
                hp: 40,
                life_max: 2000,
                ai: [0.0; 4],
                spawned_from_statue: false,
                difficulty: 1.0,
                shimmer: false,
                release_owner: 255,
            })),
            // The life width switch: the > 127 and > 32767 arms. Kind 50 =
            // TheDestroyer (max_hp 80000): decode re-derives life_max from the
            // kind, so the width-4 arm survives the re-encode byte-equality.
            S2c::NpcInfo(Box::new(NpcInfo {
                slot: 4,
                generation: 1,
                spawn: true,
                kind: 50,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                target: 0,
                dir_right: false,
                dir_y_down: true,
                sprite_right: false,
                life_full: false,
                hp: 9000,
                life_max: 80000,
                ai: [0.0; 4],
                spawned_from_statue: false,
                difficulty: 1.0,
                shimmer: false,
                release_owner: 255,
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
                enemy_grab_delay: 0,
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
                enemy_grab_delay: 0,
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
            // S1-I-1 wave I: the 34/81 bodies.
            S2c::ChestUpdates(ChestUpdates {
                action: 4,
                x: 100,
                y: 200,
                style: 2,
                chest: -1,
            }),
            S2c::CombatTextInt(CombatTextInt {
                x: 64.5,
                y: -8.25,
                color: Rgb(100, 255, 100),
                amount: 20,
            }),
            // S1-I-1 wave I census remainder: the 35/36 host relays.
            S2c::PlayerHeal(PlayerHeal {
                player_id: 1,
                amount: -5,
            }),
            S2c::PlayerZone(PlayerZone {
                player_id: 1,
                zone1: BitsByte(255),
                zone2: BitsByte(0),
                zone3: BitsByte(128),
                zone4: BitsByte(1),
                zone5: BitsByte(0),
                town_npcs: 255,
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
            player_id: 0,
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

    // ---- S1-I-1 NPC family, first half (d91): 23/24/27/28/29 ----

    #[test]
    fn s1_i_1_npc_info_verbatim_body() {
        // `NetMessage.cs:686-763` — slot, generation, position, velocity,
        // (ushort)target, flags1, flags2, the flag-gated ai floats, the
        // (short)netID, the width-switched life, the catchable releaseOwner.
        // Kind 0 = Slime: max_hp 25 → width byte 1; not catchable (vanilla 1).
        let n = NpcInfo {
            slot: 3,
            generation: 2,
            spawn: false,
            kind: 0,
            x: 64.0,
            y: 96.0,
            vx: 1.5,
            vy: -2.0,
            target: 255,
            dir_right: true,
            dir_y_down: false,
            sprite_right: true,
            life_full: false,
            hp: 20,
            life_max: 25,
            ai: [0.0, 2.0, 0.0, -1.0],
            spawned_from_statue: false,
            difficulty: 1.0,
            shimmer: false,
            release_owner: 255,
        };
        let f = S2c::NpcInfo(Box::new(n)).encode();
        assert_eq!(f[2], 23);
        let mut expect: Vec<u8> = vec![3, 2];
        expect.extend_from_slice(&64.0f32.to_le_bytes());
        expect.extend_from_slice(&96.0f32.to_le_bytes());
        expect.extend_from_slice(&1.5f32.to_le_bytes());
        expect.extend_from_slice(&(-2.0f32).to_le_bytes());
        expect.extend_from_slice(&255u16.to_le_bytes());
        // flags1: b0 dir + b3 ai[1] + b5 ai[3] + b6 sprite = 0x69.
        expect.push(0x69);
        expect.push(0x00); // flags2: all clear
        expect.extend_from_slice(&2.0f32.to_le_bytes());
        expect.extend_from_slice(&(-1.0f32).to_le_bytes());
        expect.extend_from_slice(&0i16.to_le_bytes());
        expect.push(1); // lifeMax 25 ≤ 127 → sbyte width
        expect.push(20);
        assert_eq!(&f[3..], &expect[..]);
        let back = S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode");
        assert_eq!(back.encode(), f);
        // Full health omits the life block entirely (`:734`).
        let full = NpcInfo { life_full: true, hp: 25, ..n };
        let ff = S2c::NpcInfo(Box::new(full)).encode();
        assert_eq!(ff.len(), f.len() - 2);
        assert_eq!(ff[3 + 20], 0xE9); // flags1 b7 set (body: 2+16+2 header)
    }

    #[test]
    fn s1_i_1_npc_info_width_switch_and_optional_fields() {
        // The width byte keys on lifeMax, NOT life (`NetMessage.cs:737-744`):
        // a 250-max Guide (protocol 5) at hp 200 rides width 2 + a short.
        let guide = NpcInfo {
            slot: 0,
            generation: 1,
            spawn: true,
            kind: 5,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            target: 0,
            dir_right: false,
            dir_y_down: false,
            sprite_right: false,
            life_full: false,
            hp: 200,
            life_max: 250,
            ai: [0.0; 4],
            spawned_from_statue: true,
            difficulty: 2.0,
            shimmer: true,
            release_owner: 255,
        };
        let f = S2c::NpcInfo(Box::new(guide)).encode();
        // flags2: b1 statue + b2 difficulty + b3 spawn + b4 shimmer = 0x1E.
        assert_eq!(f[3 + 21], 0x1E);
        let tail = &f[3 + 22..]; // after flags2: no ai, then netID
        assert_eq!(&tail[..2], &5i16.to_le_bytes());
        assert_eq!(&tail[2..6], &2.0f32.to_le_bytes()); // difficulty
        assert_eq!(tail[6], 2); // lifeMax 250 > 127 → short width
        assert_eq!(&tail[7..9], &200i16.to_le_bytes());
        assert_eq!(tail.len(), 9); // Guide is not catchable
        // > 32767 → width 4 + int (`NetMessage.cs:751-753`).
        let big = NpcInfo { kind: -1, hp: 40000, life_max: 40000, ..guide };
        let bf = S2c::NpcInfo(Box::new(big)).encode();
        let bt = &bf[3 + 22..];
        assert_eq!(bt[6], 4);
        assert_eq!(&bt[7..11], &40000i32.to_le_bytes());
        // The catchable trailing byte: protocol 22 = Goldfish, vanilla 55
        // (`Main.npcCatchable[55]`, `Main.cs:9331`).
        let fish = NpcInfo { kind: 22, hp: 3, life_max: 5, release_owner: 4, ..guide };
        let ff = S2c::NpcInfo(Box::new(fish)).encode();
        assert_eq!(*ff.last().unwrap(), 4);
        let back = S2c::decode(ff[2], &mut NetReader::new(&ff[3..])).expect("decode");
        assert_eq!(back.encode(), ff);
        // The player-count byte (flags2 b0) is consumed on decode
        // (`MessageBuffer.cs:1589-1593`) even though the port never sends it.
        let mut body = f[3..].to_vec();
        body[21] |= 0x01; // set flags2 b0
        body.insert(24, 3); // the byte rides after the netID, before difficulty
        let mut r = NetReader::new(&body);
        let d = NpcInfo::read_from(&mut r).expect("decode with player count");
        assert_eq!(d.kind, 5);
        assert_eq!(d.difficulty, 2.0);
        assert_eq!(d.hp, 200);
    }

    #[test]
    fn s1_i_1_npc_info_sync_anchor_king_slime() {
        // `NPCID.cs:10457`: SyncAnchor[50] = (0.5, 1) — King Slime (protocol
        // 4, cs_id 50) is the ONLY anchored kind. The wire position is
        // `position + Size * anchor`; the reader undoes it.
        let (w, h) = crate::npc::NpcKind::KingSlime.size();
        let n = NpcInfo {
            slot: 1,
            generation: 1,
            spawn: false,
            kind: 4,
            x: 100.0,
            y: 200.0,
            vx: 0.0,
            vy: 0.0,
            target: 255,
            dir_right: true,
            dir_y_down: false,
            sprite_right: true,
            life_full: true,
            hp: 2000,
            life_max: 2000,
            ai: [0.0; 4],
            spawned_from_statue: false,
            difficulty: 1.0,
            shimmer: false,
            release_owner: 255,
        };
        let f = S2c::NpcInfo(Box::new(n)).encode();
        let mut r = NetReader::new(&f[3..]);
        assert_eq!(r.u8(), Some(1));
        assert_eq!(r.u8(), Some(1));
        assert_eq!(r.f32(), Some(100.0 + w * 0.5));
        assert_eq!(r.f32(), Some(200.0 + h));
        let back = S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode");
        match back {
            S2c::NpcInfo(d) => {
                assert_eq!(d.x, 100.0);
                assert_eq!(d.y, 200.0);
            }
            other => panic!("expected 23, got {:?}", other.msg_id()),
        }
        assert_eq!(back.encode(), f);
    }

    #[test]
    fn s1_i_1_projectile_verbatim_body() {
        // `NetMessage.cs:776-841` — key, position, velocity, (short)type,
        // flags1, the optional flags2 (ai[2]), then the flag-gated fields in
        // C# order: ai0, ai1, banner, damage, knockBack, originalDamage, ai2.
        let key = projectile_key(2, 5, 3);
        let p = ProjectileState {
            key,
            x: 10.0,
            y: 20.0,
            vx: 1.0,
            vy: -1.0,
            kind: 7,
            ai0: Some(0.5),
            ai1: None,
            banner_id: 42,
            damage: 15,
            knock_back: 2.5,
            original_damage: 20,
            ai2: Some(-3.0),
        };
        let f = S2c::Projectile(Box::new(p)).encode();
        assert_eq!(f[2], 27);
        let mut expect: Vec<u8> = Vec::new();
        expect.extend_from_slice(&key.to_le_bytes());
        for v in [10.0f32, 20.0, 1.0, -1.0] {
            expect.extend_from_slice(&v.to_le_bytes());
        }
        expect.extend_from_slice(&7i16.to_le_bytes());
        // flags1: b0 ai0 + b2 ai2-present + b3 banner + b4 damage + b5 kb
        // + b6 originalDamage = 0x7D; flags2 = 0x01 (ai[2]).
        expect.push(0x7D);
        expect.push(0x01);
        expect.extend_from_slice(&0.5f32.to_le_bytes());
        expect.extend_from_slice(&42u16.to_le_bytes());
        expect.extend_from_slice(&15i16.to_le_bytes());
        expect.extend_from_slice(&2.5f32.to_le_bytes());
        expect.extend_from_slice(&20i16.to_le_bytes());
        expect.extend_from_slice(&(-3.0f32).to_le_bytes());
        assert_eq!(&f[3..], &expect[..]);
        assert_eq!(S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap().encode(), f);
        let c = C2s::Projectile(Box::new(p)).encode();
        assert_eq!(c, f, "27's body is identical in both directions");
        // The all-zero minimal body: no flags2, no conditional fields.
        let bare = ProjectileState {
            ai0: None,
            ai1: None,
            ai2: None,
            banner_id: 0,
            damage: 0,
            knock_back: 0.0,
            original_damage: 0,
            ..p
        };
        let bf = S2c::Projectile(Box::new(bare)).encode();
        assert_eq!(bf.len(), 3 + 4 + 16 + 2 + 1);
        assert_eq!(bf[3 + 22], 0);
    }

    #[test]
    fn s1_i_1_npc_strike_verbatim_body() {
        // `NetMessage.cs:844-850` — (byte)slot, generation, (short)damage,
        // knockback float, (byte)(hitDirection + 1), (byte)crit.
        let st = NpcStrike {
            npc: 9,
            generation: 4,
            damage: 37,
            knock_back: 3.5,
            hit_direction: -1,
            crit: true,
        };
        let f = S2c::NpcStrike(st).encode();
        assert_eq!(f[2], 28);
        let mut expect: Vec<u8> = vec![9, 4];
        expect.extend_from_slice(&37i16.to_le_bytes());
        expect.extend_from_slice(&3.5f32.to_le_bytes());
        expect.push(0); // -1 + 1 = 0
        expect.push(1);
        assert_eq!(&f[3..], &expect[..]);
        assert_eq!(S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap().encode(), f);
        let c = C2s::NpcStrike(st).encode();
        assert_eq!(c, f);
        // The -1 direction sentinel wraps through the byte (`:849`): the
        // reader's `(byte) - 1` recovers it (`MessageBuffer.cs:1776`).
        let back = NpcStrike::read_from(&mut NetReader::new(&f[3..])).unwrap();
        assert_eq!(back.hit_direction, -1);
        // The instant-kill send rides damage -1 (`NPC.cs:25947`).
        let kill = NpcStrike { damage: -1, crit: false, ..st };
        let kf = C2s::NpcStrike(kill).encode();
        assert_eq!(&kf[5..7], &(-1i16).to_le_bytes());
        assert_eq!(NpcStrike::read_from(&mut NetReader::new(&kf[3..])).unwrap().damage, -1);
    }

    #[test]
    fn s1_i_1_projectile_dead_verbatim_body() {
        // `NetMessage.cs:852-855` — (int)key + Vector2(number2, number3).
        // The Kill tail sends the entry position (`Projectile.cs:77256`);
        // the owner-lost despawn sends NaN/NaN (`Projectile.cs:16697`).
        let key = projectile_key(1, 4, 3);
        let f = S2c::ProjectileDead { key, x: 8.0, y: 9.0 }.encode();
        assert_eq!(f[2], 29);
        let mut expect: Vec<u8> = Vec::new();
        expect.extend_from_slice(&key.to_le_bytes());
        expect.extend_from_slice(&8.0f32.to_le_bytes());
        expect.extend_from_slice(&9.0f32.to_le_bytes());
        assert_eq!(&f[3..], &expect[..]);
        assert_eq!(S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap().encode(), f);
        // The client→host body carries the same position (the entry snapshot).
        let c = C2s::ProjectileDead { key, x: 8.0, y: 9.0 }.encode();
        assert_eq!(c, f);
        let back = C2s::decode(c[2], &mut NetReader::new(&c[3..])).unwrap();
        match back {
            C2s::ProjectileDead { key: k, x, y } => {
                assert_eq!(k, key);
                assert_eq!(x, 8.0);
                assert_eq!(y, 9.0);
            }
            other => panic!("expected 29, got {:?}", other.msg_id()),
        }
        // NaN rides verbatim (the bit pattern, not a canonicalised NaN).
        let nan = C2s::ProjectileDead { key, x: f32::NAN, y: f32::NAN }.encode();
        let mut r = NetReader::new(&nan[3..]);
        assert_eq!(r.i32(), Some(key));
        assert!(r.f32().unwrap().is_nan());
        assert!(r.f32().unwrap().is_nan());
    }

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

    // ---- S1-I-1 wave B (W4-d89): the 1-16 region census ----

    #[test]
    fn s1_i_1b_tile_frame_section_round_trips() {
        // `NetMessage.cs:427-432`: four (short)s — startX, startY,
        // endXInclusive, endYInclusive (`MessageBuffer.cs:895-899` reads
        // them into `WorldGen.SectionTileFrame`).
        let t = TileFrameSection {
            start_x: 3,
            start_y: -2,
            end_x: 7,
            end_y: 5,
        };
        let f = S2c::TileFrameSection(t).encode();
        assert_eq!(f[2], 11);
        let mut body = Vec::new();
        for v in [3i16, -2, 7, 5] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TileFrameSection(d) => assert_eq!(d, t),
            other => panic!("expected 11, got {:?}", other.msg_id()),
        }
        // A truncated body (3 of 4 shorts) is malformed, not a partial frame.
        assert!(S2c::decode(11, &mut NetReader::new(&body[..6])).is_none());
    }

    #[test]
    fn s1_i_1b_unknown_15_is_a_bare_no_op_frame() {
        // `MessageID.cs:38-39` ([Old]); no writer arm; the reader's ignore
        // group (`MessageBuffer.cs:4483-4490`) consumes nothing.
        let f = S2c::Unknown15.encode();
        assert_eq!(f[2], 15);
        assert_eq!(f.len(), 3, "id 15 has no body");
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::Unknown15 => {}
            other => panic!("expected 15, got {:?}", other.msg_id()),
        }
    }

    // ---- S1-I-1 wave B: per-id round trips for the world-sync region ----
    // Same convention as wave A: pin the id byte, the exact C# body bytes
    // (`NetMessage.cs`'s writer arms) and the decode round trip.

    #[test]
    fn s1_i_1b_tile_edit_round_trips() {
        // `NetMessage.cs:524-530`: (byte)kind + (short)x + (short)y +
        // (short)value + (byte)style.
        let t = TileEdit {
            kind: 1,
            x: 4180,
            y: 1130,
            value: 30,
            style: 2,
        };
        let f = S2c::TileEdit(t).encode();
        assert_eq!(f[2], 17);
        let mut body = vec![1];
        body.extend_from_slice(&4180i16.to_le_bytes());
        body.extend_from_slice(&1130i16.to_le_bytes());
        body.extend_from_slice(&30i16.to_le_bytes());
        body.push(2);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TileEdit(d) => assert_eq!(d, t),
            other => panic!("expected 17, got {:?}", other.msg_id()),
        }
        let c = C2s::TileEdit(t).encode();
        assert_eq!(c[2], 17);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TileEdit(d) => assert_eq!(d, t),
            other => panic!("expected 17, got {:?}", other.msg_id()),
        }
        // Kind 0 (KillTile) with the fail flag riding `value` = 1
        // (`DelegateMethods.cs:571`'s `1f` → the host relay's num152).
        let kill = TileEdit { kind: 0, x: -3, y: 0, value: 1, style: 0 };
        let k = C2s::TileEdit(kill).encode();
        match C2s::decode(k[2], &mut NetReader::new(&k[3..])).expect("decode") {
            C2s::TileEdit(d) => assert_eq!(d, kill),
            other => panic!("expected 17, got {:?}", other.msg_id()),
        }
        // A truncated body (4 of 8 bytes) must not decode.
        assert!(C2s::decode(17, &mut NetReader::new(&body[..4])).is_none());
    }

    #[test]
    fn s1_i_1b_tile_frame_section_world_sync_witness() {
        // `NetMessage.cs:427-432`: four shorts; the reader feeds
        // `WorldGen.SectionTileFrame(startX, startY, endXInclusive,
        // endYInclusive)` (`MessageBuffer.cs:898`, `WorldGen.cs:80343`).
        let f = TileFrameSection {
            start_x: 200,
            start_y: 150,
            end_x: 399,
            end_y: 299,
        };
        let frame = S2c::TileFrameSection(f).encode();
        assert_eq!(frame[2], 11);
        let mut body = Vec::new();
        for v in [200i16, 150, 399, 299] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        assert_eq!(&frame[3..], &body);
        match S2c::decode(frame[2], &mut NetReader::new(&frame[3..])).expect("decode") {
            S2c::TileFrameSection(d) => assert_eq!(d, f),
            other => panic!("expected 11, got {:?}", other.msg_id()),
        }
        // Read-side only (deprecated, 0 SendData sites) — no C2s codec.
        assert!(C2s::decode(11, &mut NetReader::new(&body)).is_none());
    }

    #[test]
    fn s1_i_1b_door_toggle_round_trips() {
        // `NetMessage.cs:537-542`: (byte)action + (short)x + (short)y +
        // ((number4 == 1f) ? 1 : 0).
        let d = DoorToggle {
            action: 0,
            x: 4200,
            y: 1100,
            direction: 1,
        };
        let f = S2c::DoorToggle(d).encode();
        assert_eq!(f[2], 19);
        let mut body = vec![0];
        body.extend_from_slice(&4200i16.to_le_bytes());
        body.extend_from_slice(&1100i16.to_le_bytes());
        body.push(1);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::DoorToggle(dd) => assert_eq!(dd, d),
            other => panic!("expected 19, got {:?}", other.msg_id()),
        }
        let c = C2s::DoorToggle(d).encode();
        assert_eq!(c[2], 19);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::DoorToggle(dd) => assert_eq!(dd, d),
            other => panic!("expected 19, got {:?}", other.msg_id()),
        }
        // The tall-gate close (action 5) rides direction 0 — the reader
        // maps `!= 0` to 1 else -1 (`MessageBuffer.cs:1306`).
        let gate = DoorToggle { action: 5, x: 10, y: 20, direction: 0 };
        let g = C2s::DoorToggle(gate).encode();
        assert_eq!(g[3], 5);
        assert_eq!(*g.last().unwrap(), 0);
    }

    #[test]
    fn s1_i_1b_tile_and_wall_paint_round_trip() {
        // `NetMessage.cs:1081-1087` — 63 and 64 share the case: (short)x +
        // (short)y + (byte)paint + (byte)coat.
        let p = TilePaint {
            x: 4210,
            y: 1120,
            paint: 13,
            coat: 1,
        };
        let mut body = Vec::new();
        body.extend_from_slice(&4210i16.to_le_bytes());
        body.extend_from_slice(&1120i16.to_le_bytes());
        body.push(13);
        body.push(1);
        for (mk, sid, rid) in [
            (C2s::TilePaint as fn(TilePaint) -> C2s, 63u8, 63u8),
            (C2s::WallPaint as fn(TilePaint) -> C2s, 64, 64),
        ] {
            let c = mk(p).encode();
            assert_eq!(c[2], sid);
            assert_eq!(&c[3..], &body);
            match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
                C2s::TilePaint(d) if rid == 63 => assert_eq!(d, p),
                C2s::WallPaint(d) if rid == 64 => assert_eq!(d, p),
                other => panic!("expected {rid}, got {:?}", other.msg_id()),
            }
        }
        for (mk, sid) in [
            (S2c::TilePaint as fn(TilePaint) -> S2c, 63u8),
            (S2c::WallPaint as fn(TilePaint) -> S2c, 64),
        ] {
            let f = mk(p).encode();
            assert_eq!(f[2], sid);
            assert_eq!(&f[3..], &body);
            match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::TilePaint(d) if sid == 63 => assert_eq!(d, p),
                S2c::WallPaint(d) if sid == 64 => assert_eq!(d, p),
                other => panic!("expected {sid}, got {:?}", other.msg_id()),
            }
        }
        // coat == 0 is the plain-paint arm (`MessageBuffer.cs:2915-2922`).
        let plain = TilePaint { coat: 0, ..p };
        let c = C2s::TilePaint(plain).encode();
        assert_eq!(*c.last().unwrap(), 0);
    }

    #[test]
    fn s1_i_1b_tile_picking_round_trips() {
        // `NetMessage.cs:1509-1514`: (byte)player + (short)x + (short)y +
        // (byte)damage.
        let p = TilePicking {
            player_id: 6,
            x: 4220,
            y: 1140,
            damage: 35,
        };
        let mut body = vec![6];
        body.extend_from_slice(&4220i16.to_le_bytes());
        body.extend_from_slice(&1140i16.to_le_bytes());
        body.push(35);
        let c = C2s::TilePicking(p).encode();
        assert_eq!(c[2], 125);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TilePicking(d) => assert_eq!(d, p),
            other => panic!("expected 125, got {:?}", other.msg_id()),
        }
        let f = S2c::TilePicking(p).encode();
        assert_eq!(f[2], 125);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TilePicking(d) => assert_eq!(d, p),
            other => panic!("expected 125, got {:?}", other.msg_id()),
        }
    }

    // ---- S1-I-1 projectile-family first half (W4-d93): 27/28/29/95 ----

    #[test]
    fn s1_i_1d93_sync_projectile_body_bytes() {
        // `NetMessage.cs:776-842` (case 27), verbatim field order: key int,
        // Vector2 position, Vector2 velocity, (short)type, flags1 BitsByte,
        // [flags2 BitsByte when flags1 bit 2], ai[0], ai[1], (ushort)banner,
        // (short)damage, float knockBack, (short)originalDamage, ai[2].
        let st = ProjectileState {
            key: projectile_key(1, 2, 0),
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
        };
        let f = st.frame();
        assert_eq!(f[2], 27);
        let mut body = Vec::new();
        body.extend_from_slice(&st.key.to_le_bytes());
        for v in [16.0f32, -32.5, 1.5, -2.0] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        body.extend_from_slice(&4i16.to_le_bytes());
        // bits: 0 ai0, 2 flags2-present, 4 damage, 5 knockback.
        body.push(0x35);
        body.push(0x01); // flags2: bit 0 = ai[2] present
        body.extend_from_slice(&0.25f32.to_le_bytes());
        body.extend_from_slice(&12i16.to_le_bytes());
        body.extend_from_slice(&3.0f32.to_le_bytes());
        body.extend_from_slice(&(-1.0f32).to_le_bytes());
        assert_eq!(&f[3..], &body);
        let back = C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode");
        assert_eq!(back.encode(), f);
        // The minimal body (`MessageBuffer.cs:1715-1724` with every
        // conditional absent): flags1 = 0 writes NO flags2 byte and no
        // extras — 23 bytes of body.
        let bare = ProjectileState {
            key: 513,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            kind: 1,
            ai0: None,
            ai1: None,
            banner_id: 0,
            damage: 0,
            knock_back: 0.0,
            original_damage: 0,
            ai2: None,
        };
        let b = bare.frame();
        assert_eq!(b[2], 27);
        assert_eq!(b.len(), 3 + 4 + 16 + 2 + 1);
        assert_eq!(b[3 + 4 + 16 + 2], 0); // flags1 = 0, no flags2 follows
        assert_eq!(
            C2s::decode(b[2], &mut NetReader::new(&b[3..]))
                .expect("decode")
                .encode(),
            b
        );
    }

    #[test]
    fn s1_i_1d93_kill_projectile_body_bytes() {
        // `NetMessage.cs:852-855` (case 29): `number` (the key int) +
        // `Vector2(number2, number3)`. Both directions carry the position
        // verbatim — the C2s used to hardcode NaN, which dropped the kill
        // site's entry snapshot (`Projectile.cs:77256`).
        let key = projectile_key(2, 7, 1);
        let mut body = Vec::new();
        body.extend_from_slice(&key.to_le_bytes());
        body.extend_from_slice(&12.5f32.to_le_bytes());
        body.extend_from_slice(&(-8.0f32).to_le_bytes());
        let c = C2s::ProjectileDead {
            key,
            x: 12.5,
            y: -8.0,
        }
        .encode();
        assert_eq!(c[2], 29);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ProjectileDead { key: k, x, y } => {
                assert_eq!((k, x, y), (key, 12.5, -8.0));
            }
            other => panic!("expected 29, got {:?}", other.msg_id()),
        }
        let s = S2c::ProjectileDead {
            key,
            x: 12.5,
            y: -8.0,
        }
        .encode();
        assert_eq!(s[2], 29);
        assert_eq!(&s[3..], &body);
        // The NaN/NaN silent-deactivate variant (`Projectile.cs:16697`)
        // survives the wire bit-for-bit (`MessageBuffer.cs:1835-1843`).
        let nan = C2s::ProjectileDead {
            key,
            x: f32::NAN,
            y: f32::NAN,
        }
        .encode();
        match C2s::decode(nan[2], &mut NetReader::new(&nan[3..])).expect("decode") {
            C2s::ProjectileDead { x, y, .. } => {
                assert!(x.is_nan() && y.is_nan());
            }
            other => panic!("expected 29, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1b_the_1_to_16_region_is_fully_mapped() {
        // The census against `MessageID.cs:9-41`: every id 1..=16 has a
        // constant in `id` (11/15 landed with wave B). 17-19 stay outside
        // this family (17 superseded by 20, 18 rides internal 205).
        let region: [u8; 16] = [
            id::CONNECT,
            id::DISCONNECT,
            id::PLAYER_ID,
            id::PLAYER_INFO,
            id::PLAYER_INVENTORY,
            id::WORLD_REQUEST,
            id::WORLD_INFO,
            id::SPAWN_REQUEST,
            id::STATUS,
            id::TILE_BLOCK,
            id::TILE_FRAME_SECTION,
            id::PLAYER_SPAWN,
            id::PLAYER_STATE,
            id::PLAYER_ACTIVE,
            id::UNKNOWN_15,
            id::PLAYER_HP,
        ];
        for (i, v) in region.iter().enumerate() {
            assert_eq!(*v as usize, i + 1, "MessageID region slot {}", i + 1);
        }
    }

    #[test]
    fn s1_i_1b_team_change_ui_shares_the_45_body() {
        // `NetMessage.cs:970-974` — 157 shares 45's writer case: (byte)player
        // + (byte)team. The reader tail (`MessageBuffer.cs:2355-2363`) is the
        // dispatcher's job (S1-I-4).
        let t = TeamChange { player_id: 3, team: 2 };
        let c = C2s::TeamChangeUi(t).encode();
        assert_eq!(c[2], 157);
        assert_eq!(&c[3..], &[3, 2]);
        // Byte-identical body to the 45 codec, only the id byte differs.
        let plain = C2s::TeamChange(t).encode();
        assert_eq!(&c[3..], &plain[3..]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TeamChangeUi(d) => assert_eq!(d, t),
            other => panic!("expected 157, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1d93_damage_npc_body_bytes() {
        // `NetMessage.cs:844-850` (case 28): (byte)npc, (byte)generation,
        // (short)damage, float knockback, (byte)(hitDirection + 1),
        // (byte)crit. The NPC-family apply arms belong to the NPC half
        // (W4-d91/d92); this pins the shared wire layout only.
        let st = NpcStrike {
            npc: 7,
            generation: 2,
            damage: -13,
            knock_back: 5.5,
            hit_direction: -1,
            crit: true,
        };
        let c = C2s::NpcStrike(st).encode();
        assert_eq!(c[2], 28);
        let mut body = vec![7, 2];
        body.extend_from_slice(&(-13i16).to_le_bytes());
        body.extend_from_slice(&5.5f32.to_le_bytes());
        body.push(0); // (-1 + 1f) as byte — the wire offset (`:849`)
        body.push(1);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::NpcStrike(d) => assert_eq!(d, st),
            other => panic!("expected 28, got {:?}", other.msg_id()),
        }
        let s = S2c::NpcStrike(st).encode();
        assert_eq!(s[2], 28);
        assert_eq!(&s[3..], &body);
        match S2c::decode(s[2], &mut NetReader::new(&s[3..])).expect("decode") {
            S2c::NpcStrike(d) => assert_eq!(d, st),
            other => panic!("expected 28, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1b_extra_spawn_loaded_round_trips() {
        // `NetMessage.cs:1659-1661`: one (byte)player; the reader spawns with
        // `PlayerSpawnContext.TeamSwap` = 3 (`MessageBuffer.cs:4453-4454`,
        // `PlayerSpawnContext.cs:5-8`).
        let f = S2c::ExtraSpawnLoaded { player_id: 4 }.encode();
        assert_eq!(f[2], 158);
        assert_eq!(&f[3..], &[4]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ExtraSpawnLoaded { player_id } => assert_eq!(player_id, 4),
            other => panic!("expected 158, got {:?}", other.msg_id()),
        }
        // Read-side only — the C# sends it from the server's 157 tail.
        assert!(C2s::decode(158, &mut NetReader::new(&[4])).is_none());
    }

    #[test]
    fn s1_i_1b_section_request_round_trips() {
        // `NetMessage.cs:1662-1665` writes two (short)s; the reader reads two
        // ReadUInt16 (`MessageBuffer.cs:4460`) — the non-negative section
        // indices are the same bytes either way.
        let c = C2s::SectionRequest { section_x: 21, section_y: 7 }.encode();
        assert_eq!(c[2], 159);
        let mut body = Vec::new();
        body.extend_from_slice(&21u16.to_le_bytes());
        body.extend_from_slice(&7u16.to_le_bytes());
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::SectionRequest { section_x, section_y } => {
                assert_eq!((section_x, section_y), (21, 7));
            }
            other => panic!("expected 159, got {:?}", other.msg_id()),
        }
        // The section math the request keys on: `Netplay.GetSectionX/Y`
        // (`Netplay.cs:770-777`) = the port's `section_x/y` — a spawn at
        // tile (4200, 1100) asks for section (21, 7).
        assert_eq!(section_x(4200), 21);
        assert_eq!(section_y(1100), 7);
        // Truncated body must not decode.
        assert!(C2s::decode(159, &mut NetReader::new(&body[..3])).is_none());
    }

    #[test]
    fn s1_i_1d93_murder_portal_round_trips() {
        // `NetMessage.cs:1333-1336` (case 95): (ushort)number (the peer's
        // owner id) + (byte)number2 (the portal form, ai[1]). Client→server
        // only — the C# host never sends 95 (`MessageBuffer.cs:3519-3537`
        // reads it at netMode == 2 only).
        let c = C2s::MurderPortal { owner: 3, form: 1 }.encode();
        assert_eq!(c[2], 95);
        assert_eq!(&c[3..], &[3, 0, 1]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::MurderPortal { owner, form } => assert_eq!((owner, form), (3, 1)),
            other => panic!("expected 95, got {:?}", other.msg_id()),
        }
        // No S2c counterpart exists in the C# reader.
        assert!(S2c::decode(95, &mut NetReader::new(&[3, 0, 1])).is_none());
    }

    #[test]
    fn s1_i_1d93_golf_ball_in_cup_body_bytes() {
        // `NetMessage.cs:1521-1527` (case 128): (byte)owner, (ushort)x,
        // (ushort)y, (ushort)hits, (ushort)type. The writer's field order is
        // NOT the send's argument order (`GolfHelper.cs:109` passes
        // owner, hits, type, 0f, x, y → number/2/3/5/6); the reader mirrors
        // the writer (`MessageBuffer.cs:4060-4064`).
        let g = GolfBallInCup {
            owner: 3,
            x: 1234,
            y: 567,
            hits: 7,
            ball_type: 520,
        };
        let mut body = vec![3];
        body.extend_from_slice(&1234u16.to_le_bytes());
        body.extend_from_slice(&567u16.to_le_bytes());
        body.extend_from_slice(&7u16.to_le_bytes());
        body.extend_from_slice(&520u16.to_le_bytes());
        let c = C2s::GolfBallInCup(g).encode();
        assert_eq!(c[2], 128);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::GolfBallInCup(d) => assert_eq!(d, g),
            other => panic!("expected 128, got {:?}", other.msg_id()),
        }
        // The host's relay (`MessageBuffer.cs:4067`) writes the same arm.
        let s = S2c::GolfBallInCup(g).encode();
        assert_eq!(s[2], 128);
        assert_eq!(&s[3..], &body);
        match S2c::decode(s[2], &mut NetReader::new(&s[3..])).expect("decode") {
            S2c::GolfBallInCup(d) => assert_eq!(d, g),
            other => panic!("expected 128, got {:?}", other.msg_id()),
        }
        // The swapped-pair trap: the send args are owner, hits, type, 0f,
        // x, y — a codec that wrote hits/type before x/y fails above; pin
        // the asymmetry with a second fixture (distinct ushort values).
        let g2 = GolfBallInCup {
            owner: 255,
            x: 0,
            y: 65535,
            hits: 65535,
            ball_type: 1,
        };
        let c2 = C2s::GolfBallInCup(g2).encode();
        assert_eq!(&c2[3..], &[255, 0, 0, 255, 255, 255, 255, 1, 0]);
        match C2s::decode(c2[2], &mut NetReader::new(&c2[3..])).expect("decode") {
            C2s::GolfBallInCup(d) => assert_eq!(d, g2),
            other => panic!("expected 128, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn s1_i_1d93_mass_wire_operation_body_bytes() {
        // `NetMessage.cs:1412-1418` (case 109): (short)ps.X, (short)ps.Y,
        // (short)pe.X, (short)pe.Y, (byte)ToolMode. Client→server only — the
        // host arm (`MessageBuffer.cs:3774-3788`) runs the operation and
        // never relays the 109.
        let m = MassWireOperation {
            start_x: -12,
            start_y: 300,
            end_x: 45,
            end_y: -7,
            tool_mode: 0x21, // Red | Cutter (`WiresUI.cs:13-21`)
        };
        let mut body = Vec::new();
        body.extend_from_slice(&(-12i16).to_le_bytes());
        body.extend_from_slice(&300i16.to_le_bytes());
        body.extend_from_slice(&45i16.to_le_bytes());
        body.extend_from_slice(&(-7i16).to_le_bytes());
        body.push(0x21);
        let c = C2s::MassWireOperation(m).encode();
        assert_eq!(c[2], 109);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::MassWireOperation(d) => assert_eq!(d, m),
            other => panic!("expected 109, got {:?}", other.msg_id()),
        }
        // The C# has no server→client 109 send site.
        assert!(S2c::decode(109, &mut NetReader::new(&body)).is_none());
    }

    #[test]
    fn s1_i_1d93_mass_wire_operation_pay_body_bytes() {
        // `NetMessage.cs:1419-1423` (case 110): (short)itemType,
        // (short)count, (byte)player. Server→client only — the host bills
        // 530 then 849 (`Wiring.cs:441-442`); the client pays only its own
        // (`MessageBuffer.cs:3798`).
        let p = MassWireOperationPay {
            item_type: 530,
            count: 17,
            player: 2,
        };
        let mut body = Vec::new();
        body.extend_from_slice(&530i16.to_le_bytes());
        body.extend_from_slice(&17i16.to_le_bytes());
        body.push(2);
        let s = S2c::MassWireOperationPay(p).encode();
        assert_eq!(s[2], 110);
        assert_eq!(&s[3..], &body);
        match S2c::decode(s[2], &mut NetReader::new(&s[3..])).expect("decode") {
            S2c::MassWireOperationPay(d) => assert_eq!(d, p),
            other => panic!("expected 110, got {:?}", other.msg_id()),
        }
        // The actuator twin — the second bill of the pair.
        let p2 = MassWireOperationPay {
            item_type: 849,
            count: 0,
            player: 255,
        };
        let s2 = S2c::MassWireOperationPay(p2).encode();
        assert_eq!(&s2[3..], &[0x51, 0x03, 0, 0, 255]);
        match S2c::decode(s2[2], &mut NetReader::new(&s2[3..])).expect("decode") {
            S2c::MassWireOperationPay(d) => assert_eq!(d, p2),
            other => panic!("expected 110, got {:?}", other.msg_id()),
        }
        // The C# has no client→server 110 send site.
        assert!(C2s::decode(110, &mut NetReader::new(&body)).is_none());
    }

    // ---- W4-d96 (S1-I-1 player-family first half): per-id round trips ----
    // Each test pins the frame's id byte, the exact C# body bytes
    // (`NetMessage.cs`'s writer arms) and the decode round trip.

    /// `5 SyncEquipment` (`NetMessage.cs:199-222`): the id byte leads —
    /// `(byte)number, (short)slot, (short)stack, (byte)prefix, (short)type,
    /// BitsByte{favorited, blocked}`. The stack clamp (`:210-213`) and the
    /// `== 1` active test of 14 are pinned here too.
    #[test]
    fn w4_d96_inventory_leads_with_the_id_byte() {
        let item = NetItem {
            kind: 757,
            stack: 1,
            prefix: 81,
            favorited: true,
        };
        let mut body = vec![4];
        body.extend_from_slice(&12i16.to_le_bytes());
        body.extend_from_slice(&1i16.to_le_bytes());
        body.push(81);
        body.extend_from_slice(&757i16.to_le_bytes());
        body.push(1); // bit 0 favorited; bit 1 (blocked) clear
        let c = C2s::Inventory {
            player_id: 4,
            slot: 12,
            item,
        }
        .encode();
        assert_eq!(c[2], 5);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::Inventory {
                player_id,
                slot,
                item: d,
            } => {
                assert_eq!(player_id, 4);
                assert_eq!(slot, 12);
                assert_eq!(d, item);
            }
            other => panic!("expected 5, got {:?}", other.msg_id()),
        }
        // The S2c relay carries the same leading id (`NetMessage.cs:201`).
        let f = S2c::Inventory {
            player_id: 4,
            slot: 12,
            item,
        }
        .encode();
        assert_eq!(f[2], 5);
        assert_eq!(&f[3..], &body);
        // A negative stack clamps to 0 on read (`NetMessage.cs:210-213`).
        let mut neg = vec![4];
        neg.extend_from_slice(&12i16.to_le_bytes());
        neg.extend_from_slice(&(-5i16).to_le_bytes());
        neg.push(0);
        neg.extend_from_slice(&757i16.to_le_bytes());
        neg.push(0);
        match C2s::decode(5, &mut NetReader::new(&neg)).expect("decode") {
            C2s::Inventory { item: d, .. } => assert_eq!(d.stack, 0),
            other => panic!("expected 5, got {:?}", other.msg_id()),
        }
    }

    /// `14 PlayerActive` (`NetMessage.cs:515-518`): `(byte)number +
    /// (byte)number2`; the read side tests the byte `== 1`
    /// (`MessageBuffer.cs:1070`), so 2 deactivates.
    #[test]
    fn w4_d96_player_active_tests_the_byte_equals_one() {
        let f = S2c::PlayerActive {
            player_id: 3,
            active: true,
        }
        .encode();
        assert_eq!(f[2], 14);
        assert_eq!(&f[3..], &[3, 1]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerActive { player_id, active } => {
                assert_eq!(player_id, 3);
                assert!(active);
            }
            other => panic!("expected 14, got {:?}", other.msg_id()),
        }
        // `num233 == 1` only (`MessageBuffer.cs:1070`): 2 is NOT active.
        match S2c::decode(14, &mut NetReader::new(&[3, 2])).expect("decode") {
            S2c::PlayerActive { active, .. } => assert!(!active),
            other => panic!("expected 14, got {:?}", other.msg_id()),
        }
        let c = C2s::PlayerActive {
            player_id: 3,
            active: false,
        }
        .encode();
        assert_eq!(c[2], 14);
        assert_eq!(&c[3..], &[3, 0]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerActive { player_id, active } => {
                assert_eq!(player_id, 3);
                assert!(!active);
            }
            other => panic!("expected 14, got {:?}", other.msg_id()),
        }
    }

    /// `12 PlayerSpawn` (`NetMessage.cs:433-445`): the id byte leads —
    /// `(byte)number, (short)SpawnX, (short)SpawnY, (int)respawnTimer,
    /// (short)deathsPVE, (short)deathsPVP, (byte)team, (byte)context`.
    #[test]
    fn w4_d96_player_spawn_leads_with_the_id_byte() {
        let s = PlayerSpawn {
            player_id: 6,
            x: 4200,
            y: 1100,
            respawn_timer: 300,
            deaths_pve: 3,
            deaths_pvp: 1,
            team: 2,
            spawn_context: 1,
        };
        let mut body = vec![6];
        body.extend_from_slice(&4200i16.to_le_bytes());
        body.extend_from_slice(&1100i16.to_le_bytes());
        body.extend_from_slice(&300i32.to_le_bytes());
        body.extend_from_slice(&3i16.to_le_bytes());
        body.extend_from_slice(&1i16.to_le_bytes());
        body.push(2);
        body.push(1);
        let c = C2s::PlayerSpawn(s).encode();
        assert_eq!(c[2], 12);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerSpawn(d) => assert_eq!(d, s),
            other => panic!("expected 12, got {:?}", other.msg_id()),
        }
        let f = S2c::PlayerSpawn(s).encode();
        assert_eq!(f[2], 12);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerSpawn(d) => assert_eq!(d, s),
            other => panic!("expected 12, got {:?}", other.msg_id()),
        }
    }

    /// `35 PlayerHeal` (`NetMessage.cs:929-932`): `(byte)number +
    /// (short)number2`, both directions (`MessageBuffer.cs:2160-2177`).
    #[test]
    fn w4_d96_player_heal_round_trips() {
        let h = PlayerHeal {
            player_id: 3,
            amount: 42,
        };
        let mut body = vec![3];
        body.extend_from_slice(&42i16.to_le_bytes());
        let f = S2c::PlayerHeal(h).encode();
        assert_eq!(f[2], 35);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerHeal(d) => assert_eq!(d, h),
            other => panic!("expected 35, got {:?}", other.msg_id()),
        }
        let c = C2s::PlayerHeal(h).encode();
        assert_eq!(c[2], 35);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerHeal(d) => assert_eq!(d, h),
            other => panic!("expected 35, got {:?}", other.msg_id()),
        }
    }

    // ---- S1-I-1 wave I: the item/chest-family region (21/22/30-34/80/81) ----

    /// `21 SyncItem` bit 3 — `enemyGrabDelayTime > 0` sets bit 3 and appends
    /// the clamped byte AFTER the shimmer tail (`NetMessage.cs:652`/
    /// `:660-668`); the reader consumes it (`MessageBuffer.cs:1455`/`:1459`).
    #[test]
    fn s1_i_1_item_state_enemy_grab_bit_round_trips() {
        let d = ItemState {
            slot: 12,
            x: 100.0,
            y: 200.0,
            vx: 1.5,
            vy: -0.5,
            stack: 3,
            prefix: 81,
            kind: 757,
            shimmered: false,
            shimmer_time: 0.0,
            enemy_grab_delay: 100,
        };
        let f = S2c::ItemState(Box::new(d)).encode();
        assert_eq!(f[2], 21);
        let mut body = Vec::new();
        body.extend_from_slice(&12i16.to_le_bytes());
        for v in [100.0f32, 200.0, 1.5, -0.5] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        body.extend_from_slice(&3i16.to_le_bytes());
        body.push(81);
        body.push(1 << 3); // bit 3 only — no shimmer tail
        body.extend_from_slice(&757i16.to_le_bytes());
        body.push(100); // the clamped enemy-grab byte, LAST
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemState(back) => assert_eq!(*back, d),
            other => panic!("expected 21, got {:?}", other.msg_id()),
        }
        // Both bits set: the shimmer pair comes first, the enemy byte last
        // (`:660-668`'s order).
        let both = ItemState {
            shimmered: true,
            shimmer_time: 0.9,
            enemy_grab_delay: 255,
            ..d
        };
        let f = S2c::ItemState(Box::new(both)).encode();
        assert_eq!(f[3 + 2 + 16 + 2 + 1], (1 << 2) | (1 << 3));
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemState(back) => assert_eq!(*back, both),
            other => panic!("expected 21, got {:?}", other.msg_id()),
        }
        // Zero delay writes NO tail byte and clears bit 3 (`:652`'s `> 0`).
        let none = ItemState {
            enemy_grab_delay: 0,
            ..d
        };
        let f = S2c::ItemState(Box::new(none)).encode();
        assert_eq!(f[3 + 2 + 16 + 2 + 1], 0);
        assert_eq!(f.len(), 3 + 2 + 16 + 2 + 1 + 1 + 2);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemState(back) => assert_eq!(*back, none),
            other => panic!("expected 21, got {:?}", other.msg_id()),
        }
    }

    /// `34 ChestUpdates` (`NetMessage.cs:913-927`): byte action + three
    /// shorts + the fifth short — the host writes `number5`, the client a
    /// literal 0 (`:926`); the host reader zeroes it
    /// (`MessageBuffer.cs:1991-1994`), the client keeps it (`:1990`).
    #[test]
    fn s1_i_1_chest_updates_round_trips() {
        let u = ChestUpdates {
            action: 0,
            x: 120,
            y: 340,
            style: 4,
            chest: 17,
        };
        let f = S2c::ChestUpdates(u).encode();
        assert_eq!(f[2], 34);
        let mut body = vec![0];
        for v in [120i16, 340, 4, 17] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ChestUpdates(d) => assert_eq!(d, u),
            other => panic!("expected 34, got {:?}", other.msg_id()),
        }
        // The client's destroy ask (`Player.cs:53787`): action 1, style and
        // index default to 0 — the writer's else branch (`:924-927`).
        let c = C2s::ChestUpdates(ChestUpdates {
            action: 1,
            x: 120,
            y: 340,
            style: 0,
            chest: 17, // ignored on the wire: the client writes 0
        })
        .encode();
        assert_eq!(c[2], 34);
        assert_eq!(&c[c.len() - 2..], &0i16.to_le_bytes());
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ChestUpdates(d) => {
                assert_eq!(d.action, 1);
                assert_eq!((d.x, d.y, d.style, d.chest), (120, 340, 0, 0));
            }
            other => panic!("expected 34, got {:?}", other.msg_id()),
        }
        // The refusal relay: chest = -1 (`MessageBuffer.cs:2006`).
        let refused = ChestUpdates { chest: -1, ..u };
        let f = S2c::ChestUpdates(refused).encode();
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ChestUpdates(d) => assert_eq!(d.chest, -1),
            other => panic!("expected 34, got {:?}", other.msg_id()),
        }
    }

    /// `81 CombatTextInt` (`NetMessage.cs:1179-1186`): float x, float y,
    /// `WriteRGB`'s R/G/B bytes (`Utils.cs:1369-1374`), `(int)` amount.
    /// Host→client only — no `C2s` arm (`MessageBuffer.cs:3227-3233` reads
    /// it at `netMode == 1` only).
    #[test]
    fn s1_i_1_combat_text_int_round_trips() {
        let t = CombatTextInt {
            x: 1234.5,
            y: -678.25,
            color: Rgb(100, 255, 100), // CombatText.HealLife
            amount: 42,
        };
        let f = S2c::CombatTextInt(t).encode();
        assert_eq!(f[2], 81);
        let mut body = Vec::new();
        body.extend_from_slice(&1234.5f32.to_le_bytes());
        body.extend_from_slice(&(-678.25f32).to_le_bytes());
        body.extend_from_slice(&[100, 255, 100]);
        body.extend_from_slice(&42i32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::CombatTextInt(d) => assert_eq!(d, t),
            other => panic!("expected 81, got {:?}", other.msg_id()),
        }
        // The reader truncates the floats to the Rectangle's ints
        // (`MessageBuffer.cs:3230-3231`'s `(int)ReadSingle()`).
        assert_eq!(t.x as i32, 1234);
        assert_eq!(t.y as i32, -678);
    }

    /// `35 PlayerHeal` (`NetMessage.cs:929-932`): byte player id + short
    /// amount; the host overrides the id to the sender and relays
    /// (`MessageBuffer.cs:2163-2166`/`:2174`).
    #[test]
    fn s1_i_1_player_heal_round_trips() {
        let h = PlayerHeal {
            player_id: 3,
            amount: 20,
        };
        let f = C2s::PlayerHeal(h).encode();
        assert_eq!(f[2], 35);
        let mut body = vec![3u8];
        body.extend_from_slice(&20i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::PlayerHeal(d) => assert_eq!(d, h),
            other => panic!("expected 35, got {:?}", other.msg_id()),
        }
        // The host relay keeps the same layout (`:2174`'s TrySendData).
        let f = S2c::PlayerHeal(h).encode();
        assert_eq!(f[2], 35);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerHeal(d) => assert_eq!(d, h),
            other => panic!("expected 35, got {:?}", other.msg_id()),
        }
    }

    /// `40 SyncTalkNPC` (`NetMessage.cs:952-955`): `(byte)number +
    /// (short)talkNPC`; `-1` = not talking (`MessageBuffer.cs:2256-2270`).
    #[test]
    fn w4_d96_talk_npc_round_trips() {
        let t = TalkNpc {
            player_id: 2,
            npc: 17,
        };
        let mut body = vec![2];
        body.extend_from_slice(&17i16.to_le_bytes());
        let f = S2c::TalkNpc(t).encode();
        assert_eq!(f[2], 40);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TalkNpc(d) => assert_eq!(d, t),
            other => panic!("expected 40, got {:?}", other.msg_id()),
        }
        let c = C2s::TalkNpc(t).encode();
        assert_eq!(c[2], 40);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TalkNpc(d) => assert_eq!(d, t),
            other => panic!("expected 40, got {:?}", other.msg_id()),
        }
        // The close-conversation value (`Player.SetTalkNPC(-1)`).
        let off = TalkNpc {
            player_id: 2,
            npc: -1,
        };
        let mut body = vec![2];
        body.extend_from_slice(&(-1i16).to_le_bytes());
        let c = C2s::TalkNpc(off).encode();
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TalkNpc(d) => assert_eq!(d, off),
            other => panic!("expected 40, got {:?}", other.msg_id()),
        }
    }

    /// `36 SyncPlayerZone` (`NetMessage.cs:933-944`): `(byte)number +
    /// zone1..5 (BitsByte each) + (byte)townNPCs` — seven bytes, both
    /// directions (`MessageBuffer.cs:2178-2201`). `zone5[0]` is the faeling
    /// edge bit (`:2186`/`:2195-2198`).
    #[test]
    fn w4_d96_player_zone_round_trips() {
        let z = PlayerZone {
            player_id: 4,
            zone1: BitsByte(0b0000_0010), // jungle
            zone2: BitsByte(0b0001_0000),
            zone3: BitsByte(0b1000_0001),
            zone4: BitsByte(0b0010_0100),
            zone5: BitsByte(0b0000_0001), // the faeling edge bit
            town_npcs: 3,
        };
        let body = [4, 0x02, 0x10, 0x81, 0x24, 0x01, 3];
        let c = C2s::PlayerZone(z).encode();
        assert_eq!(c[2], 36);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerZone(d) => assert_eq!(d, z),
            other => panic!("expected 36, got {:?}", other.msg_id()),
        }
        let f = S2c::PlayerZone(z).encode();
        assert_eq!(f[2], 36);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerZone(d) => assert_eq!(d, z),
            other => panic!("expected 36, got {:?}", other.msg_id()),
        }
        // All-zero zones (`Player.cs:1364-1372`'s defaults) stay seven bytes.
        let empty = PlayerZone {
            player_id: 0,
            ..Default::default()
        };
        let c = C2s::PlayerZone(empty).encode();
        assert_eq!(&c[3..], &[0, 0, 0, 0, 0, 0, 0]);
    }

    /// `41 ItemRotationAndAnimation` (`NetMessage.cs:956-960`):
    /// `(byte)number + itemRotation (float) + (short)itemAnimation` —
    /// seven bytes, both directions (`MessageBuffer.cs:2271-2289`).
    #[test]
    fn w4_d96_item_anim_round_trips() {
        let a = ItemAnim {
            player_id: 2,
            rotation: -0.75,
            animation: 18,
        };
        let mut body = vec![2];
        body.extend_from_slice(&(-0.75f32).to_le_bytes());
        body.extend_from_slice(&18i16.to_le_bytes());
        let c = C2s::ItemAnim(a).encode();
        assert_eq!(c[2], 41);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ItemAnim(d) => assert_eq!(d, a),
            other => panic!("expected 41, got {:?}", other.msg_id()),
        }
        let f = S2c::ItemAnim(a).encode();
        assert_eq!(f[2], 41);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemAnim(d) => assert_eq!(d, a),
            other => panic!("expected 41, got {:?}", other.msg_id()),
        }
    }

    /// `43 ManaEffect` (`NetMessage.cs:966-969`): `(byte)number +
    /// (short)number2`, both directions (`MessageBuffer.cs:2307-2324`) —
    /// `35 PlayerHeal`'s mana twin, byte-for-byte the same shape.
    #[test]
    fn w4_d96_mana_effect_round_trips() {
        let m = ManaEffect {
            player_id: 5,
            amount: 60,
        };
        let mut body = vec![5];
        body.extend_from_slice(&60i16.to_le_bytes());
        let c = C2s::ManaEffect(m).encode();
        assert_eq!(c[2], 43);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ManaEffect(d) => assert_eq!(d, m),
            other => panic!("expected 43, got {:?}", other.msg_id()),
        }
        let f = S2c::ManaEffect(m).encode();
        assert_eq!(f[2], 43);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ManaEffect(d) => assert_eq!(d, m),
            other => panic!("expected 43, got {:?}", other.msg_id()),
        }
    }

    /// `16 PlayerLifeMana` (`NetMessage.cs:519-523`): the writer sends
    /// `(byte)number + (short)statLife + (short)statLifeMax` RAW; the read
    /// side floors `statLifeMax` at 20 (`MessageBuffer.cs:1107-1110`) and
    /// leaves `statLife` untouched (`dead = statLife <= 0` is the apply
    /// arm's derivation, `:1111`).
    #[test]
    fn w4_d96_player_hp_read_floors_the_max() {
        let raw = C2s::PlayerHp {
            player_id: 2,
            hp: -5,
            max_hp: 6,
        };
        let mut body = vec![2];
        body.extend_from_slice(&(-5i16).to_le_bytes());
        body.extend_from_slice(&6i16.to_le_bytes());
        let c = raw.encode();
        assert_eq!(c[2], 16);
        assert_eq!(&c[3..], &body, "the writer does not clamp");
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerHp {
                player_id,
                hp,
                max_hp,
            } => {
                assert_eq!(player_id, 2);
                assert_eq!(hp, -5, "statLife rides raw; dead is derived on apply");
                assert_eq!(max_hp, 20, "statLifeMax < 20 floors to 20 (:1107-1110)");
            }
            other => panic!("expected 16, got {:?}", other.msg_id()),
        }
        // The S2c relay arm clamps identically.
        let f = S2c::PlayerHp {
            player_id: 2,
            hp: 90,
            max_hp: 19,
        }
        .encode();
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerHp { max_hp, hp, .. } => {
                assert_eq!(max_hp, 20);
                assert_eq!(hp, 90);
            }
            other => panic!("expected 16, got {:?}", other.msg_id()),
        }
        // A legal max rides untouched.
        let ok = C2s::PlayerHp {
            player_id: 0,
            hp: 400,
            max_hp: 500,
        };
        let c = ok.encode();
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerHp {
                player_id,
                hp,
                max_hp,
            } => assert_eq!((player_id, hp, max_hp), (0, 400, 500)),
            other => panic!("expected 16, got {:?}", other.msg_id()),
        }
    }

    /// `13 PlayerControls`' conditional tails (`NetMessage.cs:492-508`):
    /// velocity (b2) then `(ushort)mount.Type` (b7) then the potion use+home
    /// Vector2 pair (c6) then the camera Vector2 (d5) — in that gate order,
    /// ahead of the port's render extension. The reader consumes them at
    /// the same positions (`MessageBuffer.cs:993-997`, `:1021-1038`, `:1051`).
    #[test]
    fn w4_d96_player_state_conditional_tails_verbatim() {
        let ps = PlayerState {
            player_id: 3,
            flags_a: BitsByte(0b0100_0000),
            flags_b: BitsByte(0b1000_0100), // velocity + mount
            flags_c: BitsByte(0b0100_0000), // potion return
            flags_d: BitsByte(0b0010_0000), // camera target
            selected_item: 2,
            x: 64.0,
            y: 96.0,
            vx: 1.5,
            vy: -2.5,
            mount_type: 12,
            potion_use: (10.0, 20.0),
            potion_home: (30.0, 40.0),
            camera_target: (50.0, 60.0),
            on_ground: true,
            leg_frame: 4,
            body_frame: 9,
            hp: 80,
            max_hp: 100,
        };
        let mut body = vec![3, 0x40, 0x84, 0x40, 0x20, 2];
        for v in [64.0f32, 96.0, 1.5, -2.5] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        body.extend_from_slice(&12u16.to_le_bytes()); // mount (b7)
        for v in [10.0f32, 20.0, 30.0, 40.0] {
            body.extend_from_slice(&v.to_le_bytes()); // potion pair (c6)
        }
        for v in [50.0f32, 60.0] {
            body.extend_from_slice(&v.to_le_bytes()); // camera (d5)
        }
        // The port's render extension rides AFTER the C# tails.
        body.push(1); // on_ground
        body.extend_from_slice(&4u32.to_le_bytes());
        body.extend_from_slice(&9u32.to_le_bytes());
        body.extend_from_slice(&80i32.to_le_bytes());
        body.extend_from_slice(&100i32.to_le_bytes());
        let c = C2s::PlayerState(ps).encode();
        assert_eq!(c[2], 13);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerState(d) => assert_eq!(d, ps),
            other => panic!("expected 13, got {:?}", other.msg_id()),
        }
        // All gates clear: the C# body is 15 bytes (id + 4 flags + selected +
        // position), the extension follows immediately.
        let bare = PlayerState {
            player_id: 1,
            x: 5.0,
            y: 6.0,
            ..Default::default()
        };
        let c = C2s::PlayerState(bare).encode();
        let mut body = vec![1, 0, 0, 0, 0, 0];
        body.extend_from_slice(&5.0f32.to_le_bytes());
        body.extend_from_slice(&6.0f32.to_le_bytes());
        assert_eq!(&c[3..3 + body.len()], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerState(d) => assert_eq!(d, bare),
            other => panic!("expected 13, got {:?}", other.msg_id()),
        }
    }

    /// `4 SyncPlayer`'s difficulty bits (`NetMessage.cs:166-179`): 1 → bit 0,
    /// 2 → bit 1, 3 → bit **3**; bit 2 is `extraAccessory`. The reader maps
    /// them back the same way (`MessageBuffer.cs:323-341`).
    #[test]
    fn w4_d96_player_info_difficulty_bit_three() {
        let mut pi = PlayerInfo::with_name(1, "Jour");
        pi.flags1 = BitsByte(0b0000_1100); // difficulty 3 (bit 3) + extraAccessory (bit 2)
        let body = pi.write_to(NetWriter::open(id::PLAYER_INFO)).finish();
        // flags1 is the third-from-last byte (flags2, flags3 follow).
        let f1 = body[body.len() - 3];
        assert_eq!(f1, 0b0000_1100);
        let back = PlayerInfo::read_from(&mut NetReader::new(&body[3..])).expect("decode");
        assert!(back.flags1.get(3), "difficulty 3 rides bit 3, not bit 2");
        assert!(back.flags1.get(2), "bit 2 is extraAccessory");
        assert!(!back.flags1.get(0) && !back.flags1.get(1));
    }

    /// `36 SyncPlayerZone` (`NetMessage.cs:933-944`): byte player id, the
    /// five `BitsByte` zone words as their implicit bytes, `(byte)townNPCs`
    /// — the host relays and latches zone5[0]'s rising edge into
    /// `SpawnFaelings` (`MessageBuffer.cs:2186`/`:2195-2199`). The g48
    /// keep-one repointed d98's fixture onto d96's `BitsByte` zone fields
    /// (byte-identical wire).
    #[test]
    fn s1_i_1_player_zone_round_trips() {
        let z = PlayerZone {
            player_id: 1,
            zone1: BitsByte(0b1010_0101),
            zone2: BitsByte(0x00),
            zone3: BitsByte(0xFF),
            zone4: BitsByte(0b0000_0001),
            zone5: BitsByte(0b1000_0000),
            town_npcs: 7,
        };
        let f = C2s::PlayerZone(z).encode();
        assert_eq!(f[2], 36);
        assert_eq!(&f[3..], &[1, 0xA5, 0x00, 0xFF, 0x01, 0x80, 7]);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::PlayerZone(d) => assert_eq!(d, z),
            other => panic!("expected 36, got {:?}", other.msg_id()),
        }
        let f = S2c::PlayerZone(z).encode();
        assert_eq!(f[2], 36);
        assert_eq!(&f[3..], &[1, 0xA5, 0x00, 0xFF, 0x01, 0x80, 7]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerZone(d) => assert_eq!(d, z),
            other => panic!("expected 36, got {:?}", other.msg_id()),
        }
        // The zone5[0] edge bit the host's SpawnFaelings gate reads.
        assert_eq!(z.zone5.0 & 1, 0);
        let edge = PlayerZone { zone5: BitsByte(1), ..z };
        assert_eq!(edge.zone5.0 & 1, 1);
    }

    /// `22 ItemOwner` (`NetMessage.cs:675-685`): short slot, byte
    /// reserved-for, 7-bit hold, byte grab player, 7-bit grab delay, the
    /// position pair — `hold`/`grab_delay` at 300 pin the TWO-byte varint
    /// form (`0xAC 0x02`); the reader mirrors (`MessageBuffer.cs:1552-1562`).
    #[test]
    fn s1_i_1_item_owner_round_trips() {
        let o = ItemOwner {
            slot: 5,
            reserved_for: 2,
            hold: 300,
            grab_delay_player: 255,
            grab_delay: 300,
            x: 16.0,
            y: -32.5,
        };
        let f = S2c::ItemOwner(Box::new(o)).encode();
        assert_eq!(f[2], 22);
        let mut body = Vec::new();
        body.extend_from_slice(&5i16.to_le_bytes());
        body.extend_from_slice(&[2, 0xAC, 0x02, 255, 0xAC, 0x02]);
        body.extend_from_slice(&16.0f32.to_le_bytes());
        body.extend_from_slice(&(-32.5f32).to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemOwner(back) => assert_eq!(*back, o),
            other => panic!("expected 22, got {:?}", other.msg_id()),
        }
        // The one-byte varint form: hold 15 / grab delay 100 (`:681`/`:683`).
        let small = ItemOwner { hold: 15, grab_delay: 100, ..o };
        let f = S2c::ItemOwner(Box::new(small)).encode();
        assert_eq!(&f[5..9], &[2, 15, 255, 100]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemOwner(back) => assert_eq!(*back, small),
            other => panic!("expected 22, got {:?}", other.msg_id()),
        }
    }

    /// `31 RequestChestOpen` (`NetMessage.cs:860-863`): two shorts, the
    /// chest tile's top-left; the host reads them at
    /// `MessageBuffer.cs:1873-1874`. Client→host only.
    #[test]
    fn s1_i_1_chest_open_request_round_trips() {
        let q = ChestOpenRequest { x: 100, y: -200 };
        let f = C2s::ChestOpenRequest(q).encode();
        assert_eq!(f[2], 31);
        let mut body = Vec::new();
        body.extend_from_slice(&100i16.to_le_bytes());
        body.extend_from_slice(&(-200i16).to_le_bytes());
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::ChestOpenRequest(d) => assert_eq!(d, q),
            other => panic!("expected 31, got {:?}", other.msg_id()),
        }
    }

    /// `32 SyncChestItem` (`NetMessage.cs:864-877`): short chest, byte slot,
    /// short stack, byte prefix, short type — the stack comes BEFORE the
    /// prefix, the type last (the name-null zeroing is the writer's job).
    /// Both directions share the layout (`MessageBuffer.cs:1899-1903`).
    #[test]
    fn s1_i_1_chest_item_round_trips() {
        let c = ChestItem { chest: 2, slot: 9, stack: 250, prefix: 81, kind: 757 };
        let mut body = Vec::new();
        body.extend_from_slice(&2i16.to_le_bytes());
        body.push(9);
        body.extend_from_slice(&250i16.to_le_bytes());
        body.push(81);
        body.extend_from_slice(&757i16.to_le_bytes());
        let f = C2s::ChestItem(c).encode();
        assert_eq!(f[2], 32);
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::ChestItem(d) => assert_eq!(d, c),
            other => panic!("expected 32, got {:?}", other.msg_id()),
        }
        let f = S2c::ChestItem(c).encode();
        assert_eq!(f[2], 32);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ChestItem(d) => assert_eq!(d, c),
            other => panic!("expected 32, got {:?}", other.msg_id()),
        }
    }

    /// `33 SyncPlayerChest` (`NetMessage.cs:879-911`,
    /// `MessageBuffer.cs:1920-1937`): three shorts + the length byte; a
    /// `1..=20` name rides `BinaryWriter.Write(string)` (7-bit length +
    /// UTF-8), `0` writes no text, `255` the refusal. The reader zeroes any
    /// other length (`num24 = 0`, `:1934`) and re-derives the accepted
    /// length from the read text.
    #[test]
    fn s1_i_1_player_chest_round_trips() {
        let c = PlayerChest { chest: 3, x: 100, y: 200, name_len: 5, name: "chest".into() };
        let mut body = Vec::new();
        body.extend_from_slice(&3i16.to_le_bytes());
        body.extend_from_slice(&100i16.to_le_bytes());
        body.extend_from_slice(&200i16.to_le_bytes());
        body.push(5);
        body.push(5); // the string's own 7-bit length byte
        body.extend_from_slice(b"chest");
        let f = C2s::PlayerChest(c.clone()).encode();
        assert_eq!(f[2], 33);
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::PlayerChest(d) => assert_eq!(d, c),
            other => panic!("expected 33, got {:?}", other.msg_id()),
        }
        let f = S2c::PlayerChest(c.clone()).encode();
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerChest(d) => assert_eq!(d, c),
            other => panic!("expected 33, got {:?}", other.msg_id()),
        }
        // `0`/`255` write NO text after the length byte (`:906-910`).
        for len in [0u8, 255] {
            let bare = PlayerChest { name_len: len, name: String::new(), ..c.clone() };
            let f = C2s::PlayerChest(bare.clone()).encode();
            assert_eq!(f.len(), 3 + 7);
            assert_eq!(f[9], len);
            match C2s::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                C2s::PlayerChest(d) => assert_eq!(d, bare),
                other => panic!("expected 33, got {:?}", other.msg_id()),
            }
        }
        // A stray length (e.g. 21) reads as "no name", `num24 = 0`
        // (`MessageBuffer.cs:1932-1935`); 255 keeps its refusal value.
        let mut stray = Vec::new();
        stray.extend_from_slice(&3i16.to_le_bytes());
        stray.extend_from_slice(&100i16.to_le_bytes());
        stray.extend_from_slice(&200i16.to_le_bytes());
        stray.push(21);
        match C2s::decode(33, &mut NetReader::new(&stray)).expect("decode") {
            C2s::PlayerChest(d) => {
                assert_eq!(d.name_len, 0);
                assert_eq!(d.name, "");
            }
            other => panic!("expected 33, got {:?}", other.msg_id()),
        }
    }

    /// `80 SyncPlayerChestIndex` (`NetMessage.cs:1175-1178`): byte player
    /// id + short chest index; the client's `-3..8000` range gate lives in
    /// the apply arm (`MessageBuffer.cs:3221`, netglue). Host→client only.
    #[test]
    fn s1_i_1_player_chest_index_round_trips() {
        for chest in [-3i16, -1, 0, 7999] {
            let ci = PlayerChestIndex { player_id: 3, chest };
            let f = S2c::PlayerChestIndex(ci).encode();
            assert_eq!(f[2], 80);
            let mut body = vec![3u8];
            body.extend_from_slice(&chest.to_le_bytes());
            assert_eq!(&f[3..], &body);
            match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::PlayerChestIndex(d) => assert_eq!(d, ci),
                other => panic!("expected 80, got {:?}", other.msg_id()),
            }
        }
    }
}
