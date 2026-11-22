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
    /// `81 CombatTextInt` (S1-I-1 wave I): a peer's damage number —
    /// `(float)x, (float)y, RGB, (int)amount` (`NetMessage.cs:1179-1186`,
    /// `MessageBuffer.cs:3227-3233`, host→client only).
    pub const COMBAT_TEXT_INT: u8 = 81;
    /// `119 CombatTextString` (W4-g50, S1-I-4): a peer's combat text —
    /// `(float)x, (float)y, RGB, NetworkText` (`NetMessage.cs:1188-1196`,
    /// `MessageBuffer.cs:3235-3242`, host→client only; the 81 string twin).
    pub const COMBAT_TEXT_STRING: u8 = 119;
    /// `84 PlayerStealth` (W4-g50, S1-I-4): `(byte)plr + (float)stealth`
    /// (`MessageID.cs:182`, `NetMessage.cs:1197-1204`,
    /// `MessageBuffer.cs:3246-3260` — the host overrides the id to the
    /// sender and relays). Both directions.
    pub const PLAYER_STEALTH: u8 = 84;
    /// `92 SyncExtraValue` (`MessageID.cs:198`) — `(short)` npc slot,
    /// `(int)` value, two `(float)`s (`NetMessage.cs:1321-1326`,
    /// `MessageBuffer.cs:3442-3461`): a DELTA in on the client→host leg,
    /// the accumulated TOTAL out on the host→client leg. Both directions.
    pub const NPC_EXTRA_VALUE: u8 = 92;
    /// `86 TileEntitySharing` (W4-g50, S1-I-4): `(int)` id + `(bool)`
    /// present + the `TileEntity.Write(networkSend: true)` record when
    /// present (`MessageID.cs:186`, `NetMessage.cs:1216-1225`,
    /// `MessageBuffer.cs:3272-3293`, host→client only).
    pub const TILE_ENTITY_SHARING: u8 = 86;

    // ---- W4-g51 FINAL: the 8 BIG apply-owner-needed ids' codecs, lifted
    // from origin/task/W4-f21-net-ids-verify-tail 5274a097 (the TE families
    // + the dryad). The g50 86/87 seams (write_net/read_net/add_net) ride
    // tr-core; the consumers land in netglue (the TE store is the consumer
    // now — the draw halves stay d57-TE-render / M7-TE-2).

    /// `87 TileEntityPlacement` (W4-g50, S1-I-4): `(short)x, (short)y,
    /// (byte)type` (`MessageID.cs:188`, `NetMessage.cs:1227-1231`,
    /// `MessageBuffer.cs:3294-3305`, client→host only).
    pub const TILE_ENTITY_PLACEMENT: u8 = 87;
    /// `88 ItemTweaker` (W4-g50, S1-I-4): `(short)` item slot + two
    /// BitsBytes gating the `Item.inner` stat writes
    /// (`MessageID.cs:190`, `NetMessage.cs:1232-1296`,
    /// `MessageBuffer.cs:3306-3376`, host→client only).
    pub const ITEM_TWEAKER: u8 = 88;
    /// `89 ItemFrameTryPlacing` (W4-g50, S1-I-4): `(short)x, (short)y,
    /// (short)type, (byte)prefix, (short)stack` (`MessageID.cs:192`,
    /// `NetMessage.cs:1297-1306`, `MessageBuffer.cs:3377-3393`,
    /// client→host only).
    pub const ITEM_FRAME_TRY_PLACING: u8 = 89;
    /// `90 SpawnInstancedItem` (W4-g50, S1-I-4): the `21 ItemDrop` body
    /// verbatim (`NetMessage.cs:646-669`'s shared `case 21: case 90:`);
    /// the client marks the drop instanced + reserved for itself
    /// (`MessageBuffer.cs:1489-1493`). Host→client, one target per player
    /// (`WorldItem.MakeInstanced`, `WorldItem.cs:321-336`).
    pub const SPAWN_INSTANCED_ITEM: u8 = 90;
    /// `91 SyncEmoteBubble` (`MessageID.cs:196`) — an emote bubble's
    /// create/update/removal (`NetMessage.cs:1307-1320`,
    /// `MessageBuffer.cs:3394-3441`; the host's `EmoteBubble.NewBubble`/
    /// `NewBubbleNPC` sends, `EmoteBubble.cs:128`, `:147`).
    pub const EMOTE_BUBBLE: u8 = 91;
    /// `95 MurderSomeoneElsesPortal` (W4-g50, S1-I-4): `(ushort)` owner +
    /// `(byte)` form (`MessageID.cs:204`, `NetMessage.cs:1333-1336`,
    /// `MessageBuffer.cs:3519-3537`, client→host only).
    pub const MURDER_PORTAL: u8 = 95;
    /// `96 TeleportPlayerThroughPortal` (W4-g50, S1-I-4): `(byte)` player,
    /// `(short)` color, dest Vector2, live-velocity Vector2
    /// (`MessageID.cs:206`, `NetMessage.cs:1337-1346`,
    /// `MessageBuffer.cs:3538-3557`). Both directions.
    pub const TELEPORT_PLAYER_PORTAL: u8 = 96;
    /// `99 MinionRestTargetUpdate` (W4-g50, S1-I-4): `(byte)` player +
    /// `MinionRestTargetPoint` Vector2 (`MessageID.cs:212`,
    /// `NetMessage.cs:1353-1356`, `MessageBuffer.cs:3570-3583`). Both
    /// directions — the host forces the id and relays.
    pub const MINION_REST_TARGET: u8 = 99;
    /// `115 MinionAttackTargetUpdate` (W4-g50, S1-I-4): `(byte)` player +
    /// `(short)MinionAttackTargetNPC` (`MessageID.cs:244`,
    /// `NetMessage.cs:1357-1360`, `MessageBuffer.cs:3584-3597`). Both
    /// directions — the host forces the id and relays.
    pub const MINION_ATTACK_TARGET: u8 = 115;
    /// `100 TeleportNPCThroughPortal` (W4-g50, S1-I-4): `(ushort)` npc,
    /// `(short)` color, dest Vector2, live-velocity Vector2
    /// (`MessageID.cs:214`, `NetMessage.cs:1361-1369`,
    /// `MessageBuffer.cs:3598-3610`, host→client only).
    pub const NPC_PORTAL_TELEPORT: u8 = 100;
    /// `101 UpdateTowerShieldStrengths` (W4-g50, S1-I-4): four ushorts
    /// Solar/Vortex/Nebula/Stardust (`MessageID.cs:216`,
    /// `NetMessage.cs:1371-1376`, `MessageBuffer.cs:3611-3651`,
    /// host→client only).
    pub const TOWER_SHIELDS: u8 = 101;
    /// `102 NebulaLevelupRequest` (W4-g50, S1-I-4): `(byte)` picker,
    /// `(ushort)` buffType, picker-centre Vector2 (`MessageID.cs:218`,
    /// `NetMessage.cs:1377-1382`, `MessageBuffer.cs:3652-3706`). Both
    /// directions — the host forces the id and relays.
    pub const NEBULA_LEVELUP_REQUEST: u8 = 102;
    /// `103 MoonlordHorror` (W4-g50, S1-I-4): the two NPC statics as raw
    /// ints, MaxMoonLordCountdown then MoonLordCountdown (`MessageID.cs:220`,
    /// `NetMessage.cs:1383-1386`, `MessageBuffer.cs:3708-3714`,
    /// host→client only).
    pub const MOONLORD_COUNTDOWN: u8 = 103;
    /// `104 ShopOverride` (W4-g50, S1-I-4): `(byte)` slot, `(short)` type,
    /// `(short)` stack, `(byte)` prefix, `(int)` value, BitsByte buyOnce
    /// (`MessageID.cs:222`, `NetMessage.cs:1387-1394`,
    /// `MessageBuffer.cs:3715-3735`, host→client only).
    pub const SHOP_OVERRIDE: u8 = 104;
    /// `105 GemLockToggle` (W4-g50, S1-I-4): `(short)x, (short)y, (bool)on`
    /// (`MessageID.cs:224`, `NetMessage.cs:1395-1399`,
    /// `MessageBuffer.cs:3736-3741`, client→host only).
    pub const GEM_LOCK_TOGGLE: u8 = 105;
    /// `106 PoofOfSmoke` (W4-g50, S1-I-4): one uint — the HalfVector2
    /// PackedValue (`MessageID.cs:226`, `NetMessage.cs:1400-1402`,
    /// `MessageBuffer.cs:3742-3751`, host→client only).
    pub const POOF_OF_SMOKE: u8 = 106;
    /// `107 SmartTextMessage` (W4-g50, S1-I-4): RGB + NetworkText +
    /// `(short)` WidthLimit (`MessageID.cs:228`, `NetMessage.cs:769-774`,
    /// `MessageBuffer.cs:3752-3757`, host→client only).
    pub const SMART_TEXT: u8 = 107;
    /// `108 WiredCannonShot` (W4-g50, S1-I-4): `(short)` damage, float
    /// knockback, `(short)` x/y/angle/ammo, `(byte)` owner
    /// (`MessageID.cs:230`, `NetMessage.cs:1403-1411`,
    /// `MessageBuffer.cs:3758-3773`, host→client only, addressed to the owner).
    pub const WIRED_CANNON_SHOT: u8 = 108;
    /// `109 MassWireOperation` (W4-g50, S1-I-4): four `(short)`s + `(byte)`
    /// tool mode (`MessageID.cs:232`, `NetMessage.cs:1412-1418`,
    /// `MessageBuffer.cs:3774-3788`, client→host only).
    pub const MASS_WIRE_OPERATION: u8 = 109;
    /// `110 MassWireOperationPay` (W4-g50, S1-I-4): `(short)` item type,
    /// `(short)` count, `(byte)` player (`MessageID.cs:234`,
    /// `NetMessage.cs:1419-1423`, `MessageBuffer.cs:3789-3808`,
    /// host→client only, addressed to the billed player).
    pub const MASS_WIRE_OPERATION_PAY: u8 = 110;
    /// `111 ToggleParty` (W4-g50, S1-I-4): no body (`MessageID.cs:236`,
    /// `BirthdayParty.cs:154`'s client ask, `MessageBuffer.cs:3809-3814`,
    /// client→host only).
    pub const TOGGLE_PARTY: u8 = 111;
    /// `112 SpecialFX` (W4-g50, S1-I-4): `(byte)` effect, `(int)` x/y,
    /// `(byte)` b4, `(short)` s5, `(byte)` b6 (`MessageID.cs:238`,
    /// `NetMessage.cs:1424-1431`, `MessageBuffer.cs:3815-3840`,
    /// host→client only).
    pub const SPECIAL_FX: u8 = 112;
    /// `113 CrystalInvasionStart` (W4-g50, S1-I-4): `(short)x, (short)y`
    /// (`MessageID.cs:240`, `NetMessage.cs:1432-1435`,
    /// `MessageBuffer.cs:3841-3854`, client→host only).
    pub const CRYSTAL_INVASION_START: u8 = 113;
    /// `114 CrystalInvasionWipeAllTheThingsss` (W4-g50, S1-I-4): no body
    /// (`MessageID.cs:242`, `DD2Event.cs:636`'s host broadcast,
    /// `MessageBuffer.cs:3855-3860`, host→client only).
    pub const CRYSTAL_INVASION_WIPE: u8 = 114;
    /// `116 CrystalInvasionSendWaitTime` (W4-g50, S1-I-4): one raw `(int)`
    /// (`MessageID.cs:246`, `NetMessage.cs:1436-1438`,
    /// `MessageBuffer.cs:3861-3866`, host→client only).
    pub const CRYSTAL_INVASION_WAIT: u8 = 116;
    /// `117 PlayerHurtV2` (W4-g50, S1-I-4): `(byte)` player, the death
    /// reason, `(short)` damage, `(byte)(dir+1)`, the crit/pvp BitsByte,
    /// `(sbyte)` cooldown (`MessageID.cs:248`, `NetMessage.cs:1439-1446`,
    /// `MessageBuffer.cs:3867-3886`). Both directions.
    pub const PLAYER_HURT_V2: u8 = 117;
    /// `118 PlayerDeathV2` (W4-g50, S1-I-4): `(byte)` player, the death
    /// reason, `(short)` damage, `(byte)(dir+1)`, the pvp BitsByte
    /// (`MessageID.cs:250`, `NetMessage.cs:1447-1453`,
    /// `MessageBuffer.cs:3887-3904`). Both directions.
    pub const PLAYER_DEATH_V2: u8 = 118;
    /// `120 Emoji` (W4-g50, S1-I-4): `(byte)` player, `(byte)` emote id
    /// (`MessageID.cs:254`, `NetMessage.cs:1454-1457`,
    /// `MessageBuffer.cs:3905-3919`, client→host only).
    pub const EMOJI: u8 = 120;
    /// `83 Unused83` (W4-g50, S1-I-4): deprecated (`MessageID.cs:179-180`,
    /// `[Old]`), no writer arm; the reader's ignore group
    /// (`MessageBuffer.cs:4483-4490`, with 15/25/26/44/67/93).
    pub const UNUSED_83: u8 = 83;
    /// `93 SocialHandshake` (W4-g50, S1-I-4): deprecated (`MessageID.cs:200`),
    /// no writer arm; the reader's ignore group (`MessageBuffer.cs:4483-4490`).
    pub const UNUSED_93: u8 = 93;
    /// `85 QuickStackChests` (W4-g50, S1-I-4): the client's quick-stack ask —
    /// `(int)` slot count + `(short)` slot ids + `(byte)` smartStack
    /// (`MessageID.cs:184`, `NetMessage.cs:1205-1210`,
    /// `MessageBuffer.cs:3261-3266`); the host's answer is the blocked-chest
    /// list (`NetMessage.cs:1213`, `MessageBuffer.cs:3267-3270`).
    pub const QUICK_STACK_CHESTS: u8 = 85;
    /// `94 DevCommands` (W4-g50, S1-I-4): `string` command + `(int)` number +
    /// `(float)` number2 + `(float)` number3 (`MessageID.cs:202`,
    /// `NetMessage.cs:1327-1332`, `MessageBuffer.cs:3463-3518`). Both
    /// directions — the debug-command sync (`DebugOptions.SyncToJoiningPlayer`).
    pub const DEV_COMMANDS: u8 = 94;
    pub const PLAYER_STATE: u8 = 13; // NetMessage.cs:446-513 / MessageBuffer.cs:952-1059
    pub const ITEM_DROP: u8 = 21; // NetMessage.cs:646-669 / MessageBuffer.cs:1445-1535
    pub const ITEM_OWNER: u8 = 22; // NetMessage.cs:675-685 / MessageBuffer.cs:1552-1562
    /// `34 ChestUpdates` (S1-I-1 wave I): the chest/dresser/piggy-bank
    /// place/destroy relay (`NetMessage.cs:913-927`,
    /// `MessageBuffer.cs:1984-2158`; the client send sites
    /// `Player.cs:53787-53795`).
    pub const CHEST_UPDATES: u8 = 34;
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
    /// `56 UniqueTownNPCInfoSyncRequest` (`MessageID.cs:125`) — the town NPC's
    /// given name + variation (`NetMessage.cs:1046-1053`,
    /// `MessageBuffer.cs:2612-2630`). W4-g49 (transplanted from W4-d92).
    pub const UNIQUE_TOWN_NPC_INFO: u8 = 56;
    /// `61 SpawnBossUseLicenseStartEvent` (`MessageID.cs:135`) — a boss/event
    /// summon (`NetMessage.cs:1073-1076`, `MessageBuffer.cs:2740-2878`).
    /// W4-g49 (transplanted from W4-d99).
    pub const SPAWN_BOSS_EVENT: u8 = 61;
    /// `63 SyncTilePaintOrCoating` (`MessageID.cs:139`) — a tile paint/coat
    /// (`NetMessage.cs:1081-1087`, `MessageBuffer.cs:2909-2928`). W4-g49
    /// (transplanted from W4-d90).
    pub const TILE_PAINT: u8 = 63;
    /// `64 SyncWallPaintOrCoating` (`MessageID.cs:141`) — a wall paint/coat
    /// (`NetMessage.cs:1081-1087`, `MessageBuffer.cs:2929-2948`). W4-g49.
    pub const WALL_PAINT: u8 = 64;
    /// `65 TeleportEntity` (`MessageID.cs:143`) — a player/NPC teleport
    /// (`NetMessage.cs:1088-1109`, `MessageBuffer.cs:2949-3035`). W4-g49
    /// (transplanted from W4-d99).
    pub const TELEPORT_ENTITY: u8 = 65;
    /// `66 Unknown66` (`MessageID.cs:145`) — the Moon Leech Clot's owner heal
    /// (`NetMessage.cs:1110-1113`, `MessageBuffer.cs:3036-3055`). W4-g49
    /// (transplanted from W4-d94).
    pub const HEAL_PLAYER: u8 = 66;
    /// `68 Unknown68` (`MessageID.cs:149`) — the client's UUID string
    /// (`NetMessage.cs:1114-1116`, `MessageBuffer.cs:3056-3058`). W4-g49
    /// (transplanted from W4-d95).
    pub const CLIENT_UUID: u8 = 68;
    /// `73 RequestTeleportationByServer` (`MessageID.cs:159`) — a conch/potion/
    /// shellphone teleport request (`NetMessage.cs:1142-1144`,
    /// `MessageBuffer.cs:3127-3146`). W4-g49 (transplanted from W4-d99).
    pub const REQUEST_TELEPORT: u8 = 73;
    /// `75 AnglerQuestFinished` (`MessageID.cs:163`) — no body either side
    /// (`MessageBuffer.cs:3154-3163` adds the sender's name on the host).
    /// W4-g49 (transplanted from W4-d99).
    pub const ANGLER_QUEST_FINISHED: u8 = 75;
    /// `77 TemporaryAnimation` (`MessageID.cs:167`) — a temporary tile
    /// animation (`NetMessage.cs:1154-1159`, `MessageBuffer.cs:3183-3185`).
    /// W4-g49 (transplanted from W4-d99).
    pub const TEMPORARY_ANIMATION: u8 = 77;
    /// `78 InvasionProgressReport` (`MessageID.cs:169`) — the invasion/moon/
    /// DD2 progress bar (`NetMessage.cs:1160-1165`,
    /// `MessageBuffer.cs:3186-3191`). W4-g49 (transplanted from W4-d99).
    pub const INVASION_PROGRESS: u8 = 78;
    pub const TRAVEL_SHOP: u8 = 72; // MessageID.cs:157 / NetMessage.cs:1134-1141 / MessageBuffer.cs:3118-3126
    pub const ANGLER_QUEST: u8 = 74; // MessageID.cs:161 / NetMessage.cs:1145-1148 / MessageBuffer.cs:3147-3153
    pub const REVENGE_MARKER: u8 = 126; // MessageID.cs:266 / NetMessage.cs:1515-1517 / MessageBuffer.cs:4043-4048
    pub const DEAD_PLAYER: u8 = 135; // MessageID.cs:284 / NetMessage.cs:1569-1571 / MessageBuffer.cs:4195-4203
    /// `140 SetMiscEventValues` (`MessageID.cs:294`) — kind 0 is the credits
    /// roll's remaining time (`NetMessage.cs:1591-1594`,
    /// `CreditsRollEvent.cs:27-33`, `MessageBuffer.cs:4238-4249`); kinds 1/2
    /// are the client's town-slime transform requests (`NPC.cs:19164`/
    /// `:19140`). W4-g51 wired kind 0 (the world field + the join push) and
    /// the host's kind 1/2 unlock gates; the Transform halves park with the
    /// unported town-slime kinds.
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
    /// `128 LandGolfBallInCup` (`MessageID.cs:270`) — a client's golf ball
    /// (a projectile) dropped into a cup: `(byte)owner + (ushort)hitX +
    /// (ushort)hitY + (ushort)numberOfHits + (ushort)ballType`
    /// (`NetMessage.cs:1521-1527`, sent by `GolfHelper.ContactListener.
    /// PutBallInCup` at `netMode == 1`, `GolfHelper.cs:107-110`). The host
    /// relays it to the others (`MessageBuffer.cs:4058-4074`, the
    /// `ignoreClient = owner` resend); the client runs the cup text/effects
    /// (`PutBallInCup_TextAndEffects`, the golf FX of M4-GOLF-1).
    pub const GOLF_BALL_IN_CUP: u8 = 128;

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

    // ---- W4-g49: the F21-TAIL apply ids (codecs transplanted byte-verbatim
    // from origin/task/W4-f21-net-ids-verify-tail 5274a097, batches 3/4/5/7) ----
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
    /// `79 PlaceObject` (`MessageID.cs:171`) — writer
    /// `NetMessage.cs:1166-1174`, reader `MessageBuffer.cs:3192-3215`
    /// (host spam count + `InWorld(…, 10)` + section-sent gate, then
    /// `WorldGen.PlaceObject` + `SendObjectPlacement` relay).
    /// `SendObjectPlacement` (`NetMessage.cs:2573-2588`) sends it BOTH
    /// ways: client placement asks (`Player.cs:40147`/`:45819`,
    /// `Projectile.cs:25356`) and the host's placement relay.
    pub const PLACE_OBJECT: u8 = 79;
    /// `44 Unknown44` (`MessageID.cs:99-100`, `[Old("Deprecated.")]`) — no
    /// writer arm, no body; the same reader ignore group
    /// (`MessageBuffer.cs:4483-4490`). d96's census row confirms it dead.
    pub const UNKNOWN_44: u8 = 44;
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
    /// `MessageBuffer.cs:2531-2564` runs on BOTH sides (the `netMode == 2`
    /// relay halves are host-only): kind 1 `Chest.Unlock`, 2
    /// `WorldGen.UnlockDoor`, 3 `Chest.Lock`, each host-relayed as `52` +
    /// `SendTileSquare(…, 2)`. All send sites are `netMode == 1`
    /// (`Player.cs:32813`/`:32832`/`:33197`/`:33218`/`:41660`); the host
    /// relay (`:2541`/`:2550`/`:2559`) gives the client an S2c read too.
    pub const LOCK_AND_UNLOCK: u8 = 52;
    /// `67 Unknown67` (`MessageID.cs:147`) — no writer arm, no body; the
    /// reader ignore group (`MessageBuffer.cs:4483-4490`). Note the accept
    /// gate (`:165`) does NOT whitelist it pre-spawn (93 is whitelisted
    /// there but still ignored by the reader group).
    pub const UNKNOWN_67: u8 = 67;

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
    /// `127 RemoveRevengeMarker` (`MessageID.cs:268`) — the 126 marker's
    /// removal twin (`NetMessage.cs:1518-1520`, `MessageBuffer.cs:4049-4057`).
    /// W4-g51 lifted the parked d99 codec and wired the client arm.
    pub const REMOVE_REVENGE_MARKER: u8 = 127;
    /// `130 FishOutNPC` (`MessageID.cs:274`) — a client fished an NPC out
    /// (`NetMessage.cs:1528-1532`, `MessageBuffer.cs:4086-4120`). W4-g51
    /// lifted the parked d92 codec and wired the host spawn arm.
    pub const FISH_OUT_NPC: u8 = 130;
    /// `131 TamperWithNPC` (`MessageID.cs:276`) — the host tames an NPC's
    /// immunity (`NetMessage.cs:1533-1541`, `MessageBuffer.cs:4122-4133`).
    /// W4-g51 lifted the parked d92 codec and wired the client arm.
    pub const TAMPER_WITH_NPC: u8 = 131;
    /// `134 UpdatePlayerLuckFactors` (`MessageID.cs:282`) — a player's luck
    /// factors (`NetMessage.cs:1555-1567`, `MessageBuffer.cs:4164-4193`).
    /// W4-g51 lifted the parked d97 codec and wired the apply+relay arms.
    pub const UPDATE_PLAYER_LUCK_FACTORS: u8 = 134;
    /// `136 SyncCavernMonsterType` (`MessageID.cs:286`) — the cavern spawn
    /// grid (`NetMessage.cs:1572-1582`, `MessageBuffer.cs:4204-4217`).
    /// W4-g51 lifted the parked d92 codec and wired the client store
    /// (`World::cavern_monster_type`) + the join push (`:870`).
    pub const CAVERN_MONSTER_TYPES: u8 = 136;
    /// `137 RequestNPCBuffRemoval` (`MessageID.cs:288`) — a client asks the
    /// host to drop an NPC buff (`NetMessage.cs:1583-1586`,
    /// `MessageBuffer.cs:4219-4228`). W4-g51 lifted the parked d92 codec and
    /// wired the host gate (the `CanBeRemovedByNetMessage` set is empty).
    pub const NPC_BUFF_REMOVAL_REQUEST: u8 = 137;
    /// `141 LucyTheAxeMessage` (`MessageID.cs:296`) — a talking-axe popup
    /// (`NetMessage.cs:1595-1602`, `MessageBuffer.cs:4265-4281`). W4-g51
    /// lifted the parked d99 codec; the host relay is wired, the client
    /// `CreateFromNet` popup is parked (no Lucy popup system yet).
    pub const LUCY_POPUP: u8 = 141;
    /// `143 CrystalInvasionRequestedToSkipWaitTime` (`MessageID.cs:300`) —
    /// a client asked to skip the DD2 wave break, no body
    /// (`DD2Event.cs:1790`, `MessageBuffer.cs:4298-4303`). W4-g51 lifted the
    /// parked d99 codec and wired the host `AttemptToSkipWaitTime` arm.
    pub const CRYSTAL_INVASION_SKIP: u8 = 143;
    /// `150 SpectatePlayer` (`MessageID.cs:314`) — a player (re)targets
    /// spectating (`NetMessage.cs:1639-1642`, `MessageBuffer.cs:4361-4384`).
    /// W4-g51 lifted the parked d97 codec; the apply is parked (the port has
    /// no spectating system yet).
    pub const SPECTATE_PLAYER: u8 = 150;
    /// `153 NPCDebuffDamage` (`MessageID.cs:320`) — an NPC's debuff tick
    /// damage, host-sent (`NetMessage.cs:1646-1649`,
    /// `MessageBuffer.cs:4408-4417`). W4-g51 lifted the parked d92 codec and
    /// wired the client `GetHurtByDebuff` arm.
    pub const NPC_DEBUFF_DAMAGE: u8 = 153;
    /// `162 DamageNPCAck` (`MessageID.cs:338`) — the host's ack of a `28`
    /// strike, no body (`MessageBuffer.cs:1768-1771` sends, `:1819-1824`
    /// pops `NPC.pendingDamages[0]`). W4-g51 lifted the parked d92 codec and
    /// wired the queue: the client strike pushes (`send_npc_strike`), the
    /// `23` reader subtracts the pending sum, the ack pops FIFO.
    pub const NPC_STRIKE_ACK: u8 = 162;

    // ---- W4-g51 f21 cross-check: the SMALL apply arms lifted from the
    // W4-f21 tail census (origin/task/W4-f21-net-ids-verify-tail 5274a097).
    // The bodyless/ignore/small-store ids whose apply is self-contained.
    // The BIG ones (the TE families 121/122/123/124/133/149/156, the dryad
    // 144) landed in the FINAL phase below (the ANALYSIS status table).
    /// `129 FinishedConnectingToServer` (`MessageID.cs:272`) — NO writer
    /// case: `TrySendData(129, whoAmI)` (`MessageBuffer.cs:937`, the
    /// State-3→10 join arm) writes a bare frame. The reader is client-only
    /// (`MessageBuffer.cs:4075-4085`): the team re-push (`45` when
    /// `team > 0`), `FixUIScale`, `TrySetPreparationState(ProcessingData)`.
    /// S2c-only, bodyless.
    pub const FINISHED_CONNECTING: u8 = 129;
    /// `138 ClientSyncedInventory` (`MessageID.cs:290`) — NO writer case
    /// (`SendData(138)` at `Main.cs:17700`, after the inventory-sync sweep,
    /// writes a bare frame) and NO reader case: the frame falls to the
    /// `default` arm (`MessageBuffer.cs:4477-4482`), which only boots a
    /// State-0 sender and otherwise does nothing. C2s-only, bodyless.
    pub const CLIENT_SYNCED_INVENTORY: u8 = 138;
    /// `139 SetCountsAsHostForGameplay` (`MessageID.cs:292`) — writer
    /// `NetMessage.cs:1587-1590` (`(byte)player + (number2 == 1f)` bool),
    /// reader `MessageBuffer.cs:4230-4237` (`netMode != 2` →
    /// `Main.countsAsHostForGameplay[id] = flag`). Host send: the join
    /// arm's `DoesPlayerSlotCountAsAHost` push (`MessageBuffer.cs:934`).
    /// S2c-only.
    pub const COUNTS_AS_HOST: u8 = 139;
    /// `145 SyncItemsWithShimmerDeprecated` (`MessageID.cs:304`) — DEAD:
    /// no writer case, no reader case (the default arm drops it), zero send
    /// sites tree-wide. The shimmer item sync became `146 ShimmerActions`.
    /// Constant + census doc only (the 24 precedent).
    pub const SHIMMER_ITEMS_DEPRECATED: u8 = 145;
    /// `146 ShimmerActions` (`MessageID.cs:306`) — writer
    /// `NetMessage.cs:1611-1623` (`(byte)kind` then kind 0: a Vector2;
    /// kind 1: a Vector2 + `(int)amount`), reader
    /// `MessageBuffer.cs:4310-4327` (client-only: kind 0
    /// `WorldItem.ShimmerEffect`, kind 1 `AddCoinLuck`). Host sends:
    /// `WorldItem.cs:1665` (the shimmer FX) + `:1716` (the coin luck).
    /// S2c-only.
    pub const SHIMMER_ACTIONS: u8 = 146;
    /// `148 SyncItemCannotBeTakenByEnemiesDeprecated` (`MessageID.cs:310`)
    /// — DEAD: no writer case, no reader case, zero send sites tree-wide.
    /// The enemy-grab lock became the `21` bit-3 tail. Constant + census
    /// doc only (the 24/145 precedent).
    pub const ITEM_ENEMY_LOCK_DEPRECATED: u8 = 148;
    /// `152 ItemUseSound` (`MessageID.cs:318`) — writer
    /// `NetMessage.cs:1643-1645` (`(byte)player` only), reader
    /// `MessageBuffer.cs:4386-4407` (host id-force + relay; client plays
    /// the peer's held item's `UseSound` at its Centre with
    /// `useSoundPitch`). Client send: `Player.cs:51692`. Both directions.
    pub const ITEM_USE_SOUND: u8 = 152;
    /// `160 ItemPosition` (`MessageID.cs:334`) — writer
    /// `NetMessage.cs:1666-1672` (`(short)slot + WriteVector2(position)`),
    /// reader `MessageBuffer.cs:4463-4470` (`netMode != 2` → the position
    /// snap). Host send: the section item sweep
    /// (`WorldItem.SyncItemsInSection`, `WorldItem.cs:1864`). S2c-only.
    pub const ITEM_POSITION: u8 = 160;
    /// `161 HostToken` (`MessageID.cs:336`) — writer
    /// `NetMessage.cs:1673-1675` (`text.ToString()` — a plain string),
    /// reader `MessageBuffer.cs:4471-4476` (host-only:
    /// `player.host = HostToken non-empty && == text`). Client send: the
    /// join push when `Netplay.HostToken` is set (`MessageBuffer.cs:275`).
    /// Gate-whitelisted pre-spawn (`:165`). C2s-only.
    pub const HOST_TOKEN: u8 = 161;
    /// `121 TEDisplayDollDataSync` (`MessageID.cs:256`) — writer
    /// `NetMessage.cs:1458-1474` (byte player + int entityId + byte
    /// itemIndex + byte command + `WriteData`/`WriteDummySync`), reader
    /// `MessageBuffer.cs:3920-3941` (host id-force + relay; client
    /// `ReadData`). Both directions. W4-g51 FINAL.
    pub const TE_DISPLAY_DOLL_SYNC: u8 = 121;
    /// `122 RequestTileEntityInteraction` (`MessageID.cs:258`) — writer
    /// `NetMessage.cs:1475-1478` (int entityId + byte player; -1 clears),
    /// reader `MessageBuffer.cs:3942-3977` (host IsOccupied gate + relay;
    /// client Set/Clear anchor). Both directions. W4-g51 FINAL.
    pub const TE_INTERACTION_REQUEST: u8 = 122;
    /// `123 WeaponsRackTryPlacing` (`MessageID.cs:260`) — the exact 89
    /// body (`NetMessage.cs:1479-1488`, reader `MessageBuffer.cs:3978-3994`,
    /// host-only `TEWeaponsRack.TryPlacing`). Client send:
    /// `TEWeaponsRack.cs:212`. C2s-only. W4-g51 FINAL.
    pub const WEAPONS_RACK_TRY_PLACING: u8 = 123;
    /// `124 TEHatRackItemSync` (`MessageID.cs:262`) — writer
    /// `NetMessage.cs:1489-1508` (byte player + int entityId + byte slot
    /// with the DYE +2 fold, then `TEHatRack.WriteItem` — STACK BEFORE
    /// PREFIX), reader `MessageBuffer.cs:3995-4022` (host id-force +
    /// relay; client `ReadItem`). Both directions. W4-g51 FINAL.
    pub const TE_HAT_RACK_ITEM_SYNC: u8 = 124;
    /// `133 FoodPlatterTryPlacing` (`MessageID.cs:280`) — the exact 89
    /// body (`NetMessage.cs:1545-1554`, reader `MessageBuffer.cs:4147-4163`,
    /// host-only `TEFoodPlatter.TryPlacing`). Client send:
    /// `TEFoodPlatter.cs:133`. C2s-only. W4-g51 FINAL.
    pub const FOOD_PLATTER_TRY_PLACING: u8 = 133;
    /// `144 RequestQuestEffect` (`MessageID.cs:302`) — NO writer case:
    /// `SendData(144)` writes a bare frame (`Main.cs:40287`, the Stardew
    /// dryad bit). Reader `MessageBuffer.cs:4304-4309`: host-only
    /// `NPC.HaveDryadDoStardewAnimation` (`NPC.cs:19184-19229`). C2s-only,
    /// bodyless. W4-g51 FINAL.
    pub const REQUEST_QUEST_EFFECT: u8 = 144;
    /// `149 DeadCellsDisplayJarTryPlacing` (`MessageID.cs:312`) — the exact
    /// 89 body (`NetMessage.cs:1629-1638`, reader
    /// `MessageBuffer.cs:4344-4360`, host-only
    /// `TEDeadCellsDisplayJar.TryPlacing`). Client send:
    /// `TEDeadCellsDisplayJar.cs:150`. C2s-only. W4-g51 FINAL.
    pub const DEAD_CELLS_JAR_TRY_PLACING: u8 = 149;
    /// `156 TELeashedEntityAnchorPlaceItem` (`MessageID.cs:326`) — writer
    /// `NetMessage.cs:1654-1658` (three shorts: x, y, itemType), reader
    /// `MessageBuffer.cs:4439-4449` (host-only `Point16` lookup +
    /// `InsertItem`). Client sends: `TELeashedEntityAnchorWithItem.cs:60`,
    /// `TEKiteAnchor.cs:66`. C2s-only. W4-g51 FINAL.
    pub const LEASHED_ANCHOR_PLACE_ITEM: u8 = 156;

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
    /// `39 ReleaseItemOwnership` (`MessageID.cs:89`) — live: writer
    /// `NetMessage.cs:948-951`, reader
    /// `MessageBuffer.cs:2232-2255`; the client releases its reservation
    /// (`WorldItem.cs:256`), the host re-assigns and may answer with `22`
    /// (`WorldItem.cs:264` sends it to one client).
    pub const RELEASE_ITEM_OWNERSHIP: u8 = 39;
    /// `93 SocialHandshake` (`MessageID.cs:200`) — no writer arm, no body;
    /// the reader ignore group (`MessageBuffer.cs:4483-4490`). Unlike its
    /// group peers it IS accept-gate-whitelisted pre-spawn
    /// (`MessageBuffer.cs:165`'s `b != 93` escape) — the C# accepts the
    /// frame early, then drops it.
    pub const SOCIAL_HANDSHAKE: u8 = 93;

    // ---- W4-c27: the entity-state ids (apply/relay arms land with them) ----
    pub const SET_TIME: u8 = 18; // MessageID.cs:45 / NetMessage.cs:531-536 / MessageBuffer.cs:1290-1298
    pub const MISC_DATA_SYNC: u8 = 51; // MessageID.cs:115 / NetMessage.cs:1010-1013 / MessageBuffer.cs:2482-2530
    pub const ADD_PLAYER_BUFF_PVP: u8 = 55; // MessageID.cs:123 / NetMessage.cs:1041-1045 / MessageBuffer.cs:2594-2611
    pub const SYNC_DODGE: u8 = 62; // MessageID.cs:137 / NetMessage.cs:1077-1080 / MessageBuffer.cs:2879-2908
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
#[derive(Clone, Debug)]
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
    /// `5 PlayerInventory` messages (`MessageBuffer.cs:399-462`). The key
    /// is the vanilla `PlayerItemSlotID.SlotId` (i16 — the bank4 rows ride
    /// 699+, W4-g50's 85 restore re-sends them; W4-c28: a u8 key truncated
    /// bank4/loadout ids).
    pub inventory: HashMap<i16, NetItem>,
    /// `Main.player[id].buffType[]`'s live ids, fed by `50 PlayerBuffs`
    /// (`MessageBuffer.cs:2454-2480` sets every listed buff's time to 60).
    pub buffs: Vec<u16>,
    /// `Main.player[id].chest` (`Player.cs` field): which container the peer
    /// has open, fed by `80 SyncPlayerChestIndex`
    /// (`MessageBuffer.cs:3216-3227`). `None` = closed (`-1`), the `Default`.
    pub chest: Option<i16>,
    /// `Player.hostile` (`Player.cs:2570`) — fed by `30 TogglePVP`
    /// (`MessageBuffer.cs:1859`, W4-g48); the `117` hurt gate's mutual-PvP
    /// term reads it (`MessageBuffer.cs:3870`; W4-g50).
    pub hostile: bool,
    /// `Player.team` (`Player.cs`) — fed by `12 PlayerSpawn`
    /// (`MessageBuffer.cs:914`) and `45 TeamChange` (`MessageBuffer.cs:2336`,
    /// W4-g49); the PVP chat line's colour keys on it (`:1864`,
    /// `Main.teamColor`), the `102` fan's team gate reads it
    /// (`MessageBuffer.cs:3666`; W4-g50). `i32` = the C# field's type (the
    /// W4-g48 `u8` widened at the g49 union; the wire byte is `u8`).
    pub team: i32,
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
    /// `Main.player[id].statMana`/`statManaMax`, fed by `42 Unknown42`
    /// (`MessageBuffer.cs:2290-2306`). W4-g49.
    pub mana: i32,
    pub max_mana: i32,
    /// `Main.player[id].anglerQuestsFinished`/`golferScoreAccumulated`, fed
    /// by `76 QuestsCountSync` (`MessageBuffer.cs:3174-3175`). W4-g49.
    pub angler_quests_finished: i32,
    pub golfer_score: i32,
    /// `Main.player[id].stealth` (`Player.cs:1562`, default 1f), fed by
    /// `84 PlayerStealth` (`MessageBuffer.cs:3246-3260`; W4-g50).
    pub stealth: f32,
    /// `Main.player[id].emoteTime` (`Player.cs:792`), fed by the `91`
    /// reader's player-anchor arm (`MessageBuffer.cs:3421`; W4-g50).
    pub emote_time: i32,
    /// `Main.player[id].lastPortalColorIndex`, fed by the `96` reader
    /// (`MessageBuffer.cs:3549`; W4-g50).
    pub last_portal_color_index: i16,
    /// `Main.player[id].MinionRestTargetPoint` (`Player.cs:3006`), fed by
    /// `99 MinionRestTargetUpdate` (`MessageBuffer.cs:3577`; W4-g50).
    pub minion_rest_target_point: (f32, f32),
    /// `Main.player[id].MinionAttackTargetNPC` (`Player.cs:3008`, -1 none),
    /// fed by `115 MinionAttackTargetUpdate` (`MessageBuffer.cs:3591`;
    /// W4-g50).
    pub minion_attack_target_npc: i32,
    /// `Main.player[id]`'s luck factors, fed by `134 UpdatePlayerLuckFactors`
    /// (`MessageBuffer.cs:4164-4193`). Stored-not-simulated on the peer (the
    /// port does not run remote player sims); the local player's live state
    /// is `Player.luck_state`. `None` = never synced (the C# field defaults).
    pub luck: Option<UpdatePlayerLuckFactors>,
    /// `Main.player[id].piggyBankProjTracker` / `.voidLensChest`, fed by
    /// `142 SyncProjectileTrackers` (`MessageBuffer.cs:4282-4297`).
    /// Stored-not-simulated (the port has no piggy-bank/void-lens projectile
    /// tracking consumer yet). `None` = never synced (the C# field defaults).
    pub trackers: Option<ProjectileTrackers>,
    /// `Main.player[id].CurrentLoadoutIndex` / `.hideVisibleAccessory`, fed
    /// by `147 SyncLoadout` (`MessageBuffer.cs:4328-4343`). The loadout swap
    /// is unported (no loadout system); the accessory-visibility bits are
    /// stored-not-simulated on the peer. `None` = never synced.
    pub loadout: Option<SyncLoadout>,
    /// `Main.player[id].host`, fed by `161 HostToken`
    /// (`MessageBuffer.cs:4471-4476`). The port has no `Netplay.HostToken`
    /// config, so the compare refuses every claim and this stays false
    /// (stored-not-simulated). W4-g51 f21 cross-check.
    pub host: bool,
    /// `Main.countsAsHostForGameplay[id]`, fed by `139 SetCountsAsHostForGameplay`
    /// (`MessageBuffer.cs:4230-4237`). The flag gates a co-op peer's host-side
    /// gameplay authority (`CreativePowersHelper.cs:217`); the port stores it
    /// on the peer snapshot (the creative-power authority check is unported).
    /// W4-g51 f21 cross-check.
    pub counts_as_host: bool,
    /// `Main.player[id].zone1..5` + `townNPCs`, fed by `36 SyncPlayerZone`
    /// (`MessageBuffer.cs:2187-2192`). W4-c27.
    pub zone: PlayerZone,
    /// `Player.itemRotation`/`itemAnimation`, fed by `41
    /// ItemRotationAndAnimation` (`MessageBuffer.cs:2281-2282`); `channel`
    /// re-derived from the selected inventory item (`:2283`). W4-c27.
    pub item_rotation: f32,
    pub item_animation: i16,
    pub channel: bool,
}

