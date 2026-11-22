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
- `tr-core/src/tileobject.rs` — the system (`can_place`/`place`, `def_for_tile`
  ≈ `GetTileData`, `base_def` ≈ `SetupBaseObject` `TileObjectData.cs:1743-1803`);
  the DATA lives in GENERATED `tr-core/src/tileobject/registry.rs` (S1-F-7 wave A,
  `tools/GenTileObjectData/gen.py`): all 389 `addTile` kinds, 604 defs.
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
| `Framing.SelfFrame8Way` + `BlockStyle` + `WillItBlend` | PORTED (W4-d110, code-only) | `framing.rs::self_frame_8way` | full 47-row lookup + style gates; dispatch covers 255-268 AND the 385/446/447/448 `goto case 255` fallthrough (`WorldGen.cs:81409-81413`/`:84818`); witnesses `w4_d110_*` written NOT run |
| `SquareTileFrame` / `SquareWallFrame` (3×3 refresh) | PARTIAL | `framing.rs:1132-1175` (`reframe_area`) | square re-frame, radius clamped 1..4 (superset of 3×3) |
| `Framing.WallFrame` | PORTED | `framing.rs::wall_frame_of_variant` + `wall_frame_head` | `TruncatesWalls`/`invisibleWall` gates (S1-P-i33), `wallLargeFrames` arms + wall-21 roll (S1-A-3), wall-id clamp + wall==0 paint/coating reset (W4-d110, code-only) |
| `wallFrameLookup` / `centerWallFrameLookup` | PORTED | `framing.rs:1069-1093` | 20×3 table + the mask-15 offset add |
| phlebas/lazure wall lookups (`Framing.cs:35-39`, `:378-387`) | NOT-FOUND | — | large-frame walls keep the generic mask |
| `TileObjectData` registry | PORTED (generated) | `tileobject/registry.rs` (GENERATED) | S1-F-7 wave A 2026-11-18: `tools/GenTileObjectData/gen.py` interprets `Initialize` `:2110-5014` 1:1 — all 389 tiles / 604 defs; freeze `tests/tileobject_registry_freeze.rs`, `gen.py --check`; W4-d111 re-census 2026-11-20 confirmed 0 missing `addTile` ids (witness pins `tests/tileobject_registry_witness.rs`) |
| `TileObjectData` style alternation (`RandomStyleRange`, `GetStyle_Detritus/SmallPiles/Stalactite` `:5294-5312`) | PARTIAL | `tileobject.rs::base_def` + `edit.rs:2600-2612` | base/style routing ported; dynamic stalactite/detritus arms not found |
| `GetTileData` / `CustomPlace` / `TryGetTileBounds` / coordinate pickup | PORTED (subset) | `tileobject.rs` (tests `get_tile_data_resolution`, `coordinate_full_extents`) | `CoordinateFullWidth/Height` `:256-263` |

| `TileObjectData` registry | PARTIAL | `tileobject.rs:293-330` | 36 defs registered out of the C#'s full registry (`Initialize` `:2110`); ROADMAP:1319 admits "remaining: the full-registry generator" |
| `TileObjectData` style alternation (`RandomStyleRange`, `GetStyle_Detritus/SmallPiles/Stalactite` `:5294-5327`) | PORTED | `tileobject.rs::resolve_tile_def` + `apply_style_override` (W4-d112) + the `RandomStyleRange` draw (S1-F-7b) | the tile-based `GetTileData(Tile)` frameY consumer `:5196-5261` ported 1:1 (thresholds 36/18/36-72-90); witnesses `tests/tileobject_resolve_witness_d112.rs` written-not-run |
| `GetTileData` / `CustomPlace` / `TryGetTileBounds` / coordinate pickup | PORTED (subset) | `tileobject.rs` (`get_tile_data` = the (type,style,alternate) overload `:5126-5161`, `resolve_tile_def` = the Tile overload `:5196-5261`; tests `get_tile_data_resolution`, `coordinate_full_extents`) | `TryGetTileBounds` `:5163-5194` still unported (coordinate pickup consumer) |

