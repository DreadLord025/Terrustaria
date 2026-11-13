# 08 — Tile data & framing

> Wave 1 · researched 2026-11-12 · sources: `decompiled/` (Terraria 1.4.5.8), `crates/`, `docs/ROADMAP.md`

## Scope

The tile record itself plus everything that decides how a tile draws against its
neighbours. **C#:** `Terraria/Tile.cs` (1 000+ lines: the type/wall/liquid fields and
the packed header accessors), `Terraria/Framing.cs` (346 lines: `SelfFrame8Way`,
`WallFrame`, the 8-way/wall frame lookup tables built in `Initialize`),
`Terraria/WorldGen.cs::SquareTileFrame` (`:80185-80196`) / `SquareWallFrame`
(`:80198-80209`), `Terraria/WorldGen.cs::TileFrameCosmetic` (`~:81035-84836`, the
merge-framing switch that dominates real framing), and
`Terraria/ObjectData/TileObjectData.cs` (5 328 lines: the multi-tile placement
registry + style alternation). **Rust:** `crates/tr-core/src/tile.rs` (177 lines),
`crates/tr-core/src/framing.rs` (1 416 lines), `crates/tr-core/src/tileobject.rs`
(3 825 lines), `crates/tr-core/src/tiledata/` (GENERATED), plus the save-side header
decode in `crates/tr-core/src/wld/mod.rs:441-564`.

## Key entities

- `Terraria/Tile.cs::Tile` fields `:8-24` — `type`, `wall`, `liquid`, `sTileHeader`,
  `bTileHeader`, `bTileHeader2`, `bTileHeader3`, `frameX`, `frameY`. Header bits:
  colour=`sTileHeader&0x1F` (`:579`), active=`0x20` (`:589`), inActive=`0x40` (`:606`),
  wire=`0x80` (`:623`), wire2=`0x100` (`:640`), wire3=`0x200` (`:657`), halfBrick=`0x400`
  (`:674`), actuator=`0x800` (`:691`), slope=`0x7000` (`:708`), fullbrightWall=`0x8000`
  (`:718`); liquidType=`bTileHeader&0x60>>5` (`:235`), wallColor=low 5 bits (`:335-343`);
  bTileHeader2 = wallFrameX nibble (`:454`), frameNumber bits 4-5 (`:464`), wallFrameNumber
  bits 6-7 (`:474`); bTileHeader3 = wallFrameY bits 0-2 (`:484`), checkingLiquid (`:494`),
  skipLiquid (`:511`), invisibleBlock (`:528`), invisibleWall (`:545`), fullbrightBlock (`:562`).
- `Terraria/Framing.cs::SelfFrame8Way` `:193-324` — the BlockStyle/`WillItBlend` (`:180`) 8-way
  framer; rolls `WorldGen.genRand.Next(0,3)` per tile (`:319`). **Only caller:**
  `WorldGen.cs:84833` (the 255-268 arm of `TileFrameCosmetic`).
- `Terraria/Framing.cs::WallFrame` `:326-408` — 4-neighbour mask (+diagonals via
  `centerWallFrameLookup` `:401-404`), `wallLargeFrames` phlebas/lazure arms (`:378-387`),
  the wall-21 extra roll (`:391-394`), and the wall==0 paint reset (`:337-342`).
- `tr-core/src/tile.rs::Tile` `:15-32` — 12-byte packed `{kind,wall,frame_x,frame_y,liquid,flags,meta,wall_frame}`;
  `meta = slope | paint<<3`, `wall_frame = col | row<<4`, accessors `:34-150`.
- `tr-core/src/framing.rs::frame_of` `:801-1004` — the merge framer: `framing_type`
  (`:112`), `neighbour` (`:126`), 20-rule table `RULES` (`:42-108`), set-family merges
  (`:913-970`), grass family branch `grass_frame_of` (`:224-798`).
- `tr-core/src/framing.rs::frame_world` `:1010-1063`, `reframe_area` `:1132-1175`,
  `wall_frame_of` `:1106-1128`, `frame_walls` `:1179-1199`.
- `tr-core/src/tileobject.rs::TABLE` `:293-330` — 36 `TileObjectDef`s; `TileObjectDef::211`;
  `base_def` (`:456`) = `TileObjectData.SetupBaseObject` (`TileObjectData.cs:1743-1803`).