impl Default for PlayerNet {
    /// The field defaults the C# player ctor carries for the synced subset
    /// (`Player.cs:1562`'s `stealth = 1f`, `:3008`'s -1 attack target);
    /// everything else stays zeroed.
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            hp: 0,
            max_hp: 0,
            facing: 0,
            selected: 0,
            on_ground: false,
            leg_frame: 0,
            body_frame: 0,
            off_x: 0.0,
            off_y: 0.0,
            inventory: HashMap::new(),
            buffs: Vec::new(),
            chest: None,
            hostile: false,
            team: 0,
            spawn_x: 0,
            spawn_y: 0,
            respawn_timer: 0,
            deaths_pve: 0,
            deaths_pvp: 0,
            dead: false,
            zones: [0; 5],
            town_npcs: 0,
            talk_npc: None,
            mana: 0,
            max_mana: 0,
            angler_quests_finished: 0,
            golfer_score: 0,
            stealth: 1.0,
            emote_time: 0,
            last_portal_color_index: 0,
            minion_rest_target_point: (0.0, 0.0),
            minion_attack_target_npc: -1,
            luck: None,
            trackers: None,
            loadout: None,
            host: false,
            counts_as_host: false,
            zone: PlayerZone {
                player_id: 0,
                zone1: BitsByte::new(),
                zone2: BitsByte::new(),
                zone3: BitsByte::new(),
                zone4: BitsByte::new(),
                zone5: BitsByte::new(),
                town_npcs: 0,
            },
            item_rotation: 0.0,
            item_animation: 0,
            channel: false,
        }
    }
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
/// `favorited` and bit 1 the server's "blocked slot" indicator
/// (`RestoreToPlayer`'s `flag ? 1 : 0`, `QuickStacking.cs:355`; the client
/// pulses the slot red — the pulse visual is unported, W4-g50).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NetItem {
    pub kind: i16,
    pub stack: i16,
    pub prefix: u8,
    pub favorited: bool,
    /// BitsByte bit 1 — `ItemSlot.IndicateBlockedSlot`'s flag (the 85
    /// quick-stack restore re-send carries it; every other 5 send is 0).
    pub blocked: bool,
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

/// `msg 56 UniqueTownNPCInfoSyncRequest`'s body (`NetMessage.cs:1046-1053`):
/// `(short)` npc slot, then — ONLY in the host's answer (`netMode == 2`) —
/// `GivenName` (`BinaryWriter.Write(string)`: 7-bit length + UTF-8) and
/// `(int)` townNpcVariationIndex. The client's request is the bare slot
/// (`NPC.cs:90564`); the host's read arm answers it back to the asker
/// (`MessageBuffer.cs:2626`), the client's read arm stores the pair behind
/// the `0..Main.maxNPCs` slot gate (`:2615-2623`). W4-g49 (from W4-d92).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniqueTownNpcInfo {
    pub npc: i16,
    /// Present only in the host's answer (`NetMessage.cs:1048` netMode gate).
    pub info: Option<(String, i32)>,
}

/// `msg 61 SpawnBossInvader`'s body (`NetMessage.cs:1073-1076`): two shorts —
/// the player to spawn on and the event selector. Positive selectors are
/// `NPCID` boss types gated by `NPCID.Sets.MPAllowedEnemies`
/// (`MessageBuffer.cs:2748-2753`); negatives are the event ladder. Client→
/// server only — every send site is a `netMode == 1` arm; the host answers
/// with `7`/`78` pushes. W4-g49 (from W4-d99).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnBossEvent {
    pub player: i16,
    pub event: i16,
}

/// `msg 63/64 SyncTilePaintOrCoating / SyncWallPaintOrCoating`'s shared body
/// (`NetMessage.cs:1081-1087`): the tile, the paint id byte and the coat flag
/// byte — `coat == 0` paints (`WorldGen.paintTile`/`paintWall`), anything
/// else coats (`paintCoatTile`/`paintCoatWall`,
/// `MessageBuffer.cs:2909-2948`), then the host relays. W4-g49 (from W4-d90).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TilePaint {
    pub x: i16,
    pub y: i16,
    pub paint: u8,
    /// `b12`/`b10`: 0 = paint, non-zero = illuminant coat.
    pub coat: u8,
}

/// `msg 65 TeleportEntity`'s body (`NetMessage.cs:1088-1109`): a BitsByte
/// (bit 0/1 = the kind's low two bits, bit 2 = "take the position from the
/// player", bit 3 = the extraInfo int follows), `(short)` entity slot, the
/// destination floats, `(byte)` teleport style, then the optional
/// `(int)` extraInfo (the portal color for style 4). Kinds: 0 player
/// teleport, 1 NPC teleport (host, style != 4), 2 player-to-player, 3 the
/// client's ack. The host forces the entity to the sender
/// (`MessageBuffer.cs:2953-2956`), overrides the position from the live player
/// when bit 2 is set (`:2979-2982`). Both directions. The C# writes bit 3 only
/// when `number7 != 0`: senders pass `None` for 0. W4-g49 (from W4-d99).
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

/// `msg 66 Unknown66`'s body (`NetMessage.cs:1110-1113`): the player id then
/// the heal amount as a `short`. Sent by the Moon Leech Clot's contact arm
/// (`Projectile.cs:28513-28524`); the host applies the same clamp and relays
/// (`MessageBuffer.cs:3036-3055`, the `num71 > 0` gate lives in the apply arm,
/// not the codec). W4-g49 (from W4-d94).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealPlayer {
    pub player_id: u8,
    pub heal: i16,
}

/// `msg 68 Unknown68`'s body (`NetMessage.cs:1114-1116`): one
/// `BinaryWriter.Write(string)` — `Main.clientUUID`, the client's
/// `Guid.NewGuid().ToString()` (`Main.cs:10618`). Sent in the `3 PlayerId`
/// join push right after `4 SyncPlayer` (`MessageBuffer.cs:248-249`); the
/// host's arm reads the string and drops it (`:3056-3058`). The id is
/// gate-whitelisted pre-spawn (`MessageBuffer.cs:165`). W4-g49 (from W4-d95).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientUuid {
    pub uuid: String,
}

/// `msg 73 RequestTeleport`'s body (`NetMessage.cs:1142-1144`): one byte —
/// the teleport item kind: 0 teleportation potion, 1 magic conch
/// (`Player.cs:43029`), 2 demon conch (`:43062`), 3 shellphone home
/// (`:43103`), 4 no-space fallback (`:6069`). The host runs the matching
/// `Player` method on the sender (`MessageBuffer.cs:3127-3146`), which
/// answers with a `65`. Client→server only. W4-g49 (from W4-d99).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestTeleport {
    pub kind: u8,
}

/// `msg 77 SyncTilePicking`'s body (`NetMessage.cs:1154-1159`):
/// `(short)` animationType, `(ushort)` tileType, `(short)` x, `(short)` y —
/// `Animation.NewTemporaryAnimation`'s four args verbatim
/// (`MessageBuffer.cs:3183-3185`). Sent only by the host's
/// `SendTemporaryAnimation` (`NetMessage.cs:2590-2596`, `netMode == 2`) to
/// one client; server→client only. W4-g49 (from W4-d99).
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
/// (`MessageBuffer.cs:3186-3191`, client-only read). Server→client only.
/// W4-g49 (from W4-d99).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvasionProgress {
    pub progress: i32,
    pub progress_max: i32,
    pub icon: i8,
    pub wave: i8,
}