| `GetTileData` / `CustomPlace` / `TryGetTileBounds` / coordinate pickup | PORTED (subset) | `tileobject.rs` (tests `get_tile_data_resolution`, `coordinate_full_extents`; W4-d113: `try_get_tile_bounds` — the :5163-5194 coordinate walk, resolver-parameterised on d112's `GetTileData(Tile)` seam, tests `w4_d113_walk_*`) | `CoordinateFullWidth/Height` `:256-263` |

| `GetTileData` / `CustomPlace` / `TryGetTileBounds` / coordinate pickup | PORTED (subset) | `tileobject.rs` (tests `get_tile_data_resolution`, `coordinate_full_extents`) | `CoordinateFullWidth/Height` `:256-263`; **W4-e12**: the int overload's alternate tail corrected 1:1 (:5150-5159 — an out-of-range alternate KEEPS the resolved def, the port answered None; consumers `TileObject.Place` :31 / `DrawPreview` :859; witnesses `test_w4_e12_*` written-not-run). The Tile overload :5196-5261 + bounds :5163-5194 are the d112/d113/e11/d114 branch claims |

| `TileObjectData` style alternation (`RandomStyleRange`, `GetStyle_Detritus/SmallPiles/Stalactite` `:5294-5327`) | PORTED | `tileobject.rs::get_tile_data_for_tile` + `resolve_from_frames` (W4-e11) | the frameY override arms + the `[Style, Style+RandomStyleRange]` bands (:5249-5258); witnesses `tileobject_frame_resolution_witness.rs` (written-not-run) |
| `GetTileData` / `CustomPlace` / `TryGetTileBounds` / coordinate pickup | PORTED | `tileobject.rs` (`get_tile_data`, `get_tile_data_for_tile` :5196-5261, `try_get_tile_bounds` :5163-5194, `check_water_death`/`check_lava_death` :5083-5119 — W4-e11) | `CoordinateFullWidth/Height` `:256-263`; the `Liquid.cs:1190` settlement-kill wiring is the liquid task's seam |
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
- `tests/tileobject_registry_freeze.rs` (2): the generated registry's FNV-1a pin
  (389 tiles / 604 defs) + the `BY_KIND` lookup census.
- `tests/tileobject_registry_witness.rs` (7, W4-d111 2026-11-20): the full
  389-kind `addTile` census vs the C# `Initialize` scan, plus verbatim def pins
  for kinds 19/10/11/3/165/171/185 (sub-tiles, alternates, hooks, style-override
  tags) — batch 1 of the d11x series covers the first 120 registered kinds
  (3..=270); d112+ start at kind 271.

- `tests/tileobject_resolve_witness_d112.rs` (10, W4-d112 2026-11-20, WRITTEN
  NOT RUN — code-only lane): the `resolve_tile_def` frameY consumer
  (`GetTileData(Tile)` :5196-5261) — the three `GetStyle_*` override thresholds
  incl. boundaries (233/185/165), chair 15 multiplier + sub-tile-own
  alternates, door 10 line-skip re-decode + style-19 sub-tile, lamp 93
  zero-wrap line-skip, sunflower 20 overlapping `[Style, Style+Range]`
  windows (inclusive upper bound, first-match), torch 4 sub-tile→alternate
  walk, the 3-kind override tag census.
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
   (framing.rs no longer has `variant`/`wall_variant`). **W4-d109 (2026-11-20) added the
   save half**: the port's `.trwld` v18 tail persists `World::variants` and a v18 load
   reuses the stored values in the full passes (`World::variants_from_save`) — a
   port-only extension, since the C# `.wld` never serializes the bits (the refutation
   above stands for the vanilla file). ANALYSIS "Autotiling" records the old→new.

- **~~`Framing.SelfFrame8Way` not ported~~ — RESOLVED by W4-d110 (2026-11-20, code-only)**.
  `self_frame_8way` + `find_block_style`/`will_it_blend` + the const 256-mask lookup port
  `Framing.cs:9-23/92-98/141-191/193-324` verbatim; dispatch in `frame_of_variant` covers
  255-268 AND 385/446/447/448 — the earlier claim "sole caller ids 255-268" missed the
  inner-switch `case 385/446/447/448: break` + `goto case 255` fallthrough
  (`WorldGen.cs:81409-81413`, `:84818`). Variant rolls stay with the d109/S1-A-3 pass
  mechanism. Deviation: null lookup rows (sloped/half gemspark masks with no
  `Add8WayLookup` entry) return `None` instead of the C#'s throw (ANALYSIS "Autotiling").
  Witnesses `w4_d110_*` written NOT run.
