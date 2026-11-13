# 42 вЂ” Save/load (IO) + Stage-2 compression hooks

> Wave 4 В· researched 2026-11-12 В· sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

C#: `decompiled/Terraria/IO/WorldFile.cs` (3845 lines вЂ” the whole `.wld` format),
`decompiled/Terraria/IO/PlayerFileData.cs` (191 вЂ” the `.plr` file object),
`decompiled/Terraria/Player.cs:54673-54800` (`SavePlayer` в†’ the encrypted `.plr`),
`decompiled/Terraria/Main.cs:19737-19765` (`DoUpdate_AutoSave` вЂ” the autosave cadence),
`decompiled/Terraria/Tile.cs:155-197` (`isTheSameAs` вЂ” the RLE batching equality),
`decompiled/Terraria/IO/FileMetadata.cs` (the 20-byte metadata + `IncrementAndWrite`).

Rust: `crates/tr-core/src/wld/mod.rs` (1982 вЂ” the `.wld` reader/writer),
`crates/tr-core/src/wld/sections/` (12 files: `mod.rs` 263, `header.rs` 70,
`flags.rs` 602, `tiles.rs` 243, `chests.rs` 445, `signs.rs` 375, `npcs.rs` 724,
`tile_entities.rs` 1219, `pressure_plates.rs` 203, `town_rooms.rs` 215,
`bestiary.rs` 496, `creative_powers.rs` 571),
`crates/tr-core/src/save.rs` (1740 вЂ” the port's **own** `.trwld` format),
`crates/tr-app/src/app/saveglue.rs` (1107 вЂ” the App's save dispatch + the map write),
`crates/tr-app/src/player_file.rs` (541 вЂ” the `.plr` reader/writer).

Responsibility: the byte-level persistence of the world (11 sections) and of the
character (one encrypted file), plus the ticks that trigger them. There are **two
Rust formats** for the same world вЂ” the vanilla-shaped `.wld` (`wld/`) and the
port's native `.trwld` (`save.rs`) вЂ” and the App picks one per file path.

## Key entities

- `Terraria/IO/WorldFile.cs::SaveWorld_Version2` (1150-1168) вЂ” writes the 11
  sections in order and returns their end-offsets.
- `Terraria/IO/WorldFile.cs::SaveFileFormatHeader` (1170-1206) вЂ” `i32 326`, the
  20-byte `FileMetadata`, `i16 11`, 11 Г— `i32` pointer placeholders, then
  `u16 TileID.Count` + the frame-importance bitmap (LSB-first).
- `Terraria/IO/WorldFile.cs::SaveHeaderPointers` (1208-1219) вЂ” seeks to 0, rewrites
  `326`, skips the 20 metadata bytes, writes `i16 11` and the 11 pointers.
- `Terraria/IO/WorldFile.cs::SaveWorldTiles` (1432-1604) вЂ” the per-tile record
  writer: header chain `b/b2/b3/b4` + payload + the run count (the only
  compression in the world format).
- `Terraria/IO/WorldFile.cs::SaveChests` (1606-1676) В· `SaveSigns` (1678) В·
  `SaveNPCs` (1703-1750) В· `SaveTileEntities` (3302-3313) В·
  `SaveWeightedPressurePlates` (3359-3371) В· `SaveTownManager` (3385) В·
  `SaveBestiary` (3396) В· `SaveCreativePowers` (3412) В· `SaveFooter` (1752-1758).
- `Terraria/IO/WorldFile.cs::LoadWorld_Version2` (1760-1850) вЂ” the pointer walk
  with the per-section version gates; `LoadWorldTiles` (2519-2685) вЂ” the tile reader.
- `Terraria/IO/WorldFile.cs::InternalSaveWorld` (908-964) вЂ” builds the whole file
  in a `MemoryStream(7000000)`, writes it, then re-reads it for
  `ValidateWorld` and restores the `.bak` on failure.
- `Terraria/IO/WorldFile.cs::DoRollingBackups` (966-1004) вЂ” the numbered `.bak` rotation.
- `Terraria/Player.cs::SavePlayer` (54673-54694) в†’ `InternalSavePlayerFile`
  (54717-54748) вЂ” the `.plr`: `RijndaelManaged` AES with `ENCRYPTION_KEY` as **both**
  key and IV, then `i32 326` + `FileMetadata` + `Player.Serialize` (54750+).
- `Terraria/Main.cs::DoUpdate_AutoSave` (19737-19765) вЂ” the cadence.
- `Terraria/Tile.cs::isTheSameAs` (155-197) вЂ” the RLE batching equality.
- `crates/tr-core/src/wld/mod.rs::to_bytes` (347-389) вЂ” the whole-file assembler
  (prologue + 11 bodies + footer + pointer backfill); `::save` (308-323) вЂ” the
  tmp-file + rename write; `::FORMAT_VERSION` (23) = 326; `::SECTION_COUNT` (25) = 11.
- `crates/tr-core/src/wld/sections/mod.rs::DEFS` (111-168, load) and
  `::SAVE_DEFS` (194-275, write) вЂ” the two registries in `SaveWorld_Version2` order.
- `crates/tr-core/src/wld/sections/tiles.rs::write` (30) / `::write_tile_record` (45)
  / `::encoded_same` (219) / `::allows_batching` (211) вЂ” the tile codec.
- `crates/tr-core/src/save.rs::save` (187) / `::write_world` (210) / `::MAGIC` (19) вЂ”
  the `.trwld` writer (`"TRWLD01\0"`, now v13).
- `crates/tr-app/src/app/saveglue.rs::App::save_world` (27-166) вЂ” the one save
  dispatch; `::App::wld_sections` (175); `::App::save_map` (197) / `::load_map` (210).
- `crates/tr-app/src/player_file.rs::cbc_decrypt` (94) / `::cbc_encrypt` (113) /
  `::read_player_file` (135) / `::write_player_bytes` (339) вЂ” the `.plr` codec.

## Dependencies

- Calls into (C#): `Main.DoUpdate_AutoSave` в†’ `WorldGen.saveToonWhilePlaying` /
  `WorldGen.saveAndPlay` в†’ `WorldFile.SaveWorld`; `PlayerFileData.CreateAndSave` в†’
  `Player.SavePlayer`; `Tile.isTheSameAs`; `TileID.Sets.AllowsSaveCompressionBatching`;
  `FileMetadata.IncrementAndWrite`; `FileUtilities.Write` (в†’ `SocialAPI.Cloud.Write`
  when `useCloudSaving`); `BestiaryUnlocksTracker`; `CreativePowerManager`;
  `TownRoomManager`; `TileEntity.Write`; `Chest`/`Item`.
- Called by (C#): `UIWorldListItem.PlayGame` в†’ `Main.LoadWorld` в†’
  `WorldFile.LoadWorld_Version2`; `Main.autoSave` (the config toggle, `Main.cs:1137`,
  persisted as `"AutoSave"`, `Main.cs:4097`/`:4333`).
- Shared data structures: `Main.tile`, `Main.chest`, `Main.sign`, `Main.npc`,
  `TileEntity.ByID`, `TownManager` rooms, the bestiary trackers, the creative-power
  table вЂ” all read straight out of the globals by the writers.
- Rust: `wld::save`/`to_bytes` в†ђ `saveglue::App::save_world` в†ђ the `frame.rs`
  autosave tick / F9 / the window-close request / the menu's Quit;
  `wld::parse_with_sections` в†ђ `App::new` / `App::swap_world`;
  `save::save` (the `.trwld`) в†ђ the same dispatch; `player_file` в†ђ `persist`/`save_list`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `.wld` prologue + pointer table (v326) | PORTED | `wld/mod.rs::to_bytes` (347), `sections/header.rs::write_bitmap` | Same shape: `i32 326`, 20-byte metadata, `i16 11`, 11 Г— `i32` pointers backfilled at offset 24. The revision is the previous file's +1 (`FileMetadata.IncrementAndWrite`). `test_DW_1_4_the_prologue_and_pointers_are_hand_computed` pins the bytes. |
| `.wld` world header + flags | PARTIAL | `sections/header.rs::write_world_head`, `sections/flags.rs::write` | Layout is 1:1. Generator version/`UniqueId`/`WorldId`/`LastPlayed` are now persisted and roundtripped on `World` (S1-J-2, 2026-11-13). Remaining deviations: unowned secret-seed/flavour fields keep defaults and seed text remains the decimal seed (the C#'s can be any text). |
| `.wld` tile section | PARTIAL | `sections/tiles.rs::write` | Record layout, header chain, payload order and the 1-/2-byte run count are exact. **The four wire bits are not written** (`b2` 0x02/0x04/0x08 = wire/wire2/wire3, `b3` 0x20 = wire4) в†’ not byte-identical, and the wire state is lost on load. |
| `.wld` chests / signs / NPCs / TEs / plates / town rooms / bestiary / creative powers | PORTED (shape) | `sections/chests.rs`, `signs.rs`, `npcs.rs`, `tile_entities.rs`, `pressure_plates.rs`, `town_rooms.rs`, `bestiary.rs`, `creative_powers.rs` | Every codec mirrors the C# field-by-field, keeps the C# version gates and refuses hostile counts. Two documented gaps: the `version < 251` `unlocked*Spawn` recompute (npcs.rs) and `TEDisplayDoll` state (tile_entities.rs). |
| `.wld` footer | PARTIAL | `wld/mod.rs` (367-370) | `bool true`, `string name`, `i32 0` вЂ” the C# writes the real `WorldId` (`SaveFooter:1752-1758`); the port keeps no world id. |
| `.wld` load path | PORTED | `wld/mod.rs::parse_table` (203), `load_sections` (395) | The pointer walk + the per-section end-offset check + the version gates (`sections/mod.rs::DEFS`). |
| `.wld` write path (atomicity) | PARTIAL | `wld::save` (308-323) | tmp-file + rename (atomic). **No `.bak`, no rolling backups, no post-write `ValidateWorld`** (C# `DoRollingBackups` 966-1004, `InternalSaveWorld` 941-962). |
| `.trwld` native format | PORTED | `save.rs::save` (187) / `write_world` (210) | v13 (`"TRWLD01\0"`), RLE per row, the full 14-byte tile struct, the v6-v10 tails (flags subset, wire side table, TEs, chests, banners, bestiary, town NPCs, ore tiers). The town-rooms / bestiary-sights+chats / creative-powers twins are still open (app-side state). |
| `.plr` reader + writer | PARTIAL | `player_file.rs` (94-404) | Real AES-128-CBC reader/writer with the vanilla key, `FileMetadata`, the section walk and the stat caps. The writer mirrors only a **subset** of `Player.Serialize` (sanitized name + starter items per the ROADMAP row) в†’ byte-parity with the C# is unproven. |
| `.map` (explored map) | PORTED (separate) | `saveglue.rs::App::save_map` (197) / `load_map` (210) | Outside the world/player formats; noted here because the C# writes it inside `Player.InternalSavePlayer`. |
| Autosave cadence | PORTED | `saveglue.rs::save_world` (27-162), the `frame.rs` tick | The port's 600 s branch (`frame.rs`, `!gameMenu && autoSave`) is the single-player reduction of `Main.cs:19751-19765` (`(autoSave || netMode == 2)`, which nets to `autoSave` at `netMode == 0`), and `save_world` writes the world + player sidecar + map. The 300 s `saveToonWhilePlaying` branch (`Main.cs:19743-19749`) is gated `netMode == 1` (`:19739`) — **multiplayer client only**, unreachable in the single-player port (S1-J-6 refuted 2026-11-13); the menu arm (`:19767-19769`) only stops the timer. |
| Cloud save path | MISSING | вЂ” | The C# writes `Main.CloudWorldPath` / `Main.CloudPlayerPath` through `FileUtilities.Write(..., useCloudSaving)` в†’ `SocialAPI.Cloud.Write` (`WorldFile.cs:940`, `Main.cs:4820-4870`). The port writes a local path only; cloud is Steam/Social = out of scope (module 38). |
| Save-time "temp" snapshot | VERIFIED+PORTED (S1-J-7) | вЂ” | The C# snapshots day/time/weather into `_temp*` and restores it on load (`SetTempToOngoing` 1040-1066 / `SetOngoingToTemps` 152-179). The Rust flags writer emits the world's own values; the exact temp mechanism was not checked. |

### Section table (byte formats, C# vs Rust)

| Section (pointer) | C# bytes format | Rust bytes format | byte-identical? | Notes |
| --- | --- | --- | --- | --- |
| 0 prologue | `i32 326` + `FileMetadata` (20) + `i16 11` + 11Г—`i32` pointers + `u16 TileID.Count` + importance bitmap (LSB-first) | same (`wld/mod.rs:350-355`, `header.rs::write_bitmap`) | **YES** (layout+values) | Only the revision differs by design (+1 per save). The bitmap uses the world's own table в†’ a shorter table yields a shorter bitmap. |
| 1 world header | `string name`, `string seed`, `u64 generatorVersion`, `16 B GUID`, `i32 worldId`, 4Г—`i32` bounds, `i32 maxTilesY`, `i32 maxTilesX` | same order/shape | **NO** (values) | generator version 0, GUID zeros, world id 0, seed = decimal text. |
| 1 flags block | `SaveWorldFlags:1238-1430` вЂ” a long fixed sequence of `bool/i32/i64/f32/f64` | `flags.rs::write` вЂ” same field order, unowned fields = C# defaults | **NO** (values) | `LastPlayed` = min date (the C# writes `UtcNow`), the secret-seed booleans = `false`. |
| 2 tiles | per-tile record: `b` [+`b2` +`b3` +`b4` chain] + payload (`u8 type` [`u8 type>>8`] [`4 B frames`] [`u8 paint`] [`u8 wall`] [`u8 wallColor`] [`u8 liquid`] [`u8 wall>>8`]) + run count (`u8`, or `u8` low + `u8` high) | `sections/tiles.rs::write_tile_record` вЂ” identical record, identical byte order | **NO** | The C# writes the 4 wire bits; the port drops them. The batching equality also differs (`isTheSameAs` compares the wire bits в†’ different run boundaries). |
| 3 chests | `i16 count`; per chest `i32 x`, `i32 y`, `string name`, `i32 maxItems` (v294+; older files carry one shared `i16` before the loop); per slot `i16 stack` (+ `i32 type`, `u8 prefix` when stack > 0) | `sections/chests.rs` вЂ” same | **UNKNOWN** | Field-for-field mirror; no golden-byte fixture exists. |
| 4 signs | `i16 count`; per sign `string text`, `i32 x`, `i32 y` (text **before** position) | `sections/signs.rs` вЂ” same | **UNKNOWN** | The text-first order is preserved. |
| 5 NPCs | 3 lists: (a) shimmered town bitmap = `i32 count` + `i32` indices, (b) terminator-delimited town NPCs (`bool active`, `i32 netID`, `string givenName`, `Vector2`, `bool homeless`, `i32 homeX/homeY`, `BitsByte`, вЂ¦), (c) terminator-delimited `SavesAndLoads` NPCs | `sections/npcs.rs` вЂ” same | **UNKNOWN** | The `version < 251` `unlocked*Spawn` recompute is deliberately not ported (documented). |
| 6 tile entities | `i32 count`; per entity `byte type`, `i32 id`, `i16 x`, `i16 y` + the kind's extra data | `sections/tile_entities.rs` вЂ” same | **UNKNOWN** | `TEDisplayDoll` (id 3) is consumed but not stored в†’ dropped on load. |
| 7 pressure plates | `i32 count`; per plate `i32 x`, `i32 y` | `sections/pressure_plates.rs` вЂ” same | **UNKNOWN** | The `bool[255]` arrays are rebuilt on load, as in the C#. |
| 8 town rooms | `i32 count`; per room `i32 npcID`, `i32 x`, `i32 y` | `sections/town_rooms.rs` вЂ” same | **UNKNOWN** | The room state itself lives on the app side (`tr-app/src/town.rs`). |
| 9 bestiary | 3 sub-lists (kills / sights / chats), each `i32 count` + per entry `string creditId` (+ `i32 count` for kills) | `sections/bestiary.rs` вЂ” same | **UNKNOWN** | Keyed by the bestiary **credit-id string**, never the net id. |
| 10 creative powers | terminator-delimited `{ bool true, ushort id, payload }` until `bool false` | `sections/creative_powers.rs` вЂ” same | **UNKNOWN** | An unknown id **breaks** the loop in both (the C# `break`, not a skip). |
| footer | `bool true`, `string worldName`, `i32 WorldId` | `wld/mod.rs` (367-370) вЂ” `bool true`, `string name`, `i32 0` | **NO** | The port keeps no world id. |

"UNKNOWN" = the layout is mirrored but **no byte-level golden fixture** was found
against a C#-written file; the only byte fixtures are hand-computed inside the Rust
tests (`wld/mod.rs:1200-1281`).

## Compression hooks (Stage-2)

**What is already compressed in the C# format:** exactly one thing вЂ” the tile
section's vertical run-length batching (`SaveWorldTiles:1574-1598`): a run count of
1 byte (`b |= 0x40`) or 2 bytes (`b |= 0x80`), gated by
`TileID.Sets.AllowsSaveCompressionBatching`. It is *not* a "TILE chunk": there is
no chunk container anywhere in the format вЂ” the 11 sections are contiguous and
addressed by one 11-entry pointer table. A grep of `decompiled/Terraria/IO/*.cs`,
`Main.cs` and `Utils.cs` for `Deflate|GZip|Compression|QuickCompress|Compress`
returns **only** `ResourcePack.cs` (an unrelated zip) вЂ” the world and player saves
carry no deflate/zlib at all. Everything else (flags, header, the 8 entity
sections, the footer) is **raw** little-endian structs.

**Hook points that cannot break the uncompressed path:**

1. `wld::to_bytes` (`crates/tr-core/src/wld/mod.rs:347-389`) вЂ” the eleven section
   bodies are already isolated `Vec<u8>` blobs and the writer already computes each
   section's end offset, so a per-section "compress or not" wrapper is a local
   change that leaves the 326 path byte-identical. **Constraint:** the vanilla
   loader rejects `version > 326` as `LaterVersion` (`WorldFile.cs:196-200`), so a
   compressed `.wld` must bump the version field (327) or use a new extension вЂ”
   the vanilla then refuses it cleanly instead of mis-parsing.
2. Inside the tile codec вЂ” a second lossless stage over the tile section bytes
   only (the highest-value target: the tile section is the bulk of a `.wld`).
   The RLE/VLQ predictor the owner chose fits here: the section is already a
   run-length stream, so a column-wise delta/VLQ pass on the *encoded* tile records
   is the natural stage-1, with deflate as stage-2.
3. `save.rs::write_world` (`crates/tr-core/src/save.rs:210`) вЂ” the port's **own**
   `.trwld` format. Compressing here breaks no vanilla file but breaks the port's
   own readers unless both halves move together: the cheapest place to prove the
   compressor, the least valuable for interop.
4. `player_file.rs::write_player_bytes` (339) / `cbc_encrypt` (113) вЂ” the `.plr`
   is **encrypted** (AES, `Player.cs:54734-54736`), so a compressor must run on the
   plaintext *inside* the crypto (compress в†’ encrypt) or on the ciphertext
   (useless). The reader decompresses after `cbc_decrypt` (94). The vanilla client
   cannot read the result в†’ port-only files.

**Lossless-only is confirmed correct by the data.** Every payload is discrete
integers вЂ” tile ids, frame coordinates, paint ids, coords, item ids, bestiary
credit-id strings, the pointer table. There is no raster, no float-heavy block
(the only floats are `time`, `rain`, `wind` in the flags block, a few bytes each).
A JPEG/WebP-style lossy codec has no valid target: it would corrupt the tile grid,
item ids and the pointer table, and the save is the game's persistent state
(determinism is a hard invariant). Lossless (RLE/VLQ predictor + deflate) is the
only defensible Stage-2.

**Background save/load (Stage-2 idea) вЂ” feasible, with one caveat.** The IO
structure allows it: `to_bytes` already assembles the entire file in memory and
`wld::save` writes it in one `std::fs::write` to a tmp file + rename
(`wld/mod.rs:319-321`), so the *write* can move to a worker thread with no
structural change (the tmp+rename already makes it atomic). The **serialization
must stay on the main thread** вЂ” it reads the live world, which mutates every
tick. The vanilla does the same thing synchronously (`WorldFile.IOLock`,
`InternalSaveWorld` 926-940: one `MemoryStream`, one write), so a background write
is an invisible divergence. On the load side the port already loads at a frame
boundary (`App::swap_world`), so a background *parse* is possible but adds little.

## Tests

- `crates/tr-core/src/wld/mod.rs` вЂ” 39 `#[test]`s, incl.
  `test_DW_1_4_the_prologue_and_pointers_are_hand_computed` (the byte fixture),
  `test_DW_1_1_load_keeps_its_signature_and_reads_a_file`,
  `test_DW_1_3_rejects_a_misaligned_header_pointer` /
  `..._rejects_a_tile_pointer_past_eof` / `..._rejects_a_backwards_section_pointer`
  / `..._rejects_a_section_pointer_past_eof` / `..._walks_a_legacy_six_pointer_table`,
  `test_DW_1_4_the_revision_reads_the_previous_file`,
  `test_DW_1_5_every_section_stub_round_trips_its_empty_encoding`,
  `save_writes_every_section_that_has_a_writer`,
  `test_DW_7_5_a_section_with_data_and_no_writer_is_refused`,
  `parses_the_tile_section_column_major`,
  `the_importance_bitmap_round_trips_lsb_first` / `the_full_byte_bitmap_corner_parses`,
  `test_DW_2_3_an_air_column_encodes_as_one_short_rle_record` /
  `..._a_long_air_column_encodes_the_two_byte_rle` /
  `..._a_non_batchable_kind_writes_no_rle_run` /
  `..._the_rle_run_detection_matches_is_the_same_as`.
- `crates/tr-core/src/wld/sections/tiles.rs` вЂ” 0 tests (the tile codec is covered
  from `wld/mod.rs`'s tests); each other `sections/*.rs` file carries its own unit
  tests (not counted here).
- `crates/tr-core/src/save.rs` вЂ” 35 tests (the `.trwld` v2..v13 tail walk:
  `dw_2_1_a_v6_payload_loads_the_seeds_but_not_the_v7_tail`,
  `m5_seed_1_the_secret_seed_flags_round_trip_in_v12`, вЂ¦).
- `crates/tr-app/src/app/saveglue.rs` вЂ” 14 tests, incl.
  `test_DW_7_1_a_full_world_round_trips_every_section`,
  `test_DW_7_3_a_wld_save_writes_every_section_and_increments_the_revision`,
  `test_DW_7_2_the_trwld_path_is_unchanged`,
  `an_empty_world_saves_every_section_at_its_empty_encoding`,
  `a_legacy_pointer_table_keeps_the_missing_sections_at_their_defaults`,
  `dw_2_2_a_saved_chest_round_trips_through_the_app` вЂ¦
- `crates/tr-app/src/player_file.rs` вЂ” 6 tests:
  `round_trip_through_the_crypto_and_the_sections`,
  `the_favorited_bits_survive_a_save_and_a_load`,
  `the_armor_and_dye_favorites_are_version_gated`,
  `the_metadata_carries_the_player_type`, `the_caps_clamp_on_read`,
  `the_vanilla_key_is_the_utf16_of_h3y_guyz`.
- **No test compares the Rust output against a C#-written file** вЂ” every fixture is
  either hand-computed in Rust or a Rust round-trip. That is the core gap behind
  the deviations below.

## Deviations & risks

1. **The `.wld` tile section drops the four wire bits** (C# `b2` 0x02/0x04/0x08,
   `b3` 0x20) вЂ” `crates/tr-core/src/wld/sections/tiles.rs:103-105` states it
   explicitly: the port keeps wires in a side table (`World.wires`, used by the
   `.trwld` v6 tail) and its `.wld` reader never restores them.
   Severity: **behavioural** (data loss on a `.wld` round trip + no byte parity).
   Consequence 2: the C#'s batching equality (`Tile.isTheSameAs:155-197`, which
   compares `sTileHeader`/`bTileHeader`, i.e. the wire bits) differs from the
   port's `encoded_same` (`tiles.rs:219-259`), so the **run boundaries** differ
   too в†’ the byte divergence is not only in the header bits.
   ROADMAP/ANALYSIS: the Rust module doc admits it locally; the M6-SAVE-1 ROADMAP
   row claims "all 11 `.wld` sections read + written" without this caveat.
2. **The `.wld` world header + flags write placeholder values** (generator
   version 0, GUID zeros, world id 0, `LastPlayed` = min date, the secret-seed
   booleans `false`) вЂ” `sections/header.rs:49-60`, `sections/flags.rs:220-274`.
   Severity: **behavioural** (a vanilla client reads a different seed/GUID/world id)
   but harmless inside the port. The deviations are documented per line in the Rust
   source; the ROADMAP row does not mention them.
3. **No rolling backups, no `.bak`, no post-write validation** вЂ” the C# keeps up
   to 9 numbered backups (`DoRollingBackups:966-1004`) and re-reads the file to
   validate it, restoring the `.bak` on failure (`InternalSaveWorld:941-962`).
   The port writes tmp+rename only (`wld::save:319-321`).
   Severity: **behavioural** (data-safety gap, a corrupted write is not rolled back).
4. **The `.plr` writer is a subset mirror** of `Player.Serialize` (sanitized name +
   starter items per the ROADMAP row) в†’ the port's `.plr` is not byte-comparable
   with a vanilla one. Severity: **behavioural** (interop); a vanilla client would
   read a partial character.
5. **Cloud save is missing** (`SocialAPI.Cloud`) вЂ” severity: **behavioural**, but
   out of scope by the module-38 non-goals; it should be an explicit ROADMAP
   non-goal rather than an open row.
6. **~~Autosave cadence parity is unverified~~ — REFUTED (S1-J-6,
   2026-11-13).** The 300 s `saveToonWhilePlaying` branch
   (`Main.cs:19743-19749`) is gated `!Main.gameMenu && Main.netMode == 1`
   (`:19739`) — a **multiplayer client, in-world** player+map save, unreachable
   at the port's `netMode == 0`. The menu arm (`:19767-19769`) only stops the
   timer and saves nothing. The port's 600 s branch (`!gameMenu && autoSave`) is
   the faithful single-player reduction, and `save_world` covers world + player
   + map.
7. **The port has two world formats** (`.wld` vanilla-shaped and `.trwld` native),
   chosen by file extension at save time (`saveglue.rs:27-28`) вЂ” the vanilla's
   single `.wld` is split. Severity: **cosmetic/architectural**, but it doubles the
   save surface for Stage-2 (two compressors, two readers).
8. The C#'s "temp" save snapshot (day/time/weather restored on load) — CLOSED
   by S1-J-7 (2026-11-13): the mechanism verified against the source and the rain
   triple ported as the `.trwld` v14 block вЂ” severity **UNKNOWN**.

## Task seeds

- **S** вЂ” write the four wire bits in `sections/tiles.rs::write_tile_record`
  (`b2` 0x02/0x04/0x08, `b3` 0x20) and add them to `encoded_same`; the source
  data already exists (`World.wires`, the `.trwld` v6 side table). Closes
  deviation 1 and makes the tile section byte-comparable.
- **M** вЂ” a golden-byte harness: one C#-written `.wld` (or one hand-computed
  fixture per section) + a test that asserts the Rust writer's bytes are
  identical. Today the only byte fixtures live inside the Rust tests
  (`wld/mod.rs:1200-1281`) and prove self-consistency, not C# parity.
- **S** вЂ” own the world id / GUID / generator version / `LastPlayed` in `World`
  (or in `WorldSide`) and write them; today they are hardcoded zeros/min-dates.
- **S** вЂ” `.plr` byte parity: extend `player_file.rs::write_player_bytes` to the
  full `Player.Serialize` field list (`Player.cs:54750+`) and pin it with a
  C#-produced fixture.
- **S** вЂ” rolling backups + the post-write `ValidateWorld` rollback, mirroring
  `DoRollingBackups` + `InternalSaveWorld:941-962` (or document the omission in
  ANALYSIS).
- **S** вЂ” close the cloud-save row as an explicit non-goal (module 38) and drop
  it from the Saves status line.
- **~~S~~ — verify the autosave tick against `Main.cs:19737-19765`. DONE
  (S1-J-6, 2026-11-13, refuted):** the 600 s branch (`!gameMenu && autoSave`) is
  the single-player path and is ported; the 300 s branch is `netMode == 1` only
  (multiplayer client), and the menu arm only stops the timer.
- **M (Stage-2)** вЂ” the compressor seam: add a `Compression` switch to
  `wld::to_bytes` that deflates the tile-section body (and optionally the
  flags/header bodies) and bumps `FORMAT_VERSION` to 327, keeping the 326 path
  byte-identical; the vanilla then rejects the compressed file cleanly
  (`WorldFile.cs:196-200`). Measure the ratio on a real `.wld`.
- **S (Stage-2)** вЂ” the RLE/VLQ predictor stage over the already-RLE'd tile
  stream (column-wise delta + VLQ) before deflate, with a round-trip equality
  test on a real world; the predictor must be a pure function of the section
  bytes so the 326 path is untouched.
- **S (Stage-2)** вЂ” the `.plr` hook: compress the plaintext before
  `cbc_encrypt`, decompress after `cbc_decrypt` (port-only files).
- **S (Stage-2)** вЂ” background save: move `wld::save`'s `std::fs::write` onto a
  worker thread (keep `to_bytes` on the main thread; the tmp+rename is already
  atomic). No vanilla-visible change.
- **S** вЂ” documentation: the ROADMAP "Saves" status line still says `.trwld v10`
  (now v13) and "favourites round-trip broken" (BUG-03 is closed) вЂ” refresh it,
  and add the wire-bit and header-value deviations to the M6-SAVE-1 row.
