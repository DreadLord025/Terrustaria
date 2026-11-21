# 17 — Netcode (NetMessage / sockets / Netplay)

> Wave 6 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The whole multiplayer transport and synchronisation layer, split across two trees that
do not correspond file-for-file.

**C# (2 sides, 4 big files):**
- `decompiled/Terraria/ID/MessageID.cs` (346 lines) — 163 `const byte` message ids
  (`NeverCalled = 0` … `DamageNPCAck = 162`, `Count = 163`, `MessageID.cs:340-346`).
- `decompiled/Terraria/NetMessage.cs` (2 923 lines) — the single writer. `TrySendData`
  (:68), `SendData` (:81) with a 141-arm `switch (msgType)` (:130-1844) that serialises
  every body, plus the client-side `SendPacketToServer` (:1845) / `SendPacket` (:1850)
  and the join bundle `greetPlayer` (:2757), `SyncConnectedPlayer` (:2806),
  `SyncOnePlayer` (:2885), `SyncOnePlayer_ItemArray` (:2962).
- `decompiled/Terraria/MessageBuffer.cs` (4 510 lines) — the single reader.
  `GetData` (:125) with the accept gate (:158-173), the 159-arm `switch (b)`
  (:179-4471), and the tile codec `CompressTileBlock` entry points it calls into.
- `decompiled/Terraria/Netplay.cs` (798 lines) — client state machine
  (`InnerClientLoop` :459-577, states 0→6 then 10), `Initialize` (:729, the
  `whoAmI = i` slot assignment at :740/:744), idle/timeout sweep.
- `decompiled/Terraria/Net/*` (12 files): `NetManager.cs` (176), `NetPacket.cs` (45),
  `Ping.cs` (44), `RemoteAddress.cs`, `TcpAddress.cs`, `SteamAddress.cs`,
  `WeGameAddress.cs`, `AddressType.cs`, `ServerMode.cs`, `LegacyNetBufferPool.cs`,
  `NetModule.cs`, `NetGroupInfo.cs`.
- `decompiled/Terraria/GameContent/NetModules/*.cs` — 12 modules (Liquid, Text, Ping,
  Ambience, Bestiary, CreativeUnlocks, CreativePowers, CreativeUnlocksReport,
  CreativePowerPermissions, TeleportPylon, Particles, Debug).

**Rust (3 files, one real stack — not glue):**
- `crates/tr-app/src/net.rs` (3 258 lines) — a real TCP net stack: `TcpListener`
  (`Server`, :2572-2579) and `TcpStream` (`Client::connect`, :2866-2868) with a
  length-prefixed frame (`MIN_PACKET = 3` / `MAX_PACKET = u16::MAX`, :155-156),
  `C2s` (:1225) / `S2c` (:1289) payload enums, `ServerEvent`/`ClientEvent`, and the
  25-message-id `id` module (:56-95).
- `crates/tr-app/src/app/netglue.rs` (2 820 lines) — the Game↔net adapter: accept gate,
  host/client event loops, world streaming, prediction smoothing, slot rebuilds.
- `crates/tr-app/src/app/netmodules.rs` (495 lines) — 11 of the 12 NetModules.

Module 05's `netglue.rs` note is confirmed accurate but incomplete: the transport,
codec, id table and both connection halves live in `net.rs`, not in glue.

## Key entities

- `decompiled/Terraria/ID/MessageID.cs::MessageID` (1-346) — the id table; `Count = 163`
  is the only version guard on the read side (`MessageBuffer.cs:139-142` returns
  silently for `b >= Count`).
- `decompiled/Terraria/NetMessage.cs::SendData` (81-1844) — the write path; the body
  arms are the payload shapes.
- `decompiled/Terraria/NetMessage.cs::CheckBytes` (2486-2557) — the read pump:
  pulls 2-byte-LE-length frames, calls `GetData`, resets the reader position.
- `decompiled/Terraria/MessageBuffer.cs::GetData` (125-4493) — the read dispatch, the
  accept gate (:158-173) and the 159 arms.
- `decompiled/Terraria/Netplay.cs::InnerClientLoop` (459-577) — the connect /
  world-request / spawn / in-world state machine.
- `decompiled/Terraria/Net/Ping.cs::Ping.Update` (23-35) — ping measurement
  (`SendData(154)` every 250 ms, `CurrentPing` from a stopwatch).
- `crates/tr-app/src/net.rs::id` (56-95) — 25 C#-id-mapped constants + 5 internal ids
  201-205 (`INT_EDIT`, `INT_PICKUP`, `INT_JOINED`, `INT_LEFT`, `INT_TIME`).
- `crates/tr-app/src/net.rs::client_state` / `::slot_state` (103-146) — the two state
  machines (client 0,1,2,3,4,5,6,10; slot 0,-1,1,2,3,10).
- `crates/tr-app/src/net.rs::Server` (2572) / `::Client` (2866) — TCP listener/client,
  frame encode/decode.
- `crates/tr-app/src/app/netglue.rs::host_gate` (151-169) — the ported accept gate.
- `crates/tr-app/src/app/netglue.rs::client_event` (775-1128) — client receive side.
- `crates/tr-app/src/app/netglue.rs::net_tick` (1426) / `::net_interp_tick` (1620) —
  send cadence and remote-entity smoothing.
- `crates/tr-app/src/app/netmodules.rs::module_id` (34-45) — the 11 ported modules.

## Dependencies

- **Calls into:** `NetMessage` ← every gameplay subsystem (census below).
  `MessageBuffer.GetData` ← `Main`/`Player`/`NPC`/`Item`/`WorldGen`/`Wiring`/
  `TileEntities`. `Netplay` ← `Main.Update`, `NetManager` (ReLogic sockets), `Ping`.
- **Called by:** `Main.Update`/`UpdateInMainThread` → `Netplay.UpdateInMainThread`
  (`Netplay.cs:747-758`); the socket read callbacks → `NetMessage.ReceiveBytes`
  (`:2460`) → `CheckBytes` (`:2486`) → `GetData`.