- **~~`WallFrame` incomplete~~ — RESOLVED (S1-P-i33 + S1-A-3 + W4-d110)**. The
  `TruncatesWalls[tile.type]` neighbour condition and the `invisibleWall()` gate landed
  with S1-P-i33 (`wall_mask_neighbour`); the `wallLargeFrames == 1/2` phlebas/lazure arms
  with their re-store and the wall-21 third-variant roll with S1-A-3; the wall-id clamp
  (`wall >= WallID.Count` → 0, `Framing.cs:332-335`) and the `wall == 0` →
  `wallColor(0)` + `ClearWallPaintAndCoating()` reset (`:337-342`) with W4-d110
  (`wall_frame_head`, code-only). Remaining deviation:
  `ShouldShowInvisibleBlocksAndWalls()` reads as false (unmodelled, the
  `render/tile_cracks.rs` precedent), and `frame_walls` runs the heads in the copy phase
  after the parallel mask reads (ANALYSIS "Autotiling" item 7).
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
- **~~`TileObjectData` registry is hand-maintained~~ — RESOLVED by S1-F-7 wave A
  (2026-11-18).** The 36-def hand `TABLE` is retired; `tileobject/registry.rs` is
  GENERATED by `tools/GenTileObjectData/gen.py` (all 389 `addTile` kinds, 604 defs
  incl. sub-tiles/alternates, the copy-on-write `Initialize` chain interpreted
  statement-by-statement). Never hand-edit it; `gen.py --check` + the
  `tileobject_registry_freeze` FNV pin guard drift. W4-d111 (2026-11-20) re-censused
  the C# `addTile` ids against the registry: 389/389 present, 0 missing — the
  "port ~120 missing defs" task series (d111-d114/e11/e12) is a no-op for data;
  d111 landed witness pins instead (`tests/tileobject_registry_witness.rs`,
  batch range = first 120 registered kinds 3..=270; d112+ start at 271).
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
- **~~M — Write a `TileObjectData` generator~~ — DONE (S1-F-7 wave A):**
  `tools/GenTileObjectData/gen.py` parses `TileObjectData.Initialize` (`:2110`)
  into the GENERATED `tileobject/registry.rs`; `RandomStyleRange` alternation
  landed with wave B. Remaining: the `GetStyle_Detritus/SmallPiles/Stalactite`
  (`:5294-5312`) frameY resolution — the tags are generated data
  (`get_style_override`), the tile-based `GetTileData(Tile)` consumer
  (`:5196-5261`) is the open half. (`reframe_area` keeps object frames —
  tested by `test_DW_1_6`.)

- **M** — Write a `TileObjectData` generator (extend or clone `tools/GenTileData`) that
  parses `TileObjectData.Initialize` (`:2110`) into the `TABLE` consts; then the style
  alternation arms `GetStyle_Detritus/SmallPiles/Stalactite` (`:5294-5312`) and
  `RandomStyleRange` alternation. (Depends on the S tasks above only insofar as
  `reframe_area` must keep object frames — already tested by `test_DW_1_6`.)
  **Status:** generator DONE (S1-F-7 wave A, `tools/GenTileObjectData/gen.py`);
  `RandomStyleRange` DONE (S1-F-7b); the `GetStyle_*` frameY resolution DONE
  (W4-d112, `tileobject::resolve_tile_def` — the `GetTileData(Tile)` consumer
  `:5196-5261`; witnesses written-not-run). Remaining: wire the framing /
  liquid-death consumers onto the resolver + `TryGetTileBounds` `:5163-5194`.

  **DONE**: the generator landed with S1-F-7 wave A (389 tiles/604 defs,
  d111 census proved it complete); the `RandomStyleRange` runtime hook with
  wave B (S1-F-7b); the tile-based `GetTileData(Tile)` frameY resolution
  (:5196-5261, the override arms :5294-5327) with **W4-e11** —
  `tileobject::get_tile_data_for_tile` + `try_get_tile_bounds` + the
  tile-based `check_water_death`/`check_lava_death`, witnessed by
  `tests/tileobject_frame_resolution_witness.rs` (written-not-run).