// ---- W4-g49 F21-TAIL: the transplanted bodies (byte-verbatim from
// origin/task/W4-f21-net-ids-verify-tail 5274a097) ----

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
/// byte then an `int`. Kind 0 is the credits roll's remaining frames
/// (`CreditsRollEvent.cs:27-33`, S2c); kinds 1/2 are the client's town-slime
/// transform requests (`NPC.cs:19164`/`:19140`, C2s) — the host applies
/// (`MessageBuffer.cs:4250-4260`). Both directions since W4-g51 id-140.
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
/// `NPC.Spawner.SpawnFaelings` — WIRED W4-g48, `spawner::spawn_faelings`;
/// the Shimmerfly 677 spawn is roster-gated until the kind lands).
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
    /// `56 UniqueTownNPCInfoSyncRequest` - a client asks for a town NPC's
    /// given name + variation (`NPC.cs:90564`; the host answers `:2626`).
    /// W4-g49.
    UniqueTownNpcInfo(UniqueTownNpcInfo),
    /// `61 SpawnBossInvader` - a client used a boss/event summon item
    /// (`Player.ItemCheck_UseBossSpawners`/`UseEventItems`). W4-g49.
    SpawnBossEvent(SpawnBossEvent),
    /// `63 SyncTilePaintOrCoating` - a client painted/coated a tile
    /// (`WorldGen.paintTile`/`paintCoatTile`). W4-g49.
    TilePaint(TilePaint),
    /// `64 SyncWallPaintOrCoating` - a client painted/coated a wall
    /// (`WorldGen.paintWall`/`paintCoatWall`). W4-g49.
    WallPaint(TilePaint),
    /// `65 TeleportEntity` - a client teleported (kind 0/2) or acks the
    /// host's teleport (kind 3). W4-g49.
    TeleportEntity(TeleportEntity),
    /// `66 Unknown66` - this client's Moon Leech Clot healed its owner
    /// (`Projectile.cs:28513-28524`). W4-g49.
    HealPlayer(HealPlayer),
    /// `68 Unknown68` — the client's UUID string, pushed right after
    /// `4 PlayerInfo` in the `3 PlayerId` join arm. W4-g49.
    ClientUuid(ClientUuid),
    /// `73 RequestTeleport` - a client used a conch/potion/shellphone
    /// (`Player.cs:43029`/`:43062`/`:43103`/`:6069`). W4-g49.
    RequestTeleport(RequestTeleport),
    /// `75 AnglerQuestFinished` - a client finished today's angler quest
    /// (`Main.cs:40254`). No body. W4-g49.
    AnglerQuestFinished,
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
    /// `58 InstrumentSound` — a client played its held instrument
    /// (`Player.cs:46544` etc.; host id-force + relay
    /// `MessageBuffer.cs:2642-2650`). W4-g49 (from W4-f21).
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
    /// `48 LiquidUpdate` — a client's single-tile liquid change
    /// (`NetMessage.sendWater`'s `netMode == 1` arm, `NetMessage.cs:2782`;
    /// reader `MessageBuffer.cs:2407-2446`). Deprecated in favour of module
    /// 82's liquid stream, but the arm is live.
    LiquidUpdate(LiquidUpdate),
    /// `52 LockAndUnlock` — a client unlocked/locked a chest or door
    /// (`Player.cs:32813`/`:33197`/`:41660`; host apply + relay
    /// `MessageBuffer.cs:2531-2564`). The send sites are all `netMode == 1`;
    /// the host relay (`:2541`/`:2550`/`:2559`) carries it back to the
    /// clients as S2c (the reader applies on both sides).
    LockAndUnlock(LockAndUnlock),
    /// `84 PlayerStealth` - this client's stealth value
    /// (`MessageBuffer.cs:3246-3260` forces the id and relays; W4-g50).
    PlayerStealth(PlayerStealth),
    /// `92 SyncExtraValue` - a coin pickup's credit delta for a town NPC
    /// (`WorldItem.cs:1186-1194`; `MessageBuffer.cs:3455-3459` adds it and
    /// relays the total; W4-g50).
    NpcExtraValue(NpcExtraValue),
    /// `87 TileEntityPlacement` - a client placed a tile entity
    /// (`TETrainingDummy.Hook_AfterPlacement` etc.;
    /// `MessageBuffer.cs:3294-3305` places it and answers with 86; W4-g50).
    TileEntityPlacement(TileEntityPlacement),
    /// `89 ItemFrameTryPlacing` - a client placed an item into a frame
    /// (`TEItemFrame.PlaceItemInFrame`, `TEItemFrame.cs:154`;
    /// `MessageBuffer.cs:3377-3393`; W4-g50).
    ItemFrameTryPlacing(TryPlacingItem),
    /// `96 TeleportPlayerThroughPortal` — the portal-gun owner's self
    /// teleport (`PortalHelper.cs:188-192`); the host forces the id to the
    /// sender and relays (`MessageBuffer.cs:3541-3555`; W4-g50).
    PlayerPortalTeleport(PlayerPortalTeleport),
    /// `99 MinionRestTargetUpdate` - this client's minion rest point moved
    /// (`Main.cs:17678-17681`; `MessageBuffer.cs:3570-3583` relays; W4-g50).
    MinionRestTarget(MinionRestTarget),
    /// `115 MinionAttackTargetUpdate` - this client's minion attack target
    /// changed (`Main.cs:17682-17685`; `MessageBuffer.cs:3584-3597` relays;
    /// W4-g50).
    MinionAttackTarget(MinionAttackTarget),
    /// `102 NebulaLevelupRequest` - this client picked up a nebula booster
    /// (`Player.cs:35052`; `MessageBuffer.cs:3654-3660` relays; W4-g50).
    NebulaLevelupRequest(NebulaLevelupRequest),
    /// `105 GemLockToggle` - this client toggled a gem lock
    /// (`Player.cs:32547`/`:32559`; `MessageBuffer.cs:3736-3741` runs
    /// `WorldGen.ToggleGemLock`; W4-g50).
    GemLockToggle(GemLockToggle),
    /// `111 ToggleParty` — this client asked to toggle the manual party
    /// (`BirthdayParty.cs:154`); the host flips it (`MessageBuffer.cs:3809-
    /// 3814`; W4-g50). No body.
    ToggleParty,
    /// `113 CrystalInvasionStart` — this client placed the eternia crystal
    /// (`DD2Event.cs:574-577`); the host summons on the sender
    /// (`MessageBuffer.cs:3841-3854`; W4-g50).
    CrystalInvasionStart(CrystalInvasionStart),
    /// `117 PlayerHurtV2` — this client was hurt (`Player.Hurt`'s net send);
    /// the host applies + relays (`MessageBuffer.cs:3867-3886`; W4-g50).
    PlayerHurtV2(PlayerHurtV2),
    /// `118 PlayerDeathV2` — this client died (`Player.KillMe`'s net send);
    /// the host forces the id, applies + relays (`MessageBuffer.cs:3887-
    /// 3904`; W4-g50).
    PlayerDeathV2(PlayerDeathV2),
    /// `120 Emoji` — this client sent an emote (`MakeLocalPlayerEmote`'s
    /// `netMode != 0` arm, `EmoteBubble.cs:289`); the host validates + bubbles
    /// it (`MessageBuffer.cs:3905-3919`; W4-g50).
    Emoji(Emoji),
    /// `85 QuickStackChests` — this client's quick-stack ask
    /// (`QuickStacking.SendQuickStackToNearbyChests`, `QuickStacking.cs:287-297`);
    /// the host runs the transfer (`MessageBuffer.cs:3261-3266`; W4-g50).
    QuickStackRequest(QuickStackRequest),
    /// `94 DevCommands` — a debug-command sync / quickload probe
    /// (`DebugOptions.SyncToJoiningPlayer`, `DebugOptions.cs:64-72`);
    /// the reader gates on `enableDebugCommands` (`MessageBuffer.cs:3463-3518`;
    /// W4-g50).
    DevCommands(DevCommands),
    /// `130 FishOutNPC` - a client fished an NPC out at the tile point
    /// (`Player.cs:52423-52426`; the host spawns, `MessageBuffer.cs:4086-4121`).
    FishOutNpc(FishOutNpc),
    /// `134 UpdatePlayerLuckFactors` - a client's luck factors
    /// (`Player.UpdateLuck`'s `luckNeedsSync` send, `Player.cs:29048`; the
    /// host applies + relays, `MessageBuffer.cs:4164-4193`).
    UpdatePlayerLuckFactors(UpdatePlayerLuckFactors),
    /// `137 RequestNPCBuffRemoval` - a client asks the host to drop an NPC
    /// buff (`NPC.cs:89995-89998`; `MessageBuffer.cs:4219-4229`).
    NpcBuffRemovalRequest(NpcBuffRemovalRequest),
    /// `140 SetMiscEventValues` - kinds 1/2 are the CLIENT's town-slime
    /// transform requests (`NPC.TransformCopperSlime`/`TransformElderSlime`'s
    /// `netMode == 1` arms, `NPC.cs:19164`/`:19140`); the host applies
    /// (`MessageBuffer.cs:4250-4260`). Kind 0 is host-sent only (the credits
    /// roll, `S2c::MiscEventValue`).
    MiscEventValue(MiscEventValue),
    /// `141 LucyTheAxeMessage` - a talking-axe popup
    /// (`LucyAxeMessage.cs:133`; the host relays, `:4274`).
    LucyPopup(LucyPopup),
    /// `143 DD2EventSkipWaitTime` - a client asked to skip the DD2 wait,
    /// no body (`DD2Event.cs:1790`, `MessageBuffer.cs:4298-4303`).
    CrystalInvasionSkip,
    /// `150 SpectatePlayer` - a client (re)targets spectating
    /// (`Player.SetOrRequestSpectating`, `Player.cs:17252`; the host arm
    /// `MessageBuffer.cs:4361-4384`).
    SpectatePlayer(SpectatePlayer),
    /// `138 ClientSyncedInventory` — the client's "inventory sweep done"
    /// notice, bodyless (`Main.cs:17700`); NO reader case — the default
    /// arm drops it (`MessageBuffer.cs:4477-4482`). C2s-only. W4-g51 f21.
    ClientSyncedInventory,
    /// `152 ItemUseSound` — a client used an item with a sound
    /// (`Player.cs:51692`; host id-force + relay
    /// `MessageBuffer.cs:4386-4407`). Both directions. W4-g51 f21.
    ItemUseSound { player_id: u8 },
    /// `161 HostToken` — the client's host-token claim, pushed in the
    /// join bundle when set (`MessageBuffer.cs:275`; host compare
    /// `:4471-4476`). Gate-whitelisted pre-spawn (`:165`). C2s-only.
    /// W4-g51 f21.
    HostToken(String),
    /// `121 TEDisplayDollDataSync` — a client changed a display-doll slot
    /// or pose (`MessageBuffer.cs:3920-3941`). Both directions. W4-g51.
    DisplayDollSync(DisplayDollSync),
    /// `122 RequestTileEntityInteraction` — a client opened/closed a
    /// tile-entity interaction (`MessageBuffer.cs:3942-3977`). Both
    /// directions. W4-g51.
    TileEntityInteraction(TileEntityInteraction),
    /// `123 WeaponsRackTryPlacing` — a client placed an item on a weapon
    /// rack (`MessageBuffer.cs:3978-3994`, host-only). C2s-only. W4-g51.
    WeaponsRackTryPlacing(TryPlacingItem),
    /// `124 TEHatRackItemSync` — a client changed a hat-rack slot/dye
    /// (`MessageBuffer.cs:3995-4022`). Both directions. W4-g51.
    HatRackItemSync(HatRackItemSync),
    /// `133 FoodPlatterTryPlacing` — a client placed food on a platter
    /// (`MessageBuffer.cs:4147-4163`, host-only). C2s-only. W4-g51.
    FoodPlatterTryPlacing(TryPlacingItem),
    /// `144 RequestQuestEffect` — the client's Stardew-dryad ask,
    /// bodyless (`MessageBuffer.cs:4304-4309`, host-only). C2s-only.
    /// W4-g51.
    RequestQuestEffect,
    /// `149 DeadCellsDisplayJarTryPlacing` — a client placed an item in
    /// a Dead Cells display jar (`MessageBuffer.cs:4344-4360`, host-only).
    /// C2s-only. W4-g51.
    DeadCellsJarTryPlacing(TryPlacingItem),
    /// `156 TELeashedEntityAnchorPlaceItem` — a client placed an item on
    /// a kite/critter anchor (`MessageBuffer.cs:4439-4449`, host-only).
    /// C2s-only. W4-g51.
    LeashedAnchorPlaceItem(LeashedAnchorPlaceItem),
    /// `39 ReleaseItemOwnership` — a client releases its item reservation
    /// (`WorldItem.FindOwner`'s `netMode == 1` arm, `WorldItem.cs:256`;
    /// reader `MessageBuffer.cs:2232-2255`).
    ReleaseItemOwnership(ReleaseItemOwnership),
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
    /// `56 UniqueTownNPCInfoSyncRequest` - the host's answer with the town
    /// NPC's given name + variation (`MessageBuffer.cs:2617-2623`). W4-g49.
    UniqueTownNpcInfo(UniqueTownNpcInfo),
    /// `63 SyncTilePaintOrCoating` — the host's paint relay. W4-g49.
    TilePaint(TilePaint),
    /// `64 SyncWallPaintOrCoating` — the host's paint relay. W4-g49.
    WallPaint(TilePaint),
    /// `65 TeleportEntity` — the host's player/NPC teleport broadcast. W4-g49.
    TeleportEntity(TeleportEntity),
    /// `66 Unknown66` - a peer's Moon Leech heal, relayed by the host. W4-g49.
    HealPlayer(HealPlayer),
    /// `77 SyncTilePicking` - a temporary tile animation for one client
    /// (`Animation.NewTemporaryAnimation`). W4-g49.
    TemporaryAnimation(TemporaryAnimation),
    /// `78 ReportInvasionProgress` - the invasion/moon/DD2 progress bar
    /// (`Main.ReportInvasionProgress`). W4-g49.
    InvasionProgress(InvasionProgress),
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
    /// `57 Unknown57` — the world-evil bytes, join push
    /// (`MessageBuffer.cs:867`) + `WorldGen.cs:71173`; client-only reader
    /// (`MessageBuffer.cs:2631-2638`). S2c-only. W4-g49 (from W4-f21).
    WorldEvilBytes(WorldEvilBytes),
    /// `58 InstrumentSound` — a peer's instrument, host relay
    /// (`MessageBuffer.cs:2649`). W4-g49 (from W4-f21).
    InstrumentSound(InstrumentSound),
    /// `76 QuestsCountSync` — a peer's quest/golf counts, host relay
    /// (`MessageBuffer.cs:3178`).
    QuestsCountSync(QuestsCountSync),
    /// `79 PlaceObject` — the host's placement relay
    /// (`SendObjectPlacement`'s `netMode == 2` half,
    /// `NetMessage.cs:2577-2581` + `MessageBuffer.cs:3212`).
    PlaceObject(PlaceObject),
    /// `44 Unknown44` — deprecated (`MessageID.cs:99-100`), no body; the
    /// reader ignore group (`MessageBuffer.cs:4483-4490`).
    Unknown44,
    /// `67 Unknown67` — no body; the same reader ignore group
    /// (`MessageBuffer.cs:4483-4490`).
    Unknown67,
    /// `48 LiquidUpdate` — one tile's liquid, host fan-out
    /// (`NetMessage.sendWater`'s per-section loop, `NetMessage.cs:2785-2796`;
    /// the drained-tile re-push `MessageBuffer.cs:2441`).
    LiquidUpdate(LiquidUpdate),
    /// `52 LockAndUnlock` — the host's unlock/lock relay
    /// (`MessageBuffer.cs:2541`/`:2550`/`:2559`); the reader applies the
    /// same `Chest.Unlock`/`WorldGen.UnlockDoor`/`Chest.Lock` on the client.
    LockAndUnlock(LockAndUnlock),
    /// `119 CombatTextString` - a peer's combat text, the 81 string twin
    /// (`NetMessage.SendCombatText`; W4-g50).
    CombatTextString(CombatTextString),
    /// `84 PlayerStealth` - a peer's stealth value, relayed by the host
    /// (`MessageBuffer.cs:3257`; W4-g50).
    PlayerStealth(PlayerStealth),
    /// `92 SyncExtraValue` - a town NPC's accumulated coin credit, relayed
    /// as the TOTAL (`MessageBuffer.cs:3458`; W4-g50).
    NpcExtraValue(NpcExtraValue),
    /// `86 TileEntitySharing` - one entity's create/update/removal
    /// (`TileEntity.Write(networkSend: true)`, `NetMessage.cs:1216-1225`;
    /// W4-g50).
    TileEntitySharing(TileEntitySharing),
    /// `88 ItemTweaker` - a world item's inner-stat tweak
    /// (`NetMessage.cs:1232-1296`; the port's drops carry no mutable inner
    /// stats, so the client apply is parked — the codec is byte-verbatim;
    /// W4-g50).
    ItemTweaker(ItemTweaker),
    /// `90 SpawnInstancedItem` - the 21 body, instanced + reserved for the
    /// receiving client (`WorldItem.MakeInstanced`, `WorldItem.cs:321-336`;
    /// `MessageBuffer.cs:1489-1493`; W4-g50).
    SpawnInstancedItem(Box<ItemState>),
    /// `91 SyncEmoteBubble` — a bubble's create/update/removal, host-only
    /// (`EmoteBubble.NewBubble`/`NewBubbleNPC`'s `netMode == 2` sends,
    /// `EmoteBubble.cs:128`, `:147`; the reader is `netMode == 1`-gated,
    /// `MessageBuffer.cs:3396`).
    EmoteBubble(EmoteBubble),
    /// `96 TeleportPlayerThroughPortal` — a peer's portal teleport, relayed
    /// by the host (`MessageBuffer.cs:3554`; W4-g50).
    PlayerPortalTeleport(PlayerPortalTeleport),
    /// `99 MinionRestTargetUpdate` - a peer's minion rest point, relayed
    /// (`MessageBuffer.cs:3580`; W4-g50).
    MinionRestTarget(MinionRestTarget),
    /// `115 MinionAttackTargetUpdate` - a peer's minion attack target,
    /// relayed (`MessageBuffer.cs:3594`; W4-g50).
    MinionAttackTarget(MinionAttackTarget),
    /// `100 TeleportNPCThroughPortal` - an NPC pulled through a portal
    /// (`PortalHelper.cs:195-204`; `MessageBuffer.cs:3598-3610`; W4-g50).
    NpcPortalTeleport(NpcPortalTeleport),
    /// `101 UpdateTowerShieldStrengths` - the four Lunar tower shields
    /// (`NetMessage.cs:1371-1376`; `MessageBuffer.cs:3611-3651`; W4-g50).
    TowerShields(TowerShields),
    /// `102 NebulaLevelupRequest` - a peer's nebula booster pickup, relayed
    /// (`MessageBuffer.cs:3659`; W4-g50).
    NebulaLevelupRequest(NebulaLevelupRequest),
    /// `103 MoonlordHorror` - the Moon Lord countdown statics
    /// (`NetMessage.cs:1383-1386`; `MessageBuffer.cs:3708-3714`; W4-g50).
    MoonlordCountdown(MoonlordCountdown),
    /// `104 ShopOverride` - one open-shop slot's rebuilt item
    /// (`NetMessage.cs:1387-1394`; `MessageBuffer.cs:3715-3735`; W4-g50).
    ShopOverride(ShopOverride),
    /// `106 PoofOfSmoke` - a smoke poof at a half-float position
    /// (`NetMessage.cs:1400-1402`; the client replays `Utils.PoofOfSmoke`,
    /// `MessageBuffer.cs:3742-3751`; W4-g50).
    PoofOfSmoke(PoofOfSmoke),
    /// `107 SmartTextMessage` - a multiline chat line + the sound-12 cue
    /// (`NetMessage.cs:769-774`; `MessageBuffer.cs:3752-3757`; W4-g50).
    SmartText(SmartText),
    /// `108 WiredCannonShot` - a wire-fired cannon shot redirected to its
    /// owner (`WorldGen.cs:50689-50692`; `MessageBuffer.cs:3758-3773`;
    /// W4-g50).
    WiredCannonShot(WiredCannonShot),
    /// `112 SpecialFX` - a tree-grow / fairy effect burst
    /// (`NetMessage.cs:1424-1431`; `MessageBuffer.cs:3815-3840`; W4-g50).
    SpecialFx(SpecialFx),
    /// `114 CrystalInvasionWipeAllTheThingsss` - the DD2 entity wipe
    /// (`DD2Event.cs:636`; `MessageBuffer.cs:3855-3860`; W4-g50). No body.
    CrystalInvasionWipe,
    /// `116 CrystalInvasionSendWaitTime` - the wave-break countdown
    /// (`DD2Event.cs:947-950`; `MessageBuffer.cs:3861-3866`; W4-g50).
    CrystalInvasionWait(CrystalInvasionWait),
    /// `117 PlayerHurtV2` - a peer's hurt, relayed by the host
    /// (`MessageBuffer.cs:3882`; W4-g50).
    PlayerHurtV2(PlayerHurtV2),
    /// `118 PlayerDeathV2` - a peer's death, relayed by the host
    /// (`MessageBuffer.cs:3901`; W4-g50).
    PlayerDeathV2(PlayerDeathV2),
    /// `83 Unused83` — deprecated (`MessageID.cs:179-180`), no body; the
    /// reader's ignore group (`MessageBuffer.cs:4483-4490`; W4-g50).
    Unused83,
    /// `93 SocialHandshake` — deprecated (`MessageID.cs:200`), no body; the
    /// reader's ignore group (`MessageBuffer.cs:4483-4490`; W4-g50).
    Unused93,
    /// `85 QuickStackChests` — the host's blocked-chest list answer
    /// (`QuickStacking.WriteBlockedChestList`, `QuickStacking.cs:323-330`;
    /// `MessageBuffer.cs:3267-3270`; W4-g50).
    QuickStackBlocked(QuickStackBlocked),
    /// `94 DevCommands` — a debug-command sync / quickload response
    /// (`DebugOptions.SyncToJoiningPlayer`, `DebugOptions.cs:64-72`;
    /// `MessageBuffer.cs:3463-3518`; W4-g50).
    DevCommands(DevCommands),
    /// `125 SyncTilePicking` — another player's pickaxe hit
    /// (`MessageBuffer.cs:4033-4036` applies it to the local player).
    TilePicking(TilePicking),
    /// `127 RemoveRevengeMarker` - a coin-loss revenge marker is gone
    /// (`CoinLossRevengeSystem.cs:451`, `MessageBuffer.cs:4049-4057`).
    RemoveRevengeMarker(RemoveRevengeMarker),
    /// `131 TamperWithNPC` - the host tames an NPC's immunity (the 485
    /// statue-break 582 spawns, `WorldGen.cs:48373-48376`;
    /// `MessageBuffer.cs:4122-4133`).
    TamperWithNpc(TamperWithNpc),
    /// `134 UpdatePlayerLuckFactors` - a peer's luck factors, relayed by the
    /// host (`MessageBuffer.cs:4191`); the client applies the eight fields +
    /// `RecalculateLuck` (`:4180-4188`).
    UpdatePlayerLuckFactors(UpdatePlayerLuckFactors),
    /// `136 SyncCavernMonsterType` - the cavern spawn grid, the join push
    /// (`MessageBuffer.cs:870`, read `:4204-4218`).
    CavernMonsterTypes(CavernMonsterTypes),
    /// `141 LucyTheAxeMessage` - the host's relay of a talking-axe popup
    /// (`MessageBuffer.cs:4274`; the client CreateFromNet, `:4278`).
    LucyPopup(LucyPopup),
    /// `150 SpectatePlayer` - a peer's spectate target, relayed by the host
    /// on a stop (`MessageBuffer.cs:4378`) or sent from the host's own
    /// `SetOrRequestSpectating` (`Player.cs:17271`); the client applies a
    /// peer's value or its own stop (`:4380-4383`).
    SpectatePlayer(SpectatePlayer),
    /// `153 NPCDebuffDamage` - an NPC's debuff tick damage, host-sent and
    /// relayed (`MessageBuffer.cs:4408-4417`); the client applies
    /// `GetHurtByDebuff` (`NPC.cs:91864-91888`).
    NpcDebuffDamage(NpcDebuffDamage),
    /// `158 ExtraSpawnSectionLoaded` — the team-swap respawn: the extra
    /// spawn section is in, `Spawn(TeamSwap)` (`MessageBuffer.cs:4450-4456`,
    /// sent by the `157` tail `:2361`).
    ExtraSpawnLoaded {
        player_id: u8,
    },
    /// `129 FinishedConnectingToServer` — the join arm's "you're in"
    /// notice, bodyless (`MessageBuffer.cs:937`; the client's team
    /// re-push + UI-scale + preparation-state tail `:4075-4085`).
    /// S2c-only. W4-g51 f21.
    FinishedConnecting,
    /// `139 SetCountsAsHostForGameplay` — the join arm's host-authority
    /// flag push (`MessageBuffer.cs:934`; client-only store
    /// `:4230-4237`). S2c-only. W4-g51 f21.
    CountsAsHost(CountsAsHost),
    /// `146 ShimmerActions` — the shimmer FX / coin-luck push
    /// (`WorldItem.cs:1665`/`:1716`; client-only reader
    /// `MessageBuffer.cs:4310-4327`). S2c-only. W4-g51 f21.
    ShimmerAction(ShimmerAction),
    /// `152 ItemUseSound` — a peer's item-use sound, host relay
    /// (`MessageBuffer.cs:4395`). Both directions. W4-g51 f21.
    ItemUseSound { player_id: u8 },
    /// `160 ItemPosition` — a drop's position snap, the section sweep
    /// (`WorldItem.cs:1864`; client-only reader
    /// `MessageBuffer.cs:4463-4470`). S2c-only. W4-g51 f21.
    ItemPosition(ItemPosition),
    /// `121 TEDisplayDollDataSync` — the host's doll relay
    /// (`MessageBuffer.cs:3938`). Both directions. W4-g51.
    DisplayDollSync(DisplayDollSync),
    /// `122 RequestTileEntityInteraction` — the host's anchor relay
    /// (`MessageBuffer.cs:3955`/`:3961`). Both directions. W4-g51.
    TileEntityInteraction(TileEntityInteraction),
    /// `124 TEHatRackItemSync` — the host's hat-rack relay
    /// (`MessageBuffer.cs:4019`). Both directions. W4-g51.
    HatRackItemSync(HatRackItemSync),
    /// `18 SetTime` - the host's clock (`NetMessage.cs:531-536`; the client
    /// applies it at `netMode == 1`, `MessageBuffer.cs:1290-1298`). W4-c27.
    SetTime(SetTime),
    /// `51 MiscDataSync` - the host's misc fan-out (kinds 2/4;
    /// `MessageBuffer.cs:2498`, `:2512`). W4-c27.
    MiscDataSync(MiscDataSync),
    /// `55 AddPlayerBuffPvP` - a PvP debuff aimed at us, sent by the host to
    /// the target alone (`MessageBuffer.cs:2603`). W4-c27.
    PlayerBuffPvp(PlayerBuffPvp),
    /// `62 SyncDodge` - a peer's dodge, relayed by the host
    /// (`MessageBuffer.cs:2905`). W4-c27.
    SyncDodge(SyncDodge),
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
    /// `162 DamageNPCAck` - the host's ack of a `28` strike, no body
    /// (`MessageBuffer.cs:1768-1771` sends, `:1819-1824` pops
    /// `NPC.pendingDamages[0]`).
    NpcStrikeAck,
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
    /// `25 Unused25` — deprecated (`MessageID.cs:59-60`), no body; the
    /// reader's ignore group accepts and drops it
    /// (`MessageBuffer.cs:4483-4490`, shared with 15/26/44/67/83/93).
    Unused25,
    /// `26 Unused26` — deprecated (`MessageID.cs:62-63`), no body; the same
    /// reader ignore group (`MessageBuffer.cs:4483-4490`).
    Unused26,
    /// `39 ReleaseItemOwnership` — the host's release notice to one client
    /// (`WorldItem.cs:264`'s `SendData(39, num, …)`; the reader's
    /// `netMode == 1` arm re-runs `FindOwner(forceAssignToServer: true)`,
    /// `MessageBuffer.cs:2237-2243`).
    ReleaseItemOwnership(ReleaseItemOwnership),
    /// `93 SocialHandshake` — no body; the reader ignore group
    /// (`MessageBuffer.cs:4483-4490`), accept-gate-whitelisted (`:165`).
    SocialHandshake,
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
            Self::UniqueTownNpcInfo(_) => id::UNIQUE_TOWN_NPC_INFO,
            Self::SpawnBossEvent(_) => id::SPAWN_BOSS_EVENT,
            Self::TilePaint(_) => id::TILE_PAINT,
            Self::WallPaint(_) => id::WALL_PAINT,
            Self::TeleportEntity(_) => id::TELEPORT_ENTITY,
            Self::HealPlayer(_) => id::HEAL_PLAYER,
            Self::ClientUuid(_) => id::CLIENT_UUID,
            Self::RequestTeleport(_) => id::REQUEST_TELEPORT,
            Self::AnglerQuestFinished => id::ANGLER_QUEST_FINISHED,
            Self::PlayerChest(_) => id::PLAYER_CHEST,
            Self::PlayerBuffs(_) => id::PLAYER_BUFFS,
            Self::TogglePvp(_) => id::TOGGLE_PVP,
            Self::PlayerMana(_) => id::PLAYER_MANA,
            Self::TeamChange(_) => id::TEAM_CHANGE,
            Self::ProjectileTrackers(_) => id::PROJECTILE_TRACKERS,
            Self::SyncLoadout(_) => id::SYNC_LOADOUT,
            Self::TileEdit(_) => id::TILE_EDIT,
            Self::DoorToggle(_) => id::DOOR_TOGGLE,
            Self::TilePicking(_) => id::TILE_PICKING,
            Self::TeamChangeUi(_) => id::TEAM_CHANGE_UI,
            Self::SectionRequest { .. } => id::SECTION_REQUEST,

            Self::PlayerHeal(_) => id::PLAYER_HEAL,
            Self::TalkNpc(_) => id::TALK_NPC,
            Self::PlayerZone(_) => id::PLAYER_ZONE,
            Self::ItemAnim(_) => id::ITEM_ANIM,
            Self::ManaEffect(_) => id::MANA_EFFECT,
            Self::InstrumentSound(_) => id::INSTRUMENT_SOUND,
            Self::QuestsCountSync(_) => id::QUESTS_COUNT_SYNC,
            Self::PlaceObject(_) => id::PLACE_OBJECT,
            Self::LiquidUpdate(_) => id::LIQUID_UPDATE,
            Self::LockAndUnlock(_) => id::LOCK_AND_UNLOCK,
            Self::PlayerStealth(_) => id::PLAYER_STEALTH,
            Self::NpcExtraValue(_) => id::NPC_EXTRA_VALUE,
            Self::TileEntityPlacement(_) => id::TILE_ENTITY_PLACEMENT,
            Self::ItemFrameTryPlacing(_) => id::ITEM_FRAME_TRY_PLACING,
            Self::PlayerPortalTeleport(_) => id::TELEPORT_PLAYER_PORTAL,
            Self::MinionRestTarget(_) => id::MINION_REST_TARGET,
            Self::MinionAttackTarget(_) => id::MINION_ATTACK_TARGET,
            Self::NebulaLevelupRequest(_) => id::NEBULA_LEVELUP_REQUEST,
            Self::GemLockToggle(_) => id::GEM_LOCK_TOGGLE,
            Self::ToggleParty => id::TOGGLE_PARTY,
            Self::CrystalInvasionStart(_) => id::CRYSTAL_INVASION_START,
            Self::PlayerHurtV2(_) => id::PLAYER_HURT_V2,
            Self::PlayerDeathV2(_) => id::PLAYER_DEATH_V2,
            Self::Emoji(_) => id::EMOJI,
            Self::QuickStackRequest(_) => id::QUICK_STACK_CHESTS,
            Self::DevCommands(_) => id::DEV_COMMANDS,
            Self::FishOutNpc(_) => id::FISH_OUT_NPC,
            Self::UpdatePlayerLuckFactors(_) => id::UPDATE_PLAYER_LUCK_FACTORS,
            Self::NpcBuffRemovalRequest(_) => id::NPC_BUFF_REMOVAL_REQUEST,
            Self::MiscEventValue(_) => id::MISC_EVENT_VALUES,
            Self::LucyPopup(_) => id::LUCY_POPUP,
            Self::CrystalInvasionSkip => id::CRYSTAL_INVASION_SKIP,
            Self::SpectatePlayer(_) => id::SPECTATE_PLAYER,
            Self::ClientSyncedInventory => id::CLIENT_SYNCED_INVENTORY,
            Self::ItemUseSound { .. } => id::ITEM_USE_SOUND,
            Self::HostToken(_) => id::HOST_TOKEN,
            Self::DisplayDollSync(_) => id::TE_DISPLAY_DOLL_SYNC,
            Self::TileEntityInteraction(_) => id::TE_INTERACTION_REQUEST,
            Self::WeaponsRackTryPlacing(_) => id::WEAPONS_RACK_TRY_PLACING,
            Self::HatRackItemSync(_) => id::TE_HAT_RACK_ITEM_SYNC,
            Self::FoodPlatterTryPlacing(_) => id::FOOD_PLATTER_TRY_PLACING,
            Self::RequestQuestEffect => id::REQUEST_QUEST_EFFECT,
            Self::DeadCellsJarTryPlacing(_) => id::DEAD_CELLS_JAR_TRY_PLACING,
            Self::LeashedAnchorPlaceItem(_) => id::LEASHED_ANCHOR_PLACE_ITEM,
            Self::ReleaseItemOwnership(_) => id::RELEASE_ITEM_OWNERSHIP,
            Self::MiscDataSync(_) => id::MISC_DATA_SYNC,
            Self::PlayerBuffPvp(_) => id::ADD_PLAYER_BUFF_PVP,
            Self::SyncDodge(_) => id::SYNC_DODGE,
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
                    b.set(1, item.blocked);
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
            // `NetMessage.cs:1046-1053`: (short)npc; the name + variation
            // ride only the host's answer (the client sends the bare slot).
            Self::UniqueTownNpcInfo(t) => {
                let w = w.i16(t.npc);
                match &t.info {
                    Some((name, variation)) => w.str(name).i32(*variation),
                    None => w,
                }
            }
            // `NetMessage.cs:1073-1076`: (short)player + (short)event.
            Self::SpawnBossEvent(e) => w.i16(e.player).i16(e.event),
            // `NetMessage.cs:1081-1087`: short x + short y + byte paint +
            // byte coat.
            Self::TilePaint(p) | Self::WallPaint(p) => {
                w.i16(p.x).i16(p.y).u8(p.paint).u8(p.coat)
            }
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
            // `NetMessage.cs:1110-1113`: (byte)player + (short)heal.
            Self::HealPlayer(h) => w.u8(h.player_id).i16(h.heal),
            // `NetMessage.cs:1114-1116`: the UUID string.
            Self::ClientUuid(u) => w.str(&u.uuid),
            // `NetMessage.cs:1142-1144`: (byte)kind.
            Self::RequestTeleport(q) => w.u8(q.kind),
            // No writer arm — bodyless (`MessageBuffer.cs:3154-3163`).
            Self::AnglerQuestFinished => w,
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
            // `NetMessage.cs:987-995`: two (short)s + liquid + liquidType().
            Self::LiquidUpdate(l) => w.i16(l.x).i16(l.y).u8(l.liquid).u8(l.liquid_type),
            // `NetMessage.cs:1014-1018`: (byte)action + two (short)s; the
            // player id is NOT written.
            Self::LockAndUnlock(l) => w.u8(l.action).i16(l.x).i16(l.y),
            // `NetMessage.cs:1521-1527`: byte owner + ushort x + ushort y +
            // ushort hits + ushort type (the writer's field order).
            Self::GolfBallInCup(g) => w.u8(g.owner).u16(g.x).u16(g.y).u16(g.hits).u16(g.ball_type),
            // `NetMessage.cs:1528-1532`: (ushort)tile x, (ushort)tile y,
            // (short)npc type.
            Self::FishOutNpc(f) => w.u16(f.tile_x).u16(f.tile_y).i16(f.npc_type),
            // `NetMessage.cs:1555-1567`: the eight luck fields in writer order.
            Self::UpdatePlayerLuckFactors(l) => l.write_to(w),
            // `NetMessage.cs:1583-1586`: (short)npc, (ushort)buff type.
            Self::NpcBuffRemovalRequest(q) => w.i16(q.npc).u16(q.buff_type),
            // `NetMessage.cs:1591-1594`: (byte)kind + (int)number.
            Self::MiscEventValue(v) => w.u8(v.kind).i32(v.value),
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
            // `NetMessage.cs:1639-1642`: (byte)number, (short)number2.
            Self::SpectatePlayer(s) => w.u8(s.player_id).i16(s.target),
            // No writer arm — bodyless (`Main.cs:17700` SendData(138)).
            Self::ClientSyncedInventory => w,
            // `NetMessage.cs:1643-1645`: (byte)player only.
            Self::ItemUseSound { player_id } => w.u8(*player_id),
            // `NetMessage.cs:1673-1675`: a plain string.
            Self::HostToken(t) => w.str(t),
            // `NetMessage.cs:1227-1231`: short x, short y, byte type.
            Self::TileEntityPlacement(p) => w.i16(p.x).i16(p.y).u8(p.kind),
            // `NetMessage.cs:1297-1306` — the TryPlacing body (the 123/133/
            // 149 twins are byte-identical, f21's pin).
            Self::ItemFrameTryPlacing(p) => {
                w.i16(p.x).i16(p.y).i16(p.kind).u8(p.prefix).i16(p.stack)
            }
            // `NetMessage.cs:1458-1474`: (byte)player + int id +
            // (byte)itemIndex + (byte)command + WriteData/WriteDummySync.
            Self::DisplayDollSync(d) => write_doll_sync(d, w),
            // `NetMessage.cs:1475-1478`: int entityId + (byte)player.
            Self::TileEntityInteraction(t) => w.i32(t.entity_id).u8(t.player_id),
            // `NetMessage.cs:1479-1488` — the exact 89 body under id 123.
            Self::WeaponsRackTryPlacing(p) => {
                w.i16(p.x).i16(p.y).i16(p.kind).u8(p.prefix).i16(p.stack)
            }
            // `NetMessage.cs:1489-1508`: (byte)player + int id + the
            // already-folded wire slot + WriteItem (or the air fallback,
            // which is the same 5 bytes as an air TeItem slot).
            Self::HatRackItemSync(h) => h
                .item
                .write_slot(w.u8(h.player_id).i32(h.entity_id).u8(h.wire_slot)),
            // `NetMessage.cs:1545-1554` — the exact 89 body under id 133.
            Self::FoodPlatterTryPlacing(p) => {
                w.i16(p.x).i16(p.y).i16(p.kind).u8(p.prefix).i16(p.stack)
            }
            // No writer case for 144 — `SendData(144)` writes a bare frame.
            Self::RequestQuestEffect => w,
            // `NetMessage.cs:1629-1638` — the exact 89 body under id 149.
            Self::DeadCellsJarTryPlacing(p) => {
                w.i16(p.x).i16(p.y).i16(p.kind).u8(p.prefix).i16(p.stack)
            }
            // `NetMessage.cs:1654-1658`: three (short)s.
            Self::LeashedAnchorPlaceItem(p) => w.i16(p.x).i16(p.y).i16(p.item_type),
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
            // `NetMessage.cs:1321-1326` — short npc, int value, two floats.
            Self::NpcExtraValue(e) => w.i16(e.npc).i32(e.value).f32(e.x).f32(e.y),
            // `NetMessage.cs:1337-1346` — byte player, short color, dest
            // floats, live-velocity floats.
            Self::PlayerPortalTeleport(p) => w
                .u8(p.player)
                .i16(p.color_index)
                .f32(p.x)
                .f32(p.y)
                .f32(p.velocity_x)
                .f32(p.velocity_y),
            // `NetMessage.cs:1353-1356`: (byte)number + WriteVector2.
            Self::MinionRestTarget(m) => w.u8(m.player_id).f32(m.x).f32(m.y),
            // `NetMessage.cs:1357-1360`: (byte)number + (short)target.
            Self::MinionAttackTarget(m) => w.u8(m.player_id).i16(m.npc),
            // `NetMessage.cs:1377-1382`: byte picker + ushort buffType + the
            // picker-centre floats.
            Self::NebulaLevelupRequest(n) => {
                w.u8(n.player_id).u16(n.buff_type).f32(n.x).f32(n.y)
            }
            // `NetMessage.cs:1395-1399`: short x, short y, bool on.
            Self::GemLockToggle(g) => w.i16(g.x).i16(g.y).bool(g.on),
                .i16(m.start_x)
                .i16(m.start_y)
                .i16(m.end_x)
                .i16(m.end_y)
                .u8(m.tool_mode),
            // `BirthdayParty.cs:154` — no body.
            Self::ToggleParty => w,
            // `NetMessage.cs:1432-1435`: (short)x + (short)y.
            Self::CrystalInvasionStart(c) => w.i16(c.x).i16(c.y),
            // `NetMessage.cs:1439-1446` via `SendPlayerHurt` (`:2598-2605`).
            Self::PlayerHurtV2(h) => h.write_to(w),
            // `NetMessage.cs:1447-1453` via `SendPlayerDeath` (`:2607-2613`).
            Self::PlayerDeathV2(d) => d.write_to(w),
            // `NetMessage.cs:1454-1457`: (byte)number, (byte)number2.
            Self::Emoji(e) => w.u8(e.player_id).u8(e.emote_id),
            // `NetMessage.cs:1205-1210` / `QuickStacking.cs:299-306`: int
            // count, the short slot ids, the byte smartStack flag.
            Self::QuickStackRequest(q) => {
                let mut w = w.i32(q.slots.len() as i32);
                for slot in &q.slots {
                    w = w.i16(*slot);
                }
                w.bool(q.smart_stack)
            }
            // `NetMessage.cs:1327-1332`: string command, int number, float
            // number2, float number3.
            Self::DevCommands(d) => w.str(&d.command).i32(d.number).f32(d.number2).f32(d.number3),
            // `NetMessage.cs:948-951`: (short)slot + (number2 != 0f) bool.
            Self::ReleaseItemOwnership(r) => w.i16(r.slot).bool(r.force_assign_to_server),
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
                        blocked: flags.get(1),
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
            // `MessageBuffer.cs:2614`: the host reads ONLY the (short) slot —
            // the request is bodyless past it (`NPC.cs:90564`).
            id::UNIQUE_TOWN_NPC_INFO => Self::UniqueTownNpcInfo(UniqueTownNpcInfo {
                npc: r.i16()?,
                info: None,
            }),
            // `MessageBuffer.cs:2742-2743`: (short)player + (short)event.
            id::SPAWN_BOSS_EVENT => Self::SpawnBossEvent(SpawnBossEvent {
                player: r.i16()?,
                event: r.i16()?,
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
            // `MessageBuffer.cs:2951-2978`: BitsByte + entity + Vector2 +
            // style + gated extraInfo.
            id::TELEPORT_ENTITY => Self::TeleportEntity(read_teleport_entity(r)?),
            // `MessageBuffer.cs:3038-3039`: (byte)player + (short)heal.
            id::HEAL_PLAYER => Self::HealPlayer(HealPlayer {
                player_id: r.u8()?,
                heal: r.i16()?,
            }),
            // `MessageBuffer.cs:3057`: the UUID string.
            id::CLIENT_UUID => Self::ClientUuid(ClientUuid { uuid: r.str()? }),
            // `MessageBuffer.cs:3128`: (byte)kind.
            id::REQUEST_TELEPORT => Self::RequestTeleport(RequestTeleport { kind: r.u8()? }),
            // `MessageBuffer.cs:3154-3163`: no body.
            id::ANGLER_QUEST_FINISHED => Self::AnglerQuestFinished,
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
            // `MessageBuffer.cs:4060-4064`: owner, x, y, hits, type.
            id::GOLF_BALL_IN_CUP => Self::GolfBallInCup(GolfBallInCup {
                owner: r.u8()?,
                x: r.u16()?,
                y: r.u16()?,
                hits: r.u16()?,
                ball_type: r.u16()?,
            }),
            // `MessageBuffer.cs:4092-4094`: (ushort)x, (ushort)y, (short)type.
            id::FISH_OUT_NPC => Self::FishOutNpc(FishOutNpc {
                tile_x: r.u16()?,
                tile_y: r.u16()?,
                npc_type: r.i16()?,
            }),
            // `MessageBuffer.cs:4166-4174`: the eight luck fields.
            id::UPDATE_PLAYER_LUCK_FACTORS => {
                Self::UpdatePlayerLuckFactors(UpdatePlayerLuckFactors::read_from(r)?)
            }
            // `MessageBuffer.cs:4222-4223`: (short)npc, (ushort)buff type.
            id::NPC_BUFF_REMOVAL_REQUEST => Self::NpcBuffRemovalRequest(NpcBuffRemovalRequest {
                npc: r.i16()?,
                buff_type: r.u16()?,
            }),
            // `MessageBuffer.cs:4240-4241`: (byte)kind + (int)value.
            id::MISC_EVENT_VALUES => Self::MiscEventValue(MiscEventValue {
                kind: r.u8()?,
                value: r.i32()?,
            }),
            // `MessageBuffer.cs:4267-4271`: source, variation, velocity
            // floats, then the position ints.
            id::LUCY_POPUP => Self::LucyPopup(LucyPopup {
                source: r.u8()?,
                variation: r.u8()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
                x: r.i32()?,
                y: r.i32()?,
            }),
            // `MessageBuffer.cs:4298-4303`: no body.
            id::CRYSTAL_INVASION_SKIP => Self::CrystalInvasionSkip,
            // `MessageBuffer.cs:4363-4368`: (byte)player, (short)target.
            id::SPECTATE_PLAYER => Self::SpectatePlayer(SpectatePlayer {
                player_id: r.u8()?,
                target: r.i16()?,
            }),
            // No reader case in the C# (the default arm drops it); the port
            // decodes the bare frame so the host arm can no-op explicitly.
            id::CLIENT_SYNCED_INVENTORY => Self::ClientSyncedInventory,
            // `MessageBuffer.cs:4388`: (byte)player.
            id::ITEM_USE_SOUND => Self::ItemUseSound {
                player_id: r.u8()?,
            },
            // `MessageBuffer.cs:4473`: a plain string.
            id::HOST_TOKEN => Self::HostToken(r.str()?),
            // `MessageBuffer.cs:3297-3299`: short x, short y, byte type.
            id::TILE_ENTITY_PLACEMENT => Self::TileEntityPlacement(TileEntityPlacement {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.u8()?,
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
            // `MessageBuffer.cs:4153-4157` — the exact 89 read under 133.
            id::FOOD_PLATTER_TRY_PLACING => Self::FoodPlatterTryPlacing(TryPlacingItem {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.i16()?,
                prefix: r.u8()?,
                stack: r.i16()?,
            }),
            // `MessageBuffer.cs:4304-4309`: the host reads NOTHING.
            id::REQUEST_QUEST_EFFECT => Self::RequestQuestEffect,
            // `MessageBuffer.cs:4350-4354` — the exact 89 read under 149.
            id::DEAD_CELLS_JAR_TRY_PLACING => Self::DeadCellsJarTryPlacing(TryPlacingItem {
                x: r.i16()?,
                y: r.i16()?,
                kind: r.i16()?,
                prefix: r.u8()?,
                stack: r.i16()?,
            }),
            // `MessageBuffer.cs:4442-4443`: two ReadInt16 (the Point16) +
            // ReadInt16 itemType.
            id::LEASHED_ANCHOR_PLACE_ITEM => Self::LeashedAnchorPlaceItem(LeashedAnchorPlaceItem {
                x: r.i16()?,
                y: r.i16()?,
                item_type: r.i16()?,
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
            // `MessageBuffer.cs:3777-3781` — the four shorts + the mode byte.
            id::MASS_WIRE_OPERATION => Self::MassWireOperation(MassWireOperation {
                start_x: r.i16()?,
                start_y: r.i16()?,
                end_x: r.i16()?,
                end_y: r.i16()?,
                tool_mode: r.u8()?,
            }),
            // `MessageBuffer.cs:3444-3447`: (short)npc, (int)value, two
            // ReadSingles (the writer's number3/number4 are floats).
            id::NPC_EXTRA_VALUE => Self::NpcExtraValue(NpcExtraValue {
                npc: r.i16()?,
                value: r.i32()?,
                x: r.f32()?,
                y: r.f32()?,
            }),
            // `MessageBuffer.cs:3540-3548` — byte player, short color, dest
            // Vector2, live-velocity Vector2.
            id::TELEPORT_PLAYER_PORTAL => Self::PlayerPortalTeleport(PlayerPortalTeleport {
                player: r.u8()?,
                color_index: r.i16()?,
                x: r.f32()?,
                y: r.f32()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
            }),
            // `MessageBuffer.cs:3572-3577` — byte player + ReadVector2.
            id::MINION_REST_TARGET => Self::MinionRestTarget(MinionRestTarget {
                player_id: r.u8()?,
                x: r.f32()?,
                y: r.f32()?,
            }),
            // `MessageBuffer.cs:3586-3591` — byte player + (short) target.
            id::MINION_ATTACK_TARGET => Self::MinionAttackTarget(MinionAttackTarget {
                player_id: r.u8()?,
                npc: r.i16()?,
            }),
            // `MessageBuffer.cs:3654-3656` — byte picker, ushort buffType,
            // the picker-centre Vector2.
            id::NEBULA_LEVELUP_REQUEST => Self::NebulaLevelupRequest(NebulaLevelupRequest {
                player_id: r.u8()?,
                buff_type: r.u16()?,
                x: r.f32()?,
                y: r.f32()?,
            }),
            // `MessageBuffer.cs:3739` — short x, short y, bool on.
            id::GEM_LOCK_TOGGLE => Self::GemLockToggle(GemLockToggle {
                x: r.i16()?,
                y: r.i16()?,
                on: r.bool()?,
            }),
            // `MessageBuffer.cs:3809-3814` — no body.
            id::TOGGLE_PARTY => Self::ToggleParty,
            // `MessageBuffer.cs:3843-3844` — (short)x + (short)y.
            id::CRYSTAL_INVASION_START => Self::CrystalInvasionStart(CrystalInvasionStart {
                x: r.i16()?,
                y: r.i16()?,
            }),
            // `MessageBuffer.cs:3869-3878`: the direction byte minus 1, the
            // crit/pvp BitsByte, the sbyte cooldown counter.
            id::PLAYER_HURT_V2 => Self::PlayerHurtV2(PlayerHurtV2::read_from(r)?),
            // `MessageBuffer.cs:3889-3897`: the direction byte minus 1, the
            // pvp-only BitsByte — no cooldown sbyte follows.
            id::PLAYER_DEATH_V2 => Self::PlayerDeathV2(PlayerDeathV2::read_from(r)?),
            // `MessageBuffer.cs:3907-3912`: (byte)player, (byte)emoteId; the
            // `0 <= id < EmoteID.Count` gate is the host arm's, not the codec's.
            id::EMOJI => Self::Emoji(Emoji {
                player_id: r.u8()?,
                emote_id: r.u8()?,
            }),
            // `MessageBuffer.cs:3265` / `QuickStacking.cs:308-321`: int count,
            // the short slot ids, the byte smartStack flag.
            id::QUICK_STACK_CHESTS => {
                let n = r.i32()?;
                if n < 0 {
                    return None;
                }
                let mut slots = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    slots.push(r.i16()?);
                }
                let smart_stack = r.bool()?;
                Self::QuickStackRequest(QuickStackRequest { slots, smart_stack })
            }
            // `MessageBuffer.cs:3465-3468`: string command, int number, float
            // number2, float number3 (the reader consumes all four).
            id::DEV_COMMANDS => Self::DevCommands(DevCommands {
                command: r.str()?,
                number: r.i32()?,
                number2: r.f32()?,
                number3: r.f32()?,
            }),
            // `MessageBuffer.cs:2234-2236`: ReadInt16 + ReadBoolean.
            id::RELEASE_ITEM_OWNERSHIP => Self::ReleaseItemOwnership(ReleaseItemOwnership {
                slot: r.i16()?,
                force_assign_to_server: r.bool()?,
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

/// `65 TeleportEntity` (`MessageBuffer.cs:2949-2978`): the BitsByte first,
/// `(short)` entity, the destination Vector2, `(byte)` style, then the
/// `(int)` extraInfo only when bit 3 is set; kind is reassembled from bits
/// 0/1 (`num82++` / `num82 += 2`). W4-g49 (from W4-d99).
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
            Self::NpcStrikeAck => id::NPC_STRIKE_ACK,
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
            Self::CombatTextString(_) => id::COMBAT_TEXT_STRING,
            Self::PlayerStealth(_) => id::PLAYER_STEALTH,
            Self::NpcExtraValue(_) => id::NPC_EXTRA_VALUE,
            Self::TileEntitySharing(_) => id::TILE_ENTITY_SHARING,
            Self::ItemTweaker(_) => id::ITEM_TWEAKER,
            Self::SpawnInstancedItem(_) => id::SPAWN_INSTANCED_ITEM,
            Self::EmoteBubble(_) => id::EMOTE_BUBBLE,
            Self::PlayerPortalTeleport(_) => id::TELEPORT_PLAYER_PORTAL,
            Self::MinionRestTarget(_) => id::MINION_REST_TARGET,
            Self::MinionAttackTarget(_) => id::MINION_ATTACK_TARGET,
            Self::NpcPortalTeleport(_) => id::NPC_PORTAL_TELEPORT,
            Self::TowerShields(_) => id::TOWER_SHIELDS,
            Self::NebulaLevelupRequest(_) => id::NEBULA_LEVELUP_REQUEST,
            Self::MoonlordCountdown(_) => id::MOONLORD_COUNTDOWN,
            Self::ShopOverride(_) => id::SHOP_OVERRIDE,
            Self::PoofOfSmoke(_) => id::POOF_OF_SMOKE,
            Self::SmartText(_) => id::SMART_TEXT,
            Self::WiredCannonShot(_) => id::WIRED_CANNON_SHOT,
            Self::SpecialFx(_) => id::SPECIAL_FX,
            Self::CrystalInvasionWipe => id::CRYSTAL_INVASION_WIPE,
            Self::CrystalInvasionWait(_) => id::CRYSTAL_INVASION_WAIT,
            Self::PlayerHurtV2(_) => id::PLAYER_HURT_V2,
            Self::PlayerDeathV2(_) => id::PLAYER_DEATH_V2,
            Self::Unused83 => id::UNUSED_83,
            Self::Unused93 => id::UNUSED_93,
            Self::QuickStackBlocked(_) => id::QUICK_STACK_CHESTS,
            Self::DevCommands(_) => id::DEV_COMMANDS,
            Self::Unused25 => id::UNUSED_25,
            Self::Unused26 => id::UNUSED_26,
            Self::ReleaseItemOwnership(_) => id::RELEASE_ITEM_OWNERSHIP,
            Self::SocialHandshake => id::SOCIAL_HANDSHAKE,
            Self::Edit { .. } => id::INT_EDIT,
            Self::ChestItem(_) => id::CHEST_ITEM,
            Self::ChestUpdates(_) => id::CHEST_UPDATES,
            Self::CombatTextInt(_) => id::COMBAT_TEXT_INT,
            Self::PlayerChest(_) => id::PLAYER_CHEST,
            Self::SignData(_) => id::SIGN_DATA,
            Self::PlayerBuffs(_) => id::PLAYER_BUFFS,
            Self::NpcBuffs(_) => id::NPC_BUFFS,
            Self::UniqueTownNpcInfo(_) => id::UNIQUE_TOWN_NPC_INFO,
            Self::TilePaint(_) => id::TILE_PAINT,
            Self::WallPaint(_) => id::WALL_PAINT,
            Self::TeleportEntity(_) => id::TELEPORT_ENTITY,
            Self::HealPlayer(_) => id::HEAL_PLAYER,
            Self::TemporaryAnimation(_) => id::TEMPORARY_ANIMATION,
            Self::InvasionProgress(_) => id::INVASION_PROGRESS,
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
            Self::TilePicking(_) => id::TILE_PICKING,
            Self::ExtraSpawnLoaded { .. } => id::EXTRA_SPAWN_LOADED,

            Self::PlayerHeal(_) => id::PLAYER_HEAL,
            Self::TalkNpc(_) => id::TALK_NPC,
            Self::PlayerZone(_) => id::PLAYER_ZONE,
            Self::ItemAnim(_) => id::ITEM_ANIM,
            Self::ManaEffect(_) => id::MANA_EFFECT,
            Self::WorldEvilBytes(_) => id::WORLD_EVIL_BYTES,
            Self::InstrumentSound(_) => id::INSTRUMENT_SOUND,
            Self::QuestsCountSync(_) => id::QUESTS_COUNT_SYNC,
            Self::PlaceObject(_) => id::PLACE_OBJECT,
            Self::Unknown44 => id::UNKNOWN_44,
            Self::Unknown67 => id::UNKNOWN_67,
            Self::LiquidUpdate(_) => id::LIQUID_UPDATE,
            Self::LockAndUnlock(_) => id::LOCK_AND_UNLOCK,
            Self::RemoveRevengeMarker(_) => id::REMOVE_REVENGE_MARKER,
            Self::TamperWithNpc(_) => id::TAMPER_WITH_NPC,
            Self::UpdatePlayerLuckFactors(_) => id::UPDATE_PLAYER_LUCK_FACTORS,
            Self::CavernMonsterTypes(_) => id::CAVERN_MONSTER_TYPES,
            Self::LucyPopup(_) => id::LUCY_POPUP,
            Self::SpectatePlayer(_) => id::SPECTATE_PLAYER,
            Self::NpcDebuffDamage(_) => id::NPC_DEBUFF_DAMAGE,
            Self::FinishedConnecting => id::FINISHED_CONNECTING,
            Self::CountsAsHost(_) => id::COUNTS_AS_HOST,
            Self::ShimmerAction(_) => id::SHIMMER_ACTIONS,
            Self::ItemUseSound { .. } => id::ITEM_USE_SOUND,
            Self::ItemPosition(_) => id::ITEM_POSITION,
            Self::DisplayDollSync(_) => id::TE_DISPLAY_DOLL_SYNC,
            Self::TileEntityInteraction(_) => id::TE_INTERACTION_REQUEST,
            Self::HatRackItemSync(_) => id::TE_HAT_RACK_ITEM_SYNC,
            Self::SetTime(_) => id::SET_TIME,
            Self::MiscDataSync(_) => id::MISC_DATA_SYNC,
            Self::PlayerBuffPvp(_) => id::ADD_PLAYER_BUFF_PVP,
            Self::SyncDodge(_) => id::SYNC_DODGE,
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
                    b.set(1, item.blocked);
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
            // `SendData` has no case 162 — the bare id is the whole frame.
            Self::NpcStrikeAck => w,
            // `25/26/44/67/83` have no writer arm; bare frames mirror the
            // reader's ignore group (`MessageBuffer.cs:4483-4490`).
            Self::Unused25
            | Self::Unused26
            | Self::Unknown44
            | Self::Unknown67
            | Self::Unused83
            | Self::SocialHandshake => w,
            // `NetMessage.cs:948-951` / `:987-995` — the same bodies as C2s.
            Self::ReleaseItemOwnership(r) => w.i16(r.slot).bool(r.force_assign_to_server),
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
            // `NetMessage.cs:1046-1053`: (short)npc, then the netMode == 2
            // answer's string + (int) variation.
            Self::UniqueTownNpcInfo(t) => {
                let w = w.i16(t.npc);
                match &t.info {
                    Some((name, variation)) => w.str(name).i32(*variation),
                    None => w,
                }
            }
            // `NetMessage.cs:1081-1087`: short x + short y + byte paint +
            // byte coat.
            Self::TilePaint(p) | Self::WallPaint(p) => {
                w.i16(p.x).i16(p.y).u8(p.paint).u8(p.coat)
            }
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
            // `NetMessage.cs:1110-1113`: (byte)player + (short)heal.
            Self::HealPlayer(h) => w.u8(h.player_id).i16(h.heal),
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
            // `NetMessage.cs:1054-1058`: tGood, tEvil, tBlood — three bytes.
            Self::WorldEvilBytes(e) => w.u8(e.good).u8(e.evil).u8(e.blood),
            // `NetMessage.cs:1059-1062` — the shared body.
            Self::InstrumentSound(s) => w.u8(s.player_id).f32(s.pitch),
            // `NetMessage.cs:1149-1153` — the shared quest/golf body.
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
            // `44/67` have no writer arm; bare frames mirror the reader's
            // ignore group (`MessageBuffer.cs:4483-4490`).
            Self::Unknown44 | Self::Unknown67 => w,
            // `NetMessage.cs:987-995` — the same liquid body as C2s.
            Self::LiquidUpdate(l) => w.i16(l.x).i16(l.y).u8(l.liquid).u8(l.liquid_type),
            // `NetMessage.cs:1014-1018` — the same lock body as C2s (the
            // host relay re-sends `number = 0`, never on the wire).
            Self::LockAndUnlock(l) => w.u8(l.action).i16(l.x).i16(l.y),
            // `NetMessage.cs:1188-1196` — float pair, RGB, NetworkText.
            Self::CombatTextString(t) => {
                let w = w.f32(t.x).f32(t.y).rgb(t.color);
                t.text.clone().write_to(w)
            }
            // `NetMessage.cs:1197-1204` — byte id + float stealth.
            Self::PlayerStealth(s) => w.u8(s.player_id).f32(s.stealth),
            // `NetMessage.cs:1321-1326` — short npc, int value, two floats.
            Self::NpcExtraValue(e) => w.i16(e.npc).i32(e.value).f32(e.x).f32(e.y),
            // `NetMessage.cs:1216-1225` — int id, bool present, then the
            // networkSend record when present (the raw bytes carry it).
            Self::TileEntitySharing(t) => {
                let w = w.i32(t.id).bool(!t.record.is_empty());
                w.raw(&t.record)
            }
            // `NetMessage.cs:1232-1296` — the gated inner-stat writes.
            Self::ItemTweaker(t) => t.write_to(w),
            // `NetMessage.cs:646-669` — the shared 21/90 body.
            Self::SpawnInstancedItem(d) => d.write_to(w),
            // `NetMessage.cs:1307-1320`: int id + byte anchor kind; the tail
            // rides unless the kind is 255, and the metadata short only when
            // the signed emote is negative.
            Self::EmoteBubble(b) => {
                let w = w.i32(b.id).u8(b.anchor_kind);
                if b.anchor_kind == 255 {
                    w
                } else {
                    let w = w
                        .u16(b.anchor_index)
                        .u16(b.life_time)
                        .u8(b.emote as u8);
                    if b.emote < 0 {
                        w.i16(b.metadata)
                    } else {
                        w
                    }
                }
            }
            // `NetMessage.cs:1337-1346` — byte player, short color, dest
            // floats, live-velocity floats.
            Self::PlayerPortalTeleport(p) => w
                .u8(p.player)
                .i16(p.color_index)
                .f32(p.x)
                .f32(p.y)
                .f32(p.velocity_x)
                .f32(p.velocity_y),
            // `NetMessage.cs:1353-1356`: (byte)number + WriteVector2.
            Self::MinionRestTarget(m) => w.u8(m.player_id).f32(m.x).f32(m.y),
            // `NetMessage.cs:1357-1360`: (byte)number + (short)target.
            Self::MinionAttackTarget(m) => w.u8(m.player_id).i16(m.npc),
            // `NetMessage.cs:1371-1376`: the four shields in written order.
            Self::TowerShields(t) => w.u16(t.solar).u16(t.vortex).u16(t.nebula).u16(t.stardust),
            // `NetMessage.cs:1361-1369`: (ushort)npc + (short)color + the
            // destination floats + WriteVector2(the NPC's live velocity).
            Self::NpcPortalTeleport(p) => w
                .u16(p.npc)
                .i16(p.color_index)
                .f32(p.x)
                .f32(p.y)
                .f32(p.velocity_x)
                .f32(p.velocity_y),
            // `NetMessage.cs:1377-1382`: byte picker + ushort buffType + the
            // picker-centre floats.
            Self::NebulaLevelupRequest(n) => {
                w.u8(n.player_id).u16(n.buff_type).f32(n.x).f32(n.y)
            }
            // `NetMessage.cs:1383-1386`: the two NPC statics as raw ints,
            // MaxMoonLordCountdown first.
            Self::MoonlordCountdown(m) => w.i32(m.max).i32(m.countdown),
            // `NetMessage.cs:1387-1394`: byte slot, short type, short stack
            // (negative floored at 0), byte prefix, int value, buyOnce byte.
            Self::ShopOverride(s) => w
                .u8(s.slot)
                .i16(s.kind)
                .i16(s.stack.max(0))
                .u8(s.prefix)
                .i32(s.value)
                .u8(u8::from(s.buy_once)),
            // `NetMessage.cs:1400-1402`: the HalfVector2's PackedValue.
            Self::PoofOfSmoke(p) => w.u32(p.packed),
            // `NetMessage.cs:769-774`: the three RGB bytes, the NetworkText,
            // the short width limit.
            Self::SmartText(t) => {
                let w = w.u8(t.color.0).u8(t.color.1).u8(t.color.2);
                let w = t.text.clone().write_to(w);
                w.i16(t.width_limit)
            }
            // `NetMessage.cs:1403-1411`: (short)damage + float knockback +
            // (short)x/y/angle/ammo + (byte)owner.
            Self::WiredCannonShot(c) => w
                .i16(c.damage)
                .f32(c.knockback)
                .i16(c.x)
                .i16(c.y)
                .i16(c.angle)
                .i16(c.ammo)
                .u8(c.owner),
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
            // `NetMessage.cs:1439-1446` via `SendPlayerHurt` (`:2598-2605`).
            Self::PlayerHurtV2(h) => h.write_to(w),
            // `NetMessage.cs:1447-1453` via `SendPlayerDeath` (`:2607-2613`).
            Self::PlayerDeathV2(d) => d.write_to(w),
            // Deprecated (`MessageID.cs:179-180`) — no body.
            Self::Unused83 => w,
            // Deprecated (`MessageID.cs:200`) — no body.
            Self::Unused93 => w,
            // `NetMessage.cs:1213` / `QuickStacking.cs:323-330`: int count,
            // the ushort chest indices.
            Self::QuickStackBlocked(b) => {
                let mut w = w.i32(b.chests.len() as i32);
                for chest in &b.chests {
                    w = w.u16(*chest);
                }
                w
            }
            // `NetMessage.cs:1327-1332`: string command, int number, float
            // number2, float number3.
            Self::DevCommands(d) => w.str(&d.command).i32(d.number).f32(d.number2).f32(d.number3),
            // `NetMessage.cs:1518-1520`: one raw (int) UniqueID.
            Self::RemoveRevengeMarker(m) => w.i32(m.unique_id),
            // `NetMessage.cs:1533-1541`: (ushort)npc, (byte)kind; kind 1
            // appends (int)time + (short)fromWho.
            Self::TamperWithNpc(t) => {
                let w = w.u16(t.npc).u8(t.kind);
                match t.immune {
                    Some((time, from_who)) => w.i32(time).i16(from_who),
                    None => w,
                }
            }
            // `NetMessage.cs:1555-1567`: the eight luck fields (the relay
            // reuses the writer case).
            Self::UpdatePlayerLuckFactors(l) => l.write_to(w),
            // `NetMessage.cs:1572-1582`: the 2×3 grid, row-major (ushort)s.
            Self::CavernMonsterTypes(c) => {
                let mut w = w;
                for row in &c.types {
                    for ty in row {
                        w = w.u16(*ty);
                    }
                }
                w
            }
            // `NetMessage.cs:1595-1602`: (byte)source + (byte)variation +
            // the velocity floats + (int)x + (int)y (the relay reuses the
            // writer case).
            Self::LucyPopup(l) => w
                .u8(l.source)
                .u8(l.variation)
                .f32(l.velocity_x)
                .f32(l.velocity_y)
                .i32(l.x)
                .i32(l.y),
            // `NetMessage.cs:1639-1642`: (byte)number, (short)number2 (the
            // relay reuses the writer case).
            Self::SpectatePlayer(s) => w.u8(s.player_id).i16(s.target),
            // `NetMessage.cs:1646-1649`: (byte)npc, (short)damage.
            Self::NpcDebuffDamage(d) => w.u8(d.npc).i16(d.damage),
            // No writer arm — bodyless (`MessageBuffer.cs:937` TrySendData(129)).
            Self::FinishedConnecting => w,
            // `NetMessage.cs:1587-1590`: (byte)player + bool.
            Self::CountsAsHost(c) => w.u8(c.player_id).bool(c.counts_as_host),
            // `NetMessage.cs:1611-1623`: (byte)kind + the kind's tail.
            Self::ShimmerAction(a) => match *a {
                ShimmerAction::Effect { x, y } => w.u8(0).f32(x).f32(y),
                ShimmerAction::CoinLuck { x, y, amount } => {
                    w.u8(1).f32(x).f32(y).i32(amount)
                }
                ShimmerAction::Unknown(k) => w.u8(k),
            },
            // `NetMessage.cs:1643-1645`: (byte)player only.
            Self::ItemUseSound { player_id } => w.u8(*player_id),
            // `NetMessage.cs:1666-1672`: (short)slot + Vector2 position.
            Self::ItemPosition(p) => w.i16(p.slot).f32(p.x).f32(p.y),
            // `NetMessage.cs:1458-1474` — the shared doll writer (the relay
            // re-derives the tail from the live doll, `MessageBuffer.cs:3938`).
            Self::DisplayDollSync(d) => write_doll_sync(d, w),
            // `NetMessage.cs:1475-1478`: int entityId + (byte)player.
            Self::TileEntityInteraction(t) => w.i32(t.entity_id).u8(t.player_id),
            // `NetMessage.cs:1489-1508` — the folded slot + WriteItem (the
            // relay re-folds the dye flag, `MessageBuffer.cs:4019`).
            Self::HatRackItemSync(h) => h
                .item
                .write_slot(w.u8(h.player_id).i32(h.entity_id).u8(h.wire_slot)),
            // `NetMessage.cs:531-536`: (byte)dayTime + (int)time + sunModY +
            // moonModY (both `short`, `Main.cs:1185`/`:1187`).
            Self::SetTime(t) => w
                .u8(u8::from(t.day_time))
                .i32(t.time)
                .i16(t.sun_mod_y)
                .i16(t.moon_mod_y),
            // `NetMessage.cs:1010-1013`: (byte)number + (byte)number2.
            Self::MiscDataSync(d) => w.u8(d.player_id).u8(d.kind),
            // `NetMessage.cs:1041-1045`: (byte)number + (ushort)number2 +
            // (int)number3.
            Self::PlayerBuffPvp(b) => w.u8(b.player_id).u16(b.buff_type).i32(b.time),
            // `NetMessage.cs:1077-1080`: (byte)number + (byte)number2.
            Self::SyncDodge(d) => w.u8(d.player_id).u8(d.kind),
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
                        blocked: flags.get(1),
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
            // `MessageBuffer.cs:1819-1824`: no body.
            id::NPC_STRIKE_ACK => Self::NpcStrikeAck,
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
            // `MessageBuffer.cs:2614-2622`: (short)npc, then the host
            // answer's string + (int) variation.
            id::UNIQUE_TOWN_NPC_INFO => Self::UniqueTownNpcInfo(UniqueTownNpcInfo {
                npc: r.i16()?,
                info: Some((r.str()?, r.i32()?)),
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
            // `MessageBuffer.cs:2951-2978`: BitsByte + entity + Vector2 +
            // style + gated extraInfo.
            id::TELEPORT_ENTITY => Self::TeleportEntity(read_teleport_entity(r)?),
            // `MessageBuffer.cs:3038-3039`: (byte)player + (short)heal.
            id::HEAL_PLAYER => Self::HealPlayer(HealPlayer {
                player_id: r.u8()?,
                heal: r.i16()?,
            }),
            // `MessageBuffer.cs:3184`: short, ushort, short, short.
            id::TEMPORARY_ANIMATION => Self::TemporaryAnimation(TemporaryAnimation {
                animation_type: r.i16()?,
                tile_type: r.u16()?,
                x: r.i16()?,
                y: r.i16()?,
            }),
            // `MessageBuffer.cs:3189`: int, int, sbyte, sbyte.
            id::INVASION_PROGRESS => Self::InvasionProgress(InvasionProgress {
                progress: r.i32()?,
                progress_max: r.i32()?,
                icon: r.i8()?,
                wave: r.i8()?,
            }),
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
            // `MessageBuffer.cs:2634-2636`: three ReadByte, good→evil→blood.
            id::WORLD_EVIL_BYTES => Self::WorldEvilBytes(WorldEvilBytes {
                good: r.u8()?,
                evil: r.u8()?,
                blood: r.u8()?,
            }),
            // `MessageBuffer.cs:2641-2646` — the shared read.
            id::INSTRUMENT_SOUND => Self::InstrumentSound(InstrumentSound {
                player_id: r.u8()?,
                pitch: r.f32()?,
            }),
            // `MessageBuffer.cs:3166-3175` — the shared quest/golf read.
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
            // `MessageBuffer.cs:4483-4490` — the ignore group: bare frames.
            id::UNKNOWN_44 => Self::Unknown44,
            id::UNKNOWN_67 => Self::Unknown67,
            // `MessageBuffer.cs:2409-2412` — the shared liquid read.
            id::LIQUID_UPDATE => Self::LiquidUpdate(LiquidUpdate {
                x: r.i16()?,
                y: r.i16()?,
                liquid: r.u8()?,
                liquid_type: r.u8()?,
            }),
            // `MessageBuffer.cs:2533-2535` — the shared lock read.
            id::LOCK_AND_UNLOCK => Self::LockAndUnlock(LockAndUnlock {
                action: r.u8()?,
                x: r.i16()?,
                y: r.i16()?,
            }),
            // `MessageBuffer.cs:3235-3242` — float pair, RGB, NetworkText.
            id::COMBAT_TEXT_STRING => Self::CombatTextString(CombatTextString {
                x: r.f32()?,
                y: r.f32()?,
                color: r.rgb()?,
                text: NetText::read_from(r)?,
            }),
            // `MessageBuffer.cs:3248-3253` — byte id + ReadSingle stealth.
            id::PLAYER_STEALTH => Self::PlayerStealth(PlayerStealth {
                player_id: r.u8()?,
                stealth: r.f32()?,
            }),
            // `MessageBuffer.cs:3444-3447`: (short)npc, (int)value, two
            // ReadSingles (the writer's number3/number4 are floats).
            id::NPC_EXTRA_VALUE => Self::NpcExtraValue(NpcExtraValue {
                npc: r.i16()?,
                value: r.i32()?,
                x: r.f32()?,
                y: r.f32()?,
            }),
            // `MessageBuffer.cs:3278-3291`: int id, bool present; the
            // networkSend record rides the rest of the body when present.
            id::TILE_ENTITY_SHARING => {
                let id = r.i32()?;
                let present = r.bool()?;
                let record = if present { r.rest()?.to_vec() } else { Vec::new() };
                Self::TileEntitySharing(TileEntitySharing { id, record })
            }
            // `MessageBuffer.cs:3306-3376` — the gated inner-stat reads.
            id::ITEM_TWEAKER => Self::ItemTweaker(ItemTweaker::read_from(r)?),
            // `MessageBuffer.cs:1445-1493` — the shared 21/90 body.
            id::SPAWN_INSTANCED_ITEM => {
                Self::SpawnInstancedItem(Box::new(ItemState::read_from(r)?))
            }
            // `MessageBuffer.cs:3394-3441`: int id + byte kind; 255 removes
            // and reads nothing else. The metadata gate tests the READ byte
            // (`num247 < 0`) — a byte is never negative, so the tail is dead
            // here even when the writer sent it (`emote < 0`); the port keeps
            // that verbatim (the decoded metadata is always 0).
            id::EMOTE_BUBBLE => {
                let id = r.i32()?;
                let anchor_kind = r.u8()?;
                if anchor_kind == 255 {
                    Self::EmoteBubble(EmoteBubble::removal(id))
                } else {
                    let anchor_index = r.u16()?;
                    let life_time = r.u16()?;
                    let emote = r.u8()?;
                    Self::EmoteBubble(EmoteBubble {
                        id,
                        anchor_kind,
                        anchor_index,
                        life_time,
                        emote: emote as i32,
                        metadata: 0,
                    })
                }
            }
            // `MessageBuffer.cs:3540-3548` — byte player, short color, dest
            // Vector2, live-velocity Vector2.
            id::TELEPORT_PLAYER_PORTAL => Self::PlayerPortalTeleport(PlayerPortalTeleport {
                player: r.u8()?,
                color_index: r.i16()?,
                x: r.f32()?,
                y: r.f32()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
            }),
            // `MessageBuffer.cs:3572-3577` — byte player + ReadVector2.
            id::MINION_REST_TARGET => Self::MinionRestTarget(MinionRestTarget {
                player_id: r.u8()?,
                x: r.f32()?,
                y: r.f32()?,
            }),
            // `MessageBuffer.cs:3586-3591` — byte player + (short) target.
            id::MINION_ATTACK_TARGET => Self::MinionAttackTarget(MinionAttackTarget {
                player_id: r.u8()?,
                npc: r.i16()?,
            }),
            // `MessageBuffer.cs:3614-3617` — the four ushorts in order.
            id::TOWER_SHIELDS => Self::TowerShields(TowerShields {
                solar: r.u16()?,
                vortex: r.u16()?,
                nebula: r.u16()?,
                stardust: r.u16()?,
            }),
            // `MessageBuffer.cs:3600-3604` — ushort npc, short color, dest
            // Vector2, live-velocity Vector2.
            id::NPC_PORTAL_TELEPORT => Self::NpcPortalTeleport(NpcPortalTeleport {
                npc: r.u16()?,
                color_index: r.i16()?,
                x: r.f32()?,
                y: r.f32()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
            }),
            // `MessageBuffer.cs:3654-3656` — byte picker, ushort buffType,
            // the picker-centre Vector2.
            id::NEBULA_LEVELUP_REQUEST => Self::NebulaLevelupRequest(NebulaLevelupRequest {
                player_id: r.u8()?,
                buff_type: r.u16()?,
                x: r.f32()?,
                y: r.f32()?,
            }),
            // `MessageBuffer.cs:3711-3712` — the two raw ints, Max first.
            id::MOONLORD_COUNTDOWN => Self::MoonlordCountdown(MoonlordCountdown {
                max: r.i32()?,
                countdown: r.i32()?,
            }),
            // `MessageBuffer.cs:3719-3724` — byte slot, short type, short
            // stack, byte prefix, int value, the buyOnce BitsByte.
            id::SHOP_OVERRIDE => {
                let slot = r.u8()?;
                let kind = r.i16()?;
                let stack = r.i16()?;
                let prefix = r.u8()?;
                let value = r.i32()?;
                let flags = r.bitsbyte()?;
                Self::ShopOverride(ShopOverride {
                    slot,
                    kind,
                    stack,
                    prefix,
                    value,
                    buy_once: flags.get(0),
                })
            }
            // `MessageBuffer.cs:3747` — the packed HalfVector2 uint.
            id::POOF_OF_SMOKE => Self::PoofOfSmoke(PoofOfSmoke { packed: r.u32()? }),
            // `MessageBuffer.cs:3755` — RGB, NetworkText, short width limit.
            id::SMART_TEXT => Self::SmartText(SmartText {
                color: r.rgb()?,
                text: NetText::read_from(r)?,
                width_limit: r.i16()?,
            }),
            // `MessageBuffer.cs:3761-3767` — short damage, float knockback,
            // short x/y/angle/ammo, byte owner.
            id::WIRED_CANNON_SHOT => Self::WiredCannonShot(WiredCannonShot {
                damage: r.i16()?,
                knockback: r.f32()?,
                x: r.i16()?,
                y: r.i16()?,
                angle: r.i16()?,
                ammo: r.i16()?,
                owner: r.u8()?,
            }),
            // `MessageBuffer.cs:3817-3822` — byte effect, int x/y, byte b4,
            // short s5, byte b6.
            id::SPECIAL_FX => Self::SpecialFx(SpecialFx {
                effect: r.u8()?,
                x: r.i32()?,
                y: r.i32()?,
                b4: r.u8()?,
                s5: r.i16()?,
                b6: r.u8()?,
            }),
            // `MessageBuffer.cs:3855-3860` — no body.
            id::CRYSTAL_INVASION_WIPE => Self::CrystalInvasionWipe,
            // `MessageBuffer.cs:3864` — the raw (int) countdown.
            id::CRYSTAL_INVASION_WAIT => Self::CrystalInvasionWait(CrystalInvasionWait {
                time_left: r.i32()?,
            }),
            // `MessageBuffer.cs:3869-3878`: the direction byte minus 1, the
            // crit/pvp BitsByte, the sbyte cooldown counter.
            id::PLAYER_HURT_V2 => Self::PlayerHurtV2(PlayerHurtV2::read_from(r)?),
            // `MessageBuffer.cs:3889-3897`: the direction byte minus 1, the
            // pvp-only BitsByte — no cooldown sbyte follows.
            id::PLAYER_DEATH_V2 => Self::PlayerDeathV2(PlayerDeathV2::read_from(r)?),
            // `MessageBuffer.cs:4488` — the deprecated ignore group, no body.
            id::UNUSED_83 => Self::Unused83,
            // `MessageBuffer.cs:4489` — the deprecated ignore group, no body.
            id::UNUSED_93 => Self::Unused93,
            // `MessageBuffer.cs:3269` / `QuickStacking.cs:332-341`: int count,
            // the ushort chest indices.
            id::QUICK_STACK_CHESTS => {
                let n = r.i32()?;
                if n < 0 {
                    return None;
                }
                let mut chests = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    chests.push(r.u16()?);
                }
                Self::QuickStackBlocked(QuickStackBlocked { chests })
            }
            // `MessageBuffer.cs:3465-3468`: string command, int number, float
            // number2, float number3 (the reader consumes all four).
            id::DEV_COMMANDS => Self::DevCommands(DevCommands {
                command: r.str()?,
                number: r.i32()?,
                number2: r.f32()?,
                number3: r.f32()?,
            }),
            // `MessageBuffer.cs:4051`: one raw (int) UniqueID.
            id::REMOVE_REVENGE_MARKER => Self::RemoveRevengeMarker(RemoveRevengeMarker {
                unique_id: r.i32()?,
            }),
            // `MessageBuffer.cs:4125-4131`: (ushort)npc, (byte)kind; kind 1
            // appends (int)time + (short)fromWho.
            id::TAMPER_WITH_NPC => {
                let npc = r.u16()?;
                let kind = r.u8()?;
                let immune = if kind == 1 {
                    Some((r.i32()?, r.i16()?))
                } else {
                    None
                };
                Self::TamperWithNpc(TamperWithNpc { npc, kind, immune })
            }
            // `MessageBuffer.cs:4166-4174`: the eight luck fields.
            id::UPDATE_PLAYER_LUCK_FACTORS => {
                Self::UpdatePlayerLuckFactors(UpdatePlayerLuckFactors::read_from(r)?)
            }
            // `MessageBuffer.cs:4210-4216`: exactly six (ushort)s, row-major.
            id::CAVERN_MONSTER_TYPES => {
                let mut types = [[0u16; 3]; 2];
                for row in &mut types {
                    for ty in row {
                        *ty = r.u16()?;
                    }
                }
                Self::CavernMonsterTypes(CavernMonsterTypes { types })
            }
            // `MessageBuffer.cs:4267-4271`: source, variation, velocity
            // floats, then the position ints.
            id::LUCY_POPUP => Self::LucyPopup(LucyPopup {
                source: r.u8()?,
                variation: r.u8()?,
                velocity_x: r.f32()?,
                velocity_y: r.f32()?,
                x: r.i32()?,
                y: r.i32()?,
            }),
            // `MessageBuffer.cs:4363-4368`: (byte)player, (short)target.
            id::SPECTATE_PLAYER => Self::SpectatePlayer(SpectatePlayer {
                player_id: r.u8()?,
                target: r.i16()?,
            }),
            // `MessageBuffer.cs:4410-4411`: (byte)npc, (short)damage.
            id::NPC_DEBUFF_DAMAGE => Self::NpcDebuffDamage(NpcDebuffDamage {
                npc: r.u8()?,
                damage: r.i16()?,
            }),
            // `MessageBuffer.cs:4075-4085`: no body.
            id::FINISHED_CONNECTING => Self::FinishedConnecting,
            // `MessageBuffer.cs:4233-4234`: (byte)player + bool.
            id::COUNTS_AS_HOST => Self::CountsAsHost(CountsAsHost {
                player_id: r.u8()?,
                counts_as_host: r.bool()?,
            }),
            // `MessageBuffer.cs:4313-4325`: (byte)kind + the kind's tail.
            id::SHIMMER_ACTIONS => Self::ShimmerAction(match r.u8()? {
                0 => ShimmerAction::Effect {
                    x: r.f32()?,
                    y: r.f32()?,
                },
                1 => ShimmerAction::CoinLuck {
                    x: r.f32()?,
                    y: r.f32()?,
                    amount: r.i32()?,
                },
                k => ShimmerAction::Unknown(k),
            }),
            // `MessageBuffer.cs:4388`: (byte)player.
            id::ITEM_USE_SOUND => Self::ItemUseSound {
                player_id: r.u8()?,
            },
            // `MessageBuffer.cs:4466-4467`: (short)slot + Vector2.
            id::ITEM_POSITION => Self::ItemPosition(ItemPosition {
                slot: r.i16()?,
                x: r.f32()?,
                y: r.f32()?,
            }),
            // `MessageBuffer.cs:3922-3935` — the shared doll read.
            id::TE_DISPLAY_DOLL_SYNC => Self::DisplayDollSync(read_doll_sync(r)?),
            // `MessageBuffer.cs:3944-3945`: ReadInt32 + ReadByte.
            id::TE_INTERACTION_REQUEST => Self::TileEntityInteraction(TileEntityInteraction {
                entity_id: r.i32()?,
                player_id: r.u8()?,
            }),
            // `MessageBuffer.cs:3997-4016`: byte + int + byte slot + the
            // 5-byte WriteItem tail (the refusal path reads int+byte, the
            // same width).
            id::TE_HAT_RACK_ITEM_SYNC => Self::HatRackItemSync(HatRackItemSync {
                player_id: r.u8()?,
                entity_id: r.i32()?,
                wire_slot: r.u8()?,
                item: TeItem::read_slot(r)?,
            }),
            // `25/26/44/67/83` — the reader's ignore group reads nothing
            // (`MessageBuffer.cs:4483-4490`); bare frames decode.
            id::UNUSED_25 => Self::Unused25,
            id::UNUSED_26 => Self::Unused26,
            id::SOCIAL_HANDSHAKE => Self::SocialHandshake,
            // `MessageBuffer.cs:2234-2236` / `:2409-2412` — the shared
            // reader reads the same shapes on both sides.
            id::RELEASE_ITEM_OWNERSHIP => Self::ReleaseItemOwnership(ReleaseItemOwnership {
                slot: r.i16()?,
                force_assign_to_server: r.bool()?,
            }),
            // `MessageBuffer.cs:1290-1298`: byte == 1 day flag + int + 2 shorts.
            id::SET_TIME => Self::SetTime(SetTime {
                day_time: r.u8()? == 1,
                time: r.i32()?,
                sun_mod_y: r.i16()?,
                moon_mod_y: r.i16()?,
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
            _ => return None,
        })
    }
}
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
    /// server keeps the same per-client copy of the inventory. The key is
    /// the vanilla `PlayerItemSlotID.SlotId` (i16 — bank4 rides 699+;
    /// `MessageBuffer.cs:402` reads `ReadInt16`, W4-c28).
    pub inventory: HashMap<i16, NetItem>,
    /// `RemoteClient.TileSections` (`RemoteClient.cs:31`): the flattened
    /// `sections_w * sections_h` latch grid of the sections already streamed.
    pub sections: Vec<bool>,
    /// The grid width the latch is sized for (0 = not allocated yet).
    pub sections_w: i32,
    /// The peer's remote IP (`Netplay.Clients[id].Socket.GetRemoteAddress()`,
    /// `NetMessage.cs:2880`'s `DoesPlayerSlotCountAsAHost` input). `None`
    /// until the accept records it. W4-g51 FINAL id-139.
    pub peer_ip: Option<std::net::IpAddr>,
    /// `NetMessage.buffer[id].broadcast` (`MessageBuffer.cs:35`/`:81`, set
    /// `:928` when the slot reaches State 10): the `SendData` fan-out gate —
    /// only the default arm's `10 TileBlock` bypasses it for State >= 3
    /// (`NetMessage.cs:1805`).
    pub broadcast: bool,
    /// `RemoteClient.TileSectionsCheckTime` (`RemoteClient.cs:33`): the last
    /// `Main.GameUpdateCount` each section was checked at — `IsSectionActive`
    /// (`:220-224`) reads it with the 60-tick `SectionInactiveTime` window
    /// (`DataStructures/ActiveSections.cs:65`), NOT the streamed latch above.
    pub section_check_time: Vec<u32>,
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
                // The peer's remote IP — `DoesPlayerSlotCountAsAHost`'s
                // `IsLocalHost` input (`NetMessage.cs:2880`, W4-g51 id-139).
                let peer_ip = stream.peer_addr().ok().map(|a| a.ip());
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
                        peer_ip,
                        broadcast: false,
                        section_check_time: Vec::new(),
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

    /// W4-g52 — whether the slot's `NetMessage.buffer[id].broadcast` gate is
    /// open (`MessageBuffer.cs:928`, State 10). The flag-motion callers
    /// (`ProjSync::note_broadcast`'s per-client legs, `NetMessage.cs:1784`)
    /// skip not-connected / not-broadcast slots before writing flags.
    pub fn broadcast_open(&self, id: u32) -> bool {
        self.slots
            .lock()
            .ok()
            .and_then(|s| s.get(&id).map(|c| c.broadcast))
            .unwrap_or(false)
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
                // `NetMessage.buffer[whoAmI].broadcast = true`
                // (`MessageBuffer.cs:928`): the `12 PlayerSpawn` arm raises
                // State 3 -> 10 and opens the slot's `SendData` fan-out gate
                // in the same step. The flag never lowers again (the slot
                // dies instead — `Reset` `:81` clears it on reuse).
                if state >= slot_state::IN_WORLD {
                    c.broadcast = true;
                }
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

    /// `NetMessage.DoesPlayerSlotCountAsAHost` (`NetMessage.cs:2876-2883`):
    /// `State == 10 && Socket.GetRemoteAddress().IsLocalHost()`. The C#
    /// `TcpAddress.IsLocalHost` is `Address.Equals(IPAddress.Loopback)`
    /// (`Net/TcpAddress.cs:23-26`) — exactly IPv4 127.0.0.1 (NOT the IPv6
    /// loopback, NOT the whole 127/8 block). W4-g51 FINAL id-139.
    pub fn slot_counts_as_host(&self, id: u32) -> bool {
        self.slots
            .lock()
            .ok()
            .and_then(|s| {
                s.get(&id).map(|c| {
                    c.state == slot_state::IN_WORLD
                        && c.peer_ip
                            == Some(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
                })
            })
            .unwrap_or(false)
    }

    /// Stores one peer inventory slot — `Main.player[id].inventory[slot]`
    /// (`MessageBuffer.cs:399-462`'s `slot.Item = item`).
    pub fn set_slot_inventory(&self, id: u32, slot: i16, item: NetItem) {
        if let Ok(mut s) = self.slots.lock() {
            if let Some(c) = s.get_mut(&id) {
                c.inventory.insert(slot, item);
            }
        }
    }

    /// The peer's inventory slots (a snapshot).
    pub fn slot_inventory(&self, id: u32) -> HashMap<i16, NetItem> {
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

    /// The `SendData` default-arm fan-out (`NetMessage.cs:1801-1811`): every
    /// connected slot whose `buffer.broadcast` is open, minus `ignore` — the
    /// `ignoreClient` parameter. The one bypass: a State >= 3 slot still
    /// receives `10 TileBlock` while its sections stream (`:1805`'s
    /// `(broadcast || (State >= 3 && msgType == 10))`). The `34/69` and `13`
    /// arms (`:1696-1707`/`:1763-1773`) are the same predicate without the
    /// bypass — which only ever fires for id 10, so this covers them too.
    fn fanout(&self, msg: &S2c, ignore: Option<u32>) {
        self.fanout_filtered(msg, ignore, None, 0);
    }

    /// The per-message relay arms (`NetMessage.cs:1692-1812`): the `20`
    /// SectionRange filter (`:1708-1718`), the `28` strike filter
    /// (`:1751-1762`) and the `27` projectile filter (`:1774-1800`). `ctx`
    /// carries the entity fields those arms read; `now` is
    /// `Main.GameUpdateCount` for the `IsSectionActive` window. The `23`
    /// arm's skipped-sync counters mutate per NPC and live in the decision
    /// layer (`net_npc_sync::send_state`), which emits targeted sends — a
    /// `23` reaching this fn is a priority one (the death/despawn pushes),
    /// where the C# filter is the plain broadcast gate anyway (`:1722-1731`).
    fn fanout_filtered(&self, msg: &S2c, ignore: Option<u32>, ctx: Option<RelayCtx>, now: u32) {
        let frame = msg.encode();
        let msg_id = msg.msg_id();
        // The `20` arm's `SectionRange((int)Math.Max(number3, number4),
        // number, (int)number2)` — the square's size + first corner
        // (`NetMessage.cs:1712`, the SendTileSquare writer `:2629`).
        let square = match msg {
            S2c::SectionRange(r) => Some((
                (r.w as i32).max(r.h as i32),
                r.x as i32,
                r.y as i32,
            )),
            _ => None,
        };
        // Snapshot the eligible ids under the slots lock, then push under the
        // clients lock — never both at once (the accept/reader threads take
        // them in the other order).
        let eligible: Vec<u32> = self
            .slots
            .lock()
            .map(|s| {
                s.iter()
                    .filter(|(id, c)| {
                        if Some(**id) == ignore {
                            return false;
                        }
                        match msg_id {
                            // `:1708-1718` — the 20 arm: broadcast gate +
                            // the four-corner latch test, NO State-3 bypass.
                            id::SECTION_RANGE => {
                                let Some((size, fx, fy)) = square else {
                                    return false;
                                };
                                c.broadcast && slot_section_range(c, size, fx, fy)
                            }
                            // `:1751-1762` — the 28 arm: the dead NPC's
                            // strike reaches everyone; a live one only the
                            // section-active clients.
                            id::NPC_STRIKE => match ctx {
                                Some(RelayCtx::Strike { dead, section }) => {
                                    c.broadcast
                                        && (dead || slot_section_active(c, section.0, section.1, now))
                                }
                                None => false,
                            },
                            // `:1774-1800` — the 27 arm: flag3 bypasses the
                            // section filter; otherwise section-active only.
                            id::PROJECTILE => match ctx {
                                Some(RelayCtx::Projectile {
                                    ignores_sections,
                                    section,
                                }) => {
                                    c.broadcast
                                        && (ignores_sections
                                            || slot_section_active(c, section.0, section.1, now))
                                }
                                None => false,
                            },
                            // `:1801-1811` — the default arm (and 34/69/13).
                            _ => {
                                c.broadcast
                                    || (c.state >= slot_state::SPAWN_REQUESTED
                                        && msg_id == id::TILE_BLOCK)
                            }
                        }
                    })
                    .map(|(id, _)| *id)
                    .collect()
            })
            .unwrap_or_default();
        if let Ok(clients) = self.clients.lock() {
            for id in eligible {
                if let Some(o) = clients.get(&id) {
                    o.push(frame.clone(), false);
                }
            }
        }
    }

    /// Broadcasts to every slot — `SendData(msg, -1, -1, ...)`'s default arm
    /// (`NetMessage.cs:1801-1811`).
    pub fn broadcast(&self, msg: &S2c) {
        self.fanout(msg, None);
    }

    /// Broadcasts to every slot except `except` — the C# `SendData`'s
    /// `ignoreClient` routing (`NetMessage.cs:1692-1826`).
    pub fn broadcast_except(&self, msg: &S2c, except: u32) {
        self.fanout(msg, Some(except));
    }

    /// The `28 DamageNPC` relay (`NetMessage.cs:1751-1762`): every broadcast-
    /// open slot except `ignore`, section-culled unless the NPC died. `now`
    /// is `Main.GameUpdateCount` (`RemoteClient.IsSectionActive`'s clock).
    pub fn relay_strike(&self, msg: &S2c, ignore: u32, ctx: RelayCtx, now: u32) {
        self.fanout_filtered(msg, Some(ignore), Some(ctx), now);
    }

    /// The `27 SyncProjectile` relay (`NetMessage.cs:1774-1800`): every
    /// broadcast-open slot except `ignore`, section-culled unless flag3
    /// (`type 12 / projPet / aiStyle 11 / netImportant`). The skipped-flag
    /// motion runs at the caller through `ProjSync::note_broadcast`.
    pub fn relay_projectile(&self, msg: &S2c, ignore: Option<u32>, ctx: RelayCtx, now: u32) {
        self.fanout_filtered(msg, ignore, Some(ctx), now);
    }

    /// The `20 SendTileSquare` fan-out (`NetMessage.cs:1708-1718`): the
    /// square's four corners must land in a section the slot already holds
    /// (`RemoteClient.SectionRange`, the latch — NOT the check-time grid).
    /// No ignore at the C# sites (`SendTileSquare` passes `whoAmi = -1`).
    pub fn relay_tile_square(&self, msg: &S2c, ignore: Option<u32>) {
        self.fanout_filtered(msg, ignore, None, 0);
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
                if c.section_check_time.len() != want {
                    c.section_check_time = vec![0; want];
                }
            }
        }
    }

    /// `RemoteClient.SectionRange(size, firstX, firstY)`
    /// (`RemoteClient.cs:191-218`): true when ANY of the square's four corners
    /// lands in a section already streamed to this slot (`TileSections`, the
    /// latch — not the check-time grid). The `SendData(20)` relay filter
    /// (`NetMessage.cs:1712`) passes the clamped `max(w, h)` as the size.
    pub fn section_range(&self, id: u32, size: i32, first_x: i32, first_y: i32) -> bool {
        let Ok(s) = self.slots.lock() else {
            return false;
        };
        let Some(c) = s.get(&id) else {
            return false;
        };
        slot_section_range(c, size, first_x, first_y)
    }

    /// `RemoteClient.IsSectionActive(sectionCoords)`
    /// (`RemoteClient.cs:220-224`): the check-time grid + the 60-tick
    /// `SectionInactiveTime` window — NOT the streamed latch. Coords clamp to
    /// `[0, maxSections]` (`ActiveSections.cs:53-56`); an unallocated or
    /// out-of-grid slot reads inactive.
    pub fn is_section_active(&self, id: u32, sx: i32, sy: i32, now: u32) -> bool {
        let Ok(s) = self.slots.lock() else {
            return false;
        };
        let Some(c) = s.get(&id) else {
            return false;
        };
        slot_section_active(c, sx, sy, now)
    }

    /// The `CheckSection_ForClient` stamp (`RemoteClient.cs:165`):
    /// `TileSectionsCheckTime[i, j] = Main.GameUpdateCount` for every section
    /// in the fluff box around the position — the `IsSectionActive` refresh.
    pub fn mark_section_checked(&self, id: u32, sx: i32, sy: i32, now: u32) {
        let Ok(mut s) = self.slots.lock() else {
            return;
        };
        let Some(c) = s.get_mut(&id) else {
            return;
        };
        if c.sections_w == 0 || sx < 0 || sy < 0 {
            return;
        }
        let i = (sy * c.sections_w + sx) as usize;
        if let Some(t) = c.section_check_time.get_mut(i) {
            *t = now;
        }
    }

    /// W4-g52 — a snapshot of the slot's check-time grid + its width, for
    /// the cadence's per-(client, NPC-section) `IsSectionActive` reads
    /// (`NetMessage.cs:1740`). The decision layer evaluates
    /// `section_active_at(t, now)` off this same-tick snapshot (the stamps
    /// run ahead of the cadence in `net_tick`'s fixed-step loop).
    pub fn section_check_times(&self, id: u32) -> (i32, Vec<u32>) {
        self.slots
            .lock()
            .ok()
            .and_then(|s| {
                s.get(&id)
                    .map(|c| (c.sections_w, c.section_check_time.clone()))
            })
            .unwrap_or((0, Vec::new()))
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
                    blocked: false,
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

    /// W4-g49 `56 UniqueTownNPCInfoSyncRequest` (`NetMessage.cs:1046-1053`):
    /// (short)npc; the netMode == 2 answer appends GivenName (7-bit length +
    /// UTF-8) + (int)variation. The client request is the bare slot
    /// (`NPC.cs:90564`). Byte-verbatim from W4-d92.
    #[test]
    fn w4_g49_unique_town_npc_info_round_trips() {
        let req = UniqueTownNpcInfo {
            npc: 17,
            info: None,
        };
        let c = C2s::UniqueTownNpcInfo(req.clone()).encode();
        assert_eq!(c, vec![5, 0, 56, 17, 0]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::UniqueTownNpcInfo(d) => assert_eq!(d, req),
            other => panic!("expected 56, got {:?}", other.msg_id()),
        }
        // The host answer (`MessageBuffer.cs:2619-2622` reads string + int).
        let ans = UniqueTownNpcInfo {
            npc: 17,
            info: Some(("Andrew".into(), 1)),
        };
        let f = S2c::UniqueTownNpcInfo(ans.clone()).encode();
        assert_eq!(f[2], 56);
        let mut body = Vec::new();
        body.extend_from_slice(&17i16.to_le_bytes());
        body.push(6); // Write7BitEncodedInt(6) — one byte
        body.extend_from_slice(b"Andrew");
        body.extend_from_slice(&1i32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::UniqueTownNpcInfo(d) => assert_eq!(d, ans),
            other => panic!("expected 56, got {:?}", other.msg_id()),
        }
        // Truncated before the variation int must not decode.
        assert!(S2c::decode(56, &mut NetReader::new(&body[..12])).is_none());
        // The bare-slot request must not decode on the S2c side (the client
        // arm always reads the string + int, `MessageBuffer.cs:2619-2621`).
        assert!(S2c::decode(56, &mut NetReader::new(&[17, 0])).is_none());
    }

    /// W4-g49 `61 SpawnBossInvader` (`NetMessage.cs:1073-1076`): (short)player
    /// + (short)selector. Positive = an NPCID boss; negative = the event
    /// ladder — pin -19 (slime rain) so the sign bit survives the i16 round
    /// trip. Client→server only. Byte-verbatim from W4-d99.
    #[test]
    fn w4_g49_spawn_boss_event_round_trips() {
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

    /// W4-g49 `63/64 SyncTilePaintOrCoating / SyncWallPaintOrCoating`
    /// (`NetMessage.cs:1081-1087`): 63 and 64 share the case — (short)x +
    /// (short)y + (byte)paint + (byte)coat. Byte-verbatim from W4-d90.
    #[test]
    fn w4_g49_tile_and_wall_paint_round_trip() {
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

    /// W4-g49 `65 TeleportEntity` (`NetMessage.cs:1088-1103`): BitsByte27
    /// (bit0/1 = kind low bits, bit2 = getPosFromPlayer, bit3 = extraInfo
    /// gate), (short)entity, float x, float y, (byte)style, gated (int)extra.
    /// Byte-verbatim from W4-d99.
    #[test]
    fn w4_g49_teleport_entity_round_trips() {
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

    /// W4-g49 `66 Unknown66` (`NetMessage.cs:1110-1113`): (byte)number +
    /// (short)number2 — the Moon Leech Clot's heal (`Projectile.cs:28524`).
    /// Byte-verbatim from W4-d94.
    #[test]
    fn w4_g49_heal_player_round_trips() {
        let h = HealPlayer {
            player_id: 2,
            heal: 37,
        };
        let f = S2c::HealPlayer(h).encode();
        assert_eq!(f[2], 66);
        let mut body = vec![2];
        body.extend_from_slice(&37i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::HealPlayer(d) => assert_eq!(d, h),
            other => panic!("expected 66, got {:?}", other.msg_id()),
        }
        let c = C2s::HealPlayer(h).encode();
        assert_eq!(c[2], 66);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::HealPlayer(d) => assert_eq!(d, h),
            other => panic!("expected 66, got {:?}", other.msg_id()),
        }
    }

    /// W4-g49 `68 Unknown68` (`NetMessage.cs:1114-1116`): one
    /// BinaryWriter.Write(string) — Main.clientUUID, a Guid string, so the
    /// wire is the 7-bit byte count (36 = 0x24) + the ASCII. Byte-verbatim
    /// from W4-d95.
    #[test]
    fn w4_g49_client_uuid_round_trips() {
        let uuid = "0f8fad5b-d9cb-469f-a165-70867728950e";
        let u = ClientUuid {
            uuid: uuid.to_string(),
        };
        let c = C2s::ClientUuid(u.clone()).encode();
        assert_eq!(c[2], 68);
        let mut body = vec![36];
        body.extend_from_slice(uuid.as_bytes());
        assert_eq!(&c[3..], &body);
        assert_eq!(c.len(), 3 + 1 + 36);
        let mut r = NetReader::new(&c[3..]);
        match C2s::decode(c[2], &mut r).expect("decode") {
            C2s::ClientUuid(d) => assert_eq!(d, u),
            other => panic!("expected 68, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The length prefix counts UTF-8 BYTES, not chars (BinaryWriter).
        let multi = ClientUuid {
            uuid: "aé☃".to_string(), // 1 + 2 + 3 = 6 bytes
        };
        let c = C2s::ClientUuid(multi.clone()).encode();
        assert_eq!(&c[3..4], &[6]);
        match C2s::decode(68, &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ClientUuid(d) => assert_eq!(d, multi),
            other => panic!("expected 68, got {:?}", other.msg_id()),
        }
        // The empty string is a bare 0 length byte; a truncated payload
        // must not decode (`MessageBuffer.cs:3056-3058` ReadString).
        let empty = C2s::ClientUuid(ClientUuid {
            uuid: String::new(),
        })
        .encode();
        assert_eq!(&empty[3..], &[0]);
        assert!(C2s::decode(68, &mut NetReader::new(&[36, b'a'])).is_none());
    }

    /// W4-g49 `73 RequestTeleport` (`NetMessage.cs:1142-1144`): one (byte)kind
    /// — the reader's ladder 0..4 (`MessageBuffer.cs:3128-3145`:
    /// potion/magic conch/demon conch/shellphone/no-space). Byte-verbatim
    /// from W4-d99.
    #[test]
    fn w4_g49_request_teleport_round_trips() {
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

    /// W4-g49 `75 AnglerQuestFinished`: no `case 75` writer arm in
    /// NetMessage.cs and the reader consumes nothing
    /// (`MessageBuffer.cs:3154-3163` — it only adds the sender's name on the
    /// host): the frame is exactly [len=3][75]. Byte-verbatim from W4-d99.
    #[test]
    fn w4_g49_angler_quest_finished_is_bodyless() {
        let f = C2s::AnglerQuestFinished.encode();
        assert_eq!(f, vec![3, 0, 75]);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::AnglerQuestFinished => {}
            other => panic!("expected 75, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
    }

    /// W4-g49 `77 SyncTilePicking` (`NetMessage.cs:1154-1159`): (short)number
    /// (animationType), (ushort)number2 (tileType), (short)number3 (x),
    /// (short)number4 (y) — `SendTemporaryAnimation`'s argument order
    /// (`:2594`); the reader feeds Animation.NewTemporaryAnimation
    /// (`MessageBuffer.cs:3184`). Server→client. Byte-verbatim from W4-d99.
    #[test]
    fn w4_g49_temporary_animation_round_trips() {
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

    /// W4-g49 `78 ReportInvasionProgress` (`NetMessage.cs:1160-1165`):
    /// (int)progress, (int)max, (sbyte)icon, (sbyte)wave. Pin the DD2 shape
    /// (icon 3, wave 5, `DD2Event.cs:191`) and a negative sbyte so the sign
    /// survives. Byte-verbatim from W4-d99.
    #[test]
    fn w4_g49_invasion_progress_round_trips() {
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

    /// W4-g49 F21-TAIL `57 Unknown57` — byte-pin transplanted verbatim
    /// from W4-f21 batch 4 (`w4_f21_world_evil_bytes_round_trips`).
    #[test]
    fn w4_g49_world_evil_bytes_round_trips() {
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

    /// W4-g49 F21-TAIL `58 InstrumentSound` — byte-pin transplanted verbatim
    /// from W4-f21 batch 5 (`w4_f21_instrument_sound_round_trips_both_directions`).
    #[test]
    fn w4_g49_instrument_sound_round_trips_both_directions() {
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

    /// W4-g49 F21-TAIL `76 QuestsCountSync` — byte-pin transplanted verbatim
    /// from W4-f21 batch 6 (`w4_f21_quests_count_sync_round_trips_both_directions`).
    #[test]
    fn w4_g49_quests_count_sync_round_trips_both_directions() {
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

    /// W4-g49 F21-TAIL `79 PlaceObject` — byte-pin transplanted verbatim
    /// from W4-f21 batch 6 (`w4_f21_place_object_round_trips_both_directions`).
    #[test]
    fn w4_g49_place_object_round_trips_both_directions() {
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

    /// W4-g49 F21-TAIL `48 LiquidUpdate` — byte-pin transplanted verbatim
    /// from W4-f21 batch 3 (`w4_f21_liquid_update_round_trips_both_directions`).
    #[test]
    fn w4_g49_liquid_update_round_trips_both_directions() {
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

    /// W4-g49 F21-TAIL `52 LockAndUnlock` — byte-pin transplanted from
    /// W4-f21 batch 3 (`w4_f21_lock_and_unlock_round_trips`), extended with
    /// the S2c direction the host relay (`MessageBuffer.cs:2541`) needs.
    #[test]
    fn w4_g49_lock_and_unlock_round_trips_both_directions() {
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
            // The host relay carries the same body back to the clients.
            let f = S2c::LockAndUnlock(b).encode();
            assert_eq!(f[2], 52);
            assert_eq!(&f[3..], &body);
            match S2c::decode(52, &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::LockAndUnlock(d) => assert_eq!(d, b),
                other => panic!("expected 52, got {:?}", other.msg_id()),
            }
        }
        // A truncated y must not decode.
        assert!(C2s::decode(52, &mut NetReader::new(&[1, 0, 0, 0])).is_none());
    }

    /// W4-g49 F21-TAIL `44 Unknown44` / `67 Unknown67` — the reader ignore
    /// group (`MessageBuffer.cs:4483-4490`): no writer arm, no body, bare
    /// frames accepted and dropped. Transplanted from W4-f21's
    /// `w4_f21_deprecated_ignore_group_26_44_67_83_are_bare_frames` (the
    /// 26/83 twins are g48/g50's, keep-one at union).
    #[test]
    fn w4_g49_ignore_group_44_67_are_bare_frames() {
        for (msg, expect_id) in [(S2c::Unknown44, 44u8), (S2c::Unknown67, 67u8)] {
            let f = msg.encode();
            assert_eq!(f[2], expect_id);
            assert_eq!(f.len(), 3);
            let mut r = NetReader::new(&f[3..]);
            let d = S2c::decode(f[2], &mut r).expect("decode");
            assert_eq!(d.msg_id(), expect_id);
            assert_eq!(r.remaining(), 0);
        }
        // No C2s variant — the ids have no writer arm (client send sites).
        assert!(C2s::decode(44, &mut NetReader::new(&[])).is_none());
        assert!(C2s::decode(67, &mut NetReader::new(&[])).is_none());
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
            blocked: false,
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

    // ---- W4-g50 (S1-I-4 lane 3): the 81-120 apply region ----
    // WRITTEN NOT RUN (code-only swarm, no compiler) — the byte pins follow
    // the d89-d100 convention: the frame's id byte + the exact C# body bytes.

    /// `81 CombatTextInt` (`NetMessage.cs:1179-1186`): float x, float y,
    /// `WriteRGB`'s R/G/B bytes (`Utils.cs:1369-1374`), `(int)` amount.
    /// Host→client only — no `C2s` arm (`MessageBuffer.cs:3227-3233` reads
    /// it at `netMode == 1` only).
    #[test]
    fn w4_g50_combat_text_int_round_trips() {
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

    /// `119 CombatTextString` (`NetMessage.cs:1188-1196`): the 81 twin —
    /// float x, float y, RGB, then the `NetworkText` (literal mode: the mode
    /// byte 0 + `BinaryWriter.Write(string)`). Host→client only
    /// (`MessageBuffer.cs:3235-3242`).
    #[test]
    fn w4_g50_combat_text_string_round_trips() {
        let t = CombatTextString {
            x: -12.5,
            y: 900.75,
            color: Rgb(255, 160, 80), // CombatText.DamagedHostile
            text: NetText("Belsin was slain...".into()),
        };
        let f = S2c::CombatTextString(t.clone()).encode();
        assert_eq!(f[2], 119);
        let mut body = Vec::new();
        body.extend_from_slice(&(-12.5f32).to_le_bytes());
        body.extend_from_slice(&900.75f32.to_le_bytes());
        body.extend_from_slice(&[255, 160, 80]);
        body.push(0); // NetworkText literal mode (`NetworkText.cs:65-70`)
        body.push(20); // the 7-bit length of "Belsin was slain..."
        body.extend_from_slice(b"Belsin was slain...");
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::CombatTextString(d) => assert_eq!(d, t),
            other => panic!("expected 119, got {:?}", other.msg_id()),
        }
        // A truncated body (no text) is malformed, not a partial frame.
        assert!(S2c::decode(119, &mut NetReader::new(&body[..11])).is_none());
    }

    /// `84 PlayerStealth` (`NetMessage.cs:1197-1204`): byte player id +
    /// float stealth. Both directions — the host forces the id to the
    /// sender and relays (`MessageBuffer.cs:3249-3258`).
    #[test]
    fn w4_g50_player_stealth_round_trips() {
        let s = PlayerStealth {
            player_id: 2,
            stealth: 0.375,
        };
        let f = S2c::PlayerStealth(s).encode();
        assert_eq!(f[2], 84);
        let mut body = vec![2u8];
        body.extend_from_slice(&0.375f32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerStealth(d) => assert_eq!(d, s),
            other => panic!("expected 84, got {:?}", other.msg_id()),
        }
        let c = C2s::PlayerStealth(s).encode();
        assert_eq!(c[2], 84);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerStealth(d) => assert_eq!(d, s),
            other => panic!("expected 84, got {:?}", other.msg_id()),
        }
        // The C# default (`Player.cs:1562`): a fresh peer snapshot is 1f.
        assert_eq!(PlayerNet::default().stealth, 1.0);
    }

    /// `92 SyncExtraValue` (`NetMessage.cs:1321-1326`): short npc, int
    /// value, two floats — the writer's number3/number4 are floats and the
    /// reader `ReadSingle`s them (`MessageBuffer.cs:3444-3447`). A DELTA in
    /// on the client leg, the TOTAL out on the host leg.
    #[test]
    fn w4_g50_npc_extra_value_round_trips() {
        let e = NpcExtraValue {
            npc: 17,
            value: -250,
            x: 123.5,
            y: -67.25,
        };
        let c = C2s::NpcExtraValue(e).encode();
        assert_eq!(c[2], 92);
        let mut body = Vec::new();
        body.extend_from_slice(&17i16.to_le_bytes());
        body.extend_from_slice(&(-250i32).to_le_bytes());
        body.extend_from_slice(&123.5f32.to_le_bytes());
        body.extend_from_slice(&(-67.25f32).to_le_bytes());
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::NpcExtraValue(d) => assert_eq!(d, e),
            other => panic!("expected 92, got {:?}", other.msg_id()),
        }
        let f = S2c::NpcExtraValue(e).encode();
        assert_eq!(f[2], 92);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::NpcExtraValue(d) => assert_eq!(d, e),
            other => panic!("expected 92, got {:?}", other.msg_id()),
        }
    }

    /// `86 TileEntitySharing` (`NetMessage.cs:1216-1225`): int id, bool
    /// present, then the `networkSend: true` record — type byte, position
    /// shorts, extra data, NO id (`TileEntity.cs:188-197`). The item-frame
    /// record is the raw triple (no presence byte, `TEItemFrame.cs:40-45`).
    #[test]
    fn w4_g50_tile_entity_sharing_round_trips() {
        use tr_core::tile_entities::{TeData, TeItem, TeKind, TileEntity};
        let e = TileEntity {
            id: 7,
            x: 120,
            y: 340,
            kind: TeKind::ItemFrame,
            data: TeData::Item(Some(TeItem {
                kind: 29,
                prefix: 81,
                stack: 1,
            })),
        };
        let mut w = tr_core::binio::Writer::new();
        e.write_net(&mut w);
        let record = w.into_inner();
        // type 1, x 120, y 340, then the triple (29, 81, 1) — 9 bytes, no id.
        let mut expect = vec![1u8];
        expect.extend_from_slice(&120i16.to_le_bytes());
        expect.extend_from_slice(&340i16.to_le_bytes());
        expect.extend_from_slice(&29i16.to_le_bytes());
        expect.push(81);
        expect.extend_from_slice(&1i16.to_le_bytes());
        assert_eq!(record, expect);
        let t = TileEntitySharing {
            id: 7,
            record: record.clone(),
        };
        let f = S2c::TileEntitySharing(t.clone()).encode();
        assert_eq!(f[2], 86);
        let mut body = Vec::new();
        body.extend_from_slice(&7i32.to_le_bytes());
        body.push(1); // present
        body.extend_from_slice(&record);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TileEntitySharing(d) => assert_eq!(d, t),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
        // The record decodes back to the entity (the id rides the body head).
        let back = TileEntity::read_net(&mut tr_core::binio::Reader::new(&record))
            .expect("record decodes");
        assert_eq!(back.kind, TeKind::ItemFrame);
        assert_eq!((back.x, back.y), (120, 340));
        assert_eq!(back.data, e.data);
        // The removal shape: id + false, no record.
        let gone = TileEntitySharing {
            id: 7,
            record: Vec::new(),
        };
        let f = S2c::TileEntitySharing(gone.clone()).encode();
        assert_eq!(f.len(), 3 + 4 + 1);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TileEntitySharing(d) => assert_eq!(d, gone),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
    }

    /// `87 TileEntityPlacement` (`NetMessage.cs:1227-1231`): short x, short
    /// y, byte type. Client→host only (`MessageBuffer.cs:3294-3305`).
    #[test]
    fn w4_g50_tile_entity_placement_round_trips() {
        let p = TileEntityPlacement {
            x: 120,
            y: 340,
            kind: 1, // TEItemFrame
        };
        let c = C2s::TileEntityPlacement(p).encode();
        assert_eq!(c[2], 87);
        let mut body = Vec::new();
        body.extend_from_slice(&120i16.to_le_bytes());
        body.extend_from_slice(&340i16.to_le_bytes());
        body.push(1);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TileEntityPlacement(d) => assert_eq!(d, p),
            other => panic!("expected 87, got {:?}", other.msg_id()),
        }
    }

    /// `88 ItemTweaker` (`NetMessage.cs:1232-1296`): short slot, BitsByte,
    /// the gated fields in bit order; bit 7 opens the second BitsByte.
    /// Host→client only (`MessageBuffer.cs:3306-3376`).
    #[test]
    fn w4_g50_item_tweaker_round_trips() {
        // A full tweak: every flag set, both BitsBytes.
        let mut flags = BitsByte::new();
        for i in 0..8 {
            flags.set(i, true);
        }
        let mut flags2 = BitsByte::new();
        for i in 0..6 {
            flags2.set(i, true);
        }
        let t = ItemTweaker {
            slot: 12,
            flags,
            color: 0xFF804020,
            damage: 35,
            knockback: 6.5,
            use_animation: 20,
            use_time: 15,
            shoot: 29,
            shoot_speed: 12.0,
            flags2,
            width: 24,
            height: 24,
            scale: 1.25,
            ammo: 4,
            use_ammo: 5,
            not_ammo: true,
        };
        let f = S2c::ItemTweaker(t).encode();
        assert_eq!(f[2], 88);
        let mut body = Vec::new();
        body.extend_from_slice(&12i16.to_le_bytes());
        body.push(0xFF); // flags
        body.extend_from_slice(&0xFF804020u32.to_le_bytes());
        body.extend_from_slice(&35u16.to_le_bytes());
        body.extend_from_slice(&6.5f32.to_le_bytes());
        body.extend_from_slice(&20u16.to_le_bytes());
        body.extend_from_slice(&15u16.to_le_bytes());
        body.extend_from_slice(&29i16.to_le_bytes());
        body.extend_from_slice(&12.0f32.to_le_bytes());
        body.push(0x3F); // flags2 (bits 0-5)
        body.extend_from_slice(&24u16.to_le_bytes());
        body.extend_from_slice(&24u16.to_le_bytes());
        body.extend_from_slice(&1.25f32.to_le_bytes());
        body.extend_from_slice(&4i16.to_le_bytes());
        body.extend_from_slice(&5i16.to_le_bytes());
        body.push(1); // not_ammo
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemTweaker(d) => assert_eq!(d, t),
            other => panic!("expected 88, got {:?}", other.msg_id()),
        }
        // A minimal tweak: only the damage flag — no second BitsByte.
        let mut flags = BitsByte::new();
        flags.set(1, true);
        let t = ItemTweaker {
            slot: 0,
            flags,
            damage: 10,
            ..Default::default()
        };
        let f = S2c::ItemTweaker(t).encode();
        let mut body = Vec::new();
        body.extend_from_slice(&0i16.to_le_bytes());
        body.push(0x02);
        body.extend_from_slice(&10u16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemTweaker(d) => assert_eq!(d, t),
            other => panic!("expected 88, got {:?}", other.msg_id()),
        }
    }

    /// `89 ItemFrameTryPlacing` (`NetMessage.cs:1297-1306`): short x, short
    /// y, short type, byte prefix, short stack. Client→host only
    /// (`MessageBuffer.cs:3377-3393`).
    #[test]
    fn w4_g50_item_frame_try_placing_round_trips() {
        let p = TryPlacingItem {
            x: 120,
            y: 340,
            kind: 29,
            prefix: 81,
            stack: 1,
        };
        let c = C2s::ItemFrameTryPlacing(p).encode();
        assert_eq!(c[2], 89);
        let mut body = Vec::new();
        body.extend_from_slice(&120i16.to_le_bytes());
        body.extend_from_slice(&340i16.to_le_bytes());
        body.extend_from_slice(&29i16.to_le_bytes());
        body.push(81);
        body.extend_from_slice(&1i16.to_le_bytes());
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ItemFrameTryPlacing(d) => assert_eq!(d, p),
            other => panic!("expected 89, got {:?}", other.msg_id()),
        }
    }

    /// `90 SpawnInstancedItem` (`NetMessage.cs:646-669`'s shared `case 21:
    /// case 90:`): the body is the 21 ItemDrop verbatim — slot, position,
    /// velocity, stack, prefix, flags, type (+ the shimmer tail on bit 2).
    #[test]
    fn w4_g50_spawn_instanced_item_is_the_21_body() {
        let d = ItemState {
            slot: 12,
            x: 100.5,
            y: 200.25,
            vx: 1.5,
            vy: -2.2,
            stack: 3,
            prefix: 81,
            kind: 29,
            shimmered: false,
            shimmer_time: 0.0,
        };
        let f = S2c::SpawnInstancedItem(Box::new(d)).encode();
        assert_eq!(f[2], 90);
        // Byte-identical to the 21 frame apart from the id byte.
        let g = S2c::ItemState(Box::new(d)).encode();
        assert_eq!(g[2], 21);
        assert_eq!(&f[3..], &g[3..]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SpawnInstancedItem(back) => assert_eq!(*back, d),
            other => panic!("expected 90, got {:?}", other.msg_id()),
        }
    }

    /// `91 SyncEmoteBubble` (`NetMessage.cs:1307-1320`): int id, byte
    /// anchor kind; the tail rides unless the kind is 255, and the metadata
    /// short only when the signed emote is negative. The reader's metadata
    /// gate tests the READ byte (`num247 < 0`) — dead, so the decoded
    /// metadata is always 0 (`MessageBuffer.cs:3412-3417`).
    #[test]
    fn w4_g50_emote_bubble_round_trips() {
        let b = EmoteBubble {
            id: 42,
            anchor_kind: 1,
            anchor_index: 3,
            life_time: 360,
            emote: 17,
            metadata: 0,
        };
        let f = S2c::EmoteBubble(b).encode();
        assert_eq!(f[2], 91);
        let mut body = Vec::new();
        body.extend_from_slice(&42i32.to_le_bytes());
        body.push(1);
        body.extend_from_slice(&3u16.to_le_bytes());
        body.extend_from_slice(&360u16.to_le_bytes());
        body.push(17);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::EmoteBubble(d) => assert_eq!(d, b),
            other => panic!("expected 91, got {:?}", other.msg_id()),
        }
        // The removal shape: id + 255, no tail.
        let gone = EmoteBubble::removal(42);
        let f = S2c::EmoteBubble(gone).encode();
        assert_eq!(f.len(), 3 + 4 + 1);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::EmoteBubble(d) => assert_eq!(d, gone),
            other => panic!("expected 91, got {:?}", other.msg_id()),
        }
        // The negative-emote writer quirk: the metadata short rides the
        // wire, but the reader's dead gate drops it (metadata decodes 0).
        let neg = EmoteBubble {
            id: 7,
            anchor_kind: 0,
            anchor_index: 5,
            life_time: 180,
            emote: -1,
            metadata: 22,
        };
        let f = S2c::EmoteBubble(neg).encode();
        let mut body = Vec::new();
        body.extend_from_slice(&7i32.to_le_bytes());
        body.push(0);
        body.extend_from_slice(&5u16.to_le_bytes());
        body.extend_from_slice(&180u16.to_le_bytes());
        body.push(0xFF); // (byte)(-1)
        body.extend_from_slice(&22i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::EmoteBubble(d) => {
                assert_eq!(d.emote, 255, "the read byte widens unsigned");
                assert_eq!(d.metadata, 0, "the dead gate drops the tail");
            }
            other => panic!("expected 91, got {:?}", other.msg_id()),
        }
    }

    /// `95 MurderSomeoneElsesPortal` (`NetMessage.cs:1333-1336`): ushort
    /// owner + byte form. Client→host only (`MessageBuffer.cs:3519-3537`).
    #[test]
    fn w4_g50_murder_portal_round_trips() {
        let c = C2s::MurderPortal { owner: 3, form: 1 }.encode();
        assert_eq!(c[2], 95);
        let mut body = Vec::new();
        body.extend_from_slice(&3u16.to_le_bytes());
        body.push(1);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::MurderPortal { owner, form } => assert_eq!((owner, form), (3, 1)),
            other => panic!("expected 95, got {:?}", other.msg_id()),
        }
    }

    /// `96 TeleportPlayerThroughPortal` (`NetMessage.cs:1337-1346`): byte
    /// player, short color, dest Vector2, live-velocity Vector2 — the
    /// velocity is the entity's CURRENT velocity, not the destination
    /// echoed back. Both directions.
    #[test]
    fn w4_g50_player_portal_teleport_round_trips() {
        let p = PlayerPortalTeleport {
            player: 2,
            color_index: 5,
            x: 1234.5,
            y: -678.25,
            velocity_x: 3.5,
            velocity_y: -1.25,
        };
        let mut body = Vec::new();
        body.push(2);
        body.extend_from_slice(&5i16.to_le_bytes());
        body.extend_from_slice(&1234.5f32.to_le_bytes());
        body.extend_from_slice(&(-678.25f32).to_le_bytes());
        body.extend_from_slice(&3.5f32.to_le_bytes());
        body.extend_from_slice(&(-1.25f32).to_le_bytes());
        let c = C2s::PlayerPortalTeleport(p).encode();
        assert_eq!(c[2], 96);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerPortalTeleport(d) => assert_eq!(d, p),
            other => panic!("expected 96, got {:?}", other.msg_id()),
        }
        let f = S2c::PlayerPortalTeleport(p).encode();
        assert_eq!(f[2], 96);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerPortalTeleport(d) => assert_eq!(d, p),
            other => panic!("expected 96, got {:?}", other.msg_id()),
        }
    }

    /// `99 MinionRestTargetUpdate` (`NetMessage.cs:1353-1356`): byte player
    /// + WriteVector2. Both directions — the host forces the id and relays
    /// (`MessageBuffer.cs:3570-3583`).
    #[test]
    fn w4_g50_minion_rest_target_round_trips() {
        let m = MinionRestTarget {
            player_id: 2,
            x: 320.5,
            y: 640.25,
        };
        let mut body = vec![2u8];
        body.extend_from_slice(&320.5f32.to_le_bytes());
        body.extend_from_slice(&640.25f32.to_le_bytes());
        let c = C2s::MinionRestTarget(m).encode();
        assert_eq!(c[2], 99);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::MinionRestTarget(d) => assert_eq!(d, m),
            other => panic!("expected 99, got {:?}", other.msg_id()),
        }
        let f = S2c::MinionRestTarget(m).encode();
        assert_eq!(f[2], 99);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::MinionRestTarget(d) => assert_eq!(d, m),
            other => panic!("expected 99, got {:?}", other.msg_id()),
        }
    }

    /// `115 MinionAttackTargetUpdate` (`NetMessage.cs:1357-1360`): byte
    /// player + (short) target (-1 = none). Both directions
    /// (`MessageBuffer.cs:3584-3597`).
    #[test]
    fn w4_g50_minion_attack_target_round_trips() {
        let m = MinionAttackTarget {
            player_id: 3,
            npc: -1,
        };
        let mut body = vec![3u8];
        body.extend_from_slice(&(-1i16).to_le_bytes());
        let c = C2s::MinionAttackTarget(m).encode();
        assert_eq!(c[2], 115);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::MinionAttackTarget(d) => assert_eq!(d, m),
            other => panic!("expected 115, got {:?}", other.msg_id()),
        }
        let f = S2c::MinionAttackTarget(m).encode();
        assert_eq!(f[2], 115);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::MinionAttackTarget(d) => assert_eq!(d, m),
            other => panic!("expected 115, got {:?}", other.msg_id()),
        }
    }

    /// `100 TeleportNPCThroughPortal` (`NetMessage.cs:1361-1369`): ushort
    /// npc, short color, dest Vector2, live-velocity Vector2. Host→client
    /// only (`MessageBuffer.cs:3598-3610`).
    #[test]
    fn w4_g50_npc_portal_teleport_round_trips() {
        let p = NpcPortalTeleport {
            npc: 42,
            color_index: 3,
            x: 900.5,
            y: 1200.25,
            velocity_x: -2.0,
            velocity_y: 4.5,
        };
        let f = S2c::NpcPortalTeleport(p).encode();
        assert_eq!(f[2], 100);
        let mut body = Vec::new();
        body.extend_from_slice(&42u16.to_le_bytes());
        body.extend_from_slice(&3i16.to_le_bytes());
        body.extend_from_slice(&900.5f32.to_le_bytes());
        body.extend_from_slice(&1200.25f32.to_le_bytes());
        body.extend_from_slice(&(-2.0f32).to_le_bytes());
        body.extend_from_slice(&4.5f32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::NpcPortalTeleport(d) => assert_eq!(d, p),
            other => panic!("expected 100, got {:?}", other.msg_id()),
        }
    }

    /// `101 UpdateTowerShieldStrengths` (`NetMessage.cs:1371-1376`): the
    /// four ushorts in written order Solar/Vortex/Nebula/Stardust. The
    /// codec carries the RAW values — the 0..100 clamp is the apply arm's
    /// job (`MessageBuffer.cs:3618-3649`).
    #[test]
    fn w4_g50_tower_shields_round_trips() {
        let t = TowerShields {
            solar: 100,
            vortex: 55,
            nebula: 0,
            stardust: 65535, // over the max — the apply clamps it
        };
        let f = S2c::TowerShields(t).encode();
        assert_eq!(f[2], 101);
        let mut body = Vec::new();
        for v in [100u16, 55, 0, 65535] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TowerShields(d) => assert_eq!(d, t),
            other => panic!("expected 101, got {:?}", other.msg_id()),
        }
    }

    /// `102 NebulaLevelupRequest` (`NetMessage.cs:1377-1382`): byte picker,
    /// ushort buffType, the picker-centre floats. Both directions — the
    /// host forces the id and relays (`MessageBuffer.cs:3654-3660`).
    #[test]
    fn w4_g50_nebula_levelup_request_round_trips() {
        let n = NebulaLevelupRequest {
            player_id: 2,
            buff_type: 173,
            x: 320.5,
            y: 640.25,
        };
        let mut body = vec![2u8];
        body.extend_from_slice(&173u16.to_le_bytes());
        body.extend_from_slice(&320.5f32.to_le_bytes());
        body.extend_from_slice(&640.25f32.to_le_bytes());
        let c = C2s::NebulaLevelupRequest(n).encode();
        assert_eq!(c[2], 102);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::NebulaLevelupRequest(d) => assert_eq!(d, n),
            other => panic!("expected 102, got {:?}", other.msg_id()),
        }
        let f = S2c::NebulaLevelupRequest(n).encode();
        assert_eq!(f[2], 102);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::NebulaLevelupRequest(d) => assert_eq!(d, n),
            other => panic!("expected 102, got {:?}", other.msg_id()),
        }
    }

    /// `103 MoonlordHorror` (`NetMessage.cs:1383-1386`): the two raw ints,
    /// MaxMoonLordCountdown first. Host→client only
    /// (`MessageBuffer.cs:3708-3714`).
    #[test]
    fn w4_g50_moonlord_countdown_round_trips() {
        let m = MoonlordCountdown {
            max: 3600,
            countdown: 1800,
        };
        let f = S2c::MoonlordCountdown(m).encode();
        assert_eq!(f[2], 103);
        let mut body = Vec::new();
        body.extend_from_slice(&3600i32.to_le_bytes());
        body.extend_from_slice(&1800i32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::MoonlordCountdown(d) => assert_eq!(d, m),
            other => panic!("expected 103, got {:?}", other.msg_id()),
        }
    }

    /// `104 ShopOverride` (`NetMessage.cs:1387-1394`): byte slot, short
    /// type, short stack (negative floored at 0), byte prefix, int value,
    /// the buyOnce BitsByte. Host→client only (`MessageBuffer.cs:3715-3735`).
    #[test]
    fn w4_g50_shop_override_round_trips() {
        let s = ShopOverride {
            slot: 3,
            kind: 88,
            stack: 5,
            prefix: 81,
            value: 40000,
            buy_once: true,
        };
        let f = S2c::ShopOverride(s).encode();
        assert_eq!(f[2], 104);
        let mut body = vec![3u8];
        body.extend_from_slice(&88i16.to_le_bytes());
        body.extend_from_slice(&5i16.to_le_bytes());
        body.push(81);
        body.extend_from_slice(&40000i32.to_le_bytes());
        body.push(1); // buyOnce bit 0
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ShopOverride(d) => assert_eq!(d, s),
            other => panic!("expected 104, got {:?}", other.msg_id()),
        }
        // The writer floors a negative stack at 0 (`:1390`).
        let neg = ShopOverride { stack: -3, ..s };
        let f = S2c::ShopOverride(neg).encode();
        assert_eq!(&f[5..7], &0i16.to_le_bytes());
    }

    /// `105 GemLockToggle` (`NetMessage.cs:1395-1399`): short x, short y,
    /// bool on. Client→host only (`MessageBuffer.cs:3736-3741`).
    #[test]
    fn w4_g50_gem_lock_toggle_round_trips() {
        let g = GemLockToggle {
            x: 120,
            y: 340,
            on: true,
        };
        let c = C2s::GemLockToggle(g).encode();
        assert_eq!(c[2], 105);
        let mut body = Vec::new();
        body.extend_from_slice(&120i16.to_le_bytes());
        body.extend_from_slice(&340i16.to_le_bytes());
        body.push(1);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::GemLockToggle(d) => assert_eq!(d, g),
            other => panic!("expected 105, got {:?}", other.msg_id()),
        }
    }

    /// `106 PoofOfSmoke` (`NetMessage.cs:1400-1402`): one uint — the
    /// HalfVector2 PackedValue (x low 16, y high 16). Host→client only
    /// (`MessageBuffer.cs:3742-3751`).
    #[test]
    fn w4_g50_poof_of_smoke_round_trips() {
        let packed = pack_half_vector2(123.5, -67.25);
        let p = PoofOfSmoke { packed };
        let f = S2c::PoofOfSmoke(p).encode();
        assert_eq!(f[2], 106);
        assert_eq!(&f[3..], &packed.to_le_bytes());
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PoofOfSmoke(d) => assert_eq!(d, p),
            other => panic!("expected 106, got {:?}", other.msg_id()),
        }
        // The half-float round trip keeps the packed bits exact.
        let (x, y) = unpack_half_vector2(packed);
        assert_eq!(pack_half_vector2(x, y), packed);
        // The known half-float pins: 1.0 = 0x3C00, -2.0 = 0xC000.
        assert_eq!(half_pack(1.0), 0x3C00);
        assert_eq!(half_pack(-2.0), 0xC000);
        assert_eq!(half_unpack(0x3C00), 1.0);
        assert_eq!(half_unpack(0xC000), -2.0);
    }

    /// `107 SmartTextMessage` (`NetMessage.cs:769-774`): the three RGB
    /// bytes, the NetworkText, the short width limit. Host→client only
    /// (`MessageBuffer.cs:3752-3757`).
    #[test]
    fn w4_g50_smart_text_round_trips() {
        let t = SmartText {
            color: Rgb(255, 200, 150),
            text: NetText("A wave is approaching...".into()),
            width_limit: 400,
        };
        let f = S2c::SmartText(t.clone()).encode();
        assert_eq!(f[2], 107);
        let mut body = vec![255u8, 200, 150];
        body.push(0); // NetworkText literal mode
        body.push(24); // the 7-bit length
        body.extend_from_slice(b"A wave is approaching...");
        body.extend_from_slice(&400i16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SmartText(d) => assert_eq!(d, t),
            other => panic!("expected 107, got {:?}", other.msg_id()),
        }
    }

    /// `108 WiredCannonShot` (`NetMessage.cs:1403-1411`): short damage,
    /// float knockback, short x/y/angle/ammo, byte owner. Host→client only,
    /// addressed to the owner (`MessageBuffer.cs:3758-3773`).
    #[test]
    fn w4_g50_wired_cannon_shot_round_trips() {
        let c = WiredCannonShot {
            damage: 300,
            knockback: 8.0,
            x: 120,
            y: 340,
            angle: 4,
            ammo: 4,
            owner: 2,
        };
        let f = S2c::WiredCannonShot(c).encode();
        assert_eq!(f[2], 108);
        let mut body = Vec::new();
        body.extend_from_slice(&300i16.to_le_bytes());
        body.extend_from_slice(&8.0f32.to_le_bytes());
        body.extend_from_slice(&120i16.to_le_bytes());
        body.extend_from_slice(&340i16.to_le_bytes());
        body.extend_from_slice(&4i16.to_le_bytes());
        body.extend_from_slice(&4i16.to_le_bytes());
        body.push(2);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::WiredCannonShot(d) => assert_eq!(d, c),
            other => panic!("expected 108, got {:?}", other.msg_id()),
        }
    }

    /// `109 MassWireOperation` (`NetMessage.cs:1412-1418`): four shorts
    /// (start x/y, end x/y) + the byte tool mode. Client→host only
    /// (`MessageBuffer.cs:3774-3788`).
    #[test]
    fn w4_g50_mass_wire_operation_round_trips() {
        let m = MassWireOperation {
            start_x: 10,
            start_y: 20,
            end_x: 30,
            end_y: 40,
            tool_mode: 0x21, // Red | Cutter
        };
        let c = C2s::MassWireOperation(m).encode();
        assert_eq!(c[2], 109);
        let mut body = Vec::new();
        for v in [10i16, 20, 30, 40] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        body.push(0x21);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::MassWireOperation(d) => assert_eq!(d, m),
            other => panic!("expected 109, got {:?}", other.msg_id()),
        }
    }

    /// `110 MassWireOperationPay` (`NetMessage.cs:1419-1423`): short item
    /// type, short count, byte player. Host→client only, addressed to the
    /// billed player (`MessageBuffer.cs:3789-3808`).
    #[test]
    fn w4_g50_mass_wire_operation_pay_round_trips() {
        let p = MassWireOperationPay {
            item_type: 530, // Wire
            count: 12,
            player: 2,
        };
        let f = S2c::MassWireOperationPay(p).encode();
        assert_eq!(f[2], 110);
        let mut body = Vec::new();
        body.extend_from_slice(&530i16.to_le_bytes());
        body.extend_from_slice(&12i16.to_le_bytes());
        body.push(2);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::MassWireOperationPay(d) => assert_eq!(d, p),
            other => panic!("expected 110, got {:?}", other.msg_id()),
        }
    }

    /// `111 ToggleParty` (`BirthdayParty.cs:154`): no body — a bare 3-byte
    /// frame. Client→host only (`MessageBuffer.cs:3809-3814`).
    #[test]
    fn w4_g50_toggle_party_is_a_bare_frame() {
        let c = C2s::ToggleParty.encode();
        assert_eq!(c[2], 111);
        assert_eq!(c.len(), 3, "id 111 has no body");
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ToggleParty => {}
            other => panic!("expected 111, got {:?}", other.msg_id()),
        }
    }

    /// `112 SpecialFX` (`NetMessage.cs:1424-1431`): byte effect, int x/y,
    /// byte b4, short s5, byte b6. Host→client only
    /// (`MessageBuffer.cs:3815-3840`).
    #[test]
    fn w4_g50_special_fx_round_trips() {
        let f = SpecialFx {
            effect: 1,
            x: 120,
            y: 340,
            b4: 5,
            s5: 1278,
            b6: 1,
        };
        let frame = S2c::SpecialFx(f).encode();
        assert_eq!(frame[2], 112);
        let mut body = vec![1u8];
        body.extend_from_slice(&120i32.to_le_bytes());
        body.extend_from_slice(&340i32.to_le_bytes());
        body.push(5);
        body.extend_from_slice(&1278i16.to_le_bytes());
        body.push(1);
        assert_eq!(&frame[3..], &body);
        match S2c::decode(frame[2], &mut NetReader::new(&frame[3..])).expect("decode") {
            S2c::SpecialFx(d) => assert_eq!(d, f),
            other => panic!("expected 112, got {:?}", other.msg_id()),
        }
    }

    /// `113 CrystalInvasionStart` (`NetMessage.cs:1432-1435`): short x,
    /// short y. Client→host only (`MessageBuffer.cs:3841-3854`).
    #[test]
    fn w4_g50_crystal_invasion_start_round_trips() {
        let c = CrystalInvasionStart { x: 120, y: 340 };
        let frame = C2s::CrystalInvasionStart(c).encode();
        assert_eq!(frame[2], 113);
        let mut body = Vec::new();
        body.extend_from_slice(&120i16.to_le_bytes());
        body.extend_from_slice(&340i16.to_le_bytes());
        assert_eq!(&frame[3..], &body);
        match C2s::decode(frame[2], &mut NetReader::new(&frame[3..])).expect("decode") {
            C2s::CrystalInvasionStart(d) => assert_eq!(d, c),
            other => panic!("expected 113, got {:?}", other.msg_id()),
        }
    }

    /// `114 CrystalInvasionWipeAllTheThingsss` (`DD2Event.cs:636`): no body
    /// — a bare 3-byte frame. Host→client only (`MessageBuffer.cs:3855-3860`).
    #[test]
    fn w4_g50_crystal_invasion_wipe_is_a_bare_frame() {
        let f = S2c::CrystalInvasionWipe.encode();
        assert_eq!(f[2], 114);
        assert_eq!(f.len(), 3, "id 114 has no body");
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::CrystalInvasionWipe => {}
            other => panic!("expected 114, got {:?}", other.msg_id()),
        }
    }

    /// `116 CrystalInvasionSendWaitTime` (`NetMessage.cs:1436-1438`): one
    /// raw int. Host→client only (`MessageBuffer.cs:3861-3866`).
    #[test]
    fn w4_g50_crystal_invasion_wait_round_trips() {
        let t = CrystalInvasionWait { time_left: 1800 };
        let f = S2c::CrystalInvasionWait(t).encode();
        assert_eq!(f[2], 116);
        assert_eq!(&f[3..], &1800i32.to_le_bytes());
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::CrystalInvasionWait(d) => assert_eq!(d, t),
            other => panic!("expected 116, got {:?}", other.msg_id()),
        }
    }

    /// `117 PlayerHurtV2` (`NetMessage.cs:1439-1446`): byte player, the
    /// death reason, short damage, byte (dir+1), the crit/pvp BitsByte,
    /// sbyte cooldown. Both directions (`MessageBuffer.cs:3867-3886`).
    #[test]
    fn w4_g50_player_hurt_v2_round_trips() {
        let h = PlayerHurtV2 {
            player_id: 2,
            reason: PlayerDeathReason {
                source_npc_index: 5,
                ..Default::default()
            },
            damage: 35,
            hit_direction: -1,
            crit: true,
            pvp: false,
            cooldown_counter: 0,
        };
        let c = C2s::PlayerHurtV2(h.clone()).encode();
        assert_eq!(c[2], 117);
        // byte id, the reason flag byte (bit 1 = npc index), the short 5,
        // short damage, byte (dir+1)=0, BitsByte crit=1, sbyte cooldown.
        let mut body = vec![2u8, 0x02];
        body.extend_from_slice(&5i16.to_le_bytes());
        body.extend_from_slice(&35i16.to_le_bytes());
        body.push(0); // -1 + 1
        body.push(0x01); // crit bit 0
        body.push(0); // cooldown sbyte
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerHurtV2(d) => assert_eq!(d, h),
            other => panic!("expected 117, got {:?}", other.msg_id()),
        }
        let f = S2c::PlayerHurtV2(h.clone()).encode();
        assert_eq!(f[2], 117);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerHurtV2(d) => assert_eq!(d, h),
            other => panic!("expected 117, got {:?}", other.msg_id()),
        }
    }

    /// `118 PlayerDeathV2` (`NetMessage.cs:1447-1453`): byte player, the
    /// death reason, short damage, byte (dir+1), the pvp BitsByte — no crit
    /// bit, no cooldown sbyte. Both directions (`MessageBuffer.cs:3887-3904`).
    #[test]
    fn w4_g50_player_death_v2_round_trips() {
        let d = PlayerDeathV2 {
            player_id: 3,
            reason: PlayerDeathReason {
                source_other_index: 255, // LegacyDefault
                ..Default::default()
            },
            damage: 999,
            hit_direction: 1,
            pvp: true,
        };
        let c = C2s::PlayerDeathV2(d.clone()).encode();
        assert_eq!(c[2], 118);
        // byte id, reason flag (bit 3 = other index), byte 255, short damage,
        // byte (dir+1)=2, BitsByte pvp=1.
        let mut body = vec![3u8, 0x08, 255];
        body.extend_from_slice(&999i16.to_le_bytes());
        body.push(2); // 1 + 1
        body.push(0x01); // pvp bit 0
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::PlayerDeathV2(back) => assert_eq!(back, d),
            other => panic!("expected 118, got {:?}", other.msg_id()),
        }
        let f = S2c::PlayerDeathV2(d.clone()).encode();
        assert_eq!(f[2], 118);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::PlayerDeathV2(back) => assert_eq!(back, d),
            other => panic!("expected 118, got {:?}", other.msg_id()),
        }
    }

    /// `120 Emoji` (`NetMessage.cs:1454-1457`): byte player, byte emote id.
    /// Client→host only (`MessageBuffer.cs:3905-3919`); the EmoteID.Count
    /// gate is the host arm's, not the codec's.
    #[test]
    fn w4_g50_emoji_round_trips() {
        let e = Emoji {
            player_id: 2,
            emote_id: 17,
        };
        let c = C2s::Emoji(e).encode();
        assert_eq!(c[2], 120);
        assert_eq!(&c[3..], &[2u8, 17]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::Emoji(d) => assert_eq!(d, e),
            other => panic!("expected 120, got {:?}", other.msg_id()),
        }
        // An out-of-range id still round-trips (the gate is the host's).
        let over = Emoji {
            player_id: 0,
            emote_id: 255,
        };
        let c = C2s::Emoji(over).encode();
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::Emoji(d) => assert_eq!(d, over),
            other => panic!("expected 120, got {:?}", other.msg_id()),
        }
    }

    /// `83 Unused83` (`MessageID.cs:179-180`, `[Old]`): no body — a bare
    /// 3-byte frame; the reader's ignore group (`MessageBuffer.cs:4483-4490`).
    #[test]
    fn w4_g50_unused_83_is_a_bare_no_op_frame() {
        let f = S2c::Unused83.encode();
        assert_eq!(f[2], 83);
        assert_eq!(f.len(), 3, "id 83 has no body");
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::Unused83 => {}
            other => panic!("expected 83, got {:?}", other.msg_id()),
        }
    }

    /// `93 SocialHandshake` (`MessageID.cs:200`): no body — a bare 3-byte
    /// frame; the reader's ignore group (`MessageBuffer.cs:4483-4490`).
    #[test]
    fn w4_g50_unused_93_is_a_bare_no_op_frame() {
        let f = S2c::Unused93.encode();
        assert_eq!(f[2], 93);
        assert_eq!(f.len(), 3, "id 93 has no body");
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::Unused93 => {}
            other => panic!("expected 93, got {:?}", other.msg_id()),
        }
    }

    /// `85 QuickStackChests` (`NetMessage.cs:1205-1215`): the client leg is
    /// int count + short slot ids + byte smartStack; the host leg is int
    /// count + ushort chest indices. Both round-trip byte-verbatim.
    #[test]
    fn w4_g50_quick_stack_chests_round_trips() {
        // The client ask: two slots, smartStack on.
        let q = QuickStackRequest {
            slots: vec![10, 55],
            smart_stack: true,
        };
        let c = C2s::QuickStackRequest(q.clone()).encode();
        assert_eq!(c[2], 85);
        let mut body = Vec::new();
        body.extend_from_slice(&2i32.to_le_bytes());
        body.extend_from_slice(&10i16.to_le_bytes());
        body.extend_from_slice(&55i16.to_le_bytes());
        body.push(1);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::QuickStackRequest(d) => assert_eq!(d, q),
            other => panic!("expected 85, got {:?}", other.msg_id()),
        }
        // The host answer: two blocked chests.
        let b = QuickStackBlocked {
            chests: vec![3, 7],
        };
        let f = S2c::QuickStackBlocked(b.clone()).encode();
        assert_eq!(f[2], 85);
        let mut body = Vec::new();
        body.extend_from_slice(&2i32.to_le_bytes());
        body.extend_from_slice(&3u16.to_le_bytes());
        body.extend_from_slice(&7u16.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::QuickStackBlocked(d) => assert_eq!(d, b),
            other => panic!("expected 85, got {:?}", other.msg_id()),
        }
        // An empty list round-trips (count 0, no tail).
        let empty = QuickStackBlocked { chests: vec![] };
        let f = S2c::QuickStackBlocked(empty.clone()).encode();
        assert_eq!(f.len(), 3 + 4);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::QuickStackBlocked(d) => assert_eq!(d, empty),
            other => panic!("expected 85, got {:?}", other.msg_id()),
        }
    }

    /// `94 DevCommands` (`NetMessage.cs:1327-1332`): string command, int
    /// number, float number2, float number3. Both directions — the reader
    /// consumes all four fields before the enableDebugCommands gate
    /// (`MessageBuffer.cs:3465-3472`).
    #[test]
    fn w4_g50_dev_commands_round_trips() {
        let d = DevCommands {
            command: "/setserverping".into(),
            number: 0,
            number2: 120.0,
            number3: 0.0,
        };
        let c = C2s::DevCommands(d.clone()).encode();
        assert_eq!(c[2], 94);
        let mut body = Vec::new();
        body.push(14); // the 7-bit length of "/setserverping"
        body.extend_from_slice(b"/setserverping");
        body.extend_from_slice(&0i32.to_le_bytes());
        body.extend_from_slice(&120.0f32.to_le_bytes());
        body.extend_from_slice(&0.0f32.to_le_bytes());
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::DevCommands(back) => assert_eq!(back, d),
            other => panic!("expected 94, got {:?}", other.msg_id()),
        }
        let f = S2c::DevCommands(d.clone()).encode();
        assert_eq!(f[2], 94);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::DevCommands(back) => assert_eq!(back, d),
            other => panic!("expected 94, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `125 SyncTilePicking` body byte-pins the writer order
    /// (`NetMessage.cs:1509-1514`: byte player, short x, short y, byte
    /// damage) and round-trips both directions.
    #[test]
    fn w4_g51_tile_picking_round_trips() {
        let p = TilePicking {
            player_id: 3,
            x: 120,
            y: -45,
            damage: 37,
        };
        let mut body = vec![3u8];
        body.extend_from_slice(&120i16.to_le_bytes());
        body.extend_from_slice(&(-45i16).to_le_bytes());
        body.push(37);
        let c = C2s::TilePicking(p).encode();
        assert_eq!(c[2], 125);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::TilePicking(d) => assert_eq!(d, p),
            other => panic!("expected 125, got {:?}", other.msg_id()),
        }
        let f = S2c::TilePicking(p).encode();
        assert_eq!(f[2], 125);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TilePicking(d) => assert_eq!(d, p),
            other => panic!("expected 125, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `127 RemoveRevengeMarker` body is one raw (int) UniqueID
    /// (`NetMessage.cs:1518-1520`); S2c-only.
    #[test]
    fn w4_g51_remove_revenge_marker_round_trips() {
        let f = S2c::RemoveRevengeMarker(RemoveRevengeMarker { unique_id: -7 }).encode();
        assert_eq!(f[2], 127);
        assert_eq!(&f[3..], &(-7i32).to_le_bytes());
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::RemoveRevengeMarker(d) => assert_eq!(d.unique_id, -7),
            other => panic!("expected 127, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `128 LandGolfBallInCup` body byte-pins the writer's field
    /// order (`NetMessage.cs:1521-1527`: owner, x, y, hits, type — NOT the
    /// argument order) and round-trips both directions.
    #[test]
    fn w4_g51_golf_ball_in_cup_round_trips() {
        let g = GolfBallInCup {
            owner: 2,
            x: 300,
            y: 400,
            hits: 3,
            ball_type: 706,
        };
        let mut body = vec![2u8];
        body.extend_from_slice(&300u16.to_le_bytes());
        body.extend_from_slice(&400u16.to_le_bytes());
        body.extend_from_slice(&3u16.to_le_bytes());
        body.extend_from_slice(&706u16.to_le_bytes());
        let c = C2s::GolfBallInCup(g).encode();
        assert_eq!(c[2], 128);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::GolfBallInCup(d) => assert_eq!(d, g),
            other => panic!("expected 128, got {:?}", other.msg_id()),
        }
        let f = S2c::GolfBallInCup(g).encode();
        assert_eq!(f[2], 128);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::GolfBallInCup(d) => assert_eq!(d, g),
            other => panic!("expected 128, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `130 FishOutNPC` body byte-pins the tile-coordinate order
    /// (`NetMessage.cs:1528-1532`: ushort x, ushort y, short type); C2s-only.
    #[test]
    fn w4_g51_fish_out_npc_round_trips() {
        let f = FishOutNpc {
            tile_x: 100,
            tile_y: 200,
            npc_type: 682,
        };
        let mut body = Vec::new();
        body.extend_from_slice(&100u16.to_le_bytes());
        body.extend_from_slice(&200u16.to_le_bytes());
        body.extend_from_slice(&682i16.to_le_bytes());
        let c = C2s::FishOutNpc(f).encode();
        assert_eq!(c[2], 130);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::FishOutNpc(d) => assert_eq!(d, f),
            other => panic!("expected 130, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `131 TamperWithNPC` body byte-pins the kind-1 tail
    /// (`NetMessage.cs:1533-1541`: ushort npc, byte kind, then int time +
    /// short fromWho ONLY when kind==1); S2c-only. The bare kind-0 body
    /// carries no tail.
    #[test]
    fn w4_g51_tamper_with_npc_round_trips() {
        let t = TamperWithNpc {
            npc: 7,
            kind: 1,
            immune: Some((20, -1)),
        };
        let mut body = Vec::new();
        body.extend_from_slice(&7u16.to_le_bytes());
        body.push(1);
        body.extend_from_slice(&20i32.to_le_bytes());
        body.extend_from_slice(&(-1i16).to_le_bytes());
        let f = S2c::TamperWithNpc(t).encode();
        assert_eq!(f[2], 131);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::TamperWithNpc(d) => assert_eq!(d, t),
            other => panic!("expected 131, got {:?}", other.msg_id()),
        }
        // The bare kind-0 body: no tail.
        let bare = TamperWithNpc {
            npc: 7,
            kind: 0,
            immune: None,
        };
        let g = S2c::TamperWithNpc(bare).encode();
        assert_eq!(&g[3..], &[7, 0, 0]);
        match S2c::decode(g[2], &mut NetReader::new(&g[3..])).expect("decode") {
            S2c::TamperWithNpc(d) => assert_eq!(d, bare),
            other => panic!("expected 131, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `134 UpdatePlayerLuckFactors` body byte-pins the eight
    /// luck fields in writer order (`NetMessage.cs:1555-1567`) and
    /// round-trips both directions.
    #[test]
    fn w4_g51_update_player_luck_factors_round_trips() {
        let l = UpdatePlayerLuckFactors {
            player_id: 2,
            ladybug_luck_time_left: 3600,
            torch_luck: 0.25,
            luck_potion: 3,
            has_garden_gnome_nearby: true,
            broken_mirror_bad_luck: false,
            equipment_based_luck_bonus: 0.05,
            coin_luck: 1234.5,
            kite_luck_level: 2,
        };
        let mut body = vec![2u8];
        body.extend_from_slice(&3600i32.to_le_bytes());
        body.extend_from_slice(&0.25f32.to_le_bytes());
        body.push(3);
        body.push(1); // gnome true
        body.push(0); // mirror false
        body.extend_from_slice(&0.05f32.to_le_bytes());
        body.extend_from_slice(&1234.5f32.to_le_bytes());
        body.push(2);
        let c = C2s::UpdatePlayerLuckFactors(l).encode();
        assert_eq!(c[2], 134);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::UpdatePlayerLuckFactors(d) => assert_eq!(d, l),
            other => panic!("expected 134, got {:?}", other.msg_id()),
        }
        let f = S2c::UpdatePlayerLuckFactors(l).encode();
        assert_eq!(f[2], 134);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::UpdatePlayerLuckFactors(d) => assert_eq!(d, l),
            other => panic!("expected 134, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `136 SyncCavernMonsterType` body byte-pins the 2×3 grid
    /// row-major (`NetMessage.cs:1572-1582`: six ushorts, outer i<2 inner
    /// j<3); S2c-only.
    #[test]
    fn w4_g51_cavern_monster_types_round_trips() {
        let c = CavernMonsterTypes {
            types: [[1, 2, 3], [4, 5, 6]],
        };
        let mut body = Vec::new();
        for row in &c.types {
            for ty in row {
                body.extend_from_slice(&ty.to_le_bytes());
            }
        }
        let f = S2c::CavernMonsterTypes(c).encode();
        assert_eq!(f[2], 136);
        assert_eq!(&f[3..], &body);
        assert_eq!(body.len(), 12, "six ushorts");
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::CavernMonsterTypes(d) => assert_eq!(d, c),
            other => panic!("expected 136, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `137 RequestNPCBuffRemoval` body byte-pins the slot+buff
    /// pair (`NetMessage.cs:1583-1586`: short npc, ushort buff); C2s-only.
    #[test]
    fn w4_g51_npc_buff_removal_request_round_trips() {
        let q = NpcBuffRemovalRequest {
            npc: 7,
            buff_type: 20,
        };
        let mut body = Vec::new();
        body.extend_from_slice(&7i16.to_le_bytes());
        body.extend_from_slice(&20u16.to_le_bytes());
        let c = C2s::NpcBuffRemovalRequest(q).encode();
        assert_eq!(c[2], 137);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::NpcBuffRemovalRequest(d) => assert_eq!(d, q),
            other => panic!("expected 137, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51 f21 cross-check: the bodyless `129 FinishedConnecting` (S2c) and
    /// `138 ClientSyncedInventory` (C2s) are bare 3-byte frames; the deprecated
    /// `145`/`148` are census constants with no variant.
    #[test]
    fn w4_g51_f21_bodyless_and_deprecated_ids() {
        let f = S2c::FinishedConnecting.encode();
        assert_eq!(f.len(), 3);
        assert_eq!(f[2], 129);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::FinishedConnecting => {}
            other => panic!("expected 129, got {:?}", other.msg_id()),
        }
        let c = C2s::ClientSyncedInventory.encode();
        assert_eq!(c.len(), 3);
        assert_eq!(c[2], 138);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ClientSyncedInventory => {}
            other => panic!("expected 138, got {:?}", other.msg_id()),
        }
        assert_eq!(id::SHIMMER_ITEMS_DEPRECATED, 145);
        assert_eq!(id::ITEM_ENEMY_LOCK_DEPRECATED, 148);
    }

    /// W4-g51 f21 cross-check: `139 CountsAsHost` byte-pins the player+bool
    /// pair (`NetMessage.cs:1587-1590`) and round-trips S2c.
    #[test]
    fn w4_g51_f21_counts_as_host_round_trips() {
        for flag in [false, true] {
            let c = CountsAsHost { player_id: 3, counts_as_host: flag };
            let f = S2c::CountsAsHost(c).encode();
            assert_eq!(f[2], 139);
            assert_eq!(&f[3..], &[3u8, u8::from(flag)]);
            match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
                S2c::CountsAsHost(d) => assert_eq!(d, c),
                other => panic!("expected 139, got {:?}", other.msg_id()),
            }
        }
    }

    /// W4-g51 f21 cross-check: `146 ShimmerAction`'s three shapes — kind 0
    /// Vector2, kind 1 Vector2+int, unknown kind no tail
    /// (`NetMessage.cs:1611-1623`).
    #[test]
    fn w4_g51_f21_shimmer_action_round_trips() {
        let fx = ShimmerAction::Effect { x: 100.5, y: -20.25 };
        let f = S2c::ShimmerAction(fx).encode();
        assert_eq!(f[2], 146);
        assert_eq!(f[3], 0);
        assert_eq!(f.len(), 3 + 1 + 8);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ShimmerAction(d) => assert_eq!(d, fx),
            other => panic!("expected 146, got {:?}", other.msg_id()),
        }
        let coin = ShimmerAction::CoinLuck { x: 1.0, y: 2.0, amount: 10000 };
        let f = S2c::ShimmerAction(coin).encode();
        assert_eq!(f[3], 1);
        assert_eq!(f.len(), 3 + 1 + 8 + 4);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ShimmerAction(d) => assert_eq!(d, coin),
            other => panic!("expected 146, got {:?}", other.msg_id()),
        }
        let f = S2c::ShimmerAction(ShimmerAction::Unknown(7)).encode();
        assert_eq!(f.len(), 3 + 1, "unknown kind: no tail");
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ShimmerAction(d) => assert_eq!(d, ShimmerAction::Unknown(7)),
            other => panic!("expected 146, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51 f21 cross-check: `152 ItemUseSound` is one player byte, both
    /// directions (`NetMessage.cs:1643-1645`).
    #[test]
    fn w4_g51_f21_item_use_sound_round_trips() {
        let c = C2s::ItemUseSound { player_id: 5 }.encode();
        assert_eq!(c[2], 152);
        assert_eq!(&c[3..], &[5u8]);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::ItemUseSound { player_id } => assert_eq!(player_id, 5),
            other => panic!("expected 152, got {:?}", other.msg_id()),
        }
        let f = S2c::ItemUseSound { player_id: 5 }.encode();
        assert_eq!(f[2], 152);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemUseSound { player_id } => assert_eq!(player_id, 5),
            other => panic!("expected 152, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51 f21 cross-check: `160 ItemPosition` byte-pins slot+Vector2 (no
    /// velocity, `NetMessage.cs:1666-1672`) and round-trips S2c.
    #[test]
    fn w4_g51_f21_item_position_round_trips() {
        let p = ItemPosition { slot: 399, x: 1234.5, y: -67.25 };
        let mut body = Vec::new();
        body.extend_from_slice(&399i16.to_le_bytes());
        body.extend_from_slice(&1234.5f32.to_le_bytes());
        body.extend_from_slice(&(-67.25f32).to_le_bytes());
        let f = S2c::ItemPosition(p).encode();
        assert_eq!(f[2], 160);
        assert_eq!(&f[3..], &body);
        assert_eq!(f.len(), 3 + 2 + 8, "no velocity");
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemPosition(d) => assert_eq!(d, p),
            other => panic!("expected 160, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51 f21 cross-check: `161 HostToken` is a plain string, C2s
    /// (`NetMessage.cs:1673-1675`); the empty token round-trips too.
    #[test]
    fn w4_g51_f21_host_token_round_trips() {
        for token in ["secret-host-token".to_string(), String::new()] {
            let c = C2s::HostToken(token.clone()).encode();
            assert_eq!(c[2], 161);
            match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
                C2s::HostToken(d) => assert_eq!(d, token),
                other => panic!("expected 161, got {:?}", other.msg_id()),
            }
        }
    }

    // ---- W4-g51 FINAL: the 8 BIG ids' codec witnesses (f21 lifts) ----
    // WRITTEN NOT RUN (code-only lane).

    /// The TryPlacing quadruplet (89/123/133/149) shares ONE body — f21's
    /// byte-equality pin: the four ids encode the SAME bytes for the same
    /// `TryPlacingItem` (`NetMessage.cs:1297-1306`/`:1479-1488`/`:1545-1554`/
    /// `:1629-1638`). The 89/123/133/149 readers are byte-identical too.
    #[test]
    fn w4_g51_try_placing_quadruplet_is_byte_equal() {
        let p = TryPlacingItem {
            x: 120,
            y: 240,
            kind: 29,
            prefix: 81,
            stack: 1,
        };
        let mut body = Vec::new();
        body.extend_from_slice(&120i16.to_le_bytes());
        body.extend_from_slice(&240i16.to_le_bytes());
        body.extend_from_slice(&29i16.to_le_bytes());
        body.push(81);
        body.extend_from_slice(&1i16.to_le_bytes());
        let frames = [
            C2s::ItemFrameTryPlacing(p).encode(),
            C2s::WeaponsRackTryPlacing(p).encode(),
            C2s::FoodPlatterTryPlacing(p).encode(),
            C2s::DeadCellsJarTryPlacing(p).encode(),
        ];
        let ids = [89u8, 123, 133, 149];
        for (f, id) in frames.iter().zip(ids) {
            assert_eq!(f[2], id);
            assert_eq!(&f[3..], &body, "id {id} shares the 89 body");
        }
        // Each reader decodes its own id back to the same body.
        assert!(matches!(
            C2s::decode(123, &mut NetReader::new(&body)).unwrap(),
            C2s::WeaponsRackTryPlacing(d) if d == p
        ));
        assert!(matches!(
            C2s::decode(133, &mut NetReader::new(&body)).unwrap(),
            C2s::FoodPlatterTryPlacing(d) if d == p
        ));
        assert!(matches!(
            C2s::decode(149, &mut NetReader::new(&body)).unwrap(),
            C2s::DeadCellsJarTryPlacing(d) if d == p
        ));
    }

    /// `121 TEDisplayDollDataSync`: the command-keyed tail — command 2 is
    /// the pose byte, commands 0/1/3 are the STACK-BEFORE-PREFIX slot triple
    /// (`TEDisplayDoll.cs:758-764`). The dummy fallback is byte-width-
    /// identical, so it decodes as the real shape (`DollSyncData::Dummy` is
    /// writer-side only). Both directions.
    #[test]
    fn w4_g51_display_doll_sync_round_trips() {
        // command 2 (pose): header + one pose byte.
        let pose = DisplayDollSync {
            player_id: 3,
            entity_id: 77,
            item_index: 0,
            command: 2,
            data: DollSyncData::Pose(5),
        };
        let f = C2s::DisplayDollSync(pose).encode();
        assert_eq!(f[2], 121);
        // byte player + int id + byte idx + byte cmd + byte pose = 8 bytes.
        assert_eq!(f.len(), 3 + 1 + 4 + 1 + 1 + 1);
        assert_eq!(f[3], 3);
        assert_eq!(&f[4..8], &77i32.to_le_bytes());
        assert_eq!(f[10], 2);
        assert_eq!(f[11], 5);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            C2s::DisplayDollSync(d) => assert_eq!(d, pose),
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
        // command 0 (equip slot): the WriteItem order type+stack+prefix.
        let item = DisplayDollSync {
            player_id: 3,
            entity_id: 77,
            item_index: 4,
            command: 0,
            data: DollSyncData::Item(TeItem {
                kind: 100,
                prefix: 81,
                stack: 1,
            }),
        };
        let f = S2c::DisplayDollSync(item).encode();
        assert_eq!(f[2], 121);
        // header 7 + ushort type + ushort stack + byte prefix = 12 bytes.
        assert_eq!(f.len(), 3 + 7 + 5);
        assert_eq!(&f[10..12], &100u16.to_le_bytes());
        assert_eq!(&f[12..14], &1u16.to_le_bytes());
        assert_eq!(f[14], 81);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            S2c::DisplayDollSync(d) => assert_eq!(d, item),
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
        // The dummy fallback (command 0): int 0 + byte 0 — decodes as an
        // air slot (the byte widths match WriteItem).
        let dummy = DisplayDollSync {
            player_id: 0,
            entity_id: -1,
            item_index: 0,
            command: 0,
            data: DollSyncData::Dummy,
        };
        let f = C2s::DisplayDollSync(dummy).encode();
        assert_eq!(f.len(), 3 + 7 + 5);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            C2s::DisplayDollSync(d) => {
                assert!(matches!(d.data, DollSyncData::Item(it) if it.is_air()));
            }
            other => panic!("expected 121, got {:?}", other.msg_id()),
        }
    }

    /// `122 RequestTileEntityInteraction`: int entityId + byte player; -1
    /// clears. Both directions (`NetMessage.cs:1475-1478`).
    #[test]
    fn w4_g51_te_interaction_round_trips() {
        for entity_id in [42i32, -1] {
            let t = TileEntityInteraction {
                entity_id,
                player_id: 2,
            };
            let mut body = Vec::new();
            body.extend_from_slice(&entity_id.to_le_bytes());
            body.push(2);
            let c = C2s::TileEntityInteraction(t).encode();
            assert_eq!(c[2], 122);
            assert_eq!(&c[3..], &body);
            match C2s::decode(c[2], &mut NetReader::new(&c[3..])).unwrap() {
                C2s::TileEntityInteraction(d) => assert_eq!(d, t),
                other => panic!("expected 122, got {:?}", other.msg_id()),
            }
            let f = S2c::TileEntityInteraction(t).encode();
            assert_eq!(f[2], 122);
            assert_eq!(&f[3..], &body);
            match S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
                S2c::TileEntityInteraction(d) => assert_eq!(d, t),
                other => panic!("expected 122, got {:?}", other.msg_id()),
            }
        }
    }

    /// `124 TEHatRackItemSync`: the +2 dye fold/unfold + the STACK-BEFORE-
    /// PREFIX WriteItem tail (`NetMessage.cs:1489-1508`,
    /// `MessageBuffer.cs:4005-4009`). Both directions.
    #[test]
    fn w4_g51_hat_rack_item_sync_round_trips_and_folds() {
        // wire slot 3 = dye slot 1 (the +2 fold).
        let h = HatRackItemSync {
            player_id: 1,
            entity_id: 9,
            wire_slot: 3,
            item: TeItem {
                kind: 3454,
                prefix: 0,
                stack: 1,
            },
        };
        assert!(h.is_dye());
        assert_eq!(h.slot_index(), 1);
        let f = C2s::HatRackItemSync(h).encode();
        assert_eq!(f[2], 124);
        // byte player + int id + byte slot + ushort type + ushort stack +
        // byte prefix = 1+4+1+5 = 11 bytes.
        assert_eq!(f.len(), 3 + 11);
        assert_eq!(f[3], 1);
        assert_eq!(&f[4..8], &9i32.to_le_bytes());
        assert_eq!(f[8], 3);
        assert_eq!(&f[9..11], &3454u16.to_le_bytes());
        assert_eq!(&f[11..13], &1u16.to_le_bytes());
        assert_eq!(f[13], 0);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            C2s::HatRackItemSync(d) => assert_eq!(d, h),
            other => panic!("expected 124, got {:?}", other.msg_id()),
        }
        // wire slot 0 = item slot 0 (no fold).
        let item = HatRackItemSync {
            wire_slot: 0,
            ..h
        };
        assert!(!item.is_dye());
        assert_eq!(item.slot_index(), 0);
        let f = S2c::HatRackItemSync(item).encode();
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            S2c::HatRackItemSync(d) => assert_eq!(d, item),
            other => panic!("expected 124, got {:?}", other.msg_id()),
        }
    }

    /// `144 RequestQuestEffect` is bodyless (NO writer case, the host reads
    /// nothing — `MessageBuffer.cs:4304-4309`). C2s-only.
    #[test]
    fn w4_g51_request_quest_effect_is_bodyless() {
        let f = C2s::RequestQuestEffect.encode();
        assert_eq!(f.len(), 3, "the bare 3-byte frame");
        assert_eq!(f[2], 144);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            C2s::RequestQuestEffect => {}
            other => panic!("expected 144, got {:?}", other.msg_id()),
        }
    }

    /// `156 TELeashedEntityAnchorPlaceItem`: three shorts x/y/itemType
    /// (`NetMessage.cs:1654-1658`). C2s-only.
    #[test]
    fn w4_g51_leashed_anchor_place_item_round_trips() {
        let p = LeashedAnchorPlaceItem {
            x: 50,
            y: 60,
            item_type: 3034,
        };
        let mut body = Vec::new();
        body.extend_from_slice(&50i16.to_le_bytes());
        body.extend_from_slice(&60i16.to_le_bytes());
        body.extend_from_slice(&3034i16.to_le_bytes());
        let f = C2s::LeashedAnchorPlaceItem(p).encode();
        assert_eq!(f[2], 156);
        assert_eq!(&f[3..], &body);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            C2s::LeashedAnchorPlaceItem(d) => assert_eq!(d, p),
            other => panic!("expected 156, got {:?}", other.msg_id()),
        }
    }

    /// `86 TileEntitySharing`: int id + bool present + the networkSend
    /// record; the absent shape carries no record (`NetMessage.cs:1216-1225`,
    /// `MessageBuffer.cs:3278-3291`). S2c-only.
    #[test]
    fn w4_g51_tile_entity_sharing_round_trips() {
        // A live item-frame record: type 1 + x + y + the raw triple.
        let e = tr_core::tile_entities::TileEntity {
            id: 0,
            x: 10,
            y: 20,
            kind: tr_core::tile_entities::TeKind::ItemFrame,
            data: tr_core::tile_entities::TeData::Item(Some(tr_core::tile_entities::TeItem {
                kind: 29,
                prefix: 81,
                stack: 1,
            })),
        };
        let mut w = tr_core::binio::Writer::new();
        e.write_net(&mut w);
        let record = w.into_inner();
        let t = TileEntitySharing {
            id: 5,
            record: record.clone(),
        };
        let f = S2c::TileEntitySharing(t.clone()).encode();
        assert_eq!(f[2], 86);
        assert_eq!(&f[3..7], &5i32.to_le_bytes());
        assert_eq!(f[7], 1); // present
        assert_eq!(&f[8..], &record);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            S2c::TileEntitySharing(d) => assert_eq!(d, t),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
        // The removal shape: present false, no record.
        let gone = TileEntitySharing {
            id: 5,
            record: Vec::new(),
        };
        let f = S2c::TileEntitySharing(gone.clone()).encode();
        assert_eq!(f.len(), 3 + 4 + 1);
        assert_eq!(f[7], 0);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            S2c::TileEntitySharing(d) => assert_eq!(d, gone),
            other => panic!("expected 86, got {:?}", other.msg_id()),
        }
    }

    /// `87 TileEntityPlacement`: short x + short y + byte type
    /// (`NetMessage.cs:1227-1231`). C2s-only.
    #[test]
    fn w4_g51_tile_entity_placement_round_trips() {
        let p = TileEntityPlacement {
            x: 10,
            y: 20,
            kind: 1,
        };
        let f = C2s::TileEntityPlacement(p).encode();
        assert_eq!(f[2], 87);
        assert_eq!(&f[3..5], &10i16.to_le_bytes());
        assert_eq!(&f[5..7], &20i16.to_le_bytes());
        assert_eq!(f[7], 1);
        match C2s::decode(f[2], &mut NetReader::new(&f[3..])).unwrap() {
            C2s::TileEntityPlacement(d) => assert_eq!(d, p),
            other => panic!("expected 87, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `141 LucyTheAxeMessage` body byte-pins the swapped-pair
    /// trap (`NetMessage.cs:1595-1602`: source, variation, the velocity
    /// floats BEFORE the position ints) and round-trips both directions.
    #[test]
    fn w4_g51_lucy_popup_round_trips() {
        let l = LucyPopup {
            source: 2,
            variation: 5,
            velocity_x: 1.5,
            velocity_y: -2.0,
            x: 100,
            y: 200,
        };
        let mut body = vec![2u8, 5];
        body.extend_from_slice(&1.5f32.to_le_bytes());
        body.extend_from_slice(&(-2.0f32).to_le_bytes());
        body.extend_from_slice(&100i32.to_le_bytes());
        body.extend_from_slice(&200i32.to_le_bytes());
        let c = C2s::LucyPopup(l).encode();
        assert_eq!(c[2], 141);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::LucyPopup(d) => assert_eq!(d, l),
            other => panic!("expected 141, got {:?}", other.msg_id()),
        }
        let f = S2c::LucyPopup(l).encode();
        assert_eq!(f[2], 141);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::LucyPopup(d) => assert_eq!(d, l),
            other => panic!("expected 141, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51 id-140: the `140` C2s twin (kinds 1/2, the client's town-slime
    /// transform requests) byte-pins the kind byte + int value
    /// (`NetMessage.cs:1591-1594`) and round-trips — the same body the S2c
    /// kind-0 credits arm carries.
    #[test]
    fn w4_g51_misc_event_value_c2s_round_trips() {
        for (kind, value) in [(1u8, 7i32), (2, 12), (0, 28800)] {
            let v = MiscEventValue { kind, value };
            let mut body = vec![kind];
            body.extend_from_slice(&value.to_le_bytes());
            let c = C2s::MiscEventValue(v).encode();
            assert_eq!(c[2], 140);
            assert_eq!(&c[3..], &body);
            match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
                C2s::MiscEventValue(d) => assert_eq!(d, v),
                other => panic!("expected 140, got {:?}", other.msg_id()),
            }
        }
    }

    /// W4-g51: the `143 CrystalInvasionSkip` frame is bodyless — a bare
    /// 3-byte frame (length 3, id 143, no body; `DD2Event.cs:1790`).
    #[test]
    fn w4_g51_crystal_invasion_skip_is_bodyless() {
        let c = C2s::CrystalInvasionSkip.encode();
        assert_eq!(c.len(), 3, "the bare 3-byte frame");
        assert_eq!(c[2], 143);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::CrystalInvasionSkip => {}
            other => panic!("expected 143, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `150 SpectatePlayer` body byte-pins the player+target
    /// pair (`NetMessage.cs:1639-1642`: byte player, short target; -1 stops)
    /// and round-trips both directions.
    #[test]
    fn w4_g51_spectate_player_round_trips() {
        let s = SpectatePlayer {
            player_id: 3,
            target: -1,
        };
        let mut body = vec![3u8];
        body.extend_from_slice(&(-1i16).to_le_bytes());
        let c = C2s::SpectatePlayer(s).encode();
        assert_eq!(c[2], 150);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::SpectatePlayer(d) => assert_eq!(d, s),
            other => panic!("expected 150, got {:?}", other.msg_id()),
        }
        let f = S2c::SpectatePlayer(s).encode();
        assert_eq!(f[2], 150);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SpectatePlayer(d) => assert_eq!(d, s),
            other => panic!("expected 150, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `153 NPCDebuffDamage` body byte-pins the slot+damage pair
    /// (`NetMessage.cs:1646-1649`: byte npc, short damage); S2c-only.
    #[test]
    fn w4_g51_npc_debuff_damage_round_trips() {
        let d = NpcDebuffDamage { npc: 7, damage: 50 };
        let mut body = vec![7u8];
        body.extend_from_slice(&50i16.to_le_bytes());
        let f = S2c::NpcDebuffDamage(d).encode();
        assert_eq!(f[2], 153);
        assert_eq!(&f[3..], &body);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::NpcDebuffDamage(x) => assert_eq!(x, d),
            other => panic!("expected 153, got {:?}", other.msg_id()),
        }
    }


    /// W4-g51: the `158 ExtraSpawnLoaded` body is one byte
    /// (`NetMessage.cs:1659-1661`); S2c-only.
    #[test]
    fn w4_g51_extra_spawn_loaded_round_trips() {
        let f = S2c::ExtraSpawnLoaded { player_id: 4 }.encode();
        assert_eq!(f[2], 158);
        assert_eq!(&f[3..], &[4]);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ExtraSpawnLoaded { player_id } => assert_eq!(player_id, 4),
            other => panic!("expected 158, got {:?}", other.msg_id()),
        }
    }

    /// W4-g51: the `159 RequestSection` body byte-pins the two section
    /// indices (`NetMessage.cs:1662-1665` writes shorts, the reader reads
    /// ushorts — same bytes for non-negative indices); C2s-only.
    #[test]
    fn w4_g51_section_request_round_trips() {
        let mut body = Vec::new();
        body.extend_from_slice(&21u16.to_le_bytes());
        body.extend_from_slice(&7u16.to_le_bytes());
        let c = C2s::SectionRequest {
            section_x: 21,
            section_y: 7,
        }
        .encode();
        assert_eq!(c[2], 159);
        assert_eq!(&c[3..], &body);
        match C2s::decode(c[2], &mut NetReader::new(&c[3..])).expect("decode") {
            C2s::SectionRequest {
                section_x,
                section_y,
            } => assert_eq!((section_x, section_y), (21, 7)),
            other => panic!("expected 159, got {:?}", other.msg_id()),
        }
    }

    /// W4-g52 — `162 DamageNPCAck` (`MessageID.cs:338`): NO body, `SendData`
    /// has no case 162, so the frame is the bare id (`MessageBuffer.cs:
    /// 1768-1771` sends it at the head of the `28` arm; `:1819-1824` the
    /// client pops `NPC.pendingDamages[0]`). Byte-verbatim with the d92 codec.
    #[test]
    fn w4_g52_npc_strike_ack_is_bodyless() {
        let f = S2c::NpcStrikeAck.encode();
        assert_eq!(f, vec![3, 0, 162]);
        assert_eq!(S2c::NpcStrikeAck.msg_id(), id::NPC_STRIKE_ACK);
        match S2c::decode(f[2], &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::NpcStrikeAck => {}
            other => panic!("expected 162, got {:?}", other.msg_id()),
        }
        // The client never sends it — no C2s arm exists.
        assert!(C2s::decode(162, &mut NetReader::new(&[])).is_none());
    }

    /// W4-g52 — `34 ChestUpdates` (`NetMessage.cs:913-927`): byte action +
    /// three shorts + the fifth short — the host writes `number5`, the client
    /// a literal 0 (`:926`); the host reader zeroes it
    /// (`MessageBuffer.cs:1991-1994`), the client keeps it (`:1990`).
    /// Byte-verbatim with the g48 codec (keep-one at merge).
    #[test]
    fn w4_g52_chest_updates_round_trips() {
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

    // ---- W4-g52: the SendData relay arms (NetMessage.cs:1692-1826) ----

    /// A live server with `n` raw-socket peers (no handshake — the relay
    /// tests drive the slot state directly and read the pushed frames).
    fn relay_server_with_peers(n: usize) -> (Server, Vec<TcpStream>) {
        let server = Server::start(0, String::new()).unwrap();
        let addr = server.addr().to_string();
        let mut socks = Vec::new();
        for _ in 0..n {
            socks.push(TcpStream::connect(&addr).unwrap());
        }
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        while server.client_count() < n && Instant::now() < deadline {
            thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(server.client_count(), n, "the accept thread registered all");
        (server, socks)
    }

    /// Drains everything queued on a peer socket; returns the frame ids.
    fn drained_ids(sock: &mut TcpStream) -> Vec<u8> {
        sock.set_read_timeout(Some(std::time::Duration::from_millis(150)))
            .unwrap();
        let mut ids = Vec::new();
        while let Ok(f) = read_frame(sock) {
            ids.push(f[0]);
        }
        ids
    }

    fn empty_tile_block() -> S2c {
        S2c::TileBlock(Box::new(TileBlock {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
            records: vec![TileRecord {
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
            }],
        }))
    }

    fn one_tile_square(x: i16, y: i16, w: u8, h: u8) -> S2c {
        let mut r = match empty_tile_block() {
            S2c::TileBlock(b) => b.records[0],
            _ => unreachable!(),
        };
        r.active = true;
        r.kind = 1;
        S2c::SectionRange(Box::new(SectionRange {
            x,
            y,
            w,
            h,
            change_type: 0,
            records: vec![r; (w as usize) * (h as usize)],
        }))
    }

    /// The default arm (`NetMessage.cs:1801-1811`): the `broadcast` flag
    /// opens at State 10 (`MessageBuffer.cs:928`), the State >= 3 bypass is
    /// `10 TileBlock` ONLY, and `ignoreClient` drops the sender.
    #[test]
    fn w4_g52_default_arm_gates_on_broadcast_with_the_tile_block_bypass() {
        let (server, mut socks) = relay_server_with_peers(2);
        // Both FRESH: nothing fans out.
        server.broadcast(&S2c::Ping);
        assert!(drained_ids(&mut socks[0]).is_empty());
        assert!(drained_ids(&mut socks[1]).is_empty());
        // Slot 1 in-world, slot 2 still streaming (State 3).
        server.set_state(1, slot_state::IN_WORLD);
        server.set_state(2, slot_state::SPAWN_REQUESTED);
        server.broadcast(&S2c::Ping);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::PING]);
        assert!(
            drained_ids(&mut socks[1]).is_empty(),
            "State 3 has no broadcast flag — the bypass is id 10 only"
        );
        server.broadcast(&empty_tile_block());
        assert_eq!(drained_ids(&mut socks[0]), vec![id::TILE_BLOCK]);
        assert_eq!(
            drained_ids(&mut socks[1]),
            vec![id::TILE_BLOCK],
            "`(State >= 3 && msgType == 10)` streams sections pre-join"
        );
        // ignoreClient = 1: the sender never gets its own relay.
        server.broadcast_except(&S2c::Ping, 1);
        assert!(drained_ids(&mut socks[0]).is_empty());
        assert!(drained_ids(&mut socks[1]).is_empty());
        server.set_state(2, slot_state::IN_WORLD);
        server.broadcast_except(&S2c::Ping, 1);
        assert!(drained_ids(&mut socks[0]).is_empty());
        assert_eq!(drained_ids(&mut socks[1]), vec![id::PING]);
    }

    /// The `20` arm (`NetMessage.cs:1708-1718`): the square's four corners
    /// (`RemoteClient.SectionRange`, `RemoteClient.cs:191-218`) must land in
    /// a section the slot already holds — the LATCH, not the check-time grid
    /// — and there is no State-3 bypass on this arm.
    #[test]
    fn w4_g52_tile_square_relays_only_to_slots_holding_a_corner_section() {
        let (server, mut socks) = relay_server_with_peers(3);
        for i in 1..=2 {
            server.set_state(i, slot_state::IN_WORLD);
            server.ensure_sections(i, 900, 600); // 5x4 grid
        }
        // Slot 3 never reaches State 10 — its broadcast flag stays closed
        // (the C# State never lowers; `Reset` clears on reuse only).
        server.set_state(3, slot_state::SPAWN_REQUESTED);
        server.ensure_sections(3, 900, 600);
        server.mark_section_sent(1, 0, 0);
        server.mark_section_sent(2, 2, 1);
        server.mark_section_sent(3, 0, 0); // latched, but no broadcast flag
        // A check-time stamp alone does NOT satisfy the 20 arm.
        server.mark_section_checked(2, 0, 0, 1000);
        // 1x1 at (10,10): every corner in section (0,0) — slot 1 only.
        server.relay_tile_square(&one_tile_square(10, 10, 1, 1), None);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::SECTION_RANGE]);
        assert!(drained_ids(&mut socks[1]).is_empty());
        assert!(
            drained_ids(&mut socks[2]).is_empty(),
            "the 20 arm has no State-3 bypass (NetMessage.cs:1712)"
        );
        // 3x3 at (198,148): corners (0,0)/(1,0)/(0,1)/(1,1) — slot 1 holds
        // (0,0); slot 2's (2,1) is not a corner.
        server.relay_tile_square(&one_tile_square(198, 148, 3, 3), None);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::SECTION_RANGE]);
        assert!(drained_ids(&mut socks[1]).is_empty());
        // 1x1 at (410,160): section (2,1) — slot 2 only.
        server.relay_tile_square(&one_tile_square(410, 160, 1, 1), None);
        assert!(drained_ids(&mut socks[0]).is_empty());
        assert_eq!(drained_ids(&mut socks[1]), vec![id::SECTION_RANGE]);
        // Nobody holds (4,3): the square reaches no one.
        server.relay_tile_square(&one_tile_square(850, 550, 1, 1), None);
        assert!(drained_ids(&mut socks[0]).is_empty());
        assert!(drained_ids(&mut socks[1]).is_empty());
        // The size is max(w, h) (`:1712`): a 1x3 at (199,0) reaches section
        // (1,0) with its far corner — slot 1's (0,0) is the first corner.
        server.relay_tile_square(&one_tile_square(199, 0, 1, 3), None);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::SECTION_RANGE]);
        assert!(drained_ids(&mut socks[1]).is_empty());
    }

    /// The `28` arm (`NetMessage.cs:1751-1762`): a live NPC's strike only
    /// reaches section-active clients; a dead one's reaches everyone. The
    /// window is the 60-tick `SectionInactiveTime` (`ActiveSections.cs:65`).
    #[test]
    fn w4_g52_strike_relays_by_section_activity_or_death() {
        let (server, mut socks) = relay_server_with_peers(3);
        for i in 1..=3 {
            server.set_state(i, slot_state::IN_WORLD);
            server.ensure_sections(i, 900, 600);
        }
        server.mark_section_checked(1, 0, 0, 1000);
        server.mark_section_checked(3, 0, 0, 1000); // the sender
        let strike = S2c::NpcStrike(NpcStrike {
            npc: 0,
            generation: 1,
            damage: 10,
            knock_back: 3.0,
            hit_direction: 1,
            crit: false,
        });
        let live = RelayCtx::Strike {
            dead: false,
            section: (0, 0),
        };
        server.relay_strike(&strike, 3, live, 1000);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::NPC_STRIKE]);
        assert!(
            drained_ids(&mut socks[1]).is_empty(),
            "slot 2 never checked section (0,0) — culled"
        );
        assert!(
            drained_ids(&mut socks[2]).is_empty(),
            "ignoreClient = the striking peer"
        );
        // The window edge: 940 + 60 >= 1000 passes, 1001 fails.
        server.mark_section_checked(2, 0, 0, 940);
        server.relay_strike(&strike, 3, live, 1000);
        assert_eq!(drained_ids(&mut socks[1]), vec![id::NPC_STRIKE]);
        server.relay_strike(&strike, 3, live, 1001);
        assert!(drained_ids(&mut socks[1]).is_empty());
        // Death bypasses the section filter for everyone but the sender.
        let dead = RelayCtx::Strike {
            dead: true,
            section: (0, 0),
        };
        server.relay_strike(&strike, 3, dead, 5000);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::NPC_STRIKE]);
        assert_eq!(drained_ids(&mut socks[1]), vec![id::NPC_STRIKE]);
        assert!(drained_ids(&mut socks[2]).is_empty());
    }

    /// The `27` arm (`NetMessage.cs:1774-1800`): flag3 (`type 12 / projPet /
    /// aiStyle 11 / netImportant`, `:1777`) bypasses the section filter;
    /// otherwise the client's section must be active.
    #[test]
    fn w4_g52_projectile_relays_by_flag3_or_section_activity() {
        let (server, mut socks) = relay_server_with_peers(2);
        for i in 1..=2 {
            server.set_state(i, slot_state::IN_WORLD);
            server.ensure_sections(i, 900, 600);
        }
        server.mark_section_checked(1, 1, 1, 2000);
        let ps = S2c::Projectile(Box::new(ProjectileState {
            key: projectile_key(1, 0, 1),
            x: 300.0,
            y: 300.0,
            vx: 4.0,
            vy: 0.0,
            kind: 1,
            ai0: None,
            ai1: None,
            banner_id: 0,
            damage: 10,
            knock_back: 2.0,
            original_damage: 10,
            ai2: None,
        }));
        let plain = RelayCtx::Projectile {
            ignores_sections: false,
            section: (1, 1),
        };
        server.relay_projectile(&ps, Some(2), plain, 2000);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::PROJECTILE]);
        assert!(drained_ids(&mut socks[1]).is_empty());
        let flag3 = RelayCtx::Projectile {
            ignores_sections: true,
            section: (1, 1),
        };
        server.relay_projectile(&ps, None, flag3, 9000);
        assert_eq!(drained_ids(&mut socks[0]), vec![id::PROJECTILE]);
        assert_eq!(
            drained_ids(&mut socks[1]),
            vec![id::PROJECTILE],
            "flag3 clears the section cull for every client"
        );
        // The window expiry culls the plain shot again.
        server.relay_projectile(&ps, None, plain, 2061);
        assert!(drained_ids(&mut socks[0]).is_empty());
        assert!(drained_ids(&mut socks[1]).is_empty());
    }

    /// The `34/69` arm (`NetMessage.cs:1696-1707`) is the plain broadcast
    /// gate — no section filter, no State-3 bypass. The port's `69 ChestName`
    /// rides it; `34 ChestUpdates` has no port body yet.
    #[test]
    fn w4_g52_chest_name_relays_on_the_plain_broadcast_gate() {
        let (server, mut socks) = relay_server_with_peers(2);
        server.set_state(1, slot_state::IN_WORLD);
        server.set_state(2, slot_state::IN_WORLD);
        // No sections allocated at all — the 69 arm does not read them.
        let name = S2c::ChestName(ChestName {
            chest: 0,
            x: 3,
            y: 5,
            name: "loot".into(),
        });
        server.broadcast_except(&name, 1);
        assert!(drained_ids(&mut socks[0]).is_empty());
        assert_eq!(drained_ids(&mut socks[1]), vec![id::CHEST_NAME]);
    }

    /// `section_active_at`'s uint wrap + the 60-tick boundary
    /// (`RemoteClient.cs:223`, `ActiveSections.cs:65`), and the latch's
    /// out-of-grid refusal (`RemoteClient.cs:212`'s throw inside the try).
    #[test]
    fn w4_g52_section_predicates_wrap_and_refuse_out_of_grid() {
        assert!(section_active_at(1000, 1060), "the boundary is inclusive");
        assert!(!section_active_at(1000, 1061));
        assert!(
            section_active_at(u32::MAX - 10, 49),
            "the C# uint add wraps: MAX-10 + 60 = 49"
        );
        assert!(!section_active_at(u32::MAX - 10, 50));
        let (server, _socks) = relay_server_with_peers(1);
        server.set_state(1, slot_state::IN_WORLD);
        // Unallocated grid: both predicates refuse.
        assert!(!server.section_range(1, 1, 0, 0));
        assert!(!server.is_section_active(1, 0, 0, 0));
        server.ensure_sections(1, 900, 600);
        server.mark_section_sent(1, 0, 0);
        // A negative corner is refused, not panicked (the C# throws inside
        // SendData's try; TrySendData swallows it).
        assert!(!server.section_range(1, 1, -5, -5));
        // The far corner still lands in the latched section.
        assert!(server.section_range(1, 1, 199, 149));
        assert!(!server.section_range(1, 1, 200, 150));
        // IsSectionActive clamps negatives onto (0,0) (ActiveSections.cs:53).
        server.mark_section_checked(1, 0, 0, 500);
        assert!(server.is_section_active(1, -3, -7, 560));
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
    fn w4_f21_client_synced_inventory_is_a_bodyless_default_drop() {
        // `138` — NO writer case (`SendData(138)` at Main.cs:17700 writes
        // a bare frame) and NO reader case: the default arm
        // (`MessageBuffer.cs:4477-4482`) reads nothing.
        let f = C2s::ClientSyncedInventory.encode();
        assert_eq!(f[2], 138);
        assert_eq!(f.len(), 3);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::ClientSyncedInventory => {}
            other => panic!("expected 138, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only.
        assert!(S2c::decode(138, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_counts_as_host_round_trips() {
        // `NetMessage.cs:1587-1590`: (byte)player + (number2 == 1f) bool;
        // the join arm sends flag13.ToInt() (`MessageBuffer.cs:934`).
        for flag in [true, false] {
            let c = CountsAsHost { player_id: 3, counts_as_host: flag };
            let f = S2c::CountsAsHost(c).encode();
            assert_eq!(f[2], 139);
            assert_eq!(&f[3..], &[3, u8::from(flag)]);
            let mut r = NetReader::new(&f[3..]);
            match S2c::decode(f[2], &mut r).expect("decode") {
                S2c::CountsAsHost(d) => assert_eq!(d, c),
                other => panic!("expected 139, got {:?}", other.msg_id()),
            }
            assert_eq!(r.remaining(), 0);
        }
        // S2c-only.
        assert!(C2s::decode(139, &mut NetReader::new(&[3, 1])).is_none());
        // A truncated bool must not decode.
        assert!(S2c::decode(139, &mut NetReader::new(&[3])).is_none());
    }

    #[test]
    fn w4_f21_request_quest_effect_is_a_bodyless_client_ask() {
        // `144` has NO writer case — `SendData(144)` (Main.cs:40287, the
        // Stardew dryad bit) writes only the header; the host reader reads
        // nothing (`MessageBuffer.cs:4304-4309`).
        let f = C2s::RequestQuestEffect.encode();
        assert_eq!(f[2], 144);
        assert_eq!(f.len(), 3);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::RequestQuestEffect => {}
            other => panic!("expected 144, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only.
        assert!(S2c::decode(144, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_shimmer_items_deprecated_is_censused_dead() {
        // `145` — no writer case, no reader case, zero send sites; the
        // shimmer item sync became 146. Constant + census doc only (the
        // 24 precedent: no variant for a frame with neither arm).
        assert_eq!(id::SHIMMER_ITEMS_DEPRECATED, 145);
        assert!(C2s::decode(145, &mut NetReader::new(&[])).is_none());
        assert!(S2c::decode(145, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_shimmer_actions_round_trips() {
        // `NetMessage.cs:1611-1623`: (byte)kind; 0 → Vector2, 1 →
        // Vector2 + (int)amount, else nothing.
        let fx = ShimmerAction::Effect { x: 100.5, y: -20.25 };
        let f = S2c::ShimmerAction(fx).encode();
        assert_eq!(f[2], 146);
        let mut body = vec![0u8];
        body.extend_from_slice(&100.5f32.to_le_bytes());
        body.extend_from_slice(&(-20.25f32).to_le_bytes());
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::ShimmerAction(d) => assert_eq!(d, fx),
            other => panic!("expected 146, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The coin-luck arm (`WorldItem.cs:1716`: the stack rides number4).
        let coin = ShimmerAction::CoinLuck { x: 1.0, y: 2.0, amount: 10000 };
        let f = S2c::ShimmerAction(coin).encode();
        let mut body = vec![1u8];
        body.extend_from_slice(&1.0f32.to_le_bytes());
        body.extend_from_slice(&2.0f32.to_le_bytes());
        body.extend_from_slice(&10000i32.to_le_bytes());
        assert_eq!(&f[3..], &body);
        match S2c::decode(146, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ShimmerAction(d) => assert_eq!(d, coin),
            other => panic!("expected 146, got {:?}", other.msg_id()),
        }
        // An unknown kind writes/reads NOTHING after the byte.
        let f = S2c::ShimmerAction(ShimmerAction::Unknown(7)).encode();
        assert_eq!(&f[3..], &[7]);
        match S2c::decode(146, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ShimmerAction(d) => assert_eq!(d, ShimmerAction::Unknown(7)),
            other => panic!("expected 146, got {:?}", other.msg_id()),
        }
        // S2c-only.
        assert!(C2s::decode(146, &mut NetReader::new(&[0, 0, 0, 0, 0, 0, 0, 0, 0])).is_none());
        // A truncated coin amount must not decode.
        assert!(S2c::decode(146, &mut NetReader::new(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])).is_none());
    }

    #[test]
    fn w4_f21_item_enemy_lock_deprecated_is_censused_dead() {
        // `148` — no writer case, no reader case, zero send sites; the
        // enemy-grab lock became the 21 bit-3 tail (d98's wave I).
        // Constant + census doc only (the 24/145 precedent).
        assert_eq!(id::ITEM_ENEMY_LOCK_DEPRECATED, 148);
        assert!(C2s::decode(148, &mut NetReader::new(&[])).is_none());
        assert!(S2c::decode(148, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_dead_cells_jar_try_placing_is_the_89_twin() {
        // `NetMessage.cs:1629-1638` — the exact 89 body under id 149.
        let b = TryPlacingItem {
            x: 60,
            y: 400,
            kind: 5335,
            prefix: 0,
            stack: 1,
        };
        let f149 = C2s::DeadCellsJarTryPlacing(b).encode();
        let f89 = C2s::ItemFrameTryPlacing(b).encode();
        assert_eq!(f149[2], 149);
        assert_eq!(&f149[3..], &f89[3..]); // identical bodies
        let mut r = NetReader::new(&f149[3..]);
        match C2s::decode(f149[2], &mut r).expect("decode") {
            C2s::DeadCellsJarTryPlacing(d) => assert_eq!(d, b),
            other => panic!("expected 149, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only.
        assert!(S2c::decode(149, &mut NetReader::new(&f149[3..])).is_none());
    }

    #[test]
    fn w4_f21_item_use_sound_round_trips_both_directions() {
        // `NetMessage.cs:1643-1645`: (byte)player only — the sound itself
        // is re-derived from the peer's held item on the client
        // (`MessageBuffer.cs:4399-4404`).
        let f = C2s::ItemUseSound { player_id: 5 }.encode();
        assert_eq!(f[2], 152);
        assert_eq!(&f[3..], &[5]);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::ItemUseSound { player_id } => assert_eq!(player_id, 5),
            other => panic!("expected 152, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        let f = S2c::ItemUseSound { player_id: 5 }.encode();
        assert_eq!(f[2], 152);
        match S2c::decode(152, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::ItemUseSound { player_id } => assert_eq!(player_id, 5),
            other => panic!("expected 152, got {:?}", other.msg_id()),
        }
        // A missing byte must not decode.
        assert!(C2s::decode(152, &mut NetReader::new(&[])).is_none());
    }

    #[test]
    fn w4_f21_leashed_anchor_place_item_round_trips() {
        // `NetMessage.cs:1654-1658`: three (short)s — x, y, itemType.
        let b = LeashedAnchorPlaceItem { x: -4, y: 500, item_type: 4765 };
        let mut body = (-4i16).to_le_bytes().to_vec();
        body.extend_from_slice(&500i16.to_le_bytes());
        body.extend_from_slice(&4765i16.to_le_bytes());
        let f = C2s::LeashedAnchorPlaceItem(b).encode();
        assert_eq!(f[2], 156);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::LeashedAnchorPlaceItem(d) => assert_eq!(d, b),
            other => panic!("expected 156, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // C2s-only.
        assert!(S2c::decode(156, &mut NetReader::new(&body)).is_none());
        // A truncated itemType must not decode.
        assert!(C2s::decode(156, &mut NetReader::new(&body[..5])).is_none());
    }

    #[test]
    fn w4_f21_item_position_round_trips() {
        // `NetMessage.cs:1666-1672`: (short)slot + WriteVector2(position)
        // — no velocity (the section-sweep snap, WorldItem.cs:1864).
        let p = ItemPosition { slot: 399, x: 1234.5, y: -67.25 };
        let mut body = 399i16.to_le_bytes().to_vec();
        body.extend_from_slice(&1234.5f32.to_le_bytes());
        body.extend_from_slice(&(-67.25f32).to_le_bytes());
        let f = S2c::ItemPosition(p).encode();
        assert_eq!(f[2], 160);
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match S2c::decode(f[2], &mut r).expect("decode") {
            S2c::ItemPosition(d) => assert_eq!(d, p),
            other => panic!("expected 160, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // S2c-only.
        assert!(C2s::decode(160, &mut NetReader::new(&body)).is_none());
        // A truncated y must not decode.
        assert!(S2c::decode(160, &mut NetReader::new(&body[..9])).is_none());
    }

    #[test]
    fn w4_f21_host_token_round_trips() {
        // `NetMessage.cs:1673-1675`: one plain string (text.ToString());
        // the join push sends Netplay.HostToken (`MessageBuffer.cs:275`).
        let token = "a-host-token-123".to_string();
        let f = C2s::HostToken(token.clone()).encode();
        assert_eq!(f[2], 161);
        let mut body = vec![token.len() as u8];
        body.extend_from_slice(token.as_bytes());
        assert_eq!(&f[3..], &body);
        let mut r = NetReader::new(&f[3..]);
        match C2s::decode(f[2], &mut r).expect("decode") {
            C2s::HostToken(d) => assert_eq!(d, token),
            other => panic!("expected 161, got {:?}", other.msg_id()),
        }
        assert_eq!(r.remaining(), 0);
        // The empty token is a bare 0 length byte (the reader's
        // IsNullOrWhiteSpace gate then refuses it, :4474).
        let f = C2s::HostToken(String::new()).encode();
        assert_eq!(&f[3..], &[0]);
        // C2s-only.
        assert!(S2c::decode(161, &mut NetReader::new(&body)).is_none());
        // A truncated payload must not decode.
        assert!(C2s::decode(161, &mut NetReader::new(&[16, b'a'])).is_none());
    }

    #[test]
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

    /// W4-c27 SEAL (research/332): the PARKED `sunModY`/`moonModY` offsets
    /// still ride the wire verbatim — the full `short` range round-trips
    /// (`NetMessage.cs:534-535`, both `short`, `Main.cs:1185`/`:1187`), so a
    /// future title-screen-drag sender needs no codec change.
    #[test]
    fn w4_c27_set_time_mod_offsets_ride_the_full_short_range() {
        let t = SetTime {
            day_time: true,
            time: 0,
            sun_mod_y: i16::MIN,
            moon_mod_y: i16::MAX,
        };
        let f = S2c::SetTime(t).encode();
        assert_eq!(f[2], 18);
        // Body: day byte + 4 time bytes, then the two shorts LE.
        assert_eq!(&f[8..12], &[0x00, 0x80, 0xff, 0x7f]);
        match S2c::decode(18, &mut NetReader::new(&f[3..])).expect("decode") {
            S2c::SetTime(d) => assert_eq!(d, t),
            other => panic!("expected 18, got {:?}", other.msg_id()),
        }
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

    #[test]
    fn w4_f21_tail_census_this_branch_closes_43_of_44() {
        // The f21 census: MessageID 1-162 vs net.rs + the d89-d100/c27
        // lanes found 44 MISSING ids; 132 PlayLegacySound is dead (zero
        // PlayNetSound call-sites) and skipped per the task. This branch
        // ports the other 43 — constants for all of them (the dead
        // 24/145/148 are census-doc-only, the 24 precedent).
        let f21_ids: [u8; 43] = [
            24, 25, 26, 39, 44, 48, 52, 57, 58, 67, 76, 79, 83, 85, 86, 87, 88, 89, 90, 93, 94,
            104, 105, 107, 111, 119, 121, 122, 123, 124, 129, 133, 138, 139, 144, 145, 146, 148,
            149, 152, 156, 160, 161,
        ];
        // Every f21 id has a constant in `id` (checked via the encode
        // path where a variant exists, and the constant where dead).
        assert_eq!(id::UNUSED_MELEE_STRIKE, 24);
        assert_eq!(id::SHIMMER_ITEMS_DEPRECATED, 145);
        assert_eq!(id::ITEM_ENEMY_LOCK_DEPRECATED, 148);
        // The live ones round-trip their id byte through msg_id().
        let live_c2s: Vec<u8> = vec![
            C2s::ReleaseItemOwnership(ReleaseItemOwnership { slot: 0, force_assign_to_server: false }).msg_id(),
            C2s::LiquidUpdate(LiquidUpdate { x: 0, y: 0, liquid: 0, liquid_type: 0 }).msg_id(),
            C2s::LockAndUnlock(LockAndUnlock { action: 1, x: 0, y: 0 }).msg_id(),
            C2s::InstrumentSound(InstrumentSound { player_id: 0, pitch: 0.0 }).msg_id(),
            C2s::QuestsCountSync(QuestsCountSync { player_id: 0, angler_quests_finished: 0, golfer_score: 0 }).msg_id(),
            C2s::PlaceObject(PlaceObject { x: 0, y: 0, kind: 0, style: 0, alternative: 0, random: 0, direction: true }).msg_id(),
            C2s::QuickStackRequest(QuickStackRequest { slots: vec![], smart_stack: false }).msg_id(),
            C2s::TileEntityPlacement(TileEntityPlacement { x: 0, y: 0, kind: 0 }).msg_id(),
            C2s::ItemFrameTryPlacing(TryPlacingItem { x: 0, y: 0, kind: 0, prefix: 0, stack: 0 }).msg_id(),
            C2s::DevCommands(DevCommands { command: String::new(), number: 0, number2: 0.0, number3: 0.0 }).msg_id(),
            C2s::GemLockToggle(GemLockToggle { x: 0, y: 0, on: false }).msg_id(),
            C2s::ToggleParty.msg_id(),
            C2s::DisplayDollSync(DisplayDollSync { player_id: 0, entity_id: 0, item_index: 0, command: 2, data: DollSyncData::Pose(0) }).msg_id(),
            C2s::TileEntityInteraction(TileEntityInteraction { entity_id: -1, player_id: 0 }).msg_id(),
            C2s::WeaponsRackTryPlacing(TryPlacingItem { x: 0, y: 0, kind: 0, prefix: 0, stack: 0 }).msg_id(),
            C2s::HatRackItemSync(HatRackItemSync { player_id: 0, entity_id: 0, wire_slot: 0, item: TeItem::default() }).msg_id(),
            C2s::FoodPlatterTryPlacing(TryPlacingItem { x: 0, y: 0, kind: 0, prefix: 0, stack: 0 }).msg_id(),
            C2s::ClientSyncedInventory.msg_id(),
            C2s::RequestQuestEffect.msg_id(),
            C2s::DeadCellsJarTryPlacing(TryPlacingItem { x: 0, y: 0, kind: 0, prefix: 0, stack: 0 }).msg_id(),
            C2s::ItemUseSound { player_id: 0 }.msg_id(),
            C2s::LeashedAnchorPlaceItem(LeashedAnchorPlaceItem { x: 0, y: 0, item_type: 0 }).msg_id(),
            C2s::HostToken(String::new()).msg_id(),
        ];
        for got in &live_c2s {
            assert!(f21_ids.contains(got), "unexpected C2s id {got}");
        }
        let live_s2c: Vec<u8> = vec![
            S2c::Unused25.msg_id(),
            S2c::Unused26.msg_id(),
            S2c::Unknown44.msg_id(),
            S2c::Unknown67.msg_id(),
            S2c::Unused83.msg_id(),
            S2c::SocialHandshake.msg_id(),
            S2c::ReleaseItemOwnership(ReleaseItemOwnership { slot: 0, force_assign_to_server: false }).msg_id(),
            S2c::LiquidUpdate(LiquidUpdate { x: 0, y: 0, liquid: 0, liquid_type: 0 }).msg_id(),
            S2c::WorldEvilBytes(WorldEvilBytes { good: 0, evil: 0, blood: 0 }).msg_id(),
            S2c::InstrumentSound(InstrumentSound { player_id: 0, pitch: 0.0 }).msg_id(),
            S2c::QuestsCountSync(QuestsCountSync { player_id: 0, angler_quests_finished: 0, golfer_score: 0 }).msg_id(),
            S2c::PlaceObject(PlaceObject { x: 0, y: 0, kind: 0, style: 0, alternative: 0, random: 0, direction: true }).msg_id(),
            S2c::QuickStackBlocked(QuickStackBlocked { chests: vec![] }).msg_id(),
            S2c::TileEntitySharing(TileEntitySharing { id: 0, record: Vec::new() }).msg_id(),
            S2c::ItemTweaker(ItemTweaker::default()).msg_id(),
            S2c::SpawnInstancedItem(Box::new(ItemState { slot: 0, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, stack: 0, prefix: 0, kind: 0, shimmered: false, shimmer_time: 0.0, enemy_grab_delay: 0 })).msg_id(),
            S2c::DevCommands(DevCommands { command: String::new(), number: 0, number2: 0.0, number3: 0.0 }).msg_id(),
            S2c::ShopOverride(ShopOverride { slot: 0, kind: 0, stack: 0, prefix: 0, value: 0, buy_once: false }).msg_id(),
            S2c::SmartText(SmartText { color: Rgb(0, 0, 0), text: NetText(String::new()), width_limit: 0 }).msg_id(),
            S2c::CombatTextString(CombatTextString { x: 0.0, y: 0.0, color: Rgb(0, 0, 0), text: NetText(String::new()) }).msg_id(),
            S2c::DisplayDollSync(DisplayDollSync { player_id: 0, entity_id: 0, item_index: 0, command: 2, data: DollSyncData::Pose(0) }).msg_id(),
            S2c::TileEntityInteraction(TileEntityInteraction { entity_id: -1, player_id: 0 }).msg_id(),
            S2c::HatRackItemSync(HatRackItemSync { player_id: 0, entity_id: 0, wire_slot: 0, item: TeItem::default() }).msg_id(),
            S2c::FinishedConnecting.msg_id(),
            S2c::CountsAsHost(CountsAsHost { player_id: 0, counts_as_host: false }).msg_id(),
            S2c::ShimmerAction(ShimmerAction::Unknown(0)).msg_id(),
            S2c::ItemUseSound { player_id: 0 }.msg_id(),
            S2c::ItemPosition(ItemPosition { slot: 0, x: 0.0, y: 0.0 }).msg_id(),
        ];
        for got in &live_s2c {
            assert!(f21_ids.contains(got), "unexpected S2c id {got}");
        }
        // 132 is the ONLY missing id this branch does not carry — the
        // dead PlayLegacySound skip (zero PlayNetSound call-sites).
        assert!(!f21_ids.contains(&132));
        assert!(C2s::decode(132, &mut NetReader::new(&[])).is_none());
        assert!(S2c::decode(132, &mut NetReader::new(&[])).is_none());
    }
}

/// `msg 119 CombatTextString`'s body (`NetMessage.cs:1188-1196`, W4-g50):
/// the 81 twin with a `NetworkText` in place of the int — two floats,
/// `WriteRGB`'s three bytes, then `text.Serialize(writer)`. The client
/// truncates the floats and spawns `CombatText.NewText` with
/// `NetworkText.Deserialize(...).ToString()` at `Rectangle(x, y, 0, 0)`
/// (`MessageBuffer.cs:3235-3242`); the C# send sites are the death-message
/// and crit broadcasts (`NetMessage.SendCombatText`).
#[derive(Clone, Debug, PartialEq)]
pub struct CombatTextString {
    pub x: f32,
    pub y: f32,
    pub color: Rgb,
    pub text: NetText,
}

/// `msg 84 PlayerStealth`'s body (`NetMessage.cs:1197-1204`, W4-g50): the
/// player id byte then `Main.player[b3].stealth` as a `float`
/// (`Player.cs:1562`, default 1f). Sent by the stealth-armour update
/// (`Player.cs:25766`-area); the host overrides the id to the sender and
/// relays (`MessageBuffer.cs:3249-3258`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerStealth {
    pub player_id: u8,
    pub stealth: f32,
}

/// `msg 92 SyncExtraValue`'s body (`NetMessage.cs:1321-1326`): `(short)` npc
/// slot, `(int)` value, `(float)` x, `(float)` y (the writer's `number3` /
/// `number4` are floats — the reader `ReadSingle`s them,
/// `MessageBuffer.cs:3446-3447`). The value is a DELTA on the client→host
/// leg (the host does `extraValue += num229` then relays the accumulated
/// total, `MessageBuffer.cs:3457-3458`) and the TOTAL on the host→client
/// leg (the client assigns it and pings the coins at (x, y), `:3452-3453`).
/// Sent when a dropped coin is absorbed by a town NPC
/// (`WorldItem.cs:1186-1194`, the `netMode != 0` branch).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpcExtraValue {
    pub npc: i16,
    pub value: i32,
    pub x: f32,
    pub y: f32,
}

/// `msg 86 TileEntitySharing`'s body (`NetMessage.cs:1216-1225`, W4-g50):
/// `(int)number` (the entity id), `(bool)` present (`TileEntity.TryGet`),
/// then — when present — `TileEntity.Write(writer, result3, networkSend:
/// true)`: the type byte, the position shorts and the extra data, NO id
/// (`TileEntity.cs:188-197`'s network half). The reader removes the entity
/// when absent and re-adds it under the leading id when present
/// (`MessageBuffer.cs:3272-3293`). The record rides as raw bytes — the
/// per-kind shape is `tr_core::tile_entities::TileEntity::read_net`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileEntitySharing {
    pub id: i32,
    /// The `networkSend: true` record (type byte + position + extra data);
    /// empty when `present` is false.
    pub record: Vec<u8>,
}

/// `msg 87 TileEntityPlacement`'s body (`NetMessage.cs:1227-1231`, W4-g50):
/// `(short)number` x, `(short)number2` y, `(byte)number3` the entity type
/// (`TileEntitiesManager.RegisterAll` order, 0..=10). The host places the
/// entity when the cell is in-world and free, then answers with the `86`
/// sharing broadcast (`MessageBuffer.cs:3294-3305`,
/// `TileEntityType.cs:17-20`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileEntityPlacement {
    pub x: i16,
    pub y: i16,
    pub kind: u8,
}

/// `msg 88 ItemTweaker`'s body (`NetMessage.cs:1232-1296`, W4-g50): the
/// `(short)` item slot, a BitsByte gating seven `Item.inner` stat writes,
/// then — on bit 7 — a second BitsByte gating six more. The fields ride
/// only when their flag is set (the C# `if (bitsByte[i])` arms); an unset
/// field keeps its default and is not written. The reader applies them to
/// `Main.item[slot].inner` (`MessageBuffer.cs:3306-3376`) — the port's world
/// drops carry no mutable inner stats, so the apply is parked (the codec is
/// byte-verbatim for the census). Width/height are `(ushort)`-written but
/// `ReadInt16`-read (`:1272`/`:3352`); the two bytes are identical.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemTweaker {
    pub slot: i16,
    pub flags: BitsByte,
    /// bit 0 — `inner.color.PackedValue` (`uint`).
    pub color: u32,
    /// bit 1 — `inner.damage` (`ushort`).
    pub damage: u16,
    /// bit 2 — `inner.knockBack` (`float`).
    pub knockback: f32,
    /// bit 3 — `inner.useAnimation` (`ushort`).
    pub use_animation: u16,
    /// bit 4 — `inner.useTime` (`ushort`).
    pub use_time: u16,
    /// bit 5 — `inner.shoot` (`short`).
    pub shoot: i16,
    /// bit 6 — `inner.shootSpeed` (`float`).
    pub shoot_speed: f32,
    /// bit 7 — the second BitsByte is present.
    pub flags2: BitsByte,
    /// flags2 bit 0 — `inner.width` (`ushort` write / `short` read).
    pub width: i16,
    /// flags2 bit 1 — `inner.height`.
    pub height: i16,
    /// flags2 bit 2 — `inner.scale` (`float`).
    pub scale: f32,
    /// flags2 bit 3 — `inner.ammo` (`short`).
    pub ammo: i16,
    /// flags2 bit 4 — `inner.useAmmo` (`short`).
    pub use_ammo: i16,
    /// flags2 bit 5 — `inner.notAmmo` (`bool`).
    pub not_ammo: bool,
}

impl ItemTweaker {
    /// `NetMessage.cs:1236-1294`: the slot, the first BitsByte, the gated
    /// fields in bit order, then — on bit 7 — the second BitsByte and its
    /// gated fields.
    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut w = w.i16(self.slot).bitsbyte(self.flags);
        if self.flags.get(0) {
            w = w.u32(self.color);
        }
        if self.flags.get(1) {
            w = w.u16(self.damage);
        }
        if self.flags.get(2) {
            w = w.f32(self.knockback);
        }
        if self.flags.get(3) {
            w = w.u16(self.use_animation);
        }
        if self.flags.get(4) {
            w = w.u16(self.use_time);
        }
        if self.flags.get(5) {
            w = w.i16(self.shoot);
        }
        if self.flags.get(6) {
            w = w.f32(self.shoot_speed);
        }
        if self.flags.get(7) {
            w = w.bitsbyte(self.flags2);
            if self.flags2.get(0) {
                w = w.u16(self.width as u16);
            }
            if self.flags2.get(1) {
                w = w.u16(self.height as u16);
            }
            if self.flags2.get(2) {
                w = w.f32(self.scale);
            }
            if self.flags2.get(3) {
                w = w.i16(self.ammo);
            }
            if self.flags2.get(4) {
                w = w.i16(self.use_ammo);
            }
            if self.flags2.get(5) {
                w = w.bool(self.not_ammo);
            }
        }
        w
    }

    /// `MessageBuffer.cs:3312-3374`: the slot, the BitsByte, the gated
    /// reads; absent fields keep the default (the C# leaves `inner` alone).
    fn read_from(r: &mut NetReader) -> Option<Self> {
        let slot = r.i16()?;
        let flags = r.bitsbyte()?;
        let mut t = Self {
            slot,
            flags,
            ..Default::default()
        };
        if flags.get(0) {
            t.color = r.u32()?;
        }
        if flags.get(1) {
            t.damage = r.u16()?;
        }
        if flags.get(2) {
            t.knockback = r.f32()?;
        }
        if flags.get(3) {
            t.use_animation = r.u16()?;
        }
        if flags.get(4) {
            t.use_time = r.u16()?;
        }
        if flags.get(5) {
            t.shoot = r.i16()?;
        }
        if flags.get(6) {
            t.shoot_speed = r.f32()?;
        }
        if flags.get(7) {
            t.flags2 = r.bitsbyte()?;
            if t.flags2.get(0) {
                t.width = r.i16()?;
            }
            if t.flags2.get(1) {
                t.height = r.i16()?;
            }
            if t.flags2.get(2) {
                t.scale = r.f32()?;
            }
            if t.flags2.get(3) {
                t.ammo = r.i16()?;
            }
            if t.flags2.get(4) {
                t.use_ammo = r.i16()?;
            }
            if t.flags2.get(5) {
                t.not_ammo = r.bool()?;
            }
        }
        Some(t)
    }
}

/// `msg 91 SyncEmoteBubble`'s body (`NetMessage.cs:1307-1320`): the bubble
/// id (`int`), the anchor kind byte, then — unless the kind is `255` (the
/// removal marker, `MessageBuffer.cs:3402-3408`) — the anchor index
/// (`ushort`), the lifetime (`ushort`), the emote byte and, gated on
/// `number5 < 0`, the metadata `short`. The anchor pair is
/// `EmoteBubble.SerializeNetAnchor` (`EmoteBubble.cs:78-98`): kind 0 NPC,
/// 1 Player, 2 Projectile + `entity.whoAmI`; the reader rebuilds it with
/// `DeserializeNetAnchor` (`:100-109`) and sets `emoteTime = 360` on a
/// player anchor (`MessageBuffer.cs:3419-3422`).
///
/// Verbatim quirk: the C# writer's emote byte is `(byte)number5` and its
/// metadata gate is `number5 < 0`, while the reader tests the READ byte
/// (`num247 < 0`, `MessageBuffer.cs:3412-3417`) — a byte is never negative,
/// so the metadata tail is dead on both sides for every emote the game
/// sends (`NewBubble`/`NewBubbleNPC`, `EmoteBubble.cs:128`, `:147`). The
/// port keeps the writer's shape: `metadata` rides only when the signed
/// `emote` is negative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmoteBubble {
    pub id: i32,
    /// `255` = remove the bubble; else 0 NPC / 1 Player / 2 Projectile.
    pub anchor_kind: u8,
    /// `DeserializeNetAnchor`'s entity index (`whoAmI`); only read when
    /// `anchor_kind != 255`.
    pub anchor_index: u16,
    /// `lifeTime`/`lifeTimeStart` (`MessageBuffer.cs:3431-3432`).
    pub life_time: u16,
    /// The emote id as the writer's signed `number5`; the wire byte is its
    /// low 8 bits.
    pub emote: i32,
    /// `EmoteBubble.metadata` (`EmoteBubble.cs:30`) — the NPC-head index
    /// for the `-1` emote (`:227-229`); written only when `emote < 0`.
    pub metadata: i16,
}

impl EmoteBubble {
    /// The removal frame (`MessageBuffer.cs:3402-3408`): id + `255`, no
    /// tail. The C# has no dedicated remove send site — the shape is the
    /// reader's removal arm.
    pub fn removal(id: i32) -> Self {
        Self {
            id,
            anchor_kind: 255,
            anchor_index: 0,
            life_time: 0,
            emote: 0,
            metadata: 0,
        }
    }
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
/// the velocity (`:3549-3551`). Both directions — the NPC twin (100) rides
/// the projectile-support family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerPortalTeleport {
    pub player: u8,
    pub color_index: i16,
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

/// `msg 99 MinionRestTargetUpdate`'s body (`NetMessage.cs:1353-1356`): the
/// player id then `Player.MinionRestTargetPoint` as a `Vector2` (two `float`s,
/// `Player.cs:3006`). Sent when the local point drifts from the last-synced
/// copy (`Main.cs:17678-17681`); the server overrides the id to the sender
/// and relays (`MessageBuffer.cs:3570-3583`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinionRestTarget {
    pub player_id: u8,
    pub x: f32,
    pub y: f32,
}

/// `msg 115 MinionAttackTargetUpdate`'s body (`NetMessage.cs:1357-1360`): the
/// player id then `Player.MinionAttackTargetNPC` as a `short`
/// (`Player.cs:3008`, `-1` = none). Sent on change (`Main.cs:17682-17685`);
/// the server overrides the id and relays (`MessageBuffer.cs:3584-3597`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinionAttackTarget {
    pub player_id: u8,
    pub npc: i16,
}

/// `msg 100 TeleportNPCThroughPortal`'s body (`NetMessage.cs:1361-1369`):
/// `(ushort)number` (the NPC slot), `(short)number4` (the portal color index
/// `owner * 2 + ai[1]`, `PortalHelper.cs:181`), `number2`/`number3` (the
/// destination floats) then `WriteVector2(nPC2.velocity)` — the NPC's
/// *current* velocity, read from the live entity at send time, NOT the
/// destination floats echoed back. The host sends it after the portal-gun
/// (type 602) pulls an NPC through (`PortalHelper.cs:195-204`, followed by a
/// `23` NPCInfo push); the client flips the color parity
/// (`num145 + (num145 % 2 == 0 ? 1 : -1)`), teleports with style 4, overwrites
/// the velocity and zeroes `netOffset` (`MessageBuffer.cs:3598-3610`).
/// Server→client only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpcPortalTeleport {
    pub npc: u16,
    pub color_index: i16,
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

/// `msg 101 UpdateTowerShieldStrengths`'s body (`NetMessage.cs:1371-1376`):
/// the four `NPC.ShieldStrengthTower*` statics as `ushort`s in the written
/// order Solar, Vortex, Nebula, Stardust. Sent by the 629 bolt's shield chip
/// (`Projectile.cs:72944-72978`), the apocalypse start (`WorldGen.cs:86689`)
/// and the join push (`MessageBuffer.cs:869`). The reader clamps each to
/// `0..LunarShieldPowerMax` (= 100, `NPC.cs:95252`) — that clamp is the apply
/// arm's job (`MessageBuffer.cs:3611-3651`); the codec carries the raw
/// ushorts. Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TowerShields {
    pub solar: u16,
    pub vortex: u16,
    pub nebula: u16,
    pub stardust: u16,
}

/// `msg 102 NebulaLevelupRequest`'s body (`NetMessage.cs:1377-1382`):
/// `(byte)number, (ushort)number2, number3, number4` — the picker, the
/// booster's `buffType` and the picker's centre (`Player.cs:35052` passes
/// `Center.X`/`Center.Y` as the two floats). The host relays verbatim with
/// the sender forced into the id byte (`MessageBuffer.cs:3654-3660`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NebulaLevelupRequest {
    pub player_id: u8,
    /// The booster item's `Item.buffType` — 173 life / 176 mana / 179 damage
    /// (`MessageBuffer.cs:3681-3692`'s dust-kind switch).
    pub buff_type: u16,
    pub x: f32,
    pub y: f32,
}

/// `msg 103 MoonlordHorror`'s body (`NetMessage.cs:1383-1386`): the two
/// `NPC` statics as raw ints — `MaxMoonLordCountdown` then `MoonLordCountdown`
/// (`NPC.cs:6031-6033`). Sent by the apocalypse start (`WorldGen.cs:86755`)
/// and the join push (`MessageBuffer.cs:868`); the client stores both
/// (`:3708-3714`). Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoonlordCountdown {
    pub max: i32,
    pub countdown: i32,
}

/// `msg 104 ShopOverride`'s body (`NetMessage.cs:1387-1394`): `(byte)number`
/// the shop slot, `(short)number2` the item type, `(short)number3` the stack
/// (the writer floors a negative at 0, `:1390`), `(byte)number4` the prefix,
/// `(int)number5` the value, `(byte)number6` the `BitsByte{bit0 buyOnce}`.
/// The client rebuilds the open shop's slot (`SetDefaults(type)`, stack,
/// `Prefix(prefix)`, value, buyOnce) when a shop is open and the slot is in
/// range (`MessageBuffer.cs:3715-3735`). Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShopOverride {
    pub slot: u8,
    pub kind: i16,
    pub stack: i16,
    pub prefix: u8,
    pub value: i32,
    pub buy_once: bool,
}

/// `msg 105 GemLockToggle`'s body (`NetMessage.cs:1395-1399`): `(short)number`
/// x, `(short)number2` y, `(bool)(number3 == 1f)` on. The host runs
/// `WorldGen.ToggleGemLock(x, y, on)` (`MessageBuffer.cs:3736-3741`, the
/// reader is `netMode != 1`-gated); the client sends it after consuming the
/// gem (`Player.cs:32547`/`:32559`). Client→host only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GemLockToggle {
    pub x: i16,
    pub y: i16,
    pub on: bool,
}

/// `HalfTypeHelper.Convert(float)` — the FNA/XNA float32→float16 pack
/// (FNA `src/Graphics/PackedVector/HalfTypeHelper.cs`, the code Terraria's
/// `HalfVector2` rides; not in the decompiled trees, which stop at the game
/// assemblies). Verbatim: the round-to-nearest-even mantissa adds and the
/// subnormal/overflow arms, sign in bit 15.
pub fn half_pack(f: f32) -> u16 {
    let i = f.to_bits() as i32;
    let s = (i >> 16) & 0x0000_8000;
    let mut e = ((i >> 23) & 0x0000_00ff) - (127 - 15);
    let mut m = i & 0x007f_ffff;
    if e <= 0 {
        if e < -10 {
            return s as u16;
        }
        m |= 0x0080_0000;
        let t = 14 - e;
        let a = (1 << (t - 1)) - 1;
        let b = (m >> t) & 1;
        m = (m + a + b) >> t;
        (s | m) as u16
    } else if e == 0xff - (127 - 15) {
        if m == 0 {
            (s | 0x7c00) as u16
        } else {
            m >>= 13;
            (s | 0x7c00 | m | if m == 0 { 1 } else { 0 }) as u16
        }
    } else {
        m += 0x0000_0fff + ((m >> 13) & 1);
        if (m & 0x0080_0000) != 0 {
            m = 0;
            e += 1;
        }
        if e > 30 {
            return (s | 0x7c00) as u16;
        }
        (s | (e << 10) | (m >> 13)) as u16
    }
}

/// `HalfTypeHelper.Convert(ushort)` — the float16→float32 expand, verbatim
/// (the subnormal normalize loop, `exp` starting at -14). Inf/NaN expand
/// bit-for-bit (exp field 31 → 0xff, the payload rides the mantissa shift).
pub fn half_unpack(value: u16) -> f32 {
    let mut mantissa = (value & 1023) as u32;
    let mut exp: u32 = 0xffff_fff2; // -14
    let rst = if (value & 0x7c00) == 0 {
        if mantissa != 0 {
            while (mantissa & 1024) == 0 {
                exp = exp.wrapping_sub(1);
                mantissa <<= 1;
            }
            mantissa &= 0xffff_fbff;
            (((value as u32 & 0x8000) << 16) | (exp.wrapping_add(127) << 23))
                | (mantissa << 13)
        } else {
            (value as u32 & 0x8000) << 16
        }
    } else {
        (((value as u32 & 0x8000) << 16)
            | (((((value as u32 >> 10) & 0x1f).wrapping_sub(15)).wrapping_add(127)) << 23))
            | (mantissa << 13)
    };
    f32::from_bits(rst)
}

/// `HalfVector2.PackHelper` (`HalfVector2.cs:94-97` in the FNA tree): x in
/// the low 16 bits, y in the high 16.
pub fn pack_half_vector2(x: f32, y: f32) -> u32 {
    half_pack(x) as u32 | ((half_pack(y) as u32) << 16)
}

/// `HalfVector2.ToVector2` (`HalfVector2.cs:37-43`): the inverse split.
pub fn unpack_half_vector2(packed: u32) -> (f32, f32) {
    (
        half_unpack(packed as u16),
        half_unpack((packed >> 16) as u16),
    )
}

/// `msg 106 PoofOfSmoke`'s body (`NetMessage.cs:1400-1402`): one `uint` —
/// `new HalfVector2(number, number2).PackedValue`, the poof position in two
/// float16s. Sent by the host at the slime-transform poof
/// (`Projectile.cs:14742-14746`), the critter poof (`NPC.cs:79882-79887`)
/// and the wiring lamps (`Wiring.cs:826`, `:2276`, `:2330`, `:2398`); the
/// client replays `Utils.PoofOfSmoke` (`MessageBuffer.cs:3742-3751`,
/// `Utils.cs:1078-1093` — the gore/dust burst). The payload keeps the packed
/// bits so the wire is byte-exact; use [`pack_half_vector2`]/
/// [`unpack_half_vector2`] at the seams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoofOfSmoke {
    pub packed: u32,
}

/// `msg 107 SmartTextMessage`'s body (`NetMessage.cs:769-774`): the three
/// RGB bytes (`number2`/`number3`/`number4`), the `NetworkText`, then the
/// `(short)number5` width limit. The client runs `Main.NewTextMultiline`
/// (`MessageBuffer.cs:3752-3757`, `Main.cs:64638-64642` — the chat line +
/// the sound-12 cue). Server→client only.
#[derive(Clone, Debug, PartialEq)]
pub struct SmartText {
    pub color: Rgb,
    pub text: NetText,
    pub width_limit: i16,
}

/// `msg 108 WiredCannonShot`'s body (`NetMessage.cs:1403-1411`):
/// `(short)number` (damage), `number2` (knockback float), `(short)number3`
/// (tile x), `(short)number4` (tile y), `(short)number5` (the cannon angle
/// 0..8), `(short)number6` (the ammo id) then `(byte)number7` (the owner
/// player). Sent by the host when a wire fires a cannon / bunny cannon
/// (ammo 4/5) owned by a remote player — `WorldGen.ShootFromCannon`
/// redirects instead of spawning (`WorldGen.cs:50689-50692`); the owning
/// client checks `num75 == Main.myPlayer` and replays the shot locally with
/// `fromWire: true` (`MessageBuffer.cs:3758-3773`). Server→client only,
/// addressed to one owner (`remoteClient = owner`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WiredCannonShot {
    pub damage: i16,
    pub knockback: f32,
    pub x: i16,
    pub y: i16,
    pub angle: i16,
    pub ammo: i16,
    pub owner: u8,
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

/// `msg 113 CrystalInvasionStart`'s body (`NetMessage.cs:1432-1435`):
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

/// `msg 116 CrystalInvasionSendWaitTime`'s body (`NetMessage.cs:1436-1438`):
/// one raw `(int)` — `DD2Event._timeLeftUntilSpawningBegins`
/// (`DD2Event.SetEnemySpawningOnHold`, `DD2Event.cs:947-950`, host-only
/// send). The client stores `TimeLeftBetweenWaves`
/// (`MessageBuffer.cs:3861-3866`). Server→client only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrystalInvasionWait {
    pub time_left: i32,
}

/// `PlayerDeathReason` (`DataStructures/PlayerDeathReason.cs`), the shared
/// reason block of `117 PlayerHurtV2` / `118 PlayerDeathV2`
/// (`NetMessage.cs:1441`/`:1449` write it via `_currentPlayerDeathReason`,
/// set by `SendPlayerHurt`/`SendPlayerDeath`, `:2600`/`:2609`). The indices
/// default to -1 and the type/prefix fields to 0 (`:8-20`); the wire is the
/// flag byte then the present fields in bit order (`WriteSelfTo`,
/// `:138-182`; `FromReader`, `:184-221`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerDeathReason {
    /// bit 0 — `(short)` when != -1 (`PlayerDeathReason.cs:141`/`:150-153`).
    pub source_player_index: i16,
    /// bit 1 — `(short)` when != -1 (`:142`/`:154-157`).
    pub source_npc_index: i16,
    /// bit 2 — `(short)` when != -1 (`:143`/`:158-161`).
    pub source_projectile_local_index: i16,
    /// bit 3 — `(byte)` when != -1 (`:144`/`:162-165`). The C# casts the int
    /// to a byte, so -1 would ride as 255; the port keeps the flag test on
    /// the un-cast value (`LegacyDefault`'s 255 is a *present* other-index).
    pub source_other_index: i16,
    /// bit 4 — `(short)` when != 0 (`:145`/`:166-169`).
    pub source_projectile_type: i16,
    /// bit 5 — `(short)` when != 0 (`:146`/`:170-173`).
    pub source_item_type: i16,
    /// bit 6 — `(byte)` when != 0 (`:147`/`:174-177`).
    pub source_item_prefix: u8,
    /// bit 7 — the string when present (`:148`/`:178-181`). The port's wire
    /// strings are literal-mode `BinaryWriter.Write(string)`
    /// (a 7-bit length + UTF-8), same as every other body here.
    pub source_custom_reason: Option<String>,
}

impl Default for PlayerDeathReason {
    /// The C# field initialisers (`PlayerDeathReason.cs:8-22`): -1 indices,
    /// 0 types, no custom reason.
    fn default() -> Self {
        Self {
            source_player_index: -1,
            source_npc_index: -1,
            source_projectile_local_index: -1,
            source_other_index: -1,
            source_projectile_type: 0,
            source_item_type: 0,
            source_item_prefix: 0,
            source_custom_reason: None,
        }
    }
}

impl PlayerDeathReason {
    /// `WriteSelfTo` (`PlayerDeathReason.cs:138-182`): the flag byte then the
    /// present fields in bit order.
    pub fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut b = BitsByte::new();
        b.set(0, self.source_player_index != -1);
        b.set(1, self.source_npc_index != -1);
        b.set(2, self.source_projectile_local_index != -1);
        b.set(3, self.source_other_index != -1);
        b.set(4, self.source_projectile_type != 0);
        b.set(5, self.source_item_type != 0);
        b.set(6, self.source_item_prefix != 0);
        b.set(7, self.source_custom_reason.is_some());
        let mut w = w.bitsbyte(b);
        if b.get(0) {
            w = w.i16(self.source_player_index);
        }
        if b.get(1) {
            w = w.i16(self.source_npc_index);
        }
        if b.get(2) {
            w = w.i16(self.source_projectile_local_index);
        }
        if b.get(3) {
            w = w.u8(self.source_other_index as u8);
        }
        if b.get(4) {
            w = w.i16(self.source_projectile_type);
        }
        if b.get(5) {
            w = w.i16(self.source_item_type);
        }
        if b.get(6) {
            w = w.u8(self.source_item_prefix);
        }
        if let Some(reason) = &self.source_custom_reason {
            w = w.str(reason);
        }
        w
    }

    /// `FromReader` (`PlayerDeathReason.cs:184-221`): absent fields keep the
    /// constructor defaults (-1 / 0 / null).
    pub fn read_from(r: &mut NetReader) -> Option<Self> {
        let b = r.bitsbyte()?;
        Some(Self {
            source_player_index: if b.get(0) { r.i16()? } else { -1 },
            source_npc_index: if b.get(1) { r.i16()? } else { -1 },
            source_projectile_local_index: if b.get(2) { r.i16()? } else { -1 },
            // The wire byte widens back to the int field; an absent flag
            // keeps -1, a present 255 stays 255 (`LegacyDefault`).
            source_other_index: if b.get(3) { r.u8()? as i16 } else { -1 },
            source_projectile_type: if b.get(4) { r.i16()? } else { 0 },
            source_item_type: if b.get(5) { r.i16()? } else { 0 },
            source_item_prefix: if b.get(6) { r.u8()? } else { 0 },
            source_custom_reason: if b.get(7) { Some(r.str()?) } else { None },
        })
    }
}

/// `msg 117 PlayerHurtV2`'s body (`NetMessage.cs:1439-1446`): `(byte)number`,
/// the death reason, `(short)number2` damage, `(byte)(number3 + 1f)`
/// direction, `(byte)number4` the `BitsByte{bit0 critical, bit1 pvp}` packed
/// by `SendPlayerHurt` (`:2601-2604`), `(sbyte)number5` the immunity-cooldown
/// counter. The reader subtracts the 1 back off the direction
/// (`MessageBuffer.cs:3874`) and unpacks bit0/bit1 (`:3875-3877`).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerHurtV2 {
    pub player_id: u8,
    pub reason: PlayerDeathReason,
    pub damage: i16,
    /// The wire value is `hitDirection + 1` (`NetMessage.cs:1443`); -1 rides
    /// as 0, +1 as 2. Stored un-offset like `NpcStrike.hit_direction`.
    pub hit_direction: i8,
    pub crit: bool,
    pub pvp: bool,
    /// `Player.Hurt`'s `cooldownCounter` (`ImmunityCooldownID`), an `sbyte`
    /// on the wire (`NetMessage.cs:1445`, `MessageBuffer.cs:3878`).
    pub cooldown_counter: i8,
}

impl PlayerHurtV2 {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut b = BitsByte::new();
        b.set(0, self.crit);
        b.set(1, self.pvp);
        let w = w.u8(self.player_id);
        let w = self.reason.write_to(w);
        w.i16(self.damage)
            .u8(self.hit_direction.wrapping_add(1) as u8)
            .bitsbyte(b)
            .i8(self.cooldown_counter)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let player_id = r.u8()?;
        let reason = PlayerDeathReason::read_from(r)?;
        let damage = r.i16()?;
        let hit_direction = (r.u8()? as i8).wrapping_sub(1);
        let b = r.bitsbyte()?;
        let cooldown_counter = r.i8()?;
        Some(Self {
            player_id,
            reason,
            damage,
            hit_direction,
            crit: b.get(0),
            pvp: b.get(1),
            cooldown_counter,
        })
    }
}

/// `msg 118 PlayerDeathV2`'s body (`NetMessage.cs:1447-1453`):
/// `(byte)number`, the death reason, `(short)number2` damage,
/// `(byte)(number3 + 1f)` direction, `(byte)number4` the
/// `BitsByte{bit0 pvp}` packed by `SendPlayerDeath` (`:2610-2612`).
/// Unlike 117 there is no crit bit and no trailing cooldown sbyte. The
/// reader subtracts the 1 back off the direction and unpacks bit0 as pvp
/// (`MessageBuffer.cs:3896-3897`) before `Player.KillMe` (`:3898`).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerDeathV2 {
    pub player_id: u8,
    pub reason: PlayerDeathReason,
    pub damage: i16,
    /// The wire value is `hitDirection + 1` (`NetMessage.cs:1451`); -1 rides
    /// as 0, +1 as 2. Stored un-offset like `PlayerHurtV2.hit_direction`.
    pub hit_direction: i8,
    pub pvp: bool,
}

impl PlayerDeathV2 {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        let mut b = BitsByte::new();
        b.set(0, self.pvp);
        let w = w.u8(self.player_id);
        let w = self.reason.write_to(w);
        w.i16(self.damage)
            .u8(self.hit_direction.wrapping_add(1) as u8)
            .bitsbyte(b)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        let player_id = r.u8()?;
        let reason = PlayerDeathReason::read_from(r)?;
        let damage = r.i16()?;
        let hit_direction = (r.u8()? as i8).wrapping_sub(1);
        let b = r.bitsbyte()?;
        Some(Self {
            player_id,
            reason,
            damage,
            hit_direction,
            pvp: b.get(0),
        })
    }
}

