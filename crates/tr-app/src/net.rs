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

    // ---- S1-I-1 W4-f21: the net-id TAIL census (MessageID 1-162 vs net.rs +
    // the d89-d100/c27 lanes). Bodies + codecs only; the dispatcher apply/relay
    // arms are the g48-g52 apply-wiring lanes / S1-I-4. ----
    /// `24 UnusedMeleeStrike` (`MessageID.cs:57`) — DEAD: no `SendData(24)`
    /// call site anywhere; the writer arm exists (`NetMessage.cs:765-768`,
    /// `(short)npc + (byte)player`) but the reader asserts false
    /// (`MessageBuffer.cs:1706-1708`, `Invariant.Assert(false,
    /// "UnusedMeleeStrike")`) and reads NOTHING — there is no live reader
    /// codec to port. d91 censused it dead; constant + census doc only, the
    /// writer bytes recorded above verbatim (no variant is invented for an
    /// asserting reader).
    pub const UNUSED_MELEE_STRIKE: u8 = 24;
    /// `25 Unused25` (`MessageID.cs:59-60`, `[Old("Deprecated. Use
    /// NetTextModule instead.")]`) — no writer arm, no body; the reader's
    /// ignore group (`MessageBuffer.cs:4483-4490`, shared with
    /// 15/26/44/67/83/93) accepts and drops it.
    pub const UNUSED_25: u8 = 25;
    /// `26 Unused26` (`MessageID.cs:62-63`, `[Old("Deprecated.")]`) — no
    /// writer arm, no body; the same reader ignore group
    /// (`MessageBuffer.cs:4483-4490`).
    pub const UNUSED_26: u8 = 26;
    /// `44 Unknown44` (`MessageID.cs:99-100`, `[Old("Deprecated.")]`) — no
    /// writer arm, no body; the same reader ignore group
    /// (`MessageBuffer.cs:4483-4490`). d96's census row confirms it dead.
    pub const UNKNOWN_44: u8 = 44;
    /// `39 ReleaseItemOwnership` (`MessageID.cs:89`) — live: writer
    /// `NetMessage.cs:948-951`, reader
    /// `MessageBuffer.cs:2232-2255`; the client releases its reservation
    /// (`WorldItem.cs:256`), the host re-assigns and may answer with `22`
    /// (`WorldItem.cs:264` sends it to one client).
    pub const RELEASE_ITEM_OWNERSHIP: u8 = 39;
    /// `48 LiquidUpdate` (`MessageID.cs:108-109`, `[Old("Deprecated. Use
    /// NetLiquidModule instead.")]`) — the per-tile liquid send still
    /// exists: writer `NetMessage.cs:987-995`, reader
    /// `MessageBuffer.cs:2407-2446` (host `SquareTileFrame` + a `48` re-push
    /// when the liquid drained to 0, `:2441`); `NetMessage.sendWater`
    /// (`:2778-2797`) is the send site. The port's liquid lane uses module
    /// 82; this is the verbatim legacy body.
    pub const LIQUID_UPDATE: u8 = 48;
    /// `52 LockAndUnlock` (`MessageID.cs:117`) — writer
    /// `NetMessage.cs:1014-1018` (`(byte)action + (short)x + (short)y`,
    /// the player id `number` is NOT written); reader
    /// `MessageBuffer.cs:2531-2564` is host-only: kind 1 `Chest.Unlock`,
    /// 2 `WorldGen.UnlockDoor`, 3 `Chest.Lock`, each relayed as `52` +
    /// `SendTileSquare(…, 2)`. All send sites are `netMode == 1`
    /// (`Player.cs:32813`/`:32832`/`:33197`/`:33218`/`:41660`) — C2s-only.
    pub const LOCK_AND_UNLOCK: u8 = 52;
    /// `57 Unknown57` (`MessageID.cs:127`) — the world-evil census push:
    /// writer `NetMessage.cs:1054-1058` (`WorldGen.tGood/tEvil/tBlood`,
    /// three `byte`s, `WorldGen.cs:4180-4184`); reader
    /// `MessageBuffer.cs:2631-2638` is client-only. Sends: the join push
    /// (`MessageBuffer.cs:867`) + `WorldGen.cs:71173` — S2c-only.
    pub const WORLD_EVIL_BYTES: u8 = 57;
    /// `58 InstrumentSound` (`MessageID.cs:129`) — writer
    /// `NetMessage.cs:1059-1062` (`(byte)player + float pitch`); reader
    /// `MessageBuffer.cs:2639-2681`: the host forces the id to the sender
    /// and relays, the client plays the held instrument's sound
    /// (`Player.cs:46544`/`:46573`/`:46587`/`:46602`/`:46622` send sites).
    pub const INSTRUMENT_SOUND: u8 = 58;
    /// `76 QuestsCountSync` (`MessageID.cs:165`) — writer
    /// `NetMessage.cs:1149-1153` (`(byte)player + int anglerQuestsFinished
    /// + int golferScoreAccumulated`); reader
    /// `MessageBuffer.cs:3164-3182`: the host forces the id to the sender
    /// and relays (`:3178`, the only live send site), the client stores a
    /// peer's counts when `id != myPlayer || ServerSideCharacter`.
    pub const QUESTS_COUNT_SYNC: u8 = 76;
    /// `67 Unknown67` (`MessageID.cs:147`) — no writer arm, no body; the
    /// reader ignore group (`MessageBuffer.cs:4483-4490`). Note the accept
    /// gate (`:165`) does NOT whitelist it pre-spawn (93 is whitelisted
    /// there but still ignored by the reader group).
    pub const UNKNOWN_67: u8 = 67;
    /// `83 Unused83` (`MessageID.cs:179-180`, `[Old("Deprecated.")]`) — no
    /// writer arm, no body; the same reader ignore group
    /// (`MessageBuffer.cs:4483-4490`).
    pub const UNUSED_83: u8 = 83;
    /// `79 PlaceObject` (`MessageID.cs:171`) — writer
    /// `NetMessage.cs:1166-1174`, reader `MessageBuffer.cs:3192-3215`
    /// (host spam count + `InWorld(…, 10)` + section-sent gate, then
    /// `WorldGen.PlaceObject` + `SendObjectPlacement` relay).
    /// `SendObjectPlacement` (`NetMessage.cs:2573-2588`) sends it BOTH
    /// ways: client placement asks (`Player.cs:40147`/`:45819`,
    /// `Projectile.cs:25356`) and the host's placement relay.
    pub const PLACE_OBJECT: u8 = 79;
    /// `85 QuickStackChests` (`MessageID.cs:184`) — writer
    /// `NetMessage.cs:1205-1215` is DIRECTION-SPLIT: `netMode == 1` writes
    /// the inventory-slot list + a smart-stack byte
    /// (`QuickStacking.WriteNetInventorySlots`, `QuickStacking.cs:299-306`
    /// + `:296`'s send), the host writes the blocked-chest list
    /// (`WriteBlockedChestList`, `:323-330` + `:277`'s send). Reader
    /// `MessageBuffer.cs:3261-3271` mirrors the split.
    pub const QUICK_STACK_CHESTS: u8 = 85;
    /// `86 TileEntitySharing` (`MessageID.cs:186`) — writer
    /// `NetMessage.cs:1216-1226` (`int id + bool exists`, then
    /// `TileEntity.Write(networkSend: true)`), reader
    /// `MessageBuffer.cs:3272-3293` (client-only: `false` removes the
    /// entity, `true` reads it via `TileEntity.Read(reader, 326, true)` and
    /// re-adds under the frame's id). Host sends: the placement hooks
    /// (`TileEntityType.cs:19`, `TELeashedEntityAnchor.cs:11`) + the
    /// training-dummy (de)activation (`TETrainingDummy.cs:143`/`:156`).
    pub const TILE_ENTITY_SHARING: u8 = 86;
    /// `87 TileEntityPlacement` (`MessageID.cs:188`) — writer
    /// `NetMessage.cs:1227-1231` (`(short)x + (short)y + (byte)type`);
    /// reader `MessageBuffer.cs:3294-3305` is host-only:
    /// `InWorld + !TryGetAt` gate then `TileEntity.PlaceEntityNet`. All
    /// send sites are the client placement hooks (`TEItemFrame.cs:25`
    /// etc.) — C2s-only.
    pub const TILE_ENTITY_PLACEMENT: u8 = 87;
    /// `88 ItemTweaker` (`MessageID.cs:190`) — writer
    /// `NetMessage.cs:1232-1296` (the two-BitsByte conditional stat
    /// override), reader `MessageBuffer.cs:3306-3376` (client-only, the
    /// `0..=400` slot gate). Host sends: the drop-rule tweaks
    /// (`CommonCode.cs:67-92`). S2c-only.
    pub const ITEM_TWEAKER: u8 = 88;
    /// `89 ItemFrameTryPlacing` (`MessageID.cs:192`) — writer
    /// `NetMessage.cs:1297-1306`, reader `MessageBuffer.cs:3377-3393`
    /// (host-only `TEItemFrame.TryPlacing` under
    /// `DefaultAssignNewItemsToPlayer`). Client send: `TEItemFrame.cs:150`.
    /// C2s-only. The 123/133/149 twins share this exact body.
    pub const ITEM_FRAME_TRY_PLACING: u8 = 89;
    /// `90 SpawnInstancedItem` (`MessageID.cs:194`) — the writer SHARES
    /// case 21 (`NetMessage.cs:646-669`): the exact `ItemState` body under
    /// a different id. Host-only send: `WorldItem.MakeInstanced`
    /// (`WorldItem.cs:321-336`, per-player `SendData(90, i, …)` then
    /// `TurnToAir`). The reader shares case 21 too
    /// (`MessageBuffer.cs:1445-1536`); the `b == 90` client tail sets
    /// `instanced = true` + reserves for `myPlayer` (`:1489-1493`).
    /// S2c-only. The bit-3 enemyGrabDelay tail is d98's wave-I extension
    /// of the shared `ItemState` body — it merges with this variant.
    pub const SPAWN_INSTANCED_ITEM: u8 = 90;
    /// `93 SocialHandshake` (`MessageID.cs:200`) — no writer arm, no body;
    /// the reader ignore group (`MessageBuffer.cs:4483-4490`). Unlike its
    /// group peers it IS accept-gate-whitelisted pre-spawn
    /// (`MessageBuffer.cs:165`'s `b != 93` escape) — the C# accepts the
    /// frame early, then drops it.
    pub const SOCIAL_HANDSHAKE: u8 = 93;
    /// `94 DevCommands` (`MessageID.cs:202`) — writer
    /// `NetMessage.cs:1327-1332` (`string command + int + float + float`),
    /// reader `MessageBuffer.cs:3463-3518` (the `enableDebugCommands` gate
    /// + the `/showdebug` / `/randomizeprojslots` / `/setserverping` /
    /// `/quickload-*` ladder). Both sides send (`DebugOptions.cs:68-70`,
    /// `QuickLoad.cs:171`, the reader's own relays).
    pub const DEV_COMMANDS: u8 = 94;
    /// `104 ShopOverride` (`MessageID.cs:222`) — writer
    /// `NetMessage.cs:1387-1394`, reader `MessageBuffer.cs:3715-3735`.
    /// VERBATIM QUIRK: the two sides DISAGREE on the third field — the
    /// writer sends `(short)number3 < 0 ? 0f : number3` (a 4-byte FLOAT,
    /// the stack), the reader reads `ReadInt16` (2 bytes) then misaligns
    /// the rest. Zero `SendData(104)` call sites exist tree-wide, so the
    /// mismatch is dead code in vanilla; the port keeps BOTH shapes
    /// verbatim (encode = writer, decode = reader) and documents the
    /// divergence. S2c-only (the reader is `netMode == 1`-gated).
    pub const SHOP_OVERRIDE: u8 = 104;
    /// `105 GemLockToggle` (`MessageID.cs:224`) — writer
    /// `NetMessage.cs:1395-1399` (`(short)x + (short)y + (number3 == 1f)`
    /// bool), reader `MessageBuffer.cs:3736-3741` (`netMode != 1` →
    /// `WorldGen.ToggleGemLock`). Client sends: `Player.cs:32547`/`:32559`.
    /// C2s-only in practice; the reader accepts on host AND single-player.
    pub const GEM_LOCK_TOGGLE: u8 = 105;
    /// `107 SmartTextMessage` (`MessageID.cs:228`) — writer
    /// `NetMessage.cs:769-775` (`(byte)r + (byte)g + (byte)b +
    /// NetworkText + (short)widthLimit`), reader
    /// `MessageBuffer.cs:3752-3757` (client-only `NewTextMultiline`).
    /// Sends: the sign-read wiring messages (`Wiring.cs:1208`/`:1228`,
    /// width 460). S2c-only.
    pub const SMART_TEXT_MESSAGE: u8 = 107;
    /// `111 ToggleParty` (`MessageID.cs:236`) — NO writer case at all
    /// (`NetMessage.cs`'s switch skips 111): `SendData(111)` writes a bare
    /// 3-byte frame. The only send site is the client's manual-party
    /// toggle (`BirthdayParty.cs:154`, `netMode == 1`); the reader is
    /// host-only `BirthdayParty.ToggleManualParty()`
    /// (`MessageBuffer.cs:3809-3814`). C2s-only, bodyless.
    pub const TOGGLE_PARTY: u8 = 111;
    /// `119 CombatTextString` (`MessageID.cs:252`) — writer
    /// `NetMessage.cs:1188-1196` (`float x + float y + WriteRGB +
    /// NetworkText`), reader `MessageBuffer.cs:3235-3242` (client-only
    /// `CombatText.NewText` with the `(int)` float truncation). The 81
    /// CombatTextInt twin (d98) carries an int amount instead of the
    /// text. No live `SendData(119)` site tree-wide — the writer arm is
    /// the census record. S2c-only.
    pub const COMBAT_TEXT_STRING: u8 = 119;
    /// `121 TEDisplayDollDataSync` (`MessageID.cs:256`) — writer
    /// `NetMessage.cs:1458-1474` (`(byte)player + int entityId +
    /// (byte)itemIndex + (byte)command` + the doll's `WriteData` or the
    /// dummy fallback), reader `MessageBuffer.cs:3920-3941` (host id-force
    /// + relay). Client sends: `ItemSlot.cs:1109-1762`, the doll's own
    /// pose sync `TEDisplayDoll.cs:907`.
    pub const TE_DISPLAY_DOLL_SYNC: u8 = 121;
    /// `122 RequestTileEntityInteraction` (`MessageID.cs:258`) — writer
    /// `NetMessage.cs:1475-1478` (`int entityId + (byte)player`), reader
    /// `MessageBuffer.cs:3942-3977` (host: the -1 clear / the
    /// `IsOccupied`-gated anchor set + relay; client: the anchor
    /// clear/set). Client sends: `TileEntity.cs:280`/`:285`,
    /// `Main.cs:17605`.
    pub const TE_INTERACTION_REQUEST: u8 = 122;
    /// `123 WeaponsRackTryPlacing` (`MessageID.cs:260`) — writer
    /// `NetMessage.cs:1479-1488` (the exact 89 body), reader
    /// `MessageBuffer.cs:3978-3994` (host-only `TEWeaponsRack.TryPlacing`).
    /// Client send: `TEWeaponsRack.cs:212`. C2s-only.
    pub const WEAPONS_RACK_TRY_PLACING: u8 = 123;
    /// `124 TEHatRackItemSync` (`MessageID.cs:262`) — writer
    /// `NetMessage.cs:1489-1508` (`(byte)player + int entityId +
    /// (byte)slot` with the DYE +2 fold, then `TEHatRack.WriteItem` or the
    /// `int 0 + byte 0` fallback), reader `MessageBuffer.cs:3995-4022`
    /// (the `>= 2` dye unfold + the `>= 2` slot refusal). Client sends:
    /// `ItemSlot.cs:1117`/`:1216`/`:1766`/`:1770`, `TEHatRack.cs:322`/`:368`.
    pub const TE_HAT_RACK_ITEM_SYNC: u8 = 124;
    /// `129 FinishedConnectingToServer` (`MessageID.cs:272`) — NO writer
    /// case: `TrySendData(129, whoAmI)` (`MessageBuffer.cs:937`, the
    /// State-3→10 join arm) writes a bare frame. The reader is client-only
    /// (`MessageBuffer.cs:4075-4085`): the team re-push (`45` when
    /// `team > 0`), `FixUIScale`, `TrySetPreparationState(ProcessingData)`.
    /// S2c-only, bodyless.
    pub const FINISHED_CONNECTING: u8 = 129;
    /// `133 FoodPlatterTryPlacing` (`MessageID.cs:280`) — writer
    /// `NetMessage.cs:1545-1554` (the exact 89 body), reader
    /// `MessageBuffer.cs:4147-4163` (host-only `TEFoodPlatter.TryPlacing`).
    /// Client send: `TEFoodPlatter.cs:133`. C2s-only.
    pub const FOOD_PLATTER_TRY_PLACING: u8 = 133;

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

