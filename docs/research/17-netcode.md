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

| 21 | SyncItem | `id::ITEM_DROP` | `net.rs`; auto-converts to 151 when air (`NetMessage.cs:89-100`); S1-I-1 wave I added bit 3 `enemyGrabDelayTime` + the clamped tail byte (`:652`/`:665-668`, reader `MessageBuffer.cs:1459`/`:1487`) and the `ItemDrop.enemy_grab_delay_time` decay (`WorldItem.cs:449-452`); the ownership bits 0/1 stay 0 on the diff cadence (spawn-send-only in C#, `Item.cs:49768`) |
| 22 | ItemOwner | `id::ITEM_OWNER` | `net.rs`; reservation expiry (`:675-685`) — wave I re-verified field-for-field |
| 23 | SyncNPC | `id::NPC_INFO` | `net.rs:82`; arm `:686-748` |
| 24 | UnusedMeleeStrike | NOT-FOUND | dead (`MessageID.cs:57`) |
| 25-26 | Unused25 / Unused26 | NOT-FOUND | deprecated; 25 for NetTextModule (`MessageID.cs:59-60`) |
| 27 | SyncProjectile | `id::PROJECTILE` | `net.rs:70`; arm `NetMessage.cs:776-841` — verbatim since S1-I-1 d91 (key, pos, vel, `(short)type`, flags1 + optional flags2, then ai0/ai1/banner/damage/knockBack/originalDamage/ai2 in flag order); reader `MessageBuffer.cs:1709-1764`; byte pin `s1_i_1_projectile_verbatim_body` |
| 28 | DamageNPC | `id::NPC_STRIKE` | `net.rs:69`; arm `NetMessage.cs:844-850` — `(byte)slot`, generation, `(short)damage`, knockback, `(byte)(hitDir+1)`, `(byte)crit`; reader `MessageBuffer.cs:1766-1817` (the 162 ack `:1770`, the death cascades `:1808-1815`); byte pin `s1_i_1_npc_strike_verbatim_body` |
| 29 | KillProjectile | `id::PROJECTILE_DEAD` | `net.rs:71`; arm `NetMessage.cs:852-855` — `(int)key` + `Vector2(x, y)`; since S1-I-1 d91 BOTH directions carry the position (the client's Kill-tail entry snapshot `Projectile.cs:77256`, the NaN owner-lost despawn `:16697`); the host relays it verbatim and a finite position snaps the slot before the kill (`MessageBuffer.cs:1835-1843`, `projectile.rs::kill_remote_at`); byte pin `s1_i_1_projectile_dead_verbatim_body` |
| 30-36 | TogglePVP / RequestChestOpen (31) / SyncChestItem (32) / SyncPlayerChest (33) / ChestUpdates / PlayerHeal / SyncPlayerZone | `id::TOGGLE_PVP` / `id::CHEST_OPEN_REQUEST` / `id::CHEST_ITEM` / `id::PLAYER_CHEST` | `net.rs` — 30 has a body + codec since S1-I-1 wave A (`NetMessage.cs:856-859`, `MessageBuffer.cs:1851-1867`; dispatcher parked for I-4); 31/32/33 ported by S1-E-27 (`MessageBuffer.cs:1868-1982`, the writer arms `NetMessage.cs:864-911`): the client asks with `31`, the host answers `SendChestContentsTo` (`155` + one `32` per slot), `33` to the asker and `80` to the others, the rigged chest also firing `59`; 32 relays to the other clients (`:1915`). 30 `TogglePVP`, 34 `ChestUpdates`, 35 `PlayerHeal` and 36 `SyncPlayerZone` remain NOT-FOUND (no `SendData` arm of 37's shape) |

| 27 | SyncProjectile | `id::PROJECTILE` | `net.rs:70`; arm `:780-841` |
| 28 | DamageNPC | `id::NPC_STRIKE` | `net.rs:69`; cascades a 23 when `spawnNeedsSyncing` (`:102-107`) |
| 29 | KillProjectile | `id::PROJECTILE_DEAD` | `net.rs:71` |
| 30-36 | TogglePVP / RequestChestOpen (31) / SyncChestItem (32) / SyncPlayerChest (33) / ChestUpdates / PlayerHeal / SyncPlayerZone | `id::TOGGLE_PVP` / `id::CHEST_OPEN_REQUEST` / `id::CHEST_ITEM` / `id::PLAYER_CHEST` | `net.rs` — 30 has a body + codec since S1-I-1 wave A (`NetMessage.cs:856-859`, `MessageBuffer.cs:1851-1867`; dispatcher parked for I-4); 31/32/33 ported by S1-E-27 (`MessageBuffer.cs:1868-1982`, the writer arms `NetMessage.cs:864-911`): the client asks with `31`, the host answers `SendChestContentsTo` (`155` + one `32` per slot), `33` to the asker and `80` to the others, the rigged chest also firing `59`; 32 relays to the other clients (`:1915`). 34 `ChestUpdates` remains NOT-FOUND; 35 `PlayerHeal` + 36 `SyncPlayerZone` gained bodies + codecs in W4-d96 (batches 1/4, `NetMessage.cs:929-932`/`:933-944` — the census table above), dispatchers parked for I-4 |

| 30-36 | TogglePVP / RequestChestOpen (31) / SyncChestItem (32) / SyncPlayerChest (33) / ChestUpdates / PlayerHeal / SyncPlayerZone | `id::TOGGLE_PVP` / `id::CHEST_OPEN_REQUEST` / `id::CHEST_ITEM` / `id::PLAYER_CHEST` / `id::CHEST_UPDATES` / `id::PLAYER_HEAL` / `id::PLAYER_ZONE` | `net.rs` — 30 has a body + codec since S1-I-1 wave A (`NetMessage.cs:856-859`, `MessageBuffer.cs:1851-1867`; dispatcher parked for I-4); 31/32/33 ported by S1-E-27 (`MessageBuffer.cs:1868-1982`, the writer arms `NetMessage.cs:864-911`): the client asks with `31`, the host answers `SendChestContentsTo` (`155` + one `32` per slot), `33` to the asker and `80` to the others, the rigged chest also firing `59`; 32 relays to the other clients (`:1915`). S1-I-1 wave I (2026-11-20) ported 34 `ChestUpdates` (`id::CHEST_UPDATES`, `NetMessage.cs:913-927` — the fifth short is `number5` server-side / literal 0 client-side; the host place/destroy pass `MessageBuffer.cs:1995-2157` parked for I-4) and fixed 31's `UsingChest` to count the host's own open chest (`Chest.cs:502-508`). The wave-I verify round corrected the old NOT-FOUND claim: 35 `PlayerHeal` and 36 `SyncPlayerZone` DO exist on both sides (`NetMessage.cs:929-932`/`:933-944`, `MessageBuffer.cs:2160-2177`/`:2178-2202`) and now have bodies + codecs (`id::PLAYER_HEAL`/`id::PLAYER_ZONE` — 35 = byte plr + short amount, the host overrides the id to the sender, calls `HealEffect` unless local-without-SSC and relays; 36 = byte plr + the five zone bytes + `(byte)townNPCs`, the host relays and spawns faelings on zone5[0]'s rising edge); dispatcher arms parked for I-4 |
| 37 | RequestPassword | `id::PASSWORD_REQUEST` | `net.rs:74` |
| 38 | SendPassword | `id::PASSWORD` | `net.rs:75`; the one message the gate lets through at State -1 |
| 39-48 | ReleaseItemOwnership … OpenSignRequest (46) / OpenSignResponse (47) / LiquidUpdate | `id::PLAYER_MANA` (42) / `id::TEAM_CHANGE` (45) / `id::SIGN_REQUEST` / `id::SIGN_DATA` | `net.rs` — 42 (mana, `NetMessage.cs:961-965`) and 45 (`:970-974`) have bodies + codecs since S1-I-1 wave A (dispatchers parked for I-4); 46/47 ported by S1-E-27 (`MessageBuffer.cs:2366-2405`, writer `NetMessage.cs:979-986`): `Sign.ReadSign`/`TextSign` live in the new `signs.rs`, the record is rewritten on the host and relayed to the other clients when the text changed (`:2398`). 39/44/48 remain NOT-FOUND (44 deprecated dead, the reader's no-op group `:4483-4490`; 48 deprecated for NetLiquidModule `MessageID.cs:108-109`); 41 `ItemRotationAndAnimation` + 43 `ManaEffect` gained bodies + codecs in W4-d96 (batches 4/5, `NetMessage.cs:956-960`/`:966-969` — the census table above); 42/68 gate-whitelisted |
| 49 | InitialSpawn | `id::INITIAL_SPAWN` | `net.rs` — the C# meaning (the client spawns in, `MessageBuffer.cs:2447-2453`) is now the port's too: `S2c::InitialSpawn` drives `State 6 → 10` + `12 PlayerSpawn` with `SpawningIntoWorld` |
| 50-81 | PlayerBuffs (50) … CombatTextInt | `id::PLAYER_BUFFS` / `id::ADD_NPC_BUFF` / `id::NPC_BUFFS` / `id::HIT_SWITCH` / `id::CHEST_NAME` / `id::PLAYER_CHEST_INDEX` + S1-I-1 wave B: `id::TILE_PAINT` (63) / `id::WALL_PAINT` (64) | 63/64 share the writer case — `(short)x + (short)y + (byte)paint + (byte)coat` (`NetMessage.cs:1081-1087`, readers `MessageBuffer.cs:2909-2948` paint vs coat + host relay); bodies + codecs only, parked for I-4. `net.rs` — S1-E-27 ported 50 (`MessageBuffer.cs:2454-2480`: the owner's own list is skipped `:2461-2464`, a peer's lands with every time forced to 60 `:2470-2471`), 53/54 (`:2565-2593`: the host applies `AddBuff(quiet: true)` and pushes the full list back), 59 (`:2682-2693`: `Wiring.HitSwitch` then a relay to every other client), 69 (`:3059-3078`: the rename, `Chest.CreateWorldChest` when the slot is absent) and 80 (`:3216-3227`, the `-3..8000` range gate). S1-I-1 wave A added bodies + codecs for 60 (`id::NPC_HOUSE`, `NetMessage.cs:1067-1072`), 72 (`id::TRAVEL_SHOP`, 40 shorts, `:1134-1141`) and 74 (`id::ANGLER_QUEST`, `:1145-1148`) — dispatchers parked for I-4. The rest stay NOT-FOUND: 77 `SendTemporaryAnimation` (:2590); 79 `SendObjectPlacement` (:2573) |

| 50-81 | PlayerBuffs (50) … CombatTextInt | `id::PLAYER_BUFFS` / `id::ADD_NPC_BUFF` / `id::NPC_BUFFS` / `id::HIT_SWITCH` / `id::CHEST_NAME` / `id::PLAYER_CHEST_INDEX` | `net.rs` — S1-E-27 ported 50 (`MessageBuffer.cs:2454-2480`: the owner's own list is skipped `:2461-2464`, a peer's lands with every time forced to 60 `:2470-2471`), 53/54 (`:2565-2593`: the host applies `AddBuff(quiet: true)` and pushes the full list back), 59 (`:2682-2693`: `Wiring.HitSwitch` then a relay to every other client), 69 (`:3059-3078`: the rename, `Chest.CreateWorldChest` when the slot is absent) and 80 (`:3216-3227`, the `-3..8000` range gate). S1-I-1 wave A added bodies + codecs for 60 (`id::NPC_HOUSE`, `NetMessage.cs:1067-1072`), 72 (`id::TRAVEL_SHOP`, 40 shorts, `:1134-1141`) and 74 (`id::ANGLER_QUEST`, `:1145-1148`) — dispatchers parked for I-4. S1-I-1 wave I added 81 `CombatTextInt` (`id::COMBAT_TEXT_INT`, `NetMessage.cs:1179-1186` — float pair + RGB + int amount; the client apply spawns the heal number, `MessageBuffer.cs:3227-3233`; the one send site `NPC.cs:77442`). The rest stay NOT-FOUND: 77 `SendTemporaryAnimation` (:2590); 79 `SendObjectPlacement` (:2573) |
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

### The f21b apply audit (2026-11-21) — research/276

The apply-arm coverage census across the g48/g49/g50/g51/g52 tips, the
remaining-gap list (39/41/43/48/52/79/58-client/TE families/144/150) and the
union keep-one notes live in `276-f21b-net-apply-audit.md` (renumbered
239 → 276 per research/150 round-2 §6.3 — 239 belongs to d42's earlier push).
The f21b round WIRED the small unclaimed arms: **57** client store + join push
(`World.t_good/t_evil/t_blood`), **76** host force/store/relay + both own-echo
gates, **58** host id-force/relay (`PlayerNet.angler_quests_finished/
golfer_score`); `w4_f21b_*` witnesses WRITTEN NOT RUN. Nothing from the net
lanes is in `origin/main` yet (checked at exec time, `a725acb04`).

NetModules map 11/12: Liquid, Text, Ping, Ambience, Bestiary, CreativeUnlocks,
CreativePowers, CreativeUnlocksReport, TeleportPylon, Particles,
CreativePowerPermissions — `netmodules.rs::module_id` (34-45). **NetDebugModule has no
Rust counterpart** (module 11).

## W4 player-family census (the d96/d97/c27 lane split)

The player-family ids split into three lanes (2026-11-20, S1-I-1 wave W4):
**d96** (`task/W4-d96-net-ids-player-1`) owns the FIRST half — the low
region around `4/13/16/42/45` per `MessageID.cs`; **d97**
(`task/W4-d97-net-ids-player-2`) owns the second half (the high-region
player ids); **c27** (`task/W4-c27-net-entity-state-ids`) owns the live
entity-state ids (`13/21/23/27`'s dispatcher/cadence wiring) — d96 does NOT
touch that set's apply/relay arms (they stay parked for S1-I-4).

d96's region, id by id (writer = `NetMessage.cs`, reader =
`MessageBuffer.cs`; all bodies + codecs verbatim, dispatchers parked):

| id | C# name | writer | reader | d96 status |
| --- | --- | --- | --- | --- |
| 4 | SyncPlayer | :147-197 | :284-397 | read clamps verbatim (batch 3: skinVariant 0..11, voiceVariant 1..4, NaN pitch → 0 then ±1, hair ≥ 228 → 0, name `.Trim().Trim()`); difficulty bits corrected — 3 → bit **3**, bit 2 = extraAccessory (:166-179, batch 6) |
| 5 | SyncEquipment | :199-222 | :399-462 | leading `(byte)number` added (batch 2); negative-stack clamp citation repointed to the writer arm (:210-213) |
| 12 | PlayerSpawn | :433-445 | :901-951 | leading id byte verbatim + S2c relay variant (batch 1) |
| 13 | PlayerControls | :446-513 | :952-1059 | conditional tails verbatim (batch 6): velocity b2, `(ushort)mount.Type` b7, potion use+home Vector2 pair c6, camera Vector2 d5 (:492-508), ahead of the port's render extension; values stored, not simulated |
| 14 | PlayerActive | :515-518 | :1061-1093 | decode tests the byte `== 1` (:1070), not `!= 0` (batch 2) |
| 16 | PlayerLifeMana | :519-523 | :1095-1117 | writer raw; read floors `statLifeMax` at 20 (:1107-1110), `dead = statLife <= 0` left to the apply arm (batch 5) |
| 30 | TogglePVP | :856-859 | :1851-1867 | wave A body re-verified verbatim (`(byte)id + hostile` bool); the chat fan-out (:1863-1864) is S1-I-4 |
| 35 | PlayerHeal | :929-932 | :2160-2177 | body + codecs (batch 1); send site `Player.HealEffect` (`Player.cs:4522-4529`) |
| 36 | SyncPlayerZone | :933-944 | :2178-2201 | body + codecs (batch 4): id + zone1..5 BitsBytes + `(byte)townNPCs`; the zone5[0] faeling edge (:2195-2198) rides S1-I-4; send sites `Main.cs:17634` (change test) + `:64533` (900-tick heartbeat) |
| 40 | SyncTalkNPC | :952-955 | :2256-2270 | body + codecs (batch 1); `Player.SetTalkNPC` (`Player.cs:4375`) |
| 41 | ItemRotationAndAnimation | :956-960 | :2271-2289 | body + codecs (batch 4): id + float rotation + `(short)itemAnimation`; the `channel` re-derive (:2283) rides S1-I-4; sent beside a 13 at `Player.cs:43693`/`:47374`/`:47380` |
| 42 | Unknown42 (mana) | :961-965 | :2290-2306 | wave A body re-verified verbatim (`(byte)id + (short)statMana + (short)statManaMax`); the own-echo skip (:2297-2300) is the apply arm's |
| 43 | ManaEffect | :966-969 | :2307-2324 | body + codecs (batch 5) — 35's mana twin; the reader has NO ServerSideCharacter escape (:2315, unlike 35's :2168); send site `Player.ManaEffect` (`Player.cs:4531-4538`) |
| 44 | Unknown44 | — | :4483-4490 | DEAD — deprecated (`MessageID.cs:99-100`), no writer arm, the reader's shared no-op group |
| 45 | TeamChange | :970-974 | :2325-2364 | wave A body re-verified verbatim (`(byte)id + (byte)team`); shares the arm with 157 TeamChangeFromUI (d90's world-sync region); the team chat fan-out + the 157 team-spawn tail are S1-I-4 |

Not d96's: `33 SyncPlayerChest` (chest family, S1-E-27), the second-half
player ids `50 PlayerBuffs` (already S1-E-27) / `55 AddPlayerBuffPvP` /
`84 PlayerStealth` / `117 PlayerHurtV2` / `118 PlayerDeathV2` /
`134 UpdatePlayerLuckFactors` / `135 DeadPlayer` (wave A body) /
`150 SpectatePlayer` (d97's census), and the c27 entity-state set.

Witnesses: `net.rs::tests::w4_d96_*` — one byte-pinned round trip per id
(the frame's id byte + the exact C# body bytes + the decode), plus
`w4_d96_player_info_read_clamps`, `w4_d96_player_hp_read_floors_the_max`,
`w4_d96_player_state_conditional_tails_verbatim`,
`w4_d96_player_info_difficulty_bit_three`. Code-only mode: WRITTEN NOT RUN
(no compiler invoked); verification = whole-arm C# re-reads.

## The g48 net-apply-status table (ids 1-40, 2026-11-21) — the f21 cross-note

The task's cross-note asked for `research/107 net-apply-status` on
`origin/task/W4-f21-net-ids-verify-tail`; **no research/107 file exists on
any pushed branch** (checked at exec time — f21's census lives in THIS
file's "W4-f21 tail census" section, `5274a097`, ids 43-162). This section
is the update-compatible ids 1-40 apply-status table for the g48 lane
(`task/W4-g48-net-apply-arms-1`): every id's host/client apply state on the
g48 tip, with the C# reader cite. Status vocabulary: **WIRED** = the apply
arm runs the C# body (deviations noted); **PARKED** = body/codec ported,
dispatcher arm still a no-op (the owning lane named); **DEAD** = no live
reader/writer (the f21 census verdicts); **N/A** = the id is not a
`GetData` apply case in the port's flow (the join FSM owns it).

| id | C# name | reader (`MessageBuffer.cs`) | g48 apply status |
| --- | --- | --- | --- |
| 1 | Connect | :184-214 | N/A — the join FSM (`netglue` C2s::Connect, version gate + slot assign) |
| 2 | Disconnect | :215-222 | WIRED — S2c::Disconnect → `net_disconnect` (pre-g48) |
| 3 | PlayerId | :225-283 | N/A — the join FSM (slot id + the State-1 tail) |
| 4 | SyncPlayer | :284-397 | WIRED — `PlayerInfo` store (pre-g48; d96 read clamps in the codec) |
| 5 | SyncEquipment | :399-462 | WIRED — the inventory slot store + relay (pre-g48) |
| 6 | RequestWorld | :462-472 | WIRED — the WorldInfo + section stream answer (pre-g48) |
| 7 | WorldInfo | :473-658 | WIRED — `client_install_world` (pre-g48) |
| 8 | SpawnRequest | :671-806 | WIRED — the section burst + InitialSpawn (pre-g48) |
| 9 | Status | :877-888 | WIRED — the load-status line (pre-g48) |
| 10 | TileBlock | :889-894 | WIRED — `apply_tile_block` (pre-g48) |
| 11 | TileFrameSection | :895-899 | WIRED g48 — `framing::section_tile_frame` verbatim + map burst (`3e551bb3`) |
| 12 | PlayerSpawn | :901-950 | WIRED g48 — host store/relay + client Spawn_SetPosition (`3e551bb3`) |
| 13 | PlayerControls | :952-1059 | WIRED — the state store + relay (pre-g48); the mount/potion/camera SIMULATION tails parked (c27/d96 note) |
| 14 | PlayerActive | :1061-1088 | WIRED — the join/leave fan-out (pre-g48) |
| 15 | Unknown15 | :4483-4490 | DEAD — the reader ignore group (d89) |
| 16 | PlayerLifeMana | :1095-1117 | WIRED g48 — hp store + dead derivation + relay (`e50c2919`) |
| 17 | TileManipulation | :1119-1288 | WIRED g48 — the full kind dispatch (`e50c2919`); kind 15 FrameTrack (`3a310ccc`) + kind 21 ReplaceTile (`cafd7f2f`) closed the last two cited kinds; kinds 18/19 SetCurrentUser attribution stays the M7-WIRE-2 gap |
| 18 | SetTime | :1289-1297 | PARKED on this tip — FINAL re-probe: the reader/codec LANDED on the clock lane's c27 tip `412b6bc0d` (batch 1 `6fb7bcb25`: `id::SET_TIME = 18` + `SetTime` body `NetMessage.cs:531-536` + client apply arm + `w4_c27_set_time_applies_the_hosts_clock`; sunModY/moonModY parked, no port field). The g48 tip still rides internal 205 (the 10 Hz host broadcast); the send switch is c28's cadence lane — c28 tip `6379f881b` is census-only (tool + tables, no switch; its `net_player_sync.rs:24-25` explicitly assigns the 18 body/apply to c27). CORRECTION re-probe: the C# has ZERO `SendData(18)` sites (the census `per_id_counts` has no 18 key) — the live clock sync rides `7 WorldData` (event-driven + the 3600-tick `UpdateServer` heartbeat); the exact c28 switch instruction is published in research/310 (chain 244→281→310, registry UNIFIED FINAL §9.2) |
| 19 | ToggleDoorState | :1299-1334 | WIRED g48 — doors 0/1 (`0e6fd9c2`) + trapdoors 2/3 / tall gates 4/5 via the c42 canonical `edit/shift.rs` (`a714f106`; the wiring-lane SendData(19) echoes stay research/95 §8.2) |
| 20 | SectionRange | :1335-1443 | WIRED — `apply_section_range` (pre-g48); the g52 latch filter is that lane's |
| 21 | ItemDrop | :1445-1535 | WIRED — the item store (pre-g48, d98 grab-delay tail) |
| 22 | ItemOwner | :1552-1562 | WIRED — the reservation store (pre-g48) |
| 23 | NPCInfo | :1563-1704 | WIRED — the NPC store/spawn (pre-g48, d91 body) |
| 24 | UnusedMeleeStrike | :1706-1708 | DEAD — zero send sites, reader asserts (f21 census) |
| 25 | Unused25 | :4483-4490 | DEAD — the ignore group (f21) |
| 26 | Unused26 | :4483-4490 | DEAD — the ignore group (f21) |
| 27 | Projectile | :1711-1763 | WIRED — the projectile store (pre-g48, d93 body) |
| 28 | NPCStrike | :1766-1817 | WIRED — the strike apply + relay (pre-g48, g52 cascade) |
| 29 | ProjectileDead | :1825-1850 | WIRED — the kill apply (pre-g48) |
| 30 | TogglePVP | :1851-1867 | WIRED g48 — store/relay + team-coloured chat (`0e6fd9c2`) |
| 31 | RequestChestOpen | :1868-1895 | WIRED — the chest contents answer + rigged 59 (S1-E-27) |
| 32 | SyncChestItem | :1897-1918 | WIRED — the slot store + relay (S1-E-27) |
| 33 | SyncPlayerChest | :1920-1982 | WIRED — the open-index store (S1-E-27) |
| 34 | ChestUpdates | :1984-2158 | WIRED g48 — host place/destroy + relays, client direct pass (`f086297b`) |
| 35 | PlayerHeal | :2160-2177 | WIRED g48 — peer HealEffect text + relay (`f086297b`) |
| 36 | SyncPlayerZone | :2178-2201 | WIRED g48 — six-byte store + relay (`9e037b7c`) + the faeling edge → `spawner::spawn_faelings` (`9c7646e0`; the Shimmerfly 677 spawn roster-gated, the scan draws verbatim) |
| 37 | PasswordRequest | :2203-2216 | N/A — the join FSM (pre-g48) |
| 38 | Password | :2218-2231 | N/A — the join FSM (pre-g48) |
| 39 | ReleaseItemOwnership | :2232-2255 | PARKED — FINALIZED (close-or-finalize verdict): f21 codec `RELEASE_ITEM_OWNERSHIP = 39` (`e1d90bcd`, tip `5274a097` — slot i16 + forceAssignToServer bool, `NetMessage.cs:948-951`), the dispatcher arm stays the item lane's. Scope pin (`MessageBuffer.cs:2232-2255`): host leg = reservation-is-mine gate → `FindOwner(forceAssignToServer: true)`; client leg = clear reservation (255, timer 0) → `FindOwner(force)` → if still 255 re-announce via `22`. `WorldItem.FindOwner` (`WorldItem.cs:188-271`) needs the `CanPullItem`/`ItemSpace`/magnet/hopper peer scan + `timeToKeepReservation` + `instanced` — none rowed on this tip (`ItemDrop` carries no reservation fields; `net_reserved` is the client-side 22 store). No apply-arm lane g49/g50/g51 took it (grep empty at all three tips); M6-NET-5's ATLAS row is the 27/28/29 sync, so the 39 arm is the ITEM lane's remaining scope — parked with citation, not a g48 gap. CORRECTION re-probe: the full task-seed (the FindOwner/ReserveFor `:188-285` window, the unrowed-fields census, the lane adjudication incl. the ROADMAP S1-I-4a "stay M6-NET-5" forward-reference correction) is published in research/318 (chain 245→282→318, registry UNIFIED FINAL §9.2) |
| 40 | SyncTalkNPC | :2256-2270 | WIRED g48 — SetTalkNPC store + bestiary chat registration + relay (`9e037b7c`) |

The g48 witnesses: `netglue::g48_apply_tests::g48_*` — one loopback pair
per wired family (`in_world_pair`), WRITTEN NOT RUN (code-only mode). The
remaining PARKED rows' owners: 18 (the clock lane — c27 body/apply landed,
c28 cadence switch pending), 39 (the item lane — finalized scope above);
the 1-40 region has no other missing apply arm on this tip.

### FINAL census verdict (NET-APPLY-1, 2026-11-21, re-probe at the lane tips)

The ids 1-40 table above is FINAL for the g48 lane. Re-probe evidence:

- **18 SetTime / internal 205:** c27 tip `412b6bc0d` carries the landed
  reader (`net.rs` `id::SET_TIME`/`SetTime` + `netglue.rs` `S2c::SetTime`
  client apply, sunModY/moonModY parked); c28 tip `6379f881b` is the
  SendData-trigger census ONLY (918 sites, tool + md/json — the 10 Hz
  internal-205 broadcast is NOT yet switched there). g48 keeps its
  205 broadcast untouched (the c28 lane owns the send switch); row 13 of
  the deviations table stays admitted until that switch lands.
  CORRECTION re-probe: the C# has ZERO `SendData(18)` sites — the live
  clock sync rides `7 WorldData` (event edges + the 3600-tick
  `UpdateServer` heartbeat); the exact switch instruction (205 → the
  id-7 cadence, the c27 18 reader kept wire-compat) is published in
  research/310.
- **39 ReleaseItemOwnership:** finalized PARKED with the full scope pin
  (the table row) — no lane g49/g50/g51 carries an apply arm; the item
  lane owns FindOwner + the 22 re-announce. Not closable on g48.
  CORRECTION re-probe: the item-lane task-seed (the lane adjudication +
  the `WorldItem.cs:188-285` FindOwner/ReserveFor window + the unrowed
  reservation-fields census) is published in research/318.
- Every other 1-40 id: WIRED (30 — ids 2/4-14/16/17/19-23/27-36/40),
  N/A join-FSM (4: 1/3/37/38), DEAD (4: 15/24/25/26), PARKED-with-owner
  (2: 18/39). Counted 40/40 mechanically off the table's status column.
- The witness census: 16 `g48_*` loopback tests in
  `netglue.rs::g48_apply_tests` + 1 `w4_g48_the_pound_fail_tail_never_removes_the_track`
  core pin (research/272) = 17 lane witnesses, ALL WRITTEN NOT RUN.
> **W4-g51 SEAL cross-note (2026-11-21, research/359):** the NOT-FOUND rows
> above are the BASE-trunk census — the g48-g51 apply lanes landed the tail
> on their branches. For the g51 range: 129/138/139/145/146/148/152/160/161
> (the f21 cross-check lifts) + the 8 BIG TE/dryad arms
> 121/122/123/124/133/144/149/156 (the FINAL phase) are WIRED on
> `task/W4-g51-net-apply-arms-4`; 139's join push is the g52c master input
> (`Server::slot_counts_as_host`, IPv4-loopback-exact). The per-id verdict
> table + the union keep-one hunks + the net-121-family bible:
> `359-g51-net-te-family-seal.md`; the sibling coverage census:
> f21b's `276-f21b-net-apply-audit.md` (239→276 per registry 150 §6.3).

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
| 13 | **`18 SetTime` unported** — the clock rides internal `205 Time` at 10 Hz instead of C#'s join-time `7` + world flags (ANALYSIS.md:7275-7278). CORRECTION re-probe: the C# never sends 18 at all (zero census sites) — the faithful target is the id-7 event cadence + the 3600-tick heartbeat; the switch instruction is research/310 (c28's). | behavioural | yes |
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
  flags being carried in `7 WorldInfo`. (NET-APPLY-1 re-probe: the reader
  half LANDED on c27 `412b6bc0d`; the remaining scope is c28's send-switch
  cadence — see the FINAL census verdict above. CORRECTION: the C# sends
  ZERO 18s — the clock rides `7 WorldData`; the exact switch instruction
  is research/310.)
- **M — Remote-player smoothing parity**: document or port the C#/ReLogic interpolation
  so `net_interp_tick`'s constants are not arbitrary. Depends on: nothing (research).
- **L — The per-system passes** ROADMAP.md:1067-1071 parks: peer inventories, creative
  powers' per-player split, pylon/bestiary module bodies, the exact clock path.
  Depends on: the join bundle and the chest round trip.
- **L — Dedicated-server mode** (`Main.dedServ`, `ServerMode.cs`, `NetManager`): the
  port has no headless host path; the branch is exercised in-process only.

## The g52c-139-master receipt (W4-g48 NET-APPLY-1, 2026-11-21)

**Receipt.** The task's cross-note pointed at `research/139` on
`origin/task/W4-g52` as the master-final table. Probe result at exec
time: the pushed g52 lane is `task/W4-g52-net-relay-host` tip `590b93565`
— it carries NO research/139 and no 1-40 master-final table (its docs
are the ANALYSIS "The SendData host relay gating" relay-census section,
`NetMessage.cs:1692-1826`). The g52c-net-master sub-round is UNPUSHED —
the research-number registry (research/150, d54 tip) reserves 139 for it
and records the g89 collision as ONE-SIDED (g89 renamed its own 139 →
145, `41dabfbc8`). This lane therefore does NOT take 139 (its own
final-phase doc took the free 237 — final rename chain 237→239→272 per
research/150 round 2 §6.3: 237 belongs to e23's earlier push, 239 to
d42's 12:55:30 push, research/272); the receipt
of what g52's pushed tip DOES carry:

- **The 1-40 statuses feed:** g52's relay census explicitly defers the
  client applies to "g48-g51" — its `21` instanced/air send gate, `162`
  damage-ack send and `34` host place/destroy pass all name the apply
  side as this lane family's. The g48 FINAL table above IS the 1-40
  apply-status feed for the g52c master table when it lands: 34 WIRED
  g48 (`f086297b`) overlaps g52's host remainder (`cdc8fa7b8` +
  `e2a5968a4` — the destroy-arm `can_kill_tile`/`chest_holds_items` gate
  + store-removal compensation is g52's LATER refinement; g48's
  `host_chest_updates` destroy arm has neither, re-read at
  `net_apply.rs:920-957`; the merger takes g52's destroy arm over
  g48's, keep-one).
- **The 6-codec-branches keep-one instruction:** g52 replicated the g48
  `34 ChestUpdates` codec and the d92 `162 NpcStrikeAck` codec
  BYTE-VERBATIM ("keep-one at merge", its ANALYSIS deviations +
  remainder-round note). Re-verified at both tips: the `ChestUpdates`
  struct (action u8 + x/y/style/chest i16) and both codec arms
  (`NetMessage.cs:913-927`, the server `number5` short) are identical
  on g48 and g52 — KEEP ONE copy at the union, either direction is
  byte-safe. The `162` codec exists on g52 (`NpcStrikeAck` bodyless)
  and g51 (`51d01bc87` — the pending-damage queue apply); g48 has
  NEITHER (out of its 1-40 scope) — the merger keeps g51's apply arm
  with either codec copy.

## NET-APPLY-1 declaration (W4-g48 FINAL, 2026-11-21) — research/17 tail SEALED

**Declaration.** The g48 apply-arms lane 1 (`task/W4-g48-net-apply-arms-1`)
is COMPLETE for its scope: the update-compatible ids 1-40 host/client
apply arms are wired on this tip per the FINAL census above (30 WIRED /
4 N/A / 4 DEAD / 2 PARKED-with-owner), the five cited skips closed
(ANALYSIS "The g48 cited-skip closures"), the six codec branches merged
in (d89 → d90 → d91 → d93 → d96 → d98, the keep-one/keep-both merge
commits `a190d705e`/`178d14e17`/`e241c25bd`/`9c40ed2d1`/`8802562ce`/
`648c7cdbd`), and 17 lane witnesses WRITTEN NOT RUN (16 `g48_*` loopback
+ 1 minecart fail-tail core pin, research/272). No compiler invoked
anywhere in the lane (code-only mode); verification = whole-arm C#
re-reads + byte-identity hash pins of the landed canonical modules.

**SEAL.** This tail (the f21 cross-note table, the FINAL census verdict,
the g52c receipt, this declaration) is SEALED for the g48 lane: later
rounds edit the rows above only with a re-probe citation, never re-open
the scope. The lane's remaining external dependencies are recorded, not
owned: 18's send switch (c28), 39's FindOwner arm (the item lane), the 34
destroy-arm refinement (g52), the 17 kinds 18/19 SetCurrentUser
attribution (M7-WIRE-2), the 13 simulation tails (c27/d96).

## NET-APPLY-1 correction-phase receipt (W4-g48, 2026-11-21) — the SEALED tail stands

The correction round ran UNDER the seal (re-probe citations only, no
scope re-open, no code change). Receipt of the five items:

1. **The 237 triple-collision → this lane's doc RENAMED 237→239
   (superseded: final chain 237→239→272, see the REGISTRY-RENAME
   receipt below).**
   The research/150 registry (d54 tip `38bfd1d22`, §3 rules:
   first-pushed-timestamp-wins, losers renumber on their OWN branch)
   adjudicates: e23 `237-bestiary-ui-final.md` first-add `7a7779690`
   2026-11-21T10:53:03+02:00 KEEPS 237; g61
   `237-g61-spawn-family-final.md` `fd406ecba` 12:12:59 → **238** (that
   lane's own rename; round 2: also lost, → 273); this lane's
   `237-f11-tail-fix.md` `34029437a` 12:15:09 → **239** (round 2: 239 =
   d42's earlier push, → **272**). The then-358-tip scan found exactly
   these three 237 claims and no 238/239 claim; the round-2 refresh's
   later sweep found d42's 238/239 pushes. Rename + all refs executed on
   this branch (INDEX row, ANALYSIS final-phase note, PORT_MAP minecart
   row, this file's witness counts + the g52c receipt note).
2. **The f11-minecart-mute-fix consumption:** the f11e-retire wave
   (tip `14ca0a8d5`, phase 1b landed the 233/234/235 registry renames)
   did NOT consume the no-op hunk — `minecart.rs` blob `5efef353`
   UNCHANGED from `84ed6342b`, the killing `kill_tile(world, i, j,
   true)` still at tip lines 360-362. **The f11 branch still carries
   the defect.** The exact hunk is re-published (unchanged bytes,
   re-pinned lines) in research/272 for the f11f-seal/merger; the e20
   union-survivor fail slice re-probed ALREADY correct (the
   condition-return + the app-side no-kill 314 slice, tip `46fdcfd8e`),
   so the hunk binds only on any f11-body-survives merge path.
3. **The c28-18-send-switch publish:** research/310 (published as 244,
   chain 244→281→310 — the REGISTRY-RENAME round, then the UNIFIED
   FINAL §9.2) — the C# has ZERO
   `SendData(18)` sites (c28's own census json, no 18 key); the live
   clock sync rides `7 WorldData` (the event edges + the 3600-tick
   `UpdateServer` heartbeat). The "205→18" framing is corrected to
   "internal 205 → the id-7 cadence", the c27 18 reader kept
   wire-compat; the exact 5-step switch instruction is published for
   the c28 lane (its tip `6379f881b` remains census-only — the switch
   is still pending there).
4. **The 39-ReleaseItemOwnership item-lane task-seed:** research/318
   (published as 245, chain 245→282→318 — the REGISTRY-RENAME round,
   then the UNIFIED FINAL §9.2) —
   the lane adjudication (NOT g49/g50/g51, grep-empty re-verified at
   `af1f172f5`/`714f1f736`/`fcfb92f26`; NOT the landed M6-NET-5 ATLAS
   row — that is the 27/28/29 sync; the ROADMAP S1-I-4a "stay M6-NET-5"
   note is a forward-reference to the milestone remainder; the ITEM
   lane / d98 region owns it) + the full `WorldItem.cs:188-285`
   FindOwner/ReserveFor scope pin + the unrowed reservation-fields
   census + the peer-inventory dependency.
5. **This file's number:** `17-netcode.md` is the BASE-TRUNK netcode
   master (all 358 tips carry the identical filename; the registry's
   collision set covers only NEW W4 docs — 17 is NOT a collision key
   and needs NO rename). The lane's renamed doc is the 272 tail-fix
   publish (item 1, final chain 237→239→272); the SEAL above stands
   unchanged — this receipt is
   appended AFTER it, the rows above carry only re-probe citations.

Verification: code-only — no cargo/rustc/tests/rustfmt/graphify
invoked. All tip hashes re-fetched this round (`git fetch origin
+refs/heads/*:refs/remotes/origin/*`, 358 task tips); the first-add
timestamps via `git log --diff-filter=A --format=%cI`; the C# windows
re-read direct (`NetMessage.cs:224-260`/`:531-536`/`:675-684`/
`:948-951`, `MessageBuffer.cs:473-497`/`:1290-1298`/`:1552-1562`/
`:2232-2255`, `Main.cs:64546-64604` + the id-7 edge sites,
`WorldItem.cs:188-285`, `Wiring.cs:2610-2635`); the f11/e20/c27/c28/
f21/d98/sibling blobs re-read at the pinned hashes.

## NET-APPLY-1 REGISTRY-RENAME receipt (W4-g48, 2026-11-21) — the SEALED tail stands

The research/150 round-2 refresh (d54 tip `72226e500`, §6/§6.3) landed
AFTER this lane's correction push (`2b61530b4` 14:30:16 — outside the
refresh's final sweep; the cutoff note applies: post-sweep pushes
self-adjudicate, losers take free numbers ≥281 after a fresh all-tip
sweep). Three renames executed on this branch, docs-only:

1. **`239-f11-tail-fix.md` → `272-f11-tail-fix.md`** — §6.3 g48 row
   (the registry's sweep saw the initial 237 add `34029437a` 12:15:09,
   not the 14:30:16 self-rename push): chain **237→239→272**. 237 = e23
   `7a7779690` 10:53:03 (KEEPS); 239 = d42 `bac408be0` 12:55:30 (both
   earlier than this lane's claims; g56's 239 → 274, f21's → 276,
   g61's 237→238 chain → 273).
2. **`244-c28-18-send-switch.md` → `281-c28-18-send-switch.md`** — 244
   sits inside the refresh's consumed 244–280 assignment block
   (= c31 jungle-residuals, §6.3); a fresh 362-tip sweep also found
   post-sweep 244 claims by f39 (`8704691a3` 14:32:01) and d53
   (`a106d13e8` 14:56:57). **281 unclaimed at every tip.** (Chain
   history — the 281 self-pick was later ruled CONTESTED by the
   UNIFIED FINAL §9.2: chain extended 244→281→**310**, receipt below.)
3. **`245-39-item-ownership-seed.md` → `282-39-item-ownership-seed.md`**
   — 245 = the d81/g53 `18b-element-census` fork assignment (§6.3),
   never free. **282 unclaimed at every tip** (the only 280+ doc
   anywhere: c42's assigned `280-c42-wiring-seal.md`). (Chain history
   — likewise CONTESTED per §9.2: chain extended 245→282→**318**.)

All refs re-pointed: INDEX rows 272/281/282 + the lane note, ANALYSIS
(the final-phase + correction-phase sections + the REGISTRY-RENAME
section), PORT_MAP minecart row, this file (rows 18/39 + the FINAL
verdict bullets + the deviations row 13 + the task seeds + the witness
counts + the g52c note + the correction receipt items 1/3/4/5, each
carrying the rename citation). `17-netcode.md` itself stays 17 (the
base-trunk master, not a collision key). The SEAL stands — renumber
citations only, no scope re-open, no code change, no
cargo/rustc/tests/rustfmt/graphify invoked.

## NET-APPLY-1 UNIFIED FINAL receipt (W4-g48, 2026-11-21) — §9.2 renumbers, the SEAL stands

The research/150 UNIFIED FINAL (d54 tip `b243c7086`, §9.2) rules the
round-2 self-renames above CONTESTED: 281/282 belong to f26's §7.3
assignments (`68-findframe-census-addendum-7h/7i.md`) — the 281–303
block was RESERVED at §7.3 publication, self-picks into it are NOT
honored. Two renames executed on this branch per the §9.2 rows:

1. **`281-c28-18-send-switch.md` → `310-c28-18-send-switch.md`** —
   chain **244→281→310** (§9.2 row 310). **g48's 244 KEY WIN stands**
   (§7.1/§7.4, pre-publish 14:30:16) — the key stays winner-consumed,
   only the doc filename renumbers.
2. **`282-39-item-ownership-seed.md` → `318-39-item-ownership-seed.md`**
   — chain **245→282→318** (§9.2 row 318). **g48's 245 KEY WIN stands**
   (§7.1, pre-publish) — likewise winner-consumed.

All refs re-pointed: INDEX rows **310/318** + the lane note, ANALYSIS
(the correction-phase pointers + the REGISTRY-RENAME chain-history +
the new UNIFIED FINAL section), research/272's sibling-chain note,
this file (rows 18/39 + the FINAL verdict bullets + deviations row 13
+ the task seeds + the correction receipt items 3/4 + the
REGISTRY-RENAME receipt chain-history annotations), and the two docs'
headers/chain-banners. The SEAL stands — renumber citations only, no
scope re-open, no code change, no cargo/rustc/tests/rustfmt/graphify
invoked.