- **Shared data structures:** `Main.netMode` (0 SP / 1 client / 2 server) gates every
  arm; `NetMessage.buffer[257]` (`MessageBuffer` per slot, 256 = the local client);
  `Netplay.Clients[256]`; `RemoteClient.State`; `Main.player[i]` slot identity;
  `BitsByte` flags (`net.rs::BitsByte`, :162).

## Connection flow (C# → Rust)

| Step | C# | Rust |
| --- | --- | --- |
| connect | `Netplay.Connect` (:425-457) sets `Main.netMode = 1`, `WorldPreparationState.AwaitingData`, spawns the loop thread | `net::Client::connect` (`net.rs:2867`) |
| hello | state 0→1, `SendData(1)` = `"Terraria" + 326` (`Netplay.cs:469-474`, `NetMessage.cs:132-134`) | `id::CONNECT = 1`, `CONNECT_VERSION = "Terraria326"` (`net.rs:36,57`) |
| player data | state 2 `"Net.SendingPlayerData"`, `SendData(3)` (`Netplay.cs:476-479`) | `PLAYER_ID = 3` (:59) |
| world request | state 3, `SendData(6)` (`Netplay.cs:480-483`) | `WORLD_REQUEST = 6` (:62) |
| world data | state 4→5, clears the world, streams `SendData(7)` + `10` sections + `9` progress (`Netplay.cs:484-516`) | `WORLD_INFO = 7`, `TILE_BLOCK = 10`, `STATUS = 9` (:63,67,65) |
| spawn | state 6, `SendData(8, …SpawnX, SpawnY, team)` (`Netplay.cs:511-516`) | `SPAWN_REQUEST = 8` (:64) |
| in world | state 10, `SendData(12)` (`Netplay.cs:517-519`+) | `PLAYER_SPAWN = 12` (:76), `client_state::IN_WORLD = 10` (:124) |

Rust mirrors the machine one-for-one — including the client's `WORLD_LOADING` and the
host's `slot_state` (-1 = awaiting password, `net.rs:135`). Only one order difference:
the port never enters a `worldCleared` arm (it swaps `App.world` in place, see
ANALYSIS.md:7288-7293).

## Port status — message-id map (the centrepiece)

`SendData` writes **140** of the 163 ids; the 23 without a body arm are
0, 6, 15, 25, 26, 37, 44, 49, 67, 75, 82, 83, 93, 111, 114, 129, 138, 143, 144, 145,
148, 154, 162. `GetData` reads **159**; the only three it never dispatches are
**138, 145, 148** (deprecated, removed ids). Everything else is read somewhere.
`net.rs` ports **25** ids + 5 internal ones (as researched; S1-E-27/S1-I-5 grew
that, and S1-I-1 wave A added the join bundle's 11 — see the map's per-row
notes).