// ---- S1-I-1 W4-f21: the net-id tail census bodies ----

/// `msg 39 ReleaseItemOwnership`'s body (`NetMessage.cs:948-951`): the item
/// slot `(short)` then `number2 != 0f` as a bool (`forceAssignToServer`).
/// The reader (`MessageBuffer.cs:2232-2255`) releases the reservation when
/// it belongs to the reader's player and calls `FindOwner`; the host may
/// answer with a `22 ItemOwner` re-announcement (`:2251`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReleaseItemOwnership {
    pub slot: i16,
    pub force_assign_to_server: bool,
}

/// `msg 48 LiquidUpdate`'s body (`NetMessage.cs:987-995`): the tile as two
/// `(short)`s, `tile.liquid` byte and `tile.liquidType()` byte. The reader
/// (`MessageBuffer.cs:2407-2446`) applies both, runs the host spam check
/// (`:2413-2427`, the ±10-tile window around the sender's centre) and
/// re-frames; a drained tile (`b2 == 0`) is re-pushed as `48` (`:2441`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiquidUpdate {
    pub x: i16,
    pub y: i16,
    pub liquid: u8,
    /// `Tile.liquidType()` (`Tile.cs:235`): 0 water, 1 lava, 2 honey,
    /// 3 shimmer.
    pub liquid_type: u8,
}

/// `msg 52 LockAndUnlock`'s body (`NetMessage.cs:1014-1018`): the action
/// byte (`number2`), the tile as two `(short)`s (`number3`/`number4`). The
/// reader (`MessageBuffer.cs:2531-2564`) maps 1 chest-unlock, 2 door-unlock,
/// 3 chest-lock; any other kind is a silent no-op. The host relays with
/// `number = 0` (`:2541`/`:2550`/`:2559`) — the player id never rides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockAndUnlock {
    /// 1 = `Chest.Unlock`, 2 = `WorldGen.UnlockDoor`, 3 = `Chest.Lock`.
    pub action: u8,
    pub x: i16,
    pub y: i16,
}

/// `msg 57 Unknown57`'s body (`NetMessage.cs:1054-1058`): the three
/// world-evil bytes `WorldGen.tGood`, `tEvil`, `tBlood` in that order
/// (writer order good→evil→blood; the reader assigns the same order,
/// `MessageBuffer.cs:2634-2636`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldEvilBytes {
    pub good: u8,
    pub evil: u8,
    pub blood: u8,
}

/// `msg 58 InstrumentSound`'s body (`NetMessage.cs:1059-1062`): the player
/// id byte then the pitch float (`number2`, the raw `writer.Write(number2)`
/// — no cast). The reader (`MessageBuffer.cs:2639-2681`) forces the id to
/// the sender on the host and relays; the client's switch plays the guitar
/// chords (4057/4372/4715), drums (4673) or the item sound (507 → Item35,
/// 1305 → Item47, else Item26) with `Main.musicPitch = pitch`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstrumentSound {
    pub player_id: u8,
    pub pitch: f32,
}

/// `msg 76 QuestsCountSync`'s body (`NetMessage.cs:1149-1153`): the player
/// id byte, `anglerQuestsFinished` int, `golferScoreAccumulated` int. The
/// reader (`MessageBuffer.cs:3164-3182`) skips its own echo unless
/// `ServerSideCharacter`, forces the id to the sender on the host and
/// relays to the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestsCountSync {
    pub player_id: u8,
    pub angler_quests_finished: i32,
    pub golfer_score: i32,
}

/// `msg 79 PlaceObject`'s body (`NetMessage.cs:1166-1174`): the tile as two
/// `(short)`s, the object `(short)` type, the `(short)` style, the
/// `(byte)` alternative, the `(sbyte)` random and `number7 == 1` as a bool
/// (the reader maps it back to `direction = 1 : -1`,
/// `MessageBuffer.cs:3200`). The host gates on `InWorld(x, y, 10)` + the
/// section having been sent (`:3204`) before `WorldGen.PlaceObject`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaceObject {
    pub x: i16,
    pub y: i16,
    pub kind: i16,
    pub style: i16,
    pub alternative: u8,
    pub random: i8,
    /// The writer's `number7 == 1` bool; the reader's `direction` is
    /// `1` when true, `-1` when false.
    pub direction: bool,
}

/// `msg 85 QuickStackChests`'s DIRECTION-SPLIT body
/// (`NetMessage.cs:1205-1215`): the client→host form is the quick-stack
/// inventory (`QuickStacking.WriteNetInventorySlots`,
/// `QuickStacking.cs:299-306`: an `int` count then that many `(short)`
/// `PlayerItemSlotID.SlotId`s) plus the smart-stack byte; the host→client
/// form is the blocked-chest list (`WriteBlockedChestList`, `:323-330`: an
/// `int` count then that many `(ushort)` chest indices). The two forms are
/// distinct wire shapes keyed on `netMode`, so the port keeps them as two
/// variants rather than one ambiguous body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickStackRequest {
    /// `QuickStacking.netInv.slots[i].SlotId` — the `PlayerItemSlotID`
    /// slot references, in order.
    pub slots: Vec<i16>,
    /// `number == 1` — the smart-stack flag (`QuickStacking.cs:296`).
    pub smart_stack: bool,
}

/// The host→client half of `85` (`NetMessage.cs:1213`,
/// `QuickStacking.WriteBlockedChestList` `:323-330`): the chests the
/// quick-stack was blocked from, as `(ushort)` indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickStackBlockedChests {
    pub chests: Vec<u16>,
}

/// One `(short)type + (byte)prefix + (short)stack` item triple — the shape
/// every tile-entity `WriteExtraData` uses for a non-air slot
/// (`TEItemFrame.cs:42-44` and its five twins).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeItem {
    pub kind: i16,
    pub prefix: u8,
    pub stack: i16,
}

impl TeItem {
    /// The `WriteExtraData` triple order: `(short)type + (byte)prefix +
    /// (short)stack` (`TEItemFrame.cs:42-44` and its twins).
    fn write_to(self, w: NetWriter) -> NetWriter {
        w.i16(self.kind).u8(self.prefix).i16(self.stack)
    }
    fn read_from(r: &mut NetReader) -> Option<Self> {
        Some(Self {
            kind: r.i16()?,
            prefix: r.u8()?,
            stack: r.i16()?,
        })
    }
    /// The `WriteItem` single-slot order: `(ushort)type + (ushort)stack +
    /// (byte)prefix` — STACK BEFORE PREFIX, unlike the extra-data triple
    /// (`TEDisplayDoll.cs:758-764`, `TEHatRack.cs:378-388`; the readers
    /// `:766-770` / `:390-394` match).
    fn write_slot(self, w: NetWriter) -> NetWriter {
        w.u16(self.kind as u16).u16(self.stack as u16).u8(self.prefix)
    }
    fn read_slot(r: &mut NetReader) -> Option<Self> {
        Some(Self {
            kind: r.u16()? as i16,
            stack: r.u16()? as i16,
            prefix: r.u8()?,
        })
    }
    /// C#'s `IsAir` (`Item.cs:385-394`): `type <= 0 || stack <= 0`.
    fn is_air(self) -> bool {
        self.kind <= 0 || self.stack <= 0
    }
}

/// The per-kind extra data of a tile entity on the `networkSend: true`
/// path — the manager's registration order IS the type byte
/// (`TileEntitiesManager.cs:26-39`): 0 TrainingDummy, 1 ItemFrame,
/// 2 LogicSensor, 3 DisplayDoll, 4 WeaponsRack, 5 HatRack, 6 FoodPlatter,
/// 7 TeleportationPylon, 8 DeadCellsDisplayJar, 9 KiteAnchor,
/// 10 CritterAnchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TileEntityData {
    /// 0 — `(short)npc` (`TETrainingDummy.cs:119-127`).
    TrainingDummy { npc: i16 },
    /// 1 — one item triple (`TEItemFrame.cs:40-53`).
    ItemFrame(TeItem),
    /// 2 — NOTHING on the net path: `WriteExtraData` is
    /// `if (!networkSend)`-gated (`TELogicSensor.cs:380-396`).
    LogicSensor,
    /// 3 — the doll's three flag bytes + pose + the non-air triples
    /// (`TEDisplayDoll.cs:157-278`): equip bits 0-7, dye bits 0-7, pose
    /// byte, then bits3 (misc[0], equip[8], dye[8]); the reader folds
    /// bits3[1]/[2] into the 9th equip/dye slots (`:236`/`:248`).
    DisplayDoll {
        equip: [TeItem; 9],
        dyes: [TeItem; 9],
        misc: [TeItem; 1],
        pose: u8,
    },
    /// 4 — one item triple (`TEWeaponsRack.cs:53-66`).
    WeaponsRack(TeItem),
    /// 5 — one flag byte (items 0/1, dyes 0/1) + the non-air triples
    /// (`TEHatRack.cs:56-110`).
    HatRack { items: [TeItem; 2], dyes: [TeItem; 2] },
    /// 6 — one item triple (`TEFoodPlatter.cs:41-54`).
    FoodPlatter(TeItem),
    /// 7 — no `WriteExtraData` override: the base no-op
    /// (`TETeleportationPylon.cs`, `TileEntity.cs:209-215`).
    TeleportationPylon,
    /// 8 — one item triple (`TEDeadCellsDisplayJar.cs:40-53`).
    DeadCellsDisplayJar(TeItem),
    /// 9 — `(short)itemType` (`TEKiteAnchor` inherits
    /// `TELeashedEntityAnchorWithItem.cs:10-18`).
    KiteAnchor { item_type: i16 },
    /// 10 — `(short)itemType` (the same base, `TECritterAnchor.cs:7`).
    CritterAnchor { item_type: i16 },
}

impl TileEntityData {
    /// The manager's type byte (`TileEntitiesManager.RegisterAll` order).
    pub fn type_id(&self) -> u8 {
        match self {
            Self::TrainingDummy { .. } => 0,
            Self::ItemFrame(_) => 1,
            Self::LogicSensor => 2,
            Self::DisplayDoll { .. } => 3,
            Self::WeaponsRack(_) => 4,
            Self::HatRack { .. } => 5,
            Self::FoodPlatter(_) => 6,
            Self::TeleportationPylon => 7,
            Self::DeadCellsDisplayJar(_) => 8,
            Self::KiteAnchor { .. } => 9,
            Self::CritterAnchor { .. } => 10,
        }
    }

    /// `WriteExtraData(writer, networkSend: true)` — the per-kind tail.
    fn write_extra(&self, w: NetWriter) -> NetWriter {
        match self {
            Self::TrainingDummy { npc } => w.i16(*npc),
            Self::ItemFrame(i) | Self::WeaponsRack(i) | Self::FoodPlatter(i) => i.write_to(w),
            Self::DeadCellsDisplayJar(i) => i.write_to(w),
            Self::LogicSensor | Self::TeleportationPylon => w,
            Self::KiteAnchor { item_type } | Self::CritterAnchor { item_type } => w.i16(*item_type),
            Self::DisplayDoll {
                equip,
                dyes,
                misc,
                pose,
            } => {
                // `TEDisplayDoll.cs:159-184`: equip bits, dye bits, pose,
                // then bits3 = misc[0] | equip[8]<<1 | dyes[8]<<2.
                let mut b1 = BitsByte::new();
                let mut b2 = BitsByte::new();
                let mut b3 = BitsByte::new();
                for i in 0..8 {
                    b1.set(i, !equip[i].is_air());
                    b2.set(i, !dyes[i].is_air());
                }
                b3.set(0, !misc[0].is_air());
                b3.set(1, !equip[8].is_air());
                b3.set(2, !dyes[8].is_air());
                let mut w = w.bitsbyte(b1).bitsbyte(b2).u8(*pose).bitsbyte(b3);
                // The triples ride in array order, non-air only
                // (`:185-214`): all 9 equips, all 9 dyes, the 1 misc.
                for i in equip.iter().chain(dyes.iter()).chain(misc.iter()) {
                    if !i.is_air() {
                        w = i.write_to(w);
                    }
                }
                w
            }
            Self::HatRack { items, dyes } => {
                // `TEHatRack.cs:58-83`: one flag byte (items 0/1 then
                // dyes 0/1), then the non-air triples items-first.
                let mut b = BitsByte::new();
                b.set(0, !items[0].is_air());
                b.set(1, !items[1].is_air());
                b.set(2, !dyes[0].is_air());
                b.set(3, !dyes[1].is_air());
                let mut w = w.bitsbyte(b);
                for i in items.iter().chain(dyes.iter()) {
                    if !i.is_air() {
                        w = i.write_to(w);
                    }
                }
                w
            }
        }
    }

    /// `ReadExtraData(reader, gameVersion: 326, networkSend: true)` — the
    /// per-kind read (`MessageBuffer.cs:3288` passes 326, so the doll's
    /// `>= 307`/`>= 308` gates are both live and the `== 311` quirk is
    /// dead). Absent slots decode as air (the C# `new Item()` default).
    fn read_extra(kind: u8, r: &mut NetReader) -> Option<Self> {
        Some(match kind {
            0 => Self::TrainingDummy { npc: r.i16()? },
            1 => Self::ItemFrame(TeItem::read_from(r)?),
            2 => Self::LogicSensor,
            4 => Self::WeaponsRack(TeItem::read_from(r)?),
            6 => Self::FoodPlatter(TeItem::read_from(r)?),
            7 => Self::TeleportationPylon,
            8 => Self::DeadCellsDisplayJar(TeItem::read_from(r)?),
            9 => Self::KiteAnchor {
                item_type: r.i16()?,
            },
            10 => Self::CritterAnchor {
                item_type: r.i16()?,
            },
            3 => {
                let b1 = r.bitsbyte()?;
                let b2 = r.bitsbyte()?;
                let pose = r.u8()?; // gameVersion 326 >= 307
                let b3 = r.bitsbyte()?; // 326 >= 308; the == 311 quirk is dead
                let mut equip = [TeItem::default(); 9];
                let mut dyes = [TeItem::default(); 9];
                let mut misc = [TeItem::default(); 1];
                // `:236`: the equip mask folds b3[1] into slot 8.
                for (i, slot) in equip.iter_mut().enumerate() {
                    if b1.get(i) || (i == 8 && b3.get(1)) {
                        *slot = TeItem::read_from(r)?;
                    }
                }
                // `:248`: the dye mask folds b3[2] into slot 8.
                for (i, slot) in dyes.iter_mut().enumerate() {
                    if b2.get(i) || (i == 8 && b3.get(2)) {
                        *slot = TeItem::read_from(r)?;
                    }
                }
                if b3.get(0) {
                    misc[0] = TeItem::read_from(r)?;
                }
                Self::DisplayDoll {
                    equip,
                    dyes,
                    misc,
                    pose,
                }
            }
            5 => {
                let b = r.bitsbyte()?;
                let mut items = [TeItem::default(); 2];
                let mut dyes = [TeItem::default(); 2];
                for (i, slot) in items.iter_mut().enumerate() {
                    if b.get(i) {
                        *slot = TeItem::read_from(r)?;
                    }
                }
                for (i, slot) in dyes.iter_mut().enumerate() {
                    if b.get(i + 2) {
                        *slot = TeItem::read_from(r)?;
                    }
                }
                Self::HatRack { items, dyes }
            }
            _ => return None, // InvalidEntityID (TileEntitiesManager.cs:17-24)
        })
    }
}