/// `msg 120 Emoji`'s body (`NetMessage.cs:1454-1457`): `(byte)number` the
/// player, `(byte)number2` the emote id. The host validates the id against
/// `EmoteID.Count` = 151 (`UI/EmoteID.cs:317`) before `EmoteBubble.NewBubble`
/// with the 360-tick lifetime (`MessageBuffer.cs:3912-3916`); the codec
/// keeps the byte verbatim either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emoji {
    pub player_id: u8,
    pub emote_id: u8,
}

/// `msg 85 QuickStackChests`'s client→host body (`NetMessage.cs:1205-1210`,
/// `QuickStacking.WriteNetInventorySlots`, `QuickStacking.cs:299-306`):
/// `(int)` the slot count, then one `(short)` `PlayerItemSlotID.SlotId` per
/// packed quick-stackable slot, then `(byte)number` the smartStack flag.
/// The host reads it into a scratch `SourceInventory`
/// (`QuickStacking.ReadNetInventory`, `:308-321`) and runs
/// `QuickStackToNearbyChests` (`MessageBuffer.cs:3265`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickStackRequest {
    /// The `PlayerItemSlotID.SlotId` list (the packed quick-stackable slots).
    pub slots: Vec<i16>,
    /// `smartStack` — the category-aware second pass (`QuickStacking.cs:430`).
    pub smart_stack: bool,
}