- `crates/tr-core/src/tiledata/` — `mod.rs` (hand-written; `TILE_COUNT=754`, `WALL_COUNT=367`),
  generated `sets.rs` (46 `TileSet`s), `merge.rs` (`MERGE` dense bitset, `LARGE_FRAMES`,
  `GRASS_DIRT`), `emissive.rs` (`EMISSIVE[754]`, `WALL_LIGHT[367]`); generator
  `tools/GenTileData/Program.cs` (reads `TileID.cs`, `Main.cs`, `WorldGen.cs`, guards
  `TileID.Count == 754`).

## Dependencies

- Calls into: `tiledata::{sets,merge,emissive}` (framing), `tile.rs`, `world.rs`
  (`World::tile/index`), `torch::check_torch` (`framing.rs:1173`), `wiring.rs` (wire state),
  `edit.rs` (placement hooks), `growth.rs`/`living_trees.rs`/`spider_gem_moss.rs` (worldgen).
- Called by: `tr-app/src/app/mod.rs:3138-3141` (`frame_world` + `frame_walls` at world load),
  `worldgen/pipeline.rs:3051`, `edit.rs:2626/3376/3483/3570/3575` and `tileobject.rs:2161/3134`
  (`reframe_area`, radius 2), `growth.rs` (radius 1/3), `worldgen/desert.rs`.