/// `msg 86 TileEntitySharing`'s body (`NetMessage.cs:1216-1226`): the
/// entity `int` id, the `TryGet` bool, and — when present —
/// `TileEntity.Write(networkSend: true)` (`TileEntity.cs:173-197`): the
/// type byte, the `(short)` position pair (the id is NOT re-written on the
/// net path), then the per-kind extra data. `exists == false` is the
/// removal frame (the reader's `TileEntity.Remove`,
/// `MessageBuffer.cs:3279-3285`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileEntitySharing {
    pub id: i32,
    pub x: i16,
    pub y: i16,
    /// `None` = the removal frame (`flag2 == false`).
    pub entity: Option<TileEntityData>,
}

/// `msg 87 TileEntityPlacement`'s body (`NetMessage.cs:1227-1231`): the
/// tile as two `(short)`s and the entity type byte (the manager's
/// registration id). The host validates the tile + emptiness before
/// `PlaceEntityNet` (`MessageBuffer.cs:3300-3303`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileEntityPlacement {
    pub x: i16,
    pub y: i16,
    pub entity_type: u8,
}

/// `msg 88 ItemTweaker`'s body (`NetMessage.cs:1232-1296`): the item slot
/// `(short)`, then two flag bytes gating the stat overrides. BitsByte 1:
/// 0 color (`uint` packed), 1 damage (`ushort`), 2 knockback (`float`),
/// 3 useAnimation (`ushort`), 4 useTime (`ushort`), 5 shoot (`short`),
/// 6 shootSpeed (`float`), 7 = the second BitsByte rides. BitsByte 2:
/// 0 width (`ushort` writer / `short` reader — the wire is two bytes
/// either way), 1 height (same), 2 scale (`float`), 3 ammo (`short`),
/// 4 useAmmo (`short`), 5 notAmmo (`bool`). Absent fields stay unset.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemTweaker {
    pub slot: i16,
    pub color: Option<u32>,
    pub damage: Option<u16>,
    pub knockback: Option<f32>,
    pub use_animation: Option<u16>,
    pub use_time: Option<u16>,
    pub shoot: Option<i16>,
    pub shoot_speed: Option<f32>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub scale: Option<f32>,
    pub ammo: Option<i16>,
    pub use_ammo: Option<i16>,
    pub not_ammo: Option<bool>,
}

impl ItemTweaker {
    /// The second BitsByte rides only when at least one of its six fields
    /// is set (the writer's `bitsByte[7]` gate, `NetMessage.cs:1267`).
    fn has_second(&self) -> bool {
        self.width.is_some()
            || self.height.is_some()
            || self.scale.is_some()
            || self.ammo.is_some()
            || self.use_ammo.is_some()
            || self.not_ammo.is_some()
    }
}

/// `msg 89 ItemFrameTryPlacing`'s body (`NetMessage.cs:1297-1306`): the
/// tile as two `(short)`s, the held item's `(short)` type, its prefix
/// byte, and the `(short)` stack (`number5` — always 1 at the send site,
/// `TEItemFrame.cs:150`). The player/slot indices (`number4`/`number3`)
/// select the item on the WRITER side only; they never ride the wire.
/// The 123 WeaponsRack / 133 FoodPlatter / 149 DeadCellsJar twins share
/// this exact shape (`NetMessage.cs:1479-1488`/`:1545-1554`/`:1629-1638`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TryPlacingItem {
    pub x: i16,
    pub y: i16,
    pub kind: i16,
    pub prefix: u8,
    pub stack: i16,
}

/// `msg 94 DevCommands`'s body (`NetMessage.cs:1327-1332`): the command
/// string (`text.ToString()` — a plain `BinaryWriter.Write(string)`, NOT
/// a NetworkText), then `number` int, `number2` float, `number3` float.
/// The reader (`MessageBuffer.cs:3465-3468`) reads the string, the int,
/// `(int)ReadSingle()` and DROPS the third float. The debug gate +
/// command ladder are the apply arm's.
#[derive(Clone, Debug, PartialEq)]
pub struct DevCommands {
    pub command: String,
    pub arg_int: i32,
    pub arg_float: f32,
    /// The writer's third float; the reader consumes and discards it
    /// (`MessageBuffer.cs:3468`'s bare `ReadSingle()`).
    pub arg_float2: f32,
}

/// `msg 104 ShopOverride`'s WRITER body (`NetMessage.cs:1387-1394`):
/// `(byte)slot + (short)type + float stack (negative clamped to 0f) +
/// (byte)prefix + int value + (byte)flags`. The READER
/// (`MessageBuffer.cs:3719-3724`) reads `byte + short + SHORT + byte +
/// int + byte` — a 2-byte stack where the writer sends 4. No send site
/// exists, so the port keeps the writer shape for encode and the reader
/// shape for decode, verbatim on both sides (the round trip is NOT
/// byte-identity past the stack field — that is the C# bug, pinned).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShopOverride {
    pub slot: u8,
    pub kind: i16,
    /// The writer's float stack; `(short)stack < 0` writes `0f`
    /// (`NetMessage.cs:1390`'s ternary).
    pub stack: f32,
    pub prefix: u8,
    pub value: i32,
    /// `bitsByte22[0]` is `buyOnce` (`MessageBuffer.cs:3732`).
    pub flags: u8,
}

/// `msg 105 GemLockToggle`'s body (`NetMessage.cs:1395-1399`): the tile
/// as two `(short)`s and `number3 == 1f` as a bool (the gem lock's on/off;
/// `Player.cs:32547` sends 1f, `:32559` the default 0f). The reader calls
/// `WorldGen.ToggleGemLock(x, y, on)` when `netMode != 1`
/// (`MessageBuffer.cs:3737-3740`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GemLockToggle {
    pub x: i16,
    pub y: i16,
    pub on: bool,
}

/// `msg 107 SmartTextMessage`'s body (`NetMessage.cs:769-775`): the colour
/// as three separate `(byte)` casts (r, g, b — the writer does NOT call
/// `WriteRGB`, but the byte order is identical, `Utils.cs:1369-1374`),
/// then the `NetworkText` and the `(short)` width limit. The reader
/// (`MessageBuffer.cs:3752-3757`) reads `ReadRGB` (R, G, B,
/// `Utils.cs:1387-1390`), the text and the limit, and calls
/// `NewTextMultiline(force: false)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmartTextMessage {
    pub color: Rgb,
    pub text: NetText,
    /// `WidthLimit` — 460 at the sign-read send sites
    /// (`Wiring.cs:1208`/`:1228`).
    pub width_limit: i16,
}

/// `msg 119 CombatTextString`'s body (`NetMessage.cs:1188-1196`): the
/// position as two raw floats (`number2`/`number3`), the colour via
/// `WriteRGB` (the `(uint)number` packed value re-expanded), then the
/// `NetworkText`. The reader (`MessageBuffer.cs:3238-3240`) truncates the
/// floats with `(int)` casts into the `Rectangle(x, y, 0, 0)` location —
/// the truncation is the apply arm's; the codec keeps the raw floats.
#[derive(Clone, Debug, PartialEq)]
pub struct CombatTextString {
    pub x: f32,
    pub y: f32,
    pub color: Rgb,
    pub text: NetText,
}

/// The doll-sync tail (`TEDisplayDoll.WriteData`, `TEDisplayDoll.cs:780-800`
/// / `WriteDummySync` `:824-833`): command 2 carries the pose byte;
/// commands 0/1/3 carry one item triple from the equip/dye/misc array.
/// The no-entity fallback (`Dummy`) writes a zero pose byte for command 2
/// and `int 0 + byte 0` otherwise — the SAME byte widths as the real
/// shapes (1, and 4+1 = 2+1+2 = 5), so a dummy frame decodes cleanly as
/// `Pose(0)` / an air `TeItem` and the C# reader's `ReadDummySync`
/// (`:835-844`) likewise just consumes and discards. `Dummy` is therefore
/// a writer-side variant; decode always yields the real-doll shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DollSyncData {
    /// command 2 — the pose byte (`ReadPose`, `:846-849`).
    Pose(u8),
    /// commands 0 (equip) / 1 (dyes) / 3 (misc) — one item triple.
    Item(TeItem),
    /// The no-entity fallback (`WriteDummySync`): command 2 → one zero
    /// byte; else `int 0 + byte 0`. Writer-side only (see above).
    Dummy,
}

/// `msg 121 TEDisplayDollDataSync`'s body (`NetMessage.cs:1458-1474`):
/// `(byte)player + int entityId + (byte)itemIndex + (byte)command`, then
/// the doll's `WriteData(itemIndex, command)` when the entity exists,
/// else `WriteDummySync`. The reader (`MessageBuffer.cs:3920-3941`)
/// forces the player id to the sender on the host and relays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayDollSync {
    pub player_id: u8,
    pub entity_id: i32,
    pub item_index: u8,
    /// 0 equip, 1 dyes, 2 pose, 3 misc (`TEDisplayDoll.cs:782-798`).
    pub command: u8,
    pub data: DollSyncData,
}

/// `msg 122 RequestTileEntityInteraction`'s body
/// (`NetMessage.cs:1475-1478`): the entity `int` id (`-1` = clear the
/// anchor) and the player byte. The host
/// (`MessageBuffer.cs:3950-3963`) forces the player to the sender,
/// refuses an occupied entity (`TileEntity.IsOccupied`) and relays; the
/// client (`:3964-3975`) clears or sets the interaction anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileEntityInteraction {
    /// `-1` clears the anchor (`MessageBuffer.cs:3952-3956`).
    pub entity_id: i32,
    pub player_id: u8,
}

/// `msg 124 TEHatRackItemSync`'s body (`NetMessage.cs:1489-1508`):
/// `(byte)player + int entityId + (byte)slot`, where the writer FOLDS the
/// dye flag into the slot (`num4 += 2` when `number4 == 1f`, `:1492-1496`)
/// and the reader UNFOLDS it (`>= 2` → dye, `-2`,
/// `MessageBuffer.cs:4005-4009`). The tail is `TEHatRack.WriteItem`
/// (`TEHatRack.cs:378-388`: ushort type + ushort stack + byte prefix) or
/// the no-entity `int 0 + byte 0` fallback (`:1505-1506`) — again
/// byte-width-identical (5), so the fallback decodes as an air slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HatRackItemSync {
    pub player_id: u8,
    pub entity_id: i32,
    /// The WIRE slot byte: 0/1 items, 2/3 dyes (the +2 fold).
    pub wire_slot: u8,
    pub item: TeItem,
}

