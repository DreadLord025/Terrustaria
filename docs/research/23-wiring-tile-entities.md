# 23 — Wiring / TileEntities

> Wave 2 · researched 2026-11-12 · sources: decompiled (Terraria 1.4.5.8) + crates (Terraria-Rust) + docs/ATLAS.md §19, docs/ROADMAP.md M7 rows, docs/ANALYSIS.md "Wiring engine / Tile entities / Logic gates"

## Scope

| C# source | lines | Rust target |
| --- | --- | --- |
| `Terraria/Wiring.cs` | ~3 430 (ATLAS: 3 431) | `crates/tr-core/src/wiring.rs` (205 647 B), `crates/tr-app/src/world_sim/wiring.rs` (30 869 B) |
| `Terraria/DataStructures/TileEntity.cs` | 344 | `crates/tr-core/src/tile_entities.rs` (72 300 B) |
| `Terraria/DataStructures/TileEntitiesManager.cs` | ~90 | `crates/tr-core/src/tile_entities.rs::TeKind` |
| `Terraria/GameContent/Tile_Entities/*.cs` | 16 files (26 740 B doll … 1 171 B anchor base) | `crates/tr-core/src/tile_entities.rs` |
| `Terraria/IO/WorldFile.cs` §Save/LoadTileEntities | `:3302` / `:3315` | `crates/tr-core/src/wld/sections/tile_entities.rs` (48 434 B) |
| `Terraria/GameContent/PressurePlateHelper.cs` | ~170 | — (no port found) |

Responsibility: the wiring engine turns a switch/lever/plate press into a four-colour flood fill over the wire tiles, and every wire-connected device (lamps, gates, traps, statues, teleporters, pumps, dials, cannons) reacts to it. The tile-entity framework stores the world objects that own an item slot, a sensor state or a leashed entity, keyed by the registration order that doubles as the save/net byte.

## Key entities

### C# (`decompiled/Terraria/Wiring.cs`)

- `Wiring.TripWire(int left, int top, int width, int height)` (`:525-666`) — the four colour passes in the order **1 red → 2 blue → 3 green → 4 yellow**, each pass collecting the colour's tiles in the rect, resetting the teleport pads and the pump queues, calling `HitWire`, then `XferWater` if both pump queues are non-empty. After the passes: `running = false`, the four pad pairs `Teleport()`ed (`:654-663`), then `PixelBoxPass()` (`:664`) and `LogicGatePass()` (`:665`).
- `Wiring.HitWire(DoubleStack<Point16> next, int wireType)` (`:837-979`) — the BFS: seeds get `SkipWire` + `_toProcess[cell] = 4`; the 4-neighbour walk is ordered **j=0 down (y+1), j=1 up (y−1), j=2 right (x+1), j=3 left (x−1)**; bounds `[2, maxTilesX-2]`; a cell already in `_toProcess` decrements the counter and is *not* re-enqueued; a fresh cell is enqueued with `_toProcess[cell] = b` where `b = 3`, or `b = 0` when the **neighbour** tile is a 424 wire pipe / 445 pixel box (`:900-904`, `:971-974`). The 424 cell also filters the propagation direction by `frameX / 18` (0 straight, 1 cross, 2 corner — `:905-928`); the 445 cell records a trigger byte in `_PixelBoxTriggers` (`:929-943`). `_wireSkip` and `_toProcess` are cleared at the end.
- `Wiring.HitWireSingle(int i, int j)` (`:996-2554`) — the ~70-arm device switch: 144 timer, 421/422 lamp swap, the `255..268` type-pair swap (`:1034-1050`), 419 logic lamp, 406/452/411/425 frame shifts, 356/663 dials, 209/212 cannons, 335/338 fireworks, 235 teleporter, 429, 149/244/565/42/93/95/100/126/173/564 the light family, 593/594 volcanoes, 137 traps, 443 geyser, 105 statue, 349 mushroom, 531 boulder, 142/143 pumps, 141/210 explosives/mine, plus the `Toggle*` helpers (`:2800-3073`).
- `Wiring.HitSwitch(int i, int j)` (`:259-378`) — the switch table: `135|314|423|428|442|476` → 1×1 trip; `440` → 3×3 trip; `136` → frameY toggle + 1×1; `210` → mine; `443` → geyser; `144` → frameY toggle + `CheckMech(18000)`; `441|468` → 2×2 trip; `467` with `frameX/36 == 4` → 2×2 trip; **else `132` (lever) or `411` (detonator)** → ±36 frame shift on the 2×2 block, `CheckMech(60)` for 411, 2×2 trip (`:343-377`).
- `Wiring.UpdateMech()` (`:145-257`) — the cooldown queue: three cannon cooldowns, the tile-144 timer periods **60/180/300/30/15 by `frameX/18`** (`:186-204`), the `IEEERemainder(t, p) == 0` re-trip at 18000, the expiry path (144 frameY reset, 411 2×2 lamp-pair frame flip), the back-to-front walk and the shift-down compaction.
- `Wiring.CheckMech(int i, int j, int time)` (`:455-473`) — one entry per position, **cap 999** (arrays of 1000).
- `Wiring.SkipWire` (`:115`, `:120`), `Wiring.ClearAll` (`:125`), `Wiring.Initialize` (`:90`, the `_wireSkip`/`_wireList`/`_toProcess`/`_Gates*`/`_LampsToCheck`/`_PixelBoxTriggers`/pump/mech arrays), `Wiring.running` (`:531`, `:654`).
- `Wiring.XferWater()` (`:475-523`) — the pump transfer (cap 255 per tile, the type lock, the per-transfer liquid re-read at `:506`).
- `Wiring.DeActive`/`ReActive` (`:3195`/`:3225`), `Wiring.Actuate`/`ActuateForced` (`:389`/`:407`), `Wiring.Teleport` (`:3120-3189`), `Wiring.TeleporterHitboxIntersects` (`:3185`), `Wiring.ExplodeMine` (`:3074`), `Wiring.GeyserTrap` (`:3084`), `Wiring.Extractinator` (`:2554`), `Wiring.Hopper` (`:2638`), `Wiring.MassWireOperation` (`:419`, inner `:3235`).
- Callers of `HitSwitch`: `Player.cs:32177` (lever/switch/timer use), `:32339` + `:34823` (detonator), `Collision.cs:3303-3346` (player contact: plates, mine 210, geyser 443), `Minecart.cs:1286`, `PressurePlateHelper.cs:161`, `WorldGen.cs:46522`/`:62588`, `MessageBuffer.cs:1890`/`:2687` (net).