/// `msg 85 QuickStackChests`'s host→client body (`NetMessage.cs:1213`,
/// `QuickStacking.WriteBlockedChestList`, `QuickStacking.cs:323-330`):
/// `(int)` the blocked-chest count, then one `(ushort)` chest index per
/// blocked chest. The client runs `IndicateBlockedChests`
/// (`MessageBuffer.cs:3269`, the red pulse over each chest).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickStackBlocked {
    pub chests: Vec<u16>,
}

/// `msg 94 DevCommands`'s body (`NetMessage.cs:1327-1332`): the command
/// `string` (`text.ToString()`), `(int)number`, `(float)number2`,
/// `(float)number3`. The reader consumes all four fields, then gates on
/// `DebugOptions.enableDebugCommands` (`MessageBuffer.cs:3465-3472`) before
/// the `/showdebug` / `/randomizeprojslots` / `/setserverping` /
/// `/quickload-*` switch (`:3473-3516`). Both directions — the host's
/// `DebugOptions.SyncToJoiningPlayer` push (`DebugOptions.cs:64-72`) and the
/// quickload probe/response relay.
#[derive(Clone, Debug, PartialEq)]
pub struct DevCommands {
    pub command: String,
    pub number: i32,
    pub number2: f32,
    pub number3: f32,
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

/// `msg 130 FishOutNPC`'s body (`NetMessage.cs:1528-1532`): `(ushort)` tile
/// x, `(ushort)` tile y, `(short)` npc type. The host multiplies both
/// coordinates by 16 and `NPC.NewNPC`s the type (`MessageBuffer.cs:4104-4110`);
/// type 682 (Red Slime) also flips `NPC.unlockedSlimeRedSpawn` once and
/// re-pushes `7 WorldData` (`:4095-4103`). The wire carries TILE coordinates
/// — the sender already divided by 16 (`Player.cs:52425`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FishOutNpc {
    pub tile_x: u16,
    pub tile_y: u16,
    pub npc_type: i16,
}