impl HatRackItemSync {
    /// The reader's unfold (`MessageBuffer.cs:4005-4009`): `>= 2` is a
    /// dye slot, the index is `wire_slot - 2`.
    pub fn is_dye(&self) -> bool {
        self.wire_slot >= 2
    }
    pub fn slot_index(&self) -> u8 {
        if self.wire_slot >= 2 {
            self.wire_slot - 2
        } else {
            self.wire_slot
        }
    }
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
    /// `39 ReleaseItemOwnership` — a client releases its item reservation
    /// (`WorldItem.FindOwner`'s `netMode == 1` arm, `WorldItem.cs:256`;
    /// reader `MessageBuffer.cs:2232-2255`).
    ReleaseItemOwnership(ReleaseItemOwnership),
    /// `48 LiquidUpdate` — a client's single-tile liquid change
    /// (`NetMessage.sendWater`'s `netMode == 1` arm, `NetMessage.cs:2782`;
    /// reader `MessageBuffer.cs:2407-2446`). Deprecated in favour of module
    /// 82's liquid stream, but the arm is live.
    LiquidUpdate(LiquidUpdate),
    /// `52 LockAndUnlock` — a client unlocked/locked a chest or door
    /// (`Player.cs:32813`/`:33197`/`:41660`; host apply + relay
    /// `MessageBuffer.cs:2531-2564`). C2s-only.
    LockAndUnlock(LockAndUnlock),
    /// `58 InstrumentSound` — a client played its held instrument
    /// (`Player.cs:46544` etc.; host id-force + relay
    /// `MessageBuffer.cs:2642-2650`).
    InstrumentSound(InstrumentSound),
    /// `76 QuestsCountSync` — a client's quest/golf counts (the reader
    /// accepts them from a peer or `ServerSideCharacter`,
    /// `MessageBuffer.cs:3164-3182`).
    QuestsCountSync(QuestsCountSync),
    /// `79 PlaceObject` — a client placed a multi-tile object
    /// (`Player.cs:40147`/`:45819`, `Projectile.cs:25356` via
    /// `SendObjectPlacement`; host gate + relay
    /// `MessageBuffer.cs:3192-3215`).
    PlaceObject(PlaceObject),
    /// `85 QuickStackChests` — the client's quick-stack ask (the
    /// `netMode == 1` writer half, `NetMessage.cs:1206-1210`;
    /// `QuickStacking.cs:296`).
    QuickStackRequest(QuickStackRequest),
    /// `87 TileEntityPlacement` — a client placed a tile entity
    /// (`TEItemFrame.cs:25` and the other placement hooks; host gate +
    /// `PlaceEntityNet`, `MessageBuffer.cs:3294-3305`). C2s-only.
    TileEntityPlacement(TileEntityPlacement),
    /// `89 ItemFrameTryPlacing` — a client tries to place its held item
    /// on an item frame (`TEItemFrame.cs:150`; host `TryPlacing`,
    /// `MessageBuffer.cs:3377-3393`). C2s-only.
    ItemFrameTryPlacing(TryPlacingItem),
    /// `94 DevCommands` — a debug command (`DebugOptions.cs:68-70`,
    /// `QuickLoad.cs:171`; the reader's gate + ladder
    /// `MessageBuffer.cs:3463-3518`). Both directions.
    DevCommands(DevCommands),
    /// `105 GemLockToggle` — a client toggled a gem lock
    /// (`Player.cs:32547`/`:32559`; host `WorldGen.ToggleGemLock`,
    /// `MessageBuffer.cs:3736-3741`). C2s-only.
    GemLockToggle(GemLockToggle),
    /// `111 ToggleParty` — the client's manual-party ask, bodyless
    /// (`BirthdayParty.cs:154`; host `ToggleManualParty`,
    /// `MessageBuffer.cs:3809-3814`). C2s-only.
    ToggleParty,
    /// `121 TEDisplayDollDataSync` — a client changed a doll slot/pose
    /// (`ItemSlot.cs:1109-1762`; host id-force + relay
    /// `MessageBuffer.cs:3920-3941`). Both directions.
    DisplayDollSync(DisplayDollSync),
    /// `122 RequestTileEntityInteraction` — a client opened/closed a
    /// tile-entity interaction (`TileEntity.cs:280`/`:285`;
    /// `MessageBuffer.cs:3942-3977`). Both directions.
    TileEntityInteraction(TileEntityInteraction),
    /// `123 WeaponsRackTryPlacing` — a client tries to place its held
    /// item on a weapons rack (`TEWeaponsRack.cs:212`; host `TryPlacing`,
    /// `MessageBuffer.cs:3978-3994`). C2s-only; the 89 body twin.
    WeaponsRackTryPlacing(TryPlacingItem),
    /// `124 TEHatRackItemSync` — a client changed a hat-rack slot/dye
    /// (`ItemSlot.cs:1117` etc.; host id-force + dye-unfold + relay
    /// `MessageBuffer.cs:3995-4022`). Both directions.
    HatRackItemSync(HatRackItemSync),
    /// `133 FoodPlatterTryPlacing` — a client tries to place its held
    /// item on a food platter (`TEFoodPlatter.cs:133`; host `TryPlacing`,
    /// `MessageBuffer.cs:4147-4163`). C2s-only; the 89 body twin.
    FoodPlatterTryPlacing(TryPlacingItem),
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
    /// `25 Unused25` — deprecated (`MessageID.cs:59-60`), no body; the
    /// reader's ignore group accepts and drops it
    /// (`MessageBuffer.cs:4483-4490`, shared with 15/26/44/67/83/93).
    Unused25,
    /// `26 Unused26` — deprecated (`MessageID.cs:62-63`), no body; the same
    /// reader ignore group (`MessageBuffer.cs:4483-4490`).
    Unused26,
    /// `44 Unknown44` — deprecated (`MessageID.cs:99-100`), no body; the same
    /// reader ignore group (`MessageBuffer.cs:4483-4490`).
    Unknown44,
    /// `67 Unknown67` — no body; the same reader ignore group
    /// (`MessageBuffer.cs:4483-4490`).
    Unknown67,
    /// `83 Unused83` — deprecated (`MessageID.cs:179-180`), no body; the
    /// same reader ignore group (`MessageBuffer.cs:4483-4490`).
    Unused83,
    /// `39 ReleaseItemOwnership` — the host's release notice to one client
    /// (`WorldItem.cs:264`'s `SendData(39, num, …)`; the reader's
    /// `netMode == 1` arm re-runs `FindOwner(forceAssignToServer: true)`,
    /// `MessageBuffer.cs:2237-2243`).
    ReleaseItemOwnership(ReleaseItemOwnership),
    /// `48 LiquidUpdate` — one tile's liquid, host fan-out
    /// (`NetMessage.sendWater`'s per-section loop, `NetMessage.cs:2785-2796`;
    /// the drained-tile re-push `MessageBuffer.cs:2441`).
    LiquidUpdate(LiquidUpdate),
    /// `57 Unknown57` — the world-evil bytes, join push
    /// (`MessageBuffer.cs:867`) + `WorldGen.cs:71173`; client-only reader
    /// (`MessageBuffer.cs:2631-2638`). S2c-only.
    WorldEvilBytes(WorldEvilBytes),
    /// `58 InstrumentSound` — a peer's instrument, host relay
    /// (`MessageBuffer.cs:2649`).
    InstrumentSound(InstrumentSound),
    /// `76 QuestsCountSync` — a peer's quest/golf counts, host relay
    /// (`MessageBuffer.cs:3178`).
    QuestsCountSync(QuestsCountSync),
    /// `79 PlaceObject` — the host's placement relay
    /// (`SendObjectPlacement`'s `netMode == 2` half,
    /// `NetMessage.cs:2577-2581` + `MessageBuffer.cs:3212`).
    PlaceObject(PlaceObject),
    /// `85 QuickStackChests` — the blocked-chest list (the host writer
    /// half, `NetMessage.cs:1213`; `QuickStacking.cs:277`).
    QuickStackBlockedChests(QuickStackBlockedChests),
    /// `86 TileEntitySharing` — one tile entity's full state or removal
    /// (the placement hooks' host push, `TileEntityType.cs:19`; the
    /// client-only reader `MessageBuffer.cs:3272-3293`). S2c-only.
    TileEntitySharing(Box<TileEntitySharing>),
    /// `88 ItemTweaker` — a drop's stat overrides (the drop-rule tweaks,
    /// `CommonCode.cs:67-92`; client-only reader
    /// `MessageBuffer.cs:3306-3376`). S2c-only.
    ItemTweaker(ItemTweaker),
    /// `90 SpawnInstancedItem` — a per-player instanced drop
    /// (`WorldItem.MakeInstanced`, `WorldItem.cs:332`); the body is the
    /// shared case-21 `ItemState`, the client tail marks it instanced +
    /// reserved for `myPlayer` (`MessageBuffer.cs:1489-1493`). S2c-only.
    SpawnInstancedItem(Box<ItemState>),
    /// `93 SocialHandshake` — no body; the reader ignore group
    /// (`MessageBuffer.cs:4483-4490`), accept-gate-whitelisted (`:165`).
    SocialHandshake,
    /// `94 DevCommands` — the host's debug relay
    /// (`MessageBuffer.cs:3488`/`:3503`). Both directions.
    DevCommands(DevCommands),
    /// `104 ShopOverride` — a shop slot override (no live send site; the
    /// writer/reader stack-width mismatch is kept verbatim, see the
    /// struct doc). S2c-only.
    ShopOverride(ShopOverride),
    /// `107 SmartTextMessage` — a multiline smart text (the sign-read
    /// wiring messages, `Wiring.cs:1208`/`:1228`; client-only reader
    /// `MessageBuffer.cs:3752-3757`). S2c-only.
    SmartTextMessage(SmartTextMessage),
    /// `119 CombatTextString` — a text combat number (the 81 int twin's
    /// text half; no live send site, the writer arm is the census
    /// record; client-only reader `MessageBuffer.cs:3235-3242`).
    /// S2c-only.
    CombatTextString(CombatTextString),
    /// `121 TEDisplayDollDataSync` — the host's doll relay
    /// (`MessageBuffer.cs:3938`). Both directions.
    DisplayDollSync(DisplayDollSync),
    /// `122 RequestTileEntityInteraction` — the host's anchor relay
    /// (`MessageBuffer.cs:3955`/`:3961`). Both directions.
    TileEntityInteraction(TileEntityInteraction),
    /// `124 TEHatRackItemSync` — the host's hat-rack relay
    /// (`MessageBuffer.cs:4019`, re-folded via `flag19.ToInt()`). Both
    /// directions.
    HatRackItemSync(HatRackItemSync),
    /// `129 FinishedConnectingToServer` — the join arm's "you're in"
    /// notice, bodyless (`MessageBuffer.cs:937`; the client's team
    /// re-push + UI-scale + preparation-state tail `:4075-4085`).
    /// S2c-only.
    FinishedConnecting,
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
            Self::ReleaseItemOwnership(_) => id::RELEASE_ITEM_OWNERSHIP,
            Self::LiquidUpdate(_) => id::LIQUID_UPDATE,
            Self::LockAndUnlock(_) => id::LOCK_AND_UNLOCK,
            Self::InstrumentSound(_) => id::INSTRUMENT_SOUND,
            Self::QuestsCountSync(_) => id::QUESTS_COUNT_SYNC,
            Self::PlaceObject(_) => id::PLACE_OBJECT,
            Self::QuickStackRequest(_) => id::QUICK_STACK_CHESTS,
            Self::TileEntityPlacement(_) => id::TILE_ENTITY_PLACEMENT,
            Self::ItemFrameTryPlacing(_) => id::ITEM_FRAME_TRY_PLACING,
            Self::DevCommands(_) => id::DEV_COMMANDS,
            Self::GemLockToggle(_) => id::GEM_LOCK_TOGGLE,
            Self::ToggleParty => id::TOGGLE_PARTY,
            Self::DisplayDollSync(_) => id::TE_DISPLAY_DOLL_SYNC,
            Self::TileEntityInteraction(_) => id::TE_INTERACTION_REQUEST,
            Self::WeaponsRackTryPlacing(_) => id::WEAPONS_RACK_TRY_PLACING,
            Self::HatRackItemSync(_) => id::TE_HAT_RACK_ITEM_SYNC,
            Self::FoodPlatterTryPlacing(_) => id::FOOD_PLATTER_TRY_PLACING,
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
            // `NetMessage.cs:948-951`: (short)slot + (number2 != 0f) bool.
            Self::ReleaseItemOwnership(r) => w.i16(r.slot).bool(r.force_assign_to_server),
            // `NetMessage.cs:987-995`: two (short)s + liquid + liquidType().
            Self::LiquidUpdate(l) => w.i16(l.x).i16(l.y).u8(l.liquid).u8(l.liquid_type),
            // `NetMessage.cs:1014-1018`: (byte)action + two (short)s; the
            // player id is NOT written.
            Self::LockAndUnlock(l) => w.u8(l.action).i16(l.x).i16(l.y),
            // `NetMessage.cs:1059-1062`: (byte)player + raw float pitch.
            Self::InstrumentSound(s) => w.u8(s.player_id).f32(s.pitch),
            // `NetMessage.cs:1149-1153`: (byte)player + two raw ints.
            Self::QuestsCountSync(q) => w
                .u8(q.player_id)
                .i32(q.angler_quests_finished)
                .i32(q.golfer_score),
            // `NetMessage.cs:1166-1174`: two (short)s + (short)type +
            // (short)style + (byte)alt + (sbyte)random + (number7 == 1).
            Self::PlaceObject(p) => w
                .i16(p.x)
                .i16(p.y)
                .i16(p.kind)
                .i16(p.style)
                .u8(p.alternative)
                .i8(p.random)
                .bool(p.direction),
            // `NetMessage.cs:1206-1210` + `QuickStacking.cs:299-306`:
            // int count + (short) slots, then the smart-stack byte.
            Self::QuickStackRequest(q) => {
                let mut w = w.i32(q.slots.len() as i32);
                for s in &q.slots {
                    w = w.i16(*s);
                }
                w.bool(q.smart_stack)
            }
            // `NetMessage.cs:1227-1231`: (short)x + (short)y + (byte)type.
            Self::TileEntityPlacement(p) => w.i16(p.x).i16(p.y).u8(p.entity_type),
            // `NetMessage.cs:1297-1306`: two (short)s + (short)type +
            // prefix byte + (short)stack; the player/slot never ride.
            Self::ItemFrameTryPlacing(p) => {
                w.i16(p.x).i16(p.y).i16(p.kind).u8(p.prefix).i16(p.stack)
            }
            // `NetMessage.cs:1327-1332`: plain string + int + float + float.
            Self::DevCommands(d) => w
                .str(&d.command)
                .i32(d.arg_int)
                .f32(d.arg_float)
                .f32(d.arg_float2),
            // `NetMessage.cs:1395-1399`: two (short)s + (number3 == 1f).
            Self::GemLockToggle(g) => w.i16(g.x).i16(g.y).bool(g.on),
            // No writer case for 111 — `SendData(111)` writes a bare frame.
            Self::ToggleParty => w,
            // `NetMessage.cs:1458-1474`: (byte)player + int id +
            // (byte)itemIndex + (byte)command + WriteData/WriteDummySync.
            Self::DisplayDollSync(d) => write_doll_sync(d, w),
            // `NetMessage.cs:1475-1478`: int entityId + (byte)player.
            Self::TileEntityInteraction(t) => w.i32(t.entity_id).u8(t.player_id),
            // `NetMessage.cs:1479-1488` — the exact 89 body under id 123.
            Self::WeaponsRackTryPlacing(p) => {
                w.i16(p.x).i16(p.y).i16(p.kind).u8(p.prefix).i16(p.stack)
            }
            // `NetMessage.cs:1545-1554` — the exact 89 body under id 133.
            Self::FoodPlatterTryPlacing(p) => {
                w.i16(p.x).i16(p.y).i16(p.kind).u8(p.prefix).i16(p.stack)
            }
            // `NetMessage.cs:1489-1508`: (byte)player + int id + the
            // already-folded wire slot + WriteItem (or the air fallback,
            // which is the same 5 bytes as an air TeItem slot).
            Self::HatRackItemSync(h) => h
                .item
                .write_slot(w.u8(h.player_id).i32(h.entity_id).u8(h.wire_slot)),
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
            // `MessageBuffer.cs:2234-2236`: ReadInt16 + ReadBoolean.
            id::RELEASE_ITEM_OWNERSHIP => Self::ReleaseItemOwnership(ReleaseItemOwnership {
                slot: r.i16()?,
                force_assign_to_server: r.bool()?,
            }),
            // `MessageBuffer.cs:2409-2412`: two ReadInt16 + two ReadByte.
            id::LIQUID_UPDATE => Self::LiquidUpdate(LiquidUpdate {
                x: r.i16()?,
                y: r.i16()?,
                liquid: r.u8()?,
                liquid_type: r.u8()?,
            }),
            // `MessageBuffer.cs:2533-2535`: ReadByte + two ReadInt16.
            id::LOCK_AND_UNLOCK => Self::LockAndUnlock(LockAndUnlock {
                action: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
            }),
            // `MessageBuffer.cs:2641-2646`: ReadByte + ReadSingle (the id
            // force to the sender is the apply arm's, not the codec's).
            id::INSTRUMENT_SOUND => Self::InstrumentSound(InstrumentSound {
                player_id: r.u8()?,
                pitch: r.f32()?,
            }),
            // `MessageBuffer.cs:3166-3175`: ReadByte + two ReadInt32.
            id::QUESTS_COUNT_SYNC => Self::QuestsCountSync(QuestsCountSync {
                player_id: r.u8()?,
                angler_quests_finished: r.i32()?,
                golfer_score: r.i32()?,
            }),
            // `MessageBuffer.cs:3194-3200`: two ReadInt16 + ReadInt16 type +
            // ReadInt16 style + ReadByte alt + ReadSByte random + ReadBoolean
            // (mapped to direction 1/-1 by the apply arm).
            id::PLACE_OBJECT => Self::PlaceObject(PlaceObject {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.i16()?,
                style: r.i16()?,
                alternative: r.u8()?,
                random: r.i8()?,
                direction: r.bool()?,
            }),
            // `MessageBuffer.cs:3265` → `QuickStacking.ReadNetInventory`
            // (`QuickStacking.cs:308-321`): int count + (short) slots, then
            // the smart-stack ReadBoolean (`:3265`).
            id::QUICK_STACK_CHESTS => {
                let n = r.i32()?;
                let mut slots = Vec::new();
                for _ in 0..n.max(0) {
                    slots.push(r.i16()?);
                }
                Self::QuickStackRequest(QuickStackRequest {
                    slots,
                    smart_stack: r.bool()?,
                })
            }
            // `MessageBuffer.cs:3297-3299`: two ReadInt16 + ReadByte.
            id::TILE_ENTITY_PLACEMENT => Self::TileEntityPlacement(TileEntityPlacement {
                x: r.i16()?,
                y: r.i16()?,
                entity_type: r.u8()?,
            }),
            // `MessageBuffer.cs:3383-3387`: two ReadInt16 + ReadInt16 type
            // + ReadByte prefix + ReadInt16 stack.
            id::ITEM_FRAME_TRY_PLACING => Self::ItemFrameTryPlacing(TryPlacingItem {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.i16()?,
                prefix: r.u8()?,
                stack: r.i16()?,
            }),
            // `MessageBuffer.cs:3465-3468`: ReadString + ReadInt32 +
            // (int)ReadSingle + a bare ReadSingle (consumed, discarded —
            // the port keeps it in arg_float2 for the byte census).
            id::DEV_COMMANDS => Self::DevCommands(DevCommands {
                command: r.str()?,
                arg_int: r.i32()?,
                arg_float: r.f32()?,
                arg_float2: r.f32()?,
            }),
            // `MessageBuffer.cs:3739`: two ReadInt16 + ReadBoolean.
            id::GEM_LOCK_TOGGLE => Self::GemLockToggle(GemLockToggle {
                x: r.i16()?,
                y: r.i16()?,
                on: r.bool()?,
            }),
            // `MessageBuffer.cs:3809-3814`: the host reads NOTHING.
            id::TOGGLE_PARTY => Self::ToggleParty,
            // `MessageBuffer.cs:3922-3935` — the shared doll read.
            id::TE_DISPLAY_DOLL_SYNC => Self::DisplayDollSync(read_doll_sync(r)?),
            // `MessageBuffer.cs:3944-3945`: ReadInt32 + ReadByte.
            id::TE_INTERACTION_REQUEST => Self::TileEntityInteraction(TileEntityInteraction {
                entity_id: r.i32()?,
                player_id: r.u8()?,
            }),
            // `MessageBuffer.cs:3984-3988` — the exact 89 read under 123.
            id::WEAPONS_RACK_TRY_PLACING => Self::WeaponsRackTryPlacing(TryPlacingItem {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.i16()?,
                prefix: r.u8()?,
                stack: r.i16()?,
            }),
            // `MessageBuffer.cs:4153-4157` — the exact 89 read under 133.
            id::FOOD_PLATTER_TRY_PLACING => Self::FoodPlatterTryPlacing(TryPlacingItem {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.i16()?,
                prefix: r.u8()?,
                stack: r.i16()?,
            }),
            // `MessageBuffer.cs:3997-4016`: byte + int + byte slot; the
            // dye unfold + the >= 2 refusal are the apply arm's — the
            // codec keeps the wire slot verbatim and reads the 5-byte
            // tail either way (the refusal path reads int+byte, the same
            // width as ReadItem's ushort+ushort+byte).
            id::TE_HAT_RACK_ITEM_SYNC => Self::HatRackItemSync(HatRackItemSync {
                player_id: r.u8()?,
                entity_id: r.i32()?,
                wire_slot: r.u8()?,
                item: TeItem::read_slot(r)?,
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

/// `121`'s shared writer (`NetMessage.cs:1458-1474`): the header then the
/// command-keyed tail — `TEDisplayDoll.WriteData` (`TEDisplayDoll.cs:780-800`)
/// for a real doll, `WriteDummySync` (`:824-833`) for the fallback.
fn write_doll_sync(d: &DisplayDollSync, w: NetWriter) -> NetWriter {
    let w = w
        .u8(d.player_id)
        .i32(d.entity_id)
        .u8(d.item_index)
        .u8(d.command);
    match d.data {
        DollSyncData::Pose(p) => w.u8(p),
        DollSyncData::Item(i) => i.write_slot(w),
        DollSyncData::Dummy => {
            if d.command == 2 {
                w.u8(0)
            } else {
                w.i32(0).u8(0)
            }
        }
    }
}

/// `121`'s shared reader (`MessageBuffer.cs:3922-3935` →
/// `TEDisplayDoll.ReadData` `:802-822` / `ReadDummySync` `:835-844`): the
/// command byte keys the tail; the dummy shapes are byte-width-identical
/// to the real ones, so both decode as the real shapes (see
/// [`DollSyncData`]).
fn read_doll_sync(r: &mut NetReader) -> Option<DisplayDollSync> {
    let player_id = r.u8()?;
    let entity_id = r.i32()?;
    let item_index = r.u8()?;
    let command = r.u8()?;
    let data = if command == 2 {
        DollSyncData::Pose(r.u8()?)
    } else {
        DollSyncData::Item(TeItem::read_slot(r)?)
    };
    Some(DisplayDollSync {
        player_id,
        entity_id,
        item_index,
        command,
        data,
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
            Self::Unused25 => id::UNUSED_25,
            Self::Unused26 => id::UNUSED_26,
            Self::Unknown44 => id::UNKNOWN_44,
            Self::Unknown67 => id::UNKNOWN_67,
            Self::Unused83 => id::UNUSED_83,
            Self::ReleaseItemOwnership(_) => id::RELEASE_ITEM_OWNERSHIP,
            Self::LiquidUpdate(_) => id::LIQUID_UPDATE,
            Self::WorldEvilBytes(_) => id::WORLD_EVIL_BYTES,
            Self::InstrumentSound(_) => id::INSTRUMENT_SOUND,
            Self::QuestsCountSync(_) => id::QUESTS_COUNT_SYNC,
            Self::PlaceObject(_) => id::PLACE_OBJECT,
            Self::QuickStackBlockedChests(_) => id::QUICK_STACK_CHESTS,
            Self::TileEntitySharing(_) => id::TILE_ENTITY_SHARING,
            Self::ItemTweaker(_) => id::ITEM_TWEAKER,
            Self::SpawnInstancedItem(_) => id::SPAWN_INSTANCED_ITEM,
            Self::SocialHandshake => id::SOCIAL_HANDSHAKE,
            Self::DevCommands(_) => id::DEV_COMMANDS,
            Self::ShopOverride(_) => id::SHOP_OVERRIDE,
            Self::SmartTextMessage(_) => id::SMART_TEXT_MESSAGE,
            Self::CombatTextString(_) => id::COMBAT_TEXT_STRING,
            Self::DisplayDollSync(_) => id::TE_DISPLAY_DOLL_SYNC,
            Self::TileEntityInteraction(_) => id::TE_INTERACTION_REQUEST,
            Self::HatRackItemSync(_) => id::TE_HAT_RACK_ITEM_SYNC,
            Self::FinishedConnecting => id::FINISHED_CONNECTING,
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
            // `25/26/44/67/83` have no writer arm; bare frames mirror the
            // reader's ignore group (`MessageBuffer.cs:4483-4490`).
            Self::Unused25
            | Self::Unused26
            | Self::Unknown44
            | Self::Unknown67
            | Self::Unused83
            | Self::SocialHandshake => w,
            // `NetMessage.cs:646-669` — the shared case-21 body under id 90.
            Self::SpawnInstancedItem(d) => d.write_to(w),
            // `NetMessage.cs:1327-1332` — the shared dev-command body.
            Self::DevCommands(d) => w
                .str(&d.command)
                .i32(d.arg_int)
                .f32(d.arg_float)
                .f32(d.arg_float2),
            // `NetMessage.cs:1387-1394` — the WRITER shape: byte + short +
            // FLOAT stack (negative → 0f) + byte + int + byte.
            Self::ShopOverride(s) => {
                // The C# `(short)number3` is the x86 two-step float→int→
                // short truncation; `as i32 as i16` mirrors it (a plain
                // `as i16` would saturate instead of wrapping).
                let stack = if (s.stack as i32 as i16) < 0 { 0.0 } else { s.stack };
                w.u8(s.slot)
                    .i16(s.kind)
                    .f32(stack)
                    .u8(s.prefix)
                    .i32(s.value)
                    .u8(s.flags)
            }
            // `NetMessage.cs:769-775`: (byte)r + (byte)g + (byte)b +
            // NetworkText + (short)widthLimit.
            Self::SmartTextMessage(m) => m
                .text
                .clone()
                .write_to(w.rgb(m.color))
                .i16(m.width_limit),
            // `NetMessage.cs:1188-1196`: float x + float y + WriteRGB +
            // NetworkText.
            Self::CombatTextString(c) => c
                .text
                .clone()
                .write_to(w.f32(c.x).f32(c.y).rgb(c.color)),
            // `NetMessage.cs:1458-1474` / `:1475-1478` — the shared bodies.
            Self::DisplayDollSync(d) => write_doll_sync(d, w),
            Self::TileEntityInteraction(t) => w.i32(t.entity_id).u8(t.player_id),
            // `NetMessage.cs:1489-1508` — the shared hat-rack body.
            Self::HatRackItemSync(h) => h
                .item
                .write_slot(w.u8(h.player_id).i32(h.entity_id).u8(h.wire_slot)),
            // No writer case for 129 — `TrySendData(129)` writes a bare
            // frame (`MessageBuffer.cs:937`).
            Self::FinishedConnecting => w,
            // `NetMessage.cs:948-951` / `:987-995` — the same bodies as C2s.
            Self::ReleaseItemOwnership(r) => w.i16(r.slot).bool(r.force_assign_to_server),
            Self::LiquidUpdate(l) => w.i16(l.x).i16(l.y).u8(l.liquid).u8(l.liquid_type),
            // `NetMessage.cs:1054-1058`: tGood, tEvil, tBlood — three bytes.
            Self::WorldEvilBytes(e) => w.u8(e.good).u8(e.evil).u8(e.blood),
            // `NetMessage.cs:1059-1062` / `:1149-1153` — the shared bodies.
            Self::InstrumentSound(s) => w.u8(s.player_id).f32(s.pitch),
            Self::QuestsCountSync(q) => w
                .u8(q.player_id)
                .i32(q.angler_quests_finished)
                .i32(q.golfer_score),
            // `NetMessage.cs:1166-1174` — the shared placement body.
            Self::PlaceObject(p) => w
                .i16(p.x)
                .i16(p.y)
                .i16(p.kind)
                .i16(p.style)
                .u8(p.alternative)
                .i8(p.random)
                .bool(p.direction),
            // `NetMessage.cs:1213` + `QuickStacking.cs:323-330`: int count
            // + (ushort) chest indices.
            Self::QuickStackBlockedChests(b) => {
                let mut w = w.i32(b.chests.len() as i32);
                for c in &b.chests {
                    w = w.u16(*c);
                }
                w
            }
            // `NetMessage.cs:1216-1226`: int id + bool exists; when present,
            // `TileEntity.Write(networkSend: true)` (`TileEntity.cs:173-197`):
            // type byte + (short) position pair (NO id on the net path) +
            // the per-kind extra data.
            Self::TileEntitySharing(t) => {
                let w = w.i32(t.id).bool(t.entity.is_some());
                match &t.entity {
                    None => w,
                    Some(e) => e.write_extra(w.u8(e.type_id()).i16(t.x).i16(t.y)),
                }
            }
            // `NetMessage.cs:1232-1296`: (short)slot + BitsByte 1, the
            // gated fields in bit order, then BitsByte 2 + its gated tail.
            Self::ItemTweaker(t) => {
                let mut b1 = BitsByte::new();
                b1.set(0, t.color.is_some());
                b1.set(1, t.damage.is_some());
                b1.set(2, t.knockback.is_some());
                b1.set(3, t.use_animation.is_some());
                b1.set(4, t.use_time.is_some());
                b1.set(5, t.shoot.is_some());
                b1.set(6, t.shoot_speed.is_some());
                b1.set(7, t.has_second());
                let mut w = w.i16(t.slot).bitsbyte(b1);
                if let Some(v) = t.color {
                    w = w.u32(v);
                }
                if let Some(v) = t.damage {
                    w = w.u16(v);
                }
                if let Some(v) = t.knockback {
                    w = w.f32(v);
                }
                if let Some(v) = t.use_animation {
                    w = w.u16(v);
                }
                if let Some(v) = t.use_time {
                    w = w.u16(v);
                }
                if let Some(v) = t.shoot {
                    w = w.i16(v);
                }
                if let Some(v) = t.shoot_speed {
                    w = w.f32(v);
                }
                if t.has_second() {
                    let mut b2 = BitsByte::new();
                    b2.set(0, t.width.is_some());
                    b2.set(1, t.height.is_some());
                    b2.set(2, t.scale.is_some());
                    b2.set(3, t.ammo.is_some());
                    b2.set(4, t.use_ammo.is_some());
                    b2.set(5, t.not_ammo.is_some());
                    w = w.bitsbyte(b2);
                    if let Some(v) = t.width {
                        w = w.u16(v);
                    }
                    if let Some(v) = t.height {
                        w = w.u16(v);
                    }
                    if let Some(v) = t.scale {
                        w = w.f32(v);
                    }
                    if let Some(v) = t.ammo {
                        w = w.i16(v);
                    }
                    if let Some(v) = t.use_ammo {
                        w = w.i16(v);
                    }
                    if let Some(v) = t.not_ammo {
                        w = w.bool(v);
                    }
                }
                w
            }
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
            // `25/26/44/67/83` — the reader's ignore group reads nothing
            // (`MessageBuffer.cs:4483-4490`); bare frames decode.
            id::UNUSED_25 => Self::Unused25,
            id::UNUSED_26 => Self::Unused26,
            id::UNKNOWN_44 => Self::Unknown44,
            id::UNKNOWN_67 => Self::Unknown67,
            id::UNUSED_83 => Self::Unused83,
            id::SOCIAL_HANDSHAKE => Self::SocialHandshake,
            // `MessageBuffer.cs:1445-1459` — the shared case-21 read; the
            // instanced/reserved tail (`:1489-1493`) is the apply arm's.
            id::SPAWN_INSTANCED_ITEM => Self::SpawnInstancedItem(Box::new(ItemState::read_from(r)?)),
            // `MessageBuffer.cs:3465-3468` — the shared dev-command read.
            id::DEV_COMMANDS => Self::DevCommands(DevCommands {
                command: r.str()?,
                arg_int: r.i32()?,
                arg_float: r.f32()?,
                arg_float2: r.f32()?,
            }),
            // `MessageBuffer.cs:3719-3724` — the READER shape: byte +
            // short + SHORT stack (the writer sends a float — the C#
            // mismatch, kept verbatim) + byte + int + byte. The decoded
            // stack is widened to the struct's f32 field.
            id::SHOP_OVERRIDE => Self::ShopOverride(ShopOverride {
                slot: r.u8()?,
                kind: r.i16()?,
                stack: r.i16()? as f32,
                prefix: r.u8()?,
                value: r.i32()?,
                flags: r.u8()?,
            }),
            // `MessageBuffer.cs:3755`: ReadRGB + NetworkText.Deserialize +
            // ReadInt16 width limit.
            id::SMART_TEXT_MESSAGE => Self::SmartTextMessage(SmartTextMessage {
                color: r.rgb()?,
                text: NetText::read_from(r)?,
                width_limit: r.i16()?,
            }),
            // `MessageBuffer.cs:3238-3240`: two ReadSingle (the (int)
            // truncation is the apply arm's) + ReadRGB + NetworkText.
            id::COMBAT_TEXT_STRING => Self::CombatTextString(CombatTextString {
                x: r.f32()?,
                y: r.f32()?,
                color: r.rgb()?,
                text: NetText::read_from(r)?,
            }),
            // `MessageBuffer.cs:3922-3935` / `:3944-3945` — shared reads.
            id::TE_DISPLAY_DOLL_SYNC => Self::DisplayDollSync(read_doll_sync(r)?),
            id::TE_INTERACTION_REQUEST => Self::TileEntityInteraction(TileEntityInteraction {
                entity_id: r.i32()?,
                player_id: r.u8()?,
            }),
            // `MessageBuffer.cs:3997-4016` — the shared hat-rack read.
            id::TE_HAT_RACK_ITEM_SYNC => Self::HatRackItemSync(HatRackItemSync {
                player_id: r.u8()?,
                entity_id: r.i32()?,
                wire_slot: r.u8()?,
                item: TeItem::read_slot(r)?,
            }),
            // `MessageBuffer.cs:4075-4085`: the client reads NOTHING.
            id::FINISHED_CONNECTING => Self::FinishedConnecting,
            // `MessageBuffer.cs:2234-2236` / `:2409-2412` — the shared
            // reader reads the same shapes on both sides.
            id::RELEASE_ITEM_OWNERSHIP => Self::ReleaseItemOwnership(ReleaseItemOwnership {
                slot: r.i16()?,
                force_assign_to_server: r.bool()?,
            }),
            id::LIQUID_UPDATE => Self::LiquidUpdate(LiquidUpdate {
                x: r.i16()?,
                y: r.i16()?,
                liquid: r.u8()?,
                liquid_type: r.u8()?,
            }),
            // `MessageBuffer.cs:2634-2636`: three ReadByte, good→evil→blood.
            id::WORLD_EVIL_BYTES => Self::WorldEvilBytes(WorldEvilBytes {
                good: r.u8()?,
                evil: r.u8()?,
                blood: r.u8()?,
            }),
            // `MessageBuffer.cs:2641-2646` / `:3166-3175` — the shared reads.
            id::INSTRUMENT_SOUND => Self::InstrumentSound(InstrumentSound {
                player_id: r.u8()?,
                pitch: r.f32()?,
            }),
            id::QUESTS_COUNT_SYNC => Self::QuestsCountSync(QuestsCountSync {
                player_id: r.u8()?,
                angler_quests_finished: r.i32()?,
                golfer_score: r.i32()?,
            }),
            // `MessageBuffer.cs:3194-3200` — the shared placement read.
            id::PLACE_OBJECT => Self::PlaceObject(PlaceObject {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.i16()?,
                style: r.i16()?,
                alternative: r.u8()?,
                random: r.i8()?,
                direction: r.bool()?,
            }),
            // `MessageBuffer.cs:3269` → `QuickStacking.ReadBlockedChestList`
            // (`QuickStacking.cs:332-341`): int count + (ushort) indices.
            id::QUICK_STACK_CHESTS => {
                let n = r.i32()?;
                let mut chests = Vec::new();
                for _ in 0..n.max(0) {
                    chests.push(r.u16()?);
                }
                Self::QuickStackBlockedChests(QuickStackBlockedChests { chests })
            }
            // `MessageBuffer.cs:3278-3291`: int id + bool; false removes
            // (no tail read), true reads `TileEntity.Read(reader, 326,
            // networkSend: true)` — type byte + position pair + extra data.
            id::TILE_ENTITY_SHARING => {
                let id = r.i32()?;
                let exists = r.bool()?;
                if !exists {
                    Self::TileEntitySharing(Box::new(TileEntitySharing {
                        id,
                        x: 0,
                        y: 0,
                        entity: None,
                    }))
                } else {
                    let kind = r.u8()?;
                    let x = r.i16()?;
                    let y = r.i16()?;
                    Self::TileEntitySharing(Box::new(TileEntitySharing {
                        id,
                        x,
                        y,
                        entity: Some(TileEntityData::read_extra(kind, r)?),
                    }))
                }
            }
            // `MessageBuffer.cs:3312-3374`: ReadInt16 slot (the 0..=400
            // gate is the apply arm's), BitsByte 1 + the gated reads, then
            // BitsByte 2 + its tail. Width/height read as ReadInt16 here
            // (`:3352`/`:3356`) vs the writer's ushort — the port decodes
            // the raw two bytes unsigned, matching the writer's domain.
            id::ITEM_TWEAKER => {
                let slot = r.i16()?;
                let b1 = r.bitsbyte()?;
                let mut t = ItemTweaker {
                    slot,
                    ..Default::default()
                };
                if b1.get(0) {
                    t.color = Some(r.u32()?);
                }
                if b1.get(1) {
                    t.damage = Some(r.u16()?);
                }
                if b1.get(2) {
                    t.knockback = Some(r.f32()?);
                }
                if b1.get(3) {
                    t.use_animation = Some(r.u16()?);
                }
                if b1.get(4) {
                    t.use_time = Some(r.u16()?);
                }
                if b1.get(5) {
                    t.shoot = Some(r.i16()?);
                }
                if b1.get(6) {
                    t.shoot_speed = Some(r.f32()?);
                }
                if b1.get(7) {
                    let b2 = r.bitsbyte()?;
                    if b2.get(0) {
                        t.width = Some(r.u16()?);
                    }
                    if b2.get(1) {
                        t.height = Some(r.u16()?);
                    }
                    if b2.get(2) {
                        t.scale = Some(r.f32()?);
                    }
                    if b2.get(3) {
                        t.ammo = Some(r.i16()?);
                    }
                    if b2.get(4) {
                        t.use_ammo = Some(r.i16()?);
                    }
                    if b2.get(5) {
                        t.not_ammo = Some(r.bool()?);
                    }
                }
                Self::ItemTweaker(t)
            }
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

    // ---- S1-I-1 W4-f21: the net-id tail census witnesses (WRITTEN NOT RUN,
    // code-only). Each pins the id byte + the exact C# body bytes. ----

    #[test]
    fn w4_f21_unused_melee_strike_is_censused_dead() {
        // `24` — no `SendData(24)` site exists; the reader asserts false and
        // reads nothing (`MessageBuffer.cs:1706-1708`). The constant records
        // the id; the writer's `(short)npc + (byte)player` bytes
        // (`NetMessage.cs:765-768`) are documented at the constant, NOT
        // ported as a live codec (an asserting reader has no decode).
        assert_eq!(id::UNUSED_MELEE_STRIKE, 24);
        // The writer shape, pinned as raw bytes for the census record:
        let w = NetWriter::open(id::UNUSED_MELEE_STRIKE).i16(7).u8(3).finish();
        assert_eq!(w[2], 24);
        assert_eq!(&w[3..], &[7, 0, 3]);
        // No S2c/C2s decode arm exists for 24 (the reader asserts).
        assert!(S2c::decode(24, &mut NetReader::new(&[])).is_none());
        assert!(C2s::decode(24, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_unused_25_is_a_bare_no_op_frame() {
        // `25` — `[Old("Deprecated. Use NetTextModule instead.")]`
        // (`MessageID.cs:59-60`); the reader's ignore group
        // (`MessageBuffer.cs:4483-4490`) reads nothing.
        let f = S2c::Unused25.encode();
        assert_eq!(f[2], 25);
        assert_eq!(f.len(), 3); // bare frame, no body
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::Unused25 => {}
            other => panic!("expected 25, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn w4_f21_release_item_ownership_round_trips_both_directions() {
        // `NetMessage.cs:948-951`: (short)number + (number2 != 0f) bool.
        // `WorldItem.cs:256` sends force=true/false from the client;
        // `:264`'s host notice carries the default false.
        for force in [false, true] {
            let b = ReleaseItemOwnership {
                slot: 399,
                force_assign_to_server: force,
            };
            let mut body = 399i16.to_le_bytes().to_vec();
            body.push(u8::from(force));
            let f = C2s::ReleaseItemOwnership(b).encode();
            assert_eq!(f[2], 39);
            assert_eq!(&f[3..], &body);
            let mut r = NetReader::new(&f[3..]);
            match C2s::decode(f[2], &mut r).expect("decode") {
                C2s::ReleaseItemOwnership(d) => assert_eq!(d, b),
                other => panic!("expected 39, got {:?}", other.msg_id()),
            }
            assert_eq!(r.remaining(), 0);
            let f = S2c::ReleaseItemOwnership(b).encode();
            assert_eq!(f[2], 39);
            assert_eq!(&f[3..], &body);
            match S2c::decode(39, &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::ReleaseItemOwnership(d) => assert_eq!(d, b),
                other => panic!("expected 39, got {:?}", other.msg_id()),
            }
        }
        // Negative slots ride the (short) verbatim; a truncated bool fails.
        let f = C2s::ReleaseItemOwnership(ReleaseItemOwnership {
            slot: -1,
            force_assign_to_server: true,
        })
        .encode();
        assert_eq!(&f[3..], &[0xFF, 0xFF, 1]);
        assert!(C2s::decode(39, &mut NetReader::new(&[1, 0])).is_none());
    }

    #[test]
    fn w4_f21_liquid_update_round_trips_both_directions() {
        // `NetMessage.cs:987-995`: (short)x + (short)y + tile.liquid byte +
        // tile.liquidType() byte (`Tile.cs:235-238`: (bTileHeader & 0x60)
        // >> 5 — 0 water / 1 lava / 2 honey / 3 shimmer).
        let b = LiquidUpdate {
            x: -12,
            y: 340,
            liquid: 255,
            liquid_type: 3, // shimmer
        };
        let mut body = (-12i16).to_le_bytes().to_vec();
        body.extend_from_slice(&340i16.to_le_bytes());
        body.push(255);
        body.push(3);
        let f = C2s::LiquidUpdate(b).encode();
        assert_eq!(f[2], 48);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::LiquidUpdate(d) => assert_eq!(d, b),
            other => panic!("expected 48, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        let f = S2c::LiquidUpdate(b).encode();
        assert_eq!(f[2], 48);
        assert_eq!(&f[3..], &body);
        match S2c::decode(48, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::LiquidUpdate(d) => assert_eq!(d, b),
            other => panic!("expected 48, got {:?}", other.msg_id()),
        }
        // The drained-tile shape (`MessageBuffer.cs:2439-2442` re-pushes it).
        let drained = LiquidUpdate {
            x: 5,
            y: 6,
            liquid: 0,
            liquid_type: 0,
        };
        let f = S2c::LiquidUpdate(drained).encode();
        assert_eq!(&f[3..], &[5, 0, 6, 0, 0, 0]);
        // A truncated liquidType byte must not decode.
        assert!(S2c::decode(48, &mut NetReader::new(&body[..5])).is_none());
    }

    #[test]
    fn w4_f21_lock_and_unlock_round_trips() {
        // `NetMessage.cs:1014-1018`: (byte)number2 + (short)number3 +
        // (short)number4 — the player id `number` is NOT on the wire (the
        // host relay re-sends it as 0, `MessageBuffer.cs:2541`).
        for action in [1u8, 2, 3, 0, 255] {
            let b = LockAndUnlock {
                action,
                x: -5,
                y: 1200,
            };
            let mut body = vec![action];
            body.extend_from_slice(&(-5i16).to_le_bytes());
            body.extend_from_slice(&1200i16.to_le_bytes());
            let f = C2s::LockAndUnlock(b).encode();
            assert_eq!(f[2], 52);
            assert_eq!(f.len(), 3 + 5);
            assert_eq!(&f[3..], &body);
            let mut r = NetReader::new(&f[3..]);
            match C2s::decode(f[2], &mut r).expect("decode") {
                C2s::LockAndUnlock(d) => assert_eq!(d, b),
                other => panic!("expected 52, got {:?}", other.msg_id()),
            }
            assert_eq!(r.remaining(), 0);
        }
        // C2s-only: the reader arm is host-gated, no S2c variant exists.
        assert!(S2c::decode(52, &mut NetReader::new(&[1, 0, 0, 0, 0])).is_none());
        // A truncated y must not decode.
        assert!(C2s::decode(52, &mut NetReader::new(&[1, 0, 0, 0])).is_none());
    }

    #[test]
    fn w4_f21_world_evil_bytes_round_trips() {
        // `NetMessage.cs:1054-1058`: tGood, tEvil, tBlood — three raw bytes
        // in the writer's order; the reader assigns the same order
        // (`MessageBuffer.cs:2634-2636`).
        let b = WorldEvilBytes {
            good: 200,
            evil: 100,
            blood: 50,
        };
        let f = S2c::WorldEvilBytes(b).encode();
        assert_eq!(f[2], 57);
        assert_eq!(&f[3..], &[200, 100, 50]);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::WorldEvilBytes(d) => assert_eq!(d, b),
            other => panic!("expected 57, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // S2c-only: no C2s variant (the reader is netMode==1-gated).
        assert!(C2s::decode(57, &mut NetReader::new(&[1, 2, 3])).is_none());
        // A truncated blood byte must not decode.
        assert!(S2c::decode(57, &mut NetReader::new(&[1, 2])).is_none());
    }

    #[test]
    fn w4_f21_instrument_sound_round_trips_both_directions() {
        // `NetMessage.cs:1059-1062`: (byte)number + raw float number2 (no
        // cast — the pitch rides as a full float, `Player.cs:46544`'s num6).
        let b = InstrumentSound {
            player_id: 4,
            pitch: 0.45,
        };
        let mut body = vec![4u8];
        body.extend_from_slice(&0.45f32.to_le_bytes());
        let f = C2s::InstrumentSound(b).encode();
        assert_eq!(f[2], 58);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::InstrumentSound(d) => assert_eq!(d, b),
            other => panic!("expected 58, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        let f = S2c::InstrumentSound(b).encode();
        assert_eq!(f[2], 58);
        match S2c::decode(58, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::InstrumentSound(d) => assert_eq!(d, b),
            other => panic!("expected 58, got {:?}", other.msg_id()),
        }
        // A negative pitch rides verbatim (the drums' num9 can be negative
        // before the round; the wire is the raw float).
        let neg = InstrumentSound {
            player_id: 0,
            pitch: -0.25,
        };
        let f = C2s::InstrumentSound(neg).encode();
        let mut body = vec![0u8];
        body.extend_from_slice(&(-0.25f32).to_le_bytes());
        assert_eq!(&f[3..], &body);
        // A truncated pitch must not decode.
        assert!(C2s::decode(58, &mut NetReader::new(&[4, 0, 0])).is_none());
    }

    #[test]
    fn w4_f21_quests_count_sync_round_trips_both_directions() {
        // `NetMessage.cs:1149-1153`: (byte)number + raw int
        // anglerQuestsFinished + raw int golferScoreAccumulated.
        let b = QuestsCountSync {
            player_id: 2,
            angler_quests_finished: 42,
            golfer_score: 1_000_000_000, // the AccumulateGolfingScore cap
        };
        let mut body = vec![2u8];
        body.extend_from_slice(&42i32.to_le_bytes());
        body.extend_from_slice(&1_000_000_000i32.to_le_bytes());
        let f = C2s::QuestsCountSync(b).encode();
        assert_eq!(f[2], 76);
        assert_eq!(f.len(), 3 + 9);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::QuestsCountSync(d) => assert_eq!(d, b),
            other => panic!("expected 76, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        let f = S2c::QuestsCountSync(b).encode();
        assert_eq!(f[2], 76);
        match S2c::decode(76, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::QuestsCountSync(d) => assert_eq!(d, b),
            other => panic!("expected 76, got {:?}", other.msg_id()),
        }
        // A truncated golf score must not decode.
        assert!(S2c::decode(76, &mut NetReader::new(&body[..8])).is_none());
    }

    #[test]
    fn w4_f21_place_object_round_trips_both_directions() {
        // `NetMessage.cs:1166-1174`: (short)x + (short)y + (short)type +
        // (short)style + (byte)alt + (sbyte)random + (number7 == 1) bool.
        let b = PlaceObject {
            x: -3,
            y: 500,
            kind: 21, // a chest
            style: 4,
            alternative: 2,
            random: -7,
            direction: true,
        };
        let mut body = (-3i16).to_le_bytes().to_vec();
        body.extend_from_slice(&500i16.to_le_bytes());
        body.extend_from_slice(&21i16.to_le_bytes());
        body.extend_from_slice(&4i16.to_le_bytes());
        body.push(2);
        body.push((-7i8) as u8);
        body.push(1);
        let f = C2s::PlaceObject(b).encode();
        assert_eq!(f[2], 79);
        assert_eq!(f.len(), 3 + 12);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::PlaceObject(d) => assert_eq!(d, b),
            other => panic!("expected 79, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        let f = S2c::PlaceObject(b).encode();
        assert_eq!(f[2], 79);
        match S2c::decode(79, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlaceObject(d) => assert_eq!(d, b),
            other => panic!("expected 79, got {:?}", other.msg_id()),
        }
        // direction=false rides as 0 (the reader maps it to -1).
        let left = PlaceObject {
            direction: false,
            ..b
        };
        let f = C2s::PlaceObject(left).encode();
        assert_eq!(*f.last().unwrap(), 0);
        // A truncated direction byte must not decode.
        assert!(C2s::decode(79, &mut NetReader::new(&body[..11])).is_none());
    }

    #[test]
    fn w4_f21_quick_stack_direction_split_round_trips() {
        // C2s half (`NetMessage.cs:1206-1210` + `QuickStacking.cs:299-306`):
        // int count + (short) SlotIds + the smart-stack byte.
        let q = QuickStackRequest {
            slots: vec![0, 10, 58, 179], // inventory + a bank slot id
            smart_stack: true,
        };
        let mut body = 4i32.to_le_bytes().to_vec();
        for s in &q.slots {
            body.extend_from_slice(&s.to_le_bytes());
        }
        body.push(1);
        let f = C2s::QuickStackRequest(q.clone()).encode();
        assert_eq!(f[2], 85);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::QuickStackRequest(d) => assert_eq!(d, q),
            other => panic!("expected 85, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // S2c half (`NetMessage.cs:1213` + `QuickStacking.cs:323-330`):
        // int count + (ushort) chest indices, NO smart-stack byte.
        let b = QuickStackBlockedChests {
            chests: vec![3, 7999],
        };
        let mut body = 2i32.to_le_bytes().to_vec();
        for c in &b.chests {
            body.extend_from_slice(&c.to_le_bytes());
        }
        let f = S2c::QuickStackBlockedChests(b.clone()).encode();
        assert_eq!(f[2], 85);
        assert_eq!(&f[3..], &body);
        match S2c::decode(85, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::QuickStackBlockedChests(d) => assert_eq!(d, b),
            other => panic!("expected 85, got {:?}", other.msg_id()),
        }
        // The empty forms are bare int-0 counts (+ the C2s smart byte).
        let f = C2s::QuickStackRequest(QuickStackRequest {
            slots: vec![],
            smart_stack: false,
        })
        .encode();
        assert_eq!(&f[3..], &[0, 0, 0, 0, 0]);
        let f = S2c::QuickStackBlockedChests(QuickStackBlockedChests { chests: vec![] }).encode();
        assert_eq!(&f[3..], &[0, 0, 0, 0]);
        // A truncated slot list must not decode.
        assert!(C2s::decode(85, &mut NetReader::new(&[2, 0, 0, 0, 1, 0])).is_none());
    }

    #[test]
    fn w4_f21_tile_entity_placement_round_trips() {
        // `NetMessage.cs:1227-1231`: (short)x + (short)y + (byte)type —
        // the type is the manager's registration id
        // (`TileEntitiesManager.cs:26-39`: ItemFrame = 1).
        let b = TileEntityPlacement {
            x: -8,
            y: 900,
            entity_type: 1,
        };
        let mut body = (-8i16).to_le_bytes().to_vec();
        body.extend_from_slice(&900i16.to_le_bytes());
        body.push(1);
        let f = C2s::TileEntityPlacement(b).encode();
        assert_eq!(f[2], 87);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::TileEntityPlacement(d) => assert_eq!(d, b),
            other => panic!("expected 87, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only: the reader is host-gated, no S2c variant.
        assert!(S2c::decode(87, &mut NetReader::new(&body)).is_none());
        // A truncated type byte must not decode.
        assert!(C2s::decode(87, &mut NetReader::new(&body[..4])).is_none());
    }

    #[test]
    fn w4_f21_tile_entity_sharing_item_frame_round_trips() {
        // `NetMessage.cs:1216-1226` + `TileEntity.cs:173-197` (net path):
        // int id + bool + type byte + (short)x + (short)y + extra data.
        // ItemFrame's extra is one (short)type + (byte)prefix + (short)stack
        // (`TEItemFrame.cs:40-45`).
        let t = TileEntitySharing {
            id: 42,
            x: 100,
            y: 200,
            entity: Some(TileEntityData::ItemFrame(TeItem {
                kind: 3093,
                prefix: 81,
                stack: 1,
            })),
        };
        let mut body = 42i32.to_le_bytes().to_vec();
        body.push(1); // exists
        body.push(1); // type: ItemFrame
        body.extend_from_slice(&100i16.to_le_bytes());
        body.extend_from_slice(&200i16.to_le_bytes());
        body.extend_from_slice(&3093i16.to_le_bytes());
        body.push(81);
        body.extend_from_slice(&1i16.to_le_bytes());
        let f = S2c::TileEntitySharing(Box::new(t.clone())).encode();
        assert_eq!(f[2], 86);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::TileEntitySharing(d) => assert_eq!(*d, t),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The removal frame: id + false, NO tail (`MessageBuffer.cs:3279`).
        let rm = TileEntitySharing {
            id: 42,
            x: 0,
            y: 0,
            entity: None,
        };
        let f = S2c::TileEntitySharing(Box::new(rm.clone())).encode();
        let mut body = 42i32.to_le_bytes().to_vec();
        body.push(0);
        assert_eq!(&f[3..], &body);
        match S2c::decode(86, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TileEntitySharing(d) => assert_eq!(*d, rm),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
        // An invalid type id fails the read (InvalidEntityID).
        let mut bad = 1i32.to_le_bytes().to_vec();
        bad.push(1);
        bad.push(11); // one past CritterAnchor
        bad.extend_from_slice(&[0, 0, 0, 0]);
        assert!(S2c::decode(86, &mut NetReader::new(&bad)).is_none());
    }

    #[test]
    fn w4_f21_tile_entity_sharing_doll_hat_rack_flags() {
        // The doll's three flag bytes + pose (`TEDisplayDoll.cs:159-184`):
        // equip[0] + equip[8] (bits3[1]) + dyes[8] (bits3[2]) + misc[0]
        // (bits3[0]) non-air; the triples ride equips→dyes→misc in order.
        let mut equip = [TeItem::default(); 9];
        let mut dyes = [TeItem::default(); 9];
        let mut misc = [TeItem::default(); 1];
        equip[0] = TeItem { kind: 100, prefix: 0, stack: 1 };
        equip[8] = TeItem { kind: 200, prefix: 5, stack: 1 };
        dyes[8] = TeItem { kind: 300, prefix: 0, stack: 1 };
        misc[0] = TeItem { kind: 400, prefix: 2, stack: 1 };
        let t = TileEntitySharing {
            id: 7,
            x: 10,
            y: 20,
            entity: Some(TileEntityData::DisplayDoll { equip, dyes, misc, pose: 3 }),
        };
        let f = S2c::TileEntitySharing(Box::new(t.clone())).encode();
        assert_eq!(f[2], 86);
        // body: id(4) + exists(1) + type(1) + x(2) + y(2) + b1(1) + b2(1)
        // + pose(1) + b3(1) + 4 triples x 5 bytes.
        let b = &f[3..];
        assert_eq!(b[4], 1); // exists
        assert_eq!(b[5], 3); // DisplayDoll
        assert_eq!(b[10], 0b0000_0001); // b1: equip[0]
        assert_eq!(b[11], 0); // b2: no low dyes
        assert_eq!(b[12], 3); // pose
        assert_eq!(b[13], 0b0000_0111); // b3: misc + equip8 + dye8
        assert_eq!(f.len(), 3 + 14 + 20);
        let mut r = NetReader::new(b);
        match S2c::decode(86, &mut r).expect("decode") {
            S2c::TileEntitySharing(d) => assert_eq!(*d, t),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The hat rack's single flag byte (`TEHatRack.cs:58-63`): items[1]
        // + dyes[0] non-air → bits 1 and 2.
        let t = TileEntitySharing {
            id: 8,
            x: 1,
            y: 2,
            entity: Some(TileEntityData::HatRack {
                items: [
                    TeItem::default(),
                    TeItem { kind: 500, prefix: 0, stack: 1 },
                ],
                dyes: [
                    TeItem { kind: 600, prefix: 1, stack: 1 },
                    TeItem::default(),
                ],
            }),
        };
        let f = S2c::TileEntitySharing(Box::new(t.clone())).encode();
        let b = &f[3..];
        assert_eq!(b[5], 5); // HatRack
        assert_eq!(b[10], 0b0000_0110); // items[1] | dyes[0]
        assert_eq!(f.len(), 3 + 11 + 10);
        match S2c::decode(86, &mut NetReader::new(b)).expect("decode") {
            S2c::TileEntitySharing(d) => assert_eq!(*d, t),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
        // LogicSensor writes NOTHING on the net path
        // (`TELogicSensor.cs:380-387`'s !networkSend gate); the pylon has
        // no override at all.
        for e in [TileEntityData::LogicSensor, TileEntityData::TeleportationPylon] {
            let t = TileEntitySharing { id: 9, x: 3, y: 4, entity: Some(e) };
            let f = S2c::TileEntitySharing(Box::new(t.clone())).encode();
            assert_eq!(f.len(), 3 + 10); // id + bool + type + position only
            match S2c::decode(86, &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::TileEntitySharing(d) => assert_eq!(*d, t),
                other => panic!("expected 86, got {:?}", other.msg_id()),
            }
        }
        // The leashed anchors' (short)itemType tail
        // (`TELeashedEntityAnchorWithItem.cs:10-13`).
        let t = TileEntitySharing {
            id: 10,
            x: 5,
            y: 6,
            entity: Some(TileEntityData::KiteAnchor { item_type: 4765 }),
        };
        let f = S2c::TileEntitySharing(Box::new(t.clone())).encode();
        assert_eq!(&f[3 + 10..], &4765i16.to_le_bytes());
        match S2c::decode(86, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TileEntitySharing(d) => assert_eq!(*d, t),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
    }

    #[test]
    fn w4_f21_item_tweaker_round_trips() {
        // `NetMessage.cs:1232-1296`: (short)slot + BitsByte 1 + gated
        // fields; BitsByte 2 rides only under bit 7. The CommonCode.cs
        // sends use number2 = 1f → damage-only (bit 1).
        let dmg_only = ItemTweaker {
            slot: 12,
            damage: Some(50),
            ..Default::default()
        };
        let f = S2c::ItemTweaker(dmg_only).encode();
        assert_eq!(f[2], 88);
        let mut body = 12i16.to_le_bytes().to_vec();
        body.push(0b0000_0010); // bit 1: damage
        body.extend_from_slice(&50u16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::ItemTweaker(d) => assert_eq!(d, dmg_only),
            other => panic!("expected 88, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The full two-byte form: every field set.
        let full = ItemTweaker {
            slot: 400,
            color: Some(0xFF_80_40_20),
            damage: Some(999),
            knockback: Some(7.5),
            use_animation: Some(20),
            use_time: Some(15),
            shoot: Some(-3),
            shoot_speed: Some(12.25),
            width: Some(30),
            height: Some(40),
            scale: Some(1.5),
            ammo: Some(40),
            use_ammo: Some(97),
            not_ammo: Some(true),
        };
        let f = S2c::ItemTweaker(full).encode();
        let b = &f[3..];
        assert_eq!(b[2], 0xFF); // all 8 bits of BitsByte 1
        // b1 fields: color 4 + damage 2 + kb 4 + anim 2 + time 2 + shoot 2
        // + speed 4 = 20 bytes, then BitsByte 2 at offset 2+1+20 = 23.
        assert_eq!(b[23], 0b0011_1111); // all 6 bits of BitsByte 2
        let mut r = NetReader::new(b);
        match S2c::decode(88, &mut r).expect("decode") {
            S2c::ItemTweaker(d) => assert_eq!(d, full),
            other => panic!("expected 88, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The empty form: slot + a zero flag byte, no second byte.
        let empty = ItemTweaker { slot: -1, ..Default::default() };
        let f = S2c::ItemTweaker(empty).encode();
        assert_eq!(&f[3..], &[0xFF, 0xFF, 0]);
        match S2c::decode(88, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemTweaker(d) => assert_eq!(d, empty),
            other => panic!("expected 88, got {:?}", other.msg_id()),
        }
        // A second-byte-only tweak still rides bit 7 + the byte.
        let scale_only = ItemTweaker { slot: 0, scale: Some(2.0), ..Default::default() };
        let f = S2c::ItemTweaker(scale_only).encode();
        assert_eq!(&f[3..5], &[0, 0]);
        assert_eq!(f[5], 0b1000_0000); // bit 7 only
        assert_eq!(f[6], 0b0000_0100); // scale bit 2
        // A truncated gated field must not decode.
        assert!(S2c::decode(88, &mut NetReader::new(&[12, 0, 2, 50])).is_none());
    }

    #[test]
    fn w4_f21_item_frame_try_placing_round_trips() {
        // `NetMessage.cs:1297-1306`: (short)x + (short)y + (short)type +
        // prefix byte + (short)stack — the player/slot indices select the
        // item on the writer side only, never on the wire.
        let b = TryPlacingItem {
            x: -2,
            y: 700,
            kind: 3093,
            prefix: 81,
            stack: 1,
        };
        let mut body = (-2i16).to_le_bytes().to_vec();
        body.extend_from_slice(&700i16.to_le_bytes());
        body.extend_from_slice(&3093i16.to_le_bytes());
        body.push(81);
        body.extend_from_slice(&1i16.to_le_bytes());
        let f = C2s::ItemFrameTryPlacing(b).encode();
        assert_eq!(f[2], 89);
        assert_eq!(f.len(), 3 + 9);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::ItemFrameTryPlacing(d) => assert_eq!(d, b),
            other => panic!("expected 89, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only: the reader is host-gated.
        assert!(S2c::decode(89, &mut NetReader::new(&body)).is_none());
        // A truncated stack must not decode.
        assert!(C2s::decode(89, &mut NetReader::new(&body[..8])).is_none());
    }

    #[test]
    fn w4_f21_spawn_instanced_item_shares_the_case_21_body() {
        // `NetMessage.cs:646-647`: case 21 and case 90 share the writer —
        // the same ItemState bytes under a different id byte.
        let d = ItemState {
            slot: 401,
            x: 100.5,
            y: -20.25,
            vx: 1.0,
            vy: -2.0,
            stack: 3,
            prefix: 81,
            kind: 757,
            shimmered: false,
            shimmer_time: 0.0,
        };
        let f90 = S2c::SpawnInstancedItem(Box::new(d)).encode();
        let f21 = S2c::ItemState(Box::new(d)).encode();
        assert_eq!(f90[2], 90);
        assert_eq!(f21[2], 21);
        assert_eq!(&f90[3..], &f21[3..]); // identical bodies
        let mut r = NetReader::new(&f90[3..]);
        match S2c::decode(f90[2], &mut r).expect("decode") {
            S2c::SpawnInstancedItem(got) => assert_eq!(*got, d),
            other => panic!("expected 90, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The shimmer tail rides the shared body too.
        let sh = ItemState { shimmered: true, shimmer_time: 12.5, ..d };
        let f = S2c::SpawnInstancedItem(Box::new(sh)).encode();
        match S2c::decode(90, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SpawnInstancedItem(got) => assert_eq!(*got, sh),
            other => panic!("expected 90, got {:?}", other.msg_id()),
        }
        // S2c-only: no C2s arm (MakeInstanced is host-gated).
        assert!(C2s::decode(90, &mut NetReader::new(&f90[3..])).is_none());
    }

    #[test]
    fn w4_f21_social_handshake_is_a_bare_whitelisted_frame() {
        // `93` — the reader ignore group's last member
        // (`MessageBuffer.cs:4483-4490`); unlike 15/25/26/44/67/83 it IS
        // accept-gate-whitelisted pre-spawn (`:165`'s `b != 93` escape).
        let f = S2c::SocialHandshake.encode();
        assert_eq!(f[2], 93);
        assert_eq!(f.len(), 3);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::SocialHandshake => {}
            other => panic!("expected 93, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn w4_f21_dev_commands_round_trips_both_directions() {
        // `NetMessage.cs:1327-1332`: plain string (NOT NetworkText) + int
        // + float + float. `DebugOptions.cs:68`'s /showdebug shape.
        let d = DevCommands {
            command: "/showdebug".to_string(),
            arg_int: 0,
            arg_float: 1.0,
            arg_float2: 0.0,
        };
        let mut body = vec![10u8]; // the 7-bit length of "/showdebug"
        body.extend_from_slice(b"/showdebug");
        body.extend_from_slice(&0i32.to_le_bytes());
        body.extend_from_slice(&1.0f32.to_le_bytes());
        body.extend_from_slice(&0.0f32.to_le_bytes());
        let f = C2s::DevCommands(d.clone()).encode();
        assert_eq!(f[2], 94);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::DevCommands(got) => assert_eq!(got, d),
            other => panic!("expected 94, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        let f = S2c::DevCommands(d.clone()).encode();
        assert_eq!(f[2], 94);
        match S2c::decode(94, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::DevCommands(got) => assert_eq!(got, d),
            other => panic!("expected 94, got {:?}", other.msg_id()),
        }
        // The /setserverping shape carries a real int + float
        // (`DebugOptions.cs:70`: number2 = Shared_ServerPing).
        let ping = DevCommands {
            command: "/setserverping".to_string(),
            arg_int: 0,
            arg_float: 250.0,
            arg_float2: 0.0,
        };
        let f = C2s::DevCommands(ping.clone()).encode();
        match C2s::decode(94, &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::DevCommands(got) => assert_eq!(got, ping),
            other => panic!("expected 94, got {:?}", other.msg_id()),
        }
        // A truncated float tail must not decode.
        assert!(C2s::decode(94, &mut NetReader::new(&body[..body.len() - 1])).is_none());
    }

    #[test]
    fn w4_f21_shop_override_keeps_the_writer_reader_mismatch_verbatim() {
        // The WRITER (`NetMessage.cs:1387-1394`): byte + short + FLOAT
        // stack + byte + int + byte = 13 body bytes.
        let s = ShopOverride {
            slot: 3,
            kind: 757,
            stack: 5.0,
            prefix: 81,
            value: 10000,
            flags: 1, // buyOnce
        };
        let f = S2c::ShopOverride(s).encode();
        assert_eq!(f[2], 104);
        let mut body = vec![3u8];
        body.extend_from_slice(&757i16.to_le_bytes());
        body.extend_from_slice(&5.0f32.to_le_bytes()); // the FLOAT stack
        body.push(81);
        body.extend_from_slice(&10000i32.to_le_bytes());
        body.push(1);
        assert_eq!(&f[3..], &body);
        assert_eq!(f.len(), 3 + 13);
        // The READER (`MessageBuffer.cs:3719-3724`) reads a SHORT stack —
        // 11 body bytes. Feeding the writer's frame to the reader
        // misaligns exactly as the C# does: the short eats the float's
        // low two bytes (5.0f32 = 0x40A00000 → low short 0), the prefix
        // byte eats the float's third byte (0xA0 = 160), the int eats
        // [0x40, 81, 0x10, 0x27] and the flags byte eats 0x00.
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(104, &mut r).expect("decode") {
            S2c::ShopOverride(d) => {
                assert_eq!(d.slot, 3);
                assert_eq!(d.kind, 757);
                assert_eq!(d.stack, 0.0); // the float's low short
                assert_eq!(d.prefix, 0xA0); // the float's third byte
                assert_eq!(d.value, i32::from_le_bytes([0x40, 81, 0x10, 0x27]));
                assert_eq!(d.flags, 0);
            }
            other => panic!("expected 104, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 2); // the writer's two extra bytes
        // A clean READER-shaped frame round trips through decode.
        let mut rbody = vec![3u8];
        rbody.extend_from_slice(&757i16.to_le_bytes());
        rbody.extend_from_slice(&5i16.to_le_bytes()); // the SHORT stack
        rbody.push(81);
        rbody.extend_from_slice(&10000i32.to_le_bytes());
        rbody.push(1);
        match S2c::decode(104, &mut NetReader::new(&rbody)).expect("decode") {
            S2c::ShopOverride(d) => {
                assert_eq!(d.stack, 5.0);
                assert_eq!(d.prefix, 81);
                assert_eq!(d.value, 10000);
                assert_eq!(d.flags, 1);
            }
            other => panic!("expected 104, got {:?}", other.msg_id()),
        }
        // The negative-stack clamp: (short)(-3f) < 0 → the writer sends 0f.
        let neg = ShopOverride { stack: -3.0, ..s };
        let f = S2c::ShopOverride(neg).encode();
        assert_eq!(&f[6..10], &0.0f32.to_le_bytes());
        // S2c-only: no C2s arm.
        assert!(C2s::decode(104, &mut NetReader::new(&rbody)).is_none());
    }

    #[test]
    fn w4_f21_gem_lock_toggle_round_trips() {
        // `NetMessage.cs:1395-1399`: (short)x + (short)y + (number3 == 1f)
        // bool — `Player.cs:32547` sends 1f (on), `:32559` the default 0f.
        for on in [true, false] {
            let b = GemLockToggle { x: -9, y: 800, on };
            let mut body = (-9i16).to_le_bytes().to_vec();
            body.extend_from_slice(&800i16.to_le_bytes());
            body.push(u8::from(on));
            let f = C2s::GemLockToggle(b).encode();
            assert_eq!(f[2], 105);
            assert_eq!(&f[3..], &body);
            let mut r = NetReader::new(&f[3..]);
            match C2s::decode(f[2], &mut r).expect("decode") {
                C2s::GemLockToggle(d) => assert_eq!(d, b),
                other => panic!("expected 105, got {:?}", other.msg_id()),
            }
            assert_eq!(r.remaining(), 0);
        }
        // C2s-only in practice: no S2c variant.
        assert!(S2c::decode(105, &mut NetReader::new(&[0, 0, 0, 0, 1])).is_none());
        // A truncated bool must not decode.
        assert!(C2s::decode(105, &mut NetReader::new(&[0, 0, 0, 0])).is_none());
    }

    #[test]
    fn w4_f21_smart_text_message_round_trips() {
        // `NetMessage.cs:769-775`: (byte)r + (byte)g + (byte)b +
        // NetworkText + (short)widthLimit. The Wiring.cs:1208 sign-read
        // fixture: the pink colour + width 460.
        let m = SmartTextMessage {
            color: Rgb(255, 100, 200),
            text: NetText("A sign's text".to_string()),
            width_limit: 460,
        };
        let f = S2c::SmartTextMessage(m.clone()).encode();
        assert_eq!(f[2], 107);
        let mut body = vec![255, 100, 200];
        body.push(0); // NetworkText literal mode
        body.push(13); // the 7-bit length of "A sign's text"
        body.extend_from_slice(b"A sign's text");
        body.extend_from_slice(&460i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::SmartTextMessage(d) => assert_eq!(d, m),
            other => panic!("expected 107, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // S2c-only: no C2s arm.
        assert!(C2s::decode(107, &mut NetReader::new(&body)).is_none());
        // A truncated width limit must not decode.
        assert!(S2c::decode(107, &mut NetReader::new(&body[..body.len() - 1])).is_none());
    }

    #[test]
    fn w4_f21_toggle_party_is_a_bodyless_client_ask() {
        // `111` has NO writer case — `SendData(111)` (BirthdayParty.cs:154)
        // writes only the frame header; the host reader reads nothing
        // (`MessageBuffer.cs:3809-3814`).
        let f = C2s::ToggleParty.encode();
        assert_eq!(f[2], 111);
        assert_eq!(f.len(), 3);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::ToggleParty => {}
            other => panic!("expected 111, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only: no S2c arm (the reader is host-gated).
        assert!(S2c::decode(111, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_combat_text_string_round_trips() {
        // `NetMessage.cs:1188-1196`: float x + float y + WriteRGB +
        // NetworkText — the 81 int twin's text half (d98 owns 81).
        let c = CombatTextString {
            x: 123.75,
            y: -45.5,
            color: Rgb(255, 240, 20),
            text: NetText("Immune!".to_string()),
        };
        let f = S2c::CombatTextString(c.clone()).encode();
        assert_eq!(f[2], 119);
        let mut body = 123.75f32.to_le_bytes().to_vec();
        body.extend_from_slice(&(-45.5f32).to_le_bytes());
        body.extend_from_slice(&[255, 240, 20]);
        body.push(0); // literal mode
        body.push(7); // "Immune!" length
        body.extend_from_slice(b"Immune!");
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::CombatTextString(d) => assert_eq!(d, c),
            other => panic!("expected 119, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The raw floats ride (the reader's (int) truncation is apply-side).
        // S2c-only: no C2s arm.
        assert!(C2s::decode(119, &mut NetReader::new(&body)).is_none());
        // A truncated text must not decode.
        assert!(S2c::decode(119, &mut NetReader::new(&body[..body.len() - 1])).is_none());
    }

    #[test]
    fn w4_f21_display_doll_sync_round_trips_both_directions() {
        // `NetMessage.cs:1458-1474`: (byte)player + int id + (byte)slot +
        // (byte)command + WriteData. Command 0/1/3 = the WriteItem slot
        // order (ushort type + ushort STACK + byte PREFIX —
        // TEDisplayDoll.cs:758-764), command 2 = the pose byte.
        let d = DisplayDollSync {
            player_id: 1,
            entity_id: 77,
            item_index: 4,
            command: 0,
            data: DollSyncData::Item(TeItem { kind: 100, prefix: 81, stack: 1 }),
        };
        let mut body = vec![1u8];
        body.extend_from_slice(&77i32.to_le_bytes());
        body.push(4);
        body.push(0);
        body.extend_from_slice(&100u16.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes()); // stack BEFORE prefix
        body.push(81);
        let f = C2s::DisplayDollSync(d).encode();
        assert_eq!(f[2], 121);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::DisplayDollSync(got) => assert_eq!(got, d),
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        let f = S2c::DisplayDollSync(d).encode();
        assert_eq!(f[2], 121);
        match S2c::decode(121, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::DisplayDollSync(got) => assert_eq!(got, d),
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
        // The pose command (2) carries ONE byte
        // (`TEDisplayDoll.cs:785-789`, the :907 pose-sync send).
        let pose = DisplayDollSync {
            command: 2,
            item_index: 3, // the pose rides item_index at :907
            data: DollSyncData::Pose(3),
            ..d
        };
        let f = C2s::DisplayDollSync(pose).encode();
        assert_eq!(f.len(), 3 + 7 + 1);
        match C2s::decode(121, &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::DisplayDollSync(got) => assert_eq!(got, pose),
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
        // The dummy fallback is byte-width-identical: command 2 → one
        // zero byte (decodes Pose(0)); else int 0 + byte 0 (decodes an
        // air TeItem) — `WriteDummySync` :824-833.
        let dummy_pose = DisplayDollSync { command: 2, data: DollSyncData::Dummy, ..d };
        let f = C2s::DisplayDollSync(dummy_pose).encode();
        assert_eq!(*f.last().unwrap(), 0);
        match C2s::decode(121, &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::DisplayDollSync(got) => assert_eq!(got.data, DollSyncData::Pose(0)),
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
        let dummy_item = DisplayDollSync { command: 1, data: DollSyncData::Dummy, ..d };
        let f = C2s::DisplayDollSync(dummy_item).encode();
        assert_eq!(f.len(), 3 + 7 + 5);
        match C2s::decode(121, &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::DisplayDollSync(got) => {
                assert_eq!(got.data, DollSyncData::Item(TeItem::default()));
            }
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
        // A truncated slot tail must not decode.
        assert!(C2s::decode(121, &mut NetReader::new(&body[..body.len() - 1])).is_none());
    }

    #[test]
    fn w4_f21_tile_entity_interaction_round_trips_both_directions() {
        // `NetMessage.cs:1475-1478`: int entityId + (byte)player. The -1
        // clear (`TileEntity.cs:280`, `Main.cs:17605`) and the open
        // (`TileEntity.cs:285`).
        for entity_id in [-1i32, 42] {
            let t = TileEntityInteraction { entity_id, player_id: 2 };
            let mut body = entity_id.to_le_bytes().to_vec();
            body.push(2);
            let f = C2s::TileEntityInteraction(t).encode();
            assert_eq!(f[2], 122);
            assert_eq!(&f[3..], &body);
            let mut r = NetReader::new(&f[3..]);
            match C2s::decode(f[2], &mut r).expect("decode") {
                C2s::TileEntityInteraction(got) => assert_eq!(got, t),
                other => panic!("expected 122, got {:?}", other.msg_id()),
            }
            assert_eq!(r.remaining(), 0);
            let f = S2c::TileEntityInteraction(t).encode();
            assert_eq!(f[2], 122);
            match S2c::decode(122, &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::TileEntityInteraction(got) => assert_eq!(got, t),
                other => panic!("expected 122, got {:?}", other.msg_id()),
            }
        }
        // A truncated player byte must not decode.
        assert!(C2s::decode(122, &mut NetReader::new(&[42, 0, 0, 0])).is_none());
    }

    #[test]
    fn w4_f21_weapons_rack_try_placing_is_the_89_twin() {
        // `NetMessage.cs:1479-1488` — the exact 89 body under id 123.
        let b = TryPlacingItem {
            x: 12,
            y: 300,
            kind: 3384,
            prefix: 0,
            stack: 1,
        };
        let f123 = C2s::WeaponsRackTryPlacing(b).encode();
        let f89 = C2s::ItemFrameTryPlacing(b).encode();
        assert_eq!(f123[2], 123);
        assert_eq!(f89[2], 89);
        assert_eq!(&f123[3..], &f89[3..]); // identical bodies
        let mut r = NetReader::new(&f123[3..]);
        match C2s::decode(f123[2], &mut r).expect("decode") {
            C2s::WeaponsRackTryPlacing(d) => assert_eq!(d, b),
            other => panic!("expected 123, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only.
        assert!(S2c::decode(123, &mut NetReader::new(&f123[3..])).is_none());
    }

    #[test]
    fn w4_f21_hat_rack_item_sync_round_trips_both_directions() {
        // `NetMessage.cs:1489-1508`: (byte)player + int id + (byte)slot
        // with the dye +2 fold, then WriteItem (ushort type + ushort
        // stack + byte prefix, TEHatRack.cs:385-387).
        for wire_slot in [0u8, 1, 2, 3] {
            let h = HatRackItemSync {
                player_id: 1,
                entity_id: 55,
                wire_slot,
                item: TeItem { kind: 100, prefix: 81, stack: 1 },
            };
            let mut body = vec![1u8];
            body.extend_from_slice(&55i32.to_le_bytes());
            body.push(wire_slot);
            body.extend_from_slice(&100u16.to_le_bytes());
            body.extend_from_slice(&1u16.to_le_bytes());
            body.push(81);
            let f = C2s::HatRackItemSync(h).encode();
            assert_eq!(f[2], 124);
            assert_eq!(&f[3..], &body);
            let mut r = NetReader::new(&f[3..]);
            match C2s::decode(f[2], &mut r).expect("decode") {
                C2s::HatRackItemSync(d) => {
                    assert_eq!(d, h);
                    // The reader's unfold (:4005-4009).
                    assert_eq!(d.is_dye(), wire_slot >= 2);
                    assert_eq!(d.slot_index(), wire_slot % 2);
                }
                other => panic!("expected 124, got {:?}", other.msg_id()),
            }
            assert_eq!(r.remaining(), 0);
            let f = S2c::HatRackItemSync(h).encode();
            assert_eq!(f[2], 124);
            match S2c::decode(124, &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::HatRackItemSync(d) => assert_eq!(d, h),
                other => panic!("expected 124, got {:?}", other.msg_id()),
            }
        }
        // The no-entity fallback (`NetMessage.cs:1505-1506`: int 0 +
        // byte 0) is the same 5 bytes as an air slot — decodes air.
        let fallback = HatRackItemSync {
            player_id: 0,
            entity_id: -1,
            wire_slot: 0,
            item: TeItem::default(),
        };
        let f = C2s::HatRackItemSync(fallback).encode();
        assert_eq!(f.len(), 3 + 6 + 5);
        match C2s::decode(124, &mut NetReader::new(&f[3..])).expect("decode") {
            C2s::HatRackItemSync(d) => assert_eq!(d.item, TeItem::default()),
            other => panic!("expected 124, got {:?}", other.msg_id()),
        }
        // A truncated prefix byte must not decode.
        assert!(C2s::decode(124, &mut NetReader::new(&[1, 55, 0, 0, 0, 0, 100, 0, 1, 0])).is_none());
    }

    #[test]
    fn w4_f21_finished_connecting_is_a_bodyless_join_notice() {
        // `129` has NO writer case — `TrySendData(129, whoAmI)`
        // (`MessageBuffer.cs:937`, the State-3→10 arm) writes only the
        // header; the client reader reads nothing (`:4075-4085`).
        let f = S2c::FinishedConnecting.encode();
        assert_eq!(f[2], 129);
        assert_eq!(f.len(), 3);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::FinishedConnecting => {}
            other => panic!("expected 129, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // S2c-only.
        assert!(C2s::decode(129, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_food_platter_try_placing_is_the_89_twin() {
        // `NetMessage.cs:1545-1554` — the exact 89 body under id 133.
        let b = TryPlacingItem {
            x: 40,
            y: 600,
            kind: 5000,
            prefix: 0,
            stack: 1,
        };
        let f133 = C2s::FoodPlatterTryPlacing(b).encode();
        let f89 = C2s::ItemFrameTryPlacing(b).encode();
        assert_eq!(f133[2], 133);
        assert_eq!(&f133[3..], &f89[3..]); // identical bodies
        let mut r = NetReader::new(&f133[3..]);
        match C2s::decode(f133[2], &mut r).expect("decode") {
            C2s::FoodPlatterTryPlacing(d) => assert_eq!(d, b),
            other => panic!("expected 133, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only.
        assert!(S2c::decode(133, &mut NetReader::new(&f133[3..])).is_none());
    }

    #[test]
    fn w4_f21_deprecated_ignore_group_26_44_67_83_are_bare_frames() {
        // `26 Unused26` (`MessageID.cs:62-63`), `44 Unknown44`
        // (`MessageID.cs:99-100`), `67 Unknown67` (`MessageID.cs:147`) and
        // `83 Unused83` (`MessageID.cs:179-180`) share the reader's ignore
        // group with 15/25/93 (`MessageBuffer.cs:4483-4490`): no writer arm,
        // no body, accepted and dropped.
        for (msg, expect_id) in [
            (S2c::Unused26, 26u8),
            (S2c::Unknown44, 44u8),
            (S2c::Unknown67, 67u8),
            (S2c::Unused83, 83u8),
        ] {
            let f = msg.encode();
            assert_eq!(f[2], expect_id);
            assert_eq!(f.len(), 3);
            let mut r = NetReader::new(&f[3..]);
            let d = S2c::decode(f[2], &mut r).expect("decode");
            assert_eq!(d.msg_id(), expect_id);
            assert_eq!(r.remaining(), 0);
        }
    }
}