| id | C# name | Rust | Evidence |
| --- | --- | --- | --- |
| 0 | NeverCalled | — (dead) | gate-buster only, `MessageBuffer.cs:169` |
| 1 | Hello | `id::CONNECT` | `net.rs:57`; body `"Terraria" + 326` (`NetMessage.cs:133`) |
| 2 | Kick | `id::DISCONNECT` | `net.rs:58`; `text.Serialize` + `Kicked = true` (`NetMessage.cs:135-142`) |
| 3 | PlayerInfo | `id::PLAYER_ID` | `net.rs:59`; `(byte)remoteClient` + false (`:143-145`) |
| 4 | SyncPlayer | `id::PLAYER_INFO` | `net.rs:60`; the long arm `:147-197` |
| 5 | SyncEquipment | `id::PLAYER_INVENTORY` | `net.rs:61`; accepted and dropped host-side (ANALYSIS.md:7261-7266) |
| 6 | RequestWorldData | `id::WORLD_REQUEST` | `net.rs:62`; no body |
| 7 | WorldData | `id::WORLD_INFO` | `net.rs:63`; many fields ride defaults (ANALYSIS.md:7267-7274) |
| 8 | SpawnTileData | `id::SPAWN_REQUEST` | `net.rs:64` |
| 9 | StatusTextSize | `id::STATUS` | `net.rs:65` |
| 10 | TileSection | `id::TILE_BLOCK` | `net.rs:67` |
| 11 | TileFrameSection | NOT-FOUND | deprecated (`MessageID.cs:29-30`); no `SendData` arm |
| 12 | PlayerSpawn | `id::PLAYER_SPAWN` | `net.rs:76` |
| 13 | PlayerControls | `id::PLAYER_STATE` | `net.rs:78`; arm `NetMessage.cs:446-513` |
| 14 | PlayerActive | `id::PLAYER_ACTIVE` | `net.rs:66` |
| 15 | Unknown15 | NOT-FOUND | deprecated (`MessageID.cs:38-39`) |
| 16 | PlayerLifeMana | `id::PLAYER_HP` | `net.rs:73` |
| 17-19 | TileManipulation, SetTime, ToggleDoorState | NOT-FOUND | 17 superseded by 20 (`MessageID.cs:43`); 18's clock rides internal `205` (ANALYSIS.md:7275-7278) |
| 20 | AreaTileChange | `id::SECTION_RANGE` | `net.rs:72`; arm `:543-644` |
| 21 | SyncItem | `id::ITEM_DROP` | `net.rs:79`; auto-converts to 151 when air (`NetMessage.cs:89-100`) |
| 22 | ItemOwner | `id::ITEM_OWNER` | `net.rs:80`; reservation expiry (`:675-685`) |
| 23 | SyncNPC | `id::NPC_INFO` | `net.rs:82`; arm `:686-748` |
| 24 | UnusedMeleeStrike | NOT-FOUND | dead (`MessageID.cs:57`) |
| 25-26 | Unused25 / Unused26 | NOT-FOUND | deprecated; 25 for NetTextModule (`MessageID.cs:59-60`) |
| 27 | SyncProjectile | `id::PROJECTILE` | `net.rs:70`; arm `:780-841` |
| 28 | DamageNPC | `id::NPC_STRIKE` | `net.rs:69`; cascades a 23 when `spawnNeedsSyncing` (`:102-107`) |
| 29 | KillProjectile | `id::PROJECTILE_DEAD` | `net.rs:71` |
| 30-36 | TogglePVP / RequestChestOpen (31) / SyncChestItem (32) / SyncPlayerChest (33) / ChestUpdates / PlayerHeal / SyncPlayerZone | `id::TOGGLE_PVP` / `id::CHEST_OPEN_REQUEST` / `id::CHEST_ITEM` / `id::PLAYER_CHEST` | `net.rs` — 30 has a body + codec since S1-I-1 wave A (`NetMessage.cs:856-859`, `MessageBuffer.cs:1851-1867`; dispatcher parked for I-4); 31/32/33 ported by S1-E-27 (`MessageBuffer.cs:1868-1982`, the writer arms `NetMessage.cs:864-911`): the client asks with `31`, the host answers `SendChestContentsTo` (`155` + one `32` per slot), `33` to the asker and `80` to the others, the rigged chest also firing `59`; 32 relays to the other clients (`:1915`). 30 `TogglePVP`, 34 `ChestUpdates`, 35 `PlayerHeal` and 36 `SyncPlayerZone` remain NOT-FOUND (no `SendData` arm of 37's shape) |
| 37 | RequestPassword | `id::PASSWORD_REQUEST` | `net.rs:74` |
| 38 | SendPassword | `id::PASSWORD` | `net.rs:75`; the one message the gate lets through at State -1 |
| 39-48 | ReleaseItemOwnership … OpenSignRequest (46) / OpenSignResponse (47) / LiquidUpdate | `id::PLAYER_MANA` (42) / `id::TEAM_CHANGE` (45) / `id::SIGN_REQUEST` / `id::SIGN_DATA` | `net.rs` — 42 (mana, `NetMessage.cs:961-965`) and 45 (`:970-974`) have bodies + codecs since S1-I-1 wave A (dispatchers parked for I-4); 46/47 ported by S1-E-27 (`MessageBuffer.cs:2366-2405`, writer `NetMessage.cs:979-986`): `Sign.ReadSign`/`TextSign` live in the new `signs.rs`, the record is rewritten on the host and relayed to the other clients when the text changed (`:2398`). 39-45 and 48 remain NOT-FOUND: no `SendData` arm; 42/68 gate-whitelisted; 48 deprecated for NetLiquidModule (`MessageID.cs:108-109`) |
| 49 | InitialSpawn | `id::INITIAL_SPAWN` | `net.rs` — the C# meaning (the client spawns in, `MessageBuffer.cs:2447-2453`) is now the port's too: `S2c::InitialSpawn` drives `State 6 → 10` + `12 PlayerSpawn` with `SpawningIntoWorld` |
| 50-81 | PlayerBuffs (50) … CombatTextInt | `id::PLAYER_BUFFS` / `id::ADD_NPC_BUFF` / `id::NPC_BUFFS` / `id::HIT_SWITCH` / `id::CHEST_NAME` / `id::PLAYER_CHEST_INDEX` | `net.rs` — S1-E-27 ported 50 (`MessageBuffer.cs:2454-2480`: the owner's own list is skipped `:2461-2464`, a peer's lands with every time forced to 60 `:2470-2471`), 53/54 (`:2565-2593`: the host applies `AddBuff(quiet: true)` and pushes the full list back), 59 (`:2682-2693`: `Wiring.HitSwitch` then a relay to every other client), 69 (`:3059-3078`: the rename, `Chest.CreateWorldChest` when the slot is absent) and 80 (`:3216-3227`, the `-3..8000` range gate). S1-I-1 wave A added bodies + codecs for 60 (`id::NPC_HOUSE`, `NetMessage.cs:1067-1072`), 72 (`id::TRAVEL_SHOP`, 40 shorts, `:1134-1141`) and 74 (`id::ANGLER_QUEST`, `:1145-1148`) — dispatchers parked for I-4. The rest stay NOT-FOUND: 77 `SendTemporaryAnimation` (:2590); 79 `SendObjectPlacement` (:2573) |
| 82 | NetModules | `id::NET_MODULE` | `net.rs:68`; the C# arm is `switch (b4)` (`NetMessage.cs:746`) |
| 83 | Unused83 | NOT-FOUND | deprecated (`MessageID.cs:179-180`) |
| 84-96 | PlayerStealth … TeleportPlayer | NOT-FOUND | 93 gate-whitelisted, no `SendData` arm |
| 97-98 | AchievementNPCKilled : AchievementProgressionEvent | `id::NPC_KILLED` / `id::PROGRESSION_EVENT` | S1-I-5: `net.rs` `S2c::NpcKilled` / `S2c::ProgressionEvent`, each one `(short)` (`NetMessage.cs:1347-1352`), read only at `netMode == 1` (`MessageBuffer.cs:3558-3569`). Host→client only — no `C2s`. The kill credit rides `NPC.playerInteraction[256]` (`NPC.cs:6085`) and the progression id is `Main._progressionEvent` (`AchievementsHelper.cs:227-237`) |
| 99-119 | CombatTextString | NOT-FOUND | `SendPlayerHurt`/`SendPlayerDeath` (`NetMessage.cs:2598,2607`) unported |
| 120-128 | Emoji … LandGolfBallInCup | `id::REVENGE_MARKER` (126) | 126 has a body + codec since S1-I-1 wave A (`RevengeMarker.WriteSelfTo`, `CoinLossRevengeSystem.cs:275-286`; the join push `SendAllMarkersToPlayer` `:498-507`); the rest NOT-FOUND |
| 129 | FinishedConnectingToServer | NOT-FOUND | no body either side |
| 130-137 | FishOutNPC … RequestNPCBuffRemoval | `id::DEAD_PLAYER` (135) | 135 has a body + codec since S1-I-1 wave A (`NetMessage.cs:1569-1571`, the join arm `statLife <= 0` `:2897-2900`); the rest NOT-FOUND |
| 138 | ClientSyncedInventory | NOT-FOUND | **the one id `GetData` never dispatches** |
| 139-142 | SetCountsAsHostForGameplay … Trackers | `id::MISC_EVENT_VALUES` (140) / `id::PROJECTILE_TRACKERS` (142) | 140 (kind 0 = credits roll, `CreditsRollEvent.cs:27-33`) and 142 (`TrackedProjectileReference` ×2) have bodies + codecs since S1-I-1 wave A; 139/141 NOT-FOUND — C# has `DoesPlayerSlotCountAsAHost` (`NetMessage.cs:2876`) |
| 143-144 | CrystalInvasionSkip / RequestQuestEffect | NOT-FOUND | |
| 145 | SyncItemsWithShimmerDeprecated | NOT-FOUND | **not read by `GetData`** |
| 146-147 | ShimmerActions / SyncLoadout | `id::SYNC_LOADOUT` (147) | 147 has a body + codec since S1-I-1 wave A (`NetMessage.cs:1624-1628`, the visibility ushort `:1882-1893`); gate-whitelisted; 146 NOT-FOUND |
| 148 | SyncItemCannotBeTakenByEnemiesDeprecated | NOT-FOUND | **not read by `GetData`** |
| 149-153, 155-158 | DeadCellsDisplayJar … SyncChestSize (155) … ExtraSpawnSection | `id::CHEST_SIZE` | `net.rs` — 155 ported by S1-E-27 (`MessageBuffer.cs:4429-4438`): `Chest.Resize` grows/shrinks the slot list, the client materialising the array slot first (`App::materialise_chest_slot`) because the port ships chests lazily where the C# holds all of `Main.chest[8000]`. 149-153, 156-158 stay NOT-FOUND; 151 is `SyncItemDespawn` |
| 154 | Ping | `id::PING` | `net.rs` — `C2s::Ping`/`S2c::Ping`, the server echoes it back (`MessageBuffer.cs:4419-4428`), timed by `net_ping_tick`/`ping_received` (`Net/Ping.cs:24-48`) |
| 159 | RequestSection | NOT-FOUND | the port streams sections unrequested on join (`host_stream_sections`) |
| 160-162 | ItemPosition / HostToken / DamageNPCAck | NOT-FOUND | `161` compares against `Netplay.HostToken` (`MessageBuffer.cs:4471-4476`) |
| 201-205 | *(internal)* | `INT_EDIT/PICKUP/JOINED/LEFT/TIME` | `net.rs:90-94`, deliberately outside 1..162 so the gate rejects them pre-spawn |

### The W4-f21 tail census (2026-11-21) — supersedes the NOT-FOUND rows above for the tail ids

The f21 lane re-censused **MessageID 1-162** against `net.rs` (54 base ids)
plus every pushed/in-flight codec lane (c27 18/36/41/43/51/55/62/84; d89
11/15; d90 17/19/63/64/125/157/158/159; d91 23 re-verify + 24 dead; d92
56/92/130/131/136/137/153/162; d93 27/28/29/95/109/110/128; d94
66/99/100/101/106/108/115; d95 68/91; d96 4/5/12/13/14/16/35/36/40/41/43
fixes; d97 102/117/118/120/134/150/157; d98 21/22/30-34/35/36/80/81; d99
51/61/65/73/75/77/78/96/103/112/113/114/116/127/141/143; d100 none) and
found **44 missing ids**. **132 PlayLegacySound is dead** — zero
`PlayNetSound(` call-sites tree-wide (only the definition,
`NetMessage.cs:2615-2619`) — and is skipped per the task. The f21 branch
ports the other **43**, bodies + codecs verbatim, dispatcher arms parked
for the g48-g52 apply-wiring lanes / S1-I-4:

| id | C# name | Rust | Evidence (writer / reader) |
| --- | --- | --- | --- |
| 24 | UnusedMeleeStrike | `id::UNUSED_MELEE_STRIKE` (census-only) | dead: zero send sites, reader `Assert(false)` reads nothing (`NetMessage.cs:765-768` / `MessageBuffer.cs:1706-1708`); no variant invented |
| 25/26/44/67/83/93 | the reader ignore group | `S2c::Unused25/Unused26/Unknown44/Unknown67/Unused83/SocialHandshake` | bare frames (`MessageBuffer.cs:4483-4490`); 93 alone is accept-gate-whitelisted (`:165`) |
| 39 | ReleaseItemOwnership | `ReleaseItemOwnership` both ways | `:948-951` / `:2232-2255`; short slot + force bool |
| 48 | LiquidUpdate | `LiquidUpdate` both ways | `:987-995` / `:2407-2446`; two shorts + liquid + liquidType bytes |
| 52 | LockAndUnlock | `C2s::LockAndUnlock` | `:1014-1018` / `:2531-2564`; byte action + two shorts, player id NOT written |
| 57 | Unknown57 | `S2c::WorldEvilBytes` | `:1054-1058` / `:2631-2638`; tGood/tEvil/tBlood three bytes |
| 58 | InstrumentSound | both ways | `:1059-1062` / `:2639-2681`; byte player + raw float pitch |
| 76 | QuestsCountSync | both ways | `:1149-1153` / `:3164-3182`; byte + two ints; the relay `:3178` is the only live send |
| 79 | PlaceObject | both ways | `:1166-1174` / `:3192-3215`; the sbyte random + direction bool |
| 85 | QuickStackChests | `C2s::QuickStackRequest` / `S2c::QuickStackBlockedChests` | `:1205-1215` / `:3261-3271`; the netMode-split writer kept as TWO variants (`QuickStacking.cs:299-341`) |
| 86 | TileEntitySharing | `S2c::TileEntitySharing` | `:1216-1226` / `:3272-3293`; the full 11-kind `TileEntityData` table (`TileEntitiesManager.cs:26-39`), networkSend path |
| 87 | TileEntityPlacement | `C2s::TileEntityPlacement` | `:1227-1231` / `:3294-3305` |
| 88 | ItemTweaker | `S2c::ItemTweaker` | `:1232-1296` / `:3306-3376`; the two-BitsByte conditional stats |
| 89/123/133/149 | the TryPlacing quadruplets | `C2s::ItemFrameTryPlacing/WeaponsRackTryPlacing/FoodPlatterTryPlacing/DeadCellsJarTryPlacing` | `:1297-1306`/`:1479-1488`/`:1545-1554`/`:1629-1638`; one shared `TryPlacingItem` body, byte-equality pinned |
| 90 | SpawnInstancedItem | `S2c::SpawnInstancedItem` | shares case 21 (`:646-647` / `:1445-1446`); the client instanced tail `:1489-1493` |
| 94 | DevCommands | both ways | `:1327-1332` / `:3463-3518`; plain string + int + two floats |
| 104 | ShopOverride | `S2c::ShopOverride` | `:1387-1394` / `:3715-3735`; the writer-float/reader-short stack MISMATCH kept verbatim (zero send sites, dead vanilla bug) |
| 105 | GemLockToggle | `C2s::GemLockToggle` | `:1395-1399` / `:3736-3741` |
| 107 | SmartTextMessage | `S2c::SmartTextMessage` | `:769-775` / `:3752-3757`; r/g/b bytes + NetworkText + short width |
| 111 | ToggleParty | `C2s::ToggleParty` | NO writer case; bare frame (`BirthdayParty.cs:154` / `:3809-3814`) |
| 119 | CombatTextString | `S2c::CombatTextString` | `:1188-1196` / `:3235-3242`; the 81 twin's text half; zero live sends |
| 121 | TEDisplayDollDataSync | both ways | `:1458-1474` / `:3920-3941`; the command-keyed WriteData/WriteDummySync (byte-width-identical fallbacks) |
| 122 | RequestTileEntityInteraction | both ways | `:1475-1478` / `:3942-3977`; int id (-1 clear) + byte player |
| 124 | TEHatRackItemSync | both ways | `:1489-1508` / `:3995-4022`; the dye +2 fold/unfold, wire slot kept verbatim |
| 129 | FinishedConnectingToServer | `S2c::FinishedConnecting` | NO writer case; bare frame (`MessageBuffer.cs:937` / `:4075-4085`) |
| 138 | ClientSyncedInventory | `C2s::ClientSyncedInventory` | NO writer AND NO reader case; bare frame, default-arm drop (`Main.cs:17700`) |
| 139 | SetCountsAsHostForGameplay | `S2c::CountsAsHost` | `:1587-1590` / `:4230-4237`; the join push `:934` |
| 144 | RequestQuestEffect | `C2s::RequestQuestEffect` | NO writer case; bare frame (`Main.cs:40287` / `:4304-4309`) |
| 145/148 | the shimmer/enemy-lock deprecated pair | census constants only | neither arm, zero sites; superseded by 146 / the 21 bit-3 tail (d98) |
| 146 | ShimmerActions | `S2c::ShimmerAction` | `:1611-1623` / `:4310-4327`; kind 0 Vector2 / kind 1 Vector2+int / unknown no tail |
| 152 | ItemUseSound | both ways | `:1643-1645` / `:4386-4407`; byte player only |
| 156 | TELeashedEntityAnchorPlaceItem | `C2s::LeashedAnchorPlaceItem` | `:1654-1658` / `:4439-4449`; three shorts |
| 160 | ItemPosition | `S2c::ItemPosition` | `:1666-1672` / `:4463-4470`; short slot + Vector2, no velocity |
| 161 | HostToken | `C2s::HostToken` | `:1673-1675` / `:4471-4476`; plain string, gate-whitelisted (`:165`) |

Witnesses: `net.rs::tests::w4_f21_*` — one byte-pinned round trip per id
(or per shape family), the 104 misalignment pin, the 90-vs-21 and
TryPlacing-quadruplet byte-equality pins, and the 43-id census witness.
WRITTEN NOT RUN (code-only lane). With the sibling lanes merged, the
MessageID 1-162 census closes: every id has a constant, a codec, or a
documented dead/ignore verdict; 132 stays the one deliberate dead skip.

### The f21b apply audit (2026-11-21) — research/239

The apply-arm coverage census across the g48/g49/g50/g51/g52 tips, the
remaining-gap list (39/41/43/48/52/79/58-client/TE families/144/150) and the
union keep-one notes live in `239-f21b-net-apply-audit.md`. The f21b round
WIRED the small unclaimed arms: **57** client store + join push
(`World.t_good/t_evil/t_blood`), **76** host force/store/relay + both own-echo
gates, **58** host id-force/relay (`PlayerNet.angler_quests_finished/
golfer_score`); `w4_f21b_*` witnesses WRITTEN NOT RUN. Nothing from the net
lanes is in `origin/main` yet (checked at exec time, `a725acb04`).

NetModules map 11/12: Liquid, Text, Ping, Ambience, Bestiary, CreativeUnlocks,
CreativePowers, CreativeUnlocksReport, TeleportPylon, Particles,
CreativePowerPermissions — `netmodules.rs::module_id` (34-45). **NetDebugModule has no
Rust counterpart** (module 11).

## SendData call-site census (subsystems calling the writer)

`grep -c "NetMessage.SendData("` over `decompiled/` — **774 sites in 68 files**:

| subsystem | sites | representative ids |
| --- | --- | --- |
| `Terraria/Player.cs` | 197 | 13, 16, 21, 27, 35, 50, 84, 134, 147 |
| `Terraria/NPC.cs` | 118 | 23, 28, 53, 62, 78, 137, 153 |
| `Terraria/WorldGen.cs` | 70 | 7, 10, 17, 20, 79, 109 |
| `Terraria/Main.cs` | 64 | 18, 51, 52, 77, 112 |
| `Terraria/Projectile.cs` | 43 | 27, 29, 59, 108 |
| `Terraria/NetMessage.cs` | 37 | internal re-sends (28→23, 21→151) |
| `Terraria/Wiring.cs` | 24 | 19, 20, 59 |
| `Terraria/UI/ItemSlot.cs` | 22 | 32, 33, 34, 85 |
| `GameContent/Events/DD2Event.cs` | 19 | 78, 112 |
| `Terraria/UI/ChestUI.cs` | 14 | 31, 32, 34, 69 |
| `Terraria/WorldItem.cs` | 12 | 21, 22, 39, 151 |
| `Terraria/MessageBuffer.cs` | 11 | echo arms (48, 23, 13, 27, 20) |
| `Terraria/GameContent/DoorOpeningHelper.cs` | 7 | 19 |
| remaining 55 files | ≤6 each | mostly 79 PlaceObject / 121 TileEntity arms |

(The Rust side has no equivalent fan-out — a single `net_tick` (`netglue.rs:1426`) plus
the edit/pickup request entry points `request_edit` (:1768) and the module senders.)

## Connection flow details

- **Slot identity:** C# assigns `NetMessage.buffer[i].whoAmI = i` for all 257 buffers at
  init (`Netplay.cs:733-744`); the local client is always slot 256. The client is told
  its own id by `3 PlayerInfo`'s first byte (`NetMessage.cs:144`), which the C# server
  overrides to `whoAmI` on the client side. Rust carries the same override for
  `14`/`16` (`netglue.rs` comment, `net.rs:1243-1253`).
- **Accept gate:** C# `MessageBuffer.cs:158-173` — at State -1 only `38 Password`
  passes; at State < 10 only ids ≤ 12 plus the whitelist
  `93, 16, 42, 50, 38, 68, 147, 161` pass, else `BootPlayer`; at State 0 only `1`
  passes. `netglue.rs::host_gate` (151-169) replicates it exactly, including
  `GATE_WHITELIST = [93, 16, 42, 50, 38, 68, 147, 161, 12]` — one extra entry (`12`),
  needed because the port's spawn message is a normal in-flow id.
- **Disconnect:** C# `NetMessage.BootPlayer(plr, msg)` (`:2568`) → `2 Kick` with the
  text + `Kicked = true`; `SyncDisconnectedPlayer` (`:2799`) and the `PendingTermination`
  sweep in `CheckBytes` (`:2488-2496`) run `Player.Hooks.PlayerDisconnect`. Rust has
  `net_disconnect` (`netglue.rs:1351`) and `host_slot_left` (:765); the 120 s
  `IDLE_TIMEOUT` sweep replaces the C# `TimeOutTimer > 7200` tick counter
  (ANALYSIS.md:7294-7297).
- **Ping:** C# `Net/Ping.cs` — `SendData(154)` every 250 ms, `CurrentPing` from a
  `Stopwatch`, with `PingRecieved()` closing the round trip; displayed through
  `Main.ActiveNetDiagnosticsUI`. The port now matches: `id::PING = 154`,
  `C2s::Ping`/`S2c::Ping` (`net.rs`), the server echoes it back
  (`netglue.rs`, C2s dispatch), and `net_ping_tick`/`ping_received`
  (`app/netmodules.rs`) are the `Ping.Update`/`PingRecieved` pair, with
  `Ping.Reset` on connect. NetModule **2** is `NetPingModule` — the *map* ping
  relay (`NetworkInitializer.cs:13`, `Main.cs:56541-56551`), which now
  re-broadcasts markers via `broadcast_except` and adds them to `map.pings`.
- **Chat:** C# carries chat through `NetTextModule` (module id 1) plus the old ids
  25/26 (both deprecated). Rust `netmodules.rs` has `ChatMessage`/`ServerMessage`
  (132-184) and `send_chat_message` (:467), tested by
  `a_chat_message_relays_through_the_host` (`netglue.rs:2426`-ish test list).

## Client-side prediction (vanilla arrangement)

Vanilla keeps the local player fully authoritative on the client and interpolates only
*remote* entities:

- The local client runs the whole `Player` update locally — 107 `Main.netMode == 1`
  guards in `Player.cs` alone gate *only* the parts that must not be predicted
  (damage, death, tile-placement authority, NPC spawn authority).
- Local controls go out at the fixed step as `13 PlayerControls`
  (`Player.cs:6908`: `SendData(13, -1, -1, null, Main.myPlayer)`), and the server
  echoes state back as `13`/`16`/`14` — the client renders its own player from its
  own simulation, never from the echo.
- Remote players and NPCs are *interpolated*, not simulated: the port's implementation
  is `net_interp_tick` (`netglue.rs:1620-1657`) — per-frame decay of an `off_x/off_y`
  error offset for peers and `npc.net_offset` for NPCs, with `SMOOTHING_RANGE = 300`
  and a hard snap above it (`a_teleport_farther_than_the_range_snaps_the_offset`).
  **Correction (S1-I-3, 2026-11-14): this is NOT a port-invented mechanism.** The
  C# has the same offsets: `Player.netOffset` (`Player.cs:1622`) accrues on every
  incoming `13 PlayerControls` (`MessageBuffer.cs:1000-1004`) and decays in
  `Player.UpdateNetOffset` (`Player.cs:28721-28750` — `max(2, 10% of the lag)`
  per tick, the collision-boosted step at `:28737-28747`, the under-2 px clear at
  `:28730-28734`); the NPC twin is `NPC.UpdateNetOffset` (`NPC.cs:90280-90310` —
  the `2 + 2*(len/range)` step, the hard snap above the range and the town NPC's
  drop at its home `:90306-90310`). The earlier "port-invented" reading of this
  section was wrong. The C# counterpart for *where* peers are seeded is
  `net_peer_spawn_pos` (`netglue.rs:1666-1671`, citing `MessageBuffer.cs:908-930`).
- Stale NPC culling: `NPC_STALE = 200 ms` (`netglue.rs:52`) with
  `npc_snapshots_keep_the_slot_and_despawn_on_absence` — C# despawns only on the
  server's explicit `23` with `active = false`, so this is a client-side timeout that
  vanilla does not have.

## Tests

`grep "#\[test\]"`:
- `crates/tr-app/src/net.rs` — 8: `writer_reader_round_trip_primitives`,
  `long_string_uses_seven_bit_length`, `truncated_body_decodes_to_none`,
  `player_info_round_trip`, `world_info_round_trip`,
  `messages_round_trip_through_the_frame`, `server_messages_round_trip`,
  `short_packet_is_a_protocol_error`.
- `crates/tr-app/src/app/netglue.rs` — 26: `handshake_drives_both_sides_to_in_world`,
  `password_gates_the_join`, `wrong_password_is_kicked`, `wrong_version_is_kicked`,
  `a_fresh_slot_may_only_send_connect`, `a_duplicate_connect_is_ignored`,
  `the_world_blob_round_trips`, `the_load_status_reaches_the_client`,
  `player_state_syncs_across_the_wire`, `the_net_offset_decays_towards_zero`
  (`the_net_offset_decays_toward_zero`), `a_teleport_farther_than_the_range_snaps_the_offset`,
  `npc_snapshots_keep_the_slot_and_despawn_on_absence`, `item_states_sync_and_remove`,
  `a_pickup_request_removes_the_drop_on_the_host`,
  `the_host_own_snapshot_reaches_the_client`, `the_join_streams_the_spawn_sections`,
  `a_moved_peer_streams_the_sections_it_approaches`,
  `a_tile_edit_replicates_as_a_section_range`,
  `a_client_edit_request_reaches_the_host`,
  `the_tile_codec_round_trips_runs_and_widths`,
  `a_chat_message_relays_through_the_host`, `the_ping_module_measures_an_rtt`,
  `the_ambience_module_relays_a_spawn`, `a_peer_projectile_syncs_in_and_out`,
  `the_host_projectile_broadcast_reaches_the_client`,
  `a_strike_is_validated_and_relayed_by_the_host`.
- `crates/tr-app/src/app/netmodules.rs` — 0 (`#[test]`-free; covered indirectly).

## Deviations & risks

| # | Mismatch | Severity | Admitted? |
| --- | --- | --- | --- |
| 1 | **Most of the 163 message ids are still unported**, though S1-E-27 (2026-11-16) closed the chest/sign/wiring/buff families — `31/32/33/46/47/50/53/54/59/69/80/155` now have bodies, host arms and client arms — and S1-I-1 wave A (2026-11-18) added the join bundle's `30/42/45/60/72/74/126/135/140/142/147` as bodies + codecs only (dispatcher arms parked; the fan-out is I-4). The 774 C# `SendData` sites still funnel into ~10 Rust send paths, so the team/emote/luck/pylon/invasion messages remain no-ops on the Rust wire. | behavioural | yes — ROADMAP.md:1067-1071 parks the per-system passes |
| 2 | **Internal ids 201-205 sit outside the C# 1..162 range.** `INT_EDIT` replaces `17 TileManipulation`, `INT_PICKUP` replaces the `21`/`151` pickup flow, `INT_JOINED`/`INT_LEFT` replace the `4`/`12`/`14` slot pushes. The port cannot talk to a vanilla client and vice versa. | behavioural (wire-incompatible) | yes — `net.rs:84-89` documents the choice |
| 3 | **`49 InitialSpawn` now carries the C# meaning** (fixed 2026-11-16, S1-E-28) — `MessageBuffer.cs:2447-2453` moves the *client* to State 10 and calls `Spawn`; the port's `id::INITIAL_SPAWN` does exactly that (`State 6 → 10` + `12 PlayerSpawn` with `spawn_context = 1`, `Player.cs:37578-37581`). | behavioural | **fixed** (S1-E-28) |
| 4 | **Ping is id 154 in both** (fixed 2026-11-16, S1-E-28) — `Net/Ping.cs:31` sends a bare `154` and the port's RTT rides the same id (`C2s::Ping`/`S2c::Ping`, echo at `MessageBuffer.cs:4419-4428`); NetModule 2 is back to the `NetPingModule` map-ping relay. | behavioural | **fixed** (S1-E-28) |
| 5 | **`5 PlayerInventory` is carried** (fixed 2026-11-14, S1-I-2) — the server attributes the slot to the sending client, stores it in that client's inventory copy and relays it to the other clients (`MessageBuffer.cs:409-412`/`:454-457`, the `PlayerItemSlotID.CanRelay` slots). The `2 SyncEquipment` arm (the 59+20+10+5+5+4×40+trash push) now reaches the peers. | behavioural | **fixed** (S1-I-2) |
| 6 | **`7 WorldInfo` carries defaults** for 11 bg styles, ice/jungle backs, tree/cave backups, 13 tree-top bytes, 14 flag groups, ore tiers, dungeon point, spawn list, `lobbyId` (ANALYSIS.md:7267-7274). `lobbyId = 0` without a social API. | behavioural | yes |
| 7 | **`NetworkText` is literal-mode only** (mode byte 0 + string, `NetworkText.cs:65-70`); keyed/format text and the substitution list are not ported, so kick/status text is English-literal, not localized (ANALYSIS.md:7256-7260). | behavioural | yes |
| 8 | **`5 WorldChunk`/whole-world blob replaces `10 TileSection` streaming.** ANALYSIS.md:7250-7255 admits M6-NET-3 was to replace the producer; `client_install_world` (`netglue.rs:1231`) consumes slices announced by `9 Status` and closed by `49`. | behavioural | yes |
| 9 | **Client-side smoothing is ported, not invented** (corrected 2026-11-14, S1-I-3). The C#'s `Player.netOffset` (`Player.cs:1622`) accrues on every incoming `13 PlayerControls` (`MessageBuffer.cs:1000-1004`) and decays in `Player.UpdateNetOffset` (`:28721-28750`: `max(2, 10% of the lag)` per tick, the collision-boosted step, the under-2 px clear); the NPC twin is `NPC.UpdateNetOffset` (`NPC.cs:90280-90310`: the `2 + 2*(len/range)` step, the hard snap above the range, the town NPC's drop at its home). S1-I-3 landed the two arms the port had missed — the peers' under-2 px clear and the collision-boosted step — and the NPC's town-home clear. | pixel / behavioural | **fixed** (S1-I-3 — the port matches the C#'s rules; the dt scaling (one 60 Hz tick per `dt * 60`) stays a port convenience) |
| 10 | **NPC stale timeout (200 ms) is port-only** — vanilla despawns only on an explicit server `23` with `active = false`. | behavioural | no |
| 11 | **The port's `12 PlayerSpawn` arm is gate-whitelisted** (`GATE_WHITELIST` contains 12) whereas C# allows `b > 12` for the 9 whitelisted ids only; harmless, but the gate no longer matches `MessageBuffer.cs:165` literally. | cosmetic | no |
| 12 | **Idle sweep is tick-count → wall-clock**: `TimeOutTimer > 7200` ticks (`Main.cs:64610-64618`) became a 120 s `IDLE_TIMEOUT` (`net.rs:41`). 7200 ticks at 60 Hz = 120 s, so the intent matches, but a frame-paced sweep is not tick-exact. | determinism | yes — ANALYSIS.md:7294-7297 "port-tuned" |
| 13 | **`18 SetTime` unported** — the clock rides internal `205 Time` at 10 Hz instead of C#'s join-time `7` + world flags (ANALYSIS.md:7275-7278). | behavioural | yes |
| 14 | **`138/145/148` have no read arm at all** in `GetData` — if any peer ever sends them the packet is silently dropped (the `default` arm only boots a pre-spawn client). | behavioural | n/a (deprecated ids) |
| 15 | **Name gate is ported, creative-mode mismatch checks are not** (`Net.PlayerIsCreativeAndWorldIsNotCreative`) — ANALYSIS.md:7279-7283 defers them, so a creative player can join a non-creative world silently. | behavioural | yes |
| 16 | **`PlayerSpawnContext.SpawningIntoWorld` / the `WorldGen.clearWorld` arm has no port equivalent** (`Netplay.cs:484-505`); the in-process world swap replaces it (ANALYSIS.md:7288-7293). | behavioural | yes |
| 17 | **`SendData` fan-out is 774 → ~10 sites**: the join bundle `SyncOnePlayer` + full inventory + NPC houses + travel shop + angler quest + revenge markers (`NetMessage.cs:2806-2853`) is not reproduced; `SendNPCHousesAndTravelShop`, `SyncChestContentsForSection`, `SyncNPCsForSection` (`:2724`, `:2741`) are absent. | behavioural | yes (parked per-system passes) |
| 18 | **`netSpam` throttle, `netImportant` protection, `DoesPlayerSlotCountAsAHost`** (ROADMAP.md:1063-1065) are unported — the send-throttle behaviour of the C# server has no Rust equivalent. | behavioural | no |
| 19 | **`NetBestiaryModule` carries only the `Kill` unlock kind** (S1-I-5): the `Sight`/`Chat` bodies (`NetBestiaryModule.cs:25-39`) are not sent and `BestiaryKill::read_from` rejects their tag bytes, because the port's `NPCWasNearPlayerTracker`/`NPCWasChatWithTracker` halves are not net-synced (the sets exist and save, they are just never updated in MP). |

## Task seeds

- **S — Map the ported client onto real ids.** Replace `201-205` with the C# ids they
  stand in for (`17`/`21`+`151`/`4`+`12`+`14`) and keep a compatibility note in
  `ANALYSIS.md`. Unblocks any future vanilla interop work. Depends on: nothing.
- **S — Rename `WORLD_COMPLETE` (49).** Confirm the intended semantic against
  `MessageBuffer.cs:2447-2453` and either rename the const or add the real
  initial-spawn arm so the two meanings cannot collide. Depends on: nothing.
- **S — Port ping as id 154** (`Net/Ping.cs:23-35`) alongside module 2, and expose
  `CurrentPing` to the HUD. Depends on: nothing.
- **M — Port the join bundle**: `SyncConnectedPlayer`/`SyncOnePlayer`/
  `SyncOnePlayer_ItemArray` (`NetMessage.cs:2806-2962`) including the 59-slot
  inventory push (`player_item_slot`, `netglue.rs:1798-1820` already has the offsets).
  Depends on: a peer-inventory store (see the id-5 gap).
- **M — Chest/sign/wiring round trip**: `31-33`, `69`, `19`, `20`, `59`, `85` from the
  census (ItemSlot 22 + ChestUI 14 + Wiring 24 sites). Depends on: id mapping (S task 1).
- **M — `18 SetTime` 1:1 clock sync** (drop the internal `205`). Depends on: the world
  flags being carried in `7 WorldInfo`.
- **M — Remote-player smoothing parity**: document or port the C#/ReLogic interpolation
  so `net_interp_tick`'s constants are not arbitrary. Depends on: nothing (research).
- **L — The per-system passes** ROADMAP.md:1067-1071 parks: peer inventories, creative
  powers' per-player split, pylon/bestiary module bodies, the exact clock path.
  Depends on: the join bundle and the chest round trip.
- **L — Dedicated-server mode** (`Main.dedServ`, `ServerMode.cs`, `NetManager`): the
  port has no headless host path; the branch is exercised in-process only.