### C# tile entities

- `TileEntity` (`DataStructures/TileEntity.cs`) — the static books `ByID`/`ByPosition`/`UpdateEntities` (`:15-22`), `MaxEntitiesPerChunk = 1000` (`:14`), `Write`/`Read` (`:173-185`) → type byte + `WriteInner` (`:186-195`: `ID` int32 when not a net send, `Position.X`/`Y` int16) + `WriteExtraData`/`ReadExtraData` (`:209-211` empty by default), `PerformUpdates` (`:56`), `Place`/`Kill`/`Remove`/`TryGetAt`.
- `TileEntitiesManager.RegisterAll()` (`DataStructures/TileEntitiesManager.cs:26-37`) — the id table: **0 TETrainingDummy · 1 TEItemFrame · 2 TELogicSensor · 3 TEDisplayDoll · 4 TEWeaponsRack · 5 TEHatRack · 6 TEFoodPlatter · 7 TETeleportationPylon · 8 TEDeadCellsDisplayJar · 9 TEKiteAnchor · 10 TECritterAnchor**.
- Payloads: `TEItemFrame`/`TEWeaponsRack`/`TEFoodPlatter`/`TEDeadCellsDisplayJar` → `(short type, byte prefix, short stack)`; `TEHatRack` → presence bitsByte + 2 items + 2 dyes; `TETrainingDummy` → `(short npc`; `TELogicSensor` → `byte logicCheck` + `bool On`; `TELeashedEntityAnchorWithItem` (kite/critter) → `(short itemType)`; `TEDisplayDoll` → 3 bitsBytes + pose + up to 19 triples; `TETeleportationPylon` → nothing.

### Rust

- `crates/tr-core/src/wiring.rs::WireColor` (`:80`, `Red=1 Blue=2 Green=3 Yellow=4`), `WireState` (`:91`, the four sparse `HashSet`s + the `.trwld` wire section at `:152`/`:175`), `Wiring` (`:207`), `StatueSpawn` (`:43`), `DeviceShot` (`:274`), `TrapRequest` (`:292`).
- `Wiring::check_mech` (`:393`), `Wiring::update` (`:408`), `Wiring::hit_switch` (`:496`), `Wiring::trip_wire` (`:549`), `Wiring::statue_trigger` (`:801`), `Wiring::xfer_water` (`:1661`), `Wiring::check_logic_gate` (`:1760`), `Wiring::trip_wire_color` (`:1852`), `Wiring::hit_wire` (`:1884`), `Wiring::hit_wire_single` (`:1930`), `Wiring::actuate_forced`/`de_active`/`re_active` (`:2132`/`:2151`/`:2191`), `faulty_next` (`:372`) + `rand_next` (`:387`), `TO_PROCESS_COUNT = 4` (`:333`), `MAX_MECHS = 999` (`:335`).
- `crates/tr-core/src/tile_entities.rs::TeItem` (`:48`), `TeKind` (`:81`), `TeKind::from_id` (`:96`), `TeKind::tile` (`:114`), `TeKind::is_tile_valid` (`:138`), `TeData` (`:194`), `TileEntity` (`:301`), `TileEntities` (`:342`) with `place`/`remove_at`/`set_item`/`write_all`/`read_all`/`live_entities`, `update_logic_sensors` (`:721`), `figure_sensor_state` (`:789`), `update_training_dummies` (`:831`), `hat_rack_write`/`hat_rack_read` (`:885`/`:898`).
- `crates/tr-core/src/wld/sections/tile_entities.rs::read`/`read_entity`/`read_extra` (`:102`/`:116`/`:134`), `skip_display_doll` (`:182`), `write`/`write_extra` (`:231`/`:245`), `apply_loaded` (`:299`), `drop_unstorable` (`:322`), `is_storable` (`:350`), `store_data` (`:359`), `from_world` (`:400`).
- `crates/tr-app/src/world_sim/wiring.rs::update_wiring` (`:19`) — the 60 Hz tick, the statue/trap/shot drains, `statue_npc` (`:294`), `spear_cap_allows` (`:318`), `bunny_cannon_can_fire` (`:365`), `fountain_anchored` (`:386`), `spawn_device_kind` (`:396`).
- `crates/tr-app/src/world_sim/tile_entities.rs::update_tile_entities` (`:14`) — the 60 Hz `te_step`, the player hitboxes, the sensor scan.
- `crates/tr-app/src/app/mod.rs:2931` — `app.wiring.seed_faulty_rng(world_seed | 1)`.

## Dependencies

- Calls into: `WorldGen.SquareTileFrame`/`TileFrame`/`InWorld`/`KillTile`, `WorldGen.TileFrame` (the frame re-layout after a device acts), `Collision.SolidTiles` (the statue body gates), `NPC.MechSpawn`/`NPC.NewNPC`/`Item.MechSpawn`/`Item.NewItem` (the caps + the spawns), `Player`/`NPC`/`Projectile` (the teleport moves), `SoundEngine.PlaySound(28, …)` (the switch clicks), `NetMessage.SendData(59/106/110/…)` (the net sync), `TileEntity.manager`/`GenerateInstance`, `Main.rand` (the statue rolls), `Main.Sundialing`/`Moondialing`.
- Called by: `Player` (lever/switch/timer/detonator use), `Collision` (the player-contact switch), `Minecart` (the pressure track), `PressurePlateHelper.PokeLocation`, `WorldGen` (dungeon spawns), `MessageBuffer` (net), `WorldGen.UpdateWorld` → `TileEntity.PerformUpdates` (`:71507`).
- Shared data structures: `World.wires: WireState` (four colours), `World.tile_entities: TileEntities` (the `ByID`/`ByPosition` books), `World.pressure_plates` (the pressed weighted plates), the `CheckMech` queue (999 entries), the `Wiring` struct's own RNG stream, the app's NPC/projectile pools (the `MechSpawn` caps).