/// `msg 131 TamperWithNPC`'s body (`NetMessage.cs:1533-1541`): `(ushort)` npc
/// slot, `(byte)` kind; kind 1 appends `(int)` immune time + `(short)`
/// fromWho. The client's kind-1 arm calls `GetImmuneTime(time, fromWho)`
/// (`MessageBuffer.cs:4128-4131`); a slot ≥ `Main.maxNPCs` tames a throwaway
/// `new NPC()` instead (`:4127`). The only send site is the 485 statue-break
/// 582 spawn's `GetImmuneTime(-1, 20)` (`WorldGen.cs:48373-48376`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TamperWithNpc {
    pub npc: u16,
    pub kind: u8,
    /// Present only when `kind == 1` (the `(int)` time + `(short)` fromWho).
    pub immune: Option<(i32, i16)>,
}

/// `msg 134 UpdatePlayerLuckFactors`'s body (`NetMessage.cs:1555-1567`):
/// `(byte)number`, then the eight `Player` luck fields in writer order —
/// `ladyBugLuckTimeLeft` (int, `Player.cs:3084`), `torchLuck` (float,
/// `:3062`), `luckPotion` (byte, `:1134`), `HasGardenGnomeNearby` (bool,
/// `:2076`), `brokenMirrorBadLuck` (bool, `:2078`),
/// `equipmentBasedLuckBonus` (float, `:2774`), `coinLuck` (float, `:3092`),
/// `kiteLuckLevel` (byte, `:3094`). The reader mirrors the order
/// (`MessageBuffer.cs:4166-4174`) and applies + `RecalculateLuck` (`:4188`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UpdatePlayerLuckFactors {
    pub player_id: u8,
    pub ladybug_luck_time_left: i32,
    pub torch_luck: f32,
    pub luck_potion: u8,
    pub has_garden_gnome_nearby: bool,
    pub broken_mirror_bad_luck: bool,
    pub equipment_based_luck_bonus: f32,
    pub coin_luck: f32,
    pub kite_luck_level: u8,
}

