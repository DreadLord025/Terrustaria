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
that, S1-I-1 wave A added the join bundle's 11, and wave B (2026-11-20) closed
the 1-16 region census with `11`/`15` (W4-d89) plus the world-sync region's 9 —
`11/17/19/63/64/125/157/158/159` (W4-d90), bodies + codecs with the dispatchers
parked for I-4 — see the map's per-row notes).

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
| 11 | TileFrameSection | `id::TILE_FRAME_SECTION` | S1-I-1 wave B (W4-d89 + W4-d90): `S2c::TileFrameSection`, four `(short)`s (`NetMessage.cs:427-432` → `WorldGen.SectionTileFrame`, `MessageBuffer.cs:895-900`, `WorldGen.cs:80343`); the writer arm EXISTS — the earlier "no SendData arm" note was wrong; no C# call site sends it (deprecated, `MessageID.cs:29-30`); S2c-only, netglue read arm parked for I-4 |
| 12 | PlayerSpawn | `id::PLAYER_SPAWN` | `net.rs:76` |
| 13 | PlayerControls | `id::PLAYER_STATE` | `net.rs:78`; arm `NetMessage.cs:446-513` |
| 14 | PlayerActive | `id::PLAYER_ACTIVE` | `net.rs:66` |
| 15 | Unknown15 | `id::UNKNOWN_15` | S1-I-1 wave B (W4-d89): `S2c::Unknown15`, a bare frame — deprecated (`MessageID.cs:38-39`), the reader's ignore group accepts it (`MessageBuffer.cs:4483-4490`, shared with 25/26/44/67/83/93) |
| 16 | PlayerLifeMana | `id::PLAYER_HP` | `net.rs:73` |
| 17-19 | TileManipulation, SetTime, ToggleDoorState | `id::TILE_EDIT` (17) / `id::DOOR_TOGGLE` (19) | S1-I-1 wave B **corrected the census**: 17 is NOT superseded by 20 — both live. 17 has ~30 direct send sites (`Player.cs:39626`, `DelegateMethods.cs:515`, `Liquid.cs:1204`, `Collision.cs:3353`) and the host relays client edits as 17 (`MessageBuffer.cs:1285`, the `SendTileSquare(-1,x,y,5)` fallback only when a placement survives `:1281`); body `(byte)kind + short x/y + (short)value + (byte)style` (`NetMessage.cs:524-530`). 19: `(byte)action + short x/y + (byte)dir` (`:537-542`, reader `MessageBuffer.cs:1299-1334`). Bodies + codecs only; dispatchers parked for I-4 (`INT_EDIT` 201 unchanged). 18 SetTime still NOT-FOUND — the clock rides internal `205` (ANALYSIS.md:7275-7278) |
| 20 | AreaTileChange | `id::SECTION_RANGE` | `net.rs:72`; arm `:543-644` |
| 21 | SyncItem | `id::ITEM_DROP` | `net.rs:79`; auto-converts to 151 when air (`NetMessage.cs:89-100`) |
| 22 | ItemOwner | `id::ITEM_OWNER` | `net.rs:80`; reservation expiry (`:675-685`) |
| 23 | SyncNPC | `id::NPC_INFO` | `net.rs:105`; arm `NetMessage.cs:686-763` — verbatim body since S1-I-1 d91 (ai[4] flag-gated floats before the netID, `(ushort)target`, lifeMax-keyed life width, statue/difficulty/shimmer flags2, the catchable `releaseOwner` tail, King Slime's SyncAnchor offset); reader `MessageBuffer.cs:1563-1704` |
| 24 | UnusedMeleeStrike | NOT-FOUND (dead) | writer arm exists (`NetMessage.cs:765-768`: `(short)number` + `(byte)number2`) but NO `SendData(24)` call site anywhere; the reader asserts false (`MessageBuffer.cs:1706-1708`, `Invariant.Assert(condition: false, "UnusedMeleeStrike")`) — deliberately unported (S1-I-1 d91 census) |
| 25-26 | Unused25 / Unused26 | NOT-FOUND | deprecated; 25 for NetTextModule (`MessageID.cs:59-60`) |
| 27 | SyncProjectile | `id::PROJECTILE` | `net.rs:70`; arm `NetMessage.cs:776-841` — verbatim since S1-I-1 d91 (key, pos, vel, `(short)type`, flags1 + optional flags2, then ai0/ai1/banner/damage/knockBack/originalDamage/ai2 in flag order); reader `MessageBuffer.cs:1709-1764`; byte pin `s1_i_1_projectile_verbatim_body` |
| 28 | DamageNPC | `id::NPC_STRIKE` | `net.rs:69`; arm `NetMessage.cs:844-850` — `(byte)slot`, generation, `(short)damage`, knockback, `(byte)(hitDir+1)`, `(byte)crit`; reader `MessageBuffer.cs:1766-1817` (the 162 ack `:1770`, the death cascades `:1808-1815`); byte pin `s1_i_1_npc_strike_verbatim_body` |
| 29 | KillProjectile | `id::PROJECTILE_DEAD` | `net.rs:71`; arm `NetMessage.cs:852-855` — `(int)key` + `Vector2(x, y)`; since S1-I-1 d91 BOTH directions carry the position (the client's Kill-tail entry snapshot `Projectile.cs:77256`, the NaN owner-lost despawn `:16697`); the host relays it verbatim and a finite position snaps the slot before the kill (`MessageBuffer.cs:1835-1843`, `projectile.rs::kill_remote_at`); byte pin `s1_i_1_projectile_dead_verbatim_body` |
| 30-36 | TogglePVP / RequestChestOpen (31) / SyncChestItem (32) / SyncPlayerChest (33) / ChestUpdates / PlayerHeal / SyncPlayerZone | `id::TOGGLE_PVP` / `id::CHEST_OPEN_REQUEST` / `id::CHEST_ITEM` / `id::PLAYER_CHEST` | `net.rs` — 30 has a body + codec since S1-I-1 wave A (`NetMessage.cs:856-859`, `MessageBuffer.cs:1851-1867`; dispatcher parked for I-4); 31/32/33 ported by S1-E-27 (`MessageBuffer.cs:1868-1982`, the writer arms `NetMessage.cs:864-911`): the client asks with `31`, the host answers `SendChestContentsTo` (`155` + one `32` per slot), `33` to the asker and `80` to the others, the rigged chest also firing `59`; 32 relays to the other clients (`:1915`). 30 `TogglePVP`, 34 `ChestUpdates`, 35 `PlayerHeal` and 36 `SyncPlayerZone` remain NOT-FOUND (no `SendData` arm of 37's shape) |
| 37 | RequestPassword | `id::PASSWORD_REQUEST` | `net.rs:74` |
| 38 | SendPassword | `id::PASSWORD` | `net.rs:75`; the one message the gate lets through at State -1 |
| 39-48 | ReleaseItemOwnership … OpenSignRequest (46) / OpenSignResponse (47) / LiquidUpdate | `id::PLAYER_MANA` (42) / `id::TEAM_CHANGE` (45) / `id::SIGN_REQUEST` / `id::SIGN_DATA` | `net.rs` — 42 (mana, `NetMessage.cs:961-965`) and 45 (`:970-974`) have bodies + codecs since S1-I-1 wave A (dispatchers parked for I-4); 46/47 ported by S1-E-27 (`MessageBuffer.cs:2366-2405`, writer `NetMessage.cs:979-986`): `Sign.ReadSign`/`TextSign` live in the new `signs.rs`, the record is rewritten on the host and relayed to the other clients when the text changed (`:2398`). 39-45 and 48 remain NOT-FOUND: no `SendData` arm; 42/68 gate-whitelisted; 48 deprecated for NetLiquidModule (`MessageID.cs:108-109`) |
| 49 | InitialSpawn | `id::INITIAL_SPAWN` | `net.rs` — the C# meaning (the client spawns in, `MessageBuffer.cs:2447-2453`) is now the port's too: `S2c::InitialSpawn` drives `State 6 → 10` + `12 PlayerSpawn` with `SpawningIntoWorld` |
| 50-81 | PlayerBuffs (50) … CombatTextInt | `id::PLAYER_BUFFS` / `id::ADD_NPC_BUFF` / `id::NPC_BUFFS` / `id::HIT_SWITCH` / `id::CHEST_NAME` / `id::PLAYER_CHEST_INDEX` + S1-I-1 wave B: `id::TILE_PAINT` (63) / `id::WALL_PAINT` (64) | 63/64 share the writer case — `(short)x + (short)y + (byte)paint + (byte)coat` (`NetMessage.cs:1081-1087`, readers `MessageBuffer.cs:2909-2948` paint vs coat + host relay); bodies + codecs only, parked for I-4. `net.rs` — S1-E-27 ported 50 (`MessageBuffer.cs:2454-2480`: the owner's own list is skipped `:2461-2464`, a peer's lands with every time forced to 60 `:2470-2471`), 53/54 (`:2565-2593`: the host applies `AddBuff(quiet: true)` and pushes the full list back), 59 (`:2682-2693`: `Wiring.HitSwitch` then a relay to every other client), 69 (`:3059-3078`: the rename, `Chest.CreateWorldChest` when the slot is absent) and 80 (`:3216-3227`, the `-3..8000` range gate). S1-I-1 wave A added bodies + codecs for 60 (`id::NPC_HOUSE`, `NetMessage.cs:1067-1072`), 72 (`id::TRAVEL_SHOP`, 40 shorts, `:1134-1141`) and 74 (`id::ANGLER_QUEST`, `:1145-1148`) — dispatchers parked for I-4. The rest stay NOT-FOUND: 77 `SendTemporaryAnimation` (:2590); 79 `SendObjectPlacement` (:2573) |
| 82 | NetModules | `id::NET_MODULE` | `net.rs:68`; the C# arm is `switch (b4)` (`NetMessage.cs:746`) |
| 83 | Unused83 | NOT-FOUND | deprecated (`MessageID.cs:179-180`) |
| 84-96 | PlayerStealth … TeleportPlayer | NOT-FOUND | 93 gate-whitelisted, no `SendData` arm |
| 97-98 | AchievementNPCKilled : AchievementProgressionEvent | `id::NPC_KILLED` / `id::PROGRESSION_EVENT` | S1-I-5: `net.rs` `S2c::NpcKilled` / `S2c::ProgressionEvent`, each one `(short)` (`NetMessage.cs:1347-1352`), read only at `netMode == 1` (`MessageBuffer.cs:3558-3569`). Host→client only — no `C2s`. The kill credit rides `NPC.playerInteraction[256]` (`NPC.cs:6085`) and the progression id is `Main._progressionEvent` (`AchievementsHelper.cs:227-237`) |
| 99-119 | CombatTextString | NOT-FOUND | `SendPlayerHurt`/`SendPlayerDeath` (`NetMessage.cs:2598,2607`) unported |
| 120-128 | Emoji … LandGolfBallInCup | `id::REVENGE_MARKER` (126) / `id::TILE_PICKING` (125) | 125 since S1-I-1 wave B: `(byte)plr + short x/y + (byte)damage` (`NetMessage.cs:1509-1514`, host overrides the id + relays `MessageBuffer.cs:4023-4042`), parked for I-4. 126 has a body + codec since S1-I-1 wave A (`RevengeMarker.WriteSelfTo`, `CoinLossRevengeSystem.cs:275-286`; the join push `SendAllMarkersToPlayer` `:498-507`); the rest NOT-FOUND |
| 129 | FinishedConnectingToServer | NOT-FOUND | no body either side |
| 130-137 | FishOutNPC … RequestNPCBuffRemoval | `id::DEAD_PLAYER` (135) | 135 has a body + codec since S1-I-1 wave A (`NetMessage.cs:1569-1571`, the join arm `statLife <= 0` `:2897-2900`); the rest NOT-FOUND |
| 138 | ClientSyncedInventory | NOT-FOUND | **the one id `GetData` never dispatches** |
| 139-142 | SetCountsAsHostForGameplay … Trackers | `id::MISC_EVENT_VALUES` (140) / `id::PROJECTILE_TRACKERS` (142) | 140 (kind 0 = credits roll, `CreditsRollEvent.cs:27-33`) and 142 (`TrackedProjectileReference` ×2) have bodies + codecs since S1-I-1 wave A; 139/141 NOT-FOUND — C# has `DoesPlayerSlotCountAsAHost` (`NetMessage.cs:2876`) |
| 143-144 | CrystalInvasionSkip / RequestQuestEffect | NOT-FOUND | |
| 145 | SyncItemsWithShimmerDeprecated | NOT-FOUND | **not read by `GetData`** |
| 146-147 | ShimmerActions / SyncLoadout | `id::SYNC_LOADOUT` (147) | 147 has a body + codec since S1-I-1 wave A (`NetMessage.cs:1624-1628`, the visibility ushort `:1882-1893`); gate-whitelisted; 146 NOT-FOUND |
| 148 | SyncItemCannotBeTakenByEnemiesDeprecated | NOT-FOUND | **not read by `GetData`** |
| 149-153, 155-158 | DeadCellsDisplayJar … SyncChestSize (155) … ExtraSpawnSection | `id::CHEST_SIZE` (155) / `id::TEAM_CHANGE_UI` (157) / `id::EXTRA_SPAWN_LOADED` (158) | 157/158 since S1-I-1 wave B: 157 shares 45's writer case (`NetMessage.cs:970-971`) — same `TeamChange` body, the reader adds the team-spawn tail (`MessageBuffer.cs:2355-2363` → `CheckSection` + `SendData(158)`), C2s-only (the host relays as 45, sent from `Main.cs:42766`); 158 is `(byte)plr` (`NetMessage.cs:1659-1661`) → `Spawn(TeamSwap)` (`MessageBuffer.cs:4450-4456`), S2c-only. Parked for I-4. `net.rs` — 155 ported by S1-E-27 (`MessageBuffer.cs:4429-4438`): `Chest.Resize` grows/shrinks the slot list, the client materialising the array slot first (`App::materialise_chest_slot`) because the port ships chests lazily where the C# holds all of `Main.chest[8000]`. 149-153, 156-158 stay NOT-FOUND; 151 is `SyncItemDespawn` |
| 154 | Ping | `id::PING` | `net.rs` — `C2s::Ping`/`S2c::Ping`, the server echoes it back (`MessageBuffer.cs:4419-4428`), timed by `net_ping_tick`/`ping_received` (`Net/Ping.cs:24-48`) |
| 159 | RequestSection | `id::SECTION_REQUEST` | S1-I-1 wave B: body + C2s codec — writer two `(short)`s (`NetMessage.cs:1662-1665`), reader two `ReadUInt16` (`MessageBuffer.cs:4460` → `SendSection`), same bytes; sent from `Player.cs:41534` / `WorldItem.cs:375`. The port still streams sections unrequested on join (`host_stream_sections`); the on-request host arm is parked for I-4 |
| 160-162 | ItemPosition / HostToken / DamageNPCAck | NOT-FOUND | `161` compares against `Netplay.HostToken` (`MessageBuffer.cs:4471-4476`) |
| 201-205 | *(internal)* | `INT_EDIT/PICKUP/JOINED/LEFT/TIME` | `net.rs:90-94`, deliberately outside 1..162 so the gate rejects them pre-spawn |

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
- `crates/tr-app/src/net.rs` — 8 core + the per-id families (`s1_i_1_*` 11 wave A,
  `s1_i_1b_*` 8 wave B — written not run on the code-only wave-B branch):
  `writer_reader_round_trip_primitives`,
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
| 1 | **Most of the 163 message ids are still unported**, though S1-E-27 (2026-11-16) closed the chest/sign/wiring/buff families — `31/32/33/46/47/50/53/54/59/69/80/155` now have bodies, host arms and client arms — S1-I-1 wave A (2026-11-18) added the join bundle's `30/42/45/60/72/74/126/135/140/142/147` as bodies + codecs only, and wave B (2026-11-20) the world-sync region's `11/17/19/63/64/125/157/158/159` the same way (dispatcher arms parked; the fan-out is I-4). The 774 C# `SendData` sites still funnel into ~10 Rust send paths, so the team/emote/luck/pylon/invasion messages remain no-ops on the Rust wire. | behavioural | yes — ROADMAP.md:1067-1071 parks the per-system passes |
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
- ~~**S — Rename `WORLD_COMPLETE` (49).**~~ Done (S1-E-28 renamed + re-armed;
  wave B (W4-d89) purged the last stale `WorldComplete` doc in
  `client_state::IN_WORLD`). The semantic is confirmed against
  `MessageBuffer.cs:2447-2453`.
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