## Propagation order — 1:1 check

The order is the part of the wiring engine that a behavioural difference shows up in, so it was checked step by step against the C#.

| Step | C# | Rust | Verdict |
| --- | --- | --- | --- |
| colour pass order | 1 `wire()` red → 2 `wire2()` blue → 3 `wire3()` green → 4 `wire4()` yellow (`Wiring.cs:540-653`) | `WireColor::COLORS` = Red, Blue, Green, Yellow (`wiring.rs:137-142`), iterated in `trip_wire` (`:551-556`) | 1:1 |
| the seed set per pass | every tile in the rect whose colour bit is set, scanned row-major (`Wiring.cs:542-553`, `:570-581`, `:602-613`, `:630-641`) | the same rect scan, filtered by `world.wires.get(i, j, color)` (`wiring.rs:1869-1872`) | 1:1 |
| the pump-queue reset | `_numInPump = 0; _numOutPump = 0;` before each pass (`:560-561`, `:589-590`, `:617-618`, `:645-646`) | `in_pumps.clear(); out_pumps.clear();` per pass (`wiring.rs:1864-1865`) | 1:1 |
| the teleport-pad reset | `_teleport[0..1] = -1,-1` before each pass (`:554-557`, `:582-585`, `:598-601`, `:626-629`) | `self.teleport = [(-1.0,-1.0); 2]` per pass (`wiring.rs:1863`) | 1:1 |
| the `XferWater` gate | after each pass, only when both queues are non-empty (`:563-566`, `:591-594`, `:619-622`, `:647-650`) | `if !in_pumps.is_empty() && !out_pumps.is_empty()` (`wiring.rs:1875-1877`) | 1:1 |
| the BFS queue | `DoubleStack<Point16>` popped from the **front** (`:851`), the seeds pushed back after the counter setup (`:840-847`) | `VecDeque`, `pop_front` (`wiring.rs:1893`), the seeds enqueued in scan order (`:1887`) | 1:1 |
| the neighbour order | j=0 → (x, y+1), j=1 → (x, y−1), j=2 → (x+1, y), j=3 → (x−1, y) (`:863-885`) | `[(0,1),(0,-1),(1,0),(-1,0)]` (`wiring.rs:1897`), the direction byte via `dx_y_to_dir` (`:2290`) | 1:1 |
| the bounds | `num2 < 2 \|\| num2 >= maxTilesX-2 \|\| num3 < 2 \|\| num3 >= maxTilesY-2` (`:886-889`) | `nx < 2 \|\| nx >= width-2 \|\| ny < 2 \|\| ny >= height-2` (`wiring.rs:1900`) | 1:1 |
| the multi-trigger counter | 4 on the seeds (`:844`), `b` (3) on an enqueue (`:973`), decremented on a re-encounter and removed at zero (`:956-967`) | `TO_PROCESS_COUNT = 4` (`:333`), 3 on an enqueue (`:1920`), the same decrement/remove (`:1907-1913`) | 1:1 in arithmetic, **wrong cell** for the `b = 0` rule (D1) |
| the device re-fire | the counter is *removed* at zero, so a later path re-enqueues the cell and the device fires again | the cell is skipped after its first firing, so it never re-fires (D2) | **diverges** |
| the pad pair collection | eight `Vector2`s, the pair of each pass copied out (`:568-569`, `:596-597`, `:624-625`, `:652-653`) | `pairs[pass]` per pass (`wiring.rs:558`) | 1:1 |
| the tail | `running = false` → the four `Teleport()` calls (`:654-663`) → `PixelBoxPass()` (`:664`) → `LogicGatePass()` (`:665`) | `pixel_box_pass` → the four teleports → `logic_gate_pass` (`wiring.rs:560-569`) | **order differs** (D3) |
| the statue spawn order | the style's `SelectRandom(Main.rand, …)` roll happens **inside the switch, before every gate** (`:2167`, `:2229`, `:2251`), and the second table runs only when `num142 <= -1` (`:2279`) | the same eager roll in the match arm (`:840-881`), the second table only when the first did not spawn (`:902`) | 1:1 in order; the **stream** differs (D14) |
| the statue cap order | `CheckMech` before `NPC.MechSpawn` before `NewNPC` (`:2266-2270`) | `check_mech` → the cap ids ride the request → the app applies the cap (`wiring.rs:888-898`, `world_sim/wiring.rs`) | 1:1 (the cap lives app-side, where the entity lists are) |

## `.wld` tile-entity section — the byte layout

The C# writer is `WorldFile.SaveTileEntities` (`WorldFile.cs:3302-3314`): `writer.Write(TileEntity.ByID.Count)`, then per entity `TileEntity.Write(writer, ent)` (`TileEntity.cs:173-176`) → the type byte, then `WriteInner` (`:186-195`) → the `ID` int32 (only when it is not a net send), `Position.X` int16, `Position.Y` int16, then `WriteExtraData`. The reader `LoadTileEntities` (`:3315-3358`) clears the books, reassigns the ids sequentially (`:3323-3324`), replaces a same-position record (`:3324-3328`), drops the out-of-world and invalid-tile records (`:3335-3353`) and finally calls `OnWorldLoaded` on every entity (`:3355-3358`).

| C# kind | C# extra bytes | Rust `.wld` codec | Verdict |
| --- | --- | --- | --- |
| 1 ItemFrame / 4 WeaponsRack / 6 FoodPlatter / 8 DeadCellsDisplayJar | `(short type, byte prefix, short stack)` | `WldTeExtra::Item` — the same three fields (`sections/tile_entities.rs:246-251`) | byte-identical |
| 5 HatRack | the presence bitsByte + 2 items + 2 dyes | `WldTeExtra::HatRack` — the bitsByte then the present triples (`:252-268`) | byte-identical |
| 0 TrainingDummy | `(short npc` | `WldTeExtra::Dummy { npc: i16 }` (`:269`) | byte-identical |
| 2 LogicSensor | `byte logicCheck` + `bool On` | `WldTeExtra::Logic { check, on }` (`:270-273`) | byte-identical |
| 9 KiteAnchor / 10 CritterAnchor | `(short itemType` | `WldTeExtra::Anchor { item_type: i16 }` (`:274-276`) | byte-identical |
| 7 TeleportationPylon | none (`TileEntity.cs:209-211`) | `WldTeExtra::None` | byte-identical |
| 3 DisplayDoll | 3 bitsBytes + the pose byte (≥307) + the misc byte (≥308) + ≤19 triples, with the version-311 tail rule | read and discarded (`skip_display_doll`, `:182-217`), the record written back empty, and the entity **dropped** from the books (`is_storable`, `:350`) | **lossy** (D9) |