impl UpdatePlayerLuckFactors {
    fn write_to(&self, w: NetWriter) -> NetWriter {
        w.u8(self.player_id)
            .i32(self.ladybug_luck_time_left)
            .f32(self.torch_luck)
            .u8(self.luck_potion)
            .bool(self.has_garden_gnome_nearby)
            .bool(self.broken_mirror_bad_luck)
            .f32(self.equipment_based_luck_bonus)
            .f32(self.coin_luck)
            .u8(self.kite_luck_level)
    }

    fn read_from(r: &mut NetReader) -> Option<Self> {
        Some(Self {
            player_id: r.u8()?,
            ladybug_luck_time_left: r.i32()?,
            torch_luck: r.f32()?,
            luck_potion: r.u8()?,
            has_garden_gnome_nearby: r.bool()?,
            broken_mirror_bad_luck: r.bool()?,
            equipment_based_luck_bonus: r.f32()?,
            coin_luck: r.f32()?,
            kite_luck_level: r.u8()?,
        })
    }
}

/// `msg 136 SyncCavernMonsterType`'s body (`NetMessage.cs:1572-1582`): the
/// 2×3 `NPC.cavernMonsterType` grid (`NPC.cs:6585`, allocated `new int[2, 3]`
/// at `:95268`) written row-major as six `(ushort)`s — outer loop `i < 2`,
/// inner `j < 3`. The client stores them verbatim (`MessageBuffer.cs:4210-4216`);
/// the join push is `MessageBuffer.cs:870`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CavernMonsterTypes {
    /// `[i][j]`, row-major — the C#'s `int[2, 3]` narrowed to the wire's
    /// `ushort` (the values are npc types, `NPC.cs:18123-18129`).
    pub types: [[u16; 3]; 2],
}