- **S** — Audit `tiledata` coverage: list `TileID.Sets.*` actually consumed across
  `crates/`, diff against `tiledata/sets.rs`, migrate hand-copied sets (e.g.
  `wind.rs`'s `SwaysInWindBasic`) into the generator.
- **M** — Golden framing test: load a reference `.wld`, run `frame_world`/`frame_walls`,
  pin `frame_x/frame_y/wall_frame` for a fixed coordinate set (catches both the rule tables
  and the variant policy). Depends on the `frameNumber` decision above.
- **S** — Perf hardening (optional, behaviour-frozen): replace `WireState`'s four
  `HashSet`s with an index-keyed bitset (same `get/set` API) — measure with `TR_STATS`.

## W4-e12 — TileObjectData remainder census (2026-11-20, code-only)

> Sixth/remainder lane of the TileObjectData batch (d111–d114/e11 before). The
> "missing defs" premise of the row above is **STALE**: S1-F-7 wave A already
> generated the FULL registry. Re-verified against source, not docs:
>
> - `python tools/GenTileObjectData/gen.py . --check` → `registry.rs is current
>   (389 tiles, 604 defs)`.
> - The distinct `addTile` kinds of `Initialize` (`TileObjectData.cs:2110-5014`,
>   incl. the `435..439` loop `:2149-2160`) == the registry's top-level kinds
>   exactly: **389 == 389, set-difference empty both ways**. Zero placement defs
>   are missing.
>
> **The genuinely missing TileObjectData DATA is the draw-field class.** `gen.py`
> PARSES `DrawXOffset/DrawYOffset/DrawFlipHorizontal/DrawFlipVertical/DrawStepDown`
> (`TileObjectData.cs:778-941`), `DrawStyleOffset` (`:1637-1671`),
> `DrawFrameOffsets` (`:1460-1501`) and the two visual overrides
> `StyleWrapLimitVisualOverride`/`styleLineSkipVisualOverride` (`:1042-1106`)
> through the same copy-on-write machinery, but does NOT emit them (the
> documented S1-F-9 DrawData remainder, `tileobject.rs:50-55`). 254 top-level
> kinds carry a non-default draw value. The sole C# consumer is
> `TileObject.DrawPreview` (`TileObject.cs:855-953`): `num5 = DrawYOffset` `:864`,
> `drawXOffset = DrawXOffset` `:865`, `num3 += DrawStyleOffset` `:866`, the two
> visual overrides `:869-876`, the `DrawStepDown` solid-above bump `:900-903`,
> the 567 top-row special `:904-907`, the parity flips `:929-936`, and the
> `DrawFrameOffsets[i,j]` source-rect add `:946-953`. Emitting these fields is a
> generator + struct + freeze-hash change owned by S1-F-9, NOT this data lane.
>
> **This lane's range (collision-free with the sibling batches):** emission
> indices **501–604** of `registry.rs` = the last **104 defs**, owned by tile
> kinds **605–753** (76 kinds). Verified kind-disjoint: every def owned by those
> kinds falls exactly in the tail (no def of an earlier kind leaks in, no tail
> def belongs to an earlier kind). Deliverable = a verbatim per-def witness
> (`crates/tr-core/tests/tileobject_e12_defs.rs`) pinning the range's placement
> identity fields against the C#-interpreted values, plus the draw-field
> remainder table below. Data-only; no production behaviour touched; RNG
> untouched (pure data). Witnesses WRITTEN NOT RUN (code-only mode).
>
> Draw-field remainder for the range (the non-default values, verbatim from the
> interpreted `Initialize`; `Y`=`DrawYOffset`, `X`=`DrawXOffset`, `FH`=
> `DrawFlipHorizontal`, `FO`=`DrawFrameOffsets`, `WVO`/`LSVO`= the visual
> overrides). All other range kinds carry the `SetupBaseObject` defaults
> (`:1793-1796`, X/Y/step 0, flips false) and null `DrawFrameOffsets`:
>
> | kind(s) | draw values | C# anchor |
> |---|---|---|
> | 605–614, 619–623, 629, 632, 640, 642–645, 647–654, 657, 658, 699, 705, 706, 710, 720, 721, 725, 753 | Y=2 | the per-kind `DrawYOffset = 2` writes |
> | 615, 637, 703 | FH=true | `:4964`/`:4324`/`:4296` chains |
> | 617 (+A1) | Y=2, WVO=2, LSVO=0 | `:3426-3435` |
> | 646 | Y=−4 | `:2373` chain |
> | 660 | Y=−2; A1 Y=−10 | `:2464-2468` |
> | 698 (+S1/S2/A1s) | X=10, Y=−2 (alts Y=−10), FO=[[0,0,20,0],[0,−2,20,28]] (sub1 X+20, sub2 X+40) | `:2470-2502` |
> | 700 | Y=2, FH=true | `:4268` chain |
> | 701 | Y=−16, FH=true | `:4280` chain |
> | 711 | FO=[[0,0,0,0],[0,0,2,0]],[[0,0,0,2],[0,0,2,2]] | `:3638-3649` |
> | 723/724 | base Y=2; A1 Y=−2, A2 X=2, A3 X=−2 | `:4517`/`:4549` chains |
> | 733 | Y=2, FO=3×3 all [54,0,0,0] | `:4125` chain |
> | 751 | X=9, Y=−8, FO=[[0,0,34,32],[0,0,−16,−16]],[[0,0,−16,−16],[0,0,−16,−16]] | `:4015` chain |