The id reassignment, the same-position replacement, the `InWorld(x, y, 1)` margin, the `CheckValidTile` drop and the `OnWorldLoaded` walk are all present in `apply_loaded` (`:299-320`) and `drop_unstorable` (`:322-338`).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `TripWire` four colour passes | PORTED | `wiring.rs::trip_wire` (`:549`), `trip_wire_color` (`:1852`) | Order red→blue→green→yellow matches `Wiring.cs:525-653`; per-pass pad/pump reset and `XferWater` gate match `:558-567` |
| `TripWire` tail order (Teleport → PixelBox → LogicGate) | PARTIAL | `wiring.rs:560-569` | The Rust runs `pixel_box_pass` **before** the teleport loop; the C# runs it **after** (`Wiring.cs:654-665`) |
| `HitWire` flood (BFS, bounds, `_toProcess`) | PARTIAL | `wiring.rs::hit_wire` (`:1884`) | Order, bounds `[2, max-2]`, counter 4/3 and the "removed at zero" re-fire are right; the 424/445 handling is wrong (see D1) and the device skip is unconditional (D2) |
| `HitWireSingle` device arms | PARTIAL | `wiring.rs::hit_wire_single` (`:1930`) | ~22 of the C#'s ~70 arms; the light family, the `255..268` swap and several small arms are absent (D8) |
| `HitSwitch` rect table | PARTIAL | `wiring.rs::hit_switch` (`:496`) | All arms except the `132`/`411` else-arm are present (D5) |
| `CheckMech` (999 cap) | PORTED | `wiring.rs::check_mech` (`:393`) | `Wiring.cs:455-473` verbatim |
| `UpdateMech` (timers 60/180/300/30/15, 411 flip) | PORTED | `wiring.rs::update` (`:408`) | `Wiring.cs:145-257`; the queue compaction differs on the out-of-world branch (D16) |
| `XferWater` (pumps 142/143) | PORTED | `wiring.rs::xfer_water` (`:1661`), `trip_wire_color` (`:1874`) | 255 cap, type lock, per-transfer re-read match `Wiring.cs:475-523` |
| Statues (105) — style tables, order, caps | PORTED | `wiring.rs::statue_trigger` (`:801`) + `world_sim/wiring.rs` | The first/second table arms, the `num142 <= -1` fall-through, the style-63 veto and the item-cap ids match `Wiring.cs:2125-2470`; the **RNG source differs** (D14) |
| Traps (137) / cannons (209, 212) / fireworks (335, 338) / explosives (141) / mine (210) / geyser (443) | PORTED | `wiring.rs::trap_emit` (`:645`), `cannon_fire` (`:1183`), `snowball_fire` (`:1329`), `launch_rocket*` (`:1400`/`:1437`), `firework_fountain` (`:1478`), `explosives` (`:1504`), `explode_mine` (`:1522`), `geyser_trap` (`:1540`) | Device arms + the app-side caps; the projectile families 167-174/415-418/281/654/733/99 are still unported (ROADMAP's own note) |
| Logic gates (419/420) | PORTED | `wiring.rs::logic_gate_pass` (`:1730`), `check_logic_gate` (`:1760`), `poke_logic_gate` (`:1721`) | The six types by `frameY/18`, the lamp-stack count, the broken-lamp mark match `Wiring.cs:682-831` |
| Teleporters (235) | PORTED | `wiring.rs` `235` arm (`:1992`), `teleport` (`:588`), `world_sim/wiring.rs` | The pad registration, the reject rule and the 48×48 boxes match `Wiring.cs:3120-3189` |
| Actuators (`ActuateForced`/`DeActive`/`ReActive`) | PORTED | `wiring.rs::actuate_forced` (`:2132`), `de_active` (`:2151`), `re_active` (`:2191`) | The guard chain (226, NotReallySolid, the exceptions, `PreventsActuationUnder`, `CanKillTile`) matches `Wiring.cs:3195-3232` |
| Pressure plates (135/428/442) — the presser | MISSING | — | `PressurePlateHelper.Update/PokeLocation` (`PressurePlateHelper.cs:19`, `:156-161`) has no port; only the `.wld` section (`wld/sections/pressure_plates.rs`) and the worldgen placement exist |
| Player→switch bridge | MISSING | — | `Player.cs:32177`/`:32339`/`:34823`, `Collision.cs:3303-3346`, `Minecart.cs:1286` have no Rust caller of `hit_switch` (only the sensor trip at `tile_entities.rs:780`) |
| Player wire/switch placement | MISSING | — | `World.wires.set` has no production caller outside `worldgen/temple.rs:1709` and the net load `netglue.rs:720` |
| `Wiring.running` guard | MISSING | — | `Wiring.cs:531`/`:654`; `WorldGen.cs:31895`/`:37717`/`:50994`+ guard world-gen during a flood |
| `MassWireOperation` (wire tool, item consumption) | MISSING | — | `Wiring.cs:419-453` + `:3235-3422` |
| `TileEntity` framework (books, ids, placement) | PORTED | `tile_entities.rs::TileEntities` (`:342`), `TeKind` (`:81`) | The id table matches `TileEntitiesManager.cs:26-37`; `is_tile_valid` (`:138`) matches the per-TE `IsTileValidForEntity` rules |
| TE item payloads (frame/rack/platter/hat rack) | PORTED | `tile_entities.rs::TeData::Item`/`HatRack` (`:194`) | The `(type, prefix, stack)` triple + the hat-rack presence bitsByte match `TEItemFrame.cs:47-56` / `TEHatRack.cs:59-80` |
| Logic sensor behaviour | PORTED | `tile_entities.rs::update_logic_sensors` (`:721`), `figure_sensor_state` (`:789`) | The 7 check types, the 82×162 player box, the liquid grace and the rising-edge trips match `TELogicSensor.cs`; driven from `world_sim/tile_entities.rs:31` |
| Training dummy behaviour | PARTIAL | `tile_entities.rs::update_training_dummies` (`:831`) | The state machine exists but has **no production caller** (only the tests) (D15); the NPC 488 spawn also waits for the M4-NPC roster |
| Display doll / pylon / anchors / Dead Cells jar | MISSING | ids only (`TeKind`) | The doll's payload is read-skipped (`wld/sections/tile_entities.rs::skip_display_doll`, `:182`), the pylon network and the leashed anchors are absent; `OnWorldLoaded` is a no-op (`tile_entities.rs:450`) |
| `.wld` TE section (read + write) | PARTIAL | `wld/sections/tile_entities.rs` | The count + `(u8 ty, i32 id, i16 x, i16 y)` + per-kind extra match `WorldFile.cs:3302-3333`; the doll is dropped instead of carried (D9) |
| `TileEntity.MaxEntitiesPerChunk = 1000` | MISSING | — | `TileEntity.cs:14`; the books are unbounded (documented in ANALYSIS) |

### Switch-type coverage (`HitSwitch` + the pressers)

| Device | tile | C# behaviour | Rust |
| --- | --- | --- | --- |
| Lever | 132 | `HitSwitch:343-377` — ±36 frame shift, `CheckMech(60)`, 2×2 trip | **MISSING** |
| Switch | 136 | `:275-287` — frameY toggle + 1×1 trip | PORTED (`wiring.rs:513`) |
| Timer | 144 | `:296-311` — frameY toggle + `CheckMech(18000)` | PORTED (`:518`) |
| Pressure plate | 135 | trip arm `:265`; the press is `PressurePlateHelper` | trip PORTED, presser MISSING |
| Weighted pressure plate | 428 | idem (`:265`) | trip PORTED, presser MISSING |
| Projectile pressure pad | 442 | idem (`:265`) | trip PORTED, presser MISSING |
| Logic sensor (the only "detector" tile in 1.4.5.8) | 423 | `:265` trip + the `TELogicSensor` behaviour | PORTED (trip + sensor) |
| Minecart track | 314 | `:265` 1×1 trip | PORTED |
| Golf hole | 476 | `:265` 1×1 trip | PORTED |
| Gem lock | 440 | `:270-274` 3×3 trip | PORTED (`:510`) |
| Fake containers | 441 / 468 | `:312-325` 2×2 trip | PORTED (`:526`) |
| Large gem lock | 467 (`frameX/36 == 4`) | `:326-342` 2×2 trip | PORTED (`:532`) |
| Land mine | 210 | `:288-291` → `ExplodeMine` | PORTED (`:536`) |
| Geyser trap | 443 | `:292-295` → `GeyserTrap` | PORTED (`:539`) |
| Detonator | 411 | `:343-377` (the else-arm) + the player use/landing | **MISSING** |
| Statues | 105 / 349 / 531 | `HitWireSingle:2125/2480/1994` | PORTED (order 1:1, RNG source differs) |
| Gears | — | **no gear tile exists in 1.4.5.8** (`TileID.cs` has no `Gear`) | n/a |

### Tile-entity coverage

| id | C# class | tile | C# payload | Rust |
| --- | --- | --- | --- | --- |
| 0 | `TETrainingDummy` | 378 | `(short npc` | id + validity + a state machine, **not driven** (D15) |
| 1 | `TEItemFrame` | 395 | triple | PORTED (item + validity) |
| 2 | `TELogicSensor` | 423 | `byte check` + `bool On` | PORTED (behaviour + payload) |
| 3 | `TEDisplayDoll` | 470 | 3 bitsBytes + pose + ≤19 triples | id + validity only; payload **read-skipped**, entity **dropped** (D9) |
| 4 | `TEWeaponsRack` | 471 | triple | PORTED (item + validity) |
| 5 | `TEHatRack` | 475 | bitsByte + 2 items + 2 dyes | PORTED (item + validity) |
| 6 | `TEFoodPlatter` | 520 | triple | PORTED (item + validity); the **food gate** not in core |
| 7 | `TETeleportationPylon` | 597 | none | id + validity only (the network is M7-PYLON-1) |
| 8 | `TEDeadCellsDisplayJar` | 698 | triple | id + validity; the `.wld` codec reads it, the native `TeData` does not (D13) |
| 9 | `TEKiteAnchor` | 723 | `(short itemType` | id + validity + payload; the leashed entity MISSING |
| 10 | `TECritterAnchor` | 724 | `(short itemType` | id + validity + payload; the leashed entity MISSING |

## Tests

- `crates/tr-core/src/wiring.rs` — the wiring suites: `signal_propagates_along_the_wire_and_fires_the_lamp`, `colours_are_isolated`, `a_wire_ring_terminates`, `the_multi_trigger_counter_absorbs_converging_paths`, `the_timer_pulses_on_its_period_then_pops`, `check_mech_dedupes_and_caps`, the six gate tests (`the_and_gate_is_on_only_when_all_lamps_are_on`, `the_or_gate…`, `the_xor_gate…`, `the_gate_output_flip_re_trips_its_own_wire_line`, `the_faulty_lamp_rolls_the_output`, `the_overflow_marks_the_gate_broken`), `the_actuator_toggles_the_inactive_bit`, `the_wide_switch_trips_a_3x3_rect`, the wire-section round trip (`the_wire_section_round_trips_every_colour`, `the_wire_section_is_byte_deterministic`, `a_truncated_wire_section_reads_none`), the statue suites (`the_slime_statue_emits_a_blue_slime_request`, `dw_1_1_the_first_table_emits_the_verbatim_ids_at_num141`, `dw_1_1_the_select_random_styles_roll_exactly_once`, `dw_1_1_two_statues_roll_in_the_flood_order`, `dw_1_1_style_63_carries_the_165_veto`, `dw_1_1_styles_64_and_71_block_when_the_body_is_solid`, `the_blocked_flag7_spawn_still_consumes_its_roll`, `dw_1_2_the_second_table_emits_every_npc_style_at_its_verbatim_anchor`, `dw_1_2_the_item_styles_keep_their_cap_ids_and_anchors`, `dw_1_2_style_34_rewrites_the_2x3_mushroom_body`, `dw_1_2_the_teleport_statues_emit_their_rosters`, `dw_1_3_the_mushroom_statue_flips_frame_x_by_216`, `dw_1_3_the_boulder_statue_requests_the_99_projectile`, `the_statue_body_lands_in_the_skip_set_and_fires_once`), the trap suites (`the_dart_trap_fires_the_verbatim_shot`, `the_trap_variants_map_verbatim`, `the_check_mech_cooldown_gates_the_trap`, `dw_2_2_the_ceiling_spiky_ball_fires_twice_from_one_roll_set`, `the_ceiling_spear_trap_fires_straight_down`, `the_floor_spiky_ball_spread_bounds_follow_the_direction`), the dial suites (`dw_2_5_wire_signal_arms_the_sundial`, `dw_2_5_wire_signal_arms_the_moondial`, `dw_2_5_wire_refusal_on_running_cooldown_is_a_no_op`), the cannon/fireworks suites (`dw_3_1_the_cannon_fires_the_verbatim_ammo_request`, `dw_3_2_the_fireworks_box_rolls_415_plus_four`, `dw_3_3_the_explosives_kill_the_tile_and_fire_the_108`, `dw_3_3_the_geyser_fires_by_the_frame_column`, `dw_3_3_the_toilet_flushes_the_733`, `dw_3_4_the_volcano_traps_flip_their_frames`, `dw_3_5_the_device_arms_stay_silent_on_foreign_tiles`), the actuation suite (`the_actuated_block_loses_and_regains_solidity`, `the_not_really_solid_and_exception_kinds_refuse`, `the_plantera_bulb_is_guarded_below_the_surface`, `actuating_under_a_registered_object_pops_it`), and the teleporter/pump suites (`two_pads_register_and_emit_one_event`, `the_same_pad_hit_twice_registers_once`, `close_pads_are_rejected`, `the_half_brick_lowers_the_anchor`, `the_pump_moves_water_and_adopts_the_type`, `the_pump_registration_caps_at_19`, `the_pump_respects_the_255_cap_and_the_type_lock`).
- `crates/tr-core/src/tile_entities.rs` — `registration_order_matches_the_manager`, `item_frame_validity_checks_the_frames`, `the_critter_anchor_requires_the_lady_bug_tile`, `the_display_doll_validity_checks_the_frames`, `place_and_remove_round_trip_the_books`, `weapon_rack_and_food_platter_frames`, `the_item_triple_round_trips_the_c_sharp_layout`, `the_hat_rack_presence_bits_round_trip`, `the_books_round_trip_the_save_layout`, `write_all_is_deterministic_and_pins_the_layout`, `the_load_drops_the_records_on_invalid_cells`, `a_record_with_an_unknown_kind_byte_is_refused`, `a_duplicated_cell_keeps_the_port_insert_semantics`, `the_sensor_kinds_and_tiles_are_registered`, `the_day_sensor_trips_once_on_the_rising_edge`, `the_liquid_sensor_keeps_the_grace`, `the_lava_sensor_ignores_water`, `the_player_above_sensor_scans_the_box`, `the_dummy_spawns_near_players_and_deactivates_on_the_lost_tie`, `the_live_entities_walk_the_id_book_and_skip_tombstones`, `load_entity_replaces_a_same_position_record`, `the_id_book_is_dense_across_a_load_sequence`, `the_world_loaded_hook_leaves_the_books_alone`.
- `crates/tr-core/src/wld/sections/tile_entities.rs` — `test_DW_5_5_the_section_is_implemented`, `test_DW_5_3_the_records_round_trip_byte_equal`, `test_DW_5_3_a_record_head_is_hand_computed`, `test_DW_5_3_the_stored_ids_are_discarded_and_reassigned`, `test_DW_5_3_next_id_is_the_record_count_after_load`, `test_DW_5_3_the_hat_rack_and_item_frame_payloads_survive`, `test_DW_5_3_an_invalid_anchor_tile_drops_the_entity`, `test_DW_5_3_an_entity_outside_the_world_is_dropped`, `test_DW_5_3_a_same_position_record_replaces_the_earlier_one`, `test_DW_5_3_the_written_section_reads_back_itself`, `test_DW_5_4_a_display_doll_record_is_skipped_and_the_cursor_lands_on_the_next_record`, `test_DW_5_4_the_doll_skip_is_version_gated`, `test_DW_5_4_a_dummy_and_a_sensor_record_ride_the_same_stream`, `test_DW_5_4_the_skipped_doll_leaves_the_books_on_load`, `test_DW_5_4_the_empty_doll_form_stays_parseable`, `test_DW_5_4_an_unknown_type_byte_refuses_the_section`, `a_negative_count_is_refused`, `a_truncated_payload_is_refused_at_every_kind`, `a_hostile_count_does_not_allocate`, `the_anchor_item_type_round_trips_as_a_short`, `test_DW_5_7_all_four_sections_participate_in_one_framed_file`, `test_DW_5_5_the_citations_are_present`.
- `crates/tr-app/src/world_sim/wiring.rs` — `dw_4_1_the_new_data_rows_and_the_ai_spawn`, `dw_4_3_the_slime_statue_spawns_at_the_verbatim_bottom_center`, `dw_4_3_the_unrostered_statue_kinds_drop`, `dw_4_3_the_item_caps_filter_by_type`, `dw_4_4_the_teleport_statue_moves_only_the_rolled_npc`, `dw_4_4_an_empty_roster_rolls_and_moves_nothing`, `dw_4_2_the_ceiling_spiky_ball_cap_counts_kind_185`, `dw_4_2_the_floor_spiky_ball_cap_never_blocks`, `dw_4_2_the_bunny_gate_blocks_only_the_spawn`, `dw_4_2_the_fountain_dedupe_blocks_the_anchor`.

No cargo run was performed (research only, per the module rules).

## Deviations & risks

| # | Deviation | Severity | Admitted? |
| --- | --- | --- | --- |
| D1 | **424/445 handling in the flood is on the wrong cell.** The C# computes `b` from the **neighbour** tile (`Wiring.cs:890-904`, `:971-974`) so a wire-pipe/pixel-box neighbour gets no `_toProcess` count; the Rust tests the **current** cell (`wiring.rs:1917`). The 424 direction filter (`Wiring.cs:905-928`) and the 445 trigger-byte bookkeeping (`:929-943`) have no Rust counterpart — `pixel_box_triggers` is never written (only read at `wiring.rs:576`), so a pixel box can never fire. | behavioural | No — ANALYSIS claims "`b = 0` through junction boxes (424/445)" as ported |
| D2 | **The device cell is unconditionally skipped after firing.** `hit_wire_single` inserts `(i,j)` into the skip set (`wiring.rs:1946`); the C# only skips cells that explicitly call `SkipWire`, and its `_toProcess` counter is *removed at zero* so a 5+-path convergence re-fires the device (`Wiring.cs:956-968`). The Rust therefore never re-fires a device within one flood. | behavioural | No — ANALYSIS explicitly claims the re-fire |
| D3 | `PixelBoxPass` runs **before** the teleport pair loop in Rust (`wiring.rs:560-567`); the C# runs it after (`Wiring.cs:654-665`). | behavioural (currently latent, D1 kills the pass) | No |
| D4 | `Wiring.running` (the world-gen guard, `Wiring.cs:531`/`:654`, `WorldGen.cs:31895`+) is not ported. | behavioural (low) | No |
| D5 | `HitSwitch`'s `132` (lever) / `411` (detonator) else-arm (`Wiring.cs:343-377`) is missing from `hit_switch` (`wiring.rs:496-543`): no ±36 frame shift, no `CheckMech(60)`, no 2×2 trip. | behavioural | No — the ROADMAP row cites only `:259-330` |
| D6 | **No player→switch bridge.** `Player.cs:32177`/`:32339`/`:34823`, `Collision.cs:3303-3346`, `Minecart.cs:1286`, `PressurePlateHelper.cs:161` — no Rust caller of `hit_switch` outside the sensor trip (`tile_entities.rs:780`). The player cannot press a lever/switch/timer in the port. | behavioural | No |
| D7 | **`PressurePlateHelper` is unported** (`Update`, `UpdatePlayerPosition`, `UpdatePlatePosition`, `MoveInto/MoveAwayFrom`, `PokeLocation`): the weighted plates (135/428/442) never press. Only the `.wld` point list and the worldgen placement exist. | behavioural | No |
| D8 | **`HitWireSingle` is ~22 arms of the C#'s ~70.** Missing: the `255..268` type-pair swap (`:1034-1050`), 406/452/411/425 frame shifts, the whole light family 42/93/95/100/126/173/564/149/244/565 (`ToggleHolidayLight`/`ToggleLamp`/`Toggle2x2Light`/`ToggleChandelier`/`ToggleLampPost`/`ToggleHangingLantern`/`ToggleCampFire`/`ToggleFirePlace`), plus 21/467/219/642/35/139/207/410/480/509/657/658/720/721/725/733/455/34/314/33/49/174/372/646/92. The ROADMAP M7-WIRE-2 row claims "every device arm of `HitWireSingle`/`HitSwitch` verbatim" — that is **false**. | behavioural | No |
| D9 | **The display doll is dropped, not carried.** `is_storable` (`wld/sections/tile_entities.rs:350`) refuses the doll, so a loaded doll is deleted from the books and never written back; the C# keeps it fully (`TEDisplayDoll.WriteExtraData`). A `.wld` round trip loses the doll. | behavioural (save fidelity) | Partly — M6-SAVE-1 records "the doll consumed-not-kept" |
| D10 | `TileEntity.OnWorldLoaded` is a no-op (`tile_entities.rs:450`); the C# calls it on every entity (`WorldFile.cs:3355-3358`) and `TELeashedEntityAnchorWithItem.OnWorldLoaded` respawns the leashed entity. | behavioural | Partly — M7-TE-2 lists the anchors as a follow-up |
| D11 | The native `.trwld` TE payload layout diverges from the C#'s `WriteExtraData` (presence byte before the item triple, `i32` dummy npc, `i32` sensor grace — `tile_entities.rs:219`). The `.wld` codec is the faithful one; the native format is the port's own. | by design | Yes (the native format is port-private) |
| D12 | `MaxEntitiesPerChunk = 1000` (`TileEntity.cs:14`) is not enforced. | cosmetic | Yes (ANALYSIS) |
| D13 | The Dead Cells jar's item is read by the `.wld` codec but maps to `TeData::None` in the native `TeData::read` (`tile_entities.rs:259-300` omits `TeKind::DeadCellsDisplayJar`). | behavioural (minor) | No |
| D14 | **The statue RNG source is not the C#'s.** The C# rolls `Utils.SelectRandom(Main.rand, …)` (`Wiring.cs:2167`, `:2229`, `:2251`, `:2354`) off the *engine* stream; the Rust uses its own xorshift stream (`faulty_next`, `wiring.rs:372`) seeded once per world (`app/mod.rs:2931`). The **order** is 1:1 (the roll precedes every gate; the second table runs only when the first did not spawn), but the *values* differ from vanilla and the faulty-lamp roll shares the same stream. | determinism | Partly — ATLAS calls it "the app's single core-stream roll" |
| D15 | `update_training_dummies` (`tile_entities.rs:831`) has **no production caller** — only the unit tests; the app's 60 Hz TE step drives the sensors alone (`world_sim/tile_entities.rs:31`). The dummy never spawns its NPC. | behavioural | No — M7-TE-2 claims the dummy engine half as done |
| D16 | The out-of-world branch of the mech queue: the C# does `Wiring._numMechs--` **without** shifting (`Wiring.cs:165-170`), so the last entry is dropped; the Rust's `Vec::remove` shifts the tail, so the last entry survives. | behavioural (obscure) | No |
| D17 | The player cannot place wire/actuator/lever tiles: `World.wires.set` has no production caller outside `worldgen/temple.rs:1709` and the net load `netglue.rs:720`; `MassWireOperation` (`Wiring.cs:419`) is unported. | behavioural | No — the wiring system is effectively inert in normal play |
| D18 | The `HitWireSingle` case-419 arm in Rust does not call `SkipWire` (the C# does, `Wiring.cs:1064`) — it relies on the blanket skip of D2, which hides the difference but changes the re-fire semantics. | behavioural | No |
| D19 | Sounds (`SoundEngine.PlaySound(28, …)`) are not wired to the audio engine. | cosmetic | Yes (ANALYSIS) |
| D20 | Net sync (`NetMessage.SendTileSquare`/`SendData(59/106/110)`), `Main.netMode == 1` early-outs and `Wiring.SetCurrentUser` are not ported. | out of scope (MP) | Yes |

### ROADMAP DONE re-verification

| Row | Claim | Verdict | Evidence |
| --- | --- | --- | --- |
| M7-WIRE-1 (engine) | the four colours, the `HitWire` BFS with `SkipWire`, the `_toProcess` counter, the `[2, max-2]` bounds, `HitWireSingle`'s subset, `HitSwitch`'s table, `CheckMech`/`UpdateMech` | **PARTIAL** | The counter/bounds/order are right, but D1 (wrong cell for 424/445, missing 424 filter, missing 445 trigger bytes), D2 (no re-fire), D5 (missing 132/411) contradict the row |
| M7-WIRE-2 (traps & statues) | "every device arm of `HitWireSingle`/`HitSwitch` verbatim" | **PARTIAL** | D8 (the light family, the `255..268` swap and ~40 small arms are absent), D14 (the RNG source) |
| M7-WIRE-3 (actuators) | the guard chain + the frame cascade + the wire overlay | **PASS** (core) | `de_active`/`re_active` (`wiring.rs:2151`/`:2191`) match `Wiring.cs:3195-3232`; the overlay render was not re-verified in this pass |
| M7-WIRE-4 (logic gates) | the six types, the lamp stack, the chaining, the faulty lamp | **PASS** (mechanics) / determinism caveat | `check_logic_gate` (`wiring.rs:1760`) matches `Wiring.cs:720-831`; the faulty-lamp roll uses the port's stream (D14) |
| M7-WIRE-5 (teleporters & pumps) | the pads, `Teleport()`, the pump cap 19, `XferWater` | **PASS** | `wiring.rs:588`/`:1661`/`:2018` vs `Wiring.cs:3120-3189`/`:475-523`/`:2043-2125`; the cap-19 and the 255 liquid cap match |
| M7-TE-1 (framework) | the ids, the books, the item halves | **PASS** (with D12/D13) | `TeKind` (`tile_entities.rs:81`) vs `TileEntitiesManager.cs:26-37`; the payloads match the `WriteExtraData` shapes |
| M7-TE-2 (behaviour halves) | the sensors + the dummy engine half | **PARTIAL** | The sensors are driven (D-free); the dummy engine is **not** (D15), and the doll/pylon/anchors/jar remain follow-ups as the row itself admits |
| M6-SAVE-1 (the `.wld` TE section) | the section is ported both ways | **PARTIAL** | The codec is faithful for 10 of 11 kinds; the doll is dropped (D9) |

## Task seeds

1. **Fix the flood's 424/445 handling** (S) — port `Wiring.cs:890-943`: compute `b` from the *neighbour* tile, add the 424 `frameX/18` direction filter, and write `_PixelBoxTriggers` so `PixelBoxPass` can fire. Unblocks D1/D3. Deps: none.
2. **Restore the multi-trigger re-fire** (S) — remove the unconditional `skip.insert` in `hit_wire_single` (`wiring.rs:1946`) and let the C#'s "counter removed at zero" path re-fire; keep the per-case `SkipWire` calls. Deps: task 1 (both touch `hit_wire`).
3. **Add the `132`/`411` `HitSwitch` arm** (S) — the ±36 frame shift, `CheckMech(60)`, the 2×2 trip (`Wiring.cs:343-377`). Deps: none.
4. **Port the player→switch bridge** (M) — `Player.cs:32177`/`:32339`/`:34823` and `Collision.cs:3303-3346` → `wiring.hit_switch`; the lever/switch/timer/detonator become usable. Deps: task 3 (the lever arm) for the 132 case.
5. **Port `PressurePlateHelper`** (M) — the per-player weight tracking, `MoveInto`/`MoveAwayFrom`, `PokeLocation` → `hit_switch`; the weighted plates (135/428/442) then press. Deps: task 4.
6. **Wire placement / `MassWireOperation`** (L) — the wire tool's rect placement, the item consumption (530/849), and the `World.wires.set` write path; without it the whole system is inert (D17). Deps: task 4.
7. **Port the missing `HitWireSingle` device arms** (L) — the `255..268` swap, the light family (42/93/95/100/126/173/564/149/244/565/406/452/411/425) and the `Toggle*` helpers; split into two tasks if needed (frame shifts vs. toggles). Deps: task 1 (the skip semantics).
8. **Drive the training dummy from the sim** (S) — call `update_training_dummies` in `world_sim/tile_entities.rs` and give the dummy its NPC 488 tie; needs the M4-NPC roster for the spawn itself. Deps: the NPC roster task.
9. **Carry the display doll through the `.wld`** (L) — the 19-triple payload in `TeData`, `is_tile_valid`, `FixLoadedData` (`TEDisplayDoll.cs:952`) and the write-back; remove the doll from `is_storable`. Deps: the doll's behaviour half (M7-TE-2 follow-up).
10. **The pylon network + the leashed anchors** (L) — `TETeleportationPylon`'s network and `TELeashedEntityAnchorWithItem.OnWorldLoaded` (the respawn), plus `TileEntities::on_world_loaded` becoming a real hook (D10). Deps: the doll task shares the TE payload work.
11. **Adopt the engine RNG stream for the statue rolls** (M) — replace `faulty_next` with the app's `Main.rand` equivalent so the statue picks match vanilla; the order is already right (D14). Deps: the RNG-stream task in the determinism work.
12. **The `Wiring.running` guard** (S) — set the flag around `trip_wire` and gate the world-gen passes that check it (`WorldGen.cs:31895`+). Deps: none.