/// `msg 137 RequestNPCBuffRemoval`'s body (`NetMessage.cs:1583-1586`):
/// `(short)` npc slot + `(ushort)` buff type. The host gates the slot to
/// `0..Main.maxNPCs` then runs `RequestBuffRemoval` (`MessageBuffer.cs:4222-4227`);
/// the client sends it from `NPC.RequestBuffRemoval` behind the
/// `BuffID.Sets.CanBeRemovedByNetMessage` gate (`NPC.cs:89987-89998`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcBuffRemovalRequest {
    pub npc: i16,
    pub buff_type: u16,
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
}

/// `msg 150 SpectatePlayer`'s body (`NetMessage.cs:1639-1642`):
/// `(byte)number` the spectating player, `(short)number2` the target —
/// -1 stops spectating (`MessageBuffer.cs:4372-4378`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpectatePlayer {
    pub player_id: u8,
    pub target: i16,
}

/// `msg 153 NPCDebuffDamage`'s body (`NetMessage.cs:1646-1649`): `(byte)` npc
/// slot + `(short)` damage. BOTH sides apply `GetHurtByDebuff` and the host
/// relays to the others (`MessageBuffer.cs:4410-4416`); the damage lands on
/// `realLife` when set, the immortal gate skips the subtraction, and a
/// life ≤ 0 is floored to 1 + a 9999 `StrikeNPCNoInteraction`
/// (`NPC.cs:91864-91888`). Host-sent only (the electric-eel pulse,
/// `NPC.cs:91853-91860`), so it is `S2c`-only like `97`/`98`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcDebuffDamage {
    pub npc: u8,
    pub damage: i16,
}

/// `msg 139 SetCountsAsHostForGameplay`'s body
/// (`NetMessage.cs:1587-1590`): the player byte and `number2 == 1f` as a
/// bool. The reader stores `Main.countsAsHostForGameplay[id]` when
/// `netMode != 2` (`MessageBuffer.cs:4231-4236`) — the flag gates the
/// host-side gameplay authority for a co-op peer. W4-g51 f21 cross-check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountsAsHost {
    pub player_id: u8,
    pub counts_as_host: bool,
}

/// `msg 146 ShimmerActions`'s body (`NetMessage.cs:1611-1623`): the kind
/// byte, then kind 0 → `WriteVector2(number2, number3)` (the shimmer FX
/// position); kind 1 → the Vector2 + `(int)number4` (the coin amount).
/// Any other kind writes NOTHING after the byte (the switch has no
/// default). The reader (`MessageBuffer.cs:4313-4325`) mirrors the two
/// arms; unknown kinds read nothing. W4-g51 f21 cross-check.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShimmerAction {
    /// kind 0 — `WorldItem.ShimmerEffect(position)`
    /// (`MessageBuffer.cs:4316`).
    Effect { x: f32, y: f32 },
    /// kind 1 — `AddCoinLuck(position, amount)` (`:4320-4322`).
    CoinLuck { x: f32, y: f32, amount: i32 },
    /// Any other kind byte — no tail on either side.
    Unknown(u8),
}

/// `msg 160 ItemPosition`'s body (`NetMessage.cs:1666-1672`): the item
/// slot `(short)` then `WriteVector2(worldItem.position)`. The reader
/// (`MessageBuffer.cs:4466-4468`) snaps `Main.item[slot].position` when
/// `netMode != 2` — the section-sweep position correction, no velocity.
/// W4-g51 f21 cross-check.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemPosition {
    pub slot: i16,
    pub x: f32,
    pub y: f32,
}

// ---- W4-g51 FINAL: the TE-family bodies (f21 codecs lifted) ----

/// One `(short)type + (byte)prefix + (short)stack` item triple — the shape
/// every tile-entity `WriteExtraData` uses for a non-air slot
/// (`TEItemFrame.cs:42-44` and its five twins). The TryPlacing quadruplet's
/// wire item; the doll/hat-rack `WriteItem` order is STACK BEFORE PREFIX
/// (`write_slot`/`read_slot` below). W4-g51 FINAL (f21's `TeItem`).
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
    pub fn is_air(self) -> bool {
        self.kind <= 0 || self.stack <= 0
    }
}

/// `msg 89 ItemFrameTryPlacing`'s body (`NetMessage.cs:1297-1306`): the
/// tile as two `(short)`s, the held item's `(short)` type, its prefix
/// byte, and the `(short)` stack (`number5` — always 1 at the send site,
/// `TEItemFrame.cs:150`). The player/slot indices (`number4`/`number3`)
/// select the item on the WRITER side only; they never ride the wire.
/// The 123 WeaponsRack / 133 FoodPlatter / 149 DeadCellsJar twins share
/// this exact shape (`NetMessage.cs:1479-1488`/`:1545-1554`/`:1629-1638`)
/// — f21's byte-equality pin. W4-g51 FINAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TryPlacingItem {
    pub x: i16,
    pub y: i16,
    pub kind: i16,
    pub prefix: u8,
    pub stack: i16,
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
/// W4-g51 FINAL (f21's `DollSyncData`).
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
/// W4-g51 FINAL.
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
/// (`MessageBuffer.cs:3942-3977`) forces the player to the sender,
/// refuses an occupied entity (`TileEntity.IsOccupied`) and relays; the
/// client clears or sets the interaction anchor. W4-g51 FINAL.
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
/// W4-g51 FINAL.
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

/// `msg 156 TELeashedEntityAnchorPlaceItem`'s body
/// (`NetMessage.cs:1654-1658`): three `(short)`s — the anchor tile x, y
/// and the item type. The reader (`MessageBuffer.cs:4439-4449`) builds
/// the `Point16(x, y)`, looks up the `TELeashedEntityAnchorWithItem` and
/// calls `InsertItem(itemType)` (which respawns the leashed entity,
/// `TELeashedEntityAnchorWithItem.cs:32-36`). W4-g51 FINAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeashedAnchorPlaceItem {
    pub x: i16,
    pub y: i16,
    pub item_type: i16,
}

/// `121`'s shared writer (`NetMessage.cs:1458-1474`): the header then the
/// command-keyed tail — `TEDisplayDoll.WriteData` (`TEDisplayDoll.cs:780-800`)
/// for a real doll, `WriteDummySync` (`:824-833`) for the fallback.
/// W4-g51 FINAL (f21's `write_doll_sync`).
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
/// [`DollSyncData`]). W4-g51 FINAL (f21's `read_doll_sync`).
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

/// `ActiveSections.SectionInactiveTime` (`DataStructures/ActiveSections.cs:65`)
/// — the tick window a checked section stays "active" for `IsSectionActive`.
pub const SECTION_INACTIVE_TIME: u32 = 60;

/// `RemoteClient.IsSectionActive`'s predicate (`RemoteClient.cs:220-224`):
/// `TileSectionsCheckTime[x, y] + SectionInactiveTime >= Main.GameUpdateCount`
/// (the C# uint add wraps; `wrapping_add` matches).
pub fn section_active_at(last_check: u32, now: u32) -> bool {
    last_check.wrapping_add(SECTION_INACTIVE_TIME) >= now
}

/// `RemoteClient.SectionRange` over one slot's latch (`RemoteClient.cs:191-218`)
/// — the four-corner test the `SendData(20)` relay arm runs per client
/// (`NetMessage.cs:1712`). Free fn so the fan-out can call it under the slots
/// lock it already holds.
fn slot_section_range(c: &Slot, size: i32, first_x: i32, first_y: i32) -> bool {
    // The four corner offsets, the C#'s i = 0..3 order.
    const CORNERS: [(i32, i32); 4] = [(0, 0), (1, 0), (0, 1), (1, 1)];
    if c.sections_w == 0 {
        return false;
    }
    for (dx, dy) in CORNERS {
        let sx = section_x(first_x + dx * size);
        let sy = section_y(first_y + dy * size);
        // The C# indexes TileSections directly (a negative coordinate throws
        // inside SendData's try); the port refuses out-of-grid.
        if sx < 0 || sy < 0 {
            continue;
        }
        let i = (sy * c.sections_w + sx) as usize;
        if c.sections.get(i) == Some(&true) {
            return true;
        }
    }
    false
}

/// `RemoteClient.IsSectionActive` over one slot's check-time grid
/// (`RemoteClient.cs:220-224`), under a held slots lock. Coords clamp to
/// `[0, maxSections]` (`ActiveSections.cs:53-56`); an unallocated or
/// out-of-grid slot reads inactive.
fn slot_section_active(c: &Slot, sx: i32, sy: i32, now: u32) -> bool {
    if c.sections_w == 0 {
        return false;
    }
    let (sx, sy) = (sx.max(0), sy.max(0));
    let i = (sy * c.sections_w + sx) as usize;
    c.section_check_time
        .get(i)
        .is_some_and(|&t| section_active_at(t, now))
}

/// The entity context the `SendData` relay arms read off `Main.npc[number]` /
/// `Main.projectile[number]` (`NetMessage.cs:1751-1800`). The port keeps the
/// entities beside the server, so the caller passes the two fields the arms
/// need: the culling exemption and the entity's `NetSectionCoordinates`
/// (`NPC.cs:6899` / `Projectile.cs:602` — `GetSectionX((int)position.X >> 4)`).
#[derive(Clone, Copy, Debug)]
pub enum RelayCtx {
    /// `case 28` (`NetMessage.cs:1751-1762`): `nPC5.life <= 0` bypasses the
    /// section filter; otherwise the client's section must be active.
    Strike { dead: bool, section: (i32, i32) },
    /// `case 27` (`NetMessage.cs:1774-1800`): `flag3` (`type == 12 ||
    /// Main.projPet[type] || aiStyle == 11 || netImportant`, `:1777`) bypasses
    /// the section filter. The per-projectile skipped-flag motion rides
    /// `net_proj_sync::ProjSync::note_broadcast` at the same site.
    Projectile {
        ignores_sections: bool,
        section: (i32, i32),
    },
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