- Shared data structures: `Tile` (12 B), `World::wires: WireState` (4 `HashSet`s,
  `wiring.rs:91-96`), `World::extras: HashMap<u32,u8>` (wall paint + coatings,
  `world.rs:20`), `World::frame_important`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Tile` field set (type/wall/liquid/frameX/frameY + slopes/paint) | PORTED | `tile.rs:15-32` | all render-relevant fields; see deviations for packing |
| wire bits (`Tile.cs:623-672`, `wire4` `:437`) | PORTED, relocated | `world.rs:25`, `wiring.rs:91-96` | 4 colours kept; moved out of the 12-byte tile |
| invisibleBlock/invisibleWall/fullbrightWall | PORTED, relocated | `world.rs:20` (`extras`), `wld/mod.rs:554-563` | coatings ride the sparse extras byte |
| checkingLiquid/skipLiquid (`:494-527`) | NOT-FOUND | — | no equivalent state found; liquid sim uses its own path |
| slope-only side effects in framing (`TileFrameCosmetic` slope masks) | PORTED | `framing.rs:818-864` | sloped neighbours do not contribute an open side |
| Merge framing (`TileFrameCosmetic`, `TileMergeAttempt` `WorldGen.cs:81035`) | PORTED | `framing.rs:896-970`, `tiledata/merge.rs:2334` | generated directed merge bitset + set families |
| `selfFrame8WayLookup`-style rule table | PORTED (substitute) | `framing.rs:42-108` | 20 rules over the 8-bit neighbour mask |
| Grass family (5 override chains, `-2` marker) | PORTED | `framing.rs:224-798` | chains A/B/C mirror `WorldGen.cs:82216-83184` |
| `frameNumber` per-tile RNG (`genRand` `:319`, `:390`) | PARTIAL | `framing.rs:138`, `:1097` | replaced by a position hash (documented) |
| `Main.tileLargeFrames` variant override | PORTED | `framing.rs:988-995`, `tiledata/merge.rs:10` | `LARGE_FRAMES` 0/1/2 arms |
| `frame_world` / `Main.tileFrameImportant` gate | PORTED | `framing.rs:1010-1063` | band-parallel, skips important tiles |
| `Framing.SelfFrame8Way` + `BlockStyle` + `WillItBlend` | NOT-FOUND | — | no `self_frame`/`eight_way` symbol in `crates/`; sole caller `WorldGen.cs:84833` (ids 255-268) |
| `SquareTileFrame` / `SquareWallFrame` (3×3 refresh) | PARTIAL | `framing.rs:1132-1175` (`reframe_area`) | square re-frame, radius clamped 1..4 (superset of 3×3) |
| `Framing.WallFrame` | PARTIAL | `framing.rs:1106-1128` | missing `TruncatesWalls`/`invisibleWall` gates, `wallLargeFrames` arms, wall-21 roll, wall==0 reset |
| `wallFrameLookup` / `centerWallFrameLookup` | PORTED | `framing.rs:1069-1093` | 20×3 table + the mask-15 offset add |
| phlebas/lazure wall lookups (`Framing.cs:35-39`, `:378-387`) | NOT-FOUND | — | large-frame walls keep the generic mask |
| `TileObjectData` registry | PARTIAL | `tileobject.rs:293-330` | 36 defs registered out of the C#'s full registry (`Initialize` `:2110`); ROADMAP:1319 admits "remaining: the full-registry generator" |
| `TileObjectData` style alternation (`RandomStyleRange`, `GetStyle_Detritus/SmallPiles/Stalactite` `:5294-5312`) | PARTIAL | `tileobject.rs::base_def` + `edit.rs:2600-2612` | base/style routing ported; dynamic stalactite/detritus arms not found |
| `GetTileData` / `CustomPlace` / `TryGetTileBounds` / coordinate pickup | PORTED (subset) | `tileobject.rs` (tests `get_tile_data_resolution`, `coordinate_full_extents`) | `CoordinateFullWidth/Height` `:256-263` |
| `tiledata` sets | PARTIAL | `tiledata/sets.rs` | 46 `TileSet`s vs 152 `Factory.CreateBoolSet(` sites in `TileID.cs` (upper bound: nested/Conversion sets) |
| `tiledata` merge relation | PORTED | `tiledata/merge.rs` | dense 754×12-word bitset, generated |
| emissive tile/wall tables | PORTED | `tiledata/emissive.rs` | full coverage 754 + 367 |

## Tests

- `framing.rs` (8): `block_interior_uses_plain_frames`, `block_merges_dirt_with_stone`,
  `important_tiles_are_left_alone`, `walls_pick_lookup_cells`, `lone_grass_over_dirt_blends`,
  `grass_step_wraps_over_dirt`, `enclosed_grass_uses_checkerboard`,
  `grass_between_dirt_blends_both_ways`.
- `tile.rs` (1): `packed_tile_is_twelve_bytes` (size + accessor pin).
- `tileobject.rs` (~18): `test_DW_1_1_wooden_set_geometry_matches_csharp`,
  `test_DW_1_1_def_for_tile_lookup`, `test_DW_1_1_coordinate_full_extents`,
  `test_DW_1_2_can_place_*` (free/occupied, bounds, anchor, direction, liquid, cut tiles, anchor
  type), `test_DW_1_1_chest_cannot_be_placed_in_lava`, `test_DW_1_3_place_*` (writes frames,
  style frames, failed place, kills cut tiles), `test_DW_1_4_door_*`, `test_DW_1_5_module_doc_maps_csharp_fields`,
  `test_DW_1_6_reframe_area_keeps_object_frames`, `test_calculate_placement_style_matches_csharp`,
  `test_get_tile_data_resolution`, `test_is_valid_anchor_lists`.
- No test pins the `frameNumber`/`wallFrameNumber` variant question, and no golden
  `.wld`-load framing test exists.

- **~~`frameNumber` is a hash, not the stored/RNG value~~ — RESOLVED by S1-A-3 (2026-11-13)** with a
   correction: the claim above that the C# "stores per-tile `frameNumber`/`wallFrameNumber`
   in the save" is **wrong** — `SaveWorldTiles` (`WorldFile.cs:1432-1604`) and
   `LoadWorldTiles` (`:2519-2685`) never touch those bits; they live only in the runtime
   `Tile.bTileHeader2` bits 4-7 (`Tile.cs:464-482`) and reset on load. The port now carries
   the same mechanism: `World::variants` (2+2 bits), the reset roll from per-band
   `UnifiedRandom` streams in the full passes and `World::frame_rng` at the edit sites
   (the ANALYSIS "Autotiling" section records the deliberate stream deviations), the
   phlebas/lazure wall lookups with their re-store (which exposed the missing fourth
   `wallFrameLookup` column) and the wall-21 quirk. The hash functions are gone
   (framing.rs no longer has `variant`/`wall_variant`).

- **`Framing.SelfFrame8Way` not ported** — *behavioural*. Its only caller is
  `WorldGen.cs:84833` for ids 255-268; those tile ids get the generic merge path in the
  port (no `self_frame` symbol exists). Touched ids include torch/statue-family frames.
- **`WallFrame` incomplete** — *behavioural/pixel*. Missing: the `TruncatesWalls[tile.type]`
  neighbour condition (`Framing.cs:348/356/364/372`), the `ShouldShowInvisibleBlocksAndWalls`
  / `invisibleWall()` gate (`:344`), and the `wall == 0` →
  `wallColor(0)` + `ClearWallPaintAndCoating()` reset (`:337-342`). Consequence: walls
  behind echo-coated/invisible blocks frame one variant off. (S1-A-3 landed the rest of
  this list: the `wallLargeFrames == 1/2` phlebas/lazure arms with their re-store and
  the wall-21 third-variant roll.)
- **Wiring + coatings moved out of `Tile`** — *cosmetic/structural*. `WireState` uses four
  `HashSet<(i32,i32)>` (`wiring.rs:91-96`) and wall paint/coatings use
  `World::extras: HashMap<u32,u8>` (`world.rs:20`, `wld/mod.rs:460-511`). Memory/perf
  caution vs the C# in-tile bits; the save round-trip re-packs them correctly per the
  reader's `WorldFile.cs` citations, but "unknown key" handling for extras is sparse by
  construction (absence = default).
- **`reframe_area` is a square, not the C# 3×3 ring, with a clamped radius** —
   *cosmetic*. `WorldGen.SquareTileFrame` frames exactly the 3×3 block
   (`WorldGen.cs:80185-80209`); `reframe_area` (`framing.rs:1132-1134`) clamps radius to
   1..4 and callers pass 2/3. Superset behaviour, extra work per edit; the wall side
  (framing.rs:1164-1169) writes through two `set_wall_frame` calls whose net result is
   correct but reads as a bug.
- **`tiledata` set coverage** — *gap*. 46 generated `TileSet`s vs 152
  `Factory.CreateBoolSet(` call sites in `TileID.cs`; many extras are the nested
  `Conversion.*`/sub-class factories, but sets needed outside framing are hand-copied
  (`tiledata/mod.rs` names `wind.rs`'s `SwaysInWindBasic`) — duplication drift risk.
- **`TileObjectData` registry is hand-maintained** — *gap*, admitted by ROADMAP:1319.
  36 consts in `TABLE` vs the C#'s `Initialize` table; no generator, so every added
  object style is a manual transcription with its own citation comments.
- **Parallel-framing claim** (`frame_world`/`frame_walls`, ROADMAP:1247 "outputs identical")
   — *verification gap*. The comment's premise ("frames depend only on kinds/flags") holds for
   the merge path; the hash `variant()` is position-only, so the band-parallel scheme is
   sound, but no golden test proves equality against the C# frames.

## Task seeds

- **S** — Port `Framing.SelfFrame8Way` + `BlockStyle` + `FindBlockStyle` + `WillItBlend`
  (`Framing.cs:9-19, 141-191`) and call it from the 255-268 arm of the ported
  `TileFrameCosmetic` (`WorldGen.cs:84819-84833`); test with a torch-family island.
- **S** — Finish `wall_frame_of`: `TruncatesWalls` gate, `invisibleWall` gate, wall-21 roll,
  wall==0 paint/coating reset (`Framing.cs:337-396`); unit test mask 15 across all 9
  `(i%3, j%3)` offsets against `Framing.cs:401-404`.
- **S** — Port the `wallLargeFrames == 1/2` phlebas/lazure lookups (`Framing.cs:35-39, 378-387`)
  into `wall_frame_of` (they override the variant, not the mask).
- **S** — Decide + document the `frameNumber` policy: decode the stored byte from the save
  (and use it when reframing) or keep the hash; add a test that a loaded world's frames are
  stable across two loads. (Depends on a shipped test `.wld`.)
- **M** — Write a `TileObjectData` generator (extend or clone `tools/GenTileData`) that
  parses `TileObjectData.Initialize` (`:2110`) into the `TABLE` consts; then the style
  alternation arms `GetStyle_Detritus/SmallPiles/Stalactite` (`:5294-5312`) and
  `RandomStyleRange` alternation. (Depends on the S tasks above only insofar as
  `reframe_area` must keep object frames — already tested by `test_DW_1_6`.)
- **S** — Audit `tiledata` coverage: list `TileID.Sets.*` actually consumed across
  `crates/`, diff against `tiledata/sets.rs`, migrate hand-copied sets (e.g.
  `wind.rs`'s `SwaysInWindBasic`) into the generator.
- **M** — Golden framing test: load a reference `.wld`, run `frame_world`/`frame_walls`,
  pin `frame_x/frame_y/wall_frame` for a fixed coordinate set (catches both the rule tables
  and the variant policy). Depends on the `frameNumber` decision above.
- **S** — Perf hardening (optional, behaviour-frozen): replace `WireState`'s four
  `HashSet`s with an index-keyed bitset (same `get/set` API) — measure with `TR_STATS`.
